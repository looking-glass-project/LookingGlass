//! Native brush/reward components from staged prerequisites, not a full paired route.
use super::*;
use crate::level::CoopPlayer;
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
pub(super) fn victory(o: &mut Encounter) {
    if o.saved.kind == Kind::Lair {
        o.saved.ticks = 10800;
        o.saved.wave_count = 2;
        o.saved.waves.clear();
        o.phase(Phase::Reward);
    } else {
        o.saved.boss.hit(
            Hit {
                id: 0,
                damage: 2000.,
                kind: crate::combat::DamageKind::Other,
                knockback: Vec3::ZERO,
            },
            Kind::Grounds,
        );
        o.phase(Phase::Bridge);
    }
}
pub(super) fn world(map: &Bsp, o: &Encounter) -> Result<World> {
    let mut w = World::from_bsp(map)?;
    w.set_dynamic(o.colliders());
    Ok(w)
}
pub(super) fn floor(w: &World, at: Vec3) -> Result<Player> {
    let top = at + Vec3::Z * 256.;
    let bottom = at - Vec3::Z * 1024.;
    let tr = w.body_trace(top, bottom);
    ensure!(
        !tr.start_solid && tr.fraction < 1. && tr.normal.z > 0.65,
        "No native fixture floor near {at:?}"
    );
    let mut p = Player::new(top.lerp(bottom, tr.fraction));
    p.grounded = true;
    p.ground_normal = tr.normal;
    ensure!(w.body_clear(p.feet), "Obstructed fixture floor");
    Ok(p)
}
pub(super) fn pair(w: &World, at: Vec3) -> Result<[Player; 2]> {
    let a = floor(w, at)?;
    let roster =
        crate::coop::world::Roster::new(w, a.eye(), 0., Stats::for_level("grounds1", None))?;
    Ok(roster.players.map(|b| b.player))
}
fn restore(a: &mut Assets, map: &Bsp, o: &Encounter) -> Result<Encounter> {
    let bytes = crate::coop::codec::encode(&serde_json::to_vec(&o.snapshot())?)?;
    let mut copy = Encounter::load(a, map, o.saved.kind)?;
    copy.restore(
        &serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,
        map,
    )?;
    ensure!(
        copy.snapshot() == o.snapshot(),
        "Paired world changed on restore"
    );
    Ok(copy)
}
fn clocks(a: &mut Assets, map: &Bsp, kind: Kind) -> Result<()> {
    for hz in [30, 60, 144] {
        let mut o = Encounter::load(a, map, kind)?;
        victory(&mut o);
        o.rebuild(map)?;
        let mut solo = Encounter::load(a, map, kind)?;
        solo.restore(&o.snapshot(), map)?;
        let mut w = world(map, &o)?;
        let mut sw = world(map, &solo)?;
        let at = o.data.points[if kind == Kind::Lair {
            "alice_dead_pos1"
        } else {
            "alice_end1"
        }]
        .translation;
        let mut p = pair(&w, at)?;
        let mut single = p[0].clone();
        let mut s = std::array::from_fn(|_| Stats::for_level(kind.map(), None));
        o.coop_prepare(&mut slots(&mut p, &mut s));
        let before = o.snapshot();
        let bodies = serde_json::to_value(&p)?;
        for dt in [0., -1., f32::NAN] {
            o.pair_advance(dt, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        }
        ensure!(
            o.snapshot() == before && serde_json::to_value(&p)? == bodies,
            "Paused world moved"
        );
        for _ in 0..hz * if kind == Kind::Grounds { 14 } else { 2 } {
            let dt = 1. / hz as f32;
            o.pair_advance(dt, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            solo.advance(dt, map, &mut sw, &mut single, &[])?;
            let mut actual = o.snapshot();
            actual.as_object_mut().unwrap().remove("coop");
            ensure!(
                actual == solo.snapshot() && o.transforms() == solo.transforms(),
                "{} {hz}Hz world differs",
                kind.map()
            );
        }
        restore(a, map, &o)?;
        println!(
            "PASS {} {hz}Hz shared world/solo parity, pause and compressed restore",
            kind.map()
        );
    }
    Ok(())
}
fn rewards(a: &mut Assets, map: &Bsp) -> Result<()> {
    for first in 0..2 {
        for dead in [false, true] {
            let mut o = Encounter::load(a, map, Kind::Lair)?;
            victory(&mut o);
            let mut w = world(map, &o)?;
            let at = o.data.volumes["pickup_eye_trigger"]
                .interior_point()
                .context("No Eye volume")?
                - Vec3::Z * 20.;
            let mut p = pair(&w, o.data.points["alice_dead_pos1"].translation)?;
            let mut s = [
                Stats::for_level("jlair2", None),
                Stats::for_level("jlair2", None),
            ];
            for (i, s) in s.iter_mut().enumerate() {
                s.set_health(43. + i as f32 * 14.)?;
                ensure!(
                    s.spend_will(17. + i as f32 * 10.),
                    "Reward fixture could not spend Will"
                );
            }
            if dead {
                s[1 - first].damage(1000.);
            }
            p[first] = Player::new(at);
            o.coop_prepare(&mut slots(&mut p, &mut s));
            let before = s.each_ref().map(|s| (s.sanity(), s.will(), s.copies(7)));
            o.pair_contacts(0., &mut slots(&mut p, &mut s));
            ensure!(
                s[first].copies(7) == before[first].2,
                "Paused Eye collected"
            );
            o.pair_contacts(0.1, &mut slots(&mut p, &mut s));
            ensure!(
                s[first].copies(7) == before[first].2 + 1
                    && s[1 - first].copies(7) == before[1 - first].2
                    && s[first].will() == 100.
                    && s[1 - first].will() == before[1 - first].1
                    && !o.personal_eye_visible(Some(first))
                    && o.personal_eye_visible(Some(1 - first))
                    && !o.saved.eye,
                "First Eye collection consumed partner reward or committed exit"
            );
            for _ in 0..35 {
                o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            }
            ensure!(
                !o.saved.exit.committed && o.saved.phase == Phase::Reward,
                "Left before both Eye collections"
            );
            o = restore(a, map, &o)?;
            o.pair_contacts(0.1, &mut slots(&mut p, &mut s));
            ensure!(
                s[first].copies(7) == before[first].2 + 1,
                "Restored Eye duplicated"
            );
            p[1 - first] = Player::new(at);
            if dead {
                o.pair_contacts(0.1, &mut slots(&mut p, &mut s));
                ensure!(
                    !s[1 - first].alive() && s[1 - first].copies(7) == before[1 - first].2,
                    "Dead partner received Eye"
                );
                s[1 - first].set_health(25.)?;
            }
            o.pair_contacts(0.1, &mut slots(&mut p, &mut s));
            ensure!(
                s[1 - first].copies(7) == before[1 - first].2 + 1
                    && o.saved.eye
                    && (0..2).all(|i| !o.personal_eye_visible(Some(i))),
                "Partner's restored Eye unavailable"
            );
            ensure!(
                (0..2).all(|i| s[i].will() == 100.
                    && s[i].sanity()
                        == if dead && i == 1 - first {
                            25.
                        } else {
                            before[i].0
                        }),
                "Eye changed personal health/Will"
            );
            for _ in 0..21 {
                o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            }
            let exit = o
                .coop_events()
                .transition
                .context("Missing automatic paired Eye exit")?;
            ensure!(
                Kind::Lair.exit().matches(&exit) && o.coop_events().transition.is_none(),
                "Wrong/repeated paired exit"
            );
            o.transition_failed(&exit);
            let snap = o.snapshot();
            o.pair_advance(0., map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            ensure!(
                o.snapshot() == snap && o.coop_events().transition.is_none(),
                "Paused retry advanced"
            );
            for _ in 0..11 {
                o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            }
            ensure!(
                o.coop_events().transition == Some(exit),
                "Failed exit did not retry without use input"
            );
            restore(a, map, &o)?;
            println!("PASS jlair2 Eye slot{first} first dead{dead}: personal inventory/view, restored collection, both-ready exit and automatic retry");
        }
    }
    Ok(())
}
fn compatibility(a: &mut Assets, map: &Bsp, kind: Kind) -> Result<()> {
    for completed in [false, true] {
        let mut o = Encounter::load(a, map, kind)?;
        if completed {
            victory(&mut o);
            if kind == Kind::Lair {
                o.saved.eye = true;
            }
        }
        let old = o.snapshot();
        let mut copy = Encounter::load(a, map, kind)?;
        copy.restore(&old, map)?;
        ensure!(copy.snapshot() == old, "Legacy solo snapshot rewritten");
        copy.pair_state();
        let old_pair = copy.snapshot();
        ensure!(
            old_pair["coop"].get("world").is_none(),
            "Old paired fixture unexpectedly has world fields"
        );
        copy.restore(&old_pair, map)?;
        let eye = copy.pair_world().eye;
        ensure!(
            eye == [kind == Kind::Lair && completed; 2],
            "Legacy reward import changed ownership"
        );
        let valid = copy.snapshot();
        for (field, value) in [
            ("actor", serde_json::json!(2)),
            ("pending_scene", serde_json::json!(1)),
            ("eye", serde_json::json!([true, false])),
        ] {
            let mut bad = valid.clone();
            bad["coop"]["world"][field] = value;
            ensure!(
                copy.restore(&bad, map).is_err() && copy.snapshot() == valid,
                "Corrupt paired {field} accepted"
            );
        }
        println!("PASS {} legacy solo/paired world import completed{completed} and invalid-state rejection",kind.map());
    }
    Ok(())
}
fn deck(map: &Bsp, o: &Encounter) -> Result<[Player; 2]> {
    let (m, at, q) = *o
        .poses
        .iter()
        .find(|(m, _, _)| {
            o.data
                .brushes
                .iter()
                .any(|b| b.index == *m && b.name == "drawbridge")
        })
        .unwrap();
    let hull = Collider::model(map, m, at, q, true)?;
    let w = world(map, o)?;
    let model = &map.models[m];
    let centre = at + q * ((model.min + model.max) * 0.5);
    let mut found = Vec::new();
    let offsets = [0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 6, -6];
    for x in offsets {
        for y in offsets {
            let top = centre + vec3(x as f32 * 24., y as f32 * 24., 1200.);
            let end = top - Vec3::Z * 2400.;
            let tr = hull.trace(
                top + crate::collision::PLAYER_CENTER,
                end + crate::collision::PLAYER_CENTER,
                crate::collision::PLAYER_HALF,
            );
            if tr.start_solid || tr.fraction >= 1. || tr.normal.z <= 0.65 {
                continue;
            }
            let at = top.lerp(end, tr.fraction);
            let ground = hull.trace(
                at + crate::collision::PLAYER_CENTER,
                at + crate::collision::PLAYER_CENTER - Vec3::Z * 3.,
                vec3(1., 1., 28.),
            );
            if !w.body_clear(at)
                || ground.start_solid
                || ground.fraction >= 1.
                || ground.normal.z <= 0.65
            {
                continue;
            }
            if found.iter().any(|p: &Player| p.feet.distance(at) < 64.) {
                continue;
            }
            let mut p = Player::new(at);
            p.grounded = true;
            p.ground_normal = tr.normal;
            found.push(p);
            if found.len() == 2 {
                return Ok([found.remove(0), found.remove(0)]);
            }
        }
    }
    anyhow::bail!(
        "No native drawbridge rider pair at t{} centre{centre:?}",
        o.saved.time
    )
}
fn machinery(a: &mut Assets, map: &Bsp) -> Result<()> {
    for dead in [None, Some(0), Some(1)] {
        let mut o = Encounter::load(a, map, Kind::Grounds)?;
        victory(&mut o);
        o.saved.time = 12.;
        o.rebuild(map)?;
        let mut p = deck(map, &o)?;
        let before = p.each_ref().map(|p| p.feet);
        let mut s = [
            Stats::for_level("grounds1", None),
            Stats::for_level("grounds1", None),
        ];
        if let Some(i) = dead {
            s[i].damage(1000.);
        }
        let mut w = world(map, &o)?;
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        if o.saved.time == 12. {
            let old = *o
                .poses
                .iter()
                .find(|(m, _, _)| {
                    o.data
                        .brushes
                        .iter()
                        .any(|b| b.index == *m && b.name == "drawbridge")
                })
                .unwrap();
            let hull = Collider::model(map, old.0, old.1, old.2, true)?;
            o.saved.time = 12.1;
            o.rebuild(map)?;
            let next = *o.poses.iter().find(|(m, _, _)| *m == old.0).unwrap();
            for (slot, p) in p.iter().enumerate() {
                let foot = next.1 + next.2 * (old.2.inverse() * (p.feet - old.1));
                let support = hull.trace(
                    p.feet + crate::collision::PLAYER_CENTER,
                    p.feet + crate::collision::PLAYER_CENTER - Vec3::Z * 3.,
                    vec3(1., 1., 28.),
                );
                w.set_dynamic(
                    o.poses
                        .iter()
                        .filter(|(m, at, _)| *m != old.0 && at.z > -90000.)
                        .map(|(m, at, q)| Collider::model(map, *m, *at, *q, true))
                        .collect::<Result<Vec<_>>>()?,
                );
                let trace = w.body_trace(p.feet, foot);
                w.set_dynamic(o.colliders());
                eprintln!("Bridge slot{slot}: from{:?} to{foot:?}, support solid{} fraction{} normal{:?}, path solid{} fraction{}, clear{}",
                    p.feet,support.start_solid,support.fraction,support.normal,trace.start_solid,trace.fraction,w.body_clear(foot));
            }
            anyhow::bail!("Bridge did not lower with riders {dead:?}");
        }
        ensure!(
            o.saved.time > 12.,
            "Bridge did not lower with riders {dead:?}"
        );
        for i in 0..2 {
            ensure!(
                if dead == Some(i) {
                    p[i].feet == before[i]
                } else {
                    p[i].feet.distance(before[i]) > 0.1 && w.body_clear(p[i].feet)
                },
                "Bridge missed/embedded slot{i}"
            );
        }
        let mut copy = restore(a, map, &o)?;
        let mut p2 = p.clone();
        let mut s2 = s.clone();
        let mut w2 = world(map, &copy)?;
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        copy.pair_advance(0.1, map, &mut w2, &mut slots(&mut p2, &mut s2), &[])?;
        ensure!(
            o.snapshot() == copy.snapshot()
                && serde_json::to_value(&p)? == serde_json::to_value(&p2)?,
            "Restored moving bridge diverged"
        );
        println!(
            "PASS grounds1 native bridge carry dead{dead:?}, clear riders and saved continuation"
        );
    }
    for blocked in 0..2 {
        let mut o = Encounter::load(a, map, Kind::Grounds)?;
        victory(&mut o);
        o.saved.time = 12.;
        o.rebuild(map)?;
        let mut p = deck(map, &o)?;
        let before = p.each_ref().map(|p| p.feet);
        let mut s = [
            Stats::for_level("grounds1", None),
            Stats::for_level("grounds1", None),
        ];
        let mut w = world(map, &o)?;
        let centre = p[blocked].feet + crate::collision::PLAYER_CENTER;
        let obstacle = Collider::box_bounds(centre - Vec3::splat(8.), centre + Vec3::splat(8.));
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[obstacle])?;
        ensure!(
            o.saved.time == 12. && p.each_ref().map(|p| p.feet) == before,
            "Blocked carry was not atomic"
        );
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        ensure!(
            o.saved.time > 12.
                && p.iter()
                    .zip(before)
                    .all(|(p, at)| p.feet.distance(at) > 0.1),
            "Released bridge remained blocked"
        );
        println!(
            "PASS grounds1 bridge slot{blocked} obstruction rolls back both riders then releases"
        );
    }
    Ok(())
}
fn doors(a: &mut Assets, map: &Bsp) -> Result<()> {
    for opener in 0..2 {
        let mut o = Encounter::load(a, map, Kind::Grounds)?;
        victory(&mut o);
        o.phase(Phase::Done);
        o.rebuild(map)?;
        let mut w = world(map, &o)?;
        let mut p = pair(&w, o.data.points["gdeath"].translation)?;
        let mut s = [
            Stats::for_level("grounds1", None),
            Stats::for_level("grounds1", None),
        ];
        // Start the floor probe below the castle roof, as ordinary traversal does.
        p[opener] = floor(&w, vec3(1300., 192., 0.))?;
        s[opener].damage(1000.);
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        ensure!(
            !o.saved.door_requested && o.saved.door == 0.,
            "Dead player opened exit door"
        );
        s[opener].set_health(25.)?;
        for _ in 0..21 {
            o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        }
        ensure!(
            o.saved.door_requested && o.saved.door == 2. && p.iter().all(|p| w.body_clear(p.feet)),
            "Living slot{opener} did not open clear exit"
        );
        o.event("CatThread");
        o.event("GnomeGuard");
        o.pair_advance(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        o.event("CatThread");
        o.event("GnomeGuard");
        ensure!(
            o.saved.cat == Some(0.1) && o.saved.gnome == Some(0.1),
            "Repeated epilogue contact restarted its clock"
        );
        restore(a, map, &o)?;
        println!(
            "PASS grounds1 exit door slot{opener}, dead exclusion, epilogue clocks and saved state"
        );
    }
    Ok(())
}
pub(crate) fn check(a: &mut Assets) -> Result<()> {
    for kind in [Kind::Lair, Kind::Grounds] {
        let map = Bsp::parse(&a.read(&format!("maps/{}.bsp", kind.map()))?)?;
        clocks(a, &map, kind)?;
        compatibility(a, &map, kind)?;
        if kind == Kind::Lair {
            rewards(a, &map)?;
        } else {
            machinery(a, &map)?;
            doors(a, &map)?;
        }
    }
    Ok(())
}
