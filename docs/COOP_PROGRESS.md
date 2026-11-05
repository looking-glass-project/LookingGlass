# Co-op progress

Updated 5 November 2026. Branch: `coop`. **Estimated overall completion: 50.1%.**
The older 42% reports are historical. This percentage measures delivered and
verified behavior against the full co-op scope; it is not a running test's progress.

Six of the 39 campaign visits have combined verification evidence; 33 remain open.
Earlier contact-system repeats are still required. Remaining work includes full
paired traversal, both scene owners, deaths and checkpoint recovery, reconnects,
shared transports, swimming/leaves and single-player regressions across the campaign.

| Area | Credited points | Maximum |
| --- | ---: | ---: |
| Foundation and independent client diagnostics | 8 | 10 |
| Internet Host/Join | 7 | 15 |
| Two players, movement and presentation | 11 | 15 |
| Combat, individual resources and death | 9.75 | 20 |
| Scenes, transport and progression | 11.25 | 20 |
| Full campaign verification | 3.08 | 20 |
| **Total** | **50.08, rounded to 50.1** | **100** |

Latest native milestone: Burning Curiosity source `56160ff` passes **26 staged
cases plus one summary**. Each living player receives the original arrival
grant once and retains their own two essence pickups. The sinking platform waits
for both living riders, carries them at 30/60/144 Hz, transfers them onto the
uneven floor independently and returns when empty. A blocked return rolls back
the platform and both bodies. Original arrival and first-entrant Caterpillar
scenes pass watched/skipped, living/dead partner and compressed-save cases.

The locked release passes seven single-player component checks and the full
native combat route; both outputs match the previous baseline exactly. The
separate cast-free traversal still has its pre-existing step-54 lava-jump
failure, also unchanged. These results add **0.75 points** under the existing
component weighting: 0.25 resources, 0.25 movement and 0.25 progression.
No additional campaign visit is counted as complete. Next is Burning Curiosity's
full route with two independent internet clients, followed by recovery and
reconnect checks. See [Burning Curiosity evidence](JLAIR1.md).

Current validation: **913 active Rust tests** pass (three ignored), along with
the locked release, fifteen Python evidence guards, seven package checks and
the 910-file source worktree audit. Negative tests cover partial boarding,
floating bodies, marginal ledges, damaging-liquid navigation and stationary
bodies that retain collision velocity, alongside valid sloped landings.
No images were inspected for these milestones.

