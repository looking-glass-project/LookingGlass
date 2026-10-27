//! Short offline reproductions from captured authoritative gameplay reports.
//! Only initial restoration writes state; every subsequent tick uses Arena and
//! ordinary controls. This is not a replay of missing network packets or inputs.
use super::{
    arena::{Arena, Snapshot},
    protocol::{Input, Visit},
    world::{Body, Frame, Roster},
    PlayerId,
};
use crate::{assets::Assets, bsp::Bsp, collision::World, movement::Controls};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    report: PathBuf,
    #[serde(default)]
    pointer: String,
    /// Export this isolated current visit through the normal save machinery.
    #[serde(default)]
    export_checkpoint: bool,
    /// Retain the resulting native state for focused combat diagnosis.
    #[serde(default)]
    capture_final: bool,
    cases: Vec<Case>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    ticks: u32,
    /// None is a diagnostic observation, never a passing gameplay assertion.
    expected: Option<Expected>,
    #[serde(default)]
    players: [Vec<Action>; 2],
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Expected {
    TeamWipe,
    BothAlive,
    OneDead,
    Revived,
}
#[derive(Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
struct Action {
    start: u32,
    ticks: u32,
    wish: Vec2,
    swim: Vec3,
    rise: f32,
    run: bool,
    jump: bool,
    primary: bool,
    alternate: bool,
    weapon: Option<u8>,
    yaw: Option<f32>,
    pitch: Option<f32>,
}
impl Action {
    fn sample(&self, tick: u32, yaw: f32, pitch: f32) -> (Controls, Input) {
        (
            Controls {
                wish: self.wish,
                swim: self.swim,
                rise: self.rise,
                run: self.run,
                jump: self.jump && tick == self.start,
                ..Default::default()
            },
            Input {
                wish: self.wish,
                swim: self.swim,
                rise: self.rise,
                run: self.run,
                primary: self.primary,
                alternate: self.alternate,
                weapon: self.weapon,
                yaw: self.yaw.unwrap_or(yaw),
                pitch: self.pitch.unwrap_or(pitch),
                ..Default::default()
            },
        )
    }
}
impl Case {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.name.is_empty() && self.name.len() <= 80 && (1..=3600).contains(&self.ticks),
            "Encounter cases require a name and 1..3600 ticks"
        );
        for actions in &self.players {
            let mut end = 0;
            for action in actions {
                let next = action
                    .start
                    .checked_add(action.ticks)
                    .context("Input duration overflow")?;
                ensure!(
                    action.ticks > 0 && action.start >= end && next <= self.ticks,
                    "Encounter inputs must be ordered, non-overlapping and inside the case"
                );
                action.sample(action.start, 0., 0.).1.validate()?;
                end = next;
            }
        }
        Ok(())
    }
    fn sample(&self, tick: u32, bodies: &[Body; 2]) -> [(Controls, Input); 2] {
        std::array::from_fn(|i| {
            let body = &bodies[i];
            self.players[i]
                .iter()
                .find(|a| tick >= a.start && tick - a.start < a.ticks)
                .map_or(
                    (
                        Controls::default(),
                        Input {
                            yaw: body.yaw,
                            pitch: body.pitch,
                            ..Default::default()
                        },
                    ),
                    |a| a.sample(tick, body.yaw, body.pitch),
                )
        })
    }
}

