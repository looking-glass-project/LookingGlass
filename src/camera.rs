//! Responsive orbit with collision-safe retraction and a damped return from obstacles.
use crate::collision::World;
use macroquad::prelude::*;

mod handoff;
pub use handoff::Handoff;

const HALF: Vec3 = Vec3::splat(4.);
const RELEASE_DELAY: f32 = 0.12;
/// Camera pivot height over the feet, measured live in the original (camera z - player z at level pitch).
const NATIVE_PIVOT: f32 = 90.;

#[derive(Default)]
pub struct Follow {
    previous_feet: Option<Vec3>,
    height: f32,
    arm: f32,
    release: f32,
    visible: bool,
}

impl Follow {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn update(
        &mut self,
        world: &World,
        feet: Vec3,
        direction: Vec3,
        distance: f32,
        dt: f32,
    ) -> (Vec3, Vec3, bool) {
        let dt = dt.clamp(0., 0.05);
        let fresh = self
            .previous_feet
            .is_none_or(|last| last.distance(feet) > 128.);
        self.previous_feet = Some(feet);
        // Original cgame (FUN_30025a40): the arm runs back along the full view direction from a
        // point 90 units above the player's origin, and the camera looks along the view, not at Alice.
        let pivot = feet + Vec3::Z * NATIVE_PIVOT;
        // Filter small steps/landings vertically, without lagging mouse aim or strafing.
        self.height = if fresh {
            pivot.z
        } else {
            (self.height + (pivot.z - self.height) * (1. - (-18. * dt).exp()))
                .clamp(pivot.z - 12., pivot.z + 12.)
        };
        let mut target = vec3(pivot.x, pivot.y, self.height);
        let anchor_hit = world.camera_sweep(pivot, target, HALF);
        target = pivot.lerp(
            target,
            if anchor_hit.start_solid {
                0.
            } else {
                anchor_hit.fraction
            },
        );
        let offset = -direction.normalize_or_zero() * distance.clamp(0., 1024.);
        let (eye, _, visible) = self.resolve(world, target, offset, dt, fresh, f32::INFINITY);
        (eye, eye + direction, visible)
    }

    /// Preserve the rail camera's framing, but keep its arm out of tunnel walls.
    pub fn rail(
        &mut self,
        world: &World,
        camera: crate::cinematic::Camera,
        ahead: &[crate::cinematic::Camera],
        dt: f32,
    ) -> crate::cinematic::Camera {
        let fresh = self
            .previous_feet
            .is_none_or(|p| p.distance(camera.target) > 128.);
        self.previous_feet = Some(camera.target);
        let length = camera.eye.distance(camera.target);
        let anticipated = ahead.iter().fold(length, |limit, next| {
            let hit = world.camera_sweep(next.target, next.eye, HALF);
            // An embedded future anchor cannot supply a useful arm constraint.
            if hit.start_solid {
                limit
            } else {
                limit.min(length * hit.fraction)
            }
        });
        let (eye, target, _) = self.resolve(
            world,
            camera.target,
            camera.eye - camera.target,
            dt.clamp(0., 0.05),
            fresh,
            anticipated,
        );
        crate::cinematic::Camera::look(eye, target)
    }

    fn resolve(
        &mut self,
        world: &World,
        target: Vec3,
        offset: Vec3,
        dt: f32,
        fresh: bool,
        anticipated: f32,
    ) -> (Vec3, Vec3, bool) {
        let length = offset.length();
        let hit = world.camera_sweep(target, target + offset, HALF);
        let limit = length * if hit.start_solid { 0. } else { hit.fraction };
        let preferred = limit.min(anticipated);
        if fresh {
            self.arm = preferred;
            self.release = if limit < length { RELEASE_DELAY } else { 0. };
        } else if limit < self.arm || (limit < length && limit <= self.arm + 0.01) {
            // Safety wins over smoothing when a wall or moving object closes in.
            self.arm = limit;
            self.release = RELEASE_DELAY;
        } else if preferred < self.arm {
            // The known rail route gives time to ease inward before the hard stop.
            self.arm += (preferred - self.arm) * (1. - (-12. * dt).exp());
            self.release = RELEASE_DELAY;
        } else {
            let ease_dt = (dt - self.release).max(0.);
            self.release = (self.release - dt).max(0.);
            self.arm += (preferred - self.arm) * (1. - (-8. * ease_dt).exp());
        }
        let eye = target + offset * (self.arm / length.max(0.001));
        // Hysteresis avoids flickering Alice on/off as the camera grazes an edge.
        self.visible = self.arm > if self.visible { 30. } else { 42. };
        let look_at = if self.arm < 0.1 {
            eye - offset.normalize_or_zero()
        } else {
            target
        };
        (eye, look_at, self.visible)
    }
}

