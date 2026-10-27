//! Shared machinery and an owned, original chess puzzle between scene handoffs.
use super::*;
use crate::level::CoopPlayer;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub actor: Option<usize>,
    pub reunion: bool,
    pub pose: Transform,
    pub previous: [Option<Vec3>; 2],
}
impl State {
    pub fn validate(&self, s: &Saved) -> Result<()> {
        ensure!(self.actor.is_none_or(|i| i < 2)
            && (!(s.board.is_some() || s.pull.is_some() || self.reunion) || self.actor.is_some())
            && (!self.reunion || s.board.is_none())
            && self.pose.translation.is_finite() && self.pose.translation.abs().max_element() < 100000.
            && self.pose.rotation.is_finite() && (self.pose.rotation.length_squared()-1.).abs() < 0.01
            && self.previous.iter().flatten().all(|p| p.is_finite() && p.abs().max_element() < 100000.),
            "Invalid paired chess sequence");
        Ok(())
    }
}
impl Realm {
    pub(super) fn pair_prepare(&mut self, players: &mut [CoopPlayer<'_>; 2]) {
        self.saved.coop.get_or_insert_with(|| State {
            actor: (self.saved.board.is_some() || self.saved.pull.is_some()).then_some(0),
            reunion: false, pose: self.player_pose, previous: [None; 2],
        });
        if let Some(i) = self.saved.coop.as_ref().unwrap().actor {
            if !players[i].stats.alive() {
                if let Some(board) = self.saved.board.take() { self.saved.retry = Some(board.piece); }
                if let Some((lever,_)) = self.saved.pull.take() {
                    self.saved.fired.remove(if lever == 0 {"bell_lever"} else {"water_lever"});
                    self.saved.lever_started[lever as usize] = None;
                }
                let c = self.saved.coop.as_mut().unwrap();
                c.actor = None;
                c.reunion = false;
            } else {
                self.pair_pose(i, players[i].player);
            }
        }
        if let Some(i) = self.coop_focus_player() {
            // A waiting body is absent from this interactive solo sequence,
            // just as it is during the original cutscene.
            players[1-i].stats.scripted_immunity = players[1-i].stats.scripted_immunity.max(0.1);
        }
    }
    pub(super) fn pair_pose(&mut self, i: usize, player: &Player) {
        self.player_pose = Transform { translation: player.feet,
            rotation: Quat::from_rotation_z(player.script_facing) };
        if let Some(c) = &mut self.saved.coop { c.actor = Some(i); c.pose = self.player_pose; }
    }
    pub(super) fn paired_pose(&self) -> Transform {
        self.saved.coop.as_ref().map_or(self.player_pose, |c| c.pose)
    }
    pub(super) fn pair_advance(&mut self, dt: f32, map: &Bsp, world: &mut World,
        players: &mut [CoopPlayer<'_>; 2], fixed: &[Collider]) -> Result<()> {
        if !dt.is_finite() || dt <= 0. { return Ok(()); }
        self.pair_prepare(players);
        if self.scripted() { return Ok(()); }
        let dt = dt.min(0.1);
        self.saved.age += dt;
        let actor = self.saved.coop.as_ref().unwrap().actor;
        if let Some((lever, t)) = &mut self.saved.pull {
            *t += dt;
            if let Some(i) = actor { players[i].player.velocity = Vec3::ZERO; }
            if *t >= self.data.lever_duration {
                let lever = *lever;
                self.saved.pull = None;
                if lever == 0 { self.saved.bell = Some(self.saved.age); }
                else { self.saved.pending = Some(scene::Kind::Water); }
            }
        }
        if self.saved.bell.is_some_and(|t| self.saved.age-t >= 5.) && !self.saved.fired.contains("bell_scene") {
            self.saved.pending = Some(scene::Kind::Bell);
            self.fire("bell_scene");
        }
        {
            let mut living = players.iter_mut().filter(|p| p.stats.alive()).map(|p| &mut *p.player).collect::<Vec<_>>();
            self.move_pair_world(dt, map, world, &mut living, fixed)?;
        }
        self.move_cast(dt);
        if !self.scripted() {
            if let Some(i) = actor.filter(|_| self.saved.board.is_some()) {
                let piece = self.saved.board.as_ref().unwrap().piece;
                self.step_board(dt, players[i].player, world);
                if self.saved.damage > 0. { players[i].stats.damage(std::mem::take(&mut self.saved.damage)); }
                if self.saved.board.is_none() {
                    let completed = if piece == Piece::Bishop {self.saved.bishop_done} else {self.saved.knight_done};
                    self.saved.coop.as_mut().unwrap().reunion = completed && players[i].stats.alive();
                    if !completed { self.saved.coop.as_mut().unwrap().actor = None; }
                }
            }
            let focus = self.coop_focus_player();
            for (i, p) in players.iter_mut().enumerate().filter(|(_,p)| p.stats.alive()) {
                if focus.is_none_or(|owner| owner == i) && self.hazardous(p.player.feet) { p.stats.damage(1000.); }
            }
        }
        if let Some(i) = self.saved.coop.as_ref().unwrap().actor { self.pair_pose(i, players[i].player); }
        self.rebuild(map)?;
        world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        Ok(())
    }
    pub(super) fn pair_contacts(&mut self, dt: f32, players: &mut [CoopPlayer<'_>;2]) -> Option<usize> {
        if !dt.is_finite() || dt <= 0. { return None; }
        self.pair_prepare(players);
        if self.scripted() {
            return self.saved.coop.as_ref().unwrap().actor.filter(|i| players[*i].stats.alive())
                .or_else(|| players.iter().position(|p| p.stats.alive()));
        }
        let previous = self.saved.coop.as_ref().unwrap().previous;
        self.saved.coop.as_mut().unwrap().previous = players.each_ref().map(|p| p.stats.alive().then_some(p.player.feet));
        if let Some(piece) = self.saved.retry.filter(|_| self.saved.board.is_none()) {
            let volume = &self.data.volumes[if piece == Piece::Bishop {"bishop_retry"} else {"cinema_knight_start_trigger"}];
            let first = players.iter().enumerate().filter(|(_,p)| p.stats.alive()).filter_map(|(i,p)| {
                let from = previous[i].unwrap_or(p.player.feet)+PLAYER_CENTER;
                let to = p.player.feet+PLAYER_CENTER;
                volume.touches(from,to,PLAYER_HALF).then(|| (i, volume.trace(from,to,PLAYER_HALF).fraction))
            }).min_by(|a,b| a.1.total_cmp(&b.1)).map(|(i,_)| i);
            if let Some(i) = first {
                self.saved.retry = None;
                self.saved.pending = Some(if piece == Piece::Bishop {scene::Kind::Bishop} else {scene::Kind::Knight});
                self.pair_pose(i, players[i].player);
                return Some(i);
            }
        }
        None
    }
}
