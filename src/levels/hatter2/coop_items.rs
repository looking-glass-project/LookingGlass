//! Personal rewards share the encounter clock, never each other's consumption.
use super::*;
use crate::level::CoopPlayer;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Items {
    initialized: [bool; 2],
    blade: [bool; 2],
    easy: [bool; 2],
    pub(super) ready: [bool; 2],
    pub(super) index: [usize; 2],
    pub(super) wait: [f32; 2],
    pub(super) drop_taken: Vec<[bool; 2]>,
}
impl Items {
    pub(super) fn validate(&self, saved: &Saved) -> Result<()> {
        ensure!(
            self.index.iter().all(|i| *i < 4)
                && self
                    .wait
                    .iter()
                    .all(|t| t.is_finite() && (0. ..=10.).contains(t))
                && self.drop_taken.len() == saved.drops.len()
                && (0..2).all(|i| !self.ready[i] || self.wait[i] == 0.),
            "Invalid paired Hatter rewards"
        );
        Ok(())
    }
}
impl Encounter {
    pub(super) fn pair_items(&mut self) -> &mut Items {
        let existing_pair = self.saved.coop.is_some();
        self.saved
            .coop
            .get_or_insert_with(Default::default)
            .items
            .get_or_insert_with(|| Items {
                // Existing active visits must not receive a new arrival refill.
                initialized: [self.saved.initialized
                    || self.saved.phase != Phase::Arrival
                    || existing_pair; 2],
                blade: [false; 2],
                easy: [self.easy; 2],
                ready: [self.saved.essence_live; 2],
                index: [self.saved.essence; 2],
                wait: [self.saved.essence_wait; 2],
                drop_taken: vec![[false; 2]; self.saved.drops.len()],
            })
    }
    pub(super) fn pair_prepare(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        self.alive = players.iter().any(|p| p.stats.alive());
        self.stopped = players
            .iter()
            .any(|p| p.stats.alive() && p.stats.powers.stopped > 0.);
        let items = self.pair_items();
        for (slot, p) in players.iter_mut().enumerate() {
            if !items.initialized[slot] {
                if p.stats.alive() {
                    p.stats.full_stats();
                }
                // A player who arrives dead keeps the ordinary low-health revival.
                items.initialized[slot] = true;
            }
            items.easy[slot] = p.stats.difficulty == crate::powerups::Difficulty::Easy;
            items.blade[slot] = p.stats.collected.contains("hatter2:11");
        }
        self.saved.initialized = true;
        self.saved.blade |= players
            .iter()
            .any(|p| p.stats.collected.contains("hatter2:11"));
        self.saved.watch |= players
            .iter()
            .any(|p| p.stats.collected.contains(&self.data.watch_key));
    }
    pub(super) fn pair_pickups(&mut self, dt: f32, players: &mut [CoopPlayer<'_>; 2]) {
        if !dt.is_finite() || dt <= 0. || self.scripted() {
            return;
        }
        self.pair_prepare(players);
        for (slot, p) in players
            .iter_mut()
            .enumerate()
            .filter(|(_, p)| p.stats.alive())
        {
            let items = self.saved.coop.as_mut().unwrap().items.as_mut().unwrap();
            if !items.blade[slot]
                && (p.player.feet + Vec3::Z * 20. - self.data.blade)
                    .abs()
                    .cmple(vec3(40., 40., 64.))
                    .all()
            {
                p.stats.collected.insert("hatter2:11".into());
                items.blade[slot] = true;
                self.saved.blade = true;
            }
            let at = self.personal_essence(slot);
            let items = self.saved.coop.as_mut().unwrap().items.as_mut().unwrap();
            if items.ready[slot]
                && (p.stats.sanity() < 100. || p.stats.will() < 100.)
                && (p.player.feet + Vec3::Z * 32. - at)
                    .abs()
                    .cmple(vec3(36., 36., 64.))
                    .all()
            {
                p.stats.essence(25.);
                items.ready[slot] = false;
                if items.easy[slot] {
                    items.index[slot] =
                        (items.index[slot] + 1) % self.data.healing.len().clamp(1, 4);
                    items.wait[slot] = 10.;
                }
            }
            for ((at, age), taken) in self.saved.drops.iter().zip(&mut items.drop_taken) {
                if !taken[slot]
                    && (p.stats.sanity() < 100. || p.stats.will() < 100.)
                    && (p.player.feet + Vec3::Z * 24. - *at)
                        .abs()
                        .cmple(vec3(32., 32., 56.))
                        .all()
                {
                    p.stats.essence(if *age < 10. {
                        50.
                    } else if *age < 30. {
                        25.
                    } else {
                        15.
                    });
                    taken[slot] = true;
                }
            }
        }
        if self.saved.phase == Phase::Victory && self.saved.blade && self.saved.watch {
            self.phase(Phase::WatchCat);
        }
    }
    pub(super) fn pair_item_clock(&mut self, dt: f32) {
        if let Some(items) = self.saved.coop.as_mut().and_then(|s| s.items.as_mut()) {
            for slot in 0..2 {
                if items.wait[slot] > 0. {
                    items.wait[slot] = (items.wait[slot] - dt).max(0.);
                    if items.wait[slot] == 0. {
                        items.ready[slot] = true;
                    }
                }
            }
        }
    }
    pub(super) fn pair_drop_age(&mut self, dt: f32) {
        let taken = &mut self
            .saved
            .coop
            .as_mut()
            .unwrap()
            .items
            .as_mut()
            .unwrap()
            .drop_taken;
        for (_, age) in &mut self.saved.drops {
            *age += dt;
        }
        for i in (0..self.saved.drops.len()).rev() {
            if self.saved.drops[i].1 >= 70. {
                self.saved.drops.remove(i);
                taken.remove(i);
            }
        }
    }
    fn personal_essence(&self, slot: usize) -> Vec3 {
        let items = self.saved.coop.as_ref().unwrap().items.as_ref().unwrap();
        if items.easy[slot] && !self.data.healing.is_empty() {
            self.data.healing[items.index[slot].min(self.data.healing.len() - 1)]
        } else {
            self.data
                .healing
                .last()
                .copied()
                .unwrap_or(vec3(5664., 768., 48.))
        }
    }
    pub(super) fn visible_blade(&self, local: Option<usize>) -> bool {
        match local
            .filter(|s| *s < 2)
            .zip(self.saved.coop.as_ref().and_then(|s| s.items.as_ref()))
        {
            Some((slot, items)) => !items.blade[slot],
            None => !self.saved.blade,
        }
    }
    pub(super) fn visible_essence(&self, local: Option<usize>) -> Option<Vec3> {
        match local
            .filter(|s| *s < 2)
            .zip(self.saved.coop.as_ref().and_then(|s| s.items.as_ref()))
        {
            Some((slot, items)) => items.ready[slot].then(|| self.personal_essence(slot)),
            None => self.saved.essence_live.then(|| self.essence_position()),
        }
    }
    pub(super) fn visible_drops(&self, local: Option<usize>) -> Vec<(Vec3, f32)> {
        let personal = local
            .filter(|s| *s < 2)
            .zip(self.saved.coop.as_ref().and_then(|s| s.items.as_ref()));
        self.saved
            .drops
            .iter()
            .enumerate()
            .filter(|(i, _)| personal.is_none_or(|(slot, s)| !s.drop_taken[*i][slot]))
            .map(|(_, d)| *d)
            .collect()
    }
}
