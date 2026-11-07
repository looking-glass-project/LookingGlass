//! Normal paired courses through both visits; no scripted state injection.
use super::*;
use crate::coop::native_route::{Command,Navigation,Spec};

fn run(a:&mut Assets,returning:bool)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        let mut commands=vec![Command::Staff(false)];
        // Spend ordinary personal Will on ranged cards in the first visit.
        // The return preserves it for the mandatory Staff wall instead.
        if !returning {commands.push(Command::Cards(true));}
        for goal in check::goals(returning)? {
            // A cutscene can interrupt the approach. Finish the same ordinary
            // path after its reunion, as the single-player course does.
            commands.push(Command::Approach(goal.to_array()));
            commands.push(Command::Approach(goal.to_array()));
            if !returning && (goal.x-2408.).abs()<1. && (goal.y-4504.).abs()<1. {
                commands.push(Command::Clear(1000.));
            }
        }
        if returning {
            commands.extend([Command::Watch,Command::Staff(true),Command::Wait(120*10),
                Command::Approach([6504.,5432.,768.03125]),Command::Walk([6504.,5432.,768.03125])]);
        } else {
            // This marker is the near face of an exit against a wall. Searching
            // upward for footing would select the roof instead of the passage.
            commands.push(Command::Nav([1528.,4352.,576.03125]));
            commands.push(Command::Walk([1528.,4352.,576.03125]));
        }
        commands.push(Command::Wait(120*30));
        crate::coop::native_route::check(a,Spec{map:"wforest",
            entry:if returning{"wforest_start2"}else{"wforest_start1"},
            exit:if returning{HEDGE}else{CHESS},commands,navigation:Navigation::LandedPair,
            verify:|arena,players| {
                let forest=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Forest>())
                    .context("Missing Caterpillar's Plot")?;
                if forest.saved.returning {
                    ensure!(forest.saved.exit.committed && forest.hedge_ready()
                        && forest.saved.wall_health==0. && forest.saved.done[Kind::Wall.index()],
                        "Return forest missed its wall scene or ordinary Staff destruction");
                } else {
                    // The first visit's gated trigger transitions directly;
                    // ExitState is only used by the return's timed departure.
                    ensure!(forest.first_ready() && forest.saved.done.iter().take(4).all(|done|*done)
                        && players.iter().all(|p|p.stats.staff_component),
                        "First forest missed its original scenes or shared Staff component");
                }
                Ok(())
            }}).await
    })
}
pub(super) fn first(a:&mut Assets)->super::super::BoxFuture<'_> {run(a,false)}
pub(super) fn returning(a:&mut Assets)->super::super::BoxFuture<'_> {run(a,true)}
