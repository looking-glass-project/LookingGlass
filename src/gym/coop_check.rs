//! Native-map checks for paired bleacher motion and co-op lever ownership.
use super::*;
use crate::{coop::PlayerId, interaction::Interactions, movement::{Controls, FixedClock}};
use anyhow::ensure;

pub fn check(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for first in PlayerId::ALL {
        let mut world = World::from_bsp(map)?;
        let mut interactions = Interactions::load(map)?;
        interactions.set_entry(assets,map,"skool2",None)?;
        interactions.sync_coop(&mut world);
        let lever = interactions.gym.as_ref().context("Missing co-op gym")?.lever;
        let mut player = Player::spawn(&world, lever.translation + vec3(-45., 0., 48.))
            .context("Co-op gym lever approach blocked")?;
        for _ in 0..240 { player.tick(&world, Controls::default()); }
        let players = [player.clone(), player];
        let aim = (lever.point(vec3(36., 0., 26.)) - players[0].eye()).normalize();
        let (_,owner) = interactions.doors_pair(1./120.,map,&mut world,
            std::array::from_fn(|i| Some((&players[i],aim,i==first.index()))))?;
        ensure!(owner == Some(first) && interactions.used == 1
            && interactions.gym.as_ref().unwrap().used
            && interactions.school2.as_ref().unwrap().hold_bleachers(),
            "Gym lever did not belong to the activating co-op player");
        let before = serde_json::to_value(interactions.gym.as_ref().unwrap().snapshot())?;
        let mut players = players;
        interactions.movers_pair(1./60.,map,&mut world,&mut players.each_mut())?;
        ensure!(serde_json::to_value(interactions.gym.as_ref().unwrap().snapshot())? == before,
            "Bleachers moved before the original scene's release cue");
        println!("PASS co-op gym lever {first:?}: one activation, original scene owner and movement hold");
    }
    for fps in [30,60,144] {
        for blocker in 0..2 {
            let mut world = World::from_bsp(map)?;
            let mut gym = Gym::load(map)?;
            gym.used = true;
            world.set_dynamic(gym.colliders().collect());
            let mut players = [Player::new(vec3(2100.,-2560.,0.1)),
                               Player::new(vec3(2100.,-2512.,0.1))];
            players[blocker].feet = vec3(2240.,-2560.,0.1);
            for _ in 0..fps*10 {
                gym.advance_players(1./fps as f32,map,&mut world,&mut players.each_mut(),&[])?;
                ensure!(players.iter().all(|p| world.body_clear(p.feet)),
                    "Bleachers crushed player {blocker} at {fps} Hz");
            }
            ensure!(gym.steps.iter().any(|s| s.duration>0. && s.time<s.duration+1.),
                "Bleachers ignored the co-op blocker");
            let snapshot = serde_json::from_slice(&serde_json::to_vec(&gym.snapshot())?)?;
            let mut restored = Gym::load(map)?;
            restored.restore(&snapshot,map)?;
            let mut mirror_world = World::from_bsp(map)?;
            mirror_world.set_dynamic(restored.colliders().collect());
            players[blocker].feet = vec3(2100.,-2608.,0.1);
            let mut copies = players.clone();
            for _ in 0..fps*10 {
                gym.advance_players(1./fps as f32,map,&mut world,&mut players.each_mut(),&[])?;
                restored.advance_players(1./fps as f32,map,&mut mirror_world,&mut copies.each_mut(),&[])?;
            }
            ensure!(gym.steps.iter().all(|s| (s.origin-(s.base-Vec3::X*s.distance)).length()<0.01),
                "Bleachers did not resume after the co-op blocker left");
            ensure!(serde_json::to_value(gym.snapshot())? == serde_json::to_value(restored.snapshot())?
                && players.each_ref().map(|p| p.feet) == copies.each_ref().map(|p| p.feet),
                "Restored paired bleachers diverged");
            let mut clocks: [FixedClock;2] = Default::default();
            for (i,p) in players.iter_mut().enumerate() { *p=Player::new(vec3(2180.,-2560.+48.*i as f32,16.1)); }
            for _ in 0..fps*4 {
                for (clock,p) in clocks.iter_mut().zip(&mut players) {
                    clock.advance(1./fps as f64,&world,p,Controls{wish:Vec2::X,..Default::default()});
                    ensure!(world.body_clear(p.feet),"Paired bleacher climb embedded a player");
                }
                if players.iter().all(|p|p.feet.z>=190.) { break; }
            }
            ensure!(players.iter().all(|p| p.feet.z>=190.),"Both players cannot climb extended bleachers");
            println!("PASS co-op gym {fps} Hz blocker {blocker}: obstruction, resume, snapshot and both physical climbs");
        }
        let mut world = World::from_bsp(map)?;
        let mut gym = Gym::load(map)?;
        world.set_dynamic(gym.colliders().collect());
        gym.used = true;
        // Retracted rows overlap vertically. Stage two bodies only after a
        // moving tread is exposed, keeping measurable travel to test. Treads
        // extend in 20-unit increments, so a partly exposed row has less left.
        let mut outside = Player::new(vec3(2100.,-2560.,0.1));
        let mut landing = None;
        for _ in 0..480 {
            landing = gym.steps.iter().enumerate()
                .filter(|(_,s)|s.distance-(s.base.x-s.origin.x)>2.)
                .find_map(|(i,s)| {
                    let bounds = &map.models[s.model];
                    [-8.,0.,8.,16.,24.].into_iter().find_map(|offset| {
                        let a = vec3(bounds.min.x+offset,
                            (bounds.min.y+bounds.max.y)*0.5,bounds.max.z+0.04)+s.origin;
                        let pair=[a,a+Vec3::Y*48.];
                        pair.iter().all(|at| world.body_clear(*at) && crate::moving::aboard(&s.collider,*at))
                            .then_some((i,pair))
                    })
                });
            if landing.is_some() { break; }
            gym.advance(1./120.,map,&mut world,&mut outside,&[])?;
        }
        let (index,feet) = landing.context("No two riders fit on an exposed moving bleacher")?;
        let remaining = gym.steps[index].distance-(gym.steps[index].base.x-gym.steps[index].origin.x);
        let mut players = feet.map(Player::new);
        for _ in 0..fps*8 {
            gym.advance_players(1./fps as f32,map,&mut world,&mut players.each_mut(),&[])?;
            ensure!(players.iter().all(|p| world.body_clear(p.feet)),"Paired bleacher rider was embedded");
            ensure!((players[1].feet-players[0].feet-(feet[1]-feet[0])).length()<0.01,
                "Bleacher moved only one rider");
        }
        ensure!((players[0].feet-(feet[0]-Vec3::X*remaining)).length()<0.1,
            "Both riders did not travel with the extending bleacher");
        println!("PASS co-op gym {fps} Hz: two riders carried together through full extension");
    }
    Ok(())
}
