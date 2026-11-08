//! Linux gamepads through evdev. Steam Input (and the kernel xpad driver)
//! present an Xbox 360 layout, so events map straight onto XInput's state.
use super::RawPad;
use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;
const EV_ABS: u16 = 3;
const SYN_DROPPED: u16 = 3;
const BTN_SOUTH: usize = 0x130;
const AXES: [u16; 8] = [0, 1, 2, 3, 4, 5, 0x10, 0x11]; // X Y Z RX RY RZ HAT0X HAT0Y
const RESCAN: Duration = Duration::from_secs(1);

#[repr(C)]
struct InputEvent {
    _time: [libc::c_long; 2],
    kind: u16,
    code: u16,
    value: i32,
}
#[repr(C)]
#[derive(Default)]
struct AbsInfo {
    value: i32,
    min: i32,
    max: i32,
    _fuzz: i32,
    _flat: i32,
    _resolution: i32,
}

// _IOC(_IOC_READ, 'E', nr, size)
const fn ior(nr: u64, size: usize) -> u64 {
    (2 << 30) | ((size as u64) << 16) | (0x45 << 8) | nr
}

struct Device {
    path: PathBuf,
    file: File,
    ranges: [(i32, i32); 8],
    abs: [i32; 8],
    buttons: u32,
}

