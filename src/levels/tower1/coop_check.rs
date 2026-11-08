//! Supplied-map shared clocks, contacts and personal push integration.
use super::*;
use crate::{collision::{PLAYER_CENTER, PLAYER_HALF}, coop::{protocol::{Input, Visit},
    stage::Stage, world::{self, Roster}, PlayerId}, inventory::Stats,
    level::CoopPlayer, movement::Controls};
#[path = "coop_scene_check.rs"]
mod scenes;

fn tower(stage: &mut Stage) -> &mut Tower {
    stage.interactions.levels.iter_mut().find_map(|s| s.ctl.downcast_mut()).unwrap()
}
fn bodies(r: &mut Roster) -> [CoopPlayer<'_>; 2] {
    r.players.each_mut().map(|b| CoopPlayer {player: &mut b.player, stats: &mut b.stats})
}
fn fixture(a: &mut Assets, map: &Bsp, intro: bool) -> Result<(Stage, World, Roster)> {
    let visit = Visit {epoch: 1, map: "tower1".into(), entry: None,
        difficulty: map.difficulty, combat_preview: false};
    let mut w = World::from_bsp(map)?;
    let mut s = Stage::load(a, map, &mut w, &visit, true)?;
    if !intro { tower(&mut s).upgraded(); s.owner = None; }
    s.interactions.sync_coop(&mut w);
    let (eye, yaw) = interaction::spawn(map, None);
    let r = Roster::new_with(&w, eye, yaw, Stats::for_level("tower1", None),
        |at| !s.interactions.hazardous(at))?;
    Ok((s, w, r))
}
fn restore(s: &mut Stage, map: &Bsp, w: &mut World) -> Result<()> {
    let saved = serde_json::to_value(s.snapshot())?;
    let camera = s.camera(w).map(|c| (c.eye, c.target, c.up));
    let bytes = crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
    s.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?, map, w)?;
    ensure!(serde_json::to_value(s.snapshot())? == saved
        && s.camera(w).map(|c| (c.eye, c.target, c.up)) == camera,
        "Airborne compressed restore changed shared state or camera");
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/tower1.bsp")?)?;
    let mut failed = Vec::new();
    for (name, run) in [("clocks", clocks as fn(&mut Assets, &Bsp) -> Result<()>),
        ("contacts", contacts), ("pushes", pushes), ("scenes", scenes::check), ("exits", exits)] {
        if let Err(error) = run(a, &map) {
            println!("FAIL paired Airborne {name}: {error:#}"); failed.push(name);
        }
    }
    ensure!(failed.is_empty(), "Airborne component failures: {failed:?}");
    Ok(())
}

fn clocks(a: &mut Assets, map: &Bsp) -> Result<()> {
    for hz in [30, 60, 144] {
        let (mut s, mut w, mut r) = fixture(a, map, false)?;
        let mut solo = Tower::load(a, map)?; solo.upgraded();
        let mut player = r.players[0].player.clone();
        for face in 1..=4 {
            let event = format!("Face{face}Thread");
            tower(&mut s).event(&event); solo.event(&event);
        }
        let people = serde_json::to_value(&r.players)?;
        for n in 0..hz * 6 {
            s.before_move(&mut r, map, &mut w, &Default::default(), 1. / hz as f32)?;
            solo.advance(1. / hz as f32, map, &mut w, &mut player, &[])?;
            ensure!(tower(&mut s).snapshot() == solo.snapshot(), "Paired face clock differs from solo at {hz}Hz tick{n}: paired{} solo{}", tower(&mut s).snapshot(), solo.snapshot());
            for face in &solo.faces {
                let player = w.traversal.pushes.iter().find(|p| p.id.0 == face.push).unwrap();
                let actor = w.traversal.actor_pushes.iter().find(|p| p.id.0 == face.push).unwrap();
                let n = solo.faces.iter().position(|f| f.push == face.push).unwrap();
                ensure!(player.enabled == solo.gust(n) && actor.enabled == player.enabled,
                    "Paired player/enemy gust phases disagree");
            }
            if n == hz / 2 { restore(&mut s, map, &mut w)?; }
        }
        ensure!(serde_json::to_value(&r.players)? == people, "Shared clock moved a body or changed resources");
        for owner in 0..2 {
            tower(&mut s).saved.faces.fill(1.);
            r.players[owner].stats.watch().map_err(anyhow::Error::msg)?;
            let before = tower(&mut s).snapshot();
            for _ in 0..10 {
                let dt = world::world_dt(&r.players, FIXED_DT);
                s.before_move(&mut r, map, &mut w, &Default::default(), dt)?;
            }
            ensure!(tower(&mut s).snapshot() == before, "Living Watch did not freeze shared gusts");
            r.players[owner].stats.damage(1000.);
            let dt = world::world_dt(&r.players, FIXED_DT);
            s.before_move(&mut r, map, &mut w, &Default::default(), dt)?;
            ensure!(tower(&mut s).saved.faces.iter().all(|t| (*t - 1. - FIXED_DT as f64).abs() < 1e-7),
                "Dead Watch owner froze the survivor's gusts");
            r.players[owner].stats.set_health(100.)?;
            r.players[owner].stats.powers.stopped = 0.;
        }
        let saved = tower(&mut s).snapshot();
        for dt in [0., -1., f32::NAN, f32::INFINITY] {
            tower(&mut s).coop_advance_roster(dt, map, &mut w, &mut bodies(&mut r), &[])?;
        }
        for p in &mut r.players { p.stats.damage(1000.); }
        tower(&mut s).coop_advance_roster(FIXED_DT, map, &mut w, &mut bodies(&mut r), &[])?;
        ensure!(tower(&mut s).snapshot() == saved, "Paused/invalid/all-dead paired gust advanced");
        println!("PASS paired Airborne {hz}Hz: solo clock parity, synchronized player/enemy gusts, both living/dead Watches, pause and compressed restore");
    }
    Ok(())
}

