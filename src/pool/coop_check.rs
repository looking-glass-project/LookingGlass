//! Actual-map two-body machinery and hazard checks; not full-route evidence.
use super::*;
use crate::{assets::Assets, combat::{PlayerTarget, Target, COOP_BLUE, COOP_RED},
    movement::FIXED_DT};

fn riders(pool: &Pool, map: &Bsp, world: &World, name: &str) -> Result<[Player; 2]> {
    let object = pool.objects.iter().find(|o| o.name == name).context("Missing leaf")?;
    let model = &map.models[object.model];
    let center = object.pose.0 + object.pose.1 * ((model.min + model.max) * 0.5);
    let fixed = World::from_bsp(map)?;
    let mut positions = Vec::new();
    for x in (-160..=160).step_by(16) {
        for y in (-160..=160).step_by(16) {
            let above = center + vec3(x as f32, y as f32, 160.);
            let below = above - Vec3::Z * 320.;
            let hit = object.collider.trace(above + PLAYER_CENTER, below + PLAYER_CENTER, PLAYER_HALF);
            let feet = above.lerp(below, hit.fraction);
            if !hit.start_solid && hit.fraction < 1. && hit.normal.z > 0.65
                && world.body_clear(feet) && crate::moving::aboard(&object.collider, feet)
                && fixed.body_trace(feet, feet - Vec3::Z * 3.).fraction >= 1. {
                positions.push(feet);
            }
        }
    }
    let (a, b) = positions.iter().flat_map(|a| positions.iter().map(move |b| (*a, *b)))
        .filter(|(a, b)| a.distance(*b) >= 48. && a.distance(*b) <= 112.)
        .min_by(|(a,b), (c,d)| (a.distance(center)+b.distance(center))
            .total_cmp(&(c.distance(center)+d.distance(center))))
        .with_context(|| format!("No two supported positions on {name}"))?;
    Ok([a, b].map(|feet| { let mut p = Player::new(feet); p.grounded = true; p }))
}

fn leaves(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for index in 0..4 {
        for reversed in [false, true] {
            let mut pool = Pool::load(assets, map)?;
            pool.state.age = 100.;
            pool.state.talking = true;
            pool.state.talked = true;
            pool.state.drops[index] = Some(0.);
            pool.rebuild(map)?;
            let mut world = World::from_bsp(map)?;
            world.set_dynamic(pool.colliders().collect());
            let name = format!("rideleaf{}obj", index + 1);
            let thread = format!("rideleaf{}start", index + 1);
            let mut pair = riders(&pool, map, &world, &name)?;
            if reversed { pair.reverse(); }
            let feet = pair.each_ref().map(|p| p.feet);
            ensure!(pool.boarding_ready(&thread, &[feet[0], feet[1] + Vec3::X * 1000.]) == Some(false),
                "Leaf {index} left without its partner");
            ensure!(pool.boarding_ready(&thread, &feet) == Some(true)
                && pool.boarding_ready(&thread, &feet[..1]) == Some(true),
                "Leaf {index} rejected ready living riders");
            pool.event(&thread).context("Leaf start event missing")?;
            let before = serde_json::to_value(pool.snapshot())?;
            pool.advance_players(0., map, &mut world, &mut pair.each_mut(), &[])?;
            ensure!(before == serde_json::to_value(pool.snapshot())?
                && pair.each_ref().map(|p| p.feet) == feet, "Paused pair leaf moved");
            for _ in 0..120 {
                pool.advance_players(FIXED_DT, map, &mut world, &mut pair.each_mut(), &[])?;
                ensure!(pair.iter().all(|p| world.body_clear(p.feet)), "Leaf embedded a rider");
            }
            ensure!((pool.state.age - 101.).abs() < 0.01, "Pair advanced the shared clock twice");
            ensure!(pair.iter().zip(feet).all(|(p, old)| p.feet.distance(old) > 1.),
                "Leaf {index} did not carry both riders, reversed={reversed}");
            let mut restored = Pool::load(assets, map)?;
            restored.restore(&pool.snapshot(), map)?;
            let mut other = pair.clone();
            let mut other_world = World::from_bsp(map)?;
            for _ in 0..30 {
                pool.advance_players(FIXED_DT, map, &mut world, &mut pair.each_mut(), &[])?;
                restored.advance_players(FIXED_DT, map, &mut other_world, &mut other.each_mut(), &[])?;
            }
            ensure!(serde_json::to_value(pool.snapshot())? == serde_json::to_value(restored.snapshot())?
                && pair.iter().zip(&other).all(|(a,b)| a.feet.distance(b.feet) < 0.01),
                "Pair leaf restore diverged");
            println!("PASS Pool leaf{} both riders, boarding, pause and restore; reversed={reversed}", index+1);
        }
    }
    Ok(())
}

