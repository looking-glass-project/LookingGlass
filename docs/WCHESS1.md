# Pale Realm implementation

`wchess1`, route 19, is owned by `levels::wchess1`. The destination is
`wchess2$wchess2_start1`. Start it through `tools/launchers/Launch-Pale-Realm.cmd`; its saves are
separate from normal playtest saves. Save envelope 12 and the 7,900,000 hit range
are retained. Data and footage are not redistributed.

## Playable systems

The owner draws and collides the quad gate, knight gate, lift, puzzle controls,
crushers, spike corridor and pool machinery. Model ownership excludes duplicate
placed chess actors. White and red pieces use the chess combat family, faction
targeting, authored spawn receivers and per-actor saved health/activation. White
actors start invulnerable; the escort and guard explicitly opt into damage.
Instructor health floors are 125/150; the rook floor is 180. The escorted pawn,
rook/bishop duel and optional bullied-pawn sequence retain their distinct paths.
Monster launch pads apply a swept ballistic flight using the same target formula
as player launch pads. Difficulty filters apply before the roster is built.

The bishop and knight lessons use reviewed arrival-thread tables, including
intermediate legs, heading and arrow masks. Jumping, attacking and summoning are
suppressed while disguised. Corner/edge contact starts a collision-checked run;
the normal model and held toy are replaced by the supplied TAN chess piece,
attached to Alice's ORIGIN bone so its authored coordinates stand above the board.
Pits revert the disguise, crushers inflict 1000 damage, and failed lessons have a
saved retry. A completed lesson cannot be re-armed.

The knight gate latches after four counted deaths, or eight if the optional gang
was activated before the original threshold. A late gang cannot close or re-open
that gate. Shot and touch activation of the gang share a one-shot latch. Both
red gallery bishops must die before the spike corridor is removed. The bell
lever retains a five-second free interval before the gate scene. Each lever has
reach, facing and obstruction checks, a single use and the supplied pull motion.

The water sequence starts its world clock 4.7 seconds into staging. A three-second
wait precedes a five-second rise of 96 units. The liquid surface moves from -384
to -288; the exit block is removed only when the rise finishes. Skipping never
fast-forwards that machinery. Swimming and climbing use the player's real liquid
and collision state. The exit requires completed knight traversal and raised water.

Five scenes cover arrival, both transformations, bell and water. They save the
C1 clock, captured player pose and actor starting pose. Pausing holds the clocks;
watching and skipping use the same completion code, with the authored bell-skip
rook endpoint applied explicitly. The supplied camera tracks, actor clips,
transformation particles, bell, refusal motion and machinery cues are loaded from
local data. This visit contains no voiced dialogue.

## Data decisions and native research

- **High confidence:** the native `chesspuzzle` setter toggles player flag 0x40;
  the state predicate reads that flag. `bosslevel` writes a separate global.
  `killdemons` is a separate entity traversal, not an implicit effect of either
  setter. Research addresses: 0x100aabf0, 0x100a7460, 0x100aac50, 0x100a5c40.
- **High confidence:** Actor HEALTH compares health below a floating threshold;
  Player HEALTH uses an integer threshold. Addresses 0x1000e000/0x100ab5a0.
- **High confidence:** `max_inactive_time` controls enemy retention. Native idle
  callbacks in combat mode (Actor+0x528 = 2) follow enemy loss and a five-second
  idle interval. PVS refresh occurs only in the separate mode 1; it does not keep
  a lost combat order alive. Addresses 0x10010fb0, 0x1000c380, 0x10022510 and
  0x10020bd0. This owner uses ten seconds without sight followed by five idle
  seconds. Native polling adds a variable delay which is not reproduced. This
  corrects an earlier private interpretation that applied the idle-mode PVS test
  to combat orders.
- **Reviewed deviation:** the bishop file comments out its corner-opening calls.
  Keeping every wedge solid makes the first reachable move impossible. Only the
  arrival thread's armed corners are opened here; all other directions remain shut.
- **High confidence:** three knight destinations are undefined. Those directions
  remain unavailable. `knight_11_13` retains its authored crusher leg; it is not
  silently repaired into a safe route. Graph identities describe arrival threads,
  not merely board square numbers.
- **Reviewed choice:** duplicate instructor destination names resolve to the first
  BSP entity, #293. Crusher initial phases are deterministic per entity within the
  authored half-second window, and persist through the saved world clock.
- **Source correction:** `pawn_npc2` starts its walk when the gallery bishops spawn;
  it does not wait for their deaths. Their death callback only removes the spikes.
- **High confidence:** dead red-rook/wall-break threads have no placed caller. They
  are intentionally absent. The final undefined attack callback is a no-op.

## Persistence and checks

The tower clock hands turn about the map's Y axle, keeping their original pivots
and separate speeds. The script's X angle denotes pitch; interpreting it as a
Cartesian X rotation swung the hands through the clock face. Their phase still
uses the saved world clock, including pause and load.

New snapshots include arrival nodes and move legs, disguised state, cast, levers,
world clocks, water, lifts and gate latches. Invalid state is rejected before it
replaces the owner. A save from before this controller restarts at the entrance,
retains inventory/resources, and re-arms the supported formerly consumed scene and
combat threads. This deliberately avoids stranding an old dry-pool position.

- `--wchess1-check`: scene watch/skip/pause/save futures, actual damage callbacks,
  four/eight thresholds, shot/touch latches, hazards, retries and migration.
  Clock coverage samples both supplied hand meshes over a complete 48-second
  slow-hand turn, checking fixed depth, radius, pivots and pause/restore phase.
- `--wchess1-route-check` and `--wchess1-route-skip-check`: ordinary player inputs
  from the entrance through both puzzles, gallery fights, lift, both levers,
  water swim/climb and the real Castling exit. No teleport or resource injection.
- `--save-check-write` / `--save-check-read` with
  `LOOKING_GLASS_SAVE_CASE=wchess1-`: six independent native save/restart cases.
- `--wchess1-render-check`: authored staging plus representative gameplay poses.
  Includes seven samples of the opening clock camera across the fast hand's turn.

Private evidence, exact build identity and remaining fidelity limits are recorded
in `private/wchess1/WORKLOG.md`. Precise native navigation, randomized polling,
sub-frame camera timing and the transformation puff remain fidelity comparisons;
static and silent native checks do not constitute an audio listening comparison.

## Co-op cast components

The paired combat step advances the shared resident cast once. Red pieces choose
a visible living Alice, with personal melee/beam damage, while explicit scripted
duels retain their NPC target. Ordinary faction choices also require sight in
co-op. Either player's incoming projectiles can raise a knight's native shield;
the original damage rules, protected actors and death callbacks remain shared.
Optional per-actor player identities preserve committed attacks across saved
states and compressed guest snapshots. Single-player snapshots omit these fields.

`--wchess1-coop-combat-check` stages isolated encounters using the real map roster
and animation timings. It checks target visibility/death/invisibility in both
slots, personal attacks and sounds, saved bishop beam ownership, faction duels,
shield vulnerability, invalid/legacy saves, cinematic freezing, delayed cast
walks/spawns, four/eight knight-gate death thresholds and the gallery callback.
The combat fixtures use a simple floor and deliberate cover; they do not prove
native-map navigation or rendered effects.

The later paired-world adapter below now owns scripted cast walks, keeping one
update per shared world tick. Isolated cast fixtures retain their own clock.
Full paired traversal and independent internet clients remain unverified.

Verification: source `15daa28e441dcbb6efb60710485229df6426ea5c4ce358d56e33a6713037c456`
passes all **40 cases** in about two seconds without a window or images
(`private/coop-wchess1-combat-components2.log`). The single-player component check
and both full watched/skipped routes exit successfully and their complete
normalized logs match the pre-change `2990c66` baseline. The frozen executable is
`private/coop-wchess1-combat-15daa28.exe`, SHA-256
`5764d6359878daf2d0de3bf2b069b0fcfcd1ac395f96f7e598449705d2be5b9a`.

## Co-op world and puzzles

