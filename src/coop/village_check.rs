//! Accelerated native two-body traversal of the opening chapter. Both bodies
//! use the production arena and ordinary input; this is not a network test.
use super::{arena::Arena, protocol::{Input, Visit}, world::Roster, PlayerId};
use crate::{assets::Assets, movement::{Controls, FIXED_DT}, render::Scene, route::Route};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;
use std::collections::{BTreeMap, VecDeque};

struct Replay {
    scene: Scene,
    arena: Arena,
    roster: Roster,
    art: crate::viewer::LevelArt,
    leader: PlayerId,
    owners: BTreeMap<String, PlayerId>,
    reunions: usize,
    moving_speech: usize,
}
impl Replay {
    fn new(assets: &mut Assets, leader: PlayerId) -> Result<Self> {
        let visit = Visit { epoch: 1, map: "gvillage".into(), entry: None, difficulty: Default::default(), combat_preview: false };
        let mut scene = Scene::load_for(assets, &visit.map, visit.difficulty)?;
        let mut arena = Arena::load(assets, &scene.map, &mut scene.world, &visit, false)?;
        let (eye, yaw) = crate::interaction::spawn(&scene.map, None);
        let roster = Roster::new_with(&scene.world, eye, yaw, crate::inventory::Stats::for_level("gvillage", None),
            |at| !arena.stage.interactions.hazardous(at))?;
        arena.reset_characters(&roster.players);
        let mut art = crate::viewer::LevelArt::default();
        art.prepare(assets, &scene, &arena.stage.interactions, false)?;
        Ok(Self { scene, arena, roster, art, leader, owners: BTreeMap::new(), reunions: 0, moving_speech: 0 })
    }
    async fn step(&mut self, movement: [Controls; 2]) -> Result<()> {
        let was_scene = self.arena.stage.owner;
        let speech = self.arena.stage.story.busy() && !self.arena.stage.busy();
        let before = self.roster.players.each_ref().map(|b| (b.player.feet, b.warped_at));
        let mut inputs = std::array::from_fn(|i| (movement[i], Input {
            yaw: self.roster.players[i].yaw, pitch: self.roster.players[i].pitch, ..Default::default()
        }));
        if let Some(owner) = was_scene {
            // Advance original dialogue through its ordinary key edge; do not skip scenes.
            inputs[owner.index()].0.use_pressed = self.roster.tick % 30 == 0;
        } else {
            if speech { inputs[self.leader.index()].0.use_pressed = self.roster.tick % 30 == 0; }
            let mut targets = self.arena.npcs.targets();
            if let Some(cast) = &self.arena.stage.interactions.encounters { targets.extend(cast.targets()); }
            targets.extend(self.arena.stage.interactions.shot_targets());
            let context = crate::combat::Context { world: &self.scene.world, targets: &targets };
            for (i, body) in self.roster.players.iter().enumerate() {
                let nearest = targets.iter().filter(|t| t.center.distance(body.player.eye()) < 1000.)
                    .filter(|t| crate::combat::contact(&context, body.player.eye(), t.center, 1.)
                        .is_some_and(|(id, _)| id == t.id))
                    .min_by(|a,b| a.center.distance_squared(body.player.eye()).total_cmp(&b.center.distance_squared(body.player.eye())));
                if let Some(target) = nearest.filter(|_| body.stats.copies(0) > 0) {
                    let delta = target.center - body.player.eye();
                    inputs[i].1.yaw = delta.y.atan2(delta.x);
                    inputs[i].1.pitch = (delta.z / delta.length().max(1.)).asin().clamp(-1.56,1.56);
                    inputs[i].1.weapon = Some(0);
                    inputs[i].1.primary = delta.length() < 80.;
                    inputs[i].1.alternate = !inputs[i].1.primary;
                } else if movement[i].wish.length() > 0. {
                    inputs[i].1.yaw = movement[i].wish.y.atan2(movement[i].wish.x);
                    inputs[i].1.pitch = 0.;
                }
            }
        }
        self.arena.step(&mut self.roster, &self.scene.map, &mut self.scene.world, inputs)?;
        if speech && !self.arena.stage.busy() {
            ensure!(self.roster.players.iter().zip(before).all(|(b, (_, warp))| b.warped_at == warp),
                "Ordinary speech teleported a player");
            if self.roster.players.iter().zip(before).any(|(b, (feet, _))| b.player.feet.distance(feet) > 0.1) {
                self.moving_speech += 1;
            }
        }
        if let Some(owner) = self.arena.stage.owner {
            if let Some(id) = self.arena.stage.interactions.village.as_ref().unwrap().cinema.scene_id() {
                if let Some(old) = self.owners.insert(id.into(), owner) { ensure!(old == owner, "Scene owner changed mid-scene"); }
            }
        }
        if was_scene.is_some() && self.arena.stage.owner.is_none() && self.arena.stage.transition.is_none() {
            self.reunions += 1;
            ensure!(self.roster.players[0].player.feet.distance(self.roster.players[1].player.feet) <= 110., "Scene did not reunite players");
            ensure!(self.roster.players.iter().all(|b| !b.player.grounded || b.player.velocity == Vec3::ZERO),
                "Scene released grounded Alice with stale movement momentum");
            self.draw();
            crate::viewer::save_capture(&crate::preferences::path("coop-village")
                .join(format!("{:?}-reunion-{}.png", self.leader, self.reunions)))?;
        }
        ensure!(self.roster.players.iter().all(|b| b.stats.alive()), "Route player died at tick {}: {:?}",
            self.roster.tick, self.roster.players.each_ref().map(|b| b.player.feet));
        ensure!(self.roster.checkpoint_generation == 0, "Route unexpectedly rolled back to a checkpoint");
        if !self.arena.stage.busy() {
            ensure!(self.roster.players.iter().all(|b| self.scene.world.body_clear(b.player.feet)), "Route body is obstructed");
        }
        if self.roster.tick % 120 == 0 {
            self.draw();
            next_frame().await;
        }
        Ok(())
    }
    fn draw(&mut self) {
        clear_background(BLACK);
        let feet = self.roster.players[self.leader.index()].player.feet;
        let camera = self.arena.stage.camera(&self.scene.world).unwrap_or_else(|| {
            let (eye, target, _) = crate::camera::Follow::default().update(&self.scene.world,
                feet, vec3(-1., 1., -0.6).normalize(), 220., FIXED_DT);
            crate::cinematic::Camera::look(eye, target)
        });
        set_camera(&Camera3D { position: camera.eye, target: camera.target, up: camera.up,
            fovy: 60_f32.to_radians(), z_near: 2., z_far: 30000., ..Default::default() });
        let transforms = self.arena.stage.interactions.transforms();
        self.scene.draw(camera.eye, self.roster.tick as f32 * FIXED_DT, false, false, &transforms);
        self.art.story_pose(&self.arena.stage.story);
        self.art.draw(&self.arena.stage.interactions, &self.arena.stage.hints, &self.scene.atmosphere, camera.eye, false);
        if !self.arena.stage.busy() {
            for (character, body) in self.arena.characters.iter_mut().zip(&self.roster.players) {
                character.atmosphere(&self.scene.atmosphere, camera.eye);
                character.draw_replica(body.player.feet, body.player.feet, false);
            }
        }
        self.arena.npcs.draw(camera.eye, (camera.target - camera.eye).normalize_or_zero(), &self.scene.atmosphere, false);
        self.scene.draw(camera.eye, self.roster.tick as f32 * FIXED_DT, false, true, &transforms);
        self.art.effects(&self.arena.stage.interactions, camera.eye, &self.scene.atmosphere);
        set_default_camera();
        draw_text(&format!("Co-op village native route / {:?} first / tick {}", self.leader, self.roster.tick), 20., 30., 24., WHITE);
    }
    async fn wait(&mut self, seconds: f32) -> Result<()> {
        for _ in 0..(seconds / FIXED_DT) as usize {
            if self.arena.stage.transition.is_some() { break; }
            self.step(Default::default()).await?;
        }
        Ok(())
    }
    async fn go(&mut self, goal: Vec3, navigate: bool) -> Result<()> {
        let mut plans: [VecDeque<(Controls, Vec3)>; 2] = Default::default();
        let mut ages = [0; 2];
        let mut leader_reached = None;
        for _ in 0..18000 {
            if self.arena.stage.transition.is_some() { return Ok(()); }
            if self.arena.stage.busy() {
                plans.iter_mut().for_each(VecDeque::clear);
                ages = [0; 2];
                leader_reached = None;
                self.step(Default::default()).await?;
                continue;
            }
            let reached = self.roster.players.each_ref().map(|b| {
                let delta = goal - b.player.feet;
                if navigate { delta.length() < 42. && b.player.grounded }
                else { delta.truncate().length() < 8. && delta.z.abs() < 35. }
            });
            if reached.into_iter().all(|r| r) {
                println!("PASS co-op village leg {goal:?}: {:?}", self.roster.players.each_ref().map(|b| b.player.feet));
                return Ok(());
            }
            if reached[self.leader.index()] { leader_reached.get_or_insert(self.roster.tick); }
            let mut moves = [Controls::default(); 2];
            let mut planned = [false; 2];
            for id in PlayerId::ALL {
                let i = id.index();
                if reached[i] || id != self.leader && leader_reached.is_none() { continue; }
                if navigate {
                    if plans[i].is_empty() {
                        plans[i] = Route::plan(&self.scene.world, &self.roster.players[i].player, goal, 22.)
                            .or_else(|_| Route::plan(&self.scene.world, &self.roster.players[i].player, goal, 40.))
                            .map_err(|states| anyhow::anyhow!("No co-op route for {id:?} from {:?} to {goal:?} ({states} states)", self.roster.players[i].player.feet))?.into();
                        ages[i] = 0;
                    }
                    if let Some((input, _)) = plans[i].front() {
                        moves[i] = Controls { jump: input.jump && ages[i] == 0, ..*input };
                        planned[i] = true;
                    }
                } else { moves[i].wish = (goal - self.roster.players[i].player.feet).truncate().normalize_or_zero(); }
            }
            self.step(moves).await?;
            for i in 0..2 {
                if planned[i] {
                    ages[i] += 1;
                    if ages[i] == 24 {
                        if let Some((_, expected)) = plans[i].pop_front() {
                            if expected.distance(self.roster.players[i].player.feet) > 8. { plans[i].clear(); }
                        }
                        ages[i] = 0;
                    }
                }
            }
        }
        anyhow::bail!("Co-op route stalled toward {goal:?} at {:?}", self.roster.players.each_ref().map(|b| b.player.feet))
    }
    async fn route(&mut self) -> Result<()> {
        ensure!(self.roster.players.iter().all(|b| b.stats.equipped().is_none()), "Village must start unarmed");
        self.wait(6.).await?;
        for goal in [vec3(-4180.,5900.,16.),vec3(-4192.,5416.,-52.),vec3(-4412.,5328.,-52.),vec3(-4780.,5392.,-120.),vec3(-4968.,5424.,-120.)] { self.go(goal,false).await?; }
        self.wait(6.).await?;
        self.go(vec3(-5100.,5200.,-120.),true).await?;
        self.go(vec3(-5100.,4780.,-56.),true).await?;
        self.wait(6.).await?;
        for goal in [vec3(-4784.,4784.,-120.),vec3(-4512.,4800.,-96.),vec3(-4288.,4808.,-136.),vec3(-3984.,4712.,-176.),vec3(-3972.,4332.,-160.),vec3(-3960.,4056.,-224.),vec3(-3968.,3840.,-272.),vec3(-3776.,3832.,-208.),vec3(-3544.,3856.,-136.)] { self.go(goal,false).await?; }
        ensure!(self.roster.players.iter().all(|b| b.stats.copies(0) == 1 && self.arena.pickups.iter().any(|p|
            matches!(p.kind, crate::inventory::PickupKind::Weapon(0)) && b.stats.collected.contains(&p.id))), "Both players must collect the real hallway Blade");
        self.wait(15.).await?;
        for goal in [vec3(-3208.,4040.,-56.),vec3(-2800.,3872.,-96.),vec3(-2808.,3520.,-96.),vec3(-2744.,3304.,-96.),vec3(-2936.,3024.,128.)] { self.go(goal,true).await?; }
        self.wait(8.).await?;
        self.go(vec3(-3112.,2584.,40.),true).await?;
        self.go(vec3(-3312.,2440.,48.),false).await?;
        self.wait(8.).await?;
        ensure!(self.arena.stage.transition == Some(("pandemonium".into(),Some("player_start".into()))), "The authored village exit was not reached");
        ensure!(self.arena.stage.interactions.village.as_ref().unwrap().cinema.state.done.len() == 6, "Not all six village scenes completed");
        ensure!(self.reunions == 6 && self.moving_speech > 0,
            "Voice-only lines must permit movement without an extra scene reunion");
        ensure!(self.owners.len() == 6 && self.owners.iter().all(|(name, owner)|
            *owner == if name == "entry" { PlayerId::Blue } else { self.leader }),
            "The first arriving player did not own every triggered village scene: {:?}", self.owners);
        Ok(())
    }
}

pub async fn check(assets: &mut Assets) -> Result<()> {
    let folder = crate::preferences::path("coop-village");
    std::fs::create_dir_all(&folder)?;
    for leader in PlayerId::ALL {
        let mut replay = Replay::new(assets, leader)?;
        let result = replay.route().await;
        replay.draw();
        crate::viewer::save_capture(&folder.join(format!("{leader:?}.png")))?;
        std::fs::write(folder.join(format!("{leader:?}.json")), serde_json::to_vec(&serde_json::json!({
            "passed": result.is_ok(), "error": result.as_ref().err().map(|e| format!("{e:#}")), "leader": leader,
            "owners": replay.owners, "reunions": replay.reunions, "moving_speech_ticks": replay.moving_speech,
            "frame": replay.roster.frame(1,0),
            "arena": replay.arena.snapshot(), "build": env!("LOOKING_GLASS_BUILD"),
            "scope": "native two-body production simulation; not a separate-process network test"
        }))?)?;
        result.with_context(|| format!("Village led by {leader:?}"))?;
        println!("PASS native co-op village {leader:?}: unarmed entrance to authored exit, both Blade pickups, six scenes, {} reunions, {} ticks", replay.reunions, replay.roster.tick);
    }
    Ok(())
}
