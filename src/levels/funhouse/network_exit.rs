//! A continuous local test input sequence for the final seesaw's upper exit.
//! This reads this client's restored world and emits controls; it cannot move bodies.
use super::*;
use crate::coop::{arena::Arena, protocol::Input, world::Frame, PlayerId};

const APPROACH: Vec3 = vec3(1920., -3820., 150.);
const TIP: Vec3 = vec3(1920., -3860., 195.);
const UPPER: Vec3 = vec3(1920., -4050., 256.);

#[derive(Clone, Serialize)]
pub(crate) struct ExitReceipt {
    step: usize,
    tick: u64,
    feet: Vec3,
    surface: Option<RouteSurface>,
}

#[derive(Clone, Serialize)]
pub(crate) struct ExitReport {
    pub step: usize,
    pub reached: bool,
    start_tick: u64,
    start_jumps: u64,
    spawned_at: u64,
    receipts: Vec<ExitReceipt>,
}

pub(crate) struct NetworkExit {
    report: ExitReport,
    last_tick: Option<u64>,
    context: (u64, u64, u64, u64),
    counterweight: bool,
    jump_at: Option<u64>,
    wish: Vec2,
    local: PlayerId,
    recorded_start: bool,
}

fn at_surface(p: &Player, surface: &RouteSurface, slot: usize) -> bool {
    p.grounded && surface.supported[slot]
        && (p.feet - surface.goal).truncate().length() < 8.
        && (p.feet.z - surface.goal.z).abs() < 35.
}

fn settling(delta: Vec2, velocity: Vec2, lead: f32) -> Vec2 {
    ((delta - velocity * lead) / (48. + 640. * lead)).clamp_length_max(1.)
}

fn running(delta: Vec2, velocity: Vec2, lead: f32) -> Vec2 {
    let steering = delta - velocity * lead;
    // Brake with ordinary analogue input near the predicted stopping point.
    // Do not reverse just because the delayed velocity predicts an overshoot.
    if steering.dot(delta) < 0. { return Vec2::ZERO; }
    steering.normalize_or_zero() * ((2. * 420. * steering.length()).sqrt().min(320.) * 0.95 / 320.)
}

impl NetworkExit {
    pub(crate) fn new(frame: &Frame, arena: &Arena, local: PlayerId) -> Result<Self> {
        let o = arena.stage.interactions.levels.iter()
            .find_map(|s| s.ctl.downcast_ref::<Funhouse>()).context("Seesaw exit used outside Funhouse")?;
        let b = &frame.players[local.index()];
        ensure!(b.stats.alive() && !arena.stage.busy() && o.surface_supports(460, &b.player)
            && (b.player.feet.y + 3400.).abs() < 24. && (b.player.feet.x - 1920.).abs() < 96.
            && o.saved.fulcrums[62].x <= -43., "Seesaw exit needs its supported loaded-front approach");
        let other = &frame.players[1-local.index()];
        Ok(Self {
            report: ExitReport { step: 103, reached: false, start_tick: frame.tick,
                start_jumps: b.player.jumps, spawned_at: b.spawned_at, receipts: Vec::new() },
            last_tick: None, context: (frame.epoch, frame.checkpoint_generation, b.warped_at, b.spawned_at),
            counterweight: other.stats.alive() && o.surface_supports(460, &other.player) && other.player.feet.y > -3550.,
            jump_at: None, wish: Vec2::ZERO, local, recorded_start: false,
        })
    }

    pub(crate) fn report(&self) -> ExitReport { self.report.clone() }

    /// Observe only an actually sent/restored frame. Host input sampling also
    /// sees intermediate physics ticks that never cross the network.
    pub(crate) fn record_state(&mut self, frame: &Frame, arena: &Arena) {
        let Some(o) = arena.stage.interactions.levels.iter()
            .find_map(|s| s.ctl.downcast_ref::<Funhouse>()) else { return; };
        let b = &frame.players[self.local.index()];
        let p = &b.player;
        if !b.stats.alive() || arena.stage.busy() || self.context !=
            (frame.epoch, frame.checkpoint_generation, b.warped_at, b.spawned_at) { return; }
        if !self.recorded_start {
            self.report.start_tick=frame.tick;self.report.start_jumps=p.jumps;
            self.recorded_start=true;
        }
        if self.report.step <= 104 {
            let goal=if self.report.step==103 {APPROACH} else {TIP};
            if let Some(surface)=route_surface(o,460,goal,frame.players.each_ref().map(|b|&b.player)) {
                if at_surface(p,&surface,self.local.index()) {
                    self.report.receipts.push(ExitReceipt{step:self.report.step,tick:frame.tick,feet:p.feet,surface:Some(surface)});
                    self.report.step+=1;
                }
            }
        } else if self.report.step==105 {
            let delta=UPPER-p.feet;
            if p.grounded && p.jumps>self.report.start_jumps && delta.truncate().length()<20. && delta.z.abs()<52. {
                self.report.receipts.push(ExitReceipt{step:105,tick:frame.tick,feet:p.feet,surface:None});
                self.report.step=106;self.report.reached=true;
            }
        }
    }