The host now advances shared water, gates, spikes, cast walks and lift machinery
once. The lift waits for every living rider before departing and carries both
bodies atomically, retaining the original obstruction checks. Crusher damage
belongs only to the player touching it. Either Alice can use a lever; its saved
pull, delayed scene and original water/bell clock retain that actor. A death
during an unfinished pull allows the survivor to retry it.

The first scene actor controls the original Bishop/Knight disguise and puzzle.
Their partner waits, is hidden and watches that actor until the original goal
releases both at safe positions. The disguise retains its original movement
restrictions. An actor death releases the partner and preserves the puzzle's
retry. The ten-second death countdown runs during interactive puzzle control;
an expired revival waits until the safe handoff. Existing cinematic countdown
pausing is unchanged. Health and other resources remain personal.

The scene and lever art use the actual actor's Blue/Red clothing. Transformation
particles restart their world-space birth positions when the scene or actor
changes, including a guest first receiving the new effect midway through it.
The automated renderer caught the Knight transformation retaining Bishop birth
positions. A separate native test caught a bell-lever scene ending on a narrow
ledge; only completed lever handoffs opt into a bounded adjacent-floor search,
with full path, support and hazard checks. Ordinary revival placement is unchanged.

`--wchess1-coop-world-check` passes **79 distinct native-data cases**:

| Coverage | Cases |
| --- | ---: |
| Shared state/transforms at 30/60/144 Hz, pause and compressed restore | 3 |
| Native lift boarding, ascent, mid-ride save, top wait and living/dead underside obstruction | 6 |
| Personal Bishop/Knight crusher damage, both player slots | 4 |
| Five original scenes, either actor, watched/skipped, living/dead partner | 40 |
| Both complete puzzles through ordinary touch-strip inputs, either actor, watched/skipped lesson, living/dead partner | 16 |
| Both levers, either actor, saved uninterrupted/interrupted pulls and original scenes | 8 |
| Puzzle actor death, released survivor, saved retry and ten-second 25-health recovery | 2 |

Setup repeats its scene checks inside some puzzle cases; those log lines are not
counted as additional cases. These are focused native-map fixtures, not a combined
entrance-to-exit combat route.

`--wchess1-coop-render-check` passes four Bishop/Knight × Blue/Red cases in Anode.
Each submits the Alice model/materials and compares the real transformation
particle pass with compressed guest state: **1,200 batches per side per case,
zero dropped batches**. No screenshots are captured or inspected. This checks
render execution and effect submissions, not pixel-perfect appearance or a
two-process internet session.

Verified source: `cbeb7342c21921b3fd9e34d0e2a9f0c923b0ce495d49020d349a9c3296d64183`.
Frozen executable: `private/coop-wchess1-world-cbeb734.exe`, SHA-256
`bf74b27ea0eb7c693b4e79930e21fb451641df169deeba63dac081abcd998d6e`.
Native and renderer logs: `private/coop-wchess1-world-components5.log` and
`private/coop-wchess1-render5.log`.

On that same source, the 40 cast cases pass again, and the complete normalized
single-player component/watched-route/skipped-route logs match the `2990c66`
baseline exactly. The locked release build and **815 Rust tests** pass (one
existing test remains ignored); the seven source-package tests also pass.

This visit remains partial. Full paired traversal/combat, independent internet
play, rendered combat effects, visit-level team wipes/checkpoint reload and
disconnect/rejoin still require combined evidence.

## Continuous co-op route and captured door contact

`--wchess1-coop-route-check` now drives the production Arena from the entrance
with both player orders and watched/skipped scenes. It uses ordinary movement,
funded attacks, door use, original puzzle inputs and lever use. Navigation goals
never write player positions, inventory, health or progression. Every tick
checks for death/wipe, unscripted relocation and body overlap; regular compressed
guest snapshots must match. Completion requires all five scenes, both puzzles,
the water unlock and a real Castling arrival preserving both players' resources.

The default course is `src/levels/wchess1/coop_steps.json`. Optional selectors
`LOOKING_GLASS_CHESS_COOP_FIRST=Blue|Red` and
`LOOKING_GLASS_CHESS_COOP_SCENES=watched|skipped` narrow a run. A local
`LOOKING_GLASS_CHESS_COOP_COURSE` override supports diagnosis. Reports under the
isolated settings directory record the source identity, course SHA-256, original
course identity, current goal, recent inputs and complete captured world. A
different course cannot receive `full_route_passed=true`.
`--wchess1-coop-route-probe` reads `LOOKING_GLASS_CHESS_ROUTE_REPORT` and examines
collision on isolated copies. Its optional `LOOKING_GLASS_CHESS_ROUTE_GOAL` is a
JSON `[x,y,z]`; it never changes a running session or supplies traversal credit.

The route exposed a real contact failure beside the fully opened supply door:
normal separation pointed into its angled surface, while rounding rejected a
safe retreat along the actual movement. Co-op separation now checks a bounded
retreat of approaching bodies and one collision skin of clearance for glancing
wall contacts. Full swept collision, body clearance and support checks remain;
stationary/departing partners are not used to make room for an approaching body.
The regression restores the original door's 90-degree pose and verifies that
the captured blocked trace exists before checking both player slots for 240
movement ticks, with unchanged resources and a stationary partner.

Source `24a8a6f5f69995f93e34245d5865951e10c5665dbde4b0b219ba1e4270b8db11`
passes **81 native world cases**, including these two contact cases, in
`private/coop-wchess1-components12.log`. The frozen executable is
`private/coop-wchess1-world-24a8a6f.exe`, SHA-256
`2955ce2cdac1bb7d9fe586e8e5161b9f975b12a3759cdedf08afb35cebcdf09a`.
All three complete normalized single-player check logs still match `2990c66`.
The locked release build, **816 Rust tests** (one existing ignored test), seven
source-package tests and source worktree audit pass on this source.

On that source, the four default full-route attempts were **0/4**. They retained separation
through the formerly failing contact, but stop at supply-return step 22 because
the automated follower blocks the next waypoint. See
`private/coop-wchess1-route12-all.log` and its per-case captured reports under
`private/coop-wchess1-route12-all-settings/`. This is a verified contact fix and
repeatable route diagnosis, not a completed campaign visit. Overall progress
remains **45.8%**; full traversal and the independent internet/recovery matrix
remain open. No images were captured or inspected.

### Descending contact and route progress, 27 October 2026

The continuous route subsequently exposed two descending-body contacts: an
upper gallery with a stationary partner on a slope, and a narrow stair passage
without room for a sideways correction. Separation now assigns vertical entry
to the approaching body. If a safe lateral correction cannot fit, it can stop
that body at the original vertical contact after checking the swept path and
clearance. Meaningful landing damage belongs to the falling player; resting
gravity does not repeatedly emit landing events or charge damage. The stationary
partner remains unchanged. Four captured native cases cover both player slots
for 240 ticks each; two unit tests cover vertical entry and a confined high-speed
landing with 120 resting ticks.

The route driver now checks short, outward follower-yield segments, allows a
follower safely supported on its leader to finish a stair waypoint, records an
immediately skipped Intro's owner, and tries ordinary jumps before a detour search.
It still uses ordinary movement, combat, resource and scene rules throughout.

Source `24e052098b1c4d305d2e34af768f7acf2a6dca05ae5faabb04399ce12f12501a`
passes **85 native world cases**, **818 Rust tests** (one existing ignored test),
the locked release build and seven source-package tests. All three complete
normalized single-player check logs match the `2990c66` baseline. Frozen binary:
`private/coop-wchess1-world-24e0520.exe`, SHA-256
`3901756fd67de2a2ce259e41fc9a0d8238c3f7917943f9ed455a805417a8733e`.
Evidence is in `private/coop-wchess1-components19.log`,
`private/coop-wchess1-unit19.log` and the three `*-24e0520.log` single-player logs.

Before adding jump recovery, source `df70c2f` reached default step **149 of 161**
with Blue 65 health, Red 85 health, both puzzles completed, all five original
scene owners recorded and raised water. The native capture shows a small static
lip stopping walking at the water entrance; an isolated ordinary jump clears it.
That probe is diagnosis, not traversal credit. The Blue/watched attempt at that stage
on `24e0520` changes earlier timing and catches another small horizontal overlap
at platform-edge step **98**, tick 43,262. Both players remain alive. Preserve
`private/coop-wchess1-route19-settings/wchess1-Blue-false-failed.json` for the next
contact regression; `private/coop-wchess1-route17.log` records the farther run.
Neither run completed the route. Full traversal, independent internet/recovery
and broader rendered combat effects remain open; the overall estimate remains
**45.8%**. This batch used no screenshots or visual inspection.

