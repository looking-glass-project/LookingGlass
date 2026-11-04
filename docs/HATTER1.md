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

At this scene milestone, closing-door catch-up rules, the end clock and exit,
continuous two-client traversal, death/checkpoint recovery and transition/rejoin
remained open. These checks did not complete a campaign visit.


## Closing doors wait for living partners (4 November 2026)

A native input fixture reproduced premature laboratory closure with either
first player, including when the partner was still at the outside lever.
Co-op now defers NORETURN and Close_ClockRoom_Door until every living body is
past the east boundary of that door's authored closing-trigger brush. Both
routes enter westward. Using the closing approach also excludes the laboratory
lever alcove, which is west of the door but has not entered the laboratory.
The existing contact system retains the first living requester and unconsumed
one-shot event while waiting. The notice reads "Waiting for your partner".
Original closing motion and single-player callbacks are unchanged.

Source `a00709984d156f1da223004de955c4371a962bd27e983d5d7f77911467e7adf2`, executable SHA-256
`5860ca23d0027e10a2bae75a77d90b06b4216cb8eda0792249336d640c605c9f`, passes **18 new staged local input cases** under
`--hatter1-coop-check`: both doors, either first player and four living/dead
combinations (16), plus a waiting partner at the laboratory lever (two).
Each begins with staged prerequisites and footings; subsequent traversal uses
ordinary movement. Checks preserve personal health and separation, restore a
paused/compressed waiting checkpoint, exclude dead partners, stop both-dead
machinery, retain living ownership and verify completed original closing poses.
Resetting contact history does not replay a consumed closure.

The gear-door cases begin at the static open doorway. An earlier attempt to
include the rotating great gear stalled with idle movement; an attempt to hold
position using running inputs changed health. Those failures remain retained.
They were not converted into a full gear-traversal pass. The safe doorway fixture
isolates closure; it does not verify the preceding gear course or native combat.

The final batch has **56 machinery/door records** and **41 scene records**
(40 cases plus summary). Single-player 11+1 logs still match the pre-adapter
baseline exactly. Evidence: `private/hatter1-coop-a007099/manifest.json`.
Overall credit remains 48.6%, with six combined campaign visits. The rotating
gear, final clock/exit, full two-client route, recovery and rejoin remain open.

## Independent client route automation (4 November 2026)

`tools/test_coop_hatter1.py` drives a fresh prefix with two independent profiles
and the real internet Host/Join path by default. Both players must physically
reach every movement waypoint and receive their own acknowledged command.
The first walks to an observed clear, supported neighbor before the second
arrives; a narrow passage can require another observed walking step to leave
the arrival lane clear. It prefers waiting beyond the doorway to avoid pinning
the incoming player against an open door's tip. Final arrival tolerances and
player spacing are unchanged.
Lever actions use ordinary E input, checkpoints require neutral controls, and
the verifier compares sent/restored player, world and native state hashes.
The evidence retains exact fixture, executable, course and local map identities.

The initial runner supports nav/walk/wait/lever prefixes and rejects a requested
prefix containing unsupported actions. Its default stops after step 17; timed
input, later path nodes, scene handoffs and the exit are still future work.
It aborts on death, respawn or team wipe, so this is not a recovery test.
No authoritative position, resource or progression is written by the runner.

The first live attempts exposed two automation gaps: Clockwork owns its doors
outside the generic door list, and proximity can open the selected door while
leaving a neighboring door closed. Read-only controller observations now tell
the route driver which closed door its ordinary E press will open, including
the existing native nearby-door group. Native use and single-player door
behavior are unchanged. Local passage fixtures start at captured door footings
with either Alice and either a closed or proximity-open first door; their staged
starting state does not count as fresh route traversal. In the proximity-open
case the waiting partner must walk out of the incoming approach before the
follower crosses; an E press cannot bypass personal space or door geometry.

On source `a96876c`, a retained fresh prefix independently verifies steps 0-2,
six personal arrivals and 493 matching player/world/native states. That run
failed at step 3 because its first neighbor ring did not leave enough waiting
space. A repeat failed at step 2 with the proximity-open door edge case.
The native reproduction retained that failure with both actors before adding
the ordinary partner yield. Neither run establishes a completed Clockwork visit.

