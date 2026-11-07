//! Normal paired lever, combat and underwater progression through the maze.
use super::*;
use crate::coop::native_route::{Command,Navigation,Spec};

#[derive(Deserialize)]
struct Step {pos:[f32;3],mode:String}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        let map=Bsp::parse(&a.read("maps/hedge2.bsp")?)?;
        let steps:Vec<Step>=serde_json::from_slice(include_bytes!("route_steps.json"))?;
        let mut commands=vec![Command::Cards(true),Command::Staff(false)];
        for (index,step) in steps.into_iter().enumerate() {
            // The pickup spur turns back along a narrow ledge. Clear it in
            // single file before the partner starts the return jump.
            if [94,95].contains(&index) {commands.push(Command::Approach(step.pos));continue;}
            // Leave the upper combat ledge with normal air steering; a ground
            // route can keep planning through the partner at this corner.
            if index==118 {commands.push(Command::Steer{at:step.pos,mode:"precise".into(),single_file:false});continue;}
            let command=match step.mode.as_str() {
                "nav"=>Command::Nav(step.pos),
                "jump"=>Command::Steer{at:step.pos,mode:step.mode,single_file:index==112},
                "swim"|"pipe"=>Command::Swim{at:step.pos,planned:step.mode=="pipe"},
                "clear"=>Command::Clear(step.pos[0]),
                "wait"=>Command::Wait((step.pos[0]*120.) as u64),
                "watch_ready"=>Command::WatchReady,
                "watch"=>{commands.push(Command::WatchReady);Command::Watch},
                "staff"=>Command::Staff(true),
                "use"=>{
                    let index=step.pos[0] as usize;
                    ensure!(index<LEVERS.len(),"Unknown route lever");
                    let point=at(&map.entities[LEVERS[index]]).point(vec3(36.,0.,26.));
                    commands.extend([Command::WorldRunning,Command::Interact(point.to_array())]);
                    Command::Wait(264)
                },
                _=>anyhow::bail!("Unsupported paired maze mode {}",step.mode),
            };
            commands.push(command);
        }
        commands.push(Command::Wait(120*30));
        crate::coop::native_route::check(a,Spec{map:"hedge2",entry:"hedge2_start1",
            exit:crate::level::spec::ExitSpec{map:"tower2",entrance:"tower2_start1"},
            commands,navigation:Navigation::LandedPair,verify:|arena,players| {
                let maze=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Maze>())
                    .context("Missing Mystifying Madness")?;
                ensure!(maze.saved.finished==[true,true] && players.iter().all(|b|b.stats.turtle_air),
                    "Maze route omitted a lever scene or personal breathing shell");
                Ok(())
            }}).await
    })
}
