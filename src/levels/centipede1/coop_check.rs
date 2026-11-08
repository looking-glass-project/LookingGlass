//! Staged native co-op evidence; full traversal/network coverage is separate.
use super::*;
use crate::{combat::PlayerTarget,coop::{protocol::Visit,stage::Stage,world::Roster,PlayerId},
    level::CoopPlayer,movement::{Controls,FIXED_DT,EYE_HEIGHT}};

fn setup(a:&mut Assets,map:&Bsp)->Result<(Flora,World)> {
    let f=Flora::load(a,map)?;let mut w=World::from_bsp(map)?;w.set_dynamic(f.colliders());Ok((f,w))
}
fn players(w:&World)->Result<([Player;2],[Stats;2])> {
    let p=Player::spawn(w,vec3(-280.,4700.,-600.)).context("Flora clock fixture blocked")?;
    Ok(([p.clone(),p],std::array::from_fn(|_|Stats::for_level("centipede1",None))))
}
fn tick(f:&mut Flora,map:&Bsp,w:&mut World,p:&mut [Player;2],s:&mut [Stats;2],dt:f32)->Result<()> {
    let [p0,p1]=p.each_mut();let [s0,s1]=s.each_mut();
    f.coop_advance_roster(dt,map,w,&mut [CoopPlayer{player:p0,stats:s0},CoopPlayer{player:p1,stats:s1}],&[])
}
fn shared(f:&Flora)->serde_json::Value {let mut s=f.snapshot();s.as_object_mut().unwrap().remove("coop");s}
fn clocks(a:&mut Assets,map:&Bsp)->Result<()> {
    for hz in [30,60,144] {for dead in [None,Some(0),Some(1)] {
        let (mut f,mut w)=setup(a,map)?;let (mut sp,mut sw)=setup(a,map)?;
        let (mut p,mut s)=players(&w)?;let mut single=p[0].clone();
        // Coincident clock probes make both observers equivalent to the single
        // observer. This is a timing comparison, not a body-spacing fixture.
        f.phase(Phase::Countdown);sp.phase(Phase::Countdown);
        s[0].set_health(54.)?;s[1].set_health(5.)?;
        if let Some(i)=dead {s[i].set_health(0.)?;}
        tick(&mut f,map,&mut w,&mut p,&mut s,0.)?;
        let mut updates=0_i32;
        while f.saved.phase!=Phase::Done && updates<hz*11 {
            tick(&mut f,map,&mut w,&mut p,&mut s,1./hz as f32)?;
            sp.advance(1./hz as f32,map,&mut sw,&mut single,&[])?;
            ensure!(shared(&f)==shared(&sp),"Flora shared clock/waves differ {hz}Hz dead{dead:?} tick{updates}");
            updates+=1;
            if updates==hz*2 {
                let saved=f.snapshot();let bodies=serde_json::to_value((&p,&s))?;
                for dt in [0.,-1.,f32::NAN] {tick(&mut f,map,&mut w,&mut p,&mut s,dt)?;}
                ensure!(f.snapshot()==saved && serde_json::to_value((&p,&s))?==bodies,"Paused Flora changed");
                let bytes=crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
                f.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map)?;
                ensure!(f.snapshot()==saved,"Flora snapshot changed");
                let mut old=saved.clone();old.as_object_mut().unwrap().remove("coop");
                let mut restored=Flora::load(a,map)?;restored.restore(&old,map)?;
                ensure!(shared(&restored)==shared(&f),"Old Flora save lost progress");
                let mut bad=saved;bad["coop"]["scene_owner"]=serde_json::json!(2);
                let valid=f.snapshot();ensure!(f.restore(&bad,map).is_err() && f.snapshot()==valid,"Invalid paired owner mutated Flora");
            }
        }
        ensure!((updates as i32-hz as i32*10).abs()<=1 && f.saved.wave==4,"Wrong paired countdown/wave count");
        for (i,stats) in s.iter_mut().enumerate() {
            stats.damage(10000.);
            ensure!(stats.sanity()==if dead==Some(i) {0.} else if i==0 {10.} else {5.},"Countdown healed/revived or failed to protect slot{i}");
        }
        ensure!(f.update(&mut w,&p[0],Vec3::X,false).transition==Some(EXIT.destination())
            && f.update(&mut w,&p[0],Vec3::X,false).transition.is_none(),"Flora departure missing/repeated");
        f.transition_failed(&EXIT.destination());
        ensure!(f.update(&mut w,&p[0],Vec3::X,false).transition.is_none(),"Flora retry was immediate");
        for _ in 0..11 {f.advance(0.1,map,&mut w,&mut single,&[])?;}
        ensure!(f.update(&mut w,&single,Vec3::X,false).transition==Some(EXIT.destination()),"Flora retry froze");
        println!("PASS Flora paired {hz}Hz dead={dead:?}: single-player wave/clock parity, ten seconds, pause/save, personal health floor and exit retry");
    }}
    let (mut f,mut w)=setup(a,map)?;let (mut p,mut s)=players(&w)?;
    f.phase(Phase::Countdown);for stats in &mut s {stats.set_health(0.)?;}
    tick(&mut f,map,&mut w,&mut p,&mut s,0.)?;let saved=f.snapshot();
    for _ in 0..120 {tick(&mut f,map,&mut w,&mut p,&mut s,FIXED_DT)?;}
    ensure!(f.snapshot()==saved && s.iter().all(|s|!s.alive()),"Dead team advanced/revived during departure");
    println!("PASS Flora dead team freezes waves/departure without resurrection");Ok(())
}
fn spawn_visibility(a:&mut Assets,map:&Bsp)->Result<()> {
    let (mut base,world)=setup(a,map)?;let (p,_)=players(&world)?;let eye=p[0].feet+Vec3::Z*32.;
    base.phase(Phase::Countdown);
    base.spawn_wave(&world,eye);
    let first=base.saved.ants.iter().find(|s|s.group==1).context("No native hidden brave spawn")?.ant.feet;
    let node=base.data.braves.iter().find(|n|world.sweep(n.translation,eye,Vec3::ZERO).fraction<1.).unwrap();
    let other=world.actor_footing(node.translation,crate::collision::PLAYER_CENTER,crate::collision::PLAYER_HALF,256.)
        .context("No supported observer beside brave chain")?;
    ensure!(world.body_clear(other) && world.sweep(node.translation,other+Vec3::Z*32.,Vec3::ZERO).fraction>=1.,"Observer cannot see candidate spawn");
    for observer in 0..2 {for dead in [false,true] {
        let (mut f,mut w)=setup(a,map)?;let (mut p,mut s)=players(&w)?;
        p[observer]=Player::new(other);if dead {s[observer].set_health(0.)?;}
        f.phase(Phase::Countdown);tick(&mut f,map,&mut w,&mut p,&mut s,FIXED_DT)?;
        let first_spawned=f.saved.ants.iter().any(|s|s.group==1 && s.ant.feet.distance(first)<0.01);
        ensure!(first_spawned==dead,"Wave appeared in living observer's view or used a dead observer");
        for soldier in f.saved.ants.iter().filter(|s|s.group>0) {
            let chain=if soldier.group==1 {&f.data.braves} else {&f.data.killers};
            let node=chain.iter().find(|n|w.actor_footing(n.translation,Vec3::Z*36.,Vec3::splat(36.),256.)
                .is_some_and(|at|at.distance(soldier.ant.feet)<0.01)).context("Wave soldier outside native chain")?;
            ensure!(p.iter().zip(&s).filter(|(_,s)|s.alive()).all(|(p,_)|
                w.sweep(node.translation,p.feet+Vec3::Z*32.,Vec3::ZERO).fraction<1.),"Spawn visible to a living player");
        }
        println!("PASS Flora wave observer{observer} dead={dead}: both living views respected and dead views ignored");
    }}
    let (mut f,world)=setup(a,map)?;
    f.phase(Phase::Countdown);
    for _ in 0..3 {f.spawn_wave_for(&world,&[eye,eye]);}
    ensure!(f.saved.ants.iter().filter(|s|s.group==1 && s.ant.health>0.).count()==3,"Native brave cap fixture incomplete");
    let saved=f.snapshot();let count=f.saved.ants.len();f.spawn_wave_for(&world,&[eye,eye]);
    ensure!(f.saved.wave==4 && f.saved.ants.len()==count,"Shared live-brave cap spawned extra Ants");
    f.restore(&saved,map)?;let id=f.saved.ants.iter().find(|s|s.group==1).unwrap().ant.id;
    f.hit(Hit{id:BASE+id,damage:1000.,kind:crate::combat::DamageKind::Knife,knockback:Vec3::ZERO});
    f.spawn_wave_for(&world,&[eye,eye]);
    ensure!(f.saved.wave==4 && f.saved.ants.len()>count && f.saved.ants.iter().filter(|s|s.group==1 && s.ant.health>0.).count()==3,
        "Ordinary kill failed to free native brave slot");
    println!("PASS Flora shared brave cap, ordinary kill and final bounded wave");Ok(())
}
fn targets(w:&World,from:Vec3)->Option<[PlayerTarget;2]> {
    let mut eyes=Vec::new();
    for distance in [160.,320.,480.] {for angle in 0..16 {
        let angle=angle as f32*std::f32::consts::TAU/16.;let eye=from+vec3(angle.cos()*distance,angle.sin()*distance,0.);
        let sight=w.sweep(from,eye,Vec3::splat(0.5));
        if !sight.start_solid && sight.fraction>=1. && w.body_clear(eye-Vec3::Z*EYE_HEIGHT) {eyes.push(eye);}
    }}
    let pair=eyes.iter().flat_map(|a|eyes.iter().map(move|b|(*a,*b)))
        .find(|(a,b)|a.distance(*b)>240. && from.distance(*b)>from.distance(*a)+80.)?;
    Some([pair.0,pair.1].map(|eye|PlayerTarget{eye,alive:true,hidden:false}))
}
fn combat_fixture(a:&mut Assets,map:&Bsp)->Result<(Flora,World,usize,[PlayerTarget;2])> {
    let (mut f,w)=setup(a,map)?;
    let (index,players)=f.saved.ants.iter().enumerate().find_map(|(i,s)|targets(&w,s.ant.target(0).center).map(|p|(i,p)))
        .context("No native Flora combat fixture")?;
    for s in &mut f.saved.ants {s.ant.enabled=false;}
    f.saved.ants[index].ant.enabled=true;f.saved.ants[index].ant.script_wait=false;
    Ok((f,w,index,players))
}
fn combat(a:&mut Assets,map:&Bsp)->Result<()> {
    for near in 0..2 {for excluded in 0..3 {
        let (mut f,w,index,mut p)=combat_fixture(a,map)?;if near==1 {p.swap(0,1);}
        p[near].alive=excluded!=1;p[near].hidden=excluded==2;
        let saved=f.snapshot();let paused=f.coop_combat(0.,&w,p);
        ensure!(f.snapshot()==saved && paused.players.iter().all(|p|p.damage==0.),"Paused Flora Ant advanced");
        f.coop_combat(FIXED_DT,&w,p);
        ensure!(f.saved.ants[index].ant.opponents.selected_player==Some(if excluded==0 {near}else{1-near}),"Flora Ant chose wrong visible living target");
        println!("PASS Flora nearest target near{near} excluded{excluded}");
    }}
    for victim in 0..2 {
        let (mut f,w,index,p)=combat_fixture(a,map)?;let center=p[victim].eye-Vec3::Z*20.;
        f.saved.ants[index].ant.shots.push(crate::ant::Bullet{at:center-Vec3::X*20.,direction:Vec3::X,age:0.,grenade:false});
        let out=f.coop_combat(FIXED_DT,&w,p);
        ensure!(out.players[victim].damage>0. && out.players[1-victim].damage==0. && out.damage==0.,"Flora projectile hit wrong player");
        let saved=f.snapshot();let mut restored=Flora::load(a,map)?;restored.restore(&saved,map)?;
        ensure!(restored.snapshot()==saved,"Flora projectile state lost in snapshot");
        println!("PASS Flora native projectile victim{victim}: personal feedback and saved actor state");
    }
    Ok(())
}
fn owner(s:&mut Stage)->&mut Flora {s.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Flora>()).unwrap()}
fn scenes(a:&mut Assets,map:&Bsp)->Result<()> {
    let visit=Visit{epoch:1,map:"centipede1".into(),entry:Some("centipede1_start1".into()),difficulty:Default::default(),combat_preview:false};
    for first in PlayerId::ALL {for skip in [false,true] {for dead in [false,true] {
        let mut w=World::from_bsp(map)?;let mut stage=Stage::load(a,map,&mut w,&visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
        let mut roster=Roster::new(&w,eye,yaw,Stats::for_level("centipede1",visit.entry.as_deref()))?;
        roster.players[0].stats.set_health(54.)?;roster.players[1].stats.set_health(5.)?;
        roster.players[1].stats.spend_will(17.);
        if dead {roster.players[first.partner().index()].stats.set_health(0.)?;}
        let resources=roster.players.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
        roster.players[first.index()].player=Player::spawn(&w,vec3(-280.,4700.,-600.)).context("Flora approach blocked")?;
        stage.before_move(&mut roster,map,&mut w,&Default::default(),0.)?;
        for _ in 0..600 {
            let before=roster.players.each_ref().map(|b|b.player.feet);
            roster.players[first.index()].player.tick(&w,Controls{wish:Vec2::Y,..Default::default()});
            stage.contacts(&mut roster,&w,before)?;if stage.owner.is_some(){break;}
        }
        ensure!(stage.owner==Some(first) && owner(&mut stage).saved.phase==Phase::Rush,"Wrong Flora ambush owner");
        let actor=roster.players[first.index()].player.feet;
        let mut waiting=roster.players[first.partner().index()].player.clone();waiting.release_rope();let waiting=serde_json::to_value(waiting)?;
        for n in 0..120*120 {
            if n==24 || skip && n==252 {
                let skipped=stage.interactions.skip_cinematic(map,&mut w,&mut roster.players[first.index()].player,&mut stage.story)?;
                ensure!(skipped==(n==252),"Flora rush skip boundary changed");
            }
            if n==120 {
                let saved=stage.snapshot();let bodies=serde_json::to_value(&roster.players)?;
                stage.before_move(&mut roster,map,&mut w,&Default::default(),0.)?;
                let bytes=crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
                stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,&mut w)?;
                ensure!(serde_json::to_value(stage.snapshot())?==serde_json::to_value(saved)? && serde_json::to_value(&roster.players)?==bodies,"Paused/restored Flora scene changed");
            }
            stage.scene_step(&mut roster,map,&mut w,&Default::default())?;
            ensure!(owner(&mut stage).saved.alice.translation==actor && owner(&mut stage).saved.coop.as_ref().unwrap().scene_owner==Some(first.index()),"Scene Alice pose belongs to wrong player");
            if stage.owner.is_none(){break;}
            ensure!(serde_json::to_value(&roster.players[first.partner().index()].player)?==waiting,"Flora scene moved partner");
        }
        ensure!(!stage.busy() && owner(&mut stage).saved.phase==Phase::Countdown && owner(&mut stage).saved.wave==0
            && owner(&mut stage).saved.time==0.,
            "Flora did not reunite before choosing paired wave spawns");
        for (b,saved) in roster.players.iter().zip(&resources) {
            ensure!(serde_json::to_value(&b.stats)?==*saved && w.body_clear(b.player.feet),"Flora reunion changed resources/clearance");b.player.validate_world(&w)?;
        }
        if !dead {ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Flora reunion overlaps");}
        let mut updates=0_i32;
        while stage.transition.is_none() && updates<120*11 {
            stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;updates+=1;
            if stage.owner.is_some(){stage.scene_step(&mut roster,map,&mut w,&Default::default())?;}
        }
        ensure!((updates-1200).abs()<=1 && stage.transition.as_ref().is_some_and(|e|EXIT.matches(e)) && owner(&mut stage).saved.wave==4,
            "Paired ambush exit/countdown failed: {updates} updates");
        ensure!(roster.players.iter().zip(resources).all(|(b,s)|serde_json::to_value(&b.stats).unwrap()==s),"Departure changed personal resources");
        println!("PASS Flora scene {first:?} skip={skip} dead_partner={dead}: native trigger, actor pose, frozen partner, pause/save, reunion and timed departure");
    }}}
    Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/centipede1.bsp")?)?;
    clocks(a,&map)?;spawn_visibility(a,&map)?;combat(a,&map)?;scenes(a,&map)?;
    println!("PASS Flora 31 native co-op component cases; full paired traversal remains unverified");Ok(())
}
