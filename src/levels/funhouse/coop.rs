//! One shared machinery/battle clock, personal essence and automatic scene ownership.
use super::*;
use crate::{combat::PlayerTarget, level::CoopPlayer, movement::FIXED_DT};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    initialized: [bool;2],
    started: bool,
    ready: [bool;2],
    side: [usize;2],
    wait: [f32;2],
    generation: [u32;2],
    actor: Option<usize>,
    targets: BTreeMap<usize,usize>,
}
impl State {
    pub(super) fn validate(&self, saved:&Saved)->Result<()> {
        ensure!(self.side.iter().all(|i|*i<2)
            && self.wait.iter().all(|t|t.is_finite() && (0. ..=10.).contains(t))
            && self.actor.is_none_or(|i|i<2) && (!self.started || saved.fight || saved.exit.committed)
            && self.targets.len()<=saved.bosses.len()+saved.minis.len()
            && self.targets.iter().all(|(id,slot)|*slot<2
                && saved.bosses.iter().chain(&saved.minis).any(|b|b.id==*id)),
            "Invalid paired Funhouse state");
        Ok(())
    }
}
fn paired_step(boss:&mut battle::Boss, state:&mut State, data:&data::Data, world:&World,
    targets:[PlayerTarget;2], count:usize, out:&mut Feedback)->Option<Transform> {
    boss.opponents.players=Some(targets);
    boss.opponents.selected_player=state.targets.get(&boss.id).copied();
    let spawned=boss.step(data,world,targets[0].eye,false,count,out);
    if let Some(slot)=boss.opponents.selected_player {state.targets.insert(boss.id,slot);}
    else {state.targets.remove(&boss.id);}
    spawned
}
impl Funhouse {
    fn pair_state(&mut self)->&mut State {
        self.saved.coop.get_or_insert_with(||State {
            initialized:[self.saved.initialized;2], started:self.saved.fight,
            ready:[self.saved.essence;2], side:[self.saved.essence_side;2],
            wait:[self.saved.essence_wait;2], generation:[self.saved.essence_generation;2],
            actor:None, targets:BTreeMap::new(),
        })
    }
    pub(super) fn pair_prepare(&mut self, players:&mut [CoopPlayer<'_>;2]) {
        let state=self.pair_state();
        for (slot,p) in players.iter_mut().enumerate() {
            if !state.initialized[slot] {
                if p.stats.alive() {p.stats.full_stats();}
                state.initialized[slot]=true;
            }
        }
        self.saved.initialized=true;
        let state=self.saved.coop.as_mut().unwrap();
        if self.saved.scene.is_none() {state.actor=None;}
        if self.saved.fight && !state.started {
            state.started=true;
            state.ready=[self.saved.essence;2]; state.side=[self.saved.essence_side;2];
            state.wait=[self.saved.essence_wait;2]; state.generation=[self.saved.essence_generation;2];
        }
    }
    pub(super) fn pair_scene(&mut self, owner:usize, stats:&mut Stats) {
        let state=self.pair_state();
        if !state.initialized[owner] && stats.alive() {stats.full_stats();}
        state.initialized[owner]=true; state.actor=Some(owner);
        self.saved.initialized=true;
    }
    pub(super) fn pair_advance(&mut self,dt:f32,map:&Bsp,world:&mut World,
        players:&mut [CoopPlayer<'_>;2],fixed:&[Collider])->Result<()> {
        if !dt.is_finite() || dt<=0. || self.scripted()
            || !players.iter().any(|p|p.stats.alive()) {return Ok(());}
        self.pair_prepare(players);
        let dt=dt.min(0.1);
        {
            let mut living=players.iter_mut().filter(|p|p.stats.alive())
                .map(|p|&mut *p.player).collect::<Vec<_>>();
            self.move_world_players(dt,map,world,&mut living,fixed)?;
        }
        self.saved.exit.advance(dt);
        // Ordinary Jack speech keeps both players exploring. Cinematic scenes
        // are advanced by Stage with their bound actor, never by this path.
        if let Some(actor)=players.iter_mut().find(|p|p.stats.alive()) {
            self.advance_scene(dt,map,world,actor.player)?;
        }
        self.rebuild(map)?;
        world.set_dynamic(fixed.iter().cloned().chain(self.colliders()).collect());
        self.traversal(&mut world.traversal);
        Ok(())
    }
    pub(super) fn pair_actor(&mut self, players:&[CoopPlayer<'_>;2])->Option<usize> {
        // Collection is personal; the first living collector starts the shared
        // ordinary speech on the next contact phase. No cinematic owner/freeze
        // is needed, and the completed speech never replays for the partner.
        if self.saved.scene.is_none() && !self.saved.jack
            && players.iter().any(|p|p.stats.alive() && p.stats.collected.contains(&self.data.jack_key)) {
            self.begin(Kind::Jack);
        }
        if self.saved.scene.is_none() && !self.saved.gas && !self.saved.fight
            && self.saved.all_cells.is_some_and(|t|self.saved.time>=t+1.) {
            // This scene starts automatically after the shared clock puzzle.
            // Select a living participant nearest the gas machinery and bind
            // it in the contact phase before any cinematic world tick.
            let at=self.data.gas_focus;
            let actor=(0..2).filter(|&i|players[i].stats.alive()).min_by(|&a,&b|
                players[a].player.feet.distance_squared(at).total_cmp(&players[b].player.feet.distance_squared(at)))?;
            self.saved.gas_start=Some(self.saved.time);
            self.begin(Kind::Gas);self.pair_state().actor=Some(actor);
        }
        if self.saved.fight && self.saved.scene.is_none() && self.saved.death_wait>=5.
            && self.saved.bosses.iter().all(|b|b.health<=0.) {
            // Bind only after Stage has applied this tick's damage.
            let at=(self.saved.bosses[0].at+self.saved.bosses[1].at)*0.5;
            let actor=(0..2).filter(|&i|players[i].stats.alive()).min_by(|&a,&b|
                players[a].player.feet.distance_squared(at).total_cmp(&players[b].player.feet.distance_squared(at)))?;
            self.saved.minis.clear();
            self.pair_state().targets.retain(|id,_|*id==BASE || *id==BASE+1);
            self.begin(Kind::Hatter);
            self.pair_state().actor=Some(actor);
        }
        self.scripted().then(||self.saved.coop.as_ref().and_then(|s|s.actor)).flatten()
            .filter(|&slot|players[slot].stats.alive())
    }
    pub(super) fn visible_essence(&self, local:Option<usize>)->Option<Vec3> {
        if let Some((slot,state))=local.filter(|&slot|slot<2).and_then(|slot|self.saved.coop.as_ref().map(|s|(slot,s))) {
            if !self.saved.fight || self.saved.floor.is_some() || !state.ready[slot] {return None;}
            return Some(self.data.points[if state.side[slot]==0 {"help_me1"} else {"help_me2"}].translation);
        }
        self.saved.essence.then(||self.essence_position())
    }
    fn pair_healing(&mut self, world:&World, players:&mut [CoopPlayer<'_>;2]) {
        let state=self.saved.coop.as_mut().unwrap();
        for (slot,p) in players.iter_mut().enumerate() {
            if state.wait[slot]>0. {
                state.wait[slot]=(state.wait[slot]-FIXED_DT).max(0.);
                if state.wait[slot]==0. {state.ready[slot]=true;}
            }
            let at=self.data.points[if state.side[slot]==0 {"help_me1"} else {"help_me2"}].translation;
            if p.stats.alive() && state.ready[slot] && (p.player.feet+Vec3::Z*32.-at).length()<44.
                && (p.stats.sanity()<100. || p.stats.will()<100.) {
                let ray=world.sweep(p.player.eye(),at,Vec3::ZERO);
                if !ray.start_solid && ray.fraction>=1. {
                    p.stats.essence(25.); state.ready[slot]=false;state.side[slot]=1-state.side[slot];
                    state.wait[slot]=10.;state.generation[slot]=state.generation[slot].saturating_add(1);
                }
            }
        }
    }
    pub(super) fn pair_fight(&mut self, dt:f32, world:&World, players:&mut [CoopPlayer<'_>;2])->Feedback {
        let mut out=Feedback::default();
        if !dt.is_finite() || dt<=0. || !self.saved.fight || self.saved.scene.is_some()
            || !players.iter().any(|p|p.stats.alive()) {return out;}
        self.pair_prepare(players);
        let targets=players.each_ref().map(|p|PlayerTarget {
            eye:p.player.eye(),alive:p.stats.alive(),hidden:p.stats.ignores_alice(),
        });
        self.saved.step+=dt.min(0.1);
        while self.saved.step+1e-7>=FIXED_DT {
            self.saved.step=(self.saved.step-FIXED_DT).max(0.);
            let state=self.saved.coop.as_mut().unwrap();
            for i in 0..2 {
                let count=self.saved.minis.iter().filter(|b|b.family==i && b.health>0.).count();
                if let Some(p)=paired_step(&mut self.saved.bosses[i],state,&self.data,world,targets,count,&mut out) {
                    if self.saved.serial<89_000 {
                        let id=BASE+100+self.saved.serial;self.saved.serial+=1;
                        self.saved.minis.push(battle::Boss::new(id,i,true,p));
                    }
                }
            }
            for boss in &mut self.saved.minis {
                paired_step(boss,state,&self.data,world,targets,usize::MAX,&mut out);
            }
            self.saved.minis.retain(|b|b.health>0. || b.time<10.);
            state.targets.retain(|id,_|self.saved.bosses.iter().chain(&self.saved.minis).any(|b|b.id==*id));
            self.pair_healing(world,players);
            if self.saved.bosses.iter().all(|b|b.health<=0.) {
                self.saved.death_wait=(self.saved.death_wait+FIXED_DT).min(5.);
            }
        }
        out
    }
}
