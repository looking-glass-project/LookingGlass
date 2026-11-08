//! Gameplay messages are directional: guests can only identify themselves,
//! acknowledge a loaded visit, and supply bounded controls. Never accept a
//! guest's position, inventory, damage or world state as authoritative.
use crate::{movement::Controls, powerups::Difficulty};
use anyhow::{ensure, Result};
use macroquad::prelude::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub const REVISION: u32 = 8;
pub const INPUT_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_EDGES: u64 = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub protocol: u32,
    pub build: String,
    pub game_data: String,
}
impl Identity {
    pub fn current(game_data: String) -> Self {
        Self {
            protocol: REVISION,
            build: env!("LOOKING_GLASS_BUILD").into(),
            game_data,
        }
    }

    pub fn compatible(&self, other: &Self) -> Result<()> {
        ensure!(
            self.protocol == other.protocol && self.build == other.build,
            "Both players need the same Looking Glass co-op build."
        );
        ensure!(
            self.game_data == other.game_data,
            "Both players need matching game data and mods."
        );
        ensure!(
            valid_digest(&self.build) && valid_digest(&self.game_data),
            "The co-op build or game data identity is invalid."
        );
        Ok(())
    }
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// A host-issued epoch changes on every visit/reload, even re-entering the same
/// map. Old input and readiness acknowledgements cannot cross that boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Visit {
    pub epoch: u64,
    pub map: String,
    pub entry: Option<String>,
    pub difficulty: Difficulty,
    #[serde(default)]
    pub combat_preview: bool,
}
impl Visit {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.epoch > 0
                && self.map.len() <= 64
                && crate::interaction::destination(&self.map).is_some(),
            "The host selected an unknown campaign level."
        );
        ensure!(
            self.entry.as_ref().is_none_or(|entry| !entry.is_empty()
                && entry.len() <= 128
                && entry
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))),
            "The host selected an invalid level entrance."
        );
        ensure!(
            !self.combat_preview || self.map == "skool1" && self.entry.is_none(),
            "The combat fixture requires the school entrance."
        );
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub wish: Vec2,
    pub swim: Vec3,
    pub rise: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub run: bool,
    pub primary: bool,
    pub alternate: bool,
    /// Held input; the host requires a fresh 0.65-second hold by the scene owner.
    #[serde(default)]
    pub skip_scene: bool,
    pub attack_presses: [u64; 2],
    pub weapon: Option<u8>,
    /// Cumulative edges survive a short press between network sends and queue
    /// backpressure. The host consumes each new edge once on its own clock.
    pub jumps: u64,
    pub uses: u64,
    /// Separate grab/release edges preserve the action the player saw. A late
    /// grab must not toggle an automatic grip off on the authoritative host.
    pub rope_edges: [u64; 2],
}
impl Input {
    pub fn classify_rope_use(&mut self, previous_uses: u64, player: &crate::movement::Player,
        world: &crate::collision::World, talking: bool) {
        if talking || self.uses == previous_uses { return; }
        let action = if player.rope.is_some() { Some(1) }
            else if player.rope_prompt(world) == Some("E grab rope") { Some(0) }
            else { None };
        if let Some(action) = action {
            self.rope_edges[action] += self.uses - previous_uses;
            self.uses = previous_uses;
        }
    }
    /// Called on the sampled edges, after dialogue and before movement locks.
    /// This requests normal rope physics; it never supplies a grip or position.
    pub fn rope_use(&self, player: &crate::movement::Player, world: &crate::collision::World) -> bool {
        if player.rope.is_some() { self.rope_edges[1] > 0 }
        else { self.rope_edges[0] > 0 && player.rope_prompt(world) == Some("E grab rope") }
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.wish.is_finite()
                && self.wish.length_squared() <= 1.001
                && self.swim.is_finite()
                && self.swim.length_squared() <= 1.001
                && self.rise.is_finite()
                && self.rise.abs() <= 1.
                && self.yaw.is_finite()
                && self.yaw.abs() <= std::f32::consts::PI
                && self.pitch.is_finite()
                && self.pitch.abs() <= 1.56
                && self.weapon.is_none_or(|w| w < 10),
            "Invalid co-op controls."
        );
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum FromGuest {
    Hello {
        identity: Identity,
    },
    /// Ordered after the last old-visit input, before any synchronous loading.
    Loading {
        epoch: u64,
    },
    Ready {
        epoch: u64,
    },
    /// Receipt of a validated host snapshot, never a guest-authored world state.
    Seen {
        epoch: u64,
        tick: u64,
    },
    /// Ordered after pre-suspension inputs, establishing cancelled button counters.
    ControlsReset {
        epoch: u64,
        reset: u64,
        sequence: u64,
        sampled_micros: u64,
        controls: Input,
    },
    Input {
        epoch: u64,
        sequence: u64,
        /// Relative client sampling time; never a world/simulation timestamp.
        sampled_micros: u64,
        controls: Input,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HostMessage<T> {
    Hello { identity: Identity },
    Enter { visit: Visit },
    Playing { epoch: u64 },
    ResetControls { epoch: u64, reset: u64 },
    Frame { frame: T },
}
// Use an external tag so JSON decodes the typed payload directly. An internal
// tag buffers through Serde Content, which cannot restore integer entity-map
// keys in the shared event runtime (although standalone snapshots roundtrip).
pub type FromHost = HostMessage<Box<super::world::Frame>>;

/// Host-owned latch. Network traffic never advances the simulation clock.
#[derive(Default)]
pub struct RemoteInput {
    sequence: u64,
    last: Input,
    received: Option<Instant>,
    pending_jumps: u64,
    pending_uses: u64,
    pending_attacks: [u64; 2],
    pending_rope: [u64; 2],
}

#[path = "input_timeline.rs"]
mod input_timeline;
pub use input_timeline::{InputBacklog, InputOutbox, TimedInput};
impl RemoteInput {
    pub(crate) fn cancel(&mut self) {
        self.pending_jumps = 0; self.pending_uses = 0;
        self.pending_attacks = [0; 2]; self.pending_rope = [0; 2];
        self.received = None;
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn accept(&mut self, sequence: u64, input: Input, now: Instant) -> Result<()> {
        // Duplicates do not refresh the watchdog or replay button edges.
        if sequence <= self.sequence {
            return Ok(());
        }
        input.validate()?;
        let jumps = input.jumps.checked_sub(self.last.jumps);
        let uses = input.uses.checked_sub(self.last.uses);
        let attacks: [Option<u64>; 2] = std::array::from_fn(|i| {
            input.attack_presses[i].checked_sub(self.last.attack_presses[i])
        });
        let rope: [Option<u64>; 2] = std::array::from_fn(|i| input.rope_edges[i].checked_sub(self.last.rope_edges[i]));
        ensure!(
            jumps.is_some_and(|n| n <= MAX_EDGES)
                && uses.is_some_and(|n| n <= MAX_EDGES)
                && attacks.iter().all(|n| n.is_some_and(|n| n <= MAX_EDGES))
                && rope.iter().all(|n| n.is_some_and(|n| n <= MAX_EDGES)),
            "Invalid co-op button sequence."
        );
        ensure!(
            self.pending_jumps + jumps.unwrap() <= MAX_EDGES
                && self.pending_uses + uses.unwrap() <= MAX_EDGES
                && (0..2).all(|i| self.pending_attacks[i] + attacks[i].unwrap() <= MAX_EDGES
                    && self.pending_rope[i] + rope[i].unwrap() <= MAX_EDGES),
            "Too many pending co-op actions."
        );
        self.pending_jumps += jumps.unwrap();
        self.pending_uses += uses.unwrap();
        for i in 0..2 {
            self.pending_attacks[i] += attacks[i].unwrap();
            self.pending_rope[i] += rope[i].unwrap();
        }
        self.last = input;
        self.sequence = sequence;
        self.received = Some(now);
        Ok(())
    }

    pub fn sample(&mut self, now: Instant) -> (Controls, Input) {
        if self
            .received
            .is_none_or(|at| now.saturating_duration_since(at) >= INPUT_TIMEOUT)
        {
            self.pending_jumps = 0;
            self.pending_uses = 0;
            self.pending_attacks = [0; 2];
            self.pending_rope = [0; 2];
            // Preserve the last view, but never keep walking or attacking.
            return (
                Controls::default(),
                Input {
                    yaw: self.last.yaw,
                    pitch: self.last.pitch,
                    ..Input::default()
                },
            );
        }
        let jump = self.pending_jumps > 0;
        let use_pressed = self.pending_uses > 0;
        self.pending_jumps = self.pending_jumps.saturating_sub(1);
        self.pending_uses = self.pending_uses.saturating_sub(1);
        let mut input = self.last;
        input.rope_edges = self.pending_rope.map(|n| u64::from(n > 0));
        for n in &mut self.pending_rope { *n = n.saturating_sub(1); }
        input.primary |= self.pending_attacks[0] > 0;
        input.alternate |= self.pending_attacks[1] > 0;
        for n in &mut self.pending_attacks {
            *n = n.saturating_sub(1);
        }
        (
            Controls {
                wish: self.last.wish,
                swim: self.last.swim,
                rise: self.last.rise,
                run: self.last.run,
                jump,
                use_pressed,
            },
            input,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rope_world() -> crate::collision::World {
        let mut world = crate::collision::World::fixture(&[]);
        world.traversal.ropes.push(crate::traversal::Rope { id:crate::entity::Id(1), model:1, enabled:true,
            origin:Vec3::ZERO, anchor:Vec3::Z*320., length:320. });
        world
    }
    #[test]
    fn delayed_grab_after_automatic_attachment_stays_attached_and_explicit_release_still_works() {
        let world = rope_world();
        let mut player = crate::movement::Player::new(Vec3::new(24.,0.,40.));
        let mut keys = Input { uses:1, ..Default::default() };
        keys.classify_rope_use(0,&player,&world,false);
        assert_eq!((keys.uses,keys.rope_edges),(0,[1,0]));
        // The host's ordinary airborne physics catches the rope while that
        // press is in flight. Apply the actual encoded/decoded input afterward.
        player.tick(&world, Controls::default());
        assert!(player.rope.is_some());
        let mut latch = RemoteInput::default(); let now = Instant::now();
        let received:Input = serde_json::from_slice(&serde_json::to_vec(&keys).unwrap()).unwrap();
        latch.accept(1,received,now).unwrap();
        let (mut movement,input) = latch.sample(now);
        movement.use_pressed |= input.rope_use(&player,&world);
        assert!(!movement.use_pressed);
        player.tick(&world,movement);
        assert!(player.rope.is_some(), "A delayed grab released an automatic grip");
        keys.uses += 1; keys.classify_rope_use(0,&player,&world,false);
        assert_eq!((keys.uses,keys.rope_edges),(0,[1,1]));
        latch.accept(2,keys,now).unwrap();
        let (mut movement,input) = latch.sample(now);
        movement.use_pressed |= input.rope_use(&player,&world);
        assert!(movement.use_pressed); player.tick(&world,movement);
        assert!(player.rope.is_none());
        assert!(!input.rope_use(&player,&world), "A delayed release cannot become a grab");
        latch.accept(3,keys,now).unwrap();
        assert_eq!(latch.sample(now).1.rope_edges,[0,0], "Acknowledged edges replayed");
    }
    #[test]
    fn rope_edges_are_bounded_expire_and_do_not_replace_dialogue_or_other_use_actions() {
        let now = Instant::now(); let mut latch = RemoteInput::default();
        let keys = Input { rope_edges:[2,1], ..Default::default() };
        latch.accept(1,keys,now).unwrap();
        assert_eq!(latch.sample(now).1.rope_edges,[1,1]);
        assert_eq!(latch.sample(now).1.rope_edges,[1,0]);
        assert_eq!(latch.sample(now).1.rope_edges,[0,0]);
        assert!(latch.accept(2,Input::default(),now).is_err());
        assert!(latch.accept(2,Input { rope_edges:[11,1],..keys },now).is_err());
        latch.accept(2,Input { rope_edges:[3,1],..keys },now).unwrap();
        assert_eq!(latch.sample(now+INPUT_TIMEOUT).1.rope_edges,[0,0]);
        let world = rope_world();
        let mut player = crate::movement::Player::new(Vec3::new(24.,0.,40.));
        let mut talking = Input { uses:1,..Default::default() };
        talking.classify_rope_use(0,&player,&world,true);
        assert_eq!((talking.uses,talking.rope_edges),(1,[0,0]));
        player.feet.x=500.;
        talking.classify_rope_use(0,&player,&world,false);
        assert_eq!((talking.uses,talking.rope_edges),(1,[0,0]));
    }
    #[test]
    fn host_envelope_preserves_integer_world_entity_keys() {
        let map = crate::interaction::tests::fixture();
        let world = crate::interaction::Interactions::load(&map).unwrap();
        let expected = serde_json::to_value(world.snapshot()).unwrap();
        let packet = HostMessage::Frame { frame: world.snapshot() };
        let bytes = super::super::codec::encode(&serde_json::to_vec(&packet).unwrap()).unwrap();
        let decoded = super::super::codec::decode(&bytes).unwrap();
        let restored: HostMessage<crate::interaction::Snapshot> = serde_json::from_slice(&decoded).unwrap();
        let HostMessage::Frame { frame } = restored else { panic!("Changed message kind"); };
        assert_eq!(serde_json::to_value(frame).unwrap(), expected);
    }
    #[test]
    fn attacks_survive_a_short_click_but_do_not_replay_or_outlive_input_timeout() {
        let now = Instant::now();
        let mut latch = RemoteInput::default();
        let input = Input {
            attack_presses: [1, 1],
            ..Default::default()
        };
        latch.accept(1, input, now).unwrap();
        let (_, sampled) = latch.sample(now);
        assert!(sampled.primary && sampled.alternate);
        latch.accept(1, input, now).unwrap();
        let (_, sampled) = latch.sample(now);
        assert!(!sampled.primary && !sampled.alternate);
        latch
            .accept(
                2,
                Input {
                    primary: true,
                    attack_presses: [2, 1],
                    ..input
                },
                now,
            )
            .unwrap();
        assert!(latch.sample(now).1.primary);
        assert!(latch.sample(now).1.primary);
        let (_, sampled) = latch.sample(now + INPUT_TIMEOUT);
        assert!(!sampled.primary && !sampled.alternate);
        assert!(
            latch.accept(3, input, now).is_err(),
            "Counters cannot roll backwards"
        );
    }
    #[test]
    fn held_scene_skip_releases_and_expires_with_the_other_controls() {
        let now=Instant::now();let mut latch=RemoteInput::default();
        let held=Input{skip_scene:true,..Default::default()};
        latch.accept(1,held,now).unwrap();
        assert!(latch.sample(now).1.skip_scene);
        latch.accept(2,Input::default(),now).unwrap();
        assert!(!latch.sample(now).1.skip_scene);
        latch.accept(3,held,now).unwrap();
        assert!(!latch.sample(now+INPUT_TIMEOUT).1.skip_scene);
        latch.accept(3,held,now+INPUT_TIMEOUT).unwrap();
        assert!(!latch.sample(now+INPUT_TIMEOUT).1.skip_scene);
        latch.accept(4,held,now+INPUT_TIMEOUT).unwrap();latch.cancel();
        assert!(!latch.sample(now+INPUT_TIMEOUT).1.skip_scene);
    }
    #[test]
    fn controls_cannot_grant_speed_or_upload_world_state() {
        assert!(Input {
            wish: Vec2::splat(1.),
            ..Input::default()
        }
        .validate()
        .is_err());
        assert!(Input {
            yaw: f32::NAN,
            ..Input::default()
        }
        .validate()
        .is_err());
        assert!(Input {
            weapon: Some(10),
            ..Input::default()
        }
        .validate()
        .is_err());
        let mut forged = serde_json::to_value(FromGuest::Input {
            epoch: 1,
            sequence: 1,
            sampled_micros: 0,
            controls: Input::default(),
        })
        .unwrap();
        forged["position"] = serde_json::json!([0, 0, 0]);
        assert!(serde_json::from_value::<FromGuest>(forged).is_err());
        assert!(serde_json::from_str::<FromGuest>(r#"{"type":"Enter","visit":{}}"#).is_err());
    }
    #[test]
    fn short_presses_survive_coalescing_but_duplicates_never_repeat_them() {
        let now = Instant::now();
        let mut input = RemoteInput::default();
        input
            .accept(
                1,
                Input {
                    jumps: 2,
                    uses: 1,
                    ..Input::default()
                },
                now,
            )
            .unwrap();
        let (first, _) = input.sample(now);
        assert!(first.jump && first.use_pressed);
        input
            .accept(
                1,
                Input {
                    jumps: 2,
                    uses: 1,
                    ..Input::default()
                },
                now,
            )
            .unwrap();
        let (second, _) = input.sample(now);
        assert!(second.jump && !second.use_pressed);
        assert!(!input.sample(now).0.jump);
        assert!(input.accept(2, Input::default(), now).is_err());
    }
    #[test]
    fn stalled_or_replayed_input_releases_movement_and_attack() {
        let now = Instant::now();
        let mut input = RemoteInput::default();
        input
            .accept(
                1,
                Input {
                    wish: Vec2::X,
                    primary: true,
                    jumps: 1,
                    yaw: 1.,
                    ..Input::default()
                },
                now,
            )
            .unwrap();
        assert_eq!(input.sample(now).0.wish, Vec2::X);
        input
            .accept(1, Input::default(), now + INPUT_TIMEOUT)
            .unwrap();
        let (movement, buttons) = input.sample(now + INPUT_TIMEOUT);
        assert_eq!(movement.wish, Vec2::ZERO);
        assert!(!movement.jump && !buttons.primary);
        assert_eq!(buttons.yaw, 1.);
        input
            .accept(
                2,
                Input {
                    jumps: 1,
                    ..Input::default()
                },
                now + INPUT_TIMEOUT,
            )
            .unwrap();
        assert!(!input.sample(now + INPUT_TIMEOUT).0.jump);
    }
    #[test]
    fn different_builds_data_and_unknown_visits_are_rejected() {
        let current = Identity::current("0".repeat(64));
        current.compatible(&current).unwrap();
        let mut other = current.clone();
        other.build = "1".repeat(64);
        assert!(current.compatible(&other).is_err());
        other = current.clone();
        other.game_data = "2".repeat(64);
        assert!(current.compatible(&other).is_err());
        let mut visit = Visit {
            epoch: 1,
            map: "gvillage".into(),
            entry: None,
            difficulty: Difficulty::Normal,
            combat_preview: false,
        };
        visit.validate().unwrap();
        visit.map = "../../outside".into();
        assert!(visit.validate().is_err());
    }
}
