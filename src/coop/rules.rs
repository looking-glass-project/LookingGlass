//! Shared host decisions, independent of rendering, focus and network delivery.
use super::PlayerId;
use crate::{inventory::Stats, movement::Player};
use macroquad::prelude::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The simulation uses the movement clock's 120 Hz. Render/network frames do
/// not advance this timer, and a stalled connection cannot accelerate it.
pub const RESPAWN_TICKS: u16 = 1200;
/// Initial design choice for "low health": 25 out of the normal 100 Sanity.
pub const RESPAWN_HEALTH: f32 = 25.;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Life {
    Alive,
    Spectating {
        ticks_left: u16,
    },
    /// The timer expired; remain a spectator until collision-safe placement is
    /// available. Never revive at a guessed point inside a wall or over a void.
    AwaitingSafeSpawn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathEvent {
    Spectate { dead: PlayerId, follow: PlayerId },
    RespawnNear { dead: PlayerId, survivor: PlayerId },
    RestoreCheckpoint,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lives {
    states: [Life; 2],
    restoring_checkpoint: bool,
}
impl Default for Lives {
    fn default() -> Self {
        Self {
            states: [Life::Alive; 2],
            restoring_checkpoint: false,
        }
    }
}
impl Lives {
    pub fn checkpoint_ready(&self) -> bool {
        !self.restoring_checkpoint && self.states == [Life::Alive; 2]
    }
    pub fn state(&self, player: PlayerId) -> Life {
        self.states[player.index()]
    }

    /// Call once per host simulation tick, after damage for BOTH players has
    /// resolved. This makes simultaneous deaths a wipe, never a race to revive.
    pub fn tick(&mut self, alive: [bool; 2], cinematic: bool) -> Vec<DeathEvent> {
        if self.restoring_checkpoint {
            return Vec::new();
        }
        if !alive[0] && !alive[1] {
            self.restoring_checkpoint = true;
            return vec![DeathEvent::RestoreCheckpoint];
        }
        let mut events = Vec::new();
        for id in PlayerId::ALL {
            let i = id.index();
            if alive[i] {
                self.states[i] = Life::Alive;
                continue;
            }
            match self.states[i] {
                Life::Alive => {
                    self.states[i] = Life::Spectating {
                        ticks_left: RESPAWN_TICKS,
                    };
                    events.push(DeathEvent::Spectate {
                        dead: id,
                        follow: id.partner(),
                    });
                }
                Life::Spectating { ticks_left } if !cinematic => {
                    if ticks_left <= 1 {
                        self.states[i] = Life::AwaitingSafeSpawn;
                        events.push(DeathEvent::RespawnNear {
                            dead: id,
                            survivor: id.partner(),
                        });
                    } else {
                        self.states[i] = Life::Spectating {
                            ticks_left: ticks_left - 1,
                        };
                    }
                }
                _ => (),
            }
        }
        events
    }

    /// The placement system retries pending spawns as the survivor moves.
    /// After successful placement and health restoration, call `respawned`.
    pub fn pending_spawn(&self) -> Option<(PlayerId, PlayerId)> {
        if self.restoring_checkpoint {
            return None;
        }
        PlayerId::ALL.into_iter().find_map(|id| {
            (self.state(id) == Life::AwaitingSafeSpawn).then_some((id, id.partner()))
        })
    }

    /// Placement and health have already succeeded. Publish the living state
    /// in that same snapshot instead of waiting for the following simulation tick.
    pub fn respawned(&mut self, id: PlayerId) {
        debug_assert_eq!(self.state(id), Life::AwaitingSafeSpawn);
        self.states[id.index()] = Life::Alive;
    }

    /// Clear the wipe latch only AFTER an earlier checkpoint has loaded and
    /// BOTH bodies have valid placement. Failed loads keep the latch intact.
    pub fn checkpoint_restored(&mut self, both_alive: [bool; 2]) -> bool {
        if self.restoring_checkpoint && both_alive == [true, true] {
            *self = Self::default();
            true
        } else {
            false
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.states.iter().all(|state| match state {
                Life::Spectating { ticks_left } => (1..=RESPAWN_TICKS).contains(ticks_left),
                _ => true,
            }),
            "Invalid co-op respawn countdown"
        );
        Ok(())
    }
}

/// Visibility includes the enemy's existing view cone/range and a world sight
/// trace, supplied by its controller. An unseen nearer player cannot distract
/// the enemy from a visible farther player. Equal distances use stable IDs.
pub fn closest_visible(
    origin: Vec3,
    players: [&Player; 2],
    stats: [&Stats; 2],
    mut visible: impl FnMut(PlayerId, Vec3) -> bool,
) -> Option<PlayerId> {
    if !origin.is_finite() {
        return None;
    }
    PlayerId::ALL
        .into_iter()
        .filter(|id| {
            let i = id.index();
            stats[i].alive()
                && !stats[i].ignores_alice()
                && players[i].feet.is_finite()
                && visible(*id, players[i].eye())
        })
        .min_by(|a, b| {
            origin
                .distance_squared(players[a.index()].eye())
                .total_cmp(&origin.distance_squared(players[b.index()].eye()))
                .then(a.cmp(b))
        })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scene {
    pub key: String,
    pub triggered_by: PlayerId,
}

/// Keys are scoped to a campaign visit by the host, never trusted client text.
/// Only the first accepted arrival starts a scene. Repeated packets/contacts
/// cannot restart it; completion remains recorded after saving and loading.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Scenes {
    active: Option<Scene>,
    completed: BTreeSet<String>,
}
impl Scenes {
    pub fn begin(&mut self, key: &str, triggered_by: PlayerId, alive: bool) -> bool {
        if !alive
            || key.is_empty()
            || key.len() > 256
            || self.active.is_some()
            || self.completed.contains(key)
        {
            return false;
        }
        self.active = Some(Scene {
            key: key.to_owned(),
            triggered_by,
        });
        true
    }

    pub fn active(&self) -> Option<&Scene> {
        self.active.as_ref()
    }

    /// The caller must place both players safely at the authored end position
    /// before acknowledging completion. A failed placement retains the scene.
    pub fn complete(&mut self, reunited: bool) -> Option<Scene> {
        if !reunited {
            return None;
        }
        let scene = self.active.take()?;
        self.completed.insert(scene.key.clone());
        Some(scene)
    }
}

/// Use live boarding volumes for lifts/rafts that can carry both players.
/// A dead or absent passenger does not silently satisfy the boarding barrier.
pub fn all_aboard(alive: [bool; 2], in_boarding_area: [bool; 2]) -> bool {
    alive == [true, true] && in_boarding_area == [true, true]
}

#[cfg(test)]
mod tests {
    use super::*;
    use PlayerId::{Blue, Red};

    #[test]
    fn death_spectates_for_ten_simulation_seconds_before_requesting_safe_spawn() {
        assert!((RESPAWN_TICKS as f32 * crate::movement::FIXED_DT - 10.).abs() < 0.00001);
        let mut lives = Lives::default();
        assert_eq!(
            lives.tick([true, false], false),
            vec![DeathEvent::Spectate {
                dead: Red,
                follow: Blue
            }]
        );
        for _ in 0..RESPAWN_TICKS - 1 {
            assert!(lives.tick([true, false], false).is_empty());
        }
        assert!(lives.pending_spawn().is_none());
        assert_eq!(
            lives.tick([true, false], false),
            vec![DeathEvent::RespawnNear {
                dead: Red,
                survivor: Blue
            }]
        );
        for _ in 0..200 {
            assert!(lives.tick([true, false], false).is_empty());
        }
        assert_eq!(lives.pending_spawn(), Some((Red, Blue)));
        lives.tick([true, true], false);
        assert_eq!(lives.state(Red), Life::Alive);
        assert!(lives.pending_spawn().is_none());
    }

    #[test]
    fn both_deaths_restore_once_even_when_respawn_was_due_this_tick() {
        let mut lives = Lives::default();
        lives.tick([false, true], false);
        for _ in 0..RESPAWN_TICKS - 1 {
            lives.tick([false, true], false);
        }
        assert_eq!(
            lives.tick([false, false], false),
            vec![DeathEvent::RestoreCheckpoint]
        );
        assert!(lives.pending_spawn().is_none());
        for _ in 0..120 {
            assert!(lives.tick([false, false], false).is_empty());
        }
        assert!(!lives.checkpoint_restored([true, false]));
        assert!(lives.checkpoint_restored([true, true]));
        assert_eq!(
            lives.tick([false, false], false),
            vec![DeathEvent::RestoreCheckpoint]
        );
    }

    #[test]
    fn cinematic_holds_countdown_and_saved_countdown_resumes_without_restarting() {
        let mut lives = Lives::default();
        lives.tick([false, true], false);
        for _ in 0..120 {
            lives.tick([false, true], false);
        }
        for _ in 0..500 {
            lives.tick([false, true], true);
        }
        assert_eq!(lives.state(Blue), Life::Spectating { ticks_left: 1080 });
        let mut loaded: Lives =
            serde_json::from_str(&serde_json::to_string(&lives).unwrap()).unwrap();
        loaded.validate().unwrap();
        for _ in 0..1079 {
            assert!(loaded.tick([false, true], false).is_empty());
        }
        assert_eq!(
            loaded.tick([false, true], false),
            vec![DeathEvent::RespawnNear {
                dead: Blue,
                survivor: Red
            }]
        );
    }

    #[test]
    fn nearest_visible_target_excludes_occlusion_death_and_invisibility() {
        let near = Player::new(Vec3::new(20., 0., 0.));
        let far = Player::new(Vec3::new(80., 0., 0.));
        let mut blue = Stats::default();
        let mut red = Stats::default();
        assert_eq!(
            closest_visible(Vec3::ZERO, [&near, &far], [&blue, &red], |_, _| true),
            Some(Blue)
        );
        assert_eq!(
            closest_visible(Vec3::ZERO, [&near, &far], [&blue, &red], |id, _| id == Red),
            Some(Red)
        );
        blue.set_health(0.).unwrap();
        assert_eq!(
            closest_visible(Vec3::ZERO, [&near, &far], [&blue, &red], |_, _| true),
            Some(Red)
        );
        red.invisible = 2.;
        assert_eq!(
            closest_visible(Vec3::ZERO, [&near, &far], [&blue, &red], |_, _| true),
            None
        );
    }

    #[test]
    fn first_arrival_owns_scene_until_both_players_are_reunited() {
        let mut scenes = Scenes::default();
        assert!(scenes.begin("gvillage$first/gnome", Red, true));
        assert!(!scenes.begin("gvillage$first/gnome", Blue, true));
        assert!(!scenes.begin("gvillage$first/another", Blue, true));
        assert_eq!(scenes.active().unwrap().triggered_by, Red);
        assert!(scenes.complete(false).is_none());
        assert!(scenes.active().is_some());
        assert_eq!(scenes.complete(true).unwrap().triggered_by, Red);
        let mut loaded: Scenes =
            serde_json::from_str(&serde_json::to_string(&scenes).unwrap()).unwrap();
        assert!(!loaded.begin("gvillage$first/gnome", Blue, true));
        assert!(!loaded.begin("gvillage$first/another", Red, false));
        assert!(loaded.begin("gvillage$first/another", Blue, true));
    }

    #[test]
    fn shared_lift_waits_for_both_living_passengers() {
        for alive in [[false, false], [true, false], [false, true], [true, true]] {
            for aboard in [[false, false], [true, false], [false, true], [true, true]] {
                assert_eq!(
                    all_aboard(alive, aboard),
                    alive == [true, true] && aboard == [true, true]
                );
            }
        }
    }
}
