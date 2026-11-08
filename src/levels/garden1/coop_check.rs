//! Focused native paired contracts; these are not full campaign traversals.
use super::*;
use crate::{
    collision::{PLAYER_CENTER, PLAYER_HALF},
    coop::{protocol::Visit, stage::Stage, world::Roster, PlayerId},
    level::CoopPlayer,
    movement::{Controls, FIXED_DT},
};

fn bodies(pair: &mut [(Player, Stats); 2]) -> [CoopPlayer<'_>; 2] {
    pair.each_mut()
        .map(|(player, stats)| CoopPlayer { player, stats })
}
fn owner(stage: &mut Stage) -> &mut Garden {
    stage
        .interactions
        .levels
        .iter_mut()
        .find_map(|s| s.ctl.downcast_mut::<Garden>())
        .unwrap()
}
fn pair() -> [(Player, Stats); 2] {
    std::array::from_fn(|_| (Player::new(Vec3::ZERO), Stats::for_level("garden1", None)))
}
fn scene(
    stage: &mut Stage,
    roster: &mut Roster,
    map: &Bsp,
    world: &mut World,
    skip: bool,
    expected: PlayerId,
) -> Result<()> {
    let waiting = roster.players[expected.partner().index()].player.feet;
    for tick in 0..120 * 180 {
        ensure!(stage.owner == Some(expected), "Garden scene changed actor");
        if skip && tick == 24 {
            stage.interactions.skip_cinematic(
                map,
                world,
                &mut roster.players[expected.index()].player,
                &mut stage.story,
            )?;
        }
        stage.scene_step(roster, map, world, &Default::default())?;
        if stage.owner.is_none() {
            break;
        }
        ensure!(
            roster.players[expected.partner().index()].player.feet == waiting,
            "Garden scene moved waiting partner"
        );
        if tick % 120 == 0 {
            let saved = stage.snapshot();
            let feet = roster.players.each_ref().map(|b| b.player.feet);
            stage.before_move(roster, map, world, &Default::default(), 0.)?;
            ensure!(
                serde_json::to_value(&saved)? == serde_json::to_value(stage.snapshot())?
                    && feet == roster.players.each_ref().map(|b| b.player.feet),
                "Paused Garden scene advanced"
            );
            let encoded = crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
            stage.restore(
                &serde_json::from_slice(&crate::coop::codec::decode(&encoded)?)?,
                map,
                world,
            )?;
        }
    }
    ensure!(stage.owner.is_none(), "Garden scene failed to finish");
    ensure!(
        roster
            .players
            .iter()
            .all(|b| world.body_clear(b.player.feet)),
        "Garden reunion embedded a player"
    );
    ensure!(
        roster.players[0]
            .player
            .feet
            .distance(roster.players[1].player.feet)
            >= 36.,
        "Garden reunion overlapped players"
    );
    stage.before_move(roster, map, world, &Default::default(), FIXED_DT)?;
    Ok(())
}
fn scenes(a: &mut Assets, map: &Bsp) -> Result<()> {
    let visit = Visit {
        epoch: 1,
        map: "garden1".into(),
        entry: None,
        difficulty: Default::default(),
        combat_preview: false,
    };
    for first in PlayerId::ALL {
        for skip in [false, true] {
            let mut world = World::from_bsp(map)?;
            let mut stage = Stage::load(a, map, &mut world, &visit, false)?;
            let (eye, yaw) = crate::interaction::spawn(map, None);
            let mut roster = Roster::new(&world, eye, yaw, Stats::for_level("garden1", None))?;
            for (i, b) in roster.players.iter_mut().enumerate() {
                b.stats.set_health(41. - i as f32 * 17.)?;
                b.stats.spend_will(20. + i as f32 * 17.);
                b.stats.turtle_air = false;
            }
            // The automatic arrival is a simultaneous start (Blue wins the tie).
            stage.scene_step(&mut roster, map, &mut world, &Default::default())?;
            ensure!(
                roster
                    .players
                    .iter()
                    .all(|b| b.stats.sanity() == 100. && b.stats.will() == 100.),
                "Garden entry did not restore both players"
            );
            for (i, b) in roster.players.iter_mut().enumerate() {
                b.stats.set_health(63. - i as f32 * 16.)?;
                b.stats.spend_will(17. + i as f32 * 14.);
            }
            scene(
                &mut stage,
                &mut roster,
                map,
                &mut world,
                skip,
                PlayerId::Blue,
            )?;
            ensure!(
                roster
                    .players
                    .iter()
                    .all(|b| b.stats.turtle_air && b.player.breath.shell),
                "Garden scene shell grant missed the waiting player"
            );
            ensure!(
                roster.players.each_ref().map(|b| b.stats.sanity()) == [63., 47.]
                    && roster.players.each_ref().map(|b| b.stats.will()) == [83., 69.],
                "Garden entry healed twice"
            );
            // Stage only the approach; ordinary movement enters the native Rabbit trigger.
            roster.players[first.index()].player = Player::spawn(&world, vec3(-1280., -316., 736.))
                .context("Garden Rabbit approach blocked")?;
            for _ in 0..240 {
                let before = roster.players.each_ref().map(|b| b.player.feet);
                roster.players[first.index()].player.tick(
                    &world,
                    Controls {
                        wish: Vec2::X,
                        ..Default::default()
                    },
                );
                stage.contacts(&mut roster, &world, before)?;
                if stage.owner.is_some() {
                    break;
                }
            }
            ensure!(
                stage.owner == Some(first) && owner(&mut stage).scene_id() == Some(RABBIT),
                "First Garden Rabbit contact did not own scene"
            );
            scene(&mut stage, &mut roster, map, &mut world, skip, first)?;
            ensure!(
                owner(&mut stage).saved.rabbit_done
                    && roster.players.iter().all(|b| b.stats.alive()),
                "Garden Rabbit completion missing"
            );
            println!("PASS Garden {first:?} skip={skip}: paired one-time arrival, both shells, native Rabbit ownership, frozen partner, pause/save and clear reunion");
        }
    }
    Ok(())
}
fn riders(g: &Garden, index: usize, w: &World) -> Result<[Player; 2]> {
    let pad = &g.movers.pads[index];
    let mut points = Vec::new();
    for x in (-80..=80).step_by(16) {
        for y in (-80..=80).step_by(16) {
            let top = pad.pose.translation + vec3(x as f32, y as f32, 128.);
            let trace = pad.collider.trace(
                top + PLAYER_CENTER,
                top - Vec3::Z * 256. + PLAYER_CENTER,
                PLAYER_HALF,
            );
            let feet = top - Vec3::Z * 256. * trace.fraction;
            if !trace.start_solid
                && trace.fraction < 1.
                && trace.normal.z > 0.65
                && w.body_clear(feet)
            {
                points.push(feet);
            }
        }
    }
    let (one, two) = points
        .iter()
        .flat_map(|a| points.iter().map(move |b| (*a, *b)))
        .filter(|(a, b)| a.distance(*b) >= 48. && (a.z - b.z).abs() < 2.)
        .min_by(|(a, b), (c, d)| {
            let cost = |one: Vec3, two: Vec3| {
                (one - pad.pose.translation).truncate().length_squared()
                    + (two - pad.pose.translation).truncate().length_squared()
            };
            cost(*a, *b).total_cmp(&cost(*c, *d))
        })
        .context("Garden pad has no separated pair of supported positions")?;
    Ok([Player::new(one), Player::new(two)])
}
fn pads(a: &mut Assets, map: &Bsp) -> Result<()> {
    for hz in [30, 60, 144] {
        for index in 0..3 {
            let mut g = Garden::load(a, map)?;
            g.upgraded();
            let mut w = World::from_bsp(map)?;
            w.set_dynamic(g.colliders());
            let mut players = pair();
            for (body, player) in players.iter_mut().zip(riders(&g, index, &w)?) {
                body.0 = player;
            }
            let local = players
                .each_ref()
                .map(|b| b.0.feet - g.movers.pads[index].pose.translation);
            for _ in 0..hz * 75 {
                g.advance_roster(1. / hz as f32, map, &mut w, &mut bodies(&mut players), &[])?;
                ensure!(
                    players.iter().all(|b| w.body_clear(b.0.feet)),
                    "Garden pad embedded a rider"
                );
                for (body, offset) in players.iter().zip(local) {
                    ensure!(
                        (body.0.feet - g.movers.pads[index].pose.translation - offset).length()
                            < 1.,
                        "Garden pad{index} lost rider: {:?}",
                        body.0.feet
                    );
                }
            }
            ensure!(
                g.saved.world.coop.as_ref().unwrap().pads[index] > 1.,
                "Garden paired pad never moved"
            );
            let saved = g.snapshot();
            let mut resumed = Garden::load(a, map)?;
            resumed.restore(&saved, map)?;
            let mut other = players.clone();
            let mut other_world = World::from_bsp(map)?;
            for _ in 0..120 {
                g.advance_roster(FIXED_DT, map, &mut w, &mut bodies(&mut players), &[])?;
                resumed.advance_roster(
                    FIXED_DT,
                    map,
                    &mut other_world,
                    &mut bodies(&mut other),
                    &[],
                )?;
                ensure!(
                    g.snapshot() == resumed.snapshot()
                        && serde_json::to_value(&players)? == serde_json::to_value(&other)?,
                    "Garden paired pad restore diverged"
                );
            }
            let before = g.snapshot();
            let feet = players.each_ref().map(|p| p.0.feet);
            g.advance_roster(0., map, &mut w, &mut bodies(&mut players), &[])?;
            ensure!(
                g.snapshot() == before && feet == players.each_ref().map(|p| p.0.feet),
                "Paused Garden pads advanced"
            );
            println!("PASS Garden pad{index} {hz}Hz: two separated riders, 75 seconds of shared support, pause and exact saved continuation");
        }
    }
    Ok(())
}

