//! Actual-data component checks, separate from full two-client campaign proof.
use super::*;
use crate::{collision::{PLAYER_CENTER, PLAYER_HALF}, combat::PlayerTarget,
    coop::{stage::Stage, world::Roster, protocol::{Input, Visit}, PlayerId},
    movement::{Controls, FIXED_DT}};

fn pair_on(owner: &PoolTwo, map: &Bsp, world: &World, index: usize) -> Result<[Player; 2]> {
    let pad = &owner.pond.pads[index];
    let model = &map.models[pad.model];
    let center = pad.pose.point((model.min + model.max) * 0.5);
    let mut spots = Vec::new();
    let fixed = World::from_bsp(map)?;
    for x in (-144..=144).step_by(16) { for y in (-144..=144).step_by(16) {
        let high = center + vec3(x as f32, y as f32, 160.);
        let low = high - Vec3::Z * 320.;
        let hit = pad.collider.trace(high + PLAYER_CENTER, low + PLAYER_CENTER, PLAYER_HALF);
        let feet = high.lerp(low, hit.fraction);
        if !hit.start_solid && hit.fraction < 1. && hit.normal.z > 0.65 && world.body_clear(feet)
            && crate::moving::aboard(&pad.collider, feet)
            && fixed.body_trace(feet, feet - Vec3::Z * 3.).fraction >= 1. { spots.push(feet); }
    }}
    let (a,b) = spots.iter().flat_map(|a| spots.iter().map(move |b| (*a,*b)))
        .filter(|(a,b)| (48. ..=96.).contains(&a.distance(*b)))
        .min_by(|(a,b),(c,d)| (a.distance(center)+b.distance(center))
            .total_cmp(&(c.distance(center)+d.distance(center))))
        .with_context(|| format!("No two clear deck positions on {}", pad.name))?;
    Ok([a,b].map(|feet| { let mut p = Player::new(feet); p.grounded = true; p }))
}

fn rides(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for index in 0..7 { for reversed in [false, true] {
        let mut owner = PoolTwo::load(assets, map)?;
        let mut world = World::from_bsp(map)?;
        world.set_dynamic(owner.colliders());
        let mut pair = pair_on(&owner, map, &world, index)?;
        if reversed { pair.reverse(); }
        let start = pair.each_ref().map(|p| p.feet);
        let from = owner.pond.pads[index].pose;
        if index >= 5 {
            let thread = format!("leaf{}startmoving", index-4);
            ensure!(owner.coop_boarding_ready(&thread, &[start[0], start[1]+Vec3::X*1000.]) == Some(false),
                "Ride left its partner");
            ensure!(owner.coop_boarding_ready(&thread, &start) == Some(true)
                && owner.coop_boarding_ready(&thread, &start[..1]) == Some(true), "Ready riders rejected");
            owner.event(&thread).context("Missing leaf event")?;
        }
        let saved = owner.snapshot();
        owner.coop_advance(0., map, &mut world, &mut pair.each_mut(), &[])?;
        ensure!(owner.snapshot() == saved && pair.each_ref().map(|p| p.feet) == start, "Paused pond moved");
        for _ in 0..120 {
            owner.coop_advance(FIXED_DT, map, &mut world, &mut pair.each_mut(), &[])?;
            ensure!(pair.iter().all(|p| world.body_clear(p.feet)), "Pond embedded a rider");
        }
        ensure!((owner.saved.pond.age-1.).abs() < 0.001, "Pond advanced twice");
        let to = owner.pond.pads[index].pose;
        ensure!(to.translation.distance(from.translation)>0.1
            && pair.iter().zip(start).all(|(p, old)| p.feet.distance(
                to.point(from.rotation.conjugate()*(old-from.translation)))<0.1),
            "Pad {} did not carry both riders with its actual pose: start={start:?}, end={:?}, clock={:?}",
            owner.pond.pads[index].name, pair.each_ref().map(|p|p.feet), owner.saved.pond.clocks[index]);
        let mut restored = PoolTwo::load(assets, map)?;
        restored.restore(&owner.snapshot(), map)?;
        let mut other = pair.clone();
        let mut other_world = World::from_bsp(map)?;
        for _ in 0..30 {
            owner.coop_advance(FIXED_DT,map,&mut world,&mut pair.each_mut(),&[])?;
            restored.coop_advance(FIXED_DT,map,&mut other_world,&mut other.each_mut(),&[])?;
        }
        ensure!(owner.snapshot() == restored.snapshot()
            && pair.each_ref().map(|p|p.feet) == other.each_ref().map(|p|p.feet), "Pond restore diverged");
        println!("PASS Hollow pad{index}: paired carry, one clock, boarding, pause/restore; reversed={reversed}");
    }}
    Ok(())
}

