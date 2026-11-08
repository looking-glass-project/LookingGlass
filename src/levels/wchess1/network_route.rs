//! Independent-client version of the ordinary-input Pale Realm course.
//! The outer fixture chooses a course step only after both clients finish it.
//! Each client reads its own received world and controls only its local player.
use super::*;
use crate::{coop::{arena::Arena, protocol::Input, world::Frame, PlayerId},
    movement::{Controls,FIXED_DT}, route::{fight,Route}};
use std::{collections::VecDeque, sync::mpsc};
type Plan=VecDeque<(Controls,Vec3)>;
struct Worker { step:usize, phase:u8, warp:u64, start:Vec3, result:mpsc::Receiver<std::result::Result<Vec<(Controls,Vec3)>,usize>> }

pub(super) fn lift_phase(phase:u8,assembled:bool,boarded:bool,arrived:bool)->u8 {
    match phase {0 if assembled || boarded=>1,1 if arrived=>2,_=>phase}
}

fn record_trail(trail:&mut VecDeque<(Vec3,bool)>,feet:Vec3,airborne:bool) {
    // Keep takeoffs and landings even when they are closer than the spatial
    // sampling interval. Otherwise two jumps can merge into one airborne path
    // and the follower loses the supported takeoff between them.
    if trail.back().is_none_or(|(at,air)|at.distance(feet)>12. || *air!=airborne) {
        trail.push_back((feet,airborne));
    }
}

/// A follower may land before reaching every sample of a recorded descent.
/// Rejoin only nearby, continuously supported ground; never shortcut a jump
/// from its takeoff or aim back up at an already completed fall.
pub(super) fn rejoin_landing(trail:&mut VecDeque<(Vec3,bool)>,p:&Player,world:&World,
    hazardous:impl Fn(Vec3)->bool)->bool {
    if !p.grounded || p.swimming || p.climbing() || p.ledge.is_some() || p.rope.is_some()
        || !trail.front().is_some_and(|(at,air)|*air && at.z-p.feet.z>64.) {return false;}
    let candidate=trail.iter().enumerate().skip(1).filter(|(_, (at,air))| {
        let delta=*at-p.feet;
        if *air || delta.truncate().length()>48. || delta.z.abs()>18.
            || !world.body_clear(*at) {return false;}
        let path=world.body_trace(p.feet,*at);
        if path.start_solid || path.fraction<1. {return false;}
        let segments=(delta.length()/12.).ceil().max(1.) as usize;
        (0..=segments).all(|n| {
            let point=p.feet.lerp(*at,n as f32/segments as f32);
            let floor=world.body_trace(point,point-Vec3::Z*24.);
            !floor.start_solid && floor.fraction<1. && floor.normal.z>0.65
                && world.liquid_at(point+Vec3::Z)==0 && !hazardous(point)
        })
    }).min_by(|(_, (a,_)),(_, (b,_))|a.distance_squared(p.feet).total_cmp(&b.distance_squared(p.feet)))
        .map(|(index,_)|index);
    if let Some(index)=candidate {trail.drain(..index);true} else {false}
}

