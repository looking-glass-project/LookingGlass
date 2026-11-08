//! Two visits, one map: independent scene, collectible and passage ownership.
mod art;
mod check;
mod coop;
mod coop_check;
mod coop_route;
pub(crate) use check::drive as drive_route;
mod data;
mod motion;
mod scene;
use super::{state, Check, Registration, Run};
use crate::{
    assets::Assets,
    bsp::Bsp,
    collision::{Collider, World, PLAYER_CENTER, PLAYER_HALF},
    combat::{DamageKind, Hit, Target},
    event::{Condition, Facts},
    interaction::Events,
    inventory::{PickupKind, Stats},
    level::exit::ExitState,
    level::spec::ExitSpec,
    level::{LevelArt, LevelController, TriggerInfo, Upgrade},
    movement::Player,
    skeletal::Transform,
    story::{BeatSpec, Story},
};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;
use scene::{Kind, Scene};
use serde::{Deserialize, Serialize};
use std::any::Any;
const FIRST_BASE: usize = 7_800_000;
const RETURN_BASE: usize = 8_700_000;
const CHESS: ExitSpec = ExitSpec {
    map: "wchess1",
    entrance: "wchess1_start1",
};
const HEDGE: ExitSpec = ExitSpec {
    map: "hedge1",
    entrance: "hedge1_start1",
};
#[derive(Clone, Serialize, Deserialize)]
struct Saved {
    version: u8,
    returning: bool,
    initialized: bool,
    clock: f32,
    scene: Option<Scene>,
    done: [bool; 5],
    staff: bool,
    armed: bool,
    cave: f32,
    chess: f32,
    secret: Option<f32>,
    wall_health: f32,
    wall_visible: bool,
    wall_cooldown: f32,
    destruction: Option<f32>,
    blunder: Option<f32>,
    blunder_done: bool,
    essence: u8,
    essence_delay: f32,
    exit: ExitState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    coop: Option<coop::State>,
}
impl Saved {
    fn new(returning: bool) -> Self {
        Self {
            version: 1,
            returning,
            initialized: false,
            clock: 0.,
            scene: None,
            done: [false; 5],
            staff: false,
            armed: false,
            cave: 0.,
            chess: 0.,
            secret: None,
            wall_health: 100.,
            wall_visible: false,
            wall_cooldown: 0.,
            destruction: None,
            blunder: None,
            blunder_done: false,
            essence: 0,
            essence_delay: 0.,
            exit: ExitState::default(),
            coop: None,
        }
    }
}
impl state::State for Saved {
    const VERSION: u8 = 1;
    fn version(&self) -> u8 {
        self.version
    }
    fn validate(&self, v: state::Visit) -> Result<()> {
        ensure!(v.returning == self.returning, "WForest visit mismatch");
        if let Some(c) = &self.coop { c.validate(self.returning)?; }
        state::clock("world", self.clock, 1e6)?;
        state::clock("cave gate", self.cave, 4.)?;
        state::clock("chess gate", self.chess, 4.2)?;
        state::clock("wall health", self.wall_health, 100.)?;
        state::clock("wall visibility debounce", self.wall_cooldown, 0.1)?;
        state::clock("essence delay", self.essence_delay, 10.)?;
        state::clock("exit retry", self.exit.retry_time, 1.)?;
        if let Some(t) = self.secret {
            state::clock("secret door", t, 6.)?;
        }
        if let Some(t) = self.destruction {
            state::clock("wall destruction", t, 5.5)?;
        }
        if let Some(t) = self.blunder {
            state::clock("secret Cat", t, 3600.)?;
        }
        ensure!(
            self.essence < 2 && (!self.blunder_done || self.blunder.is_some()),
            "Invalid secret state"
        );
        ensure!(
            (self.wall_health == 0.) == self.destruction.is_some(),
            "Wall health disagrees with destruction"
        );
        ensure!(
            self.exit.retry_time == 0. || self.exit.committed,
            "Uncommitted exit retry"
        );
        if self.returning {
            ensure!(
                !self.staff
                    && !self.armed
                    && self.cave == 0.
                    && self.chess == 0.
                    && self.done[..4].iter().all(|v| !v),
                "First visit state in return visit"
            );
        } else {
            ensure!(
                self.secret.is_none()
                    && self.destruction.is_none()
                    && self.wall_health == 100.
                    && self.blunder.is_none()
                    && !self.done[4]
                    && self.essence_delay == 0.,
                "Return state in first visit"
            );
            ensure!(
                (self.cave == 0. || self.staff)
                    && (!self.done[1] || self.staff)
                    && (!self.done[2] || self.armed)
                    && (self.chess == 0. || self.armed)
                    && (!self.done[3] || self.done[2]),
                "Out-of-order first visit progression"
            );
        }
        if let Some(s) = &self.scene {
            s.validate()?;
            ensure!(
                s.kind.returning() == self.returning && !self.done[s.kind.index()],
                "Invalid active visit scene"
            );
            ensure!(
                match s.kind {
                    Kind::Staff => self.staff,
                    Kind::Caterpillar => self.armed,
                    Kind::Chess => self.staff && self.done[2] && self.chess == 4.2,
                    Kind::Wall => self.wall_health > 0.,
                    Kind::Arrival => true,
                },
                "Scene precedes its prerequisite"
            );
        }
        ensure!(
            !self.exit.committed
                || if self.returning {
                    self.destruction == Some(5.5)
                } else {
                    self.staff && self.done[1] && self.done[2] && self.chess == 4.2
                },
            "Wrong or premature exit"
        );
        Ok(())
    }
}
struct Forest {
    saved: Saved,
    data: data::Data,
    motion: motion::Motion,
}
impl Forest {
    fn load(a: &mut Assets, map: &Bsp, returning: bool) -> Result<Self> {
        let mut f = Self {
            saved: Saved::new(returning),
            data: data::Data::load(a, map)?,
            motion: motion::Motion::load(map)?,
        };
        f.motion.rebuild(map, &f.saved)?;
        Ok(f)
    }
    fn fact(&self, suffix: &str) -> String {
        format!("{}.{suffix}", self.id())
    }
    fn first_ready(&self) -> bool {
        !self.saved.returning
            && self.saved.staff
            && self.saved.done[1]
            && self.saved.done[2]
            && self.saved.chess == 4.2
            && !self.scripted()
    }
    fn hedge_ready(&self) -> bool {
        self.saved.returning && self.saved.destruction == Some(5.5) && !self.scripted()
    }
    fn altar(&self) -> bool {
        !self.saved.returning && !self.saved.staff
    }
    fn begin(&mut self, kind: Kind) {
        if self.saved.scene.is_none()
            && !self.saved.done[kind.index()]
            && self.saved.returning == kind.returning()
        {
            self.saved.scene = Some(Scene::new(kind));
        }
    }
    fn exit_spec(&self) -> ExitSpec {
        if self.saved.returning {
            HEDGE
        } else {
            CHESS
        }
    }
}
impl LevelController for Forest {
    fn id(&self) -> &'static str {
        if self.saved.returning {
            "wforest-return"
        } else {
            "wforest"
        }
    }
    fn facts(&self) -> Facts {
        let mut f = Facts::default();
        for (name, value) in [
            ("first", !self.saved.returning),
            ("return", self.saved.returning),
            ("altar", self.altar()),
            (
                "caterpillar",
                !self.saved.returning
                    && self.saved.armed
                    && !self.saved.done[2]
                    && !self.scripted(),
            ),
            ("chess-talk", self.first_ready() && !self.saved.done[3]),
            ("chess-exit", self.first_ready()),
            ("hedge-exit", self.hedge_ready()),
            (
                "secret",
                self.saved.returning && self.saved.secret.is_none(),
            ),
            (
                "wall-warning",
                self.saved.returning
                    && self.saved.wall_health > 0.
                    && !self.saved.done[4]
                    && !self.scripted(),
            ),
            (
                "wall-show",
                self.saved.wall_health > 0.
                    && !self.saved.wall_visible
                    && self.saved.wall_cooldown == 0.,
            ),
            (
                "wall-hide",
                self.saved.wall_health > 0.
                    && self.saved.wall_visible
                    && self.saved.wall_cooldown == 0.,
            ),
            ("hint", !self.saved.returning && self.saved.done[2]),
        ] {
            f.flag(&self.fact(name), value);
        }
        f
    }
    fn gate(&self, t: &TriggerInfo<'_>) -> Option<Condition> {
        let key = match t.id.0 {
            25 => "wall-warning",
            63 => "chess-talk",
            67 => "wall-show",
            70 => "secret",
            81 => "altar",
            83 => "hedge-exit",
            110 => "hint",
            119 | 716 => "caterpillar",
            126 => "chess-exit",
            620 => "wall-hide",
            _ => return None,
        };
        Some(Condition::flag(&self.fact(key)))
    }
    fn event(&mut self, n: &str) -> Option<Events> {
        match n {
            "Give_Eyestaff" if !self.saved.returning => self.saved.armed = true,
            "Alice_Gets_Eyestaff" if !self.saved.returning && self.saved.staff => {
                self.begin(Kind::Staff)
            }
            "WForest_Cinema1" if !self.saved.returning && self.saved.armed => {
                self.begin(Kind::Caterpillar)
            }
            "Cat_ChessTalk" if self.first_ready() => self.begin(Kind::Chess),
            "Alice_Destroy_Wall" if self.saved.returning && self.saved.wall_health > 0. => {
                self.begin(Kind::Wall)
            }
            "Open_Humpty_Door" if self.saved.returning => {
                self.saved.secret.get_or_insert(0.);
            }
            "Wall_Show" if self.saved.wall_health > 0. => {
                self.saved.wall_visible = true;
                self.saved.wall_cooldown = 0.1;
            }
            "Wall_Hide" if self.saved.wall_health > 0. => {
                self.saved.wall_visible = false;
                self.saved.wall_cooldown = 0.1;
            }
            "Hedge_Maze_Entrance" if self.hedge_ready() => {
                self.saved.exit.committed = true;
            }
            "Cat_WallTalk" => {}
            // Gated events are consumed safely even if an old trigger attempts a stale delivery.
            "Give_Eyestaff"
            | "Alice_Gets_Eyestaff"
            | "WForest_Cinema1"
            | "Cat_ChessTalk"
            | "Alice_Destroy_Wall"
            | "Open_Humpty_Door"
            | "Hedge_Maze_Entrance"
            | "Wall_Show"
            | "Wall_Hide" => {}
            _ => return None,
        }
        let spec = self.exit_spec();
        Some(Events {
            transition: self.saved.exit.request(spec),
            ..Default::default()
        })
    }
    fn prepare_player(&mut self, stats: &mut Stats, p: &mut Player) {
        if !self.saved.initialized {
            stats.full_stats();
            self.saved.initialized = true;
        }
        if self.saved.returning && stats.copies(7) == 0 {
            stats.apply(PickupKind::Weapon(7), 100.);
        }
        if self.saved.staff {
            stats.staff_component = true;
        }
        if self.altar() && !self.scripted() {
            let d = (p.feet + PLAYER_CENTER
                - (self.data.at("first_eyestaff").translation + Vec3::Z * 37.5))
                .abs();
            if d.cmple(PLAYER_HALF + vec3(10., 10., 37.5)).all() {
                self.saved.staff = true;
                self.saved.armed = true;
                stats.staff_component = true;
                self.begin(Kind::Staff);
            }
        }
        if self.saved.returning
            && self.saved.blunder.is_none()
            && stats.collected.contains("wforest:73")
        {
            self.saved.blunder = Some(0.);
        }
        if self.saved.returning && self.saved.essence_delay == 0. && !self.scripted() {
            let at = self
                .data
                .at(if self.saved.essence == 0 {
                    "get_me1"
                } else {
                    "get_me2"
                })
                .translation;
            if (p.feet + PLAYER_CENTER).distance(at) < 42.
                && (stats.sanity() < 100. || stats.will() < 100.)
            {
                stats.essence(25.);
                self.saved.essence ^= 1;
                self.saved.essence_delay = 10.;
            }
        }
        if self.saved.destruction.is_some_and(|t| t < 1.5) {
            stats.scripted_immunity = stats.scripted_immunity.max(0.1);
        }
    }
    fn update(&mut self, _: &mut World, _: &Player, _: Vec3, _: bool) -> Events {
        let spec = self.exit_spec();
        Events {
            transition: self.saved.exit.request(spec),
            ..Default::default()
        }
    }
    fn coop_prepare(&mut self, players: &mut [crate::level::CoopPlayer<'_>; 2]) {
        self.prepare_pair(players);
    }
    fn coop_advance_roster(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [crate::level::CoopPlayer<'_>; 2], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. || self.scripted() || !players.iter().any(|p| p.stats.alive()) { return Ok(()); }
        self.prepare_pair(players);
        let dt=dt.min(0.1);
        self.advance_clocks(dt);
        let mut living=players.iter_mut().filter(|p|p.stats.alive()).map(|p|&mut *p.player).collect::<Vec<_>>();
        self.motion.advance_players(dt,map,world,&mut living,fixed,&mut self.saved)
    }
    fn coop_contacts(&mut self, dt: f32, players: &mut [crate::level::CoopPlayer<'_>; 2]) -> Option<usize> {
        if !dt.is_finite() || dt<=0. { return None; }
        self.contacts_pair(players)
    }
    fn coop_respawned(&mut self, player: usize) {
        if let Some(c)=&mut self.saved.coop { c.previous[player]=None; }
    }
    fn coop_scene_step(&mut self, _: usize, _: &mut Player, _: &mut Stats) {
        if let Some(c)=&mut self.saved.coop { c.previous=[None;2]; }
    }
    fn coop_reunion_limits(&self, player: &Player) -> Option<(f32,f32)> {
        (!self.saved.returning && self.saved.done[1]
            && player.feet.distance(self.data.at("alice_eyestaff_pos1").translation)<1.)
            .then_some((256.,256.))
    }
    fn transition_failed(&mut self, e: &(String, Option<String>)) {
        if self.exit_spec().matches(e) {
            self.saved.exit.failed();
        }
    }
    fn advance(
        &mut self,
        dt: f32,
        map: &Bsp,
        w: &mut World,
        p: &mut Player,
        fixed: &[Collider],
    ) -> Result<()> {
        if dt <= 0. || !dt.is_finite() {
            return Ok(());
        }
        let dt = dt.min(0.1);
        self.advance_clocks(dt);
        self.motion.advance(dt, map, w, p, fixed, &mut self.saved)?;
        self.advance_scene(dt, w, p)?;
        Ok(())
    }
    fn transforms(&self) -> Vec<(usize, Vec3, Quat)> {
        self.motion.transforms(&self.saved)
    }
    fn colliders(&self) -> Vec<Collider> {
        let mut out = self.motion.colliders(&self.saved);
        if self.saved.returning {
            let at = self.data.at("humpty_actor1").translation;
            out.push(Collider::box_bounds(
                at + vec3(-48., -48., 64.),
                at + vec3(48., 48., 192.),
            ));
        }
        out
    }
    fn targets(&self) -> Vec<Target> {
        if !self.saved.returning || self.saved.wall_health <= 0. {
            return vec![];
        }
        vec![Target {
            id: RETURN_BASE,
            center: self.data.wall_center,
            half: self.data.wall_half + Vec3::ONE,
        }]
    }
    fn hit(&mut self, h: Hit) -> Option<&'static str> {
        if h.id == RETURN_BASE
            && self.saved.returning
            && self.saved.wall_health > 0.
            && h.kind == DamageKind::EyeStaff
            && h.damage.is_finite()
            && h.damage > 0.
        {
            self.saved.wall_health = (self.saved.wall_health - h.damage).max(0.);
            if self.saved.wall_health == 0. {
                self.saved.destruction = Some(0.);
                return Some("sound/weapon/blunderbuss/bb_explode.wav");
            }
        }
        None
    }
    fn scripted(&self) -> bool {
        self.saved.scene.is_some()
    }
    fn controlled(&self) -> bool {
        self.scripted()
    }
    fn allow_cheshire(&self) -> bool {
        !self.scripted()
    }
    fn scene_id(&self) -> Option<&'static str> {
        self.saved.scene.as_ref().map(|s| s.kind.id())
    }
    fn entry_story(&mut self, _: &mut Story) -> bool {
        if !self.saved.returning {
            self.begin(Kind::Arrival);
        }
        true
    }
    fn prepare_story(&self, story: &mut Story) -> bool {
        self.prepare_scene_story(story)
    }
    fn sync_story(&mut self, story: &Story) {
        self.sync_scene_story(story);
    }
    fn dialogue_complete(&mut self, n: &str) -> Events {
        self.complete_scene_dialogue(n);
        Events::default()
    }
    fn skip(&mut self, _: &Bsp, w: &mut World, p: &mut Player, story: &mut Story) -> Result<bool> {
        let Some(s) = &self.saved.scene else {
            return Ok(false);
        };
        let id = s.kind.dialogue();
        self.finish_scene(w, p, true)?;
        story.finish_sequence(id);
        Ok(true)
    }
    fn camera(&self, _: &World) -> Option<crate::cinematic::Camera> {
        self.scene_camera()
    }
    fn fade(&self) -> Option<(Color, f32)> {
        self.saved.scene.as_ref().map(|s| {
            (
                if s.kind == Kind::Arrival {
                    BLACK
                } else {
                    WHITE
                },
                (1. - s.time / 0.5).clamp(0., 1.),
            )
        })
    }
    fn particles(&self, p: &mut crate::particles::Steam) {
        p.gate(&[
            (crate::entity::Id(96), self.altar()),
            (crate::entity::Id(73), self.saved.returning),
            (crate::entity::Id(2), false),
            (crate::entity::Id(734), false),
        ]);
    }
    fn objective(&self) -> Option<String> {
        Some(if self.saved.returning { if self.hedge_ready() { "Continue through the opened wall into the hedge maze." }
            else { "Use the Eye Staff to break the wall leading to the hedge maze. Humpty's secret is optional." } }
            else if !self.saved.staff { "Reach the Staff altar above the pit." }
            else if !self.saved.done[2] { "Follow the opened cavern to the Caterpillar's clearing." }
            else { "Follow the chess path through the raised gate." }.into())
    }
    fn sound_state(
        &self,
        loops: &mut Vec<crate::audio::LoopCue>,
        clocks: &mut Vec<crate::audio::world::Clock>,
    ) {
        if self.altar() {
            loops.push(crate::audio::LoopCue {
                clock: Some(self.saved.clock),
                id: FIRST_BASE + 96,
                path: "sound/weapon/altar.wav",
                origin: self.data.at("first_eyestaff").translation,
            });
        }
        if self.saved.cave > 0. && self.saved.cave < 4. {
            loops.push(crate::audio::LoopCue {
                clock: Some(self.saved.cave),
                id: FIRST_BASE + 26,
                path: "sound/ambience/special/portcullis_loop.wav",
                origin: vec3(2776., 1301., 0.),
            });
        }
        clocks.push(crate::audio::world::Clock {
            key: "wforest.cave",
            time: self.saved.cave,
            period: None,
            origin: vec3(2776., 1301., 0.),
            cues: &[(4., "sound/ambience/special/door_slam2.wav")],
        });
        clocks.push(crate::audio::world::Clock {
            key: "wforest.chess",
            time: self.saved.chess,
            period: None,
            origin: vec3(1591., 4352., 704.),
            cues: &[(0., "sound/world/door/chess_gate1.wav")],
        });
        if let Some(t) = self.saved.secret {
            clocks.push(crate::audio::world::Clock {
                key: "wforest-return.secret",
                time: t,
                period: None,
                origin: vec3(3650., 1094., 424.),
                cues: &[
                    (0., "sound/ambience/special/slow scrape.wav"),
                    (2., "sound/ambience/special/stone grind.wav"),
                ],
            });
        }
        if let Some(s) = &self.saved.scene {
            if matches!(s.kind, Kind::Arrival | Kind::Staff | Kind::Chess) {
                clocks.push(crate::audio::world::Clock {
                    key: match s.kind {
                        Kind::Arrival => "wforest.cat.arrival",
                        Kind::Staff => "wforest.cat.staff",
                        _ => "wforest.cat.chess",
                    },
                    time: s.time,
                    period: None,
                    origin: self.cat().map_or(Vec3::ZERO, |c| c.0.translation),
                    cues: &[(0.5, "sound/character/cheshire_cat/appear.wav")],
                });
            }
            if s.kind == Kind::Staff {
                if let Some(t) = s.ending {
                    clocks.push(crate::audio::world::Clock {
                        key: "wforest.cat.staff-out",
                        time: t,
                        period: None,
                        origin: self.data.at("cat_eyestaff_pos1").translation,
                        cues: &[(0., "sound/character/cheshire_cat/disappear.wav")],
                    });
                }
            }
        }
    }
    fn snapshot(&self) -> serde_json::Value {
        state::save(&self.saved)
    }
    fn restore(&mut self, v: &serde_json::Value, map: &Bsp) -> Result<()> {
        let restored = state::load(
            v,
            state::Visit {
                returning: self.saved.returning,
            },
        )?;
        self.motion.rebuild(map, &restored)?;
        self.saved = restored;
        Ok(())
    }
    fn upgraded(&mut self) {
        self.saved.initialized = true;
    }
    fn upgrade(&self) -> Upgrade {
        Upgrade {
            rearm: [
                "Give_Eyestaff",
                "WForest_Cinema1",
                "Cat_ChessTalk",
                "Alice_Destroy_Wall",
                "Open_Humpty_Door",
                "Hedge_Maze_Entrance",
            ]
            .iter()
            .map(|s| (*s).into())
            .collect(),
            ..Default::default()
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
fn owns_npc(n: &str, _: &str) -> bool {
    matches!(
        n,
        "cat_actor1"
            | "cat_actor2"
            | "cat_actor3"
            | "blunder_cat"
            | "caterpillar_actor1"
            | "humpty_actor1"
    )
}
const BEATS: &[BeatSpec] = &[
    BeatSpec::linear(
        "wforest",
        "Cat1_Dialog",
        "wforest_cinematics",
        "Cat1_Dialog",
        1,
    ),
    BeatSpec::linear(
        "wforest",
        "Alice_Gets_Eyestaff",
        "wforest_cinematics",
        "Alice_Gets_Eyestaff",
        1,
    ),
    BeatSpec::linear(
        "wforest",
        "WForest_Cinema1",
        "wforest_cinematics",
        "WForest_Cinema1",
        5,
    ),
    BeatSpec::linear(
        "wforest",
        "Cat_ChessTalk",
        "wforest_cinematics",
        "Cat_ChessTalk",
        3,
    ),
];
pub static REGISTRATION: Registration = Registration {
    id: "wforest",
    applies: |m, e| super::first_visit(m, e, "wforest"),
    load: |a, m, _, _| Ok(Box::new(Forest::load(a, m, false)?)),
    art: Some(|a, _, _| Ok(Box::new(art::Art::load(a)?))),
    owns_submodel: |m, e| m == "wforest" && motion::owned(e),
    owns_npc,
    target_base: Some(FIRST_BASE),
    story_beats: BEATS,
    checks: &[
        Check {
            flag: "--wforest-coop-route-check",
            help: "Walk paired Caterpillar's Plot through the original chess departure.",
            run: Run::Windowed(coop_route::first),
        },
        Check {
            flag: "--wforest-coop-check",
            help: "Verify both WForest visits with paired movers, personal pickups and scene owners.",
            run: Run::Headless(coop_check::check),
        },
        Check {
            flag: "--wforest-check",
            help: "Verify first-visit scenes, altar and gates.",
            run: Run::Headless(check::first),
        },
        Check {
            flag: "--wforest-render-check",
            help: "Capture both WForest visits.",
            run: Run::Windowed(check::render),
        },
        Check {
            flag: "--wforest-route-check",
            help: "Replay the first WForest route.",
            run: Run::Headless(check::route_first),
        },
    ],
    save_cases: &[super::SaveCase {
        name: "wforest-staff-gate",
        visit: "wforest$first",
        stage: Some(check::stage_first),
        behavior: Some(|i, _, _| {
            let f = i
                .levels
                .iter_mut()
                .find_map(|s| s.ctl.downcast_mut::<Forest>())
                .context("Missing saved WForest")?;
            ensure!(
                f.saved.staff && f.saved.cave > 0.,
                "Lost Staff/gate progress"
            );
            Ok(())
        }),
    }],
    visibility: &[],
};
pub static RETURN_REGISTRATION: Registration = Registration {
    id: "wforest-return",
    applies: |m, e| m == "wforest" && super::returning(m, e),
    load: |a, m, _, _| Ok(Box::new(Forest::load(a, m, true)?)),
    art: REGISTRATION.art,
    owns_submodel: REGISTRATION.owns_submodel,
    owns_npc,
    target_base: Some(RETURN_BASE),
    story_beats: &[
        BeatSpec::linear(
            "wforest",
            "Alice_Destroy_Wall",
            "wforest_cinematics",
            "Alice_Destroy_Wall",
            1,
        ),
        BeatSpec::linear("wforest", "blunder_cat", "../wforest", "blunder_cat", 1),
    ],
    checks: &[
        Check {
            flag: "--wforest-return-check",
            help: "Verify return wall, secret and route gates.",
            run: Run::Headless(check::returning),
        },
        Check {
            flag: "--wforest-return-coop-route-check",
            help: "Walk paired return forest through the Staff wall and original maze departure.",
            run: Run::Windowed(coop_route::returning),
        },
        Check {
            flag: "--wforest-return-route-check",
            help: "Replay the return WForest route.",
            run: Run::Headless(check::route_return),
        },
        Check {
            flag: "--wforest-return-render-check",
            help: "Capture both WForest visits, including the return wall and Humpty.",
            run: Run::Windowed(check::render),
        },
    ],
    save_cases: &[
        super::SaveCase {
            name: "wforest-return-secret",
            visit: "wforest$return",
            stage: Some(check::stage_secret),
            behavior: Some(|i, _, _| {
                let f = i
                    .levels
                    .iter_mut()
                    .find_map(|s| s.ctl.downcast_mut::<Forest>())
                    .context("Missing saved return")?;
                ensure!(
                    f.saved.secret.is_some() && f.saved.wall_health == 60. && !f.first_ready(),
                    "Lost return secret/wall state"
                );
                Ok(())
            }),
        },
        super::SaveCase {
            name: "wforest-return-debris",
            visit: "wforest$return",
            stage: Some(check::stage_debris),
            behavior: Some(|i, _, _| {
                let f = i
                    .levels
                    .iter_mut()
                    .find_map(|s| s.ctl.downcast_mut::<Forest>())
                    .context("Missing saved return")?;
                ensure!(
                    f.saved.wall_health == 0. && f.saved.destruction.is_some(),
                    "Destroyed wall resurrected"
                );
                Ok(())
            }),
        },
    ],
    visibility: &[],
};