Burning Curiosity's first fresh internet segment now passes **steps 0-3** with
eight personal arrivals, watched entry, both early lava crossings and 2,133
matching player/world/native states. Both survive without revival at 98.75/95.42
health; a normal checkpoint before step 4 is retained. The longer approach and
full route remain open. This segment adds no percentage or completed-visit credit.
A later resumed attempt has independently verified evidence through **4-20**:
34 personal arrivals, five forward waiting walks and 5,582 matching states, with
no deaths. That longer attempt fails at guest planning for step 21; its successful
prefix is bounded separately. This also adds no percentage or completed visit.
Source `ca4cc26` then corrects the automated planner's floor-plane checks, keeping
its eight-unit support margin and requiring actual body contact. The previously
stalled climb now passes in a separate resumed **21-23** internet run: six personal
arrivals, one forward waiting walk and 2,224 matching states. Both finish at
100 health without revival; a normal checkpoint before 24 is retained. Native
component and single-player baseline outputs remain identical.
Later bounded **24-25** evidence adds four personal arrivals and 1,563 matching
states. Two attempts fail at 26 under fire-spitter attacks. The runner now allows
ordinary combat dodging before parking and uses the original route's Cards
preference. A resumed **26-29** run passes eight personal arrivals, the fight and
1,374 matching states, ending at 50/96 health without death or revival. Its normal
checkpoint before 30 is retained. A further attempt verifies **30-31** separately
(four arrivals, 352 matching states), then fails the forward climb from 32 toward
33 when Red falls into lava. A separate retry verifies **30-32** with six
arrivals and 783 matching states, taking a different ascent before failing
parking at 33; climb reliability remains open. Forward waiting now uses actual
native dry bays instead of treating the next route point as a safe stopping
place. Bounded **33-37** evidence has ten arrivals and 1,309 matching states.
A separate bounded **38** replay uses ordinary jumps onto the original hot
waypoint and back onto dry ground: two arrivals and 462 matching states,
93.75/41.67 health, no deaths or revivals. Its longer attempt fails at 39 when
the planner walks into lava. Those full attempts stay failed. Source `259d4f5`
adds optional liquid avoidance and corrects blocked-walk detection to use actual
position progress instead of retained collision velocity. A resumed **39-42**
internet replay now passes eight personal arrivals and **1,335 matching states**,
ending at 100/56.67 health with no deaths or revivals. The normal checkpoint
before 43 is retained. A subsequent remote three-jump sequence fails at 43.
Source `5c4a693` stops timed sequences after the first planned landing and settles
the actual body before planning onward. The runner also follows successive
original approach points when a narrow slope has no nearby waiting bay, with
separate personal receipts for every intermediate point. Bounded **43-45**
evidence then passes six arrivals and **869 matching states**, ending at
100/71.67 health without deaths or revivals. That longer attempt fails at 46
when a new plan starts during an unfinished ledge pull. Its normal checkpoint
before 46 is retained. Source `c7576ca` then waits for active ledge pulls to
finish before starting another route search. A separate **46-49** internet run
passes eight personal arrivals and **1,017 matching states** at 100/46.67 health,
with no deaths or revivals. A separate resumed **50-52** segment passes six
arrivals and **699 matching states** with no further damage, death or revival.
The normal checkpoint before 53 retains both players beside the raised platform.
Separate evidence covers approach points 0-52; a fresh continuous repeat is
still required. The later platform checks below do not complete the remaining
route, recovery or reconnect coverage.
A separate **53** internet segment on `9f1acd0` now passes both personal
arrivals and actual single-rider waiting: Red has full native deck contact,
Blue remains off the platform, and it stays raised for at least 120 ticks in
both clients. There are **317 matching player/world/native states**, no new
damage, death or revival, and a normal checkpoint before 54. This build only
adds read-only platform contact diagnostics; the native platform cases and
single-player outputs remain identical.
The paired platform crossing now passes in a closed **54-only** segment:
both players reach the opposite bank alive, Blue waits there for Red, a normal
checkpoint before 55 is saved, and 152 player/world/native states match.
A longer **50-54** repeat then passes ten personal arrivals, actual boarding,
waiting and carry, both ordinary jumps and stable bank waiting. All **1,145**
comparable player/world/native states match; both retain **100/46.67 health**
without damage, death or revival. The current checkpoint before 55 is retained.
The crossing destination is explicitly the flat bank `[-7208,768,1632.03125]`,
instead of the single-player fixture's sloping point 54. That point is not a
game trigger. The original course file is retained and the changed crossing
goal is recorded; this is no claim of landing at the old coordinate. Only
normal controls are used, with native collision, resources and level behavior
unchanged. Remaining route steps 55 onward, a fresh complete visit, recovery
and reconnect coverage are still open. No screenshots were used.
Total remains **50.1%**; no extra completed visit is credited.

Latest network milestone: a **fresh full About Face route passes** in two
independent internet clients on source `3298768`, using ordinary controls with
40 ms delay and 20 ms jitter. Ten personal arrivals, both reward pickups, the
shared lift, Red-owned watched Gryphon exit and actual shared `jlair1` arrival
are verified. The closed evidence has **6,805 matching player/world/native
states**. Blue's death at 23,061 revives at 24,261 with exactly 25 health and
verified resources, placement and spectator/returning cameras. A separate
resumed ending verifies Blue ownership with 2,169 matching states.

Latest recovery milestone: Red also revives after exactly 1,200 active ticks
at 25 health, with resources, safe placement and cameras verified across 1,087
matching states. A fresh guest process rejoins an unchanged normal victory save
with personal resources, pickup ownership and shared progress preserved: 77
matching states, including 58 after rejoining. Timed drops age normally.

Latest wipe milestone: both players die through ordinary combat and return
together at tick 7,060 to the normal checkpoint saved at 3,421, at 100/100 health.
Source `116fc99` adds an opt-in exact restore receipt before another tick can
change the world. Characters, enemies, pickups, shared progression and personal
resources match the actual saved checkpoint. Both clients agree on 1,058 states,
including 27 after restoration; the closed 917-row trace verifies the boundary.

All 905 active Rust tests, the locked release, 83 native cases plus three
summaries, 16 About Face Python guards and seven package checks pass. The
single-player component and full-route outputs still match the baseline exactly.
Renderer counters from the earlier full route show particles/effects in both
clients with zero dropped batches; no images were inspected. Combat/transport/
transition reconnects and remaining ending cases keep this visit partial.
The remaining About Face checks stay open in the matrix. This recovery milestone
retained the then-current **49.3%** estimate and six combined visits.

