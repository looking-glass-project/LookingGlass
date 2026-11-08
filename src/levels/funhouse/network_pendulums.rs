//! Opt-in ordinary test inputs across the bank and three moving pendulums.
//! Receipts observe actual transmitted/restored bodies; controls never relocate them.
use super::*;
use crate::coop::{arena::Arena, protocol::Input, world::Frame, PlayerId};

const BANK: Vec3 = vec3(-288., -3500., -32.);
const EXIT: Vec3 = vec3(640., -3500., 56.);

#[derive(Clone, Serialize)]
pub(crate) struct PendulumSupport {
    entity: usize,
    name: String,
    goal: Vec3,
    supported: [bool; 2],
}
#[derive(Clone, Serialize)]
pub(crate) struct PendulumReceipt {
    step: usize,
    start_tick: u64,
    start_jumps: u64,
    tick: u64,
    feet: Vec3,
    support: Option<PendulumSupport>,
}
#[derive(Clone, Serialize)]
pub(crate) struct PendulumReport {
    pub step: usize,
    pub reached: bool,
    start_tick: Option<u64>,
    start_jumps: u64,
    receipts: Vec<PendulumReceipt>,
}
pub(crate) struct NetworkPendulums {
    report: PendulumReport,
    context: (u64,u64,u64,u64),
    local: PlayerId,
    last_tick: Option<u64>,
    control_step: usize,
    runup: Option<u64>,
    jumped: Option<u64>,
    last_jump: Option<u64>,
    flight: f32,
    bank: Vec3,
    wish: Vec2,
}
fn name(step: usize) -> &'static str {
    match step {129=>"pend1",130=>"pend2",131=>"pend3",_=>unreachable!()}
}
fn point(o:&motion::Object, time:f32) -> Vec3 {
    let (depth,amp,phase)=match o.name.as_str() {
        "pend1"=>(-592.,24.,0.),"pend2"=>(-624.,12.,0.5),"pend3"=>(-560.,24.,0.),_=>unreachable!(),
    };
    let angle=(amp*((time/8.+phase)*std::f32::consts::TAU).sin()).to_radians();
    o.base.translation+Quat::from_rotation_x(angle)*vec3(0.,0.,depth)
}
fn supports(o:&motion::Object,p:&Player)->bool {
    let hit=o.collider.trace(p.feet+PLAYER_CENTER,p.feet+PLAYER_CENTER-Vec3::Z*3.,PLAYER_HALF);
    p.grounded && o.solid && !hit.start_solid && hit.fraction<1. && hit.normal.z>=0.65
}
fn arrived(p:&Player,start_jumps:u64,s:&PendulumSupport,slot:usize)->bool {
    p.grounded && p.jumps>start_jumps && s.supported[slot] && (p.feet.x-s.goal.x).abs()<65.
}
fn steer(goal:Vec3,p:&Player,lead:f32)->Vec2 {
    ((goal-p.feet-p.velocity*lead).truncate()/(40.+320.*lead)).clamp_length_max(1.)
}
fn bank_exit_ready(feet:Vec3,grounded:bool)->bool {
    let delta=EXIT-feet;
    let flight=(400.+(160000.-1600.*delta.z).max(0.).sqrt())/800.;
    grounded && delta.z<70. && delta.truncate().length()<flight*285.
}
impl NetworkPendulums {
    pub(crate) fn new(frame:&Frame,arena:&Arena,local:PlayerId)->Result<Self> {
        ensure!(arena.stage.interactions.levels.iter().any(|s|s.ctl.downcast_ref::<Funhouse>().is_some()),"Pendulums require Funhouse");
        let b=&frame.players[local.index()];
        ensure!(b.stats.alive() && b.player.grounded && !arena.stage.busy()
            && (b.player.feet-BANK).truncate().length()<80. && (b.player.feet.z-BANK.z).abs()<35.,
            "Pendulums require the supported near bank");
        Ok(Self {report:PendulumReport {step:128,reached:false,start_tick:None,start_jumps:0,receipts:Vec::new()},
            context:(frame.epoch,frame.checkpoint_generation,b.warped_at,b.spawned_at),local,
            last_tick:None,control_step:0,runup:None,jumped:None,last_jump:None,flight:0.,bank:b.player.feet,wish:Vec2::ZERO})
    }
    pub(crate) fn report(&self)->PendulumReport {self.report.clone()}
    pub(crate) fn record_state(&mut self,frame:&Frame,arena:&Arena) {
        let Some(o)=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Funhouse>()) else {return;};
        let b=&frame.players[self.local.index()];let p=&b.player;
        if self.report.reached || !b.stats.alive() || arena.stage.busy() || self.context!=
            (frame.epoch,frame.checkpoint_generation,b.warped_at,b.spawned_at) {return;}
        if self.report.start_tick.is_none() {
            self.report.start_tick=Some(frame.tick);self.report.start_jumps=p.jumps;
        }
        let start=self.report.start_tick.unwrap();
        let mut support=None;
        let reached=match self.report.step {
            128=>frame.tick-start>=1200 && p.grounded && p.jumps>self.report.start_jumps
                && (p.feet-BANK).truncate().length()<20. && (p.feet.z-BANK.z).abs()<35.,
            129..=131=> {
                let Some(q)=o.objects.iter().find(|q|q.name==name(self.report.step)) else {return;};
                let s=PendulumSupport {entity:q.id,name:q.name.clone(),goal:point(q,o.saved.time),
                    supported:frame.players.each_ref().map(|b|supports(q,&b.player))};
                let reached=arrived(p,self.report.start_jumps,&s,self.local.index());support=Some(s);reached
            }
            132=>p.grounded && p.jumps>self.report.start_jumps && (p.feet-EXIT).truncate().length()<20.
                && (p.feet.z-EXIT.z).abs()<52.,
            _=>false,
        };
        if reached {
            self.report.receipts.push(PendulumReceipt {step:self.report.step,start_tick:start,
                start_jumps:self.report.start_jumps,tick:frame.tick,feet:p.feet,support});
            self.report.step+=1;self.report.start_tick=None;self.report.reached=self.report.step==133;
        }
    }
    pub(crate) fn sample(&mut self,frame:&Frame,arena:&Arena,lead:f32,keys:&mut Input)->Result<()> {
        let o=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Funhouse>())
            .context("Pendulum course left Funhouse")?;
        let b=&frame.players[self.local.index()];let p=&b.player;
        ensure!(b.stats.alive() && !arena.stage.busy() && self.context==
            (frame.epoch,frame.checkpoint_generation,b.warped_at,b.spawned_at),"Pendulum course interrupted by death, relocation or scene");
        if self.last_tick!=Some(frame.tick) {
            ensure!(self.last_tick.is_none_or(|t|frame.tick>t && frame.tick-t<=360),"Pendulum observation clock changed");
            self.last_tick=Some(frame.tick);
            ensure!(self.report.start_tick.is_none_or(|t|frame.tick-t<120*45),"Pendulum stage timed out");
            if self.control_step!=self.report.step {
                self.control_step=self.report.step;self.runup=None;self.jumped=None;self.last_jump=None;self.bank=p.feet;
            }
            let mut jump=false;
            let goal=match self.report.step {
                128=> {
                    jump=self.report.start_tick.is_some_and(|t|frame.tick-t<1200) && p.grounded
                        && self.last_jump.is_none_or(|t|frame.tick-t>=120);
                    BANK
                }
                129..=131=> {
                    let q=o.objects.iter().find(|q|q.name==name(self.report.step)).context("Missing pendulum")?;
                    let point=|ahead:f32|point(q,o.saved.time+ahead);
                    if let Some(start)=self.jumped {
                        point((self.flight-(frame.tick.saturating_sub(start)) as f32/120.).max(0.)+lead)
                    } else {
                        self.flight=(400.+(160000.-1600.*(point(lead).z-p.feet.z)).max(0.).sqrt())/800.;
                        for _ in 0..3 {
                            self.flight=(400.+(160000.-1600.*(point(self.flight+lead).z-p.feet.z)).max(0.).sqrt())/800.;
                        }
                        let target=point(self.flight+lead+0.1);let delta=target-p.feet;
                        if self.runup.is_some() || p.grounded && delta.truncate().length()<self.flight*285. && delta.z<70. {
                            let start=*self.runup.get_or_insert(frame.tick);
                            if frame.tick>=start+12 {
                                jump=true;self.jumped=Some(frame.tick+(lead*120.) as u64);
                            }
                            target
                        } else {
                            o.objects.iter().find(|q|matches!(q.name.as_str(),"pend1"|"pend2"|"pend3") && supports(q,p))
                                .map(|q|point_at_wait(q,o.saved.time,lead))
                                .unwrap_or(vec3(self.bank.x,target.y.clamp(-3750.,-3450.),self.bank.z))
                        }
                    }
                }
                132=> {
                    if self.jumped.is_some() {EXIT} else {
                        let q=o.objects.iter().find(|q|q.name=="pend3").context("Missing exit pendulum")?;
                        // The platform keeps carrying the rider while a remote
                        // input is in flight. Check the actual predicted takeoff
                        // position, including the short ordinary running start.
                        let departure=p.feet+point(q,o.saved.time+lead+0.1)-point(q,o.saved.time);
                        if self.runup.is_some() || self.report.start_tick.is_some() && bank_exit_ready(departure,p.grounded) {
                            let start=*self.runup.get_or_insert(frame.tick);
                            if frame.tick>=start+12 {jump=true;self.jumped=Some(frame.tick);}
                            EXIT
                        } else {point_at_wait(q,o.saved.time,lead)}
                    }
                }
                _=>p.feet,
            };
            if p.ledge.as_ref().is_some_and(|l|!l.pulling) && self.last_jump.is_none_or(|t|frame.tick-t>=120) {jump=true;}
            if jump {keys.jumps+=1;self.last_jump=Some(frame.tick);}
            self.wish=if self.report.reached {Vec2::ZERO} else {steer(goal,p,lead)};
        }
        keys.wish=self.wish;keys.swim=self.wish.extend(0.);keys.run=true;
        Ok(())
    }
}
fn point_at_wait(q:&motion::Object,time:f32,lead:f32)->Vec3 {point(q,time+lead)+Vec3::X*32.}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pendulum_receipt_requires_an_actual_jump_and_own_support() {
        let mut p=Player::new(Vec3::ZERO);p.grounded=true;p.jumps=2;
        let mut s=PendulumSupport {entity:1,name:"pend1".into(),goal:Vec3::ZERO,supported:[true,false]};
        assert!(arrived(&p,1,&s,0));assert!(!arrived(&p,1,&s,1));assert!(!arrived(&p,2,&s,0));
        p.grounded=false;assert!(!arrived(&p,1,&s,0));p.grounded=true;p.feet.x=65.;assert!(!arrived(&p,1,&s,0));
        p.feet=Vec3::ZERO;s.supported=[false,true];assert!(!arrived(&p,1,&s,0));
    }
    #[test]
    fn far_bank_jump_waits_for_a_reachable_departure() {
        // Captured failed remote departure: the moving platform carried Red
        // backwards before the immediate jump input arrived.
        assert!(!bank_exit_ready(vec3(360.4616,-3708.8477,72.1587),true));
        assert!(!bank_exit_ready(vec3(363.0183,-3624.1406,55.8086),true));
        assert!(bank_exit_ready(vec3(416.,-3500.,40.),true));
        assert!(!bank_exit_ready(vec3(416.,-3500.,40.),false));
        assert!(!bank_exit_ready(vec3(416.,-3500.,-32.),true));
    }
}
