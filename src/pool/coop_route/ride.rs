//! Collision-aware ordinary steering and jumps for a moving leaf's obstacles.
use super::*;
impl Driver {
    pub(super) fn ride(&mut self,pool:&Pool,world:&World,players:&[Body;2],i:usize,index:usize,clock:f32)->Controls {
        let name=format!("rideleaf{}obj",index+1);
        let leaf=pool.objects.iter().find(|o|o.name==name).unwrap();
        let p=&players[i].player;
        let ideal=vec2(if i==0 {-24.}else{24.},0.);
        if self.age==0 {self.aim=ideal;}
        if self.age%12==0 && p.grounded {
            let mut best=None;
            for x in [-48.,-24.,0.,24.,48.] {for y in [-32.,0.,32.] {
                let off=vec2(x,y);
                let Some((at,_))=leaf.collider.rider_feet(leaf.pose.0+off.extend(8.)) else {continue;};
                if !world.body_clear(at) || (at-players[1-i].player.feet).truncate().length()<38. {continue;}
                let path=world.body_trace(p.feet,at);
                if path.start_solid || path.fraction<1. {continue;}
                let local=leaf.pose.1.inverse()*(at-leaf.pose.0);
                let mut previous=at;
                let safe=(1..=10).take_while(|n| {
                    let pose=pool.paths[index].pose(clock+*n as f32*0.15);
                    let next=pose.0+pose.1*local;
                    let hit=world.body_trace(previous,next);previous=next;
                    !hit.start_solid && hit.fraction>=1. && world.body_clear(next)
                }).count();
                let score=safe as f32*1000.-at.distance(p.feet)-off.distance(ideal)*0.2;
                if best.is_none_or(|(_,s)|score>s) {best=Some((off,score));}
            }}
            if let Some((aim,_))=best {self.aim=aim;}
        }
        let future=pool.paths[index].pose(clock+0.3);
        let carried=future.0+future.1*leaf.pose.1.inverse()*(p.feet-leaf.pose.0);
        let ahead=world.body_trace(p.feet,carried);
        let jump=p.grounded && self.age.saturating_sub(self.last_jump)>90
            && (ahead.start_solid || ahead.fraction<1.);
        if jump {self.last_jump=self.age;}
        let center=if !p.grounded || jump {future.0}else{leaf.pose.0};
        let d=center+self.aim.extend(0.)-p.feet;
        self.age+=1;
        Controls{wish:(d.truncate()/50.).clamp_length_max(1.),swim:d.normalize_or_zero(),
            rise:if p.immersion.level>0 {1.}else{0.},run:true,jump,..Default::default()}
    }
}
