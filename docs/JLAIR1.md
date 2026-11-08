# Burning Curiosity (`jlair1`)

`tools/launchers/Launch-Burning-Curiosity.cmd` starts the restored visit at `jlair1_start1`.
The normal exit leads to `jlair2$jlair2_start1`, where the existing Jabberwock
survival encounter takes over. Local original game assets are required.

## Restored behavior

- Gryphon arrival: Alice rides the animated `tag_alice` attachment along the
  authored flight path, dismounts, hears the Gryphon's dialogue, and watches the
  departure. Watching and skipping place her on the same checked floor.
- The lava platform uses the original brush, 100-unit speed and 128-unit sink
  limit. Its visible and collision poses agree. It carries a rider down,
  recovers when unoccupied, and transfers the rider onto the uneven pool bed
  without dragging her through it. Scenes and pause stop the mover.
- The map's native enemies use their existing combat systems and original
  activation groups. Three spawned imps now follow their monster-only launch
  pads once, then resume normal movement. Pending and airborne launches survive
  saves. This visit has 41 supported imps; the original two `t157` placements
  have no activation and remain absent.
- The Caterpillar scene includes the single Cheshire apparition, the reveal
  zoom, original camera tracks, dialogue and actor gestures. Scene actors are
  removed from ordinary placement, so they cannot appear early or be cloned.
  Watching and skipping commit the same position and completion latch.
- The two final triggered meta-essences spawn and remain available until
  collected. Their activation and individual collection flags persist.
- Original route semantics are retained: the Caterpillar speech does not invent
  an exit lock. The exit is inhibited during a scene and uses the authored
  destination. Missing original `magmaman03` remains a no-op.

## Persistence

The versioned controller stores scene kind, speech cursor, gesture epochs,
reveal/ending/skip clocks, arrival and Caterpillar completion, sink displacement
and the two essence flags. Invalid state is rejected before changing the live
controller. Older saves without this controller resume free exploration and
rearm the previously unimplemented Caterpillar trigger instead of replaying the
arrival. Global save format remains unchanged.

## Verification

- `--jlair1-check`: rider carry/recovery at 30, 60 and 144 Hz; pause and corrupt
  state rejection; both scenes on their full dialogue clocks, watched and
  skipped; equal handoffs; no scene replay; restored scene/essence continuation;
  duplicate Jacks Will refill; a separately staged ordinary exit contact.
- `--jlair1-traversal-check`: continuous fresh entrance-to-exit traversal with
  production movement and scene handling, without the native enemy cast.
- `--jlair1-route-check`: the same continuous route with the native cast,
  Normal difficulty and ordinary combat. Set `LOOKING_GLASS_JLAIR1_SKIP=1`
  for the skipped variant. The watched route advances dialogue interactively;
  the full unhurried dialogue timing is checked separately above. Route checks
  reject death, recovery, cheats, authored teleports and a wrong exit, and
  use the carried Pocket Watch before the last ledge to protect the pull-up, and
  compare all ten carried weapon slots, the Staff component and resources
  before starting the destination scene. They then require the survival boss
  to become active.
- `--jlair1-render-check`: staged native captures of flight, landing,
  departure, Cheshire, the reveal, Caterpillar and both platform poses.
- The same render command with `LOOKING_GLASS_JLAIR1_SAVE=write`, then `read`
  in a separate process, exercises the production Store/Continue loader at six
  moments: flight, landing dialogue, a partly sunk platform, Cheshire speech,
  Caterpillar speech and completed scenes.
- Fire Imp unit checks cover launch activation, pause, midair save continuation
  and one-time landing. Existing `--jlair2-check` covers the downstream encounter.

Private recordings and diagnostics live under `private/jlair1-work`; extracted
original scripts and dialogue are not distributed. Native visual tests run in
Anode's hidden desktop with audio disabled. Audio listening and a full
performance sampling campaign are not covered by these checks.

## Recorded acceptance run

The selected release retains Water Logged and the previous restorations. It
passes all 626 unit tests plus the Jabberwock and Water Logged controller checks.
The integrated shared workspace also passes `cargo check --locked`.

Both fresh native routes activate all 41 supported imps and every resident
ambush group. The skipped run uses 18,851 physics ticks and 76 jumps; the watched
run uses 21,230 ticks and 76 jumps. Both finish with 91.58 Sanity, no recovery,
no cheats, no authored teleports, and exactly the intended exit. Both reach the
active Jabberwock survival encounter with all ten weapon counts, resources and
the Staff component preserved. The route uses the carried Pocket Watch before
the exposed final climb, where enemy knockback can otherwise interrupt a pull-up.
Brief lava contact (220 physics ticks) is survived; these are not damage-free runs.

All eight native captures and all six Store write/fresh-read cases pass on the
same selected executable. See `private/jlair1-work/release.json` and its frozen
source manifest for the build identity and detailed evidence.

## Co-op adapter and evidence, 5 November 2026

Source `56160ff` adds personal arrival grants and essence ownership, shared
sinking-platform movement and the original scenes through the paired controller.
Only living players receive the arrival grant, once; a later low-health revival
does not refill. Existing visits retain their health and consumed pickups. Each
client sees its own remaining essence, with ownership preserved in compressed
saves. Single-player snapshots omit the optional co-op state.

The platform starts when every living player has full footprint support. Once
underway it keeps carrying remaining riders if a partner leaves or reaches the
uneven floor. Empty platforms return, dead partners do not block departure, and
an obstructed return rolls back the shared displacement and both bodies.

`--jlair1-coop-check` passes **26 staged native cases plus one summary**:

- Eight personal arrival, pickup, legacy-save and resource-preservation cases.
- Six platform cases: 30/60/144 Hz boarding and carry, independently verified
  static-floor handoffs, saved continuation, empty return, either dead partner
  and atomic rollback when a returning platform would embed a jumper.
