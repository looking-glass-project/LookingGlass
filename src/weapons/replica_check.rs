//! Native pixel checks for host-produced weapon effects after network restore.
use super::*;
use crate::{character::Character, coop::PlayerId, movement::Player};

async fn picture(actor: &mut Character, camera: &Camera3D) -> Image {
    clear_background(Color::new(0.10,0.12,0.16,1.));
    set_camera(camera);
    let atmosphere=crate::environment::Atmosphere::default();
    crate::render_fx::begin_view(camera,0.,&atmosphere,true);
    actor.atmosphere(&atmosphere,camera.position);
    actor.draw_effects(camera.position,true,true);
    let counts=crate::render_fx::finish();
    assert_eq!(counts.1,0,"Replica effect draw dropped geometry");
    set_default_camera();
    let image=get_screen_data();
    next_frame().await;
    image
}

pub(super) async fn check(assets: &mut Assets) -> Result<()> {
    let world=World::fixture(&[
        (vec3(-500.,-500.,-20.),vec3(500.,500.,0.)),
        (vec3(180.,-300.,0.),vec3(200.,300.,300.)),
    ]);
    let context=CombatContext{world:&world,targets:&[]};
    let mut player=Player::new(Vec3::Z*0.04);player.grounded=true;
    let mut evidence=Vec::new();
    for id in PlayerId::ALL {
        let mut host=Character::load_for_player(assets,id)?;
        let mut guest=Character::load_for_player(assets,id)?;
        for (field,weapon,alternate) in [("trail",0,false),("flight_trails",1,false),("sparks",0,true)] {
            host.reset(&player,0.);
            let mut stats=crate::inventory::Stats::weapon_preview();stats.select(weapon);
            for _ in 0..120 {
                host.update_funded(1./120.,&player,false,WeaponInput{selected:weapon,aim:Vec3::X,
                    ..Default::default()},&context,Some(&mut stats));
            }
            let mut captured=None;
            for tick in 0..600 {
                host.update_funded(1./120.,&player,false,WeaponInput{selected:weapon,aim:Vec3::X,
                    click:(tick==0).then_some(alternate),..Default::default()},&context,Some(&mut stats));
                let value=serde_json::to_value(host.snapshot())?;
                if value["projectiles"][field].as_array().is_some_and(|v|v.len()>=8) {
                    captured=Some(value);break;
                }
            }
            let saved=captured.with_context(||format!("{id:?} attack produced no {field}"))?;
            let particles=saved["projectiles"][field].as_array().unwrap();
            let key=if field=="sparks"{"position"}else{"a"};
            let target:Vec3=serde_json::from_value(particles[particles.len()/2][key].clone())?;
            let camera=Camera3D{position:target+vec3(70.,-160.,80.),target,up:Vec3::Z,
                fovy:50_f32.to_radians(),z_near:0.5,z_far:5000.,..Default::default()};
            let original=picture(&mut host,&camera).await;
            for _ in 0..3 {
                guest.restore(&serde_json::from_value(saved.clone())?)?;
                let received=serde_json::to_value(guest.snapshot())?;
                ensure!(received["projectiles"][field]==saved["projectiles"][field],
                    "{id:?} replication lost {field}");
                let replica=picture(&mut guest,&camera).await;
                ensure!(original.bytes.len()==replica.bytes.len()
                    && original.bytes.iter().zip(&replica.bytes).all(|(a,b)|a.abs_diff(*b)<=1),
                    "{id:?} restored {field} changed rendered pixels");
            }
            // Negative control: emulate the old omitted field. Other props and
            // effects remain, so equality alone cannot pass with missing art.
            let mut missing=saved.clone();missing["projectiles"].as_object_mut().unwrap().remove(field);
            guest.restore(&serde_json::from_value(missing)?)?;
            let absent=picture(&mut guest,&camera).await;
            let visible=original.bytes.chunks_exact(4).zip(absent.bytes.chunks_exact(4))
                .filter(|(a,b)|a[..3].iter().zip(&b[..3]).any(|(x,y)|x.abs_diff(*y)>1)).count();
            ensure!(visible>8,"{id:?} {field} was not visibly rendered ({visible} pixels)");
            evidence.push(serde_json::json!({"player":id,"effect":field,"particles":particles.len(),
                "visible_pixels":visible,"repeated_restores":3,"pixel_match":true}));
            println!("PASS {id:?} {field}: {} live elements, {visible} visible pixels, three matching restores",
                particles.len());
        }
    }
    std::fs::create_dir_all(crate::preferences::path(""))?;
    std::fs::write(crate::preferences::path("coop-weapon-effects.json"),serde_json::to_vec_pretty(
        &serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),"passed":true,"cases":evidence,
            "scope":"Native funded attacks and serialized Character restore; independent clients remain separate"}))?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn legacy() -> ProjectileSave {
        serde_json::from_value(serde_json::json!({"projectiles":[],"shots":0,"impacts":0})).unwrap()
    }
    #[test]
    fn older_saves_have_no_stale_effects_and_new_saves_keep_world_positions() {
        let mut save=legacy();save.validate().unwrap();
        assert!(save.sparks.is_empty() && save.trail.is_empty() && save.flight_trails.is_empty());
        save.sparks.push(Spark{position:vec3(4.,5.,6.),velocity:Vec3::Z,life:0.3,color:WHITE});
        save.trail.push(Trail{a:Vec3::X,b:Vec3::Y,life:0.2,color:BLUE,electric:true});
        save.flight_trails=save.trail.clone();
        let serialized=serde_json::to_value(&save).unwrap();
        let roundtrip:ProjectileSave=serde_json::from_value(serialized.clone()).unwrap();
        roundtrip.validate().unwrap();
        assert_eq!(serde_json::to_value(roundtrip.migrated()).unwrap(),serialized);
    }
    #[test]
    fn corrupt_or_unbounded_effects_are_rejected() {
        let spark=Spark{position:Vec3::ZERO,velocity:Vec3::X,life:0.3,color:WHITE};
        for bad in [Spark{position:Vec3::NAN,..spark.clone()},Spark{life:f32::INFINITY,..spark.clone()},
            Spark{color:Color::new(-1.,0.,0.,1.),..spark.clone()}] {
            let mut save=legacy();save.sparks.push(bad);assert!(save.validate().is_err());
        }
        let mut save=legacy();save.sparks=vec![spark;257];assert!(save.validate().is_err());
        save.sparks.clear();
        let trail=Trail{a:Vec3::ZERO,b:Vec3::X,life:0.2,color:WHITE,electric:false};
        save.trail=vec![trail.clone();33];assert!(save.validate().is_err());
        save.trail.clear();save.flight_trails=vec![trail;257];assert!(save.validate().is_err());
    }
}
