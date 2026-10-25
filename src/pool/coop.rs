//! One freely controlled world clock, with all living leaf passengers.
//! Original single-Alice performances continue through Pool::advance.
use super::*;
use crate::combat::{Feedback, PlayerTarget};

impl Pool {
    pub(crate) fn boarding_ready(&self, thread: &str, feet: &[Vec3]) -> Option<bool> {
        let index = (0..4).find(|i| thread == format!("rideleaf{}start", i + 1))?;
        let leaf = self.objects.iter().find(|o| o.name == format!("rideleaf{}obj", index + 1))?;
        Some(!feet.is_empty() && feet.iter().all(|at| crate::moving::aboard(&leaf.collider, *at)))
    }

    pub(crate) fn advance_players(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [&mut Player], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. { return Ok(()); }
        // Small swept steps preserve bank/underside obstruction checks at low
        // render rates. Holding one leaf must not stop the other world clocks.
        let mut remaining = dt.min(0.1);
        while remaining > 0.000001 {
            let step = remaining.min(crate::movement::FIXED_DT);
            remaining -= step;
            let previous_solid = self.objects.iter().map(|o| self.solid(&o.name)).collect::<Vec<_>>();
            let previous_age = self.state.age;
            self.state.age += step;
            let elapsed = self.state.age - previous_age;
            let turtle_time = self.turtle_path.nodes[self.turtle_path.nodes.len() - 2].time + 0.5;
            if self.state.turtle[0].is_some_and(|t| self.state.age - t >= turtle_time) {
                self.state.drops[1].get_or_insert(self.state.turtle[0].unwrap() + turtle_time);
            }
            for index in 0..self.objects.len() {
                let object = &self.objects[index];
                let next_pose = self.pose(&object.name, object.base);
                let solid = self.solid(&object.name);
                if next_pose == object.pose && solid == previous_solid[index] { continue; }
                let next = Collider::model(map, object.model, next_pose.0, next_pose.1, true)?;
                world.set_dynamic(fixed.iter().cloned()
                    .chain(self.objects.iter().enumerate().filter(|(i, o)|
                        *i != index && self.solid(&o.name)).map(|(_, o)| o.collider.clone()))
                    .chain(self.clips.iter().cloned()).chain(self.rock_colliders()).collect());
                // A train leaf recycled to the start is a new arrival. Check
                // that it does not appear inside a body, but never carry that
                // body across the discontinuity in its looping path.
                let recycled = next_pose.0.distance(object.pose.0) >= 128.;
                let accepted = if recycled || !previous_solid[index] {
                    !solid || players.iter().all(|p| !next.touches(
                        p.feet + PLAYER_CENTER, p.feet + PLAYER_CENTER, PLAYER_HALF))
                } else {
                    crate::moving::carry(world, &object.collider, &next,
                        Transform { translation: object.pose.0, rotation: object.pose.1 },
                        Transform { translation: next_pose.0, rotation: next_pose.1 }, solid, players)
                };
                if accepted {
                    let object = &mut self.objects[index];
                    object.pose = next_pose;
                    object.collider = next;
                } else {
                    *self.state.blocked_time.entry(object.name.clone()).or_default() += elapsed;
                }
            }
        }
        world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        Ok(())
    }

    pub(crate) fn combat_players(&mut self, dt: f32, world: &World,
        players: [PlayerTarget; 2]) -> Feedback {
        let mut out = Feedback::default();
        for ant in &mut self.state.boulders.as_mut().unwrap().ants {
            ant.opponents.players = Some(players);
            ant.opponents.summon = None;
            ant.notarget = false;
            ant.update(dt, world, players[0].eye, &self.cinema, &mut out);
        }
        out
    }
}
