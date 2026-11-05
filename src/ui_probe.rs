//! Passive, opt-in UI receipts for native input automation in isolated profiles.
//! No commands are accepted and no gameplay state is changed.
use macroquad::prelude::*;
use serde_json::{json, Value};
use std::{cell::RefCell, fs::OpenOptions, io::Write, sync::OnceLock};

pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("LOOKING_GLASS_UI_PROBE").as_deref() == Ok("1"))
}
thread_local! {
    static PREVIOUS: RefCell<Value> = const { RefCell::new(Value::Null) };
    static NOTES: RefCell<std::collections::BTreeMap<String, Value>> = const { RefCell::new(std::collections::BTreeMap::new()) };
}
pub fn note(name: &str, value: Value) {
    if !enabled() { return; }
    NOTES.with(|notes| {
        let mut notes = notes.borrow_mut();
        if notes.get(name) == Some(&value) { return; }
        let record = json!({"event":name,"value":value,"pid":std::process::id(),
            "build":env!("LOOKING_GLASS_BUILD")});
        if let Ok(mut file) = OpenOptions::new().create(true).append(true)
            .open(crate::preferences::path("ui-events.jsonl")) {
            if writeln!(file, "{record}").is_ok() { notes.insert(name.into(), value); }
        }
    });
}
pub fn screen(name: &str, buttons: &[(String, Rect)], message: &str) {
    if !enabled() { return; }
    let canvas = crate::ui::Canvas::new(screen_width(), screen_height());
    let data = json!({"screen":name,"message":message,"pid":std::process::id(),
        "build":env!("LOOKING_GLASS_BUILD"),"buttons":buttons.iter().map(|(label, rect)| {
            let r=canvas.rect(*rect); json!({"label":label,"rect":[r.x,r.y,r.w,r.h]})
        }).collect::<Vec<_>>()});
    PREVIOUS.with(|previous| {
        if *previous.borrow() == data { return; }
        if let Ok(bytes) = serde_json::to_vec(&data) {
            if crate::save::atomic_write(&crate::preferences::path("ui-live.json"), &bytes).is_ok() {
                *previous.borrow_mut() = data.clone();
                note("screen", data);
            }
        }
    });
}
pub fn solo(name: &str, game: &crate::save::Game) {
    if !enabled() { return; }
    use sha2::{Digest, Sha256};
    if let Ok(bytes) = serde_json::to_vec(game) {
        note(name, json!({"sha256":format!("{:x}",Sha256::digest(bytes)),
            "time":get_time(),"visit":game.current,"feet":game.player.feet,"health":game.stats.sanity()}));
    }
}
