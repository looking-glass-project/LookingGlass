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

The co-op playable audit on 7 November connects 26 ordinary save/resume
sections from the entrance through all 150 course actions to both players'
actual About Face arrival. Every selected handoff matches the checkpoint bytes
and the native exact-resume receipt. A previously unchecked 14–16 prefix now
passes its frozen verifier; disconnected receipts are excluded. This reuses
historical build-specific gameplay, not a fresh uninterrupted latest-build run.
See [playable acceptance](COOP_PLAYABLE.md) for the remaining campaign scope.

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

## Ordinary partner yield and first two levers (4 November 2026)

The route runner now handles a stalled follower in contact with the waiting
first player. It sends a neutral command to the follower and checks its
acknowledgment and stopped body. The waiter then walks to an observed native
supported neighbor clear of the planned approach, and the follower resumes.
The yield is bounded to three attempts per arrival. Its verifier checks the
observed candidate list, the follower's stop and the waiter's separate arrival.
The original collision, spacing, death and waypoint checks remain in force.

An initial reproduction showed that a pinned player can retain nonzero attempted
velocity while its position stays fixed. Contact detection therefore uses stalled
positional progress and a stationary waiter; it still requires the follower to
stop after the neutral command. All 38 Python guards pass, including that captured
case, unsafe candidate rejection and missing acknowledgment rejection. Native
source remains `dbd12c8`; its 881 active Rust tests and locked release are unchanged.

A fresh internet run now independently verifies **steps 0-15**: 24 course
arrivals, eight intermediate personal arrivals, four actual jumps, one native
pull-up, two timed holds, two ordinary lever activations and one partner yield.
It compares **2,997 matching player/world/native states**, ending at tick 24945
with both players alive at 89 health. Copied and current fixture hashes match.
Evidence:
`private/coop/hatter1-route-ee5c6bed594b480faf570313031ba090/completed-prefix-audit.json`.

The requested 0-17 run fails at step 16 and remains marked failed in its original
observations. The first player repeatedly replans on the rotating great gear;
the partner is far away, so this is not another personal-space blockage. The
scoped audit excludes that attempt. Gear-relative arrival/waiting and continuous
crossing still need verification. The normal checkpoint before step 14 is
available for a shorter resumed reproduction; resumed evidence must remain
separate from fresh traversal. No complete campaign visit is added.

## Rotating gear arrival and waiting (4 November 2026)

The ordinary-input test driver now observes the lowered gear's tangential carry
velocity and compensates while approaching or holding a world-space waypoint.
Arrival checks use total world motion; reaching the waypoint keeps the ordinary
counter-motion input active. Actual collider support gates this observation.
The shared machinery controller retains its normal movement, collision rollback
and saved state. Its angular speed is unchanged.

Two new Rust regressions cover delayed arrival/holding and the distinction
between standing still relative to the gear and standing still in world space.
Eight staged native-map cases cover both rider orders with observation and
input delays of 0, 12, 24 and 45 ticks per direction. They require sustained
supported arrivals without jumps or overlap. These component setups are not
evidence of fresh route traversal.

Source `623fece836f009f75f550c3eea90434e16a4000445315d38197a32d6a4e5b909`, executable SHA-256
`e7130c982c5c8e7db0c2d4d3f0b793d1681b6b8ab0755e22d8ae9a55c218703a`, passes
883 active Rust tests with one ignored, the locked release, all 68 native paired
machinery/door checks and the unchanged single-player 11+1 baseline. Earlier
scene checks retain their original build attribution.

The Python runner can resume an unchanged ordinary checkpoint with an exact
native restore receipt. Its first resumed attempt lets Blue reach and hold the
gear waypoint, but Red's walking plan enters the gap without taking off and
Red dies. That failure is retained in
`private/coop/hatter1-route-6522f5134dd74472b59af253eac60c5c`.
Boarding now uses an observed supported approach, an ordinary short run-up and
one jump per player. Verification requires each command acknowledgment, actual
host-recorded takeoff, airborne observation and personal grounded arrival.
All 43 Python guards pass, including missing-partner, wrong-footing, missing-
takeoff and wrong-command rejection.

The next internet run, with 40 ms delay and 20 ms jitter configured, independently
verifies **resumed steps 14-16**: four personal course arrivals, two actual gear
jumps, one timed hold and **447 matching player/world/native states**. The
starting checkpoint is byte-for-byte unchanged and its native restore is exact
at tick 23083. Blue holds its fixed waiting point for 1,095 observed ticks while
Red boards, with maximum horizontal error 2.753 units across 48 observations.
Both remain at 89 health. The scoped audit ends at tick 26721; it does not claim
a fresh run or a complete visit.

Evidence:
`private/coop/hatter1-route-27e4e2f0224e41a5bbf6c1888138af2d/completed-prefix-audit.json`.
The original observations still mark the requested 14-17 run failed: the next
exit waypoint times out with both alive and repeated planning. The driver drops
worker results when the body has moved 16 units from the planning start, and
currently gives neutral movement while waiting for such a result. Rotating carry
can therefore invalidate the starting position. Holding a supported planning
position and checking world-relative settling is the next investigation; that
fix and successful exit traversal are not yet verified. No new campaign credit
is added.

## Stable route planning and gear exit (4 November 2026)

The test driver now holds a world-space starting point with ordinary movement
while its route worker runs. On a carrier, readiness requires settled world
motion; own velocity alone is insufficient. This fixes repeated rejection of
plans whose starting body had drifted with the gear. The existing 16-unit
stale-start rejection is unchanged. A new command, relocation, expired input,
scene or accepted plan clears the held start; losing support also clears it.

Two new Rust regressions check delayed settling and continued holding of one
valid start, plus support loss and reacquisition. At 0, 12, 24 and 45 ticks of
observation/input delay per direction, a 30-second simulation requires at least
ten seconds of holding and stays within the stale-start limit throughout the
period after first readiness. The initial shorter test did not allow that much
stable time at the longest delay; its failure is retained.

Source `cf5d5511069355b2f0f1c096174c596db789e4daf5518ccb991eb6ab53c9ca89`, executable SHA-256
`a9861fdb8da6bf11e9413f77376045cd5ef16c4529c5dd2b73f65dd14970cd62`, passes
885 active Rust tests with one ignored, the locked release and all 68 paired
native cases. The single-player 11+1 outputs still match the baseline exactly.

The first resumed internet attempt accepts both exit plans; Red reaches the
exit, but Blue loses the rotating support on a walking route and dies on the
floor. That whole run remains failed in
`private/coop/hatter1-route-e32215f5d76e4bce8258c98f239c3f2f`.
It creates a normal checkpoint before step 17, enabling short exit retries.

