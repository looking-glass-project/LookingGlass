//! Native components only: full paired traversal and internet coverage remain separate.
use super::*;
use crate::{coop::{protocol::Visit,stage::Stage,world::Roster,PlayerId},
    level::CoopPlayer,movement::{Controls,FIXED_DT},powerups::Difficulty};

fn setup(a:&mut Assets,map:&Bsp)->Result<(Garden,World)> {
    let g=Garden::load(a,map)?;
    let mut w=World::from_bsp(map)?; w.set_dynamic(g.colliders());
    Ok((g,w))
}
fn players()->([Player;2],[Stats;2]) {
    ([Player::new(vec3(0.,0.,16000.)),Player::new(vec3(100.,0.,16000.))],
        std::array::from_fn(|_|Stats::for_level("garden4",Some("garden4_start1"))))
}
fn tick(g:&mut Garden,map:&Bsp,w:&mut World,p:&mut [Player;2],s:&mut [Stats;2],dt:f32)->Result<()> {
    let [p0,p1]=p.each_mut(); let [s0,s1]=s.each_mut();
    g.coop_advance_roster(dt,map,w,&mut [CoopPlayer{player:p0,stats:s0},CoopPlayer{player:p1,stats:s1}],&[])
}
fn shared(g:&Garden)->serde_json::Value {
    let mut s=g.snapshot();
    let c=s["course"].as_object_mut().unwrap();
    c.remove("previous"); c.remove("coop_previous"); s
}
fn clocks(a:&mut Assets,map:&Bsp)->Result<()> {
    for hz in [30,60,144] {
        let (mut g,mut w)=setup(a,map)?; let (mut single,mut sw)=setup(a,map)?;
        let (mut p,mut s)=players(); let mut sp=p[0].clone();
        for n in ["Ice_Fall1","Ice_Fall2","Ice_Fall3"] {g.event(n);single.event(n);}
        for n in 0..hz*3 {
            tick(&mut g,map,&mut w,&mut p,&mut s,1./hz as f32)?;
            single.advance(1./hz as f32,map,&mut sw,&mut sp,&[])?;
            ensure!(shared(&g)==shared(&single) && g.transforms()==single.transforms(),"Paired ice clock differs {hz}Hz tick{n}");
            if n==hz {
                let saved=g.snapshot(); let bodies=serde_json::to_value((&p,&s))?;
                for dt in [0.,-1.,f32::NAN] {tick(&mut g,map,&mut w,&mut p,&mut s,dt)?;}
                ensure!(g.snapshot()==saved && serde_json::to_value((&p,&s))?==bodies,"Paused ice changed");
                let mut restored=Garden::load(a,map)?;restored.restore(&saved,map)?;
                ensure!(restored.snapshot()==saved,"Paired ice save changed");g=restored;
            }
        }
        println!("PASS Icy Reception {hz}Hz paired clocks: single-player parity, pause and saved continuation");
    }
    for difficulty in Difficulty::ALL {
        let mut map=Bsp::parse(&a.read("maps/garden4.bsp")?)?;map.difficulty=difficulty;
        let (mut g,mut w)=setup(a,&map)?; let (mut single,mut sw)=setup(a,&map)?;
        let (mut p,mut s)=players(); let mut sp=p[0].clone();
        for name in ["Ice_Fall1","Ice_Fall2","Ice_Fall3","Garden4_Boulder_End"] {g.event(name);single.event(name);}
        for _ in 0..120*40 {
            tick(&mut g,&map,&mut w,&mut p,&mut s,FIXED_DT)?;
            single.advance(FIXED_DT,&map,&mut sw,&mut sp,&[])?;
        }
        ensure!(g.course().crushed[0].is_some()==!g.data.easy && g.course().crushed[1].is_some()==!g.data.easy
            && g.course().crushed[2..].iter().all(Option::is_some) && g.course().wall.is_some(),"Missing paired collapse/wall callback");
        ensure!(g.objects.iter().filter(|o|o.name.starts_with("brokenwall")).all(|o|!o.solid && o.pose.is_none()),"Paired wall stayed closed");
        for ctl in [&mut g,&mut single] {ctl.output(&crate::event::Effect::Trigger(crate::entity::Id(78)));}
        tick(&mut g,&map,&mut w,&mut p,&mut s,FIXED_DT)?;
        single.advance(FIXED_DT,&map,&mut sw,&mut sp,&[])?;
        ensure!(shared(&g)==shared(&single) && g.transforms()==single.transforms()
            && g.objects.iter().any(|o|o.name=="block_back" && o.solid),"Paired geometry/fog differs from single-player");
        println!("PASS Icy Reception {difficulty:?}: shared native floor/wall callbacks and fog match single-player");
    }
    Ok(())
}
fn contacts(a:&mut Assets,map:&Bsp)->Result<()> {
    for k in 0..7 {for victim in 0..2 {for dead_partner in [false,true] {
        let (mut g,mut w)=setup(a,map)?;let (mut p,mut s)=players();
        g.activate_rock(k);
        // Wait using the native clock until the chosen rock actually moves.
        for _ in 0..120*10 {
            if g.course().rocks[k].velocity.length()>0. {break;}
            tick(&mut g,map,&mut w,&mut p,&mut s,FIXED_DT)?;
        }
        ensure!(g.course().rocks[k].velocity.length()>0.,"Rock {k} never started");
        p[victim].feet=g.course().rocks[k].position-crate::collision::PLAYER_CENTER;
        if dead_partner {p[1-victim].feet=p[victim].feet;s[1-victim].damage(10000.);}
        let other=serde_json::to_value((&p[1-victim],&s[1-victim]))?;
        let initial=serde_json::to_value((&p[victim],&s[victim]))?;
        let mut expected=s[victim].clone();
        if !matches!(k,2|3|4) {expected.damage(g.data.rocks[k].damage);}
        tick(&mut g,map,&mut w,&mut p,&mut s,FIXED_DT)?;
        if matches!(k,2|3|4) {
            ensure!(serde_json::to_value((&p[victim],&s[victim]))?==initial,"Harmless rock hit Alice");
        } else {
            ensure!(p[victim].velocity.length()>0. && serde_json::to_value(&s[victim])?==serde_json::to_value(expected)?,
                "Rock {k} missed or applied incorrect native damage to living slot{victim}");
        }
        ensure!(serde_json::to_value((&p[1-victim],&s[1-victim]))?==other,"Rock hit the wrong slot");
        ensure!(g.course().damage==0. && g.course().impulse==Vec3::ZERO,"Paired hit leaked into single-player feedback");
        let saved=g.snapshot();let mut restored=Garden::load(a,map)?;restored.restore(&saved,map)?;
        ensure!(restored.snapshot()==saved,"Saved paired rock contact changed");
        println!("PASS Icy Reception rock{k} victim{victim} dead_partner={dead_partner}: personal contact/harmlessness and save");
    }}}
    Ok(())
}
fn altar(a:&mut Assets)->Result<()> {
    let mut map=Bsp::parse(&a.read("maps/garden4.bsp")?)?;map.difficulty=Difficulty::Easy;
    for arriving in 0..2 {
        let (mut g,mut w)=setup(a,&map)?;let (mut p,mut s)=players();
        let center=g.data.easy_altar.interior_point().unwrap()-crate::collision::PLAYER_CENTER;
        p[arriving].feet=center-Vec3::X*200.;
        tick(&mut g,&map,&mut w,&mut p,&mut s,FIXED_DT)?;
        let before=g.snapshot();
        g.restore(&before,&map)?;
        p[arriving].feet=center+Vec3::X*200.;
        tick(&mut g,&map,&mut w,&mut p,&mut s,FIXED_DT)?;
        ensure!(g.course().altar.is_some() && g.course().rocks[2].started,"Easy altar missed saved slot{arriving} sweep");
        let at=g.course().altar;
        for _ in 0..240 {tick(&mut g,&map,&mut w,&mut p,&mut s,FIXED_DT)?;}
        ensure!(g.course().altar==at && g.course().rocks[3].started,"Altar restarted or omitted second rock");
        g.coop_respawned(arriving);
        ensure!(g.course().coop_previous.unwrap()[arriving].is_none(),"Respawn retained old altar sweep");
        let saved=g.snapshot();let mut old=saved.clone();old["course"].as_object_mut().unwrap().remove("coop_previous");
        let mut restored=Garden::load(a,&map)?;restored.restore(&old,&map)?;
        ensure!(shared(&restored)==shared(&g) && restored.course().coop_previous.is_none(),"Older ice save lost progress");
        let mut bad=saved;bad["course"]["coop_previous"]=serde_json::json!([[100001.,0.,0.],null]);
        let valid=restored.snapshot();
        ensure!(restored.restore(&bad,&map).is_err() && restored.snapshot()==valid,"Invalid paired contact mutated save");
        println!("PASS Icy Reception Easy altar slot{arriving}: swept saved contact, one shared activation, respawn and old-save compatibility");
    }
    let (mut g,mut w)=setup(a,&map)?;let (mut p,mut s)=players();
    p[0].feet=g.data.easy_altar.interior_point().unwrap()-crate::collision::PLAYER_CENTER;
    s[0].damage(10000.);tick(&mut g,&map,&mut w,&mut p,&mut s,FIXED_DT)?;
    ensure!(g.course().altar.is_none(),"Dead player activated Easy altar");
    println!("PASS Icy Reception Easy altar ignores dead bodies");
    Ok(())
}
fn owner(stage:&mut Stage)->&mut Garden {
    stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Garden>()).unwrap()
}
fn scenes(a:&mut Assets,map:&Bsp)->Result<()> {
    let visit=Visit{epoch:1,map:"garden4".into(),entry:Some("garden4_start1".into()),difficulty:Default::default(),combat_preview:false};
    for marble in [true,false] {for first in PlayerId::ALL {for skip in [false,true] {
        if marble && skip {continue;}
        let mut w=World::from_bsp(map)?;let mut stage=Stage::load(a,map,&mut w,&visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
        let mut roster=Roster::new(&w,eye,yaw,Stats::for_level("garden4",visit.entry.as_deref()))?;
        roster.players[1].stats.spend_will(17.);
        let resources=roster.players.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
        if marble {
            let e=map.entities.iter().find(|e|e.get("thread").is_some_and(|n|n==course::MARBLE)).context("Missing marble trigger")?;
            let model=e["model"].trim_start_matches('*').parse()?;let pose=data::at(e);
            let at=Collider::model(map,model,pose.translation,pose.rotation,false)?.interior_point().context("Empty marble trigger")?
                -crate::collision::PLAYER_CENTER;
            roster.players[first.index()].player=Player::new(at);
            let before=roster.players.each_ref().map(|b|b.player.feet);
            stage.contacts(&mut roster,&w,before)?;
        } else {
            roster.players[first.index()].player=Player::spawn(&w,vec3(-1632.,5120.,-4768.)).context("Caterpillar approach blocked")?;
            for _ in 0..240 {
                let before=roster.players.each_ref().map(|b|b.player.feet);
                roster.players[first.index()].player.tick(&w,Controls{wish:Vec2::Y,..Default::default()});
                stage.contacts(&mut roster,&w,before)?;
                if stage.owner.is_some(){break;}
            }
        }
        ensure!(stage.owner==Some(first),"Wrong Icy Reception scene owner marble{marble} {first:?}");
        let mut waiting=roster.players[first.partner().index()].player.clone();waiting.release_rope();
        let waiting=serde_json::to_value(waiting)?;
        for n in 0..120*300 {
            if n==12 {
                let snap=stage.snapshot();let bodies=serde_json::to_value(&roster.players)?;
                stage.before_move(&mut roster,map,&mut w,&Default::default(),0.)?;
                let bytes=crate::coop::codec::encode(&serde_json::to_vec(&snap)?)?;
                stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,&mut w)?;
                ensure!(serde_json::to_value(stage.snapshot())?==serde_json::to_value(snap)? && serde_json::to_value(&roster.players)?==bodies,"Paused/restored ice scene changed");
            }
            if n==24 && (skip || marble) {
                let changed=stage.interactions.skip_cinematic(map,&mut w,&mut roster.players[first.index()].player,&mut stage.story)?;
                ensure!(changed==!marble,"Marble skip/closing skip contract changed");
            }
            stage.scene_step(&mut roster,map,&mut w,&Default::default())?;
            if stage.owner.is_none(){break;}
            ensure!(stage.owner==Some(first) && serde_json::to_value(&roster.players[first.partner().index()].player)?==waiting,"Scene moved waiting partner");
        }
        ensure!(!stage.busy(),"Icy Reception scene never reunited players");
        for (b,s) in roster.players.iter().zip(&resources) {
            ensure!(w.body_clear(b.player.feet) && b.stats.alive() && serde_json::to_value(&b.stats)?==*s,"Unsafe ice reunion/resources");
            b.player.validate_world(&w)?;
        }
        ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Ice reunion overlapped");
        if marble {
            let rock=owner(&mut stage).course().rocks[6].position;
            ensure!(owner(&mut stage).course().marble_done && roster.players.iter().all(|b|b.player.feet.distance(rock)>240.),"Marble reunion lacks safe lead");
        } else {
            if skip {ensure!(!owner(&mut stage).ready(),"Skipped portal unlocked too early");}
            for _ in 0..120*5 {stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;}
            ensure!(owner(&mut stage).ready(),"Paired portal unlock timer froze");
            // The current walk speed needs over two seconds to cross this gap.
            for _ in 0..600 {
                let before=roster.players.each_ref().map(|b|b.player.feet);
                roster.players[first.index()].player.tick(&w,Controls{wish:Vec2::X,..Default::default()});
                stage.contacts(&mut roster,&w,before)?;
                if stage.transition.is_some(){break;}
            }
            ensure!(stage.transition.as_ref().is_some_and(|e|EXIT.matches(e)),"Paired portal did not transition");
        }
        println!("PASS Icy Reception scene marble={marble} {first:?} skip={skip}: original trigger, frozen partner, pause/save, resources and safe reunion/portal");
    }}}
    Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/garden4.bsp")?)?;
    clocks(a,&map)?;contacts(a,&map)?;altar(a)?;scenes(a,&map)?;
    println!("PASS Icy Reception 44 native co-op component cases; full paired traversal remains unverified");
    Ok(())
}
