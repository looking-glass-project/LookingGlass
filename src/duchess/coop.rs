//! Host-owned boss clock, captured actor and personal healing cooldowns.
use super::*;
use crate::level::CoopPlayer;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub(super) struct State {
    pub target: Option<usize>,
    pub grab: Option<usize>,
    pub essence: [[f32; 4]; 2],
}
impl State {
    pub fn validate(&self, boss: &super::State) -> Result<()> {
        ensure!(self.target.is_none_or(|i| i < 2) && self.grab.is_none_or(|i| i < 2)
            && (self.grab.is_some() == (boss.stage == Stage::Fighting && boss.action == Action::Bite))
            && boss.initialized, "Invalid paired Duchess actor");
        ensure!(self.essence.iter().flatten().all(|v| v.is_finite() && (0. ..=10.).contains(v)),
            "Invalid personal Duchess healing clock");
        Ok(())
    }
}

impl Duchess {
    pub fn coop_prepare(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        if self.state.coop.is_some() { return; }
        // Match the original visit's one-time entry restoration for each Alice.
        // Joining/restoring a fight or respawning must never grant a fresh heal.
        if !self.state.initialized && self.state.stage == Stage::Waiting {
            for body in players.iter_mut().filter(|b| b.stats.alive()) { body.stats.restore(); }
        }
        self.state.initialized = true;
        // A legacy single-Alice save can already be in the middle of a bite.
        let grab = (self.state.stage == Stage::Fighting && self.state.action == Action::Bite).then_some(0);
        self.state.coop = Some(State { essence: [self.state.essence; 2], grab, target: grab });
    }

    pub fn coop_pickup(&mut self, stats: &Stats) -> bool {
        if self.state.stage != Stage::Waiting || !stats.collected.contains(&self.pickup_id) { return false; }
        self.stage(Stage::Expanding);
        true
    }

    pub fn coop_controlled(&self, player: usize) -> bool {
        self.state.coop.as_ref().is_some_and(|c| c.grab == Some(player))
            && self.state.stage == Stage::Fighting && self.state.action == Action::Bite
            && self.state.clock < self.timing.frame(Action::Bite, 64.)
    }
    pub fn coop_respawned(&mut self, player: usize) {
        if self.state.coop.as_ref().is_some_and(|c| c.grab == Some(player)) {
            self.action(Action::Chase);
            self.state.wait = 0.8;
        }
    }

    pub fn essence_cooldown(&self, player: Option<usize>, item: usize) -> f32 {
        player.filter(|i| *i < 2).and_then(|i| self.state.coop.as_ref().map(|c| c.essence[i][item]))
            .unwrap_or(self.state.essence[item])
    }

    pub(super) fn coop_hold_clear(&self, victim: usize, hold: Vec3) -> bool {
        let Some(index) = combat::player_slot(victim) else { return true; };
        self.state.opponents.players.is_none_or(|players| {
            let other = players[1-index];
            let feet = other.eye - Vec3::Z * crate::movement::EYE_HEIGHT;
            !other.alive || (hold.z-feet.z).abs() >= crate::collision::PLAYER_HALF.z * 2.
                || hold.truncate().distance(feet.truncate()) >= crate::coop::separation::DISTANCE
        })
    }

    pub fn coop_combat(&mut self, dt: f32, world: &World, players: &mut [CoopPlayer<'_>; 2], paused: bool) -> Feedback {
        let mut out = Feedback::default();
        if !dt.is_finite() || dt <= 0. || paused || self.state.stage != Stage::Fighting { return out; }
        self.coop_prepare(players);
        let dt = dt.min(0.1);
        for body in players.iter_mut().filter(|b| b.stats.alive()) {
            if self.state.attacks == 0 && body.stats.selected() == 3 { body.stats.select(0); }
        }
        self.state.accumulator += f64::from(dt);
        while self.state.accumulator + 1e-9 >= 1. / 120. {
            self.state.accumulator -= 1. / 120.;
            let targets = players.each_ref().map(|b| combat::PlayerTarget {
                eye: b.player.eye(), alive: b.stats.alive(), hidden: b.stats.ignores_alice(),
            });
            if self.state.coop.as_ref().unwrap().grab.is_some_and(|i| !targets[i].alive) {
                self.action(Action::Chase);
                self.state.wait = 0.8;
            }
            self.state.opponents.players = Some(targets);
            self.state.opponents.summon = None;
            self.state.opponents.selected_player = self.state.coop.as_ref().unwrap().target;
            let chosen = self.state.coop.as_ref().unwrap().grab.or_else(|| {
                let (_, _, id) = self.state.opponents.aim(world, self.state.feet + Vec3::Z*64.,
                    targets[0].eye, false, self.state.action != Action::Chase);
                combat::player_slot(id)
            }).unwrap_or(0);
            self.state.notarget = false;
            self.step(world, players[chosen].player, &mut out);
            self.state.coop.as_mut().unwrap().target = self.state.opponents.selected_player;
        }
        for (index, body) in players.iter_mut().enumerate().filter(|(_, b)| b.stats.alive()) {
            for (i, pos) in self.essence_origins.iter().enumerate() {
                let cooldown = &mut self.state.coop.as_mut().unwrap().essence[index][i];
                *cooldown = (*cooldown-dt).max(0.);
                if *cooldown == 0. && crate::inventory::touches_pickup(body.player.feet, *pos, world)
                    && body.stats.apply(PickupKind::Essence, 25.) {
                    *cooldown = 10.;
                    self.state.essence_taken += 1;
                }
            }
        }
        out
    }
}
