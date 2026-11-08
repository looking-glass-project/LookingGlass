//! Supplied-map components; not full-route or independent-client evidence.
use super::*;
use crate::{inventory::Stats, level::CoopPlayer, movement::FIXED_DT};

fn setup(map: &Bsp) -> Result<(Maze, World, [Player; 2], [Stats; 2])> {
    let maze = Maze::load(map)?;
    let mut world = World::from_bsp(map)?;
    world.set_dynamic(maze.colliders());
    let players = [3200., 3264.].map(|x| Player::new(vec3(x, 4288., 1920.1)));
    ensure!(players.iter().all(|p| world.body_clear(p.feet)), "Maze fixture entrance blocked");
    Ok((maze, world, players, std::array::from_fn(|_| Stats::for_level("hedge1", None))))
}
fn tick(maze: &mut Maze, map: &Bsp, world: &mut World, players: &mut [Player; 2],
    stats: &mut [Stats; 2], dt: f32, fixed: &[Collider]) -> Result<()> {
    let [p0, p1] = players.each_mut();
    let [s0, s1] = stats.each_mut();
    maze.coop_advance_roster(dt, map, world, &mut [
        CoopPlayer { player: p0, stats: s0 }, CoopPlayer { player: p1, stats: s1 },
    ], fixed)
}
fn plate(maze: &Maze, slot: usize) -> Player {
    let mut p = Player::new(vec3(3112. + slot as f32 * 48., 4096., maze.data.doors[0].top + 0.03125));
    p.grounded = true;
    p
}
fn child() -> CompanionContact {
    CompanionContact { name: "seek_kid_chase".into(), feet: vec3(3136., 4096., 1944.),
        half: vec3(16., 16., 30.), visible: true, holding: true }
}

