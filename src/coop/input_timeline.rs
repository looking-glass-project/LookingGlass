//! Preserve sampled movement intervals when ordered packets arrive together.
//! Durations schedule bounded controls on the host's existing 120 Hz clock.
//! They cannot advance that clock or supply positions or gameplay state.
use super::*;
use std::collections::VecDeque;

const MAX_SAMPLES: usize = 64;
const MAX_SPAN_MICROS: u64 = 2_000_000;
const TICK_UNITS: u128 = 1_000_000;
const SETTLED_IDLE_UNITS: u128 = 30 * TICK_UNITS;

/// Valid controls arrived too late to replay within the bounded timeline.
/// The session can recover through its ordered reset barrier; invalid clocks
/// and action counters remain ordinary protocol errors.
#[derive(Debug)]
pub struct InputBacklog;
impl std::fmt::Display for InputBacklog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("Player controls are too far ahead of the host.") }
}
impl std::error::Error for InputBacklog {}

#[derive(Clone, Copy)]
pub struct SampledInput {
    pub input: Input,
    pub sampled_micros: u64,
}

/// Preserve unsent changes through transport backpressure, rather than sending
/// a later jump with the movement that should have followed it.
#[derive(Default)]
pub struct InputOutbox(VecDeque<SampledInput>);
impl InputOutbox {
    pub fn push(&mut self, input:Input, sampled_micros:u64) -> Result<()> {
        input.validate()?;
        if let Some(last)=self.0.back() {
            ensure!(sampled_micros>=last.sampled_micros,"Input clock moved backwards.");
            if last.input==input { return Ok(()); }
        }
        ensure!(self.0.len()<MAX_SAMPLES && self.0.front().is_none_or(|first|
            sampled_micros-first.sampled_micros<=MAX_SPAN_MICROS),
            "The connection cannot keep up with player controls. Rejoin the host.");
        self.0.push_back(SampledInput{input,sampled_micros});
        Ok(())
    }
    pub fn front(&self) -> Option<SampledInput> { self.0.front().copied() }
    pub fn pop(&mut self) { self.0.pop_front(); }
    pub fn clear(&mut self) { self.0.clear(); }
    pub fn len(&self) -> usize { self.0.len() }
}

#[derive(Clone, Copy)]
struct Sample {
    sequence: u64,
    input: Input,
    at: u64,
}

#[derive(Default)]
pub struct TimedInput {
    latch: RemoteInput,
    origin: Option<(u64, Instant)>,
    accepted: Option<Sample>,
    received: Option<Instant>,
    current: Option<Sample>,
    queued: VecDeque<Sample>,
    // Microseconds multiplied by 120, avoiding per-packet tick rounding drift.
    elapsed: u128,
}

fn counters(input:Input) -> [u64;6] {
    [input.jumps,input.uses,input.attack_presses[0],input.attack_presses[1],
        input.rope_edges[0],input.rope_edges[1]]
}

fn idle(input: Input) -> bool {
    input.wish == Vec2::ZERO && input.swim == Vec3::ZERO && input.rise == 0.
        && !input.primary && !input.alternate && !input.skip_scene
}

/// Looking around while idle changes presentation, not the movement interval.
/// Keep action counters and weapon selection distinct, even when stationary.
fn same_interval(mut a: Input, mut b: Input, look_only: bool) -> bool {
    if look_only && idle(a) && idle(b) { a.yaw=0.; a.pitch=0.; b.yaw=0.; b.pitch=0.; }
    a == b
}

impl TimedInput {
    // Acknowledgements retain their existing meaning: received and validated.
    pub fn sequence(&self) -> u64 { self.accepted.map_or(0,|s|s.sequence) }

