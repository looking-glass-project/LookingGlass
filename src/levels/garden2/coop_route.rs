//! Ordinary controls through the production Arena; failures retain state and input history.
//! The authored single-player route supplies waypoints, never gameplay mutations.
use super::*;
use crate::{
    coop::{arena::Arena, protocol::{Input, Visit}, world::{Body, Roster}, PlayerId},
    inventory::Stats,
    movement::{Controls, FIXED_DT},
    render::Scene as RenderScene,
    route::{fight, Route},
};
use std::collections::VecDeque;

#[derive(Deserialize)]
struct Step {
    pos: [f32; 3],
    #[serde(default)]
    mode: String,
    #[serde(default)]
    frames: usize,
    #[serde(default)]
    save: Option<String>,
    #[serde(default)]
    paired_mode: Option<String>,
}
#[derive(Default)]
struct Driver {
    index: usize,
    ticks: usize,
    quiet: usize,
    jumped: bool,
    last: Vec3,
    plan: VecDeque<(Controls, Vec3)>,
    plan_tick: usize,
    fight: fight::Tracker,
}
impl Driver {
    fn advance(&mut self, feet: Vec3) {
        self.index += 1;
        self.reset(feet);
    }
    fn reset(&mut self, feet: Vec3) {
        self.ticks = 0;
        self.quiet = 0;
        self.jumped = false;
        self.last = feet;
        self.plan.clear();
        self.plan_tick = 0;
    }
    fn sample(&mut self, steps: &[Step], arena: &Arena, world: &World,
        body: &Body, id: PlayerId, waiting: bool) -> Result<(Controls, Input)> {
        let p = &body.player;
        let mut control = Controls::default();
        let mut input = Input {yaw:body.yaw, pitch:body.pitch, ..Default::default()};
        let mut targets = arena.npcs.targets();
        targets.extend(arena.stage.interactions.levels_targets());
        self.fight.observe(&targets, FIXED_DT);
        let context = crate::combat::Context {world, targets:&targets};
        let attack = fight::plan(&context, p.eye(), &targets, &self.fight,
            body.stats.copies(1)>0 && body.stats.will()>=fight::CARD_COST);
        let locked = arena.characters[id.index()].attack_movement_locked();
        if attack.target.is_some() && p.rope.is_none() {
            input.yaw = attack.aim.y.atan2(attack.aim.x);
            input.pitch = attack.aim.z.asin().clamp(-1.56,1.56);
            if !locked {
                input.weapon = Some(attack.weapon as u8);
                input.primary = !attack.alternate;
                input.alternate = attack.alternate;
            }
            self.fight.weapon = attack.weapon;
            self.fight.last_aim = attack.aim;
        }
        if locked || waiting { return Ok((control,input)); }
        loop {
            let Some(s) = steps.get(self.index) else { return Ok((control,input)); };
            let goal = Vec3::from_array(s.pos);
            let delta = goal-p.feet;
            let d = delta.truncate();
            ensure!(self.ticks<120*90,
                "Herbaceous {id:?} stalled at step{} {} goal{goal:?}, feet{:?}",self.index,s.mode,p.feet);
            let done = match s.mode.as_str() {
                "input" | "inputjump" | "release" => self.ticks>=s.frames,
                "wait" => self.ticks>=(goal.x*120.) as usize,
                "clear" => self.quiet>=120,
                "rope" => p.rope.is_some() && delta.z.abs()<10.,
                "grab" if p.rope.is_some() => true,
                "rise" if d.length()<35. && p.feet.z>=goal.z && p.velocity.z>0. => true,
                "vent" if d.length()<60. && (p.updraft_time>0. || p.knockback_time>0.)
                    && p.velocity.z>100. && p.feet.z>goal.z-70. => true,
                _ => d.length()<28. && delta.z.abs()<90. && (p.grounded || p.immersion.level>0),
            };
            if done { self.advance(p.feet); continue; }
            match s.mode.as_str() {
                "clear" => {
                    let visible = targets.iter().any(|t|t.center.distance(p.eye())<goal.x
                        && fight::aim_point(&context,p.eye(),t).is_some());
                    self.quiet = if visible {0} else {self.quiet+1};
                }
                "wait" => (),
                "input" | "inputjump" | "release" => {
                    control = Controls {wish:goal.truncate(),rise:goal.z,run:true,
                        jump:s.mode=="inputjump" && self.ticks==0,
                        use_pressed:s.mode=="release" && self.ticks==0,..Default::default()};
                }
                "rope" => {
                    control.rise = delta.z.signum();
                    control.use_pressed = self.ticks==0 && p.rope.is_none();
                }
                "nav" => {
                    if self.plan.is_empty() {
                        self.plan = Route::plan(world,p,goal,28.)
                            .map_err(|n|anyhow::anyhow!("Herbaceous route search exhausted {n} nodes"))?.into();
                        self.plan_tick = 0;
                    }
                    if let Some((next,_)) = self.plan.front() {
                        control = *next;
                        control.jump &= self.plan_tick==0;
                        self.plan_tick += 1;
                        if self.plan_tick==24 { self.plan.pop_front(); self.plan_tick=0; }
                    }
                }
                _ => {
                    let stuck = self.ticks%120==119 && p.feet.distance(self.last)<18.;
                    if self.ticks%120==119 { self.last=p.feet; }
                    let jump = (matches!(s.mode.as_str(),"jump"|"vent"|"leap") && !self.jumped
                        && (p.grounded || s.mode=="leap" && p.rope.is_some())) || stuck;
                    self.jumped |= jump;
                    control = Controls {
                        wish:if s.mode=="rise" {(d/100.).clamp_length_max(1.)} else {d.normalize_or_zero()},
                        swim:d.normalize_or_zero().extend(0.),
                        rise:if p.immersion.level>0 || s.mode=="leap" && self.ticks==0 {1.} else {0.},
                        run:true,jump,
                        use_pressed:s.mode=="grab" || s.mode=="leap" && self.ticks==0,
                    };
                }
            }
            self.ticks += 1;
            // Do not start an attack wind-up in the middle of a timed leap or grip.
            if matches!(s.mode.as_str(),"jump"|"vent"|"rise"|"grab"|"rope"|"leap"|"input"|"inputjump"|"release") {
                input.weapon=None; input.primary=false; input.alternate=false;
            }
            return Ok((control,input));
        }
    }
}
fn garden(arena: &Arena) -> &Garden {
    arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Garden>()).unwrap()
}
fn record(arena: &Arena, roster: &Roster, drivers: &[Driver;2], first: PlayerId,
    skip: bool, samples: usize, history: &VecDeque<serde_json::Value>, suffix: &str) -> Result<()> {
    let report=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"first":first,"skip":skip,
        "samples":samples,"full_route_passed":suffix=="passed", "steps":drivers.each_ref().map(|d|d.index),
        "step_ticks":drivers.each_ref().map(|d|d.ticks), "frame":roster.frame(1,0),
        "world":arena.snapshot(),"recent_inputs":history});
    std::fs::write(crate::preferences::path(&format!("garden2-{first:?}-{skip}-{suffix}.json")),
        serde_json::to_vec(&report)?)?;
    Ok(())
}
async fn route(a: &mut Assets, first: PlayerId, skip: bool) -> Result<()> {
    let recovery_mode=std::env::var("LOOKING_GLASS_GARDEN2_COOP_RECOVERY").unwrap_or_else(|_|"strict".into());
    ensure!(matches!(recovery_mode.as_str(),"strict"|"allowed"),"Invalid Herbaceous recovery mode");
    let allow_revival=recovery_mode=="allowed";
    let visit=Visit {epoch:1,map:"garden2".into(),entry:Some("garden2_start1".into()),
        difficulty:Default::default(),combat_preview:false};
    let mut scene=RenderScene::load_for(a,&visit.map,visit.difficulty)?;
    let mut arena=Arena::load(a,&scene.map,&mut scene.world,&visit,false)?;
    let (eye,yaw)=crate::interaction::spawn(&scene.map,visit.entry.as_deref());
    let mut roster=Roster::new_with(&scene.world,eye,yaw,Stats::for_level(&visit.map,visit.entry.as_deref()),
        |at| !arena.stage.interactions.hazardous(at))?;
    arena.reset_characters(&roster.players);
    let mut replica_world=World::from_bsp(&scene.map)?;
    let mut replica=Arena::load(a,&scene.map,&mut replica_world,&visit,false)?;
    let course=if let Ok(path)=std::env::var("LOOKING_GLASS_GARDEN2_COOP_ROUTE") {
        std::fs::read(path).context("Cannot read Herbaceous diagnostic course")?
    } else {include_bytes!("route_steps.json").to_vec()};
    let mut steps:Vec<Step>=serde_json::from_slice(&course)?;
    for step in &mut steps {
        if let Some(mode)=step.paired_mode.take() { step.mode=mode; }
    }
    ensure!(!steps.is_empty() && steps.iter().all(|s|s.pos.iter().all(|v|v.is_finite())
        && s.frames<=120*90),"Invalid Herbaceous diagnostic course");
    std::fs::write(crate::preferences::path("garden2-coop-course.json"),course)?;
    let mut drivers:[Driver;2]=Default::default();
    let mut history=VecDeque::new();
    let mut samples=0;
    let mut free_ticks=0;
    let mut reunions=0;
    let mut rejoins=Vec::new();
    let mut arrival=None;
    let mut trace=crate::coop::recovery_trace::Trace::new(crate::preferences::path(
        &format!("garden2-{first:?}-{skip}-recovery.jsonl")))?;
    trace.record(&roster,&scene.world,1,"garden2",arena.stage.busy(),false,
        |at|!arena.stage.interactions.hazardous(at))?;
    let result:Result<()> = async {
        for tick in 0..120*900 {
            let before=roster.players.each_ref().map(|b|b.player.feet);
            let old_owner=arena.stage.owner;
            let old_warps=roster.players.each_ref().map(|b|b.warped_at);
            let old_spawns=roster.players.each_ref().map(|b|b.spawned_at);
            let busy=arena.stage.busy();
            let indices=drivers.each_ref().map(|d|d.index);
            let mut inputs=std::array::from_fn(|_|(Controls::default(),Input::default()));
            if !arena.stage.busy() {
                for id in PlayerId::ALL {
                    if !roster.players[id.index()].stats.alive() { continue; }
                    let regroup=allow_revival && !roster.players[id.partner().index()].stats.alive()
                        && roster.players[id.index()].player.grounded;
                    inputs[id.index()]=drivers[id.index()].sample(&steps,&arena,&scene.world,
                        &roster.players[id.index()],id,regroup || id!=first && free_ticks<240)?;
                }
                free_ticks+=1;
            } else if skip {
                let id=arena.stage.owner.context("Herbaceous scene has no owner")?;
                arena.stage.interactions.skip_cinematic(&scene.map,&mut scene.world,
                    &mut roster.players[id.index()].player,&mut arena.stage.story)?;
            }
            history.push_back(serde_json::json!({"tick":roster.tick,"feet":before,
                "steps":drivers.each_ref().map(|d|d.index),"inputs":inputs.each_ref().map(|(c,i)|
                    serde_json::json!({"wish":c.wish,"swim":c.swim,"rise":c.rise,"jump":c.jump,
                        "run":c.run,"use_pressed":c.use_pressed,"keys":i}))}));
            if history.len()>240 { history.pop_front(); }
            arena.step(&mut roster,&scene.map,&mut scene.world,inputs)?;
            trace.record(&roster,&scene.world,1,"garden2",busy||arena.stage.busy(),
                arena.stage.interactive_actor().is_some(),|at|!arena.stage.interactions.hazardous(at))?;
            if allow_revival && roster.checkpoint_generation==0 {
                for i in 0..2 {
                    if !roster.players[i].stats.alive() { drivers[i].reset(roster.players[i].player.feet); }
                    if roster.players[i].spawned_at!=old_spawns[i] {
                        ensure!(roster.players[1-i].stats.alive() && roster.players[i].stats.sanity()==25.,"Invalid Herbaceous revival");
                        let next=drivers[1-i].index;
                        rejoins.push(serde_json::json!({"tick":roster.tick,"player":i,"from_step":drivers[i].index,"to_step":next}));
                        drivers[i]=Driver{index:next,..Default::default()};
                        drivers[i].reset(roster.players[i].player.feet);
                    }
                }
            }
            if let Some(id)=arena.stage.owner.filter(|id|Some(*id)==old_owner) {
                ensure!(roster.players[id.partner().index()].player.feet==before[id.partner().index()],
                    "Herbaceous cutscene moved the waiting player");
            }
            if old_owner.is_some() && arena.stage.owner.is_none() {
                reunions+=1;
                // The scene's safe reunion is gameplay. Only synchronize the input cursor.
                let next=drivers.iter().map(|d|d.index).max().unwrap();
                for (d,b) in drivers.iter_mut().zip(&roster.players) { d.index=next; d.reset(b.player.feet); }
                ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,
                    "Herbaceous reunion overlapped the pair");
            }
            ensure!(roster.checkpoint_generation==0 && roster.players.iter().all(|b|
                (allow_revival || b.stats.alive() && b.spawned_at==0) && !b.stats.god && !b.stats.notarget)
                && (old_owner.is_some() || roster.players.each_ref().map(|b|b.warped_at)==old_warps),
                "Herbaceous death/recovery/warp tick{tick} steps{:?} health{:?}",
                drivers.each_ref().map(|d|d.index),roster.players.each_ref().map(|b|b.stats.sanity()));
            if !arena.stage.busy() && roster.players.iter().all(|b|b.stats.alive()) {
                let d=roster.players[0].player.feet-roster.players[1].player.feet;
                ensure!(d.truncate().length()>=crate::coop::separation::DISTANCE-0.05
                    || d.z.abs()>=crate::collision::PLAYER_HALF.z*2.-0.05,
                    "Herbaceous bodies overlap tick{tick}: {d:?}");
            }
            for id in PlayerId::ALL {
                for k in indices[id.index()]..drivers[id.index()].index {
                    if let Some(name)=steps.get(k).and_then(|s|s.save.as_ref()) {
                        record(&arena,&roster,&drivers,first,skip,samples,&history,&format!("{id:?}-{k}-{name}"))?;
                    }
                }
            }
            if tick%600==0 {
                let packed=crate::coop::codec::encode(&serde_json::to_vec(&arena.snapshot())?)?;
                replica.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&packed)?)?,
                    &scene.map,&mut replica_world)?;
                ensure!(serde_json::to_value(replica.snapshot())?==serde_json::to_value(arena.snapshot())?,
                    "Herbaceous snapshot differs after guest restoration");
                samples+=1;
                println!("Herbaceous {first:?} skip={skip} t={} steps{:?} health{:?} feet{:?}",
                    tick/120,drivers.each_ref().map(|d|d.index),roster.players.each_ref().map(|b|b.stats.sanity()),
                    roster.players.each_ref().map(|b|b.player.feet));
                clear_background(BLACK);
                next_frame().await;
            }
            if arena.stage.transition.is_some() { break; }
        }
        let g=garden(&arena);
        ensure!(g.saved.arrived && g.saved.bridge1 && g.saved.bridge2 && g.saved.cat
            && g.saved.scene.is_none() && reunions>=4
            && g.objects.iter().filter(|o|o.group!=0).all(|o|o.pose.is_none())
            // A player can normally miss the optional Mallet after reviving
            // beyond its ledge. Playable mode records that personal inventory;
            // strict mode retains the two-pickup coverage requirement.
            && roster.players.iter().all(|b|b.stats.alive()
                && (allow_revival || b.stats.collected.contains("garden2:57")))
            && roster.players.iter().any(|b|b.stats.collected.contains("garden2:57"))
            && arena.stage.transition==Some(("garden3".into(),Some("garden3_start1".into()))),
            "Herbaceous route omitted the Mallet, bridges, scenes or authored exit");
        let next_visit=Visit {epoch:2,map:"garden3".into(),entry:Some("garden3_start1".into()),..visit};
        let mut next=RenderScene::load_for(a,&next_visit.map,next_visit.difficulty)?;
        let next_arena=Arena::load(a,&next.map,&mut next.world,&next_visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(&next.map,next_visit.entry.as_deref());
        let arrived=roster.arrive_with(&next.world,eye,yaw,&next_visit.map,next_visit.entry.as_deref(),
            |at| !next_arena.stage.interactions.hazardous(at))?;
        ensure!(arrived.players[0].player.feet.distance(arrived.players[1].player.feet)>=36.,
            "Rolling Stones arrival overlaps players");
        for (old,new) in roster.players.iter().zip(&arrived.players) {
            ensure!(serde_json::to_value(&old.stats)?==serde_json::to_value(&new.stats)?
                && next.world.body_clear(new.player.feet),"Rolling Stones arrival changed resources or embedded a player");
        }
        arrival=Some(serde_json::json!({"visit":next_visit,"frame":arrived.frame(2,0)}));
        Ok(())
    }.await;
    record(&arena,&roster,&drivers,first,skip,samples,&history,if result.is_ok(){"passed"}else{"failed"})?;
    let path=crate::preferences::path(&format!("garden2-{first:?}-{skip}-{}.json",if result.is_ok(){"passed"}else{"failed"}));
    let mut report:serde_json::Value=serde_json::from_slice(&std::fs::read(&path)?)?;
    report["allow_revival"]=serde_json::json!(allow_revival);
    report["reunions"]=serde_json::json!(reunions);
    report["revival_rejoins"]=serde_json::json!(rejoins);
    report["arrival"]=serde_json::json!(arrival);
    report["personal_mallet"]=serde_json::json!(roster.players.each_ref().map(|b|b.stats.collected.contains("garden2:57")));
    std::fs::write(path,serde_json::to_vec(&report)?)?;
    result?;
    println!("PASS Herbaceous full paired route {first:?} skip={skip}: {} ticks, {samples} replicas, {reunions} reunions",roster.tick);
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> super::super::BoxFuture<'_> {
    Box::pin(async move {
        std::fs::create_dir_all(crate::preferences::path(""))?;
        let selected=std::env::var("LOOKING_GLASS_GARDEN2_COOP_FIRST").ok();
        let scenes=std::env::var("LOOKING_GLASS_GARDEN2_COOP_SCENES").ok();
        ensure!(selected.as_deref().is_none_or(|v|matches!(v,"Blue"|"Red")),
            "LOOKING_GLASS_GARDEN2_COOP_FIRST must be Blue or Red");
        ensure!(scenes.as_deref().is_none_or(|v|matches!(v,"watched"|"skipped")),
            "LOOKING_GLASS_GARDEN2_COOP_SCENES must be watched or skipped");
        let mut failures=Vec::new();
        for first in PlayerId::ALL {
            if selected.as_ref().is_some_and(|s|*s!=format!("{first:?}")) {continue;}
            for skip in [false,true] {
                if scenes.as_deref().is_some_and(|s|(s=="skipped")!=skip) {continue;}
                if let Err(e)=route(a,first,skip).await {
                    println!("FAIL Herbaceous {first:?} skip={skip}: {e:#}");
                    failures.push(format!("{first:?}/{skip}: {e:#}"));
                }
            }
        }
        ensure!(failures.is_empty(),"Herbaceous routes failed: {}",failures.join("; "));
        Ok(())
    })
}
