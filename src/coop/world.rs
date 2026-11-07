//! Host-owned bodies and individual resources. Shared controllers must advance
//! once around this per-player step, never once for each client/render window.
use super::{
    protocol::Input,
    rules::{DeathEvent, Lives, RESPAWN_HEALTH},
    PlayerId,
};
use crate::{
    collision::World,
    inventory::Stats,
    movement::{Controls, Player, FIXED_DT},
};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use crate::movement::EYE_HEIGHT;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Body {
    pub player: Player,
    pub stats: Stats,
    pub yaw: f32,
    pub pitch: f32,
    pub running: bool,
    /// Host tick of the most recent spawn. Presentation must never interpolate
    /// through a death/respawn or checkpoint restore, even over short distances.
    pub spawned_at: u64,
    /// Teleports and scene reunions also cut visual interpolation, but are not deaths.
    pub warped_at: u64,
}
impl Body {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.player.feet.is_finite()
                && self.player.velocity.is_finite()
                && self.yaw.is_finite()
                && self.pitch.is_finite()
                && self.pitch.abs() <= 1.56,
            "Invalid co-op player state."
        );
        self.player.validate_save()?;
        self.stats.validate_save()
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub epoch: u64,
    pub tick: u64,
    pub acknowledged_input: u64,
    pub checkpoint_generation: u64,
    pub players: [Body; 2],
    pub lives: Lives,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat: Option<Box<super::arena::Snapshot>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sounds: Option<super::audio::Batch>,
}
impl Frame {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.epoch > 0, "Invalid co-op visit.");
        ensure!(self.checkpoint_generation <= self.tick, "Invalid co-op checkpoint generation.");
        for body in &self.players {
            body.validate()?;
            ensure!(body.spawned_at <= self.tick, "Co-op spawn is in the future.");
            ensure!(body.warped_at <= self.tick, "Co-op relocation is in the future.");
        }
        if let Some(sounds) = &self.sounds { sounds.validate(self.tick)?; }
        self.lives.validate()
    }
}

