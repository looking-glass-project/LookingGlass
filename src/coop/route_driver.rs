//! Explicit local test controller. Only ordinary client inputs leave this module;
//! it cannot set positions, health, inventory or encounters. A neutral host
//! command may request the same checkpoint operation as the pause menu.
//! Each process plans from its own received view. The harness supplies goals via
//! that process's isolated profile, never authoritative state through the wire.
use super::{arena::Arena, protocol::Input, world::Frame, PlayerId};
use crate::{collision::World, movement::{Controls, Player}, route::Route};
use anyhow::{ensure, Context, Result};
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
    /// Project an ordinary walking goal onto a named live Funhouse fulcrum.
    #[serde(default)]
    surface: Option<usize>,
    /// Continuous ordinary-input Funhouse seesaw approach and upper jump.
    #[serde(default)]
    funhouse_exit: bool,
    /// Continuous ordinary inputs for bank jumps and moving pendulums.
    #[serde(default)]
    funhouse_pendulums: bool,
    /// Opt-in host evidence, separate from gameplay state and the network wire.
    #[serde(default)]
    recovery_trace: bool,
    #[serde(default)]
    marker: Option<usize>,
    /// Read-only projection of a pathnode onto this client's collision world.
    #[serde(default)]
    pathnode: Option<usize>,
    #[serde(default)]
    horizontal: bool,
    /// Ordinary three-dimensional swim input toward an explicit local test goal.
    #[serde(default)]
    swim: bool,
    #[serde(default)]
    rope: Option<RopeCommand>,
    #[serde(default)]
    run: bool,
    #[serde(default)]
    jump: bool,
    /// Timed authored steering toward a point, including points beyond a wall.
    /// The outer runner measures duration; this never claims point arrival.
    #[serde(default)]
    hold_goal: bool,
    #[serde(default)]
    runup: Option<Runup>,
    #[serde(default)]
    use_aim: Option<Vec2>,
    /// Aim and throw the owned Blade at an active authored shootable trigger.
    /// Completion requires observing that same target before it disappears.
    #[serde(default)]
    shoot_switch: Option<String>,
    /// Leave puzzle targets for explicit shooting commands in authored routes.
    /// Legacy diagnostics retain their existing target selection by default.
    #[serde(default)]
    avoid_switches: bool,
    #[serde(default)]
    watch_dialogue: bool,
    /// Hold the earned Watch's ordinary primary input. Inventory, cost and
    /// recharge remain authoritative weapon behavior, observed by the fixture.
    #[serde(default)]
    use_watch: bool,
    /// Ordinary held skip input; authoritative scene ownership still applies.
    #[serde(default)]
    skip_scene: bool,
    #[serde(default)]
    beyond_step: Option<usize>,
    /// Opt-in swimming fixture: identifies the first arrival, not a state write.
    #[serde(default)]
    temple_follow: Option<PlayerId>,
    /// Ordinary-input Garden course, planned independently from this client view.
    #[serde(default)]
    garden_follow: Option<PlayerId>,
    /// Ordinary-input Sanctum fight/climb course from this client's own view.
    #[serde(default)]
    sanctum_follow: Option<PlayerId>,
    /// Pale Realm course step, interpreted independently in each client.
    #[serde(default)]
    chess_follow: Option<PlayerId>,
    #[serde(default)]
    chess_step: Option<usize>,
    /// Castling course step, interpreted independently in each client.
    #[serde(default)]
    castling_follow: Option<PlayerId>,
    #[serde(default)]
    castling_step: Option<usize>,
    /// Initial test-course step for an explicitly restored Garden segment.
    #[serde(default)]
    garden_start: Option<usize>,
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
    /// Stop a planned climb at its first native hang so the outer fixture can
    /// observe the catch and send a separate ordinary pull-up input.
    #[serde(default)]
    ledge_handoff: bool,
    #[serde(default)]
    landing_feedback: bool,
    /// Keep ordinary navigation out of damaging liquids; explicit local
    /// crossing commands can still visit an authored hot waypoint.
    #[serde(default)]
    avoid_liquid_damage: bool,
    /// End a timed route after its first landing, then settle and plan again
    /// from the actual body before attempting another jump.
    #[serde(default)]
    settle_landings: bool,
    attack: bool,
    /// Opt-in route tactic. Sends normal weapon/attack inputs and spends the
    /// player's actual Will; old fixtures retain their Blade-only behavior.
    #[serde(default)]
    prefer_cards: bool,
    /// Ordinary Ice Wand primary fire when owned, funded, dry and in range.
    #[serde(default)]
    prefer_ice: bool,
    /// Ordinary Queen2 checkpoint combat, planned from this client's own view.
    #[serde(default)]
    queen_finale: bool,
    /// Opt-in ordinary-input combat planning: actual melee reach, led throws
    /// and safe strafing while a route command waits without a movement goal.
    #[serde(default)]
    combat_tactics: bool,
    #[serde(default)]
    checkpoint: bool,
    capture: bool,
    finish: bool,
}
fn default_arrival_radius() -> f32 { 8. }
fn hold_wish(delta:Vec2)->Vec2 {
    let wish=(delta*(4./320.)).clamp_length_max(1.);
    if wish.length_squared()<0.000001 {vec2(0.001,0.)} else {wish}
}
fn delayed_hold_wish(delta:Vec2, velocity:Vec2, lead:f32)->Vec2 {
    if lead == 0. { return hold_wish(delta); }
    // A remote timed hold uses delayed positions and delayed input delivery.
    // Predict the pending movement and lower the feedback gain just as ordinary
    // waypoint steering does. The local authored input and world physics stay
    // unchanged; this only chooses the remote test driver's movement input.
    let wish=((delta-velocity*lead)/(80.+640.*lead)).clamp_length_max(1.);
    if wish.length_squared()<0.000001 {vec2(0.001,0.)} else {wish}
}

pub(crate) fn straight_wish(delta: Vec2, velocity: Vec2, lead: f32, run: bool, limit: Option<f32>) -> Vec2 {
    let steering = delta - velocity * lead;
    let full = if run { 320. } else { crate::movement::WALK_SPEED };
    // Delayed velocity feedback also needs a smaller gain. Keeping the local
    // gain makes a remote runner reverse repeatedly around a small waypoint.
    let wish = (steering / (48. + 2. * full * lead)).clamp_length_max(1.);
    // The limit caps requested speed; multiplying every input by it also
    // suppresses already-slow final corrections and can stall on a slope.
    limit.map_or(wish, |speed| wish.clamp_length_max((speed / full).min(1.)))
}