Source `f6b1a4f2a21759258b9c6f3e533adb46de2aa3f2e1463dfaa23bb52547cf5be3`, executable SHA-256
`30be7e42ddec7be031b2f617b4c0036c5a64777e06a5ce97c052b6a4e08bb158`, passes all
60 machinery/door records and 40 scene cases plus summary. The four ordinary
door cases include both actors and proximity-open starts with partner yield.
Single-player 11+1 logs still match the pre-adapter baseline exactly. All 879
active Rust tests and the 14 Python receipt/parking guards pass.
Native evidence: `private/hatter1-coop-f6b1a4f/manifest.json`.

The fresh internet retry on this source completes both step-0 arrivals, then
stops at step 1 after Red dies during the automated descent. Blue reaches the
step-1 footing with 89 health after ordinary fall damage. Red is still airborne
when health reaches zero; the precise death cause has not been isolated.
Evidence is retained in `private/coop/hatter1-route-53fb361cf47c4fe1999ff406883e0f55`.
No death guard, body spacing, combat rule or resource was changed to continue.
The opening descent and full live prefix remain unverified on this build.

## Observed entrance jumps and ledge pull-ups (4 November 2026)

Later ordinary-input runs isolate the entrance failure. The recorded Red death
occurs inside the bounds of the authored fatal-fall volume (entity 50, upper
height 56), while the successful Blue approach jumps over the descent. The
co-op runner now uses observed takeoff, lower landing and climb footings for
both players, requiring a real jump and individual arrivals. A tight footing
can require normal centering to within two units before choosing a waiting spot.
When a player catches the forward ledge, the runner sends one ordinary pull-up
input and waits for the native climb to complete. It never writes player state.

Jump proof uses each client's own command acknowledgment and the authoritative
host's input events for both slots. It checks actual grounded takeoff and exact
jump-counter increments. Ledge proof additionally requires the initial hang,
native pulling animation, completed climb counter, grounded finish and the
host's actual pull-up input. The 27 Python guards reject missing, mismatched or
unobserved receipts. The optional absent ledge field retains its native meaning.

On unchanged native source `f6b1a4f`, the fresh **0-5** prefix verifies ten course
arrivals, eight intermediate personal arrivals, four actual jumps, one ledge
pull-up, one timed hold and **872 matching player/world/native states**. Both
players remain alive with 89 health. Evidence and independent scoped audit:
`private/coop/hatter1-route-7e2266f6888f4e0387e3718c5a7795ac/completed-prefix-audit.json`.
The audit excludes the later stalled portion and does not claim a full run.

That run fails at step 6: the SP route marker is at `[-3904,2750,208.03125]`,
while the body stops on a clear grounded footing at `[-3919.03125,2750,208.03125]`.
SP's native planning accepts a 22-unit radius; the co-op verifier requires eight.
The co-op target is explicitly offset 16 units west to the observed footing.
Both source and effective courses, their hashes and the adjustment are retained.
The actual arrival radius remains eight. A later fresh repeat verifies 0-5
again (830 matching states), then Blue reaches this adjusted target before Red
falls on the approach. Evidence:
`private/coop/hatter1-route-38158c87d2684412b34f0373fc7e38eb/completed-prefix-audit.json`.
The target adjustment still needs a complete live-prefix repeat. Full-route,
recovery and rejoin coverage stay open.

## Grounded navigation and latest retained failure (4 November 2026)

The optional test driver now tries a grounded walking route before the existing
timed jump planner. It requires supported steering points with space for arrival
variation; required gaps still use the existing jump search. Single-player route
selection and gameplay movement are unchanged. A native regression drives a
supported detour using delayed observations and inputs at 0, 12 and 24 ticks.
The waiting-position guard also requires a short forward step before considering
an incoming-side position that could obstruct the follower at a doorway.

