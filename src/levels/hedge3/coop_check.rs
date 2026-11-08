//! Focused shared clock, two-rider and saved personal-damage checks.
use super::*;
use crate::{inventory::Stats, level::CoopPlayer};
fn pair<'a>(p: &'a mut [Player; 2], s: &'a mut [Stats; 2]) -> [CoopPlayer<'a>; 2] {
    let [p, q] = p; let [s, t] = s;
    [CoopPlayer {player: p, stats: s}, CoopPlayer {player: q, stats: t}]
}
fn isolated(map: &Bsp) -> Result<Labyrinth> {
    let mut m = Labyrinth::load(map)?;
    m.objects.retain(|o| o.name == "lavagear");
    ensure!(m.objects.len() == 1, "Missing shared gear");
    m.saved.machines = vec![Mechanism::default()]; m.rebuild(); Ok(m)
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/hedge3.bsp")?)?;
    let mut m = Labyrinth::load(&map)?;
    let mut solo = Labyrinth::load(&map)?;
    let mut w = World::fixture(&[]);
    let mut p = [Player::new(Vec3::splat(10000.)), Player::new(Vec3::splat(11000.))];
    let mut stats = [Stats::default(), Stats::default()];
    let mut one = p[0].clone();
    for _ in 0..120 {
        m.coop_advance_roster(FIXED_DT, &map, &mut w, &mut pair(&mut p, &mut stats), &[])?;
        solo.advance(FIXED_DT, &map, &mut w, &mut one, &[])?;
        ensure!(m.snapshot()["machines"] == solo.snapshot()["machines"], "Shared machinery clock ran twice");
    }
    m.saved.coop = None; m.saved.crush = true; m.pair_state();
    let saved = m.snapshot(); let mut copy = Labyrinth::load(&map)?; copy.restore(&saved, &map)?;
    let hit = copy.damage_pair(FIXED_DT, &mut pair(&mut p, &mut stats));
    ensure!(hit.players[0].damage == 1000. && hit.players[1].damage == 0., "Imported crush assigned to wrong player");
    ensure!(copy.damage_pair(FIXED_DT, &mut pair(&mut p, &mut stats)).players[0].damage == 0., "Crush delivered twice");
    let mut m = isolated(&map)?;
    let o = &m.objects[0]; let mut spots = Vec::<Vec3>::new();
    for x in -6..=6 { for y in -6..=6 {
        let top = o.base + vec3(x as f32 * 36., y as f32 * 36., 600.);
        let bottom = top - Vec3::Z * 1000.;
        let hit = o.collider.trace(top + PLAYER_CENTER, bottom + PLAYER_CENTER, PLAYER_HALF);
        let feet = top.lerp(bottom, hit.fraction);
        if !hit.start_solid && hit.fraction < 1. && hit.normal.z > 0.99
            && spots.iter().all(|p| p.distance(feet) > 80.) {
            spots.push(feet);
        }
    }}
    ensure!(spots.len() >= 2, "Missing paired gear footing");
    p = [Player::new(spots[0]), Player::new(spots[1])];
    let start = p.each_ref().map(|p| p.feet);
    let mut copy = isolated(&map)?; let mut q = p.clone();
    for tick in 0..120 {
        m.coop_advance_roster(FIXED_DT, &map, &mut w, &mut pair(&mut p, &mut stats), &[])?;
        if tick == 60 { copy.restore(&m.snapshot(), &map)?; q = p.clone(); }
        if tick > 60 {
            copy.coop_advance_roster(FIXED_DT, &map, &mut w, &mut pair(&mut q, &mut stats), &[])?;
            ensure!(copy.snapshot() == m.snapshot() && p.each_ref().map(|p| p.feet) == q.each_ref().map(|p| p.feet),
                "Paired gear continuation diverged");
        }
        ensure!(p.iter().all(|p| w.body_clear(p.feet)), "Gear embedded a rider");
    }
    ensure!((0..2).all(|i| p[i].feet.distance(start[i]) > 1.), "Gear did not carry both riders");
    println!("PASS paired Labyrinth: one clock, legacy pending hit, personal delivery, two riders and exact saved future");
    Ok(())
}
