//! Original single-Alice arrival through the paired Stage, with both owners.
use super::*;
pub(super) fn check(a: &mut Assets, map: &Bsp) -> Result<()> {
    for first in PlayerId::ALL { for skip in [None, Some(0.2), Some(3.)] { for dead in [false, true] {
        let (mut s, mut w, mut r) = fixture(a, map, true)?;
        r.players[0].stats.set_health(47.)?; r.players[1].stats.set_health(63.)?;
        let other = first.partner().index();
        if dead { r.players[other].stats.damage(1000.); }
        let resources = serde_json::to_value(r.players.each_ref().map(|p| &p.stats))?;
        let home = r.players[first.index()].player.feet;
        let grounded_home = r.players[first.index()].player.grounded;
        let waiting = r.players[other].player.feet;
        s.owner = Some(first); s.accept(Events::default(), first, &mut r, &w);
        let mut done = false; let mut skipped = false; let mut camera = false;
        for tick in 0..120 * 60 {
            let just_restored = tick == 12 || tick == 360;
            if just_restored {
                let saved = tower(&mut s).snapshot(); let people = serde_json::to_value(&r.players)?;
                for dt in [0., -1., f32::NAN, f32::INFINITY] {
                    tower(&mut s).coop_advance_roster(dt, map, &mut w, &mut bodies(&mut r), &[])?;
                }
                ensure!(tower(&mut s).snapshot() == saved && serde_json::to_value(&r.players)? == people,
                    "Paused paired arrival mutated scene or either body");
                restore(&mut s, map, &mut w)?;
            }
            let mut input = [(Controls::default(), Input::default()); 2];
            input[other].1.skip_scene = true;
            // A restored scene requires a fresh released-then-held skip press.
            input[first.index()].1.skip_scene = !just_restored
                && skip.is_some_and(|at| tick as f32 * FIXED_DT >= at);
            s.scene_step(&mut r, map, &mut w, &input)?;
            skipped |= s.notices[first.index()].as_ref().is_some_and(|n| n.text == "Scene skipped");
            if s.owner.is_none() { done = true; break; }
            ensure!(s.owner == Some(first) && r.players[other].player.feet == waiting,
                "Airborne intro changed actor or moved the waiting partner");
            if let Some(scene) = &tower(&mut s).saved.scene {
                ensure!(scene.clock.home.is_some_and(|pose| pose.translation == home), "Arrival captured wrong home");
                ensure!(s.camera(&w).is_some(), "Original Airborne camera missing"); camera = true;
            }
        }
        ensure!(done && camera && skipped == skip.is_some() && s.story.has_seen(INTRO)
            && tower(&mut s).saved.arrived && !tower(&mut s).scripted(),
            "Airborne arrival failed: owner{first:?}, requested{skip:?}, dead{dead}, done{done}, skipped{skipped}");
        ensure!(r.players[first.index()].player.feet == home
            && serde_json::to_value(r.players.each_ref().map(|p| &p.stats))? == resources,
            "Airborne arrival changed home or personal resources");
        if dead {
            ensure!(!r.players[other].stats.alive() && r.players[other].player.feet == waiting,
                "Arrival revived or moved dead partner");
        } else {
            // Preserve the authored owner pose, which can be above its first
            // landing. The reunited partner still needs a supported footprint.
            ensure!(r.players.iter().all(|p| w.body_clear(p.player.feet))
                && (!grounded_home || crate::recovery::supported(&w, r.players[first.index()].player.feet))
                && crate::recovery::supported(&w, r.players[other].player.feet)
                && r.players[0].player.feet.distance(r.players[1].player.feet) >= crate::coop::separation::DISTANCE - 0.05,
                "Airborne reunion unsafe: owner grounded={grounded_home}, feet={:?}", r.players.each_ref().map(|p| p.player.feet));
        }
        println!("PASS paired Airborne intro {first:?} skip{skip:?} dead{dead} grounded_home={grounded_home}: original camera/home/story, compressed restore, frozen partner and safe reunion");
    }}}
    Ok(())
}