Source `4fd024cc8ae88f702dd1906130cca877f689e138108f9d522f14fce00c8edc0a`, executable SHA-256
`3bce2ae53c5a8d5fdd229e44e684c9b544f3f622f49af84ccbe32faa1d7da6a8`, passes 880 active
Rust tests (one ignored), the locked release build, 27 Python guards and seven
source-package tests. Earlier machinery/scene/SP component checks remain evidence
for `f6b1a4f`, rather than being relabeled as results for this navigation build.

The first internet attempt fails during invite DNS resolution before gameplay.
The next connects and completes Blue's entrance, but Red dies at step 1 before
the second entrance jump. The authoritative trace shows Red walking beyond the
lower takeoff footing and falling to height 55.16 inside the fatal-fall volume;
the guest driver is still settling that footing. Red's actual jump count stays
at one. Observed guest round-trip time has median 340 ms and p95 742 ms, with
the configured 40 ms delay / 20 ms jitter test conditions. These
observations identify a remaining delayed steering failure; they do not prove
that grounded path selection caused it.

Evidence: `private/coop/hatter1-route-f00c1e89aa4642638900339a49ba3fba`.
The test terminated and its owned clients closed. No fresh-prefix or campaign
credit is added. The next reproduction should focus on this lower-footing
approach and its measured delay, before another longer route attempt.

## Delayed final steering (4 November 2026)

The narrowed native reproduction starts on a small platform and uses ordinary
walking controls with separate delayed observation and input queues. The old
landing helper leaves the supported final-approach area instead of settling.
Grounded final corrections now use the existing delay-aware walking gain;
airborne braking, arrival tolerances and gameplay movement are unchanged.
The regression stays grounded and settles within two units at per-direction
delays of 0, 12, 24, 45 and 84 ticks (up to 1.4 seconds combined). All 881 active
Rust tests pass, with one ignored. The locked release, 27 Python receipt guards
and seven source-package tests also pass.

The first live retry passes Red's previously failing lower footing and actual
second jump, then stops with both alive: the waiting Blue body blocks the end
of Red's native pull-up. The entrance's waiting move now centers within two
units, leaving that animation room to finish. It uses the same observed footing
and ordinary movement, with the native collision and spacing checks unchanged.

Source `dbd12c88779018c0678abb3a19c85ad3190fe40b1b7125c9bcc7ba03d96db3cc`, executable SHA-256
`7182e2884137b318bb0a612944f6495f8ad66bc929c062f05c74e08944f02708`, passes two fresh
internet entrance runs (steps 0-1), with Blue and Red respectively going first.
Each verifies four course arrivals, eight intermediate personal arrivals, four
actual jumps, one completed native pull-up and both players alive at 89 health.
The runs compare 697 and 693 matching player/world/native states respectively;
both finish by making the normal shared checkpoint before step 2. Frozen
fixture dependencies and current source hashes match for both runs.

Evidence: `private/coop/hatter1-route-d177a40e2c1146fc98c7dbfd30350a75` and
`private/coop/hatter1-route-d454e04f8f9e4ffba903c1b281590855`.
The blocked-pull-up failure remains in
`private/coop/hatter1-route-8dab0a63d70142da9b9d3e6546099b6b`.
This establishes the short entrance in both orders, not the full Clockwork visit.
The longer opening route, recovery and rejoin remain open. No campaign credit
is added for this automation correction.

A subsequent fresh 0-17 attempt independently verifies the completed 0-5 prefix:
ten course arrivals, eight intermediate arrivals, four jumps, one pull-up, one
timed hold and 981 matching states. The scoped audit ends at tick 9290 and
excludes the later failed step. Evidence:
`private/coop/hatter1-route-0a037c4f58ed4b0c92318c4ffcbbdfe5/completed-prefix-audit.json`.
At step 6 Red reaches the adjusted target and parks, but her waiting body then
obstructs Blue's planned approach. Their horizontal separation remains 36 units;
the run times out with both alive, at 89 and 79 health. This needs ordinary
partner yielding or a better waiting route, rather than relaxing collision or
counting the obstructed follower as having arrived.
