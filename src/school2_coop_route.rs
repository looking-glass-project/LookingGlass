//! Normal paired main quest, physical gym/stairs and personal authored portals.
use crate::{assets::Assets, coop::native_route::{Command, CourseStep, Navigation, Spec}};
use anyhow::{ensure, Context};

pub(crate) fn check(a:&mut Assets)->crate::levels::BoxFuture<'_> {
    if let Some(path)=std::env::var_os("LOOKING_GLASS_SCHOOL2_REUNION_PROBE") {
        return Box::pin(async move { reunion_probe(a,path.into()) });
    }
    let mut c=vec![Command::Staff(false),Command::Cards(true),Command::Wait(120)];
    let nav=|c:&mut Vec<Command>,at:[f32;3]|c.push(if [-3010.,-2680.].contains(&at[0]) || at==[-2780.,-2560.,480.]
        || at==[1200.,-2880.,0.] || at==[2150.,-2080.,0.] {
        Command::Nav(at)
    } else {Command::Course(vec![CourseStep{at,mode:"walk".into(),ticks:0,clock:None}])});
    let stroll=|c:&mut Vec<Command>,points:&[[f32;3]]| {
        for &at in points {nav(c,at);c.push(Command::Clear(700.));}
    };
    let passage=|c:&mut Vec<Command>,points:&[[f32;3]]| {
        let up=points[0]==[2400.,-2150.,160.];let down=points[0]==[1880.,-2940.,440.];
        c.push(Command::CourseFile(points.iter().map(|at|CourseStep{at:*at,
            mode:if down || up {"nav"}else{"walk"}.into(),ticks:0,clock:None}).collect()));
    };
    let trail=|at,portal|Command::Trail{at,portal,direct:portal,escort:None,escort_player:None};
    stroll(&mut c,&[[-64.,-3100.,0.],[250.,-2800.,0.],[480.,-2560.,0.],[800.,-2560.,0.],
        [1150.,-2560.,0.],[1030.,-2560.,0.],[1030.,-2880.,0.],[1200.,-2880.,0.],
        [1470.,-2880.,0.],[1470.,-3040.,0.],[2150.,-3040.,0.],[2150.,-2000.,0.]]);
    c.push(Command::Positions([Some([2163.,-1859.,64.]),None]));
    c.push(Command::Interact([2244.,-1859.,90.]));c.push(Command::Wait(840));
    nav(&mut c,[2150.,-2080.,0.]);
    passage(&mut c,&[[2400.,-2150.,160.],[2440.,-2150.,192.],[2270.,-2040.,250.],
        [1950.,-2016.,400.],[1710.,-2000.,448.],[1600.,-2275.,440.],[1680.,-2350.,440.],
        [1472.,-2560.,440.],[1710.,-2800.,440.],[1880.,-2940.,440.],[2077.,-3090.,440.]]);
    c.push(Command::Clear(1400.));c.push(Command::Wait(1200));
    passage(&mut c,&[[1880.,-2940.,440.],[1710.,-2800.,440.],[1472.,-2560.,440.],
        [1680.,-2350.,440.],[1600.,-2275.,440.],[1710.,-2000.,448.],[1950.,-2016.,400.],
        [2270.,-2040.,250.],[2440.,-2150.,192.],[2400.,-2150.,160.],[2150.,-2080.,0.]]);
    stroll(&mut c,&[[2150.,-3040.,0.],[1470.,-3040.,0.],[1470.,-2880.,0.],[1440.,-2880.,0.],
        [1040.,-2880.,0.],[1040.,-2560.,0.]]);
    c.push(Command::Nav([480.,-2560.,0.]));c.push(Command::Clear(700.));
    stroll(&mut c,&[[250.,-2290.,0.],[-64.,-2060.,0.]]);
    passage(&mut c,&[[-64.,-2200.,0.],[-416.,-2200.,0.],[-416.,-2480.,0.],[-272.,-2448.,48.],
        [-192.,-2400.,80.],[-128.,-2336.,112.],[-96.,-2272.,128.],[-32.,-2272.,136.],
        [64.,-2368.,184.],[192.,-2464.,240.],[256.,-2528.,256.],[416.,-2528.,256.],
        [432.,-2400.,256.],[288.,-2208.,256.],[160.,-2112.,256.]]);
    stroll(&mut c,&[[32.,-1984.,256.],[-64.,-1744.,256.],[-64.,-1640.,256.],[-64.,-1270.,256.]]);
    c.push(Command::Wait(960));c.push(Command::Clear(700.));
    nav(&mut c,[-64.,-1020.,256.]);c.push(Command::Clear(700.));c.push(Command::Wait(2160));
    stroll(&mut c,&[[40.,-930.,256.],[150.,-600.,256.],[36.,-452.,256.]]);
    c.push(Command::Wait(120));
    for at in [[150.,-600.,256.],[40.,-930.,256.],[-64.,-1270.,256.],[-64.,-1504.,256.]] {nav(&mut c,at);}
    c.push(trail([-312.,-1504.,256.],true));c.push(trail([-1120.,-2540.,512.],false));
    for at in [[-1470.,-2560.,512.],[-1740.,-2560.,512.],[-2070.,-2560.,512.],
        [-2320.,-2560.,512.],[-2640.,-2560.,512.],[-2780.,-2560.,512.],[-3010.,-2550.,488.]] {nav(&mut c,at);}
    c.push(Command::Wait(840));nav(&mut c,[-3010.,-2550.,504.]);c.push(Command::Wait(120));
    for at in [[-2780.,-2560.,480.],[-2680.,-2560.,512.],[-2320.,-2560.,512.],
        [-2070.,-2560.,512.],[-1740.,-2560.,512.],[-1470.,-2560.,512.],[-1120.,-2540.,512.]] {nav(&mut c,at);}
    c.push(trail([-1112.,-2280.,512.],true));c.push(trail([-64.,-1504.,256.],false));
    nav(&mut c,[-64.,-1020.,256.]);nav(&mut c,[0.,-820.,256.]);c.push(Command::Wait(1080));
    nav(&mut c,[-28.,-695.,256.]);nav(&mut c,[0.,-760.,256.]);c.push(Command::Wait(120));
    nav(&mut c,[-64.,-1270.,256.]);c.push(Command::Wait(1200));
    crate::coop::native_route::check(a,Spec{map:"skool2",entry:"skool2_start1",
        exit:crate::level::spec::ExitSpec{map:"skool1",entrance:"skool1_start2"},
        navigation:Navigation::LandedPair,commands:c,verify:|arena,players| {
            let s=arena.stage.interactions.school2.as_ref().context("Missing School Two quest")?;
            ensure!(s.quest.stage==crate::school2_quest::Stage::Complete,"Incomplete School Two quest");
            ensure!(players.iter().all(|b|b.stats.school_items.potion && b.stats.school_items.star),
                "Missing personal quest rewards");Ok(())
        }})
}

