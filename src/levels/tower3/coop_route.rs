//! Normal paired Machinations progression with the live lift clocks.
use super::*;
use crate::coop::native_route::{Command,CourseStep,CourseClock,Navigation,Spec};

#[derive(Clone,Deserialize)]
struct Step {pos:[f32;3],#[serde(default)] mode:String,#[serde(default)] frames:u64}
fn flush(commands:&mut Vec<Command>,pending:&mut Vec<CourseStep>) {
    if !pending.is_empty() {commands.push(Command::Course(std::mem::take(pending)));}
}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        let map=Bsp::parse(&a.read("maps/tower3.bsp")?)?;
        let objects=motion::load(&map)?;
        let roots=objects.iter().enumerate().filter(|(_,o)|o.parent.is_none())
            .map(|(i,_)|i).collect::<Vec<_>>();
        let steps:Vec<Step>=serde_json::from_slice(include_bytes!("route_steps.json"))?;
        let clock=|step:&Step|->Result<CourseClock> {
            let mut k=objects.iter().position(|o|o.id==step.frames as usize)
                .context("Missing Machinations phase part")?;
            while let Some(parent)=objects[k].parent {k=parent;}
            let index=roots.iter().position(|i|*i==k).context("Missing Machinations root clock")?;
            Ok(CourseClock{controller:"tower3".into(),pointer:format!("/clocks/{index}"),
                phase:step.pos[0] as f64,period:step.pos[1] as f64})
        };
        let mut commands=vec![Command::ContinueWithSurvivor(true),Command::Cards(true),Command::Staff(true)];
        let mut pending=Vec::new();
        for (index,original) in steps.iter().enumerate() {
            let mut step=original.clone();
            // Intentional fall-return demonstrations have separate recovery
            // coverage; follow the normal connected machinery course here.
            if index==0 || matches!(step.mode.as_str(),"fall"|"room") {continue;}
            if index==7 {step.mode="jump".into();}
            if index==48 {step.mode="precise".into();}
            if index==85 {step.pos=[2320.,1200.,1590.];}
            if index==86 {step.pos=[2110.,1440.,1544.];}
            if index==92 {step.pos=[2080.,2015.,1770.];}
            if index==9 {commands.push(Command::CourseFile(std::mem::take(&mut pending)));}
            if let Some(entity)=match index {20=>Some(47),23=>Some(156),28=>Some(48),_=>None} {
                flush(&mut commands,&mut pending);
                commands.push(Command::MachineBoard{entity,local:[0.,0.,16.]});
                continue;
            }
            if (47..=58).contains(&index) {
                // Let the leader leave each narrow pillar before the follower
                // lands, while both use their own live pedal phase and support.
                let phase=if step.mode=="phase" {
                    let mut c=clock(&step)?;
                    // The pedal and receiving lift must both be in reach.
                    if index==55 {c.period=60.;c.phase=21.8;}
                    Some(c)
                } else {None};
                pending.push(CourseStep{at:step.pos,mode:if phase.is_some() {"wait".into()} else {step.mode.clone()},
                    ticks:step.frames,clock:phase});
                continue;
            }
            if index==70 {
                pending.push(CourseStep{at:step.pos,mode:step.mode,ticks:step.frames,clock:None});
                commands.push(Command::CourseFile(std::mem::take(&mut pending)));
                continue;
            }
            if index>=94 {
                // Keep each rider moving off the narrow vertical gear as soon
                // as their own landing and height permit it.
                if index==95 {step.pos=[1915.,1980.,1875.];step.mode=String::new();step.frames=0;}
                pending.push(CourseStep{at:step.pos,mode:step.mode,ticks:step.frames,clock:None});
                continue;
            }
            if matches!(step.mode.as_str(),""|"walk"|"jump"|"air"|"hop"|"precise") {
                pending.push(CourseStep{at:step.pos,mode:step.mode.clone(),ticks:0,clock:None});
                continue;
            }
            flush(&mut commands,&mut pending);
            commands.push(match step.mode.as_str() {
                "phase"|"holdphase"=>{
                    let c=clock(&step)?;
                    Command::Cycle{controller:c.controller,pointer:c.pointer,
                        phase:c.phase,period:c.period,hold:step.mode=="holdphase"}
                },
                "board"=>Command::MachineBoard{entity:step.frames as usize,local:step.pos},
                "rise"=>Command::RideHeight(step.pos[2]),
                "wait"=>Command::Wait(step.frames),
                "input"=>Command::Drive{direction:step.pos,ticks:step.frames},
                _=>anyhow::bail!("Unsupported Machinations course mode {}",step.mode),
            });
        }
        flush(&mut commands,&mut pending);
        commands.push(Command::Wait(3600));
        crate::coop::native_route::check(a,Spec{map:"tower3",entry:"tower3_start1",
            exit:crate::level::spec::ExitSpec{map:"grounds1",entrance:"grounds1_start1"},
            commands,navigation:Navigation::LandedPair,verify:|arena,_| {
                let tower=arena.stage.interactions.levels.iter()
                    .find_map(|s|s.ctl.downcast_ref::<Tower>()).context("Missing live Tower machinery")?;
                ensure!(tower.saved.arrived && tower.saved.scene.is_none(),"Machinations intro incomplete");
                ensure!(tower.saved.coop.as_ref().is_some_and(|s|s.room==[3,3]),
                    "Both players must reach the final room checkpoint");
                Ok(())
            }}).await
    })
}