- Twelve original arrival/Caterpillar cases, including both Caterpillar owners,
  watched/skipped play, living/dead partners, owner-only skipping, compressed
  mid-scene restores, camera continuity and preserved resources. The original
  actor keeps the authored endpoint; the newly placed partner must have clear,
  supported, separated ground nearby.

The release's seven `--jlair1-check` records and two full
`--jlair1-route-check` records match source `116fc99` output byte-for-byte.
The native combat route reaches the intended downstream encounter. The separate
cast-free `--jlair1-traversal-check` still fails the step-54 lava jump exactly as
the baseline does; it is not reported as passing. Private build manifests retain
the full source/executable identities and result hashes.

All 906 active Rust tests pass (three ignored), including negative boarding
cases for partial support, jumping and a floating body with a stale grounded
flag. The locked release, seven package checks and the 908-file source worktree
audit pass. No screenshot inspection was used for this milestone.

These are native component and single-player regression results. A fresh full
route with two independent internet clients, rendered personal pickups, deaths,
shared checkpoint recovery and reconnects remain unverified for this visit.
No additional completed campaign visit is credited by these staged checks.

The first fresh two-client internet approach now passes **steps 0-3**, including
the watched arrival and both actual early lava crossings. Source `56160ff` has
eight personal waypoint receipts, two separately verified return-to-approach
walks, supported waiting positions and **2,133 matching player/world/native
states** with 40 ms delay and 20 ms jitter. Both players survive at 98.75/95.42
health; the closed host trace has no deaths or revivals. A normal checkpoint
before step 4 is retained. An earlier attempt failed when Red approached the
crossing from the sideways waiting bay; it remains failed evidence.

A separate resumed attempt verifies **steps 4-20** through its closed evidence:
34 personal arrivals, five ordinary forward waiting walks and **5,582 matching
player/world/native states**, with no deaths or revivals. On steep slopes without
a same-height waiting bay, the leader walks to the next original route point so
the follower can arrive. That longer attempt fails at step 21 when Red's planner
times out; the successful prefix is explicitly bounded before that failure.

Source `ca4cc26` fixes the automated planner's sloped-footing checks: its existing
eight-unit support margin follows the actual floor plane, with real body contact
required. A lower floor cannot justify marginal contact with an upper ledge.
The new regression also verifies ordinary walking up axis-aligned and diagonal
slopes. All **907 active Rust tests** pass (three ignored); the locked release,
26 paired cases plus summary, single-player baseline comparisons, three Python
evidence guards, seven package checks and the 910-file worktree audit pass.

The previously stalled climb now passes in a separate **21-23** internet replay
from the unchanged normal checkpoint: six personal arrivals, one forward waiting
walk and **2,224 matching player/world/native states**. Both players finish at
100 health with no deaths or revivals. The ordinary checkpoint before 24 is
retained. This replay uses the original step-21 goal and the corrected planner;
the unsuccessful extra approach point was removed.

Later closed evidence verifies steps **24-25** separately: four personal arrivals,
one forward waiting walk and **1,563 matching states** without death or revival.
That longer attempt fails at 26. The route driver held a fixed waiting point while
fire-spitters attacked; enabling ordinary dodging kept Blue on the ledge, but the
Ice preference exhausted Will before winning the fight. Both failed attempts
remain recorded as failures.

The runner now resolves visible threats before parking, using the original
route's homing Cards preference and normal Blade fallback. It then returns to
the actual waypoint before recording arrival. A resumed **26-29** run passes:
eight personal arrivals, four parking walks, one forward waiting walk and
**1,374 matching player/world/native states**. Both nearby fire-spitters die;
Blue and Red finish at **50/96 health**, with no deaths or revivals in the closed
85-row host trace. The ordinary checkpoint before 30 is retained.

A further run has independently verified **30-31** evidence: four personal
arrivals, three parking walks, **352 matching states** and 22 closed host trace
rows without death or revival. Its longer attempt fails during the forward
waiting climb from 32 toward 33 when Red misses the ascent and falls into lava.
This bounded prefix is not a passing full attempt. No added campaign visit or
percentage credit is claimed for these separate segments.

A separate retry has bounded **30-32** evidence: six arrivals, three parking
walks, one forward walk and **783 matching states**, without death or revival.
It takes a different route up the climb and then fails parking at 33. This
does not establish a fix for the earlier missed climb. The waiting routine now
recognizes an exhausted set of previously visited nearby positions.

Another attempt exposes an unsafe assumption: the next original route point,
38, is in shallow lava. It is a transit point, not somewhere to wait for the
partner. Forward waiting now stops at an observed native dry, supported bay
along the approach. Bounded **33-37** evidence passes ten personal arrivals,
six parking walks, one forward wait, one combat wait and **1,309 matching states**
with no deaths or revivals. The longer attempt still fails at 38.

The runner now jumps from an observed higher takeoff onto the original hot
waypoint, earns the unchanged personal arrival receipt, then jumps back to a
recently observed dry footing before waiting or planning. A separate bounded
**38** replay verifies two personal arrivals, both real airborne crossings and
both dry returns, with **462 matching states** and no deaths or revivals. Blue
and Red finish at **93.75/41.67 health**. Normal liquid damage remains active;
the checkpoint before 39 is unchanged. That longer attempt fails at 39 when
the ordinary planner selects another path through lava. None of these bounded
prefixes turns its failed parent attempt into a full-route pass.

Source `259d4f5` adds opt-in avoidance of damaging liquids to the automated
navigation search. Every predicted physics tick and the remote late-takeoff
margin must stay clear of liquid damage. The original hot waypoint uses its
explicit crossing instead; single-player searches retain their previous policy.
Regressions replay ordinary walks around lava and jumps over it.

