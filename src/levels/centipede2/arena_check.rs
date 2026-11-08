//! Production two-player pipeline, with telemetry instead of image inspection.
use super::*;
use crate::{coop::{arena::Arena, protocol::{Input,Visit}, world::Roster, PlayerId},
    movement::{Controls,FIXED_DT}, render::Scene};

fn visit() -> Visit {
    Visit{epoch:1,map:"centipede2".into(),entry:Some("centipede2_start1".into()),
        difficulty:Default::default(),combat_preview:false}
}
fn owner(arena:&Arena)->&Centipede {
    arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Centipede>()).unwrap()
}
fn owner_mut(arena:&mut Arena)->&mut Centipede {
    arena.stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Centipede>()).unwrap()
}
fn fixture(a:&mut Assets,actor:PlayerId)->Result<(Scene,Arena,Roster)> {
    let v=visit();
    let mut scene=Scene::load_for(a,&v.map,v.difficulty)?;
    let mut arena=Arena::load(a,&scene.map,&mut scene.world,&v,false)?;
    let (o,_,mut pair)=coop_check::fixture(a,&scene.map)?;
    if actor==PlayerId::Red {pair.reverse();}
    let (eye,yaw)=crate::interaction::spawn(&scene.map,v.entry.as_deref());
    let mut roster=Roster::new(&scene.world,eye,yaw,Stats::for_level("centipede2",None))?;
    for (body,(player,stats)) in roster.players.iter_mut().zip(pair) {body.player=player;body.stats=stats;}
    *owner_mut(&mut arena)=o;
    arena.stage.owner=None;
    arena.stage.interactions.sync_coop(&mut scene.world);
    arena.reset_characters(&roster.players);
    roster.remember_checkpoint();arena.remember_checkpoint();
    Ok((scene,arena,roster))
}
fn acid(arena:&mut Arena,at:Vec3) {
    let o=owner_mut(arena);
    o.saved.battle.set(battle::Action::Return);
    let start=at+Vec3::Z*32.-Vec3::X*20.;
    o.saved.battle.shots.push(battle::Shot{at:start,start,velocity:Vec3::X*550.,age:0.,ended:None,fragment:false,normal:Vec3::ZERO});
}
async fn controls(a:&mut Assets,actor:PlayerId)->Result<()> {
    let (mut scene,mut arena,mut roster)=fixture(a,actor)?;
    let index=actor.index();let partner=actor.partner().index();
    let at=roster.players[index].player.feet;
    let o=owner_mut(&mut arena);let b=&mut o.saved.battle;
    b.hits=4;b.stage=3;
    let delta=at-b.pose.translation;b.pose.rotation=Quat::from_rotation_z(delta.y.atan2(delta.x));
    b.set(battle::Action::Strike);b.time=o.data.boss().duration("attack_grab_strike");
    arena.step(&mut roster,&scene.map,&mut scene.world,Default::default())?;
    ensure!(owner(&arena).coop_controlled(index) && !arena.stage.busy(),"Arena grab became a shared scene or captured nobody");
    let start=roster.players[partner].player.feet;
    let direction=[Vec2::Y,Vec2::NEG_Y,Vec2::X,Vec2::NEG_X].into_iter().find(|wish| {
        let mut probe=roster.players[partner].player.clone();
        for _ in 0..24 {probe.tick(&scene.world,Controls{wish:*wish,..Default::default()});}
        probe.feet.distance(start)>12. && scene.world.body_clear(probe.feet)
    }).context("No short partner movement probe")?;
    let will=roster.players[index].stats.will();
    for _ in 0..24 {
        let mut inputs=[(Controls::default(),Input::default());2];
        inputs[index]=(Controls{wish:Vec2::NEG_X,..Default::default()},Input{primary:true,weapon:Some(1),..Default::default()});
        inputs[partner].0.wish=direction;
        arena.step(&mut roster,&scene.map,&mut scene.world,inputs)?;
        ensure!(owner(&arena).coop_controlled(index) && !owner(&arena).coop_controlled(partner),"Grab locked wrong body");
        ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Grab overlapped players");
    }
    ensure!(roster.players[partner].player.feet.distance(start)>12.
        && roster.players[index].stats.will()==will && owner(&arena).saved.battle.hits==4,"Grab blocked partner or spent victim weapon resources");
    let saved=arena.snapshot();let bytes=crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
    arena.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,&scene.map,&mut scene.world)?;
    ensure!(serde_json::to_value(arena.snapshot())?==serde_json::to_value(saved)?,"Grab replica differs");
    println!("PASS Sanctum Arena {actor:?} grab: victim controls locked, partner moves, no overlap, compressed replica agrees");
    next_frame().await;
    Ok(())
}
async fn attacks(a:&mut Assets,actor:PlayerId)->Result<()> {
    let (mut scene,mut arena,mut roster)=fixture(a,actor)?;
    let index=actor.index();let partner=actor.partner().index();
    acid(&mut arena,roster.players[index].player.feet);
    arena.step(&mut roster,&scene.map,&mut scene.world,Default::default())?;
    ensure!(roster.players[index].stats.sanity()==85. && roster.players[partner].stats.sanity()==100.,"Arena acid damaged wrong personal health");
    println!("PASS Sanctum Arena {actor:?} acid: native feedback reaches only the struck player's health");
    next_frame().await;
    let (mut scene,mut arena,mut roster)=fixture(a,actor)?;
    owner_mut(&mut arena).saved.battle.set(battle::Action::Crush);
    let will=roster.players.each_ref().map(|p|p.stats.will());
    for _ in 0..360 {
        let target=owner(&arena).saved.battle.targets(&owner(&arena).data).into_iter().find(|t|t.id==BASE).context("No weak-point target")?;
        let aim=(target.center-roster.players[index].player.eye()).normalize_or_zero();
        let mut inputs=[(Controls::default(),Input::default());2];
        inputs[index].1=Input{primary:true,weapon:Some(1),yaw:aim.y.atan2(aim.x),pitch:aim.z.asin(),..Default::default()};
        arena.step(&mut roster,&scene.map,&mut scene.world,inputs)?;
        if owner(&arena).saved.battle.hits>0 {break;}
    }
    ensure!(owner(&arena).saved.battle.hits==1 && roster.players[index].stats.will()<will[index]
        && roster.players[partner].stats.will()==will[partner],"Funded native Cards input missed weak point or charged partner");
    println!("PASS Sanctum Arena {actor:?} weapon: funded Cards input hits the animated weak point, partner Will unchanged");
    next_frame().await;
    Ok(())
}
async fn recovery(a:&mut Assets,actor:PlayerId)->Result<()> {
    let (mut scene,mut arena,mut roster)=fixture(a,actor)?;
    let index=actor.index();let partner=actor.partner().index();
    roster.players[index].stats.set_health(10.)?;
    roster.players[index].stats.spend_will(17.);
    roster.players[partner].stats.invisible=20.;
    acid(&mut arena,roster.players[index].player.feet);
    arena.step(&mut roster,&scene.map,&mut scene.world,Default::default())?;
    ensure!(!roster.players[index].stats.alive() && roster.players[partner].stats.sanity()==100.,"Native acid did not isolate death");
    let death_tick=roster.tick;
    for _ in 0..1199 {
        ensure!(crate::coop::world::camera_player(&roster.frame(1,0),actor)==actor.partner(),"Spectator camera stopped following survivor");
        arena.step(&mut roster,&scene.map,&mut scene.world,Default::default())?;
        ensure!(!roster.players[index].stats.alive(),"Player revived early");
    }
    arena.step(&mut roster,&scene.map,&mut scene.world,Default::default())?;
    ensure!(roster.tick-death_tick==1200 && roster.players[index].stats.sanity()==25.
        && roster.players[index].stats.will()==83. && roster.players[partner].stats.sanity()==100.,"Wrong revive timer/resources");
    ensure!(roster.players[index].player.feet.distance(roster.players[partner].player.feet)>=36.
        && roster.players[index].player.feet.distance(roster.players[partner].player.feet)<=128.
        && scene.world.body_clear(roster.players[index].player.feet),"Unsafe or distant arena revival");
    println!("PASS Sanctum Arena {actor:?} recovery: native acid death, spectator follow, 1200-tick/25-health personal revival");
    next_frame().await;
    Ok(())
}
async fn wipe(a:&mut Assets)->Result<()> {
    let (mut scene,mut arena,mut roster)=fixture(a,PlayerId::Blue)?;
    let saved=owner(&arena).snapshot();let positions=roster.players.each_ref().map(|p|p.player.feet);
    for i in 0..2 {roster.players[i].stats.set_health(10.)?;acid(&mut arena,roster.players[i].player.feet);}
    arena.step(&mut roster,&scene.map,&mut scene.world,Default::default())?;
    ensure!(roster.checkpoint_generation==1 && roster.players.iter().all(|p|p.stats.sanity()==100.)
        && owner(&arena).snapshot()==saved && roster.players.each_ref().map(|p|p.player.feet)==positions,"Team wipe did not restore paired checkpoint and boss");
    println!("PASS Sanctum Arena wipe: both native acid deaths restore both bodies/resources and the boss checkpoint");
    next_frame().await;Ok(())
}
async fn effects(a:&mut Assets)->Result<()> {
    let (mut scene,mut host,mut roster)=fixture(a,PlayerId::Blue)?;
    owner_mut(&mut host).saved.battle.set(battle::Action::Spit);
    let v=visit();let mut guest_world=World::from_bsp(&scene.map)?;
    let mut guest=Arena::load(a,&scene.map,&mut guest_world,&v,false)?;
    let mut host_art=crate::viewer::LevelArt::default();host_art.prepare(a,&scene,&host.stage.interactions,false)?;host_art.coop_view(0);
    let mut guest_art=crate::viewer::LevelArt::default();guest_art.prepare(a,&scene,&guest.stage.interactions,false)?;guest_art.coop_view(1);
    let target=owner(&host).saved.battle.pose.translation+Vec3::Z*160.;
    let view=Camera3D{position:target+vec3(700.,-600.,300.),target,up:Vec3::Z,z_near:2.,z_far:30000.,..Default::default()};
    let mut batches=[0;2];let mut matches=0;let mut saw_shots=false;
    for tick in 0..120 {
        host.step(&mut roster,&scene.map,&mut scene.world,Default::default())?;
        saw_shots|=!owner(&host).saved.battle.shots.is_empty();
        if tick%4!=0 {continue;}
        let saved=host.snapshot();let bytes=crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
        guest.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,&scene.map,&mut guest_world)?;
        ensure!(serde_json::to_value(guest.snapshot())?==serde_json::to_value(saved)?,"Guest boss/particle replica differs");matches+=1;
        for (i,(arena,art)) in [(&host,&mut host_art),(&guest,&mut guest_art)].into_iter().enumerate() {
            clear_background(BLACK);set_camera(&view);
            crate::render_fx::begin_view(&view,tick as f32*FIXED_DT,&scene.atmosphere,false);
            art.draw(&arena.stage.interactions,&arena.stage.hints,&scene.atmosphere,view.position,false);
            art.effects(&arena.stage.interactions,view.position,&scene.atmosphere);
            let (drawn,dropped)=crate::render_fx::finish();batches[i]+=drawn;
            ensure!(dropped==0,"Particle pass dropped geometry");
            set_default_camera();next_frame().await;
        }
    }
    ensure!(saw_shots && batches.iter().all(|n|*n>0),"Live spit produced no rendered effects on one side");
    println!("PASS Sanctum Arena effects: {matches} exact host/guest snapshots, particle batches {batches:?}, no drops");
    Ok(())
}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        for actor in PlayerId::ALL {controls(a,actor).await?;attacks(a,actor).await?;recovery(a,actor).await?;}
        wipe(a).await?;effects(a).await?;
        println!("PASS Sanctum 10 production Arena cases; full paired traversal and internet remain open");Ok(())
    })
}
