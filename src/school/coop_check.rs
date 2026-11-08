//! Focused native-data fixtures, not a traversal or campaign completion claim.
use super::*;
use crate::movement::{Controls, FixedClock, FIXED_DT};
use anyhow::ensure;

pub(crate) const LIFTS: [(&str, &str, Vec3, f32); 2] = [
    (
        "Skool1_Lift_Up",
        "library_spiral_lift",
        Vec3::new(144., 4360., -263.96),
        64.,
    ),
    (
        "Skool1_Elevator_Up",
        "elevator2",
        Vec3::new(-276., 4360., 400.04),
        696.,
    ),
];

/// Find two non-overlapping upright bodies on the authored support. Tests still
/// require both to traverse the complete lift cycle without leaving it.
pub(crate) fn riders(school: &School, world: &World, name: &str, at: Vec3) -> Result<[Player; 2]> {
    let collider = &school.object(name).collider;
    ensure!(
        world.body_clear(at) && crate::moving::aboard(collider, at),
        "Invalid first rider on {name}: {at:?}"
    );
    // Prefer the inside of the spiral: its outside edge passes under a landing
    // lip near the top. Obstructed positions are covered separately below.
    let partner = [Vec3::X, -Vec3::Y, Vec3::Y, -Vec3::X]
        .into_iter()
        .flat_map(|d| [48., 64., 40.].map(|n| at + d * n))
        .find(|p| world.body_clear(*p) && crate::moving::aboard(collider, *p))
        .with_context(|| format!("No second rider fits on {name}"))?;
    Ok([at, partner].map(|p| {
        let mut p = Player::new(p);
        p.grounded = true;
        p
    }))
}

pub fn check(assets: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&assets.read("maps/skool1.bsp")?)?;
    landed_partner(&map)?;
    for (thread, name, start, top) in LIFTS {
        for fps in [30, 60, 144] {
            let mut school = School::load(&map, false)?;
            let mut world = World::from_bsp(&map)?;
            world.set_dynamic(school.colliders().collect());
            let mut players = riders(&school, &world, name, start)?;
            let separation = players[0].feet.distance(players[1].feet);
            school.event(thread).context("Missing lift event")?;
            let before = serde_json::to_value(school.snapshot())?;
            school.advance_players(0., &map, &mut world, &mut players.each_mut(), &[])?;
            ensure!(
                before == serde_json::to_value(school.snapshot())?,
                "Paused paired lift advanced"
            );
            let mut highest = players.each_ref().map(|p| p.feet.z);
            let mut clocks: [FixedClock; 2] = Default::default();
            let mut mirror = School::load(&map, false)?;
            let mut mirror_world = World::from_bsp(&map)?;
            for frame in 0..fps * 25 {
                let replicate = frame % fps == 0;
                let mut copies = players.clone();
                if replicate {
                    let state = serde_json::from_slice(&serde_json::to_vec(&school.snapshot())?)?;
                    mirror.restore(&state, &map)?;
                    mirror_world.set_dynamic(mirror.colliders().collect());
                    mirror.advance_players(
                        1. / fps as f32,
                        &map,
                        &mut mirror_world,
                        &mut copies.each_mut(),
                        &[],
                    )?;
                }
                school.advance_players(
                    1. / fps as f32,
                    &map,
                    &mut world,
                    &mut players.each_mut(),
                    &[],
                )?;
                if replicate {
                    ensure!(
                        copies.each_ref().map(|p| p.feet) == players.each_ref().map(|p| p.feet)
                            && serde_json::to_value(mirror.snapshot())?
                                == serde_json::to_value(school.snapshot())?,
                        "Restored lift diverged from live world"
                    );
                }
                for (i, p) in players.iter_mut().enumerate() {
                    ensure!(
                        world.body_clear(p.feet),
                        "{name} embedded rider {i} before physics, {fps} Hz frame {frame}: {:?}",
                        p.feet
                    );
                    clocks[i].advance(1. / fps as f64, &world, p, Controls::default());
                    ensure!(
                        world.body_clear(p.feet),
                        "{name} embedded rider {i} after physics, {fps} Hz"
                    );
                    highest[i] = highest[i].max(p.feet.z);
                }
                ensure!(
                    (players[0].feet.distance(players[1].feet) - separation).abs() < 0.25,
                    "{name} rider offsets drifted at {fps} Hz frame {frame}: {:?}",
                    players.each_ref().map(|p| p.feet)
                );
            }
            ensure!(
                highest.iter().all(|h| *h >= top - 1.),
                "{name} failed to carry both to top: {highest:?}"
            );
            ensure!(
                (school.clock - 25.).abs() < 0.01,
                "School clock advanced per rider"
            );
            println!("PASS co-op {name} {fps} Hz: both riders reached {highest:?}, offsets retained, clear bodies before/after physics, paused clock and restored continuation");
        }

        // A temporary ceiling over either rider must stop the common clock and
        // both bodies atomically, then resume when that obstruction is removed.
        for blocked in 0..2 {
            let mut school = School::load(&map, false)?;
            let mut world = World::from_bsp(&map)?;
            world.set_dynamic(school.colliders().collect());
            let mut players = riders(&school, &world, name, start)?;
            school.event(thread);
            let at = players[blocked].feet;
            let height = PLAYER_CENTER.z + PLAYER_HALF.z;
            let roof = Collider::box_bounds(
                at + vec3(-18., -18., height + 1.),
                at + vec3(18., 18., height + 17.),
            );
            for _ in 0..360 {
                school.advance_players(
                    FIXED_DT,
                    &map,
                    &mut world,
                    &mut players.each_mut(),
                    std::slice::from_ref(&roof),
                )?;
            }
            let stopped = players.each_ref().map(|p| p.feet);
            let time = school.object(name).motion.as_ref().unwrap().time;
            for _ in 0..120 {
                school.advance_players(
                    FIXED_DT,
                    &map,
                    &mut world,
                    &mut players.each_mut(),
                    std::slice::from_ref(&roof),
                )?;
            }
            ensure!(
                time < 1.1
                    && school.object(name).motion.as_ref().unwrap().time == time
                    && players.each_ref().map(|p| p.feet) == stopped,
                "{name} did not stop atomically for rider {blocked}"
            );
            for _ in 0..120 {
                school.advance_players(FIXED_DT, &map, &mut world, &mut players.each_mut(), &[])?;
            }
            ensure!(
                players
                    .iter()
                    .zip(stopped)
                    .all(|(p, at)| p.feet.z > at.z + 10.),
                "{name} failed to resume"
            );
        }
        println!("PASS co-op {name}: obstruction over either rider stops both and lift clock; removal resumes");

        // Hold each departure if just one passenger is aboard; permit an empty
        // return and a sole living rider. The higher-level caller excludes dead bodies.
        let mut school = School::load(&map, false)?;
        let mut world = World::from_bsp(&map)?;
        world.set_dynamic(school.colliders().collect());
        let initial = riders(&school, &world, name, start)?;
        school.event(thread);
        let mut players = initial.clone();
        players[1].feet = map.spawn().0;
        for _ in 0..360 {
            school.advance_players(FIXED_DT, &map, &mut world, &mut players.each_mut(), &[])?;
        }
        ensure!(
            players[0].feet.distance(start) < 0.1
                && school.object(name).motion.as_ref().unwrap().time <= 1.,
            "{name} departed without partner"
        );
        players[1] = initial[1].clone();
        for _ in 0..120 {
            school.advance_players(FIXED_DT, &map, &mut world, &mut players.each_mut(), &[])?;
        }
        ensure!(
            players
                .iter()
                .zip(&initial)
                .all(|(p, old)| p.feet.z > old.feet.z + 10.),
            "{name} did not depart with both"
        );
        // Continue to the top stop, then leave only one rider aboard.
        for _ in 0..720 {
            school.advance_players(FIXED_DT, &map, &mut world, &mut players.each_mut(), &[])?;
        }
        players[1].feet = map.spawn().0;
        for _ in 0..960 {
            school.advance_players(FIXED_DT, &map, &mut world, &mut players.each_mut(), &[])?;
        }
        ensure!(
            players[0].feet.z >= top - 1.,
            "{name} returned without waiting for partner"
        );
        players[0].feet = map.spawn().0 + Vec3::X * 48.;
        for _ in 0..900 {
            school.advance_players(FIXED_DT, &map, &mut world, &mut players.each_mut(), &[])?;
        }
        ensure!(
            school
                .object(name)
                .pose
                .origin
                .distance(school.object(name).base.origin)
                < 0.1,
            "Empty {name} did not return"
        );
        players = initial;
        players[1].feet = map.spawn().0;
        let mut highest = players[0].feet.z;
        for _ in 0..3000 {
            school.advance_players(FIXED_DT, &map, &mut world, &mut [&mut players[0]], &[])?;
            highest = highest.max(players[0].feet.z);
        }
        ensure!(
            highest >= top - 1.,
            "Dead partner prevented {name} departure"
        );
        println!(
            "PASS co-op {name}: bottom/top departure gates, empty return and sole living rider"
        );
    }
    Ok(())
}