The exit fixture now returns each player to the previously verified gear
waypoint and uses a short ordinary westward run-up and jump. Its verifier checks
both players' acknowledged takeoff footing, authoritative jump events, airborne
observations and personal grounded arrivals. The waiting partner holds its
observed world position on the gear. All 50 Python guards pass, including
missing-partner, missing-takeoff, wrong-command and wrong-floor rejection.

A resumed step-17 internet run with 40 ms delay and 20 ms jitter passes both
actual exit jumps and personal arrivals, comparing **245 matching player/world/
native states**. Both remain at 89 health. The starting checkpoint is unchanged;
the native exact restore is verified at tick 26704. The normal checkpoint before
18 is retained at tick 28713. Copied and current fixture hashes match.
Evidence: `private/coop/hatter1-route-6345aaf2de8b476eabf0b34a938cd817`.
This proves the resumed exit segment, not fresh traversal or a complete campaign
visit. Full Clockwork, recovery, reconnects and later machinery remain open.

The following fresh run physically reaches step 21, where the stair tread has
no supported waiting neighbor. Its audit separately catches an unretained
step-17 duplicate command acknowledgment: the guest accepted command 142 at
tick 27568, but the one-row-per-tick history still contained command 141.
The original record is unchanged. The strict scoped audit therefore credits
only **fresh 0-16**: 26 course arrivals, eight intermediate arrivals, four
entrance and two gear jumps, one pull-up, two holds, two levers, one partner yield
and **3,318 matching states**, with both at 89 health.
Evidence: `private/coop/hatter1-route-4287b3e171a547dc87a12a8eb752d3f5/completed-prefix-audit.json`.
The separately verified resumed step-17 evidence above remains valid.

Clockwork arrivals now wait until the command acknowledgment and reached flag
are present in the retained observation for the same authoritative epoch/tick.
They do not overwrite history or weaken the final verifier. The stair fixture
centers the first player and walks to the next authored footing before bringing
the follower to the original target. This is limited to steps 21-23, with
bounded ordinary walking targets and separate arrival/order receipts. The
first stair retry passes 21 but fails at 22 when it still attempts a sideward
neighbor; the fixture now uses the next tread throughout the short flight.
All **59 Python guards** pass, including the captured acknowledgment case and
stair handoff ordering. Native source remains `cf5d551`.

The next Anode internet run independently verifies **resumed 18-24**: 12 personal
arrivals, three stair handoffs, one timed hold and **454 matching player/world/
native states**, with both at 89 health. The unchanged starting checkpoint is
restored exactly at tick 28713; the scoped audit ends at 32526. Copied and current
fixture hashes match. Evidence:
`private/coop/hatter1-route-2f6496a9b42348c7ac8e21ec3a0bc75e/completed-prefix-audit.json`.
The original 18-38 request remains failed at 25 because the first arrival has no
suitable corridor waiting position. A normal checkpoint before 25 is retained
at tick 32620 for the next short reproduction. No complete visit is credited.

## Corridor handoffs and delayed landing (4 November 2026)

The corridor fixture now moves the first arrival to the next authored navigation
footing before the follower approaches. This is limited to seven corridor
waypoints, with bounded spacing and height differences; it cannot skip a lever
or wait. Each forward move needs its own grounded arrival and command receipt.

On native source `cf5d551`, the resumed **25-35** section independently passes
16 personal arrivals, seven corridor handoffs, two timed holds, one lever and
**1,073 matching player/world/native states**. Both survive at 89/100 health.
The normal checkpoint restores exactly at tick 32620; the audit ends at 41363.
Copied and current fixture hashes matched at audit time. Evidence:
`private/coop/hatter1-route-438d88376355413d9e777c0a69842a4e/completed-prefix-audit.json`.
The longer 25-38 request remains failed at 36 because the jump approach has no
suitable waiting neighbor. This scoped result does not prove a complete visit.

The fixture now groups gap steps 36-42 into one sequential paired crossing.
Each player must reach the approach, hold for the original preparation time,
perform a real jump and land with a personal acknowledgment. The first crosses
and moves to the next authored landing-side footing before the second approaches.
The crossing uses feedback air steering; it does not claim identical playback
of the original single-player timed inputs. Its live verification is pending.
All **72 Python guards** pass, including corridor handoff ordering, partial gap
sequence rejection and supported preparation holds.

The first 33-42 retry fails before the gap: Red overshoots the step-33 platform
while Blue waits safely ahead. Evidence is retained unchanged in
`private/coop/hatter1-route-e8f07a7ea14c42c88c5b548edf50088a`.
The driver's final landing control waited for a delayed observation of downward
velocity, so braking arrived too late. It now predicts whether ordinary gravity
will make the player descend by the time the input arrives, retaining the
existing wall, floor-support, distance and height checks. Updrafts are excluded
from that prediction. Gameplay movement physics is unchanged.

A captured-motion regression reproduces the fall with the previous handoff
timing, then requires supported settled landings with 0, 6, 12 and 18 ticks of
observation and input delay per direction. All **886 active Rust tests** pass,
with one ignored. Source
`fdbfbde9620c22756b1600b59f2aed1594d87b7696ddca0520f49e1db5431799`, executable SHA-256
`70710270621f99e36fd82d97a172c7d5e5e344debebdac1769d74fdf796c5442`, passes
the locked release and all 68 paired native cases. The single-player 11+1
outputs still match the baseline exactly.

The live internet retry verifies **resumed 33-34** with four personal arrivals,
two corridor handoffs and **237 matching player/world/native states**, ending
at tick 40554 with 89/100 health. The unchanged checkpoint restores exactly
at tick 38760; copied and current fixture hashes match. This includes the
previously failed step-33 landing. Evidence:
`private/coop/hatter1-route-e35d66df3dc64b1abfbe7982c9238d2c/completed-prefix-audit.json`.
The requested 33-42 run remains failed at 35: Red misses the supported landing
during a different jump approach while Blue waits safely at 36. The gap
sequence is still untested in the live game. The failure and original
observations are retained; this partial retry does not complete Clockwork or
add campaign credit.

## Supported remote landing plans (4 November 2026)

The step-35 failure reproduces headlessly in the actual map: the captured
route lands with its original input timing, but extending one northward air
input by six physics ticks (50 ms) misses the platform. Its nominal first
contact barely catches the platform with the side of the body. Later walking
onto the platform hides that fragile contact in the planned endpoint.

Remote diagnostic plans now check support at the first airborne-to-grounded
transition in each candidate, using the center and four eight-unit offsets.
The existing supported-walking criterion is shared by that check. Zero-margin
single-player planning is unchanged. A synthetic regression distinguishes
marginal first contact from the later safe endpoint and checks the new plan.

The native Clockwork suite retains the old route as a negative comparison,
then runs the newly searched route with each input segment shortened, unchanged
and lengthened by six ticks, at 0, 6, 12 and 18 ticks of observation and input
delay per direction. All **60 variants** remain above the landing floor and
settle at the destination. These are staged component fixtures, not continuous
paired traversal evidence.

