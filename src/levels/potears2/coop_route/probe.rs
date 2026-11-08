//! Read-only geometry inspection of a failed normal route; never route evidence.
use super::*;
pub(super) fn run(a:&mut Assets,path:&std::ffi::OsStr)->Result<()> {
    let report:serde_json::Value=serde_json::from_slice(&std::fs::read(path)?)?;
    let visit=serde_json::from_value(report["visit"].clone())?;
    let map=Bsp::parse(&a.read("maps/potears2.bsp")?)?;
    let mut world=World::from_bsp(&map)?;
    let mut arena=Arena::load(a,&map,&mut world,&visit,false)?;
    arena.replicate(&serde_json::from_value(report["world"].clone())?,&map,&mut world)?;
    let bodies:[Body;2]=serde_json::from_value(report["frame"]["players"].clone())?;
    let pond=owner(&arena)?;
    let index=if pond.saved.pond.leaves[1].is_some() {6}else{5};
    let pad=&pond.pond.pads[index];let clock=pond.saved.pond.clocks[index];
    let mut without=world.clone();
    without.set_dynamic(pond.colliders.iter().cloned().chain(pond.pond.pads.iter().enumerate()
        .filter(|(i,_)|*i!=index).map(|(_,p)|p.collider.clone())).collect());
    let next_at=pond.pond.future(index,clock+f64::from(crate::movement::FIXED_DT));
    let next=Collider::model(&map,pad.model,next_at,pad.pose.rotation,true)?;
    let mut rows=Vec::new();
    for (i,b) in bodies.iter().enumerate() {
        let p=&b.player;let carried=p.feet+next_at-pad.pose.translation;
        let feet=next.rider_feet(carried).map(|v|v.0);
        let tr=feet.map(|f|without.body_trace(p.feet,f));
        println!("RIDER {i} at {:?} local {:?} aboard={} supported={} next={feet:?} trace={tr:?}",p.feet,
            p.feet-pad.pose.translation,aboard(pad,p),crate::moving::supported_by(&without,&pad.collider,p.feet,false));
        for x in [-64.,-32.,0.,32.,64.] {for y in [-40.,-20.,0.,20.,40.] {
            let Some(at)=deck(&world,pad,vec2(x,y)) else {continue;};
            let tr=world.body_trace(p.feet,at);let local=at-pad.pose.translation;
            let safe=(0..=8).take_while(|n|world.body_clear(pond.pond.future(index,clock+*n as f64*0.2)+local)).count();
            let mut previous=at;
            let swept=(0..=8).take_while(|n| {
                let at=pond.pond.future(index,clock+*n as f64*0.2)+local;
                let t=world.body_trace(previous,at);previous=at;
                !t.start_solid && t.fraction>=1. && world.body_clear(at)
            }).count();
            rows.push(serde_json::json!({"player":i,"offset":[x,y],"at":at,"path":tr.fraction,
                "start_solid":tr.start_solid,"safe":safe,"swept":swept,
                "partner_distance":(at-bodies[1-i].player.feet).truncate().length()}));
        }}
    }
    println!("POND PROBE {}",serde_json::json!({"clock":clock,"center":pad.pose.translation,"next":next_at,"candidates":rows}));
    Ok(())
}