/// Read-only geometry diagnostic for a captured failed route. Never supplies
/// a successful traversal result or changes the saved players/world.
fn reunion_probe(a:&mut Assets,path:std::path::PathBuf)->anyhow::Result<()> {
    use crate::{bsp::Bsp,collision::World,coop::{arena::Arena,protocol::Visit},movement::Player};
    let r:serde_json::Value=serde_json::from_slice(&std::fs::read(path)?)?;
    let visit:Visit=serde_json::from_value(r["visit"].clone())?;
    ensure!(visit.map=="skool2","Wrong reunion probe map");
    let map=Bsp::parse(&a.read("maps/skool2.bsp")?)?;
    let mut world=World::from_bsp(&map)?;
    let mut arena=Arena::load(a,&map,&mut world,&visit,false)?;
    arena.replicate(&serde_json::from_value(r["world"].clone())?,&map,&mut world)?;
    let owner=arena.stage.owner.context("Probe has no scene owner")?;
    let p:Player=serde_json::from_value(r["frame"]["players"][owner.index()]["player"].clone())?;
    let anchor=arena.stage.interactions.school2.as_ref().and_then(|s|s.coop_reunion_anchor(&p));
    let pair=anchor.and_then(|at|crate::coop::placement::scene_pair(&world,at,
        |at|!arena.stage.interactions.hazardous(at)));
    let landings=anchor.map(|at|crate::coop::placement::scene_landings(&world,at,
        |at|!arena.stage.interactions.hazardous(at)).into_iter().map(|p|p.feet).collect::<Vec<_>>());
    let report=serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"owner":owner,"feet":p.feet,
        "anchor":anchor,"pair":pair.map(|(a,b)|[a.feet,b.feet]),"landings":landings});
    std::fs::write(crate::preferences::path("skool2-reunion-probe.json"),serde_json::to_vec_pretty(&report)?)?;
    println!("School Two read-only reunion probe: {report}");Ok(())
}
