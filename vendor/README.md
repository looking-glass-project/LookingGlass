# Vendored crates (PS5 port)

Patched copies, used through `[patch.crates-io]` / path dependencies in `Cargo.toml`. Each keeps its
own licence file. Upstream examples, CI files and other unused files were removed.

| Crate | Upstream | Changes |
| --- | --- | --- |
| `miniquad` | 0.4.8 (crates.io) | Vulkan rendering backend (`src/graphics/vulkan/`, feature `vulkan`), a window-less Vulkan loop (`src/native/vulkan_headless.rs`), the PS5 platform (`src/native/ps5.rs`, `src/native/ps5c/`), backend-neutral `set_depth_write_override` / `texture_grab_screen`, `MINIQUAD_SHADER_DUMP` in the GL backend |
| `macroquad` | 0.4.14 (crates.io) | `grab_screen` through miniquad; the Linux-only glFlush/glFinish skipped on Vulkan |
| `cpal` | 0.15.3 (crates.io) | FreeBSD/PS5 no longer selects ALSA; the null host plays through the PS5's audio output there |
| `ps5-std-rt` | neokuze/ps5-rust `19a6ac0`, GPL-3.0-or-later | `PS5_STD_RT_TITLE=1` skips the bundled libc copy (`PS5_PORT.md`) |
