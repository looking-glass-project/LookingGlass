//! Ordinary test controls for crossing the timed corridor, one player at a time.
//! Forecasts query native spike hulls; no live pose, clock or damage is changed.
use super::*;
use super::motion::waves;
use crate::movement::FIXED_DT;

#[derive(Default,Serialize)]
pub(super) struct Crossing { committed:bool, next_check:f32, attempts:u32 }

fn spikes(realm:&Realm)->impl Iterator<Item=(u8,&motion::Object)> {
    realm.objects.iter().filter_map(|o|o.name.strip_prefix("op1_spike")
        .and_then(|s|s.parse().ok()).map(|id|(id,o)))
}
pub(super) fn hazardous_after(realm:&Realm,feet:Vec3,after:f32)->bool {
    let age=realm.saved.age+after;
    let Some(start)=realm.saved.spikes.filter(|_|realm.saved.spikes_off.is_none_or(|off|age-off<1.)) else{return false;};
    spikes(realm).any(|(id,o)|waves::position(id,age-start).is_some_and(|pos| {
        // Translating the query oppositely is equivalent to moving this hull.
        let point=feet+PLAYER_CENTER-(Vec3::from_array(pos)-o.pose.translation);
        o.volume.trace(point,point,PLAYER_HALF).start_solid
    }))
}
fn in_corridor(realm:&Realm,feet:Vec3)->bool {
    spikes(realm).any(|(id,o)|waves::WAVE.iter().filter(|(i,_,_)|*i==id).any(|(_,_,pos)| {
        let low=feet+PLAYER_CENTER-(Vec3::from_array(*pos)-o.pose.translation);
        o.volume.touches(low,low-Vec3::Z*160.,PLAYER_HALF)
    }))
}
pub(super) fn arrived(realm:&Realm,p:&Player,hold:Vec3,goal:Vec3,radius:f32)->bool {
    let direction=(goal-hold).truncate().normalize_or_zero();
    let d=(p.feet-goal).truncate();
    // The exit is a crossing plane. A delayed stop can carry the leader beyond
    // the goal; that must not trap the waiting partner behind the spikes.
    p.grounded && d.dot(direction)>=-radius && d.perp_dot(direction).abs()<80.
        && (p.feet.z-goal.z).abs()<45. && !in_corridor(realm,p.feet)
}
fn walking(p:&Player,goal:Vec3,approach:bool)->Controls {
    let d=(goal-p.feet).truncate();
    Controls{wish:d.normalize_or_zero()*if approach{(d.length()/48.).min(1.)}else{1.},
        run:true,..Default::default()}
}
fn approach(p:&Player,hold:Vec3,lead:f32)->Controls {
    let mut c=walking(p,hold,true);
    // Keep at most twelve units of queued approach travel. Otherwise a remote
    // follower can overshoot this safe anchor into the first spike before its
    // neutral input arrives, without ever reaching the crossing planner.
    c.wish=c.wish.clamp_length_max((12./(320.*(lead+0.2))).min(0.2));
    c
}
impl Crossing {
    fn can_cross(&mut self,realm:&Realm,world:&World,p:&Player,hold:Vec3,goal:Vec3,lead:f32)->bool {
        if realm.saved.age<self.next_check {return false;}
        self.next_check=realm.saved.age+0.1;self.attempts+=1;
        let mut future=p.clone();
        let starts=((lead+0.2)/0.05).ceil() as usize;
        for tick in 1..=360 {
            future.tick(world,walking(&future,goal,false));
            if !world.body_clear(future.feet) {return false;}
            // Cover uncertain input arrival, including the stale-frame delay.
            for offset in 0..=starts {
                if hazardous_after(realm,future.feet,tick as f32*FIXED_DT+offset as f32*0.05) {return false;}
            }
            if arrived(realm,&future,hold,goal,20.) {return true;}
        }
        false
    }
    pub(super) fn controls(&mut self,realm:&Realm,world:&World,p:&Player,leader:&Player,
        first:bool,hold:Vec3,goal:Vec3,lead:f32)->(Controls,bool) {
        if arrived(realm,p,hold,goal,if first{24.}else{80.}) {return (Controls::default(),true);}
        if !first && !arrived(realm,leader,hold,goal,24.) {return (Controls::default(),false);}
        if !self.committed {
            if (hold-p.feet).truncate().length()>12. {return (approach(p,hold,lead),false);}
            if !p.grounded || p.velocity.truncate().length()>5. || realm.saved.spikes.is_none()
                || !self.can_cross(realm,world,p,hold,goal,lead) {return (Controls::default(),false);}
            self.committed=true;
        }
        (walking(p,goal,false),false)
    }
}
