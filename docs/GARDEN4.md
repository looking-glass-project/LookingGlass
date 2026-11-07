# Icy Reception

`garden4` now owns the full frozen approach and its Caterpillar portal. Start it
with `tools/launchers/Launch-Icy-Reception.cmd`, or select Icy Reception with Tab in the normal
launcher. Save envelope 12, route index 15 and reserved hit base 7,500,000 remain.

## Frozen approach

All 14 breakable-floor brushes, 13 static ramp brushes and eight blocking-wall
slabs have matching drawing and collision. The floors drop and turn in their
authored groups when the rolling-rock waypoints fire their callbacks. The first
two collapses are inhibited on Easy; the altar sequence still runs there through
its separate pickup-sized contact volume. Repeated rock activation is idempotent.

The seven rocks use the shared timed waypoint simulation: per-node speeds,
gravity, continuous player contact, rolling models, callbacks and saved motion.
Ice rock collision bounds come from their supplied models. The altar pair and
wall-smashing boulder remain harmless. The second end boulder and marble retain
their damage. Collapse/bounce cues and bounded camera shake accompany the motion.

The unskippable marble scene uses its two supplied camera tracks. It removes the
lower ice and returns Alice to the authored marker, settled on supporting ground,
with a checked lead over the marble. The end boulder opens the wall in eight
staggered pieces. Crossing the fog trigger restores the return-blocking brush;
the shared fog change and both authored steam vents remain active.

## Caterpillar and moving exit

Trigger #49 starts the Caterpillar's ten dialogue beats, with original voices,
subtitles, lip/head bindings, Alice/Caterpillar acting and eight camera tracks.
The closing performance includes smoke, bounded drugview and the portal reveal.
Watching restores control after the 14-second reveal. Skipping retains the closing
line and 5.5-second reveal, then waits four more seconds before enabling the exit.
Holding skip cannot restart the closing performance.

The initially hidden portal #8 follows its six-node path. Its bound exit #6 uses
the same transform and rotated offset every frame, so its editor position is not
an early exit. Both endings commit the same supported Alice pose and portal
endpoint. Only a completed scene with an expired unlock delay allows entry into
`centipede1$centipede1_start1`. Failed level loads retain the exit retry latch.
No resources or upgrades are granted by the scene.

## Persistence and verification

Floor, wall, rock, scene, speech and unlock clocks are saved. Pause freezes them;
malformed rock states are rejected before replacing live state. Older scene-only
saves preserve consumed cave events, so a previously used wall trigger cannot
strand the player. They do not gain a completed Caterpillar scene. Controller-less
saves likewise keep the portal closed and recover consumed encounter triggers.

- `--garden4-check`: all four difficulty configurations, floor/wall callbacks,
  harmless rocks, fog backstop, pause/restored motion, 30/60/144 Hz marble handoff,
  watched/skipped scene boundaries, interruption, old saves and exit retries.
- `--garden4-route-check` and `--garden4-skip-route-check`: continuous Normal/Easy
  traversal from the entrance, Ice Wand pickup, marble escape, wall passage,
  both steam vents, Caterpillar and normal entry into Fungiferous Flora with
  carried inventory/resources. These drive ordinary movement; scene handoffs
  are the only scripted relocation.
- `--garden4-render-check`: nine explicitly staged native captures.
- `LOOKING_GLASS_SAVE_CASE=garden4-` with separate `--save-check-write` and
  `--save-check-read` processes: nine native persistence fixtures, including
  falling ice, moving wall, marble camera, dialogue, smoke and delayed exit.

The route check covers world progression, not a full Ladybug combat balance pass;
placed Ladybugs retain the shared resident controller. Exact original drugview
distortion and portal rotation semantics remain visual-fidelity research. Native
checks are silent, so they do not establish audible playback quality. Original
scripts/assets and private captures are not part of the source package.

## Paired adapter

The co-op world now advances the native seven-rock simulation, floor and wall
callbacks, fog backstop and portal clocks once per shared update. Rock contacts
retain stable Blue/Red identities, ignore dead bodies and apply native damage
and knockback only to the struck player. The harmless altar/wall rocks retain
their original behavior. Generic resident enemies still use the shared cast.