fn rocks(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for victim in 0..2 {
        let mut pool = Pool::load(assets, map)?;
        let position = pool.state.boulders.as_ref().unwrap().rocks[2].position;
        let targets = [COOP_BLUE, COOP_RED].map(|id| Target {
            id, center: if id == [COOP_BLUE,COOP_RED][victim] { position }
                else { position + Vec3::X * 2000. }, half: PLAYER_HALF,
        });
        let before = serde_json::to_value(pool.snapshot())?;
        let paused = pool.advance_boulders_for(0., &targets);
        ensure!(paused.players.iter().all(|p| p.damage == 0.)
            && before == serde_json::to_value(pool.snapshot())?, "Paused rock changed");
        pool.event("Tears1_Boulder3").unwrap();
        pool.state.age += 0.01;
        let out = pool.advance_boulders_for(0.01, &targets);
        ensure!(out.players[victim].damage == 50. && out.players[1-victim].damage == 0.
            && out.damage == 0. && pool.state.boulders.as_ref().unwrap().damage == 0.,
            "Moving rock did not isolate damage to player{victim}");
        ensure!(out.players[victim].impulse.length() > 0.
            && out.players[1-victim].impulse == Vec3::ZERO, "Rock impulse leaked to partner");
        println!("PASS Pool moving rock: player{victim} damage50/impulse, partner unchanged, pause");
    }
    Ok(())
}

// Use the authored trigger and shared contact history, rather than invoking the
// leaf event directly. Waiting must not spend a one-shot trigger's activation.
fn contacts(assets: &mut Assets, map: &Bsp) -> Result<()> {
    use crate::{coop::PlayerId, interaction::{Interactions, coop::{Contacts, Motion}}};
    for index in 0..4 {
        for first in PlayerId::ALL {
            for dead in [None, Some(first), Some(first.partner())] {
                let mut interactions = Interactions::load(map)?;
                interactions.set_entry(assets, map, "potears1", None)?;
                let pool = interactions.pool.as_mut().unwrap();
                pool.state.age = 100.;
                pool.state.talking = true;
                pool.state.talked = true;
                pool.state.drops[index] = Some(0.);
                pool.rebuild(map)?;
                let mut world = World::from_bsp(map)?;
                interactions.sync_coop(&mut world);
                let name = format!("rideleaf{}obj", index + 1);
                let thread = format!("rideleaf{}start", index + 1);
                let pair = riders(interactions.pool.as_ref().unwrap(), map, &world, &name)?;
                let entity = map.entities.iter().position(|e| e.get("thread") == Some(&thread))
                    .context("Missing authored leaf trigger")?;
                let key = format!("trigger/{entity}");
                let mut feet = [crate::interaction::spawn(map, None).0; 2];
                feet[first.index()] = pair[0].feet;
                let motion = |feet: [Vec3; 2]| feet.map(|at| Some(Motion { before: at, after: at }));
                let mut contacts = Contacts::new(&interactions);
                for _ in 0..120 {
                    let out = interactions.triggers_pair(FIXED_DT, &mut contacts, motion(feet))?;
                    ensure!(out.waiting[first.index()]
                        && !out.activations.iter().any(|a| a.entity.0 == entity),
                        "{thread} did not wait for its living partner");
                }
                ensure!(interactions.pool.as_ref().unwrap().state.rides[index].is_none()
                    && interactions.event_world.snapshot().usage[&key].count == 0,
                    "Waiting consumed {thread}");
                // Round-trip the compressed network representation of both the
                // world and its pending first-arrival request before boarding.
                let wire = crate::coop::protocol::HostMessage::Frame {
                    frame: (interactions.snapshot(), contacts),
                };
                let bytes = crate::coop::codec::encode(&serde_json::to_vec(&wire)?)?;
                let crate::coop::protocol::HostMessage::Frame { frame: (saved, mut contacts) } =
                    serde_json::from_slice::<crate::coop::protocol::HostMessage<
                        (crate::interaction::Snapshot, Contacts)>>(&crate::coop::codec::decode(&bytes)?)?
                    else { anyhow::bail!("Leaf envelope changed"); };
                interactions.restore(&saved, map)?;
                contacts.validate(&interactions)?;
                feet[first.partner().index()] = pair[1].feet;
                let mut paths = motion(feet);
                if let Some(id) = dead { paths[id.index()] = None; }
                let out = interactions.triggers_pair(FIXED_DT, &mut contacts, paths)?;
                let events = out.activations.iter().filter(|a| a.entity.0 == entity).collect::<Vec<_>>();
                let owner = if dead == Some(first) { first.partner() } else { first };
                ensure!(events.len() == 1 && events[0].player == owner
                    && interactions.pool.as_ref().unwrap().state.rides[index].is_some()
                    && interactions.event_world.snapshot().usage[&key].count == 1,
                    "{thread} did not activate once for the first living requester");
                let mut bodies = [pair[0].clone(), pair[1].clone()];
                if first == PlayerId::Red { bodies.reverse(); }
                let before = bodies.each_ref().map(|p| p.feet);
                for _ in 0..180 {
                    let mut living = bodies.iter_mut().enumerate()
                        .filter(|(i, _)| dead.is_none_or(|id| id.index() != *i))
                        .map(|(_, p)| p).collect::<Vec<_>>();
                    interactions.movers_pair(FIXED_DT, map, &mut world, &mut living)?;
                    for p in living { p.tick(&world, Default::default()); }
                }
                for (i, p) in bodies.iter().enumerate() {
                    if dead.is_none_or(|id| id.index() != i) {
                        ensure!(p.feet.distance(before[i]) > 50. && p.grounded && world.body_clear(p.feet),
                            "{thread} integrated carry lost living rider {i}");
                    } else {
                        ensure!(p.feet == before[i], "Leaf carried a dead body");
                    }
                }
                println!("PASS Pool leaf{} authored contact, wait, wire restore, first={first:?}, dead={dead:?}, integrated carry", index + 1);
            }
        }
    }
    Ok(())
}

