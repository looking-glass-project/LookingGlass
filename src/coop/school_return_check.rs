//! Focused native return-quest gates, using the production paired stage.
//! Bodies are staged at authored contacts; this is not a traversal/network test.
use super::{protocol::Visit, stage::Stage, world::Roster, PlayerId};
use crate::{assets::Assets, bsp::Bsp, collision::World, inventory::Stats,
    movement::{Player, EYE_HEIGHT, FIXED_DT}, school_return::Phase};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;

pub(super) fn check(assets: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&assets.read("maps/skool1.bsp")?)?;
    let visit = Visit { epoch: 1, map: "skool1".into(), entry: Some("skool1_start2".into()),
        difficulty: Default::default(), combat_preview: false };
    check_lift(assets, &map, &visit)?;
    for first in PlayerId::ALL {
        for holder in PlayerId::ALL {
            for dead_partner in [false, true] {
                let mut world = World::from_bsp(&map)?;
                let mut stage = Stage::load(assets, &map, &mut world, &visit, false)?;
                let (eye, yaw) = crate::interaction::spawn(&map, visit.entry.as_deref());
                let mut roster = Roster::new(&world, eye, yaw, Stats::for_level("skool1", visit.entry.as_deref()))?;
                for body in &mut roster.players { body.stats.school_items = Default::default(); }
                roster.players[holder.index()].stats.school_items.star = true;
                roster.players[holder.partner().index()].stats.school_items.potion = true;
                roster.players[0].stats.damage(13.);
                roster.players[1].stats.damage(27.);
                if dead_partner { roster.players[first.partner().index()].stats.damage(1000.); }
                let mut expected = roster.players.each_ref().map(|b| b.stats.clone());
                let door = vec3(692., 4174., 384.03125);
                roster.players[first.index()].player = Player::spawn(&world, door + Vec3::Z * EYE_HEIGHT)
                    .context("Cannot stage return star-door contact")?;
                stage.before_move(&mut roster, &map, &mut world, &Default::default(), FIXED_DT)?;
                let before = roster.players.each_ref().map(|b| b.player.feet);
                stage.contacts(&mut roster, &world, before)?;
                let state = stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap();
                ensure!(state.phase == Phase::Open && !state.star && state.potion,
                    "Carried return rewards did not open star doors: first {first:?}, holder {holder:?}, dead partner {dead_partner}");
                for stats in &mut expected { stats.school_items.star = false; }
                ensure!(serde_json::to_value(roster.players.each_ref().map(|b| &b.stats))?
                    == serde_json::to_value(&expected)?, "Return star gate changed personal resources or left spent copies");
                println!("PASS co-op return star gate first {first:?}, reward holder {holder:?}, dead partner {dead_partner}: shared quest, personal resources preserved");
                // Stage only the later prerequisite/position to isolate the
                // authored potion contact and performance. The lift and actual
                // route to this walkway require separate traversal evidence.
                stage.interactions.school.as_mut().unwrap().return_visit.as_mut().unwrap().enter(Phase::Observatory);
                let exit = vec3(340., 4396., 1344.03125);
                roster.players[first.index()].player = Player::spawn(&world, exit + Vec3::Z * EYE_HEIGHT)
                    .context("Cannot stage observatory exit contact")?;
                stage.before_move(&mut roster, &map, &mut world, &Default::default(), FIXED_DT)?;
                let before = roster.players.each_ref().map(|b| b.player.feet);
                stage.contacts(&mut roster, &world, before)?;
                ensure!(stage.owner == Some(first), "First arrival did not own the observatory exit");
                let waiting = roster.players[first.partner().index()].player.feet;
                for _ in 0..120*20 {
                    if stage.transition.is_some() { break; }
                    stage.scene_step(&mut roster, &map, &mut world, &Default::default())?;
                    ensure!(stage.owner == Some(first)
                        && roster.players[first.partner().index()].player.feet == waiting,
                        "Observatory performance moved its waiting partner or changed owner");
                    let phase = stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap().phase;
                    for (body, old) in roster.players.iter().zip(&expected) {
                        ensure!(body.stats.school_items.potion == (old.school_items.potion && phase < Phase::Drinking),
                            "Potion was consumed at the wrong scene phase");
                    }
                }
                ensure!(stage.transition == Some(("potears1".into(), Some("potears1_start1".into()))),
                    "Observatory did not request the authored Pool of Tears entrance: phase {:?}, owner {:?}",
                    stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap().phase, stage.owner);
                for stats in &mut expected { stats.school_items.potion = false; }
                ensure!(serde_json::to_value(roster.players.each_ref().map(|b| &b.stats))?
                    == serde_json::to_value(&expected)?, "Potion scene changed personal resources or left spent copies");
                println!("PASS co-op return potion first {first:?}, reward holder {holder:?}, dead partner {dead_partner}: original scene, frozen partner, phase consumption and authored exit");
            }
        }
    }
    Ok(())
}

