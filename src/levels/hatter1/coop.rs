//! One machinery clock and a stable owner for personal lever actions.
use super::*;
use crate::level::CoopPlayer;

pub(super) fn closing_approach(map: &Bsp, thread: &str) -> Result<f32> {
    let e = map.entities.iter().find(|e| e.get("thread").is_some_and(|s| s == thread))
        .context("Missing Clockwork closing contact")?;
    let model: usize = e.get("model").and_then(|s| s.strip_prefix('*'))
        .context("Clockwork closing contact has no brush")?.parse()?;
    let x = at(e).translation.x + map.models.get(model).context("Missing closing contact model")?.max.x;
    ensure!(x.is_finite(), "Invalid Clockwork closing approach");
    Ok(x)
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub initialized: [bool; 2],
    pub actor: Option<usize>,
    pub port_actor: Option<usize>,
}
impl State {
    pub fn validate(&self, saved: &Saved) -> Result<()> {
        ensure!(self.actor.is_none_or(|i| i < 2) && self.port_actor.is_none_or(|i| i < 2)
            && (saved.pull.is_none() || self.actor.is_some()), "Invalid paired Clockwork owner");
        Ok(())
    }
}
impl Clockwork {
    pub(super) fn pair_state(&mut self) -> &mut State {
        self.saved.coop.get_or_insert_with(|| State {
            initialized: [self.saved.initialized; 2],
            actor: self.saved.pull.is_some().then_some(0), port_actor: None,
        })
    }
    pub(super) fn pair_prepare(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        let state = self.pair_state();
        for (i, p) in players.iter_mut().enumerate() {
            if !state.initialized[i] {
                if p.stats.alive() { p.stats.full_stats(); }
                state.initialized[i] = true;
            }
        }
        self.saved.initialized = true;
        if self.saved.coop.as_ref().unwrap().actor.is_some_and(|i| !players[i].stats.alive()) {
            if let Some(pull) = self.saved.pull.take() {
                if pull.started && pull.lever == 4 { self.saved.levers[4] = None; }
            }
            self.saved.coop.as_mut().unwrap().actor = None;
        }
        if self.saved.pull.is_none() && !self.scripted() {
            self.saved.coop.as_mut().unwrap().actor = None;
        }
        if self.saved.levers[3].is_none() && self.saved.pull.as_ref().is_none_or(|p| p.lever != 3) {
            self.saved.coop.as_mut().unwrap().port_actor = None;
        }
    }
    pub(super) fn pair_actor(&mut self, players: &mut [CoopPlayer<'_>; 2]) -> Option<usize> {
        if !self.scripted() { return None; }
        let actor = self.pair_state().actor.filter(|&i| players[i].stats.alive())
            .or_else(|| players.iter().position(|p| p.stats.alive()));
        self.pair_state().actor = actor;
        actor
    }
    pub(super) fn pair_advance(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [CoopPlayer<'_>; 2], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. { return Ok(()); }
        self.pair_prepare(players);
        if self.scripted() || !players.iter().any(|p| p.stats.alive()) { return Ok(()); }
        let dt = dt.min(0.1);
        {
            let mut living = players.iter_mut().filter(|p| p.stats.alive())
                .map(|p| &mut *p.player).collect::<Vec<_>>();
            self.advance_pair_motion(dt, map, world, &mut living, fixed)?;
        }
        if let Some(i) = self.saved.coop.as_ref().unwrap().actor {
            // A lever approach is ordinary personal movement. The partner
            // remains free to play and must not become a path through a body.
            let other = &players[1 - i];
            if other.stats.alive() {
                let center = other.player.feet + PLAYER_CENTER;
                let half = vec3(crate::coop::separation::DISTANCE - PLAYER_HALF.x,
                    crate::coop::separation::DISTANCE - PLAYER_HALF.y, PLAYER_HALF.z);
                world.set_dynamic(fixed.iter().cloned().chain(self.colliders())
                    .chain(std::iter::once(Collider::box_bounds(center - half, center + half))).collect());
            }
            self.advance_pull(dt, world, players[i].player);
            world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        }
        if self.saved.port.is_none() && self.saved.scene.is_none()
            && self.saved.levers[3].is_some_and(|t| self.saved.age >= t + 4.) {
            let actor = self.saved.coop.as_ref().unwrap().port_actor.filter(|&i| players[i].stats.alive())
                .or_else(|| players.iter().position(|p| p.stats.alive()));
            self.begin(Kind::Port);
            self.saved.coop.as_mut().unwrap().actor = actor;
        }
        // Stage binds a newly cinematic actor before its first scene tick.
        // Ordinary speech keeps both players exploring and advances only once.
        if !self.scripted() {
            if let Some(p) = players.iter_mut().find(|p| p.stats.alive()) {
                self.advance_scene(dt, world, p.player)?;
            }
        }
        self.rebuild(map)?;
        world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        Ok(())
    }
}
