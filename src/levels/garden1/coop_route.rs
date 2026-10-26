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
    mode: String,
    #[serde(default)]
    frames: usize,
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
    ) -> Result<(Controls, Input)> {
        let body = &roster.players[id.index()];
        let p = &body.player;
        let mut control = Controls::default();
        let mut input = Input {
            yaw: body.yaw,
            ..Default::default()
        };
        // Do not consume a timed jump while the production weapon animation
        // will discard movement. Let the action finish using neutral controls.
        if arena.characters[id.index()].attack_movement_locked() {
            return Ok((control, input));
        }
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
            self.cards && body.stats.copies(1) > 0 && body.stats.will() >= fight::CARD_COST,
        );
        if attack.target.is_some() && p.rope.is_none() {
            input.weapon = Some(attack.weapon as u8);
            input.primary = !attack.alternate;
            input.alternate = attack.alternate;
            input.yaw = attack.aim.y.atan2(attack.aim.x);
            input.pitch = attack.aim.z.asin().clamp(-1.56, 1.56);
            self.fight.weapon = attack.weapon;
            self.fight.last_aim = attack.aim;
        }
        loop {
            let Some(step) = steps.get(self.index) else {
                return Ok((control, input));
            };
            let goal = Vec3::from_array(step.pos);
            let delta = goal - p.feet;
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
                "grab" => p.rope.is_some(),
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
                }
                "wait" => (),
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
                    control.rise = delta.z.signum();
                    control.use_pressed = self.ticks == 0 && p.rope.is_none();
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
                            || (step.mode.is_empty() && stuck),
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
fn record(
    arena: &Arena,
    roster: &Roster,
    drivers: &[Driver; 2],
    first: PlayerId,
    skip: bool,
    samples: usize,
) -> Result<()> {
    let proof = serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"first":first,"skip":skip,"samples":samples,
        "steps":drivers.each_ref().map(|d|d.index),"step_ticks":drivers.each_ref().map(|d|d.ticks),
        "frame":roster.frame(1,0),"world":arena.snapshot()});
    std::fs::write(
        crate::preferences::path(&format!("garden1-{first:?}-{skip}-route.json")),
        serde_json::to_vec(&proof)?,
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
    let steps: Vec<Step> = serde_json::from_slice(include_bytes!("route_steps.json"))?;
    let mut drivers: [Driver; 2] = Default::default();
    let mut free_ticks = 0;
    let mut samples = 0;
    let mut rabbit_seen = false;
    let mut reunions = 0;
    let mut close_recorded = false;
    for tick in 0..120 * 1200 {
        let owner_before = arena.stage.owner;
        let feet_before = roster.players.each_ref().map(|b| b.player.feet);
        let warps_before = roster.players.each_ref().map(|b| b.warped_at);
        let inputs = (|| -> Result<_> {
            let mut out = std::array::from_fn(|_| (Controls::default(), Input::default()));
            if !arena.stage.busy() {
                for id in PlayerId::ALL {
                    // The follower can climb faster from the other entrance
                    // offset. Wait on the second rope before swinging toward
                    // Rabbit so this variant actually exercises the chosen order.
                    if id != first && !rabbit_seen && drivers[id.index()].index == 23 {
                        continue;
                    }
                    if id == first || free_ticks >= 240 {
                        out[id.index()] = drivers[id.index()].sample(
                            &steps,
                            &arena,
                            &scene.world,
                            &roster,
                            id,
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
    println!(
        "PASS full Garden co-op {first:?} skip={skip}: {} ticks, {samples} replicas, health {:?}",
        roster.tick,
        roster.players.each_ref().map(|b| b.stats.sanity())
    );
    Ok(())
}
pub(super) async fn check(a: &mut Assets) -> Result<()> {
    std::fs::create_dir_all(crate::preferences::path(""))?;
    for first in PlayerId::ALL {
        if std::env::var("LOOKING_GLASS_GARDEN_COOP_FIRST")
            .ok()
            .is_some_and(|v| v != format!("{first:?}"))
        {
            continue;
        }
        for skip in [false, true] {
            route(a, first, skip).await?;
        }
    }
    Ok(())
}
