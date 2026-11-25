//! Alice's swimming shell and low-air warning, derived from saved player state.
use crate::{
    assets::Assets,
    environment::Atmosphere,
    movement::Player,
    particles::Attached,
    skeletal::{Skeleton, Transform},
    texture,
    weapons::Prop,
};
use anyhow::{Context, Result};
use macroquad::prelude::*;
use std::{cell::RefCell, collections::BTreeMap};

pub struct Art {
    shell: Prop,
    bubbles: Attached,
    /// `fx_catchfire`: flames and smoke while Alice stands in lava and briefly after.
    fire: Option<Attached>,
    burn: Option<f32>,
    burn_left: f32,
    events: crate::animation_events::Model,
    warning_duration: f32,
    warning_frame: f32,
    back: usize,
    mouth: usize,
    visible: bool,
    warning: Option<f32>,
    view: RefCell<(Atmosphere, Vec3)>,
    /// `fx_splashsmall`: spawned whenever Alice's feet first touch liquid (native player event 6).
    splash: Option<Attached>,
    splashes: Vec<Splash>,
    /// `splash.tan`: the animated water crown (23 frames, additive `splashmap`).
    splash_mesh: Option<crate::weapons::Prop>,
    /// `None` until the first step after a reset or restore: that step only records the state, so
    /// loading or teleporting into water is not a splash.
    last_level: Option<u8>,
    last_fall: f32,
}
struct Splash {
    fx: Attached,
    at: Vec3,
    scale: f32,
    age: f32,
}
/// Longest-lived splash sprite (the ripple rings live 2.5 s).
const SPLASH_LIFE: f32 = 2.6;
/// The model's own `scale 0.65` times the event scale.
const SPLASH_MODEL_SCALE: f32 = 0.65;