/// Exercise real map corners, doorways and moving-model collision from supported feet.
pub fn check(assets: &mut crate::assets::Assets) -> anyhow::Result<()> {
    use crate::{bsp::Bsp, interaction, movement::Player};
    use anyhow::ensure;
    for name in ["gvillage", "skool1", "skool2"] {
        let map = Bsp::parse(&assets.read(&format!("maps/{name}.bsp"))?)?;
        let mut world = World::from_bsp(&map)?;
        let mut interactions = interaction::Interactions::load(&map)?;
        interactions.set_entry(assets, &map, name, None)?;
        interactions.sync(&mut world);
        let mut points = vec![interaction::spawn(&map, None).0];
        points.extend(
            map.entities
                .iter()
                .filter_map(|e| e.get("origin").and_then(|s| interaction::vector(s))),
        );
        let mut checked = 0;
        let mut blocked = 0;
        for point in points {
            let Some(player) = Player::spawn(&world, point) else {
                continue;
            };
            if !world.body_clear(player.feet) {
                continue;
            }
            let mut camera = Follow::default();
            for frame in 0..360 {
                let yaw = frame as f32 * std::f32::consts::TAU / 360.;
                let pitch = (yaw * 2.).sin() * 0.7;
                let direction = vec3(
                    yaw.cos() * pitch.cos(),
                    yaw.sin() * pitch.cos(),
                    pitch.sin(),
                );
                let (eye, _, _) = camera.update(&world, player.feet, direction, 220., 1. / 60.);
                let pivot = player.feet + Vec3::Z * 36.;
                let sight = world.camera_sweep(pivot, eye, HALF * 0.99);
                ensure!(
                    !world.camera_sweep(eye, eye, HALF * 0.99).start_solid
                        && !sight.start_solid
                        && sight.fraction > 0.999,
                    "Camera crossed collision in {name}, feet={:?}, yaw={yaw}",
                    player.feet
                );
                blocked += usize::from(camera.arm < 200.);
            }
            checked += 1;
            if checked == 48 {
                break;
            }
        }
        ensure!(
            checked >= 8 && blocked > 0,
            "Insufficient camera collision coverage in {name}"
        );
        println!("PASS camera {name}: {checked} supported positions, {} orbit samples, {blocked} shortened arms, clear camera volumes and sightlines", checked * 360);
    }
    crate::pandemonium::cinema::camera_check(assets)
}

#[cfg(test)]
mod tests {
    #[test]
    fn idle_view_settles_at_native_yaw_and_level_pitch() {
        let mut v = super::IdleView::default();
        let start = vec3(0.6, 0.6, 0.5).normalize();
        let mut d = start;
        for _ in 0..120 {
            d = v.update(start, true, 1. / 60.);
        }
        let yaw = (d.y.atan2(d.x) - start.y.atan2(start.x)).rem_euclid(std::f32::consts::TAU);
        assert!((yaw - super::IdleView::YAW).abs() < 0.01 && d.z.abs() < 0.01);
        for _ in 0..120 {
            d = v.update(start, false, 1. / 60.);
        }
        assert!(d.distance(start) < 1e-3);
    }

    use super::*;