Earlier native milestone: About Face source `3298768` passes **83 staged cases
plus three summaries** for combat, personal resources, both lifts and the original
scenes. The bridge waits for both players to clear it; lifts enforce boarding and
carry living riders together, including obstruction rollback and catch-up from a
supported destination. Each player retains their own rewards. Legacy saves and
low-health revivals keep their health. Gryphon scenes bind the first entrant's
pose on the trigger frame and preserve the partner through watched/skipped play.

**905 active Rust tests** (three ignored), the locked release, seven package
checks and the 900-file source worktree audit pass. Both single-player checks,
including the full input route, match the unchanged baseline exactly. No images
were inspected. Under the existing component weighting this adds **0.75 points**:
0.25 combat/resources, 0.25 movement and 0.25 progression. The total then was **49.3%**.

No campaign row is completed by staged components. Six of 39 retain combined
verification evidence. About Face's remaining recovery/reconnect and
ending cases stay open. Clockwork's full-route and
recovery gaps also remain. See [About Face evidence](HATTER2.md).

Latest completed traversal sections: resumed **144-145** and **146-149** pass.
Both players enter the lowered clock portal through ordinary jump/climb input,
then personally reach the final lever approach. The first arrival leaves the
portal exit clear for the second. This section has **361 matching
player/world/native states** and retains the normal checkpoint before 146.

Red then uses the final lever and owns the watched 30-second Stop scene. Blue
stays hidden and still; both reunite safely. After the original 20-second wait,
Red approaches and enters the original exit, moving both players into About Face.
The shared transition has its own receipt, rather than claiming that both
players walked to the exit waypoint. This section has **1,122 matching states**,
including **27 after arrival**, with health preserved at **26.93/23** and personal
resource, camera and body-clearance checks passing. No new deaths occur.

Earlier resumed 141 verifies Red's exact 10-second revival and a new personal
chair approach; 142-143 verifies onward waiting. All **210 Clockwork Python
guards**, **15 generic recovery guards** and seven package checks pass. Those Clockwork runs used native
source `84c8eee`; its previous 905 active Rust tests, 72 paired/driver checks and
exact single-player 11+1 baseline were not rerun for those Python-only changes.

Remaining Clockwork work: fresh full-route, remaining scene-owner, recovery and reconnect verification.
These checkpoint segments through the exit do not complete the full visit's
verification matrix or add another campaign row.
See [Clockwork evidence](HATTER1.md).

Earlier diagnostic work: source `84c8eee` detects an actual off-course landing
before sending later jumps from the old plan. It also reports nearby dry,
fully supported return targets from native geometry, without moving either
body. The chair fixture requires the real jump and landing on that target.

The new resumed **140** replay records both personal chair arrivals, one Red
pull-up and Blue's safe return, with **2,312 matching player/world/native
states**. It still **fails**: Red reaches the chair with about **8.53 health**
and dies landing after the return jump. Her approach already incurred normal
damage, including slime. No rescue warp or revival occurs before either arrival.
The earlier replay timed out during a corrected route search; the bounded
approach allowance is now 180 seconds, while departure remains 30 seconds.
Neither failed replay earns a completed paired step or campaign visit.

Validation passes **905 active Rust tests** (three ignored), the locked release,
**72 native paired/driver checks**, the exact single-player **11+1** baseline,
**169 Python guards** and seven source-package checks. The 40 scene cases still
belong to earlier source `04d3b0c`. A survivable third-chair approach/departure,
the fourth chair and later route remain open. Overall stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Previously completed traversal section: resumed **138** now passes for both players.
Each reaches the original chair waypoint, then uses an observed native jump to
return to dry ground before the partner approaches. The run verifies the puzzle
contact, two personal arrivals, two departures, one observed Red pull-up and
**1,828 matching player/world/native states**. Both survive at **67/67.73 health**;
ordinary slime and combat damage is retained. A normal checkpoint before 139 is
saved. The earlier failed waiting arrangements remain recorded separately.

The next requested 139-143 run completes **139**. Its separately audited section
adds two arrivals, two departures and **1,741 matching states**, ending at
**54/43.53 health**. Step 140 remains unverified: Blue's selected return point
is beneath a higher landing, and Red's delayed jump route triggers the original
rescue portal before a new search times out. The runtime now enforces the same
return-height check as the final verifier, so it stops at that mismatch earlier.

The fixture returns to the first nearby dry footing from the actual approach.
A reproduced selection bug chose a later point beneath the chair rim, leaving
Blue above her target; its regression now rejects that choice. All **166 Python
guards** and seven source-package checks pass. Native source remains `99a66d6`,
with **901 active Rust tests** (three ignored), the locked release, **72 native
paired/driver checks** and the exact single-player **11+1** baseline. The previous
**40 scene cases** belong to source `04d3b0c`; they were not repeated for these
diagnostic changes. The remaining chairs, second portal, final scene/exit and
full-route/recovery/reconnect repeats remain open. No additional campaign visit
is credited; overall stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier completed traversal section: Clockwork's resumed **136-137** portal section passes
with two personal approach arrivals, two input-driven native climbs, separate
portal crossings, one waiting handoff and **202 matching player/world/native
states**. Both players remain alive at **80/100 health**; the leader takes two
ordinary hits while waiting beyond the portal. No health is reset for the test.

