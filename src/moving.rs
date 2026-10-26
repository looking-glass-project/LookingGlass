//! Commit a moving support and all its riders together, or leave every rider
//! untouched. The caller temporarily excludes this mover from `world` and only
//! commits the proposed collider/clock when this returns true.
use crate::{
    collision::{Collider, World, PLAYER_CENTER, PLAYER_HALF},
    movement::Player,
    skeletal::Transform,
};
use macroquad::prelude::*;

pub fn aboard(collider: &Collider, feet: Vec3) -> bool {
    let trace = collider.trace(
        feet + PLAYER_CENTER,
        feet + PLAYER_CENTER - Vec3::Z * 3.,
        PLAYER_HALF,
    );
    !trace.start_solid && trace.fraction < 1. && trace.normal.z > 0.65
}

/// A nearby lift is only a support if another surface is not at or above it. At a
/// landing the downwards probe can hit both the floor and the lift underneath;
/// carrying that body would pull it into the floor and stop every other rider.
/// At equal height, a straight vertical departure can pick up the body. A
/// rotating/sideways platform must not recapture someone already on a landing.
pub(crate) fn supported_by(world: &World, collider: &Collider, feet: Vec3, lifting_off: bool) -> bool {
    let support = collider.trace(
        feet + PLAYER_CENTER,
        feet + PLAYER_CENTER - Vec3::Z * 3.,
        PLAYER_HALF,
    );
    if support.start_solid || support.fraction >= 1. || support.normal.z <= 0.65 {
        return false;
    }
    let other = world.body_trace(feet, feet - Vec3::Z * 3.);
    let landing = other.fraction < 1.
        && other.fraction <= support.fraction + 0.001
        && !other.start_solid
        && other.normal.z > 0.65;
    !landing || (lifting_off && (other.fraction - support.fraction).abs() <= 0.001)
}

