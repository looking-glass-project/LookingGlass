//! Staged checks of the shared quest/combat adapter, not campaign traversal.
use super::*;
use anyhow::ensure;
use crate::{combat::DamageKind, movement::{EYE_HEIGHT, FIXED_DT}};

fn players(feet: [Vec3; 2]) -> [PlayerTarget; 2] {
    feet.map(|feet| PlayerTarget { eye: feet + Vec3::Z * EYE_HEIGHT,
        alive: true, hidden: false })
}

pub fn check(assets: &mut Assets, map: &Bsp) -> Result<()> {
    let initial = School2::load(assets, map)?;
    // Isolate each authored enemy type on an unobstructed floor. This makes
    // target selection/damage assertions independent of route navigation.
    let floor = World::fixture(&[(vec3(-2000.,-2000.,-20.), vec3(2000.,2000.,0.))]);
    check_diagnostics(&initial, &floor, [vec3(65.,0.,0.04), vec3(1200.,0.,0.04)], None)?;
    for victim in 0..2 {
        for kind in ["boojum", "rescue", "reinforcement"] {
            let mut school = initial.clone();
            school.placed = true;
            for b in &mut school.boojums { b.active = false; }
            let mut feet = [vec3(1200.,0.,0.04); 2];
            feet[victim] = vec3(65.,0.,0.04);
            let targets = players(feet);
            match kind {
                "boojum" => {
                    school.boojums[0] = Boojum::new(vec3(0.,0.,0.04), 0.);
                    school.boojums[0].active = true;
                }
                "rescue" => {
                    school.quest.enter(Stage::Rescue);
                    school.lab = vec3(-55.,0.,0.04);
                    school.guards[0] = Guard::new(vec3(0.,0.,0.04), 0., 1.);
                    school.guards[1] = Guard::new(vec3(-1500.,0.,0.04), 0., 1.);
                    let mut unprovoked = Feedback::default();
                    for _ in 0..360 {
                        unprovoked.append(school.update_players(FIXED_DT, &floor, targets, 0, false).1);
                    }
                    ensure!(unprovoked.damage == 0. && unprovoked.players.iter().all(|p|p.damage == 0.)
                        && school.guards.iter().all(|g|g.opponents.players.is_none()),
                        "Gnome attacks damaged a player before provocation");
                    school.guards[0] = Guard::new(vec3(0.,0.,0.04), 0., 1.);
                    school.hit_kind(ENEMY_BASE+100, 1., DamageKind::Cards);
                }
                _ => {
                    school.dice_guards = true;
                    for (i,g) in school.reinforcements.iter_mut().enumerate() {
                        *g = Guard::new(vec3(-1500.,i as f32*100.,0.04), 0., 1.);
                    }
                    school.reinforcements[0] = Guard::new(vec3(0.,0.,0.04), 0., 1.);
                }
            }
            let mut feedback = Feedback::default();
            for _ in 0..720 {
                feedback.append(school.update_players(FIXED_DT, &floor, targets, 0, false).1);
            }
            ensure!(feedback.players[victim].damage > 0. && feedback.players[1-victim].damage == 0.
                && feedback.damage == 0., "School {kind} misrouted damage for player {victim}: {:?}",
                feedback.players.each_ref().map(|p|p.damage));
            let report_id = ENEMY_BASE + match kind { "boojum" => 0, "rescue" => 100, _ => 200 };
            check_diagnostics(&school, &floor, feet, Some((report_id, victim)))?;
            let state = serde_json::from_slice(&serde_json::to_vec(&school.snapshot())?)?;
            let mut restored = school.clone();
            restored.restore(&state, map)?;
            for _ in 0..120 {
                let a = school.update_players(FIXED_DT, &floor, targets, 0, false).1;
                let b = restored.update_players(FIXED_DT, &floor, targets, 0, false).1;
                ensure!(a.players.each_ref().map(|p| (p.damage,p.impulse)) ==
                    b.players.each_ref().map(|p| (p.damage,p.impulse)),
                    "School {kind} damage diverged after save/restore");
            }
            ensure!(serde_json::to_value(school.snapshot())? == serde_json::to_value(restored.snapshot())?,
                "School {kind} state diverged after save/restore");
            // Dead/concealed players cannot remain the selected target.
            for dead in [false, true] {
                let mut hidden = targets;
                hidden[victim].alive = !dead;
                hidden[victim].hidden = !dead;
                school.update_players(FIXED_DT, &floor, hidden, 0, false);
                let selected = match kind {
                    "boojum" => school.boojums[0].opponents.selected_player,
                    "rescue" => school.guards[0].opponents.selected_player,
                    _ => school.reinforcements[0].opponents.selected_player,
                };
                ensure!(selected == Some(1-victim), "School {kind} retained dead/concealed target");
            }
            school.update(0., &floor, feet[0], false);
            ensure!(school.boojums.iter().all(|b|b.opponents.players.is_none())
                && school.guards.iter().chain(&school.reinforcements).all(|g|g.opponents.players.is_none()),
                "Single-player School2 retained the co-op roster");
            println!("PASS co-op School2 {kind}, victim {victim}: individual damage, alive/visible targeting, save continuation, SP reset");
        }
    }
    quest_contacts(assets, map, &initial)?;
    barrier_and_clock(map, &initial)?;
    Ok(())
}

