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
pub(super) struct Walker { pub cursor: usize, age: u64, jumped: bool }
impl Walker {
    pub(super) fn input(&mut self, player:&crate::movement::Player, steps:&[Step], partner_done:bool, clock:Option<bool>)->Controls {
        let Some(step)=steps.get(self.cursor) else {return Controls::default()};
        let delta=Vec3::from_array(step.at)-player.feet;
        let close=if self.cursor+1==steps.len() && partner_done {70.}else{22.};
        let done=if let Some(ready)=clock {ready} else if step.mode=="wait" {self.age>=step.ticks} else {
            delta.truncate().length()<close && delta.z.abs()<64.
                && (step.mode=="air" || player.grounded || player.swimming)
        };
        if done {self.cursor+=1;self.age=0;self.jumped=false;return Controls::default();}
        self.age+=1;
        if clock.is_some() || step.mode=="wait" {return Controls::default();}
        let jump=player.ledge.as_ref().is_some_and(|h|!h.pulling) || player.grounded
            && (step.mode=="hop" || matches!(step.mode.as_str(),"jump"|"air") && (!self.jumped || delta.z>24.));
        self.jumped|=jump;
        Controls{wish:if step.mode=="air" && delta.truncate().length()<30. {
            (delta.truncate()*3.-player.velocity.truncate()*0.7).clamp_length_max(320.)/320.
        } else {delta.truncate().normalize_or_zero()},swim:delta.normalize_or_zero(),
            rise:delta.z.signum(),run:true,jump,..Default::default()}
    }
    pub(super) fn rejoin(&mut self, partner:Self) { *self=Self{cursor:partner.cursor,..Default::default()}; }
}
