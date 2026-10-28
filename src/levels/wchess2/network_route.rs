//! Opt-in diagnostic controls derived independently from each client's view.
//! No authoritative body, resource, actor or progression writes are available.
use super::*;
use crate::{coop::{arena::Arena,protocol::Input,world::Frame,PlayerId},
    movement::FIXED_DT,route::{fight,Route}};
use std::{collections::VecDeque,sync::mpsc};
struct Worker { start:Vec3, result:mpsc::Receiver<std::result::Result<Vec<(Controls,Vec3)>,usize>> }
fn steering(delta:Vec2,velocity:Vec2,lead:f32)->Vec2 {
    // A delayed velocity correction must also lower the feedback gain. With
    // the local gain, a remote client repeatedly reverses at full speed around
    // a breadcrumb before either its observation or its new input catches up.
    ((delta-velocity*lead)/(48.+640.*lead)).clamp_length_max(1.)
}
fn approach_gap(world:&World,feet:Vec3,direction:Vec2,lead:f32)->bool {
    // A latency-length probe can land on the far platform and miss the gap.
    // Check the whole approach and allow for acceleration while input travels.
    let distance=40.+320.*lead;
    let samples=(distance/12.).ceil().max(1.) as usize;
    (1..=samples).any(|n| {
        let at=feet+direction.extend(0.)*(distance*n as f32/samples as f32);
        let floor=world.body_trace(at,at-Vec3::Z*32.);
        floor.start_solid || floor.fraction==1. || floor.normal.z<0.65
    })
}
fn follow_ground_trail(trail:&mut VecDeque<Vec3>,world:&World,p:&Player,lead:f32,
    hazardous:impl Fn(Vec3)->bool) {
    if !p.grounded || p.swimming || p.ledge.is_some() || p.climbing() || p.rope.is_some() {return;}
    while trail.len()>1 {
        let at=trail[1];let delta=at-p.feet;let trace=world.body_trace(p.feet,at);
        if delta.truncate().length()>48.+320.*lead || delta.z.abs()>18.
            || trace.start_solid || trace.fraction<1. || !world.body_clear(at) {break;}
        let segments=(delta.length()/12.).ceil().max(1.) as usize;
        if !(0..=segments).all(|n| {
            let point=p.feet.lerp(at,n as f32/segments as f32);
            let floor=world.body_trace(point,point-Vec3::Z*24.);
            !floor.start_solid && floor.fraction<1. && floor.normal.z>0.65
                && world.liquid_at(point+Vec3::Z)==0 && !hazardous(point)
        }) {break;}
        trail.pop_front();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delayed_arrival(delay:usize,old_gain:bool)->(f32,f32) {
        let world=World::fixture(&[(vec3(-1000.,-6000.,-164.),vec3(1000.,-4000.,-64.))]);
        // Actual stalled follower and oldest trail point from the public run.
        let mut p=Player::new(vec3(-311.54166,-4899.908,-63.96875));
        p.velocity=vec3(-215.91142,-196.79211,0.);p.grounded=true;
        let goal=vec3(-305.97296,-4869.8247,-63.96875);
        let mut observations=VecDeque::from(vec![p.clone();delay+1]);
        let mut inputs=VecDeque::from(vec![Controls::default();delay+1]);
        let lead=(2*delay+6) as f32*FIXED_DT;
        let mut command=Controls::default();
        for tick in 0..2400 {
            let seen=observations.pop_front().unwrap();
            if tick%6==0 {
                let d=(goal-seen.feet).truncate();
                let wish=if old_gain {(d-seen.velocity.truncate()*lead).clamp_length_max(48.)/48.}
                    else {steering(d,seen.velocity.truncate(),lead)};
                command=Controls{wish,run:true,..Default::default()};
            }
            inputs.push_back(command);p.tick(&world,inputs.pop_front().unwrap());
            observations.push_back(p.clone());
            assert!(p.grounded && world.body_clear(p.feet));
        }
        (p.feet.distance(goal),p.velocity.truncate().length())
    }
    #[test]
    fn delayed_castling_follower_settles_instead_of_reversing_forever() {
        let (error,speed)=delayed_arrival(17,true);
        assert!(error>8. || speed>25.,"Old controller unexpectedly settled: {error}/{speed}");
        for delay in [0,6,17,36] {
            let (error,speed)=delayed_arrival(delay,false);
            assert!(error<2. && speed<5.,"delay {delay}: error {error}, speed {speed}");
        }
    }
    fn delayed_boarding(old_probe:bool)->(bool,u64) {
        let world=World::fixture(&[
            // Bounds of the solid exit-approach brush in the level geometry.
            (vec3(-1312.,-160.,496.),vec3(-1184.,32.,512.)),
            (vec3(-1088.,-200.,200.),vec3(-960.,200.,512.)),
            (vec3(-2000.,-1000.,100.),vec3(0.,1000.,216.))]);
        let mut p=Player::new(vec3(-1231.4225,-69.87011,512.03125));p.grounded=true;
        let goal=vec3(-1024.,-100.,512.03125);
        let delay=17;let lead=(2*delay+6) as f32*FIXED_DT;
        let mut observations=VecDeque::from(vec![p.clone();delay+1]);
        let mut inputs=VecDeque::from(vec![Controls::default();delay+1]);
        let mut command=Controls::default();let mut jumped=None;let mut boarded=false;
        for tick in 0..480 {
            let seen=observations.pop_front().unwrap();
            if tick%6==0 {
                let d=(goal-seen.feet).truncate();let direction=d.normalize_or_zero();
                let gap=if old_probe {
                    let at=seen.feet+direction.extend(0.)*(40.+seen.velocity.truncate().length()*lead);
                    let floor=world.body_trace(at,at-Vec3::Z*32.);
                    floor.start_solid || floor.fraction==1. || floor.normal.z<0.65
                } else {approach_gap(&world,seen.feet,direction,lead)};
                let jump=seen.grounded && seen.feet.x< -1103. && gap
                    && jumped.is_none_or(|last|tick-last>=120);
                if jump {jumped=Some(tick);}
                command=Controls{wish:steering(d,seen.velocity.truncate(),lead),run:true,jump,..Default::default()};
            }
            inputs.push_back(command);p.tick(&world,inputs.pop_front().unwrap());
            observations.push_back(p.clone());
            boarded |= p.grounded && p.feet.x> -1103. && (p.feet.z-512.03125).abs()<1.;
        }
        (boarded,p.jumps)
    }
    #[test]
    fn delayed_castling_boarding_emits_a_jump_before_crossing_the_gap() {
        let old=delayed_boarding(true);
        let (boarded,jumps)=delayed_boarding(false);
        // The simple deck permits step-up recovery; the live arched platform
        // does not. Reproduce the missing jump here, and require an actual
        // jump/landing from the repair. The network replay verifies the arch.
        assert!(old.1==0 && boarded && jumps==1,"Old result {old:?}; new boarded {boarded}, jumps {jumps}");
    }
    #[test]
    fn castling_trail_lookahead_requires_a_clear_supported_safe_path() {
        let floor=(vec3(-100.,-100.,-100.),vec3(200.,100.,0.));
        let mut p=Player::new(vec3(0.,0.,0.03125));p.grounded=true;
        let points=VecDeque::from([vec3(12.,0.,p.feet.z),vec3(96.,0.,p.feet.z)]);
        let world=World::fixture(&[floor]);let mut trail=points.clone();
        follow_ground_trail(&mut trail,&world,&p,0.28,|_|false);
        assert_eq!(trail.len(),1);
        let gap=World::fixture(&[(floor.0,vec3(20.,100.,0.)),(vec3(80.,-100.,-100.),floor.1)]);
        let wall=World::fixture(&[floor,(vec3(45.,-100.,0.),vec3(50.,100.,100.))]);
        for blocked in [&gap,&wall] {
            let mut trail=points.clone();follow_ground_trail(&mut trail,blocked,&p,0.28,|_|false);
            assert_eq!(trail,points);
        }
        let mut trail=points.clone();
        follow_ground_trail(&mut trail,&world,&p,0.28,|at|at.x>40. && at.x<60.);
        assert_eq!(trail,points);
        p.grounded=false;
        follow_ground_trail(&mut trail,&world,&p,0.28,|_|false);
        assert_eq!(trail,points);
    }
}
pub(crate) struct NetworkDriver {
    index:Option<usize>,last_tick:Option<u64>,ticks:u64,done:bool,scene_seen:bool,
    trail:VecDeque<Vec3>,last:Vec3,stuck:u32,stuck_clock:u64,
    worker:Option<Worker>,plan:VecDeque<(Controls,Vec3)>,plan_age:u64,
    tracker:fight::Tracker,cached:(Controls,Input),jump_at:Option<u64>,use_at:Option<u64>,
}
impl NetworkDriver {
    pub(crate) fn new()->Result<Self> {
        let course=check::COURSE.iter().enumerate().map(|(i,pos)|serde_json::json!({
            "mode":if i+1==check::COURSE.len(){"portal"}else{"walk"},"pos":pos})).collect::<Vec<_>>();
        std::fs::write(crate::preferences::path("wchess2-coop-course.json"),serde_json::to_vec(&course)?)?;
        Ok(Self{index:None,last_tick:None,ticks:0,done:false,scene_seen:false,trail:Default::default(),
            last:Vec3::ZERO,stuck:0,stuck_clock:0,worker:None,plan:Default::default(),plan_age:0,
            tracker:Default::default(),cached:Default::default(),jump_at:None,use_at:None})
    }
    pub(crate) fn reached(&self)->bool {self.done}
    pub(crate) fn report(&self)->serde_json::Value {
        serde_json::json!({"step":self.index,"reached":self.done,"observed_tick":self.last_tick,
            "ticks":self.ticks,"scene_seen":self.scene_seen,"planning":self.worker.is_some(),
            "plan_remaining":self.plan.len(),"stuck":self.stuck,"trail":self.trail,
            "input":self.cached.1,"movement":{"wish":self.cached.0.wish,"jump":self.cached.0.jump,
                "use":self.cached.0.use_pressed,"run":self.cached.0.run}})
    }
    pub(crate) fn spectate(&mut self,tick:u64) {
        self.last_tick=Some(tick);self.index=None;self.done=false;self.scene_seen=false;
        self.cached=Default::default();self.trail.clear();self.worker=None;self.plan.clear();
        self.tracker=Default::default();self.jump_at=None;self.use_at=None;
    }
    pub(crate) fn sample(&mut self,index:usize,frame:&Frame,arena:&Arena,world:&World,
        local:PlayerId,first:PlayerId,lead:f32,keys:&mut Input)->Result<()> {
        ensure!(index<check::COURSE.len(),"Invalid Castling course step");
        let castle=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Castle>())
            .context("Castling course used outside Castling")?;
        let body=&frame.players[local.index()];
        if self.index!=Some(index) {
            ensure!(self.index.is_none_or(|old|index==old+1),"Castling course skipped a step");
            self.index=Some(index);self.ticks=0;self.done=false;self.scene_seen=false;
            self.worker=None;self.plan.clear();self.plan_age=0;self.stuck=0;self.stuck_clock=0;
            self.last=body.player.feet;self.cached=Default::default();
        }
        let fresh=self.last_tick!=Some(frame.tick);
        if fresh {
            let elapsed=self.last_tick.map_or(1,|old|frame.tick.saturating_sub(old));
            ensure!((1..=360).contains(&elapsed),"Castling observation clock reset or stalled");
            self.last_tick=Some(frame.tick);self.cached=Default::default();
            if arena.stage.busy() {
                self.scene_seen |= arena.stage.owner.is_some();
                self.trail.clear();self.plan.clear();self.worker=None;self.stuck=0;
            } else {
                self.ticks+=elapsed;ensure!(self.ticks<120*180,"Castling course step {index} stalled");
                if self.scene_seen {self.done=true;}
                else {
                    let (controls,ready)=self.walk(index,frame,arena,world,castle,local,first,lead,elapsed)?;
                    self.cached.0=controls;self.done=ready;
                }
                self.combat(frame,arena,world,castle,local,elapsed as f32*FIXED_DT);
            }
        }
        let (c,i)=self.cached;
        keys.wish=c.wish;keys.run=c.run;keys.yaw=i.yaw;keys.pitch=i.pitch.clamp(-1.56,1.56);
        keys.weapon=i.weapon;keys.primary=i.primary;keys.alternate=i.alternate;
        if fresh && c.jump && self.jump_at.is_none_or(|t|frame.tick.saturating_sub(t)>=120) {
            keys.jumps+=1;self.jump_at=Some(frame.tick);
        }
        if fresh && c.use_pressed && self.use_at.is_none_or(|t|frame.tick.saturating_sub(t)>=30) {
            keys.uses+=1;self.use_at=Some(frame.tick);
        }
        keys.validate()
    }
    fn walk(&mut self,index:usize,frame:&Frame,arena:&Arena,world:&World,castle:&Castle,
        local:PlayerId,first:PlayerId,lead:f32,elapsed:u64)->Result<(Controls,bool)> {
        let p=&frame.players[local.index()].player;
        let leader=frame.players[first.index()].player.feet;let gap=p.feet-leader;
        let portal=index+1==check::COURSE.len();let course=Vec3::from_array(check::COURSE[index]);
        let goal=if portal {
            vec3(-1024.,if local==first{-100.}else{-28.},512.03125)
                +castle.portal_pose().translation-castle.data.point("exit_portal").translation
        }else{course};
        if local!=first && !portal {
            if self.trail.back().is_none_or(|at|at.distance(leader)>12.) {self.trail.push_back(leader);}
            while self.trail.front().is_some_and(|at|{let d=*at-p.feet;d.truncate().length()<8. && d.z.abs()<45.}) {self.trail.pop_front();}
            follow_ground_trail(&mut self.trail,world,p,lead,|at|arena.stage.interactions.hazardous(at));
            ensure!(self.trail.len()<2000,"Castling follower lost leader trail");
        }
        let arrival=if local!=first && !portal{leader-p.feet}else{goal-p.feet};
        let ready=arrival.truncate().length()<if portal{12.}else if local==first{22.}else{80.}
            && arrival.z.abs()<if portal{24.}else{60.} && p.grounded
            && (local!=first || lead==0. || p.velocity.truncate().length()<25.);
        if local!=first && !portal && gap.truncate().length()<72. && gap.z.abs()<60. {
            let forward=(course-leader).truncate().normalize_or_zero();
            let leader_ready=(course-leader).truncate().length()<22. && (course.z-leader.z).abs()<60.;
            let carrying=gap.z < -(PLAYER_HALF.z*2.-0.1) && gap.truncate().length()<crate::coop::separation::DISTANCE;
            if !leader_ready && (gap.truncate().dot(forward)>-4. || carrying) {
                let side=vec2(-forward.y,forward.x);
                for wish in [side,-side,forward,-forward] {
                    if gap.truncate().dot(wish)<0. {continue;}
                    let at=p.feet+wish.extend(0.)*12.;
                    let path=world.body_trace(p.feet,at);let floor=world.body_trace(at,at-Vec3::Z*32.);
                    if !path.start_solid && path.fraction==1. && !floor.start_solid && floor.fraction<1.
                        && floor.normal.z>0.65 && !arena.stage.interactions.hazardous(at) {
                        self.trail.clear();self.plan.clear();self.worker=None;
                        return Ok((Controls{wish,..Default::default()},false));
                    }
                }
            }
            return Ok((Controls::default(),ready));
        }
        if ready {self.plan.clear();self.worker=None;return Ok((Controls::default(),true));}
        let target=if local!=first && !portal {self.trail.front().copied().unwrap_or(goal)}else{goal};
        if let Some(worker)=&self.worker {
            match worker.result.try_recv() {
                Ok(result)=> {
                    if p.feet.distance(worker.start)<48. {
                        self.plan=result.map_err(|n|anyhow::anyhow!("Castling local path step{index} exhausted {n} states"))?.into();
                        self.plan_age=0;
                    }
                    self.worker=None;
                }
                Err(mpsc::TryRecvError::Empty)=>return Ok((Controls::default(),false)),
                Err(mpsc::TryRecvError::Disconnected)=>anyhow::bail!("Castling route planner stopped"),
            }
        }
        self.stuck_clock+=elapsed;
        if self.stuck_clock>=120 {
            self.stuck_clock=0;self.stuck=if p.feet.distance(self.last)<8.{self.stuck+1}else{0};self.last=p.feet;
            if self.stuck>=2 && self.plan.is_empty() && p.grounded {
                let nearby=p.feet+(target-p.feet).clamp_length_max(120.);
                let world=world.clone();let start=p.clone();let (send,result)=mpsc::channel();
                self.worker=Some(Worker{start:p.feet,result});
                std::thread::spawn(move||{let _=send.send(Route::plan(&world,&start,nearby,22.));});
                return Ok((Controls::default(),false));
            }
        }
        if let Some((input,expected))=self.plan.front() {
            let mut c=*input;
            if self.plan.iter().all(|(c,_)|!c.jump) {
                let d=*expected-p.feet;
                c.wish=steering(d.truncate(),p.velocity.truncate(),lead);
                if p.grounded && d.truncate().length()<8. && d.z.abs()<35. {self.plan.pop_front();}
            } else {
                c.jump &= self.plan_age==0;self.plan_age+=elapsed;
                if self.plan_age>=24 {self.plan.pop_front();self.plan_age=0;}
            }
            return Ok((c,false));
        }
        let d=target-p.feet;
        // Anticipate the arrival of a jump input, without moving the observed body.
        let board_jump=portal && p.grounded && !castle.portal_support(p) && d.truncate().length()<330.
            && approach_gap(world,p.feet,d.truncate().normalize_or_zero(),lead);
        Ok((Controls{wish:steering(d.truncate(),p.velocity.truncate(),lead),run:true,
            jump:board_jump || p.ledge.as_ref().is_some_and(|h|!h.pulling),..Default::default()},false))
    }
    fn combat(&mut self,frame:&Frame,arena:&Arena,world:&World,castle:&Castle,local:PlayerId,dt:f32) {
        let b=&frame.players[local.index()];
        let mut targets=arena.npcs.targets();targets.extend(arena.stage.interactions.levels_targets());
        let foes=targets.iter().filter(|t|castle.hostile_target(t.id)).copied().collect::<Vec<_>>();
        self.tracker.observe(&foes,dt);
        let ctx=crate::combat::Context{world,targets:&targets};
        let mut attack=fight::plan(&ctx,b.player.eye(),&foes,&self.tracker,b.stats.copies(1)>0 && b.stats.will()>=fight::CARD_COST);
        if b.stats.copies(4)>0 && b.stats.will()>=0.75 && !b.player.swimming {
            if let Some((id,at))=foes.iter().filter_map(|t|fight::aim_point(&ctx,b.player.eye(),t).map(|p|(t.id,p)))
                .filter(|(_,p)|p.distance(b.player.eye())<360.)
                .min_by(|(_,a),(_,c)|a.distance_squared(b.player.eye()).total_cmp(&c.distance_squared(b.player.eye()))) {
                attack.weapon=4;attack.alternate=false;attack.target=Some(id);attack.aim=(at-b.player.eye()).normalize_or_zero();
            }
        }
        let (c,i)=&mut self.cached;i.yaw=b.yaw;i.pitch=b.pitch;
        if attack.target.is_some() {
            i.yaw=attack.aim.y.atan2(attack.aim.x);i.pitch=attack.aim.z.asin();i.weapon=Some(attack.weapon as u8);
            i.primary=!attack.alternate;i.alternate=attack.alternate;
            self.tracker.weapon=attack.weapon;self.tracker.last_aim=attack.aim;
        }else if c.wish.length_squared()>0. {i.yaw=c.wish.y.atan2(c.wish.x);i.pitch=0.;}
        let aim=c.wish.extend(0.).normalize_or_zero();
        if castle.use_door(world,b.player.eye(),aim).is_some()
            || arena.stage.interactions.closed_door_in_reach(world,b.player.eye(),aim).is_some() {
            c.use_pressed=true;i.yaw=aim.y.atan2(aim.x);i.pitch=0.;i.primary=false;i.alternate=false;
        }
    }
}