    #[test]
    fn bite_camera_follows_the_recorded_topdown_behaviour() {
        let world = World::fixture(&[]);
        let mut bite = Bite::default();
        let head = vec3(0., 0., Bite::VIEW_HEIGHT);
        let dt = 1. / 60.;
        let mut last = None;
        for frame in 0..180 {
            // Frame 34.5 passes the first bite cue.
            last = bite.update(&world, Some(frame as f32 / 3.), Vec3::X, head, dt);
        }
        let camera = last.unwrap();
        // Looking almost straight down at the held player, 128 back along the view.
        let forward = (camera.target - camera.eye).normalize();
        assert!((forward.z + 75f32.to_radians().sin()).abs() < 0.06, "{forward}");
        assert!(camera.eye.z > head.z + 100.);
        // A bite kicks roll together with pitch, then it decays away.
        let mut bite = Bite::default();
        bite.update(&world, Some(33.), Vec3::X, head, dt);
        let kicked = bite.update(&world, Some(34.), Vec3::X, head, dt).unwrap();
        assert!((kicked.up - Vec3::Z).length() > 0.03);
        // Release blends back to the player's view within 0.15 s.
        let mut done = false;
        for _ in 0..12 {
            done |= bite.update(&world, None, Vec3::X, head, dt).is_none();
        }
        assert!(done);
    }

    fn wall(x: f32) -> World {
        World::fixture(&[(vec3(x - 10., -200., -100.), vec3(x, 200., 200.))])
    }
    fn safe(world: &World, eye: Vec3, target: Vec3) {
        assert!(!world.camera_sweep(eye, eye, HALF * 0.99).start_solid);
        let sight = world.camera_sweep(target, eye, HALF * 0.99);
        assert!(!sight.start_solid && sight.fraction > 0.999);
    }

    #[test]
    fn wall_retraction_is_safe_without_the_old_overhead_lift() {
        let world = wall(-22.);
        let mut camera = Follow::default();
        let (eye, target, visible) = camera.update(&world, Vec3::ZERO, Vec3::X, 132., 1. / 60.);
        safe(&world, eye, Vec3::Z * NATIVE_PIVOT);
        assert!(!visible);
        assert!(
            eye.z - NATIVE_PIVOT < 3.,
            "A close wall must not swing the camera overhead"
        );
        assert!(eye.x > -18.);
    }

    #[test]
    fn intermittent_obstacles_cannot_pump_camera_back_and_forth() {
        let empty = World::fixture(&[]);
        let wall = wall(-50.);
        let mut camera = Follow::default();
        camera.update(&empty, Vec3::ZERO, Vec3::X, 132., 1. / 60.);
        let (near, target, _) = camera.update(&wall, Vec3::ZERO, Vec3::X, 132., 1. / 60.);
        safe(&wall, near, Vec3::Z * NATIVE_PIVOT);
        for i in 0..30 {
            let w = if i % 2 == 0 { &empty } else { &wall };
            let (eye, target, _) = camera.update(w, Vec3::ZERO, Vec3::X, 132., 1. / 60.);
            safe(w, eye, Vec3::Z * NATIVE_PIVOT);
            assert!(eye.distance(near) < 9.);
        }
        let mut previous = near;
        for _ in 0..90 {
            let (eye, target, _) = camera.update(&empty, Vec3::ZERO, Vec3::X, 132., 1. / 60.);
            safe(&empty, eye, Vec3::Z * NATIVE_PIVOT);
            assert!(eye.distance(previous) < 12.);
            previous = eye;
        }
        assert!(camera.arm > 131.9);
    }

    #[test]
    fn orbit_around_a_corner_always_keeps_a_clear_sightline() {
        let world = World::fixture(&[
            (vec3(-80., -300., -40.), vec3(-70., 70., 200.)),
            (vec3(-300., 70., -40.), vec3(70., 80., 200.)),
        ]);
        let mut camera = Follow::default();
        for i in 0..1440 {
            let angle = i as f32 * 0.013;
            let direction = vec3(angle.cos(), angle.sin(), 0.);
            let (eye, target, _) = camera.update(&world, Vec3::ZERO, direction, 220., 1. / 120.);
            safe(&world, eye, Vec3::Z * NATIVE_PIVOT);
        }
    }

