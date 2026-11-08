//! Host-owned native combat. Campaign controllers join this once-per-tick
//! pipeline later; clients only restore presentation, never run enemy AI.
use super::{
    protocol::{Input, Visit},
    world::{Body, Roster},
    PlayerId,
};
use crate::{
    assets::Assets,
    bsp::Bsp,
    character::Character,
    collision::World,
    combat,
    inventory::{self, Catalog, Pickup},
    movement::{Controls, FIXED_DT},
    npc::Npcs,
};
use anyhow::Result;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub stage: super::stage::Snapshot,
    pub npcs: crate::npc::Snapshot,
    pub characters: [crate::character::Snapshot; 2],
}

pub struct Arena {
    pub stage: super::stage::Stage,
    pub characters: [Character; 2],
    pub npcs: Npcs,
    pub pickups: Vec<Pickup>,
    pub catalog: Catalog,
    pub visit_key: String,
    initial: crate::npc::Snapshot,
    checkpoint_characters: Option<[crate::character::Snapshot; 2]>,
    alive: [bool; 2],
    world_audio: crate::audio::world::State,
}
impl Arena {
    pub fn load(assets: &mut Assets, map: &Bsp, world: &mut World, visit: &Visit, staged: bool) -> Result<Self> {
        let stage = super::stage::Stage::load(assets, map, world, visit, staged)?;
        let npcs = Npcs::load(
            assets,
            map,
            &visit.map,
            visit.entry.as_deref(),
            false,
            staged,
        )?;
        let initial = npcs.snapshot();
        let catalog = Catalog::load(assets)?;
        let pickups =
            inventory::pickups_for_visit(map, &visit.map, visit.entry.as_deref(), &catalog);
        let mut world_audio = crate::audio::world::State::default();
        world_audio.update(crate::audio::world::collect(&stage.interactions,&mut Vec::new()));
        Ok(Self {
            stage,
            npcs,
            initial,
            checkpoint_characters: None,
            catalog,
            pickups,
            visit_key: crate::save::visit_key(&visit.map, visit.entry.as_deref()),
            characters: [
                Character::load_for_player(assets, PlayerId::Blue)?,
                Character::load_for_player(assets, PlayerId::Red)?,
            ],
            alive: [true; 2],
            world_audio,
        })
    }
    pub fn reset_characters(&mut self, bodies: &[Body; 2]) {
        for (character, body) in self.characters.iter_mut().zip(bodies) {
            character.reset(&body.player, body.yaw);
        }
        self.alive = bodies.each_ref().map(|p| p.stats.alive());
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            stage: self.stage.snapshot(),
            npcs: self.npcs.snapshot(),
            characters: self.characters.each_ref().map(|c| c.snapshot()),
        }
    }
    pub fn replicate(&mut self, s: &Snapshot, map: &Bsp, world: &mut World) -> Result<()> {
        self.stage.restore(&s.stage, map, world)?;
        self.npcs.replicate(&s.npcs)?;
        for (c, state) in self.characters.iter_mut().zip(&s.characters) {
            c.replicate(state)?;
        }
        self.sync_cast();
        Ok(())
    }
    /// Load into a newly constructed arena before exposing any restored state.
    pub fn restore_checkpoint(&mut self, state: &Snapshot, map: &Bsp, world: &mut World) -> Result<()> {
        self.stage.restore(&state.stage, map, world)?;
        anyhow::ensure!(!self.stage.busy() && !self.stage.interactions.scripted() && !self.stage.story.busy(),
            "Cannot resume a co-op checkpoint during a scene or level transition");
        self.npcs.restore(&state.npcs)?;
        for (character, saved) in self.characters.iter_mut().zip(&state.characters) {
            character.restore(saved)?;
        }
        self.alive = [true; 2];
        self.sync_cast();
        self.remember_checkpoint();
        Ok(())
    }
    pub fn remember_checkpoint(&mut self) {
        self.stage.remember_checkpoint();
        self.initial = self.npcs.snapshot();
        self.checkpoint_characters = Some(self.characters.each_ref().map(|c| c.snapshot()));
    }
    pub fn restore_visit(&mut self, state: &super::campaign::VisitState, visit: &Visit, map: &Bsp, world: &mut World) -> Result<()> {
        state.matches(visit)?;
        self.stage.restore(&state.stage, map, world)?;
        self.npcs.restore(&state.npcs)?;
        self.stage.reentered();
        self.sync_cast();
        self.initial = self.npcs.snapshot();
        Ok(())
    }
    pub fn step(
        &mut self,
        roster: &mut Roster,
        map: &Bsp,
        world: &mut World,
        mut inputs: [(Controls, Input); 2],
    ) -> Result<()> {
        if self.stage.busy() {
            for character in &mut self.characters {
                character.cancel_weapon_action();
                character.dismiss_summon();
            }
            self.stage.scene_step(roster, map, world, &inputs)?;
            self.stage.interactions.activate_enemies();
            self.npcs.activate_levels(&self.stage.interactions.levels);
            self.npcs.level_patrols(&self.stage.interactions.levels);
            self.story_cast(roster, world);
            return self.finish(roster, map, world, true);
        }
        self.stage.conversations(roster, world, &mut inputs, &mut self.npcs);
        if !self.stage.busy() {
            let talking = self.stage.story.busy();
            let advance = talking && inputs.iter().zip(&roster.players)
                .any(|((movement, _), body)| body.stats.alive() && movement.use_pressed);
            if talking {
                // E belongs to this dialogue, never also to a door or ledge.
                for (movement, _) in &mut inputs { movement.use_pressed = false; }
            }
            self.stage.speech_step(roster, world, advance);
        }
        if self.stage.busy() {
            roster.tick += 1;
            return self.finish(roster, map, world, true);
        }
        let world_dt = super::world::world_dt(&roster.players, FIXED_DT);
        // Moving rocks can defeat a released pusher before the ordinary weapon
        // phase. Preserve that live identity for each player's earned reward.
        let environmental_loot_before = self.stage.interactions.pool.as_ref()
            .map(|p| p.loot_sources()).unwrap_or_default();
        // Plate/gate controllers sample the same live cast as solo play. These
        // contacts are derived again after restore, before the shared mover tick.
        self.stage.interactions.companions_for_movers(&self.npcs.companions());
        self.stage.interactions.actors_for_movers(&self.npcs.targets());
        self.stage.before_move(roster, map, world, &inputs, world_dt)?;
        // A delayed shared output may have begun a scene. Its first scene tick
        // belongs to the next fixed step, never a second world update here.
        if self.stage.busy() {
            roster.tick += 1;
            return self.finish(roster, map, world, true);
        }
        if self.stage.story.busy() {
            for character in &mut self.characters {
                character.cancel_weapon_action();
                character.dismiss_summon();
            }
        }
        let held = std::array::from_fn(|i| self.stage.interactions.duchess.as_ref()
            .is_some_and(|d| d.coop_controlled(i))
            || self.stage.interactions.levels.iter().any(|s| s.ctl.coop_controlled(i)));
        for (index, ((character, (movement, input)), body)) in self.characters.iter_mut().zip(&mut inputs).zip(&roster.players).enumerate() {
            for slot in &self.stage.interactions.levels {
                slot.ctl.coop_filter_controls(index, movement);
                if slot.ctl.coop_blocks_weapons(index) {
                    input.primary = false;
                    input.alternate = false;
                    character.cancel_weapon_action();
                }
            }
            if !self.stage.story.busy() {
                movement.use_pressed |= input.rope_use(&body.player, world);
            }
            if character.attack_movement_locked() || character.ice_locked() {
                *movement = Controls::default();
            }
            if held[index] {
                *movement = Controls::default();
                input.primary = false;
                input.alternate = false;
                character.cancel_weapon_action();
            }
        }
        let feet = roster.players.each_ref().map(|b| b.player.feet);
        let movement_audio = roster.players.each_ref().map(|b|
            (b.player.jumps,b.player.landings,b.player.immersion.level));
        roster.move_players_with_locks(world, inputs, held);
        for (body,(jumps,landings,water)) in roster.players.iter().zip(movement_audio) {
            let at = Some(body.player.eye());
            if body.player.jumps != jumps { self.stage.sounds.emit(roster.tick,"sound/character/alice/jump.wav",at,1.,false); }
            if body.player.landings != landings { self.stage.sounds.emit(roster.tick,"sound/character/alice/small_land.wav",at,1.,false); }
            if water == 0 && body.player.immersion.level > 0 { self.stage.sounds.emit(roster.tick,"sound/character/alice/splash1.wav",at,1.,false); }
            if water > 0 && body.player.immersion.level == 0 { self.stage.sounds.emit(roster.tick,"sound/character/alice/leavewater.wav",at,1.,false); }
        }
        self.stage.contacts(roster, world, feet)?;
        if self.stage.busy() { return self.finish(roster, map, world, true); }
        self.sync_pickup_positions();
        for (index, body) in roster.players.iter_mut().enumerate() {
            if body.stats.alive() {
                if let Some(weapon) = inputs[index].1.weapon { body.stats.select(weapon as usize); }
            }
        }
        self.stage.collect_pickups(roster, world, &self.pickups);
        if self.stage.busy() { return self.finish(roster, map, world, true); }
        self.stage.interactions.activate_enemies();
        self.npcs.activate_levels(&self.stage.interactions.levels);
        self.npcs.level_patrols(&self.stage.interactions.levels);
        let mut before = self.npcs.loot_sources();
        before.extend(self.stage.interactions.loot_sources());
        before.extend(environmental_loot_before);
        // One clock for the world. Either player's Watch can stop its enemies;
        // each player's own movement, weapon and resource clocks still run.
        let talking = self.stage.story.busy();
        let world_dt = if talking { 0. } else { super::world::world_dt(&roster.players, FIXED_DT) };
        for index in 0..2 {
            let character = &mut self.characters[index];
            if talking {
                character.cancel_weapon_action();
                character.dismiss_summon();
            }
            let body = &mut roster.players[index];
            if !body.stats.alive() {
                character.die();
            }
            let mut targets = self.npcs.targets();
            if let Some(encounters) = &self.stage.interactions.encounters { targets.extend(encounters.targets()); }
            if let Some(school) = &self.stage.interactions.school2 { targets.extend(school.targets()); }
            if let Some(pool) = &self.stage.interactions.pool { targets.extend(pool.targets()); }
            targets.extend(self.stage.interactions.duchess.as_ref().and_then(|d| d.target()));
            targets.extend(crate::level::targets(&self.stage.interactions.levels));
            targets.extend(self.stage.interactions.shot_targets());
            targets.extend(character.ice_targets());
            if let Some(summon) = character.summon_target() {
                targets.push(summon);
            }
            character.world_time(FIXED_DT, world_dt);
            character.notarget(body.stats.ignores_alice());
            let input = inputs[index].1;
            let click = if talking || !body.stats.alive() {
                None
            } else if input.primary {
                Some(false)
            } else if input.alternate {
                Some(true)
            } else {
                None
            };
            character.update_funded(
                FIXED_DT,
                &body.player,
                body.running,
                crate::weapons::WeaponInput {
                    dice: body.stats.copies(6),
                    selected: body.stats.equipped().unwrap_or(crate::weapons::UNARMED),
                    click,
                    aim: direction(body.yaw, body.pitch),
                    first_person: false,
                },
                &combat::Context {
                    world,
                    targets: &targets,
                },
                Some(&mut body.stats),
            );
            for mut hit in character.take_hits() {
                if character.hit_ice(hit) {
                    continue;
                }
                if hit.id == crate::dice::ALICE {
                    roster.players[index].stats.damage(hit.damage);
                    roster.players[index].player.knockback(hit.knockback * 2.5);
                } else if hit.id == crate::dice::SUMMON {
                    character.hit_summon(hit);
                } else {
                    if hit.kind.is_demon() {
                        hit.kind = hit.kind.means();
                    } else {
                        hit.damage = roster.players[index].stats.attack_damage(hit.damage);
                    }
                    if let Some(owner) = crate::level::hit_owner(&self.stage.interactions.levels, hit.id) {
                        if let Some(sound) = self.stage.interactions.levels[owner].ctl.hit(hit) {
                            let origin = targets.iter().find(|t| t.id == hit.id).map(|t| t.center);
                            self.stage.sounds.emit(roster.tick, sound, origin, 1., false);
                        }
                        continue;
                    }
                    if hit.id == crate::duchess::ID {
                        if let Some(d) = &mut self.stage.interactions.duchess {
                            if let Some(sound) = d.hit(hit.damage) {
                                self.stage.sounds.emit(roster.tick, sound, Some(d.state.feet), 1., false);
                            }
                        }
                        self.stage.accept(Default::default(), PlayerId::ALL[index], roster, world);
                        continue;
                    }
                    self.npcs.hit(hit);
                    if let Some(encounters) = &mut self.stage.interactions.encounters { encounters.hit(hit); }
                    if let Some(pool) = &mut self.stage.interactions.pool {
                        if let Some(Some(sound)) = pool.hit(hit) {
                            let origin = targets.iter().find(|t| t.id == hit.id).map(|t| t.center);
                            self.stage.sounds.emit(roster.tick, sound, origin, 1., false);
                        }
                    }
                    if let Some(school) = &mut self.stage.interactions.school2 {
                        if let Some(sound) = school.hit_attack(hit) {
                            let origin = targets.iter().find(|t|t.id == hit.id).map(|t|t.center);
                            self.stage.sounds.emit(roster.tick, sound, origin, 1., false);
                        }
                    }
                    let events = self.stage.interactions.shoot(hit);
                    self.stage.accept(events, PlayerId::ALL[index], roster, world);
                }
            }
            if self.stage.busy() { return self.finish(roster, map, world, true); }
        }
        let interactive_actor = self.stage.interactive_actor();
        let targets = std::array::from_fn(|index| { let body = &roster.players[index]; combat::PlayerTarget {
            eye: body.player.eye(),
            alive: body.stats.alive(),
            hidden: body.stats.ignores_alice() || interactive_actor.is_some_and(|a| a.index() != index),
        }});
        let mut feedback = self.npcs.update_players(world_dt, world, targets);
        if let Some(encounters) = &mut self.stage.interactions.encounters {
            feedback.append(encounters.update_players(world_dt, world, targets));
        }
        feedback.append(self.stage.school2_step(if talking { FIXED_DT } else { world_dt },
            roster, world, talking));
        if let Some(pool) = &mut self.stage.interactions.pool {
            feedback.append(pool.combat_players(world_dt, world, targets));
        }
        if let Some(d) = &mut self.stage.interactions.duchess {
            if world_dt > 0. {
                if let Some(target) = d.target() {
                    d.threatened(self.characters.iter().any(|c| c.threatens(world, target)));
                }
            }
            let mut players = roster.players.each_mut().map(|b| crate::level::CoopPlayer {
                player: &mut b.player, stats: &mut b.stats,
            });
            feedback.append(d.coop_combat(world_dt, world, &mut players, talking));
        }
        for slot in &mut self.stage.interactions.levels {
            if world_dt > 0. {
                slot.ctl.coop_threats(&|target| self.characters.iter()
                    .any(|c| c.threatens(world, *target)));
            }
            let mut players = roster.players.each_mut().map(|b| crate::level::CoopPlayer {
                player: &mut b.player, stats: &mut b.stats,
            });
            feedback.append(slot.ctl.coop_combat_roster(world_dt, world, &mut players));
        }
        self.stage.sounds.feedback(roster.tick,&feedback);
        for (body, hit) in roster.players.iter_mut().zip(feedback.players) {
            body.stats.damage(hit.damage);
            body.player.knockback(hit.impulse);
            if hit.will_drain > 0. {
                body.stats.spend_will(hit.will_drain.min(body.stats.will()));
            }
        }
        let mut after = self.npcs.loot_sources();
        after.extend(self.stage.interactions.loot_sources());
        for body in &mut roster.players {
            crate::loot::tick(
                &mut body.stats,
                &self.visit_key,
                world_dt,
                &before,
                &after,
                body.player.feet,
                world,
            );
        }
        self.story_cast(roster, world);
        self.finish(roster, map, world, self.stage.busy())
    }
    fn story_cast(&mut self, roster: &Roster, world: &World) {
        let owner = self.stage.speaker();
        self.npcs.story_pose(&self.stage.story, FIXED_DT, world, roster.players[owner.index()].player.eye());
    }
    // The hidden cast is derived presentation, not a saved NPC field. Rebuild
    // it on guests and after a wipe as well as during host simulation.
    fn sync_cast(&mut self) {
        if let Some(village) = &self.stage.interactions.village {
            village.cinema.apply_npcs(&mut self.npcs);
        }
        self.sync_pickup_positions();
    }
    fn sync_pickup_positions(&mut self) {
        self.stage.interactions.sync_pickup_positions(&mut self.pickups);
    }
    fn finish(&mut self, roster: &mut Roster, map: &Bsp, world: &mut World, cinematic: bool) -> Result<()> {
        let generation = roster.checkpoint_generation;
        // Restore collision before validating the saved body placements.
        if roster.players.iter().all(|b| !b.stats.alive()) {
            self.stage.reset(map, world)?;
            self.npcs.restore(&self.initial)?;
        }
        // Keep a pending revival out of a single-character puzzle, just as
        // during its cutscene. The ten-second clock still runs; place the
        // partner at the safe handoff as soon as the sequence releases control.
        roster.finish_tick_with_spawn_hold(world, cinematic, self.stage.interactive_actor().is_some(),
            |at| !self.stage.interactions.hazardous(at))?;
        self.stage.sync_school_return_items(roster);
        if generation != roster.checkpoint_generation {
            self.stage.sounds.clear();
            self.world_audio = crate::audio::world::State::default();
            if let Some(saved) = &self.checkpoint_characters {
                for (character, state) in self.characters.iter_mut().zip(saved) {
                    character.restore(state)?;
                }
                self.alive = [true; 2];
            } else {
                self.reset_characters(&roster.players);
            }
        } else {
            for (index, (character, body)) in
                self.characters.iter_mut().zip(&roster.players).enumerate()
            {
                let alive = body.stats.alive();
                if alive && (!self.alive[index] || body.warped_at == roster.tick) {
                    character.reset(&body.player, body.yaw);
                }
                if alive && !self.alive[index] {
                    self.stage.respawned(PlayerId::ALL[index]);
                }
                if !alive {
                    character.die();
                }
                self.alive[index] = alive;
            }
        }
        self.sync_cast();
        if let Some((time,finished)) = self.stage.interactions.beyond.as_ref().and_then(|b|b.rage_scene()) {
            let owner = self.stage.speaker().index();
            self.characters[owner].rage_scene(time,finished,&roster.players[owner].player);
        }
        for (character,body) in self.characters.iter_mut().zip(&roster.players) {
            for cue in character.take_audio() {
                self.stage.sounds.emit(roster.tick,&cue.path,Some(body.player.eye()),cue.volume,true);
            }
            for (path,origin) in character.take_world_audio() {
                self.stage.sounds.emit(roster.tick,path,Some(origin),1.,false);
            }
        }
        let clocks = crate::audio::world::collect(&self.stage.interactions,&mut Vec::new());
        for (path,origin) in self.world_audio.update(clocks) {
            let origin = (path != "sound/character/alice/death_fall.wav").then_some(origin);
            self.stage.sounds.emit(roster.tick,path,origin,1.,false);
        }
        Ok(())
    }
}

pub fn direction(yaw: f32, pitch: f32) -> Vec3 {
    vec3(
        yaw.cos() * pitch.cos(),
        yaw.sin() * pitch.cos(),
        pitch.sin(),
    )
}
