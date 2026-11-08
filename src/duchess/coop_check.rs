//! Real-map paired boss contracts; these staged cases are not a campaign route.
use super::*;
use crate::{coop::{protocol::Visit, stage::Stage as SharedStage, world::Roster, PlayerId},
    movement::{EYE_HEIGHT, FIXED_DT}, level::CoopPlayer};

type Pair = [(Player, Stats); 2];
fn step(d: &mut Duchess, world: &World, pair: &mut Pair, dt: f32) -> Feedback {
    let mut bodies = pair.each_mut().map(|(player, stats)| CoopPlayer { player, stats });
    d.coop_combat(dt, world, &mut bodies, false)
}
fn fixture(assets: &mut Assets, map: &Bsp) -> Result<(Duchess, World, Pair)> {
    let mut d = Duchess::load(assets, map)?;
    d.stage(Stage::Fighting);
    d.state.initialized = true;
    d.state.started = true;
    d.rebuild(map)?;
    let mut world = World::from_bsp(map)?;
    world.set_dynamic(d.colliders().collect());
    let positions = [90., 260.].map(|distance| {
        (0..32).find_map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 32.;
            let at = d.state.feet + vec3(angle.cos(), angle.sin(), 0.) * distance;
            let p = Player::spawn(&world, at + Vec3::Z * (EYE_HEIGHT + 12.))?;
            ((p.feet.z - d.state.feet.z).abs() < 24.
                && world.sweep(d.state.feet + Vec3::Z*64., p.eye(), Vec3::splat(0.5)).fraction >= 1.)
                .then_some(p)
        }).context("No visible native Duchess combat position")
    });
    let [near, far] = positions;
    let mut pair = [near?, far?].map(|p| (p, Stats::for_level("potears3", None)));
    d.coop_prepare(&mut pair.each_mut().map(|(player, stats)| CoopPlayer { player, stats }));
    Ok((d, world, pair))
}

fn combat(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for victim in 0..2 {
        let (mut d, world, mut pair) = fixture(assets, map)?;
        if victim == 1 { pair.reverse(); }
        d.state.wait = 10.;
        let before = serde_json::to_value(d.snapshot())?;
        let paused = step(&mut d, &world, &mut pair, 0.);
        ensure!(paused.players.iter().all(|f| f.damage == 0.)
            && serde_json::to_value(d.snapshot())? == before, "Paused boss advanced");
        step(&mut d, &world, &mut pair, FIXED_DT);
        ensure!(d.state.coop.as_ref().unwrap().target == Some(victim), "Not nearest visible Alice");
        pair[victim].1.invisible = 10.;
        step(&mut d, &world, &mut pair, FIXED_DT);
        ensure!(d.state.coop.as_ref().unwrap().target == Some(1-victim), "Boss targeted invisible Alice");
        pair[victim].1.invisible = 0.;
        pair[victim].1.set_health(0.)?;
        step(&mut d, &world, &mut pair, FIXED_DT);
        ensure!(d.state.coop.as_ref().unwrap().target == Some(1-victim), "Boss targeted a dead Alice");
        pair[victim].1.set_health(100.)?;
        d.action(Action::Smack);
        d.state.coop.as_mut().unwrap().target = Some(victim);
        d.state.clock = d.timing.frame(Action::Smack, 9.) - FIXED_DT*0.5;
        let hit = step(&mut d, &world, &mut pair, FIXED_DT);
        ensure!(hit.players[victim].damage == 15. && hit.players[1-victim].damage == 0.
            && hit.damage == 0., "Melee hit wrong Alice");
        ensure!(step(&mut d, &world, &mut pair, FIXED_DT).players[victim].damage == 0., "Melee repeated its cue");
        println!("PASS Duchess player{victim}: nearest visible target, death retarget, personal melee and pause");

        // Enter the bite through the real Grab transition, not a captured-state override.
        d.action(Action::Grab);
        d.state.clock = d.timing.duration(Action::Grab) - FIXED_DT*0.5;
        step(&mut d, &world, &mut pair, FIXED_DT);
        ensure!(d.coop_controlled(victim) && !d.coop_controlled(1-victim), "Grab captured wrong Alice");
        let saved = d.snapshot();
        let mut restored = Duchess::load(assets, map)?;
        restored.restore(&serde_json::from_slice(&serde_json::to_vec(&saved)?)?, map)?;
        let mut other = pair.clone();
        let partner = pair[1-victim].0.feet;
        let mut damage = [0.; 2];
        for _ in 0..((d.timing.frame(Action::Bite, 66.) / FIXED_DT) as usize + 2) {
            let a = step(&mut d, &world, &mut pair, FIXED_DT);
            let b = step(&mut restored, &world, &mut other, FIXED_DT);
            for i in 0..2 { damage[i] += a.players[i].damage; }
            ensure!(a.players.each_ref().map(|f| f.damage) == b.players.each_ref().map(|f| f.damage)
                && pair.each_ref().map(|p| p.0.feet) == other.each_ref().map(|p| p.0.feet)
                && serde_json::to_value(d.snapshot())? == serde_json::to_value(restored.snapshot())?,
                "Saved bite diverged");
            ensure!(pair[1-victim].0.feet == partner, "Bite moved the other Alice");
        }
        ensure!(damage[victim] == 15. && damage[1-victim] == 0.
            && pair[victim].0.velocity.length() > 100. && pair[1-victim].0.velocity == Vec3::ZERO,
            "Bite hurt/threw wrong Alice: {damage:?}");
        ensure!(!d.coop_controlled(victim), "Throw retained movement lock");
        println!("PASS Duchess player{victim}: real grab, three isolated bites, throw, release, exact saved continuation");

        let (mut d, world, mut pair) = fixture(assets, map)?;
        if victim == 1 { pair.reverse(); }
        d.action(Action::Grab);
        d.state.clock = d.timing.duration(Action::Grab);
        step(&mut d, &world, &mut pair, FIXED_DT);
        pair[victim].1.set_health(0.)?;
        step(&mut d, &world, &mut pair, FIXED_DT);
        ensure!(!d.coop_controlled(victim) && d.state.coop.as_ref().unwrap().grab.is_none(), "Dead victim stayed captured");
        println!("PASS Duchess player{victim}: death releases captured actor");
    }
    Ok(())
}