fn guards(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let world = World::fixture(&[(vec3(-2000.,-2000.,-100.),vec3(2000.,2000.,0.))]);
    for near in 0..2 {
        let mut owner = PoolTwo::load(assets, map)?;
        owner.saved.guards[0].feet = vec3(0.,0.,0.03125);
        owner.saved.guards[0].yaw = 0.;
        owner.saved.guards[1].feet = vec3(-1500.,-1500.,0.03125);
        let players = std::array::from_fn(|i| PlayerTarget { eye:vec3(if i==near {80.} else {700.},0.,48.03125),
            alive:true,hidden:false });
        let saved = owner.snapshot();
        let paused = owner.coop_combat(0.,&world,players);
        ensure!(owner.snapshot()==saved && paused.players.iter().all(|p|p.damage==0.),"Paused guard advanced");
        let mut damage=[0.;2];
        for _ in 0..600 {
            for (total,out) in damage.iter_mut().zip(owner.coop_combat(FIXED_DT,&world,players).players) {
                *total += out.damage;
            }
            if damage[near]>0. { break; }
        }
        ensure!(owner.saved.guards[0].opponents.selected_player==Some(near)
            && damage[near]>0. && damage[1-near]==0.,"Guard chose wrong player: {damage:?}");
        let mut hidden=players;hidden[near].hidden=true;
        owner.coop_combat(FIXED_DT,&world,hidden);
        ensure!(owner.saved.guards[0].opponents.selected_player==Some(1-near),"Guard ignored invisibility");
        hidden[1-near].alive=false;
        owner.coop_combat(FIXED_DT,&world,hidden);
        ensure!(owner.saved.guards[0].opponents.selected_player.is_none(),"Guard targeted dead/concealed players");
        println!("PASS Hollow guard: nearer player{near}, isolated damage, concealment/death and pause");
    }
    Ok(())
}