fn pad_obstructions(a: &mut Assets, map: &Bsp) -> Result<()> {
    for index in 0..3 {
        for blocked in 0..2 {
            let mut exercised = false;
            let mut samples = Vec::new();
            // Choose a native phase with enough horizontal motion to place a
            // thin obstacle beyond today's hull but inside the proposed sweep.
            for phase in 0..75 {
                let time = phase as f32;
                let mut g = Garden::load(a, map)?;
                g.upgraded();
                g.saved.world.time = time;
                g.movers.rebuild(map, &g.saved.world)?;
                let mut w = World::from_bsp(map)?;
                w.set_dynamic(g.colliders());
                let Ok(supported) = riders(&g, index, &w) else {
                    continue;
                };
                let mut players = pair();
                for (b, p) in players.iter_mut().zip(supported) {
                    b.0 = p;
                }
                g.prepare_roster(&mut bodies(&mut players));
                let saved = g.snapshot();
                let before = players.clone();
                g.advance_roster(FIXED_DT, map, &mut w, &mut bodies(&mut players), &[])?;
                let delta = players[blocked].0.feet - before[blocked].0.feet;
                let axis = if delta.x.abs() > delta.y.abs() { 0 } else { 1 };
                samples.push((time, delta));
                if delta[axis].abs() < 0.01 {
                    continue;
                }
                let mut center = before[blocked].0.feet + PLAYER_CENTER;
                let gap = (delta[axis].abs() * 0.25).min(0.08);
                center[axis] += delta[axis].signum() * (PLAYER_HALF[axis] + gap + 0.02);
                let mut half = Vec3::splat(4.);
                half[axis] = 0.02;
                let obstacle = Collider::box_bounds(center - half, center + half);
                g.restore(&saved, map)?;
                players = before.clone();
                w.set_dynamic(
                    g.colliders()
                        .into_iter()
                        .chain([obstacle.clone()])
                        .collect(),
                );
                ensure!(
                    players.iter().all(|b| w.body_clear(b.0.feet)),
                    "Pad obstruction starts inside a player"
                );
                g.advance_roster(
                    FIXED_DT,
                    map,
                    &mut w,
                    &mut bodies(&mut players),
                    &[obstacle.clone()],
                )?;
                let clocks = g.saved.world.coop.as_ref().unwrap();
                ensure!(
                    clocks.pads[index] == time
                        && g.saved.world.time > time
                        && clocks
                            .pads
                            .iter()
                            .enumerate()
                            .all(|(i, t)| i == index || *t > time),
                    "Blocked pad stopped independent Garden clocks"
                );
                ensure!(
                    serde_json::to_value(&players)? == serde_json::to_value(&before)?,
                    "Blocked Garden pad partially carried a rider"
                );
                let blocked_save = g.snapshot();
                g.restore(&blocked_save, map)?;
                ensure!(
                    g.snapshot() == blocked_save,
                    "Stopped pad lost its saved clock"
                );
                g.advance_roster(FIXED_DT, map, &mut w, &mut bodies(&mut players), &[])?;
                ensure!(
                    g.saved.world.coop.as_ref().unwrap().pads[index] > time
                        && players
                            .iter()
                            .zip(&before)
                            .all(|(p, b)| p.0.feet.distance(b.0.feet) > delta[axis].abs() * 0.5),
                    "Garden pad failed to resume both riders after clearing obstruction"
                );

                // A corpse at the obstructed seat must neither move nor hold
                // the living partner's pad. Revival placement is owned elsewhere.
                g.restore(&saved, map)?;
                players = before.clone();
                players[blocked].1.set_health(0.)?;
                let corpse = players[blocked].0.clone();
                g.advance_roster(
                    FIXED_DT,
                    map,
                    &mut w,
                    &mut bodies(&mut players),
                    &[obstacle],
                )?;
                ensure!(
                    g.saved.world.coop.as_ref().unwrap().pads[index] > time
                        && players[1 - blocked]
                            .0
                            .feet
                            .distance(before[1 - blocked].0.feet)
                            > delta[axis].abs() * 0.5
                        && serde_json::to_value(&players[blocked].0)?
                            == serde_json::to_value(&corpse)?,
                    "Dead Garden rider blocked movement or was carried"
                );
                exercised = true;
                break;
            }
            ensure!(
                exercised,
                "No native phase exercised Garden pad{index} blocked slot{blocked}: {samples:?}"
            );
            println!("PASS Garden pad{index} blocked slot{blocked}: atomic stop, independent clocks, saved stop/resume and dead rider exclusion");
        }
    }
    Ok(())
}

