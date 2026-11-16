# Looking Glass for PS5 (homebrew)

An experimental port of [Looking Glass](https://github.com/looking-glass-project/LookingGlass), the
Rust re-remaster of American McGee's Alice, to a PS5 that runs homebrew. **Status: builds; not yet
run on a console.** Expect the first runs to need fixes.

No game data, Sony software or keys are included. You need your own copy of the game's data.

## How it works

| Part | Where | What |
| --- | --- | --- |
| Renderer | `vendor/miniquad/src/graphics/vulkan/` | A Vulkan backend for miniquad (the engine's graphics library), replacing OpenGL. The game's GLSL ES shaders are translated to SPIR-V at run time with [naga](https://github.com/gfx-rs/wgpu/tree/trunk/naga). On Linux, `MINIQUAD_BACKEND=vulkan` selects it; there it matches the OpenGL output (97–99% of pixels bit-identical on the maps tested). |
| PS5 platform | `vendor/miniquad/src/native/ps5.rs` | Vulkan from the statically linked RADV, presentation through `VK_KHR_display` (renders at 1920x1080, scaled to the display), DualSense polling. Console services come from [PS5_VulkanTemplate](https://github.com/mihawk-99/PS5_VulkanTemplate)'s `platform.c` (MIT), in `native/ps5c/`. |
| Input | `src/input.rs` | The DualSense translated to the engine's XInput layout, so every binding and menu works unchanged. |
| Audio | `vendor/cpal/src/host/null/` | On the PS5, cpal's fallback host plays through the console's 48 kHz output; rodio is unchanged. |
| Title | `ps5/build.sh`, `ps5/link-title.sh` | Cargo uses `link-title.sh` as its linker: PS5_Vulkan's RADV link recipe, a check that refuses imports that would be null at run time, then the signed `eboot.bin` and the title folder. |
| std glue | `vendor/ps5-std-rt/` | From [neokuze/ps5-rust](https://github.com/neokuze/ps5-rust), GPL-3.0-or-later, one change (see its `PS5_PORT.md`). |

Smaller changes to the engine: the few direct OpenGL calls go through backend-neutral miniquad hooks
(`set_depth_write_override`, `texture_grab_screen`), co-op TLS uses rustls instead of OpenSSL, and on
the console the game starts in its title folder (`src/main.rs`).

## Building

On Arch Linux (native, or WSL2 on Windows: `wsl --install archlinux`):

```bash
bash ps5/tools/setup-wsl-stack.sh      # once: packages, Rust nightly, the PS5 stack and RADV (~10 min)
bash ps5/build.sh                      # the title: ps5/dist/PPSA99641/
```

The stack is mihawk-99's: [PS5_Vulkan](https://github.com/mihawk-99/PS5_Vulkan) (RADV for the PS5,
built from [PS5_Mesa](https://github.com/mihawk-99/PS5_Mesa)), PS5_VulkanTemplate and its payload SDK
fork [PS5_PayloadSDK](https://github.com/mihawk-99/PS5_PayloadSDK).

## Installing

Needs a PS5 that runs homebrew: an enabler such as etaHEN, ShadowMountPlus and an FTP server
(ftpsrv, port 2121).

1. Copy `ps5/dist/PPSA99641/` to the console as `/data/homebrew/PPSA99641/`.
2. Copy your game's `Alice1/bin/base` folder (the English 2011 data with `pak0.pk3`) into it, so it
   becomes `/data/homebrew/PPSA99641/Alice1/bin/base/`. Settings and saves go to `private/` there.
3. Launch **Looking Glass** from the home screen (let ShadowMountPlus rescan if the tile is missing).

The game's messages go to the console's klog (`nc <ps5-ip> 3232` with klogsrv running).

Controls: left stick move, right stick look, Cross jump/select, Circle back, OPTIONS menu, touch pad
click = Back.

## Licences

Looking Glass and its changes here are MIT (`LICENSE-MIT`). Vendored crates keep their own licences
(miniquad, macroquad, cpal: MIT/Apache-2.0; `vendor/ps5-std-rt`: GPL-3.0-or-later; `platform.c`: MIT).
**A built `eboot.bin` links GPL-3.0 code** (ps5-std-rt, the payload SDK's startup code, PS5_Vulkan's
RADV build), so a distributed build is GPL-3.0 and its complete source must be offered with it, as
for the other titles built on this stack. See `ps5/LICENSING.md`.

American McGee's Alice is an Electronic Arts property; see `LEGAL.md`. This port is not affiliated
with or endorsed by EA, the Looking Glass project or Sony Interactive Entertainment. Running homebrew
needs a modified console, which may void its warranty or breach the platform's terms of service.