Source `6c7c12dc145ce999ea440dbf2d94c2d5544dbc0ef11e0a4461afe8d5614ecd7e`, executable SHA-256
`cab2b3023290114231943f50b7599e99c8ddc66ba9fa1f0f8ae806e179e62313`, passes
**887 active Rust tests** with one ignored, the locked release and all **69
native paired/driver checks**. The single-player 11+1 outputs still match the
baseline exactly.

The independent-client retry verifies **resumed 33-35**: six personal arrivals,
three corridor handoffs and **271 matching player/world/native states**, ending
at tick 41035 with 89/100 health. The unchanged checkpoint restores exactly at
tick 38760; copied and current fixture hashes match. Evidence:
`private/coop/hatter1-route-c45a934cfbfe4b75b3f497eb6aa29dce/completed-prefix-audit.json`.
Blue then crosses the gap and reaches the landing-side waiting point. Red fails
on the approach to step 36, where premature final-landing control clears a
second planned jump from an intermediate sinking pillar. The original 33-42
request remains failed, and the paired gap has not passed.

## Retaining consecutive jumps (4 November 2026)

Final landing control now waits until the route has issued every planned jump.
An unissued jump at the front of the plan also blocks that handoff. This keeps
a supported intermediate island from causing the driver to discard the next
takeoff while steering toward the final destination.

The first headless reproduction held the geometry still and did not reproduce
the failure. The corrected case advances the actual paired Clockwork machinery,
including its sinking pillars, and delays both player and collision-world
observations. The old handoff loses the second jump and falls. The new handoff
performs both actual jumps and settles at the goal with 0, 6, 12 and 18 ticks
of observation and input delay per direction. This is a staged component check.
An optional ignored Rust test runs the same native-map case when
`LOOKING_GLASS_TEST_DATA` points to the original game data; the release native
suite also includes it. All **887 active Rust tests** pass, with two ignored.
The native-map test was separately run and passed.

Source `628eba9be19a2e593aa504a87c7c071b6b3c14907f59344cb36cf026cb92d87e`, executable SHA-256
`edc13ae3e410705c5d0736f7914bf1165015b4d73f28b0533ec22985a3968afc`, passes
the locked release and all **70 native paired/driver checks**. The single-player
11+1 outputs still match the baseline exactly.

The resumed live attempt independently verifies both personal approaches to
36/37 and both preparation holds: four arrivals, two holds, Red's two actual
approach jumps and **177 matching player/world/native states**. The unchanged
checkpoint restores exactly at tick 41127; the audit ends at 42717 with
100/98.867 health. Copied and current fixture hashes match. Evidence:
`private/coop/hatter1-route-6f3d23c11fad42819a6b019811cc8fea/approaches-audit.json`.
Blue crosses the main gap and waits beyond it. Red takes off but slows too soon
over empty space, falls short of the far bank and dies. The whole 36-42 bundle
remains failed; the approach audit does not mark any bundled course step or
campaign visit complete.

## Direct airborne steering (4 November 2026)

Direct waypoint steering now uses an air-acceleration stopping-distance rule
while an ordinary jump remains over unsupported space. Above a supported
landing it resumes the existing damped approach. Grounded, swimming, ledge and
updraft steering keep the existing rule, and explicit speed limits still apply.
Movement physics and jump strength are unchanged.

A native-map regression performs the original short run-up and one actual
jump, with paired machinery and delayed player/world observations. The old
grounded-arrival rule reproduces the shortfall. The new steering settles at
the far waypoint with 0, 6, 12, 18, 24 and 30 ticks of observation and input
delay per direction, covering up to 500 ms combined delay. This includes the
roughly 418 ms lead recorded in the failed live run. All **887 active Rust
tests** pass with three ignored; the optional native gap test was run separately
and passes all six delays.

Source `a07517e9ff6621d46dd14d084bf1708c8ec1d4de64f28116a1a26623d047a305`, executable SHA-256
`ee9b04beb5c2def5c9b0b4cf1902e855b9bbd84d5d2d0c5c04231d32e80448fc`, passes
the locked release and all **71 native paired/driver checks**. The single-player
11+1 outputs still match the baseline exactly.

The independent-client internet retry, with 40 ms delay and 20 ms jitter
configured, passes **resumed 36-42**. It verifies six personal arrivals, both
actual gap jumps, both preparation holds, the first player's separate waiting
handoff and **232 matching player/world/native states**. Both finish alive at
100/99.867 health. The unchanged checkpoint restores exactly at tick 41127;
copied and current fixture hashes match. The runner saves an ordinary checkpoint
before 43 at tick 42953 and ends observation at 42957. Evidence:
`private/coop/hatter1-route-6d83c0428b584f6d956209de30da932f`.
This completes the resumed gap sequence, not fresh Clockwork traversal or a
campaign visit. The later pathnode course, machinery, scenes, recovery and
reconnect checks remain open. Overall completion stays 48.6%.

## Independent pathnode resolution (4 November 2026)

The paired runner now supports authored pathnode steps. A neutral local query
in each client resolves the identified `info_pathnode` onto that client's
collision world, using the same body-footing query and bounds as the
single-player route. Its report records the entity, original position, resolved
feet, command, epoch and sample tick. It supplies no movement, activation or
arrival claim. Legacy marker commands retain their behavior.

The runner requires both retained projection acknowledgments and matching
footings before converting that course step to an explicit navigation goal.
It preserves the original course and both native reports alongside the updated
effective course hash. Each player must still earn a personal grounded arrival
and the first must provide a separate waiting handoff. Invalid entities,
unsupported origins, mixed movement/progression queries, missing partner
reports, unretained acknowledgments and mismatched footings are rejected.

Source `6f589de8d7a765146dfb57b2a2a2bbb9f487566d4752182dd2edce0874994fa1`, executable SHA-256
`a36d138a1fec2e6ea1dfd7803f848ad5f34cc8aaa1f689cac63eada01567b52c`, passes
**889 active Rust tests** with three ignored, the locked release, **82 Python
guards** and **72 native paired/driver checks**. The added native case resolves
all sixteen post-gap pathnodes to clear, supported footings in the prepared
level. The single-player 11+1 outputs remain exactly baseline-equivalent.
The independent-client internet run, with 40 ms delay and 20 ms jitter,
completes **resumed 43-51** and then fails selecting a waiting position at 52.
Its original failed result is retained. A separate scoped audit of 43-51
verifies 18 personal arrivals, 16 independent pathnode projections, nine
waiting handoffs and **2,634 matching player/world/native states**. Both finish
alive at 88/100 health; the starting checkpoint restores exactly at tick 42953.
The audit loads the retained, hash-checked fixture versions. Evidence:
`private/coop/hatter1-route-4b64211837de463da40f441361c21697/completed-prefix-audit.json`.
This does not complete Clockwork or extend the fresh-run evidence.

