//! Host-owned shared world, story clock and single-Alice scene ownership.
use super::{
    arena::direction,
    protocol::{Input, Visit},
    world::{Body, Roster},
    PlayerId,
};
use crate::{
    assets::Assets,
    bsp::Bsp,
    collision::World,
    interaction::coop::{ContactFlow, Contacts, Motion},
    interaction::{Events, Interactions},
    movement::{Controls, Player, FIXED_DT},
    story::Story,
};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Notice {
    pub text: String,
    pub until: u64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub world: crate::interaction::Snapshot,
    pub contacts: Contacts,
    pub story: crate::story::Snapshot,
    pub hints: crate::cheshire::Snapshot,
    pub owner: Option<PlayerId>,
    pub last_owner: PlayerId,
    pub transition: Option<(String, Option<String>)>,
    pub notices: [Option<Notice>; 2],
    #[serde(default)]
    pub voice_running: bool,
}

pub struct Stage {
    pub sounds: super::audio::Journal,
    pub voice_running: bool,
    pub interactions: Interactions,
    pub story: Story,
    pub hints: crate::cheshire::Hints,
    pub owner: Option<PlayerId>,
    pub transition: Option<(String, Option<String>)>,
    pub notices: [Option<Notice>; 2],
    contacts: Contacts,
    last_owner: PlayerId,
    initial: Option<Snapshot>,
    // Local input progress is intentionally not checkpointed or replicated.
    skip: crate::cinematic::Skip,
}
impl Stage {
    pub fn load(
        assets: &mut Assets,
        map: &Bsp,
        world: &mut World,
        visit: &Visit,
        staged: bool,
    ) -> Result<Self> {
        crate::save::validate_entry(map, visit.entry.as_deref())?;
        let mut interactions = Interactions::load(map)?;
        interactions.set_entry(assets, map, &visit.map, visit.entry.as_deref())?;
        // The staged native guard replaces the authored encounter cast, as in single player.
        if staged {
            interactions.encounters = None;
        }
        let mut story = Story::load(assets, &visit.map);
        let hints = crate::cheshire::Hints::load(assets, map, &visit.map)?;
        if !staged {
            interactions.entry_story(&mut story);
        }
        interactions.sync_coop(world);
        let owner = controls_scene(&interactions).then_some(PlayerId::Blue);
        let contacts = Contacts::new(&interactions);
        let mut stage = Self {
            sounds: super::audio::Journal::default(),
            voice_running: false,
            interactions,
            story,
            hints,
            owner,
            contacts,
            transition: None,
            notices: [None, None],
            last_owner: PlayerId::Blue,
            initial: None,
            skip: Default::default(),
        };
        stage.initial = Some(stage.snapshot());
        Ok(stage)
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            world: self.interactions.snapshot(),
            contacts: self.contacts.clone(),
            story: self.story.snapshot(),
            hints: self.hints.snapshot(),
            owner: self.owner,
            last_owner: self.last_owner,
            transition: self.transition.clone(),
            notices: self.notices.clone(),
            voice_running: self.voice_running,
        }
    }
    pub fn restore(&mut self, state: &Snapshot, map: &Bsp, world: &mut World) -> Result<()> {
        ensure!(
            state.notices.iter().flatten().all(|n| n.text.len() <= 2048),
            "Invalid co-op world notice"
        );
        if let Some((map, entry)) = &state.transition {
            Visit {
                epoch: 1,
                map: map.clone(),
                entry: entry.clone(),
                difficulty: Default::default(),
                combat_preview: false,
            }
            .validate()?;
        }
        self.interactions.restore(&state.world, map)?;
        state.contacts.validate(&self.interactions)?;
        self.contacts = state.contacts.clone();
        self.hints.restore(&state.hints)?;
        self.story.restore(&state.story, &self.hints)?;
        self.owner = state.owner;
        self.last_owner = state.last_owner;
        self.transition = state.transition.clone();
        self.notices = state.notices.clone();
        self.voice_running = state.voice_running;
        self.skip = Default::default();
        self.interactions.sync_coop(world);
        Ok(())
    }
    pub(super) fn import_single_player(&mut self, level: &crate::save::Level,
        map: &Bsp, world: &mut World, current: bool) -> Result<()> {
        self.interactions.restore(&level.interactions, map)?;
        self.hints.restore(&level.hints)?;
        self.story.restore(&level.story, &self.hints)?;
        self.contacts = if current { Contacts::from_single_player(&self.interactions) }
            else { Contacts::new(&self.interactions) };
        self.owner = controls_scene(&self.interactions).then_some(PlayerId::Blue);
        self.last_owner = PlayerId::Blue;
        self.transition = None;
        self.notices = [None, None];
        self.voice_running = false;
        self.skip = Default::default();
        self.interactions.sync_coop(world);
        self.remember_checkpoint();
        Ok(())
    }
    pub fn reset(&mut self, map: &Bsp, world: &mut World) -> Result<()> {
        self.restore(&self.initial.as_ref().unwrap().clone(), map, world)
    }
    pub fn remember_checkpoint(&mut self) {
        self.initial = Some(self.snapshot());
    }
    /// World progress survives leaving a visit. Re-entering places both players
    /// at its entrance, so old contact sweeps, notices and the departure request
    /// no longer belong to those bodies. The restored world is the new checkpoint.
    pub fn reentered(&mut self) {
        self.contacts = Contacts::new(&self.interactions);
        self.notices = [None, None];
        self.transition = None;
        self.owner = controls_scene(&self.interactions).then_some(self.last_owner);
        self.initial = Some(self.snapshot());
    }
    pub fn busy(&self) -> bool {
        self.owner.is_some() || self.transition.is_some()
    }
    pub fn speaker(&self) -> PlayerId { self.owner.unwrap_or(self.last_owner) }
    pub fn interactive_actor(&self) -> Option<PlayerId> {
        self.interactions.levels.iter().find_map(|s| s.ctl.coop_focus_player())
            .and_then(|i| PlayerId::ALL.get(i).copied())
    }
    pub fn respawned(&mut self, id: PlayerId) {
        self.contacts.reset_player(id);
        if let Some(d) = &mut self.interactions.duchess { d.coop_respawned(id.index()); }
        for slot in &mut self.interactions.levels { slot.ctl.coop_respawned(id.index()); }
    }

    pub fn accept(&mut self, events: Events, id: PlayerId, roster: &mut Roster, world: &World) {
        self.sounds.events(roster.tick, &events);
        apply(
            &events,
            id,
            &mut roster.players,
            world,
            roster.tick,
            &mut self.story,
            &mut self.transition,
            &mut self.notices,
        );
        if self.owner.is_none() && controls_scene(&self.interactions) {
            self.owner = Some(id);
            self.last_owner = id;
        }
        self.bind_registered_scene(roster);
    }
    fn bind_registered_scene(&mut self, roster: &Roster) {
        if let Some(owner) = self.owner {
            let player = &roster.players[owner.index()].player;
            for slot in &mut self.interactions.levels {
                if slot.ctl.scripted() { slot.ctl.coop_scene_bound(owner.index(), player); }
            }
        }
    }
    pub fn sync_school_return_items(&mut self, roster: &mut Roster) {
        if let Some(state) = self.interactions.school.as_mut().and_then(|s| s.return_visit.as_mut()) {
            // A dead teammate retains their inventory. Include it both when
            // supplying a story reward and when consuming every spent copy.
            state.inventory_many(&mut roster.players.each_mut().map(|body| &mut body.stats.school_items));
        }
    }
    pub fn school2_step(&mut self, dt: f32, roster: &mut Roster, world: &mut World,
        paused: bool) -> crate::combat::Feedback {
        let speaker = self.speaker();
        let targets = roster.players.each_ref().map(|b| crate::combat::PlayerTarget {
            eye: b.player.eye(), alive: b.stats.alive(), hidden: b.stats.ignores_alice(),
        });
        let Some(school) = &mut self.interactions.school2 else { return Default::default(); };
        let (events, feedback, collector) = school.update_players(dt, world, targets, speaker.index(), paused);
        // These are shared story ingredients/rewards. Ordinary consumables,
        // health, Will and weapon ownership stay in each player's own Stats.
        for body in &mut roster.players { body.stats.school_items = school.quest.items.clone(); }
        self.accept(events, collector.map_or(speaker, |i| PlayerId::ALL[i]), roster, world);
        self.interactions.sync_coop(world);
        feedback
    }
    /// Ordinary speech keeps exploration running, as in single player. Only a
    /// controller-owned camera/motion scene freezes and later reunites the pair.
    pub fn speech_step(&mut self, roster: &mut Roster, world: &World, advance: bool) {
        let ready = self.interactions.prepare_story(&mut self.story);
        self.voice_running = ready;
        self.story.tick(if ready { FIXED_DT } else { 0. }, advance);
        self.interactions.sync_cinematic_story(&self.story);
        for name in self.story.take_completed() {
            let events = self.interactions.completed_dialogue(&name);
            self.accept(events, self.last_owner, roster, world);
        }
        self.hints.sync(&self.interactions);
        self.transition = self.interactions.take_story_exit(&mut self.story).or(self.transition.take());
        if self.owner.is_none() && controls_scene(&self.interactions) {
            self.owner = Some(self.last_owner);
        }
        self.bind_registered_scene(roster);
    }
    pub fn conversations(&mut self, roster: &mut Roster, world: &World,
        inputs: &mut [(Controls, Input); 2], npcs: &mut crate::npc::Npcs) {
        for id in PlayerId::ALL {
            if self.busy() { break; }
            let body = &roster.players[id.index()];
            let input = &mut inputs[id.index()];
            if !body.stats.alive() || !input.0.use_pressed || body.player.rope_prompt(world).is_some() { continue; }
            let Some(talk) = self.interactions.conversation(world, body.player.eye(),
                direction(input.1.yaw, input.1.pitch), npcs, &self.story) else { continue; };
            if let Some(events) = self.interactions.start_conversation(talk, &mut self.story) {
                self.last_owner = id;
                input.0.use_pressed = false;
                npcs.begin_talk(talk.actor());
                self.accept(events, id, roster, world);
            }
        }
    }
    pub fn collect_pickups(&mut self, roster: &mut Roster, world: &World, pickups: &[crate::inventory::Pickup]) {
        let feet: Vec<_> = roster.players.iter().filter(|b| b.stats.alive()).map(|b| b.player.feet).collect();
        let rage_ready = self.interactions.beyond.as_ref().is_none_or(|b| b.rage_boarding_ready(&feet));
        let mut dice_collector = None;
        for (index, body) in roster.players.iter_mut().enumerate().filter(|(_, b)| b.stats.alive()) {
            let dice_before = body.stats.copies(6);
            for pickup in pickups {
                if pickup.id == "fortress2:37" && !rage_ready {
                    if !body.stats.collected.contains(&pickup.id)
                        && crate::inventory::touches_pickup(body.player.feet, pickup.origin, world) {
                        self.notices[index] = Some(Notice {
                            text: "Waiting for both players to board".into(), until: roster.tick + 120,
                        });
                    }
                } else {
                    crate::inventory::collect(&mut body.stats, std::slice::from_ref(pickup), body.player.feet, world);
                }
            }
            // Match the ordinary viewer's pickup trigger, once for the shared
            // story while keeping each player's weapon collection personal.
            let event = crate::school2::cinema::DICE;
            if body.stats.copies(6) > dice_before && !self.story.has_seen(event)
                && self.story.trigger(event) {
                dice_collector = Some(PlayerId::ALL[index]);
            }
        }
        if let Some(id) = dice_collector {
            if let Some(school) = &mut self.interactions.school2 {
                school.begin_queued_dice(&self.story);
            }
            // Begin and bind together, before the next speech tick can choose
            // an earlier speaker instead of the actual first collector.
            self.accept(Events::default(), id, roster, world);
        }
        self.pickup_scenes(roster, world);
    }
    fn pickup_scenes(&mut self, roster: &mut Roster, world: &World) {
        let feet: Vec<_> = roster.players.iter().filter(|b| b.stats.alive()).map(|b| b.player.feet).collect();
        for id in PlayerId::ALL {
            if self.busy() { break; }
            let body = &roster.players[id.index()];
            if !body.stats.alive() { continue; }
            if self.interactions.duchess.as_mut().is_some_and(|d| d.coop_pickup(&body.stats)) {
                self.sounds.emit(roster.tick, "sound/ambience/special/roomsplit.wav", None, 1., false);
                self.accept(Events::default(), id, roster, world);
                break;
            }
            if let Some(village) = &mut self.interactions.village {
                village.cinema.sync_pickups(&body.stats, &mut self.story);
                self.accept(Events::default(), id, roster, world);
            }
            let body = &mut roster.players[id.index()];
            if self.interactions.beyond.as_mut().is_some_and(|b| {
                if !b.rage_boarding_ready(&feet) || !b.pickup(&body.stats) { return false; }
                b.bind_rage_player(&mut body.player, body.yaw);
                true
            }) {
                self.story.trigger("Fortress2_Rage_Pickup");
                self.accept(Events::default(), id, roster, world);
            }
        }
    }
    fn prepare_registered_players(&mut self, roster: &mut Roster) {
        for slot in &mut self.interactions.levels {
            let mut players = roster.players.each_mut().map(|b| crate::level::CoopPlayer {
                player: &mut b.player, stats: &mut b.stats,
            });
            slot.ctl.coop_prepare(&mut players);
        }
    }
    pub fn before_move(
        &mut self,
        roster: &mut Roster,
        map: &Bsp,
        world: &mut World,
        inputs: &[(Controls, Input); 2],
        dt: f32,
    ) -> Result<()> {
        self.skip = Default::default();
        self.prepare_registered_players(roster);
        self.sync_school_return_items(roster);
        if let Some(d) = &mut self.interactions.duchess {
            let mut players = roster.players.each_mut().map(|b| crate::level::CoopPlayer {
                player: &mut b.player, stats: &mut b.stats,
            });
            d.coop_prepare(&mut players);
        }
        if !roster.players[self.last_owner.index()].stats.alive() {
            if let Some(alive) = PlayerId::ALL
                .into_iter()
                .find(|id| roster.players[id.index()].stats.alive())
            {
                self.last_owner = alive;
            }
        }
        let events = self.interactions.advance_shared(dt);
        self.accept(events, self.last_owner, roster, world);
        if self.busy() {
            return Ok(());
        }
        for id in PlayerId::ALL {
            if roster.players[id.index()].stats.alive() {
                let events = self.interactions.pandemonium_player(world, &roster.players[id.index()].player);
                self.accept(events, id, roster, world);
                if self.busy() { return Ok(()); }
            }
        }
        let interactive_actor = self.interactive_actor();
        let (events, scene_owner) = self.interactions.doors_pair(
            dt,
            map,
            world,
            std::array::from_fn(|i| {
                let body = &roster.players[i];
                body.stats.alive().then_some((
                    &body.player,
                    direction(inputs[i].1.yaw, inputs[i].1.pitch),
                    inputs[i].0.use_pressed && body.player.rope_prompt(world).is_none()
                        && interactive_actor.is_none_or(|actor| actor.index() == i),
                ))
            }),
        )?;
        // The whole pair's input has already run. Bind the actual activating
        // player before accepting either event, including an empty Blue event.
        if let Some(owner) = scene_owner {
            self.owner = Some(owner);
            self.last_owner = owner;
        }
        for (id, events) in PlayerId::ALL.into_iter().zip(events) {
            self.accept(events, id, roster, world);
        }
        if !self.busy() {
            let feet: Vec<_> = roster.players.iter().filter(|b| b.stats.alive())
                .map(|b| b.player.feet).collect();
            if self.interactions.beyond.as_ref().is_some_and(|b| b.waiting_for_partner(&feet)) {
                for (index, _) in roster.players.iter().enumerate().filter(|(_, b)| b.stats.alive()) {
                    self.notices[index] = Some(Notice {
                        text: "Waiting for both players to board".into(), until: roster.tick + 120,
                    });
                }
            }
            let mut bodies = roster.players.each_mut().map(|b| crate::level::CoopPlayer {
                player: &mut b.player, stats: &mut b.stats,
            });
            self.interactions.movers_roster(dt, map, world, &mut bodies)?;
            for slot in &mut self.interactions.levels {
                let mut players = roster.players.each_mut().map(|b| crate::level::CoopPlayer {
                    player: &mut b.player, stats: &mut b.stats,
                });
                if let Some(index) = slot.ctl.coop_contacts(dt, &mut players) {
                    ensure!(index < 2 && players[index].stats.alive(), "Invalid paired scene actor");
                    self.owner = Some(PlayerId::ALL[index]);
                    self.last_owner = PlayerId::ALL[index];
                    break;
                }
            }
            if let Some(pool) = &mut self.interactions.pool {
                let targets = roster.players.iter().enumerate().filter(|(_, b)| b.stats.alive())
                    .map(|(i, b)| crate::combat::Target {
                        id: [crate::combat::COOP_BLUE, crate::combat::COOP_RED][i],
                        center: b.player.feet + crate::collision::PLAYER_CENTER,
                        half: crate::collision::PLAYER_HALF,
                    }).collect::<Vec<_>>();
                let feedback = pool.advance_boulders_for(dt, &targets);
                for (body, hit) in roster.players.iter_mut().zip(feedback.players) {
                    body.stats.damage(hit.damage);
                    body.player.velocity += hit.impulse;
                }
                self.interactions.sync_coop(world);
            }
            if controls_scene(&self.interactions) {
                self.owner = Some(self.last_owner);
            }
        }
        self.bind_registered_scene(roster);
        Ok(())
    }
    pub fn contacts(
        &mut self,
        roster: &mut Roster,
        world: &World,
        before: [Vec3; 2],
    ) -> Result<()> {
        let actor = self.interactive_actor();
        let paths = std::array::from_fn(|i| {
            (roster.players[i].stats.alive() && actor.is_none_or(|a| a.index() == i)).then_some(Motion {
                before: before[i],
                after: roster.players[i].player.feet,
            })
        });
        let mut last = self.last_owner;
        let result = self.interactions.triggers_pair_resolve(
            FIXED_DT,
            &mut self.contacts,
            paths,
            |activation| {
                last = activation.player;
                self.sounds.events(roster.tick, &activation.events);
                apply(
                    &activation.events,
                    activation.player,
                    &mut roster.players,
                    world,
                    roster.tick,
                    &mut self.story,
                    &mut self.transition,
                    &mut self.notices,
                )
            },
        )?;
        self.last_owner = last;
        for id in PlayerId::ALL {
            if result.waiting[id.index()] {
                self.notices[id.index()] = Some(Notice {
                    text: "Waiting for your partner".into(),
                    until: roster.tick + 120,
                });
            }
        }
        self.accept(result.shared, last, roster, world);
        if self.owner.is_none() && controls_scene(&self.interactions) {
            self.owner = Some(last);
        }
        self.bind_registered_scene(roster);
        self.sync_school_return_items(roster);
        Ok(())
    }
    pub fn scene_id(&self) -> Option<&'static str> {
        let i=&self.interactions;
        i.scene_id_after(i.duchess.as_ref().and_then(|d|d.scene_id())
            .or_else(||i.pandemonium.as_ref().and_then(|p|p.scene_id()))
            .or_else(||i.pool.as_ref().and_then(|p|p.scene_id()))
            .or_else(||i.school.as_ref().and_then(|s|s.scene_id()))
            .or_else(||i.school2.as_ref().and_then(|s|s.scene_id()))
            .or_else(||i.beyond.as_ref().and_then(|b|b.scene_id()))
            .or_else(||i.fortress.as_ref().and_then(|f|f.state.cinema.scene_id()))
            .or_else(||i.village.as_ref().and_then(|v|v.cinema.scene_id())))
    }
    /// Original scene code receives exactly one actor. The other body is frozen
    /// and omitted from rendering until both can reunite at supported positions.
    pub fn scene_step(
        &mut self,
        roster: &mut Roster,
        map: &Bsp,
        world: &mut World,
        inputs: &[(Controls, Input); 2],
    ) -> Result<()> {
        self.prepare_registered_players(roster);
        roster.tick += 1;
        let Some(owner) = self.owner else {
            return Ok(());
        };
        let scene = self.scene_id();
        if self.skip.update(scene, roster.players[owner.index()].stats.alive(),
            inputs[owner.index()].1.skip_scene, FIXED_DT)
            && self.interactions.skip_cinematic(map, world,
                &mut roster.players[owner.index()].player, &mut self.story)? {
            for notice in &mut self.notices {
                *notice = Some(Notice { text: "Scene skipped".into(), until: roster.tick + 240 });
            }
        }
        // Scene control supersedes traversal for both actors. Do not resume a
        // grip at its old anchor after the scene moves Alice somewhere else.
        for body in &mut roster.players { body.player.release_rope(); }
        let body = &roster.players[owner.index()];
        let events = self.interactions.update(
            FIXED_DT,
            map,
            world,
            &body.player,
            direction(body.yaw, body.pitch),
            false,
        )?;
        self.accept(events, owner, roster, world);
        self.interactions.advance_school(
            FIXED_DT,
            map,
            world,
            &mut roster.players[owner.index()].player,
        )?;
        for slot in &mut self.interactions.levels {
            let body = &mut roster.players[owner.index()];
            slot.ctl.coop_scene_step(owner.index(), &mut body.player, &mut body.stats);
        }
        if let Some(d) = &mut self.interactions.duchess
            .as_mut().filter(|d| d.state.stage != crate::duchess::Stage::Fighting) {
            let body = &mut roster.players[owner.index()];
            let feedback = d.update(FIXED_DT, world, &mut body.player, &mut body.stats, &mut self.story);
            self.sounds.feedback(roster.tick, &feedback);
        }
        self.sync_school_return_items(roster);
        let feedback = self.school2_step(FIXED_DT, roster, world, true);
        self.sounds.feedback(roster.tick, &feedback);
        self.speech_step(roster, world, inputs[owner.index()].0.use_pressed);
        self.interactions.sync_coop(world);
        if self.transition.is_none() && !controls_scene(&self.interactions) && !self.story.busy() {
            if self.interactions.levels.iter().any(|s| s.ctl.coop_defer_reunion()) {
                // The lesson has ended, but its actor must play the original
                // puzzle before bringing the partner onto the finish platform.
                for id in PlayerId::ALL {
                    self.contacts.reset_player(id);
                    roster.players[id.index()].warped_at = roster.tick;
                }
                self.owner = None;
                return Ok(());
            }
            // A lethal personal scene returns to the ordinary spectator/respawn
            // system. Leave the survivor at their safe waiting position.
            if !roster.players[owner.index()].stats.alive() {
                self.contacts.reset_player(owner);
                self.owner = None;
                return Ok(());
            }
            let partner = owner.partner();
            if roster.players[partner.index()].stats.alive() {
                let mut owner_landing=None;
                let Some(player) =
                    super::placement::reunion_with_partner(world, &roster.players[owner.index()].player,
                        &roster.players[partner.index()].player, |at| {
                        !self.interactions.hazardous(at)
                            && self.interactions.beyond.as_ref().is_none_or(|b| b.reunion_safe(at))
                    }).or_else(|| {
                        let actor=&roster.players[owner.index()].player;
                        let anchor=self.interactions.school2.as_ref()?.coop_reunion_anchor(actor)?;
                        let (mut first,second)=super::placement::scene_pair(world,anchor,
                            |at|!self.interactions.hazardous(at))?;
                        first.script_facing=actor.script_facing;
                        owner_landing=Some(first);
                        Some(second)
                    }).or_else(|| {
                        let actor=&roster.players[owner.index()].player;
                        self.interactions.levels.iter().find_map(|slot| {
                            let (radius,drop)=slot.ctl.coop_reunion_limits(actor)?;
                            super::placement::reunion_with_limits(world,actor,radius,drop,|at| {
                                !self.interactions.hazardous(at)
                                    && self.interactions.beyond.as_ref().is_none_or(|b|b.reunion_safe(at))
                            })
                        })
                    })
                else {
                    return Ok(());
                };
                if let Some(first)=owner_landing {roster.players[owner.index()].player=first;}
                roster.players[partner.index()].player = player;
            }
            let facing = roster.players[owner.index()].player.script_facing;
            for body in &mut roster.players {
                // A grounded dialogue can interrupt a run beside a drop. Do not
                // release its stored momentum after several seconds without
                // movement control. Airborne/scripted motion keeps its physics.
                if body.player.grounded { body.player.velocity = Vec3::ZERO; }
                body.yaw = facing;
                body.pitch = 0.;
                body.running = false;
                body.warped_at = roster.tick;
            }
            for id in PlayerId::ALL {
                self.contacts.reset_player(id);
            }
            self.owner = None;
        }
        Ok(())
    }
    pub fn camera(&self, world: &World) -> Option<crate::cinematic::Camera> {
        if self.owner.is_none() {
            return None;
        }
        let i = &self.interactions;
        i.camera_after(
            i.duchess
                .as_ref()
                .and_then(|s| s.camera(world))
                .or_else(|| i.pandemonium.as_ref().and_then(|s| s.camera()))
                .or_else(|| i.pool.as_ref().and_then(|s| s.camera()))
                .or_else(|| i.school.as_ref().and_then(|s| s.scene_camera()))
                .or_else(|| i.school2.as_ref().and_then(|s| s.scene_camera()))
                .or_else(|| i.beyond.as_ref().and_then(|s| s.scene_camera_in(world)))
                .or_else(|| {
                    i.fortress
                        .as_ref()
                        .and_then(|s| s.cinema.camera(&s.state.cinema))
                })
                .or_else(|| i.village.as_ref().and_then(|s| s.cinema.camera())),
            world,
        )
        .or_else(|| {
            i.school
                .as_ref()
                .and_then(|s| s.return_visit.as_ref())
                .and_then(|s| s.camera())
                .map(|(eye, target, _)| crate::cinematic::Camera::look(eye, target))
        })
    }
    pub fn draw_transforms(&self, world: &World) -> Vec<(usize, Vec3, Quat)> {
        let mut transforms = self.interactions.transforms();
        // The rope mesh already contains its deformed world points. A stable
        // local origin also avoids choosing either player's swing as its owner.
        transforms.retain(|t| !world.traversal.ropes.iter().any(|r| r.model == t.0));
        transforms.extend(world.traversal.ropes.iter().map(|r| (r.model, r.origin, Quat::IDENTITY)));
        transforms
    }
    pub fn rope_hands(&self, world: &World, bodies: &[Body; 2], characters: &[crate::character::Character; 2]) -> Vec<(usize, crate::rope::Grip)> {
        bodies.iter().zip(characters).filter(|(b, _)| b.stats.alive()).filter_map(|(b, character)| {
            let grip = b.player.rope.as_ref()?;
            let rope = world.traversal.rope(grip.id)?;
            let physical = b.player.feet + Vec3::Z * 40.;
            Some((rope.model, crate::rope::Grip { physical,
                hand: character.replica_rope_hand(b.player.feet).unwrap_or(physical) }))
        }).collect()
    }
    pub fn scene_fovy(&self, aspect: f32) -> f32 {
        self.interactions
            .beyond
            .as_ref()
            .and_then(|b| b.scene_fovy(aspect))
            .or_else(|| {
                self.interactions
                    .levels
                    .iter()
                    .find_map(|s| s.ctl.scene_fovy(aspect))
            })
            .unwrap_or(75_f32.to_radians())
    }
    pub fn fade(&self) -> Color {
        if self.owner.is_none() {
            return Color::new(0., 0., 0., 0.);
        }
        let i = &self.interactions;
        i.fade_after(
            i.village
                .as_ref()
                .map(|v| v.cinema.fade())
                .or_else(|| i.fortress.as_ref().map(|f| f.cinema.fade(&f.state.cinema)))
                .or_else(|| i.pandemonium.as_ref().map(|p| (p.fade_color(), p.fade())))
                .or_else(|| i.pool.as_ref().map(|p| p.fade()))
                .or_else(|| i.beyond.as_ref().map(|b| b.scene_fade()))
                .or_else(|| i.school.as_ref().map(|s| s.scene_fade()))
                .or_else(|| i.school2.as_ref().map(|s| s.scene_fade())),
        )
        .map_or(Color::new(0., 0., 0., 0.), |(c, a)| Color { a, ..c })
    }
}