pub fn carry(
    world: &World,
    previous: &Collider,
    next: &Collider,
    from: Transform,
    to: Transform,
    solid: bool,
    players: &mut [&mut Player],
) -> bool {
    let mut planned = Vec::with_capacity(players.len());
    let lifting_off = to.translation.z > from.translation.z
        && to.translation.truncate() == from.translation.truncate()
        && to.rotation == from.rotation;
    for player in players.iter() {
        let carried = to.translation
            + to.rotation * from.rotation.inverse() * (player.feet - from.translation);
        let riding = solid && player.velocity.z <= 1.
            && supported_by(world, previous, player.feet, lifting_off);
        if riding {
            let Some((feet, normal)) = next.rider_feet(carried) else {
                return false;
            };
            let trace = world.body_trace(player.feet, feet);
            if trace.start_solid || trace.fraction < 1. || !world.body_clear(feet) {
                return false;
            }
            planned.push(Some((feet, normal)));
        } else {
            if solid
                && next.touches(
                    player.feet + PLAYER_CENTER,
                    player.feet + PLAYER_CENTER,
                    PLAYER_HALF,
                )
            {
                return false;
            }
            planned.push(None);
        }
    }
    for (player, plan) in players.iter_mut().zip(planned) {
        if let Some((feet, normal)) = plan {
            player.feet = feet;
            player.grounded = true;
            player.ground_normal = normal;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pose(at: Vec3, yaw: f32) -> Transform {
        Transform {
            translation: at,
            rotation: Quat::from_rotation_z(yaw),
        }
    }
    fn platform(z: f32) -> Collider {
        Collider::fixture(vec3(-100., -100., z - 8.), vec3(100., 100., z))
    }

    #[test]
    fn either_obstructed_rider_stops_both_without_partial_carry() {
        for blocked in 0..2 {
            let mut bodies = [
                Player::new(vec3(-48., 0., 0.04)),
                Player::new(vec3(48., 0., 0.04)),
            ];
            let before = bodies.each_ref().map(|p| p.feet);
            let at = before[blocked];
            let height = PLAYER_CENTER.z + PLAYER_HALF.z;
            let world = World::fixture(&[(
                at + vec3(-20., -20., height + 1.),
                at + vec3(20., 20., height + 17.),
            )]);
            assert!(!carry(
                &world,
                &platform(0.),
                &platform(2.),
                pose(Vec3::ZERO, 0.),
                pose(Vec3::Z * 2., 0.),
                true,
                &mut bodies.each_mut()
            ));
            assert_eq!(bodies.each_ref().map(|p| p.feet), before);
            assert!(carry(
                &World::fixture(&[]),
                &platform(0.),
                &platform(2.),
                pose(Vec3::ZERO, 0.),
                pose(Vec3::Z * 2., 0.),
                true,
                &mut bodies.each_mut()
            ));
            assert!(bodies.iter().all(|p| p.feet.z > 2. && p.grounded));
        }
    }

    #[test]
    fn rotating_support_carries_both_offsets_and_does_not_drag_a_jumper() {
        let world = World::fixture(&[]);
        let mut bodies = [
            Player::new(vec3(-48., 0., 0.04)),
            Player::new(vec3(48., 0., 0.04)),
        ];
        assert!(carry(
            &world,
            &platform(0.),
            &platform(0.),
            pose(Vec3::ZERO, 0.),
            pose(Vec3::ZERO, std::f32::consts::FRAC_PI_2),
            true,
            &mut bodies.each_mut()
        ));
        assert!(bodies[0].feet.distance(vec3(0., -48., 0.04)) < 0.1);
        assert!(bodies[1].feet.distance(vec3(0., 48., 0.04)) < 0.1);
        bodies[1].velocity.z = 300.;
        let jumping = bodies[1].feet;
        assert!(carry(
            &world,
            &platform(0.),
            &platform(-2.),
            pose(Vec3::ZERO, 0.),
            pose(-Vec3::Z * 2., 0.),
            true,
            &mut bodies.each_mut()
        ));
        assert_eq!(bodies[1].feet, jumping);
        assert!(bodies[0].feet.z < -1.9);
    }

    #[test]
    fn partner_under_descending_support_stops_it_before_either_body_changes() {
        for rider in 0..2 {
            let mut bodies = [Player::new(Vec3::ZERO), Player::new(Vec3::ZERO)];
            bodies[rider].feet = vec3(-48., 0., 80.04);
            bodies[1 - rider].feet = vec3(48., 0., 0.04);
            let before = bodies.each_ref().map(|p| p.feet);
            assert!(!carry(
                &World::fixture(&[]),
                &platform(80.),
                &platform(60.),
                pose(Vec3::Z * 80., 0.),
                pose(Vec3::Z * 60., 0.),
                true,
                &mut bodies.each_mut()
            ));
            assert_eq!(bodies.each_ref().map(|p| p.feet), before);
        }
    }

    #[test]
    fn landed_partner_does_not_ride_the_lift_below_the_floor() {
        for landed in 0..2 {
            let world = World::fixture(&[(vec3(20., -100., -8.), vec3(100., 100., 2.))]);
            let mut bodies = [
                Player::new(vec3(-48., 0., 0.04)),
                Player::new(vec3(48., 0., 2.04)),
            ];
            if landed == 0 {
                bodies.reverse();
            }
            let floor_feet = bodies[landed].feet;
            assert!(
                aboard(&platform(0.), floor_feet),
                "Reproduce nearby lift support through floor"
            );
            assert!(carry(
                &world,
                &platform(0.),
                &platform(1.),
                pose(Vec3::ZERO, 0.),
                pose(Vec3::Z, 0.),
                true,
                &mut bodies.each_mut()
            ));
            assert_eq!(bodies[landed].feet, floor_feet);
            assert!(bodies[1 - landed].feet.z > 1. && world.body_clear(bodies[1 - landed].feet));
        }
    }

    #[test]
    fn landed_partner_still_blocks_a_mover_that_would_hit_their_body() {
        for landed in 0..2 {
            let world = World::fixture(&[(vec3(20., -100., -8.), vec3(100., 100., 2.))]);
            let mut bodies = [
                Player::new(vec3(-48., 0., 0.04)),
                Player::new(vec3(48., 0., 2.04)),
            ];
            if landed == 0 {
                bodies.reverse();
            }
            let before = bodies.each_ref().map(|p| p.feet);
            assert!(!carry(
                &world,
                &platform(0.),
                &platform(4.),
                pose(Vec3::ZERO, 0.),
                pose(Vec3::Z * 4., 0.),
                true,
                &mut bodies.each_mut()
            ));
            assert_eq!(bodies.each_ref().map(|p| p.feet), before);
        }
    }

    #[test]
    fn rising_lift_takes_riders_off_a_coplanar_landing() {
        let world = World::fixture(&[(vec3(-100., -100., -8.), vec3(100., 100., 0.))]);
        let mut bodies = [Player::new(vec3(-48., 0., 0.04)), Player::new(vec3(48., 0., 0.04))];
        assert!(carry(&world, &platform(0.), &platform(1.), pose(Vec3::ZERO, 0.),
            pose(Vec3::Z, 0.), true, &mut bodies.each_mut()),
            "A rising lift at the same height as its landing must pick up both riders");
        assert!(bodies.iter().all(|body| body.feet.z > 1. && world.body_clear(body.feet)));
    }

    #[test]
    fn nearly_arrived_rotating_lift_does_not_recapture_a_landed_body() {
        let world = World::fixture(&[(vec3(20., -100., -8.), vec3(100., 100., 2.))]);
        let mut bodies = [Player::new(vec3(-48., 0., 2.038)), Player::new(vec3(48., 0., 2.04))];
        let landed = bodies[1].feet;
        assert!(carry(&world, &platform(1.998), &platform(2.), pose(Vec3::Z * 1.998, 0.),
            pose(Vec3::Z * 2., 0.05), true, &mut bodies.each_mut()));
        assert_eq!(bodies[1].feet, landed);
        assert!(bodies[0].feet.y.abs() > 1. && world.body_clear(bodies[0].feet));
    }

    #[test]
    fn floor_keeps_precedence_when_lift_reaches_the_same_height() {
        let world = World::fixture(&[(vec3(20., -100., -8.), vec3(100., 100., 2.))]);
        let mut bodies = [
            Player::new(vec3(-48., 0., 2.04)),
            Player::new(vec3(48., 0., 2.04)),
        ];
        let floor_feet = bodies[1].feet;
        assert!(carry(
            &world,
            &platform(2.),
            &platform(2.),
            pose(Vec3::Z * 2., 0.),
            pose(Vec3::Z * 2., 0.05),
            true,
            &mut bodies.each_mut()
        ));
        assert_eq!(bodies[1].feet, floor_feet);
        assert!(bodies[0].feet.y.abs() > 1.);
    }
}
