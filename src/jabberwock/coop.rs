//! One boss/wave clock, stable target slots and personal healing in both arenas.
use super::*;
use crate::{combat::PlayerTarget, level::CoopPlayer};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub boss_player: Option<usize>,
    pub wave_players: [Option<usize>; 6],
    pub essence: [usize; 2],
    pub wait: [f32; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world: Option<coop_world::State>,
}
impl State {
    pub fn validate(&self, kind: Kind) -> Result<()> {
        ensure!(
            self.boss_player.is_none_or(|i| i < 2)
                && self.wave_players.iter().all(|p| p.is_none_or(|i| i < 2))
                && self
                    .essence
                    .iter()
                    .all(|i| *i < if kind == Kind::Lair { 2 } else { 3 })
                && self
                    .wait
                    .iter()
                    .all(|t| t.is_finite() && (0. ..=8.).contains(t)),
            "Invalid paired Jabberwock combat/resources"
        );
        Ok(())
    }
}
impl Encounter {
    pub(super) fn pair_state(&mut self) -> &mut State {
        self.saved.coop.get_or_insert_with(|| State {
            boss_player: None,
            wave_players: [None; 6],
            // Import the current cycle and remaining cooldown; joining is not a refill.
            essence: [self.saved.essence; 2],
            wait: [self.saved.essence_wait; 2],
            world: None,
        })
    }
    pub(super) fn personal_essence(&self, local: Option<usize>) -> Option<Vec3> {
        if self.saved.phase != Phase::Fight {
            return None;
        }
        let (index, wait) = local
            .filter(|i| *i < 2)
            .and_then(|i| self.saved.coop.as_ref().map(|s| (s.essence[i], s.wait[i])))
            .unwrap_or((self.saved.essence, self.saved.essence_wait));
        (wait == 0.).then(|| {
            self.data.points[if self.saved.kind == Kind::Lair {
                ["help_me1", "help_me2"][index]
            } else {
                ["get_me3", "get_me1", "get_me2"][index]
            }]
            .translation
        })
    }
    pub(super) fn pair_resources(&mut self, dt: f32, players: &mut [CoopPlayer<'_>; 2]) {
        if !dt.is_finite()
            || dt <= 0.
            || self.saved.phase != Phase::Fight
            || !players.iter().any(|p| p.stats.alive())
        {
            return;
        }
        self.pair_state();
        let dt = dt.min(0.1);
        for wait in &mut self.saved.coop.as_mut().unwrap().wait {
            *wait = (*wait - dt).max(0.);
        }
        for (slot, p) in players
            .iter_mut()
            .enumerate()
            .filter(|(_, p)| p.stats.alive())
        {
            let Some(at) = self.personal_essence(Some(slot)) else {
                continue;
            };
            let delta = (p.player.feet + Vec3::Z * 32. - at).abs();
            if (p.stats.sanity() < 100. || p.stats.will() < 100.)
                && delta.x < 36.
                && delta.y < 36.
                && delta.z < 64.
            {
                p.stats.essence(50.);
                let state = self.saved.coop.as_mut().unwrap();
                state.essence[slot] =
                    (state.essence[slot] + 1) % if self.saved.kind == Kind::Lair { 2 } else { 3 };
                state.wait[slot] = 8.;
            }
        }
    }
    pub(super) fn pair_fight(
        &mut self,
        dt: f32,
        world: &World,
        players: &mut [CoopPlayer<'_>; 2],
    ) -> Feedback {
        if !dt.is_finite()
            || dt <= 0.
            || self.saved.phase != Phase::Fight
            || !players.iter().any(|p| p.stats.alive())
        {
            return Feedback::default();
        }
        self.pair_state();
        let targets = players.each_ref().map(|p| PlayerTarget {
            eye: p.player.eye(),
            alive: p.stats.alive(),
            hidden: p.stats.ignores_alice(),
        });
        let stopped = players
            .iter()
            .any(|p| p.stats.alive() && p.stats.powers.stopped > 0.);
        self.fight(
            dt,
            world,
            targets[0].eye,
            targets.iter().all(|p| !p.alive || p.hidden),
            stopped,
            Some(targets),
        )
    }
}
