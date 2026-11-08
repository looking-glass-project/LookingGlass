# Co-op playtest fixes: 0.33.0-preview.2 candidate

This candidate addresses the first remote two-player playtest of preview.1.
Both players must use the same candidate executable. Save formats and game-data
requirements are unchanged.

## Changes

- Setup distinguishes download completion from a saved installation. Closing
  during installation finishes the work without launching the game; explicit
  Cancel still stops it. Open files folder locates the installation or download.
- Video settings include 2560 x 1600, preserving older resolution indices.
- Each co-op player renders their own world-space aim glow.
- Host > Start chapter opens the existing chapter and difficulty chooser.
  Hosting an existing game or resuming a checkpoint keeps its saved difficulty.
- Tab and the console work on both clients. Shared chapter changes, restart and
  saving belong to the host. Console help lists the supported co-op commands.
- Normal play no longer serializes and writes the whole world to disk five
  times per second. Opt-in diagnostics use a bounded background writer.
- Live player snapshots retain particle caches; disk restores still reset them.
- A network stall can deliver an old neutral input followed by a fresh one in
  the same poll. The host now collapses that stale idle interval instead of
  rejecting the guest. Active input intervals and action counters remain bounded;
  a separate elapsed-time check rejects forged clocks.
- F11 shows frame rate and writes compact frame-time/network diagnostics to
  `private/coop-performance.json` (or the selected profile directory).

## Repeating the focused checks

Run the native tests in a separate Windows desktop with compatible local data.
They create isolated profiles and use public internet invites by default.

```powershell
python tools/test_coop_menu_features.py --exe <exe> --data <base> --seat-session <id>
python tools/test_coop_import.py --exe <exe> --data <base>
python tools/test_coop_performance.py --exe <exe> --data <base> --seconds 90 --delay-ms 160 --jitter-ms 50 --metrics-only
```

The performance test observes every frame through compact per-second windows,
including the opening scene and its handoff. It also verifies that full-world
diagnostics remain disabled. Failure receipts retain samples and exit codes.
Use the ordinary report mode only when comparing older builds without the F11
metrics; those older sampled reports do not measure every frame.

## Scope of the evidence

On 8 November 2026 the candidate passed 973 Rust tests (six ignored), sixteen
setup tests, seven source-package tests and two launcher tests. A native display
check rendered a 2560 x 1600 client area. Native menu tests exercised both aim
renderers, host difficulty, both Tab/console overlays and shared chapter travel.
The final executable also passed public-invite solo import, checkpoint resume,
guest crash/rejoin, abrupt host loss, resume after loss and backup recovery.
Personal resources and exact saved state survived; the original solo save did
not change. The rejoin harness waits for observed disconnect instead of assuming
the public relay releases a crashed guest's slot within half a second.

Tested executable SHA-256:
`b82ccce9342023881847d5f55b6e3faa34e2497c126990230f1f23a644ca78ac`.

The 90-second public-invite performance run added 160 ms delay plus up to 50 ms
jitter per sender. Both clients stayed connected and recorded 5,448 frames each,
with median reporting-window FPS approximately 60 and no frames over 50 ms.
The guest's worst frame was 36 ms during the opening and 21.5 ms after it.
Earlier runs exposed and retained evidence of the stale-idle rejection above.
This stress test does not establish low end-to-end input latency.

A follow-up audit of the same run found guest input acknowledgements reaching
3.35 seconds and the oldest pending input reaching 3.50 seconds despite smooth
rendering. Post-cutscene p95 values were 3.18 and 3.42 seconds respectively.
These measurements include the injected delay and public relay; they are not
the cousin's measured latency. Guest responsiveness and post-cutscene recovery
therefore remain open. The performance runner now summarizes network timing
alongside frame timing, including separate scene and post-scene intervals.

The remote player's severe camera stutter has not been reproduced on the test
PC. Removing synchronous diagnostics and repeated particle resets addresses
avoidable render-thread work, but a local two-process run with injected delay
does not establish performance on a second GPU in another country. The candidate
needs another remote playtest before that report can be marked fully resolved.
