//! Normal paired inputs through the ice course. No player/world staging.
use super::*;
use crate::{coop::{arena::Arena, protocol::{Input, Visit}, world::{Body, Roster}, PlayerId},
    movement::{Controls, FIXED_DT}, route::{fight, Route}};
use std::collections::VecDeque;

fn garden(arena:&Arena)->&Garden {
    arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref()).unwrap()
}
#[derive(Default)]
struct Driver {
    step:usize, ticks:usize, plan_tick:usize, retry:usize, touched:bool,
    goal:Option<Vec3>, radius:f32, plan:VecDeque<(Controls,Vec3)>, fight:fight::Tracker,
}
impl Driver {
    fn reset(&mut self,step:usize) { *self=Self{step,..Default::default()}; }
    fn sample(&mut self,goals:&[[f32;3]],arena:&Arena,world:&World,body:&Body,id:PlayerId,
        waiting:bool)->Result<(Controls,Input)> {
        let p=&body.player;
        let mut input=Input{yaw:body.yaw,pitch:body.pitch,..Default::default()};
        let mut targets=arena.npcs.targets();targets.extend(arena.stage.interactions.levels_targets());
        self.fight.observe(&targets,FIXED_DT);
        let context=crate::combat::Context{world,targets:&targets};
        let attack=fight::plan(&context,p.eye(),&targets,&self.fight,
            body.stats.copies(1)>0 && body.stats.will()>=fight::CARD_COST);
        let locked=arena.characters[id.index()].attack_movement_locked();
        if attack.target.is_some() {
            input.yaw=attack.aim.y.atan2(attack.aim.x);input.pitch=attack.aim.z.asin().clamp(-1.56,1.56);
            if !locked {
                input.weapon=Some(attack.weapon as u8);
                input.primary=!attack.alternate;input.alternate=attack.alternate;
            }
            self.fight.weapon=attack.weapon;self.fight.last_aim=attack.aim;
        }
        if locked || waiting {return Ok((Controls::default(),input));}
        if let Some(hang)=&p.ledge {
            ensure!(self.ticks<120*90,"Ice {id:?} cannot pull up at step{}",self.step);
            self.plan.clear();self.plan_tick=0;self.goal=None;self.retry=0;self.ticks+=1;
            input.primary=false;input.alternate=false;
            return Ok((Controls{wish:hang.forward,jump:!hang.pulling,run:true,..Default::default()},input));
        }
        if garden(arena).saved.complete {
            return Ok((Controls{wish:if garden(arena).ready(){Vec2::X}else{Vec2::ZERO},..Default::default()},input));
        }
        loop {
            let Some(raw)=goals.get(self.step) else {return Ok((Controls::default(),input));};
            let guess=Vec3::from_array(*raw);
            ensure!(self.ticks<120*90,"Ice {id:?} stalled at step{} goal{guess:?}, feet{:?}",self.step,p.feet);
            let vent=matches!(guess.x as i32,-976|-848) && (2700. ..=3250.).contains(&guess.y);
            if vent {
                let delta=guess.truncate()-p.feet.truncate();
                self.touched|=p.steam;
                if self.touched && p.feet.z>=guess.z && delta.length()<80. {
                    println!("PASS ice {id:?} vent step{} at {:?}",self.step,p.feet);
                    self.reset(self.step+1);continue;
                }
                let c=Controls{wish:(delta*3.-p.velocity.truncate()*0.65).clamp_length_max(1.),
                    run:true,jump:self.ticks==0,..Default::default()};
                self.ticks+=1;return Ok((c,input));
            }
            if self.plan_tick==24 {
                let (_,expected)=self.plan.pop_front().context("Ice plan cursor lost its input")?;
                self.plan_tick=0;
                if p.feet.distance(expected)>8. { self.plan.clear();self.goal=None; }
            }
            let goal=if let Some(goal)=self.goal {goal} else {
                let goal=[0.,64.,128.,256.,-64.].into_iter().find_map(|z|world.actor_footing(
                    guess+Vec3::Z*z,crate::collision::PLAYER_CENTER,crate::collision::PLAYER_HALF,768.))
                    .with_context(||format!("Ice {id:?} has no support for step{} {guess:?}",self.step))?;
                self.goal=Some(goal);self.radius=22.;goal
            };
            // Finish the planned landing before starting the next waypoint.
            // Cutting a 24-tick input short changes the next jump's momentum.
            if self.plan.is_empty() && p.grounded && p.feet.distance(goal)<self.radius+2. {
                self.reset(self.step+1);continue;
            }
            if self.plan.is_empty() {
                if !p.grounded || self.retry>0 {
                    self.retry=self.retry.saturating_sub(1);self.ticks+=1;
                    return Ok((Controls{wish:if p.grounded{Vec2::ZERO}else{
                        (goal-p.feet).truncate().normalize_or_zero()},run:true,..Default::default()},input));
                }
                let (plan,radius)=match Route::plan(world,p,goal,22.) {
                    Ok(plan)=>(plan,22.),
                    Err(_)=>match Route::plan(world,p,goal,40.) {
                        Ok(plan)=>(plan,40.),
                        Err(n)=> {
                            // A fallen partner can be reunited by the other's
                            // normal scene arrival. Keep simulating both players;
                            // the per-waypoint deadline still rejects a real stall.
                            println!("Ice {id:?} step{} waits for a route from {:?} ({n} nodes)",self.step,p.feet);
                            self.retry=120;self.ticks+=1;
                            return Ok((Controls::default(),input));
                        },
                    },
                };
                self.plan=plan.into();self.radius=radius;self.plan_tick=0;
                if self.plan.is_empty() {continue;}
            }
            let mut c=self.plan.front().unwrap().0;c.jump&=self.plan_tick==0;
            self.plan_tick+=1;self.ticks+=1;
            if attack.target.is_none() && c.wish.length_squared()>0. {input.yaw=c.wish.y.atan2(c.wish.x);}
            return Ok((c,input));
        }
    }
}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        std::fs::create_dir_all(crate::preferences::path(""))?;
        let visit=Visit{epoch:1,map:"garden4".into(),entry:Some("garden4_start1".into()),
            difficulty:Default::default(),combat_preview:false};
        let map=Bsp::parse(&a.read("maps/garden4.bsp")?)?;
        let mut world=World::from_bsp(&map)?;
        let mut arena=Arena::load(a,&map,&mut world,&visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(&map,visit.entry.as_deref());
        let mut roster=Roster::new_with(&world,eye,yaw,Stats::for_level("garden4",visit.entry.as_deref()),
            |at|!arena.stage.interactions.hazardous(at))?;
        arena.reset_characters(&roster.players);
        let mut replica_world=World::from_bsp(&map)?;
        let mut replica=Arena::load(a,&map,&mut replica_world,&visit,false)?;
        let goals=route::goals()?;
        ensure!(!goals.is_empty() && goals.iter().flatten().all(|v|v.is_finite()),"Invalid ice course");
        std::fs::write(crate::preferences::path("garden4-coop-course.json"),serde_json::to_vec(&goals)?)?;
        let mut drivers:[Driver;2]=Default::default();
        let mut trace=crate::coop::recovery_trace::Trace::new(crate::preferences::path("garden4-recovery.jsonl"))?;
        trace.record(&roster,&world,1,"garden4",arena.stage.busy(),false,|at|!arena.stage.interactions.hazardous(at))?;
        let(mut samples,mut reunions,mut free_ticks)=(0,0,0);
        let mut rejoins=Vec::new();let mut scene_joins=Vec::new();let mut history=VecDeque::new();
        let mut arrival=None;
        let result:Result<()>=async {
            for _ in 0..120*900 {
                let owner=arena.stage.owner;let busy=arena.stage.busy();
                let feet=roster.players.each_ref().map(|b|b.player.feet);
                let warps=roster.players.each_ref().map(|b|b.warped_at);
                let spawns=roster.players.each_ref().map(|b|b.spawned_at);
                let mut inputs=[(Controls::default(),Input::default());2];
                if !busy {
                    for id in PlayerId::ALL {
                        if !roster.players[id.index()].stats.alive(){continue;}
                        let waiting=id==PlayerId::Red && free_ticks<120;
                        inputs[id.index()]=drivers[id.index()].sample(&goals,&arena,&world,&roster.players[id.index()],id,waiting)?;
                    }
                    free_ticks+=1;
                }
                history.push_back(serde_json::json!({"tick":roster.tick,"feet":feet,
                    "steps":drivers.each_ref().map(|d|d.step),"inputs":inputs.each_ref().map(|(c,i)|
                        serde_json::json!({"wish":c.wish,"jump":c.jump,"run":c.run,"keys":i}))}));
                if history.len()>240 {history.pop_front();}
                arena.step(&mut roster,&map,&mut world,inputs)?;
                trace.record(&roster,&world,1,"garden4",busy||arena.stage.busy(),arena.stage.interactive_actor().is_some(),
                    |at|!arena.stage.interactions.hazardous(at))?;
                ensure!(roster.checkpoint_generation==0 && roster.players.iter().all(|b|!b.stats.god&&!b.stats.notarget)
                    && (owner.is_some()||roster.players.each_ref().map(|b|b.warped_at)==warps),
                    "Ice route wiped or warped at tick{}",roster.tick);
                if let Some(id)=arena.stage.owner.filter(|id|Some(*id)==owner) {
                    ensure!(roster.players[id.partner().index()].player.feet==feet[id.partner().index()],"Ice scene moved waiting partner");
                }
                if let Some(id)=owner.filter(|_|arena.stage.owner.is_none()) {
                    reunions+=1;
                    let next=if garden(&arena).saved.complete{goals.len()}else{drivers[id.index()].step+1};
                    scene_joins.push(serde_json::json!({"tick":roster.tick,"owner":id,"from_steps":drivers.each_ref().map(|d|d.step),"to_step":next}));
                    for driver in &mut drivers {driver.reset(next);}
                    ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Ice reunion overlaps");
                }
                for i in 0..2 {
                    if roster.players[i].spawned_at!=spawns[i] {
                        ensure!(roster.players[1-i].stats.alive() && roster.players[i].stats.sanity()==25.,"Invalid ice revival");
                        let next=drivers[1-i].step;
                        rejoins.push(serde_json::json!({"tick":roster.tick,"player":i,"from_step":drivers[i].step,"to_step":next}));
                        drivers[i].reset(next);
                    }else if !roster.players[i].stats.alive(){drivers[i].reset(drivers[i].step);}
                }
                if roster.tick%600==0 {
                    let packed=crate::coop::codec::encode(&serde_json::to_vec(&arena.snapshot())?)?;
                    replica.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&packed)?)?,&map,&mut replica_world)?;
                    ensure!(serde_json::to_value(replica.snapshot())?==serde_json::to_value(arena.snapshot())?,"Ice replica differs");
                    samples+=1;
                    println!("Paired ice tick{} steps{:?} health{:?} feet{:?}",roster.tick,drivers.each_ref().map(|d|d.step),
                        roster.players.each_ref().map(|b|b.stats.sanity()),roster.players.each_ref().map(|b|b.player.feet));
                    clear_background(BLACK);next_frame().await;
                }
                if arena.stage.transition.is_some(){break;}
            }
            let g=garden(&arena);
            ensure!(reunions>=2 && g.saved.complete && g.course().altar.is_some() && g.course().marble_done
                && g.course().wall.is_some() && g.course().fog && g.course().crushed[..2].iter().all(Option::is_some)
                && roster.players.iter().all(|b|b.stats.alive())
                && roster.players.iter().any(|b|b.stats.collected.contains("garden4:42"))
                && arena.stage.transition==Some(EXIT.destination()),"Ice route omitted hazards, wand, scenes or actual exit");
            let next_visit=Visit{epoch:2,map:EXIT.map.into(),entry:Some(EXIT.entrance.into()),..visit};
            let next_map=Bsp::parse(&a.read("maps/centipede1.bsp")?)?;
            let mut next_world=World::from_bsp(&next_map)?;
            let next_arena=Arena::load(a,&next_map,&mut next_world,&next_visit,false)?;
            let (eye,yaw)=crate::interaction::spawn(&next_map,next_visit.entry.as_deref());
            let arrived=roster.arrive_with(&next_world,eye,yaw,EXIT.map,next_visit.entry.as_deref(),|at|!next_arena.stage.interactions.hazardous(at))?;
            ensure!(arrived.players[0].player.feet.distance(arrived.players[1].player.feet)>=36.,"Flora arrival overlaps");
            for (old,new) in roster.players.iter().zip(&arrived.players) {
                ensure!(serde_json::to_value(&old.stats)?==serde_json::to_value(&new.stats)? && next_world.body_clear(new.player.feet),
                    "Flora arrival changed personal resources or embedded a player");
            }
            arrival=Some(serde_json::json!({"visit":next_visit,"frame":arrived.frame(2,0)}));
            Ok(())
        }.await;
        let report=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"full_route_passed":result.is_ok(),
            "error":result.as_ref().err().map(|e|format!("{e:#}")),"frame":roster.frame(1,0),"world":arena.snapshot(),
            "steps":drivers.each_ref().map(|d|d.step),"samples":samples,"reunions":reunions,"revival_rejoins":rejoins,
            "scene_joins":scene_joins,"history":history,"arrival":arrival,
            "personal_wand":roster.players.each_ref().map(|b|b.stats.collected.contains("garden4:42"))});
        std::fs::write(crate::preferences::path("garden4-coop-route.json"),serde_json::to_vec(&report)?)?;
        result?;
        println!("PASS paired Icy Reception into Fungiferous Flora: {} ticks, {samples} replicas",roster.tick);
        Ok(())
    })
}
