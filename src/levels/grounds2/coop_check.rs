//! Shared reinforcement clocks and actual two-player guard targeting.
use super::*;
use crate::{combat::{Feedback, PlayerTarget}, coop::{protocol::Visit, stage::Stage, world::Roster}, movement::FIXED_DT};
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/grounds2.bsp")?)?;
    let mut w = World::from_bsp(&map)?;
    let visit = Visit {epoch: 1, map: "grounds2".into(), entry: None, difficulty: map.difficulty, combat_preview: false};
    let mut stage = Stage::load(a, &map, &mut w, &visit, true)?;
    let (eye, yaw) = interaction::spawn(&map, None);
    let mut roster = Roster::new_with(&w, eye, yaw, Stats::for_level("grounds2", None), |_| true)?;
    for _ in 0..3000 {
        stage.scene_step(&mut roster, &map, &mut w, &Default::default())?;
        if stage.owner.is_none() { break; }
    }
    ensure!(stage.owner.is_none() && roster.players.iter().all(|p| p.stats.alive() && w.body_clear(p.player.feet)),
        "Battle Royale arrival failed to reunite pair");
    println!("PASS paired Battle Royale: original arrival and safe two-body reunion");
    let mut r = Royale::load(a, &map)?; let mut p = Player::new(r.data.path[0]);
    r.finish(&map, &mut w, &mut p)?;
    for n in [1, 3, 5, 7] { r.event(&format!("Spawner{n:02}")); }
    let hidden = [PlayerTarget {eye: p.eye(), alive: true, hidden: true}; 2];
    for tick in 0..960 {
        r.coop_combat(FIXED_DT, &w, hidden);
        if tick == 119 { ensure!(r.saved.groups.iter().all(|g| g.count == 1), "Reinforcements advanced twice"); }
    }
    ensure!(r.saved.groups.iter().all(|g| g.count == 2)
        && r.saved.actors.iter().filter(|a| a.spawned).all(|a| a.body.health > 0. && a.flight.is_none()),
        "Paired reinforcement cadence/cap/landing changed");
    let saved = r.snapshot(); let mut resumed = Royale::load(a, &map)?; resumed.restore(&saved, &map)?;
    for _ in 0..120 {
        r.coop_combat(FIXED_DT, &w, hidden); resumed.coop_combat(FIXED_DT, &w, hidden);
        ensure!(r.snapshot() == resumed.snapshot(), "Saved paired wave future diverged");
    }
    println!("PASS paired Battle Royale: shared spawn cadence, live cap, native landings and saved continuation");
    for victim in 0..2 {
        let mut r = Royale::load(a, &map)?;
        r.saved.done = true; r.saved.time = 22.;
        r.saved.groups[0] = waves::Group {started: true, count: 2, wait: 1.1};
        for i in 0..2 { r.saved.actors[i].spawned = true; }
        r.saved.actors[1].body.health = 0.;
        r.saved.actors[0].body = crate::cards::Guard::new(crate::cards::Kind::Spade, vec3(0., 0., 0.1), 0., 1., 0);
        r.data.arena = World::fixture(&[(vec3(-2000., -2000., -16.), vec3(2000., 2000., 0.))]);
        let mut targets = [PlayerTarget {eye: vec3(0., 900., 48.), alive: true, hidden: false}; 2];
        targets[victim].eye = vec3(55., 0., 48.);
        let mut feedback = Feedback::default();
        for _ in 0..600 { feedback.append(r.coop_combat(FIXED_DT, &World::fixture(&[]), targets)); }
        ensure!(feedback.players[victim].damage > 0. && feedback.players[1-victim].damage == 0. && feedback.damage == 0.,
            "Wave guard did not isolate nearest victim {victim}");
        targets[victim].alive = false;
        r.saved.actors[0].body.set(crate::cards::Phase::Idle);
        r.coop_combat(FIXED_DT, &World::fixture(&[]), targets);
        ensure!(r.saved.actors[0].body.opponents.selected_player == Some(1-victim), "Guard targeted dead Alice");
        println!("PASS paired Battle Royale victim{victim}: nearest target, personal damage and survivor retarget");
    }
    Ok(())
}
