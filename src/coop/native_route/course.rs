//! Independent waypoint cursors keep the following player on the same path.
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Step {
    pub at: [f32; 3],
    pub mode: String,
    #[serde(default)] pub ticks: u64,
    #[serde(default,skip_serializing_if="Option::is_none")] pub clock: Option<Clock>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Clock {pub controller:String,pub pointer:String,pub phase:f64,pub period:f64}
impl Clock {
    pub(super) fn ready(&self,arena:&Arena)->Result<bool> {
        ensure!(self.period.is_finite() && self.period>0.,"Invalid walking course period");
        let slot=arena.stage.interactions.levels.iter().find(|s|s.reg.id==self.controller)
            .context("Missing walking course clock controller")?;
        let state=slot.ctl.snapshot();
        let value=state.pointer(&self.pointer).and_then(|v|v.as_f64()).context("Missing walking course clock")?;
        Ok((value.rem_euclid(self.period)-self.phase).abs()<f64::from(FIXED_DT)*0.75)
    }
}
#[derive(Clone, Copy, Default)]
pub(super) struct Walker { pub cursor: usize, age: u64, jumped: bool, last: Option<Vec3> }
impl Walker {
    pub(super) fn input(&mut self, player:&crate::movement::Player, steps:&[Step], partner_done:bool,
        clock:Option<bool>, boarding:Option<&machine::Pose>)->Controls {
        let Some(step)=steps.get(self.cursor) else {return Controls::default()};
        if step.mode.starts_with("drive") {
            if self.age>=step.ticks || step.mode=="drive_ground_jump" && self.age==0 && !player.grounded {
                self.cursor+=1;self.age=0;self.jumped=false;self.last=None;return Controls::default();
            }
            let first=self.age==0;self.age+=1;
            let wish=vec2(step.at[0],step.at[1]);
            return Controls{wish,swim:wish.extend(0.),rise:step.at[2],run:true,
                jump:first && step.mode.contains("jump"),use_pressed:first && step.mode.contains("use"),..Default::default()};
        }
        let goal=Vec3::from_array(step.at);
        let delta=boarding.map_or(goal,|pose|pose.center+pose.rotation*goal)-player.feet;
        let close=if self.cursor+1==steps.len() && partner_done {70.}else{22.};
        let steering=step.mode=="steer";
        let done=if let Some(ready)=clock {ready}
        else if let Some(pose)=boarding {pose.supports(player)}
        else if step.mode=="align" {delta.truncate().length()<2. && delta.z.abs()<24.
            && player.grounded && player.velocity.truncate().length()<10.}
        else if step.mode=="rise" {player.grounded && player.feet.z>=goal.z}
        else if step.mode=="wait" {self.age>=step.ticks} else {
            delta.truncate().length()<close && delta.z.abs()<(if steering {48.}else{64.})
                && (steering || step.mode=="air" || player.grounded || player.swimming)
        };
        if done {self.cursor+=1;self.age=0;self.jumped=false;self.last=None;return Controls::default();}
        let stuck=steering && self.age%120==119 && self.last.is_some_and(|at|player.feet.distance(at)<18.);
        if self.age%120==0 {self.last=Some(player.feet);}
        self.age+=1;
        if clock.is_some() || matches!(step.mode.as_str(),"wait"|"rise") {return Controls::default();}
        let jump=stuck || player.ledge.as_ref().is_some_and(|h|!h.pulling) || player.grounded
            && (boarding.is_some() && step.mode!="pond_board" && !self.jumped || step.mode=="hop"
                || matches!(step.mode.as_str(),"jump"|"air"|"precise") && (!self.jumped || delta.z>24.));
        self.jumped|=jump;
        Controls{wish:if step.mode=="align" {(delta.truncate()/80.).clamp_length_max(1.)}
        else if step.mode=="pond_board" {(delta.truncate()/100.).clamp_length_max(1.)}
        else if boarding.is_some() && delta.truncate().length()<60. {
            (delta.truncate()*4.-player.velocity.truncate()*0.5).clamp_length_max(320.)/320.
        } else if step.mode=="precise" || step.mode=="air" && delta.truncate().length()<30. {
            (delta.truncate()*3.-player.velocity.truncate()*0.7).clamp_length_max(320.)/320.
        } else {delta.truncate().normalize_or_zero()},swim:delta.normalize_or_zero(),
            rise:if steering || matches!(step.mode.as_str(),"align"|"pond_board") {
                if player.immersion.level>0 {1.}else{0.}
            }else{delta.z.signum()},run:true,jump,..Default::default()}
    }
    pub(super) fn rejoin(&mut self, partner:Self) { *self=Self{cursor:partner.cursor,..Default::default()}; }
}