pub(super) fn check(a: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&a.read("maps/hedge1.bsp")?)?;
    for hz in [30, 60, 144] { for first in 0..2 {
        let (mut maze, mut world, mut players, mut stats) = setup(&map)?;
        players[first] = plate(&maze, first);
        ensure!(world.body_clear(players[first].feet), "Plate start obstructed");
        let (mut solo, mut solo_world, _, _) = setup(&map)?;
        let mut alice = players[first].clone();
        for _ in 0..hz {
            tick(&mut maze, &map, &mut world, &mut players, &mut stats, 1./hz as f32, &[])?;
            solo.advance(1./hz as f32, &map, &mut solo_world, &mut alice, &[])?;
            ensure!(maze.snapshot() == solo.snapshot() && players[first].feet == alice.feet,
                "Maze shared clock or carrying differs from solo");
        }
        ensure!(maze.saved.doors == [1.; 3] && !maze.ready()
            && maze.exit_contact(&EXIT.destination()).unwrap().transition.is_none(),
            "Personal plate failed or bypassed the child's escort");
        let saved = maze.snapshot();
        let bodies = serde_json::to_value((&players, &stats))?;
        for dt in [0., -1., f32::NAN] {
            tick(&mut maze, &map, &mut world, &mut players, &mut stats, dt, &[])?;
        }
        ensure!(maze.snapshot() == saved && serde_json::to_value((&players, &stats))? == bodies,
            "Paused paired gate moved");
        let mut restored = Maze::load(&map)?; restored.restore(&saved, &map)?;
        let mut copies = players.clone(); let mut resources = stats.clone();
        let mut restored_world = World::from_bsp(&map)?;
        for _ in 0..hz {
            tick(&mut maze, &map, &mut world, &mut players, &mut stats, 1./hz as f32, &[])?;
            tick(&mut restored, &map, &mut restored_world, &mut copies, &mut resources, 1./hz as f32, &[])?;
        }
        ensure!(maze.snapshot() == restored.snapshot() && serde_json::to_value((&players, &stats))?
            == serde_json::to_value((&copies, &resources))?, "Maze restored continuation differs");
        println!("PASS paired Maze {hz}Hz slot{first}: real plate carry, solo clock parity, escort gate, pause and restore");
    }}
    for blocker in 0..2 {
        let (mut maze, mut world, _, mut stats) = setup(&map)?;
        let mut players = std::array::from_fn(|i| plate(&maze, i));
        let before = players.each_ref().map(|p| p.feet);
        let at = before[blocker];
        let fixed = [Collider::box_bounds(at - vec3(10., 10., 2.), at + vec3(10., 10., -0.1))];
        world.set_dynamic(fixed.iter().cloned().chain(maze.colliders()).collect());
        ensure!(players.iter().all(|p| world.body_clear(p.feet)), "Blocked-carry fixture starts obstructed");
        tick(&mut maze, &map, &mut world, &mut players, &mut stats, FIXED_DT, &fixed)?;
        ensure!(maze.saved.doors[0] == 0. && before == players.each_ref().map(|p| p.feet),
            "Blocked plate moved a rider before checking the partner");
        for _ in 0..120 { tick(&mut maze, &map, &mut world, &mut players, &mut stats, FIXED_DT, &[])?; }
        ensure!(maze.saved.doors[0] == 1. && players.iter().zip(before).all(|(p, old)|
            p.feet.distance(old + maze.data.doors[0].slide) < 0.001), "Unblocked plate did not carry both riders");
        println!("PASS paired Maze blocker{blocker}: atomic blocked carry and both riders released");
    }
    for dead in 0..2 {
        let (mut maze, mut world, mut players, mut stats) = setup(&map)?;
        players[dead] = plate(&maze, dead); stats[dead].damage(1000.);
        let body = serde_json::to_value(&players[dead])?;
        for _ in 0..120 { tick(&mut maze, &map, &mut world, &mut players, &mut stats, FIXED_DT, &[])?; }
        ensure!(maze.saved.doors == [0.; 3] && serde_json::to_value(&players[dead])? == body,
            "Dead body pressed the plate or was carried");
        maze.companion_contacts(&[child()]);
        for _ in 0..120 { tick(&mut maze, &map, &mut world, &mut players, &mut stats, FIXED_DT, &[])?; }
        ensure!(maze.ready() && maze.exit_contact(&EXIT.destination()).unwrap().transition == Some(EXIT.destination()),
            "Real child occupancy failed to enable shared departure");
        ensure!(maze.coop_events().transition.is_none(),
            "Paired exit emitted twice");
        maze.transition_failed(&EXIT.destination());
        for _ in 0..124 { tick(&mut maze, &map, &mut world, &mut players, &mut stats, FIXED_DT, &[])?; }
        ensure!(maze.coop_events().transition == Some(EXIT.destination()),
            "Paired exit retry stalled");
        println!("PASS paired Maze dead{dead}: dead-body exclusion, child-owned Hold, single departure and retry");
    }
    for blocker in 0..2 {
        let (mut maze, mut world, mut players, mut stats) = setup(&map)?;
        maze.saved.doors = [1.; 3];
        world.set_dynamic(maze.colliders());
        players[blocker].feet = vec3(2912., 4288., 1920.1);
        ensure!(world.body_clear(players[blocker].feet), "Open gate test is blocked");
        for _ in 0..120 {
            tick(&mut maze, &map, &mut world, &mut players, &mut stats, FIXED_DT, &[])?;
            ensure!(players.iter().all(|p| world.body_clear(p.feet)), "Closing gate crushed a player");
        }
        ensure!(maze.saved.doors[1..].iter().any(|p| *p > 0.), "Blocked gate ignored a living player");
        players[blocker].feet = vec3(3200. + blocker as f32 * 64., 4288., 1920.1);
        for _ in 0..120 { tick(&mut maze, &map, &mut world, &mut players, &mut stats, FIXED_DT, &[])?; }
        ensure!(maze.saved.doors == [0.; 3], "Gate stayed blocked after player left");
        println!("PASS paired Maze gate blocker{blocker}: safe closing and release");
    }
    Ok(())
}

