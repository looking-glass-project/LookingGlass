//! Real Stage ownership, scene clocks and reunion with staged battle prerequisites.
use super::*;
use crate::{
    coop::{
        protocol::{Input, Visit},
        stage::Stage,
        world::Roster,
        PlayerId,
    },
    movement::{Controls, FIXED_DT},
};
fn ctl(stage: &mut Stage) -> &mut Encounter {
    stage
        .interactions
        .levels
        .iter_mut()
        .find_map(|s| s.ctl.downcast_mut())
        .unwrap()
}
fn checkpoint(stage: &mut Stage, r: &mut Roster, map: &Bsp, w: &mut World) -> Result<()> {
    let saved = serde_json::to_value(stage.snapshot())?;
    let people = serde_json::to_value(&r.players)?;
    let camera = stage.camera(w).map(|c| (c.eye, c.target, c.up));
    let poses = stage.draw_transforms(w);
    let mut players = r.players.each_mut().map(|p| crate::level::CoopPlayer {
        player: &mut p.player,
        stats: &mut p.stats,
    });
    for dt in [0., -1., f32::NAN] {
        ctl(stage).pair_advance(dt, map, w, &mut players, &[])?;
        ctl(stage).pair_contacts(dt, &mut players);
    }
    ensure!(
        serde_json::to_value(stage.snapshot())? == saved
            && serde_json::to_value(&r.players)? == people,
        "Paused scene mutated world/players"
    );
    let bytes = crate::coop::codec::encode(&serde_json::to_vec(&saved)?)?;
    stage.restore(
        &serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,
        map,
        w,
    )?;
    ensure!(
        serde_json::to_value(stage.snapshot())? == saved
            && camera == stage.camera(w).map(|c| (c.eye, c.target, c.up))
            && poses == stage.draw_transforms(w),
        "Scene restore lost camera/world/owner"
    );
    Ok(())
}
fn setup(
    a: &mut Assets,
    map: &Bsp,
    kind: Kind,
    first: PlayerId,
    dead: bool,
) -> Result<(World, Stage, Roster)> {
    let visit = Visit {
        epoch: 1,
        map: kind.map().into(),
        entry: None,
        difficulty: Default::default(),
        combat_preview: false,
    };
    let mut w = World::from_bsp(map)?;
    let mut stage = Stage::load(a, map, &mut w, &visit, true)?;
    let (eye, yaw) = crate::interaction::spawn(map, None);
    let mut r = Roster::new(&w, eye, yaw, Stats::for_level(kind.map(), None))?;
    r.players[0].stats.set_health(47.)?;
    r.players[1].stats.set_health(63.)?;
    if dead {
        r.players[first.partner().index()].stats.damage(1000.);
    }
    stage.owner = Some(first);
    stage.accept(Events::default(), first, &mut r, &w);
    Ok((w, stage, r))
}
fn finale(
    stage: &mut Stage,
    r: &mut Roster,
    map: &Bsp,
    w: &mut World,
    kind: Kind,
    first: PlayerId,
) -> Result<()> {
    let o = ctl(stage);
    o.finish_intro(w, &mut r.players[first.index()].player)?;
    if kind == Kind::Lair {
        o.saved.ticks = 10799;
        o.wave();
        o.wave();
    } else {
        o.saved.boss.flying = true;
        o.saved.boss.landed = false;
        o.saved.boss.at.z += 148.;
        o.saved.boss.hit(
            Hit {
                id: 0,
                damage: 2000.,
                kind: crate::combat::DamageKind::Other,
                knockback: Vec3::ZERO,
            },
            kind,
        );
    }
    o.rebuild(map)?;
    stage.owner = None;
    stage.story.finish_sequence(kind.intro());
    stage.interactions.sync_coop(w);
    if kind == Kind::Lair {
        let inside = ctl(stage).data.volumes["end_trigger"]
            .interior_point()
            .context("No finale volume")?
            - crate::collision::PLAYER_CENTER;
        // The first arrival waits while the timer is still closed. The second
        // arrival must not turn a Red-first wait into an arbitrary Blue tie.
        let outside = inside + Vec3::X * 4096.;
        ensure!(
            !ctl(stage).data.volumes["end_trigger"].touches(
                outside + crate::collision::PLAYER_CENTER,
                outside + crate::collision::PLAYER_CENTER,
                crate::collision::PLAYER_HALF
            ),
            "Waiting fixture already occupies finale trigger"
        );
        r.players[first.partner().index()].player = Player::new(outside);
        r.players[first.index()].player = Player::new(inside);
        let before = r.players.each_ref().map(|p| p.player.feet);
        stage.contacts(r, w, before)?;
        ensure!(
            stage.owner.is_none() && ctl(stage).saved.phase == Phase::Fight,
            "Early finale bypassed survival timer"
        );
        if r.players[first.partner().index()].stats.alive() {
            r.players[first.partner().index()].player = Player::new(inside);
        }
        let before = r.players.each_ref().map(|p| p.player.feet);
        stage.contacts(r, w, before)?;
        ctl(stage).saved.ticks = 10800;
        let before = r.players.each_ref().map(|p| p.player.feet);
        stage.contacts(r, w, before)?;
    } else {
        let at = ctl(stage).saved.boss.at + vec3(-300., 0., -148.);
        let mut players = coop_world_check::pair(w, at)?;
        let boss = ctl(stage).saved.boss.at;
        if players[0].feet.distance_squared(boss) > players[1].feet.distance_squared(boss) {
            players.swap(0, 1);
        }
        r.players[first.index()].player = players[0].clone();
        r.players[first.partner().index()].player = players[1].clone();
        for tick in 0..1300 {
            stage.before_move(r, map, w, &Default::default(), FIXED_DT)?;
            if tick == 120 {
                checkpoint(stage, r, map, w)?;
            }
            if stage.owner.is_some() {
                break;
            }
        }
    }
    ensure!(
        stage.owner == Some(first) && stage.scene_id() == Some(kind.outro()),
        "{} finale chose {:?}, expected {first:?}",
        kind.map(),
        stage.owner
    );
    ensure!(
        ctl(stage)
            .saved
            .coop
            .as_ref()
            .unwrap()
            .world
            .as_ref()
            .unwrap()
            .actor
            == Some(first.index()),
        "Finale owner not bound before first frame"
    );
    Ok(())
}
fn scene(
    a: &mut Assets,
    map: &Bsp,
    kind: Kind,
    first: PlayerId,
    outro: bool,
    skip: bool,
    dead: bool,
) -> Result<()> {
    let (mut w, mut stage, mut r) = setup(a, map, kind, first, dead)?;
    if outro {
        finale(&mut stage, &mut r, map, &mut w, kind, first)?;
    }
    checkpoint(&mut stage, &mut r, map, &mut w)?;
    let other = first.partner().index();
    let waiting = r.players[other].player.feet;
    let resources = serde_json::to_value(r.players.each_ref().map(|p| &p.stats))?;
    let mut skipped = false;
    let mut finished = None;
    let mut lines = std::collections::BTreeSet::new();
    for tick in 0..30000 {
        if tick == 120 || tick == 660 {
            checkpoint(&mut stage, &mut r, map, &mut w)?;
        }
        let mut inputs = [(Controls::default(), Input::default()); 2];
        inputs[other].1.skip_scene = true;
        inputs[first.index()].1.skip_scene = skip && tick >= 240;
        stage.scene_step(&mut r, map, &mut w, &inputs)?;
        skipped |= stage.notices[first.index()]
            .as_ref()
            .is_some_and(|n| n.text == "Scene skipped");
        if let Some((line, _)) =
            stage
                .story
                .progress(if outro { kind.outro() } else { kind.intro() })
        {
            lines.insert(line);
        }
        if stage.owner.is_none() {
            finished = Some(tick);
            break;
        }
        ensure!(
            r.players[other].player.feet == waiting,
            "Scene moved waiting partner"
        );
        if ctl(&mut stage).scripted() {
            ensure!(
                stage.camera(&w).is_some()
                    && ctl(&mut stage).hides_player()
                    && ctl(&mut stage)
                        .performances()
                        .iter()
                        .filter(|p| p.model == "alice")
                        .count()
                        == 1,
                "Original camera/single Alice lost"
            );
        }
        ensure!(
            ctl(&mut stage)
                .saved
                .coop
                .as_ref()
                .unwrap()
                .world
                .as_ref()
                .unwrap()
                .actor
                == Some(first.index()),
            "Scene owner changed"
        );
    }
    let phase = ctl(&mut stage).saved.phase;
    let time = ctl(&mut stage).saved.time;
    ensure!(
        finished.is_some() && skipped == skip,
        "{} scene did not finish: outro{outro}, skip{skip}, owner{:?}, phase{phase:?}, t{time}",
        kind.map(),
        stage.owner
    );
    ensure!(
        serde_json::to_value(r.players.each_ref().map(|p| &p.stats))? == resources,
        "Scene changed personal resources"
    );
    let expected = if !outro {
        Phase::Fight
    } else if kind == Kind::Lair {
        Phase::Reward
    } else {
        Phase::Bridge
    };
    ensure!(
        ctl(&mut stage).saved.phase == expected,
        "Wrong scene handoff"
    );
    if dead {
        ensure!(
            r.players[other].player.feet == waiting && !r.players[other].stats.alive(),
            "Scene revived/moved dead partner"
        );
    } else {
        ensure!(
            r.players.iter().all(|p| w.body_clear(p.player.feet))
                && crate::recovery::supported(&w, r.players[other].player.feet)
                && r.players[0].player.feet.distance(r.players[1].player.feet)
                    >= crate::coop::separation::DISTANCE - 0.05,
            "Reunion embedded, unsupported or overlapped partner"
        );
    }
    if outro && kind == Kind::Grounds {
        ensure!(
            ctl(&mut stage).saved.time == 0.,
            "Bridge began before reunion"
        );
    }
    if !skip {
        ensure!(!lines.is_empty(), "Watched scene omitted dialogue");
    }
    println!("PASS {} Stage {first:?} outro{outro} skip{skip} dead{dead}: original camera/actor, frozen partner, pause/restore, resources and reunion ({} ticks)",kind.map(),finished.unwrap());
    Ok(())
}
fn walk(
    stage: &mut Stage,
    r: &mut Roster,
    map: &Bsp,
    w: &mut World,
    id: PlayerId,
    goal: Vec3,
) -> Result<()> {
    for tick in 0..3600 {
        let player = &r.players[id.index()].player;
        let delta = goal - player.feet;
        if delta.truncate().length() < 22. && player.grounded {
            return Ok(());
        }
        let mut inputs = [(Controls::default(), Input::default()); 2];
        inputs[id.index()].0 = Controls {
            wish: delta.truncate().normalize_or_zero(),
            run: true,
            jump: player.grounded && delta.z > 18. && delta.truncate().length() < 180.,
            ..Default::default()
        };
        stage.before_move(r, map, w, &inputs, FIXED_DT)?;
        ensure!(
            !stage.busy(),
            "Unexpected scene/exit before a departure step"
        );
        let before = r.players.each_ref().map(|p| p.player.feet);
        r.move_players(w, inputs);
        stage.contacts(r, w, before)?;
        stage.speech_step(r, w, false);
        ensure!(
            r.players
                .iter()
                .all(|p| p.stats.alive() && w.body_clear(p.player.feet)),
            "Departure walker died or embedded at tick{tick}, {:?}",
            r.players.each_ref().map(|p| p.player.feet)
        );
        if stage.transition.is_some() {
            return Ok(());
        }
    }
    anyhow::bail!(
        "Departure walker {id:?} stopped at {:?}, goal{goal:?}",
        r.players[id.index()].player.feet
    )
}
fn departure(a: &mut Assets, map: &Bsp, first: PlayerId) -> Result<()> {
    let (mut w, mut stage, mut r) = setup(a, map, Kind::Grounds, first, false)?;
    coop_world_check::victory(ctl(&mut stage));
    ctl(&mut stage).rebuild(map)?;
    stage.owner = None;
    stage.story.finish_sequence(Kind::Grounds.intro());
    stage.story.finish_sequence(Kind::Grounds.outro());
    stage.interactions.sync_coop(&mut w);
    let at = ctl(&mut stage).data.points["alice_end1"].translation;
    let people = coop_world_check::pair(&w, at)?;
    for (body, player) in r.players.iter_mut().zip(people) {
        body.player = player;
    }
    for tick in 0..1600 {
        stage.before_move(&mut r, map, &mut w, &Default::default(), FIXED_DT)?;
        r.move_players(&w, Default::default());
        if tick == 600 {
            checkpoint(&mut stage, &mut r, map, &mut w)?;
        }
    }
    ensure!(
        ctl(&mut stage).saved.phase == Phase::Done,
        "Bridge never finished lowering"
    );
    for id in [first, first.partner()] {
        let y = 192. + if id == first { -48. } else { 48. };
        for x in [150., 750., 1300.] {
            let goal = coop_world_check::floor(&w, vec3(x, y, 0.))?.feet;
            walk(&mut stage, &mut r, map, &mut w, id, goal)?;
            ensure!(
                stage.transition.is_none(),
                "Castle exit occurred before bridge crossing"
            );
        }
        for _ in 0..250 {
            stage.before_move(&mut r, map, &mut w, &Default::default(), FIXED_DT)?;
            r.move_players(&w, Default::default());
        }
    }
    ensure!(
        ctl(&mut stage).saved.door == 2.,
        "Exit gate did not open for walkers"
    );
    checkpoint(&mut stage, &mut r, map, &mut w)?;
    let goal = coop_world_check::floor(&w, vec3(1700., 192., 0.))?.feet;
    walk(&mut stage, &mut r, map, &mut w, first, goal)?;
    ensure!(
        stage
            .transition
            .as_ref()
            .is_some_and(|e| Kind::Grounds.exit().matches(e))
            && stage.speaker() == first
            && r.players.iter().all(|p| p.stats.alive()),
        "Missing living pair's normal castle departure"
    );
    println!("PASS grounds1 post-battle {first:?}-first native movement across bridge, gate, restored progress and actual exit trigger");
    Ok(())
}
pub(crate) fn check(a: &mut Assets) -> Result<()> {
    for kind in [Kind::Lair, Kind::Grounds] {
        let map = Bsp::parse(&a.read(&format!("maps/{}.bsp", kind.map()))?)?;
        for first in PlayerId::ALL {
            for outro in [false, true] {
                for skip in [false, true] {
                    for dead in [false, true] {
                        scene(a, &map, kind, first, outro, skip, dead)?;
                    }
                }
            }
        }
        if kind == Kind::Grounds {
            for first in PlayerId::ALL {
                departure(a, &map, first)?;
            }
        }
    }
    Ok(())
}
