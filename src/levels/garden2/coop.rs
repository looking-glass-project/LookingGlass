//! One host clock for the seesaw, delayed cast and two-player combat.
use super::*;
use crate::combat::{Feedback, PlayerTarget};

impl Garden {
    pub(super) fn advance_pair(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [&mut Player], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. || self.scripted() { return Ok(()); }
        let dt = dt.min(0.1);
        self.saved.fade = (self.saved.fade-dt).max(0.);
        self.rebuild(map)?;
        self.advance_fulcrum_pair(dt, map, world, players, fixed)?;
        self.advance_spawns(dt);
        Ok(())
    }

    fn advance_fulcrum_pair(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [&mut Player], fixed: &[Collider]) -> Result<()> {
        let object = self.objects.iter().find(|o|o.model==18).context("Missing paired fulcrum")?;
        let from = object.pose.context("Missing paired fulcrum pose")?;
        let previous = object.collider.clone();
        // The same support test owns both weight and carrying. A nearby floor
        // must not make a landed player weigh down the deck underneath it.
        let rest = fixed.iter().cloned().chain(self.objects.iter()
            .filter(|o|o.model!=18 && o.pose.is_some()).map(|o|o.collider.clone())).collect::<Vec<_>>();
        world.set_dynamic(rest);
        let mut weight = Vec2::ZERO;
        let mut count = 0;
        let mut attached = false;
        for p in players.iter() {
            if !crate::moving::supported_by(world, &previous, p.feet, false) { continue; }
            if p.rope.is_some() || p.ledge.is_some() || p.climbing() {
                attached = true;
            } else if p.velocity.z <= 1. {
                let local = from.rotation.inverse()*(p.feet-from.translation);
                weight += vec2(-local.y,local.x)/160.*12.;
                count += 1;
            }
        }
        let old = self.saved.fulcrum;
        let target = if count==0 { Vec2::ZERO } else { weight/count as f32 };
        if !attached {
            self.saved.fulcrum += (target.clamp(Vec2::splat(-12.),Vec2::splat(12.))-old)
                .clamp(Vec2::splat(-2.*dt),Vec2::splat(2.*dt));
        }
        if self.saved.fulcrum != old {
            self.rebuild(map)?;
            let next = self.objects.iter().find(|o|o.model==18).unwrap();
            if !crate::moving::carry(world, &previous, &next.collider, from,
                next.pose.unwrap(), true, players) {
                self.saved.fulcrum = old;
                self.rebuild(map)?;
            }
        }
        world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        Ok(())
    }

    pub(super) fn combat_pair(&mut self, dt: f32, world: &World,
        players: [PlayerTarget; 2]) -> Feedback {
        let mut out = Feedback::default();
        if !dt.is_finite() || dt<=0. || self.scripted() { return out; }
        let dt = dt.min(0.1);
        let eye = players.iter().find(|p|p.alive).map_or(Vec3::ZERO,|p|p.eye);
        self.advance_ambush(dt,world);
        for ant in self.saved.ants.iter_mut().filter(|a|a.enabled) {
            ant.notarget = false;
            ant.opponents.players = Some(players);
            ant.opponents.summon = None;
            ant.update(dt,world,eye,&self.data,&mut out);
        }
        for lady in self.saved.ladies.iter_mut().filter(|l|l.enabled) {
            lady.actor.notarget = false;
            lady.actor.opponents.players = Some(players);
            lady.actor.opponents.summon = None;
            out.append(lady.actor.advance(dt,world,eye,self.data.lady));
        }
        out
    }
}
