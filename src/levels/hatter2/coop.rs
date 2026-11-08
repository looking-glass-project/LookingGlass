//! Shared Hatter combat clock and stable personal attack targets.
//! Personal pickup state is owned by `coop_items`.
use super::*;
use crate::{
    combat::{DamageKind, PlayerTarget},
    level::CoopPlayer,
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub(super) struct State {
    pub(super) boss_player: Option<usize>,
    pub(super) wave_players: [Option<usize>; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) items: Option<coop_items::Items>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) actor: Option<usize>,
}
impl State {
    pub(super) fn validate(&self, saved: &Saved) -> Result<()> {
        if let Some(items) = &self.items {
            items.validate(saved)?;
        }
        ensure!(
            self.actor.is_none_or(|i| i < 2)
                && self.boss_player.is_none_or(|i| i < 2)
                && self.wave_players.iter().all(|p| p.is_none_or(|i| i < 2))
                && self
                    .wave_players
                    .iter()
                    .skip(saved.waves.len())
                    .all(Option::is_none),
            "Invalid paired Hatter targets"
        );
        Ok(())
    }
}
impl Encounter {
    pub(super) fn pair_fight(
        &mut self,
        dt: f32,
        world: &World,
        players: &mut [CoopPlayer<'_>; 2],
    ) -> Feedback {
        let mut out = Feedback::default();
        if !dt.is_finite()
            || dt <= 0.
            || self.scripted()
            || !players.iter().any(|p| p.stats.alive())
        {
            return out;
        }
        let targets = players.each_ref().map(|p| PlayerTarget {
            eye: p.player.eye(),
            alive: p.stats.alive(),
            hidden: p.stats.ignores_alice(),
        });
        self.pair_items();
        let stopped = players
            .iter()
            .any(|p| p.stats.alive() && p.stats.powers.stopped > 0.);
        let state = self.saved.coop.get_or_insert_with(Default::default);
        self.saved.boss.opponents.players = Some(targets);
        self.saved.boss.opponents.selected_player = state.boss_player;
        for shot in &mut self.saved.boss.shots {
            // Legacy single-player syringes were fired at the original Alice.
            if shot.syringe && shot.player.is_none() {
                shot.player = Some(0);
            }
        }
        for (index, wave) in self.saved.waves.iter_mut().enumerate() {
            wave.actor.opponents.players = Some(targets);
            wave.actor.opponents.selected_player = state.wave_players[index];
        }
        self.saved.remainder += f64::from(dt.min(0.1));
        while self.saved.remainder + 1e-9 >= 1. / 120. {
            self.saved.remainder = (self.saved.remainder - 1. / 120.).max(0.);
            self.saved
                .boss
                .projectile_step(battle::STEP, world, targets[0].eye, stopped, &mut out);
            if stopped {
                continue;
            }
            if self.saved.phase == Phase::Fight && !self.saved.boss.dying() {
                let tick = self.saved.cycle;
                if tick % 2880 == 0 {
                    out.spatial_sounds.push((
                        "sound/ambience/special/clock_gong_2.wav",
                        self.data.points["tower_down"].translation,
                    ));
                }
                if tick == 0 {
                    self.saved.boss.set(battle::Action::Idle);
                }
                if tick == 240 {
                    self.saved.boss.at = self.data.points["tower_down"].translation;
                    self.saved.boss.set(battle::Action::Ready);
                    self.saved.essence = 0;
                    self.saved.essence_live = true;
                    self.saved.essence_wait = 0.;
                    let items = self.pair_items();
                    items.ready = [true; 2];
                    items.index = [0; 2];
                    items.wait = [0.; 2];
                }
                if tick == 5760 {
                    self.saved.boss.set(battle::Action::Stand);
                }
                let up =
                    5760 + (self.data.boss().duration("ready_2_stand") * 120.).ceil() as u32 + 240;
                if tick == up {
                    self.saved.boss.at = self.data.points["tower_up"].translation;
                    self.saved.boss.set(battle::Action::Idle);
                }
                if tick == 6024 {
                    self.spawn_wave();
                    self.saved.coop.as_mut().unwrap().wave_players = [None; 2];
                }
                if tick == 9000 {
                    for wave in &mut self.saved.waves {
                        wave.actor.hit(Hit {
                            id: 0,
                            damage: 1000.,
                            kind: DamageKind::Other,
                            knockback: Vec3::ZERO,
                        });
                    }
                    let centre = self.data.points["hatter_splash"].translation;
                    for (slot, p) in targets.iter().enumerate().filter(|(_, p)| p.alive) {
                        let ray = world.sweep(centre + Vec3::Z * 24., p.eye, Vec3::ZERO);
                        if centre.distance(p.eye) < 300. && !ray.start_solid && ray.fraction >= 1. {
                            out.strike(
                                [crate::combat::COOP_BLUE, crate::combat::COOP_RED][slot],
                                200.,
                                (p.eye - centre).normalize_or_zero() * 200.,
                                DamageKind::Other,
                            );
                        }
                    }
                }
                self.saved.cycle = (tick + 1) % 11520;
            }
            self.pair_drop_age(battle::STEP);
            let mobile =
                self.saved.phase == Phase::Fight && (300..5760).contains(&self.saved.cycle);
            let early = self.saved.phase == Phase::Arrival
                && targets
                    .iter()
                    .any(|p| p.alive && !p.hidden && self.saved.boss.at.distance(p.eye) < 500.);
            self.saved.boss.step(
                world,
                targets[0].eye,
                mobile || early,
                mobile,
                false,
                &self.data,
                &mut out,
            );
            self.saved.coop.as_mut().unwrap().boss_player =
                self.saved.boss.opponents.selected_player;
            for (index, wave) in self.saved.waves.iter_mut().enumerate() {
                wave.actor.opponents.players = Some(targets);
                if wave.launch < 1.5 {
                    wave.launch = (wave.launch + battle::STEP).min(1.5);
                    let t = wave.launch / 1.5;
                    wave.actor.feet =
                        wave.from.lerp(wave.to, t) + Vec3::Z * (4. * t * (1. - t) * 100.);
                } else {
                    wave.actor.notarget = targets.iter().all(|p| !p.alive || p.hidden);
                    wave.actor
                        .update(battle::STEP, world, targets[0].eye, &self.data, &mut out);
                }
                self.saved.coop.as_mut().unwrap().wave_players[index] =
                    wave.actor.opponents.selected_player;
                if wave.actor.health == 0. && !wave.rewarded {
                    wave.rewarded = true;
                    if self.saved.drops.len() < 4 {
                        self.saved.drops.push((wave.actor.feet + Vec3::Z * 20., 0.));
                        self.saved
                            .coop
                            .as_mut()
                            .unwrap()
                            .items
                            .as_mut()
                            .unwrap()
                            .drop_taken
                            .push([false; 2]);
                    }
                }
            }
            if self.saved.boss.action == battle::Action::Gone && !self.won() {
                self.saved.waves.clear();
                self.saved.coop.as_mut().unwrap().wave_players = [None; 2];
                self.saved.essence_live = false;
                let items = self.pair_items();
                items.ready = [false; 2];
                self.phase(Phase::Victory);
            }
        }
        out
    }
}
