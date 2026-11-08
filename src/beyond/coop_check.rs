//! Staged native adapter checks on authored geometry, not a full campaign route.
use super::*;
use crate::{assets::Assets, coop::{protocol::{Input, Visit}, stage::Stage, world::Roster, PlayerId},
    inventory::{self, Catalog, Stats}, movement::{Controls, FixedClock, EYE_HEIGHT, FIXED_DT}};
use anyhow::Context;

fn player(world: &World, eye: Vec3) -> Result<Player> {
    let mut p = Player::spawn(world, eye).context("Beyond co-op fixture has no support")?;
    for _ in 0..120 { p.tick(world, Controls::default()); }
    ensure!(p.grounded && world.body_clear(p.feet), "Beyond fixture is not grounded/clear at {:?}", p.feet);
    Ok(p)
}
fn inputs() -> [(Controls, Input); 2] { std::array::from_fn(|_| (Controls::default(), Input::default())) }
fn lift_origin(b: &Beyond) -> Vec3 { b.objects.iter().find(|o|o.name == "ragelift").unwrap().pose.0 }
// Ground tracing leaves a 1/32-unit collision margin above an exact spawn.
const SUPPORT_MARGIN: f32 = 1. / 32. + 0.0001;
fn setup(assets: &mut Assets, map: &Bsp) -> Result<(Stage, World, Roster)> {
    let mut world = World::from_bsp(map)?;
    let visit = Visit { epoch:1, map:"fortress2".into(), entry:None, difficulty:map.difficulty, combat_preview:false };
    let stage = Stage::load(assets, map, &mut world, &visit, false)?;
    let (eye, yaw) = crate::interaction::spawn(map, None);
    let roster = Roster::new(&world, eye, yaw, Stats::for_level("fortress2", None))?;
    Ok((stage, world, roster))
}
fn step(stage: &mut Stage, roster: &mut Roster, map: &Bsp, world: &mut World,
    controls: [(Controls, Input); 2]) -> Result<()> {
    if stage.busy() {
        stage.scene_step(roster, map, world, &controls)?;
    } else {
        stage.before_move(roster, map, world, &controls, FIXED_DT)?;
        if stage.busy() {
            roster.tick += 1;
        } else {
            let before = roster.players.each_ref().map(|b| b.player.feet);
            roster.move_players(world, controls);
            stage.contacts(roster, world, before)?;
        }
    }
    ensure!(roster.players.iter().all(|b| b.stats.alive()), "Staged adapter injured a player");
    Ok(())
}
fn carry(map: &Bsp) -> Result<()> {
    for fps in [30,60,144] {
        let mut b = Beyond::load(map)?;
        b.state.rage_started = Some(0.);
        b.state.rage_lift = Some(RageLift::Lowering(0.));
        let mut world = World::from_bsp(map)?;
        world.set_dynamic(b.colliders().collect());
        let mut players = [player(&world,vec3(-160.,3552.,88.))?,player(&world,vec3(-112.,3552.,88.))?];
        ensure!(players.iter().all(|p| b.aboard("ragelift",p.feet)), "Both fixture riders must board the Rage lift");
        let start = players.each_ref().map(|p|p.feet);
        let paused = serde_json::to_value(b.snapshot())?;
        ensure!(paused.get("lift_wait").is_none(), "Unused co-op wait changed single-player save output");
        b.advance_players(0.,map,&mut world,&mut players.each_mut(),&[])?;
        ensure!(paused == serde_json::to_value(b.snapshot())? && start == players.each_ref().map(|p|p.feet), "Paused paired lift moved");
        let mut solo = Beyond::load(map)?;
        solo.restore(&b.snapshot(),map)?;
        let mut solo_world = World::from_bsp(map)?;
        solo_world.set_dynamic(solo.colliders().collect());
        let mut one = players[0].clone();
        let mut clocks: [FixedClock;2] = Default::default();
        let mut solo_clock = FixedClock::default();
        for frame in 0..fps*5 {
            let dt = 1. / fps as f32;
            if frame % fps == 0 {
                let state = serde_json::from_slice(&serde_json::to_vec(&b.snapshot())?)?;
                let mut restored = Beyond::load(map)?;
                restored.restore(&state,map)?;
                let mut w = World::from_bsp(map)?;
                w.set_dynamic(restored.colliders().collect());
                let mut copy = players.clone();
                restored.advance_players(dt,map,&mut w,&mut copy.each_mut(),&[])?;
                b.advance_players(dt,map,&mut world,&mut players.each_mut(),&[])?;
                ensure!(copy.each_ref().map(|p|p.feet) == players.each_ref().map(|p|p.feet)
                    && serde_json::to_value(restored.snapshot())? == serde_json::to_value(b.snapshot())?, "Restored paired lift diverged");
            } else { b.advance_players(dt,map,&mut world,&mut players.each_mut(),&[])?; }
            solo.advance(dt,map,&mut solo_world,&mut one,&[])?;
            ensure!(one.feet == players[0].feet && solo.state.age == b.state.age, "Pair changed the original one-rider clock/carry");
            for (i,p) in players.iter_mut().enumerate() {
                ensure!(world.body_clear(p.feet), "Paired lift embedded rider {i}");
                clocks[i].advance(dt as f64,&world,p,Controls::default());
            }
            solo_clock.advance(dt as f64,&solo_world,&mut one,Controls::default());
        }
        ensure!(players.iter().zip(start).all(|(p,at)|(p.feet-at+Vec3::Z*48.).length()<0.1), "Lift did not lower both riders 48 units");
        println!("PASS Beyond paired Rage lift {fps} Hz: both carried, one clock, pause, JSON restore and single-player parity");
    }
    // The later elevator rises. A low ceiling over either rider must roll back
    // both proposed positions and the shared clock, then resume when removed.
    for blocked in 0..2 {
        let mut b = Beyond::load(map)?;
        b.state.age = 4.; b.state.rage_started = Some(0.);
        b.state.rage_lift = Some(RageLift::Lowering(0.)); b.state.last_started = Some(4.);
        b.rebuild(map)?;
        let mut world = World::from_bsp(map)?; world.set_dynamic(b.colliders().collect());
        let mut players = [player(&world,vec3(-160.,3552.,48.))?,player(&world,vec3(-112.,3552.,48.))?];
        let start = players.each_ref().map(|p|p.feet);
        let ceiling = Collider::box_bounds(start[blocked]+vec3(-18.,-18.,PLAYER_HALF.z*2.+0.1),
            start[blocked]+vec3(18.,18.,PLAYER_HALF.z*2.+40.));
        for _ in 0..120 { b.advance_players(FIXED_DT,map,&mut world,&mut players.each_mut(),std::slice::from_ref(&ceiling))?; }
        ensure!(b.state.age == 4. && start == players.each_ref().map(|p|p.feet), "Blocked rider {blocked} did not stop both elevator carries");
        b.advance_players(FIXED_DT,map,&mut world,&mut players.each_mut(),&[])?;
        ensure!(players.iter().zip(start).all(|(p,at)|p.feet.z>at.z), "Elevator did not resume after removing ceiling");
    }
    Ok(())
}
fn bridge(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for first in PlayerId::ALL {
        let (mut stage,mut world,mut roster) = setup(assets,map)?;
        let b = stage.interactions.beyond.as_ref().unwrap();
        let rear = player(&world,b.gs(1)+Vec3::Z*(4.+EYE_HEIGHT))?;
        let center = player(&world,b.gs(2)+Vec3::Z*(4.+EYE_HEIGHT))?;
        let front = player(&world,b.gs(3)+Vec3::Z*(4.+EYE_HEIGHT))?;
        ensure!(b.boarding_ready("GS_Move",&[front.feet,rear.feet]) == Some(false)
            && b.boarding_ready("GS_Move",&[front.feet,center.feet]) == Some(true)
            && b.boarding_ready("GS_MoveBack",&[center.feet,front.feet]) == Some(false)
            && b.boarding_ready("GS_MoveBack",&[center.feet,rear.feet]) == Some(true)
            && b.boarding_ready("GS_Move",&[front.feet]) == Some(true), "Rolling walkway boarding areas are wrong");
        roster.players[first.index()].player = front;
        roster.players[first.partner().index()].player = rear;
        for _ in 0..30 { step(&mut stage,&mut roster,map,&mut world,inputs())?; }
        let b = stage.interactions.beyond.as_ref().unwrap();
        ensure!(b.state.walkway == 2 && !b.sliding() && stage.notices[first.index()].is_some(), "Bridge left the trailing player or missed the waiting notice");
        roster.players[first.partner().index()].player = center;
        for _ in 0..120 { step(&mut stage,&mut roster,map,&mut world,inputs())?; }
        let b = stage.interactions.beyond.as_ref().unwrap();
        ensure!(b.state.walkway == 3 && !b.sliding() && roster.players.iter().all(|p| world.body_clear(p.player.feet)), "Bridge failed to advance with both aboard");
        println!("PASS Beyond rolling bridge first {first:?}: rear rider waits, stable tiles board both, one forward move");
    }
    Ok(())
}
fn elevator(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for first in PlayerId::ALL {
        let (mut stage,mut world,mut roster) = setup(assets,map)?;
        let b = stage.interactions.beyond.as_mut().unwrap();
        b.state.age = 4.; b.state.rage_started = Some(0.);
        b.state.rage_lift = Some(RageLift::Lowering(0.)); b.state.last_started = Some(4.);
        b.rebuild(map)?;
        stage.interactions.sync_coop(&mut world);
        let a = first.index(); let other = first.partner().index();
        let away = roster.players[other].player.clone();
        roster.players[a].player = player(&world,vec3(-160.,3552.,48.))?;
        let start = roster.players[a].player.feet;
        let lift_start = lift_origin(stage.interactions.beyond.as_ref().unwrap());
        for _ in 0..120 { step(&mut stage,&mut roster,map,&mut world,inputs())?; }
        ensure!(stage.interactions.beyond.as_ref().unwrap().state.age > 4.9
            && lift_origin(stage.interactions.beyond.as_ref().unwrap()) == lift_start
            && roster.players[a].player.feet.distance(start) <= SUPPORT_MARGIN && stage.notices[a].is_some(),
            "Cycling elevator left lone rider {first:?} or froze the other machinery: age {}, wait {}, start {start:?}, players {:?}, clear {:?}, notice {}",
            stage.interactions.beyond.as_ref().unwrap().state.age,
            stage.interactions.beyond.as_ref().unwrap().state.lift_wait,
            roster.players.each_ref().map(|p|p.player.feet),
            roster.players.each_ref().map(|p|world.body_clear(p.player.feet)), stage.notices[a].is_some());
        // Boarding is derived from the restored scene and live bodies, not an
        // unsaved timer that could disappear on reconnect/checkpoint reload.
        let state = serde_json::from_slice(&serde_json::to_vec(&stage.snapshot())?)?;
        stage.restore(&state,map,&mut world)?;
        let b = stage.interactions.beyond.as_mut().unwrap();
        for wait in [-1., f64::NAN, f64::INFINITY, b.state.age as f64 + 1.] {
            let mut invalid = b.snapshot(); invalid.lift_wait = wait;
            ensure!(b.restore(&invalid,map).is_err(), "Accepted an invalid saved elevator wait");
        }
        step(&mut stage,&mut roster,map,&mut world,inputs())?;
        ensure!(lift_origin(stage.interactions.beyond.as_ref().unwrap()) == lift_start
            && roster.players[a].player.feet.distance(start) <= SUPPORT_MARGIN,
            "Restore bypassed the elevator wait: {start:?} -> {:?}", roster.players[a].player.feet);
        roster.players[other].player = player(&world,vec3(-112.,3552.,48.))?;
        for _ in 0..120 { step(&mut stage,&mut roster,map,&mut world,inputs())?; }
        ensure!(roster.players.iter().all(|p|p.player.feet.z > start.z + 90.), "Boarded elevator did not carry both upward: {:?}", roster.players.each_ref().map(|p|p.player.feet));
        // A departing rider holds the remaining one. Once both have left, the
        // unoccupied elevator must be free to cycle back for the next boarding.
        roster.players[other].player = away.clone();
        let age = stage.interactions.beyond.as_ref().unwrap().state.age;
        let stopped = roster.players[a].player.feet;
        let lift_stopped = lift_origin(stage.interactions.beyond.as_ref().unwrap());
        step(&mut stage,&mut roster,map,&mut world,inputs())?;
        ensure!(lift_origin(stage.interactions.beyond.as_ref().unwrap()) == lift_stopped
            && roster.players[a].player.feet.distance(stopped) <= SUPPORT_MARGIN
            && stage.interactions.beyond.as_ref().unwrap().state.age > age,
            "Elevator left the second disembarking rider or froze the other machinery: age {age} -> {}, {stopped:?} -> {:?}",
            stage.interactions.beyond.as_ref().unwrap().state.age, roster.players[a].player.feet);
        roster.players[a].player = away.clone();
        step(&mut stage,&mut roster,map,&mut world,inputs())?;
        let b = stage.interactions.beyond.as_ref().unwrap();
        ensure!(lift_origin(b).z > lift_stopped.z,
            "Empty elevator cannot return");
        // A dead partner must never become a permanent boarding requirement.
        let b = stage.interactions.beyond.as_ref().unwrap();
        let lift = b.objects.iter().find(|o|o.name == "ragelift").unwrap();
        let top = lift.pose.0 + vec3(0.,0.,192. + EYE_HEIGHT);
        roster.players[a].player = player(&world,top)?;
        roster.players[other].stats.set_health(0.)?;
        let before = roster.players[a].player.feet;
        stage.before_move(&mut roster,map,&mut world,&inputs(),FIXED_DT)?;
        ensure!(roster.players[a].player.feet.z > before.z, "Dead partner prevented surviving rider's elevator use");
        println!("PASS Beyond cycling elevator first {first:?}: boarding wait/notice, saved wait, other machinery keeps running, paired ride, disembarking hold, empty return and sole survivor");
    }
    Ok(())
}
fn late_rage(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let catalog = Catalog::load(assets)?;
    let pickups = inventory::pickups(map,"fortress2",&catalog);
    for first in PlayerId::ALL {
        for skip in [false,true] {
            let (mut stage,mut world,mut roster) = setup(assets,map)?;
            let b = stage.interactions.beyond.as_mut().unwrap();
            b.state.last_started = Some(0.);
            b.state.age = 4.5;
            b.rebuild(map)?;
            stage.interactions.sync_coop(&mut world);
            let a = first.index(); let other = first.partner().index();
            roster.players[a].player = player(&world,vec3(-160.,3552.,88.))?;
            roster.players[other].player = player(&world,vec3(-96.,3552.,88.))?;
            let waiting = roster.players[other].player.feet;
            let before = lift_origin(stage.interactions.beyond.as_ref().unwrap());
            stage.collect_pickups(&mut roster,&world,&pickups);
            ensure!(stage.owner == Some(first), "Late Rage fixture did not collect the box");
            let b = stage.interactions.beyond.as_mut().unwrap();
            b.rebuild(map)?;
            ensure!(lift_origin(b) == before, "Late Rage jumped the cycling elevator");
            stage.interactions.sync_coop(&mut world);
            ensure!(roster.players.iter().all(|p|world.body_clear(p.player.feet)), "Late Rage embedded a rider");
            let mut count = 0;
            while stage.busy() && count < 2400 {
                if count == 120 {
                    let paused = serde_json::to_value(stage.snapshot())?;
                    stage.interactions.advance_school(0.,map,&mut world,&mut roster.players[a].player)?;
                    ensure!(paused == serde_json::to_value(stage.snapshot())?, "Paused late Rage advanced");
                    let saved = serde_json::from_slice(&serde_json::to_vec(&stage.snapshot())?)?;
                    stage.restore(&saved,map,&mut world)?;
                }
                if skip && count == 240 {
                    ensure!(stage.interactions.skip_cinematic(map,&mut world,&mut roster.players[a].player,&mut stage.story)?,
                        "Late Rage skip rejected");
                }
                step(&mut stage,&mut roster,map,&mut world,inputs())?;
                count += 1;
                if stage.busy() {
                    ensure!(roster.players[other].player.feet == waiting, "Late Rage moved its hidden partner");
                    ensure!(world.body_clear(roster.players[a].player.feet), "Late Rage embedded its owner");
                }
            }
            ensure!(!stage.busy() && (skip || count >= 1400), "Late Rage never completed its scene");
            for _ in 0..120 { step(&mut stage,&mut roster,map,&mut world,inputs())?; }
            let b = stage.interactions.beyond.as_ref().unwrap();
            ensure!(roster.players.iter().all(|p| p.player.grounded && world.body_clear(p.player.feet)
                && b.aboard("ragelift",p.player.feet)), "Late Rage failed to reunite both riders on the lift");
            println!("PASS Beyond late Rage first {first:?}, skip {skip}: continuous lift, pause/save/restore, frozen partner and supported reunion");
        }
    }
    Ok(())
}
fn interactions(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let catalog = Catalog::load(assets)?;
    let pickups = inventory::pickups(map,"fortress2",&catalog);
    for first in PlayerId::ALL {
        let (mut stage,mut world,mut roster) = setup(assets,map)?;
        let a = first.index(); let other = first.partner().index();
        roster.players[a].player = player(&world,vec3(-160.,3552.,88.))?;
        let start = roster.players[a].player.feet;
        stage.collect_pickups(&mut roster,&world,&pickups);
        ensure!(!stage.busy() && stage.notices[a].is_some() && !roster.players[a].stats.collected.contains("fortress2:37"), "Rage pickup did not wait for the partner");
        roster.players[other].player = player(&world,vec3(-96.,3552.,88.))?;
        let waiting = roster.players[other].player.feet;
        stage.collect_pickups(&mut roster,&world,&pickups);
        ensure!(stage.owner == Some(first) && roster.players[a].stats.collected.contains("fortress2:37")
            && !roster.players[other].stats.collected.contains("fortress2:37"), "Rage collection/scene ownership was not individual");
        let mut count = 0;
        while stage.busy() && count < 2400 {
            let mut control = inputs(); control[a].0.use_pressed = count % 30 == 0;
            step(&mut stage,&mut roster,map,&mut world,control)?; count+=1;
            if stage.busy() { ensure!(roster.players[other].player.feet == waiting, "Hidden partner moved during Rage"); }
        }
        ensure!(!stage.busy() && count >= 1400, "Rage did not finish its original scene");
        let b = stage.interactions.beyond.as_ref().unwrap();
        ensure!(roster.players.iter().all(|p|b.aboard("ragelift",p.player.feet)), "Rage reunion left a rider outside the lift");
        for _ in 0..600 { step(&mut stage,&mut roster,map,&mut world,inputs())?; }
        ensure!(roster.players.iter().all(|p|p.player.grounded && world.body_clear(p.player.feet)
            && (p.player.feet.z-start.z+48.).abs()<0.2), "Rage lift did not carry both after the reunion");
        // The uncollected copy remains reachable by an ordinary jump from the
        // lowered lift. It grants only the second player and never replays the scene.
        // The first collector must walk off the item's centre. Personal-space
        // contact now correctly prevents the partner walking through that body.
        let pickup_at=roster.players[a].player.feet;
        let waiting=roster.players[other].player.feet;
        let aside=[Vec3::Y,-Vec3::Y,-Vec3::X,Vec3::X].into_iter().map(|d|pickup_at+d*48.)
            .find(|at| {
                let floor=world.body_trace(*at,*at-Vec3::Z*4.);
                world.body_clear(*at) && !floor.start_solid && floor.fraction<1. && floor.normal.z>0.65
                    && at.truncate().distance(waiting.truncate())>=48.
                    && stage.interactions.beyond.as_ref().unwrap().aboard("ragelift",*at)
            }).context("Rage lift has no separate waiting space")?;
        for _ in 0..240 {
            let delta=(aside-roster.players[a].player.feet).truncate();
            if delta.length()<3. {break;}
            let mut control=inputs();control[a].0.wish=delta.normalize_or_zero()*(delta.length()/24.).min(1.);
            step(&mut stage,&mut roster,map,&mut world,control)?;
        }
        ensure!(roster.players[a].player.feet.truncate().distance(aside.truncate())<3.
            && roster.players.iter().all(|p|p.player.grounded && world.body_clear(p.player.feet)
                && stage.interactions.beyond.as_ref().unwrap().aboard("ragelift",p.player.feet)),
            "First Rage collector did not walk safely aside on the lift");
        let mut control = inputs(); control[other].0.wish = (pickup_at-roster.players[other].player.feet).truncate().normalize_or_zero();
        for _ in 0..55 { step(&mut stage,&mut roster,map,&mut world,control)?; }
        for tick in 0..90 {
            let mut control = inputs(); control[other].0.jump = tick == 0;
            step(&mut stage,&mut roster,map,&mut world,control)?;
            stage.collect_pickups(&mut roster,&world,&pickups);
            ensure!(!stage.busy(), "Second personal Rage copy replayed the shared scene");
        }
        ensure!(roster.players.iter().all(|p|p.stats.collected.contains("fortress2:37") && p.stats.powers.rage>0.), "Second player lost their personal Rage copy");
        println!("PASS Beyond Rage first {first:?}: all-board barrier, personal grant, original scene, frozen partner, lift reunion/descent and separate second pickup");

        let (mut stage,mut world,mut roster) = setup(assets,map)?;
        stage.interactions.beyond.as_mut().unwrap().state.puzzle_started = true;
        for (index,(id,y)) in [(first,465.),(first.partner(),593.),(first,529.)].into_iter().enumerate() {
            roster.players[id.index()].player = player(&world,vec3(1320.,y,-285.))?;
            let mut control = inputs(); control[id.index()].0.use_pressed = true;
            control[id.index()].1.yaw = 0.;
            if index == 2 {
                // The partner also retries their already-used lever on the
                // solving tick. The final operator still owns the scene.
                control[id.partner().index()].0.use_pressed = true;
                control[id.partner().index()].1.yaw = 0.;
            }
            stage.before_move(&mut roster,map,&mut world,&control,FIXED_DT)?;
        }
        ensure!(stage.interactions.beyond.as_ref().unwrap().state.solved && stage.owner == Some(first), "Either-player lever sequence/scene ownership failed");
        let sounds = stage.sounds.batch(roster.tick);
        sounds.validate(roster.tick)?;
        ensure!(sounds.cues.iter().map(|c|c.path.as_str()).collect::<Vec<_>>() == SOUNDS,
            "Shared musical notes did not enter the sound journal once in their played order");
        println!("PASS Beyond levers first {first:?}: players share the three-note sequence and final operator owns its scene");
    }
    Ok(())
}
pub fn check(assets: &mut Assets) -> Result<()> {
    println!("Beyond co-op adapters / build {}",env!("LOOKING_GLASS_BUILD"));
    let map = Bsp::parse(&assets.read("maps/fortress2.bsp")?)?;
    carry(&map)?;
    bridge(assets,&map)?;
    elevator(assets,&map)?;
    late_rage(assets,&map)?;
    interactions(assets,&map)
}
