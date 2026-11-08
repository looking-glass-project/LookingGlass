//! Staged adapter checks using the real host arena and authored geometry.
//! These are not a complete route or a separate-client/network playtest.
use super::{arena::Arena, protocol::{Input, Visit}, world::Roster, PlayerId};
use crate::{assets::Assets, movement::{Controls, Player, EYE_HEIGHT, FIXED_DT}, render::Scene};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;

pub(super) struct Check {
    pub scene: Scene,
    pub arena: Arena,
    pub roster: Roster,
    pub leader: Option<PlayerId>,
    art: crate::viewer::LevelArt,
}
impl Check {
    pub fn new(assets: &mut Assets) -> Result<Self> {
        println!("Pandemonium co-op check / build {}", env!("LOOKING_GLASS_BUILD"));
        let visit = Visit { epoch: 1, map: "pandemonium".into(), entry: None,
            difficulty: Default::default(), combat_preview: false };
        let mut scene = Scene::load_for(assets, &visit.map, visit.difficulty)?;
        let mut arena = Arena::load(assets, &scene.map, &mut scene.world, &visit, false)?;
        let (eye, yaw) = crate::interaction::spawn(&scene.map, None);
        let roster = Roster::new_with(&scene.world, eye, yaw,
            crate::inventory::Stats::for_level(&visit.map, None),
            |at| !arena.stage.interactions.hazardous(at))?;
        arena.reset_characters(&roster.players);
        let mut art = crate::viewer::LevelArt::default();
        art.prepare(assets, &scene, &arena.stage.interactions, false)?;
        Ok(Self { scene, arena, roster, art, leader: None })
    }
    pub async fn step(&mut self, movement: [Controls; 2]) -> Result<()> {
        let mut inputs = std::array::from_fn(|i| (movement[i], Input {
            yaw: self.roster.players[i].yaw, ..Default::default()
        }));
        if let Some(leader) = self.leader {
            if self.arena.stage.story.busy() {
                let speaker = self.arena.stage.owner.unwrap_or(leader).index();
                inputs[speaker].0.use_pressed = self.roster.tick % 30 == 0;
            }
            if !self.arena.stage.busy() {
                let mut targets = self.arena.npcs.targets();
                if let Some(cast) = &self.arena.stage.interactions.encounters { targets.extend(cast.targets()); }
                targets.extend(self.arena.stage.interactions.shot_targets());
                let context = crate::combat::Context { world: &self.scene.world, targets: &targets };
                for (i, body) in self.roster.players.iter().enumerate() {
                    let aim = movement[i].wish.extend(0.);
                    if self.arena.stage.interactions.prompt(&self.scene.world, body.player.eye(), aim) == Some("E  open door") {
                        inputs[i].0.use_pressed = true;
                    }
                    if let Some(target) = targets.iter().filter(|t| t.center.distance(body.player.eye()) < 1000.)
                        .filter(|t| crate::combat::contact(&context, body.player.eye(), t.center, 1.).is_some_and(|(id, _)| id == t.id))
                        .min_by(|a,b| a.center.distance_squared(body.player.eye()).total_cmp(&b.center.distance_squared(body.player.eye())))
                        .filter(|_| body.stats.copies(0) > 0) {
                        let delta = target.center - body.player.eye();
                        inputs[i].1.yaw = delta.y.atan2(delta.x);
                        inputs[i].1.pitch = (delta.z / delta.length().max(1.)).asin().clamp(-1.56, 1.56);
                        inputs[i].1.weapon = Some(0);
                        inputs[i].1.primary = delta.length() < 80.;
                        inputs[i].1.alternate = !inputs[i].1.primary;
                    } else if aim.length() > 0. { inputs[i].1.yaw = aim.y.atan2(aim.x); }
                }
            }
        }
        self.arena.step(&mut self.roster, &self.scene.map, &mut self.scene.world, inputs)?;
        // Accelerated routes still advance the rope at every simulation tick;
        // occasional captures must not leave its free tail seconds behind.
        let transforms = self.arena.stage.draw_transforms(&self.scene.world);
        let hands = self.arena.stage.rope_hands(&self.scene.world, &self.roster.players, &self.arena.characters);
        self.scene.update_ropes_many(FIXED_DT, &hands, &transforms);
        if self.roster.checkpoint_generation != 0 || self.roster.players.iter().any(|b| !b.stats.alive()) {
            self.capture(&format!("failure-{}", self.roster.tick), self.leader.unwrap_or(PlayerId::Blue))?;
            std::fs::write(crate::preferences::path("coop-pandemonium").join(format!("failure-{}.json",self.roster.tick)),
                serde_json::to_vec_pretty(&serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"frame":self.roster.frame(1,0),"world":self.arena.snapshot()}))?)?;
            anyhow::bail!("Unexpected death/wipe at tick {}: {:?}", self.roster.tick,
                self.roster.players.each_ref().map(|b| (b.player.feet, b.stats.sanity())));
        }
        if self.roster.tick % 120 == 0 { self.draw(self.leader.unwrap_or(PlayerId::Blue)); next_frame().await; }
        Ok(())
    }
    pub fn draw(&mut self, watched: PlayerId) {
        clear_background(BLACK);
        let pair_on_rope = self.roster.players.iter().all(|b| b.player.rope.is_some());
        let feet = if pair_on_rope { self.roster.players.iter().map(|b| b.player.feet).sum::<Vec3>() * 0.5 }
            else { self.roster.players[watched.index()].player.feet };
        let camera = self.arena.stage.camera(&self.scene.world).unwrap_or_else(|| {
            let (eye, target, _) = crate::camera::Follow::default().update(&self.scene.world,
                feet, vec3(-1., 1., -0.35).normalize(), 220., FIXED_DT);
            crate::cinematic::Camera::look(eye, target)
        });
        let view = Camera3D { position: camera.eye, target: camera.target, up: camera.up,
            fovy: 60_f32.to_radians(), z_near: 2., z_far: 30000., ..Default::default() };
        set_camera(&view);
        crate::lighting::select(vec![], camera.eye, &self.scene.world);
        crate::render_fx::begin_view(&view, self.roster.tick as f32 * FIXED_DT, &self.scene.atmosphere, false);
        self.arena.stage.interactions.presentation.apply(&mut self.scene);
        let transforms = self.arena.stage.draw_transforms(&self.scene.world);
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
        draw_text(&format!("Pandemonium co-op check / {watched:?} / tick {}", self.roster.tick), 20., 30., 24., WHITE);
    }
    pub fn capture(&mut self, label: &str, watched: PlayerId) -> Result<()> {
        self.draw(watched);
        crate::viewer::save_capture(&crate::preferences::path("coop-pandemonium").join(format!("{label}.png")))
    }
    fn place(&mut self, id: PlayerId, feet: Vec3) -> Result<()> {
        let player = Player::spawn(&self.scene.world, feet + Vec3::Z * EYE_HEIGHT)
            .context("Staged adapter point is obstructed")?;
        self.roster.players[id.index()].player = player;
        self.arena.stage.respawned(id);
        self.arena.reset_characters(&self.roster.players);
        Ok(())
    }
    async fn rope(&mut self, first: PlayerId) -> Result<()> {
        ensure!(!self.scene.world.traversal.managed_ropes, "Single-player rope still owns both players");
        let world = &self.scene.world;
        let mut found = None;
        'find: for rope in &world.traversal.ropes {
            for drop in [160., 240., 320., 80.] {
                let feet = rope.anchor - Vec3::Z * (drop + 40.) + Vec3::X * 24.;
                if drop + 112. > rope.length || !world.body_clear(feet)
                    || !world.body_clear(feet - Vec3::Z * 56. + Vec3::X * 24.) { continue; }
                let mut probe = Player::new(feet);
                probe.tick(world, Controls { use_pressed: true, ..Default::default() });
                if probe.rope.as_ref().is_some_and(|g| g.id == rope.id) {
                    found = Some(feet); break 'find;
                }
            }
        }
        let feet = found.context("No accessible authored Pandemonium rope")?;
        let partner = first.partner();
        self.roster.players[first.index()].player = Player::new(feet);
        // Start outside the automatic-grab reach so the partner's later E edge
        // is a grab, not a release of a grip acquired on the first frame.
        self.roster.players[partner.index()].player = Player::new(feet - Vec3::Z * 56. + Vec3::X * 24.);
        ensure!(world.body_clear(self.roster.players[partner.index()].player.feet), "Second rope staging point obstructed");
        let input = |id: PlayerId, control: Controls| std::array::from_fn(|i| if i == id.index() { control } else { Controls::default() });
        self.step(input(first, Controls { use_pressed: true, ..Default::default() })).await?;
        ensure!(self.roster.players[first.index()].player.rope.is_some(), "First player cannot grab the rope");
        self.step(input(partner, Controls { use_pressed: true, ..Default::default() })).await?;
        ensure!(self.roster.players.iter().all(|b| b.player.rope.is_some()), "Second grab displaced the first grip");
        let before = self.roster.players.each_ref().map(|b| b.player.feet);
        for _ in 0..60 {
            self.step(std::array::from_fn(|i| Controls { rise: if i == first.index() { 1. } else { -1. }, ..Default::default() })).await?;
            for body in &self.roster.players { body.player.validate_world(&self.scene.world)?; }
        }
        ensure!(self.roster.players[first.index()].player.feet.z > before[first.index()].z + 10.
            && self.roster.players[partner.index()].player.feet.z < before[partner.index()].z - 10.,
            "Climbing one rope changed the partner's controls");
        self.capture(&format!("{first:?}-two-grips"), first)?;
        let frame = self.roster.frame(1, 0);
        let decoded: super::world::Frame = serde_json::from_slice(&super::codec::decode(
            &super::codec::encode(&serde_json::to_vec(&frame)?)?)?)?;
        decoded.validate()?;
        for body in &decoded.players { body.player.validate_world(&self.scene.world)?; }
        ensure!(serde_json::to_value(&decoded)? == serde_json::to_value(&frame)?, "Two grips did not survive the wire envelope");
        self.step(input(first, Controls { use_pressed: true, ..Default::default() })).await?;
        ensure!(self.roster.players[first.index()].player.rope.is_none()
            && self.roster.players[partner.index()].player.rope.is_some(), "One release detached the partner");
        println!("PASS Pandemonium {first:?} first: independent grabs, opposing climbs, wire roundtrip and personal release");
        Ok(())
    }
    async fn cart(&mut self, first: PlayerId) -> Result<()> {
        use crate::pandemonium::Cart;
        let partner = first.partner();
        self.place(first, vec3(-5024., 1368., 32.))?;
        self.place(partner, vec3(-5024., 1600., -25.))?;
        for _ in 0..240 {
            self.step([Controls::default(); 2]).await?;
            ensure!(self.arena.stage.interactions.pandemonium.as_ref().unwrap().state.cart == Cart::Waiting
                && self.arena.stage.owner.is_none(), "Minecart departed without its partner");
        }
        ensure!(self.arena.stage.notices[first.index()].as_ref().is_some_and(|n| n.text.contains("board")),
            "Missing shared boarding notice");
        let state = self.arena.snapshot();
        self.arena.replicate(&serde_json::from_slice(&serde_json::to_vec(&state)?)?, &self.scene.map, &mut self.scene.world)?;
        self.place(partner, vec3(-5000., 1368., 32.))?;
        self.step([Controls::default(); 2]).await?;
        ensure!(self.arena.stage.owner == Some(first), "Minecart lost its first waiting requester");
        let waiting = self.roster.players[partner.index()].player.feet;
        for _ in 0..14_400 {
            let mut movement = [Controls::default(); 2];
            movement[first.index()].use_pressed = self.roster.tick % 30 == 0;
            self.step(movement).await?;
            if self.arena.stage.owner.is_none() { break; }
            ensure!(self.arena.stage.owner == Some(first)
                && self.roster.players[partner.index()].player.feet == waiting,
                "Minecart changed its scene owner or moved the waiting partner");
        }
        let p = self.arena.stage.interactions.pandemonium.as_ref().unwrap();
        ensure!(p.state.cart == Cart::Finished && p.state.rides == 1 && self.arena.stage.owner.is_none(),
            "Minecart did not finish exactly once");
        ensure!(self.roster.players[0].player.feet.distance(self.roster.players[1].player.feet) <= 110.
            && self.roster.players[0].warped_at == self.roster.players[1].warped_at
            && self.roster.players[0].warped_at > 0, "Minecart did not safely reunite the pair");
        ensure!(self.roster.players.iter().all(|b| self.scene.world.body_clear(b.player.feet)), "Blocked minecart reunion");
        self.capture(&format!("{first:?}-cart-reunion"), first)?;
        let key = self.scene.map.entities.iter().find(|e| e.get("targetname").is_some_and(|n| n == "gnome_key"))
            .and_then(|e| e.get("origin")).and_then(|s| crate::interaction::vector(s)).context("Missing key marker")?;
        self.place(partner, key - Vec3::Z * EYE_HEIGHT)?;
        self.step([Controls::default(); 2]).await?;
        ensure!(self.arena.stage.interactions.pandemonium.as_ref().unwrap().state.key, "Partner cannot collect shared quest key");
        println!("PASS Pandemonium {first:?} first: wait for both, retained requester, one authored cart ride, safe reunion and partner key pickup");
        Ok(())
    }
}

pub async fn check(assets: &mut Assets) -> Result<()> {
    for first in PlayerId::ALL {
        Check::new(assets)?.rope(first).await?;
        Check::new(assets)?.cart(first).await?;
    }
    println!("PASS staged Pandemonium co-op adapters / build {} / full traversal and public clients remain unverified", env!("LOOKING_GLASS_BUILD"));
    Ok(())
}
