//! Explicit local test controller. Only ordinary client inputs leave this module;
//! it cannot set positions, health, inventory or encounters. A neutral host
//! command may request the same checkpoint operation as the pause menu.
//! Each process plans from its own received view. The harness supplies goals via
//! that process's isolated profile, never authoritative state through the wire.
use super::{arena::Arena, protocol::Input, world::Frame, PlayerId};
use crate::{collision::World, movement::{Controls, Player}, route::Route};
use anyhow::{ensure, Result};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeSet, VecDeque}, path::PathBuf, sync::mpsc, time::{Duration, Instant, SystemTime}};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    id: u64,
    heartbeat: u64,
    epoch: u64,
    goal: Option<Vec3>,
    #[serde(default)]
    marker: Option<usize>,
    #[serde(default)]
    horizontal: bool,
    #[serde(default)]
    rope: Option<RopeCommand>,
    #[serde(default)]
    run: bool,
    #[serde(default)]
    jump: bool,
    #[serde(default)]
    runup: Option<Runup>,
    #[serde(default)]
    use_aim: Option<Vec2>,
    #[serde(default)]
    watch_dialogue: bool,
    #[serde(default)]
    beyond_step: Option<usize>,
    /// Opt-in swimming fixture: identifies the first arrival, not a state write.
    #[serde(default)]
    temple_follow: Option<PlayerId>,
    /// Test controller's estimate of observation plus input delivery delay.
    /// Only anticipates steering; it never changes the received body or physics.
    #[serde(default)]
    input_lead: f32,
    /// Optional analogue-input cap for narrow test-route landings. This scales
    /// ordinary movement input only; authoritative movement stays unchanged.
    #[serde(default)]
    speed_limit: Option<f32>,
    /// Straight-walking stopping precision for authored steps and ledges.
    /// Older fixtures keep their original eight-unit tolerance.
    #[serde(default = "default_arrival_radius")]
    arrival_radius: f32,
    navigate: bool,
    attack: bool,
    /// Opt-in route tactic. Sends normal weapon/attack inputs and spends the
    /// player's actual Will; old fixtures retain their Blade-only behavior.
    #[serde(default)]
    prefer_cards: bool,
    #[serde(default)]
    checkpoint: bool,
    capture: bool,
    finish: bool,
}
fn default_arrival_radius() -> f32 { 8. }

fn straight_wish(delta: Vec2, velocity: Vec2, lead: f32, run: bool, limit: Option<f32>) -> Vec2 {
    let steering = delta - velocity * lead;
    let wish = steering.normalize_or_zero() * (steering.length() / 48.).min(1.);
    let full = if run { 320. } else { crate::movement::WALK_SPEED };
    // The limit caps requested speed; multiplying every input by it also
    // suppresses already-slow final corrections and can stall on a slope.
    limit.map_or(wish, |speed| wish.clamp_length_max((speed / full).min(1.)))
}
impl Command {
    fn validate(&self) -> Result<()> {
        ensure!(self.id > 0 && self.epoch > 0 && self.heartbeat > 0, "Invalid route command identity");
        ensure!(self.input_lead.is_finite() && (0. ..=1.5).contains(&self.input_lead), "Invalid input lead");
        ensure!(self.arrival_radius.is_finite() && (1. ..=8.).contains(&self.arrival_radius),
            "Invalid route arrival radius");
        if let Some(speed) = self.speed_limit {
            ensure!(speed.is_finite() && (1. ..=320.).contains(&speed)
                && !self.navigate && self.rope.is_none() && self.runup.is_none(),
                "Invalid route input speed limit");
        }
        ensure!(self.goal.is_none_or(|p| p.is_finite() && p.abs().max_element() < 1_000_000.),
            "Invalid route goal");
        ensure!(self.goal.is_none() || self.marker.is_none(), "Choose a route goal or a marker");
        ensure!(self.marker.is_none_or(|i| i < 100_000), "Invalid route marker");
        ensure!(!(self.run || self.jump) || !self.navigate && self.rope.is_none(),
            "Explicit run/jump needs straight movement, not a planned route or rope command");
        if let Some(runup) = &self.runup {
            ensure!(self.jump && self.run && self.goal.is_some() && (1..=500).contains(&runup.millis)
                && runup.wish.is_finite() && (runup.wish.length()-1.).abs()<0.01,
                "Invalid running-jump approach");
        }
        if let Some(rope) = &self.rope { rope.validate()?; }
        if let Some(aim) = self.use_aim {
            ensure!(aim.is_finite() && aim.x.abs() <= std::f32::consts::TAU && aim.y.abs() <= 1.56
                && self.goal.is_none() && self.marker.is_none() && self.rope.is_none()
                && !self.navigate && !self.attack && !self.run && !self.jump, "Invalid control-use command");
        }
        if let Some(step) = self.beyond_step {
            ensure!((1..=15).contains(&step) && self.goal.is_none() && self.marker.is_none()
                && self.rope.is_none() && self.use_aim.is_none() && !self.navigate && !self.run && !self.jump,
                "Invalid moving-step command");
        }
        if self.temple_follow.is_some() {
            ensure!(self.goal.is_none() && self.marker.is_none() && self.rope.is_none()
                && self.use_aim.is_none() && self.beyond_step.is_none() && self.speed_limit.is_none()
                && !self.navigate && !self.attack && !self.run && !self.jump && !self.checkpoint && !self.finish,
                "Temple swimming requires its own neutral route command");
        }
        ensure!(!self.checkpoint || (self.goal.is_none() && self.marker.is_none()
            && self.rope.is_none() && self.use_aim.is_none() && self.beyond_step.is_none()
            && !self.navigate && !self.attack && !self.run && !self.jump && !self.finish),
            "Checkpoint requests require neutral inputs");
        Ok(())
    }
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Runup {
    millis: u16,
    wish: Vec2,
}
impl Runup {
    fn active(&self, local: PlayerId, ticks: u64, elapsed: Duration) -> bool {
        if local == PlayerId::Blue {
            ticks < (u64::from(self.millis)*120).div_ceil(1000)
        } else {
            elapsed < Duration::from_millis(u64::from(self.millis))
        }
    }
}
#[derive(Clone, Deserialize)]
#[serde(tag="action", rename_all="snake_case", deny_unknown_fields)]
enum RopeCommand {
    Approach { goal: Vec2 },
    Climb { rise: f32 },
    Release { goal: Vec2 },
}
impl RopeCommand {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Approach { goal } | Self::Release { goal } => ensure!(goal.is_finite() && goal.abs().max_element() < 1_000_000., "Invalid rope goal"),
            Self::Climb { rise } => ensure!(rise.is_finite() && rise.abs() <= 1., "Invalid rope climb input"),
        }
        Ok(())
    }
    fn sample(&self, player: &Player, world: &World, keys: &mut Input, jumped: &mut bool, used: &mut bool) -> (&'static str, bool) {
        match self {
            Self::Approach { goal } => {
                if player.rope.is_some() { return ("holding rope", true); }
                keys.wish = (*goal - player.feet.truncate()).normalize_or_zero(); keys.run = true;
                if !*jumped { keys.jumps += 1; *jumped = true; }
                // One edge per command: an unacknowledged grab must never turn
                // into a second E press that releases a delayed guest's grip.
                if !*used && player.rope_prompt(world).is_some() { keys.uses += 1; *used = true; }
                ("approaching rope", false)
            }
            Self::Climb { rise } => {
                keys.rise = *rise;
                ("climbing rope", player.rope.is_some())
            }
            Self::Release { goal } => {
                if player.rope.is_some() && !*used { keys.uses += 1; *used = true; }
                if player.rope.is_none() && player.grounded { return ("landed from rope", true); }
                // Steer away during the drop, otherwise the automatic nearby
                // grab can catch the same rope again before a route can plan.
                keys.wish = (*goal - player.feet.truncate()).normalize_or_zero(); keys.run = true;
                ("releasing rope", false)
            }
        }
    }
}
#[derive(Default, Serialize)]
pub struct Report {
    pub command: Option<u64>,
    pub phase: &'static str,
    pub reached: bool,
    pub captured: Option<u64>,
    pub checkpoint: Option<CheckpointReport>,
    pub replans: usize,
    pub error: Option<String>,
    pub observed_epoch: u64,
    pub observed_tick: u64,
    pub scene: Option<String>,
    pub speech: bool,
    pub clear: [bool; 2],
    /// Nearby supported stopping points; only the local fixture reads these.
    pub neighbors: [Vec<Vec3>; 2],
    pub blade_pickups: [bool; 2],
    pub cards_pickups: [bool; 2],
    pub expected: Option<Vec3>,
    pub plan_remaining: usize,
    pub jumps: u64,
    /// Read-only encounter diagnostics. Selection exists only on the host;
    /// guests restore enemy presentation without running another AI simulation.
    pub guards: Vec<EnemyReport<crate::combat::State>>,
    pub boojums: Vec<EnemyReport<crate::boojum::State>>,
    pub beyond_steps: Option<[Vec3; 15]>,
    pub network_states: VecDeque<NetworkState>,
    pub swim_planning: Option<SwimPlanning>,
}
#[derive(Serialize)]
pub struct SwimPlanning {
    requested_tick: u64,
    completed_tick: u64,
    prepare_ms: f32,
    search_ms: f32,
    accepted: bool,
}
#[derive(Serialize)]
pub struct CheckpointReport {
    pub command: u64,
    pub epoch: u64,
    pub tick: u64,
    pub error: Option<String>,
}
#[derive(Serialize)]
pub struct NetworkState {
    epoch: u64,
    tick: u64,
    sha256: String,
    /// Includes shared movers, quest progress and scene state at this exact
    /// sent/restored tick, independent of local diagnostic report timing.
    world_sha256: String,
    /// Native residents are separate from the registered encounter controllers.
    native_sha256: String,
}
#[derive(Serialize)]
pub struct EnemyReport<S> {
    id: usize,
    name: String,
    active: bool,
    feet: Vec3,
    health: f32,
    state: S,
    selected_player: Option<usize>,
    distance: [f32; 2],
    line_of_sight: [bool; 2],
}
/// School Two retains its original quest-owned enemies alongside Encounters.
/// Observe both pools so a route cannot mistake an unreported live enemy for
/// cleared combat. This does not run AI or modify either client world.
pub(crate) fn school2_reports(school: &crate::school2::School2, frame: &Frame, world: &World)
    -> (Vec<EnemyReport<crate::combat::State>>, Vec<EnemyReport<crate::boojum::State>>) {
    let view = |center: Vec3| (
        frame.players.each_ref().map(|b| center.distance(b.player.eye())),
        frame.players.each_ref().map(|b| {
            let trace = world.sweep(center, b.player.eye(), Vec3::splat(0.5));
            b.stats.alive() && !b.stats.ignores_alice() && !trace.start_solid && trace.fraction >= 1.
        }),
    );
    let guards = school.diagnostic_guards().map(|(id, name, active, guard)| {
        let (distance, line_of_sight) = view(guard.target(id).center);
        EnemyReport { id, name, active, feet:guard.feet, health:guard.health, state:guard.state,
            selected_player:guard.opponents.selected_player, distance, line_of_sight }
    }).collect();
    let boojums = school.boojums.iter().enumerate().map(|(i, boojum)| {
        let id = crate::school2::ENEMY_BASE + i;
        let (distance, line_of_sight) = view(boojum.target(id).center);
        EnemyReport { id, name:format!("school2_boojum_{i}"), active:boojum.active && !school.cinematic(),
            feet:boojum.feet, health:boojum.health, state:boojum.state,
            selected_player:boojum.opponents.selected_player, distance, line_of_sight }
    }).collect();
    (guards, boojums)
}
struct Worker {
    id: u64,
    epoch: u64,
    warped: u64,
    start: Vec3,
    result: mpsc::Receiver<Result<PlannedRoute, usize>>,
}
struct PlannedRoute {
    steps: Vec<(Controls, Vec3)>,
    walking: bool,
}
struct SwimWorker {
    command: u64,
    location: (u64,u64,u64,u64),
    tick: u64,
    from: Vec3,
    prepare_ms: f32,
    result: mpsc::Receiver<(Vec3,f32)>,
}
// Timed input sequences are needed for jumps. A route that stays grounded can
// instead follow its intermediate positions, so a moving obstacle cannot make
// it turn before it has actually reached the corner.
fn plan_stays_grounded(world: &World, start: &Player, steps: &[(Controls, Vec3)]) -> bool {
    if !start.grounded || start.swimming || steps.is_empty() { return false; }
    let mut player=start.clone();
    for (input,_) in steps {
        if input.jump { return false; }
        for _ in 0..24 {
            player.tick(world,*input);
            if !player.grounded || player.swimming || player.ledge.is_some() { return false; }
        }
    }
    true
}
fn walking_control(player: &Player, point: Vec3, lead: f32) -> (Vec2, bool) {
    let delta=point-player.feet;
    let reached=player.grounded && delta.truncate().length()<8. && delta.z.abs()<35.;
    let steering=delta.truncate()-player.velocity.truncate()*lead;
    (steering.normalize_or_zero()*(steering.length()/48.).min(1.),reached)
}
struct WalkingProgress {
    feet: Vec3,
    advanced: Instant,
}
impl WalkingProgress {
    fn new(feet: Vec3, now: Instant) -> Self { Self {feet,advanced:now} }
    fn stalled(&mut self, feet: Vec3, now: Instant) -> bool {
        if feet.distance(self.feet)>8. { self.feet=feet; self.advanced=now; }
        now.duration_since(self.advanced)>=Duration::from_secs(1)
    }
}
/// The host clamps a slow render frame to 100 ms of simulation. Match that
/// bound locally without using delayed snapshot arrivals as a movement clock.
/// A long frame must never fast-forward through inputs that were not sampled.
struct NavigationClock {
    sampled: Instant,
    elapsed: Duration,
}
impl NavigationClock {
    fn new(now: Instant) -> Self { Self { sampled:now, elapsed:Duration::ZERO } }
    fn advance(&mut self, now: Instant) -> bool {
        self.elapsed += now.duration_since(self.sampled).min(Duration::from_millis(100));
        self.sampled = now;
        let step = Duration::from_millis(200);
        if self.elapsed < step { return false; }
        self.elapsed -= step;
        true
    }
}

