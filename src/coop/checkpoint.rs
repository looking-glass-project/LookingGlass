//! Host-owned co-op saves, separate from every single-player slot. Network
//! identity and invites never enter the file; resuming creates a fresh session.
use super::{arena::Arena, campaign::Campaign, protocol::Visit, world::{Frame, Roster}};
use crate::collision::World;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs::{self, File}, io::Read, path::{Path, PathBuf}};

const VERSION: u32 = 1;
// Includes up to 39 cached visits, in addition to the current shared world.
const LIMIT: u64 = 32 * 1024 * 1024;

#[path = "checkpoint_check.rs"]
mod check;
pub use check::verify_writes;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub visit: Visit,
    pub frame: Frame,
    pub campaign: Campaign,
}
impl Checkpoint {
    pub fn validate(&self) -> Result<()> {
        self.visit.validate()?;
        self.frame.validate()?;
        ensure!(!self.visit.combat_preview && self.visit.epoch == self.frame.epoch
            && crate::campaign::visits().contains(&crate::save::visit_key(&self.visit.map, self.visit.entry.as_deref())),
            "This encounter cannot be saved as a campaign checkpoint");
        ensure!(self.frame.lives.checkpoint_ready() && self.frame.players.iter().all(|b|
            b.stats.alive() && b.stats.difficulty == self.visit.difficulty),
            "Both players must be alive to save a co-op checkpoint");
        let world = self.frame.combat.as_ref().context("Checkpoint has no shared world")?;
        ensure!(world.stage.owner.is_none() && world.stage.transition.is_none(),
            "Wait until the scene or chapter transition finishes before saving");
        self.campaign.validate(self.visit.difficulty)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    game_data: String,
    checksum: String,
    payload: serde_json::Value,
}

pub struct Loaded {
    pub checkpoint: Checkpoint,
    pub recovered_backup: bool,
}
pub struct Store {
    path: PathBuf,
    fingerprint: String,
}
impl Store {
    pub fn current(fingerprint: String) -> Self {
        Self { path: crate::preferences::path("coop-checkpoint.json"), fingerprint }
    }
    fn backup(&self) -> PathBuf { self.path.with_extension("json.bak") }
    pub fn available() -> bool {
        let path = crate::preferences::path("coop-checkpoint.json");
        path.exists() || path.with_extension("json.bak").exists()
    }
    fn decode(&self, bytes: &[u8]) -> Result<Checkpoint> {
        let saved: Checkpoint = serde_json::from_value(unpack(bytes, &self.fingerprint)?)?;
        saved.validate()?;
        Ok(saved)
    }
    pub fn load(&self) -> Result<Loaded> {
        match read(&self.path).and_then(|bytes| self.decode(&bytes)) {
            Ok(checkpoint) => Ok(Loaded { checkpoint, recovered_backup: false }),
            Err(primary) => match read(&self.backup()).and_then(|bytes| self.decode(&bytes)) {
                Ok(checkpoint) => Ok(Loaded { checkpoint, recovered_backup: true }),
                Err(backup) => anyhow::bail!("Could not read the co-op checkpoint ({primary}); backup: {backup}"),
            },
        }
    }
    fn write(&self, saved: &Checkpoint) -> Result<()> {
        saved.validate()?;
        let bytes = pack(serde_json::to_value(saved)?, &self.fingerprint)?;
        if let Some(parent) = self.path.parent() { fs::create_dir_all(parent)?; }
        // A damaged primary must never replace the last known good backup.
        if let Ok(previous) = read(&self.path) {
            if self.decode(&previous).is_ok() {
                crate::save::atomic_write(&self.backup(), &previous)?;
            }
        }
        crate::save::atomic_write(&self.path, &bytes)
    }
    /// Commit the disk file before changing the in-memory team-wipe point.
    /// Failed placement, serialization or I/O leaves the previous point intact.
    pub fn save(&self, visit: &Visit, roster: &mut Roster, arena: &mut Arena,
        world: &World, campaign: &Campaign) -> Result<()> {
        ensure!(!arena.stage.busy() && !arena.stage.interactions.scripted() && !arena.stage.story.busy(),
            "Wait until the scene or chapter transition finishes before saving");
        let mut frame = roster.frame(visit.epoch, 0);
        Roster::from_checkpoint(&frame, world, |at| !arena.stage.interactions.hazardous(at))?;
        frame.combat = Some(Box::new(arena.snapshot()));
        self.write(&Checkpoint { visit: visit.clone(), frame, campaign: campaign.clone() })?;
        // Only acknowledge level requests after durable disk commit. The wipe
        // point must include the acknowledgement as well.
        for slot in &mut arena.stage.interactions.levels {
            if slot.ctl.checkpoint_requested() { slot.ctl.checkpoint_written(); }
        }
        arena.remember_checkpoint();
        roster.remember_checkpoint();
        Ok(())
    }
}
fn pack(payload: serde_json::Value, fingerprint: &str) -> Result<Vec<u8>> {
    crate::save::validate_json(&payload)?;
    let checksum = format!("{:x}", Sha256::digest(serde_json::to_vec(&payload)?));
    let bytes = serde_json::to_vec(&Envelope { format: "looking-glass-coop".into(), version: VERSION,
        game_data: fingerprint.into(), checksum, payload })?;
    ensure!(bytes.len() as u64 <= LIMIT, "Co-op checkpoint is too large");
    Ok(bytes)
}
fn unpack(bytes: &[u8], fingerprint: &str) -> Result<serde_json::Value> {
    ensure!(bytes.len() as u64 <= LIMIT, "Co-op checkpoint is too large");
    let envelope: Envelope = serde_json::from_slice(bytes).context("Co-op checkpoint is damaged")?;
    ensure!(envelope.format == "looking-glass-coop" && envelope.version == VERSION,
        "Co-op checkpoint needs a compatible game version");
    ensure!(envelope.game_data == fingerprint, "Co-op checkpoint needs matching game data and mods");
    ensure!(format!("{:x}", Sha256::digest(serde_json::to_vec(&envelope.payload)?)) == envelope.checksum,
        "Co-op checkpoint integrity check failed");
    crate::save::validate_json(&envelope.payload)?;
    Ok(envelope.payload)
}
fn read(path: &Path) -> Result<Vec<u8>> {
    let file = File::open(path).with_context(|| format!("Cannot open {}", path.display()))?;
    ensure!(file.metadata()?.len() <= LIMIT, "Co-op checkpoint is too large");
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= LIMIT, "Co-op checkpoint is too large");
    Ok(bytes)
}

