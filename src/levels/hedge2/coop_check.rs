//! Focused paired lever, breathing and closing-door checks on the supplied map.
use super::*;
use crate::level::CoopPlayer;
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/hedge2.bsp")?)?;
    let mut w = World::from_bsp(&map)?;
    for actor in 0..2 {
        let mut m = Maze::load(a, &map)?;
        let mut players = [Player::new(m.bubbles[0]), Player::new(Vec3::splat(10000.))];
        let mut stats = [Stats::default(), Stats::default()];
        for p in &mut players { p.breath.submerged = 19.; }
        let [p, q] = &mut players; let [s, t] = &mut stats;
        let mut pair = [CoopPlayer {player: p, stats: s}, CoopPlayer {player: q, stats: t}];
        m.coop_prepare(&mut pair);
        ensure!(pair.iter().all(|p| p.stats.turtle_air && p.player.breath.shell)
            && pair[0].player.breath.remaining() == 20. && pair[1].player.breath.remaining() == 1.,
            "Paired bubble contact shared breath or omitted shell");
        for k in 0..2 {
            let mut p = Player::new(m.levers[k].translation);
            w.set_dynamic(m.colliders());
            m.coop_update(actor, &mut w, &p, m.levers[k].rotation * Vec3::X, true);
            ensure!(m.saved.used[k] && m.scripted(), "Paired use failed to start lever {k}");
            for _ in 0..2160 { m.advance(1./120., &map, &mut w, &mut p, &[])?; }
            ensure!(m.saved.finished[k], "Paired lever did not finish");
        }
        ensure!(m.ready(), "Paired levers did not open exit");
        let saved = m.snapshot();
        let mut resumed = Maze::load(a, &map)?;
        resumed.restore(&saved, &map)?;
        ensure!(resumed.snapshot() == saved, "Paired gates lost on restore");
        println!("PASS paired Mystifying Madness actor{actor}: personal breath, both use inputs, gate completion and restore");
    }
    for blocker in 0..2 {
        let mut m = Maze::load(a, &map)?;
        m.saved.used[0] = true; m.saved.fired[0] = true; m.saved.finished[0] = true;
        m.saved.release = Some(0.1); m.saved.motion[0] = 0.; m.rebuild(&map)?;
        // Door 1 opens into this body; the other body is safely distant.
        m.saved.motion[1] = 0.1 / m.objects[1].duration; m.rebuild(&map)?;
        let contact = m.objects[1].collider.interior_point().context("Missing gate hull")? - PLAYER_CENTER;
        m.saved.motion[1] = 0.; m.rebuild(&map)?;
        let mut p = [Player::new(Vec3::splat(10000.)), Player::new(Vec3::splat(11000.))];
        p[blocker].feet = contact;
        m.coop_advance(0.1, &map, &mut w, &mut p.each_mut(), &[])?;
        ensure!(m.saved.motion[1] == 0., "Door ignored body {blocker}");
    }
    println!("PASS paired Mystifying Madness doors respect both bodies");
    Ok(())
}
