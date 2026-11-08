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
        if support.start_solid || support.fraction >= 1. || support.normal.z < 0.65 {
            // Ordinary movement can already be resting on the edge of its
            // footprint, with no floor directly under the centre. Do not make
            // such a body uncorrectable, or contact accumulates every tick.
            // Retain the centre rule when it was supported before correction.
            let old_ground=world.body_trace(p.feet,p.feet-Vec3::Z*2.);
            if old_ground.start_solid || old_ground.fraction>=1. || old_ground.normal.z<0.65 {return false;}
            let old_depth=old_ground.normal.truncate().abs().dot(PLAYER_HALF.truncate())/old_ground.normal.z;
            let old_centre=world.sweep(p.feet+Vec3::Z,p.feet-Vec3::Z*(2.+old_depth),Vec3::ZERO);
            if old_centre.start_solid || old_centre.fraction<1. && old_centre.normal.z>=0.65 {return false;}
            // The intersection of the old and new footprints must itself rest
            // on walkable floor. This cannot transfer support across a gap or
            // move the body beyond the edge that was holding it up.
            let half=PLAYER_HALF.truncate()-(to-p.feet).truncate().abs()*0.5;
            if half.min_element()<=0. {return false;}
            let centre=((to+p.feet).truncate()*0.5).extend(to.z)+crate::collision::PLAYER_CENTER;
            let shared=world.sweep(centre,centre-Vec3::Z*2.,half.extend(PLAYER_HALF.z));
            if shared.start_solid || shared.fraction>=1. || shared.normal.z<0.65 {return false;}
        }
    }
    true
}

fn contact_position(world: &World, body: &Body, to: Vec3) -> Option<Vec3> {
    if clear(world, body, to) { return Some(to); }
    let p = &body.player;
    let trace = world.body_trace(p.feet, to);
    if trace.start_solid || trace.fraction >= 1. { return None; }
    // A tangent sweep can touch the resting floor at fraction zero after
    // rounding at native map coordinates. Give the destination one skin of
    // clearance, then recheck the entire path, body and centre support.
    let offset = if p.grounded && !p.swimming && trace.normal.z >= 0.65 {
        Vec3::Z * (crate::collision::SKIN / trace.normal.z)
    } else if trace.normal.z.abs() < 0.65 {
        // Rotated doors have the same rounding at a glancing wall contact.
        // Move outward by one skin, retaining the full swept/body/support tests.
        trace.normal * crate::collision::SKIN
    } else {return None;};
    let candidate=to+offset;
    clear(world, body, candidate).then_some(candidate)
}

fn previous_footing(world: &World, body: &Body, previous: Vec3) -> Option<Vec3> {
    let p=&body.player;
    if !p.grounded || p.swimming {return None;}
    // A tick can move from footprint-only support onto centre support just as
    // another player blocks it. Returning to that actual previous position is
    // not creating a new overhang. Only the final retreat may use this exception;
    // normal separation and intermediate candidates retain the centre check.
    let trace=world.body_trace(p.feet,previous);
    let ground=world.body_trace(previous,previous-Vec3::Z*2.);
    (!trace.start_solid && trace.fraction>=1. && world.body_clear(previous)
        && !ground.start_solid && ground.fraction<1. && ground.normal.z>=0.65)
        .then_some(previous)
}

fn contact_correction(world: &World, body: &Body, along: Vec3, outward: Vec2, amount: f32) -> Option<Vec3> {
    let p=&body.player;
    let enough=|to:Vec3| (to-p.feet).truncate().dot(outward)>=amount-0.0001;
    let to=p.feet+along*amount;
    if let Some(at)=contact_position(world,body,to).filter(|at|enough(*at)) {return Some(at);}
    let wall=world.body_trace(p.feet,to);
    if wall.start_solid || wall.fraction>=1. || wall.normal.z.abs()>=0.65 {return None;}
    // A bevel can lift a walking body by a fraction of a unit. Returning to
    // its last position then hits that same wall's collision skin; nudging out
    // of the wall also nudges into the partner. Try the wall tangent instead.
    // Keep the existing ground plane, or horizontal height while airborne,
    // and retain the full swept clearance, support and immersion checks.
    let plane=if p.grounded && p.ground_normal.z>=0.65 {p.ground_normal} else {Vec3::Z};
    let tangent=crate::movement::constrain(along,&[wall.normal,plane,-plane]);
    let projection=tangent.truncate().dot(outward);
    if projection<0.25 {return None;}
    contact_position(world,body,p.feet+tangent*(amount/projection)).filter(|at|enough(*at))
}

