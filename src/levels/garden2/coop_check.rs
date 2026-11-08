//! Native component checks. Positions/encounters are staged, never route proof.
use super::*;
use crate::{collision::{PLAYER_CENTER, PLAYER_HALF}, combat::PlayerTarget,
    coop::{protocol::Visit,stage::Stage,world::Roster,PlayerId}, inventory::Stats,
    level::CoopPlayer, movement::{Controls,FIXED_DT}};

fn free(a: &mut Assets, map: &Bsp) -> Result<(Garden,World)> {
    let mut g = Garden::load(a,map)?;
    g.upgraded();
    g.saved.legacy = false;
    let mut world = World::from_bsp(map)?;
    world.set_dynamic(g.colliders());
    Ok((g,world))
}
fn standing(g: &Garden, world: &World) -> Result<Vec<(Vec2,Vec3)>> {
    let object = g.objects.iter().find(|o|o.model==18).unwrap();
    let pose = object.pose.unwrap();
    let mut points = Vec::new();
    for x in (-96..=96).step_by(16) {
        for y in (-96..=96).step_by(16) {
            let top = pose.translation + pose.rotation*vec3(x as f32,y as f32,256.);
            let bottom = top-Vec3::Z*512.;
            let trace = object.collider.trace(top+PLAYER_CENTER,bottom+PLAYER_CENTER,PLAYER_HALF);
            let feet = top.lerp(bottom,trace.fraction);
            if !trace.start_solid && trace.fraction<1. && trace.normal.z>0.65 && world.body_clear(feet) {
                let local = (pose.rotation.inverse()*(feet-pose.translation)).truncate();
                points.push((local,feet));
            }
        }
    }
    ensure!(points.len()>1,"No separated native fulcrum positions");
    Ok(points)
}
fn riders(g: &Garden, world: &World, balanced: bool) -> Result<[Player;2]> {
    let points = standing(g,world)?;
    let pair = points.iter().flat_map(|a|points.iter().map(move |b|(*a,*b)))
        .filter(|(a,b)|a.1.distance(b.1)>=48.)
        .min_by(|(a,b),(c,d)| {
            let cost = |a:(Vec2,Vec3),b:(Vec2,Vec3)| if balanced {
                (a.0+b.0).length_squared() + (a.0.length_squared()+b.0.length_squared())*0.0001
            } else { a.0.distance_squared(vec2(64.,-32.))+b.0.distance_squared(vec2(64.,32.)) };
            cost(*a,*b).total_cmp(&cost(*c,*d))
        }).context("Fulcrum cannot fit a separated pair")?;
    if balanced { ensure!((pair.0.0+pair.1.0).length()<0.1,"No balanced native fulcrum pair"); }
    Ok([Player::new(pair.0.1),Player::new(pair.1.1)])
}
fn platform(a: &mut Assets, map: &Bsp) -> Result<()> {
    for hz in [30,60,144] {
        let (mut g,mut world)=free(a,map)?;
        let mut pair=riders(&g,&world,false)?;
        let before=pair.each_ref().map(|p|p.feet);
        for tick in 0..hz*2 {
            g.coop_advance(1./hz as f32,map,&mut world,&mut pair.each_mut(),&[])?;
            for p in &pair {
                ensure!(world.body_clear(p.feet),"Fulcrum embedded a rider at {hz}Hz tick{tick}");
                p.validate_world(&world)?;
            }
            ensure!(pair[0].feet.distance(pair[1].feet)>=35.95,"Fulcrum overlapped riders");
            if tick==hz {
                let snap=g.snapshot();
                let bodies=serde_json::to_value(&pair)?;
                g.coop_advance(0.,map,&mut world,&mut pair.each_mut(),&[])?;
                ensure!(g.snapshot()==snap && serde_json::to_value(&pair)?==bodies,"Paused fulcrum moved");
                let poses=g.transforms();
                let mut restored=Garden::load(a,map)?;
                restored.restore(&snap,map)?;
                ensure!(restored.snapshot()==snap && restored.transforms()==poses,"Saved fulcrum changed");
                g=restored;
                world.set_dynamic(g.colliders());
            }
        }
        ensure!(g.saved.fulcrum.length()>0.1 && g.saved.fulcrum.abs().max_element()<=12.,"Pair did not tilt fulcrum");
        ensure!(pair.iter().zip(before).all(|(p,old)|p.feet.distance(old)>0.1),"Fulcrum failed to carry both riders");
        println!("PASS Herbaceous paired fulcrum {hz}Hz: both carried, clear, pause/save, tilt {:?}",g.saved.fulcrum);
    }
    let (mut g,mut world)=free(a,map)?;
    let mut pair=riders(&g,&world,true)?;
    let before=pair.each_ref().map(|p|p.feet);
    for _ in 0..120 { g.coop_advance(FIXED_DT,map,&mut world,&mut pair.each_mut(),&[])?; }
    ensure!(g.saved.fulcrum.length()<0.01 && before==pair.each_ref().map(|p|p.feet),"Opposite riders did not balance");
    println!("PASS Herbaceous opposite riders balance the native fulcrum");
    for dead in 0..2 {
        let (mut g,mut world)=free(a,map)?;
        let mut pair=riders(&g,&world,false)?;
        let mut stats:[Stats;2]=std::array::from_fn(|_|Stats::for_level("garden2",None));
        stats[dead].damage(10000.);
        let dead_before=serde_json::to_value(&pair[dead])?;
        let mut single=Garden::load(a,map)?;
        single.restore(&g.snapshot(),map)?;
        let mut single_world=World::from_bsp(map)?;
        single_world.set_dynamic(single.colliders());
        let mut live=pair[1-dead].clone();
        for _ in 0..120 {
            let [p0,p1]=pair.each_mut();
            let [s0,s1]=stats.each_mut();
            g.coop_advance_roster(FIXED_DT,map,&mut world,&mut [
                CoopPlayer {player:p0,stats:s0},CoopPlayer {player:p1,stats:s1}],&[])?;
            single.advance(FIXED_DT,map,&mut single_world,&mut live,&[])?;
        }
        ensure!(g.saved.fulcrum.distance(single.saved.fulcrum)<0.001
            && pair[1-dead].feet.distance(live.feet)<0.1,"Living rider differs from single-player fulcrum");
        ensure!(serde_json::to_value(&pair[dead])?==dead_before,"Dead partner was carried");
        println!("PASS Herbaceous dead slot{dead}: survivor matches single-player fulcrum, dead body unchanged");
    }
    Ok(())
}

