//! Two-body contacts over one authoritative world. Player contact history is
//! separate from the single-player save representation and from shared rules.
use super::*;
use crate::coop::PlayerId;
use anyhow::ensure;
use serde::{Deserialize, Serialize};
mod lift_check;

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Contact {
    inside: bool,
    cooldown: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TriggerContacts {
    id: Id,
    players: [Contact; 2],
    waiting: Option<PlayerId>,
}

/// Save/restore this together with the shared interaction snapshot. A respawn
/// clears only that player's history; it must not rearm a shared one-shot rule.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contacts {
    triggers: Vec<TriggerContacts>,
}
impl Contacts {
    pub fn new(world: &Interactions) -> Self {
        Self {
            triggers: world
                .triggers
                .iter()
                .map(|t| TriggerContacts {
                    id: t.id,
                    players: [Contact::default(); 2],
                    waiting: None,
                })
                .collect(),
        }
    }
    pub fn reset_player(&mut self, id: PlayerId) {
        for trigger in &mut self.triggers {
            trigger.players[id.index()] = Contact::default();
            if trigger.waiting == Some(id) {
                trigger.waiting = None;
            }
        }
    }
    fn relocate_player(&mut self, id: PlayerId) {
        for trigger in &mut self.triggers {
            trigger.players[id.index()].inside = false;
            if trigger.waiting == Some(id) {
                trigger.waiting = None;
            }
        }
    }
    pub fn validate(&self, world: &Interactions) -> Result<()> {
        ensure!(
            self.triggers.len() == world.triggers.len(),
            "Co-op contact count does not match the visit"
        );
        for (saved, trigger) in self.triggers.iter().zip(&world.triggers) {
            ensure!(
                saved.id == trigger.id
                    && saved
                        .players
                        .iter()
                        .all(|p| p.cooldown.is_finite() && (0. ..=1.).contains(&p.cooldown)),
                "Invalid co-op trigger contact"
            );
        }
        Ok(())
    }
}

/// Only living, freely controlled players supply a path. Scenes/respawns must
/// not sweep a teleported body through every trigger between the two positions.
#[derive(Clone, Copy)]
pub struct Motion {
    pub before: Vec3,
    pub after: Vec3,
}

pub struct Activation {
    pub player: PlayerId,
    pub entity: Id,
    pub events: Events,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContactFlow {
    Continue,
    /// Death or relocation invalidated the rest of this body's old sweep.
    StopPlayer,
    /// A shared scene/transition took control of both players.
    StopWorld,
}

#[derive(Default)]
pub struct PairEvents {
    pub waiting: [bool; 2],
    /// Delayed world outputs, produced once before either player's contacts.
    pub shared: Events,
    /// Ordered by swept arrival time, then stable entity/player identity for ties.
    /// Damage and teleport affect this player; story/exit ownership is shared.
    pub activations: Vec<Activation>,
}

impl Interactions {
    /// Co-op grips belong to each replicated Player. The original Pandemonium
    /// controller keeps its one-body grip for single-player saves and play.
    pub fn sync_coop(&self, world: &mut World) {
        self.sync(world);
        world.traversal.managed_ropes = false;
    }

    fn boarding_ready(&self, trigger: usize, feet: &[Vec3]) -> Option<bool> {
        let TriggerKind::Script(thread) = &self.triggers[trigger].kind else {
            return None;
        };
        if self.pandemonium.is_some() && thread == "Minecart_Thread" {
            return Some(!feet.is_empty() && feet.iter().all(|at| {
                self.triggers[trigger].volume.touches(*at + PLAYER_CENTER, *at + PLAYER_CENTER, PLAYER_HALF)
            }));
        }
        self.pool.as_ref().and_then(|p| p.boarding_ready(thread, feet))
            .or_else(|| self.beyond.as_ref().and_then(|b| b.boarding_ready(thread, feet))).or_else(|| self.school
            .as_ref()
            .and_then(|s| s.boarding_ready(thread, feet)))
            .or_else(|| self.levels.iter().find_map(|s| s.ctl.coop_boarding_ready(thread, feet)))
    }

