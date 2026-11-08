//! Native fan placement boundaries and the ordinary death/revival clock.
use super::*;
use crate::coop::{rules::{Life, RESPAWN_TICKS}, world::{camera_player, safe_near_with},
    protocol::Visit, stage::Stage, PlayerId};

pub(super) fn check(a: &mut Assets, map: &Bsp) -> Result<()> {
    let (t, mut w, r) = fixture(a, map)?;
    let visit = Visit {epoch: 1, map: "tower2".into(), entry: None,
        difficulty: map.difficulty, combat_preview: false};
    let stage = Stage::load(a, map, &mut w, &visit, false)?;
    for fan in 0..5 {
        let contact = fan_contact(&t, map, &w, fan)?;
        ensure!(w.body_clear(contact) && stage.interactions.hazardous(contact),
            "Clear gap in fan{fan} was accepted for future placement");
        let o = t.objects.iter().find(|o| o.name == format!("fan0{}", fan + 1)).unwrap();
        for degrees in (0..360).step_by(5) {
            let blade = Collider::model(map, o.model, o.base,
                Quat::from_axis_angle(super::super::motion::axis(fan), (degrees as f32).to_radians()), true)?;
            let point = blade.interior_point().context("Native fan lacks a solid brush")?;
            for offset in [Vec3::ZERO, PLAYER_HALF, -PLAYER_HALF] {
                ensure!(stage.interactions.hazardous(point + offset - PLAYER_CENTER),
                    "Fan{fan} full body escaped placement bound at {degrees} degrees");
            }
        }
        for slot in 0..2 {
            let mut frame = r.frame(1, 0);
            frame.players[slot].player = Player::new(contact);
            // Establish that ordinary current-pose geometry would accept it.
            Roster::from_checkpoint(&frame, &w, |_| true)?;
            ensure!(Roster::from_checkpoint(&frame, &w,
                |at| !stage.interactions.hazardous(at)).is_err(), "Unsafe fan checkpoint accepted");
        }
        let mut copy = restored(a, map, &t)?;
        copy.stopped = true;
        copy.saved.machine.as_mut().unwrap().fans = [137.; 5];
        copy.rebuild(map)?;
        ensure!(copy.hazardous(contact), "Watch/angle/restore removed fan placement exclusion");
        println!("PASS paired Tower fan{fan} placement: current clear gap rejected, full rotation/body bounds, both checkpoint slots and restored Watch");
    }

    // Find actual supplied-map water positions. The old check must offer an
    // unsafe body; the new check either chooses safe water or waits for a move.
    let mut reroute = None;
    let mut hold = None;
    'fans: for o in t.objects.iter().filter(|o| o.name.starts_with("fan")) {
        for x in (-192..=192).step_by(24) { for y in (-192..=192).step_by(24) { for z in (-192..=192).step_by(24) {
            let at = o.base + vec3(x as f32, y as f32, z as f32);
            if !w.body_clear(at) || crate::water::Immersion::sample(&w, at).level < 2 { continue; }
            let Some(old) = safe_near_with(&w, at, |_| true) else { continue; };
            if !t.hazardous(old.feet) { continue; }
            let safe = safe_near_with(&w, at, |feet| !t.hazardous(feet));
            if safe.is_some() && !t.hazardous(at) && reroute.is_none() { reroute = Some(at); }
            if safe.is_none() && hold.is_none() { hold = Some(at); }
            if reroute.is_some() && hold.is_some() { break 'fans; }
        }}}
    }
    let reroute = reroute.context("No native safe survivor with unsafe old revival choice")?;
    let hold = hold.context("No native fan gap needing delayed revival")?;
    for dead in PlayerId::ALL {
        revival(a, map, dead, reroute, false)?;
        revival(a, map, dead, hold, true)?;
    }
    Ok(())
}

fn revival(a: &mut Assets, map: &Bsp, dead: PlayerId, at: Vec3, hold: bool) -> Result<()> {
    let (mut t, mut w, mut r) = fixture(a, map)?;
    let slot = dead.index(); let survivor = dead.partner().index();
    r.players[survivor].player = Player::new(at);
    // A Watch may leave the survivor in a currently clear blade gap. Its
    // expiry must not put a freshly spawned partner into the next blade sweep.
    r.players[survivor].stats.watch().map_err(anyhow::Error::msg)?;
    r.players[slot].stats.damage(1000.);
    let mut expected = r.players[slot].stats.clone();
    expected.set_health(25.)?;
    let inventory = serde_json::to_value(expected)?;
    let finish = |r: &mut Roster, t: &Tower, w: &World| {
        r.finish_tick_with(w, false, |feet| !t.hazardous(feet))
    };
    finish(&mut r, &t, &w)?;
    ensure!(camera_player(&r.frame(1, 0), dead) == dead.partner(), "Dead Tower camera lost survivor");
    for tick in 1..=RESPAWN_TICKS {
        r.tick += 1;
        step(&mut t, &mut w, &mut r, map, FIXED_DT)?;
        finish(&mut r, &t, &w)?;
        if tick < RESPAWN_TICKS { ensure!(!r.players[slot].stats.alive(), "Early Tower revival"); }
        if tick == RESPAWN_TICKS / 2 {
            t = restored(a, map, &t)?;
            // Reconnection carries this frame without turning a death into a
            // checkpoint or resetting its remaining simulation ticks.
            let wire = crate::coop::codec::encode(&serde_json::to_vec(&r.frame(1, 0))?)?;
            let frame: crate::coop::world::Frame = serde_json::from_slice(&crate::coop::codec::decode(&wire)?)?;
            frame.validate()?; r.players = frame.players; r.lives = frame.lives;
        }
    }
    if hold {
        ensure!(r.lives.state(dead) == Life::AwaitingSafeSpawn && !r.players[slot].stats.alive(),
            "Tower revival did not wait beyond ten seconds in unsafe gap");
        for _ in 0..30 { r.tick += 1; finish(&mut r, &t, &w)?; }
        ensure!(!r.players[slot].stats.alive(), "Unsafe pending revival escaped hold");
        // Stage the survivor's subsequent position on the proven drain course;
        // this is a placement test, not a claim of traversing out of the gap.
        r.players[survivor].player = Player::new(vec3(2930., 400., -220.));
        r.tick += 1; finish(&mut r, &t, &w)?;
    }
    ensure!(r.players[slot].stats.sanity() == 25. && r.lives.state(dead) == Life::Alive
        && r.players[slot].spawned_at == r.tick && serde_json::to_value(&r.players[slot].stats)? == inventory,
        "Tower revival lost health, clock or personal inventory");
    t.coop_respawned(slot);
    let feet = r.players[slot].player.feet;
    ensure!(w.body_clear(feet) && !t.hazardous(feet)
        && feet.distance(r.players[survivor].player.feet) <= 100., "Tower revival is not safely beside survivor");
    r.players[survivor].stats.powers.stopped = 0.;
    // Let every blade pass through a full cycle while both staged bodies stay
    // at their chosen placements; no movement or invulnerability masks contact.
    for _ in 0..480 {
        step(&mut t, &mut w, &mut r, map, FIXED_DT)?;
        let feedback = t.damage_pair(FIXED_DT, &mut roster(&mut r));
        ensure!(feedback.players.iter().all(|p| p.damage == 0.), "Fan hit a safe revival after Watch expired");
    }
    println!("PASS paired Tower revival {dead:?} hold={hold}: spectator, 1200 ticks, compressed countdown, safe 25-health spawn and full live fan cycle");
    Ok(())
}
