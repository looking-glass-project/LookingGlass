//! Nonblocking session lifecycle shared by the menu, simulation and probes.
//! A connected socket alone never means the guest is allowed to drive Alice.
use super::{
    protocol::{FromGuest, FromHost, Identity, Input, InputOutbox, TimedInput, Visit},
    transport::{Peer, State, MAX_INPUT, MAX_STATE},
};
use anyhow::{bail, ensure, Result};
use serde::Serialize;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const CHECK_TIMEOUT: Duration = Duration::from_secs(15);
const LOAD_TIMEOUT: Duration = Duration::from_secs(120);
const FRAMES_IN_FLIGHT: usize = 4;

/// TCP/relay buffers can hold seconds of stale snapshots even when the local
/// channels are small. Receipt feedback bounds the entire path to the guest.
#[derive(Default)]
struct SnapshotWindow {
    pending: VecDeque<u64>,
    sent: Option<u64>,
    seen: Option<u64>,
}
impl SnapshotWindow {
    fn acknowledge(&mut self, tick: u64) -> Result<()> {
        if self.seen == Some(tick) { return Ok(()); }
        let index = self.pending.iter().position(|&sent| sent == tick)
            .ok_or_else(||anyhow::anyhow!("Guest acknowledged an unsent or old snapshot."))?;
        self.pending.drain(..=index);
        self.seen = Some(tick);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Waiting,
    Connecting,
    Checking,
    Loading,
    Playing,
    Ended,
}

fn queue<T: Serialize>(pending: &mut VecDeque<Vec<u8>>, message: &T) -> Result<()> {
    ensure!(pending.len() < 4, "Too many pending session changes.");
    let bytes = serde_json::to_vec(message)?;
    ensure!(
        bytes.len() <= MAX_INPUT,
        "Co-op session message is too large."
    );
    pending.push_back(bytes);
    Ok(())
}
fn flush(peer: &Peer, pending: &mut VecDeque<Vec<u8>>) -> Result<()> {
    while let Some(bytes) = pending.front() {
        if !peer.send(bytes.clone())? {
            break;
        }
        pending.pop_front();
    }
    Ok(())
}

pub struct Host {
    peer: Peer,
    identity: Identity,
    visit: Visit,
    phase: Phase,
    generation: u64,
    deadline: Option<Instant>,
    pending: VecDeque<Vec<u8>>,
    local_ready: bool,
    guest_ready: bool,
    guest_loading: bool,
    input: TimedInput,
    control_reset: u64,
    awaiting_controls: bool,
    frames: SnapshotWindow,
}
impl Host {
    pub fn new(peer: Peer, identity: Identity, visit: Visit) -> Result<Self> {
        identity.compatible(&identity)?;
        visit.validate()?;
        Ok(Self {
            peer,
            identity,
            visit,
            phase: Phase::Waiting,
            generation: 0,
            deadline: None,
            pending: VecDeque::new(),
            local_ready: false,
            guest_ready: false,
            guest_loading: false,
            input: TimedInput::default(),
            control_reset: 0,
            awaiting_controls: false,
            frames: SnapshotWindow::default(),
        })
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn visit(&self) -> &Visit {
        &self.visit
    }
    /// Keep polling old controls until the guest's ordered Loading message.
    /// After that barrier it can send only Ready, so synchronous asset loading
    /// cannot fill the bounded input queue with an in-flight internet burst.
    pub fn can_load(&self) -> bool {
        self.phase != Phase::Loading || self.guest_loading
    }
    pub fn acknowledged_input(&self) -> u64 {
        self.input.sequence()
    }
    pub fn input_timing(&self) -> serde_json::Value {
        let mut timing = self.input.timing();
        timing["unacknowledged_frames"] = self.frames.pending.len().into();
        timing["latest_sent_frame"] = self.frames.sent.into();
        timing["acknowledged_frame"] = self.frames.seen.into();
        timing["control_reset"] = self.control_reset.into();
        timing["awaiting_controls"] = self.awaiting_controls.into();
        timing
    }
    pub fn can_send_frame(&self) -> bool {
        self.phase == Phase::Playing && self.pending.is_empty()
            && self.frames.pending.len() < FRAMES_IN_FLIGHT
    }
    pub fn send_frame(&mut self, frame: super::world::Frame) -> Result<bool> {
        if !self.can_send_frame() {
            return Ok(false);
        }
        ensure!(
            frame.epoch == self.visit.epoch,
            "Host snapshot belongs to another visit."
        );
        ensure!(self.frames.sent.is_none_or(|tick| frame.tick > tick),
            "Host snapshot is out of order.");
        frame.validate()?;
        let tick = frame.tick;
        let sent = self.peer.send(super::codec::encode(&serde_json::to_vec(
            &FromHost::Frame {
                frame: Box::new(frame),
            },
        )?)?)?;
        if sent {
            self.frames.pending.push_back(tick);
            self.frames.sent = Some(tick);
        }
        Ok(sent)
    }

    pub fn poll(&mut self, now: Instant) -> Result<()> {
        let result = self.poll_inner(now);
        if result.is_err() {
            self.peer.disconnect();
            self.phase = Phase::Waiting;
            self.input = TimedInput::default();
            self.control_reset = 0; self.awaiting_controls = false;
            self.frames = SnapshotWindow::default();
            self.pending.clear();
            self.deadline = None;
            self.guest_loading = false;
        }
        result
    }
    fn poll_inner(&mut self, now: Instant) -> Result<()> {
        if self.peer.state() != State::Connected {
            if self.phase != Phase::Waiting {
                if let State::Disconnected(reason) = self.peer.state() {
                    eprintln!("Co-op guest connection ended: {reason}");
                }
            }
            self.phase = if self.peer.state() == State::Stopped {
                Phase::Ended
            } else {
                Phase::Waiting
            };
            self.input = TimedInput::default();
            self.control_reset = 0; self.awaiting_controls = false;
            self.frames = SnapshotWindow::default();
            self.guest_ready = false;
            self.guest_loading = false;
            self.pending.clear();
            self.deadline = None;
            return Ok(());
        }
        if self.generation != self.peer.generation() {
            self.generation = self.peer.generation();
            self.phase = Phase::Checking;
            self.guest_ready = false;
            self.guest_loading = false;
            self.input = TimedInput::default();
            self.control_reset = 0; self.awaiting_controls = false;
            self.frames = SnapshotWindow::default();
            self.pending.clear();
            self.deadline = Some(now + CHECK_TIMEOUT);
            queue(
                &mut self.pending,
                &FromHost::Hello {
                    identity: self.identity.clone(),
                },
            )?;
        }
        if self.phase == Phase::Waiting {
            return Ok(());
        } // rejection is closing
        ensure!(
            self.deadline.is_none_or(|deadline| now < deadline),
            "The guest took too long to join or load the level."
        );
        for _ in 0..4 {
            let Some(bytes) = self.peer.receive() else {
                break;
            };
            ensure!(bytes.len() <= MAX_INPUT, "Guest message exceeds its limit.");
            let message: FromGuest = serde_json::from_slice(&bytes)
                .map_err(|_| anyhow::anyhow!("The guest sent an invalid session message."))?;
            match message {
                FromGuest::Hello { identity } => {
                    ensure!(self.phase == Phase::Checking, "Unexpected guest handshake.");
                    self.identity.compatible(&identity)?;
                    self.phase = Phase::Loading;
                    self.deadline = Some(now + LOAD_TIMEOUT);
                    queue(
                        &mut self.pending,
                        &FromHost::Enter {
                            visit: self.visit.clone(),
                        },
                    )?;
                }
                FromGuest::Loading { epoch } => {
                    ensure!(
                        matches!(self.phase, Phase::Loading | Phase::Playing),
                        "Guest loading arrived before identification."
                    );
                    ensure!(
                        epoch <= self.visit.epoch,
                        "Guest is loading a future visit."
                    );
                    if epoch == self.visit.epoch {
                        self.guest_loading = true;
                    }
                }
                FromGuest::Ready { epoch } => {
                    ensure!(
                        matches!(self.phase, Phase::Loading | Phase::Playing),
                        "Guest has not completed the handshake."
                    );
                    ensure!(
                        epoch <= self.visit.epoch,
                        "Guest acknowledged a future level."
                    );
                    if epoch == self.visit.epoch {
                        ensure!(
                            self.guest_loading,
                            "Guest readiness arrived before the loading barrier."
                        );
                        self.guest_ready = true;
                    }
                }
                FromGuest::Seen {epoch, tick} => {
                    ensure!(matches!(self.phase, Phase::Loading | Phase::Playing),
                        "Guest snapshot receipt arrived before identification.");
                    ensure!(epoch <= self.visit.epoch, "Guest acknowledged a future visit.");
                    if epoch == self.visit.epoch && self.phase == Phase::Playing {
                        self.frames.acknowledge(tick)?;
                    }
                }
                FromGuest::ControlsReset { epoch, reset, sequence, sampled_micros, controls } => {
                    ensure!(epoch <= self.visit.epoch, "Guest reset controls for a future visit.");
                    if epoch == self.visit.epoch {
                        ensure!(self.phase == Phase::Playing && self.awaiting_controls
                            && reset == self.control_reset && sequence > self.input.sequence(),
                            "Unexpected guest control reset.");
                        self.input.accept(sequence, controls, sampled_micros, now)?;
                        self.input.cancel();
                        self.awaiting_controls = false;
                    }
                }
                FromGuest::Input {
                    epoch,
                    sequence,
                    sampled_micros,
                    controls,
                } => {
                    ensure!(
                        matches!(self.phase, Phase::Loading | Phase::Playing),
                        "Guest input arrived before identification."
                    );
                    ensure!(
                        epoch <= self.visit.epoch,
                        "Guest controls refer to a future level."
                    );
                    // In-flight input from a departing visit is ordinary, not an error.
                    if epoch == self.visit.epoch && self.phase == Phase::Playing {
                        self.input.accept(sequence, controls, sampled_micros, now)?;
                        // Inputs ahead of the ordered reset reply were sampled
                        // while the host was suspended. Validate their counters
                        // without applying or retaining their movement intervals.
                        if self.awaiting_controls { self.input.cancel(); }
                    }
                }
            }
        }
        if self.phase == Phase::Loading && self.local_ready && self.guest_ready {
            queue(
                &mut self.pending,
                &FromHost::Playing {
                    epoch: self.visit.epoch,
                },
            )?;
            self.phase = Phase::Playing;
            self.deadline = None;
        }
        flush(&self.peer, &mut self.pending)
    }

    pub fn loaded(&mut self, epoch: u64) -> Result<()> {
        ensure!(epoch == self.visit.epoch, "Host loaded the wrong visit.");
        self.local_ready = true;
        Ok(())
    }

    /// Every transition, checkpoint restore and restart gets a fresh epoch.
    pub fn enter(
        &mut self,
        map: String,
        entry: Option<String>,
        difficulty: crate::powerups::Difficulty,
        now: Instant,
    ) -> Result<()> {
        let visit = Visit {
            epoch: self
                .visit
                .epoch
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("Session visit limit reached."))?,
            map,
            entry,
            difficulty,
            combat_preview: false,
        };
        visit.validate()?;
        self.visit = visit;
        self.local_ready = false;
        self.guest_ready = false;
        self.guest_loading = false;
        self.input = TimedInput::default();
        self.control_reset = 0; self.awaiting_controls = false;
        self.frames = SnapshotWindow::default();
        if matches!(self.phase, Phase::Loading | Phase::Playing) {
            queue(
                &mut self.pending,
                &FromHost::Enter {
                    visit: self.visit.clone(),
                },
            )?;
            self.phase = Phase::Loading;
            self.deadline = Some(now + LOAD_TIMEOUT);
        }
        Ok(())
    }
    pub fn cancel_controls(&mut self) -> Result<()> {
        self.input.cancel();
        if self.phase == Phase::Playing && !self.awaiting_controls {
            self.control_reset = self.control_reset.checked_add(1)
                .ok_or_else(||anyhow::anyhow!("Session control reset limit reached."))?;
            queue(&mut self.pending, &FromHost::ResetControls {
                epoch:self.visit.epoch, reset:self.control_reset,
            })?;
            self.awaiting_controls = true;
        }
        Ok(())
    }
    pub fn controls(&mut self, now: Instant) -> (crate::movement::Controls, Input) {
        if self.phase == Phase::Playing
            && !self.awaiting_controls
            && self.peer.state() == State::Connected
            && self.generation == self.peer.generation()
        {
            self.input.sample(now)
        } else {
            (Default::default(), Default::default())
        }
    }
}

pub struct Guest {
    peer: Peer,
    identity: Identity,
    phase: Phase,
    deadline: Instant,
    verified: bool,
    visit: Option<Visit>,
    new_visit: bool,
    local_ready: bool,
    sequence: u64,
    sent_inputs: VecDeque<(u64, Instant)>,
    input_started: Instant,
    queued_controls: InputOutbox,
    latest_controls: Input,
    control_reset: u64,
    input_round_trip_ms: Option<f32>,
    frame: Option<super::world::Frame>,
    last_tick: Option<u64>,
    receipt: Option<u64>,
    pending: VecDeque<Vec<u8>>,
}
impl Guest {
    pub fn new(peer: Peer, identity: Identity, now: Instant) -> Result<Self> {
        identity.compatible(&identity)?;
        Ok(Self {
            peer,
            identity,
            phase: Phase::Connecting,
            deadline: now + CHECK_TIMEOUT,
            verified: false,
            visit: None,
            new_visit: false,
            local_ready: false,
            sequence: 0,
            sent_inputs: VecDeque::new(),
            input_started: now,
            queued_controls: InputOutbox::default(),
            latest_controls: Input::default(),
            control_reset: 0,
            input_round_trip_ms: None,
            frame: None,
            last_tick: None,
            receipt: None,
            pending: VecDeque::new(),
        })
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn take_visit(&mut self) -> Option<Visit> {
        if self.new_visit {
            self.new_visit = false;
            self.visit.clone()
        } else {
            None
        }
    }
    pub fn take_frame(&mut self) -> Option<super::world::Frame> {
        self.frame.take()
    }
    /// Passive diagnostics only: timings never affect movement or packet order.
    pub fn input_timing(&self) -> serde_json::Value {
        serde_json::json!({
            "sent_input": self.sequence,
            "queued_input_samples": self.queued_controls.len(),
            "round_trip_ms": self.input_round_trip_ms,
            "oldest_pending_ms": self.sent_inputs.front().map(|(_, at)| at.elapsed().as_secs_f32()*1000.),
        })
    }
    pub fn poll(&mut self, now: Instant) -> Result<()> {
        if self.phase == Phase::Ended {
            return Ok(());
        }
        let result = self.poll_inner(now);
        if result.is_err() {
            self.peer.stop();
            self.phase = Phase::Ended;
            self.pending.clear();
        }
        result
    }
    fn poll_inner(&mut self, now: Instant) -> Result<()> {
        match self.peer.state() {
            State::Disconnected(reason) => bail!("{reason}"),
            State::Stopped => bail!("The co-op session has ended."),
            State::Connected if self.phase == Phase::Connecting => {
                queue(
                    &mut self.pending,
                    &FromGuest::Hello {
                        identity: self.identity.clone(),
                    },
                )?;
                self.phase = Phase::Checking;
                self.deadline = now + CHECK_TIMEOUT;
            }
            _ => (),
        }
        ensure!(
            self.phase == Phase::Playing || now < self.deadline,
            "The host took too long to connect or load the level."
        );
        for _ in 0..4 {
            let Some(bytes) = self.peer.receive() else {
                break;
            };
            ensure!(
                bytes.len() <= MAX_STATE,
                "Host session message exceeds its limit."
            );
            let decoded = super::codec::decode(&bytes)?;
            let message: FromHost = serde_json::from_slice(&decoded)
                .map_err(|_| anyhow::anyhow!("The host sent an invalid session message."))?;
            match message {
                FromHost::Hello { identity } => {
                    ensure!(
                        self.phase == Phase::Checking && !self.verified,
                        "Unexpected host handshake."
                    );
                    self.identity.compatible(&identity)?;
                    self.verified = true;
                }
                FromHost::Enter { visit } => {
                    ensure!(self.verified, "Host has not completed the handshake.");
                    visit.validate()?;
                    ensure!(
                        self.visit
                            .as_ref()
                            .is_none_or(|old| visit.epoch > old.epoch),
                        "Host sent an old level transition."
                    );
                    self.visit = Some(visit);
                    self.new_visit = true;
                    self.local_ready = false;
                    self.phase = Phase::Loading;
                    self.deadline = now + LOAD_TIMEOUT;
                    self.sequence = 0;
                    self.input_started = now;
                    self.queued_controls.clear();
                    self.latest_controls = Input::default(); self.control_reset = 0;
                    self.sent_inputs.clear();
                    self.input_round_trip_ms = None;
                    self.frame = None;
                    self.last_tick = None;
                    self.receipt = None;
                    self.pending.clear();
                    queue(
                        &mut self.pending,
                        &FromGuest::Loading {
                            epoch: self.visit.as_ref().unwrap().epoch,
                        },
                    )?;
                }
                FromHost::Playing { epoch } => {
                    ensure!(
                        self.phase == Phase::Loading
                            && self.local_ready
                            && self.visit.as_ref().is_some_and(|v| v.epoch == epoch),
                        "Host started before both players loaded the level."
                    );
                    self.phase = Phase::Playing;
                }
                FromHost::ResetControls { epoch, reset } => {
                    ensure!(self.phase == Phase::Playing
                        && self.visit.as_ref().is_some_and(|v|v.epoch == epoch)
                        && reset > self.control_reset, "Unexpected host control reset.");
                    self.control_reset = reset;
                    self.queued_controls.clear();
                    self.sent_inputs.clear(); self.input_round_trip_ms = None;
                    self.sequence = self.sequence.checked_add(1)
                        .ok_or_else(||anyhow::anyhow!("Session input limit reached."))?;
                    let sampled_micros = self.input_started.elapsed().as_micros().try_into()
                        .map_err(|_|anyhow::anyhow!("Session input clock limit reached."))?;
                    // The ordered reply follows all already-sent old inputs.
                    // Its counters consume suspended presses without acting on
                    // them; only later, freshly sampled controls may drive Alice.
                    queue(&mut self.pending, &FromGuest::ControlsReset {
                        epoch, reset, sequence:self.sequence, sampled_micros,
                        controls:self.latest_controls,
                    })?;
                }
                FromHost::Frame { frame } => {
                    ensure!(
                        self.phase == Phase::Playing
                            && self.visit.as_ref().is_some_and(|v| v.epoch == frame.epoch),
                        "Host snapshot arrived outside its active visit."
                    );
                    ensure!(
                        self.last_tick.is_none_or(|tick| frame.tick > tick)
                            && frame.acknowledged_input <= self.sequence,
                        "Host snapshot is out of order."
                    );
                    frame.validate()?;
                    if let Some((_, sent)) = self.sent_inputs.iter().find(|(sequence, _)| *sequence == frame.acknowledged_input) {
                        self.input_round_trip_ms = Some(sent.elapsed().as_secs_f32()*1000.);
                    }
                    while self.sent_inputs.front().is_some_and(|(sequence, _)| *sequence <= frame.acknowledged_input) {
                        self.sent_inputs.pop_front();
                    }
                    self.last_tick = Some(frame.tick);
                    self.receipt = Some(frame.tick);
                    self.frame = Some(*frame);
                }
            }
        }
        flush(&self.peer, &mut self.pending)?;
        // Coalesce receipts to the newest validated snapshot in this poll.
        // This is independent of input, including while the player's menu is open.
        if self.phase == Phase::Playing && self.pending.is_empty() {
            if let Some(tick) = self.receipt {
                if self.peer.send(serde_json::to_vec(&FromGuest::Seen {
                    epoch: self.visit.as_ref().unwrap().epoch, tick,
                })?)? { self.receipt = None; }
            }
        }
        self.flush_controls()
    }
    pub fn loaded(&mut self, epoch: u64) -> Result<()> {
        ensure!(
            self.phase == Phase::Loading
                && !self.local_ready
                && self.visit.as_ref().is_some_and(|v| v.epoch == epoch),
            "Guest loaded the wrong visit."
        );
        queue(&mut self.pending, &FromGuest::Ready { epoch })?;
        self.local_ready = true;
        Ok(())
    }
    /// Queue each sampled change through backpressure. True means accepted by
    /// the bounded outbox; only the host's fixed ticks can apply these controls.
    pub fn controls(&mut self, controls: Input) -> Result<bool> {
        if self.phase != Phase::Playing { return Ok(false); }
        let sampled_micros=self.input_started.elapsed().as_micros().try_into()
            .map_err(|_| anyhow::anyhow!("Session input clock limit reached."))?;
        self.queued_controls.push(controls,sampled_micros)?;
        self.latest_controls = controls;
        self.flush_controls()?;
        Ok(true)
    }
    fn flush_controls(&mut self) -> Result<()> {
        if self.phase != Phase::Playing || !self.pending.is_empty() { return Ok(()); }
        while let Some(sample)=self.queued_controls.front() {
            let sequence=self.sequence.checked_add(1)
                .ok_or_else(||anyhow::anyhow!("Session input limit reached."))?;
            if !self.peer.send(serde_json::to_vec(&FromGuest::Input {
                epoch:self.visit.as_ref().unwrap().epoch,sequence,
                sampled_micros:sample.sampled_micros,controls:sample.input,
            })?)? { break; }
            self.queued_controls.pop();
            self.sequence=sequence;
            if self.sent_inputs.len()==128 { self.sent_inputs.pop_front(); }
            self.sent_inputs.push_back((sequence,Instant::now()));
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coop::transport::{self, Conditions};
    use macroquad::prelude::Vec2;
    use std::thread;

    #[test]
    fn suspended_host_uses_an_ordered_barrier_before_fresh_controls() {
        let (mut host, mut guest, _) = pair();play(&mut host,&mut guest);
        guest.controls(Input{wish:Vec2::X,..Default::default()}).unwrap();
        until(&mut host,&mut guest,|h,_|h.acknowledged_input()==1);
        assert_eq!(host.controls(Instant::now()).0.wish,Vec2::X);
        host.cancel_controls().unwrap();
        for number in 0..24 {
            // The guest has not read the reset yet and continues to send old
            // controls. Their delivery spans many of the host's four-message polls.
            guest.controls(Input{wish:if number%2==0{Vec2::X}else{Vec2::Y},
                primary:true,jumps:1,..Default::default()}).unwrap();
            thread::sleep(Duration::from_millis(2));
            host.poll(Instant::now()).unwrap();
            let (c,i)=host.controls(Instant::now());
            assert_eq!(c.wish,Vec2::ZERO);assert!(!c.jump && !i.primary);
            host.cancel_controls().unwrap();
            assert_eq!(host.control_reset,1,"Repeated suspension queued duplicate resets");
        }
        until(&mut host,&mut guest,|h,_|!h.awaiting_controls);
        assert_eq!(host.input.timing()["buffered_control_ms"],0.);
        let (c,i)=host.controls(Instant::now());
        assert_eq!(c.wish,Vec2::ZERO);assert!(!c.jump && !i.primary);
        let before=host.acknowledged_input();
        guest.controls(Input{wish:-Vec2::X,jumps:1,..Default::default()}).unwrap();
        until(&mut host,&mut guest,|h,_|h.acknowledged_input()>before);
        let c=host.controls(Instant::now()).0;
        assert_eq!(c.wish,-Vec2::X);assert!(!c.jump,"Suspended jump reappeared after the barrier");
        let before=host.acknowledged_input();
        guest.controls(Input{wish:-Vec2::X,jumps:2,..Default::default()}).unwrap();
        until(&mut host,&mut guest,|h,_|h.acknowledged_input()>before);
        let jumps=(0..120).filter(|_|host.controls(Instant::now()).0.jump).count();
        assert_eq!(jumps,1,"A fresh post-resume jump was lost or duplicated");
    }

    #[test]
    fn control_reset_barrier_cannot_affect_a_later_visit() {
        let (mut host,mut guest,_)=pair();play(&mut host,&mut guest);
        host.cancel_controls().unwrap();
        host.enter("skool1".into(),None,Default::default(),Instant::now()).unwrap();
        play(&mut host,&mut guest);
        assert!(!host.awaiting_controls);assert_eq!(host.control_reset,0);
        guest.controls(Input{wish:Vec2::Y,..Default::default()}).unwrap();
        until(&mut host,&mut guest,|h,_|h.acknowledged_input()==1);
        assert_eq!(host.controls(Instant::now()).0.wish,Vec2::Y);
    }

    #[test]
    fn unsolicited_and_future_control_resets_are_rejected() {
        for requested in [false,true] {
            let (mut host,mut guest,_)=pair();play(&mut host,&mut guest);
            if requested {host.cancel_controls().unwrap();}
            assert!(guest.peer.send(serde_json::to_vec(&FromGuest::ControlsReset {
                epoch:1,reset:2,sequence:1,sampled_micros:0,controls:Input::default(),
            }).unwrap()).unwrap());
            let deadline=Instant::now()+Duration::from_secs(2);
            loop {
                if host.poll(Instant::now()).is_err() {break;}
                assert!(Instant::now()<deadline,"Accepted an unsolicited or future control reset");
                thread::sleep(Duration::from_millis(2));
            }
            assert_eq!(host.phase(),Phase::Waiting);
        }
    }

    #[test]
    fn delayed_snapshot_reader_resumes_with_latest_state_without_losing_controls() {
        use crate::{collision::World, inventory::Stats, movement::EYE_HEIGHT};
        use macroquad::prelude::{vec3, Vec3};
        let (mut host, mut guest, _) = pair();
        play(&mut host, &mut guest);
        let world = World::fixture(&[(vec3(-1000., -1000., -100.), vec3(1000., 1000., 0.))]);
        let roster = crate::coop::world::Roster::new(&world,
            Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        let mut admitted = Vec::new();
        // The transport thread can drain the app's small queue into TCP even
        // while the receiving game cannot consume snapshots. Bound freshness
        // across that boundary, not merely the size of one internal channel.
        for tick in (6..=144).step_by(6) {
            let mut frame = roster.frame(1, 0);
            frame.tick = tick;
            if host.send_frame(frame).unwrap() { admitted.push(tick); }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(!admitted.is_empty());
        assert!(admitted.len() <= 4, "Queued {} obsolete snapshots behind a delayed reader", admitted.len());
        guest.controls(Input {wish: Vec2::X, jumps: 1, ..Default::default()}).unwrap();
        until(&mut host, &mut guest, |h,g|
            h.acknowledged_input() == 1 && g.last_tick == admitted.last().copied());
        let controls = host.controls(Instant::now()).0;
        assert_eq!(controls.wish, Vec2::X);
        assert!(controls.jump);
        assert!(!host.controls(Instant::now()).0.jump);

        let mut latest = roster.frame(1, 1);
        latest.tick = 1200;
        let deadline = Instant::now() + Duration::from_secs(2);
        while !host.send_frame(latest.clone()).unwrap() {
            assert!(Instant::now() < deadline, "Snapshot receipt did not reopen the sender");
            host.poll(Instant::now()).unwrap();
            guest.poll(Instant::now()).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
        until(&mut host, &mut guest, |_,g|g.last_tick == Some(1200));
        assert_eq!(guest.take_frame().unwrap().tick, 1200);
    }

    fn snapshot(epoch: u64, tick: u64) -> crate::coop::world::Frame {
        use crate::{collision::World, inventory::Stats, movement::EYE_HEIGHT};
        use macroquad::prelude::{vec3, Vec3};
        let world = World::fixture(&[(vec3(-1000., -1000., -100.), vec3(1000., 1000., 0.))]);
        let roster = crate::coop::world::Roster::new(&world,
            Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        let mut frame = roster.frame(epoch, 0);
        frame.tick = tick;
        frame
    }

    #[test]
    fn snapshot_receipts_reject_unsent_ticks_and_future_visits() {
        for (epoch,tick) in [(1,0),(1,7),(2,6)] {
            let (mut host, mut guest, _) = pair();
            play(&mut host, &mut guest);
            assert!(host.send_frame(snapshot(1,6)).unwrap());
            assert!(guest.peer.send(serde_json::to_vec(&FromGuest::Seen {epoch,tick}).unwrap()).unwrap());
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                if host.poll(Instant::now()).is_err() { break; }
                assert!(Instant::now() < deadline, "Accepted invalid receipt {epoch}/{tick}");
                thread::sleep(Duration::from_millis(5));
            }
            assert_eq!(host.phase(), Phase::Waiting);
        }
    }

    #[test]
    fn snapshot_receipts_reset_at_transition_and_rejoin_without_replaying_old_state() {
        let (mut host, mut guest, invite) = pair();
        play(&mut host, &mut guest);
        for tick in [6,12,18,24] { assert!(host.send_frame(snapshot(1,tick)).unwrap()); }
        assert!(!host.send_frame(snapshot(1,30)).unwrap());
        host.enter("skool1".into(), None, Default::default(), Instant::now()).unwrap();
        play(&mut host, &mut guest);
        assert!(guest.peer.send(serde_json::to_vec(&FromGuest::Seen {epoch:1,tick:24}).unwrap()).unwrap());
        assert!(host.send_frame(snapshot(2,6)).unwrap());
        until(&mut host, &mut guest, |h,g|h.frames.seen == Some(6) && g.last_tick == Some(6));
        // Duplicating the last receipt is idempotent and cannot open extra slots.
        assert!(guest.peer.send(serde_json::to_vec(&FromGuest::Seen {epoch:2,tick:6}).unwrap()).unwrap());
        for tick in [12,18,24,30] { assert!(host.send_frame(snapshot(2,tick)).unwrap()); }
        assert!(!host.send_frame(snapshot(2,36)).unwrap());
        thread::sleep(Duration::from_millis(30));
        host.poll(Instant::now()).unwrap();
        assert!(!host.send_frame(snapshot(2,36)).unwrap());
        drop(guest);
        let deadline = Instant::now() + Duration::from_secs(2);
        while host.phase() != Phase::Waiting {
            host.poll(Instant::now()).unwrap();
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
        let mut replacement = Guest::new(transport::join(invite, Conditions::default()).unwrap(),
            identity(), Instant::now()).unwrap();
        play(&mut host, &mut replacement);
        assert!(host.send_frame(snapshot(2,1200)).unwrap());
        until(&mut host, &mut replacement, |h,g|h.frames.seen == Some(1200) && g.last_tick == Some(1200));
        assert_eq!(replacement.take_frame().unwrap().tick, 1200);
    }

    fn identity() -> Identity {
        Identity::current("0".repeat(64))
    }
    fn visit() -> Visit {
        Visit {
            epoch: 1,
            map: "gvillage".into(),
            entry: None,
            difficulty: Default::default(),
            combat_preview: false,
        }
    }
    fn pair() -> (Host, Guest, super::super::invite::Invite) {
        let delay = Conditions {
            delay_ms: 10,
            jitter_ms: 10,
        };
        let (peer, invite) = transport::host(delay).unwrap();
        let host = Host::new(peer, identity(), visit()).unwrap();
        let guest = Guest::new(
            transport::join(invite.clone(), delay).unwrap(),
            identity(),
            Instant::now(),
        )
        .unwrap();
        (host, guest, invite)
    }
    fn until(host: &mut Host, guest: &mut Guest, done: impl Fn(&Host, &Guest) -> bool) {
        let started = Instant::now();
        loop {
            host.poll(Instant::now()).unwrap();
            guest.poll(Instant::now()).unwrap();
            if done(host, guest) {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "session did not reach expected state"
            );
            thread::sleep(Duration::from_millis(2));
        }
    }
    fn play(host: &mut Host, guest: &mut Guest) {
        until(host, guest, |_, g| g.phase() == Phase::Loading);
        let visit = guest.take_visit().unwrap();
        guest.loaded(visit.epoch).unwrap();
        host.loaded(visit.epoch).unwrap();
        until(host, guest, |h, g| {
            h.phase() == Phase::Playing && g.phase() == Phase::Playing
        });
    }

    #[test]
    fn both_players_must_load_then_only_new_visit_inputs_can_move_the_guest() {
        let (mut host, mut guest, _) = pair();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Loading);
        assert_eq!(guest.take_visit().unwrap().epoch, 1);
        assert!(guest.take_visit().is_none());
        guest.loaded(1).unwrap();
        until(&mut host, &mut guest, |h, _| h.guest_ready);
        assert_eq!(host.phase(), Phase::Loading);
        assert!(!guest
            .controls(Input {
                wish: Vec2::X,
                ..Default::default()
            })
            .unwrap());
        host.loaded(1).unwrap();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Playing);
        assert!(guest
            .controls(Input {
                wish: Vec2::X,
                jumps: 1,
                ..Default::default()
            })
            .unwrap());
        until(&mut host, &mut guest, |h, _| h.input.sequence() == 1);
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::X);
        assert!(!host.controls(Instant::now()).0.jump);

        host.enter("skool1".into(), None, Default::default(), Instant::now())
            .unwrap();
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::ZERO);
        // An old packet already on the wire must not survive loading a new map.
        guest
            .peer
            .send(
                serde_json::to_vec(&FromGuest::Input {
                    epoch: 1,
                    sequence: 2,
                    sampled_micros: 0,
                    controls: Input {
                        wish: Vec2::X,
                        jumps: 2,
                        ..Default::default()
                    },
                })
                .unwrap(),
            )
            .unwrap();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Loading);
        assert_eq!(guest.take_visit().unwrap().epoch, 2);
        guest.loaded(2).unwrap();
        host.loaded(2).unwrap();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Playing);
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::ZERO);
        guest
            .controls(Input {
                wish: Vec2::Y,
                ..Default::default()
            })
            .unwrap();
        until(&mut host, &mut guest, |h, _| h.input.sequence() == 1);
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::Y);
    }