### Contact while already overhanging a platform

The step-98 capture showed that ordinary movement had left both Alices supported
by part of their collision footprints, with no floor directly under either
centre. A safe separation correction was rejected by the extra centre-support
check, allowing a small overlap to accumulate. The correction now accepts this
existing overhang only when the old and new footprints have an intersection
that itself rests on walkable floor. Swept clearance, body clearance and floor
support remain required. A previously supported centre cannot become an
overhang, and a correction cannot transfer support across disjoint footprints.

Two exact native landing replays cover both player slots for 240 movement ticks,
checking separation, support, personal resources and the unchanged waiting
partner. Two unit tests additionally cover an already overhanging approach,
rejection of a newly created overhang, loss of floor support and a support gap.
Source `7895e86774ebba6b8e09f9b33f2fcbdd48518cde91968956c1fe7dfdf8ce23ff`
passes **87 native world cases and 820 Rust tests** (one existing ignored test).
All three single-player logs still match `2990c66`. Logs are
`private/coop-wchess1-components23.log`, `private/coop-wchess1-unit23.log` and
the three `*-7895e86.log` single-player checks. Its frozen executable is
`private/coop-wchess1-world-7895e86.exe`, SHA-256
`05bbfc600ef11dfae1fdf735395c88d42c71ca3d77989569c315d94003775345`.

All four continuous routes on that source clear the edge contact and a low
doorway, then enter the final water section with both players alive. Each stops
at step 149 because the test follower aims at the leader's old jump arc above
the water. This is captured in `private/coop-wchess1-route23.log` and its four
failure reports. The driver now distinguishes airborne trail samples from
grounded and swimming positions; it also steps aside or backward when carrying
the leader below a low doorway. These changes do not alter gameplay physics,
resources, the authored course or final waypoint height checks.

The unchanged production contact code also passes the independent public-internet
entrance regression on source `a2b939d`, with **40 ms delay and 20 ms jitter** per
sender. Both approaching-player roles stop at 36 units without pushing their
partner, both models remain marked visible, and both clients emit ambient
particles and submit real effect batches without drops. All **601** matching
sent/restored states agree, with no death, respawn or wipe. Evidence is
`private/coop/presence-test-2f498f4b699d4afb835b00f4c5b86f6b/`. No images were
inspected. This remains an entrance contact/effect check, not full Pale Realm
internet traversal or broad combat-effects coverage.

### Full native paired traversal verified

Source `7fa6ff40caeb63cbaaff677790cbd7d4275ab061c9c584913eab8cb573688ca0`
passes **all four default entrance-to-exit routes**. Each uses the unchanged
161-step course, normal combat and personal resources, both original puzzles,
all five scenes, swimming and the real Castling arrival. No player dies or
respawns, no team wipe occurs, and no cheat or unscripted warp is allowed.

| Leader | Scenes | Movement ticks | Matching compressed guest snapshots |
| --- | --- | ---: | ---: |
| Blue | Watched | 68,888 | 114 |
| Blue | Skipped | 61,817 | 103 |
| Red | Watched | 68,863 | 114 |
| Red | Skipped | 61,792 | 102 |

All four finish with both players alive at 100 health and unchanged personal
resources across the chapter arrival. The Intro belongs to Blue at initial
entry; the other scenes belong to the selected first-arriving player. The
**433** snapshot comparisons cover the production Arena world. These are native
paired routes within one process; separate-client internet traversal remains open.

The driver now records whether a trail sample was airborne, preventing a swimmer
from aiming back into an old jump arc. When an arrived leader occupies the
follower's bank landing, it walks onto checked floor and waits for the real
pull-up to finish, then satisfies the original waypoint arrival checks. It does
not move either body directly or relax collision/depth assertions.

The final locked release build, **820 Rust tests** (one existing ignored test),
**87 native world cases** and all three complete single-player baseline log
comparisons pass on this exact source. The default course SHA-256 is
`a1011d08ec00c817dbe4599ca0bfdc32c967d5eec4402e6a8c0a43c3055174bd`.
Frozen executable: `private/coop-wchess1-world-7fa6ff4.exe`, SHA-256
`88d5df04578e4afb2b42048eac24cb384269eb002d1b40caff883594b20bf342`.
Logs are `private/coop-wchess1-route26.log`, the three
`private/coop-wchess1-route26-{Blue-skipped,Red-watched,Red-skipped}.log` files,
`private/coop-wchess1-components26.log`, `private/coop-wchess1-unit26.log` and the
three `*-7fa6ff4.log` single-player checks. The four success reports and
`private/coop-wchess1-route26-summary.json` verify source/course identity, scene
ownership, all 161 steps and survival. Full internet routes, broader rendered
combat effects, team-wipe/checkpoint reload and reconnect coverage remain open.
Overall progress stays **45.8%**; no combined campaign row is completed here.

### Independent-client route fixture

`tools/test_coop_chess.py` runs the same embedded 161-step course through two
independent native clients and isolated profiles. Public internet joining is the
default, with 40 ms added delay and 20 ms jitter per sender. `--local` explicitly
selects a local connection; `--first blue|red` chooses the leading player.

Each client derives movement, attacks, door/lever use and chess-piece directions
from its own received world. The harness advances the authored step after both
clients report completion. It never supplies positions, resources or world
progression. Path searches run outside input sampling so they cannot freeze the
host's simulation clock. The reports retain embedded course identity, body and
world hashes, scene ownership, swimming, personal resources and next-visit arrival.

Run this fixture in Anode with `--exe` pointing at a frozen release build and
`--data` pointing at a compatible local installation. All evidence stays under
`private/coop/chess-route-*`; no screenshots are required. This fixture currently
watches the original scenes. Skipping, recovery and rejoining require separate
tests, and adding the fixture alone does not establish a passing internet route.

The first independent-client attempt on source `e48ba75` was stopped at step 0:
full-delay velocity anticipation made the guest circle small trail points.
Reducing approach gain according to measured delay and limiting velocity
anticipation to 50 ms lets source `83059a1` complete steps **0–13**, including
the original Bishop puzzle and reunion, through the public connection. It then
fails at step 14: Blue exhausts Will and takes five 20-point pawn hits while
waiting outside Blade range for the follower. This is a failed clean traversal,
not a passing route or completed campaign row. The private reports preserve the
last living state and the fatal state for diagnosis.

Evidence: `private/coop/chess-route-55a6a0cf920b4e4ca60d8efcdd3f1552/` and
`private/coop/chess-route-3e39aa3af5d64059829f0b36b0b74ef6/`. The second source
passes the locked release build, **820 unit tests** (one existing ignored),
**87 native world cases** and all three complete single-player baseline log
comparisons. Full internet traversal remains open. Further fixture changes add
scene-name reporting and preserve free melee attacks during delayed waits;
their route result must be checked independently.

The next public attempts on `3a2f9d8` and `4530145` again complete steps 0–13
and fail at 14. The first of these records weapon switching near a glancing
doorway; a read-only probe of the saved state confirms normal walking can slide
through that approach. The later controller predicts that ordinary walking on
a copy and retains a close target. It defeats the first pawn but Blue dies to
the following attacker while the guest catches up. It is still not a passing
route. Evidence: `private/coop/chess-route-ae1e25b8e6d54a43b694e1586bf23282/`,
`private/coop/chess-route-6bf7a58eedf04f65aeb28e7e2ceb7489/` and
`private/coop-wchess1-internet3-combat-probe.{json,log}`. The latter route records
full host state before each step and both clients' pre/post-damage states for
focused encounter diagnosis without screenshots. These captures are diagnostic
data, not completed traversal evidence.

