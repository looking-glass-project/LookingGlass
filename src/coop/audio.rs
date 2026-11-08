//! Transient host-owned cues. This bounded journal is replicated, never saved in
//! checkpoints: restoring gameplay must not replay an old hit or puzzle note.
use crate::{combat::Feedback, interaction::Events};
use anyhow::{ensure, Result};
use macroquad::prelude::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
mod playback;
pub use playback::Playback;

pub const CAPACITY: usize = 256;
pub const MAX_AGE: u64 = 240;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cue {
    pub sequence: u64,
    pub tick: u64,
    pub path: String,
    pub origin: Option<Vec3>,
    pub volume: f32,
    pub animation: bool,
}
fn valid_path(path: &str) -> bool {
    path.len() <= 256 && path.starts_with("sound/") && path.ends_with(".wav")
        && path.is_ascii() && !path.bytes().any(|c| c.is_ascii_control() || c == b'\\' || c == b':')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Batch {
    pub latest: u64,
    pub cues: Vec<Cue>,
}
impl Batch {
    pub fn validate(&self, tick: u64) -> Result<()> {
        ensure!(self.cues.len() <= CAPACITY, "Too many co-op sound cues");
        let mut previous = 0;
        let mut previous_tick = 0;
        for cue in &self.cues {
            ensure!(cue.sequence > previous && cue.sequence <= self.latest
                && cue.tick >= previous_tick && cue.tick <= tick && tick - cue.tick <= MAX_AGE
                && valid_path(&cue.path) && cue.origin.is_none_or(|at| at.is_finite())
                && cue.volume.is_finite() && (0. ..=4.).contains(&cue.volume), "Invalid co-op sound cue");
            previous = cue.sequence;
            previous_tick = cue.tick;
        }
        ensure!(self.cues.last().is_none_or(|cue| cue.sequence == self.latest), "Incomplete co-op sound journal");
        Ok(())
    }
}
#[derive(Default)]
pub struct Journal {
    latest: u64,
    cues: VecDeque<Cue>,
}
impl Journal {
    pub fn emit(&mut self, tick: u64, path: &str, origin: Option<Vec3>, volume: f32, animation: bool) {
        let path = path.replace('\\', "/").to_ascii_lowercase();
        if !valid_path(&path) || !volume.is_finite() || !(0. ..=4.).contains(&volume)
            || origin.is_some_and(|at| !at.is_finite()) { return; }
        self.latest += 1;
        self.cues.push_back(Cue { sequence:self.latest, tick, path, origin, volume, animation });
        while self.cues.len() > CAPACITY { self.cues.pop_front(); }
    }
    pub fn events(&mut self, tick: u64, events: &Events) {
        if let Some(path) = &events.sound { self.emit(tick,path,None,1.,false); }
    }
    pub fn feedback(&mut self, tick: u64, feedback: &Feedback) {
        for (path,at) in &feedback.cue_sounds { self.emit(tick,path,Some(*at),1.,false); }
        for path in &feedback.sounds { self.emit(tick,path,None,1.,false); }
        for (path,at) in &feedback.spatial_sounds { self.emit(tick,path,Some(*at),1.,false); }
    }
    pub fn batch(&self, tick: u64) -> Batch {
        Batch { latest:self.latest, cues:self.cues.iter().filter(|c| tick.saturating_sub(c.tick) <= MAX_AGE).cloned().collect() }
    }
    pub fn clear(&mut self) { self.cues.clear(); }
}

#[derive(Default)]
pub struct Receiver {
    scope: Option<(u64,u64)>,
    latest: u64,
}
impl Receiver {
    /// A first snapshot establishes a boundary, including after a fresh join,
    /// travel or wipe. Later repeated/collapsed snapshots emit each cue once.
    pub fn observe(&mut self, scope:(u64,u64), tick:u64, batch:&Batch) -> Result<Vec<Cue>> {
        batch.validate(tick)?;
        if self.scope != Some(scope) {
            self.scope = Some(scope); self.latest = batch.latest;
            return Ok(Vec::new());
        }
        ensure!(batch.latest >= self.latest, "Co-op sound timeline went backwards");
        let fresh = batch.cues.iter().filter(|c|c.sequence > self.latest).cloned().collect();
        self.latest = batch.latest;
        Ok(fresh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const NOTE: &str = "sound/world/test.wav";
    #[test]
    fn snapshots_repeat_and_collapse_without_losing_or_replaying_notes() {
        for hz in [30,60,144] {
            let mut journal = Journal::default();
            let mut receiver = Receiver::default();
            receiver.observe((1,0),0,&journal.batch(0)).unwrap();
            let mut heard = Vec::new(); let mut tick = 0;
            for frame in 0..hz*2 {
                let due = (frame+1)*120/hz;
                while tick < due {
                    tick += 1;
                    if [17,18,65,173].contains(&tick) { journal.emit(tick as u64,NOTE,None,1.,false); }
                }
                // A client can miss an entire snapshot while still obtaining
                // its cues from the next cumulative snapshot.
                if frame % 7 != 0 { heard.extend(receiver.observe((1,0),tick as u64,&journal.batch(tick as u64)).unwrap()); }
            }
            assert_eq!(heard.iter().map(|c|c.sequence).collect::<Vec<_>>(),[1,2,3,4]);
        }
    }
    #[test]
    fn join_travel_wipe_and_expired_history_are_not_replayed() {
        let mut journal = Journal::default(); let mut receiver = Receiver::default();
        journal.emit(1,NOTE,None,1.,false);
        assert!(receiver.observe((1,0),2,&journal.batch(2)).unwrap().is_empty());
        journal.emit(3,NOTE,None,1.,false);
        assert_eq!(receiver.observe((1,0),3,&journal.batch(3)).unwrap().len(),1);
        journal.emit(4,NOTE,None,1.,false);
        assert!(receiver.observe((1,1),4,&journal.batch(4)).unwrap().is_empty());
        journal.clear(); journal.emit(5,NOTE,None,1.,false);
        assert_eq!(receiver.observe((1,1),5,&journal.batch(5)).unwrap()[0].sequence,4);
        assert!(receiver.observe((2,0),5,&journal.batch(5)).unwrap().is_empty());
        journal.emit(6,NOTE,None,1.,false);
        assert!(receiver.observe((2,0),300,&journal.batch(300)).unwrap().is_empty());
        journal.emit(301,NOTE,None,1.,false);
        assert_eq!(receiver.observe((2,0),301,&journal.batch(301)).unwrap().len(),1);
    }
    #[test]
    fn bursts_are_bounded_and_wire_fields_are_validated() {
        let mut journal = Journal::default();
        for _ in 0..1000 { journal.emit(10,NOTE,None,1.,false); }
        let batch = journal.batch(10);
        assert_eq!(batch.cues.len(),CAPACITY); batch.validate(10).unwrap();
        let mut invalid = batch.clone(); invalid.cues[1].sequence = invalid.cues[0].sequence;
        assert!(invalid.validate(10).is_err());
        for path in ["../secret.wav", "sound/../secret.wav", "sound/a\\b.wav", "sound/a:secret.wav"] {
            let mut invalid = batch.clone(); invalid.cues[0].path = path.into();
            assert!(invalid.validate(10).is_err());
        }
        let mut invalid = batch.clone(); invalid.cues[0].volume = f32::NAN;
        assert!(invalid.validate(10).is_err());
        let mut invalid = batch; invalid.cues[0].tick = 11;
        assert!(invalid.validate(10).is_err());
    }
}