The failed parking selection required a short sideways detour before moving
farther from the shared waypoint. The runner now permits up to eight observed,
collision-checked neighbor steps, reserving the incoming player's space and
avoiding previously visited positions. Intermediate detours never count as a
completed waiting handoff. All **85 Python guards** pass, including the captured
corner, loop rejection and partner-space reservation.

The first 44-59 retry stops at 46 when the host session closes and the guest
reports that closure. Neither log contains a crash message; both driver commands
have `finish=false`. That failed result remains retained at
`private/coop/hatter1-route-937a08ae2a92485b9f3176899f3aaf79`. The runner now
records process exit codes before cleanup to distinguish future closures.

The repeat on the same native build passes **resumed 44-59**, using independent
internet clients with 40 ms delay and 20 ms jitter. The unchanged ordinary
checkpoint restores exactly at tick 43797. It verifies **32 personal arrivals**,
**32 independent pathnode projections**, **16 completed waiting handoffs** and
**3,565 matching player/world/native states**. At 52, the first Alice takes
three supported walking steps, including the lateral detour, before the second
arrives; no recovery warp or state write is used. Both finish alive at 88/78
health. Copied and current fixture hashes match. Evidence:
`private/coop/hatter1-route-7934840f90554821bb7e0525b89058a9`.
The ordinary checkpoint before 60 is saved at tick 76618. Later machinery,
scenes, recovery, reconnects and full traversal remain open; the overall estimate
stays 48.6% with six combined campaign visits verified.

## Watched Hare and Port handoffs (4 November 2026)

The route runner now observes both clients through these two original scenes.
It checks the actual moving player or lever user against the shared owner,
native actor and camera, hides/freezes the partner throughout, and preserves
each player's resources. Watched Hare playback must retain all twelve dialogue
cursors and its opening action; Port must run its full 14.75 seconds. A completed
handoff requires matching new reunion stamps, living supported bodies, clear
collision, separation, nearby placement and the partner visible again.

An approach interrupted by Hare ends at the native reunion. Its completed scene
is recorded separately from ordinary personal waypoint arrivals. If the first
player had reached the waypoint and was beginning a waiting move, that unfinished
move must name an observed supported neighbor and cannot claim a parking arrival.
The original single-player trigger relocates the actor; returning to a stale
waiting position after that transition was an incorrect runner action.

The original resumed 60-67 run watches Hare with Red and verifies its shared
reunion, then times out while trying to return to the pre-scene waiting point.
The failed run remains retained. A separate scene-only audit uses its frozen,
hash-checked fixture and verifies **1,791 matching player/world/native states**,
with both players at 88/78 health. It credits no completed course step. Evidence:
`private/coop/hatter1-route-629599a73df44282b6b1c2751eb4bd67/hare-scene-audit.json`.

With the explicit scene handoff, **resumed 60-67 passes**. Blue triggers and
watches Hare; Red uses the actual Port lever and owns its delayed scene. The run
verifies nine personal arrivals, four independent pathnode projections, four
completed waiting handoffs, two timed holds, one lever, both scene handoffs and
**2,971 matching player/world/native states**. The unchanged checkpoint restores
exactly at tick 76618. Both finish at 88/78 health, and a normal checkpoint before
68 is saved at tick 97795. Evidence:
`private/coop/hatter1-route-93d2ea687faf4849837d2a385b272232`.

A targeted repeat from its unchanged normal pre-lever checkpoint verifies
**resumed 66-67 with Blue using the lever and owning Port**, with **378 matching
states** and the same personal health. The `--port-player` option selects which
client sends ordinary use input; it does not change authoritative state. Evidence:
`private/coop/hatter1-route-0e01d36e4dfc4dbc836f743d69da480b`.
The preceding attempt failed before gameplay when the new invite's internet
address could not be resolved after the built-in retries. That startup failure
is retained at `private/coop/hatter1-route-a4801fca5b374daeb7bcd5f2433a4438`;
success with a new session does not prove that transient DNS issue fixed.

All runs use independent internet clients with 40 ms delay and 20 ms jitter.
Native source stays `6f589de`; its existing 889 active Rust tests, release,
72 native checks and exact single-player 11+1 baseline remain attributed to that
unchanged build. **110 Python guards**, seven source-package checks and the source
audit pass. Frozen fixture versions accompany each result. Both watched scene
owners now have live evidence; live skipped scenes, recovery/reconnects, later
machinery and complete fresh traversal remain open. Overall stays 48.6%.

## Raised-path combat and return route (4 November 2026)

A resumed 68-108 attempt completes 68-76, then Red dies during combat at 77.
An Automaton's punches move her off the raised path onto the damaging floor.
The original failure remains retained. A separate audit of only the completed
section verifies 16 personal arrivals, 12 pathnode projections, eight waiting
handoffs, one timed hold and 1,237 matching player/world/native states, ending
at 88/66 health. Evidence:
`private/coop/hatter1-route-535e9d49adb344e1bbb1e6fa4ae9cd4a/completed-prefix-audit.json`.

The opt-in combat controller previously accepted body clearance and predicted
height while Alice could have only marginal support at the edge. It now requires
grounded support across the full stance throughout its delayed movement and
braking prediction. A regression runs ordinary movement with observation/input
delays of 0, 12 and 24 ticks and still requires useful movement. It fails on the
old controller at tick 248 with no delay, then passes with the correction.
Enemy attacks, damage, player movement physics and progression remain unchanged.

The new frozen build `894292f` passes **resumed 68-81** through two independent
internet clients with 40 ms delay and 20 ms jitter. The unchanged starting
checkpoint restores exactly at tick 97795. Both clients verify 26 personal
arrivals, 16 pathnode projections, 13 completed waiting handoffs (23 walking
steps), one timed hold and **2,294 matching player/world/native states**.
Both remain alive without respawning, ending at **16.53/100 health**. The low
remaining Blue health is retained, not reset. An ordinary checkpoint before 82
is saved at tick 114340 for continuing the route. Evidence:
`private/coop/hatter1-route-e129882e7deb47c0a8e9b4c68b6f3b0b`.

Validation: **890 active Rust tests** pass (three ignored), the locked release
build passes, all **72 native paired/driver checks** pass, and the single-player
11+1 results match the original baseline exactly. All **110 Python guards** and
seven source-package checks pass. Frozen fixture hashes match the tested files.
This is a completed resumed section, not complete Clockwork traversal or another
verified campaign visit. Later scenes, transports, recovery/reconnect repeats
and fresh traversal remain open; overall completion stays **48.6%**.

## Return stairs and room combat (4 November 2026)