The preceding **128-135** doorway section is separately audited with ten
personal arrivals, five waiting handoffs, three timed holds and **562 matching
states**, ending at 100/100 health. The first player observes both rotating
leaves opening and walks forward before the partner approaches. The original
route's later wait and both players' waypoint arrivals remain required.

A reproduced portal bug could place one Alice directly inside the other.
An occupied exit now waits for the living partner to move clear. The test
controller also preserves a consumed jump across teleports, while allowing
new commands and new visits to jump normally. The retained failed portal run
is rejected by the verifier for its repeated jump at the destination.

Frozen source `04d3b0c` passes **897 active Rust tests** (three ignored), the
locked release, **72 native paired/driver checks**, the unchanged single-player
**11+1** baseline and **40 staged scene cases**. All **151 Python guards** and
seven source-package checks pass. The following **138-143** attempt times out
at the first chair approach after an incomplete remote ledge pull; it earns no
completed steps. Both players remain alive at 80/90 health. The chair section, second portal, final
scene/exit, full-route and recovery/reconnect repeats remain open. No additional
complete campaign visit is credited; overall remains **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **109-116** section passes with
eight personal arrivals, the watched Blue-owned Gryphon scene, four waiting
handoffs, three timed holds and **1,353 matching player/world/native states**.
A separate Red-owned Gryphon replay of **112-113** passes with **960 matching
states**. Both runs finish at 100/100 health and verify all seven dialogue
stages, hidden/frozen partners, preserved resources and safe shared reunions.

The next requested **117-136** run stops at 128 because its narrow approach
offers no usable sideways waiting position. Its separately audited **117-127**
section passes with 22 arrivals, 20 pathnode projections, 11 waiting handoffs
and **1,514 matching states**, also at 100/100 health. A normal checkpoint
before 128 is available for the next retry. The original failure is retained.

Native source remains `11c5ba2`; all **40 staged paired scene cases** now pass
on this exact build, including both owners, watched/skipped scenes, pause/restore
and living/dead partners. All **131 Python guards** pass. No further complete
campaign visit is credited. Later traversal, portals, the final scene,
recovery/reconnects and full-route repeats remain open; overall stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **107-108** doorway section passes
on source `11c5ba2`, with four personal arrivals, two observed door approaches,
one forward handoff, one waiting handoff and **692 matching player/world/native
states**. Both players finish at 100 health, and an ordinary checkpoint before
109 is saved. The runner lines up at authored waiting points and observes both
rotating leaves opening before continuing to the original waypoint. Its precise
walking handoff now checks walls while permitting normal step-up clearance.

All **893 active Rust tests**, the locked release, **72 native checks**, the
unchanged single-player **11+1** baseline, **124 Python guards** and seven
source-package checks pass. Earlier unsuccessful doorway replays are retained.
This completes the previously stalled section, not the whole visit. Later
Clockwork scenes/traversal, recovery/reconnects and campaign repeats remain open;
overall completion stays **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's completed resumed **99-106** section
verifies 16 personal arrivals, two pathnode projections, seven forward handoffs,
one waiting handoff and **1,845 matching player/world/native states**, with both
players at 100 health. The requested 99-108 run times out at 107 while the leader
approaches its next waiting point. It stops about 16 units short with the native
controller reporting `settling landing`; this remains the next investigation.
The original failure and the separately audited completed section are retained.

This batch also verifies **81-93** (2,421 matching states, 46/85 health) and
**91-103** (3,181 matching states, 100/100 health), each scoped separately from
its original longer-run failure. The test controller now coordinates narrow
stairs and winding paths, permits ordinary defensive movement while waiting on
static ground, and bounds an unsuccessful melee approach before allowing a
normal Blade throw. Source `cefabf5` passes **891 active Rust tests**, the locked
release, **72 native checks** and the exact single-player **11+1** baseline;
all **116 Python guards** pass. No additional complete visit is credited.
Later machinery/scenes, recovery/reconnects and full campaign repeats remain
open. Overall stays **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **68-81** section passes with
26 personal arrivals, 16 pathnode projections, 13 waiting handoffs, one timed
hold and **2,294 matching player/world/native states**. Both finish alive at
16.53/100 health. The combat test controller now requires supported footing
under its full stance through delayed movement and braking. The regression
fails on the old controller and passes with the correction; the real internet
replay also passes the fight that previously killed Red. Source `894292f`
passes **890 active Rust tests**, the release, **72 native checks**, the exact
single-player **11+1** baseline and **110 Python guards**. Ordinary checkpoints
allow targeted retries near this fight and the next route section.