fn contact(start: Vec3, finish: Vec3) -> Option<(f32, Vec3)> {
    let delta = finish - start;
    let xy = start.truncate();
    let travel = delta.truncate();
    // Continuous contact prevents even fast opposing runs/knockback swapping
    // sides in one tick. Players on different floors remain independent.
    let a = travel.length_squared();
    let b = xy.dot(travel);
    let c = xy.length_squared() - DISTANCE * DISTANCE;
    let (enter, leave) = if a < 0.000001 {
        if c >= 0. { return None; }
        (0., 1.)
    } else {
        let discriminant = b*b - a*c;
        if discriminant < 0. { return None; }
        (((-b-discriminant.sqrt())/a).max(0.),
         ((-b+discriminant.sqrt())/a).min(1.))
    };
    // Vertical climbing can enter body contact after the horizontal circles
    // already overlap. Intersect both time intervals instead of testing z only
    // at the first horizontal contact (which would miss that entire tick).
    let half_height = PLAYER_HALF.z * 2.;
    let (below, above) = if delta.z.abs() < 0.000001 {
        if start.z.abs() >= half_height { return None; }
        (0., 1.)
    } else {
        let t0 = (-half_height-start.z)/delta.z;
        let t1 = (half_height-start.z)/delta.z;
        (t0.min(t1).max(0.), t0.max(t1).min(1.))
    };
    let contact = enter.max(below);
    if contact >= leave.min(above) { return None; }
    let normal = (xy + travel * contact).try_normalize()
        .or_else(|| xy.try_normalize()).unwrap_or(Vec2::X);
    let vertical = below >= enter && start.z.abs() >= half_height;
    Some((contact, if vertical { Vec3::Z * start.z.signum() } else { normal.extend(0.) }))
}

/// A checked pull-up cannot be pushed sideways off its authored path. Pause
/// only the approaching climber, including its path clock, until room clears.
pub fn wait_for_climbers(world: &World, bodies: &mut [Body; 2], previous: &[crate::movement::Player; 2]) {
    if bodies.iter().any(|b| !b.stats.alive()) { return; }
    let start = previous[0].feet - previous[1].feet;
    for _ in 0..2 {
        let end = bodies.each_ref().map(|b| b.player.feet);
        let Some((_, normal)) = contact(start, end[0] - end[1]) else { break; };
        let mut changed = false;
        let mut waited = [false; 2];
        for index in 0..2 {
            let p = &mut bodies[index].player;
            let old = &previous[index];
            let guided = p.ledge.is_some() || p.climbing()
                || p.grounded && (old.ledge.is_some() || old.climbing());
            let outward = if index == 0 { normal } else { -normal };
            if !guided || (p.feet - old.feet).dot(outward) >= -0.000001 { continue; }
            let trace = world.body_trace(p.feet, old.feet);
            if trace.start_solid || trace.fraction < 1. || !world.body_clear(old.feet) { continue; }
            p.wait_at_climb_contact(old);
            waited[index] = true;
            changed = true;
        }
        if waited == [true; 2] && bodies.iter().all(|b| b.player.climbing()) {
            // Two converging low pull-ups otherwise pause each other's clocks
            // forever, ignoring movement input. Release one at its checked
            // current position so ordinary gravity, steering and separation
            // can clear the bank. Do not displace either body or a waiting partner.
            bodies[1].player.cancel_climb();
        }
        if !changed { break; }
    }
}