/// Reproduce the public two-client stop just below the library's upper lip.
/// One body has stepped onto the static landing while the other still rides.
fn landed_partner(map: &Bsp) -> Result<()> {
    for reversed in [false, true] {
        let mut school = School::load(map, false)?;
        school
            .event("Skool1_Lift_Up")
            .context("Missing spiral lift event")?;
        let o = school.object_mut("library_spiral_lift");
        let motion = o.motion.as_mut().unwrap();
        motion.time = 6.9499798;
        o.pose = motion.sample(motion.time);
        o.collider = Collider::model(map, o.model, o.pose.origin, o.pose.rotation(), true)?;
        let mut world = World::from_bsp(map)?;
        world.set_dynamic(school.colliders().collect());
        let mut players = [
            Player::new(vec3(252.01315, 4253.7007, 61.296814)),
            Player::new(vec3(217.55937, 4235.815, 64.03125)),
        ];
        if reversed {
            players.reverse();
        }
        let landed = if reversed { 0 } else { 1 };
        let floor_feet = players[landed].feet;
        ensure!(
            players.iter().all(|p| world.body_clear(p.feet)),
            "Spiral landing fixture starts embedded"
        );
        for _ in 0..60 {
            school.advance_players(FIXED_DT, map, &mut world, &mut players.each_mut(), &[])?;
            ensure!(
                players.iter().all(|p| world.body_clear(p.feet)),
                "Spiral landing transition embedded a body"
            );
        }
        ensure!(
            players[landed].feet == floor_feet,
            "Spiral lift dragged a player off the static landing"
        );
        ensure!(
            players[1 - landed].feet.z >= 63.99,
            "Landed partner stranded the remaining spiral rider: {:?}",
            players.each_ref().map(|p| p.feet)
        );
        println!("PASS native spiral landing reversed={reversed}: static partner stays put, remaining rider reaches top");
    }
    Ok(())
}
