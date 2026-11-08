//! Continuous native Arena route using only ordinary inputs and scene skips.
//! Course positions are navigation goals, never writes to gameplay state.
use super::*;
use crate::{coop::{arena::Arena, protocol::{Input,Visit},world::Roster,PlayerId},
    movement::FIXED_DT,render::Scene as RenderScene,route::{fight,Route}};
use std::collections::{BTreeMap,VecDeque};
use sha2::{Digest,Sha256};

#[derive(Clone,Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct Step { pub mode:String, #[serde(default)] pub pos:[f32;3], #[serde(default)] pub value:f32,
    #[serde(default)] pub name:String, #[serde(default="arrival_radius")] pub radius:f32,
    #[serde(default)] pub hold:[f32;3] }
fn arrival_radius()->f32 {24.}
struct Replay {
    arena:Arena,scene:RenderScene,roster:Roster,guest:Arena,guest_world:World,
    first:PlayerId,skip:bool,index:usize,trackers:[fight::Tracker;2],samples:usize,
    owners:BTreeMap<String,PlayerId>,history:VecDeque<serde_json::Value>,trail:VecDeque<(Vec3,bool)>,
    course_sha256:String,default_course:bool,goal:Option<Vec3>,
}
impl Replay {
    fn realm(&self)->&Realm {self.arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref()).unwrap()}
    fn observe_scene(&mut self)->Result<()> {
        if let Some(id)=self.arena.stage.owner {
            if let Some(name)=self.realm().scene_id() {
                if name!="cinema_wchess1_intro_thread" {ensure!(id==self.first,"Wrong first chess actor {name}: {id:?}");}
                if let Some(previous)=self.owners.insert(name.into(),id) {ensure!(previous==id,"Chess scene changed actor");}
            }
        }Ok(())
    }
    fn inputs(&mut self,controls:[Controls;2],fight:bool)->[(Controls,Input);2] {
        let mut targets=self.arena.npcs.targets();targets.extend(self.arena.stage.interactions.levels_targets());
        let foes=targets.iter().filter(|t|self.realm().hostile_target(t.id)).copied().collect::<Vec<_>>();
        std::array::from_fn(|i| {
            let b=&self.roster.players[i];let mut keys=Input{yaw:b.yaw,pitch:b.pitch,..Default::default()};
            let mut control=controls[i];
            if fight && !self.arena.stage.busy() && self.arena.stage.interactive_actor().is_none() {
                self.trackers[i].observe(&foes,FIXED_DT);
                let ctx=crate::combat::Context{world:&self.scene.world,targets:&targets};
                let mut attack=fight::plan(&ctx,b.player.eye(),&foes,&self.trackers[i],
                    b.stats.copies(1)>0 && b.stats.will()>=fight::CARD_COST);
                if b.stats.copies(4)>0 && b.stats.will()>=0.75 && b.player.immersion.level==0 {
                    if let Some(point)=foes.iter().filter_map(|t|fight::aim_point(&ctx,b.player.eye(),t))
                        .filter(|p|p.distance(b.player.eye())<360.)
                        .min_by(|a,c|a.distance_squared(b.player.eye()).total_cmp(&c.distance_squared(b.player.eye()))) {
                        attack.weapon=4;attack.alternate=false;attack.aim=(point-b.player.eye()).normalize_or_zero();
                        attack.target=Some(0);
                    }
                }
                if attack.target.is_some() {
                    keys.yaw=attack.aim.y.atan2(attack.aim.x);keys.pitch=attack.aim.z.asin().clamp(-1.56,1.56);
                    keys.weapon=Some(attack.weapon as u8);keys.primary=!attack.alternate;keys.alternate=attack.alternate;
                    self.trackers[i].weapon=attack.weapon;self.trackers[i].last_aim=attack.aim;
                }
            }
            // Use the real reach/sight prompt and ordinary E input, as the
            // single-player driver does. Face the door for that input; weapon
            // aiming must not turn a valid use into a press in another direction.
            let aim=control.wish.extend(0.).normalize_or_zero();
            if !self.arena.stage.busy() && self.arena.stage.interactive_actor().is_none()
                && self.arena.stage.interactions.closed_door_in_reach(&self.scene.world,b.player.eye(),aim).is_some() {
                control.use_pressed=true;keys.yaw=aim.y.atan2(aim.x);keys.pitch=0.;
                keys.primary=false;keys.alternate=false;
            }
            (control,keys)
        })
    }
    async fn tick(&mut self,inputs:[(Controls,Input);2])->Result<()> {
        let before=self.roster.players.each_ref().map(|b|b.player.feet);
        let old_owner=self.arena.stage.owner;let old_focus=self.arena.stage.interactive_actor();
        let old_warps=self.roster.players.each_ref().map(|b|b.warped_at);
        // The initial pending scene can be skipped before the first Arena step.
        // Observe its real owner before that input, as well as new scenes afterward.
        self.observe_scene()?;
        if self.skip && self.arena.stage.busy() {
            let i=self.arena.stage.owner.context("Missing chess scene actor")?.index();
            self.arena.stage.interactions.skip_cinematic(&self.scene.map,&mut self.scene.world,
                &mut self.roster.players[i].player,&mut self.arena.stage.story)?;
        }
        self.history.push_back(serde_json::json!({"tick":self.roster.tick,"step":self.index,"feet":before,
            "inputs":inputs.each_ref().map(|(c,i)|serde_json::json!({"wish":c.wish,"swim":c.swim,
                "jump":c.jump,"rise":c.rise,"use":c.use_pressed,"run":c.run,"keys":i}))}));
        if self.history.len()>240 {self.history.pop_front();}
        self.arena.step(&mut self.roster,&self.scene.map,&mut self.scene.world,inputs)?;
        if let Some(id)=self.arena.stage.owner {
            if old_owner==Some(id) {ensure!(self.roster.players[id.partner().index()].player.feet==before[id.partner().index()],"Waiting chess partner moved");}
        }
        self.observe_scene()?;
        ensure!(self.roster.checkpoint_generation==0 && self.roster.players.iter().all(|b|
            b.stats.alive() && b.spawned_at==0 && !b.stats.god && !b.stats.notarget),
            "Chess route death/wipe step{} tick{}: {:?}",self.index,self.roster.tick,
            self.roster.players.each_ref().map(|b|(b.player.feet,b.stats.sanity())));
        ensure!(old_owner.is_some() || old_focus.is_some()
            || self.roster.players.each_ref().map(|b|b.warped_at)==old_warps,"Unscripted chess route warp");
        if !self.arena.stage.busy() && self.arena.stage.interactive_actor().is_none() {
            if old_owner.is_some() || old_focus.is_some() {self.trail.clear();}
            let leader=&self.roster.players[self.first.index()].player;
            let at=leader.feet;
            if self.trail.back().is_none_or(|(p,_)|p.distance(at)>12.) {
                self.trail.push_back((at,!leader.grounded && !leader.swimming));
            }
            ensure!(self.trail.len()<2000,"Chess follower lost the leader's path");
            let d=self.roster.players[0].player.feet-self.roster.players[1].player.feet;
            ensure!(d.truncate().length()>=crate::coop::separation::DISTANCE-0.05 || d.z.abs()>=PLAYER_HALF.z*2.-0.05,"Chess bodies overlapped");
        }
        if self.roster.tick%600==0 {
            let bytes=crate::coop::codec::encode(&serde_json::to_vec(&self.arena.snapshot())?)?;
            self.guest.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,&self.scene.map,&mut self.guest_world)?;
            ensure!(serde_json::to_value(self.guest.snapshot())?==serde_json::to_value(self.arena.snapshot())?,"Chess route replica differs");
            self.samples+=1;
            println!("Chess {:?} skip={} t={} step{} health{:?} feet{:?}",self.first,self.skip,self.roster.tick/120,self.index,
                self.roster.players.each_ref().map(|b|b.stats.sanity()),self.roster.players.each_ref().map(|b|b.player.feet));
            clear_background(BLACK);next_frame().await;
        }
        Ok(())
    }
    async fn step(&mut self,controls:[Controls;2],fight:bool)->Result<()> {
        let inputs=self.inputs(controls,fight);self.tick(inputs).await
    }
    async fn scene_end(&mut self)->Result<()> {
        for _ in 0..24000 {
            if !self.arena.stage.busy(){return Ok(());}
            self.step(Default::default(),false).await?;
        }anyhow::bail!("Chess cinematic handoff stalled")
    }
    async fn wait(&mut self,seconds:f32)->Result<()> {
        for _ in 0..(seconds/FIXED_DT).ceil() as usize {
            if self.arena.stage.transition.is_some(){break;}
            self.step(Default::default(),true).await?;
        }self.scene_end().await
    }
    async fn spikes(&mut self,s:&Step)->Result<()> {
        let mut crossing:[spike_route::Crossing;2]=Default::default();
        self.goal=Some(Vec3::from_array(s.pos));
        for _ in 0..120*60 {
            let planned=std::array::from_fn::<_,2,_>(|i|crossing[i].controls(self.realm(),&self.scene.world,
                &self.roster.players[i].player,&self.roster.players[self.first.index()].player,
                i==self.first.index(),Vec3::from_array(s.hold),Vec3::from_array(s.pos),0.));
            if planned.iter().all(|(_,ready)|*ready) {self.trail.clear();return Ok(());}
            self.step(planned.map(|(c,_)|c),false).await?;
        }
        anyhow::bail!("Timed corridor crossing stalled")
    }
    async fn go(&mut self,goals:[Vec3;2],precise:bool,radius:f32)->Result<()> {
        self.goal=Some(goals[self.first.index()]);
        let mut plans:[VecDeque<(Controls,Vec3)>;2]=Default::default();let mut ages=[0;2];
        let mut last=self.roster.players.each_ref().map(|b|b.player.feet);let mut stuck=[0;2];
        let mut clearing_landing=false;
        for t in 0..120*90 {
            // The precise pair of goals belongs to the lift deck, whose height
            // can change as soon as both players have stepped aboard.
            let goals=if precise {goals.map(|mut p|{p.z=256.+self.realm().saved.elevator.height;p})}else{goals};
            if self.arena.stage.transition.is_some(){return Ok(());}
            if self.arena.stage.busy(){self.scene_end().await?;return Ok(());}
            if self.arena.stage.interactive_actor().is_some(){return Ok(());}
            let ready=std::array::from_fn::<_,2,_>(|i|{
                let p=&self.roster.players[i].player;let d=goals[i]-p.feet;
                // A follower stopped safely on the leader must let the leader
                // continue down the stairs; waiting for both to touch that
                // same patch of floor would hold them at contact forever.
                let gap=p.feet-self.roster.players[self.first.index()].player.feet;
                let on_leader=!precise && i!=self.first.index() && p.velocity.z.abs()<0.1
                    && (gap.z-PLAYER_HALF.z*2.).abs()<0.05
                    && gap.truncate().length()<crate::coop::separation::DISTANCE;
                d.truncate().length()<if precise {12.}else if i==self.first.index(){radius}else{80.}
                    && d.z.abs()<if precise{24.}else if on_leader{80.}else{45.}
                    && (p.grounded || p.swimming || on_leader)
            });
            if ready.iter().all(|v|*v){if precise{self.trail.clear();}return Ok(());}
            clearing_landing=!precise && self.roster.players[self.first.partner().index()].player.climbing()
                && (clearing_landing || ready[self.first.index()] && self.roster.players[self.first.index()].player.grounded);
            let mut c=[Controls::default();2];
            for id in [self.first,self.first.partner()] {
                let i=id.index();let p=&self.roster.players[i].player;
                if id==self.first && clearing_landing {
                    // Once the leader has reached a bank, give the follower's
                    // checked pull-up room to finish. Walk clear, then wait;
                    // both original arrival checks still have to pass afterward.
                    let gap=p.feet-self.roster.players[id.partner().index()].player.feet;
                    if gap.truncate().length()<80. {
                        let away=gap.truncate().try_normalize().unwrap_or(Vec2::X);
                        let side=vec2(-away.y,away.x);
                        for wish in [away,side,-side] {
                            let at=p.feet+wish.extend(0.)*12.;
                            let path=self.scene.world.body_trace(p.feet,at);
                            let floor=self.scene.world.body_trace(at,at-Vec3::Z*32.);
                            if !path.start_solid && path.fraction==1. && !floor.start_solid && floor.fraction<1.
                                && floor.normal.z>0.65 && !self.arena.stage.interactions.hazardous(at) {
                                c[i].wish=wish;break;
                            }
                        }
                    }
                    continue;
                }
                if id!=self.first && !precise {
                    while self.trail.front().is_some_and(|(at,airborne)| {
                        let d=*at-p.feet;
                        // After splashing down, follow the leader's next water
                        // or landing position, not its jump arc above the pool.
                        // Grounded shore samples and the final depth remain required.
                        p.swimming && *airborne && d.z>45.
                            || d.truncate().length()<8. && d.z.abs()<45.
                    }) {self.trail.pop_front();}
                }
                let goal=if id!=self.first && !precise {self.trail.front().map(|(at,_)|*at).unwrap_or(goals[i])}else{goals[i]};
                let d=goal-p.feet;
                if self.arena.characters[i].attack_movement_locked(){continue;}
                let gap=p.feet-self.roster.players[self.first.index()].player.feet;
                if !precise && id!=self.first && !ready[self.first.index()] && gap.truncate().length()<80. && gap.z.abs()<70. {
                    // A scene reunion can put the follower ahead of the chosen
                    // leader. Yield sideways with ordinary input so neither
                    // player pins the other against the personal-space boundary.
                    let forward=(goals[self.first.index()]-self.roster.players[self.first.index()].player.feet).truncate().try_normalize().unwrap_or(Vec2::X);
                    // Step aside when the leader has landed on this follower,
                    // even if the follower is behind. Otherwise a low doorway
                    // can hold the upper body while the lower one waits forever.
                    let carries_leader=gap.z < -(PLAYER_HALF.z*2.-0.1)
                        && gap.truncate().length()<crate::coop::separation::DISTANCE;
                    if gap.truncate().dot(forward)>-4. || carries_leader {
                        let side=vec2(-forward.y,forward.x);
                        // Reversing out of a narrow supply passage can leave
                        // the follower standing on the leader's next goal.
                        // Even an arrived follower must yield; if neither side
                        // fits, try ahead or backward along the clear corridor.
                        // Every candidate must move away from the leader.
                        for wish in [side,-side,forward,-forward] {
                            // Check a short walking segment each tick. A full
                            // body-width lookahead wrongly rejects room to
                            // yield beside the open supply door's hinge.
                            let at=p.feet+wish.extend(0.)*12.;
                            // Pick the side away from the leader. A clear wall
                            // trace alone can still steer into the other Alice.
                            if gap.truncate().dot(wish)<0. {continue;}
                            let path=self.scene.world.body_trace(p.feet,at);
                            let floor=self.scene.world.body_trace(at,at-Vec3::Z*32.);
                            if !path.start_solid && path.fraction==1. && !floor.start_solid && floor.fraction<1.
                                && floor.normal.z>0.65 && !self.arena.stage.interactions.hazardous(at) {
                                c[i].wish=wish;self.trail.clear();break;
                            }
                        }
                    }
                    continue;
                }
                if ready[i] {continue;}
                if t%120==119 {
                    stuck[i]=if p.feet.distance(last[i])<16.{stuck[i]+1}else{0};last[i]=p.feet;
                    // Try an ordinary jump over a low obstruction before
                    // planning a detour. The final water entrance has a lip;
                    // the single-player route also hops here before swimming.
                    if (2..4).contains(&stuck[i]) && p.grounded && plans[i].is_empty() {
                        c[i]=Controls{wish:d.truncate().normalize_or_zero(),swim:d.normalize_or_zero(),
                            jump:true,run:true,..Default::default()};
                        continue;
                    }
                    if stuck[i]>=4 && plans[i].is_empty() {
                        println!("Chess search step{} {id:?} {:?} -> {goal:?}",self.index,p.feet);
                        plans[i]=Route::plan(&self.scene.world,p,goal,22.)
                            .map_err(|n|anyhow::anyhow!("Chess path step{} {id:?} exhausted {n} states",self.index))?.into();ages[i]=0;
                    }
                }
                if let Some((input,_))=plans[i].front() {
                    c[i]=*input;c[i].jump &= ages[i]==0;ages[i]+=1;
                    if ages[i]>=24 {plans[i].pop_front();ages[i]=0;}
                } else {
                    c[i]=Controls{wish:d.truncate().normalize_or_zero()*(d.truncate().length()/48.).min(1.),
                        swim:d.normalize_or_zero(),rise:if p.swimming && d.z>30.{1.}else{0.},run:true,
                        jump:p.ledge.as_ref().is_some_and(|h|!h.pulling) || p.grounded && d.z>24. && d.truncate().length()<330.,
                        ..Default::default()};
                }
            }
            self.step(c,true).await?;
        }anyhow::bail!("Chess walk step{} stalled goals{goals:?}",self.index)
    }
    async fn board(&mut self,piece:Piece)->Result<()> {
        for _ in 0..3600 {
            if self.realm().saved.board.is_some(){break;}
            self.step(Default::default(),true).await?;
        }
        self.scene_end().await?;
        ensure!(self.arena.stage.interactive_actor()==Some(self.first),"Missing owned chess puzzle");
        // The original scene ends with a controlled run onto square zero;
        // wait for that leg before counting the first player-chosen move.
        for _ in 0..1200 {if !self.realm().controlled(){break;}self.step(Default::default(),false).await?;}
        ensure!(!self.realm().controlled(),"Chess disguise never reached its starting square");
        let dirs=if piece==Piece::Bishop{check::BISHOP}else{check::KNIGHT};
        for &d in dirs {
            let node=self.realm().saved.board.as_ref().context("Missing chess disguise")?.node;
            let mut moved=false;
            for _ in 0..1200 {
                let mut c=[Controls::default();2];
                c[self.first.index()]=Controls{wish:if moved{Vec2::ZERO}else{check::wish(piece,d)},run:true,..Default::default()};
                self.step(c,false).await?;
                let Some(b)=&self.realm().saved.board else {break;};
                moved|=b.moving.is_some();
                if moved && b.moving.is_none() && b.node!=node{break;}
            }
            ensure!(self.realm().saved.board.as_ref().is_none_or(|b|b.node!=node && b.moving.is_none()),"Chess board move stalled");
        }
        for _ in 0..240 {if !self.arena.stage.busy() && self.arena.stage.interactive_actor().is_none(){break;}self.step(Default::default(),false).await?;}
        ensure!(self.realm().saved.board.is_none() && self.arena.stage.interactive_actor().is_none(),"Chess puzzle did not release both players");
        self.record("checkpoint")?;Ok(())
    }
    async fn lift(&mut self)->Result<()> {
        self.go([vec3(-720.,-544.,64.);2],false,7.).await?;
        for _ in 0..1200 {if self.realm().saved.elevator.phase==1{break;}self.step(Default::default(),true).await?;}
        ensure!(self.realm().saved.elevator.phase==1,"Chess lift never lowered");
        let mut goals=[vec3(-752.,-384.,64.);2];goals[self.first.index()]=vec3(-800.,-384.,64.);
        self.go(goals,true,12.).await?;
        for _ in 0..1200 {
            if self.roster.players.iter().all(|b|b.player.feet.z>250. && b.player.grounded){break;}
            self.step(Default::default(),true).await?;
        }
        ensure!(self.roster.players.iter().all(|b|b.player.feet.z>250.),"Chess lift did not carry both riders");
        self.trail.clear();
        self.go([vec3(-800.,-256.,256.);2],false,7.).await
    }
    async fn lever(&mut self,name:&str)->Result<()> {
        let at=self.realm().data.point(name).point(vec3(36.,0.,26.));let i=self.first.index();
        let aim=(at-self.roster.players[i].player.eye()).normalize_or_zero();
        ensure!(self.realm().lever(&self.scene.world,self.roster.players[i].player.eye(),aim).is_some(),"Chess lever {name} out of reach");
        let mut input=self.inputs(Default::default(),false);input[i].0.use_pressed=true;
        input[i].1.yaw=aim.y.atan2(aim.x);input[i].1.pitch=aim.z.asin();self.tick(input).await?;
        self.wait(self.realm().data.lever_duration+if name=="bell_lever"{5.2}else{0.2}).await?;
        self.scene_end().await?;self.record("checkpoint")
    }
    async fn clear(&mut self,radius:f32,rush:bool)->Result<()> {
        let home=self.roster.players.each_ref().map(|b|b.player.feet);let mut quiet=0;
        for _ in 0..10800 {
            let mut c=[Controls::default();2];let mut seen=false;
            let targets=self.realm().targets();let ctx=crate::combat::Context{world:&self.scene.world,targets:&targets};
            for i in 0..2 {
                let p=&self.roster.players[i].player;
                if let Some(t)=targets.iter().filter(|t|self.realm().hostile_target(t.id) && t.center.distance(home[i]+PLAYER_CENTER)<radius
                    && fight::aim_point(&ctx,p.eye(),t).is_some())
                    .min_by(|a,b|a.center.distance_squared(p.eye()).total_cmp(&b.center.distance_squared(p.eye()))) {
                    seen=true;let d=(t.center-p.eye()).truncate();
                    if rush && d.length()>48. {c[i].wish=d.normalize_or_zero();c[i].run=true;}
                }
            }
            quiet=if seen{0}else{quiet+1};if quiet>=120{return Ok(());}
            self.step(c,true).await?;
        }anyhow::bail!("Chess fight step{} stalled",self.index)
    }
    fn record(&self,suffix:&str)->Result<()> {
        let value=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"first":self.first,"skip":self.skip,
            "step":self.index,"samples":self.samples,"owners":self.owners,"frame":self.roster.frame(1,0),
            "world":self.arena.snapshot(),"recent_inputs":self.history,"goal":self.goal,"trail":self.trail,
            "course_sha256":self.course_sha256,"default_course":self.default_course,
            "course_passed":suffix=="passed","full_route_passed":suffix=="passed" && self.default_course});
        std::fs::write(crate::preferences::path(&format!("wchess1-{:?}-{}-{suffix}.json",self.first,self.skip)),serde_json::to_vec(&value)?)?;Ok(())
    }
}
async fn route(a:&mut Assets,first:PlayerId,skip:bool)->Result<()> {
    let visit=Visit{epoch:1,map:"wchess1".into(),entry:Some("wchess1_start1".into()),difficulty:Default::default(),combat_preview:false};
    let mut scene=RenderScene::load_for(a,&visit.map,visit.difficulty)?;
    let mut arena=Arena::load(a,&scene.map,&mut scene.world,&visit,false)?;
    let (eye,yaw)=crate::interaction::spawn(&scene.map,visit.entry.as_deref());
    let roster=Roster::new_with(&scene.world,eye,yaw,Stats::for_level(&visit.map,visit.entry.as_deref()),|at|!arena.stage.interactions.hazardous(at))?;
    arena.reset_characters(&roster.players);
    let mut guest_world=World::from_bsp(&scene.map)?;let guest=Arena::load(a,&scene.map,&mut guest_world,&visit,false)?;
    let bytes=if let Ok(path)=std::env::var("LOOKING_GLASS_CHESS_COOP_COURSE"){std::fs::read(path)?}else{include_bytes!("coop_steps.json").to_vec()};
    let mut r=Replay{arena,scene,roster,guest,guest_world,first,skip,index:0,trackers:Default::default(),samples:0,owners:Default::default(),history:Default::default(),trail:Default::default(),
        course_sha256:format!("{:x}",Sha256::digest(&bytes)),default_course:bytes==include_bytes!("coop_steps.json"),goal:None};
    let steps:Vec<Step>=serde_json::from_slice(&bytes)?;
    ensure!(!steps.is_empty() && steps.len()<1000 && steps.iter().all(|s|s.pos.iter().all(|v|v.is_finite()) && s.value.is_finite() && (0. ..=2000.).contains(&s.value)
        && s.radius.is_finite() && (1. ..=40.).contains(&s.radius)),"Invalid chess course");
    let result:Result<()>=async {
        r.step(Default::default(),false).await?;r.scene_end().await?;
        for (index,s) in steps.iter().enumerate() {
            r.index=index;println!("Chess step{index} {s:?}");
            match s.mode.as_str() {
                "walk"=>r.go([Vec3::from_array(s.pos);2],false,s.radius).await?,
                "wait"=>r.wait(s.value).await?,
                "spikes"=>r.spikes(s).await?,
                "clear"|"rush"=>r.clear(s.value,s.mode=="rush").await?,
                "lift"=>r.lift().await?,"lever"=>r.lever(&s.name).await?,
                "bishop"=>r.board(Piece::Bishop).await?,"knight"=>r.board(Piece::Knight).await?,
                _=>anyhow::bail!("Unknown chess course mode {}",s.mode),
            }
            if r.arena.stage.transition.is_some(){break;}
        }
        ensure!(r.realm().saved.bishop_done && r.realm().saved.knight_done && r.realm().water_ready()
            && r.owners.len()==5 && r.arena.stage.transition==Some(("wchess2".into(),Some("wchess2_start1".into()))),"Chess route did not finish all puzzles/scenes/exit");
        let next_visit=Visit{epoch:2,map:"wchess2".into(),entry:Some("wchess2_start1".into()),..visit};
        let mut next=RenderScene::load_for(a,&next_visit.map,next_visit.difficulty)?;
        let next_arena=Arena::load(a,&next.map,&mut next.world,&next_visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(&next.map,next_visit.entry.as_deref());
        let arrived=r.roster.arrive_with(&next.world,eye,yaw,&next_visit.map,next_visit.entry.as_deref(),|at|!next_arena.stage.interactions.hazardous(at))?;
        for (old,new) in r.roster.players.iter().zip(&arrived.players) {ensure!(serde_json::to_value(&old.stats)?==serde_json::to_value(&new.stats)? && next.world.body_clear(new.player.feet),"Castling arrival changed resources or embedded Alice");}
        Ok(())
    }.await;
    r.record(if result.is_ok(){"passed"}else{"failed"})?;result?;
    println!("PASS Pale Realm co-op {first:?} skip={skip} default_course={}: {} ticks, {} replicas, five scenes and Castling arrival",r.default_course,r.roster.tick,r.samples);Ok(())
}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {Box::pin(async move {
    std::fs::create_dir_all(crate::preferences::path(""))?;
    let chosen=std::env::var("LOOKING_GLASS_CHESS_COOP_FIRST").ok();let scenes=std::env::var("LOOKING_GLASS_CHESS_COOP_SCENES").ok();
    ensure!(chosen.as_deref().is_none_or(|s|matches!(s,"Blue"|"Red")) && scenes.as_deref().is_none_or(|s|matches!(s,"watched"|"skipped")),"Invalid chess route selector");
    let mut failed=Vec::new();for first in PlayerId::ALL {for skip in [false,true] {
        if chosen.as_ref().is_some_and(|s|*s!=format!("{first:?}")) || scenes.as_deref().is_some_and(|s|(s=="skipped")!=skip){continue;}
        if let Err(e)=route(a,first,skip).await {println!("FAIL full chess {first:?} skip={skip}: {e:#}");failed.push(format!("{first:?}/{skip}: {e:#}"));}
    }}ensure!(failed.is_empty(),"Chess routes failed: {}",failed.join("; "));Ok(())
})}

/// Read-only diagnosis of a captured route failure. These isolated physics
/// copies are never counted as traversal and never modify a running game.
pub(super) fn probe(a:&mut Assets)->Result<()> {
    let path=std::env::var_os("LOOKING_GLASS_CHESS_ROUTE_REPORT").context("Set LOOKING_GLASS_CHESS_ROUTE_REPORT to a captured route report")?;
    let report:serde_json::Value=serde_json::from_slice(&std::fs::read(path)?)?;
    let map=Bsp::parse(&a.read("maps/wchess1.bsp")?)?;
    let visit=Visit{epoch:1,map:"wchess1".into(),entry:Some("wchess1_start1".into()),difficulty:Default::default(),combat_preview:false};
    let mut world=World::from_bsp(&map)?;
    let mut stage=crate::coop::stage::Stage::load(a,&map,&mut world,&visit,false)?;
    stage.restore(&serde_json::from_value(report["world"]["stage"].clone())?,&map,&mut world)?;
    let frame:crate::coop::world::Frame=serde_json::from_value(report["frame"].clone())?;
    let first:PlayerId=serde_json::from_value(report["first"].clone())?;
    let saved=&frame.players[first.index()].player;
    let mut goal:Vec3=if let Ok(text)=std::env::var("LOOKING_GLASS_CHESS_ROUTE_GOAL") {serde_json::from_str(&text)?}
        else {serde_json::from_value(report["goal"].clone()).context("Report has no goal; set LOOKING_GLASS_CHESS_ROUTE_GOAL to [x,y,z]")?};
    ensure!(goal.is_finite(),"Invalid probe goal");
    // Keep a standing horizontal probe above the floor's collision skin.
    if (goal.z-saved.feet.z).abs()<0.1 {goal.z=saved.feet.z;}
    let bare=World::from_bsp(&map)?;
    let r=stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Realm>()).unwrap();
    println!("PROBE captured {:?} -> {goal:?}, trace {:?}, geometry {:?}",saved.feet,
        world.body_trace(saved.feet,goal),world.sweep_geometry(saved.feet+PLAYER_CENTER,goal+PLAYER_CENTER,PLAYER_HALF));
    println!("PROBE static trace {:?}, saved player {saved:?}",bare.body_trace(saved.feet,goal));
    let pair=frame.players.each_ref().map(|b|b.player.feet);
    let normal=(pair[0]-pair[1]).truncate().normalize_or_zero();
    let overlap=crate::coop::separation::DISTANCE-(pair[0]-pair[1]).truncate().length();
    for (i,b) in frame.players.iter().enumerate() {
        let at=b.player.feet;
        let corrected=at+normal.extend(0.)*overlap*if i==0{1.}else{-1.};
        for (name,to) in [("current",at),("corrected",corrected),("skin",corrected+Vec3::Z*crate::collision::SKIN)] {
            let ground=world.body_trace(to,to-Vec3::Z*2.);
            let depth=ground.normal.truncate().abs().dot(PLAYER_HALF.truncate())/ground.normal.z.max(0.001);
            println!("PROBE contact slot{i} {name} {to:?}: path{:?} clear{} ground{ground:?} centre{:?}",
                world.body_trace(at,to),world.body_clear(to),world.sweep(to+Vec3::Z,to-Vec3::Z*(2.+depth),Vec3::ZERO));
        }
    }
    for (id,e) in map.entities.iter().enumerate() {
        if let Some(model)=e.get("model").and_then(|s|s.strip_prefix('*')).and_then(|s|s.parse::<usize>().ok()) {
            let c=Collider::model(&map,model,data::at(e).translation,Quat::IDENTITY,true)?;
            let t=c.trace(saved.feet+PLAYER_CENTER,goal+PLAYER_CENTER,PLAYER_HALF);
            if t.start_solid || t.fraction<1. {println!("PROBE map model{id} {:?} {:?} trace{t:?}",e.get("classname"),e.get("targetname"));}
        }
    }
    for o in &r.objects {
        if o.solid {
            let t=o.collider.trace(saved.feet+PLAYER_CENTER,goal+PLAYER_CENTER,PLAYER_HALF);
            if t.start_solid || t.fraction<1. {println!("PROBE native object {} {} model{} trace{t:?}",o.id,o.name,o.model);}
        }
    }
    for origin in [saved.feet] {
        for (run,analog) in [(false,false),(true,false),(true,true)] {
            let mut p=saved.clone();p.feet=origin;p.velocity=Vec3::ZERO;
            for _ in 0..240 {
                let d=goal-p.feet;if d.truncate().length()<7.{break;}
                p.tick(&world,Controls{wish:d.truncate().normalize_or_zero()*if analog{(d.truncate().length()/48.).min(1.)}else{1.},run,..Default::default()});
            }
            println!("PROBE raw physics origin{origin:?} run{run} analog{analog}: {:?} clear{}",p.feet,world.body_clear(p.feet));
        }
    }
    for (label,w) in [("restored",&world),("static",&bare)] {
        for jump in [false,true] {
            let mut p=Player::new(saved.feet);
            for _ in 0..360 {
                let d=goal-p.feet;if d.truncate().length()<7.{break;}
                p.tick(w,Controls{wish:d.truncate().normalize_or_zero(),run:false,jump,..Default::default()});
            }
            println!("PROBE fresh {label} jump{jump}: {:?}",p.feet);
        }
    }
    if saved.swimming {
        for deliberate in [false,true] {
            let mut p=saved.clone();
            for _ in 0..360 {
                let d=goal-p.feet;
                if p.grounded && d.length()<22. {break;}
                let direction=d.truncate().normalize_or_zero();
                p.tick(&world,Controls{wish:direction*if deliberate{1.}else{(d.truncate().length()/48.).min(1.)},
                    swim:d.normalize_or_zero(),rise:if p.swimming && d.z>30.{1.}else{0.},run:true,
                    ..Default::default()});
                ensure!(world.body_clear(p.feet),"Captured bank replay entered a solid");
            }
            println!("PROBE bank deliberate{deliberate}: feet{:?} grounded{} climbs{} distance{}",
                p.feet,p.grounded,p.climbs,p.feet.distance(goal));
            if report["bank"].as_bool()==Some(true) {
                ensure!(if deliberate {p.grounded && p.feet.distance(goal)<22. && p.climbs>saved.climbs}
                    else {!p.grounded && p.climbs==saved.climbs},"Captured bank intent regression did not reproduce");
            }
        }
    }
    if report["plan"].as_bool()==Some(true) {
        let path=Route::plan(&world,saved,goal,22.)
            .map_err(|n|anyhow::anyhow!("Captured-state path exhausted {n} states"))?;
        let mut player=saved.clone();
        for (input,expected) in &path {
            for tick in 0..24 {
                player.tick(&world,Controls{jump:input.jump && tick==0,..*input});
                ensure!(world.body_clear(player.feet),"Captured-state plan entered a solid");
            }
            ensure!(player.feet.distance(*expected)<0.01,"Captured-state plan replay diverged");
        }
        ensure!(player.grounded && player.ledge.is_none() && player.feet.distance(goal)<22.,
            "Captured-state plan did not reach supported goal");
        println!("PASS captured-state ordinary-input path: {} segments, {:?} -> {:?}; isolated diagnosis only",
            path.len(),saved.feet,player.feet);
        std::fs::create_dir_all(crate::preferences::path(""))?;
        std::fs::write(crate::preferences::path("wchess1-captured-plan.json"),
            serde_json::to_vec_pretty(&serde_json::json!({"start":saved.feet,"goal":goal,
                "steps":path.iter().map(|(c,p)|serde_json::json!({"wish":c.wish,"jump":c.jump,
                    "run":c.run,"expected":p})).collect::<Vec<_>>()}))?)?;
    }
    Ok(())
}
