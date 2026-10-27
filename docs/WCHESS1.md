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
