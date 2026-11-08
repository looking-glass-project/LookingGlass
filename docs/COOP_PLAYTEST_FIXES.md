# Co-op playtest fixes and validation

This work addresses the first remote two-player playtest of preview.1.
Both players must use the same candidate executable. Save formats and game-data
requirements are unchanged.
The preview.4 test candidate includes the latency optimization pass below.
Both players need preview.4; older candidates have different co-op build identities.
F11 displays frame timing and the active graphics renderer.

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

## Initial candidate evidence

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

## Network follow-up

The latency injector had its own throughput bug: four delayed packets at
160â€“210 ms limited ordinary input traffic to about 20 messages per second.
A 40 Hz regression accumulated a 2.88-second queue before the correction.
Delayed traffic now has room for packets in flight while retaining the old
32 MiB byte ceiling. Normal, undelayed channels stay at four packets. All
17 transport tests passed, including slow-reader backpressure and cancellation.
The earlier injected-delay timings above therefore combine real path latency
with this simulator artifact; they are not estimates of the cousin's connection.

Undelayed public gameplay still showed a problem: a 90-second run delivered
426 snapshots (10.42 MB), with guest acknowledgement median 701 ms, p95 2.82 s,
and snapshot age reaching 7.87 s. Rendering remained approximately 60 FPS.
Data-free small-packet probes on the same PC were much faster; alternating
small and 25 KB replies also exposed size-sensitive delays without rendering.

Snapshots now send exact byte changes from their preceding admitted snapshot.
The receiver checks the base hash and all copy, insert and output bounds,
reconstructs the original bytes, then runs normal state validation. Full
snapshots remain the fallback whenever smaller, and bases reset at visit or
connection changes. No save schema, gameplay authority or dependency changed.

In a second undelayed public 90-second run, 1,056 snapshots used 2.67 MB:
approximately 90% fewer bytes per snapshot and 2.5 times as many updates.
Guest acknowledgement median was 167 ms and p95 618 ms. After the cutscene,
median was 184 ms and p95 302 ms, with no frames over 50 ms. The public routes
were separate and conditions vary; these figures demonstrate improvement in
these runs, not a guaranteed internet latency. A 2.53-second acknowledgement
outlier remains, so responsiveness is not yet fully resolved.

The development build passed 978 Rust tests (six ignored), seven source-package
tests and a locked Windows release build. Regression tests cover exact delta
reconstruction through shifted content, alternating full/delta frames, missing
or wrong bases, truncated packets and invalid copy/insert/output sizes.

The first native movement/turning stress test exposed another failure after
the opening: a long burst of active controls exceeded the bounded playback
timeline and disconnected the guest. That condition now initiates the existing
ordered control-reset handshake. Stale movement and button edges are consumed
without replay, then fresh controls resume; short bursts retain their original
movement intervals. Invalid clocks/counters still fail validation. A session
regression verifies connection survival, no replayed jump/attack, and exactly
one fresh jump after the reset. A subsequent 95-second native run through a
public invite added 160 ms delay plus up to 50 ms jitter per sender and stayed
connected through the opening and real post-scene keyboard/mouse input. It
observed 1,081 mouse events, 0.445 radians of camera rotation and 81.6 units of
guest movement. Guest post-scene acknowledgement median/p95 were 499/684 ms
(including injected delay); 2,177 post-scene frames had no frame above 50 ms,
with a worst frame of 34.2 ms. One earlier guest frame exceeded 50 ms in the full
5,737-frame run. This run did not trigger the reset barrier, whose forced
backlog/action behavior is covered by the session regression.

The same executable then passed public-invite solo import, checkpoint resume,
guest crash/rejoin, abrupt host loss, resume after loss and backup recovery.
Both network views restored exact saved state, personal inventory remained
separate and the original single-player save was unchanged. Its SHA-256 is
`7fea0fc3fcaf97babfe0d6b847ab4f7e042054f22188fb77be014dc68cebf7c2`.

