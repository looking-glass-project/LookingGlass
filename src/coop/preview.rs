//! Windowed development entry point for shared movement and combat. This
//! does not claim full campaign/controller support. Co-op checkpoints are kept
//! separately from all single-player saves.
use super::{
    hosting::{Hosting, Status},
    invite::Invite,
    protocol::{Identity, Input, RemoteInput, Visit},
    session::{Guest, Host, Phase},
    transport::{self, Conditions},
    world::{self, Frame, Roster},
    PlayerId,
};
use crate::{assets::Assets, inventory::Stats, movement::FIXED_DT, Options};
use anyhow::{Context, Result};
use macroquad::prelude::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct Launch {
    pub host: bool,
    pub internet: bool,
    pub invite_file: Option<PathBuf>,
    pub invite: Option<Invite>,
    pub start: Option<Visit>,
    pub drive: bool,
    pub death_check: Option<PlayerId>,
    pub scene_check: Option<PlayerId>,
    pub travel_check: bool,
    pub resume: bool,
    pub single_player: Option<crate::save::Game>,
    pub checkpoint_check: bool,
    pub route_driver: Option<PathBuf>,
    pub conditions: Conditions,
}

fn follow_body_visible(watched: bool, show_watched: bool, camera: Vec3, feet: Vec3) -> bool {
    // Only hide the camera's own subject when the follow camera is pressed
    // against it. A nearby partner is still a separate, visible player.
    !watched || (show_watched && camera.distance(feet + Vec3::Z * 32.) > 48.)
}

#[cfg(test)]
mod tests {
    use super::follow_body_visible;
    use macroquad::prelude::*;

    #[test]
    fn close_partner_survives_follow_camera_culling_after_scene_reunion() {
        // Captured Checkmate reunion: the players are 48 units apart, but Red
        // is only 46.86 units from Blue's camera and previously disappeared.
        let camera = vec3(516.03125, 2723.7375, 298.63898);
        let blue = vec3(608., 2736., 256.03125);
        let red = vec3(560., 2736., 256.03125);
        assert!(follow_body_visible(true, true, camera, blue));
        assert!(follow_body_visible(false, true, camera, red));
        // A genuinely cramped own camera still hides its subject, without
        // applying that decision to the other player (including spectating).
        assert!(!follow_body_visible(true, true, camera, red));
        assert!(!follow_body_visible(true, false, camera, blue));
        assert!(follow_body_visible(false, false, camera, red));
    }
}

