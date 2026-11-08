//! Collision-checked partner placement, including authored slopes and swimming.
use crate::{
    collision::World,
    movement::{Player, EYE_HEIGHT},
    water::{Immersion, Liquid},
};
use macroquad::prelude::*;

/// Entrance markers may be above their playable landing. An ordinary revival
/// uses the narrower search instead, to remain near the survivor.
pub fn entrance(world: &World, feet: Vec3, safe: impl Fn(Vec3) -> bool) -> Option<Player> {
    near(world, feet, 256., safe)
}

/// Atomic placement at an authored scene endpoint. Both bodies must fit on
/// safe, fully supported ground before the caller changes either player.
pub(crate) fn scene_pair(world:&World, anchor:Vec3, safe:impl Fn(Vec3)->bool)->Option<(Player,Player)> {
    scene_landings(world,anchor,&safe).into_iter().find_map(|first| {
        reunion(world,&first,&safe).map(|second|(first,second))
    })
}
pub(crate) fn scene_landings(world:&World, anchor:Vec3, safe:impl Fn(Vec3)->bool)->Vec<Player> {
    // This is a cutscene endpoint search, not movement from the pre-scene
    // body. An authored mark can lie against trim or a railing; accept only
    // nearby actual floor that fits both players before applying the warp.
    let mut landings=Vec::new();
    for radius in (0..=24).map(|r|r as f32*4.) {
        for index in 0..32 {
            let angle=index as f32*std::f32::consts::TAU/32.;
            let at=anchor+vec3(angle.cos(),angle.sin(),0.)*radius+Vec3::Z*18.;
            let Some(first)=beside_with_step(world,at,at,50.,0.,&safe) else {continue;};
            landings.push(first);
        }
    }
    landings
}

/// A completed scene can restore Alice at the airborne point where it started.
/// Place the partner on nearby supported ground so ordinary gravity can resume
/// for the owner. This wider search must not change revival placement rules.
pub fn reunion(world: &World, owner: &Player, safe: impl Fn(Vec3) -> bool) -> Option<Player> {
    let drop = if owner.grounded { 32. } else { 256. };
    near(world, owner.feet, drop, &safe).or_else(|| {
        // A valid scene landing may have no upward clearance under a shelf.
        // Try a level sweep from the actual body position, retaining the same
        // distance/drop limits, full footprint and hazard checks. Ordinary
        // revival keeps its existing placement rules.
        for radius in [48., 72., 96.] {
            for direction in directions() {
                if let Some(player) = beside_with_step(world, owner.feet,
                    owner.feet + direction * radius, drop, 0., &safe) {
                    return Some(player);
                }
            }
        }
        // A small landing can fit a partner between the eight coarse spokes,
        // especially when the scene owner stands near its edge. Search finer
        // angles before leaving the completed scene locked. Keep the owner in
        // place and retain every wall, full-footprint, drop and hazard check.
        for radius in [40., 48., 56., 64., 80., 96.] {
            for index in 0..32 {
                let angle = index as f32 * std::f32::consts::TAU / 32.;
                let point = owner.feet + vec3(angle.cos(), angle.sin(), 0.) * radius;
                for step in [18., 0.] {
                    if let Some(player) = beside_with_step(world, owner.feet, point, drop, step, &safe) {
                        return Some(player);
                    }
                }
            }
        }
        None
    })
}

/// A second scene can end beside a partner already standing on a safe nearby
/// pad. If no new placement fits, keep that existing position rather than
/// leaving the completed scene locked. This never moves a body through a wall.
pub fn reunion_with_partner(world: &World, owner: &Player, partner: &Player,
    safe: impl Fn(Vec3) -> bool) -> Option<Player> {
    reunion(world, owner, &safe).or_else(|| {
        let delta = partner.feet - owner.feet;
        let sight = world.sweep(owner.eye(), partner.eye(), Vec3::splat(0.5));
        (owner.grounded && partner.grounded && delta.truncate().length() >= 40.
            && delta.length() <= 128. && delta.z.abs() <= 32.
            && supported(world, partner.feet) && safe(partner.feet)
            && !sight.start_solid && sight.fraction >= 1.)
            .then(|| Player::spawn(world, partner.feet + Vec3::Z * EYE_HEIGHT)).flatten()
    })
}

