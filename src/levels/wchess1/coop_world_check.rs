//! Native paired machinery and original puzzle inputs; not a full campaign route.
use super::*;
use crate::{coop::{protocol::{Input,Visit},stage::Stage,world::Roster,PlayerId},
    level::CoopPlayer,movement::FIXED_DT};
fn slots<'a>(p:&'a mut [Player;2],s:&'a mut [Stats;2])->[CoopPlayer<'a>;2] {
    let [a,b]=p.each_mut();let [x,y]=s.each_mut();
    [CoopPlayer{player:a,stats:x},CoopPlayer{player:b,stats:y}]
}
fn free(a:&mut Assets,map:&Bsp)->Result<(Realm,World,[Player;2],[Stats;2])> {
    let mut r=Realm::load(a,map)?;r.saved.pending=None;r.saved.intro=true;
    let mut w=World::from_bsp(map)?;w.set_dynamic(r.colliders());
    let mut p=[Player::new(vec3(0.,0.,16000.)),Player::new(vec3(100.,0.,16000.))];
    let mut s=std::array::from_fn(|_|Stats::for_level("wchess1",None));
    r.coop_prepare(&mut slots(&mut p,&mut s));Ok((r,w,p,s))
}
fn tick(r:&mut Realm,map:&Bsp,w:&mut World,p:&mut [Player;2],s:&mut [Stats;2],dt:f32)->Result<()> {
    r.coop_advance_roster(dt,map,w,&mut slots(p,s),&[])
}
fn roundtrip(a:&mut Assets,map:&Bsp,r:&Realm)->Result<Realm> {
    let bytes=crate::coop::codec::encode(&serde_json::to_vec(&r.snapshot())?)?;
    let mut q=Realm::load(a,map)?;q.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map)?;
    ensure!(q.snapshot()==r.snapshot(),"Paired world changed across compressed restore");Ok(q)
}
fn clocks(a:&mut Assets,map:&Bsp)->Result<()> {
    for hz in [30,60,144] {
        let (mut r,mut w,mut p,mut s)=free(a,map)?;
        let mut sp=Realm::load(a,map)?;sp.saved.pending=None;sp.saved.intro=true;
        let mut sw=World::from_bsp(map)?;let mut actor=p[0].clone();
        for x in [&mut r,&mut sp] {
            x.saved.bishop_done=true;x.saved.knight_gate=true;x.saved.knight_opened=Some(0.);
            x.saved.water=Some(0.);x.saved.spikes=Some(0.);x.saved.secret=true;
        }
        for n in 0..hz*9 {
            tick(&mut r,map,&mut w,&mut p,&mut s,1./hz as f32)?;
            sp.advance(1./hz as f32,map,&mut sw,&mut actor,&[])?;
            let mut shared=r.snapshot();shared.as_object_mut().unwrap().remove("coop");
            ensure!(shared==sp.snapshot() && r.transforms()==sp.transforms(),"World clock diverged {hz}Hz tick{n}");
            if n==hz {
                let saved=r.snapshot();let bodies=serde_json::to_value((&p,&s))?;
                for dt in [0.,-1.,f32::NAN] {tick(&mut r,map,&mut w,&mut p,&mut s,dt)?;}
                ensure!(saved==r.snapshot() && bodies==serde_json::to_value((&p,&s))?,"Paused world changed");
                r=roundtrip(a,map,&r)?;
            }
        }
        ensure!(r.water_ready(),"Shared water never rose");
        println!("PASS paired Pale Realm {hz}Hz: native gates/water/spikes/lift/clock parity, pause and restore");
    }Ok(())
}
fn lift(a:&mut Assets,map:&Bsp)->Result<()> {
    for missing in 0..2 {
        let (mut r,mut w,mut p,mut s)=free(a,map)?;
        r.saved.elevator=motion::Lift{phase:1,time:0.,height:-192.};r.rebuild(map)?;w.set_dynamic(r.colliders());
        let mut deck=Vec::new();
        for x in [-840.,-820.,-800.,-780.,-760.,-740.] {for y in [-420.,-400.,-380.,-360.,-340.] {
            let feet=vec3(x,y,64.03125);
            if w.body_clear(feet) && r.lift_support(feet) {deck.push(feet);}
        }}
        let pair=deck.iter().flat_map(|a|deck.iter().map(move|b|[*a,*b]))
            .find(|p|p[0].distance(p[1])>=48.).context("Native lift has no two-body fixture")?;
        p[0]=Player::new(pair[0]);p[1]=Player::new(pair[1]);
        p[missing]=Player::new(vec3(-720.,-544.,64.03125));
        ensure!(r.coop_boarding_ready("elevator_up_thread",&p.each_ref().map(|p|p.feet))==Some(false),"Lift accepted absent rider");
        r.saved.elevator.up();
        for _ in 0..180 {tick(&mut r,map,&mut w,&mut p,&mut s,FIXED_DT)?;}
        ensure!(r.saved.elevator.height == -192.,"Lift left its waiting partner");
        p[missing]=Player::new(pair[missing]);
        ensure!(r.coop_boarding_ready("elevator_up_thread",&p.each_ref().map(|p|p.feet))==Some(true),"Two riders could not board");
        for n in 0..720 {
            tick(&mut r,map,&mut w,&mut p,&mut s,FIXED_DT)?;
            ensure!(p.iter().all(|p|w.body_clear(p.feet)) && (p[0].feet.z-p[1].feet.z).abs()<0.01,"Lift split or crushed riders");
            if n==180 {r=roundtrip(a,map,&r)?;}
            if r.saved.elevator.phase==4 {break;}
        }
        ensure!(r.saved.elevator.phase==4 && p.iter().all(|p|p.feet.z>255.),"Native lift failed to carry both");
        let top=r.saved.elevator.height;
        p[missing]=Player::new(vec3(-800.,-256.,256.03125));
        for _ in 0..360 {tick(&mut r,map,&mut w,&mut p,&mut s,FIXED_DT)?;}
        ensure!(r.saved.elevator.height==top,"Occupied lift descended without partner");
        println!("PASS native paired lift missing{missing}: boarding delay, atomic ascent, mid-ride save and top wait");
    }
    for blocked in 0..2 {for dead in [false,true] {
        let (mut r,mut w,mut p,mut s)=free(a,map)?;
        r.saved.elevator=motion::Lift{phase:0,time:2.2833356,height:-135.82239};r.rebuild(map)?;w.set_dynamic(r.colliders());
        p[blocked]=Player::new(vec3(-755.6678,-459.48755,56.146362));
        if dead {s[blocked].damage(1000.);}
        let before=r.saved.elevator.height;
        for _ in 0..3 {tick(&mut r,map,&mut w,&mut p,&mut s,1./30.)?;}
        ensure!(if dead {r.saved.elevator.height<before} else {r.saved.elevator.height==before && w.body_clear(p[blocked].feet)},"Lift underside ignored living status");
        println!("PASS native paired lift underside slot{blocked} dead={dead}: original obstruction and living roster");
    }}Ok(())
}
fn hazards(a:&mut Assets,map:&Bsp)->Result<()> {
    for piece in [Piece::Bishop,Piece::Knight] {for victim in 0..2 {
        let (mut r,mut w,mut p,mut s)=free(a,map)?;
        if piece==Piece::Knight {r.saved.bishop_done=true;r.saved.knight_gate=true;}
        p[victim]=Player::new(r.data.square(piece,if piece==Piece::Bishop {4} else {10}));
        let partner=serde_json::to_value(&s[1-victim])?;
        for _ in 0..120 {tick(&mut r,map,&mut w,&mut p,&mut s,FIXED_DT)?;}
        ensure!(!s[victim].alive() && serde_json::to_value(&s[1-victim])?==partner,"Crusher damaged wrong actor");
        println!("PASS native {piece:?} crusher victim{victim}: lethal personal damage, partner resources unchanged");
    }}Ok(())
}
fn ctl(stage:&mut Stage)->&mut Realm {stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut()).unwrap()}
fn setup(a:&mut Assets,map:&Bsp)->Result<(Stage,Roster,World)> {
    let visit=Visit{epoch:1,map:"wchess1".into(),entry:Some("wchess1_start1".into()),difficulty:Default::default(),combat_preview:false};
    let mut w=World::from_bsp(map)?;let mut stage=Stage::load(a,map,&mut w,&visit,true)?;
    let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
    let mut roster=Roster::new(&w,eye,yaw,Stats::for_level("wchess1",visit.entry.as_deref()))?;
    stage.before_move(&mut roster,map,&mut w,&Default::default(),0.)?;
    Ok((stage,roster,w))
}
fn stage_save(stage:&mut Stage,map:&Bsp,w:&mut World)->Result<()> {
    let value=serde_json::to_value(stage.snapshot())?;let bytes=crate::coop::codec::encode(&serde_json::to_vec(&value)?)?;
    stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,w)?;
    ensure!(serde_json::to_value(stage.snapshot())?==value,"Paired sequence snapshot changed");Ok(())
}
fn capture(stage:&Stage,roster:&Roster,label:&str)->Result<()> {
    std::fs::write("private/coop-wchess1-world-failure.json",serde_json::to_vec(&serde_json::json!({
        "label":label,"stage":stage.snapshot(),"players":roster.players}))?)?;Ok(())
}
fn free_step(stage:&mut Stage,roster:&mut Roster,map:&Bsp,w:&mut World,mut inputs:[(Controls,Input);2])->Result<()> {
    let cinematic;
    if stage.busy() {
        stage.scene_step(roster,map,w,&inputs)?;cinematic=true;
    } else {
        stage.before_move(roster,map,w,&inputs,FIXED_DT)?;
        if stage.busy() {roster.tick+=1;cinematic=true;} else {
            let held=std::array::from_fn(|i|stage.interactions.levels.iter().any(|s|s.ctl.coop_controlled(i)));
            for (i,(c,_)) in inputs.iter_mut().enumerate() {
                for s in &stage.interactions.levels {s.ctl.coop_filter_controls(i,c);}
                if held[i] {*c=Controls::default();}
            }
            let before=roster.players.each_ref().map(|b|b.player.feet);
            roster.move_players_with_locks(w,inputs,held);
            stage.contacts(roster,w,before)?;cinematic=stage.busy();
        }
    }
    roster.finish_tick_with_spawn_hold(w,cinematic,stage.interactive_actor().is_some(),|at|!stage.interactions.hazardous(at))
}
fn scene(a:&mut Assets,map:&Bsp,kind:scene::Kind,first:PlayerId,skip:bool,dead:bool)->Result<(Stage,Roster,World)> {
    let (mut stage,mut roster,mut w)=setup(a,map)?;
    let r=ctl(&mut stage);r.saved.pending=Some(kind);r.saved.intro=kind!=scene::Kind::Intro;
    if kind==scene::Kind::Knight {r.saved.bishop_done=true;r.saved.knight_gate=true;}
    let home=r.data.point(match kind {scene::Kind::Bishop=>"alice_bishop_puzzle_dest1",scene::Kind::Knight=>"alice_knight_puzzle_dest1",_=>"wchess1_start1"}).translation;
    roster.players[first.index()].player=Player::new(home);
    roster.players[0].stats.damage(13.);roster.players[1].stats.damage(31.);
    if dead {roster.players[first.partner().index()].stats.damage(1000.);}
    stage.owner=Some(first);
    let waiting=roster.players[first.partner().index()].player.feet;
    let resources=roster.players.each_ref().map(|p|(p.stats.sanity(),p.stats.will()));
    for n in 0..24000 {
        if n==30 {stage_save(&mut stage,map,&mut w)?;}
        if skip && n>24 {stage.interactions.skip_cinematic(map,&mut w,&mut roster.players[first.index()].player,&mut stage.story)?;}
        stage.scene_step(&mut roster,map,&mut w,&Default::default())?;
        roster.finish_tick_with(&w,true,|at|!stage.interactions.hazardous(at))?;
        if !stage.busy() {break;}
        ensure!(roster.players[first.partner().index()].player.feet==waiting,"Scene moved waiting partner");
    }
    if stage.busy() {capture(&stage,&roster,"scene handoff")?;anyhow::bail!("Paired {kind:?} handoff stalled {first:?} skip={skip} dead={dead}");}
    ensure!(roster.players.each_ref().map(|p|(p.stats.sanity(),p.stats.will()))==resources,"Scene changed personal resources");
    if matches!(kind,scene::Kind::Bishop|scene::Kind::Knight) {
        ensure!(stage.interactive_actor()==Some(first) && ctl(&mut stage).coop_defer_reunion()
            && roster.players[first.partner().index()].player.feet==waiting,"Lesson did not retain puzzle ownership/waiting partner");
    } else if !dead {
        ensure!(roster.players.iter().all(|p|w.body_clear(p.player.feet))
            && roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Scene reunion overlapped");
    }
    println!("PASS paired {kind:?} {first:?} skip={skip} dead={dead}: original scene, saved ownership, personal resources and handoff");
    Ok((stage,roster,w))
}
fn boards(a:&mut Assets,map:&Bsp)->Result<()> {
    for piece in [Piece::Bishop,Piece::Knight] {for first in PlayerId::ALL {for skip in [false,true] {for dead in [false,true] {
        let kind=if piece==Piece::Bishop {scene::Kind::Bishop} else {scene::Kind::Knight};
        let (mut stage,mut roster,mut w)=scene(a,map,kind,first,skip,dead)?;
        let waiting=roster.players[first.partner().index()].player.feet;
        for _ in 0..1200 {
            if !ctl(&mut stage).controlled() {break;}
            free_step(&mut stage,&mut roster,map,&mut w,Default::default())?;
        }
        let dirs=if piece==Piece::Bishop {check::BISHOP} else {check::KNIGHT};
        for (step,&d) in dirs.iter().enumerate() {
            let before=ctl(&mut stage).saved.board.as_ref().context("Missing disguise")?.node;
            let mut moved=false;
            for n in 0..1200 {
                let mut inputs=[(Controls::default(),Input::default());2];
                inputs[first.index()].0=Controls{wish:if moved {Vec2::ZERO} else {check::wish(piece,d)},run:true,jump:true,..Default::default()};
                inputs[first.partner().index()].0=Controls{wish:Vec2::X,run:true,jump:true,use_pressed:true,..Default::default()};
                free_step(&mut stage,&mut roster,map,&mut w,inputs)?;
                if n==10 {stage_save(&mut stage,map,&mut w)?;}
                if !roster.players[first.index()].stats.alive() {capture(&stage,&roster,"board death")?;anyhow::bail!("Paired {piece:?} died step{step} {first:?}");}
                let Some(board)=&ctl(&mut stage).saved.board else {break;};
                moved|=board.moving.is_some();
                ensure!(roster.players[first.partner().index()].player.feet==waiting,"Waiting partner moved during board");
                if dead {ensure!(!roster.players[first.partner().index()].stats.alive(),"Revival inserted a partner into the solo puzzle");}
                if moved && board.moving.is_none() && board.node!=before {break;}
            }
            if ctl(&mut stage).saved.board.as_ref().is_some_and(|b|b.node==before||b.moving.is_some()) {
                capture(&stage,&roster,"board step stalled")?;anyhow::bail!("Paired {piece:?} step{step} stalled {first:?}");
            }
        }
        for _ in 0..240 {if !stage.busy() && stage.interactive_actor().is_none(){break;}free_step(&mut stage,&mut roster,map,&mut w,Default::default())?;}
        if stage.busy() || stage.interactive_actor().is_some() {capture(&stage,&roster,"board reunion")?;anyhow::bail!("Paired {piece:?} completion reunion stalled {first:?}");}
        if dead {
            for _ in 0..1201 {
                if roster.players[first.partner().index()].stats.alive() {break;}
                let ready=roster.lives.pending_spawn().is_some();
                free_step(&mut stage,&mut roster,map,&mut w,Default::default())?;
                if ready && !roster.players[first.partner().index()].stats.alive() {
                    capture(&stage,&roster,"ready puzzle revival")?;
                    anyhow::bail!("Ready revival did not resume at puzzle handoff");
                }
            }
            ensure!(roster.players[first.partner().index()].stats.sanity()==25.,"Puzzle revival did not complete its countdown");
        }
        ensure!(ctl(&mut stage).saved.board.is_none() && roster.players.iter().all(|p|p.stats.alive() && p.player.jumps==0 && w.body_clear(p.player.feet))
            && roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Puzzle completion did not restore two safe players");
        println!("PASS full native {piece:?} puzzle {first:?} skip={skip} dead={dead}: ordinary touch-strip inputs, saved moves, waiting/reviving partner and safe finish");
    }}}}Ok(())
}
fn levers(a:&mut Assets,map:&Bsp)->Result<()> {
    for lever in 0..2 {for first in PlayerId::ALL {for interrupted in [false,true] {
        let (mut stage,mut roster,mut w)=setup(a,map)?;
        stage.owner=None;
        let r=ctl(&mut stage);r.saved.pending=None;r.saved.intro=true;
        r.saved.bishop_done=true;r.saved.knight_done=true;r.saved.knight_gate=true;
        let at=r.data.point(if lever==0 {"bell_lever"} else {"water_lever"}).point(vec3(36.,0.,26.));
        let position=(0..16).find_map(|n| {
            let angle=n as f32*std::f32::consts::TAU/16.;
            let eye=at+vec3(angle.cos()*80.,angle.sin()*80.,16.);
            let p=Player::spawn(&w,eye)?;
            let aim=(at-p.eye()).normalize_or_zero();
            (r.lever(&w,p.eye(),aim)==Some(lever)).then_some((p,aim))
        }).context("No supported native lever-use position")?;
        let owner=first.index();roster.players[owner].player=position.0.clone();
        let yaw=position.1.y.atan2(position.1.x);
        let pitch=position.1.z.asin();
        let mut input=[(Controls::default(),Input::default());2];
        input[owner]=(Controls{use_pressed:true,..Default::default()},Input{yaw,pitch,..Default::default()});
        stage.before_move(&mut roster,map,&mut w,&input,FIXED_DT)?;
        ensure!(ctl(&mut stage).saved.pull.is_some() && ctl(&mut stage).saved.coop.as_ref().unwrap().actor==Some(owner),"Lever ignored actual player");
        stage_save(&mut stage,map,&mut w)?;
        let expected=if interrupted {
            roster.players[owner].stats.damage(1000.);
            stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
            ensure!(ctl(&mut stage).saved.pull.is_none() && !ctl(&mut stage).saved.fired.contains(if lever==0 {"bell_lever"} else {"water_lever"}),"Death permanently consumed unfinished lever");
            let other=1-owner;roster.players[other].player=position.0;
            input[owner]=(Controls::default(),Input::default());input[other]=(Controls{use_pressed:true,..Default::default()},Input{yaw,pitch,..Default::default()});
            stage.before_move(&mut roster,map,&mut w,&input,FIXED_DT)?;first.partner()
        } else {first};
        let health=roster.players.each_ref().map(|b|(b.stats.sanity(),b.stats.will()));
        for _ in 0..2400 {
            if stage.busy(){break;}
            stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
        }
        ensure!(stage.owner==Some(expected),"Delayed lever scene lost owner lever{lever} {first:?} interrupted={interrupted}");
        let kind=if lever==0 {scene::Kind::Bell} else {scene::Kind::Water};
        ensure!(ctl(&mut stage).saved.pending==Some(kind),"Lever queued wrong native scene");
        for _ in 0..3600 {
            stage.interactions.skip_cinematic(map,&mut w,&mut roster.players[expected.index()].player,&mut stage.story)?;
            stage.scene_step(&mut roster,map,&mut w,&Default::default())?;
            if !stage.busy(){break;}
        }
        if stage.busy() {capture(&stage,&roster,"lever reunion")?;anyhow::bail!("Lever reunion stalled {lever} {expected:?}");}
        ensure!(roster.players.each_ref().map(|b|(b.stats.sanity(),b.stats.will()))==health,"Lever changed personal resources");
        if lever==1 {
            for _ in 0..1000 {stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;}
            ensure!(ctl(&mut stage).water_ready(),"Paired lever never raised water/unlocked exit");
        }
        println!("PASS native lever{lever} {first:?} interrupted={interrupted}: personal use, saved pull, original timer/scene owner and recovery");
    }}}Ok(())
}
fn board_death(a:&mut Assets,map:&Bsp)->Result<()> {
    for first in PlayerId::ALL {
        let (mut stage,mut roster,mut w)=scene(a,map,scene::Kind::Bishop,first,true,false)?;
        let r=ctl(&mut stage);let valid=r.snapshot();let mut invalid=valid.clone();
        invalid["coop"]["actor"]=serde_json::json!(2);
        ensure!(r.restore(&invalid,map).is_err() && r.snapshot()==valid,"Invalid puzzle owner mutated state");
        let partner=first.partner().index();let health=roster.players[partner].stats.sanity();
        roster.players[first.index()].stats.damage(1000.);
        stage.before_move(&mut roster,map,&mut w,&Default::default(),FIXED_DT)?;
        ensure!(stage.interactive_actor().is_none() && ctl(&mut stage).saved.board.is_none()
            && ctl(&mut stage).saved.retry==Some(Piece::Bishop),"Dead chess actor kept partner locked or lost retry");
        stage_save(&mut stage,map,&mut w)?;
        for n in 0..1202 {
            free_step(&mut stage,&mut roster,map,&mut w,Default::default())?;
            if n<1199 {ensure!(!roster.players[first.index()].stats.alive(),"Puzzle actor revived before ten seconds");}
            if roster.players[first.index()].stats.alive(){break;}
        }
        ensure!(roster.players[first.index()].stats.sanity()==25. && roster.players[partner].stats.sanity()==health
            && w.body_clear(roster.players[first.index()].player.feet),"Puzzle death did not revive safely beside survivor");
        println!("PASS puzzle death {first:?}: partner released, retry and invalid-owner validation, saved ten-second 25-health revival");
    }Ok(())
}
fn wall_contact(a:&mut Assets,map:&Bsp)->Result<()> {
    let mut w=World::from_bsp(map)?;
    let visit=Visit{epoch:1,map:"wchess1".into(),entry:Some("wchess1_start1".into()),difficulty:Default::default(),combat_preview:false};
    let mut stage=Stage::load(a,map,&mut w,&visit,false)?;
    // The captured wall is the fully opened supply door, not a static brush.
    // Restore that native door pose as an explicit component fixture.
    let mut saved=serde_json::to_value(stage.snapshot())?;
    let door=saved["world"]["doors"].as_array_mut().unwrap().iter_mut()
        .find(|d|d["id"]==294).context("Missing supply door")?;
    door["angle"]=serde_json::json!(90.);door["target"]=serde_json::json!(90.);
    stage.restore(&serde_json::from_value(saved)?,map,&mut w)?;
    // Exact movement endpoints captured by the continuous route at its supply
    // corridor. This isolated replay is a contact regression, not route credit.
    let before=[vec3(-10.2879038,-532.2437744,64.03125),vec3(7.7714453,-500.8675842,64.03125)];
    let after=[vec3(-9.862754,-531.8186035,64.03125),vec3(7.8765364,-500.816925,64.03125)];
    let blocked=w.body_trace(after[0],before[0]);
    ensure!(blocked.fraction<1. && blocked.normal.z.abs()<0.01,"Native fixture lost the captured glancing door contact");
    for moving in 0..2 {
        let mut roster=Roster::new(&w,after[0]+Vec3::Z*crate::movement::EYE_HEIGHT,0.,Stats::default())?;
        for (b,at) in roster.players.iter_mut().zip(after) {
            b.player=Player::new(at);b.player.grounded=true;
            ensure!(w.body_clear(at),"Captured chess contact embedded in native geometry");
        }
        roster.players[0].player.velocity=vec3(51.017956,51.01796,0.);
        roster.players[0].stats.spend_will(17.);
        let mut previous=before;
        if moving==1 {roster.players.swap(0,1);previous.swap(0,1);}
        let still=roster.players[1-moving].player.feet;
        let resources=roster.players.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
        crate::coop::separation::resolve(&w,&mut roster.players,previous);
        for tick in 0..240 {
            let d=roster.players[0].player.feet-roster.players[1].player.feet;
            ensure!(d.truncate().length()>=crate::coop::separation::DISTANCE-0.05,"Native chess wall overlap moving{moving} tick{tick}");
            for (i,b) in roster.players.iter().enumerate() {
                ensure!(w.body_clear(b.player.feet) && w.body_trace(b.player.feet,b.player.feet-Vec3::Z*2.).fraction<1.
                    && serde_json::to_value(&b.stats)?==resources[i],"Wall correction lost support or personal resources");
            }
            ensure!(roster.players[1-moving].player.feet==still,"Wall correction shoved the partner");
            let d=vec3(0.,-528.,64.)-roster.players[moving].player.feet;
            let mut inputs=[(Controls::default(),Input::default());2];
            inputs[moving].0=Controls{wish:d.truncate().normalize_or_zero()*(d.truncate().length()/48.).min(1.),run:true,..Default::default()};
            roster.move_players(&w,inputs);
        }
        println!("PASS captured native chess wall contact slot{moving}: 240 ordinary movement ticks, separation/support/resources and stationary partner");
    }Ok(())
}
fn falling_contact(map:&Bsp)->Result<()> {
    let w=World::from_bsp(map)?;
    for (name,before,normal,velocity) in [
        ("gallery",[vec3(527.38501,1186.87915,63.935966),vec3(543.67627,1162.8291,121.33673)],
            vec3(0.24253564,0.,0.97014254),vec3(1.5649414,-3.0802407,-686.6662)),
        ("narrow stairs",[vec3(3743.95288,2299.99707,197.53348),vec3(3743.62769,2304.29248,254.00348)],
            vec3(0.,-0.44721362,0.89442724),vec3(2.1972656,-1.7285156,-126.66669)),
    ] {for moving in 0..2 {
        let mut roster=Roster::new(&w,before[0]+Vec3::Z*crate::movement::EYE_HEIGHT,0.,Stats::default())?;
        for (b,at) in roster.players.iter_mut().zip(before) {b.player=Player::new(at);}
        roster.players[0].player.grounded=true;
        roster.players[0].player.ground_normal=normal;
        roster.players[1].player.velocity=velocity;
        if moving==0 {roster.players.swap(0,1);}
        let still=roster.players[1-moving].player.feet;
        for tick in 0..240 {
            roster.move_players(&w,Default::default());
            let d=roster.players[0].player.feet-roster.players[1].player.feet;
            ensure!(d.truncate().length()>=crate::coop::separation::DISTANCE-0.05 || d.z.abs()>=PLAYER_HALF.z*2.-0.05,
                "Captured descending chess overlap {name} slot{moving} tick{tick}");
            ensure!(roster.players[1-moving].player.feet.distance(still)<0.01,"Falling Alice shoved her stationary partner");
            ensure!(roster.players.iter().all(|b|w.body_clear(b.player.feet)),"Descending contact embedded a body");
        }
        println!("PASS captured native descending contact {name} slot{moving}: 240 physics ticks, no overlap or partner shove");
    }}Ok(())
}
fn overhanging_contact(map:&Bsp)->Result<()> {
    let w=World::from_bsp(map)?;
    let before=[vec3(3722.08813,3084.90088,0.031249762),vec3(3699.91260,3056.54175,3.9199333)];
    let after=[before[0],vec3(3699.93066,3056.56470,0.031249762)];
    for moving in 0..2 {
        let mut roster=Roster::new(&w,after[0]+Vec3::Z*crate::movement::EYE_HEIGHT,0.,Stats::default())?;
        for (b,at) in roster.players.iter_mut().zip(after) {
            b.player=Player::new(at);b.player.grounded=true;
            ensure!(w.body_trace(at,at-Vec3::Z*2.).fraction<1.
                && w.sweep(at+Vec3::Z,at-Vec3::Z*2.,Vec3::ZERO).fraction==1.,
                "Captured edge no longer has footprint-only support");
        }
        let mut previous=before;
        if moving==0 {roster.players.swap(0,1);previous.swap(0,1);}
        let still=roster.players[1-moving].player.feet;
        let resources=roster.players.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
        crate::coop::separation::resolve(&w,&mut roster.players,previous);
        for tick in 0..240 {
            let d=roster.players[0].player.feet-roster.players[1].player.feet;
            ensure!(d.truncate().length()>=crate::coop::separation::DISTANCE-0.001,
                "Captured overhang overlap slot{moving} tick{tick}");
            ensure!(roster.players[1-moving].player.feet==still,"Edge correction shoved the waiting partner");
            for (i,b) in roster.players.iter().enumerate() {
                ensure!(w.body_clear(b.player.feet) && b.player.grounded
                    && w.body_trace(b.player.feet,b.player.feet-Vec3::Z*2.).fraction<1.
                    && serde_json::to_value(&b.stats)?==resources[i],"Edge correction lost support or resources");
            }
            let mut input=[(Controls::default(),Input::default());2];
            input[moving].0=Controls{wish:(still-roster.players[moving].player.feet).truncate().normalize_or_zero(),run:true,..Default::default()};
            roster.move_players(&w,input);
        }
        println!("PASS captured native overhang contact slot{moving}: 240 physics ticks, separation/shared support/resources and stationary partner");
    }Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/wchess1.bsp")?)?;
    wall_contact(a,&map)?;falling_contact(&map)?;overhanging_contact(&map)?;
    clocks(a,&map)?;lift(a,&map)?;hazards(a,&map)?;
    for kind in [scene::Kind::Intro,scene::Kind::Bishop,scene::Kind::Knight,scene::Kind::Bell,scene::Kind::Water] {
        for first in PlayerId::ALL {for skip in [false,true] {for dead in [false,true] {scene(a,&map,kind,first,skip,dead)?;}}}
    }
    boards(a,&map)?;levers(a,&map)?;board_death(a,&map)?;
    println!("PASS paired Pale Realm world/scenes/puzzles; full combat traversal, internet and recovery remain separate");Ok(())
}

/// Exercise both Alice materials and the native transformation emitter on the
/// real renderer, including the compressed guest state. No screenshots needed.
pub(super) fn render(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        let mut level=crate::render::Scene::load(a,"wchess1")?;
        let initial=Realm::load(a,&level.map)?;
        let mut art=art::Art::load(a,&initial)?;
        for kind in [scene::Kind::Bishop,scene::Kind::Knight] {for actor in 0..2 {
            let (mut r,mut w,mut p,mut stats)=free(a,&level.map)?;
            r.saved.pending=Some(kind);
            if kind==scene::Kind::Knight {r.saved.bishop_done=true;r.saved.knight_gate=true;}
            p[actor]=Player::new(r.data.point(if kind==scene::Kind::Bishop {"alice_bishop_puzzle_dest1"} else {"alice_knight_puzzle_dest1"}).translation);
            r.coop_scene_step(actor,&mut p[actor],&mut stats[actor]);
            r.advance(FIXED_DT,&level.map,&mut w,&mut p[actor],&[])?;
            let t=r.scene_times(r.saved.scene.as_ref().unwrap())[3]+0.3;
            while r.saved.scene.as_ref().unwrap().clock.time<t {
                r.advance(FIXED_DT,&level.map,&mut w,&mut p[actor],&[])?;
                r.coop_scene_step(actor,&mut p[actor],&mut stats[actor]);
            }
            let guest=roundtrip(a,&level.map,&r)?;
            let target=r.alice_scene_pose().unwrap().0.translation+Vec3::Z*40.;
            let camera=target+vec3(160.,-160.,100.);
            let mut batches=[0;2];
            for (side,body) in [&r,&guest].into_iter().enumerate() {
                for _ in 0..3 {
                    clear_background(BLACK);
                    let view=Camera3D{position:camera,target,up:Vec3::Z,fovy:60_f32.to_radians(),z_near:2.,z_far:20000.,..Default::default()};
                    set_camera(&view);
                    crate::render_fx::begin_view(&view,body.saved.age,&level.atmosphere,false);
                    level.draw(camera,body.saved.age,false,false,&body.transforms());
                    art.draw(body,&level.atmosphere,camera,false);
                    let (_,dropped)=crate::render_fx::finish();ensure!(dropped==0,"Chess model pass dropped batches");
                    crate::render_fx::begin_view(&view,body.saved.age,&level.atmosphere,false);
                    art.effects(body,camera,&level.atmosphere);
                    let (drawn,dropped)=crate::render_fx::finish();
                    ensure!(drawn>0 && dropped==0,"Native chess transformation particles absent side{side} actor{actor}");
                    batches[side]+=drawn;
                    set_default_camera();next_frame().await;
                }
            }
            ensure!(batches[0]==batches[1],"Guest transformation rendering differs");
            println!("PASS native {kind:?} render actor{actor}: original Alice/material, host/guest transformation batches {batches:?}, zero drops");
        }}
        Ok(())
    })
}
