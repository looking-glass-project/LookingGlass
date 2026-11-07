//! Component checks using the original Queen rigs, arenas, story and checkpoint writer.
//! A graphics context is required to construct the saved character state; no input or captures.
use super::*;
use crate::{combat::{self, DamageKind}, coop::{protocol::Visit, stage::Stage, world::Roster, PlayerId},
    level::CoopPlayer, movement::FIXED_DT};

fn pair<'a>(p: &'a mut [Player; 2], s: &'a mut [Stats; 2]) -> [CoopPlayer<'a>; 2] {
    let [a,b] = p; let [x,y] = s;
    [CoopPlayer {player:a,stats:x}, CoopPlayer {player:b,stats:y}]
}
fn owner(s: &mut Stage) -> &mut Queen {
    s.interactions.levels.iter_mut().find_map(|l| l.ctl.downcast_mut()).unwrap()
}
fn scene(s: &mut Stage, r: &mut Roster, map: &Bsp, w: &mut World, ending: bool) -> Result<()> {
    for tick in 0..36000 {
        s.scene_step(r, map, w, &Default::default())?;
        if tick == 240 { let saved = s.snapshot(); s.restore(&saved, map, w)?; }
        if ending && owner(s).ending_ready() || !ending && !s.busy() { return Ok(()); }
    }
    anyhow::bail!("Paired Queen scene stalled at {:?}", owner(s).saved.phase)
}
fn checkpoint(a: &mut Assets, map: &Bsp, w: &mut World, visit: &Visit,
    s: Stage, mut r: Roster) -> Result<(Stage,Roster)> {
    let mut arena = crate::coop::arena::Arena::load(a,map,w,visit,false)?;
    arena.stage=s;
    arena.stage.interactions.sync_coop(w);
    arena.reset_characters(&r.players);
    let store=crate::coop::checkpoint::Store::current(a.fingerprint()?);
    store.save(visit,&mut r,&mut arena,w,&Default::default())?;
    ensure!(!owner(&mut arena.stage).checkpoint_requested(),"Successful save did not acknowledge Queen request");
    let saved=store.load()?.checkpoint;
    // The live world must equal the file even when there was no level request.
    crate::coop::checkpoint::verify_restored(&saved,&r,&arena)?;
    arena.restore_checkpoint(saved.frame.combat.as_ref().unwrap(),map,w)?;
    r=Roster::from_checkpoint(&saved.frame,w,|at|!arena.stage.interactions.hazardous(at))?;
    crate::coop::checkpoint::verify_restored(&saved,&r,&arena)?;
    Ok((arena.stage,r))
}
fn progression(a: &mut Assets, map: &Bsp) -> Result<()> {
    let mut w = World::from_bsp(map)?;
    let visit = Visit {epoch:1,map:"qlair".into(),entry:None,difficulty:map.difficulty,combat_preview:false};
    let mut s = Stage::load(a, map, &mut w, &visit, false)?;
    let (eye,yaw) = crate::interaction::spawn(map, None);
    let mut r = Roster::new_with(&w, eye, yaw, Stats::for_level("qlair",None), |_| true)?;
    s.before_move(&mut r,map,&mut w,&Default::default(),FIXED_DT)?;
    (s,r)=checkpoint(a,map,&mut w,&visit,s,r)?;
    ensure!(!owner(&mut s).saved.checkpoint_saved,"Ordinary save acknowledged an unrequested Queen checkpoint");
    println!("PASS paired Queen: initial-area autosave and exact disk restore without a level checkpoint request");
    let e = owner(&mut s).event(INTRO).unwrap();
    s.accept(e,PlayerId::Red,&mut r,&w);
    ensure!(s.owner == Some(PlayerId::Red), "Queen intro lost Red owner");
    scene(&mut s,&mut r,map,&mut w,false)?;
    ensure!(owner(&mut s).saved.phase == Phase::Queen1 && r.players.iter().all(|p| w.body_clear(p.player.feet)),
        "Queen intro failed paired reunion");
    println!("PASS paired Queen: Red-owned original introduction, scene restore and safe reunion");
    for (id,damage,phase) in [(BASE,2500.,Phase::Birth),(BASE+1,3501.,Phase::Death)] {
        owner(&mut s).hit(Hit {id,damage,kind:DamageKind::Knife,knockback:Vec3::ZERO});
        let mut players = r.players.each_mut().map(|b| CoopPlayer {player:&mut b.player,stats:&mut b.stats});
        owner(&mut s).coop_combat_roster(FIXED_DT,&w,&mut players);
        ensure!(owner(&mut s).saved.phase == phase,"Paired Queen death transition absent");
        s.before_move(&mut r,map,&mut w,&Default::default(),FIXED_DT)?;
        scene(&mut s,&mut r,map,&mut w,phase == Phase::Death)?;
        if phase == Phase::Birth {
            ensure!(owner(&mut s).saved.phase == Phase::Queen2 && owner(&mut s).checkpoint_requested()
                && r.players.iter().all(|p| p.stats.sanity() == 100. && w.body_clear(p.player.feet)),
                "Second arena/refill/checkpoint handoff failed");
            (s,r)=checkpoint(a,map,&mut w,&visit,s,r)?;
            ensure!(!owner(&mut s).checkpoint_requested(),"Queen checkpoint repeated after restore");
            println!("PASS paired Queen: original birth, personal refill, second-arena reunion and restored checkpoint request");
        }
    }
    ensure!(owner(&mut s).ending_ready() && s.transition.is_none(),"Finale did not request ending");
    let saved = s.snapshot(); s.restore(&saved,map,&mut w)?;
    ensure!(owner(&mut s).ending_ready(),"Completed finale lost across replication");
    println!("PASS paired Queen: original death/ending clock and durable completion for both clients");
    Ok(())
}
fn combat_resources(a: &mut Assets,map: &Bsp) -> Result<()> {
    let w = World::fixture(&[]);
    let mut q = Queen::load(a,map)?;
    let mut p = [Player::new(Vec3::ZERO),Player::new(Vec3::X*400.)];
    let mut s = [Stats::for_level("qlair",None),Stats::for_level("qlair",None)];
    q.coop_prepare(&mut pair(&mut p,&mut s));
    q.start(Phase::Queen1);
    for slot in 0..2 {
        s[slot].set_health(40.)?;
        p[slot].feet = q.personal_essence(Some(slot)).unwrap().translation;
        q.coop_prepare(&mut pair(&mut p,&mut s));
        ensure!(s[slot].sanity() == 65.,"Personal Queen essence absent");
    }
    ensure!(q.saved.coop.as_ref().unwrap().wait == [10.;2],"One Alice consumed partner's essence");
    let saved = q.snapshot(); q.restore(&saved,map)?;
    q.coop_prepare(&mut pair(&mut p,&mut s));
    ensure!(s.iter().all(|s| s.sanity()==65.),"Queen restore refilled resources");
    println!("PASS paired Queen: independent healing cycles, cooldowns and no rejoin refill");
    for victim in 0..2 {
        q.start(Phase::Queen1);
        let from = q.queen1_pose().translation;
        p[victim].feet = from + Vec3::X*400. - Vec3::Z*32.;
        p[1-victim].feet = from + Vec3::Y*1800. - Vec3::Z*32.;
        q.coop_combat_roster(FIXED_DT,&w,&mut pair(&mut p,&mut s));
        ensure!(q.saved.coop.as_ref().unwrap().target == Some(victim),"Queen did not select nearest living player");
        // The original telekinesis capture window restrains only its selected body.
        q.saved.attack = battle::Attack::Grab;
        q.saved.attack_time = q.tele_prep() + 1.666 - FIXED_DT*0.5;
        q.saved.attack_target = p[victim].feet + Vec3::Z*32.;
        q.coop_combat_roster(FIXED_DT,&w,&mut pair(&mut p,&mut s));
        ensure!(q.coop_controlled(victim) && !q.coop_controlled(1-victim)
            && !q.controlled() && q.camera(&w).is_none(),"Queen grab stole partner's controls/camera");
        q.saved.grab_arrived = true; q.saved.clock = 1.;
        let f = q.coop_combat_roster(FIXED_DT,&w,&mut pair(&mut p,&mut s));
        ensure!(f.players[victim].damage > 0. && f.players[1-victim].damage == 0. && f.damage == 0.,
            "Queen squeeze hit wrong player");
        let saved = q.snapshot(); q.restore(&saved,map)?;
        ensure!(q.coop_controlled(victim),"Queen grab victim lost on restore");
        s[victim].damage(10000.); q.coop_prepare(&mut pair(&mut p,&mut s));
        q.coop_combat_roster(FIXED_DT,&w,&mut pair(&mut p,&mut s));
        ensure!(!q.saved.grabbed && q.saved.coop.as_ref().unwrap().target == Some(1-victim),"Queen retained dead victim");
        s[victim].set_health(65.)?;
        println!("PASS paired Queen victim{victim}: nearest target, personal grab/pulse, restored capture and survivor retarget");
    }
    // An aimed missile can strike the other player's body; splash also sees both.
    q.start(Phase::Queen1);
    let model = q.data.projectiles.iter().find(|(_,s)| s.splash>0. && s.radius>100.)
        .map(|(n,_)|n.clone()).context("Queen splash projectile absent")?;
    for victim in 0..2 {
        let spec = &q.data.projectiles[&model];
        p[victim].feet = Vec3::ZERO; p[1-victim].feet = Vec3::Y*60.;
        q.saved.projectiles = vec![battle::Shot {model:model.clone(),at:vec3(-40.,0.,32.),
            velocity:Vec3::X*500.,age:0.,damage:spec.damage,force:spec.force,life:spec.life,
            end:None,seek_at:1.,victim:Some([combat::COOP_BLUE,combat::COOP_RED][1-victim]),trail:Vec::new()}];
        let mut f = Feedback::default();
        let bodies = combat::Opponents {players:Some(p.each_ref().map(|p|combat::PlayerTarget {eye:p.eye(),alive:true,hidden:false})),..Default::default()}.bodies(p[0].eye());
        q.projectiles(0.1,&bodies,&w,&mut f);
        ensure!(f.players[victim].damage>0. && f.players[1-victim].damage>0. && f.damage==0.,"Queen projectile/splash lost a body");
    }
    q.saved.phase = Phase::Queen2; q.saved.queen1=0.;q.saved.powered=true;q.saved.checkpoint=true;
    q.saved.time=6.;q.saved.motion=0;q.saved.position=4;
    for i in 0..2 {q.coop_trigger_actor(Some(i));q.event("MoveQueen4");q.event("MoveQueen1");}
    q.coop_trigger_actor(None);
    ensure!(q.saved.coop.as_ref().unwrap().platform==[1;2],"Paired platform contacts toggled shared state twice");
    let saved=q.snapshot();q.restore(&saved,map)?;
    println!("PASS paired Queen: either projectile victim, two-body splash and independent restored platform contacts");
    for phase in [Phase::Queen1,Phase::Queen2] { for victim in 0..2 {
        q.start(phase);q.saved.time=6.;q.saved.queen1=if phase==Phase::Queen1 {2500.}else{0.};
        q.saved.powered=phase==Phase::Queen2;q.saved.refilled=q.saved.powered;
        q.saved.checkpoint=q.saved.powered;q.saved.motion=0;q.saved.position=4;q.saved.platform=4;
        q.saved.body=q.data.points["point4"];q.saved.body.translation.z=40.;
        q.saved.coop.as_mut().unwrap().platform=[4;2];q.saved.coop.as_mut().unwrap().target=None;
        let from=if phase==Phase::Queen1 {q.queen1_pose().translation}else{q.saved.body.translation+Vec3::Z*1000.};
        p[victim].feet=from+Vec3::X*400.-Vec3::Z*32.;p[1-victim].feet=from+Vec3::Y*2000.-Vec3::Z*32.;
        q.coop_combat_roster(FIXED_DT,&w,&mut pair(&mut p,&mut s));
        q.saved.attack=if phase==Phase::Queen1 {battle::Attack::Ice}else{battle::Attack::Spore};
        q.saved.attack_time=0.;let mut fired=false;
        for _ in 0..1800 {
            q.coop_combat_roster(FIXED_DT,&w,&mut pair(&mut p,&mut s));
            if q.saved.projectiles.iter().any(|p|p.victim==Some([combat::COOP_BLUE,combat::COOP_RED][victim])) {
                fired=true;break;
            }
        }
        ensure!(fired,"Native {phase:?} animation did not fire at victim{victim}");
        println!("PASS paired {phase:?} victim{victim}: native animation events aim projectiles at selected player");
    }}
    Ok(())
}
pub(super) fn windowed(a: &mut Assets) -> super::super::BoxFuture<'_> {
    Box::pin(async move { check(a) })
}
fn check(a:&mut Assets)->Result<()> {
    let map=Bsp::parse(&a.read("maps/qlair.bsp")?)?;
    let mut failed=Vec::new();
    for (name,run) in [("combat/resources",combat_resources as fn(&mut Assets,&Bsp)->Result<()>),("progression",progression)] {
        if let Err(e)=run(a,&map) {println!("FAIL paired Queen {name}: {e:#}");failed.push(name);}
    }
    ensure!(failed.is_empty(),"Paired Queen failures: {failed:?}");Ok(())
}