fn personal(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for collector in 0..2 {
        for item_index in 0..4 {
        let (mut d, world, mut pair) = fixture(assets, map)?;
        d.state.wait = 100.;
        for (_, stats) in &mut pair { stats.set_health(40.)?; }
        let item = d.essence_origins[item_index];
        pair[collector].0 = Player::spawn(&world, item + Vec3::Z*EYE_HEIGHT)
            .context("Cannot reach native boss healing pickup")?;
        step(&mut d, &world, &mut pair, FIXED_DT);
        ensure!(pair[collector].1.sanity() == 65. && pair[1-collector].1.sanity() == 40.
            && d.essence_cooldown(Some(collector), item_index) == 10.
            && d.essence_cooldown(Some(1-collector), item_index) == 0.,
            "Healing fixture failed: collector={collector} health={:?} clocks={:?} feet={:?} item={item:?}",
            pair.each_ref().map(|p| p.1.sanity()), d.state.coop.as_ref().unwrap().essence,
            pair.each_ref().map(|p| p.0.feet));
        let first_position = pair[collector].0.clone();
        pair[collector].0 = pair[1-collector].0.clone();
        pair[1-collector].0 = first_position;
        step(&mut d, &world, &mut pair, FIXED_DT);
        ensure!(pair[1-collector].1.sanity() == 65. && pair[collector].1.sanity() == 65.
            && d.essence_cooldown(Some(1-collector), item_index) == 10., "Partner lost personal pickup");
        let mut saved = Duchess::load(assets, map)?;
        saved.restore(&d.snapshot(), map)?;
        ensure!(saved.essence_cooldown(Some(0), item_index) == d.essence_cooldown(Some(0), item_index)
            && saved.essence_cooldown(Some(1), item_index) == d.essence_cooldown(Some(1), item_index), "Personal healing save lost");
        pair[collector].1.set_health(25.)?;
        d.coop_respawned(collector);
        d.coop_prepare(&mut pair.each_mut().map(|(player,stats)| CoopPlayer { player, stats }));
        ensure!(pair[collector].1.sanity() == 25., "Respawn received entry restoration");
        println!("PASS Duchess player{collector} item{item_index}: independent healing visibility/cooldown, save and no respawn heal");
        }
    }
    Ok(())
}

