//! Window-less platform loop for the Vulkan backend.
//!
//! Linux: `MINIQUAD_BACKEND=vulkan` runs the application on lavapipe (or any Vulkan driver)
//! without a window, at `conf.window_width x conf.window_height`; used to compare the
//! Vulkan backend against GL output. The PS5 platform reuses `run_with` and adds display
//! presentation (VK_KHR_display) and pad input.
use crate::{conf::Conf, native::NativeDisplayData, EventHandler};

struct NoClipboard;
impl crate::native::Clipboard for NoClipboard {
    fn get(&mut self) -> Option<String> { None }
    fn set(&mut self, _: &str) {}
}

pub fn run<F>(conf: &Conf, f: F)
where
    F: 'static + FnOnce() -> Box<dyn EventHandler>,
{
    run_with(conf, (conf.window_width, conf.window_height), f, |_| {})
}

/// Sets up the display data (screen size `size`), creates the handler and runs update/draw until quit.
/// `pump` runs once per frame before `update` (input on the PS5).
pub fn run_with<F, P>(conf: &Conf, size: (i32, i32), f: F, mut pump: P)
where
    F: 'static + FnOnce() -> Box<dyn EventHandler>,
    P: FnMut(&mut dyn EventHandler),
{
    let (tx, rx) = std::sync::mpsc::channel();
    crate::set_display(NativeDisplayData {
        high_dpi: conf.high_dpi,
        dpi_scale: 1.0,
        ..NativeDisplayData::new(size.0, size.1, tx, Box::new(NoClipboard))
    });
    let mut handler = f();
    loop {
        if crate::native_display().lock().unwrap().quit_ordered { break; }
        while let Ok(_request) = rx.try_recv() {}
        pump(&mut *handler);
        handler.update();
        handler.draw();
        let requested = crate::native_display().lock().unwrap().quit_requested;
        if requested {
            handler.quit_requested_event();
            let mut d = crate::native_display().lock().unwrap();
            if d.quit_requested { d.quit_ordered = true; }
        }
    }
}
