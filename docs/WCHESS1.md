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
That probe is diagnosis, not traversal credit. The latest Blue/watched attempt
on `24e0520` changes earlier timing and catches another small horizontal overlap
at platform-edge step **98**, tick 43,262. Both players remain alive. Preserve
`private/coop-wchess1-route19-settings/wchess1-Blue-false-failed.json` for the next
contact regression; `private/coop-wchess1-route17.log` records the farther run.
Neither run completed the route. Full traversal, independent internet/recovery
and broader rendered combat effects remain open; the overall estimate remains
**45.8%**. This batch used no screenshots or visual inspection.
