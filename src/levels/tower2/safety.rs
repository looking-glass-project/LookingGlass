//! Placement must remain outside a fan's complete rotation, even during Watch.
//! This conservative cylinder is not used for movement collision or damage.
use super::*;
use crate::collision::{PLAYER_CENTER, PLAYER_HALF, SKIN};

pub(super) struct FanSweep {
    base: Vec3,
    axis: Vec3,
    low: f32,
    high: f32,
    radius_squared: f32,
}
impl FanSweep {
    fn new(base: Vec3, axis: Vec3, min: Vec3, max: Vec3) -> Self {
        // Inline BSP bounds are local to the entity origin. Rotation preserves
        // axial extent; the farthest radial corner bounds every blade angle.
        let radial = Vec3::ONE - axis;
        let extent = min.abs().max(max.abs()) * radial;
        Self { base, axis, low: min.dot(axis), high: max.dot(axis),
            radius_squared: extent.length_squared() }
    }
    pub(super) fn contains(&self, feet: Vec3) -> bool {
        let center = feet + PLAYER_CENTER - self.base;
        let half = PLAYER_HALF + Vec3::splat(SKIN);
        let axial = center.dot(self.axis);
        if axial + half.dot(self.axis) < self.low || axial - half.dot(self.axis) > self.high {
            return false;
        }
        // Distance from the rotation axis to the nearest point on the full
        // player box, rather than only to Alice's feet or centre.
        let nearest = (center.abs() - half).max(Vec3::ZERO) * (Vec3::ONE - self.axis);
        nearest.length_squared() <= self.radius_squared
    }
}
pub(super) fn load(map: &Bsp, objects: &[Object]) -> Vec<FanSweep> {
    objects.iter().filter_map(|o| {
        let i = motion::index(&o.name, "fan")?;
        let model = &map.models[o.model];
        Some(FanSweep::new(o.base, motion::axis(i), model.min, model.max))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_body_boundaries_and_every_angle_for_all_axes() {
        let base = vec3(120., -230., 54.);
        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
            let radial = Vec3::ONE - axis;
            let min = -axis * 4. - radial * 80.;
            let max = axis * 9. + radial * 120.;
            let sweep = FanSweep::new(base, axis, min, max);
            for angle in 0..360 {
                let rotation = Quat::from_axis_angle(axis, (angle as f32).to_radians());
                for x in [min.x, max.x] { for y in [min.y, max.y] { for z in [min.z, max.z] {
                    let center = base + rotation * vec3(x, y, z);
                    assert!(sweep.contains(center - PLAYER_CENTER));
                }}}
            }
            let radius = sweep.radius_squared.sqrt();
            let direction = if axis == Vec3::X { Vec3::Y } else { Vec3::X };
            let half = PLAYER_HALF.dot(direction);
            assert!(sweep.contains(base + direction * (radius + half) - PLAYER_CENTER));
            assert!(!sweep.contains(base + direction * (radius + half + 1.) - PLAYER_CENTER));
            for (edge, sign) in [(sweep.low, -1.), (sweep.high, 1.)] {
                let at = base + axis * (edge + sign * PLAYER_HALF.dot(axis)) - PLAYER_CENTER;
                assert!(sweep.contains(at));
                assert!(!sweep.contains(at + axis * sign));
            }
        }
    }
}
