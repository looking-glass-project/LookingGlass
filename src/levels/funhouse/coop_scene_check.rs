//! Staged native scene contracts; these are not full campaign traversal proof.
use super::*;
use crate::{coop::{protocol::Visit,stage::Stage,world::Roster,PlayerId},
    level::CoopPlayer,movement::{Controls,EYE_HEIGHT,FIXED_DT}};

fn ctl(stage:&mut Stage)->&mut Funhouse {
    stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut()).unwrap()
}
fn setup(a:&mut Assets,map:&Bsp,free:bool)->Result<(World,Stage,Roster)> {
    let visit=Visit{epoch:1,map:"funhouse".into(),entry:Some("funhouse_start1".into()),
        difficulty:Default::default(),combat_preview:false};
    let mut w=World::from_bsp(map)?;let mut stage=Stage::load(a,map,&mut w,&visit,true)?;
    let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
    let roster=Roster::new(&w,eye,yaw,Stats::for_level("funhouse",visit.entry.as_deref()))?;
    // Fixtures begin after the original entry resource grant. Each scene must
    // then preserve distinct resources and must never resurrect a dead partner.
    let o=ctl(&mut stage);o.saved.initialized=true;
    if free {o.saved.scene=None;o.saved.arrived=true;o.saved.cells=2;o.rebuild(map)?;stage.owner=None;}
    stage.interactions.sync_coop(&mut w);
    Ok((w,stage,roster))
}
fn resources(roster:&Roster)->Result<serde_json::Value> {
    Ok(serde_json::to_value(roster.players.each_ref().map(|p|&p.stats))?)
}
fn checkpoint(stage:&mut Stage,roster:&mut Roster,map:&Bsp,w:&mut World)->Result<()> {
    let before=serde_json::to_value(stage.snapshot())?;let people=serde_json::to_value(&roster.players)?;
    let camera=stage.camera(w).map(|c|(c.eye,c.target,c.up));
    let transforms=stage.draw_transforms(w);let fov=stage.scene_fovy(16./9.);
    let o=ctl(stage);let mut players=roster.players.each_mut().map(|b|CoopPlayer{player:&mut b.player,stats:&mut b.stats});
    for dt in [0.,-1.,f32::NAN] {
        o.coop_advance_roster(dt,map,w,&mut players,&[])?;o.coop_combat_roster(dt,w,&mut players);
    }
    ensure!(serde_json::to_value(stage.snapshot())?==before && serde_json::to_value(&roster.players)?==people,
        "Paused scene changed world or bodies");
    let bytes=crate::coop::codec::encode(&serde_json::to_vec(&before)?)?;
    stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,w)?;
    ensure!(serde_json::to_value(stage.snapshot())?==before && camera==stage.camera(w).map(|c|(c.eye,c.target,c.up))
        && transforms==stage.draw_transforms(w) && fov==stage.scene_fovy(16./9.),
        "Restored scene changed state, camera or mover poses");
    Ok(())
}
fn cinematic(a:&mut Assets,map:&Bsp,kind:Kind,first:PlayerId,skip:bool,dead:bool)->Result<()> {
    let (mut w,mut stage,mut roster)=setup(a,map,kind!=Kind::Arrival)?;
    roster.players[0].stats.set_health(54.)?;roster.players[1].stats.set_health(67.)?;
    let other=first.partner().index();
    if dead {roster.players[other].stats.damage(1000.);}
    if kind==Kind::Arrival {
        // Fresh entry is a simultaneous arrival, normally tied to Blue. Red
        // here is an explicit restored-owner capability fixture, not a route.
        let mut state=stage.snapshot();state.owner=Some(first);state.last_owner=first;
        stage.restore(&state,map,&mut w)?;
    } else {
        let o=ctl(&mut stage);o.saved.cells=255;o.saved.all_cells=Some(0.);o.saved.time=2.;
        o.saved.gas=kind!=Kind::Gas;
        let at=o.data.points["alice_pos1"].translation;
        if kind==Kind::Hatter {
            o.saved.fight=true;
            // Isolate the native post-battle gate; combat itself has a separate
            // paired check. Keep original death wait and scene machinery.
            for id in [BASE,BASE+1] {
                o.hit(Hit{id,damage:2000.,kind:crate::combat::DamageKind::Ice,knockback:Vec3::ZERO});
            }
            o.saved.death_wait=5.;
        }
        o.rebuild(map)?;stage.interactions.sync_coop(&mut w);
        roster.players[first.index()].player=Player::spawn(&w,at+Vec3::Z*EYE_HEIGHT).context("No scene fixture footing")?;
        if kind==Kind::Tweedles {
            let e=map.entities.iter().find(|e|e.get("thread").is_some_and(|s|s==kind.id())).context("Missing native scene trigger")?;
            let model=e.get("model").and_then(|s|s.strip_prefix('*')).context("Missing trigger brush")?.parse()?;
            let volume=Collider::model(map,model,data::pose(e).translation,Quat::IDENTITY,false)?;
            roster.players[first.index()].player=Player::new(volume.interior_point().context("Empty trigger")?-PLAYER_CENTER);
            let before=roster.players.each_ref().map(|p|p.player.feet);
            stage.contacts(&mut roster,&w,before)?;
        } else {
            stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
        }
    }
    ensure!(stage.owner==Some(first) && stage.scene_id()==Some(kind.id()),"Wrong native {kind:?} owner: {:?}",stage.owner);
    let waiting=roster.players[other].player.feet;let stats=resources(&roster)?;
    let mut skipped_at=None;let mut completed_at=None;let mut reunion_wait=0;
    for tick in 0..24000 {
        if tick==120 || (kind==Kind::Hatter && tick==660) {checkpoint(&mut stage,&mut roster,map,&mut w)?;}
        let mut input=[(Controls::default(),crate::coop::protocol::Input::default());2];
        input[other].1.skip_scene=true;input[first.index()].1.skip_scene=skip && tick>=240;
        stage.scene_step(&mut roster,map,&mut w,&input)?;
        if stage.notices[first.index()].as_ref().is_some_and(|n|n.text=="Scene skipped") {skipped_at.get_or_insert(tick);}
        if tick<240 {ensure!(stage.scene_id()==Some(kind.id()),"Partner skipped {kind:?}");}
        if stage.scene_id().is_some() {
            ensure!(stage.camera(&w).is_some(),"Cinematic lost its camera");
            ensure!(ctl(&mut stage).performances().iter().filter(|p|p.model=="alice").count()<=1,
                "Cinematic duplicates Alice's authored performance");
        }
        if !stage.busy() || stage.transition.is_some() {completed_at=Some(tick);break;}
        if stage.scene_id().is_none() && !stage.story.busy() {
            reunion_wait+=1;
            ensure!(reunion_wait<240,"Reunion stalled at {:?}, grounded {}, waiting {:?}",
                roster.players[first.index()].player.feet,roster.players[first.index()].player.grounded,waiting);
        }
        ensure!(roster.players[other].player.feet==waiting,"Scene moved waiting partner");
    }
    let completed_at=completed_at.context("Scene or reunion stalled")?;
    ensure!(skipped_at.is_some()==skip,"Wrong skip result");
    if kind==Kind::Hatter {
        ensure!(stage.transition.as_ref().is_some_and(|e|EXIT.matches(e)),"Hatter outro lost its departure");
        if let Some(tick)=skipped_at {ensure!(completed_at>=tick+779,"Skip bypassed physical floor/exit tail");}
        ensure!(ctl(&mut stage).saved.floor.is_some(),"Hatter floor did not collapse");
    } else {
        ensure!(!stage.busy() && stage.camera(&w).is_none(),"Scene did not release players");
        let actor=&roster.players[first.index()].player;
        // Original endpoints may be just above their landing or on a narrow
        // single-Alice stand. Verify the ordinary body trace can land there;
        // the newly placed partner still requires full-footprint support.
        let floor=w.body_trace(actor.feet,actor.feet-Vec3::Z*256.);
        ensure!(w.body_clear(actor.feet) && !floor.start_solid && floor.fraction<1. && floor.normal.z>=0.65,
            "Actor handoff is unsafe: {:?}",actor.feet);
        if !dead {
            let p=&roster.players[other].player;let d=p.feet-actor.feet;
            ensure!(w.body_clear(p.feet) && crate::recovery::supported(&w,p.feet)
                && !stage.interactions.hazardous(p.feet) && d.truncate().length()<=256. && d.z.abs()<=256.
                && (d.truncate().length()>=35.95 || d.z.abs()>=55.95),"Partner reunion is unsafe or distant");
        } else {ensure!(roster.players[other].player.feet==waiting,"Scene relocated dead partner");}
    }
    ensure!(resources(&roster)?==stats,"Scene changed personal resources");
    println!("PASS Funhouse {kind:?} {first:?} skip{skip} dead{dead}: staged ownership, frozen partner, saved camera, handoff ({completed_at} ticks)");
    Ok(())
}
fn jack(a:&mut Assets,map:&Bsp,first:PlayerId,dead:bool)->Result<()> {
    let (mut w,mut stage,mut roster)=setup(a,map,true)?;
    let catalog=crate::inventory::Catalog::load(a)?;
    let pickups=crate::inventory::pickups_for_visit(map,"funhouse",None,&catalog);
    let entity=map.entities.iter().position(|e|e.get("pickup_thread").is_some_and(|s|s==Kind::Jack.id())).context("Missing Jack pickup")?;
    let item=pickups.iter().find(|p|p.id==format!("funhouse:{entity}")).context("Missing personal Jacks")?;
    let other=first.partner().index();if dead {roster.players[other].stats.damage(1000.);}
    roster.players[first.index()].player=Player::spawn(&w,item.origin+Vec3::Z*EYE_HEIGHT).context("No Jack collector footing")?;
    let other_stats=serde_json::to_value(&roster.players[other].stats)?;
    stage.collect_pickups(&mut roster,&w,std::slice::from_ref(item));
    stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
    ensure!(roster.players[first.index()].stats.collected.contains(&item.id)
        && serde_json::to_value(&roster.players[other].stats)?==other_stats,"Jack pickup was not personal");
    ensure!(stage.scene_id()==Some(Kind::Jack.id()) && !stage.busy() && stage.camera(&w).is_none(),
        "Real Jacks pickup did not start ordinary speech");
    let stats=resources(&roster)?;let start=roster.players[first.index()].player.feet;
    for tick in 0..24000 {
        if tick==120 {checkpoint(&mut stage,&mut roster,map,&mut w)?;}
        let mut inputs=Default::default();
        if tick<60 {
            inputs=[(Controls::default(),crate::coop::protocol::Input::default());2];
            inputs[first.index()].0.wish=Vec2::X;
        }
        stage.before_move(&mut roster,map,&mut w,&inputs,FIXED_DT)?;
        roster.move_players(&w,inputs);
        stage.speech_step(&mut roster,&w,false);
        ensure!(!stage.busy() && stage.camera(&w).is_none(),"Jack speech froze exploration");
        if ctl(&mut stage).saved.jack {break;}
    }
    ensure!(ctl(&mut stage).saved.jack && roster.players[first.index()].player.feet.distance(start)>10.,
        "Jack speech failed to complete while exploring");
    ensure!(resources(&roster)?==stats,"Jack speech changed personal resources");
    // A living second collector gets a personal copy without replaying speech.
    if !dead {
        roster.players[first.index()].player=Player::new(start+vec3(200.,0.,0.));
        roster.players[other].player=Player::spawn(&w,item.origin+Vec3::Z*EYE_HEIGHT).context("No second Jack footing")?;
        stage.collect_pickups(&mut roster,&w,std::slice::from_ref(item));
        stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
        ensure!(roster.players.iter().all(|p|p.stats.collected.contains(&item.id)) && stage.scene_id().is_none(),
            "Second Jack collection replayed speech or lost its personal copy");
    }
    println!("PASS Funhouse Jack {first:?} dead{dead}: native personal pickup, ordinary exploration, saved speech and no replay");Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/funhouse.bsp")?)?;let mut failures=Vec::new();
    for kind in [Kind::Arrival,Kind::Gas,Kind::Tweedles,Kind::Hatter] {
        for first in PlayerId::ALL {for skip in [false,true] {for dead in [false,true] {
            if let Err(e)=cinematic(a,&map,kind,first,skip,dead) {
                let message=format!("{kind:?} {first:?} skip{skip} dead{dead}: {e:#}");eprintln!("FAIL {message}");failures.push(message);
            }
        }}}
    }
    for first in PlayerId::ALL {for dead in [false,true] {
        if let Err(e)=jack(a,&map,first,dead) {
            let message=format!("Jack {first:?} dead{dead}: {e:#}");eprintln!("FAIL {message}");failures.push(message);
        }
    }}
    ensure!(failures.is_empty(),"Funhouse paired scene failures:\n{}",failures.join("\n"));
    println!("PASS Funhouse paired scene components; full traversal, rendered effects and internet remain separate");Ok(())
}