The paired runner now uses the next authored footing when the return stairs
(81-85) or winding path (100-107) cannot offer a sideways waiting position.
Each player still earns its own waypoint arrival. Before the follower arrives,
the leader must reach the next point through ordinary movement; the verifier
checks its command, grounded position and ordering. These handoffs cannot skip
an action, exceed the bounded horizontal distance or cross an excessive height
change. Earlier laboratory-corridor limits remain unchanged.

On the static return route, a waiting player can use the existing combat dodge
controls instead of repeatedly walking back into punches. A substantially
healthier nearby player can lead the approach, with an observed walking yield
if needed. The yield selector reserves 48 units around both the stopped player
and its route; its previous 64-unit body margin rejected an observed room-corner
step despite sufficient clearance. Native 36-unit personal space and the
continuous overlap checks still apply throughout actual movement.

The native test controller also bounds its close-melee approach to one second.
If the target stays outside the Blade's actual hit volume, the ordinary throw
policy becomes available. Reaching melee range renews the preference, while a
new target or visit receives its own approach window. A regression covers
out-of-reach targets, repeated snapshots, actual melee reach and a new visit.
No enemy, weapon, damage, resource or player-movement rule changes.

Two separately audited completed sections retain their original longer-run
failures. On source `894292f`, **81-93** verifies 26 personal arrivals, six
pathnode projections, five stair handoffs, eight waiting handoffs and **2,421
matching player/world/native states**, ending at 46/85 health. That requested
81-108 run then fails in the room fight at 94. Evidence:
`private/coop/hatter1-route-e95506092c354f50bebfa6e4d40f7974/completed-prefix-audit.json`.
On source `cefabf5`, **91-103** verifies 26 personal arrivals, 16 projections,
13 waiting handoffs and **3,181 matching states**. Both players clear the room
fight and finish this section at 100 health through normal pickups. Its requested
91-108 run stops at 104 because the narrow path has no sideways waiting point.
Evidence:
`private/coop/hatter1-route-e6d13cb4efad4a59a6b18610fa8b67d3/completed-prefix-audit.json`.

With forward handoffs across the full winding path, **99-106** verifies 16
personal arrivals, two projections, seven forward handoffs, one waiting handoff
and **1,845 matching player/world/native states**, ending at 100/100 health.
The unchanged checkpoint restores exactly at tick 142280, and the scoped audit
ends at 155535. The requested 99-108 run still fails at 107: the leader reaches
`[-4937.065, 1613.450, 256.03125]` while approaching the next waiting point at
`[-4928, 1600, 256.03125]`, then remains grounded and nearly stationary with the
controller reporting `settling landing`. The follower stays at its preceding
footing. Both remain alive; the timeout is retained and neither 107 nor 108 is
credited. The next investigation is the short-range landing handoff near this
geometry. Evidence:
`private/coop/hatter1-route-85ba532924904603b949c05569f4aa24/completed-prefix-audit.json`.

All runs use independent internet clients with 40 ms delay and 20 ms jitter.
Frozen fixture versions accompany each result. Source `cefabf5` passes **891
active Rust tests** (three ignored), the locked release and **72 native
paired/driver checks**. Single-player 11+1 results remain identical to the
baseline. All **116 Python guards**, seven source-package checks and the source
audit pass. Later lifts, scenes, recovery/reconnects and complete traversal remain
open. These resumed sections do not complete another campaign visit; overall
completion stays **48.6%**.

## Rotating doorway approach (4 November 2026)

The short grounded landing handoff previously discarded navigation solely
because the goal was close, even when a wall blocked direct walking. It now
requires body clearance, including the existing 18-unit step-up path and
overhead clearance. Regressions reject a tall wall, actually walk up a normal
12-unit step and reject that step under a low ceiling. Movement physics and
arrival tolerances remain unchanged. A horizontal-only intermediate correction
failed the live rising-floor approach; that unsuccessful replay is retained.

The actual doorway uses rotating leaves 92/93. A diagonal approach can block
their swing. The paired runner now uses the authored centered footings from
116 and 109 before returning to the original 108 target. It verifies both
walking arrivals, a 204-tick opening hold, both fully open native leaves and
no warp or respawn. It still requires both players to reach the original
107/108 waypoints separately; this adaptation does not skip those arrivals.

Frozen source `11c5ba2` passes **resumed 107-108** through independent internet
clients with 40 ms delay and 20 ms jitter. The unchanged checkpoint restores
exactly at tick 153910. Evidence verifies four personal arrivals, two centered
door approaches, one forward handoff, one waiting handoff and **692 matching
player/world/native states**. Both finish at **100/100 health**. The ordinary
checkpoint before 109 is saved at tick 158807. Frozen fixture hashes match the
tested files. Evidence:
`private/coop/hatter1-route-75eca5c73ada4bf296c2eda6d122413d`.

Before this fix, source `cefabf5` repeated **99-106** successfully with 16
arrivals and 1,652 matching states to save the closer checkpoint. The three
subsequent doorway failures are retained separately; none receives traversal
credit. The final source passes **893 active Rust tests** (three ignored),
the locked release, **72 native paired/driver checks** and the exact
single-player 11+1 baseline. All **124 Python guards** and seven source-package
checks pass. Later Clockwork scenes and traversal, full fresh/recovery/reconnect
repeats and the remaining campaign visits stay open. Overall remains **48.6%**.

## Gryphon ownership and return traversal (4 November 2026)

The paired runner now handles the original Gryphon contact at 112. The first
arriving player owns the scene; `--first` can select either ordinary approach
for verification. The verifier requires all seven dialogue cursors, the
5.5-second opening, the triggering movement command, a hidden and frozen
partner, unchanged personal resources during the scene, and a safe shared
reunion. Scripted reunion remains separate from personal walking arrivals.

On unchanged frozen source `11c5ba2`, **resumed 109-116** passes with eight
personal arrivals, four completed waiting handoffs (seven walking steps),
three timed holds and **1,353 matching player/world/native states**. Blue owns
the watched Gryphon scene. A separate **112-113** replay verifies Red ownership
and **960 matching states**. Both finish at **100/100 health**. Each uses two
independent internet clients with 40 ms delay and 20 ms jitter. Evidence:
`private/coop/hatter1-route-0ee347bc33704a6eabfa90b5599f27dd` and
`private/coop/hatter1-route-cf697d08473748d18bf2b66affa01b89`.

The subsequent requested **117-136** run stops at 128 after the leader reaches
its target: the observed supported neighbors do not offer a waiting position
clear of the approaching partner. Both rotating leaves 4/5 are already fully
open. The completed **117-127** section is separately audited, with 22 personal
arrivals, 20 pathnode projections, 11 waiting handoffs and **1,514 matching
player/world/native states**, ending at tick 178937 with 100/100 health. Its
ordinary checkpoint before 128 is saved at tick 179050. The failed step remains
uncredited. Evidence:
`private/coop/hatter1-route-4d6d768226d74281978d40ea93c4dabd/completed-prefix-audit.json`.

