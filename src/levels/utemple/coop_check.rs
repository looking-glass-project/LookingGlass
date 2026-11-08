//! Native paired temple checks, separate from full two-client traversal.
use super::*;
use crate::{coop::{protocol::Visit, stage::Stage, world::Roster, PlayerId},
    level::CoopPlayer, movement::{Controls, FIXED_DT}};

type Pair = [(Player, Stats); 2];
fn bodies(pair: &mut Pair) -> [CoopPlayer<'_>; 2] {
    pair.each_mut().map(|(player,stats)| CoopPlayer { player, stats })
}
fn begin(a: &mut Assets, map: &Bsp) -> Result<(Temple,World,Pair)> {
    let mut temple = Temple::load(a,map)?;
    let mut world = World::from_bsp(map)?;
    world.set_dynamic(temple.colliders());
    world.set_dynamic_liquids(temple.liquids());
    let (eye,yaw) = crate::interaction::spawn(map,None);
    let roster = Roster::new(&world,eye,yaw,Stats::for_level("utemple",None))?;
    let mut pair = roster.players.map(|b| (b.player,b.stats));
    for (index,(_,stats)) in pair.iter_mut().enumerate() {
        stats.set_health(31.+index as f32*16.)?;
        stats.spend_will(13.+index as f32*16.);
    }
    temple.advance_roster(FIXED_DT,map,&mut world,&mut bodies(&mut pair),&[])?;
    Ok((temple,world,pair))
}
fn air(a: &mut Assets, map: &Bsp) -> Result<()> {
    for breathing in 0..2 {
        let (mut t, mut world, mut pair) = begin(a,map)?;
        ensure!(pair.iter().all(|(p,s)| p.breath.shell && s.sanity()==100. && s.will()==100.), "Paired arrival lacks shell/resources");
        pair[0].1.set_health(63.)?;
        pair[1].1.set_health(47.)?;
        for p in &mut pair { p.0.breath.submerged=19.; }
        t.advance_roster(FIXED_DT,map,&mut world,&mut bodies(&mut pair),&[])?;
        ensure!(pair.iter().all(|p| p.0.breath.remaining()==20.)
            && pair.each_ref().map(|p|p.1.sanity())==[63.,47.], "Waiting protection healed resources again");
        t.event("startturtle");
        let start=t.saved.guide;
        t.event("startturtle");
        ensure!(t.saved.guide==start,"Second arrival restarted guide");
        let air=t.data.emitters.iter().find(|(_,name,_,_)|name=="panel11_bubbles").unwrap().2;
        let dry=data::origin(&map.entities[17]);
        pair[breathing].0=Player::new(air-PLAYER_CENTER);
        pair[1-breathing].0=Player::new(dry);
        for p in &mut pair { p.0.breath.shell=true; p.0.breath.submerged=19.; }
        t.coop_contacts(0.,&mut bodies(&mut pair));
        ensure!(pair[breathing].0.breath.remaining()==20. && pair[1-breathing].0.breath.remaining()==1.,
            "Air contact leaked to partner or stopped with Watch");
        let visual=t.data.emitters.iter().find(|(_,_,_,air)|!*air).unwrap().2;
        pair[breathing].0.feet=visual-PLAYER_CENTER;
        pair[breathing].0.breath.submerged=19.;
        t.coop_contacts(FIXED_DT,&mut bodies(&mut pair));
        ensure!(pair[breathing].0.breath.remaining()==1., "Visual-only bubbles supplied air");
        let submerged=t.data.guide.nodes.iter().find(|n|n.0=="tp105").unwrap().2;
        pair[breathing].0=Player::new(submerged);
        pair[breathing].0.breath.shell=true;
        for _ in 0..30 { pair[breathing].0.tick(&world,Controls { swim:Vec3::Z,..Default::default() }); }
        ensure!(pair[breathing].0.immersion.level==3 && pair[breathing].0.feet.z>submerged.z+1.,
            "Paired world omitted brush swimming water");
        pair[breathing].0=Player::new(dry);
        pair[breathing].0.breath.shell=true;
        for _ in 0..120*22 {
            t.coop_contacts(FIXED_DT,&mut bodies(&mut pair));
            pair[breathing].0.tick(&world,Controls::default());
        }
        ensure!(pair[breathing].0.breath.hits>0,"Swimmer away from air could not drown");
        pair[breathing].1.set_health(25.)?;
        t.coop_respawned(breathing);
        t.coop_contacts(FIXED_DT,&mut bodies(&mut pair));
        ensure!(pair[breathing].1.sanity()==25.,"Respawn received another entry heal");
        println!("PASS temple swimmer{breathing}: one entry grant, personal air/Watch contact, visual-only exclusion, brush swimming, drowning and no respawn heal");
    }
    Ok(())
}

