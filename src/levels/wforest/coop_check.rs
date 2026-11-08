//! Native staged components; these do not claim full campaign traversal.
use super::*;
use crate::{coop::{protocol::Visit,stage::Stage,world::Roster,PlayerId},level::CoopPlayer,movement::FIXED_DT};

fn setup(a:&mut Assets,map:&Bsp,returning:bool)->Result<(Forest,World,[Player;2],[Stats;2])> {
    let mut f=Forest::load(a,map,returning)?;let mut w=World::from_bsp(map)?;w.set_dynamic(f.colliders());
    let mut p=[Player::new(vec3(0.,0.,16000.)),Player::new(vec3(100.,0.,16000.))];
    let mut s=std::array::from_fn(|_|Stats::for_level("wforest",Some(if returning {"wforest_start2"} else {"wforest_start1"})));
    f.prepare_pair(&mut slots(&mut p,&mut s));
    Ok((f,w,p,s))
}
fn slots<'a>(p:&'a mut [Player;2],s:&'a mut [Stats;2])->[CoopPlayer<'a>;2] {
    let [p0,p1]=p.each_mut();let [s0,s1]=s.each_mut();
    [CoopPlayer{player:p0,stats:s0},CoopPlayer{player:p1,stats:s1}]
}
fn tick(f:&mut Forest,map:&Bsp,w:&mut World,p:&mut [Player;2],s:&mut [Stats;2],dt:f32)->Result<()> {
    f.coop_advance_roster(dt,map,w,&mut slots(p,s),&[])
}
fn shared(f:&Forest)->serde_json::Value {let mut v=f.snapshot();v.as_object_mut().unwrap().remove("coop");v}
fn clocks(a:&mut Assets,map:&Bsp)->Result<()> {
    for returning in [false,true] {for hz in [30,60,144] {
        let (mut f,mut w,mut p,mut s)=setup(a,map,returning)?;
        let mut sp=Forest::load(a,map,returning)?;let mut sw=World::from_bsp(map)?;sw.set_dynamic(sp.colliders());
        let mut player=p[0].clone();let mut stats=s[0].clone();sp.prepare_player(&mut stats,&mut player);
        for x in [&mut f,&mut sp] {
            if returning {
                x.event("Open_Humpty_Door");
                x.hit(Hit{id:RETURN_BASE,damage:100.,kind:DamageKind::EyeStaff,knockback:Vec3::ZERO});
            } else {x.saved.staff=true;x.saved.armed=true;x.saved.done[1]=true;x.saved.done[2]=true;}
        }
        for n in 0..hz*7 {
            tick(&mut f,map,&mut w,&mut p,&mut s,1./hz as f32)?;
            sp.advance(1./hz as f32,map,&mut sw,&mut player,&[])?;
            ensure!(shared(&f)==sp.snapshot() && f.transforms()==sp.transforms(),"Paired WForest clocks differ return={returning} {hz}Hz tick{n}");
            if n==hz {
                let saved=f.snapshot();let bodies=serde_json::to_value((&p,&s))?;
                for dt in [0.,-1.,f32::NAN] {tick(&mut f,map,&mut w,&mut p,&mut s,dt)?;}
                ensure!(f.snapshot()==saved && serde_json::to_value((&p,&s))?==bodies,"Paused paired WForest changed");
                let mut restored=Forest::load(a,map,returning)?;restored.restore(&saved,map)?;
                ensure!(restored.snapshot()==saved,"Saved paired movers changed");f=restored;
            }
        }
        ensure!(if returning {f.hedge_ready() && f.saved.secret==Some(6.)} else {f.first_ready()},"Paired gates did not finish");
        println!("PASS paired WForest return={returning} {hz}Hz: native mover clock parity, pause and save continuation");
    }}
    Ok(())
}
fn personal(a:&mut Assets,map:&Bsp)->Result<()> {
    for local in 0..2 {for dead in [false,true] {
        let (mut f,mut w,mut p,mut s)=setup(a,map,true)?;
        s[local].damage(50.);s[local].spend_will(50.);
        if dead {s[1-local].damage(1000.);}
        let other=serde_json::to_value(&s[1-local])?;
        p[local].feet=f.data.at("get_me1").translation-PLAYER_CENTER;
        let before=s[local].sanity();f.contacts_pair(&mut slots(&mut p,&mut s));
        ensure!(s[local].sanity()==before+25. && f.saved.coop.as_ref().unwrap().wait[local]==10.
            && f.saved.coop.as_ref().unwrap().essence[local]==1 && f.personal_essence(Some(local)).is_none()
            && f.personal_essence(Some(1-local)).is_some() && serde_json::to_value(&s[1-local])?==other,"Healing was shared or missing");
        let saved=f.snapshot();let mut restored=Forest::load(a,map,true)?;restored.restore(&saved,map)?;f=restored;
        let health=s[local].sanity();f.contacts_pair(&mut slots(&mut p,&mut s));
        ensure!(s[local].sanity()==health,"Cooldown allowed a duplicate pickup");
        p[local].feet=vec3(0.,0.,16000.);
        for _ in 0..1201 {tick(&mut f,map,&mut w,&mut p,&mut s,FIXED_DT)?;}
        ensure!(f.saved.coop.as_ref().unwrap().wait[local]==0. && f.personal_essence(Some(local))==Some(f.data.at("get_me2").translation),"Personal essence did not return");
        p[local].feet=f.data.at("get_me2").translation-PLAYER_CENTER;
        f.contacts_pair(&mut slots(&mut p,&mut s));ensure!(s[local].sanity()==100.,"Second essence did not heal");
        if !dead {
            p[1-local].feet=f.data.at("get_me1").translation-PLAYER_CENTER;s[1-local].damage(25.);
            f.contacts_pair(&mut slots(&mut p,&mut s));ensure!(s[1-local].sanity()==100.,"Partner lost their individual first station");
        }
        let valid=f.snapshot();let mut bad=valid.clone();bad["coop"]["wait"][local]=serde_json::json!(-1.);
        ensure!(f.restore(&bad,map).is_err() && f.snapshot()==valid,"Invalid personal save mutated world");
        let legacy=shared(&f);f.restore(&legacy,map)?;ensure!(f.saved.coop.is_none(),"Legacy save acquired paired fields");
        let stats=serde_json::to_value(&s)?;f.prepare_pair(&mut slots(&mut p,&mut s));
        ensure!(serde_json::to_value(&s)?==stats,"Older saved visit refilled personal resources");
        println!("PASS WForest healing slot{local} dead_partner={dead}: independent stations, cooldown, save and legacy resources");
    }}
    for first in 0..2 {for dead in [false,true] {
        let (mut f,_,mut p,mut s)=setup(a,map,false)?;
        let at=f.data.at("first_eyestaff").translation;
        p[first].feet=at-Vec3::X*100.;p[1-first].feet=at-Vec3::Y*300.;
        if dead {s[1-first].damage(1000.);}
        f.contacts_pair(&mut slots(&mut p,&mut s));let saved=f.snapshot();f.restore(&saved,map)?;
        p[first].feet=at+Vec3::X*100.;
        ensure!(f.contacts_pair(&mut slots(&mut p,&mut s))==Some(first) && f.saved.staff && f.saved.armed
            && s.iter().all(|s|s.staff_component && s.copies(7)==0),"Fast altar crossing failed or gave a usable Staff");
        ensure!(f.contacts_pair(&mut slots(&mut p,&mut s)).is_none(),"Staff scene restarted");
        f.coop_respawned(first);ensure!(f.saved.coop.as_ref().unwrap().previous[first].is_none(),"Respawn retained stale altar path");
        println!("PASS WForest Staff slot{first} dead_partner={dead}: saved swept arrival, shared quest item and one scene");
    }}
    Ok(())
}
fn controller(stage:&mut Stage)->&mut Forest {
    stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Forest>()).unwrap()
}
fn movers(a:&mut Assets,map:&Bsp)->Result<()> {
    for rider in 0..2 {for blocked in [false,true] {for dead in [false,true] {
        let (mut f,_,mut p,mut s)=setup(a,map,false)?;
        // The native gate's top is inside the authored tunnel ceiling. Isolate
        // its real mover/collider to test carry and a deliberately placed ceiling;
        // this is not evidence of an accessible ride through the native map.
        let mut w=World::fixture(&[]);
        f.saved.staff=true;f.saved.armed=true;f.saved.done[2]=true;
        let e=map.entities.iter().find(|e|e.get("targetname").is_some_and(|n|n=="chessgate")).context("Missing chess gate")?;
        let model=e["model"].trim_start_matches('*').parse()?;let origin=crate::interaction::vector(&e["origin"]).unwrap();
        f.motion.isolate_for_check(model);w.set_dynamic(f.colliders());
        let gate=Collider::model(map,model,origin,Quat::IDENTITY,true)?;
        let center=gate.interior_point().context("Chess gate is empty")?;
        p[rider].feet=(-4..=4).flat_map(|x|(-4..=4).map(move |y|vec3(x as f32*16.,y as f32*16.,0.)))
            .find_map(|offset| {
                let from=center+offset+Vec3::Z*1024.;let to=center+offset;
                let trace=gate.trace(from+PLAYER_CENTER,to+PLAYER_CENTER,vec3(1.,1.,PLAYER_HALF.z));
                let feet=from.lerp(to,trace.fraction);
                (!trace.start_solid && trace.fraction<1. && trace.normal.z>0.65 && w.body_clear(feet)).then_some(feet)
            }).context("No clear supported position on isolated chess gate")?;
        p[rider].grounded=true;
        if dead {s[rider].damage(1000.);}
        let fixed=if blocked {vec![Collider::box_bounds(p[rider].feet+vec3(-50.,-50.,57.),p[rider].feet+vec3(50.,50.,100.))]} else {vec![]};
        w.set_dynamic(fixed.iter().cloned().chain(f.colliders()).collect());
        let before=p.each_ref().map(|p|p.feet);
        f.coop_advance_roster(0.1,map,&mut w,&mut slots(&mut p,&mut s),&fixed)?;
        if blocked && !dead {
            ensure!(f.saved.chess==0. && p.each_ref().map(|p|p.feet)==before,"Blocked rider did not stop shared gate atomically");
        } else {
            ensure!(f.saved.chess==0.1 && p[1-rider].feet==before[1-rider],"Gate clock {} froze or moved non-rider: before{before:?} after{:?}",f.saved.chess,p.each_ref().map(|p|p.feet));
            ensure!(if dead {p[rider].feet==before[rider]} else {p[rider].feet.distance(before[rider]+Vec3::Z*(192.*0.1/4.2))<0.001},"Gate did not carry only its living rider");
        }
        ensure!(p.iter().zip(&s).all(|(p,s)|!s.alive() || w.body_clear(p.feet)),"Gate embedded living rider");
        println!("PASS WForest isolated native gate rider{rider} blocked={blocked} dead={dead}: carry, atomic obstruction and dead-body exclusion");
    }}}
    Ok(())
}
fn scenes(a:&mut Assets,map:&Bsp)->Result<()> {
    for kind in [Kind::Arrival,Kind::Staff,Kind::Caterpillar,Kind::Chess,Kind::Wall] {
        for first in PlayerId::ALL {for skip in [false,true] {for dead in [false,true] {
            let visit=Visit{epoch:1,map:"wforest".into(),entry:Some(if kind.returning(){"wforest_start2"}else{"wforest_start1"}.into()),difficulty:Default::default(),combat_preview:false};
            let mut w=World::from_bsp(map)?;let mut stage=Stage::load(a,map,&mut w,&visit,true)?;
            let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
            let mut roster=Roster::new(&w,eye,yaw,Stats::for_level("wforest",visit.entry.as_deref()))?;
            stage.before_move(&mut roster,map,&mut w,&Default::default(),0.)?;
            roster.players[0].stats.damage(13.);roster.players[1].stats.damage(31.);roster.players[1].stats.spend_will(17.);
            if dead {roster.players[first.partner().index()].stats.damage(1000.);}
            let f=controller(&mut stage);
            f.saved.staff=matches!(kind,Kind::Staff|Kind::Caterpillar|Kind::Chess);f.saved.armed=f.saved.staff;
            if kind==Kind::Chess {f.saved.done[1]=true;f.saved.done[2]=true;f.saved.cave=4.;f.saved.chess=4.2;}
            if kind==Kind::Staff {
                f.saved.staff=false;f.saved.armed=false;
                roster.players[first.index()].player=Player::spawn(&w,f.data.at("first_eyestaff").translation).context("Paired altar spawn blocked")?;
                stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
            } else if kind==Kind::Arrival {f.begin(kind);stage.owner=Some(first);} else {
                let id=match kind {Kind::Caterpillar=>119,Kind::Chess=>63,Kind::Wall=>25,_=>unreachable!()};
                let e=&map.entities[id];let model=e["model"].trim_start_matches('*').parse()?;
                let origin=e.get("origin").and_then(|s|crate::interaction::vector(s)).unwrap_or_default();
                let at=Collider::model(map,model,origin,Quat::IDENTITY,false)?.interior_point().context("Empty paired scene trigger")?-PLAYER_CENTER;
                roster.players[first.index()].player=Player::new(at);
                let before=roster.players.each_ref().map(|p|p.player.feet);
                stage.contacts(&mut roster,&w,before)?;
            }
            ensure!(stage.owner==Some(first),"Wrong WForest scene owner {kind:?} {first:?}");
            // Shared quest flags are expected; preserve individual health/Will and weapons.
            let resources=roster.players.each_ref().map(|b|(b.stats.sanity(),b.stats.will(),b.stats.copies(7)));
            let mut waiting=roster.players[first.partner().index()].player.clone();waiting.release_rope();
            let waiting=serde_json::to_value(waiting)?;
            for n in 0..120*180 {
                if n==12 {
                    let saved=stage.snapshot();let bytes=crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
                    stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,&mut w)?;
                    ensure!(serde_json::to_value(stage.snapshot())?==serde_json::to_value(saved)?,"Paired scene save changed");
                }
                if skip && n==24 {ensure!(stage.interactions.skip_cinematic(map,&mut w,&mut roster.players[first.index()].player,&mut stage.story)?,"Paired scene refused skip");}
                stage.scene_step(&mut roster,map,&mut w,&Default::default())?;
                if stage.owner.is_none(){break;}
                ensure!(stage.owner==Some(first) && serde_json::to_value(&roster.players[first.partner().index()].player)?==waiting,"Scene moved frozen partner");
            }
            if stage.busy() || !controller(&mut stage).saved.done[kind.index()] {
                let report=serde_json::json!({"kind":format!("{kind:?}"),"first":first,"skip":skip,"dead_partner":dead,"stage":stage.snapshot(),"players":roster.players});
                std::fs::write("private/coop-wforest-scene-failure.json",serde_json::to_vec(&report)?)?;
                let owner=stage.owner;let feet=roster.players.each_ref().map(|p|p.player.feet);
                let saved=&controller(&mut stage).saved;
                anyhow::bail!("Paired WForest scene stalled {kind:?}: owner{owner:?} done{:?} cave{} chess{} scene{:?} feet{feet:?}; private/coop-wforest-scene-failure.json",
                    saved.done,saved.cave,saved.chess,saved.scene.as_ref().map(|s|(format!("{:?}",s.kind),s.time,s.ending)));
            }
            for (b,resources) in roster.players.iter().zip(resources) {
                ensure!((b.stats.sanity(),b.stats.will(),b.stats.copies(7))==resources,"Scene changed personal resources");
                if b.stats.alive() {b.player.validate_world(&w)?;}
            }
            if !dead {ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"WForest reunion overlapped");}
            println!("PASS WForest {kind:?} {first:?} skipped={skip} dead_partner={dead}: original scene, frozen partner, save, resources and reunion");
        }}}
    }
    Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/wforest.bsp")?)?;
    clocks(a,&map)?;personal(a,&map)?;scenes(a,&map)?;movers(a,&map)?;
    println!("PASS WForest 62 staged co-op cases across both visits; full traversal, internet, recovery and effects remain separate");
    Ok(())
}
