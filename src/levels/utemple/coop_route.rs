//! Full entrance-to-exit swimming with ordinary inputs in the production Arena.
//! No authored events, placements, resources or hazard states are written.
use super::*;
use crate::{coop::{arena::Arena, protocol::{Input, Visit}, world::Roster, PlayerId},
    movement::{Controls, FIXED_DT}, render::Scene};

fn owner(arena: &Arena) -> &Temple {
    arena.stage.interactions.levels.iter().find_map(|s| s.ctl.downcast_ref::<Temple>()).unwrap()
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
            let mut hazards = temple.motion.ahead(&scene.map,&temple.saved,1.)?;
            hazards.extend(temple.motion.ahead(&scene.map,&temple.saved,0.5)?);
            hazards.extend(scene.world.traversal.pushes.iter().filter(|p|p.enabled).map(|p|p.volume.clone()));
            for id in PlayerId::ALL {
                let body = &roster.players[id.index()].player;
                let target = if temple.saved.guide.is_none() {
                    if id == first { vec3(-88.,-5980.,1330.) } else { body.feet }
                } else if temple.saved.open && id == first {
                    vec3(-2144.,3352.,488.)
                } else {
                    // Follow the same air trail a second apart. Each swimmer
                    // still navigates independently around current/future traps.
                    let age = temple.saved.clock - if id == first {0.} else {1.};
                    temple.data.shell_pose(temple.turtle_pose(age),age.max(0.) as f32).translation-PLAYER_CENTER
                };
                goals[id.index()] = super::swim_check::waypoint(&scene.world,body.feet,target,&hazards);
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