pub fn resolve(world: &World, bodies: &mut [Body; 2], previous: [Vec3; 2]) {
    if bodies.iter().any(|b| !b.stats.alive()) { return; }
    let end = bodies.each_ref().map(|b| b.player.feet);
    let start = previous[0] - previous[1];
    let finish = end[0] - end[1];
    let Some((contact, contact_normal)) = contact(start, finish) else { return; };
    let xy = start.truncate();
    let travel = (finish - start).truncate();
    let normal = (xy + travel * contact).try_normalize()
        .or_else(|| xy.try_normalize()).unwrap_or(Vec2::X);
    let penetration = DISTANCE - finish.truncate().dot(normal);
    if penetration <= 0. { return; }
    let motion = [end[0]-previous[0], end[1]-previous[1]];
    let mut approach = [(-motion[0].truncate().dot(normal)).max(0.),
                    motion[1].truncate().dot(normal).max(0.)];
    // A descending/jumping body can enter vertically while its horizontal
    // motion is stationary or already moving away. Only that approaching body
    // yields; splitting the correction would shove the stationary partner.
    if contact_normal.z != 0.
        || approach[0]+approach[1] <= 0.0001 && bodies.iter().any(|b|b.player.rope.is_some()) {
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
        if let Some(candidate) = contact_correction(world, &bodies[index], along, outward, penetration * weights[index]) {
            let p = &mut bodies[index].player;
            p.feet = candidate;
            let inward = p.velocity.truncate().dot(outward).min(0.);
            // Cancel the approach along the same surface. Keeping its upward
            // component after stopping uphill motion would launch Alice.
            p.velocity -= along * inward;
        }
    }
    if (bodies[0].player.feet-bodies[1].player.feet).truncate().dot(normal) >= DISTANCE-0.001 {
        return;
    }
    // Sliding along a wall can make the normal correction point into that
    // wall. Retreat approaching bodies along their actual movement instead.
    // Check the pair before applying either result; a stationary or departing
    // partner retains its position, and no correction may lose floor support.
    for fraction in [(contact-0.001).max(0.),0.] {
        let mut candidates=bodies.each_ref().map(|b|b.player.feet);
        let mut valid=true;
        for i in 0..2 {
            if approach[i]<=0. {continue;}
            if !movable(&bodies[i]) {valid=false;break;}
            let to=previous[i].lerp(end[i],fraction);
            // Side contact only blocks horizontal approach. Rewinding an
            // airborne body's height every tick can suspend it beside a wall
            // while gravity keeps accumulating. Preserve its vertical travel
            // when the swept world and final pair clearance allow it.
            let p=&bodies[i].player;
            let position=(contact_normal.z==0. && !p.grounded && !p.swimming)
                .then(||contact_position(world,&bodies[i],vec3(to.x,to.y,p.feet.z))).flatten()
                .or_else(||contact_position(world,&bodies[i],to)).or_else(|| {
                let p=&bodies[i].player;
                // A just-landed body can have an airborne previous position.
                // Keep its landing height when retreating horizontally: the
                // old height may no longer pass the grounded support check.
                // The swept path, floor support and pair separation still
                // have to pass before either body's position changes.
                (p.grounded && !p.swimming && to.z>p.feet.z)
                    .then(||contact_position(world,&bodies[i],vec3(to.x,to.y,p.feet.z))).flatten()
            }).or_else(|| (fraction==0.).then(||previous_footing(world,&bodies[i],previous[i])).flatten());
            if let Some(at)=position {candidates[i]=at;}
            else {valid=false;break;}
        }
        let separated=candidates[0]-candidates[1];
        let vertical_stop=contact_normal.z!=0. && separated.z*start.z.signum()>=PLAYER_HALF.z*2.;
        if !valid || !vertical_stop && separated.truncate().dot(normal)<DISTANCE-0.001 {continue;}
        for i in 0..2 {
            if approach[i]<=0. {continue;}
            let outward=if i==0 {normal}else{-normal};
            let p=&mut bodies[i].player;
            p.feet=candidates[i];
            if vertical_stop {
                // A narrow passage may have no safe sideways correction. Stop
                // at the original vertical contact instead of falling through
                // the partner. Record the falling player's impact, without
                // repeating landing events for one-tick gravity at resting contact.
                let up=if i==0 {start.z.signum()}else{-start.z.signum()};
                let inward=(p.velocity.z*up).min(0.);
                let impact=up>0. && inward < -crate::movement::GRAVITY*crate::movement::FIXED_DT*2. && !p.grounded;
                if impact {
                    p.landings+=1;
                    p.landing_speed=-p.velocity.z*if p.immersion.level>0 {0.5}else{1.};
                }
                let landing=impact.then_some(p.landing_speed);
                p.velocity.z-=up*inward;
                if let Some(speed)=landing {bodies[i].stats.fall(speed);}
                continue;
            }
            let mut along=outward.extend(0.);
            if p.grounded && p.ground_normal.z>=0.65 {
                along.z=-outward.dot(p.ground_normal.truncate())/p.ground_normal.z;
            }
            p.velocity-=along*p.velocity.truncate().dot(outward).min(0.);
        }
        break;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Stats, movement::{Controls, Player, EYE_HEIGHT, FIXED_DT}, coop::world::Roster};
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
    fn following_pull_up_waits_then_finishes_in_either_player_slot() {
        let world = World::fixture(&[
            (vec3(-3200.,3900.,2700.),vec3(-2000.,4300.,2800.)),
            (vec3(-2820.,3900.,2800.),vec3(-2000.,4300.,2960.)),
        ]);
        for leader in 0..2 { for touching in [false,true] {
            let mut r = pair(&world,[vec3(-3000.,4000.,2800.04),vec3(-3100.,4000.,2800.04)]);
            for i in 0..2 {
                let leading = i == leader;
                let y = if leading {4083.7578125} else {4073.809814453125};
                let feet = if leading {vec3(if touching {-2808.942626953125} else {-2809.146728515625},y,2960.15625)}
                    else {vec3(-2835.03125,y,if touching {2904.15625} else {2903.917724609375})};
                let p = &mut r.players[i].player;
                *p = Player::new(feet);
                p.ledge = Some(serde_json::from_value(serde_json::json!({
                    "start":[-2835.03125,y,2899.15625],"top":2960.03125,"forward":[1.,0.],
                    "elapsed":(if leading {2.2000010013580322} else {0.25000008940696716})-if touching {0.} else {FIXED_DT},
                    "pulling":true,"side":0.,"forward_held":true
                })).unwrap());
                p.validate_world(&world).unwrap();
            }
            let mut waits = 0;
            for _ in 0..360 {
                let old = r.players[1-leader].player.feet;
                r.move_players(&world,[(Controls{wish:Vec2::X,jump:true,run:true,..Default::default()},Default::default());2]);
                if r.players[1-leader].player.feet == old {waits+=1;}
                assert!(separate(&r.players),"Climbers overlap: {:?}",r.players.each_ref().map(|b|b.player.feet));
                for b in &r.players {b.player.validate_save().unwrap();b.player.validate_world(&world).unwrap();}
            }
            assert!(waits>0,"Follower never waited");
            assert!(r.players.iter().all(|b|b.player.ledge.is_none() && b.player.climbs==1));
        }}
    }
    #[test]
    fn simultaneous_converging_pull_ups_release_without_trapping_both_players() {
        let world=World::fixture(&[
            (vec3(-500.,-500.,-100.),vec3(500.,500.,0.)),
            (vec3(50.,-100.,0.),vec3(500.,100.,64.)),
        ]);
        for reversed in [false,true] {
            let mut r=pair(&floor(),[vec3(-80.,0.,0.04),vec3(80.,0.,0.04)]);
            for (i,b) in r.players.iter_mut().enumerate() {
                let side=if (i==0)^reversed {-1.}else{1.};
                let start=vec3(34.,side*18.,0.03125);
                let raised=start+Vec3::Z*66.;
                let end=vec3(74.,side*8.,64.03125);
                let elapsed=0.2583334_f32;
                let feet=start.lerp(raised,elapsed/0.65/0.4);
                let mut saved=serde_json::to_value(Player::new(feet)).unwrap();
                saved["climb"]=serde_json::json!({"start":start,"raised":raised,"end":end,"elapsed":elapsed});
                b.player=serde_json::from_value(saved).unwrap();
            }
            for tick in 0..600 {
                let input=std::array::from_fn(|i| (Controls{run:true,
                    // Step away from the wall after release, then make an
                    // ordinary jump retry once the partner has cleared it.
                    wish:if (120..180).contains(&tick) && r.players[i].player.feet.x<80. {-Vec2::X}
                        else {(vec2(240.,0.)-r.players[i].player.feet.truncate()).normalize_or_zero()},
                    jump:r.players[i].player.grounded && r.players[i].player.feet.x<80.,
                    ..Default::default()},Default::default()));
                r.move_players(&world,input);
                assert!(separate(&r.players),"Converging pull-ups overlap");
                for b in &r.players {
                    assert!(world.body_clear(b.player.feet));
                    b.player.validate_save().unwrap();
                    assert_eq!(b.stats.sanity(),100.);
                }
                if r.players.iter().all(|b|!b.player.climbing() && b.player.grounded && b.player.feet.x>150.) {break;}
            }
            assert!(r.players.iter().all(|b|!b.player.climbing() && b.player.grounded && b.player.feet.x>150.),
                "Converging pull-ups remained trapped: {:?}",r.players.each_ref().map(|b|&b.player));
        }
    }
    #[test]
    fn low_climb_waits_without_shoving_then_resumes_when_partner_moves_or_dies() {
        let world = World::fixture(&[
            (vec3(-500.,-500.,-100.),vec3(500.,500.,0.)),
            (vec3(50.,-100.,0.),vec3(500.,100.,64.)),
        ]);
        for moving in 0..2 { for dies in [false,true] {
            let mut r = pair(&world,[vec3(34.,0.,0.04),vec3(92.,0.,64.04)]);
            if moving==1 {r.players.swap(0,1);}
            r.move_players(&world,Default::default());
            let stationary = r.players[1-moving].player.feet;
            let mut inputs = [(Controls::default(),Default::default());2];
            inputs[moving].0 = Controls{wish:Vec2::X,jump:true,..Default::default()};
            for _ in 0..180 {
                r.move_players(&world,inputs);
                assert!(separate(&r.players));
                assert_eq!(r.players[1-moving].player.feet,stationary);
                r.players[moving].player.validate_world(&world).unwrap();
            }
            assert!(r.players[moving].player.climbing());
            // A saved wait must retain a valid path clock and be resumable.
            r.players[moving].player=serde_json::from_value(serde_json::to_value(&r.players[moving].player).unwrap()).unwrap();
            if dies {r.players[1-moving].stats.damage(10000.);}
            else {inputs[1-moving].0.wish=Vec2::X;}
            inputs[moving].0.jump=false;
            for _ in 0..120 {
                r.move_players(&world,inputs);
                if !dies {assert!(separate(&r.players));}
            }
            assert!(!r.players[moving].player.climbing());
            assert!(r.players[moving].player.feet.x>76.);
        }}
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
    fn wall_slide_contact_retreats_along_motion_without_shoving_the_partner() {
        let world=World::fixture(&[
            (vec3(-500.,-500.,-100.),vec3(500.,500.,0.)),
            (vec3(-500.,-500.,0.),vec3(-15.,500.,200.)),
        ]);
        for moving in 0..2 {
            let mut r=pair(&world,[vec3(0.03125,-2.,0.03125),vec3(20.03125,30.,0.03125)]);
            if moving==1 {r.players.swap(0,1);}
            let still=serde_json::to_value(&r.players[1-moving]).unwrap();
            let old=r.players.each_ref().map(|b|b.player.feet);
            r.players[moving].player.feet.y=1.;
            r.players[moving].player.velocity=Vec3::Y*104.;
            resolve(&world,&mut r.players,old);
            assert!(separate(&r.players));
            assert_eq!(serde_json::to_value(&r.players[1-moving]).unwrap(),still);
            let mut input=[(Controls::default(),Default::default());2];
            input[moving].0=Controls{wish:Vec2::Y,run:true,..Default::default()};
            for _ in 0..240 {
                r.move_players(&world,input);
                assert!(separate(&r.players));
                assert!(r.players.iter().all(|b|clear(&world,b,b.player.feet)));
                assert_eq!(r.players[1-moving].player.feet,old[1-moving]);
            }
            input[moving].0.wish=-Vec2::Y;
            for _ in 0..60 {r.move_players(&world,input);}
            assert!(r.players[moving].player.feet.y < -50.);
        }
    }
    #[test]
    fn airborne_wall_contact_preserves_free_vertical_motion() {
        let world=World::fixture(&[(vec3(-500.,-500.,-500.),vec3(-15.,500.,500.))]);
        for moving in 0..2 { for dz in [-10.,10.] {
            let mut bodies=std::array::from_fn(|_|Body{player:Player::new(Vec3::ZERO),
                stats:Stats::default(),yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0});
            bodies[moving].player.feet=vec3(0.03125,-10.,100.);
            bodies[1-moving].player.feet=vec3(35.03125,0.,104.);
            let previous=bodies.each_ref().map(|b|b.player.feet);
            bodies[moving].player.feet+=vec3(0.,8.,dz);
            bodies[moving].player.velocity=vec3(0.,200.,dz/FIXED_DT);
            bodies[1-moving].player.feet.z+=dz;
            let partner=serde_json::to_value(&bodies[1-moving]).unwrap();
            resolve(&world,&mut bodies,previous);
            assert!(separate(&bodies));
            assert_eq!(bodies[moving].player.feet.z,100.+dz,
                "Side contact must not suspend a falling or rising player");
            assert_eq!(bodies[moving].player.velocity.z,dz/FIXED_DT);
            assert_eq!(serde_json::to_value(&bodies[1-moving]).unwrap(),partner);
            for b in &bodies { assert!(world.body_clear(b.player.feet)); }
        }}
    }
    #[test]
    fn landing_beside_a_wall_retreats_at_floor_height_without_shoving() {
        let world=World::fixture(&[
            (vec3(3700.,2600.,-100.),vec3(4000.,3000.,0.)),
            (vec3(3700.,2600.,0.),vec3(3808.,3000.,200.)),
            (vec3(3868.,2600.,0.),vec3(4000.,3000.,200.)),
        ]);
        for moving in 0..2 {
            // Set up the roster independently of entrance-placement searches;
            // this corridor fixture exercises an already-running encounter.
            let mut r=pair(&floor(),[vec3(-80.,0.,0.03125),vec3(80.,0.,0.03125)]);
            for (b,at) in r.players.iter_mut().zip([vec3(3849.4208984,2801.5256348,0.03125),
                vec3(3823.0561523,2826.0388184,0.03125)]) {
                b.player=Player::new(at);b.player.grounded=true;b.player.ground_normal=Vec3::Z;
            }
            r.players[1].player.feet.z=2.9198363;
            r.players[1].player.grounded=false;
            r.players[1].player.velocity=vec3(-5.19008875,-5.58211374,-100.000305);
            if moving==0 {r.players.swap(0,1);}
            let stationary=r.players[1-moving].player.feet;
            let resources=r.players.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
            for tick in 0..240 {
                let d=vec3(3840.,2784.,0.)-r.players[moving].player.feet;
                let mut input=[(Controls::default(),Default::default());2];
                input[moving].0=Controls{wish:d.truncate().normalize_or_zero()*(d.truncate().length()/48.).min(1.),run:true,..Default::default()};
                r.move_players(&world,input);
                assert!(separate(&r.players),"Landing contact overlaps slot{moving} tick{tick}");
                assert_eq!(r.players[1-moving].player.feet,stationary);
                for (i,b) in r.players.iter().enumerate() {
                    assert!(clear(&world,b,b.player.feet));
                    assert_eq!(serde_json::to_value(&b.stats).unwrap(),resources[i]);
                }
            }
            assert!(r.players[moving].player.grounded);
        }
    }
    #[test]
    fn vertical_entry_moves_only_the_approaching_body_in_either_slot() {
        for moving in 0..2 {for direction in [-1.,1.] {
            let world=World::fixture(&[]);
            let mut bodies=std::array::from_fn(|_|Body{player:Player::new(Vec3::ZERO),
                stats:Stats::default(),yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0});
            bodies[moving].player.feet=vec3(20.,0.,-direction*58.);
            let previous=bodies.each_ref().map(|b|b.player.feet);
            let still=serde_json::to_value(&bodies[1-moving]).unwrap();
            bodies[moving].player.feet+=vec3(0.01,0.,direction*6.);
            bodies[moving].player.velocity=vec3(1.2,0.,direction*720.);
            resolve(&world,&mut bodies,previous);
            assert!(separate(&bodies));
            assert_eq!(serde_json::to_value(&bodies[1-moving]).unwrap(),still);
            assert!(world.body_clear(bodies[moving].player.feet));
        }}
    }
    #[test]
    fn narrow_vertical_contact_preserves_partner_and_charges_personal_landing() {
        let world=World::fixture(&[
            (vec3(-100.,-100.,-100.),vec3(100.,100.,0.)),
            (vec3(-100.,-100.,0.),vec3(-20.,100.,200.)),
            (vec3(20.,-100.,0.),vec3(100.,100.,200.)),
            (vec3(-100.,-100.,0.),vec3(100.,-20.,200.)),
            (vec3(-100.,20.,0.),vec3(100.,100.,200.)),
        ]);
        for moving in 0..2 {
            let mut bodies=std::array::from_fn(|_|Body{player:Player::new(Vec3::Z*0.03125),
                stats:Stats::default(),yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0});
            bodies[1-moving].player.grounded=true;
            bodies[moving].player.feet.z=58.;
            let previous=bodies.each_ref().map(|b|b.player.feet);
            let still=serde_json::to_value(&bodies[1-moving]).unwrap();
            bodies[moving].player.feet.z=52.;bodies[moving].player.velocity.z=-800.;
            resolve(&world,&mut bodies,previous);
            assert!(separate(&bodies));
            assert_eq!(serde_json::to_value(&bodies[1-moving]).unwrap(),still);
            assert!(bodies[moving].stats.sanity()<100. && bodies[moving].stats.alive());
            assert_eq!(bodies[moving].player.velocity.z,0.);
            assert_eq!(bodies[moving].player.landings,1);
            for b in &bodies {assert!(world.body_clear(b.player.feet));}
            let health=bodies[moving].stats.sanity();
            for _ in 0..120 {
                let previous=bodies.each_ref().map(|b|b.player.feet);
                for b in &mut bodies {b.player.tick(&world,Controls::default());}
                resolve(&world,&mut bodies,previous);
                assert!(separate(&bodies));
                assert_eq!(serde_json::to_value(&bodies[1-moving]).unwrap(),still);
                assert_eq!(bodies[moving].player.landings,1);
                assert_eq!(bodies[moving].stats.sanity(),health);
            }
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
    fn repeated_diagonal_slope_contact_does_not_accumulate_fall_damage() {
        let world = World::ramp_fixture();
        for moving in 0..2 { for jumping in [false,true] {
            let feet = [vec3(0.,0.,7.54),vec3(12.,34.,13.54)];
            let mut r = pair(&world,if moving==1 {feet} else {[feet[1],feet[0]]});
            for tick in 0..1200 {
                let delta = vec2(3.,7.) - r.players[moving].player.feet.truncate();
                let mut input = [(Controls::default(),Default::default());2];
                input[moving].0 = Controls {wish:delta.normalize_or_zero(),run:true,
                    jump:jumping && tick%120==119,..Default::default()};
                r.move_players(&world,input);
                assert_eq!(r.players[moving].stats.sanity(),100.,
                    "False slope impact moving{moving} jumping{jumping} tick{tick}: feet{:?} velocity{:?} landing{}",
                    r.players[moving].player.feet,r.players[moving].player.velocity,r.players[moving].player.landing_speed);
            }
        }}
    }
    #[test]
    fn airborne_contact_on_a_slope_does_not_store_a_fatal_fall() {
        let normal = vec3(0.,0.4166547,0.9090648);
        let anchor = vec3(-2207.909,-1146.6525,-296.875);
        let plane = crate::bsp::Plane {normal,
            distance:normal.dot(anchor)-normal.abs().dot(vec3(15.,15.,0.))-crate::collision::SKIN};
        let world = World::sloped_fixture(vec3(-2800.,-1600.,-1200.),vec3(-1800.,-700.,1000.),plane);
        let floating = vec3(-2174.2693,-1159.9398,-280.88013);
        for moving in 0..2 {
            let mut r = pair(&world,[anchor,anchor+Vec3::X*80.]);
            r.players[1-moving].player = Player::spawn(&world,anchor+Vec3::Z*EYE_HEIGHT).unwrap();
            r.players[moving].player = Player::new(floating);
            for tick in 0..480 {
                let mut inputs = [(Controls::default(),Default::default());2];
                inputs[moving].0 = Controls {run:true,
                    wish:(vec2(-2200.,-1150.)-r.players[moving].player.feet.truncate()).normalize_or_zero(),
                    ..Default::default()};
                if tick>=420 {
                    inputs[1-moving].0 = Controls {wish:vec2(-0.3711243,0.9285832),run:true,..Default::default()};
                }
                r.move_players(&world,inputs);
                assert_eq!(r.players[moving].stats.sanity(),100.,
                    "Contact stored a fall moving{moving} tick{tick}: feet{:?} velocity{:?} landing{}",
                    r.players[moving].player.feet,r.players[moving].player.velocity,r.players[moving].player.landing_speed);
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
    fn already_overhanging_approach_separates_without_shoving_the_partner() {
        let world=World::fixture(&[(vec3(-6.,-200.,-100.),vec3(6.,200.,0.))]);
        for moving in 0..2 {
            let mut bodies=std::array::from_fn(|_|Body{player:Player::new(vec3(8.,0.,0.03125)),
                stats:Stats::default(),yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0});
            for b in &mut bodies {b.player.grounded=true;}
            bodies[moving].player.feet.y=-DISTANCE;
            let still=serde_json::to_value(&bodies[1-moving]).unwrap();
            for _ in 0..240 {
                let previous=bodies.each_ref().map(|b|b.player.feet);
                bodies[moving].player.tick(&world,Controls{wish:Vec2::Y,run:true,..Default::default()});
                resolve(&world,&mut bodies,previous);
                assert!(separate(&bodies));
                assert_eq!(serde_json::to_value(&bodies[1-moving]).unwrap(),still);
                assert!(bodies[moving].player.grounded && world.body_clear(bodies[moving].player.feet));
                assert!(world.body_trace(bodies[moving].player.feet,bodies[moving].player.feet-Vec3::Z*2.).fraction<1.);
            }
        }
    }
    #[test]
    fn overhang_correction_cannot_create_an_overhang_or_cross_a_support_gap() {
        let world=World::fixture(&[(vec3(-6.,-200.,-100.),vec3(6.,200.,0.))]);
        let mut body=Body{player:Player::new(vec3(0.,0.,0.03125)),stats:Stats::default(),
            yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0};
        body.player.grounded=true;
        assert!(!clear(&world,&body,vec3(8.,0.,0.03125)));
        body.player.feet.x=8.;
        assert!(!clear(&world,&body,vec3(22.,0.,0.03125)));
        let gap=World::fixture(&[(vec3(-20.,-100.,-100.),vec3(-10.,100.,0.)),
            (vec3(10.,-100.,-100.),vec3(20.,100.,0.))]);
        body.player.feet.x=-24.;
        assert!(!clear(&gap,&body,vec3(24.,0.,0.03125)));
    }
    #[test]
    fn contact_at_a_support_boundary_can_retreat_to_the_actual_previous_footing() {
        let world=World::fixture(&[
            (vec3(-6.,-200.,-100.),vec3(6.,200.,0.)),
            (vec3(-100.,0.2,-100.),vec3(100.,200.,0.)),
            (vec3(-100.,-200.,0.),vec3(-7.,200.,200.)),
        ]);
        for swap in [false,true] {
            let mut bodies=std::array::from_fn(|_|Body{player:Player::new(Vec3::ZERO),
                stats:Stats::default(),yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0});
            let mut previous=[vec3(43.057129,8.355615,0.03125),vec3(8.03125,-0.4,0.03125)];
            for (b,at) in bodies.iter_mut().zip([vec3(42.857178,9.198877,0.03125),vec3(8.03125,0.44765625,0.03125)]) {
                b.player.feet=at;b.player.grounded=true;
            }
            assert!(!clear(&world,&bodies[1],previous[1]),"Normal correction must not create an overhang");
            assert_eq!(previous_footing(&world,&bodies[1],previous[1]),Some(previous[1]));
            if swap {bodies.swap(0,1);previous.swap(0,1);}
            let departing=usize::from(swap);
            let unchanged=serde_json::to_value(&bodies[departing]).unwrap();
            let resources=bodies.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
            resolve(&world,&mut bodies,previous);
            assert!(separate(&bodies));
            assert_eq!(serde_json::to_value(&bodies[departing]).unwrap(),unchanged);
            assert_eq!(bodies[1-departing].player.feet,previous[1-departing]);
            for (i,b) in bodies.iter().enumerate() {
                assert!(world.body_clear(b.player.feet));
                assert!(world.body_trace(b.player.feet,b.player.feet-Vec3::Z*2.).fraction<1.);
                assert_eq!(serde_json::to_value(&b.stats).unwrap(),resources[i]);
            }
        }
    }
    #[test]
    fn previous_footing_still_requires_a_clear_supported_dry_retreat() {
        let world=World::fixture(&[
            (vec3(-100.,-100.,-100.),vec3(100.,100.,0.)),
            (vec3(-5.,-100.,0.),vec3(5.,100.,100.)),
        ]);
        let mut body=Body{player:Player::new(vec3(40.,0.,0.03125)),stats:Stats::default(),
            yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0};
        body.player.grounded=true;
        assert_eq!(previous_footing(&world,&body,vec3(50.,0.,0.03125)),Some(vec3(50.,0.,0.03125)));
        for at in [vec3(-40.,0.,0.03125),vec3(120.,0.,0.03125),vec3(40.,0.,10.),vec3(0.,0.,0.03125)] {
            assert!(previous_footing(&world,&body,at).is_none());
        }
        body.player.swimming=true;
        assert!(previous_footing(&world,&body,body.player.feet).is_none());
        body.player.swimming=false;body.player.grounded=false;
        assert!(previous_footing(&world,&body,body.player.feet).is_none());
    }
    #[test]
    fn wall_tangent_contact_preserves_height_and_cannot_cross_a_corner() {
        let floor=(vec3(-500.,-500.,-100.),vec3(500.,500.,0.));
        let wall=(vec3(-500.,-500.,0.),vec3(0.,500.,300.));
        let world=World::fixture(&[floor,wall]);
        let mut body=Body{player:Player::new(vec3(15.01,0.,0.04)),stats:Stats::default(),
            yaw:0.,pitch:0.,running:false,spawned_at:0,warped_at:0};
        let outward=vec2(-1.,-1.).normalize();
        for grounded in [false,true] {
            body.player.grounded=grounded;
            let at=contact_correction(&world,&body,outward.extend(0.),outward,0.5).unwrap();
            assert!((at-body.player.feet).truncate().dot(outward)>=0.4999);
            assert_eq!(at.z,body.player.feet.z);
            assert!(world.body_clear(at) && world.body_trace(body.player.feet,at).fraction==1.);
            // No tangent can separate directly opposing bodies at this wall.
            assert!(contact_correction(&world,&body,-Vec3::X,-Vec2::X,0.5).is_none());
            let corner=World::fixture(&[floor,wall,(vec3(0.,-500.,0.),vec3(500.,-15.01,300.))]);
            assert!(contact_correction(&corner,&body,outward.extend(0.),outward,0.5).is_none());
        }
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
