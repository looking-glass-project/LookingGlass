//! Ordinary paired swimming, flusher contacts and live fan-gap traversal.
use super::*;
use crate::coop::native_route::{Command,Navigation,Spec,SwimStep};

#[derive(Deserialize)]
struct Step {pos:[f32;3],mode:String,#[serde(default)] frames:u64}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        let steps:Vec<Step>=serde_json::from_slice(include_bytes!("route_steps.json"))?;
        let mut commands=vec![Command::Cards(true),Command::Staff(false)];
        for (index,step) in steps.iter().enumerate() {
            // Use the owned Watch to pass the cross-current with both swimmers.
            if index==28 {commands.push(Command::Watch);}
            // The vacated tray springs back above the solo waypoint's height.
            if index==43 {commands.push(Command::Approach(step.pos));continue;}
            if index==46 {
                commands.push(Command::SwimFile(steps[46..=47].iter().map(|s|
                    SwimStep{at:s.pos,planned:s.mode=="pipe"}).collect()));
                continue;
            }
            if index==47 {continue;}
            commands.push(match step.mode.as_str() {
                "nav"=>Command::Nav(step.pos),
                "swim"|"pipe"|"waterpath"=>Command::Swim{at:step.pos,planned:step.mode!="swim"},
                // Keep the first player moving so the following pull-up clears.
                // The second tall step needs a full-speed jump in single file.
                "jump"|"walk"=>Command::Steer{at:step.pos,
                    mode:if index==20 {"jump".into()}
                        else if step.mode=="jump" || [36,38].contains(&index) {"air".into()}
                        else {step.mode.clone()},single_file:index==20},
                "wait"=>Command::Wait(step.frames),
                "input"=>Command::Drive{direction:step.pos,ticks:step.frames},
                "fan"=>Command::SwimFan{entity:"fan05".into(),controller:"tower2".into(),
                    angle:"/machine/fans/4".into(),speed:100.,height:280.},
                _=>anyhow::bail!("Unsupported paired water route mode {}",step.mode),
            });
        }
        commands.push(Command::Wait(120*30));
        crate::coop::native_route::check(a,Spec{map:"tower2",entry:"tower2_start1",
            exit:crate::level::spec::ExitSpec{map:"hedge3",entrance:"hedge3_start1"},
            commands,navigation:Navigation::LandedPair,verify:|arena,_| {
                let tower=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Tower>())
                    .context("Missing Water Logged")?;
                ensure!(tower.saved.arrived && tower.saved.stages.iter().all(|s|s.is_some_and(|t|t>=5.))
                    && tower.exit_ready(),"Water route omitted an introduction or flusher stage");
                Ok(())
            }}).await
    })
}