fn blocked_start(assets: &mut Assets, map: &Bsp) -> Result<()> {
    for blocked in 0..2 {
        let mut owner=PoolTwo::load(assets,map)?;
        let mut world=World::from_bsp(map)?;world.set_dynamic(owner.colliders());
        let mut pair=pair_on(&owner,map,&world,6)?;
        let feet=pair.each_ref().map(|p|p.feet);
        let from=owner.pond.pads[6].pose;
        owner.event("leaf2startmoving").unwrap();
        // Predict only the authored pad pose, to place a fixture obstruction.
        // The production adapter must perform and reject the real body sweep.
        let mut preview=PoolTwo::load(assets,map)?;preview.restore(&owner.snapshot(),map)?;
        ensure!(preview.pond.pads[6].pose.translation==from.translation
            && preview.pond.pads[6].pose.rotation==from.rotation,"Pending departure changed on restore");
        preview.saved.pond.pending[1]=false;preview.saved.pond.clocks[6]=f64::from(FIXED_DT);preview.rebuild(map)?;
        let end=preview.pond.pads[6].pose.point(from.rotation.conjugate()*(feet[blocked]-from.translation));
        let other_end=preview.pond.pads[6].pose.point(from.rotation.conjugate()*(feet[1-blocked]-from.translation));
        let obstacle=[-14.,0.,14.].into_iter().flat_map(|x|[-14.,0.,14.].into_iter().map(move |y|vec3(x,y,0.)))
            .map(|offset| {
                let center=end+PLAYER_CENTER+offset;
                Collider::box_bounds(center-Vec3::splat(1.),center+Vec3::splat(1.))
            }).find(|obstacle|feet.iter().chain(std::iter::once(&other_end)).all(|at|
                !obstacle.touches(*at+PLAYER_CENTER,*at+PLAYER_CENTER,PLAYER_HALF)))
            .context("No isolated fixture obstruction for the turning rider")?;
        world.set_dynamic(owner.colliders().into_iter().chain([obstacle.clone()]).collect());
        ensure!(pair.iter().all(|p|world.body_clear(p.feet)),"Blocked-start fixture already overlaps rider");
        owner.coop_advance(FIXED_DT,map,&mut world,&mut pair.each_mut(),std::slice::from_ref(&obstacle))?;
        ensure!(owner.saved.pond.pending[1] && owner.saved.pond.clocks[6]==0.
            && pair.each_ref().map(|p|p.feet)==feet && owner.pond.pads[6].pose.translation==from.translation,
            "Blocked departure partly moved leaf or rider");
        let mut restored=PoolTwo::load(assets,map)?;restored.restore(&owner.snapshot(),map)?;
        ensure!(restored.pond.pads[6].pose.translation==from.translation
            && restored.pond.pads[6].pose.rotation==from.rotation,"Held departure changed after save");
        let mut other=pair.clone();let mut other_world=World::from_bsp(map)?;
        owner.coop_advance(FIXED_DT,map,&mut world,&mut pair.each_mut(),&[])?;
        restored.coop_advance(FIXED_DT,map,&mut other_world,&mut other.each_mut(),&[])?;
        ensure!(!owner.saved.pond.pending[1] && owner.saved.pond.clocks[6]>0.
            && owner.snapshot()==restored.snapshot() && pair.each_ref().map(|p|p.feet)==other.each_ref().map(|p|p.feet)
            && pair.iter().all(|p|world.body_clear(p.feet)),"Cleared departure did not resume both riders consistently");
        println!("PASS Hollow leaf2 blocked player{blocked}: atomic start, retained boarding pose, save and release");
    }
    Ok(())
}

fn boarding(assets: &mut Assets, map: &Bsp) -> Result<()> {
    use crate::interaction::coop::{Contacts, Motion};
    for index in 5..7 { for first in PlayerId::ALL { for dead in [None,Some(first),Some(first.partner())] {
        let mut interactions=Interactions::load(map)?;
        interactions.set_entry(assets,map,"potears2",None)?;
        let mut world=World::from_bsp(map)?;
        interactions.sync_coop(&mut world);
        let owner=interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<PoolTwo>()).unwrap();
        let pair=pair_on(owner,map,&world,index)?;
        let thread=format!("leaf{}startmoving",index-4);
        let entity=map.entities.iter().position(|e|e.get("thread")==Some(&thread)).context("Missing leaf trigger")?;
        let mut contacts=Contacts::new(&interactions);
        let mut feet=[crate::interaction::spawn(map,None).0;2];feet[first.index()]=pair[0].feet;
        let paths=|feet:[Vec3;2]|feet.map(|at|Some(Motion{before:at,after:at}));
        for _ in 0..30 {
            let out=interactions.triggers_pair(FIXED_DT,&mut contacts,paths(feet))?;
            ensure!(out.waiting[first.index()] && !out.activations.iter().any(|a|a.entity.0==entity),
                "Hollow leaf did not wait for partner");
        }
        let saved=interactions.snapshot();
        let mut copy:Contacts=serde_json::from_slice(&serde_json::to_vec(&contacts)?)?;
        interactions.restore(&saved,map)?;copy.validate(&interactions)?;
        feet[first.partner().index()]=pair[1].feet;
        let mut paths=paths(feet);
        if let Some(dead)=dead { paths[dead.index()]=None; }
        let out=interactions.triggers_pair(FIXED_DT,&mut copy,paths)?;
        let events=out.activations.iter().filter(|a|a.entity.0==entity).collect::<Vec<_>>();
        let expected=if dead==Some(first) {first.partner()} else {first};
        ensure!(events.len()==1 && events[0].player==expected,"Hollow leaf lost first living requester");
        let mut bodies=pair.clone();if first==PlayerId::Red {bodies.reverse();}
        let before=bodies.each_ref().map(|p|p.feet);
        for _ in 0..120 {
            let mut living=bodies.iter_mut().enumerate().filter(|(i,_)|dead.is_none_or(|id|id.index()!=*i))
                .map(|(_,p)|p).collect::<Vec<_>>();
            interactions.movers_pair(FIXED_DT,map,&mut world,&mut living)?;
        }
        ensure!(bodies.iter().enumerate().filter(|(i,_)|dead.is_none_or(|id|id.index()!=*i))
            .all(|(i,p)|p.feet.distance(before[i])>1. && world.body_clear(p.feet)),"Registered paired mover hook missed rider");
        println!("PASS Hollow leaf{} authored boarding: first={first:?}, dead={dead:?}, save and paired movement",index-4);
    }}}
    Ok(())
}