fn controls_scene(interactions: &Interactions) -> bool {
    interactions.scripted()
        || interactions.duchess.as_ref().is_some_and(|d| d.cinematic())
        || interactions.school.as_ref().is_some_and(|s| s.cinematic())
}

fn apply(
    events: &Events,
    id: PlayerId,
    players: &mut [Body; 2],
    world: &World,
    tick: u64,
    story: &mut Story,
    transition: &mut Option<(String, Option<String>)>,
    notices: &mut [Option<Notice>; 2],
) -> ContactFlow {
    let partner=&players[id.partner().index()];
    let occupied=events.teleport.is_some_and(|(feet,_)| {
        let delta=feet-partner.player.feet;
        partner.stats.alive() && delta.truncate().length()<super::separation::DISTANCE
            && delta.z.abs()<crate::collision::PLAYER_HALF.z*2.
    });
    let body = &mut players[id.index()];
    body.stats.damage(events.damage);
    if let Some(text) = &events.message {
        notices[id.index()] = Some(Notice {
            text: text.clone(),
            until: tick + 360,
        });
    }
    if !body.stats.alive() {
        notices[id.index()] = None;
        return ContactFlow::StopPlayer;
    }
    let mut flow = ContactFlow::Continue;
    if let Some((feet, yaw)) = events.teleport {
        flow = ContactFlow::RetryPlayer;
        if occupied {
            // Movement separation ran before contacts. Do not publish an
            // overlapping teleport frame and rely on a later tick to fix it.
            // The contact resolver retains the portal cooldown and rearms this
            // player's contact, so ordinary contact can retry after clearance.
            notices[id.index()]=Some(Notice {
                text:"Waiting for your partner to move clear".into(),until:tick+120,
            });
        } else if world.body_clear(feet) {
            body.player = Player::new(feet);
            body.yaw = yaw;
            body.pitch = 0.;
            body.warped_at = tick;
            flow = ContactFlow::StopPlayer;
            if notices[id.index()].as_ref().is_some_and(|n| matches!(n.text.as_str(),
                "Waiting for your partner to move clear" | "Teleport destination is obstructed")) {
                notices[id.index()]=None;
            }
        } else {
            notices[id.index()] = Some(Notice {
                text: "Teleport destination is obstructed".into(),
                until: tick + 360,
            });
        }
    }
    for event in &events.story {
        story.trigger(event);
    }
    if events.transition.is_some() {
        *transition = events.transition.clone();
    }
    if story.busy() || transition.is_some() {
        flow = ContactFlow::StopWorld;
    }
    flow
}

