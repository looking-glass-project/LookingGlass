# Crazed Clockwork — hatter1

The visit now has a persistent controller for its machinery, five levers, mirror puzzle, chair puzzle, scenes and gated `hatter2` exit. The original assets stay in the user's local installation; the controller does not execute original scripts.

## Restored progression

- The two gear levers work in either order. Each works once, with the original two-second delay. The first starts the great gear; the second lowers it and raises the passage barrier. Crossing the passage closes the barrier behind Alice.
- The spiral and plunger move. The lift rises and lowers with its rider, and the floating platform follows its authored loop.
- The mirror field has five stationary pillars and eight sinking traps. The false versions appear only in the mirror, follow the real traps and have no collision. Cheshire's hint also appears only in the mirror.
- The clockroom lever opens its paired leaves; crossing the room closes them. The laboratory conversation enables the port lever. Its scene opens the port, both Hatter door leaves and the three automatic gates.
- The Gryphon conversation removes the exit clip and door. The extending bridge and its rescue teleport become available on the onward route.
- Four distinct chair contacts lower the floating clock. Repeated contact with one chair cannot solve the puzzle. The clock door and teleport move with the clock. The floor rescues cannot bypass the preceding puzzle.
- The final lever plays the clock-stop camera sequence, settles the clock hands and opens both exit leaves. The exit enables only when the sequence finishes. Watching and skipping use the same final puzzle state.
- The onward exit loads the existing About Face battle at its default arrival. The battle controller is registered in the combined build.

Arrival restores resources once, as the original map does. Save data includes lever use, door movement, sinking platforms, chair contacts, scene position, speech cursor and completion flags. Older saves without this controller restart this visit at its entrance while retaining their carried resources.

## Verification

The mirror's near-plane clipping accounts for its reversed horizontal projection.
The reflected room remains visible at shallow angles beside the entrance pillar.
Native mirror checks cover the normal approach, Cheshire's hint and two close
side views, and reject an empty reflection. Projection tests cover seven view
angles at 4:3, 16:9 and 21:9 while retaining the aperture clipping plane.

The mirror pass now honors the main camera's PVS for stationary apertures and
uses its reflected, obliquely clipped projection for geometry and transparent
effect rejection. Moving mirrors retain transformed visibility checks; missing
PVS data fails open. Reflection resolution, update rate and puzzle state are
unchanged. The native render check rejects unnecessary reflections in the gear
room and laboratory, alongside the four visible-mirror checks above.

Ten fixed Clockwork scenes matched pixel-for-pixel before and after this change.
Native release profiling and exact diagnostic sources are retained locally in
`private/clockwork-performance-fix`. Timings include driver submission/readback
costs and vary with system load; they are not hardware-independent FPS claims.

### Lever interaction correction — 1 October 2026

All five levers now include Alice's supplied `use_lever` performance, the moving
handle and its timed sound. Alice approaches using normal player collision and
step movement; an obstructed approach cancels without consuming the lever. A
collision-checked oblique camera frames the interaction. The final clock scene
follows the pull. Pause and optional saved action state retain the approach and
animation phase; old saves do not replay already-used levers.

The lever contract exercises all five contacts at 30/60/144 Hz, pause and
restored continuation. Native captures cover approach, reach, pull and finish.
The Clockwork Store suite includes an eighth case saved during the hand action.
These are component and persistence checks, not a new full-map campaign run.

- `--hatter1-check`: lever ordering, repeated/out-of-order events, pause, rejected saves, moving teleport, 30/60/144 Hz clock-stop completion, watched/skipped conversations and an input-driven crossing into the implemented Hatter arena.
- `--hatter1-traversal-check`: production lever input, all thirteen mirror-field pillars, a lift rider, four cushion contacts and movement from each authored launch pad onto its cushion. The component starting positions are staged; this is not a full-map combat playthrough.
- `--hatter1-render-check`: native machinery, mirror and scene captures. `LOOKING_GLASS_CLOCKWORK_SAVE=write` / `read` selects separate-process native Store fixtures, including a conversation in progress.

## Fidelity limits

The existing spline and character renderers interpret the supplied camera and animation assets. The laboratory's introductory walks use bounded timings, and its peripheral child-processing machinery is not fully reenacted. The post-conversation dunk/electric loop is a bounded visual interpretation. Sink limits without an explicit map value use 192 units (16 for cushions), with a capped return speed; native sink dead-time semantics remain an approximation. The mirror renders the room, puzzle brushes and its own Cheshire actor; it does not add reflections of every dynamic creature. The script references a nonexistent `rage_gear`, so there is no such mover to restore. These limits do not replace the required puzzle contacts or enable an early exit.

