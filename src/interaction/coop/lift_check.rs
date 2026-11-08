use super::*;
use crate::{
    assets::Assets,
    movement::FIXED_DT,
    school::coop_check::{riders, LIFTS},
};
use anyhow::Context;

pub fn check(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let mut lifts = LIFTS.to_vec();
    lifts.push((
        "Skool1_OLift_Up",
        "observatory_lift",
        vec3(692., 4370., 384.04),
        1016.,
    ));
    for (thread, name, start, _) in lifts {
        for first in PlayerId::ALL {
            for death in [None, Some(first.partner()), Some(first)] {
                let mut interactions = Interactions::load(map)?;
                let mut world = World::from_bsp(map)?;
                let returning = name == "observatory_lift";
                interactions.set_entry(
                    assets,
                    map,
                    "skool1",
                    returning.then_some("skool1_start2"),
                )?;
                interactions.sync(&mut world);
                if returning {
                    let mut stats =
                        crate::inventory::Stats::for_level("skool1", Some("skool1_start2"));
                    stats.school_items.star = true;
                    let school = interactions.school.as_mut().unwrap();
                    school.sync_inventory(&mut stats);
                    school
                        .event("Skool1_Setup_OLift")
                        .context("Cannot open observatory gate")?;
                    let mut outside = Player::new(map.spawn().0);
                    for _ in 0..150 {
                        interactions.advance_school(FIXED_DT, map, &mut world, &mut outside)?;
                    }
                }
                let aboard = riders(interactions.school.as_ref().unwrap(), &world, name, start)?;
                let mut feet = [Vec3::ZERO; 2];
                feet[first.index()] = aboard[0].feet;
                feet[first.partner().index()] = map.spawn().0;
                let index = interactions
                    .triggers
                    .iter()
                    .position(|t| matches!(&t.kind, TriggerKind::Script(s) if s == thread))
                    .context("Missing authored lift trigger")?;
                let id = interactions.triggers[index].id;
                ensure!(
                    interactions.triggers[index].volume.touches(
                        start + PLAYER_CENTER,
                        start + PLAYER_CENTER,
                        PLAYER_HALF
                    ),
                    "Rider outside authored {thread} contact"
                );
                let key = format!("trigger/{}", id.0);
                let mut contacts = Contacts::new(&interactions);
                let path = |feet: [Vec3; 2]| {
                    feet.map(|at| {
                        Some(Motion {
                            before: at,
                            after: at,
                        })
                    })
                };
                let before =
                    serde_json::to_value(interactions.school.as_ref().unwrap().snapshot())?;
                for _ in 0..240 {
                    let events = interactions.triggers_pair(FIXED_DT, &mut contacts, path(feet))?;
                    ensure!(
                        events.waiting[first.index()]
                            && !events.activations.iter().any(|e| e.entity == id),
                        "{thread} did not wait for partner"
                    );
                }
                ensure!(
                    before
                        == serde_json::to_value(interactions.school.as_ref().unwrap().snapshot())?
                        && interactions.event_world.snapshot().usage[&key].count == 0,
                    "Waiting consumed {thread}"
                );
                // Restore the exact shared state and per-player contacts over the
                // real externally-tagged, compressed network envelope.
                let wire = crate::coop::protocol::HostMessage::Frame {
                    frame: (interactions.snapshot(), contacts.clone()),
                };
                let encoded = crate::coop::codec::encode(&serde_json::to_vec(&wire)?)?;
                let crate::coop::protocol::HostMessage::Frame {
                    frame: (saved, saved_contacts),
                } = serde_json::from_slice::<
                    crate::coop::protocol::HostMessage<(super::super::Snapshot, Contacts)>,
                >(&crate::coop::codec::decode(&encoded)?)?
                else {
                    anyhow::bail!("Lift envelope kind changed");
                };
                interactions.restore(&saved, map)?;
                contacts = saved_contacts;
                contacts.validate(&interactions)?;
                feet[first.partner().index()] = aboard[1].feet;
                let mut paths = path(feet);
                if let Some(dead) = death {
                    paths[dead.index()] = None;
                }
                let events = interactions.triggers_pair(FIXED_DT, &mut contacts, paths)?;
                let activations = events
                    .activations
                    .iter()
                    .filter(|e| e.entity == id)
                    .collect::<Vec<_>>();
                ensure!(
                    activations.len() == 1
                        && activations[0].player
                            == if death == Some(first) {
                                first.partner()
                            } else {
                                first
                            },
                    "{thread} lost first requester {first:?} or fired more than once"
                );
                ensure!(
                    interactions.event_world.snapshot().usage[&key].count == 1,
                    "Wrong lift activation count"
                );
                ensure!(
                    contacts.triggers[index].waiting.is_none(),
                    "Fulfilled boarding request stayed pending"
                );
                println!("PASS {thread}, first {first:?}, dead {death:?}: waits without consuming contact, wire restore retains living requester, activates once");
            }
        }
    }
    Ok(())
}