#[cfg(test)]
mod teleport_tests {
    use super::*;
    use crate::{inventory::Stats, movement::EYE_HEIGHT};

    #[test]
    fn occupied_portal_waits_without_changing_either_player_and_retries_after_clearance() {
        let world=World::fixture(&[(vec3(-1000.,-1000.,-100.),vec3(1000.,1000.,0.))]);
        let destination=vec3(0.,0.,0.03125);
        for id in PlayerId::ALL {
            let mut roster=Roster::new(&world,destination+Vec3::Z*EYE_HEIGHT,0.,Stats::default()).unwrap();
            roster.players[id.index()].player=Player::spawn(&world,vec3(-200.,0.,EYE_HEIGHT+0.03125)).unwrap();
            roster.players[id.index()].warped_at=5;
            roster.players[id.partner().index()].player=Player::spawn(&world,destination+Vec3::Z*EYE_HEIGHT).unwrap();
            roster.players[id.partner().index()].stats.spend_will(17.);
            let before=serde_json::to_value(&roster.players).unwrap();
            let mut story=Story::default();let mut transition=None;let mut notices=[None,None];
            let event=Events {teleport:Some((destination,1.2)),..Default::default()};
            assert!(apply(&event,id,&mut roster.players,&world,20,&mut story,&mut transition,&mut notices)==ContactFlow::RetryPlayer);
            assert_eq!(serde_json::to_value(&roster.players).unwrap(),before,"An occupied portal must not overlap players or change personal state");
            assert!(notices[id.index()].as_ref().is_some_and(|n|n.text.contains("partner")));
            roster.players[id.partner().index()].player=Player::spawn(&world,vec3(80.,0.,EYE_HEIGHT+0.03125)).unwrap();
            let partner=serde_json::to_value(&roster.players[id.partner().index()]).unwrap();
            let resources=serde_json::to_value(&roster.players[id.index()].stats).unwrap();
            apply(&event,id,&mut roster.players,&world,21,&mut story,&mut transition,&mut notices);
            let body=&roster.players[id.index()];
            assert_eq!(body.player.feet,destination);assert_eq!(body.warped_at,21);
            assert_eq!(body.yaw,1.2);assert_eq!(body.pitch,0.);
            assert_eq!(serde_json::to_value(&body.stats).unwrap(),resources);
            assert_eq!(serde_json::to_value(&roster.players[id.partner().index()]).unwrap(),partner);
            assert!(notices[id.index()].is_none());
        }
    }

