//! Actual Stage contacts and scene clocks from staged native prerequisites.
use super::*;
use crate::{
    coop::{
        protocol::{Input, Visit},
        stage::Stage,
        world::Roster,
        PlayerId,
    },
    level::CoopPlayer,
    movement::{Controls, FIXED_DT},
};
fn ctl(stage: &mut Stage) -> &mut Encounter {
    stage
        .interactions
        .levels
        .iter_mut()
        .find_map(|s| s.ctl.downcast_mut())
        .unwrap()
}
fn setup(a: &mut Assets, map: &Bsp, first: PlayerId, dead: bool) -> Result<(World, Stage, Roster)> {
    let visit = Visit {
        epoch: 1,
        map: "hatter2".into(),
        entry: None,
        difficulty: Default::default(),
        combat_preview: false,
    };
    let mut w = World::from_bsp(map)?;
    let mut stage = Stage::load(a, map, &mut w, &visit, true)?;
    let (eye, yaw) = crate::interaction::spawn(map, None);
    let mut roster = Roster::new(&w, eye, yaw, Stats::for_level("hatter2", None))?;
    ctl(&mut stage).saved.initialized = true;
    roster.players[0].stats.set_health(54.)?;
    roster.players[1].stats.set_health(67.)?;
    if dead {
        roster.players[first.partner().index()].stats.damage(1000.);
    }
    Ok((w, stage, roster))
}
fn resources(r: &Roster) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(
        r.players.each_ref().map(|p| &p.stats),
    )?)
}
fn checkpoint(stage: &mut Stage, r: &mut Roster, map: &Bsp, w: &mut World) -> Result<()> {
    let saved = serde_json::to_value(stage.snapshot())?;
    let people = serde_json::to_value(&r.players)?;
    let camera = stage.camera(w).map(|c| (c.eye, c.target, c.up));
    let transforms = stage.draw_transforms(w);
    let o = ctl(stage);
    let mut players = r.players.each_mut().map(|p| CoopPlayer {
        player: &mut p.player,
        stats: &mut p.stats,
    });
    for dt in [0., -1., f32::NAN] {
        o.pair_advance(dt, map, w, &mut players, &[])?;
        o.pair_pickups(dt, &mut players);
    }
    ensure!(
        serde_json::to_value(stage.snapshot())? == saved
            && serde_json::to_value(&r.players)? == people,
        "Paused scene changed state"
    );
    let bytes = crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
    stage.restore(
        &serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,
        map,
        w,
    )?;
    ensure!(
        serde_json::to_value(stage.snapshot())? == saved
            && camera == stage.camera(w).map(|c| (c.eye, c.target, c.up))
            && transforms == stage.draw_transforms(w),
        "Restored scene changed world/camera"
    );
    Ok(())
}
fn contact(
    stage: &mut Stage,
    r: &mut Roster,
    w: &World,
    thread: &str,
    first: PlayerId,
) -> Result<()> {
    let at = ctl(stage).data.volumes[thread]
        .interior_point()
        .context("Empty scene trigger")?;
    r.players[first.index()].player = Player::new(at - crate::collision::PLAYER_CENTER);
    let before = r.players.each_ref().map(|p| p.player.feet);
    stage.contacts(r, w, before)?;
    Ok(())
}
fn escape(a: &mut Assets, map: &Bsp, first: PlayerId, skip: bool, dead: bool) -> Result<()> {
    let (mut w, mut stage, mut r) = setup(a, map, first, dead)?;
    let o = ctl(&mut stage);
    coop_world_check::victory(o);
    o.saved.blade = true;
    o.saved.watch = true;
    o.phase(Phase::ExitReady);
    o.saved.door = 1.4;
    o.rebuild(map)?;
    stage.interactions.sync_coop(&mut w);
    contact(&mut stage, &mut r, &w, GRYPHON, first)?;
    let at = r.players[first.index()].player.feet;
    ensure!(
        ctl(&mut stage)
            .performances()
            .iter()
            .find(|p| p.model == "alice")
            .is_some_and(|p| p.pose.translation == at)
            && ctl(&mut stage).saved.coop.as_ref().and_then(|s| s.actor) == Some(first.index()),
        "Gryphon first frame used the wrong activating body"
    );
    checkpoint(&mut stage, &mut r, map, &mut w)?;
    ensure!(
        stage.owner == Some(first) && stage.scene_id() == Some(GRYPHON),
        "Gryphon ignored first entrant {first:?}"
    );
    let other = first.partner().index();
    let waiting = r.players[other].player.feet;
    let stats = resources(&r)?;
    let mut completed = None;
    let mut skipped = false;
    for tick in 0..24000 {
        if tick == 120 || tick == 660 {
            checkpoint(&mut stage, &mut r, map, &mut w)?;
        }
        let mut inputs = [(Controls::default(), Input::default()); 2];
        inputs[other].1.skip_scene = true;
        inputs[first.index()].1.skip_scene = skip && tick >= 240;
        stage.scene_step(&mut r, map, &mut w, &inputs)?;
        skipped |= stage.notices[first.index()]
            .as_ref()
            .is_some_and(|n| n.text == "Scene skipped");
        ensure!(
            r.players[other].player.feet == waiting,
            "Gryphon moved the hidden waiting partner"
        );
        if tick < 240 {
            ensure!(
                stage.scene_id() == Some(GRYPHON),
                "Partner skipped the owner's scene"
            );
        }
        if stage.transition.is_some() {
            completed = Some(tick);
            break;
        }
        ensure!(
            stage.camera(&w).is_some()
                && ctl(&mut stage)
                    .performances()
                    .iter()
                    .filter(|p| p.model == "alice")
                    .count()
                    == 1,
            "Gryphon lost its original camera or single Alice performance"
        );
        ensure!(
            ctl(&mut stage).saved.coop.as_ref().and_then(|s| s.actor) == Some(first.index()),
            "Saved scene owner changed"
        );
    }
    ensure!(
        completed.is_some()
            && skipped == skip
            && stage.transition.as_ref().is_some_and(|e| EXIT.matches(e)),
        "Gryphon failed original watched/skipped departure"
    );
    ensure!(
        resources(&r)? == stats && ctl(&mut stage).saved.phase == Phase::Done,
        "Gryphon changed personal resources or lost commit"
    );
    println!("PASS paired Hatter Gryphon {first:?} skip{skip} dead{dead}: native contact, frozen partner, original camera/actor, pause/restore and shared exit ({} ticks)",completed.unwrap());
    Ok(())
}
fn speech(a: &mut Assets, map: &Bsp, watch: bool, first: PlayerId, dead: bool) -> Result<()> {
    let (mut w, mut stage, mut r) = setup(a, map, first, dead)?;
    let explorers = r.players.each_ref().map(|p| p.player.clone());
    if watch {
        let o = ctl(&mut stage);
        coop_world_check::victory(o);
        o.saved.blade = true;
        let key = o.data.watch_key.clone();
        r.players[first.index()].stats.collected.insert(key);
        stage.before_move(&mut r, map, &mut w, &Default::default(), FIXED_DT)?;
    } else {
        contact(&mut stage, &mut r, &w, "hatter_cat", first)?;
    }
    ensure!(
        stage.scene_id() == Some(if watch { "ExitTest" } else { "hatter_cat" }),
        "Cat speech did not start"
    );
    for (body, p) in r.players.iter_mut().zip(explorers) {
        body.player = p;
    }
    let stats = resources(&r)?;
    let start = r.players.each_ref().map(|p| p.player.feet);
    let mut done = false;
    for tick in 0..24000 {
        if tick == 120 {
            checkpoint(&mut stage, &mut r, map, &mut w)?;
        }
        let mut inputs = [(Controls::default(), Input::default()); 2];
        if tick < 60 {
            for input in &mut inputs {
                input.0.wish = Vec2::X;
            }
        }
        stage.before_move(&mut r, map, &mut w, &inputs, FIXED_DT)?;
        r.move_players(&w, inputs);
        stage.speech_step(&mut r, &w, false);
        ensure!(
            !stage.busy() && stage.camera(&w).is_none(),
            "Cat speech froze exploration"
        );
        if stage.scene_id().is_none() {
            done = true;
            break;
        }
    }
    ensure!(
        done && ctl(&mut stage).saved.phase
            == if watch {
                Phase::ExitReady
            } else {
                Phase::Fight
            },
        "Cat speech lost progression"
    );
    for (body, at) in r.players.iter().zip(start) {
        ensure!(
            if body.stats.alive() {
                body.player.feet.distance(at) > 10.
            } else {
                body.player.feet == at
            },
            "Cat speech did not preserve free/dead movement"
        );
    }
    ensure!(
        resources(&r)? == stats,
        "Cat speech changed personal resources"
    );
    println!("PASS paired Hatter ordinary Cat watch{watch} {first:?} dead{dead}: native contact/reward, free exploration and saved progression");
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/hatter2.bsp")?)?;
    let mut failures = vec![];
    for first in PlayerId::ALL {
        for dead in [false, true] {
            for skip in [false, true] {
                if let Err(e) = escape(a, &map, first, skip, dead) {
                    let message = format!("Gryphon {first:?} skip{skip} dead{dead}: {e:#}");
                    eprintln!("FAIL {message}");
                    failures.push(message);
                }
            }
            for watch in [false, true] {
                if let Err(e) = speech(a, &map, watch, first, dead) {
                    let message = format!("Cat watch{watch} {first:?} dead{dead}: {e:#}");
                    eprintln!("FAIL {message}");
                    failures.push(message);
                }
            }
        }
    }
    ensure!(
        failures.is_empty(),
        "Paired Hatter scene failures: {}",
        failures.join("; ")
    );
    println!(
        "PASS paired Hatter scene components; full co-op traversal/recovery remains unverified"
    );
    Ok(())
}