fn check_diagnostics(school: &School2, world: &World, feet: [Vec3; 2],
    selected: Option<(usize, usize)>) -> Result<()> {
    use crate::{coop::{route_driver::school2_reports, world::Roster}, inventory::Stats};
    let before = serde_json::to_value(school.snapshot())?;
    let mut roster = Roster::new(world, feet[0] + Vec3::Z * EYE_HEIGHT, 0., Stats::for_level("skool2", None))?;
    for (body, feet) in roster.players.iter_mut().zip(feet) { body.player.feet = feet; }
    let mut frame = roster.frame(1, 0);
    let (guards, boojums) = school2_reports(school, &frame, world);
    let reports = serde_json::to_value(guards)?.as_array().unwrap().iter().cloned()
        .chain(serde_json::to_value(boojums)?.as_array().unwrap().iter().cloned()).collect::<Vec<_>>();
    ensure!(reports.len() == school.guards.len() + school.reinforcements.len() + school.boojums.len(),
        "School2 diagnostics omitted a legacy enemy");
    let active = reports.iter().filter(|r|r["active"] == true && r["health"].as_f64().unwrap() > 0.)
        .map(|r|r["id"].as_u64().unwrap() as usize).collect::<std::collections::BTreeSet<_>>();
    ensure!(active == school.targets().iter().map(|t|t.id).collect(),
        "School2 diagnostic identities/activation differ from combat targets");
    if let Some((id, victim)) = selected {
        let report = reports.iter().find(|r|r["id"] == id).context("Missing selected School2 enemy")?;
        ensure!(report["selected_player"] == victim && report["line_of_sight"][victim] == true
            && report["distance"][victim].as_f64() < report["distance"][1-victim].as_f64(),
            "School2 diagnostic did not identify the nearer visible victim");
    }
    for victim in 0..2 {
        frame.players[victim].stats.set_health(0.)?;
        let (guards, boojums) = school2_reports(school, &frame, world);
        for reports in [serde_json::to_value(guards)?, serde_json::to_value(boojums)?] {
            ensure!(reports.as_array().unwrap().iter().all(|r|r["line_of_sight"][victim] == false),
                "School2 diagnostics reported a dead player as visible");
        }
        frame.players[victim].stats.set_health(100.)?;
    }
    ensure!(serde_json::to_value(school.snapshot())? == before, "School2 diagnostics changed live state");
    Ok(())
}

fn barrier_and_clock(map: &Bsp, initial: &School2) -> Result<()> {
    let world = World::from_bsp(map)?;
    for occupant in 0..2 {
        let mut school = initial.clone();
        school.quest.enter(Stage::Battle);
        for b in &mut school.boojums { b.active = false; }
        let index = school.objects.iter().position(|o|o.name == "boojum_clip").unwrap();
        // This map's brush is enemy-only. Use its authored volume as a
        // blocking fixture to exercise insertion protection; production keeps
        // the original collision mask and does not wall Alice in with it.
        let Shape::Brush(model) = school.objects[index].shape else { unreachable!() };
        let pose = school.objects[index].pose;
        school.objects[index].collider = Collider::model(map, model, pose.origin, pose.rotation, false)?;
        let inside = school.objects[index].collider.interior_point().context("No barrier interior")? - PLAYER_CENTER;
        let outside = vec3(-64.,-3440.,-32.);
        let mut feet = [outside; 2];
        feet[occupant] = inside;
        let before = school.quest.time;
        school.update_players(FIXED_DT,&world,players(feet),0,false);
        ensure!(!school.objects[index].enabled, "School barrier closed over player {occupant}");
        ensure!((school.quest.time-before-FIXED_DT).abs() < 0.000001,
            "Two players advanced the school quest clock more than once");
        school.update_players(FIXED_DT,&world,players([outside; 2]),0,false);
        ensure!(school.objects[index].enabled, "School barrier did not close after both left");
    }
    println!("PASS co-op School2 barrier protects either occupant; quest clock advances once per world tick");
    Ok(())
}