    #[test]
    fn separate_or_dead_partners_do_not_block_authored_portal_arrivals() {
        let world=World::fixture(&[(vec3(-1000.,-1000.,-100.),vec3(1000.,1000.,0.))]);
        let destination=vec3(0.,0.,0.03125);
        for id in PlayerId::ALL {for (offset,dead) in [(Vec3::X*36.,false),(Vec3::Z*56.,false),(Vec3::ZERO,true)] {
            let mut roster=Roster::new(&world,vec3(-200.,0.,EYE_HEIGHT+0.03125),0.,Stats::default()).unwrap();
            let partner=&mut roster.players[id.partner().index()];
            partner.player=Player::new(destination+offset);
            if dead {partner.stats.damage(1000.);}
            let before=serde_json::to_value(partner).unwrap();
            let mut story=Story::default();let mut transition=None;let mut notices=[None,None];
            let event=Events {teleport:Some((destination,0.4)),..Default::default()};
            apply(&event,id,&mut roster.players,&world,25,&mut story,&mut transition,&mut notices);
            assert_eq!(roster.players[id.index()].player.feet,destination);
            assert_eq!(roster.players[id.index()].warped_at,25);
            assert_eq!(serde_json::to_value(&roster.players[id.partner().index()]).unwrap(),before);
            assert!(notices[id.index()].is_none());
        }}
    }

