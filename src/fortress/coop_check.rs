//! Staged paired support checks on the authored split room, not a campaign route.
use super::*;
use crate::{assets::Assets, movement::{Controls, FixedClock, FIXED_DT}};
use anyhow::ensure;

fn setup(assets: &mut Assets, map: &Bsp) -> Result<(Fortress, World, [Player; 2])> {
    let mut fortress = Fortress::load(assets, map, false)?;
    fortress.state.cinema.arrival_done = true;
    let mut world = World::from_bsp(map)?;
    world.set_dynamic(fortress.colliders().collect());
    let players = [vec3(-400.,-2112.,-191.96875),vec3(-304.,-2112.,-191.96875)].map(|feet| {
        let mut player = Player::new(feet); player.grounded = true; player
    });
    let floor = &fortress.objects.iter().find(|o| o.name == "split_middle").unwrap().collider;
    ensure!(players.iter().all(|p| world.body_clear(p.feet) && crate::moving::aboard(floor,p.feet)),
        "Invalid paired split-room fixture");
    fortress.event("Fortress1_Start_Split");
    Ok((fortress,world,players))
}

pub fn check(assets: &mut Assets) -> Result<()> {
    println!("Fortress co-op supports / build {}", env!("LOOKING_GLASS_BUILD"));
    let map = Bsp::parse(&assets.read("maps/fortress1.bsp")?)?;
    for fps in [30,60,144] {
        let (mut fortress,mut world,mut players) = setup(assets,&map)?;
        let start = players.each_ref().map(|p|p.feet);
        let saved = serde_json::to_value(fortress.snapshot())?;
        fortress.advance_players(0.,&map,&mut world,&mut players.each_mut(),&[])?;
        ensure!(saved == serde_json::to_value(fortress.snapshot())? && start == players.each_ref().map(|p|p.feet),
            "Paused split room moved");
        let mut clocks: [FixedClock;2] = Default::default();
        let mut mirror = Fortress::load(assets,&map,false)?;
        let mut mirror_world = World::from_bsp(&map)?;
        let mut solo = Fortress::load(assets,&map,false)?;
        solo.restore(&fortress.snapshot(),&map)?;
        let mut solo_world = World::from_bsp(&map)?;
        solo_world.set_dynamic(solo.colliders().collect());
        let mut solo_player = players[0].clone();
        let mut solo_clock = FixedClock::default();
        for frame in 0..fps*11 {
            let dt = 1. / fps as f32;
            let mut copies = players.clone();
            if frame % fps == 0 {
                let state = serde_json::from_slice(&serde_json::to_vec(&fortress.snapshot())?)?;
                mirror.restore(&state,&map)?;
                mirror_world.set_dynamic(mirror.colliders().collect());
                mirror.advance_players(dt,&map,&mut mirror_world,&mut copies.each_mut(),&[])?;
            }
            fortress.advance_players(dt,&map,&mut world,&mut players.each_mut(),&[])?;
            solo.advance(dt,&map,&mut solo_world,&mut solo_player,&[])?;
            if frame % fps == 0 {
                ensure!(copies.each_ref().map(|p|p.feet) == players.each_ref().map(|p|p.feet)
                    && serde_json::to_value(mirror.snapshot())? == serde_json::to_value(fortress.snapshot())?,
                    "Restored split-room carry diverged");
            }
            ensure!(solo_player.feet == players[0].feet && solo.state.split == fortress.state.split,
                "Paired split clock/carry differs from one-player advance");
            for (i,p) in players.iter_mut().enumerate() {
                ensure!(world.body_clear(p.feet),"Split room embedded rider {i} at {fps} Hz");
                clocks[i].advance(1./fps as f64,&world,p,Controls::default());
                ensure!(world.body_clear(p.feet),"Split-room rider physics embedded {i}");
            }
            solo_clock.advance(1./fps as f64,&solo_world,&mut solo_player,Controls::default());
        }
        ensure!(fortress.state.split == 1. && players.iter().zip(start).all(|(p,at)|
            (p.feet-at-Vec3::Y*220.).length()<0.2),"Split room did not carry both riders to its endpoint");
        println!("PASS Fortress split room {fps} Hz: both carried 220 units, one clock, pause and restore");
    }
    for blocked in 0..2 {
        let (mut fortress,mut world,mut players) = setup(assets,&map)?;
        let at = players[blocked].feet;
        let wall = Collider::box_bounds(at+vec3(-19.,PLAYER_HALF.y+1.,-4.),
            at+vec3(19.,PLAYER_HALF.y+10.,80.));
        for _ in 0..120 {
            fortress.advance_players(FIXED_DT,&map,&mut world,&mut players.each_mut(),std::slice::from_ref(&wall))?;
        }
        let stopped = players.each_ref().map(|p|p.feet);
        let split = fortress.state.split;
        for _ in 0..120 {
            fortress.advance_players(FIXED_DT,&map,&mut world,&mut players.each_mut(),std::slice::from_ref(&wall))?;
        }
        ensure!(split < 0.02 && split == fortress.state.split && stopped == players.each_ref().map(|p|p.feet),
            "Obstructed rider {blocked} did not stop both and the shared split clock");
        for _ in 0..120 { fortress.advance_players(FIXED_DT,&map,&mut world,&mut players.each_mut(),&[])?; }
        ensure!(players.iter().zip(stopped).all(|(p,at)|p.feet.y>at.y+15.),"Split room failed to resume");
        println!("PASS Fortress split room: obstruction of rider {blocked} stops both; removal resumes");
    }
    Ok(())
}
