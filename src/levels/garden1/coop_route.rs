//! Ordinary-input paired course through the production combat Arena.
//! The test only chooses controls; scenes, resources and placements belong to gameplay.
use super::*;
use crate::{
    coop::{
        arena::Arena,
        protocol::{Input, Visit},
        world::{Body, Frame, Roster},
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
    /// Optional steering goal for the delayed independent-client test course.
    #[serde(default)]
    network_pos: Option<[f32; 3]>,
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
// A delayed observation can cross a periodic input boundary without containing
// its exact tick. Fire once for the interval, never once per render of a frame.
fn due(tick: usize, elapsed: usize, period: usize, phase: usize) -> bool {
    (phase + period - tick % period) % period < elapsed
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
        body: &Body,
        id: PlayerId,
        wait_before: Option<usize>,
        elapsed: usize,
        lead: f32,
    ) -> Result<(Controls, Input)> {
        let p = &body.player;
        let mut control = Controls::default();
        let mut input = Input {
            yaw: body.yaw,
            pitch: body.pitch,
            ..Default::default()
        };
        let movement_locked = arena.characters[id.index()].attack_movement_locked();
        let targets = arena.npcs.targets();
        self.fight.observe(&targets, FIXED_DT * elapsed as f32);
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
            let steering = delta - p.velocity * lead;
            // A rope approach is a running interception, not a place to stop.
            // Braking at its preceding waypoint makes a delayed guest reach
            // the rope only after falling below its free end.
            let running_approach = lead > 0. && p.rope.is_none()
                && (step.mode == "grab" || (step.mode.is_empty()
                    && steps.get(self.index + 1).is_some_and(|s|
                        s.mode == "grab" || (p.grounded && s.mode == "inputjump"))));
            // Queue a remote jump before its observed approach has already run
            // over the edge. The input still reaches the host normally; this
            // anticipates travel during delivery rather than moving the body.
            let approach =
                if (p.grounded && steps.get(self.index + 1).is_some_and(|s| s.mode == "jump"))
                    || (running_approach && step.mode.is_empty()) {
                    20. + p.velocity.truncate().length() * lead
                } else {
                    20.
                };
            let rise = if lead > 0. {
                (steering.z / 40.).clamp(-1., 1.)
            } else {
                delta.z.signum()
            };
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
                _ => delta.truncate().length() < approach && delta.z.abs() < 52.,
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
                    self.quiet = if visible { 0 } else { self.quiet + elapsed };
                    if self.ticks > 0 && due(self.ticks, elapsed, 600, 0) {
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
                        && self.ticks + elapsed > step.frames
                        && due(self.ticks, elapsed, 240, 0)
                        && visible
                        && p.grounded;
                }
                "wait" => (),
                "grab" if p.rope.is_some() => control.rise = rise,
                "hold" => {
                    control.wish = (steering.truncate() * 4. / 320.).clamp_length_max(1.);
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
                        control.rise = rise;
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
                            jump: p.grounded && approach.z > 25. && due(self.ticks, elapsed, 60, 0),
                            use_pressed: p.rope_prompt(world) == Some("E grab rope")
                                && due(self.ticks, elapsed, 4, 0),
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
                        self.plan_tick += elapsed;
                        while self.plan_tick >= 24 && !self.plan.is_empty() {
                            self.plan.pop_front();
                            self.plan_tick -= 24;
                        }
                    }
                }
                _ => {
                    let check_stuck = due(self.ticks, elapsed, 120, 119);
                    let stuck = check_stuck && p.feet.distance(self.last) < 18.;
                    if check_stuck {
                        self.last = p.feet;
                    }
                    control = Controls {
                        wish: if lead > 0. && !running_approach {
                            (steering.truncate() * 4. / 320.).clamp_length_max(1.)
                        } else {
                            delta.truncate().normalize_or_zero()
                        },
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
                        use_pressed: step.mode == "grab"
                            && due(self.ticks, elapsed, 4, 0)
                            && p.rope.is_none(),
                        ..Default::default()
                    };
                }
            }
            self.ticks += elapsed;
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
/// Independent client-side version of the ordinary-input course. It observes
/// the client's received frame and never writes a body, resource or world state.
pub(crate) struct NetworkDriver {
    steps: Vec<Step>,
    driver: Driver,
    last_tick: Option<u64>,
    free_ticks: usize,
    first_rope_cleared: bool,
    cached: (Controls, Input),
}
#[derive(Serialize)]
pub(crate) struct CourseReport {
    step: usize,
    mode: String,
    ticks: usize,
    quiet: usize,
    cards: bool,
    observed_tick: Option<u64>,
}
fn rope_edge(mode: &str, keys: &mut Input) -> Result<()> {
    // A grab sent before the prompt appears can arrive after automatic
    // attachment. Preserve its intent instead of sending a generic E toggle.
    match mode {
        "grab" | "rope" => keys.rope_edges[0] += 1,
        "release" => keys.rope_edges[1] += 1,
        _ => anyhow::bail!("Unknown Garden course rope action: {mode}"),
    }
    Ok(())
}
impl NetworkDriver {
    /// Spectator time must not become a traversal input interval on revival.
    pub(crate) fn spectate(&mut self, tick: u64) {
        self.last_tick = Some(tick);
        self.cached = Default::default();
        self.driver.plan.clear();
        self.driver.plan_tick = 0;
        self.driver.ticks = 0;
        self.driver.quiet = 0;
        self.driver.fight = Default::default();
    }

    pub(crate) fn new(start: usize) -> Result<Self> {
        let course = include_bytes!("coop_steps.json");
        std::fs::write(crate::preferences::path("garden1-coop-course.json"), course)?;
        let mut steps: Vec<Step> = serde_json::from_slice(course)?;
        for step in &mut steps {
            if let Some(pos) = step.network_pos {
                step.pos = pos;
            }
        }
        ensure!(start < steps.len(), "Garden course start is out of range");
        // A checkpoint fixture may begin at an explicit ordinary-input step.
        // It changes the test controller only, never the restored world/body.
        let cards = steps[..start].iter().rev().find(|s| s.mode == "cards")
            .is_some_and(|s| s.pos[0] != 0.);
        Ok(Self {
            steps,
            driver: Driver { index: start, cards, ..Default::default() },
            last_tick: None,
            free_ticks: if start == 0 { 0 } else { 240 },
            first_rope_cleared: false,
            cached: Default::default(),
        })
    }
    pub(crate) fn report(&self) -> CourseReport {
        CourseReport {
            step: self.driver.index,
            mode: self
                .steps
                .get(self.driver.index)
                .map_or("finished", |s| s.mode.as_str())
                .into(),
            ticks: self.driver.ticks,
            quiet: self.driver.quiet,
            cards: self.driver.cards,
            observed_tick: self.last_tick,
        }
    }
    pub(crate) fn sample(
        &mut self,
        frame: &Frame,
        arena: &Arena,
        world: &World,
        local: PlayerId,
        first: PlayerId,
        lead: f32,
        keys: &mut Input,
    ) -> Result<()> {
        let controller = arena
            .stage
            .interactions
            .levels
            .iter()
            .find_map(|s| s.ctl.downcast_ref::<Garden>())
            .context("Garden course command used outside Garden")?;
        let fresh = self.last_tick != Some(frame.tick);
        if fresh {
            let elapsed = self
                .last_tick
                .map_or(1, |old| frame.tick.saturating_sub(old) as usize);
            ensure!(
                elapsed > 0 && elapsed <= 120 * 3,
                "Garden course observation clock reset or stalled"
            );
            self.last_tick = Some(frame.tick);
            let body = &frame.players[local.index()];
            self.cached = Default::default();
            if !arena.stage.busy() && body.stats.alive() {
                // The Rabbit scene rejoins both players beyond the first two
                // ropes. Resume the authored course from that observed story
                // completion, even if a guest missed intermediate scene frames.
                if controller.saved.rabbit_done && self.driver.index < 26 {
                    self.driver.index = 26;
                    self.driver.ticks = 0;
                    self.driver.quiet = 0;
                    self.driver.plan.clear();
                    self.driver.last = body.player.feet;
                }
                let leader = &frame.players[first.index()].player;
                self.first_rope_cleared |= leader.feet.y > -560. && leader.feet.z > 0.;
                let wait_before = (local != first && !controller.saved.rabbit_done)
                    .then_some(if self.first_rope_cleared { 20 } else { 5 });
                if local == first || self.free_ticks >= 240 {
                    self.cached = self.driver.sample(
                        &self.steps,
                        arena,
                        world,
                        body,
                        local,
                        wait_before,
                        elapsed,
                        lead,
                    )?;
                }
                self.free_ticks += elapsed;
            }
        }
        let (control, input) = self.cached;
        keys.wish = control.wish;
        keys.swim = control.swim;
        keys.rise = control.rise;
        keys.run = control.run;
        keys.yaw = input.yaw;
        keys.pitch = input.pitch;
        keys.weapon = input.weapon;
        keys.primary = input.primary;
        keys.alternate = input.alternate;
        if fresh {
            keys.jumps += u64::from(control.jump);
            if control.use_pressed {
                rope_edge(&self.steps[self.driver.index].mode, keys)?;
            }
        }
        keys.validate()
    }
}

#[cfg(test)]
mod clock_tests {
    use super::due;
    #[test]
    fn spectator_clock_retains_route_but_drops_predeath_inputs() {
        use super::*;
        let mut course = NetworkDriver {
            steps: Vec::new(),
            driver: Driver { index: 43, ticks: 336, quiet: 17, cards: true, ..Default::default() },
            last_tick: Some(14212), free_ticks: 240, first_rope_cleared: true,
            cached: (Controls { jump: true, ..Default::default() }, Input { primary: true, ..Default::default() }),
        };
        course.spectate(15425);
        assert_eq!(course.last_tick, Some(15425));
        assert_eq!(course.driver.index, 43);
        assert!(course.driver.cards);
        assert_eq!((course.driver.ticks, course.driver.quiet), (0, 0));
        assert!(!course.cached.0.jump && !course.cached.1.primary);
    }
    #[test]
    fn coalesced_observations_do_not_miss_periodic_control_edges() {
        for period in [4, 60, 120, 240, 600] {
            for phase in [0, period - 1] {
                for stride in [1, 3, 7, 17] {
                    for start in 0..period * 2 {
                        let expected = (start..start + stride).any(|tick| tick % period == phase);
                        assert_eq!(due(start, stride, period, phase), expected);
                        assert!(!due(start, 0, period, phase));
                    }
                }
            }
        }
    }
    #[test]
    fn early_grab_cannot_release_a_rope_that_attaches_before_delivery() {
        use crate::{
            coop::protocol::{Input, RemoteInput},
            movement::{Controls, Player},
        };
        use macroquad::prelude::*;
        let mut world = crate::collision::World::fixture(&[]);
        world.traversal.ropes.push(crate::traversal::Rope {
            id: crate::entity::Id(1),
            model: 1,
            enabled: true,
            origin: Vec3::ZERO,
            anchor: Vec3::Z * 320.,
            length: 320.,
        });
        let far = Player::new(vec3(300., 0., 40.));
        assert!(far.rope_prompt(&world).is_none());
        let mut keys = Input::default();
        super::rope_edge("grab", &mut keys).unwrap();
        assert_eq!(keys.uses, 0);
        // By delivery, the authoritative body has caught the rope naturally.
        let mut player = Player::new(vec3(24., 0., 40.));
        player.tick(&world, Controls::default());
        assert!(player.rope.is_some());
        let mut remote = RemoteInput::default();
        let now = std::time::Instant::now();
        remote.accept(1, keys, now).unwrap();
        let (mut movement, input) = remote.sample(now);
        movement.use_pressed |= input.rope_use(&player, &world);
        player.tick(&world, movement);
        assert!(player.rope.is_some(), "A delayed grab detached the player");
        super::rope_edge("release", &mut keys).unwrap();
        remote.accept(2, keys, now).unwrap();
        let (mut movement, input) = remote.sample(now);
        movement.use_pressed |= input.rope_use(&player, &world);
        player.tick(&world, movement);
        assert!(player.rope.is_none(), "The intended release was lost");
        assert_eq!(keys.uses, 0);
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
async fn route(a: &mut Assets, first: PlayerId, skip: bool, allow_revival: bool) -> Result<()> {
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
    let mut steps: Vec<Step> = serde_json::from_slice(&course)?;
    if allow_revival {
        for step in &mut steps {
            if let Some(point) = step.network_pos { step.pos = point; }
        }
    }
    let steering_lead: f32 = std::env::var("LOOKING_GLASS_GARDEN_COOP_STEERING")
        .ok().map(|s|s.parse()).transpose()?.unwrap_or(0.);
    ensure!(steering_lead.is_finite() && (0. ..=0.5).contains(&steering_lead),
        "Garden steering lookahead must be between zero and half a second");
    ensure!(
        !steps.is_empty()
            && steps.iter().all(|s| s.pos.iter().all(|v| v.is_finite())
                && s.red_pos
                    .is_none_or(|pos| pos.iter().all(|v| v.is_finite()))
                && s.network_pos
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
    let mut rejoins = Vec::new();
    let mut scene_starts = Vec::new();
    let rejoin_path = crate::preferences::path(&format!("garden1-{first:?}-{skip}-rejoins.json"));
    std::fs::write(&rejoin_path, b"[]")?;
    let mut recovery = allow_revival.then(|| crate::coop::recovery_trace::Trace::new(
        crate::preferences::path(&format!("garden1-{first:?}-{skip}-recovery.jsonl")))).transpose()?;
    if let Some(trace) = &mut recovery {
        trace.record(&roster, &scene.world, visit.epoch, &visit.map, arena.stage.busy(),
            arena.stage.interactive_actor().is_some(), |at| !arena.stage.interactions.hazardous(at))?;
    }
    for tick in 0..120 * 1200 {
        let indices_before = drivers.each_ref().map(|d| d.index);
        let owner_before = arena.stage.owner;
        let feet_before = roster.players.each_ref().map(|b| b.player.feet);
        let warps_before = roster.players.each_ref().map(|b| b.warped_at);
        let ropes_before = roster.players.each_ref().map(|b| b.player.rope.clone());
        let landings_before = roster.players.each_ref().map(|b| b.player.landings);
        let health_before = roster.players.each_ref().map(|b| b.stats.sanity());
        let movement_before = roster.players.each_ref().map(|b| b.player.clone());
        let spawns_before = roster.players.each_ref().map(|b| b.spawned_at);
        let inputs = (|| -> Result<_> {
            let mut out = std::array::from_fn(|_| (Controls::default(), Input::default()));
            if !arena.stage.busy() {
                for id in PlayerId::ALL {
                    if !roster.players[id.index()].stats.alive() { continue; }
                    let wait_before = if allow_revival && !roster.players[id.partner().index()].stats.alive()
                        && roster.players[id.index()].player.grounded {
                        // Regroup on reached ground before attempting another
                        // crossing alone. Combat remains available while waiting.
                        Some(drivers[id.index()].index)
                    } else {(id != first && !rabbit_seen).then_some(
                        if drivers[first.index()].index < 11 {
                            5
                        } else {
                            20
                        },
                    )};
                    if id == first || free_ticks >= 240 {
                        out[id.index()] = drivers[id.index()].sample(
                            &steps,
                            &arena,
                            &scene.world,
                            &roster.players[id.index()],
                            id,
                            wait_before,
                            1,
                            steering_lead,
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
        // Keep actual input progress separate from a later revival's cursor jump.
        let sampled_indices = drivers.each_ref().map(|d| d.index);
        let cinematic_before = arena.stage.busy();
        arena.step(&mut roster, &scene.map, &mut scene.world, inputs)?;
        if let Some(trace) = &mut recovery {
            trace.record(&roster, &scene.world, visit.epoch, &visit.map,
                cinematic_before || arena.stage.busy(), arena.stage.interactive_actor().is_some(),
                |at| !arena.stage.interactions.hazardous(at))?;
        }
        if owner_before.is_none() && arena.stage.owner.is_some() {
            scene_starts.push(serde_json::json!({"tick":roster.tick,"owner":arena.stage.owner,
                "feet":roster.players.each_ref().map(|b|b.player.feet)}));
        }
        if allow_revival && roster.checkpoint_generation == 0 {
            for i in 0..2 {
                let body = &roster.players[i];
                if health_before[i] > 0. && !body.stats.alive() {
                    drivers[i].plan.clear();
                    drivers[i].plan_tick = 0;
                    flight_ticks[i] = 0;
                }
                if body.spawned_at != spawns_before[i] {
                    ensure!(health_before[i] == 0. && body.stats.sanity() == 25.
                        && body.spawned_at == roster.tick && roster.players[1-i].stats.alive(),
                        "Garden continuation has no ordinary revival");
                    // Respawn has already placed Alice beside the survivor. Only
                    // the test's input cursor moves; never rewind across a one-way jump.
                    let next = drivers[1-i].index;
                    let cards = drivers[1-i].cards;
                    rejoins.push(serde_json::json!({"tick":roster.tick,"player":i,
                        "from_step":drivers[i].index,"to_step":next,"survivor_step":drivers[1-i].index,
                        "feet":body.player.feet,"survivor_feet":roster.players[1-i].player.feet}));
                    drivers[i] = Driver {index:next,cards,last:body.player.feet,..Default::default()};
                    std::fs::write(&rejoin_path, serde_json::to_vec(&rejoins)?)?;
                    println!("Garden ordinary revival player{i} tick{} follows survivor step{next}",roster.tick);
                }
            }
        }
        for id in PlayerId::ALL {
            for step in indices_before[id.index()]..sampled_indices[id.index()] {
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
                if body.player.landing_speed > 800. {
                    std::fs::write(crate::preferences::path(&format!(
                        "garden1-{first:?}-{skip}-impact-{}-{id:?}.json",roster.tick)),
                        serde_json::to_vec(&serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),
                            "before":movement_before,"frame":roster.frame(1,0),"world":arena.snapshot(),
                            "steps":drivers.each_ref().map(|d|d.index),"inputs":inputs.each_ref().map(|(c,n)|
                                serde_json::json!({"wish":c.wish,"run":c.run,"jump":c.jump,"rise":c.rise,
                                    "swim":c.swim,"use_pressed":c.use_pressed,"weapon":n.weapon}))}))?)?;
                }
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
                allow_revival || arena.stage.owner == Some(first),
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
                .all(|b| !b.stats.god && !b.stats.notarget
                    && (allow_revival || b.stats.alive() && b.spawned_at == 0))
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
        if !arena.stage.busy() && roster.players.iter().all(|b|b.stats.alive())
            && roster.players.iter().any(|b|b.player.rope.is_some()) {
            let spacing = roster.players[0].player.feet - roster.players[1].player.feet;
            if spacing.truncate().length() < crate::coop::separation::DISTANCE-0.05
                && spacing.z.abs() < crate::collision::PLAYER_HALF.z*2.-0.05 {
                record(&arena, &roster, &drivers, first, skip, samples)?;
                anyhow::bail!("Garden rope bodies overlap at tick{tick}: {spacing:?}");
            }
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
            && roster.players.iter().all(|b| b.stats.alive())
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
    let mut result = report(&arena, &roster, &drivers, first, skip, samples);
    result["full_route_passed"] = true.into();
    result["allow_revival"] = allow_revival.into();
    result["steering_lead"] = steering_lead.into();
    result["network_waypoints"] = allow_revival.into();
    result["revival_rejoins"] = serde_json::to_value(&rejoins)?;
    result["scene_starts"] = serde_json::to_value(&scene_starts)?;
    result["reunions"] = reunions.into();
    result["arrival"] = serde_json::json!({"visit":next_visit,"frame":arrived.frame(2,0)});
    std::fs::write(crate::preferences::path(&format!("garden1-{first:?}-{skip}-route.json")),
        serde_json::to_vec(&result)?)?;
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
    let recovery = std::env::var("LOOKING_GLASS_GARDEN_COOP_RECOVERY").ok();
    ensure!(recovery.as_deref().is_none_or(|s| matches!(s,"strict" | "allowed")),
        "LOOKING_GLASS_GARDEN_COOP_RECOVERY must be strict or allowed");
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
            route(a, first, skip, recovery.as_deref() == Some("allowed")).await?;
        }
    }
    Ok(())
}