fn quest_contacts(assets: &mut Assets, map: &Bsp, initial: &School2) -> Result<()> {
    use crate::coop::{protocol::Visit, stage::Stage as CoopStage, world::Roster};
    let visit = Visit { epoch: 1, map: "skool2".into(), entry: None,
        difficulty: Default::default(), combat_preview: false };
    for collector in 0..2 {
        let mut world = World::from_bsp(map)?;
        let mut stage = CoopStage::load(assets, map, &mut world, &visit, false)?;
        let (eye,yaw) = crate::interaction::spawn(map,None);
        let mut roster = Roster::new(&world,eye,yaw,crate::inventory::Stats::for_level("skool2",None))?;
        roster.players[0].stats.set_health(71.)?;
        roster.players[1].stats.set_health(43.)?;
        roster.players[1].stats.spend_will(23.);
        let personal = roster.players.each_ref().map(|b|(b.stats.sanity(),b.stats.will()));
        for (phase,name) in [(Stage::Jumbogrow,"jumbo_shelf1"), (Stage::Lollipop,"ig_lollypop"),
            (Stage::Rewards,"shrink_potion"), (Stage::Rewards,"lucky_star")] {
            let school = stage.interactions.school2.as_mut().unwrap();
            school.quest.enter(phase);
            school.cinema = None;
            for b in &mut school.boojums { b.active = false; }
            // Search actual unobstructed pickup reach, including its vertical
            // extent. This fixture does not claim a navigated approach.
            let item = initial.items[name];
            let mut contact = None;
            for z in [0.,-16.,-32.,16.] {
                for offset in [Vec3::X,-Vec3::X,Vec3::Y,-Vec3::Y,Vec3::ZERO] {
                    let feet = item + offset*32. + Vec3::Z*z;
                    let eye = feet + Vec3::Z*48.;
                    let trace = world.sweep(eye,item+Vec3::Z*16.,Vec3::splat(0.5));
                    if world.body_clear(feet) && !trace.start_solid && trace.fraction >= 1. {
                        contact = Some(feet); break;
                    }
                }
                if contact.is_some() { break; }
            }
            let feet = contact.with_context(||format!("No clear test contact for {name}"))?;
            roster.players[collector].player = Player::new(feet);
            roster.players[1-collector].player = Player::new(vec3(-64.,-3440.,-32.));
            roster.players[collector].stats.set_health(0.)?;
            stage.school2_step(FIXED_DT,&mut roster,&mut world,false);
            ensure!(stage.interactions.school2.as_ref().unwrap().quest.items ==
                roster.players[1-collector].stats.school_items, "Shared story inventory diverged");
            let before = stage.interactions.school2.as_ref().unwrap().quest.items.clone();
            roster.players[collector].stats.set_health(personal[collector].0)?;
            stage.school2_step(FIXED_DT,&mut roster,&mut world,false);
            let school = stage.interactions.school2.as_ref().unwrap();
            ensure!(school.quest.items != before, "Living player {collector} did not collect {name}");
            ensure!(roster.players.iter().all(|b|b.stats.school_items == school.quest.items),
                "Shared story reward missing from partner");
            ensure!(roster.players.each_ref().map(|b|(b.stats.sanity(),b.stats.will())) == personal,
                "Story collection altered individual health/Will");
            let before = school.quest.time;
            let age = school.age;
            stage.school2_step(FIXED_DT,&mut roster,&mut world,true);
            let school = stage.interactions.school2.as_ref().unwrap();
            ensure!(school.quest.time == before && school.age > age,
                "Paused dialogue advanced quest or froze actor presentation");
        }
        ensure!(stage.interactions.school2.as_ref().unwrap().quest.stage == Stage::Complete,
            "Collecting both shared rewards failed to open the return portal");
        println!("PASS integrated co-op School2 quest, collector {collector}: four real-map contacts, living-only collection, shared story rewards, personal resources and paused clock");
    }
    Ok(())
}