/// Opt-in for an authored scene ending on a narrow perch. Ordinary scene
/// reunion and death revival retain their existing smaller search. Even this
/// fallback cannot cross a wall, land on an edge or enter a controller hazard.
pub fn reunion_with_limits(world:&World, owner:&Player, radius:f32, drop:f32,
    safe:impl Fn(Vec3)->bool)->Option<Player> {
    if !radius.is_finite() || !drop.is_finite() || !(40. ..=256.).contains(&radius)
        || !(0. ..=256.).contains(&drop) {return None;}
    for distance in [40.,48.,64.,80.,96.,112.,128.,160.,192.,224.,256.] {
        if distance>radius {break;}
        for index in 0..32 {
            let angle=index as f32*std::f32::consts::TAU/32.;
            let point=owner.feet+vec3(angle.cos(),angle.sin(),0.)*distance;
            for step in [18.,0.] {
                if let Some(player)=beside_with_step(world,owner.feet,point,drop,step,&safe) {return Some(player);}
            }
        }
    }
    None
}

pub fn near(world: &World, feet: Vec3, drop: f32, safe: impl Fn(Vec3) -> bool) -> Option<Player> {
    for radius in [48., 72., 96.] {
        for direction in directions() {
            if let Some(player) = beside(world, feet, feet + direction * radius, drop, &safe) {
                return Some(player);
            }
        }
    }
    None
}

/// Prefer nearby ground/water. Over a void, a valid native rope can support a
/// second player at a different grip length without inventing a landing.
pub(super) fn revival(world: &World, survivor: &Player, safe: impl Fn(Vec3) -> bool) -> Option<Player> {
    near(world, survivor.feet, 32., &safe).or_else(|| on_rope(world, survivor, &safe))
}

fn on_rope(world: &World, survivor: &Player, safe: &impl Fn(Vec3) -> bool) -> Option<Player> {
    let grip = survivor.rope.as_ref()?;
    let rope = world.traversal.rope(grip.id)?;
    survivor.validate_save().ok()?;
    survivor.validate_world(world).ok()?;
    let center = rope.anchor - Vec3::Z * 40.;
    let radial = (survivor.feet - center).try_normalize()?;
    // A 72-unit radial gap exceeds the diagonal of our horizontal/vertical
    // personal-space bounds, even while the two grips swing independently.
    for offset in [-72., 72., -96., 96.] {
        let length = grip.length + offset;
        if !(32. ..=rope.length).contains(&length) { continue; }
        let feet = center + radial * length;
        if !clear_path(world, survivor.feet, feet) || !safe(feet)
            || Immersion::sample(world, feet).level != 0 { continue; }
        let delta = feet - survivor.feet;
        if delta.truncate().length() < super::separation::DISTANCE
            && delta.z.abs() < crate::collision::PLAYER_HALF.z * 2. { continue; }
        let mut player = Player::new(feet);
        player.rope = Some(crate::movement::RopeGrip {id:grip.id, length, rise:0.});
        player.script_motion = 1;
        player.script_facing = survivor.script_facing;
        // Keep angular motion with the survivor, discarding radial momentum.
        player.velocity = ((survivor.velocity - radial * survivor.velocity.dot(radial))
            * (length / grip.length)).clamp_length_max(500.);
        if player.validate_save().is_ok() && player.validate_world(world).is_ok() {
            return Some(player);
        }
    }
    None
}

fn directions() -> [Vec3; 8] {
    [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y,
     vec3(1.,1.,0.).normalize(), vec3(-1.,1.,0.).normalize(),
     vec3(1.,-1.,0.).normalize(), vec3(-1.,-1.,0.).normalize()]
}

/// Read-only options for the opt-in route harness. These are destinations to
/// walk to, never placements applied to the authoritative player. Keep them
/// close and at the same landing height; the harness chooses its approach side.
pub(super) fn neighbors(world: &World, feet: Vec3, safe: impl Fn(Vec3) -> bool) -> Vec<Vec3> {
    directions().into_iter().filter_map(|direction| {
        beside(world, feet, feet + direction * 48., 18., &safe)
            .filter(|p|p.grounded && (p.feet.z-feet.z).abs() <= 18.1)
            .map(|p|p.feet)
    }).collect()
}

fn beside(world: &World, feet: Vec3, beside: Vec3, drop: f32, safe: &impl Fn(Vec3) -> bool) -> Option<Player> {
    beside_with_step(world, feet, beside, drop, 18., safe)
}