    /// The reviewed adapters advance shared machinery once and carry all living
    /// bodies atomically. Other controllers require their own explicit adapters.
    pub fn movers_pair(
        &mut self,
        dt: f32,
        map: &Bsp,
        world: &mut World,
        players: &mut [&mut Player],
    ) -> Result<()> {
        if let Some(beyond) = &mut self.beyond {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone()).collect::<Vec<_>>();
            beyond.advance_players(dt, map, world, players, &fixed)?;
            for trigger in &mut self.triggers {
                let entity = &map.entities[trigger.id.0];
                if let Some(base) = entity.get("origin").and_then(|s| vector(s)) {
                    if let Some((position, rotation)) = beyond.trigger_pose(&trigger.name, base) {
                        trigger.volume = Collider::model(map, model(entity, map).unwrap(), position, rotation, false)?;
                    }
                }
            }
        }
        if let Some(fortress) = &mut self.fortress {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone()).collect::<Vec<_>>();
            fortress.advance_players(dt, map, world, players, &fixed)?;
            for trigger in &mut self.triggers {
                let entity = &map.entities[trigger.id.0];
                if let Some(base) = entity.get("origin").and_then(|s| vector(s)) {
                    if let Some((position, rotation)) = fortress.trigger_pose(&trigger.name, base) {
                        trigger.volume = Collider::model(map, model(entity, map).unwrap(), position, rotation, false)?;
                    }
                }
            }
        }
        if let Some(pandemonium) = &mut self.pandemonium {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone()).collect::<Vec<_>>();
            pandemonium.advance_players(dt, map, world, players, &fixed)?;
        }
        if let Some(village) = &mut self.village {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone()).collect::<Vec<_>>();
            village.advance_players(dt, map, world, players, &fixed)?;
        }
        if let Some(school) = &mut self.school {
            let fixed = self
                .doors
                .iter()
                .filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone())
                .chain(self.platforms.iter().map(|p| p.2.clone()))
                .collect::<Vec<_>>();
            school.advance_players(dt, map, world, players, &fixed)?;
        }
        if let Some(gym) = &mut self.gym {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone())
                .chain(self.school2.iter().flat_map(|s| s.colliders())).collect::<Vec<_>>();
            let clock = if self.school2.as_ref().is_some_and(|s| s.hold_bleachers()) { 0. } else { dt };
            gym.advance_players(clock, map, world, players, &fixed)?;
        }
        if let Some(school2) = &mut self.school2 {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone())
                .chain(self.gym.iter().flat_map(|g| g.colliders())).collect::<Vec<_>>();
            school2.advance_players(dt, map, world, players, &fixed)?;
        }
        if let Some(pool) = &mut self.pool {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone()).collect::<Vec<_>>();
            pool.advance_players(dt, map, world, players, &fixed)?;
        }
        for index in 0..self.levels.len() {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone())
                .chain(self.platforms.iter().map(|p| p.2.clone()))
                .chain(self.levels.iter().enumerate().filter(|(i, _)| *i != index)
                    .flat_map(|(_, s)| s.ctl.colliders())).collect::<Vec<_>>();
            self.levels[index].ctl.coop_advance(dt, map, world, players, &fixed)?;
        }
        self.sync_coop(world);
        Ok(())
    }
    /// Door input and one door-motion step. Controllers/movers keep their own
    /// once-per-world step; this does not call a single-player controller twice.
    pub fn doors_pair(
        &mut self,
        dt: f32,
        map: &Bsp,
        world: &mut World,
        players: [Option<(&Player, Vec3, bool)>; 2],
    ) -> Result<([Events; 2], Option<PlayerId>)> {
        let mut events = std::array::from_fn(|_| Events::default());
        let mut scene_owner = None;
        if dt <= 0. || !dt.is_finite() {
            return Ok((events, scene_owner));
        }
        let mut operated = BTreeSet::new();
        let bodies = players.iter().flatten().map(|p| p.0).collect::<Vec<_>>();
        self.refresh_door_locks(&bodies);
        for d in &mut self.doors {
            if let Some(locked) = self.levels.iter().find_map(|s| s.ctl.door_locked(d.id)) {
                d.locked = locked;
                if locked {
                    d.target = 0.;
                }
            }
        }
        for (index, input) in players.iter().enumerate() {
            if let Some((player, aim, true)) = input {
                if let Some(event) = self.beyond.as_mut().and_then(|b| b.activate(world, player.eye(), *aim)) {
                    events[index].merge(event);
                    self.used += 1;
                } else if let Some(event) = self.use_gym(world, player, *aim) {
                    events[index].merge(event);
                } else {
                    events[index].merge(self.use_door(world, player, *aim, &mut operated));
                }
                // A lever can start an original one-Alice scene. Later input in
                // this host tick must not operate another control through it.
                if self.scripted() {
                    scene_owner = Some(PlayerId::ALL[index]);
                    break;
                }
            }
        }
        self.advance_doors(dt, map, &bodies)?;
        self.sync_coop(world);
        Ok((events, scene_owner))
    }

    /// Personal contacts with a shared quest item must include either player.
    /// Rope use stays with Player::tick, so neither grip consumes the other's E.
    pub fn pandemonium_player(&mut self, world: &World, player: &Player) -> Events {
        let Some(p) = &mut self.pandemonium else { return Events::default(); };
        let events = p.update(world, player, false);
        if p.state.leaving {
            for door in &mut self.doors {
                if door.name == "t141" { door.enabled = false; }
            }
        }
        events
    }

    pub fn triggers_pair(
        &mut self,
        dt: f32,
        contacts: &mut Contacts,
        paths: [Option<Motion>; 2],
    ) -> Result<PairEvents> {
        self.triggers_pair_resolve(dt, contacts, paths, |_| ContactFlow::Continue)
    }

    /// Resolve personal damage/teleport immediately after each authoritative
    /// dispatch, before a later arrival can consume a shared rule or start a scene.
    pub fn triggers_pair_resolve(
        &mut self,
        dt: f32,
        contacts: &mut Contacts,
        paths: [Option<Motion>; 2],
        mut resolve: impl FnMut(&Activation) -> ContactFlow,
    ) -> Result<PairEvents> {
        contacts.validate(self)?;
        ensure!(
            paths
                .iter()
                .flatten()
                .all(|p| p.before.is_finite() && p.after.is_finite()),
            "Invalid co-op trigger path"
        );
        let mut result = PairEvents::default();
        if dt <= 0. || !dt.is_finite() {
            return Ok(result);
        }
        result.shared = self.advance_trigger_clock(dt);
        if self.school.as_ref().is_some_and(|s| s.cinematic())
            || self.scripted()
            || result.shared.transition.is_some()
            || !result.shared.story.is_empty()
        {
            return Ok(result);
        }
        let mut arrivals = Vec::new();
        for index in 0..self.triggers.len() {
            if !self.contact_trigger(index) {
                continue;
            }
            let trigger = &self.triggers[index];
            let enabled = self.event_world.allowed(
                &Event::Entity(trigger.id, Input::Activate),
                &self.event_facts(),
            );
            for id in PlayerId::ALL {
                let contact = &mut contacts.triggers[index].players[id.index()];
                let Some(path) = paths[id.index()].filter(|_| enabled) else {
                    contact.inside = false;
                    contact.cooldown = (contact.cooldown - dt).max(0.);
                    continue;
                };
                contact.cooldown = (contact.cooldown - dt).max(0.);
                let hit = trigger.volume.trace(
                    path.before + PLAYER_CENTER,
                    path.after + PLAYER_CENTER,
                    PLAYER_HALF,
                );
                let crossed = hit.start_solid || hit.fraction < 1.;
                if crossed
                    && (!contact.inside || matches!(trigger.kind, TriggerKind::Hurt(_)))
                    && contact.cooldown == 0.
                {
                    arrivals.push((if hit.start_solid { 0. } else { hit.fraction }, index, id));
                }
                contact.inside = trigger.volume.touches(
                    path.after + PLAYER_CENTER,
                    path.after + PLAYER_CENTER,
                    PLAYER_HALF,
                );
            }
        }
        arrivals.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        let mut shared_fired = BTreeSet::new();
        let mut stopped = [false; 2];
        for (_, index, mut id) in arrivals {
            if stopped[id.index()] || !self.contact_trigger(index) {
                continue;
            }
            let trigger = &self.triggers[index];
            let personal = matches!(
                trigger.kind,
                TriggerKind::Hurt(_) | TriggerKind::Fall | TriggerKind::Teleport(..)
            );
            if !personal && shared_fired.contains(&index) {
                continue;
            }
            let entity = trigger.id;
            let event = Event::Entity(entity, Input::Activate);
            if !self.event_world.allowed(&event, &self.event_facts()) {
                continue;
            }
            let feet = paths
                .iter()
                .enumerate()
                .filter(|(i, p)| p.is_some() && !stopped[*i])
                .map(|(_, p)| p.unwrap().after)
                .collect::<Vec<_>>();
            if let Some(ready) = self.boarding_ready(index, &feet) {
                let pending = &mut contacts.triggers[index];
                if pending
                    .waiting
                    .is_some_and(|owner| paths[owner.index()].is_none() || stopped[owner.index()])
                {
                    pending.waiting = None;
                }
                let owner = *pending.waiting.get_or_insert(id);
                if !ready {
                    // Retry while the requester's feet remain in the trigger;
                    // do not consume its one-shot rule or personal cooldown.
                    pending.players[id.index()].inside = false;
                    result.waiting[owner.index()] = true;
                    continue;
                }
                pending.waiting = None;
                id = owner;
            }
            if !personal {
                shared_fired.insert(index);
            }
            // The shared callback still owns authored consequences. Its old
            // single-body cooldown is copied to the triggering body's slot,
            // then restored so it cannot suppress the partner's contact.
            let old_cooldown = trigger.cooldown;
            let events = self.dispatch(event);
            contacts.triggers[index].players[id.index()].cooldown = self.triggers[index].cooldown;
            self.triggers[index].cooldown = old_cooldown;
            let takes_control =
                self.scripted() || !events.story.is_empty() || events.transition.is_some();
            let relocated = events.teleport.is_some();
            let activation = Activation {
                player: id,
                entity,
                events,
            };
            let flow = resolve(&activation);
            result.activations.push(activation);
            if relocated || flow == ContactFlow::StopPlayer {
                stopped[id.index()] = true;
                // Keep portal cooldowns: clearing them can create a teleport
                // loop when an authored destination overlaps its own volume.
                contacts.relocate_player(id);
            }
            if takes_control || flow == ContactFlow::StopWorld {
                break;
            }
        }
        Ok(result)
    }
}

