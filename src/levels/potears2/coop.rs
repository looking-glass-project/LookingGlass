//! Explicit host adapters; single-player saves keep their original fish state.
use super::*;
use crate::{collision::{PLAYER_CENTER, PLAYER_HALF}, level::CoopPlayer};

#[derive(Clone, Default, Serialize, Deserialize)]
pub(super) struct Contacts {
    pub players: [fish::State; 2],
    pub owner: Option<usize>,
}
impl Contacts {
    pub fn validate(&self, active: &fish::State) -> Result<()> {
        for state in &self.players {
            state.validate()?;
            ensure!(state.attack.is_none() && !state.killed, "Personal pond clock contains a shared scene");
        }
        ensure!(self.owner.is_none_or(|i| i < 2)
            && self.owner.is_some() == active.attack.is_some(), "Invalid paired fish scene owner");
        Ok(())
    }
}

impl PoolTwo {
    pub(super) fn advance_pair(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [&mut Player], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. { return Ok(()); }
        let dt = dt.min(0.1);
        if self.saved.guards.iter().all(|g| g.health == 0.) {
            self.saved.gate_time = (self.saved.gate_time + dt).min(2.);
        }
        self.rebuild(map)?;
        let pond_fixed = fixed.iter().cloned().chain(self.colliders.iter().cloned()).collect::<Vec<_>>();
        self.pond.advance_players(dt, map, &mut self.saved.pond, world, players, &pond_fixed)?;
        world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        Ok(())
    }

    pub(super) fn fish_contacts(&mut self, dt: f32, players: &mut [CoopPlayer<'_>; 2]) -> Option<usize> {
        if !dt.is_finite() || dt <= 0. || self.saved.scene.phase != Phase::Dormant
            || self.saved.fish.attack.is_some() { return None; }
        let contacts = self.saved.coop.get_or_insert_with(Contacts::default);
        let mut first: Option<(usize, f64, fish::State)> = None;
        for (index, body) in players.iter_mut().enumerate() {
            let state = &mut contacts.players[index];
            if !body.stats.alive() { *state = fish::State::default(); continue; }
            let inside = self.pond.fish.touches(body.player.feet + PLAYER_CENTER,
                body.player.feet + PLAYER_CENTER, PLAYER_HALF);
            let prior = state.exposure;
            state.step(dt, inside, body.player);
            if state.attack.is_some() {
                // Earliest threshold crossing wins; Blue breaks an exact tie.
                // A losing contact remains personal and may trigger later.
                if first.as_ref().is_none_or(|(_, exposure, _)| prior > *exposure) {
                    first = Some((index, prior, state.clone()));
                }
                state.attack = None;
            }
        }
        let (owner, _, state) = first?;
        contacts.owner = Some(owner);
        self.saved.fish = state;
        Some(owner)
    }
}