fn crush_positions(t:&mut Temple,map:&Bsp,w:&mut World)->Result<(Vec<Vec3>,f32)> {
    t.saved.clams[0]=Some(1.01);
    t.motion.rebuild(map,&t.saved)?;
    w.set_dynamic(t.colliders());
    let state=t.snapshot();
    t.saved.clams[0]=Some(1.11);
    t.motion.rebuild(map,&t.saved)?;
    let object=t.motion.objects.iter().find(|o|o.name=="oyster_block1").unwrap();
    let collider=object.collider.clone();
    let damage=object.damage;
    let model=&map.models[object.model];
    let lo=model.min+object.pose.translation-PLAYER_CENTER-PLAYER_HALF;
    let hi=model.max+object.pose.translation-PLAYER_CENTER+PLAYER_HALF;
    let mut future=World::from_bsp(map)?;
    future.set_dynamic(t.colliders());
    let mut candidates=Vec::new();
    'scan: for x in (lo.x.ceil() as i32..=hi.x.floor() as i32).step_by(16) {
        for y in (lo.y.ceil() as i32..=hi.y.floor() as i32).step_by(16) {
            for z in (lo.z.ceil() as i32..=hi.z.floor() as i32).step_by(8) {
                let feet=vec3(x as f32,y as f32,z as f32);
                if collider.touches(feet+PLAYER_CENTER,feet+PLAYER_CENTER,PLAYER_HALF)
                    && w.body_clear(feet) && !future.body_clear(feet) {
                    candidates.push(feet);
                    if candidates.len()==4096 { break 'scan; }
                }
            }
        }
    }
    t.restore(&state,map)?;
    ensure!(!candidates.is_empty() && damage>0.,"No native closing-clam contact fixture");
    Ok((candidates,damage))
}
fn crush(a:&mut Assets,map:&Bsp)->Result<()> {
    for victim in 0..2 {
        for dead_partner in [false,true] {
            let (mut t,mut world,mut pair)=begin(a,map)?;
            let (points,damage)=crush_positions(&mut t,map,&mut world)?;
            pair[victim].0=Player::new(points[0]);
            pair[victim].1.set_health(77.)?;
            pair[1-victim].1.set_health(if dead_partner {0.} else {39.})?;
            let other=pair[1-victim].1.sanity();
            let initial=t.snapshot();
            let feet=pair.each_ref().map(|p|p.0.feet);
            t.advance_roster(0.,map,&mut world,&mut bodies(&mut pair),&[])?;
            ensure!(t.snapshot()==initial && feet==pair.each_ref().map(|p|p.0.feet),"Paused clam changed");
            t.advance_roster(0.1,map,&mut world,&mut bodies(&mut pair),&[])?;
            ensure!(pair[victim].1.sanity()==77.-damage && pair[1-victim].1.sanity()==other,
                "Crush affected wrong slot: victim{victim}, dead partner={dead_partner}, health={:?}, damage={damage}",
                pair.each_ref().map(|p|p.1.sanity()));
            ensure!(world.body_clear(pair[victim].0.feet),"Clam embedded its victim");
            ensure!(t.update(&mut world,&pair[victim].0,Vec3::ZERO,false).damage==0.,"Crush leaked to shared event");
            let saved=t.snapshot();
            ensure!(saved["coop"]["cooldowns"][victim]==0.5,"Crush lacks personal cooldown");
            let mut copy=Temple::load(a,map)?;
            copy.restore(&serde_json::from_slice(&serde_json::to_vec(&saved)?)?,map)?;
            let mut other_pair=pair.clone();
            let mut other_world=World::from_bsp(map)?;
            for _ in 0..90 {
                t.advance_roster(FIXED_DT,map,&mut world,&mut bodies(&mut pair),&[])?;
                copy.advance_roster(FIXED_DT,map,&mut other_world,&mut bodies(&mut other_pair),&[])?;
                ensure!(t.snapshot()==copy.snapshot() && serde_json::to_value(&pair)?==serde_json::to_value(&other_pair)?,
                    "Restored clam/contact continuation differs");
            }
            let mut invalid=saved;
            invalid["coop"]["cooldowns"][victim]=serde_json::json!(0.6);
            ensure!(copy.restore(&invalid,map).is_err(),"Invalid personal cooldown accepted");
            println!("PASS temple crush player{victim} partner_dead={dead_partner}: own damage/cooldown, clear body, pause and saved continuation");
        }
    }
    Ok(())
}