/// Actual school geometry, ordinary movement, both opener roles, and two
/// independent contacts with the authored fatal-fall volume. No rendered claim.
pub fn check(assets: &mut crate::assets::Assets) -> Result<()> {
    use crate::movement::{Controls, FixedClock};
    let map = Bsp::parse(&assets.read("maps/skool1.bsp")?)?;
    for opener in PlayerId::ALL {
        for fps in [30, 60, 144] {
            let mut world = World::from_bsp(&map)?;
            let mut interactions = Interactions::load(&map)?;
            interactions.sync(&mut world);
            let blue = Player::spawn(&world, map.spawn().0)
                .ok_or_else(|| anyhow::anyhow!("School start obstructed"))?;
            let red = Player::spawn(&world, blue.eye() + Vec3::Y * 48.)
                .ok_or_else(|| anyhow::anyhow!("Partner start obstructed"))?;
            let mut players = [blue, red];
            let mut clocks = [FixedClock::default(), FixedClock::default()];
            let dt = 1. / fps as f32;
            let movement = Controls {
                wish: Vec2::Y,
                ..Default::default()
            };
            for _ in 0..fps * 6 {
                for (clock, player) in clocks.iter_mut().zip(&mut players) {
                    clock.advance(dt as f64, &world, player, movement);
                }
            }
            ensure!(
                players.iter().all(|p| p.feet.y > 2100. && p.feet.y < 2212.),
                "Closed school door did not stop both players"
            );
            let stopped = players.each_ref().map(|p| p.feet);
            for _ in 0..fps {
                for (clock, player) in clocks.iter_mut().zip(&mut players) {
                    clock.advance(dt as f64, &world, player, movement);
                }
            }
            ensure!(
                players
                    .iter()
                    .zip(stopped)
                    .all(|(p, at)| p.feet.distance(at) < 1.),
                "Players were still approaching the door, not blocked by it"
            );
            interactions.doors_pair(
                dt,
                &map,
                &mut world,
                std::array::from_fn(|i| Some((&players[i], Vec3::Y, i == opener.index()))),
            )?;
            ensure!(
                interactions.used == 1,
                "School door use was not owned by the chosen player"
            );
            for _ in 0..fps * 2 {
                interactions.doors_pair(
                    dt,
                    &map,
                    &mut world,
                    players.each_ref().map(|p| Some((p, Vec3::Y, false))),
                )?;
                for (clock, player) in clocks.iter_mut().zip(&mut players) {
                    clock.advance(dt as f64, &world, player, movement);
                    ensure!(
                        world.body_clear(player.feet),
                        "Door embedded a co-op player"
                    );
                }
            }
            ensure!(
                players.iter().all(|p| p.feet.y > 2260.),
                "Both players did not cross the opened school door"
            );
            println!("PASS co-op school door: {opener:?} opens, both cross, {fps} Hz");
        }
    }
    let mut interactions = Interactions::load(&map)?;
    let mut contacts = Contacts::new(&interactions);
    let motion = Motion {
        before: vec3(692., 4364., 120.),
        after: vec3(692., 4364., -20.),
    };
    let events = interactions.triggers_pair(0.1, &mut contacts, [Some(motion); 2])?;
    for player in PlayerId::ALL {
        ensure!(
            events.activations.iter().any(|e| e.player == player
                && e.events.damage >= 100.
                && map.entities[e.entity.0]
                    .get("classname")
                    .is_some_and(|s| s == "trigger_fall")),
            "Fatal fall did not hit both players"
        );
    }
    let saved_world = interactions.snapshot();
    let saved_contacts = serde_json::to_vec(&contacts)?;
    let mut restored = Interactions::load(&map)?;
    restored.restore(&saved_world, &map)?;
    let mut contacts: Contacts = serde_json::from_slice(&saved_contacts)?;
    contacts.reset_player(PlayerId::Blue);
    let repeated = restored.triggers_pair(0.1, &mut contacts, [Some(motion); 2])?;
    ensure!(
        repeated
            .activations
            .iter()
            .any(|e| e.player == PlayerId::Blue && e.events.damage >= 100.)
            && !repeated
                .activations
                .iter()
                .any(|e| e.player == PlayerId::Red && e.events.damage > 0.),
        "Restoring/resetting one contact changed the partner's cooldown"
    );
    println!("PASS co-op authored school fatal fall: personal damage, snapshot roundtrip and independent respawn reset");
    lift_check::check(assets, &map)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trigger(class: &str) -> (Bsp, Interactions, Contacts) {
        let mut map = super::super::tests::fixture();
        map.entities[0].insert("classname".into(), class.into());
        map.entities[0].insert("thread".into(), String::new());
        if class == "trigger_teleport" {
            map.entities[0].insert("target".into(), "landing".into());
            map.entities.push(Entity::from([
                ("classname".into(), "func_teleportdest".into()),
                ("targetname".into(), "landing".into()),
                ("origin".into(), "200 200 0".into()),
            ]));
        }
        let world = Interactions::load(&map).unwrap();
        let contacts = Contacts::new(&world);
        (map, world, contacts)
    }
    fn motion(before: f32, after: f32) -> Option<Motion> {
        Some(Motion {
            before: vec3(32., before, 0.),
            after: vec3(32., after, 0.),
        })
    }
    fn damage(events: &PairEvents, id: PlayerId) -> f32 {
        events
            .activations
            .iter()
            .filter(|e| e.player == id)
            .map(|e| e.events.damage)
            .sum()
    }

    #[test]
    fn both_hazard_victims_keep_independent_cooldowns_and_dead_bodies_do_not_fire() {
        let (_, mut world, mut contacts) = trigger("trigger_hurt");
        let blue = world
            .triggers_pair(0.1, &mut contacts, [motion(40., 100.), None])
            .unwrap();
        assert_eq!(damage(&blue, PlayerId::Blue), 10.);
        let red = world
            .triggers_pair(0.1, &mut contacts, [motion(100., 100.), motion(40., 100.)])
            .unwrap();
        assert_eq!(damage(&red, PlayerId::Blue), 0.);
        assert_eq!(damage(&red, PlayerId::Red), 10.);
        let blue = world
            .triggers_pair(0.41, &mut contacts, [motion(100., 100.); 2])
            .unwrap();
        assert_eq!(damage(&blue, PlayerId::Blue), 10.);
        assert_eq!(damage(&blue, PlayerId::Red), 0.);
        let red = world
            .triggers_pair(0.1, &mut contacts, [None, motion(100., 100.)])
            .unwrap();
        assert_eq!(damage(&red, PlayerId::Blue), 0.);
        assert_eq!(damage(&red, PlayerId::Red), 10.);
        contacts.reset_player(PlayerId::Blue);
        let both = world
            .triggers_pair(0.6, &mut contacts, [motion(40., 100.); 2])
            .unwrap();
        assert_eq!(both.activations.len(), 2);
        assert!(both.activations.iter().all(|e| e.events.damage == 10.));
    }

    #[test]
    fn teleports_belong_to_each_contact_not_to_a_shared_cooldown() {
        let (_, mut world, mut contacts) = trigger("trigger_teleport");
        let both = world
            .triggers_pair(0.1, &mut contacts, [motion(40., 100.); 2])
            .unwrap();
        assert_eq!(both.activations.len(), 2);
        for (event, id) in both.activations.iter().zip(PlayerId::ALL) {
            assert_eq!(event.player, id);
            assert_eq!(event.events.teleport.unwrap().0, vec3(200., 200., 0.));
        }
        contacts.reset_player(PlayerId::Red);
        let again = world
            .triggers_pair(0.1, &mut contacts, [motion(100., 100.); 2])
            .unwrap();
        assert_eq!(again.activations.len(), 1);
        assert_eq!(again.activations[0].player, PlayerId::Red);
    }

    #[test]
    fn fatal_contact_cannot_consume_a_later_shared_exit_for_either_player() {
        for victim in PlayerId::ALL {
            let (mut map, _, _) = trigger("trigger_hurt");
            let mut exit = map.entities[0].clone();
            exit.insert("classname".into(), "trigger_changelevel".into());
            exit.insert("origin".into(), "0 200 0".into());
            exit.insert("map".into(), "skool2".into());
            map.entities.push(exit);
            let mut world = Interactions::load(&map).unwrap();
            let mut contacts = Contacts::new(&world);
            let mut stats = crate::inventory::Stats::default();
            stats.set_health(1.).unwrap();
            let paths =
                std::array::from_fn(|i| (i == victim.index()).then(|| motion(40., 240.).unwrap()));
            let events = world
                .triggers_pair_resolve(0.1, &mut contacts, paths, |activation| {
                    stats.damage(activation.events.damage);
                    if stats.alive() {
                        ContactFlow::Continue
                    } else {
                        ContactFlow::StopPlayer
                    }
                })
                .unwrap();
            assert!(!stats.alive());
            assert_eq!(events.activations.len(), 1);
            assert_eq!(world.event_world.snapshot().usage["trigger/1"].count, 0);
            let partner = std::array::from_fn(|i| {
                (i == victim.partner().index()).then(|| motion(160., 240.).unwrap())
            });
            let events = world.triggers_pair(0.1, &mut contacts, partner).unwrap();
            assert_eq!(events.activations.len(), 1);
            assert_eq!(events.activations[0].player, victim.partner());
            assert_eq!(
                events.activations[0].events.transition,
                Some(("skool2".into(), None))
            );
        }
    }

    #[test]
    fn portal_stops_old_sweep_but_keeps_cooldown_and_partners_arrival() {
        for traveller in PlayerId::ALL {
            let (mut map, _, _) = trigger("trigger_teleport");
            let mut exit = map.entities[0].clone();
            exit.insert("classname".into(), "trigger_changelevel".into());
            exit.insert("origin".into(), "0 200 0".into());
            exit.insert("map".into(), "skool2".into());
            map.entities.push(exit);
            let mut world = Interactions::load(&map).unwrap();
            let mut contacts = Contacts::new(&world);
            let paths = std::array::from_fn(|i| {
                if i == traveller.index() {
                    motion(40., 240.)
                } else {
                    motion(150., 195.)
                }
            });
            let events = world.triggers_pair(0.1, &mut contacts, paths).unwrap();
            assert_eq!(events.activations.len(), 2);
            assert_eq!(events.activations[0].player, traveller);
            assert!(events.activations[0].events.teleport.is_some());
            assert_eq!(events.activations[1].player, traveller.partner());
            assert!(events.activations[1].events.transition.is_some());
            assert!(contacts.triggers[0].players[traveller.index()].cooldown > 0.);
            // Reappear inside the portal volume: it cannot immediately bounce.
            let paths = std::array::from_fn(|i| {
                (i == traveller.index()).then(|| motion(100., 100.).unwrap())
            });
            assert!(world
                .triggers_pair(0.01, &mut contacts, paths)
                .unwrap()
                .activations
                .is_empty());
        }
    }

    #[test]
    fn first_swept_arrival_owns_shared_once_trigger_in_either_order() {
        for first in PlayerId::ALL {
            let (_, mut world, mut contacts) = trigger("trigger_once");
            let paths = std::array::from_fn(|i| {
                if i == first.index() {
                    motion(60., 100.)
                } else {
                    motion(0., 100.)
                }
            });
            let events = world.triggers_pair(0.1, &mut contacts, paths).unwrap();
            assert_eq!(events.activations.len(), 1);
            assert_eq!(events.activations[0].player, first);
            assert_eq!(events.activations[0].entity, Id(0));
            contacts = Contacts::new(&world);
            assert!(world
                .triggers_pair(1., &mut contacts, paths)
                .unwrap()
                .activations
                .is_empty());
            assert_eq!(world.event_world.snapshot().usage["trigger/0"].count, 1);
        }
    }

    #[test]
    fn partner_occupancy_does_not_retrigger_or_suppress_reentry() {
        let (_, mut world, mut contacts) = trigger("trigger_multiple");
        assert_eq!(
            world
                .triggers_pair(0.1, &mut contacts, [motion(40., 100.); 2])
                .unwrap()
                .activations
                .len(),
            1
        );
        assert!(world
            .triggers_pair(0.1, &mut contacts, [motion(100., 100.); 2])
            .unwrap()
            .activations
            .is_empty());
        assert!(world
            .triggers_pair(0.1, &mut contacts, [motion(100., 40.), motion(100., 100.)])
            .unwrap()
            .activations
            .is_empty());
        let events = world
            .triggers_pair(0.1, &mut contacts, [motion(40., 100.), motion(100., 100.)])
            .unwrap();
        assert_eq!(events.activations.len(), 1);
        assert_eq!(events.activations[0].player, PlayerId::Blue);
    }

    #[test]
    fn competing_exits_use_arrival_time_instead_of_entity_or_player_order() {
        for first in PlayerId::ALL {
            let mut map = super::super::tests::fixture();
            map.entities[0].insert("classname".into(), "trigger_changelevel".into());
            map.entities[0].insert("map".into(), "later".into());
            let mut earlier = map.entities[0].clone();
            earlier.insert("origin".into(), "0 200 0".into());
            earlier.insert("map".into(), "earlier".into());
            map.entities.push(earlier);
            let mut world = Interactions::load(&map).unwrap();
            let mut contacts = Contacts::new(&world);
            let paths = std::array::from_fn(|i| {
                if i == first.index() {
                    motion(160., 220.)
                } else {
                    motion(0., 100.)
                }
            });
            let events = world.triggers_pair(0.1, &mut contacts, paths).unwrap();
            assert_eq!(events.activations.len(), 1);
            assert_eq!(events.activations[0].player, first);
            assert_eq!(events.activations[0].entity, Id(1));
            assert_eq!(
                events.activations[0].events.transition,
                Some(("earlier".into(), None))
            );
            assert_eq!(world.event_world.snapshot().usage["trigger/0"].count, 0);
        }
    }

    #[test]
    fn shared_delayed_world_events_advance_once_and_pause_with_zero_time() {
        let (map, mut world, mut contacts) = trigger("trigger_multiple");
        let mut initial = Facts::default();
        initial.count("delivered", 0);
        world.event_world = Runtime::new(
            Registry::new(&map.entities),
            vec![
                Rule {
                    key: "start".into(),
                    event: Event::Signal("start".into()),
                    condition: Condition::Always,
                    once: true,
                    cooldown: 0.,
                    actions: vec![Action::Send {
                        event: Event::Signal("later".into()),
                        delay: 1.,
                    }],
                },
                Rule {
                    key: "later".into(),
                    event: Event::Signal("later".into()),
                    condition: Condition::Always,
                    once: true,
                    cooldown: 0.,
                    actions: vec![Action::Count("delivered".into(), 1)],
                },
            ],
            initial,
            Facts::default(),
        )
        .unwrap();
        world.dispatch(Event::Signal("start".into()));
        for dt in [0., 0.6, 0.] {
            world
                .triggers_pair(dt, &mut contacts, [motion(40., 40.); 2])
                .unwrap();
            assert_eq!(
                world.event_world.snapshot().facts.0["delivered"],
                event::Value::Count(0)
            );
        }
        world.triggers_pair(0.5, &mut contacts, [None; 2]).unwrap();
        assert_eq!(
            world.event_world.snapshot().facts.0["delivered"],
            event::Value::Count(1)
        );
    }

    #[test]
    fn world_and_contact_restore_keep_personal_cooldowns_and_validate_identity() {
        let (map, mut world, mut contacts) = trigger("trigger_hurt");
        world
            .triggers_pair(0.1, &mut contacts, [motion(40., 100.), None])
            .unwrap();
        let saved = world.snapshot();
        let bytes = serde_json::to_vec(&contacts).unwrap();
        let mut restored = Interactions::load(&map).unwrap();
        restored.restore(&saved, &map).unwrap();
        let mut contacts: Contacts = serde_json::from_slice(&bytes).unwrap();
        let events = restored
            .triggers_pair(0.1, &mut contacts, [motion(100., 100.), motion(40., 100.)])
            .unwrap();
        assert_eq!(damage(&events, PlayerId::Blue), 0.);
        assert_eq!(damage(&events, PlayerId::Red), 10.);
        contacts.triggers[0].players[0].cooldown = f32::NAN;
        assert!(contacts.validate(&restored).is_err());
        contacts.reset_player(PlayerId::Blue);
        contacts.triggers[0].id = Id(100);
        assert!(contacts.validate(&restored).is_err());
    }

    #[test]
    fn simultaneous_door_use_is_one_toggle_and_sweep_checks_either_body() {
        for blocker in PlayerId::ALL {
            let map = super::super::tests::fixture();
            let mut world = World::from_bsp(&map).unwrap();
            let mut interactions = Interactions::load(&map).unwrap();
            interactions.sync(&mut world);
            let mut players = std::array::from_fn::<_, 2, _>(|_| Player::new(vec3(32., 40., 0.)));
            interactions
                .doors_pair(
                    0.05,
                    &map,
                    &mut world,
                    players.each_ref().map(|p| Some((p, Vec3::Y, true))),
                )
                .unwrap();
            assert_eq!(interactions.used, 1);
            assert_eq!(interactions.doors[0].angle, 4.5);
            assert_eq!(interactions.doors[0].target, 90.);
            for _ in 0..30 {
                interactions
                    .doors_pair(
                        0.05,
                        &map,
                        &mut world,
                        players.each_ref().map(|p| Some((p, Vec3::Y, false))),
                    )
                    .unwrap();
            }
            players[blocker.index()].feet = vec3(23., 123., 0.);
            interactions.doors[0].target = 0.;
            for _ in 0..30 {
                interactions
                    .doors_pair(
                        0.05,
                        &map,
                        &mut world,
                        players.each_ref().map(|p| Some((p, Vec3::Y, false))),
                    )
                    .unwrap();
                assert!(players.iter().all(|p| world.body_clear(p.feet)));
            }
            assert!(interactions.doors[0].angle > 1.);
        }
    }
}
