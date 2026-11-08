//! Shared machines, with crush contacts delivered to their actual victim.
use super::*;
use crate::level::CoopPlayer;
impl Labyrinth {
    pub(super) fn pair_state(&mut self) -> &mut [bool; 2] {
        self.saved.coop.get_or_insert_with(|| [std::mem::take(&mut self.saved.crush), false])
    }
    pub(super) fn advance_pair(&mut self, dt: f32, w: &mut World,
        players: &mut [CoopPlayer<'_>; 2], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. || !players.iter().any(|p| p.stats.alive()) { return Ok(()); }
        self.pair_state();
        let slots = players.iter().enumerate().filter(|(_, p)| p.stats.alive())
            .map(|(i, _)| i).collect::<Vec<_>>();
        let mut living = players.iter_mut().filter(|p| p.stats.alive())
            .map(|p| &mut *p.player).collect::<Vec<_>>();
        let hits = self.move_players(dt.min(0.1), w, &mut living, fixed)?;
        for (slot, hit) in slots.into_iter().zip(hits) { self.pair_state()[slot] |= hit; }
        Ok(())
    }
    pub(super) fn damage_pair(&mut self, dt: f32, players: &mut [CoopPlayer<'_>; 2]) -> crate::combat::Feedback {
        let mut out = crate::combat::Feedback::default();
        if !dt.is_finite() || dt <= 0. { return out; }
        for (slot, player) in players.iter().enumerate() {
            if std::mem::take(&mut self.pair_state()[slot]) && player.stats.alive() {
                out.players[slot].damage = 1000.;
            }
        }
        out
    }
}