pub struct Roster {
    pub players: [Body; 2],
    pub lives: Lives,
    pub tick: u64,
    pub checkpoint_generation: u64,
    checkpoint: [Body; 2],
}
/// One world clock for machinery, enemies, particles and audio. Only living
/// Watch owners can stop it. Overlapping freezes end at the later expiry;
/// their durations must not be added when both expire within the same frame.
pub fn world_dt(players: &[Body; 2], dt: f32) -> f32 {
    players.iter().filter(|body| body.stats.alive())
        .map(|body| body.stats.powers.world_dt(dt)).fold(dt, f32::min)
}
impl Roster {
    pub fn from_checkpoint(frame: &Frame, world: &World, safe: impl Fn(Vec3) -> bool) -> Result<Self> {
        frame.validate()?;
        ensure!(frame.players.iter().all(|b| b.stats.alive())
            && frame.lives.checkpoint_ready(),
            "Both players must be alive to save a co-op checkpoint");
        for body in &frame.players {
            body.player.validate_world(world)?;
            ensure!(world.body_clear(body.player.feet) && safe(body.player.feet),
                "A co-op checkpoint player is obstructed or in a hazard");
        }
        Ok(Self {
            checkpoint: frame.players.clone(), players: frame.players.clone(), lives: Lives::default(),
            tick: frame.tick, checkpoint_generation: frame.checkpoint_generation,
        })
    }
    /// Blue retains the complete live body; only the new partner is placed.
    pub(super) fn from_single_player(world: &World, player: &Player, stats: &Stats,
        yaw: f32, pitch: f32, safe: impl Fn(Vec3) -> bool) -> Result<Self> {
        ensure!(stats.alive(), "Retry or load a living Alice before hosting this game");
        player.validate_world(world)?;
        ensure!(world.body_clear(player.feet) && safe(player.feet),
            "Move Alice to a safe position before hosting this game");
        let partner = super::placement::revival(world, player, &safe)
            .or_else(|| super::placement::reunion(world, player, &safe))
            .context("There is no safe space beside Alice; move to a wider area before hosting")?;
        let players = [player.clone(), partner].map(|player| Body {
            player, stats: stats.clone(), yaw, pitch, running: false,
            spawned_at: 0, warped_at: 0,
        });
        let frame = Frame { epoch: 1, tick: 0, acknowledged_input: 0,
            checkpoint_generation: 0, players, lives: Lives::default(), combat: None, sounds: None };
        Self::from_checkpoint(&frame, world, safe)
    }
    pub fn remember_checkpoint(&mut self) {
        self.checkpoint = self.players.clone();
    }
    /// Real exits carry each player's earned inventory and resources separately.
    /// A dead partner keeps the same countdown while the next visit loads.
    #[cfg(test)]
    pub fn arrive(&self, world: &World, eye: Vec3, yaw: f32, map: &str, entry: Option<&str>) -> Result<Self> {
        self.arrive_with(world, eye, yaw, map, entry, |_| true)
    }
    pub fn arrive_with(&self, world: &World, eye: Vec3, yaw: f32, map: &str, entry: Option<&str>, safe: impl Fn(Vec3) -> bool) -> Result<Self> {
        let mut next = Self::new_with(world, eye, yaw, self.players[0].stats.clone(), safe)?;
        next.tick = self.tick;
        next.checkpoint_generation = self.checkpoint_generation;
        next.lives = self.lives.clone();
        for (body, old) in next.players.iter_mut().zip(&self.players) {
            body.stats = old.stats.clone();
            // A scripted health floor belongs to the visit being left. The new
            // controller must supply its own rule before movement/combat.
            body.stats.minimum_sanity = 0.;
            body.stats.ensure_level_weapons(map, entry);
            body.spawned_at = old.spawned_at;
            body.warped_at = self.tick;
        }
        next.checkpoint = next.players.clone();
        // Both bodies must be living when restoring this entrance checkpoint.
        // A partner who was dead during the exit retains their own inventory.
        for body in &mut next.checkpoint {
            if !body.stats.alive() { body.stats.set_health(RESPAWN_HEALTH)?; }
        }
        Ok(next)
    }
    pub fn new(world: &World, eye: Vec3, yaw: f32, stats: Stats) -> Result<Self> {
        Self::new_with(world, eye, yaw, stats, |_| true)
    }
    pub fn new_with(world: &World, eye: Vec3, yaw: f32, stats: Stats, safe: impl Fn(Vec3) -> bool) -> Result<Self> {
        let blue = Player::spawn(world, eye).context("Alice's entrance is obstructed.")?;
        ensure!(safe(blue.feet), "Alice's entrance is hazardous.");
        let red = super::placement::entrance(world, blue.feet, safe)
            .context("There is no safe space for player two at this entrance.")?;
        let players = [blue, red].map(|player| Body {
            player,
            stats: stats.clone(),
            yaw,
            pitch: 0.,
            running: false,
            spawned_at: 0,
            warped_at: 0,
        });
        Ok(Self {
            checkpoint: players.clone(),
            players,
            lives: Default::default(),
            tick: 0,
            checkpoint_generation: 0,
        })
    }
    #[cfg(test)]
    pub fn step(&mut self, world: &World, inputs: [(Controls, Input); 2]) -> Result<()> {
        self.move_players(world, inputs);
        self.finish_tick(world)
    }
    /// Shared combat resolves after movement and before the one death-policy step.
    pub fn move_players(&mut self, world: &World, inputs: [(Controls, Input); 2]) {
        self.move_players_with_locks(world, inputs, [false; 2]);
    }
    /// A boss hold locks only its captured actor; personal clocks still advance.
    pub fn move_players_with_locks(&mut self, world: &World, inputs: [(Controls, Input); 2], locked: [bool; 2]) {
        self.tick += 1;
        let previous_players = self.players.each_ref().map(|body| body.player.clone());
        let previous = previous_players.each_ref().map(|player| player.feet);
        for ((body, (controls, look)), locked) in self.players.iter_mut().zip(inputs).zip(locked) {
            if !body.stats.alive() {
                continue;
            }
            body.yaw = look.yaw;
            body.pitch = look.pitch;
            body.running = controls.run;
            body.stats.update(FIXED_DT);
            body.stats.prepare_player(&mut body.player);
            if locked { body.running = false; continue; }
            let landings = body.player.landings;
            body.player.tick(world, controls);
            body.stats
                .damage(std::mem::take(&mut body.player.liquid_damage));
            if body.player.landings != landings {
                body.stats.fall(body.player.landing_speed);
            }
            if crate::recovery::outside_world(world, body.player.feet) {
                body.stats.damage(10000.);
            }
        }
        super::separation::wait_for_climbers(world, &mut self.players, &previous_players);
        super::separation::resolve(world, &mut self.players, previous);
    }
    #[cfg(test)]
    pub fn finish_tick(&mut self, world: &World) -> Result<()> {
        self.finish_tick_with(world, false, |_| true)
    }
    pub fn finish_tick_with(&mut self, world: &World, cinematic: bool, safe: impl Fn(Vec3) -> bool) -> Result<()> {
        self.finish_tick_with_spawn_hold(world, cinematic, false, safe)
    }
    /// Interactive solo puzzles keep counting down, but cannot insert a second
    /// body until their safe handoff. Cinematics retain their existing pause.
    pub fn finish_tick_with_spawn_hold(&mut self, world: &World, cinematic: bool,
        hold_spawn: bool, safe: impl Fn(Vec3) -> bool) -> Result<()> {
        let events = self
            .lives
            .tick(self.players.each_ref().map(|p| p.stats.alive()), cinematic);
        if events.contains(&DeathEvent::RestoreCheckpoint) {
            self.checkpoint_generation += 1;
            self.players = self.checkpoint.clone();
            for body in &mut self.players { body.spawned_at = self.tick; }
            ensure!(
                self.players.iter().all(|p| world.body_clear(p.player.feet)),
                "Co-op checkpoint is obstructed."
            );
            self.lives.checkpoint_restored([true; 2]);
        } else if let Some((dead, survivor)) = self.lives.pending_spawn().filter(|_| !cinematic && !hold_spawn) {
            if let Some(player) = super::placement::revival(world, &self.players[survivor.index()].player, safe) {
                let body = &mut self.players[dead.index()];
                body.player = player;
                body.stats.set_health(RESPAWN_HEALTH)?;
                body.spawned_at = self.tick;
                self.lives.respawned(dead);
            }
        }
        Ok(())
    }
    pub fn frame(&self, epoch: u64, acknowledged_input: u64) -> Frame {
        Frame {
            epoch,
            tick: self.tick,
            acknowledged_input,
            checkpoint_generation: self.checkpoint_generation,
            players: self.players.clone(),
            lives: self.lives.clone(),
            combat: None,
            sounds: None,
        }
    }
}

