//! Staged native component fixtures, not continuous paired traversal evidence.
use super::*;
use crate::{level::CoopPlayer, movement::FIXED_DT};

fn slots<'a>(p: &'a mut [Player; 2], s: &'a mut [Stats; 2]) -> [CoopPlayer<'a>; 2] {
    let [a, b] = p.each_mut(); let [x, y] = s.each_mut();
    [CoopPlayer { player: a, stats: x }, CoopPlayer { player: b, stats: y }]
}
fn people(feet: [Vec3; 2]) -> [Player; 2] {
    feet.map(|at| { let mut p = Player::new(at); p.grounded = true; p })
}
fn stats() -> [Stats; 2] { std::array::from_fn(|_| Stats::for_level("hatter1", None)) }
fn world(map: &Bsp, o: &Clockwork) -> Result<World> {
    let mut w = World::from_bsp(map)?; w.set_dynamic(o.colliders()); Ok(w)
}
fn clear(w: &World, p: &[Player; 2]) -> bool {
    let delta = p[0].feet - p[1].feet;
    p.iter().all(|p| w.body_clear(p.feet)) && (delta.truncate().length() >= 35.95 || delta.z.abs() >= 55.95)
}
fn deck(map: &Bsp, o: &Clockwork, name: &str) -> Result<[Player; 2]> {
    let w = world(map, o)?;
    let object = o.objects.iter().find(|o| o.name == name).context("Missing deck")?;
    let model = &map.models[object.model];
    let center = object.pose.translation + (model.min + model.max) * 0.5;
    let z = object.pose.translation.z + model.max.z + 0.03125;
    let p = people([-24., 24.].map(|x| vec3(center.x + x, center.y, z)));
    ensure!(clear(&w, &p) && p.iter().all(|p| o.supported_by(name, p.feet)), "Invalid {name} deck fixture: {:?}", p.each_ref().map(|p| p.feet));
    Ok(p)
}
fn tick(o: &mut Clockwork, map: &Bsp, w: &mut World, p: &mut [Player; 2], s: &mut [Stats; 2]) -> Result<()> {
    o.coop_advance_roster(FIXED_DT, map, w, &mut slots(p, s), &[])
}
fn roundtrip(a: &mut Assets, map: &Bsp, o: &Clockwork) -> Result<Clockwork> {
    let data = crate::coop::codec::encode(&serde_json::to_vec(&o.snapshot())?)?;
    let mut copy = Clockwork::load(a, map)?;
    copy.restore(&serde_json::from_slice(&crate::coop::codec::decode(&data)?)?, map)?;
    ensure!(copy.snapshot() == o.snapshot(), "Paired Clockwork changed across save/restore"); Ok(copy)
}
fn clocks(a: &mut Assets, map: &Bsp) -> Result<()> {
    for hz in [30, 60, 144] {
        let mut o = Clockwork::load(a, map)?; let mut solo = Clockwork::load(a, map)?;
        let mut w = world(map, &o)?; let mut sw = world(map, &solo)?;
        let mut p = people([vec3(10000., 10000., 0.), vec3(10100., 10000., 0.)]);
        let mut single = p[0].clone(); let mut s = stats();
        o.coop_prepare(&mut slots(&mut p, &mut s)); solo.saved.initialized = true;
        s[0].damage(17.); s[1].damage(41.);
        o.coop_prepare(&mut slots(&mut p, &mut s));
        ensure!(s[0].sanity() == 83. && s[1].sanity() == 59., "Preparation refilled personal health");
        let before = o.snapshot();
        for dt in [0., -1., f32::NAN] { o.coop_advance_roster(dt, map, &mut w, &mut slots(&mut p, &mut s), &[])?; }
        ensure!(o.snapshot() == before, "Paused paired machinery moved");
        for n in 0..hz * 2 {
            let dt = 1. / hz as f32;
            o.coop_advance_roster(dt, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            solo.advance(dt, map, &mut sw, &mut single, &[])?;
            let mut state = o.snapshot(); state.as_object_mut().unwrap().remove("coop");
            ensure!(state == solo.snapshot() && o.transforms() == solo.transforms(), "Paired clock differs {hz}Hz tick{n}");
        }
        ensure!((o.saved.age - 2.).abs() < 0.001, "Clock advanced twice");
        o = roundtrip(a, map, &o)?;
        let mut invalid = o.snapshot(); invalid["coop"]["actor"] = 2.into();
        ensure!(o.restore(&invalid, map).is_err(), "Invalid owner accepted");
        println!("PASS Clockwork paired {hz}Hz clock, pause, personal initialization and saved owner validation");
    }
    Ok(())
}
fn gear_steering(a: &mut Assets, map: &Bsp) -> Result<()> {
    use std::collections::VecDeque;
    use crate::movement::Controls;
    for delay in [0,12,24,45] { for reversed in [false,true] {
        let mut o=Clockwork::load(a,map)?;
        o.saved.age=10.;o.saved.levers[0]=Some(1.);o.saved.levers[1]=Some(2.);o.rebuild(map)?;
        let mut w=world(map,&o)?;
        let mut goals=[vec3(-4512.,3008.,208.03125),vec3(-4352.,3168.,208.03125)];
        if reversed {goals.reverse();}
        let mut p=people(goals.map(|g|g+vec3(12.,-6.,0.)));let mut s=stats();
        ensure!(clear(&w,&p) && p.iter().all(|p|o.route_gear_velocity(p).is_some()),"Staged gear footing lacks support");
        let initial_carry=p.each_ref().map(|p|o.route_gear_velocity(p).unwrap());
        let mut views=VecDeque::from(vec![(p.clone(),initial_carry);delay+1]);
        let mut pending=VecDeque::from(vec![[Controls::default();2];delay]);
        for n in 0..2400 {
            let (seen,carry)=views.pop_front().unwrap();
            let controls=std::array::from_fn(|i| {
                let (wish,_)=crate::coop::route_driver::carried_goal_control(&seen[i],goals[i],carry[i],2.*delay as f32/120.,8.);
                Controls {wish,run:true,..Default::default()}
            });
            pending.push_back(controls);
            for (p,input) in p.iter_mut().zip(pending.pop_front().unwrap()) {p.tick(&w,input);}
            tick(&mut o,map,&mut w,&mut p,&mut s)?;
            ensure!(clear(&w,&p) && p.iter().all(|p|p.grounded && p.jumps==0 && o.route_gear_velocity(p).is_some()),
                "Native gear lost a rider delay{delay} reversed{reversed} at{n}");
            views.push_back((p.clone(),p.each_ref().map(|p|o.route_gear_velocity(p).unwrap())));
            if n>=2160 {
                ensure!(p.iter().zip(goals).all(|(p,goal)|crate::coop::route_driver::carried_goal_control(
                    p,goal,o.route_gear_velocity(p).unwrap(),0.,8.).1),"Native carried arrivals did not hold delay{delay} reversed{reversed}");
            }
        }
        let mut airborne=p[0].clone();airborne.grounded=false;
        ensure!(o.route_gear_velocity(&airborne).is_none(),"Airborne player reported as a carried rider");
        let mut outside=p[0].clone();outside.feet+=Vec3::X*2000.;
        ensure!(o.route_gear_velocity(&outside).is_none(),"Unrelated floor reported as gear support");
        println!("PASS Clockwork ordinary gear steering delay{delay} reversed{reversed}: two supported arrivals and sustained holding");
    }}
    Ok(())
}
fn rides(a: &mut Assets, map: &Bsp) -> Result<()> {
    for reversed in [false, true] {
        let mut o = Clockwork::load(a, map)?; let mut w = world(map, &o)?;
        let mut p = deck(map, &o, "Lift1")?; if reversed { p.reverse(); }
        let mut s = stats(); let start = p.each_ref().map(|p| p.feet);
        o.event("Lift_Down");
        for _ in 0..30 { tick(&mut o, map, &mut w, &mut p, &mut s)?; ensure!(clear(&w, &p), "Lift embedded a rider"); }
        ensure!(p.iter().zip(start).all(|(p, at)| (p.feet.z - at.z + 170.).abs() < 0.1), "Both lift riders did not descend");
        o.event("Lift_Up");
        for n in 0..30 {
            tick(&mut o, map, &mut w, &mut p, &mut s)?;
            ensure!(clear(&w, &p), "Ascending lift embedded a rider");
            if n == 10 { o = roundtrip(a, map, &o)?; }
        }
        ensure!(p.iter().zip(start).all(|(p, at)| p.feet.distance(at) < 0.1), "Both lift riders did not return");
        println!("PASS Clockwork two lift riders reversed{reversed}: descent, ascent and mid-ride restore");
    }
    for blocked in 0..2 {
        let mut o = Clockwork::load(a, map)?; o.saved.lift = 170.; o.saved.lift_up = true; o.rebuild(map)?;
        let mut w = world(map, &o)?; let mut p = deck(map, &o, "Lift1")?; let mut s = stats();
        o.coop_prepare(&mut slots(&mut p, &mut s));
        let before = o.snapshot(); let feet = p.each_ref().map(|p| p.feet);
        let at = p[blocked].feet;
        let ceiling = Collider::box_bounds(at + vec3(-18., -18., 62.), at + vec3(18., 18., 66.));
        o.coop_advance_roster(FIXED_DT, map, &mut w, &mut slots(&mut p, &mut s), &[ceiling])?;
        ensure!(o.snapshot() == before && p.each_ref().map(|p| p.feet) == feet, "Blocked rider split the shared lift transaction");
        tick(&mut o, map, &mut w, &mut p, &mut s)?;
        ensure!(p.iter().zip(feet).all(|(p, at)| p.feet.z > at.z + 7.) && clear(&w, &p), "Lift did not resume after obstruction cleared");
        println!("PASS Clockwork lift blocked rider{blocked}: atomic rollback and release");
    }
    for reversed in [false, true] {
        let mut o = Clockwork::load(a, map)?; let mut w = world(map, &o)?;
        let mut p = deck(map, &o, "floater1")?; if reversed { p.reverse(); }
        let start = p.each_ref().map(|p| p.feet); let mut s = stats();
        o.event("Start_Floater1");
        for n in 0..120 {
            tick(&mut o, map, &mut w, &mut p, &mut s)?;
            ensure!(clear(&w, &p) && p.iter().all(|p| o.supported_by("floater1", p.feet)), "Floater lost or embedded a rider at {n}");
            if n == 60 { o = roundtrip(a, map, &o)?; }
        }
        ensure!(o.saved.age > 0.99 && p.iter().zip(start).all(|(p, at)| p.feet.distance(at) > 10.), "Floater failed to advance both riders");
        println!("PASS Clockwork two floater riders reversed{reversed}: shared track and mid-ride restore");
    }
    Ok(())
}
fn boarding(a: &mut Assets, map: &Bsp) -> Result<()> {
    use crate::{coop::PlayerId, interaction::{Interactions, coop::{Contacts, Motion}}};
    for (thread, name) in [("Lift_Up", "Lift1"), ("Start_Floater1", "floater1")] {
        let entity = map.entities.iter().position(|e| e.get("thread").is_some_and(|s| s == thread)).context("Boarding trigger missing")?;
        for first in PlayerId::ALL { for dead in [None, Some(first), Some(first.partner())] {
            let mut x = Interactions::load(map)?; x.set_entry(a, map, "hatter1", None)?;
            let o = x.levels.iter_mut().find_map(|s| s.ctl.downcast_mut::<Clockwork>()).unwrap();
            if name == "Lift1" { o.saved.lift = 170.; o.saved.lift_up = false; o.rebuild(map)?; }
            let mut pair = deck(map, o, name)?;
            if name == "Lift1" {
                // The deck is wider than the tiny central activation volume.
                // These separated footings touch both the deck and that trigger.
                pair[0].feet.x += 2.; pair[1].feet.x -= 2.;
            }
            let mut contacts = Contacts::new(&x);
            let mut feet = [vec3(10000., 10000., 0.); 2]; feet[first.index()] = pair[0].feet;
            if name == "Lift1" { feet[first.partner().index()] = vec3(-4900., 2464., 144.03125); }
            let paths = |v: [Vec3; 2]| v.map(|at| Some(Motion { before: at, after: at }));
            for _ in 0..3 {
                let out = x.triggers_pair(FIXED_DT, &mut contacts, paths(feet))?;
                ensure!(!out.activations.iter().any(|a| a.entity.0 == entity), "{thread} departed before both boarded");
                ensure!(out.waiting[first.index()], "{thread} did not register the waiting requester at {:?}", feet[first.index()]);
            }
            let saved = x.snapshot(); x.restore(&saved, map)?;
            contacts = serde_json::from_value(serde_json::to_value(&contacts)?)?; contacts.validate(&x)?;
            feet[first.partner().index()] = pair[1].feet;
            let mut path = paths(feet); if let Some(dead) = dead { path[dead.index()] = None; }
            let out = x.triggers_pair(FIXED_DT, &mut contacts, path)?;
            let events = out.activations.iter().filter(|a| a.entity.0 == entity).collect::<Vec<_>>();
            let expected = if dead == Some(first) { first.partner() } else { first };
            ensure!(events.len() == 1 && events[0].player == expected, "{thread} lost its first living requester");
            let o = x.levels.iter().find_map(|s| s.ctl.downcast_ref::<Clockwork>()).unwrap();
            ensure!(if name == "Lift1" { o.saved.lift_up } else { o.saved.floater.is_some() }, "Boarding did not activate {thread}");
            println!("PASS Clockwork {thread} first{first:?} dead{dead:?}: actual contacts wait, restore and depart");
        }}
    }
    // A living caller can request the empty upper lift from its lower floor.
    let mut x = Interactions::load(map)?; x.set_entry(a, map, "hatter1", None)?;
    let mut c = Contacts::new(&x);
    let at = vec3(-4900., 2464., 144.03125);
    let o = x.levels.iter().find_map(|s| s.ctl.downcast_ref::<Clockwork>()).unwrap();
    let w = world(map, o)?;
    let ground = w.body_trace(at, at - Vec3::Z * 3.);
    ensure!(w.body_clear(at) && !ground.start_solid && ground.fraction < 1. && ground.normal.z > 0.65,
        "Lower lift caller is not on a supported clear floor");
    let out = x.triggers_pair(FIXED_DT, &mut c, [Some(Motion { before: at, after: at }), None])?;
    let entity = map.entities.iter().position(|e| e.get("thread").is_some_and(|s| s == "Lift_Down")).unwrap();
    ensure!(out.activations.iter().any(|a| a.entity.0 == entity), "Lower floor could not recall empty lift");
    let o = x.levels.iter().find_map(|s| s.ctl.downcast_ref::<Clockwork>()).unwrap();
    ensure!(!o.saved.lift_up, "Recall did not lower lift");
    println!("PASS Clockwork actual lower-floor contact recalls the empty upper lift");
    Ok(())
}
fn levers(a: &mut Assets, map: &Bsp) -> Result<()> {
    for actor in 0..2 { for lever in 0..5 {
        let mut o = Clockwork::load(a, map)?;
        o.saved.age = 100.; o.saved.initialized = true;
        if lever >= 2 { o.saved.levers[0] = Some(1.); o.saved.levers[1] = Some(2.); }
        if lever >= 3 { o.saved.levers[2] = Some(5.); o.saved.hare = Some(10.); }
        if lever == 4 {
            o.saved.levers[3] = Some(12.); o.saved.port = Some(30.); o.saved.gryphon = Some(40.);
            o.saved.extendo = Some(60.); o.saved.cubes = [true; 4]; o.saved.clock = Some(90.);
        }
        o.rebuild(map)?; let mut w = world(map, &o)?; let at = o.levers[lever];
        let feet = w.actor_footing(at.point(vec3(-64., 0., 60.)), PLAYER_CENTER, PLAYER_HALF, 128.).context("Lever approach lacks support")?;
        let mut p = people([vec3(10000., 10000., 0.); 2]); p[actor] = people([feet; 2])[0].clone();
        let partner = serde_json::to_value(&p[1 - actor])?; let mut s = stats();
        o.coop_prepare(&mut slots(&mut p, &mut s)); s[0].damage(17.); s[1].damage(41.);
        let aim = (at.translation + Vec3::Z * 20. - p[actor].eye()).normalize();
        o.coop_update(actor, &mut w, &p[actor], aim, true);
        ensure!(o.saved.pull.is_some() && o.coop_controlled(actor) && !o.coop_controlled(1 - actor), "Lever did not bind its actual user");
        o.coop_update(1 - actor, &mut w, &p[1 - actor], aim, true);
        ensure!(o.saved.coop.as_ref().unwrap().actor == Some(actor), "Partner stole lever action");
        for n in 0..960 {
            tick(&mut o, map, &mut w, &mut p, &mut s)?;
            ensure!(serde_json::to_value(&p[1 - actor])? == partner, "Lever action moved another player");
            ensure!(s[0].sanity() == 83. && s[1].sanity() == 59., "Lever reset personal health");
            if n == 60 { o = roundtrip(a, map, &o)?; }
            if o.scripted() {
                ensure!(o.coop_contacts(FIXED_DT, &mut slots(&mut p, &mut s)) == Some(actor), "Delayed scene selected the wrong lever owner");
                break;
            }
        }
        ensure!(o.saved.levers[lever].is_some() && o.saved.pull.is_none(), "Personal lever never completed");
        if lever == 3 { ensure!(o.saved.scene.as_ref().is_some_and(|s| s.kind == Kind::Port), "Missing delayed Port scene"); }
        if lever == 4 { ensure!(o.saved.scene.as_ref().is_some_and(|s| s.kind == Kind::Stop), "Missing post-hand Stop scene"); }
        println!("PASS Clockwork lever{lever} actor{actor}: actual use, personal approach, saved hand action and scene owner");
    }}
    Ok(())
}
fn sinks(a: &mut Assets, map: &Bsp) -> Result<()> {
    for reversed in [false, true] {
        let mut o = Clockwork::load(a, map)?; o.saved.initialized = true;
        o.saved.age = 100.; o.saved.levers[0] = Some(0.); o.saved.levers[1] = Some(1.); o.rebuild(map)?;
        let mut w = world(map, &o)?; let mut indices = Vec::new(); let mut feet = Vec::new();
        for id in [808, 809] {
            let k = o.objects.iter().position(|o| o.id == id).unwrap(); indices.push(k);
            let at = o.objects[k].base.translation;
            feet.push(w.actor_footing(vec3(at.x, at.y, 340.), PLAYER_CENTER, PLAYER_HALF, 60.).context("Paired sinking pillar lacks support")?);
        }
        let mut p = people([feet[0], feet[1]]); if reversed { p.reverse(); }
        let mut s = stats(); ensure!(clear(&w, &p), "Invalid paired pillar fixture");
        for _ in 0..6 { tick(&mut o, map, &mut w, &mut p, &mut s)?; ensure!(clear(&w, &p), "Sinking pillars embedded a rider"); }
        ensure!(indices.iter().all(|&k| o.saved.sinks[k] > 50.), "Both occupied pillars did not respond");
        o = roundtrip(a, map, &o)?;
        p = people([vec3(10000., 10000., 0.), vec3(10100., 10000., 0.)]);
        for _ in 0..252 { tick(&mut o, map, &mut w, &mut p, &mut s)?; }
        ensure!(indices.iter().all(|&k| o.saved.sinks[k] == 0.), "Empty paired pillars did not reset");
        println!("PASS Clockwork two sinking pillars reversed{reversed}: independent occupancy, saved state and reset");
    }
    Ok(())
}
fn partner_blocks_pull(a: &mut Assets, map: &Bsp) -> Result<()> {
    for actor in 0..2 {
        let mut o = Clockwork::load(a, map)?; let mut w = world(map, &o)?;
        let at = o.levers[0];
        let approach = w.actor_footing(at.point(vec3(-64., 0., 60.)), PLAYER_CENTER, PLAYER_HALF, 128.).context("Blocked pull approach")?;
        let goal = w.actor_footing(at.point(vec3(-2.65, 0., 0.2)) + Vec3::Z * 16., PLAYER_CENTER, PLAYER_HALF, 64.).context("Blocked pull goal")?;
        let mut p = people([goal; 2]); p[actor] = people([approach; 2])[0].clone(); let mut s = stats();
        ensure!(clear(&w, &p), "Invalid blocked pull fixture");
        let partner = serde_json::to_value(&p[1 - actor])?;
        let aim = (at.translation + Vec3::Z * 20. - p[actor].eye()).normalize();
        o.coop_prepare(&mut slots(&mut p, &mut s));
        o.coop_update(actor, &mut w, &p[actor], aim, true);
        ensure!(o.saved.pull.is_some(), "Blocked pull never requested");
        for _ in 0..240 {
            tick(&mut o, map, &mut w, &mut p, &mut s)?;
            ensure!(clear(&w, &p) && serde_json::to_value(&p[1 - actor])? == partner, "Lever approach overlapped or displaced its partner");
        }
        ensure!(o.saved.pull.is_none() && o.saved.levers[0].is_none(), "Blocked pull bypassed normal approach");
        println!("PASS Clockwork blocked lever actor{actor}: retains personal space and normal approach timeout");
    }
    Ok(())
}
fn death(a: &mut Assets, map: &Bsp) -> Result<()> {
    for actor in 0..2 {
        let mut o = Clockwork::load(a, map)?;
        o.saved.initialized = true; o.saved.age = 100.;
        o.saved.levers = [Some(1.), Some(2.), Some(5.), Some(12.), None];
        o.saved.hare = Some(10.); o.saved.port = Some(30.); o.saved.gryphon = Some(40.);
        o.saved.extendo = Some(60.); o.saved.cubes = [true; 4]; o.saved.clock = Some(90.);
        o.rebuild(map)?; let mut w = world(map, &o)?; let at = o.levers[4];
        let feet = w.actor_footing(at.point(vec3(-64., 0., 60.)), PLAYER_CENTER, PLAYER_HALF, 128.).context("Final lever death approach")?;
        let mut p = people([vec3(10000., 10000., 0.); 2]); p[actor] = people([feet; 2])[0].clone(); let mut s = stats();
        o.coop_prepare(&mut slots(&mut p, &mut s));
        let aim = (at.translation + Vec3::Z * 20. - p[actor].eye()).normalize();
        o.coop_update(actor, &mut w, &p[actor], aim, true);
        for _ in 0..180 {
            tick(&mut o, map, &mut w, &mut p, &mut s)?;
            if o.saved.pull.as_ref().is_some_and(|p| p.started) { break; }
        }
        ensure!(o.saved.pull.as_ref().is_some_and(|p| p.started) && o.saved.scene.is_none(), "Final hand action did not defer scene");
        s[1 - actor].damage(1000.); o.coop_prepare(&mut slots(&mut p, &mut s));
        ensure!(o.saved.pull.is_some() && o.saved.levers[4].is_some(), "Partner death cancelled the owner's hand action");
        s[actor].damage(1000.); o.coop_prepare(&mut slots(&mut p, &mut s));
        ensure!(o.saved.pull.is_none() && o.saved.levers[4].is_none() && o.saved.scene.is_none(), "Owner death left final lever unusable");
        let age = o.saved.age; tick(&mut o, map, &mut w, &mut p, &mut s)?;
        ensure!(o.saved.age == age, "Dead team advanced machinery");
        s[actor].set_health(25.)?; o.coop_prepare(&mut slots(&mut p, &mut s));
        ensure!(s[actor].sanity() == 25., "Preparation healed a revived player");
        o = roundtrip(a, map, &o)?;
        ensure!(o.saved.levers[4].is_none(), "Save lost usable final lever after death");
        println!("PASS Clockwork final lever actor{actor}: partner death, owner cancellation and low-health recovery");
    }
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/hatter1.bsp")?)?;
    let mut failures = Vec::new();
    for (name, check) in [("clocks", clocks as fn(&mut Assets, &Bsp) -> Result<()>),
        ("rides", rides), ("gear steering", gear_steering), ("corridor landings",corridor_landings),
        ("consecutive jumps",consecutive_jumps), ("gap steering",gap_steering), ("pathnodes",pathnodes), ("boarding", boarding), ("levers", levers), ("sinks", sinks),
        ("blocked pull", partner_blocks_pull), ("death", death), ("closing doors", super::coop_door_check::check)] {
        if let Err(error) = check(a, &map) { eprintln!("FAIL Clockwork {name}: {error:#}"); failures.push(name); }
    }
    ensure!(failures.is_empty(), "Clockwork paired components failed: {failures:?}"); Ok(())
}

fn pathnodes(assets:&mut Assets,map:&Bsp)->Result<()> {
    let mut o=Clockwork::load(assets,map)?;
    o.saved.age=100.;o.saved.levers[0]=Some(1.);o.saved.levers[1]=Some(2.);o.saved.levers[2]=Some(3.);o.rebuild(map)?;
    let w=world(map,&o)?;
    let course:Vec<serde_json::Value>=serde_json::from_str(include_str!("route_steps.json"))?;
    for step in &course[44..60] {
        let id=step["node"].as_u64().context("Changed Clockwork pathnode section")? as usize;
        let (_,feet)=crate::coop::route_driver::pathnode_goal(&w,map.entities.get(id).context("Missing pathnode")?)?;
        ensure!(w.body_clear(feet),"Projected pathnode {id} embeds the player");
        let hit=w.body_trace(feet,feet-Vec3::Z*2.);
        ensure!(!hit.start_solid && hit.fraction<1. && hit.normal.z>=0.65,"Projected pathnode {id} lacks support");
    }
    println!("PASS Clockwork sixteen post-gap pathnode projections: clear and supported native footings");
    Ok(())
}

/// A staged native-map regression for the remote diagnostic driver. This is
/// not a full route or paired campaign completion claim.
fn corridor_landings(assets:&mut Assets,map:&Bsp)->Result<()> {
    use crate::movement::Controls;
    use std::collections::VecDeque;
    let o=Clockwork::load(assets,map)?;let w=world(map,&o)?;
    let goal=vec3(-5120.,2544.,320.03125);
    let mut start=Player::new(vec3(-5235.65185546875,2402.392822265625,320.03125));start.grounded=true;
    // Captured old route: ordinary timing lands, but 50 ms extra northward
    // input makes the body miss the edge before final landing control applies.
    for extra in [0,6] {
        let mut p=start.clone();
        for (index,wish) in [vec2(1.,1.).normalize(),Vec2::Y,Vec2::Y,vec2(1.,1.).normalize(),
            vec2(1.,1.).normalize(),vec2(1.,-1.).normalize(),-Vec2::X].into_iter().enumerate() {
            for t in 0..24+if index==2 {extra} else {0} {
                p.tick(&w,Controls{wish,run:true,jump:index==0&&t==0,..Default::default()});
            }
        }
        ensure!(if extra==0 {p.grounded && p.feet.distance(goal)<22.} else {!p.grounded && p.feet.z<goal.z-60.},
            "Old corridor timing reproduction changed: extra{extra}, {:?}",p.feet);
    }
    let plan=crate::route::Route::plan_with_takeoff_margin(&w,&start,goal,22.,12)
        .map_err(|states|anyhow::anyhow!("No supported corridor route after {states} states"))?;
    let mut cases=0;
    for delay in [0,6,12,18] { for changed in 0..plan.len() { for extra in [-6,0,6] {
        let sequence=plan.iter().enumerate().flat_map(|(i,(c,_))| {
            (0..24+if i==changed {extra} else {0}).map(move |t|Controls{jump:c.jump&&t==0,..*c})
        }).collect::<Vec<_>>();
        let mut p=start.clone();
        let mut views=VecDeque::from(vec![p.clone();delay+1]);
        let mut pending=VecDeque::from(vec![Controls::default();delay]);
        for tick in 0..720 {
            let view=views.pop_front().unwrap();
            let control=crate::coop::route_driver::landing_control(&w,&view,goal,2.*delay as f32/120.,8.)
                .map(|(wish,_)|Controls{wish,..Default::default()})
                .unwrap_or_else(||sequence.get(tick).copied().unwrap_or_default());
            pending.push_back(control);p.tick(&w,pending.pop_front().unwrap());views.push_back(p.clone());
            ensure!(p.feet.z>=goal.z-1.,"Remote plan fell delay{delay} segment{changed} extra{extra} tick{tick}: {:?}",p.feet);
        }
        ensure!(crate::coop::route_driver::landing_control(&w,&p,goal,0.,8.).is_some_and(|(_,settled)|settled),
            "Remote plan did not settle delay{delay} segment{changed} extra{extra}: {:?}",p.feet);
        cases+=1;
    }}}
    println!("PASS Clockwork remote corridor landing: old timing failure reproduced; {cases} delayed and varied-input arrivals");
    Ok(())
}

fn consecutive_jumps(assets:&mut Assets,map:&Bsp)->Result<()> {
    use crate::movement::Controls;
    use crate::coop::route_driver::{landing_control,pending_jump};
    use std::collections::VecDeque;
    let o=Clockwork::load(assets,map)?;let w=world(map,&o)?;
    let goal=vec3(-5080.,2784.,320.03125);
    let mut start=Player::new(vec3(-5115.419921875,2547.447998046875,320.03125));start.grounded=true;
    let plan=crate::route::Route::plan_with_takeoff_margin(&w,&start,goal,22.,12)
        .map_err(|states|anyhow::anyhow!("No consecutive jump route after {states} states"))?;
    let jumps=plan.iter().filter(|(c,_)|c.jump).count() as u64;
    ensure!(jumps>=2,"Consecutive jump fixture no longer needs two takeoffs");
    let sequence=plan.iter().flat_map(|(c,_)|(0..24).map(move |t|Controls{jump:c.jump&&t==0,..*c})).collect::<Vec<_>>();
    for (delay,legacy) in [(0,true),(0,false),(6,false),(12,false),(18,false)] {
        let mut o=Clockwork::load(assets,map)?;
        o.saved.initialized=true;o.saved.age=100.;o.saved.levers[0]=Some(1.);o.saved.levers[1]=Some(2.);o.rebuild(map)?;
        let mut w=world(map,&o)?;
        let mut pair=people([vec3(-5296.,3008.,320.03125),start.feet]);let mut health=stats();
        let mut fell=false;let mut handed_off=false;
        let mut views=VecDeque::from(vec![(pair[1].clone(),w.clone());delay+1]);
        let mut pending=VecDeque::from(vec![Controls::default();delay]);
        for step in 0..960 {
            let (view,seen_world)=views.pop_front().unwrap();
            let remains=pending_jump(sequence.iter().skip(step).map(|c|c.jump),false);
            let landing=(!remains || legacy).then(||landing_control(&seen_world,&view,goal,2.*delay as f32/120.,8.)).flatten();
            let control=if let Some((wish,_))=landing {handed_off=true;Controls{wish,..Default::default()}}
                else if handed_off {Controls::default()}
                else {sequence.get(step).copied().unwrap_or_default()};
            pending.push_back(control);pair[1].tick(&w,pending.pop_front().unwrap());
            tick(&mut o,map,&mut w,&mut pair,&mut health)?;
            views.push_back((pair[1].clone(),w.clone()));
            if pair[1].feet.z<goal.z-35. {fell=true;break;}
        }
        let p=&pair[1];
        if legacy {ensure!(fell && p.jumps<jumps,"Old premature handoff did not reproduce the missing jump: {:?}",p.feet);}
        else {ensure!(!fell && p.jumps==jumps && landing_control(&w,p,goal,0.,8.).is_some_and(|(_,settled)|settled),
            "Consecutive jumps failed delay{delay}: {:?}, {}/{jumps} jumps",p.feet,p.jumps);}
    }
    println!("PASS Clockwork consecutive jumps: old premature handoff reproduced; both takeoffs and settled landing at four delays");
    Ok(())
}

#[cfg(test)]
mod consecutive_probe {
    use super::*;
    #[test]
    #[ignore = "requires LOOKING_GLASS_TEST_DATA"]
    fn native_consecutive_jump_probe() {
        let base=std::env::var_os("LOOKING_GLASS_TEST_DATA").unwrap();
        let mut assets=Assets::open(std::path::Path::new(&base)).unwrap();
        let map=Bsp::parse(&assets.read("maps/hatter1.bsp").unwrap()).unwrap();
        consecutive_jumps(&mut assets,&map).unwrap();
    }
    #[test]
    #[ignore = "requires LOOKING_GLASS_TEST_DATA"]
    fn native_gap_steering_probe() {
        let base=std::env::var_os("LOOKING_GLASS_TEST_DATA").unwrap();
        let mut assets=Assets::open(std::path::Path::new(&base)).unwrap();
        let map=Bsp::parse(&assets.read("maps/hatter1.bsp").unwrap()).unwrap();
        gap_steering(&mut assets,&map).unwrap();
    }
}

fn gap_steering(assets:&mut Assets,map:&Bsp)->Result<()> {
    use crate::movement::Controls;
    use crate::coop::route_driver::{direct_goal_wish,straight_wish,landing_control};
    use std::collections::VecDeque;
    let goal=vec3(-5080.,3024.,320.03125);
    for (delay,legacy) in [(18,true),(0,false),(6,false),(12,false),(18,false),(24,false),(30,false)] {
        let mut o=Clockwork::load(assets,map)?;let mut w=world(map,&o)?;
        let mut p=people([vec3(-5296.,3008.,320.03125),vec3(-5079.9365234375,2793.36669921875,320.03125)]);
        let mut health=stats();let mut fell=false;
        let mut views=VecDeque::from(vec![(p[1].clone(),w.clone());delay+1]);
        let mut pending=VecDeque::from(vec![Controls::default();delay]);
        for step in 0..960 {
            let (view,seen_world)=views.pop_front().unwrap();
            let lead=2.*delay as f32/120.;
            let wish=if step<12 {Vec2::Y}
                else if legacy {straight_wish((goal-view.feet).truncate(),view.velocity.truncate(),lead,true,None)}
                else {direct_goal_wish(&seen_world,&view,goal,lead,true,None)};
            pending.push_back(Controls{wish,run:true,jump:step==12,..Default::default()});
            p[1].tick(&w,pending.pop_front().unwrap());tick(&mut o,map,&mut w,&mut p,&mut health)?;
            views.push_back((p[1].clone(),w.clone()));
            if p[1].feet.z<goal.z-35. {fell=true;break;}
        }
        if legacy {ensure!(fell && p[1].jumps==1,"Old early gap braking no longer reproduces: {:?}",p[1].feet);}
        else {ensure!(!fell && p[1].jumps==1 && landing_control(&w,&p[1],goal,0.,8.).is_some_and(|(_,settled)|settled),
            "Gap steering failed delay{delay}: {:?}, velocity{:?}",p[1].feet,p[1].velocity);}
    }
    println!("PASS Clockwork direct gap steering: old early braking reproduced; one actual jump and settled landing at six delays");
    Ok(())
}
