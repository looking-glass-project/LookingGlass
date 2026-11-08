//! Native regression for the real Dice pickup -> scene -> ambush path.
use super::{protocol::Visit, stage::Stage, world::Roster, PlayerId};
use crate::{assets::Assets, bsp::Bsp, collision::World,
    inventory::{Catalog, Stats}, movement::{Player, EYE_HEIGHT}};
use anyhow::{ensure, Context, Result};
use macroquad::prelude::*;

pub(super) fn check(assets: &mut Assets) -> Result<()> {
    let map = Bsp::parse(&assets.read("maps/skool2.bsp")?)?;
    let catalog = Catalog::load(assets)?;
    let pickups = crate::inventory::pickups_for_visit(&map, "skool2", None, &catalog);
    let dice = pickups.iter().find(|p| p.id == "skool2:44").context("Missing authored Demon Dice")?;
    let visit = Visit { epoch:1, map:"skool2".into(), entry:None,
        difficulty:Default::default(), combat_preview:false };
    for first in PlayerId::ALL {
        let mut world = World::from_bsp(&map)?;
        let mut stage = Stage::load(assets, &map, &mut world, &visit, false)?;
        let (eye, yaw) = crate::interaction::spawn(&map, None);
        let mut roster = Roster::new(&world, eye, yaw, Stats::for_level("skool2", None))?;
        // Only this native component stages a body at the real pickup. The
        // two-client route must reach it from the entrance by ordinary inputs.
        let altar = vec3(-906., -3170., -191.96875);
        ensure!(world.body_clear(altar) && crate::recovery::supported(&world, altar),
            "Dice fixture altar is not supported");
        roster.players[first.index()].player = Player::spawn(&world, altar + Vec3::Z*EYE_HEIGHT)
            .context("Cannot stage Dice collector")?;
        let partner = first.partner().index();
        let waiting = roster.players[partner].player.feet;
        stage.collect_pickups(&mut roster, &world, std::slice::from_ref(dice));
        ensure!(roster.players[first.index()].stats.copies(6) == 1
            && roster.players[partner].stats.copies(6) == 0,
            "Dice pickup was not personal for {first:?}");
        ensure!(stage.owner == Some(first)
            && stage.interactions.school2.as_ref().unwrap().scene_id() == Some("dice_cat")
            && stage.story.sequence_pending("dice_cat"),
            "First Dice collector did not start and own the original scene: {first:?}");
        let mut mirror_world = World::from_bsp(&map)?;
        let mut mirror = Stage::load(assets, &map, &mut mirror_world, &visit, false)?;
        let mut count = 0;
        while stage.busy() && count < 120*180 {
            stage.scene_step(&mut roster, &map, &mut world, &Default::default())?;
            count += 1;
            if stage.busy() {
                ensure!(stage.owner == Some(first) && roster.players[partner].player.feet == waiting,
                    "Dice scene changed owner or moved its waiting partner");
            }
            if count % 120 == 0 || !stage.busy() {
                let bytes = super::codec::encode(&serde_json::to_vec(&stage.snapshot())?)?;
                let state = serde_json::from_slice(&super::codec::decode(&bytes)?)?;
                mirror.restore(&state, &map, &mut mirror_world)?;
                ensure!(serde_json::to_value(mirror.snapshot())? == serde_json::to_value(stage.snapshot())?,
                    "Dice scene replica diverged");
            }
        }
        ensure!(!stage.busy() && !stage.story.sequence_pending("dice_cat"), "Dice scene did not finish");
        let school = stage.interactions.school2.as_ref().unwrap();
        ensure!(school.diagnostic_guards().filter(|(_,name,_,_)|name.starts_with("club_spawn"))
            .all(|(_,_,active,_)|active), "Completed Dice scene did not release its ambush");
        ensure!(roster.players.iter().all(|b| b.stats.alive() && b.spawned_at == 0
            && b.warped_at > 0 && world.body_clear(b.player.feet)
            && crate::recovery::supported(&world,b.player.feet)), "Dice reunion was unsafe");
        ensure!((36. ..=110.).contains(&roster.players[0].player.feet.distance(roster.players[1].player.feet)),
            "Dice scene did not reunite separated players nearby");
        // The other player gets their own copy later, without replaying the
        // completed shared scene or changing its owner to the second collector.
        roster.players[first.index()].player = Player::spawn(&world, eye).context("Cannot restore waiting position")?;
        roster.players[partner].player = Player::spawn(&world, altar + Vec3::Z*EYE_HEIGHT)
            .context("Cannot stage second Dice collector")?;
        for _ in 0..2 { stage.collect_pickups(&mut roster, &world, std::slice::from_ref(dice)); }
        ensure!(roster.players.iter().all(|b| b.stats.copies(6) == 1 && b.stats.collected.contains(&dice.id))
            && !stage.busy() && !stage.story.sequence_pending("dice_cat") && stage.speaker() == first,
            "Second personal Dice pickup replayed or reassigned the shared scene");
        println!("PASS co-op Dice first {first:?}: personal pickup, original scene/owner, frozen partner, replica, reunion, ambush and no replay ({count} ticks)");
    }
    check_cramped_reunion(assets, &map, &visit, dice)?;
    check_gym_departure(assets, &map, &visit)?;
    Ok(())
}

