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

fn separate_rope(world: &World, body: &mut Body, outward: Vec3, distance: f32) {
    let p = &mut body.player;
    let Some(grip) = &p.rope else { return; };
    let Some(rope) = world.traversal.rope(grip.id) else { return; };
    let center = rope.anchor - Vec3::Z * 40.;
    let radial = p.feet - center;
    let height = radial.dot(outward) + distance;
    if height.abs() > grip.length { return; }
    // Intersect the pendulum sphere with the contact plane. Normalising a
    // horizontal push back onto the sphere would shrink it and retain overlap.
    let tangent = (radial - outward * radial.dot(outward))
        .try_normalize().unwrap_or(-Vec3::Z);
    let candidate = center + outward * height
        + tangent * (grip.length * grip.length - height * height).max(0.).sqrt();
    let trace = world.body_trace(p.feet, candidate);
    if trace.start_solid || trace.fraction < 1. || !world.body_clear(candidate) { return; }
    p.feet = candidate;
    let radial = (candidate-center).normalize_or_zero();
    p.velocity = crate::movement::constrain(p.velocity, &[outward, radial, -radial]);
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

fn contact_position(world: &World, body: &Body, to: Vec3) -> Option<Vec3> {
    if clear(world, body, to) { return Some(to); }
    let p = &body.player;
    if !p.grounded || p.swimming { return None; }
    let trace = world.body_trace(p.feet, to);
    if trace.start_solid || trace.fraction >= 1. || trace.normal.z < 0.65 { return None; }
    // A tangent sweep can touch the resting floor at fraction zero after
    // rounding at native map coordinates. Give the destination one skin of
    // clearance, then recheck the entire path, body and centre support.
    let raised = to + Vec3::Z * (crate::collision::SKIN / trace.normal.z);
    clear(world, body, raised).then_some(raised)
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
    let (enter, leave) = if a < 0.000001 {
        if c >= 0. { return; }
        (0., 1.)
    } else {
        let discriminant = b*b - a*c;
        if discriminant < 0. { return; }
        (((-b-discriminant.sqrt())/a).max(0.),
         ((-b+discriminant.sqrt())/a).min(1.))
    };
    // Vertical climbing can enter body contact after the horizontal circles
    // already overlap. Intersect both time intervals instead of testing z only
    // at the first horizontal contact (which would miss that entire tick).
    let half_height = PLAYER_HALF.z * 2.;
    let (below, above) = if delta.z.abs() < 0.000001 {
        if start.z.abs() >= half_height { return; }
        (0., 1.)
    } else {
        let t0 = (-half_height-start.z)/delta.z;
        let t1 = (half_height-start.z)/delta.z;
        (t0.min(t1).max(0.), t0.max(t1).min(1.))
    };
    let contact = enter.max(below);
    if contact >= leave.min(above) { return; }
    let normal = (xy + travel * contact).try_normalize()
        .or_else(|| xy.try_normalize()).unwrap_or(Vec2::X);
    let penetration = DISTANCE - finish.truncate().dot(normal);
    if penetration <= 0. { return; }
    let motion = [end[0]-previous[0], end[1]-previous[1]];
    let mut approach = [(-motion[0].truncate().dot(normal)).max(0.),
                    motion[1].truncate().dot(normal).max(0.)];
    if approach[0]+approach[1] <= 0.0001 && bodies.iter().any(|b|b.player.rope.is_some()) {
        let vertical = start.z.signum();
        approach = [(-motion[0].z*vertical).max(0.), (motion[1].z*vertical).max(0.)];
    }
    let sum = approach[0] + approach[1];
    let weights = if sum > 0.0001 { [approach[0]/sum, approach[1]/sum] } else { [0.5; 2] };
    for index in 0..2 {
        if weights[index] == 0. { continue; }
        let outward = if index == 0 { normal } else { -normal };
        if bodies[index].player.rope.is_some() {
            separate_rope(world, &mut bodies[index], outward.extend(0.), penetration*weights[index]);
            continue;
        }
        if !movable(&bodies[index]) { continue; }
        let mut along = outward.extend(0.);
        let p = &bodies[index].player;
        if p.grounded && p.ground_normal.z >= 0.65 {
            along.z = -outward.dot(p.ground_normal.truncate()) / p.ground_normal.z;
        }
        let candidate = end[index] + along * penetration * weights[index];
        if let Some(candidate) = contact_position(world, &bodies[index], candidate) {
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
    fn rope_body(feet: Vec3, length: f32, velocity: Vec3) -> Body {
        let mut player = Player::new(feet);
        player.rope = Some(crate::movement::RopeGrip { id:crate::entity::Id(353), length, rise:0. });
        player.script_motion = 1;
        player.velocity = velocity;
        Body { player, stats:Stats::default(), yaw:0., pitch:0., running:false, spawned_at:0, warped_at:0 }
    }
    fn rope_world(anchor: Vec3, length: f32) -> World {
        let mut world = World::fixture(&[]);
        world.traversal.ropes.push(crate::traversal::Rope {
            id:crate::entity::Id(353),model:1,enabled:true,origin:anchor,anchor,length,
        });
        world
    }
    fn separate(bodies: &[Body; 2]) -> bool {
        let d = bodies[0].player.feet - bodies[1].player.feet;
        d.truncate().length() >= DISTANCE - 0.05 || d.z.abs() >= PLAYER_HALF.z * 2. - 0.05
    }
    #[test]
    fn recorded_same_rope_contact_stays_separate_and_preserves_valid_grips() {
        let world = rope_world(vec3(1644.,-1912.,-64.),632.);
        let mut bodies = [
            rope_body(vec3(1750.547363,-1963.254517,-671.413452),579.601013,
                vec3(153.596725,-58.847328,34.157661)),
            rope_body(vec3(1756.052734,-1962.931396,-651.688538),561.348877,
                vec3(244.322754,-110.158646,60.230522)),
        ];
        bodies[1].stats.spend_will(17.);
        let resources = bodies.each_ref().map(|b| serde_json::to_value(&b.stats).unwrap());
        for tick in 0..240 {
            let previous = bodies.each_ref().map(|b| b.player.feet);
            if tick > 0 {
                for (i, b) in bodies.iter_mut().enumerate() {
                    b.player.tick(&world, Controls {rise:if i==0 {1.} else {-1.},
                        wish:if i==0 {Vec2::X} else {-Vec2::X},..Default::default()});
                }
            }
            resolve(&world,&mut bodies,previous);
            assert!(separate(&bodies), "Rope overlap at tick{tick}: {:?}", bodies.each_ref().map(|b|b.player.feet));
            for (i,b) in bodies.iter().enumerate() {
                b.player.validate_save().unwrap();
                b.player.validate_world(&world).unwrap();
                assert_eq!(b.player.rope.as_ref().unwrap().id,crate::entity::Id(353));
                assert_eq!(serde_json::to_value(&b.stats).unwrap(),resources[i]);
            }
        }
    }
    #[test]
    fn rope_contact_preserves_a_stationary_partner_and_short_grip_bounds() {
        for length in [32.,400.] {
            let world = rope_world(Vec3::Z*600.,500.);
            let on_sphere = |x:f32| vec3(x,0.,560.-(length*length-x*x).sqrt());
            for moving in 0..2 {
                let mut bodies = [rope_body(on_sphere(-18.),length,Vec3::ZERO),
                    rope_body(on_sphere(18.),length,Vec3::ZERO)];
                let previous = bodies.each_ref().map(|b|b.player.feet);
                let still = serde_json::to_value(&bodies[1-moving]).unwrap();
                bodies[moving].player.feet = on_sphere(if moving==0 {-8.} else {8.});
                bodies[moving].player.velocity = if moving==0 {Vec3::X*100.} else {-Vec3::X*100.};
                resolve(&world,&mut bodies,previous);
                assert!(separate(&bodies));
                assert_eq!(serde_json::to_value(&bodies[1-moving]).unwrap(),still);
                for b in &bodies {
                    b.player.validate_save().unwrap();
                    b.player.validate_world(&world).unwrap();
                    assert_eq!(b.player.rope.as_ref().unwrap().length,length);
                }
            }
        }
    }
    #[test]
    fn vertical_rope_contact_moves_only_the_climber() {
        let world = rope_world(Vec3::Z*600.,500.);
        for moving in 0..2 {
            for above in [false,true] {
                let lengths = if above { [300.,360.] } else { [360.,300.] };
                let mut bodies = std::array::from_fn(|i| {
                    let length = lengths[usize::from(i!=moving)];
                    rope_body(Vec3::Z*(560.-length),length,Vec3::ZERO)
                });
                let previous = bodies.each_ref().map(|b|b.player.feet);
                let still = serde_json::to_value(&bodies[1-moving]).unwrap();
                let change = if above {5.} else {-5.};
                bodies[moving].player.rope.as_mut().unwrap().length += change;
                bodies[moving].player.feet.z -= change;
                resolve(&world,&mut bodies,previous);
                assert!(separate(&bodies),"Vertical contact: moving{moving}, above{above}");
                assert_eq!(serde_json::to_value(&bodies[1-moving]).unwrap(),still);
                for b in &bodies { b.player.validate_world(&world).unwrap(); }
            }
        }
    }
    #[test]
    fn blocked_rope_contact_does_not_push_into_a_wall_or_detach() {
        let mut world = World::fixture(&[(vec3(-500.,-500.,0.),vec3(-32.,500.,600.))]);
        world.traversal = rope_world(Vec3::Z*600.,500.).traversal;
        let on_sphere = |x:f32| vec3(x,0.,560.-(400.*400.-x*x).sqrt());
        let mut bodies = [rope_body(on_sphere(-18.),400.,Vec3::ZERO),
            rope_body(on_sphere(18.),400.,Vec3::ZERO)];
        let previous = bodies.each_ref().map(|b|b.player.feet);
        bodies[0].player.feet = on_sphere(-8.);
        let before = serde_json::to_value(&bodies).unwrap();
        resolve(&world,&mut bodies,previous);
        assert_eq!(serde_json::to_value(&bodies).unwrap(),before);
        for b in &bodies {
            b.player.validate_world(&world).unwrap();
            assert!(world.body_clear(b.player.feet));
        }
    }
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
