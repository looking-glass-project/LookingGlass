//! Normal-start two-body route. No placement, damage, resource or world-state
//! writes: the production arena receives only ordinary movement/combat inputs.
use super::{pandemonium_check::Check, PlayerId};
use crate::{assets::Assets, movement::{Controls, FIXED_DT}, route::Route};
use anyhow::{ensure, Result};
use macroquad::prelude::*;
use std::collections::{BTreeMap, VecDeque};

struct Replay {
    game: Check,
    first: PlayerId,
    owners: BTreeMap<String, PlayerId>,
    reunions: usize,
    together: bool,
}
impl Replay {
    async fn step(&mut self, movement: [Controls; 2]) -> Result<()> {
        let owner = self.game.arena.stage.owner;
        let feet = self.game.roster.players.each_ref().map(|b| b.player.feet);
        self.game.step(movement).await?;
        if let Some(current) = self.game.arena.stage.owner {
            if owner == Some(current) {
                ensure!(self.game.roster.players[current.partner().index()].player.feet == feet[current.partner().index()],
                    "Waiting partner moved during a scene");
            }
            if let Some(scene) = self.game.arena.stage.interactions.pandemonium.as_ref().unwrap().scene_id() {
                if let Some(previous) = self.owners.insert(scene.into(), current) {
                    ensure!(previous == current, "Scene changed owner");
                }
            }
        } else if owner.is_some() && self.game.arena.stage.transition.is_none() {
            self.reunions += 1;
            let bodies = &self.game.roster.players;
            ensure!(bodies[0].warped_at == bodies[1].warped_at
                && bodies[0].player.feet.distance(bodies[1].player.feet) <= 110., "Missing scene reunion");
            self.game.capture(&format!("{:?}-route-reunion-{}", self.first, self.reunions), self.first)?;
        }
        Ok(())
    }
    async fn wait(&mut self, seconds: f32) -> Result<()> {
        for _ in 0..(seconds / FIXED_DT) as usize {
            if self.game.arena.stage.transition.is_some() { break; }
            self.step(Default::default()).await?;
        }
        Ok(())
    }
    async fn go(&mut self, goal: Vec3, navigate: bool) -> Result<()> {
        self.go_until(goal, navigate, |_| false).await
    }
    async fn go_until(&mut self, goal: Vec3, navigate: bool, stop: impl Fn(&Check) -> bool) -> Result<()> {
        self.go_with_height(goal, navigate, true, stop).await
    }
    async fn walk_xy(&mut self, goal: Vec2) -> Result<()> {
        self.go_with_height(goal.extend(0.), false, false, |_| false).await
    }
    async fn go_with_height(&mut self, goal: Vec3, navigate: bool, check_height: bool, stop: impl Fn(&Check) -> bool) -> Result<()> {
        let mut plans: [VecDeque<(Controls, Vec3)>; 2] = Default::default();
        let mut ages = [0; 2];
        let mut lead_arrived = self.together;
        for _ in 0..18_000 {
            if self.game.arena.stage.transition.is_some() || stop(&self.game) { return Ok(()); }
            if self.game.arena.stage.busy() {
                plans.iter_mut().for_each(VecDeque::clear); ages = [0; 2]; lead_arrived = self.together;
                self.step(Default::default()).await?; continue;
            }
            let reached = self.game.roster.players.each_ref().map(|b| {
                let delta = goal - b.player.feet;
                if navigate { delta.length() < 42. && b.player.grounded }
                else { delta.truncate().length() < 8. && (!check_height || delta.z.abs() < 35.) }
            });
            if reached.into_iter().all(|r| r) {
                println!("PASS Pandemonium {:?} route leg {goal:?}, health {:?}", self.first,
                    self.game.roster.players.each_ref().map(|b|b.stats.sanity())); return Ok(());
            }
            lead_arrived |= reached[self.first.index()];
            let mut movement = [Controls::default(); 2];
            let mut planned = [false; 2];
            for id in PlayerId::ALL {
                let i = id.index();
                if reached[i] || (id != self.first && !lead_arrived) { continue; }
                if navigate {
                    if plans[i].is_empty() {
                        let body = &self.game.roster.players[i].player;
                        plans[i] = Route::plan(&self.game.scene.world, body, goal, 22.)
                            .or_else(|_| Route::plan(&self.game.scene.world, body, goal, 40.))
                            .map_err(|states| anyhow::anyhow!("No Pandemonium route for {id:?} from {:?} to {goal:?} ({states} states)", body.feet))?.into();
                        ages[i] = 0;
                    }
                    if let Some((input, _)) = plans[i].front() {
                        movement[i] = Controls { jump: input.jump && ages[i] == 0, ..*input }; planned[i] = true;
                    }
                } else {
                    movement[i].wish = (goal - self.game.roster.players[i].player.feet).truncate().normalize_or_zero();
                }
            }
            self.step(movement).await?;
            for i in 0..2 {
                if planned[i] {
                    ages[i] += 1;
                    if ages[i] == 24 {
                        if let Some((_, expected)) = plans[i].pop_front() {
                            if expected.distance(self.game.roster.players[i].player.feet) > 8. { plans[i].clear(); }
                        }
                        ages[i] = 0;
                    }
                }
            }
        }
        anyhow::bail!("Pandemonium route stalled toward {goal:?}: {:?}", self.game.roster.players.each_ref().map(|b| b.player.feet))
    }
    async fn route(&mut self) -> Result<()> {
        self.wait(0.5).await?;
        self.go(vec3(-3400.,2190.,-32.), false).await?;
        for id in [self.first, self.first.partner()] {
            for tick in 0..400 {
                let body = &self.game.roster.players[id.index()].player;
                if body.rope.is_some() { break; }
                let mut movement = [Controls::default(); 2];
                movement[id.index()] = Controls { wish: (vec2(-3552.,2032.) - body.feet.truncate()).normalize_or_zero(),
                    run: true, jump: tick == 0, use_pressed: body.rope_prompt(&self.game.scene.world).is_some(), ..Default::default() };
                self.step(movement).await?;
            }
            ensure!(self.game.roster.players[id.index()].player.rope.is_some(), "{id:?} did not grab the entrance rope");
        }
        for _ in 0..120 { self.step([Controls { rise: -1., ..Default::default() }; 2]).await?; }
        self.game.capture(&format!("{:?}-route-rope", self.first), self.first)?;
        self.step([Controls { use_pressed: true, ..Default::default() }; 2]).await?;
        for goal in [vec3(-3920.,2300.,-280.),vec3(-4152.,2544.,-280.)] { self.go(goal,true).await?; }
        self.wait(10.).await?;
        for goal in [vec3(-4500.,2560.,-264.),vec3(-5000.,2560.,-240.)] { self.go(goal,false).await?; }
        for goal in [vec3(-5024.,2200.,-160.),vec3(-5024.,2080.,-133.),vec3(-5024.,1840.,-92.),vec3(-5024.,1600.,-25.)] { self.go(goal,true).await?; }
        self.go_until(vec3(-5024.,1368.,32.),false,|g| g.arena.stage.interactions.pandemonium.as_ref().unwrap().state.cart != crate::pandemonium::Cart::Waiting).await?;
        self.wait(65.).await?;
        ensure!(self.game.arena.stage.interactions.pandemonium.as_ref().unwrap().state.rides == 1, "Minecart did not finish");
        self.go(vec3(-3264.,-560.,440.),false).await?;
        self.wait(8.).await?;
        self.go(vec3(-3378.,-1040.,424.),true).await?;
        self.go(vec3(-3378.,-1234.,448.),false).await?;
        self.wait(3.).await?;
        self.go(vec3(-3412.,-1036.,440.),false).await?;
        for id in [598,601,599,600,602,603,604,605,606,607,608,609] {
            let point = crate::interaction::vector(&self.game.scene.map.entities[id]["origin"]).unwrap();
            self.go(point,false).await?;
        }
        self.go_until(vec3(-4312.,-2528.,536.),false,|g| g.arena.stage.interactions.pandemonium.as_ref().unwrap().state.key).await?;
        self.wait(3.).await?;
        ensure!(self.game.arena.stage.interactions.pandemonium.as_ref().unwrap().state.key, "Gnome key was not collected");
        for id in [610,611,612,613,614,615,616,617,618,619] {
            let point = crate::interaction::vector(&self.game.scene.map.entities[id]["origin"]).unwrap();
            self.go(point,false).await?;
        }
        let warped = self.game.roster.players.each_ref().map(|b| b.warped_at);
        for id in [self.first, self.first.partner()] {
            // The first portal arrival can start the return scene. The waiting
            // partner reunites at its end, after more than five seconds.
            for _ in 0..2400 {
                let body = &self.game.roster.players[id.index()];
                if body.warped_at != warped[id.index()] { break; }
                let mut movement = [Controls::default(); 2];
                movement[id.index()].wish = (vec2(-3674.,-2276.) - body.player.feet.truncate()).normalize_or_zero();
                self.step(movement).await?;
            }
            ensure!(self.game.roster.players[id.index()].warped_at != warped[id.index()], "{id:?} missed the return portal");
        }
        self.wait(3.).await?;
        // The return street slopes below the portal landing; match the original
        // route's horizontal goals rather than waiting at an impossible height.
        // Both move through its damaging ground together. Waiting there for an
        // entire second crossing needlessly doubles the first player's exposure.
        self.together = true;
        for goal in [vec2(-3600.,1100.),vec2(-3600.,1700.)] { self.walk_xy(goal).await?; }
        for goal in [vec3(-3800.,1900.,-384.),vec3(-3920.,2300.,-280.)] { self.go(goal,true).await?; }
        self.together = false;
        for goal in [vec3(-3968.,2560.,-232.),vec3(-3968.,2624.,-232.)] { self.go(goal,false).await?; }
        self.go_until(vec3(-3968.,2900.,-232.),false,|g| g.arena.stage.interactions.pandemonium.as_ref().unwrap().state.departure).await?;
        self.wait(30.).await?;
        ensure!(self.game.arena.stage.transition == Some(("fortress1".into(),Some("fortress1_start1".into()))), "Missing authored airship exit");
        ensure!(self.game.roster.players.iter().all(|b| b.stats.copies(1) > 0), "A player missed the Cards pickup");
        ensure!(self.owners.len() == 4 && self.owners.values().all(|&id| id == self.first), "Wrong scene ownership: {:?}", self.owners);
        let proof = serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"first":self.first,"owners":self.owners,
            "reunions":self.reunions,"frame":self.game.roster.frame(1,0),"world":self.game.arena.snapshot()});
        std::fs::write(crate::preferences::path("coop-pandemonium").join(format!("{:?}-route.json",self.first)),serde_json::to_vec_pretty(&proof)?)?;
        println!("PASS normal-start Pandemonium co-op {:?}: {} ticks, {} reunions, health {:?}; single-process route",self.first,
            self.game.roster.tick,self.reunions,self.game.roster.players.each_ref().map(|b|b.stats.sanity()));
        Ok(())
    }
}
pub async fn check(assets: &mut Assets) -> Result<()> {
    for first in PlayerId::ALL {
        let mut game = Check::new(assets)?;
        game.leader = Some(first);
        Replay { game, first, owners:BTreeMap::new(),reunions:0, together:false }.route().await?;
    }
    Ok(())
}
