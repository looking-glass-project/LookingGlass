//! Native persistence fixture. Uses isolated profiles; not campaign traversal proof.
use crate::{assets::Assets, character::Character, cheshire::Hints, interaction::Interactions,
    inventory::{self, Catalog, Stats}, movement::{Controls, Player, FIXED_DT}, npc::Npcs,
    render::Scene, save::{self, Game, Level, Slot, Store, View}, story::Story, Options};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;

fn same(a: &impl serde::Serialize, b: &impl serde::Serialize, label: &str) -> Result<()> {
    ensure!(serde_json::to_value(a)? == serde_json::to_value(b)?, "Import changed {label}");
    Ok(())
}

pub async fn check(assets: &mut Assets, options: &Options) -> Result<()> {
    ensure!(options.save_dir != std::path::Path::new("private/saves")
        && !options.save_dir.join("slot1.json").exists()
        && !options.save_dir.join("slot1.previous.json").exists(),
        "Run the import fixture with a fresh isolated --profile; existing saves are never replaced");
    clear_background(BLACK);
    next_frame().await;
    let mut scene = Scene::load_for(assets, "skool1", Default::default())?;
    let mut interactions = Interactions::load(&scene.map)?;
    interactions.set_entry(assets, &scene.map, "skool1", None)?;
    interactions.sync(&mut scene.world);
    let mut player = Player::spawn(&scene.world, crate::interaction::spawn(&scene.map, None).0)
        .context("Solo fixture entrance obstructed")?;
    let entrance = player.feet;
    // The same native solo movement/use path opens the entrance door.
    for tick in 0..1200 {
        let wish = if (120..960).contains(&tick) { Vec2::Y } else { Vec2::ZERO };
        player.tick(&scene.world, Controls { wish, ..Default::default() });
        interactions.update(FIXED_DT, &scene.map, &mut scene.world, &player, Vec3::Y, tick == 1000)?;
    }
    ensure!(interactions.used > 0 && player.feet.distance(entrance) > 100.,
        "Solo fixture did not move and open its door");
    let mut stats = Stats::for_level("skool1", None);
    stats.apply(inventory::PickupKind::Weapon(6), 1.);
    let catalog = Catalog::load(assets)?;
    for item in inventory::pickups(&scene.map, "skool1", &catalog) {
        inventory::collect(&mut stats, std::slice::from_ref(&item), item.origin, &scene.world);
        if !stats.collected.is_empty() { break; }
    }
    ensure!(!stats.collected.is_empty(), "Fixture needs an earned pickup entitlement");
    stats.set_health(83.)?;
    stats.spend_will(19.);
    let story = Story::load(assets, "skool1");
    let hints = Hints::load(assets, &scene.map, "skool1")?;
    let npcs = Npcs::load(assets, &scene.map, "skool1", None, false, false)?;
    let mut alice = Character::load(assets)?;
    alice.reset(&player, 1.2);
    let level = Level { map: "skool1".into(), entry: None, interactions: interactions.snapshot(),
        npcs: npcs.snapshot(), story: story.snapshot(), hints: hints.snapshot(),
        environment_clock: 10., pickup_clock: 10. };
    let current = level.key();
    let mut campaign = save::Campaign::default();
    campaign.levels.insert(current.clone(), level.clone());
    // A separate cached return visit must retain its own controller state.
    let mut previous = Interactions::load(&scene.map)?;
    previous.set_entry(assets, &scene.map, "skool1", Some("skool1_start2"))?;
    let mut returning = level.clone();
    returning.entry = Some("skool1_start2".into());
    returning.interactions = previous.snapshot();
    returning.npcs = Npcs::load(assets, &scene.map, "skool1", Some("skool1_start2"), false, false)?.snapshot();
    campaign.completed.insert(returning.key());
    campaign.levels.insert(returning.key(), returning.clone());
    let game = Game { current, campaign, stats, player: player.clone(),
        view: View { position: player.eye(), yaw: 1.2, pitch: 0.23, third_person: true,
            flying: false, fullbright: false, spawn_landing: false },
        recovery: Default::default(), character: alice.snapshot() };
    // Exercise the current ordinary-save format, including its language-neutral
    // identity. Import must accept this as well as legacy identities.
    let store = Store::for_assets(options.save_dir.clone(), assets)?;
    store.write(Slot::One, &game)?;
    let path = options.save_dir.join("slot1.json");
    let original = std::fs::read(&path)?;
    let converted = super::solo::convert(assets, store.read(Slot::One)?.game)?;
    let blue = &converted.frame.players[0];
    let red = &converted.frame.players[1];
    same(&blue.player, &game.player, "Blue's full body")?;
    same(&blue.stats, &game.stats, "Blue's resources")?;
    same(&red.stats, &game.stats, "Red's earned inventory")?;
    ensure!(blue.yaw == game.view.yaw && blue.pitch == game.view.pitch, "Import lost camera facing");
    ensure!(blue.player.feet.distance(red.player.feet) >= 40., "Imported bodies overlap");
    let combat = converted.frame.combat.as_ref().unwrap();
    same(&combat.stage.world, &level.interactions, "shared doors and quests")?;
    same(&combat.stage.story, &level.story, "story")?;
    same(&combat.stage.hints, &level.hints, "hints")?;
    same(&combat.npcs, &level.npcs, "NPCs")?;
    same(&combat.characters[0], &game.character, "Blue's character")?;
    let cached = converted.campaign.cached(&super::protocol::Visit {
        entry: returning.entry.clone(), ..converted.visit.clone()
    }).context("Cached return visit lost")?;
    same(&cached.stage.world, &returning.interactions, "cached return visit")?;
    ensure!(converted.campaign.completed() == 1, "Import incorrectly completed the current visit");
    let mut independent = red.stats.clone();
    independent.damage(17.);
    independent.spend_will(3.);
    same(&blue.stats, &game.stats, "independent resources")?;
    for dead in [false, true] {
        let mut invalid = game.clone();
        if dead { invalid.stats.damage(10000.); } else { invalid.view.flying = true; }
        ensure!(super::solo::convert(assets, invalid).is_err(), "Unsafe solo import succeeded");
    }
    ensure!(std::fs::read(&path)? == original, "Import modified its solo save");
    std::fs::write(crate::preferences::path("coop-import-expected.json"), serde_json::to_vec(&converted)?)?;
    println!("PASS solo import: moved body, camera, earned inventory, opened door, story, NPCs, character and cached return visit; safe separated Red; rejected dead/flying imports; solo file unchanged");
    super::checkpoint::verify_writes(assets, &converted)?;
    check_swimmer(assets)?;
    Ok(())
}