Source `685a05d` allows the delayed follower to pursue a short clear stretch of
the leader's recorded path without braking at every sample. This lookahead is
limited to supported dry floor, with complete body clearance and floor/hazard
checks; jump, bank and swimming samples retain the original trail. The next
public-internet attempt clears the former step-14 fight and completes **71 of
161 steps (0–70)**, including the Bishop puzzle and bell sequence. It times out
at step 71 with Blue hanging at `(1008.97, 303.03, 27.16)` and Red waiting nearby.
Both survive at 18/70 health. The saved course report shows no active path search;
the independent controller currently starts searches only while grounded, unlike
the native replay. This is a diagnosis lead, not a verified repair.

The failed run retains **8,213 exact matching states** for player, shared-world
and native-resident hashes. Both clients observe Blue owning the Intro, Bishop
and Bell scenes. Their rendered-effect counters are positive with zero reported
drops, but counters alone do not establish correct visibility of every combat
effect. Evidence is `private/coop/chess-route-0f0791b0561542b4ab128f7f0f76af28/`,
including `prefix-analysis.json`, `before-step-071.json`, full history, damage
captures and the terminal failure. No screenshots were inspected. The run has
ended and both owned clients have closed; full internet traversal remains open.

The final locked release build and **820 active Rust tests** (one existing
ignored) pass on this source. It also passes **87 Pale Realm native world
cases**, all three complete single-player baseline log comparisons, all 39
campaign entrance checks plus four underwater recovery cases, 82 shared-world
checks, five Fortress adapter checks and 15 Beyond adapter checks. These are
regressions and component tests, not a substitute for the remaining full routes,
recovery, rejoining and combat-effect verification. Overall progress remains
**45.8%**, with no additional completed campaign row.

Exact source: `685a05dc1265948c2e1ba8f0e72138793d844c240467fab20cd47681e8b3a9a6`.
Frozen executable: `private/coop-wchess1-world-685a05d.exe`, SHA-256
`5f5ffe9273dc56fd958331993887db1b9d8505c91beecc3ca4aba3df34fe8f33`.
Logs: `private/coop-wchess1-network-{build6,unit6}.log`,
`private/coop-wchess1-internet5.log` and
`private/coop-wchess1-{components,check,route-check,route-skip-check,campaign,shared-world,fortress,beyond}-685a05d.log`.

### Delayed follower and ledge recovery

The independent route now searches from a real hanging state as well as a
grounded one. A captured-state physics probe of the step-71 failure finds and
replays 12 ordinary-input segments to supported ground near the original goal.
The probe reads a captured frame/world, changes only isolated copies and earns
no traversal credit. Set `plan: true` in its private input report to retain the
chosen controls and verify every replayed segment against the planner.

The first full internet repeat on `d35e931` fails earlier at step 62: the follower
brakes at closely spaced airborne trail points and repeatedly hops on the ramp,
leaving the leader exposed to two pawns. Its 4,066 matching network states agree;
the captured damage is real, not a passing clean route. Source `4923202` follows
nearby clear airborne samples through the recorded landing, while retaining
support checks on grounded shortcuts and excluding swimming, ledge pulls, climbs
and ropes. It clears that encounter with both players at 100 health and completes
steps 0–70. The run is then deliberately stopped after repeated step-71 escape
attempts: the follower stands behind the hanging leader and blocks the planned
release. Evidence: `private/coop/chess-route-9e20cdfedea34044940702c72472e0cd/`
and `private/coop/chess-route-86b8888e17c74bffa54f0b2181c649e6/`.

The follower now walks to checked floor away from a nearby hanging leader so
that the leader can release and go around the obstruction. Source `4adff1c`
passes a two-client **public-internet segment covering steps 71–74**, using the
unchanged normal save from the preceding full run and 40/20 ms delay/jitter.
All **414** matching player/world/native-resident states agree, with no deaths,
unscripted warps or free-play overlap. This proves the focused fix, not the
complete 161-step route. Evidence:
`private/coop/chess-resume-349603a8cf0c495290d60271f1095440/`.
The earlier resumed attempt on `4923202` restored correctly and reproduced the
hang, then its guest reported that the host ended the session; that unexplained
connection ending remains recorded in
`private/coop/chess-resume-951a27b26a6c4a5cbb7a606b4482e265/`.

`tools/test_coop_chess.py --save-before 62 71` requests the same save operation as
the host menu and retains the resulting files byte-for-byte. The separate
`tools/test_coop_chess_resume.py` accepts `--checkpoint`, `--start-step` and
`--last-step`, verifies restored placement and personal resources, and replays
only that segment with two independent clients. Its result always sets
`full_route_passed: false`. The verified checkpoint before step 71 has SHA-256
`52cae121ed673012ac411327306cc0542870a13ad0f2ffe1a46e4caa0c0c524b`.

The presence verifier now derives active puzzle ownership from the board/reunion
state. A remembered actor alone no longer exempts ordinary play from overlap and
camera checks. It also rejects unscripted warps and movement by the waiting puzzle
partner. Four positive/negative guard tests pass, and the stricter checks pass
over the earlier 2,355 host and 2,317 guest observations. No screenshots were
inspected for this work; combat-effect visibility still needs separate evidence.

Source: `4adff1c24f1c8e5fb0e542d761e1128e8897fd536d8104a08449ecb8ee1ada5b`.
Frozen executable: `private/coop-wchess1-world-4adff1c.exe`, SHA-256
`b873a875c79aedfd899111de9704911adb4b3d0f600b46afd4fdcaaa8f2ca2a1`.
The locked release build, **820 active Rust tests** (one existing ignored),
**87 native world cases**, all three complete single-player baseline log
comparisons, seven source-package tests and four presence-verifier tests pass.
Logs are `private/coop-wchess1-network-{build10,unit10}.log` and
`private/coop-wchess1-{components,check,route-check,route-skip-check}-4adff1c.log`.

The fresh full-route repeat on that build completes **97 of 161 steps (0–96)**,
including both original puzzles, the bell sequence and the previously blocked
step 71. Both players remain alive at 68 health. It times out at step 97 when
the arrived leader occupies the follower's ledge pull-up destination. The normal
player-contact system holds the pull-up safely; the automated leader incorrectly
waits because its existing landing-clearance rule only recognizes ordinary
climbs. All **9,205** matching player/world/native-resident states agree, and the
stricter presence checks pass across 2,630 host and 2,574 guest observations.
Evidence: `private/coop/chess-route-2e8d380017c04ea3be50d7e04d1e8da9/` and
`private/coop-wchess1-internet8.log`. This is still a failed full traversal.

The landing-clearance rule now recognizes a partner's active ledge pull-up as
well as an ordinary climb. Source `766eb15` restores the same unchanged normal
checkpoint before step 71 and completes **55 resumed steps (71–125)**, including
the previously blocked pull-up at 97. It then fails at the step-126 combat clear:
Blue dies, with Red still alive at 30 health. The final living observation has
Blue at 8 health, Red at 50 and both nearly out of Will. This clean-route fixture
stops on death; it does not establish the later revival/wipe outcome. There are
**3,442** exact matching player/world/native-resident states, and presence checks
pass across 1,037 host and 1,005 guest observations. The remaining tail has not
passed. Evidence is `private/coop/chess-resume-ebee548717b44f7ab664b8e71967b82f/`
and `private/coop-wchess1-resume3.log`, including unchanged normal checkpoints
before steps 97 and 100, full pre-step states and damage captures.

Exact final source: `766eb15988f6ed692e97fb7d805f7ab4c31ac6a377739679dfcb25625d7c448c`.
Frozen executable: `private/coop-wchess1-world-766eb15.exe`, SHA-256
`e4588cbcc3d3bf29a51d7ef389a58c310f1aba1b3d034ecd00e17bbbdb2cb42c`.
The locked release build, **820 active Rust tests** (one existing ignored),
**87 native world cases** and all three complete single-player baseline log
comparisons pass. Logs are `private/coop-wchess1-network-{build11,unit11}.log` and
`private/coop-wchess1-{components,check,route-check,route-skip-check}-766eb15.log`.
Overall progress remains **45.8%**. Full fresh routes in both arrival orders,
the rest of the water section, combat effects and recovery/rejoin remain open;
combining successful fragments from different builds does not complete a row.

## Independent-client revival fixture

