//! Ordinary paired controls through the live chase; no body or world staging.
use super::*;
use crate::{coop::{arena::Arena, protocol::{Input, Visit}, world::Roster},
    inventory::Stats, movement::Controls};

fn garden(arena: &Arena) -> &Garden {
    arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Garden>()).unwrap()
}
#[derive(Default)]
struct Driver { step: usize, ticks: usize, last: Vec3 }
impl Driver {
    fn sample(&mut self, goals: &[[f32;3]], world: &World, p: &Player) -> Result<(Controls,bool)> {
        while let Some(raw)=goals.get(self.step) {
            let goal=Vec3::from_array(*raw);
            let delta=goal.truncate()-p.feet.truncate();
            if delta.length()<30. && p.feet.z<goal.z+180. {
                self.next(p.feet); continue;
            }
            ensure!(self.ticks<120*30,"Paired chase stalled at step{} goal{goal:?}, feet{:?}",self.step,p.feet);
            let pad=world.traversal.pushes.iter().find(|pad|
                pad.origin.truncate().distance(goal.truncate())<160.);
            let touching=pad.is_some_and(|pad|pad.volume.touches(
                p.feet+crate::collision::PLAYER_CENTER,
                p.feet+crate::collision::PLAYER_CENTER+p.velocity/120.,crate::collision::PLAYER_HALF));
            let jump=self.ticks%90==89 && p.feet.distance(self.last)<25.;
            if self.ticks%90==89 { self.last=p.feet; }
            self.ticks+=1;
            return Ok((Controls{wish:delta.normalize_or_zero(),run:true,jump,..Default::default()},touching));
        }
        Ok((Controls::default(),false))
    }
    fn next(&mut self, feet: Vec3) { self.step+=1; self.ticks=0; self.last=feet; }
}
pub(super) fn check(a: &mut Assets) -> super::super::BoxFuture<'_> {
    Box::pin(async move {
        std::fs::create_dir_all(crate::preferences::path(""))?;
        let visit=Visit{epoch:1,map:"garden3".into(),entry:Some("garden3_start1".into()),
            difficulty:Default::default(),combat_preview:false};
        let map=Bsp::parse(&a.read("maps/garden3.bsp")?)?;
        let mut world=World::from_bsp(&map)?;
        let mut arena=Arena::load(a,&map,&mut world,&visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(&map,visit.entry.as_deref());
        let mut roster=Roster::new_with(&world,eye,yaw,Stats::for_level("garden3",visit.entry.as_deref()),
            |at|!arena.stage.interactions.hazardous(at))?;
        arena.reset_characters(&roster.players);
        let mut guest_world=World::from_bsp(&map)?;
        let mut guest=Arena::load(a,&map,&mut guest_world,&visit,false)?;
        let goals=route::goals()?;
        ensure!(!goals.is_empty() && goals.iter().flatten().all(|x|x.is_finite()),"Invalid chase course");
        std::fs::write(crate::preferences::path("garden3-coop-course.json"),serde_json::to_vec(&goals)?)?;
        let mut drivers:[Driver;2]=Default::default();
        let mut trace=crate::coop::recovery_trace::Trace::new(crate::preferences::path("garden3-recovery.jsonl"))?;
        trace.record(&roster,&world,1,"garden3",arena.stage.busy(),false,|at|!arena.stage.interactions.hazardous(at))?;
        let mut samples=0;
        let mut reunions=0;
        let mut rejoins=Vec::new();
        let mut launches=Vec::new();
        let mut arrival=None;
        let mut free_ticks=0;
        let result:Result<()> = async {
            for _ in 0..120*600 {
                let owner=arena.stage.owner;
                let busy=arena.stage.busy();
                let feet=roster.players.each_ref().map(|b|b.player.feet);
                let warps=roster.players.each_ref().map(|b|b.warped_at);
                let spawned=roster.players.each_ref().map(|b|b.spawned_at);
                let mut inputs=[(Controls::default(),Input::default());2];
                let mut touching=[false;2];
                if !busy {
                    for i in 0..2 {
                        if !roster.players[i].stats.alive() { continue; }
                        // Let the leader leave the shared reunion footing
                        // before the follower enters the narrow first descent.
                        if i==1 && free_ticks<120 { continue; }
                        let (c,pad)=drivers[i].sample(&goals,&world,&roster.players[i].player)?;
                        inputs[i].0=c; touching[i]=pad;
                        if c.wish.length_squared()>0. { inputs[i].1.yaw=c.wish.y.atan2(c.wish.x); }
                    }
                    free_ticks+=1;
                }
                arena.step(&mut roster,&map,&mut world,inputs)?;
                trace.record(&roster,&world,1,"garden3",busy||arena.stage.busy(),
                    arena.stage.interactive_actor().is_some(),|at|!arena.stage.interactions.hazardous(at))?;
                ensure!(roster.checkpoint_generation==0 && roster.players.iter().all(|b|!b.stats.god&&!b.stats.notarget)
                    && (owner.is_some()||roster.players.each_ref().map(|b|b.warped_at)==warps),
                    "Paired chase wiped or warped at tick{}",roster.tick);
                if let Some(id)=arena.stage.owner.filter(|id|Some(*id)==owner) {
                    ensure!(roster.players[id.partner().index()].player.feet==feet[id.partner().index()],
                        "Chase scene moved its waiting partner");
                }
                if owner.is_some() && arena.stage.owner.is_none() {
                    reunions+=1;
                    ensure!(roster.players[0].player.feet.distance(roster.players[1].player.feet)>=36.,"Chase reunion overlaps");
                }
                for i in 0..2 {
                    if roster.players[i].spawned_at!=spawned[i] {
                        ensure!(roster.players[1-i].stats.alive() && roster.players[i].stats.sanity()==25.,"Invalid chase revival");
                        let next=drivers[1-i].step;
                        rejoins.push(serde_json::json!({"tick":roster.tick,"player":i,"from_step":drivers[i].step,"to_step":next}));
                        drivers[i]=Driver{step:next,ticks:0,last:roster.players[i].player.feet};
                    } else if touching[i] && roster.players[i].player.velocity.z>100. {
                        launches.push(serde_json::json!({"tick":roster.tick,"player":i,"step":drivers[i].step}));
                        drivers[i].next(roster.players[i].player.feet);
                    }
                }
                if roster.tick%120==0 {
                    let packed=crate::coop::codec::encode(&serde_json::to_vec(&arena.snapshot())?)?;
                    guest.replicate(&serde_json::from_slice(&crate::coop::codec::decode(&packed)?)?,&map,&mut guest_world)?;
                    ensure!(serde_json::to_value(guest.snapshot())?==serde_json::to_value(arena.snapshot())?,"Chase replica differs");
                    samples+=1;
                    if roster.tick%600==0 {
                        println!("Paired chase tick{} steps{:?} health{:?} feet{:?}",roster.tick,drivers.each_ref().map(|d|d.step),
                            roster.players.each_ref().map(|b|b.stats.sanity()),roster.players.each_ref().map(|b|b.player.feet));
                    }
                    clear_background(BLACK); next_frame().await;
                }
                if arena.stage.transition.is_some() { break; }
            }
            let g=garden(&arena);
            // Fast normal play can reach the exit ahead of the marble's later
            // ice callback and gate latches. Those clocks have component checks.
            ensure!(reunions==1 && g.saved.arrived && g.saved.end.is_some() && g.saved.pillar.is_some()
                && roster.players.iter().all(|b|b.stats.alive())
                && arena.stage.transition==Some(("garden4".into(),Some("garden4_start1".into()))),
                "Paired chase omitted progression or live exit");
            let next_visit=Visit{epoch:2,map:"garden4".into(),entry:Some("garden4_start1".into()),..visit};
            let next_map=Bsp::parse(&a.read("maps/garden4.bsp")?)?;
            let mut next_world=World::from_bsp(&next_map)?;
            let next_arena=Arena::load(a,&next_map,&mut next_world,&next_visit,false)?;
            let (eye,yaw)=crate::interaction::spawn(&next_map,next_visit.entry.as_deref());
            let arrived=roster.arrive_with(&next_world,eye,yaw,"garden4",next_visit.entry.as_deref(),|at|!next_arena.stage.interactions.hazardous(at))?;
            ensure!(arrived.players[0].player.feet.distance(arrived.players[1].player.feet)>=36.,"Chase arrival overlaps");
            for (old,new) in roster.players.iter().zip(&arrived.players) {
                ensure!(serde_json::to_value(&old.stats)?==serde_json::to_value(&new.stats)? && next_world.body_clear(new.player.feet),
                    "Chase onward resources or placement differ");
            }
            arrival=Some(serde_json::json!({"visit":next_visit,"frame":arrived.frame(2,0)}));
            Ok(())
        }.await;
        let report=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"full_route_passed":result.is_ok(),
            "error":result.as_ref().err().map(|e|format!("{e:#}")),"frame":roster.frame(1,0),"world":arena.snapshot(),
            "steps":drivers.each_ref().map(|d|d.step),"samples":samples,"reunions":reunions,"revival_rejoins":rejoins,
            "launches":launches,"arrival":arrival});
        std::fs::write(crate::preferences::path("garden3-coop-route.json"),serde_json::to_vec(&report)?)?;
        result?;
        println!("PASS paired Rolling Stones into Icy Reception: {} ticks, {samples} replicas",roster.tick);
        Ok(())
    })
}