fn beside_with_step(world: &World, feet: Vec3, beside: Vec3, drop: f32, step: f32,
    safe: &impl Fn(Vec3) -> bool) -> Option<Player> {
    let source = Immersion::sample(world, feet);
    // A swimmer needs clear water at the same depth, not a dry floor.
    // Trace against walls and sample water along the route, rather than
    // accepting only an endpoint on the far side of a shore.
    if source.kind == Liquid::Water
        && source.level >= 2
        && safe(beside)
        && clear_path(world, feet, beside)
        && (0..=8).all(|i| {
            let at = feet.lerp(beside, i as f32 / 8.);
            let liquid = Immersion::sample(world, at);
            liquid.kind == Liquid::Water && liquid.level >= 2
        })
    {
        return Player::spawn(world, beside + Vec3::Z * EYE_HEIGHT);
    }
    let candidate = beside + Vec3::Z * step;
    if !clear_path(world, feet + Vec3::Z * step.min(1.), candidate) {
        return None;
    }
    let distance = drop + step;
    let floor = world.body_trace(candidate, candidate - Vec3::Z * distance);
    if floor.start_solid || floor.fraction >= 1. || floor.normal.z < 0.65 {
        return None;
    }
    let landing = candidate - Vec3::Z * (distance * floor.fraction);
    if !supported(world, landing)
        || !safe(landing)
        || !clear_path(world, candidate, landing)
    {
        return None;
    }
    Player::spawn(world, landing + Vec3::Z * EYE_HEIGHT)
}

fn clear_path(world: &World, from: Vec3, to: Vec3) -> bool {
    let hit = world.body_trace(from, to);
    !hit.start_solid && hit.fraction >= 1. && world.body_clear(to)
}

