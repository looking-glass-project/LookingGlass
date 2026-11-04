//! Input-driven local door crossings with staged prerequisites, not a full route.
use super::*;
use crate::{coop::{protocol::{Input, Visit}, stage::Stage, world::Roster, PlayerId},
    interaction::coop::Contacts, level::CoopPlayer, movement::{Controls, FIXED_DT}};

struct Course {
    world: World,
    stage: Stage,
    roster: Roster,
    laboratory: bool,
    closing_x: f32,
    expected_owner: PlayerId,
    closure: Option<f32>,
    lanes: [f32; 2],
    health: [f32; 2],
    anchors: [Vec2; 2],
}
fn ctl(stage: &mut Stage) -> &mut Clockwork {
    stage.interactions.levels.iter_mut().find_map(|s| s.ctl.downcast_mut()).unwrap()
}
impl Course {
    fn new(a: &mut Assets, map: &Bsp, laboratory: bool, first: PlayerId, lever_partner: bool) -> Result<Self> {
        let visit = Visit { epoch: 1, map: "hatter1".into(), entry: Some("hatter1_start1".into()),
            difficulty: Default::default(), combat_preview: false };
        let mut world = World::from_bsp(map)?;
        let mut stage = Stage::load(a, map, &mut world, &visit, true)?;
        let o = ctl(&mut stage);
        o.saved.initialized = true; o.saved.age = 100.;
        o.saved.levers[0] = Some(1.); o.saved.levers[1] = Some(2.);
        if laboratory { o.saved.levers[2] = Some(5.); }
        o.rebuild(map)?; stage.interactions.sync_coop(&mut world);
        let (eye, yaw) = interaction::spawn(map, visit.entry.as_deref());
        let mut roster = Roster::new(&world, eye, yaw, Stats::for_level("hatter1", visit.entry.as_deref()))?;
        // Begin the gear-door case at its static open doorway. Traversing and
        // waiting on the rotating great gear is a separate machinery course.
        let x = if laboratory { -5296. } else { -4664. };
        let z = if laboratory { 320. } else { 208. };
        for (i, body) in roster.players.iter_mut().enumerate() {
            let y = 3008. + if i == first.index() { -24. } else { 24. };
            let desired = if lever_partner && i != first.index() { vec3(-5512., 2497., 320.) } else { vec3(x, y, z) };
            let feet = world.actor_footing(desired + Vec3::Z * 16., PLAYER_CENTER, PLAYER_HALF, 64.)
                .context("Door course start lacks footing")?;
            body.player = Player::new(feet); body.player.grounded = true;
            body.stats.set_health(if i == 0 { 54. } else { 67. })?;
        }
        let thread = if laboratory { "Close_ClockRoom_Door" } else { "NORETURN" };
        let entity = map.entities.iter().find(|e| e.get("thread").is_some_and(|s| s == thread)).context("Missing closing contact")?;
        let model: usize = entity["model"].trim_start_matches('*').parse()?;
        let closing_x = at(entity).translation.x + map.models[model].max.x;
        let anchors = roster.players.each_ref().map(|b| b.player.feet.truncate());
        let course = Self { world, stage, roster, laboratory, closing_x, expected_owner: first, closure: None, anchors,
            lanes: std::array::from_fn(|i| if i == first.index() { -24. } else { 24. }), health: [54., 67.] };
        course.safe()?;
        Ok(course)
    }
    fn closed(&mut self) -> Option<f32> {
        let o = ctl(&mut self.stage);
        if self.laboratory { o.saved.clockroom_closed } else { o.saved.no_return }
    }
    fn safe(&self) -> Result<()> {
        for (body, health) in self.roster.players.iter().zip(self.health) {
            ensure!(body.stats.sanity() == health, "Door course unexpectedly changed personal health: {} -> {} at {:?}",
                health, body.stats.sanity(), body.player.feet);
            if body.stats.alive() {
                ensure!(self.world.body_clear(body.player.feet), "Door course embedded a living player: {:?}", body.player.feet);
            }
        }
        if self.roster.players.iter().all(|p| p.stats.alive()) {
            let delta = self.roster.players[0].player.feet - self.roster.players[1].player.feet;
            ensure!(delta.truncate().length() >= 35.95 || delta.z.abs() >= 55.95, "Door course overlapped players");
        }
        Ok(())
    }
    fn step(&mut self, map: &Bsp, goals: [Option<Vec2>; 2], must_wait: bool) -> Result<()> {
        let before = self.roster.players.each_ref().map(|b| b.player.feet);
        let inputs = std::array::from_fn(|i| (Controls {
            wish: ((goals[i].unwrap_or(self.anchors[i]) - before[i].truncate()) / 24.).clamp_length_max(1.),
            run: true,
            ..Default::default()
        }, Input::default()));
        self.stage.before_move(&mut self.roster, map, &mut self.world, &inputs, FIXED_DT)?;
        self.roster.move_players(&self.world, inputs);
        self.stage.contacts(&mut self.roster, &self.world, before)?;
        self.stage.speech_step(&mut self.roster, &self.world, false);
        ensure!(!self.stage.busy(), "Door course unexpectedly entered a cinematic");
        self.safe()?;
        if let Some(when) = self.closed() {
            ensure!(!must_wait, "Door closed while the partner was still outside: {:?}", self.roster.players.each_ref().map(|b| b.player.feet));
            ensure!(self.roster.players.iter().filter(|p| p.stats.alive())
                .all(|p| p.player.feet.x + PLAYER_HALF.x <= self.closing_x + 0.001),
                "Closing contact consumed before every living body passed its approach");
            if let Some(old) = self.closure { ensure!(old == when, "One-shot door closure replayed"); }
            else {
                ensure!(self.stage.speaker() == self.expected_owner, "Closing contact lost its first living requester");
                self.closure = Some(when);
            }
        }
        Ok(())
    }
    fn walk(&mut self, map: &Bsp, id: PlayerId, must_wait: bool) -> Result<()> {
        let lane = self.lanes[id.index()];
        let points = if self.laboratory {
            vec![vec2(-5416., 3000. + lane), vec2(-5536., 2912. + lane), vec2(-5760., 2912. + lane)]
        } else { vec![vec2(-4776., 3008. + lane)] };
        for point in points {
            for _ in 0..600 {
                if self.roster.players[id.index()].player.feet.truncate().distance(point) < 5. { break; }
                let mut goals = [None; 2]; goals[id.index()] = Some(point);
                self.step(map, goals, must_wait)?;
            }
            ensure!(self.roster.players[id.index()].player.feet.truncate().distance(point) < 5.,
                "Normal door crossing stalled before {point:?}, at {:?}", self.roster.players[id.index()].player.feet);
            self.anchors[id.index()] = point;
        }
        Ok(())
    }
    fn checkpoint(&mut self, map: &Bsp) -> Result<()> {
        let before = serde_json::to_value((self.stage.snapshot(), self.roster.frame(1, 0)))?;
        let o = ctl(&mut self.stage);
        let mut people = self.roster.players.each_mut().map(|b| CoopPlayer { player: &mut b.player, stats: &mut b.stats });
        for dt in [0., -1., f32::NAN] { o.coop_advance_roster(dt, map, &mut self.world, &mut people, &[])?; }
        ensure!(serde_json::to_value((self.stage.snapshot(), self.roster.frame(1, 0)))? == before, "Paused door changed state");
        let bytes = crate::coop::codec::encode(&serde_json::to_vec(&before)?)?;
        let (stage, frame) = serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?;
        self.stage.restore(&stage, map, &mut self.world)?;
        self.roster = Roster::from_checkpoint(&frame, &self.world, |p| !self.stage.interactions.hazardous(p))?;
        ensure!(serde_json::to_value((self.stage.snapshot(), self.roster.frame(1, 0)))? == before,
            "Door save/restore lost bodies, waiting order or world state");
        Ok(())
    }
}
fn crossing(a: &mut Assets, map: &Bsp, laboratory: bool, first: PlayerId, dead: u8) -> Result<()> {
    let mut c = Course::new(a, map, laboratory, first, false)?;
    c.walk(map, first, true)?;
    for _ in 0..60 { c.step(map, [None; 2], true)?; }
    ensure!(c.stage.notices[first.index()].as_ref().is_some_and(|n| n.text == "Waiting for your partner"),
        "Missing partner-waiting notice");
    c.checkpoint(map)?;
    if dead & 1 != 0 {
        c.roster.players[first.index()].stats.damage(1000.); c.health[first.index()] = 0.; c.expected_owner = first.partner();
    }
    if dead & 2 != 0 { c.roster.players[first.partner().index()].stats.damage(1000.); c.health[first.partner().index()] = 0.; }
    if dead == 3 {
        let age = ctl(&mut c.stage).saved.age;
        for _ in 0..120 { c.step(map, [None; 2], true)?; }
        ensure!(ctl(&mut c.stage).saved.age == age, "Dead team advanced closing machinery");
    } else {
        if dead != 2 { c.walk(map, first.partner(), false)?; }
        else {
            // Re-enter the trigger normally so an absent/dead partner no longer
            // blocks a waiting living caller who had already walked beyond it.
            let id = first.index();
            let y = c.roster.players[id].player.feet.y;
            let goal = vec2(c.closing_x - PLAYER_HALF.x - 1., y);
            for _ in 0..240 {
                let mut goals = [None; 2]; goals[id] = Some(goal); c.step(map, goals, false)?;
                if c.closed().is_some() { break; }
            }
        }
        for _ in 0..120 { c.step(map, [None; 2], false)?; }
        ensure!(c.closed().is_some(), "Door never closed after the surviving players passed");
        let names: &[&str] = if laboratory { &["ClockDoor_Left", "ClockDoor_Right"] } else { &["no_return"] };
        for name in names {
            let o = ctl(&mut c.stage).objects.iter().find(|o| o.name == *name).unwrap();
            ensure!(o.pose.translation.distance(o.base.translation) < 0.01, "Door did not complete its original closing motion");
        }
        let mut saved = c.stage.snapshot(); saved.contacts = Contacts::new(&c.stage.interactions);
        c.stage.restore(&saved, map, &mut c.world)?;
        for _ in 0..10 { c.step(map, [None; 2], false)?; }
    }
    println!("PASS Clockwork closing door laboratory{laboratory} first{first:?} dead{dead}: normal crossing, waiting restore, living ownership and original closure");
    Ok(())
}
fn lever_wait(a: &mut Assets, map: &Bsp, first: PlayerId) -> Result<()> {
    let mut c = Course::new(a, map, true, first, true)?;
    c.walk(map, first, true)?;
    for _ in 0..120 { c.step(map, [None; 2], true)?; }
    c.checkpoint(map)?;
    ensure!(c.closed().is_none(), "Laboratory lever alcove counted as inside the laboratory");
    println!("PASS Clockwork laboratory first{first:?}: a partner at the outside lever does not permit closure");
    Ok(())
}
fn ordinary_door(a: &mut Assets, map: &Bsp, first: PlayerId, proximity_open: bool) -> Result<()> {
    let mut c = Course::new(a, map, true, first, false)?;
    for id in PlayerId::ALL {
        let feet = if id == first { vec3(-3302.0173, 2779.03125, 192.03125) }
            else { vec3(-3266.425, 2784.4333, 192.03125) };
        let body = &mut c.roster.players[id.index()];
        body.player = Player::new(feet); body.player.grounded = true;
        c.anchors[id.index()] = feet.truncate();
    }
    if proximity_open {
        // Captured fresh-route stall: the northern automatic leaf has opened,
        // but E must still open its closed neighbor through native use_door.
        let o = ctl(&mut c.stage);
        let k = o.objects.iter().position(|o| o.id == 119).context("Missing entrance door")?;
        o.saved.doors[k] = 1.; o.saved.holds[k] = 3.;
        o.rebuild(map)?; c.stage.interactions.sync_coop(&mut c.world);
    }
    c.safe()?;
    let eye = c.roster.players[first.index()].player.eye();
    let door = c.stage.interactions.closed_door_in_reach(&c.world, eye, Vec3::NEG_Y)
        .context("Route driver cannot observe Clockwork's ordinary door")?;
    ensure!(c.stage.interactions.closed_door(door), "Closed native door was not identified");
    let mut input = [(Controls::default(), Input::default()); 2];
    input[first.index()].0.use_pressed = true;
    input[first.index()].1.yaw = -std::f32::consts::FRAC_PI_2;
    c.stage.before_move(&mut c.roster, map, &mut c.world, &input, FIXED_DT)?;
    ensure!(!c.stage.interactions.closed_door(door), "Ordinary E press did not acknowledge opening");
    if proximity_open {
        // The captured partner was standing on the incoming side beside the
        // open leaf, pinning the actor against its tip. Walk that partner
        // through first; opening a door must not bypass personal space.
        let partner = first.partner();
        let waiting = vec2(-3266.425, 2700.);
        for _ in 0..240 {
            let mut goals = [None; 2]; goals[partner.index()] = Some(waiting);
            c.step(map, goals, true)?;
            if c.roster.players[partner.index()].player.feet.truncate().distance(waiting) < 5. { break; }
        }
        ensure!(c.roster.players[partner.index()].player.feet.truncate().distance(waiting) < 5.,
            "Waiting partner could not walk out of the incoming door approach");
        c.anchors[partner.index()] = waiting;
    }
    let goal = vec2(-3296., 2750.);
    for _ in 0..240 {
        let mut goals = [None; 2]; goals[first.index()] = Some(goal);
        c.step(map, goals, true)?;
        if c.roster.players[first.index()].player.feet.truncate().distance(goal) < 5. { break; }
    }
    ensure!(c.roster.players[first.index()].player.feet.truncate().distance(goal) < 5.,
        "Ordinary E did not allow the player to cross the door");
    println!("PASS Clockwork ordinary door {first:?} proximity_open{proximity_open}: native observation, one E press and actual passage");
    Ok(())
}
pub(super) fn check(a: &mut Assets, map: &Bsp) -> Result<()> {
    let mut failures = Vec::new();
    for laboratory in [false, true] { for first in PlayerId::ALL { for dead in 0..4 {
        if let Err(e) = crossing(a, map, laboratory, first, dead) {
            let message = format!("laboratory{laboratory} first{first:?} dead{dead}: {e:#}");
            eprintln!("FAIL Clockwork closing {message}"); failures.push(message);
        }
    }}}
    for first in PlayerId::ALL {
        if let Err(e) = lever_wait(a, map, first) {
            let message = format!("laboratory lever first{first:?}: {e:#}");
            eprintln!("FAIL Clockwork closing {message}"); failures.push(message);
        }
        for proximity_open in [false, true] { if let Err(e) = ordinary_door(a, map, first, proximity_open) {
            let message = format!("ordinary door first{first:?} proximity_open{proximity_open}: {e:#}");
            eprintln!("FAIL Clockwork {message}"); failures.push(message);
        } }
    }
    ensure!(failures.is_empty(), "Closing door cases failed: {}", failures.join("; "));
    Ok(())
}
