//! Co-op entry and personal menus. Connected-session controls are drawn one
//! frame at a time so opening a menu never stops the partner's simulation.
use super::{invite::Invite, preview::Launch, protocol::Visit};
use crate::{
    assets::Assets,
    input::Input,
    menu::Menu,
    preferences::Preferences,
    ui::{Canvas, Ui, PAPER_TEXT},
    Options,
};
use macroquad::prelude::*;

#[derive(Default)]
struct LinkInput {
    text: String,
    selected: bool,
}
impl LinkInput {
    fn replace(&mut self, text: &str) {
        // Bound clipboard input before storing it. Invite::parse is still the
        // authority; do not truncate an overlong link into a different invite.
        self.text = if text.len() <= 2048 {
            text.trim().to_owned()
        } else {
            String::new()
        };
        self.selected = false;
    }
    fn character(&mut self, c: char) {
        if !c.is_ascii_graphic() {
            return;
        }
        if self.selected {
            self.text.clear();
            self.selected = false;
        }
        if self.text.len() < 2048 {
            self.text.push(c);
        }
    }
    fn backspace(&mut self) {
        if self.selected {
            self.text.clear();
        } else {
            self.text.pop();
        }
        self.selected = false;
    }
    fn queued(&mut self, newest_first: Vec<char>) {
        // Macroquad 0.4.14 pops its character queue from the end. Preserve
        // input order when Windows delivers several characters in one frame.
        for c in newest_first.into_iter().rev() { self.character(c); }
    }
}