`tools/test_coop_chess_recovery.py` starts from an unchanged normal checkpoint,
uses the ordinary course controls, and allows a real combat death in its target
encounter. The survivor then retreats along visited waypoints while the dead
client keeps its course selected. The verifier checks the 1,200-tick countdown,
moving spectator camera, return to the revived player's camera, 25 health,
nearby safe placement, retained personal inventory and matching client states.
It grants neither team-wipe nor full-route credit. Five guard tests reject
incorrect countdowns, health, cameras, inventory and missing lifecycle evidence.

The course driver now observes the simulation clock while spectating and drops
its old movement plan. Revival can restart the current authored step from the
actual new body rather than treating the ten-second countdown as a stale-client
gap. This changes the diagnostic controls, not damage, health or revival rules.

The first public-connection attempt with 40/20 ms delay/jitter on source
`bad8f7613700380354a72c815c4cff35dcbbbe538a1c601afc2c7bb97872a5d2`
passes **17 approach steps (100–116)**, then fails at the automated combat clear
at step **117**. Both players remain alive at 68/50 health; a nearby Knight
remains alive at about 8.8 health. No death or revival occurs, so the recovery
checks remain unexercised in this visit. The captured history passes presence
checks over 1,109 host and 1,070 guest observations, with **3,649** matching
player/world/native-resident states. The failure is retained in
`private/coop/chess-recovery-d7eefa49900d48ae907795d95c97da64/` and
`private/coop-wchess1-recovery1.log`. The next step is to diagnose the combat
controls from that retained state before repeating the revival attempt.

The frozen executable is `private/coop-wchess1-world-bad8f76.exe`, SHA-256
`c45a2f454dd7b28ade379facf93504d33fc43c977d2aef6f42fdac2ee22bee57`.
The locked release build, **820 active Rust tests** (one existing ignored),
five recovery guards, four presence guards and seven source-package tests pass.
Build/test logs are `private/coop-wchess1-network-{build12,unit12}.log`.
No images were inspected; overall progress remains **45.8%** with no new
completed campaign visit.

### Combat stall and verified Blue revival

An isolated replay of the step-117 failure shows that repeated stationary Blade
throws, higher/lower aim and Cards leave Knight 72 at 8.798 health. A normal
roughly 52-unit right sidestep followed by throws defeats it without damaging
either player. Every input experiment matches its identical repeat. These are
diagnostic observations, not full-route passes; final native snapshots are
retained in `private/coop-wchess1-combat-probe13{,b}/coop-encounter-result.json`.

The independent-client course now notices eight seconds of attacks without
damage and tries a short supported sidestep during a stationary combat clear.
Copied ordinary movement checks the floor, body clearance, hazards and a
120-unit distance from the start of the fight before issuing real controls.
The captured public-connection encounter passes in 31.8 seconds, with both
players still at 68/50 health and 164 matching player/world/native states:
`private/coop/chess-resume-0edf563a9280414f9695e9921e5886d1/`.

From the unchanged ordinary checkpoint before step 100, the next public
40/20 ms delay/jitter run passes **all 26 approach steps (100–125)**. Normal
checkpoints show Knight 72 at 8.798 health before step 117 and zero before 126,
confirming actual defeat. Blue then dies in ordinary combat at tick **53,891**
and revives at **55,091** with **25 health**. Both clients verify the moving
spectator camera, return to Blue's camera, nearby placement and unchanged
personal inventory. All **1,761** matching player/world/native states agree.
Evidence: `private/coop/chess-recovery-156b3b516cd547828fecb2667f19d738/` and
`private/coop-wchess1-recovery2.log`. This is individual revival evidence;
team wipes, rejoining and complete traversal remain separate requirements.

The next short replay wins the fight without a death and is deliberately
stopped without recovery credit. The fixture now fails promptly for that
outcome. Its optional explicit victim keeps fighting while the partner retreats
using ordinary controls; no health or enemy values are changed. A first
Red-focused attempt exposes the two solid players blocking one another at the
doorway, so the retreat now passes around the waiting partner before moving
back along the visited route. A second retreat waypoint is moved inside the
corridor after a wall-clearance timeout; those failed attempts remain recorded.
Successful as well as failed runs now retain final reports for checking actual
native enemy outcomes.

The corrected Red-focused public replay passes in **38.9 seconds** from the
unchanged normal checkpoint before step 126. Blue first moves around Red and
retreats while Red continues ordinary combat. Red dies at tick **54,140** and
revives at **55,340** with **25 health**; Blue remains at 68. Both clients pass
the countdown, moving spectator camera, camera return, nearby placement and
personal-inventory checks, with **281** matching player/world/native states.
Evidence: `private/coop/chess-recovery-2bea58bfba4f48b581461b5c1b4168c2/` and
`private/coop-wchess1-recovery6-red.log`. Both individual victim roles are now
covered for this encounter; neither run supplies team-wipe or full-route credit.

Tested engine source: `0782e29193767137c3c82fd752975326d4efd7f13bfa4adb7cc9a05943a4694a`.
Frozen executable: `private/coop-wchess1-world-0782e29.exe`, SHA-256
`f922b7f94107a45bc651f3f71b9442ff705235797afbe41a0db11b090caecefc`.
The locked release build and **820 active Rust tests** pass (one existing
ignored); logs are `private/coop-wchess1-network-{build14,unit14}.log`.
Overall progress remains **45.8%**; no additional complete campaign row is claimed.

### Shared checkpoint wipe

`tools/test_coop_chess_wipe.py` resumes the unchanged normal checkpoint before
step 126. Both players approach the Pawns using ordinary walking without
attacking; the game supplies all damage and performs the shared restore. The
fixture checks a common new spawn tick, the original visit and personal
health/inventory, bounded placement at the saved positions, restored native
enemy health and progression, separate bodies, camera ownership/subject and
matching player/world/native states. Five guard tests reject plausible but
incorrect restores, including stale observations and a camera following the
other body. It supplies no full-route credit.

An isolated native replay on source `0782e29` repeats the same ordinary inputs
twice. Both die by offset 786, restoring at tick **54,177** with the saved
**68/50 health**. The complete player and combat state is checked at the atomic
restore, and both repeats match exactly. Evidence is
`private/coop-wchess1-wipe-native14/coop-encounter-result.json`; the unchanged
checkpoint conversion and its hash are retained in
`private/coop-wchess1-wipe-fixture14/provenance.json`.

The separate public-client test with 40/20 ms delay/jitter passes on that build
in 29.6 seconds, restoring both at **54,191** with **124** matching states:
`private/coop/chess-wipe-d309554de99c429089057a107eadb618/`.
The strengthened camera/subject version repeats successfully on source
`67e3d1fdc912647c916835af1f3682038ac7a43712eaace2d66ef148304fd1f7`
in 29.5 seconds, with restore tick **54,189** and **118** matching states:
`private/coop/chess-wipe-9c2f8cc7bba64568b1d3660dabc5c4d5/`.
Logs are `private/coop-wchess1-wipe-{native14,internet14,internet15}.log`.
This establishes this encounter's shared-wipe behavior alongside the two
individual revival roles. The rest of the level and reconnect matrix remain open.

### Automated doorway and swimming-exit recovery

The independent route controller now measures progress toward its goal instead
of counting sideways shuffling or jumping in place as progress. A follower
waiting immediately behind the leader also leaves 56 units of checked floor
space for a doorway detour. On source `6130a91`, the public resumed route clears
steps **129–150**, including the formerly blocked water-lever doorway, before
exposing a separate bank-climb stall at step 151. Evidence:
`private/coop/chess-resume-f3df79566b2e4077866b970b8c1a217f/`.

At that bank, the follower's recorded target is almost directly overhead. The
controller's analog arrival braking reduces directional input below the normal
climb threshold. It now keeps a deliberate horizontal direction while asking
for a bank climb. This changes automated controls, not gameplay physics or
collision. The unchanged captured state reproduces the old stall with zero
climbs, then reaches supported ground with one normal climb using the corrected
input. Run the existing `--wchess1-coop-route-probe` with a captured swimming
report containing `bank: true` to assert both outcomes; this is isolated
diagnosis, not traversal evidence. Local evidence:
`private/coop-wchess1-bank151-intent17.log`.

