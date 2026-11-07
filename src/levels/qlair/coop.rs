//! Shared finale clock and attacks, with durable personal resources and captures.
use super::*;
use crate::{combat::{self, Opponents, PlayerTarget}, level::CoopPlayer};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub target: Option<usize>,
    pub actor: usize,
    pub initialized: [bool; 2],
    pub refilled: [bool; 2],
    pub essence: [u8; 2],
    pub wait: [f32; 2],
    pub platform: [i8; 2],
}
impl State {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.actor < 2 && self.target.is_none_or(|i| i < 2)
            && self.essence.iter().all(|i| *i < 4)
            && self.wait.iter().all(|t| t.is_finite() && (0. ..=10.).contains(t))
            && self.platform.iter().all(|p| *p != 0 && (-4..=4).contains(p)),
            "Invalid paired Queen state");
        Ok(())
    }
}
impl Queen {
    pub(super) fn pair_state(&mut self) -> &mut State {
        self.saved.coop.get_or_insert_with(|| State {
            target: self.saved.grabbed.then_some(0), actor: 0,
            // Import the existing resource cycle; joining/rejoining is not a refill.
            initialized: [self.saved.initialized; 2], refilled: [self.saved.refilled; 2],
            essence: [self.saved.essence; 2], wait: [self.saved.essence_wait; 2],
            platform: [self.saved.platform; 2],
        })
    }
    pub(super) fn personal_essence(&self, local: Option<usize>) -> Option<Transform> {
        if !matches!(self.saved.phase, Phase::Queen1 | Phase::Queen2) { return None; }
        let (index, wait) = local.filter(|i| *i < 2)
            .and_then(|i| self.saved.coop.as_ref().map(|c| (c.essence[i], c.wait[i])))
            .unwrap_or((self.saved.essence, self.saved.essence_wait));
        (wait == 0.).then(|| self.data.points[&format!("{}_me{}",
            if self.saved.phase == Phase::Queen1 { "get" } else { "help" }, index + 1)])
    }
    pub(super) fn prepare_pair(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        self.pair_state();
        self.saved.alive = players.iter().any(|p| p.stats.alive());
        if self.saved.grabbed && self.saved.coop.as_ref().unwrap().target
            .is_none_or(|i| !players[i].stats.alive()) { self.saved.grabbed = false; }
        for (i, p) in players.iter_mut().enumerate().filter(|(_, p)| p.stats.alive()) {
            let c = self.saved.coop.as_mut().unwrap();
            if !c.initialized[i] { Self::refill(p.stats); c.initialized[i] = true; }
            if self.saved.powered && !c.refilled[i] { Self::refill(p.stats); c.refilled[i] = true; }
            if let Some(at) = self.personal_essence(Some(i)) {
                let delta = (p.player.feet - at.translation).abs();
                if delta.x <= 32. && delta.y <= 32. && delta.z <= 64.
                    && (p.stats.sanity() < 100. || p.stats.will() < 100.) {
                    p.stats.essence(if self.saved.phase == Phase::Queen1 { 25. } else { 50. });
                    let c = self.saved.coop.as_mut().unwrap();
                    c.essence[i] = (c.essence[i] + 1) % if self.saved.phase == Phase::Queen1 { 2 } else { 4 };
                    c.wait[i] = 10.;
                }
            }
        }
        let c = self.saved.coop.as_ref().unwrap();
        self.saved.initialized = c.initialized[0]; self.saved.refilled = c.refilled[0];
        self.saved.essence = c.essence[0]; self.saved.essence_wait = c.wait[0];
        let actor = if self.saved.grabbed { c.target.unwrap_or(c.actor) } else { c.actor };
        self.track_actor(players[actor].player);
    }
    pub(super) fn track_actor(&mut self, p: &Player) {
        self.saved.alice = Transform {translation: p.feet, rotation: Quat::from_rotation_z(p.script_facing)};
    }
    pub(super) fn pair_fight(&mut self, dt: f32, world: &World,
        players: &mut [CoopPlayer<'_>; 2]) -> Feedback {
        if !dt.is_finite() || dt <= 0. || !players.iter().any(|p| p.stats.alive()) {
            return Feedback::default();
        }
        self.pair_state();
        let targets = players.each_ref().map(|p| PlayerTarget {
            eye: p.player.eye(), alive: p.stats.alive(), hidden: p.stats.ignores_alice(),
        });
        let mut opponents = Opponents {players: Some(targets),
            selected_player: self.saved.coop.as_ref().unwrap().target, ..Default::default()};
        let origin = if self.saved.phase == Phase::Queen1 { self.queen1_pose().translation }
            else { self.saved.body.translation + Vec3::Z * 1000. };
        // A captured player stays the victim until release or death. Other attacks
        // retain a visible target during their windup, then choose the nearest again.
        if !self.saved.grabbed {
            opponents.aim(world, origin, targets[0].eye, false, self.saved.attack != battle::Attack::Idle);
        }
        let selected = opponents.selected_player.filter(|i| targets[*i].alive);
        self.saved.coop.as_mut().unwrap().target = selected;
        if self.saved.grabbed && selected.is_none() { self.saved.grabbed = false; }
        if let Some(i) = selected.filter(|_| self.saved.phase == Phase::Queen2) {
            self.select_platform(self.saved.coop.as_ref().unwrap().platform[i]);
        }
        let bodies = opponents.bodies(targets[0].eye);
        let target = selected.and_then(|i| bodies.iter().find(|t|
            combat::player_slot(t.id) == Some(i)).copied());
        let i = selected.unwrap_or_else(|| players.iter().position(|p| p.stats.alive()).unwrap());
        let p = &mut players[i];
        let out = self.battle_with(dt, world, p.player, p.stats, bodies, target);
        if self.saved.grabbed { self.track_actor(p.player); }
        out
    }
    pub(super) fn select_platform(&mut self, platform: i8) {
        if self.saved.platform == platform { return; }
        self.saved.platform = platform;
        if platform < 0 && self.saved.position != 5 && self.saved.middle == 0. {
            self.saved.middle = 17.; self.saved.motion = 1; self.saved.motion_time = 0.;
        }
    }
}