fn bridge_clock(a: &mut Assets, map: &Bsp) -> Result<()> {
    let mut g = Garden::load(a, map)?;
    g.upgraded();
    let mut players = pair();
    for b in &mut players {
        b.1.set_health(0.)?;
    }
    let mut w = World::from_bsp(map)?;
    g.event("Bridge_Drop");
    for _ in 0..30 {
        g.advance_roster(0.1, map, &mut w, &mut bodies(&mut players), &[])?;
    }
    let saved = g.snapshot();
    let mut restored = Garden::load(a, map)?;
    restored.restore(&saved, map)?;
    let mut other = players.clone();
    let mut other_world = World::from_bsp(map)?;
    for _ in 0..150 {
        g.advance_roster(0.1, map, &mut w, &mut bodies(&mut players), &[])?;
        restored.advance_roster(0.1, map, &mut other_world, &mut bodies(&mut other), &[])?;
        ensure!(
            g.snapshot() == restored.snapshot() && g.transforms() == restored.transforms(),
            "Garden bridge co-op clock restore diverged"
        );
    }
    ensure!(
        g.transforms().len() == 1,
        "Garden co-op bridge did not retire its seven pieces"
    );
    println!("PASS Garden paired bridge clock: activation, saved continuation and all seven pieces retired");
    Ok(())
}
fn resources_and_migration(a: &mut Assets, map: &Bsp) -> Result<()> {
    for dead in 0..2 {
        let mut g = Garden::load(a, map)?;
        let mut p = pair();
        p[dead].1.set_health(0.)?;
        p[1 - dead].1.set_health(29.)?;
        g.prepare_roster(&mut bodies(&mut p));
        ensure!(
            p[dead].1.sanity() == 0. && p[1 - dead].1.sanity() == 100.,
            "Garden entry revived a dead player"
        );
        p[dead].1.set_health(25.)?;
        g.prepare_roster(&mut bodies(&mut p));
        ensure!(
            p[dead].1.sanity() == 25.,
            "Garden respawn repeated entry heal"
        );
    }
    let mut g = Garden::load(a, map)?;
    g.upgraded();
    g.saved.world.time = 37.5;
    g.movers.rebuild(map, &g.saved.world)?;
    let old = g.snapshot();
    ensure!(
        old["world"].get("coop").is_none(),
        "Single-player save gained paired state"
    );
    let poses = g
        .movers
        .pads
        .iter()
        .map(|p| p.pose.translation)
        .collect::<Vec<_>>();
    let mut restored = Garden::load(a, map)?;
    restored.restore(&old, map)?;
    let mut p = pair();
    p[0].1.set_health(55.)?;
    p[1].1.set_health(21.)?;
    restored.prepare_roster(&mut bodies(&mut p));
    restored.movers.rebuild(map, &restored.saved.world)?;
    ensure!(
        restored
            .movers
            .pads
            .iter()
            .map(|p| p.pose.translation)
            .collect::<Vec<_>>()
            == poses
            && p.each_ref().map(|b| b.1.sanity()) == [55., 21.],
        "Garden migration replayed arrival or moved pads"
    );
    let before = restored.snapshot();
    let mut bad = before.clone();
    bad["world"]["coop"]["pads"][1] = serde_json::json!(38.);
    ensure!(
        restored.restore(&bad, map).is_err() && restored.snapshot() == before,
        "Invalid Garden paired clock partially applied"
    );
    println!("PASS Garden personal arrival/dead partner/respawn and old-save migration with atomic invalid-state rejection");
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/garden1.bsp")?)?;
    resources_and_migration(a, &map)?;
    scenes(a, &map)?;
    pads(a, &map)?;
    pad_obstructions(a, &map)?;
    bridge_clock(a, &map)?;
    if let Ok(path) = std::env::var("LOOKING_GLASS_GARDEN_CONTACT_REPORT") {
        replay_contact(a, &map, &path)?;
    }
    Ok(())
}