    #[test]
    fn steps_pause_teleports_and_frame_rates_remain_stable() {
        let empty = World::fixture(&[]);
        let wall = wall(-50.);
        let mut ends = Vec::new();
        for hz in [30, 60, 144] {
            let mut camera = Follow::default();
            camera.update(&wall, Vec3::ZERO, Vec3::X, 132., 0.);
            for _ in 0..hz {
                camera.update(&empty, Vec3::ZERO, Vec3::X, 132., 1. / hz as f32);
            }
            ends.push(camera.arm);
        }
        assert!(ends.windows(2).all(|p| (p[0] - p[1]).abs() < 0.001));
        let mut camera = Follow::default();
        camera.update(&empty, Vec3::ZERO, Vec3::X, 132., 0.);
        let (eye, _, _) = camera.update(&empty, Vec3::Z * 18., Vec3::X, 132., 1. / 60.);
        assert!((96. ..108.).contains(&eye.z));
        assert_eq!(
            camera.update(&empty, Vec3::Z * 18., Vec3::X, 132., 0.).0,
            eye
        );
        let (eye, _, _) = camera.update(&empty, Vec3::Z * 1000., Vec3::X, 132., 1. / 60.);
        assert_eq!(eye.z, 1090.);
        camera.reset();
        let (eye, _, _) = camera.update(&empty, Vec3::Z * 18., Vec3::X, 132., 0.);
        assert_eq!(eye.z, 108.);
    }
}

/// `camera bulletcam 0 0 0 1 damped animate waterice` (alice_torso.st FROZEN):
/// an `fx_bulletcam` (scale 1.7) at Alice's origin and facing plays
/// models/camera/waterice.tan; the view sits on `tag_camera` and looks at
/// `tag_target`, orbiting and closing in while she is encased.
pub struct FrozenOrbit {
    camera: Vec<Vec3>,
    target: Vec<Vec3>,
    frame_time: f32,
}
impl FrozenOrbit {
    const SCALE: f32 = 1.7;
    pub fn load(assets: &mut crate::assets::Assets) -> anyhow::Result<Self> {
        let tan = crate::tan::Model::parse(&assets.read("models/camera/waterice.tan")?)?;
        let tag = |name: &str| {
            tan.tags
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("waterice.tan lacks {name}"))
        };
        Ok(Self {
            camera: tag("tag_camera")?,
            target: tag("tag_target")?,
            frame_time: tan.frame_time,
        })
    }
    fn sample(points: &[Vec3], frame_time: f32, time: f32) -> Vec3 {
        let last = points.len() - 1;
        let at = (time.max(0.) / frame_time).min(last as f32);
        let i = (at.floor() as usize).min(last);
        points[i].lerp(points[(i + 1).min(last)], at - i as f32)
    }
    pub fn camera(
        &self,
        world: &World,
        time: f32,
        origin: Vec3,
        yaw: f32,
    ) -> crate::cinematic::Camera {
        let rotation = Quat::from_rotation_z(yaw);
        let place = |p: Vec3| origin + rotation * (p * Self::SCALE);
        let target = place(Self::sample(&self.target, self.frame_time, time));
        let wanted = place(Self::sample(&self.camera, self.frame_time, time));
        // Keep the lens out of walls: pull it in along the line to the target.
        let hit = world.sweep(target, wanted, HALF);
        let eye = if hit.start_solid {
            target
        } else {
            target.lerp(wanted, hit.fraction)
        };
        crate::cinematic::Camera::look(eye, target)
    }
}