/// Hosts or resumes co-op while retaining the caller's suspended single-player
/// game. Co-op checkpoints have their own files and never replace its saves.
pub async fn open(assets: &mut Assets, options: &Options, backdrop: &mut Menu, visit: Visit, single_player: Option<crate::save::Game>) {
    let ui = match Ui::load(assets) {
        Ok(ui) => ui,
        Err(error) => {
            backdrop.message = error.to_string();
            return;
        }
    };
    let prefs = Preferences::load();
    let mut input = Input::default();
    let mut link = LinkInput::default();
    let mut joining = false;
    let mut hosting = false;
    let mut choosing = false;
    let maps = assets.maps();
    let choices = crate::campaign::level_choices(&maps);
    let mut chapters = crate::chapters::Chapters::default();
    let mut difficulty = visit.difficulty;
    let mut selected = 0;
    let mut message = String::new();
    show_mouse(false);
    // The activating main-menu key/click must not also select Host.
    next_frame().await;
    loop {
        if is_quit_requested() {
            break;
        }
        let focused = crate::look::window_focused();
        input.update(&prefs, focused);
        if focused && input.ui(KeyCode::Escape) {
            if choosing {
                choosing = false;
            } else if joining || hosting {
                joining = false;
                hosting = false;
                selected = 0;
                message.clear();
            } else {
                break;
            }
        }
        if joining && focused {
            let control = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
            if control && is_key_pressed(KeyCode::A) {
                link.selected = true;
            }
            if control && is_key_pressed(KeyCode::V) {
                paste(&mut link, &mut message);
            } else if is_key_pressed(KeyCode::Backspace)
                || is_key_pressed(KeyCode::Delete) && link.selected
            {
                link.backspace();
            }
            let queued = std::iter::from_fn(get_char_pressed).collect();
            if !control { link.queued(queued); }
        } else {
            while get_char_pressed().is_some() {}
        }
        backdrop.backdrop();
        if crate::ui_probe::enabled() { crate::ui_probe::note("coop-message", serde_json::json!(message)); }
        if joining && crate::ui_probe::enabled() {
            use sha2::{Digest, Sha256};
            crate::ui_probe::note("invite-input", serde_json::json!({"length":link.text.len(),
                "sha256":format!("{:x}",Sha256::digest(link.text.as_bytes()))}));
        }
        panel(&ui, if joining { "Join co-op" } else if hosting { "Host co-op" } else { "Co-op" });
        let mut launch = None;
        if choosing {
            let action = chapters.update(&input, choices.len(), &mut difficulty);
            chapters.draw(&ui, &choices, &maps, difficulty, focused, input.using_pad,
                &crate::chapters::Labels::default());
            crate::ui_probe::screen("coop-chapters", &[], "");
            if action == Some(crate::chapters::Hit::Back) {
                choosing = false;
            } else if !choices.is_empty() && (action == Some(crate::chapters::Hit::Begin)
                || focused && input.ui(KeyCode::Enter)) {
                let choice = &choices[chapters.selected];
                let start = Visit { epoch: 1, map: maps[choice.map].clone(),
                    entry: choice.entry.map(str::to_owned), difficulty, combat_preview: false };
                launch = Some(host_launch(&start, false));
                choosing = false;
            }
        } else if joining {
            text(
                &ui,
                "Paste the complete invite link from your friend.",
                120.,
                16.,
            );
            let canvas = Canvas::new(screen_width(), screen_height());
            let field = Rect::new(106., 155., 428., 76.);
            let r = canvas.rect(field);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.03, 0.02, 0.02, 0.9));
            draw_rectangle_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                2.,
                if link.selected { YELLOW } else { PAPER_TEXT },
            );
            let shown: String = link.text.chars().take(156).collect();
            let shown = if shown.is_empty() {
                "Invite link".to_owned()
            } else {
                shown
            };
            // URLs have no spaces, so wrap by character without changing the input.
            for (row, chunk) in shown.as_bytes().chunks(52).take(3).enumerate() {
                ui.courier.draw_left(
                    canvas,
                    &String::from_utf8_lossy(chunk),
                    Rect::new(114., 164. + row as f32 * 20., 412., 20.),
                    14.,
                    WHITE,
                );
            }
            text(
                &ui,
                "Ctrl+V: paste   Ctrl+A: replace   Enter: join",
                240.,
                13.,
            );
            if let Some(choice) = buttons(
                &ui,
                &input,
                &mut selected,
                &["Join", "Paste link", "Back"],
                280.,
            ) {
                match choice {
                    0 => match Invite::parse(&link.text) {
                        Ok(invite) => {
                            launch = Some(Launch {
                                host: false,
                                internet: true,
                                invite_file: None,
                                invite: Some(invite),
                                start: None,
                                drive: false,
                                death_check: None,
                                scene_check: None,
                                travel_check: false,
                                resume: false,
                                single_player: None,
                                checkpoint_check: false,
                                route_driver: None,
                                conditions: Default::default(),
                            })
                        }
                        Err(error) => message = error.to_string(),
                    },
                    1 => {
                        paste(&mut link, &mut message);
                        selected = 0;
                    }
                    _ => {
                        joining = false;
                        selected = 0;
                        message.clear();
                    }
                }
            }
        } else if hosting {
            text(&ui, "Choose where to continue together.", 120., 17.);
            text(&ui, if single_player.is_some() {
                "Host this game keeps your current progress."
            } else { "To host a solo save, load it first, then open Co-op from ESC." }, 148., 14.);
            text(&ui, "Each new session creates a fresh invite link.", 192., 14.);
            let host_options = host_choices(single_player.is_some(), super::checkpoint::Store::available());
            let labels = host_options.iter().map(|c| c.label()).collect::<Vec<_>>();
            if let Some(choice) = buttons(&ui, &input, &mut selected, &labels, 224.) {
                match host_options[choice] {
                    HostChoice::Back => { hosting = false; selected = 0; message.clear(); }
                    HostChoice::Current => {
                        let mut current = host_launch(&visit, false);
                        current.single_player = single_player.clone();
                        launch = Some(current);
                    }
                    HostChoice::Chapter => {
                        choosing = true;
                        let selected = maps.iter().position(|m| m == &visit.map).unwrap_or(0);
                        chapters.open(crate::campaign::choice_position(&choices, selected, visit.entry.as_deref()), choices.len());
                    }
                    HostChoice::Saved => launch = Some(host_launch(&visit, true)),
                }
            }
        } else {
            text(&ui, "Play together as blue and red Alice.", 120., 18.);
            text(
                &ui,
                "Host creates an invite. Join uses your friend's link.",
                152.,
                15.,
            );
            text(&ui, "The host keeps the shared checkpoint for both players.", 182., 14.);
            if let Some(choice) =
                buttons(&ui, &input, &mut selected, &["Host", "Join", "Back"], 224.)
            {
                match choice {
                    0 => {
                        hosting = true;
                        selected = 0;
                        message.clear();
                    }
                    1 => {
                        joining = true;
                        selected = 0;
                        message.clear();
                    }
                    _ => break,
                }
            }
            text(
                &ui,
                "Both players need the same co-op build.",
                378.,
                12.,
            );
        }
        if !message.is_empty() {
            let canvas = Canvas::new(screen_width(), screen_height());
            ui.dialog(canvas.rect(Rect::new(86., 410., 470., 66.)));
            text(&ui, &message, 418., 13.);
        }
        if focused {
            ui.cursor();
        }
        capture();
        if let Some(launch) = launch {
            input.suppress();
            next_frame().await;
            let result = super::preview::run(assets, options, &launch).await;
            if crate::ui_probe::enabled() { crate::ui_probe::note("coop-result", serde_json::json!({
                "time":get_time(),"host":launch.host,"error":result.as_ref().err().map(|e|e.to_string())})); }
            message = match result {
                Ok(true) => return,
                Ok(false) => "Co-op closed. You can host or join again.".into(),
                Err(error) => {
                    eprintln!("Co-op session ended: {error}");
                    format!("Could not continue co-op: {error}")
                }
            };
            // Macroquad's text queue survives gameplay frames. Movement keys
            // typed there must not be appended to the retained invite on return.
            while get_char_pressed().is_some() {}
            set_default_camera();
            set_cursor_grab(false);
            show_mouse(false);
            input.suppress();
            selected = 0;
        }
        next_frame().await;
    }
    show_mouse(true);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostChoice { Current, Saved, Chapter, Back }