// A broad planner landing is not a settled destination. Correct it at input
// sampling frequency; the outer harness reports only every 200 ms and can miss
// a brief landing entirely while delayed movement carries Alice over its edge.
fn landing_control(world: &World, player: &Player, goal: Vec3, lead: f32, radius: f32) -> Option<(Vec2, bool)> {
    let delta = goal-player.feet;
    // Waiting for a delayed grounded snapshot is too late on a small pad.
    // Brake on the final descent, once above and near the destination, with
    // a clear horizontal approach. Being beside a platform is not enough:
    // slowing the planned sideways approach there can miss its edge entirely.
    // Require support below the body near the intended landing height first.
    // Match the route's 35-unit walking height tolerance for sloping supports
    // and approximate waypoints; a two-unit band misses legitimate landings.
    let height_tolerance=35.;
    let descent = !player.grounded && !player.swimming && player.ledge.is_none()
        && player.velocity.z < 0. && (-128. ..=height_tolerance).contains(&delta.z)
        && delta.truncate().length() < 128. && {
            let hit=world.body_trace(player.feet,goal+Vec3::Z*(-delta.z));
            let down=vec3(player.feet.x,player.feet.y,goal.z-height_tolerance);
            let support=world.body_trace(player.feet,down);
            !hit.start_solid && hit.fraction >= 1.
                && !support.start_solid && support.fraction < 1. && support.normal.z >= 0.65
                && (player.feet.lerp(down,support.fraction).z-goal.z).abs() < height_tolerance
        };
    if !(player.grounded && delta.length()<42.) && !descent { return None; }
    let settled = player.grounded && delta.truncate().length() < radius && player.velocity.truncate().length() < 20.;
    let steering = delta.truncate()-player.velocity.truncate()*lead;
    let wish = if settled { Vec2::ZERO } else { steering.normalize_or_zero()*(steering.length()/48.).min(1.) };
    Some((wish,settled))
}

// Selection policy must not remove unselected bodies from the collision
// context: an idle guard can still stand between Alice and an active foe.
fn attack_point(context: &crate::combat::Context<'_>, eye: Vec3, target: &crate::combat::Target) -> Option<Vec3> {
    let standard=crate::route::fight::aim_point(context,eye,target);
    let clear=|point| crate::combat::contact(context,eye,point,8.)
        .is_some_and(|(id,_)|id==target.id);
    if standard.is_some_and(clear) { return standard; }
    // A narrow sight ray may see the centre while a real projectile clips
    // the platform rim. Check exposed sides as well as the top of the body;
    // all points stay inside its actual hit box and retain world occlusion.
    let extent=(target.half-Vec3::splat(4.)).max(Vec3::ZERO);
    for z in [1.,0.,-1.] { for x in [0.,-1.,1.] { for y in [0.,-1.,1.] {
        let point=target.center+extent*vec3(x,y,z);
        if clear(point) { return Some(point); }
    }}}
    standard
}
fn attack_target<'a>(world: &World, targets: &'a [crate::combat::Target], eye: Vec3,
    eligible: impl Fn(usize) -> bool) -> Option<(&'a crate::combat::Target, Vec3)> {
    let context = crate::combat::Context { world, targets };
    targets.iter().filter(|t| eligible(t.id) && t.center.distance(eye) < 1000.)
        .filter_map(|t|attack_point(&context,eye,t).map(|point|(t,point)))
        .min_by(|(a,_),(b,_)| a.center.distance_squared(eye).total_cmp(&b.center.distance_squared(eye)))
}

