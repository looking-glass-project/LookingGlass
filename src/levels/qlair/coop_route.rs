//! Fresh paired finale using ordinary movement, weapons and production recovery.
use super::*;
use crate::{coop::{arena::Arena, checkpoint, native_route::Walking,
    protocol::{Input,Visit}, recovery_trace, world::Roster},
    movement::{Controls,FIXED_DT}, route::fight};

fn owner(arena:&Arena)->&Queen {
    arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref()).unwrap()
}
#[derive(Default)]
struct Pilot { path:Walking, corner:usize, camp:Option<Vec3>, wish:Vec2, strafe:f32 }

pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move { run(a).await })
}
async fn run(a:&mut Assets)->Result<()> {
    let visit=Visit{epoch:1,map:"qlair".into(),entry:Some("qlair_start1".into()),
        difficulty:Default::default(),combat_preview:false};
    let map=Bsp::parse(&a.read("maps/qlair.bsp")?)?;
    let mut world=World::from_bsp(&map)?;
    let mut arena=Arena::load(a,&map,&mut world,&visit,false)?;
    let (eye,yaw)=crate::interaction::spawn(&map,visit.entry.as_deref());
    let mut roster=Roster::new_with(&world,eye,yaw,Stats::for_level("qlair",None),
        |at|!arena.stage.interactions.hazardous(at))?;
    arena.reset_characters(&roster.players);
    let mut guest_world=World::from_bsp(&map)?;
    let mut guest=Arena::load(a,&map,&mut guest_world,&visit,false)?;
    let mut pilots:[Pilot;2]=Default::default();
    pilots[1].corner=2;
    let mut trackers:[fight::Tracker;2]=Default::default();
    let mut samples=0;
    let mut reunions=0;
    let mut phases=Vec::new();
    let mut scenes=Vec::new();
    let mut life=Vec::new();
    let mut saved_checkpoint=None;
    let mut last_phase=Phase::Corridor;
    std::fs::create_dir_all(crate::preferences::path(""))?;
    let recovery_name="qlair-recovery.jsonl";
    let mut recovery=recovery_trace::Trace::new(crate::preferences::path(recovery_name))?;
    recovery.record(&roster,&world,1,"qlair",false,false,|at|!arena.stage.interactions.hazardous(at))?;
    let result:Result<()>=async {
        for tick in 0..120*1200 {
            let q=owner(&arena);
            let phase=q.saved.phase;
            if phase!=last_phase {
                phases.push(serde_json::json!({"phase":phase,"tick":roster.tick,
                    "queens":[q.saved.queen1,q.saved.queen2],"health":roster.players.each_ref().map(|b|b.stats.sanity())}));
                println!("QUEEN {phase:?} tick{} health{:?}",roster.tick,roster.players.each_ref().map(|b|b.stats.sanity()));
                last_phase=phase;
                pilots=Default::default();pilots[1].corner=2;
            }
            if phase==Phase::Done {break;}
            let old_owner=arena.stage.owner;
            let before=roster.players.each_ref().map(|b|b.player.feet);
            let alive=roster.players.each_ref().map(|b|b.stats.alive());
            let mut inputs=[(Controls::default(),Input::default());2];
            let targets=q.battle_targets();
            let context=crate::combat::Context{world:&world,targets:&targets};
            for i in 0..2 {
                let b=&roster.players[i];let p=&b.player;let pilot=&mut pilots[i];
                inputs[i].1.yaw=b.yaw;inputs[i].1.pitch=b.pitch;
                if !b.stats.alive() || arena.stage.busy() {continue;}
                trackers[i].observe(&targets,FIXED_DT);
                let target_id=if phase==Phase::Queen1 {BASE}else{BASE+1};
                let aim=targets.iter().find(|t|t.id==target_id)
                    .and_then(|t|fight::aim_point(&context,p.eye(),t));
                let attack=if let Some(point)=aim.filter(|_|b.stats.copies(7)>0 && b.stats.will()>21.) {
                    fight::Plan{target:Some(target_id),aim:(point-p.eye()).normalize_or_zero(),weapon:7,alternate:false}
                } else {fight::plan(&context,p.eye(),&targets,&trackers[i],b.stats.will()>=fight::CARD_COST)};
                if attack.target.is_some() {
                    inputs[i].1.weapon=Some(attack.weapon as u8);
                    inputs[i].1.primary=!attack.alternate;inputs[i].1.alternate=attack.alternate;
                    inputs[i].1.yaw=attack.aim.y.atan2(attack.aim.x);
                    inputs[i].1.pitch=attack.aim.z.asin().clamp(-1.56,1.56);
                }
                inputs[i].0=match phase {
                    Phase::Corridor=>Controls{wish:Vec2::Y,run:true,..Default::default()},
                    Phase::Queen1=>{
                        let corners=[vec2(-160.,-850.),vec2(160.,-850.),vec2(160.,-600.),vec2(-160.,-600.)];
                        let delta=corners[pilot.corner%4]-p.feet.truncate();
                        if delta.length()<45. {pilot.corner+=1;}
                        Controls{wish:delta.normalize_or_zero(),run:true,jump:tick%180==i as u64*90,..Default::default()}
                    },
                    Phase::Queen2=>{
                        let camp=if let Some(camp)=pilot.camp {camp} else {
                            let base=check::arena_camp(&world,vec3(-120.,3784.,984.),Vec2::X)?;
                            let camp=base+Vec3::X*if i==0 {-48.}else{48.};
                            pilot.camp=Some(camp);camp
                        };
                        if p.feet.distance(camp)>95. {
                            pilot.path.input(&world,p,camp,32.,true)?
                        } else {
                            let from=p.feet.truncate()-camp.truncate();
                            if pilot.strafe==0. {pilot.strafe=if i==0 {1.}else{-1.};}
                            if from.x*pilot.strafe>35. {pilot.strafe=-pilot.strafe;}
                            if tick%12==0 {
                                pilot.wish=check::dodge_player(q,&world,p,camp,Vec2::X,
                                    vec2(pilot.strafe,-from.y/40.).clamp_length_max(1.),false);
                            }
                            Controls{wish:pilot.wish,..Default::default()}
                        }
                    },
                    _=>Controls::default(),
                };
            }
            arena.step(&mut roster,&map,&mut world,inputs)?;
            for i in 0..2 {pilots[i].path.advanced(&roster.players[i].player);}
            recovery.record(&roster,&world,1,"qlair",old_owner.is_some()||arena.stage.busy(),
                arena.stage.interactive_actor().is_some(),|at|!arena.stage.interactions.hazardous(at))?;
            for i in 0..2 {
                let b=&roster.players[i];
                if b.stats.alive()!=alive[i] {
                    life.push(serde_json::json!({"slot":i,"tick":roster.tick,"alive":b.stats.alive(),"spawned_at":b.spawned_at}));
                    pilots[i]=Default::default();
                }
            }
            if old_owner.is_none() && arena.stage.owner.is_some() {
                scenes.push(serde_json::json!({"tick":roster.tick,"owner":arena.stage.owner,"phase":owner(&arena).saved.phase}));
            }
            if let Some(id)=arena.stage.owner.filter(|id|Some(*id)==old_owner) {
                ensure!(roster.players[id.partner().index()].player.feet==before[id.partner().index()],"Queen scene moved hidden partner");
            }
            if old_owner.is_some() && arena.stage.owner.is_none() {reunions+=1;}
            ensure!(roster.checkpoint_generation==0,"Finale route team-wiped at tick{}",roster.tick);
            ensure!(roster.players.iter().all(|b|!b.stats.god&&!b.stats.notarget),"Finale enabled cheats");
            if saved_checkpoint.is_none() && owner(&arena).checkpoint_requested()
                && !arena.stage.busy() && roster.players.iter().all(|b|b.stats.alive()) {
                let store=checkpoint::Store::current(a.fingerprint()?);
                store.save(&visit,&mut roster,&mut arena,&world,&Default::default())?;
                let saved=store.load()?.checkpoint;
                checkpoint::verify_restored(&saved,&roster,&arena)?;
                guest.restore_checkpoint(saved.frame.combat.as_ref().unwrap(),&map,&mut guest_world)?;
                let loaded=Roster::from_checkpoint(&saved.frame,&guest_world,|at|!guest.stage.interactions.hazardous(at))?;
                checkpoint::verify_restored(&saved,&loaded,&guest)?;
                saved_checkpoint=Some(serde_json::json!({"tick":roster.tick,"phase":owner(&arena).saved.phase,"exact_restore":true}));
            }
            if tick%120==0 || owner(&arena).ending_ready() {
                roster.frame(1,0).validate()?;
                let snapshot=arena.snapshot();
                let packed=crate::coop::codec::encode(&serde_json::to_vec(&snapshot)?)?;
                guest.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&packed)?)?,&map,&mut guest_world)?;
                ensure!(serde_json::to_value(snapshot)?==serde_json::to_value(guest.snapshot())?,"Finale replica differs");
                samples+=1;
                if tick%2400==0 {println!("QUEEN t={} {:?} hp{}/{} personal{:?} feet{:?}",tick/120,owner(&arena).saved.phase,
                    owner(&arena).saved.queen1,owner(&arena).saved.queen2,roster.players.each_ref().map(|b|b.stats.sanity()),
                    roster.players.each_ref().map(|b|b.player.feet));}
                clear_background(BLACK);next_frame().await;
            }
        }
        ensure!(owner(&arena).ending_ready()&&owner(&guest).ending_ready()&&arena.stage.transition.is_none(),"Finale did not complete both replicas");
        ensure!(roster.players.iter().all(|b|b.stats.alive()),"Finale ended without both players alive");
        ensure!(saved_checkpoint.is_some(),"Missing normal second-arena checkpoint and exact restore");
        Ok(())
    }.await;
    let report=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"visit":visit,"full_route_passed":result.is_ok(),
        "error":result.as_ref().err().map(|e|format!("{e:#}")),"replica_samples":samples,"reunions":reunions,
        "life_events":life,"scene_starts":scenes,"phases":phases,"checkpoint":saved_checkpoint,
        "frame":roster.frame(1,0),"world":arena.snapshot(),"arrival":null,"recovery_log":recovery_name,
        "ending_ready":owner(&arena).ending_ready(),"health_floors":roster.players.each_ref().map(|b|b.stats.minimum_sanity),
        "scope":"Normal paired finale combat, scenes, checkpoint and ending state; film/credits presentation is separate"});
    std::fs::write(crate::preferences::path("qlair-coop-route.json"),serde_json::to_vec(&report)?)?;
    result?;
    println!("PASS normal paired Queen finale: {} ticks, {samples} exact replicas, actual ending",roster.tick);
    Ok(())
}
