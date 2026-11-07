//! Paired ordinary steering through the authored wind course.
use super::*;
use crate::coop::native_route::{Command,Navigation,Spec};

#[derive(Deserialize)]
struct Step {
    pos:[f32;3],
    mode:String,
    #[serde(default)]
    frames:usize,
}
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        let steps:Vec<Step>=serde_json::from_slice(include_bytes!("route_steps.json"))?;
        let mut commands=vec![Command::ContinueWithSurvivor(true),Command::Cards(true),Command::Staff(false)];
        for (i,s) in steps.iter().enumerate() {
            // The final three updrafts require continuous personal steering.
            // A shared waypoint would leave the first rider waiting in air.
            if (70..=72).contains(&i) {continue;}
            if i==69 {
                commands.push(Command::Flight{takeoff:steps[68].pos,
                    via:steps[69..=71].iter().map(|s|s.pos).collect(),landing:steps[72].pos});
                continue;
            }
            // Let the front player clear these narrow landings before the
            // partner approaches; the ordinary planner handles the corners.
            if [7,35,49].contains(&i) {commands.push(Command::Approach(s.pos));continue;}
            commands.push(match s.mode.as_str() {
                "nav"=>Command::Nav(s.pos),
                "precise"|"air"=>Command::Steer{at:s.pos,mode:s.mode.clone(),single_file:false},
                "phase"=>{
                    ensure!((1..=4).contains(&s.frames),"Unknown Tower face");
                    Command::Phase{controller:"tower1".into(),pointer:format!("/faces/{}",s.frames-1),
                        start:s.pos[0] as f64,end:s.pos[1] as f64}
                },
                _=>anyhow::bail!("Unsupported paired Tower course mode {}",s.mode),
            });
        }
        commands.push(Command::Wait(120*30));
        crate::coop::native_route::check(a,Spec{map:"tower1",entry:"tower1_start1",
            exit:crate::level::spec::ExitSpec{map:"hedge2",entrance:"hedge2_start1"},
            commands,navigation:Navigation::LandedPair,verify:|arena,_| {
                let tower=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Tower>())
                    .context("Missing Airborne Terror")?;
                ensure!(tower.saved.arrived && tower.saved.scene.is_none() && arena.stage.story.has_seen(INTRO),
                    "Tower course omitted its original arrival scene");
                Ok(())
            }}).await
    })
}