The first live avoidance replay stays alive but stalls short of 39. Although
Blue's position is unchanged, collision retains a velocity above the old
replanning threshold. The driver now detects blocked walking from actual
position progress and grounded contact. Its regression includes that recorded
velocity, airborne rejection and movement resetting the timeout. All **910
active Rust tests** pass (three ignored), as do the locked release and 26 native
paired cases plus summary. The seven single-player component records, two full
combat-route records and known cast-free step-54 failure still match the
unchanged baseline exactly.

A separate resumed **39-42** internet replay now passes on that source, starting
from the unchanged normal checkpoint after 38. Both personally reach all four
original waypoints: eight arrivals, three parking walks, one forward wait, one
combat wait and **1,335 matching player/world/native states**. Blue reaches the
same previously blocked position, then continues to the original goal. Both
finish at **100/56.67 health**, with no deaths or revivals in the closed 79-row
host trace. The ordinary checkpoint before 43 is retained. This is a passing
segment, not a fresh full-route pass or another completed campaign visit.

The next attempt fails at 43: the remote timed plan strings three jumps
together, and the actual intermediate landings drift enough to start a later
jump from the wrong ledge. Source `5c4a693` adds opt-in landing stops to the
diagnostic route driver. It retains the original inputs through the first
planned landing, then settles the actual body and searches again before the
next jump. Walking-only paths and the original single-player policy are unchanged.
All 912 active Rust tests pass (three ignored), the locked release passes, and
the native paired and single-player baseline outputs remain identical.

Red then personally reaches 43 at unchanged health, but the run stops when the
next narrow slope has no waiting bay. The runner now follows successive original
approach points until it observes a supported bay with room. Each intermediate
point needs its own acknowledged, ordered personal arrival; none earns paired
waypoint credit early. Nine Python guards include skipped-intermediate-point,
wrong-owner and unacknowledged-approach rejection.

The next run has independently verified **43-45** evidence: six personal
arrivals, three parking walks, two forward waits and **869 matching
player/world/native states**, with no deaths or revivals in the closed 52-row
trace. Both finish that bounded prefix at **100/71.67 health**. The unchanged
normal checkpoint before 46 is retained. The parent attempt fails at 46 when
the planner snapshots Red during a still-active ledge pull; zero reported
velocity does not mean that the animation has stopped moving the body. This
prefix does not pass its failed parent attempt or complete the campaign visit.

Source `c7576ca` waits for an active ledge pull to finish before starting a
new search. It sends neutral input while the original animation completes;
a stationary hang can still plan the input that begins its pull. The regression
uses the recorded zero-velocity pull and rejects a stale grounded flag too.
All **913 active Rust tests** pass (three ignored), the locked release passes,
and the 26 native paired cases plus summary and single-player baseline outputs
remain identical, including the known cast-free step-54 failure.

A separate resumed **46-49** internet run passes on that source: eight personal
arrivals, seven parking walks, one combat wait and **1,017 matching
player/world/native states**. Both finish at **100/46.67 health**, with no deaths
or revivals in the closed 62-row trace. Ordinary combat damage at 49 is retained.
The unchanged normal checkpoint before 50 is saved. This segment does not
establish a fresh full route or add a completed campaign visit.

The following resumed **50-52** segment also passes on `c7576ca`: six personal
arrivals, two parking walks, one forward wait and **699 matching
player/world/native states**. Both retain **100/46.67 health** without new damage,
deaths or revivals; the closed host trace has 43 rows. A normal checkpoint before
53 retains both players on the approach, with the shared platform still raised
and stationary. The separate segment evidence now covers approach points 0-52,
but it is not one fresh continuous run on the latest build.

`tools/test_coop_jlair1.py` supports approach/platform steps 0-54, retains exact ordinary
checkpoint receipts and freezes its source dependencies in private evidence.
The fresh passing run covers only 0-3; later segments have separate evidence above.
The approach still needs a fresh continuous repeat, including climb reliability
from 32. The shared platform now has the paired crossing evidence below. The subsequent
climb, Caterpillar and onward transition are still open.
An explicit `--checkpoint` plus `--start-step` tests a separate resumed segment;
it never claims a fresh full route. Fifteen Python guards reject missing or
duplicated personal arrivals, unacknowledged movement, respawn substitutes,
invalid combat waits, revisited waiting bays, stale hazard exits, unsupported
hot takeoffs, jumps without an acknowledged airborne body and skipped forward
approach points.

```powershell
python tools/test_coop_jlair1.py --exe <frozen-executable> --build <full-source-id> --data <local-base-directory> --last-step 3 --save-before 0 3
python -m unittest discover -s tools -p "test_coop_jlair1_guards.py"
```

## Actual paired boarding and crossing

Source `9f1acd0` adds read-only deck contact, full-footprint boarding, sink
displacement and departure state to the local route report. It changes no
gameplay/save behavior. The final source passes 913 active Rust tests and the
locked release. Its 26 paired native cases plus summary, seven single-player
components and both full native route records match the recorded baseline.
The known cast-free single-player step-54 failure remains unchanged.

A resumed **53-only** internet segment passes both personal arrivals and
single-rider waiting. Red has full deck contact while Blue remains off it;
both clients observe at least 120 ticks with zero sink displacement and no
departure. All 317 comparable player/world/native states match. Both retain
100/46.67 health without damage, death or revival.

Earlier crossing attempts fail at the single-player fixture's sloped point
54, or while stopping there to request a new plan. It is a transit coordinate,
not a game trigger. The paired fixture now explicitly records its crossing
destination as the adjacent flat bank `[-7208,768,1632.03125]`; the original
course file and hash remain unchanged. It does not claim personal arrivals at
the old coordinate. Each Alice must instead make an actual ordinary jump to
this bank, with an acknowledged airborne observation and grounded arrival.
The second rider uses a short ordinary running start when leaving the right
boarding pad. No positions, health, collisions or level progress are edited.

