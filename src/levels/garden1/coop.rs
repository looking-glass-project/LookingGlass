//! Paired arrival resources and one shared update for Garden's machinery.
use super::*;
use crate::{
    collision::{PLAYER_CENTER, PLAYER_HALF},
    level::CoopPlayer,
};

impl Garden {
    pub(super) fn prepare_roster(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        self.saved.world.enable_coop();
        if !self.saved.initialized {
            for body in players.iter_mut().filter(|b| b.stats.alive()) {
                body.stats.apply(crate::inventory::PickupKind::Sanity, 100.);
                body.stats.apply(crate::inventory::PickupKind::Will, 100.);
            }
            self.saved.initialized = true;
        }
        for body in players.iter_mut().filter(|b| b.stats.alive()) {
            if self.saved.shell {
                body.stats.turtle_air = true;
            }
            body.player.breath.shell = body.stats.turtle_air;
        }
    }
    pub(super) fn advance_roster(
        &mut self,
        dt: f32,
        map: &Bsp,
        world: &mut World,
        players: &mut [CoopPlayer<'_>; 2],
        fixed: &[Collider],
    ) -> Result<()> {
        if !dt.is_finite() || dt <= 0. {
            return Ok(());
        }
        let dt = dt.min(0.1);
        self.prepare_roster(players);
        self.saved.fade = (self.saved.fade - dt).max(0.);
        let portals: Vec<_> = self
            .data
            .portals
            .iter()
            .filter(|_| self.saved.portals)
            .map(|(_, _, c)| c.clone())
            .collect();
        let mut living = players
            .iter_mut()
            .filter(|b| b.stats.alive())
            .map(|b| &mut *b.player)
            .collect::<Vec<_>>();
        self.movers.carry_players(
            dt,
            map,
            &mut self.saved.world,
            world,
            &mut living,
            fixed,
            &portals,
        )?;
        self.traversal(&mut world.traversal);
        Ok(())
    }
}

/// Attached bodies may obstruct a support, but must keep their rope/ledge anchor.
pub(super) fn carry(
    world: &World,
    previous: &Collider,
    next: &Collider,
    from: Transform,
    to: Transform,
    players: &mut [&mut Player],
) -> bool {
    if players
        .iter()
        .filter(|p| p.rope.is_some() || p.ledge.is_some())
        .any(|p| next.touches(p.feet + PLAYER_CENTER, p.feet + PLAYER_CENTER, PLAYER_HALF))
    {
        return false;
    }
    let mut free = players
        .iter_mut()
        .filter(|p| p.rope.is_none() && p.ledge.is_none())
        .map(|p| &mut **p)
        .collect::<Vec<_>>();
    crate::moving::carry(world, previous, next, from, to, true, &mut free)
}