fn scenes(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let visit = Visit { epoch: 1, map: "potears3".into(), entry: None,
        difficulty: Default::default(), combat_preview: false };
    let catalog = crate::inventory::Catalog::load(assets)?;
    let pickups = crate::inventory::pickups_for_visit(map, "potears3", None, &catalog);
    let jack = pickups.iter().find(|p| p.id == "potears3:21").context("Missing native Jackbomb")?;
    for first in PlayerId::ALL {
        for skip in [false, true] {
            let mut world = World::from_bsp(map)?;
            let mut shared = SharedStage::load(assets, map, &mut world, &visit, false)?;
            let (eye, yaw) = crate::interaction::spawn(map, None);
            let mut roster = Roster::new(&world, eye, yaw, Stats::for_level("potears3", None))?;
            for body in &mut roster.players { body.stats.set_health(30.)?; }
            shared.before_move(&mut roster, map, &mut world, &Default::default(), FIXED_DT)?;
            ensure!(roster.players.iter().all(|b| b.stats.sanity() == 100.), "Entry restoration missed Alice");
            let index = first.index();
            let partner = first.partner().index();
            roster.players[index].player = Player::spawn(&world, jack.origin + Vec3::Z*EYE_HEIGHT)
                .context("Cannot stand at native Jackbomb")?;
            let waiting = roster.players[partner].player.feet;
            shared.collect_pickups(&mut roster, &world, std::slice::from_ref(jack));
            ensure!(shared.owner == Some(first) && shared.interactions.duchess.as_ref().unwrap().scene_id() == Some(INTRO)
                && roster.players[index].stats.collected.contains(&jack.id)
                && !roster.players[partner].stats.collected.contains(&jack.id), "Wrong pickup/intro owner");
            let mut replica_world = World::from_bsp(map)?;
            let mut replica = SharedStage::load(assets, map, &mut replica_world, &visit, false)?;
            for tick in 0..120*180 {
                if skip && tick == 120 {
                    shared.interactions.duchess.as_mut().unwrap().skip(map, &mut world,
                        &mut roster.players[index].player, &mut shared.story)?;
                }
                shared.scene_step(&mut roster, map, &mut world, &Default::default())?;
                if tick % 120 == 0 {
                    let bytes = crate::coop::codec::encode(&serde_json::to_vec(&shared.snapshot())?)?;
                    replica.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?, map, &mut replica_world)?;
                    ensure!(serde_json::to_value(replica.snapshot())? == serde_json::to_value(shared.snapshot())?, "Scene replica differs");
                }
                if !shared.busy() { break; }
                ensure!(shared.owner == Some(first) && roster.players[partner].player.feet == waiting, "Scene moved partner/owner");
            }
            ensure!(!shared.busy() && shared.interactions.duchess.as_ref().unwrap().state.stage == Stage::Fighting,
                "Intro did not release control");
            ensure!(roster.players.iter().all(|b| b.stats.sanity() == 100. && world.body_clear(b.player.feet))
                && (36. ..=110.).contains(&roster.players[0].player.feet.distance(roster.players[1].player.feet)),
                "Unsafe intro reunion");
            ensure!(shared.interactions.duchess.as_ref().unwrap().state.clock == 0., "Scene advanced combat before paired handoff");
            println!("PASS Duchess {first:?} skip={skip}: entry restoration, personal Jackbomb, original intro, frozen partner, replica, reunion");

            // Staged boss defeat isolates the original reward/transition contract.
            shared.interactions.duchess.as_mut().unwrap().hit(600.);
            shared.accept(Default::default(), first, &mut roster, &world);
            let waiting = roster.players[partner].player.feet;
            for tick in 0..120*180 {
                if skip && tick == 120 {
                    shared.interactions.duchess.as_mut().unwrap().skip(map, &mut world,
                        &mut roster.players[index].player, &mut shared.story)?;
                }
                shared.scene_step(&mut roster, map, &mut world, &Default::default())?;
                if shared.transition.is_some() { break; }
                ensure!(shared.owner == Some(first) && roster.players[partner].player.feet == waiting,
                    "Outro lost owner or moved partner");
            }
            ensure!(shared.transition.as_ref().is_some_and(|t| t.0 == "utemple")
                && shared.interactions.duchess.as_ref().unwrap().state.shell_returned,
                "Original reward/temple exit missing");
            println!("PASS Duchess {first:?} skip={skip}: original death/reward scene and temple transition");
        }
    }
    Ok(())
}

pub fn check(assets: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&assets.read("maps/potears3.bsp")?)?;
    combat(assets, &map)?;
    scenes(assets, &map)?;
    personal(assets, &map)?;
    Ok(())
}

