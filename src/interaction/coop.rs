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
    /// Arrival order while a shared rule is closed, or a personal portal is
    /// obstructed. Bodies must remain at the trigger; never queues damage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    deferred: Vec<PlayerId>,
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
                    deferred: Vec::new(),
                })
                .collect(),
        }
    }
    /// Keep Blue's existing contact/cooldown when a solo game becomes co-op.
    /// Red has no previous contacts and must arrive through the normal rules.
    pub fn from_single_player(world: &Interactions) -> Self {
        let mut contacts = Self::new(world);
        for (saved, trigger) in contacts.triggers.iter_mut().zip(&world.triggers) {
            saved.players[PlayerId::Blue.index()] = Contact {
                inside: trigger.inside, cooldown: trigger.cooldown,
            };
        }
        contacts
    }
    pub fn reset_player(&mut self, id: PlayerId) {
        for trigger in &mut self.triggers {
            trigger.players[id.index()] = Contact::default();
            trigger.deferred.retain(|p| *p != id);
            if trigger.waiting == Some(id) {
                trigger.waiting = None;
            }
        }
    }
    fn relocate_player(&mut self, id: PlayerId) {
        for trigger in &mut self.triggers {
            trigger.players[id.index()].inside = false;
            trigger.deferred.retain(|p| *p != id);
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
                    && saved.deferred.len() <= 2
                    && saved.deferred.iter().collect::<BTreeSet<_>>().len() == saved.deferred.len()
                    && (saved.deferred.is_empty() || !matches!(trigger.kind,
                        TriggerKind::Hurt(_) | TriggerKind::Fall))
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
    /// A blocked personal portal retries after its cooldown while still touched.
    RetryPlayer,
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
        if let TriggerKind::Exit(map, _) = &self.triggers[trigger].kind {
            return self.levels.iter().find_map(|s| s.ctl.coop_exit_boarding_ready(map, feet));
        }
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
        self.movers_pair_impl(dt, map, world, players, true)
    }
    pub fn movers_roster(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [crate::level::CoopPlayer<'_>; 2]) -> Result<()> {
        {
            let mut living = players.iter_mut().filter(|p| p.stats.alive())
                .map(|p| &mut *p.player).collect::<Vec<_>>();
            self.movers_pair_impl(dt, map, world, &mut living, false)?;
        }
        for index in 0..self.levels.len() {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone())
                .chain(self.platforms.iter().map(|p| p.2.clone()))
                .chain(self.levels.iter().enumerate().filter(|(i, _)| *i != index)
                    .flat_map(|(_, s)| s.ctl.colliders())).collect::<Vec<_>>();
            self.levels[index].ctl.coop_advance_roster(dt, map, world, players, &fixed)?;
        }
        self.level_trigger_poses(map)?;
        self.sync_coop(world);
        Ok(())
    }
    fn movers_pair_impl(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [&mut Player], registered: bool) -> Result<()> {
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
        if !registered { self.sync_coop(world); return Ok(()); }
        for index in 0..self.levels.len() {
            let fixed = self.doors.iter().filter(|d| d.enabled && self.event_world.enabled(d.id))
                .map(|d| d.collider.clone())
                .chain(self.platforms.iter().map(|p| p.2.clone()))
                .chain(self.levels.iter().enumerate().filter(|(i, _)| *i != index)
                    .flat_map(|(_, s)| s.ctl.colliders())).collect::<Vec<_>>();
            self.levels[index].ctl.coop_advance(dt, map, world, players, &fixed)?;
        }
        self.level_trigger_poses(map)?;
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
                for slot in &mut self.levels {
                    events[index].merge(slot.ctl.coop_update(index, world, player, *aim, true));
                }
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
        let personal_facts = PlayerId::ALL.map(|id| {
            for slot in &mut self.levels { slot.ctl.coop_trigger_actor(Some(id.index())); }
            let facts = self.event_facts();
            for slot in &mut self.levels { slot.ctl.coop_trigger_actor(None); }
            facts
        });
        let mut arrivals = Vec::new();
        for index in 0..self.triggers.len() {
            if !self.contact_trigger(index) {
                contacts.triggers[index].deferred.clear();
                continue;
            }
            let trigger = &self.triggers[index];
            let personal = matches!(trigger.kind,
                TriggerKind::Hurt(_) | TriggerKind::Fall | TriggerKind::Teleport(..))
                || matches!(&trigger.kind, TriggerKind::Script(thread)
                    if self.levels.iter().any(|s| s.ctl.coop_personal_thread(thread)));
            let mut present = [false; 2];
            let mut blocked = Vec::new();
            let mut eligible = Vec::new();
            for id in PlayerId::ALL {
                let enabled = self.event_world.allowed(&Event::Entity(trigger.id, Input::Activate),
                    &personal_facts[id.index()]);
                let deferred = contacts.triggers[index].deferred.contains(&id);
                let contact = &mut contacts.triggers[index].players[id.index()];
                let Some(path) = paths[id.index()] else {
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
                let inside = trigger.volume.touches(
                    path.after + PLAYER_CENTER,
                    path.after + PLAYER_CENTER,
                    PLAYER_HALF,
                );
                let fraction = if hit.start_solid { 0. } else { hit.fraction };
                // Remember a waiting shared activator even while the condition
                // is false. Rechecking both stationary bodies on the opening
                // frame would otherwise replace their order with a Blue tie.
                present[id.index()] = if enabled { crossed } else { inside };
                if !enabled {
                    contact.inside = false;
                    if !personal && inside { blocked.push((fraction, id)); }
                    continue;
                }
                if crossed
                    && (!contact.inside || deferred || matches!(trigger.kind, TriggerKind::Hurt(_)))
                    && contact.cooldown == 0.
                {
                    eligible.push((fraction, id));
                }
                contact.inside = inside;
            }
            let deferred = &mut contacts.triggers[index].deferred;
            deferred.retain(|id| present[id.index()]);
            blocked.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            for (_, id) in blocked {
                if !deferred.contains(&id) { deferred.push(id); }
            }
            for (fraction, id) in eligible {
                let order = deferred.iter().position(|p| *p == id).unwrap_or(2);
                arrivals.push((fraction, index, order, id));
            }
        }
        arrivals.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2)).then(a.3.cmp(&b.3)));
        let mut shared_fired = BTreeSet::new();
        let mut stopped = [false; 2];
        for (_, index, _, mut id) in arrivals {
            if stopped[id.index()] || !self.contact_trigger(index) {
                continue;
            }
            let trigger = &self.triggers[index];
            let personal = matches!(
                trigger.kind,
                TriggerKind::Hurt(_) | TriggerKind::Fall | TriggerKind::Teleport(..)
            ) || matches!(&trigger.kind, TriggerKind::Script(thread)
                if self.levels.iter().any(|s| s.ctl.coop_personal_thread(thread)));
            if !personal && shared_fired.contains(&index) {
                continue;
            }
            let entity = trigger.id;
            let event = Event::Entity(entity, Input::Activate);
            if !personal && self.levels.iter().any(|s|s.ctl.coop_once_trigger(entity))
                && self.event_world.has_fired(&event) {
                continue;
            }
            for slot in &mut self.levels { slot.ctl.coop_trigger_actor(Some(id.index())); }
            let allowed = self.event_world.allowed(&event, &self.event_facts());
            for slot in &mut self.levels { slot.ctl.coop_trigger_actor(None); }
            if !allowed {
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
                contacts.triggers[index].deferred.clear();
            }
            // The shared callback still owns authored consequences. Its old
            // single-body cooldown is copied to the triggering body's slot,
            // then restored so it cannot suppress the partner's contact.
            let old_cooldown = trigger.cooldown;
            for slot in &mut self.levels { slot.ctl.coop_trigger_actor(Some(id.index())); }
            let events = self.dispatch(event);
            for slot in &mut self.levels { slot.ctl.coop_trigger_actor(None); }
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
            if relocated || matches!(flow,ContactFlow::StopPlayer|ContactFlow::RetryPlayer) {
                stopped[id.index()] = true;
                // Keep portal cooldowns: clearing them can create a teleport
                // loop when an authored destination overlaps its own volume.
                contacts.relocate_player(id);
                if flow==ContactFlow::RetryPlayer && relocated {
                    contacts.triggers[index].deferred.push(id);
                }
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
    fn solo_import_keeps_blues_hurt_cooldown_and_reds_first_contact() {
        let (_, mut world, _) = trigger("trigger_hurt");
        world.triggers[0].inside = true;
        world.triggers[0].cooldown = 0.8;
        let mut contacts = Contacts::from_single_player(&world);
        contacts.validate(&world).unwrap();
        let events = world.triggers_pair(0.1, &mut contacts, [motion(100., 100.); 2]).unwrap();
        assert_eq!(damage(&events, PlayerId::Blue), 0.);
        assert!(damage(&events, PlayerId::Red) > 0.);
        assert!(contacts.triggers[0].players[0].inside);
    }

    fn gated_trigger() -> (Interactions, Contacts) {
        let (map, mut world, contacts) = trigger("trigger_once");
        let mut initial = Facts::default();initial.flag("ready", false);
        world.event_world = Runtime::new(Registry::new(&map.entities), vec![Rule {
            key: "trigger/0".into(),event: Event::Entity(Id(0),Input::Activate),
            condition: Condition::flag("ready"),once: true,cooldown: 0.,actions: vec![],
        }], initial, Facts::default()).unwrap();
        (world, contacts)
    }

    #[test]
    fn gated_shared_arrival_survives_pause_and_save_in_either_player_order() {
        for first in PlayerId::ALL {
            let (mut world, mut contacts) = gated_trigger();
            let paths = std::array::from_fn(|i| if i==first.index() {motion(40.,100.)} else {motion(40.,40.)});
            assert!(world.triggers_pair(0.1,&mut contacts,paths).unwrap().activations.is_empty());
            assert_eq!(contacts.triggers[0].deferred,vec![first]);
            let saved=serde_json::to_value(&contacts).unwrap();
            world.triggers_pair(0.,&mut contacts,[motion(100.,100.);2]).unwrap();
            assert_eq!(saved,serde_json::to_value(&contacts).unwrap());
            let paths=std::array::from_fn(|i| if i==first.index() {motion(100.,100.)} else {motion(40.,100.)});
            world.triggers_pair(0.1,&mut contacts,paths).unwrap();
            contacts=serde_json::from_value(serde_json::to_value(&contacts).unwrap()).unwrap();
            contacts.validate(&world).unwrap();
            let snapshot=world.event_world.snapshot();world.event_world.restore(&snapshot).unwrap();
            world.event_world.import_flag("ready",true);
            let events=world.triggers_pair(0.1,&mut contacts,[motion(100.,100.);2]).unwrap();
            assert_eq!(events.activations.len(),1);
            assert_eq!(events.activations[0].player,first);
            assert!(contacts.triggers[0].deferred.is_empty());
            assert!(world.triggers_pair(1.,&mut contacts,[motion(100.,100.);2]).unwrap().activations.is_empty());
            assert_eq!(world.event_world.snapshot().usage["trigger/0"].count,1);
        }
    }

    #[test]
    fn gated_arrival_releases_a_departed_dead_or_relocated_waiter() {
        for first in PlayerId::ALL {for withdrawal in 0..3 {
            let (mut world, mut contacts)=gated_trigger();
            let paths=std::array::from_fn(|i| if i==first.index() {motion(40.,100.)} else {motion(40.,40.)});
            world.triggers_pair(0.1,&mut contacts,paths).unwrap();
            world.triggers_pair(0.1,&mut contacts,[motion(100.,100.);2]).unwrap();
            let paths=std::array::from_fn(|i| if i!=first.index() {motion(100.,100.)}
                else if withdrawal==1 {None} else {motion(100.,40.)});
            if withdrawal==2 {contacts.relocate_player(first);}
            world.triggers_pair(0.1,&mut contacts,paths).unwrap();
            if withdrawal==1 {contacts.reset_player(first);}
            // Returning before the condition opens puts this player behind
            // the partner who kept waiting; it must not restore old priority.
            let paths=std::array::from_fn(|i| if i==first.index() {motion(40.,100.)} else {motion(100.,100.)});
            world.triggers_pair(0.1,&mut contacts,paths).unwrap();
            assert_eq!(contacts.triggers[0].deferred,vec![first.partner(),first]);
            world.event_world.import_flag("ready",true);
            let events=world.triggers_pair(0.1,&mut contacts,[motion(100.,100.);2]).unwrap();
            assert_eq!(events.activations.len(),1);assert_eq!(events.activations[0].player,first.partner());
        }}
    }

    #[test]
    fn gated_same_frame_contacts_keep_swept_order_and_personal_effects_stay_separate() {
        for first in PlayerId::ALL {
            let (mut world,mut contacts)=gated_trigger();
            let paths=std::array::from_fn(|i| if i==first.index() {motion(60.,100.)} else {motion(0.,100.)});
            world.triggers_pair(0.1,&mut contacts,paths).unwrap();
            world.event_world.import_flag("ready",true);
            let events=world.triggers_pair(0.1,&mut contacts,[motion(100.,100.);2]).unwrap();
            assert_eq!(events.activations[0].player,first);
        }
        for class in ["trigger_hurt","trigger_teleport"] {
            let (_,mut world,mut contacts)=trigger(class);
            world.event_world.set_enabled(Id(0),false).unwrap();
            world.triggers_pair(0.1,&mut contacts,[motion(40.,100.);2]).unwrap();
            assert!(contacts.triggers[0].deferred.is_empty());
            world.event_world.set_enabled(Id(0),true).unwrap();
            let events=world.triggers_pair(0.1,&mut contacts,[motion(100.,100.);2]).unwrap();
            assert_eq!(events.activations.len(),2);
            assert_eq!(events.activations[0].player,PlayerId::Blue);
            assert_eq!(events.activations[1].player,PlayerId::Red);
        }
    }

    #[test]
    fn contact_saves_accept_legacy_state_and_reject_invalid_deferred_owners() {
        let (world,contacts)=gated_trigger();
        let old=serde_json::to_value(&contacts).unwrap();
        assert!(old["triggers"][0].get("deferred").is_none());
        let restored:Contacts=serde_json::from_value(old.clone()).unwrap();
        restored.validate(&world).unwrap();assert!(restored.triggers[0].deferred.is_empty());
        let mut bad=old.clone();bad["triggers"][0]["deferred"]=serde_json::json!(["Red","Red"]);
        assert!(serde_json::from_value::<Contacts>(bad).unwrap().validate(&world).is_err());
        let (_,personal,_)=trigger("trigger_hurt");
        let mut bad=old;bad["triggers"][0]["deferred"]=serde_json::json!(["Red"]);
        assert!(serde_json::from_value::<Contacts>(bad).unwrap().validate(&personal).is_err());
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
    fn blocked_personal_portal_retries_after_cooldown_and_restore_until_clear() {
        for id in PlayerId::ALL {
            let (_,mut world,mut contacts)=trigger("trigger_teleport");
            let paths=std::array::from_fn(|i|if i==id.index(){motion(40.,100.)}else{None});
            let first=world.triggers_pair_resolve(0.1,&mut contacts,paths,|_|ContactFlow::RetryPlayer).unwrap();
            assert_eq!(first.activations.len(),1);
            // The pending retry is part of the existing saved contact state.
            contacts=serde_json::from_value(serde_json::to_value(contacts).unwrap()).unwrap();
            contacts.validate(&world).unwrap();
            let standing=std::array::from_fn(|i|if i==id.index(){motion(100.,100.)}else{None});
            let mut attempts=0;
            for tick in 0..21 {
                let events=world.triggers_pair_resolve(0.1,&mut contacts,standing,|_|ContactFlow::RetryPlayer).unwrap();
                if tick<9 {assert!(events.activations.is_empty(),"Portal ignored its cooldown");}
                attempts+=events.activations.len();
            }
            assert_eq!(attempts,2,"Stationary blocked player lost the retry");
            let mut accepted=0;
            for _ in 0..11 {
                accepted+=world.triggers_pair_resolve(0.1,&mut contacts,standing,|_|ContactFlow::StopPlayer).unwrap().activations.len();
            }
            assert_eq!(accepted,1);
            for _ in 0..20 {
                assert!(world.triggers_pair(0.1,&mut contacts,standing).unwrap().activations.is_empty(),
                    "A successful portal retained a pending retry");
            }
            contacts.reset_player(id);
            world.triggers_pair_resolve(0.1,&mut contacts,paths,|_|ContactFlow::RetryPlayer).unwrap();
            let outside=std::array::from_fn(|i|if i==id.index(){motion(0.,0.)}else{None});
            for _ in 0..15 {assert!(world.triggers_pair(0.1,&mut contacts,outside).unwrap().activations.is_empty());}
            assert!(contacts.triggers.iter().all(|t|!t.deferred.contains(&id)));
        }
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
