//! Staged native battle components. These checks do not claim paired traversal.
use super::*;
use crate::{combat::{DamageKind,PlayerTarget},level::CoopPlayer,movement::FIXED_DT};

fn slots<'a>(p:&'a mut [Player;2],s:&'a mut [Stats;2])->[CoopPlayer<'a>;2] {
    let [a,b]=p.each_mut();let [x,y]=s.each_mut();
    [CoopPlayer{player:a,stats:x},CoopPlayer{player:b,stats:y}]
}
fn people(near:usize)->[PlayerTarget;2] {
    let mut p=[PlayerTarget{eye:vec3(70.,0.,56.03125),alive:true,hidden:false},
        PlayerTarget{eye:vec3(0.,240.,56.03125),alive:true,hidden:false}];
    if near==1 {p.swap(0,1);} p
}
fn floor(wall:bool)->World {
    let mut boxes=vec![(vec3(-3000.,-3000.,-64.),vec3(3000.,3000.,0.))];
    if wall {boxes.push((vec3(35.,-32.,0.),vec3(45.,32.,200.)));}
    World::fixture(&boxes)
}
fn boss(family:usize,mini:bool)->battle::Boss {
    battle::Boss::new(BASE+family,family,mini,Transform{translation:Vec3::Z*0.03125,rotation:Quat::IDENTITY})
}
fn fixture(a:&mut Assets,map:&Bsp)->Result<Funhouse> {
    let mut o=Funhouse::load(a,map)?;
    o.saved.scene=None;o.saved.initialized=true;o.saved.fight=true;o.saved.essence=false;
    o.saved.bosses=vec![boss(0,false),boss(1,false)];
    o.saved.bosses[1].at=vec3(0.,800.,0.03125);
    Ok(o)
}
fn summary(out:&Feedback)->serde_json::Value {
    serde_json::json!({"players":out.players.map(|p|(p.damage,p.impulse,p.will_drain)),
        "legacy":(out.damage,out.impulse),"sounds":out.cue_sounds})
}
fn restored(a:&mut Assets,map:&Bsp,o:&Funhouse)->Result<Funhouse> {
    let bytes=crate::coop::codec::encode(&serde_json::to_vec(&o.snapshot())?)?;
    let mut q=Funhouse::load(a,map)?;
    q.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map)?;
    ensure!(q.snapshot()==o.snapshot(),"Paired Funhouse save changed");Ok(q)
}
fn targeting(data:&data::Data)->Result<()> {
    for family in 0..2 {for mini in [false,true] {for near in 0..2 {for excluded in 0..5 {
        let mut b=boss(family,mini);let mut p=people(near);let w=floor(excluded==3);
        if excluded==1 {p[near].alive=false;} if excluded==2 {p[near].hidden=true;}
        if excluded==4 {for p in &mut p {p.alive=false;}}
        b.opponents.players=Some(p);
        b.step(data,&w,p[0].eye,false,0,&mut Feedback::default());
        let expected=if excluded==4 {None} else {Some(if excluded==0 {near} else {1-near})};
        ensure!(b.opponents.selected_player==expected,"Wrong Tweedle target family{family} mini{mini} near{near} excluded{excluded}");
        println!("PASS Funhouse target family{family} mini{mini} near{near} exclusion{excluded}");
    }}}} Ok(())
}
fn attacks(data:&data::Data)->Result<()> {
    let w=floor(false);
    for family in 0..2 {for mini in [false,true] {for victim in 0..2 {
        let mut b=boss(family,mini);let p=people(victim);b.opponents.players=Some(p);
        b.action=battle::Action::Knife;
        b.time=(if family==0 {8.} else {7.})*data.rigs[b.model()].frame(b.clip())-FIXED_DT*0.5;
        let mut out=Feedback::default();
        for _ in 0..2 {b.step(data,&w,p[0].eye,false,0,&mut out);}
        ensure!(out.players[victim].damage==10. && out.players[1-victim].damage==0. && out.damage==0.,
            "Tweedle knife routing: {}",summary(&out));
        println!("PASS Funhouse native knife family{family} mini{mini} victim{victim}");
    }}}
    for victim in 0..2 {
        let mut b=boss(0,false);
        let mut p=[PlayerTarget{eye:vec3(100.,0.,56.),alive:true,hidden:false},
            PlayerTarget{eye:vec3(160.,0.,56.),alive:true,hidden:false}];
        if victim==1 {p.swap(0,1);} b.opponents.players=Some(p);
        b.grenades.push(battle::Grenade{at:vec3(100.,0.,36.),velocity:Vec3::ZERO,age:0.,ended:None,serial:1});
        let mut out=Feedback::default();b.step(data,&w,p[0].eye,false,0,&mut out);
        ensure!(out.players[victim].damage==5. && out.players[1-victim].damage>0. && out.damage==0.
            && b.grenades[0].ended.is_some(),"Grenade direct/splash did not hit actual bodies: {}",summary(&out));
        let mut again=Feedback::default();b.step(data,&w,p[0].eye,false,0,&mut again);
        ensure!(again.players.iter().all(|p|p.damage==0.),"Ended grenade dealt damage twice");
        println!("PASS Funhouse grenade direct victim{victim}, partner splash and one explosion");
    }
    for blocked in [false,true] {
        let mut b=boss(0,false);let mut p=people(0);
        p[0].eye=vec3(-50.,0.,56.);p[1].eye=vec3(50.,0.,56.);b.opponents.players=Some(p);
        b.grenades.push(battle::Grenade{at:vec3(0.,0.,36.),velocity:Vec3::ZERO,age:2.,ended:None,serial:1});
        let wall=floor(blocked);let mut out=Feedback::default();b.step(data,&wall,p[0].eye,false,0,&mut out);
        ensure!(out.players[0].damage>0. && (out.players[1].damage>0.)==!blocked && out.damage==0.,
            "Grenade splash ignored body or wall: {}",summary(&out));
        println!("PASS Funhouse two-player grenade radius and wall blocked={blocked}");
    }
    for family in 0..2 {
        let mut b=boss(family,false);let mut p=people(0);
        p[0].eye=vec3(150.,0.,56.03125);p[1].eye=vec3(270.,0.,56.03125);
        b.opponents.players=Some(p);b.action=battle::Action::Fall;b.time=0.3;b.velocity=-Vec3::Z*100.;
        let mut out=Feedback::default();b.step(data,&w,p[0].eye,false,0,&mut out);
        ensure!((out.players[0].damage-25.).abs()<0.1 && (out.players[1].damage-5.).abs()<0.1 && out.damage==0.,
            "Ground pound did not preserve personal radial falloff: {}",summary(&out));
        println!("PASS Funhouse ground pound family{family}, both players with original falloff");
    }
    Ok(())
}
fn healing_and_saves(a:&mut Assets,map:&Bsp)->Result<()> {
    let mut o=fixture(a,map)?;let legacy=o.snapshot();
    ensure!(legacy.get("coop").is_none() && legacy["bosses"][0].get("opponents").is_none(),"SP save acquired paired fields");
    let mut p=[Player::new(vec3(1200.,0.,0.03125)),Player::new(vec3(0.,1200.,0.03125))];
    let mut s=std::array::from_fn(|_|Stats::for_level("funhouse",None));
    o.coop_prepare(&mut slots(&mut p,&mut s));let good=o.snapshot();
    for (field,value) in [("side",serde_json::json!([2,0])),("wait",serde_json::json!([0.,10.1])),
        ("actor",serde_json::json!(2)),("targets",serde_json::json!({"999999":0}))] {
        let mut bad=good.clone();bad["coop"][field]=value;
        ensure!(o.restore(&bad,map).is_err() && o.snapshot()==good,"Invalid paired save mutated state: {field}");
    }
    o.restore(&legacy,map)?;ensure!(o.snapshot()==legacy,"Legacy shape changed");
    println!("PASS Funhouse legacy shape and rejected personal/target saves");
    let mut cooling=fixture(a,map)?;cooling.saved.essence_wait=FIXED_DT;
    let mut waiting=std::array::from_fn(|_|Player::new(vec3(10000.,0.,0.)));
    let mut healthy=std::array::from_fn(|_|Stats::for_level("funhouse",None));
    cooling.coop_prepare(&mut slots(&mut waiting,&mut healthy));
    ensure!(cooling.visible_essence(Some(0)).is_none(),"Legacy cooldown was skipped");
    cooling.coop_combat_roster(FIXED_DT,&World::fixture(&[]),&mut slots(&mut waiting,&mut healthy));
    for slot in 0..2 {ensure!(cooling.visible_essence(Some(slot))==Some(cooling.essence_position()),
        "Completed legacy cooldown stayed invisible to slot{slot}");}
    println!("PASS Funhouse legacy cooldown becomes visible to both players after expiry");
    for first in 0..2 {
        let mut o=fixture(a,map)?;o.saved.essence=true;
        let mut p=[Player::new(vec3(10000.,0.,0.)),Player::new(vec3(10000.,0.,0.))];
        let mut s=std::array::from_fn(|_|Stats::for_level("funhouse",None));
        s[0].set_health(40.)?;s[1].set_health(50.)?;
        let at=o.essence_position();p[first]=Player::new(at-Vec3::Z*32.);
        let w=World::fixture(&[]);let old=s[first].sanity();let other=serde_json::to_value(&s[1-first])?;
        o.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut p,&mut s));
        ensure!(s[first].sanity()==old+25. && serde_json::to_value(&s[1-first])?==other,"Essence shared personal reward");
        ensure!(o.visible_essence(Some(first)).is_none(),"Collected essence remains visible to collector");
        ensure!(o.visible_essence(Some(1-first))==Some(at),"Partner lost own essence");
        o=restored(a,map,&o)?;let state=o.snapshot();let resources=serde_json::to_value(&s)?;
        for dt in [0.,-1.,f32::NAN] {o.coop_combat_roster(dt,&w,&mut slots(&mut p,&mut s));}
        ensure!(o.snapshot()==state && serde_json::to_value(&s)?==resources,"Paused battle advanced healing");
        p[first]=Player::new(vec3(10000.,0.,0.));p[1-first]=Player::new(at-Vec3::Z*32.);
        let hp=s[1-first].sanity();o.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut p,&mut s));
        ensure!(s[1-first].sanity()==hp+25.,"Partner could not collect own essence");
        let saved=o.snapshot();
        ensure!((saved["coop"]["wait"][first].as_f64().unwrap()-(10.-f64::from(FIXED_DT))).abs()<0.00001,
            "Personal cooldown advanced more than once");
        for stat in &mut s {stat.damage(1000.);}
        let before=o.snapshot();o.coop_combat_roster(0.1,&w,&mut slots(&mut p,&mut s));
        ensure!(o.snapshot()==before,"Dead team advanced battle");
        println!("PASS Funhouse personal healing first{first}, local presentation, pause, restore and dead-team freeze");
    }
    Ok(())
}
fn continuation(a:&mut Assets,map:&Bsp)->Result<()> {
    let mut o=fixture(a,map)?;let w=floor(false);
    let mut p=[Player::new(vec3(250.,0.,0.03125)),Player::new(vec3(0.,550.,0.03125))];
    let mut s=std::array::from_fn(|_|Stats::for_level("funhouse",None));
    for boss in &mut o.saved.bosses {boss.action=battle::Action::Rattle;}
    o.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut p,&mut s));
    let mut q=restored(a,map,&o)?;let mut qp=p.clone();let mut qs=s.clone();
    for _ in 0..480 {
        let left=o.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut p,&mut s));
        let right=q.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut qp,&mut qs));
        ensure!(summary(&left)==summary(&right) && o.snapshot()==q.snapshot(),"Saved paired battle diverged");
    }
    println!("PASS Funhouse saved committed targets, grenades and exact battle continuation");
    for survivor in [None,Some(0),Some(1)] {
        let mut o=fixture(a,map)?;let mut p=p.clone();let mut s=s.clone();
        for b in &mut o.saved.bosses {b.hit(Hit{id:b.id,damage:2000.,kind:DamageKind::Ice,knockback:Vec3::ZERO});}
        for _ in 0..601 {o.coop_combat_roster(FIXED_DT,&w,&mut slots(&mut p,&mut s));}
        ensure!(o.saved.scene.is_none() && o.saved.death_wait>=5.,"Victory started before contact/damage phase");
        for (i,s) in s.iter_mut().enumerate() {if Some(i)!=survivor {s.damage(1000.);}}
        let before=o.snapshot();let actor=o.coop_contacts(FIXED_DT,&mut slots(&mut p,&mut s));
        ensure!(actor==survivor,"Victory selected a dead player");
        if survivor.is_none() {ensure!(o.snapshot()==before,"Dead team started a victory scene");}
        else {
            ensure!(o.saved.scene.as_ref().is_some_and(|s|s.kind==Kind::Hatter),"Original Hatter scene missing");
            let mut scene_world=World::from_bsp(map)?;
            o.finish_scene(map,&mut scene_world,&mut p[survivor.unwrap()])?;
            ensure!(o.saved.exit.committed && !o.saved.fight,"Victory did not finish");
            restored(a,map,&o)?;
            ensure!((0..2).all(|slot|o.visible_essence(Some(slot)).is_none()),"Healing stayed visible after victory");
        }
        println!("PASS Funhouse post-damage victory owner {survivor:?}");
    }
    Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/funhouse.bsp")?)?;let native=Funhouse::load(a,&map)?;
    targeting(&native.data)?;attacks(&native.data)?;
    healing_and_saves(a,&map)?;continuation(a,&map)?;
    println!("PASS Funhouse paired battle components; machinery, scenes and full traversal remain separate checks");
    Ok(())
}
