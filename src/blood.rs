//! Hit blood: Sentient damage spawns the actor's tik `bloodmodel` (blood_spray,
//! blood_spray_green or blood_spray_sparks) at the hit as a short-lived effect
//! (FAKK2 Sentient::ArmorDamage posts EV_Remove 1 s after spawning it).
use crate::{assets::Assets, particles::Attached, skeletal::Transform, texture};
use macroquad::prelude::*;
use std::collections::BTreeMap;

const EMIT: f32 = 1.;
const LINGER: f32 = 0.8;

#[derive(Default)]
pub struct Blood {
    /// Actor model -> its bloodmodel template (None: the tik bleeds nothing).
    models: BTreeMap<String, Option<String>>,
    templates: BTreeMap<String, Option<Attached>>,
    live: Vec<(Attached, Vec3, f32)>,
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
    pub fn spawn(
        &mut self,
        assets: &mut Assets,
        specs: &BTreeMap<String, texture::MaterialSpec>,
        model: &str,
        at: Vec3,
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
            self.live.push((t.fork(), at, 0.));
        }
    }
    pub fn update(&mut self, dt: f32) {
        for (_, _, age) in &mut self.live {
            *age += dt;
        }
        self.live.retain(|(_, _, age)| *age < EMIT + LINGER);
    }
    pub fn draw(&mut self, camera: Vec3, atmosphere: &crate::environment::Atmosphere) {
        for (effect, at, age) in &mut self.live {
            let origin = Transform { translation: *at, rotation: Quat::IDENTITY };
            effect.draw(*age, 1., |_, _| origin, |_, t, default| default && t < EMIT, camera, atmosphere);
        }
    }
}
