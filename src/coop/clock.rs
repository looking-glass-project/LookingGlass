//! Bound work per rendered frame without losing the host's simulation time.
use crate::movement::FIXED_DT;
use super::protocol::INPUT_TIMEOUT;

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
    use std::time::{Duration, Instant};

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