    pub(crate) fn sample(&mut self, frame: &Frame, arena: &Arena, local: PlayerId, lead: f32, keys: &mut Input) -> Result<()> {
        let o = arena.stage.interactions.levels.iter()
            .find_map(|s| s.ctl.downcast_ref::<Funhouse>()).context("Seesaw exit left Funhouse")?;
        let b = &frame.players[local.index()];
        let p = &b.player;
        ensure!(self.context == (frame.epoch, frame.checkpoint_generation, b.warped_at, b.spawned_at)
            && b.stats.alive() && !arena.stage.busy(), "Seesaw exit interrupted by death, relocation or scene");
        let fresh = self.last_tick != Some(frame.tick);
        if fresh {
            ensure!(self.last_tick.is_none_or(|t| frame.tick > t && frame.tick-t <= 360), "Seesaw observation clock changed");
            ensure!(self.report.reached || frame.tick-self.report.start_tick < 120*30, "Seesaw exit timed out");
            self.last_tick = Some(frame.tick);
            self.wish = Vec2::ZERO;
            match self.report.step {
                103 => {
                    // The lone rider runs through 103 toward 104 continuously.
                    // A counterweighted board instead needs time to open the tip.
                    self.wish = if self.counterweight { settling((APPROACH-p.feet).truncate(), p.velocity.truncate(), lead) }
                        else { running((TIP-p.feet).truncate(), p.velocity.truncate(), lead) };
                }
                104 => {
                    let waiting = self.counterweight && o.saved.fulcrums[62].x < -30.;
                    let goal = if waiting { APPROACH } else { TIP };
                    self.wish = if self.counterweight { settling((goal-p.feet).truncate(), p.velocity.truncate(), lead) }
                        else { running((goal-p.feet).truncate(), p.velocity.truncate(), lead) };
                }
                105 => {
                    let delta = UPPER-p.feet;
                    self.wish = settling(delta.truncate(), p.velocity.truncate(), lead);
                    let pulling = p.ledge.as_ref().is_some_and(|l| !l.pulling);
                    if self.jump_at.is_none() || pulling && frame.tick-self.jump_at.unwrap() >= 120 {
                        keys.jumps += 1; self.jump_at = Some(frame.tick);
                    }
                }
                _ => (),
            }
        }
        keys.wish = self.wish; keys.swim = self.wish.extend(0.); keys.run = true;
        if !keys.primary && !keys.alternate && self.wish.length_squared() > 0. {
            keys.yaw = self.wish.y.atan2(self.wish.x); keys.pitch = 0.;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exit_receipt_requires_own_real_support_and_original_bounds() {
        let mut p=Player::new(APPROACH); p.grounded=true;
        let mut s=RouteSurface {entity:460,requested:APPROACH,goal:APPROACH,supported:[true,false]};
        assert!(at_surface(&p,&s,0)); assert!(!at_surface(&p,&s,1));
        p.grounded=false; assert!(!at_surface(&p,&s,0)); p.grounded=true;
        p.feet.z-=36.; assert!(!at_surface(&p,&s,0)); p.feet=APPROACH+Vec3::X*8.;
        assert!(!at_surface(&p,&s,0)); p.feet=APPROACH; s.supported=[false,true];
        assert!(!at_surface(&p,&s,0)); assert!(at_surface(&p,&s,1));
    }
    #[test]
    fn raised_tip_running_does_not_reverse_from_prediction_alone() {
        assert_eq!(running(vec2(0.,-20.),vec2(0.,-320.),0.3),Vec2::ZERO);
        assert!(running(vec2(0.,-200.),vec2(0.,-320.),0.3).y < -0.8);
        assert!(running(vec2(0.,20.),vec2(0.,-320.),0.3).y > 0.);
    }
}
