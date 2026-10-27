//! Castling's native actors and animations with two personal combat targets.
//! These component checks do not establish paired machinery or a full route.
use super::*;
use crate::{chess::{Kind,Phase},combat::{DamageKind,Feedback,Hit,PlayerTarget},movement::FIXED_DT};

fn setup(a:&mut Assets,map:&Bsp)->Result<Castle> {
    let mut r=Castle::load(a,map)?;
    for actor in &mut r.saved.cast {actor.piece.active=false;actor.piece.spawn_delay=None;}
    Ok(r)
}
fn floor(blocked:bool)->World {
    let mut boxes=vec![(vec3(-3000.,-3000.,-64.),vec3(3000.,3000.,0.))];
    if blocked {boxes.push((vec3(45.,-32.,0.),vec3(65.,32.,240.)));}
    World::fixture(&boxes)
}
fn players(near:usize,distance:f32)->[PlayerTarget;2] {
    let mut p=[vec3(distance,0.,52.1),vec3(0.,240.,52.1)]
        .map(|eye|PlayerTarget{eye,alive:true,hidden:false});
    if near==1 {p.swap(0,1);} p
}
fn actor(r:&Castle,kind:Kind)->Result<usize> {
    r.saved.cast.iter().position(|a|!a.white && a.piece.kind==kind)
        .context("Missing native Castling enemy family")
}
fn activate(r:&mut Castle,index:usize,feet:Vec3) {
    let a=&mut r.saved.cast[index];
    a.piece=crate::chess::Piece::new(a.piece.kind,feet,0.,a.piece.scale,a.id());
    a.walk=None;a.foe=None;a.player=None;
}
fn restored(a:&mut Assets,map:&Bsp,r:&Castle)->Result<Castle> {
    let bytes=crate::coop::codec::encode(&serde_json::to_vec(&r.snapshot())?)?;
    let mut q=Castle::load(a,map)?;
    q.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map)?;
    ensure!(q.snapshot()==r.snapshot(),"Castling paired snapshot changed across restore");Ok(q)
}
fn feedback(f:&Feedback)->serde_json::Value {
    serde_json::json!({"players":f.players.map(|p|(p.damage,p.impulse,p.will_drain)),
        "legacy":(f.damage,f.impulse,f.will_drain),"sounds":f.sounds,"spatial":f.spatial_sounds})
}
fn selection(a:&mut Assets,map:&Bsp)->Result<()> {
    for kind in [Kind::Pawn,Kind::Knight,Kind::Bishop,Kind::Rook] {for near in 0..2 {for excluded in 0..4 {
        let mut r=setup(a,map)?;let index=actor(&r,kind)?;activate(&mut r,index,Vec3::Z*0.1);
        let mut p=players(near,130.);p[near].alive=excluded!=1;p[near].hidden=excluded==2;
        let w=floor(excluded==3);let before=r.snapshot();
        for dt in [0.,-1.,f32::NAN] {
            let f=r.coop_combat(dt,&w,p);
            ensure!(r.snapshot()==before && f.players.iter().all(|p|p.damage==0.),"Paused Castling combat changed");
        }
        r.coop_combat(FIXED_DT,&w,p);
        ensure!(r.saved.cast[index].player==Some(if excluded==0 {near}else{1-near}),
            "Wrong Castling {kind:?} target near{near} excluded{excluded}: {:?}",r.saved.cast[index].player);
        restored(a,map,&r)?;
        println!("PASS Castling {kind:?} near{near} excluded{excluded}: closest visible living target, pause and saved ownership");
    }}}Ok(())
}
fn attacks(a:&mut Assets,map:&Bsp)->Result<()> {
    for kind in [Kind::Pawn,Kind::Knight,Kind::Bishop,Kind::Rook] {for victim in 0..2 {
        let mut r=setup(a,map)?;let index=actor(&r,kind)?;activate(&mut r,index,Vec3::Z*0.1);
        let p=players(victim,70.);let w=floor(false);let mut out=Feedback::default();
        for _ in 0..1200 {
            out.append(r.coop_combat(FIXED_DT,&w,p));
            if out.players[victim].damage>0. {break;}
        }
        ensure!(out.players[victim].damage>0. && out.players[1-victim].damage==0.
            && out.damage==0. && !out.spatial_sounds.is_empty(),
            "Castling {kind:?} attack hit wrong victim{victim}: {}",feedback(&out));
        println!("PASS Castling {kind:?} victim{victim}: native attack frames, personal damage and sound");
    }}
    for victim in 0..2 {
        let mut r=setup(a,map)?;let index=actor(&r,Kind::Bishop)?;activate(&mut r,index,Vec3::Z*0.1);
        let mut p=players(victim,300.);p[1-victim].eye=vec3(0.,700.,52.1);let w=floor(false);
        for _ in 0..600 {r.coop_combat(FIXED_DT,&w,p);if r.saved.cast[index].piece.phase==Phase::Beam {break;}}
        ensure!(r.saved.cast[index].piece.phase==Phase::Beam,"Castling bishop did not begin a beam");
        p[1-victim].eye=vec3(0.,90.,52.1);let mut q=restored(a,map,&r)?;let mut out=Feedback::default();
        for _ in 0..120 {
            let f=r.coop_combat(FIXED_DT,&w,p);let g=q.coop_combat(FIXED_DT,&w,p);
            ensure!(feedback(&f)==feedback(&g) && r.snapshot()==q.snapshot(),"Saved Castling beam future diverged");
            out.append(f);if out.players[victim].damage>0. {break;}
        }
        ensure!(out.players[victim].damage==8. && out.players[1-victim].damage==0.
            && r.saved.cast[index].piece.beam.is_some(),"Castling beam lost its victim or visible effect state");
        println!("PASS Castling beam victim{victim}: committed owner after restore, personal impact and beam state");
    }Ok(())
}
fn factions(a:&mut Assets,map:&Bsp)->Result<()> {
    for near in 0..2 {for forced in [false,true] {
        let mut r=setup(a,map)?;let red=actor(&r,Kind::Knight)?;
        let white=r.saved.cast.iter().position(|a|a.white && a.piece.kind==Kind::Pawn && !a.name.is_empty())
            .context("Missing named native white pawn")?;
        activate(&mut r,red,Vec3::Z*0.1);activate(&mut r,white,vec3(70.,0.,0.1));
        r.saved.cast[white].piece.script_wait=true;r.saved.cast[white].damageable=true;
        if forced {r.saved.cast[red].foe=Some(r.saved.cast[white].name.clone());}
        let mut p=players(near,50.);p[near].eye=vec3(30.,-30.,52.1);
        ensure!(p[near].eye.distance(r.saved.cast[red].piece.feet)
            <r.saved.cast[white].piece.target(0).center.distance(r.saved.cast[red].piece.feet),"Castling faction fixture chose farther Alice");
        let health=r.saved.cast[white].piece.health;let mut out=Feedback::default();let w=floor(false);
        for _ in 0..1200 {
            out.append(r.coop_combat(FIXED_DT,&w,p));
            if out.players[near].damage>0. || r.saved.cast[white].piece.health<health {break;}
        }
        ensure!(if forced {r.saved.cast[white].piece.health<health && out.players.iter().all(|p|p.damage==0.)}
            else {r.saved.cast[white].piece.health==health && out.players[near].damage>0. && out.players[1-near].damage==0.},
            "Castling duel/nearest-player routing failed near{near} forced{forced}");
        ensure!(out.damage==0.,"Castling NPC duel damaged legacy Alice");restored(a,map,&r)?;
        println!("PASS Castling factions near{near} forced{forced}: scripted duel priority and separate personal damage");
    }}Ok(())
}
fn compatibility(a:&mut Assets,map:&Bsp)->Result<()> {
    let mut r=setup(a,map)?;let index=actor(&r,Kind::Knight)?;activate(&mut r,index,Vec3::Z*0.1);
    let id=r.saved.cast[index].id();r.coop_threats(&|target|target.id==id);
    ensure!(r.saved.cast[index].piece.phase==Phase::Block,"Castling knight ignored paired projectile threat");
    r.cast_hit(Hit{id,damage:40.,kind:DamageKind::Electric,knockback:Vec3::ZERO});
    ensure!(r.saved.cast[index].piece.health==110.,"Castling shield lost electric vulnerability");
    let valid=r.snapshot();let mut bad=valid.clone();bad["cast"][index]["player"]=serde_json::json!(2);
    ensure!(r.restore(&bad,map).is_err() && r.snapshot()==valid,"Invalid Castling target modified saved state");
    ensure!(valid["cast"].as_array().unwrap().iter().all(|a|a.get("player").is_none()),"Legacy Castling shape acquired player fields");
    r.restore(&valid,map)?;r.saved.pending=Some(scene::Kind::Queen);let before=r.snapshot();
    r.coop_threats(&|_|true);r.coop_combat(FIXED_DT,&floor(false),players(0,70.));
    ensure!(r.snapshot()==before,"Royal scene advanced paired combat");
    println!("PASS Castling shield, electric damage, legacy/invalid saves and cinematic combat freeze");
    let mut r=setup(a,map)?;
    for group in [1,2] {
        let event=format!("start_battle_group{group}_thread");r.event(&event);
        let before=r.snapshot();r.event(&event);ensure!(r.snapshot()==before,"Castling duel event rearmed");
        let prefix=format!("battle{group}_");
        ensure!(r.saved.cast.iter().any(|a|a.name.starts_with(&prefix) && a.foe.is_some()),"Castling lost native battle links");
    }
    ensure!(r.saved.cast.iter().filter(|a|a.white && a.name.starts_with("battle")).all(|a|a.piece.health==10000.),"White battle health changed");
    let rook=r.saved.cast.iter().position(|a|a.name.starts_with("w_rook")).context("Missing royal rook")?;
    let id=r.saved.cast[rook].id();r.saved.cast[rook].piece.active=true;
    r.cast_hit(Hit{id,damage:10000.,kind:DamageKind::Other,knockback:Vec3::ZERO});
    ensure!(r.saved.cast[rook].piece.health==10.,"Royal rook health floor changed");restored(a,map,&r)?;
    println!("PASS Castling native duel links, one-shot groups, white battle health and royal rook health floor");
    Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/wchess2.bsp")?)?;
    selection(a,&map)?;attacks(a,&map)?;factions(a,&map)?;compatibility(a,&map)?;
    println!("PASS Castling paired cast components; world, full internet traversal and recovery remain separate");Ok(())
}
