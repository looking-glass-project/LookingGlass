//! Staged, headless checks with the supplied rigs. Not a paired route or internet check.
use super::*;
use crate::level::CoopPlayer;

fn slots<'a>(p: &'a mut [Player; 2], s: &'a mut [Stats; 2]) -> [CoopPlayer<'a>; 2] {
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
fn stats(kind: Kind) -> [Stats; 2] {
    std::array::from_fn(|_| Stats::for_level(kind.map(), None))
}
fn at_eye(eye: Vec3) -> Player {
    Player::new(eye - Vec3::Z * crate::movement::EYE_HEIGHT)
}
fn fight(a: &mut Assets, map: &Bsp, kind: Kind) -> Result<Encounter> {
    let mut o = Encounter::load(a, map, kind)?;
    let world = World::from_bsp(map)?;
    let mut p = Player::new(Vec3::ZERO);
    o.finish_intro(&world, &mut p)?;
    Ok(o)
}
fn restored(a: &mut Assets, map: &Bsp, o: &Encounter) -> Result<Encounter> {
    let bytes = crate::coop::codec::encode(&serde_json::to_vec(&o.snapshot())?)?;
    let mut copy = Encounter::load(a, map, o.saved.kind)?;
    copy.restore(
        &serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,
        map,
    )?;
    ensure!(
        copy.snapshot() == o.snapshot(),
        "Paired Jabberwock restore changed state"
    );
    Ok(copy)
}
fn clock_checks(a: &mut Assets, map: &Bsp, kind: Kind) -> Result<()> {
    for hz in [30, 60, 144] {
        let mut pair = fight(a, map, kind)?;
        let mut solo = fight(a, map, kind)?;
        let world = World::from_bsp(map)?;
        let mut p = [vec3(10000., 10000., 0.), vec3(10100., 10000., 0.)].map(Player::new);
        let mut s = stats(kind);
        for s in &mut s {
            s.notarget = true;
        }
        let before = pair.snapshot();
        for dt in [0., -1., f32::NAN, f32::INFINITY] {
            pair.coop_combat_roster(dt, &world, &mut slots(&mut p, &mut s));
            pair.pair_resources(dt, &mut slots(&mut p, &mut s));
        }
        ensure!(
            pair.snapshot() == before,
            "Invalid/paused time changed encounter"
        );
        let mut player = p[0].clone();
        let mut single = s[0].clone();
        let mut story = Story::load(a, kind.map());
        for n in 0..hz * if kind == Kind::Lair { 91 } else { 12 } {
            let dt = 1. / hz as f32;
            pair.coop_combat_roster(dt, &world, &mut slots(&mut p, &mut s));
            solo.combat(&mut Combat {
                dt,
                world: &world,
                player: &mut player,
                stats: &mut single,
                story: &mut story,
                notarget: true,
                summon: None,
                threatens: &|_| false,
            });
            if n % hz == 0 {
                let mut actual = pair.snapshot();
                actual.as_object_mut().unwrap().remove("coop");
                ensure!(
                    actual == solo.snapshot(),
                    "{} {hz}Hz combat differs at tick {n}",
                    kind.map()
                );
            }
        }
        if kind == Kind::Lair {
            ensure!(
                pair.saved.ticks == 10800 && pair.saved.waves.len() == 6,
                "Survival schedule changed"
            );
        } else {
            ensure!(pair.saved.boss.flying, "Grounds takeoff did not run");
        }
        let mut copy = restored(a, map, &pair)?;
        let mut p2 = p.clone();
        let mut s2 = s.clone();
        for _ in 0..120 {
            pair.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
            copy.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p2, &mut s2));
        }
        ensure!(
            copy.snapshot() == pair.snapshot(),
            "Compressed continuation differs"
        );
        let before = pair.snapshot();
        for path in ["boss_player", "wave_players", "essence", "wait"] {
            let mut bad = before.clone();
            bad["coop"][path] = match path {
                "boss_player" => 2.into(),
                "wave_players" => serde_json::json!([null, null, null, null, null, 2]),
                "essence" => serde_json::json!([0, 3]),
                _ => serde_json::json!([0., 8.1]),
            };
            ensure!(
                pair.restore(&bad, map).is_err() && pair.snapshot() == before,
                "Invalid {path} mutated encounter"
            );
        }
        for s in &mut s {
            s.damage(1000.);
        }
        pair.coop_combat_roster(0.1, &world, &mut slots(&mut p, &mut s));
        pair.pair_resources(0.1, &mut slots(&mut p, &mut s));
        ensure!(pair.snapshot() == before, "Both dead advanced clock");
        println!("PASS {} paired {hz}Hz solo combat parity, pause, both dead, compressed continuation and corrupt-save rejection", kind.map());
    }
    Ok(())
}
fn targets(a: &mut Assets, map: &Bsp, kind: Kind) -> Result<()> {
    for near in 0..2 {
        for mode in ["visible", "dead", "hidden", "occluded"] {
            let mut o = fight(a, map, kind)?;
            o.saved.boss.set(battle::Action::Ready);
            let at = o.saved.boss.at;
            let mut p = [at + vec3(0., 500., 0.); 2].map(Player::new);
            p[near] = Player::new(at + vec3(110., 0., 0.));
            let mut s = stats(kind);
            if mode == "dead" {
                s[near].damage(1000.);
            }
            if mode == "hidden" {
                s[near].notarget = true;
            }
            let world = if mode == "occluded" {
                World::fixture(&[(at + vec3(40., -30., 0.), at + vec3(70., 30., 200.))])
            } else {
                World::fixture(&[])
            };
            o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
            let expected = if mode == "visible" { near } else { 1 - near };
            ensure!(
                o.saved.coop.as_ref().unwrap().boss_player == Some(expected),
                "{} wrong {mode} target with slot {near} nearest",
                kind.map()
            );
            let mut copy = restored(a, map, &o)?;
            let mut p2 = p.clone();
            let mut s2 = s.clone();
            o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
            copy.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p2, &mut s2));
            ensure!(
                copy.snapshot() == o.snapshot(),
                "Selected target lost on restore"
            );
            println!(
                "PASS {} {mode} target and restore, nearest slot {near}",
                kind.map()
            );
        }
        let mut o = fight(a, map, kind)?;
        o.saved.boss.set(battle::Action::Ready);
        let at = o.saved.boss.at;
        let mut p = [at + vec3(500., 0., 0.); 2].map(Player::new);
        p[near] = Player::new(at + vec3(110., 0., 0.));
        let mut s = stats(kind);
        let world = World::fixture(&[]);
        o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
        o.saved.boss.set(battle::Action::Spit);
        p.swap(0, 1);
        o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
        ensure!(
            o.saved.coop.as_ref().unwrap().boss_player == Some(near),
            "Committed attack changed target"
        );
        s[near].damage(1000.);
        o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
        ensure!(
            o.saved.coop.as_ref().unwrap().boss_player == Some(1 - near),
            "Boss retained dead target"
        );
        println!(
            "PASS {} committed slot {near}, then survivor retarget",
            kind.map()
        );
    }
    Ok(())
}
fn attacks(a: &mut Assets, map: &Bsp, kind: Kind) -> Result<()> {
    let world = World::fixture(&[]);
    for victim in 0..2 {
        for cover in [false, true] {
            let mut o = fight(a, map, kind)?;
            o.saved.boss.set(battle::Action::Punch);
            o.saved.boss.yaw = 0.;
            o.saved.boss.time = 4. * o.data.boss().frame(o.saved.boss.clip()) - battle::STEP * 0.5;
            let at = o.saved.boss.at;
            let mut p = [at + vec3(-500., 0., 0.); 2].map(Player::new);
            p[victim] = Player::new(at + vec3(110., 0., 0.));
            let mut s = stats(kind);
            let w = if cover {
                World::fixture(&[(at + vec3(40., -50., 0.), at + vec3(60., 50., 200.))])
            } else {
                World::fixture(&[])
            };
            let hit = o.coop_combat_roster(battle::STEP, &w, &mut slots(&mut p, &mut s));
            ensure!(
                hit.damage == 0.
                    && hit.players[1 - victim].damage == 0.
                    && hit.players[victim].damage == if cover { 0. } else { 5. },
                "Melee routing/cover differs"
            );
            println!("PASS {} melee slot {victim}, cover={cover}", kind.map());
        }
        for spiral in [false, true] {
            let mut o = fight(a, map, kind)?;
            o.saved.boss.set(battle::Action::Ready);
            let at = o.saved.boss.at + Vec3::Z * 400.;
            let mut p = [at + vec3(0., 200., 20.); 2].map(at_eye);
            p[victim] = at_eye(at + vec3(60., 0., 20.));
            let mut s = stats(kind);
            // A hidden body remains physically hittable; it does not attract the boss.
            s[victim].notarget = true;
            o.saved.boss.shots.push(battle::Shot {
                at,
                start: at,
                velocity: Vec3::X * if spiral { 1000. } else { 400. },
                age: 0.,
                ended: None,
                spiral,
                bounced: false,
                id: 1,
            });
            let mut amounts = [0.; 2];
            let mut explosions = 0;
            for _ in 0..30 {
                let out = o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
                ensure!(out.damage == 0., "Attack leaked into solo damage");
                for i in 0..2 {
                    amounts[i] += out.players[i].damage;
                }
                explosions += out
                    .spatial_sounds
                    .iter()
                    .filter(|(n, _)| *n == "sound/weapon/staff/explode1.wav")
                    .count();
            }
            ensure!(
                amounts[victim] == if spiral { 150. } else { 5. },
                "Direct hit repeated or missing: {amounts:?}"
            );
            ensure!(
                if spiral {
                    amounts[1 - victim] > 0. && amounts[1 - victim] < 100. && explosions == 1
                } else {
                    amounts[1 - victim] == 0. && explosions == 0
                },
                "Spiral splash/deduplication differs"
            );
            println!("PASS {} {} hits hidden slot {victim}, physical body routing and splash deduplication", kind.map(), if spiral { "spiral" } else { "breath" });
        }
        let mut o = fight(a, map, kind)?;
        o.saved.boss.set(battle::Action::Beam);
        o.saved.boss.hold = 2.;
        let start = o
            .data
            .boss()
            .tag(
                "tag_eye_emitter",
                o.saved.boss.clip(),
                battle::STEP,
                o.saved.boss.pose(),
                o.saved.boss.loops(),
            )
            .translation;
        o.saved.boss.aim = start + Vec3::X * 500.;
        let mut p = [start + vec3(250., 0., 20.); 2].map(at_eye);
        p[victim] = at_eye(start + vec3(100., 0., 20.));
        let mut s = stats(kind);
        let out = o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
        ensure!(
            out.damage == 0.
                && out.players[victim].damage == 5.
                && out.players[1 - victim].damage == 0.,
            "Beam did not stop on first living body"
        );
        println!("PASS {} beam intercepted by slot {victim}", kind.map());
    }
    let mut o = fight(a, map, kind)?;
    o.saved.boss.set(battle::Action::Dive);
    o.saved.boss.flying = true;
    o.saved.boss.landed = false;
    let centre = o.saved.boss.target(0).center;
    o.saved.boss.aim = o.saved.boss.at + Vec3::X * 100.;
    let mut p = [centre + vec3(50., 0., 20.), centre + vec3(0., 50., 20.)].map(at_eye);
    let mut s = stats(kind);
    let out = o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
    ensure!(
        out.damage == 0. && out.players.iter().all(|p| p.damage == 25.),
        "Dive omitted a contacted body"
    );
    let out = o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
    ensure!(
        out.players.iter().all(|p| p.damage == 0.),
        "Dive contact repeated"
    );
    println!("PASS {} dive contacts both bodies once", kind.map());
    Ok(())
}
fn resources(a: &mut Assets, map: &Bsp, kind: Kind) -> Result<()> {
    for first in 0..2 {
        let mut o = fight(a, map, kind)?;
        o.saved.essence = 1;
        o.saved.essence_wait = 3.25;
        let solo = o.snapshot();
        let mut imported = Encounter::load(a, map, kind)?;
        imported.restore(&solo, map)?;
        let mut p = [vec3(10000., 10000., 0.); 2].map(Player::new);
        let mut s = stats(kind);
        for s in &mut s {
            s.damage(60.);
        }
        imported.coop_prepare(&mut slots(&mut p, &mut s));
        let state = imported.saved.coop.as_ref().unwrap();
        ensure!(
            state.essence == [1; 2] && state.wait == [3.25; 2],
            "Solo import reset healing cycle"
        );
        for _ in 0..40 {
            imported.pair_resources(0.1, &mut slots(&mut p, &mut s));
        }
        let at = imported
            .personal_essence(Some(first))
            .context("Healing never returned")?;
        p[first] = Player::new(at - Vec3::Z * 32.);
        imported.pair_resources(0.1, &mut slots(&mut p, &mut s));
        ensure!(
            s[first].sanity() == 90.
                && s[1 - first].sanity() == 40.
                && imported.personal_essence(Some(first)).is_none()
                && imported.personal_essence(Some(1 - first)) == Some(at),
            "Personal collection consumed partner's essence"
        );
        let mut copy = restored(a, map, &imported)?;
        let mut p2 = p.clone();
        let mut s2 = s.clone();
        p[1 - first] = Player::new(at - Vec3::Z * 32.);
        p2[1 - first] = p[1 - first].clone();
        imported.pair_resources(0.1, &mut slots(&mut p, &mut s));
        copy.pair_resources(0.1, &mut slots(&mut p2, &mut s2));
        ensure!(
            s[1 - first].sanity() == 90.
                && s2[1 - first].sanity() == 90.
                && imported.snapshot() == copy.snapshot(),
            "Restored partner collection differs"
        );
        p = [vec3(10000., 10000., 0.); 2].map(Player::new);
        let before = imported.snapshot();
        imported.pair_resources(0., &mut slots(&mut p, &mut s));
        ensure!(
            imported.snapshot() == before,
            "Pause advanced personal healing"
        );
        s[first].damage(1000.);
        for _ in 0..81 {
            imported.pair_resources(0.1, &mut slots(&mut p, &mut s));
        }
        ensure!(
            imported.saved.coop.as_ref().unwrap().wait == [0.; 2],
            "Living survivor failed to advance both cooldowns"
        );
        let at = imported.personal_essence(Some(first)).unwrap();
        p[first] = Player::new(at - Vec3::Z * 32.);
        imported.pair_resources(0.1, &mut slots(&mut p, &mut s));
        ensure!(
            !s[first].alive() && imported.saved.coop.as_ref().unwrap().wait[first] == 0.,
            "Dead player consumed healing"
        );
        println!("PASS {} individual essence slot {first} first, solo import, compressed restore, pause and dead exclusion", kind.map());
    }
    Ok(())
}
fn watch(a: &mut Assets, map: &Bsp, kind: Kind) -> Result<()> {
    for holder in 0..2 {
        let mut o = fight(a, map, kind)?;
        o.saved.boss.set(battle::Action::Ready);
        if kind == Kind::Lair {
            o.saved.ticks = 3600;
            o.wave();
        }
        let at = o.saved.boss.at + Vec3::Z * 400.;
        for (id, spiral) in [(1, false), (2, true)] {
            o.saved.boss.shots.push(battle::Shot {
                at,
                start: at,
                velocity: Vec3::X * 1000.,
                age: 0.,
                ended: None,
                spiral,
                bounced: false,
                id,
            });
        }
        let mut p = [vec3(10000., 10000., 0.); 2].map(Player::new);
        let mut s = stats(kind);
        s[holder].powers.stopped = 1.;
        o.coop_combat_roster(
            battle::STEP,
            &World::fixture(&[]),
            &mut slots(&mut p, &mut s),
        );
        ensure!(
            o.saved.boss.time > 0.
                && o.saved.boss.shots[0].age > 0.
                && o.saved.boss.shots[1].age == 0.
                && o.saved.waves.iter().all(|w| w.launch == 0.),
            "Watch changed boss immunity or missed paired waves/spirals"
        );
        s[holder].damage(1000.);
        o.coop_combat_roster(
            battle::STEP,
            &World::fixture(&[]),
            &mut slots(&mut p, &mut s),
        );
        ensure!(
            o.saved.boss.shots[1].age > 0. && o.saved.waves.iter().all(|w| w.launch > 0.),
            "Dead Watch holder froze survivor's combat"
        );
        println!("PASS {} Watch slot {holder}: boss/breath immune, spiral/waves frozen, dead holder excluded", kind.map());
    }
    Ok(())
}
fn waves(a: &mut Assets, map: &Bsp) -> Result<()> {
    let kind = Kind::Lair;
    let world = World::fixture(&[]);
    for near in 0..2 {
        let mut o = fight(a, map, kind)?;
        o.saved.ticks = 7200;
        o.wave();
        o.wave();
        let at = o.saved.waves[0].to;
        for w in &mut o.saved.waves {
            w.launch = 1.5;
            w.actor.feet = at;
        }
        let mut p = [at + vec3(500., 0., 0.); 2].map(Player::new);
        p[near] = Player::new(at + vec3(110., 0., 0.));
        let mut s = stats(kind);
        o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
        ensure!(
            o.saved.coop.as_ref().unwrap().wave_players == [Some(near); 6],
            "Wave target slots missing"
        );
        let mut copy = restored(a, map, &o)?;
        let mut p2 = p.clone();
        let mut s2 = s.clone();
        for _ in 0..60 {
            o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
            copy.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p2, &mut s2));
        }
        ensure!(
            o.snapshot() == copy.snapshot(),
            "Wave targets lost on compressed restore"
        );
        s[near].damage(1000.);
        o.coop_combat_roster(battle::STEP, &world, &mut slots(&mut p, &mut s));
        ensure!(
            o.saved.coop.as_ref().unwrap().wave_players == [Some(1 - near); 6],
            "Wave retained dead target"
        );
        println!("PASS jlair2 all six wave targets slot {near}, compressed continuation and survivor retarget");
    }
    Ok(())
}
pub(crate) fn check(a: &mut Assets) -> Result<()> {
    for kind in [Kind::Lair, Kind::Grounds] {
        let map = Bsp::parse(&a.read(&format!("maps/{}.bsp", kind.map()))?)?;
        clock_checks(a, &map, kind)?;
        targets(a, &map, kind)?;
        attacks(a, &map, kind)?;
        resources(a, &map, kind)?;
        watch(a, &map, kind)?;
        if kind == Kind::Lair {
            waves(a, &map)?;
        }
    }
    Ok(())
}
