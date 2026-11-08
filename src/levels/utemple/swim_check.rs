//! Input-only 3-D navigation for the temple route check. No player or world mutation.
use super::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};
type Cell = (i16, i16, i16);
// The route sends running swim input (220 units/s, acceleration 700). A clear
// line alone does not leave enough room to turn while an async search is pending.
// This is planning clearance only; it never changes a real collision hull.
const BRAKING_DISTANCE: f32 = 220. * 220. / (2. * 700.);
pub(super) fn clear_future_leg(hazards: &[Collider], from: Vec3, to: Vec3) -> bool {
    clear_with(hazards,from,to,PLAYER_HALF+Vec3::splat(BRAKING_DISTANCE))
}
fn clear_with(hazards: &[Collider], from: Vec3, to: Vec3, half: Vec3) -> bool {
    !hazards.iter().any(|h|h.touches(from+PLAYER_CENTER,to+PLAYER_CENTER,half))
}
fn steering_clear(world: &World, player: &Player, goal: Vec3, hazards: &[Collider]) -> bool {
    // A forecast can already surround the current body; allow an escape from
    // those hulls. For all others, check the actual curved steering trajectory,
    // including inertia while a background search retains this goal.
    let checked:Vec<_>=hazards.iter().filter(|h|!h.touches(player.feet+PLAYER_CENTER,
        player.feet+PLAYER_CENTER,PLAYER_HALF)).collect();
    if checked.is_empty() { return true; }
    let mut probe=player.clone();
    // Cover the worker's maximum accepted age (120 ticks), including braking
    // after reaching the waypoint. A half-second check can end just before an
    // unavoidable overshoot and leave the next replan no safe first step.
    for _ in 0..120 {
        let before=probe.feet;
        let offset=goal-before;
        probe.tick(world,crate::movement::Controls {
            swim:offset.normalize_or_zero()*(offset.length()/30.).min(1.),
            run:true,..Default::default()
        });
        if checked.iter().any(|h|h.touches(before+PLAYER_CENTER,probe.feet+PLAYER_CENTER,PLAYER_HALF)) {
            return false;
        }
    }
    true
}
pub(super) fn waypoint(world: &World, player: &Player, to: Vec3, hazards: &[Collider]) -> Vec3 {
    let from=player.feet;
    let safe = |p: Vec3| clear_future_leg(hazards,p,p);
    let clear = |a: Vec3, b: Vec3| world.body_trace(a, b).fraction == 1.;
    let future_clear = |a: Vec3, b: Vec3| clear_future_leg(hazards,a,b);
    if clear(from, to) && future_clear(from, to) && steering_clear(world,player,to,hazards) {
        return to;
    }
    let step = 48.;
    let point = |c: Cell| from + vec3(c.0 as f32, c.1 as f32, c.2 as f32) * step;
    let mut queue = BinaryHeap::new();
    let mut cost = HashMap::new();
    let mut parent = HashMap::new();
    let origin = (0, 0, 0);
    cost.insert(origin, 0.);
    queue.push((Reverse(0), origin));
    let (mut best, mut distance) = (origin, from.distance(to));
    for _ in 0..6000 {
        let Some((_, c)) = queue.pop() else { break };
        let p = point(c);
        if p.distance(to) < distance && safe(p) {
            best = c;
            distance = p.distance(to);
        }
        if c != origin && clear(p, to) && future_clear(p, to) {
            best = c;
            break;
        }
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if x == 0 && y == 0 && z == 0 {
                        continue;
                    }
                    let n = (c.0 + x, c.1 + y, c.2 + z);
                    if n.0.abs() > 40 || n.1.abs() > 40 || n.2.abs() > 30 {
                        continue;
                    }
                    let q = point(n);
                    if !clear(p, q) {
                        continue;
                    }
                    if c==origin && !steering_clear(world,player,q,hazards) {
                        continue;
                    }
                    // Entering braking clearance is less costly than crossing
                    // the obstacle itself. Keep escape possible when a newly
                    // forecast hull already surrounds the current swimmer.
                    let next = cost[&c] + p.distance(q)
                        + if future_clear(p,q) { 0. }
                            else if clear_with(hazards,p,q,PLAYER_HALF) { 100. }
                            else { 500. };
                    if cost.get(&n).is_some_and(|v| *v <= next) {
                        continue;
                    }
                    cost.insert(n, next);
                    parent.insert(n, c);
                    queue.push((Reverse(((next + q.distance(to) * 1.2) * 100.) as i32), n));
                }
            }
        }
    }
    while let Some(&p) = parent.get(&best) {
        if p == origin {
            break;
        }
        best = p;
    }
    point(best)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn swim_waypoint_does_not_cut_through_a_future_trap() {
        let mut world = World::fixture(&[]);
        world.add_liquid(Vec3::splat(-1000.),Vec3::splat(1000.),32,None);
        let hazard = Collider::fixture(vec3(23.,-30.,-30.),vec3(25.,30.,90.));
        let from = Vec3::ZERO;
        let to = vec3(240.,0.,0.);
        let next = waypoint(&world,&Player::new(from),to,std::slice::from_ref(&hazard));
        assert!(next.distance(from)>1., "Planner did not find the open detour");
        assert!(!hazard.touches(from+PLAYER_CENTER,next+PLAYER_CENTER,PLAYER_HALF),
            "First swim leg cuts through the future trap: {next:?}");
    }
    #[test]
    fn swim_route_leaves_room_to_turn_with_existing_momentum() {
        let mut world = World::fixture(&[]);
        world.add_liquid(Vec3::splat(-1000.),Vec3::splat(1000.),32,None);
        let hazard = Collider::fixture(vec3(40.,20.,-30.),vec3(60.,40.,100.));
        let target=vec3(240.,0.,0.);
        let mut player=Player::new(Vec3::ZERO);
        player.swimming=true;
        player.velocity=Vec3::Y*220.;
        let mut goal=player.feet;
        for tick in 0..360 {
            // A live client can retain the previous goal for half a second
            // while the background route search finishes.
            if tick%60==0 {
                goal=waypoint(&world,&player,target,std::slice::from_ref(&hazard));
            }
            let before=player.feet;
            let offset=goal-before;
            player.tick(&world,crate::movement::Controls {
                swim:offset.normalize_or_zero()*(offset.length()/30.).min(1.),
                run:true,..Default::default()
            });
            assert!(!hazard.touches(before+PLAYER_CENTER,player.feet+PLAYER_CENTER,PLAYER_HALF),
                "Steering crossed a future trap at tick{tick}: {before:?} -> {:?}, goal {goal:?}, velocity {:?}",player.feet,player.velocity);
        }
        assert!(player.feet.distance(target)<16.,"Safe swimmer did not reach its goal: {:?}",player.feet);
    }
}