    #[test]
    fn transition_drains_old_inputs_before_a_slow_host_load() {
        let (mut host, mut guest, _) = pair();
        play(&mut host, &mut guest);
        host.enter("skool1".into(), None, Default::default(), Instant::now())
            .unwrap();
        assert!(!host.can_load());
        // The guest has not received Enter yet. These controls must drain before
        // loading, not sit in the four-packet channel for the whole disk stall.
        for _ in 0..12 {
            guest
                .controls(Input {
                    wish: Vec2::X,
                    ..Default::default()
                })
                .unwrap();
            host.poll(Instant::now()).unwrap();
            assert!(!host.can_load());
            thread::sleep(Duration::from_millis(10));
        }
        until(&mut host, &mut guest, |h, g| {
            h.can_load() && g.phase() == Phase::Loading
        });
        assert_eq!(guest.take_visit().unwrap().epoch, 2);
        guest.loaded(2).unwrap();
        guest.poll(Instant::now()).unwrap();
        // Longer than the transport's ordinary two-second consumer timeout.
        thread::sleep(Duration::from_millis(2300));
        assert_eq!(host.peer.state(), State::Connected);
        assert_eq!(guest.peer.state(), State::Connected);
        host.loaded(2).unwrap();
        until(&mut host, &mut guest, |h, g| {
            h.phase() == Phase::Playing && g.phase() == Phase::Playing
        });
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::ZERO);
        guest
            .controls(Input {
                wish: Vec2::Y,
                ..Default::default()
            })
            .unwrap();
        until(&mut host, &mut guest, |h, _| h.input.sequence() == 1);
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::Y);
    }

    #[test]
    fn mismatch_releases_the_guest_slot_and_a_matching_player_can_rejoin() {
        let (peer, invite) = transport::host(Conditions::default()).unwrap();
        let mut host = Host::new(peer, identity(), visit()).unwrap();
        host.loaded(1).unwrap();
        let mut wrong = identity();
        wrong.game_data = "1".repeat(64);
        let mut guest = Guest::new(
            transport::join(invite.clone(), Conditions::default()).unwrap(),
            wrong,
            Instant::now(),
        )
        .unwrap();
        let started = Instant::now();
        let mut rejected = false;
        loop {
            rejected |= host.poll(Instant::now()).is_err();
            rejected |= guest.poll(Instant::now()).is_err();
            if rejected && host.peer.state() != State::Connected && host.generation > 0 {
                break;
            }
            assert!(started.elapsed() < Duration::from_secs(5));
            thread::sleep(Duration::from_millis(2));
        }
        drop(guest);
        let mut replacement = Guest::new(
            transport::join(invite, Conditions::default()).unwrap(),
            identity(),
            Instant::now(),
        )
        .unwrap();
        until(&mut host, &mut replacement, |_, g| {
            g.phase() == Phase::Loading
        });
        replacement.loaded(1).unwrap();
        until(&mut host, &mut replacement, |_, g| {
            g.phase() == Phase::Playing
        });
        assert_eq!(host.controls(Instant::now()).0.wish, Vec2::ZERO);
    }

    #[test]
    fn disconnect_and_loading_timeout_release_controls_and_keep_host_available() {
        let (mut host, mut guest, invite) = pair();
        play(&mut host, &mut guest);
        guest
            .controls(Input {
                wish: Vec2::X,
                primary: true,
                ..Default::default()
            })
            .unwrap();
        until(&mut host, &mut guest, |h, _| h.input.sequence() == 1);
        drop(guest);
        let started = Instant::now();
        while host.phase() != Phase::Waiting {
            host.poll(Instant::now()).unwrap();
            assert!(started.elapsed() < Duration::from_secs(5));
            thread::sleep(Duration::from_millis(2));
        }
        let (motion, buttons) = host.controls(Instant::now());
        assert_eq!(motion.wish, Vec2::ZERO);
        assert!(!buttons.primary);
        let mut guest = Guest::new(
            transport::join(invite, Conditions::default()).unwrap(),
            identity(),
            Instant::now(),
        )
        .unwrap();
        until(&mut host, &mut guest, |_, g| g.phase() == Phase::Loading);
        assert!(host.poll(Instant::now() + LOAD_TIMEOUT).is_err());
        assert_eq!(host.phase(), Phase::Waiting);
    }

    #[test]
    fn guest_input_moves_only_its_host_body_and_the_guest_receives_that_authoritative_state() {
        use crate::{collision::World, inventory::Stats, movement::EYE_HEIGHT};
        use macroquad::prelude::{vec3, Vec3};
        let (mut host, mut guest, _) = pair();
        play(&mut host, &mut guest);
        let world = World::fixture(&[(vec3(-1000., -1000., -100.), vec3(1000., 1000., 0.))]);
        let mut roster = crate::coop::world::Roster::new(
            &world,
            Vec3::Z * (EYE_HEIGHT + 0.04),
            0.,
            Stats::default(),
        )
        .unwrap();
        let start = roster.players[1].player.feet;
        guest
            .controls(Input {
                wish: Vec2::Y,
                ..Default::default()
            })
            .unwrap();
        until(&mut host, &mut guest, |h, _| h.acknowledged_input() == 1);
        for _ in 0..120 {
            roster
                .step(&world, [Default::default(), host.controls(Instant::now())])
                .unwrap();
        }
        assert!(host
            .send_frame(roster.frame(1, host.acknowledged_input()))
            .unwrap());
        until(&mut host, &mut guest, |_, g| g.frame.is_some());
        let received = guest.take_frame().unwrap();
        assert!(received.players[0].player.feet.y.abs() < 0.1);
        assert!((received.players[1].player.feet.y - start.y - 104.).abs() < 5.);
        assert_eq!(
            received.players[1].player.feet,
            roster.players[1].player.feet
        );
        assert_eq!(received.acknowledged_input, 1);
    }
}