fn stage_checks(a:&mut Assets,map:&Bsp)->Result<()> {
    let visit=Visit { epoch:1,map:"utemple".into(),entry:None,difficulty:Default::default(),combat_preview:false };
    let (mut t,mut world,mut pair)=begin(a,map)?;
    t.event("startturtle");
    for _ in 0..2400 {
        t.advance_roster(0.1,map,&mut world,&mut bodies(&mut pair),&[])?;
        if t.saved.open { break; }
    }
    ensure!(t.saved.open,"Paired guide never opened temple exit");
    let gate=t.snapshot();
    for first in PlayerId::ALL {
        let mut world=World::from_bsp(map)?;
        let mut stage=Stage::load(a,map,&mut world,&visit,false)?;
        let (eye,yaw)=crate::interaction::spawn(map,None);
        let mut roster=Roster::new(&world,eye,yaw,Stats::for_level("utemple",None))?;
        for _ in 0..120 {
            stage.before_move(&mut roster,map,&mut world,&Default::default(),FIXED_DT)?;
            roster.move_players(&world,Default::default());
        }
        let temple=stage.interactions.levels.iter_mut().find_map(|s|s.ctl.downcast_mut::<Temple>()).unwrap();
        ensure!((temple.saved.clock-1.).abs()<0.001 && roster.players.iter().all(|b| b.player.breath.shell),
            "Production stage failed stable-roster hook or doubled shared clock");
        temple.restore(&gate,map)?;
        stage.interactions.sync_coop(&mut world);
        let base=stage.snapshot();
        let starting=roster.players.clone();
        for skip in [false,true] {
            stage.restore(&base,map,&mut world)?;
            roster.players=starting.clone();
            let index=first.index();
            let partner=first.partner().index();
            let exit=vec3(-2144.,3352.,480.);
            ensure!(world.body_clear(exit),"Native temple exit fixture blocked");
            roster.players[index].player=Player::new(exit);
            let waiting=roster.players[partner].player.feet;
            let mut before=roster.players.each_ref().map(|b|b.player.feet);
            before[index]+=Vec3::X*128.;
            stage.contacts(&mut roster,&world,before)?;
            ensure!(stage.owner==Some(first),"First temple exit contact did not own scene");
            let mut replica_world=World::from_bsp(map)?;
            let mut replica=Stage::load(a,map,&mut replica_world,&visit,false)?;
            for tick in 0..120*9 {
                if skip && tick==120 {
                    stage.interactions.skip_cinematic(map,&mut world,&mut roster.players[index].player,&mut stage.story)?;
                }
                stage.scene_step(&mut roster,map,&mut world,&Default::default())?;
                ensure!(roster.players[partner].player.feet==waiting,"Temple scene moved frozen partner");
                if tick%120==0 {
                    let bytes=crate::coop::codec::encode(&serde_json::to_vec(&stage.snapshot())?)?;
                    replica.restore(&serde_json::from_slice(&crate::coop::codec::decode(&bytes)?)?,map,&mut replica_world)?;
                    ensure!(serde_json::to_value(replica.snapshot())?==serde_json::to_value(stage.snapshot())?,"Temple scene replica differs");
                }
                if stage.transition.is_some() {break;}
            }
            ensure!(stage.transition.as_ref().is_some_and(|e|EXIT.matches(e)),"Temple scene failed garden transition");
            ensure!(roster.players.iter().all(|b|b.stats.alive()),"Temple ending drowned a player");
            println!("PASS temple {first:?} skip={skip}: one shared clock, shell grants, original exit contact/scene, frozen partner, replica and garden transition");
        }
    }
    Ok(())
}
fn simultaneous_and_rollback(a:&mut Assets,map:&Bsp)->Result<()> {
    for blocked in [None,Some(0),Some(1)] {
        let (mut t,mut world,mut pair)=begin(a,map)?;
        let (points,damage)=crush_positions(&mut t,map,&mut world)?;
        let (one,two)=points.iter().flat_map(|a|points.iter().map(move|b|(*a,*b)))
            .find(|(a,b)| (a.x-b.x).abs()>=48. && (a.y-b.y).abs()>=48. && (a.z-b.z).abs()<16.)
            .context("No two distinct closing-clam contacts")?;
        pair[0].0=Player::new(one);
        pair[1].0=Player::new(two);
        let before=pair.each_ref().map(|p|p.0.feet);
        let old_clams=t.saved.clams;
        let mut fixed=Vec::new();
        if let Some(index)=blocked {
            // A snug six-sided fixture leaves the starting body clear, but no
            // displacement of 2..64 units can escape the advancing native clam.
            let center=before[index]+PLAYER_CENTER;
            for axis in 0..3 {
                for sign in [-1.,1.] {
                    let mut lo=center-PLAYER_HALF-Vec3::ONE;
                    let mut hi=center+PLAYER_HALF+Vec3::ONE;
                    if sign>0. { lo[axis]=center[axis]+PLAYER_HALF[axis]+1.; hi[axis]=center[axis]+100.; }
                    else { hi[axis]=center[axis]-PLAYER_HALF[axis]-1.; lo[axis]=center[axis]-100.; }
                    fixed.push(Collider::box_bounds(lo,hi));
                }
            }
            // Diagonally separated contacts leave the other Alice outside this
            // cage, so her proposed escape must also roll back atomically.
        }
        let original=pair.each_ref().map(|p|p.0.feet);
        world.set_dynamic(fixed.iter().cloned().chain(t.colliders()).collect());
        ensure!(pair.iter().all(|p|world.body_clear(p.0.feet)),"Crush fixture starts obstructed");
        t.advance_roster(0.1,map,&mut world,&mut bodies(&mut pair),&fixed)?;
        ensure!(pair.iter().all(|p|world.body_clear(p.0.feet)),"Paired clam left embedded body");
        if let Some(index)=blocked {
            ensure!(pair.each_ref().map(|p|p.0.feet)==original && t.saved.clams==old_clams
                && pair.iter().all(|p|p.1.sanity()==100.-damage),
                "One blocked actor failed atomic machinery rollback");
            println!("PASS temple player{index}: blocked clam rolls shared motion back, personal damage and clear pair");
        } else {
            ensure!(pair.iter().all(|p|p.1.sanity()==100.-damage)
                && pair[0].0.feet.distance(pair[1].0.feet)>=36.,
                "Simultaneous crush shared a cooldown or overlapped players");
            println!("PASS temple simultaneous clam: both take own damage and remain separate");
        }
    }
    Ok(())
}
pub(super) fn check(assets: &mut Assets) -> Result<()> {
    if let Some(path) = std::env::var_os("LOOKING_GLASS_TEMPLE_REPLAY_REPORT") {
        return super::coop_route::replay_report(assets,std::path::Path::new(&path));
    }
    let map=Bsp::parse(&assets.read("maps/utemple.bsp")?)?;
    air(assets,&map)?;
    crush(assets,&map)?;
    simultaneous_and_rollback(assets,&map)?;
    stage_checks(assets,&map)?;
    Ok(())
}