pub async fn run(assets: &mut Assets, options: &Options, launch: &Launch) -> Result<bool> {
    let ui = crate::ui::Ui::load(assets)?;
    show_mouse(false);
    let _ = std::fs::remove_file(crate::preferences::path("coop-live.json"));
    let mut hosting = None;
    let mut completed = false;
    let mut audio = super::audio::Playback::new(options);
    let result = async {
        let prefs = crate::preferences::Preferences::load();
        let mut setup_input = crate::input::Input::default();
        let mut setup_selected = 0;
        let identity = Identity::current(assets.fingerprint()?);
        anyhow::ensure!(!launch.resume || launch.host, "Only the host loads the shared co-op checkpoint");
        let checkpoint_store = super::checkpoint::Store::current(identity.game_data.clone());
        let from_slot = options.load.filter(|_| launch.invite_file.is_some());
        let importing = launch.single_player.is_some() || from_slot.is_some();
        anyhow::ensure!(!importing || launch.host && !launch.resume && !launch.travel_check
            && launch.death_check.is_none() && launch.scene_check.is_none() && !options.combat_preview,
            "Host an existing single-player game separately from Continue co-op or staged checks");
        let imported = if let Some(game) = &launch.single_player {
            Some(super::solo::convert(assets, game.clone())?)
        } else if let Some(slot) = from_slot {
            let store = crate::save::Store::for_assets(options.save_dir.clone(), assets)?;
            Some(super::solo::convert(assets, store.read(slot)?.game)?)
        } else { None };
        let resumed = if let Some(checkpoint) = imported {
            Some(super::checkpoint::Loaded { checkpoint, recovered_backup: false })
        } else if launch.resume { Some(checkpoint_store.load()?) } else { None };
        let (mut host, mut guest, invite) = if launch.host {
            hosting = Some(Hosting::start(crate::preferences::path("coop"),
                PathBuf::from("private/coop/helpers"), launch.internet, launch.conditions)?);
            let setup = hosting.as_ref().unwrap();
            let (peer, invite) = loop {
                if is_quit_requested() || is_key_pressed(KeyCode::Escape) { return Ok(()); }
                if let Some(ready) = setup.take_ready()? { break ready; }
                clear_background(BLACK);
                super::menu::panel(&ui, "Host co-op");
                super::menu::text(&ui, match setup.status() {
                    Status::PreparingHelper => "Preparing internet hosting...",
                    Status::CheckingConnection => "Checking your invite...",
                    _ => "Creating your invite...",
                }, 150., 18.);
                setup_input.update(&prefs, crate::look::window_focused());
                if super::menu::buttons(&ui, &setup_input, &mut setup_selected, &["Cancel"], 280.).is_some() { return Ok(()); }
                if crate::look::window_focused() { ui.cursor(); }
                super::menu::capture();
                next_frame().await;
            };
            let mut visit = resumed.as_ref().map(|saved| saved.checkpoint.visit.clone()).or_else(|| launch.start.clone())
                .unwrap_or_else(|| Visit { epoch: 1, map: options.map.clone(), entry: options.entry.clone(), difficulty: options.difficulty, combat_preview: options.combat_preview });
            // A resumed file carries gameplay clocks, never a previous network session.
            visit.epoch = 1;
            let session = Host::new(peer, identity, visit)?;
            if let Some(path) = &launch.invite_file {
                if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
                std::fs::write(path, invite.link())?;
            }
            (Some(session), None, Some(invite))
        } else {
            let invite = if let Some(invite) = &launch.invite { invite.clone() } else {
                Invite::parse(&std::fs::read_to_string(launch.invite_file.as_ref().context("Paste an invite link first.")?).context("The host invite is not ready.")?)?
            };
            (None, Some(Guest::new(transport::join(invite, launch.conditions)?, identity, Instant::now())?), None)
        };
        let local = if launch.host { PlayerId::Blue } else { PlayerId::Red };
        let mut visit = host.as_ref().map(|h| h.visit().clone());
        while visit.is_none() {
            if is_quit_requested() || is_key_pressed(KeyCode::Escape) { return Ok(()); }
            let session = guest.as_mut().unwrap();
            session.poll(Instant::now())?;
            visit = session.take_visit();
            clear_background(BLACK);
            super::menu::panel(&ui, "Join co-op");
            super::menu::text(&ui, "Connecting to your friend...", 150., 18.);
            setup_input.update(&prefs, crate::look::window_focused());
            if super::menu::buttons(&ui, &setup_input, &mut setup_selected, &["Cancel"], 280.).is_some() { return Ok(()); }
            if crate::look::window_focused() { ui.cursor(); }
            super::menu::capture();
            next_frame().await;
        }
        let mut visit = visit.unwrap();
        anyhow::ensure!(launch.death_check.is_none() || visit.combat_preview,
            "The co-op death check requires the staged school combat preview.");
        anyhow::ensure!(launch.scene_check.is_none() || visit.map == "skool1" && visit.entry.is_none()
            && !visit.combat_preview && launch.death_check.is_none(), "The scene fixture requires the first school visit.");
        anyhow::ensure!(!launch.travel_check || visit.map == "skool1" && !visit.combat_preview
            && launch.death_check.is_none() && launch.scene_check.is_none(), "The travel fixture requires the school.");
        anyhow::ensure!(!launch.checkpoint_check || !visit.combat_preview && launch.death_check.is_none()
            && launch.scene_check.is_none() && (launch.travel_check || !launch.host || resumed.is_some()),
            "The checkpoint fixture requires staged travel or a resumed host");
        clear_background(BLACK);
        ui.toast("Opening Wonderland for two...", screen_height() * 0.5);
        next_frame().await;
        let mut scene = crate::render::Scene::load_for(assets, &visit.map, visit.difficulty)?;
        audio.load(assets,&visit.map,&scene.map);
        let mut arena = super::arena::Arena::load(assets, &scene.map, &mut scene.world, &visit, visit.combat_preview)?;
        if let Some(saved) = &resumed {
            arena.restore_checkpoint(saved.checkpoint.frame.combat.as_ref().unwrap(), &scene.map, &mut scene.world)?;
        }
        let mut level_art = crate::viewer::LevelArt::default();
        level_art.prepare(assets, &scene, &arena.stage.interactions, false)?;
        let mut effects = super::effects::Effects::load(assets, &scene.map)?;
        let hud = crate::hud::Hud::load(assets)?;
        let mut aim_pointer = crate::targeting::Pointer::load(assets)?;
        let mut pickup_art = crate::powerups::Art::load(assets)?;
        let (eye, initial_yaw) = crate::interaction::spawn(&scene.map, visit.entry.as_deref());
        let eye = options.start_at.map_or(eye, |p| p + Vec3::Z * crate::movement::EYE_HEIGHT);
        let eye = if launch.scene_check.is_some() { vec3(-2440., 2368., -480. + crate::movement::EYE_HEIGHT) } else { eye };
        let mut stats = Stats::for_level(&visit.map, visit.entry.as_deref());
        stats.difficulty = visit.difficulty;
        let mut roster = if let Some(saved) = &resumed {
            Some(Roster::from_checkpoint(&saved.checkpoint.frame, &scene.world, |at| !arena.stage.interactions.hazardous(at))?)
        } else if launch.host {
            let mut initial = Roster::new_with(&scene.world, eye, initial_yaw, stats, |at| !arena.stage.interactions.hazardous(at))?;
            if launch.scene_check == Some(PlayerId::Blue) {
                // The staged entrance is a narrow single-file approach. Assign
                // its collision-checked front slot to the requested first
                // arrival before simulation, rather than driving Blue through
                // the waiting Red. The real contact still starts the scene.
                initial.players.swap(0, 1);
                initial.remember_checkpoint();
            }
            Some(initial)
        } else { None };
        if resumed.is_none() {
            if let Some(roster) = &roster { arena.reset_characters(&roster.players); }
        }
        if launch.checkpoint_check || launch.route_driver.is_some() {
            if let Some(saved) = &resumed {
                super::checkpoint::verify_restored(&saved.checkpoint, roster.as_ref().unwrap(), &arena)?;
                println!("PASS co-op checkpoint fixture: both players, characters, world and NPCs restored exactly before simulation");
                if launch.route_driver.is_some() {
                    std::fs::write(crate::preferences::path("route-resume.json"), serde_json::to_vec(
                        &serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"), "verified":true,
                            "epoch":visit.epoch, "tick":saved.checkpoint.frame.tick}))?)?;
                }
            }
        }
        if let Some(host) = &mut host { host.loaded(visit.epoch)?; }
        if let Some(guest) = &mut guest { guest.loaded(visit.epoch)?; }
        let mut display = roster.as_ref().map(|r| r.frame(visit.epoch, 0));
        let mut presented: Option<Frame> = None;
        let mut yaw = initial_yaw;
        let mut pitch = 0.;
        let mut look = crate::look::MouseLook::default();
        look.sensitivity = prefs.sensitivity;
        look.inverted = prefs.invert_mouse;
        let mut controls = crate::input::Input::default();
        let mut skip_prompt = crate::cinematic::Skip::default();
        let mut camera = crate::camera::Follow::default();
        let mut camera_subject = None;
        let mut local_latch = RemoteInput::default();
        let mut keys = Input::default();
        let mut local_sequence = 0;
        let mut simulation_clock = super::clock::Clock::default();
        let mut menu = false;
        let mut chapters_open = false;
        let maps = assets.maps();
        let choices = crate::campaign::level_choices(&maps);
        let mut chapters = crate::chapters::Chapters::default();
        let mut chapter_difficulty = visit.difficulty;
        let mut console = crate::console::Console::new();
        let mut camera_distance = 180.;
        let detailed_reports = launch.drive || crate::ui_probe::enabled()
            || std::env::var("LOOKING_GLASS_COOP_DIAGNOSTICS").as_deref() == Ok("1");
        let report_writer = detailed_reports.then(|| super::diagnostics::Writer::new(crate::preferences::path("coop-live.json")));
        let mut performance_open = std::env::var("LOOKING_GLASS_COOP_METRICS").as_deref() == Ok("1");
        let mut performance_writer = None;
        let renderer = super::diagnostics::renderer();
        let mut performance_frames = super::diagnostics::Frames::default();
        let mut performance_work = super::diagnostics::Work::default();
        let mut previous_work = [0.;12];
        let mut performance_text = String::new();
        let mut menu_selected = 0;
        let mut copied_at: Option<Instant> = None;
        let mut rejected_guest = String::new();
        let mut snapshot_clock = super::clock::Snapshots::default();
        let mut rendered = 0;
        let mut started_playing = None;
        let mut death_approach = None;
        let mut report_at = Instant::now();
        let mut input_clock = super::clock::Inputs::new(Instant::now());
        let mut weapon_buttons = crate::weapon_rules::Buttons::default();
        weapon_buttons.block();
        let mut captured_midpoint = false;
        let mut captured_recovery = [false; 3];
        let mut restored_at = None;
        let mut pending_visit = None;
        let mut campaign = resumed.as_ref().map(|s| s.checkpoint.campaign.clone()).unwrap_or_default();
        // Route tests use isolated profiles and exercise production checkpoint
        // behavior. The older staged movement/combat fixtures still opt out.
        let automatic_checkpoints = !launch.drive || launch.route_driver.is_some();
        let mut checkpoint_retry: Option<Instant> = None;
        let mut finale_at: Option<Instant> = None;
        let mut finale_sent: Option<u64> = None;
        let mut checkpoint_pending = (resumed.is_none() || importing) && launch.host && automatic_checkpoints && !visit.combat_preview;
        let mut checkpoint_message = resumed.as_ref().map(|s| (Instant::now(), if importing {
            "Your single-player progress is ready for two. Co-op saves separately.".to_owned()
        } else if s.recovered_backup {
            "Recovered the previous co-op checkpoint from its backup.".to_owned()
        } else { "Co-op checkpoint restored for both players.".to_owned() }));
        let mut leaving_visit = None;
        let mut local_location = None;
        let mut reunited_at = None;
        let mut captured_scene = false;
        let mut entered_tick = 0;
        let mut travel_used = false;
        let mut travel_returned_at = None;
        let mut travel_captured = false;
        let mut fixture_checkpoint_saved = false;
        let mut fixture_checkpoint_wiped = false;
        let mut checkpoint_captured = false;
        let mut route_driver = launch.route_driver.clone().map(super::route_driver::Driver::new);
        loop {
            let now = Instant::now();
            let mut guest_poll_ms = 0.;
            let mut replicate_ms = 0.;
            let mut driver_ms = 0.;
            let mut simulation_ms = 0.;
            let mut send_ms = 0.;
            let mut draw_ms = 0.;
            let mut draw_phases = [0.;6];
            if let Some(driver) = &mut route_driver { driver.poll(now); }
            if let Some(setup) = &hosting {
                if let Status::Failed(reason) = setup.status() { anyhow::bail!("{reason}"); }
            }
            if let Some(host) = &mut host {
                // A bad/incompatible guest is disconnected by Host::poll.
                // Keep the host and its invite available for a valid rejoin.
                if let Err(error) = host.poll(now) {
                    eprintln!("Co-op guest rejected: {error}");
                    rejected_guest = error.to_string();
                }
                if host.phase() == Phase::Playing { rejected_guest.clear(); }
            }
            if let Some(guest) = &mut guest {
                let polling = Instant::now();
                if let Err(error) = guest.poll(now) {
                    if arena.stage.interactions.levels.iter().any(|s| s.ctl.ending_ready()) {
                        completed = true; break;
                    }
                    return Err(error);
                }
                guest_poll_ms = polling.elapsed().as_secs_f32()*1000.;
                if let Some(next) = guest.take_visit() { pending_visit = Some(next); }
                if let Some(mut frame) = guest.take_frame() {
                    if let Some(combat) = frame.combat.take() {
                        let restoring = Instant::now();
                        arena.replicate(&combat, &scene.map, &mut scene.world)?;
                        replicate_ms = restoring.elapsed().as_secs_f32()*1000.;
                        if let Some(driver) = &mut route_driver { driver.record_state(&frame, &arena)?; }
                    }
                    display = Some(frame);
                }
            }
            if pending_visit.is_some() && host.as_ref().is_none_or(|host| host.can_load()) {
                let next = pending_visit.take().unwrap();
                clear_background(BLACK);
                ui.toast("Travelling to the next part of Wonderland...", screen_height() * 0.5);
                next_frame().await;
                let mut next_scene = crate::render::Scene::load_for(assets, &next.map, next.difficulty)?;
                let mut next_arena = super::arena::Arena::load(assets, &next_scene.map, &mut next_scene.world, &next, false)?;
                if launch.host {
                    let saved = leaving_visit.as_ref().filter(|saved: &&super::campaign::VisitState|
                        saved.key() == crate::save::visit_key(&next.map, next.entry.as_deref()))
                        .or_else(|| campaign.cached(&next));
                    if let Some(saved) = saved {
                        next_arena.restore_visit(saved, &next, &next_scene.map, &mut next_scene.world)?;
                        if launch.travel_check {
                            anyhow::ensure!(serde_json::to_value(&saved.stage.world)? == serde_json::to_value(next_arena.stage.interactions.snapshot())?
                                && serde_json::to_value(&saved.npcs)? == serde_json::to_value(next_arena.npcs.snapshot())?, "Co-op cached world did not restore exactly");
                            println!("PASS co-op travel fixture: cached shared world and NPC state restored exactly");
                        }
                    }
                }
                let mut next_art = crate::viewer::LevelArt::default();
                next_art.prepare(assets, &next_scene, &next_arena.stage.interactions, false)?;
                let (eye, facing) = crate::interaction::spawn(&next_scene.map, next.entry.as_deref());
                let arrived = if let Some(old) = &roster {
                    Some(old.arrive_with(&next_scene.world, eye, facing, &next.map, next.entry.as_deref(), |at| !next_arena.stage.interactions.hazardous(at))?)
                } else if launch.host {
                    let mut stats = Stats::for_level(&next.map, next.entry.as_deref());
                    stats.difficulty = next.difficulty;
                    Some(Roster::new_with(&next_scene.world, eye, facing, stats, |at| !next_arena.stage.interactions.hazardous(at))?)
                } else { None };
                if let Some(arrived) = &arrived { next_arena.reset_characters(&arrived.players); }
                if let Some(leaving) = leaving_visit.take() { campaign.departed(leaving)?; }
                roster = arrived;
                entered_tick = roster.as_ref().map_or(0, |r| r.tick);
                scene = next_scene;
                arena = next_arena;
                level_art = next_art;
                effects = super::effects::Effects::load(assets, &scene.map)?;
                visit = next;
                audio.load(assets,&visit.map,&scene.map);
                checkpoint_pending = launch.host && automatic_checkpoints && !visit.combat_preview;
                if let Some(host) = &mut host { host.loaded(visit.epoch)?; }
                if let Some(guest) = &mut guest { guest.loaded(visit.epoch)?; }
                display = roster.as_ref().map(|r| r.frame(visit.epoch, 0));
                presented = None;
                camera.reset();
                camera_subject = None;
                local_location = None;
                yaw = facing;
                pitch = 0.;
                keys = Input::default();
                local_latch = RemoteInput::default();
                simulation_clock = super::clock::Clock::default();
                snapshot_clock = super::clock::Snapshots::default();
                weapon_buttons.block();
                // Do not charge the synchronous asset-loading time to movement.
                next_frame().await;
                continue;
            }
            if arena.stage.interactions.levels.iter().any(|s| s.ctl.ending_ready()) {
                let at = *finale_at.get_or_insert(now);
                // The ordinary replicated world contains completion. Keep polling
                // until the guest receipts that frame before closing the host.
                if host.as_ref().is_some_and(|h| finale_sent.is_some_and(|tick| h.acknowledged_frame(tick)))
                    || now.duration_since(at) >= Duration::from_secs(5) {
                    completed = true; break;
                }
            }
            let playing = host.as_ref().map_or_else(|| guest.as_ref().unwrap().phase() == Phase::Playing,
                |h| h.phase() == Phase::Playing);
            if playing { started_playing.get_or_insert(now); }
            let elapsed = get_frame_time().clamp(0., 0.1);
            let focused = crate::look::window_focused();
            if focused && is_key_pressed(KeyCode::F11) { performance_open = !performance_open; }
            if performance_open && playing {
                performance_work.sample(previous_work);
                if let Some(timing) = performance_frames.sample(get_frame_time()) {
                    performance_text = format!("{:.0} FPS / worst frame {:.1} ms", timing["fps"].as_f64().unwrap_or(0.), timing["max_ms"].as_f64().unwrap_or(0.));
                    let writer = performance_writer.get_or_insert_with(|| super::diagnostics::Writer::new(crate::preferences::path("coop-performance.json")));
                    if let Ok(bytes) = serde_json::to_vec(&serde_json::json!({"build":env!("LOOKING_GLASS_BUILD"),
                        "pid":std::process::id(),"host":launch.host,"playing":playing,"map":visit.map,"renderer":renderer,
                        "local_view":display.as_ref().map(|f| serde_json::json!({
                            "feet":f.players[local.index()].player.feet,"yaw":yaw,"pitch":pitch})),
                        "scene":arena.stage.busy(),"timing":timing,"work":performance_work.take(),"network":guest.as_ref().map(Guest::input_timing)
                            .or_else(|| host.as_ref().map(Host::input_timing))})) { writer.write(bytes); }
                }
            }
            controls.update(&prefs, focused);
            let mut chars = std::iter::from_fn(get_char_pressed).collect::<Vec<_>>();
            chars.reverse();
            let toggle_console = focused && (is_key_pressed(KeyCode::GraveAccent)
                || chars.iter().any(|c| matches!(c, '`' | '~' | '\u{00ac}')));
            if toggle_console {
                console.open = !console.open; menu = false; chapters_open = false;
                controls.suppress(); chars.clear();
            } else if focused && controls.ui(KeyCode::Escape) {
                if console.open { console.open = false; }
                else if chapters_open { chapters_open = false; }
                else { menu = !menu; menu_selected = 0; }
                controls.suppress();
            }
            if focused && !console.open && controls.key(&prefs, KeyCode::Tab, true) {
                chapters_open = !chapters_open; menu = false;
                chapter_difficulty = visit.difficulty;
                let selected = maps.iter().position(|m| m == &visit.map).unwrap_or(0);
                chapters.open(crate::campaign::choice_position(&choices, selected, visit.entry.as_deref()), choices.len());
            }
            if is_quit_requested() || menu && focused && is_key_pressed(KeyCode::Q) { break; }
            let mut travel = None;
            if console.open && focused {
                if let Some(command) = console.input(&chars) {
                    use crate::console::Command;
                    match command {
                        Err(error) => console.print(error),
                        Ok(Command::Help) => {
                            console.print("help clear version status maplist cg_cameradist quit");
                            console.print("Host: map <level[$entrance]> restart save");
                            console.print("Tab opens chapters and difficulty. Continue co-op in the main menu loads the shared save.");
                        }
                        Ok(Command::Clear) => console.clear(),
                        Ok(Command::Version) => console.print(format!("Looking Glass {} / co-op", env!("CARGO_PKG_VERSION"))),
                        Ok(Command::Status) => console.print(format!("{} / {} / {}", visit.map, visit.difficulty.name(), if launch.host { "Host (Blue)" } else { "Guest (Red)" })),
                        Ok(Command::Maps) => for choice in &choices {
                            console.print(format!("{}{}", maps[choice.map], choice.entry.map_or(String::new(), |e| format!("${e}"))));
                        },
                        Ok(Command::CameraDistance(value)) => {
                            if let Some(value) = value { camera_distance = value.clamp(32., 600.); }
                            console.print(format!("Co-op camera distance: {camera_distance}"));
                        }
                        Ok(Command::Quit) => break,
                        Ok(Command::Map(map, entry)) if launch.host => {
                            if choices.iter().any(|c| maps[c.map] == map && c.entry == entry.as_deref()) {
                                travel = Some((map, entry, visit.difficulty));
                            } else { console.print("Unknown chapter or entrance. Use maplist."); }
                        }
                        Ok(Command::Restart) if launch.host => travel = Some((visit.map.clone(), visit.entry.clone(), visit.difficulty)),
                        Ok(Command::Save) if launch.host => {
                            match roster.as_mut().context("Wait for chapter loading to finish")
                                .and_then(|r| checkpoint_store.save(&visit, r, &mut arena, &scene.world, &campaign)) {
                                Ok(()) => { checkpoint_pending = false; console.print("Co-op checkpoint saved for both players."); }
                                Err(error) => console.print(format!("Save failed: {error}")),
                            }
                        }
                        Ok(Command::Map(..) | Command::Restart | Command::Save) => console.print("The host controls shared chapters and checkpoints."),
                        Ok(_) => console.print("This command is unavailable in co-op. Type help for supported commands."),
                    }
                }
            }
            if chapters_open && focused {
                let action = chapters.update(&controls, choices.len(), &mut chapter_difficulty);
                if action == Some(crate::chapters::Hit::Back) { chapters_open = false; }
                else if !choices.is_empty() && (action == Some(crate::chapters::Hit::Begin) || controls.ui(KeyCode::Enter)) {
                    if launch.host {
                        let choice = &choices[chapters.selected];
                        travel = Some((maps[choice.map].clone(), choice.entry.map(str::to_owned), chapter_difficulty));
                    } else { checkpoint_message = Some((now, "Ask the host to start a shared chapter.".into())); }
                }
            }
            if let Some((map, entry, difficulty)) = travel {
                host.as_mut().unwrap().enter(map, entry, difficulty, now)?;
                pending_visit = Some(host.as_ref().unwrap().visit().clone());
                roster = None; leaving_visit = None; campaign = Default::default();
                menu = false; console.open = false; chapters_open = false;
                controls.suppress();
                next_frame().await;
                continue;
            }
            let overlay = menu || chapters_open || console.open;
            if focused && is_key_pressed(KeyCode::F8) {
                if let Some(invite) = &invite { miniquad::window::clipboard_set(&invite.link()); copied_at = Some(now); }
            }
            if let Some(frame) = &display {
                let body = &frame.players[local.index()];
                let location = (visit.epoch, body.spawned_at, body.warped_at);
                if local_location != Some(location) {
                    yaw = body.yaw;
                    pitch = body.pitch;
                    local_location = Some(location);
                }
            }
            look.update(focused && !overlay && !launch.drive && playing, &mut yaw, &mut pitch);
            let flat = vec2(yaw.cos(), yaw.sin());
            let right = vec2(yaw.sin(), -yaw.cos());
            let direction = vec3(yaw.cos() * pitch.cos(), yaw.sin() * pitch.cos(), pitch.sin());
            let input_active = focused && !overlay && playing && !launch.drive;
            let held = |key| input_active && controls.key(&prefs, key, false);
            let pressed = |key| input_active && controls.key(&prefs, key, true);
            let axes = if input_active { controls.movement() } else { Vec2::ZERO };
            let forward = axes.y + f32::from(held(KeyCode::W)) - f32::from(held(KeyCode::S));
            let sideways = axes.x + f32::from(held(KeyCode::D)) - f32::from(held(KeyCode::A));
            keys.wish = (flat * forward + right * sideways).clamp_length_max(1.);
            keys.swim = (direction * forward + right.extend(0.) * sideways).clamp_length_max(1.);
            keys.rise = f32::from(held(KeyCode::Space)) - f32::from(held(KeyCode::LeftControl));
            keys.run = input_active && prefs.run(held(KeyCode::LeftShift));
            keys.yaw = (yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
            keys.pitch = pitch;
            keys.jumps += u64::from(pressed(KeyCode::Space));
            let previous_uses = keys.uses;
            keys.uses += u64::from(pressed(KeyCode::E));
            keys.skip_scene = held(KeyCode::Enter) || input_active && controls.pad_held("A");
            let owns_scene = arena.stage.owner == Some(local)
                && display.as_ref().is_some_and(|f| f.players[local.index()].stats.alive());
            skip_prompt.update(if owns_scene { arena.stage.scene_id() } else { None },
                input_active && owns_scene, keys.skip_scene, elapsed);
            let click = weapon_buttons.sample([
                controls.action(&prefs, "Mouse 1", false), controls.action(&prefs, "Mouse 2", false)
            ], input_active);
            keys.primary = click == Some(false);
            keys.alternate = click == Some(true);
            for i in 0..2 { keys.attack_presses[i] += u64::from(weapon_buttons.pressed[i]
                && if i == 0 { keys.primary } else { keys.alternate }); }
            if !input_active { keys.weapon = None; }
            for (i, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
                KeyCode::Key5, KeyCode::Key6, KeyCode::Key7, KeyCode::Key8, KeyCode::Key9, KeyCode::Key0].into_iter().enumerate() {
                if pressed(key) { keys.weapon = Some(i as u8); }
            }
            // Deterministic local test input still travels through the ordinary
            // session and host simulation, even while this window lacks focus.
            if launch.drive && playing {
                let age = started_playing.unwrap().elapsed().as_secs_f32();
                keys.run = false;
                keys.wish = if (2. ..4.).contains(&age) { if launch.host { Vec2::Y } else { Vec2::X } } else { Vec2::ZERO };
                keys.swim = keys.wish.extend(0.);
                if let Some(driver) = &mut route_driver {
                    if !launch.host {
                        if let Some(frame) = &display { driver.sample(frame, &arena, &scene.world, &scene.map, local, &mut keys, now); }
                        yaw = keys.yaw; pitch = keys.pitch;
                    }
                } else if launch.travel_check {
                    let tick = display.as_ref().map_or(0, |f| f.tick);
                    keys.wish = if visit.epoch == 1 && (120..960).contains(&tick) { Vec2::Y } else { Vec2::ZERO };
                    keys.yaw = std::f32::consts::FRAC_PI_2;
                    keys.pitch = 0.;
                    keys.swim = keys.wish.extend(0.);
                    keys.primary = false;
                    keys.alternate = false;
                    keys.weapon = None;
                    if launch.host && visit.epoch == 1 && tick >= 1000 && !travel_used {
                        keys.uses += 1;
                        travel_used = true;
                    }
                } else if launch.checkpoint_check {
                    keys.wish = Vec2::ZERO;
                    keys.swim = Vec3::ZERO;
                    keys.primary = false;
                    keys.alternate = false;
                    keys.weapon = None;
                } else if let Some(first) = launch.scene_check {
                    keys.wish = if local == first && age >= 2. && !arena.stage.busy()
                        && display.as_ref().is_some_and(|f| f.players.iter().all(|b| b.warped_at == 0)) { -Vec2::X } else { Vec2::ZERO };
                    keys.yaw = std::f32::consts::PI;
                    keys.pitch = 0.;
                    keys.swim = keys.wish.extend(0.);
                    keys.primary = false;
                    keys.alternate = false;
                    keys.weapon = None;
                } else if let Some(victim) = launch.death_check {
                    // Let the real guard inflict damage. No health writes or
                    // synthetic kill commands are used by this scenario.
                    keys.wish = Vec2::ZERO;
                    if victim == PlayerId::Blue && local == PlayerId::Blue && age >= 2. {
                        if let Some(frame) = &display {
                            // The contact radius prevents walking through Red.
                            // Walk around her using measured arrivals, so Blue
                            // becomes the nearest target without moving either
                            // body directly or changing enemy/damage rules.
                            let (anchor, step) = death_approach.get_or_insert((
                                frame.players[PlayerId::Red.index()].player.feet.truncate(), 0));
                            let offsets = [vec2(64., -48.), vec2(64., 64.), vec2(0., 64.)];
                            if let Some(offset) = offsets.get(*step) {
                                let delta = *anchor + *offset - frame.players[local.index()].player.feet.truncate();
                                if delta.length() < 4. { *step += 1; }
                                else { keys.wish = delta.normalize(); }
                            }
                        }
                    }
                    // The survivor walks during the first part of the countdown
                    // so the spectator check exercises a moving camera subject.
                    if local != victim && display.as_ref().is_some_and(|frame|
                        matches!(frame.lives.state(victim), super::rules::Life::Spectating { ticks_left } if ticks_left > 1120)) {
                        keys.wish = if local == PlayerId::Blue { Vec2::Y } else { Vec2::X };
                    }
                    keys.swim = keys.wish.extend(0.);
                    keys.primary = false;
                    keys.alternate = false;
                    keys.weapon = None;
                } else if visit.combat_preview {
                    // A reproducible real-weapon encounter, not injected damage.
                    keys.wish = Vec2::ZERO;
                    keys.swim = Vec3::ZERO;
                    keys.yaw = std::f32::consts::FRAC_PI_2;
                    keys.pitch = 0.;
                    keys.weapon = Some(0);
                    keys.primary = false;
                    if let (Some(frame), Some(target)) = (&display, arena.npcs.targets().first()) {
                        let delta = target.center - frame.players[local.index()].player.eye();
                        keys.yaw = delta.y.atan2(delta.x);
                        keys.pitch = (delta.z / delta.length().max(1.)).asin().clamp(-1.56, 1.56);
                        // Prepare a clear approach around Red's contact radius
                        // while she remains the nearer guard target.
                        if local == PlayerId::Blue && (2. ..8.).contains(&age) {
                            let approach = frame.players[PlayerId::Red.index()].player.feet.truncate()
                                + vec2(96., -48.) - frame.players[local.index()].player.feet.truncate();
                            if approach.length() > 4. { keys.wish = approach.normalize(); }
                        }
                        // Give the guard time to strike the nearer Red player
                        // before both blades can interrupt its attack animation.
                        if (8. ..13.).contains(&age) {
                            keys.wish = if delta.truncate().length() > 60. {
                                delta.truncate().normalize_or_zero()
                            } else { Vec2::ZERO };
                            // Coordinate the first attacks so one blade cannot
                            // finish the encounter before its partner arrives.
                            keys.primary = delta.length() < 95.
                                && frame.players.iter().all(|body| target.center.distance(body.player.eye()) < 95.);
                        }
                    }
                }
            }
            if let Some(frame) = &display {
                keys.classify_rope_use(previous_uses, &frame.players[local.index()].player, &scene.world,
                    arena.stage.busy() || arena.stage.story.busy());
            }
            // Only the host simulates a local body. A guest sends its counters
            // to the host; an unused local latch would accumulate every E/jump
            // until its pending-edge limit ended a long guest session.
            if launch.host && !(launch.drive && route_driver.is_some()) {
                local_sequence += 1;
                local_latch.accept(local_sequence, keys, now)?;
            }
            if let Some(guest) = &mut guest {
                let sampled = Instant::now();
                if input_clock.due(sampled) && guest.controls(keys)? { input_clock.sent(sampled); }
            }
            if let (Some(host), Some(roster)) = (&mut host, &mut roster) {
                if playing {
                    let (steps, suspended) = simulation_clock.advance(get_frame_time());
                    if suspended {
                        if let Some(driver)=&mut route_driver {
                            driver.record_input_event(serde_json::json!({"kind":"suspended",
                                "epoch":visit.epoch,"tick":roster.tick,"frame_ms":get_frame_time()*1000.,
                                "before":host.input_timing()}));
                        }
                        host.cancel_controls()?; local_latch.cancel();
                    }
                    for _ in 0..steps {
                        // The diagnostic planner describes exact 24-tick input
                        // segments. Sampling once per render frame can hold a
                        // turn past its boundary, then discard the plan while
                        // Alice is still moving toward an edge. Sample the host
                        // from the current body before every physics step.
                        // Human controls and the remote input path are unchanged.
                        if launch.drive {
                            if let Some(driver) = &mut route_driver {
                                let sampling=Instant::now();
                                let frame = roster.frame(visit.epoch, host.acknowledged_input());
                                driver.sample(&frame, &arena, &scene.world, &scene.map, local, &mut keys, now);
                                yaw = keys.yaw; pitch = keys.pitch;
                                local_sequence += 1;
                                local_latch.accept(local_sequence, keys, now)?;
                                driver_ms+=sampling.elapsed().as_secs_f32()*1000.;
                            }
                        }
                        let check_wipe = launch.checkpoint_check && resumed.as_ref().is_some_and(|s|
                            roster.tick >= s.checkpoint.frame.tick + 120) && !fixture_checkpoint_wiped;
                        if check_wipe {
                            for body in &mut roster.players { body.stats.damage(10000.); }
                            anyhow::ensure!(checkpoint_store.save(&visit, roster, &mut arena, &scene.world, &campaign).is_err(),
                                "A dead team replaced its checkpoint");
                        }
                        let was_cinematic=arena.stage.busy();
                        if let Some(driver)=&mut route_driver {
                            driver.record_recovery(roster,&arena,&scene.world,visit.epoch,&visit.map,was_cinematic);
                        }
                        let inputs=[local_latch.sample(now), host.controls(now)];
                        let jump_event=route_driver.as_ref().filter(|_|inputs.iter().any(|(c,_)|c.jump))
                            .map(|_|serde_json::json!({"kind":"jump","epoch":visit.epoch,"tick":roster.tick,
                                "input":inputs.iter().map(|(c,_)|c.jump).collect::<Vec<_>>(),
                                "before":roster.players.each_ref().map(|b|(&b.player.feet,b.player.grounded,b.player.jumps)),
                                "attack_locked":arena.characters.each_ref().map(|c|c.attack_movement_locked()),
                                "network":host.input_timing()}));
                        let previous_generation = roster.checkpoint_generation;
                        let stepping=Instant::now();
                        arena.step(roster, &scene.map, &mut scene.world, inputs)?;
                        simulation_ms+=stepping.elapsed().as_secs_f32()*1000.;
                        if route_driver.is_some() && roster.checkpoint_generation != previous_generation {
                            // Observe an ordinary gameplay wipe before another simulation
                            // tick can age timers or move enemies. This opt-in receipt never
                            // kills players, loads a checkpoint or changes the restored state.
                            let saved = checkpoint_store.load()?;
                            super::checkpoint::verify_restored(&saved.checkpoint, roster, &arena)?;
                            anyhow::ensure!(serde_json::to_value(&visit)? == serde_json::to_value(&saved.checkpoint.visit)?
                                && serde_json::to_value(&campaign)? == serde_json::to_value(&saved.checkpoint.campaign)?,
                                "Team wipe changed the saved visit or campaign progress");
                            anyhow::ensure!(roster.checkpoint_generation == previous_generation + 1,
                                "Team wipe skipped a checkpoint generation");
                            let path = crate::preferences::path(&format!("route-wipe-{}.json", roster.tick));
                            std::fs::write(path, serde_json::to_vec(&serde_json::json!({
                                "passed": true, "build": env!("LOOKING_GLASS_BUILD"),
                                "tick": roster.tick, "generation": roster.checkpoint_generation,
                                "previous_generation": previous_generation,
                                "recovered_backup": saved.recovered_backup, "checkpoint": saved.checkpoint
                            }))?)?;
                        }
                        if let Some(driver)=&mut route_driver {
                            if let Some(mut event)=jump_event {
                                event["after"]=serde_json::json!(roster.players.each_ref()
                                    .map(|b|(&b.player.feet,b.player.grounded,b.player.jumps)));
                                driver.record_input_event(event);
                            }
                            driver.record_recovery(roster,&arena,&scene.world,visit.epoch,&visit.map,
                                was_cinematic || arena.stage.busy());
                        }
                        if check_wipe {
                            let saved = resumed.as_ref().unwrap();
                            super::checkpoint::verify_restored(&saved.checkpoint, roster, &arena)?;
                            anyhow::ensure!(roster.checkpoint_generation == saved.checkpoint.frame.checkpoint_generation + 1,
                                "Both deaths failed to restore the saved checkpoint");
                            std::fs::write(crate::preferences::path("coop-checkpoint-verified.json"), serde_json::to_vec(&serde_json::json!({
                                "passed": true, "recovered_backup": saved.recovered_backup, "epoch": visit.epoch,
                                "generation": roster.checkpoint_generation, "completed_visits": campaign.completed()
                            }))?)?;
                            println!("PASS co-op checkpoint fixture: team wipe restored saved bodies, resources, character actions, shared world and NPCs exactly");
                            fixture_checkpoint_wiped = true;
                        }
                    }
                    if launch.checkpoint_check && launch.travel_check && visit.epoch == 3
                        && roster.tick >= entered_tick + 120 && !fixture_checkpoint_saved {
                        // Explicit fixture resources distinguish the two saved inventories.
                        roster.players[0].stats.set_health(83.)?;
                        roster.players[1].stats.set_health(47.)?;
                        roster.players[0].stats.spend_will(5.);
                        roster.players[1].stats.spend_will(11.);
                        checkpoint_store.save(&visit, roster, &mut arena, &scene.world, &campaign)?;
                        checkpoint_store.save(&visit, roster, &mut arena, &scene.world, &campaign)?;
                        fixture_checkpoint_saved = true;
                        println!("PASS co-op checkpoint fixture: saved shared visit history and distinct player resources with a good backup");
                    }
                    let mut frame = roster.frame(visit.epoch, host.acknowledged_input());
                    frame.sounds = Some(arena.stage.sounds.batch(frame.tick));
                    if snapshot_clock.due(frame.tick) && host.can_send_frame() {
                        let sending=Instant::now();
                        let mut network = frame.clone();
                        network.combat = Some(Box::new(arena.snapshot()));
                        if host.send_frame(network)? {
                            if let Some(driver) = &mut route_driver { driver.record_state(&frame, &arena)?; }
                            snapshot_clock.sent(frame.tick);
                            if arena.stage.interactions.levels.iter().any(|s| s.ctl.ending_ready()) {
                                finale_sent.get_or_insert(frame.tick);
                            }
                        }
                        send_ms=sending.elapsed().as_secs_f32()*1000.;
                    }
                    display = Some(frame);
                    if launch.drive {
                        if let Some(driver) = &mut route_driver {
                            if let Some(command) = driver.checkpoint_request(visit.epoch, now) {
                                // Use the normal host menu operation, including all save
                                // validation and its atomic disk-before-memory commit.
                                let save_started = Instant::now();
                                let result = checkpoint_store.save(&visit, roster, &mut arena, &scene.world, &campaign);
                                if result.is_ok() { checkpoint_pending = false; }
                                driver.checkpoint_result(command, visit.epoch, roster.tick, save_started.elapsed(),
                                    result.err().map(|error| error.to_string()));
                            }
                        }
                    }
                    let requested = arena.stage.interactions.levels.iter().any(|s| s.ctl.checkpoint_requested());
                    if automatic_checkpoints && (checkpoint_pending || requested)
                        && checkpoint_retry.is_none_or(|at| now.duration_since(at) >= Duration::from_secs(5))
                        && !arena.stage.busy() && !arena.stage.story.busy()
                        && roster.players.iter().all(|b| b.stats.alive()) {
                        checkpoint_message = Some((now, match checkpoint_store.save(&visit, roster, &mut arena, &scene.world, &campaign) {
                            Ok(()) => { checkpoint_pending = false; checkpoint_retry = None;
                                "Co-op checkpoint saved for both players.".into() },
                            Err(error) => { checkpoint_retry = Some(now); format!("Checkpoint was not saved: {error}") },
                        }));
                    }
                    // Explicit travel fixture: it requests normal load barriers
                    // without claiming the quest-gated exits were played through.
                    if launch.travel_check && arena.stage.transition.is_none() {
                        if visit.epoch == 1 && roster.tick >= 1440 {
                            arena.stage.transition = Some(("skool1".into(), Some("skool1_start2".into())));
                        } else if visit.epoch == 2 && roster.tick >= entered_tick + 600 {
                            arena.stage.transition = Some(("skool1".into(), Some("skool1_start1".into())));
                        }
                    }
                    if let Some((map, entry)) = arena.stage.transition.take() {
                        leaving_visit = Some(super::campaign::VisitState::capture(&visit, &arena));
                        host.enter(map, entry, visit.difficulty, now)?;
                        pending_visit = Some(host.visit().clone());
                    }
                } else { simulation_clock = super::clock::Clock::default(); }
            }
            let drawing=Instant::now();
            clear_background(BLACK);
            if let Some(frame) = &display {
                let mut visual = frame.clone();
                let presentation_previous = if launch.host { None } else {
                    presented.as_ref().map(|p| p.players.each_ref().map(|b|
                        (b.player.feet, b.spawned_at, b.warped_at, b.stats.alive())))
                };
                if !launch.host {
                    if let Some(previous) = &presented {
                        for (body, old) in visual.players.iter_mut().zip(&previous.players) {
                            world::interpolate_body(body, old, elapsed);
                        }
                    }
                }
                let watched = arena.stage.owner.or_else(|| arena.stage.interactive_actor())
                    .unwrap_or_else(|| world::camera_player(&visual, local));
                let watched_body = &visual.players[watched.index()];
                let camera_subject_feet = watched_body.player.feet;
                let subject = (watched, watched_body.spawned_at, watched_body.warped_at);
                if camera_subject != Some(subject) { camera.reset(); camera_subject = Some(subject); }
                let (follow_eye, follow_target, show_local) = camera.update(&scene.world, watched_body.player.feet, direction, camera_distance, elapsed);
                let scripted = arena.stage.camera(&scene.world);
                let (camera_eye, target, up) = scripted.map_or((follow_eye, follow_target, Vec3::Z), |c| (c.eye, c.target, c.up));
                let listening = target - camera_eye;
                let section = Instant::now();
                audio.update(assets,&arena,frame,&scene.world,watched,camera_eye,listening.y.atan2(listening.x),elapsed,playing)?;
                draw_phases[0] = section.elapsed().as_secs_f32()*1000.;
                let section = Instant::now();
                let view = Camera3D { position: camera_eye, target, up,
                    fovy: if scripted.is_some() { arena.stage.scene_fovy(screen_width() / screen_height()) } else { 60_f32.to_radians() },
                    z_near: 2., z_far: 30000., ..Default::default() };
                set_camera(&view);
                effects.update(assets, &scene.map, &scene.world, &arena, frame, local, camera_eye)?;
                crate::lighting::select(effects.lights(&arena), camera_eye, &scene.world);
                crate::render_fx::begin_view(&view, frame.tick as f32 * FIXED_DT, &scene.atmosphere, false);
                draw_phases[1] = section.elapsed().as_secs_f32()*1000.;
                let section = Instant::now();
                arena.stage.interactions.presentation.apply(&mut scene);
                let transforms = arena.stage.draw_transforms(&scene.world);
                let hands = arena.stage.rope_hands(&scene.world, &visual.players, &arena.characters);
                scene.update_ropes_many(elapsed, &hands, &transforms);
                scene.draw(camera_eye, frame.tick as f32 * FIXED_DT, false, false, &transforms);
                level_art.story_pose(&arena.stage.story);
                level_art.coop_view(local.index());
                level_art.draw(&arena.stage.interactions, &arena.stage.hints, &scene.atmosphere, camera_eye, false);
                draw_phases[2] = section.elapsed().as_secs_f32()*1000.;
                let section = Instant::now();
                // Rage uses the owning player's authoritative transformation
                // pose, rather than a separate cinematic Alice in LevelArt.
                let rage_actor = arena.stage.interactions.beyond.as_ref()
                    .and_then(|b| b.rage_scene())
                    .and_then(|(_, finished)| (!finished).then_some(arena.stage.speaker().index()));
                let mut visible_players = [false; 2];
                for (index, (character, body)) in arena.characters.iter_mut().zip(&visual.players).enumerate() {
                    character.power_appearance(&body.stats);
                    character.water_appearance(&body.player);
                    character.atmosphere(&scene.atmosphere, camera_eye);
                    let show_body = if scripted.is_some() { rage_actor == Some(index) } else {
                        (arena.stage.owner.is_none() || index == watched.index())
                        && follow_body_visible(index == watched.index(), show_local, camera_eye, body.player.feet)
                    };
                    let show_body = show_body && !arena.stage.interactions.levels.iter()
                        .any(|s| s.ctl.coop_hides_player(index));
                    visible_players[index] = show_body;
                    if show_body {
                        character.draw_replica(body.player.feet, frame.players[index].player.feet, false);
                    }
                    character.draw_effects(camera_eye, false, show_body);
                }
                draw_phases[3] = section.elapsed().as_secs_f32()*1000.;
                let section = Instant::now();
                arena.npcs.draw(camera_eye, (target - camera_eye).normalize_or_zero(), &scene.atmosphere, false);
                let personal = &frame.players[local.index()].stats;
                let body = &visual.players[local.index()];
                let can_aim = !overlay && playing && personal.alive() && scripted.is_none()
                    && !arena.stage.busy() && !arena.stage.story.busy()
                    && !arena.stage.interactions.level_blocks_weapons()
                    && body.player.script_motion == 0 && body.player.ledge.is_none()
                    && !body.player.climbing();
                let mut targets = arena.npcs.targets();
                if let Some(encounters) = &arena.stage.interactions.encounters { targets.extend(encounters.targets()); }
                if let Some(school) = &arena.stage.interactions.school2 { targets.extend(school.targets()); }
                if let Some(pool) = &arena.stage.interactions.pool { targets.extend(pool.targets()); }
                targets.extend(arena.stage.interactions.duchess.as_ref().and_then(|d| d.target()));
                targets.extend(crate::level::targets(&arena.stage.interactions.levels));
                targets.extend(arena.stage.interactions.shot_targets());
                aim_pointer.draw(&crate::combat::Context { world: &scene.world, targets: &targets },
                    crate::targeting::Sight { eye: body.player.eye(), direction,
                        weapon: can_aim.then(|| personal.equipped()).flatten() },
                    elapsed, camera_eye, &scene.atmosphere);
                hud.draw_pickups(&arena.pickups, personal, camera_eye, frame.tick as f32 * FIXED_DT);
                pickup_art.draw(&arena.pickups, personal, &arena.visit_key, &scene.atmosphere, camera_eye, frame.tick as f32 * FIXED_DT);
                if let Some(loot) = personal.loot.get(&arena.visit_key) {
                    pickup_art.draw(&loot.pickups(&arena.visit_key), personal, &arena.visit_key, &scene.atmosphere, camera_eye, frame.tick as f32 * FIXED_DT);
                }
                draw_phases[4] = section.elapsed().as_secs_f32()*1000.;
                let section = Instant::now();
                scene.draw_with_particles(camera_eye, listening.normalize_or_zero(),
                    frame.tick as f32 * FIXED_DT, false, &transforms, &effects.steam);
                if let Some(encounters) = &arena.stage.interactions.encounters { encounters.draw_waves(); }
                if let Some(school) = &arena.stage.interactions.school2 {
                    for boojum in &school.boojums { boojum.draw_waves(); }
                }
                level_art.effects(&arena.stage.interactions, camera_eye, &scene.atmosphere);
                crate::lighting::flares(camera_eye, listening.normalize_or_zero());
                let effect_batches = crate::render_fx::finish();
                draw_phases[5] = section.elapsed().as_secs_f32()*1000.;
                presented = Some(visual);
                set_default_camera();
                let fade = arena.stage.fade();
                if fade.a > 0. { draw_rectangle(0., 0., screen_width(), screen_height(), fade); }
                if !arena.stage.busy() { hud.draw_coop(personal, &arena.catalog, &controls, &prefs); }
                level_art.hud(&arena.stage.interactions);
                arena.stage.story.draw(&controls.label(&prefs, "E"), &ui);
                skip_prompt.draw(&ui, &controls.label(&prefs, "Enter"));
                if let Some(notice) = &arena.stage.notices[local.index()] {
                    if notice.until > frame.tick { ui.toast(&notice.text, screen_height() * 0.3); }
                }
                ui.toast(&format!("{} Alice / Sanity {:.0}", if launch.host { "Blue" } else { "Red" }, frame.players[local.index()].stats.sanity()), 24.);
                if !frame.players[local.index()].stats.alive() {
                    let message = match frame.lives.state(local) {
                        super::rules::Life::Spectating { ticks_left } => format!("Following your partner / returning in {}", (ticks_left + 119) / 120),
                        _ => "Waiting for a safe place beside your partner...".into(),
                    };
                    ui.toast(&message, screen_height() * 0.4);
                }
                draw_ms=drawing.elapsed().as_secs_f32()*1000.;
                if detailed_reports && report_at.elapsed() >= Duration::from_millis(200) {
                    let observing=Instant::now();
                    if let Some(driver) = &mut route_driver { driver.observe(frame, &arena, &scene.world); }
                    let observe_ms=observing.elapsed().as_secs_f32()*1000.;
                    // Diagnostics are optional. A reader/virus scanner can
                    // briefly hold this file on Windows; never end gameplay.
                    if let Ok(bytes) = serde_json::to_vec(&serde_json::json!({ "role": if launch.host { "host" } else { "guest" },
                        "pid": std::process::id(), "build": env!("LOOKING_GLASS_BUILD"), "playing": playing, "frame": frame, "focused": focused,
                        "session_phase": format!("{:?}", host.as_ref().map_or_else(|| guest.as_ref().unwrap().phase(), Host::phase)),
                        "pending_visit": pending_visit, "can_load": host.as_ref().map(Host::can_load),
                        "camera_player": watched, "camera": camera_eye, "camera_target": target,
                        "camera_subject_feet": camera_subject_feet,
                        "audio": audio.report,
                        "presentation": { "visible_players": visible_players, "aim_glow": aim_pointer.visible(),
                            "player_feet": presented.as_ref().map(|p| p.players.each_ref().map(|b| b.player.feet)),
                            "previous_players": presentation_previous, "elapsed": elapsed,
                            "particles": effects.steam.counts(), "effect_batches": effect_batches },
                        "network_timing": guest.as_ref().map(|g| g.input_timing())
                            .or_else(||host.as_ref().map(|h|h.input_timing())),
                        "frame_timing": {"frame_ms": get_frame_time()*1000., "guest_poll_ms": guest_poll_ms, "replicate_ms": replicate_ms,
                            "driver_ms":driver_ms,"simulation_ms":simulation_ms,"send_ms":send_ms,
                            "draw_ms":draw_ms,"observe_ms":observe_ms,
                            "simulation_debt_ms": simulation_clock.deferred_ms()},
                        "combat": arena.snapshot(), "visit": visit, "completed_visits": if launch.host { Some(campaign.completed()) } else { None },
                        "route_driver": route_driver.as_ref().map(|d| &d.report) })) {
                        report_writer.as_ref().unwrap().write(bytes);
                    }
                    report_at = now;
                }
            }
            set_default_camera();
            let hint_y = |offset| screen_height() - offset * crate::ui::overlay_scale();
            if performance_open && playing {
                ui.toast(&performance_text, hint_y(120.));
                ui.toast(&renderer, hint_y(180.));
            } else {
                ui.toast(if playing { "Co-op preview" }
                    else if started_playing.is_some() { "Your partner left. Share the invite to rejoin." }
                    else { "Waiting for the other player to finish loading..." }, hint_y(120.));
            }
            ui.toast(if menu { "Esc: resume / Q: leave co-op" } else if launch.host { "Esc: menu / F8: copy invite" } else { "Esc: menu" }, hint_y(60.));
            if let Some((at, message)) = &checkpoint_message {
                if !menu && at.elapsed() < Duration::from_secs(6) { ui.toast(message, 90.); }
            }
            // This panel is part of the current frame, not a nested waiting
            // loop. Network input, simulation and snapshots continue above it.
            if menu || !playing {
                show_mouse(false);
                super::menu::panel(&ui, if launch.host { "Hosting co-op" } else { "Co-op" });
                let message = if let Some((_, text)) = checkpoint_message.as_ref().filter(|(at, _)| at.elapsed() < Duration::from_secs(6)) {
                    text.as_str()
                } else if copied_at.is_some_and(|at| at.elapsed() < Duration::from_secs(4)) {
                    "Invite copied. Send it to your friend."
                } else if !rejected_guest.is_empty() { &rejected_guest }
                else if playing { "Your partner can keep playing while this menu is open." }
                else if started_playing.is_some() { "Your partner left. They can rejoin with the same invite." }
                else if launch.host { "Copy your invite and send it to your friend. They choose Join and paste the link." }
                else { "Waiting for both players to finish loading..." };
                if crate::ui_probe::enabled() { crate::ui_probe::note("coop-panel-message", serde_json::json!(message)); }
                super::menu::text(&ui, message, 128., 16.);
                let labels: &[&str] = if playing {
                    if launch.host { &["Resume", "Save checkpoint", "Copy invite", "Leave co-op"] } else { &["Resume", "Leave co-op"] }
                } else if launch.host && roster.is_some() { &["Copy invite", "Save checkpoint", "Leave co-op"] }
                else if launch.host { &["Copy invite", "Leave co-op"] } else { &["Leave co-op"] };
                menu_selected = menu_selected.min(labels.len() - 1);
                if let Some(choice) = super::menu::buttons(&ui, &controls, &mut menu_selected, labels, 208.) {
                    match labels[choice] {
                        "Resume" => menu = false,
                        "Copy invite" => {
                            miniquad::window::clipboard_set(&invite.as_ref().unwrap().link());
                            copied_at = Some(now);
                            checkpoint_message = None;
                        },
                        "Save checkpoint" => {
                            let result = checkpoint_store.save(&visit, roster.as_mut().unwrap(), &mut arena, &scene.world, &campaign);
                            if crate::ui_probe::enabled() { crate::ui_probe::note("checkpoint-save", serde_json::json!({
                                "tick":roster.as_ref().unwrap().tick,"error":result.as_ref().err().map(|e|e.to_string())})); }
                            if result.is_ok() { checkpoint_pending = false; }
                            checkpoint_message = Some((now, match result {
                                Ok(()) => "Co-op checkpoint saved for both players.".into(),
                                Err(error) => format!("Checkpoint was not saved: {error}"),
                            }));
                        },
                        _ => break,
                    }
                }
                if focused { ui.cursor(); }
                super::menu::capture();
            } else if chapters_open {
                chapters.draw(&ui, &choices, &maps, chapter_difficulty, focused, controls.using_pad,
                    &crate::chapters::Labels::default());
                if focused { ui.cursor(); }
                crate::ui_probe::screen("coop-chapters", &[], "");
            } else if console.open {
                console.draw(&ui);
                crate::ui_probe::screen("coop-console", &[], "");
            } else {
                crate::ui_probe::screen("coop-play", &[], "");
            }
            if focused && is_key_pressed(KeyCode::F12) {
                crate::viewer::save_capture(&crate::preferences::path("coop-screenshot.png"))?;
            }
            rendered += u64::from(playing);
            if launch.drive && !captured_midpoint && started_playing.is_some_and(|start| start.elapsed() >= Duration::from_secs(6)) {
                crate::viewer::save_capture(&crate::preferences::path("coop-midpoint.png"))?;
                captured_midpoint = true;
            }
            if launch.death_check.is_some() {
                if let Some(frame) = &display {
                    let milestones = [
                        frame.players.iter().any(|p| !p.stats.alive()),
                        frame.checkpoint_generation == 0 && frame.players.iter().any(|p| p.spawned_at > 0 && p.stats.alive()),
                        frame.checkpoint_generation > 0,
                    ];
                    for (i, name) in ["spectating", "respawn", "checkpoint"].into_iter().enumerate() {
                        if milestones[i] && !captured_recovery[i] {
                            crate::viewer::save_capture(&crate::preferences::path(&format!("coop-{name}.png")))?;
                            captured_recovery[i] = true;
                        }
                    }
                    if milestones[2] { restored_at.get_or_insert(now); }
                }
                if restored_at.is_some_and(|at: Instant| at.elapsed() >= Duration::from_secs(if launch.host { 3 } else { 2 })) {
                    crate::viewer::save_capture(&crate::preferences::path("coop-screenshot.png"))?;
                    break;
                }
                anyhow::ensure!(!started_playing.is_some_and(|start| start.elapsed() > Duration::from_secs(90)),
                    "The staged death/respawn/wipe scenario did not complete.");
            }
            if launch.scene_check.is_some() {
                if !captured_scene && arena.stage.owner.is_some() && started_playing.is_some_and(|at| at.elapsed().as_secs() >= 25) {
                    crate::viewer::save_capture(&crate::preferences::path("coop-scene.png"))?;
                    captured_scene = true;
                }
                if !arena.stage.busy() && display.as_ref().is_some_and(|f| f.players.iter().all(|b| b.warped_at > 0)) {
                    reunited_at.get_or_insert(now);
                }
                if reunited_at.is_some_and(|at: Instant| at.elapsed() >= Duration::from_secs(if launch.host { 4 } else { 3 })) {
                    crate::viewer::save_capture(&crate::preferences::path("coop-screenshot.png"))?;
                    break;
                }
                anyhow::ensure!(!started_playing.is_some_and(|at| at.elapsed().as_secs() > 180), "Co-op scene did not reunite both players");
            }
            if launch.travel_check {
                if visit.epoch == 3 && playing { travel_returned_at.get_or_insert(now); }
                if travel_returned_at.is_some_and(|at: Instant| at.elapsed() >= Duration::from_secs(5)) && !travel_captured {
                    crate::viewer::save_capture(&crate::preferences::path("coop-screenshot.png"))?;
                    travel_captured = true;
                }
                if travel_returned_at.is_some_and(|at: Instant| at.elapsed() >= Duration::from_secs(if launch.host { 10 } else { 8 })) {
                    break;
                }
                anyhow::ensure!(!started_playing.is_some_and(|at| at.elapsed().as_secs() > 180), "Co-op travel fixture did not finish");
            }
            if launch.checkpoint_check && !launch.travel_check {
                if started_playing.is_some_and(|at| at.elapsed() >= Duration::from_secs(5)) && !checkpoint_captured {
                    crate::viewer::save_capture(&crate::preferences::path("coop-screenshot.png"))?;
                    checkpoint_captured = true;
                }
                if started_playing.is_some_and(|at| at.elapsed() >= Duration::from_secs(if launch.host { 10 } else { 8 })) {
                    anyhow::ensure!(!launch.host || fixture_checkpoint_wiped, "Checkpoint fixture did not verify a team wipe");
                    break;
                }
            }
            if let Some(driver) = &mut route_driver { if driver.capture_and_finish(now)? { break; } }
            if launch.drive && launch.route_driver.is_none() && !launch.travel_check && !launch.checkpoint_check && launch.death_check.is_none() && launch.scene_check.is_none() && started_playing.is_some_and(|start| start.elapsed()
                >= Duration::from_secs(if launch.host { 12 } else { 10 }
                    + if visit.combat_preview { 4 } else { 0 })) {
                crate::viewer::save_capture(&crate::preferences::path("coop-screenshot.png"))?;
                break;
            }
            if options.frames.is_some_and(|limit| rendered >= limit) {
                if let Some(path) = &options.capture { crate::viewer::save_capture(path)?; }
                break;
            }
            previous_work = [guest_poll_ms, replicate_ms, simulation_ms, send_ms, draw_ms, now.elapsed().as_secs_f32()*1000.,
                draw_phases[0],draw_phases[1],draw_phases[2],draw_phases[3],draw_phases[4],draw_phases[5]];
            next_frame().await;
        }
        look.release();
        Ok(())
    }.await;
    if let Some(hosting) = hosting {
        hosting.cancel();
        let started = Instant::now();
        while !matches!(hosting.status(), Status::Stopped | Status::Failed(_)) && started.elapsed() < Duration::from_secs(5) {
            next_frame().await;
        }
        if let Some(path) = &launch.invite_file {
            let _ = std::fs::remove_file(path);
        }
    }
    let _ = std::fs::remove_file(crate::preferences::path("coop-live.json"));
    show_mouse(true);
    let audio_result = audio.finish();
    result.and(audio_result)?;
    if completed {
        println!("Co-op campaign complete; playing ending and credits");
        // A completed world is never saved over the last playable checkpoint.
        // Host/Resume therefore replays the final encounter, as in single player.
        let prefs = crate::preferences::Preferences::load();
        let mut input = crate::input::Input::default();
        input.suppress();
        let mut film_audio = crate::audio::Audio::new(options.no_audio, None);
        let film = crate::movie::play(assets, &mut film_audio, &ui, &mut input, &prefs, "ending").await;
        if let Err(error) = &film { eprintln!("Ending film failed: {error:#}"); }
        film_audio.finish()?;
        if !matches!(film, Ok(false)) {
            let mut menu = crate::menu::Menu::load(assets)?;
            menu.completion_credits(&prefs, &mut input).await;
        }
    }
    Ok(completed)
}
