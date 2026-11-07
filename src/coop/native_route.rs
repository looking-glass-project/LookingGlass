//! Accelerated ordinary-input campaign diagnostics. Never seeds gameplay state.
use super::{arena::Arena, protocol::{Input, Visit}, world::Roster};
use crate::{assets::Assets, bsp::Bsp, collision::{World, PLAYER_CENTER, PLAYER_HALF},
    inventory::Stats, movement::{Controls, FIXED_DT}, route::{Route, fight}};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Command {
    Node(usize), Nav([f32; 3]), Clear(f32), Wait(u64), Watch, Staff(bool),
    HighNode(usize), Walk([f32; 3]), Positions([Option<[f32; 3]>; 2]),
    Rise { center: [f32; 2], height: f32, landing: [f32; 3] },
    Board { thread: String, center: [f32; 2] },
}
pub(crate) struct Spec {
    pub map: &'static str,
    pub entry: &'static str,
    pub exit: crate::level::spec::ExitSpec,
    pub commands: Vec<Command>,
    pub navigation: Navigation,
    pub verify: fn(&Arena) -> Result<()>,
}
pub(crate) enum Navigation {
    /// Preserve the battlefield course's nearby-waypoint handoff.
    Nearby,
    /// Finish jumps before turning; leave room for a partner on narrow landings.
    LandedPair,
}
#[derive(Default)]
struct Walking {
    goal: Option<Vec3>,
    steps: VecDeque<(Controls, Vec3)>,
    age: u32,
    settling: u32,
}
impl Walking {
    fn input(&mut self, world: &World, player: &crate::movement::Player, goal: Vec3, close: f32, require_landing: bool) -> Result<Controls> {
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
    fn advanced(&mut self, player: &crate::movement::Player) {
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
    let mut trackers: [fight::Tracker; 2] = Default::default();
    let mut step = 0;
    let mut step_ticks = 0;
    let mut home = roster.players.each_ref().map(|b| b.player.feet);
    let mut quiet = 0;
    let mut staff = true;
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
    let result: Result<()> = async {
        for tick in 0..120*1800 {
            let command = commands.get(step).context("Route commands ended before the actual exit")?;
            let busy = arena.stage.busy();
            let old_owner = arena.stage.owner;
            let before = roster.players.each_ref().map(|b| b.player.feet);
            let alive = roster.players.each_ref().map(|b| b.stats.alive());
            let spawned = roster.players.each_ref().map(|b| b.spawned_at);
            let all = targets(&arena);
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
            let mut inputs = [(Controls::default(), Input::default()); 2];
            let mut planned = [false; 2];
            let mut work = false;
            let mut walk_touch = false;
            let mut nav_touch = false;
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
                let foes = all.iter().copied().filter(|t| t.center.distance(p.eye()) <= radius).collect::<Vec<_>>();
                let mut attack = fight::plan(&context, p.eye(), &foes, &trackers[i], false);
                let visible = foes.iter().filter_map(|t| fight::aim_point(&context, p.eye(), t).map(|at|(t,at)))
                    .min_by(|(_,a),(_,b)| a.distance_squared(p.eye()).total_cmp(&b.distance_squared(p.eye())));
                if staff && body.stats.copies(7)>0 && body.stats.will()>21. {
                    if let Some((target,point)) = visible {
                        attack = fight::Plan { target:Some(target.id), aim:(point-p.eye()).normalize_or_zero(), weapon:7, alternate:false };
                    }
                }
                if attack.target.is_some() {
                    inputs[i].1.weapon=Some(attack.weapon as u8);
                    inputs[i].1.primary=!attack.alternate; inputs[i].1.alternate=attack.alternate;
                    inputs[i].1.yaw=attack.aim.y.atan2(attack.aim.x); inputs[i].1.pitch=attack.aim.z.asin().clamp(-1.56,1.56);
                }
                let personal_goal = if let Command::Positions(goals)=command {goals[i].map(Vec3::from_array)} else {goal};
                if let Some(goal) = personal_goal {
                    inputs[i].0=walking[i].input(&world,p,goal,38.,require_landing)?; planned[i]=true;
                    // One body reaches the shared marker while the other keeps
                    // normal personal space nearby. Separate personal goals
                    // retain their own arrival bounds.
                    let landed=!require_landing || p.grounded;
                    nav_touch |= landed && p.feet.distance(goal)<38.;
                    let nearby=if matches!(command,Command::Positions(_)) || !require_landing {48.} else {80.};
                    work |= !landed || p.feet.distance(goal)>=nearby;
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
                    let drop = (body.stats.sanity()<99. || body.stats.will()<99.).then(|| body.stats.loot.get(&arena.visit_key))
                        .flatten().and_then(|loot| loot.drops.iter()
                            .filter(|d| d.origin.truncate().distance(home[i].truncate())<*radius && (d.origin.z-p.feet.z).abs()<90.)
                            .min_by(|a,b| a.origin.distance_squared(p.feet).total_cmp(&b.origin.distance_squared(p.feet))));
                    if let Some(drop)=drop {
                        let at=world.actor_footing(drop.origin+Vec3::Z*16.,PLAYER_CENTER,PLAYER_HALF,180.)
                            .context("Unsupported personal essence")?;
                        inputs[i].0=walking[i].input(&world,p,at,22.,require_landing)?; planned[i]=true; work=true;
                    } else if let Some((target,_))=visible {
                        inputs[i].0=Controls{wish:super::route_driver::combat_dodge(&world,p,home[i],target.center,roster.tick,0.,
                            Some(roster.players[1-i].player.feet),false,|at| !arena.stage.interactions.hazardous(at)),run:true,..Default::default()};
                        work=true;
                    }
                } else if matches!(command,Command::Watch) {
                    inputs[i].1.weapon=Some(9);inputs[i].1.primary=true;inputs[i].1.alternate=false;
                }
            }
            if matches!(command,Command::Walk(_)) && !walk_touch {work=true;}
            if require_landing && goal.is_some() && !nav_touch {work=true;}
            if !busy {
                step_ticks += 1;
                quiet = if work {0} else {quiet+1};
            }
            history.push_back(serde_json::json!({"tick":roster.tick,"step":step,"command":command,"feet":before,
                "health":roster.players.each_ref().map(|b|b.stats.sanity()),"will":roster.players.each_ref().map(|b|b.stats.will()),
                "movement":inputs.each_ref().map(|(c,_)|serde_json::json!({"wish":c.wish,"jump":c.jump,"run":c.run})),
                "steam":roster.players.each_ref().map(|b|b.player.steam),"grounded":roster.players.each_ref().map(|b|b.player.grounded)}));
            if history.len()>240 {history.pop_front();}
            arena.step(&mut roster,&map,&mut world,inputs)?;
            if old_owner.is_none() && arena.stage.owner.is_some() {
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
                }
            }
            if let Some(owner)=arena.stage.owner.filter(|id|Some(*id)==old_owner) {
                ensure!(roster.players[owner.partner().index()].player.feet==before[owner.partner().index()],"Scene moved hidden partner");
            }
            if old_owner.is_some() && arena.stage.owner.is_none() {reunions+=1;}
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
            if !busy && roster.players.iter().all(|b|b.stats.alive()) {
                let done=match command {
                    Command::Node(_) | Command::HighNode(_) | Command::Nav(_) | Command::Walk(_)
                        | Command::Positions(_) | Command::Rise{..} => !work,
                    Command::Board{..} => false,
                    Command::Clear(_) => quiet>=240,
                    Command::Wait(ticks) => step_ticks>=*ticks,
                    Command::Watch => step_ticks>=120,
                    Command::Staff(enabled) => {staff=*enabled;true},
                };
                if done {
                    println!("DONE {} step{step} {command:?} tick{} feet{:?}",spec.map,roster.tick,roster.players.each_ref().map(|b|b.player.feet));
                    completed.push(serde_json::json!({"step":step,"command":command,"tick":roster.tick,"feet":roster.players.each_ref().map(|b|b.player.feet),
                        "steam_seen":steam_seen,"rise_done":rise_done}));
                    step+=1;step_ticks=0;quiet=0;walking=Default::default();
                    rising=[false;2];steam_seen=[false;2];rise_done=[false;2];board_lead=None;
                    home=roster.players.each_ref().map(|b|b.player.feet);
                }
            }
            let limit=if let Command::Wait(ticks)=command {ticks.saturating_add(120*120)} else {120*120};
            ensure!(step_ticks<limit,"Route stalled at step{step} {command:?}");
        }
        ensure!(arena.stage.transition.as_ref().is_some_and(|e|spec.exit.matches(e))
            && roster.players.iter().all(|b|b.stats.alive()),"Missing living pair's actual onward exit");
        (spec.verify)(&arena)?;
        let next_visit=Visit{epoch:2,map:spec.exit.map.into(),entry:Some(spec.exit.entrance.into()),..visit.clone()};
        let next_map=Bsp::parse(&a.read(&format!("maps/{}.bsp",next_visit.map))?)?;
        let mut next_world=World::from_bsp(&next_map)?;
        let next_arena=Arena::load(a,&next_map,&mut next_world,&next_visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(&next_map,next_visit.entry.as_deref());
        let arrived=roster.arrive_with(&next_world,eye,yaw,&next_visit.map,next_visit.entry.as_deref(),
            |at|!next_arena.stage.interactions.hazardous(at))?;
        for (old,new) in roster.players.iter().zip(&arrived.players) {
            ensure!(serde_json::to_value(&old.stats)?==serde_json::to_value(&new.stats)? && next_world.body_clear(new.player.feet),
                "Onward arrival changed resources or obstructed a player");
        }
        arrival=Some(serde_json::json!({"visit":next_visit,"frame":arrived.frame(2,0)}));
        Ok(())
    }.await;
    let report=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"full_route_passed":result.is_ok(),
        "error":result.as_ref().err().map(|e|format!("{e:#}")),"commands":commands,"completed":completed,"step":step,
        "replica_samples":samples,"reunions":reunions,"life_events":life_events,"scene_starts":scene_starts,
        "board_wait_ticks":board_wait_ticks,"frame":roster.frame(1,0),
        "world":arena.snapshot(),"arrival":arrival,"recent_inputs":history,
        "scope":"Fresh normal two-player production Arena route; compressed replicas, not internet transport"});
    std::fs::create_dir_all(crate::preferences::path(""))?;
    std::fs::write(crate::preferences::path(&format!("{}-coop-route.json",spec.map)),serde_json::to_vec(&report)?)?;
    result?;
    println!("PASS native co-op {}: {} ticks, {samples} exact replicas and actual {} arrival",spec.map,roster.tick,spec.exit.map);
    Ok(())
}