fn contacts(a: &mut Assets, map: &Bsp) -> Result<()> {
    for first in PlayerId::ALL { for dead in [false, true] { for (point, expected) in [
        (vec3(1600., 680., 1800.), vec![0]),
        (vec3(2080., 1100., 1800.), vec![1, 2]),
        (vec3(2400., 1200., 2500.), vec![3]),
    ] {
        let (mut s, mut w, mut r) = fixture(a, map, false)?;
        r.players[first.index()].player.feet = point;
        if dead { r.players[first.index()].stats.damage(1000.); }
        let before = r.players.each_ref().map(|p| p.player.feet);
        s.contacts(&mut r, &w, before)?;
        if dead {
            ensure!(tower(&mut s).saved.faces == [6.; 4], "Dead Alice activated face");
            r.players[first.partner().index()].player.feet = point;
            let before = r.players.each_ref().map(|p| p.player.feet);
            s.contacts(&mut r, &w, before)?;
        }
        ensure!(expected.iter().all(|&n| tower(&mut s).saved.faces[n] == 0.), "Living contact failed to start gust");
        let mut resumed = None;
        for tick in 0..723 {
            s.before_move(&mut r, map, &mut w, &Default::default(), FIXED_DT)?;
            let before = r.players.each_ref().map(|p| p.player.feet);
            s.contacts(&mut r, &w, before)?;
            if tick == 240 {
                let (mut copy, mut cw, mut cr) = fixture(a, map, false)?;
                let bytes = crate::coop::codec::encode(&serde_json::to_vec(&s.snapshot())?)?;
                copy.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?, map, &mut cw)?;
                cr.players = r.players.clone();
                resumed = Some((copy, cw, cr));
            } else if let Some((copy, cw, cr)) = &mut resumed {
                copy.before_move(cr, map, cw, &Default::default(), FIXED_DT)?;
                let before = cr.players.each_ref().map(|p| p.player.feet);
                copy.contacts(cr, cw, before)?;
                ensure!(serde_json::to_value(copy.snapshot())? == serde_json::to_value(s.snapshot())?,
                    "Restored occupied face future differs");
            }
        }
        ensure!(expected.iter().all(|&n| tower(&mut s).saved.faces[n] < 0.1)
            && (0..4).filter(|n| !expected.contains(n)).all(|n| tower(&mut s).saved.faces[n] == 6.),
            "Occupied face did not repeat once after six seconds or touched another face");
        println!("PASS paired Airborne contacts {first:?} dead={dead} faces{expected:?}: native trigger, dead exclusion, six-second repeat and restored future");
    }}}
    Ok(())
}

