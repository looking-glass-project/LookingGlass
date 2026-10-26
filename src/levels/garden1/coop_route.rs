//! Ordinary-input paired course through the production combat Arena.
//! The test only chooses controls; scenes, resources and placements belong to gameplay.
use super::*;
use crate::{
    coop::{
        arena::Arena,
        protocol::{Input, Visit},
        world::Roster,
        PlayerId,
    },
    movement::{Controls, FIXED_DT},
    render::Scene,
    route::{fight, Route},
};
use std::collections::VecDeque;

#[derive(Deserialize)]
struct Step {
    pos: [f32; 3],
    #[serde(default)]
    red_pos: Option<[f32; 3]>,
    #[serde(default)]
    mode: String,
    #[serde(default)]
    frames: usize,
}
impl Step {
    fn goal(&self, id: PlayerId) -> Vec3 {
        Vec3::from_array(if id == PlayerId::Red {
            self.red_pos.unwrap_or(self.pos)
        } else {
            self.pos
        })
    }
}
#[derive(Default)]
struct Driver {
    index: usize,
    ticks: usize,
    quiet: usize,
    cards: bool,
    last: Vec3,
    plan: VecDeque<(Controls, Vec3)>,
    plan_tick: usize,
    fight: fight::Tracker,
}
impl Driver {
    fn next(&mut self, feet: Vec3) {
        self.index += 1;
        self.ticks = 0;
        self.quiet = 0;
        self.last = feet;
        self.plan.clear();
        self.plan_tick = 0;
    }
    fn sample(
        &mut self,
        steps: &[Step],
        arena: &Arena,
        world: &World,
        roster: &Roster,
        id: PlayerId,
        wait_before: Option<usize>,
    ) -> Result<(Controls, Input)> {
        let body = &roster.players[id.index()];
        let p = &body.player;
        let mut control = Controls::default();
        let mut input = Input {
            yaw: body.yaw,
            pitch: body.pitch,
            ..Default::default()
        };
        let movement_locked = arena.characters[id.index()].attack_movement_locked();
        let targets = arena.npcs.targets();
        self.fight.observe(&targets, FIXED_DT);
        let context = crate::combat::Context {
            world,
            targets: &targets,
        };
        let attack = fight::plan(
            &context,
            p.eye(),
            &targets,
            &self.fight,
            (self.cards || wait_before == Some(self.index))
                && body.stats.copies(1) > 0
                && body.stats.will() >= fight::CARD_COST,
        );
        if attack.target.is_some() && p.rope.is_none() {
            if !movement_locked {
                input.weapon = Some(attack.weapon as u8);
                input.primary = !attack.alternate;
                input.alternate = attack.alternate;
            }
            input.yaw = attack.aim.y.atan2(attack.aim.x);
            input.pitch = attack.aim.z.asin().clamp(-1.56, 1.56);
            self.fight.weapon = attack.weapon;
            self.fight.last_aim = attack.aim;
        }
        // Keep tracking the target until the projectile actually leaves Alice's
        // hand. Resetting pitch during the wind-up makes elevated shots miss.
        // Movement stays neutral and timed traversal inputs are not consumed.
        if movement_locked {
            return Ok((control, input));
        }
        loop {
            let Some(step) = steps.get(self.index) else {
                return Ok((control, input));
            };
            let goal = step.goal(id);
            let delta = goal - p.feet;
            // A timed swing without an explicit climb step assumes the grip
            // height of its approach. An obstacle jump can catch the same rope
            // much higher; descend through ordinary controls before swinging.
            let settle_grip =
                step.mode == "grab" && steps.get(self.index + 1).is_some_and(|s| s.mode == "input");
            // Wait on the bank with combat available, not hanging from a rope
            // under fire. This changes only the test player's chosen inputs.
            if wait_before == Some(self.index) {
                return Ok((control, input));
            }
            ensure!(
                self.ticks < 120 * 90,
                "Garden {id:?} stalled at step{} {} goal{goal:?}, feet{:?}",
                self.index,
                step.mode,
                p.feet
            );
            let done = match step.mode.as_str() {
                "cards" => {
                    self.cards = goal.x != 0.;
                    true
                }
                "wait" | "hold" | "input" | "inputjump" | "release" => self.ticks >= step.frames,
                "clear" => self.quiet >= 120,
                "rope" => p.rope.is_some() && delta.z.abs() < 8.,
                "grab" => p.rope.is_some() && (!settle_grip || delta.z.abs() < 8.),
                "launch" => p.velocity.z > 500.,
                _ => delta.truncate().length() < 20. && delta.z.abs() < 52.,
            };
            if done {
                self.next(p.feet);
                continue;
            }
            match step.mode.as_str() {
                "clear" => {
                    let visible = targets.iter().any(|t| {
                        t.center.distance(p.eye()) < goal.x
                            && fight::aim_point(&context, p.eye(), t).is_some()
                    });
                    self.quiet = if visible { 0 } else { self.quiet + 1 };
                    if self.ticks > 0 && self.ticks % 600 == 0 {
                        let mut blade =
                            crate::weapons::Projectile::thrown_blade(p.eye(), attack.aim);
                        let (hit, contact_normal) = blade.contact_step(1., &context);
                        let point = attack
                            .target
                            .and_then(|id| targets.iter().find(|t| t.id == id))
                            .and_then(|t| fight::aim_point(&context, p.eye(), t));
                        let card_hit = point.and_then(|point| {
                            let mut card = crate::weapons::Projectile::card(
                                p.eye(),
                                (point - p.eye()).normalize_or_zero(),
                                None,
                            );
                            let first = card.contact_step(0.75, &context).0;
                            first.or_else(|| card.contact_step(0.75, &context).0)
                        });
                        let saved = serde_json::to_value(arena.characters[id.index()].snapshot())?;
                        let mut in_flight = Vec::new();
                        for value in saved["projectiles"]["projectiles"].as_array().unwrap() {
                            let mut projectile: crate::weapons::Projectile =
                                serde_json::from_value(value.clone())?;
                            let age = projectile.age;
                            let (hit, normal) = projectile.contact_step(1., &context);
                            in_flight.push((age, hit.map(|h| h.id), normal));
                        }
                        println!(
                            "Garden clear {id:?} step{} ticks{} target={:?} eye={:?} aim={:?} blade_hit={:?} contact_normal={contact_normal:?} card_hit={:?} in_flight={in_flight:?}",
                            self.index, self.ticks, attack.target, p.eye(), attack.aim, hit.map(|h| h.id), card_hit.map(|h| h.id)
                        );
                    }
                    // A clear can opt into vertical jump shots after its first
                    // stationary attempt, to see over low cover. This uses the
                    // ordinary jump input without moving the firing position.
                    control.jump = step.frames > 0
                        && self.ticks >= step.frames
                        && self.ticks % 240 == 0
                        && visible
                        && p.grounded;
                }
                "wait" => (),
                "grab" if p.rope.is_some() => control.rise = delta.z.signum(),
                "hold" => {
                    control.wish = (delta.truncate() * 4. / 320.).clamp_length_max(1.);
                    control.run = true;
                }
                "input" | "inputjump" | "release" => {
                    control.wish = goal.truncate().clamp_length_max(1.);
                    control.swim = goal.clamp_length_max(1.);
                    control.rise = goal.z.clamp(-1., 1.);
                    control.run = true;
                    control.jump = step.mode == "inputjump" && self.ticks == 0;
                    control.use_pressed = step.mode == "release" && self.ticks == 0;
                }
                "rope" => {
                    if p.rope.is_some() {
                        control.rise = delta.z.signum();
                    } else {
                        let grab = steps[..self.index]
                            .iter()
                            .rev()
                            .find(|s| s.mode == "grab")
                            .context("Garden rope climb has no approach")?;
                        let approach = grab.goal(id) - p.feet;
                        control = Controls {
                            wish: approach.truncate().normalize_or_zero(),
                            swim: approach.normalize_or_zero(),
                            rise: approach.z.signum(),
                            run: true,
                            jump: p.grounded && approach.z > 25. && self.ticks % 60 == 0,
                            use_pressed: p.rope_prompt(world) == Some("E grab rope")
                                && self.ticks % 4 == 0,
                            ..Default::default()
                        };
                    }
                }
                "nav" => {
                    if self.plan.is_empty() {
                        self.plan = Route::plan(world, p, goal, 20.)
                            .map_err(|count| {
                                anyhow::anyhow!("Garden route search exhausted {count} nodes")
                            })?
                            .into();
                        self.plan_tick = 0;
                    }
                    if let Some((next, _)) = self.plan.front() {
                        control = *next;
                        control.jump &= self.plan_tick == 0;
                        self.plan_tick += 1;
                        if self.plan_tick == 24 {
                            self.plan.pop_front();
                            self.plan_tick = 0;
                        }
                    }
                }
                _ => {
                    let stuck = self.ticks % 120 == 119 && p.feet.distance(self.last) < 18.;
                    if self.ticks % 120 == 119 {
                        self.last = p.feet;
                    }
                    control = Controls {
                        wish: delta.truncate().normalize_or_zero(),
                        swim: delta.normalize_or_zero(),
                        rise: if delta.z.abs() > 25. {
                            delta.z.signum()
                        } else {
                            0.
                        },
                        run: true,
                        jump: (step.mode == "jump" && self.ticks == 0)
                            || ((step.mode.is_empty()
                                || (matches!(step.mode.as_str(), "grab" | "launch")
                                    && p.grounded))
                                && stuck),
                        use_pressed: step.mode == "grab" && self.ticks % 4 == 0 && p.rope.is_none(),
                        ..Default::default()
                    };
                }
            }
            self.ticks += 1;
            if matches!(
                step.mode.as_str(),
                "jump" | "launch" | "grab" | "rope" | "release" | "input" | "inputjump"
            ) {
                input.weapon = None;
                input.primary = false;
                input.alternate = false;
            }
            if !input.primary && !input.alternate && control.wish.length_squared() > 0.01 {
                input.yaw = control.wish.y.atan2(control.wish.x);
            }
            return Ok((control, input));
        }
    }
}
fn garden(arena: &Arena) -> &Garden {
    arena
        .stage
        .interactions
        .levels
        .iter()
        .find_map(|s| s.ctl.downcast_ref::<Garden>())
        .unwrap()
}
fn report(
    arena: &Arena,
    roster: &Roster,
    drivers: &[Driver; 2],
    first: PlayerId,
    skip: bool,
    samples: usize,
) -> serde_json::Value {
    serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"first":first,"skip":skip,"samples":samples,
        "steps":drivers.each_ref().map(|d|d.index),"step_ticks":drivers.each_ref().map(|d|d.ticks),
        "cards":drivers.each_ref().map(|d|d.cards),"frame":roster.frame(1,0),"world":arena.snapshot()})
}
fn record(
    arena: &Arena,
    roster: &Roster,
    drivers: &[Driver; 2],
    first: PlayerId,
    skip: bool,
    samples: usize,
) -> Result<()> {
    std::fs::write(
        crate::preferences::path(&format!("garden1-{first:?}-{skip}-route.json")),
        serde_json::to_vec(&report(arena, roster, drivers, first, skip, samples))?,
    )?;
    Ok(())
}
async fn route(a: &mut Assets, first: PlayerId, skip: bool) -> Result<()> {
    let visit = Visit {
        epoch: 1,
        map: "garden1".into(),
        entry: Some("garden1_start1".into()),
        difficulty: Default::default(),
        combat_preview: false,
    };
    let mut scene = Scene::load_for(a, &visit.map, visit.difficulty)?;
    let mut arena = Arena::load(a, &scene.map, &mut scene.world, &visit, false)?;
    let (eye, yaw) = crate::interaction::spawn(&scene.map, visit.entry.as_deref());
    let mut roster = Roster::new_with(
        &scene.world,
        eye,
        yaw,
        Stats::for_level("garden1", visit.entry.as_deref()),
        |at| !arena.stage.interactions.hazardous(at),
    )?;
    arena.reset_characters(&roster.players);
    let mut replica_world = World::from_bsp(&scene.map)?;
    let mut replica = Arena::load(a, &scene.map, &mut replica_world, &visit, false)?;
    let mut art = crate::viewer::LevelArt::default();
    art.prepare(a, &scene, &arena.stage.interactions, false)?;
    let course = if let Ok(path) = std::env::var("LOOKING_GLASS_GARDEN_COOP_ROUTE") {
        std::fs::read(path).context("Cannot read Garden co-op test course")?
    } else {
        include_bytes!("coop_steps.json").to_vec()
    };
    let steps: Vec<Step> = serde_json::from_slice(&course)?;
    ensure!(
        !steps.is_empty()
            && steps.iter().all(|s| s.pos.iter().all(|v| v.is_finite())
                && s.red_pos
                    .is_none_or(|pos| pos.iter().all(|v| v.is_finite()))
                && s.frames <= 120 * 90),
        "Invalid Garden co-op test course"
    );
    // Keep the exact controls course beside the state report, including any
    // private diagnostic override, so a result can be reproduced.
    std::fs::write(
        crate::preferences::path("garden1-coop-course.json"),
        &course,
    )?;
    let mut drivers: [Driver; 2] = Default::default();
    let mut free_ticks = 0;
    let mut samples = 0;
    let mut rabbit_seen = false;
    let mut reunions = 0;
    let mut close_recorded = false;
    let mut flight_ticks = [0; 2];
    for tick in 0..120 * 1200 {
        let indices_before = drivers.each_ref().map(|d| d.index);
        let owner_before = arena.stage.owner;
        let feet_before = roster.players.each_ref().map(|b| b.player.feet);
        let warps_before = roster.players.each_ref().map(|b| b.warped_at);
        let ropes_before = roster.players.each_ref().map(|b| b.player.rope.clone());
        let landings_before = roster.players.each_ref().map(|b| b.player.landings);
        let health_before = roster.players.each_ref().map(|b| b.stats.sanity());
        let inputs = (|| -> Result<_> {
            let mut out = std::array::from_fn(|_| (Controls::default(), Input::default()));
            if !arena.stage.busy() {
                for id in PlayerId::ALL {
                    let wait_before = (id != first && !rabbit_seen).then_some(
                        if drivers[first.index()].index < 11 {
                            5
                        } else {
                            20
                        },
                    );
                    if id == first || free_ticks >= 240 {
                        out[id.index()] = drivers[id.index()].sample(
                            &steps,
                            &arena,
                            &scene.world,
                            &roster,
                            id,
                            wait_before,
                        )?;
                    }
                }
                free_ticks += 1;
            } else if skip {
                let id = arena.stage.owner.context("Garden scene has no owner")?;
                arena.stage.interactions.skip_cinematic(
                    &scene.map,
                    &mut scene.world,
                    &mut roster.players[id.index()].player,
                    &mut arena.stage.story,
                )?;
            }
            Ok(out)
        })();
        let inputs = match inputs {
            Ok(v) => v,
            Err(e) => {
                record(&arena, &roster, &drivers, first, skip, samples)?;
                return Err(e);
            }
        };
        arena.step(&mut roster, &scene.map, &mut scene.world, inputs)?;
        for id in PlayerId::ALL {
            for step in indices_before[id.index()]..drivers[id.index()].index {
                if let Some(s) = steps
                    .get(step)
                    .filter(|s| matches!(s.mode.as_str(), "launch" | "release"))
                {
                    flight_ticks[id.index()] = 360;
                    // The captured driver counters and bodies both include the
                    // first input tick after the launch/release step completed.
                    std::fs::write(
                        crate::preferences::path(&format!(
                            "garden1-{first:?}-{skip}-{id:?}-{}-{step}.json",
                            s.mode
                        )),
                        serde_json::to_vec(&report(
                            &arena, &roster, &drivers, first, skip, samples,
                        ))?,
                    )?;
                }
            }
            let body = &roster.players[id.index()];
            if flight_ticks[id.index()] > 0 {
                flight_ticks[id.index()] -= 1;
                if flight_ticks[id.index()] % 10 == 0 {
                    let p = &body.player;
                    let control = inputs[id.index()].0;
                    println!(
                        "Garden flight {}",
                        serde_json::json!({
                            "id": id, "tick": roster.tick, "step": drivers[id.index()].index,
                            "feet": p.feet, "velocity": p.velocity, "grounded": p.grounded,
                            "ledge": p.ledge, "health": body.stats.sanity(), "wish": control.wish,
                            "rise": control.rise, "jump": control.jump,
                            "attack_locked": arena.characters[id.index()].attack_movement_locked()
                        })
                    );
                }
            }
            if body.player.landings != landings_before[id.index()]
                && body.player.landing_speed > 550.
            {
                println!(
                    "Garden {id:?} hard landing tick{tick} step{} speed={} health={}->{} feet={:?}",
                    drivers[id.index()].index,
                    body.player.landing_speed,
                    health_before[id.index()],
                    body.stats.sanity(),
                    body.player.feet
                );
            }
            if ropes_before[id.index()].is_some() && body.player.rope.is_none() {
                println!(
                    "Garden {id:?} rope released tick{tick} step{} use={} health={} feet={:?} velocity={:?}",
                    drivers[id.index()].index, inputs[id.index()].0.use_pressed,
                    body.stats.sanity(), body.player.feet, body.player.velocity
                );
            }
        }
        let spacing = roster.players[0].player.feet - roster.players[1].player.feet;
        if !close_recorded
            && !arena.stage.busy()
            && spacing.truncate().length() < 20.
            && spacing.z.abs() < 40.
        {
            let report = serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"before":feet_before,
                "steps":drivers.each_ref().map(|d|d.index),"frame":roster.frame(1,0),"world":arena.snapshot()});
            std::fs::write(
                crate::preferences::path(&format!("garden1-{first:?}-{skip}-close-contact.json")),
                serde_json::to_vec(&report)?,
            )?;
            close_recorded = true;
            println!(
                "Garden close-contact captured tick{tick}, steps{:?}",
                drivers.each_ref().map(|d| d.index)
            );
        }
        if garden(&arena).scene_id() == Some(RABBIT) {
            rabbit_seen = true;
            ensure!(
                arena.stage.owner == Some(first),
                "Garden Rabbit scene had wrong first arrival"
            );
        }
        if let Some(id) = arena.stage.owner.filter(|id| Some(*id) == owner_before) {
            ensure!(
                roster.players[id.partner().index()].player.feet
                    == feet_before[id.partner().index()],
                "Garden scene moved waiting player"
            );
        }
        if owner_before.is_some() && arena.stage.owner.is_none() {
            reunions += 1;
            let next = drivers.iter().map(|d| d.index).max().unwrap();
            for (d, b) in drivers.iter_mut().zip(&roster.players) {
                d.index = next;
                d.ticks = 0;
                d.plan.clear();
                d.last = b.player.feet;
            }
            ensure!(
                roster.players[0]
                    .player
                    .feet
                    .distance(roster.players[1].player.feet)
                    >= 36.,
                "Garden reunion overlapped players"
            );
        }
        let valid = roster.checkpoint_generation == 0
            && roster
                .players
                .iter()
                .all(|b| b.stats.alive() && b.spawned_at == 0)
            && (owner_before.is_some()
                || roster.players.each_ref().map(|b| b.warped_at) == warps_before);
        if !valid {
            record(&arena, &roster, &drivers, first, skip, samples)?;
            anyhow::bail!(
                "Garden route death/recovery/warp tick{tick}, steps{:?}, feet{:?}, health{:?}",
                drivers.each_ref().map(|d| d.index),
                roster.players.each_ref().map(|b| b.player.feet),
                roster.players.each_ref().map(|b| b.stats.sanity())
            );
        }
        if tick % 600 == 0 {
            let packed = crate::coop::codec::encode(&serde_json::to_vec(&arena.snapshot())?)?;
            replica.replicate(
                &serde_json::from_slice(&crate::coop::codec::decode(&packed)?)?,
                &scene.map,
                &mut replica_world,
            )?;
            ensure!(
                serde_json::to_value(replica.snapshot())?
                    == serde_json::to_value(arena.snapshot())?,
                "Garden route replica differs"
            );
            samples += 1;
            record(&arena, &roster, &drivers, first, skip, samples)?;
            println!(
                "Garden {first:?} skip={skip} t={} steps={:?} health={:?} feet={:?}",
                tick / 120,
                drivers.each_ref().map(|d| d.index),
                roster.players.each_ref().map(|b| b.stats.sanity()),
                roster.players.each_ref().map(|b| b.player.feet)
            );
        }
        if tick % 120 == 0 {
            clear_background(BLACK);
            let at = roster.players[first.index()].player.feet;
            let camera = arena.stage.camera(&scene.world).unwrap_or_else(|| {
                crate::cinematic::Camera::look(
                    at + vec3(160., 180., 100.),
                    at + crate::collision::PLAYER_CENTER,
                )
            });
            let view = Camera3D {
                position: camera.eye,
                target: camera.target,
                up: camera.up,
                fovy: 60_f32.to_radians(),
                z_near: 2.,
                z_far: 20000.,
                ..Default::default()
            };
            set_camera(&view);
            crate::lighting::select(vec![], camera.eye, &scene.world);
            crate::render_fx::begin_view(&view, tick as f32 * FIXED_DT, &scene.atmosphere, false);
            let transforms = arena.stage.draw_transforms(&scene.world);
            scene.draw(
                camera.eye,
                tick as f32 * FIXED_DT,
                false,
                false,
                &transforms,
            );
            art.draw(
                &arena.stage.interactions,
                &arena.stage.hints,
                &scene.atmosphere,
                camera.eye,
                false,
            );
            if !arena.stage.busy() {
                for (c, b) in arena.characters.iter_mut().zip(&roster.players) {
                    c.atmosphere(&scene.atmosphere, camera.eye);
                    c.draw_replica(b.player.feet, b.player.feet, false);
                }
            }
            arena.npcs.draw(
                camera.eye,
                (camera.target - camera.eye).normalize_or_zero(),
                &scene.atmosphere,
                false,
            );
            scene.draw(camera.eye, tick as f32 * FIXED_DT, false, true, &transforms);
            art.effects(&arena.stage.interactions, camera.eye, &scene.atmosphere);
            set_default_camera();
            next_frame().await;
        }
        if arena.stage.transition.is_some() {
            break;
        }
    }
    record(&arena, &roster, &drivers, first, skip, samples)?;
    ensure!(
        rabbit_seen
            && reunions == 2
            && garden(&arena).saved.world.rabbit.is_some()
            && arena
                .stage
                .transition
                .as_ref()
                .is_some_and(|e| e.0 == "garden2" && e.1.as_deref() == Some("garden2_start1")),
        "Garden paired course did not reach its authored exit"
    );
    let (map, entry) = arena.stage.transition.as_ref().unwrap();
    let next_visit = Visit {
        epoch: 2,
        map: map.clone(),
        entry: entry.clone(),
        ..visit
    };
    let mut next_scene = Scene::load_for(a, map, next_visit.difficulty)?;
    let next_arena = Arena::load(
        a,
        &next_scene.map,
        &mut next_scene.world,
        &next_visit,
        false,
    )?;
    let (eye, yaw) = crate::interaction::spawn(&next_scene.map, entry.as_deref());
    let arrived = roster.arrive_with(&next_scene.world, eye, yaw, map, entry.as_deref(), |at| {
        !next_arena.stage.interactions.hazardous(at)
    })?;
    for (old, new) in roster.players.iter().zip(&arrived.players) {
        ensure!(
            serde_json::to_value(&old.stats)? == serde_json::to_value(&new.stats)?
                && new.stats.alive()
                && next_scene.world.body_clear(new.player.feet),
            "Garden2 arrival changed personal resources or obstructed a player"
        );
    }
    ensure!(
        arrived.players[0]
            .player
            .feet
            .distance(arrived.players[1].player.feet)
            >= 36.,
        "Garden2 arrival overlapped players"
    );
    println!(
        "PASS full Garden co-op into Garden2 {first:?} skip={skip}: {} ticks, {samples} replicas, exact resources, health {:?}",
        roster.tick,
        roster.players.each_ref().map(|b| b.stats.sanity())
    );
    Ok(())
}
pub(super) async fn check(a: &mut Assets) -> Result<()> {
    std::fs::create_dir_all(crate::preferences::path(""))?;
    let selected = std::env::var("LOOKING_GLASS_GARDEN_COOP_FIRST").ok();
    let scenes = std::env::var("LOOKING_GLASS_GARDEN_COOP_SCENES").ok();
    ensure!(
        selected
            .as_deref()
            .is_none_or(|v| matches!(v, "Blue" | "Red")),
        "LOOKING_GLASS_GARDEN_COOP_FIRST must be Blue or Red"
    );
    ensure!(
        scenes
            .as_deref()
            .is_none_or(|s| matches!(s, "watched" | "skipped")),
        "LOOKING_GLASS_GARDEN_COOP_SCENES must be watched or skipped"
    );
    for first in PlayerId::ALL {
        if selected
            .as_ref()
            .is_some_and(|v| *v != format!("{first:?}"))
        {
            continue;
        }
        for skip in [false, true] {
            if scenes.as_deref().is_some_and(|s| (s == "skipped") != skip) {
                continue;
            }
            route(a, first, skip).await?;
        }
    }
    Ok(())
}
