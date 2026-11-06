//! Shared tank machinery with personal current cooldowns and blade contacts.
use super::*;
use crate::level::CoopPlayer;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub currents: [[f32; 4]; 2],
    pub crush: [bool; 2],
}
impl State {
    pub fn validate(&self) -> Result<()> {
        for timer in self.currents.iter().flatten() {
            state::clock("Paired Tower current cooldown", *timer, 0.2)?;
        }
        Ok(())
    }
}
impl Tower {
    fn pair_state(&mut self) -> &mut State {
        if self.saved.coop.is_none() {
            let m = self.saved.machine.as_mut().unwrap();
            self.saved.coop = Some(State {
                currents: [m.currents; 2],
                // An imported pending solo hit belongs to the original Alice.
                crush: [std::mem::take(&mut m.crush), false],
            });
        }
        self.saved.coop.as_mut().unwrap()
    }
    pub(super) fn prepare_pair(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        self.pair_state();
        for p in players.iter_mut().filter(|p| p.stats.alive()) {
            self.prepare_player(p.stats, p.player);
        }
        self.stopped = players.iter().any(|p| p.stats.alive() && p.stats.powers.stopped > 0.);
    }
    pub(super) fn advance_pair(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [CoopPlayer<'_>; 2], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. || self.scripted()
            || !players.iter().any(|p| p.stats.alive()) {
            return Ok(());
        }
        self.prepare_pair(players);
        let dt = dt.min(0.1);
        self.saved.fade = (self.saved.fade - dt).max(0.);
        let slots = players.iter().enumerate().filter(|(_, p)| p.stats.alive())
            .map(|(i, _)| i).collect::<Vec<_>>();
        let mut living = players.iter_mut().filter(|p| p.stats.alive())
            .map(|p| &mut *p.player).collect::<Vec<_>>();
        let crush = self.move_machinery_players(dt, map, world, &mut living, fixed, true)?;
        for (index, slot) in slots.into_iter().enumerate() {
            self.saved.coop.as_mut().unwrap().crush[slot] |= crush[index];
            Self::apply_currents(&self.currents,
                &mut self.saved.coop.as_mut().unwrap().currents[slot], dt, living[index]);
        }
        // Keep the solo field meaningful without sharing either player's timer.
        self.saved.machine.as_mut().unwrap().currents = self.saved.coop.as_ref().unwrap().currents[0];
        Ok(())
    }
    pub(super) fn damage_pair(&mut self, dt: f32, players: &mut [CoopPlayer<'_>; 2]) -> crate::combat::Feedback {
        let mut feedback = crate::combat::Feedback::default();
        if !dt.is_finite() || dt <= 0. || self.scripted() { return feedback; }
        let state = self.pair_state();
        for (slot, player) in players.iter().enumerate() {
            if std::mem::take(&mut state.crush[slot]) && player.stats.alive() {
                feedback.players[slot].damage = 1000.;
            }
        }
        feedback
    }
}