fn targets(world: &World, from: Vec3) -> Result<[PlayerTarget;2]> {
    let mut eyes=Vec::new();
    for distance in [160.,320.,480.] {
        for angle in 0..16 {
            let angle=angle as f32*std::f32::consts::TAU/16.;
            let eye=from+vec3(angle.cos()*distance,angle.sin()*distance,0.);
            let sight=world.sweep(from,eye,Vec3::splat(0.5));
            if !sight.start_solid && sight.fraction>=1.
                && world.body_clear(eye-Vec3::Z*crate::movement::EYE_HEIGHT) { eyes.push(eye); }
        }
    }
    let pair=eyes.iter().flat_map(|a|eyes.iter().map(move|b|(*a,*b)))
        .find(|(a,b)|a.distance(*b)>240. && from.distance(*b)>from.distance(*a)+80.)
        .context("No separated visible target pair around native resident")?;
    Ok([pair.0,pair.1].map(|eye|PlayerTarget{eye,alive:true,hidden:false}))
}
fn combat(a: &mut Assets,map: &Bsp)->Result<()> {
    let (mut g,mut world)=free(a,map)?;
    g.spawn_ladies(1);
    let mut pair=riders(&g,&world,true)?;
    for _ in 0..60 { g.coop_advance(1./60.,map,&mut world,&mut pair.each_mut(),&[])?; }
    let delay=g.saved.ladies.iter().find(|l|l.id==599).unwrap().delay.context("Lady spawned twice as fast")?;
    ensure!((delay-1.1).abs()<0.001,"Wrong shared spawn clock {delay}");
    let snap=g.snapshot();
    g.coop_advance(0.,map,&mut world,&mut pair.each_mut(),&[])?;
    ensure!(snap==g.snapshot(),"Paused spawn clock advanced");
    for _ in 0..67 { g.coop_advance(1./60.,map,&mut world,&mut pair.each_mut(),&[])?; }
    ensure!(g.saved.ladies.iter().find(|l|l.id==599).unwrap().enabled,"Delayed Lady never spawned");
    let count=g.saved.ladies.len();
    g.spawn_ladies(1);
    ensure!(g.saved.ladies.len()==count && g.saved.ladies.iter().find(|l|l.id==599).unwrap().delay.is_none(),"Lady respawned");
    println!("PASS Herbaceous delayed Lady spawn uses one shared clock and pauses");
    let (mut ambush,ambush_world)=free(a,map)?;
    ambush.event("Ant_Ambush2").context("Missing paired native ambush")?;
    let origin=ambush.saved.ants.iter().find(|ant|ant.id==139).unwrap().feet;
    let unseen=[PlayerTarget {eye:vec3(1502.,1048.,-1028.),alive:true,hidden:true};2];
    for tick in 0..120*9 {
        ambush.coop_combat(FIXED_DT,&ambush_world,unseen);
        if tick==60 {
            let saved=ambush.snapshot();
            ambush.coop_combat(0.,&ambush_world,unseen);
            ensure!(ambush.snapshot()==saved,"Paused paired ambush advanced");
            let mut restored=Garden::load(a,map)?;
            restored.restore(&saved,map)?;
            ensure!(restored.snapshot()==saved,"Paired ambush restore diverged");
            ambush=restored;
        }
    }
    let ant=ambush.saved.ants.iter().find(|ant|ant.id==139).unwrap();
    ensure!(ambush.saved.under_ambush==Some(8.) && ant.enabled && !ant.script_wait
        && ant.feet.distance(origin)>80.,"Paired native ambush failed to complete");
    let saved=ambush.snapshot();
    ambush.event("Ant_Ambush2");
    ensure!(ambush.snapshot()==saved,"Paired native ambush replayed");
    println!("PASS Herbaceous paired Corporal ambush clock, pause/save and one-shot release");
    for lady in [false,true] {
        for near in 0..2 {
            for excluded in [0,1,2] { // nearest eligible, nearest dead, nearest hidden
                let (mut g,world)=free(a,map)?;
                for ant in &mut g.saved.ants { ant.enabled=false; }
                let (index,mut players)=if lady {
                    g.saved.ladies.iter().enumerate().find_map(|(i,l)|targets(&world,l.actor.target(0).center).ok().map(|p|(i,p)))
                } else {
                    g.saved.ants.iter().enumerate().find_map(|(i,ant)|targets(&world,ant.target(0).center).ok().map(|p|(i,p)))
                }.context("No native combat target fixture")?;
                if near==1 { players.swap(0,1); }
                players[near].alive=excluded!=1;
                players[near].hidden=excluded==2;
                if lady {g.saved.ladies[index].enabled=true;} else {g.saved.ants[index].enabled=true;}
                let before=g.snapshot();
                let paused=g.coop_combat(0.,&world,players);
                ensure!(g.snapshot()==before && paused.players.iter().all(|p|p.damage==0.),"Paused paired enemy advanced");
                g.coop_combat(FIXED_DT,&world,players);
                let selected=if lady {g.saved.ladies[index].actor.opponents.selected_player}
                    else {g.saved.ants[index].opponents.selected_player};
                ensure!(selected==Some(if excluded==0 {near}else{1-near}),
                    "Native target selection failed: lady{lady} near{near} excluded{excluded} got{selected:?}");
                println!("PASS Herbaceous native target lady={lady} near={near} excluded={excluded}");
            }
        }
    }
    // A live native Ant projectile enters each player's actual collision body.
    // This staged impact verifies feedback routing, without changing health or
    // damage in the production controller to make a route pass.
    for lady in [false,true] {
      for victim in 0..2 {
        let (mut g,world)=free(a,map)?;
        let (index,players)=if lady {g.saved.ladies.iter().enumerate()
            .find_map(|(i,l)|targets(&world,l.actor.target(0).center).ok().map(|p|(i,p)))} else {g.saved.ants.iter().enumerate()
            .find_map(|(i,ant)|targets(&world,ant.target(0).center).ok().map(|p|(i,p)))
        }
            .context("No native projectile fixture")?;
        for ant in &mut g.saved.ants { ant.enabled=false; }
        let center=players[victim].eye-Vec3::Z*20.;
        if lady {
            let actor=&mut g.saved.ladies[index];
            actor.enabled=true;
            actor.actor.acorns.push(serde_json::from_value(serde_json::json!({
                "position":center-Vec3::X*24.,"velocity":Vec3::X*600.,"age":0.
            }))?);
        } else {
            let ant=&mut g.saved.ants[index];
            ant.enabled=true;
            ant.shots.push(crate::ant::Bullet {at:center-Vec3::X*20.,direction:Vec3::X,age:0.,grenade:false});
        }
        let feedback=g.coop_combat(FIXED_DT,&world,players);
        ensure!(feedback.players[victim].damage>0. && feedback.players[1-victim].damage==0.
            && feedback.damage==0.,"Native hit was not personal for lady{lady} slot{victim}");
        println!("PASS Herbaceous native projectile lady={lady} damages only slot{victim}");
        if lady {
            let bursts=&g.saved.ladies[index].actor.bursts;
            ensure!(bursts.len()==1,"Native Lady impact omitted its visual burst");
            let expected=serde_json::to_value(bursts)?;
            let wire=crate::coop::codec::encode(&serde_json::to_vec(&g.snapshot())?)?;
            let mut restored=Garden::load(a,map)?;
            restored.restore(&serde_json::from_slice(&crate::coop::codec::decode(&wire)?)?,map)?;
            ensure!(serde_json::to_value(&restored.saved.ladies[index].actor.bursts)?==expected,
                "Native Lady burst lost during guest snapshot restore");
            println!("PASS Herbaceous native Lady impact slot{victim}: visual burst survives guest snapshot restore");
        }
      }
    }
    Ok(())
}

