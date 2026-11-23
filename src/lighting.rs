//! Bounded original-declaration lights, occluded flares and projected contact shadows.
use crate::{assets::Assets, collision::World, texture};
use anyhow::Result;
use macroquad::prelude::*;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
    pub radius: f32,
    pub only_models: bool,
    pub flare: bool,
}
thread_local! {
    static LIGHTS: RefCell<Vec<Light>> = const {RefCell::new(Vec::new())};
    static FIGURE: RefCell<Option<Texture2D>> = const {RefCell::new(None)};
    static SHADOW: RefCell<Option<Texture2D>> = const {RefCell::new(None)};
    static FLARE: RefCell<Option<Texture2D>> = const {RefCell::new(None)};
    static GENERATION: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static APPLIED: RefCell<Vec<(usize, u64, bool)>> = const { RefCell::new(Vec::new()) };
    static CROQUET: RefCell<Vec<Light>> = const {RefCell::new(Vec::new())};
}
pub fn select(mut lights: Vec<Light>, eye: Vec3, world: &World) {
    lights.retain(|l| {
        l.position.is_finite() && l.color.is_finite() && l.radius > 0. && l.radius <= 8192.
    });
    lights.sort_by(|a, b| score(*b, eye).total_cmp(&score(*a, eye)));
    lights.truncate(32);
    // A bounded visibility query rejects lights fully hidden from this room.
    lights.retain(|l| world.sweep(eye, l.position, Vec3::ZERO).fraction >= 0.995);
    lights.truncate(8);
    LIGHTS.with(|v| *v.borrow_mut() = lights);
    GENERATION.with(|g| g.set(g.get() + 1));
}
fn score(light: Light, eye: Vec3) -> f32 {
    light.radius / (light.position.distance(eye) + light.radius)
}
pub(crate) fn fixture_lights(lights: Vec<Light>) {
    LIGHTS.with(|v| *v.borrow_mut() = lights.into_iter().take(8).collect());
    GENERATION.with(|g| g.set(g.get() + 1));
}
pub(crate) fn count() -> usize {
    LIGHTS.with(|l| l.borrow().len())
}
pub fn uniforms() -> Vec<UniformDesc> {
    (0..8)
        .flat_map(|i| {
            [
                UniformDesc::new(&format!("FxLight{i}"), UniformType::Float4),
                UniformDesc::new(&format!("FxColor{i}"), UniformType::Float4),
            ]
        })
        .collect()
}
pub fn fragment(source: &str) -> String {
    let mut s = String::new();
    for i in 0..8 {
        s += &format!("uniform vec4 FxLight{i}; uniform vec4 FxColor{i};\n");
    }
    s += "vec3 dynamicLight(vec3 position){vec3 light=vec3(0.0);\n";
    for i in 0..8 {
        s+=&format!("if(FxColor{i}.w>0.5)light+=FxColor{i}.rgb*max(0.0,1.0-distance(position,FxLight{i}.xyz)/max(FxLight{i}.w,0.001));\n");
    }
    s += "return light;}\n";
    source.replace("// LIGHTS", &s)
}
const LIGHT_NAMES: [&str; 8] = ["FxLight0", "FxLight1", "FxLight2", "FxLight3", "FxLight4", "FxLight5", "FxLight6", "FxLight7"];
const COLOR_NAMES: [&str; 8] = ["FxColor0", "FxColor1", "FxColor2", "FxColor3", "FxColor4", "FxColor5", "FxColor6", "FxColor7"];
pub fn apply(material: &Material, model: bool, enabled: bool) {
    // Another caller's flags may now be on the material.
    APPLIED.with(|a| a.borrow_mut().clear());
    write_lights(material, model, enabled);
}
/// World-geometry variant: a material already carrying this frame's lights with the same
/// `enabled` flag is left alone (the shared materials are drawn hundreds of times a frame).
pub fn apply_world(material: &Material, enabled: bool) {
    let key = (material as *const Material as usize, GENERATION.with(|g| g.get()), enabled);
    let fresh = APPLIED.with(|a| {
        let mut a = a.borrow_mut();
        if a.iter().any(|k| *k == key) {
            return false;
        }
        a.retain(|k| k.0 != key.0);
        a.push(key);
        true
    });
    if fresh {
        write_lights(material, false, enabled);
    }
}
fn write_lights(material: &Material, model: bool, enabled: bool) {
    LIGHTS.with(|lights| {
        let lights = lights.borrow();
        for i in 0..8 {
            let light = lights
                .get(i)
                .filter(|l| enabled && (model || !l.only_models));
            material.set_uniform(
                LIGHT_NAMES[i],
                light.map_or(Vec4::ZERO, |l| l.position.extend(l.radius)),
            );
            material.set_uniform(
                COLOR_NAMES[i],
                light.map_or(Vec4::ZERO, |l| l.color.extend(1.)),
            );
        }
    });
}
pub fn load_art(assets: &mut Assets) -> Result<()> {
    if SHADOW.with(|s| s.borrow().is_some()) {
        return Ok(());
    }
    let specs = texture::read_materials(assets)?;
    CROQUET.with(|s| {
        *s.borrow_mut() =
            crate::particles::declared_lights(assets, "croquetball").unwrap_or_default()
    });
    for (slot, names) in [
        (false, &["markshadow", "textures/common/shadow"] as &[&str]),
        (true, &["flare", "lensflare", "textures/sprites/flare"]),
    ] {
        for name in names {
            if let Some(path) = texture::resolve(assets, name, &specs) {
                let image = texture::decode(assets, &path)?;
                let t = Texture2D::from_rgba8(image.width, image.height, &image.pixels);
                t.set_filter(FilterMode::Linear);
                if slot {
                    FLARE.with(|s| *s.borrow_mut() = Some(t));
                } else {
                    SHADOW.with(|s| *s.borrow_mut() = Some(t));
                }
                break;
            }
        }
    }
    Ok(())
}
pub fn croquet(position: Vec3) -> Vec<Light> {
    CROQUET.with(|s| {
        s.borrow()
            .iter()
            .map(|l| Light {
                position: position + l.position,
                ..*l
            })
            .collect()
    })
}
pub fn shadow(world: &World, feet: Vec3, radius: f32) {
    let Some(texture) = SHADOW.with(|s| s.borrow().clone()) else {
        return;
    };
    let hit = world.sweep(feet + Vec3::Z * 4., feet - Vec3::Z * 160., Vec3::ZERO);
    if hit.fraction >= 1. || hit.normal.z < 0.4 {
        return;
    }
    let center = (feet + Vec3::Z * 4.).lerp(feet - Vec3::Z * 160., hit.fraction) + hit.normal * 0.3;
    let strength = (0.55 * (1. - feet.distance(center) / 160.)).clamp(0., 0.55);
    let tangent = hit.normal.cross(Vec3::X).normalize_or_zero();
    let bitangent = hit.normal.cross(tangent);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (position, uv) in [(center, Vec2::splat(0.5))]
        .into_iter()
        .chain((0..=24).map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 24.;
            (
                center + (tangent * a.cos() + bitangent * a.sin()) * radius,
                vec2(a.cos(), a.sin()) * 0.5 + Vec2::splat(0.5),
            )
        }))
    {
        // Clip each rim point against nearby support; shadows never bridge a pit.
        let h = world.sweep(
            position + hit.normal * 2.,
            position - hit.normal * 5.,
            Vec3::ZERO,
        );
        vertices.push(Vertex {
            position,
            uv,
            normal: Vec4::ZERO,
            color: [
                0,
                0,
                0,
                if h.fraction < 1. {
                    (strength * 255.) as u8
                } else {
                    0
                },
            ],
        });
    }
    for i in 1..=24 {
        indices.extend([0, i, i + 1]);
    }
    crate::render_fx::effect(
        &Mesh {
            vertices,
            indices,
            texture: Some(texture),
        },
        crate::materials::Blend::Alpha,
    );
}
pub fn flares(eye: Vec3, direction: Vec3) {
    let Some(texture) = FLARE.with(|s| s.borrow().clone()) else {
        return;
    };
    LIGHTS.with(|lights| {
        for l in lights.borrow().iter().filter(|l| l.flare) {
            let right = direction.cross(Vec3::Z).try_normalize().unwrap_or(Vec3::X);
            let up = right.cross(direction).normalize_or_zero();
            let r = (if l.radius > 1. { l.radius * 0.3 } else { l.radius }).clamp(8., 48.);
            let color: [u8; 4] = Color::new(
                l.color.x,
                l.color.y,
                l.color.z,
                (eye.distance(l.position) / 32.).min(1.),
            )
            .into();
            let vertices = [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)]
                .into_iter()
                .map(|(x, y)| Vertex {
                    position: l.position + (right * x + up * y) * r,
                    uv: vec2(x * 0.5 + 0.5, 0.5 - y * 0.5),
                    normal: Vec4::ZERO,
                    color,
                })
                .collect();
            crate::render_fx::effect(
                &Mesh {
                    vertices,
                    indices: vec![0, 1, 2, 0, 2, 3],
                    texture: Some(texture.clone()),
                },
                crate::materials::Blend::Add,
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn light_selection_is_bounded_and_rejects_nonfinite() {
        let mut lights = (0..100)
            .map(|i| Light {
                position: Vec3::X * i as f32,
                color: Vec3::ONE,
                radius: 100.,
                only_models: false,
                flare: false,
            })
            .collect::<Vec<_>>();
        lights.push(Light {
            position: Vec3::splat(f32::NAN),
            ..lights[0]
        });
        select(lights, Vec3::ZERO, &World::fixture(&[]));
        LIGHTS.with(|l| {
            let l = l.borrow();
            assert_eq!(l.len(), 8);
            assert_eq!(l[0].position, Vec3::ZERO);
        });
    }
    #[test]
    fn opaque_wall_rejects_light_and_flare() {
        let world = World::fixture(&[(vec3(5., -20., -20.), vec3(6., 20., 20.))]);
        select(
            vec![Light {
                position: Vec3::X * 10.,
                color: Vec3::ONE,
                radius: 100.,
                only_models: false,
                flare: true,
            }],
            Vec3::ZERO,
            &world,
        );
        assert_eq!(count(), 0);
    }
}
/// Half-extent in world units and mask resolution of the figure shadow.
const FIGURE_RADIUS: f32 = 64.;
const FIGURE_SIZE: usize = 128;
/// Light direction the original measured on Alice's entity (`lightDir`, backEnd entity +0x1d8) in
/// the Duchess/Hatter area; the game reads it from the level lighting at her position.
pub const FIGURE_LIGHT: Vec3 = Vec3::new(-0.490, 0.235, 0.840);
/// The original's `projectionShadow` shader (scripts/common.shader) with `deformVertexes
/// projectionShadow` (RB_ProjectionShadowDeform, FUN_004a2970): every model triangle is slid along
/// the entity light direction onto the floor plane at its origin (light.z clamped to >= 0.5 as
/// `lightDir + (0.5 - d) * up`), then drawn black with `GL_SRC_ALPHA GL_ONE_MINUS_SRC_ALPHA` and
/// `alphaGen entity`. The entity alpha comes from CG_EntityShadow (FUN_3001eb60): trace 128.5 units
/// down from 5 above the origin and use `(1 - fraction) * 0.7`. Overlapping front-facing
/// triangles blend again, exactly as the separate draws do natively.
pub fn figure_shadow(world: &World, feet: Vec3, triangles: &[[Vec3; 3]], light: Vec3) {
    let from = feet + Vec3::Z * 5.;
    let to = from - Vec3::Z * 128.5;
    let hit = world.sweep(from, to, Vec3::ZERO);
    if hit.fraction >= 1. || hit.normal.z < 0.4 || triangles.is_empty() {
        return;
    }
    let center = from.lerp(to, hit.fraction) + hit.normal * 0.3;
    let alpha = ((1. - hit.fraction) * 0.7).clamp(0., 1.);
    let mut light = light.normalize_or_zero();
    if light.z < 0.5 {
        light.z += 0.5 - light.z;
    }
    let slide = light / light.z;
    let n = FIGURE_SIZE;
    let scale = n as f32 / (2. * FIGURE_RADIUS);
    let mut layers = vec![0_u8; n * n];
    let pixel = |p: Vec3| {
        let flat = p - slide * (p.z - feet.z);
        vec2(
            (flat.x - feet.x) * scale + n as f32 * 0.5,
            (flat.y - feet.y) * scale + n as f32 * 0.5,
        )
    };
    for t in triangles {
        let [a, b, c] = [pixel(t[0]), pixel(t[1]), pixel(t[2])];
        let area = (b - a).perp_dot(c - a);
        if area <= 1e-6 {
            continue;
        }
        let lo = a.min(b).min(c).floor().max(Vec2::ZERO);
        let hi = a.max(b).max(c).ceil().min(Vec2::splat(n as f32 - 1.));
        for y in lo.y as usize..=hi.y as usize {
            for x in lo.x as usize..=hi.x as usize {
                let p = vec2(x as f32 + 0.5, y as f32 + 0.5);
                let w0 = (b - p).perp_dot(c - p) / area;
                let w1 = (c - p).perp_dot(a - p) / area;
                let w2 = 1. - w0 - w1;
                if w0 >= 0. && w1 >= 0. && w2 >= 0. {
                    let l = &mut layers[y * n + x];
                    *l = l.saturating_add(1);
                }
            }
        }
    }
    let mut bytes = Vec::with_capacity(n * n * 4);
    for k in layers {
        let covered = 1. - (1. - alpha).powi(k as i32);
        bytes.extend([0, 0, 0, (covered * 255.) as u8]);
    }
    let texture = FIGURE.with(|f| {
        let mut f = f.borrow_mut();
        let image = Image {
            bytes,
            width: n as u16,
            height: n as u16,
        };
        match f.as_ref() {
            Some(t) => t.update(&image),
            None => {
                let t = Texture2D::from_image(&image);
                t.set_filter(FilterMode::Linear);
                *f = Some(t);
            }
        }
        f.clone().unwrap()
    });
    // The mask is centred on the feet; the quad lies on the floor plane found by the trace.
    let vertices = [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)]
        .into_iter()
        .map(|(x, y): (f32, f32)| {
            let dx = x * FIGURE_RADIUS;
            let dy = y * FIGURE_RADIUS;
            let dz = -(hit.normal.x * dx + hit.normal.y * dy) / hit.normal.z;
            Vertex {
                position: vec3(feet.x, feet.y, center.z) + vec3(dx, dy, dz),
                uv: vec2(x * 0.5 + 0.5, y * 0.5 + 0.5),
                normal: Vec4::ZERO,
                color: [255, 255, 255, 255],
            }
        })
        .collect();
    crate::render_fx::effect(
        &Mesh {
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
            texture: Some(texture),
        },
        crate::materials::Blend::Alpha,
    );
}
