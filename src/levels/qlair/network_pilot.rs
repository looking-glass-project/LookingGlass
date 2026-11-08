//! Optional local client test pilot; reads the received finale and emits inputs.
use super::*;
use crate::{coop::{arena::Arena,protocol::Input,world::Frame,PlayerId},route::fight};
#[derive(Default)]
pub(crate) struct NetworkPilot {
    camp:Option<Vec3>,tracker:fight::Tracker,stamp:Option<(u64,u64)>,strafe:f32,
}
impl NetworkPilot {
    pub(crate) fn sample(&mut self,frame:&Frame,arena:&Arena,world:&World,local:PlayerId,
        keys:&mut Input)->Result<Option<Vec3>> {
        let q=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Queen>())
            .context("Queen pilot outside finale")?;
        if q.saved.phase!=Phase::Queen2 || arena.stage.busy() {return Ok(None);}
        let i=local.index();let body=&frame.players[i];let p=&body.player;
        let camp=if let Some(at)=self.camp {at}else {
            let at=check::arena_camp(world,vec3(-120.,3784.,984.),Vec2::X)?
                +Vec3::X*if i==0 {-48.}else{48.};
            self.camp=Some(at);at
        };
        let targets=q.battle_targets();
        let context=crate::combat::Context{world,targets:&targets};
        if self.stamp!=Some((frame.epoch,frame.tick)) {
            let dt=self.stamp.filter(|s|s.0==frame.epoch && s.1<frame.tick)
                .map_or(1./120.,|s|(frame.tick-s.1)as f32/120.);
            self.tracker.observe(&targets,dt);self.stamp=Some((frame.epoch,frame.tick));
        }
        let point=targets.iter().find(|t|t.id==BASE+1)
            .and_then(|t|fight::aim_point(&context,p.eye(),t));
        let attack=if let Some(at)=point.filter(|_|body.stats.copies(7)>0 && body.stats.will()>21.) {
            fight::Plan{target:Some(BASE+1),aim:(at-p.eye()).normalize_or_zero(),weapon:7,alternate:false}
        }else {fight::plan(&context,p.eye(),&targets,&self.tracker,
            body.stats.copies(1)>0 && body.stats.will()>=fight::CARD_COST)};
        if attack.target.is_some() {
            keys.weapon=Some(attack.weapon as u8);keys.primary=!attack.alternate;keys.alternate=attack.alternate;
            keys.yaw=attack.aim.y.atan2(attack.aim.x);keys.pitch=attack.aim.z.asin().clamp(-1.56,1.56);
        }
        if p.feet.distance(camp)>95. {return Ok(Some(camp));}
        let from=p.feet.truncate()-camp.truncate();
        if self.strafe==0. {self.strafe=if i==0 {1.}else{-1.};}
        if from.x*self.strafe>35. {self.strafe=-self.strafe;}
        keys.wish=check::dodge_player(q,world,p,camp,Vec2::X,
            vec2(self.strafe,-from.y/40.).clamp_length_max(1.),false);
        keys.run=false;
        Ok(None)
    }
}