Native checks run muted on the hidden test desktop. They do not verify audible playback or every combat encounter and difficulty.


## Initial paired adapter (4 November 2026)

Co-op now has an explicit world-clock and lever adapter. It aggregates both
living players' supports and door occupancy, advances shared machinery once,
and validates both carried bodies before committing their movement. A blocked
lift rider rolls back the shared motion; a blocked cushion can stop independently.
Lift departure and the floating platform wait for all living riders on their
actual supports. The lower-floor contact can still recall the empty upper lift.

Each lever belongs to its actual user. Its approach uses normal collision,
including the partner's body; a blocked approach times out without consuming
the lever. The partner remains independently controllable during the hand action.
The delayed Port scene retains its initiator, and the final Stop scene waits
for the hand animation. A partner death cannot cancel the owner's pull; an
owner death rearms an unfinished final lever. Per-player initialization avoids
refilling a low-health revival. Optional co-op save state retains old SP saves.

`--hatter1-coop-check` passes **38 staged native component records** on source
`9ce4f981ed1d4c9111c9cf2a706201ee99afaa66e0b090d9452b8360c4f01ea2`, executable SHA-256
`0faa7bacc4502883bd09dd8b75288fac524851fa8186c6d5c70abb59a4db1d9f`:
three clock/rate/pause/initialization cases; six lift/floater carry/obstruction
cases; 13 actual boarding/recall contact cases; ten personal lever cases; two
independent sinking-pillar cases; two blocked approaches; two final-lever deaths.
Checks include compressed restore during movement/hand actions, saved waiting
order, either slot, dead-player exclusion and non-overlapping placement.

The single-player `--hatter1-check` (11 records) and
`--hatter1-traversal-check` (one record) match their pre-change `f9e7029` logs
exactly. All 879 active Rust tests, the locked release, seven source-package
tests and source-worktree 890-file audit pass. Local evidence is retained in
`private/hatter1-coop-9ce4f98/manifest.json`.

The first boarding fixture failed because separated deck positions missed the
small central lift trigger by one unit. Moving those staged positions inward
two units retained 44-unit player separation and exercised the actual trigger.
The waiting player first contacts the lower recall floor before boarding, as
on a normal approach. The production boarding guard was retained.

These are components, not a fresh two-client playthrough. At this initial
adapter milestone, complete paired cinematic handoffs, remaining machinery/exit
boarding, internet traversal, recovery and transition/rejoin remained open.
No campaign visit is marked complete. Single-player scene/physics paths remain
unchanged; no original assets are included in the source changes.


## Paired scene handoffs (4 November 2026)

`--hatter1-coop-scene-check` passes 40 staged native cases (41 PASS records,
including its final summary) on source
`ec48aa77d0ab0276003f38857e13150d2d65742a9744fca6fa30c2891e0d687d`, executable SHA-256
`1aa190a6b475ab2632f485c7cf0b81e6718062cc76e2fe440ecfbe54cce507e0`.
Hare and Gryphon start through their native contact brushes; Port and Stop
start through actual lever inputs and the original approach/hand-action delay.
Prerequisite progress and starting positions are staged explicitly.

All four cinematics run with either owner, watched or held-skipped, and a
living or dead partner (32 cases). A partner cannot skip the owner's scene.
Checks retain the waiting player's position, preserve both personal resource
sets, compare camera, machinery and authored Alice pose after paused/compressed
restores, and verify the original progression outcome. Living partners reunite
on supported, clear, nonhazardous, separated positions; dead partners stay put.

Mirror and Hint cover both initiators with living/dead partners (eight cases).
Their speech clock advances once, neither player acquires scene control or
hidden-body flags, and both living players can move from distinct entrance
footings. Dead partners stay put. These are state-level presentation checks,
not a new rendered-image comparison.

The same build passes all 38 paired machinery records. Single-player component
logs (11+1 records) match the pre-adapter baseline byte-for-byte after newline
normalization. Evidence is retained in `private/hatter1-coop-ec48aa7/manifest.json`.
No gameplay change was needed for these scene cases.

Closing-door catch-up rules (NORETURN and the laboratory door), the end clock
and exit, continuous two-client traversal, death/checkpoint recovery and
transition/rejoin remain open. These checks do not complete a campaign visit.