/// The original's camera while the Duchess holds Alice, read from the running game: Alice's
/// HELD state selects the engine's `topdown` camera. Yaw locks where the player was looking,
/// pitch eases to 75 degrees (about 11/s), each of the three bites kicks pitch and roll
/// together to 4.7 degrees (decaying near 1.95/s), and on release the view blends back to the
/// player's own in about 0.15 s. The follow camera then sits `cg_cameradist` (128) back from the
/// view origin, raised by `cg_cameraheight` and `cg_cameraverticaldisplacement` (16 each).
#[derive(Default)]
pub struct Bite {
    active: bool,
    yaw: f32,
    pitch: f32,
    kick: f32,
    frame: f32,
    leaving: Option<(f32, f32, f32)>,
}
impl Bite {
    /// Height of the view origin above Alice's feet in the recorded camera.
    pub const VIEW_HEIGHT: f32 = 58.;
    const RELEASE: f32 = 0.15;
    pub fn update(
        &mut self,
        world: &crate::collision::World,
        frame: Option<f32>,
        direction: Vec3,
        head: Vec3,
        dt: f32,
    ) -> Option<crate::cinematic::Camera> {
        let player_yaw = direction.y.atan2(direction.x);
        let player_pitch = (-direction.normalize_or_zero().z).clamp(-1., 1.).asin();
        let rate = 1. - (-11. * dt).exp();
        let mut pitch;
        let mut roll;
        if let Some(frame) = frame {
            if !self.active {
                *self = Self { active: true, yaw: player_yaw, pitch: player_pitch, kick: 0., frame, leaving: None };
            }
            self.leaving = None;
            self.pitch += (75f32.to_radians() - self.pitch) * rate;
            if [34., 44., 54.].iter().any(|&f| self.frame < f && frame >= f) {
                self.kick = 4.7f32.to_radians();
            }
            self.frame = frame;
            self.kick *= (-1.95 * dt).exp();
            pitch = self.pitch + self.kick;
            roll = self.kick;
        } else if self.active {
            let (from_pitch, from_kick, elapsed) = self.leaving.get_or_insert((self.pitch + self.kick, self.kick, 0.));
            *elapsed += dt;
            let t = (*elapsed / Self::RELEASE).clamp(0., 1.);
            pitch = *from_pitch + (player_pitch - *from_pitch) * t;
            roll = *from_kick * (1. - t);
            let turn = (player_yaw - self.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
            self.yaw += turn * t;
            if t >= 1. {
                self.active = false;
                self.leaving = None;
                return None;
            }
        } else {
            return None;
        }
        pitch = pitch.clamp(-1.5, 1.5);
        roll = roll.clamp(-0.5, 0.5);
        let forward = vec3(pitch.cos() * self.yaw.cos(), pitch.cos() * self.yaw.sin(), -pitch.sin());
        let origin = head + Vec3::Z * 16.;
        let eye = origin - forward * 128. + Vec3::Z * 16.;
        let trace = world.sweep(origin, eye, Vec3::splat(4.));
        let eye = if trace.start_solid { eye } else { origin.lerp(eye, (trace.fraction - 0.01).clamp(0., 1.)) };
        let mut camera = crate::cinematic::Camera::look(eye, eye + forward * 128.);
        // Positive roll in the original tilts the view clockwise.
        let right = forward.cross(Vec3::Z).normalize_or_zero();
        if right != Vec3::ZERO {
            let up = right.cross(forward).normalize_or_zero();
            camera.up = up * roll.cos() + right * roll.sin();
        }
        Some(camera)
    }
}

/// Native `camera frontright` idle view. fgame FUN_100b82e0 case 5 sets camera flags |= 2|8, pitch
/// offset 0 and yaw offset 210.0 (0x10173b60); cgame subtracts it from the view yaw (= +150) and
/// blends at about 11 per second (measured live). Other native offsets: frontleft 120, front 180,
/// side_left 90, side_right -90, topdown pitch -75.
#[derive(Default)]
pub struct IdleView {
    blend: f32,
    last: Vec3,
}

impl IdleView {
    pub const YAW: f32 = 150. * std::f32::consts::PI / 180.;
    pub fn update(&mut self, direction: Vec3, wanted: bool, dt: f32) -> Vec3 {
        // The native state requires the view angles to be unchanged.
        let turned = self.last.distance(direction) > 1e-3;
        self.last = direction;
        let target = if wanted && !turned { 1. } else { 0. };
        self.blend += (target - self.blend) * (1. - (-11. * dt.clamp(0., 0.1)).exp());
        if self.blend < 1e-3 {
            self.blend = 0.;
            return direction;
        }
        let yaw = direction.y.atan2(direction.x) + Self::YAW * self.blend;
        let pitch = direction.z.clamp(-1., 1.).asin() * (1. - self.blend);
        vec3(yaw.cos() * pitch.cos(), yaw.sin() * pitch.cos(), pitch.sin())
    }
    pub fn reset(&mut self) {
        self.blend = 0.;
    }
}
