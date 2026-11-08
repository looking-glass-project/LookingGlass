//! Paired puzzle progression, scene handoffs and spawned guard combat.
use super::*;
use crate::{combat::{Feedback, PlayerTarget}, coop::{protocol::Visit, stage::Stage, world::Roster, PlayerId}, movement::FIXED_DT};
fn owner(s: &mut Stage) -> &mut Keep {
    s.interactions.levels.iter_mut().find_map(|s| s.ctl.downcast_mut()).unwrap()
}
fn scene(s: &mut Stage, r: &mut Roster, map: &Bsp, w: &mut World) -> Result<()> {
    for tick in 0..21600 {
        s.scene_step(r, map, w, &Default::default())?;
        if tick == 240 {
            let saved = s.snapshot(); s.restore(&saved, map, w)?;
            ensure!(serde_json::to_value(saved)? == serde_json::to_value(s.snapshot())?, "Keep scene restore changed state");
        }
        if s.owner.is_none() || s.transition.is_some() { break; }
    }
    ensure!(s.owner.is_none() || s.transition.is_some(), "Paired Keep scene stalled");
    ensure!(r.players.iter().all(|p| p.stats.alive() && w.body_clear(p.player.feet)), "Keep scene embedded/killed a player");
    Ok(())
}
fn progression(a: &mut Assets, map: &Bsp) -> Result<()> {
    let visit = Visit {epoch: 1, map: "keep".into(), entry: None, difficulty: map.difficulty, combat_preview: false};
    let mut w = World::from_bsp(&map)?;
    let mut s = Stage::load(a, &map, &mut w, &visit, false)?;
    let (eye, yaw) = interaction::spawn(&map, None);
    let mut r = Roster::new_with(&w, eye, yaw, Stats::for_level("keep", None), |_| true)?;
    scene(&mut s, &mut r, &map, &mut w)?;
    ensure!(owner(&mut s).saved.arrival, "Paired Keep arrival unfinished");
    let target = owner(&mut s).data.at("mirror_lever").point(vec3(36., 0., 26.));
    let mut standing = None;
    for radius in [64., 96., 112.] { for n in 0..16 {
        let angle = n as f32 * std::f32::consts::TAU / 16.;
        let above = target + vec3(radius * angle.cos(), radius * angle.sin(), 64.);
        if let Some(feet) = w.actor_footing(above, PLAYER_CENTER, PLAYER_HALF, 192.) {
            let p = Player::new(feet); let aim = (target - p.eye()).normalize_or_zero();
            if w.body_clear(feet) && owner(&mut s).lever(&w, p.eye(), aim) { standing = Some((p, aim)); break; }
        }
    } if standing.is_some() { break; }}
    let (p, aim) = standing.context("No reachable mirror lever fixture")?;
    r.players[1].player = p;
    let e = owner(&mut s).coop_update(1, &mut w, &r.players[1].player, aim, true);
    s.accept(e, PlayerId::Red, &mut r, &w);
    ensure!(s.owner == Some(PlayerId::Red), "Mirror use lost Red owner");
    scene(&mut s, &mut r, &map, &mut w)?;
    for (round, (enter, win)) in [
        ("Door_Club_Close", "Tweedle_Club_Win"),
        ("Door_Diamond_Close", "Jabber_Diamond_Win"),
        ("Door_Spade_Close", "Hatter_Spade_Win"),
    ].into_iter().enumerate() {
        if round > 0 {
            owner(&mut s).rotate(); s.accept(Events::default(), PlayerId::Red, &mut r, &w);
            scene(&mut s, &mut r, &map, &mut w)?;
        }
        owner(&mut s).event(enter);
        for _ in 0..125 { s.before_move(&mut r, &map, &mut w, &Default::default(), FIXED_DT)?; }
        ensure!(owner(&mut s).room_ready(round), "Paired portrait timer did not arm");
        let target = s.interactions.switch_target(win).context("Missing correct portrait")?;
        let e = s.interactions.shoot(Hit {id: target.id, damage: 1., kind: crate::combat::DamageKind::Knife, knockback: Vec3::ZERO});
        s.accept(e, PlayerId::Red, &mut r, &w);
        ensure!(owner(&mut s).saved.won[round], "Correct paired portrait was ignored");
        let saved = s.snapshot(); s.restore(&saved, &map, &mut w)?;
    }
    for _ in 0..300 {
        s.before_move(&mut r, &map, &mut w, &Default::default(), FIXED_DT)?;
        if s.owner.is_some() { break; }
    }
    scene(&mut s, &mut r, &map, &mut w)?;
    ensure!(owner(&mut s).saved.heart_open, "Three paired wins did not open heart gate");
    let e = owner(&mut s).event("Keep_Cheshire_Dead").unwrap();
    s.accept(e, PlayerId::Red, &mut r, &w);
    scene(&mut s, &mut r, &map, &mut w)?;
    if s.transition.is_none() { s.before_move(&mut r, &map, &mut w, &Default::default(), FIXED_DT)?; }
    ensure!(s.transition.as_ref().is_some_and(|d| EXIT.matches(d)), "Final Keep scene lost Queen departure");
    ensure!(owner(&mut s).coop_events().transition.is_none(), "Keep exit delivered twice");
    println!("PASS paired Keep: arrival, Red lever/scene ownership, three portraits, restored state, heart gate and final Queen departure");
    Ok(())
}
fn combat(a: &mut Assets, map: &Bsp) -> Result<()> {
    for group in ["club", "diamond", "spade", "heart"] { for victim in 0..2 {
        let mut k = Keep::load(a, &map)?; k.upgraded(); k.saved.elapsed = 5.;
        k.spawn_group(group); k.saved.spawned.truncate(1);
        let guard = &mut k.saved.spawned.first_mut().context("Missing native Keep guard")?.guard;
        match guard {
            guards::Body::Basic(g) => { g.feet = vec3(0., 0., 0.1); g.yaw = 0.; }
            guards::Body::Card(g) => { g.feet = vec3(0., 0., 0.1); g.yaw = 0.; }
        }
        let world = World::fixture(&[(vec3(-2000., -2000., -16.), vec3(2000., 2000., 0.))]);
        let mut players = [PlayerTarget {eye: vec3(0., 900., 48.), alive: true, hidden: false}; 2];
        players[victim].eye = vec3(55., 0., 48.);
        let mut hits = Feedback::default();
        for _ in 0..900 { hits.append(k.coop_combat(FIXED_DT, &world, players)); }
        ensure!(hits.players[victim].damage > 0. && hits.players[1-victim].damage == 0. && hits.damage == 0.,
            "Keep {group} lost personal damage for victim{victim}");
        let saved = k.snapshot(); k.restore(&saved, &map)?;
        players[victim].alive = false;
        for _ in 0..120 { k.coop_combat(FIXED_DT, &world, players); }
        let selected = match &k.saved.spawned[0].guard {
            guards::Body::Basic(g) => g.opponents.selected_player,
            guards::Body::Card(g) => g.opponents.selected_player,
        };
        ensure!(selected == Some(1-victim), "Keep guard retained dead target");
        println!("PASS paired Keep {group} victim{victim}: nearest target, isolated damage, saved guard and survivor retarget");
    }}
    Ok(())
}

