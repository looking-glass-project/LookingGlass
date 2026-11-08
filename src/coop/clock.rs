//! Bound work per rendered frame without losing the host's simulation time.
use crate::movement::FIXED_DT;
use super::protocol::INPUT_TIMEOUT;
use std::time::{Duration, Instant};

/// Sample current input at 30 Hz without accumulating render-frame overshoot.
pub struct Inputs { next: Instant }
impl Inputs {
    const PERIOD: Duration = Duration::from_nanos(1_000_000_000 / 30);
    pub fn new(now: Instant) -> Self { Self { next: now } }
    pub fn due(&self, now: Instant) -> bool { now >= self.next }
    pub fn sent(&mut self, now: Instant) {
        let late = now.saturating_duration_since(self.next).as_nanos() % Self::PERIOD.as_nanos();
        self.next = now + Self::PERIOD - Duration::from_nanos(late as u64);
    }
}

/// A stable 20 Hz cadence on the 120 Hz simulation clock. Small render jitter
/// must not postpone every subsequent snapshot by another whole render frame.
#[derive(Default)]
pub struct Snapshots { last_tick: u64 }
impl Snapshots {
    pub fn due(&self, tick: u64) -> bool { tick / 6 > self.last_tick / 6 }
    pub fn sent(&mut self, tick: u64) { self.last_tick = tick; }
}

#[derive(Default)]
pub struct Clock {
    remaining: f64,
}
impl Clock {
    pub fn advance(&mut self, elapsed: f32) -> (usize, bool) {
        // Beyond the input watchdog, resume from fresh controls rather than
        // replaying a suspended game's button presses in one catch-up burst.
        if !elapsed.is_finite() || elapsed < 0.
            || self.remaining + f64::from(elapsed) >= INPUT_TIMEOUT.as_secs_f64() {
            self.remaining = 0.;
            return (0, true);
        }
        self.remaining += f64::from(elapsed);
        let steps = (self.remaining / f64::from(FIXED_DT)).floor().min(12.) as usize;
        self.remaining -= steps as f64 * f64::from(FIXED_DT);
        (steps, false)
    }
    pub fn deferred_ms(&self) -> f64 { self.remaining * 1000. }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coop::protocol::{Input, TimedInput};
    use macroquad::prelude::Vec2;

    #[test]
    fn input_pacing_keeps_thirty_hertz_and_never_sends_a_catch_up_burst() {
        let start=Instant::now(); let mut clock=Inputs::new(start); let mut sent=0;
        for frame in 0..600_u64 {
            let now=start+Duration::from_nanos(frame*16_666_667+(frame%7)*300_000);
            if clock.due(now) { clock.sent(now); sent+=1; }
            assert!(!clock.due(now),"Sent twice in the same frame");
        }
        assert_eq!(sent,300,"Input sampling accumulated render jitter");
        let resumed=start+Duration::from_secs(60);
        assert!(clock.due(resumed)); clock.sent(resumed);
        assert!(!clock.due(resumed+Duration::from_millis(30)));
        assert!(clock.due(resumed+Duration::from_millis(34)));
    }

    #[test]
    fn snapshot_pacing_keeps_twenty_hertz_through_render_jitter_without_replaying_missed_frames() {
        let mut clock=Snapshots::default(); let mut sent=0;
        // Ten seconds rendered near 60 Hz, alternately early/late by one tick.
        for frame in 0..600 {
            let tick=frame*2+frame%2;
            if clock.due(tick) { clock.sent(tick); sent+=1; }
        }
        assert_eq!(sent,199,"Render jitter reduced the promised snapshot rate");
        assert!(clock.due(2400)); // a stall/backpressure interval
        clock.sent(2400);
        for tick in 2400..2406 { assert!(!clock.due(tick),"Replayed a missed snapshot"); }
        assert!(clock.due(2406));
    }

    #[test]
    fn brief_render_stalls_do_not_permanently_delay_guest_controls() {
        let mut clock = Clock::default();
        let mut input = TimedInput::default();
        let start = Instant::now();
        let mut micros = 0_u64;
        let mut packet = 0_u64;
        let mut steps = 0;
        for frame in 0..2_400 {
            let interval = match frame { 200 => 350_000, 700 => 400_000, 1600 => 200_000, _ => 10_000 };
            micros += interval;
            let now = start + Duration::from_micros(micros);
            // A regular 30 Hz sender with 50 ms delivery delay, including the
            // packet batches polled after each render stall.
            while packet * 100_000 / 3 + 50_000 <= micros {
                let sampled = packet * 100_000 / 3;
                input.accept(packet + 1, Input {
                    wish: if sampled / 1_000_000 % 2 == 0 { Vec2::X } else { Vec2::Y },
                    ..Default::default()
                }, sampled, now).unwrap();
                packet += 1;
            }
            let (count, suspended) = clock.advance(interval as f32 / 1_000_000.);
            assert!(!suspended);
            assert!(count <= 12, "A render stall caused an unbounded catch-up frame");
            for _ in 0..count { input.sample(now); }
            steps += count;
        }
        let expected = (micros as f64 / 1_000_000. / f64::from(FIXED_DT)).floor() as usize;
        assert!(steps.abs_diff(expected) <= 1, "Host lost {} ticks after brief render stalls", expected - steps);
        let buffered = input.timing()["buffered_control_ms"].as_f64().unwrap();
        assert!(buffered < 100., "Render stalls left {buffered:.1} ms of permanent control delay");
    }

    #[test]
    fn suspension_discards_old_time_instead_of_replaying_a_long_pause() {
        for elapsed in [0.5, 30., f32::NAN, f32::INFINITY, -1.] {
            let mut clock = Clock::default();
            assert_eq!(clock.advance(0.35), (12, false));
            assert_eq!(clock.advance(elapsed), (0, true));
            assert_eq!(clock.deferred_ms(), 0.);
            assert_eq!(clock.advance(FIXED_DT), (1, false));
        }
        // Repeated overloaded frames also have a finite catch-up budget.
        let mut clock = Clock::default();
        assert_eq!(clock.advance(0.3), (12, false));
        assert_eq!(clock.advance(0.3), (0, true));
    }
}