pub(crate) fn direct_goal_wish(world:&World,player:&Player,goal:Vec3,lead:f32,run:bool,limit:Option<f32>)->Vec2 {
    let delta=(goal-player.feet).truncate();let velocity=player.velocity.truncate();
    if !player.grounded && !player.swimming && player.ledge.is_none() && player.updraft_time<=0.
        && landing_control_with_support(world,player,goal,lead,8.,false).is_none() {
        // Grounded arrival damping spends too much of a long jump moving
        // slowly. Over a gap, use the ordinary air acceleration's stopping
        // distance; resume the damped approach above supported landing ground.
        // Choosing damping does not finish a planned jump. Physical body
        // support is enough here; the planner handoff below additionally needs
        // a fully supported footprint before discarding its movement plan.
        let steering=delta-velocity*lead;
        let full=if run {320.} else {crate::movement::WALK_SPEED};
        let wish=steering.normalize_or_zero()*((2.*420.*steering.length()).sqrt().min(full)*0.95/full);
        limit.map_or(wish,|speed|wish.clamp_length_max((speed/full).min(1.)))
    } else {straight_wish(delta,velocity,lead,run,limit)}
}
impl Command {
    fn validate(&self) -> Result<()> {
        ensure!(self.id > 0 && self.epoch > 0 && self.heartbeat > 0, "Invalid route command identity");
        if let Some(id)=self.pathnode {
            ensure!(id<100_000 && self.goal.is_none() && self.marker.is_none() && self.surface.is_none()
                && self.rope.is_none() && self.runup.is_none() && self.use_aim.is_none() && self.shoot_switch.is_none()
                && self.beyond_step.is_none() && self.temple_follow.is_none() && self.garden_follow.is_none()
                && self.sanctum_follow.is_none() && self.chess_follow.is_none() && self.castling_follow.is_none()
                && !self.funhouse_exit && !self.funhouse_pendulums && !self.hold_goal && !self.swim
                && !self.navigate && !self.horizontal && !self.attack && !self.run && !self.jump
                && !self.checkpoint && !self.finish && !self.skip_scene,
                "Pathnode observation requires neutral inputs");
        }
        ensure!(self.input_lead.is_finite() && (0. ..=1.5).contains(&self.input_lead), "Invalid input lead");
        ensure!(self.garden_start.is_none() || self.garden_follow.is_some(),
            "A Garden start requires the Garden course");
        ensure!(self.chess_step.is_some()==self.chess_follow.is_some()
            && self.chess_step.is_none_or(|i|i<1000), "Invalid Pale Realm course selector");
        ensure!(self.castling_step.is_some()==self.castling_follow.is_some()
            && self.castling_step.is_none_or(|i|i<203), "Invalid Castling course selector");
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
        ensure!(!self.avoid_liquid_damage || self.navigate && self.goal.is_some(),
            "Avoiding liquid damage requires a planned goal");
        ensure!(!self.settle_landings || self.navigate && self.goal.is_some(),
            "Settling landings requires a planned goal");
        ensure!(self.surface.is_none_or(|id| id < 100_000 && self.goal.is_some()
            && !self.navigate && !self.horizontal && !self.swim && !self.jump && !self.hold_goal
            && self.rope.is_none() && self.runup.is_none() && self.use_aim.is_none()
            && self.beyond_step.is_none() && !self.checkpoint && !self.finish),
            "A surface goal requires an ordinary straight walk");
        ensure!(!self.swim || (self.goal.is_some() && !self.horizontal && !self.navigate
            && self.rope.is_none() && self.runup.is_none() && self.use_aim.is_none()
            && self.beyond_step.is_none() && self.temple_follow.is_none() && self.garden_follow.is_none() && self.sanctum_follow.is_none() && self.chess_follow.is_none() && self.castling_follow.is_none()
            && self.speed_limit.is_none() && !self.jump && !self.checkpoint && !self.finish),
            "Explicit swimming requires its own three-dimensional route goal");
        ensure!(self.marker.is_none_or(|i| i < 100_000), "Invalid route marker");
        if self.use_watch {
            ensure!(self.goal.is_none() && self.marker.is_none() && self.pathnode.is_none()
                && self.surface.is_none() && self.rope.is_none() && self.runup.is_none()
                && self.use_aim.is_none() && self.shoot_switch.is_none() && self.beyond_step.is_none()
                && self.temple_follow.is_none() && self.garden_follow.is_none()
                && self.sanctum_follow.is_none() && self.chess_follow.is_none() && self.castling_follow.is_none()
                && !self.funhouse_exit && !self.funhouse_pendulums && !self.navigate && !self.horizontal
                && !self.swim && !self.hold_goal && !self.attack && !self.run && !self.jump
                && !self.checkpoint && !self.finish && !self.skip_scene,
                "Watch input requires its own neutral route command");
        }
        ensure!(!self.jump || !self.navigate && self.rope.is_none(),
            "Explicit jump needs straight movement, not a planned route or rope command");
        ensure!(!self.ledge_handoff || self.navigate && self.goal.is_some(),
            "A ledge handoff requires a planned route goal");
        ensure!(!self.landing_feedback || self.navigate && self.goal.is_some(),
            "Landing feedback requires a planned route goal");
        ensure!(!self.run || self.rope.is_none(), "Rope controls own their running input");
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
        let courses=[self.temple_follow,self.garden_follow,self.sanctum_follow,self.chess_follow,self.castling_follow].iter().filter(|c|c.is_some()).count();
        if self.funhouse_exit || self.funhouse_pendulums {
            ensure!(!(self.funhouse_exit && self.funhouse_pendulums), "Cannot combine Funhouse courses");
            ensure!(self.goal.is_none() && self.marker.is_none() && self.surface.is_none()
                && self.rope.is_none() && self.use_aim.is_none() && self.beyond_step.is_none()
                && self.runup.is_none() && self.shoot_switch.is_none() && self.speed_limit.is_none()
                && courses==0 && !self.swim && !self.horizontal && !self.navigate && !self.hold_goal
                && !self.run && !self.jump && !self.checkpoint && !self.finish,
                "The Funhouse exit requires its own ordinary-input sequence");
        }
        if self.hold_goal {
            ensure!(self.goal.is_some() && self.marker.is_none() && self.run && !self.jump
                && !self.navigate && !self.swim && self.rope.is_none() && self.runup.is_none()
                && self.use_aim.is_none() && self.shoot_switch.is_none() && self.beyond_step.is_none()
                && self.speed_limit.is_none() && courses==0 && !self.checkpoint && !self.finish,
                "A timed hold requires its own running goal");
        }
        if let Some(thread)=&self.shoot_switch {
            ensure!(!thread.is_empty() && thread.len()<=128
                && thread.bytes().all(|c|c.is_ascii_alphanumeric() || c==b'_')
                && self.goal.is_none() && self.marker.is_none() && self.rope.is_none()
                && self.use_aim.is_none() && self.beyond_step.is_none() && self.speed_limit.is_none()
                && self.runup.is_none() && courses==0 && !self.swim && !self.navigate
                && !self.attack && !self.run && !self.jump && !self.checkpoint && !self.finish,
                "Shooting a switch requires its own neutral route command");
        }
        if courses>0 {
            ensure!(self.goal.is_none() && self.marker.is_none() && self.rope.is_none()
                && self.use_aim.is_none() && self.beyond_step.is_none() && self.speed_limit.is_none()
                && !self.navigate && !self.attack && !self.run && !self.jump && !self.checkpoint && !self.finish
                && courses==1,
                "A level course requires its own neutral route command");
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
    Climb { rise: f32, #[serde(default)] wish: Vec2 },
    Release { goal: Vec2, #[serde(default)] jump: bool },
}
impl RopeCommand {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Approach { goal } | Self::Release { goal, .. } => ensure!(goal.is_finite() && goal.abs().max_element() < 1_000_000., "Invalid rope goal"),
            Self::Climb { rise, wish } => ensure!(rise.is_finite() && rise.abs() <= 1.
                && wish.is_finite() && wish.length_squared() <= 1.001, "Invalid rope climb input"),
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
            Self::Climb { rise, wish } => {
                keys.rise = *rise; keys.wish = *wish;
                ("climbing rope", player.rope.is_some())
            }
            Self::Release { goal, jump } => {
                // Space + E is the native jump-off gesture. Keep its held rise
                // while a delayed grip is visible, but emit only one use edge.
                if *jump && player.rope.is_some() { keys.rise = 1.; }
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
    pub pathnode: Option<PathnodeReport>,
    pub replans: usize,
    pub error: Option<String>,
    pub recovery_trace_error: Option<String>,
    pub observed_epoch: u64,
    pub observed_tick: u64,
    pub scene: Option<String>,
    pub speech: bool,
    pub clear: [bool; 2],
    pub ledge_pull_clear: [bool; 2],
    pub return_landings: [Vec<Vec3>; 2],
    pub landing_replans: usize,
    pub last_landing_replan: Option<(u64,Vec3)>,
    /// Nearest visible native hostile within the route's 700-unit clear radius.
    pub native_threats: [Option<usize>; 2],
    /// Nearby supported stopping points; only the local fixture reads these.
    pub neighbors: [Vec<Vec3>; 2],
    /// Supported neighbors with a projectile-width sight line to the commanded
    /// switch. This is a current-world observation, not a guaranteed future hit.
    pub switch_approaches: [Vec<Vec3>; 2],
    pub surface: Option<crate::levels::funhouse::RouteSurface>,
    pub carry_velocity: Option<Vec3>,
    pub funhouse_exit: Option<crate::levels::funhouse::ExitReport>,
    pub funhouse_pendulums: Option<crate::levels::funhouse::PendulumReport>,
    pub curiosity_platform: Option<crate::levels::jlair1::PlatformReport>,
    pub blade_pickups: [bool; 2],
    pub cards_pickups: [bool; 2],
    pub expected: Option<Vec3>,
    pub plan_remaining: usize,
    pub jumps: u64,
    /// Read-only encounter diagnostics. Selection exists only on the host;
    /// guests restore enemy presentation without running another AI simulation.
    pub guards: Vec<EnemyReport<crate::combat::State>>,
    pub boojums: Vec<EnemyReport<crate::boojum::State>>,
    pub snarks: Vec<EnemyReport<crate::snark::Phase>>,
    pub beyond_steps: Option<[Vec3; 15]>,
    pub network_states: VecDeque<NetworkState>,
    /// Bounded host-only input evidence; never sent to peers or saved as gameplay.
    pub input_events: VecDeque<serde_json::Value>,
    pub swim_planning: Option<SwimPlanning>,
    pub garden_course: Option<crate::levels::garden1::CourseReport>,
    pub sanctum_course: Option<serde_json::Value>,
    pub chess_course: Option<serde_json::Value>,
    pub castling_course: Option<serde_json::Value>,
}
#[derive(Serialize)]
pub struct PathnodeReport {
    command:u64,
    epoch:u64,
    tick:u64,
    entity:usize,
    origin:Vec3,
    feet:Vec3,
}

pub(crate) fn pathnode_goal(world:&World,entity:&crate::levels::Entity)->Result<(Vec3,Vec3)> {
    ensure!(entity.get("classname").is_some_and(|c|c=="info_pathnode"),"Not a pathnode");
    let origin=entity.get("origin").and_then(|s|crate::interaction::vector(s))
        .filter(|p|p.is_finite() && p.abs().max_element()<1_000_000.).context("Missing finite pathnode origin")?;
    let feet=world.actor_footing(origin+Vec3::Z*32.,crate::collision::PLAYER_CENTER,
        crate::collision::PLAYER_HALF,240.).context("Unsupported pathnode")?;
    Ok((origin,feet))
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
    pub save_ms: f32,
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
    /// Compact actual bodies for verifying landings between outer report polls.
    movement: [MovementEvidence; 2],
}
#[derive(Serialize)]
struct MovementEvidence {
    feet: Vec3,
    velocity: Vec3,
    grounded: bool,
    jumps: u64,
    health: f32,
    spawned_at: u64,
    warped_at: u64,
}
impl From<&super::world::Body> for MovementEvidence {
    fn from(b: &super::world::Body) -> Self {
        Self { feet:b.player.feet, velocity:b.player.velocity, grounded:b.player.grounded,
            jumps:b.player.jumps, health:b.stats.sanity(), spawned_at:b.spawned_at, warped_at:b.warped_at }
    }
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
    ledge_goal: Option<Vec3>,
}

#[derive(Default)]
struct LandingFeedback {
    airborne: bool,
    trail: VecDeque<Vec3>,
}
impl LandingFeedback {
    fn start(&mut self, at: Vec3) {
        self.airborne=false;self.trail.clear();self.trail.push_back(at);
    }
    fn advance(&mut self, at: Vec3) {
        self.trail.push_back(at);
        // Retain more than the maximum command lead, including segment turns.
        while self.trail.len()>16 {self.trail.pop_front();}
    }
    fn misplaced(&mut self, player: &Player, next: Option<Vec3>) -> bool {
        if !player.grounded {
            if !player.swimming && player.ledge.is_none() {self.airborne=true;}
            return false;
        }
        if !std::mem::take(&mut self.airborne) {return false;}
        let points=self.trail.iter().copied().chain(next).collect::<Vec<_>>();
        if points.len()<2 {return false;}
        // Compare against the recent spatial path, not the current timed end
        // point: a delayed but correct landing can be several segments behind.
        let on_path=points.windows(2).any(|p| {
            let delta=p[1]-p[0];
            let t=if delta.length_squared()>0.001 {
                ((player.feet-p[0]).dot(delta)/delta.length_squared()).clamp(0.,1.)
            } else {0.};
            player.feet.distance(p[0]+delta*t)<=48.
        });
        !on_path
    }
}

fn return_landings(world: &World, from: Vec3, goal: Vec3, safe: impl Fn(Vec3)->bool) -> Vec<Vec3> {
    if from.truncate().distance(goal.truncate())>432. {return Vec::new();}
    let mut result=Vec::new();
    for radius in [0.,16.,32.,48.] {
        for index in 0..if radius==0. {1} else {8} {
            let angle=index as f32*std::f32::consts::TAU/8.;
            let point=from.truncate()+vec2(angle.cos(),angle.sin())*radius;
            let Some(at)=world.actor_footing(point.extend(goal.z+48.),
                crate::collision::PLAYER_CENTER,crate::collision::PLAYER_HALF,320.) else {continue;};
            let drop=goal.z-at.z;
            if (56. ..=256.).contains(&drop) && super::placement::supported(world,at) && safe(at)
                && !result.iter().any(|p: &Vec3|p.distance(at)<1.) {result.push(at);}
        }
    }
    result
}
fn stop_at_ledge(world: &World, start: &Player, goal: Vec3,
    mut steps: Vec<(Controls, Vec3)>) -> (Vec<(Controls, Vec3)>,Option<Vec3>) {
    let mut player=start.clone();
    for index in 0..steps.len() {
        let input=steps[index].0;
        for tick in 0..24 {
            player.tick(world,Controls{jump:input.jump && tick==0,..input});
            if player.ledge.as_ref().is_some_and(|hang| goal.z>=hang.top-35.
                && (goal-player.feet).truncate().dot(hang.forward)>0.) {
                // A timed pull can reach the host before its delayed catch.
                // Keep the catch input. The live handoff then approaches until
                // it observes the hang and waits for an explicit pull input.
                let target=steps[index].1;
                steps.truncate(index+1);
                return (steps,Some(target));
            }
        }
    }
    (steps,None)
}
fn ledge_handoff_wish(world: &World, player: &Player, goal: Vec3, grip: Option<Vec3>, lead: f32) -> Option<Vec2> {
    if player.grounded || player.swimming { None }
    else if let Some(hang)=&player.ledge {
        let mut wish=Vec2::ZERO;
        if !hang.pulling && !hang.pull_clear(world) {
            if let Some(grip)=grip {
                let right=vec2(hang.forward.y,-hang.forward.x);
                let delta=(grip-player.feet).truncate().dot(right);
                if delta.abs()>0.5 {
                    let speed=(delta.abs()/(2.+crate::ledge::SHIMMY_SPEED*lead)).clamp(0.3,1.);
                    wish=right*delta.signum()*speed;
                }
            }
        }
        Some(wish)
    }
    else { Some((goal-player.feet).truncate().normalize_or_zero()) }
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
fn through_first_landing(world: &World, start: &Player,
    mut steps: Vec<(Controls, Vec3)>) -> Vec<(Controls, Vec3)> {
    // A timed turn before takeoff can run past an edge before a guest's jump
    // arrives. Walk a supported grounded prefix with positional feedback,
    // then plan the jump again from the actual settled body. Only stop after
    // progress toward the route's endpoint: a backward run-up may exist to
    // build takeoff velocity, and stopping it can replan the same loop forever.
    // A plan starting with a jump, or airborne, keeps its whole first landing.
    let destination=steps.last().map(|s|s.1);
    let mut approach=start.clone();
    let mut supported_prefix:Option<usize>=None;
    if start.grounded && !start.swimming && start.ledge.is_none() {
        'prefix: for (index,(input,_)) in steps.iter().enumerate() {
            if input.jump {
                if let Some(count)=supported_prefix {
                    if destination.is_some_and(|goal|steps[count-1].1.distance(goal)+24.<start.feet.distance(goal)) {
                        steps.truncate(count); return steps;
                    }
                }
                break;
            }
            for _ in 0..24 {
                approach.tick(world,*input);
                if !approach.grounded || approach.swimming || approach.ledge.is_some() { break 'prefix; }
            }
            if approach.feet.distance(start.feet)>24. && crate::route::supported_footing(world,approach.feet)
                && (approach.immersion.level==0 || approach.immersion.kind.damage_per_second()==0.) {
                supported_prefix=Some(index+1);
            }
        }
    }
    let mut player=start.clone();
    let mut airborne=false;
    for (index,(input,_)) in steps.iter().enumerate() {
        for tick in 0..24 {
            player.tick(world,Controls{jump:input.jump && tick==0,..*input});
            airborne |= !player.grounded;
        }
        if airborne && player.grounded && !player.swimming && player.ledge.is_none() {
            // Remote input delay can shift one landing enough that a later
            // takeoff starts from the wrong ledge. Retain this full input
            // segment, then let the normal settled-body gate choose a fresh
            // route. No intermediate arrival or body position is invented.
            steps.truncate(index+1);
            break;
        }
    }
    steps
}
fn straight_grounded_sections(start: Vec3, steps: Vec<(Controls, Vec3)>) -> Vec<(Controls, Vec3)> {
    let mut result = Vec::new();
    let mut anchor = start;
    for (i, step) in steps.iter().enumerate() {
        if let Some(next) = steps.get(i+1) {
            let line = next.1-anchor;
            let fraction = (step.1-anchor).dot(line)/line.length_squared();
            // Remove only points on the same forward three-dimensional line.
            // Preserve every turn/height change in the verified grounded plan.
            if (0. ..1.).contains(&fraction)
                && (step.1-anchor-line*fraction).length()<0.001 { continue; }
        }
        result.push(*step);
        anchor = step.1;
    }
    result
}
fn walking_control(player: &Player, point: Vec3, lead: f32, run: bool) -> (Vec2, bool) {
    let delta=point-player.feet;
    let reached=player.grounded && delta.truncate().length()<8. && delta.z.abs()<35.;
    let steering=delta.truncate()-player.velocity.truncate()*lead;
    // Running needs gentler corrections when observations and inputs are in
    // flight. Otherwise a delayed turn alternates full-speed corrections and
    // never reaches its next waypoint. Keep legacy walking gain unchanged.
    let slowing = if run { (640.*lead).max(48.) } else { 48. };
    (steering.normalize_or_zero()*(steering.length()/slowing).min(1.),reached)
}
fn grounded_waypoint_ready(world: &World, player: &Player, point: Vec3, next: Option<Vec3>, run: bool) -> bool {
    if walking_control(player,point,0.,run).1 { return true; }
    let Some(next)=next else { return false; };
    if !run || !player.grounded || player.swimming || player.ledge.is_some()
        || player.feet.truncate().distance(point.truncate())>=22.
        || (player.feet.z-point.z).abs()>2. || (next.z-point.z).abs()>2. { return false; }
    // A running traveler need not stop at every nearby steering point. Join
    // the next segment only if the actual body can cross that small corner
    // and has continuous floor support. Never use this for final landings.
    let onward=next-point;
    let join=point+onward.clamp_length_max(32.);
    let trace=world.body_trace(player.feet,join);
    if trace.start_solid || trace.fraction<1. { return false; }
    let samples=(player.feet.distance(join)/8.).ceil().max(1.) as usize;
    (0..=samples).all(|i| {
        let at=player.feet.lerp(join,i as f32/samples as f32);
        let support=world.body_trace(at+Vec3::Z*2.,at-Vec3::Z*2.);
        !support.start_solid && support.fraction<1. && support.normal.z>=0.65
    })
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
    fn obstructed(&mut self, player: &Player, now: Instant) -> bool {
        // Collision can retain tangential velocity while the body stays still
        // at a corner. Measure actual displacement, not the requested speed.
        self.stalled(player.feet,now) && player.grounded
    }
}
/// Follow sampled wall time, like the guest input timeline. The host retains
/// simulation debt after a brief stall, so discarding elapsed guest time can
/// hold a run-up past its takeoff. Advance at most once per sampled frame;
/// after a stall give the next, previously unsampled input its full interval.
struct NavigationClock {
    sampled: Instant,
    elapsed: Duration,
}
impl NavigationClock {
    fn new(now: Instant) -> Self { Self { sampled:now, elapsed:Duration::ZERO } }
    fn advance(&mut self, now: Instant) -> bool {
        let interval=now.duration_since(self.sampled);
        if interval.is_zero() { return false; }
        self.elapsed += interval;
        self.sampled = now;
        let step = Duration::from_millis(200);
        if self.elapsed < step { return false; }
        self.elapsed = if interval>Duration::from_millis(100) { Duration::ZERO }
            else { self.elapsed-step };
        true
    }
}

// A broad planner landing is not a settled destination. Correct it at input
// sampling frequency; the outer harness reports only every 200 ms and can miss
// a brief landing entirely while delayed movement carries Alice over its edge.
pub(crate) fn carried_goal_control(player: &Player, goal: Vec3, carry: Vec3, lead: f32, radius: f32) -> (Vec2,bool) {
    let delta=goal-player.feet;
    let velocity=(player.velocity+carry).truncate();
    let settled=player.grounded && delta.truncate().length()<radius && delta.z.abs()<35. && velocity.length()<20.;
    // Request ordinary running input against the carrier's motion, then add
    // delayed positional feedback. Keep compensating after arrival: neutral
    // input would immediately carry the waiting player away from the waypoint.
    let wish=(straight_wish(delta.truncate(),velocity,lead,true,None)-carry.truncate()/320.).clamp_length_max(1.);
    (wish,settled)
}
fn planning_hold_control(anchor: &mut Option<Vec3>, player: &Player, carry: Option<Vec3>, lead: f32) -> (Vec2,bool) {
    // A pull-up advances the body through its animation even with zero
    // velocity. Starting a worker before it finishes snapshots an unstable
    // ledge pose and can reject every route just before the real landing.
    if player.ledge.as_ref().is_some_and(|hang|hang.pulling) {
        *anchor=None;
        return (Vec2::ZERO,false);
    }
    if let Some(carry)=carry {
        // A route worker snapshots the current world and starting body. Hold
        // that world position while it runs so passive carry cannot invalidate
        // every result. This remains ordinary input, including while settling.
        let goal=*anchor.get_or_insert(player.feet);
        carried_goal_control(player,goal,carry,lead,8.)
    } else {
        *anchor=None;
        (Vec2::ZERO,(player.grounded || player.ledge.is_some()) && player.velocity.truncate().length()<20.)
    }
}
pub(crate) fn pending_jump(steps:impl IntoIterator<Item=bool>,current_issued:bool)->bool {
    steps.into_iter().enumerate().any(|(index,jump)|jump && (index>0 || !current_issued))
}

fn supported_goal_walk(world: &World, player: &Player, goal: Vec3) -> bool {
    let delta=goal-player.feet;
    if !player.grounded || player.swimming || player.ledge.is_some()
        || delta.truncate().length()>=42. || delta.z.abs()>=35. { return false; }
    let end=vec3(goal.x,goal.y,player.feet.z);
    let trace=world.body_trace(player.feet,end);
    if trace.start_solid || trace.fraction<1. { return false; }
    let samples=(delta.truncate().length()/8.).ceil().max(1.) as u32;
    (0..=samples).all(|i|crate::route::supported_footing(world,player.feet.lerp(end,i as f32/samples as f32)))
}

pub(crate) fn landing_control(world: &World, player: &Player, goal: Vec3, lead: f32, radius: f32) -> Option<(Vec2, bool)> {
    landing_control_with_support(world,player,goal,lead,radius,true)
}

fn landing_control_with_support(world: &World, player: &Player, goal: Vec3, lead: f32, radius: f32,
    full_footprint: bool) -> Option<(Vec2, bool)> {
    let delta = goal-player.feet;
    // Waiting for a delayed grounded snapshot is too late on a small pad.
    // Brake on the final descent, once above and near the destination, with
    // a clear horizontal approach. Being beside a platform is not enough:
    // slowing the planned sideways approach there can miss its edge entirely.
    // Require support below the body near the intended landing height first.
    // Match the route's 35-unit walking height tolerance for sloping supports
    // and approximate waypoints; a two-unit band misses legitimate landings.
    let height_tolerance=35.;
    // A delayed snapshot can still show ascent even though a new input will
    // arrive after the apex. Anticipate only ordinary gravity; retain the
    // existing current-footing and clear-approach checks below.
    let descending_on_arrival=player.velocity.z<0. || (player.updraft_time<=0.
        && player.velocity.z<crate::movement::GRAVITY*lead.clamp(0.,1.5));
    let descent = !player.grounded && !player.swimming && player.ledge.is_none()
        && descending_on_arrival && (-128. ..=height_tolerance).contains(&delta.z)
        && delta.truncate().length() < 128. && {
            let hit=world.body_trace(player.feet,goal+Vec3::Z*(-delta.z));
            let supported_at=|at:Vec3| {
                let down=vec3(at.x,at.y,goal.z-height_tolerance);
                let support=world.body_trace(at,down);
                let floor=at.lerp(down,support.fraction);
                !support.start_solid && support.fraction<1. && support.normal.z>=0.65
                    && (floor.z-goal.z).abs()<height_tolerance
                    && (!full_footprint || crate::route::supported_footing(world,floor))
            };
            // A box edge over the ledge is not sufficient footing. A delayed
            // guest can already have crossed onto the pad, however: allow its
            // ordinary velocity prediction only with a clear approach and the
            // same supported landing footprint. No player state is changed.
            let anticipated=player.feet+player.velocity.truncate().extend(0.)*lead.clamp(0.,1.5);
            let approach=world.body_trace(player.feet,anticipated);
            !hit.start_solid && hit.fraction>=1. && (supported_at(player.feet)
                || full_footprint && lead>0. && !approach.start_solid && approach.fraction>=1. && supported_at(anticipated))
        };
    // Approximate waypoint heights can put a short ordinary walk outside the
    // spherical planner handoff. A 32-unit-high marker over a flat floor can
    // never satisfy the planner's first 22-unit search, wasting a full search.
    // Keep the walking height bound and require a clear, continuously supported
    // approach before extending only the handoff (not the arrival tolerance).
    let nearby_walk=player.grounded && delta.truncate().length()<42. && delta.z.abs()<height_tolerance && {
        let end=vec3(goal.x,goal.y,player.feet.z);
        let trace=world.body_trace(player.feet,end);
        let samples=(delta.truncate().length()/8.).ceil().max(1.) as u32;
        !trace.start_solid && trace.fraction>=1. && (1..=samples).all(|i| {
            let at=player.feet.lerp(end,i as f32/samples as f32);
            let floor=world.body_trace(at,at-Vec3::Z*2.);
            let centre=world.sweep(at+Vec3::Z*0.1,at-Vec3::Z*2.,Vec3::ZERO);
            !floor.start_solid && floor.fraction<1. && floor.normal.z>=0.65
                && !centre.start_solid && centre.fraction<1. && centre.normal.z>=0.65
        })
    };
    // Proximity alone cannot replace a route around an obstruction. Once this
    // handoff clears the route, straight steering would otherwise push into
    // the same wall forever, even with a clear destination on its other side.
    let near_grounded=player.grounded && delta.length()<42. && {
        let end=vec3(goal.x,goal.y,player.feet.z);
        let hit=world.body_trace(player.feet,end);
        (!hit.start_solid && hit.fraction>=1.) || {
            // A horizontal sweep touches rising ground before normal movement
            // steps onto it. Use the same bounded raise/across/drop clearance
            // as Player::tick; a tall wall or a low ceiling still blocks it.
            let up=world.body_trace(player.feet,player.feet+Vec3::Z*crate::movement::STEP_HEIGHT);
            let height=crate::movement::STEP_HEIGHT*up.fraction;
            let raised=player.feet+Vec3::Z*height;
            let across=world.body_trace(raised,end+Vec3::Z*height);
            let down=world.body_trace(end+Vec3::Z*height,end-Vec3::Z*2.);
            !up.start_solid && !up.all_solid && up.fraction>0.
                && !across.start_solid && across.fraction>=1.
                && !down.start_solid && down.fraction<1. && down.normal.z>=0.65
        }
    };
    if !near_grounded && !nearby_walk && !descent { return None; }
    let settled = player.grounded && delta.truncate().length() < radius && player.velocity.truncate().length() < 20.;
    let steering = delta.truncate()-player.velocity.truncate()*lead;
    // Once supported, use the same delay-aware gain as ordinary final walking.
    // Velocity prediction with a fixed gain oscillates at internet tail latency
    // and can carry a remote Alice off a small takeoff footing. Airborne braking
    // retains its stronger correction so it can still catch the landing.
    let wish = if settled { Vec2::ZERO } else if player.grounded {
        straight_wish(delta.truncate(),player.velocity.truncate(),lead,false,None)
    } else { steering.normalize_or_zero()*(steering.length()/48.).min(1.) };
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

pub(crate) fn visible_switch_approaches(context:&crate::combat::Context<'_>,target:crate::combat::Target,
    neighbors:&[Vec3])->Vec<Vec3> {
    neighbors.iter().copied().filter(|feet| {
        let eye=*feet+Vec3::Z*crate::movement::EYE_HEIGHT;
        attack_point(context,eye,&target).is_some_and(|point|
            crate::combat::contact(context,eye,point,8.).is_some_and(|(id,_)|id==target.id))
    }).collect()
}

/// Read-only headless diagnosis of retained switch positions. This samples the
/// real collision world but does not activate a switch or advance a game.
pub(crate) fn switch_probe(context:&crate::combat::Context<'_>,eye:Vec3,target:crate::combat::Target)
    -> serde_json::Value {
    let selected=attack_point(context,eye,&target);
    let sample=|point:Vec3| {
        let direction=crate::route::fight::lead(eye,point,Vec3::ZERO);
        let mut blade=crate::weapons::Projectile::thrown_blade(eye,direction);
        let mut impact=None;
        for _ in 0..120 {
            let (hit,normal)=blade.contact_step(1./120.,context);
            if hit.is_some() || normal.is_some() {impact=Some(hit.map(|h|h.id));break;}
        }
        serde_json::json!({"point":point,"ray1":crate::combat::contact(context,eye,point,1.).map(|h|h.0),
            "ray8":crate::combat::contact(context,eye,point,8.).map(|h|h.0),"impact":impact,
            "hits_switch":impact==Some(Some(target.id))})
    };
    let mut candidates=Vec::new();
    for z in [-0.75,-0.4,0.,0.4,0.75] {for y in [-0.7,0.,0.7] {for x in [-0.7,0.,0.7] {
        candidates.push(sample(target.center+target.half*vec3(x,y,z)));
    }}}
    serde_json::json!({"eye":eye,"target":{"id":target.id,"center":target.center,"half":target.half},
        "selected":selected.map(sample),"candidates":candidates})
}

/// Keep all combat bodies in the visibility context, including friendly ones.
fn combat_targets(interactions:&crate::interaction::Interactions,
    mut targets:Vec<crate::combat::Target>) -> Vec<crate::combat::Target> {
    if let Some(cast)=&interactions.encounters {targets.extend(cast.targets());}
    if let Some(school)=&interactions.school2 {targets.extend(school.targets());}
    // Native campaign controllers own chess pieces and bosses. Leaving them
    // out makes an attack-enabled replay stand idle beside an attacking pawn.
    targets.extend(interactions.levels_targets());
    targets.extend(interactions.shot_targets());
    targets
}

pub(crate) fn native_threat(world:&World,interactions:&crate::interaction::Interactions,
    targets:Vec<crate::combat::Target>,body:&super::world::Body) -> Option<usize> {
    if !body.stats.alive() {return None;}
    let targets=combat_targets(interactions,targets);
    let eye=body.player.eye();
    attack_target(world,&targets,eye,|id| {
        crate::level::hit_owner(&interactions.levels,id)
            .is_some_and(|slot|interactions.levels[slot].ctl.hostile_target(id))
        && targets.iter().any(|t|t.id==id && t.center.distance(eye)<=700.)
    }).map(|(target,_)|target.id)
}

/// Shared by the diagnostic driver and native data checks; only emits ordinary
/// weapon controls. The checks need no renderer or authoritative state writes.
pub(crate) fn combat_controls(world:&World,interactions:&crate::interaction::Interactions,
    targets:Vec<crate::combat::Target>,body:&super::world::Body,prefer_cards:bool,avoid_switches:bool,
    idle_guard:impl Fn(usize)->bool,keys:&mut Input) {
    if body.stats.copies(0)==0 {return;}
    let targets=combat_targets(interactions,targets);
    let switches=if avoid_switches {interactions.shot_targets()} else {Vec::new()};
    // Retain non-hostile/idle bodies for occlusion, but do not attack them.
    if let Some((_,point))=attack_target(world,&targets,body.player.eye(),|id|
        crate::level::hit_owner(&interactions.levels,id)
            .is_none_or(|slot|interactions.levels[slot].ctl.hostile_target(id))
        && (!prefer_cards || !idle_guard(id)) && !switches.iter().any(|t|t.id==id)) {
        let delta=point-body.player.eye();
        let cards=prefer_cards && body.stats.copies(1)>0 && body.stats.will()>=crate::route::fight::CARD_COST;
        keys.weapon=Some(if cards {1} else {0});
        keys.primary=cards || delta.length()<80.;keys.alternate=!keys.primary;
        let aim=if keys.alternate {crate::route::fight::lead(body.player.eye(),point,Vec3::ZERO)}
            else {delta.normalize_or_zero()};
        keys.yaw=aim.y.atan2(aim.x);keys.pitch=aim.z.asin().clamp(-1.56,1.56);
    }
}

#[derive(Default)]
struct SwitchAim { seen:Option<usize> }
impl SwitchAim {
    fn sample(&mut self,context:&crate::combat::Context<'_>,target:Option<crate::combat::Target>,
        eye:Vec3,keys:&mut Input)->Result<bool> {
        let Some(target)=target else {
            ensure!(self.seen.is_some(),"Switch missing or already used before this command");
            return Ok(true);
        };
        ensure!(self.seen.is_none_or(|id|id==target.id),"Switch target changed during shooting");
        self.seen=Some(target.id);
        if let Some(point)=attack_point(context,eye,&target) {
            let aim=crate::route::fight::lead(eye,point,Vec3::ZERO);
            keys.weapon=Some(0);keys.primary=false;keys.alternate=true;
            keys.yaw=aim.y.atan2(aim.x);keys.pitch=aim.z.asin().clamp(-1.56,1.56);
        }
        Ok(false)
    }
}

#[derive(Default)]
struct CombatAim {
    tracker: crate::route::fight::Tracker,
    sampled: Option<(u64,u64)>,
    closing_since: Option<(u64,usize,u64)>,
}
fn target_may_be_in_range(eye:Vec3,target:&crate::combat::Target,range:f32)->bool {
    // Use the whole hit box: tall bosses can be reachable even when their
    // centre is distant. Keep every body in Context for occlusion and tracking.
    // The one-unit margin makes this a conservative cheap rejection only.
    ((eye-target.center).abs()-target.half).max(Vec3::ZERO).length_squared()<=(range+1.).powi(2)
}
fn ice_aim(context:&crate::combat::Context<'_>, foes:&[crate::combat::Target],
    body:&super::world::Body, enabled:bool, keys:&mut Input) -> Option<crate::combat::Target> {
    if !enabled || body.stats.copies(4)==0 || body.stats.will()<0.75 || body.player.immersion.level!=0 {
        return None;
    }
    let eye=body.player.eye();
    let (target,point)=foes.iter().filter(|target|target_may_be_in_range(eye,target,360.)).filter_map(|target|
        crate::route::fight::aim_point(context,eye,target).map(|point|(*target,point)))
        .filter(|(_,point)|point.distance(eye)<360.)
        .min_by(|a,b|a.1.distance_squared(eye).total_cmp(&b.1.distance_squared(eye)))?;
    let aim=(point-eye).normalize_or_zero();
    keys.weapon=Some(4);keys.primary=true;keys.alternate=false;
    keys.yaw=aim.y.atan2(aim.x);keys.pitch=aim.z.asin().clamp(-1.56,1.56);
    Some(target)
}
impl CombatAim {
    fn sample(&mut self, stamp:(u64,u64), context:&crate::combat::Context<'_>,
        foes:&[crate::combat::Target], eye:Vec3, cards:bool, keys:&mut Input)
        -> Option<crate::combat::Target> {
        if self.sampled!=Some(stamp) {
            let dt=match self.sampled {
                Some((epoch,tick)) if epoch==stamp.0 && stamp.1>tick => (stamp.1-tick) as f32/120.,
                _=>{self.tracker=Default::default();1./120.},
            };
            self.tracker.observe(context.targets,dt);self.sampled=Some(stamp);
        }
        let closing=foes.iter().filter(|target|target_may_be_in_range(eye,target,220.))
            .filter_map(|target|crate::route::fight::aim_point(context,eye,target)
                .map(|point|(*target,point)))
            .filter(|(_,point)|point.distance(eye)<220.)
            .min_by(|(_,a),(_,b)|a.distance_squared(eye).total_cmp(&b.distance_squared(eye)));
        // Give ordinary movement a second to close before throwing away the
        // Blade. If geometry or knockback keeps the target outside its actual
        // melee volume, do not keep swinging out of reach indefinitely.
        self.closing_since=closing.filter(|_|!cards).map(|(target,point)| {
            let now=(stamp.0,target.id,stamp.1);
            if crate::weapons::blade_melee(context,eye,(point-eye).normalize_or_zero())
                .iter().any(|hit|hit.id==target.id) {return now;}
            self.closing_since.filter(|&(epoch,id,tick)|epoch==stamp.0 && id==target.id && tick<=stamp.1)
                .unwrap_or(now)
        });
        self.tracker.hold_blade=self.closing_since.is_some_and(|(_,_,tick)|stamp.1-tick<120);
        let reachable=foes.iter().copied().filter(|target|
            target_may_be_in_range(eye,target,crate::route::fight::THROW_RANGE)).collect::<Vec<_>>();
        let mut plan=crate::route::fight::plan(context,eye,&reachable,&self.tracker,cards);
        if plan.target.is_none() && self.tracker.hold_blade {
            if let Some((target,point))=closing {
                plan=crate::route::fight::Plan{target:Some(target.id),aim:(point-eye).normalize_or_zero(),weapon:0,alternate:false};
            }
        }
        let target=foes.iter().find(|t|Some(t.id)==plan.target).copied()?;
        keys.weapon=Some(plan.weapon as u8);keys.primary=!plan.alternate;keys.alternate=plan.alternate;
        keys.yaw=plan.aim.y.atan2(plan.aim.x);keys.pitch=plan.aim.z.asin().clamp(-1.56,1.56);
        Some(target)
    }
}

pub(crate) fn combat_dodge(world:&World, player:&Player, home:Vec3, enemy:Vec3, tick:u64, lead:f32,
    other:Option<Vec3>, melee:bool, safe:impl Fn(Vec3)->bool)->Vec2 {
    if !player.grounded || player.swimming || player.rope.is_some() {return Vec2::ZERO;}
    let delta=(enemy-player.eye()).truncate();let toward=delta.normalize_or_zero();
    let across=vec2(-toward.y,toward.x)*if (tick/96)%2==0 {1.} else {-1.};
    let radial=if melee {(delta.length()-60.).mul_add(1./120.,0.).clamp(-0.5,1.)}
        else if delta.length()>300. {0.6} else if delta.length()<120. {-0.5} else {0.};
    let retreat=(home-player.feet).truncate().normalize_or_zero();
    let wish=if (player.feet-home).truncate().length()>120. {retreat}
        else {(across*if melee {0.3} else {0.9}+toward*radial).normalize_or_zero()};
    let leash=160_f32.max((player.feet-home).truncate().length());
    let delay=(lead.clamp(0.,1.5)*120.).ceil() as u32;
    // Remote snapshots describe an earlier body. Cover its in-flight motion,
    // the time a new input can remain unseen, and its ordinary braking tail.
    // Shorter analogue sidesteps remain available on a narrow supported strip.
    let pending=player.velocity.truncate()/320.;
    for wish in [wish,-wish,retreat,wish*0.35,-wish*0.35,retreat*0.35] {
        if wish==Vec2::ZERO {continue;}
        let mut trial=player.clone();
        let clear=(0..72+2*delay).all(|step| {
            let input=if step<delay {pending.clamp_length_max(1.)}
                else if step<24+2*delay {wish} else {Vec2::ZERO};
            trial.tick(world,Controls{wish:input,run:true,..Default::default()});
            // A body can remain grounded with only its outer edge on a ledge.
            // Keep supported room beneath the whole stance before committing
            // a delayed dodge, including its braking tail.
            trial.grounded && super::placement::supported(world,trial.feet)
                && trial.feet.z>=player.feet.z-12.
                && (trial.feet-home).truncate().length()<=leash && safe(trial.feet)
                && other.is_none_or(|at| (trial.feet-at).truncate().length()>=36. || (trial.feet.z-at.z).abs()>=56.)
        });
        if clear {return wish;}
    }
    Vec2::ZERO
}

pub struct Driver {
    path: PathBuf,
    recovery_trace: Option<super::recovery_trace::Trace>,
    command: Option<Command>,
    modified: Option<SystemTime>,
    polled: Instant,
    refreshed: Instant,
    used: Instant,
    door_requests: BTreeSet<crate::entity::Id>,
    door_context: Option<(u64, u64, u64, u64)>,
    worker: Option<Worker>,
    planning_hold: Option<Vec3>,
    segment_landing: Option<Vec3>,
    plan: VecDeque<(Controls, Vec3)>,
    ledge_goal: Option<Vec3>,
    landing_feedback: LandingFeedback,
    walking_plan: bool,
    walking_progress: WalkingProgress,
    step_tick: u64,
    step_at: Instant,
    navigation_clock: NavigationClock,
    jumped: bool,
    rope_used: bool,
    control_used: bool,
    switch_aim: SwitchAim,
    combat_aim: CombatAim,
    combat_home: Option<(u64,Vec3)>,
    combat_location: Option<(u64,u64,u64)>,
    step_height: Option<f32>,
    swim_goal: Option<(u64, Vec3)>,
    swim_worker: Option<SwimWorker>,
    garden_course: Option<crate::levels::garden1::NetworkDriver>,
    sanctum_course: Option<crate::levels::centipede2::NetworkDriver>,
    chess_course: Option<crate::levels::wchess1::NetworkDriver>,
    castling_course: Option<crate::levels::wchess2::NetworkDriver>,
    funhouse_exit: Option<crate::levels::funhouse::NetworkExit>,
    funhouse_pendulums: Option<crate::levels::funhouse::NetworkPendulums>,
    queen_finale: Option<crate::levels::qlair::NetworkPilot>,
    location: Option<(u64, u64, u64)>,
    pub report: Report,
}
impl Driver {
    fn observe_location(&mut self, location: (u64, u64, u64), tick: u64, now: Instant) {
        if self.location != Some(location) {
            // A relocation invalidates the path, not a jump already consumed
            // by this command. A new visit, command or planned segment rearms it.
            if self.location.is_some_and(|old| old.0 != location.0) { self.jumped=false; }
            self.plan.clear(); self.planning_hold=None; self.segment_landing=None; self.ledge_goal=None; self.landing_feedback=LandingFeedback::default(); self.location = Some(location);
            self.swim_goal = None;
            self.step_tick = tick; self.step_at = now;
            self.navigation_clock = NavigationClock::new(now);
        }
    }
    fn direct_jump(command: &Command, local: PlayerId, ticks: u64, elapsed: Duration,
        keys: &mut Input, jumped: &mut bool) -> bool {
        if let Some(runup) = command.runup.as_ref().filter(|r|
            !*jumped && r.active(local,ticks,elapsed)) {
            // Sample the short run-up beside the input, without an outer
            // report round-trip that could send a player off the pad.
            keys.wish=runup.wish; keys.swim=keys.wish.extend(0.);
            true
        } else {
            if command.jump && !*jumped { keys.jumps+=1; *jumped=true; }
            false
        }
    }
    pub fn record_input_event(&mut self, event:serde_json::Value) {
        self.report.input_events.push_back(event);
        while self.report.input_events.len()>32 { self.report.input_events.pop_front(); }
    }
    pub fn record_recovery(&mut self, roster:&super::world::Roster, arena:&Arena, world:&World,
        epoch:u64, map:&str, cinematic:bool) {
        if !self.command.as_ref().is_some_and(|c|c.recovery_trace) || self.report.recovery_trace_error.is_some() {return;}
        let result=(||->Result<()> {
            if self.recovery_trace.is_none() {
                self.recovery_trace=Some(super::recovery_trace::Trace::new(self.path.with_file_name("recovery-trace.jsonl"))?);
            }
            self.recovery_trace.as_mut().unwrap().record(roster,world,epoch,map,cinematic,
                arena.stage.interactive_actor().is_some(),|at|!arena.stage.interactions.hazardous(at))
        })();
        if let Err(error)=result {self.report.recovery_trace_error=Some(error.to_string());}
    }
    /// Compare actual sent/restored states, independent of the two report clocks.
    /// Only the opt-in local harness keeps this bounded diagnostic history.
    pub fn record_state(&mut self, frame: &Frame, arena: &Arena) -> Result<()> {
        use sha2::{Digest, Sha256};
        if let Some(course)=&mut self.funhouse_exit {
            course.record_state(frame,arena);self.report.funhouse_exit=Some(course.report());
        }
        if let Some(course)=&mut self.funhouse_pendulums {
            course.record_state(frame,arena);self.report.funhouse_pendulums=Some(course.report());
        }
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
            native_sha256:format!("{:x}",Sha256::digest(native)),
            movement:frame.players.each_ref().map(MovementEvidence::from) });
        while states.len() > 64 { states.pop_front(); }
        Ok(())
    }
    pub fn new(path: PathBuf) -> Self {
        let now = Instant::now();
        Self { path, recovery_trace:None, command: None, modified: None, polled: now, refreshed: now, used: now,
            door_requests: BTreeSet::new(), door_context: None,
            worker: None, planning_hold:None, segment_landing:None, plan: VecDeque::new(), ledge_goal:None, landing_feedback:LandingFeedback::default(), walking_plan:false, step_tick: 0, step_at: now,
            walking_progress:WalkingProgress::new(Vec3::ZERO,now),
            navigation_clock:NavigationClock::new(now), jumped: false, rope_used: false,
            control_used: false, switch_aim:SwitchAim::default(), combat_aim:CombatAim::default(),combat_home:None,combat_location:None,
            step_height: None, swim_goal: None, swim_worker: None,
            garden_course: None, sanctum_course: None, chess_course: None, castling_course: None, funhouse_exit:None, funhouse_pendulums:None, queen_finale:None, location: None, report: Report::default() }
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
                if self.command.as_ref().is_none_or(|old|old.castling_follow!=command.castling_follow || old.epoch!=command.epoch) {
                    self.castling_course=None;self.report.castling_course=None;
                }
                if self.command.as_ref().is_none_or(|old|old.chess_follow!=command.chess_follow || old.epoch!=command.epoch) {
                    self.chess_course=None;self.report.chess_course=None;
                }
                if self.command.as_ref().is_none_or(|old| old.id != command.id || old.shoot_switch!=command.shoot_switch) {
                    self.queen_finale=None;
                    self.funhouse_exit=None;self.report.funhouse_exit=None;
                    self.funhouse_pendulums=None;self.report.funhouse_pendulums=None;
                    self.plan.clear(); self.planning_hold=None; self.segment_landing=None; self.ledge_goal=None; self.landing_feedback=LandingFeedback::default(); self.jumped = false; self.rope_used = false; self.location = None;
                    self.control_used = false;
                    self.switch_aim=SwitchAim::default();
                    self.step_height = None;
                    self.swim_goal = None;
                    self.garden_course = None; self.report.garden_course = None;
                    self.sanctum_course = None; self.report.sanctum_course = None;
                    self.report.error = None;
                }
                self.report.command = Some(command.id);
                self.command = Some(command); self.modified = Some(modified); self.refreshed = now;
            }
            Ok(None) => (),
            Err(error) => { self.report.error = Some(error.to_string()); self.command = None; self.plan.clear(); self.planning_hold=None; self.segment_landing=None; }
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
    pub fn checkpoint_result(&mut self, command: u64, epoch: u64, tick: u64, elapsed: Duration, error: Option<String>) {
        self.report.checkpoint = Some(CheckpointReport { command, epoch, tick, save_ms: elapsed.as_secs_f32()*1000., error });
    }
    fn neutral(keys: &mut Input) {
        keys.wish = Vec2::ZERO; keys.swim = Vec3::ZERO; keys.rise = 0.;
        keys.run = false; keys.primary = false; keys.alternate = false; keys.weapon = None;
        keys.skip_scene = false;
    }
    // Observe after simulation, alongside the frame being written. Input sampling
    // happens before a host step and can otherwise label a finished scene with
    // its previous name but the new (empty) owner.
    pub fn observe(&mut self, frame: &Frame, arena: &Arena, world: &World) {
        self.report.observed_epoch = frame.epoch;
        self.report.observed_tick = frame.tick;
        self.report.curiosity_platform = arena.stage.interactions.levels.iter().find_map(|s|
            crate::levels::jlair1::route_platform(s.ctl.as_ref(), frame.players.each_ref().map(|b| &b.player)));
        self.report.surface = self.command.as_ref()
            .filter(|c| c.epoch == frame.epoch && self.report.command == Some(c.id))
            .and_then(|c| Some((c.surface?, c.goal?)))
            .and_then(|(id, goal)| arena.stage.interactions.levels.iter().find_map(|s|
                crate::levels::funhouse::route_surface(s.ctl.as_ref(), id, goal,
                    frame.players.each_ref().map(|b| &b.player))));
        self.report.beyond_steps = arena.stage.interactions.beyond.as_ref()
            .map(|b|std::array::from_fn(|i|b.step_goal(i+1)));
        self.report.clear = frame.players.each_ref().map(|b| !b.stats.alive() || world.body_clear(b.player.feet));
        self.report.ledge_pull_clear=frame.players.each_ref().map(|b|
            b.player.ledge.as_ref().is_some_and(|hang|hang.pull_clear(world)));
        self.report.return_landings=Default::default();
        if let Some(goal)=self.command.as_ref().filter(|c|c.epoch==frame.epoch && c.landing_feedback).and_then(|c|c.goal) {
            self.report.return_landings=frame.players.each_ref().map(|b| {
                if b.stats.alive() && b.player.grounded && !arena.stage.busy() {
                    return_landings(world,b.player.feet,goal,|at|!arena.stage.interactions.hazardous(at))
                } else {Vec::new()}
            });
        }
        self.report.native_threats = frame.players.each_ref().map(|b|
            native_threat(world,&arena.stage.interactions,arena.npcs.targets(),b));
        self.report.neighbors = frame.players.each_ref().map(|b| {
            if b.stats.alive() && b.player.grounded && !arena.stage.busy() {
                super::placement::neighbors(world,b.player.feet,|at| !arena.stage.interactions.hazardous(at))
            } else { Vec::new() }
        });
        self.report.switch_approaches=Default::default();
        if let Some(target)=self.command.as_ref()
            .filter(|c|c.epoch==frame.epoch && self.report.command==Some(c.id))
            .and_then(|c|c.shoot_switch.as_deref())
            .and_then(|thread|arena.stage.interactions.switch_target(thread)) {
            // Queries run with the retained report, not on every physics tick.
            // Only existing safe neighbors are considered; movement still uses
            // ordinary client controls and must earn its own arrival receipt.
            let targets=combat_targets(&arena.stage.interactions,arena.npcs.targets());
            self.report.switch_approaches=std::array::from_fn(|slot| {
                let mut targets=targets.clone();let other=&frame.players[1-slot];
                if other.stats.alive() {targets.push(crate::combat::Target {
                    id:if slot==0 {crate::combat::COOP_RED} else {crate::combat::COOP_BLUE},
                    center:other.player.feet+crate::collision::PLAYER_CENTER,half:crate::collision::PLAYER_HALF});}
                visible_switch_approaches(&crate::combat::Context{world,targets:&targets},target,
                    &self.report.neighbors[slot])
            });
        }
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
            .or_else(|| arena.stage.interactions.beyond.as_ref().and_then(|b|b.scene_id()))
            .or_else(|| arena.stage.interactions.levels.iter()
                .find_map(|s| s.ctl.scene_id())).map(str::to_owned);
        self.report.speech = arena.stage.story.busy() && !arena.stage.busy();
        self.report.snarks = arena.npcs.snarks().map(|(id, snark, active)| {
            let center = snark.target(id).center;
            EnemyReport {
                id, name:snark.kind.model().into(), active, feet:snark.feet,
                health:snark.health, state:snark.phase,
                selected_player:snark.opponents.selected_player,
                distance:frame.players.each_ref().map(|b|center.distance(b.player.eye())),
                line_of_sight:frame.players.each_ref().map(|b| {
                    let trace=world.sweep(center,b.player.eye(),Vec3::splat(0.5));
                    b.stats.alive() && !b.stats.ignores_alice() && !trace.start_solid && trace.fraction>=1.
                }),
            }
        }).collect();
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
        self.report.boojums.extend(arena.npcs.route_boojums().map(|(id, boojum, active)| {
            let center=boojum.target(id).center;
            EnemyReport {
                id, name:format!("resident_boojum_{id}"), active,
                feet:boojum.feet, health:boojum.health, state:boojum.state,
                selected_player:boojum.opponents.selected_player,
                distance:frame.players.each_ref().map(|b|center.distance(b.player.eye())),
                line_of_sight:frame.players.each_ref().map(|b| {
                    let trace=world.sweep(center,b.player.eye(),Vec3::splat(0.5));
                    b.stats.alive() && !b.stats.ignores_alice() && !trace.start_solid && trace.fraction>=1.
                }),
            }
        }));
        if let Some(school) = &arena.stage.interactions.school2 {
            let (guards, boojums) = school2_reports(school, frame, world);
            self.report.guards.extend(guards);
            self.report.boojums.extend(boojums);
        }
    }
    pub fn sample(&mut self, frame: &Frame, arena: &Arena, world: &World, map: &crate::bsp::Bsp, local: PlayerId, keys: &mut Input, now: Instant) {
        Self::neutral(keys);
        self.report.reached = false;
        self.report.pathnode = None;
        self.report.carry_velocity = None;
        self.report.expected = self.plan.front().map(|(_, at)| *at);
        self.report.plan_remaining = self.plan.len();
        self.report.jumps = keys.jumps;
        self.report.phase = "idle";
        let body = &frame.players[local.index()];
        self.observe_doors((frame.epoch, frame.checkpoint_generation, body.warped_at, body.spawned_at),
            &arena.stage.interactions);
        let location = (frame.epoch, body.warped_at, body.spawned_at);
        // Ordinary waypoint commands reset path planning, but must not erase
        // the velocity observations used to lead the next thrown Blade.
        if self.combat_location!=Some(location) {
            self.combat_aim=Default::default();self.combat_home=None;self.combat_location=Some(location);
        }
        self.observe_location(location, frame.tick, now);
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
                        Ok(mut plan) => {
                            if plan.walking && self.command.as_ref().is_some_and(|c|c.run) {
                                plan.steps=straight_grounded_sections(worker.start,plan.steps);
                            }
                            let trace = serde_json::json!({ "build": env!("LOOKING_GLASS_BUILD"),
                                "epoch": worker.epoch, "command": worker.id, "start": worker.start,
                                "walking":plan.walking,"ledge_goal":plan.ledge_goal,
                                "steps": plan.steps.iter().map(|(input, expected)|serde_json::json!({
                                    "wish": input.wish, "jump": input.jump, "run": input.run, "expected": expected
                                })).collect::<Vec<_>>() });
                            if let Ok(bytes) = serde_json::to_vec(&trace) {
                                let _ = std::fs::write(crate::preferences::path(&format!("route-plan-{}-{}.json",worker.id,self.report.replans)),bytes);
                            }
                            self.walking_plan=plan.walking;
                            self.segment_landing=if !plan.walking && self.command.as_ref().is_some_and(|c|c.settle_landings) {
                                plan.steps.last().map(|s|s.1)
                            } else {None};
                            self.ledge_goal=plan.ledge_goal;
                            self.landing_feedback.start(worker.start);
                            self.planning_hold=None;
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
            self.planning_hold=None; self.report.phase = "waiting for command"; return;
        };
        if !body.stats.alive() {
            if let Some(course)=&mut self.garden_course {
                course.spectate(frame.tick);self.report.garden_course=Some(course.report());
            }
            if let Some(course)=&mut self.castling_course {
                course.spectate(frame.tick);self.report.castling_course=Some(course.report());
            }
            if let Some(course)=&mut self.chess_course {
                course.spectate(frame.tick);self.report.chess_course=Some(course.report());
            }
            self.report.phase = "spectating"; return;
        }
        if command.checkpoint { self.report.phase = "saving checkpoint"; return; }
        if let Some(id)=command.pathnode {
            let result=map.entities.get(id).context("Missing pathnode").and_then(|e|pathnode_goal(world,e));
            match result {
                Ok((origin,feet))=>self.report.pathnode=Some(PathnodeReport {
                    command:command.id,epoch:frame.epoch,tick:frame.tick,entity:id,origin,feet,
                }),
                Err(error)=>self.report.error=Some(error.to_string()),
            }
            self.report.phase="observing pathnode";return;
        }
        keys.skip_scene = command.skip_scene;
        if !command.watch_dialogue && arena.stage.story.busy() && arena.stage.owner == Some(local)
            && now.duration_since(self.used) >= Duration::from_millis(250) {
            keys.uses += 1; self.used = now;
        }
        if let Some(first) = command.castling_follow {
            let result=(|| -> Result<()> {
                if self.castling_course.is_none() {self.castling_course=Some(crate::levels::wchess2::NetworkDriver::new()?);}
                let course=self.castling_course.as_mut().unwrap();
                course.sample(command.castling_step.unwrap(),frame,arena,world,local,first,command.input_lead,keys)?;
                self.report.reached=course.reached();self.report.castling_course=Some(course.report());Ok(())
            })();
            if let Err(error)=result {self.report.error=Some(error.to_string());}
            self.report.phase=if arena.stage.busy(){"scene"}else{"following Castling course"};
            return;
        }
        if let Some(first) = command.chess_follow {
            let result=(|| -> Result<()> {
                if self.chess_course.is_none() {self.chess_course=Some(crate::levels::wchess1::NetworkDriver::new()?);}
                let course=self.chess_course.as_mut().unwrap();
                course.sample(command.chess_step.unwrap(),frame,arena,world,local,first,command.input_lead,keys)?;
                self.report.reached=course.reached();self.report.chess_course=Some(course.report());Ok(())
            })();
            if let Err(error)=result {self.report.error=Some(error.to_string());}
            self.report.phase=if arena.stage.busy(){"scene"}else{"following Pale Realm course"};
            return;
        }
        if let Some(first) = command.sanctum_follow {
            let result=(|| -> Result<()> {
                if self.sanctum_course.is_none() {self.sanctum_course=Some(crate::levels::centipede2::NetworkDriver::new()?);}
                let course=self.sanctum_course.as_mut().unwrap();
                course.sample(frame,arena,local,first,command.input_lead,keys)?;
                self.report.sanctum_course=Some(course.report());Ok(())
            })();
            if let Err(error)=result {self.report.error=Some(error.to_string());}
            self.report.phase=if arena.stage.busy() {"scene"} else {"following Sanctum course"};
            return;
        }
        if let Some(first) = command.garden_follow {
            let result = (|| -> Result<()> {
                if self.garden_course.is_none() {
                    self.garden_course = Some(crate::levels::garden1::NetworkDriver::new(command.garden_start.unwrap_or(0))?);
                }
                let course = self.garden_course.as_mut().unwrap();
                course.sample(frame, arena, world, local, first, command.input_lead, keys)?;
                self.report.garden_course = Some(course.report());
                Ok(())
            })();
            if let Err(error) = result { self.report.error = Some(error.to_string()); }
            self.report.phase = if arena.stage.busy() { "scene" } else { "following Garden course" };
            return;
        }
        if arena.stage.busy() {
            self.plan.clear(); self.planning_hold=None; self.segment_landing=None; self.ledge_goal=None; self.landing_feedback=LandingFeedback::default(); self.jumped = false; self.report.phase = "scene";
            self.swim_goal = None;
            return;
        }
        if command.use_watch {
            keys.weapon=Some(9);keys.primary=true;keys.alternate=false;
            self.report.phase="using watch";
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
        if let Some(thread)=&command.shoot_switch {
            let target=arena.stage.interactions.switch_target(thread);
            self.report.expected=target.map(|t|t.center);
            self.report.phase="shooting switch";
            if body.stats.copies(0)==0 {
                self.report.error=Some("Shooting a switch requires an owned Blade".into());return;
            }
            let targets=combat_targets(&arena.stage.interactions,arena.npcs.targets());
            let context=crate::combat::Context{world,targets:&targets};
            match self.switch_aim.sample(&context,target,body.player.eye(),keys) {
                Ok(reached)=>{self.report.reached=reached;if reached {self.report.phase="switch completed";}},
                Err(error)=>self.report.error=Some(error.to_string()),
            }
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
        }
        let queen_goal=if command.queen_finale {
            let pilot=self.queen_finale.get_or_insert_with(Default::default);
            match pilot.sample(frame,arena,world,local,keys) {
                Ok(Some(goal))=>Some(goal),
                Ok(None)=>{self.report.phase="Queen finale combat";self.report.reached=true;return;},
                Err(error)=>{self.report.error=Some(error.to_string());return;},
            }
        }else{None};
        if command.attack && !command.queen_finale {
            if command.combat_tactics && body.stats.copies(0)>0 {
                let interactions=&arena.stage.interactions;
                let npcs=arena.npcs.targets();
                let mut targets=combat_targets(interactions,npcs.clone());
                let switches=if command.avoid_switches {interactions.shot_targets()} else {Vec::new()};
                let foes=targets.iter().filter(|t|
                    crate::level::hit_owner(&interactions.levels,t.id)
                        .is_none_or(|slot|interactions.levels[slot].ctl.hostile_target(t.id))
                    && !self.report.guards.iter().any(|g|g.id==t.id && g.state==crate::combat::State::Idle)
                    && (!npcs.iter().any(|n|n.id==t.id) || arena.npcs.route_engaged(t.id,body.player.eye()))
                    && !switches.iter().any(|s|s.id==t.id)).copied().collect::<Vec<_>>();
                let other=&frame.players[1-local.index()];
                if other.stats.alive() {
                    targets.push(crate::combat::Target{id:if local.index()==0 {crate::combat::COOP_RED} else {crate::combat::COOP_BLUE},
                        center:other.player.feet+crate::collision::PLAYER_CENTER,half:crate::collision::PLAYER_HALF});
                }
                let context=crate::combat::Context{world,targets:&targets};
                let cards=command.prefer_cards && body.stats.copies(1)>0 && body.stats.will()>=crate::route::fight::CARD_COST;
                let normal=self.combat_aim.sample((frame.epoch,frame.tick),&context,&foes,body.player.eye(),cards,keys);
                let target=ice_aim(&context,&foes,body,command.prefer_ice,keys).or(normal);
                if let Some(target)=target {
                    if command.goal.is_none() && command.marker.is_none() && command.rope.is_none()
                        && command.use_aim.is_none() && command.beyond_step.is_none() && !command.funhouse_exit && !command.funhouse_pendulums {
                        if self.combat_home.is_none_or(|(id,_)|id!=command.id) {self.combat_home=Some((command.id,body.player.feet));}
                        keys.wish=combat_dodge(world,&body.player,self.combat_home.unwrap().1,target.center,frame.tick,command.input_lead,
                            other.stats.alive().then_some(other.player.feet),keys.weapon==Some(0)&&keys.primary,
                            |at|!interactions.hazardous(at));
                        keys.run=keys.wish!=Vec2::ZERO;
                    }
                }
            } else {
                combat_controls(world,&arena.stage.interactions,arena.npcs.targets(),body,command.prefer_cards,command.avoid_switches,
                    |id|self.report.guards.iter().any(|g|g.id==id && g.state==crate::combat::State::Idle),keys);
            }
        }
        // Rope movement can be combined with ordinary weapon input. Apply aim
        // after movement-facing so a swing does not redirect an intended shot.
        if command.rope.is_some() { return; }
        if command.funhouse_pendulums {
            let result=(|| -> Result<()> {
                if self.funhouse_pendulums.is_none() { self.funhouse_pendulums=Some(crate::levels::funhouse::NetworkPendulums::new(frame,arena,local)?); }
                let course=self.funhouse_pendulums.as_mut().unwrap();
                course.sample(frame,arena,command.input_lead,keys)?;
                let report=course.report();self.report.reached=report.reached;
                self.report.funhouse_pendulums=Some(report);Ok(())
            })();
            if let Err(error)=result {self.report.error=Some(error.to_string());}
            self.report.phase="following Funhouse pendulums";
            return;
        }
        if command.funhouse_exit {
            let result=(|| -> Result<()> {
                if self.funhouse_exit.is_none() { self.funhouse_exit=Some(crate::levels::funhouse::NetworkExit::new(frame,arena,local)?); }
                let course=self.funhouse_exit.as_mut().unwrap();
                course.sample(frame,arena,local,command.input_lead,keys)?;
                let report=course.report();self.report.reached=report.reached;
                self.report.funhouse_exit=Some(report);Ok(())
            })();
            if let Err(error)=result {self.report.error=Some(error.to_string());}
            self.report.phase="following Funhouse upper exit";
            return;
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
        } else { queen_goal.or(command.goal) };
        let Some(goal) = goal else { return; };
        let surface = command.surface.and_then(|id| arena.stage.interactions.levels.iter().find_map(|s|
            crate::levels::funhouse::route_surface(s.ctl.as_ref(), id, goal,
                frame.players.each_ref().map(|b| &b.player))));
        if command.surface.is_some() && surface.is_none() {
            self.report.error = Some("No support beneath the requested surface goal".into());
            return;
        }
        let goal = surface.as_ref().map_or(goal, |s| s.goal);
        let delta = goal-body.player.feet;
        if command.navigate && command.landing_feedback && !self.walking_plan
            && self.landing_feedback.misplaced(&body.player,self.plan.front().map(|s|s.1)) {
            self.plan.clear();self.planning_hold=None;self.segment_landing=None;self.ledge_goal=None;
            self.landing_feedback=LandingFeedback::default();
            self.report.landing_replans+=1;
            self.report.last_landing_replan=Some((frame.tick,body.player.feet));
        }
        let carry=(!command.swim && !command.jump).then(||arena.stage.interactions.levels.iter()
            .find_map(|s|s.ctl.route_carry_velocity(&body.player))).flatten();
        self.report.carry_velocity=carry;
        if !self.plan.is_empty() {self.planning_hold=None;}
        if command.hold_goal {
            keys.wish=carry.map_or_else(||delayed_hold_wish(delta.truncate(),body.player.velocity.truncate(),command.input_lead),
                |v|carried_goal_control(&body.player,goal,v,command.input_lead,command.arrival_radius).0);keys.run=true;
            if !keys.primary && !keys.alternate {
                keys.yaw=keys.wish.y.atan2(keys.wish.x);keys.pitch=0.;
            }
            self.report.expected=Some(goal);self.report.phase="holding toward goal";
            return;
        }
        // A nearby supported intermediate island can still require another
        // jump. Settling toward the final goal would erase that queued takeoff.
        // A delayed run can land directly on the destination shelf instead of
        // catching the planned ledge. Its queued pull-up would then become an
        // ordinary jump off that shelf. Cancel it only after actual grounding
        // with a clear, continuously supported walk to the final goal. A gap
        // or a different floor still requires the pending takeoff.
        let can_land=command.navigate && (!pending_jump(self.plan.iter().map(|(input,_)|input.jump),self.jumped)
            || command.settle_landings && supported_goal_walk(world,&body.player,goal));
        let landing = if can_land {
            landing_control(world,&body.player,goal,command.input_lead,command.arrival_radius)
        } else { None };
        let intermediate=if can_land && command.settle_landings && landing.is_none() {
            self.segment_landing.and_then(|at|landing_control(world,&body.player,at,command.input_lead,command.arrival_radius))
        } else {None};
        let carried=carry.filter(|_|!command.navigate || landing.is_some())
            .map(|v|carried_goal_control(&body.player,goal,v,command.input_lead,command.arrival_radius));
        self.report.reached = if let Some((_,settled))=carried { settled }
            else if command.navigate { landing.is_some_and(|(_,settled)|settled) }
            else if command.swim { delta.length()<command.arrival_radius }
            else { delta.truncate().length()<command.arrival_radius && (command.horizontal || delta.z.abs()<35.) };
        self.report.reached &= surface.as_ref().is_none_or(|s| s.supported[local.index()]);
        if self.report.reached {
            if let Some((wish,_))=carried { keys.wish=wish;keys.swim=wish.extend(0.);keys.run=true; }
            self.report.phase="reached"; self.plan.clear(); self.segment_landing=None; return;
        }
        if command.swim {
            if !body.player.swimming {
                self.report.error=Some("Explicit swim goal requires a swimming player".into()); return;
            }
            let steering=delta-body.player.velocity*command.input_lead;
            keys.swim=steering.normalize_or_zero()*(steering.length()/30.).min(1.);
            keys.run=command.run;
            if !keys.primary && !keys.alternate && keys.swim.length_squared()>0.01 {
                keys.yaw=keys.swim.y.atan2(keys.swim.x);
                keys.pitch=(keys.swim.z/keys.swim.length()).asin().clamp(-1.56,1.56);
            }
            self.report.expected=Some(goal); self.report.phase="swimming to goal";
            return;
        }
        if let Some((wish,_)) = carried {
            self.report.phase="settling carried landing";
            self.plan.clear();keys.wish=wish;keys.swim=wish.extend(0.);keys.run=true;
        } else if let Some((wish,_)) = landing {
            self.report.phase="settling landing";
            self.plan.clear();self.segment_landing=None;
            keys.wish=wish; keys.swim=wish.extend(0.); keys.run=false;
        } else if let Some((wish,settled))=intermediate {
            // A shortened plan can end on a lower ledge, far from the final
            // waypoint. Retain that actual planned landing after clearing the
            // timed inputs, and brake there before the next search. Settling
            // this intermediate support never reports the final goal reached.
            self.report.phase="settling intermediate landing";
            self.report.expected=self.segment_landing;
            self.plan.clear();keys.wish=wish;keys.swim=wish.extend(0.);keys.run=false;
            if settled {self.segment_landing=None;}
        } else if command.navigate && self.walking_plan && !self.plan.is_empty() {
            self.report.phase="following walking route";
            // A swinging obstacle can stop against Alice and never clear the
            // old path. Plan around its current pose after a second without
            // progress instead of pushing into it indefinitely.
            if self.walking_progress.obstructed(&body.player,now) {
                self.plan.clear();
                self.report.phase="replanning obstructed walk";
            } else {
                while self.plan.front().is_some_and(|(_,point)|
                    grounded_waypoint_ready(world,&body.player,*point,self.plan.get(1).map(|s|s.1),command.run)) {
                    self.plan.pop_front();
                }
                if let Some((_,point))=self.plan.front() {
                    keys.wish=carry.map_or_else(||walking_control(&body.player,*point,command.input_lead,command.run).0,
                        |v|carried_goal_control(&body.player,*point,v,command.input_lead,8.).0);
                    keys.swim=keys.wish.extend(0.);
                    // Opt-in ordinary running on grounded routes. Timed jumps
                    // retain their planned speed; final landings still walk.
                    keys.run=command.run || carry.is_some();
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
                self.landing_feedback.advance(expected);
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
            } else {
                let (wish,settled)=planning_hold_control(&mut self.planning_hold,&body.player,carry,command.input_lead);
                let handoff=command.ledge_handoff.then(||ledge_handoff_wish(world,&body.player,goal,self.ledge_goal,command.input_lead)).flatten();
                keys.wish=handoff.unwrap_or(wish);keys.swim=keys.wish.extend(0.);keys.run=carry.is_some() || handoff.is_some();
                self.report.expected=self.planning_hold;
                if carry.is_some() {self.report.phase="settling planning start";}
                if self.worker.is_none() && self.report.error.is_none() && settled
                    && !(command.ledge_handoff && body.player.ledge.is_some()) {
                    let (send,result)=mpsc::channel();
                    let world=world.clone(); let player:Player=body.player.clone();
                    self.worker=Some(Worker { id:command.id, epoch:frame.epoch, warped:body.warped_at, start:player.feet,result });
                    self.report.replans+=1;
                    // Remote segments can run longer when updates coalesce or
                    // arrive with jitter. Avoid plans that jump at the last safe
                    // physics tick of an edge; leave 100 ms of support in reserve.
                    let margin = if local == PlayerId::Red { 12 } else { 0 };
                    let ledge_handoff=command.ledge_handoff;
                    let avoid_liquid_damage=command.avoid_liquid_damage;
                    let settle_landings=command.settle_landings;
                    std::thread::spawn(move || {
                        let search=|walking,tolerance| {
                            if avoid_liquid_damage {Route::plan_without_liquid_damage(&world,&player,goal,tolerance,margin,walking)}
                            else if walking {Route::plan_grounded(&world,&player,goal,tolerance)}
                            else {Route::plan_with_takeoff_margin(&world,&player,goal,tolerance,margin)}
                        };
                        let plan=search(true,22.)
                        .map(|steps|PlannedRoute {walking:true,steps,ledge_goal:None})
                        .or_else(|_|search(false,22.)
                            .or_else(|_|search(false,40.))
                            .map(|steps| {
                                let steps=if settle_landings {through_first_landing(&world,&player,steps)} else {steps};
                                let (steps,ledge_goal)=if ledge_handoff {stop_at_ledge(&world,&player,goal,steps)} else {(steps,None)};
                                PlannedRoute {walking:plan_stays_grounded(&world,&player,&steps),steps,ledge_goal}
                            }));
                        let _=send.send(plan); });
                }
            }
            if self.worker.is_some() { self.report.phase="planning"; }
        } else {
            self.report.phase="walking";
            keys.wish=direct_goal_wish(world,&body.player,goal,
                command.input_lead,command.run,command.speed_limit);
            keys.swim=keys.wish.extend(0.);
            keys.run=command.run;
            if Self::direct_jump(command,local,frame.tick.saturating_sub(self.step_tick),
                now.duration_since(self.step_at),keys,&mut self.jumped) {
                self.report.phase="running approach";
            }
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
    #[ignore = "requires LOOKING_GLASS_TEST_DATA"]
    fn recorded_curiosity_takeoff_probe() {
        let base=std::env::var_os("LOOKING_GLASS_TEST_DATA").unwrap();
        let mut assets=crate::assets::Assets::open(std::path::Path::new(&base)).unwrap();
        let map=crate::bsp::Bsp::parse(&assets.read("maps/jlair1.bsp").unwrap()).unwrap();
        let world=World::from_bsp(&map).unwrap();
        let mut start=Player::new(vec3(-6123.7124,2723.22,1984.03125));start.grounded=true;
        for legacy in [true,false] {
            let mut player=start.clone();
            let now=Instant::now();let mut clock=NavigationClock::new(now);
            let mut old_elapsed=Duration::ZERO;let mut sampled=now;
            let mut issued=false;let mut takeoff=None;
            for tick in 0..60 {
                let mut jump=false;
                // Hold the first sampled run input across a 300 ms guest
                // frame, then sample again after 100 ms. The host keeps ticking.
                if [36,48].contains(&tick) && !issued {
                    let at=now+Duration::from_micros(tick*1_000_000/120);
                    let advance=if legacy {
                        old_elapsed+=(at-sampled).min(Duration::from_millis(100));
                        sampled=at;old_elapsed>=Duration::from_millis(200)
                    } else {clock.advance(at)};
                    if advance {jump=true;issued=true;takeoff=Some((tick,player.feet,player.grounded));}
                }
                player.tick(&world,Controls{wish:if issued {vec2(0.70710677,0.70710677)} else {Vec2::Y},run:true,jump,..Default::default()});
            }
            let (tick,feet,grounded)=takeoff.unwrap();
            println!("Curiosity legacy{legacy} takeoff tick{tick} {feet:?} grounded{grounded}, actual jumps{}",player.jumps);
            if legacy {assert_eq!(tick,48);assert!(!grounded);assert_eq!(player.jumps,0);}
            else {assert_eq!(tick,36);assert!(grounded);assert_eq!(player.jumps,1);}
        }
    }
    #[test]
    fn planning_waits_for_an_active_ledge_pull_even_at_zero_velocity() {
        let mut player=Player::new(vec3(-6182.801,-785.8058,1552.1563));
        player.grounded=false;player.velocity=Vec3::ZERO;
        player.ledge=Some(serde_json::from_value(serde_json::json!({
            "start":[-6161.5894,-798.5328,1491.1563],"top":1552.03125,
            "forward":[-0.8574929,0.51449573],"elapsed":2.1,"pulling":true,
            "side":0.,"forward_held":false
        })).unwrap());
        let mut anchor=Some(player.feet);
        assert_eq!(planning_hold_control(&mut anchor,&player,None,0.),(Vec2::ZERO,false));
        assert!(anchor.is_none());
        player.grounded=true; // A stale contact flag cannot finish the pull.
        assert!(!planning_hold_control(&mut anchor,&player,None,0.).1);
        player.grounded=false;player.ledge.as_mut().unwrap().pulling=false;
        assert!(planning_hold_control(&mut anchor,&player,None,0.).1,"A stationary hang can still plan its pull input");
        player.ledge=None;player.grounded=true;
        assert!(planning_hold_control(&mut anchor,&player,None,0.).1,"A completed pull can plan onward");
    }
    #[test]
    fn remote_route_finishes_its_first_landing_before_planning_later_jumps() {
        let world=World::fixture(&[(vec3(-100.,-100.,-40.),vec3(1600.,100.,0.))]);
        let start=Player::spawn(&world,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let mut player=start.clone();let mut steps=Vec::new();
        for index in 0..13 {
            let input=Controls{wish:Vec2::X,run:true,jump:index==0 || index==6,..Default::default()};
            for tick in 0..24 {player.tick(&world,Controls{jump:input.jump && tick==0,..input});}
            steps.push((input,player.feet));
        }
        assert_eq!(player.jumps,start.jumps+2);
        let original_len=steps.len();
        let shortened=through_first_landing(&world,&start,steps.clone());
        assert!(shortened.len()<7 && shortened.len()<original_len);
        let mut landed=start.clone();
        for (index,(input,expected)) in shortened.iter().enumerate() {
            assert_eq!(*expected,steps[index].1);
            assert_eq!(input.wish,steps[index].0.wish);
            assert_eq!(input.jump,steps[index].0.jump);
            for tick in 0..24 {landed.tick(&world,Controls{jump:input.jump && tick==0,..*input});}
        }
        assert!(landed.grounded && landed.jumps==start.jumps+1);
        assert_eq!(landed.feet,shortened.last().unwrap().1);
        let walking=vec![(Controls{wish:Vec2::X,..Default::default()},Vec3::ZERO);8];
        assert_eq!(through_first_landing(&world,&start,walking).len(),8);
    }
    #[test]
    fn remote_route_walks_its_grounded_turn_before_planning_takeoff() {
        let world=World::fixture(&[(vec3(-300.,-300.,-40.),vec3(600.,600.,0.))]);
        let start=Player::spawn(&world,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let mut player=start.clone();let mut steps=Vec::new();
        for index in 0..9 {
            let input=Controls{wish:if index==0 {Vec2::Y} else {Vec2::X},
                run:true,jump:index==2,..Default::default()};
            for tick in 0..24 {player.tick(&world,Controls{jump:input.jump && tick==0,..input});}
            steps.push((input,player.feet));
        }
        assert_eq!(player.jumps,start.jumps+1);
        let approach=through_first_landing(&world,&start,steps.clone());
        assert_eq!(approach.len(),2);
        assert!(plan_stays_grounded(&world,&start,&approach));
        assert!(super::super::placement::supported(&world,approach.last().unwrap().1));
        assert_eq!(approach.last().unwrap().1,steps[1].1);
        assert!(approach.iter().all(|s|!s.0.jump));
        // Starting the remaining sequence at its actual takeoff must retain
        // the jump and an ordinary landing, rather than return an empty plan.
        let mut takeoff=start.clone();
        for (input,_) in &approach {for _ in 0..24 {takeoff.tick(&world,*input);}}
        let flight=through_first_landing(&world,&takeoff,steps[2..].to_vec());
        assert!(flight[0].0.jump && flight.len()>1);
        for (input,_) in &flight {
            for tick in 0..24 {takeoff.tick(&world,Controls{jump:input.jump && tick==0,..*input});}
        }
        assert!(takeoff.grounded && takeoff.jumps==start.jumps+1);
    }
    #[test]
    fn remote_route_preserves_a_runup_that_temporarily_moves_away_from_its_goal() {
        let world=World::fixture(&[(vec3(-400.,-100.,-40.),vec3(1000.,100.,0.))]);
        let start=Player::spawn(&world,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let mut player=start.clone();let mut steps=Vec::new();
        for index in 0..9 {
            let input=Controls{wish:if index==0 {-Vec2::X} else {Vec2::X},
                run:true,jump:index==2,..Default::default()};
            for tick in 0..24 {player.tick(&world,Controls{jump:input.jump&&tick==0,..input});}
            steps.push((input,player.feet));
        }
        let destination=steps.last().unwrap().1;
        assert!(steps[1].1.distance(destination)>=start.feet.distance(destination),
            "Fixture must build velocity while temporarily moving away from its destination");
        assert!(crate::route::supported_footing(&world,steps[1].1));
        let shortened=through_first_landing(&world,&start,steps);
        assert!(shortened.len()>2 && shortened[2].0.jump,"Run-up was stopped before its takeoff");
        let mut actual=start.clone();
        for (input,_) in shortened {
            for tick in 0..24 {actual.tick(&world,Controls{jump:input.jump&&tick==0,..input});}
        }
        assert!(actual.grounded && actual.jumps==start.jumps+1
            && actual.feet.distance(destination)<start.feet.distance(destination));
    }

    #[test]
    fn remote_takeoff_can_settle_on_an_ordinary_narrow_walkway() {
        let world=World::fixture(&[(vec3(-100.,-12.,-40.),vec3(600.,12.,0.))]);
        let start=Player::spawn(&world,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let mut player=start.clone();let mut steps=Vec::new();
        for index in 0..9 {
            let input=Controls{wish:Vec2::X,run:true,jump:index==2,..Default::default()};
            for tick in 0..24 {player.tick(&world,Controls{jump:input.jump && tick==0,..input});}
            steps.push((input,player.feet));
        }
        // Respawning needs a full body footprint, but ordinary movement can
        // use this authored narrow floor with the route's eight-unit margin.
        assert!(!super::super::placement::supported(&world,steps[1].1));
        assert!(crate::route::supported_footing(&world,steps[1].1));
        let approach=through_first_landing(&world,&start,steps);
        assert_eq!(approach.len(),2);
        assert!(plan_stays_grounded(&world,&start,&approach));
    }
    #[test]
    fn landing_settling_requires_a_planned_goal() {
        let mut value=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":[0,200,160],
            "navigate":true,"settle_landings":true,"attack":false,"capture":false,"finish":false});
        assert!(serde_json::from_value::<Command>(value.clone()).unwrap().validate().is_ok());
        value["goal"]=serde_json::Value::Null;
        assert!(serde_json::from_value::<Command>(value.clone()).unwrap().validate().is_err());
        value["goal"]=serde_json::json!([0,200,160]);value["navigate"]=false.into();
        assert!(serde_json::from_value::<Command>(value).unwrap().validate().is_err());
    }
    #[test]
    fn obstructed_walk_uses_displacement_even_when_collision_retains_velocity() {
        let world=World::fixture(&[(vec3(-100.,-100.,-40.),vec3(100.,100.,0.))]);
        let mut player=Player::spawn(&world,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let now=Instant::now();
        let mut progress=WalkingProgress::new(player.feet,now);
        // The recorded Burning Curiosity stop retained this velocity for over
        // a minute without changing its supported, dry feet position.
        player.velocity=vec3(-59.24968,-197.07399,-100.558105);
        assert!(!progress.obstructed(&player,now+Duration::from_millis(999)));
        assert!(progress.obstructed(&player,now+Duration::from_secs(1)));
        player.grounded=false;
        assert!(!progress.obstructed(&player,now+Duration::from_secs(2)));
        player.grounded=true;player.feet.x+=9.;
        assert!(!progress.obstructed(&player,now+Duration::from_secs(3)));
        assert!(!progress.obstructed(&player,now+Duration::from_millis(3999)));
        assert!(progress.obstructed(&player,now+Duration::from_secs(4)));
    }
    #[test]
    fn liquid_avoidance_requires_a_planned_goal() {
        let mut value=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":[0,200,160],
            "navigate":true,"avoid_liquid_damage":true,"attack":false,"capture":false,"finish":false});
        assert!(serde_json::from_value::<Command>(value.clone()).unwrap().validate().is_ok());
        value["goal"]=serde_json::Value::Null;
        assert!(serde_json::from_value::<Command>(value.clone()).unwrap().validate().is_err());
        value["goal"]=serde_json::json!([0,200,160]);value["navigate"]=false.into();
        assert!(serde_json::from_value::<Command>(value).unwrap().validate().is_err());
    }
    #[test]
    fn remote_landing_feedback_rejects_the_recorded_clockwork_missed_landing() {
        let mut feedback=LandingFeedback::default();
        feedback.start(vec3(-703.18274,3222.927,64.03125));
        for at in [vec3(-703.18274,3186.05176,64.03125),vec3(-733.97363,3134.80664,65.15625),
            vec3(-779.22852,3089.55469,129.1562),vec3(-816.39954,3040.95117,161.15616),
            vec3(-822.04095,2982.99194,159.83939),vec3(-815.8537,2925.1792,126.17596)] {
            feedback.advance(at);
        }
        let next=Some(vec3(-808.18878,2903.03125,90.72325));
        let mut actual=Player::new(vec3(-818.36395,3090.13013,65.15625));actual.grounded=true;
        assert!(!feedback.misplaced(&actual,next),"Do not mistake pre-takeoff input latency for a missed landing");
        actual.grounded=false;assert!(!feedback.misplaced(&actual,next));
        actual.grounded=true;assert!(feedback.misplaced(&actual,next));
        assert!(!feedback.misplaced(&actual,next),"One observed landing produces one replan");
        feedback.start(Vec3::ZERO);
        assert!(!feedback.misplaced(&actual,next),"A new plan resets old flight evidence");
    }
    #[test]
    fn delayed_landings_on_the_recent_path_do_not_restart_valid_routes() {
        let mut feedback=LandingFeedback::default();feedback.start(Vec3::ZERO);
        for i in 1..=8 {feedback.advance(vec3(i as f32*40.,0.,0.));}
        let mut actual=Player::new(vec3(220.,0.,0.));
        actual.grounded=false;assert!(!feedback.misplaced(&actual,Some(vec3(360.,0.,0.))));
        actual.grounded=true;assert!(!feedback.misplaced(&actual,Some(vec3(360.,0.,0.))));
        // Several later segments may have been sent while this valid landing
        // was in flight. Spatial interpolation keeps that delayed observation.
        assert!(actual.feet.distance(vec3(360.,0.,0.))>120.);
    }
    #[test]
    fn projected_return_uses_the_upper_supported_floor_without_moving_a_body() {
        let world=World::fixture(&[(vec3(-100.,-100.,-64.),vec3(100.,100.,0.)),
            (vec3(-50.,-50.,32.),vec3(50.,50.,64.))]);
        let actual=Player::new(Vec3::ZERO);let goal=vec3(0.,200.,160.);
        let landings=return_landings(&world,actual.feet,goal,|_|true);
        assert!(!landings.is_empty() && (landings[0].z-64.).abs()<0.1);
        assert_eq!(actual.feet,Vec3::ZERO);
        assert!(return_landings(&world,actual.feet,goal,|_|false).is_empty());
        let narrow=World::fixture(&[(vec3(-5.,-5.,0.),vec3(5.,5.,64.))]);
        assert!(return_landings(&narrow,Vec3::ZERO,goal,|_|true).is_empty(),"No waiting target on an unsupported edge");
    }
    #[test]
    fn landing_feedback_requires_a_planned_goal() {
        let mut value=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":[0,200,160],
            "navigate":true,"landing_feedback":true,"attack":false,"capture":false,"finish":false});
        assert!(serde_json::from_value::<Command>(value.clone()).unwrap().validate().is_ok());
        value["goal"]=serde_json::Value::Null;
        assert!(serde_json::from_value::<Command>(value.clone()).unwrap().validate().is_err());
        value["goal"]=serde_json::json!([0,200,160]);value["navigate"]=false.into();
        assert!(serde_json::from_value::<Command>(value).unwrap().validate().is_err());
    }
    #[test]
    fn ledge_handoff_waits_for_the_delayed_pullup_instead_of_releasing() {
        let world=World::fixture(&[(vec3(16.,-100.,-100.),vec3(200.,100.,64.))]);
        let mut planned=Player::new(vec3(0.,0.,6.));planned.velocity=vec3(30.,0.,-80.);
        let inputs=[Controls{wish:Vec2::X,..Default::default()},
            Controls{wish:Vec2::X,jump:true,..Default::default()},
            Controls{wish:-Vec2::X,..Default::default()}];
        let mut prediction=planned.clone();let mut steps=Vec::new();
        for input in inputs {
            for tick in 0..24 {prediction.tick(&world,Controls{jump:input.jump && tick==0,..input});}
            steps.push((input,prediction.feet));
        }
        let (steps,grip)=stop_at_ledge(&world,&planned,vec3(100.,0.,64.),steps);
        // A slightly later catch makes the timed pull input arrive in midair.
        // The fixture's observed catch then needs another 36 ticks to deliver
        // its explicit pull. No body/ledge state is assigned during the replay.
        let mut player=Player::new(vec3(0.,0.,14.));player.velocity=vec3(30.,0.,120.);
        let mut views=VecDeque::from(vec![player.clone();24]);let mut pull=None;
        for tick in 0..480 {
            let view=views.pop_front().unwrap();
            if pull.is_none() && view.ledge.as_ref().is_some_and(|h|!h.pulling) {pull=Some(tick+12);}
            let hold=ledge_handoff_wish(&world,&view,vec3(100.,0.,64.),grip,0.3).unwrap_or(Vec2::ZERO);
            let mut input=steps.get(tick/24).map_or(Controls{wish:hold,..Default::default()},|(input,_)|
                Controls{jump:input.jump && tick%24==0,..*input});
            if pull==Some(tick) {input.jump=true;}
            player.tick(&world,input);views.push_back(player.clone());
            assert!(world.body_clear(player.feet));
        }
        assert!(pull.is_some(),"No observed native catch");
        assert!(player.grounded && player.ledge.is_none() && player.climbs==1 && player.feet.z>63.9,
            "Delayed pull lost the ledge: feet={:?}, climbs={}",player.feet,player.climbs);
    }
    #[test]
    fn ledge_handoff_keeps_routes_to_lower_or_opposite_goals_intact() {
        let world=World::fixture(&[(vec3(16.,-100.,-100.),vec3(200.,100.,64.))]);
        let mut start=Player::new(vec3(0.,0.,6.));start.velocity=vec3(30.,0.,-80.);
        let steps=vec![(Controls{wish:Vec2::X,..Default::default()},vec3(0.,0.,3.125)),
            (Controls{wish:-Vec2::X,..Default::default()},vec3(-20.,0.,-10.))];
        for goal in [vec3(100.,0.,0.),vec3(-100.,0.,64.)] {
            let (remaining,grip)=stop_at_ledge(&world,&start,goal,steps.clone());
            assert_eq!(remaining.len(),steps.len());assert!(grip.is_none());
        }
        let open=World::fixture(&[]);
        assert_eq!(stop_at_ledge(&open,&start,vec3(100.,0.,64.),steps.clone()).0.len(),steps.len());
    }
    #[test]
    fn ledge_handoff_shimmies_clear_of_an_obstructed_pull_before_jumping() {
        let world=World::fixture(&[(vec3(16.,-100.,-100.),vec3(200.,100.,64.)),
            (vec3(24.,-100.,64.),vec3(60.,0.,140.))]);
        let mut player=Player::new(vec3(0.,-30.,6.));player.velocity=vec3(30.,0.,-80.);
        player.tick(&world,Controls{wish:Vec2::X,..Default::default()});
        assert!(player.ledge.as_ref().is_some_and(|h|!h.pull_clear(&world)));
        let mut rejected=player.clone();rejected.tick(&world,Controls{jump:true,..Default::default()});
        assert!(rejected.ledge.as_ref().is_some_and(|h|!h.pulling));
        let mut views=VecDeque::from(vec![player.clone();24]);
        let mut pending=VecDeque::from(vec![Controls::default();12]);let mut sent=false;
        for _ in 0..900 {
            let view=views.pop_front().unwrap();
            let wish=ledge_handoff_wish(&world,&view,vec3(100.,40.,64.),Some(vec3(0.,40.,3.125)),0.3).unwrap_or(Vec2::ZERO);
            let jump=!sent && view.ledge.as_ref().is_some_and(|h|h.pull_clear(&world));
            sent|=jump;pending.push_back(Controls{wish,jump,..Default::default()});
            player.tick(&world,pending.pop_front().unwrap());views.push_back(player.clone());
            assert!(world.body_clear(player.feet));
        }
        assert!(sent && player.grounded && player.ledge.is_none() && player.climbs==1 && player.feet.y>14.9);
    }
    #[test]
    fn ledge_handoff_requires_an_explicit_planned_goal() {
        let mut value=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":[100,0,64],
            "navigate":true,"ledge_handoff":true,"attack":false,"capture":false,"finish":false});
        assert!(serde_json::from_value::<Command>(value.clone()).unwrap().validate().is_ok());
        value["goal"]=serde_json::Value::Null;
        assert!(serde_json::from_value::<Command>(value.clone()).unwrap().validate().is_err());
        value["goal"]=serde_json::json!([100,0,64]);value["navigate"]=false.into();
        assert!(serde_json::from_value::<Command>(value).unwrap().validate().is_err());
    }
    #[test]
    fn a_consumed_jump_is_not_replayed_after_relocation() {
        for local in [PlayerId::Blue, PlayerId::Red] {
            let path=std::env::temp_dir().join(format!("looking-glass-jump-{}-{:?}.json",std::process::id(),local));
            let mut driver=Driver::new(path.clone());
            let mut command=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":[512,0,0],
                "jump":true,"navigate":false,"attack":false,"capture":false,"finish":false});
            let mut now=driver.refreshed;
            let mut keys=Input::default();
            for (index,(epoch,id)) in [(1,1),(1,2),(2,2)].into_iter().enumerate() {
                command["id"]=id.into();command["epoch"]=epoch.into();
                std::fs::write(&path,serde_json::to_vec(&command).unwrap()).unwrap();
                let file=std::fs::File::options().write(true).open(&path).unwrap();
                file.set_times(std::fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH+Duration::from_secs(index as u64+1))).unwrap();
                now+=Duration::from_millis(60);driver.poll(now);
                assert_eq!(driver.report.command,Some(id));
                for stamp in [(epoch,0,0),(epoch,30,0),(epoch,30,40)] {
                    driver.observe_location(stamp,50,now);
                    for _ in 0..3 {
                        Driver::direct_jump(driver.command.as_ref().unwrap(),local,0,Duration::ZERO,
                            &mut keys,&mut driver.jumped);
                        assert_eq!(keys.jumps,index as u64+1,"Wrong jump count across command/visit changes: {stamp:?}");
                    }
                }
            }
            std::fs::remove_file(path).unwrap();
        }
    }
    #[test]
    fn ice_preference_requires_own_resources_range_and_a_clear_target() {
        use crate::inventory::{PickupKind,Stats};
        let world=World::fixture(&[]);
        let mut body=super::super::world::Body {player:Player::new(Vec3::ZERO),stats:Stats::default(),
            yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0};
        let enemy=crate::combat::Target {id:7,center:body.player.eye()+Vec3::X*200.,half:Vec3::splat(16.)};
        let targets=[enemy];let context=crate::combat::Context{world:&world,targets:&targets};
        let mut keys=Input::default();
        assert!(ice_aim(&context,&targets,&body,true,&mut keys).is_none());
        assert!(body.stats.apply(PickupKind::Weapon(4),1.));
        let saved=serde_json::to_value(&body.stats).unwrap();
        assert!(ice_aim(&context,&targets,&body,false,&mut keys).is_none());
        assert_eq!(ice_aim(&context,&targets,&body,true,&mut keys).unwrap().id,7);
        assert_eq!(keys.weapon,Some(4));assert!(keys.primary && !keys.alternate);
        assert_eq!(serde_json::to_value(&body.stats).unwrap(),saved,"Input selection must not grant/spend resources");
        let friend=crate::combat::Target{id:crate::combat::COOP_RED,center:body.player.eye()+Vec3::X*80.,half:Vec3::splat(24.)};
        let both=[enemy,friend];let blocked=crate::combat::Context{world:&world,targets:&both};
        assert!(ice_aim(&blocked,&targets,&body,true,&mut keys).is_none());
        let wall=World::fixture(&[(vec3(60.,-100.,-100.),vec3(70.,100.,200.))]);
        assert!(ice_aim(&crate::combat::Context{world:&wall,targets:&targets},&targets,&body,true,&mut keys).is_none());
        let far=[crate::combat::Target{center:body.player.eye()+Vec3::X*500.,..enemy}];
        assert!(ice_aim(&crate::combat::Context{world:&world,targets:&far},&far,&body,true,&mut keys).is_none());
        body.player.immersion.level=1;
        assert!(ice_aim(&context,&targets,&body,true,&mut keys).is_none());
        body.player.immersion.level=0;assert!(body.stats.spend_will(100.));
        assert!(ice_aim(&context,&targets,&body,true,&mut keys).is_none());
    }
    #[test]
    fn combat_tactics_uses_actual_melee_reach_and_respects_occluders() {
        let world=World::fixture(&[]);let eye=vec3(0.,0.,48.);
        // The broad body overlaps the actual melee volume although its centre
        // lies beyond the old 80-unit shortcut. Cards must not displace it.
        let enemy=crate::combat::Target{id:7,center:vec3(90.,0.,48.),half:vec3(40.,16.,32.)};
        let targets=[enemy];let context=crate::combat::Context{world:&world,targets:&targets};
        let mut aim=CombatAim::default();let mut keys=Input::default();
        assert_eq!(aim.sample((1,1),&context,&targets,eye,true,&mut keys).unwrap().id,7);
        assert_eq!(keys.weapon,Some(0));assert!(keys.primary && !keys.alternate);
        assert!(crate::weapons::blade_melee(&context,eye,Vec3::X).iter().any(|h|h.id==7 && h.damage>0.));
        let wall=World::fixture(&[(vec3(20.,-100.,-100.),vec3(30.,100.,200.))]);
        let blocked=crate::combat::Context{world:&wall,targets:&targets};
        let mut keys=Input::default();
        assert!(aim.sample((1,2),&blocked,&targets,eye,true,&mut keys).is_none());
        assert!(keys.weapon.is_none() && !keys.primary && !keys.alternate);
        let friend=crate::combat::Target{id:crate::combat::COOP_RED,center:vec3(30.,0.,48.),half:Vec3::splat(20.)};
        let both=[enemy,friend];let blocked=crate::combat::Context{world:&world,targets:&both};
        assert!(aim.sample((1,3),&blocked,&targets,eye,true,&mut keys).is_none());
    }
    #[test]
    fn combat_range_rejection_matches_unfiltered_aim_with_large_bodies_and_occluders() {
        use crate::{combat::{Context,Target},route::fight};
        let eye=vec3(0.,0.,48.);
        let open=World::fixture(&[]);
        let wall=World::fixture(&[(vec3(200.,-200.,-100.),vec3(210.,200.,200.))]);
        let floor=World::fixture(&[(vec3(-4000.,-4000.,-100.),vec3(4000.,4000.,0.))]);
        let distant=(0..64).map(|i|Target{id:100+i,center:vec3(3000.+i as f32*30.,500.,48.),
            half:Vec3::splat(20.)}).collect::<Vec<_>>();
        for (center,half) in [
            (vec3(400.,0.,48.),Vec3::splat(20.)),
            (vec3(1000.,0.,48.),Vec3::splat(20.)),
            (vec3(1025.,0.,48.),Vec3::splat(20.)),
            // Its centre is beyond throw range; its exposed upper body is not.
            (vec3(500.,0.,-1500.),vec3(40.,40.,1600.)),
        ] {
            let enemy=Target{id:7,center,half};
            let mut foes=distant.clone();foes.push(enemy);
            assert!(distant.iter().all(|t|!target_may_be_in_range(eye,t,fight::THROW_RANGE)));
            for world in [&open,&wall,&floor] { for friend in [false,true] { for cards in [false,true] {
                let mut targets=foes.clone();
                if friend { targets.push(Target{id:crate::combat::COOP_RED,
                    center:vec3(100.,0.,48.),half:Vec3::splat(25.)}); }
                let context=Context{world,targets:&targets};
                let original=fight::plan(&context,eye,&foes,&fight::Tracker::default(),cards);
                let mut aim=CombatAim::default();let mut keys=Input::default();
                let selected=aim.sample((1,1),&context,&foes,eye,cards,&mut keys);
                assert_eq!(selected.map(|t|t.id),original.target);
                if selected.is_some() {
                    assert_eq!(keys.weapon,Some(original.weapon as u8));
                    assert_eq!(keys.alternate,original.alternate);
                    let direction=vec3(keys.pitch.cos()*keys.yaw.cos(),keys.pitch.cos()*keys.yaw.sin(),keys.pitch.sin());
                    assert!(direction.distance(original.aim)<0.0001);
                }
            }}}
        }
        let tall=Target{id:7,center:vec3(500.,0.,-1500.),half:vec3(40.,40.,1600.)};
        let context=Context{world:&floor,targets:&[tall]};
        assert_eq!(CombatAim::default().sample((1,1),&context,&[tall],eye,true,&mut Input::default()).unwrap().id,7);
    }
    #[test]
    fn combat_tactics_leads_actual_moving_blade_hits_without_resampling_stale_frames() {
        let world=World::fixture(&[]);let eye=vec3(0.,0.,48.);
        let mut aim=CombatAim::default();let mut keys=Input::default();
        let mut target=crate::combat::Target{id:7,center:vec3(400.,-200.,48.),half:Vec3::splat(8.)};
        for i in 0..=40 {
            target.center.y=-200.+i as f32*5.;
            let targets=[target];let context=crate::combat::Context{world:&world,targets:&targets};
            aim.sample((1,i*6),&context,&targets,eye,false,&mut keys);
        }
        let velocity=aim.tracker.velocity(7);
        let targets=[target];let context=crate::combat::Context{world:&world,targets:&targets};
        for _ in 0..20 {aim.sample((1,240),&context,&targets,eye,false,&mut keys);}
        assert_eq!(aim.tracker.velocity(7),velocity);
        assert!(velocity.y>99. && keys.alternate && keys.yaw>0.05);
        let direction=vec3(keys.pitch.cos()*keys.yaw.cos(),keys.pitch.cos()*keys.yaw.sin(),keys.pitch.sin());
        let mut led=crate::weapons::Projectile::thrown_blade(eye,direction);
        let mut old=crate::weapons::Projectile::thrown_blade(eye,crate::route::fight::lead(eye,target.center,Vec3::ZERO));
        let (mut hit,mut old_hit)=(false,false);
        for _ in 0..100 {
            target.center.y+=100./120.;
            let targets=[target];let context=crate::combat::Context{world:&world,targets:&targets};
            hit|=led.contact_step(1./120.,&context).0.is_some_and(|h|h.id==7);
            old_hit|=old.contact_step(1./120.,&context).0.is_some_and(|h|h.id==7);
        }
        assert!(hit && !old_hit,"Led throw must hit the moving body that static aim misses");
        aim.sample((2,0),&context,&targets,eye,false,&mut keys);
        assert_eq!(aim.tracker.velocity(7),Vec3::ZERO);
    }
    #[test]
    fn combat_dodge_uses_supported_safe_space_without_entering_the_partner() {
        let world=World::fixture(&[(vec3(-250.,-250.,-100.),vec3(250.,250.,0.))]);
        let mut p=Player::new(vec3(0.,0.,0.03125));p.grounded=true;
        let home=p.feet;let enemy=vec3(200.,0.,48.);let other=vec3(0.,80.,0.03125);
        let wish=combat_dodge(&world,&p,home,enemy,0,0.,Some(other),false,|at|at.x<100.);
        assert!(wish.y<0.,"Unsafe first direction should reverse away from the partner");
        assert_eq!(p.feet,home,"Prediction changed the real body");
        for i in 0..72 {
            p.tick(&world,Controls{wish:if i<24 {wish} else {Vec2::ZERO},run:true,..Default::default()});
            assert!(p.grounded && world.body_clear(p.feet) && (p.feet-other).truncate().length()>=36.);
        }
        assert_eq!(combat_dodge(&world,&p,home,enemy,0,0.,None,false,|_|false),Vec2::ZERO);
    }
    #[test]
    fn combat_dodge_keeps_delayed_inputs_on_a_narrow_platform() {
        let world=World::fixture(&[(vec3(-64.,-300.,-100.),vec3(64.,300.,0.))]);
        for delay in [0,12,24] {
            let mut p=Player::spawn(&world,vec3(16.,0.,crate::movement::EYE_HEIGHT+0.03125)).unwrap();
            let home=p.feet;let enemy=vec3(200.,180.,48.);
            let mut views=VecDeque::from(vec![p.clone();delay+1]);
            let mut inputs=VecDeque::from(vec![Controls::default();delay]);
            let mut movement=0.;
            for tick in 0..1200 {
                let view=views.pop_front().unwrap();
                let wish=combat_dodge(&world,&view,home,enemy,tick,2.*delay as f32/120.,None,false,|_|true);
                inputs.push_back(Controls {wish,run:true,..Default::default()});
                let old=p.feet;p.tick(&world,inputs.pop_front().unwrap());movement+=p.feet.distance(old);
                assert!(p.grounded && p.feet.z.abs()<0.1,"Delayed dodge left support: delay{delay},tick{tick},feet{:?}",p.feet);
                views.push_back(p.clone());
            }
            assert!(movement>24.,"Safe prediction disabled all useful dodging");
        }
    }
    #[test]
    fn combat_dodge_keeps_a_full_footprint_before_enemy_knockback() {
        let world=World::fixture(&[(vec3(-64.,-300.,-100.),vec3(64.,300.,0.))]);
        for delay in [0,12,24] {
            let mut p=Player::spawn(&world,vec3(16.,0.,crate::movement::EYE_HEIGHT+0.03125)).unwrap();
            let home=p.feet;let enemy=vec3(200.,180.,48.);
            let mut views=VecDeque::from(vec![p.clone();delay+1]);
            let mut inputs=VecDeque::from(vec![Controls::default();delay]);
            let mut movement=0.;
            for tick in 0..1200 {
                let view=views.pop_front().unwrap();
                let wish=combat_dodge(&world,&view,home,enemy,tick,2.*delay as f32/120.,None,false,|_|true);
                inputs.push_back(Controls{wish,run:true,..Default::default()});
                let old=p.feet;p.tick(&world,inputs.pop_front().unwrap());movement+=p.feet.distance(old);
                assert!(super::super::placement::supported(&world,p.feet),
                    "Combat accepted marginal edge contact: delay{delay},tick{tick},feet{:?}",p.feet);
                views.push_back(p.clone());
            }
            assert!(movement>24.,"Footprint checks disabled all useful dodging");
        }
    }
    #[test]
    fn combat_tactics_retains_a_dry_blade_for_an_approaching_enemy() {
        let world=World::fixture(&[(vec3(-400.,-400.,-100.),vec3(400.,400.,0.))]);
        let mut player=Player::new(vec3(0.,0.,0.03125));player.grounded=true;
        let target=crate::combat::Target{id:7,center:vec3(200.,0.,43.),half:crate::clockwork::HALF};
        let targets=[target];let context=crate::combat::Context{world:&world,targets:&targets};
        let mut aim=CombatAim::default();let mut keys=Input::default();
        aim.sample((1,1),&context,&targets,player.eye(),false,&mut keys);
        assert_eq!(keys.weapon,Some(0));assert!(keys.primary && !keys.alternate);
        assert!(crate::weapons::blade_melee(&context,player.eye(),Vec3::X).is_empty(),"No invented out-of-reach hit");
        let wish=combat_dodge(&world,&player,player.feet,target.center,1,0.,None,true,|_|true);
        assert!(wish.x>0.8,"Dry melee should close toward the real hit volume");
        let mut funded=Input::default();
        aim.sample((1,2),&context,&targets,player.eye(),true,&mut funded);
        assert_eq!(funded.weapon,Some(1));assert!(funded.primary && !funded.alternate);
    }
    #[test]
    fn combat_tactics_stops_waiting_for_an_unreachable_melee_target() {
        let world=World::fixture(&[(vec3(-400.,-400.,-100.),vec3(400.,400.,0.))]);
        let eye=vec3(0.,0.,48.03125);
        let mut target=crate::combat::Target{id:7,center:vec3(200.,0.,43.),half:crate::clockwork::HALF};
        let mut aim=CombatAim::default();
        for tick in [1,60,120,121,121,240] {
            let targets=[target];let context=crate::combat::Context{world:&world,targets:&targets};
            assert!(crate::weapons::blade_melee(&context,eye,Vec3::X).is_empty());
            let mut keys=Input::default();
            assert_eq!(aim.sample((1,tick),&context,&targets,eye,false,&mut keys).unwrap().id,target.id);
            assert_eq!(keys.weapon,Some(0));
            assert_eq!(keys.alternate,tick>=121,"Unreachable target retained the Blade at tick{tick}");
            assert_eq!(keys.primary,!keys.alternate);
        }
        // Real reach restores the melee preference, and a new visit receives
        // its own approach window instead of inheriting the expired one.
        target.center.x=50.;
        let targets=[target];let context=crate::combat::Context{world:&world,targets:&targets};
        assert!(crate::weapons::blade_melee(&context,eye,Vec3::X).iter().any(|h|h.id==target.id));
        let mut keys=Input::default();aim.sample((1,241),&context,&targets,eye,false,&mut keys);
        assert!(keys.primary && !keys.alternate);
        target.center.x=200.;
        let targets=[target];let context=crate::combat::Context{world:&world,targets:&targets};
        aim.sample((2,1),&context,&targets,eye,false,&mut keys);
        assert!(keys.primary && !keys.alternate);
    }
    #[test]
    fn timed_hold_keeps_normal_input_against_a_blocked_authored_point() {
        let world=World::fixture(&[(vec3(-100.,-100.,-100.),vec3(200.,100.,0.)),
            (vec3(84.,-100.,0.),vec3(100.,100.,200.))]);
        let mut p=Player::new(vec3(40.,0.,0.03125));p.grounded=true;
        let goal=vec2(100.,0.);
        for _ in 0..120 {
            let wish=hold_wish(goal-p.feet.truncate());
            assert!(wish.x>0. && wish.length()<=1.);
            p.tick(&world,Controls{wish,run:true,..Default::default()});
            assert!(world.body_clear(p.feet) && p.grounded);
        }
        let boundary=84.-crate::collision::PLAYER_HALF.x;
        assert!(p.feet.x>40. && (p.feet.x-boundary).abs()<0.1
            && goal.distance(p.feet.truncate())>30.,"Unexpected blocked footing: {:?}",p.feet);
        assert_eq!(hold_wish(Vec2::ZERO),vec2(0.001,0.));
        assert_eq!(delayed_hold_wish(vec2(23.,-17.),vec2(300.,50.),0.),hold_wish(vec2(23.,-17.)));
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":[100,0,0],
            "run":true,"hold_goal":true,"navigate":false,"attack":true,"capture":false,"finish":false});
        assert!(serde_json::from_value::<Command>(base.clone()).unwrap().validate().is_ok());
        for (field,value) in [("goal",serde_json::Value::Null),("run",false.into()),
            ("navigate",true.into()),("jump",true.into()),("checkpoint",true.into()),
            ("shoot_switch","MirrorTrigger".into())] {
            let mut invalid=base.clone();invalid[field]=value;
            assert!(serde_json::from_value::<Command>(invalid).unwrap().validate().is_err());
        }
    }
    #[test]
    fn timed_hold_keeps_delayed_feedback_on_the_mirror_perch() {
        let world=World::fixture(&[(vec3(520.,680.,-200.),vec3(704.,820.,0.))]);
        let goal=vec2(665.,775.);
        let replay=|delay:usize,predict:bool| {
            let mut p=Player::new(vec3(589.,751.,0.03125));p.grounded=true;
            let mut views=VecDeque::from(vec![p.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay+1]);
            let lead=(2*delay+6) as f32/120.;
            let mut command=Controls::default();
            for tick in 0..1200 {
                let seen=views.pop_front().unwrap();
                if tick%6==0 {
                    let delta=goal-seen.feet.truncate();
                    let wish=if predict {delayed_hold_wish(delta,seen.velocity.truncate(),lead)} else {hold_wish(delta)};
                    command=Controls{wish,run:true,..Default::default()};
                }
                pending.push_back(command);p.tick(&world,pending.pop_front().unwrap());
                if !p.grounded || !world.body_clear(p.feet) {return (false,p);}
                views.push_back(p.clone());
            }
            (true,p)
        };
        assert!(!replay(36,false).0,"The old delayed hold must reproduce the fall");
        for delay in [0,6,17,36] {
            let (safe,p)=replay(delay,true);
            assert!(safe && p.feet.truncate().distance(goal)<12.,
                "Delayed hold left its support: delay={delay}, feet={:?}",p.feet);
        }
    }
    #[test]
    fn switch_aim_uses_a_real_blade_path_and_requires_an_observed_target() {
        let clear=World::fixture(&[]);
        let wall=World::fixture(&[(vec3(40.,-100.,-100.),vec3(60.,100.,200.))]);
        let eye=vec3(0.,0.,48.);
        let target=crate::combat::Target{id:7,center:vec3(150.,0.,48.),half:Vec3::splat(16.)};
        let targets=[target];
        let mut aim=SwitchAim::default();let mut keys=Input::default();
        let context=crate::combat::Context{world:&clear,targets:&targets};
        assert!(aim.sample(&context,None,eye,&mut keys).is_err());
        let blocked=crate::combat::Context{world:&wall,targets:&targets};
        assert!(!aim.sample(&blocked,Some(target),eye,&mut keys).unwrap());
        assert!(!keys.primary && !keys.alternate && keys.weapon.is_none());
        assert!(!aim.sample(&context,Some(target),eye,&mut keys).unwrap());
        assert_eq!(keys.weapon,Some(0));assert!(keys.alternate && !keys.primary);
        assert_eq!(keys.wish,Vec2::ZERO);assert_eq!(keys.jumps,0);assert_eq!(keys.uses,0);
        let direction=vec3(keys.pitch.cos()*keys.yaw.cos(),keys.pitch.cos()*keys.yaw.sin(),keys.pitch.sin());
        let mut shot=crate::weapons::Projectile::thrown_blade(eye,direction);
        let mut hit=None;
        for _ in 0..120 {hit=hit.or(shot.contact_step(1./120.,&context).0);}
        assert_eq!(hit.unwrap().id,target.id);
        let wrong=crate::combat::Target{id:8,..target};
        assert!(aim.sample(&context,Some(wrong),eye,&mut Input::default()).is_err());
        assert!(aim.sample(&context,None,eye,&mut Input::default()).unwrap());
    }
    #[test]
    fn switch_approaches_keep_world_and_partner_occlusion() {
        let world=World::fixture(&[(vec3(40.,-25.,0.),vec3(60.,25.,120.))]);
        let target=crate::combat::Target{id:7,center:vec3(150.,0.,48.),half:Vec3::splat(8.)};
        let blocked=Vec3::ZERO;let clear=vec3(0.,120.,0.);
        let candidates=[blocked,clear];let targets=[target];
        let context=crate::combat::Context{world:&world,targets:&targets};
        assert_eq!(visible_switch_approaches(&context,target,&candidates),vec![clear]);
        let partner=crate::combat::Target{id:crate::combat::COOP_RED,
            center:vec3(30.,96.,48.),half:crate::collision::PLAYER_HALF};
        let targets=[target,partner];
        assert!(visible_switch_approaches(&crate::combat::Context{world:&world,targets:&targets},
            target,&candidates).is_empty());
    }
    #[test]
    fn shooting_switch_commands_exclude_other_actions_and_bound_names() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":null,
            "navigate":false,"attack":false,"capture":false,"finish":false,
            "shoot_switch":"MirrorTrigger","avoid_switches":true});
        assert!(serde_json::from_value::<Command>(base.clone()).unwrap().validate().is_ok());
        for (field,value) in [("goal",serde_json::json!([0,0,0])),("marker",1.into()),
            ("attack",true.into()),("navigate",true.into()),("checkpoint",true.into()),
            ("finish",true.into()),("jump",true.into()),("run",true.into()),
            ("use_aim",serde_json::json!([0,0])),("temple_follow","Blue".into())] {
            let mut invalid=base.clone();invalid[field]=value;
            assert!(serde_json::from_value::<Command>(invalid).unwrap().validate().is_err(),"Accepted {field}");
        }
        for name in [String::new(),"a".repeat(129),"../other".into()] {
            let mut invalid=base.clone();invalid["shoot_switch"]=name.into();
            assert!(serde_json::from_value::<Command>(invalid).unwrap().validate().is_err());
        }
    }
    #[test]
    fn delayed_straight_movement_settles_at_a_small_side_step() {
        let world=World::fixture(&[(vec3(500.,2800.,-100.),vec3(1000.,3300.,0.))]);
        for delay in [0,6,17,36] { for run in [false,true] {
            // Actual Funhouse parking goal; the delayed guest circled it while
            // the first actor waited. Model both observation and input queues.
            let mut p=Player::new(vec3(743.6277,3000.6494,0.03125));p.grounded=true;
            let goal=vec2(743.6277,3048.6494);
            let mut views=VecDeque::from(vec![p.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay+1]);
            let lead=(2*delay+6) as f32/120.;
            let mut command=Controls::default();
            for tick in 0..2400 {
                let seen=views.pop_front().unwrap();
                if tick%6==0 {
                    command=Controls {wish:straight_wish(goal-seen.feet.truncate(),
                        seen.velocity.truncate(),lead,run,None),run,..Default::default()};
                }
                pending.push_back(command);p.tick(&world,pending.pop_front().unwrap());
                views.push_back(p.clone());
                assert!(p.grounded && world.body_clear(p.feet));
            }
            assert!(p.feet.truncate().distance(goal)<2. && p.velocity.truncate().length()<1.,
                "Side step did not settle: delay={delay}, run={run}, feet={:?}, velocity={:?}",p.feet,p.velocity);
        }}
    }
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
        driver.checkpoint_result(7,2,100,Duration::ZERO,Some("Wait until the scene finishes".into()));
        assert_eq!(driver.checkpoint_request(2,now),None);
        driver.command.as_mut().unwrap().heartbeat+=1;
        assert_eq!(driver.checkpoint_request(2,now),None);
        driver.command.as_mut().unwrap().id=8;
        assert_eq!(driver.checkpoint_request(2,now),Some(8));
        driver.checkpoint_result(8,2,120,Duration::from_millis(12),None);
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
                if landing.is_none() && replan && progress.obstructed(&view,now) {
                    steps=Route::plan_with_takeoff_margin(&blocked,&view,goal,22.,12).unwrap();
                    assert!(plan_stays_grounded(&blocked,&view,&steps));
                    progress=WalkingProgress::new(view.feet,now);next=0;replans+=1;
                }
                while next<steps.len() && walking_control(&view,steps[next].1,0.,false).1 {next+=1;}
                let wish=landing.map_or_else(||walking_control(&view,
                    steps.get(next).map_or(goal,|s|s.1),2.*delay as f32/120.,false).0,|s|s.0);
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
    fn grounded_search_walks_around_a_gap_with_delayed_observations_and_inputs() {
        let world=World::fixture(&[
            (vec3(-200.,-120.,-100.),vec3(100.,120.,0.)),
            (vec3(80.,60.,-100.),vec3(340.,120.,0.)),
            (vec3(320.,-120.,-100.),vec3(600.,120.,0.)),
        ]);
        let start=Player::spawn(&world,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let goal=vec3(420.,0.,0.);
        let steps=Route::plan_grounded(&world,&start,goal,22.).expect("Walkable route around gap");
        assert!(steps.iter().all(|(input,_)|!input.jump && !input.run));
        assert!(steps.iter().any(|(_,at)|at.y>60.),"Must use the supported detour");
        let steps=straight_grounded_sections(start.feet,steps);
        for delay in [0,12,24] {
            let mut player=start.clone();
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay]);
            let mut next=0;
            for _ in 0..3600 {
                let view=views.pop_front().unwrap();
                while next<steps.len() && grounded_waypoint_ready(&world,&view,steps[next].1,
                    steps.get(next+1).map(|s|s.1),true) { next+=1; }
                let lead=2.*delay as f32/120.;
                let (wish,run)=landing_control(&world,&view,goal,lead,8.).map_or_else(||
                    (walking_control(&view,steps.get(next).map_or(goal,|s|s.1),lead,true).0,true),|s|(s.0,false));
                pending.push_back(Controls {wish,run,..Default::default()});
                player.tick(&world,pending.pop_front().unwrap());
                assert!(player.grounded && player.jumps==0,"Left the supported route: delay{delay}, {:?}",player.feet);
                views.push_back(player.clone());
                if player.feet.distance(goal)<8. && player.velocity.truncate().length()<20. { break; }
            }
            assert!(player.feet.distance(goal)<8.,"Missed goal at delay{delay}: {:?}",player.feet);
        }
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
        let replay=|feedback:bool,delay:usize,run:bool| {
            let route=if run {straight_grounded_sections(start.feet,steps.clone())} else {steps.clone()};
            let mut player=start.clone();
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay]);
            let mut next=0;
            let mut lowest=player.feet.z;
            for tick in 0..1200 {
                let world=if tick<96 {&blocked} else {&open};
                let view=views.pop_front().unwrap();
                let input=if feedback {
                    while next<route.len() && grounded_waypoint_ready(world,&view,route[next].1,
                        route.get(next+1).map(|s|s.1),run) {next+=1;}
                    let target=route.get(next).map_or(goal,|s|s.1);
                    landing_control(world,&view,goal,2.*delay as f32/120.,8.)
                        .map(|(wish,_)|Controls {wish,..Default::default()})
                        .unwrap_or(Controls {wish:walking_control(&view,target,2.*delay as f32/120.,run).0,run,..Default::default()})
                } else {steps.get(tick/24).map_or(Controls::default(),|s|s.0)};
                pending.push_back(input);
                player.tick(world,pending.pop_front().unwrap());
                views.push_back(player.clone());
                lowest=lowest.min(player.feet.z);
            }
            (player,lowest)
        };
        assert!(replay(false,0,false).1 < -100.,"Timed control did not reproduce the missed turn");
        for run in [false,true] { for delay in [0,12,24] {
            let (player,lowest)=replay(true,delay,run);
            assert!(lowest>=-0.1 && player.grounded && player.feet.distance(goal)<8.,
                "Feedback route failed run={run} after {delay} ticks of delay: {:?}, minimum {lowest}",player.feet);
        }}
    }
    #[test]
    fn running_grounded_routes_cross_long_corridors_without_waypoint_stops() {
        let world=World::fixture(&[(vec3(-40.,-48.,-100.),vec3(1000.,48.,0.))]);
        let start=Player::spawn(&world,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let mut planned=start.clone();
        let input=Controls {wish:Vec2::X,run:true,..Default::default()};
        let steps:Vec<_>=(0..13).map(|_| {
            for _ in 0..24 {planned.tick(&world,input);}
            (input,planned.feet)
        }).collect();
        assert!(plan_stays_grounded(&world,&start,&steps));
        let goal=planned.feet;
        for delay in [0,12,24] {
            let mut times=[0;2];
            for (mode,run) in [false,true].into_iter().enumerate() {
                let route=if run {straight_grounded_sections(start.feet,steps.clone())} else {steps.clone()};
                let mut player=start.clone();
                let mut views=VecDeque::from(vec![player.clone();delay+1]);
                let mut pending=VecDeque::from(vec![Controls::default();delay]);
                let mut next=0;
                // Include the deliberately slow legacy walking baseline at
                // 400 ms round trip; compare arrivals, not a short timeout.
                for tick in 0..6000 {
                    let view=views.pop_front().unwrap();
                    let lead=2.*delay as f32/120.;
                    while next<route.len() && grounded_waypoint_ready(&world,&view,route[next].1,
                        route.get(next+1).map(|s|s.1),run) {next+=1;}
                    let control=landing_control(&world,&view,goal,lead,8.)
                        .map(|(wish,_)|Controls {wish,..Default::default()})
                        .unwrap_or(Controls {wish:walking_control(&view,
                            route.get(next).map_or(goal,|s|s.1),lead,run).0,run,..Default::default()});
                    pending.push_back(control);
                    player.tick(&world,pending.pop_front().unwrap());
                    assert!(player.grounded && player.feet.z.abs()<0.1);
                    views.push_back(player.clone());
                    if landing_control(&world,&player,goal,lead,8.).is_some_and(|(_,settled)|settled) {
                        times[mode]=tick; break;
                    }
                }
                assert!(times[mode]>0,"Corridor never settled: run={run} delay={delay}, feet={:?}",player.feet);
            }
            assert!(times[1]<times[0],"Running did not improve traversal: delay={delay} ticks={times:?}");
        }
    }
    #[test]
    fn running_corner_advancement_requires_clear_body_and_floor_support() {
        let floor=(vec3(-100.,-100.,-100.),vec3(100.,150.,0.));
        let open=World::fixture(&[floor]);
        let player=Player::spawn(&open,vec3(-20.,0.,crate::movement::EYE_HEIGHT)).unwrap();
        let point=vec3(0.,0.,player.feet.z);
        let next=point+Vec3::Y*100.;
        assert!(grounded_waypoint_ready(&open,&player,point,Some(next),true));
        assert!(!grounded_waypoint_ready(&open,&player,point,Some(next),false));
        assert!(!grounded_waypoint_ready(&open,&player,point,None,true));
        let wall=World::fixture(&[floor,(vec3(-30.,20.,0.),vec3(-20.,35.,100.))]);
        assert!(!grounded_waypoint_ready(&wall,&player,point,Some(next),true));
        let gap=World::fixture(&[(vec3(-40.,-40.,-100.),vec3(4.,40.,0.)),
            (vec3(64.,-40.,-100.),vec3(150.,40.,0.))]);
        let player=Player::spawn(&gap,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        assert!(!grounded_waypoint_ready(&gap,&player,player.feet+Vec3::X*10.,
            Some(player.feet+Vec3::X*100.),true));
    }
    #[test]
    fn delayed_running_leaves_a_nearby_corner_under_repeated_knockback() {
        let world=World::fixture(&[(vec3(-100.,-150.,-100.),vec3(250.,100.,0.))]);
        // Relative positions from Red's Checkmate waypoint-39 failure.
        let start=Player::spawn(&world,Vec3::Z*crate::movement::EYE_HEIGHT).unwrap();
        let route=[vec3(9.374,-2.829,start.feet.z),vec3(54.626,-48.081,start.feet.z),
            vec3(85.53,-58.69,start.feet.z)];
        let goal=route[2];
        let mut arrivals=[None;2];
        for (mode,advance) in [false,true].into_iter().enumerate() {
            let mut player=start.clone(); player.velocity=vec3(23.735,-3.133,0.);
            let delay=17;
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay]);
            let mut next=0;
            for tick in 0..1200 {
                let view=views.pop_front().unwrap();
                let lead=2.*delay as f32/120.;
                while next<route.len() && if advance {
                    grounded_waypoint_ready(&world,&view,route[next],route.get(next+1).copied(),true)
                } else {walking_control(&view,route[next],lead,true).1} {next+=1;}
                let control=landing_control(&world,&view,goal,lead,8.)
                    .map(|(wish,_)|Controls{wish,..Default::default()})
                    .unwrap_or(Controls{wish:walking_control(&view,
                        route.get(next).copied().unwrap_or(goal),lead,true).0,run:true,..Default::default()});
                if tick%72==0 && player.feet.truncate().length()<36. {
                    player.knockback(vec3(-80.,20.,0.));
                }
                pending.push_back(control); player.tick(&world,pending.pop_front().unwrap());
                assert!(player.grounded && player.feet.z.abs()<0.1);
                views.push_back(player.clone());
                if landing_control(&world,&player,goal,lead,8.).is_some_and(|(_,settled)|settled) {
                    arrivals[mode]=Some(tick); break;
                }
            }
        }
        let improved=arrivals[1].expect("Delayed runner did not leave the knockback area");
        assert!(arrivals[0].is_none_or(|old|improved<old),"Corner advancement did not improve arrival: {arrivals:?}");
    }
    #[test]
    fn a_supported_goal_walk_can_replace_an_obsolete_pull_but_not_a_gap_jump() {
        let pad=World::fixture(&[(vec3(-80.,-80.,-100.),vec3(80.,80.,0.))]);
        let goal=Vec3::Z*0.03125;
        let mut player=Player::new(vec3(10.,28.,0.03125));player.grounded=true;
        player.velocity=vec3(-212.,-216.,0.);
        assert!(pending_jump([false,true],false));
        assert!(supported_goal_walk(&pad,&player,goal));
        // Once a live landing is observed, ordinary landing steering brakes
        // here instead of executing the queued ledge-pull as a new jump.
        for _ in 0..240 {
            let (wish,_)=landing_control(&pad,&player,goal,0.,2.).unwrap();
            player.tick(&pad,Controls{wish,..Default::default()});
        }
        assert!(player.grounded && player.jumps==0 && player.feet.distance(goal)<2.);
        let gap=World::fixture(&[
            (vec3(-100.,-80.,-100.),vec3(-8.,80.,0.)),
            (vec3(8.,-80.,-100.),vec3(100.,80.,0.)),
        ]);
        player.feet=vec3(-20.,0.,0.03125);player.velocity=Vec3::ZERO;
        assert!(!supported_goal_walk(&gap,&player,vec3(20.,0.,0.03125)),
            "A nearby island must retain its pending jump");
        assert!(!supported_goal_walk(&pad,&player,vec3(0.,0.,80.)),
            "A different floor cannot cancel a pending climb");
        player.grounded=false;
        assert!(!supported_goal_walk(&pad,&player,goal),"Airborne proximity is not a completed landing");
    }

    #[test]
    fn landing_handoff_requires_supported_footing_beneath_the_descending_body() {
        let world=World::fixture(&[(vec3(0.,-100.,-100.),vec3(120.,100.,0.))]);
        let goal=vec3(48.,0.,0.03125);
        for x in [-10.,4.,12.] {
            let mut player=Player::new(vec3(x,0.,40.));
            player.grounded=false;
            player.velocity=vec3(120.,0.,-40.);
            let support=world.body_trace(player.feet,vec3(x,0.,-35.));
            assert!(support.fraction<1.,"Fixture must overlap the landing with its body box");
            assert_eq!(landing_control(&world,&player,goal,0.,8.).is_some(),x>=8.,
                "Lateral jump steering was replaced before the feet had a supported landing at x={x}");
            assert!(landing_control(&world,&player,goal,0.2,8.).is_some(),
                "Delayed landing must anticipate actual movement onto supported footing");
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
    fn carried_waypoint_holds_on_rotating_support_with_delayed_inputs() {
        let world=World::fixture(&[(vec3(-512.,-512.,-100.),vec3(512.,512.,0.))]);
        let omega=50_f32.to_radians();
        let rotate=Quat::from_rotation_z(omega*crate::movement::FIXED_DT);
        let goal=vec3(160.,0.,0.03125);
        for delay in [0,12,24,45] {
            let mut player=Player::spawn(&world,goal+vec3(20.,-10.,crate::movement::EYE_HEIGHT)).unwrap();
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay]);
            for tick in 0..2400 {
                let view=views.pop_front().unwrap();
                let carry=Vec3::Z.cross(view.feet)*omega;
                let (wish,_)=carried_goal_control(&view,goal,carry,2.*delay as f32/120.,8.);
                pending.push_back(Controls {wish,run:true,..Default::default()});
                player.tick(&world,pending.pop_front().unwrap());
                player.feet=rotate*player.feet;
                assert!(player.grounded && player.jumps==0);
                views.push_back(player.clone());
                if tick>=2160 {
                    assert!(carried_goal_control(&player,goal,Vec3::Z.cross(player.feet)*omega,0.,8.).1,
                        "Lost carried arrival at delay {delay}: {:?}, {:?}",player.feet,player.velocity);
                }
            }
        }
    }
    #[test]
    fn carried_arrival_checks_world_motion_and_keeps_compensating() {
        let mut p=Player::new(vec3(160.,0.,0.));p.grounded=true;
        let carry=vec3(0.,140.,0.);
        assert!(!carried_goal_control(&p,p.feet,carry,0.,8.).1,"Idle input is still moving with the gear");
        p.velocity=-carry;
        let (wish,settled)=carried_goal_control(&p,p.feet,carry,0.,8.);
        assert!(settled && (wish*320.+carry.truncate()).length()<0.001,
            "Arrived waiter must retain ordinary counter-motion input");
    }
    #[test]
    fn pending_route_keeps_its_start_on_a_rotating_support() {
        let world=World::fixture(&[(vec3(-512.,-512.,-100.),vec3(512.,512.,0.))]);
        let omega=50_f32.to_radians();
        let rotate=Quat::from_rotation_z(omega*crate::movement::FIXED_DT);
        for delay in [0,12,24,45] {
            let mut player=Player::spawn(&world,vec3(160.,0.,crate::movement::EYE_HEIGHT+0.03125)).unwrap();
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay]);
            let mut anchor=None;let mut worker_start=None;let mut held=0;
            // Allow convergence at the longest delay, then require ten full
            // seconds with the same valid worker start. A planner must not
            // repeatedly recapture its start to hide drift.
            for _ in 0..3600 {
                let view=views.pop_front().unwrap();
                let carry=Vec3::Z.cross(view.feet)*omega;
                let (wish,ready)=planning_hold_control(&mut anchor,&view,Some(carry),2.*delay as f32/120.);
                if ready && worker_start.is_none() {worker_start=Some(view.feet);}
                pending.push_back(Controls{wish,run:true,..Default::default()});
                player.tick(&world,pending.pop_front().unwrap());
                player.feet=rotate*player.feet;
                views.push_back(player.clone());
                assert!(player.grounded && player.jumps==0);
                if let Some(start)=worker_start {
                    assert!(player.feet.distance(start)<16.,"Pending plan start drifted at delay {delay}: {:?}",player.feet-start);
                    held+=1;
                }
            }
            assert!(held>=1200,"Insufficient stable planning time at delay {delay}: {held} ticks, feet {:?}, velocity {:?}, anchor {anchor:?}",player.feet,player.velocity);
        }
    }
    #[test]
    fn planning_hold_requires_world_settling_and_releases_lost_support() {
        let mut p=Player::new(vec3(160.,0.,0.));p.grounded=true;
        let carry=vec3(0.,140.,0.);let mut anchor=None;
        assert!(!planning_hold_control(&mut anchor,&p,Some(carry),0.).1);
        p.velocity=-carry;
        let (wish,ready)=planning_hold_control(&mut anchor,&p,Some(carry),0.);
        assert!(ready && wish.length()>0. && anchor==Some(p.feet));
        p.grounded=false;
        assert_eq!(planning_hold_control(&mut anchor,&p,None,0.),(Vec2::ZERO,false));
        assert!(anchor.is_none());
        p.feet+=Vec3::X*100.;p.grounded=true;p.velocity=Vec3::ZERO;
        assert_eq!(planning_hold_control(&mut anchor,&p,None,0.),(Vec2::ZERO,true));
        planning_hold_control(&mut anchor,&p,Some(carry),0.);
        assert_eq!(anchor,Some(p.feet));
    }
    #[test]
    fn blocked_grounded_landing_keeps_navigation_in_control() {
        let floor=(vec3(-200.,-200.,-100.),vec3(200.,200.,0.));
        let wall=(vec3(-1.,-30.,0.),vec3(1.,30.,100.));
        let blocked=World::fixture(&[floor,wall]);
        let open=World::fixture(&[floor]);
        let player=Player::spawn(&blocked,vec3(-20.,0.,crate::movement::EYE_HEIGHT+0.03125)).unwrap();
        let goal=vec3(20.,0.,0.03125);
        assert!(player.grounded && blocked.body_clear(player.feet) && blocked.body_clear(goal));
        assert!(player.feet.distance(goal)<42. && blocked.body_trace(player.feet,goal).fraction<1.);
        for lead in [0.,0.2,0.6] {
            assert!(landing_control(&blocked,&player,goal,lead,8.).is_none(),
                "A nearby blocked goal must retain navigation instead of pushing into the wall");
            assert!(landing_control(&open,&player,goal,lead,8.).is_some_and(|(wish,settled)|wish.x>0. && !settled));
        }
    }
    #[test]
    fn grounded_landing_can_walk_up_a_normal_step() {
        let floor=(vec3(-200.,-200.,-100.),vec3(200.,200.,0.));
        let step=(vec3(0.,-50.,0.),vec3(100.,50.,12.));
        let world=World::fixture(&[floor,step]);
        let mut player=Player::spawn(&world,vec3(-20.,0.,crate::movement::EYE_HEIGHT+0.03125)).unwrap();
        let goal=vec3(20.,0.,12.03125);
        assert!(player.grounded && world.body_clear(goal));
        let horizontal=vec3(goal.x,goal.y,player.feet.z);
        assert!(world.body_trace(player.feet,horizontal).fraction<1., "Horizontal clearance alone rejects a legal step");
        for _ in 0..240 {
            let (wish,settled)=landing_control(&world,&player,goal,0.,8.).expect("A normal step must retain precise walking");
            if settled { break; }
            player.tick(&world,Controls {wish,..Default::default()});
        }
        assert!(player.grounded && player.feet.distance(goal)<8., "Ordinary movement must actually climb the step: {:?}",player.feet);
        let ceiling=World::fixture(&[floor,step,(vec3(-100.,-50.,60.),vec3(100.,50.,70.))]);
        let low=Player::spawn(&ceiling,vec3(-20.,0.,crate::movement::EYE_HEIGHT+0.03125)).unwrap();
        assert!(landing_control(&ceiling,&low,goal,0.,8.).is_none(),"Stepping requires headroom");
    }
    #[test]
    fn precise_landing_stays_on_a_small_footing_at_internet_tail_latency() {
        let world=World::fixture(&[(vec3(-48.,-48.,-100.),vec3(48.,24.,0.))]);
        let goal=Vec3::ZERO;
        // The live Clockwork failure oscillates around a two-unit takeoff
        // target, then walks off before its next jump. Exercise the observed
        // median, p95 and peak delay range with ordinary native walking.
        for delay in [0,12,24,45,84] {
            let mut player=Player::spawn(&world,
                vec3(28.,-15.,crate::movement::EYE_HEIGHT+0.03125)).unwrap();
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls::default();delay]);
            let lead=2.*delay as f32/120.;
            for _ in 0..2400 {
                let view=views.pop_front().unwrap();
                let (wish,_)=landing_control(&world,&view,goal,lead,2.)
                    .unwrap_or_else(||panic!("Left supported final approach at delay {delay}: {:?}",view.feet));
                pending.push_back(Controls {wish,..Default::default()});
                player.tick(&world,pending.pop_front().unwrap());
                assert!(player.grounded && player.jumps==0,
                    "Walked off takeoff footing at delay {delay}: {:?}",player.feet);
                views.push_back(player.clone());
            }
            assert!(landing_control(&world,&player,goal,lead,2.).unwrap().1,
                "Did not settle at delay {delay}: {:?}, {:?}",player.feet,player.velocity);
        }
    }
    #[test]
    fn navigation_finishes_a_nearby_approximate_height_without_a_full_search() {
        let floor=(vec3(-100.,-100.,-100.),vec3(100.,100.,0.));
        let world=World::fixture(&[floor]);
        let goal=vec3(0.,0.,32.);
        for delay in [0,12,24] {
            let mut player=Player::spawn(&world,vec3(28.4505,-15.4922,
                crate::movement::EYE_HEIGHT+0.03125)).unwrap();
            assert!(player.feet.distance(goal)>42. && (player.feet-goal).truncate().length()<42.);
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut inputs=VecDeque::from(vec![Controls::default();delay]);
            for _ in 0..600 {
                let view=views.pop_front().unwrap();
                let (wish,_) = landing_control(&world,&view,goal,2.*delay as f32/120.,8.)
                    .expect("Nearby supported approach incorrectly needs full path search");
                inputs.push_back(Controls {wish,..Default::default()});
                player.tick(&world,inputs.pop_front().unwrap());
                assert!(player.grounded && player.feet.z.abs()<0.1);
                views.push_back(player.clone());
            }
            assert!(landing_control(&world,&player,goal,0.,8.).unwrap().1);
            assert!((player.feet-goal).truncate().length()<8.);
        }
        let player=Player::spawn(&world,vec3(28.4505,-15.4922,
            crate::movement::EYE_HEIGHT+0.03125)).unwrap();
        let wall=World::fixture(&[floor,(vec3(5.,-100.,0.),vec3(6.,100.,100.))]);
        assert!(landing_control(&wall,&player,goal,0.,8.).is_none(),"Approach crosses a wall");
        let edge=World::fixture(&[(vec3(20.,-100.,-100.),vec3(100.,100.,0.))]);
        assert!(landing_control(&edge,&player,goal,0.,8.).is_none(),"Approach leaves support");
        assert!(landing_control(&world,&player,Vec3::Z*36.,0.,8.).is_none(),"Height bound widened");
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
        airborne.velocity.z=200.;
        assert!(landing_control(&world,&airborne,goal,0.2,8.).is_none());
        airborne.velocity.z=100.;
        assert!(landing_control(&world,&airborne,goal,0.2,8.).is_some());
        airborne.updraft_time=1.;
        assert!(landing_control(&world,&airborne,goal,0.2,8.).is_none());
    }
    #[test]
    fn shortened_route_brakes_on_its_intermediate_landing_before_replanning() {
        let world=World::fixture(&[(vec3(-48.,-48.,-100.),vec3(48.,48.,0.))]);
        let intermediate=vec3(0.,0.,0.03125);
        let destination=vec3(240.,0.,96.);
        for (delay,settle_intermediate) in [(0,false),(0,true),(6,true),(12,true),(18,true)] {
            let mut player=Player::new(vec3(-51.58984,-63.51709,73.33676));
            player.velocity=vec3(226.7417,173.23021,206.6663);
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls{wish:Vec2::Y,run:true,..Default::default()};delay]);
            let mut handed_off=false;let mut fell=false;let mut settled=false;
            for tick in 0..720 {
                let view=views.pop_front().unwrap();let lead=2.*delay as f32/120.;
                assert!(landing_control(&world,&view,destination,lead,8.).is_none(),
                    "An intermediate landing cannot establish final waypoint arrival");
                let landing=settle_intermediate.then(||landing_control(&world,&view,intermediate,lead,8.)).flatten();
                let control=if let Some((wish,reached))=landing {
                    handed_off=true;settled|=reached;Controls{wish,..Default::default()}
                } else if handed_off {Controls::default()}
                else {Controls{wish:if tick<24 {vec2(-0.70710677,0.70710677)} else {-Vec2::X},run:true,..Default::default()}};
                pending.push_back(control);player.tick(&world,pending.pop_front().unwrap());
                if player.feet.z< -0.1 {fell=true;break;}
                views.push_back(player.clone());
            }
            if settle_intermediate {assert!(!fell && settled && player.feet.distance(intermediate)<8.,
                "Intermediate landing failed at delay{delay}: {:?}",player.feet);}
            else {assert!(fell,"Missing intermediate steering must reproduce the fall");}
        }
    }
    #[test]
    fn delayed_corridor_landing_brakes_before_the_snapshot_descends() {
        let world=World::fixture(&[(vec3(-48.,-48.,-100.),vec3(48.,48.,0.))]);
        let goal=vec3(0.,0.,0.03125);
        // The failed Red corridor approach, relative to its landing waypoint.
        // Before its delayed snapshot shows descent, its actual body has
        // already crossed much of the small landing with northward momentum.
        for (delay,legacy) in [(18,true),(0,false),(6,false),(12,false),(18,false)] {
            let mut player=Player::new(vec3(-51.58984,-63.51709,73.33676));
            player.velocity=vec3(226.7417,173.23021,206.6663);
            let mut views=VecDeque::from(vec![player.clone();delay+1]);
            let mut pending=VecDeque::from(vec![Controls{wish:Vec2::Y,run:true,..Default::default()};delay]);
            let mut handed_off=false;let mut fell=false;
            for tick in 0..720 {
                let view=views.pop_front().unwrap();
                let landing=(!legacy || view.velocity.z<0.).then(||
                    landing_control(&world,&view,goal,2.*delay as f32/120.,8.)).flatten();
                let control=if let Some((wish,_))=landing {
                    handed_off=true;Controls{wish,..Default::default()}
                } else if handed_off {Controls::default()}
                else {Controls{wish:if tick<24 {vec2(-0.70710677,0.70710677)} else {-Vec2::X},run:true,..Default::default()}};
                pending.push_back(control);player.tick(&world,pending.pop_front().unwrap());
                if player.feet.z< -0.1 {fell=true;break;}
                views.push_back(player.clone());
            }
            if legacy {assert!(fell,"Captured delayed landing should reproduce the old late-braking fall: {:?}",player.feet);}
            else {assert!(!fell && handed_off && landing_control(&world,&player,goal,0.,8.).is_some_and(|(_,settled)|settled),
                "Corridor landing did not settle delay{delay}: {:?}",player.feet);}
        }
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
        // Consume the expired input, but never skip the next unsampled jump
        // or compress its interval while recovering from a long frame.
        let after_stall = start + Duration::from_millis(550);
        assert!(clock.advance(after_stall));
        assert!(!clock.advance(after_stall), "An unsampled jump step was discarded");
        assert!(!clock.advance(after_stall + Duration::from_millis(100)));
        assert!(clock.advance(after_stall + Duration::from_millis(200)));
        assert!(!clock.advance(after_stall + Duration::from_millis(200)));
    }
    #[test]
    fn a_stalled_guest_issues_an_expired_takeoff_on_its_first_new_frame() {
        let start=Instant::now();
        let mut clock=NavigationClock::new(start);
        let resumed=start+Duration::from_millis(300);
        assert!(clock.advance(resumed),"Discarding stall time holds the run-up past the ledge");
        assert!(!clock.advance(resumed),"The next input must be sampled before advancing again");
        assert!(!clock.advance(resumed+Duration::from_millis(100)));
        assert!(clock.advance(resumed+Duration::from_millis(200)));
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
        let release = RopeCommand::Release { goal:Vec2::X*300., jump:true }; used=false;
        for _ in 0..30 { Driver::neutral(&mut keys); release.sample(&player,&world,&mut keys,&mut jumped,&mut used); }
        assert_eq!(keys.uses,2);
        assert_eq!(keys.rise,1.);
        player.rope=None; player.grounded=false;
        Driver::neutral(&mut keys);
        assert!(!release.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        assert!(keys.run && keys.wish.x>0.);
        assert_eq!(keys.rise,0.);
        player.grounded=true; Driver::neutral(&mut keys);
        assert!(release.sample(&player,&world,&mut keys,&mut jumped,&mut used).1);
        assert_eq!(keys.wish,Vec2::ZERO);
    }
    #[test]
    fn rope_jump_off_reaches_a_higher_landing_using_native_controls() {
        let mut world=World::fixture(&[
            (vec3(-1000.,-1000.,-192.),vec3(1000.,1000.,-128.)),
            (vec3(80.,-100.,-128.),vec3(400.,100.,64.))]);
        // Begin directly below the strand, as in the captured Checkmate run.
        // Establish outward momentum with ordinary swing input before letting
        // go. Jump-off must clear the upper landing; an ordinary drop cannot.
        let anchor=Vec3::Z*322.;
        world.traversal.ropes.push(crate::traversal::Rope{id:crate::entity::Id(1),model:1,enabled:true,
            origin:Vec3::ZERO,anchor,length:400.});
        for jump in [false,true] {
            let json=if jump {serde_json::json!({"action":"release","goal":[200,0],"jump":true})}
                else {serde_json::json!({"action":"release","goal":[200,0]})};
            let release: RopeCommand=serde_json::from_value(json).unwrap();release.validate().unwrap();
            let mut player=Player::new(Vec3::ZERO);
            player.rope=Some(crate::movement::RopeGrip{id:crate::entity::Id(1),length:(anchor-Vec3::Z*40.).length(),rise:1.});
            let (mut jumped,mut used)=(false,false);let mut keys=Input::default();let mut landed=false;
            let swing: RopeCommand=serde_json::from_value(serde_json::json!({
                "action":"climb","rise":0,"wish":[1,0]})).unwrap();swing.validate().unwrap();
            for _ in 0..120 {
                Driver::neutral(&mut keys);swing.sample(&player,&world,&mut keys,&mut jumped,&mut used);
                player.tick(&world,Controls{wish:keys.wish,rise:keys.rise,..Default::default()});
                if player.feet.x>12. {break;}
            }
            assert!(player.rope.is_some() && player.feet.x>12. && player.velocity.x>0.);
            for _ in 0..240 {
                let old_uses=keys.uses;Driver::neutral(&mut keys);
                release.sample(&player,&world,&mut keys,&mut jumped,&mut used);
                player.tick(&world,Controls{wish:keys.wish,run:keys.run,rise:keys.rise,
                    use_pressed:keys.uses!=old_uses,..Default::default()});
                assert!(world.body_clear(player.feet));
                landed|=player.grounded && player.feet.z>=64. && player.feet.x>80.;
            }
            assert_eq!(keys.uses,1,"Repeated release edge");
            assert_eq!(landed,jump,"Jump-off {jump}: final feet {:?}",player.feet);
        }
    }
    #[test]
    fn surface_goals_are_opt_in_straight_walks() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":[1920.,-3400.,0.],
            "navigate":false,"attack":true,"capture":false,"finish":false});
        let old:Command=serde_json::from_value(base.clone()).unwrap();
        assert!(old.surface.is_none() && old.validate().is_ok());
        let mut projected=base;projected["surface"]=460.into();
        assert!(serde_json::from_value::<Command>(projected.clone()).unwrap().validate().is_ok());
        for flag in ["navigate","horizontal","jump","hold_goal","swim","checkpoint","finish"] {
            let mut bad=projected.clone();bad[flag]=true.into();
            assert!(serde_json::from_value::<Command>(bad).unwrap().validate().is_err(),"{flag}");
        }
        projected["goal"]=serde_json::Value::Null;
        assert!(serde_json::from_value::<Command>(projected).unwrap().validate().is_err());
    }
    #[test]
    fn funhouse_exit_is_opt_in_and_cannot_mix_route_commands() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":null,
            "navigate":false,"attack":true,"capture":false,"finish":false});
        assert!(!serde_json::from_value::<Command>(base.clone()).unwrap().funhouse_exit);
        assert!(!serde_json::from_value::<Command>(base.clone()).unwrap().funhouse_pendulums);
        for course in ["funhouse_exit","funhouse_pendulums"] {
        let mut active=base.clone();active[course]=true.into();
        assert!(serde_json::from_value::<Command>(active.clone()).unwrap().validate().is_ok());
        let mut both=active.clone();both["funhouse_exit"]=true.into();both["funhouse_pendulums"]=true.into();
        assert!(serde_json::from_value::<Command>(both).unwrap().validate().is_err());
        for flag in ["navigate","horizontal","jump","run","hold_goal","swim","checkpoint","finish"] {
            let mut bad=active.clone();bad[flag]=true.into();
            assert!(serde_json::from_value::<Command>(bad).unwrap().validate().is_err(),"{flag}");
        }
        for (key,value) in [("goal",serde_json::json!([1920,-3820,150])),("surface",serde_json::json!(460)),
            ("speed_limit",serde_json::json!(100)),("chess_follow",serde_json::json!("Blue"))] {
            let mut bad=active.clone();bad[key]=value;
            assert!(serde_json::from_value::<Command>(bad).unwrap().validate().is_err(),"{key}");
        }
        }
    }
    #[test]
    fn pathnodes_resolve_to_supported_feet_and_reject_other_entities() {
        let world=World::fixture(&[(vec3(-100.,-100.,-100.),vec3(100.,100.,0.))]);
        let mut entity=crate::levels::Entity::from([
            ("classname".into(),"info_pathnode".into()),("origin".into(),"8 12 20".into())]);
        let (origin,feet)=pathnode_goal(&world,&entity).unwrap();
        assert_eq!(origin,vec3(8.,12.,20.));
        assert!(feet.distance(vec3(8.,12.,0.03125))<0.001 && world.body_clear(feet));
        entity.insert("classname".into(),"info_null".into());
        assert!(pathnode_goal(&world,&entity).is_err());
        entity.insert("classname".into(),"info_pathnode".into());
        for value in ["500 500 20","8 12 1000","NaN 12 20","bad"] {
            entity.insert("origin".into(),value.into());
            assert!(pathnode_goal(&world,&entity).is_err(),"{value}");
        }
    }
    #[test]
    fn pathnode_observation_cannot_supply_movement_or_progression_input() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":null,"navigate":false,
            "attack":false,"capture":false,"finish":false,"pathnode":712});
        assert!(serde_json::from_value::<Command>(base.clone()).unwrap().validate().is_ok());
        for (key,value) in [("goal",serde_json::json!([0,0,0])),("marker",1.into()),("pathnode",100000.into()),
            ("attack",true.into()),("navigate",true.into()),("run",true.into()),("jump",true.into()),
            ("checkpoint",true.into()),("skip_scene",true.into()),("finish",true.into()),
            ("use_aim",serde_json::json!([0,0])),("chess_follow","Blue".into())] {
            let mut bad=base.clone();bad[key]=value;
            assert!(serde_json::from_value::<Command>(bad).unwrap().validate().is_err(),"{key}");
        }
    }
    #[test]
    fn watch_input_cannot_mix_with_navigation_checkpoint_or_combat_commands() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":null,
            "navigate":false,"attack":false,"capture":false,"finish":false,"use_watch":true});
        assert!(serde_json::from_value::<Command>(base.clone()).unwrap().validate().is_ok());
        for (field,value) in [("goal",serde_json::json!([0,0,0])),("marker",1.into()),("pathnode",1.into()),
            ("attack",true.into()),("navigate",true.into()),("run",true.into()),("jump",true.into()),
            ("checkpoint",true.into()),("skip_scene",true.into()),("finish",true.into()),
            ("use_aim",serde_json::json!([0,0])),("chess_follow","Blue".into())] {
            let mut mixed=base.clone();mixed[field]=value;
            assert!(serde_json::from_value::<Command>(mixed).unwrap().validate().is_err(),"{field}");
        }
    }
    #[test]
    fn legacy_route_commands_remain_valid_and_rope_inputs_are_bounded() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":null,"navigate":false,"attack":true,"capture":false,"finish":false});
        let old:Command=serde_json::from_value(base.clone()).unwrap(); assert!(old.validate().is_ok());
        assert!(old.rope.is_none() && old.marker.is_none() && old.pathnode.is_none() && !old.horizontal && !old.run && !old.jump);
        assert!(old.use_aim.is_none() && !old.watch_dialogue && !old.use_watch && old.beyond_step.is_none());
        assert!(old.speed_limit.is_none() && !old.prefer_cards && old.temple_follow.is_none() && old.garden_follow.is_none());
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
        let mut running=base.clone(); running["run"]=true.into(); running["navigate"]=true.into();
        assert!(serde_json::from_value::<Command>(running).unwrap().validate().is_ok());
        for invalid in [serde_json::json!({"action":"climb","rise":2}),
            serde_json::json!({"action":"climb","rise":0,"wish":[2,0]}),
            serde_json::json!({"action":"release","goal":[1e9,0]})] {
            let mut input=base.clone(); input["rope"]=invalid;
            assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
        }
        let mut input=base; input["goal"]=serde_json::json!([1,2,3]); input["marker"]=serde_json::json!(598);
        assert!(serde_json::from_value::<Command>(input).unwrap().validate().is_err());
    }
    #[test]
    fn level_courses_cannot_be_combined_with_other_route_actions() {
        let base=serde_json::json!({"id":1,"heartbeat":1,"epoch":1,"goal":null,
            "navigate":false,"attack":false,"capture":false,"finish":false});
        for field in ["temple_follow", "garden_follow", "sanctum_follow", "chess_follow", "castling_follow"] { for first in ["Blue","Red"] {
            let mut swim=base.clone(); swim[field]=first.into();
            if field=="chess_follow" {swim["chess_step"]=0.into();}
            if field=="castling_follow" {swim["castling_step"]=0.into();}
            assert!(serde_json::from_value::<Command>(swim.clone()).unwrap().validate().is_ok());
            for (field,value) in [("goal",serde_json::json!([0,0,0])),("marker",1.into()),
                ("attack",true.into()),("navigate",true.into()),("checkpoint",true.into()),
                ("finish",true.into()),("beyond_step",1.into()),("speed_limit",80.into())] {
                let mut invalid=swim.clone(); invalid[field]=value;
                assert!(serde_json::from_value::<Command>(invalid).unwrap().validate().is_err());
            }
        }
        }
        for field in ["temple_follow", "garden_follow", "sanctum_follow", "chess_follow", "castling_follow"] {
            let mut invalid=base.clone(); invalid[field]="Green".into();
            assert!(serde_json::from_value::<Command>(invalid).is_err());
        }
        for fields in [["temple_follow","garden_follow"],["temple_follow","sanctum_follow"],["garden_follow","sanctum_follow"]] {
            let mut both=base.clone();
            for field in fields {both[field]="Blue".into();}
            assert!(serde_json::from_value::<Command>(both).unwrap().validate().is_err());
        }
        for invalid in [serde_json::json!({"chess_follow":"Blue"}),serde_json::json!({"chess_step":0}),
            serde_json::json!({"chess_follow":"Blue","chess_step":1000}),
            serde_json::json!({"chess_follow":"Blue","chess_step":0,"garden_follow":"Red"}),
            serde_json::json!({"castling_follow":"Blue"}),serde_json::json!({"castling_step":0}),
            serde_json::json!({"castling_follow":"Blue","castling_step":203}),
            serde_json::json!({"castling_follow":"Blue","castling_step":0,"chess_follow":"Red","chess_step":0})] {
            let mut command=base.clone();
            for (key,value) in invalid.as_object().unwrap() {command[key]=value.clone();}
            assert!(serde_json::from_value::<Command>(command).unwrap().validate().is_err());
        }
    }
}