fn stage_owner(stage: &mut Stage)->&mut Garden {
    stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Garden>()).unwrap()
}
fn scenes(a: &mut Assets,map: &Bsp)->Result<()> {
    let visit=Visit {epoch:1,map:"garden2".into(),entry:None,difficulty:Default::default(),combat_preview:false};
    for kind in [Kind::Arrival,Kind::North,Kind::South,Kind::Second,Kind::Cat] {
        for first in PlayerId::ALL {
            if kind==Kind::Arrival && first==PlayerId::Red {continue;}
            for skip in [false,true] {
                let mut world=World::from_bsp(map)?;
                let mut stage=Stage::load(a,map,&mut world,&visit,false)?;
                let (eye,yaw)=crate::interaction::spawn(map,None);
                let mut roster=Roster::new(&world,eye,yaw,Stats::for_level("garden2",None))?;
                roster.players[1].stats.spend_will(17.);
                let resources=roster.players.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
                if kind!=Kind::Arrival {
                    let g=stage_owner(&mut stage);
                    g.upgraded();
                    g.saved.legacy=false;
                    stage.owner=None;
                    stage.interactions.sync_coop(&mut world);
                    let start=check::approach(kind);
                    let feet=world.actor_footing(start,PLAYER_CENTER,PLAYER_HALF,200.).context("Herbaceous staged approach unsupported")?;
                    let p=&mut roster.players[first.index()].player;
                    *p=Player::new(feet);
                    p.grounded=true;
                    let trigger=match kind {Kind::North=>130,Kind::South=>129,Kind::Second=>114,Kind::Cat=>52,_=>unreachable!()};
                    let to=data::at(&map.entities[trigger]).translation;
                    for _ in 0..240 {
                        let previous=roster.players.each_ref().map(|b|b.player.feet);
                        let p=&mut roster.players[first.index()].player;
                        p.tick(&world,Controls{wish:(to-p.feet).truncate().normalize_or_zero(),..Default::default()});
                        stage.contacts(&mut roster,&world,previous)?;
                        if stage.owner.is_some(){break;}
                    }
                }
                ensure!(stage.owner==Some(first) && stage_owner(&mut stage).scene_id()==Some(kind.id()),
                    "Wrong native scene owner/kind {kind:?} {first:?}");
                let mut waiting=roster.players[first.partner().index()].player.clone();
                waiting.release_rope();
                let waiting=serde_json::to_value(waiting)?;
                let mut restored=false;
                for tick in 0..120*180 {
                    if skip && tick==24 {stage.interactions.skip_cinematic(map,&mut world,
                        &mut roster.players[first.index()].player,&mut stage.story)?;}
                    if tick==12 {
                        let state=stage.snapshot();
                        let before=serde_json::to_value(&state)?;
                        let bodies=serde_json::to_value(&roster.players)?;
                        stage.before_move(&mut roster,map,&mut world,&Default::default(),0.)?;
                        ensure!(serde_json::to_value(stage.snapshot())?==before && serde_json::to_value(&roster.players)?==bodies,"Paused Herbaceous scene changed");
                        let encoded=crate::coop::codec::encode(&serde_json::to_vec(&state)?)?;
                        stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&encoded)?)?,map,&mut world)?;
                        ensure!(serde_json::to_value(stage.snapshot())?==before,"Restored Herbaceous scene changed");
                        restored=true;
                    }
                    stage.scene_step(&mut roster,map,&mut world,&Default::default())?;
                    if stage.owner.is_none(){break;}
                    ensure!(stage.owner==Some(first) && serde_json::to_value(&roster.players[first.partner().index()].player)?==waiting,
                        "Herbaceous scene moved waiting partner or changed owner");
                }
                ensure!(restored && stage.owner.is_none(),"Herbaceous scene did not finish {kind:?}");
                for (b,stats) in roster.players.iter().zip(resources) {
                    ensure!(world.body_clear(b.player.feet) && b.stats.alive() && serde_json::to_value(&b.stats)?==stats,"Herbaceous reunion unsafe or changed personal resources");
                    b.player.validate_world(&world)?;
                }
                ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Herbaceous reunion overlapped");
                println!("PASS Herbaceous {kind:?} {first:?} skip={skip}: native trigger, one actor, frozen partner, pause/save, safe reunion, personal resources");
            }
        }
    }
    Ok(())
}
fn bank_contact(a: &mut Assets,map: &Bsp)->Result<()> {
    let (_,world)=free(a,map)?;
    let before=[vec3(2257.344970703125,-2632.409423828125,-1980.719970703125),
        vec3(2220.603271484375,-2639.881591796875,-1978.78564453125)];
    let end=vec3(2254.731689453125,-2632.940673828125,-1980.5823974609375);
    let normal=vec3(0.04076966643333435,0.05775703117251396,0.9974978566169739);
    for approaching in 0..2 {
        let mut bodies=std::array::from_fn(|i| {
            let mut player=Player::new(if i==approaching {end} else {before[1]});
            player.grounded=true;player.ground_normal=normal;
            if i==approaching {player.velocity=vec3(-313.5879211425781,-63.73851776123047,16.507526397705078);}
            crate::coop::world::Body {player,stats:Stats::for_level("garden2",None),yaw:0.,pitch:0.,
                running:i==approaching,spawned_at:0,warped_at:0}
        });
        let previous=if approaching==0 {before} else {[before[1],before[0]]};
        let outward=(end-before[1]).truncate().normalize_or_zero();
        let along=outward.extend(-outward.dot(normal.truncate())/normal.z);
        let candidate=end+along*(crate::coop::separation::DISTANCE-(end-before[1]).truncate().length());
        let trace=world.body_trace(end,candidate);
        let ground=world.body_trace(candidate,candidate-Vec3::Z*2.);
        let depth=ground.normal.truncate().abs().dot(PLAYER_HALF.truncate())/ground.normal.z;
        let support=world.sweep(candidate+Vec3::Z,candidate-Vec3::Z*(2.+depth),Vec3::ZERO);
        println!("Herbaceous bank contact slot{approaching}: path={trace:?}, clear={}, ground={ground:?}, centre={support:?}",world.body_clear(candidate));
        let resources=bodies.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
        crate::coop::separation::resolve(&world,&mut bodies,previous);
        for tick in 0..240 {
            let delta=bodies[0].player.feet-bodies[1].player.feet;
            ensure!(delta.truncate().length()>=crate::coop::separation::DISTANCE-0.05,
                "Native bank contact overlapped at slot{approaching} tick{tick}: {delta:?}");
            ensure!(bodies[1-approaching].player.feet.distance(before[1])<0.01,"Bank contact shoved stationary partner");
            for (b,saved) in bodies.iter().zip(&resources) {
                ensure!(world.body_clear(b.player.feet) && serde_json::to_value(&b.stats)?==*saved,"Bank contact embedded a player or changed resources");
                b.player.validate_world(&world)?;
            }
            if tick==239 {break;}
            let previous=bodies.each_ref().map(|b|b.player.feet);
            let wish=(bodies[1-approaching].player.feet-bodies[approaching].player.feet).truncate().normalize_or_zero();
            bodies[approaching].player.tick(&world,Controls{wish,run:true,..Default::default()});
            crate::coop::separation::resolve(&world,&mut bodies,previous);
        }
        println!("PASS Herbaceous captured bank contact slot{approaching}: 240 clear separated ticks, stationary partner and personal resources preserved");
    }
    Ok(())
}
pub(super) fn check(a: &mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/garden2.bsp")?)?;
    let selected=std::env::var("LOOKING_GLASS_GARDEN2_COOP_CASE").ok();
    ensure!(selected.as_deref().is_none_or(|v|matches!(v,"platform"|"combat"|"scenes"|"contact")),"Invalid Herbaceous check selection");
    let checks:[(&str,fn(&mut Assets,&Bsp)->Result<()>);4]=[("platform",platform),("combat",combat),("scenes",scenes),("contact",bank_contact)];
    let mut failures=Vec::new();
    for (name,run) in checks {
        if selected.as_deref().is_some_and(|v|v!=name){continue;}
        if let Err(error)=run(a,&map) {println!("FAIL Herbaceous {name}: {error:#}");failures.push(format!("{name}: {error:#}"));}
    }
    ensure!(failures.is_empty(),"Herbaceous co-op failures: {}",failures.join("; "));
    println!("PASS Herbaceous staged co-op components; full traversal, network, recovery and rejoin remain unverified");
    Ok(())
}