/// Smooth normal network movement, but snap to a newly spawned body. The
/// authoritative frame, health and spawn clock are never changed by rendering.
pub fn interpolate_body(body: &mut Body, previous: &Body, elapsed: f32) {
    if body.spawned_at == previous.spawned_at
        && body.warped_at == previous.warped_at
        && body.stats.alive() == previous.stats.alive()
        && body.player.feet.distance(previous.player.feet) < 128.
    {
        body.player.feet = previous.player.feet.lerp(body.player.feet, (elapsed * 20.).clamp(0., 1.));
    }
}

/// Clear placement close to the survivor on supported ground or in the same
/// safe swimming volume, without crossing a wall or dropping to a distant floor.
#[cfg(test)]
pub fn safe_near(world: &World, feet: Vec3) -> Option<Player> {
    safe_near_with(world, feet, |_| true)
}

/// Read-only recovery evidence using co-op's actual slope/edge placement rules.
/// A supported footprint alone must not label a floating body as grounded.
pub(crate) fn revival_supported(world: &World, player: &Player) -> bool {
    if !player.grounded || !super::placement::supported(world, player.feet) { return false; }
    let floor=world.body_trace(player.feet,player.feet-Vec3::Z*3.);
    !floor.start_solid && floor.fraction<1. && floor.normal.z>=0.65
}
pub fn safe_near_with(world: &World, feet: Vec3, safe: impl Fn(Vec3) -> bool) -> Option<Player> {
    super::placement::near(world, feet, 32., safe)
}