All **131 Python guards** pass. The exact native build also passes **40 staged
paired scene cases**: both owners, watched/skipped cinematics, living/dead
partners, pause/restore and ordinary speech. Those component fixtures do not
substitute for actual traversal. Later return movement, portals, the final
scene, recovery/reconnects and complete-route repeats remain open. No new
complete campaign visit is credited; overall completion remains **48.6%**.

## Return doorway and first portal (4 November 2026)

The narrow doorway at 128 now retains its 204-tick opening hold, observes both
rotating leaves 4/5 fully open, and walks the first player to the authored
forward footing before the partner approaches. Both original waypoint arrivals
and the later original wait remain required. On source `11c5ba2`, the requested
128-136 run completes **128-135** before failing at the next waiting arrangement.
Its frozen, separately audited completed section verifies ten personal arrivals,
four ordinary waiting handoffs, one door handoff, three timed holds and **562
matching player/world/native states**, ending at 100/100 health. Evidence:
`private/coop/hatter1-route-8b6c5b5fb21d4db2b66c7084acf1b4df/completed-prefix-audit.json`.

At 136, parking backwards on the stairs obstructed the following player. The
paired runner now lets each player separately approach and use the original
jump input to climb through the first portal. The leader must walk to an
observed supported point clear of the destination before the follower crosses.
Both clients must observe each personal warp and safe landing. A scene or
revival cannot substitute for the crossing.

A native regression reproduced another overlap case: generic teleport contact
could put the incoming Alice directly inside a living partner because ordinary
movement separation had already run. The portal now waits while that destination
is occupied, retaining personal resources and the existing cooldown. Once clear,
ordinary contact can retry. Both player identities, dead or separate partners,
and a solid destination are covered by the regression tests.

The first live portal replay on source `2f6cef0` reached both destinations but
failed verification. Its original jump input correctly caused a native climb,
which does not increment the jump count. It also exposed a separate diagnostic
controller bug: a consumed jump was rearmed at teleport, producing an unwanted
second jump. The verifier now requires actual jump input and a native jump or
climb, and rejects a repeated destination jump. The controller keeps a consumed
jump through relocation; new commands and new visits still rearm it. The retained
failed run demonstrates the repeated jump and remains uncredited:
`private/coop/hatter1-route-68afaaa0cba2449694c48181e302d019`.

Frozen source `04d3b0c` passes **resumed 136-137** using the unchanged ordinary
checkpoint at tick 183151, two independent internet clients, 40 ms delay and
20 ms jitter. It verifies two source arrivals, two input-driven native climbs,
two personal portal crossings, one waiting handoff and **202 matching
player/world/native states**. The players finish at **80/100 health**: Blue
takes two hits while grounded at the waiting point after crossing. No death,
revival or resource reset occurs. The normal checkpoint before 138 is saved at
tick 184650. Frozen fixture hashes match the tested tools. Evidence:
`private/coop/hatter1-route-2986a1557db14f268352b96cc9bd0e50/result-summary.json`.

The following requested **138-143** segment times out at 138, with no completed
waypoints. Red's approach observes a ledge and requests a pull-up, but the
receipt never reaches completion; Red ends on the lower floor. Both remain
alive at 80/90 health. The failure and unchanged checkpoint are retained for
investigation; this run adds no traversal credit. Evidence:
`private/coop/hatter1-route-6c4437f598384236a4fde90d9d17fbf9`.

All **897 active Rust tests** (three ignored), the locked release, **72 native
paired/driver checks**, the exact single-player **11+1** baseline, **40 staged
scene cases**, **151 Python guards** and seven source-package checks pass.
Later chair traversal, the second portal, final scene/exit and complete fresh,
recovery and reconnect repeats remain open. This is partial visit evidence;
overall completion remains **48.6%**.

## Observed chair pulls and remaining waiting failure (4 November 2026)

The failed first-chair replay exposed a diagnostic input timing problem: the
planned pull arrived before the remote body's actual catch, and the following
away input dropped the ledge before the observed pull could arrive. Opted-in
chair routes now stop at the first simulated catch, keep approaching until the
real catch is observed and wait for an explicit ordinary pull input. If the
native full-body pull destination is obstructed, ordinary sideways inputs move
the grip toward the planned clear position. The original ledge physics,
collision and timing remain unchanged; its existing clearance query is exposed
read-only to the diagnostic report.

Native regressions use actual player physics with delayed observations and
inputs. They require a completed climb and clear grounded landing, including
an initially obstructed pull that needs a shimmy. Lower and opposite goals keep
their original routes; the handoff is explicitly enabled only for planned
chair approaches. The first live attempt with the new observation protocol
completed Red's climb, but the fixture kept walking under the consumed pull
command and Red died on the lower surface. That failure remains retained at
`private/coop/hatter1-route-06dd0794413a454cad79c0497d044647`.

The fixture now validates the pull as soon as its own native climb count
increases and the player lands, then issues a fresh planned command to the
original goal with normal combat enabled. It records the climb's actual ending
tick separately from waypoint arrival. A guard reproduces the stuck command
with the retained old fixture for both players and passes with the correction;
grounding alone or the partner's climb cannot complete the receipt.

The next replay on source `99a66d6` restores the unchanged checkpoint at tick
184650 through two independent internet clients with 40 ms delay and 20 ms
jitter. Red completes the pull at 191629, then reaches the original first-chair
waypoint at 191896 and activates its native puzzle contact. A separate component
audit verifies the input, animation, completed pull, personal arrival, replicated
contact and **1,088 matching player/world/native states**. Both are alive at the
last retained observation, with 80 and about 73.73 health. Evidence:
`private/coop/hatter1-route-b5fd0e9a54f344099aa89ddd6f2a1b55/chair-component-audit.json`.

The requested paired 138-143 replay still fails at 138: after that first arrival,
no observed walking neighbor provides a waiting position. The report identifies
slime immersion on the lower chair surface, explaining the continuing damage.
The current fixture must find an ordinary safe departure before bringing the
partner up. This audit credits one player's components only; the second arrival,
completed paired step, remaining chairs and full visit are unverified. Failed
runs and their original observations remain unchanged.

Frozen source `99a66d6` passes **901 active Rust tests** (three ignored), the
locked release, **72 native paired/driver checks** and the exact single-player
**11+1** baseline. All **154 Python guards**, seven source-package checks and the
894-file source worktree audit pass. The earlier 40 scene cases were not repeated
on this diagnostic-only change. No further campaign visit is credited; overall
completion remains **48.6%**.

## First paired chair and dry departures (4 November 2026)

