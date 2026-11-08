//! Real-map component contracts, separate from full two-client traversal.
use super::*;
use crate::{level::CoopPlayer, movement::{Controls, EYE_HEIGHT, FIXED_DT},
    coop::{protocol::Visit, stage::Stage, world::Roster, PlayerId}};
type Pair = [(Player, Stats); 2];
fn combat(o: &mut Centipede, w: &World, p: &mut Pair, dt: f32) -> Feedback {
    o.coop_combat_roster(dt,w,&mut p.each_mut().map(|(player,stats)|CoopPlayer{player,stats}))
}
fn advance(o: &mut Centipede, map: &Bsp, w: &mut World, p: &mut Pair, dt: f32) -> Result<()> {
    o.coop_advance_roster(dt,map,w,&mut p.each_mut().map(|(player,stats)|CoopPlayer{player,stats}),&[])
}
pub(super) fn fixture(a: &mut Assets, map: &Bsp) -> Result<(Centipede,World,Pair)> {
    let mut o=Centipede::load(a,map)?;o.phase(Phase::Fight);o.rebuild(map)?;
    let mut w=World::from_bsp(map)?;w.set_dynamic(o.colliders());
    let origin=o.saved.battle.pose.translation;
    let mut positions=Vec::new();
    for distance in [280.,550.] {
        let p=(0..32).find_map(|i| {
            let angle=i as f32*std::f32::consts::TAU/32.;
            let at=origin+vec3(angle.cos(),angle.sin(),0.)*distance;
            let p=Player::spawn(&w,at+Vec3::Z*(EYE_HEIGHT+12.))?;
            let sight=w.sweep(origin+Vec3::Z*96.,p.eye(),Vec3::splat(0.5));
            ((p.feet.z-origin.z).abs()<24. && !sight.start_solid && sight.fraction>=1.).then_some(p)
        }).context("No native Sanctum combat position")?;
        positions.push(p);
    }
    let mut p=[positions.remove(0),positions.remove(0)].map(|p|(p,Stats::for_level("centipede2",None)));
    o.prepare_pair(&mut p.each_mut().map(|(player,stats)|CoopPlayer{player,stats}));
    Ok((o,w,p))
}
fn attacks(a: &mut Assets,map:&Bsp)->Result<()> {
    for victim in 0..2 {
        let (mut o,w,mut p)=fixture(a,map)?;if victim==1 {p.reverse();}
        for excluded in 0..3 {
            p[victim].1.set_health(if excluded==2 {0.} else {100.})?;
            p[victim].1.invisible=if excluded==1 {10.} else {0.};
            combat(&mut o,&w,&mut p,FIXED_DT);
            ensure!(o.saved.battle.coop.as_ref().unwrap().target==Some(if excluded==0 {victim}else{1-victim}),"Centipede target ignored living/visible distance");
        }
        println!("PASS Sanctum target{victim}: nearest, invisible and dead filtering");
        let (mut o,w,mut p)=fixture(a,map)?;if victim==1 {p.reverse();}
        let target=p[victim].0.feet+Vec3::Z*32.;
        p[victim].1.invisible=10.; // Aim switches, but the existing projectile still strikes this body.
        o.saved.battle.shots.push(battle::Shot{at:target-Vec3::X*20.,start:target-Vec3::X*20.,velocity:Vec3::X*550.,age:0.,ended:None,fragment:false,normal:Vec3::ZERO});
        let hit=combat(&mut o,&w,&mut p,FIXED_DT);
        ensure!(hit.players[victim].damage==15. && hit.players[1-victim].damage==0. && hit.damage==0.,"Acid hit wrong player");
        ensure!(o.saved.battle.shots.len()==7,"Acid did not split into six fragments");
        println!("PASS Sanctum acid{victim}: personal collision and six fragments");

        let (mut o,w,mut p)=fixture(a,map)?;if victim==1 {p.reverse();}
        let delta=(p[victim].0.feet-o.saved.battle.pose.translation).with_z(0.).normalize_or_zero();
        let at=o.saved.battle.pose.translation+delta*180.;
        p[victim].0=Player::spawn(&w,at+Vec3::Z*(EYE_HEIGHT+12.)).context("No melee footing")?;
        o.saved.battle.pose.rotation=Quat::from_rotation_z(delta.y.atan2(delta.x));
        o.saved.battle.set(battle::Action::Crush);
        o.saved.battle.time=30.*o.data.boss().frame("attack_crush")-FIXED_DT*0.5;
        let hit=combat(&mut o,&w,&mut p,FIXED_DT);
        ensure!(hit.players[victim].damage==25. && hit.players[1-victim].damage==0. && hit.damage==0.,"Crush hit wrong player");
        ensure!(combat(&mut o,&w,&mut p,FIXED_DT).players[victim].damage==0.,"Crush repeated its hit frame");
        p[victim].1.set_health(0.)?;
        o.saved.battle.time=30.*o.data.boss().frame("attack_crush")-FIXED_DT*0.5;
        ensure!(combat(&mut o,&w,&mut p,FIXED_DT).players.iter().all(|p|p.damage==0.),"Crush hit dead player");
        println!("PASS Sanctum melee{victim}: personal hit, single cue and dead-body filter");

        let (mut o,w,mut p)=fixture(a,map)?;if victim==1 {p.reverse();}
        o.saved.battle.hits=4;o.saved.battle.stage=3;
        let delta=p[victim].0.feet-o.saved.battle.pose.translation;
        o.saved.battle.pose.rotation=Quat::from_rotation_z(delta.y.atan2(delta.x));
        o.saved.battle.set(battle::Action::Strike);
        o.saved.battle.time=o.data.boss().duration("attack_grab_strike");
        combat(&mut o,&w,&mut p,FIXED_DT);
        ensure!(o.coop_controlled(victim) && !o.coop_controlled(1-victim),"Native strike failed to capture only its target");
        ensure!(o.alice_performance().is_none(),"Co-op grab drew a duplicate blue Alice");
        let saved=o.snapshot();let mut restored=Centipede::load(a,map)?;restored.restore(&saved,map)?;
        let mut copy=p.clone();let partner=p[1-victim].0.feet;
        for _ in 0..150 {
            combat(&mut o,&w,&mut p,FIXED_DT);combat(&mut restored,&w,&mut copy,FIXED_DT);
            ensure!(o.snapshot()==restored.snapshot() && serde_json::to_value(&p)?==serde_json::to_value(&copy)?,"Saved grab diverged");
            ensure!(p[1-victim].0.feet==partner,"Grab moved partner");
        }
        ensure!(!o.coop_controlled(victim) && p[victim].0.velocity.length()>500. && p[1-victim].0.velocity==Vec3::ZERO,"Throw reached wrong body or retained lock");
        o.restore(&saved,map)?;p[victim].1.set_health(0.)?;combat(&mut o,&w,&mut p,FIXED_DT);
        ensure!(!o.coop_controlled(victim) && !o.saved.battle.grabbed,"Dead actor remained captured");
        o.restore(&saved,map)?;o.coop_respawned(victim);
        ensure!(!o.coop_controlled(victim),"Respawn retained boss lock");
        println!("PASS Sanctum grab{victim}: native strike, personal throw, saved continuation, death/respawn release");
    }
    Ok(())
}
fn clocks(a:&mut Assets,map:&Bsp)->Result<()> {
    for hz in [30,60,144] {for action in [battle::Action::Spit,battle::Action::Larvae] {
        let (mut o,w,mut p)=fixture(a,map)?;o.saved.battle.set(action);
        let saved=o.snapshot();let bodies=serde_json::to_value(&p)?;
        for dt in [0.,-1.,f32::NAN] {combat(&mut o,&w,&mut p,dt);}
        ensure!(o.snapshot()==saved && serde_json::to_value(&p)?==bodies,"Paused boss advanced");
        for _ in 0..hz {combat(&mut o,&w,&mut p,1./hz as f32);}
        ensure!(if action==battle::Action::Spit {!o.saved.battle.shots.is_empty()} else {o.saved.battle.larvae.len()==2},"Lost/duplicated attack cue");
        let saved=o.snapshot();o.restore(&saved,map)?;ensure!(o.snapshot()==saved,"Boss restore changed state");
        println!("PASS Sanctum {hz}Hz {action:?}: one clock, pause and save");
    }}
    Ok(())
}
fn pickups(a:&mut Assets,map:&Bsp)->Result<()> {
    for collector in 0..2 {
        let (mut o,mut w,mut p)=fixture(a,map)?;
        p[0].1.set_health(40.)?;p[1].1.set_health(40.)?;
        let at=o.personal_essence(Some(collector)).unwrap();
        p[collector].0=Player::spawn(&w,at+Vec3::Z*EYE_HEIGHT).context("No native essence footing")?;
        advance(&mut o,map,&mut w,&mut p,FIXED_DT)?;
        ensure!(p[collector].1.sanity()==90. && p[1-collector].1.sanity()==40.,"Shared healing resources");
        ensure!(o.personal_essence(Some(collector)).is_none() && o.personal_essence(Some(1-collector))==Some(at),"Pickup disappeared for other player");
        p[1-collector].0=p[collector].0.clone();advance(&mut o,map,&mut w,&mut p,FIXED_DT)?;
        ensure!(p[0].1.sanity()==90. && p[1].1.sanity()==90.,"Second player could not collect same essence");
        let saved=o.snapshot();let resources=serde_json::to_value(&p)?;
        o.restore(&saved,map)?;advance(&mut o,map,&mut w,&mut p,0.)?;
        ensure!(o.snapshot()==saved && serde_json::to_value(&p)?==resources,"Paused/restored pickup changed");
        let mut invalid=saved.clone();invalid["coop"]["essence"][0]=serde_json::json!(3);
        ensure!(o.restore(&invalid,map).is_err() && o.snapshot()==saved,"Bad pickup save accepted/mutated state");
        for _ in 0..1201 {advance(&mut o,map,&mut w,&mut p,FIXED_DT)?;}
        ensure!(o.personal_essence(Some(0)).is_some() && o.personal_essence(Some(1)).is_some(),"Personal healing did not recharge");
        println!("PASS Sanctum essence{collector}: personal resources, visibility, cooldown and validated save");
    }
    Ok(())
}
fn owner(s:&mut Stage)->&mut Centipede {s.interactions.levels.iter_mut().find_map(|l|l.ctl.downcast_mut::<Centipede>()).unwrap()}
fn progression(a:&mut Assets,map:&Bsp)->Result<()> {
    let visit=Visit{epoch:1,map:"centipede2".into(),entry:Some("centipede2_start1".into()),difficulty:Default::default(),combat_preview:false};
    for first in PlayerId::ALL {
        let (mut o,mut w,mut p)=fixture(a,map)?;
        p[0].1.invisible=10.;p[1].1.invisible=10.;
        for hit in 0..6 {
            o.saved.battle.set(battle::Action::Crush);
            let attack=Hit{id:BASE,damage:45.,kind:crate::combat::DamageKind::Knife,knockback:Vec3::ZERO};
            o.hit(attack);o.hit(attack);
            ensure!(o.saved.battle.hits==hit+1,"Two players duplicated weak-point reaction");
            for _ in 0..3000 {
                combat(&mut o,&w,&mut p,FIXED_DT);
                if matches!(o.saved.battle.action,battle::Action::Walk|battle::Action::Dead) {break;}
            }
            ensure!(o.saved.battle.stage==((hit+1)/2+1).min(3),"Wrong paired boss stage");
        }
        ensure!(o.saved.battle.action==battle::Action::Dead,"Six reactions did not finish death/thrashes");
        o.saved.battle.coop.as_mut().unwrap().target=Some(first.index());
        let mut stage=Stage::load(a,map,&mut w,&visit,false)?;
        *owner(&mut stage)=o;stage.owner=None;
        let mut roster=Roster::new(&w,p[0].0.eye(),0.,p[0].1.clone())?;
        for (body,(player,stats)) in roster.players.iter_mut().zip(p) {body.player=player;body.stats=stats;}
        let resources=roster.players.each_ref().map(|p|serde_json::to_value(&p.stats).unwrap());
        let partner=roster.players[first.partner().index()].player.feet;
        stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
        ensure!(stage.owner==Some(first) && owner(&mut stage).saved.phase==Phase::Drop,"Wrong defeat scene owner");
        for tick in 0..1500 {
            if tick==120 {
                let saved=stage.snapshot();let bodies=serde_json::to_value(&roster.players)?;
                stage.before_move(&mut roster,map,&mut w,&Default::default(),0.)?;
                let bytes=crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
                stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,&mut w)?;
                ensure!(serde_json::to_value(stage.snapshot())?==serde_json::to_value(saved)? && serde_json::to_value(&roster.players)?==bodies,"Paused/restored spike scene changed");
            }
            stage.scene_step(&mut roster,map,&mut w,&Default::default())?;
            if !stage.busy() {break;}
            ensure!(roster.players[first.partner().index()].player.feet==partner,"Spike scene moved partner");
        }
        ensure!(!stage.busy() && owner(&mut stage).saved.phase==Phase::Climb,"Spikes failed reunion");
        ensure!(owner(&mut stage).poses.iter().any(|(m,_,_)|*m==2),"No shared climbing geometry");
        ensure!(roster.players.iter().zip(&resources).all(|(p,s)|serde_json::to_value(&p.stats).unwrap()==*s),"Defeat changed personal resources");
        let goal=owner(&mut stage).data.points["alice_pos2"].translation;
        roster.players[first.index()].player=Player::spawn(&w,goal+Vec3::Z*EYE_HEIGHT).context("Mushroom scene footing blocked")?;
        for _ in 0..240 {
            let before=roster.players.each_ref().map(|p|p.player.feet);
            roster.players[first.index()].player.tick(&w,Controls{wish:Vec2::X,..Default::default()});
            stage.contacts(&mut roster,&w,before)?;
            if stage.owner.is_some() {break;}
        }
        ensure!(stage.owner==Some(first) && owner(&mut stage).saved.phase==Phase::Greeting,"Mushroom contact did not bind first arrival");
        let partner=roster.players[first.partner().index()].player.feet;
        let input=[(Controls{use_pressed:true,..Default::default()},Default::default());2];
        for _ in 0..12000 {
            stage.scene_step(&mut roster,map,&mut w,&input)?;
            ensure!(roster.players[first.partner().index()].player.feet==partner,"Mushroom scene moved partner");
            if stage.transition.is_some() {break;}
        }
        ensure!(stage.transition.as_ref().is_some_and(|e|EXIT.matches(e)) && owner(&mut stage).saved.phase==Phase::Done,"Mushroom departure failed");
        ensure!(roster.players.iter().zip(&resources).all(|(p,s)|serde_json::to_value(&p.stats).unwrap()==*s),"Mushroom changed personal resources");
        println!("PASS Sanctum progression {first:?}: six paired reactions, three stages, death/spikes, pause/save/reunion, actual mushroom contact and departure");
    }
    Ok(())
}
fn scenes(a:&mut Assets,map:&Bsp)->Result<()> {
    let visit=Visit{epoch:1,map:"centipede2".into(),entry:Some("centipede2_start1".into()),difficulty:Default::default(),combat_preview:false};
    for first in PlayerId::ALL {for skip in [false,true] {for dead in [false,true] {
        let mut w=World::from_bsp(map)?;let mut stage=Stage::load(a,map,&mut w,&visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
        let mut roster=Roster::new(&w,eye,yaw,Stats::for_level("centipede2",visit.entry.as_deref()))?;
        roster.players[0].stats.set_health(54.)?;roster.players[1].stats.set_health(5.)?;
        if dead {roster.players[first.partner().index()].stats.set_health(0.)?;}
        stage.owner=Some(first);
        let resources=roster.players.each_ref().map(|p|serde_json::to_value(&p.stats).unwrap());
        let waiting=roster.players[first.partner().index()].player.feet;
        for tick in 0..6000 {
            if skip && tick==24 {stage.interactions.skip_cinematic(map,&mut w,&mut roster.players[first.index()].player,&mut stage.story)?;}
            stage.scene_step(&mut roster,map,&mut w,&Default::default())?;
            if !stage.busy() {break;}
            ensure!(roster.players[first.partner().index()].player.feet==waiting,"Intro moved partner");
        }
        ensure!(!stage.busy() && owner(&mut stage).saved.phase==Phase::Slide,"Intro failed reunion");
        ensure!(roster.players.iter().zip(&resources).all(|(p,s)|serde_json::to_value(&p.stats).unwrap()==*s),"Intro changed resources");
        if !dead {ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Slide reunion overlap");}
        stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
        ensure!(roster.players.iter().all(|p|p.stats.minimum_sanity==30.),"Both slide health floors missing");
        println!("PASS Sanctum intro {first:?} skip={skip} dead={dead}: frozen partner, reunion and personal health floor");
    }}}
    Ok(())
}
fn slide(a:&mut Assets,map:&Bsp)->Result<()> {
    for first in 0..2 {
        let (mut o,mut w,mut p)=fixture(a,map)?;
        o.phase(Phase::Slide);o.saved.coop=None;
        let at=o.data.points["alice_slide_pos1"].translation;
        for (player,stats) in &mut p {
            *player=Player::spawn(&w,at+Vec3::Z*EYE_HEIGHT).context("Slide fixture blocked")?;
            stats.set_health(20.)?;
        }
        advance(&mut o,map,&mut w,&mut p,FIXED_DT)?;
        ensure!(p.iter().all(|(_,s)|s.minimum_sanity==30.),"Missing personal slide protection");
        let center=o.arena_trigger.interior_point().context("No arena trigger interior")?;
        p[first].0=Player::new(center-crate::collision::PLAYER_CENTER);
        ensure!(w.body_clear(p[first].0.feet),"Arena trigger fixture blocked");
        advance(&mut o,map,&mut w,&mut p,FIXED_DT)?;
        ensure!(o.saved.phase==Phase::Fight && p[first].1.minimum_sanity==0. && p[1-first].1.minimum_sanity==30.,"First arrival removed partner's slide protection");
        p[first].1.damage(10000.);p[1-first].1.damage(10000.);
        ensure!(!p[first].1.alive() && p[1-first].1.sanity()==20.,"Slide floor healed, revived or protected wrong player");
        p[1-first].0=p[first].0.clone();advance(&mut o,map,&mut w,&mut p,FIXED_DT)?;
        ensure!(p[1-first].1.minimum_sanity==0.,"Later arrival kept slide protection");
        p[1-first].1.damage(10000.);
        let saved=o.snapshot();advance(&mut o,map,&mut w,&mut p,FIXED_DT)?;
        ensure!(o.snapshot()==saved && p.iter().all(|(_,s)|!s.alive()),"Dead team advanced or resurrected");
        println!("PASS Sanctum slide{first}: personal arrival protection, low-health floor and dead-team freeze");
    }
    Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/centipede2.bsp")?)?;
    attacks(a,&map)?;clocks(a,&map)?;pickups(a,&map)?;scenes(a,&map)?;slide(a,&map)?;progression(a,&map)?;
    println!("PASS Sanctum native co-op components; full paired route remains open");Ok(())
}
