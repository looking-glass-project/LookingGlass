//! Original arrival camera/story through the real paired Stage; no rendered claims.
use super::*;
use crate::coop::{protocol::{Input, Visit},stage::Stage,PlayerId};
fn tower(stage:&mut Stage)->&mut Tower {
    stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut()).unwrap()
}
fn checkpoint(stage:&mut Stage,r:&mut Roster,map:&Bsp,w:&mut World)->Result<()> {
    let saved=serde_json::to_value(stage.snapshot())?;
    let people=serde_json::to_value(&r.players)?;
    let camera=stage.camera(w).map(|c|(c.eye,c.target,c.up));
    let poses=stage.draw_transforms(w);
    for dt in [0.,-1.,f32::NAN] {tower(stage).advance_pair(dt,map,w,&mut roster(r),&[])?;}
    ensure!(serde_json::to_value(stage.snapshot())?==saved && serde_json::to_value(&r.players)?==people,"Paused Tower scene changed");
    let compressed=crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
    stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&compressed)?)?,map,w)?;
    ensure!(serde_json::to_value(stage.snapshot())?==saved && poses==stage.draw_transforms(w)
        && camera==stage.camera(w).map(|c|(c.eye,c.target,c.up)),"Tower scene restore changed camera, owner or machinery");
    Ok(())
}
pub(super) fn check(a:&mut Assets,map:&Bsp)->Result<()> {
    for first in PlayerId::ALL {for skip in [None,Some(0.2),Some(3.)] {for dead in [false,true] {
        let visit=Visit {epoch:1,map:"tower2".into(),entry:Some("tower2_start1".into()),difficulty:Default::default(),combat_preview:false};
        let mut w=World::from_bsp(map)?;
        let mut stage=Stage::load(a,map,&mut w,&visit,true)?;
        let (eye,yaw)=interaction::spawn(map,visit.entry.as_deref());
        let mut r=Roster::new(&w,eye,yaw,Stats::for_level("tower2",None))?;
        r.players[0].stats.set_health(47.)?;r.players[1].stats.set_health(63.)?;
        let other=first.partner().index();if dead {r.players[other].stats.damage(1000.);}
        tower(&mut stage).prepare_pair(&mut roster(&mut r));
        let resources=serde_json::to_value(r.players.each_ref().map(|p|&p.stats))?;
        let home=r.players[first.index()].player.feet;let waiting=r.players[other].player.feet;
        stage.owner=Some(first);stage.accept(Events::default(),first,&mut r,&w);
        let mut skipped=false;let mut done=false;let mut saw_camera=false;
        for tick in 0..120*60 {
            let just_restored=tick==12 || tick==360;
            if just_restored {checkpoint(&mut stage,&mut r,map,&mut w)?;}
            let mut input=[(Controls::default(),Input::default());2];
            input[other].1.skip_scene=true;
            // Loading deliberately clears local held-key intent. Release once
            // after restoration, then make the same fresh press a player would.
            input[first.index()].1.skip_scene=!just_restored && skip.is_some_and(|at|tick as f32*FIXED_DT>=at);
            stage.scene_step(&mut r,map,&mut w,&input)?;
            skipped|=stage.notices[first.index()].as_ref().is_some_and(|n|n.text=="Scene skipped");
            if stage.owner.is_none() {done=true;break;}
            ensure!(stage.owner==Some(first) && r.players[other].player.feet==waiting,"Tower intro changed owner or waiting partner");
            if let Some(scene)=&tower(&mut stage).saved.scene {
                ensure!(scene.clock.home.is_some_and(|p|p.translation==home),"Tower performance used the wrong Alice pose");
                ensure!(stage.camera(&w).is_some(),"Original Tower camera missing");saw_camera=true;
            }
        }
        ensure!(done && saw_camera && skipped==skip.is_some() && stage.story.has_seen(INTRO)
            && tower(&mut stage).saved.arrived && !tower(&mut stage).scripted(),
            "Tower introduction did not finish correctly: owner{first:?}, requested{skip:?}, dead{dead}, done{done}, skipped{skipped}, camera{saw_camera}");
        ensure!(serde_json::to_value(r.players.each_ref().map(|p|&p.stats))?==resources
            && r.players[first.index()].player.feet==home,"Tower scene changed resources or actor's supported home");
        if dead {
            ensure!(!r.players[other].stats.alive() && r.players[other].player.feet==waiting,"Tower scene revived/moved dead partner");
        } else {
            ensure!(r.players.iter().all(|p|w.body_clear(p.player.feet) && crate::recovery::supported(&w,p.player.feet))
                && r.players[0].player.feet.distance(r.players[1].player.feet)>=crate::coop::separation::DISTANCE-0.05,"Tower reunion embedded or overlapped Alice");
        }
        println!("PASS paired Tower intro owner{first:?} skip{skip:?} dead{dead}: original camera/home/story, pause/restore, frozen partner and safe reunion");
    }}}
    let mut failed=Vec::new();
    for first in PlayerId::ALL {for flusher in 0..3 {
        if let Err(error)=press(a,map,first,flusher) {
            println!("FAIL paired Tower physical flusher{flusher} actor{first:?}: {error:#}");
            failed.push((first,flusher));
        }
    }}
    ensure!(failed.is_empty(),"Physical flusher cases failed: {failed:?}");
    Ok(())
}