    #[test]
    fn an_unoccupied_but_solid_portal_destination_remains_blocked() {
        let world=World::fixture(&[(vec3(-1000.,-1000.,-100.),vec3(1000.,1000.,0.)),
            (vec3(-32.,-32.,0.),vec3(32.,32.,100.))]);
        for id in PlayerId::ALL {
            let mut roster=Roster::new(&world,vec3(-200.,0.,EYE_HEIGHT+0.03125),0.,Stats::default()).unwrap();
            let before=serde_json::to_value(&roster.players).unwrap();
            let mut story=Story::default();let mut transition=None;let mut notices=[None,None];
            let event=Events {teleport:Some((vec3(0.,0.,0.03125),1.2)),..Default::default()};
            apply(&event,id,&mut roster.players,&world,20,&mut story,&mut transition,&mut notices);
            assert_eq!(serde_json::to_value(&roster.players).unwrap(),before);
            assert_eq!(notices[id.index()].as_ref().unwrap().text,"Teleport destination is obstructed");
        }
    }
}

/// Authored school scene, with each player owning the real trigger in turn.
/// This is a focused scene fixture, not a traversal or campaign completion claim.
pub fn check(assets: &mut Assets) -> Result<()> {
    super::school_return_check::check(assets)?;
    super::school2_pickup_check::check(assets)?;
    let map = Bsp::parse(&assets.read("maps/skool1.bsp")?)?;
    let visit = Visit {
        epoch: 1,
        map: "skool1".into(),
        entry: None,
        difficulty: Default::default(),
        combat_preview: false,
    };
    for (thread, name, start, top) in crate::school::coop_check::LIFTS {
        for first in PlayerId::ALL {
            let mut world = World::from_bsp(&map)?;
            let mut stage = Stage::load(assets, &map, &mut world, &visit, false)?;
            let aboard = crate::school::coop_check::riders(
                stage.interactions.school.as_ref().unwrap(),
                &world,
                name,
                start,
            )?;
            let (eye, yaw) = crate::interaction::spawn(&map, None);
            let mut roster = Roster::new(
                &world,
                eye,
                yaw,
                crate::inventory::Stats::for_level("skool1", None),
            )?;
            roster.players[first.index()].player = aboard[0].clone();
            let inputs = std::array::from_fn(|_| (Controls::default(), Input::default()));
            let mut highest = [start.z; 2];
            for tick in 0..1800 {
                if tick == 240 {
                    roster.players[first.partner().index()].player = aboard[1].clone();
                }
                stage.before_move(&mut roster, &map, &mut world, &inputs, FIXED_DT)?;
                let before = roster.players.each_ref().map(|b| b.player.feet);
                roster.move_players(&world, inputs);
                stage.contacts(&mut roster, &world, before)?;
                roster.finish_tick_with(&world, false, |at| !stage.interactions.hazardous(at))?;
                ensure!(
                    !stage.busy() && roster.checkpoint_generation == 0,
                    "Ordinary lift unexpectedly started a scene or wipe"
                );
                for (i, b) in roster.players.iter().enumerate() {
                    ensure!(
                        b.stats.alive() && world.body_clear(b.player.feet),
                        "Integrated lift injured/embedded rider {i}"
                    );
                    highest[i] = highest[i].max(b.player.feet.z);
                }
                if tick < 240 {
                    ensure!(
                        roster.players[first.index()].player.feet.distance(start) < 0.1,
                        "Integrated {thread} left without its partner"
                    );
                }
            }
            ensure!(
                highest.iter().all(|h| *h >= top - 1.),
                "Integrated {thread} did not carry both: {highest:?}"
            );
            println!("PASS integrated co-op {name}, first {first:?}: host world clock, boarding notice/contact, ordinary body physics, both reach top");
        }
    }
    for first in PlayerId::ALL {
        let mut world = World::from_bsp(&map)?;
        let mut stage = Stage::load(assets, &map, &mut world, &visit, false)?;
        let mut mirror_world = World::from_bsp(&map)?;
        let mut mirror = Stage::load(assets, &map, &mut mirror_world, &visit, false)?;
        let (eye, yaw) = crate::interaction::spawn(&map, None);
        let mut roster = Roster::new(
            &world,
            eye,
            yaw,
            crate::inventory::Stats::for_level("skool1", None),
        )?;
        ensure!(!stage.busy(), "Unexpected school entry scene in fixture");
        let partner = first.partner().index();
        let waiting = roster.players[partner].player.feet;
        let landing = vec3(-2514., 2368., -480.);
        ensure!(
            world.body_clear(landing),
            "Theatre test contact is obstructed"
        );
        roster.players[first.index()].player = Player::new(landing);
        let before = roster.players.each_ref().map(|b| b.player.feet);
        stage.contacts(&mut roster, &world, before)?;
        ensure!(
            stage.owner == Some(first),
            "The first arriving player did not own the theatre scene"
        );
        let mut count = 0;
        while stage.busy() && count < 120 * 180 {
            stage.scene_step(&mut roster, &map, &mut world, &Default::default())?;
            count += 1;
            if stage.busy() {
                ensure!(
                    roster.players[partner].player.feet == waiting,
                    "Scene moved the waiting partner"
                );
            }
            if count % 120 == 0 || !stage.busy() {
                let wire = super::protocol::HostMessage::Frame {
                    frame: stage.snapshot(),
                };
                let encoded = super::codec::encode(&serde_json::to_vec(&wire)?)?;
                let super::protocol::HostMessage::Frame { frame: state } =
                    serde_json::from_slice::<super::protocol::HostMessage<Snapshot>>(
                        &super::codec::decode(&encoded)?,
                    )?
                else {
                    anyhow::bail!("Scene wire roundtrip changed message kind");
                };
                mirror.restore(&state, &map, &mut mirror_world)?;
                ensure!(
                    mirror.owner == stage.owner && mirror.story.busy() == stage.story.busy(),
                    "Scene replica drifted"
                );
                let camera = stage.camera(&world).map(|c| (c.eye, c.target));
                let copy = mirror.camera(&mirror_world).map(|c| (c.eye, c.target));
                ensure!(camera == copy, "Replicated scene camera diverged");
                ensure!(
                    mirror.interactions.transforms() == stage.interactions.transforms(),
                    "Scene mover replica diverged"
                );
            }
        }
        ensure!(
            !stage.busy() && stage.transition.is_none(),
            "School theatre did not finish in 180 seconds"
        );
        let bodies = &roster.players;
        ensure!(
            bodies.iter().all(|b| world.body_clear(b.player.feet)
                && crate::recovery::supported(&world, b.player.feet)
                && !stage.interactions.hazardous(b.player.feet)),
            "Scene reunion was not safe"
        );
        let separation = bodies[0].player.feet.distance(bodies[1].player.feet);
        ensure!(
            (40. ..=110.).contains(&separation),
            "Scene did not reunite both players nearby"
        );
        ensure!(
            bodies
                .iter()
                .all(|b| b.warped_at == roster.tick && b.spawned_at == 0),
            "Scene reunion reported a death/respawn"
        );
        println!("PASS co-op theatre owner {first:?}: {count} fixed ticks, original camera/story, waiting partner frozen, snapshot replica, safe reunion {separation:.2}");
        // A revisit's wipe must retain the completed scene, not resurrect the
        // fresh world's opening puzzle state. Exercise actual world advancement
        // after re-entry before rolling back to that new checkpoint.
        mirror.restore(&stage.snapshot(), &map, &mut mirror_world)?;
        mirror.reentered();
        let checkpoint = serde_json::to_value(mirror.snapshot())?;
        ensure!(!mirror.busy(), "Cached theatre replayed its finished scene");
        mirror.before_move(
            &mut roster,
            &map,
            &mut mirror_world,
            &Default::default(),
            FIXED_DT,
        )?;
        ensure!(
            serde_json::to_value(mirror.snapshot())? != checkpoint,
            "Revisit checkpoint test did not advance the world"
        );
        mirror.reset(&map, &mut mirror_world)?;
        ensure!(
            serde_json::to_value(mirror.snapshot())? == checkpoint,
            "Revisit wipe discarded completed world/story progress"
        );
        ensure!(
            checkpoint != serde_json::to_value(stage.initial.as_ref().unwrap())?,
            "Revisit checkpoint is indistinguishable from a fresh visit"
        );
        println!("PASS co-op revisit {first:?}: finished theatre remains finished after world advancement and checkpoint rollback");
        stage.reset(&map, &mut world)?;
        for b in &mut roster.players {
            b.stats.damage(10000.);
        }
        roster.finish_tick_with(&world, false, |at| !stage.interactions.hazardous(at))?;
        ensure!(
            roster.checkpoint_generation == 1 && roster.players.iter().all(|b| b.stats.alive()),
            "Scene wipe did not restore both bodies"
        );
        ensure!(
            serde_json::to_value(stage.snapshot())?
                == serde_json::to_value(stage.initial.as_ref().unwrap())?,
            "Wipe did not restore shared story and world"
        );
        println!(
            "PASS co-op theatre wipe {first:?}: shared world/story/contact checkpoint restored"
        );
        // Regression: the recipe scene may be entered during a jump and gives
        // control back at that same airborne position. A revival-sized search
        // cannot reach its floor, and waiting while freezing gravity deadlocks.
        let airborne = vec3(203.63718, 2444.5977, 742.67017);
        ensure!(world.body_clear(airborne), "Airborne recipe contact is obstructed");
        ensure!(super::world::safe_near_with(&world, airborne, |_| true).is_none(),
            "Airborne regression no longer exercises the narrow revival search");
        roster.players[first.index()].player = Player::new(airborne);
        let health = roster.players.each_ref().map(|b| b.stats.sanity());
        stage.owner = Some(first);
        stage.last_owner = first;
        stage.scene_step(&mut roster, &map, &mut world, &Default::default())?;
        ensure!(!stage.busy() && roster.players[first.index()].player.feet == airborne,
            "Finished airborne scene did not release ordinary owner movement");
        let other = &roster.players[first.partner().index()].player;
        ensure!(other.grounded && (other.feet.z-696.).abs() < 1.
            && !stage.interactions.hazardous(other.feet), "Airborne reunion did not find the recipe floor");
        ensure!(roster.players.iter().all(|b| b.warped_at == roster.tick),
            "Airborne reunion did not cut both presentation tracks");
        for _ in 0..120 { roster.move_players(&world, Default::default()); }
        ensure!(roster.players[first.index()].player.grounded
            && roster.players.each_ref().map(|b| b.stats.sanity()) == health,
            "Recipe owner did not land normally after reunion");
        println!("PASS co-op airborne recipe handoff {first:?}: partner on safe floor, owner gravity resumes, no health grant or damage");
        // This ordinary book-scene entry fits under a shelf, but the old
        // placement probe starts one unit higher and intersects its overhang.
        // Run the original book scene: its completion removes the book's old
        // collider, so a fresh entrance world is not a valid handoff fixture.
        let (book, book_world, _, book_story) =
            crate::school::cinema_check::setup(assets, &map, "book_cinematic")?;
        stage.interactions = book;
        stage.story = book_story;
        stage.contacts = Contacts::new(&stage.interactions);
        world = book_world;
        stage.interactions.sync_coop(&mut world);
        let edge = vec3(122.937584, 2510.9956, 696.03125);
        ensure!(world.body_clear(edge) && !world.body_clear(edge + Vec3::Z),
            "Recipe edge no longer exercises the low-clearance handoff");
        roster.players[first.index()].player = Player::spawn(&world, edge + Vec3::Z * crate::movement::EYE_HEIGHT)
            .context("Recipe edge regression is obstructed")?;
        let waiting = vec3(222.30391, 2800.97, 716.03125);
        roster.players[first.partner().index()].player = Player::spawn(&world,
            waiting + Vec3::Z * crate::movement::EYE_HEIGHT).context("Recipe waiting book is obstructed")?;
        stage.owner = Some(first);
        stage.last_owner = first;
        for _ in 0..9000 {
            stage.scene_step(&mut roster, &map, &mut world, &Default::default())?;
            if !stage.busy() { break; }
            ensure!(stage.owner == Some(first)
                && roster.players[first.partner().index()].player.feet.distance(waiting) < 0.1,
                "Recipe scene moved its frozen partner or changed owner");
        }
        ensure!(!stage.busy() && roster.players[first.index()].player.feet.distance(edge) < 0.1,
            "Finished recipe scene did not release its owner at the book edge: busy={} feet={:?}",
            stage.busy(), roster.players[first.index()].player.feet);
        let other = &roster.players[first.partner().index()].player;
        ensure!(other.grounded && (other.feet.z-edge.z).abs() < 1.
            && other.feet.distance(edge) >= 36. && other.feet.distance(edge) <= 110.
            && world.body_clear(other.feet) && !stage.interactions.hazardous(other.feet),
            "Recipe edge reunion did not find separate safe footing");
        ensure!(roster.players.each_ref().map(|b| b.stats.sanity()) == health,
            "Recipe edge reunion changed health");
        ensure!(stage.story.has_seen("book_cinematic")
            && super::placement::near(&world, edge, 32., |at| !stage.interactions.hazardous(at)).is_none(),
            "Recipe handoff must finish the original scene and exercise the old placement failure");
        println!("PASS co-op recipe edge handoff {first:?}: owner retained, partner supported and separate, scene releases");
    }
    Ok(())
}