pub(crate) struct NetworkDriver {
    steps:Vec<coop_route::Step>, index:Option<usize>, phase:u8,
    last_tick:Option<u64>, ticks:u64, quiet:u64, home:[Vec3;2],
    free:bool, walked_scene:bool, done:bool, clearing_landing:bool,
    trail:VecDeque<(Vec3,bool)>, last:Vec3, stuck:u64, stuck_clock:u64,
    plan:Plan, plan_age:u64, worker:Option<Worker>,
    board_node:Option<usize>, board_move:usize, board_moving:bool,
    jump_at:Option<u64>, use_at:Option<u64>, tracker:fight::Tracker,
    closing:Option<usize>,
    missed:Option<(usize,f32,u64)>, sidestep:Option<Vec3>,
    cached:(Controls,Input),
    crossing:spike_route::Crossing,
}
impl NetworkDriver {
    pub(crate) fn new()->Result<Self> {
        let bytes=include_bytes!("coop_steps.json");
        std::fs::write(crate::preferences::path("wchess1-coop-course.json"),bytes)?;
        Ok(Self{steps:serde_json::from_slice(bytes)?,index:None,phase:0,last_tick:None,
            ticks:0,quiet:0,home:[Vec3::ZERO;2],free:false,walked_scene:false,done:false,
            clearing_landing:false,trail:Default::default(),last:Vec3::ZERO,stuck:0,stuck_clock:0,
            plan:Default::default(),plan_age:0,worker:None,board_node:None,board_move:0,
            board_moving:false,jump_at:None,use_at:None,tracker:Default::default(),closing:None,
            missed:None,sidestep:None,cached:Default::default(),crossing:Default::default()})
    }
    pub(crate) fn reached(&self)->bool {self.done}
    pub(crate) fn spectate(&mut self,tick:u64) {
        // The ten-second countdown is real simulation time. Observe it while
        // controls are neutral so revival is not mistaken for a stale client.
        // Restart only the current authored step from the actual revived body.
        self.last_tick=Some(tick);self.index=None;self.done=false;
        self.cached=Default::default();self.trail.clear();self.worker=None;
        self.tracker=Default::default();self.jump_at=None;self.use_at=None;
        self.missed=None;self.sidestep=None;
        self.crossing=Default::default();
    }
    pub(crate) fn report(&self)->serde_json::Value {
        serde_json::json!({"step":self.index,"mode":self.index.map(|i|&self.steps[i].mode),
            "phase":self.phase,"ticks":self.ticks,"quiet":self.quiet,"reached":self.done,
            "observed_tick":self.last_tick,"stuck":self.stuck,"planning":self.worker.is_some(),
            "plan_remaining":self.plan.len(),"expected":self.plan.front().map(|(_,p)|p),
            "board_move":self.board_move,"board_node":self.board_node,"trail":self.trail,
            "closing_target":self.closing,
            "missed_target":self.missed,"sidestep":self.sidestep,
            "spike_crossing":self.crossing,
            "input":self.cached.1,"movement":{"wish":self.cached.0.wish,"swim":self.cached.0.swim,
                "jump":self.cached.0.jump,"use":self.cached.0.use_pressed,"run":self.cached.0.run}})
    }
    fn reset_walk(&mut self,feet:Vec3) {
        self.plan.clear();self.plan_age=0;self.stuck=0;self.stuck_clock=0;
        self.last=feet;self.clearing_landing=false;
    }
    pub(crate) fn sample(&mut self,index:usize,frame:&Frame,arena:&Arena,world:&World,
        local:PlayerId,first:PlayerId,lead:f32,keys:&mut Input)->Result<()> {
        ensure!(index<self.steps.len(),"Invalid Pale Realm course step");
        let realm=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Realm>())
            .context("Pale Realm course used outside Pale Realm")?;
        if self.index!=Some(index) {
            ensure!(self.index.is_none_or(|old|index==old+1),"Pale Realm course skipped a step");
            // The timed corridor has its own traversal controller. Its old
            // leader trail would send the follower back through the spikes.
            if self.index.is_some_and(|old|self.steps[old].mode=="spikes") {self.trail.clear();}
            self.index=Some(index);self.phase=0;self.ticks=0;self.quiet=0;
            self.home=frame.players.each_ref().map(|b|b.player.feet);
            self.free=false;self.walked_scene=false;self.done=false;
            self.cached=Default::default();
            self.closing=None;
            self.missed=None;self.sidestep=None;
            self.crossing=Default::default();
            self.board_node=None;self.board_move=0;self.board_moving=false;
            self.reset_walk(frame.players[local.index()].player.feet);
        }
        let fresh=self.last_tick!=Some(frame.tick);
        if fresh {
            let elapsed=self.last_tick.map_or(1,|old|frame.tick.saturating_sub(old));
            ensure!((1..=360).contains(&elapsed),"Pale Realm observation clock reset or stalled");
            self.last_tick=Some(frame.tick);self.cached=Default::default();
            let s=self.steps[index].clone();
            let body=&frame.players[local.index()];
            if arena.stage.busy() {
                if self.free && s.mode=="walk" {self.walked_scene=true;}
                self.trail.clear();self.reset_walk(body.player.feet);
            } else {
                self.free=true;self.ticks+=elapsed;
                ensure!(self.ticks<120*180,"Pale Realm course step {index} stalled");
                let leader=&frame.players[first.index()].player;
                if local!=first && arena.stage.interactive_actor().is_none() {
                    record_trail(&mut self.trail,leader.feet,!leader.grounded && !leader.swimming);
                }
                ensure!(self.trail.len()<2000,"Pale Realm follower lost leader trail");
                let mut c=Controls::default();
                if !self.done {
                    match s.mode.as_str() {
                        "walk"=> {
                            if self.walked_scene || arena.stage.interactive_actor().is_some() {self.done=true;}
                            else {let (input,ready)=self.walk([Vec3::from_array(s.pos);2],false,s.radius,frame,arena,world,local,first,lead,elapsed)?;c=input;self.done=ready;}
                        }
                        "wait"=>self.done=self.ticks as f32*FIXED_DT>=s.value,
                        "spikes"=> {
                            let (input,ready)=self.crossing.controls(realm,world,&body.player,
                                &frame.players[first.index()].player,local==first,
                                Vec3::from_array(s.hold),Vec3::from_array(s.pos),lead);
                            c=input;self.done=ready;
                        }
                        "clear"|"rush"=> {
                            let targets=realm.targets();let ctx=crate::combat::Context{world,targets:&targets};
                            let mut seen=false;
                            for id in PlayerId::ALL {
                                let p=&frame.players[id.index()].player;
                                if let Some(t)=targets.iter().filter(|t|realm.hostile_target(t.id)
                                    && t.center.distance(self.home[id.index()]+PLAYER_CENTER)<s.value
                                    && fight::aim_point(&ctx,p.eye(),t).is_some())
                                    .min_by(|a,b|a.center.distance_squared(p.eye()).total_cmp(&b.center.distance_squared(p.eye()))) {
                                    seen=true;let d=(t.center-p.eye()).truncate();
                                    if id==local && s.mode=="rush" && d.length()>48. {c.wish=d.normalize_or_zero();c.run=true;}
                                }
                            }
                            self.quiet=if seen{0}else{self.quiet+elapsed};self.done=self.quiet>=120;
                        }
                        "bishop"|"knight"=> {
                            let piece=if s.mode=="bishop"{Piece::Bishop}else{Piece::Knight};
                            let completed=if piece==Piece::Bishop{realm.saved.bishop_done}else{realm.saved.knight_done};
                            self.done=completed && realm.saved.board.is_none() && arena.stage.interactive_actor().is_none();
                            if let Some(b)=&realm.saved.board {
                                ensure!(b.piece==piece && arena.stage.interactive_actor()==Some(first),"Wrong owned chess puzzle");
                                // The scene's run onto node zero must finish before the first chosen move.
                                if !realm.controlled() || self.board_node.is_some() {
                                    let node=*self.board_node.get_or_insert(b.node);
                                    if b.node!=node && b.moving.is_none() {
                                        self.board_move+=1;self.board_node=Some(b.node);self.board_moving=false;
                                    }
                                    self.board_moving|=b.moving.is_some();
                                    let dirs=if piece==Piece::Bishop{check::BISHOP}else{check::KNIGHT};
                                    if local==first && !self.board_moving {
                                        if let Some(&direction)=dirs.get(self.board_move) {c.wish=check::wish(piece,direction);c.run=true;}
                                    }
                                }
                            }
                        }
                        "lever"=> {
                            self.done=realm.saved.pull.is_none() && arena.stage.interactive_actor().is_none()
                                && if s.name=="bell_lever" {realm.saved.bell.is_some_and(|t|realm.saved.age-t>=5.2)} else {realm.water_ready()};
                            if !self.done && local==first && realm.saved.pull.is_none()
                                && !realm.saved.fired.contains(&s.name) {
                                let at=realm.data.point(&s.name).point(vec3(36.,0.,26.));
                                let aim=(at-body.player.eye()).normalize_or_zero();
                                ensure!(realm.lever(world,body.player.eye(),aim).is_some(),"Chess lever out of reach");
                                self.cached.1.yaw=aim.y.atan2(aim.x);self.cached.1.pitch=aim.z.asin();c.use_pressed=true;
                            }
                        }
                        "lift"=> {
                            // A peer can miss the brief all-ready frame after
                            // the other client boards or leaves. Actual lift
                            // support and this player's arrival remain visible.
                            let boarded=frame.players.iter().any(|b|b.player.grounded && realm.lift_support(b.player.feet));
                            let arrived=body.player.grounded && body.player.feet.z>250.;
                            let observed=lift_phase(self.phase,false,boarded,arrived);
                            if observed!=self.phase {
                                self.phase=observed;self.trail.clear();self.reset_walk(body.player.feet);
                            }
                            let (goals,precise,radius)=match self.phase {
                                0=>([vec3(-720.,-544.,64.);2],false,7.),
                                1=>{let mut g=[vec3(-752.,-384.,256.+realm.saved.elevator.height);2];
                                    g[first.index()].x=-800.;(g,true,12.)},
                                _=>([vec3(-800.,-256.,256.);2],false,7.),
                            };
                            let (input,ready)=self.walk(goals,precise,radius,frame,arena,world,local,first,lead,elapsed)?;c=input;
                            let next=lift_phase(self.phase,ready && realm.saved.elevator.phase==1,boarded,arrived);
                            if next!=self.phase {self.phase=next;self.trail.clear();self.reset_walk(body.player.feet);}
                            else if self.phase==2 && ready {self.done=true;}
                        }
                        _=>anyhow::bail!("Unknown Pale Realm course mode"),
                    }
                }
                self.cached.0=c;
                if s.mode!="lever" {self.cached.1=Input{yaw:body.yaw,pitch:body.pitch,..Default::default()};}
                if arena.stage.interactive_actor().is_none() && s.mode!="lever" && s.mode!="spikes" {
                    self.combat(frame,arena,world,local,elapsed);
                }
                if self.cached.0.jump && self.jump_at.is_none_or(|t|frame.tick.saturating_sub(t)>=60) {self.jump_at=Some(frame.tick);}
                else {self.cached.0.jump=false;}
                if self.cached.0.use_pressed && self.use_at.is_none_or(|t|frame.tick.saturating_sub(t)>=90) {self.use_at=Some(frame.tick);}
                else {self.cached.0.use_pressed=false;}
            }
        }
        let (c,i)=self.cached;
        keys.wish=c.wish;keys.swim=c.swim;keys.rise=c.rise;keys.run=c.run;
        keys.yaw=i.yaw;keys.pitch=i.pitch.clamp(-1.56,1.56);keys.weapon=i.weapon;
        keys.primary=i.primary;keys.alternate=i.alternate;
        if fresh {keys.jumps+=u64::from(c.jump);keys.uses+=u64::from(c.use_pressed);}
        keys.validate()
    }
    fn combat(&mut self,frame:&Frame,arena:&Arena,world:&World,local:PlayerId,elapsed:u64) {
        let realm=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Realm>()).unwrap();
        let b=&frame.players[local.index()];let mut targets=arena.npcs.targets();targets.extend(arena.stage.interactions.levels_targets());
        let foes=targets.iter().filter(|t|realm.hostile_target(t.id)).copied().collect::<Vec<_>>();
        self.tracker.observe(&foes,elapsed as f32*FIXED_DT);
        let ctx=crate::combat::Context{world,targets:&targets};
        let mut attack=fight::plan(&ctx,b.player.eye(),&foes,&self.tracker,b.stats.copies(1)>0 && b.stats.will()>=fight::CARD_COST);
        // Keep a free melee strike instead of replacing it with a paid stream.
        if !(attack.target.is_some() && attack.weapon==0 && !attack.alternate)
            && b.stats.copies(4)>0 && b.stats.will()>=0.75 && b.player.immersion.level==0 {
            if let Some(at)=foes.iter().filter_map(|t|fight::aim_point(&ctx,b.player.eye(),t))
                .filter(|p|p.distance(b.player.eye())<360.)
                .min_by(|a,c|a.distance_squared(b.player.eye()).total_cmp(&c.distance_squared(b.player.eye()))) {
                attack.weapon=4;attack.alternate=false;attack.aim=(at-b.player.eye()).normalize_or_zero();attack.target=Some(0);
            }
        }
        let (c,i)=&mut self.cached;
        // Network waiting can leave the leader just beyond Blade reach while
        // a pawn keeps hitting. Close the small gap on checked floor, using the
        // Blade in hand rather than throwing it away for its recovery period.
        if !self.done && (c.wish==Vec2::ZERO || self.closing.is_some()) && b.player.grounded && b.stats.copies(0)>0 {
            if let Some((target,point))=foes.iter().filter_map(|t|fight::aim_point(&ctx,b.player.eye(),t).map(|p|(t,p)))
                .filter(|(t,p)|self.closing.is_none_or(|id|id==t.id)
                    && p.distance(b.player.eye())<if self.closing.is_some(){180.}else{140.}
                    && (p.z-b.player.eye().z).abs()<60.)
                .min_by(|(_,a),(_,c)|a.distance_squared(b.player.eye()).total_cmp(&c.distance_squared(b.player.eye()))) {
                let aim=(point-b.player.eye()).normalize_or_zero();
                let wish=aim.truncate().normalize_or_zero();
                let melee=crate::weapons::blade_melee(&ctx,b.player.eye(),aim).iter().any(|h|h.id==target.id);
                // A doorway's glancing wall trace can reject a straight line
                // even though normal walking slides safely along it. Predict
                // that existing movement on a copy; never assign the copy back.
                let mut future=b.player.clone();let mut safe=true;
                for _ in 0..12 {
                    future.tick(world,Controls{wish,..Default::default()});
                    safe &=future.grounded && world.body_clear(future.feet)
                        && !arena.stage.interactions.hazardous(future.feet);
                }
                if melee || safe && future.feet.distance(b.player.feet)>0.1 {
                    self.closing=Some(target.id);
                    attack.weapon=0;attack.alternate=false;attack.aim=aim;attack.target=Some(target.id);
                    if !melee {c.wish=wish;c.run=false;}
                }
            } else {self.closing=None;}
        }
        if attack.target.is_some() {
            i.yaw=attack.aim.y.atan2(attack.aim.x);i.pitch=attack.aim.z.asin();
            i.weapon=Some(attack.weapon as u8);i.primary=!attack.alternate;i.alternate=attack.alternate;
            self.tracker.weapon=attack.weapon;self.tracker.last_aim=attack.aim;
        }
        // A visible aim ray need not give the animated hand's projectile a
        // clear shot. After several real throws without damage, try a short
        // supported sidestep instead of firing forever from the same spot.
        let health=attack.target.and_then(|id|realm.saved.cast.iter()
            .find(|a|a.id()==id).map(|a|(id,a.piece.health)));
        self.missed=health.map(|(id,health)| {
            let ticks=self.missed.filter(|(old,h,_)|*old==id && health>=*h)
                .map_or(0,|(_,_,ticks)|ticks+elapsed);
            (id,health,ticks)
        });
        if self.steps[self.index.unwrap()].mode=="clear" && b.player.grounded
            && !b.player.swimming && c.wish==Vec2::ZERO && self.closing.is_none() {
            let safe_walk=|wish:Vec2,ticks:usize|->Option<Vec3> {
                let mut future=b.player.clone();
                for _ in 0..ticks {
                    future.tick(world,Controls{wish,..Default::default()});
                    if !future.grounded || !world.body_clear(future.feet)
                        || arena.stage.interactions.hazardous(future.feet)
                        || future.feet.distance(self.home[local.index()])>120. {return None;}
                }
                Some(future.feet)
            };
            if self.sidestep.is_none() && self.missed.is_some_and(|(_,_,ticks)|ticks>=960) {
                let forward=attack.aim.truncate().normalize_or_zero();
                let right=vec2(forward.y,-forward.x);
                self.sidestep=[right,-right].into_iter().find_map(|wish|safe_walk(wish,60)
                    .filter(|at|at.distance(b.player.feet)>40.));
                if let Some((_,_,ticks))=&mut self.missed {*ticks=0;}
            }
            if let Some(goal)=self.sidestep {
                let delta=(goal-b.player.feet).truncate();let wish=delta.normalize_or_zero();
                if delta.length()<6. || health.is_none() || safe_walk(wish,12).is_none() {
                    self.sidestep=None;
                } else {c.wish=wish;c.run=false;}
            }
        } else {self.sidestep=None;}
        let aim=c.wish.extend(0.).normalize_or_zero();
        if arena.stage.interactions.closed_door_in_reach(world,b.player.eye(),aim).is_some() {
            c.use_pressed=true;i.yaw=aim.y.atan2(aim.x);i.pitch=0.;i.primary=false;i.alternate=false;
        }
    }
    fn walk(&mut self,goals:[Vec3;2],precise:bool,radius:f32,frame:&Frame,arena:&Arena,
        world:&World,local:PlayerId,first:PlayerId,lead:f32,elapsed:u64)->Result<(Controls,bool)> {
        let i=local.index();let p=&frame.players[i].player;let leader=&frame.players[first.index()].player;
        let ready=std::array::from_fn::<_,2,_>(|i| {
            let p=&frame.players[i].player;let d=goals[i]-p.feet;let gap=p.feet-leader.feet;
            let on_leader=!precise && i!=first.index() && p.velocity.z.abs()<0.1
                && (gap.z-PLAYER_HALF.z*2.).abs()<0.05 && gap.truncate().length()<crate::coop::separation::DISTANCE;
            d.truncate().length()<if precise{12.}else if i==first.index(){radius}else{80.}
                && d.z.abs()<if precise{24.}else if on_leader{80.}else{45.}
                && (p.grounded || p.swimming || on_leader)
        });
        let mut c=Controls::default();
        if ready.iter().all(|v|*v) {if precise {self.trail.clear();}return Ok((c,true));}
        let partner=&frame.players[first.partner().index()].player;
        self.clearing_landing=!precise && (partner.climbing() || partner.ledge.as_ref().is_some_and(|h|h.pulling))
            && (self.clearing_landing || ready[first.index()] && leader.grounded);
        let clear_wish=|directions:&[Vec2],gap:Option<Vec2>|->Vec2 {
            for &wish in directions {
                if gap.is_some_and(|d|d.dot(wish)<0.) {continue;}
                let at=p.feet+wish.extend(0.)*12.;let path=world.body_trace(p.feet,at);
                let floor=world.body_trace(at,at-Vec3::Z*32.);
                if !path.start_solid && path.fraction==1. && !floor.start_solid && floor.fraction<1.
                    && floor.normal.z>0.65 && !arena.stage.interactions.hazardous(at) {return wish;}
            }Vec2::ZERO
        };
        if local==first && self.clearing_landing {
            let gap=p.feet-frame.players[first.partner().index()].player.feet;
            if gap.truncate().length()<80. {let away=gap.truncate().try_normalize().unwrap_or(Vec2::X);
                let side=vec2(-away.y,away.x);c.wish=clear_wish(&[away,side,-side],None);}
            return Ok((c,false));
        }
        if local!=first && !precise {
            if rejoin_landing(&mut self.trail,p,world,|at|arena.stage.interactions.hazardous(at)) {
                // A pending search belongs to the obsolete high target even
                // when its step/phase/start position still match this frame.
                self.worker=None;self.reset_walk(p.feet);
            }
            while self.trail.front().is_some_and(|(at,airborne)| {let d=*at-p.feet;
                p.swimming && *airborne && d.z>45. || d.truncate().length()<8. && d.z.abs()<45.
            }) {self.trail.pop_front();}
            // Pursue a short, clear part of the recorded path continuously.
            // Braking at every 12-unit sample makes a delayed follower crawl
            // while the leader waits exposed to enemies. Grounded shortcuts
            // require supported dry floor. During an ordinary jump, pursue the
            // next clear airborne samples through the recorded landing instead
            // of braking in midair and repeatedly hopping on the same ramp.
            let airborne=!p.grounded && p.ledge.is_none() && !p.climbing() && p.rope.is_none();
            if !p.swimming && (p.grounded || airborne) {
                while self.trail.len()>1 {
                    let (at,next_airborne)=self.trail[1];let delta=at-p.feet;
                    let trace=world.body_trace(p.feet,at);
                    if delta.truncate().length()>48.+320.*lead || trace.start_solid
                        || trace.fraction<1. || !world.body_clear(at)
                        || arena.stage.interactions.hazardous(at) {break;}
                    if airborne {
                        // Stop at the first recorded landing. Ledge pulls and
                        // swim banks never enter this ordinary-jump branch.
                        if !self.trail[0].1 || delta.z.abs()>64. {break;}
                    } else {
                        if next_airborne || delta.z.abs()>18. {break;}
                        let segments=(delta.length()/12.).ceil().max(1.) as usize;
                        let supported=(0..=segments).all(|n| {
                            let point=p.feet.lerp(at,n as f32/segments as f32);
                            let floor=world.body_trace(point,point-Vec3::Z*24.);
                            !floor.start_solid && floor.fraction<1. && floor.normal.z>0.65
                                && !arena.stage.interactions.hazardous(point)
                        });
                        if !supported {break;}
                    }
                    self.trail.pop_front();
                }
            }
        }
        let goal=if local!=first && !precise {self.trail.front().map(|(p,_)|*p).unwrap_or(goals[i])}else{goals[i]};
        let d=goal-p.feet;
        if arena.characters[i].attack_movement_locked() {return Ok((c,false));}
        let gap=p.feet-leader.feet;
        if !precise && local!=first && !ready[first.index()] && gap.truncate().length()<80. && gap.z.abs()<70. {
            // Yield without discarding the recorded approach. In particular,
            // the leader may jump while this input is in transit: losing its
            // last grounded point makes the follower jump from farther down
            // a slope toward an airborne sample instead of the takeoff.
            let forward=(goals[first.index()]-leader.feet).truncate().try_normalize().unwrap_or(Vec2::X);
            let carries=gap.z < -(PLAYER_HALF.z*2.-0.1) && gap.truncate().length()<crate::coop::separation::DISTANCE;
            if p.grounded && leader.ledge.as_ref().is_some_and(|h|!h.pulling) {
                // A caught lip may require releasing backwards before going
                // around. Give that ordinary escape room instead of standing
                // immediately behind the hanging leader and blocking its plan.
                let away=gap.truncate().try_normalize().unwrap_or(-forward);
                let side=vec2(-away.y,away.x);
                c.wish=clear_wish(&[away,side,-side],Some(gap.truncate()));
            } else if gap.truncate().dot(forward)>-4. || carries {let side=vec2(-forward.y,forward.x);
                // A diagonal approach can make all four relative directions
                // hit the sides of a narrow hall. Also try along its axes,
                // retaining the same floor, hazard and away-from-partner checks.
                c.wish=clear_wish(&[side,-side,forward,-forward,Vec2::X,-Vec2::X,Vec2::Y,-Vec2::Y],Some(gap.truncate()));
            } else if p.grounded && gap.truncate().length()<56. {
                // Leave room for a sideways doorway detour. Waiting at the
                // exact contact distance can block the leader's first planned
                // move even when the entire path is clear of level geometry.
                let away=gap.truncate().try_normalize().unwrap_or(-forward);
                let side=vec2(-away.y,away.x);
                c.wish=clear_wish(&[away,side,-side],Some(gap.truncate()));
            }return Ok((c,false));
        }
        if ready[i] {return Ok((c,false));}
        if let Some(w)=&self.worker {
            let outcome=match w.result.try_recv() {Ok(result)=>Some(result),Err(mpsc::TryRecvError::Empty)=>None,
                Err(mpsc::TryRecvError::Disconnected)=>Some(Err(0))};
            if let Some(result)=outcome {
                if self.index==Some(w.step) && self.phase==w.phase && frame.players[i].warped_at==w.warp && p.feet.distance(w.start)<24. {
                    self.plan=result.map_err(|n|anyhow::anyhow!("Pale Realm path exhausted {n} states"))?.into();self.plan_age=0;
                }self.worker=None;
            }
        }
        self.stuck_clock+=elapsed;
        if self.stuck_clock>=120 {
            self.stuck_clock%=120;
            // Shuffling along a blocked doorway or jumping in place is not
            // progress. Retain the best approach point so these motions cannot
            // keep resetting the stall clock and postpone a route search.
            self.stuck=if self.last.distance(goal)-p.feet.distance(goal)<16. {self.stuck+1}
                else {self.last=p.feet;0};
            if (2..4).contains(&self.stuck) && p.grounded && self.plan.is_empty() {c.jump=true;}
            // A caught lip can have no clear pull-up destination. Search from
            // that actual hanging state so ordinary release/shimmy inputs can
            // find a way around it, as the native route already does.
            if self.stuck>=4 && self.plan.is_empty() && self.worker.is_none()
                && (p.grounded || p.ledge.is_some()) {
                let (send,result)=mpsc::channel();let snapshot=world.clone();let player=p.clone();
                self.worker=Some(Worker{step:self.index.unwrap(),phase:self.phase,warp:frame.players[i].warped_at,start:p.feet,result});
                std::thread::spawn(move ||{let _=send.send(Route::plan_with_takeoff_margin(&snapshot,&player,goal,22.,if lead>0.{12}else{0}));});
            }
        }
        if self.worker.is_some() {return Ok((Controls::default(),false));}
        if let Some((input,_))=self.plan.front() {
            c=*input;c.jump &=self.plan_age==0;self.plan_age+=elapsed;
            if self.plan_age>=24 {self.plan.pop_front();self.plan_age=0;}
        } else {
            // A full round-trip velocity prediction reverses steering on an
            // already stale velocity, producing circles around small crumbs.
            // Instead reduce the approach gain as feedback delay grows. The
            // normal movement input and original arrival checks remain intact.
            let steering=d-p.velocity*lead.min(0.05);
            let approach=48_f32.max(320.*lead*2.);
            c.wish=steering.truncate().normalize_or_zero()*(steering.truncate().length()/approach).min(1.);
            c.swim=steering.normalize_or_zero()*if lead>0.{(steering.length()/approach).min(1.)}else{1.};
            c.rise=if p.swimming && d.z>30.{1.}else{0.};c.run=true;
            // A shore breadcrumb can be almost directly above the swimmer.
            // Keep a deliberate direction while requesting the normal bank
            // climb; analog arrival braking otherwise falls below its intent
            // threshold and holds the follower underneath the landing.
            if c.rise>0. {c.wish=d.truncate().normalize_or_zero();}
            c.jump|=p.ledge.as_ref().is_some_and(|h|!h.pulling) || p.grounded && d.z>24. && d.truncate().length()<330.;
        }Ok((c,false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landed_follower_rejoins_nearest_supported_sample_without_moving_the_body() {
        let world=World::fixture(&[(vec3(-100.,-100.,-20.),vec3(100.,100.,0.))]);
        let mut p=Player::new(vec3(0.,0.,0.03125));p.grounded=true;
        let nearest=p.feet+Vec3::X*6.;let later=p.feet+Vec3::X*30.;
        let mut trail=VecDeque::from([(p.feet+Vec3::Z*346.,true),
            (later,false),(nearest,false),(nearest+Vec3::Z*6.,true),(later,false)]);
        let before=serde_json::to_value(&p).unwrap();
        assert!(rejoin_landing(&mut trail,&p,&world,|_|false));
        assert_eq!(trail,VecDeque::from([(nearest,false),(nearest+Vec3::Z*6.,true),(later,false)]));
        assert_eq!(serde_json::to_value(&p).unwrap(),before);
    }

    #[test]
    fn landing_rejoin_rejects_walls_gaps_liquid_and_hazards() {
        let floor=(vec3(-100.,-100.,-20.),vec3(100.,100.,0.));
        let mut p=Player::new(vec3(-8.,0.,0.03125));p.grounded=true;
        let trail=VecDeque::from([(p.feet+Vec3::Z*346.,true),(vec3(40.,0.,0.03125),false)]);
        let wall=World::fixture(&[floor,(vec3(18.,-100.,0.),vec3(22.,100.,100.))]);
        let gap=World::fixture(&[(floor.0,vec3(0.,100.,0.)),(vec3(48.,-100.,-20.),floor.1)]);
        let mut water=World::fixture(&[floor]);
        water.add_liquid(vec3(10.,-100.,-10.),vec3(30.,100.,10.),0x20,None);
        for world in [wall,gap,water] {
            let mut path=trail.clone();assert!(!rejoin_landing(&mut path,&p,&world,|_|false));assert_eq!(path,trail);
        }
        let mut path=trail.clone();
        assert!(!rejoin_landing(&mut path,&p,&World::fixture(&[floor]),|at|at.x>10. && at.x<30.));
        assert_eq!(path,trail);
    }

    #[test]
    fn landing_rejoin_preserves_required_jumps_and_other_traversal() {
        let world=World::fixture(&[(vec3(-100.,-100.,-20.),vec3(100.,100.,0.))]);
        let mut p=Player::new(vec3(0.,0.,0.03125));p.grounded=true;
        for (front,landing) in [(p.feet+Vec3::Z*60.,p.feet+Vec3::X*10.),
            (p.feet+Vec3::Z*346.,p.feet+vec3(10.,0.,40.)),
            (p.feet+Vec3::Z*346.,p.feet+Vec3::X*49.)] {
            let original=VecDeque::from([(front,true),(landing,false)]);let mut path=original.clone();
            assert!(!rejoin_landing(&mut path,&p,&world,|_|false));assert_eq!(path,original);
        }
        for (grounded,swimming) in [(false,false),(true,true)] {
            p.grounded=grounded;p.swimming=swimming;
            let original=VecDeque::from([(p.feet+Vec3::Z*346.,true),(p.feet+Vec3::X*10.,false)]);
            let mut path=original.clone();
            assert!(!rejoin_landing(&mut path,&p,&world,|_|false));assert_eq!(path,original);
        }
    }

    #[test]
    fn delayed_lift_observer_catches_up_after_peer_leaves_the_ready_area() {
        // Either peer may see assembly readiness first. The later observer
        // catches up from boarding even though assembly is no longer true.
        for early in 0..2 {
            let mut phases=[0;2];
            phases[early]=lift_phase(phases[early],true,false,false);
            assert_eq!(phases[early],1);
            phases[1-early]=lift_phase(phases[1-early],false,true,false);
            assert_eq!(phases,[1,1]);
            // At the upper stop each rider can leave after their own grounded
            // arrival, including when the peer has already left its berth.
            phases[early]=lift_phase(phases[early],false,true,true);
            phases[1-early]=lift_phase(phases[1-early],false,true,true);
            assert_eq!(phases,[2,2]);
            assert_eq!(lift_phase(phases[early],false,false,false),2);
        }
        assert_eq!(lift_phase(0,false,false,false),0);
        assert_eq!(lift_phase(1,false,true,false),1);
    }

    #[test]
    fn nearby_landing_and_takeoff_remain_distinct_between_recorded_jumps() {
        // The full public route omitted this landing: it is 11.9 units from
        // the preceding airborne sample, inside the usual 12-unit interval.
        let falling=vec3(4000.348876953125,2329.49853515625,133.78106689453125);
        let landed=vec3(4000.14697265625,2319.125732421875,127.97212982177734);
        let takeoff=landed+Vec3::Z*6.;
        let mut trail=VecDeque::new();
        for sample in [(falling,true),(landed,false),(landed,false),
            (landed+Vec3::X*2.,false),(takeoff,true),(takeoff,true)] {
            record_trail(&mut trail,sample.0,sample.1);
        }
        assert_eq!(trail.into_iter().collect::<Vec<_>>(),vec![(falling,true),(landed,false),(takeoff,true)]);
    }
}
