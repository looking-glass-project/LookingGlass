# Rolling Stones owner

`garden3` serves visit 15, including `garden3_start1`; the canonical save key is
`garden3$first`. It reserves 7,400,000–7,499,999 and uses the existing format-12
controller registry. No route ordering or existing event keys change.

## Introduction and handoff

`Garden3_Start` uses the five supplied camera tracks. Cuts occur at 0, 4, 8,
16.5 and 19 seconds; the second camera path has already advanced four seconds
when selected. The cheering and flying Ladybugs use their supplied clips and
spline node speeds, including the enlarged carrier and its marble attachment.
Default weapon attachments are suppressed. No path is invented for the absent
`bug_path3`. The scene has no dialogue.

The real `chase_marble1` becomes visible at 18.5 seconds and activates at 19.5.
Alice runs at 23.5; gravity changes from 0.6 to 0.2 at 24.5; control returns at
25. The W9 instance advances with a saved 120 Hz cursor during the introduction.
Skipping advances that same instance to the common endpoint. It does not create
or restart another rock. The marble remains non-solid during the scene and
becomes solid at completion, with its elapsed time, velocity and waypoint cursor
intact. The carrier marble and cinematic cast then disappear.

Alice's deterministic landing is 80% along the authored walk/run corridor,
settled to the real floor with the run marker's authored 45-degree facing. Completion checks
body clearance and at least 200 units of marble lead. The observed lead is over
800 units; ordinary player movement is checked immediately after control returns.
The final visual running pose uses the same floor query. Fog resumes its three
ten-second fades after the handoff.

## Physical chase prerequisites

The W9 adapter accepts `models/marble.tik`, whose explicit server bounds match
the already supported boulder bounds. It follows all 56 waypoints, per-leg
speeds and four one-shot callbacks. Swept contact delivers 999 damage through
the normal combat feedback path. Its reviewed timed movement excludes scenery,
so gates cannot stop the marble. Player collision uses the same marble position
that is drawn. Paused time advances neither motion nor contact delivery.

Door entities 44, 45, 46 and 49 preserve START_OPEN parity, targeted toggles,
10,000-unit movement speed and authored travel/lip. The real trigger-to-target
graph toggles them. A forward gate latches open after the marble passes its
corresponding route section, preventing a later toggle from stranding Alice.

The pillar, its bound mushroom, 23 ice pieces, eight ending pieces and ending
clip share saved clocks and render/collision transforms. Rock callbacks and the
real Alice triggers initiate the appropriate collapses once. Static ice and
platform remnants remain. The ending clip retires after 0.6 seconds; only then
is the existing `garden4$garden4_start1` exit eligible. Existing pads and lethal
fall volumes remain owned by the shared traversal/interaction systems.

The first, pillar, second and ice quake events drive bounded camera shake from
their saved clocks. The second quake, ice rumble and later cracking/crashing cues
are audible. Settled brush platforms support ledge grabs; moving or removed
pieces immediately leave that support set.

The middle mushroom on `falling_pillar1` has collision from its actual model
triangles at the placed scale, including the cap overhang beyond the pillar.
Rendering and collision use one attachment transform. The cap supports standing
and ledge checks while settled, follows the pillar during collapse, and disappears
with it. Restoring a save rebuilds this collision; the bounce mushrooms are unchanged.
`--garden3-check` verifies five landings, stable footing, the rounded edge, collapse
and restoration. `--garden3-render-check` adds a staged capture of Alice standing
on the cap after those movement checks.
The route replay approaches the first pad nearer its eastern side and then aims
across the cap; its former input path cut through the missing collision at the rim.

## Saves and checks

The owner saves scene/camera/cast clocks, release cursor, speed, gravity, rock
motion, callback history, gate parity/travel, floor clocks, pending contact and
fog time. Restore validates consistency before replacing the live owner. Old
controller-less saves migrate to the common post-introduction chase handoff;
they do not replay the arrival. Previously pending floor/exit triggers use the
existing registry migration and remain usable.

`--garden3-check` covers watched/skipped outcomes at 30/60/144 Hz, eleven paused
and restored futures around release/gravity boundaries, actual post-handoff
movement, lethal contact, gate dispatch, the entire moving-rock path, physical
collapse, old runtime migration and malformed saves. Six `garden3-` save cases
and `--garden3-render-check` exercise the native renderer and save reader.

`--garden3-route-check` and `--garden3-skip-route-check` each start at the real
arrival, then traverse both canyon loops, the gate triggers, both mushroom pads,
the outer descending route, the ice cavern and the final platform drop using
ordinary movement input. Each runs at two paces: a sprint and a slower chase
that lets the marble start the ice collapse while Alice is still on the floor.
Both reach `garden4$garden4_start1` alive with 80 Sanity, preserving the resource
ledger and verifying a clear arrival body. They use no flight, god mode, health
injection, teleports or direct event activation. The upper cavern shortcut is
not used: its large fall can be fatal under the current shared damage rule.