On source `32f6ca1460bf21caad27f9245bebaf32e115da3964ead79d1770b16afa9d7cbe`,
the public **Blue-first steps 151–160** pass in 59.8 seconds with 40/20 ms
delay/jitter. Both players swim, climb out and reach `wchess2_start1` in epoch 2,
with personal health/inventory preserved across the transition. All **617**
compared player/world/native states match. Evidence:
`private/coop/chess-resume-e97a2a19d6554adb9b568d671f53143e/`, including retained
history and `tail-arrival-analysis.json`. These resumed segments do not form a
fresh full-level pass and add no campaign-row credit.

The **Red-first** repeat of the same unchanged checkpoint also passes all ten
steps in 49.8 seconds, with both swimmers, correct arrival and retained personal
resources. All **441** compared states match. Evidence:
`private/coop/chess-resume-3b6d05104e0b41bd865854a24c8e9358/`.
The frozen executable is `private/coop-wchess1-world-32f6ca1.exe`, SHA-256
`b82d5e41a10b16004bd557c4833f5fc4977ea969afef2fefd62d8b14f25fcb89`.
The locked release build and **820 active Rust tests** pass (one existing
ignored), as do 14 chess-fixture guard tests and seven source-package tests.
Build/test logs: `private/coop-wchess1-network-{build17,unit17}.log`;
public replay logs: `private/coop-wchess1-{tail4,tail5-red}.log`.

The recovery fixture also accepts `--continue-to 160` to continue ordinary
traversal after verifying the real combat revival. Subsequent movement remains
strictly death-free so revival cannot skip swimming or another route section.
Its current retained run verifies Red's revival and steps 126–128 before the
older doorway failure; the complete continuation still needs a passing replay.

### Fresh route after the swimming-exit fix

The next Blue-first public run on source `32f6ca1` starts at the ordinary entrance
and passes **steps 0–106**, including both chess puzzles. It then fails at step
107 after the follower falls to the lower floor beneath the leader's landing.
Both remain alive; the follower's recovery search does not finish the step.
All **9,496** compared player/world/native states agree in the retained prefix,
and ordinary player-presence checks pass. This is a new recovery case, not a
complete entrance-to-exit pass or additional campaign credit.

Evidence: `private/coop/chess-route-db85869cf3e541298054be40393ff76b/` and
`private/coop-wchess1-fresh17-blue.log`. Normal checkpoints before steps 71, 97
and 100, the step-start reports, and an unchanged captured fall state at
`private/coop-wchess1-fall107-probe.json` support a focused replay.

### Jump takeoff and narrow-corridor contact

The automated follower now retains the leader's recorded grounded approach while
yielding at close range. Previously, clearing that history during a delayed jump
made it aim at an airborne sample from too far down the ramp. The unchanged
normal checkpoint before step 100 passes steps **100–108** Blue-first on source
`aa03849`, with **355** matching network states. Evidence:
`private/coop/chess-resume-6a4a282ba26a4093a8e65ac8b0b04d8e/`.
The Red-first repeat reaches all steps but fails the strict presence check at
step 102; it is retained as a failure, not a passing traversal.

That failure exposes a gameplay collision bug. A player landing beside a wall
can no longer retreat to her previous airborne height because grounded support
validation rejects it. When the normal correction is also wall-blocked, the
players overlap. The contact resolver now tries the same horizontal retreat at
the actual landing height, retaining swept collision, floor support and pair
separation checks. It does not shove the waiting partner.

An isolated captured-state replay measures **15.26 units** of separation before
the fix and **36.00 units** afterward. Native map regressions exercise 240 ordinary
physics ticks in each player slot, preserving support and personal resources.
Both pass on source `a033218`, alongside the full paired world/scene/puzzle suite
and **823 active unit tests** (one existing ignored). Evidence:
`private/coop-wchess1-landing-{before19,after20}/coop-encounter-result.json`
and `private/coop-wchess1-native21.log`.

The first corrected public replay retains **3,131** matching states and passes
all **919 host / 904 guest** presence observations. It still fails at step 102:
the test follower's four diagonal yield choices hit the corridor walls.
Evidence: `private/coop/chess-resume-18f059e5e09a41cb8aa031e77e90b9cb/`.
The controller additionally considers axis-aligned movement under the same floor,
hazard and away-from-partner checks. These focused checks do not complete the
campaign row; fresh full routes and recovery/rejoin coverage remain open.

With that yield correction, the public **100–108** segment passes in both arrival
orders on source `efc29e77b1ea6d7374c3f9832c022667eba25501a1ba4c17fc9d11aebd1083f1`.
All ordinary presence, personal-resource and three-state agreement checks remain
enabled. Red-first retains **267** matching states in
`private/coop/chess-resume-68cb093aea3849de911d0eddc897c567/`; Blue-first retains
**318** in `private/coop/chess-resume-6f0c2cf0691f4720b9dbb6b0fe2d6215/`.
Both use 40/20 ms added delay/jitter and the same unchanged normal checkpoint.
The release build and all 823 active Rust tests pass. The three single-player
check logs on the same production-contact implementation (`a033218`) match the
pre-change baseline exactly after newline/encoding normalization.

The same build also passes the Blue-first **129–160** regression through the
doorway, water lever, swimming bank and actual next-level arrival, with **2,300**
matching player/world/native states. Both clients observe both swimmers, retain
personal resources and arrive separately at `wchess2_start1` in epoch 2.
Evidence: `private/coop/chess-resume-3dd2168214af4c818b5826973b8ee63e/`, including
`tail-arrival-analysis.json`. This remains a resumed segment; overall credit stays
**46.1%**.

### Guest rejoin followed by the swimming exit

`tools/test_coop_chess_rejoin.py` restores an unchanged normal checkpoint after
both puzzles and the water release. The guest exits, a new process rejoins the
same host, and both continue through steps 151–160 using ordinary controls.
The check preserves personal resources, position, spawn/warp clocks, completed
puzzles, lever timings and native resident identities/health. It requires at
least 20 matching player/world/native states **after** the new guest joins, so
old matching observations cannot pass the reconnect check. Six guard tests reject
progress resets, revived enemies, missing residents and stale network evidence.

The Blue-first public run on source `efc29e7` passes with 40/20 ms delay/jitter in
64 seconds. It retains **645** matching states overall, **624 after rejoin**, and
both clients reach `wchess2_start1` in epoch 2. Evidence:
`private/coop/chess-rejoin-0ab32230985f44fd95b796e5cf0035bd/`, including before/after
rejoin reports and both final reports. This verifies a graceful guest leave and
fresh-process rejoin followed by the exit; it does not simulate a network outage
or establish a fresh complete level traversal.

### Preserve closely spaced landings in the recorded route

A subsequent fresh Blue-first run on `efc29e7` again passes steps 0–106 but
misses the follower's landing at 107. Its **9,606** matching states and all
ordinary presence observations pass; recovery from the lower floor eventually
stalls. Evidence: `private/coop/chess-route-74bd18b17c814048b35c4c66e0dbe4aa/`.
The leader's last airborne sample was only 11.9 units from its landing, inside
the recorder's 12-unit spacing threshold. Consequently, no grounded point was
recorded between two separate jumps. The recorder now always retains changes
between grounded and airborne states, with the spatial threshold still applied
within each state. A regression uses those captured coordinates.

On source `68696dd7cb4c0c9cb91ed044cdd97356b7bbb2ce8b5fe8c363bccbf804155f69`,
the newer unchanged normal checkpoint passes **100–108** Blue-first over the
public connection in 40.8 seconds, with **320** matching player/world/native
states and unchanged presence/resource assertions. Evidence:
`private/coop/chess-resume-d1cafba2906d40fb82b97a50e2262755/`.
The release build and **824 active Rust tests** pass (one existing ignored).
This is a focused repeat, not a completed fresh route.

Two connection failures remain separately recorded. A simultaneous Red-first
run with four game windows disconnects at step 12 when its client cannot consume
updates fast enough (`chess-route-c47f44b9b23648cea05fa71cfc55ece0`). A later
two-client jump replay fails before gameplay because its temporary invite address
cannot be resolved (`chess-resume-713d3f01cb8a40c1b3329bf001da7667`). The successful
retry above does not erase either failure or establish connection reliability.

