# PS5 port: development notes

The running log of the port (audit, decisions, results). Paths like `C:\Alice1\...` and `build/` refer to the
developer's working folder, not to this repository; the how-to is `ps5/README.md`.

# Alice (LookingGlass) -> PS5 port: audit and plan

Target: PS5, firmware 13.60, etaHEN. Base: LookingGlass v0.33.0-preview.4 (commit fe4f381, MIT).
Game data stays the user's own; nothing from EA is bundled or committed.

## Audit findings (source read 2026-11-16)

- ~254k lines of Rust, 2021 edition, MSRV 1.86. Very platform-agnostic.
- **Graphics: macroquad 0.4.14 -> miniquad 0.4.8, OpenGL ES 2 (GLSL `#version 100`).**
  Not a CPU renderer (earlier guess was wrong). 204 files use macroquad's prelude.
  Direct miniquad use is small: ~14 ShaderSource sites, 17 `get_internal_gl`, 7 raw `gl*` calls
  (glDepthMask, glClear, glFinish, glGetString), 4 RenderTarget uses, TextureWrap/mipmap modes.
  Shaders are built at runtime (placeholder substitution such as `// LIGHTS`, `// FOG`).
- **Audio: rodio 0.20.1 (wav + symphonia-mp3) over cpal.** Uses OutputStream (3 sites), Sink,
  Decoder, SamplesBuffer. cpal has no PS5 backend.
- **Input: macroquad events + XInput.** Only 6 `cfg(windows)` sites in the whole engine
  (input.rs x3, look.rs, coop/tunnel.rs, coop/checkpoint_check.rs). Non-Windows pad path is stubbed.
- **Networking: co-op only** (tungstenite + native-tls, Cloudflare tunnel helper). Not needed for a first port.
- **Files:** settings/saves in `private/` (env `LOOKING_GLASS_SETTINGS_DIR` overrides). Plain std::fs.
- `[features]` table does not exist, so co-op cannot be switched off without a small change.

## The central decision: how to get miniquad drawing on PS5

miniquad's platform layer (window, event loop, GL loader) is in `src/native/*.rs`; its
renderer is a 43-method `RenderingBackend` trait. Two ways in:

