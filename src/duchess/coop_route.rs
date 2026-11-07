//! Fresh paired entrance, authored pickup, boss fight and watched departure.
use super::*;
use crate::coop::native_route::{Command, Navigation, Spec};

pub(crate) fn check(a: &mut Assets) -> crate::levels::BoxFuture<'_> {
    crate::coop::native_route::check(a, Spec {
        map: "potears3", entry: "potears3_start1",
        // The original departure selects the destination's default spawn.
        exit: crate::level::spec::ExitSpec { map: "utemple", entrance: "" },
        navigation: Navigation::LandedPair,
        commands: vec![Command::Staff(false), Command::Cards(true),
            Command::Nav([-832., -1000., 16.]), Command::Nav([-536., -520., 16.]),
            Command::Nav([-280., -264., 16.]), Command::Approach([-114., 66., 16.]),
            Command::Clear(4000.), Command::Wait(12000)],
        verify: |arena, players| {
            let d=arena.stage.interactions.duchess.as_ref().context("Missing Duchess")?;
            ensure!(d.state.stage==Stage::Complete && d.state.health==0.
                && d.state.shell_returned && d.state.action==Action::Death,
                "Duchess defeat and turtle rescue incomplete");
            ensure!(players.iter().any(|b|b.stats.collected.contains("potears3:21") && b.stats.copies(3)>0),
                "Actual Jackbomb pickup missing");
            Ok(())
        },
    })
}