impl HostChoice {
    fn label(self) -> &'static str {
        match self {
            Self::Current => "Host this game", Self::Saved => "Continue co-op",
            Self::Chapter => "Start chapter", Self::Back => "Back",
        }
    }
}
fn host_choices(current: bool, saved: bool) -> Vec<HostChoice> {
    let mut choices = Vec::new();
    if current { choices.push(HostChoice::Current); }
    if saved { choices.push(HostChoice::Saved); }
    choices.extend([HostChoice::Chapter, HostChoice::Back]);
    choices
}

fn host_launch(visit: &Visit, resume: bool) -> Launch {
    // Explicit isolated UI fixture only; ordinary menu hosting stays online.
    let local_probe = crate::ui_probe::enabled()
        && std::env::var("LOOKING_GLASS_UI_PROBE_LOCAL").as_deref() == Ok("1");
    Launch {
        host: true, internet: !local_probe, invite_file: None, invite: None, start: Some(visit.clone()),
        drive: false, death_check: None, scene_check: None, travel_check: false, resume,
        single_player: None, checkpoint_check: false, route_driver: None, conditions: Default::default(),
    }
}

fn paste(link: &mut LinkInput, message: &mut String) {
    match miniquad::window::clipboard_get() {
        Some(value) if value.len() <= 2048 => {
            link.replace(&value);
            message.clear();
        }
        Some(_) => *message = "That invite link is too long.".into(),
        None => *message = "Copy your friend's invite link first.".into(),
    }
}

pub fn panel(ui: &Ui, title: &str) {
    let canvas = Canvas::new(screen_width(), screen_height());
    let r = canvas.rect(Rect::new(76., 56., 488., 358.));
    ui.dialog(r);
    ui.font
        .draw(canvas, title, Rect::new(100., 72., 440., 36.), 30., WHITE);
}

