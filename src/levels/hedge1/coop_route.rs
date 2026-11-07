//! Normal paired escort, personal portal and physical gated departure.
use super::*;
use crate::coop::native_route::{Command,Navigation,Spec};
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        let steps:Vec<serde_json::Value>=serde_json::from_str(include!("route_input.rs"))?;
        let mut commands=vec![Command::ContinueWithSurvivor(false),Command::Cards(true),Command::Ice(true),Command::Staff(false)];
        let trail=|at:[f32;3],lead:bool,portal:bool,direct:bool|Command::Trail{at,escort:lead.then(||"seek_kid_chase".into()),portal,direct,
            escort_player:lead.then_some(crate::coop::PlayerId::Red)};
        for s in steps {
            if let Some(at)=s.get("go").or_else(||s.get("steer")) {
                let at:[f32;3]=serde_json::from_value(at.clone())?;
                if s["lead"]==true {
                    // Clear the narrow leg before the second Alice escorts the
                    // child through it; two side-by-side pilots can jam turns.
                    commands.push(Command::Positions([Some(at),None]));
                    if at==[5296.,4120.,1792.] {
                        // Step back down from a collected essence's raised
                        // footing before rejoining the map's walking graph.
                        commands.push(Command::Positions([None,Some(at)]));
                    }
                }
                commands.push(trail(at,s["lead"]==true,false,s.get("steer").is_some()));
                if s["lead"]==true && s.get("steer").is_some() {
                    // Close up inside the holding alcove so the child's nearest
                    // Alice is far enough past the plate for him to step on it.
                    commands.push(Command::Drive{direction:[0.,-1.,0.],ticks:120});
                }
            } else if let Some(t)=s.get("wait").and_then(|t|t.as_f64()) {
                commands.push(Command::Wait((t*120.).round() as u64));
            } else if s.get("watch_ready")==Some(&serde_json::Value::Bool(true)) {commands.push(Command::WatchReady);}
            else if s.get("watch")==Some(&serde_json::Value::Bool(true)) {commands.push(Command::Watch);}
            else if let Some(radius)=s.get("clear").and_then(|v|v.as_f64()) {commands.push(Command::Clear(radius as f32));}
            else if s.get("portal")==Some(&serde_json::Value::Bool(true)) {
                commands.push(trail([3136.,4288.,1920.],false,false,false));
                commands.push(trail([3136.,4480.,1936.],false,true,true));
            } else if s.get("assert").and_then(|v|v.as_str())==Some("holding") {
                commands.push(Command::Phase{controller:"hedge1".into(),pointer:"/doors/1".into(),start:0.98,end:1.1});
            } else {ensure!(s.get("save").is_some()||s.get("assert").is_some(),"Unknown Maze route step");}
        }
        commands.push(Command::Wait(3600));
        crate::coop::native_route::check(a,Spec{map:"hedge1",entry:"hedge1_start1",exit:EXIT,
            commands,navigation:Navigation::LandedPair,verify:|arena,_| {
                let maze=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Maze>())
                    .context("Missing actual Maze controller")?;
                ensure!(maze.ready()&&maze.saved.start_seen&&maze.saved.end_seen,"Actual child escort/gate incomplete");
                ensure!(arena.npcs.companions().iter().any(|c|c.name=="seek_kid_chase"&&c.visible&&c.holding),
                    "Real child is not holding the exit plate");
                Ok(())
            }}).await
    })
}
