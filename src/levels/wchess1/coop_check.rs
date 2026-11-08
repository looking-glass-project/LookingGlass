//! Isolated cast components using the real Pale Realm roster and animation data.
//! These checks do not establish paired puzzles, machinery or full traversal.
use super::*;
use crate::{chess::{Kind, Phase}, combat::{DamageKind, Feedback, Hit, PlayerTarget}, movement::FIXED_DT};

fn setup(a: &mut Assets, map: &Bsp) -> Result<Realm> {
    let mut r = Realm::load(a, map)?;
    r.saved.pending = None;
    r.saved.intro = true;
    for actor in &mut r.saved.cast {
        actor.piece.active = false;
        actor.piece.spawn_delay = None;
    }
    Ok(r)
}
fn floor(blocked: bool) -> World {
    let mut boxes = vec![(vec3(-3000., -3000., -64.), vec3(3000., 3000., 0.))];
    if blocked { boxes.push((vec3(45., -32., 0.), vec3(65., 32., 240.))); }
    World::fixture(&boxes)
}
fn player(at: Vec3) -> PlayerTarget {
    PlayerTarget { eye: Player::new(at).eye(), alive: true, hidden: false }
}
fn players(near: usize, distance: f32) -> [PlayerTarget; 2] {
    let mut p = [player(vec3(distance, 0., 0.1)), player(vec3(0., 240., 0.1))];
    if near == 1 { p.swap(0, 1); }
    p
}
fn activate(r: &mut Realm, index: usize, feet: Vec3) {
    let a = &mut r.saved.cast[index];
    a.piece = crate::chess::Piece::new(a.piece.kind, feet, 0., a.piece.scale, a.id());
    a.walk = None;
    a.foe = None;
    a.player = None;
}
fn actor(r: &Realm, kind: Kind) -> Result<usize> {
    r.saved.cast.iter().position(|a| !a.white && a.piece.kind == kind)
        .context("Missing native red chess family")
}
fn restored(a: &mut Assets, map: &Bsp, r: &Realm) -> Result<Realm> {
    let packet = crate::coop::codec::encode(&serde_json::to_vec(&r.snapshot())?)?;
    let value = serde_json::from_slice(&crate::coop::codec::decode(&packet)?)?;
    let mut q = Realm::load(a, map)?;
    q.restore(&value, map)?;
    ensure!(q.snapshot() == r.snapshot(), "Paired chess snapshot changed on restore");
    Ok(q)
}
fn feedback(f: &Feedback) -> serde_json::Value {
    serde_json::json!({"players":f.players.map(|p|(p.damage,p.impulse,p.will_drain)),
        "legacy":(f.damage,f.impulse,f.will_drain),"sounds":f.sounds,"spatial":f.spatial_sounds})
}
fn selection(a: &mut Assets, map: &Bsp) -> Result<()> {
    for kind in [Kind::Pawn, Kind::Knight, Kind::Bishop] { for near in 0..2 { for excluded in 0..4 {
        let mut r = setup(a, map)?;
        let index = actor(&r, kind)?;
        activate(&mut r, index, Vec3::Z * 0.1);
        let mut p = players(near, 130.);
        p[near].alive = excluded != 1;
        p[near].hidden = excluded == 2;
        let w = floor(excluded == 3);
        let before = r.snapshot();
        for dt in [0., -1., f32::NAN] {
            let f = r.coop_combat(dt, &w, p);
            ensure!(r.snapshot() == before && f.players.iter().all(|p| p.damage == 0.), "Paused chess advanced");
        }
        r.coop_combat(FIXED_DT, &w, p);
        ensure!(r.saved.cast[index].player == Some(if excluded == 0 {near} else {1-near}),
            "Wrong {kind:?} target near={near} excluded={excluded}: {:?}", r.saved.cast[index].player);
        restored(a, map, &r)?;
        println!("PASS paired {kind:?} near={near} excluded={excluded}: visible/living selection, pause and snapshot");
    }}}
    Ok(())
}
fn attacks(a: &mut Assets, map: &Bsp) -> Result<()> {
    for kind in [Kind::Pawn, Kind::Knight, Kind::Bishop] { for victim in 0..2 {
        let mut r = setup(a, map)?;
        let index = actor(&r, kind)?;
        activate(&mut r, index, Vec3::Z * 0.1);
        let p = players(victim, 70.);
        let w = floor(false);
        let mut out = Feedback::default();
        for _ in 0..1200 {
            out.append(r.coop_combat(FIXED_DT, &w, p));
            if out.players[victim].damage > 0. { break; }
        }
        ensure!(out.players[victim].damage > 0. && out.players[1-victim].damage == 0.
            && out.damage == 0. && !out.spatial_sounds.is_empty(),
            "{kind:?} damaged wrong player or failed to attack victim={victim}: {}", feedback(&out));
        println!("PASS paired {kind:?} melee victim={victim}: native attack frames, personal damage and audio");
    }}
    for victim in 0..2 {
        let mut r = setup(a, map)?;
        let index = actor(&r, Kind::Bishop)?;
        activate(&mut r, index, Vec3::Z * 0.1);
        let mut p = players(victim, 300.);
        p[1-victim].eye = vec3(0., 700., 52.1);
        let w = floor(false);
        for _ in 0..600 {
            r.coop_combat(FIXED_DT, &w, p);
            if r.saved.cast[index].piece.phase == Phase::Beam { break; }
        }
        ensure!(r.saved.cast[index].piece.phase == Phase::Beam, "Bishop did not start beam");
        // A closer visible partner must not steal a committed beam after restore.
        p[1-victim].eye = vec3(0., 90., 52.1);
        let mut q = restored(a, map, &r)?;
        let mut out = Feedback::default();
        for _ in 0..120 {
            let f = r.coop_combat(FIXED_DT, &w, p);
            let g = q.coop_combat(FIXED_DT, &w, p);
            ensure!(feedback(&f) == feedback(&g) && r.snapshot() == q.snapshot(), "Saved beam future diverged");
            out.append(f);
            if out.players[victim].damage > 0. { break; }
        }
        ensure!(out.players[victim].damage == 8. && out.players[1-victim].damage == 0.
            && r.saved.cast[index].piece.beam.is_some(), "Committed beam changed victim or lost effect state");
        restored(a, map, &r)?;
        println!("PASS paired bishop beam victim={victim}: saved attack lock, personal impact and beam state");
    }
    Ok(())
}
fn factions(a: &mut Assets, map: &Bsp) -> Result<()> {
    for near in 0..2 { for forced in [false, true] {
        let mut r = setup(a, map)?;
        let red = r.saved.cast.iter().position(|a| a.name == "red_knight1").unwrap();
        let white = r.saved.cast.iter().position(|a| a.name == "pawn_npc2").unwrap();
        activate(&mut r, red, Vec3::Z * 0.1);
        activate(&mut r, white, vec3(70., 0., 0.1));
        r.saved.cast[white].piece.script_wait = true;
        r.saved.cast[white].damageable = true;
        if forced { r.saved.cast[red].foe = Some("pawn_npc2".into()); }
        let mut p = players(near, 50.);
        p[near].eye = vec3(30., -30., 52.1);
        ensure!(p[near].eye.distance(r.saved.cast[red].piece.feet)
            < r.saved.cast[white].piece.target(0).center.distance(r.saved.cast[red].piece.feet),
            "Faction fixture's designated nearer Alice is farther than the NPC");
        let before = r.saved.cast[white].piece.health;
        let mut out = Feedback::default();
        for _ in 0..1200 {
            out.append(r.coop_combat(FIXED_DT, &floor(false), p));
            if out.players[near].damage > 0. || r.saved.cast[white].piece.health < before { break; }
        }
        ensure!(if forced {
            r.saved.cast[white].piece.health < before && out.players.iter().all(|p| p.damage == 0.)
        } else {
            r.saved.cast[white].piece.health == before && out.players[near].damage > 0.
                && out.players[1-near].damage == 0.
        }, "Faction/explicit duel routing failed near={near} forced={forced}");
        ensure!(out.damage == 0., "NPC damage leaked into legacy Alice feedback");
        restored(a, map, &r)?;
        println!("PASS paired factions near={near} forced={forced}: authored duel or nearer Alice, separate damage");
    }}
    let mut r = setup(a, map)?;
    let index = actor(&r, Kind::Knight)?;
    activate(&mut r, index, Vec3::Z * 0.1);
    let id = r.saved.cast[index].id();
    r.coop_threats(&|target| target.id == id);
    ensure!(r.saved.cast[index].piece.phase == Phase::Block, "Paired projectile threat did not raise shield");
    r.cast_hit(Hit {id, damage: 40., kind: DamageKind::Electric, knockback: Vec3::ZERO});
    ensure!(r.saved.cast[index].piece.health == 110., "Shield lost native electric vulnerability");
    let valid = r.snapshot();
    let mut bad = valid.clone();
    bad["cast"][index]["player"] = serde_json::json!(2);
    ensure!(r.restore(&bad, map).is_err() && r.snapshot() == valid, "Invalid player target changed owner");
    // Old SP states omit all paired target identities and still load unchanged.
    ensure!(valid["cast"].as_array().unwrap().iter().all(|a| a.get("player").is_none()), "SP-compatible cast gained targets");
    r.restore(&valid, map)?;
    r.saved.pending = Some(scene::Kind::Intro);
    let frozen = r.snapshot();
    r.coop_threats(&|_| true);
    r.coop_combat(FIXED_DT, &floor(false), players(0, 70.));
    ensure!(r.snapshot() == frozen, "Cutscene advanced paired cast");
    println!("PASS paired knight shield, electric damage, invalid/legacy saves and cinematic combat freeze");
    Ok(())
}
fn callbacks(a: &mut Assets, map: &Bsp) -> Result<()> {
    for gang in [false, true] {
        let mut r = setup(a, map)?;
        r.saved.bishop_done = true;
        for name in ["rook_guard1_attacked_thread", "red_knight3_thread", "red_pawns3_thread"] { r.event(name); }
        if gang { r.event("white_pawn_bullied_thread"); }
        let mut p = players(0, 1500.);
        for player in &mut p { player.hidden = true; }
        let w = World::from_bsp(map)?;
        for _ in 0..1200 { r.coop_combat(FIXED_DT, &w, p); }
        let ids: Vec<_> = r.saved.cast.iter().filter(|a| a.counted && a.piece.active).map(|a| a.id()).collect();
        let threshold = if gang {8} else {4};
        ensure!(ids.len() == threshold, "Paired delayed cast count differs gang={gang}: {}", ids.len());
        for (n, id) in ids.iter().enumerate() {
            r.cast_hit(Hit {id: *id, damage: 1000., kind: DamageKind::Other, knockback: Vec3::ZERO});
            r.coop_combat(FIXED_DT, &w, p);
            ensure!(r.saved.knight_gate == (n+1 == threshold), "Paired gate threshold changed at {n}/{threshold}");
        }
        r.event("white_pawn_bullied_thread");
        ensure!(r.saved.knight_gate && r.saved.knight_opened.is_some(), "Late gang closed the paired gate");
        restored(a, map, &r)?;
        println!("PASS paired knight gate gang={gang}: native delayed spawns/walks, {threshold} deaths and saved late latch");
    }
    let mut r = setup(a, map)?;
    r.event("red_bishop1_thread");
    let mut p = players(0, 1500.);
    for player in &mut p { player.hidden = true; }
    let w = World::from_bsp(map)?;
    for _ in 0..600 { r.coop_combat(FIXED_DT, &w, p); }
    let ids: Vec<_> = r.saved.cast.iter().filter(|a| a.name == "red_bishop1").map(|a| a.id()).collect();
    ensure!(ids.len() == 2, "Wrong gallery bishop count");
    for (n, id) in ids.into_iter().enumerate() {
        r.cast_hit(Hit {id, damage: 1000., kind: DamageKind::Other, knockback: Vec3::ZERO});
        r.coop_combat(FIXED_DT, &w, p);
        ensure!(r.saved.spikes_off.is_some() == (n == 1), "Gallery callback fired before both deaths");
    }
    restored(a, map, &r)?;
    println!("PASS paired gallery: both native bishop deaths unlock the saved spike callback");
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/wchess1.bsp")?)?;
    selection(a, &map)?;
    attacks(a, &map)?;
    factions(a, &map)?;
    callbacks(a, &map)?;
    println!("PASS Pale Realm paired cast components (40 cases); full co-op puzzles/traversal remain unverified");
    Ok(())
}