The next fresh Red-first run uses only one client pair on `68696dd` and passes
steps **0–35**, including the original Bishop scene and shared lift. At step 36,
the following Blue player dies in the active spike corridor; Red remains at
100 health. The retained prefix has **2,026** matching states and passes all
**621 host / 608 guest** presence observations. Evidence:
`private/coop/chess-route-3f0cf4146f424d70bb039d1898b36d19/`, including the exact
before/after damage reports. An unchanged host capture before step 36 is exported
through the normal checkpoint machinery in
`private/coop-wchess1-spikes36-export23/` for focused diagnosis. The native export
and one-tick idle repeat pass; this supplies no traversal credit. This failure
motivates the focused timed-corridor checks below. Fresh full-level completion
remains open.

### Timed corridor test controls

The ordinary-input test driver now crosses the timed corridor one player at a
time. Each player waits at a safe approach point and previews ordinary movement
against the native spike hulls for the complete crossing and its estimated input
delay. Arrival means clearing the exit plane, so a leader carried farther by a
delayed stop cannot strand the follower. Approach input is limited according to
the connection delay to prevent overshooting the waiting point. The completed
corridor's recorded trail is discarded before the following leg.

The native wave-position calculation is shared with this forecast; its timings,
colliders, damage and normal player controls retain their existing rules.
On source `849973177da47717255ea0c32708fc601daab41ad1c8adf23e72d2a626885a1a`,
**3,094** predicted body queries match rebuilt native hazards across a full
cycle and the switch-off delay. Both leader roles cross with zero delay and with
36 ticks (300 ms) of guest input delay sampled every six ticks. All four cases
retain health, resources and body separation. These are staged component tests.

The assertions also retain the failures that led to the fix. Build `fa7b58e`
lets the leader clear the corridor but waits indefinitely after the leader
leaves a small arrival circle. Build `9e4d8c6` exposes the remote follower's
approach overshoot. An earlier Red-led result on that build appeared to pass
36–40, but reviewing its history showed the follower retracing the spikes on
step 37. The new progress assertion rejects **both actual client histories**
from `private/coop/chess-resume-4b9830d9fd8b41f79e6429f71600266a/`; that result
is not safe-crossing evidence.

A separate initial Blue replay stopped before traversal because a captured
player still had momentum. The resume fixture now requires the native exact
restore receipt from before physics advances, matching build, epoch and saved
tick. Later observations must retain spawn/warp stamps and the resource
contract. Four fixture guard tests cover stale proof, changed lifecycle stamps,
resource failures and the captured backtracking. The six existing rejoin guard
tests also pass.

The final build passes **36–40 over the public connection with both leader
roles**, using two independent processes, 40 ms injected delay and 20 ms jitter.
Blue has **428** matching player/world/native states; Red has **292**. Both
replays require the exact native restore receipt and pass the no-death,
presence/separation, lifecycle and no-backtracking assertions. Evidence:

- Blue: `private/coop/chess-resume-1b9342c73d524dd5966433b603be7ae2/`.
- Red: `private/coop/chess-resume-d206e365f09b4ddc83d04025e98442da/`.

These resume unchanged checkpoint exports from the original host captures;
they are focused segments, not new full-level passes. The locked release build,
**824 active Rust tests** (one existing ignored), paired native world checks,
four resume guards, six rejoin guards and seven source-package tests pass.
All three complete single-player check logs match the `2990c66` baseline.
No screenshots were inspected; fixture-generated captures remain private.
Overall co-op progress stays **46.1%**, with fresh uninterrupted routes and the
broader campaign/recovery matrix still open.

### Delayed lift handoff

A fresh Blue-led full attempt on `8499731` passes 0–29, then stalls at the lift
on step 30. The host saw both players ready and boarded; the guest missed that
brief observation and continued waiting at the assembly point. The lift itself
correctly waits for both living players. The failed prefix retains **5,281**
matching player/world/native states and passes **1,570 host / 1,556 guest**
presence observations. Evidence:
`private/coop/chess-route-68121cdcc0bd44ecba08d31991ac9e1f/`.

The test driver now recognises actual deck support as persistent evidence that
boarding has begun. At the top, each client proceeds after its own grounded
arrival, even if the peer has already left its reserved berth. It still supplies
ordinary movement inputs. A unit regression covers either observer missing the
ready frame; a native test checks the captured occupied/waiting positions against
the real lift hull. The normal all-living boarding and paired carry checks pass.

Source `d7bbc999cc76b73bf9649f28dbd657846508032f5d0e1e7209e815ce7a67466e`
passes **30–40** through independent public clients with both leader roles,
starting from an unchanged checkpoint export of the actual stalled host state.
Both pass exact restore, no-death, separation, lifecycle and no-backtracking
assertions with 40 ms injected delay and 20 ms jitter:

- Blue: **493** matching states,
  `private/coop/chess-resume-20316aa42cb649098f3e9d1a4b2f604e/`.
- Red: **491** matching states,
  `private/coop/chess-resume-92486ee0cb8f44de8639711e8d673b40/`.

The locked release build, **825 active Rust tests** (one existing ignored),
and the native paired world suite pass. The preceding build also repeats all
39 entrance/four underwater recovery, 82 shared-world, five Fortress and 15
Beyond component checks; details are in `COOP_CAMPAIGN.md`. This change concerns
the opt-in route driver. No full-level or overall percentage credit is added.

The next fresh Red-led attempt on `d7bbc99` passes **0–67**, including the
repaired lift, timed spikes and first lever sequence. At 68, Blue has landed
safely on the descending ground but still follows an old airborne sample about
346 units above her. The follower repeatedly jumps below that obsolete target
while Red waits at the next goal. Both remain alive at **54/66 health**.
The failed run has **6,729** matching states and passes **1,954 host / 1,923
guest** presence observations. Evidence:
`private/coop/chess-route-4408925c9d4747acae572d74662ed3cb/`, including
`before-step-067.json`, `before-step-068.json` and `descending-trail-stall.json`.
This is a newly isolated test-driver failure, not a completed fresh traversal.
The following change addresses this captured failure; fresh full traversal
remains a separate gate.

### Rejoining a recorded descent

Source `6e98519` reconnects a grounded follower below an obsolete airborne
sample to the nearest recorded ground sample within 48 horizontal/18 vertical
units. A complete body sweep, sampled floor support, liquid exclusion and hazard
checks must all pass. It discards only the obsolete prefix, keeps later takeoffs
and landings, and cancels any path search toward the old target. It does not move
the body or change normal movement, collision, hazards or authored course steps.

Three new unit regressions cover safe selection, unchanged body state, walls,
gaps, water, hazards, upper/distant landings and ongoing airborne/swimming motion.
The existing closely spaced landing/takeoff check also passes. All **828** active
Rust tests pass (one existing ignored). The native paired world suite reproduces
the captured coordinates and verifies 240 grounded movement ticks without jumps.

The Red-led internet replay from the unchanged checkpoint **before step 67**
passes **67–70** with 40 ms delay and 20 ms jitter per sender, exact restore,
presence/lifecycle checks and **339** matching player/world/resident states.
Evidence: `private/coop/chess-resume-b2801256b754450e9b68b65cab6a4673/` and
`private/coop-wchess1-coop-world-check-6e98519.log`. Starting before the descent
records a new trail; the checkpoint after it alone would omit the failed driver
trail and would not reproduce this case. The separate Blue-led **100–108**
required-jump repeat also passes, with the same delay/jitter and exact-restore
checks and **312** matching states. Evidence:
`private/coop/chess-resume-eba16db9f6f74dd88d18573d06f18ee5/`.
Fresh full repeats remain pending; no full-level or overall percentage credit
is added here.

The next fresh Red-led run on this build completes **0–92**, including the
repaired descent, then fails the strict puzzle-owner check at 93. Both players
remain alive at **67/66 health**. The Knight gate is still closed in the reports
before 91, 92 and 93. After combat opens it, shared trigger 30 fires and Blue
owns the Knight scene/puzzle although Red was the configured leading player.
Both are already stationary inside the area when the scene begins. The paired
contact resolver currently discards contact history while a rule is ineligible;
when enabled, simultaneous inside contacts tie by player identity. Reconstruct
the captured approach and verify the first arrival before changing this logic
or the ownership assertion. Do not treat this failure as a completed route.