/// The production Arena includes GPU-backed weapon actors. Run this in Anode;
/// it checks native inputs and snapshots without needing screenshot inspection.
pub async fn arena_check(assets: &mut Assets) -> Result<()> {
    use crate::coop::{arena::Arena, protocol::Input};
    use crate::movement::Controls;
    let visit = Visit { epoch: 1, map: "potears3".into(), entry: None,
        difficulty: Default::default(), combat_preview: false };
    for actor in PlayerId::ALL {
        let mut scene = crate::render::Scene::load_for(assets, &visit.map, visit.difficulty)?;
        let mut arena = Arena::load(assets, &scene.map, &mut scene.world, &visit, false)?;
        let (eye, yaw) = crate::interaction::spawn(&scene.map, None);
        let mut roster = Roster::new(&scene.world, eye, yaw, Stats::for_level("potears3", None))?;
        let (d, _, mut pair) = fixture(assets, &scene.map)?;
        let index = actor.index();
        let partner = actor.partner().index();
        if index == 1 { pair.reverse(); }
        for (body, (player, stats)) in roster.players.iter_mut().zip(pair) {
            body.player = player; body.stats = stats;
        }
        arena.stage.interactions.duchess = Some(d);
        arena.stage.interactions.sync_coop(&mut scene.world);
        arena.reset_characters(&roster.players);
        let d = arena.stage.interactions.duchess.as_mut().unwrap();
        d.action(Action::Grab);
        d.state.clock = d.timing.duration(Action::Grab);
        arena.step(&mut roster, &scene.map, &mut scene.world, Default::default())?;
        ensure!(arena.stage.interactions.duchess.as_ref().unwrap().coop_controlled(index), "Arena failed to grab actor");
        let other_start = roster.players[partner].player.feet;
        for _ in 0..60 {
            let mut inputs = [(Controls::default(), Input::default()); 2];
            inputs[index].0.wish = Vec2::NEG_X;
            inputs[index].1.primary = true;
            inputs[partner].0.wish = Vec2::Y;
            arena.step(&mut roster, &scene.map, &mut scene.world, inputs)?;
        }
        let d = arena.stage.interactions.duchess.as_ref().unwrap();
        ensure!(d.coop_controlled(index) && d.state.health == 600.
            && roster.players[partner].player.feet.distance(other_start) > 12.,
            "Grab failed to lock only its victim's movement/weapon");
        ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet) >= 36., "Grab overlapped Alices");
        let saved = arena.snapshot();
        arena.replicate(&serde_json::from_slice(&serde_json::to_vec(&saved)?)?, &scene.map, &mut scene.world)?;
        ensure!(serde_json::to_value(arena.snapshot())? == serde_json::to_value(saved)?, "Arena bite replica differs");
        println!("PASS Duchess Arena {actor:?}: victim controls/weapon locked, partner moves, separate bodies and saved replica");

        // A real Blade hit supplies the finisher's identity to the original death
        // scene. Staging low boss health isolates hit routing, not a full fight.
        let (mut d, _, mut pair) = fixture(assets, &scene.map)?;
        if index == 1 { pair.reverse(); }
        d.state.health = 1.;
        d.state.wait = 100.;
        d.state.dodge_wait = 100.;
        arena.stage.interactions.duchess = Some(d);
        for (body, (player, stats)) in roster.players.iter_mut().zip(pair) {
            body.player = player; body.stats = stats;
        }
        arena.reset_characters(&roster.players);
        for _ in 0..360 {
            let d = arena.stage.interactions.duchess.as_ref().unwrap();
            let direction = (d.state.feet + Vec3::Z*64. - roster.players[index].player.eye()).normalize();
            let mut inputs = [(Controls::default(), Input::default()); 2];
            inputs[index].1 = Input { primary: true, weapon: Some(0),
                yaw: direction.y.atan2(direction.x), pitch: direction.z.asin(), ..Default::default() };
            arena.step(&mut roster, &scene.map, &mut scene.world, inputs)?;
            if arena.stage.busy() { break; }
        }
        ensure!(arena.stage.owner == Some(actor)
            && arena.stage.interactions.duchess.as_ref().unwrap().state.health == 0.
            && roster.players[partner].stats.sanity() == 100., "Native Blade finisher did not own death scene");
        println!("PASS Duchess Arena {actor:?}: ordinary Blade input hits boss and owns original death scene; partner unharmed");
        let d = arena.stage.interactions.duchess.as_mut().unwrap();
        let body = &mut roster.players[index];
        d.fixture("duchess-pepper", &scene.map, &mut scene.world, &mut body.player,
            &mut body.stats, &mut arena.stage.story)?;
        let target = d.state.feet + Vec3::Z*64.;
        let view = Camera3D { position: target + vec3(300., -600., 180.), target,
            up: Vec3::Z, z_near: 2., z_far: 30000., ..Default::default() };
        let mut art = crate::viewer::LevelArt::default();
        art.prepare(assets, &scene, &arena.stage.interactions, false)?;
        art.coop_view(index);
        let mut batches = 0;
        for frame in 0..12 {
            clear_background(BLACK);
            set_camera(&view);
            crate::render_fx::begin_view(&view, frame as f32*0.05, &scene.atmosphere, false);
            art.effects(&arena.stage.interactions, view.position, &scene.atmosphere);
            batches += crate::render_fx::finish().0;
            let d = arena.stage.interactions.duchess.as_mut().unwrap();
            d.state.clock += 0.05;
            for shot in &mut d.state.shots { shot.age += 0.05; shot.position += shot.velocity*0.05; }
            set_default_camera();
            next_frame().await;
        }
        ensure!(batches > 0, "Co-op particle pass emitted no Duchess effects");
        println!("PASS Duchess Arena {actor:?}: shared particle pass emitted {batches} batches");
        next_frame().await;
    }
    Ok(())
}
