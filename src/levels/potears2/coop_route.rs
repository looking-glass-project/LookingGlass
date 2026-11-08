//! Ordinary paired pond traversal; live pad queries never edit world state.
use super::*;
use crate::{coop::{arena::Arena,world::Body,native_route::{Command,Navigation,Spec}},movement::Controls};
mod probe;

#[derive(Clone,Debug,Serialize,Deserialize)]
pub(crate) enum Step {
    Steer([f32;3]), Board(String), Ride{leaf:usize,until:f64}, Shore,
    Input{ticks:u64,direction:[f32;2],rise:f32,jump:bool,use_pressed:bool},
}
#[derive(Default)]
pub(crate) struct Driver {last:Option<Vec3>,aim:Vec2}

fn owner(arena:&Arena)->Result<&PoolTwo> {
    arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref())
        .context("Missing Hollow Hideaway controller")
}
fn aboard(pad:&transport::Pad,p:&Player)->bool {
    p.grounded && crate::moving::aboard(&pad.collider,p.feet)
}
fn deck(world:&World,pad:&transport::Pad,offset:Vec2)->Option<Vec3> {
    let from=pad.pose.translation+offset.extend(80.);
    let to=from-Vec3::Z*160.;
    let hit=pad.collider.trace(from+crate::collision::PLAYER_CENTER,to+crate::collision::PLAYER_CENTER,
        crate::collision::PLAYER_HALF);
    let at=from.lerp(to,hit.fraction);
    if hit.start_solid || hit.fraction>=1. || hit.normal.z<0.65 {return None;}
    let (at,_)=pad.collider.rider_feet(at)?;
    world.body_clear(at).then_some(at)
}
impl Driver {
    pub(crate) fn input(&mut self,arena:&Arena,world:&World,players:&[Body;2],i:usize,
        step:&Step,age:u64)->Result<(Controls,bool)> {
        let p=&players[i].player;
        let other=&players[1-i].player;
        let pond=owner(arena)?;
        let mut c=Controls{run:true,..Default::default()};
        let done=match step {
            Step::Steer(at)=>{
                let goal=Vec3::from_array(*at);let d=goal-p.feet;
                let close=if other.feet.distance_squared(goal)<p.feet.distance_squared(goal) {70.}else{20.};
                let done=d.truncate().length()<close && d.z.abs()<48. && (p.grounded||p.swimming);
                if !done {
                    let stuck=age%120==119 && self.last.is_some_and(|at|p.feet.distance(at)<18.);
                    if age%120==0 {self.last=Some(p.feet);}
                    c.wish=d.truncate().normalize_or_zero();c.swim=d.normalize_or_zero();
                    c.rise=if p.immersion.level>0 {1.}else{0.};
                    c.jump=stuck || p.ledge.as_ref().is_some_and(|h|!h.pulling);
                }
                done
            }
            Step::Board(name)=>{
                let pad=pond.pond.pads.iter().find(|p|&p.name==name).context("Unknown paired pond pad")?;
                let ideal=vec2(0.,if i==0 {-24.}else{24.});
                let at=deck(world,pad,ideal).unwrap_or(pad.pose.translation+ideal.extend(8.));
                let d=at-p.feet;
                let done=aboard(pad,p) && d.truncate().length()<50. && d.z.abs()<20.;
                c.wish=(d.truncate()/70.).clamp_length_max(1.);c.swim=d.normalize_or_zero();
                c.rise=if p.immersion.level>0 {1.}else{0.};
                c.jump=!aboard(pad,p) && age%90==0;
                done
            }
            Step::Ride{leaf,until}=>{
                ensure!((1..=2).contains(leaf) && until.is_finite() && *until>=0.,"Invalid paired leaf ride");
                let index=4+leaf;let pad=&pond.pond.pads[index];
                ensure!(pond.saved.pond.leaves[leaf-1].is_some(),"Paired leaf ride before boarding");
                if age==0 {self.aim=vec2(0.,if i==0 {-24.}else{24.});}
                if age%15==0 {
                    let mut best=None;
                    for x in [-64.,-32.,0.,32.,64.] {for y in [-40.,-20.,0.,20.,40.] {
                        let off=vec2(x,y);let Some(at)=deck(world,pad,off) else {continue;};
                        if players[1-i].stats.alive() && (at-other.feet).truncate().length()<40. {continue;}
                        let path=world.body_trace(p.feet,at);
                        if path.start_solid || path.fraction<1. {continue;}
                        let local=at-pad.pose.translation;
                        let mut previous=at;
                        let safe=(0..=8).take_while(|n| {
                            let next=pond.pond.future(index,pond.saved.pond.clocks[index]+*n as f64*0.2)+local;
                            let path=world.body_trace(previous,next);previous=next;
                            !path.start_solid && path.fraction>=1. && world.body_clear(next)
                        }).count();
                        let score=safe as f32*1000.-at.distance(p.feet);
                        if best.is_none_or(|(_,s)|score>s) {best=Some((off,score));}
                    }}
                    if let Some((aim,_))=best {self.aim=aim;}
                }
                let d=pad.pose.translation+self.aim.extend(0.)-p.feet;
                c.wish=(d.truncate()/50.).clamp_length_max(0.7);c.swim=d.normalize_or_zero();
                pond.saved.pond.clocks[index]>=*until
            }
            Step::Shore=>p.grounded && p.feet.x> -335. && (-1070. ..=-960.).contains(&p.feet.y),
            Step::Input{ticks,direction,rise,jump,use_pressed}=>{
                c.wish=Vec2::from_array(*direction);c.swim=c.wish.extend(0.);c.rise=*rise;
                c.jump=age==0 && *jump;c.use_pressed=age==0 && *use_pressed;age>=*ticks
            }
        };
        Ok((c,done))
    }
}
pub(crate) fn check(a:&mut Assets)->crate::levels::BoxFuture<'_> {
    Box::pin(async move {
        if let Some(path)=std::env::var_os("LOOKING_GLASS_POND_PROBE") {return probe::run(a,&path);}
        let steps:Vec<serde_json::Value>=serde_json::from_str(include!("route_input.rs"))?;
        let mut commands=vec![Command::Staff(false),Command::Cards(true),Command::Wait(120)];
        let mut bank=Vec::new();
        let mut shore_jump=false;
        for s in steps {
            if s.get("save").is_some() {continue;}
            // Two players use an ordinary diagonal jump from lily four to the
            // bank. Lily five's carry is covered by the transport checks.
            if s.get("board").and_then(|v|v.as_str())==Some("lily5") {shore_jump=true;continue;}
            if let Some(v)=s.get("steer") {
                bank.push(crate::coop::native_route::CourseStep{at:serde_json::from_value(v.clone())?,
                    mode:if shore_jump {"air"}else{"steer"}.into(),ticks:0,clock:None});
                shore_jump=false;
                continue;
            }
            if shore_jump && (s.get("shore").is_some() || s.get("input").is_some()) {continue;}
            if !bank.is_empty() {commands.push(Command::CourseFile(std::mem::take(&mut bank)));}
            let command=if let Some(v)=s.get("board") {Command::Pond(Step::Board(v.as_str().context("Invalid pad name")?.into()))}
            else if let Some(v)=s.get("ride") {let v:[f64;2]=serde_json::from_value(v.clone())?;
                Command::Pond(Step::Ride{leaf:v[0] as usize,until:v[1]})}
            else if s.get("shore").and_then(|v|v.as_bool())==Some(true) {Command::Pond(Step::Shore)}
            else if let Some(v)=s.get("input") {let v:[f32;6]=serde_json::from_value(v.clone())?;
                Command::Pond(Step::Input{ticks:(v[0]*120.) as u64,direction:[v[1],v[2]],rise:v[3],jump:v[4]>0.,use_pressed:v[5]>0.})}
            else if let Some(v)=s.get("nav") {Command::Nav(serde_json::from_value(v.clone())?)}
            else if let Some(v)=s.get("clear") {Command::Clear(v.as_f64().context("Invalid clearing radius")? as f32)}
            else {anyhow::bail!("Unknown paired pond course step");};
            commands.push(command);
        }
        if !bank.is_empty() {commands.push(Command::CourseFile(bank));}
        crate::coop::native_route::check(a,Spec{map:"potears2",entry:"potears2_start1",exit:EXIT,
            navigation:Navigation::LandedPair,commands,verify:|arena,_| {
                let s=owner(arena)?;
                ensure!(s.saved.pond.leaves.iter().all(Option::is_some) && s.saved.pond.ladies
                    && s.saved.fish.attack.is_none() && s.ready() && s.saved.scene.phase==Phase::Done,
                    "Incomplete normal paired pond route");Ok(())
            }}).await
    })
}
