//! Local particle presentation driven by the authoritative world's switches and
//! clock. Each window uses its own player's collectible visibility.
use super::{arena::Arena, world::Frame, PlayerId};
use crate::{assets::Assets, bsp::Bsp, collision::World, movement::FIXED_DT, particles::Steam};
use anyhow::Result;
use macroquad::prelude::Vec3;

pub struct Effects {
    pub steam: Steam,
    previous: Option<(u64,u64,u64)>,
}
impl Effects {
    pub fn load(assets: &mut Assets, map: &Bsp) -> Result<Self> {
        Ok(Self { steam: Steam::load(assets,map)?, previous: None })
    }
    pub fn update(&mut self, assets: &mut Assets, map: &Bsp, world: &World,
                  arena: &Arena, frame: &Frame, local: PlayerId, eye: Vec3) -> Result<()> {
        let stamp = (frame.epoch,frame.checkpoint_generation,frame.tick);
        // Between network snapshots the authoritative switches and animation
        // clock have not changed. Particle simulation already receives dt=0;
        // avoid rebuilding those same emitter bindings on every render frame.
        // Camera-dependent drawing and light selection still run every frame.
        if self.previous == Some(stamp) { return Ok(()); }
        if self.previous.is_some_and(|old| old.0 != stamp.0 || old.1 != stamp.1 || old.2 > stamp.2) {
            // A checkpoint can re-enable emitters stopped by the theatre. Old
            // puffs must not follow the players back into the restored world.
            self.steam = Steam::load(assets,map)?;
        }
        let dt = elapsed(self.previous,stamp);
        self.previous = Some(stamp);
        let interactions = &arena.stage.interactions;
        self.steam.animate(frame.tick as f32 * FIXED_DT);
        self.steam.sync(&interactions.event_world);
        self.steam.collected(&arena.pickups,&frame.players[local.index()].stats);
        if interactions.school.as_ref().is_some_and(|s|s.theatre_finished()) { self.steam.stop(); }
        if let Some(v) = &interactions.village { v.place_particles(&mut self.steam); }
        if let Some(p) = &interactions.pandemonium { p.gate_particles(&mut self.steam); }
        interactions.place_level_particles(&mut self.steam);
        let world_dt = super::world::world_dt(&frame.players, dt);
        self.steam.update_clocks(dt,world_dt,eye,world);
        Ok(())
    }
    pub fn lights(&self, arena: &Arena) -> Vec<crate::lighting::Light> {
        let mut lights = self.steam.lights();
        for character in &arena.characters { lights.extend(character.lights()); }
        lights.extend(arena.npcs.lights());
        for slot in &arena.stage.interactions.levels { lights.extend(slot.ctl.lights()); }
        lights
    }
}
fn elapsed(previous: Option<(u64,u64,u64)>, current: (u64,u64,u64)) -> f32 {
    previous.filter(|old|old.0==current.0 && old.1==current.1 && old.2<=current.2)
        .map_or(0.,|old|((current.2-old.2) as f32*FIXED_DT).min(0.05))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unchanged_frames_pause_particles_and_loading_never_replays_old_emissions() {
        assert_eq!(elapsed(None,(1,0,50000)),0.);
        assert_eq!(elapsed(Some((1,0,120)),(1,0,120)),0.);
        assert!((elapsed(Some((1,0,120)),(1,0,123))-0.025).abs()<0.0001);
        assert_eq!(elapsed(Some((1,0,120)),(1,1,121)),0.);
        assert_eq!(elapsed(Some((1,0,120)),(2,0,500)),0.);
        assert_eq!(elapsed(Some((1,0,120)),(1,0,100)),0.);
        assert_eq!(elapsed(Some((1,0,120)),(1,0,500)),0.05);
    }
}