fn swimmer(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for blocked in 0..2 {
        let mut pool = Pool::load(assets, map)?;
        pool.state.age = 289.23477;
        pool.state.talking = true;
        pool.state.talked = true;
        pool.state.drops[1] = Some(250.);
        pool.state.rides[1] = Some(274.22177);
        pool.state.trains[1] = Some((1, 281.77826));
        pool.rebuild(map)?;
        let mut world = World::from_bsp(map)?;
        world.set_dynamic(pool.colliders().collect());
        let start = vec3(1521.5931, -2250.5078, 1316.0774);
        let mut pair = [Player::new(start), Player::spawn(&world,
            crate::interaction::spawn(map, None).0).context("Pool partner spawn")?];
        if blocked == 1 { pair.reverse(); }
        let feet = pair.each_ref().map(|p| p.feet);
        ensure!(pair.iter().all(|p| world.body_clear(p.feet)), "Swimmer fixture starts obstructed");
        let age = pool.state.age;
        for _ in 0..240 {
            pool.advance_players(FIXED_DT, map, &mut world, &mut pair.each_mut(), &[])?;
            ensure!(pair.iter().all(|p| world.body_clear(p.feet)), "Leaf embedded swimmer or partner");
        }
        ensure!(pool.state.age > age + 1.99
            && pool.state.blocked_time.values().any(|t| *t > 0.)
            && pair.each_ref().map(|p| p.feet) == feet,
            "Swimmer must hold only the obstructed mover, never the visit clock");
        let saved = pool.snapshot();
        let mut restored = Pool::load(assets, map)?;
        restored.restore(&saved, map)?;
        let mut other = pair.clone();
        let mut other_world = World::from_bsp(map)?;
        for _ in 0..120 {
            pool.advance_players(FIXED_DT, map, &mut world, &mut pair.each_mut(), &[])?;
            restored.advance_players(FIXED_DT, map, &mut other_world, &mut other.each_mut(), &[])?;
        }
        ensure!(serde_json::to_value(pool.snapshot())? == serde_json::to_value(restored.snapshot())?
            && pair.each_ref().map(|p| p.feet) == other.each_ref().map(|p| p.feet),
            "Swimmer obstruction diverged after restore");
        println!("PASS Pool swimmer player{blocked}: no embedding, independent visit clock and restore");
    }
    Ok(())
}

fn ants(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let world = World::fixture(&[(vec3(-2000., -2000., -100.), vec3(2000., 2000., 0.))]);
    for near in 0..2 {
        let mut pool = Pool::load(assets, map)?;
        let state = pool.state.boulders.as_mut().unwrap();
        state.ants[0] = ants::Ant::pusher(vec3(0.,0.,0.03125),0.,634);
        state.ants[0].enabled = true;
        let players = std::array::from_fn(|i| PlayerTarget {
            eye: vec3(if i == near { 80. } else { 700. }, 0., crate::movement::EYE_HEIGHT + 0.03125),
            alive: true, hidden: false,
        });
        let mut damage = [0.; 2];
        for _ in 0..600 {
            let out = pool.combat_players(FIXED_DT, &world, players);
            for (total, feedback) in damage.iter_mut().zip(out.players) { *total += feedback.damage; }
            if damage[near] > 0. { break; }
        }
        let ant = &pool.state.boulders.as_ref().unwrap().ants[0];
        ensure!(ant.opponents.selected_player == Some(near) && damage[near] > 0.
            && damage[1-near] == 0., "Pool ant did not target nearer player{near}: {damage:?}");
        let mut hidden = players;
        hidden[near].hidden = true;
        pool.combat_players(FIXED_DT, &world, hidden);
        ensure!(pool.state.boulders.as_ref().unwrap().ants[0].opponents.selected_player == Some(1-near),
            "Pool ant kept targeting the invisible player");
        println!("PASS Pool ant targets/damages nearer player{near}, changes target for invisibility");
    }
    Ok(())
}

pub fn check(assets: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&assets.read("maps/potears1.bsp")?)?;
    leaves(assets, &map)?;
    contacts(assets, &map)?;
    swimmer(assets, &map)?;
    rocks(assets, &map)?;
    ants(assets, &map)?;
    Ok(())
}
