//! Native machinery fixtures, distinct from a continuous two-client route.
use super::*;
use crate::{interaction::Interactions,level::CoopPlayer,movement::{Controls,FIXED_DT}};

fn slots<'a>(p:&'a mut [Player;2],s:&'a mut [Stats;2])->[CoopPlayer<'a>;2] {
    let [a,b]=p.each_mut();let [x,y]=s.each_mut();
    [CoopPlayer{player:a,stats:x},CoopPlayer{player:b,stats:y}]
}
fn free(a:&mut Assets,map:&Bsp)->Result<Funhouse> {
    let mut o=Funhouse::load(a,map)?;o.saved.scene=None;o.saved.initialized=true;o.saved.arrived=true;
    o.rebuild(map)?;Ok(o)
}
fn people(at:[Vec3;2])->[Player;2] {at.map(|at| {let mut p=Player::new(at);p.grounded=true;p})}
fn fulcrum()->[Vec3;2] {[vec3(1880.,-3430.,0.03125),vec3(1960.,-3430.,0.03125)]}
fn stats()->[Stats;2] {std::array::from_fn(|_|Stats::for_level("funhouse",None))}
fn world(map:&Bsp,o:&Funhouse)->Result<World> {
    let mut w=World::from_bsp(map)?;w.set_dynamic(o.colliders());w.set_settled_supports(o.settled_supports());Ok(w)
}
fn clear(w:&World,p:&[Player;2])->bool {
    let d=p[0].feet-p[1].feet;
    p.iter().all(|p|w.body_clear(p.feet)) && (d.truncate().length()>=35.95 || d.z.abs()>=55.95)
}
fn stripped(o:&Funhouse)->serde_json::Value {
    let mut v=o.snapshot();v.as_object_mut().unwrap().remove("coop");v
}
fn tube_people(map:&Bsp,o:&Funhouse)->Result<[Player;2]> {
    let w=world(map,o)?;let tube=o.objects.iter().find(|o|o.name=="tuber").unwrap();
    let mut p=people([Vec3::ZERO;2]);
    for (p,x) in p.iter_mut().zip([0.,80.]) {
        let high=vec3(x,-448.,160.);let low=vec3(x,-448.,-32.);
        let hit=w.body_trace(high,low);
        ensure!(!hit.start_solid && hit.fraction<1. && hit.normal.z>=0.65,"No clear native tube landing for x{x}");
        p.feet=high.lerp(low,hit.fraction);p.ground_normal=hit.normal;
        let support=tube.collider.trace(p.feet+PLAYER_CENTER,p.feet+PLAYER_CENTER-Vec3::Z*3.,PLAYER_HALF);
        ensure!(w.body_clear(p.feet) && support.fraction<1. && support.normal.z>=0.65,"Tube fixture stands on a different support");
    }
    ensure!(clear(&w,&p),"Tube fixture positions overlap");Ok(p)
}
fn tilts(a:&mut Assets,map:&Bsp)->Result<()> {
    for reversed in [false,true] {
        let mut o=free(a,map)?;let mut w=world(map,&o)?;
        let mut p=people(fulcrum());if reversed {p.reverse();}let mut s=stats();
        ensure!(clear(&w,&p),"Invalid paired fulcrum starting positions");
        o.coop_prepare(&mut slots(&mut p,&mut s));let before=o.snapshot();let bodies=serde_json::to_value(&p)?;
        for dt in [0.,-1.,f32::NAN] {o.coop_advance_roster(dt,map,&mut w,&mut slots(&mut p,&mut s),&[])?;}
        ensure!(o.snapshot()==before && serde_json::to_value(&p)?==bodies,"Paused paired world moved");
        for _ in 0..120 {
            o.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
            ensure!(clear(&w,&p),"Paired fulcrum overlaps a body");
        }
        ensure!((o.saved.time-1.).abs()<0.001 && p.iter().all(|p|p.feet.z< -20.),
            "Paired fulcrum did not carry both once: {:?}, time {}",p.each_ref().map(|p|p.feet),o.saved.time);
        let mut q=free(a,map)?;q.restore(&o.snapshot(),map)?;
        let mut qw=world(map,&q)?;let mut qp=p.clone();let mut qs=s.clone();
        for _ in 0..60 {
            o.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
            q.coop_advance_roster(FIXED_DT,map,&mut qw,&mut slots(&mut qp,&mut qs),&[])?;
        }
        ensure!(o.snapshot()==q.snapshot() && serde_json::to_value(&p)?==serde_json::to_value(&qp)?,"Saved paired world diverged");
        for p in &mut p {p.tick(&w,Controls{jump:true,..Default::default()});ensure!(p.jumps==1 && !p.grounded,"Rider cannot jump off tilted support");}
        println!("PASS Funhouse both fulcrum riders, single clock, pause, restore and normal jumps reversed={reversed}");
    }
    let mut o=free(a,map)?;let mut w=world(map,&o)?;let mut s=stats();
    let mut p=people([vec3(1920.,-3430.,0.03125),vec3(1920.,-3770.,0.03125)]);
    ensure!(clear(&w,&p),"Balanced pair starts embedded");let before=p.each_ref().map(|p|p.feet);
    for _ in 0..120 {o.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;}
    let i=o.objects.iter().position(|o|o.id==460).unwrap();
    ensure!(o.saved.fulcrums[i].length()<0.001 && before==p.each_ref().map(|p|p.feet),"Opposite riders did not balance the native board");
    println!("PASS Funhouse opposite riders balance the shared fulcrum");
    Ok(())
}
fn surface_goals(a:&mut Assets,map:&Bsp)->Result<()> {
    for angle in [-45., 0., 45.] { for rider in 0..2 {
        let mut o=free(a,map)?;
        let index=o.objects.iter().position(|o|o.id==460).unwrap();
        o.saved.fulcrums[index]=vec2(angle,0.);o.rebuild(map)?;
        let requested=vec3(1920.,-3400.,0.);
        let goal=o.surface_goal(460,requested).context("No projected seesaw goal")?;
        ensure!(goal.truncate().distance(requested.truncate())<0.001,"Projection changed horizontal goal");
        let mut p=people([vec3(1920.,-3040.,0.03125);2]);
        p[rider].feet=goal;
        let before=o.snapshot();let bodies=serde_json::to_value(&p)?;
        let report=route_surface(&o,460,requested,p.each_ref()).unwrap();
        ensure!(report.supported[rider] && !report.supported[1-rider],"Wrong seesaw support identity");
        ensure!(o.snapshot()==before && serde_json::to_value(&p)?==bodies,"Surface query mutated gameplay");
        p[rider].feet.z-=80.;
        ensure!(!route_surface(&o,460,requested,p.each_ref()).unwrap().supported[rider],"Lower floor counted as board arrival");
        p[rider].feet=goal;p[rider].grounded=false;
        ensure!(!route_surface(&o,460,requested,p.each_ref()).unwrap().supported[rider],"Airborne body counted as board arrival");
        ensure!(o.surface_goal(460,vec3(1920.,-2500.,0.)).is_none() && o.surface_goal(99999,requested).is_none(),"Unknown support accepted");
        println!("PASS Funhouse live surface projection, personal support and lower-floor rejection angle={angle} rider={rider}");
    }}Ok(())
}
fn pendulums(a:&mut Assets,map:&Bsp)->Result<()> {
    for reversed in [false,true] {
        let mut o=free(a,map)?;let mut w=world(map,&o)?;let mut s=stats();
        let mut p=people([vec3(-152.,-3552.,8.03125),vec3(-104.,-3552.,8.03125)]);
        if reversed {p.reverse();}let before=p.each_ref().map(|p|p.feet);
        ensure!(clear(&w,&p),"Paired pendulum fixture starts embedded");
        let mut moved=false;
        for _ in 0..960 {
            o.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
            ensure!(clear(&w,&p),"Pendulum embedded one rider");
            moved |= p.iter().zip(before).all(|(p,b)|p.feet.distance(b)>10.);
        }
        ensure!(moved && o.saved.time>7.99 && p.iter().zip(before).all(|(p,b)|p.feet.distance(b)<3.),
            "Paired pendulum missed full period: time{} feet{:?}",o.saved.time,p.each_ref().map(|p|p.feet));
        println!("PASS Funhouse full pendulum period carries both upright riders reversed={reversed}");
    }Ok(())
}
fn doors_and_tube(a:&mut Assets,map:&Bsp)->Result<()> {
    for id in [147,161] {for side in [-1.,1.] {for near in 0..2 {
        let mut o=free(a,map)?;let mut w=world(map,&o)?;let mut s=stats();
        let i=o.objects.iter().position(|o|o.id==id).unwrap();let object=&o.objects[i];
        let shape=&map.models[object.model];let at=object.base.translation+(shape.min+shape.max)*0.5;
        let mut p=people([vec3(10000.,10000.,0.);2]);p[near]=Player::new(vec3(at.x,at.y+65.*side,0.03125));
        for _ in 0..240 {o.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;}
        ensure!(o.saved.doors[i]==1. && clear(&w,&p),"Door {id} ignored player{near} side{side}");
        println!("PASS Funhouse door{id} admits either player near{near} side{side}");
    }}}
    for side in [-1.,1.] {
        let mut o=free(a,map)?;let mut w=world(map,&o)?;let mut s=stats();
        let i=o.objects.iter().position(|o|o.id==147).unwrap();let object=&o.objects[i];
        let shape=&map.models[object.model];let at=object.base.translation+(shape.min+shape.max)*0.5;
        let mut p=people([vec3(at.x,at.y+65.*side,0.03125),vec3(at.x,at.y-65.*side,0.03125)]);
        let before=p.each_ref().map(|p|p.feet);
        for _ in 0..240 {
            o.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
            ensure!(clear(&w,&p),"Door swept through its other waiting player");
        }
        ensure!(o.saved.doors[i]>0. && o.saved.doors[i]<1. && before==p.each_ref().map(|p|p.feet)
            && o.saved.time>1.99,"Blocked door stopped unrelated world clocks or failed to hold");
        println!("PASS Funhouse other-player door obstruction side{side}, no crush or global pause");
    }
    for reversed in [false,true] {
        let mut o=free(a,map)?;let mut w=world(map,&o)?;let mut s=stats();
        let mut p=tube_people(map,&o)?;
        if reversed {p.reverse();}let before=p.each_ref().map(|p|p.feet);
        ensure!(clear(&w,&p),"Tube riders start embedded");
        o.event("Funhouse_Rotate_Tuber").unwrap();
        for _ in 0..1020 {
            o.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
            ensure!(clear(&w,&p),"Rotating tube embedded a rider");
        }
        let pivot=o.data.points["tuber_bind"].translation;let turn=Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
        ensure!(o.saved.time>8.49 && p.iter().zip(before).all(|(p,at)|p.feet.distance(pivot+turn*(at-pivot))<0.1),
            "Tube failed its complete paired turn: time{} feet{:?}",o.saved.time,p.each_ref().map(|p|p.feet));
        println!("PASS Funhouse complete rotating-tube turn carries both bodies reversed{reversed}");
    }Ok(())
}
fn dead_and_blocked(a:&mut Assets,map:&Bsp)->Result<()> {
    for dead in 0..2 {
        let mut o=free(a,map)?;let mut solo=free(a,map)?;let mut w=world(map,&o)?;let mut sw=world(map,&solo)?;
        let mut p=people(fulcrum());p[dead]=Player::new(vec3(10000.,10000.,0.));
        let dead_before=serde_json::to_value(&p[dead])?;let mut s=stats();s[dead].damage(1000.);
        let mut single=p[1-dead].clone();
        for _ in 0..120 {
            o.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
            solo.advance(FIXED_DT,map,&mut sw,&mut single,&[])?;
        }
        ensure!(stripped(&o)==solo.snapshot() && serde_json::to_value(&p[1-dead])?==serde_json::to_value(&single)?
            && serde_json::to_value(&p[dead])?==dead_before,"Survivor machinery differs from single player");
        for s in &mut s {s.damage(1000.);}let saved=o.snapshot();
        o.coop_advance_roster(0.1,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
        ensure!(o.snapshot()==saved,"Dead team advanced the world");
        println!("PASS Funhouse dead slot{dead} unchanged, survivor matches SP exactly, dead team frozen");
    }
    for blocked in 0..2 {
        let mut o=free(a,map)?;let mut w=world(map,&o)?;let mut p=people(fulcrum());let mut s=stats();
        o.coop_prepare(&mut slots(&mut p,&mut s));let before=o.snapshot();let bodies=serde_json::to_value(&p)?;
        let mut preview=free(a,map)?;preview.restore(&before,map)?;
        let mut pw=world(map,&preview)?;let mut pp=p.clone();let mut ps=s.clone();
        preview.coop_advance_roster(0.1,map,&mut pw,&mut slots(&mut pp,&mut ps),&[])?;
        let end=pp[blocked].feet;
        ensure!(end.z<p[blocked].feet.z-1.,"Obstruction fixture has no downward motion");
        let obstacle=Collider::box_bounds(end-vec3(4.,4.,1.),end+vec3(4.,4.,0.5));
        w.set_dynamic(o.colliders().into_iter().chain([obstacle.clone()]).collect());
        ensure!(clear(&w,&p),"Obstruction starts inside a rider");
        o.coop_advance_roster(0.1,map,&mut w,&mut slots(&mut p,&mut s),std::slice::from_ref(&obstacle))?;
        ensure!(o.snapshot()==before && serde_json::to_value(&p)?==bodies,"Blocked carry partly moved a body or clock");
        o.coop_advance_roster(0.1,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
        ensure!(o.snapshot()==preview.snapshot() && serde_json::to_value(&p)?==serde_json::to_value(&pp)?,"Unblocked carry did not resume atomically");
        println!("PASS Funhouse blocked rider{blocked}: whole step held, both resume after clearing");
    }Ok(())
}
fn boarding(a:&mut Assets,map:&Bsp)->Result<()> {
    use crate::{coop::PlayerId,interaction::coop::{Contacts,Motion}};
    let thread="Funhouse_Rotate_Tuber";
    let entity=map.entities.iter().position(|e|e.get("thread").is_some_and(|s|s==thread)).unwrap();
    for first in PlayerId::ALL {for dead in [None,Some(first),Some(first.partner())] {
        let mut x=Interactions::load(map)?;x.set_entry(a,map,"funhouse",None)?;
        let o=x.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Funhouse>()).unwrap();
        o.saved.scene=None;o.saved.arrived=true;o.saved.initialized=true;o.rebuild(map)?;
        let pair=tube_people(map,o)?;
        let mut c=Contacts::new(&x);let mut feet=[vec3(10000.,10000.,0.);2];
        feet[first.index()]=pair[0].feet;
        let paths=|v:[Vec3;2]|v.map(|at|Some(Motion{before:at,after:at}));
        for _ in 0..5 {
            let out=x.triggers_pair(FIXED_DT,&mut c,paths(feet))?;
            ensure!(out.waiting[first.index()] && !out.activations.iter().any(|a|a.entity.0==entity),"Tube left its partner");
        }
        let saved=x.snapshot();let mut copy:Contacts=serde_json::from_value(serde_json::to_value(&c)?)?;
        x.restore(&saved,map)?;copy.validate(&x)?;
        feet[first.partner().index()]=pair[1].feet;
        let mut path=paths(feet);if let Some(dead)=dead {path[dead.index()]=None;}
        let out=x.triggers_pair(FIXED_DT,&mut copy,path)?;
        let events=out.activations.iter().filter(|a|a.entity.0==entity).collect::<Vec<_>>();
        let expected=if dead==Some(first) {first.partner()} else {first};
        ensure!(events.len()==1 && events[0].player==expected,"Tube lost first living requester");
        let o=x.levels.iter().find_map(|s|s.ctl.downcast_ref::<Funhouse>()).unwrap();
        ensure!(o.saved.tube.is_some(),"Registered tube event did not start");
        println!("PASS Funhouse authored tube boarding first{first:?} dead{dead:?}: wait, saved contact and one departure");
    }}Ok(())
}
fn gas(a:&mut Assets,map:&Bsp)->Result<()> {
    for nearest in 0..2 {for dead in [None,Some(nearest),Some(1-nearest)] {
        let mut o=free(a,map)?;o.saved.cells=255;o.saved.all_cells=Some(0.);o.saved.time=1.;
        let focus=o.data.gas_focus;let mut p=people([focus+Vec3::X*20.,focus+Vec3::X*200.]);
        if nearest==1 {p.reverse();}let mut s=stats();if let Some(dead)=dead {s[dead].damage(1000.);}
        let actor=o.coop_contacts(FIXED_DT,&mut slots(&mut p,&mut s));
        let expected=if dead==Some(nearest) {1-nearest} else {nearest};
        ensure!(actor==Some(expected) && o.saved.scene.as_ref().is_some_and(|s|s.kind==Kind::Gas),"Automatic gas scene chose wrong living actor");
        let mut w=world(map,&o)?;let saved=o.snapshot();
        o.coop_advance_roster(FIXED_DT,map,&mut w,&mut slots(&mut p,&mut s),&[])?;
        ensure!(o.snapshot()==saved,"Free-world path advanced a bound cinematic");
        let mut q=free(a,map)?;q.restore(&saved,map)?;ensure!(q.snapshot()==saved,"Gas ownership changed on restore");
        println!("PASS Funhouse automatic gas scene nearest{nearest} dead{dead:?}: bound living actor, single clock, save");
    }}
    let mut o=free(a,map)?;o.saved.cells=255;o.saved.all_cells=Some(0.);o.saved.time=1.;
    let mut p=people(fulcrum());let mut s=stats();for s in &mut s {s.damage(1000.);}
    let saved=o.snapshot();ensure!(o.coop_contacts(FIXED_DT,&mut slots(&mut p,&mut s)).is_none() && o.snapshot()==saved,"Dead team started gas scene");
    println!("PASS Funhouse dead team cannot start automatic gas scene");Ok(())
}
fn sloped_revival_evidence(a:&mut Assets,map:&Bsp)->Result<()> {
    let o=free(a,map)?;let w=world(map,&o)?;
    // Actual natural-revival positions from the paired descending-corridor run.
    for feet in [vec3(684.78735,-4760.3857,142.00255),vec3(658.48145,-4634.635,138.41537)] {
        let mut p=Player::spawn(&w,feet+Vec3::Z*crate::movement::EYE_HEIGHT).context("Missing sloped spawn support")?;
        ensure!(p.feet.distance(feet)<0.1 && crate::coop::world::revival_supported(&w,&p),"Actual sloped revival rejected");
        ensure!(!crate::recovery::supported(&w,p.feet),"Fixture does not exercise the old flat-ground audit mismatch");
        p.feet.z+=8.;
        ensure!(!crate::coop::world::revival_supported(&w,&p),"Floating revival counted as supported");
        p.feet=feet;p.grounded=false;
        ensure!(!crate::coop::world::revival_supported(&w,&p),"Airborne body counted as grounded revival");
        println!("PASS Funhouse actual sloped revival evidence at {feet:?}, floating/airborne rejected");
    }
    Ok(())
}
fn corridor_approach(a:&mut Assets,map:&Bsp)->Result<()> {
    let o=free(a,map)?;let w=world(map,&o)?;
    let start=vec3(60.4505,-4475.4922,64.03125);let goal=vec3(32.,-4460.,96.);
    for delay in [0,12,24] {
        let mut p=Player::spawn(&w,start+Vec3::Z*crate::movement::EYE_HEIGHT)
            .context("Recorded corridor footing has no spawn support")?;
        ensure!(p.feet.distance(start)<0.1,"Recorded corridor start changed");
        let mut views=std::collections::VecDeque::from(vec![p.clone();delay+1]);
        let mut inputs=std::collections::VecDeque::from(vec![Controls::default();delay]);
        for _ in 0..600 {
            let view=views.pop_front().unwrap();
            let (wish,_)=crate::coop::route_driver::landing_control(&w,&view,goal,2.*delay as f32/120.,8.)
                .context("Recorded supported approach incorrectly requests a full path search")?;
            inputs.push_back(Controls {wish,..Default::default()});
            p.tick(&w,inputs.pop_front().unwrap());
            ensure!(p.grounded && w.body_clear(p.feet),"Corridor approach left its supported floor");
            views.push_back(p.clone());
        }
        ensure!((p.feet-goal).truncate().length()<8. && (p.feet.z-goal.z).abs()<35.
            && p.velocity.truncate().length()<20.,"Corridor did not reach original walking bounds");
        ensure!(p.feet.distance(goal)>22.,"Fixture no longer reproduces impossible first planner tolerance");
        println!("PASS Funhouse recorded corridor approach with {delay}-tick observation/input delay");
    }
    Ok(())
}
fn cell_switch_approaches(a:&mut Assets,map:&Bsp)->Result<()> {
    let mut x=Interactions::load(map)?;x.set_entry(a,map,"funhouse",None)?;
    let target=x.switch_target("cell3").context("Missing cell3 target")?;
    // The old retry moved farther west, into the open door's line of fire.
    // A northern neighbor is reachable without entering that sweep. These are
    // geometry fixtures, separate from a live route's movement/shot receipts.
    let blocked=vec3(-1717.4673,-25.179287,0.03125);
    let north=vec3(-1676.,22.,0.03125);
    for fraction in [0.,0.5,0.8,1.] {
        let o=x.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Funhouse>()).unwrap();
        o.saved.scene=None;o.saved.initialized=true;o.saved.arrived=true;o.saved.cells=10;
        let door=o.objects.iter().position(|o|o.id==160).context("Missing cell door")?;
        o.saved.doors[door]=fraction;o.saved.door_signs[door]=1.;o.rebuild(map)?;
        let mut w=World::from_bsp(map)?;x.sync(&mut w);
        let p=Player::spawn(&w,north+Vec3::Z*crate::movement::EYE_HEIGHT).context("No northern shooting footing")?;
        ensure!(p.feet.distance(north)<0.1 && w.body_clear(p.feet),"Northern shooting position lacks clear support");
        let targets=x.shot_targets();let context=crate::combat::Context{world:&w,targets:&targets};
        let visible=crate::coop::route_driver::visible_switch_approaches(&context,target,&[blocked,north]);
        ensure!(visible.contains(&north),"Northern switch approach is blocked at door fraction {fraction}");
        if fraction==1. {ensure!(!visible.contains(&blocked),"Fixture lost the original open-door obstruction");}
        println!("PASS Funhouse cell3 northern shooting footing and native sight line, door fraction {fraction}");
    }
    Ok(())
}
pub(super) fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/funhouse.bsp")?)?;
    if let Some(path)=std::env::var_os("LOOKING_GLASS_FUNHOUSE_SWITCH_PROBE") {
        let input:serde_json::Value=serde_json::from_slice(&std::fs::read(path)?)?;
        let mut x=Interactions::load(&map)?;x.set_entry(a,&map,"funhouse",None)?;
        x.restore(&serde_json::from_value(input["world"].clone())?,&map)?;
        let mut w=World::from_bsp(&map)?;x.sync(&mut w);
        let thread=input["thread"].as_str().context("Missing diagnostic switch name")?;
        let target=x.switch_target(thread).context("Diagnostic switch is not currently active")?;
        let mut targets=x.levels_targets();targets.extend(x.shot_targets());
        if let Some(cast)=&x.encounters {targets.extend(cast.targets());}
        let context=crate::combat::Context{world:&w,targets:&targets};
        let feet:Vec<Vec3>=serde_json::from_value(input["feet"].clone())?;
        ensure!(feet.len()<=64 && feet.iter().all(|p|p.is_finite()),"Invalid diagnostic positions");
        let probes=feet.into_iter().map(|p|crate::coop::route_driver::switch_probe(&context,
            p+Vec3::Z*crate::movement::EYE_HEIGHT,target)).collect::<Vec<_>>();
        println!("{}",serde_json::to_string(&serde_json::json!({"diagnostic_only":true,"probes":probes}))?);
        return Ok(());
    }
    let mut failures=Vec::new();
    for (label,result) in [("fulcrums",tilts(a,&map)),("surface goals",surface_goals(a,&map)),("pendulums",pendulums(a,&map)),("doors/tube",doors_and_tube(a,&map)),
        ("dead/blocked",dead_and_blocked(a,&map)),("boarding",boarding(a,&map)),("gas",gas(a,&map)),
        ("sloped revival evidence",sloped_revival_evidence(a,&map)),("corridor approach",corridor_approach(a,&map)),
        ("cell switch approaches",cell_switch_approaches(a,&map))] {
        if let Err(e)=result {failures.push(format!("{label}: {e:#}"));}
    }
    ensure!(failures.is_empty(),"Funhouse paired world failures:\n{}",failures.join("\n"));
    println!("PASS Funhouse paired machinery components; full scene/reunion, traversal and internet checks remain separate");Ok(())
}
