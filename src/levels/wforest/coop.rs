//! One shared visit clock with personal healing and a swept Staff arrival.
use super::*;
use crate::level::CoopPlayer;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub essence: [u8;2],
    pub wait: [f32;2],
    pub previous: [Option<Vec3>;2],
}
impl State {
    pub fn validate(&self, returning:bool)->Result<()> {
        ensure!(self.essence.iter().all(|i|*i<2)
            && self.wait.iter().all(|t|t.is_finite() && (0. ..=10.).contains(t))
            && self.previous.iter().flatten().all(|p|p.is_finite() && p.abs().max_element()<100000.)
            && (returning || self.wait==[0.;2] && self.essence==[0;2]),"Invalid paired WForest state");
        Ok(())
    }
}
impl Forest {
    pub(super) fn prepare_pair(&mut self, players:&mut [CoopPlayer<'_>;2]) {
        self.saved.coop.get_or_insert_with(||State{essence:[self.saved.essence;2],wait:[self.saved.essence_delay;2],previous:[None;2]});
        for p in players {
            if p.stats.alive() {
                if !self.saved.initialized {p.stats.full_stats();}
                if self.saved.returning && p.stats.copies(7)==0 {p.stats.apply(PickupKind::Weapon(7),100.);}
                if self.saved.destruction.is_some_and(|t|t<1.5) {p.stats.scripted_immunity=p.stats.scripted_immunity.max(0.1);}
            }
            if self.saved.staff {p.stats.staff_component=true;}
        }
        self.saved.initialized=true;
    }
    pub(super) fn advance_clocks(&mut self,dt:f32) {
        self.saved.clock=(self.saved.clock+dt).min(1e6);
        self.saved.exit.advance(dt);
        self.saved.wall_cooldown=(self.saved.wall_cooldown-dt).max(0.);
        self.saved.essence_delay=(self.saved.essence_delay-dt).max(0.);
        if let Some(c)=&mut self.saved.coop {for t in &mut c.wait {*t=(*t-dt).max(0.);}}
        if let Some(t)=&mut self.saved.blunder {*t=(*t+dt).min(3600.);}
    }
    pub(super) fn contacts_pair(&mut self, players:&mut [CoopPlayer<'_>;2])->Option<usize> {
        self.prepare_pair(players);
        if self.scripted() {self.saved.coop.as_mut().unwrap().previous=[None;2];return None;}
        let previous=self.saved.coop.as_ref().unwrap().previous;
        self.saved.coop.as_mut().unwrap().previous=players.each_ref().map(|p|p.stats.alive().then_some(p.player.feet));
        if self.altar() {
            let at=self.data.at("first_eyestaff").translation;
            let trigger=Collider::box_bounds(at-vec3(10.,10.,0.),at+vec3(10.,10.,75.));
            let first=players.iter().enumerate().filter(|(_,p)|p.stats.alive()).filter_map(|(i,p)| {
                let from=previous[i].unwrap_or(p.player.feet)+PLAYER_CENTER;
                let to=p.player.feet+PLAYER_CENTER;
                trigger.touches(from,to,PLAYER_HALF).then(||(i,trigger.trace(from,to,PLAYER_HALF).fraction))
            }).min_by(|a,b|a.1.total_cmp(&b.1)).map(|(i,_)|i);
            if let Some(i)=first {
                self.saved.staff=true;self.saved.armed=true;
                for p in players {p.stats.staff_component=true;}
                self.begin(Kind::Staff);
                self.saved.coop.as_mut().unwrap().previous=[None;2];
                return Some(i);
            }
        }
        if self.saved.returning {
            for (i,p) in players.iter_mut().enumerate().filter(|(_,p)|p.stats.alive()) {
                if self.saved.blunder.is_none() && p.stats.collected.contains("wforest:73") {self.saved.blunder=Some(0.);}
                let c=self.saved.coop.as_mut().unwrap();
                let at=self.data.at(if c.essence[i]==0 {"get_me1"} else {"get_me2"}).translation;
                if c.wait[i]==0. && (p.player.feet+PLAYER_CENTER).distance(at)<42.
                    && (p.stats.sanity()<100. || p.stats.will()<100.) {
                    p.stats.essence(25.);c.essence[i]^=1;c.wait[i]=10.;
                }
            }
        }
        None
    }
    pub(super) fn personal_essence(&self,local:Option<usize>)->Option<Vec3> {
        if !self.saved.returning {return None;}
        let (index,wait)=local.filter(|i|*i<2).and_then(|i|self.saved.coop.as_ref().map(|c|(c.essence[i],c.wait[i])))
            .unwrap_or((self.saved.essence,self.saved.essence_delay));
        (wait==0.).then(||self.data.at(if index==0 {"get_me1"} else {"get_me2"}).translation)
    }
}
