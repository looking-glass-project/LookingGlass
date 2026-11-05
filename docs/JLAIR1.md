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