    pub fn timing(&self) -> serde_json::Value {
        let first=self.current.or_else(||self.queued.front().copied());
        let remaining=self.accepted.zip(first).map_or(0,|(last,first)|
            (u128::from(last.at-first.at)*120).saturating_sub(self.elapsed));
        serde_json::json!({"received_input":self.sequence(),"applied_input":self.latch.sequence(),
            "received_jumps":self.accepted.map_or(0,|s|s.input.jumps),
            "applied_jumps":self.latch.last.jumps,"pending_jumps":self.latch.pending_jumps,
            "queued_input_samples":self.queued.len(),"buffered_control_ms":remaining as f64/120_000.})
    }

    fn expire(&mut self, now:Instant) -> bool {
        if self.received.is_some_and(|at|now.saturating_duration_since(at)<INPUT_TIMEOUT) {
            return false;
        }
        self.cancel();
        true
    }

    pub(crate) fn cancel(&mut self) {
        self.queued.clear(); self.current=None; self.elapsed=0;
        // Cancel queued edges as well as held motion. Preserve their counters
        // so they cannot reappear when fresh traffic resumes after the timeout.
        if let Some(last)=self.accepted {
            self.latch.last=last.input; self.latch.sequence=last.sequence;
        }
        self.latch.cancel(); self.received=None;
    }

    pub fn accept(&mut self, sequence:u64, input:Input, sampled_micros:u64, now:Instant) -> Result<()> {
        if sequence<=self.sequence() { return Ok(()); }
        self.expire(now);
        input.validate()?;
        let previous=self.accepted.map_or(Input::default(),|s|s.input);
        ensure!(self.accepted.is_none_or(|s|sampled_micros>=s.at),"Input clock moved backwards.");
        let (first_sample, first_received) = self.origin.unwrap_or((sampled_micros, now));
        ensure!(u128::from(sampled_micros-first_sample)
            <= now.saturating_duration_since(first_received).as_micros()+u128::from(MAX_SPAN_MICROS),
            "Player input clock is too far ahead of real time.");
        let pending=[self.latch.pending_jumps,self.latch.pending_uses,
            self.latch.pending_attacks[0],self.latch.pending_attacks[1],
            self.latch.pending_rope[0],self.latch.pending_rope[1]];
        for ((new,old),(applied,pending)) in counters(input).into_iter().zip(counters(previous))
            .zip(counters(self.latch.last).into_iter().zip(pending)) {
            ensure!(new.checked_sub(old).is_some_and(|n|n<=MAX_EDGES)
                && new.checked_sub(applied).is_some_and(|n|n<=MAX_EDGES-pending),
                "Invalid or excessive pending co-op actions.");
        }
        // A short click can already be released while its edge is still queued.
        // Its original aim must survive until the host has consumed the action.
        let look_only = pending.iter().all(|&n| n==0) && counters(input)==counters(self.latch.last);
        let first=self.current.or_else(||self.queued.front().copied());
        let ahead=first.map_or(0,|s|u128::from(sampled_micros-s.at)*120)
            .saturating_sub(self.elapsed);
        if ahead>u128::from(MAX_SPAN_MICROS)*120 {
            // After a network stall, an old neutral sample can be followed by
            // a current one in the same poll. Replaying seconds of that idle
            // only adds latency; it contains no movement or action to retain.
            // Active intervals, changed edges and forged clocks remain bounded.
            if !(idle(input)
                && counters(input)==counters(previous) && pending.iter().all(|&n|n==0)
                && self.current.is_none_or(|s|same_interval(s.input,input,look_only))
                && self.queued.iter().all(|s|same_interval(s.input,input,look_only))) { return Err(InputBacklog.into()); }
            self.current=None; self.queued.clear(); self.elapsed=0;
        }
        let sample=Sample{sequence,input,at:sampled_micros};
        if self.queued.back().is_some_and(|s|same_interval(s.input,input,look_only)) {
            let last = self.queued.back_mut().unwrap();
            last.sequence=sequence; last.input=input;
        } else if self.queued.is_empty() && self.current.is_some_and(|s|same_interval(s.input,input,look_only)) {
            let current = self.current.as_mut().unwrap();
            current.sequence=sequence; current.input=input;
            self.latch.accept(sequence,input,now)?;
            // A late packet shifts playback behind the sender. Once the host
            // has actually applied a quarter-second of unchanged idle, there
            // are no movement intervals or pending actions to retain here.
            // Confirm that idle through the latest sample instead of keeping
            // old jitter as permanent latency for the next movement command.
            // Short pauses, active controls and queued changes keep their time.
            if self.elapsed >= SETTLED_IDLE_UNITS && pending.iter().all(|&n|n==0)
                && idle(input) {
                let confirmed=u128::from(sampled_micros-self.current.unwrap().at)*120;
                self.elapsed=self.elapsed.max(confirmed);
            }
        } else {
            if self.queued.len()>=MAX_SAMPLES { return Err(InputBacklog.into()); }
            self.queued.push_back(sample);
        }
        self.origin.get_or_insert((sampled_micros, now));
        self.accepted=Some(sample); self.received=Some(now);
        Ok(())
    }

