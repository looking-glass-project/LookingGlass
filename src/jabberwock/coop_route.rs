//! Accelerated ordinary two-player Arena progression, not a transport test.
//! No battle prerequisites, player resources or progression are seeded.
use super::*;
use crate::{
    coop::{arena::Arena, protocol::{Input, Visit}, world::Roster},
    movement::Controls,
    route::Route,
};
use std::collections::{BTreeMap, VecDeque};

fn owner(arena: &Arena) -> &Encounter {
    arena.stage.interactions.levels.iter().find_map(|s| s.ctl.downcast_ref()).unwrap()
}

pub(crate) fn check(a: &mut Assets, kind: Kind) -> crate::levels::BoxFuture<'_> {
    Box::pin(async move { route(a, kind).await })
}

async fn route(a: &mut Assets, kind: Kind) -> Result<()> {
    let visit = Visit { epoch: 1, map: kind.map().into(), entry: Some(format!("{}_start1", kind.map())),
        difficulty: Default::default(), combat_preview: false };
    let map = Bsp::parse(&a.read(&format!("maps/{}.bsp", kind.map()))?)?;
    let mut world = World::from_bsp(&map)?;
    let mut arena = Arena::load(a, &map, &mut world, &visit, false)?;
    let (eye, yaw) = crate::interaction::spawn(&map, visit.entry.as_deref());
    let mut roster = Roster::new_with(&world, eye, yaw, Stats::for_level(&visit.map, visit.entry.as_deref()),
        |at| !arena.stage.interactions.hazardous(at))?;
    arena.reset_characters(&roster.players);
    let initial_staff = roster.players.each_ref().map(|b| b.stats.copies(7));
    let mut guest_world = World::from_bsp(&map)?;
    let mut guest = Arena::load(a, &map, &mut guest_world, &visit, false)?;
    let mut corners = [0, 2];
    let goals = if kind == Kind::Lair {
        [vec2(-1200., 0.), vec2(-1000., 1000.), vec2(-500., 900.), vec2(-550., 0.)]
    } else {
        [vec2(-1200., -100.), vec2(-1200., 700.), vec2(-400., 700.), vec2(-400., -100.)]
    };
    let mut crossing = [0; 2];
    let mut crossed_at = [None; 2];
    let mut landed = false;
    let mut plans: [VecDeque<(Controls, Vec3)>; 2] = Default::default();
    let mut ages = [0; 2];
    let mut phases = BTreeMap::new();
    let mut reunions = 0;
    let mut samples = 0;
    let mut waves = Vec::new();
    let mut history = VecDeque::new();
    let mut arrival = None;
    let result: Result<()> = async {
        ensure!(kind != Kind::Lair || initial_staff == [0, 0], "Survival entrance already owns its reward");
        for tick in 0..120 * 600 {
            let o = owner(&arena);
            let phase = o.saved.phase;
            let old_wave_count = o.saved.wave_count;
            phases.entry(format!("{phase:?}")).or_insert(roster.tick);
            let old_owner = arena.stage.owner;
            let before = roster.players.each_ref().map(|b| b.player.feet);
            let mut inputs = [(Controls::default(), Input::default()); 2];
            let mut planned = [false; 2];
            for i in 0..2 {
                let body = &roster.players[i];
                inputs[i].1.yaw = body.yaw;
                inputs[i].1.pitch = body.pitch;
                if arena.stage.busy() || !body.stats.alive() { continue; }
                let p = &body.player;
                if phase == Phase::Fight {
                    let healing = (body.stats.sanity() < 75.).then(|| o.personal_essence(Some(i))).flatten();
                    let goal = if kind == Kind::Lair && o.saved.ticks >= 10800 {
                        o.data.volumes["end_trigger"].interior_point().context("Missing survival exit volume")?.truncate()
                    } else if let Some(at) = healing { at.truncate() } else { goals[corners[i]] };
                    let delta = goal - p.feet.truncate();
                    if healing.is_none() && delta.length() < 70. { corners[i] = (corners[i] + 1) % goals.len(); }
                    inputs[i].0 = Controls { wish: delta.normalize_or_zero(), run: true,
                        jump: p.ledge.as_ref().is_some_and(|h| !h.pulling)
                            || healing.is_some_and(|at| p.grounded && delta.length() < 180. && at.z - p.feet.z > 48.),
                        ..Default::default() };
                    let targets = o.targets();
                    let context = crate::combat::Context { world: &world, targets: &targets };
                    if let Some(target) = targets.iter().filter(|t| kind == Kind::Grounds || t.id != kind.base())
                        .filter(|t| crate::combat::contact(&context, p.eye(), t.center, 1.).is_some_and(|(id, _)| id == t.id))
                        .min_by(|a, b| p.eye().distance_squared(a.center).total_cmp(&p.eye().distance_squared(b.center))) {
                        let aim = (target.center - p.eye()).normalize_or_zero();
                        inputs[i].1.yaw = aim.y.atan2(aim.x); inputs[i].1.pitch = aim.z.asin().clamp(-1.56, 1.56);
                        let staff = kind == Kind::Grounds && body.stats.copies(7) > 0 && body.stats.will() > 21.;
                        inputs[i].1.weapon = Some(if staff { 7 } else { 0 });
                        inputs[i].1.alternate = !staff; inputs[i].1.primary = staff;
                    }
                } else if phase == Phase::Reward {
                    let taken = o.saved.coop.as_ref().and_then(|s| s.world.as_ref()).context("Missing personal reward state")?.eye;
                    let altar = o.data.points["eye_altar"].translation;
                    if taken[i] {
                        // Clear the small reward volume for the partner.
                        if !taken[1-i] && p.feet.truncate().distance(altar.truncate()) < 80. {
                            inputs[i].0 = Controls { wish: -Vec2::X, run: true, ..Default::default() };
                        }
                    } else if i == 0 || taken[0] {
                        if plans[i].is_empty() {
                            plans[i] = Route::plan(&world, p, altar, 22.)
                                .or_else(|_| Route::plan(&world, p, altar, 40.))
                                .map_err(|n| anyhow::anyhow!("Reward path failed for slot{i}: {n} states from {:?}", p.feet))?.into();
                            ages[i] = 0;
                        }
                        if let Some((control, _)) = plans[i].front() {
                            inputs[i].0 = Controls { jump: control.jump && ages[i] == 0, ..*control };
                            planned[i] = true;
                        }
                    }
                } else if kind == Kind::Grounds && phase == Phase::Done {
                    // Walk each lane across the lowered bridge; both reach the gate
                    // before the leading player activates the onward trigger.
                    let x = [150., 750., 1300., 1700.][crossing[i]];
                    let y = 192. + if i == 0 { -48. } else { 48. };
                    let origin = vec3(x, y, 300.);
                    let bottom = origin - Vec3::Z * 600.;
                    let goal = origin.lerp(bottom, world.body_trace(origin, bottom).fraction);
                    if p.feet.distance(goal) < 40. {
                        if crossing[i] < 2 { crossing[i] += 1; plans[i].clear(); }
                        else { crossed_at[i].get_or_insert(tick); }
                    } else if crossing[i] < 3 || crossed_at.iter().all(|at| at.is_some_and(|at| tick >= at + 250)) {
                        if plans[i].is_empty() {
                            plans[i] = Route::plan(&world, p, goal, 30.)
                                .map_err(|n| anyhow::anyhow!("Bridge path failed for slot{i}: {n} states from {:?} to {goal:?}", p.feet))?.into();
                            ages[i] = 0;
                        }
                        if let Some((control, _)) = plans[i].front() {
                            inputs[i].0 = Controls { jump: control.jump && ages[i] == 0, ..*control };
                            planned[i] = true;
                        }
                    }
                    if crossing[i] == 2 && crossed_at.iter().all(|at| at.is_some_and(|at| tick >= at + 250)) {
                        crossing[i] = 3; plans[i].clear();
                    }
                }
            }
            history.push_back(serde_json::json!({"tick":roster.tick,"phase":phase,"feet":before,
                "health":roster.players.each_ref().map(|b|b.stats.sanity()),
                "movement":inputs.each_ref().map(|(c,_)|serde_json::json!({"wish":c.wish,"jump":c.jump,"run":c.run}))}));
            if history.len() > 240 { history.pop_front(); }
            arena.step(&mut roster, &map, &mut world, inputs)?;
            let o = owner(&arena);
            landed |= kind == Kind::Grounds && phase == Phase::Fight && o.saved.boss.health <= 1000. && o.saved.boss.health > 0.;
            if o.saved.wave_count != old_wave_count {
                waves.push((o.saved.ticks, o.saved.wave_count, o.saved.waves.len()));
            }
            for i in 0..2 {
                if planned[i] {
                    ages[i] += 1;
                    if ages[i] == 24 {
                        if let Some((_, expected)) = plans[i].pop_front() {
                            if roster.players[i].player.feet.distance(expected) > 8. { plans[i].clear(); }
                        }
                        ages[i] = 0;
                    }
                }
            }
            if let Some(id) = arena.stage.owner.filter(|id| Some(*id) == old_owner) {
                ensure!(roster.players[id.partner().index()].player.feet == before[id.partner().index()], "Scene moved the hidden partner");
            }
            if old_owner.is_some() && arena.stage.owner.is_none() { reunions += 1; }
            ensure!(roster.checkpoint_generation == 0, "Jabberwock route team-wiped at tick{tick}");
            ensure!(roster.players.iter().all(|b| !b.stats.god && !b.stats.notarget), "Route enabled cheats");
            if !arena.stage.busy() && roster.players.iter().all(|b| b.stats.alive()) {
                let d = roster.players[0].player.feet - roster.players[1].player.feet;
                ensure!(d.truncate().length() >= crate::coop::separation::DISTANCE - 0.05
                    || d.z.abs() >= crate::collision::PLAYER_HALF.z * 2. - 0.05, "Route players overlap");
            }
            if tick % 120 == 0 {
                roster.frame(1, 0).validate()?;
                let snapshot = arena.snapshot();
                let bytes = crate::coop::codec::encode(&serde_json::to_vec(&snapshot)?)?;
                guest.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?, &map, &mut guest_world)?;
                ensure!(serde_json::to_value(guest.snapshot())? == serde_json::to_value(snapshot)?, "Restored guest state differs");
                samples += 1;
                if tick % 1200 == 0 { println!("{kind:?} co-op t={} phase{:?} survival{} boss{} health{:?} feet{:?}", tick/120,
                    owner(&arena).saved.phase, owner(&arena).saved.ticks, owner(&arena).saved.boss.health, roster.players.each_ref().map(|b|b.stats.sanity()),
                    roster.players.each_ref().map(|b|b.player.feet)); }
                clear_background(BLACK); next_frame().await;
            }
            if arena.stage.transition.is_some() { break; }
        }
        let o = owner(&arena);
        ensure!(arena.stage.transition.as_ref().is_some_and(|e| kind.exit().matches(e)), "Missing onward exit");
        if kind == Kind::Lair {
        ensure!(o.saved.ticks == 10800 && o.saved.wave_count == 2
            && waves == [(3600, 1, 3), (7200, 2, 6)]
            && phases.get("Outro").zip(phases.get("Fight")).is_some_and(|(end, start)| end-start >= 10800)
            && o.saved.eye && o.saved.coop.as_ref().and_then(|s| s.world.as_ref()).is_some_and(|s| s.eye == [true, true])
            && reunions == 2 && roster.players.iter().all(|b| b.stats.alive() && b.stats.copies(7) == 1),
            "Incomplete survival, waves, scenes, personal rewards or exit");
        } else {
            ensure!(landed && o.saved.boss.health == 0. && o.saved.phase == Phase::Done
                && o.saved.door == 2. && crossed_at.iter().all(Option::is_some)
                && phases.contains_key("Death") && phases.contains_key("Outro") && phases.contains_key("Bridge")
                && reunions >= 2 && roster.players.iter().all(|b| b.stats.alive()),
                "Incomplete Royal Rage battle, scenes, bridge or gate");
        }
        let next_visit = Visit { epoch: 2, map: kind.exit().map.into(), entry: Some(kind.exit().entrance.into()), ..visit.clone() };
        let next_map = Bsp::parse(&a.read(&format!("maps/{}.bsp", next_visit.map))?)?;
        let mut next_world = World::from_bsp(&next_map)?;
        let next_arena = Arena::load(a, &next_map, &mut next_world, &next_visit, false)?;
        let (eye, yaw) = crate::interaction::spawn(&next_map, next_visit.entry.as_deref());
        let arrived = roster.arrive_with(&next_world, eye, yaw, &next_visit.map, next_visit.entry.as_deref(),
            |at| !next_arena.stage.interactions.hazardous(at))?;
        for (old, new) in roster.players.iter().zip(&arrived.players) {
            ensure!(serde_json::to_value(&old.stats)? == serde_json::to_value(&new.stats)? && next_world.body_clear(new.player.feet),
                "Onward arrival changed resources or obstructed a player");
        }
        arrival = Some(serde_json::json!({"visit":next_visit,"frame":arrived.frame(2,0)}));
        Ok(())
    }.await;
    let report = serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"full_route_passed":result.is_ok(),
        "error":result.as_ref().err().map(|e|format!("{e:#}")),"phases":phases,"waves":waves,"reunions":reunions,"replica_samples":samples,
        "frame":roster.frame(1,0),"world":arena.snapshot(),"arrival":arrival,"recent_inputs":history,"crossed_at":crossed_at,"landed":landed,
        "scope":"Normal entrance-to-next-level two-body production Arena simulation with compressed replica checks; no transport, screenshots or staged battle prerequisites"});
    std::fs::create_dir_all(crate::preferences::path(""))?;
    std::fs::write(crate::preferences::path(&format!("{}-coop-route.json",kind.map())), serde_json::to_vec(&report)?)?;
    result?;
    println!("PASS native co-op Jabberwock {kind:?}: {} ticks, {samples} exact replicas and actual {} arrival", roster.tick, kind.exit().map);
    Ok(())
}
