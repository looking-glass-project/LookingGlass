//! Native all-aboard lift activation and separate gate contacts.
use super::*;
use crate::{coop::{protocol::Visit, stage::Stage, world::Roster, PlayerId}, inventory::Stats, movement::FIXED_DT};
fn owner(s: &mut Stage) -> &mut Facade {
    s.interactions.levels.iter_mut().find_map(|s| s.ctl.downcast_mut()).unwrap()
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/facade.bsp")?)?;
    for first in PlayerId::ALL {
        let visit = Visit {epoch: 1, map: "facade".into(), entry: None, difficulty: map.difficulty, combat_preview: false};
        let mut w = World::from_bsp(&map)?;
        let mut s = Stage::load(a, &map, &mut w, &visit, true)?;
        let (eye, yaw) = interaction::spawn(&map, None);
        let mut r = Roster::new_with(&w, eye, yaw, Stats::for_level("facade", None), |_| true)?;
        let deck = w.actor_footing(vec3(320., 2976., -1500.), PLAYER_CENTER, PLAYER_HALF, 200.)
            .context("Missing lift deck")?;
        r.players[first.index()].player.feet = deck;
        let before = r.players.each_ref().map(|b| b.player.feet);
        s.contacts(&mut r, &w, before)?;
        ensure!(s.owner.is_none() && owner(&mut s).saved.ride.is_none(), "Lift left without living partner");
        let deck2 = w.actor_footing(deck + vec3(48., 0., 64.), PLAYER_CENTER, PLAYER_HALF, 100.)
            .context("Missing partner deck")?;
        r.players[first.partner().index()].player.feet = deck2;
        let before = r.players.each_ref().map(|b| b.player.feet);
        s.contacts(&mut r, &w, before)?;
        ensure!(s.owner == Some(first), "Lift lost first boarding owner");
        let saved = s.snapshot(); s.restore(&saved, &map, &mut w)?;
        for _ in 0..600 {
            s.scene_step(&mut r, &map, &mut w, &Default::default())?;
            if s.transition.is_some() { break; }
        }
        ensure!(s.transition.as_ref().is_some_and(|d| EXIT.matches(d)), "Paired lift did not depart to Keep");
        ensure!(owner(&mut s).coop_events().transition.is_none(), "Lift delivered exit twice");
        println!("PASS paired Ascension {first:?}: native boarding wait, original owner, saved ride and one Keep departure");
    }
    for blocker in 0..2 {
        let mut f = Facade::load(a, &map)?;
        let mut w = World::from_bsp(&map)?;
        f.saved.gate = 1.; f.saved.gate_opening = false; f.rebuild(&map)?;
        w.set_dynamic(f.colliders());
        let doorway = w.actor_footing(vec3(320., 464., -1450.), PLAYER_CENTER, PLAYER_HALF, 300.)
            .context("Missing gateway floor")?;
        let (eye, _) = interaction::spawn(&map, None);
        let mut p = [Player::spawn(&w, eye).context("Missing entrance")?, Player::new(doorway)];
        p.swap(1, blocker);
        for _ in 0..600 { f.coop_advance(FIXED_DT, &map, &mut w, &mut p.each_mut(), &[])?; }
        ensure!(w.body_clear(p[blocker].feet) && f.saved.gate > 0., "Gate ignored blocker {blocker}");
    }
    println!("PASS paired Ascension gates respect both bodies");
    Ok(())
}