The chair fixture now requires both personal arrivals and ordinary jump
departures to previously observed dry approach footing. It records immersion,
the real host input/takeoff, flight and supported dry landing. The return target
must clear the partner and the arrival lane. A scene, rescue warp, revival,
missing personal arrival or unreplicated chair contact cannot pass.

The first attempt reached the chair with both players and returned Red safely,
but selected a later supported point partway up the chair for Blue. Approaching
that XY from above left Blue on the rim, unable to reach the lower target Z.
The original failed run is retained at
`private/coop/hatter1-route-e9b4f1b224e34a35ae1f3ccd876aa131`.
The selector now preserves the first eligible nearby dry approach footing.
Its new regression fails with the original selection and passes after the fix.

The next resumed **138** replay passes on unchanged native source `99a66d6`.
Two independent internet clients with 40 ms delay and 20 ms jitter record two
personal chair arrivals, one observed Red pull-up, two native jump departures,
the replicated puzzle contact and **1,828 matching player/world/native states**.
Both finish alive on dry ground with **67 and about 67.73 health**. No health is
refilled; normal damage during approach/departure remains. The ordinary checkpoint
before 139 is saved at tick 197099. Evidence:
`private/coop/hatter1-route-60c616cad3f741c0846d4a912e2abe6e/result-summary.json`.

All **165 Python guards** and seven source-package checks pass. Native source and
its previous 901 Rust, 72 paired/driver and exact 11+1 single-player checks are
unchanged. This is one completed resumed paired step, not a complete visit.
The later chairs, second portal, final scene/exit and fresh/recovery/reconnect
repeats remain open. Overall completion remains **48.6%**.

The following 139-143 replay completes **139** before failing at 140. Its frozen,
separately audited completed section verifies two personal chair arrivals, one
observed pull-up, two native jump departures and **1,741 matching
player/world/native states**, ending at **54/43.53 health**. The original failure
is retained; no third-chair component is included in this completed section:
`private/coop/hatter1-route-229f63cf853147c4ba7ad05695f1753a/completed-prefix-audit.json`.
Its normal checkpoint before 140 is saved at tick 209317.

At 140, Blue reaches the chair but her return lands about 33 units above the
selected approach footing. The final verifier already rejects that height
mismatch; the runtime now applies the same check before declaring departure.
Red's first delayed jump plan diverges from its predicted path and activates the
original rescue portal at tick 219344. A second search then reaches the fixture
timeout. Both remain alive at about **43/43.53 health**. The successful 139 audit
does not hide either later failure. The return target and remote jump approach
need further work; all **166 Python guards** pass, including the additional
landing-height rejection. Native source is unchanged and no full visit is credited.


## Landing feedback and third-chair arrival evidence (5 November 2026)

Source `84c8eee79cbf9d06ddff8d2207fdbd549b844b8c7bc62312e56e9a0c5bc74651`
adds opt-in feedback to the automated planned approach. After observing flight,
an off-course grounded landing clears the remaining timed inputs and requests
a route from the actual body. Recent spatial segments preserve correct delayed
landings. This changes the diagnostic controller, not player physics or damage.
A separate read-only geometry query finds nearby fully supported, dry return
footings reachable from above. The fixture still requires actual jump input,
flight and a clear dry landing within the existing XY/height tolerances.

The first replay restores the unchanged checkpoint before 140. Blue reaches
the chair and returns to the projected floor at about (-844, 2455, 64). Red's
incorrect landing is detected, and the stale plan stops before any rescue warp.
The next asynchronous search exceeds the old 90-second limit. Both remain
alive at 43/43.53 health, but no paired step is completed. Original evidence:
`private/coop/hatter1-route-f09ce06a269045bb918825453e6629b1`.

The following replay allows at most 180 seconds for the planned 140/141
approach, retaining the separate 30-second departure bound and all movement
requirements. Both players now reach the original 140 waypoint. The scoped
audit validates two personal arrivals, one Red pull, Blue's projected dry
return and both actual departure jumps, with **2,312 matching player/world/native
states**. Each client records one landing-feedback replan. Neither body uses
a rescue warp or revival before the arrivals. Evidence:
`private/coop/hatter1-route-e43ccf84de0b4b99b198cbc20ef89c80/chair-component-audit.json`.

This second replay still fails. Red has about **8.53 health** when she reaches
the chair, following ordinary damage on the approach, including slime beside
the cushion. Her return jump lands dry at roughly (-878, 2489, 64), but its
643-unit landing speed invokes the original fall damage and kills her. Blue
retains 43 health. The final paired departure is absent, and the audit explicitly
records `paired_step_passed: false` with no completed steps. The failure remains
intact; a safer approach/departure or normal resource collection needs work.

The frozen executable SHA-256 is
`7e6f98c6041ec83e6d6b376634dcfaec6a2a44876c0a25519db978a118d9969c`.
All **905 active Rust tests** (three ignored), the locked release, **72 native
paired/driver checks**, exact single-player **11+1** baseline, **169 Python
guards** and seven source-package checks pass. The previous 40 scene cases
were not repeated for this diagnostic change. The remaining chairs, second
portal, final scene/exit, full-route and recovery/reconnect repeats remain open.
No additional campaign visit is credited; overall remains **48.6%**.


## Paired third chair with a held pull-up and rim jump (5 November 2026)

A first attempt to jump directly from the pulled-up rim still failed. The old
forward command continued after the animation, so Red left the rim before the
new jump reached the authoritative simulation. The jump count did not increase;
she fell into the slime beside the cushion. The failure is retained at
`private/coop/hatter1-route-0b7d76dee9cf40d4aab9fabd111cd68c`.

The fixture now releases movement as soon as the actual pull animation is
observed. A separate neutral command must be acknowledged during that animation
and remain active at its completion. Only then does a fresh jump target the
nearby cushion. Verification requires the native takeoff, airborne observation,
dry living body through the hop, original personal arrival and unchanged
warp/revival stamps. This changes ordinary automated inputs only; physics,
damage and the route's required arrivals are unchanged. Both players' old
controller paths fail the new regressions; all **173 Python guards** pass.

The next resumed **140** run passes from the unchanged checkpoint at tick
209317, on the same native source `84c8eee`. It records two personal arrivals,
one Red pull-up with a neutral hold, one actual rim hop and two ordinary return
jumps to dry floor. The peers have **3,004 matching player/world/native states**.
Both remain alive at about **42.93/6.53 health**. Red's earlier missed landings
still incur normal damage; neither a rescue warp nor revival replaces a required
movement. Evidence:
`private/coop/hatter1-route-4493a73e818948cfa5728ed66094cfb8/result-summary.json`.