fn press(a:&mut Assets,map:&Bsp,first:PlayerId,flusher:usize)->Result<()> {
    let visit=Visit {epoch:1,map:"tower2".into(),entry:None,difficulty:Default::default(),combat_preview:false};
    let mut w=World::from_bsp(map)?;
    let mut stage=Stage::load(a,map,&mut w,&visit,true)?;
    let (eye,yaw)=interaction::spawn(map,None);
    let mut r=Roster::new(&w,eye,yaw,Stats::for_level("tower2",None))?;
    let t=tower(&mut stage);t.upgraded();
    for prev in 0..flusher {t.saved.stages[prev]=Some(8.);t.saved.machine.as_mut().unwrap().lids[prev]=if prev+1<flusher {90.}else{35.};}
    t.rebuild(map)?;
    stage.owner=None;stage.story.finish_sequence(INTRO);stage.interactions.sync_coop(&mut w);
    let goal=[vec3(800.,750.,896.),vec3(2056.,2144.,1168.),vec3(2304.,1343.,1488.)][flusher];
    // Route navigation goals are approximate. Start this isolated pressure
    // test on the actual tray, whose authored origin is far above its surface.
    let at=support_points(tower(&mut stage),map,&w,&format!("flusher0{}",flusher+1))?
        .into_iter().min_by(|a,b|a.distance_squared(goal).total_cmp(&b.distance_squared(goal)))
        .context("Native flusher start lacks supported clear footing")?;
    r.players[first.index()].player=Player::new(at);r.players[first.index()].player.grounded=true;
    let mut done=false;
    for tick in 0..120*12 {
        let input=Default::default();
        stage.before_move(&mut r,map,&mut w,&input,FIXED_DT)?;
        let before=r.players.each_ref().map(|p|p.player.feet);
        r.move_players(&w,input);stage.contacts(&mut r,&mut w,before)?;
        let feedback=tower(&mut stage).damage_pair(FIXED_DT,&mut roster(&mut r));
        for (p,hit) in r.players.iter_mut().zip(feedback.players) {p.stats.damage(hit.damage);}
        ensure!(r.players.iter().all(|p|p.stats.alive() && w.body_clear(p.player.feet)),"Native flusher course killed/embedded Alice");
        if tick==240 {checkpoint(&mut stage,&mut r,map,&mut w)?;}
        if tower(&mut stage).saved.stages[flusher].is_some_and(|t|t>=6.) {done=true;break;}
    }
    ensure!(done,"Native flusher{flusher} never raised water from actual contact, owner{first:?}, feet{:?}",r.players[first.index()].player.feet);
    ensure!((tower(&mut stage).height()-(624.+256.*(flusher+1)as f32)).abs()<0.01,"Native contact raised wrong stage");
    println!("PASS paired Tower physical flusher{flusher} actor{first:?}: normal body physics, real trigger, rising water and saved Stage");
    Ok(())
}