pub struct Driver {
    path: PathBuf,
    command: Option<Command>,
    modified: Option<SystemTime>,
    polled: Instant,
    refreshed: Instant,
    used: Instant,
    door_requests: BTreeSet<crate::entity::Id>,
    door_context: Option<(u64, u64, u64, u64)>,
    worker: Option<Worker>,
    plan: VecDeque<(Controls, Vec3)>,
    walking_plan: bool,
    walking_progress: WalkingProgress,
    step_tick: u64,
    step_at: Instant,
    navigation_clock: NavigationClock,
    jumped: bool,
    rope_used: bool,
    control_used: bool,
    step_height: Option<f32>,
    swim_goal: Option<(u64, Vec3)>,
    swim_worker: Option<SwimWorker>,
    location: Option<(u64, u64, u64)>,
    pub report: Report,
}
impl Driver {
    /// Compare actual sent/restored states, independent of the two report clocks.
    /// Only the opt-in local harness keeps this bounded diagnostic history.
    pub fn record_state(&mut self, frame: &Frame, arena: &Arena) -> Result<()> {
        use sha2::{Digest, Sha256};
        let states = &mut self.report.network_states;
        if states.back().is_some_and(|s| (s.epoch,s.tick) == (frame.epoch,frame.tick)) { return Ok(()); }
        let enemies = arena.stage.interactions.encounters.as_ref().map(|c| c.snapshot());
        let bytes = serde_json::to_vec(&(frame.epoch, frame.tick, frame.checkpoint_generation,
            &frame.players, &frame.lives, enemies))?;
        let world = serde_json::to_vec(&(frame.epoch, frame.tick, arena.stage.snapshot()))?;
        let native = serde_json::to_vec(&(frame.epoch, frame.tick, arena.npcs.snapshot()))?;
        states.push_back(NetworkState { epoch:frame.epoch, tick:frame.tick,
            sha256:format!("{:x}",Sha256::digest(bytes)),
            world_sha256:format!("{:x}",Sha256::digest(world)),
            native_sha256:format!("{:x}",Sha256::digest(native)) });
        while states.len() > 64 { states.pop_front(); }
        Ok(())
    }
    pub fn new(path: PathBuf) -> Self {
        let now = Instant::now();
        Self { path, command: None, modified: None, polled: now, refreshed: now, used: now,
            door_requests: BTreeSet::new(), door_context: None,
            worker: None, plan: VecDeque::new(), walking_plan:false, step_tick: 0, step_at: now,
            walking_progress:WalkingProgress::new(Vec3::ZERO,now),
            navigation_clock:NavigationClock::new(now), jumped: false, rope_used: false,
            control_used: false, step_height: None, swim_goal: None, swim_worker: None,
            location: None, report: Report::default() }
    }
    pub fn poll(&mut self, now: Instant) {
        if now.duration_since(self.polled) < Duration::from_millis(50) { return; }
        self.polled = now;
        let read = || -> Result<Option<(SystemTime, Command)>> {
            let meta = match self.path.metadata() { Ok(meta) => meta, Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None), Err(e) => return Err(e.into()) };
            ensure!(meta.len() <= 4096, "Route command is too large");
            let modified = meta.modified()?;
            if Some(modified) == self.modified { return Ok(None); }
            let command: Command = serde_json::from_slice(&std::fs::read(&self.path)?)?;
            command.validate()?;
            Ok(Some((modified, command)))
        };
        match read() {
            Ok(Some((modified, command))) => {
                if self.command.as_ref().is_none_or(|old| old.id != command.id) {
                    self.plan.clear(); self.jumped = false; self.rope_used = false; self.location = None;
                    self.control_used = false;
                    self.step_height = None;
                    self.swim_goal = None;
                    self.report.error = None;
                }
                self.report.command = Some(command.id);
                self.command = Some(command); self.modified = Some(modified); self.refreshed = now;
            }
            Ok(None) => (),
            Err(error) => { self.report.error = Some(error.to_string()); self.command = None; self.plan.clear(); }
        }
    }
    fn fresh(&self, now: Instant) -> bool { now.duration_since(self.refreshed) < Duration::from_secs(3) }
    /// Local host only; no save request is sent over the network. Acknowledging
    /// failures as well as successes prevents a heartbeat from retrying disk I/O.
    pub fn checkpoint_request(&self, epoch: u64, now: Instant) -> Option<u64> {
        self.command.as_ref().filter(|c| c.checkpoint && c.epoch == epoch && self.fresh(now)
            && self.report.checkpoint.as_ref().is_none_or(|r| (r.epoch,r.command) != (epoch,c.id)))
            .map(|c| c.id)
    }
    pub fn checkpoint_result(&mut self, command: u64, epoch: u64, tick: u64, error: Option<String>) {
        self.report.checkpoint = Some(CheckpointReport { command, epoch, tick, error });
    }
    fn neutral(keys: &mut Input) {
        keys.wish = Vec2::ZERO; keys.swim = Vec3::ZERO; keys.rise = 0.;
        keys.run = false; keys.primary = false; keys.alternate = false; keys.weapon = None;
    }
    // Observe after simulation, alongside the frame being written. Input sampling
    // happens before a host step and can otherwise label a finished scene with
    // its previous name but the new (empty) owner.
    pub fn observe(&mut self, frame: &Frame, arena: &Arena, world: &World) {
        self.report.observed_epoch = frame.epoch;
        self.report.observed_tick = frame.tick;
        self.report.beyond_steps = arena.stage.interactions.beyond.as_ref()
            .map(|b|std::array::from_fn(|i|b.step_goal(i+1)));
        self.report.clear = frame.players.each_ref().map(|b| !b.stats.alive() || world.body_clear(b.player.feet));
        self.report.neighbors = frame.players.each_ref().map(|b| {
            if b.stats.alive() && b.player.grounded && !arena.stage.busy() {
                super::placement::neighbors(world,b.player.feet,|at| !arena.stage.interactions.hazardous(at))
            } else { Vec::new() }
        });
        self.report.blade_pickups = frame.players.each_ref().map(|b| b.stats.copies(0) > 0
            && arena.pickups.iter().any(|p| matches!(p.kind, crate::inventory::PickupKind::Weapon(0))
                && b.stats.collected.contains(&p.id)));
        self.report.cards_pickups = frame.players.each_ref().map(|b| b.stats.copies(1) > 0
            && arena.pickups.iter().any(|p| matches!(p.kind, crate::inventory::PickupKind::Weapon(1))
                && b.stats.collected.contains(&p.id)));
        self.report.scene = arena.stage.interactions.village.as_ref().and_then(|v|v.cinema.scene_id())
            .or_else(|| arena.stage.interactions.pandemonium.as_ref().and_then(|p|p.scene_id()))
            .or_else(|| arena.stage.interactions.fortress.as_ref().and_then(|f|f.state.cinema.scene_id()))
            .or_else(|| arena.stage.interactions.school.as_ref().and_then(|s|s.scene_id()))
            .or_else(|| arena.stage.interactions.school2.as_ref().and_then(|s|s.scene_id()))
            .or_else(|| arena.stage.interactions.beyond.as_ref().and_then(|b|b.scene_id())).map(str::to_owned);
        self.report.speech = arena.stage.story.busy() && !arena.stage.busy();
        self.report.guards = arena.stage.interactions.encounters.iter().flat_map(|cast| &cast.actors).enumerate()
            .filter_map(|(index, actor)| {
                let crate::encounters::Enemy::Guard(guard) = &actor.enemy else { return None; };
                let center = guard.target(0).center;
                Some(EnemyReport {
                    id: crate::encounters::BASE + index,
                    name: actor.name.clone(), active: actor.active && actor.enabled,
                    feet: guard.feet, health: guard.health, state: guard.state,
                    selected_player: guard.opponents.selected_player,
                    distance: frame.players.each_ref().map(|b| center.distance(b.player.eye())),
                    line_of_sight: frame.players.each_ref().map(|b| {
                        let trace = world.sweep(center, b.player.eye(), Vec3::splat(0.5));
                        b.stats.alive() && !b.stats.ignores_alice() && !trace.start_solid && trace.fraction >= 1.
                    }),
                })
            }).collect();
        self.report.boojums = arena.stage.interactions.encounters.iter().flat_map(|cast| &cast.actors).enumerate()
            .filter_map(|(index, actor)| {
                let crate::encounters::Enemy::Boojum(boojum) = &actor.enemy else { return None; };
                let center = boojum.target(0).center;
                Some(EnemyReport {
                    id: crate::encounters::BASE + index,
                    name: actor.name.clone(), active: actor.active && actor.enabled,
                    feet: boojum.feet, health: boojum.health, state: boojum.state,
                    selected_player: boojum.opponents.selected_player,
                    distance: frame.players.each_ref().map(|b| center.distance(b.player.eye())),
                    line_of_sight: frame.players.each_ref().map(|b| {
                        let trace = world.sweep(center, b.player.eye(), Vec3::splat(0.5));
                        b.stats.alive() && !b.stats.ignores_alice() && !trace.start_solid && trace.fraction >= 1.
                    }),
                })
            }).collect();
        if let Some(school) = &arena.stage.interactions.school2 {
            let (guards, boojums) = school2_reports(school, frame, world);
            self.report.guards.extend(guards);
            self.report.boojums.extend(boojums);
        }
    }
    pub fn sample(&mut self, frame: &Frame, arena: &Arena, world: &World, map: &crate::bsp::Bsp, local: PlayerId, keys: &mut Input, now: Instant) {
        Self::neutral(keys);
        self.report.reached = false;
        self.report.expected = self.plan.front().map(|(_, at)| *at);
        self.report.plan_remaining = self.plan.len();
        self.report.jumps = keys.jumps;
        self.report.phase = "idle";
        let body = &frame.players[local.index()];
        self.observe_doors((frame.epoch, frame.checkpoint_generation, body.warped_at, body.spawned_at),
            &arena.stage.interactions);
        let location = (frame.epoch, body.warped_at, body.spawned_at);
        if self.location != Some(location) {
            self.plan.clear(); self.jumped = false; self.location = Some(location);
            self.swim_goal = None;
            self.step_tick = frame.tick; self.step_at = now;
            self.navigation_clock = NavigationClock::new(now);
        }
        // Never spawn a second worker while a superseded plan is still running.
        if let Some(worker) = &self.worker {
            let result = match worker.result.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => Some(Err(0)),
                Err(mpsc::TryRecvError::Empty) => None,
            };
            if let Some(result) = result {
                if self.command.as_ref().is_some_and(|c| c.id == worker.id && c.epoch == worker.epoch)
                    && frame.epoch == worker.epoch && body.warped_at == worker.warped
                    && body.player.feet.distance(worker.start) < 16. && !arena.stage.busy() {
                    match result {
                        Ok(plan) => {
                            let trace = serde_json::json!({ "build": env!("LOOKING_GLASS_BUILD"),
                                "epoch": worker.epoch, "command": worker.id, "start": worker.start,
                                "walking":plan.walking,
                                "steps": plan.steps.iter().map(|(input, expected)|serde_json::json!({
                                    "wish": input.wish, "jump": input.jump, "run": input.run, "expected": expected
                                })).collect::<Vec<_>>() });
                            if let Ok(bytes) = serde_json::to_vec(&trace) {
                                let _ = std::fs::write(crate::preferences::path(&format!("route-plan-{}-{}.json",worker.id,self.report.replans)),bytes);
                            }
                            self.walking_plan=plan.walking;
                            self.walking_progress=WalkingProgress::new(body.player.feet,now);
                            self.plan = plan.steps.into(); self.step_tick = frame.tick; self.step_at = now; self.jumped = false;
                            self.navigation_clock = NavigationClock::new(now);
                        }
                        Err(0) => self.report.error = Some("Route planning worker ended without a result".into()),
                        Err(states) => self.report.error = Some(format!("No route after {states} physics states from {:?}", worker.start)),
                    }
                }
                self.worker = None;
            }
        }
        let Some(command) = self.command.as_ref().filter(|c| c.epoch == frame.epoch && self.fresh(now)) else {
            self.report.phase = "waiting for command"; return;
        };
        if !body.stats.alive() { self.report.phase = "spectating"; return; }
        if command.checkpoint { self.report.phase = "saving checkpoint"; return; }
        if !command.watch_dialogue && arena.stage.story.busy() && arena.stage.owner == Some(local)
            && now.duration_since(self.used) >= Duration::from_millis(250) {
            keys.uses += 1; self.used = now;
        }
        if arena.stage.busy() {
            self.plan.clear(); self.jumped = false; self.report.phase = "scene";
            self.swim_goal = None;
            return;
        }
        if let Some(first) = command.temple_follow {
            let Some(temple) = arena.stage.interactions.levels.iter()
                .find_map(|s|s.ctl.downcast_ref::<crate::levels::utemple::Temple>()) else {
                self.report.error = Some("Temple swim command used outside the temple".into()); return;
            };
            let location = (frame.epoch,frame.checkpoint_generation,body.warped_at,body.spawned_at);
            if let Some(worker) = &self.swim_worker {
                match worker.result.try_recv() {
                    Ok((goal,search_ms)) => {
                        let accepted = worker.command == command.id && worker.location == location
                            && frame.tick.saturating_sub(worker.tick) <= 120
                            && body.player.feet.distance(worker.from) < 96.
                            && world.body_trace(body.player.feet,goal).fraction == 1.;
                        self.report.swim_planning = Some(SwimPlanning {
                            requested_tick:worker.tick,completed_tick:frame.tick,
                            prepare_ms:worker.prepare_ms,search_ms,accepted,
                        });
                        if accepted { self.swim_goal = Some((frame.tick,goal)); }
                        self.swim_worker = None;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.report.error = Some("Temple swimming search ended without a result".into());
                        self.swim_worker = None; return;
                    }
                    Err(mpsc::TryRecvError::Empty) => (),
                }
            }
            if self.swim_worker.is_none()
                && self.swim_goal.is_none_or(|(tick,_)| frame.tick.saturating_sub(tick) >= 12) {
                let started = Instant::now();
                let search = match temple.test_swim_search(map,world,&body.player,local == first) {
                    Ok(search) => search,
                    Err(error) => { self.report.error = Some(error.to_string()); return; }
                };
                if let Some(goal) = search.direct(world) {
                    self.swim_goal = Some((frame.tick,goal));
                } else {
                    let world = world.clone();
                    let (send,result) = mpsc::channel();
                    self.swim_worker = Some(SwimWorker {command:command.id,location,tick:frame.tick,
                        from:body.player.feet,prepare_ms:started.elapsed().as_secs_f32()*1000.,result});
                    self.report.replans += 1;
                    std::thread::spawn(move || {
                        let started = Instant::now();
                        let goal = search.waypoint(&world);
                        let _ = send.send((goal,started.elapsed().as_secs_f32()*1000.));
                    });
                }
            }
            let Some((_,goal)) = self.swim_goal else {
                self.report.phase = "planning temple swim"; return;
            };
            // A mover may have entered the old leg while the next search ran.
            // Brake through ordinary neutral input until a clear leg is ready.
            if world.body_trace(body.player.feet,goal).fraction < 1. {
                self.swim_goal = None; self.report.phase = "planning temple swim"; return;
            }
            let offset = goal-body.player.feet-body.player.velocity*command.input_lead;
            keys.swim = offset.normalize_or_zero()*(offset.length()/30.).min(1.);
            keys.run = true;
            if keys.swim.length_squared() > 0.01 {
                keys.yaw = keys.swim.y.atan2(keys.swim.x);
                keys.pitch = (keys.swim.z/keys.swim.length()).asin().clamp(-1.56,1.56);
            }
            self.report.expected = Some(goal);
            self.report.phase = "following temple guide";
            return;
        }
        if let Some(aim) = command.use_aim {
            keys.yaw = aim.x; keys.pitch = aim.y;
            if !self.control_used { keys.uses += 1; self.control_used = true; }
            self.report.phase = "using control"; self.report.reached = true;
            return;
        }
        if let Some(rope) = &command.rope {
            let (phase, reached) = rope.sample(&body.player, world, keys, &mut self.jumped, &mut self.rope_used);
            self.report.phase = phase; self.report.reached = reached;
            if keys.wish.length() > 0. { keys.yaw = keys.wish.y.atan2(keys.wish.x); keys.pitch = 0.; }
            return;
        }
        if command.attack && body.stats.copies(0) > 0 {
            let mut targets = arena.npcs.targets();
            if let Some(cast) = &arena.stage.interactions.encounters { targets.extend(cast.targets()); }
            if let Some(school) = &arena.stage.interactions.school2 { targets.extend(school.targets()); }
            targets.extend(arena.stage.interactions.shot_targets());
            // Match the native route's active-fight policy. A distant idle
            // guard may be visible beyond its engagement range; do not waste
            // Will on it before approaching. Keep every body in hit testing.
            if let Some((_,point)) = attack_target(world,&targets,body.player.eye(),|id|
                !command.prefer_cards || !self.report.guards.iter().any(|guard|
                    guard.id == id && guard.state == crate::combat::State::Idle)) {
                let delta = point-body.player.eye();
                let cards = command.prefer_cards && body.stats.copies(1) > 0
                    && body.stats.will() >= crate::route::fight::CARD_COST;
                keys.weapon = Some(if cards { 1 } else { 0 });
                keys.primary = cards || delta.length()<80.; keys.alternate = !keys.primary;
                let aim=if keys.alternate {crate::route::fight::lead(body.player.eye(),point,Vec3::ZERO)}
                    else {delta.normalize_or_zero()};
                keys.yaw=aim.y.atan2(aim.x); keys.pitch=aim.z.asin().clamp(-1.56,1.56);
            }
        }
        if let Some(index) = command.beyond_step {
            let Some(beyond) = &arena.stage.interactions.beyond else {
                self.report.error = Some("Moving-step command requires Beyond the Wall".into()); return;
            };
            let goal = beyond.step_goal(index);
            let delta = goal - body.player.feet;
            let distance = delta.truncate().length();
            self.report.expected = Some(goal);
            self.report.reached = distance < 20. && body.player.grounded && delta.z.abs() < 4.;
            if self.report.reached { self.report.phase = "on moving step"; return; }
            let falling = self.step_height.is_some_and(|z|goal.z < z);
            self.step_height = Some(goal.z);
            // Steps four places apart share their wave: the next is always
            // 102.4 units higher. Start while it falls from the upper part of
            // the wave, as in the native route, rather than waiting for an
            // impossible sub-90-unit separation or launching at the trough.
            let upper_wave = index > 12 || goal.z > 25.6*(16-index) as f32 + 80.;
            if !self.jumped && body.player.grounded && falling && upper_wave && delta.z < 110. && distance < 290. {
                keys.jumps += 1; self.jumped = true;
            }
            if self.jumped {
                let steering = delta.truncate() - body.player.velocity.truncate()*command.input_lead;
                keys.wish = steering.normalize_or_zero()
                    * ((2.*420.*steering.length()).sqrt().min(320.)*0.95/320.);
                keys.run = true;
            }
            self.report.phase = "timing moving-step jump";
            return;
        }
        let goal = if let Some(marker) = command.marker {
            match map.entities.get(marker).and_then(|e| e.get("origin")).and_then(|s| crate::interaction::vector(s))
                .filter(|p| p.is_finite() && p.abs().max_element() < 1_000_000.) {
                Some(goal) => Some(goal),
                None => { self.report.error = Some(format!("Missing route marker {marker}")); return; }
            }
        } else { command.goal };
        let Some(goal) = goal else { return; };
        let delta = goal-body.player.feet;
        let landing = if command.navigate {
            landing_control(world,&body.player,goal,command.input_lead,command.arrival_radius)
        } else { None };
        self.report.reached = if command.navigate { landing.is_some_and(|(_,settled)|settled) }
            else { delta.truncate().length()<command.arrival_radius && (command.horizontal || delta.z.abs()<35.) };
        if self.report.reached { self.report.phase="reached"; self.plan.clear(); return; }
        if let Some((wish,_)) = landing {
            self.report.phase="settling landing";
            self.plan.clear();
            keys.wish=wish; keys.swim=wish.extend(0.); keys.run=false;
        } else if command.navigate && self.walking_plan && !self.plan.is_empty() {
            self.report.phase="following walking route";
            // A swinging obstacle can stop against Alice and never clear the
            // old path. Plan around its current pose after a second without
            // progress instead of pushing into it indefinitely.
            if self.walking_progress.stalled(body.player.feet,now) && body.player.grounded
                && body.player.velocity.truncate().length()<20. {
                self.plan.clear();
                self.report.phase="replanning obstructed walk";
            } else {
                while self.plan.front().is_some_and(|(_,point)|
                    walking_control(&body.player,*point,command.input_lead).1) {
                    self.plan.pop_front();
                }
                if let Some((_,point))=self.plan.front() {
                    keys.wish=walking_control(&body.player,*point,command.input_lead).0;
                    keys.swim=keys.wish.extend(0.);
                }
            }
            self.report.plan_remaining=self.plan.len();
            self.report.expected=self.plan.front().map(|(_,point)|*point);
        } else if command.navigate {
            self.report.phase="navigating";
            // A host observes its own simulation clock directly. Guest steps
            // use local frame intervals with the same simulation-time cap.
            // Wall-clock catch-up after a slow frame used to discard whole
            // unsent steps, including jumps. Delayed snapshots are not a clock.
            while !self.plan.is_empty() && if local == PlayerId::Blue {
                frame.tick.saturating_sub(self.step_tick) >= 24
            } else { self.navigation_clock.advance(now) } {
                let (_, expected) = self.plan.pop_front().unwrap();
                self.step_tick += 24; self.step_at += Duration::from_millis(200); self.jumped=false;
                // The host can check a landing immediately. Guest snapshots
                // still show the position before its in-flight inputs: comparing
                // them here restarts a valid plan and duplicates its first move.
                // A guest waits for the full sequence and a settled body before
                // planning again from the resulting authoritative position.
                if local == PlayerId::Blue && body.player.grounded && body.player.feet.distance(expected) > 24. {
                    self.plan.clear();
                }
            }
            if let Some((input,_)) = self.plan.front() {
                keys.wish=input.wish; keys.swim=input.swim; keys.rise=input.rise; keys.run=input.run;
                if input.jump && !self.jumped {
                    keys.jumps+=1; self.jumped=true;
                }
            } else if self.worker.is_none() && self.report.error.is_none()
                && (body.player.grounded || body.player.ledge.is_some()) && body.player.velocity.truncate().length()<20. {
                let (send,result)=mpsc::channel();
                let world=world.clone(); let player:Player=body.player.clone();
                self.worker=Some(Worker { id:command.id, epoch:frame.epoch, warped:body.warped_at, start:player.feet,result });
                self.report.replans+=1;
                // Remote segments can run longer when updates coalesce or
                // arrive with jitter. Avoid plans that jump at the last safe
                // physics tick of an edge; leave 100 ms of support in reserve.
                let margin = if local == PlayerId::Red { 12 } else { 0 };
                std::thread::spawn(move || { let plan=Route::plan_with_takeoff_margin(&world,&player,goal,22.,margin)
                    .or_else(|_|Route::plan_with_takeoff_margin(&world,&player,goal,40.,margin))
                    .map(|steps|PlannedRoute {walking:plan_stays_grounded(&world,&player,&steps),steps});
                    let _=send.send(plan); });
            }
            if self.worker.is_some() { self.report.phase="planning"; }
        } else {
            self.report.phase="walking";
            keys.wish=straight_wish(delta.truncate(), body.player.velocity.truncate(),
                command.input_lead, command.run, command.speed_limit);
            keys.swim=keys.wish.extend(0.);
            keys.run=command.run;
            if let Some(runup) = command.runup.as_ref().filter(|r|
                !self.jumped && r.active(local,frame.tick.saturating_sub(self.step_tick),now.duration_since(self.step_at))) {
                // Time this short approach beside input sampling. Waiting for
                // a report in the outer harness can double its duration and
                // send a player off the pad before the jump arrives.
                keys.wish=runup.wish; keys.swim=keys.wish.extend(0.);
                self.report.phase="running approach";
            } else if command.jump && !self.jumped { keys.jumps+=1; self.jumped=true; }
        }
        if !keys.primary && !keys.alternate && keys.wish.length()>0. {
            keys.yaw=keys.wish.y.atan2(keys.wish.x); keys.pitch=0.;
        }
        self.open_door(&arena.stage.interactions, world, &body.player, keys, now);
    }
    fn open_door(&mut self, interactions: &crate::interaction::Interactions, world: &World,
        player: &Player, keys: &mut Input, now: Instant) {
        if now.duration_since(self.used) < Duration::from_millis(300) { return; }
        if let Some(id) = interactions.closed_door_in_reach(world, player.eye(),
            super::arena::direction(keys.yaw, keys.pitch)) {
            if self.door_requests.insert(id) {
                keys.uses += 1; self.used = now;
            }
        }
    }
    fn observe_doors(&mut self, context: (u64, u64, u64, u64), interactions: &crate::interaction::Interactions) {
        if self.door_context != Some(context) {
            self.door_requests.clear();
            self.door_context = Some(context);
        }
        // Keep an unconfirmed press across command changes and aim changes.
        // A timer alone can close a door before the guest sees it opening.
        // Observe even while idle, so a later genuine closure can be reopened.
        self.door_requests.retain(|id| interactions.closed_door(*id));
    }
    pub fn capture_and_finish(&mut self, now: Instant) -> Result<bool> {
        let Some(command)=self.command.as_ref().filter(|_|self.fresh(now)) else { return Ok(false); };
        if (command.capture || command.finish) && self.report.captured != Some(command.id) {
            crate::viewer::save_capture(&crate::preferences::path(&format!("route-capture-{}.png",command.id)))?;
            self.report.captured=Some(command.id);
        }
        Ok(command.finish)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn speed_limited_walking_finishes_small_corrections_with_delayed_input() {
        let world = World::fixture(&[(vec3(-100., -100., -100.), vec3(100., 100., 0.))]);
        for delay in [0, 12] { for run in [false, true] {
            let mut player = Player::spawn(&world, Vec3::Z * crate::movement::EYE_HEIGHT).unwrap();
            let goal = vec2(4., 0.);
            let mut views = VecDeque::from(vec![player.clone(); delay + 1]);
            let mut pending = VecDeque::from(vec![Controls::default(); delay]);
            for _ in 0..240 {
                let view = views.pop_front().unwrap();
                let delta = goal - view.feet.truncate();
                let wish = if delta.length() < 1. { Vec2::ZERO } else {
                    straight_wish(delta, view.velocity.truncate(), 2. * delay as f32 / 120., run, Some(20.))
                };
                pending.push_back(Controls { wish, run, ..Default::default() });
                player.tick(&world, pending.pop_front().unwrap());
                assert!(player.grounded && player.velocity.truncate().length() <= 20.001);
                views.push_back(player.clone());
            }
            assert!(player.feet.truncate().distance(goal) < 1.1,
                "Slow correction stalled: delay={delay}, run={run}, feet={:?}", player.feet);
        }}
    }
    #[test]
    fn automatic_door_waits_for_delayed_open_confirmation() {
        use crate::interaction::Interactions;
        for delay in [0, 4, 12, 20] { for opposite in [false, true] {
            let map = crate::interaction::tests::fixture();
            let mut host = Interactions::load(&map).unwrap();
            let mut world = World::from_bsp(&map).unwrap();
            host.sync(&mut world);
            let player = Player::new(vec3(32., if opposite { 160. } else { 40. }, 0.));
            let aim = if opposite { -Vec3::Y } else { Vec3::Y };
            let mut views = VecDeque::from(vec![(host.snapshot(), world.clone()); delay + 1]);
            let mut client = Interactions::load(&map).unwrap();
            let mut driver = Driver::new(PathBuf::new());
            let start = driver.used + Duration::from_millis(300);
            let mut keys = Input { yaw: aim.y.atan2(aim.x), ..Default::default() };
            for cycle in 0..2 {
                if cycle == 1 {
                    // A later deliberate closure must allow another opening.
                    host.update(0.05, &map, &mut world, &player, aim, true).unwrap();
                }
                for tick in 0..100 {
                    let (view, observed_world) = views.pop_front().unwrap();
                    client.restore(&view, &map).unwrap();
                    driver.observe_doors((1, 0, 0, 0), &client);
                    let old = keys.uses;
                    driver.open_door(&client, &observed_world, &player, &mut keys,
                        start + Duration::from_millis((cycle * 100 + tick) * 50));
                    host.update(0.05, &map, &mut world, &player, aim, keys.uses != old).unwrap();
                    views.push_back((host.snapshot(), world.clone()));
                }
                assert_eq!(host.used, cycle * 2 + 1,
                    "Delayed door confirmation caused a second toggle: {delay} frames, opposite {opposite}, cycle {cycle}");
                assert_eq!(host.prompt(&world, player.eye(), aim), Some("E  close door"));
            }
        }}
    }
    #[test]
    fn unconfirmed_door_press_survives_aim_changes_but_not_world_relocation() {
        let map = crate::interaction::tests::fixture();
        let mut world = World::from_bsp(&map).unwrap();
        let interactions = crate::interaction::Interactions::load(&map).unwrap();
        interactions.sync(&mut world);
        let player = Player::new(vec3(32., 40., 0.));
        let mut driver = Driver::new(PathBuf::new());
        let start = driver.used + Duration::from_secs(1);
        let mut keys = Input { yaw: std::f32::consts::FRAC_PI_2, ..Default::default() };
        for (index, context) in [(1, 0, 0, 0), (1, 1, 0, 0), (1, 1, 20, 0),
            (1, 1, 20, 30), (2, 1, 20, 30)].into_iter().enumerate() {
            for turn in 0..3 {
                keys.yaw = if turn == 1 { -std::f32::consts::FRAC_PI_2 } else { std::f32::consts::FRAC_PI_2 };
                driver.observe_doors(context, &interactions);
                driver.open_door(&interactions, &world, &player, &mut keys,
                    start + Duration::from_secs((index * 3 + turn) as u64));
                assert_eq!(keys.uses, index as u64 + 1);
            }
        }
    }
    #[test]
    fn checkpoint_commands_are_neutral_fresh_and_acknowledged_once_even_on_failure() {
        let value=serde_json::json!({"id":7,"heartbeat":1,"epoch":2,"goal":null,
            "navigate":false,"attack":false,"capture":false,"finish":false,"checkpoint":true});
        let command:Command=serde_json::from_value(value.clone()).unwrap();
        command.validate().unwrap();
        for (field,bad) in [("goal",serde_json::json!([0,0,0])),("attack",true.into()),
            ("navigate",true.into()),("jump",true.into()),("finish",true.into()),
            ("use_aim",serde_json::json!([0,0]))] {
            let mut invalid=value.clone(); invalid[field]=bad;
            assert!(serde_json::from_value::<Command>(invalid).unwrap().validate().is_err());
        }
        let mut driver=Driver::new(PathBuf::new());
        let now=driver.refreshed;
        driver.command=Some(command);
        assert_eq!(driver.checkpoint_request(1,now),None);
        assert_eq!(driver.checkpoint_request(2,now+Duration::from_secs(3)),None);
        assert_eq!(driver.checkpoint_request(2,now),Some(7));
        driver.checkpoint_result(7,2,100,Some("Wait until the scene finishes".into()));
        assert_eq!(driver.checkpoint_request(2,now),None);
        driver.command.as_mut().unwrap().heartbeat+=1;
        assert_eq!(driver.checkpoint_request(2,now),None);
        driver.command.as_mut().unwrap().id=8;
        assert_eq!(driver.checkpoint_request(2,now),Some(8));
        driver.checkpoint_result(8,2,120,None);
        assert_eq!(driver.checkpoint_request(2,now),None);
    }
    #[test]
    fn walking_routes_replan_when_a_new_obstacle_stops_progress() {
        let floor=(vec3(-200.,-200.,-100.),vec3(400.,200.,0.));
        let open=World::fixture(&[floor]);
        let blocked=World::fixture(&[floor,(vec3(80.,-40.,0.),vec3(100.,40.,200.))]);
        let start=Player::spawn(&open,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let goal=vec3(240.,0.,0.);
        let initial=Route::plan_with_takeoff_margin(&open,&start,goal,22.,12).unwrap();
        assert!(plan_stays_grounded(&open,&start,&initial));
        for replan in [false,true] { for delay in [0,12] {
            let mut player=start.clone();
            let mut steps=initial.clone();
            let mut next=0;
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay]);
            let epoch=Instant::now();
            let mut progress=WalkingProgress::new(player.feet,epoch);
            let mut replans=0;
            for tick in 0..2400 {
                let view=views.pop_front().unwrap();
                let now=epoch+Duration::from_secs_f64(tick as f64/120.);
                let landing=landing_control(&blocked,&view,goal,2.*delay as f32/120.,8.);
                if landing.is_none() && replan && progress.stalled(view.feet,now)
                    && view.grounded && view.velocity.truncate().length()<20. {
                    steps=Route::plan_with_takeoff_margin(&blocked,&view,goal,22.,12).unwrap();
                    assert!(plan_stays_grounded(&blocked,&view,&steps));
                    progress=WalkingProgress::new(view.feet,now);next=0;replans+=1;
                }
                while next<steps.len() && walking_control(&view,steps[next].1,0.).1 {next+=1;}
                let wish=landing.map_or_else(||walking_control(&view,
                    steps.get(next).map_or(goal,|s|s.1),2.*delay as f32/120.).0,|s|s.0);
                pending.push_back(Controls{wish,..Default::default()});
                player.tick(&blocked,pending.pop_front().unwrap());
                assert!(player.grounded,"Walking recovery left the floor");
                views.push_back(player.clone());
            }
            let reached=player.feet.distance(goal)<8.;
            assert_eq!(reached,replan,"Replan={replan}, delay={delay}: {:?}",player.feet);
            assert_eq!(replans>0,replan);
        }}
    }
    #[test]
    fn walking_routes_wait_for_progress_before_turning_past_an_obstacle() {
        let floors=[(vec3(-40.,-160.,-100.),vec3(140.,40.,0.)),
            (vec3(110.,-160.,-100.),vec3(260.,-90.,0.))];
        let open=World::fixture(&floors);
        let blocked=World::fixture(&[floors[0],floors[1],
            (vec3(-40.,-50.,0.),vec3(120.,-45.,100.))]);
        let start=Player::spawn(&open,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let diagonal=vec2(std::f32::consts::FRAC_1_SQRT_2,-std::f32::consts::FRAC_1_SQRT_2);
        let mut planned=start.clone();
        let steps:Vec<_>=[diagonal,diagonal,diagonal,Vec2::X].into_iter().map(|wish| {
            let input=Controls {wish,run:true,..Default::default()};
            for _ in 0..24 {planned.tick(&open,input);}
            (input,planned.feet)
        }).collect();
        assert!(plan_stays_grounded(&open,&start,&steps));
        let mut jumping=steps.clone(); jumping[0].0.jump=true;
        assert!(!plan_stays_grounded(&open,&start,&jumping));
        let gap=World::fixture(&[(vec3(-40.,-40.,-100.),vec3(20.,40.,0.))]);
        assert!(!plan_stays_grounded(&gap,&start,&steps));
        let goal=planned.feet-Vec3::Y*12.;
        let replay=|feedback:bool,delay:usize| {
            let mut player=start.clone();
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay]);
            let mut next=0;
            let mut lowest=player.feet.z;
            for tick in 0..1200 {
                let world=if tick<96 {&blocked} else {&open};
                let view=views.pop_front().unwrap();
                let input=if feedback {
                    while next<steps.len() && walking_control(&view,steps[next].1,0.2).1 {next+=1;}
                    let target=steps.get(next).map_or(goal,|s|s.1);
                    Controls {wish:walking_control(&view,target,2.*delay as f32/120.).0,..Default::default()}
                } else {steps.get(tick/24).map_or(Controls::default(),|s|s.0)};
                pending.push_back(input);
                player.tick(world,pending.pop_front().unwrap());
                views.push_back(player.clone());
                lowest=lowest.min(player.feet.z);
            }
            (player,lowest)
        };
        assert!(replay(false,0).1 < -100.,"Timed control did not reproduce the missed turn");
        for delay in [0,12] {
            let (player,lowest)=replay(true,delay);
            assert!(lowest>=-0.1 && player.grounded && player.feet.distance(goal)<8.,
                "Feedback route failed after {delay} ticks of delay: {:?}, minimum {lowest}",player.feet);
        }
    }
    #[test]
    fn navigation_keeps_lateral_jump_input_until_above_the_landing() {
        // The altar return crosses a gap and turns left late in its jump.
        // A clear horizontal trace in mid-air does not mean the body is over
        // the landing. Braking beside its edge drops Alice against the wall.
        let world=World::fixture(&[
            (vec3(12.,-444.,-100.),vec3(112.,-254.,0.)),
            (vec3(-108.,-64.,-100.),vec3(32.,116.,0.)),
        ]);
        let goal=Vec3::ZERO;
        let diagonal=vec2(-std::f32::consts::FRAC_1_SQRT_2,std::f32::consts::FRAC_1_SQRT_2);
        let path=[(Vec2::Y,false),(Vec2::Y,false),(Vec2::Y,true),(Vec2::Y,false),
            (diagonal,false),(diagonal,false),(-Vec2::X,false),(-Vec2::Y,false)];
        let run=|correct:bool| {
            let mut player=Player::spawn(&world,vec3(83.70465,-371.63672,
                crate::movement::EYE_HEIGHT+0.03125)).unwrap();
            let mut handed_off=false;
            for (wish,jump) in path {
                for tick in 0..24 {
                    let input=if let Some((wish,_))=correct.then(||landing_control(&world,&player,goal,0.,8.)).flatten() {
                        handed_off=true;
                        Controls {wish,..Default::default()}
                    } else if handed_off { Controls::default() }
                    else { Controls {wish,jump:jump&&tick==0,run:true,..Default::default()} };
                    player.tick(&world,input);
                }
            }
            for _ in 0..360 {
                let wish=if correct { landing_control(&world,&player,goal,0.,8.).map_or(Vec2::ZERO,|s|s.0) }
                    else { Vec2::ZERO };
                player.tick(&world,Controls {wish,..Default::default()});
            }
            player
        };
        let planned=run(false);
        assert!(planned.grounded && planned.feet.distance(goal)<42.,"Recorded jump plan missed: {:?}",planned.feet);
        let corrected=run(true);
        assert!(corrected.grounded && corrected.feet.distance(goal)<8.,
            "Landing correction abandoned the lateral approach: {:?}",corrected.feet);
    }
    #[test]
    fn navigation_settles_a_fast_edge_landing_before_reporting_arrival() {
        let world = World::fixture(&[(vec3(-100.,-24.,-100.),vec3(100.,100.,0.))]);
        for delay in [0, 12] {
            let mut player = Player::spawn(&world,vec3(20.9199,-30.2542,crate::movement::EYE_HEIGHT+0.03125)).unwrap();
            player.velocity=vec3(4.1861,-157.485,0.);
            let goal=Vec3::ZERO;
            // Recorded gym landing: old 42-unit arrival test passed while
            // Alice was still moving toward the platform edge at 157 u/s.
            assert!(player.grounded && player.feet.distance(goal)<42.);
            assert!(!landing_control(&world,&player,goal,0.17,8.).unwrap().1);
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            for _ in 0..360 {
                let view=views.pop_front().unwrap();
                let (wish,settled)=landing_control(&world,&view,goal,delay as f32/120.,8.).unwrap();
                if settled {
                    assert!(view.feet.truncate().length()<8. && view.velocity.truncate().length()<20.);
                }
                player.tick(&world,Controls {wish,..Default::default()});
                assert!(player.grounded && player.feet.z.abs()<0.1,"Landing slid off at {:?}",player.feet);
                views.push_back(player.clone());
            }
            assert!(landing_control(&world,&player,goal,0.17,8.).unwrap().1);
        }
    }
    #[test]
    fn navigation_brakes_before_a_delayed_landing_without_cutting_through_walls() {
        let floor=(vec3(-100.,-24.,-100.),vec3(100.,200.,0.));
        let world=World::fixture(&[floor]);
        // Authored route heights can be approximate, especially on sloping
        // supports. Straight walking already accepts a 35-unit height band.
        for height in [-16.,0.,16.] {
          let goal=Vec3::Z*height;
          for delay in [0, 12] {
            // Recorded final descent relative to the gym platform. Simulate
            // delayed observations AND delayed ordinary movement inputs.
            let mut player=Player::new(vec3(27.652,92.514,106.67));
            player.velocity=vec3(0.,-320.,-46.667);
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls {wish:-Vec2::Y,run:true,..Default::default()};delay]);
            for _ in 0..360 {
                let view=views.pop_front().unwrap();
                let control=landing_control(&world,&view,goal,2.*delay as f32/120.,8.)
                    .map(|(wish,_)|Controls {wish,..Default::default()})
                    .unwrap_or(Controls {wish:-Vec2::Y,run:true,..Default::default()});
                pending.push_back(control);
                player.tick(&world,pending.pop_front().unwrap());
                assert!(player.feet.z>=-0.1,"Missed platform at {:?}",player.feet);
                views.push_back(player.clone());
            }
            assert!(landing_control(&world,&player,goal,0.2,8.).unwrap().1);
          }
        }
        let goal=Vec3::ZERO;
        let mut airborne=Player::new(vec3(0.,80.,80.));
        airborne.velocity=-Vec3::Z*100.;
        let wall=World::fixture(&[floor,(vec3(-100.,30.,0.),vec3(100.,40.,200.))]);
        assert!(landing_control(&wall,&airborne,goal,0.2,8.).is_none());
        airborne.velocity.z=100.;
        assert!(landing_control(&world,&airborne,goal,0.2,8.).is_none());
    }
    #[test]
    fn unselected_idle_guard_still_blocks_aim_at_an_enemy_behind_it() {
        let world = World::fixture(&[]);
        let eye = vec3(0.,0.,40.);
        let target = |id, center| crate::combat::Target { id, center, half:Vec3::splat(10.) };
        let mut targets = vec![target(1,vec3(50.,0.,40.)), target(2,vec3(100.,0.,40.))];
        assert!(attack_target(&world,&targets,eye,|id| id!=1).is_none());
        targets.push(target(3,vec3(50.,50.,40.)));
        assert_eq!(attack_target(&world,&targets,eye,|id| id!=1).unwrap().0.id,3);
        assert_eq!(attack_target(&world,&targets,eye,|_| true).unwrap().0.id,1);
    }
    #[test]
    fn ranged_aim_finds_an_exposed_side_and_clears_the_projectile_path() {
        let world=World::fixture(&[
            (vec3(160.,-80.,0.),vec3(170.,80.,46.)),
            (vec3(140.,-4.,46.),vec3(150.,4.,120.))]);
        let targets=[crate::combat::Target{id:1,center:vec3(200.,0.,40.),half:vec3(20.,32.,32.)}];
        let context=crate::combat::Context{world:&world,targets:&targets};
        let eye=vec3(0.,0.,88.);
        assert!(crate::route::fight::aim_point(&context,eye,&targets[0]).is_none(),
            "The regression must require more than a centre or top-centre aim");
        let (target,point)=attack_target(&world,&targets,eye,|_|true).unwrap();
        assert_eq!(target.id,1);
        assert!(point.y.abs()>8. && point.z>target.center.z);
        let shoot=|point| {
            let mut shot=crate::weapons::Projectile::thrown_blade(eye,
                crate::route::fight::lead(eye,point,Vec3::ZERO));
            let mut hit=None;
            for _ in 0..120 {hit=hit.or(shot.contact_step(1./120.,&context).0);}
            hit.map(|h|h.id)
        };
        assert_eq!(shoot(target.center),None);
        assert_eq!(shoot(point),Some(1));
    }
    #[test]
    fn navigation_clock_does_not_skip_unsampled_steps_after_a_render_stall() {
        let start = Instant::now();
        let mut clock = NavigationClock::new(start);
        assert!(!clock.advance(start + Duration::from_millis(100)));
        // The recorded failure had a 450 ms frame on both clients. The host
        // advances only 100 ms, so the guest must consume just one step here.
        let after_stall = start + Duration::from_millis(550);
        assert!(clock.advance(after_stall));
        assert!(!clock.advance(after_stall), "An unsampled jump step was discarded");
        assert!(!clock.advance(after_stall + Duration::from_millis(100)));
        assert!(clock.advance(after_stall + Duration::from_millis(200)));
        assert!(!clock.advance(after_stall + Duration::from_millis(200)));
    }
    #[test]
    fn navigation_clock_preserves_normal_frame_remainders() {
        for millis in [8, 16, 33] {
            let start = Instant::now();
            let mut clock = NavigationClock::new(start);
            let mut steps = 0;
            for frame in 1..=200 {
                steps += u64::from(clock.advance(start + Duration::from_millis(frame * millis)));
                assert_eq!(steps, frame * millis / 200);
            }
        }
    }
    #[test]
    fn running_approach_uses_local_time_when_guest_snapshots_stall() {
        let runup=Runup { millis:150,wish:Vec2::X };
        assert!(runup.active(PlayerId::Blue,17,Duration::from_secs(1)));
        assert!(!runup.active(PlayerId::Blue,18,Duration::ZERO));
        assert!(runup.active(PlayerId::Red,900,Duration::from_millis(149)));
        assert!(!runup.active(PlayerId::Red,0,Duration::from_millis(150)));
        assert!(!runup.active(PlayerId::Red,0,Duration::from_millis(400)));
    }
    #[test]
    fn delayed_rope_acknowledgement_does_not_repeat_grab_or_release_edges() {
        let mut world = World::fixture(&[]);
        world.traversal.ropes.push(crate::traversal::Rope { id:crate::entity::Id(1), model:1, enabled:true,
            origin:Vec3::ZERO, anchor:Vec3::Z*320., length:320. });
        let mut player = Player::new(vec3(24.,0.,0.));
        let mut keys = Input::default();
        let (mut jumped, mut used) = (false,false);
        let approach = RopeCommand::Approach { goal:Vec2::ZERO };
        for _ in 0..30 {
            Driver::neutral(&mut keys);
            assert!(!approach.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        }
        assert_eq!((keys.jumps,keys.uses),(1,1));
        player.rope = Some(crate::movement::RopeGrip { id:crate::entity::Id(1),length:280.,rise:0. });
        Driver::neutral(&mut keys);
        assert!(approach.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        assert_eq!(keys.wish,Vec2::ZERO);
        let release = RopeCommand::Release { goal:Vec2::X*300. }; used=false;
        for _ in 0..30 { Driver::neutral(&mut keys); release.sample(&player,&world,&mut keys,&mut jumped,&mut used); }
        assert_eq!(keys.uses,2);
        player.rope=None; player.grounded=false;
        Driver::neutral(&mut keys);
        assert!(!release.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        assert!(keys.run && keys.wish.x>0.);
        player.grounded=true; Driver::neutral(&mut keys);
        assert!(release.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        assert_eq!(keys.wish,Vec2::ZERO);
    }
    #[test]
    fn legacy_route_commands_remain_valid_and_rope_inputs_are_bounded() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":null,"navigate":false,"attack":true,"capture":false,"finish":false});
        let old:Command=serde_json::from_value(base.clone()).unwrap(); assert!(old.validate().is_ok());
        assert!(old.rope.is_none() && old.marker.is_none() && !old.horizontal && !old.run && !old.jump);
        assert!(old.use_aim.is_none() && !old.watch_dialogue && old.beyond_step.is_none());
        assert!(old.speed_limit.is_none() && !old.prefer_cards && old.temple_follow.is_none());
        assert_eq!(old.arrival_radius,8.);
        for radius in [0.99,8.01] {
            let mut input=base.clone(); input["arrival_radius"]=radius.into();
            assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
        }
        let mut precise=base.clone(); precise["arrival_radius"]=2.into();
        assert!(serde_json::from_value::<Command>(precise).unwrap().validate().is_ok());
        for speed in [0.,321.] {
            let mut input=base.clone(); input["speed_limit"]=speed.into();
            assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
        }
        let mut limited=base.clone(); limited["speed_limit"]=220.into();
        assert!(serde_json::from_value::<Command>(limited.clone()).unwrap().validate().is_ok());
        limited["navigate"]=true.into();
        assert!(serde_json::from_value::<Command>(limited).unwrap().validate().is_err());
        for step in [0,16] {
            let mut input=base.clone(); input["beyond_step"]=step.into();
            assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
        }
        for lead in [-0.1,1.6] {
            let mut input=base.clone(); input["input_lead"]=lead.into();
            assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
        }
        let mut using = base.clone(); using["use_aim"] = serde_json::json!([0.,0.]); using["attack"] = false.into();
        assert!(serde_json::from_value::<Command>(using.clone()).unwrap().validate().is_ok());
        using["use_aim"] = serde_json::json!([0.,2.]);
        assert!(serde_json::from_value::<Command>(using).unwrap().validate().is_err());
        let mut jumping=base.clone(); jumping["jump"]=true.into(); jumping["run"]=true.into();
        assert!(serde_json::from_value::<Command>(jumping.clone()).unwrap().validate().is_ok());
        jumping["navigate"]=true.into();
        assert!(serde_json::from_value::<Command>(jumping).unwrap().validate().is_err());
        for invalid in [serde_json::json!({"action":"climb","rise":2}),serde_json::json!({"action":"release","goal":[1e9,0]})] {
            let mut input=base.clone(); input["rope"]=invalid;
            assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
        }
        let mut input=base; input["goal"]=serde_json::json!([1,2,3]); input["marker"]=serde_json::json!(598);
        assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
    }
    #[test]
    fn temple_swimming_cannot_be_combined_with_other_route_actions() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":null,
            "navigate":false,"attack":false,"capture":false,"finish":false});
        for first in ["Blue","Red"] {
            let mut swim=base.clone(); swim["temple_follow"]=first.into();
            assert!(serde_json::from_value::<Command>(swim.clone()).unwrap().validate().is_ok());
            for (field,value) in [("goal",serde_json::json!([0,0,0])),("marker",1.into()),
                ("attack",true.into()),("navigate",true.into()),("checkpoint",true.into()),
                ("finish",true.into()),("beyond_step",1.into()),("speed_limit",80.into())] {
                let mut invalid=swim.clone(); invalid[field]=value;
                assert!(serde_json::from_value::<Command>(invalid).unwrap().validate().is_err());
            }
        }
        let mut invalid=base; invalid["temple_follow"]="Green".into();
        assert!(serde_json::from_value::<Command>(invalid).is_err());
    }
}
