//! Ordinary paired traversal into the original ambush and timed departure.
use super::*;
pub(super) fn check(a:&mut Assets)->super::super::BoxFuture<'_> {
    Box::pin(async move {
        use crate::coop::native_route::{Command,Navigation,Spec};
        let mut commands=check::goals()?.into_iter().map(Command::Approach).collect::<Vec<_>>();
        // The original scene hands controls back for its ten-second ant ambush.
        // Normal combat inputs continue until the authored departure commits.
        commands.push(Command::Wait(120*90));
        crate::coop::native_route::check(a,Spec{map:"centipede1",entry:"centipede1_start1",exit:EXIT,
            commands,navigation:Navigation::LandedPair,verify:|arena,players| {
                let flora=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Flora>())
                    .context("Missing Fungiferous Flora")?;
                ensure!(flora.saved.phase==Phase::Done && flora.saved.exit.committed
                    && flora.saved.wave>0 && flora.saved.talk && arena.stage.story.has_seen(TALK),
                    "Flora omitted its voice-over, ambush or timed departure");
                ensure!(flora.saved.ants.iter().filter(|s|matches!(s.ant.id,36|201|228|229))
                    .all(|s|s.ant.enabled),"Flora missed the original ant activation");
                ensure!(players.iter().any(|b|b.stats.collected.contains("centipede1:299") && b.stats.copies(6)==2),
                    "Flora route missed the second personal Demon Die");
                ensure!(players.iter().all(|b|b.stats.minimum_sanity==10.),"Flora countdown floor was not personal");
                Ok(())
            }}).await
    })
}