The preceding requested 68-108 run remains failed at 77, with a separate scoped
audit crediting only 68-76 and 1,237 matching states. Later Clockwork traversal,
recovery, reconnects and complete campaign repeats remain the bottleneck. No
further complete campaign visit is credited; the estimate stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **60-67** section passes with
nine personal arrivals, two native scene handoffs, four waiting handoffs, one
lever, two timed holds and **2,971 matching player/world/native states**. Blue
owns the watched Hare scene and Red owns the delayed Port scene. Separate
retained runs verify Red's Hare scene (**1,791 matching states**) and Blue's
Port lever/scene (**378 matching states**). Both stay at 88/78 health, and the
scene checks require hidden/frozen partners, preserved resources and safe shared
reunions. Scripted reunions are recorded separately from walking arrivals.
All **110 Python guards** pass; native source remains `6f589de`. An intervening
startup failed DNS resolution; the next new internet session passed. Later
Clockwork sections, skipped-scene/recovery/reconnect repeats and full traversal
remain open. Overall completion stays **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **44-59** section passes with
32 personal arrivals, 32 independent pathnode projections, 16 waiting handoffs
and **3,565 matching player/world/native states**. Both finish alive at 88/78
health. The parking controller now takes bounded, observed sideways detours
around corners; the previously failed corner at 52 passes through ordinary
movement. A normal checkpoint before 60 is retained. Source `6f589de` passes
**889 active Rust tests**, the locked release, **72 native checks** and the exact
single-player **11+1** baseline; **85 Python guards** pass. This is resumed
section evidence, not fresh traversal or a completed campaign visit. Clockwork's
later machinery/scenes, recovery, reconnects and complete repeats remain open.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **43-51** section verifies 18
personal arrivals, 16 independent pathnode projections, nine waiting handoffs
and **2,634 matching player/world/native states**. Both finish alive at 88/100
health. The requested 43-59 run stops at 52 because its parking selector cannot
take a short sideways detour around a corner. The original failure is retained;
the completed section has a separate scoped audit. Source `6f589de` passes
**889 active Rust tests**, the locked release, **72 native checks** and the exact
single-player **11+1** baseline. The parking fix passes **85 Python guards**.
No additional complete campaign visit is credited.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **36-42** gap sequence passes
through independent internet clients: six personal arrivals, two actual gap
jumps, two preparation holds, one waiting handoff and **232 matching
player/world/native states**. Both finish alive at 100/99.867 health. A normal
checkpoint beyond the gap is retained for the next section. Remote plans now
require supported first landings and retain subsequent jumps; direct airborne
steering preserves enough speed to cross the gap before settling. Source
`a07517e` passes **887 active Rust tests**, the locked release, **71 native
paired/driver checks** and the exact single-player **11+1** baseline. This is
resumed segment evidence; full Clockwork and campaign repeats remain open.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **25-35** section verifies 16
personal arrivals, seven corridor handoffs, two timed holds, one lever and
**1,073 matching player/world/native states**, ending with both alive at 89/100
health. This is a scoped audit of a longer run that stopped at the gap approach.
The next retry exposed a delayed landing failure before that gap. An earlier
braking handoff now passes **886 active Rust tests**, including a regression
that reproduces the old fall. The new live retry verifies **33-34**, including
the previously failed landing, with four personal arrivals, two handoffs and
**237 matching states**. It then fails at 35 with Red falling during a different
jump approach; the gap sequence remains untested. Full Clockwork stays open.
The locked release, 68 paired native checks and exact single-player 11+1
baseline pass, as do **72 Python guards**. Total completion stays **48.6%**,
with six fully verified campaign visits.

Earlier automation result: Clockwork's resumed **18-24** section verifies 12
personal arrivals, all three stair handoffs, one timed hold and **454 matching
player/world/native states**, with both players at 89 health. The leader moves
to the next tread before the follower arrives. The runner also waits for command
acknowledgments retained in its history; **59 Python guards** pass. The requested
18-38 run stops at step 25 while selecting a corridor waiting position. Its
completed section is audited separately, and a normal checkpoint before 25 is
retained. Full Clockwork remains open; total completion stays **48.6%**.

