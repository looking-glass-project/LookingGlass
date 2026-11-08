//! Full entrance-to-exit swimming with ordinary inputs in the production Arena.
//! No authored events, placements, resources or hazard states are written.
use super::*;
use crate::{coop::{arena::Arena, protocol::{Input, Visit}, world::Roster, PlayerId},
    movement::{Controls, FIXED_DT}, render::Scene};

fn owner(arena: &Arena) -> &Temple {
    arena.stage.interactions.levels.iter().find_map(|s| s.ctl.downcast_ref::<Temple>()).unwrap()
}

/// Owned, read-only search inputs. The network fixture can search a cloned
/// collision world off-thread without holding up simulation or packet delivery.
pub(crate) struct SwimSearch {
    player: Player,
    target: Vec3,
    hazards: Vec<Collider>,
}
impl SwimSearch {
    pub(crate) fn direct(&self, world: &World) -> Option<Vec3> {
        (world.body_trace(self.player.feet,self.target).fraction == 1.
            && super::swim_check::clear_future_leg(&self.hazards,self.player.feet,self.target)
            // Close hazards need the curved steering check in the background
            // worker even if the straight line appears clear. Keep the cheap
            // direct path only when no hull is within half a second's travel.
            && !self.hazards.iter().any(|h|h.touches(self.player.feet+PLAYER_CENTER,
                self.player.feet+PLAYER_CENTER,PLAYER_HALF+Vec3::splat(
                    16.+self.player.velocity.length().max(if self.player.tea {440.} else {220.})*0.5))))
            .then_some(self.target)
    }
    pub(crate) fn waypoint(self, world: &World) -> Vec3 {
        super::swim_check::waypoint(world,&self.player,self.target,&self.hazards)
    }
}

impl Temple {
    /// Read-only navigation for opt-in tests. Both native and separate-client
    /// drivers use this goal and send ordinary swimming controls toward it.
    pub(crate) fn test_swim_waypoint(&self, map: &Bsp, world: &World, player: &Player,
        leading: bool) -> Result<Vec3> {
        Ok(self.test_swim_search(map,world,player,leading)?.waypoint(world))
    }
    pub(crate) fn test_swim_search(&self, map: &Bsp, world: &World, player: &Player,
        leading: bool) -> Result<SwimSearch> {
        let mut hazards = self.motion.ahead(map,&self.saved,1.)?;
        hazards.extend(self.motion.ahead(map,&self.saved,0.5)?);
        hazards.extend(world.traversal.pushes.iter().filter(|p|p.enabled).map(|p|p.volume.clone()));
        let target = if self.saved.guide.is_none() {
            if leading { vec3(-88.,-5980.,1330.) } else { player.feet }
        } else if self.saved.open && leading {
            vec3(-2144.,3352.,488.)
        } else {
            let age = self.saved.clock - if leading {0.} else {1.};
            self.data.shell_pose(self.turtle_pose(age),age.max(0.) as f32).translation-PLAYER_CENTER
        };
        Ok(SwimSearch {player:player.clone(),target,hazards})
    }
}