impl Device {
    fn open(path: PathBuf) -> Option<Self> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path)
            .ok()?;
        let mut keys = [0u8; 0x300 / 8];
        // EVIOCGBIT(EV_KEY): fills the key bitmap; buffer length is passed in the request.
        let ok = unsafe {
            libc::ioctl(file.as_raw_fd(), ior(0x20 + EV_KEY as u64, keys.len()) as _, keys.as_mut_ptr())
        } >= 0;
        if !ok || keys[BTN_SOUTH / 8] & (1 << (BTN_SOUTH % 8)) == 0 {
            return None; // Keyboards, mice and sound jacks lack the gamepad A button.
        }
        let mut dev = Device { path, file, ranges: [(0, 0); 8], abs: [0; 8], buttons: 0 };
        dev.sync_axes();
        Some(dev)
    }
    fn sync_axes(&mut self) {
        for (i, code) in AXES.iter().enumerate() {
            let mut info = AbsInfo::default();
            // EVIOCGABS(code): writes one input_absinfo; absent axes fail and stay zero.
            if unsafe {
                libc::ioctl(self.file.as_raw_fd(), ior(0x40 + *code as u64, size_of::<AbsInfo>()) as _, &mut info)
            } >= 0
            {
                self.ranges[i] = (info.min, info.max);
                self.abs[i] = info.value;
            }
        }
    }
    /// Drains queued events; false once the device is gone.
    fn pump(&mut self) -> bool {
        let mut buf = [0u8; size_of::<InputEvent>() * 64];
        loop {
            match self.file.read(&mut buf) {
                Ok(0) => return false,
                Ok(n) => {
                    for chunk in buf[..n].chunks_exact(size_of::<InputEvent>()) {
                        // Kernel writes whole input_event records; unaligned read copies one out.
                        let e = unsafe { std::ptr::read_unaligned(chunk.as_ptr() as *const InputEvent) };
                        if e.kind == EV_SYN && e.code == SYN_DROPPED {
                            self.sync_axes();
                        } else {
                            apply(&mut self.abs, &mut self.buttons, e.kind, e.code, e.value);
                        }
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return true,
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(_) => return false,
            }
        }
    }
    fn state(&self) -> RawPad {
        to_raw(&self.abs, &self.ranges, self.buttons)
    }
}

fn apply(abs: &mut [i32; 8], buttons: &mut u32, kind: u16, code: u16, value: i32) {
    if kind == EV_ABS {
        if let Some(i) = AXES.iter().position(|a| *a == code) {
            abs[i] = value;
        }
    } else if kind == EV_KEY {
        let mask = match code {
            0x130 => 0x1000,         // A
            0x131 => 0x2000,         // B
            0x133 => 0x4000,         // X (BTN_X == BTN_NORTH in xpad's layout)
            0x134 => 0x8000,         // Y
            0x136 => 0x100,          // LB
            0x137 => 0x200,          // RB
            0x13a => 0x20,           // Back
            0x13b => 0x10,           // Start
            0x13d => 0x40,           // L Stick
            0x13e => 0x80,           // R Stick
            0x220 => 1,              // D-pad as buttons (some pads)
            0x221 => 2,
            0x222 => 4,
            0x223 => 8,
            _ => 0,
        };
        if value != 0 {
            *buttons |= mask;
        } else {
            *buttons &= !mask;
        }
    }
}

fn to_raw(abs: &[i32; 8], ranges: &[(i32, i32); 8], buttons: u32) -> RawPad {
    // Map an axis to -1..1 (or 0..1 for triggers) using the device's own range.
    let norm = |i: usize| {
        let (lo, hi) = ranges[i];
        if hi <= lo {
            return 0.;
        }
        ((abs[i] - lo) as f32 / (hi - lo) as f32).clamp(0., 1.)
    };
    let stick = |i: usize, flip: bool| {
        let v = norm(i) * 2. - 1.;
        ((if flip { -v } else { v }) * 32767.) as i16
    };
    let mut dpad = 0u16;
    match abs[7].signum() { -1 => dpad |= 1, 1 => dpad |= 2, _ => {} }
    match abs[6].signum() { -1 => dpad |= 4, 1 => dpad |= 8, _ => {} }
    RawPad {
        buttons: buttons as u16 | dpad,
        lt: (norm(2) * 255.) as u8,
        rt: (norm(5) * 255.) as u8,
        lx: stick(0, false),
        ly: stick(1, true), // evdev Y grows downward, XInput upward
        rx: stick(3, false),
        ry: stick(4, true),
    }
}

thread_local! {
    static PADS: RefCell<(Vec<Device>, Option<Instant>)> = const { RefCell::new((Vec::new(), None)) };
}

pub fn read(index: u32) -> Option<RawPad> {
    PADS.with(|cell| {
        let (devs, scanned) = &mut *cell.borrow_mut();
        devs.retain_mut(Device::pump);
        let index = index as usize;
        if index >= devs.len() && scanned.is_none_or(|t| t.elapsed() >= RESCAN) {
            *scanned = Some(Instant::now());
            let mut nodes: Vec<PathBuf> = fs::read_dir("/dev/input")
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("event")))
                .filter(|p| !devs.iter().any(|d| &d.path == p))
                .collect();
            nodes.sort();
            devs.extend(nodes.into_iter().filter_map(Device::open));
        }
        devs.get(index).map(Device::state)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xbox_layout_maps_to_xinput_state() {
        let ranges = [(-32768, 32767), (-32768, 32767), (0, 255), (-32768, 32767), (-32768, 32767), (0, 255), (-1, 1), (-1, 1)];
        let (mut abs, mut buttons) = ([0, 0, 0, 0, 0, 0, 0, 0], 0);
        apply(&mut abs, &mut buttons, EV_KEY, 0x130, 1); // A
        apply(&mut abs, &mut buttons, EV_KEY, 0x13b, 1); // Start
        apply(&mut abs, &mut buttons, EV_ABS, 1, -32768); // left stick fully up
        apply(&mut abs, &mut buttons, EV_ABS, 5, 255); // RT
        apply(&mut abs, &mut buttons, EV_ABS, 0x11, -1); // hat up
        apply(&mut abs, &mut buttons, EV_ABS, 0x10, 1); // hat right
        let p = to_raw(&abs, &ranges, buttons);
        assert_eq!(p.buttons, 0x1000 | 0x10 | 1 | 8);
        assert_eq!((p.lt, p.rt), (0, 255));
        assert_eq!(p.ly, 32767);
        assert!(p.lx.abs() <= 1, "centred stick stays near zero: {}", p.lx);
        apply(&mut abs, &mut buttons, EV_KEY, 0x130, 0);
        assert_eq!(to_raw(&abs, &ranges, buttons).buttons & 0x1000, 0);
    }
    /// Hardware check: `cargo test live_pad -- --ignored --nocapture`, then press A
    /// and push the left stick up within 10 s.
    #[test]
    #[ignore]
    fn live_pad() {
        let end = Instant::now() + Duration::from_secs(10);
        let (mut seen_a, mut seen_up) = (false, false);
        while Instant::now() < end && !(seen_a && seen_up) {
            if let Some(p) = read(0) {
                seen_a |= p.buttons & 0x1000 != 0;
                seen_up |= p.ly > 16000;
            }
            std::thread::sleep(Duration::from_millis(16));
        }
        println!("pad0: A {seen_a}, left stick up {seen_up}");
        assert!(seen_a && seen_up);
    }
}
