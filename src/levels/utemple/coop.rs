//! One shared temple clock, with stable actors for breathing and crushing damage.
use super::*;
use crate::level::CoopPlayer;

#[derive(Clone, Default, Serialize, Deserialize)]
pub(super) struct State {
    cooldowns: [f32; 2],
}
impl State {
    pub fn respawned(&mut self, player: usize) {
        if let Some(cooldown) = self.cooldowns.get_mut(player) { *cooldown = 0.; }
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(self.cooldowns.iter().all(|t| t.is_finite() && (0. ..=0.5).contains(t)),
            "Invalid personal temple crush cooldown");
        Ok(())
    }
}
type Previous = (BTreeMap<String, f32>, [Option<f64>; 4]);

impl Temple {
    /// Shared by single-player and co-op: never advance the guide or traps per actor.
    pub(super) fn advance_clock(&mut self, dt: f32, map: &Bsp, world: &mut World,
        fixed: &[Collider]) -> Result<Previous> {
        self.saved.clock = (self.saved.clock + dt as f64).min(86400.);
        self.saved.crush_cooldown = (self.saved.crush_cooldown - dt).max(0.);
        if let Some(start) = self.saved.guide {
            let parameter = self.data.guide.curve.parameter((self.saved.clock-start) as f32);
            let reached = (parameter.floor() as isize + 1).clamp(0,159) as usize;
            if reached >= self.saved.node {
                for i in self.saved.node..=reached {
                    let name = self.data.guide.nodes[i].1.clone();
                    if !name.is_empty() { self.fire(&name); }
                }
                self.saved.node = reached+1;
            }
        }
        let old_clams = self.saved.clams;
        for n in 0..4 {
            self.saved.pulse[n] = false;
            if let Some(t) = &mut self.saved.clams[n] {
                let old = *t;
                *t = (*t + dt as f64).min(86400.);
                self.saved.pulse[n] = *t >= 1.
                    && (old < 1. || ((*t-1.)/4.91).floor() > ((old-1.)/4.91).floor());
            }
        }
        if let Some(t) = &mut self.saved.fishhead {
            *t += dt;
            if *t >= 1.5 { self.saved.fishhead = None; }
        }
        let old = self.saved.movers.clone();
        for t in self.saved.movers.values_mut() { *t = (*t + dt).min(60.); }
        self.motion.rebuild(map, &self.saved)?;
        world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        world.set_dynamic_liquids(self.liquids());
        Ok((old, old_clams))
    }

    pub(super) fn prepare_roster(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        if self.saved.coop.is_none() {
            if !self.saved.initialized {
                for body in players.iter_mut().filter(|b| b.stats.alive()) { body.stats.restore(); }
            }
            self.saved.initialized = true;
            self.saved.coop = Some(State { cooldowns: [self.saved.crush_cooldown; 2] });
        }
        for body in players.iter_mut().filter(|b| b.stats.alive()) {
            body.stats.arrival_grants("utemple");
            body.stats.prepare_player(body.player);
        }
    }
    pub(super) fn advance_roster(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [CoopPlayer<'_>; 2], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. { return Ok(()); }
        let dt = dt.min(0.1);
        self.prepare_roster(players);
        let (old, old_clams) = self.advance_clock(dt, map, world, fixed)?;
        let original = players.each_ref().map(|b| b.player.feet);
        let alive = players.each_ref().map(|b| b.stats.alive());
        let mut proposed = original;
        let mut damage = [0_f32; 2];
        let mut rollback = false;
        for index in 0..2 {
            if !alive[index] || self.scripted() || world.body_clear(original[index]) { continue; }
            let at = original[index];
            damage[index] = self.motion.objects.iter().filter(|o| {
                o.collider.touches(at+PLAYER_CENTER, at+PLAYER_CENTER, PLAYER_HALF)
                    && (self.saved.movers.contains_key(&o.name) || o.name.starts_with("oyster_block"))
            }).map(|o| o.damage).fold(0., f32::max);
            let mut clear = None;
            'escape: for radius in [2.,4.,8.,16.,32.,64.] {
                for direction in [Vec3::Z,Vec3::NEG_Z,Vec3::X,Vec3::NEG_X,Vec3::Y,Vec3::NEG_Y] {
                    let q = at + direction*radius;
                    let other = proposed[1-index];
                    let separated = !alive[1-index] || (q.z-other.z).abs() >= PLAYER_HALF.z*2.
                        || q.truncate().distance(other.truncate()) >= crate::coop::separation::DISTANCE;
                    if separated && world.body_clear(q) && self.data.world.body_trace(at,q).fraction == 1. {
                        clear = Some(q); break 'escape;
                    }
                }
            }
            if let Some(q) = clear { proposed[index] = q; } else { rollback = true; }
        }
        // A blocked swimmer vetoes the same shared motion for both actors. Never
        // leave the first actor displaced by a transform the second rolled back.
        if rollback {
            proposed = original;
            self.saved.movers = old;
            self.saved.clams = old_clams;
            self.saved.pulse = [false; 4];
            self.motion.rebuild(map, &self.saved)?;
            world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        }
        self.emit();
        for (index, body) in players.iter_mut().enumerate() {
            let cooldown = &mut self.saved.coop.as_mut().unwrap().cooldowns[index];
            *cooldown = (*cooldown-dt).max(0.);
            if !alive[index] { continue; }
            body.player.feet = proposed[index];
            if damage[index] > 0. && *cooldown == 0. {
                body.stats.damage(damage[index]);
                *cooldown = 0.5;
            }
            self.breathe(body.player);
        }
        Ok(())
    }
}