async fn route(assets: &mut Assets, first: PlayerId, skip: bool) -> Result<()> {
    let visit = Visit { epoch:1, map:"utemple".into(), entry:None,
        difficulty:Default::default(), combat_preview:false };
    let mut scene = Scene::load_for(assets, &visit.map, visit.difficulty)?;
    let mut arena = Arena::load(assets, &scene.map, &mut scene.world, &visit, false)?;
    let (eye,yaw) = crate::interaction::spawn(&scene.map,None);
    let mut roster = Roster::new_with(&scene.world,eye,yaw,Stats::for_level("utemple",None),
        |at| !arena.stage.interactions.hazardous(at))?;
    arena.reset_characters(&roster.players);
    let mut art = crate::viewer::LevelArt::default();
    art.prepare(assets,&scene,&arena.stage.interactions,false)?;
    let mut replica_world = World::from_bsp(&scene.map)?;
    let mut replica = Arena::load(assets,&scene.map,&mut replica_world,&visit,false)?;
    let mut goals = roster.players.each_ref().map(|b|b.player.feet);
    let mut samples = 0;
    let mut minimum_air = [20_f32;2];
    let mut scene_seen = false;
    let mut previous_owner = None;
    for tick in 0..120*260 {
        let temple = owner(&arena);
        if tick%12 == 0 && !arena.stage.busy() {
            for id in PlayerId::ALL {
                let body = &roster.players[id.index()].player;
                goals[id.index()] = temple.test_swim_waypoint(&scene.map,&scene.world,body,id == first)?;
            }
        }
        let input = std::array::from_fn(|i| {
            let offset = goals[i]-roster.players[i].player.feet;
            (Controls { swim:offset.normalize_or_zero()*(offset.length()/30.).min(1.),run:true,
                ..Default::default() }, Input { yaw:roster.players[i].yaw,..Default::default() })
        });
        let before = roster.players.each_ref().map(|b|b.player.feet);
        if skip && arena.stage.busy() {
            let id = arena.stage.owner.context("Temple scene has no owner")?;
            arena.stage.interactions.skip_cinematic(&scene.map,&mut scene.world,
                &mut roster.players[id.index()].player,&mut arena.stage.story)?;
        }
        arena.step(&mut roster,&scene.map,&mut scene.world,input)?;
        if let Some(id) = arena.stage.owner {
            scene_seen = true;
            ensure!(id == first,"Temple route wrong first arrival: {id:?}, expected {first:?}");
            if previous_owner == Some(id) {
                ensure!(roster.players[id.partner().index()].player.feet == before[id.partner().index()],
                    "Temple route moved the waiting partner");
            }
        }
        previous_owner = arena.stage.owner;
        ensure!(roster.checkpoint_generation == 0 && roster.players.iter().all(|b|b.stats.alive() && b.spawned_at == 0),
            "Temple route death/wipe at tick{tick}: {:?}",roster.players.each_ref().map(|b|
                (b.player.feet,b.stats.sanity(),b.player.breath.remaining())));
        if !arena.stage.busy() {
            for (i,b) in roster.players.iter().enumerate() {
                ensure!(scene.world.body_clear(b.player.feet),"Temple swimmer{i} embedded at tick{tick}: {:?}",b.player.feet);
                ensure!(b.warped_at == 0,"Temple free traversal teleported swimmer{i}");
                minimum_air[i] = minimum_air[i].min(b.player.breath.remaining());
            }
        }
        if tick%600 == 0 {
            let encoded = crate::coop::codec::encode(&serde_json::to_vec(&arena.snapshot())?)?;
            replica.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&encoded)?)?,&scene.map,&mut replica_world)?;
            ensure!(serde_json::to_value(replica.snapshot())? == serde_json::to_value(arena.snapshot())?,
                "Temple full-route replica differs at tick{tick}");
            samples += 1;
            println!("Temple {first:?} skip={skip} t={} node={} feet={:?} health={:?} air={:?}",
                tick/120,owner(&arena).saved.node,roster.players.each_ref().map(|b|b.player.feet),
                roster.players.each_ref().map(|b|b.stats.sanity()),roster.players.each_ref().map(|b|b.player.breath.remaining()));
        }
        if tick%120 == 0 {
            clear_background(BLACK);
            let camera = arena.stage.camera(&scene.world).unwrap_or_else(|| {
                let at = roster.players[first.index()].player.feet;
                crate::cinematic::Camera::look(at+vec3(160.,180.,100.),at+PLAYER_CENTER)
            });
            let view = Camera3D {position:camera.eye,target:camera.target,up:camera.up,
                fovy:60_f32.to_radians(),z_near:2.,z_far:20000.,..Default::default()};
            set_camera(&view);
            crate::lighting::select(vec![],camera.eye,&scene.world);
            crate::render_fx::begin_view(&view,tick as f32*FIXED_DT,&scene.atmosphere,false);
            let transforms = arena.stage.draw_transforms(&scene.world);
            scene.draw(camera.eye,tick as f32*FIXED_DT,false,false,&transforms);
            art.draw(&arena.stage.interactions,&arena.stage.hints,&scene.atmosphere,camera.eye,false);
            if !arena.stage.busy() {
                for (character,body) in arena.characters.iter_mut().zip(&roster.players) {
                    character.atmosphere(&scene.atmosphere,camera.eye);
                    character.draw_replica(body.player.feet,body.player.feet,false);
                }
            }
            arena.npcs.draw(camera.eye,(camera.target-camera.eye).normalize_or_zero(),&scene.atmosphere,false);
            scene.draw(camera.eye,tick as f32*FIXED_DT,false,true,&transforms);
            art.effects(&arena.stage.interactions,camera.eye,&scene.atmosphere);
            set_default_camera();
            next_frame().await;
        }
        if arena.stage.transition.is_some() {break;}
    }
    ensure!(scene_seen && arena.stage.transition.as_ref().is_some_and(|e|EXIT.matches(e)),
        "Temple paired route did not finish: {:?}",roster.players.each_ref().map(|b|b.player.feet));
    ensure!(roster.players.iter().all(|b|b.player.breath.hits == 0 && b.stats.turtle_air),
        "Temple paired route drowned or lost a personal shell");
    let proof = serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"first":first,"skip":skip,
        "samples":samples,"minimum_air":minimum_air,"frame":roster.frame(1,0),"world":arena.snapshot()});
    std::fs::write(crate::preferences::path(&format!("utemple-{first:?}-{skip}-route.json")),serde_json::to_vec_pretty(&proof)?)?;
    println!("PASS full temple co-op {first:?} skip={skip}: {} ticks, {samples} exact replicas, minimum air {minimum_air:?}, health {:?}",
        roster.tick,roster.players.each_ref().map(|b|b.stats.sanity()));
    Ok(())
}

