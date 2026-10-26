//! Host-owned personal space. An approaching player yields at contact; their
//! stationary partner is never shoved into a wall or off a platform.
use super::world::Body;
use crate::collision::{World, PLAYER_HALF};
use macroquad::prelude::*;

pub const DISTANCE: f32 = 36.;

fn movable(body: &Body) -> bool {
    let p = &body.player;
    p.rope.is_none() && p.ledge.is_none() && !p.climbing() && p.script_motion == 0
}

fn clear(world: &World, body: &Body, to: Vec3) -> bool {
    let p = &body.player;
    let trace = world.body_trace(p.feet, to);
    if trace.start_solid || trace.fraction < 1. || !world.body_clear(to) { return false; }
    // A contact correction must not create a fall or take a swimmer onto shore.
    if p.swimming {
        let water = crate::water::Immersion::sample(world, to);
        return water.kind == p.immersion.kind && water.level >= 2;
    }
    if p.grounded {
        let ground = world.body_trace(to, to - Vec3::Z * 2.);
        if ground.start_solid || ground.fraction >= 1. || ground.normal.z < 0.65 { return false; }
        // On a slope the uphill edge of the box rests above the ground under
        // its centre. Account for that footprint without accepting a centre
        // beyond the support's edge. Flat supports retain the two-unit probe.
        let slope_depth = ground.normal.truncate().abs().dot(PLAYER_HALF.truncate()) / ground.normal.z;
        let support = world.sweep(to + Vec3::Z, to - Vec3::Z * (2.+slope_depth), Vec3::ZERO);
        if support.start_solid || support.fraction >= 1. || support.normal.z < 0.65 { return false; }
    }
    true
}