#[derive(Serialize, PartialEq)]
struct Outcome {
    ticks: u32,
    wipe_tick: Option<u64>,
    first_death: [Option<u64>; 2],
    revivals: Vec<Revival>,
    health: [f32; 2],
    final_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_state: Option<serde_json::Value>,
    samples: Vec<serde_json::Value>,
}
#[derive(Serialize, PartialEq)]
struct Revival {
    player: PlayerId,
    death_tick: u64,
    spawned_at: u64,
    health: f32,
    feet: Vec3,
    survivor_feet: Vec3,
    rope: Option<usize>,
}
fn digest(value: &impl Serialize) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(value)?)))
}
fn run(
    arena: &mut Arena,
    map: &Bsp,
    world: &mut World,
    frame: &Frame,
    initial: &Snapshot,
    case: &Case,
    capture_final: bool,
) -> Result<Outcome> {
    arena.restore_checkpoint(initial, map, world)?;
    let mut roster =
        Roster::from_checkpoint(frame, world, |at| !arena.stage.interactions.hazardous(at))?;
    ensure!(
        serde_json::to_value(arena.snapshot())? == serde_json::to_value(initial)?,
        "Encounter restore changed the captured combat state"
    );
    ensure!(
        serde_json::to_value(&roster.players)? == serde_json::to_value(&frame.players)?,
        "Encounter restore changed captured players or resources"
    );
    let mut result = Outcome {
        ticks: 0,
        wipe_tick: None,
        first_death: [None; 2],
        revivals: vec![],
        health: frame.players.each_ref().map(|p| p.stats.sanity()),
        final_sha256: String::new(),
        final_state: None,
        samples: vec![],
    };
    let mut death_at = [None; 2];
    for offset in 0..case.ticks {
        let before = roster.players.each_ref().map(|b| b.stats.sanity());
        let dead_stats = roster
            .players
            .each_ref()
            .map(|b| (!b.stats.alive()).then(|| b.stats.clone()));
        let life_before = PlayerId::ALL.map(|id| std::mem::discriminant(&roster.lives.state(id)));
        let inputs = case.sample(offset, &roster.players);
        arena.step(&mut roster, map, world, inputs)?;
        roster.frame(frame.epoch, 0).validate().with_context(|| {
            format!(
                "Invalid replay frame tick{} grips{:?}",
                roster.tick,
                roster.players.each_ref().map(|b| b.player.rope.as_ref())
            )
        })?;
        result.ticks = offset + 1;
        let wipe = roster.checkpoint_generation != frame.checkpoint_generation;
        for i in 0..2 {
            if wipe || !roster.players[i].stats.alive() {
                result.first_death[i].get_or_insert(roster.tick);
                if before[i] > 0. {
                    death_at[i] = Some(roster.tick);
                }
            } else if let Some(mut expected) = dead_stats[i].clone() {
                let body = &roster.players[i];
                let died = death_at[i]
                    .take()
                    .context("Revival has no recorded death")?;
                ensure!(
                    body.spawned_at == roster.tick
                        && roster.tick >= died + u64::from(super::rules::RESPAWN_TICKS),
                    "Revival occurred before its ten-second countdown"
                );
                expected.set_health(super::rules::RESPAWN_HEALTH)?;
                // World drop lifetimes advance even for dead players during
                // Arena::step. They are not carried inventory. The focused
                // Roster regression checks the respawn itself preserves all
                // Stats, including these drops, without an intervening step.
                expected.loot = body.stats.loot.clone();
                ensure!(
                    serde_json::to_value(&expected)? == serde_json::to_value(&body.stats)?,
                    "Revival changed personal resources beyond restoring low health: expected {:?}, actual {:?}",
                    expected, body.stats
                );
                body.player.validate_world(world)?;
                let partner = &roster.players[1 - i].player;
                let delta = body.player.feet - partner.feet;
                ensure!(
                    world.body_clear(body.player.feet)
                        && !arena.stage.interactions.hazardous(body.player.feet)
                        && (delta.truncate().length() >= super::separation::DISTANCE
                            || delta.z.abs() >= crate::collision::PLAYER_HALF.z * 2.),
                    "Revival is obstructed, hazardous or overlaps the survivor"
                );
                result.revivals.push(Revival {
                    player: PlayerId::ALL[i],
                    death_tick: died,
                    spawned_at: body.spawned_at,
                    health: body.stats.sanity(),
                    feet: body.player.feet,
                    survivor_feet: partner.feet,
                    rope: body.player.rope.as_ref().map(|g| g.id.0),
                });
            }
        }
        result.health = roster.players.each_ref().map(|b| b.stats.sanity());
        let life_changed =
            life_before != PlayerId::ALL.map(|id| std::mem::discriminant(&roster.lives.state(id)));
        if wipe || offset % 12 == 0 || result.health != before || life_changed {
            result.samples.push(
                serde_json::json!({"tick":roster.tick,"health":result.health,
                "feet":roster.players.each_ref().map(|b|b.player.feet),
                "jumps":roster.players.each_ref().map(|b|b.player.jumps),
                "ropes":roster.players.each_ref().map(|b|b.player.rope.as_ref()),
                "camera_players":PlayerId::ALL.map(|id| super::world::camera_player(&roster.frame(frame.epoch,0),id)),
                "lives":roster.lives,"generation":roster.checkpoint_generation}),
            );
        }
        ensure!(
            arena.stage.transition.is_none() && !arena.stage.busy(),
            "Encounter entered a scene or transition; use a route test for that outcome"
        );
        if wipe {
            result.wipe_tick = Some(roster.tick);
            for (body, saved) in roster.players.iter().zip(&frame.players) {
                let mut restored = body.clone();
                ensure!(
                    restored.spawned_at == roster.tick,
                    "Wipe did not respawn both players"
                );
                restored.spawned_at = saved.spawned_at;
                ensure!(
                    serde_json::to_value(restored)? == serde_json::to_value(saved)?,
                    "Wipe lost a captured player's state or personal resources"
                );
            }
            ensure!(
                serde_json::to_value(arena.snapshot())? == serde_json::to_value(initial)?,
                "Wipe did not restore the complete encounter checkpoint"
            );
            break;
        }
    }
    result.final_sha256 = digest(&(roster.frame(frame.epoch, 0), arena.snapshot()))?;
    if capture_final {
        result.final_state = Some(serde_json::json!({
            "frame":roster.frame(frame.epoch,0), "combat":arena.snapshot()}));
    }
    Ok(result)
}