pub fn camera_player(frame: &Frame, local: PlayerId) -> PlayerId {
    if frame.players[local.index()].stats.alive() {
        local
    } else {
        local.partner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn floor() -> World {
        World::fixture(&[(vec3(-1000., -1000., -100.), vec3(1000., 1000., 0.))])
    }
    #[test]
    fn shared_watch_ignores_dead_owners_and_overlapping_expiries_are_not_added() {
        let mut r = Roster::new(&floor(), Vec3::Z * EYE_HEIGHT, 0., Stats::default()).unwrap();
        for owner in 0..2 {
            r.players[owner].stats.powers.stopped = 20.;
            r.players[owner].stats.powers.recharge = 360.;
            assert_eq!(world_dt(&r.players, FIXED_DT), 0.);
            r.players[owner].stats.damage(1000.);
            assert_eq!(world_dt(&r.players, FIXED_DT), FIXED_DT);
            r.players[owner].stats = Stats::default();
        }
        for body in &mut r.players {
            body.stats.powers.stopped = FIXED_DT * 0.25;
            body.stats.powers.recharge = 1.;
        }
        assert_eq!(world_dt(&r.players, FIXED_DT), FIXED_DT * 0.75);
        r.players[1].stats.powers.stopped = FIXED_DT * 0.5;
        assert_eq!(world_dt(&r.players, FIXED_DT), FIXED_DT * 0.5);
        r.players[1].stats.damage(1000.);
        assert_eq!(world_dt(&r.players, FIXED_DT), FIXED_DT * 0.75);
        assert_eq!(world_dt(&r.players, 0.), 0.);
    }
    #[test]
    fn either_player_revives_on_a_partners_rope_after_ten_seconds_with_own_resources() {
        use crate::{movement::RopeGrip, traversal::Rope};
        let mut world=floor();
        world.traversal.ropes.push(Rope {id:crate::entity::Id(1),model:1,enabled:true,
            origin:Vec3::ZERO,anchor:Vec3::Z*600.,length:500.});
        for dead in PlayerId::ALL {
            let mut roster=Roster::new(&world,Vec3::Z*(EYE_HEIGHT+0.04),0.,Stats::default()).unwrap();
            let survivor=dead.partner();
            let mut hanging=Player::new(Vec3::Z*300.);
            hanging.rope=Some(RopeGrip {id:crate::entity::Id(1),length:260.,rise:0.});
            hanging.script_motion=1;
            roster.players[survivor.index()].player=hanging;
            roster.players[dead.index()].stats.spend_will(17.);
            roster.players[dead.index()].stats.damage(1000.);
            let mut expected=roster.players[dead.index()].stats.clone();
            expected.set_health(RESPAWN_HEALTH).unwrap();
            roster.step(&world,Default::default()).unwrap();
            let death_tick=roster.tick;
            for _ in 0..super::super::rules::RESPAWN_TICKS-1 {
                assert_eq!(camera_player(&roster.frame(1,0),dead),survivor);
                roster.step(&world,Default::default()).unwrap();
                assert!(!roster.players[dead.index()].stats.alive());
            }
            roster.step(&world,Default::default()).unwrap();
            let revived=&roster.players[dead.index()];
            assert_eq!(revived.spawned_at,death_tick+1200);
            assert_eq!(serde_json::to_value(&revived.stats).unwrap(),serde_json::to_value(&expected).unwrap());
            assert_eq!(camera_player(&roster.frame(1,0),dead),dead);
            assert_eq!(revived.player.rope.as_ref().unwrap().id,crate::entity::Id(1));
            assert!(revived.player.feet.distance(roster.players[survivor.index()].player.feet)>71.9);
            roster.frame(1,0).validate().unwrap();
            revived.player.validate_world(&world).unwrap();
            for _ in 0..120 { roster.step(&world,Default::default()).unwrap(); }
            assert!(roster.players.iter().all(|b|b.stats.alive() && b.player.rope.is_some()));
            assert_eq!(roster.checkpoint_generation,0);
        }
    }
    #[test]
    fn checkpoint_resume_and_wipe_keep_distinct_resources_and_reject_dead_or_embedded_players() {
        let world = floor();
        let mut roster = Roster::new(&world, Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        for _ in 0..240 { roster.step(&world, Default::default()).unwrap(); }
        roster.players[0].stats.set_health(83.).unwrap();
        roster.players[1].stats.set_health(47.).unwrap();
        roster.players[0].stats.spend_will(5.);
        roster.players[1].stats.spend_will(11.);
        let saved = roster.frame(3, 0);
        let mut resumed = Roster::from_checkpoint(&saved, &world, |_| true).unwrap();
        assert_eq!(resumed.tick, 240);
        resumed.players[0].player.feet += Vec3::Y * 200.;
        for body in &mut resumed.players { body.stats.damage(10000.); }
        resumed.step(&world, Default::default()).unwrap();
        assert_eq!(resumed.checkpoint_generation, 1);
        for (body, expected) in resumed.players.iter().zip(&saved.players) {
            assert_eq!(serde_json::to_value(&body.stats).unwrap(), serde_json::to_value(&expected.stats).unwrap());
            assert_eq!(body.player.feet, expected.player.feet);
            assert_eq!(body.spawned_at, resumed.tick);
        }
        let mut invalid = saved.clone();
        invalid.players[1].stats.damage(10000.);
        assert!(Roster::from_checkpoint(&invalid, &world, |_| true).is_err());
        invalid = saved.clone();
        invalid.players[1].player.feet.z = -50.;
        assert!(Roster::from_checkpoint(&invalid, &world, |_| true).is_err());
        assert!(Roster::from_checkpoint(&saved, &world, |_| false).is_err());
        invalid = saved.clone();
        invalid.lives.tick([true, false], false);
        assert!(Roster::from_checkpoint(&invalid, &world, |_| true).is_err());
    }
    #[test]
    fn solo_import_preserves_body_and_independent_inventory_without_moving_blue() {
        let world = floor();
        let mut player = Player::spawn(&world, Vec3::Z * (EYE_HEIGHT + 0.04)).unwrap();
        player.velocity = Vec3::X * 21.;
        player.jumps = 9;
        let mut stats = Stats::default();
        stats.set_health(63.).unwrap();
        stats.spend_will(12.);
        let mut roster = Roster::from_single_player(&world, &player, &stats, 1.1, 0.3, |_| true).unwrap();
        assert_eq!(serde_json::to_value(&roster.players[0].player).unwrap(), serde_json::to_value(&player).unwrap());
        assert_eq!(roster.players[0].yaw, 1.1);
        assert_eq!(roster.players[0].pitch, 0.3);
        assert!(roster.players[0].player.feet.distance(roster.players[1].player.feet) >= 40.);
        roster.players[1].stats.damage(20.);
        roster.players[1].stats.spend_will(5.);
        assert_eq!(serde_json::to_value(&roster.players[0].stats).unwrap(), serde_json::to_value(&stats).unwrap());
        assert!(Roster::from_single_player(&world, &player, &stats, 0., 0., |_| false).is_err());
        stats.damage(10000.);
        assert!(Roster::from_single_player(&world, &player, &stats, 0., 0., |_| true).is_err());
    }
    #[test]
    fn shared_combat_damage_resolves_before_the_single_death_decision() {
        let world = floor();
        let mut roster =
            Roster::new(&world, Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        roster.move_players(&world, Default::default());
        for body in &mut roster.players {
            body.stats.damage(1000.);
        }
        roster.finish_tick(&world).unwrap();
        assert_eq!(roster.tick, 1);
        assert_eq!(roster.checkpoint_generation, 1);
        assert!(roster
            .players
            .iter()
            .all(|p| p.stats.alive() && p.stats.sanity() == 100.));
    }
    #[test]
    fn a_shared_consumable_remains_available_to_each_player_once() {
        let world = floor();
        let item = crate::inventory::Pickup {
            id: "coop:health".into(),
            model: "p_h1".into(),
            origin: Vec3::ZERO,
            kind: crate::inventory::PickupKind::Sanity,
            amount: 10.,
        };
        let mut players = [Stats::default(), Stats::default()];
        for p in &mut players {
            p.set_health(20.).unwrap();
        }
        assert_eq!(
            crate::inventory::collect(
                &mut players[0],
                std::slice::from_ref(&item),
                Vec3::ZERO,
                &world
            )
            .len(),
            1
        );
        assert_eq!(players[1].sanity(), 20.);
        assert_eq!(
            crate::inventory::collect(
                &mut players[1],
                std::slice::from_ref(&item),
                Vec3::ZERO,
                &world
            )
            .len(),
            1
        );
        assert!(crate::inventory::collect(&mut players[0], &[item], Vec3::ZERO, &world).is_empty());
        assert_eq!(players[0].sanity(), 30.);
        assert_eq!(players[1].sanity(), 30.);
    }
    #[test]
    fn leaving_a_scripted_health_floor_preserves_resources_without_protecting_the_next_visit() {
        let world=floor();
        let mut roster=Roster::new(&world,Vec3::Z*(EYE_HEIGHT+0.04),0.,
            Stats::for_level("centipede2",Some("centipede2_start1"))).unwrap();
        for (i,b) in roster.players.iter_mut().enumerate() {
            b.stats.set_health(if i==0 {20.} else {5.}).unwrap();
            b.stats.spend_will(13.+i as f32);
            b.stats.minimum_sanity=if i==0 {10.} else {30.};
        }
        let resources=roster.players.each_ref().map(|b|serde_json::to_value(&b.stats).unwrap());
        let mut next=roster.arrive(&world,Vec3::Z*(EYE_HEIGHT+0.04),0.,"centipede2",Some("centipede2_start1")).unwrap();
        for (b,s) in next.players.iter_mut().zip(resources) {
            assert_eq!(serde_json::to_value(&b.stats).unwrap(),s);
            assert_eq!(b.stats.minimum_sanity,0.);
            b.stats.damage(10000.);
            assert!(!b.stats.alive());
        }
        assert!(next.checkpoint.iter().all(|b|b.stats.minimum_sanity==0.));
    }
    #[test]
    fn captured_player_stays_put_while_partner_and_personal_clocks_continue() {
        let world = floor();
        for captured in 0..2 {
            let mut roster = Roster::new(&world, Vec3::Z*(EYE_HEIGHT+0.04), 0., Stats::default()).unwrap();
            roster.players[captured].stats.invisible = 2.;
            let before = roster.players.each_ref().map(|b| b.player.feet);
            let inputs = [(Controls { wish: Vec2::Y, run: true, ..Default::default() }, Input::default()); 2];
            for _ in 0..120 {
                roster.move_players_with_locks(&world, inputs, std::array::from_fn(|i| i == captured));
            }
            assert_eq!(roster.players[captured].player.feet, before[captured]);
            assert!(!roster.players[captured].running);
            assert!(roster.players[1-captured].player.feet.distance(before[1-captured]) > 100.);
            assert!((roster.players[captured].stats.invisible-1.).abs() < 0.01);
            assert_eq!(roster.tick, 120);
        }
    }
    #[test]
    fn two_bodies_share_the_host_clock_but_keep_separate_motion_and_health() {
        let world = floor();
        let mut roster =
            Roster::new(&world, Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        let start = roster.players[1].player.feet;
        for _ in 0..120 {
            roster
                .step(
                    &world,
                    [
                        (
                            Controls {
                                wish: Vec2::Y,
                                ..Default::default()
                            },
                            Input::default(),
                        ),
                        Default::default(),
                    ],
                )
                .unwrap();
        }
        assert!((roster.players[0].player.feet.y - 104.).abs() < 5.);
        assert!(roster.players[1].player.feet.distance(start) < 0.1);
        roster.players[1].stats.damage(10.);
        assert_eq!(roster.players[0].stats.sanity(), 100.);
        assert_eq!(roster.players[1].stats.sanity(), 90.);
    }
    #[test]
    fn interactive_spawn_hold_keeps_counting_and_releases_one_safe_revival() {
        let world = floor();
        for dead in PlayerId::ALL {
            let mut roster = Roster::new(&world, Vec3::Z*(EYE_HEIGHT+0.04), 0., Stats::default()).unwrap();
            roster.players[dead.index()].stats.damage(1000.);
            for _ in 0..=super::super::rules::RESPAWN_TICKS {
                roster.finish_tick_with_spawn_hold(&world, false, true, |_| true).unwrap();
                assert!(!roster.players[dead.index()].stats.alive());
            }
            assert_eq!(roster.lives.state(dead), super::super::rules::Life::AwaitingSafeSpawn);
            roster.finish_tick_with_spawn_hold(&world, false, false, |_| true).unwrap();
            assert_eq!(roster.players[dead.index()].stats.sanity(), RESPAWN_HEALTH);
            assert_eq!(roster.lives.state(dead), super::super::rules::Life::Alive);
            assert_eq!(roster.checkpoint_generation, 0);
        }
    }
    #[test]
    fn death_spectates_then_respawns_and_a_double_death_restores_both() {
        let world = floor();
        let mut roster =
            Roster::new(&world, Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        roster.players[1].stats.damage(1000.);
        roster.step(&world, Default::default()).unwrap();
        assert_eq!(
            camera_player(&roster.frame(1, 0), PlayerId::Red),
            PlayerId::Blue
        );
        for _ in 0..1199 {
            roster.step(&world, Default::default()).unwrap();
        }
        assert!(!roster.players[1].stats.alive());
        roster.step(&world, Default::default()).unwrap();
        assert_eq!(roster.players[1].stats.sanity(), RESPAWN_HEALTH);
        assert_eq!(roster.players[1].spawned_at, 1201);
        assert_eq!(roster.lives.state(PlayerId::Red), super::super::rules::Life::Alive);
        assert!(
            roster.players[0]
                .player
                .feet
                .distance(roster.players[1].player.feet)
                >= 40.
        );
        for body in &mut roster.players {
            body.stats.damage(1000.);
        }
        roster.step(&world, Default::default()).unwrap();
        assert!(roster.players.iter().all(|p| p.stats.sanity() == 100.));
        assert!(roster.players.iter().all(|p| p.spawned_at == roster.tick));
        assert_eq!(roster.frame(1, 0).checkpoint_generation, 1);
    }
    #[test]
    fn short_respawns_are_not_blended_with_a_dead_body() {
        let world = floor();
        let roster = Roster::new(&world, Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        let old = roster.players[0].clone();
        let mut moved = old.clone();
        moved.player.feet += Vec3::X * 48.;
        let destination = moved.player.feet;
        let mut normal = moved.clone();
        interpolate_body(&mut normal, &old, 0.01);
        assert!(normal.player.feet.distance(destination) > 30.);
        moved.spawned_at = 1201;
        interpolate_body(&mut moved, &old, 0.01);
        assert_eq!(moved.player.feet, destination);
        let mut frame = roster.frame(1, 0);
        frame.players[0].spawned_at = frame.tick + 1;
        assert!(frame.validate().is_err());
    }
    #[test]
    fn no_safe_neighbor_does_not_place_a_body_over_a_drop() {
        let small = World::fixture(&[(vec3(-20., -20., -100.), vec3(20., 20., 0.))]);
        assert!(safe_near(&small, Vec3::Z * 0.04).is_none());
    }

    #[test]
    fn a_scene_holds_pending_respawn_and_hazardous_landings_are_retried() {
        let world = floor();
        let mut roster = Roster::new(&world, Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        roster.players[1].stats.damage(1000.);
        for _ in 0..1201 {
            roster.tick += 1;
            roster.finish_tick_with(&world, false, |_| false).unwrap();
        }
        assert_eq!(roster.lives.state(PlayerId::Red), super::super::rules::Life::AwaitingSafeSpawn);
        for _ in 0..120 {
            roster.tick += 1;
            roster.finish_tick_with(&world, true, |_| true).unwrap();
            assert!(!roster.players[1].stats.alive());
        }
        let origin = roster.players[0].player.feet;
        roster.finish_tick_with(&world, false, |at| at.x < origin.x - 40.).unwrap();
        assert_eq!(roster.players[1].stats.sanity(), RESPAWN_HEALTH);
        assert!(roster.players[1].player.feet.x < origin.x - 40.);
    }

    #[test]
    fn relocation_snaps_without_reporting_a_new_life_and_exits_keep_individual_stats() {
        let world = floor();
        let mut roster = Roster::new(&world, Vec3::Z * (EYE_HEIGHT + 0.04), 0., Stats::default()).unwrap();
        roster.players[0].stats.set_health(73.).unwrap();
        roster.players[1].stats.set_health(0.).unwrap();
        roster.step(&world, Default::default()).unwrap();
        let old = roster.players[0].clone();
        let next = roster.arrive(&world, vec3(0., 48., EYE_HEIGHT + 0.04), 1., "skool2", None).unwrap();
        assert_eq!(next.players[0].stats.sanity(), 73.);
        assert_eq!(next.players[1].stats.sanity(), 0.);
        assert_eq!(next.lives.state(PlayerId::Red), roster.lives.state(PlayerId::Red));
        assert_eq!(next.players[0].spawned_at, old.spawned_at);
        assert_eq!(next.players[0].warped_at, next.tick);
        let mut visual = next.players[0].clone();
        interpolate_body(&mut visual, &old, 0.01);
        assert_eq!(visual.player.feet, next.players[0].player.feet);
        let mut invalid = next.frame(2, 0);
        invalid.players[0].warped_at += 1;
        assert!(invalid.validate().is_err());
    }
}
