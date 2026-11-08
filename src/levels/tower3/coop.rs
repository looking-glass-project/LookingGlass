//! Personal fall checkpoints and damage over one set of machine clocks.
use super::*;
use crate::level::CoopPlayer;
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub room: [u8; 2],
    pub damage: [f32; 2],
    pub hurt_wait: [f32; 2],
}
impl State {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.room.iter().all(|n| (1..=3).contains(n)), "Invalid paired Tower checkpoint");
        ensure!(self.damage.iter().all(|d| [0., 15., 1000.].contains(d)), "Invalid paired Tower damage");
        for t in self.hurt_wait { state::clock("Paired Tower crush wait", t, 0.5)?; }
        Ok(())
    }
}
pub(super) fn room(thread: &str) -> Option<u8> {
    match thread { "Room1Tele" => Some(1), "Room2Tele" => Some(2), "Room3Tele" => Some(3), _ => None }
}
impl Tower {
    pub(super) fn pair_state(&mut self) -> &mut State {
        self.saved.coop.get_or_insert_with(|| State {
            room: [self.saved.room; 2],
            damage: [std::mem::take(&mut self.saved.damage), 0.],
            hurt_wait: [self.saved.hurt_wait, 0.],
        })
    }
    pub(super) fn advance_pair(&mut self, dt: f32, w: &mut World,
        players: &mut [CoopPlayer<'_>; 2], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. || self.scripted()
            || !players.iter().any(|p| p.stats.alive()) { return Ok(()); }
        self.pair_state();
        let dt = dt.min(0.1);
        self.saved.fade = (self.saved.fade - dt).max(0.);
        let slots = players.iter().enumerate().filter(|(_, p)| p.stats.alive())
            .map(|(i, _)| i).collect::<Vec<_>>();
        let mut waits = slots.iter().map(|&i| (self.pair_state().hurt_wait[i] - dt).max(0.)).collect::<Vec<_>>();
        let mut living = players.iter_mut().filter(|p| p.stats.alive())
            .map(|p| &mut *p.player).collect::<Vec<_>>();
        let damage = self.move_players(dt, w, &mut living, fixed, &mut waits)?;
        for (n, slot) in slots.into_iter().enumerate() {
            let s = self.pair_state();
            s.damage[slot] = s.damage[slot].max(damage[n]); s.hurt_wait[slot] = waits[n];
        }
        Ok(())
    }
    pub(super) fn damage_pair(&mut self, dt: f32, players: &mut [CoopPlayer<'_>; 2]) -> crate::combat::Feedback {
        let mut out = crate::combat::Feedback::default();
        if !dt.is_finite() || dt <= 0. || self.scripted() { return out; }
        for (slot, player) in players.iter().enumerate() {
            let damage = std::mem::take(&mut self.pair_state().damage[slot]);
            if player.stats.alive() { out.players[slot].damage = damage; }
        }
        out
    }
}