pub async fn check(assets: &mut Assets) -> Result<()> {
    let path = PathBuf::from(
        std::env::var("LOOKING_GLASS_COOP_ENCOUNTER")
            .context("Set LOOKING_GLASS_COOP_ENCOUNTER to an encounter fixture JSON file")?,
    );
    let fixture_bytes = std::fs::read(&path)?;
    let fixture: Fixture = serde_json::from_slice(&fixture_bytes)?;
    ensure!(
        !fixture.cases.is_empty() && fixture.cases.len() <= 32,
        "Expected 1..32 encounter cases"
    );
    for case in &fixture.cases {
        case.validate()?;
    }
    let source = std::fs::read(path.parent().unwrap().join(&fixture.report))?;
    let report: serde_json::Value = serde_json::from_slice(&source)?;
    let captured = report
        .pointer(&fixture.pointer)
        .context("Report JSON pointer is missing")?;
    let visit: Visit = serde_json::from_value(captured["visit"].clone())?;
    let frame: Frame = serde_json::from_value(captured["frame"].clone())?;
    let snapshot: Snapshot = serde_json::from_value(captured["combat"].clone())?;
    visit.validate()?;
    frame.validate()?;
    ensure!(
        visit.epoch == frame.epoch
            && !visit.combat_preview
            && frame
                .players
                .iter()
                .all(|b| b.stats.alive() && !b.stats.god),
        "Replay requires a live, non-invulnerable campaign capture from one epoch"
    );
    let map = Bsp::parse(&assets.read(&format!("maps/{}.bsp", visit.map))?)?;
    let mut world = World::from_bsp(&map)?;
    let mut arena = Arena::load(assets, &map, &mut world, &visit, false)?;
    if fixture.export_checkpoint {
        ensure!(captured["role"].as_str() == Some("host")
            && captured["completed_visits"].as_u64() == Some(0),
            "An isolated replay checkpoint requires a host capture without completed campaign visits");
        ensure!(
            !super::checkpoint::Store::available(),
            "Choose a fresh isolated profile for the encounter checkpoint export"
        );
        arena.restore_checkpoint(&snapshot, &map, &mut world)?;
        let mut roster =
            Roster::from_checkpoint(&frame, &world, |at| !arena.stage.interactions.hazardous(at))?;
        let store = super::checkpoint::Store::current(assets.fingerprint()?);
        store.save(
            &visit,
            &mut roster,
            &mut arena,
            &world,
            &super::campaign::Campaign::default(),
        )?;
        super::checkpoint::verify_restored(&store.load()?.checkpoint, &roster, &arena)?;
        println!(
            "PASS isolated encounter checkpoint export: exact current visit, players and resources"
        );
    }
    let mut results = vec![];
    let mut failed = false;
    for case in &fixture.cases {
        let one = run(&mut arena, &map, &mut world, &frame, &snapshot, case, fixture.capture_final)?;
        let two = run(&mut arena, &map, &mut world, &frame, &snapshot, case, fixture.capture_final)?;
        ensure!(
            one == two,
            "Encounter {} differs between identical restores and inputs",
            case.name
        );
        let passed = case.expected.map(|expected| match expected {
            Expected::TeamWipe => one.wipe_tick.is_some(),
            Expected::BothAlive => one.wipe_tick.is_none() && one.first_death == [None; 2],
            Expected::OneDead => {
                one.wipe_tick.is_none()
                    && one.first_death.iter().filter(|t| t.is_some()).count() == 1
            }
            Expected::Revived => {
                one.wipe_tick.is_none()
                    && one.revivals.iter().any(|r| {
                        r.spawned_at - r.death_tick == u64::from(super::rules::RESPAWN_TICKS)
                    })
            }
        });
        failed |= passed == Some(false);
        println!(
            "{} encounter {}: {} ticks, wipe {:?}, health {:?}; deterministic repeat matched",
            match passed {
                Some(true) => "PASS",
                Some(false) => "FAIL",
                None => "OBSERVED",
            },
            case.name,
            one.ticks,
            one.wipe_tick,
            one.health
        );
        results.push(serde_json::json!({"case":case,"assertion_passed":passed,"outcome":one}));
        clear_background(BLACK);
        next_frame().await;
    }
    let output = serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),
        "captured_build":captured["build"],"report_sha256":format!("{:x}",Sha256::digest(&source)),
        "fixture_sha256":format!("{:x}",Sha256::digest(&fixture_bytes)),"start_tick":frame.tick,
        "results":results,"full_route_passed":false,"checkpoint_exported":fixture.export_checkpoint,
        "scope":"Offline restored encounter with supplied ordinary inputs; excludes original network delivery and preceding traversal"});
    let destination = crate::preferences::path("coop-encounter-result.json");
    std::fs::create_dir_all(
        destination
            .parent()
            .context("Result has no parent folder")?,
    )?;
    std::fs::write(destination, serde_json::to_vec_pretty(&output)?)?;
    ensure!(
        !failed,
        "One or more encounter expectations failed; result saved"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encounter_inputs_reject_overlap_and_emit_jump_once_preserving_look() {
        let mut case:Case=serde_json::from_str(r#"{"name":"controls","ticks":120,"expected":"both_alive","players":[[{"start":10,"ticks":30,"wish":[1,0],"jump":true}],[]]}"#).unwrap();
        case.validate().unwrap();
        let action = &case.players[0][0];
        assert!(action.sample(10, 0.7, -0.2).0.jump);
        let (movement, look) = action.sample(11, 0.7, -0.2);
        assert!(!movement.jump);
        assert_eq!((look.yaw, look.pitch), (0.7, -0.2));
        case.players[0].push(Action {
            start: 20,
            ticks: 5,
            ..Default::default()
        });
        assert!(case.validate().is_err());
        case.players[0].pop();
        case.players[0][0].wish = Vec2::new(2., 0.);
        assert!(case.validate().is_err());
    }
}
