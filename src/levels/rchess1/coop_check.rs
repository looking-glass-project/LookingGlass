//! Original Checkmate actors/scenes with staged paired fixtures, not a full route.
use super::*;
use crate::{combat::PlayerTarget,coop::{PlayerId,protocol::Visit,stage::Stage,world::Roster},
    level::CoopPlayer,movement::{Controls,FIXED_DT}};

fn slots<'a>(p:&'a mut [Player;2],s:&'a mut [Stats;2])->[CoopPlayer<'a>;2] {
    let [a,b]=p.each_mut();let [x,y]=s.each_mut();
    [CoopPlayer{player:a,stats:x},CoopPlayer{player:b,stats:y}]
}
fn fixture(a:&mut Assets,map:&Bsp)->Result<Encounter> {
    let mut r=Encounter::load(a,map)?;
    for (_,p) in &mut r.saved.pieces {p.active=false;p.spawn_delay=None;}
    // Isolate each authored actor; these are component placements, not traversal.
    r.saved.ambushed.extend(0..r.data.ambushes.len());
    Ok(r)
}
fn floor(blocked:bool)->World {
    let mut boxes=vec![(vec3(-3000.,-3000.,-64.),vec3(3000.,3000.,0.))];
    if blocked {boxes.push((vec3(45.,-32.,0.),vec3(65.,32.,240.)));}
    World::fixture(&boxes)
}
fn people(near:usize,distance:f32)->([Player;2],[Stats;2]) {
    let mut p=[Player::new(vec3(distance,0.,0.1)),Player::new(vec3(0.,240.,0.1))];
    if near==1 {p.swap(0,1);}
    (p,std::array::from_fn(|_|Stats::for_level("rchess1",None)))
}
fn restored(a:&mut Assets,map:&Bsp,r:&Encounter)->Result<Encounter> {
    let bytes=crate::coop::codec::encode(&serde_json::to_vec(&r.snapshot())?)?;
    let mut q=Encounter::load(a,map)?;
    q.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map)?;
    ensure!(q.snapshot()==r.snapshot(),"Checkmate save changed");Ok(q)
}
fn feedback(f:&Feedback)->serde_json::Value {
    serde_json::json!({"players":f.players.map(|p|(p.damage,p.impulse,p.will_drain)),
        "legacy":(f.damage,f.impulse,f.will_drain),"sounds":f.sounds,
        "spatial":f.spatial_sounds,"cues":f.cue_sounds})
}
fn selection(a:&mut Assets,map:&Bsp)->Result<()> {
    for kind in [None,Some(crate::chess::Kind::Pawn),Some(crate::chess::Kind::Knight),
        Some(crate::chess::Kind::Bishop),Some(crate::chess::Kind::Rook)] {
        for near in 0..2 {for excluded in 0..4 {
            let mut r=fixture(a,map)?;
            let piece=if let Some(kind)=kind {
                let (id,p)=r.saved.pieces.iter_mut().find(|(_,p)|p.kind==kind)
                    .with_context(||format!("No native Checkmate {kind:?}"))?;
                *p=crate::chess::Piece::new(kind,Vec3::Z*0.1,0.,p.scale,BASE+100+*id);
                Some(*id)
            } else {
                r.saved.phase=Phase::Fight;
                r.saved.boss=battle::Boss::new(Transform{translation:Vec3::Z*0.1,rotation:Quat::IDENTITY});None
            };
            let (mut p,mut s)=people(near,130.);
            if excluded==1 {s[near].damage(1000.);} if excluded==2 {s[near].invisible=10.;}
            let w=floor(excluded==3);let before=r.snapshot();
            for dt in [0.,-1.,f32::NAN] {
                let f=r.coop_combat_roster(dt,&w,&mut slots(&mut p,&mut s));
                ensure!(r.snapshot()==before && f.players.iter().all(|p|p.damage==0.),"Paused paired combat changed");
            }
            r.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut p,&mut s));
            let selected=piece.map_or(r.saved.boss.player,|id|
                r.snapshot()["coop"]["piece_players"][id.to_string()].as_u64().map(|n|n as usize));
            ensure!(selected==Some(if excluded==0 {near} else {1-near}),
                "Wrong Checkmate {kind:?} target near{near} excluded{excluded}: {selected:?}");
            restored(a,map,&r)?;
            println!("PASS Checkmate {kind:?} near{near} excluded{excluded}: native nearest-visible living selection, pause and saved target");
        }}
    }Ok(())
}
fn attacks(a:&mut Assets,map:&Bsp)->Result<()> {
    for kind in [crate::chess::Kind::Pawn,crate::chess::Kind::Knight,crate::chess::Kind::Bishop,crate::chess::Kind::Rook] {
        for victim in 0..2 {
            let mut r=fixture(a,map)?;
            let (id,piece)=r.saved.pieces.iter_mut().find(|(_,p)|p.kind==kind).context("Missing native enemy family")?;
            *piece=crate::chess::Piece::new(kind,Vec3::Z*0.1,0.,piece.scale,BASE+100+*id);
            let (mut p,mut s)=people(victim,70.);let w=floor(false);let mut out=Feedback::default();
            for _ in 0..1200 {
                out.append(r.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut p,&mut s)));
                if out.players[victim].damage>0. {break;}
            }
            ensure!(out.players[victim].damage>0. && out.players[1-victim].damage==0.
                && out.damage==0. && !out.spatial_sounds.is_empty(),"Checkmate {kind:?} attack routing: {}",feedback(&out));
            println!("PASS Checkmate {kind:?} victim{victim}: native attack frames, personal damage and sound");
        }
    }
    for victim in 0..2 {
        let r=fixture(a,map)?;let w=floor(false);
        let mut b=battle::Boss::new(Transform{translation:Vec3::Z*0.1,rotation:Quat::IDENTITY});
        let (p,s)=people(victim,80.);
        b.opponents.players=Some(std::array::from_fn(|i|PlayerTarget{eye:p[i].eye(),alive:s[i].alive(),hidden:false}));
        b.set(battle::Action::Melee);let mut out=Feedback::default();
        for _ in 0..(r.data.boss().duration("attack_1")*120.).floor() as usize {
            b.step(&w,p[0].eye(),false,false,&r.data,&mut out);
        }
        ensure!(out.players[victim].damage==10. && out.players[1-victim].damage==0.
            && out.damage==0. && !out.cue_sounds.is_empty(),"King melee routing: {}",feedback(&out));
        println!("PASS Checkmate King melee victim{victim}: native contact frames, damage once and personal sound");

        let mut r=fixture(a,map)?;r.saved.phase=Phase::Fight;
        r.saved.boss=battle::Boss::new(Transform{translation:Vec3::Z*0.1,rotation:Quat::IDENTITY});
        r.saved.boss.set(battle::Action::Beam);r.saved.boss.player=Some(victim);
        let (mut p,mut s)=people(victim,300.);p[1-victim]=Player::new(vec3(0.,90.,0.1));
        r.coop_prepare(&mut slots(&mut p,&mut s));let mut q=restored(a,map,&r)?;
        let mut qp=p.clone();let mut qs=s.clone();let mut out=Feedback::default();let mut visible=false;
        for _ in 0..(r.data.boss().duration("attack_4")*120.).floor() as usize {
            let f=r.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut p,&mut s));
            let g=q.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut qp,&mut qs));
            ensure!(feedback(&f)==feedback(&g) && r.snapshot()==q.snapshot(),"Saved King beam future diverged");
            visible|=!r.saved.boss.beams.is_empty();out.append(f);
        }
        ensure!(out.players[victim].damage==8. && out.players[1-victim].damage==0. && out.damage==0. && visible,
            "King beam routing: {}",feedback(&out));
        println!("PASS Checkmate King beam victim{victim}: saved committed target, four native contacts and visible beam state");
    }Ok(())
}
fn saves_and_victory(a:&mut Assets,map:&Bsp)->Result<()> {
    let mut r=fixture(a,map)?;let legacy=r.snapshot();
    ensure!(legacy.get("coop").is_none() && legacy["boss"].get("player").is_none(),"Legacy save acquired paired fields");
    let (mut p,mut s)=people(0,80.);r.coop_prepare(&mut slots(&mut p,&mut s));let good=r.snapshot();
    for (field,value) in [("essence",serde_json::json!([3,0])),("wait",serde_json::json!([0.,1.01])),
        ("actor",serde_json::json!(2)),("piece_players",serde_json::json!({"999999":0}))] {
        let mut bad=good.clone();bad["coop"][field]=value;
        ensure!(r.restore(&bad,map).is_err() && r.snapshot()==good,"Malformed paired save changed live state: {field}");
    }
    r.restore(&legacy,map)?;ensure!(r.snapshot()==legacy,"Legacy Checkmate save changed");
    println!("PASS Checkmate legacy shape and immutable rejected personal pickup/actor/target saves");
    r.saved.phase=Phase::Fight;r.hit(Hit{id:BASE,damage:2000.,kind:crate::combat::DamageKind::Ice,knockback:Vec3::ZERO});
    for _ in 0..2400 {
        r.coop_combat_roster(FIXED_DT,&floor(false),&mut slots(&mut p,&mut s));
        if r.saved.boss.done(&r.data) {break;}
    }
    ensure!(r.saved.boss.done(&r.data),"Native death animation did not finish");
    for stat in &mut s {stat.damage(1000.);}
    let before=r.snapshot();ensure!(r.coop_contacts(FIXED_DT,&mut slots(&mut p,&mut s)).is_none()
        && r.snapshot()==before,"Dead team started victory scene instead of checkpoint recovery");
    println!("PASS Checkmate dead-team victory guard: no scene owner or progression");Ok(())
}
fn pickups_and_clocks(a:&mut Assets,map:&Bsp)->Result<()> {
    for first in 0..2 {
        let mut r=fixture(a,map)?;r.saved.phase=Phase::Fight;
        let mut w=World::from_bsp(map)?;let (mut p,mut s)=people(first,1000.);
        s[0].set_health(40.)?;s[1].set_health(50.)?;
        let at=r.essence_position();p[first]=Player::new(at-Vec3::Z*28.);
        let other=serde_json::to_value(&s[1-first])?;
        let hp=s[first].sanity();r.coop_prepare(&mut slots(&mut p,&mut s));
        ensure!(s[first].sanity()==hp+25. && serde_json::to_value(&s[1-first])?==other,
            "Personal essence shared its reward");
        ensure!(r.visible_essence(Some(first)).1==1. && r.visible_essence(Some(1-first))==(at,0.),
            "Personal essence changed partner presentation");
        r=restored(a,map,&r)?;let before=r.snapshot();let resources=serde_json::to_value((&p,&s))?;
        for dt in [0.,-1.,f32::NAN] {r.coop_advance_roster(dt,map,&mut w,&mut slots(&mut p,&mut s),&[])?;}
        ensure!(r.snapshot()==before && serde_json::to_value((&p,&s))?==resources,"Paused essence/clock changed");
        p[first]=Player::new(vec3(0.,0.,16000.));p[1-first]=Player::new(at-Vec3::Z*28.);
        let hp=s[1-first].sanity();r.coop_prepare(&mut slots(&mut p,&mut s));
        ensure!(s[1-first].sanity()==hp+25.,"Second player lost their own copy");
        r.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
        ensure!((r.saved.clock-FIXED_DT).abs()<0.000001
            && (0..2).all(|i|(r.visible_essence(Some(i)).1-(1.-FIXED_DT)).abs()<0.000001),
            "Paired clock or personal cooldown advanced more than once");
        for stat in &mut s {stat.damage(1000.);}
        let before=r.snapshot();r.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
        r.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut p,&mut s));
        ensure!(r.snapshot()==before,"Dead team advanced world");
        println!("PASS Checkmate personal healing first{first}: separate reward/location/cooldown, pause, restore and dead-team freeze");
    }
    for hz in [30,60,144] {
        let mut r=fixture(a,map)?;r.saved.phase=Phase::Fight;
        let mut q=restored(a,map,&r)?;let mut w=World::from_bsp(map)?;let mut sw=w.clone();
        let (mut p,mut s)=people(0,1000.);let mut actor=p[0].clone();
        for _ in 0..hz*2 {
            r.coop_advance_roster(1./hz as f32,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
            q.advance(1./hz as f32,map,&mut sw,&mut actor,&[])?;
            ensure!(r.saved.clock==q.saved.clock && r.saved.time==q.saved.time && r.transforms()==q.transforms(),
                "Paired native machinery clock diverged at {hz}Hz");
        }
        println!("PASS Checkmate shared clock {hz}Hz: native waterwheel and bridge clock parity");
    }Ok(())
}
fn ctl(stage:&mut Stage)->&mut Encounter {
    stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut()).unwrap()
}
fn scene(a:&mut Assets,map:&Bsp,phase:Phase,first:PlayerId,skip:bool,dead:bool)->Result<()> {
    let label=match phase {Phase::Beheading=>BEHEAD,Phase::Intro=>INTRO,_=>KILLED};
    let visit=Visit{epoch:1,map:"rchess1".into(),entry:Some("rchess1_start1".into()),difficulty:Default::default(),combat_preview:false};
    let mut w=World::from_bsp(map)?;let mut stage=Stage::load(a,map,&mut w,&visit,true)?;
    let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
    let mut roster=Roster::new(&w,eye,yaw,Stats::for_level("rchess1",visit.entry.as_deref()))?;
    roster.players[0].stats.set_health(54.)?;roster.players[1].stats.set_health(67.)?;
    if dead {roster.players[first.partner().index()].stats.damage(1000.);}
    if phase==Phase::Killed {
        let r=ctl(&mut stage);r.saved.phase=Phase::Fight;
        for (_,p) in &mut r.saved.pieces {p.active=false;p.spawn_delay=None;}
        r.saved.ambushed.extend(0..r.data.ambushes.len());
        roster.players[first.index()].player=Player::spawn(&w,r.saved.boss.at+vec3(-160.,0.,48.))
            .context("No staged King survivor footing")?;
        r.hit(Hit{id:BASE,damage:2000.,kind:crate::combat::DamageKind::Ice,knockback:Vec3::ZERO});
        for _ in 0..2400 {
            let mut p=roster.players.each_mut().map(|b|CoopPlayer{player:&mut b.player,stats:&mut b.stats});
            r.coop_combat_roster(FIXED_DT,&w,&mut p);
            if r.saved.boss.done(&r.data) {break;}
        }
        ensure!(r.saved.boss.done(&r.data),"Boss death animation did not finish");
        stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
    } else {
        if phase==Phase::Intro {let r=ctl(&mut stage);r.saved.phase=Phase::Northern;r.saved.beheaded=true;r.rebuild(map)?;}
        let e=map.entities.iter().find(|e|e.get("thread").is_some_and(|s|s==label)).context("Missing original scene trigger")?;
        let model=e.get("model").and_then(|s|s.strip_prefix('*')).context("Scene trigger has no brush")?.parse()?;
        let volume=Collider::model(map,model,data::pose(e).translation,Quat::IDENTITY,false)?;
        let at=volume.interior_point().context("Scene trigger has no interior")?-crate::collision::PLAYER_CENTER;
        roster.players[first.index()].player=Player::new(at);
        let before=roster.players.each_ref().map(|p|p.player.feet);
        stage.contacts(&mut roster,&w,before)?;
    }
    ensure!(stage.scene_id()==Some(label) && stage.owner==Some(first),"Scene {label} bound wrong first player {first:?}: {:?}",stage.owner);
    let waiting=roster.players[first.partner().index()].player.feet;
    let resources=roster.players.each_ref().map(|p|serde_json::to_value(&p.stats).unwrap());
    for tick in 0..24000 {
        if tick==120 {
            let value=serde_json::to_value(stage.snapshot())?;let people=serde_json::to_value(&roster.players)?;
            let r=ctl(&mut stage);
            let mut p=roster.players.each_mut().map(|b|CoopPlayer{player:&mut b.player,stats:&mut b.stats});
            r.coop_advance_roster(0.,map,&mut w,&mut p,&[])?;r.coop_combat_roster(0.,&w,&mut p);
            ensure!(serde_json::to_value(stage.snapshot())?==value && serde_json::to_value(&roster.players)?==people,"Paused scene changed");
            let bytes=crate::coop::codec::encode(&serde_json::to_vec(&value)?)?;
            stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,&mut w)?;
            ensure!(serde_json::to_value(stage.snapshot())?==value,"Restored scene changed");
        }
        let mut input=[(Controls::default(),crate::coop::protocol::Input::default());2];
        input[first.partner().index()].1.skip_scene=true;input[first.index()].1.skip_scene=skip && tick>=240;
        stage.scene_step(&mut roster,map,&mut w,&input)?;
        if tick<240 {ensure!(stage.scene_id()==Some(label),"Partner skipped scene");}
        if skip && tick==330 {ensure!(stage.scene_id()!=Some(label),"Owner hold failed to skip");}
        if !stage.busy() || stage.transition.is_some() {break;}
        ensure!(roster.players[first.partner().index()].player.feet==waiting,"Scene moved waiting partner");
    }
    if phase==Phase::Killed {
        ensure!(stage.transition.as_ref().is_some_and(|e|EXIT.matches(e)),"King outro did not leave for Funhouse");
    } else {
        ensure!(!stage.busy(),"Checkmate reunion stalled");
        if !dead {ensure!(roster.players.iter().all(|p|w.body_clear(p.player.feet))
            && roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Checkmate reunion overlaps");}
    }
    ensure!(roster.players.iter().zip(&resources).all(|(p,s)|serde_json::to_value(&p.stats).unwrap()==*s),"Scene changed personal resources");
    ensure!(stage.notices[first.index()].as_ref().is_some_and(|n|n.text=="Scene skipped")==skip,"Wrong scene skip notice");
    println!("PASS Checkmate {label} {first:?} skip{skip} dead{dead}: staged native contact/death ownership, frozen partner, pause/save and handoff");
    Ok(())
}
fn route_combat(a:&mut Assets,map:&Bsp)->Result<()> {
    use crate::coop::{protocol::Input,route_driver::{combat_controls,native_threat}};
    let visit=Visit{epoch:1,map:"rchess1".into(),entry:Some("rchess1_start1".into()),difficulty:Default::default(),combat_preview:false};
    let mut w=World::from_bsp(map)?;let mut stage=Stage::load(a,map,&mut w,&visit,false)?;
    w=floor(false);
    let mut roster=Roster::new(&w,vec3(-200.,0.,48.),0.,Stats::for_level("rchess1",None))?;
    for kind in [None,Some(crate::chess::Kind::Pawn),Some(crate::chess::Kind::Knight),
        Some(crate::chess::Kind::Bishop),Some(crate::chess::Kind::Rook)] {
        let mut r=fixture(a,map)?;
        let id=if let Some(kind)=kind {
            let (id,p)=r.saved.pieces.iter_mut().find(|(_,p)|p.kind==kind).context("Missing chess family")?;
            *p=crate::chess::Piece::new(kind,Vec3::Z*0.1,0.,p.scale,BASE+100+*id);
            BASE+100+*id
        } else {
            r.saved.phase=Phase::Fight;
            r.saved.boss=battle::Boss::new(Transform{translation:Vec3::Z*0.1,rotation:Quat::IDENTITY});BASE
        };
        *ctl(&mut stage)=r;
        for local in PlayerId::ALL {
            // Component placements: the same combat controller used by each
            // real client must aim ordinary Cards input at the native enemy.
            let body=&mut roster.players[local.index()];body.player=Player::new(vec3(-200.,0.,0.1));
            ensure!(native_threat(&w,&stage.interactions,vec![],body)==Some(id),
                "Visible native threat missing for {kind:?} {local:?}");
            let mut keys=Input::default();
            combat_controls(&w,&stage.interactions,vec![],body,true,false,|_|false,&mut keys);
            ensure!(keys.weapon==Some(1) && keys.primary && !keys.alternate,
                "Route driver ignored native {kind:?} for {local:?}");
            let aim=vec3(keys.yaw.cos()*keys.pitch.cos(),keys.yaw.sin()*keys.pitch.cos(),keys.pitch.sin());
            let targets=stage.interactions.levels_targets();
            let context=crate::combat::Context{world:&w,targets:&targets};let eye=body.player.eye();
            ensure!(crate::combat::contact(&context,eye,eye+aim*1000.,1.).is_some_and(|(hit,_)|hit==id),
                "Route driver aimed away from native {kind:?}");
            let blocked=floor(true);body.player=Player::new(vec3(200.,0.,0.1));
            let mut hidden=Input::default();
            combat_controls(&blocked,&stage.interactions,vec![],body,true,false,|_|false,&mut hidden);
            ensure!(hidden.weapon.is_none() && !hidden.primary && !hidden.alternate,
                "Route driver shot through the occluding wall");
            ensure!(native_threat(&blocked,&stage.interactions,vec![],body).is_none(),
                "Clear diagnostic waited for an occluded enemy");
            body.player=Player::new(vec3(-800.,0.,0.1));
            ensure!(native_threat(&w,&stage.interactions,vec![],body).is_none(),
                "Clear diagnostic exceeded the 700-unit combat radius");
            println!("PASS Checkmate route driver {kind:?} {local:?}: ordinary Cards aim, wall occlusion and visible-threat radius");
        }
    }Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/rchess1.bsp")?)?;let mut failures=Vec::new();
    for (label,result) in [("selection",selection(a,&map)),("attacks",attacks(a,&map)),
        ("healing/clocks",pickups_and_clocks(a,&map)),("saves/victory",saves_and_victory(a,&map)),
        ("route combat",route_combat(a,&map))] {
        if let Err(e)=result {failures.push(format!("{label}: {e:#}"));}
    }
    for phase in [Phase::Beheading,Phase::Intro,Phase::Killed] {for first in PlayerId::ALL {for skip in [false,true] {for dead in [false,true] {
        if let Err(e)=scene(a,&map,phase,first,skip,dead) {failures.push(format!("{phase:?} {first:?} skip{skip} dead{dead}: {e:#}"));}
    }}}}
    ensure!(failures.is_empty(),"Checkmate paired failures:\n{}",failures.join("\n"));
    println!("PASS Checkmate paired native components; full traversal, internet and recovery remain separate");Ok(())
}