fn check_swimmer(assets: &mut Assets) -> Result<()> {
    let mut scene = Scene::load_for(assets, "utemple", Default::default())?;
    let mut interactions = Interactions::load(&scene.map)?;
    interactions.set_entry(assets, &scene.map, "utemple", None)?;
    interactions.sync(&mut scene.world);
    let mut player = Player::spawn(&scene.world, crate::interaction::spawn(&scene.map, None).0)
        .context("Swimming fixture entrance obstructed")?;
    let mut stats = Stats::for_level("utemple", None);
    for _ in 0..120 {
        interactions.prepare_player(&mut stats, &mut player);
        player.tick(&scene.world, Controls::default());
    }
    stats.set_health(63.)?;
    stats.spend_will(11.);
    ensure!(player.swimming, "Swimming fixture is not in water");
    let mut alice = Character::load(assets)?;
    alice.reset(&player, 0.7);
    let level = Level { map: "utemple".into(), entry: None, interactions: interactions.snapshot(),
        npcs: Npcs::load(assets, &scene.map, "utemple", None, false, false)?.snapshot(),
        story: Story::load(assets, "utemple").snapshot(),
        hints: Hints::load(assets, &scene.map, "utemple")?.snapshot(),
        environment_clock: 1., pickup_clock: 1. };
    let mut campaign = save::Campaign::default();
    campaign.levels.insert(level.key(), level.clone());
    let game = Game { current: level.key(), campaign, stats, player: player.clone(),
        view: View { position: player.eye(), yaw: 0.7, pitch: -0.2, third_person: true,
            flying: false, fullbright: false, spawn_landing: false },
        recovery: Default::default(), character: alice.snapshot() };
    let checkpoint = super::solo::convert(assets, game)?;
    let blue = &checkpoint.frame.players[0].player;
    let red = &checkpoint.frame.players[1].player;
    same(blue, &player, "swimmer body and air supply")?;
    ensure!(red.swimming && (red.feet.z-blue.feet.z).abs() < 0.1
        && blue.feet.distance(red.feet) >= 40., "Swimming partner lost depth or overlaps");
    println!("PASS solo swimming import: original body and air supply retained; separated Red at the same water depth");
    Ok(())
}
