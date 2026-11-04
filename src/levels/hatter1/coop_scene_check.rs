//! Native scene fixtures; these do not stand in for continuous paired traversal.
use super::*;
use crate::{coop::{protocol::{Input, Visit}, stage::Stage, world::Roster, PlayerId},
    level::CoopPlayer, movement::{Controls, FIXED_DT}};

fn ctl(stage: &mut Stage) -> &mut Clockwork {
    stage.interactions.levels.iter_mut().find_map(|s| s.ctl.downcast_mut()).unwrap()
}
fn setup(a: &mut Assets, map: &Bsp, kind: Kind, first: PlayerId, dead: bool)
    -> Result<(World, Stage, Roster)> {
    let visit = Visit { epoch: 1, map: "hatter1".into(), entry: Some("hatter1_start1".into()),
        difficulty: Default::default(), combat_preview: false };
    let mut w = World::from_bsp(map)?;
    let mut stage = Stage::load(a, map, &mut w, &visit, true)?;
    let (eye, yaw) = interaction::spawn(map, visit.entry.as_deref());
    let mut roster = Roster::new(&w, eye, yaw, Stats::for_level("hatter1", visit.entry.as_deref()))?;
    let o = ctl(&mut stage);
    // Stage the original prerequisites, then activate each scene by its actual
    // contact or lever. No claim of having traversed those prerequisites.
    o.saved.initialized = true;
    o.saved.age = 100.;
    o.saved.levers[0] = Some(1.); o.saved.levers[1] = Some(2.);
    if kind != Kind::Mirror { o.saved.levers[2] = Some(5.); }
    if !matches!(kind, Kind::Mirror | Kind::Hare) { o.saved.hare = Some(10.); }
    if matches!(kind, Kind::Gryphon | Kind::Stop | Kind::Hint) {
        o.saved.levers[3] = Some(12.); o.saved.port = Some(30.);
    }
    if kind == Kind::Stop {
        o.saved.gryphon = Some(40.); o.saved.extendo = Some(60.);
        o.saved.cubes = [true; 4]; o.saved.clock = Some(90.);
    }
    o.rebuild(map)?;
    stage.interactions.sync_coop(&mut w);
    roster.players[0].stats.set_health(54.)?;
    roster.players[1].stats.set_health(67.)?;
    if dead { roster.players[first.partner().index()].stats.damage(1000.); }
    Ok((w, stage, roster))
}
fn resources(roster: &Roster) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(roster.players.each_ref().map(|p| &p.stats))?)
}
fn checkpoint(stage: &mut Stage, roster: &mut Roster, map: &Bsp, w: &mut World) -> Result<()> {
    let before = serde_json::to_value(stage.snapshot())?;
    let people = serde_json::to_value(&roster.players)?;
    let camera = stage.camera(w).map(|c| (c.eye, c.target, c.up));
    let transforms = stage.draw_transforms(w); let fov = stage.scene_fovy(16. / 9.);
    let alice = ctl(stage).alice_pose().map(|p| (p.translation, p.rotation));
    let o = ctl(stage);
    let mut players = roster.players.each_mut().map(|p| CoopPlayer { player: &mut p.player, stats: &mut p.stats });
    for dt in [0., -1., f32::NAN] { o.coop_advance_roster(dt, map, w, &mut players, &[])?; }
    ensure!(serde_json::to_value(stage.snapshot())? == before && serde_json::to_value(&roster.players)? == people,
        "Paused scene changed state or bodies");
    let bytes = crate::coop::codec::encode(&serde_json::to_vec(&before)?)?;
    stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?, map, w)?;
    ensure!(serde_json::to_value(stage.snapshot())? == before && camera == stage.camera(w).map(|c| (c.eye, c.target, c.up))
        && transforms == stage.draw_transforms(w) && fov == stage.scene_fovy(16. / 9.),
        "Restored scene changed state, camera or machinery");
    ensure!(alice == ctl(stage).alice_pose().map(|p| (p.translation, p.rotation)),
        "Restored scene changed its authored Alice pose");
    Ok(())
}
fn activate(stage: &mut Stage, roster: &mut Roster, map: &Bsp, w: &mut World,
    kind: Kind, first: PlayerId) -> Result<()> {
    if matches!(kind, Kind::Port | Kind::Stop) {
        let lever = if kind == Kind::Port { 3 } else { 4 };
        let pose = ctl(stage).levers[lever];
        let feet = w.actor_footing(pose.point(vec3(-64., 0., 60.)), PLAYER_CENTER, PLAYER_HALF, 128.)
            .context("Scene lever approach lacks footing")?;
        let player = &mut roster.players[first.index()].player;
        *player = Player::new(feet); player.grounded = true;
        let aim = (pose.translation + Vec3::Z * 20. - player.eye()).normalize();
        let mut inputs = [(Controls::default(), Input::default()); 2];
        inputs[first.index()].0.use_pressed = true;
        inputs[first.index()].1.yaw = aim.y.atan2(aim.x);
        inputs[first.index()].1.pitch = aim.z.asin();
        stage.before_move(roster, map, w, &inputs, FIXED_DT)?;
        ensure!(ctl(stage).saved.pull.is_some(), "Actual scene lever was not used");
        for _ in 0..960 {
            stage.before_move(roster, map, w, &Default::default(), FIXED_DT)?;
            if stage.busy() { break; }
        }
    } else {
        let e = map.entities.iter().find(|e| e.get("thread").is_some_and(|s| s == kind.id()))
            .context("Missing native scene contact")?;
        let model = e.get("model").and_then(|s| s.strip_prefix('*')).context("Missing scene brush")?.parse()?;
        let volume = Collider::model(map, model, at(e).translation, Quat::IDENTITY, false)?;
        roster.players[first.index()].player = Player::new(volume.interior_point().context("Empty scene brush")? - PLAYER_CENTER);
        let before = roster.players.each_ref().map(|p| p.player.feet);
        stage.contacts(roster, w, before)?;
    }
    ensure!(stage.scene_id() == Some(kind.id()), "Native {kind:?} activation failed");
    Ok(())
}
fn cinematic(a: &mut Assets, map: &Bsp, kind: Kind, first: PlayerId, skip: bool, dead: bool) -> Result<()> {
    let (mut w, mut stage, mut roster) = setup(a, map, kind, first, dead)?;
    activate(&mut stage, &mut roster, map, &mut w, kind, first)?;
    ensure!(stage.owner == Some(first), "Wrong {kind:?} owner: {:?}", stage.owner);
    let other = first.partner().index(); let waiting = roster.players[other].player.feet;
    let stats = resources(&roster)?; let mut skipped_at = None; let mut completed_at = None;
    let mut reunion_wait = 0;
    for tick in 0..24000 {
        if tick == 120 { checkpoint(&mut stage, &mut roster, map, &mut w)?; }
        let mut inputs = [(Controls::default(), Input::default()); 2];
        inputs[other].1.skip_scene = true;
        inputs[first.index()].1.skip_scene = skip && tick >= 240;
        stage.scene_step(&mut roster, map, &mut w, &inputs)?;
        if stage.notices[first.index()].as_ref().is_some_and(|n| n.text == "Scene skipped") { skipped_at.get_or_insert(tick); }
        if tick < 240 { ensure!(stage.scene_id() == Some(kind.id()), "Partner skipped {kind:?}"); }
        if stage.scene_id().is_some() {
            ensure!(stage.owner == Some(first) && stage.camera(&w).is_some(), "Scene lost its owner or camera");
            ensure!(ctl(&mut stage).alice_pose().is_some_and(|p| p.translation.is_finite() && p.rotation.is_finite()),
                "Cinematic lost its authored Alice pose");
        }
        if !stage.busy() { completed_at = Some(tick); break; }
        if stage.scene_id().is_none() && !stage.story.busy() {
            reunion_wait += 1;
            ensure!(reunion_wait < 240, "Reunion stalled at {:?}, waiting {:?}", roster.players[first.index()].player.feet, waiting);
        }
        ensure!(roster.players[other].player.feet == waiting, "Scene moved waiting partner");
    }
    let completed_at = completed_at.context("Scene or reunion never completed")?;
    ensure!(skipped_at.is_some() == skip && stage.camera(&w).is_none(), "Wrong skip or camera release");
    let o = ctl(&mut stage);
    ensure!(match kind { Kind::Hare => o.saved.hare.is_some(), Kind::Port => o.saved.port.is_some(),
        Kind::Gryphon => o.saved.gryphon.is_some(), Kind::Stop => o.saved.stopped, _ => false },
        "Scene failed to commit its progression");
    let actor = &roster.players[first.index()].player;
    let floor = w.body_trace(actor.feet, actor.feet - Vec3::Z * 256.);
    ensure!(w.body_clear(actor.feet) && !floor.start_solid && floor.fraction < 1. && floor.normal.z >= 0.65,
        "Unsafe actor endpoint: {:?}", actor.feet);
    if !dead {
        let p = &roster.players[other].player; let delta = p.feet - actor.feet;
        ensure!(w.body_clear(p.feet) && crate::recovery::supported(&w, p.feet)
            && !stage.interactions.hazardous(p.feet) && delta.truncate().length() <= 256. && delta.z.abs() <= 256.
            && (delta.truncate().length() >= 35.95 || delta.z.abs() >= 55.95),
            "Unsafe, overlapping or distant partner reunion");
        ensure!(roster.players.iter().all(|b| b.warped_at == roster.tick), "Missing shared reunion receipt");
    } else { ensure!(roster.players[other].player.feet == waiting, "Scene relocated dead partner"); }
    ensure!(resources(&roster)? == stats, "Scene changed personal resources");
    println!("PASS Clockwork {kind:?} {first:?} skip{skip} dead{dead}: native activation, frozen partner, pause/restore and safe reunion ({completed_at} ticks)");
    Ok(())
}
fn speech(a: &mut Assets, map: &Bsp, kind: Kind, first: PlayerId, dead: bool) -> Result<()> {
    let (mut w, mut stage, mut roster) = setup(a, map, kind, first, dead)?;
    let explorers = roster.players.each_ref().map(|b| b.player.clone());
    activate(&mut stage, &mut roster, map, &mut w, kind, first)?;
    let stats = resources(&roster)?; let mut completed = false;
    // The native speech contact can be above a narrow platform. Restore both
    // distinct entrance footings to isolate each player's ordinary movement.
    for (body, player) in roster.players.iter_mut().zip(explorers) { body.player = player; }
    let start = roster.players.each_ref().map(|b| b.player.feet);
    ensure!(start[0].distance(start[1]) >= 35.95, "Speech fixture starts with overlapping players");
    for tick in 0..24000 {
        if tick == 120 { checkpoint(&mut stage, &mut roster, map, &mut w)?; }
        let mut inputs = [(Controls::default(), Input::default()); 2];
        if tick < 60 { for input in &mut inputs { input.0.wish = Vec2::X; } }
        let before = ctl(&mut stage).saved.scene.as_ref().map(|s| s.time);
        stage.before_move(&mut roster, map, &mut w, &inputs, FIXED_DT)?;
        if let (Some(before), Some(after)) = (before, ctl(&mut stage).saved.scene.as_ref().map(|s| s.time)) {
            ensure!((after - before - FIXED_DT).abs() < 0.00001, "Speech advanced more than once");
        }
        roster.move_players(&w, inputs);
        stage.speech_step(&mut roster, &w, false);
        ensure!(!stage.busy() && stage.camera(&w).is_none(), "Ordinary speech froze exploration");
        ensure!((0..2).all(|i| !ctl(&mut stage).coop_controlled(i) && !ctl(&mut stage).coop_hides_player(i)),
            "Ordinary speech took control or hid an explorer");
        if stage.scene_id().is_none() { completed = true; break; }
    }
    let index = usize::from(kind == Kind::Hint);
    ensure!(completed && ctl(&mut stage).saved.hints[index], "Speech did not complete");
    for (body, start) in roster.players.iter().zip(start) {
        ensure!(if body.stats.alive() { body.player.feet.distance(start) > 10. } else { body.player.feet == start },
            "Speech prevented a living explorer from moving or moved a dead partner");
    }
    ensure!(resources(&roster)? == stats, "Speech changed personal resources");
    println!("PASS Clockwork {kind:?} {first:?} dead{dead}: actual contact, single speech clock, free exploration and pause/restore");
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/hatter1.bsp")?)?; let mut failures = Vec::new();
    for kind in [Kind::Hare, Kind::Port, Kind::Gryphon, Kind::Stop] {
        for first in PlayerId::ALL { for skip in [false, true] { for dead in [false, true] {
            if let Err(e) = cinematic(a, &map, kind, first, skip, dead) {
                let message = format!("{kind:?} {first:?} skip{skip} dead{dead}: {e:#}");
                eprintln!("FAIL {message}"); failures.push(message);
            }
        }}}
    }
    for kind in [Kind::Mirror, Kind::Hint] { for first in PlayerId::ALL { for dead in [false, true] {
        if let Err(e) = speech(a, &map, kind, first, dead) {
            let message = format!("{kind:?} {first:?} dead{dead}: {e:#}");
            eprintln!("FAIL {message}"); failures.push(message);
        }
    }}}
    ensure!(failures.is_empty(), "Clockwork paired scene failures:\n{}", failures.join("\n"));
    println!("PASS Clockwork paired scene components; full traversal and internet checks remain separate");
    Ok(())
}
