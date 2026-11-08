//! Staged native-map machinery checks, separate from complete co-op traversal.
use super::*;
use anyhow::ensure;

fn object<'a>(school: &'a School2, name: &str) -> &'a Object {
    school.objects.iter().find(|o| o.name == name).unwrap()
}

fn support(school: &School2, name: &str) -> Result<Vec3> {
    let o = object(school, name);
    let top = o.pose.origin + Vec3::Z * 600.;
    let bottom = o.pose.origin - Vec3::Z * 600.;
    let hit = o.collider.trace(top, bottom, PLAYER_HALF);
    ensure!(!hit.start_solid && hit.fraction < 1. && hit.normal.z >= 0.65,
        "No co-op prop support on {name}");
    Ok(top.lerp(bottom, hit.fraction) - PLAYER_CENTER)
}

fn saved(school: &School2) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(school.snapshot())?)
}

pub fn check(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let initial = School2::load(assets, map)?;
    let mut world = World::from_bsp(map)?;
    world.set_dynamic(initial.colliders().collect());
    let rider_feet = [support(&initial, "floating_cabinet1")?,
                      support(&initial, "floating_books1")?];
    ensure!(rider_feet.iter().all(|p|world.body_clear(*p)), "Co-op prop riders start embedded");
    let mut endpoints = Vec::new();
    for fps in [30, 60, 144] {
        for reversed in [false, true] {
            let mut school = initial.clone();
            let feet = if reversed { [rider_feet[1], rider_feet[0]] } else { rider_feet };
            let mut players = feet.map(Player::new);
            let paused = saved(&school)?;
            school.advance_players(0., map, &mut world, &mut players.each_mut(), &[])?;
            ensure!(saved(&school)? == paused && players.each_ref().map(|p|p.feet) == feet,
                "Paused co-op school machinery moved");
            for _ in 0..fps {
                school.advance_players(1./fps as f32, map, &mut world, &mut players.each_mut(), &[])?;
                ensure!(players.iter().all(|p|world.body_clear(p.feet)), "School prop embedded a co-op rider");
            }
            ensure!(players.iter().zip(feet).all(|(p,start)|p.feet.distance(start)>1.),
                "School props failed to carry both players");
            let snapshot = serde_json::from_slice(&serde_json::to_vec(&school.snapshot())?)?;
            let mut mirror = initial.clone();
            mirror.restore(&snapshot, map)?;
            let mut mirror_world = World::from_bsp(map)?;
            let mut copies = players.clone();
            for _ in 0..fps {
                school.advance_players(1./fps as f32, map, &mut world, &mut players.each_mut(), &[])?;
                mirror.advance_players(1./fps as f32, map, &mut mirror_world, &mut copies.each_mut(), &[])?;
                ensure!(players.iter().all(|p|world.body_clear(p.feet)), "Restored prop route embedded a rider");
            }
            ensure!(saved(&school)? == saved(&mirror)?
                && players.each_ref().map(|p|p.feet) == copies.each_ref().map(|p|p.feet),
                "Paired school machinery diverged after snapshot restore");
            if reversed { players.reverse(); }
            endpoints.push(players.map(|p|p.feet));
            println!("PASS co-op school props {fps} Hz, reversed={reversed}: both riders, pause and snapshot continuation");
        }
    }
    for feet in &endpoints[1..] {
        ensure!(feet.iter().zip(endpoints[0]).all(|(a,b)|a.distance(b)<0.1),
            "Co-op prop motion depends on display rate or player order");
    }

    // Find a clear, stationary body beside the actual hanging lamp, in its
    // future swing. This avoids placing the blocker inside a collision mesh.
    world.set_dynamic(initial.colliders().collect());
    let lamp = object(&initial, "gym_light1");
    let center = lamp.collider.interior_point().context("Lamp has no collision interior")?;
    let mut blocker_feet = None;
    for direction in [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y] {
        let outside = center + direction * 300.;
        let hit = lamp.collider.trace(outside, center, PLAYER_HALF);
        if hit.start_solid || hit.fraction == 1. { continue; }
        let feet = outside.lerp(center, hit.fraction) + direction * 0.05 - PLAYER_CENTER;
        if !world.body_clear(feet) { continue; }
        for step in 1..=50 {
            let future = lamp.shape.collider(map, lamp.sample(step as f32 * 0.05))?;
            if future.touches(feet+PLAYER_CENTER, feet+PLAYER_CENTER, PLAYER_HALF) {
                blocker_feet = Some(feet);
                break;
            }
        }
        if blocker_feet.is_some() { break; }
    }
    let blocker_feet = blocker_feet.context("No clear co-op pendulum blocking fixture")?;
    for fps in [30, 60, 144] {
        for blocker in 0..2 {
            let mut school = initial.clone();
            let mut players = [Player::new(vec3(-64.,-3440.,-24.)),
                               Player::new(vec3(-16.,-3440.,-24.))];
            players[blocker].feet = blocker_feet;
            for _ in 0..fps*3 {
                school.advance_players(1./fps as f32, map, &mut world, &mut players.each_mut(), &[])?;
                ensure!(players.iter().all(|p|world.body_clear(p.feet)), "Pendulum crushed co-op player {blocker}");
                ensure!(object(&school,"gym_pend1").time == object(&school,"gym_light1").time,
                    "Blocked co-op pendulum separated from its lamp");
            }
            let held = object(&school,"gym_pend1").time;
            ensure!(held < 2.9, "Pendulum ignored co-op blocker {blocker}");
            let poses = ["gym_pend1","gym_light1"].map(|name|object(&school,name).pose);
            for _ in 0..fps {
                school.advance_players(1./fps as f32, map, &mut world, &mut players.each_mut(), &[])?;
            }
            for (name,pose) in ["gym_pend1","gym_light1"].into_iter().zip(poses) {
                let o = object(&school,name);
                ensure!(o.time == held && o.pose.origin == pose.origin && o.pose.rotation == pose.rotation,
                    "Blocked co-op rigid group advanced partially");
            }
            players[blocker].feet = vec3(-64.,-3440.,-24.);
            for _ in 0..fps {
                school.advance_players(1./fps as f32, map, &mut world, &mut players.each_mut(), &[])?;
            }
            ensure!((object(&school,"gym_pend1").time - held - 1.).abs() < 0.001,
                "Co-op pendulum did not resume after the blocker left");
            println!("PASS co-op pendulum {fps} Hz, blocker={blocker}: clear body, atomic lamp/rod stop and resume");
        }
    }

    // Exercise the production adapter, including its fixed door/bleacher set.
    let mut interactions = crate::interaction::Interactions::load(map)?;
    interactions.set_entry(assets, map, "skool2", None)?;
    let mut players = rider_feet.map(Player::new);
    for _ in 0..120 {
        interactions.movers_pair(1./120., map, &mut world, &mut players.each_mut())?;
    }
    ensure!(players.iter().zip(rider_feet).all(|(p,start)|p.feet.distance(start)>1. && world.body_clear(p.feet)),
        "School-two co-op adapter did not carry both riders");
    ensure!(interactions.school2.as_ref().unwrap().quest.time == initial.quest.time,
        "Machinery adapter also advanced the school quest/scene clock");
    println!("PASS co-op school-two production machinery adapter: both riders and no duplicate quest step");
    Ok(())
}
