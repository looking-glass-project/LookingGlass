//! Hit blood: Sentient damage spawns the actor's tik `bloodmodel` (blood_spray,
//! blood_spray_green or blood_spray_sparks) at the hit as a short-lived effect
//! (FAKK2 Sentient::ArmorDamage posts EV_Remove 1 s after spawning it).
use crate::{
    assets::Assets, collision::World, particles::Attached, skeletal::Transform, texture,
};
use macroquad::prelude::*;
use std::collections::BTreeMap;

const EMIT: f32 = 1.;
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
const FADE: f32 = 4.;
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
        let jitter = if puddle { 0. } else { 14. };
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
            life: 13.,
            age: 0.,
            texture: key,
        });
    }
    pub fn clear_decals(&mut self) {
        self.decals.clear();
    }
    pub fn update(&mut self, dt: f32) {
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
            let alpha = ((d.life - d.age) / FADE).clamp(0., 1.);
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
