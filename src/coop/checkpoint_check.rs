//! Opt-in native storage fixture; called only by --coop-import-check.
//! Real Windows sharing violations exercise failed atomic replacements. These
//! fixtures seed damage to check persistence, not to claim campaign traversal.
use super::*;
use crate::{assets::Assets, bsp::Bsp};

pub fn verify_writes(assets: &mut Assets, saved: &Checkpoint) -> Result<()> {
    let directory = crate::preferences::path("coop-write-check");
    // Refuse an existing directory even if its checkpoint is absent/damaged.
    fs::create_dir(&directory).context("Storage fixture needs a fresh isolated profile")?;
    let store = Store { path: directory.join("coop-checkpoint.json"), fingerprint: assets.fingerprint()? };
    let map = Bsp::parse(&assets.read(&format!("maps/{}.bsp", saved.visit.map))?)?;
    let mut world = World::from_bsp(&map)?;
    let mut arena = Arena::load(assets, &map, &mut world, &saved.visit, false)?;
    arena.restore_checkpoint(saved.frame.combat.as_ref().unwrap(), &map, &mut world)?;
    let mut roster = Roster::from_checkpoint(&saved.frame, &world,
        |at| !arena.stage.interactions.hazardous(at))?;
    for _ in 0..2 { store.save(&saved.visit, &mut roster, &mut arena, &world, &saved.campaign)?; }
    let baseline = store.load()?.checkpoint;
    let primary = fs::read(&store.path)?;
    let backup = fs::read(store.backup())?;
    let mut failed_replacements = 0;
    #[cfg(windows)]
    for locked_path in [store.backup(), store.path.clone()] {
        use std::os::windows::fs::OpenOptionsExt;
        arena.restore_checkpoint(baseline.frame.combat.as_ref().unwrap(), &map, &mut world)?;
        roster = Roster::from_checkpoint(&baseline.frame, &world,
            |at| !arena.stage.interactions.hazardous(at))?;
        roster.players[0].stats.damage(7.);
        roster.players[1].stats.damage(13.);
        arena.step(&mut roster, &map, &mut world, Default::default())?;
        ensure!(serde_json::to_value(arena.snapshot())? != serde_json::to_value(baseline.frame.combat.as_ref().unwrap())?,
            "Failure fixture did not advance its shared world");
        // Allow reads, but deny replacement while this handle remains open.
        let held = fs::OpenOptions::new().read(true).share_mode(1).open(&locked_path)?;
        let error = store.save(&saved.visit, &mut roster, &mut arena, &world, &saved.campaign)
            .err().context("A locked checkpoint replacement unexpectedly succeeded")?;
        drop(held);
        ensure!(error.downcast_ref::<std::io::Error>().is_some(),
            "Fixture failed validation instead of exercising a real filesystem error: {error}");
        ensure!(fs::read(&store.path)? == primary && fs::read(store.backup())? == backup,
            "Failed write changed the committed checkpoint or its backup");
        for path in [&store.path, &store.backup()] {
            ensure!(!path.with_extension(format!("json.{}.tmp", std::process::id())).exists(),
                "Failed write left its temporary file behind");
        }
        let generation = roster.checkpoint_generation;
        for body in &mut roster.players { body.stats.damage(10000.); }
        arena.step(&mut roster, &map, &mut world, Default::default())?;
        ensure!(roster.checkpoint_generation == generation + 1, "Failure fixture did not wipe");
        verify_restored(&baseline, &roster, &arena)?;
        failed_replacements += 1;
    }
    // An interrupted pre-commit write leaves a temporary file. Loading must
    // ignore it, and the next successful write must safely replace it.
    let temporary = store.path.with_extension(format!("json.{}.tmp", std::process::id()));
    fs::write(&temporary, b"{unfinished checkpoint")?;
    ensure!(!store.load()?.recovered_backup && fs::read(&store.path)? == primary,
        "An uncommitted temporary file displaced the primary checkpoint");
    arena.restore_checkpoint(baseline.frame.combat.as_ref().unwrap(), &map, &mut world)?;
    roster = Roster::from_checkpoint(&baseline.frame, &world,
        |at| !arena.stage.interactions.hazardous(at))?;
    roster.players[0].stats.damage(9.);
    roster.players[1].stats.damage(17.);
    arena.step(&mut roster, &map, &mut world, Default::default())?;
    store.save(&saved.visit, &mut roster, &mut arena, &world, &saved.campaign)?;
    ensure!(!temporary.exists(), "Successful replacement left a stale temporary file");
    let latest = store.load()?.checkpoint;
    verify_restored(&latest, &roster, &arena)?;
    for body in &mut roster.players { body.stats.damage(10000.); }
    arena.step(&mut roster, &map, &mut world, Default::default())?;
    verify_restored(&latest, &roster, &arena)?;
    // A damaged primary must never overwrite the last good backup on saving.
    fs::write(&store.path, b"{damaged primary")?;
    let recovery = store.load()?;
    ensure!(recovery.recovered_backup, "Damaged primary did not select backup");
    ensure!(serde_json::to_value(&recovery.checkpoint)? == serde_json::to_value(&baseline)?,
        "Backup did not retain the earlier full checkpoint");
    store.save(&saved.visit, &mut roster, &mut arena, &world, &saved.campaign)?;
    ensure!(fs::read(store.backup())? == backup && !store.load()?.recovered_backup,
        "Saving after recovery lost the good backup or failed to repair the primary");
    let receipt = serde_json::json!({"passed":true,"build":env!("LOOKING_GLASS_BUILD"),
        "failed_replacements":failed_replacements,"failed_write_wipes":failed_replacements,
        "stale_temporary_ignored":true,"successful_save_wipe":true,"backup_preserved":true});
    fs::write(directory.join("verified.json"), serde_json::to_vec(&receipt)?)?;
    println!("PASS co-op storage: {failed_replacements} real replacement failures retained disk and team-wipe state; stale temporary ignored; successful save updates wipe; damaged primary preserves backup");
    Ok(())
}
