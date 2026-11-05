//! Native staged checks; these do not claim an independent-client full route.
use super::*;
use crate::{level::CoopPlayer, movement::{Controls, FIXED_DT}, coop::{
    protocol::{Input, Visit}, stage::Stage, world::Roster, PlayerId,
}};

fn slots<'a>(p: &'a mut [Player; 2], s: &'a mut [Stats; 2]) -> [CoopPlayer<'a>; 2] {
    let [a, b] = p.each_mut(); let [x, y] = s.each_mut();
    [CoopPlayer { player: a, stats: x }, CoopPlayer { player: b, stats: y }]
}
fn stats() -> [Stats; 2] { std::array::from_fn(|_| Stats::for_level("jlair1", None)) }
fn free(o: &mut Curiosity) { o.saved.initialized = true; o.saved.arrived = true; o.saved.scene = None; }
fn pickup_edge(o: &Curiosity, item: usize) -> Vec3 {
    // The two authored pickups are only 32 units apart. Reach one from its
    // outside edge so the other pickup's 42-unit radius remains untouched.
    o.essence[item] + (o.essence[item] - o.essence[1-item]).normalize() * 24. - PLAYER_CENTER
}
fn world(map: &Bsp, o: &Curiosity) -> Result<World> {
    let mut w = World::from_bsp(map)?; w.set_dynamic(o.colliders()); Ok(w)
}
fn restore(a: &mut Assets, map: &Bsp, o: &Curiosity) -> Result<Curiosity> {
    let bytes = crate::coop::codec::encode(&serde_json::to_vec(&o.snapshot())?)?;
    let mut restored = Curiosity::load(a, map)?;
    restored.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?, map)?;
    ensure!(restored.snapshot() == o.snapshot(), "Paired Curiosity restore changed state");
    Ok(restored)
}
fn personal(a: &mut Assets, map: &Bsp) -> Result<()> {
    for dead in [None, Some(0), Some(1)] {
        let mut o = Curiosity::load(a, map)?;
        let mut p = std::array::from_fn(|_| Player::new(o.point("alice_pos1").translation));
        let mut s = stats();
        s[0].damage(20.); s[1].damage(60.);
        if let Some(i) = dead { s[i].damage(1000.); }
        o.prepare_pair(&mut slots(&mut p, &mut s));
        for i in 0..2 { ensure!(s[i].sanity() == if dead == Some(i) { 0. } else { 100. }, "Wrong arrival health"); }
        if let Some(i) = dead { s[i].set_health(25.)?; }
        s[0].spend_will(10.); s[1].spend_will(30.);
        let before = serde_json::to_value(&s)?;
        o.prepare_pair(&mut slots(&mut p, &mut s));
        o.coop_scene_step(0, &mut p[0], &mut s[0]);
        ensure!(serde_json::to_value(&s)? == before, "Arrival or scene refilled existing resources");
        restore(a, map, &o)?;
        println!("PASS paired Curiosity personal arrival, dead arrival and repeat preparation dead={dead:?}");
    }
    for first in 0..2 { for item in 0..2 {
        let mut o = Curiosity::load(a, map)?; free(&mut o);
        let mut p = std::array::from_fn(|_| Player::new(o.point("alice_pos1").translation));
        let mut s = stats(); o.prepare_pair(&mut slots(&mut p, &mut s));
        o.saved.essence_live = true;
        s[0].damage(50.); s[1].damage(60.);
        p[first].feet = pickup_edge(&o, item);
        let other = serde_json::to_value(&s[1-first])?;
        let health = s[first].sanity();
        o.pickups_pair(FIXED_DT, &mut slots(&mut p, &mut s));
        ensure!(s[first].sanity() == health + 15. && serde_json::to_value(&s[1-first])? == other,
            "Wrong personal healing: first {} expected {}, other unchanged {}",s[first].sanity(),health+15.,serde_json::to_value(&s[1-first])? == other);
        ensure!(!o.essence_visible(item, Some(first)) && o.essence_visible(item, Some(1-first)),
            "Personal pickup visibility is shared");
        o = restore(a, map, &o)?;
        o.pickups_pair(FIXED_DT, &mut slots(&mut p, &mut s));
        ensure!(s[first].sanity() == health + 15., "Saved pickup collected twice");
        p[first].feet = o.point("alice_pos1").translation;
        p[1-first].feet = pickup_edge(&o, item);
        let health = s[1-first].sanity();
        o.pickups_pair(FIXED_DT, &mut slots(&mut p, &mut s));
        ensure!(s[1-first].sanity() == health + 15., "Partner lost their own pickup");
        let good = o.snapshot(); let mut bad = good.clone(); bad["essence_live"] = false.into();
        ensure!(o.restore(&bad, map).is_err() && o.snapshot() == good, "Invalid personal save mutated the visit");
        println!("PASS paired Curiosity personal essence {item}, collector {first}, visibility and saved consumption");
    }}
    let mut o = Curiosity::load(a, map)?; free(&mut o);
    o.saved.essence_live = true; o.saved.essence_taken[0] = true;
    let old = o.snapshot(); ensure!(old.get("coop").is_none(), "Single-player save gained paired fields");
    o.restore(&old, map)?;
    let mut p = std::array::from_fn(|_| Player::new(pickup_edge(&o, 0)));
    let mut s = stats(); s[0].damage(20.); s[1].damage(60.);
    let before = serde_json::to_value(&s)?;
    o.prepare_pair(&mut slots(&mut p, &mut s)); o.pickups_pair(FIXED_DT, &mut slots(&mut p, &mut s));
    ensure!(serde_json::to_value(&s)? == before && (0..2).all(|i| !o.essence_visible(0, Some(i))),
        "Legacy visit regained health or consumed essence");
    println!("PASS paired Curiosity legacy save preserves health and used essence");
    Ok(())
}
fn platform(a: &mut Assets, map: &Bsp) -> Result<()> {
    for hz in [30, 60, 144] {
        let mut o = Curiosity::load(a, map)?; free(&mut o);
        let mut w = world(map, &o)?;
        let mut p: [Player; 2] = [Vec3::new(-7048., 698., 1640.), Vec3::new(-7000., 698., 1640.)].map(|at|
            w.actor_footing(at, PLAYER_CENTER, PLAYER_HALF, 20.).map(Player::new).context("Missing paired platform footing"))
            .into_iter().collect::<Result<Vec<_>>>()?.try_into().map_err(|_| anyhow::anyhow!("Invalid fixture"))?;
        for player in &mut p { player.grounded = true; }
        let mut s = stats();
        o.prepare_pair(&mut slots(&mut p, &mut s));
        let start = p.each_ref().map(|p| p.feet);
        let saved = o.snapshot(); let people = serde_json::to_value(&p)?;
        for dt in [0., -1., f32::NAN] {
            o.move_sink_pair(dt, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
            o.pickups_pair(dt, &mut slots(&mut p, &mut s));
        }
        ensure!(o.snapshot() == saved && serde_json::to_value(&p)? == people, "Paused paired platform changed");
        let outside = o.point("alice_pos1").translation;
        p[1].feet = outside;
        for _ in 0..hz { o.move_sink_pair(1./hz as f32, map, &mut w, &mut slots(&mut p, &mut s), &[])?; }
        ensure!(o.saved.sink == 0. && p[0].feet == start[0], "Platform left without both living riders");
        p[1].feet = start[1];
        for _ in 0..hz/2 { o.move_sink_pair(1./hz as f32, map, &mut w, &mut slots(&mut p, &mut s), &[])?; }
        ensure!((o.saved.sink - 50.).abs() < 0.1, "Wrong shared platform clock at {hz}Hz: {}",o.saved.sink);
        let mut bed_landings = 0;
        for i in 0..2 {
            let drop = start[i].z - p[i].feet.z;
            ensure!(drop > 0.1 && drop <= 50.1 && w.body_clear(p[i].feet), "Rider failed to descend safely at {hz}Hz");
            if (drop-50.).abs() > 0.1 {
                // The second rider reaches the original uneven bed first.
                // Remove the platform when proving that independent support.
                w.set_dynamic(vec![]);
                let floor = w.body_trace(p[i].feet,p[i].feet-Vec3::Z*3.);
                let landed = !floor.start_solid && floor.fraction < 1. && floor.normal.z > 0.65;
                w.set_dynamic(o.colliders());
                ensure!(landed, "A rider stopped following the deck without reaching the native floor");
                bed_landings += 1;
            }
        }
        ensure!(bed_landings == 1, "Fixture missed the unequal-height floor handoff");
        o = restore(a, map, &o)?;
        // A rider stepping off during an already-started descent cannot strand the partner.
        p[1].feet = outside;
        let height = p[0].feet.z;
        o.move_sink_pair(1./hz as f32, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        ensure!(p[0].feet.z < height, "Disembarking rider stranded the partner mid-descent");
        p[0].feet = outside + Vec3::Y * 48.;
        for _ in 0..hz*2 { o.move_sink_pair(1./hz as f32, map, &mut w, &mut slots(&mut p, &mut s), &[])?; }
        ensure!(o.saved.sink == 0., "Empty paired platform failed to return");
        println!("PASS paired Curiosity {hz}Hz boarding wait, two-rider descent, uneven-bed handoff, saved continuation and empty return");
    }
    for dead in 0..2 {
        let mut o = Curiosity::load(a, map)?; free(&mut o); let mut w = world(map, &o)?;
        let at = w.actor_footing(vec3(-7048.,698.,1640.), PLAYER_CENTER, PLAYER_HALF,20.).context("Missing platform")?;
        let mut p = [Player::new(at),Player::new(at)]; p[dead].feet = o.point("alice_pos1").translation;
        let mut s = stats(); s[dead].damage(1000.);
        let waiting = p[dead].feet;
        o.move_sink_pair(FIXED_DT, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
        ensure!(o.saved.sink > 0. && p[dead].feet == waiting && !s[dead].alive(), "Dead partner blocked or rode the platform");
        println!("PASS paired Curiosity living rider can depart while slot {dead} is dead");
    }
    // An empty returning platform must not rise through a departing jumper.
    let mut o = Curiosity::load(a, map)?; free(&mut o);
    o.saved.sink = 64.; o.rebuild(map)?;
    let mut w = world(map, &o)?;
    let at = w.actor_footing(vec3(-7048.,716.,1580.), PLAYER_CENTER, PLAYER_HALF,30.).context("Missing lowered platform")?;
    let mut p = [Player::new(at), Player::new(o.point("alice_pos1").translation)];
    p[0].velocity.z = 2.;
    let mut s = stats(); o.prepare_pair(&mut slots(&mut p, &mut s));
    let old = o.snapshot(); let bodies = serde_json::to_value(&p)?;
    o.move_sink_pair(0.1, map, &mut w, &mut slots(&mut p, &mut s), &[])?;
    ensure!(o.snapshot() == old && serde_json::to_value(&p)? == bodies && p.iter().all(|p|w.body_clear(p.feet)),
        "Returning platform embedded a jumper or committed part of blocked motion");
    println!("PASS paired Curiosity returning-platform obstruction rolls back world and both bodies");
    Ok(())
}
fn ctl(stage: &mut Stage) -> &mut Curiosity {
    stage.interactions.levels.iter_mut().find_map(|s| s.ctl.downcast_mut()).unwrap()
}
fn scene(a: &mut Assets, map: &Bsp, kind: Kind, first: PlayerId, skip: bool, dead: bool) -> Result<()> {
    let visit = Visit { epoch:1, map:"jlair1".into(), entry:Some("jlair1_start1".into()), difficulty:Default::default(), combat_preview:false };
    let mut w = World::from_bsp(map)?;
    let mut stage = Stage::load(a, map, &mut w, &visit, true)?;
    let (eye,yaw) = interaction::spawn(map, visit.entry.as_deref());
    let mut r = Roster::new(&w,eye,yaw,Stats::for_level("jlair1",None))?;
    ctl(&mut stage).saved.initialized = true;
    r.players[0].stats.set_health(54.)?; r.players[1].stats.set_health(67.)?;
    let other = first.partner().index();
    if dead { r.players[other].stats.damage(1000.); }
    if kind == Kind::Caterpillar {
        free(ctl(&mut stage)); stage.owner = None;
        let entity = &map.entities[26];
        let index = entity["model"].trim_start_matches('*').parse::<usize>()?;
        let trigger = Collider::model(map,index,pose(entity).translation,Quat::IDENTITY,false)?;
        let at = trigger.interior_point().context("Empty Caterpillar trigger")? - PLAYER_CENTER;
        r.players[first.index()].player = Player::new(at);
        let before = r.players.each_ref().map(|p|p.player.feet);
        stage.contacts(&mut r,&w,before)?;
    }
    ensure!(stage.owner == Some(first) && stage.scene_id() == Some(kind.id()), "Wrong first scene entrant");
    let waiting = r.players[other].player.feet;
    let resources = serde_json::to_value(r.players.each_ref().map(|p|&p.stats))?;
    let mut lines = std::collections::BTreeSet::new();
    let mut ended = None;
    for tick in 0..36000 {
        if let Some(s) = &ctl(&mut stage).saved.scene { lines.insert(s.line); }
        if tick == 120 || tick == 660 {
            let saved = serde_json::to_value(stage.snapshot())?;
            let camera = stage.camera(&w).map(|c|(c.eye,c.target,c.up));
            let bytes = crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
            stage.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,&mut w)?;
            ensure!(serde_json::to_value(stage.snapshot())? == saved && stage.camera(&w).map(|c|(c.eye,c.target,c.up)) == camera,
                "Paired scene changed after compressed restore");
        }
        let mut input = [(Controls::default(),Input::default());2];
        input[other].1.skip_scene = true;
        input[first.index()].1.skip_scene = skip && tick >= 240;
        stage.scene_step(&mut r,map,&mut w,&input)?;
        if !stage.busy() { ended = Some(tick); break; }
        ensure!(stage.owner == Some(first) && r.players[other].player.feet == waiting && stage.camera(&w).is_some(),
            "Scene changed owner, camera or waiting partner");
        ensure!(tick >= 240 || stage.scene_id() == Some(kind.id()), "Partner skipped owner's scene");
    }
    ensure!(ended.is_some(), "Paired scene did not finish");
    ensure!(serde_json::to_value(r.players.each_ref().map(|p|&p.stats))? == resources, "Scene changed personal resources");
    // Keep the original actor's authored endpoint. A grounded original actor
    // may stand near an edge; the newly placed partner still needs co-op's full
    // footprint checks below. Do not substitute a respawn site for the scene.
    let actor = &r.players[first.index()].player;
    let ground = w.body_trace(actor.feet,actor.feet-Vec3::Z*3.);
    let endpoint = ctl(&mut stage).point(if kind == Kind::Arrival {"alice_pos1"} else {"alice_cater_pos1"}).translation;
    ensure!(actor.validate_world(&w).is_ok() && actor.grounded && actor.immersion.level == 0
        && !ground.start_solid && ground.fraction < 1. && ground.normal.z > 0.65
        && (actor.feet-endpoint).truncate().length() < 0.01 && (actor.feet.z-endpoint.z).abs() < 8.
        && !stage.interactions.hazardous(actor.feet), "Unsafe or changed original owner handoff: {:?}",actor.feet);
    if dead { ensure!(r.players[other].player.feet == waiting && !r.players[other].stats.alive(), "Scene revived waiting partner"); }
    else {
        let a = r.players[first.index()].player.feet; let b = r.players[other].player.feet;
        ensure!(w.body_clear(b) && crate::coop::world::revival_supported(&w,&r.players[other].player) && a.distance(b) <= 128.
            && ((a-b).truncate().length() >= 35.95 || (a.z-b.z).abs() >= 55.95), "Unsafe shared scene reunion");
    }
    if !skip { ensure!(lines.len() == if kind == Kind::Arrival {1} else {15}, "Missing watched dialogue stages"); }
    ensure!(ctl(&mut stage).saved.arrived && (kind != Kind::Caterpillar || ctl(&mut stage).saved.caterpillar), "Missing shared scene completion");
    println!("PASS paired Curiosity {} owner{first:?} skip{skip} dead{dead}: native contact, original camera/dialogue, frozen partner, restore and reunion",kind.id());
    Ok(())
}
pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/jlair1.bsp")?)?;
    let mut failures = vec![];
    if let Err(e) = personal(a,&map) { failures.push(format!("personal: {e:#}")); }
    if let Err(e) = platform(a,&map) { failures.push(format!("platform: {e:#}")); }
    for skip in [false,true] { for dead in [false,true] {
        if let Err(e) = scene(a,&map,Kind::Arrival,PlayerId::Blue,skip,dead) {
            failures.push(format!("arrival skip{skip} dead{dead}: {e:#}"));
        }
        for first in PlayerId::ALL {
            if let Err(e) = scene(a,&map,Kind::Caterpillar,first,skip,dead) {
                failures.push(format!("Caterpillar {first:?} skip{skip} dead{dead}: {e:#}"));
            }
        }
    }}
    ensure!(failures.is_empty(), "Paired Curiosity failures: {}",failures.join("; "));
    println!("PASS paired Curiosity components; full two-client traversal/recovery remains unverified");
    Ok(())
}
