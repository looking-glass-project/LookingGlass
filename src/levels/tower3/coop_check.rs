//! Focused paired checkpoint, lift boarding, carry and save checks.
use super::*;
use crate::{inventory::Stats, level::CoopPlayer, interaction::coop::{Contacts, Motion}};
fn pair<'a>(p: &'a mut [Player; 2], s: &'a mut [Stats; 2]) -> [CoopPlayer<'a>; 2] {
    let [p, q] = p; let [s, t] = s;
    [CoopPlayer {player: p, stats: s}, CoopPlayer {player: q, stats: t}]
}
fn lift(a: &mut Assets, map: &Bsp) -> Result<Tower> {
    let mut t = Tower::load(a, map)?; t.upgraded();
    let root = t.objects.iter().position(|o| o.name == "gear03_bind").context("Missing lift")?;
    let group = motion::poses(&t.objects, root, 0.).into_iter().map(|(i, _)| i).collect::<Vec<_>>();
    for (i, o) in t.objects.iter_mut().enumerate() { o.solid &= group.contains(&i); }
    t.roots = vec![root]; t.saved.clocks = vec![0.]; t.rebuild(); Ok(t)
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/tower3.bsp")?)?;
    let mut interactions = Interactions::load(&map)?;
    interactions.set_entry(a, &map, "tower3", None)?;
    owner(&mut interactions)?.upgraded(); owner(&mut interactions)?.pair_state();
    let mut contacts = Contacts::new(&interactions);
    let contact = |id: usize| -> Option<Motion> {
        let at = interaction::vector(&map.entities[id]["origin"]).unwrap() - PLAYER_CENTER;
        Some(Motion {before: at, after: at})
    };
    interactions.triggers_pair(FIXED_DT, &mut contacts, [contact(428), contact(430)])?;
    ensure!(owner(&mut interactions)?.pair_state().room == [2, 3], "Room contact leaked between players");
    // Both bodies can enter the same repeatable checkpoint in one tick.
    contacts = Contacts::new(&interactions);
    interactions.triggers_pair(FIXED_DT, &mut contacts, [contact(4), contact(4)])?;
    ensure!(owner(&mut interactions)?.pair_state().room == [1, 1], "Shared suppression lost second personal contact");
    contacts = Contacts::new(&interactions);
    interactions.triggers_pair(FIXED_DT, &mut contacts, [contact(428), contact(430)])?;
    let t = owner(&mut interactions)?;
    let saved = t.snapshot(); let mut restored = Tower::load(a, &map)?; restored.restore(&saved, &map)?;
    for (slot, room) in [2, 3].into_iter().enumerate() {
        restored.coop_trigger_actor(Some(slot));
        for n in 1..=3 {
            let info = TriggerInfo {id: crate::entity::Id(0), name: &format!("room{n}_tele"),
                class: crate::level::TriggerClass::Teleport, thread: "", exit: None, target: None};
            ensure!(restored.gate(&info).unwrap().test(&restored.facts()) == (n == room), "Personal fall gate changed after restore");
        }
    }
    restored.coop_trigger_actor(None);
    println!("PASS paired Machinations native checkpoint contacts: split rooms, simultaneous entry and saved fall gates");
    let mut t = lift(a, &map)?; let mut w = World::fixture(&[]); w.set_dynamic(t.colliders());
    let o = t.objects.iter().find(|o| o.name == "gear03").context("Missing lift surface")?;
    let c = o.collider.as_ref().unwrap(); let mut spots = Vec::<Vec3>::new();
    for radius in [0., 48., 96., 128.] { for n in 0..16 {
        let angle = n as f32 * std::f32::consts::TAU / 16.;
        let top = o.pose.translation + vec3(radius * angle.cos(), radius * angle.sin(), 120.);
        let bottom = top - Vec3::Z * 240.;
        let hit = c.trace(top + PLAYER_CENTER, bottom + PLAYER_CENTER, PLAYER_HALF);
        let feet = top.lerp(bottom, hit.fraction);
        if !hit.start_solid && hit.fraction < 1. && hit.normal.z > 0.65 && w.body_clear(feet)
            && spots.iter().all(|p| p.distance(feet) > 60.) { spots.push(feet); }
    }}
    ensure!(spots.len() >= 2, "Missing paired lift footing");
    let mut p = [Player::new(spots[0]), Player::new(Vec3::splat(10000.))];
    let mut stats = [Stats::default(), Stats::default()];
    t.coop_advance_roster(FIXED_DT, &map, &mut w, &mut pair(&mut p, &mut stats), &[])?;
    ensure!(t.saved.clocks == [0.], "Lift left without partner");
    stats[1].damage(1000.);
    t.coop_advance_roster(FIXED_DT, &map, &mut w, &mut pair(&mut p, &mut stats), &[])?;
    ensure!(t.saved.clocks[0] > 0., "Dead partner blocked lift");
    stats[1].set_health(100.)?; t = lift(a, &map)?;
    p = [Player::new(spots[0]), Player::new(spots[1])];
    let start = p.each_ref().map(|p| p.feet);
    let mut copy = lift(a, &map)?; let mut q = p.clone();
    for tick in 0..120 {
        t.coop_advance_roster(FIXED_DT, &map, &mut w, &mut pair(&mut p, &mut stats), &[])?;
        if tick == 60 { copy.restore(&t.snapshot(), &map)?; q = p.clone(); }
        if tick > 60 {
            copy.coop_advance_roster(FIXED_DT, &map, &mut w, &mut pair(&mut q, &mut stats), &[])?;
            ensure!(copy.snapshot() == t.snapshot() && p.each_ref().map(|p| p.feet) == q.each_ref().map(|p| p.feet),
                "Paired lift continuation diverged");
        }
        ensure!(p.iter().all(|p| w.body_clear(p.feet)), "Lift embedded a rider");
    }
    ensure!((0..2).all(|i| p[i].feet.z > start[i].z + 20.), "Lift did not carry both riders");
    t.saved.coop = None; t.saved.damage = 15.; t.saved.hurt_wait = 0.3; t.pair_state();
    ensure!(t.pair_state().damage == [15., 0.] && t.pair_state().hurt_wait == [0.3, 0.], "Solo damage migration changed owner");
    t.pair_state().damage = [0., 1000.];
    let feedback = t.damage_pair(FIXED_DT, &mut pair(&mut p, &mut stats));
    ensure!(feedback.players[0].damage == 0. && feedback.players[1].damage == 1000., "Damage reached partner");
    println!("PASS paired Machinations: boarding wait, dead exclusion, two riders, saved future and personal damage");
    Ok(())
}