fn push_points(w: &World, without: &World, id: usize) -> Result<[Vec3; 2]> {
    let pad = w.traversal.pushes.iter().find(|p| p.id.0 == id).context("Missing face push")?;
    let origin = pad.volume.interior_point().context("Empty face push")? - PLAYER_CENTER;
    let mut found = Vec::<Vec3>::new();
    for x in (-96..=96).step_by(16) { for y in (-96..=96).step_by(16) { for z in (-96..=96).step_by(16) {
        let at = origin + vec3(x as f32, y as f32, z as f32);
        if !w.body_clear(at) || !pad.volume.touches(at + PLAYER_CENTER, at + PLAYER_CENTER, PLAYER_HALF)
            || found.iter().any(|p| p.distance(at) < 72.) { continue; }
        let mut pushed = Player::new(at); let mut baseline = pushed.clone();
        pushed.tick(w, Controls::default()); baseline.tick(without, Controls::default());
        if w.body_clear(pushed.feet) && (pushed.velocity - baseline.velocity).dot(pad.direction) > 1. {
            found.push(at);
            if found.len() == 2 { return Ok([found[0], found[1]]); }
        }
    }}}
    anyhow::bail!("No two clear native face push approaches for {id}")
}
fn pushes(a: &mut Assets, map: &Bsp) -> Result<()> {
    for face in 0..4 {
        let (mut s, mut w, _) = fixture(a, map, false)?;
        tower(&mut s).saved.faces[face] = 1.; s.interactions.sync_coop(&mut w);
        let id = tower(&mut s).faces[face].push;
        let mut off = w.clone();
        off.traversal.pushes.iter_mut().find(|p| p.id.0 == id).unwrap().enabled = false;
        let points = push_points(&w, &off, id)?;
        let direction = w.traversal.pushes.iter().find(|p| p.id.0 == id).unwrap().direction;
        for dead in [None, Some(PlayerId::Blue), Some(PlayerId::Red)] {
            let (_, _, mut r) = fixture(a, map, false)?;
            for (p, at) in r.players.iter_mut().zip(points) { p.player = Player::new(at); }
            if let Some(dead) = dead { r.players[dead.index()].stats.damage(1000.); }
            let before = r.players.clone();
            let (_, _, mut baseline) = fixture(a, map, false)?; baseline.players = before.clone();
            r.move_players(&w, Default::default()); baseline.move_players(&off, Default::default());
            for slot in 0..2 {
                if dead.is_some_and(|id| id.index() == slot) {
                    ensure!(serde_json::to_value(&r.players[slot])? == serde_json::to_value(&before[slot])?,
                        "Native face moved dead body");
                } else {
                    ensure!((r.players[slot].player.velocity - baseline.players[slot].player.velocity).dot(direction) > 1.
                        && w.body_clear(r.players[slot].player.feet), "Face did not push living slot {slot}");
                }
            }
            restore(&mut s, map, &mut w)?;
            println!("PASS paired Airborne face{face} push dead{dead:?}: both ordinary body integrators, native pad and dead exclusion");
        }
    }
    Ok(())
}

fn exits(a: &mut Assets, map: &Bsp) -> Result<()> {
    for first in PlayerId::ALL {
        let (mut s, mut w, mut r) = fixture(a, map, false)?;
        restore(&mut s, map, &mut w)?;
        let exit = &map.entities[22];
        let model = exit.get("model").context("Missing native Tower exit")?.trim_start_matches('*').parse()?;
        let origin = exit.get("origin").and_then(|v| interaction::vector(v)).unwrap_or(Vec3::ZERO);
        let volume = Collider::model(map, model, origin, Quat::IDENTITY, false)?;
        let at = volume.interior_point().context("Empty native Tower exit")? - PLAYER_CENTER;
        let resources = serde_json::to_value(r.players.each_ref().map(|b| &b.stats))?;
        r.players[first.index()].player.feet = at;
        let before = r.players.each_ref().map(|b| b.player.feet);
        s.contacts(&mut r, &w, before)?;
        ensure!(s.transition == Some(("hedge2".into(), Some("hedge2_start1".into())))
            && serde_json::to_value(r.players.each_ref().map(|b| &b.stats))? == resources,
            "First arrival did not select unchanged shared Hedge2 departure");
        println!("PASS paired Airborne exit {first:?}: actual exit contact selects shared Hedge2 target with unchanged resources");
    }
    Ok(())
}
