//! Staged native machinery/scene checks; these do not certify a full paired route.
use super::*;
use crate::{coop::{protocol::Visit, stage::Stage, world::Roster, PlayerId},
    inventory::Stats, level::CoopPlayer, movement::FIXED_DT};

fn free(a: &mut Assets, map: &Bsp) -> Result<(Garden,World)> {
    let mut g=Garden::load(a,map)?;
    let mut world=World::from_bsp(map)?;
    world.set_dynamic(g.colliders());
    let (eye,_)=crate::interaction::spawn(map,Some("garden3_start1"));
    let mut p=Player::spawn(&world,eye).context("Chase check spawn blocked")?;
    g.saved.skip=true;
    g.advance(FIXED_DT,map,&mut world,&mut p,&[])?;
    ensure!(g.saved.arrived && !g.scripted(),"Chase check did not finish introduction");
    Ok((g,world))
}
fn tick(g: &mut Garden,map: &Bsp,world: &mut World,p: &mut [Player;2],s: &mut [Stats;2],dt:f32)->Result<()> {
    let [a,b]=p.each_mut(); let [x,y]=s.each_mut();
    g.coop_advance_roster(dt,map,world,&mut [CoopPlayer{player:a,stats:x},CoopPlayer{player:b,stats:y}],&[])
}
fn stats()->[Stats;2] {std::array::from_fn(|_|Stats::for_level("garden3",Some("garden3_start1")))}
fn clocks(a: &mut Assets,map: &Bsp)->Result<()> {
    for hz in [30,60,144] {
        let (mut pair,mut world)=free(a,map)?;
        let (mut single,mut sw)=free(a,map)?;
        // Keep the clock probes outside the hazard's path. These are not traversal positions.
        let mut p=[Player::new(vec3(0.,0.,16000.)),Player::new(vec3(100.,0.,16000.))];
        let mut sp=p[0].clone(); let mut s=stats();
        for n in 0..hz*3 {
            tick(&mut pair,map,&mut world,&mut p,&mut s,1./hz as f32)?;
            single.advance(1./hz as f32,map,&mut sw,&mut sp,&[])?;
            ensure!(pair.snapshot()==single.snapshot() && pair.transforms()==single.transforms(),
                "Paired chase clock/geometry differs from single-player {hz}Hz tick{n}");
            if n==hz {
                let saved=pair.snapshot(); let bodies=serde_json::to_value((&p,&s))?;
                for dt in [0.,-1.,f32::NAN] {tick(&mut pair,map,&mut world,&mut p,&mut s,dt)?;}
                ensure!(pair.snapshot()==saved && serde_json::to_value((&p,&s))?==bodies,"Paused chase moved");
                let mut restored=Garden::load(a,map)?;
                restored.restore(&saved,map)?;
                ensure!(restored.snapshot()==saved && restored.transforms()==pair.transforms(),"Saved paired chase differs");
                pair=restored;
                world.set_dynamic(pair.colliders());
            }
        }
        println!("PASS Rolling Stones paired clock {hz}Hz: single-player parity, pause and restored future");
    }
    let (mut pair,mut world)=free(a,map)?;
    let (mut single,mut sw)=free(a,map)?;
    let mut p=[Player::new(vec3(0.,0.,16000.)),Player::new(vec3(100.,0.,16000.))];
    let mut sp=p[0].clone(); let mut s=stats();
    for n in 0..120*120 {
        tick(&mut pair,map,&mut world,&mut p,&mut s,FIXED_DT)?;
        single.advance(FIXED_DT,map,&mut sw,&mut sp,&[])?;
        if n%120==0 {
            ensure!(pair.snapshot()==single.snapshot() && pair.transforms()==single.transforms(),
                "Paired marble path/callback/geometry differs at tick{n}");
        }
    }
    ensure!(pair.saved.rock.node==56 && pair.saved.pillar.is_some() && pair.saved.second_quake.is_some()
        && pair.saved.ice.is_some() && pair.saved.end.is_none()
        && pair.saved.doors.iter().all(|d|d.latched && d.open),
        "Paired chase omitted a marble callback or gate");
    ensure!(pair.snapshot()==single.snapshot() && pair.transforms()==single.transforms(),"Final paired geometry differs");
    println!("PASS Rolling Stones full shared marble path, gates and collapse clocks match single-player");
    Ok(())
}
fn contacts(a: &mut Assets,map: &Bsp)->Result<()> {
    for victim in 0..2 {
        for dead_partner in [false,true] {
            let (mut g,mut world)=free(a,map)?;
            let mut s=stats();
            let mut p=[Player::new(vec3(0.,0.,16000.)),Player::new(vec3(100.,0.,16000.))];
            p[victim].feet=g.saved.rock.position-crate::collision::PLAYER_CENTER;
            if dead_partner {
                p[1-victim].feet=p[victim].feet;
                s[1-victim].damage(10000.);
            }
            let partner=serde_json::to_value((&p[1-victim],&s[1-victim]))?;
            tick(&mut g,map,&mut world,&mut p,&mut s,FIXED_DT)?;
            ensure!(!s[victim].alive() && p[victim].velocity.length()>0.,"Marble did not hit living slot{victim}");
            ensure!(serde_json::to_value((&p[1-victim],&s[1-victim]))?==partner,"Marble hit the wrong slot");
            ensure!(g.saved.damage==0. && g.saved.impulse==Vec3::ZERO,"Paired hit leaked into single-player feedback");
            let snap=g.snapshot(); let mut restored=Garden::load(a,map)?;
            restored.restore(&snap,map)?;
            ensure!(restored.snapshot()==snap,"Saved lethal contact differs");
            println!("PASS Rolling Stones marble victim{victim}, dead_partner={dead_partner}: personal lethal damage and saved contact");
        }
    }
    let (mut g,mut world)=free(a,map)?;
    let mut s=stats(); let at=g.saved.rock.position-crate::collision::PLAYER_CENTER;
    let mut p=[Player::new(at-Vec3::X*24.),Player::new(at+Vec3::X*24.)];
    for _ in 0..120 {tick(&mut g,map,&mut world,&mut p,&mut s,FIXED_DT)?;}
    ensure!(s.iter().all(|s|!s.alive()),"First marble victim shielded the second after death");
    println!("PASS Rolling Stones both marble contacts: dead bodies no longer shield living players");
    Ok(())
}
fn scenes(a: &mut Assets,map: &Bsp)->Result<()> {
    let visit=Visit{epoch:1,map:"garden3".into(),entry:Some("garden3_start1".into()),
        difficulty:Default::default(),combat_preview:false};
    for skip in [false,true] {
        let mut world=World::from_bsp(map)?;
        let mut stage=Stage::load(a,map,&mut world,&visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
        let mut roster=Roster::new(&world,eye,yaw,Stats::for_level("garden3",visit.entry.as_deref()))?;
        roster.players[1].stats.spend_will(17.);
        let resources=roster.players.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
        ensure!(stage.owner==Some(PlayerId::Blue),"Simultaneous arrival chose wrong owner");
        let mut waiting=roster.players[1].player.clone(); waiting.release_rope();
        let waiting=serde_json::to_value(waiting)?;
        for n in 0..120*30 {
            if n==12 {
                let snap=stage.snapshot(); let encoded=crate::coop::codec::encode(&serde_json::to_vec(&snap)?)?;
                let bodies=serde_json::to_value(&roster.players)?;
                stage.before_move(&mut roster,map,&mut world,&Default::default(),0.)?;
                stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&encoded)?)?,map,&mut world)?;
                ensure!(serde_json::to_value(stage.snapshot())?==serde_json::to_value(snap)?
                    && serde_json::to_value(&roster.players)?==bodies,"Paused/restored arrival differs");
            }
            if skip && n==24 {stage.interactions.skip_cinematic(map,&mut world,&mut roster.players[0].player,&mut stage.story)?;}
            stage.scene_step(&mut roster,map,&mut world,&Default::default())?;
            if stage.owner.is_none(){break;}
            ensure!(stage.owner==Some(PlayerId::Blue) && serde_json::to_value(&roster.players[1].player)?==waiting,
                "Chase arrival moved the waiting player");
        }
        ensure!(!stage.busy() && stage.owner.is_none(),"Chase arrival never ended");
        for (b,stats) in roster.players.iter().zip(resources) {
            ensure!(world.body_clear(b.player.feet) && serde_json::to_value(&b.stats)?==stats,"Reunion changed resources/clearance");
            b.player.validate_world(&world)?;
        }
        ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Reunion overlapped");
        // Exercise the same hook that production Arena invokes after the scene.
        let before=stage.snapshot();
        stage.before_move(&mut roster,map,&mut world,&Default::default(),FIXED_DT)?;
        ensure!(serde_json::to_value(stage.snapshot())?!=serde_json::to_value(before)?,"Shared chase stayed frozen after reunion");
        println!("PASS Rolling Stones arrival skip={skip}: frozen partner, saved scene, exact resources and shared chase start");
    }
    Ok(())
}
fn paired_gate_contacts(a: &mut Assets, map: &Bsp) -> Result<()> {
    use crate::{entity::Id, event::{Event, Input}, interaction::coop::{Contacts, Motion}};
    let doors=|stage:&Stage| {
        stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Garden>()).unwrap()
            .saved.doors.iter().map(|d|(d.open,d.latched)).collect::<Vec<_>>()
    };
    for first in PlayerId::ALL { for id in [41,48,47,73] {
        let visit=Visit{epoch:1,map:"garden3".into(),entry:Some("garden3_start1".into()),
            difficulty:Default::default(),combat_preview:false};
        let mut world=World::from_bsp(map)?;
        let mut stage=Stage::load(a,map,&mut world,&visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
        let mut roster=Roster::new(&world,eye,yaw,Stats::for_level("garden3",visit.entry.as_deref()))?;
        stage.interactions.skip_cinematic(map,&mut world,&mut roster.players[0].player,&mut stage.story)?;
        for _ in 0..120 {
            if !stage.busy() { break; }
            stage.scene_step(&mut roster,map,&mut world,&Default::default())?;
        }
        ensure!(!stage.busy(),"Gate fixture did not finish the original introduction");
        let center=data::at(&map.entities[id]).translation-crate::collision::PLAYER_CENTER;
        let crossed=Motion{before:center-Vec3::Y*200.,after:center+Vec3::Y*200.};
        let before=doors(&stage);
        let mut gate_contacts=Contacts::new(&stage.interactions);
        let events=stage.interactions.triggers_pair(FIXED_DT,&mut gate_contacts,
            std::array::from_fn(|i|(i==first.index()).then_some(crossed)))?;
        ensure!(events.activations.len()==1 && events.activations[0].entity==Id(id)
            && events.activations[0].player==first && doors(&stage)!=before,
            "First crossing did not toggle the authored gate group {id}");
        let opened=doors(&stage);
        let saved=stage.snapshot();
        stage.restore(&saved,map,&mut world)?;
        // Fresh guest contacts (including solo-to-co-op import or a respawn)
        // must not reset the committed shared world dispatch history.
        gate_contacts=Contacts::new(&stage.interactions);
        let events=stage.interactions.triggers_pair(1.,&mut gate_contacts,
            std::array::from_fn(|i|(i==first.partner().index()).then_some(crossed)))?;
        ensure!(events.activations.is_empty() && doors(&stage)==opened,
            "Following {first:?} closed the shared chase gates at trigger{id}");
        let event=Event::Entity(Id(id),Input::Activate);
        ensure!(stage.interactions.event_world.has_fired(&event),"Gate dispatch history was lost");
        ensure!(stage.interactions.event_world.snapshot().usage[&format!("trigger/{id}")].count==1,
            "Gate fired more than once after restore");
        println!("PASS chase trigger{id}, {first:?} first: one shared toggle after restore and fresh partner contacts");
    }}
    // Solo activation retains its original repeatable toggle semantics.
    let (mut solo,_)=free(a,map)?;
    let before=serde_json::to_value(&solo.saved.doors)?;
    for _ in 0..2 { solo.output(&crate::event::Effect::Activate(Id(49))); }
    ensure!(serde_json::to_value(&solo.saved.doors)?==before,"Co-op gate policy changed solo repeatability");
    Ok(())
}
pub(super) fn check(a: &mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/garden3.bsp")?)?;
    clocks(a,&map)?;
    contacts(a,&map)?;
    scenes(a,&map)?;
    paired_gate_contacts(a,&map)?;
    println!("PASS Rolling Stones 11 native co-op component cases; full paired traversal remains unverified");
    Ok(())
}
