//! The host's shared visit history. Return visits have their own authored setup;
//! player resources remain in the roster and are never taken from an old visit.
use super::{arena::Arena, protocol::Visit, stage};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisitState {
    pub visit: Visit,
    pub stage: stage::Snapshot,
    pub npcs: crate::npc::Snapshot,
}
impl VisitState {
    pub fn capture(visit: &Visit, arena: &Arena) -> Self {
        Self {
            visit: visit.clone(),
            stage: arena.stage.snapshot(),
            npcs: arena.npcs.snapshot(),
        }
    }
    pub fn key(&self) -> String {
        crate::save::visit_key(&self.visit.map, self.visit.entry.as_deref())
    }
    pub fn matches(&self, visit: &Visit) -> Result<()> {
        self.visit.validate()?;
        visit.validate()?;
        ensure!(
            self.key() == crate::save::visit_key(&visit.map, visit.entry.as_deref())
                && self.visit.difficulty == visit.difficulty
                && !self.visit.combat_preview
                && !visit.combat_preview,
            "Cached co-op world belongs to another visit or difficulty"
        );
        Ok(())
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    levels: BTreeMap<String, VisitState>,
    completed: BTreeSet<String>,
}
impl Campaign {
    pub(super) fn imported(states: Vec<VisitState>, completed: BTreeSet<String>,
        difficulty: crate::powerups::Difficulty) -> Result<Self> {
        let campaign = Self { levels: states.into_iter().map(|s| (s.key(), s)).collect(), completed };
        campaign.validate(difficulty)?;
        Ok(campaign)
    }
    pub fn cached(&self, visit: &Visit) -> Option<&VisitState> {
        self.levels
            .get(&crate::save::visit_key(&visit.map, visit.entry.as_deref()))
    }
    /// Commit only after the destination, both arrival positions, and its art
    /// have loaded. A failed destination must not replace a completed visit.
    pub fn departed(&mut self, state: VisitState) -> Result<()> {
        state.visit.validate()?;
        ensure!(
            !state.visit.combat_preview,
            "A staged encounter cannot enter campaign history"
        );
        let key = state.key();
        ensure!(
            crate::campaign::visits().contains(&key),
            "Unknown campaign visit in co-op history"
        );
        self.completed.insert(key.clone());
        self.levels.insert(key, state);
        Ok(())
    }
    pub fn completed(&self) -> usize {
        self.completed.len()
    }
    pub fn validate(&self, difficulty: crate::powerups::Difficulty) -> Result<()> {
        let known = crate::campaign::visits();
        ensure!(self.levels.len() <= known.len() && self.completed.iter().all(|key| self.levels.contains_key(key)),
            "Invalid co-op visit history");
        for (key, state) in &self.levels {
            state.visit.validate()?;
            ensure!(known.contains(key) && state.key() == *key
                && state.visit.difficulty == difficulty && !state.visit.combat_preview,
                "Co-op visit history contains a different campaign or difficulty");
        }
        Ok(())
    }
}

/// All authored entrances, including separate return visits. This establishes
/// placement/restoration only, never a claim of traversing those visits.
pub fn check(assets: &mut crate::assets::Assets) -> Result<()> {
    let mut failures = Vec::new();
    for (name, entry) in crate::campaign::route() {
        let result = (|| -> Result<()> {
            let visit = Visit {
                epoch: 1,
                map: (*name).into(),
                entry: entry.map(str::to_owned),
                difficulty: Default::default(),
                combat_preview: false,
            };
            let map = crate::bsp::Bsp::parse(&assets.read(&format!("maps/{name}.bsp"))?)?;
            let mut world = crate::collision::World::from_bsp(&map)?;
            let mut stage = stage::Stage::load(assets, &map, &mut world, &visit, false)?;
            let (eye, yaw) = crate::interaction::spawn(&map, *entry);
            let mut roster = super::world::Roster::new_with(
                &world,
                eye,
                yaw,
                crate::inventory::Stats::for_level(name, *entry),
                |at| !stage.interactions.hazardous(at),
            )?;
            ensure!(
                roster
                    .players
                    .iter()
                    .all(|b| world.body_clear(b.player.feet)),
                "Entrance body is embedded"
            );
            let before = stage.snapshot();
            let serialized = serde_json::to_vec(&before)?;
            stage.restore(&serde_json::from_slice(&serialized)?, &map, &mut world)?;
            ensure!(
                serde_json::to_value(stage.snapshot())? == serde_json::to_value(&before)?,
                "Entrance world snapshot changed after restore"
            );
            // Let ordinary collision/swimming settle each arrival without
            // advancing or claiming support for its scripted controller.
            for _ in 0..240 {
                roster.move_players(&world, Default::default());
                ensure!(
                    roster.players.iter().all(|b| b.stats.alive()
                        && world.body_clear(b.player.feet)
                        && !stage.interactions.hazardous(b.player.feet)),
                    "Entrance physics reached a hazard"
                );
            }
            roster.tick += 1;
            for body in &mut roster.players {
                body.stats.damage(10000.);
            }
            roster.finish_tick_with(&world, false, |at| !stage.interactions.hazardous(at))?;
            ensure!(
                roster.checkpoint_generation == 1
                    && roster
                        .players
                        .iter()
                        .all(|b| b.stats.alive() && world.body_clear(b.player.feet)),
                "Entrance checkpoint failed"
            );
            println!("PASS co-op entrance {name}{entry:?}: two clear bodies, two seconds of entrance physics, world snapshot roundtrip and entrance wipe placement");
            if matches!(*name, "utemple" | "garden1") {
                for dead in super::PlayerId::ALL {
                    let mut swimmers = super::world::Roster::new_with(
                        &world,
                        eye,
                        yaw,
                        crate::inventory::Stats::for_level(name, *entry),
                        |at| !stage.interactions.hazardous(at),
                    )?;
                    ensure!(
                        swimmers.players.iter().all(|b| b.player.swimming),
                        "Swimming fixture did not start in water"
                    );
                    swimmers.players[dead.index()].stats.damage(10000.);
                    for tick in 0..=super::rules::RESPAWN_TICKS {
                        swimmers.move_players(&world, Default::default());
                        swimmers.finish_tick_with(&world, false, |at| {
                            !stage.interactions.hazardous(at)
                        })?;
                        ensure!(
                            swimmers.players[dead.partner().index()].stats.alive(),
                            "Swimming survivor died during countdown"
                        );
                        if tick < super::rules::RESPAWN_TICKS {
                            ensure!(
                                !swimmers.players[dead.index()].stats.alive()
                                    && super::world::camera_player(&swimmers.frame(1, 0), dead)
                                        == dead.partner(),
                                "Underwater death did not spectate for the full ten seconds"
                            );
                        }
                    }
                    let revived = &swimmers.players[dead.index()];
                    ensure!(revived.stats.sanity() == super::rules::RESPAWN_HEALTH
                        && revived.player.swimming && world.body_clear(revived.player.feet)
                        && revived.player.breath.submerged == 0.
                        && (revived.player.feet.z - swimmers.players[dead.partner().index()].player.feet.z).abs() < 0.1,
                        "Underwater respawn failed health, fresh air, clearance or same-depth placement");
                    println!("PASS co-op underwater {name} {dead:?}: ten-second spectator camera, live swimmer, 25-health same-depth respawn with fresh air");
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            let failure = format!("{name}{entry:?}: {error:#}");
            eprintln!("FAIL co-op entrance {failure}");
            failures.push(failure);
        }
    }
    ensure!(
        failures.is_empty(),
        "Co-op entrances require work: {}",
        failures.join("; ")
    );
    Ok(())
}