The original observations SHA-256 is
`3aef303677220f4b3b65af31d9b632543d1b2e5ccd2e18ffc2502323304b78d6`.
The ordinary checkpoint before 141, at tick 230058, has SHA-256
`0bb79a3b908c3011327ad4ba0243052dee8882b1c73ff032d16e3c8b8b261349`.
All frozen fixture dependencies match the tested source. No native code changed;
the prior 905 active Rust tests (three ignored), locked release, 72 paired/driver
checks and exact single-player 11+1 baseline remain the relevant build evidence.
The fourth chair and later route, full-route repeats and recovery/reconnects
remain open. Red's low health is retained, not reset for the next test. This
resumed section adds no full-visit credit; overall remains **48.6%**.


## Fourth chair, individual revival and onward waiting (5 November 2026)

The paired runner can now explicitly verify recovery during chair steps 138-141.
An actual death interrupts that player's attempt and preserves its receipts
separately. The survivor continues with ordinary inputs while the native
revival timer runs. A revived player must repeat their own original approach
and dry departure. The unchanged recovery verifier checks exact timing, initial
health, preserved personal resources, clear supported placement near the survivor
and spectator/return cameras. Team wipes, waiting-partner deaths and unaccounted
revivals fail this scoped fixture; they are not silently accepted.

An initial fourth-chair attempt found no projected return floor near Red's own
observed approach. The selector now also considers native supported floor
queries near the living grounded partner, after exhausting the player's own
observations. It records that source and retains the same dry-footing, partner
clearance, actual jump and landing checks. The original failed run remains at
`private/coop/hatter1-route-08f59277d55e48d29b5ebb48b7a10c08`.

Resumed **141** then passes on unchanged native source `84c8eee`. Red dies at
tick 240518 and revives at 241718: exactly 1,200 active ticks, no paused or
safe-placement wait ticks, and 25 initial health. Her new approach starts at
241728; her personal arrival is observed at 258701 and dry return at 259132.
Both players' final arrivals and departures pass, with **4,229 matching
player/world/native states** and final health **26.93/23**. The closed native
trace has 1,438 records and passes replay verification. Evidence:
`private/coop/hatter1-route-7d4bf6e2591d4f54b5f31c1d6151083a/result-summary.json`.
Original observations SHA-256:
`811de4bd11e1c183b569694be35c401c0f2fd9972d2ebe9da3dda85e14acbb29`.

The first onward replay stalls at 142 because the waiting controller prefers a
forward pocket over an already usable side footing. The regression reproduces
that recorded geometry; the corrected choice preserves the existing clearance
requirements. Resumed **142-143** now passes with two personal arrivals, one
waiting handoff, the full five-second hold and **426 matching player/world/native
states**. Both remain at **26.93/23 health**, with no new death or revival.
Evidence:
`private/coop/hatter1-route-702a2dfdcecd46388706f29c5a0d8ca3/result-summary.json`.
Original observations SHA-256:
`76fcdc5bcff98af1bf11f7f48987809406f9520ab9b21fe4bb8e3ff6ac5da02b`.
The normal checkpoint before 144, tick 262159, has SHA-256
`c7396c87c58d6ab5d98ed7366fb8bf519e94a6ffa8c73fe5b50d029242e76b72`.

All **186 Clockwork Python guards**, **13 generic recovery guards** and seven
source-package checks pass. The frozen verifiers replay both successful runs;
native code is unchanged, so the prior 905 active Rust tests, locked release,
72 paired/driver checks and exact single-player 11+1 baseline remain the build
evidence. These runs use two processes over the internet transport with 40 ms
added delay and 20 ms jitter. Second-portal traversal, final scene/exit, fresh
full-route, Blue recovery, team wipe and reconnect repeats remain open. No full
visit is credited, and overall completion stays **48.6%**.


## Lowered clock portal, final scene and shared exit (5 November 2026)

Resumed **144-145** passes on unchanged native source `84c8eee`. Each player
uses ordinary jump/climb input to enter the clock's lowered portal. The verifier
checks the four solved chairs, completed lowering, original source and target,
native takeoff, matching personal warp stamps on both peers, unchanged life,
grounded clear arrival and a waiting handoff before the follower enters.
The generic recovery presence check accepts only these independently verified
epoch/player/warp tuples; every other unscripted warp still fails.

Two earlier attempts crossed with Red but stopped while selecting a waiting
point on the small destination platform. The runner can hold an already clear,
dry arrival, use an observed supported neighbor, or follow the original onward
waypoint before inviting the follower through. The successful run exercises the
first option; the onward fallback has guard coverage but no separate successful
live receipt yet. It records **two crossings, two personal 145 arrivals and
361 matching player/world/native states**, ending at **26.93/23 health**.
Evidence: `private/coop/hatter1-route-7b0a017e8e5e49a7a8f6e5244d7b8053/result-summary.json`.
Original observations SHA-256:
`628e0132b719642b2613926d39584f2fee0c83fa727190e4e666945e8478b6fa`.
The normal checkpoint before 146, tick 264670, has SHA-256
`0dec088f49bf70189603ad0c912aa0ff25cea7c714f2dd3e66f8f705acb96a46`.

The final scene verifier now handles the boolean `stopped` flag explicitly and
requires the full 30-second watched scene, its actual lever user, hidden/frozen
partner, preserved resources and safe reunion. A first 146-148 run passes the
scene but enters About Face while the leading player moves aside near the exit.
That run remains failed because its prefix verifier did not support a level
transition. The terminal fixture now treats 148-149 as a complete exit sequence:
one player personally reaches the original approach and walks toward the
original exit; the native trigger moves both players into the next visit.
The shared transition is not recorded as two personal waypoint arrivals.

Resumed **146-149** passes. Red owns the Stop scene, both reunite, the original
20-second wait completes, and Red enters the exit. Both clients arrive in
`hatter2`, epoch 2, at its default entrance. The verifier checks completed
Clockwork progression, acknowledged exit movement, nearby source contact,
clear grounded arrivals, preserved health/spawn stamps and personal resources,
rendered camera subjects and matching destination state. It records **1,122
matching player/world/native states**, including **27 in About Face**, with no
new death and final health **26.93/23**.
Evidence: `private/coop/hatter1-route-f2336ac9cc204138b700947146faa2fe/result-summary.json`.
Original observations SHA-256:
`e93c64e1270f78d611573f7f3f66836a4ab1428dd023eed2dc05069dd28ad78c`.

Both saved frozen verifiers replay successfully, including the closed recovery
traces. The stronger current exit resource check also passes the retained live
observations. All **210 Clockwork Python guards**, **15 generic recovery guards**,
seven package checks and the source worktree audit pass. No native source or
gameplay damage changed; the prior 905 active Rust tests, locked release,
72 native paired/driver checks and exact single-player 11+1 baseline remain the
build evidence. These are checkpoint segments over the internet transport at
40 ms added delay and 20 ms jitter. Fresh full-route, remaining scene-owner,
recovery and reconnect coverage remain open; no full visit or About Face battle
is credited. Overall completion remains **48.6%**.
