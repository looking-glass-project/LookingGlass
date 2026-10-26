use super::*;
use crate::{
    ant::Timing,
    burrow::{Insect, Kind},
    combat::{self, DamageKind},
};
const STEP: f32 = 1. / 120.;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Action {
    Walk,
    Ready,
    Crush,
    Spit,
    Larvae,
    Charge,
    Strike,
    Shake,
    Return,
    Pain,
    Death,
    Thrash,
    Dead,
}
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Shot {
    pub at: Vec3,
    pub velocity: Vec3,
    pub age: f32,
    pub ended: Option<f32>,
    pub start: Vec3,
    pub fragment: bool,
    pub normal: Vec3,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub(super) struct Paired {
    pub target: Option<usize>,
    pub grab: Option<usize>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Battle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coop: Option<Paired>,
    pub pose: Transform,
    pub action: Action,
    pub time: f32,
    pub hits: u8,
    pub stage: u8,
    pub cycle: u8,
    pub variant: u8,
    pub thrashes: u8,
    pub grabbed: bool,
    pub grab_camera: Vec3,
    pub grab_feet: Vec3,
    pub victim_feet: Vec3,
    pub larvae: Vec<Insect>,
    pub shots: Vec<Shot>,
    pub random: u32,
    pub accumulator: f64,
    pub serial: u32,
    pub pain_at: Vec3,
    pub blood_age: f32,
    pub decision: f32,
    pub quake: f32,
}
impl Battle {
    pub fn new(pose: Transform) -> Self {
        Self {
            coop: None,
            pose,
            action: Action::Walk,
            time: 0.,
            hits: 0,
            stage: 1,
            cycle: 0,
            variant: 0,
            thrashes: 0,
            grabbed: false,
            grab_camera: Vec3::ZERO,
            grab_feet: Vec3::ZERO,
            victim_feet: Vec3::ZERO,
            larvae: vec![],
            shots: vec![],
            random: 0xc312f2,
            accumulator: 0.,
            serial: 0,
            pain_at: Vec3::ZERO,
            blood_age: 5.,
            decision: 1.,
            quake: 0.,
        }
    }
    pub fn validate(&self) -> Result<()> {
        if let Some(c) = &self.coop {
            ensure!(c.target.is_none_or(|i| i < 2) && c.grab.is_none_or(|i| i < 2)
                && c.grab.is_some() == self.grabbed, "Invalid paired Centipede victim");
        }
        ensure!(
            self.pose.translation.is_finite()
                && self.pose.translation.abs().max_element() < 20000.
                && self.pose.rotation.is_finite()
                && (self.pose.rotation.length() - 1.).abs() < 0.01,
            "Invalid Centipede pose"
        );
        ensure!(
            self.hits <= 6
                && (1..=3).contains(&self.stage)
                && self.cycle < 3
                && self.variant < 2
                && self.thrashes <= 3,
            "Invalid boss stage"
        );
        ensure!(
            self.stage
                == ((self
                    .hits
                    .saturating_sub(u8::from(self.action == Action::Pain))
                    / 2)
                    + 1)
                .min(3),
            "Inconsistent boss hit stage"
        );
        ensure!(
            (self.hits == 6)
                == matches!(self.action, Action::Death | Action::Thrash | Action::Dead),
            "Invalid death state"
        );
        ensure!(
            !self.grabbed || self.action == Action::Shake,
            "Invalid grabbed state"
        );
        ensure!(
            self.accumulator.is_finite() && (0. ..0.009).contains(&self.accumulator),
            "Invalid boss simulation remainder"
        );
        for (name, t, max) in [
            ("action", self.time, 3600.),
            ("blood", self.blood_age, 5.),
            ("decision", self.decision, 5.),
            ("quake", self.quake, 1.),
        ] {
            state::clock(name, t, max)?;
        }
        ensure!(
            self.grab_camera.is_finite()
                && self.grab_feet.is_finite()
                && self.victim_feet.is_finite()
                && self.pain_at.is_finite(),
            "Invalid boss attachment"
        );
        ensure!(
            self.larvae.len() <= 16 && self.shots.len() <= 128,
            "Too many boss projectiles"
        );
        for l in &self.larvae {
            l.validate()?;
            ensure!(l.kind == Kind::Larva, "Invalid boss larva");
        }
        for s in &self.shots {
            ensure!(
                s.at.is_finite()
                    && s.start.is_finite()
                    && s.velocity.is_finite()
                    && s.velocity.length() < 1200.
                    && s.normal.is_finite(),
                "Invalid acid trajectory"
            );
            state::clock("acid", s.age, 12.)?;
            if let Some(t) = s.ended {
                ensure!((0. ..=s.age).contains(&t), "Invalid acid impact");
            }
        }
        Ok(())
    }
    fn roll(&mut self, n: u32) -> bool {
        self.random = self.random.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.random >> 16) % 100 < n
    }
    pub fn set(&mut self, a: Action) {
        self.action = a;
        self.time = 0.;
        self.serial = self.serial.wrapping_add(1);
    }
    pub fn clip(&self) -> &'static str {
        match self.action {
            Action::Walk => {
                if self.stage == 1 {
                    "walk_fast"
                } else {
                    "walk"
                }
            }
            Action::Ready => "pain_front",
            Action::Crush => "attack_crush",
            Action::Spit => "attack_spit",
            Action::Larvae => "attack_spit_buggies",
            Action::Charge => "attack_juggernaught",
            Action::Strike => "attack_grab_strike",
            Action::Shake => "attack_grab_shake",
            Action::Return => "attack_grab_return",
            Action::Pain => match self.stage {
                2 => "pain_rise_right",
                3 => "pain_rise_big",
                _ => {
                    if self.variant == 0 {
                        "pain_rise_front"
                    } else {
                        "pain_rise_big"
                    }
                }
            },
            Action::Death => "death_start",
            Action::Thrash => "death_thrash",
            Action::Dead => "death_end",
        }
    }
    pub fn weak(&self, d: &data::Data) -> bool {
        self.action == Action::Crush
            && self.time < 25. * d.boss().frame("attack_crush")
            && self.hits < 6
    }
    pub fn tag(&self, d: &data::Data, n: &str) -> Transform {
        let time = if self.loops() {
            self.time % d.boss().duration(self.clip())
        } else {
            self.time
        };
        d.boss().tag(n, self.clip(), time, self.pose)
    }
    pub fn loops(&self) -> bool {
        matches!(self.action, Action::Walk | Action::Charge)
    }
    pub fn targets(&self, d: &data::Data) -> Vec<Target> {
        let mut targets = vec![];
        if self.hits < 6 {
            for (i, (tag, min, max)) in [
                ("tag_target", vec3(-64., -64., -16.), vec3(64., 64., 64.)),
                ("tag_target", vec3(-80., -80., 16.), vec3(80., 80., 64.)),
                ("tag_mouth", vec3(-64., -64., -64.), vec3(64., 64., 72.)),
                ("spine_05", vec3(-40., -40., -64.), vec3(40., 40., 64.)),
                ("spine_03", Vec3::splat(-32.), Vec3::splat(32.)),
                ("spine_01", vec3(-48., -48., -64.), vec3(48., 48., 56.)),
                ("tail_05", vec3(-40., -40., -16.), vec3(40., 40., 32.)),
                ("tail_07", vec3(-24., -24., -16.), Vec3::splat(24.)),
            ]
            .into_iter()
            .enumerate()
            {
                let p = self.tag(d, tag);
                // The engine uses axis-aligned damage boxes at animated tags.
                targets.push(Target {
                    id: BASE + i,
                    center: p.translation + (min + max) * 0.5,
                    half: (max - min) * 0.5,
                });
            }
        }
        targets.extend(
            self.larvae
                .iter()
                .enumerate()
                .filter(|(_, l)| l.vulnerable())
                .map(|(i, l)| l.target(BASE + 100 + i)),
        );
        targets
    }
    pub fn hit(&mut self, h: Hit, d: &data::Data) -> Option<&'static str> {
        if h.id >= BASE + 100 {
            return self
                .larvae
                .get_mut(h.id - BASE - 100)
                .and_then(|l| l.hit(h));
        }
        if h.id != BASE
            || !self.weak(d)
            || !h.damage.is_finite()
            || h.damage < 1.
            || matches!(h.kind.means(), DamageKind::Fire | DamageKind::FireSword)
        {
            return None;
        }
        self.pain_at = self.tag(d, "tag_target").translation;
        self.blood_age = 0.;
        self.hits += 1;
        self.variant = u8::from(self.roll(50));
        self.grabbed = false;
        if let Some(c) = &mut self.coop { c.grab = None; }
        self.set(if self.hits == 6 {
            Action::Death
        } else {
            Action::Pain
        });
        None // The actual pain clip owns its sound, once, on the next combat step.
    }
    pub fn advance(&mut self, c: &mut Combat<'_>, d: &data::Data) -> Feedback {
        let mut out = Feedback::default();
        self.accumulator += f64::from(c.dt.min(0.1));
        while self.accumulator + 1e-9 >= f64::from(STEP) {
            self.accumulator = (self.accumulator - f64::from(STEP)).max(0.);
            self.step(c.world, c.player, c.notarget, None, d, &mut out);
        }
        out
    }
    pub fn release_player(&mut self, player: usize) {
        if self.coop.as_ref().is_some_and(|c| c.grab == Some(player)) {
            self.grabbed = false;
            self.coop.as_mut().unwrap().grab = None;
            self.set(Action::Return);
        }
    }
    pub fn advance_pair(&mut self, dt: f32, world: &World, players: &mut [crate::level::CoopPlayer<'_>; 2], d: &data::Data) -> Feedback {
        let mut out = Feedback::default();
        self.coop.get_or_insert_with(|| Paired { target: None, grab: self.grabbed.then_some(0) });
        self.accumulator += f64::from(dt.min(0.1));
        while self.accumulator + 1e-9 >= f64::from(STEP) {
            self.accumulator = (self.accumulator - f64::from(STEP)).max(0.);
            let targets = players.each_ref().map(|p| combat::PlayerTarget {
                eye: p.player.eye(), alive: p.stats.alive(), hidden: p.stats.ignores_alice(),
            });
            for (i,p) in targets.iter().enumerate() { if !p.alive { self.release_player(i); } }
            let from = self.pose.translation + Vec3::Z * 96.;
            let selected = self.coop.as_ref().unwrap().grab.or_else(|| (0..2).filter(|i| {
                let p = targets[*i];
                let sight = world.sweep(from, p.eye, Vec3::splat(0.5));
                p.alive && !p.hidden && !sight.start_solid && sight.fraction >= 1.
            }).min_by(|a,b| from.distance_squared(targets[*a].eye).total_cmp(&from.distance_squared(targets[*b].eye))));
            self.coop.as_mut().unwrap().target = selected;
            let index = selected.or_else(|| targets.iter().position(|p| p.alive)).unwrap_or(0);
            self.step(world, players[index].player, selected.is_none(), Some(targets), d, &mut out);
            self.coop.as_mut().unwrap().grab = self.grabbed.then_some(index);
            if self.hits < 6 {
                for (i,p) in players.iter_mut().enumerate().filter(|(_,p)| p.stats.alive()) {
                    if self.coop.as_ref().unwrap().grab == Some(i) { continue; }
                    let delta = p.player.feet - self.pose.translation;
                    if delta.truncate().length() < 120. && (-64. ..452.).contains(&delta.z) {
                        let goal = (self.pose.translation + delta.with_z(0.).try_normalize().unwrap_or(Vec3::X) * 120.).with_z(p.player.feet.z);
                        let trace = world.body_trace(p.player.feet, goal);
                        let at = p.player.feet.lerp(goal, trace.fraction);
                        if !trace.start_solid && pair_clear(targets, Some(i), p.player.feet, at) { p.player.feet = at; }
                    }
                }
            }
        }
        out
    }
    fn melee(&self, world: &World, player: &Player, pair: Option<[combat::PlayerTarget; 2]>, damage: f32, out: &mut Feedback) {
        let toward = self.pose.rotation * Vec3::X;
        let center = self.pose.translation + toward * 128. + Vec3::Z * 44.;
        let bodies = bodies(pair, player);
        for body in bodies {
            let delta = body.center - center;
            if delta.x.abs() < 120. && delta.y.abs() < 152. && delta.z.abs() < 100.
                && world.sweep(center, body.center, Vec3::ZERO).fraction >= 0.99 {
                out.strike(body.id, damage, delta.with_z(0.).normalize_or_zero() * 200., DamageKind::Other);
            }
        }
    }
    fn move_toward(&mut self, w: &World, delta: Vec3, speed: f32) {
        let dir = delta.with_z(0.).normalize_or_zero();
        for angle in [0., 0.65, -0.65, 1.1, -1.1] {
            let at = combat::walk_body(
                w,
                self.pose.translation,
                Quat::from_rotation_z(angle) * dir * speed.clamp(0., 600.) * STEP,
                vec3(96., 96., 96.),
            );
            if at.distance_squared(self.pose.translation) > 0.0001 {
                self.pose.translation = at;
                break;
            }
        }
    }
    fn step(&mut self, world: &World, player: &mut Player, notarget: bool, pair: Option<[combat::PlayerTarget; 2]>, d: &data::Data, out: &mut Feedback) {
        let clip = self.clip();
        let old = self.time;
        self.time = (self.time + STEP).min(3600.);
        self.blood_age = (self.blood_age + STEP).min(5.);
        self.quake = (self.quake - STEP).max(0.);
        let rig = d.boss();
        let frame = rig.frame(clip);
        let duration = rig.duration(clip);
        let cross = |f: f32| old <= f * frame && self_time_cross(old, STEP, f * frame);
        for cue in rig.audio.between(
            clip,
            crate::audio::events::Span {
                start: old,
                end: self.time,
                duration,
                frame_time: frame,
                looping: self.action == Action::Walk,
                entered: old == 0.,
            },
        ) {
            if let Some(path) = rig.sounds.get(&cue.path) {
                out.spatial_sounds.push((*path, self.pose.translation));
            }
        }
        for l in &mut self.larvae {
            l.notarget = pair.is_none() && notarget;
            l.opponents.summon = None;
            l.opponents.players = pair;
            l.step(world, player.feet + Vec3::Z * 32., d, out);
        }
        let mut fragments = Vec::new();
        for s in &mut self.shots {
            s.age += STEP;
            if s.ended.is_some() {
                continue;
            }
            if s.fragment {
                s.velocity.z -= 80. * STEP;
            }
            let end = s.at + s.velocity * STEP;
            let half = Vec3::splat(if s.fragment { 8. } else { 16. });
            let wall = world.sweep(s.at, end, half);
            let bodies = bodies(pair, player);
            let hit = bodies.iter().filter_map(|body| {
                combat::segment_box(s.at, end, body.center, vec3(16., 16., 32.) + half)
                    .filter(|f| *f < wall.fraction && !wall.start_solid).map(|f| (body.id, f))
            }).min_by(|a,b| a.1.total_cmp(&b.1));
            let fraction = hit.map(|(_, f)| f);
            if let Some((id, _)) = hit {
                out.strike(id, if s.fragment { 10. } else { 15. }, Vec3::ZERO, DamageKind::Other);
            }
            s.at = s.at.lerp(end, fraction.unwrap_or(wall.fraction));
            if fraction.is_some() || wall.start_solid || wall.fraction < 1. || s.age >= 8. {
                s.ended = Some(s.age);
                let normal = if fraction.is_none() && wall.fraction < 1. {
                    wall.normal
                } else {
                    -s.velocity.normalize_or_zero()
                };
                // Only world impacts leave a surface decal; flesh hits still scatter acid.
                s.normal = if fraction.is_none() && !wall.start_solid && wall.fraction < 1. {
                    normal
                } else {
                    Vec3::ZERO
                };
                out.spatial_sounds.push((
                    if fraction.is_some() {
                        "sound/weapon/shared/splat2.wav"
                    } else {
                        "sound/weapon/shared/splat_small.wav"
                    },
                    s.at,
                ));
                if !s.fragment && !wall.start_solid && s.age < 8. {
                    for n in 0..6 {
                        let angle = (n as f32 / 6.) * std::f32::consts::TAU;
                        let axis = normal.cross(Vec3::Z).try_normalize().unwrap_or(Vec3::X);
                        let up = normal.cross(axis);
                        let dir = (normal * 0.65 + axis * angle.cos() + up * angle.sin())
                            .normalize_or_zero();
                        let start = s.at + normal * 18.;
                        fragments.push(Shot {
                            at: start,
                            start,
                            velocity: dir * 450.,
                            age: 0.,
                            ended: None,
                            fragment: true,
                            normal: Vec3::ZERO,
                        });
                    }
                }
            }
        }
        self.shots
            .retain(|s| s.ended.is_none_or(|t| s.age - t < 1.));
        for s in fragments.into_iter().take(128 - self.shots.len()) {
            self.shots.push(s);
        }
        let delta = player.feet - self.pose.translation;
        let distance = delta.truncate().length();
        if matches!(
            self.action,
            Action::Walk | Action::Ready | Action::Spit | Action::Larvae | Action::Strike
        ) && !notarget
        {
            let yaw = self.pose.rotation.to_euler(EulerRot::ZYX).0;
            let turn = (delta.y.atan2(delta.x) - yaw + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            self.pose.rotation = Quat::from_rotation_z(yaw + turn.clamp(-2. * STEP, 2. * STEP));
        }
        match self.action {
            Action::Walk => {
                if notarget {
                    return;
                }
                if distance > 240. {
                    self.move_toward(world, delta, d.speed("c_centipede", clip));
                }
                if (old / frame / 5.).floor() != (self.time / frame / 5.).floor() {
                    self.melee(world, player, pair, 0.5, out);
                }
                self.decision = (self.decision - STEP).max(0.);
                if self.decision == 0. {
                    self.decision = 1.;
                    let sight = world
                        .sweep(
                            self.pose.translation + Vec3::Z * 96.,
                            player.feet + Vec3::Z * 32.,
                            Vec3::ZERO,
                        )
                        .fraction
                        >= 0.99;
                    if sight {
                        let action = if distance < 400. {
                            if self.stage == 3 {
                                match self.cycle {
                                    0 => Action::Strike,
                                    1 => Action::Ready,
                                    _ => Action::Larvae,
                                }
                            } else if self.stage == 2 && distance >= 300. && !self.roll(75) {
                                Action::Walk
                            } else {
                                Action::Ready
                            }
                        } else if self.stage == 3 && distance > 700. && self.roll(70) {
                            Action::Spit
                        } else if self.stage == 3 || self.stage == 2 && self.roll(20) {
                            Action::Charge
                        } else if self.stage == 1 && self.roll(20) {
                            Action::Ready
                        } else if self.roll(50) {
                            Action::Larvae
                        } else {
                            Action::Spit
                        };
                        if action == Action::Strike {
                            self.grab_camera = player.feet + vec3(80., 280., 200.);
                            self.victim_feet = player.feet;
                        }
                        self.set(action);
                    }
                }
                if self.time >= duration {
                    self.time -= duration;
                }
            }
            Action::Ready => {
                if self.time >= duration {
                    self.set(Action::Crush);
                }
            }
            Action::Crush => {
                if cross(25.) {
                    self.quake = 0.5;
                }
                if cross(30.) {
                    self.melee(world, player, pair, 25., out);
                }
                if self.time >= duration {
                    if self.stage == 3 {
                        self.cycle = 2;
                    }
                    self.set(Action::Walk);
                }
            }
            Action::Spit | Action::Larvae => {
                let larvae = self.action == Action::Larvae;
                for (n, f) in [8., if larvae { 12. } else { 14. }].into_iter().enumerate() {
                    if cross(f) && (pair.is_none() || !notarget) {
                        let start = self.tag(d, "tag_tongue").translation;
                        let aim = (player.feet + Vec3::Z * 32. - start).normalize_or_zero();
                        if larvae {
                            let slot = self
                                .larvae
                                .iter()
                                .position(|l| l.health <= 0. && l.time >= 10.)
                                .unwrap_or(self.larvae.len());
                            if slot < 16 {
                                let mut l = Insect::new(
                                    Kind::Larva,
                                    start,
                                    aim.y.atan2(aim.x),
                                    1.,
                                    self.random as usize + slot,
                                );
                                // A malformed original second launch token is resolved to 250 units/s.
                                l.launch(aim * if n == 0 { 150. } else { 250. } + Vec3::Z * 150.);
                                if slot == self.larvae.len() {
                                    self.larvae.push(l);
                                } else {
                                    self.larvae[slot] = l;
                                }
                            }
                        } else if self.shots.len() < 128 {
                            self.shots.push(Shot {
                                at: start,
                                start,
                                velocity: aim * 550.,
                                age: 0.,
                                ended: None,
                                fragment: false,
                                normal: Vec3::ZERO,
                            });
                        }
                    }
                }
                if self.time >= duration {
                    if larvae && self.stage == 3 {
                        self.cycle = 0;
                    }
                    self.set(Action::Walk);
                }
            }
            Action::Charge => {
                self.move_toward(world, self.pose.rotation * Vec3::X, 400.);
                self.quake = 0.15;
                if distance < 240. || self.time > 3. {
                    self.melee(world, player, pair, if self.stage == 2 { 25. } else { 35. }, out);
                    self.set(Action::Ready);
                }
            }
            Action::Strike => {
                if self.time >= duration {
                    self.grabbed = (pair.is_none() || !notarget) && distance < 400.
                        && world
                            .sweep(
                                self.tag(d, "tag_mandibles").translation,
                                player.feet + Vec3::Z * 32.,
                                Vec3::ZERO,
                            )
                            .fraction
                            >= 0.99;
                    self.grab_feet = player.feet;
                    if self.grabbed {
                        self.set(Action::Shake);
                    } else {
                        self.cycle = 1;
                        self.set(Action::Walk);
                    }
                }
            }
            Action::Shake => {
                if self.grabbed {
                    let mandibles = self.tag(d, "tag_mandibles").translation;
                    // DragEnemy's second tag belongs to its victim, not the Centipede.
                    let gut = d.rigs["alice"]
                        .tag(
                            "tag_gut",
                            "ready",
                            self.time,
                            Transform {
                                translation: Vec3::ZERO,
                                rotation: Quat::IDENTITY,
                            },
                        )
                        .translation;
                    let goal = self
                        .grab_feet
                        .lerp(mandibles - gut, (self.time / 0.15).clamp(0., 1.));
                    let t = world.body_trace(player.feet, goal);
                    let at = player.feet.lerp(goal, t.fraction);
                    let clear = pair.is_none_or(|targets| pair_clear(targets,
                        self.coop.as_ref().and_then(|c| c.grab), player.feet, at));
                    if !t.start_solid && clear {
                        player.feet = at;
                        player.velocity = Vec3::ZERO;
                        player.grounded = false;
                    }
                }
                if cross(17.) && self.grabbed {
                    self.grabbed = false;
                    player.velocity = self.pose.rotation * vec3(500., 50., 250.);
                }
                if self.time >= duration {
                    self.grabbed = false;
                    self.set(Action::Return);
                }
            }
            Action::Return => {
                if self.time >= duration {
                    self.cycle = 1;
                    self.set(Action::Walk);
                }
            }
            Action::Pain => {
                if self.time >= duration {
                    self.stage = (self.hits / 2 + 1).min(3);
                    self.cycle = 0;
                    self.set(Action::Walk);
                }
            }
            Action::Death => {
                if cross(18.) {
                    self.quake = 0.5;
                }
                if self.time >= duration {
                    self.set(Action::Thrash);
                }
            }
            Action::Thrash => {
                if self.time >= duration {
                    self.thrashes += 1;
                    self.set(if self.thrashes == 3 {
                        Action::Dead
                    } else {
                        Action::Thrash
                    });
                }
            }
            Action::Dead => {}
        }
        self.victim_feet = player.feet;
        // Player clipping is independent of damage boxes, so it cannot mask a weak-point trace.
        if pair.is_none() && self.hits < 6
            && !self.grabbed
            && distance < 120.
            && (-64. ..452.).contains(&(player.feet.z - self.pose.translation.z))
        {
            let goal =
                self.pose.translation + delta.with_z(0.).try_normalize().unwrap_or(Vec3::X) * 120.;
            let goal = goal.with_z(player.feet.z);
            let t = world.body_trace(player.feet, goal);
            if !t.start_solid {
                player.feet = player.feet.lerp(goal, t.fraction);
            }
        }
    }
}
fn self_time_cross(old: f32, dt: f32, t: f32) -> bool {
    old <= t && old + dt > t
}

fn bodies(pair: Option<[combat::PlayerTarget; 2]>, player: &Player) -> Vec<Target> {
    if pair.is_some() { combat::Opponents { players: pair, ..Default::default() }.bodies(player.eye()) }
    else { vec![Target { id: crate::dice::ALICE, center: player.feet + Vec3::Z*32., half: crate::collision::PLAYER_HALF }] }
}
fn pair_clear(targets: [combat::PlayerTarget; 2], actor: Option<usize>, from: Vec3, to: Vec3) -> bool {
    targets.iter().enumerate().all(|(i,p)| actor == Some(i) || !p.alive
        || combat::segment_box(from, to, p.eye - Vec3::Z*crate::movement::EYE_HEIGHT,
            vec3(crate::coop::separation::DISTANCE, crate::coop::separation::DISTANCE, crate::collision::PLAYER_HALF.z*2.)).is_none())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_boss_drag_cannot_cross_or_end_inside_the_living_partner() {
        for actor in 0..2 {
            let mut targets=[combat::PlayerTarget{eye:Vec3::Z*crate::movement::EYE_HEIGHT,alive:true,hidden:false};2];
            targets[actor].eye+=Vec3::X*100.;
            assert!(!pair_clear(targets,Some(actor),Vec3::X*100.,-Vec3::X*100.));
            assert!(!pair_clear(targets,Some(actor),Vec3::X*100.,Vec3::ZERO));
            assert!(pair_clear(targets,Some(actor),Vec3::X*100.,Vec3::X*40.));
            targets[1-actor].alive=false;
            assert!(pair_clear(targets,Some(actor),Vec3::X*100.,-Vec3::X*100.));
        }
    }
}