Evidence: `private/coop/chess-route-45239e4b05c5479998d52cc937949eed/` has
**4,983** matching states and **1,450 host / 1,419 guest** passing presence
observations. Its unchanged before-92 report is exported through the normal
checkpoint store to `private/coop-wchess1-knight92-export29/coop-checkpoint.json`;
exact export and a deterministic one-tick idle repeat pass with the original
67/66 health. This isolates the next ownership investigation without replaying
the opening or modifying personal resources.

### First arrival while a scene is gated

The source29 history and original trigger geometry show Red inside trigger 30
at tick **32412**, with Blue outside. Blue enters later, at host tick **32484**
(guest observation 32487). The Knight gate is still closed then. Once it opens,
the old resolver treats both contacts as new simultaneous arrivals and chooses
Blue. This is a production ownership bug; the strict test expectation remains.

Source `b945e0e` retains a bounded ordered list of living waiters for an
ineligible shared trigger. On eligibility, that list resolves simultaneous
inside contacts. Leaving, dying or relocating removes a waiter; returning joins
behind a partner who stayed. Dispatch consumes the list without rearming a
one-shot rule. Personal damage and teleports do not acquire shared ownership.
The optional saved field preserves this order across restoration; older saves
load with no previously recorded order. Normal swept arrivals and shared-lift
boarding still use their existing checks.

Four new unit regressions cover both arrival orders, paused time, saved order,
withdrawal/death/relocation/reentry, same-frame swept priority, independent
personal effects, legacy saves and invalid saved owners. All **832** active Rust
tests pass (one existing ignored). Native captured coordinates against the real
Knight trigger pass for both player slots, including save/restore while gated,
the correct original scene owner, a frozen partner and retained resources.
The complete paired Pale Realm suite passes **117** cases; **82** shared-world
cases and all **39 entrance/four underwater-recovery** cases also pass. The three
complete single-player Pale Realm check logs remain identical to the baseline.

The captured Red-led internet replay passes **92–97**, including the original
Knight puzzle, with **677** matching player/world/resident states, 40 ms delay
and 20 ms jitter per sender. Both clients record Red owning the Knight scene.
Exact restore, presence, no-death and lifecycle checks pass. Evidence:
`private/coop/chess-resume-eca805c106d14bea91737c7f50ddc154/` and
`private/coop-wchess1-coop-world-check-b945e0e.log`. Broad logs are
`private/coop-coop-{world,campaign}-check-b945e0e.log`. Fresh full routes remain
open; these regressions add no full-level or overall percentage credit.

### Fresh traversal reaches the exit but fails presence checks

The next fresh Red-led internet run on `b945e0e`, with 40 ms delay and 20 ms
jitter per sender, completes all **161 steps (0–160)** and both clients enter
`wchess2_start1` at epoch 2. It then fails the unchanged final presence assertion.
The guest has two overlapping observations during step **141**, at ticks
**53649/53673**: horizontal separation is approximately **32.90/28.51** and
vertical separation **16.01/13.60**, below the required clearance. No sampled
host observation overlaps. This difference does not establish the cause; retain
both histories and reproduce the full captured approach before changing physics
or the assertion.

All **8,602** common player/world/resident state records match, but that does not
override the presence failure. The process exits with code 1 after 520 seconds;
this is not a passing full route and earns no additional campaign credit.
Evidence: `private/coop/chess-route-e8cce7e9090a4db5a38ff9b4881c4718/`, including
`overlap-analysis.json` and both per-client overlap extracts. Ordinary checkpoints
before 129 and 151 remain available; use the earlier one to reproduce step 141.

The preceding attempt failed before gameplay because the guest could not resolve
the fresh invite address. Its separate evidence is retained in
`private/coop/chess-route-658d19c8f20544378eaf4f11d8a58003/`. Joining reliability
remains open; the successful connection on the retry does not fix that failure.

### Retreat at a stair support boundary

A focused Red-led replay of 129–145 on the unchanged `b945e0e` build passes with
614 matching states, so a successful segment alone does not demonstrate a fix.
The direct native regression instead starts from the failing route's last
separate guest observation, tick 53623, with its recorded positions/velocities
and ordinary walking inputs on the actual map. It reproduces overlap at tick 11
without networking on baseline `c56eb1e`. Both bodies are grounded on the slope.
The approaching body cannot separate sideways through a wall, and its retreat
is rejected because its previous footing has footprint support but no floor
under the centre. That rejects a position the body actually occupied safely.

Source `59dc7e0` permits the final retreat to that exact previous position only
when the body is grounded, dry, fully clear, has an unobstructed reverse sweep
and still has walkable support. Normal corrections and intermediate retreat
candidates retain the stricter centre-support rule. Pair separation must still
pass before either position changes, and a departing or stationary partner is
not moved by the retreat.

The native case now passes 120 ordinary movement ticks in both slot orders,
including body clearance, ground support and unchanged personal resources.
Two unit regressions cover the support boundary and reject wall crossings,
unsupported destinations, airborne returns and swimming returns. All **834**
active Rust tests pass (one existing ignored). The full native Pale Realm suite
passes **119** cases, shared-world checks pass **82**, and **39 entrance/four
underwater-recovery** checks pass. All three complete single-player Pale Realm
logs match the existing baseline. Evidence: `private/coop-wchess1-world31-baseline.log`,
`private/coop-wchess1-stair-debug-check.log` and
`private/coop-wchess1-coop-world-check-59dc7e0.log`.

The network fixture now retains the first complete overlapping report, its
preceding report and the latest peer reports for direct reproduction. It does
not alter the final presence assertion. All 24 Python route-evidence guard
tests pass after restoring the course metadata in the older presence fixtures.

On the repaired build, the two public-client replays of **129–145** pass with
either leader and 40 ms delay/20 ms jitter per sender. Red records **668**
matching player/world/resident states; Blue records **890**. Both verify exact
native checkpoint restoration before physics, no deaths or unexpected lifecycle
changes, and no overlap observations. Evidence:
`private/coop/chess-resume-bb0f6473e35b4525b53170ad989c25bf/` and
`private/coop/chess-resume-77fbe680134d4e348bbceb23173060cd/`.
The fresh Red-led repeat on `59dc7e0` passes steps **0–80**, then stalls in the
combat clear at step **81**, with both players alive at **52/57 health**. Its
prefix contains **7,192** matching player/world/resident states and no sampled
overlap (2,154 host and 2,135 guest presence observations). The remaining nearby
red knight is alive on lower ground; the combat driver stops making progress.
The cause still needs diagnosis. An exact normal checkpoint from before that
step is retained for focused reproduction; no enemy or progression rule was
weakened. Evidence: `private/coop/chess-route-428c58e515a640328743871b9385417b/`
and `private/coop-wchess1-clear81-export32/coop-checkpoint.json`.
This is a failed full route. The Blue-led repeat on the same source passes
**0–125**, including the step-81 encounter, then Blue dies in combat during
the step-126 rush. Blue entered that section at 45 health and takes three
20-damage hits; Red remains at 68 health. Its **8,388** matching states agree,
with no sampled overlaps across **2,384 host/2,357 guest** presence observations.
The ordinary checkpoint before 126 and damage reports are retained in
`private/coop/chess-route-d6bdc9658d5747f893cb353b1fa68393/`.
This also fails the full-route gate; it does not establish the remaining stair
section or post-revival continuation.
These segments and the captured physics regression do not complete the campaign
row or increase its credit.

## Pale Realm accepted

The existing Blue-led watched course passes on `841dcd6`: 68,146 ticks, 113
exact replicas, both chess puzzles, the shared lift, timed spikes, both levers
and all five scenes through actual Castling arrival. Both players finish alive
at 92/100 health, with exact personal resources and no deaths, wipes, overlap
or unscripted warps. No native code changed. Repeat with
`tools/test_coop_chess_native.py --exe <frozen-executable> --data <base-directory> --profile <new-directory>`
in Anode. Internet and rendered-camera evidence remain separate. Coverage is
**27/39 visits**, with **12** remaining and **75.3%** overall.
