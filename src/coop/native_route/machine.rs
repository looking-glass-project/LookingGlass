//! Read-only live brush poses for ordinary boarding and riding inputs.
use super::*;
use crate::collision::{Collider, model_shape::Template};

#[derive(Default)]
pub(super) struct Machine { shape: Option<(usize, Template)> }
pub(super) struct Pose { pub center: Vec3, pub rotation: Quat, pub collider: Collider }
impl Machine {
    pub(super) fn sample(&mut self, map: &Bsp, arena: &Arena, entity: usize) -> Result<Pose> {
        let model: usize = map.entities.get(entity).and_then(|e|e.get("model"))
            .and_then(|s|s.strip_prefix('*')).context("Route machine lacks a brush model")?.parse()?;
        if self.shape.as_ref().is_none_or(|(id,_)|*id!=model) {
            self.shape=Some((model,Template::model(map,model)?));
        }
        let (_,center,rotation)=arena.stage.interactions.transforms().into_iter().find(|(id,_,_)|*id==model)
            .context("Route machine lacks a live controller transform")?;
        Ok(Pose{center,rotation,collider:self.shape.as_ref().unwrap().1.at(center,rotation)})
    }
}
impl Pose {
    pub(super) fn supports(&self, player:&crate::movement::Player)->bool {
        let t=self.collider.trace(player.feet+PLAYER_CENTER,
            player.feet+PLAYER_CENTER-Vec3::Z*3.,PLAYER_HALF);
        player.grounded && !t.start_solid && t.fraction<1. && t.normal.z>=0.65
    }
}