Windows MSVC builds export NVIDIA's documented `NvOptimusEnablement = 1`
preference. The built executable's export and value were checked. This requests
the high-performance GPU on Optimus laptops; user/OS policy can override it.
The active OpenGL renderer is recorded in F11 diagnostics, rather than inferred
from installed hardware. The test PC reported an RTX 4090; this does not establish
which GPU the cousin used. See [NVIDIA's rendering policy](https://developer.download.nvidia.com/devzone/devcenter/gamegraphics/files/OptimusRenderingPolicies.pdf).
The cousin reports that single-player was smooth and only joining co-op was
choppy, so co-op-specific work remains the primary investigation.

The performance runner can exercise real guest keyboard/mouse input after the
opening scene with `--metrics-only --exercise-guest --seat-session <id>`, and
checks observed movement and camera rotation. Pass the active Anode lease
environment to use its verified window-focus operation. No screenshots or
game-state edits are needed. Compact reports also record delivered snapshot
counts, bytes and age; frame rate alone cannot prove responsive gameplay.

## Preview.3 package checks

The preview.3 executable also passed the public-invite menu acceptance checks:
host difficulty, both aim renderers, both Tab/console overlays and shared chapter
travel. The guest rendered at exactly 2560 x 1600. F11 reported its active GPU,
and its labels were visually checked after fixing overlap at larger UI scales.
The version build passed the 978-test suite; the final overlay layout was rebuilt
and checked in the native client. Seven source-package tests passed.

Final preview.3 executable SHA-256:
`3bba8bea872a31efc83b2a4027f442d81fa4fdfe20af6f1da9fbf6c9cca1a2ec`.
This remains a test candidate for the remote laptop, not a published release.

## Latency optimization pass

Idle camera turns previously prevented the input timeline from discarding old
network delay. A deterministic 50 ms one-way fixture reproduced a 250 ms delay
before fresh movement after a temporary stall. Neutral camera updates now merge
only after their action counters are consumed; the regression requires fresh
movement within 100 ms and preserves the original aim of queued attack presses.
Run-ups, short pauses, weapon changes and action edges keep their timing.

Snapshot flow adapts from four to at most twelve outstanding updates using the
fastest validated receipt. A separate 64 KiB wire-byte budget prevents a large
snapshot backlog; one oversized full snapshot may travel alone. A stalled or
forged receipt cannot free arbitrary slots or keep increasing the window.
The input and snapshot schedulers preserve their 30 Hz and 20 Hz cadence through
render jitter, and never replay missed sends after a stall. Unchanged snapshot
frames skip redundant particle bindings, and pose restoration reuses buffers.

Compact F11 diagnostics now measure CPU phases without serializing the world.
The performance harness records process CPU consumption and can place both
clients side by side with `--tile-windows --seat-session <id>`. Use a sufficiently
wide Anode display, such as 2560 x 1440, to avoid overlapping test windows.
All profiles are isolated. Neither screenshots nor live game-state edits are
needed for this performance test.

A controlled 45-second school comparison used the same two-window layout,
160 ms plus up to 50 ms jitter per sender, and real guest movement/mouse input.
Before the final cadence adjustment, admitted guest snapshots increased from
436 to 776; median snapshot age fell from 50.1 to 16.8 ms, and p95 inter-snapshot
gap from 233.3 to 83.5 ms. Input acknowledgement median stayed approximately
401 ms, which includes the injected round trip. Both builds had approximately
60 FPS median and two startup frames over 50 ms per client. These are same-PC
measurements, not evidence that the remote laptop's original stutter is fixed.
Earlier overlapping-window runs had substantial, variable stalls in both the
baseline and changed builds; they are retained as diagnostic evidence rather
than used to claim a frame-rate improvement.

Preview.4 passed 985 Rust tests (six ignored), seven source-package tests,
Python syntax checks and the exact staged source audit. The locked static-CRT
Windows release build completed. Frozen executable SHA-256:
`c02bb765612cd5de54f0c97ae2a869491476dbd7876fcf9a17108d592456cc61`.
Native follow-up results are recorded in the draft PR and private receipts.