    fn activate(&mut self, sample:Sample, now:Instant) {
        self.latch.accept(sample.sequence,sample.input,now).expect("Validated input timeline");
        self.current=Some(sample);
    }

    pub fn sample(&mut self, now:Instant) -> (Controls,Input) {
        if self.expire(now) { return self.latch.sample(now); }
        if self.current.is_none() {
            if let Some(next)=self.queued.pop_front() { self.activate(next,now); }
        }
        while let (Some(current),Some(next))=(self.current,self.queued.front().copied()) {
            let duration=u128::from(next.at-current.at)*120;
            if self.elapsed<duration { break; }
            // Preserve sub-tick rounding, but never use a late packet to skip
            // whole movement intervals that have not yet been simulated.
            self.elapsed=(self.elapsed-duration)%TICK_UNITS;
            self.queued.pop_front(); self.activate(next,now);
        }
        self.latch.received=self.received;
        let input=self.latch.sample(now);
        self.elapsed=self.elapsed.saturating_add(TICK_UNITS);
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_stalled_idle_burst_resumes_without_disconnect_or_delaying_fresh_controls() {
        let mut stream=TimedInput::default(); let start=Instant::now();
        stream.accept(1,Input::default(),0,start).unwrap();
        for tick in 0..600 { stream.sample(start+Duration::from_micros(tick*1_000_000/120)); }
        let resumed=start+Duration::from_secs(5);
        // The transport releases a stale idle followed immediately by a fresh
        // idle, a run-up and its jump. Only the empty interval can be collapsed.
        stream.accept(2,Input::default(),1_000_000,resumed).unwrap();
        stream.accept(3,Input::default(),4_500_000,resumed).unwrap();
        let run=Input{wish:Vec2::X,run:true,..Default::default()};
        stream.accept(4,run,4_600_000,resumed).unwrap();
        stream.accept(5,Input{jumps:1,..run},4_800_000,resumed).unwrap();
        for tick in 0..37 {
            let controls=stream.sample(resumed+Duration::from_micros(tick*1_000_000/120)).0;
            assert_eq!(controls.wish,if tick<12 {Vec2::ZERO} else {Vec2::X});
            assert_eq!(controls.jump,tick==36);
        }
    }

    #[test]
    fn long_active_backlogs_and_idle_clock_forgery_are_still_rejected() {
        let start=Instant::now(); let run=Input{wish:Vec2::X,..Default::default()};
        let mut stream=TimedInput::default();
        stream.accept(1,run,0,start).unwrap();
        stream.accept(2,run,1_000_000,start+Duration::from_secs(5)).unwrap();
        assert!(stream.accept(3,run,4_500_000,start+Duration::from_secs(5)).is_err());
        let mut idle=TimedInput::default();
        idle.accept(1,Input::default(),0,start).unwrap();
        assert!(idle.accept(2,Input::default(),4_500_000,start+Duration::from_millis(10)).is_err());
    }
    #[test]
    fn a_delayed_burst_keeps_the_runup_before_its_jump() {
        let mut stream=TimedInput::default();
        let now=Instant::now();
        let running=Input{wish:Vec2::Y,run:true,..Default::default()};
        // The same poll receives the run-up, jump and the following turn.
        stream.accept(1,running,0,now).unwrap();
        stream.accept(2,Input{jumps:1,..running},200_000,now).unwrap();
        stream.accept(3,Input{wish:-Vec2::X,jumps:1,..running},400_000,now).unwrap();
        assert_eq!(stream.timing()["buffered_control_ms"],400.);
        for tick in 0..48 {
            let (controls,_)=stream.sample(now+Duration::from_secs_f64(tick as f64/120.));
            assert_eq!(controls.wish,Vec2::Y,"Turn replaced the run-up at tick {tick}");
            assert_eq!(controls.jump,tick==24,"Jump lost its run-up at tick {tick}");
        }
        assert_eq!(stream.sample(now+Duration::from_millis(400)).0.wish,-Vec2::X);
    }

    #[test]
    fn delayed_controls_cross_a_real_physics_gap_in_order() {
        use crate::{collision::World,movement::{Player,EYE_HEIGHT}};
        use macroquad::prelude::vec3;
        let world=World::fixture(&[
            (vec3(-100.,-400.,-100.),vec3(100.,-240.,0.)),
            (vec3(-100.,-60.,-100.),vec3(100.,200.,0.)),
        ]);
        let mut player=Player::spawn(&world,vec3(0.,-336.,EYE_HEIGHT+0.03125)).unwrap();
        let mut stream=TimedInput::default();let start=Instant::now();
        let run=Input{wish:Vec2::Y,run:true,..Default::default()};
        stream.accept(1,run,0,start).unwrap();
        stream.accept(2,Input{jumps:1,..run},200_000,start).unwrap();
        stream.accept(3,Input{jumps:1,..Default::default()},1_200_000,start).unwrap();
        let mut sequence=3;
        for tick in 0..360 {
            let now=start+Duration::from_secs_f64(tick as f64/120.);
            if tick%24==0 {
                sequence+=1;
                stream.accept(sequence,Input{jumps:1,..Default::default()},
                    1_200_000+tick*1_000_000/120,now).unwrap();
            }
            player.tick(&world,stream.sample(now).0);
        }
        assert_eq!(player.jumps,1);
        assert!(player.grounded && player.feet.z.abs()<0.1 && player.feet.y > -60.,
            "Delayed jump missed the opposite platform: {:?}",player.feet);
    }

    #[test]
    fn sender_backpressure_retains_changes_and_merges_only_identical_samples() {
        let mut out=InputOutbox::default();
        let run=Input{wish:Vec2::Y,run:true,..Default::default()};
        out.push(run,0).unwrap();
        for at in 1..200 { out.push(run,at*1_000).unwrap(); }
        out.push(Input{jumps:1,..run},200_000).unwrap();
        out.push(Input{wish:Vec2::X,jumps:1,..run},400_000).unwrap();
        assert_eq!(out.len(),3);
        let mut stream=TimedInput::default(); let now=Instant::now(); let mut sequence=0;
        while let Some(sample)=out.front() {
            sequence+=1;stream.accept(sequence,sample.input,sample.sampled_micros,now).unwrap();out.pop();
        }
        for tick in 0..49 {
            let c=stream.sample(now+Duration::from_secs_f64(tick as f64/120.)).0;
            assert_eq!(c.jump,tick==24);
            assert_eq!(c.wish,if tick<48 {Vec2::Y} else {Vec2::X});
        }
        out.push(run,0).unwrap();
        assert!(out.push(Input{jumps:1,..run},MAX_SPAN_MICROS+1).is_err());
        out.clear();assert_eq!(out.len(),0);
        for i in 0..MAX_SAMPLES {
            out.push(Input{wish:if i%2==0 {Vec2::X} else {Vec2::Y},..Default::default()},i as u64).unwrap();
        }
        assert!(out.push(Input::default(),MAX_SAMPLES as u64).is_err());
        assert!(out.push(Input::default(),0).is_err());
    }

    #[test]
    fn timeout_cancels_queued_edges_without_replaying_them_after_recovery() {
        let mut stream=TimedInput::default();let now=Instant::now();
        stream.accept(1,Input{wish:Vec2::X,..Default::default()},0,now).unwrap();
        stream.accept(2,Input{jumps:1,primary:true,..Default::default()},200_000,now).unwrap();
        assert_eq!(stream.sample(now).0.wish,Vec2::X);
        let stale=now+INPUT_TIMEOUT;
        let (c,i)=stream.sample(stale);assert_eq!(c.wish,Vec2::ZERO);assert!(!c.jump&&!i.primary);
        assert_eq!(stream.queued.len(),0);
        stream.accept(2,Input{jumps:1,primary:true,..Default::default()},200_000,stale).unwrap();
        assert!(!stream.sample(stale).1.primary,"Duplicate refreshed the watchdog");
        stream.accept(3,Input{jumps:1,..Default::default()},600_000,stale).unwrap();
        assert!(!stream.sample(stale).0.jump,"A cancelled queued jump reappeared");
        stream.accept(4,Input{jumps:2,..Default::default()},600_000,stale).unwrap();
        assert!(stream.sample(stale).0.jump);assert!(!stream.sample(stale).0.jump);
    }

    #[test]
    fn jittered_packet_intervals_do_not_accumulate_rounding_or_unbounded_delay() {
        let mut stream=TimedInput::default();let start=Instant::now();
        let mut packet=0_u64;let mut due=0_u64;let mut largest=0;
        for tick in 0..2_000_u64 {
            let now=start+Duration::from_secs_f64(tick as f64/120.);
            while due<=tick {
                let input=Input{wish:if packet%2==0 {Vec2::X} else {Vec2::Y},
                    ..Default::default()};
                stream.accept(packet+1,input,packet*1_000_000/30,now).unwrap();
                packet+=1;
                due=due.max(packet*4+[0,2,7,1,6,3][packet as usize%6]);
            }
            stream.sample(now);
            largest=largest.max(stream.queued.len());
            if tick>120 { assert!(stream.accepted.unwrap().sequence-stream.current.unwrap().sequence<8); }
        }
        assert!(largest<8,"Input latency grew throughout an ordinary jittered connection");
    }

    #[test]
    fn an_idle_connection_recovers_from_a_brief_packet_delay() {
        let mut stream=TimedInput::default(); let start=Instant::now();
        let mut packets=VecDeque::new(); let mut last_delivery=0;
        for number in 0..90_u64 {
            let sampled=number*1_000_000/30;
            let delay=if (300_000..500_000).contains(&sampled) {250_000} else {50_000};
            last_delivery=last_delivery.max(sampled+delay);
            let wish=if sampled<600_000 {
                if sampled/100_000%2==0 {Vec2::X} else {Vec2::Y}
            } else if sampled<2_000_000 {Vec2::ZERO} else {Vec2::X};
            packets.push_back((last_delivery,number+1,sampled,Input{wish,..Default::default()}));
        }
        let mut resumed=None;
        for tick in 0..360_u64 {
            let micros=tick*1_000_000/120; let now=start+Duration::from_micros(micros);
            while packets.front().is_some_and(|p|p.0<=micros) {
                let (_,sequence,sampled,input)=packets.pop_front().unwrap();
                stream.accept(sequence,input,sampled,now).unwrap();
            }
            let controls=stream.sample(now).0;
            if micros>=2_000_000 && controls.wish==Vec2::X && resumed.is_none() {resumed=Some(micros);}
        }
        assert!(resumed.is_some_and(|time|time<=2_100_000),
            "An earlier burst still delayed fresh movement after a long idle: {resumed:?}");
    }

    #[test]
    fn looking_around_while_idle_does_not_retain_old_network_delay() {
        let mut stream=TimedInput::default(); let start=Instant::now();
        let mut packets=VecDeque::new(); let mut last_delivery=0;
        for number in 0..90_u64 {
            let sampled=number*1_000_000/30;
            let delay=if (300_000..500_000).contains(&sampled) {250_000} else {50_000};
            last_delivery=last_delivery.max(sampled+delay);
            let wish=if sampled<600_000 {
                if sampled/100_000%2==0 {Vec2::X} else {Vec2::Y}
            } else if sampled<2_000_000 {Vec2::ZERO} else {Vec2::X};
            packets.push_back((last_delivery,number+1,sampled,
                Input{wish,yaw:number as f32*0.03,pitch:(number as f32*0.04).sin(),..Default::default()}));
        }
        let mut resumed=None;
        for tick in 0..360_u64 {
            let micros=tick*1_000_000/120; let now=start+Duration::from_micros(micros);
            while packets.front().is_some_and(|p|p.0<=micros) {
                let (_,sequence,sampled,input)=packets.pop_front().unwrap();
                stream.accept(sequence,input,sampled,now).unwrap();
            }
            let controls=stream.sample(now).0;
            if micros>=2_000_000 && controls.wish==Vec2::X && resumed.is_none() {resumed=Some(micros);}
        }
        assert!(resumed.is_some_and(|time|time<=2_100_000),
            "Camera-only idle samples retained an old delay before fresh movement: {resumed:?}");
    }

    #[test]
    fn idle_look_coalescing_preserves_a_queued_clicks_original_aim() {
        let mut stream=TimedInput::default(); let start=Instant::now();
        stream.accept(1,Input::default(),0,start).unwrap();
        let click=Input{yaw:1.,jumps:1,attack_presses:[1,0],..Default::default()};
        stream.accept(2,click,100_000,start).unwrap();
        stream.accept(3,Input{yaw:2.,..click},200_000,start).unwrap();
        for tick in 0..25 {
            let (movement,input)=stream.sample(start+Duration::from_secs_f64(tick as f64/120.));
            assert_eq!(movement.jump,tick==12);
            assert_eq!(input.primary,tick==12);
            assert_eq!(input.yaw,if tick<12 {0.} else if tick<24 {1.} else {2.});
        }
    }

    #[test]
    fn idle_retiming_does_not_skip_short_pauses_or_active_controls() {
        let active=[
            Input{wish:Vec2::X,..Default::default()},
            Input{swim:Vec3::Z,..Default::default()},
            Input{rise:1.,..Default::default()},
            Input{primary:true,..Default::default()},
            Input{alternate:true,..Default::default()},
            Input{skip_scene:true,..Default::default()},
        ];
        let start=Instant::now();
        for (input,ticks) in active.into_iter().map(|input|(input,36)).chain([(Input::default(),12)]) {
            let mut stream=TimedInput::default();stream.accept(1,input,0,start).unwrap();
            for tick in 0..ticks {stream.sample(start+Duration::from_secs_f64(tick as f64/120.));}
            stream.accept(2,input,600_000,start+Duration::from_millis(300)).unwrap();
            assert_eq!(stream.elapsed,ticks*TICK_UNITS,"An unsimulated active/short pause interval was skipped");
        }
        // Retiming a settled idle must not shorten the subsequent run-up or
        // discard a queued jump, even when both arrive in the same poll.
        let mut stream=TimedInput::default();stream.accept(1,Input::default(),0,start).unwrap();
        for tick in 0..36 {stream.sample(start+Duration::from_secs_f64(tick as f64/120.));}
        let now=start+Duration::from_millis(300);
        stream.accept(2,Input::default(),600_000,now).unwrap();
        let run=Input{wish:Vec2::X,run:true,..Default::default()};
        stream.accept(3,run,700_000,now).unwrap();
        stream.accept(4,Input{jumps:1,..run},900_000,now).unwrap();
        for tick in 0..37 {
            let c=stream.sample(now+Duration::from_secs_f64(tick as f64/120.));
            assert_eq!(c.0.wish,if tick<12 {Vec2::ZERO} else {Vec2::X});
            assert_eq!(c.0.jump,tick==36);
        }
    }

    #[test]
    fn forged_clocks_counters_and_queue_floods_are_bounded() {
        let now=Instant::now();let mut stream=TimedInput::default();
        stream.accept(1,Input::default(),100,now).unwrap();
        assert!(stream.accept(2,Input::default(),99,now).is_err());
        assert!(stream.accept(2,Input::default(),u64::MAX,now).is_err());
        assert!(stream.accept(2,Input{jumps:MAX_EDGES+1,..Default::default()},100,now).is_err());
        for i in 2..=MAX_SAMPLES {
            stream.accept(i as u64,Input{wish:if i%2==0 {Vec2::X} else {Vec2::Y},
                ..Default::default()},100,now).unwrap();
        }
        assert!(stream.accept(MAX_SAMPLES as u64+1,Input::default(),100,now).is_err());
        assert_eq!(stream.sequence(),MAX_SAMPLES as u64);
    }

    #[test]
    fn suspended_host_cancels_buffered_controls_without_replaying_edges() {
        let mut stream=TimedInput::default(); let now=Instant::now();
        let run=Input{wish:Vec2::X,run:true,..Default::default()};
        let edge=Input{jumps:1,uses:1,attack_presses:[1,1],rope_edges:[1,1],..run};
        stream.accept(1,run,0,now).unwrap(); stream.sample(now);
        stream.accept(2,edge,200_000,now).unwrap();
        stream.cancel();
        let (controls,buttons)=stream.sample(now);
        assert_eq!(controls.wish,Vec2::ZERO); assert!(!controls.jump && !controls.use_pressed);
        assert!(!buttons.primary && !buttons.alternate); assert_eq!(buttons.rope_edges,[0,0]);
        assert_eq!(stream.sequence(),2); assert_eq!(stream.timing()["buffered_control_ms"],0.);
        stream.accept(2,edge,200_000,now).unwrap();
        assert_eq!(stream.sample(now).0.wish,Vec2::ZERO,"Duplicate revived suspended controls");
        stream.accept(3,edge,800_000,now).unwrap();
        let (controls,buttons)=stream.sample(now);
        assert_eq!(controls.wish,Vec2::X); assert!(!controls.jump && !controls.use_pressed);
        assert!(!buttons.primary && !buttons.alternate); assert_eq!(buttons.rope_edges,[0,0]);
        stream.accept(4,Input{jumps:2,..edge},800_000,now).unwrap();
        assert!(stream.sample(now).0.jump); assert!(!stream.sample(now).0.jump);
    }

    #[test]
    fn resumed_burst_retains_a_changed_runup_and_jump() {
        let mut stream=TimedInput::default();let start=Instant::now();
        stream.accept(1,Input::default(),0,start).unwrap();stream.sample(start);stream.cancel();
        let resumed=start+Duration::from_secs(1);
        let run=Input{wish:Vec2::X,run:true,..Default::default()};
        stream.accept(2,run,600_000,resumed).unwrap();
        stream.accept(3,run,700_000,resumed).unwrap();
        stream.accept(4,Input{jumps:1,..run},800_000,resumed).unwrap();
        for tick in 0..25 {
            let (c,_)=stream.sample(resumed+Duration::from_micros(tick*1_000_000/120));
            assert_eq!(c.wish,Vec2::X);
            assert_eq!(c.jump,tick==24,"Suspension retiming shortened the resumed run-up");
        }
    }
}