/// Require a full footprint plus a small edge margin. Probe around the actual
/// floor plane: a fixed four-unit downward ray incorrectly rejects normal hills.
pub(crate) fn supported(world: &World, feet: Vec3) -> bool {
    if !world.body_clear(feet) || Immersion::sample(world, feet).level != 0 {
        return false;
    }
    let start = feet + Vec3::Z;
    let center = world.sweep(start, start - Vec3::Z * 41., Vec3::ZERO);
    if center.start_solid || center.fraction >= 1. || center.normal.z < 0.65 {
        return false;
    }
    let ground = start - Vec3::Z * (41. * center.fraction);
    [-20., 0., 20.].into_iter().all(|x| {
        [-20., 0., 20.].into_iter().all(|y| {
            let expected = ground
                + vec3(
                    x,
                    y,
                    -(center.normal.x * x + center.normal.y * y) / center.normal.z,
                );
            let from = expected + Vec3::Z * 8.;
            let hit = world.sweep(from, from - Vec3::Z * 16., Vec3::ZERO);
            !hit.start_solid && hit.fraction < 1. && hit.normal.z >= 0.65
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_scene_pair_requires_two_safe_supported_landings() {
        let floor=(vec3(-300.,-300.,-20.),vec3(300.,300.,0.));
        let world=World::fixture(&[floor]);
        let (a,b)=scene_pair(&world,Vec3::ZERO,|_|true).unwrap();
        assert!(a.grounded && b.grounded && supported(&world,a.feet) && supported(&world,b.feet));
        assert!((a.feet-b.feet).truncate().length()>=40.);
        assert!(scene_pair(&world,Vec3::ZERO,|_|false).is_none());
        let small=World::fixture(&[(vec3(-22.,-22.,-20.),vec3(22.,22.,0.))]);
        assert!(scene_pair(&small,Vec3::ZERO,|_|true).is_none());
        let roof=(vec3(-300.,-300.,40.),vec3(300.,300.,80.));
        assert!(scene_pair(&World::fixture(&[floor,roof]),Vec3::ZERO,|_|true).is_none());
        assert!(scene_pair(&World::fixture(&[]),Vec3::ZERO,|_|true).is_none());
    }
    #[test]
    fn authored_perch_reunion_keeps_wall_support_and_hazard_checks() {
        let floor=(vec3(-400.,-400.,-20.),vec3(400.,400.,0.));
        let perch=(vec3(-20.,-20.,0.),vec3(20.,20.,200.));
        let world=World::fixture(&[floor,perch]);
        let owner=Player::spawn(&world,Vec3::Z*(200.03125+EYE_HEIGHT)).unwrap();
        assert!(reunion(&world,&owner,|_|true).is_none());
        assert!(revival(&world,&owner,|_|true).is_none());
        let partner=reunion_with_limits(&world,&owner,256.,256.,|_|true).unwrap();
        assert!(partner.grounded && supported(&world,partner.feet));
        assert!(partner.feet.z<1. && (partner.feet-owner.feet).truncate().length()>=40.);
        assert!(reunion_with_limits(&world,&owner,256.,32.,|_|true).is_none());
        assert!(reunion_with_limits(&world,&owner,256.,256.,|_|false).is_none());
        for (radius,drop) in [(257.,256.),(256.,257.),(f32::NAN,256.),(256.,f32::NAN)] {
            assert!(reunion_with_limits(&world,&owner,radius,drop,|_|true).is_none());
        }
        let walls=[(vec3(28.,-60.,0.),vec3(32.,60.,500.)),(vec3(-32.,-60.,0.),vec3(-28.,60.,500.)),
            (vec3(-60.,28.,0.),vec3(60.,32.,500.)),(vec3(-60.,-32.,0.),vec3(60.,-28.,500.))];
        let enclosed=World::fixture(&[vec![floor,perch],walls.to_vec()].concat());
        assert!(reunion_with_limits(&enclosed,&owner,256.,256.,|_|true).is_none());
    }
    #[test]
    fn rope_revival_uses_clear_separate_grips_and_rejects_blocked_or_inactive_ropes() {
        use crate::{movement::{Controls, RopeGrip}, traversal::Rope};
        let mut world = World::fixture(&[]);
        world.traversal.ropes.push(Rope {id:crate::entity::Id(1), model:1, enabled:true,
            origin:Vec3::ZERO, anchor:Vec3::Z*600., length:500.});
        let mut survivor = Player::new(vec3(80.,0.,300.));
        survivor.rope = Some(RopeGrip {id:crate::entity::Id(1),
            length:survivor.feet.distance(Vec3::Z*560.),rise:0.});
        survivor.script_motion = 1;
        survivor.velocity = Vec3::Y*80.;
        let original = serde_json::to_value(&survivor).unwrap();
        let mut partner = revival(&world,&survivor,|_|true).unwrap();
        assert_eq!(serde_json::to_value(&survivor).unwrap(),original);
        assert!(partner.rope.is_some() && !partner.grounded);
        assert!((partner.feet.distance(survivor.feet)-72.).abs()<0.001);
        for _ in 0..120 {
            survivor.tick(&world,Controls::default());
            partner.tick(&world,Controls::default());
            for player in [&survivor,&partner] {
                player.validate_save().unwrap();
                player.validate_world(&world).unwrap();
            }
            assert!(partner.feet.distance(survivor.feet)>71.9);
        }
        assert!(revival(&world,&survivor,|_|false).is_none());
        world.traversal.ropes[0].enabled=false;
        assert!(revival(&world,&survivor,|_|true).is_none());
        world.traversal.ropes[0].enabled=true;
        world.traversal.managed_ropes=true;
        assert!(revival(&world,&survivor,|_|true).is_none());
        let mut blocked=World::fixture(&[(vec3(-500.,-500.,180.),vec3(500.,500.,250.)),
            (vec3(-500.,-500.,360.),vec3(500.,500.,500.))]);
        blocked.traversal=world.traversal.clone();
        blocked.traversal.managed_ropes=false;
        let mut middle=Player::new(Vec3::Z*280.);
        middle.rope=Some(RopeGrip {id:crate::entity::Id(1),length:280.,rise:0.});
        middle.script_motion=1;
        assert!(on_rope(&blocked,&middle,&|_|true).is_none());
    }
    #[test]
    fn completed_scene_can_keep_an_already_safe_nearby_partner() {
        let floor = (vec3(-500.,-500.,-100.),vec3(500.,500.,0.));
        let world = World::fixture(&[floor]);
        let owner = Player::spawn(&world,Vec3::Z*EYE_HEIGHT).unwrap();
        let partner = Player::spawn(&world,vec3(112.,0.,EYE_HEIGHT)).unwrap();
        let safe = |at:Vec3| at.distance(partner.feet)<0.1;
        assert!(reunion(&world,&owner,safe).is_none());
        let kept = reunion_with_partner(&world,&owner,&partner,safe).unwrap();
        assert!(kept.grounded && kept.feet.distance(partner.feet)<0.1);
        assert!(reunion_with_partner(&world,&owner,&partner,|_|false).is_none());
        for at in [vec3(24.,0.,EYE_HEIGHT),vec3(129.,0.,EYE_HEIGHT)] {
            let other=Player::spawn(&world,at).unwrap();
            assert!(reunion_with_partner(&world,&owner,&other,|p|p.distance(other.feet)<0.1).is_none());
        }
        let wall = World::fixture(&[floor,(vec3(55.,-500.,0.),vec3(57.,500.,100.))]);
        assert!(reunion_with_partner(&wall,&owner,&partner,safe).is_none());
        let edge = World::fixture(&[(vec3(-20.,-20.,-100.),vec3(120.,20.,0.))]);
        assert!(reunion_with_partner(&edge,&owner,&partner,safe).is_none());
    }
    #[test]
    fn scene_reunion_finds_supported_space_between_coarse_search_directions() {
        let world = World::fixture(&[(vec3(-20., -10., -100.), vec3(90., 35., 0.))]);
        let owner = Player::spawn(&world, Vec3::Z * (EYE_HEIGHT + 0.03125)).unwrap();
        assert!(owner.grounded);
        assert!(near(&world, owner.feet, 32., |_| true).is_none());
        let partner = reunion(&world, &owner, |_| true).unwrap();
        assert!(partner.grounded && supported(&world, partner.feet));
        assert!((40. ..=110.).contains(&partner.feet.distance(owner.feet)));
        assert!(partner.feet.y >= 10. && partner.feet.y <= 15.);
        assert!(reunion(&world, &owner, |_| false).is_none());
    }
    #[test]
    fn scene_reunion_can_sweep_under_a_low_ceiling_without_crossing_walls() {
        for ceiling in [56.5, 64.] {
            let floor = (vec3(-500., -500., -100.), vec3(500., 500., 0.));
            let roof = (vec3(-500., -500., ceiling), vec3(500., 500., 100.));
            let world = World::fixture(&[floor, roof]);
            let owner = Player::spawn(&world, Vec3::Z * (EYE_HEIGHT + 0.03125)).unwrap();
            assert!(owner.grounded);
            assert!(near(&world, owner.feet, 32., |_| true).is_none());
            let partner = reunion(&world, &owner, |_| true).unwrap();
            assert!(partner.grounded && supported(&world, partner.feet));
            assert!((partner.feet.distance(owner.feet)-48.).abs() < 0.1);
            assert!(reunion(&world, &owner, |_| false).is_none());
            let wall = World::fixture(&[floor, roof,
                (vec3(19., -500., 0.), vec3(22., 500., 100.)),
                (vec3(-22., -500., 0.), vec3(-19., 500., 100.)),
                (vec3(-500., 19., 0.), vec3(500., 22., 100.)),
                (vec3(-500., -22., 0.), vec3(500., -19., 100.)),
            ]);
            assert!(reunion(&wall, &owner, |_| true).is_none());
        }
    }
    #[test]
    fn route_neighbors_use_supported_same_landing_and_respect_hazards_and_walls() {
        let floor = World::fixture(&[(vec3(-500.,-500.,-100.),vec3(500.,500.,0.))]);
        let feet = Vec3::Z*0.03125;
        let candidates = neighbors(&floor,feet,|_|true);
        assert_eq!(candidates.len(),8);
        assert!(candidates.iter().all(|p|(p.truncate().length()-48.).abs()<0.01 && p.z==feet.z));
        assert!(neighbors(&floor,feet,|_|false).is_empty());
        let narrow = World::fixture(&[(vec3(-22.,-500.,-100.),vec3(22.,500.,0.))]);
        let candidates = neighbors(&narrow,feet,|_|true);
        assert_eq!(candidates.len(),2);
        assert!(candidates.iter().all(|p|p.x==0.));
        let wall = World::fixture(&[(vec3(-500.,-500.,-100.),vec3(500.,500.,0.)),
                                   (vec3(17.,-500.,0.),vec3(20.,500.,100.))]);
        assert!(neighbors(&wall,feet,|_|true).iter().all(|p|p.x<=0.));
        assert!(neighbors(&floor,feet+Vec3::Z*40.,|_|true).is_empty());
    }
    #[test]
    fn airborne_scene_reunion_can_finish_without_widening_ordinary_revival() {
        let world = World::fixture(&[(vec3(-500., -500., -100.), vec3(500., 500., 0.))]);
        let airborne = Player::new(Vec3::Z * 46.67);
        assert!(near(&world, airborne.feet, 32., |_| true).is_none());
        let partner = reunion(&world, &airborne, |_| true).unwrap();
        assert!(partner.grounded && partner.feet.z.abs() < 0.1);
        assert!((partner.feet.truncate().length()-48.).abs() < 0.1);
        assert!(reunion(&world, &airborne, |_| false).is_none());
        assert!(reunion(&world, &Player::new(Vec3::Z * 300.), |_| true).is_none());
        let narrow = World::fixture(&[(vec3(-500., -12., -100.), vec3(500., 12., 0.))]);
        assert!(reunion(&narrow, &airborne, |_| true).is_none());
        let ledge = World::fixture(&[
            (vec3(-500.,-500.,-200.),vec3(500.,500.,-100.)),
            (vec3(-20.,-20.,-100.),vec3(20.,20.,0.)),
        ]);
        let standing = Player::spawn(&ledge, Vec3::Z * (EYE_HEIGHT+0.04)).unwrap();
        assert!(standing.grounded);
        assert!(reunion(&ledge, &standing, |_| true).is_none(),
            "A grounded owner's partner must not be moved to a lower floor");
    }
    #[test]
    fn swimmer_revives_at_the_same_depth_with_fresh_air_but_never_in_harmful_liquid() {
        use crate::collision::{Collider, Liquid as Volume};
        let mut world = World::fixture(&[]);
        let volume = |contents| Volume {
            contents,
            volume: Collider::fixture(Vec3::splat(-500.), Vec3::splat(500.)),
        };
        world.set_dynamic_liquids(vec![volume(32)]);
        let player = near(&world, Vec3::ZERO, 32., |_| true).unwrap();
        assert!(player.swimming && !player.grounded);
        assert_eq!(player.feet.z, 0.);
        assert_eq!(player.breath.submerged, 0.);
        assert!(near(&world, Vec3::ZERO, 32., |_| false).is_none());
        for harmful in [8, 16] {
            world.set_dynamic_liquids(vec![volume(harmful)]);
            assert!(near(&world, Vec3::ZERO, 32., |_| true).is_none());
        }
        // Even if a distant water volume contains the endpoint, do not cross
        // an intervening dry gap while reviving an underwater player.
        world.set_dynamic_liquids(vec![
            Volume {
                contents: 32,
                volume: Collider::fixture(Vec3::splat(-10.), vec3(10., 10., 100.)),
            },
            Volume {
                contents: 32,
                volume: Collider::fixture(vec3(40., -100., -100.), vec3(100., 100., 100.)),
            },
        ]);
        assert!(near(&world, Vec3::ZERO, 32., |_| true).is_none());
    }
    #[test]
    fn elevated_entrance_can_land_but_respawn_cannot_drop_that_far() {
        let world = World::fixture(&[(vec3(-500., -500., -100.), vec3(500., 500., 0.))]);
        let feet = Vec3::Z * 232.;
        assert!(near(&world, feet, 32., |_| true).is_none());
        let placed = entrance(&world, feet, |_| true).unwrap();
        assert!(placed.grounded && placed.feet.z.abs() < 0.1);
        assert!(entrance(&world, Vec3::Z * 300., |_| true).is_none());
        assert!(entrance(&world, feet, |at| at.z > 10.).is_none());
    }
    #[test]
    fn entrance_does_not_cross_a_wall_or_accept_a_narrow_ledge() {
        let ledge = World::fixture(&[(vec3(-500., -12., -100.), vec3(500., 12., 0.))]);
        assert!(entrance(&ledge, Vec3::Z * 0.04, |_| true).is_none());
        let world = World::fixture(&[
            (vec3(-20., -20., -100.), vec3(20., 20., 0.)),
            (vec3(30., -500., -100.), vec3(500., 500., 0.)),
            (vec3(22., -500., -10.), vec3(28., 500., 100.)),
        ]);
        assert!(entrance(&world, Vec3::Z * 0.04, |_| true).is_none());
    }
}
