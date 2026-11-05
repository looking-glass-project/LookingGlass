//! Copy a loaded solo campaign into a separate host-owned co-op checkpoint.
//! Conversion is in memory: failure cannot modify either set of save files.
use super::{arena::Arena, campaign::{Campaign, VisitState}, checkpoint::Checkpoint,
    protocol::Visit, stage::Stage, world::Roster};
use crate::{assets::Assets, bsp::Bsp, collision::World, save};
use anyhow::{ensure, Result};

pub fn convert(assets: &mut Assets, game: save::Game) -> Result<Checkpoint> {
    ensure!(!game.view.flying, "Leave fly mode before hosting this game");
    ensure!(game.stats.alive(), "Retry or load a living Alice before hosting this game");
    // Use the normal reader's validation and migrations, including cached visits.
    let mut restored = save::Restored::build(assets, game)?;
    ensure!(!restored.interactions.scripted() && !restored.story.busy(),
        "Finish the scene or conversation before hosting this game");
    let mut level = restored.game.level()?.clone();
    level.interactions = restored.interactions.snapshot();
    level.npcs = restored.npcs.snapshot();
    level.story = restored.story.snapshot();
    level.hints = restored.hints.snapshot();
    restored.game.campaign.levels.insert(level.key(), level.clone());
    let difficulty = restored.game.stats.difficulty;
    let visit = visit_for(&level, difficulty);
    let mut arena = Arena::load(assets, &restored.scene.map, &mut restored.scene.world, &visit, false)?;
    arena.stage.import_single_player(&level, &restored.scene.map, &mut restored.scene.world, true)?;
    ensure!(!arena.stage.busy() && arena.stage.interactive_actor().is_none(),
        "Finish the scene before hosting this game");
    arena.npcs.restore(&level.npcs)?;
    let roster = Roster::from_single_player(&restored.scene.world, &restored.game.player,
        &restored.game.stats, restored.game.view.yaw, restored.game.view.pitch,
        |at| !arena.stage.interactions.hazardous(at))?;
    arena.reset_characters(&roster.players);
    // Retain Blue's animation, actions and projectiles; Red starts with her own
    // idle character, so a thrown weapon or summon is never duplicated.
    arena.characters[0].restore(&restored.alice.snapshot())?;
    let mut states = Vec::new();
    for cached in restored.game.campaign.levels.values() {
        let cached_visit = visit_for(cached, difficulty);
        let stage = if cached.key() == level.key() {
            arena.stage.snapshot()
        } else {
            let mut map = Bsp::parse(&assets.read(&format!("maps/{}.bsp", cached.map))?)?;
            map.difficulty = difficulty;
            let mut world = World::from_bsp(&map)?;
            let mut stage = Stage::load(assets, &map, &mut world, &cached_visit, false)?;
            stage.import_single_player(cached, &map, &mut world, false)?;
            stage.snapshot()
        };
        states.push(VisitState { visit: cached_visit, stage, npcs: cached.npcs.clone() });
    }
    let campaign = Campaign::imported(states, restored.game.campaign.completed.clone(), difficulty)?;
    let mut frame = roster.frame(visit.epoch, 0);
    frame.combat = Some(Box::new(arena.snapshot()));
    let checkpoint = Checkpoint { visit, frame, campaign };
    checkpoint.validate()?;
    Ok(checkpoint)
}

fn visit_for(level: &save::Level, difficulty: crate::powerups::Difficulty) -> Visit {
    Visit { epoch: 1, map: level.map.clone(), entry: level.entry.clone(),
        difficulty, combat_preview: false }
}