fn fish(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let visit = Visit { epoch:1,map:"potears2".into(),entry:None,difficulty:map.difficulty,combat_preview:false };
    for victim in PlayerId::ALL {
        let mut world = World::from_bsp(map)?;
        let mut stage = Stage::load(assets,map,&mut world,&visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(map,None);
        let mut roster = Roster::new_with(&world,eye,yaw,crate::inventory::Stats::default(),|_|true)?;
        let i=victim.index();let partner=1-i;
        roster.players[i].player=Player::new(vec3(-448.,192.,-272.));
        let waiting=roster.players[partner].player.feet;
        ensure!(stage.interactions.hazardous(roster.players[i].player.feet)
            && !stage.interactions.hazardous(waiting),"Fish volume absent from safe-placement checks");
        let inputs:[(Controls,Input);2]=Default::default();
        for tick in 0..725 {
            stage.before_move(&mut roster,map,&mut world,&inputs,FIXED_DT)?;
            roster.tick+=1;
            if tick==480 {
                let saved=stage.snapshot();
                let mut restored=Stage::load(assets,map,&mut world,&visit,false)?;
                restored.restore(&saved,map,&mut world)?;
                ensure!(serde_json::to_value(saved)?==serde_json::to_value(restored.snapshot())?,"Fish exposure restore changed");
                stage=restored;
            }
            if stage.busy() { break; }
        }
        ensure!(stage.owner==Some(victim),"Fish scene bound wrong player");
        let begin=stage.snapshot();
        let age=begin.world.clone();
        let mut restored=Stage::load(assets,map,&mut world,&visit,false)?;
        restored.restore(&begin,map,&mut world)?;
        stage=restored;
        for _ in 0..245 {
            stage.scene_step(&mut roster,map,&mut world,&inputs)?;
            ensure!(roster.players[partner].player.feet==waiting && roster.players[partner].stats.sanity()==100.,
                "Fish moved or hurt survivor");
            if !stage.busy() { break; }
        }
        ensure!(!stage.busy() && !roster.players[i].stats.alive(),"Fish death did not release scene control");
        ensure!(crate::coop::world::camera_player(&roster.frame(1,0),victim)==victim.partner(),"Dead camera missed survivor");
        let dead_tick=roster.tick;
        roster.finish_tick_with(&world,false,|at|!stage.interactions.hazardous(at))?;
        for _ in 0..1200 {
            roster.tick+=1;
            roster.finish_tick_with(&world,false,|at|!stage.interactions.hazardous(at))?;
            if roster.players[i].stats.alive() { break; }
        }
        ensure!(roster.players[i].stats.sanity()==25. && roster.tick-dead_tick==1200,
            "Fish respawn did not use exact ten-second/25-health recovery");
        stage.respawned(victim);
        ensure!(roster.players[partner].player.feet==waiting
            && world.body_clear(roster.players[i].player.feet),"Fish recovery moved/embedded survivor");
        let initial:serde_json::Value=serde_json::to_value(age)?;
        let after:serde_json::Value=serde_json::to_value(stage.interactions.snapshot())?;
        ensure!(initial["levels"]["potears2"]["pond"]==after["levels"]["potears2"]["pond"],
            "Fish scene moved survivor's support");
        println!("PASS Hollow fish {victim:?}: own exposure, mid-contact/scene restore, frozen survivor, death, spectator and 1200-tick respawn");
    }
    Ok(())
}

fn pickup(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let visit=Visit{epoch:1,map:"potears2".into(),entry:None,difficulty:map.difficulty,combat_preview:false};
    let mut world=World::from_bsp(map)?;
    let mut stage=Stage::load(assets,map,&mut world,&visit,false)?;
    let catalog=crate::inventory::Catalog::load(assets)?;
    let mut items=crate::inventory::pickups_for_visit(map,"potears2",None,&catalog);
    stage.interactions.sync_pickup_positions(&mut items);
    let initial=items.iter().find(|p|p.id=="potears2:36").context("Missing pad pickup")?.origin;
    for _ in 0..120 { stage.interactions.movers_pair(FIXED_DT,map,&mut world,&mut [])?; }
    stage.interactions.sync_pickup_positions(&mut items);
    let item=items.iter().find(|p|p.id=="potears2:36").unwrap();
    ensure!(item.origin.distance(initial)>1.,"Bound pickup stayed behind its pad");
    let mut mirror=Stage::load(assets,map,&mut world,&visit,false)?;
    mirror.restore(&stage.snapshot(),map,&mut world)?;
    let mut remote=crate::inventory::pickups_for_visit(map,"potears2",None,&catalog);
    mirror.interactions.sync_pickup_positions(&mut remote);
    ensure!(remote.iter().find(|p|p.id==item.id).unwrap().origin==item.origin,"Guest pickup pose diverged");
    let (eye,yaw)=crate::interaction::spawn(map,None);
    let mut roster=Roster::new_with(&world,eye,yaw,crate::inventory::Stats::default(),|_|true)?;
    let waiting=roster.players.each_ref().map(|b|b.player.clone());
    for body in &mut roster.players {body.stats.set_health(50.)?;}
    let owner=stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<PoolTwo>()).unwrap();
    let above=item.origin+Vec3::Z*100.;let below=above-Vec3::Z*200.;
    let floor=owner.pond.pads[4].collider.trace(above+PLAYER_CENTER,below+PLAYER_CENTER,PLAYER_HALF);
    ensure!(!floor.start_solid && floor.fraction<1. && floor.normal.z>0.65,"No pickup deck");
    let feet=above.lerp(below,floor.fraction);
    for collector in 0..2 {
        roster.players[collector].player=Player::new(feet);
        stage.collect_pickups(&mut roster,&world,&items);
        ensure!(roster.players[collector].stats.collected.contains(&item.id)
            && roster.players[collector].stats.sanity()>50.,"Player missed moving personal pickup");
        if collector==0 {ensure!(roster.players[1].stats.sanity()==50.
            && !roster.players[1].stats.collected.contains(&item.id),"Pickup entitlement leaked to partner");}
        roster.players[collector].player=waiting[collector].clone();
    }
    println!("PASS Hollow moving pad pickup: host/restored positions, separate collection and healing");
    Ok(())
}

pub fn check(assets:&mut Assets)->Result<()> {
    let map=Bsp::parse(&assets.read("maps/potears2.bsp")?)?;
    let cases:[(&str,fn(&mut Assets,&Bsp)->Result<()>);6]=[
        ("rides",rides),("boarding",boarding),("blocked start",blocked_start),
        ("guards",guards),("fish",fish),("pickup",pickup)];
    let mut failures=Vec::new();
    for (name,check) in cases {
        if let Err(error)=check(assets,&map) {
            println!("FAIL Hollow co-op {name}: {error:#}");
            failures.push(format!("{name}: {error:#}"));
        }
    }
    ensure!(failures.is_empty(),"Hollow co-op checks failed: {}",failures.join("; "));
    Ok(())
}
