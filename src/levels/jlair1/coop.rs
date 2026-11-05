//! Personal arrival/healing and one atomic sinking-platform update for the pair.
use super::*;
use crate::level::CoopPlayer;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub essence_taken: [[bool; 2]; 2],
    descending: bool,
}

/// Read-only contact evidence for the local two-client route fixture.
#[derive(Serialize)]
pub(crate) struct PlatformReport {
    pub sink: f32,
    pub descending: bool,
    pub riders: [bool; 2],
    pub full: [bool; 2],
}

pub(crate) fn route_platform(ctl: &dyn LevelController, players: [&Player; 2]) -> Option<PlatformReport> {
    let o = ctl.downcast_ref::<Curiosity>()?;
    Some(PlatformReport {
        sink: o.saved.sink,
        descending: o.saved.coop.as_ref().is_some_and(|s| s.descending),
        riders: players.map(|p| o.sink_contact(p, false)),
        full: players.map(|p| o.sink_contact(p, true)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boarding_requires_the_whole_footprint_and_actual_ground_contact() {
        let o = Curiosity {
            saved: Saved::default(), points: BTreeMap::new(), paths: BTreeMap::new(),
            sink_base: Vec3::ZERO, essence: [Vec3::ZERO;2],
            sink_shape: Collider::box_bounds(vec3(-80.,-80.,-20.),vec3(80.,80.,0.)),
        };
        let mut p = Player::new(vec3(0.,0.,0.03125));
        assert!(o.sink_contact(&p,true));
        p.feet.x = 75.;
        assert!(o.sink_contact(&p,false), "Edge should still have ordinary body contact");
        assert!(!o.sink_contact(&p,true), "Partly supported body must not start departure");
        p.feet.x = 0.; p.velocity.z = 2.;
        assert!(!o.sink_contact(&p,true), "Jumping player is not aboard");
        p.velocity.z = 0.; p.feet.z = 12.; p.grounded = true;
        assert!(!o.sink_contact(&p,true), "A grounded flag cannot turn a floating body into a rider");
    }
}

impl Curiosity {
    pub(super) fn prepare_pair(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        self.saved.coop.get_or_insert(State {
            // Existing visits keep their consumption and never gain a refill.
            essence_taken: [self.saved.essence_taken; 2],
            descending: false,
        });
        if !self.saved.initialized {
            for p in players.iter_mut().filter(|p| p.stats.alive()) {
                p.stats.full_stats();
            }
            // A dead arrival keeps the normal low-health revival afterward.
            self.saved.initialized = true;
        }
    }

    pub(super) fn pickups_pair(&mut self, dt: f32, players: &mut [CoopPlayer<'_>; 2]) {
        if !dt.is_finite() || dt <= 0. || self.scripted() { return; }
        self.prepare_pair(players);
        if !self.saved.essence_live { return; }
        let taken = &mut self.saved.coop.as_mut().unwrap().essence_taken;
        for (slot, p) in players.iter_mut().enumerate().filter(|(_, p)| p.stats.alive()) {
            for (i, at) in self.essence.iter().enumerate() {
                if !taken[slot][i] && (p.player.feet + PLAYER_CENTER).distance(*at) < 42.
                    && p.stats.apply(crate::inventory::PickupKind::Essence, 15.) {
                    taken[slot][i] = true;
                }
            }
        }
    }

    pub(super) fn essence_visible(&self, pickup: usize, local: Option<usize>) -> bool {
        let taken = local.filter(|s| *s < 2).and_then(|s| self.saved.coop.as_ref().map(|c| c.essence_taken[s]))
            .unwrap_or(self.saved.essence_taken);
        self.saved.essence_live && !taken[pickup]
    }

    fn sink_contact(&self, p: &Player, full: bool) -> bool {
        if p.velocity.z > 1. { return false; }
        let from = p.feet + PLAYER_CENTER;
        let body = self.sink_shape.trace(from, from - Vec3::Z * 3., PLAYER_HALF);
        if body.start_solid || body.fraction >= 1. || body.normal.z <= 0.65 { return false; }
        if !full { return true; }
        // A body's contact height is above the centre of a sloped deck. Check
        // the footprint around the actual surface plane, not a flat 3-unit ray.
        let start = p.feet + Vec3::Z;
        let centre = self.sink_shape.trace(start, start - Vec3::Z * 41., Vec3::ZERO);
        if centre.start_solid || centre.fraction >= 1. || centre.normal.z <= 0.65 { return false; }
        let ground = start - Vec3::Z * (41. * centre.fraction);
        [-PLAYER_HALF.x, 0., PLAYER_HALF.x].into_iter().all(|x|
            [-PLAYER_HALF.y, 0., PLAYER_HALF.y].into_iter().all(|y| {
                let at = ground + vec3(x,y,-(centre.normal.x*x + centre.normal.y*y)/centre.normal.z);
                let tr = self.sink_shape.trace(at+Vec3::Z*8., at-Vec3::Z*8., Vec3::ZERO);
                !tr.start_solid && tr.fraction < 1. && tr.normal.z > 0.65
            }))
    }

    pub(super) fn move_sink_pair(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [CoopPlayer<'_>; 2], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. || self.scripted() || !players.iter().any(|p| p.stats.alive()) {
            return Ok(());
        }
        self.prepare_pair(players);
        let alive = players.each_ref().map(|p| p.stats.alive());
        let riders = std::array::from_fn::<_, 2, _>(|i| alive[i] && self.sink_contact(players[i].player, false));
        let occupied = riders.iter().any(|r| *r);
        let ready = players.iter().enumerate().all(|(i, p)| !alive[i] || self.sink_contact(p.player, true));
        let old = self.saved.sink;
        let old_descending = self.saved.coop.as_ref().unwrap().descending;
        let descending = occupied && (old_descending || ready);
        self.saved.coop.as_mut().unwrap().descending = descending;
        let speed = if descending { 100. } else if occupied { 0. } else { -100. };
        self.saved.sink = (old + dt.min(0.1) * speed).clamp(0., 128.);
        if old == self.saved.sink { return Ok(()); }
        self.rebuild(map)?;
        let mut proposed = players.each_ref().map(|p| p.player.feet);
        let mut blocked = false;
        for (slot, p) in players.iter().enumerate().filter(|(_, p)| p.stats.alive()) {
            let mut carry = if riders[slot] { Vec3::Z * (old - self.saved.sink) } else { Vec3::ZERO };
            world.set_dynamic(fixed.to_vec());
            let tr = world.body_trace(p.player.feet, p.player.feet + carry);
            // As in single player, a descending rider lands on an uneven bed.
            if carry.z < 0. && !tr.start_solid { carry *= tr.fraction; }
            proposed[slot] += carry;
            world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
            blocked |= tr.start_solid || (carry.z > 0. && tr.fraction < 1.) || !world.body_clear(proposed[slot]);
        }
        if blocked {
            self.saved.sink = old;
            self.saved.coop.as_mut().unwrap().descending = old_descending;
            self.rebuild(map)?;
            world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        } else {
            for (slot, p) in players.iter_mut().enumerate().filter(|(_, p)| p.stats.alive()) {
                p.player.feet = proposed[slot];
            }
        }
        Ok(())
    }
}
