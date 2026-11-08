//! One shared world tick, boarding stops and an atomic carry for living riders.
use super::*;
use crate::{
    collision::{PLAYER_CENTER, PLAYER_HALF},
    level::CoopPlayer,
};

impl Encounter {
    pub(super) fn pair_scene(&mut self, owner: usize, player: &Player, stats: &Stats) {
        self.alive = stats.alive();
        self.stopped = stats.powers.stopped > 0.;
        self.pair_bind_scene(owner, player);
    }
    pub(super) fn pair_bind_scene(&mut self, owner: usize, player: &Player) {
        let state = self.saved.coop.get_or_insert_with(Default::default);
        if state.actor != Some(owner) {
            self.saved.scene_from = Transform {
                translation: player.feet,
                rotation: Quat::from_rotation_z(player.script_facing),
            };
            state.actor = Some(owner);
        }
    }
    fn lift_rider(&self, map: &Bsp, k: usize, player: &Player) -> Result<bool> {
        if player.velocity.z > 1. {
            return Ok(false);
        }
        let (_, p, q) = self.poses[k];
        let hull = self.brush_collider(map, &self.data.brushes[k], p, q)?;
        let trace = hull.trace(
            player.feet + PLAYER_CENTER,
            player.feet + PLAYER_CENTER - Vec3::Z * 3.,
            vec3(1., 1., PLAYER_HALF.z),
        );
        Ok(!trace.start_solid && trace.fraction < 1. && trace.normal.z > 0.65)
    }
    fn lifts_ready(&self, map: &Bsp, world: &World, players: &[&Player]) -> Result<bool> {
        // At each departure a rider waits for the pair. A partner already on
        // the nearby destination landing need not ride back down to collect them.
        let mut occupied = false;
        for (k, b) in self
            .data
            .brushes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.name.starts_with("end_plat"))
        {
            let contacts = players
                .iter()
                .map(|p| self.lift_rider(map, k, p))
                .collect::<Result<Vec<_>>>()?;
            if !contacts.iter().any(|r| *r) {
                continue;
            }
            occupied = true;
            let up = (b.name == "end_plat1") == (self.saved.lifts < 2.);
            let centre = (b.min + b.max) * 0.5 + self.poses[k].1;
            let riders = players.iter().zip(contacts).map(|(p, contact)| {
                contact
                    && (p.feet - centre)
                        .truncate()
                        .abs()
                        .cmple(Vec2::splat(56.) - PLAYER_HALF.truncate())
                        .all()
            });
            let landing_z = if up { 400. } else { 8. };
            if !players.iter().zip(riders).all(|(p, rider)| {
                rider
                    || ((p.feet.z - landing_z).abs() <= 12.
                        && (p.feet - centre).truncate().length() <= 256.
                        && crate::recovery::supported(world, p.feet))
            }) {
                return Ok(false);
            }
        }
        // Keep the lower starting deck available. Empty upper decks return.
        Ok(occupied || self.saved.lifts >= 4.)
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
        self.pair_prepare(players);
        self.saved.coop.as_mut().unwrap().actor = None;
        let dt = dt.min(0.1);
        self.saved.exit.advance(dt);
        if self.stopped {
            return Ok(());
        }
        let old_bridge = self.saved.bridge;
        let old_lifts = self.saved.lifts;
        let old_door = self.saved.door;
        let old_poses = self.poses.clone();
        let old_shapes = self.shapes.clone();
        self.saved.clock = (self.saved.clock + dt).min(86400.);
        self.saved.time = (self.saved.time + dt).min(3600.);
        if let Some(t) = &mut self.saved.bridge {
            *t = (*t + dt).min(1.);
        }
        if self.won() {
            let at_stop = self.saved.lifts == 0. || self.saved.lifts == 4.;
            let living = players
                .iter()
                .filter(|p| p.stats.alive())
                .map(|p| &*p.player)
                .collect::<Vec<_>>();
            if !at_stop || self.lifts_ready(map, world, &living)? {
                let next = self.saved.lifts + dt;
                self.saved.lifts = if self.saved.lifts < 4. && next >= 4. {
                    4.
                } else if next >= 8. {
                    0.
                } else {
                    next
                };
            }
        }
        if self.saved.phase == Phase::ExitReady {
            self.saved.door = (self.saved.door + dt).min(1.4);
        }
        self.pair_item_clock(dt);
        if self.saved.talk_done {
            self.saved.after_talk = (self.saved.after_talk + dt).min(3600.);
        }
        if matches!(self.saved.phase, Phase::Cat | Phase::WatchCat)
            && self.saved.talk_done
            && self.saved.after_talk >= 3.
        {
            self.finish_cat();
        }
        self.rebuild(map)?;
        let mut proposed = players.each_ref().map(|p| p.player.feet);
        let mut supports = [None; 2];
        for (slot, p) in players.iter().enumerate().filter(|(_, p)| p.stats.alive()) {
            for (k, b) in self.data.brushes.iter().enumerate() {
                if !matches!(b.name.as_str(), "end_plat1" | "end_plat2" | "drawbridge") {
                    continue;
                }
                let old = old_poses[k];
                let next = self.poses[k];
                if old == next {
                    continue;
                }
                let hull = self.brush_collider(map, b, old.1, old.2)?;
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
                    proposed[slot] = next.1 + next.2 * (old.2.inverse() * (p.player.feet - old.1));
                    supports[slot] = Some(b.index);
                    break;
                }
            }
        }
        let mut blocked = false;
        for (slot, p) in players.iter().enumerate().filter(|(_, p)| p.stats.alive()) {
            let others = self
                .data
                .brushes
                .iter()
                .zip(&self.poses)
                .filter(|(b, pose)| {
                    supports[slot] != Some(b.index)
                        && pose.1.z > -90000.
                        && !b.name.starts_with("circle")
                        && !b.name.starts_with("beam")
                        && !matches!(
                            b.name.as_str(),
                            "clockmin" | "clockhr" | "clock_minute_hand" | "clock_hour_hand"
                        )
                })
                .map(|(b, pose)| self.brush_collider(map, b, pose.1, pose.2))
                .collect::<Result<Vec<_>>>()?;
            world.set_dynamic(fixed.iter().cloned().chain(others).collect());
            let trace = world.body_trace(p.player.feet, proposed[slot]);
            world.set_dynamic(fixed.iter().cloned().chain(self.shapes.clone()).collect());
            blocked |=
                trace.start_solid || trace.fraction < 1. || !world.body_clear(proposed[slot]);
        }
        if players.iter().all(|p| p.stats.alive()) {
            let d = proposed[0] - proposed[1];
            blocked |= d.truncate().length() < crate::coop::separation::DISTANCE - 0.05
                && d.z.abs() < PLAYER_HALF.z * 2. - 0.05;
        }
        if blocked {
            self.saved.bridge = old_bridge;
            self.saved.lifts = old_lifts;
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
        world.set_dynamic(fixed.iter().cloned().chain(self.shapes.clone()).collect());
        Ok(())
    }
}
