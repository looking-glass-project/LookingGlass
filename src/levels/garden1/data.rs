use super::*;
use crate::{
    cinematic::Track,
    fortress::spline::Spline,
    skeletal::{Animation, Definition, Skeleton},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Data {
    pub points: BTreeMap<String, Transform>,
    pub cameras: Vec<Spline>,
    pub swim: Spline,
    pub deadtree: Vec<Transform>,
    pub world: World,
    pub currents: Vec<crate::traversal::current::Path>,
    clips: BTreeMap<String, (f32, f32, f32)>,
    pub portals: Vec<(usize, Transform, Collider)>,
    launch: Vec<Vec3>,
}
pub(super) fn at(e: &super::super::Entity) -> Transform {
    let angles = e
        .get("angles")
        .and_then(|s| crate::interaction::vector(s))
        .unwrap_or_else(|| {
            vec3(
                0.,
                e.get("angle").and_then(|s| s.parse().ok()).unwrap_or(0.),
                0.,
            )
        });
    Transform {
        translation: e
            .get("origin")
            .and_then(|s| crate::interaction::vector(s))
            .unwrap_or_default(),
        rotation: Quat::from_rotation_z(angles.y.to_radians())
            * Quat::from_rotation_y(angles.x.to_radians())
            * Quat::from_rotation_x(angles.z.to_radians()),
    }
}
impl Data {
    pub fn load(a: &mut Assets, map: &Bsp) -> Result<Self> {
        let mut points = BTreeMap::new();
        for e in &map.entities {
            if let Some(n) = e.get("targetname") {
                points.entry(n.clone()).or_insert(at(e));
            }
        }
        for n in [
            "garden1_start1",
            "turtle_waterpos1",
            "turtle_jumppos1",
            "turtle_pos1",
            "alice_pos1",
            "alice_posx1",
            "rabbit_posx1",
            "rabbit_jumppos1",
            "rabbit_jump1",
            "alice_shell_fire",
        ] {
            ensure!(points.contains_key(n), "Missing garden marker {n}");
        }
        let mut clips = BTreeMap::new();
        for (model, names) in [
            ("alice", ALICE_CLIPS),
            ("c_mockturtle", TURTLE_CLIPS),
            ("c_whiterabbit", RABBIT_CLIPS),
        ] {
            let d = Definition::load(a, &format!("models/{model}.tik"))?;
            let rig = Skeleton::parse(&a.read(&format!("{}/{}", d.path, d.model))?)?;
            for n in names {
                let f = d
                    .animations
                    .get(*n)
                    .with_context(|| format!("Missing garden clip {model}/{n}"))?;
                let c = Animation::parse(&a.read(&format!("{}/{f}", d.path))?, rig.bones.len())?;
                clips.insert(
                    format!("{model}/{n}"),
                    (
                        c.duration(),
                        (c.distance * d.scale / c.duration()).max(1.),
                        c.frame_time,
                    ),
                );
            }
        }
        let cameras = (1..=4)
            .map(|n| {
                Ok(Spline::camera_track(
                    Track::load(a, &format!("garden1_path{n}"))?
                        .controls()
                        .collect(),
                ))
            })
            .collect::<Result<_>>()?;
        let mut nodes = Vec::new();
        let mut seen = BTreeSet::new();
        let mut next = Some("turtle_swimpath1");
        while let Some(n) = next {
            ensure!(seen.insert(n), "Unexpected loop in garden Turtle path");
            let e = map
                .entities
                .iter()
                .find(|e| e.get("targetname").is_some_and(|s| s == n))
                .context("Missing garden swim node")?;
            let p = at(e);
            let speed = e
                .get("speed")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(1.);
            ensure!(speed.is_finite() && speed > 0., "Invalid garden swim speed");
            nodes.push((p.translation, p.rotation, speed));
            next = e.get("target").map(String::as_str);
        }
        let world = World::from_bsp(map)?;
        let mut portals = Vec::new();
        for n in ["teleporter_start", "teleport_end"] {
            let e = map
                .entities
                .iter()
                .find(|e| e.get("targetname").is_some_and(|s| s == n))
                .context("Missing garden portal")?;
            let m = e["model"].trim_start_matches('*').parse()?;
            let p = at(e);
            portals.push((
                m,
                p,
                Collider::model(map, m, p.translation, p.rotation, true)?,
            ));
        }
        // Sample the supplied launch volume against BSP collision. This scene's
        // fake actor uses its target-apex impulse, not an invented rail to shore.
        let push = world
            .traversal
            .pushes
            .iter()
            .find(|p| p.id.0 == 117)
            .context("Missing arrival launch volume")?
            .clone();
        // The body below receives the impulse once, by hand, so the trigger itself is off here.
        let feet = points["garden1_start1"].translation;
        ensure!(
            push.volume.touches(
                feet + crate::collision::PLAYER_CENTER,
                feet + crate::collision::PLAYER_CENTER,
                crate::collision::PLAYER_HALF
            ),
            "Arrival outside launch volume"
        );
        // Measured from the native game with Cheat Engine (fgamex86.dll): the fake actor sits at
        // z=-688.2 under the start point when the push fires, flies the target-apex arc
        // untouched by the level geometry (apex z=-195), meets the water surface at z=-418.5
        // and is stopped by the water within ~0.4 s, floating at z=-414.8.
        const WATER_Z: f32 = -418.5;
        const FLOAT_Z: f32 = -414.8;
        // The original steps its entities in 20 Hz server frames: the push test happens at the
        // start of a frame and the whole frame then moves at that velocity, so the actor gets one
        // more full-speed frame after it leaves the trigger volume.
        const FRAME: f32 = 0.05;
        let mut pos = vec3(feet.x, feet.y, -688.2);
        let mut v = push.direction;
        let mut frames = vec![pos];
        let mut wet = false;
        let mut hold = 0;
        for _ in 0..400 {
            if wet {
                v *= 0.8;
                pos += v * FRAME;
                pos.z += (FLOAT_Z - pos.z) * 0.25;
            } else {
                let inside = push.volume.touches(
                    pos + crate::collision::PLAYER_CENTER,
                    pos + crate::collision::PLAYER_CENTER,
                    crate::collision::PLAYER_HALF,
                );
                if inside {
                    v = push.direction;
                    hold = 2;
                } else if hold > 0 {
                    // The touch is processed after the move, so the push lands a frame late.
                    hold -= 1;
                    v = push.direction;
                } else {
                    v.z -= crate::movement::GRAVITY * FRAME;
                }
                pos += v * FRAME;
                if v.z < 0. && pos.z <= WATER_Z {
                    wet = true;
                    v = vec3(v.x * 0.66, v.y * 0.66, 0.);
                }
            }
            frames.push(pos);
        }
        let mut launch = Vec::new();
        for i in 0..1200 {
            let f = i as f32 * crate::movement::FIXED_DT / FRAME;
            let n = (f as usize).min(frames.len() - 1);
            launch.push(frames[n].lerp(frames[(n + 1).min(frames.len() - 1)], f.fract()));
        }
        Ok(Self {
            deadtree: [420, 421, 422, 423, 424, 460, 650, 647, 425, 426, 427]
                .into_iter()
                .map(|id| at(&map.entities[id]))
                .collect(),
            points,
            cameras,
            swim: Spline::new(nodes, false),
            currents: crate::traversal::current::Path::load(map, &world)?,
            world,
            clips,
            portals,
            launch,
        })
    }
    pub fn duration(&self, model: &str, clip: &str) -> f32 {
        self.clips[&format!("{model}/{clip}")].0
    }
    pub fn speed(&self, model: &str, clip: &str) -> f32 {
        self.clips[&format!("{model}/{clip}")].1
    }
    pub fn frame(&self, model: &str, clip: &str) -> f32 {
        self.clips[&format!("{model}/{clip}")].2
    }
    pub fn launch_end(&self) -> f32 {
        3. + self.duration("alice", "pain_knockdown") + self.duration("alice", "idle")
    }
    pub fn launch_pose(&self, t: f32) -> Transform {
        let frame = (t.max(0.) / crate::movement::FIXED_DT).min((self.launch.len() - 1) as f32);
        let n = frame as usize;
        let p = self.launch[n].lerp(
            self.launch[(n + 1).min(self.launch.len() - 1)],
            frame.fract(),
        );
        Transform {
            translation: p,
            rotation: self.points["garden1_start1"].rotation,
        }
    }
    pub fn rabbit_run(&self) -> f32 {
        self.points["rabbit_posx1"]
            .translation
            .distance(self.points["rabbit_jumppos1"].translation)
            / self.speed("c_whiterabbit", "run")
    }
    pub fn deadtree_duration(&self) -> f32 {
        self.deadtree
            .windows(2)
            .map(|p| p[0].translation.distance(p[1].translation))
            .sum::<f32>()
            / self.speed("c_whiterabbit", "run")
    }
    pub fn deadtree_pose(&self, time: f32) -> Transform {
        let mut distance = time.max(0.) * self.speed("c_whiterabbit", "run");
        for p in self.deadtree.windows(2) {
            let delta = p[1].translation - p[0].translation;
            if distance <= delta.length() {
                return self.footing(
                    Transform {
                        translation: p[0].translation + delta.normalize_or_zero() * distance,
                        rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
                    },
                    vec3(12., 12., 32.),
                );
            }
            distance -= delta.length();
        }
        self.footing(*self.deadtree.last().unwrap(), vec3(12., 12., 32.))
    }
    pub fn footing(&self, mut pose: Transform, half: Vec3) -> Transform {
        pose.translation = self
            .world
            .actor_footing(pose.translation, Vec3::Z * half.z, half, 96.)
            .unwrap_or(pose.translation);
        pose
    }
}

#[cfg(test)]
mod launch_tests {
    #[test]
    #[ignore = "requires LOOKING_GLASS_TEST_DATA"]
    fn launch_report() {
        let base = std::env::var_os("LOOKING_GLASS_TEST_DATA").unwrap();
        let mut a = crate::assets::Assets::open(std::path::Path::new(&base)).unwrap();
        let map = crate::bsp::Bsp::parse(&a.read("maps/garden1.bsp").unwrap()).unwrap();
        let d = super::Data::load(&mut a, &map).unwrap();
        for i in 0..40 {
            let t = i as f32 * 0.1;
            println!("LAUNCH t={t} {:?}", d.launch_pose(t).translation);
        }
        println!("START {:?}", d.points["garden1_start1"].translation);
        println!("WATERPOS {:?}", d.points["turtle_waterpos1"].translation);
    }
}
