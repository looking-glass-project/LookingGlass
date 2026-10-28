//! One shared machinery clock; original royal scenes still use their single actor.
use super::*;
impl Castle {
    pub(super) fn portal_boarded(&self, feet: Vec3) -> bool {
        if !self.saved.king || self.scripted() { return false; }
        if self.portal_support(&Player::new(feet)) { return true; }
        // The volume bobs with the slab. Both players may be jumping into it.
        let delta = self.portal_pose().translation - self.data.point("exit_portal").translation;
        self.data.volumes.get("exit_trigger").is_some_and(|v| {
            let at = feet + PLAYER_CENTER - delta;
            v.trace(at, at, PLAYER_HALF).start_solid
        })
    }
    pub(super) fn advance_pair(&mut self, dt: f32, map: &Bsp, w: &mut World,
        players: &mut [&mut Player], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. || self.scripted() { return Ok(()); }
        let dt = dt.min(0.1);
        let old_pose = self.portal_pose();
        let phase = self.saved.portal.map(|epoch| self.saved.age - epoch);
        let riders = players.iter().map(|p| p.velocity.z <= 1. && self.portal_support(p)).collect::<Vec<_>>();
        let waiting = riders.iter().any(|r| *r) && !players.iter().all(|p| self.portal_boarded(p.feet));
        self.advance_clock(dt);
        if waiting { self.saved.portal = phase.map(|t| self.saved.age - t); }
        self.doors_pair(dt, map, players)?;
        self.move_cast(dt);
        self.rebuild(map)?;
        let delta = self.portal_pose().translation - old_pose.translation;
        if delta.length_squared() > 0. {
            let feet = players.iter().zip(&riders).map(|(p,rider)| p.feet
                + if *rider { delta } else { Vec3::ZERO }).collect::<Vec<_>>();
            w.set_dynamic(fixed.iter().cloned().chain(self.objects.iter()
                .filter(|o| o.solid && o.id != 23).map(|o| o.collider.clone())).collect());
            let swept = players.iter().zip(&feet).zip(&riders).all(|((p,at),rider)| {
                if !rider { return true; }
                let trace = w.body_trace(p.feet,*at);
                !trace.start_solid && trace.fraction >= 1. && w.body_clear(*at)
            });
            let portal = self.objects.iter().find(|o| o.id == 23).context("Missing Castling portal")?;
            let clear = feet.iter().all(|at| !portal.collider.trace(*at+PLAYER_CENTER,*at+PLAYER_CENTER,PLAYER_HALF).start_solid);
            let separate = feet.iter().enumerate().all(|(i,a)| feet.iter().skip(i+1).all(|b| {
                let d = *a-*b;
                d.truncate().length() >= crate::coop::separation::DISTANCE || d.z.abs() >= PLAYER_HALF.z*2.
            }));
            if swept && clear && separate {
                for ((p,at),rider) in players.iter_mut().zip(feet).zip(riders) {
                    p.feet = at;
                    if rider { p.grounded = true; }
                }
            } else {
                // Restore the shared mover, not just one rider, while other clocks run.
                self.saved.portal = phase.map(|t| self.saved.age - t);
                self.rebuild(map)?;
            }
        }
        w.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        Ok(())
    }
    fn doors_pair(&mut self, dt: f32, map: &Bsp, players: &[&mut Player]) -> Result<()> {
        let before = self.saved.doors.clone();
        self.step_doors_players(dt, &players.iter().map(|p| &**p).collect::<Vec<_>>());
        for o in &self.objects {
            let Some(k) = o.door else { continue; };
            let d = &self.saved.doors[k];
            let travel = d.amount - before[k].amount;
            if travel == 0. { continue; }
            // Sample the swept leaf, including intermediate rotations, so a
            // thin door cannot pass through a body between clear endpoints.
            let radius = (o.bounds.0-o.base.translation).truncate().length()
                .max((o.bounds.1-o.base.translation).truncate().length());
            let distance = if o.slide == Vec3::ZERO { radius*std::f32::consts::FRAC_PI_2 } else { o.slide.length() };
            let steps = (travel.abs()*distance/4.).ceil().max(1.) as usize;
            let mut blocked = false;
            for n in 1..=steps {
                let amount = before[k].amount + travel * n as f32 / steps as f32;
                let translation = o.base.translation + o.slide * amount;
                let rotation = if o.slide == Vec3::ZERO {
                    Quat::from_rotation_z(amount * d.sign * std::f32::consts::FRAC_PI_2)
                } else { o.base.rotation };
                let shape = Collider::model(map, o.model, translation, rotation, true)?;
                if players.iter().any(|p| shape.trace(p.feet+PLAYER_CENTER,p.feet+PLAYER_CENTER,PLAYER_HALF).start_solid) {
                    blocked = true; break;
                }
            }
            if blocked {
                self.saved.doors[k] = before[k].clone();
                if travel < 0. {
                    self.saved.doors[k].open();
                    self.saved.doors[k].hold = o.wait;
                }
            }
        }
        Ok(())
    }
}