pub(super) async fn cast(a: &mut Assets) -> Result<()> {
    use crate::coop::{arena::Arena, protocol::Visit, world::Roster};
    let map = Bsp::parse(&a.read("maps/hedge1.bsp")?)?;
    let visit = Visit { epoch: 1, map: "hedge1".into(), entry: None,
        difficulty: Default::default(), combat_preview: false };
    let mut world = World::from_bsp(&map)?;
    let mut arena = Arena::load(a, &map, &mut world, &visit, false)?;
    let mut roster = Roster::new(&world, vec3(3200., 4288., 1968.), 0., Stats::for_level("hedge1", None))?;
    let initial = arena.snapshot();
    for restore in [false, true] {
        if restore { arena.restore_checkpoint(&initial, &map, &mut world)?; }
        let expected = arena.npcs.companions();
        let actors = arena.npcs.targets();
        ensure!(!expected.is_empty() && !actors.is_empty(), "Cast contact fixture is empty");
        arena.step(&mut roster, &map, &mut world, Default::default())?;
        let maze = arena.stage.interactions.levels.iter().find_map(|s| s.ctl.downcast_ref::<Maze>()).unwrap();
        ensure!(maze.companions.len() == expected.len() && maze.companions.iter().zip(&expected).all(|(a,b)|
            a.name == b.name && a.feet == b.feet && a.half == b.half && a.visible == b.visible && a.holding == b.holding)
            && maze.actors.len() == actors.len() && maze.actors.iter().zip(&actors).all(|(a,b)|
                a.id == b.id && a.center == b.center && a.half == b.half),
            "Production Arena did not supply the actual restored cast before moving gates");
        println!("PASS paired Maze production Arena cast/actor contacts restore={restore}");
    }
    for (thread, stage) in [("SeekEnd", 2), ("SeekStart", 1)] { for first in 0..2 {
        arena.restore_checkpoint(&initial, &map, &mut world)?;
        let e = map.entities.iter().find(|e| e.get("thread")
            .is_some_and(|s| s.trim_end_matches("()") == thread)).context("Missing escort trigger")?;
        let shape = Collider::model(&map, e["model"].trim_start_matches('*').parse()?,
            e.get("origin").and_then(|s| crate::interaction::vector(s)).unwrap_or(Vec3::ZERO), Quat::IDENTITY, false)?;
        let eye = shape.interior_point().context("Empty escort trigger")? + Vec3::Z * 20.;
        let mut targets = [crate::combat::PlayerTarget { eye: vec3(3200., 4288., 1968.), alive: true, hidden: false }; 2];
        targets[first].eye = eye;
        arena.npcs.update_players(FIXED_DT, &world, targets);
        let snapshot = serde_json::to_value(arena.npcs.snapshot())?;
        let child = snapshot["actors"].as_array().unwrap().iter()
            .find(|v| v["spawn"]["name"] == "seek_kid").context("Missing native child")?;
        ensure!(child["guide"]["stage"] == stage, "Native child missed {thread} from slot{first}");
        let after = arena.npcs.snapshot();
        arena.npcs.replicate(&after)?;
        ensure!(serde_json::to_value(arena.npcs.snapshot())? == snapshot, "Paired child replica changed");
        println!("PASS paired Maze actual native {thread} slot{first}: one shared child and exact replica");
    }}
    // A failed destination is a shared output. It must retry without either
    // player pressing E, and cannot be emitted again after being accepted.
    arena.restore_checkpoint(&initial, &map, &mut world)?;
    let maze = arena.stage.interactions.levels.iter_mut().find_map(|s| s.ctl.downcast_mut::<Maze>()).unwrap();
    maze.saved.following = true; maze.saved.start_seen = true; maze.saved.holding = true;
    maze.saved.doors = [1.; 3];
    ensure!(maze.exit_contact(&EXIT.destination()).unwrap().transition == Some(EXIT.destination()), "Missing initial departure");
    maze.transition_failed(&EXIT.destination());
    maze.saved.exit.advance(1.1);
    ensure!(arena.stage.interactions.advance_shared(0.).transition.is_none(), "Paused exit retried");
    ensure!(arena.stage.interactions.advance_shared(FIXED_DT).transition == Some(EXIT.destination())
        && arena.stage.interactions.advance_shared(FIXED_DT).transition.is_none(), "Shared exit retry absent or duplicated");
    println!("PASS paired Maze automatic exit retry: pause, no use press, single shared output");
    Ok(())
}
