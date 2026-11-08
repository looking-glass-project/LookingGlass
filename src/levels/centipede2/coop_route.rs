//! Full native Arena route. Only ordinary movement, weapon and scene-skip inputs.
use super::*;
use crate::{coop::{arena::Arena, protocol::{Input,Visit}, world::{Body,Frame,Roster},PlayerId},
    movement::Controls, render::Scene};
use std::collections::VecDeque;
fn owner(arena:&Arena)->&Centipede {
    arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Centipede>()).unwrap()
}
fn input(arena:&Arena,body:&Body,id:PlayerId,first:PlayerId,goals:&[Vec3],steps:&mut [usize;2],climb_ticks:usize,lead:f32)->(Controls,Input) {
    let o=owner(arena);let p=&body.player;
    let mut c=Controls::default();let mut keys=Input{yaw:body.yaw,pitch:body.pitch,..Default::default()};
    match o.saved.phase {
        Phase::Slide => {c.wish=Vec2::X;}
        Phase::Fight => {
            let b=&o.saved.battle;let delta=(b.pose.translation-p.feet).truncate();
            let direction=delta.normalize_or_zero();let radial=((delta.length()-370.)/80.).clamp(-1.,1.);
            let tangent=vec2(-direction.y,direction.x)*if id==PlayerId::Blue {1.} else {-1.};
            c.wish=(direction*radial+tangent*0.55).normalize_or_zero();c.run=true;
            if body.stats.sanity()<40. {
                if let Some(at)=o.personal_essence(Some(id.index())) {c.wish=(at-p.feet).truncate().normalize_or_zero();}
            }
            let targets=b.targets(&o.data);
            let target=if b.weak(&o.data) {targets.iter().find(|t|t.id==BASE)} else {
                targets.iter().filter(|t|t.id>=BASE+100).min_by(|a,b|p.eye().distance_squared(a.center).total_cmp(&p.eye().distance_squared(b.center)))
            };
            if let Some(target)=target {
                let aim=(target.center-p.eye()).normalize_or_zero();
                keys.yaw=aim.y.atan2(aim.x);keys.pitch=aim.z.asin();keys.weapon=Some(1);keys.primary=true;
            }
        }
        Phase::Climb => {
            if id!=first && steps[first.index()]<2 && climb_ticks<240 {return (c,keys);}
            while let Some(goal)=goals.get(steps[id.index()]) {
                let delta=*goal-p.feet;let d=delta.truncate();
                if d.length()<10. && delta.z.abs()<18. && p.grounded
                    && (lead==0. || p.velocity.truncate().length()<25.) {steps[id.index()]+=1;continue;}
                let steering=d-p.velocity.truncate()*lead;
                c.wish=steering.normalize_or_zero()*(steering.length()/60.).min(1.);c.run=true;
                // A pull-up continues by itself. Repeated jump edges from a
                // delayed view can otherwise arrive after it lands and hop off.
                c.jump=p.ledge.as_ref().is_some_and(|h|!h.pulling)
                    || p.grounded && delta.z>24. && d.length()<330.;
                if d.length()>0. {keys.yaw=d.y.atan2(d.x);keys.pitch=0.;}
                break;
            }
        }
        _=>{}
    }
    (c,keys)
}
/// Opt-in client-side fixture. Uses only the received world and emits ordinary
/// controls; it cannot mutate the boss, bodies, resources or scene progression.
pub(crate) struct NetworkDriver {
    goals:Vec<Vec3>,
    steps:[usize;2],
    climb_ticks:usize,
    last_tick:Option<u64>,
    cached:(Controls,Input),
}
impl NetworkDriver {
    pub(crate) fn new()->Result<Self> {
        Ok(Self{goals:check::climb_goals()?,steps:[0;2],climb_ticks:0,last_tick:None,cached:Default::default()})
    }
    pub(crate) fn report(&self)->serde_json::Value {
        serde_json::json!({"steps":self.steps,"climb_ticks":self.climb_ticks,"observed_tick":self.last_tick})
    }
    pub(crate) fn sample(&mut self,frame:&Frame,arena:&Arena,local:PlayerId,first:PlayerId,lead:f32,keys:&mut Input)->Result<()> {
        let o=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Centipede>())
            .context("Sanctum course command used outside Sanctum")?;
        let fresh=self.last_tick!=Some(frame.tick);
        if fresh {
            let elapsed=self.last_tick.map_or(1,|old|frame.tick.saturating_sub(old) as usize);
            ensure!(elapsed>0 && elapsed<=360,"Sanctum course observation clock reset or stalled");
            self.last_tick=Some(frame.tick);self.cached=Default::default();
            if !arena.stage.busy() && frame.players[local.index()].stats.alive() {
                if o.saved.phase==Phase::Climb {self.climb_ticks+=elapsed;}
                for id in PlayerId::ALL {
                    let chosen=input(arena,&frame.players[id.index()],id,first,&self.goals,&mut self.steps,self.climb_ticks,lead);
                    if id==local {
                        self.cached=chosen;
                        if id!=first && o.saved.phase==Phase::Climb {
                            let gap=frame.players[id.index()].player.feet-frame.players[first.index()].player.feet;
                            // Leave room for the leader to brake and settle at
                            // a shared waypoint, rather than pinning them there.
                            // Also reserve the final approach for the chosen actor.
                            if self.steps[id.index()]>=7 || gap.truncate().length()<96. && gap.z.abs()<120. {
                                self.cached.0=Controls::default();
                            }
                        }
                    }
                }
            }
        }
        let (control,input)=self.cached;
        keys.wish=control.wish;keys.run=control.run;
        keys.yaw=input.yaw;keys.pitch=input.pitch.clamp(-1.56,1.56);
        keys.primary=input.primary;keys.alternate=input.alternate;keys.weapon=input.weapon;
        if fresh {keys.jumps+=u64::from(control.jump);}
        keys.validate()
    }
}
async fn route(a:&mut Assets,first:PlayerId,skip:bool)->Result<()> {
    let visit=Visit{epoch:1,map:"centipede2".into(),entry:Some("centipede2_start1".into()),difficulty:Default::default(),combat_preview:false};
    let mut scene=Scene::load_for(a,&visit.map,visit.difficulty)?;
    let mut arena=Arena::load(a,&scene.map,&mut scene.world,&visit,false)?;
    let (eye,yaw)=crate::interaction::spawn(&scene.map,visit.entry.as_deref());
    let mut roster=Roster::new_with(&scene.world,eye,yaw,Stats::for_level(&visit.map,visit.entry.as_deref()),|at|!arena.stage.interactions.hazardous(at))?;
    arena.reset_characters(&roster.players);
    let mut guest_world=World::from_bsp(&scene.map)?;
    let mut guest=Arena::load(a,&scene.map,&mut guest_world,&visit,false)?;
    let goals=check::climb_goals()?;
    let mut steps=[0;2];let mut climb_ticks=0;let mut history=VecDeque::new();let mut samples=0;let mut reunions=0;let mut mushroom_owner=None;
    let result:Result<()> = async {
        for tick in 0..120*600 {
            let before=roster.players.each_ref().map(|p|p.player.feet);
            let old_owner=arena.stage.owner;let old_phase=owner(&arena).saved.phase;
            let old_warps=roster.players.each_ref().map(|p|p.warped_at);
            let mut inputs=[(Controls::default(),Input::default());2];
            if !arena.stage.busy() {
                if old_phase==Phase::Climb {climb_ticks+=1;}
                for id in PlayerId::ALL {inputs[id.index()]=input(&arena,&roster.players[id.index()],id,first,&goals,&mut steps,climb_ticks,0.);}
            } else if skip && old_phase==Phase::Intro {
                let id=arena.stage.owner.context("Intro has no owner")?;
                arena.stage.interactions.skip_cinematic(&scene.map,&mut scene.world,&mut roster.players[id.index()].player,&mut arena.stage.story)?;
            }
            history.push_back(serde_json::json!({"tick":roster.tick,"feet":before,"steps":steps,
                "inputs":inputs.each_ref().map(|(c,i)|serde_json::json!({"wish":c.wish,"jump":c.jump,"run":c.run,"keys":i}))}));
            if history.len()>240 {history.pop_front();}
            arena.step(&mut roster,&scene.map,&mut scene.world,inputs)?;
            if let Some(id)=arena.stage.owner.filter(|id|Some(*id)==old_owner) {
                ensure!(roster.players[id.partner().index()].player.feet==before[id.partner().index()],"Scene moved waiting partner");
            }
            if old_owner.is_some() && arena.stage.owner.is_none() {reunions+=1;}
            if old_phase==Phase::Climb && owner(&arena).saved.phase==Phase::Greeting {mushroom_owner=arena.stage.owner;}
            ensure!(roster.checkpoint_generation==0 && roster.players.iter().all(|b|b.stats.alive() && b.spawned_at==0 && !b.stats.god && !b.stats.notarget)
                && (old_owner.is_some() || owner(&arena).saved.phase==Phase::Drop || roster.players.each_ref().map(|b|b.warped_at)==old_warps),
                "Sanctum death/recovery/warp tick{tick} phase{:?} steps{steps:?} health{:?}",owner(&arena).saved.phase,roster.players.each_ref().map(|b|b.stats.sanity()));
            if !arena.stage.busy() {
                let d=roster.players[0].player.feet-roster.players[1].player.feet;
                ensure!(d.truncate().length()>=crate::coop::separation::DISTANCE-0.05 || d.z.abs()>=crate::collision::PLAYER_HALF.z*2.-0.05,"Sanctum bodies overlap tick{tick}: {d:?}");
            }
            if tick%600==0 {
                let snapshot=arena.snapshot();let bytes=crate::coop::codec::encode(&serde_json::to_vec(&snapshot)?)?;
                guest.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,&scene.map,&mut guest_world)?;
                ensure!(serde_json::to_value(guest.snapshot())?==serde_json::to_value(snapshot)?,"Sanctum guest state differs");samples+=1;
                println!("Sanctum {first:?} skip={skip} t={} phase{:?} hits{} steps{steps:?} health{:?} feet{:?}",tick/120,owner(&arena).saved.phase,owner(&arena).saved.battle.hits,
                    roster.players.each_ref().map(|b|b.stats.sanity()),roster.players.each_ref().map(|b|b.player.feet));
                clear_background(BLACK);next_frame().await;
            }
            if arena.stage.transition.is_some() {break;}
        }
        ensure!(arena.stage.transition.as_ref().is_some_and(|e|EXIT.matches(e)) && owner(&arena).saved.battle.hits==6
            && reunions>=2 && mushroom_owner==Some(first),"Sanctum route omitted boss, reunions or leading player's mushroom exit");
        let next_visit=Visit{epoch:2,map:"wforest".into(),entry:Some("wforest_start1".into()),..visit.clone()};
        let mut next=Scene::load_for(a,&next_visit.map,next_visit.difficulty)?;
        let next_arena=Arena::load(a,&next.map,&mut next.world,&next_visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(&next.map,next_visit.entry.as_deref());
        let arrived=roster.arrive_with(&next.world,eye,yaw,&next_visit.map,next_visit.entry.as_deref(),|at|!next_arena.stage.interactions.hazardous(at))?;
        ensure!(arrived.players[0].player.feet.distance(arrived.players[1].player.feet)>=36.,"Pale Realm arrival overlap");
        for (old,new) in roster.players.iter().zip(&arrived.players) {
            ensure!(serde_json::to_value(&old.stats)?==serde_json::to_value(&new.stats)? && next.world.body_clear(new.player.feet),"Arrival changed resources or embedded player");
        }
        Ok(())
    }.await;
    let report=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"first":first,"skip":skip,"samples":samples,"full_route_passed":result.is_ok(),
        "error":result.as_ref().err().map(|e|format!("{e:#}")),"steps":steps,"reunions":reunions,"mushroom_owner":mushroom_owner,
        "frame":roster.frame(1,0),"world":arena.snapshot(),"recent_inputs":history});
    std::fs::write(crate::preferences::path(&format!("centipede2-{first:?}-{skip}-{}.json",if result.is_ok(){"passed"}else{"failed"})),serde_json::to_vec(&report)?)?;
    result?;println!("PASS Sanctum full paired route {first:?} skip={skip}: {}ticks, {samples} exact replicas, {reunions} reunions",roster.tick);Ok(())
}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        std::fs::create_dir_all(crate::preferences::path(""))?;
        let first=std::env::var("LOOKING_GLASS_SANCTUM_COOP_FIRST").ok();let scenes=std::env::var("LOOKING_GLASS_SANCTUM_COOP_SCENES").ok();
        ensure!(first.as_deref().is_none_or(|s|matches!(s,"Blue"|"Red")) && scenes.as_deref().is_none_or(|s|matches!(s,"watched"|"skipped")),"Invalid Sanctum route selector");
        let mut failures=Vec::new();
        for actor in PlayerId::ALL {for skip in [false,true] {
            if first.as_ref().is_some_and(|s|*s!=format!("{actor:?}")) || scenes.as_deref().is_some_and(|s|(s=="skipped")!=skip) {continue;}
            if let Err(e)=route(a,actor,skip).await {println!("FAIL Sanctum {actor:?} skip={skip}: {e:#}");failures.push(format!("{actor:?}/{skip}: {e:#}"));}
        }}
        ensure!(failures.is_empty(),"Sanctum routes failed: {}",failures.join("; "));Ok(())
    })
}
