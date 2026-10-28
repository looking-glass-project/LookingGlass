//! Native machinery and production scene/contact paths, separate from full traversal.
use super::*;
use crate::{coop::{protocol::Visit, stage::Stage, world::Roster, PlayerId},
    interaction::coop::{Contacts,Motion}, level::CoopPlayer, movement::FIXED_DT};
fn slots<'a>(p:&'a mut [Player;2],s:&'a mut [Stats;2])->[CoopPlayer<'a>;2] {
    let [a,b]=p.each_mut();let [x,y]=s.each_mut();
    [CoopPlayer{player:a,stats:x},CoopPlayer{player:b,stats:y}]
}
fn setup(a:&mut Assets,map:&Bsp)->Result<(Castle,World,[Player;2],[Stats;2])> {
    let r=Castle::load(a,map)?;let mut w=World::from_bsp(map)?;w.set_dynamic(r.colliders());
    Ok((r,w,[Player::new(vec3(0.,0.,16000.)),Player::new(vec3(100.,0.,16000.))],
        std::array::from_fn(|_|Stats::for_level("wchess2",None))))
}
fn tick(r:&mut Castle,map:&Bsp,w:&mut World,p:&mut [Player;2],s:&mut [Stats;2],dt:f32)->Result<()> {
    r.coop_advance_roster(dt,map,w,&mut slots(p,s),&[])
}
fn restored(a:&mut Assets,map:&Bsp,r:&Castle)->Result<Castle> {
    let bytes=crate::coop::codec::encode(&serde_json::to_vec(&r.snapshot())?)?;
    let mut q=Castle::load(a,map)?;q.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map)?;
    ensure!(q.snapshot()==r.snapshot(),"Castling shared world changed across save");Ok(q)
}
fn clocks(a:&mut Assets,map:&Bsp)->Result<()> {
    for hz in [30,60,144] {
        let (mut r,mut w,mut p,mut s)=setup(a,map)?;let mut sp=Castle::load(a,map)?;
        for x in [&mut r,&mut sp] {
            x.saved.queen=true;x.saved.king=true;x.saved.portal=Some(0.);
            x.saved.queen_release=Some(0.5);x.saved.fade=0.5;
        }
        let mut sw=World::from_bsp(map)?;let mut actor=p[0].clone();
        for n in 0..hz*6 {
            tick(&mut r,map,&mut w,&mut p,&mut s,1./hz as f32)?;
            sp.advance(1./hz as f32,map,&mut sw,&mut actor,&[])?;
            ensure!(r.snapshot()==sp.snapshot() && r.transforms()==sp.transforms(),"Castling shared clock diverged at {hz}Hz tick{n}");
            if n==hz {
                let before=r.snapshot();let people=serde_json::to_value((&p,&s))?;
                for dt in [0.,-1.,f32::NAN] {tick(&mut r,map,&mut w,&mut p,&mut s,dt)?;}
                ensure!(r.snapshot()==before && serde_json::to_value((&p,&s))?==people,"Paused Castling machinery changed");
                r=restored(a,map,&r)?;
            }
        }
        println!("PASS Castling shared world {hz}Hz: original clock/door/cast/portal parity, pause and save");
    }Ok(())
}
fn portal(a:&mut Assets,map:&Bsp)->Result<()> {
    for missing in 0..2 {for dead in [false,true] {
        let (mut r,mut w,_,mut s)=setup(a,map)?;
        r.saved.queen=true;r.saved.king=true;r.saved.portal=Some(0.);r.rebuild(map)?;w.set_dynamic(r.colliders());
        let deck=[vec3(-1060.,-64.,r.portal_pose().translation.z+64.03125),
            vec3(-988.,-64.,r.portal_pose().translation.z+64.03125)];
        let mut p=deck.map(Player::new);p.iter_mut().for_each(|p|p.grounded=true);
        ensure!(p.iter().all(|p|r.portal_support(p)&&w.body_clear(p.feet)),"Native portal fixture has no two-body deck");
        p[missing]=Player::new(vec3(0.,0.,16000.));if dead {s[missing].damage(1000.);}
        let before=r.portal_pose().translation;let bodies=p.each_ref().map(|p|p.feet);
        for _ in 0..240 {tick(&mut r,map,&mut w,&mut p,&mut s,FIXED_DT)?;}
        ensure!(if dead {r.portal_pose().translation.distance(before)>1.}
            else {r.portal_pose().translation.distance(before)<0.001 && p.each_ref().map(|p|p.feet)==bodies},
            "Portal boarding failed missing{missing} dead{dead}");
        if !dead {p[missing]=Player::new(deck[missing]);p[missing].grounded=true;}
        let mut low=p[1-missing].feet.z;let mut high=low;
        for n in 0..1440 {
            tick(&mut r,map,&mut w,&mut p,&mut s,FIXED_DT)?;
            for (body,_) in p.iter_mut().zip(&s).filter(|(_,s)|s.alive()) {
                body.tick(&w,Controls::default());
                ensure!(w.body_clear(body.feet) && (body.feet.z-r.portal_pose().translation.z-64.03125).abs()<0.2,
                    "Portal split/crushed rider missing{missing} dead{dead}");
            }
            low=low.min(p[1-missing].feet.z);high=high.max(p[1-missing].feet.z);
            if n==180 {r=restored(a,map,&r)?;}
        }
        ensure!(high-low>25.,"Boarded portal did not traverse its authored bob");
        println!("PASS Castling portal missing{missing} dead{dead}: wait, living roster, paired carry and saved future");
    }}
    for blocked in 0..2 {
        let (mut r,mut w,_,mut s)=setup(a,map)?;
        r.saved.queen=true;r.saved.king=true;r.saved.portal=Some(0.);
        r.saved.age=(0..240).map(|n|n as f32*0.1).find(|t|
            r.data.portal.sample(*t+0.1,false).translation.z-r.data.portal.sample(*t,false).translation.z>0.5)
            .context("Portal has no ascending native phase")?;
        r.rebuild(map)?;
        let mut p=[-1060.,-988.].map(|x|Player::new(vec3(x,-64.,r.portal_pose().translation.z+64.03125)));
        let before=p.each_ref().map(|p|p.feet);let pose=r.portal_pose().translation;
        // A fixture ceiling only above one rider rejects the whole moving deck.
        let roof=Collider::box_bounds(p[blocked].feet+vec3(-20.,-20.,PLAYER_CENTER.z+PLAYER_HALF.z+0.1),
            p[blocked].feet+vec3(20.,20.,120.));
        w.set_dynamic(r.colliders().into_iter().chain([roof.clone()]).collect());
        r.coop_advance_roster(0.1,map,&mut w,&mut slots(&mut p,&mut s),&[roof])?;
        ensure!(r.portal_pose().translation.distance(pose)<0.001 && p.each_ref().map(|p|p.feet)==before
            && p.iter().all(|p|w.body_clear(p.feet)),"Portal ignored roof slot{blocked} at {pose:?} from {before:?}");
        tick(&mut r,map,&mut w,&mut p,&mut s,0.1)?;
        ensure!(r.portal_pose().translation.z>pose.z+0.5 && p.iter().zip(before).all(|(p,b)|p.feet.z>b.z+0.5),
            "Portal failed to resume both riders after roof removal");
        println!("PASS Castling portal roof slot{blocked}: atomic mover/rider rollback");
    }Ok(())
}
fn doors(a:&mut Assets,map:&Bsp)->Result<()> {
    for near in 0..2 {for dead in [false,true] {
        let (mut r,mut w,mut p,mut s)=setup(a,map)?;
        for actor in &mut r.saved.cast {actor.piece.active=false;}
        let (k,feet)=r.objects.iter().filter(|o|o.door.is_some() && !o.locked && o.flags&64!=0 && o.flags&0xa4==0)
            .find_map(|o| {
                let center=(o.bounds.0+o.bounds.1)*0.5;
                [Vec3::X,-Vec3::X,Vec3::Y,-Vec3::Y].into_iter().find_map(|d| {
                    let extent=(o.bounds.1-o.bounds.0)*0.5;
                    let at=center+d*(extent.dot(d.abs())+40.)-Vec3::Z*PLAYER_CENTER.z;
                    w.body_clear(at).then_some((o.door.unwrap(),at))
                })
            }).context("No native proximity door fixture")?;
        p[near]=Player::new(feet);if dead{s[near].damage(1000.);}
        tick(&mut r,map,&mut w,&mut p,&mut s,0.1)?;
        ensure!(if dead{r.saved.doors[k].amount==0.}else{r.saved.doors[k].amount>0.},"Wrong door activator near{near} dead{dead}");
        println!("PASS Castling proximity door slot{near} dead{dead}: personal contact and one shared step");
    }}Ok(())
}
fn door_safety(a:&mut Assets,map:&Bsp)->Result<()> {
    for slot in 0..2 {
        let (mut r,mut w,mut p,mut s)=setup(a,map)?;
        for actor in &mut r.saved.cast {actor.piece.active=false;}
        let (k,player,aim)=r.objects.iter().filter(|o|o.door.is_some() && !o.locked && o.flags&(64|128|8)==0)
            .find_map(|o| {
                let at=(o.bounds.0+o.bounds.1)*0.5;
                [Vec3::X,-Vec3::X,Vec3::Y,-Vec3::Y].into_iter().find_map(|d| {
                    let player=Player::new(at+d*80.-Vec3::Z*crate::movement::EYE_HEIGHT);
                    let aim=(at-player.eye()).normalize_or_zero();
                    (w.body_clear(player.feet) && r.use_door(&w,player.eye(),aim)==o.door).then_some((o.door.unwrap(),player,aim))
                })
            }).context("No usable native manual door fixture")?;
        p[slot]=player;
        ensure!(!r.saved.doors[k].opening,"Manual door already open");
        r.coop_update(slot,&mut w,&p[slot],aim,true);
        ensure!(r.saved.doors[k].opening,"Manual door ignored player{slot}");
        p[slot]=Player::new(vec3(200.,0.,16000.));
        for _ in 0..240 {tick(&mut r,map,&mut w,&mut p,&mut s,FIXED_DT)?;}
        ensure!(r.saved.doors[k].amount>0.,"Manual door never moved");
        println!("PASS Castling manual door slot{slot}: original reach/aim checks and personal use");
    }
    for slot in 0..2 {for dead in [false,true] {
        let (mut r,mut w,mut p,mut s)=setup(a,map)?;
        for actor in &mut r.saved.cast {actor.piece.active=false;}
        let mut fixture=None;
        for object in 0..r.objects.len() {
            let o=&r.objects[object];
            let Some(k)=o.door.filter(|_|!o.locked && o.slide==Vec3::ZERO) else {continue;};
            r.saved.doors[k].amount=0.5;r.saved.doors[k].opening=true;r.saved.doors[k].sign=1.;
            r.rebuild(map)?;w.set_dynamic(r.colliders());
            let o=&r.objects[object];
            let next=Collider::model(map,o.model,o.base.translation,
                Quat::from_rotation_z((0.5+0.1/o.duration).min(1.)*std::f32::consts::FRAC_PI_2),true)?;
            let radius=((o.bounds.1-o.bounds.0).truncate().length()+48.) as i32;
            'search: for x in (-radius..=radius).step_by(8) {for y in (-radius..=radius).step_by(8) {
                let at=vec3(o.base.translation.x+x as f32,o.base.translation.y+y as f32,o.bounds.0.z+0.1);
                if w.body_clear(at) && next.trace(at+PLAYER_CENTER,at+PLAYER_CENTER,PLAYER_HALF).start_solid {
                    fixture=Some((k,at));break 'search;
                }
            }}
            if fixture.is_some(){break;}
            r.saved.doors[k].amount=0.;r.saved.doors[k].opening=false;
        }
        let (k,at)=fixture.context("No native swept-door obstruction fixture")?;
        p[slot]=Player::new(at);if dead{s[slot].damage(1000.);}
        let before=r.saved.doors[k].amount;
        tick(&mut r,map,&mut w,&mut p,&mut s,0.1)?;
        ensure!(if dead {r.saved.doors[k].amount>before}else{r.saved.doors[k].amount==before && w.body_clear(p[slot].feet)},
            "Native door crushed living body or stopped for dead slot{slot} dead{dead}");
        println!("PASS Castling swept door slot{slot} dead{dead}: native rotating leaf blocks only for living bodies");
    }}Ok(())
}
fn ctl(stage:&mut Stage)->&mut Castle {stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut()).unwrap()}
fn scene(a:&mut Assets,map:&Bsp,kind:scene::Kind,first:PlayerId,skip:bool,dead:bool)->Result<()> {
    let label=kind.id();
    let visit=Visit{epoch:1,map:"wchess2".into(),entry:Some("wchess2_start1".into()),difficulty:Default::default(),combat_preview:false};
    let mut w=World::from_bsp(map)?;let mut stage=Stage::load(a,map,&mut w,&visit,true)?;
    let (eye,yaw)=crate::interaction::spawn(map,visit.entry.as_deref());
    let mut roster=Roster::new(&w,eye,yaw,Stats::for_level("wchess2",visit.entry.as_deref()))?;
    if kind==scene::Kind::King {ctl(&mut stage).saved.queen=true;}
    let approach=if kind==scene::Kind::King{vec3(-1024.,1600.,192.)}else{vec3(-1024.,-4096.,-64.)};
    roster.players[first.index()].player=Player::spawn(&w,approach).context("Missing royal approach footing")?;
    roster.players[0].stats.damage(13.);roster.players[1].stats.damage(31.);
    if dead {roster.players[first.partner().index()].stats.damage(1000.);}
    for _ in 0..300 {
        let before=roster.players.each_ref().map(|p|p.player.feet);
        roster.players[first.index()].player.tick(&w,Controls{wish:Vec2::Y,..Default::default()});
        stage.contacts(&mut roster,&mut w,before)?;
        if stage.busy(){break;}
    }
    ensure!(stage.busy() && stage.owner==Some(first),"Wrong royal first arrival {label} {first:?}");
    let waiting=roster.players[first.partner().index()].player.feet;
    let resources=roster.players.each_ref().map(|p|(p.stats.sanity(),p.stats.will()));
    for n in 0..24000 {
        if n==120 {
            let value=serde_json::to_value(stage.snapshot())?;
            let bytes=crate::coop::codec::encode(&serde_json::to_vec(&value)?)?;
            stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,&mut w)?;
            ensure!(serde_json::to_value(stage.snapshot())?==value,"Royal scene save changed");
        }
        let mut inputs=[(Controls::default(),crate::coop::protocol::Input::default());2];
        inputs[first.partner().index()].1.skip_scene=true;
        inputs[first.index()].1.skip_scene=skip && n>=240;
        stage.scene_step(&mut roster,map,&mut w,&inputs)?;
        if n<240 {ensure!(stage.scene_id()==Some(label),"Waiting partner skipped the owner's scene");}
        if skip && n==330 {ensure!(stage.scene_id()!=Some(label),"Owner's held skip did not finish scene");}
        if !stage.busy(){break;}
        ensure!(roster.players[first.partner().index()].player.feet==waiting,"Royal scene moved waiting partner");
    }
    ensure!(!stage.busy(),"Royal reunion stalled {label} {first:?} skip{skip} dead{dead}");
    ensure!(stage.notices[first.index()].as_ref().is_some_and(|n|n.text=="Scene skipped")==skip,
        "Royal scene accepted the wrong player's skip");
    ensure!(resources==roster.players.each_ref().map(|p|(p.stats.sanity(),p.stats.will())),"Royal scene changed resources");
    if !dead {ensure!(roster.players.iter().all(|p|w.body_clear(p.player.feet))
        && roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Royal reunion overlaps");}
    println!("PASS Castling {label} {first:?} skip{skip} dead{dead}: actual approach ownership, frozen partner, save and safe reunion");Ok(())
}
fn exit(a:&mut Assets,map:&Bsp)->Result<()> {
    for first in PlayerId::ALL {for dead in [None,Some(first),Some(first.partner())] {
        let mut i=Interactions::load(map)?;i.set_entry(a,map,"wchess2",Some("wchess2_start1"))?;
        let mut w=World::from_bsp(map)?;
        let r=i.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Castle>()).unwrap();
        r.saved.queen=true;r.saved.king=true;r.saved.portal=Some(0.);r.rebuild(map)?;i.sync_coop(&mut w);
        let mut feet=[vec3(0.,0.,16000.);2];feet[first.index()]=vec3(-1024.,-100.,512.03125);
        let paths=|feet:[Vec3;2]|feet.map(|at|Some(Motion{before:at,after:at}));
        let mut contacts=Contacts::new(&i);
        for _ in 0..120 {
            let events=i.triggers_pair(FIXED_DT,&mut contacts,paths(feet))?;
            ensure!(events.waiting[first.index()] && events.activations.iter().all(|a|a.events.transition.is_none()),"Portal failed to wait for partner");
        }
        let bytes=crate::coop::codec::encode(&serde_json::to_vec(&(i.snapshot(),contacts))?)?;
        let (saved,c):(crate::interaction::Snapshot,Contacts)=serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?;
        i.restore(&saved,map)?;contacts=c;
        feet[first.partner().index()]=vec3(-1024.,-28.,512.03125);let mut p=paths(feet);
        if let Some(dead)=dead {p[dead.index()]=None;}
        let events=i.triggers_pair(FIXED_DT,&mut contacts,p)?;
        let exits=events.activations.iter().filter(|a|a.events.transition.is_some()).collect::<Vec<_>>();
        let owner=if dead==Some(first) {first.partner()} else {first};
        ensure!(exits.len()==1 && exits[0].player==owner && exits[0].events.transition==Some(EXIT.destination()),"Portal lost first living waiting owner");
        ensure!(i.triggers_pair(FIXED_DT,&mut contacts,p)?.activations.iter().all(|a|a.events.transition.is_none()),"Portal repeated exit");
        println!("PASS Castling portal exit {first:?} dead{dead:?}: saved waiting owner, living boarding and one-shot destination");
    }}Ok(())
}
fn moving_exit(a:&mut Assets,map:&Bsp)->Result<()> {
    for roster_path in [false,true] {for restore in [false,true] {
        let mut i=Interactions::load(map)?;i.set_entry(a,map,"wchess2",Some("wchess2_start1"))?;
        let mut w=World::from_bsp(map)?;
        let r=i.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Castle>()).unwrap();
        r.saved.queen=true;r.saved.king=true;r.saved.portal=Some(0.);r.rebuild(map)?;
        let mut people=[Player::new(vec3(0.,0.,16000.)),Player::new(vec3(100.,0.,16000.))];
        let mut stats=std::array::from_fn(|_|Stats::for_level("wchess2",None));
        for _ in 0..60 {
            if roster_path {i.movers_roster(0.1,map,&mut w,&mut slots(&mut people,&mut stats))?;}
            else {i.movers_pair(0.1,map,&mut w,&mut people.each_mut())?;}
        }
        let r=i.levels.iter().find_map(|s|s.ctl.downcast_ref::<Castle>()).unwrap();
        let delta=r.portal_pose().translation-r.data.point("exit_portal").translation;
        let probe=vec3(-1024.,-64.,458.)+delta;
        let base=&r.data.volumes["exit_trigger"];
        ensure!(delta.z< -16. && !base.trace(probe+PLAYER_CENTER,probe+PLAYER_CENTER,PLAYER_HALF).start_solid
            && r.portal_boarded(probe),"Bound exit probe does not distinguish original and current volume");
        if restore {
            let saved=i.snapshot();
            i=Interactions::load(map)?;i.set_entry(a,map,"wchess2",Some("wchess2_start1"))?;
            i.restore(&saved,map)?;
        }
        let mut contacts=Contacts::new(&i);
        let events=i.triggers_pair(FIXED_DT,&mut contacts,[Some(Motion{before:probe,after:probe}),None])?;
        ensure!(events.activations.iter().any(|e|e.events.transition==Some(EXIT.destination())),
            "Bound portal exit stayed at editor pose roster{roster_path} restore{restore}");
        println!("PASS Castling moving exit roster{roster_path} restore{restore}: actual translated contact volume, no clock advance on restore");
    }}Ok(())
}
fn sloped_wall_contact(a:&mut Assets,map:&Bsp)->Result<()> {
    let (_,w,_,_)=setup(a,map)?;
    for moving in 0..2 {
        let mut bodies=std::array::from_fn(|_|crate::coop::world::Body {
            player:Player::new(Vec3::ZERO),stats:Stats::for_level("wchess2",None),
            yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0,
        });
        // Captured immediately before the public combat-wipe overlap. The
        // native wall briefly lifts the approaching body off the flat floor.
        bodies[1-moving].player.feet=vec3(-120.723800659,-643.012023926,64.03125);
        bodies[moving].player.feet=vec3(-144.989608765,-671.818786621,64.03125);
        bodies[moving].player.velocity=vec3(-0.923340797,59.354679108,0.);
        for b in &mut bodies {b.player.grounded=true;}
        let still=serde_json::to_value(&bodies[1-moving])?;
        let resources=bodies.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
        for n in 0..120 {
            let previous=bodies.each_ref().map(|b|b.player.clone());
            for (i,b) in bodies.iter_mut().enumerate() {
                b.player.tick(&w,Controls{wish:if i==moving {vec2(-0.928,0.373)} else {Vec2::ZERO},..Default::default()});
            }
            crate::coop::separation::wait_for_climbers(&w,&mut bodies,&previous);
            crate::coop::separation::resolve(&w,&mut bodies,previous.each_ref().map(|p|p.feet));
            let d=bodies[0].player.feet-bodies[1].player.feet;
            ensure!(d.truncate().length()>=36.-0.05 || d.z.abs()>=PLAYER_HALF.z*2.-0.05,
                "Sloped-wall overlap slot{moving} tick{n}: previous {:?}, now {:?}, retreat {:?}",
                previous[moving],bodies[moving].player,w.body_trace(bodies[moving].player.feet,previous[moving].feet));
            ensure!(serde_json::to_value(&bodies[1-moving])?==still,"Wall contact moved the stationary partner");
            for (i,b) in bodies.iter().enumerate() {
                ensure!(w.body_clear(b.player.feet) && (b.player.feet.z-64.03125).abs()<1.
                    && serde_json::to_value(&b.stats)?==resources[i],"Wall contact lost safe footing/resources slot{i}");
            }
        }
        println!("PASS Castling captured sloped-wall contact slot{moving}: 120 physics ticks, stationary partner and safe separation");
    }Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/wchess2.bsp")?)?;
    let mut failures=Vec::new();
    for (label,result) in [("sloped wall contact",sloped_wall_contact(a,&map)),("clocks",clocks(a,&map)),("portal",portal(a,&map)),
        ("doors",doors(a,&map)),("door safety",door_safety(a,&map))] {
        if let Err(error)=result {failures.push(format!("{label}: {error:#}"));}
    }
    for kind in [scene::Kind::Queen,scene::Kind::King] {for first in PlayerId::ALL {for skip in [false,true] {for dead in [false,true] {
        if let Err(error)=scene(a,&map,kind,first,skip,dead) {failures.push(format!("scene: {error:#}"));}
    }}}}
    if let Err(error)=exit(a,&map) {failures.push(format!("exit: {error:#}"));}
    if let Err(error)=moving_exit(a,&map) {failures.push(format!("moving exit: {error:#}"));}
    ensure!(failures.is_empty(),"Castling paired world failures:\n{}",failures.join("\n"));
    println!("PASS Castling paired world components; full traversal, internet and recovery remain separate");Ok(())
}
