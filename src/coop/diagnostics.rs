//! Optional diagnostics never wait for a disk writer on the render thread.
use std::{path::PathBuf, sync::mpsc::{self, SyncSender}, thread};

/// Call on the window thread after its OpenGL context has been initialized.
pub fn renderer() -> String {
    // GL_RENDERER describes the active context, unlike enumerating installed GPUs.
    let name = unsafe { macroquad::miniquad::gl::glGetString(0x1F01) };
    if name.is_null() { return "Unknown renderer".into(); }
    unsafe { std::ffi::CStr::from_ptr(name.cast()) }.to_string_lossy().into_owned()
}

pub struct Writer(SyncSender<Vec<u8>>);
impl Writer {
    pub fn new(path: PathBuf) -> Self {
        let (send, receive) = mpsc::sync_channel::<Vec<u8>>(1);
        thread::spawn(move || {
            if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
            while let Ok(bytes) = receive.recv() { let _ = std::fs::write(&path, bytes); }
        });
        Self(send)
    }
    pub fn write(&self, bytes: Vec<u8>) { let _ = self.0.try_send(bytes); }
}

#[derive(Default)]
pub struct Frames {
    values: Vec<f32>,
    seconds: f32,
}
impl Frames {
    pub fn sample(&mut self, seconds: f32) -> Option<serde_json::Value> {
        if !seconds.is_finite() || seconds <= 0. { return None; }
        self.values.push(seconds * 1000.);
        self.seconds += seconds;
        if self.seconds < 1. { return None; }
        self.values.sort_by(f32::total_cmp);
        let n = self.values.len();
        let result = serde_json::json!({"frames":n,"fps":n as f32/self.seconds,
            "median_ms":self.values[n/2],"p95_ms":self.values[(n*95/100).min(n-1)],
            "max_ms":self.values[n-1],"over_50_ms":self.values.iter().filter(|&&v|v>50.).count()});
        self.values.clear(); self.seconds = 0.;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn performance_window_keeps_the_stall_between_report_frames() {
        let mut frames = Frames::default();
        for _ in 0..30 { assert!(frames.sample(0.016).is_none()); }
        assert!(frames.sample(0.2).is_none());
        let report = (0..40).find_map(|_| frames.sample(0.016)).unwrap();
        assert_eq!(report["over_50_ms"],1);
        assert_eq!(report["max_ms"],200.);
    }
    #[test]
    fn diagnostic_backpressure_drops_reports_without_waiting() {
        let (sender, _receiver) = mpsc::sync_channel(1);
        let writer = Writer(sender);
        for _ in 0..1000 { writer.write(vec![0; 32]); }
    }
}
