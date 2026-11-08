//! Continuous production Arena traversal. Waypoints steer ordinary controls;
//! they never place players, grant resources, kill enemies or advance scripts.
use super::*;
use crate::{coop::{arena::Arena,protocol::{Input,Visit},world::Roster,PlayerId},
    movement::FIXED_DT,render::Scene as RenderScene,route::{fight,Route}};
use std::collections::{BTreeMap,VecDeque};

struct Replay {
    scene:RenderScene,arena:Arena,roster:Roster,guest:Arena,guest_world:World,
    first:PlayerId,skip:bool,index:usize,trackers:[fight::Tracker;2],
    trail:VecDeque<Vec3>,history:VecDeque<serde_json::Value>,
    owners:BTreeMap<String,PlayerId>,reunions:usize,samples:usize,
}
impl Replay {
    fn castle(&self)->&Castle {
        self.arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref()).unwrap()
    }
    fn record(&self,name:&str)->Result<()> {
        let report=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"first":self.first,"skip":self.skip,
            "step":self.index,"owners":self.owners,"reunions":self.reunions,"replicas":self.samples,
            "frame":self.roster.frame(1,0),"world":self.arena.snapshot(),"recent_inputs":self.history,"trail":self.trail});
        std::fs::write(crate::preferences::path(&format!("castling-{:?}-{}-{name}.json",self.first,self.skip)),serde_json::to_vec(&report)?)?;
        Ok(())
    }
    fn observe_scene(&mut self)->Result<()> {
        if let Some(id)=self.arena.stage.owner {
            if let Some(name)=self.castle().scene_id() {
                ensure!(id==self.first,"Castling scene {name} chose {id:?}, expected {:?}",self.first);
                if let Some(old)=self.owners.insert(name.into(),id) {ensure!(old==id,"Castling scene owner changed");}
            }
        }
        Ok(())
    }
    fn inputs(&mut self,movement:[Controls;2])->[(Controls,Input);2] {
        let mut targets=self.arena.npcs.targets();
        targets.extend(self.arena.stage.interactions.levels_targets());
        let foes=targets.iter().filter(|t|self.castle().hostile_target(t.id)).copied().collect::<Vec<_>>();
        std::array::from_fn(|i| {
            let b=&self.roster.players[i];let mut c=movement[i];
            let mut keys=Input{yaw:b.yaw,pitch:b.pitch,..Default::default()};
            if !self.arena.stage.busy() {
                self.trackers[i].observe(&foes,FIXED_DT);
                let ctx=crate::combat::Context{world:&self.scene.world,targets:&targets};
                let mut attack=fight::plan(&ctx,b.player.eye(),&foes,&self.trackers[i],
                    b.stats.copies(1)>0 && b.stats.will()>=fight::CARD_COST);
                if b.stats.copies(4)>0 && b.stats.will()>=0.75 && !b.player.swimming {
                    if let Some((id,point))=foes.iter().filter_map(|t|fight::aim_point(&ctx,b.player.eye(),t).map(|p|(t.id,p)))
                        .filter(|(_,p)|p.distance(b.player.eye())<360.)
                        .min_by(|(_,a),(_,c)|a.distance_squared(b.player.eye()).total_cmp(&c.distance_squared(b.player.eye()))) {
                        attack.weapon=4;attack.alternate=false;attack.target=Some(id);
                        attack.aim=(point-b.player.eye()).normalize_or_zero();
                    }
                }
                if attack.target.is_some() {
                    keys.yaw=attack.aim.y.atan2(attack.aim.x);keys.pitch=attack.aim.z.asin().clamp(-1.56,1.56);
                    keys.weapon=Some(attack.weapon as u8);keys.primary=!attack.alternate;keys.alternate=attack.alternate;
                    self.trackers[i].weapon=attack.weapon;self.trackers[i].last_aim=attack.aim;
                } else if c.wish.length_squared()>0. {
                    keys.yaw=c.wish.y.atan2(c.wish.x);keys.pitch=0.;
                }
                let aim=c.wish.extend(0.).normalize_or_zero();
                if self.arena.stage.interactions.closed_door_in_reach(&self.scene.world,b.player.eye(),aim).is_some()
                    || self.castle().use_door(&self.scene.world,b.player.eye(),aim).is_some() {
                    c.use_pressed=true;keys.yaw=aim.y.atan2(aim.x);keys.pitch=0.;keys.primary=false;keys.alternate=false;
                }
            }
            (c,keys)
        })
    }
    async fn step(&mut self,movement:[Controls;2])->Result<()> {
        let before=self.roster.players.each_ref().map(|b|b.player.feet);
        let warps=self.roster.players.each_ref().map(|b|b.warped_at);
        let old_owner=self.arena.stage.owner;
        self.observe_scene()?;
        let inputs=self.inputs(movement);
        if self.skip && self.arena.stage.busy() {
            let i=old_owner.context("Missing Castling scene owner")?.index();
            self.arena.stage.interactions.skip_cinematic(&self.scene.map,&mut self.scene.world,
                &mut self.roster.players[i].player,&mut self.arena.stage.story)?;
        }
        self.history.push_back(serde_json::json!({"tick":self.roster.tick,"step":self.index,"feet":before,
            "inputs":inputs.each_ref().map(|(c,i)|serde_json::json!({"wish":c.wish,"jump":c.jump,"run":c.run,"use":c.use_pressed,"keys":i}))}));
        if self.history.len()>240 {self.history.pop_front();}
        self.arena.step(&mut self.roster,&self.scene.map,&mut self.scene.world,inputs)?;
        if let Some(id)=self.arena.stage.owner.filter(|id|Some(*id)==old_owner) {
            ensure!(self.roster.players[id.partner().index()].player.feet==before[id.partner().index()],"Waiting Castling partner moved");
        }
        self.observe_scene()?;
        if old_owner.is_some() && self.arena.stage.owner.is_none() {
            self.reunions+=1;self.trail.clear();
            ensure!(self.roster.players[0].player.feet.distance(self.roster.players[1].player.feet)<=120.,"Castling reunion left partner behind");
        }
        ensure!(self.roster.checkpoint_generation==0 && self.roster.players.iter().all(|b|
            b.stats.alive() && b.spawned_at==0 && !b.stats.god && !b.stats.notarget),
            "Castling death/wipe step{} tick{} health{:?}",self.index,self.roster.tick,self.roster.players.each_ref().map(|b|b.stats.sanity()));
        ensure!(old_owner.is_some() || self.roster.players.each_ref().map(|b|b.warped_at)==warps,"Unscripted Castling warp");
        if !self.arena.stage.busy() {
            let d=self.roster.players[0].player.feet-self.roster.players[1].player.feet;
            ensure!(d.truncate().length()>=crate::coop::separation::DISTANCE-0.05 || d.z.abs()>=PLAYER_HALF.z*2.-0.05,
                "Castling players overlap step{} tick{}: {d:?}",self.index,self.roster.tick);
            let leader=self.roster.players[self.first.index()].player.feet;
            if self.trail.back().is_none_or(|p|p.distance(leader)>12.) {self.trail.push_back(leader);}
            ensure!(self.trail.len()<2000,"Castling follower lost the path");
        }
        if self.roster.tick%600==0 {
            let snapshot=self.arena.snapshot();
            let bytes=crate::coop::codec::encode(&serde_json::to_vec(&snapshot)?)?;
            self.guest.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,&self.scene.map,&mut self.guest_world)?;
            ensure!(serde_json::to_value(self.guest.snapshot())?==serde_json::to_value(snapshot)?,"Castling replica differs");
            self.samples+=1;
            println!("Castling {:?} skip={} tick{} step{} health{:?} feet{:?}",self.first,self.skip,self.roster.tick,self.index,
                self.roster.players.each_ref().map(|b|b.stats.sanity()),self.roster.players.each_ref().map(|b|b.player.feet));
            clear_background(BLACK);next_frame().await;
        }
        Ok(())
    }
    async fn scene_end(&mut self)->Result<()> {
        for _ in 0..24000 {
            // A pending map load also marks Stage busy, but has no scene to
            // advance. The route's arrival check owns that next step.
            if self.arena.stage.transition.is_some() || !self.arena.stage.busy() {return Ok(());}
            self.step(Default::default()).await?;
        }
        anyhow::bail!("Castling scene stalled")
    }
    async fn go(&mut self,goal:Vec3,portal:bool)->Result<()> {
        let mut last=self.roster.players.each_ref().map(|b|b.player.feet);let mut stuck=[0;2];
        let mut plans:[VecDeque<(Controls,Vec3)>;2]=Default::default();let mut ages=[0;2];
        for t in 0..120*45 {
            if self.arena.stage.transition.is_some(){return Ok(());}
            if self.arena.stage.busy(){self.scene_end().await?;return Ok(());}
            let lead=self.roster.players[self.first.index()].player.feet;
            let goals=if portal {
                let offset=self.castle().portal_pose().translation-self.castle().data.point("exit_portal").translation;
                std::array::from_fn(|i|vec3(-1024.,if i==self.first.index(){-100.}else{-28.},512.03125)+offset)
            }else{[goal;2]};
            let ready=std::array::from_fn::<_,2,_>(|i| {
                let p=&self.roster.players[i].player;
                // The follower's stopping band is relative to the leader, who
                // may stop up to 22 units short of a navigation waypoint.
                // Final portal positions still require each distinct goal.
                let d=if !portal && i!=self.first.index(){lead-p.feet}else{goals[i]-p.feet};
                d.truncate().length()<if portal{12.}else if i==self.first.index(){22.}else{80.}
                    && d.z.abs()<if portal{24.}else{60.} && p.grounded
            });
            if ready.iter().all(|v|*v) {return Ok(());}
            let mut controls=[Controls::default();2];
            for id in [self.first,self.first.partner()] {
                let i=id.index();let p=&self.roster.players[i].player;let gap=p.feet-lead;
                if id!=self.first && !portal {
                    while self.trail.front().is_some_and(|at|{let d=*at-p.feet;d.truncate().length()<8. && d.z.abs()<45.}) {self.trail.pop_front();}
                    if gap.truncate().length()<72. && gap.z.abs()<60. {
                        let forward=(goal-lead).truncate().normalize_or_zero();
                        // A reunion can put the partner in front of the actor.
                        // Yield through a checked, supported walking segment.
                        let carrying=gap.z < -(PLAYER_HALF.z*2.-0.1)
                            && gap.truncate().length()<crate::coop::separation::DISTANCE;
                        if !ready[self.first.index()] && (gap.truncate().dot(forward)>-4. || carrying) {
                            let side=vec2(-forward.y,forward.x);
                            for wish in [side,-side,forward,-forward] {
                                if gap.truncate().dot(wish)<0. {continue;}
                                let at=p.feet+wish.extend(0.)*12.;
                                let path=self.scene.world.body_trace(p.feet,at);
                                let floor=self.scene.world.body_trace(at,at-Vec3::Z*32.);
                                if !path.start_solid && path.fraction==1. && !floor.start_solid && floor.fraction<1.
                                    && floor.normal.z>0.65 && !self.arena.stage.interactions.hazardous(at) {
                                    controls[i].wish=wish;self.trail.clear();break;
                                }
                            }
                        }
                        continue;
                    }
                }
                if ready[i] {plans[i].clear();ages[i]=0;continue;}
                let target=if id!=self.first && !portal {self.trail.front().copied().unwrap_or(goals[i])}else{goals[i]};
                let d=target-p.feet;
                if t%120==119 {
                    stuck[i]=if p.feet.distance(last[i])<8.{stuck[i]+1}else{0};last[i]=p.feet;
                    if stuck[i]>=2 && plans[i].is_empty() {
                        // The original course includes goals beside door jambs.
                        // Find ordinary movement around the current geometry;
                        // replay every step through the real paired controller.
                        // Future proximity doors are closed in this snapshot;
                        // search only around the current obstruction, then let
                        // normal approach open later doors before planning again.
                        let nearby=p.feet+(target-p.feet).clamp_length_max(120.);
                        println!("Castling local path step{} {id:?} {:?} -> {nearby:?}",self.index,p.feet);
                        plans[i]=Route::plan(&self.scene.world,p,nearby,22.)
                            .map_err(|n|anyhow::anyhow!("Castling path step{} {id:?} exhausted {n} states",self.index))?.into();
                        ages[i]=0;
                    }
                }
                if let Some((input,_))=plans[i].front() {
                    controls[i]=*input;controls[i].jump &= ages[i]==0;ages[i]+=1;
                    if ages[i]>=24 {plans[i].pop_front();ages[i]=0;}
                } else {
                    let ahead=p.feet+d.truncate().normalize_or_zero().extend(0.)*40.;
                    let support=self.scene.world.body_trace(ahead,ahead-Vec3::Z*32.);
                    // The single-player exit can fire while falling through
                    // its volume. A waiting co-op rider must actually reach the
                    // deck, so jump before leaving the approach ledge.
                    let board_jump=portal && p.grounded && !self.castle().portal_support(p)
                        && d.truncate().length()<330.
                        && (support.start_solid || support.fraction==1. || support.normal.z<0.65);
                    controls[i]=Controls{wish:d.truncate().normalize_or_zero()*(d.truncate().length()/48.).min(1.),run:true,
                        jump:board_jump || p.ledge.as_ref().is_some_and(|h|!h.pulling),..Default::default()};
                }
            }
            self.step(controls).await?;
        }
        anyhow::bail!("Castling walk step{} stalled toward {goal:?}, portal={portal}",self.index)
    }
}
async fn route(a:&mut Assets,first:PlayerId,skip:bool)->Result<()> {
    let visit=Visit{epoch:1,map:"wchess2".into(),entry:Some("wchess2_start1".into()),difficulty:Default::default(),combat_preview:false};
    let mut scene=RenderScene::load_for(a,&visit.map,visit.difficulty)?;
    let mut arena=Arena::load(a,&scene.map,&mut scene.world,&visit,false)?;
    let (eye,yaw)=crate::interaction::spawn(&scene.map,visit.entry.as_deref());
    let roster=Roster::new_with(&scene.world,eye,yaw,Stats::for_level(&visit.map,visit.entry.as_deref()),|at|!arena.stage.interactions.hazardous(at))?;
    arena.reset_characters(&roster.players);
    let mut guest_world=World::from_bsp(&scene.map)?;
    let guest=Arena::load(a,&scene.map,&mut guest_world,&visit,false)?;
    let mut r=Replay{scene,arena,roster,guest,guest_world,first,skip,index:0,trackers:Default::default(),
        trail:Default::default(),history:Default::default(),owners:Default::default(),reunions:0,samples:0};
    let result:Result<()> = async {
        for (i,goal) in check::COURSE.iter().enumerate() {
            r.index=i;r.record(&format!("before-{i}"))?;
            r.go(Vec3::from_array(*goal),i==check::COURSE.len()-1).await?;
            r.scene_end().await?;
            println!("PASS Castling leg{i} {:?} skip={skip}",first);
            if r.arena.stage.transition.is_some(){break;}
        }
        for _ in 0..600 {if r.arena.stage.transition.is_some(){break;}r.step(Default::default()).await?;}
        ensure!(r.arena.stage.transition==Some(EXIT.destination()) && r.castle().saved.king && r.castle().saved.queen,
            "Castling route did not finish original scenes and portal");
        ensure!(r.owners.len()==2 && r.reunions==2,"Castling route omitted scene ownership/reunion");
        let next_visit=Visit{epoch:2,map:"rchess1".into(),entry:Some("rchess1_start1".into()),..visit};
        let mut next=RenderScene::load_for(a,&next_visit.map,next_visit.difficulty)?;
        let next_arena=Arena::load(a,&next.map,&mut next.world,&next_visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(&next.map,next_visit.entry.as_deref());
        let arrived=r.roster.arrive_with(&next.world,eye,yaw,&next_visit.map,next_visit.entry.as_deref(),|at|!next_arena.stage.interactions.hazardous(at))?;
        for (old,new) in r.roster.players.iter().zip(&arrived.players) {
            ensure!(serde_json::to_value(&old.stats)?==serde_json::to_value(&new.stats)? && next.world.body_clear(new.player.feet),"Checkmate arrival changed resources or embedded player");
        }
        let gap=arrived.players[0].player.feet-arrived.players[1].player.feet;
        ensure!(gap.truncate().length()>=crate::coop::separation::DISTANCE || gap.z.abs()>=PLAYER_HALF.z*2.,"Checkmate arrival overlap");
        Ok(())
    }.await;
    r.record(if result.is_ok(){"passed"}else{"failed"})?;
    result?;
    println!("PASS Castling full paired route {first:?} skip={skip}: {}ticks, {} exact replicas, {} reunions",r.roster.tick,r.samples,r.reunions);
    Ok(())
}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        std::fs::create_dir_all(crate::preferences::path(""))?;
        let first=std::env::var("LOOKING_GLASS_CASTLING_COOP_FIRST").ok();
        let scenes=std::env::var("LOOKING_GLASS_CASTLING_COOP_SCENES").ok();
        ensure!(first.as_deref().is_none_or(|s|matches!(s,"Blue"|"Red"))
            && scenes.as_deref().is_none_or(|s|matches!(s,"watched"|"skipped")),"Invalid Castling route selector");
        let mut failures=Vec::new();
        for actor in PlayerId::ALL {for skip in [false,true] {
            if first.as_ref().is_some_and(|s|*s!=format!("{actor:?}")) || scenes.as_deref().is_some_and(|s|(s=="skipped")!=skip){continue;}
            if let Err(e)=route(a,actor,skip).await {println!("FAIL Castling {actor:?} skip={skip}: {e:#}");failures.push(format!("{actor:?}/{skip}: {e:#}"));}
        }}
        ensure!(failures.is_empty(),"Castling routes failed: {}",failures.join("; "));Ok(())
    })
}