/// Exact-state assertions used by the opt-in two-process checkpoint fixture.
/// Spawn stamps and the session epoch deliberately change after a team wipe.
pub fn verify_restored(saved: &Checkpoint, roster: &Roster, arena: &Arena) -> Result<()> {
    ensure!(serde_json::to_value(arena.snapshot())? == serde_json::to_value(saved.frame.combat.as_ref().unwrap())?,
        "The shared world or character state changed while restoring its checkpoint");
    for (body, expected) in roster.players.iter().zip(&saved.frame.players) {
        ensure!(serde_json::to_value(&body.player)? == serde_json::to_value(&expected.player)?
            && serde_json::to_value(&body.stats)? == serde_json::to_value(&expected.stats)?
            && body.yaw == expected.yaw && body.pitch == expected.pitch && body.running == expected.running,
            "A player or their resources changed while restoring the checkpoint");
    }
    ensure!(roster.lives.checkpoint_ready(), "Restored players are still marked as dead");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn envelope_rejects_other_data_tampering_versions_and_single_player_files() {
        let payload = serde_json::json!({"world":{"7":{"time":2.5}},"players":[33,82]});
        let bytes = pack(payload.clone(), "assets").unwrap();
        assert_eq!(unpack(&bytes, "assets").unwrap(), payload);
        assert!(unpack(&bytes, "other").is_err());
        for (key, value) in [("version", serde_json::json!(VERSION + 1)),
            ("format", serde_json::json!("looking-glass-save")),
            ("payload", serde_json::json!({"players":[100,100]}))] {
            let mut envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            envelope[key] = value;
            assert!(unpack(&serde_json::to_vec(&envelope).unwrap(), "assets").is_err());
        }
        assert!(unpack(&bytes[..bytes.len() - 3], "assets").is_err());
    }
    #[test]
    fn bounded_files_and_invalid_state_are_rejected_before_deserialization() {
        assert!(unpack(&vec![b' '; LIMIT as usize + 1], "assets").is_err());
        assert!(pack(serde_json::json!({"clock":1e30}), "assets").is_err());
        let store = Store { path: PathBuf::new(), fingerprint: "assets".into() };
        assert!(store.decode(&pack(serde_json::json!({}), "assets").unwrap()).is_err());
    }
}