fn lift_step(stage: &mut Stage, roster: &mut Roster, map: &Bsp, world: &mut World) -> Result<()> {
    let mut cinematic = stage.busy();
    if cinematic {
        stage.scene_step(roster, map, world, &Default::default())?;
    } else {
        stage.before_move(roster, map, world, &Default::default(), FIXED_DT)?;
        if stage.busy() {
            roster.tick += 1;
        } else {
            let before = roster.players.each_ref().map(|body| body.player.feet);
            roster.move_players(world, Default::default());
            stage.contacts(roster, world, before)?;
        }
        cinematic = stage.busy();
    }
    roster.finish_tick_with(world, cinematic, |at| !stage.interactions.hazardous(at))
}

/// Actual paired boarding, original Star reveal and physical ascent. Only the
/// initial bodies/rewards are staged; this does not certify entrance traversal.
fn check_lift(assets: &mut Assets, map: &Bsp, visit: &Visit) -> Result<()> {
    for first in PlayerId::ALL { for skip in [false, true] { for restore in [false, true] {
        let mut world = World::from_bsp(map)?;
        let mut stage = Stage::load(assets, map, &mut world, visit, false)?;
        let (eye, yaw) = crate::interaction::spawn(map, visit.entry.as_deref());
        let mut roster = Roster::new(&world, eye, yaw, Stats::for_level("skool1", visit.entry.as_deref()))?;
        for body in &mut roster.players {
            body.stats.school_items.star = true;
            body.stats.school_items.potion = true;
        }
        roster.players[first.index()].player = Player::spawn(&world,
            vec3(692., 4174., 384.03125) + Vec3::Z * EYE_HEIGHT).context("Star gate approach blocked")?;
        for _ in 0..150 { lift_step(&mut stage, &mut roster, map, &mut world)?; }
        ensure!(stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap().phase == Phase::Open,
            "Actual Star contact did not prepare return lift");
        let riders = crate::school::coop_check::riders(stage.interactions.school.as_ref().unwrap(),
            &world, "observatory_lift", vec3(692., 4370., 384.04))?;
        roster.players[first.index()].player = riders[0].clone();
        for _ in 0..240 {
            lift_step(&mut stage, &mut roster, map, &mut world)?;
            ensure!(!stage.busy() && roster.players[first.index()].player.feet.distance(riders[0].feet) < 0.1,
                "Observatory departed before its partner boarded");
        }
        roster.players[first.partner().index()].player = riders[1].clone();
        let mut skipped = false;
        let mut seen_scene = false;
        for tick in 0..2400 {
            let waiting = roster.players[first.partner().index()].player.feet;
            let was_scene = stage.busy();
            if was_scene {
                seen_scene = true;
                ensure!(stage.owner == Some(first), "First observatory arrival lost the Star scene");
                let scene = stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap().scene.as_ref().unwrap();
                if skip && !skipped && scene.time >= 2. {
                    ensure!(stage.interactions.skip_cinematic(map, &mut world,
                        &mut roster.players[first.index()].player, &mut stage.story)?, "Star skip rejected");
                    skipped = true;
                }
            }
            lift_step(&mut stage, &mut roster, map, &mut world)?;
            if was_scene && stage.busy() {
                ensure!(roster.players[first.partner().index()].player.feet == waiting,
                    "Star reveal moved the frozen partner");
            }
            ensure!(roster.checkpoint_generation == 0 && roster.players.iter().all(|body|
                body.stats.sanity() == 100. && body.spawned_at == 0 && world.body_clear(body.player.feet)
                    && !body.stats.school_items.star && body.stats.school_items.potion),
                "Observatory injured/embedded a rider, restored a checkpoint or spent the potion: {:?}",
                roster.players.each_ref().map(|body| body.player.feet));
            if restore && tick % 43 == 0 {
                let saved = serde_json::from_value(serde_json::to_value(stage.snapshot())?)?;
                stage.restore(&saved, map, &mut world)?;
                roster.players = serde_json::from_value(serde_json::to_value(&roster.players)?)?;
            }
            let state = stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap();
            if state.phase == Phase::Observatory && !stage.busy() { break; }
        }
        ensure!(seen_scene && skipped == skip && !stage.busy()
            && stage.interactions.school.as_ref().unwrap().return_visit.as_ref().unwrap().phase == Phase::Observatory
            && roster.players.iter().all(|body| body.player.grounded && body.player.feet.z > 1015.)
            && roster.players[0].player.feet.distance(roster.players[1].player.feet) >= 36.,
            "Observatory did not deliver two separate riders: first {first:?}, skip {skip}, restore {restore}, feet {:?}",
            roster.players.each_ref().map(|body| body.player.feet));
        println!("PASS co-op observatory lift first {first:?}, skip {skip}, restore {restore}: waits for partner, original Star scene, frozen waiter, both carried safely to top");
    }}}
    Ok(())
}
