//! Read-only water navigation for accelerated ordinary-input route checks.
use crate::{collision::World, movement::Controls, water::Immersion};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;
use std::collections::{BinaryHeap, HashMap, VecDeque};

pub(super) fn fan_gap(map:&crate::bsp::Bsp, player:&crate::movement::Player,
    name:&str, angle:f32, speed:f32)->Result<bool> {
    // Like the solo route, look ahead through the live blade sweep before
    // pressing upward swim. The fan and the real body are never altered.
    let entity=map.entities.iter().find(|e|e.get("targetname").is_some_and(|n|n==name))
        .context("Missing swim fan")?;
    let model=entity.get("model").and_then(|s|s.strip_prefix('*')).context("Swim fan lacks model")?.parse()?;
    let base=entity.get("origin").and_then(|s|crate::interaction::vector(s)).context("Swim fan lacks origin")?;
    let mut feet=player.feet;
    let mut velocity=player.velocity;
    for n in 1..=110 {
        velocity+=(Vec3::Z*220.-velocity).clamp_length_max(700.*crate::movement::FIXED_DT);
        let next=feet+velocity*crate::movement::FIXED_DT;
        let blade=crate::collision::Collider::model(map,model,base,
            Quat::from_rotation_z((angle+speed*crate::movement::FIXED_DT*n as f32).to_radians()),true)?;
        let hit=blade.trace(feet+crate::collision::PLAYER_CENTER,next+crate::collision::PLAYER_CENTER,
            crate::collision::PLAYER_HALF);
        if hit.start_solid || hit.fraction<1. {return Ok(false);}
        feet=next;
    }
    Ok(true)
}

#[derive(Default)]
pub(super) struct Swimming {
    path: Option<VecDeque<Vec3>>,
    entry: Option<Vec3>,
}
impl Swimming {
    pub(super) fn input(&mut self, world:&World, player:&crate::movement::Player,
        goal:Vec3, planned:bool, close:f32)->Result<Controls> {
        // A follower can reach a column above the dive marker. Choose an open
        // bank edge before aiming down; swimming inputs cannot cross its floor.
        if player.immersion.level<2 && (self.entry.is_some()
            || player.grounded && goal.z<player.feet.z-32.
                && goal.truncate().distance(player.feet.truncate())<48.) {
            if self.entry.is_none() {self.entry=Some(entry(world,player.feet,goal)?);}
            let delta=self.entry.unwrap()-player.feet;
            return Ok(Controls{wish:delta.truncate().normalize_or_zero(),swim:delta.normalize_or_zero(),
                run:true,jump:player.ledge.as_ref().is_some_and(|h|!h.pulling),..Default::default()});
        }
        self.entry=None;
        if self.path.is_none() {self.path=Some(if planned {plan(world,player.feet,goal)?}
            else {VecDeque::from([goal])});}
        let points=self.path.as_mut().unwrap();
        while points.len()>1 && player.feet.distance(*points.front().unwrap())<16. {points.pop_front();}
        let delta=points.front().copied().unwrap_or(goal)-player.feet;
        if points.len()<=1 && delta.length()<close {return Ok(Controls::default());}
        Ok(Controls{wish:delta.truncate().normalize_or_zero(),
            swim:delta.normalize_or_zero()*(delta.length()/30.).min(1.),run:true,
            jump:player.ledge.as_ref().is_some_and(|h|!h.pulling),..Default::default()})
    }
}

fn entry(world:&World,start:Vec3,goal:Vec3)->Result<Vec3> {
    let clear=|a:Vec3,b:Vec3|{let t=world.body_trace(a,b);!t.start_solid && t.fraction>=1.};
    let mut best=None;
    let mut cost=f32::INFINITY;
    for x in -16..=16 {for y in -16..=16 {
        let above=start+vec3(x as f32*32.,y as f32*32.,0.);
        if !clear(start,above) {continue;}
        for z in 1..=24 {
            let at=above-Vec3::Z*(z as f32*32.);
            if !clear(above,at) {break;}
            if Immersion::sample(world,at).level<2 {continue;}
            let score=start.distance(at)+at.distance(goal);
            if score<cost {cost=score;best=Some(at);}
            break;
        }
    }}
    let at=best.with_context(||format!("No clear water entry from {start:?} toward {goal:?}"))?;
    println!("DIVE {start:?} via {at:?} toward {goal:?}");
    Ok(at)
}

fn plan(world:&World,start:Vec3,goal:Vec3)->Result<VecDeque<Vec3>> {
    const GRID:f32=32.;
    type Key=(i32,i32,i32);
    let at=|k:Key|vec3(k.0 as f32,k.1 as f32,k.2 as f32)*GRID;
    let key=|p:Vec3|((p.x/GRID).round() as i32,(p.y/GRID).round() as i32,(p.z/GRID).round() as i32);
    let clear=|a:Vec3,b:Vec3|{let t=world.body_trace(a,b);!t.start_solid && t.fraction>=1.};
    let end=key(goal);
    let distance=|k:Key|(k.0-end.0).abs()+(k.1-end.1).abs()+(k.2-end.2).abs();
    let origin=key(start);
    let low=start.min(goal)-vec3(1024.,1024.,512.);
    let high=start.max(goal)+vec3(1024.,1024.,512.);
    let mut queue=BinaryHeap::new();
    let mut parents=HashMap::new();
    let mut costs=HashMap::new();
    for x in -1..=1 {for y in -1..=1 {for z in -1..=1 {
        let k=(origin.0+x,origin.1+y,origin.2+z);
        if clear(start,at(k)) {
            parents.insert(k,None);costs.insert(k,0);queue.push((-distance(k),0,k));
        }
    }}}
    let mut found=None;
    let mut attempts=0;
    while let Some((_,neg,k))=queue.pop() {
        let cost=-neg;
        if costs.get(&k).is_some_and(|c|*c<cost) {continue;}
        attempts+=1;
        ensure!(attempts<250_000,"Swimming planner exhausted reachable water");
        if distance(k)<=3 && clear(at(k),goal) {found=Some(k);break;}
        for (x,y,z) in [(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,1),(0,0,-1)] {
            let n=(k.0+x,k.1+y,k.2+z);
            let point=at(n);
            if point.cmplt(low).any() || point.cmpgt(high).any()
                || costs.get(&n).is_some_and(|c|*c<=cost+1) {continue;}
            if Immersion::sample(world,point).level>=2 && clear(at(k),point) {
                costs.insert(n,cost+1);parents.insert(n,Some(k));
                queue.push((-(cost+1+distance(n)),-(cost+1),n));
            }
        }
    }
    let mut k=found.with_context(||format!("No swimming route from {start:?} to {goal:?}"))?;
    let mut path=vec![goal];
    loop {path.push(at(k));if let Some(n)=parents[&k] {k=n;}else{break;}}
    path.reverse();
    let mut points=VecDeque::new();
    let mut current=start;
    let mut cursor=0;
    while cursor<path.len() {
        let mut last=cursor;
        while last+1<path.len() && clear(current,path[last+1]) {last+=1;}
        current=path[last];points.push_back(current);cursor=last+1;
    }
    println!("SWIM {start:?} to {goal:?}, {attempts} nodes: {points:?}");
    Ok(points)
}