A closed **54-only** segment now passes on the same frozen `9f1acd0` executable,
with two independent internet profiles, 40 ms delay and 20 ms jitter. Actual
platform waiting and carry, both bank arrivals, separation, cameras, stable
waiting and an ordinary checkpoint before 55 pass. All **152** comparable
player/world/native states match. Both retain **100/46.67 health** without new
damage, deaths or revivals.

A subsequent continuous **50-54** repeat also passes: **10 personal arrivals**,
two ordinary parking walks, one forward wait, two single-rider platform waits,
actual carry, two crossing jumps and stable bank waiting. All **1,145**
comparable player/world/native states match. Both retain **100/46.67 health**,
again without damage, death or revival. The normal checkpoint before 55 is
retained. The narrow bank's waiting point is verified through actual dry,
clear, grounded and stationary observations while the partner crosses; this
does not claim a full-margin respawn footprint on that bank.

Fifteen Python guards reject false arrivals, partial/early boarding, missing
carry, invalid climbs, falling/drifting/damaged waiting bodies and a transit
point substituted for the bank destination. A navigation regression guard
also verifies that ordinary steps still move both players and park the first;
the combined 50-54 run exercises that flow in the real clients.

These platform checks are resumed segments, not a fresh full visit. The
post-platform continuation and its remaining failures are recorded below. No additional campaign visit or percentage is credited; overall
progress remains 50.1%. No images were inspected.


## Post-platform traversal and remaining failures

The runner accepts navigation through 76, with separate forward-waiting sections
0-52 and 55-76. A waiting search cannot silently cross the shared platform or
continue into the unverified Caterpillar section. Every completed point still
requires an acknowledged grounded arrival for each player.

A closed **55-57** segment on `9f1acd0` passes six personal arrivals, ordinary
parking and combat, and **572 matching player/world/native states**. Both retain
100/46.67 health without new damage, death or revival. Its normal checkpoint
before 58 is preserved unchanged.

A later `a18e044` run completes **58-63**, then fails at 64. A bounded audit using
its frozen fixture, history and complete recovery-trace boundary verifies only
those six points: **12 personal arrivals, eight parking walks, one forward wait,
one combat wait and 1,326 matching states**. The pair ends that prefix at
100/76.67 health after ordinary damage and healing, with no deaths or revivals.
The complete attempt remains failed. A separate exact-checkpoint repeat of 58
on `0871d86` passes two arrivals and 270 matching states; it does not establish
that the earlier intermittent jump failure is eliminated.

Source `0871d86` lets the opt-in test driver finish an ordinary supported walking
approach before replanning a jump from the settled body. It uses the existing
route footing check, including its eight-unit margin; narrow walking surfaces
need not satisfy a full respawn footprint. Jump-first and airborne plans retain
their first landing. Native gameplay and the network protocol are unchanged.
A **64** prefix on this source passes both arrivals, ordinary parking and
**255 matching states**, retaining 100/76.67 health. Blue actually uses the new
grounded approach before jumping. The next point, 65, times out while planning;
the ordinary checkpoint before 65 is retained. This is bounded evidence, not a
successful complete run.

The 65 fixture now approaches the upper shelf via the original lower point 66,
as the single-player route does on its way around the eastern side. This extra
approach does not earn paired 65 or 66 credit. Blue reaches the shelf, but has
not yet waited safely for Red. A forward-waiting handoff previously selected a
brief grounded observation at about 270 units/second; it now requires a dry,
clear and settled body before interrupting navigation.

Grounded falls into damaging liquid can now request an ordinary jump to a
recently observed safe pad. The verifier distinguishes an actual jump from an
automatic bank climb and requires its native counters, action observations and
clear dry arrival. Only a grounded liquid-related no-route error is recoverable;
all other driver errors retain the shared runner's default failure. Each accepted
error must have a matching verified escape. One actual climb escape is verified
at 65, followed by another failed jump into deep lava. There is no paired 65
pass. A subsequent combined 55-64 repeat fails at 60 with Red swimming in lava,
where the grounded escape action does not apply. Its completed 55-57 prefix has
606 matching states, but the longer run is explicitly failed.

Final checks: **915 active Rust tests** (three ignored), locked release,
**19 Curiosity guards / 370 Python guards**, seven package checks and the
910-file source worktree audit pass. The 26 paired native cases plus summary,
seven single-player components and both full native route records match the
previous baseline. The known cast-free single-player 54 failure is unchanged.
No screenshots were inspected. Separate evidence now extends through 64;
continuous reliability, 65 onward, both scene owners, final exit, recovery and
reconnect coverage remain open. Overall completion remains **50.1%**, with no
additional completed campaign visit.


## Ordinary lava revival and moving spectator

The targeted `tools/test_coop_jlair1_recovery.py` fixture now passes for both
victims on frozen source `0871d86`. Each run starts from the same unchanged
ordinary checkpoint before 65, with two independent internet clients and
40 ms delay / 20 ms jitter. One Alice walks into actual lava using ordinary
inputs; the survivor walks to a native-observed safe neighboring pad during
the countdown. No damage, positions, carried resources or clocks are injected.

| Victim | Exact death tick | Exact revival tick | Matching player/world/native states |
| --- | ---: | ---: | ---: |
| Blue | 153821 | 155021 | 246 |
| Red | 153812 | 155012 | 241 |

