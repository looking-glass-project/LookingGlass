//! One shared encounter clock, personal healing and saved paired attack targets.
use super::*;
use crate::{combat::PlayerTarget, level::CoopPlayer};
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub(super) essence: [usize; 2],
    wait: [f32; 2],
    piece_players: BTreeMap<usize, usize>,
    actor: Option<usize>,
}
impl State {
    pub fn validate(&self, saved: &Saved) -> Result<()> {
        ensure!(self.essence.iter().all(|&i|i<3)
            && self.wait.iter().all(|t|t.is_finite() && (0. ..=1.).contains(t))
            && self.actor.is_none_or(|i|i<2)
            && self.piece_players.len()<=saved.pieces.len()
            && self.piece_players.iter().all(|(id,slot)|*slot<2
                && saved.pieces.iter().any(|(piece,_)|piece==id)),
            "Invalid paired Checkmate state");
        Ok(())
    }
}
impl Encounter {
    fn pair_state(&mut self) -> &mut State {
        self.saved.coop.get_or_insert_with(||State {
            essence:[self.saved.essence;2], wait:[self.saved.essence_wait;2],
            piece_players:Default::default(), actor:None,
        })
    }
    pub(super) fn visible_essence(&self,local:Option<usize>) -> (Vec3,f32) {
        match local.and_then(|slot|self.saved.coop.as_ref().map(|state|(slot,state))) {
            Some((slot,state)) => (self.data.points[&format!("get_me{}",state.essence[slot]+1)].translation
                +Vec3::Z*16.,state.wait[slot]),
            None => (self.essence_position(),self.saved.essence_wait),
        }
    }
    pub(super) fn pair_prepare(&mut self, players:&mut [CoopPlayer<'_>;2]) {
        self.pair_state();
        self.alive=players.iter().any(|p|p.stats.alive());
        if !self.scripted() { self.saved.coop.as_mut().unwrap().actor=None; }
        if self.saved.phase!=Phase::Fight {return;}
        for (slot,p) in players.iter_mut().enumerate() {
            let state=self.saved.coop.as_mut().unwrap();
            let at=self.data.points[&format!("get_me{}",state.essence[slot]+1)].translation+Vec3::Z*16.;
            if p.stats.alive() && state.wait[slot]==0.
                && (p.stats.sanity()<100. || p.stats.will()<100.)
                && (p.player.feet+Vec3::Z*28.-at).abs().cmple(vec3(32.,32.,56.)).all() {
                p.stats.essence(25.);
                state.essence[slot]=(state.essence[slot]+1)%3;
                state.wait[slot]=1.;
            }
        }
    }
    pub(super) fn pair_advance(&mut self,dt:f32,map:&Bsp,world:&mut World,
        players:&mut [CoopPlayer<'_>;2],fixed:&[Collider]) -> Result<()> {
        if !dt.is_finite() || dt<=0. {return Ok(());}
        self.pair_prepare(players);
        // The Stage advances a cinematic once, with only its bound actor.
        if self.scripted() {return Ok(());}
        let Some(actor)=players.iter().position(|p|p.stats.alive()) else {return Ok(());};
        self.advance(dt,map,world,players[actor].player,fixed)?;
        if self.saved.phase==Phase::Fight {
            for wait in &mut self.saved.coop.as_mut().unwrap().wait {*wait=(*wait-dt.min(0.1)).max(0.);}
        }
        Ok(())
    }
    pub(super) fn pair_scene(&mut self,owner:usize,player:&mut Player,stats:&mut Stats) {
        self.pair_state();self.alive=stats.alive();
        if self.saved.time<=crate::movement::FIXED_DT*2.
            && self.saved.coop.as_ref().unwrap().actor!=Some(owner) {
            self.saved.scene_from=Transform{translation:player.feet,
                rotation:Quat::from_rotation_z(player.script_facing)};
        }
        self.saved.coop.as_mut().unwrap().actor=Some(owner);
    }
    pub(super) fn pair_actor(&mut self,players:&[CoopPlayer<'_>;2])->Option<usize> {
        // Start and bind together in Stage's contact phase, after combat damage
        // has been applied. Starting during combat would let the next shared
        // event bind the previous speaker before this hook can choose a survivor.
        if self.saved.phase==Phase::Fight && self.saved.boss.done(&self.data) {
            let actor=(0..2).filter(|&i|players[i].stats.alive()).min_by(|&a,&b|
                players[a].player.feet.distance_squared(self.saved.boss.at)
                    .total_cmp(&players[b].player.feet.distance_squared(self.saved.boss.at)))?;
            self.phase(Phase::Killed);
            self.pair_state().actor=Some(actor);
            self.saved.scene_from=Transform{translation:players[actor].player.feet,
                rotation:Quat::from_rotation_z(players[actor].player.script_facing)};
        }
        self.scripted().then(||self.saved.coop.as_ref().and_then(|c|c.actor))
            .flatten().filter(|&i|players[i].stats.alive())
    }
    pub(super) fn pair_fight(&mut self,dt:f32,world:&World,
        players:&mut [CoopPlayer<'_>;2])->Feedback {
        let mut out=Feedback::default();
        if !dt.is_finite() || dt<=0. || self.scripted() || !players.iter().any(|p|p.stats.alive()) {return out;}
        self.pair_state();
        let targets=players.each_ref().map(|p|PlayerTarget {
            eye:p.player.eye(),alive:p.stats.alive(),hidden:p.stats.ignores_alice(),
        });
        for (i,a) in self.data.ambushes.iter().enumerate() {
            if !self.saved.ambushed.contains(&i) && targets.iter().any(|p|p.alive && a.touches(p.eye)) {
                self.saved.ambushed.insert(i);
                for (id,p) in &mut self.saved.pieces {
                    if a.suppress.contains(id) && !p.active && p.spawn_delay.is_none() {p.suppressed=true;}
                    if a.entities.contains(id) && !p.suppressed && !p.active && p.spawn_delay.is_none() {
                        p.spawn_delay=Some(a.delay+0.1);
                    }
                }
            }
        }
        let state=self.saved.coop.as_mut().unwrap();
        for (id,piece) in &mut self.saved.pieces {
            piece.notarget=false;piece.opponents.summon=None;
            piece.opponents.players=Some(targets);
            piece.opponents.selected_player=state.piece_players.get(id).copied();
            piece.update(dt.min(0.1),world,targets[0].eye,&self.data,&mut out);
            if let Some(slot)=piece.opponents.selected_player {state.piece_players.insert(*id,slot);}
            else {state.piece_players.remove(id);}
        }
        if self.saved.phase==Phase::Fight {
            self.saved.boss.opponents.players=Some(targets);
            self.saved.remainder+=f64::from(dt.min(0.1));
            while self.saved.remainder+1e-9>=1./120. {
                self.saved.remainder=(self.saved.remainder-1./120.).max(0.);
                self.saved.boss.step(world,targets[0].eye,false,self.incoming,&self.data,&mut out);
            }
        }
        out
    }
}