Earlier automation result: Clockwork's resumed gear exit, **step 17**, now verifies
both actual jumps and personal landings, with **245 matching player/world/native
states** and both players alive at 89 health. Route planning now holds a stable
position against gear carry using ordinary controls. The exit test jumps from
the verified gear waypoint because a static walking route can lose support as
the gear turns. Source `cf5d551` passes **885 active Rust tests**, the locked
release, **68 native paired checks**, **50 Python guards**, and the unchanged
single-player **11+1** baseline. The resumed exit is separate from fresh traversal
and does not complete another campaign visit. Overall remains **48.6%**.

Earlier automation result: Clockwork's resumed **14-16** segment now verifies both
players boarding the rotating gear, with two actual jumps, four personal course
arrivals, one timed hold and **447 matching player/world/native states**. The
first player holds its waiting point within 2.76 units while the partner boards;
both remain at 89 health. The test driver compensates for observed gear motion
using ordinary controls. Source `623fece` passes **883 active Rust tests**, the
locked release, **68 native paired checks**, **43 Python guards**, and the exact
single-player **11+1** baseline. The native gear checks include both rider orders
and delayed observations/inputs. The original requested 14-17 run still fails
at the exit with repeated planning; it remains marked failed. This resumed
segment does not extend the fresh traversal or complete another campaign visit.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's fresh **0-15** prefix verifies 24 personal
course arrivals, eight intermediate arrivals, four jumps, one pull-up, two timed
holds, two lever activations and one ordinary partner yield. All **2,997** compared
player/world/native states match, with both players alive at 89 health. The
runner now detects stalled contact even when attempted velocity remains high,
stops the follower, walks the waiting player to an observed clear position and
resumes the follower. **38 Python guards** pass. Native source stays `dbd12c8`.
The requested 0-17 run still fails at step 16 while repeatedly replanning on the
rotating gear; its scoped audit excludes that failed segment. No new complete
campaign visit is credited. Overall completion remains **48.6%**.

Earlier automation result: Clockwork's fresh entrance (steps 0-1) passes with
either player going first on source `dbd12c8`. The two internet runs verify
697 and 693 matching states, each with four course arrivals, eight intermediate
arrivals, four actual jumps and one native pull-up. Both players finish at 89
health. A delayed final-steering correction and precise waiting-footing arrival
address the observed overshoot and blocked pull-up. All 881 active Rust tests,
the locked release, 27 Python guards and seven source-package tests pass.
A longer fresh run independently verifies steps 0-5 and 981 matching states,
then stops at step 6 with both alive: the parked first player blocks the
follower's planned approach. That coordination failure is retained. No additional
campaign visit is credited, and the overall estimate remains **48.6%**.

Earlier automation milestone: Clockwork now has a fresh two-client internet
prefix runner with separate personal arrivals, supported waiting moves, normal
lever inputs and checkpoints, and network-state comparisons. Read-only door
observations let its normal E input work with Clockwork-owned door groups.
Waiting positions prefer the far side of a doorway; parking on the incoming
side had pinned the follower against an open door's tip. Source `f6b1a4f`
passes **60 native machinery/door records**, all **40 scene cases**, and the
unchanged single-player **11+1** baseline. Four new local door cases cover
both actors and the proximity-open starting state, with ordinary partner yield.
The route runner now has 27 receipt/parking/jump/pull-up guards. Source `4fd024c`
adds grounded navigation to the optional test driver and passes 880 active Rust
tests and the locked release build. The component results above remain attributed
to `f6b1a4f`; the new build has not passed the live prefix.
This is additional automation and component coverage, not another completed
campaign visit. Total credit remains **48.6%**. See [scope and retained failures](HATTER1.md).
The earlier fresh retry died at step 1. Observed entrance takeoffs and landings,
ordinary ledge pull-up input, and precise centering now pass a fresh **0-5**
prefix: ten course arrivals, eight intermediate personal arrivals, four actual
jumps, one native pull-up, one timed hold and 872 matching player/world/native
states. Both finish that prefix alive at 89 health. The run then stops at step 6,
where the original single-player marker cannot satisfy the stricter co-op
arrival radius. A repeat passes 0-5 again, then Blue reaches the adjusted step-6
target before Red falls on the approach. The latest run on `4fd024c` connects
over the internet but stops at step 1: Red walks beyond the lower entrance
footing while the test driver is settling the next takeoff position. Its second
jump has not occurred. This is a failed route test, not a hung process or a
completed visit. Failures remain retained; full Clockwork remains open.

