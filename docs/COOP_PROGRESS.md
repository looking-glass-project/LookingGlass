# Co-op progress

Updated 4 November 2026. Branch: `coop`. **Estimated overall completion: 48.6%.**
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
| Two players, movement and presentation | 10.5 | 15 |
| Combat, individual resources and death | 9.25 | 20 |
| Scenes, transport and progression | 10.75 | 20 |
| Full campaign verification | 3.08 | 20 |
| **Total** | **48.58, rounded to 48.6** | **100** |

Latest automation result: Clockwork's fresh **0-15** prefix verifies 24 personal
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
