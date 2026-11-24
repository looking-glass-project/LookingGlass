//! Hit blood: Sentient damage spawns the actor's tik `bloodmodel` (blood_spray,
//! blood_spray_green or blood_spray_sparks) at the hit as a short-lived effect
//! (FAKK2 Sentient::ArmorDamage posts EV_Remove 1 s after spawning it).
use crate::{
    assets::Assets, collision::World, particles::Attached, skeletal::Transform, texture,
};
use macroquad::prelude::*;
use std::collections::BTreeMap;

/// CE: the native spray entity (scale 1.0 for Alice and for enemies) is removed 0.5 s after the hit.
const EMIT: f32 = 0.5;
const LINGER: f32 = 0.8;

#[derive(Default)]
pub struct Blood {
    /// Actor model -> its bloodmodel template (None: the tik bleeds nothing).
    models: BTreeMap<String, Option<String>>,
    templates: BTreeMap<String, Option<Attached>>,
    live: Vec<(Attached, Vec3, f32, f32)>,
    /// Floor decals (original blood1..6_decal, o_ and g_ variants by bloodmodel).
    textures: BTreeMap<String, Option<Texture2D>>,
    decals: Vec<Decal>,
    seed: u32,
    /// Seconds of play, and when the last hit blood spawned (native: one burst per 0.2 s).
    clock: f32,
    last: Option<f32>,
}
/// Native `FUN_100d92c0` (Sentient damage) never bleeds for fire, eyebeam, lava, gas, slime,
/// drowning, falling, lifedrain, radiation or sling damage. Everything else does, ice wand included.
pub fn bleeds(kind: crate::combat::DamageKind) -> bool {
    !matches!(kind, crate::combat::DamageKind::EyeStaff | crate::combat::DamageKind::Fire)
}
/// The actor model a level controller's enemy uses, by hit id offset from its registered base.
pub fn level_model(controller: &str, local: usize) -> Option<&'static str> {
    Some(match controller {
        "centipede1" | "centipede2" | "garden2" | "potears2" => "c_armyant",
        "grounds2" | "keep" => "cardguard_diamond",
        "hatter2" if local < 100 => "c_madhatter",
        "hatter2" => "c_clockwork",
        "rchess1" if local < 100 => "c_chess_red_king",
        "rchess1" => "c_chess_red_knight",
        "wchess1" | "wchess2" => "c_chess_knight",
        "funhouse" => "c_tweedle_dee",
        _ => return None,
    })
}
struct Decal {
    center: Vec3,
    normal: Vec3,
    radius: f32,
    spin: f32,
    life: f32,
    age: f32,
    texture: String,
}
const MAX_DECALS: usize = 48;
/// Native decal polys (CE hook on the engine's poly add, game-clock): vertex RGBA 255,255,255,255
/// for 9 s, then alpha falls linearly to 0 over 1 s (removed ~10 s after spawn).
const FADE: f32 = 1.;
fn decal_alpha(life: f32, age: f32) -> f32 {
    ((life - age) / FADE).clamp(0., 1.)
}
#[cfg(test)]
mod decal_tests {
    use super::*;
    /// Native CE capture: full alpha through 9 s, ~half at 9.5 s, gone at 10 s.
    #[test]
    fn alpha_matches_native_timeline() {
        assert_eq!(decal_alpha(10., 0.), 1.);
        assert_eq!(decal_alpha(10., 8.9), 1.);
        assert!((decal_alpha(10., 9.5) - 0.5).abs() < 1e-4);
        assert_eq!(decal_alpha(10., 10.), 0.);
    }
}
impl Blood {
    fn blood_model(&mut self, assets: &mut Assets, model: &str) -> Option<String> {
        self.models
            .entry(model.to_owned())
            .or_insert_with(|| {
                let text = assets.read(&format!("models/{model}.tik")).ok()?;
                String::from_utf8_lossy(&text).lines().find_map(|l| {
                    let mut w = l.split_whitespace();
                    (w.next()?.eq_ignore_ascii_case("bloodmodel"))
                        .then(|| w.next().map(|s| s.trim_end_matches(".tik").to_lowercase()))
                        .flatten()
                })
            })
            .clone()
    }
    /// Native hit blood (`FUN_100de6f0`): a spray at the hit plus a 50% floor decal, for any
    /// damage that bleeds, at most once every 0.2 s of play.
    pub fn hit(
        &mut self,
        assets: &mut Assets,
        specs: &BTreeMap<String, texture::MaterialSpec>,
        world: &World,
        model: &str,
        at: Vec3,
        kind: crate::combat::DamageKind,
    ) {
        if !bleeds(kind) || self.blood_model(assets, model).is_none() {
            return;
        }
        if self.last.is_some_and(|t| self.clock - t < 0.2) {
            return;
        }
        self.last = Some(self.clock);
        self.spawn(assets, specs, model, at);
        self.splat(assets, specs, world, model, at, false);
    }
    pub fn spawn(
        &mut self,
        assets: &mut Assets,
        specs: &BTreeMap<String, texture::MaterialSpec>,
        model: &str,
        at: Vec3,
    ) {
        self.spawn_scaled(assets, specs, model, at, 1.);
    }
    /// A smaller burst: Alice only loses a few droplets when hurt on PC.
    pub fn spawn_scaled(
        &mut self,
        assets: &mut Assets,
        specs: &BTreeMap<String, texture::MaterialSpec>,
        model: &str,
        at: Vec3,
        scale: f32,
    ) {
        let Some(name) = self.blood_model(assets, model) else { return };
        let template = self
            .templates
            .entry(name.clone())
            .or_insert_with(|| Attached::load(assets, &name, specs).ok().flatten());
        if let Some(t) = template {
            if self.live.len() >= 16 {
                self.live.remove(0);
            }
            self.live.push((t.fork(), at, 0., scale));
        }
    }
    fn roll(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed >> 8) as f32 / (1u32 << 24) as f32
    }
    /// A blood decal on the floor under `at`: small for a hit, large for a death puddle.
    pub fn splat(
        &mut self,
        assets: &mut Assets,
        specs: &BTreeMap<String, texture::MaterialSpec>,
        world: &World,
        model: &str,
        at: Vec3,
        puddle: bool,
    ) {
        let Some(name) = self.blood_model(assets, model) else { return };
        // Original hit blood (fgamex86 FUN_100de6f0) only places a floor decal when rand() > 0.5.
        if !puddle && self.roll() <= 0.5 {
            return;
        }
        let prefix = if name.contains("green") {
            "g_"
        } else if name.contains("sparks") {
            "o_"
        } else {
            ""
        };
        let n = 1 + (self.roll() * 6.) as usize % 6;
        let key = format!("{prefix}blood{n}_decal");
        let loaded = self.textures.entry(key.clone()).or_insert_with(|| {
            let path = texture::resolve(assets, &key, specs)?;
            let image = texture::decode(assets, &path).ok()?;
            Some(Texture2D::from_rgba8(image.width, image.height, &image.pixels))
        });
        if loaded.is_none() {
            return;
        }
        let trace = world.sweep(at + Vec3::Z * 8., at - Vec3::Z * 160., Vec3::splat(1.));
        if trace.start_solid || trace.fraction >= 1. || trace.normal.z < 0.5 {
            return;
        }
        let center = (at + Vec3::Z * 8.).lerp(at - Vec3::Z * 160., trace.fraction);
        // Original: crandom(20) on X and Y for every blood decal (CE-verified on a fire imp kill).
        let jitter = 40.;
        let (jx, jy) = ((self.roll() - 0.5) * jitter, (self.roll() - 0.5) * jitter);
        let radius = if puddle { 26. + self.roll() * 14. } else { 8. + self.roll() * 8. };
        if self.decals.len() >= MAX_DECALS {
            self.decals.remove(0);
        }
        let spin = self.roll() * std::f32::consts::TAU;
        self.decals.push(Decal {
            center: center + vec3(jx, jy, 0.) + trace.normal * 0.4,
            normal: trace.normal,
            radius,
            spin,
            life: 10.,
            age: 0.,
            texture: key,
        });
    }
    pub fn clear_decals(&mut self) {
        self.decals.clear();
    }
    pub fn update(&mut self, dt: f32) {
        self.clock += dt;
        for d in &mut self.decals {
            d.age += dt;
        }
        self.decals.retain(|d| d.age < d.life);
        for (_, _, age, _) in &mut self.live {
            *age += dt;
        }
        self.live.retain(|(_, _, age, _)| *age < EMIT + LINGER);
    }
    pub fn draw(&mut self, camera: Vec3, atmosphere: &crate::environment::Atmosphere) {
        for (effect, at, age, scale) in &mut self.live {
            let origin = Transform { translation: *at, rotation: Quat::IDENTITY };
            effect.draw(*age, *scale, |_, _| origin, |_, t, default| default && t < EMIT, camera, atmosphere);
        }
    }
    pub fn draw_decals(&self, clock: f32) {
        for d in &self.decals {
            let Some(Some(texture)) = self.textures.get(&d.texture) else { continue };
            let x = d.normal.cross(Vec3::Z).try_normalize().unwrap_or(Vec3::X);
            let y = d.normal.cross(x);
            let (sin, cos) = d.spin.sin_cos();
            let (x, y) = ((x * cos + y * sin) * d.radius, (y * cos - x * sin) * d.radius);
            let alpha = decal_alpha(d.life, d.age);
            let mesh = Mesh {
                vertices: [
                    (d.center - x - y, vec2(0., 0.)),
                    (d.center + x - y, vec2(1., 0.)),
                    (d.center + x + y, vec2(1., 1.)),
                    (d.center - x + y, vec2(0., 1.)),
                ]
                .map(|(p, uv)| Vertex::new(p.x, p.y, p.z, uv.x, uv.y, WHITE))
                .to_vec(),
                indices: vec![0, 1, 2, 0, 2, 3],
                texture: Some(texture.clone()),
            };
            crate::render_fx::skin_effect(&mesh, clock, alpha);
        }
    }
}