Both full host traces prove exactly 1,200 active ticks (ten simulation seconds),
25 health on revival, unchanged carried resources, safe supported placement
within 128 units of the survivor, and no overlap. Neither trace pauses the timer
or waits for safe placement. Both client histories verify the correct living
camera subject and its rendered interpolation. Each dead player's camera must
also move with the survivor's acknowledged walk before revival; a static-camera
observation cannot pass this case. Control returns to the revived player.

The Blue run has 1,207 host trace rows; Red has 1,206. Both clients exit normally
after an ordinary saved checkpoint. Evidence folders under `private/coop/` are
`jlair1-recovery-12e77b7c7a184db583ca68b6e47bc46f` (Blue) and
`jlair1-recovery-e06a54f259844f71a8a97360368576c9` (Red). Each retains the frozen
fixture dependencies, source and executable identities, observations, complete
trace, and result. Earlier static-survivor cases also pass, while an initial
Blue probe that stopped on a dry ledge remains failed.

To reproduce, supply the frozen executable, its full source identity, local
game data and that unchanged checkpoint. Run once with each victim:

```powershell
python tools/test_coop_jlair1_recovery.py --exe <executable> --build <source-identity> --data <local-base> --checkpoint <checkpoint-before-065.json> --victim blue
python tools/test_coop_jlair1_recovery.py --exe <executable> --build <source-identity> --data <local-base> --checkpoint <checkpoint-before-065.json> --victim red
```

Five recovery guards reject wrong or repeated deaths, substituted controls,
survivor death/wipe, and unacknowledged, late or absent spectator motion. Clean
traversal still rejects all deaths and revivals. All **375 Python guards**, seven
package checks and the **912-file** source audit pass. Native source is unchanged
from the 915-test locked build and its recorded native/single-player baselines.
No screenshots were inspected.

These are targeted personal-revival cases. They do not prove paired point 65,
a full route, a team wipe, restoration from the new saved checkpoints, or
disconnect/rejoin. Those checks remain open. Overall stays **50.1%**, with six
campaign visits having combined evidence and no additional visit credited here.


## Supported landings and shelf guidance

Source `93a9dc3` corrects two failures in the opt-in automated route driver.
A descending body must have supported footing before handing control from
the jump plan to landing steering; merely overlapping a ledge with the body
box is insufficient. Delayed observations can use a clear velocity projection
onto the same supported footprint. A shortened plan also retains its actual
intermediate landing target, allowing Alice to brake on a lower ledge before
planning onward. Settling there cannot report arrival at the final waypoint.
These changes do not alter player physics, damage, collision or network rules.

Regression simulations reproduce the unsupported handoff and the fall caused
by missing intermediate steering. The latter now settles at 0, 6, 12 and 18
ticks of delay; the existing delayed-corridor case remains unchanged and passes.
In the real paired runs, Blue reaches the upper shelf and waits safely while
Red attempts the ascent. Increasing the navigation deadline from 90 to 240
seconds establishes that Red's earlier stall was an exhausted 100,001-state
search, rather than only an impatient test timeout.

The fixture now splits this ascent into ordinary navigation goals drawn from
observed native footings. Each player must earn separate, ordered receipts for
the lower approach, every guidance target and the original point 65. No guide
substitutes for the original upper-shelf arrival. The optional navigation timeout
is bounded, defaults to 90 seconds and is recorded with the evidence.

A bounded audit of `jlair1-route-e390fe9a7754411ba3a486f2e86390cf` verifies
both players' three guidance targets in that frozen fixture, plus Blue's
original upper-shelf arrival: **six guidance arrivals, 969 matching
player/world/native states and 58 complete recovery-trace rows**, without
death or revival through the audited boundary. Red's subsequent final ascent
exhausts 100,002 search states. The full attempt remains failed.

The current fixture adds a fourth observed footing before that final jump.
In `jlair1-route-fbab3f47085447ba81d4ceedbfeb5ec0`, Blue reaches all four
guides, the original upper shelf and a safe waiting bay. Red reaches the first
guide, then dies during the jump to the second. The final fourth-guide ascent
for Red is therefore still unverified. These attempts use the same unchanged
ordinary checkpoint before 65, two independent internet clients, 40 ms delay
and 20 ms jitter. There is still no paired point-65 or full-visit pass.

Validation passes **917 active Rust tests** (three ignored), the locked release,
**376 Python guards** including twenty Curiosity traversal and five recovery
guards, seven package checks and the 912-file source audit. The 26 paired native
cases plus summary, seven single-player components and full native combat-route
records match their previous baselines. The separate cast-free route retains
its known point-54 failure. No screenshots were inspected. Overall completion
remains **50.1%**, with six of 39 campaign visits carrying combined evidence.


## Delayed approach turns and retained run-ups

The failed guest approach on `93a9dc3` leaves the ground before its jump input
is applied. Its native jump count remains unchanged. The nearly identical
successful approach turns sooner. The route search reserved twelve ticks of
ground support for a delayed jump but did not reserve support for the preceding
direction change. Source `ac2c02a` applies that same optional reserve to grounded
turns. A narrow-corner physics regression fails under the old rule within the
100 ms reserve and passes with the new rule. Zero-margin native routes retain
their existing search behavior; gameplay physics and network rules are unchanged.

The next internet attempt crosses that approach, with Red actually jumping and
reaching the first shelf. Its following ascent repeatedly discards a backward
run-up at a supported stopping point. After 95 such planning segments, Red falls
and dies. Source `33885b1` retains a run-up through its first landing unless its
grounded prefix makes more than 24 units of progress toward the planned endpoint.
It still settles useful approaches and intermediate landings, and never reports
a run-up stop as arrival at the final goal. A physics regression checks that an
initial move away from the destination preserves the takeoff and real landing.

Both attempts start from the unchanged ordinary checkpoint before 65 and use
the same four-guide fixture, independent internet clients, 40 ms delay and
20 ms jitter. Bounded audits verify both lower approaches, four Blue guides,
two Red guides and Blue's original upper-shelf arrival:

