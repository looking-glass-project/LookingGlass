//! Read-only pathnode planning and ordinary walking for grounded escorts.
use super::*;

fn edge(world:&World,a:Vec3,b:Vec3)->bool {
    let distance=a.distance(b);
    if distance>500. || (a.z-b.z).abs()>240. {return false;}
    let count=(distance/20.).ceil().max(1.) as usize;
    let mut previous=a;
    for i in 1..=count {
        let sample=a.lerp(b,i as f32/count as f32);
        let Some(foot)=world.actor_footing(sample+Vec3::Z*64.,PLAYER_CENTER,PLAYER_HALF,128.) else {return false};
        if (foot.z-previous.z).abs()>28. {return false;}
        let trace=world.body_trace(previous+Vec3::Z*18.,foot+Vec3::Z*18.);
        if trace.start_solid || trace.fraction<1. {return false;}
        previous=foot;
    }
    previous.distance(b)<12.
}
fn plan(map:&Bsp,world:&World,start:Vec3,goal:Vec3)->Result<VecDeque<Vec3>> {
    let mut points=vec![start];
    points.extend(map.entities.iter().filter(|e|e.get("classname").is_some_and(|s|
        matches!(s.as_str(),"info_pathnode"|"info_waypoint")))
        .filter_map(|e|e.get("origin").and_then(|s|crate::interaction::vector(s)))
        .filter_map(|at|world.actor_footing(at+Vec3::Z*24.,PLAYER_CENTER,PLAYER_HALF,128.)));
    points.push(world.actor_footing(goal+Vec3::Z*24.,PLAYER_CENTER,PLAYER_HALF,128.).unwrap_or(goal));
    let end=points.len()-1;
    let mut costs=vec![f32::INFINITY;points.len()];
    let mut parents=vec![usize::MAX;points.len()];
    let mut visited=vec![false;points.len()];costs[0]=0.;
    loop {
        let at=(0..points.len()).filter(|i|!visited[*i])
            .min_by(|a,b|costs[*a].total_cmp(&costs[*b])).context("Empty walking graph")?;
        ensure!(costs[at].is_finite(),"No supported walking trail from {start:?} to {goal:?}");
        if at==end {break;}
        visited[at]=true;
        for next in 0..points.len() {
            if !visited[next] && edge(world,points[at],points[next]) {
                let cost=costs[at]+points[at].distance(points[next]);
                if cost<costs[next] {costs[next]=cost;parents[next]=at;}
            }
        }
    }
    let mut path=VecDeque::new();let mut at=end;
    while at!=0 {path.push_front(points[at]);at=parents[at];}
    Ok(path)
}
#[derive(Default)]
pub(super) struct Walker { path:Option<VecDeque<Vec3>>, warped:Option<u64>, age:u64,
    previous:Option<Vec3>, recovering:bool, recoveries:u8, pub done:bool }
impl Walker {
    pub(super) fn input(&mut self,map:&Bsp,world:&World,body:&super::super::world::Body,
        goal:Vec3,escort:Option<&crate::level::CompanionContact>,portal:bool,direct:bool,partner_done:bool,exit:Option<Vec3>)->Result<Controls> {
        let p=&body.player;
        let warped=*self.warped.get_or_insert(body.warped_at);
        if portal && !self.done && body.warped_at!=warped {self.done=true;self.path=None;}
        let clearing=portal && self.done && !partner_done && exit.is_some();
        if self.done && !clearing {return Ok(Controls::default());}
        let goal=if clearing {exit.unwrap()}else{goal};
        let previous=*self.previous.get_or_insert(p.feet);
        self.age+=1;
        let stuck=self.age%120==0 && p.feet.distance(previous)<8.;
        if self.age%120==0 {self.previous=Some(p.feet);}
        if self.recovering && escort.is_none_or(|c|c.holding || c.feet.truncate().distance(p.feet.truncate())<100.) {
            self.recovering=false;self.path=None;
        }
        if !self.recovering && stuck && self.recoveries<3 {
            if let Some(child)=escort.filter(|c|!c.holding && c.feet.truncate().distance(p.feet.truncate())>100.) {
                // Re-establish sight after an occluded corner with ordinary
                // walking, as in the solo escort diagnostic.
                self.path=Some(plan(map,world,p.feet,child.feet)?);
                self.recovering=true;self.recoveries+=1;
                println!("TRAIL regain escort contact at {:?}",child.feet);
            }
        }
        if self.path.is_none() {
            if !p.grounded {return Ok(Controls::default());}
            self.path=Some(if (direct || portal) && !clearing {VecDeque::from([goal])}else{plan(map,world,p.feet,goal)?});
        }
        let path=self.path.as_mut().unwrap();
        while let Some(at)=path.front() {
            let d=*at-p.feet;
            let close=if path.len()==1 && partner_done {70.}else{18.};
            if (!portal || clearing) && d.truncate().length()<close && d.z.abs()<40. {path.pop_front();} else {break;}
        }
        let at=path.front().copied().unwrap_or(goal);
        let delta=at-p.feet;
        let waiting=!self.recovering && escort.is_some_and(|c|!c.holding && c.feet.truncate().distance(p.feet.truncate())>150.
            && (c.feet-p.feet).truncate().dot(delta.truncate())<=0.);
        if !portal {self.done=!self.recovering && path.is_empty() && escort.is_none_or(|c|c.holding
            || c.feet.truncate().distance(p.feet.truncate())<120.);}
        Ok(Controls{wish:if waiting || path.is_empty() {Vec2::ZERO}else{delta.truncate().normalize_or_zero()},
            run:escort.is_none() || self.recovering,jump:escort.is_none() && stuck,..Default::default()})
    }
}
