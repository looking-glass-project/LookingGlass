//! Personal arena protection/healing with one shared progression clock.
use super::*;
use crate::{collision::{PLAYER_CENTER, PLAYER_HALF}, level::CoopPlayer};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub arena: [bool; 2],
    pub essence: [usize; 2],
    pub wait: [f32; 2],
    pub previous: [Option<Vec3>; 2],
    pub pending_scene: Option<usize>,
}
impl State {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.essence.iter().all(|i| *i < 3)
            && self.wait.iter().all(|t| t.is_finite() && (0. ..=10.).contains(t))
            && self.pending_scene.is_none_or(|i| i < 2)
            && self.previous.iter().flatten().all(|p| p.is_finite() && p.abs().max_element() < 100000.),
            "Invalid paired Centipede progression");
        Ok(())
    }
}
pub(super) fn arena_trigger(map: &Bsp) -> Result<Collider> {
    let e = map.entities.iter().find(|e| e.get("thread").is_some_and(|s| s == "Centipede2_MinHealth_Reset"))
        .context("Missing arena arrival trigger")?;
    let model = e.get("model").and_then(|s| s.strip_prefix('*')).context("Missing arena trigger model")?.parse()?;
    let origin = e.get("origin").and_then(|s| crate::interaction::vector(s)).unwrap_or_default();
    Collider::model(map, model, origin, Quat::IDENTITY, false)
}
impl Centipede {
    pub(super) fn prepare_pair(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        self.alive = players.iter().any(|p| p.stats.alive());
        let c = self.saved.coop.get_or_insert_with(|| State {
            arena: [!matches!(self.saved.phase, Phase::Intro | Phase::Slide); 2],
            essence: [self.saved.essence; 2], wait: [self.saved.essence_wait; 2],
            previous: [None; 2], pending_scene: None,
        });
        for (i, p) in players.iter_mut().enumerate() {
            p.stats.minimum_sanity = if matches!(self.saved.phase, Phase::Slide | Phase::Fight) && !c.arena[i] {30.} else {0.};
        }
    }
    pub(super) fn advance_pair(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [CoopPlayer<'_>; 2]) -> Result<()> {
        self.prepare_pair(players);
        if !dt.is_finite() || dt <= 0. || !self.alive || self.scripted() { return Ok(()); }
        let dt = dt.min(0.1);
        self.saved.clock = (self.saved.clock + dt).min(86400.);
        self.saved.time = (self.saved.time + dt).min(3600.);
        self.saved.exit.advance(dt);
        let c = self.saved.coop.as_mut().unwrap();
        for (i, p) in players.iter_mut().enumerate() {
            if !p.stats.alive() { c.previous[i] = None; continue; }
            let at = p.player.feet;
            if matches!(self.saved.phase, Phase::Slide | Phase::Fight) && !c.arena[i]
                && self.arena_trigger.touches(c.previous[i].unwrap_or(at) + PLAYER_CENTER, at + PLAYER_CENTER, PLAYER_HALF) {
                c.arena[i] = true;
                p.stats.minimum_sanity = 0.;
                if self.saved.phase == Phase::Slide {
                    self.saved.phase = Phase::Fight;
                    self.saved.time = 0.;
                    self.saved.checkpoint = true;
                }
            }
            c.previous[i] = Some(at);
            c.wait[i] = (c.wait[i] - dt).max(0.);
            if self.saved.phase == Phase::Fight && c.wait[i] == 0.
                && (p.stats.sanity() < 100. || p.stats.will() < 100.) {
                let position = self.data.points[["get_me1", "get_me2", "get_me3"][c.essence[i]]].translation;
                let delta = (at + Vec3::Z * 32. - position).abs();
                if delta.x < 36. && delta.y < 36. && delta.z < 64. {
                    p.stats.essence(50.);
                    c.essence[i] = (c.essence[i] + 1) % 3;
                    c.wait[i] = 10.;
                }
            }
        }
        if self.saved.phase == Phase::Fight && self.saved.battle.action == battle::Action::Dead {
            let owner = self.saved.battle.coop.as_ref().and_then(|c| c.target)
                .filter(|i| players[*i].stats.alive())
                .or_else(|| players.iter().position(|p| p.stats.alive())).unwrap();
            self.advance_scene(map, world, players[owner].player)?;
            self.saved.coop.as_mut().unwrap().pending_scene = Some(owner);
        }
        self.rebuild(map)
    }
    pub(super) fn personal_essence(&self, player: Option<usize>) -> Option<Vec3> {
        if self.saved.phase != Phase::Fight { return None; }
        let (index, wait) = player.filter(|i| *i < 2).and_then(|i| self.saved.coop.as_ref().map(|c| (c.essence[i], c.wait[i])))
            .unwrap_or((self.saved.essence, self.saved.essence_wait));
        (wait == 0.).then(|| self.data.points[["get_me1", "get_me2", "get_me3"][index]].translation)
    }
}
