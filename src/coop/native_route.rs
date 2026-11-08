//! Accelerated ordinary-input campaign diagnostics. Never seeds gameplay state.
use super::{arena::Arena, protocol::{Input, Visit}, world::Roster};
use crate::{assets::Assets, bsp::Bsp, collision::{World, PLAYER_CENTER, PLAYER_HALF},
    inventory::Stats, movement::{Controls, FIXED_DT}, route::{Route, fight}};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
mod swim;
mod machine;
mod course;
mod trail;
pub(crate) use course::{Step as CourseStep,Clock as CourseClock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SwimStep { pub at: [f32; 3], pub planned: bool }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Command {
    Node(usize), Nav([f32; 3]), Clear(f32), Wait(u64), Watch, Staff(bool), Cards(bool),
    HighNode(usize), Walk([f32; 3]), Positions([Option<[f32; 3]>; 2]),
    Rise { center: [f32; 2], height: f32, landing: [f32; 3] },
    Board { thread: String, center: [f32; 2] },
    /// Follow an authored point's footing in single file; stop if a scene starts.
    Approach([f32; 3]), Interact([f32; 3]), Shoot(String),
    /// Ordinary steering for authored jumps and airborne course corrections.
    Steer { at: [f32; 3], mode: String, #[serde(default)] single_file: bool },
    /// Each player follows continuous updraft waypoints without mid-air waits.
    Flight { takeoff: [f32; 3], via: Vec<[f32; 3]>, landing: [f32; 3] },
    /// Let an automated survivor leave transient goals to find revival footing.
    ContinueWithSurvivor(bool),
    /// Hold position until a live controller clock enters an authored interval.
    Phase { controller: String, pointer: String, start: f64, end: f64 },
    Swim { at: [f32; 3], planned: bool },
    /// One swimmer clears a narrow sequence before the partner enters it.
    SwimFile(Vec<SwimStep>),
    /// Ordinary fixed-duration movement through a current.
    Drive { direction: [f32; 3], ticks: u64 },
    /// Wait for a gap in a fan rotating about Z, then swim above its blades.
    SwimFan { entity: String, controller: String, angle: String, speed: f32, height: f32 },
    WatchReady, WorldRunning,
    /// Normal jump/run boarding of a live rotating brush, then passive riding.
    MachineBoard { entity: usize, local: [f32; 3] },
    MachineRide { entity: usize, angle: f32, tolerance: f32 },
    /// Stay aboard until each living rider reaches a supported lift height.
    RideHeight(f32),
    Cycle { controller: String, pointer: String, phase: f64, period: f64, #[serde(default)] hold: bool },
    Ice(bool),
    Course(Vec<CourseStep>),
    /// Complete a narrow approach, jump and landing before the partner follows.
    CourseFile(Vec<CourseStep>),
    Pond(crate::levels::potears2::coop_route::Step),
    Pool(crate::pool::coop_route::Step),
    /// Follow supported map pathnodes, waiting for a visible companion when requested.
    Trail {at:[f32;3], #[serde(default)] escort:Option<String>, #[serde(default)] portal:bool,
        #[serde(default)] direct:bool, #[serde(default)] escort_player:Option<super::PlayerId>},
}
pub(crate) struct Spec {
    pub map: &'static str,
    pub entry: &'static str,
    pub exit: crate::level::spec::ExitSpec,
    pub commands: Vec<Command>,
    pub navigation: Navigation,
    pub verify: fn(&Arena, &[super::world::Body; 2]) -> Result<()>,
}
pub(crate) enum Navigation {
    /// Preserve the battlefield course's nearby-waypoint handoff.
    Nearby,
    /// Finish jumps before turning; leave room for a partner on narrow landings.
    LandedPair,
}
#[derive(Default)]
pub(crate) struct Walking {
    goal: Option<Vec3>,
    steps: VecDeque<(Controls, Vec3)>,
    age: u32,
    settling: u32,
}
impl Walking {
    pub(crate) fn input(&mut self, world: &World, player: &crate::movement::Player, goal: Vec3, close: f32, require_landing: bool) -> Result<Controls> {
        if self.goal.is_none_or(|old| old.distance(goal) > 1.) {
            *self = Self { goal: Some(goal), ..Default::default() };
        }
        if (!require_landing || player.grounded) && player.feet.distance(goal) < close { self.steps.clear(); return Ok(Controls::default()); }
        if self.settling>0 && !player.grounded {
            self.settling-=1;
            return Ok(Controls{jump:player.ledge.as_ref().is_some_and(|h|!h.pulling),..Default::default()});
        }
        self.settling=0;
        if self.steps.is_empty() {
            println!("PLAN {:?} to {goal:?} grounded={} ledge={}",player.feet,player.grounded,player.ledge.is_some());
            let plan=Route::plan(world, player, goal, close * 0.7)
                .or_else(|_| Route::plan(world, player, goal, close * 0.9));
            self.steps=match plan {
                Ok(plan)=>plan.into(),
                Err(_) if !player.grounded=>{
                    self.settling=24;
                    return Ok(Controls{jump:player.ledge.as_ref().is_some_and(|h|!h.pulling),..Default::default()});
                }
                Err(n)=>anyhow::bail!("Path exhausted {n} states from {:?} to {goal:?}",player.feet),
            };
            self.age = 0;
        }
        Ok(self.steps.front().map_or(Controls::default(), |(c,_)|
            Controls { jump: c.jump && self.age == 0, ..*c }))
    }
    pub(crate) fn advanced(&mut self, player: &crate::movement::Player) {
        if self.steps.is_empty() { return; }
        self.age += 1;
        if self.age == 24 {
            if self.steps.pop_front().is_some_and(|(_, at)| player.feet.distance(at) > 8.) { self.steps.clear(); }
            self.age = 0;
        }
    }
}
fn targets(arena: &Arena) -> Vec<crate::combat::Target> {
    let mut targets = arena.npcs.targets();
    if let Some(e) = &arena.stage.interactions.encounters { targets.extend(e.targets()); }
    if let Some(s) = &arena.stage.interactions.school2 { targets.extend(s.targets()); }
    if let Some(p) = &arena.stage.interactions.pool { targets.extend(p.targets()); }
    targets.extend(arena.stage.interactions.duchess.as_ref().and_then(|d| d.target()));
    targets.extend(crate::level::targets(&arena.stage.interactions.levels));
    targets.retain(|t| crate::level::hit_owner(&arena.stage.interactions.levels, t.id)
        .is_none_or(|s| arena.stage.interactions.levels[s].ctl.hostile_target(t.id)));
    targets
}
pub(crate) fn check(a: &mut Assets, spec: Spec) -> crate::levels::BoxFuture<'_> {
    Box::pin(async move { run(a, spec).await })
}
async fn run(a: &mut Assets, spec: Spec) -> Result<()> {
    let require_landing=matches!(spec.navigation,Navigation::LandedPair);
    let commands = if let Some(path) = std::env::var_os("LOOKING_GLASS_COOP_NATIVE_ROUTE") {
        serde_json::from_slice::<Vec<Command>>(&std::fs::read(path)?)?
    } else { spec.commands };
    ensure!(!commands.is_empty() && commands.len() <= 2000, "Invalid route command count");
    let visit = Visit { epoch: 1, map: spec.map.into(), entry: Some(spec.entry.into()),
        difficulty: Default::default(), combat_preview: false };
    let map = Bsp::parse(&a.read(&format!("maps/{}.bsp", spec.map))?)?;
    let mut world = World::from_bsp(&map)?;
    let mut arena = Arena::load(a, &map, &mut world, &visit, false)?;
    let (eye,yaw) = crate::interaction::spawn(&map, visit.entry.as_deref());
    let mut roster = Roster::new_with(&world, eye, yaw, Stats::for_level(&visit.map, visit.entry.as_deref()),
        |at| !arena.stage.interactions.hazardous(at))?;
    arena.reset_characters(&roster.players);
    let mut guest_world = World::from_bsp(&map)?;
    let mut guest = Arena::load(a, &map, &mut guest_world, &visit, false)?;
    let mut walking: [Walking; 2] = Default::default();
    let mut pond: [crate::levels::potears2::coop_route::Driver;2]=Default::default();
    let mut pool: [crate::pool::coop_route::Driver;2]=Default::default();
    let mut swimming: [swim::Swimming; 2] = Default::default();
    let mut trackers: [fight::Tracker; 2] = Default::default();
    let mut step = 0;
    let mut step_ticks = 0;
    let mut home = roster.players.each_ref().map(|b| b.player.feet);
    let mut quiet = 0;
    let mut staff = true;
    let mut cards = false;
    let mut ice = false;
    let mut machine = machine::Machine::default();
    let mut ride_done = [false; 2];
    let mut course: [course::Walker; 2] = Default::default();
    let mut course_machines: [machine::Machine; 2] = Default::default();
    let mut trails:[trail::Walker;2]=Default::default();
    let mut continue_with_survivor = false;
    let mut history = VecDeque::new();
    let mut completed = Vec::new();
    let mut life_events = Vec::new();
    let mut scene_starts = Vec::new();
    let mut rising = [false; 2];
    let mut steam_seen = [false; 2];
    let mut rise_done = [false; 2];
    let mut board_lead = None;
    let mut board_wait_ticks = 0;
    let mut reunions = 0;
    let mut samples = 0;
    let mut arrival = None;
    let mut shot_started = false;
    let mut approach_lead = None;
    let mut approach_stuck = 0;
    let mut approach_yields = 0;
    let mut jumped = [false; 2];
    let mut flight = [0; 2];
    let mut fan_started = [false; 2];
    let mut swim_leg = [0; 2];
    std::fs::create_dir_all(crate::preferences::path(""))?;
    let recovery_name=format!("{}-recovery.jsonl",spec.map);
    let mut recovery=super::recovery_trace::Trace::new(crate::preferences::path(&recovery_name))?;
    recovery.record(&roster,&world,1,spec.map,arena.stage.busy(),false,|at|!arena.stage.interactions.hazardous(at))?;
    let result: Result<()> = async {
        for tick in 0..120*1800 {
            let command = commands.get(step).context("Route commands ended before the actual exit")?;
            let busy = arena.stage.busy();
            let old_owner = arena.stage.owner;
            let before = roster.players.each_ref().map(|b| b.player.feet);
            let alive = roster.players.each_ref().map(|b| b.stats.alive());
            let spawned = roster.players.each_ref().map(|b| b.spawned_at);
            let phase_ready=if let Command::Phase{controller,pointer,start,end}=command {
                let slot=arena.stage.interactions.levels.iter().find(|s|s.reg.id==controller)
                    .context("Missing route phase controller")?;
                let state=slot.ctl.snapshot();
                let value=state.pointer(pointer).and_then(|v|v.as_f64()).context("Missing route phase clock")?;
                value>=*start && value<*end
            } else if let Command::Cycle{controller,pointer,phase,period,..}=command {
                ensure!(period.is_finite() && *period>0.,"Invalid route machine period");
                let slot=arena.stage.interactions.levels.iter().find(|s|s.reg.id==controller)
                    .context("Missing cyclic route controller")?;
                let state=slot.ctl.snapshot();
                let value=state.pointer(pointer).and_then(|v|v.as_f64()).context("Missing machine clock")?;
                (value.rem_euclid(*period)-phase).abs()<f64::from(FIXED_DT)*0.75
            } else {false};
            let machine_pose=match command {
                Command::MachineBoard{entity,..}|Command::MachineRide{entity,..}=>Some(machine.sample(&map,&arena,*entity)?),
                _=>None,
            };
            let fan_angle=if let Command::SwimFan{controller,angle,..}=command {
                let slot=arena.stage.interactions.levels.iter().find(|s|s.reg.id==controller)
                    .context("Missing swim fan controller")?;
                Some(slot.ctl.snapshot().pointer(angle).and_then(|v|v.as_f64())
                    .context("Missing swim fan angle")? as f32)
            } else {None};
            let mut all = targets(&arena);
            let switch=if let Command::Shoot(thread)=command {
                let target=arena.stage.interactions.switch_target(thread);
                if !busy && !shot_started {
                    ensure!(target.is_some(),"Switch missing before ordinary shot: {thread}");
                    shot_started=true;
                }
                all.extend(target);
                target
            } else {None};
            let context = crate::combat::Context { world: &world, targets: &all };
            let goal = match command {
                Command::Node(id) | Command::HighNode(id) => {
                    let e = map.entities.get(*id).context("Missing route entity")?;
                    let raw = crate::interaction::vector(e.get("origin").context("Route entity has no origin")?)
                        .context("Invalid route entity origin")?;
                    let at = if matches!(command,Command::HighNode(_)) {
                        [32.,64.,128.].into_iter().find_map(|h|
                            world.actor_footing(raw+Vec3::Z*h,PLAYER_CENTER,PLAYER_HALF,256.))
                    } else {world.actor_footing(raw,PLAYER_CENTER,PLAYER_HALF,180.)};
                    Some(at
                        .with_context(|| format!("Unsupported route entity {id}"))?)
                }
                Command::Nav(v) => Some(Vec3::from_array(*v)), _ => None,
            };
            let goal=if let Command::Approach(point)=command {
                Some([0.,16.,32.,64.,128.,256.,512.].into_iter().find_map(|z|
                    world.actor_footing(Vec3::from_array(*point)+Vec3::Z*z,PLAYER_CENTER,PLAYER_HALF,512.))
                    .with_context(||format!("Unsupported approach {point:?}"))?)
            } else {goal};
            if matches!(command,Command::Approach(_)) && !busy {
                if approach_lead.is_none_or(|i:usize|!roster.players[i].stats.alive()) {
                    approach_lead=roster.players.iter().enumerate().filter(|(_,b)|b.stats.alive())
                        .min_by(|(_,a),(_,b)|a.player.feet.distance_squared(goal.unwrap())
                            .total_cmp(&b.player.feet.distance_squared(goal.unwrap()))).map(|(i,_)|i);
                }
            }
            if let Command::Steer{at,single_file:true,..}=command {
                if approach_lead.is_none_or(|i:usize|!roster.players[i].stats.alive()) {
                    approach_lead=roster.players.iter().enumerate().filter(|(_,b)|b.stats.alive())
                        .min_by(|(_,a),(_,b)|a.player.feet.distance_squared(Vec3::from_array(*at))
                            .total_cmp(&b.player.feet.distance_squared(Vec3::from_array(*at)))).map(|(i,_)|i);
                }
            }
            if let Command::SwimFile(steps)=command {
                let first=steps.first().context("Empty single-file swim course")?;
                if approach_lead.is_none_or(|i:usize|!roster.players[i].stats.alive()) {
                    approach_lead=roster.players.iter().enumerate().filter(|(_,b)|b.stats.alive())
                        .min_by(|(_,a),(_,b)|a.player.feet.distance_squared(Vec3::from_array(first.at))
                            .total_cmp(&b.player.feet.distance_squared(Vec3::from_array(first.at)))).map(|(i,_)|i);
                }
            }
            if let Command::CourseFile(steps)=command {
                let first=steps.iter().find(|s|!s.mode.starts_with("drive") && s.mode!="pond_board")
                    .or_else(||steps.first()).context("Empty single-file walking course")?;
                if approach_lead.is_none_or(|i:usize|!roster.players[i].stats.alive()) {
                    approach_lead=roster.players.iter().enumerate().filter(|(_,b)|b.stats.alive())
                        .min_by(|(_,a),(_,b)|a.player.feet.distance_squared(Vec3::from_array(first.at))
                            .total_cmp(&b.player.feet.distance_squared(Vec3::from_array(first.at)))).map(|(i,_)|i);
                }
            }
            let mut inputs = [(Controls::default(), Input::default()); 2];
            let mut planned = [false; 2];
            let mut work = false;
            let mut walk_touch = false;
            let mut nav_touch = false;
            let mut used = false;
            let user=if let Command::Interact(point)=command {
                roster.players.iter().enumerate().filter(|(_,b)|b.stats.alive())
                    .filter_map(|(i,b)| {
                        let delta=Vec3::from_array(*point)-b.player.eye();
                        arena.stage.interactions.prompt(&world,b.player.eye(),delta.normalize_or_zero())
                            .map(|_|(i,delta.length_squared()))
                    }).min_by(|a,b|a.1.total_cmp(&b.1)).map(|(i,_)|i)
            } else {None};
            let board_ready = if let Command::Board{thread,..}=command {
                roster.players.each_ref().map(|b| arena.stage.interactions.levels.iter()
                    .any(|s|s.ctl.coop_boarding_ready(thread,&[b.player.feet])==Some(true)))
            } else {[false;2]};
            if board_ready[0] {board_lead.get_or_insert(roster.tick);}
            if !busy && board_ready[0] && !board_ready[1] {board_wait_ticks+=1;}
            for i in 0..2 {
                let body = &roster.players[i];
                let p = &body.player;
                inputs[i].1.yaw = body.yaw; inputs[i].1.pitch = body.pitch;
                if busy || !body.stats.alive() { continue; }
                trackers[i].observe(&all, FIXED_DT);
                let radius = if let Command::Clear(radius) = command { *radius } else { 1000. };
                let foes = all.iter().copied().filter(|t|switch.is_none_or(|s|s.id!=t.id)
                    && t.center.distance(p.eye()) <= radius).collect::<Vec<_>>();
                let mut attack = fight::plan(&context, p.eye(), &foes, &trackers[i],
                    cards && body.stats.copies(1)>0 && body.stats.will()>=fight::CARD_COST);
                let visible = foes.iter().filter_map(|t| fight::aim_point(&context, p.eye(), t).map(|at|(t,at)))
                    .min_by(|(_,a),(_,b)| a.distance_squared(p.eye()).total_cmp(&b.distance_squared(p.eye())));
                if staff && body.stats.copies(7)>0 && body.stats.will()>21. {
                    if let Some((target,point)) = visible {
                        attack = fight::Plan { target:Some(target.id), aim:(point-p.eye()).normalize_or_zero(), weapon:7, alternate:false };
                    }
                }
                if ice && body.stats.copies(4)>0 && body.stats.will()>=0.75 && p.immersion.level==0 {
                    if let Some((target,point))=visible.filter(|(_,at)|at.distance(p.eye())<360.) {
                        attack=fight::Plan{target:Some(target.id),aim:(point-p.eye()).normalize_or_zero(),weapon:4,alternate:false};
                    }
                }
                if attack.target.is_some() {
                    inputs[i].1.weapon=Some(attack.weapon as u8);
                    inputs[i].1.primary=!attack.alternate; inputs[i].1.alternate=attack.alternate;
                    inputs[i].1.yaw=attack.aim.y.atan2(attack.aim.x); inputs[i].1.pitch=attack.aim.z.asin().clamp(-1.56,1.56);
                }
                let personal_goal = if let Command::Positions(goals)=command {goals[i].map(Vec3::from_array)} else {goal};
                if let Some(goal) = personal_goal {
                    let follow=approach_lead.is_none_or(|lead|lead==i || {
                        let leader=&roster.players[lead].player;
                        leader.grounded && leader.feet.distance(goal)<38.
                    });
                    if follow {
                        inputs[i].0=walking[i].input(&world,p,goal,38.,require_landing)?;planned[i]=true;
                    }
                    // One body reaches the shared marker while the other keeps
                    // normal personal space nearby. Separate personal goals
                    // retain their own arrival bounds.
                    let landed=!require_landing || p.grounded;
                    nav_touch |= landed && p.feet.distance(goal)<38.;
                    let nearby=if matches!(command,Command::Positions(_)) || !require_landing {48.} else {80.};
                    work |= !landed || p.feet.distance(goal)>=nearby;
                } else if let Command::Trail{at,escort,portal,direct,escort_player}=command {
                    // A narrow portal takes one body at a time. Keep the
                    // following player outside until the first actually warps.
                    if *portal && i==1 && !trails[0].done {work=true;continue;}
                    let companion=escort.as_ref().filter(|_|escort_player.is_none_or(|id|id.index()==i))
                        .and_then(|name|arena.npcs.companions().into_iter()
                        .find(|c|&c.name==name && c.visible));
                    let partner_done=trails[1-i].done;
                    let exit=commands.get(step+1).and_then(|next|if let Command::Trail{at,..}=next {Some(Vec3::from_array(*at))}else{None});
                    inputs[i].0=trails[i].input(&map,&world,body,Vec3::from_array(*at),companion.as_ref(),*portal,*direct,partner_done,exit)?;
                    work|=!trails[i].done;
                } else if let Command::Course(steps)|Command::CourseFile(steps)=command {
                    if let Some(lead)=approach_lead.filter(|_|matches!(command,Command::CourseFile(_))) {
                        if i!=lead && course[lead].cursor<steps.len() {work=true;continue;}
                    }
                    let partner_done=course[1-i].cursor>=steps.len();
                    let clock=steps.get(course[i].cursor).and_then(|s|s.clock.as_ref())
                        .map(|clock|clock.ready(&arena)).transpose()?;
                    let boarding=steps.get(course[i].cursor).filter(|s|matches!(s.mode.as_str(),"board"|"pond_board"))
                        .map(|s|course_machines[i].sample(&map,&arena,s.ticks as usize)).transpose()?;
                    if let Some(leg)=steps.get(course[i].cursor).filter(|s|s.mode=="nav") {
                        let goal=Vec3::from_array(leg.at);
                        let close=if course[i].cursor+1==steps.len() && partner_done {70.}else{32.};
                        if p.grounded && p.feet.distance(goal)<close {course[i].cursor+=1;}
                        else {inputs[i].0=walking[i].input(&world,p,goal,close,true)?;planned[i]=true;}
                    } else {inputs[i].0=course[i].input(p,steps,partner_done,clock,boarding.as_ref());}
                    work|=course[i].cursor<steps.len();
                } else if let Command::Pond(command)=command {
                    let (control,done)=pond[i].input(&arena,&world,&roster.players,i,command,step_ticks)?;
                    inputs[i].0=control;work|=!done;
                } else if let Command::Pool(command)=command {
                    let (control,done)=pool[i].input(&arena,&world,&roster.players,i,command)?;
                    inputs[i].0=control;work|=!done;
                } else if let Command::RideHeight(height)=command {
                    ride_done[i]|=p.grounded && p.feet.z>=*height;
                    work|=!ride_done[i];
                } else if let Command::MachineBoard{local,..}=command {
                    let pose=machine_pose.as_ref().unwrap();
                    if !pose.supports(p) {
                        let local=Vec3::from_array(*local);
                        let tangent=vec3(-local.y,local.x,0.).normalize_or_zero();
                        let at=pose.center+pose.rotation*(local+tangent*if i==0 {-22.}else{22.});
                        let jump=p.ledge.as_ref().is_some_and(|h|!h.pulling) || p.grounded && !jumped[i];
                        jumped[i]|=jump;
                        inputs[i].0=Controls{wish:(at-p.feet).truncate().normalize_or_zero(),run:true,jump,..Default::default()};
                        work=true;
                    }
                } else if let Command::MachineRide{angle,tolerance,..}=command {
                    let pose=machine_pose.as_ref().unwrap();
                    let d=p.feet-pose.center;
                    let delta=(d.y.atan2(d.x).to_degrees()-angle+180.).rem_euclid(360.)-180.;
                    ride_done[i]|=pose.supports(p) && delta.abs()<tolerance.max(0.5);
                    work|=!ride_done[i];
                } else if let Command::Swim{at,planned}=command {
                    let goal=Vec3::from_array(*at);
                    let lead=roster.players.iter().enumerate().filter(|(_,b)|b.stats.alive())
                        .min_by(|(_,a),(_,b)|a.player.feet.distance_squared(goal)
                            .total_cmp(&b.player.feet.distance_squared(goal))).map(|(i,_)|i)
                        .context("Swimming has no living player")?;
                    let radius=if i==lead {10.}else{64.};
                    inputs[i].0=swimming[i].input(&world,p,goal,*planned,radius)?;
                    work |= p.feet.distance(goal)>=radius;
                } else if let Command::SwimFile(steps)=command {
                    let lead=approach_lead.context("Single-file swim has no living leader")?;
                    if i!=lead && swim_leg[lead]<steps.len() {work=true;continue;}
                    if let Some(target)=steps.get(swim_leg[i]) {
                        let goal=Vec3::from_array(target.at);
                        let radius=if i!=lead && swim_leg[i]+1==steps.len() {64.}else{10.};
                        if p.feet.distance(goal)<radius {
                            swim_leg[i]+=1;swimming[i]=Default::default();
                        } else {
                            inputs[i].0=swimming[i].input(&world,p,goal,target.planned,radius)?;
                        }
                        work |= swim_leg[i]<steps.len();
                    }
                } else if let Command::Drive{direction,..}=command {
                    let direction=Vec3::from_array(*direction);
                    inputs[i].0=Controls{wish:direction.truncate(),swim:direction,rise:direction.z,
                        run:true,..Default::default()};
                } else if let Command::SwimFan{entity,speed,height,..}=command {
                    if p.feet.z<*height {
                        work=true;
                        if !fan_started[i] && step_ticks>=60 {
                            fan_started[i]=swim::fan_gap(&map,p,entity,fan_angle.unwrap(),*speed)?;
                        }
                        if fan_started[i] {inputs[i].0=Controls{swim:Vec3::Z,run:true,..Default::default()};}
                    }
                } else if let Command::Flight{takeoff,via,landing}=command {
                    let last=via.len()+1;
                    let point=|leg:usize|Vec3::from_array(if leg==0 {*takeoff}
                        else if leg==last {*landing} else {via[leg-1]});
                    let mut delta=point(flight[i])-p.feet;
                    let near=|d:Vec3,leg:usize|d.truncate().length()<22.
                        && d.z.abs()<64. && (leg>0 && leg<last || p.grounded || p.swimming);
                    if flight[i]<last && near(delta,flight[i]) {
                        flight[i]+=1;jumped[i]=false;delta=point(flight[i])-p.feet;
                    }
                    // Require one actual landing at the shared destination;
                    // the partner can settle beside it with ordinary spacing.
                    let partner=&roster.players[1-i].player;
                    let radius=if flight[i]==last && partner.grounded
                        && partner.feet.distance(point(last))<30. {70.}else{22.};
                    let done=flight[i]==last && p.grounded && delta.truncate().length()<radius && delta.z.abs()<64.;
                    if !done {
                        let jump=p.ledge.as_ref().is_some_and(|h|!h.pulling)
                            || p.grounded && (delta.z>24. || flight[i]>0 && !jumped[i]);
                        jumped[i]|=jump;
                        inputs[i].0=Controls{wish:(delta.truncate()*3.-p.velocity.truncate()*0.7).clamp_length_max(320.)/320.,
                            swim:delta.normalize_or_zero(),rise:delta.z.signum(),run:true,jump,..Default::default()};
                        work=true;
                    }
                } else if let Command::Steer{at,mode,single_file}=command {
                    let delta=Vec3::from_array(*at)-p.feet;
                    let lead=approach_lead.or_else(||roster.players.iter().enumerate().filter(|(_,b)|b.stats.alive())
                        .min_by(|(_,a),(_,b)|a.player.feet.distance_squared(Vec3::from_array(*at))
                            .total_cmp(&b.player.feet.distance_squared(Vec3::from_array(*at)))).map(|(i,_)|i))
                        .context("Steering has no living player")?;
                    let near=delta.truncate().length()<if i==lead {22.}else{70.}
                        && delta.z.abs()<64. && (matches!(mode.as_str(),"air"|"course_air") || p.grounded || p.swimming);
                    if !near {
                        work=true;
                        // Narrow banks can only fit one pull-up at a time.
                        // The other swimmer waits until the leader is ashore.
                        let leader=&roster.players[lead].player;
                        if *single_file && i!=lead && !(leader.grounded
                            && leader.feet.distance(Vec3::from_array(*at))<38.) {continue;}
                        let jump=p.ledge.as_ref().is_some_and(|h|!h.pulling)
                            || p.grounded && (mode=="hop" || matches!(mode.as_str(),"jump"|"air"|"course_air"|"precise") && (!jumped[i] || delta.z>24.));
                        jumped[i]|=jump;
                        inputs[i].0=Controls{wish:if matches!(mode.as_str(),"precise"|"air") || mode=="course_air" && delta.truncate().length()<30. {
                            (delta.truncate()*3.-p.velocity.truncate()*0.7).clamp_length_max(320.)/320.
                        } else {delta.truncate().normalize_or_zero()},swim:delta.normalize_or_zero(),rise:delta.z.signum(),run:true,jump,..Default::default()};
                        work=true;
                    }
                } else if matches!(command,Command::Phase{..}|Command::Cycle{hold:true,..}) {
                    inputs[i].0=Controls{wish:((home[i]-p.feet).truncate()*3.-p.velocity.truncate()*0.7)
                        .clamp_length_max(320.)/320.,run:true,..Default::default()};
                } else if let Command::Walk(at)=command {
                    let delta=Vec3::from_array(*at)-p.feet;
                    if delta.truncate().length()>=8. || delta.z.abs()>=35. {
                        inputs[i].0=Controls{wish:super::route_driver::straight_wish(delta.truncate(),p.velocity.truncate(),0.,false,None),..Default::default()};
                    }
                    walk_touch |= p.grounded && delta.truncate().length()<8. && delta.z.abs()<35.;
                    work |= !p.grounded || delta.truncate().length()>=48. || delta.z.abs()>=35.;
                } else if let Command::Rise{center,height,landing}=command {
                    rising[i] |= p.feet.z>=*height;
                    steam_seen[i] |= p.steam;
                    let landing=Vec3::from_array(*landing);
                    rise_done[i] |= rising[i] && p.grounded && p.feet.distance(landing)<45.;
                    if !rise_done[i] {
                        let center=Vec2::from_array(*center);
                        let angle=step_ticks as f32/120.*3.+i as f32*std::f32::consts::PI;
                        let aim=if rising[i] {landing.truncate()} else if p.steam {center+vec2(angle.cos(),angle.sin())*36.} else {center};
                        inputs[i].0=Controls{wish:((aim-p.feet.truncate())*0.06-p.velocity.truncate()*0.015).clamp_length_max(1.),
                            jump:!rising[i] && step_ticks%150==0,..Default::default()};
                    }
                    work |= !rise_done[i];
                } else if let Command::Board{center,..}=command {
                    // Hold the first rider on the actual deck for one second
                    // before the partner approaches, using ordinary steering.
                    if i==0 || board_lead.is_some_and(|at|roster.tick>=at+120) {
                        let at=Vec2::from_array(*center)+vec2(if i==0 {-24.} else {24.},0.);
                        inputs[i].0=Controls{wish:((at-p.feet.truncate())*0.06-p.velocity.truncate()*0.015).clamp_length_max(1.),
                            jump:!board_ready[i] && p.grounded,..Default::default()};
                    }
                    work=true;
                } else if let Command::Clear(radius) = command {
                    // A living Duchess can temporarily phase out of the target
                    // list; that is not the end of the encounter.
                    work |= arena.stage.interactions.duchess.as_ref().is_some_and(|d|
                        d.state.stage == crate::duchess::Stage::Fighting);
                    let drop = (body.stats.sanity()<99. || body.stats.will()<99.).then(|| body.stats.loot.get(&arena.visit_key))
                        .flatten().and_then(|loot| loot.drops.iter()
                            .filter(|d| d.origin.truncate().distance(home[i].truncate())<*radius && (d.origin.z-p.feet.z).abs()<90.)
                            .filter_map(|d|world.actor_footing(d.origin+Vec3::Z*16.,PLAYER_CENTER,PLAYER_HALF,180.)
                                .map(|at|(d,at)))
                            .min_by(|(a,_),(b,_)| a.origin.distance_squared(p.feet).total_cmp(&b.origin.distance_squared(p.feet))));
                    if let Some((_,at))=drop {
                        inputs[i].0=walking[i].input(&world,p,at,22.,require_landing)?; planned[i]=true; work=true;
                    } else if let Some((target,_))=visible {
                        inputs[i].0=Controls{wish:super::route_driver::combat_dodge(&world,p,home[i],target.center,roster.tick,0.,
                            Some(roster.players[1-i].player.feet),false,|at| !arena.stage.interactions.hazardous(at)),run:true,..Default::default()};
                        work=true;
                    }
                } else if matches!(command,Command::Watch) {
                    inputs[i].1.weapon=Some(9);inputs[i].1.primary=true;inputs[i].1.alternate=false;
                } else if let Command::Interact(point)=command {
                    if user==Some(i) {
                        let aim=(Vec3::from_array(*point)-p.eye()).normalize_or_zero();
                        inputs[i].1.yaw=aim.y.atan2(aim.x);inputs[i].1.pitch=aim.z.asin();
                        inputs[i].1.primary=false;inputs[i].1.alternate=false;
                        inputs[i].0.use_pressed=true;used=true;
                    }
                    work=true;
                } else if matches!(command,Command::Shoot(_)) {
                    if let Some(target)=switch {
                        if let Some(at)=fight::aim_point(&context,p.eye(),&target) {
                            let aim=(at-p.eye()).normalize_or_zero();
                            inputs[i].1.yaw=aim.y.atan2(aim.x);inputs[i].1.pitch=aim.z.asin();
                            inputs[i].1.weapon=Some(0);inputs[i].1.primary=false;inputs[i].1.alternate=true;
                        }
                    }
                    work=true;
                }
            }
            // Match the solo route pilot: open a door with ordinary E input
            // when its prompt is visible in the direction of movement. Only
            // one body presses each toggle per tick, so partners cannot close
            // the same door immediately after the first player opens it.
            let mut opening=Vec::new();
            for i in 0..2 {
                if busy || !roster.players[i].stats.alive() {continue;}
                let aim=inputs[i].0.wish.extend(0.).normalize_or_zero();
                if let Some(id)=arena.stage.interactions.closed_door_in_reach(&world,roster.players[i].player.eye(),aim) {
                    if !opening.contains(&id) {
                        opening.push(id);inputs[i].0.use_pressed=true;
                        inputs[i].1.yaw=aim.y.atan2(aim.x);inputs[i].1.pitch=0.;
                        inputs[i].1.primary=false;inputs[i].1.alternate=false;
                    }
                }
            }
            if matches!(command,Command::Walk(_)) && !walk_touch {work=true;}
            if require_landing && goal.is_some() && !nav_touch {work=true;}
            if let Command::Rise{landing,..}=command {
                // Once one rider reaches the landing marker, the other can
                // settle beside them. Requiring both capsules at the same
                // point can leave a grounded partner blocked just outside it.
                if rise_done.iter().any(|done|*done) && roster.players.iter().enumerate().all(|(i,b)|
                    !b.stats.alive() || (rising[i] && b.player.grounded
                        && b.player.feet.distance(Vec3::from_array(*landing))<80.)) {
                    work=false;
                }
            }
            if !busy {
                step_ticks += 1;
                quiet = if work {0} else {quiet+1};
            }
            history.push_back(serde_json::json!({"tick":roster.tick,"step":step,"command":command,"feet":before,"flight":flight,
                "course":course.each_ref().map(|w|w.cursor),
                "health":roster.players.each_ref().map(|b|b.stats.sanity()),"will":roster.players.each_ref().map(|b|b.stats.will()),
                "movement":inputs.each_ref().map(|(c,n)|serde_json::json!({"wish":c.wish,"jump":c.jump,"run":c.run,
                    "use":c.use_pressed,"weapon":n.weapon,"primary":n.primary,"alternate":n.alternate})),
                "steam":roster.players.each_ref().map(|b|b.player.steam),"grounded":roster.players.each_ref().map(|b|b.player.grounded)}));
            if history.len()>240 {history.pop_front();}
            arena.step(&mut roster,&map,&mut world,inputs)?;
            recovery.record(&roster,&world,1,spec.map,busy||arena.stage.busy(),arena.stage.interactive_actor().is_some(),
                |at|!arena.stage.interactions.hazardous(at))?;
            let began_scene=old_owner.is_none() && arena.stage.owner.is_some();
            if began_scene {
                let boarding=if let Command::Board{thread,..}=command {
                    let ready=roster.players.each_ref().map(|b|arena.stage.interactions.levels.iter()
                        .any(|s|s.ctl.coop_boarding_ready(thread,&[b.player.feet])==Some(true)));
                    ensure!(ready==[true,true],"Lift started before both players boarded");
                    Some(ready)
                } else {None};
                scene_starts.push(serde_json::json!({"tick":roster.tick,"step":step,"owner":arena.stage.owner,
                    "feet":roster.players.each_ref().map(|b|b.player.feet),"boarding":boarding}));
            }
            for i in 0..2 {
                if planned[i] {walking[i].advanced(&roster.players[i].player);}
                if alive[i]!=roster.players[i].stats.alive() || spawned[i]!=roster.players[i].spawned_at {
                    life_events.push(serde_json::json!({"tick":roster.tick,"slot":i,"alive":roster.players[i].stats.alive(),"spawned_at":roster.players[i].spawned_at}));
                    walking[i]=Default::default();
                    swimming[i]=Default::default();
                    trails[i]=Default::default();
                    jumped[i]=false;
                    if !alive[i] {flight[i]=flight[1-i];swim_leg[i]=swim_leg[1-i];let partner=course[1-i];course[i].rejoin(partner);}
                    fan_started[i]=false;
                }
            }
            if !busy && matches!(command,Command::Approach(_)) {
                if let Some(lead)=approach_lead {
                    let player=&roster.players[lead];
                    approach_stuck=if player.stats.alive() && inputs[lead].0.wish.length_squared()>0.01
                        && player.player.feet.distance_squared(before[lead])<0.01 {approach_stuck+1} else {0};
                    if approach_stuck>=120 && roster.players[1-lead].stats.alive() {
                        // Euclidean distance can choose the wrong leader at a
                        // corner. Let the blocking partner go first instead.
                        approach_lead=Some(1-lead);approach_stuck=0;approach_yields+=1;
                        walking=Default::default();
                        println!("YIELD {} step{step}: player{lead} lets player{} pass",spec.map,1-lead);
                    }
                }
            }
            if let Some(owner)=arena.stage.owner.filter(|id|Some(*id)==old_owner) {
                ensure!(roster.players[owner.partner().index()].player.feet==before[owner.partner().index()],"Scene moved hidden partner");
            }
            if old_owner.is_some() && arena.stage.owner.is_none() {
                // A real scene reunion can bring the waiting partner across
                // a narrow passage. Resume their input course at the owner's
                // current leg instead of walking back over completed ground.
                if matches!(command,Command::CourseFile(_)) {
                    let owner=old_owner.unwrap().index();let leg=course[owner];
                    course[1-owner].rejoin(leg);
                }
                reunions+=1;walking=Default::default();swimming=Default::default();approach_lead=None;approach_stuck=0;
                home=roster.players.each_ref().map(|b|b.player.feet);
            }
            ensure!(roster.checkpoint_generation==0,"Route team-wiped at step{step} tick{tick}");
            ensure!(roster.players.iter().all(|b|!b.stats.god && !b.stats.notarget),"Route enabled cheats");
            if tick%120==0 {
                roster.frame(1,0).validate()?;
                let snapshot=arena.snapshot();
                let packed=super::codec::encode(&serde_json::to_vec(&snapshot)?)?;
                guest.replicate(&serde_json::from_slice(&super::codec::decode(&packed)?)?,&map,&mut guest_world)?;
                ensure!(serde_json::to_value(snapshot)?==serde_json::to_value(guest.snapshot())?,"Replica differs");
                samples+=1;
                if tick%2400==0 {println!("{} t={} step{step} {command:?} health{:?} feet{:?}",spec.map,tick/120,
                    roster.players.each_ref().map(|b|b.stats.sanity()),roster.players.each_ref().map(|b|b.player.feet));}
                clear_background(BLACK); next_frame().await;
            }
            if arena.stage.transition.is_some() {break;}
            // A survivor must be able to leave an airborne waypoint and reach
            // safe footing while the partner's ordinary revival remains pending.
            if !busy && (roster.players.iter().all(|b|b.stats.alive())
                || continue_with_survivor && roster.players.iter().any(|b|b.stats.alive())) {
                let done=match command {
                    Command::Node(_) | Command::HighNode(_) | Command::Nav(_) | Command::Walk(_)
                        | Command::Positions(_) | Command::Rise{..} | Command::Steer{..} | Command::Flight{..}
                        | Command::Swim{..} | Command::SwimFan{..} | Command::SwimFile(_)
                        | Command::MachineBoard{..} | Command::MachineRide{..} | Command::RideHeight(_)
                        | Command::Course(_) | Command::CourseFile(_) | Command::Trail{..} | Command::Pond(_) | Command::Pool(_) => !work,
                    Command::WatchReady => roster.players.iter().filter(|b|b.stats.alive()).all(|b|b.stats.powers.recharge<=0.),
                    Command::WorldRunning => super::world::world_dt(&roster.players,FIXED_DT)>0.,
                    Command::Phase{..} | Command::Cycle{..} => phase_ready,
                    Command::Board{..} => false,
                    Command::Approach(_) => !work || began_scene,
                    Command::Interact(_) => used && began_scene,
                    Command::Shoot(thread) => shot_started && arena.stage.interactions.switch_target(thread).is_none(),
                    Command::Clear(_) => quiet>=240,
                    Command::Wait(ticks) => step_ticks>=*ticks,
                    Command::Drive{ticks,..} => step_ticks>=*ticks,
                    Command::Watch => step_ticks>=120,
                    Command::Staff(enabled) => {staff=*enabled;true},
                    Command::Cards(enabled) => {cards=*enabled;true},
                    Command::Ice(enabled) => {ice=*enabled;if ice {staff=false;}true},
                    Command::ContinueWithSurvivor(enabled) => {continue_with_survivor=*enabled;true},
                };
                if done {
                    println!("DONE {} step{step} {command:?} tick{} feet{:?}",spec.map,roster.tick,roster.players.each_ref().map(|b|b.player.feet));
                    completed.push(serde_json::json!({"step":step,"command":command,"tick":roster.tick,"feet":roster.players.each_ref().map(|b|b.player.feet),
                        "steam_seen":steam_seen,"rise_done":rise_done}));
                    step+=1;step_ticks=0;quiet=0;walking=Default::default();swimming=Default::default();
                    ride_done=[false;2];
                    course=Default::default();
                    pond=Default::default();
                    pool=Default::default();
                    trails=Default::default();
                    rising=[false;2];steam_seen=[false;2];rise_done=[false;2];board_lead=None;shot_started=false;approach_lead=None;approach_stuck=0;jumped=[false;2];flight=[0;2];fan_started=[false;2];swim_leg=[0;2];
                    home=roster.players.each_ref().map(|b|b.player.feet);
                }
            }
            let limit=match command {Command::Wait(ticks)=>ticks.saturating_add(120*120),
                Command::WatchReady=>120*400,Command::Course(_)|Command::CourseFile(_)=>120*600,_=>120*120};
            ensure!(step_ticks<limit,"Route stalled at step{step} {command:?}");
        }
        let next_entry=(!spec.exit.entrance.is_empty()).then(||spec.exit.entrance.to_owned());
        ensure!(arena.stage.transition.as_ref().is_some_and(|e|e.0==spec.exit.map && e.1==next_entry)
            && roster.players.iter().all(|b|b.stats.alive()),"Missing living pair's actual onward exit");
        (spec.verify)(&arena,&roster.players)?;
        let next_visit=Visit{epoch:2,map:spec.exit.map.into(),entry:next_entry,..visit.clone()};
        let next_map=Bsp::parse(&a.read(&format!("maps/{}.bsp",next_visit.map))?)?;
        let mut next_world=World::from_bsp(&next_map)?;
        let next_arena=Arena::load(a,&next_map,&mut next_world,&next_visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(&next_map,next_visit.entry.as_deref());
        let arrived=roster.arrive_with(&next_world,eye,yaw,&next_visit.map,next_visit.entry.as_deref(),
            |at|!next_arena.stage.interactions.hazardous(at))?;
        arrival=Some(serde_json::json!({"visit":next_visit,"frame":arrived.frame(2,0),
            "health_floors":arrived.players.each_ref().map(|b|b.stats.minimum_sanity)}));
        for (old,new) in roster.players.iter().zip(&arrived.players) {
            let mut expected=serde_json::to_value(&old.stats)?;
            // The temple grants the turtle's breathing shell on entry, as in
            // the solo campaign. This is a story reward, not a resource refill.
            if next_visit.map=="utemple" { expected["turtle_air"]=serde_json::json!(true); }
            // Normal campaign arrivals fill missing earlier weapon milestones.
            // Health, ammo, consumables and personal pickup history still carry exactly.
            if let Some(baseline)=crate::campaign::loadout(&next_visit.map,next_visit.entry.as_deref()) {
                for (owned,minimum) in expected["owned"].as_array_mut().context("Missing weapon inventory")?.iter_mut().zip(baseline) {
                    *owned=serde_json::json!(owned.as_u64().context("Invalid weapon count")?.max(minimum as u64));
                }
            }
            // The health floor is deliberately absent from serialized Stats.
            ensure!(expected==serde_json::to_value(&new.stats)?
                && new.stats.minimum_sanity==0. && next_world.body_clear(new.player.feet),
                "Onward arrival changed resources or obstructed a player");
        }
        ensure!(arrived.players[0].player.feet.distance(arrived.players[1].player.feet)>=36.,"Onward arrival overlaps");
        Ok(())
    }.await;
    let report=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"visit":visit,"full_route_passed":result.is_ok(),
        "error":result.as_ref().err().map(|e|format!("{e:#}")),"commands":commands,"completed":completed,"step":step,
        "replica_samples":samples,"reunions":reunions,"life_events":life_events,"scene_starts":scene_starts,
        "board_wait_ticks":board_wait_ticks,"approach_yields":approach_yields,"frame":roster.frame(1,0),
        "world":arena.snapshot(),"arrival":arrival,"recent_inputs":history,"recovery_log":recovery_name,
        "health_floors":roster.players.each_ref().map(|b|b.stats.minimum_sanity),
        "scope":"Fresh normal two-player production Arena route; compressed replicas, not internet transport"});
    std::fs::create_dir_all(crate::preferences::path(""))?;
    std::fs::write(crate::preferences::path(&format!("{}-coop-route.json",spec.map)),serde_json::to_vec(&report)?)?;
    result?;
    println!("PASS native co-op {}: {} ticks, {samples} exact replicas and actual {} arrival",spec.map,roster.tick,spec.exit.map);
    Ok(())
}