Earlier fix: Clockwork's gear barrier and laboratory door now wait for all
living players to pass their authored closing approach. The laboratory lever
alcove does not count as inside. Source `a007099` passes **18 local door cases**
with either first player, paused/compressed waiting-state restore, either/both
players dead and the outside-lever case. All 38 earlier machinery records,
40 scene cases and the unchanged single-player 11+1 baseline also pass.
The gear-door course begins at its open doorway; traversal/waiting on the
rotating great gear remains unverified. Total credit stays **48.6%** and the
completed campaign count is unchanged.

Earlier verification milestone: Clockwork's new native scene suite passes all
**40 cases**: four cinematics with either owner, watched/held-skipped play and
living/dead partners, plus both ordinary speech scenes with either initiator
and living/dead partners. It checks actual contact/lever activation, frozen
waiting positions, pause/compressed restore, camera and authored pose stability,
owner-only skipping, safe separated reunion and personal resource preservation.
Ordinary speech advances once while both living players move independently.
Source `ec48aa7` also passes all 38 machinery checks; the single-player 11+1
records still match the baseline exactly. This adds **0.25 scene/progression
points** for complete staged handoff coverage, not a full campaign visit.

Earlier implementation milestone: Clockwork (`hatter1`) now explicitly advances
one shared machinery clock, carries both living riders, waits for boarding,
and binds lever actions and their delayed scenes to the actual user. Source
`9ce4f98` passes **38 native paired component checks** covering lift and floater
carry, blocked-rider rollback, saved boarding contacts with either requester,
separate sinking pillars, all five levers with either actor, blocked approaches
and owner-specific death cleanup. The original single-player 11+1 component
records match the pre-change baseline exactly. All 879 active Rust tests, the
locked release, source-package seven and source-worktree 890 checks pass.

That adapter added **0.25 movement points**, using the same bounded credit as
earlier native paired-transport milestones. Full paired handoffs are now checked
separately above. Clockwork still needs rotating-gear/exit cases, continuous
two-client traversal, recovery and reconnects. Internet and completed-campaign
credit are unchanged. See [Clockwork coverage](HATTER1.md).

Funhouse's best verified fresh prefix remains **0-88** on source `f9e7029`:
122 personal arrivals, 27 own pickups, 14 paired holds, eight switch shots,
13,767 matching states and four exact ten-second revivals. Both finish at 100
health. The gas-scene fixture correction separately passes unchanged checkpoint
**89-94**, including the watched scene, safe shared reunion, six arrivals, two
holds and 578 matching states. That segment does not extend the fresh-run proof.

The latest fresh repeat with that correction fails at **36 after a team wipe**.
Its independently verified **0-34** prefix has 52 arrivals, six own pickups,
two holds, one switch and 4,788 matching states. The host trace shows both players
restored together at the normal checkpoint with 97 health; full two-client wipe
verification remains open. The failure is retained. No combat rule, resource,
position or team-wipe guard was weakened. Full Funhouse traversal remains open.

Two resumed mirror 29-32 runs also pass, including exact actual-jump evidence.
Conservative range rejection removes distant-enemy aiming work from the test
driver; its measured host p95 cost drops from 39.5 to 1.1 ms per frame on the same
PC. The earlier 0-37 prefix had no control resets; the latest run records one
early suspension. Gameplay physics, combat rules and
the control-reset safeguard are unchanged. **877 active Rust tests**, the locked
release, Funhouse battle63/world46/scene37/single-player8 and Checkmate94 pass;
single-player component output matches baseline except two benchmark timings.

Separate verified work: Funhouse's second personal pad and descent now pass
steps **151-155** with eight personal arrivals, both actual jumps and 396 matching
network states. Each player gains their own second 25-health/Will pickup.
A separate resumed combat run defeats both Tweedles, watches all eight Hatter
scene lines, preserves the hidden/frozen partner and the original floor-collapse
tail, then reaches **Hatter1 with both players alive**. It has 1,178 matching
Funhouse states plus four retained matching states in the new visit. These are
checkpoint segments; the fresh full route and remaining campaign checks stay open.

Recent verified milestones, which do not yet complete another campaign visit:

- Checkmate has separate fresh Blue-led watched-scene and Red-led skipped-scene
  internet passes, a natural team wipe and a guest reconnect pass. Other recovery
  cases remain open. Six native GPU comparisons verify both skins' weapon effects;
  real clients also receive both players' weapon effect types.
- Funhouse's return corridor, steps 35-42, passes 16 personal arrivals and 1,175
  matching player/world/native states, including Red's exact ten-second revival
  at 25 health and each player's own essence pickups.
- Funhouse machinery, steps 46-88, passes 50 personal arrivals, 11 paired holds,
  seven switch shots and 3,633 matching states. Steps 89-94 have separate scoped
  arrival, hold and pickup evidence.