| Source | Evidence folder under `private/coop/` | Matching player/world/native states | Complete recovery rows |
| --- | --- | ---: | ---: |
| `ac2c02a` | `jlair1-route-59cb57cb436b433eb993c6d2bf78d2f8` | 1,430 | 88 |
| `33885b1` | `jlair1-route-61f1328282064125bf782657a0619c59` | 1,285 | 79 |

Each bounded proof ends before the subsequent failure, with both alive and no
revival. The latest complete attempt also fails: Red now takes jumps toward
the next shelf but falls and dies, while Blue remains safely parked at 87 health.
The earlier endless run-up loop is absent in this attempt; the next ascent is
still unreliable. Neither attempt passes paired point 65 or a full visit.

Current validation: **919 active Rust tests** (three ignored), locked release,
the existing **376 Python guards**, seven package checks and the 912-file source
audit. The paired 26 native cases plus summary and the single-player component
and combat-route outputs match their prior baselines; the separate cast-free
point-54 failure is unchanged. No screenshots were inspected. Overall remains
**50.1%**, with six of 39 visits retaining combined evidence.


## Current-build campaign restart, 5 November 2026

The focused core save/rejoin milestone is complete. Campaign testing resumes
on frozen source `0b5bca3` without changing gameplay or the route fixture.
All native Curiosity comparison outputs are byte-identical to `33885b1`:
26 paired cases plus summary, seven single-player components and the complete
combat-route check. The cast-free traversal still fails at point 54. The first
long combat-route comparison ended at point 68 with Windows `AppHangB1` and
exit `3489660927`; its Anode repeat completes in 92.101 seconds and matches the
baseline. Both results are retained in `private/jlair1-core-0b5bca3/`.

The earlier supported-goal-walk change is present: an actual grounded body may
replace a queued pull-up with a clear, continuously supported walk to its goal.
Airborne bodies, gaps and different floors retain their pending jump. Its
paired point-65 verification was deferred during core work and remains open.

The unchanged ordinary checkpoint before 65 has SHA-256
`bbcfeed3258fc6268d5503968e0c08939ab6058300bc63cf40b9b0863167cfff`.
The new public attempt `jlair1-route-17cbc45b06234222a0cfc85c26a24151`
fails after 128.884 seconds. Blue reaches all four guides, the original shelf
and a waiting position. Red reaches guide zero, then the seven-segment timed
plan misses the next platform sideways and falls into lava. This is earlier
than the deferred obsolete-pull case; no paired point-65 pass is claimed.

A private prototype replaces only guide-zero-to-one's timed turns with a
200 ms ordinary run-up, one actual jump and feedback air steering to an
observed Blue landing. Both original guide and final waypoint requirements
remain. In public attempt `jlair1-route-33268d012b5e4f6285a6124be4467ac1`,
Blue's takeoff/airborne/grounded-arrival ticks are 154317/154366/154530, followed
by all four guide receipts. The guest then exits with the generic transport
read error while the host remains running. No log establishes whether the
connection or packet processing caused it. Red never attempts this crossing;
the whole attempt is failed, not a network or paired-movement pass.

The same private prototype on delayed loopback,
`jlair1-route-ec368b2b25594cee853148bad77c14a8`, fails after 91.107 seconds
on an earlier approach, before Red reaches guide zero. At host tick 162525,
the recorded jump input is true, but Red is already ungrounded at height
1927.96, below the 1984.03 ledge. Her native jump count stays 44 across the
input. This proves the action arrived too late to jump; it does not establish
which timing stage caused the delay. The plan, native input event, movement
history and timing summaries remain available for diagnosis. The next work is
to make this takeoff reliable before retrying the later ascent. Neither private
prototype has been promoted to the checked-in fixture.

These runs use separate processes/profiles, 40 ms added delay and 20 ms jitter.
No state, health, inventory or checkpoint bytes were edited to progress. No
screenshots were inspected. Overall remains 56.6%; no additional campaign visit
or completed paired point is credited.


## Automated takeoff clock, 6 November 2026

Frozen source `1a6b3ea` corrects the opt-in route driver's movement clock.
The guest input timeline uses elapsed wall time and the host retains simulation
debt after a brief stall. The route clock still discarded all but 100 ms per
frame. After a slow guest frame this could prolong an already expired run-up,
so the jump arrived only after the body had fallen off the ledge. The clock now
consumes elapsed time, advances at most once per sampled frame, and grants the
next previously unsampled input its full 200 ms interval after a stall. Normal
8/16/33 ms frame remainders are preserved. This changes diagnostic control only;
player movement rules, manual input, networking and save formats are unchanged.

The optional native `recorded_curiosity_takeoff_probe` test uses local geometry
via `LOOKING_GLASS_TEST_DATA`. From the observed ledge position, the host keeps
ticking while the guest's next two input samples occur at 300 and 400 ms. The
old clock issues the jump at tick 48, already airborne at height 1933.41, and
the native jump count remains zero. The new clock issues it at tick 36 while
grounded at height 1965.32 and records one actual jump. This is a deterministic
reproduction of the failure mode; the precise frame intervals in the earlier
live capture were not established. An ordinary unit test also rejects skipping
or shortening the next unsampled step after this stall.

All four current-build native comparison outputs are byte-identical to their
baselines: 26 paired cases plus summary, seven solo components, full combat
route, and the separate known cast-free point-54 failure. The latter still
exits 1 and is not a traversal pass. Results are retained under
`private/jlair1-clock-1a6b3ea/`.

Two public-invite attempts resume the unchanged checkpoint before 65, with
40 ms added delay, 20 ms jitter and independent processes/profiles. Both reach
all four guidance points with each player, plus Blue's original upper shelf.
Neither completes the paired original point 65:

