//! Staged native contracts, not a continuous paired campaign route.
use super::*;
use crate::{level::CoopPlayer, movement::FIXED_DT, powerups::Difficulty};
pub(super) fn slots<'a>(p: &'a mut [Player; 2], s: &'a mut [Stats; 2]) -> [CoopPlayer<'a>; 2] {
    let [a, b] = p.each_mut();
    let [x, y] = s.each_mut();
    [
        CoopPlayer {
            player: a,
            stats: x,
        },
        CoopPlayer {
            player: b,
            stats: y,
        },
    ]
}
fn stats() -> [Stats; 2] {
    std::array::from_fn(|_| Stats::for_level("hatter2", None))
}
fn people() -> [Player; 2] {
    [vec3(5640., -1088., 0.03125), vec3(5688., -1088., 0.03125)].map(|at| {
        let mut p = Player::new(at);
        p.grounded = true;
        p
    })
}
fn world(map: &Bsp, o: &Encounter) -> Result<World> {
    let mut w = World::from_bsp(map)?;
    w.set_dynamic(o.colliders());
    Ok(w)
}
fn restore(a: &mut Assets, map: &Bsp, o: &Encounter) -> Result<Encounter> {
    let encoded = crate::coop::codec::encode(&serde_json::to_vec(&o.snapshot())?)?;
    let mut copy = Encounter::load(a, map)?;
    copy.restore(
        &serde_json::from_slice(&crate::coop::codec::decode(&encoded)?)?,
        map,
    )?;
    ensure!(
        copy.snapshot() == o.snapshot(),
        "Paired Hatter reward/world restore changed state"
    );
    Ok(copy)
}
pub(super) fn victory(o: &mut Encounter) {
    o.saved.boss.set(battle::Action::Gone);
    o.saved.boss.health = 0.;
    o.saved.boss.malfunctions = 3;
    o.phase(Phase::Victory);
    o.saved.initialized = true;
}
fn clocks(a: &mut Assets, map: &Bsp) -> Result<()> {
    for hz in [30, 60, 144] {
        let mut o = Encounter::load(a, map)?;
        let mut solo = Encounter::load(a, map)?;
        let mut p = people();
        let mut s = stats();
        let mut w = world(map, &o)?;
        let mut sw = world(map, &solo)?;
        ensure!(
            p.iter()
                .all(|p| w.body_clear(p.feet) && crate::recovery::supported(&w, p.feet)),
            "World clock fixture is not on clear native entrance footing"
        );
        o.pair_prepare(&mut slots(&mut p, &mut s));
        let mut single = p[0].clone();
        solo.prepare_player(&mut s[0], &mut single);
        s[0].damage(17.);
        s[1].damage(41.);
        o.pair_prepare(&mut slots(&mut p, &mut s));
        ensure!(
            s[0].sanity() == 83. && s[1].sanity() == 59.,
            "Repeated preparation refilled health"
        );
        let before = o.snapshot();
        let people = serde_json::to_value(&p)?;
        for dt in [0., -1., f32::NAN] {
            o.pair_advance(dt, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            o.pair_pickups(dt, &mut slots(&mut p, &mut s));
        }
        ensure!(
            o.snapshot() == before && serde_json::to_value(&p)? == people,
            "Paused/invalid paired world advanced"
        );
        for n in 0..hz * 2 {
            let dt = 1. / hz as f32;
            o.pair_advance(dt, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            solo.advance(dt, map, &mut sw, &mut single, &[])?;
            let mut state = o.snapshot();
            state.as_object_mut().unwrap().remove("coop");
            ensure!(
                state == solo.snapshot() && o.transforms() == solo.transforms(),
                "Shared world differs {hz}Hz tick{n}"
            );
        }
        restore(a, map, &o)?;
        println!(
            "PASS paired Hatter {hz}Hz world parity, personal arrival grant and paused restore"
        );
    }
    for dead in 0..2 {
        let mut o = Encounter::load(a, map)?;
        let mut p = people();
        let mut s = stats();
        s[dead].damage(1000.);
        o.pair_prepare(&mut slots(&mut p, &mut s));
        ensure!(!s[dead].alive(), "Arrival resurrected a player");
        s[dead].set_health(25.)?;
        o.pair_prepare(&mut slots(&mut p, &mut s));
        ensure!(
            s[dead].sanity() == 25.,
            "Preparation bypassed low-health revival"
        );
        println!("PASS paired Hatter dead arrival and low-health revival slot{dead}");
    }
    Ok(())
}
fn healing(a: &mut Assets, map: &Bsp) -> Result<()> {
    for first in 0..2 {
        for difficulty in [Difficulty::Normal, Difficulty::Easy] {
            let mut o = Encounter::load(a, map)?;
            o.start_fight();
            o.saved.initialized = true;
            o.saved.essence_live = true;
            let mut p = people();
            let mut s = stats();
            for s in &mut s {
                s.difficulty = difficulty;
                s.set_health(40.)?;
            }
            o.pair_prepare(&mut slots(&mut p, &mut s));
            let at = o.visible_essence(Some(first)).unwrap();
            p[first].feet = at - Vec3::Z * 32.;
            p[1 - first].feet = vec3(5500., 0., 0.);
            o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
            ensure!(
                s[first].sanity() == 65.
                    && s[1 - first].sanity() == 40.
                    && o.visible_essence(Some(first)).is_none()
                    && o.visible_essence(Some(1 - first)) == Some(at),
                "One pickup consumed or healed the partner"
            );
            o = restore(a, map, &o)?;
            o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
            ensure!(
                s[first].sanity() == 65.,
                "Restored essence could be collected twice"
            );
            p[1 - first].feet = at - Vec3::Z * 32.;
            o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
            ensure!(s[1 - first].sanity() == 65., "Partner lost their essence");
            p = people();
            let mut w = world(map, &o)?;
            for _ in 0..1190 {
                o.pair_advance(FIXED_DT, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            }
            ensure!(
                (0..2).all(|slot| o.visible_essence(Some(slot)).is_none()),
                "Essence cooldown ended early"
            );
            for _ in 0..20 {
                o.pair_advance(FIXED_DT, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            }
            ensure!(
                (0..2).all(|slot| o.visible_essence(Some(slot)).is_some()
                    == (difficulty == Difficulty::Easy)),
                "Personal essence did not follow difficulty/cooldown rules"
            );
            let before = o.snapshot();
            let mut invalid = before.clone();
            invalid["coop"]["items"]["index"][0] = 4.into();
            ensure!(
                o.restore(&invalid, map).is_err() && o.snapshot() == before,
                "Invalid reward index changed saved state"
            );
            println!("PASS paired Hatter {difficulty:?} essence first{first}: personal consumption, view, cooldown and restore");
        }
    }
    for first in 0..2 {
        for (age, amount) in [(0., 50.), (15., 25.), (40., 15.)] {
            let mut o = Encounter::load(a, map)?;
            o.saved.initialized = true;
            let at = vec3(5500., 160., 24.);
            o.saved.drops.push((at, age));
            let mut p = people();
            let mut s = stats();
            for s in &mut s {
                s.set_health(20.)?;
            }
            o.pair_prepare(&mut slots(&mut p, &mut s));
            p[first].feet = at - Vec3::Z * 24.;
            p[1 - first].feet = at + Vec3::X * 100.;
            o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
            ensure!(
                s[first].sanity() == 20. + amount
                    && s[1 - first].sanity() == 20.
                    && o.visible_drops(Some(first)).is_empty()
                    && o.visible_drops(Some(1 - first)).len() == 1,
                "Drop grade or personal ownership changed"
            );
            o = restore(a, map, &o)?;
            o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
            ensure!(
                s[first].sanity() == 20. + amount,
                "Restored drop collected twice"
            );
            p[1 - first].feet = at - Vec3::Z * 24.;
            o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
            ensure!(
                s[1 - first].sanity() == 20. + amount,
                "Drop not available to partner"
            );
            o.pair_drop_age(70.);
            ensure!(o.saved.drops.is_empty(), "Expired reward retained");
            restore(a, map, &o)?;
            println!("PASS paired Hatter drop age{age} first{first}: personal grade, view, consumption and expiry");
        }
    }
    Ok(())
}
fn rewards(a: &mut Assets, map: &Bsp) -> Result<()> {
    for first in 0..2 {
        let mut o = Encounter::load(a, map)?;
        victory(&mut o);
        let mut p = people();
        let mut s = stats();
        o.pair_prepare(&mut slots(&mut p, &mut s));
        p[first].feet = o.data.blade - Vec3::Z * 20.;
        p[1 - first].feet = vec3(5500., 0., 0.);
        o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
        ensure!(
            s[first].collected.contains("hatter2:11")
                && !s[1 - first].collected.contains("hatter2:11")
                && !o.visible_blade(Some(first))
                && o.visible_blade(Some(1 - first))
                && o.saved.phase == Phase::Victory,
            "Blade ownership or Watch gate bypassed"
        );
        s[1 - first].collected.insert(o.data.watch_key.clone());
        o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
        ensure!(
            o.saved.phase == Phase::WatchCat && !o.scripted(),
            "Shared quest progress did not start ordinary Cat speech"
        );
        o = restore(a, map, &o)?;
        p[1 - first].feet = o.data.blade - Vec3::Z * 20.;
        o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
        ensure!(
            s.iter().all(|s| s.collected.contains("hatter2:11"))
                && !o.visible_blade(Some(1 - first)),
            "Partner could not collect their blade after shared progress"
        );
        println!("PASS paired Hatter blade first{first}: personal inventory/view, shared Watch gate and restored partner pickup");
    }
    Ok(())
}
fn deck(map: &Bsp, o: &Encounter, name: &str) -> Result<[Player; 2]> {
    let (k, b) = o
        .data
        .brushes
        .iter()
        .enumerate()
        .find(|(_, b)| b.name == name)
        .context("Missing Hatter lift")?;
    let mid = (b.min + b.max) * 0.5 + o.poses[k].1;
    let z = o.poses[k].1.z
        + if name == "end_plat1" {
            8.03125
        } else {
            400.03125
        };
    let p = [-24., 24.].map(|x| {
        let mut p = Player::new(vec3(mid.x + x, mid.y, z));
        p.grounded = true;
        p
    });
    let w = world(map, o)?;
    ensure!(
        p.iter()
            .all(|p| w.body_clear(p.feet) && crate::recovery::supported(&w, p.feet)),
        "Invalid native {name} deck fixture: {:?}",
        p.each_ref().map(|p| p.feet)
    );
    Ok(p)
}
fn machinery(a: &mut Assets, map: &Bsp) -> Result<()> {
    for name in ["end_plat1", "end_plat2"] {
        for dead in [None, Some(0), Some(1)] {
            let mut o = Encounter::load(a, map)?;
            victory(&mut o);
            o.rebuild(map)?;
            let mut p = deck(map, &o, name)?;
            let mut s = stats();
            if let Some(slot) = dead {
                s[slot].damage(1000.);
            }
            let before = p.each_ref().map(|p| p.feet);
            let mut w = world(map, &o)?;
            for _ in 0..240 {
                o.pair_advance(FIXED_DT, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            }
            let delta = if name == "end_plat1" { 392. } else { -392. };
            for i in 0..2 {
                ensure!(
                    (p[i].feet.z - before[i].z - if dead == Some(i) { 0. } else { delta }).abs()
                        < 0.03,
                    "{name} failed personal carry slot{i}: {:?}",
                    p[i].feet - before[i]
                );
            }
            ensure!(
                p.iter()
                    .enumerate()
                    .filter(|(i, _)| dead != Some(*i))
                    .all(|(_, p)| w.body_clear(p.feet)),
                "Lift embedded a rider"
            );
            restore(a, map, &o)?;
            println!("PASS paired Hatter {name} two-second native carry dead={dead:?}");
        }
    }
    for rider in 0..2 {
        let mut o = Encounter::load(a, map)?;
        victory(&mut o);
        o.rebuild(map)?;
        let mut p = deck(map, &o, "end_plat1")?;
        let mut s = stats();
        let board = p[1 - rider].clone();
        p[1 - rider] = people()[1 - rider].clone();
        let mut w = world(map, &o)?;
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        ensure!(o.saved.lifts == 0., "Lift left before the partner boarded");
        p[1 - rider] = board;
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        ensure!(
            o.saved.lifts == 0.1,
            "Lift did not release after both boarded"
        );
        let before = p.each_ref().map(|p| p.feet);
        let time = o.saved.lifts;
        let blocked = 1 - rider;
        let roof = Collider::box_bounds(
            p[blocked].feet + vec3(-20., -20., 57.),
            p[blocked].feet + vec3(20., 20., 70.),
        );
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[roof])?;
        ensure!(
            o.saved.lifts == time && p.each_ref().map(|p| p.feet) == before,
            "Blocked rider did not roll back the pair"
        );
        println!("PASS paired Hatter boarding and obstruction rollback first{rider}");
    }
    let o = Encounter::load(a, map)?;
    let edge = o.data.points["drawbridge"].translation.y;
    for first in 0..2 {
        let mut feet = [vec3(5664., edge + 40., 0.); 2];
        feet[1 - first].y = edge - 40.;
        ensure!(
            o.coop_boarding_ready("drawbridgeMove", &feet) == Some(false),
            "Bridge cut off the trailing player"
        );
        feet[1 - first].y = edge + 40.;
        ensure!(
            o.coop_boarding_ready("drawbridgeMove", &feet) == Some(true),
            "Bridge did not close after both crossed"
        );
        println!("PASS paired Hatter closing bridge barrier first{first}");
    }
    Ok(())
}
fn compatibility(a: &mut Assets, map: &Bsp) -> Result<()> {
    let mut o = Encounter::load(a, map)?;
    let mut old = o.snapshot();
    old["coop"] = serde_json::json!({"boss_player":null,"wave_players":[null,null]});
    o.restore(&old, map)?;
    let mut p = people();
    let mut s = stats();
    s[0].set_health(31.)?;
    s[1].set_health(47.)?;
    o.pair_prepare(&mut slots(&mut p, &mut s));
    ensure!(
        s[0].sanity() == 31. && s[1].sanity() == 47.,
        "Old co-op arrival save received a new refill"
    );
    o = restore(a, map, &o)?;
    o.pair_prepare(&mut slots(&mut p, &mut s));
    ensure!(
        s[0].sanity() == 31. && s[1].sanity() == 47.,
        "Migrated save repeated its refill"
    );
    println!("PASS paired Hatter legacy combat-only save migration preserves personal health");

    for dead in 0..2 {
        let mut o = Encounter::load(a, map)?;
        o.start_fight();
        o.saved.initialized = true;
        o.saved.essence_live = true;
        let mut p = people();
        let mut s = stats();
        s[dead].damage(1000.);
        o.pair_prepare(&mut slots(&mut p, &mut s));
        p[dead].feet = o.data.blade - Vec3::Z * 20.;
        o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
        ensure!(
            !s[dead].collected.contains("hatter2:11"),
            "Dead player collected the blade"
        );
        let at = o.visible_essence(Some(dead)).unwrap();
        p[dead].feet = at - Vec3::Z * 32.;
        o.saved.drops.push((p[dead].feet + Vec3::Z * 24., 0.));
        o.pair_items().drop_taken.push([false; 2]);
        o.pair_pickups(FIXED_DT, &mut slots(&mut p, &mut s));
        ensure!(
            !s[dead].alive()
                && o.visible_essence(Some(dead)) == Some(at)
                && o.visible_drops(Some(dead)).len() == 1,
            "Dead player consumed or revived from a reward"
        );
        println!("PASS paired Hatter dead slot{dead} cannot consume blade, essence or reinforcement drops");
    }
    let mut o = Encounter::load(a, map)?;
    o.saved.drops = vec![
        (vec3(5500., 160., 24.), 69.999),
        (vec3(5580., 160., 24.), 0.),
    ];
    o.pair_items().drop_taken = vec![[true, false], [false, true]];
    o.pair_drop_age(0.01);
    ensure!(
        o.visible_drops(Some(0)).len() == 1 && o.visible_drops(Some(1)).is_empty(),
        "Expiring a drop reassigned another drop's consumption"
    );
    o = restore(a, map, &o)?;
    let before = o.snapshot();
    for bad in 0..3 {
        let mut invalid = before.clone();
        match bad {
            0 => invalid["coop"]["items"]["drop_taken"] = serde_json::json!([]),
            1 => {
                invalid["coop"]["items"]["ready"][0] = true.into();
                invalid["coop"]["items"]["wait"][0] = 1.into();
            }
            _ => invalid["coop"]["actor"] = 2.into(),
        }
        ensure!(
            o.restore(&invalid, map).is_err() && o.snapshot() == before,
            "Malformed personal state was accepted or mutated owner"
        );
    }
    println!(
        "PASS paired Hatter independent drop expiry and atomic malformed reward/owner rejection"
    );

    for slot in 0..2 {
        let mut o = Encounter::load(a, map)?;
        o.start_fight();
        o.saved.initialized = true;
        o.saved.boss.set(battle::Action::DeathEnd);
        o.saved.boss.health = 0.;
        o.saved.boss.malfunctions = 3;
        o.saved.boss.time = o.data.boss().duration("death_end") - battle::STEP * 0.5;
        o.saved.essence_live = false;
        o.saved.essence_wait = 2.;
        let mut solo = restore(a, map, &o)?;
        let mut p = people();
        let mut s = stats();
        for s in &mut s {
            s.difficulty = Difficulty::Easy;
        }
        o.pair_prepare(&mut slots(&mut p, &mut s));
        let mut w = world(map, &o)?;
        let mut sw = world(map, &solo)?;
        let mut single = p[slot].clone();
        let mut single_stats = s[slot].clone();
        solo.prepare_player(&mut single_stats, &mut single);
        let mut story = Story::load(a, "hatter2");
        o.pair_fight(FIXED_DT, &w, &mut slots(&mut p, &mut s));
        solo.combat(&mut Combat {
            dt: FIXED_DT,
            world: &sw,
            player: &mut single,
            stats: &mut single_stats,
            story: &mut story,
            notarget: true,
            summon: None,
            threatens: &|_| false,
        });
        ensure!(
            o.won() && solo.won() && o.pair_items().wait[slot] == 2.,
            "Victory discarded an active personal cooldown"
        );
        for _ in 0..241 {
            o.pair_advance(FIXED_DT, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            solo.advance(FIXED_DT, map, &mut sw, &mut single, &[])?;
        }
        ensure!(
            solo.saved.essence_live && o.visible_essence(Some(slot)).is_some(),
            "Post-victory cooldown differed from single-player"
        );
        println!("PASS paired Hatter pending healing cooldown survives victory slot{slot} with single-player parity");
    }
    Ok(())
}
fn landings(a: &mut Assets, map: &Bsp) -> Result<()> {
    for rider in 0..2 {
        let mut o = Encounter::load(a, map)?;
        victory(&mut o);
        o.rebuild(map)?;
        let mut p = deck(map, &o, "end_plat1")?;
        let mut s = stats();
        let mut w = world(map, &o)?;
        let centre = (p[0].feet + p[1].feet) * 0.5;
        // Derive a clear, fully supported upper landing from the native map.
        let mut landing = None;
        for y in [128., 192., 256., 64., -128.] {
            for x in [0., -96., 96., -160., 160.] {
                if let Some(mut candidate) =
                    Player::spawn(&w, vec3(centre.x + x, centre.y + y, 448.))
                {
                    // Elevated spawns keep falling; use ordinary gravity to
                    // settle on the authored ledge before testing support.
                    for _ in 0..60 {
                        candidate.tick(&w, crate::movement::Controls::default());
                    }
                    if (candidate.feet.z - 400.).abs() <= 12.
                        && (candidate.feet - centre).truncate().length() <= 256.
                        && crate::recovery::supported(&w, candidate.feet)
                    {
                        landing = Some(candidate);
                        break;
                    }
                }
            }
            if landing.is_some() {
                break;
            }
        }
        p[1 - rider] = landing.context("No native upper waiting landing")?;
        let waiting = p[1 - rider].feet;
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        ensure!(
            o.saved.lifts == 0.1 && p[1 - rider].feet == waiting && p[rider].feet.z > 20.,
            "A partner already on the upper landing prevented catch-up"
        );
        println!("PASS paired Hatter lift catch-up slot{rider} with partner on a supported native upper landing");

        let mut o = Encounter::load(a, map)?;
        victory(&mut o);
        o.rebuild(map)?;
        let mut p = deck(map, &o, "end_plat1")?;
        let mut w = world(map, &o)?;
        let centre = (p[0].feet + p[1].feet) * 0.5;
        p[rider].feet.x = centre.x + if rider == 0 { -55. } else { 55. };
        ensure!(
            (p[0].feet - p[1].feet).truncate().length() >= 35.95,
            "Boarding-edge fixture overlaps its partner"
        );
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        ensure!(
            o.saved.lifts == 0.,
            "Lift accepted a passenger hanging over its boarding edge"
        );
        p[rider].feet.x = centre.x + if rider == 0 { -24. } else { 24. };
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        ensure!(
            o.saved.lifts == 0.1,
            "Full-footprint boarding did not release the lift"
        );
        println!("PASS paired Hatter full-footprint boarding edge slot{rider}");
    }
    let mut o = Encounter::load(a, map)?;
    victory(&mut o);
    o.rebuild(map)?;
    let mut p = people();
    let mut s = stats();
    let mut w = world(map, &o)?;
    ensure!(
        p.iter()
            .all(|p| w.body_clear(p.feet) && crate::recovery::supported(&w, p.feet)),
        "Empty-lift fixture is not on clear native entrance footing"
    );
    o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
    ensure!(
        o.saved.lifts == 0.,
        "Empty lower deck was unavailable for boarding"
    );
    o.saved.lifts = 4.;
    o.rebuild(map)?;
    w.set_dynamic(o.colliders());
    o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
    ensure!(
        o.saved.lifts == 4.1,
        "Empty upper deck failed to return: phase {}, clear {:?}",
        o.saved.lifts,
        p.each_ref().map(|p| w.body_clear(p.feet))
    );
    println!("PASS paired Hatter empty lower boarding stop and upper return");
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/hatter2.bsp")?)?;
    clocks(a, &map)?;
    healing(a, &map)?;
    rewards(a, &map)?;
    machinery(a, &map)?;
    compatibility(a, &map)?;
    landings(a, &map)?;
    println!(
        "PASS paired Hatter world/reward components; full internet traversal remains unverified"
    );
    Ok(())
}