Latest milestone: Funhouse steps **95-103** now have 12 verified personal arrivals,
five personal essence pickups and 1,139 matching player/world/native network states.
Both finish at 100 health, without deaths. The test uses ordinary counterbalancing,
actual native brush-support checks and visibility-aware resident enemy checks.

Latest upper-exit milestone: both players now pass steps **103-109** on source
`88b98f0`, with 12 personal arrivals, two 200-tick holds, two own pickups and
719 matching player/world/native states. A separate resumed **107-117** run
passes 22 personal arrivals, six own pickups and 1,524 matching states, without
new deaths. These are separate checkpoint segments, not a fresh full route.

Source `94488e1`, verifies the narrow descent **111-116**, including
12 personal arrivals, one actual forward waiting move, three own pickups and
1,010 matching states. Red dies naturally and revives after exactly 1,200 active
ticks at 25 health, with safe placement, preserved resources and verified camera
handoff/return. The co-op recovery logger now uses the actual slope-aware placement
rules; its old flat-ground test falsely rejected supported sloped revivals.

The waypoint-117 stall is resolved in the test controller: it now finishes a
nearby clear, supported approach within the existing walking height and arrival
bounds. A separate **116-121** prefix has six personal arrivals, three safe
waiting handoffs, one own pickup and 422 matching states, ending at 100/65 health.

Source `c8f333c` then verifies **121-125**: ten personal arrivals, five safe waiting
handoffs, two own pickups and **1,033 matching states**, ending at 90/30 health
with no deaths. The dodge controller now predicts network delay and braking;
the earlier version could walk the guest off the ledge before taking damage.

A further **121-125** replay passes with **1,079 matching states**, ten personal
arrivals, five safe waiting handoffs and two own pickups. Both finish at 70 health
without deaths. Normal checkpoints before 125 and 126 are retained, allowing
the next crossing test to resume without repeating the earlier fights.

The narrow bank and pendulums now pass as a coordinated **126-134** sequence on
source `c79ac85`: **14 personal arrivals**, all six individual pendulum landings,
both bank jump holds and **1,041 matching states**. Red crosses first and waits
off the landing route before Blue follows. Both finish at 70 health without
deaths; a normal checkpoint before 135 is retained. The final jump waits for a
reachable departure window, including platform movement during network delay.

The next **135-138** segment now passes on the same native source: six personal
corridor arrivals, three supported waiting handoffs, Red's watched Tweedles scene
and safe shared reunion, with **1,092 matching states**. Both observe all nine
scene lines; Blue remains hidden and frozen, then both return at 70 health with
their personal resources intact. A normal checkpoint before 139 is retained.
The entrance step is verified by its scene trigger and reunion: the scene closes
the door before either player can reach the old marker inside it.

A later resumed attempt independently verifies the **139-143 prefix**: ten
personal arrivals, five supported waiting handoffs and **505 matching states**,
ending at 60/70 health without deaths. It then wipes at 144: Blue dies while
waiting below the climb, and Red dies before Blue's revival. A normal checkpoint
before 144 is retained; the complete 139-146 run failed.

The climb now passes as a coordinated **144-146** sequence: six personal
arrivals, two observed following footings, one safe final waiting handoff and
**203 matching states**. Both move and jump normally, with the follower keeping
space while the leader brakes. Neither dies; final health is about 6.8/59.5.

A subsequent **147-150** replay passes both actual pad jumps, four personal
bridge arrivals, the paired 900-tick hold and **479 matching states**. Jump
durations are measured after the game acknowledges each actual jump, excluding
command-delivery delay. Each player collects their own arena essence: each gets
25 health and Will, with no shared consumption or partner healing. Both survive
at about 31.8/38.1 health. A normal checkpoint before 151 is retained.

Next are the opposite pad jump and descent from **151**, the remaining boss
fight, later scenes and exit. Fresh full traversal and the opposite crossing
order remain unverified. These segments add no completed campaign row.
Overall stays **48.1%**.

Tests run as two independent clients through the public network path on Anode,
with injected delay and jitter. Routine diagnosis uses state and log assertions.
Screenshots are not needed for each run. The current source also has 875 passing
active Rust tests and native Funhouse battle, machinery and scene checks; those
component results do not prove a full campaign route.

See the [fixed weighting and detailed history](COOP.md#delivery-and-verification-weighting),
[campaign matrix](COOP_CAMPAIGN.md) and [Funhouse evidence](FUNHOUSE.md). Future
updates should state the overall percentage, the new bounded milestone, and the
next unresolved acceptance check even when the percentage does not change.
