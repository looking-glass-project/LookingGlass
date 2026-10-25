//! Input-only 3-D navigation for the temple route check. No player or world mutation.
use super::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};
type Cell = (i16, i16, i16);
pub(super) fn waypoint(world: &World, from: Vec3, to: Vec3, hazards: &[Collider]) -> Vec3 {
    let safe = |p: Vec3| {
        !hazards
            .iter()
            .any(|h| h.touches(p + PLAYER_CENTER, p + PLAYER_CENTER, PLAYER_HALF))
    };
    let clear = |a: Vec3, b: Vec3| world.body_trace(a, b).fraction == 1.;
    let future_clear = |a: Vec3, b: Vec3| {
        !hazards
            .iter()
            .any(|h| h.touches(a + PLAYER_CENTER, b + PLAYER_CENTER, PLAYER_HALF))
    };
    if clear(from, to) && future_clear(from, to) {
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
                    // A clear endpoint can lie on the far side of a thin future
                    // obstacle. Price the whole swept body segment, as we do
                    // for the direct route, while still allowing an escape
                    // when the current body is already inside a forecast hull.
                    let next = cost[&c] + p.distance(q)
                        + if safe(q) && future_clear(p,q) { 0. } else { 500. };
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
        let world = World::fixture(&[]);
        let hazard = Collider::fixture(vec3(23.,-30.,-30.),vec3(25.,30.,90.));
        let from = Vec3::ZERO;
        let to = vec3(240.,0.,0.);
        let next = waypoint(&world,from,to,std::slice::from_ref(&hazard));
        assert!(next.distance(from)>1., "Planner did not find the open detour");
        assert!(!hazard.touches(from+PLAYER_CENTER,next+PLAYER_CENTER,PLAYER_HALF),
            "First swim leg cuts through the future trap: {next:?}");
    }
}
