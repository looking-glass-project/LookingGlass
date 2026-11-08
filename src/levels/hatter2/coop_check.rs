//! Staged combat components. This is not paired level traversal evidence.
use super::*;
use crate::{combat::PlayerTarget, level::CoopPlayer};

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
fn stats() -> [Stats; 2] {
    std::array::from_fn(|_| Stats::for_level("hatter2", None))
}
fn bodies(at: [Vec3; 2]) -> [Player; 2] {
    at.map(Player::new)
}
fn fight(a: &mut Assets, map: &Bsp) -> Result<Encounter> {
    let mut o = Encounter::load(a, map)?;
    o.start_fight();
    o.saved.cycle = 300;
    o.saved.boss.at = vec3(5664., 160., 0.);
    o.saved.boss.set(battle::Action::Ready);
    Ok(o)
}
fn restored(a: &mut Assets, map: &Bsp, o: &Encounter) -> Result<Encounter> {
    let bytes = crate::coop::codec::encode(&serde_json::to_vec(&o.snapshot())?)?;
    let mut copy = Encounter::load(a, map)?;
    copy.restore(
        &serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,
        map,
    )?;
    ensure!(
        copy.snapshot() == o.snapshot(),
        "Paired Hatter restore changed state"
    );
    Ok(copy)
}
fn clock_checks(a: &mut Assets, map: &Bsp) -> Result<()> {
    for hz in [30, 60, 144] {
        let mut pair = fight(a, map)?;
        let mut solo = fight(a, map)?;
        let world = World::from_bsp(map)?;
        let mut p = bodies([vec3(10000., 10000., 0.), vec3(10100., 10000., 0.)]);
        let mut s = stats();
        s[0].notarget = true;
        s[1].notarget = true;
        let before = pair.snapshot();
        for dt in [0., -1., f32::NAN] {
            pair.pair_fight(dt, &world, &mut slots(&mut p, &mut s));
        }
        ensure!(
            pair.snapshot() == before,
            "Paused/invalid Hatter combat advanced"
        );
        let mut player = p[0].clone();
        let mut single = s[0].clone();
        let mut story = Story::load(a, "hatter2");
        for n in 0..hz * 96 {
            let dt = 1. / hz as f32;
            pair.pair_fight(dt, &world, &mut slots(&mut p, &mut s));
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
                    "Paired combat clock diverged at {hz}Hz tick{n}"
                );
            }
        }
        pair = restored(a, map, &pair)?;
        let before = pair.snapshot();
        let mut invalid = before.clone();
        invalid["coop"]["boss_player"] = 2.into();
        ensure!(
            pair.restore(&invalid, map).is_err() && pair.snapshot() == before,
            "Invalid target mutated Hatter state"
        );
        println!(
            "PASS paired Hatter {hz}Hz full combat cycle parity, pause and compressed restore"
        );
    }
    Ok(())
}
fn target_checks(a: &mut Assets, map: &Bsp) -> Result<()> {
    let world = World::from_bsp(map)?;
    for near in 0..2 {
        for mode in ["visible", "dead", "hidden"] {
            let mut o = fight(a, map)?;
            let at = o.saved.boss.at;
            let mut p = bodies([at + vec3(500., 0., 0.), at + vec3(500., 80., 0.)]);
            p[near] = Player::new(at + vec3(110., 0., 0.));
            let mut s = stats();
            if mode == "dead" {
                s[near].damage(1000.);
            } else if mode == "hidden" {
                s[near].notarget = true;
            }
            o.pair_fight(battle::STEP, &world, &mut slots(&mut p, &mut s));
            let expected = if mode == "visible" { near } else { 1 - near };
            ensure!(
                o.saved.coop.as_ref().unwrap().boss_player == Some(expected),
                "Wrong {mode} Hatter target near slot{near}"
            );
            let mut copy = restored(a, map, &o)?;
            let mut p2 = p.clone();
            let mut s2 = s.clone();
            o.pair_fight(battle::STEP, &world, &mut slots(&mut p, &mut s));
            copy.pair_fight(battle::STEP, &world, &mut slots(&mut p2, &mut s2));
            ensure!(
                o.snapshot() == copy.snapshot(),
                "Saved target changed continuation"
            );
            println!("PASS paired Hatter {mode} nearest player slot{near} and saved continuation");
        }
    }
    for victim in 0..2 {
        for behind in [false, true] {
            let mut o = fight(a, map)?;
            let at = o.saved.boss.at;
            let mut p = bodies([at + vec3(600., 0., 0.), at + vec3(600., 80., 0.)]);
            p[victim] = Player::new(at + vec3(if behind { -110. } else { 110. }, 0., 0.));
            let mut s = stats();
            // The map starts the Hatter facing +Y; this staged swing faces +X.
            o.saved.boss.yaw = 0.;
            o.saved.boss.set(battle::Action::Cane);
            let mut out = Feedback::default();
            for _ in 0..(o.data.boss().duration("attack_cane") * 120.).floor() as usize {
                out.append(o.pair_fight(battle::STEP, &world, &mut slots(&mut p, &mut s)));
            }
            ensure!(
                out.players[victim].damage == if behind { 0. } else { 15. }
                    && out.players[1 - victim].damage == 0.
                    && out.damage == 0.,
                "Hatter melee slot{victim} behind={behind}: damage={:?}, legacy={}",
                out.players.each_ref().map(|p| p.damage),
                out.damage
            );
            println!("PASS paired Hatter personal cane impact slot{victim} behind={behind}");
        }
    }
    for hidden_slot in 0..2 {
        let mut o = fight(a, map)?;
        let at = o.saved.boss.at;
        let mut p = bodies([at + vec3(0., 250., 0.); 2]);
        p[hidden_slot] = Player::new(at + vec3(110., 0., 0.));
        let mut s = stats();
        let mut blocked = World::from_bsp(map)?;
        blocked.set_dynamic(vec![Collider::box_bounds(
            at + vec3(40., -32., 0.),
            at + vec3(70., 32., 240.),
        )]);
        o.pair_fight(battle::STEP, &blocked, &mut slots(&mut p, &mut s));
        ensure!(
            o.saved.coop.as_ref().unwrap().boss_player == Some(1 - hidden_slot),
            "Hatter preferred an occluded nearer player"
        );
        o.pair_fight(battle::STEP, &world, &mut slots(&mut p, &mut s));
        ensure!(
            o.saved.coop.as_ref().unwrap().boss_player == Some(hidden_slot),
            "Hatter did not retarget the visible nearer player"
        );
        println!(
            "PASS paired Hatter sight obstruction and nearest-visible retarget slot{hidden_slot}"
        );
    }
    Ok(())
}
fn projectiles(a: &mut Assets, map: &Bsp) -> Result<()> {
    let world = World::from_bsp(map)?;
    for hit_slot in 0..2 {
        for syringe in [false, true] {
            let mut o = fight(a, map)?;
            let at = vec3(5664., 160., 28.);
            let mut p = bodies([at + vec3(500., 0., -28.), at + vec3(500., 80., -28.)]);
            p[hit_slot] = Player::new(at + vec3(50., 0., -28.));
            let s = stats();
            o.saved.boss.opponents.players = Some(std::array::from_fn(|i| PlayerTarget {
                eye: p[i].eye(),
                alive: true,
                hidden: false,
            }));
            // The projectile was aimed at the other Alice. The nearer body intercepts it.
            o.saved.boss.shots.push(battle::Shot {
                at,
                velocity: Vec3::X * 500.,
                age: 0.,
                syringe,
                id: 1,
                ended: None,
                player: Some(1 - hit_slot),
            });
            let mut out = Feedback::default();
            for _ in 0..30 {
                o.saved
                    .boss
                    .projectile_step(battle::STEP, &world, p[0].eye(), false, &mut out);
            }
            let damage = if syringe { 20. } else { 10. };
            ensure!(
                out.players[hit_slot].damage == damage
                    && out.players[1 - hit_slot].damage == 0.
                    && out.damage == 0.,
                "Projectile failed personal interception"
            );
            ensure!(
                s.iter().all(|s| s.sanity() == 100.),
                "Diagnostic directly applied player damage"
            );
            o = restored(a, map, &o)?;
            let before = o.snapshot();
            let mut invalid = before.clone();
            invalid["boss"]["shots"][0]["player"] = 2.into();
            ensure!(
                o.restore(&invalid, map).is_err() && o.snapshot() == before,
                "Invalid projectile target accepted"
            );
            println!("PASS paired Hatter {} intercepted by slot{hit_slot}, effects and target restoration",if syringe {"syringe"}else{"cup"});
        }
    }
    Ok(())
}
fn projectile_contracts(a: &mut Assets, map: &Bsp) -> Result<()> {
    let world = World::from_bsp(map)?;
    let at = vec3(5664., 160., 300.);
    for target in 0..2 {
        for dead in [false, true] {
            let mut o = fight(a, map)?;
            let mut roster = [PlayerTarget {
                eye: at + vec3(0., 100., 20.),
                alive: true,
                hidden: false,
            }; 2];
            roster[target] = PlayerTarget {
                eye: at + vec3(0., -500., 20.),
                alive: !dead,
                hidden: false,
            };
            o.saved.boss.opponents.players = Some(roster);
            o.saved.boss.shots.push(battle::Shot {
                at,
                velocity: Vec3::X * 500.,
                age: 0.,
                syringe: true,
                id: 1,
                ended: None,
                player: Some(target),
            });
            let mut copy = restored(a, map, &o)?;
            copy.saved.boss.opponents.players = Some(roster);
            for boss in [&mut o.saved.boss, &mut copy.saved.boss] {
                boss.projectile_step(
                    battle::STEP,
                    &world,
                    roster[0].eye,
                    false,
                    &mut Feedback::default(),
                );
            }
            let velocity = o.saved.boss.shots[0].velocity;
            ensure!(
                if dead {
                    velocity == Vec3::X * 500.
                } else {
                    velocity.y < 0.
                },
                "Syringe abandoned its saved target slot{target} dead={dead}"
            );
            ensure!(
                o.snapshot() == copy.snapshot(),
                "Restored syringe changed its next trajectory"
            );
            println!("PASS paired Hatter saved syringe steering slot{target} dead={dead}");
        }
        let mut o = fight(a, map)?;
        let mut p = bodies([at + vec3(1000., 0., 0.), at + vec3(1000., 80., 0.)]);
        let mut s = stats();
        s[target].powers.stopped = 1.;
        for syringe in [false, true] {
            o.saved.boss.shots.push(battle::Shot {
                at,
                velocity: Vec3::X * 500.,
                age: 0.,
                syringe,
                id: u32::from(syringe),
                ended: None,
                player: Some(target),
            });
        }
        o.pair_fight(battle::STEP, &world, &mut slots(&mut p, &mut s));
        ensure!(
            o.saved.cycle == 300
                && o.saved.boss.shots[0].age == 0.
                && o.saved.boss.shots[1].age == battle::STEP,
            "Watch did not preserve the shared clock/cup/syringe rules for slot{target}"
        );
        println!("PASS paired Hatter Watch clock and projectile behavior slot{target}");
    }
    Ok(())
}
fn hazards_and_waves(a: &mut Assets, map: &Bsp) -> Result<()> {
    let world = World::from_bsp(map)?;
    for dead in [None, Some(0), Some(1)] {
        let mut o = fight(a, map)?;
        o.saved.cycle = 9000;
        let at = o.data.points["hatter_splash"].translation;
        let mut p = bodies([at + vec3(-50., 0., 0.), at + vec3(50., 0., 0.)]);
        let mut s = stats();
        if let Some(slot) = dead {
            s[slot].damage(1000.);
        }
        let out = o.pair_fight(battle::STEP, &world, &mut slots(&mut p, &mut s));
        for slot in 0..2 {
            ensure!(
                out.players[slot].damage == if dead == Some(slot) { 0. } else { 200. },
                "Tea hit the wrong living roster"
            );
        }
        ensure!(
            o.saved.cycle == 9001 && out.damage == 0.,
            "Tea advanced twice or damaged legacy Alice"
        );
        println!("PASS paired Hatter tea splash living roster dead={dead:?}");
    }
    let mut o = fight(a, map)?;
    o.saved.cycle = 6024;
    let mut p = bodies([vec3(5550., 160., 0.), vec3(5778., 160., 0.)]);
    let mut s = stats();
    o.pair_fight(battle::STEP, &world, &mut slots(&mut p, &mut s));
    ensure!(o.saved.waves.len() == 2, "Missing shared wave");
    for _ in 0..240 {
        o.pair_fight(battle::STEP, &world, &mut slots(&mut p, &mut s));
    }
    ensure!(
        o.saved
            .coop
            .as_ref()
            .unwrap()
            .wave_players
            .iter()
            .all(Option::is_some),
        "Reinforcements ignore both players"
    );
    o = restored(a, map, &o)?;
    o.hit(Hit {
        id: BASE,
        damage: 3000.,
        kind: crate::combat::DamageKind::Knife,
        knockback: Vec3::ZERO,
    });
    for _ in 0..120 * 40 {
        o.pair_fight(battle::STEP, &world, &mut slots(&mut p, &mut s));
        if o.won() {
            break;
        }
    }
    ensure!(
        o.won() && o.saved.boss.malfunctions == 3 && o.saved.waves.is_empty(),
        "Paired Hatter victory failed"
    );
    restored(a, map, &o)?;
    println!("PASS paired Hatter reinforcements, saved targets and original three-stage death");
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/hatter2.bsp")?)?;
    clock_checks(a, &map)?;
    target_checks(a, &map)?;
    projectiles(a, &map)?;
    projectile_contracts(a, &map)?;
    hazards_and_waves(a, &map)?;
    println!("PASS paired Hatter combat components; full co-op visit remains unverified");
    Ok(())
}