/// A recorded gym departure finishes its animation with no new placement
/// available, although the previous Gnome reunion left the partner safely nearby.
fn check_gym_departure(assets: &mut Assets, map: &Bsp, visit: &Visit) -> Result<()> {
    for first in PlayerId::ALL {
        let (interactions, mut world, _, story) =
            crate::school2::cinema_check::setup(assets,map,crate::school2::cinema::Beat::Warp)?;
        let mut stage=Stage::load(assets,map,&mut world,visit,false)?;
        stage.interactions=interactions;
        stage.story=story;
        stage.interactions.sync_coop(&mut world);
        let (eye,yaw)=crate::interaction::spawn(map,None);
        let mut roster=Roster::new(&world,eye,yaw,Stats::for_level("skool2",None))?;
        let owner_at=vec3(2075.9312,-3088.9688,440.03125);
        let partner_at=vec3(1973.2809,-3126.7268,448.03125);
        for (id,at) in [(first,owner_at),(first.partner(),partner_at)] {
            roster.players[id.index()].player=Player::spawn(&world,at+Vec3::Z*EYE_HEIGHT)
                .context("Cannot stage recorded gym departure")?;
        }
        stage.accept(Default::default(),first,&mut roster,&world);
        ensure!(stage.owner==Some(first),"Gym departure did not acquire its owner");
        for _ in 0..120*30 {
            if !stage.busy() { break; }
            stage.scene_step(&mut roster,map,&mut world,&Default::default())?;
            if stage.busy() {
                ensure!(roster.players[first.partner().index()].player.feet.distance(partner_at)<0.1,
                    "Gym departure moved the waiting partner");
            }
        }
        ensure!(!stage.busy(),"Finished gym departure left co-op controls locked");
        ensure!(roster.players[first.index()].player.feet.distance(owner_at)<0.1,
            "Gym reunion moved the scene owner");
        // The owner can stand on the narrow original walkway. Only a newly
        // placed or retained partner needs the larger recovery footprint.
        ensure!(roster.players.iter().all(|b|b.player.grounded && world.body_clear(b.player.feet)
            && !stage.interactions.hazardous(b.player.feet)
            && b.stats.sanity()==100. && b.spawned_at==0 && b.warped_at>0),
            "Gym departure did not preserve safe living players: {:?}",roster.players.iter().map(|b|
                (b.player.feet,b.player.grounded,b.stats.sanity(),b.spawned_at,b.warped_at)).collect::<Vec<_>>());
        let other=&roster.players[first.partner().index()].player;
        ensure!(crate::recovery::supported(&world,other.feet) && other.feet.distance(partner_at)<0.1,
            "Gym departure did not retain its supported nearby partner: {:?}",other.feet);
        ensure!((40. ..=128.).contains(&roster.players[0].player.feet.distance(roster.players[1].player.feet)),
            "Gym departure did not reunite nearby separate players");
        println!("PASS co-op gym departure {first:?}: original scene, frozen partner, safe reunion and released controls");
    }
    Ok(())
}

/// Recorded normal pickup position that left the scene complete but its owner
/// locked: eight placement directions missed the altar's available footprint.
fn check_cramped_reunion(assets: &mut Assets, map: &Bsp, visit: &Visit,
    dice: &crate::inventory::Pickup) -> Result<()> {
    for first in PlayerId::ALL {
        let mut world = World::from_bsp(map)?;
        let mut stage = Stage::load(assets, map, &mut world, visit, false)?;
        let (eye, yaw) = crate::interaction::spawn(map, None);
        let mut roster = Roster::new(&world, eye, yaw, Stats::for_level("skool2", None))?;
        let recorded = vec3(-936.6674194335938, -3162.12744140625, -191.96875);
        roster.players[first.index()].player = Player::spawn(&world, recorded + Vec3::Z*EYE_HEIGHT)
            .context("Cannot stage recorded Dice edge pickup")?;
        let owner_feet = roster.players[first.index()].player.feet;
        ensure!(owner_feet.distance(recorded) < 0.1 && roster.players[first.index()].player.grounded,
            "Recorded Dice pickup no longer matches its playable support");
        let partner = first.partner().index();
        let waiting = roster.players[partner].player.feet;
        stage.collect_pickups(&mut roster, &world, std::slice::from_ref(dice));
        ensure!(stage.owner == Some(first), "Recorded edge pickup did not start its scene");
        for _ in 0..120*180 {
            if !stage.busy() { break; }
            stage.scene_step(&mut roster, map, &mut world, &Default::default())?;
            if stage.busy() {
                ensure!(roster.players[partner].player.feet == waiting, "Edge scene moved its waiting partner");
            }
        }
        ensure!(!stage.busy(), "Completed Dice edge scene could not reunite its partner");
        ensure!(roster.players[first.index()].player.feet.distance(owner_feet) < 0.1,
            "Dice edge reunion relocated its owner");
        let other = &roster.players[partner];
        ensure!(other.player.grounded && world.body_clear(other.player.feet)
            && crate::recovery::supported(&world, other.player.feet)
            && (36. ..=110.).contains(&other.player.feet.distance(owner_feet))
            && roster.players.iter().all(|p|p.stats.sanity() == 100. && p.spawned_at == 0 && p.warped_at > 0),
            "Dice edge reunion did not preserve safe, separate, living players");
        println!("PASS co-op Dice edge reunion {first:?}: original owner position retained, supported partner and released control");
    }
    Ok(())
}