/// Optional exact captured-state regression. Only an offline copy is corrected;
/// live characters, resources and saved gameplay are never changed by this check.
fn replay_contact(a: &mut Assets, map: &Bsp, path: &str) -> Result<()> {
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let mut frame: crate::coop::world::Frame = serde_json::from_value(report["frame"].clone())?;
    frame.validate()?;
    let snapshot: crate::coop::arena::Snapshot = serde_json::from_value(report["world"].clone())?;
    let previous: [Vec3; 2] = serde_json::from_value(report["before"].clone())?;
    ensure!(
        previous.iter().all(|p| p.is_finite()),
        "Invalid contact history"
    );
    let visit = Visit {
        epoch: 1,
        map: "garden1".into(),
        entry: Some("garden1_start1".into()),
        difficulty: Default::default(),
        combat_preview: false,
    };
    let mut world = World::from_bsp(map)?;
    let mut stage = Stage::load(a, map, &mut world, &visit, false)?;
    stage.restore(&snapshot.stage, map, &mut world)?;
    let before = frame.players.clone();
    ensure!(
        before
            .iter()
            .all(|b| b.stats.alive() && b.player.grounded && world.body_clear(b.player.feet)),
        "Contact report is not two living supported players"
    );
    let old_gap = (before[0].player.feet - before[1].player.feet)
        .truncate()
        .length();
    ensure!(
        old_gap < 20.,
        "Contact report does not reproduce the overlap"
    );
    crate::coop::separation::resolve(&world, &mut frame.players, previous);
    let gap = (frame.players[0].player.feet - frame.players[1].player.feet)
        .truncate()
        .length();
    ensure!(
        gap >= crate::coop::separation::DISTANCE - 0.1
            && frame
                .players
                .iter()
                .all(|b| world.body_clear(b.player.feet)),
        "Native Garden contact still overlaps: {old_gap} -> {gap}"
    );
    for (i, (old, new)) in before.iter().zip(&frame.players).enumerate() {
        ensure!(
            serde_json::to_value(&old.stats)? == serde_json::to_value(&new.stats)?,
            "Contact changed resources"
        );
        if old.player.feet == previous[i] {
            ensure!(
                new.player.feet == old.player.feet,
                "Contact moved stationary native partner"
            );
        }
    }
    println!("PASS exact captured Garden bank contact: {old_gap:.3} -> {gap:.3}, clear bodies, stationary partner and personal resources preserved");
    Ok(())
}