| Evidence folder under `private/coop/` | Full attempt | Bounded matching player/world/native states | Recovery rows through last guide |
| --- | --- | ---: | ---: |
| `jlair1-route-456b53ea0b23439ebdda321509565062` | Failed, Red search exhausted 100,001 states | 1,709 | 100 |
| `jlair1-route-f96b698060444904b5eeb49fd8aadca5` | Failed, Red search exhausted 100,001 states | 992 | 59 |

The first run uses the unchanged checked-in guide targets. Its bounded proof
ends at tick 165408; Red's later final search fails from
`[-6379.8335,3217.4895,2088.0313]`. The second uses the observed starting footing
of Blue's successful final plan as its fourth guide. Its bounded proof ends at
tick 160449; Red's later final search fails from
`[-6373.0400,3210.4133,2088.0313]`. This adjustment did not resolve the search
failure and has been reverted. The final checked-in fixture retains all four
original guidance points and the original upper-shelf requirement.

Each `verified-shelf-guidance.json` independently checks the saved fixture
hashes, immutable checkpoint, exact restored state, ordered arrival receipts,
grounded/support conditions, living unmodified players, camera/presence,
network equality and recovery trace through its declared end tick. Both
explicitly retain `full_attempt_passed: false` and `full_route: false`. Neither
prefix contains a revival. These are stronger lower-climb observations, not a
complete route or a guarantee that earlier movements are now always reliable.
The next unresolved case is Red's final ascent search; raising its state bound
or weakening the arrival checks has not been used to claim progress.

Validation: 935 active Rust tests pass (four ignored), the optional native
takeoff test passes explicitly, locked release builds, 427 Python guards and
seven package tests pass. No screenshots were inspected. Anode test windows
are closed and the lease released. The core readiness evidence remains on
its separately frozen `0b5bca3` build. Overall stays 56.6%, with six combined,
21 partial and 12 unrun campaign visits; no new paired point or visit credit.


## Remote approach directions, 6 November 2026

The opt-in remote route search now retains the direction used to arrive at a
search cell. Its delayed-turn safety check continues that preceding input;
merging arrivals from different directions discarded potentially safe paths.
The native reproduction failed after 100,002 states from the settled guest
position, only 1.56 units from an earlier successful start. Keeping the incoming
direction finds a route from both positions and the exact later live failure.
The search limit, twelve-tick takeoff reserve, eight-unit first-landing support
and normal player physics remain unchanged. Zero-margin native routes retain
their previous state equivalence.

The optional native `remote_curiosity_ascent_retains_approach_directions` test
replays each selected input through actual movement/collision physics at all
three recorded starts. It requires actual jumps, clear bodies, dry travel,
grounded arrivals within 22 units, supported first contacts, and the full late
input interval at grounded turns/takeoffs. The test passes explicitly on frozen source `c893e48`; all
935 active Rust tests pass (five ignored), the locked release builds, 427 Python
guards and seven package tests pass. All four Curiosity native comparison logs
match their baselines, including the known cast-free point-54 failure. That
failed comparison remains a failure, not a completed traversal.

The paired fixture retains all four original ascent guides and adds an observed
higher ledge before the unchanged original point 65. On the earlier `1a6b3ea`
runtime, both players reached all five guides and Blue reached point 65, but
Red's final search failed. Its bounded independent proof verifies ten guide
arrivals, 1,845 matching player/world/native states and 108 recovery rows without
revivals. That attempt remains failed and adds no paired point or visit credit.

The current `c893e48` public-invite run completes the resumed **65-76** segment,
with both players reaching every original goal: 24 personal arrivals and all
ten guidance arrivals. Independent processes/profiles use 40 ms delay and
20 ms jitter. The frozen-fixture audit rechecks exact checkpoint restoration,
ordered arrivals, observed waiting bays, player/camera presence, saved checkpoint
hashes, 7,419 matching player/world/native-enemy states and 431 recovery rows.
Both finish alive at 92/77.67 health without revival. Ordinary checkpoints
before 65, 66, 70, 74, 76 and 77 are retained.

Evidence is `private/coop/jlair1-route-a6b6947a160e4842bf28b4bcfd91fd74/`, including
`verified-direction-segment.json`. The formerly stalled paired point 65 is now
verified. This is one completed segment, explicitly `full_route: false`; a fresh
continuous repeat, the ending, both scene owners, recovery/rejoin combinations
and complete campaign consolidation remain open. No state or checkpoint bytes
were edited to progress. No screenshots were inspected; both test clients
exited normally, owned windows are closed, and Anode is released.
Overall remains 56.6%, with six combined, 21 partial and 12 unrun visits.

## Resumed ending fixture, 6 November 2026

`tools/test_coop_jlair1_ending.py` starts from the unchanged ordinary checkpoint
before point 77. It watches the original Caterpillar scene with a chosen first
player, checks all fifteen scene stages on both clients, keeps the partner
hidden and frozen, and verifies the original actor endpoint and safe reunion.
It then requires personal arrivals at every ordinary ending goal. Later checks
cover individual Meta-essence, ordinary Watch activation/cost, an observed jump
and ledge pull, and actual onward travel. These later checks are implemented
but have not yet passed in a live ending run. Every result is explicitly a
resumed segment, not a complete visit.

The opt-in route driver's `use_watch` command selects the earned weapon and
holds normal primary input; ownership, Will cost and recharge remain ordinary
authoritative game behavior. It cannot mix with navigation, combat or checkpoint
commands. Normal player input, physics, saves and the network protocol are
unchanged. The point-86 item is Meta-essence; personal Jacks were earned earlier
and are required in the starting checkpoint. The pickup verifier accepts either
health restoration or Will restoration beyond normal regeneration, while
rejecting shared consumption or shared resource gains. That verifier correction
has guard coverage only; the live run has not reached this item.