pub fn text(ui: &Ui, value: &str, y: f32, size: f32) {
    let canvas = Canvas::new(screen_width(), screen_height());
    let rows = crate::story::wrap(value, 424., |s| ui.body.width(s, size));
    for (i, row) in rows.iter().take(3).enumerate() {
        ui.body.draw(
            canvas,
            row,
            Rect::new(108., y + i as f32 * (size + 5.), 424., size + 5.),
            size,
            PAPER_TEXT,
        );
    }
}

pub fn buttons(
    ui: &Ui,
    input: &Input,
    selected: &mut usize,
    labels: &[&str],
    y: f32,
) -> Option<usize> {
    if crate::ui_probe::enabled() {
        crate::ui_probe::screen("coop-menu", &labels.iter().enumerate().map(|(i,label)|
            ((*label).to_owned(), Rect::new(220., y + i as f32 * 46., 200., 36.)))
            .collect::<Vec<_>>(), "");
    }
    let focused = crate::look::window_focused();
    let canvas = Canvas::new(screen_width(), screen_height());
    let pointer = canvas.pointer(Vec2::from(mouse_position()));
    if focused && (input.ui(KeyCode::Down) || input.ui(KeyCode::Tab)) {
        *selected = (*selected + 1) % labels.len();
    }
    if focused && input.ui(KeyCode::Up) {
        *selected = (*selected + labels.len() - 1) % labels.len();
    }
    let mut click = None;
    for (index, label) in labels.iter().enumerate() {
        let rect = Rect::new(220., y + index as f32 * 46., 200., 36.);
        let hover = focused && rect.contains(pointer);
        ui.plaque(canvas.rect(rect), hover || index == *selected);
        ui.font.draw(canvas, label, rect, 24., WHITE);
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            *selected = index;
            click = Some(index);
        }
    }
    if focused && input.ui(KeyCode::Enter) {
        click = Some(*selected);
    }
    click
}

pub fn capture() {
    if crate::look::window_focused() && is_key_pressed(KeyCode::F12) {
        let _ = crate::viewer::save_capture(&crate::preferences::path("coop-menu.png"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn batched_characters_keep_event_order_and_replace_selected_invite_once() {
        let mut field = LinkInput::default();
        field.replace("old invite");
        field.selected = true;
        field.queued("https://host.test/#token".chars().rev().collect());
        assert_eq!(field.text, "https://host.test/#token");
        field.queued(vec!['x']);
        assert_eq!(field.text, "https://host.test/#tokenx");
    }
    #[test]
    fn host_choices_keep_current_game_and_saved_coop_distinct() {
        assert_eq!(host_choices(true, true), vec![HostChoice::Current, HostChoice::Saved, HostChoice::Chapter, HostChoice::Back]);
        assert_eq!(host_choices(false, true), vec![HostChoice::Saved, HostChoice::Chapter, HostChoice::Back]);
        assert_eq!(host_choices(true, false), vec![HostChoice::Current, HostChoice::Chapter, HostChoice::Back]);
        assert_eq!(host_choices(false, false), vec![HostChoice::Chapter, HostChoice::Back]);
    }
    #[test]
    fn paste_replace_and_edit_keep_the_full_invite_and_bound_clipboard_input() {
        let invite = Invite::new("https://example.trycloudflare.com")
            .unwrap()
            .link();
        let mut field = LinkInput::default();
        field.replace(&format!("  {invite}\n"));
        assert_eq!(Invite::parse(&field.text).unwrap().link(), invite);
        field.selected = true;
        field.character('x');
        assert_eq!(field.text, "x");
        field.backspace();
        assert!(field.text.is_empty());
        field.replace(&"x".repeat(2049));
        assert!(field.text.is_empty());
        field.replace(&"x".repeat(2048));
        field.character('y');
        assert_eq!(field.text.len(), 2048);
        field.selected = true;
        field.backspace();
        assert!(field.text.is_empty());
    }
}