pub fn resolve(world: &World, bodies: &mut [Body; 2], previous: [Vec3; 2]) {
    if bodies.iter().any(|b| !b.stats.alive()) { return; }
    let end = bodies.each_ref().map(|b| b.player.feet);
    let start = previous[0] - previous[1];
    let finish = end[0] - end[1];
    let delta = finish - start;
    let xy = start.truncate();
    let travel = delta.truncate();
    // Continuous contact prevents even fast opposing runs/knockback swapping
    // sides in one tick. Players on different floors remain independent.
    let a = travel.length_squared();
    let b = xy.dot(travel);
    let c = xy.length_squared() - DISTANCE * DISTANCE;
    let contact = if c < 0. { 0. } else {
        let discriminant = b*b - a*c;
        if a < 0.000001 || b >= 0. || discriminant < 0. { return; }
        let time = (-b-discriminant.sqrt())/a;
        if !(0. ..=1.).contains(&time) { return; }
        time
    };
    let height = start.z + delta.z * contact;
    if height.abs() >= PLAYER_HALF.z * 2. { return; }
    let normal = (xy + travel * contact).try_normalize()
        .or_else(|| xy.try_normalize()).unwrap_or(Vec2::X);
    let penetration = DISTANCE - finish.truncate().dot(normal);
    if penetration <= 0. { return; }
    let motion = [end[0]-previous[0], end[1]-previous[1]];
    let approach = [(-motion[0].truncate().dot(normal)).max(0.),
                    motion[1].truncate().dot(normal).max(0.)];
    let sum = approach[0] + approach[1];
    let weights = if sum > 0.0001 { [approach[0]/sum, approach[1]/sum] } else { [0.5; 2] };
    for index in 0..2 {
        if !movable(&bodies[index]) || weights[index] == 0. { continue; }
        let outward = if index == 0 { normal } else { -normal };
        let mut along = outward.extend(0.);
        let p = &bodies[index].player;
        if p.grounded && p.ground_normal.z >= 0.65 {
            along.z = -outward.dot(p.ground_normal.truncate()) / p.ground_normal.z;
        }
        let candidate = end[index] + along * penetration * weights[index];
        if clear(world, &bodies[index], candidate) {
            let p = &mut bodies[index].player;
            p.feet = candidate;
            let inward = p.velocity.truncate().dot(outward).min(0.);
            // Cancel the approach along the same surface. Keeping its upward
            // component after stopping uphill motion would launch Alice.
            p.velocity -= along * inward;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Stats, movement::{Controls, Player, EYE_HEIGHT}, coop::world::Roster};
    fn floor() -> World {
        World::fixture(&[(vec3(-500.,-500.,-100.),vec3(500.,500.,0.))])
    }
    fn pair(world: &World, feet: [Vec3; 2]) -> Roster {
        let mut r = Roster::new(world, feet[0]+Vec3::Z*EYE_HEIGHT,0.,Stats::default()).unwrap();
        for (body,at) in r.players.iter_mut().zip(feet) { body.player = Player::spawn(world,at+Vec3::Z*EYE_HEIGHT).unwrap(); }
        r
    }
    #[test]
    fn approaching_either_player_stops_without_pushing_the_partner_and_can_slide_away() {
        let world = floor();
        for moving in 0..2 {
            let mut r = pair(&world,[vec3(-80.,0.,0.04),vec3(80.,0.,0.04)]);
            let stationary = r.players[1-moving].player.feet;
            let direction = if moving == 0 { Vec2::X } else { -Vec2::X };
            let mut input = [(Controls::default(),Default::default());2];
            input[moving].0 = Controls { wish:direction,run:true,..Default::default() };
            for _ in 0..240 { r.move_players(&world,input); }
            assert!(r.players[1-moving].player.feet.distance(stationary)<0.1);
            assert!((r.players[0].player.feet.distance(r.players[1].player.feet)-DISTANCE).abs()<0.1);
            input[moving].0.wish = (direction+Vec2::Y).normalize();
            for _ in 0..60 { r.move_players(&world,input); }
            assert!(r.players[moving].player.feet.y>80.);
        }
    }
    #[test]
    fn approaching_on_a_slope_preserves_personal_space_and_the_stationary_partner() {
        let world=World::ramp_fixture();
        for moving in 0..2 {
            // The box rests on the uphill edge: z = x/2 + half.x/2 + skin.
            let mut r=pair(&world,[vec3(-60.,0.,-22.46),vec3(60.,0.,37.54)]);
            let still=r.players[1-moving].player.feet;
            let mut input=[(Controls::default(),Default::default());2];
            input[moving].0=Controls {wish:if moving==0 {Vec2::X}else{-Vec2::X},run:true,..Default::default()};
            for _ in 0..240 {
                r.move_players(&world,input);
                let spacing=(r.players[0].player.feet-r.players[1].player.feet).truncate().length();
                assert!(spacing>=DISTANCE-0.1,"Slope overlap moving{moving}: spacing{spacing}, feet{:?}",r.players.each_ref().map(|b|b.player.feet));
                assert!(r.players.iter().all(|b|world.body_clear(b.player.feet)
                    && world.body_trace(b.player.feet,b.player.feet-Vec3::Z*2.).fraction<1.),
                    "Slope contact lost support: {:?}",r.players.each_ref().map(|b|b.player.feet));
                assert!(r.players[1-moving].player.feet.distance(still)<0.1);
            }
        }
    }
    #[test]
    fn opposing_motion_and_fast_crossing_keep_the_original_sides() {
        let world = floor();
        let mut r = pair(&world,[vec3(-40.,0.,0.04),vec3(40.,0.,0.04)]);
        let old = r.players.each_ref().map(|b|b.player.feet);
        r.players[0].player.feet.x = 50.; r.players[1].player.feet.x = -50.;
        resolve(&world,&mut r.players,old);
        assert!((r.players[0].player.feet.x+18.).abs()<0.01);
        assert!((r.players[1].player.feet.x-18.).abs()<0.01);
    }
    #[test]
    fn existing_overlap_separates_but_different_floors_and_dead_bodies_do_not_block() {
        let world = floor();
        let mut r = pair(&world,[Vec3::Z*0.04;2]);
        let old = r.players.each_ref().map(|b|b.player.feet);
        resolve(&world,&mut r.players,old);
        assert!((r.players[0].player.feet.distance(r.players[1].player.feet)-DISTANCE).abs()<0.01);
        r.players[1].player.feet = r.players[0].player.feet + Vec3::Z*60.;
        let old = r.players.each_ref().map(|b|b.player.feet);
        resolve(&world,&mut r.players,old);
        assert_eq!(old,r.players.each_ref().map(|b|b.player.feet));
        r.players[1].player.feet = r.players[0].player.feet;
        r.players[1].stats.damage(10000.);
        let old = r.players.each_ref().map(|b|b.player.feet);
        resolve(&world,&mut r.players,old);
        assert_eq!(old,r.players.each_ref().map(|b|b.player.feet));
    }
    #[test]
    fn overlap_near_wall_or_edge_never_moves_a_body_into_solid_or_off_support() {
        for world in [
            World::fixture(&[(vec3(-500.,-500.,-100.),vec3(500.,500.,0.)),
                            (vec3(16.,-500.,0.),vec3(100.,500.,200.))]),
            World::fixture(&[(vec3(-16.,-500.,-100.),vec3(16.,500.,0.))]),
        ] {
            let mut bodies = std::array::from_fn(|_| Body { player:Player::spawn(&world,Vec3::Z*(EYE_HEIGHT+0.04)).unwrap(),
                stats:Stats::default(),yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0 });
            let old = bodies.each_ref().map(|b|b.player.feet);
            resolve(&world,&mut bodies,old);
            for body in &bodies { assert!(clear(&world,body,body.player.feet)); }
        }
    }
}