pub(super) async fn check(assets: &mut Assets) -> Result<()> {
    std::fs::create_dir_all(crate::preferences::path(""))?;
    for first in PlayerId::ALL {
        for skip in [false,true] { route(assets,first,skip).await?; }
    }
    Ok(())
}

/// Offline physics probe from an opt-in test report. No live game is modified.
pub(super) fn replay_report(assets: &mut Assets, path: &std::path::Path) -> Result<()> {
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let visit: Visit = serde_json::from_value(report["visit"].clone())?;
    ensure!(visit.map == "utemple", "Swim replay requires a temple report");
    let map = Bsp::parse(&assets.read("maps/utemple.bsp")?)?;
    let mut world = World::from_bsp(&map)?;
    let mut stage = crate::coop::stage::Stage::load(assets,&map,&mut world,&visit,false)?;
    stage.restore(&serde_json::from_value(report["combat"]["stage"].clone())?,&map,&mut world)?;
    let temple = stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Temple>()).unwrap();
    let index = report["probe_player"].as_u64().unwrap_or(1) as usize;
    ensure!(index < 2, "Invalid temple replay player");
    let player: Player = serde_json::from_value(report["frame"]["players"][index]["player"].clone())?;
    let planned = temple.test_swim_waypoint(&map,&world,&player,
        report["probe_leading"].as_bool().unwrap_or(false))?;
    let goal: Vec3 = if report["probe_goal"].is_null() { planned }
        else { serde_json::from_value(report["probe_goal"].clone())? };
    ensure!(goal.is_finite(), "Invalid temple replay goal");
    let trace = world.body_trace(player.feet,goal);
    println!("Replay {:?} -> {goal:?}, trace fraction={} solid={} normal={:?}, velocity={:?}",
        player.feet,trace.fraction,trace.start_solid,trace.normal,player.velocity);
    if report["probe_forecast"].as_bool().unwrap_or(false) {
        for seconds in [0.,0.05,0.1,0.15,0.2,0.25,0.35,0.5,0.75,1.] {
            let hazards=temple.motion.ahead(&map,&temple.saved,seconds)?;
            let contacts=hazards.iter().filter(|h|h.touches(player.feet+PLAYER_CENTER,
                goal+PLAYER_CENTER,PLAYER_HALF)).count();
            println!("Replay forecast {seconds:.2}s: {contacts} moving hulls cross swim leg");
        }
        // Compare ordinary steering from the same captured velocity against
        // the real advancing machinery. Other bodies receive neutral input;
        // this isolates a turn's inertia from NPC attacks and network timing.
        if report["probe_steering"].as_bool().unwrap_or(false) {
            let frame:crate::coop::world::Frame=serde_json::from_value(report["frame"].clone())?;
            for lead in [0.,0.05,0.1,0.17] {
                let mut world=World::from_bsp(&map)?;
                stage.restore(&serde_json::from_value(report["combat"]["stage"].clone())?,&map,&mut world)?;
                let mut roster=Roster::from_checkpoint(&frame,&world,|_|true)?;
                for _ in 0..120 {
                    let body=&roster.players[index].player;
                    let offset=goal-body.feet-body.velocity*lead;
                    let mut inputs=std::array::from_fn(|_|(Controls::default(),Input::default()));
                    inputs[index].0=Controls {swim:offset.normalize_or_zero()*(offset.length()/30.).min(1.),
                        run:true,..Default::default()};
                    stage.before_move(&mut roster,&map,&mut world,&inputs,FIXED_DT)?;
                    let before=roster.players.each_ref().map(|b|b.player.feet);
                    roster.move_players(&world,inputs);
                    stage.contacts(&mut roster,&world,before)?;
                    roster.finish_tick_with(&world,false,|at|!stage.interactions.hazardous(at))?;
                    if !roster.players[index].stats.alive() { break; }
                }
                let body=&roster.players[index];
                println!("Replay steering lead={lead:.2}: tick={}, health={}, feet={:?}, clear={}",
                    roster.tick,body.stats.sanity(),body.player.feet,world.body_clear(body.player.feet));
            }
        }
        return Ok(());
    }
    let toward = (goal-player.feet).normalize_or_zero();
    for direction in [toward,Vec3::X,Vec3::NEG_X,Vec3::Y,Vec3::NEG_Y,Vec3::Z,Vec3::NEG_Z] {
        let mut probe = player.clone();
        for _ in 0..120 {
            probe.tick(&world,Controls {swim:direction,run:true,..Default::default()});
            ensure!(world.body_clear(probe.feet), "Replay swimmer penetrated a surface");
        }
        println!("Replay swim {direction:?}: displacement {:?}, clear={}, velocity={:?}",
            probe.feet-player.feet,world.body_clear(probe.feet),probe.velocity);
        ensure!(probe.feet.distance(player.feet) > 1., "Replay swimmer remains pinned");
    }
    Ok(())
}
