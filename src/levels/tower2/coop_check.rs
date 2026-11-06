//! Supplied-map paired machinery and personal hazard checks; no rendered claims.
use super::*;
use crate::{collision::{PLAYER_CENTER, PLAYER_HALF}, coop::world::Roster, level::CoopPlayer};
#[path = "coop_scene_check.rs"]
mod scenes;
#[path = "coop_safety_check.rs"]
mod safety;

fn roster(r: &mut Roster) -> [CoopPlayer<'_>; 2] {
    r.players.each_mut().map(|p| CoopPlayer { player: &mut p.player, stats: &mut p.stats })
}
fn fixture(a: &mut Assets, map: &Bsp) -> Result<(Tower, World, Roster)> {
    let mut t = Tower::load(a, map)?;
    t.upgraded();
    let mut w = World::from_bsp(map)?;
    t.traversal(&mut w.traversal);
    w.set_dynamic(t.colliders()); w.set_dynamic_liquids(t.liquids());
    let (eye, yaw) = interaction::spawn(map, None);
    let mut r = Roster::new(&w, eye, yaw, Stats::for_level("tower2", None))?;
    t.prepare_pair(&mut roster(&mut r));
    Ok((t, w, r))
}
fn step(t: &mut Tower, w: &mut World, r: &mut Roster, map: &Bsp, dt: f32) -> Result<()> {
    t.advance_pair(dt, map, w, &mut roster(r), &[])
}
fn restored(a: &mut Assets, map: &Bsp, t: &Tower) -> Result<Tower> {
    let data = crate::coop::codec::encode(&serde_json::to_vec(&t.snapshot())?)?;
    let mut copy = Tower::load(a, map)?;
    copy.restore(&serde_json::from_slice(&crate::coop::codec::decode(&data)?)?, map)?;
    ensure!(copy.snapshot() == t.snapshot(), "Tower compressed state changed");
    Ok(copy)
}
fn support_points(t: &Tower, map: &Bsp, w: &World, name: &str) -> Result<Vec<Vec3>> {
    let o = t.objects.iter().find(|o| o.name == name).context("Missing fixture brush")?;
    let mut found = Vec::<Vec3>::new();
    for x in (-96..=96).step_by(16) { for y in (-96..=96).step_by(16) {
        let high = o.base + vec3(x as f32, y as f32, map.models[o.model].min.z + 180.);
        let low = high - Vec3::Z * 220.;
        let tr = o.collider.trace(high + PLAYER_CENTER, low + PLAYER_CENTER, PLAYER_HALF);
        if !tr.start_solid && tr.fraction < 1. && tr.normal.z >= 0.65 {
            let at = high.lerp(low, tr.fraction);
            if w.body_clear(at) && found.iter().all(|p| p.distance(at) >= 48.) { found.push(at); }
        }
    }}
    Ok(found)
}
fn feet_on(t: &Tower, map: &Bsp, w: &World, name: &str) -> Result<[Vec3; 2]> {
    let found=support_points(t,map,w,name)?;
    ensure!(found.len() >= 2, "No two clear riders on {name}");
    Ok([found[0], found[1]])
}
fn place(r: &mut Roster, feet: [Vec3; 2]) {
    for (p, feet) in r.players.iter_mut().zip(feet) {
        p.player = Player::new(feet); p.player.grounded = true;
    }
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/tower2.bsp")?)?;
    clocks(a, &map)?;
    migration(a, &map)?;
    trays(a, &map)?;
    blocked_trays(a, &map)?;
    lids(a, &map)?;
    fans(a, &map)?;
    safety::check(a, &map)?;
    currents(a, &map)?;
    for first in 0..2 { swim(a, &map, first)?; }
    scenes::check(a, &map)
}
fn clocks(a: &mut Assets, map: &Bsp) -> Result<()> {
    for hz in [30, 60, 144] {
        let (mut t, mut w, mut r) = fixture(a, map)?;
        let mut solo = Tower::load(a, map)?; solo.upgraded();
        let mut sw = World::from_bsp(map)?;
        let mut p = r.players[0].player.clone();
        for stage in 0..3 {
            t.event(FLUSH[stage]); solo.event(FLUSH[stage]);
            // Each Alice's Watch freezes water/lids/fans without stopping weighted trays.
            for owner in 0..2 {
                r.players[owner].stats.powers.stopped = 2.; solo.stopped = true;
                for _ in 0..hz {
                    step(&mut t, &mut w, &mut r, map, 1. / hz as f32)?;
                    solo.advance(1. / hz as f32, map, &mut sw, &mut p, &[])?;
                }
                r.players[owner].stats.powers.stopped = 0.; solo.stopped = false;
            }
            for _ in 0..hz * 8 {
                step(&mut t, &mut w, &mut r, map, 1. / hz as f32)?;
                solo.advance(1. / hz as f32, map, &mut sw, &mut p, &[])?;
            }
            let mut paired = t.snapshot(); paired.as_object_mut().unwrap().remove("coop");
            ensure!(paired == solo.snapshot(), "Shared Tower clocks differ from solo at {hz}Hz stage{stage}");
            let surface = 624. + 256. * (stage + 1) as f32;
            ensure!((t.height() - surface).abs() < 0.01
                && w.liquid_at(vec3(1776., 2400., surface - 2.)) != 0
                && w.liquid_at(vec3(1776., 2400., surface + 2.)) == 0, "Shared water surface/collision differs");
            t = restored(a, map, &t)?;
        }
        ensure!(t.exit_ready(), "Shared final shaft gate stayed closed");
        let saved = t.snapshot(); let people = serde_json::to_value(&r.players)?;
        for dt in [0., -1., f32::NAN, f32::INFINITY] { step(&mut t, &mut w, &mut r, map, dt)?; }
        ensure!(t.snapshot() == saved && serde_json::to_value(&r.players)? == people, "Paused/invalid Tower step mutated state");
        for p in &mut r.players { p.stats.damage(1000.); }
        step(&mut t, &mut w, &mut r, map, FIXED_DT)?;
        ensure!(t.snapshot() == saved, "All-dead Tower advanced");
        println!("PASS paired Tower {hz}Hz: three ordered rises, both Watches, solo clock parity, pause and compressed restore");
    }
    Ok(())
}
fn migration(a: &mut Assets, map: &Bsp) -> Result<()> {
    let (mut t, mut w, mut r) = fixture(a, map)?;
    t.restore(&serde_json::json!({"version":1,"stages":[8.,2.5,null]}), map)?;
    t.prepare_pair(&mut roster(&mut r));
    ensure!(t.saved.arrived && !t.scripted() && t.height() == 1008., "Legacy paired import replayed intro or lost water");
    t = restored(a, map, &t)?;
    let mut solo = t.snapshot(); solo.as_object_mut().unwrap().remove("coop");
    solo["machine"]["currents"] = serde_json::json!([0.1,0.05,0.15,0.2]);
    solo["machine"]["crush"] = true.into();
    t.restore(&solo, map)?; t.prepare_pair(&mut roster(&mut r));
    ensure!(t.saved.coop.as_ref().unwrap().currents == [[0.1,0.05,0.15,0.2];2]
        && t.saved.coop.as_ref().unwrap().crush == [true,false] && !t.machine().crush,
        "Solo cooldown/hit import changed ownership");
    let saved=t.snapshot();
    for bad in [0.201,-0.1,999.] {
        let mut invalid=saved.clone(); invalid["coop"]["currents"][1][2]=bad.into();
        ensure!(t.restore(&invalid,map).is_err() && t.snapshot()==saved,"Bad current state accepted/mutated live state");
    }
    let mut invalid=saved.clone();invalid["machine"]=serde_json::Value::Null;
    ensure!(t.restore(&invalid,map).is_err() && t.snapshot()==saved,"Paired state accepted missing machinery");
    let paused=t.damage_pair(0.,&mut roster(&mut r));
    ensure!(paused.players.iter().all(|p|p.damage==0.) && t.snapshot()==saved,"Pause consumed pending fan damage");
    let f=t.damage_pair(FIXED_DT,&mut roster(&mut r));
    ensure!(f.players[0].damage==1000. && f.players[1].damage==0.,"Imported hit reached wrong player");
    t.coop_respawned(0);
    ensure!(t.saved.coop.as_ref().unwrap().currents[0]==[0.;4],"Revival retained stale current wait");
    // Personal air-source contact refills neither the absent nor the dead partner.
    for slot in 0..2 {
        r.players[slot].player.feet=t.data.bubbles[0]-PLAYER_CENTER;
        r.players[1-slot].player.feet=interaction::spawn(map,None).0;
        for p in &mut r.players {p.player.breath.submerged=12.;}
        t.prepare_pair(&mut roster(&mut r));
        ensure!(r.players[slot].player.breath.remaining()==20.
            && r.players[1-slot].player.breath.submerged==12.,"Air source refilled partner");
    }
    r.players[0].stats.damage(1000.);r.players[0].player.breath.submerged=12.;
    r.players[0].player.feet=t.data.bubbles[0]-PLAYER_CENTER;
    t.prepare_pair(&mut roster(&mut r));
    ensure!(r.players[0].player.breath.submerged==12. && !r.players[0].stats.alive(),"Air source revived/refilled dead Alice");
    step(&mut t,&mut w,&mut r,map,FIXED_DT)?;
    println!("PASS paired Tower migration: old solo stages, personal cooldowns/pending hit, atomic rejection, revival reset and individual air");
    Ok(())
}
fn trays(a: &mut Assets, map: &Bsp) -> Result<()> {
    for tray in 0..3 { for dead in [None,Some(0),Some(1)] {
        let (mut t,mut w,mut r)=fixture(a,map)?;
        let feet=feet_on(&t,map,&w,&format!("flusher0{}",tray+1))?;place(&mut r,feet);
        if let Some(slot)=dead {r.players[slot].stats.damage(1000.);}
        let live=if dead==Some(0){1}else{0};r.players[live].stats.powers.stopped=20.;
        for _ in 0..360 {step(&mut t,&mut w,&mut r,map,FIXED_DT)?;r.move_players(&w,Default::default());}
        ensure!((t.machine().sink[tray]-64.).abs()<0.01 && t.machine().fans==[0.;5],"Paired tray weight/Watch failed");
        for slot in 0..2 {
            let expected=if dead==Some(slot){feet[slot]}else{feet[slot]-Vec3::Z*64.};
            ensure!(r.players[slot].player.feet.distance(expected)<0.1,"Tray moved wrong body {tray}/{slot}");
        }
        let (mut copy,mut cw,mut cr)=fixture(a,map)?;copy.restore(&t.snapshot(),map)?;cr.players=r.players.clone();
        for _ in 0..20 {
            step(&mut t,&mut w,&mut r,map,FIXED_DT)?;r.move_players(&w,Default::default());
            step(&mut copy,&mut cw,&mut cr,map,FIXED_DT)?;cr.move_players(&cw,Default::default());
            ensure!(t.snapshot()==copy.snapshot() && serde_json::to_value(&r.players)?==serde_json::to_value(&cr.players)?,"Saved tray future differs");
        }
        let (_,_,home)=fixture(a,map)?;for slot in 0..2 {r.players[slot].player=home.players[slot].player.clone();}
        for _ in 0..960 {step(&mut t,&mut w,&mut r,map,FIXED_DT)?;}
        ensure!(t.machine().sink[tray]==0.,"Vacated paired tray did not spring back");
        println!("PASS paired Tower tray{tray} dead={dead:?}: native two-body carry, Watch, saved future and vacated spring");
    }}
    Ok(())
}
fn blocked_trays(a:&mut Assets,map:&Bsp)->Result<()> {
    for blocked in 0..2 {
        let(mut t,mut w,mut r)=fixture(a,map)?;
        let feet=feet_on(&t,map,&w,"flusher01")?;place(&mut r,feet);
        r.players[0].stats.powers.stopped=20.;
        let at=feet[blocked];let fixed=[Collider::box_bounds(at-vec3(8.,8.,5.),at+vec3(8.,8.,-4.))];
        for _ in 0..180 {t.advance_pair(FIXED_DT,map,&mut w,&mut roster(&mut r),&fixed)?;}
        let depth=t.machine().sink[0];
        ensure!(depth>0. && depth<5.,"Obstruction did not stop shared tray: {depth}");
        for slot in 0..2 {ensure!(r.players[slot].player.feet.distance(feet[slot]-Vec3::Z*depth)<0.1 && w.body_clear(r.players[slot].player.feet),"Obstructed tray split/embedded riders");}
        for _ in 0..360 {step(&mut t,&mut w,&mut r,map,FIXED_DT)?;}
        ensure!(t.machine().sink[0]==64.,"Released obstruction kept tray stuck");
        println!("PASS paired Tower blocked rider{blocked}: atomic rollback and ordinary release");
    }
    Ok(())
}
fn lids(a:&mut Assets,map:&Bsp)->Result<()> {
    for lid in 0..3 {
        let(mut t,mut w,mut r)=fixture(a,map)?;
        for prev in 0..lid {t.saved.stages[prev]=Some(8.);t.saved.machine.as_mut().unwrap().lids[prev]=35.;}
        t.rebuild(map)?;w.set_dynamic(t.colliders());w.set_dynamic_liquids(t.liquids());
        let feet=feet_on(&t,map,&w,&format!("fliptop{}",lid+1))?;place(&mut r,feet);
        t.event(FLUSH[lid]);
        for _ in 0..120 {step(&mut t,&mut w,&mut r,map,FIXED_DT)?;r.move_players(&w,Default::default());}
        ensure!(t.machine().lids[lid].abs()>3. && r.players.iter().enumerate().all(|(i,p)|p.player.feet.distance(feet[i])>1. && w.body_clear(p.player.feet)),"Rotating lid failed paired carry {lid}");
        t=restored(a,map,&t)?;
        step(&mut t,&mut w,&mut r,map,FIXED_DT)?;
        println!("PASS paired Tower lid{lid}: both native supported bodies rotate clear and restore");
    }
    Ok(())
}
fn fan_contact(t:&Tower,map:&Bsp,w:&World,k:usize)->Result<Vec3> {
    let o=t.objects.iter().find(|o|o.name==format!("fan0{}",k+1)).unwrap();
    let axis=[Vec3::X,Vec3::X,Vec3::Z,Vec3::Y,Vec3::Z][k];
    let angle=if k==1 || k==3 {-8_f32}else{8_f32};
    let next=Collider::model(map,o.model,o.base,Quat::from_axis_angle(axis,angle.to_radians()),true)?;
    for x in (-192..=192).step_by(16) {for y in (-224..=224).step_by(16) {for z in (-224..=224).step_by(16) {
        let feet=o.base+vec3(x as f32,y as f32,z as f32);let center=feet+PLAYER_CENTER;
        if w.body_clear(feet) && next.trace(center,center,PLAYER_HALF).start_solid {return Ok(feet);}
    }}}
    anyhow::bail!("No clear approach to native fan{k}")
}
fn fans(a:&mut Assets,map:&Bsp)->Result<()> {
    for fan in 0..5 {
        let(t,w,_)=fixture(a,map)?;let contact=fan_contact(&t,map,&w,fan)?;
        for victim in 0..2 {for dead in [false,true] {
            let(mut t,mut w,mut r)=fixture(a,map)?;
            r.players[victim].player=Player::new(contact);
            if dead {r.players[victim].stats.damage(1000.);r.players[victim].stats.powers.stopped=10.;}
            for _ in 0..15 {step(&mut t,&mut w,&mut r,map,FIXED_DT)?;}
            let mut copy=restored(a,map,&t)?;
            let f=t.damage_pair(FIXED_DT,&mut roster(&mut r));
            let g=copy.damage_pair(FIXED_DT,&mut roster(&mut r));
            for slot in 0..2 {
                let expected=if !dead && slot==victim {1000.}else{0.};
                ensure!(f.players[slot].damage==expected && g.players[slot].damage==expected,"Fan{fan} damaged wrong/restored body victim{victim} dead{dead}");
                r.players[slot].stats.damage(f.players[slot].damage);
            }
            ensure!(r.players[1-victim].stats.alive() && !r.players[victim].stats.alive(),"Fan death affected wrong Alice");
            ensure!(t.damage_pair(FIXED_DT,&mut roster(&mut r)).players.iter().all(|p|p.damage==0.),"Fan hit replayed");
            if dead {ensure!(t.machine().fans[fan]!=0.,"Dead body blocked fan");}
            println!("PASS paired Tower fan{fan} victim{victim} dead={dead}: swept native blade, personal lethal hit, restore and no repeat");
        }}
    }
    Ok(())
}
fn currents(a:&mut Assets,map:&Bsp)->Result<()> {
    for k in 0..4 {for first in 0..2 {
        let(mut t,mut w,mut r)=fixture(a,map)?;
        let center=t.currents[k].volume.interior_point().context("Current lacks interior")?;
        let feet=center-PLAYER_CENTER;
        let homes=r.players.each_ref().map(|p|p.player.clone());
        r.players[first].player=Player::new(feet);
        step(&mut t,&mut w,&mut r,map,FIXED_DT)?;
        let expected=t.currents[k].speed;
        ensure!((r.players[first].player.velocity.dot(t.currents[k].direction)-expected).abs()<0.01,"Current did not push first body");
        r.players[first].player=homes[first].clone();r.players[1-first].player=Player::new(feet);
        step(&mut t,&mut w,&mut r,map,FIXED_DT)?;
        ensure!((r.players[1-first].player.velocity.dot(t.currents[k].direction)-expected).abs()<0.01,"First body's wait suppressed second current");
        let timers=t.saved.coop.as_ref().unwrap().currents;
        ensure!((timers[first][k]-(0.2-FIXED_DT)).abs()<0.0001 && timers[1-first][k]==0.2,"Personal current waits not independent");
        r.players[1-first].player.velocity=Vec3::ZERO;
        step(&mut t,&mut w,&mut r,map,FIXED_DT)?;
        ensure!(r.players[1-first].player.velocity==Vec3::ZERO,"Current pulse applied every tick");
        t=restored(a,map,&t)?;
        ensure!(t.saved.coop.as_ref().unwrap().currents[1-first][k]<0.2,"Restore reset current wait");
        r.players[1-first].stats.damage(1000.);r.players[1-first].player.velocity=Vec3::ZERO;
        step(&mut t,&mut w,&mut r,map,0.1)?;
        ensure!(r.players[1-first].player.velocity==Vec3::ZERO,"Current moved dead Alice");
        println!("PASS paired Tower current{k} first{first}: independent 0.2-second native pulses and compressed wait");
    }}
    Ok(())
}
fn swim(a:&mut Assets,map:&Bsp,first:usize)->Result<()> {
    let(mut t,mut w,mut r)=fixture(a,map)?;
    let at=vec3(2930.,400.,-220.);
    let other=at-Vec3::Y*64.;
    ensure!(w.body_clear(at) && w.body_clear(other),"Paired drain entrance blocked");
    r.players[first].player=Player::new(at);r.players[1-first].player=Player::new(other);
    let input=[(Controls {swim:Vec3::Y,run:true,..Default::default()},Default::default());2];
    let mut resumed=None;
    for n in 0..360 {
        step(&mut t,&mut w,&mut r,map,FIXED_DT)?;r.move_players(&w,input);
        let feedback=t.damage_pair(FIXED_DT,&mut roster(&mut r));
        ensure!(feedback.players.iter().all(|p|p.damage==0.) && r.players.iter().all(|p|p.stats.alive() && w.body_clear(p.player.feet)),"Paired drain crossing hit a blade or solid");
        if n==80 {
            let(mut copy,cw,mut cr)=fixture(a,map)?;copy.restore(&t.snapshot(),map)?;cr.players=r.players.clone();cr.tick=r.tick;
            resumed=Some((copy,cw,cr));
        } else if let Some((copy,cw,cr))=&mut resumed {
            step(copy,cw,cr,map,FIXED_DT)?;cr.move_players(cw,input);copy.damage_pair(FIXED_DT,&mut roster(cr));
            ensure!(copy.snapshot()==t.snapshot() && serde_json::to_value(&cr.players)?==serde_json::to_value(&r.players)?,"Saved paired swimming diverged");
        }
        if r.players.iter().all(|p|p.player.feet.y>740.) {break;}
    }
    ensure!(r.players.iter().all(|p|p.player.feet.y>740. && p.player.breath.remaining()>15.),"Both swimmers did not clear live drain: {:?}",r.players.each_ref().map(|p|p.player.feet));
    println!("PASS paired Tower ordinary swimming first{first}: both bodies clear live currents/fans with air and exact restored future");
    Ok(())
}