/// Exercise the actual Character draw path, including immediate paused restores.
pub async fn check(a: &mut Assets) -> Result<()> {
    use crate::{
        character::Character,
        movement::Controls,
        water::{Immersion, Liquid},
    };
    use anyhow::ensure;
    let root = std::path::Path::new("private/turtle-shell");
    std::fs::create_dir_all(root)?;
    let mut scene = crate::render::Scene::load(a, "garden1")?;
    // The authored entrance current carries an idle player out of this pool.
    // Isolate breathing here, as --swim-check does; normal gameplay keeps it.
    for current in &mut scene.world.traversal.pushes {
        current.enabled = false;
    }
    let mut player =
        Player::spawn(&scene.world, scene.map.spawn().0).context("Garden swimming spawn")?;
    let stats = crate::inventory::Stats::for_level("garden1", None);
    stats.prepare_player(&mut player);
    for _ in 0..120 * 6 {
        player.tick(&scene.world, Controls::default());
    }
    ensure!(
        player.immersion.level == 3 && player.breath.shell && player.breath.hits == 0,
        "Shell did not protect actual Garden swimmer beyond five seconds: level={}, shell={}, hits={}",
        player.immersion.level, player.breath.shell, player.breath.hits
    );
    let mut alice = Character::load(a)?;
    alice.reset(&player, 0.);
    for _ in 0..120 {
        alice.update(
            1. / 60.,
            &player,
            false,
            crate::weapons::WeaponInput {
                aim: Vec3::X,
                ..Default::default()
            },
            &crate::combat::Context {
                world: &scene.world,
                targets: &[],
            },
        );
    }
    let saved = alice.snapshot();
    let mut restored = Character::load(a)?;
    restored.restore(&serde_json::from_str(&serde_json::to_string(&saved)?)?)?;
    let saved_player: Player = serde_json::from_str(&serde_json::to_string(&player)?)?;
    let target = player.feet + Vec3::Z * 32.;
    let camera = Camera3D {
        position: target + vec3(-115., -80., 35.),
        target,
        up: Vec3::Z,
        fovy: 55_f32.to_radians(),
        z_near: 2.,
        z_far: 30000.,
        ..Default::default()
    };
    let mut pictures = Vec::new();
    for case in [
        "unowned",
        "shell",
        "paused",
        "restored",
        "dry",
        "lava",
        "low-air",
        "low-air-paused",
        "refilled",
    ] {
        player = serde_json::from_str(&serde_json::to_string(&saved_player)?)?;
        player.breath.submerged = 1.;
        if case == "unowned" {
            player.breath.shell = false;
        }
        if case == "dry" {
            player.immersion = Immersion::default();
        }
        if case == "lava" {
            player.immersion.kind = Liquid::Lava;
        }
        if case.starts_with("low-air") {
            player.breath.submerged = 15.12;
        }
        if case == "refilled" {
            player.breath.refill();
        }
        let actor = if case == "restored" {
            &mut restored
        } else {
            &mut alice
        };
        actor.water_appearance(&player);
        clear_background(Color::new(0.12, 0.2, 0.27, 1.));
        set_camera(&camera);
        crate::lighting::select(vec![], camera.position, &scene.world);
        let atmosphere = Atmosphere::default();
        crate::render_fx::begin_view(&camera, 2., &atmosphere, true);
        actor.atmosphere(&atmosphere, camera.position);
        actor.draw(player.feet, true);
        crate::render_fx::finish();
        set_default_camera();
        pictures.push(get_screen_data());
        crate::viewer::save_capture(&root.join(format!("{case}.png")))?;
        next_frame().await;
    }
    let changed = |a: usize, b: usize| {
        pictures[a]
            .bytes
            .chunks_exact(4)
            .zip(pictures[b].bytes.chunks_exact(4))
            .filter(|(x, y)| x != y)
            .count()
    };
    ensure!(changed(0, 1) > 100, "Swimming shell did not render");
    for i in [2, 3, 8] {
        ensure!(
            changed(1, i) == 0,
            "Shell pause/restore/refill differs: {i}"
        );
    }
    for i in [4, 5] {
        ensure!(changed(0, i) == 0, "Shell shown on land or in lava: {i}");
    }
    // Blended GPU channels can round one byte differently between batches;
    // permit that only at a handful of pixels, never moving particle geometry.
    let warning_rounding_only = pictures[6]
        .bytes
        .iter()
        .zip(&pictures[7].bytes)
        .all(|(a, b)| a.abs_diff(*b) <= 1);
    ensure!(
        changed(1, 6) > 20 && changed(6, 7) <= 4 && warning_rounding_only,
        "Low-air bubbles missing or advancing while paused"
    );
    // Also retain an ordinary scene with the normal collision-aware camera and liquid fog.
    player = saved_player;
    alice.water_appearance(&player);
    let (eye, target, show) =
        crate::camera::Follow::default().update(&scene.world, player.feet, Vec3::X, 128., 1. / 60.);
    ensure!(show, "Garden swimming follow camera hides Alice");
    let view = Camera3D {
        position: eye,
        target,
        up: Vec3::Z,
        z_near: 2.,
        z_far: 30000.,
        fovy: 75_f32.to_radians(),
        ..Default::default()
    };
    scene.atmosphere.liquid = Liquid::Water.tint();
    clear_background(BLACK);
    set_camera(&view);
    crate::render_fx::begin_view(&view, 2., &scene.atmosphere, false);
    scene.draw(eye, 2., false, false, &[]);
    alice.atmosphere(&scene.atmosphere, eye);
    alice.draw(player.feet, false);
    crate::render_fx::finish();
    set_default_camera();
    crate::viewer::save_capture(&root.join("garden-swimming.png"))?;
    next_frame().await;
    // Entering the water: dry for a frame, then feet in the liquid, so the native event-6 splash
    // (mist, drops and flat ripple rings) fires once and ages on the gameplay clock.
    let mut splash_player = player.clone();
    splash_player.immersion = Immersion::default();
    splash_player.velocity = Vec3::new(0., 0., -900.);
    let mut splash_alice = Character::load(a)?;
    splash_alice.reset(&splash_player, 0.);
    let tick = |c: &mut Character, p: &Player, dt: f32| {
        c.update(
            dt,
            p,
            false,
            crate::weapons::WeaponInput { aim: Vec3::X, ..Default::default() },
            &crate::combat::Context { world: &scene.world, targets: &[] },
        );
    };
    tick(&mut splash_alice, &splash_player, 1. / 60.);
    splash_player.immersion = Immersion { level: 1, kind: Liquid::Water };
    splash_player.velocity = Vec3::ZERO;
    tick(&mut splash_alice, &splash_player, 1. / 60.);
    let splash_target = splash_player.feet + Vec3::Z * 14.;
    let splash_camera = Camera3D {
        position: splash_target + vec3(-150., -110., 70.),
        target: splash_target,
        up: Vec3::Z,
        fovy: 55_f32.to_radians(),
        z_near: 2.,
        z_far: 30000.,
        ..Default::default()
    };
    let mut elapsed = 0.;
    for (name, until) in [("splash-0.15", 0.15), ("splash-0.5", 0.5), ("splash-1.2", 1.2)] {
        while elapsed < until {
            tick(&mut splash_alice, &splash_player, 1. / 60.);
            elapsed += 1. / 60.;
        }
        clear_background(Color::new(0.12, 0.2, 0.27, 1.));
        set_camera(&splash_camera);
        crate::render_fx::begin_view(&splash_camera, 2., &Atmosphere::default(), true);
        splash_alice.atmosphere(&Atmosphere::default(), splash_camera.position);
        splash_alice.draw(splash_player.feet, true);
        crate::render_fx::finish();
        set_default_camera();
        crate::viewer::save_capture(&root.join(format!("{name}.png")))?;
        next_frame().await;
    }
    println!("PASS swimming shell: {} visible pixels; paused/restore/refill identical; absent without reward/on land/in lava; low-air bubbles visible and paused; actual Garden breathing", changed(0,1));
    Ok(())
}
impl Art {
    pub fn load(
        a: &mut Assets,
        rig: &Skeleton,
        specs: &BTreeMap<String, texture::MaterialSpec>,
    ) -> Result<Self> {
        let (mut def, model) =
            crate::weapons::read_model_clip(a, "mock_shell_alice", Some("shell_alice"))?;
        // Gameplay selects the original second skin; the first is the story's
        // entity-alpha fade and would leave an opaque shell during swimming.
        for skin in def.skins.values_mut() {
            *skin = "turtleshell".into();
        }
        // This timer TAN intentionally has no render geometry or tags.
        let warning = crate::skeletal::Definition::load(a, "models/fx_alicebreath.tik")?;
        let file = warning
            .animations
            .get("bubble_on")
            .context("Missing breath warning clip")?;
        let bytes = a.read(&format!("{}/{file}", warning.path))?;
        let header = crate::skeletal::Bytes(&bytes);
        anyhow::ensure!(
            header.span(0, 4)? == b"TAN " && header.int(4)? == 2,
            "Invalid breath timer"
        );
        let frames = header.count(72, 10000)?;
        let warning_duration = header.float(84)?;
        let warning_frame = warning_duration / frames.max(1) as f32;
        anyhow::ensure!(
            frames > 0 && (0.001..=1.).contains(&warning_frame),
            "Invalid breath timing"
        );
        let tag = |name: &str| {
            rig.bones
                .iter()
                .position(|b| b.name == name)
                .with_context(|| format!("Missing swimming attachment {name}"))
        };
        Ok(Self {
            shell: Prop::build(a, def, model, specs)?,
            // Engine sprite size is texture width times scale (CE-verified), as for the other
            // underwater bubble owners; without it the 200/s breath bubbles are far too big.
            bubbles: Attached::load(a, "fx_alicebreath", specs)?
                .map(Attached::engine_sprite_size)
                .context("Missing breath bubbles")?,
            fire: Attached::load(a, "fx_catchfire", specs).ok().flatten(),
            burn: None,
            burn_left: 0.,
            events: crate::animation_events::Model::load(a, "models/fx_alicebreath.tik")?,
            warning_duration,
            warning_frame,
            back: tag("tag_back")?,
            mouth: tag("tag_mouth")?,
            visible: false,
            warning: None,
            view: RefCell::new((Atmosphere::default(), Vec3::ZERO)),
            splash: Attached::load_clip_bursts(a, "fx_splashsmall", Some("idle"), 0.05, specs)
                .ok()
                .flatten()
                .map(Attached::engine_sprite_size)
                .map(Attached::flat_sprites),
            splashes: Vec::new(),
            splash_mesh: crate::weapons::read_model_clip(a, "fx_splashsmall", Some("idle"))
                .ok()
                .and_then(|(def, model)| crate::weapons::Prop::build(a, def, model, specs).ok()),
            last_level: None,
            last_fall: 0.,
        })
    }
    pub fn reset(&mut self) {
        self.splashes.clear();
        self.last_level = None;
        self.visible = false;
        self.warning = None;
        self.bubbles = self.bubbles.fork();
        self.stop_burning();
    }
    fn stop_burning(&mut self) {
        self.burn = None;
        self.burn_left = 0.;
        if let Some(f) = &mut self.fire {
            f.restart();
        }
    }
    /// Native player event 6 (FUN_100b6ee0 in fgamex86): when the feet reach water the game spawns
    /// `fx_splashsmall` at the surface, scaled `clamp(1 - (vz + 400) / 1500, 1, 1.5)` by how fast
    /// Alice was falling. Lava plays the burn sound instead and spawns nothing.
    pub fn splash_step(&mut self, dt: f32, player: &Player) {
        for s in &mut self.splashes {
            s.age += dt;
        }
        self.splashes.retain(|s| s.age < SPLASH_LIFE);
        let level = player.immersion.level.min(3) as u8;
        if self.last_level == Some(0) && level > 0 && player.immersion.kind != crate::water::Liquid::Lava {
            if let Some(template) = &self.splash {
                let f = 1. - (self.last_fall + 400.) / 1500.;
                let event_scale = if f >= 1. { f.min(1.5) } else { 1. };
                if self.splashes.len() >= 6 {
                    self.splashes.remove(0);
                }
                self.splashes.push(Splash {
                    fx: template.fork(),
                    // Level 1 means the surface lies between 1 and 28 units above the feet.
                    at: player.feet + Vec3::Z * 14.,
                    scale: SPLASH_MODEL_SCALE * event_scale,
                    age: 0.,
                });
            }
        }
        self.last_level = Some(level);
        self.last_fall = player.velocity.z;
    }
    pub fn draw_splashes(&mut self) {
        let (atmosphere, camera) = &*self.view.borrow();
        for s in &mut self.splashes {
            let origin = Transform {
                translation: s.at,
                rotation: Quat::IDENTITY,
            };
            s.fx.draw(s.age, s.scale, |_, _| origin, |_, _, default| default, *camera, atmosphere);
            // The model's server script holds full alpha for 11 frames (0.55 s), fades it out over
            // 0.5 s, then removes the entity at the end of the 1.15 s clip.
            if let Some(mesh) = &mut self.splash_mesh {
                let alpha = (1. - (s.age - 0.55) / 0.5).clamp(0., 1.);
                if alpha > 0. && s.age < 1.15 {
                    for m in mesh.meshes_at(origin, s.scale, true, s.age, false) {
                        for v in &mut m.vertices {
                            v.color[3] = (alpha * 255.) as u8;
                        }
                        crate::render_fx::skin(m);
                    }
                }
            }
        }
    }
    pub fn burning(&self) -> bool {
        self.burn.is_some()
    }
    /// Advance the lava fire on the gameplay clock (never while paused).
    pub fn burn_step(&mut self, dt: f32, player: &Player) {
        if player.immersion.level >= 1 && player.immersion.kind == crate::water::Liquid::Lava {
            self.burn_left = 3.;
            self.burn.get_or_insert(0.);
        } else {
            self.burn_left -= dt;
            if self.burn_left <= 0. {
                if self.burn.is_some() {
                    self.stop_burning();
                }
                return;
            }
        }
        if let Some(age) = &mut self.burn {
            *age += dt;
        }
    }
    /// Flames follow Alice's origin, alive or dying, like the original originemitter.
    pub fn draw_fire(&mut self, feet: Vec3, scale: f32) {
        let (Some(age), Some(fire)) = (self.burn, &mut self.fire) else {
            return;
        };
        let origin = Transform {
            translation: feet + Vec3::Z * 24. * scale,
            rotation: Quat::IDENTITY,
        };
        let (atmosphere, camera) = &*self.view.borrow();
        fire.draw(age, scale, |_, _| origin, |_, _, default| default, *camera, atmosphere);
    }
    pub fn sync(&mut self, player: &Player) {
        self.visible = player.breath.shell
            && player.immersion.level >= 2
            && player.immersion.kind != crate::water::Liquid::Lava;
        let warning = (player.immersion.level == 3)
            .then(|| player.breath.warning_age())
            .flatten();
        if self.warning.is_some() && warning.is_none() {
            self.bubbles = self.bubbles.fork();
        }
        self.warning = warning;
    }
    pub fn atmosphere(&self, atmosphere: &Atmosphere, camera: Vec3) {
        *self.view.borrow_mut() = (atmosphere.clone(), camera);
    }
    pub fn draw(
        &mut self,
        pose: &[Transform],
        body: Transform,
        scale: f32,
        time: f32,
        fullbright: bool,
    ) {
        let tag = |index: usize| Transform {
            translation: body.point(pose[index].translation * scale),
            rotation: body.rotation * pose[index].rotation,
        };
        if self.visible {
            self.shell
                .draw_frame(tag(self.back), scale, fullbright, time, true);
        }
        if let Some(age) = self.warning {
            let mouth = tag(self.mouth);
            let (atmosphere, camera) = &*self.view.borrow();
            self.bubbles.draw(
                age,
                scale,
                |_, _| mouth,
                |name, birth, default| {
                    self.events
                        .visual(
                            "bubble_on",
                            birth,
                            self.warning_duration,
                            self.warning_frame,
                            true,
                        )
                        .emitters
                        .get(name)
                        .copied()
                        .unwrap_or(default)
                },
                *camera,
                atmosphere,
            );
        }
    }
}
