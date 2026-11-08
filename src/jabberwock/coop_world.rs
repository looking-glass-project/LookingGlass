//! Shared victory progression and atomic brush movement, with personal Eye Staff copies.
use super::*;
use crate::{
    collision::{PLAYER_CENTER, PLAYER_HALF},
    level::CoopPlayer,
};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub eye: [bool; 2],
    pub actor: Option<usize>,
    pub pending_scene: Option<usize>,
}
impl State {
    pub fn validate(&self, saved: &Saved) -> Result<()> {
        ensure!(
            self.actor.is_none_or(|i| i < 2)
                && self.pending_scene.is_none_or(|i| i < 2
                    && saved.kind == Kind::Grounds
                    && saved.phase == Phase::Outro)
                && (!self.eye.iter().any(|taken| *taken)
                    || (saved.kind == Kind::Lair
                        && matches!(saved.phase, Phase::Reward | Phase::Done)))
                && saved.eye == (saved.kind == Kind::Lair && self.eye.iter().all(|taken| *taken)),
            "Invalid paired Jabberwock progression"
        );
        Ok(())
    }
}
impl Encounter {
    pub(super) fn pair_world(&mut self) -> &mut State {
        let eye = self.saved.eye;
        self.pair_state().world.get_or_insert_with(|| State {
            // Existing completed solo/older paired visits already granted the reward.
            eye: [eye; 2],
            actor: None,
            pending_scene: None,
        })
    }
    pub(super) fn personal_eye_visible(&self, local: Option<usize>) -> bool {
        let Some((slot, state)) = local
            .filter(|i| *i < 2)
            .zip(self.saved.coop.as_ref().and_then(|s| s.world.as_ref()))
        else {
            return self.eye_visible();
        };
        self.saved.kind == Kind::Lair
            && !state.eye[slot]
            && (self.saved.phase == Phase::Reward
                || (self.saved.phase == Phase::Outro && self.saved.time >= 4.55))
    }
    pub(super) fn pair_contacts(
        &mut self,
        dt: f32,
        players: &mut [CoopPlayer<'_>; 2],
    ) -> Option<usize> {
        if !dt.is_finite() || dt <= 0. || !players.iter().any(|p| p.stats.alive()) {
            return None;
        }
        self.pair_world();
        if self.saved.phase == Phase::Reward {
            for (slot, p) in players
                .iter_mut()
                .enumerate()
                .filter(|(_, p)| p.stats.alive())
            {
                if !self
                    .saved
                    .coop
                    .as_ref()
                    .unwrap()
                    .world
                    .as_ref()
                    .unwrap()
                    .eye[slot]
                    && self.data.volumes["pickup_eye_trigger"]
                        .contains(p.player.feet + Vec3::Z * 20.)
                {
                    p.stats.apply(crate::inventory::PickupKind::Weapon(7), 100.);
                    self.saved
                        .coop
                        .as_mut()
                        .unwrap()
                        .world
                        .as_mut()
                        .unwrap()
                        .eye[slot] = true;
                }
            }
            if !self.saved.eye && self.pair_world().eye.iter().all(|taken| *taken) {
                self.saved.eye = true;
                self.saved.time = 0.;
            }
        }
        self.pair_world().pending_scene.take()
    }
    pub(super) fn pair_advance(
        &mut self,
        dt: f32,
        map: &Bsp,
        world: &mut World,
        players: &mut [CoopPlayer<'_>; 2],
        fixed: &[Collider],
    ) -> Result<()> {
        if !dt.is_finite()
            || dt <= 0.
            || self.scripted()
            || !players.iter().any(|p| p.stats.alive())
        {
            return Ok(());
        }
        self.coop_prepare(players);
        self.pair_world().actor = None;
        let dt = dt.min(0.1);
        let old_phase = self.saved.phase;
        let old_time = self.saved.time;
        let old_door = self.saved.door;
        let old_poses = self.poses.clone();
        let old_shapes = self.shapes.clone();
        if self.saved.kind == Kind::Grounds
            && self.saved.phase == Phase::Done
            && players.iter().any(|p| {
                p.stats.alive() && p.player.feet.x > 1250. && (p.player.feet.y - 192.).abs() < 200.
            })
        {
            self.saved.door_requested = true;
        }
        self.advance_clocks(dt);
        self.pair_resources(dt, players);
        match self.saved.phase {
            Phase::Fight if self.saved.kind == Kind::Grounds && self.saved.boss.health == 0. => {
                self.phase(Phase::Death)
            }
            Phase::Death => {
                self.advance_death(dt, world);
                if self.saved.phase == Phase::Outro {
                    let owner = players
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| p.stats.alive())
                        .min_by(|(_, a), (_, b)| {
                            self.saved
                                .boss
                                .at
                                .distance_squared(a.player.feet)
                                .total_cmp(&self.saved.boss.at.distance_squared(b.player.feet))
                        })
                        .unwrap()
                        .0;
                    self.pair_world().pending_scene = Some(owner);
                }
            }
            Phase::Reward if self.saved.eye && self.saved.time >= 2. => {
                self.phase(Phase::Done);
                self.saved.exit.committed = true;
            }
            Phase::Bridge if self.saved.time >= 13. => self.phase(Phase::Done),
            _ => {}
        }
        self.rebuild(map)?;
        if self.poses != old_poses {
            let mut proposed = players.each_ref().map(|p| p.player.feet);
            let mut support = [None; 2];
            for (slot, p) in players.iter().enumerate().filter(|(_, p)| p.stats.alive()) {
                for (old, next) in old_poses.iter().zip(&self.poses) {
                    let (m, at, rotation) = *old;
                    if old == next
                        || !self
                            .data
                            .brushes
                            .iter()
                            .any(|b| b.index == m && b.name == "drawbridge")
                    {
                        continue;
                    }
                    let hull = Collider::model(map, m, at, rotation, true)?;
                    let trace = hull.trace(
                        p.player.feet + PLAYER_CENTER,
                        p.player.feet + PLAYER_CENTER - Vec3::Z * 3.,
                        vec3(1., 1., PLAYER_HALF.z),
                    );
                    if p.player.velocity.z <= 1.
                        && !trace.start_solid
                        && trace.fraction < 1.
                        && trace.normal.z > 0.65
                    {
                        proposed[slot] =
                            next.1 + next.2 * (rotation.inverse() * (p.player.feet - at));
                        support[slot] = Some(m);
                    }
                }
            }
            let mut blocked = false;
            for (slot, p) in players.iter().enumerate().filter(|(_, p)| p.stats.alive()) {
                let other_shapes = self
                    .poses
                    .iter()
                    .filter(|(m, at, _)| Some(*m) != support[slot] && at.z > -90000.)
                    .map(|(m, at, q)| Collider::model(map, *m, *at, *q, true))
                    .collect::<Result<Vec<_>>>()?;
                world.set_dynamic(fixed.iter().cloned().chain(other_shapes).collect());
                let trace = world.body_trace(p.player.feet, proposed[slot]);
                world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
                blocked |=
                    trace.start_solid || trace.fraction < 1. || !world.body_clear(proposed[slot]);
            }
            if players.iter().all(|p| p.stats.alive()) {
                let delta = proposed[0] - proposed[1];
                blocked |= delta.truncate().length() < crate::coop::separation::DISTANCE - 0.05
                    && delta.z.abs() < PLAYER_HALF.z * 2. - 0.05;
            }
            if blocked {
                self.saved.phase = old_phase;
                self.saved.time = old_time;
                self.saved.door = old_door;
                self.poses = old_poses;
                self.shapes = old_shapes;
            } else {
                for (slot, p) in players
                    .iter_mut()
                    .enumerate()
                    .filter(|(_, p)| p.stats.alive())
                {
                    p.player.feet = proposed[slot];
                }
            }
        }
        world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        Ok(())
    }
}
