//! Normal paired Castle Keep course; all puzzle changes come from player inputs.
use super::*;
use crate::coop::native_route::{Command as C, Navigation, Spec};

fn walk(commands: &mut Vec<C>, points: &[[f32;3]]) {
    commands.extend(points.iter().copied().map(C::Approach));
}
fn lobby(commands: &mut Vec<C>) {
    walk(commands,&[
        [456.,-544.,512.],[376.,-544.,144.],[80.,-700.,144.],
        [48.,-640.,144.],[48.,-480.,144.],[96.,-370.,144.],
        [340.,-240.,144.],[512.,0.,144.],[512.,500.,144.],
    ]);
    // Collect the personally earned guard essence before the next ascent.
    commands.push(C::Clear(1600.));
}
fn lever(commands: &mut Vec<C>) {
    for point in [[512.,1100.,128.],[512.,1300.,128.],[512.,1600.,128.],
        [0.,1750.,256.],[0.,1930.,256.],[-180.,2470.,256.],
        [32.,3000.,256.],[512.,3208.,272.]] {
        commands.push(C::Approach(point));
        if point==[512.,1300.,128.] {
            commands.extend([C::Wait(132),C::Watch,C::Staff(false)]);
        }
        if point==[512.,1600.,128.] || point==[512.,3208.,272.] {
            commands.extend([C::Clear(350.),C::Staff(true),C::Clear(900.)]);
        }
    }
}
pub(super) fn check(a: &mut Assets) -> super::super::BoxFuture<'_> {
    Box::pin(async move {
        let map=Bsp::parse(&a.read("maps/keep.bsp")?)?;
        let marker=map.entities.iter().find(|e|e.get("targetname").is_some_and(|n|n=="mirror_lever"))
            .context("Missing normal mirror lever")?;
        let at=data::at(marker).point(vec3(36.,0.,26.)).to_array();
        let mut commands=vec![C::Wait(600)];
        for suit in 0..3 {
            if suit>0 {lobby(&mut commands);}
            lever(&mut commands);
            commands.extend([C::Approach([512.,3138.,272.]),C::Walk([512.,3138.,280.]),C::Interact(at)]);
            walk(&mut commands,&[
                [32.,3000.,256.],[-180.,2470.,256.],[0.,1930.,256.],[0.,1750.,256.],
                [512.,1600.,128.],[512.,1450.,128.],[512.,1100.,128.],[512.,500.,128.],
                [376.,-544.,144.],[376.,-384.,144.],
            ]);
            commands.extend([C::Clear(900.),C::Walk([374.,-376.,128.]),
                C::Rise{center:[512.,-384.],height:548.,landing:[456.,-544.,512.]}]);
            let path: &[[f32;3]]=match suit {
                0=>&[[512.,-850.,512.],[512.,-1100.,512.],[512.,-2210.,512.]],
                1=>&[[-530.,256.,512.],[-730.,256.,512.],[-950.,256.,512.],[-2060.,256.,512.]],
                _=>&[[1560.,256.,512.],[1752.,256.,512.],[1950.,256.,512.],[3100.,256.,512.]],
            };
            walk(&mut commands,path);
            commands.extend([C::Wait(132),C::Clear(700.),C::Shoot(match suit {
                0=>"Tweedle_Club_Win",1=>"Jabber_Diamond_Win",_=>"Hatter_Spade_Win",
            }.into()),C::Wait(264),C::Approach(path[path.len()-2]),C::Approach(match suit {
                0=>[512.,-1000.,512.],1=>[-872.,256.,512.],_=>[1896.,256.,512.],
            }),C::Wait(36),C::Approach(path[0])]);
        }
        lobby(&mut commands);lever(&mut commands);
        commands.push(C::Approach([512.,3528.,272.]));
        // The authored gate closes behind the first arrival. Let that player
        // reach the original scene, which reunites the waiting partner.
        commands.extend([C::Positions([Some([512.,4100.,256.]),None]),
            C::Positions([Some([512.,4600.,256.]),None])]);
        commands.push(C::Wait(1200));
        crate::coop::native_route::check(a,Spec{map:"keep",entry:"keep_start1",exit:EXIT,commands,
            navigation:Navigation::LandedPair,verify:|arena,_| {
                let k=arena.stage.interactions.levels.iter().find_map(|s|s.ctl.downcast_ref::<Keep>())
                    .context("Missing normal Keep controller")?;
                ensure!(k.saved.won==[true;3] && k.saved.heart_open && k.saved.death_done && k.saved.exit.committed,
                    "Keep did not finish its three portraits and original final scene");
                Ok(())
            }}).await
    })
}