The first public attempt successfully watched Blue's scene and reunited both
players, but failed an overstrict resource assertion. Independent observations
arrived one and twenty-two ordinary ticks after the authoritative reunion.
Only normal Will regeneration and loot aging had occurred. The corrected
verifier models these bounded idle ticks, including ordinary drop downgrade
and expiry; it still rejects other resource changes. A separate audit retains
that attempt as failed and proves only its watched-scene boundary, with 2,276
matching player/world/native states.

The second public attempt on frozen source `ddaf73b` passes the watched Blue
scene and paired **77-82** prefix: twelve personal arrivals, safe waiting bays,
5,219 matching player/world/native states and no revival. The ordinary checkpoint
before 78 is retained. The full attempt exits with failure after point 83 times
out: Blue remains below an intermediate planned landing and never confirms a
settled arrival. The Red-owned scene, remaining ending, pickup, Watch use,
climb and onward travel are still open. No timeout or arrival guard was relaxed.

The first evidence folder is
`private/coop/jlair1-ending-2db7bcb4fdef48619b6d14ae241aedd7/`, with
`verified-scene-boundary.json`. The second is
`private/coop/jlair1-ending-9899fbdb178b44599a060163e05f3e32/`, with
`verified-ending-prefix.json`. Both audits keep `full_attempt_passed: false`
and `full_route: false`; saved observations and frozen fixture dependencies
are retained unchanged.

Validation: 936 active native tests pass (five ignored), the locked release
build passes, and 439 Python guards plus seven package tests pass. The paired
native cases, single-player components and full native combat route preserve
their baselines; the existing cast-free point-54 failure remains a failure.
No images were inspected, owned windows are closed and Anode is released.
The separate core save/rejoin milestone remains on `0b5bca3`. No complete
campaign visit or additional percentage credit is claimed.

## Playable route continuation, 7 November 2026

Seventeen retained ordinary save/resume sections now form a verified chain
from the entrance through point 76 to the pre-Caterpillar checkpoint. Replaying
their frozen assertions confirms 29,251 matching player/world/native samples;
every input matches the preceding saved bytes and native exact-resume receipt.
This reuses historical builds and does not complete the still-unverified ending.

On `40d06c0`, a new public run verifies Blue's watched Caterpillar scene, safe
reunion and both players' points 77–82, with 4,938 matching states. The run was
stopped after reproducing the stationary diagnostic landing at point 83; its
full attempt remains failed. No additional route credit is claimed.

The Python controller now requests a fresh path when it observes two seconds
of grounded, dry, stationary intermediate settling. It retains the original
goal and arrival bounds, without changing any player or world state. The ending
fixture accepts `--start-step 78`, `83`, `86`, `90` or `98` with an unchanged ordinary
post-scene checkpoint. It checks that the saved scene is complete, verifies
exact restoration and does not claim a freshly watched scene for those runs.
It also saves before point 83, so later retries need not repeat earlier climbs.

The playable mode (`--playable`) leaves the optional off-route Meta-essence and
separate Watch choreography for polishing. The strict detour attempt died at
86 and remains failed. The required route passes 83–85 and 87–89 in separately
verified checkpoint segments (1,082 and 826 matching states). A first attempt
to park both bodies at the narrow point-90 setup failed in the harness; a
direct long search for the upper landing also failed. The final coordinated
climb uses each player's original 90/92 approach in turn, ordinary jumping and
ledge pulling to 97, and a waiting position at 98 for the leader. Both climbs
and all five approach/waiting arrivals pass an offline checkpoint audit, with
918 matching states and no revival. No health, positions or progression were
written by the fixture, and the final arrival bounds remain unchanged.

The full attempt later failed while finding a forward waiting position at 99.
Both players' verified climb is retained in the ordinary checkpoint before 98;
`--start-step 98 --playable` continues from it without claiming another climb.
The upper route and actual exit are still unverified, so Burning Curiosity
remains outside complete-route credit. All 37 Curiosity guards and seven
package checks pass; native code is unchanged from the tested `40d06c0` build.
Test windows are closed and Anode is released. No screenshots were inspected.

## Complete playable save chain, 7 November 2026

The upper-route test now sends the leader through the original points 98–105
and to the exit approach at 106, then follows with the partner. Both perform
an ordinary jump from the original point-99 approach toward flat ground beyond
the sloped lip. Continuing through incidental liquid contact keeps actual
damage and death checks active. This avoids the earlier parking/search failure
and the delayed braking that sent the guest back down the slope.

The recorded upper prefix verifies 17 personal original-point arrivals, both
jumps, 2,559 matching states and the actual checkpoint before 107. Its whole
attempt still failed because the exit validator required Alice to be grounded
at the first frame after the original arrival scene. That scene releases her
above the floor. The corrected validator requires an observed clear, living,
grounded pair within 120 ticks of scene release; a regression guard rejects
late, blocked or still-airborne arrivals. `--start-step 107 --playable` resumes
the untouched exit checkpoint without claiming another upper traversal.

The final public-invite exit run passes on `40d06c0`, with 1,383 matching states
and both players safely in `jlair2`. Joining its exact saved input to the
verified upper prefix and earlier sections completes a 24-section normal route
chain, with 43,586 matching states across its retained evidence. The private
`coop-playable-curiosity-complete-chain-audit.json` records the hash connections.
This earns one complete playable route; it is not a fresh uninterrupted run.
Optional pickup/Watch choreography and exhaustive repeats remain polishing.

All 38 focused Curiosity guards and seven package checks pass. Native code is
unchanged from the frozen build with 944 passing active Rust tests. No images
were inspected during this continuation.