fn rewards(a: &mut Assets, map: &Bsp) -> Result<()> {
    let mut k=Keep::load(a,map)?;k.upgraded();k.saved.elapsed=5.;
    let world=World::from_bsp(map)?;
    let mut personal:[crate::loot::Loot;2]=Default::default();
    let mut seen=std::collections::BTreeSet::new();
    for wave in 1..=3 {
        k.spawn_group("heart");
        let before=k.loot_sources();
        ensure!(before.len()==2 && before.iter().all(|s|!s.dead && seen.insert(s.id)),
            "Keep reused an earlier wave's reward identity");
        let saved=k.snapshot();k.restore(&saved,map)?;
        ensure!(k.loot_sources().iter().map(|s|s.id).collect::<Vec<_>>()
            ==before.iter().map(|s|s.id).collect::<Vec<_>>(),"Keep reward IDs changed on reload");
        for target in k.targets() {
            k.hit(Hit{id:target.id,damage:10000.,kind:crate::combat::DamageKind::Knife,knockback:Vec3::ZERO});
        }
        let after=k.loot_sources();
        ensure!(after.iter().all(|s|s.dead),"Keep reward fixture did not defeat its guards");
        for ledger in &mut personal {
            ledger.defeated(&before,&after,&world);
            ledger.defeated(&before,&after,&world);
            *ledger=serde_json::from_value(serde_json::to_value(&*ledger)?)?;
            ledger.validate()?;
            ensure!(ledger.awarded.len()==wave*2 && ledger.drops.len()==wave*2,
                "Keep lost or duplicated a personal wave reward");
        }
    }
    // Version-2 saves written before generations retain their original IDs.
    let mut legacy=k.snapshot();legacy.as_object_mut().unwrap().remove("spawn_generation");
    k.restore(&legacy,map)?;
    ensure!(k.saved.spawn_generation==0 && k.loot_sources().iter().zip(&k.saved.spawned)
        .all(|(source,actor)|source.id==BASE+actor.entity),"Legacy Keep rewards changed identity");
    let old=k.loot_sources().iter().map(|s|s.id).collect::<Vec<_>>();
    k.spawn_group("heart");
    ensure!(k.loot_sources().iter().all(|s|!old.contains(&s.id)),"Legacy Keep next wave reused old rewards");
    println!("PASS paired Keep rewards: three waves, six fresh personal drops each, duplicate suppression, reload and legacy IDs");
    Ok(())
}

pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/keep.bsp")?)?;
    let mut failed = Vec::new();
    for (name, run) in [("progression", progression as fn(&mut Assets, &Bsp) -> Result<()>), ("combat", combat), ("rewards", rewards)] {
        if let Err(error) = run(a, &map) {
            println!("FAIL paired Keep {name}: {error:#}"); failed.push(name);
        }
    }
    ensure!(failed.is_empty(), "Paired Keep failures: {failed:?}");
    Ok(())
}
