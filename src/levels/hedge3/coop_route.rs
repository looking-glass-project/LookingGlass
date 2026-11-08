//! Full paired Labyrinth route using live machinery and ordinary controls.
use super::*;
use crate::coop::native_route::{Command,CourseStep,CourseClock,Navigation,Spec};

#[derive(Clone,Deserialize)]
struct Step {pos:[f32;3],#[serde(default)] mode:String,#[serde(default)] frames:u64}
fn point(step:&Step)->CourseStep {CourseStep{at:step.pos,mode:step.mode.clone(),ticks:step.frames,clock:None}}
fn walk(at:[f32;3])->CourseStep {CourseStep{at,mode:"walk".into(),ticks:0,clock:None}}
fn flush(commands:&mut Vec<Command>,pending:&mut Vec<CourseStep>) {
    // A last resting player must leave room at the preceding movement goal.
    let mut waits=Vec::new();
    while pending.last().is_some_and(|p|p.mode=="wait") {waits.push(pending.pop().unwrap().ticks);}
    if !pending.is_empty() {commands.push(Command::Course(std::mem::take(pending)));}
    commands.extend(waits.into_iter().rev().map(Command::Wait));
}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        let map=Bsp::parse(&a.read("maps/hedge3.bsp")?)?;
        let objects=motion::load(&map)?;
        let steps:Vec<Step>=serde_json::from_slice(include_bytes!("route_steps.json"))?;
        let mut commands=vec![Command::ContinueWithSurvivor(true),Command::Cards(true)];
        let mut pending=Vec::new();
        for (index,original) in steps.iter().enumerate() {
            // Land beyond the turning tooth instead of stopping on its edge.
            if matches!(index,19|20|178|228..=235) {continue;}
            let mut step=original.clone();
            if index==18 {step.pos=steps[20].pos;}
            if index==11 {commands.push(Command::Course(vec![point(&step)]));continue;}
            // Fight from the broad landing and allow normal personal revival.
            if index==12 {commands.push(Command::Wait(1550));continue;}
            if index==56 {
                flush(&mut commands,&mut pending);
                commands.push(Command::Positions([Some([4580.,-2010.,48.]),Some([4580.,-1955.,48.])]));
                step.pos[0]=1.85;
            }
            if index==58 {
                flush(&mut commands,&mut pending);
                // Bring Red around Blue, line up both lanes and hold their
                // takeoff positions for the next opening in the pump arm.
                for at in [[4640.,-1812.,48.],[4640.,-2030.,48.],[4571.,-1838.,48.]] {
                    commands.push(Command::Positions([None,Some(at)]));
                }
                let k=objects.iter().position(|o|o.id==76).context("Missing pump arm")?;
                commands.push(Command::Cycle{controller:"hedge3".into(),pointer:format!("/machines/{k}/time"),
                    phase:1.85,period:8.,hold:true});
                commands.push(Command::Steer{at:step.pos,mode:"hop".into(),single_file:false});
                continue;
            }
            if index==177 {
                flush(&mut commands,&mut pending);
                // Each rider uses the same takeoff point, then clears the top
                // before the following player attempts the tall ledge grab.
                commands.push(Command::CourseFile(steps[176..=178].iter().map(point).collect()));
                continue;
            }
            if index==175 {
                // Allow the ordinary equip and activation animation to finish.
                commands.extend([Command::Watch,Command::Watch]);
            }
            if index==179 {commands.push(Command::WorldRunning);}
            if index==186 {
                flush(&mut commands,&mut pending);
                commands.push(Command::WatchReady);
                // Activation finishes as the gear reaches its boarding pose.
                step.pos[0]=16.75;
            }
            if index==187 {commands.extend([Command::Watch,Command::Watch]);}
            if index==188 {commands.push(Command::WorldRunning);}
            if index==238 {
                flush(&mut commands,&mut pending);
                commands.push(Command::Wait(1400));
            }
            if index==240 {
                // Enter the proximity field and let the stair door open.
                pending.push(walk([6656.,-2472.,1280.]));
                flush(&mut commands,&mut pending);
                commands.push(Command::Wait(180));
            }
            if index==271 {
                // Let a fallen gear rider revive on the broad exit platform.
                flush(&mut commands,&mut pending);
                commands.push(Command::Wait(1500));
            }
            if index==227 {
                flush(&mut commands,&mut pending);
                commands.push(Command::Course(vec![walk([6112.,-3704.,1152.]),walk([5976.,-3696.,1152.])]));
                commands.push(Command::Clear(450.));
                commands.push(Command::Course(vec![walk([6112.,-3704.,1152.]),walk([6256.,-3712.,1152.])]));
                // Regenerate ordinary Will under cover before fighting the
                // ranged guard. Keep Ice for targets close to either player.
                commands.extend([Command::Staff(true),Command::Wait(48000)]);
                let mut crossing=Vec::new();
                for n in 227..=235 {
                    let s=&steps[n];
                    if s.mode=="phase" {
                        let k=objects.iter().position(|o|o.id==s.pos[1] as usize)
                            .context("Missing walkway pendulum")?;
                        crossing.push(CourseStep{at:steps[n+1].pos,mode:"wait".into(),ticks:0,
                            clock:Some(CourseClock{controller:"hedge3".into(),pointer:format!("/machines/{k}/time"),
                                phase:s.pos[0] as f64,period:s.pos[2] as f64})});
                    } else {crossing.push(point(s));}
                }
                // One player clears all three hammers before the partner
                // starts; neither waits inside a hammer's return sweep.
                commands.push(Command::CourseFile(crossing));
                continue;
            }
            if matches!(step.mode.as_str(),""|"walk"|"jump"|"hop"|"air"|"wait") {
                if index<=10 {
                    commands.push(if step.mode=="wait" {Command::Wait(step.frames)} else {
                        Command::Steer{at:step.pos,mode:step.mode,single_file:false}
                    });
                } else {pending.push(point(&step));}
                continue;
            }
            flush(&mut commands,&mut pending);
            commands.push(match step.mode.as_str() {
                "staff"=>Command::Staff(true),
                "ice"=>Command::Ice(true),
                "phase"=>{
                    let index=objects.iter().position(|o|o.id==step.pos[1] as usize)
                        .context("Unknown route phase object")?;
                    Command::Cycle{controller:"hedge3".into(),pointer:format!("/machines/{index}/time"),
                        phase:step.pos[0] as f64,period:step.pos[2] as f64,hold:false}
                },
                "board"=>Command::MachineBoard{entity:step.frames as usize,local:step.pos},
                "ride"=>Command::MachineRide{entity:step.pos[1] as usize,angle:step.pos[0],tolerance:step.pos[2]},
                "nav"=>Command::Nav(step.pos),
                "clear"=>Command::Clear(step.pos[0]),
                "input"=>Command::Drive{direction:step.pos,ticks:step.frames},
                _=>anyhow::bail!("Unsupported Labyrinth route mode {}",step.mode),
            });
        }
        pending.push(walk([3808.,-1168.,2176.]));
        flush(&mut commands,&mut pending);
        commands.push(Command::Wait(120*30));
        crate::coop::native_route::check(a,Spec{map:"hedge3",entry:"hedge3_start1",
            exit:crate::level::spec::ExitSpec{map:"tower3",entrance:"tower3_start1"},
            commands,navigation:Navigation::LandedPair,verify:|arena,_| {
                ensure!(arena.stage.interactions.levels.iter().any(|s|s.ctl.downcast_ref::<Labyrinth>().is_some()),
                    "Missing live Labyrinth machinery");
                Ok(())
            }}).await
    })
}