The route records input and writes real checkpoints at the gates, in the canyon,
during the first launch, during the pillar collapse, on the collapsing ice and
during the ending drop. Reloaded runs compare complete state hashes every second
and exactly at the exit. The two introduction outcomes and two paces cover 22
saved continuations. Additional negative runs stop moving at the canyon and on
the ice, proving the marble and collapsing floor remain lethal.

Five `garden3-live-` native save/render fixtures use those physically reached
checkpoints, alongside the six original introduction fixtures. Their writer and
reader run in separate processes. Exact original bounce spin, quake waveform
and the stray Ladybug's actor navigation remain fidelity follow-ups.

## Co-op chase update

Rolling Stones now explicitly advances its native marble, gate latches and
collapsing geometry once per host tick with both living players present. The
marble's existing swept contact selects a stable player slot; only that player's
health and velocity receive the hit. Dead players no longer block the marble
from reaching the survivor. Shared callbacks and geometry use the same routines
as single-player. The original arrival still runs with one Alice and reunites
the pair afterward. No controller/save fields or format versions were added.

`--garden3-coop-check` passes 11 staged native cases on source
`6df84313f78ae427eea47d108cf006607c5b1c5658e13fa4073e2d796dae8827`:

- Shared clock/geometry parity at 30, 60 and 144 Hz, including pause and a saved future.
- The full 56-node marble path, gate latches, pillar and ice callbacks, with
  exactly the same state/geometry as single-player. The player-triggered final
  drop remains untriggered by this clock-only check.
- Either victim slot with a living or dead partner, plus both players in the
  marble's path; hits retain the native lethal damage and knockback.
- Watched/skipped arrival, a frozen waiting partner, compressed saved-scene
  restoration, separate safe reunion and exact personal resources.

The headless run takes about 3.6 seconds; log
`private/coop-garden3-components-6df8431.log`. All 808 active unit tests pass.
These are staged component checks, not full co-op traversal or internet proof.
Rendered guest effects, shared recovery/rejoin and the full paired route remain open.

Current single-player regression evidence supersedes the older route claims
above: both watched/skipped fast routes and their saved continuations pass,
but the slower routes die near node 10. The short `--garden3-check` also stops
at its old one-second movement-distance assertion. That fixture requests walking
while expecting more than 150 units; the current walk speed is 104 units/second.
The controller's scene/pause checks pass before that assertion. All three
complete logs are text-identical to the pre-adapter executable `3c996ca`:
`private/coop-garden3-single-6df8431.log`,
`private/coop-garden3-route-watch-6df8431.log`, and
`private/coop-garden3-route-skip-6df8431.log`, compared with
`private/coop-garden3-baseline-3c996ca.log`,
`private/coop-garden3-baseline-watch-3c996ca.log`, and
`private/coop-garden3-baseline-skip-3c996ca.log`.
The older walking-pace fixture assumptions still need correction; no passing
whole-suite result is claimed for those commands.


## Paired normal-route diagnostic, 7 November 2026

`--garden3-coop-route-check` now runs ordinary paired controls in the production
Arena, using the same 59 authored test waypoints as the solo driver. It records
exact recovery, guest replicas, original scene reunion and next-map resources.
Its first two attempts failed: both players stall near waypoint 16 before the
marble catches them. Reducing corner-cutting and delaying the follower's start
did not resolve that failure. Those attempts earned no route credit; the fix below now passes.

The current frozen `4bb7dac` solo fast route passes through Icy Reception and
its five saved-continuation comparisons. The subsequent paced run fails earlier
at waypoint 3, so the combined solo command exits 1. Evidence is retained in
`private/coop-rolling-solo-4bb7dac.log`. The fast solo route exits while the
marble is at node 44 and its ice callback has not fired; the new paired driver's
original final callback requirements were stricter than this legitimate fast exit.


## Shared gate fix and normal paired pass, 7 November 2026

Build `7710f92` fixes repeated co-op activation of chase triggers 41, 47, 48 and
73. Contacts had been deduplicated only within a single simulation tick, so the
follower could toggle the next gate closed on a later tick. The co-op dispatcher
now consults existing committed event usage for these four one-way markers.
Save restore, a fresh guest and respawn contacts retain that history. Other
repeatable contacts and single-player toggle behavior remain unchanged.

The fresh normal paired run reaches actual Icy Reception arrival in 10,699
ticks, with 89 exact compressed Arena replicas, one watched scene reunion,
both mushroom launches, no deaths or wipes and both players at 86 health.
Personal resources survive the onward transition exactly. Fast completion can
precede the marble's late ice callback and gate latches, which remain covered
by the shared-clock component check; these are not required before a real exit.

Eight new real-map cases cross every gate marker with each player first, then
restore the world and cross with fresh partner contacts. Each records exactly
one activation. Solo double-toggle behavior also passes. All 947 active Rust
tests and the locked release pass; the full Herbaceous route repeats with five
exact ten-second revivals. Evidence is pinned under
`private/coop-rolling-playable-3/` and
`private/coop-playable-rolling-route-verified.json`. This credits normal paired
progression; exhaustive per-visit internet, presentation and recovery checks
remain polishing work, and no new rendered-camera proof is claimed.
