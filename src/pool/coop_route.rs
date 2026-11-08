//! Normal two-player inputs over the authored banks, leaves and rope.
use super::*;
use crate::{assets::Assets,coop::{arena::Arena,world::Body,
    native_route::{Command,CourseStep,Navigation,Spec}},movement::Controls};
use serde::{Deserialize,Serialize};
#[path="route_input.rs"] mod route_input;

#[derive(Clone,Debug,Serialize,Deserialize)]
pub(crate) enum Step { Ready(usize), Ride{leaf:usize,until:f32,#[serde(default)] spread:Option<[f32;2]>,#[serde(default)] avoid:bool} }
#[derive(Default)]
pub(crate) struct Driver {local:Option<Vec3>,age:u64,aim:Vec2,last_jump:u64}
mod ride;
impl Driver {
    pub(crate) fn input(&mut self,arena:&Arena,world:&World,players:&[Body;2],i:usize,step:&Step)->Result<(Controls,bool)> {
        let pool=arena.stage.interactions.pool.as_ref().context("Missing Pool controller")?;
        let p=&players[i].player;
        match step {
            Step::Ready(leaf)=>{
                ensure!((1..=4).contains(leaf),"Invalid leaf readiness");
                Ok((Controls::default(),pool.ready(leaf-1)))
            }
            Step::Ride{leaf,until,spread,avoid}=>{
                ensure!((1..=4).contains(leaf) && until.is_finite() && *until>=0.,"Invalid leaf ride");
                let index=leaf-1;let name=format!("rideleaf{leaf}obj");
                let object=pool.objects.iter().find(|o|o.name==name).context("Missing ride leaf")?;
                let start=pool.state.rides[index].context("Leaf ride before both boarders")?;
                let clock=pool.state.age-start-pool.state.blocked_time.get(&name).copied().unwrap_or(0.);
                if *avoid {return Ok((self.ride(pool,world,players,i,index,clock),clock>=*until));}
                let local=*self.local.get_or_insert_with(|| {
                    let v=object.pose.1.inverse()*(p.feet-object.pose.0);
                    println!("POOL RIDER {i} leaf{leaf} until{until} center{:?} local{v:?}",object.pose.0);
                    v.truncate().clamp_length_max(60.).extend(0.)
                });
                let offset=spread.map_or(object.pose.1*local,|v|Vec2::from_array(v).extend(0.)*if i==0 {-1.}else{1.});
                let d=object.pose.0+offset-p.feet;
                Ok((Controls{wish:(d.truncate()/60.).clamp_length_max(1.),swim:d.normalize_or_zero(),
                    rise:if p.immersion.level>0 {1.}else{0.},run:true,..Default::default()},clock>=*until))
            }
        }
    }
}
pub(crate) fn check(a:&mut Assets)->crate::levels::BoxFuture<'_> {
    Box::pin(async move {
        let map=Bsp::parse(&a.read("maps/potears1.bsp")?)?;
        let steps:Vec<serde_json::Value>=serde_json::from_str(route_input::STEPS)?;
        let mut commands=vec![Command::Staff(false),Command::Cards(false),Command::Wait(120)];
        let mut bank=Vec::new();
        for s in steps {
            if s.get("save").is_some() {continue;}
            let point=s.get("steer").or_else(||s.get("align"));
            if let Some(v)=point {
                bank.push(CourseStep{at:serde_json::from_value(v.clone())?,mode:if s.get("align").is_some(){"align"}else{"steer"}.into(),ticks:0,clock:None});continue;
            }
            if let Some(v)=s.get("input") {
                let v:[f32;6]=serde_json::from_value(v.clone())?;
                let mode=match (v[4]>0.,v[5]>0.) {(false,false)=>"drive",(true,false)=>"drive_jump",(false,true)=>"drive_use",(true,true)=>"drive_jump_use"};
                bank.push(CourseStep{at:[v[1],v[2],v[3]],mode:mode.into(),ticks:(v[0]*120.) as u64,clock:None});continue;
            }
            if let Some(v)=s.get("jump_if_grounded") {
                let v:[f32;3]=serde_json::from_value(v.clone())?;
                bank.push(CourseStep{at:[v[1],v[2],0.],mode:"drive_ground_jump".into(),ticks:(v[0]*120.)as u64,clock:None});continue;
            }
            if let Some(v)=s.get("board") {
                let leaf=v.as_u64().context("Invalid leaf number")?;
                let name=format!("rideleaf{leaf}obj");
                let entity=map.entities.iter().position(|e|e.get("targetname")==Some(&name)).context("Missing leaf entity")?;
                bank.push(CourseStep{at:[0.,0.,8.],mode:"pond_board".into(),ticks:entity as u64,clock:None});
                commands.push(Command::CourseFile(std::mem::take(&mut bank)));continue;
            }
            if !bank.is_empty() {commands.push(Command::CourseFile(std::mem::take(&mut bank)));}
            let command=if let Some(v)=s.get("ride") {let v:[f32;2]=serde_json::from_value(v.clone())?;
                Command::Pool(Step::Ride{leaf:v[0]as usize,until:v[1],spread:None,avoid:false})}
            else if let Some(v)=s.get("ready") {Command::Pool(Step::Ready(v.as_u64().context("Invalid ready leaf")? as usize))}
            else if let Some(v)=s.get("cards") {Command::Cards(v.as_bool().context("Invalid cards flag")?)}
            else if let Some(v)=s.get("clear") {Command::Clear(v.as_f64().context("Invalid clearing radius")? as f32)}
            else if let Some(v)=s.get("nav") {Command::Nav(serde_json::from_value(v.clone())?)}
            else {anyhow::bail!("Unknown paired Pool input {s}");};
            commands.push(command);
        }
        if !bank.is_empty() {commands.push(Command::CourseFile(bank));}
        let mut paired=Vec::new();
        let mut crossing_third=false;
        for mut command in commands {
            if crossing_third {
                // Both leave the moving leaf together and reach the rope before
                // pressing E. The bank after it keeps independent cursors.
                if let Command::CourseFile(steps)=&mut command {
                    if steps.first().is_some_and(|s|s.at==[-0.1,1.,0.]) {
                        steps[0].at=[0.,1.,0.];steps[0].ticks=36;
                        paired.push(Command::Course(std::mem::take(steps)));
                        crossing_third=false;
                    }
                }
                continue;
            }
            match &mut command {
                Command::Pool(Step::Ride{leaf:2,until,spread,..}) if *until==74.=>*spread=Some([0.,-24.]),
                Command::Pool(Step::Ride{leaf:3,until,avoid,..}) if *until==8.=>{
                    *until=59.85;*avoid=true;crossing_third=true;
                }
                Command::CourseFile(steps) if steps.first().is_some_and(|s|s.at==[-0.95,-0.25,0.])=>{
                    // Red is nearest this bank. A zero-duration marker chooses
                    // the single-file leader; the following jump clears its lip.
                    steps.insert(0,CourseStep{at:[1553.9246,-2140.7822,1340.0313],mode:"wait".into(),ticks:0,clock:None});
                    paired.push(command);
                    paired.push(Command::Course(vec![CourseStep{at:[-0.5,0.,0.],mode:"drive_jump".into(),ticks:48,clock:None}]));
                    continue;
                }
                Command::CourseFile(steps) if steps.first().is_some_and(|s|s.at==[-1178.,-2787.,250.])=>{
                    // Clear the near edge so the next boarder has landing room.
                    steps.push(CourseStep{at:[-1504.,-2656.,336.],mode:"steer".into(),ticks:0,clock:None});
                }
                Command::CourseFile(steps) if steps.first().is_some_and(|s|s.at==[0.,-0.8,0.])=>{
                    command=Command::Course(std::mem::take(steps));
                }
                _=>(),
            }
            paired.push(command);
        }
        ensure!(!crossing_third,"Missing rope course after third leaf");
        crate::coop::native_route::check(a,Spec{map:"potears1",entry:"potears1_start1",
            exit:crate::level::spec::ExitSpec{map:"potears2",entrance:"potears2_start1"},commands:paired,
            navigation:Navigation::LandedPair,verify:|arena,_| {
                let p=arena.stage.interactions.pool.as_ref().context("Missing final Pool")?;
                ensure!(p.state.rides.iter().all(Option::is_some) && p.state.turtle.iter().all(Option::is_some)
                    && p.state.cinema.done.iter().all(|v|*v) && p.state.cinema.exit_sent,"Incomplete paired Pool route");
                let state=serde_json::to_value(arena.stage.interactions.snapshot())?;
                let triggers=state["triggers"].as_array().context("Missing Pool triggers")?;
                for id in [70,667,69,110,128,62,58,109,82,50,56,63,67,116,118] {
                    ensure!(triggers.iter().any(|t|t["id"]==id && t["fired"]==true),"Missing Pool route trigger {id}");
                }
                ensure!(triggers.iter().all(|t|t["reported"]==false),"Unsupported Pool contact");Ok(())
            }}).await
    })
}