1. **GLES2 over Vulkan (keep miniquad's GL backend).** Add a `native/ps5.rs` platform and obtain a GLES2 context
   from Mesa zink or ANGLE on top of the PS5 Vulkan stack (RADV via VK_KHR_display).
   Least engine-side risk, but needs a GL-over-Vulkan build for the PS5 target; unproven here.
2. **Native Vulkan `RenderingBackend`.** Implement the 43 methods on Vulkan and translate the runtime GLSL ES shaders
   to SPIR-V (glslang built for PS5, or a GLSL-ES-to-SPIR-V path). More code, but nothing exotic in the stack,
   and the PS5_Quake3e port already proves the Vulkan/display/present path.

Spike both for a day each before committing; option 2 is my default.

## Status update (2026-11-16, later)

Toolchain built and working in the Claude VM (Rust 1.100 via conda-forge + `RUSTC_BOOTSTRAP=1`, LLVM 22.1.8, PS5 Payload SDK):
`hello_notify.elf` and `hello_std.elf` build to FreeBSD x86-64 ELFs (in `C:\Alice1\build\`). **Not yet run on the console.**

**LookingGlass type-checks for `x86_64-sie-ps5-std`** (`cargo check`, whole 254k-line engine) after two dependency changes
(patch: `build/lookingglass-ps5-check.patch`; patched tree: `C:\Alice1\LookingGlass-src`):
1. `cpal` (via rodio) pulled in ALSA. Vendored a copy with FreeBSD removed from its cfg lists so it falls back to cpal's `null` host.
   This is the seam for a real PS5 audio host (sceAudioOut) later: implement cpal's Host/Device/Stream there and rodio works unchanged.
2. `tungstenite` native-tls (OpenSSL) -> `rustls-tls-webpki-roots`; one match arm in `src/coop/transport.rs` renamed `NativeTls` -> `Rustls`.
   (ps5-rust has already shown rustls/ring working on the console.)
Cargo.lock was re-resolved by the check; review before committing to it.

miniquad 0.4.8 compiles for FreeBSD but `miniquad::start()` has no branch for it, so it silently does nothing. That function is
exactly where a `native::ps5::run(&conf, f)` platform module plugs in (window/event loop + GL or Vulkan backend).
So remaining engine-side work is concentrated in: (a) miniquad PS5 platform + renderer, (b) cpal PS5 audio host, (c) DualSense input
(replace the XInput path in `src/input.rs`), (d) file locations. Gameplay code needs no changes so far.

Open: build is only type-checked, not linked or run. Linking will need ps5-std-rt (kevent/stat shims) as a dependency.

## Status update 2 (2026-11-16, night)

**The engine links as a PS5 ELF.** `build/looking-glass-ps5-stub.elf` (opt-level 2, stripped, 20.9 MB). Changes on top of update 1
(`build/lookingglass-ps5.patch`, `build/miniquad-ps5.diff`, tree in `LookingGlass-src`):
- vendored miniquad 0.4.8 with a `target_os = "freebsd"` branch in `start()` calling `native::ps5::run`, currently a stub that
  prints "PS5 platform not implemented yet" and returns (no window, no renderer). So on the console this ELF should load, run
  std start-up and exit; it is a load test, not a game.
- `ps5-std-rt` added as a FreeBSD-only dependency (expects `../ps5-rust` beside the source tree).
- `src/ps5_shims.rs`: `_Unwind_FindEnclosingFunction` / `_Unwind_GetCFA` stubs (only std backtrace printing used them).
- `ps5/build.sh` builds for `x86_64-sie-ps5-std`.
- Every strong undefined symbol in the ELF (130) is exported by the SDK's stub libraries; the two that are not
  (`posix_spawn_file_actions_addchdir[_np]`) are weak, so std falls back at runtime.

**The game runs on Linux with the retail data.** Native Linux build, Xvfb + Mesa software GL, `--no-audio`:
loads skool1 (32k triangles, 402 batches, Alice's 131 bones, 6 NPCs) and renders correctly; frame saved as
`build/linux-headless-skool1.png`. Use this as the **golden reference image** for the PS5 renderer: render the same
map/frames on the Vulkan backend and compare.

**PS5_Vulkan findings (README + docs):** RADV (Mesa 26.2, PS5 winsys) reports Vulkan 1.4 and passed the full CTS
run on the console with 0 failures (1.57M pass); vkQuake, RetroArch cores and 15 Vulkan samples run at 4K/120.
Swapchain = VK_KHR_display on VideoOut. Direct-memory pool 12 GiB. This strongly favours **option 2 (native Vulkan
RenderingBackend)**; there is no GLES context on that stack. Build RADV with `tools/build-radv.sh release`
(needs `../PS5_Mesa`, still to be cloned).

VM limits observed: the optimized builds peak at ~3.6 GB RSS on a 3.9 GB VM, and a cold build of the main crate
takes ~2.5 min. Workable, but WSL2 is the right place for iteration and for the Mesa build.

## Status update 3 (2026-11-16, late night): the Vulkan renderer works

**Decision made: native Vulkan backend for miniquad** (option 2). It renders the real game identically to GL.

- `vendor/miniquad/src/graphics/vulkan/` (~1.5k lines, `ash` 0.38): implements all of miniquad's `RenderingBackend`.
  Enabled with the `vulkan` cargo feature; selected automatically on FreeBSD/PS5, or on Linux with
  `MINIQUAD_BACKEND=vulkan` (headless loop in `native/vulkan_headless.rs`, the skeleton of the PS5 platform).
- Shaders: the game's GLSL ES 1.00 is rewritten to Vulkan GLSL 4.50 and compiled to SPIR-V at runtime with **naga**
  (pure Rust, builds for the PS5). Only 10 distinct programs across skool1/fortress1/pandemonium; all pass naga
  validation and Khronos `spirv-val`. Gotchas found: naga has no combined image samplers (split into texture+sampler),
  and naga's SPIR-V writer flips Y by default (`ADJUST_COORDINATE_SPACE`, turned off).
- Conventions: all targets in GL memory layout (row 0 = bottom), flip only at present; GL CCW = Vulkan CW;
  z remapped to 0..1 in the vertex shader; one frame in flight; 96 MiB host ring for uniforms (std140 repack),
  stream buffers and uploads; push descriptors; transfers split the render pass to keep GL ordering.
- GL behaviours that had to be mirrored: `clear()` outside a pass clears the default framebuffer (this caused
  missing walls), and textures can get mipmaps generated later (static textures now get a full mip chain).
- New backend-neutral hooks (GL + Vulkan): `set_depth_write_override` (replaces the engine's glDepthMask) and
  `texture_grab_screen` (replaces macroquad's glCopyTexImage2D). Remaining raw GL calls guarded.
- **Results vs GL (same build, lavapipe vs software GL, retail data):** skool1 98.4% of pixels bit-identical and
  4 of 816,000 off by >2 levels; fortress1 99.3% identical (6 px >8); pandemonium 97.2% identical (15 px >8).
  Also validated with the Khronos validation layer: 0 errors for the game. Images in `build/` (`*_gl.png`, `*_vk.png`).
- GL path unchanged: GL output is bit-identical to the original reference image.
- PS5 target still type-checks with the backend in.

Files: full diff of the vendored crates `build/miniquad-macroquad-vulkan.diff`; engine diff `build/lookingglass-ps5.patch`;
tree `LookingGlass-src`; translator standalone `ps5-port/glsl100-spirv`; backend conformance test `ps5-port/mqtest`;
captured shaders `ps5-port/captured-shaders`.

Not covered yet: cube maps, MSAA/resolve, array texture sources, vertex step_rate > 1, raw texture ids.
None are used by the maps tested.

**Next (PS5 platform, `native/ps5.rs`):**
1. Vulkan entry: RADV is linked statically into the title, so use `ash::Entry::from_static_fn` with the archive's
   `vk_icdGetInstanceProcAddr` instead of `Entry::load()` (dlopen).
2. A `Presenter` for VK_KHR_display on VideoOut (swapchain; blit the default image with a Y flip), modelled on
   PS5_Quake3e `ps5_glimp.c`.
3. DualSense via ScePad into miniquad events / the engine's gamepad path; cpal PS5 host on sceAudioOut.
4. Build RADV (`PS5_Vulkan/tools/build-radv.sh release`, needs `../PS5_Mesa`) — do this in WSL2.

## Status update 4 (2026-11-16, early morning): PS5 platform layer written

Reference stack: **PS5_VulkanTemplate** (mihawk-99), the foundation PS5_Quake3e itself was built on, not Quake3e.

- `vendor/miniquad/src/native/ps5.rs`: Vulkan entry from the statically linked RADV (`vk_icdGetInstanceProcAddr`),
  a `VK_KHR_display` presenter (swapchain at the display's mode; the game renders at 1920x1080 and is blitted,
  Y-flipped and scaled), pads polled once per frame. Console services come from the template's `platform.c`
  (MIT), copied unchanged to `native/ps5c/` and compiled by miniquad's build.rs (needs `PS5_PLATFORM_SDK`).
- Input: `src/input.rs` maps DualSense to the engine's XInput layout (Cross=A, Circle=B, Square=X, Triangle=Y,
  OPTIONS=Start, touch-pad click=Back); every existing binding and menu path works unchanged.
- Audio: vendored cpal's null host becomes the PS5 output on FreeBSD (48 kHz stereo i16 via `audio_start`);
  rodio needs no change.
- Title: `ps5/build.sh` builds with `ps5/link-title.sh` as cargo's linker, which applies PS5_Vulkan's RADV
  link recipe (as the template's link-title.sh), refuses null imports, signs `eboot.bin` and lays out
  `ps5/dist/PPSA99641/` (title "Looking Glass", sce_sys/param.json, sce_module/libc.prx).
- At start-up on the console the engine changes to `/data/homebrew/PPSA99641`; game data goes there as
  `Alice1/bin/base`, settings/saves in `private/`.
- Type-checks for the PS5 target; Linux build unchanged. Not yet linked: needs the RADV archive.
- Expected first-link risk: ps5-rust's `ps5-std-rt` bundles a patched copy of the stock SDK's libc.a, while the
  template's titles take libc from `libc.prx` and the fork's platform layer (with malloc/pthread wraps).

WSL2 (Arch Linux, official image) runs `build/setup-wsl-stack.sh`; log at `build/wsl-setup.log`.

## Phases

0. Unblock the dev machine: enable virtualization in BIOS (the Linux VM would not start), install WSL2 or use a Linux box.
1. Toolchain: install PS5 Payload SDK, LLVM 22, ps5-rust (nightly). Hello-world `std` payload with threads and std::fs. Confirm via etaHEN payload loader (port 9021).
2. Dependency triage on the `x86_64-sie-ps5-std` target: macroquad/miniquad, image, zip, flate2, fontdue, symphonia, serde, getrandom 0.3 (uses getrandom shim from ps5-std-rt).
   Feature-gate co-op (tungstenite/native-tls) out.
3. Platform: PS5 window/present + event loop (miniquad fork), then graphics per the decision above. Goal: main menu on screen.
4. Audio: bypass cpal; drive rodio's mixer from a thread feeding sceAudioOut at 48 kHz.
5. Input: DualSense via ScePad (+ sceUserService); map to the existing gamepad struct in input.rs; add gyro/touchpad later.
6. Data: load pak/pk3 and the unpacked data from `/data/homebrew/<TITLEID>/`; first-run import from user-supplied files only.
7. Playable milestones: menu -> first level -> full chapter -> full campaign; profile against docs/MAP_PERFORMANCE.md and SKY_PERFORMANCE.md.
8. Package as a homebrew title (sce_sys, eboot), ShadowMountPlus layout like PS5_Quake3e.

## Risks

- ~~GL-class renderer on a Vulkan-only PS5 stack~~ — resolved on Linux (update 3); unproven on RADV/PS5 until the console runs it.
- Memory: the game is ~700 MB of paks; PS5 payload memory budget and 4K surface cost need measuring.
- Missing on disk: PS5_Mesa (PS5_Vulkan, PS5_VulkanTemplate and the payload SDK are in `C:\Alice1\PS5`).
- Performance: one frame in flight and a full barrier per pass/transfer are simple but not fast; tune on hardware.
- Firmware 13.60: confirm the loader chain and that ps5-rust std payloads run as the README claims.