The Easy-only altar tracks each living player's swept path independently. This
optional saved history is validated and survives snapshots; older saves omit
it. Respawn clears only the returning player's path, and scene handoff clears
pre-scene paths to avoid sweeping across the reunion teleport. Single-player
saves continue to omit that optional field.

`--garden4-coop-check` stages native shared-clock, personal rock-contact, altar,
marble and Caterpillar cases. It includes watched/skipped Caterpillar outcomes,
both owners, frozen partners, pause/snapshot restoration, supported reunion,
the four-second skipped-scene portal delay and the bound exit trigger. These
component checks do not establish full co-op traversal, independent-client
internet playback, rendered effects or the death/wipe/rejoin matrix.

The single-player portal fixture now allows five seconds to walk from the
Caterpillar landing to the portal. Its older two-second bound ended short of
the trigger at the current walking speed; this changes only the test deadline.

Source `b7ba0573a6ff7e0fef04782db8e3fc93f29f877baee5632ea4b00f5110f8bca6`
passes all **44 staged co-op cases**: three update-rate clock comparisons, four
difficulty callback/geometry comparisons, 28 native rock/victim/dead-partner
combinations, three Easy-altar cases and six original scene-owner/ending cases.
The locked release build and all 808 active unit tests pass (one existing test
remains ignored). The full existing `--garden4-check` also passes, including
watched/skipped outcomes, restored motion and legacy-save checks. Evidence:
`private/coop-garden4-components-b7ba057.log`,
`private/coop-garden4-single-b7ba057.log` and
`private/coop-garden4-adapter-unit4.log`. No images were viewed.

Both existing single-player route commands pass on Normal and Easy. Their full
logs are text-identical to the pre-adapter `c6a4ddc` build, including the carried
resources and transition into Fungiferous Flora. Evidence:
`private/coop-garden4-route-b7ba057.log` and
`private/coop-garden4-skip-b7ba057.log`, compared with their matching
`*-baseline-c6a4ddc.log` files. These are single-player regressions; paired
entrance-to-exit traversal was unverified at that checkpoint; the later normal paired pass is recorded below.


## Normal paired route accepted, 7 November 2026

`--garden4-coop-route-check` now runs ordinary paired movement/combat in the
production Arena. It shares the solo route's 35 waypoints, completes planned
landings, pulls up actual caught ledges, rides both vents and allows normal
personal recovery. It never stages bodies or changes world progression.
Run with `python tools/test_coop_garden_native.py --map garden4 --exe <frozen-exe>
--data <base-directory> --profile <new-directory>` in Anode. This first course
uses Blue's initial lead and watches the original scenes.

Frozen `cd89026` passes **21,406 ticks** and **35 matching compressed Arena
replicas** into actual Fungiferous Flora. Both players collect their own Ice Wand
and complete each vent rise. Red activates the marble scene; Blue later
activates the Caterpillar scene. Both reunions finish and the real portal opens.
Both arrive alive at approximately 23.29/88 health, with every personal Stats
field unchanged by the onward transition and safe, separate spawn positions.

Blue's death at tick 2,348 recovers at 4,389 with 25 health: exactly 1,200 active
countdown ticks plus 841 cutscene ticks, during which the countdown pauses.
There is no team wipe, cheat, forced progression or gameplay rule change.
The 2,204-row trace verifies safe revival and preserved resources. This is a
normal route with recovery, not a flawless or new internet/camera pass.

The initial `84b284a` batch passes 947 active Rust tests and both unchanged
Normal/Easy solo routes. Later changes only refine the diagnostic controls;
the final locked release and paired run pass. Earlier failed attempts are
retained: premature grounded planning, a fallen player's immediate abort and
a missing ledge pull-up input. Final evidence is under
`private/coop-icy-playable-4/` and the source/executable/report/trace audit
`private/coop-playable-icy-route-verified.json`. Existing co-op core/internet
evidence is reused for unchanged systems. Broader per-visit permutations remain
in the polishing backlog.
