# Two-player co-op design

Development branch: `coop`, based on public main `0220970`.
For the concise current status, see [Co-op progress](COOP_PROGRESS.md).

The agreed playable co-op milestone is complete; broader polishing continues.
See [setup and save instructions](COOP_QUICKSTART.md).

Current milestone: **100% playable readiness; all 39/39 normal paired routes and
the public two-client movie/credits handoff are accepted.** The historical full-matrix
score is 81.5%, including deferred polishing. The historical exhaustive matrix retains six combined visits and
33 partial visits because it includes deferred polishing cases. The [Queen finale](COOP_QUEEN.md)
closes the last missing explicit paired adapter. The [playable milestone](COOP_PLAYABLE.md)
takes priority over exhaustive polishing. See [current progress](COOP_PROGRESS.md)
and the [castle integration batch](COOP_CASTLE.md). Earlier
percentages below describe historical checkpoints.

Burning Curiosity now passes **26 staged native co-op cases plus one summary**
on source `56160ff`: personal arrival and essence, both living riders boarding,
shared descent and independent uneven-floor handoffs, obstructed-return rollback,
and original arrival/Caterpillar scenes with either Caterpillar owner. Watched,
skipped, living/dead partner and compressed-save cases pass. Single-player
components and the full native combat route match the earlier baseline exactly;
the separate cast-free step-54 traversal failure is unchanged. This adds 0.75
points under the existing component weighting, bringing the total to **50.08,
reported as 50.1%**. Full two-client traversal, recovery and reconnects remain
open; the combined-visit count stays six. Its fresh internet opening now passes
steps 0-3: eight personal arrivals, watched entry, both early lava crossings and
2,133 matching states, without deaths. This segment adds no percentage credit.
Separately audited steps 4-20 and a resumed 21-23 run extend the approach evidence.
Source `ca4cc26` corrects sloped-footing checks in the automated planner; the
previously stalled climb passes with 2,224 matching states and both players at
100 health. A normal checkpoint before 24 is retained. All 907 active Rust tests
and the native/single-player baseline comparisons pass. Later bounded 24-25 and
30-31 evidence accompanies a passing resumed 26-29 run (1,374 matching states,
50/96 health, no revival). The runner now allows ordinary combat dodging and
uses homing Cards before parking. A later retry verifies 30-32 by a different
ascent, without establishing a fix for the earlier missed climb. Bounded 33-37
and 38 evidence adds 1,309 and 462 matching states respectively. The runner
uses observed dry waiting bays and ordinary jumps into and out of the original
shallow-lava waypoint. Both survive step 38 at 93.75/41.67 health; that longer
attempt fails at 39 while navigating through lava. Source `259d4f5` adds optional
liquid avoidance and fixes blocked-walk detection based on actual displacement.
A resumed 39-42 internet run now passes eight arrivals and 1,335 matching states,
ending at 100/56.67 health without deaths or revivals. All 910 active Rust tests
and unchanged native/single-player baseline comparisons pass. A normal checkpoint
before 43 is retained. Source `5c4a693` then separates successive timed jumps
with actual settled landings. Ordered forward approaches find waiting bays
beyond narrow slopes. Bounded 43-45 evidence adds six arrivals and 869 matching
states at 100/71.67 health without revival. The longer run fails at 46 after
planning during an active ledge pull; the normal checkpoint before 46 is retained.
Source `c7576ca` waits for that pull to finish. A separate 46-49 internet run then
passes eight arrivals and 1,017 matching states at 100/46.67 health without revival.
The normal checkpoint before 50 is retained. All 913 active Rust tests and the
unchanged native/single-player baseline comparisons pass. A separate 50-52 run
also passes six arrivals and 699 matching states with unchanged health, retaining
the normal checkpoint before the platform at 53. Separate evidence covers
approach points 0-52; a fresh uninterrupted repeat and the full route remain open.
See [Burning Curiosity](JLAIR1.md).

About Face now has a native paired adapter for combat, personal rewards,
shared machinery and original scenes. Source `3298768` passes **83 staged cases
plus three summaries**, 905 active Rust tests and a locked release. Existing
single-player component/route output matches the baseline exactly. That milestone added
0.25 points each for combat/resources, movement and progression: **49.33 total,
reported as 49.3%**. A fresh full two-client route now passes with 6,805 matching
states, personal rewards, shared lift, Red-owned watched ending, exact Blue
revival and actual paired onward arrival. A separate Blue-owned ending also
passes. Red's separate exact 1,200-tick revival adds 1,087 matching states. A
fresh guest process also rejoins the unchanged victory checkpoint with personal
resources, pickup ownership and shared progress preserved: 77 matching states,
58 after rejoining. Renderer counters show particles/effects in both clients
with no dropped batches. A fresh ordinary combat team wipe now also passes on
`116fc99`: both players restore together at tick 7,060 to the normal checkpoint
at 3,421. An opt-in native receipt verifies the exact saved bodies, resources,
characters, enemies, pickups and shared progress before the next tick. Both
clients agree on 1,058 states, including 27 after restoration. Combat/transport/
transition reconnects and remaining ending cases keep this campaign row partial;
that recovery milestone retained 49.3%. See [About Face evidence](HATTER2.md).

Latest Clockwork evidence covers resumed route sections 144-145 and 146-149:
both portal arrivals, the Red-owned watched final scene, safe reunion and the
shared exit to About Face. These are checkpoint segments; full fresh traversal,
the other final-scene owner, recovery and reconnects remain open. See
[Clockwork coverage](HATTER1.md).

Earlier Clockwork machinery milestone: native source
`9ce4f98` passes 38 staged checks for shared clocks, two-rider lifts/floater,
obstruction rollback, actual boarding contacts, both lever users, personal
blocked approaches and owner-specific recovery. Single-player component output
matches the unchanged baseline exactly; 879 active Rust tests and the locked
release pass. This earns 0.25 movement points; full scenes and paired traversal
remain open. See [Clockwork coverage](HATTER1.md).

Funhouse's best fresh evidence remains 0-88 on `f9e7029`. Its corrected gas-scene
gate separately passes checkpoint 89-94 with both clients. A subsequent fresh
repeat fails at 36 after a real team wipe; its 0-34 prefix independently passes.
Host-side checkpoint restoration is observed, but two-client wipe verification
and the complete route remain open. No completed campaign row is added.

Earlier Funhouse traversal milestone: source `c946266` now passes the resumed
return corridor, authored steps **35-42**, in independent public-internet clients
with 40 ms delay and 20 ms jitter. All **16 personal arrivals** and **1,175
matching player/world/native states** pass. Red's natural revival takes exactly
1,200 active ticks and restores 25 health, with preserved resources, safe
placement and verified spectator/return cameras. Both players walk over their
own essence from defeated Automatons. The runner had previously walked past
these drops and entered the next fight nearly dead. Final health is Blue 100 /
Red 26, and a normal checkpoint before step 43 is retained.

The fresh watched entrance through steps 0-28 remains separately verified on
`b897af2` (44 personal arrivals and 2,734 matching states), and mirror steps
29-32 on `b703a7a` (both jumps/holds, one actual shot and 615 matching states).
These are separate segment proofs; no fresh full-route pass on `c946266` is
claimed. The first machinery attempt verifies the ten personal arrivals and
three paired timed holds in steps 43-50, then fails to hit cell4 at step 51.
The earlier failed combat attempts remain recorded. A subsequent resumed
**46-88** machinery run now passes: **50 personal arrivals**, **11 paired timed
holds**, **seven actual cell-switch shots** and **3,633 matching states**. There
are no deaths and final health is 100/90. Cell3 required two ordinary walks to
observed safe shooting footings; the other six shots completed at their first
positions in this run. A normal checkpoint before step 89 is retained. The
next attempt has scoped arrival/hold/pickup proof through 94, then stops at 95.
A later queued run on `0d43ade` now verifies **95-103**: 12 personal arrivals,
five personal essence pickups and 1,139 matching player/world/native states,
with no deaths and final health 100/100. Step 98 is the third seesaw's pivot,
not a stationary bank. Ordinary counterbalancing keeps the approach accessible.
Native support diagnostics verify actual standing on the tilted board during
101 and 103; falling onto a lower floor cannot count as arrival. The clear check
now includes visible resident Boojums and their remaining attacks. The run fails
at 104 when both riders load the rear and its next target loses support. A normal
checkpoint before 103 is retained for a coordinated upper exit. Source `88b98f0`
now verifies both riders through **103-109**: 12 personal arrivals, two 200-tick
holds, two own essence pickups and 719 matching player/world/native states.
An opt-in continuous test controller sends ordinary movement and jump inputs;
each brief approach/landing receipt must match actual sent and restored frames
on both clients. No game physics or arrival bounds were relaxed.

A separate resumed **107-117** run on that source passes 22 personal arrivals,
six own pickups and 1,524 matching states, without new deaths. The subsequent
`94488e1` replay verifies **111-116** with 12 arrivals, one actual forward waiting
move, three own pickups and 1,010 matching states. Red's natural death also gives
an exact ten-second, 25-health revival with preserved resources, safe placement
and spectator/return camera proof. The recovery logger uses co-op's actual
slope-aware support rules, with floating/airborne/edge rejection checks.
That replay's waypoint-117 stall is resolved by retaining the existing walking
arrival bounds while finishing a nearby, clear, supported approach. Source
`8a6352f` has scoped **116-121** proof: six arrivals, three safe waiting handoffs,
one own pickup and 422 matching states, with no deaths and final health 100/65.

Source `c8f333c` verifies **121-125** with ten personal arrivals, five safe waiting
handoffs, two own pickups and 1,033 matching player/world/native states. Both
survive at 90/30 health. The ordinary-input dodge controller accounts for network
delay, pending motion and braking, with shorter sidesteps on narrow supports.
It previously walked the guest off a ledge before taking damage. That run stopped
at 126 where the first player's waiting point blocks the only supported approach
lane. This exposed the need to coordinate the bank and pendulums.

A further resumed **121-125** run passes with ten personal arrivals, five safe
waiting handoffs, two own pickups and **1,079 matching states**. Both finish at
70 health without deaths; normal checkpoints before 125 and 126 are retained
for the next coordinated crossing test. This is another bounded segment pass.

Source `c79ac85` now passes the queued **126-134** bank/pendulum sequence: **14
personal arrivals**, all six individual moving-platform landings, both bank jump
holds and **1,041 matching player/world/native states**. Red crosses first, walks
beyond the narrow far landing and waits on a supported footing while Blue follows.
Both finish at 70 health without deaths. The final jump predicts platform carry
during network delay and waits for ordinary jump range. A normal checkpoint before
135 is retained. The next **135-138** segment passes six personal corridor
arrivals, three supported waiting handoffs and Red's watched Tweedles scene with
safe shared reunion. Both peers observe all nine lines and **1,092 matching
states**, retaining 70 health and personal resources. The entrance is verified
by its actual scene trigger and reunion, rather than an impossible arrival
inside the door after it closes. A normal checkpoint before 139 is retained.
A further resumed attempt independently verifies **139-143** with ten personal
arrivals, five waiting handoffs and **505 matching states**, ending at 60/70 health
without deaths. It then wipes at 144 while the players take turns on the climb;
the full 139-146 attempt remains failed. A normal checkpoint before 144 is retained.
The coordinated **144-146** climb now passes six personal arrivals, ordinary
jumps, observed follower spacing and **203 matching states**, without deaths.
The subsequent **147-150** replay passes both personal pad jumps, four arrivals,
the paired 900-tick hold and **479 matching states**. Jump timing starts after
acknowledgment of the actual jump. Each player receives their own 25-health/Will
arena essence without healing or consuming the partner's copy. Both survive at
31.8/38.1 health, with a normal checkpoint before 151.
The subsequent **151-155** segment passes both second-pad jumps, eight personal
arrivals and **396 matching states**. Each player gains their own second 25-health/
Will pickup; both survive the descent at about 56.8/63.1 health.
A separate resumed boss run now defeats both Tweedles, watches all eight lines of
Blue's Hatter scene with Red hidden/frozen, preserves its original floor-collapse
tail and reaches **Hatter1 with both players alive**. It verifies **1,178 matching
Funhouse states**, plus four retained matching states in the new visit. Fresh full
traversal, opposite lead/scene ownership and remaining recovery checks stay open.

The latest frozen build, `1f789e9`, passes **876 active Rust tests** and native Funhouse
**battle63/world46/scene37**, plus **Checkmate94**. Single-player component
gameplay matches baseline except two benchmark durations. **51 Funhouse guards**,
**13 recovery guards** and **seven source-package tests** pass. These are separate
resumed segments, not a fresh full route. Overall remains **48.1%**, with six
combined campaign rows. A new fresh run verifies steps 0-28 with 44 personal
arrivals, seven pickups and 2,763 matching states, then fails at the mirror.
The shooting wait and delay-aware hold controller are corrected; the guest's
actual return jump remains the failing case. `test_coop_funhouse_full.py` joins
the existing sections with explicit fresh-start, personal traversal and original
exit checks, but has not passed a full run. See [Funhouse evidence](FUNHOUSE.md).

Latest full-route milestone: a fresh Blue-led Checkmate run now passes through
the King and into Funhouse over the public internet, including Red's verified
ten-second revival and all three watched original scenes. There are 5,765
matching player/world/native states. Sustained paired hits previously lifted the
King out of reach; co-op recoil now keeps him on supported ground. All 857 active
Rust units and the native Checkmate/shared-world/campaign checks pass, and both
single-player Checkmate logs match baseline. The separate natural team wipe now
passes with exact saved players and 229 matching states. A fresh guest reconnect
also continues through the King into Funhouse with 3,410 matching post-rejoin
states and a verified revival. Both clients receive both players' weapon effect
types. A fresh Red-led skipped-scenes run now also passes on source `afe78ef`,
with 5,777 matching states, all three Red-owned scenes and an exact ten-second
revival. Six native GPU comparisons confirm visible weapon effects for both
skins after repeated restores. Other recovery/reconnect cases and native King
and environment effect visibility remain open. The campaign row is still partial
and the overall estimate stays 48.1%. See [Checkmate evidence](RCHESS1.md).

Latest automated recovery milestone: Checkmate's northern checkpoint segment
(steps 33-44) passes in two independent internet clients with 40 ms delay and
20 ms jitter. Red dies naturally, spectates Blue for exactly 1,200 ticks, then
revives safely nearby at 25 health with her carried resources preserved. Camera
return and player/world/native network state agree. Blue walked every waypoint;
Red's six remaining waypoints are explicitly carried by revival beside her.
This resumed segment adds no completed visit or percentage credit. The strict
zero-death mode remains available. See [recovery test evidence](COOP_RECOVERY_TESTS.md).

Latest Funhouse milestone: source `e1b17cb` passes 37 scene-check records
covering 32 original cinematic combinations and four personal Jacks pickup /
ordinary-speech cases. Both actors, watched/skipped playback, living/dead partners,
paused/compressed restored cameras, frozen partners and safe reunion are covered.
The opening scene now places the partner on the nearby corridor when the narrow
Alice stand has no room; the original stand still withdraws. The real Jacks
pickup starts shared speech while both players can explore, and each keeps an
individual copy without replaying the dialogue. Hatter skipping retains the
physical floor-collapse tail and original departure request.
Its actor now stays bound until that request is delivered, including when the
partner is dead and no reunion is needed.

The preceding 63 battle and 35 machinery records still pass, and the single-player
component gameplay log remains identical apart from two measured benchmark times.
All 855 active units pass with the final production fixes and scene diagnostics.
The 94 Checkmate, 82 shared-world and 43 campaign checks also pass, with
unchanged Checkmate single-player component and full-route logs.
No screenshots were opened. See [Funhouse evidence](FUNHOUSE.md).

This adds 0.25 shared-progression points, taking the estimate from 47.8% to 48.1%.
Six campaign visits retain combined evidence; 33 remain open. Funhouse's full
two-client traversal, recovery and rendered effects remain unverified. Its earlier
single-player route still fails at step 142; this update does not establish a
new full-route pass. Checkmate now verifies recovery from Red's northern death;
its fresh Blue-led full traversal and King battle now pass; the remaining
campaign-matrix cases are still open.
Repeated tests alone do not increase the estimate.

Latest Checkmate checkpoint: two independent local clients now pass the rope
climb, upper landing and onward waypoint (steps 7-10), at 92/100 health with
243 matching player/world/native states. This uses an unchanged normal save
from the earlier internet water approach and keeps body separation checks.
The follow-up public-internet rope replay also passes, with 240 matching
states and 92/100 health, on the build with bounded DNS retries. All 851 active
unit tests and the native checks pass. The two earlier invite-resolution failures
remain recorded; full Checkmate traversal and recovery remain open. See [Checkmate evidence](RCHESS1.md).
A further public replay now passes Checkmate steps 10-26 and the Blue-owned
beheading scene, with 3,444 matching states and both players finishing at 100
health. Red waits still and hidden during the scene; both reappear afterward.
Those checkpoint segments did not complete a campaign visit.

A subsequent fresh Checkmate run exposed a camera rule that hid a nearby partner
even with safely separated bodies. Close-camera hiding now applies only to the
watched player. The source-69 internet run passes free-play visibility, including
103 host and 23 guest near-partner samples, and the original scene reunion. It
reaches step 34 before Blue dies in an ambush; the run remains failed. The same
build passes 852 active units and native checks, with unchanged single-player
baselines. A blocked network consumer now gets the existing 15-second connection
grace with unchanged queue limits, and a three-second ordered-packet stall test
passes. Full traversal, recovery and broader effects coverage remain open.

The diagnostic route controller now supports running along straight grounded
sections with delayed-input steering. Its native regression suite passes 853
active tests, including narrow turns and faster corridor traversal. Checkmate's
latest internet replay lets Blue traverse all 12 northern waypoints to the
bridge, but Red dies after waypoint 38. The per-player route check keeps that
paired run failed. This adds no completed visit.

Castling's earlier Blue-led and Red-led public internet runs pass all 203 legs.
After the sloped-wall collision fix, Blue passes the full route again with 8,793
matching player/world/native states and a 75/100-health Checkmate arrival. The
first post-fix Red run passes 0�127 and both original scenes but **stalls at leg 128**;
it remains failed despite 9,378 matching states and passing presence samples.
Its host has about one second of queued controls while the guest reports about
206 ms round-trip time. Diagnosis traces the backlog onset to a 561 ms host
pause followed by delayed old input batches. An ordered control reset now passes
a real forced-pause internet replay through legs 124�129, with 145 matching
states and both players at 100 health. The subsequent fresh Red-led run on
`cc2388e` passes all 203 legs, both original royal scenes and the Checkmate
arrival at 100/100 health, with 6,519 matching player/world/native states.
The earlier failed run remains recorded; broader effects checks and other
campaign gates remain open. Checkmate now passes 84 native paired component
checks on frozen source `26b9134`: targeting and personal attacks, separate
healing, shared clocks and all three original scenes with either owner. Both
single-player logs remain identical to baseline. These earn bounded combat and
progression credit; full two-client traversal and recovery remain open.

The first independent Checkmate replay exposed a test-controller omission: its
attack logic did not include enemies owned by native campaign controllers. Blue
waited at the second waypoint beside an undamaged pawn and died. The failed run
is retained. Native target selection is now included in the test driver. Frozen
source `fe1505f` passes all 94 Checkmate native checks, including ten ordinary
attack/occlusion cases, all 847 active units, unchanged single-player logs and
the shared-world/entrance checks. A fresh full internet replay remains pending;
this test-controller repair adds no completion credit.

Real combat recovery passes for either player: both clients agree on the
1,200-tick spectator delay, nearby 25-health revival and retained personal items.
The unchanged team-wipe encounter also passes separation and shared restoration
after the sloped-wall repair. The verified `cc2388e` build includes the original held Enter/A
scene-skip control for the scene owner, with a fresh 0.65-second hold and the
existing safe reunion. All **843 active unit tests** and the broader native
checks pass. All four independent internet scene-skip cases also pass: both
royal scenes with either owner, unchanged personal resources and safe reunion. See
[Castling internet checks](WCHESS2.md#independent-internet-traversal-checks).
Focused combat, swimming, recovery and rejoin passes below are useful progress,
but do not satisfy that gate. The timed spike corridor and its following four
steps now pass focused internet replays with either leader, including delayed
approach/stop inputs and a check against retracing the completed corridor.
These add no full-level credit; see [timed corridor checks](WCHESS1.md#timed-corridor-test-controls).
A missed readiness frame in the lift test is also fixed: both leader roles now
pass the captured lift-to-corridor segment using persistent boarding/arrival
facts. See [lift handoff checks](WCHESS1.md#delayed-lift-handoff).
The fresh Red-led run on `d7bbc99` passes 0–67, then exposes a follower pursuing
an old airborne sample after landing. Supported path reattachment now passes a
native captured-position check and the Red-led internet replay through 67–70,
with 339 matching states. The required-jump section 100–108 also passes with
312 matching states. The next fresh run completes 0–92, then exposes an ownership
bug when the Knight trigger becomes eligible with both players already inside.
Red arrived earlier, but the inactive trigger forgot that order. Saved waiting
order now fixes this for shared contacts; both roles pass native captured-approach
checks and the Red-led internet replay passes 92–97 with 677 matching states.
The strict ownership assertion is unchanged. The next fresh Red-led run reaches
the next level through all 161 steps, but fails final presence checks: two guest
observations overlap during step 141. All 8,602 matching player/world/resident
states agree. This remains a failed route, with no full-level credit. A native
replay of the captured stair approach now isolates and fixes a rejected retreat
to the previous supported footing; both player slots pass 120 ordinary movement
ticks with separation, support and resources retained. Fresh full repeats remain
open. The repaired build's fresh Red-led route passes 0–80 without sampled overlap,
then stalls during the step-81 combat clear with both players alive. Its exact
normal checkpoint is retained for diagnosis. The Blue-led repeat passes 0–125,
then Blue dies in combat during the step-126 rush. Its 8,388 matching states agree
with no sampled overlaps; the exact pre-126 checkpoint and damage reports are
retained. Neither fresh route satisfies the full-level gate.
See [stair contact repair](WCHESS1.md#retreat-at-a-stair-support-boundary).
Castling now has 48 passing paired combat cases and 45 world/scene cases: personal
targets and saved attacks, shared doors and safe portal carry, both original royal
scenes with either owner, and saved first-arrival exit boarding. The moving exit
volume now follows its visible platform in both paired updates and after restore.
Its three single-player logs match baseline and all 834 active unit tests pass;
the broad shared-world, campaign entrance/recovery and Pale Realm component suites
also pass. The continuous native route now passes all 203 legs with either player
first and scenes watched or skipped, including both royal reunions and Checkmate
arrival with exact personal resources retained. All four runs pass, with 214
exact guest-world comparisons in total. Both separate internet leader orders now
also pass on `e54bc16`: 8,749 Blue-led and 7,680 Red-led matching player/world/native
states, both original scenes and Checkmate arrivals at 75/100 and 100/100 health.
The later `c1a202c` build also passes exact checkpoint resume, graceful fresh-guest
rejoin and the shared portal exit from both saved leader roles, with 55/49 matching
states and 35/29 after rejoin. All 837 active units pass. Death/wipe recovery and
broader effects remain
open, so this adds no completed campaign row or percentage credit; see
[Castling traversal](WCHESS2.md#continuous-paired-traversal).
See [descent recovery](WCHESS1.md#rejoining-a-recorded-descent).
The earlier Pale Realm combat stall is resolved by a supported sidestep after repeated
misses. A public two-client replay passes all 26 approach steps (100–125), then
verifies Blue's real combat death, moving spectator camera and exact ten-second
25-health revival with personal inventory retained. All 1,761 matching network
states agree. A separate Red-focused replay also passes, with the same exact
countdown, camera, inventory and 25-health checks and 281 matching states.
The same encounter's shared team wipe is now verified by exact native restore
and independent public clients, including restored resources, enemies,
progression and both cameras (118 matching states in the strengthened repeat).
The swimming-exit stall in the automated follower is also resolved: public
bank-to-exit replays pass with either leader, both players swimming and arriving
at the correct next-level entrance with their resources retained (617/441
matching states). These are resumed segments, not fresh full-level passes.
Weapon sparks and swing/projectile trails now survive the guest snapshot path.
Six native pixel comparisons cover both Alices, and a public two-client combat
check observes all three effect types and both emitting players, with 59 matching
states. This adds 0.25 movement/presentation points; it does not complete any
campaign row or certify every weapon effect. See [effect checks](COOP-EFFECTS.md).
The landing-contact case in a narrow Pale Realm corridor is also fixed: the
approaching Alice retains her landing height while backing out of contact, with
the waiting partner unmoved. Both player slots pass native regressions; the
captured replay improves from 15.26 to 36.00 units of separation. The automated
follower also retains jump takeoffs and can make room along the corridor when
its diagonal yield choices are blocked. See [focused checks](WCHESS1.md#jump-takeoff-and-narrow-corridor-contact).
A graceful guest leave/rejoin followed by the swimming exit also passes, with
624 matching states recorded after the new guest process joins. The original
puzzles, native resident health and personal resources remain intact.
The release build and all 834 active Rust tests pass. Full fresh
routes in both arrival orders, the complete recovery/rejoin matrix and broader
combat-effect checks remain open; this segment adds no full-level credit.
See [recovery details](WCHESS1.md#independent-client-revival-fixture).

Herbaceous Border now has an explicit paired platform/enemy adapter and **44
passing native component cases**: shared fulcrum carrying/counterbalance, both
dead-partner roles, resident targeting/personal damage, delayed spawns and ambush,
all original scene owners/reunions with watched/skipped outcomes, and Ladybug
impact effects surviving the guest snapshot path. The same build passes existing
single-player scene/machinery checks and 808 active Rust tests. These staged
checks take about six seconds without a window. Full traversal, independent
internet clients and the recovery/rejoin matrix remain open. Build
`0601aeb39bc98a7828a398e3c59f6fc5ef637ab54ab5fe343994a079cc45f8a2`;
evidence `private/coop-garden2-components-0601aeb.log` and
`private/coop-garden2-single-0601aeb.log`. See [Herbaceous details](GARDEN2.md).

The new ordinary-input Herbaceous paired route now produces reproducible failure
states without image inspection. Its first four attempts (both initial input
orders, watched/skipped scenes) fail near the first thermal climb when one
player dies; see `private/coop-garden2-route-da5e242.log`. The cause still needs
isolation. This adds diagnosis and repeatability, not a completed campaign visit
or percentage credit. The Herbaceous work alone leaves the estimate at **43.1%**.

Rolling Stones now advances its native marble, gates and collapsing scenery
once for the shared world, with contact damage assigned to the correct living
player. Eleven staged native checks pass on source `6df8431`, including complete
marble-path clock parity with single-player, personal lethal contacts, pause/save
and the original scene's paired reunion. Its three single-player check logs are
identical before and after this adapter: fast routes pass, while older walking-
pace fixtures still fail. This adds 0.25 points for verified shared progression,
bringing the estimate to **43.3%**. Full paired traversal and internet checks are
still open; no campaign row is completed. See [Rolling Stones details](GARDEN3.md).

The captured Herbaceous bank overlap is now fixed on source `c6a4ddc`: a floor
contact caused by rounding rejected a valid separation correction. The retry
retains complete wall, body and support checks. Both player-slot regressions
pass 240 movement/contact ticks each, and all 46 Herbaceous component cases
and 808 active unit tests pass. Replaying the identical diagnostic course clears
the former overlap, then fails later at the thermal (Blue dies at step 23 while
Red reaches step 28). That remaining failure keeps the visit partial and the
overall estimate at **43.3%**. See [captured bank evidence](GARDEN2.md#captured-bank-contact).

On that same build, the focused Village entrance regression passes through two
independent public-internet clients with 40 ms delay and 20 ms jitter per sender:
both approaching-player roles stop at 36 units without pushing the partner,
both Alice models remain marked visible, and each client produces ambient
particles and draws effect batches without drops. All 579 matching sent/restored
states agree; there is no death, respawn or wipe. Evidence:
`private/coop/presence-test-05c85a8bdfd1407ab723b3832e2497eb/`. No images were
viewed. This is an entrance contact/effect regression, not full-level validation.

Icy Reception now advances its seven native rocks, collapsing ice, wall slabs,
fog backstop and portal unlock clocks for the shared world. **44 staged native
co-op cases pass** on source `b7ba057`, including all rock/player/dead-partner
combinations, Easy altar sweeps, both scene owners, watched/skipped Caterpillar
outcomes, frozen partners, saved continuation, safe reunion and the bound portal.
All 808 active unit tests and the existing single-player component suite pass.
This adds 0.25 shared-progression points, bringing the estimate to **43.6%**.
Full paired traversal, independent internet clients and recovery/rejoin are still
open; no campaign row is completed. See [Icy Reception details](GARDEN4.md).

Fungiferous Flora now has shared Ant waves and departure timing, native paired
targeting/personal damage, spawn visibility for both living players and the
actual scene owner's Alice pose. **31 native co-op cases pass** on source
`e3d0555`, including both watched/skipped owners with either a living or dead
partner. A skipped-scene regression proves that waves and the ten-second timer
wait for reunion. The next visit also clears the former visit's temporary health
floor while preserving personal resources. All 809 active unit tests pass, and
the single-player component log is identical to baseline. This adds 0.25 combat
and 0.25 shared-progression points, bringing the estimate to **44.1%**. Full
paired traversal, independent internet clients and recovery/rejoin remain open;
see [Fungiferous Flora details](CENTIPEDE1.md).

Centipede's Sanctum now has a paired boss adapter with nearest visible living
player targeting, personal acid/melee hits and grab/throw ownership, individual
healing stations and slide protection, and shared defeat/spike/mushroom timing.
**28 staged native cases pass** on source `993d722`, including both scene owners,
watched/skipped intro reunion, saved grabs and progression through the original
mushroom exit. All 810 active unit tests pass. This adds 0.25 combat and 0.25 shared
progression points, bringing the estimate to **44.6%**. Independent internet
clients, broader rendered effects and rejoin remain open.
Ten focused production Arena checks now pass on source `ead0be3`: both grab
victims, funded weapon hits, personal acid damage, native acid-death/spectator/
1,200-tick revival and a paired checkpoint wipe. Thirty guest snapshots match
exactly; both actual particle passes emit 8,287 batches with zero drops. This
verifies the existing adapter credit and leaves the estimate at **44.6%**.
All **4/4 ordinary-input native paired routes now pass** on source `0edf86a`,
with either leader and watched/skipped intro, six funded boss hits, both alive,
correct exit ownership and the real next-level arrival. This run exposed and
fixed overlap between simultaneous ledge pull-ups: the approaching climber now
waits with a valid path/pose until their partner clears it. The follower takes
a later climbing fall but survives and travels onward through the first-arrival
scene/level handoff. All 812 active unit tests and the repeated 28 component/ten
Arena cases pass. Internet/rejoin testing is still required; no campaign row or
additional percentage credit is claimed for this local route milestone.
See [Sanctum details](CENTIPEDE2.md).

Sanctum now also passes one **full Blue-first internet route with guest rejoin**
on source `bb4d83b`, using two independent clients, 40 ms delay and 20 ms jitter
per sender. All six boss hits, normal checkpoint saving, fresh guest reconnect,
correct mushroom ownership and next-level arrival pass; both players survive
at 51/49.5 health, and 1,874 matching player/world/native states agree. All four
native routes repeat successfully on the same source. Red-first internet
traversal remains open because the delayed test controller stalls on the climb;
the failure reports and authentic checkpoint permit image-free replay. Full
separate-client recovery/effects coverage is still required. This verifies more
of the existing adapter credit: **44.6% overall, six combined campaign rows**.
Evidence: `private/coop/sanctum-route-a90c37b7f8d54d69ae30417854abf9d1/`.

Both Caterpillar's Plot visits now have explicit paired progression, shared gate
and destruction clocks, individual essence stations, and first-arrival Staff
ownership. Source `2990c66` passes **62 staged co-op cases**, including 40 original
scene combinations and eight isolated native gate carry/obstruction cases.
The scene checks exposed and fixed a real reunion deadlock on the Staff pillar:
that specific handoff now permits a supported, unobstructed partner landing
within 256 horizontal/downward units. Ordinary revival is unchanged. All 813
active unit tests pass, and both single-player scene suites and routes match
their baseline logs exactly. This adds 0.25 shared-progression points per visit,
bringing the estimate to **45.1%**. Both campaign rows remain partial; full paired
traversal, combat, optional reward coverage, internet, rendered effects and
recovery/rejoin remain open. See [Caterpillar's Plot details](WFOREST.md).

Pale Realm now has a paired resident-cast combat adapter. Source `15daa28` passes
**40 isolated native-data cases** for visible living target selection in both
slots, personal melee and bishop beams, saved attack ownership, authored NPC
duels, knight shields, delayed cast walks/spawns and death callbacks. All three
single-player component/watched-route/skipped-route logs match the `2990c66`
baseline exactly. This adds 0.25 combat points, bringing the estimate to **45.3%**.
These fixtures use native actors/timings with staged combat geometry; they do not
establish native-map navigation, rendered effects or full co-op support. The
chess puzzles, shared machinery and original scene handoffs are covered by the
subsequent milestone below. Full paired traversal and internet recovery remain
open. No campaign row is completed.
See [Pale Realm details](WCHESS1.md#co-op-cast-components).

Pale Realm's paired world adapter now passes **79 native gameplay cases** on
source `cbeb734`: shared clocks, two-rider lift boarding/carry, individual crusher
damage, all five original scenes with either actor, both complete chess puzzles,
lever ownership and interrupted-puzzle recovery. The waiting player watches the
original puzzle actor; a dead partner's countdown runs during interactive play,
with a ready revival held until the safe handoff. Four automated renderer cases
also pass for Blue/Red Bishop/Knight transformations and compressed guest state.
They caught and fixed old particle positions carrying into the next transformation.
This adds 0.25 movement and 0.25 shared-progression points, bringing the estimate
to **45.8%**. Full paired traversal, independent internet clients, combat effects,
team wipes and reconnects still need this visit's combined coverage. See
[world and puzzle evidence](WCHESS1.md#co-op-world-and-puzzles).

The continuous Pale Realm Arena route now captures reproducible failures using
ordinary inputs, funded combat and compressed guest snapshots. It exposed an
angled open-door contact where the players could overlap slightly. Source
`24a8a6f` fixes that contact and passes **81 native world cases**, including both
captured player roles. All three single-player check logs still match baseline.
The latest batch fixes contact while already overhanging a supported platform,
and advances the automated follower through narrow passages, swimming and bank
pull-ups. Source `7fa6ff4` passes **all four complete native paired routes** with
either leader and watched/skipped scenes: both puzzles, all five scenes, the full
water section and the actual Castling arrival. Both players survive every run;
433 compressed guest snapshots match. **87 native world cases and 820 active
Rust tests pass**, and all three single-player logs still match baseline.
The same production contact code also passes the separate-client public-internet
entrance test with 40/20 ms delay/jitter, both models visible, rendered ambient
particles without dropped batches and 601 matching sent/restored states.
Full Pale Realm internet traversal, combat effects and recovery/rejoining remain
open. No combined campaign row is completed by these checks; the estimate
reviewed on **27 October 2026** stays **45.8% (about 46%)**. See
[route and contact evidence](WCHESS1.md#continuous-co-op-route-and-captured-door-contact).

The independent-client Pale Realm fixture now uses the same authored course
over the public connection with delay/jitter and no image inspection. Source
`685a05d` completes **71 of 161 steps (0–70)**, including the Bishop puzzle and
bell sequence. It then times out at step 71 with Blue hanging at a ledge and
Red waiting nearby; both remain alive at 18/70 health. All **8,213** matching
network states agree on players, shared world and native residents. This is
partial traversal evidence; the full route has not passed. Full pre-step and
damage-state captures preserve the failure for focused diagnosis.

The exact same source passes 820 active unit tests, 87 Pale Realm native world
cases, all three single-player baseline log comparisons, all 39 campaign entrance
checks plus four underwater recovery cases, 82 shared-world checks, five Fortress
adapter checks and 15 Beyond adapter checks. Beyond's old Rage test now has the
first collector walk aside instead of expecting the second to pass through her.
No new campaign row or percentage credit is claimed. See
[Pale Realm internet evidence](WCHESS1.md#independent-client-route-fixture) and
[Beyond regression](BEYOND.md#co-op-contact-regression-27-october-2026).

The next automation changes clear the delayed follower's ramp fight with both
players at full health and let the leader plan an escape from a caught ledge.
The follower also moves aside to avoid blocking that escape. A normal saved
checkpoint now supports focused two-client replays: **steps 71–74 pass over the
public connection with 414 exact matching states** on source `4adff1c`. The
presence checks continue through ordinary play after puzzles and verify no
overlap, wrong camera, unexpected teleporting or moving puzzle waiter. A focused
segment is not full-level evidence; the overall estimate remains **45.8%**.
See [delayed follower and ledge recovery](WCHESS1.md#delayed-follower-and-ledge-recovery).

A fresh full-route repeat on that build now completes **97 of 161 steps**,
including both puzzles and the repaired ledge. It then stalls because the
leader waits in the partner's next pull-up destination. Both remain alive;
9,205 matching states agree. This is further partial evidence, not a completed
campaign visit. Source `766eb15` extends the landing-clearance rule to ledge
pull-ups. A resumed internet run then completes **steps 71–125**, with 3,442
matching states, before Blue dies in combat at step 126. The remaining route,
full repeat in both arrival orders and recovery/rejoin checks remain open.

The overall percentage is a weighted estimate, not elapsed time or a live build
indicator. An earlier Garden build passed 4/4 native paired routes. After adding
rope contact, the current build passes **2/4 native paired routes** (both watched
variants); the skipped variants fail in late combat and a later fall. There are
still **0 full independent internet routes passed**. The furthest internet run passed
the early crossings, Rabbit scene, early combat clear and later ropes; a
repeat passed that rope and reached Blue step 93 before Red fell at the next
rope approach (step 79). A subsequent captured segment now passes both crossings
with measured-delay steering; the complete route is still unverified. Earlier runs
still retain their grenade-wipe failures. A [short encounter replay](COOP_ENCOUNTER_TESTS.md) now
reproduces saved combat and checkpoint outcomes without a full-level rerun.
Earlier snapshots are captured for the grenade investigation. Report the last
passed milestone and the specific remaining failure
alongside the percentage; do not imply that unchanged credit means no work ran.
The captured pit recovery now passes through two public internet clients: Red
revives beside hanging Blue on the same active native rope, with separate grips,
a ten-second spectator countdown and 25 health. Custom level-controlled ropes
and the rest of the Garden recovery matrix remain open.

Latest contact milestone: native rope grips now separate on their pendulum
spheres instead of bypassing player contact. Swept contact also catches vertical
climbing into a partner. The approaching player yields; the correction keeps the
grip and checks the complete body against the world before moving it. Blocked
corrections retain the valid grip rather than pushing through geometry. Custom
level-controlled ropes remain outside this change.
The exact captured overlap fails before the fix and passes afterward through
240 movement ticks with valid grips and unchanged personal resources. Both-role
short-grip, vertical-climb and wall cases pass, along with all 807 active Rust
tests (one pre-existing test ignored). The new independent-client check also
rejects all eight overlap observations missed by the earlier fixture.

Build `319effeed590564c67f85e2f3ea37cb4a9f4eb5bc3ba429c8bea8054d3be7a99`
passes the saved two-rope segment through separate public internet clients with
40 ms delay and 20 ms jitter per sender. All 115 authoritative rope-spacing
samples pass (56 host, 59 guest), and 156 matching player/world/native-state
samples agree. Both finish alive at course steps 85/84, with no respawn or wipe;
health is 49.592/53. Evidence:
`private/coop/garden-route-4c5a26bbfa07436f9c751617fa9d0d3e/`.
No images were viewed. This is a segment regression, not a completed campaign row.

Full native regression on the same build passes both watched variants into
Garden2, with correct resources and rope spacing checked every free tick:
Blue first takes 31,099 ticks/52 replica checks and ends at 68.070/100 health;
Red first takes 28,947 ticks/49 checks and ends at 100/21.549 health.
The skipped variants remain failures: Blue first dies in late combat at tick
23,231 (steps 168/163); with Red first, Blue falls at tick 16,873 (steps 131/130).
Neither reports rope overlap before the death. These failures must be resolved;
the old 4/4 result cannot establish a full-route pass for this changed build.
Evidence: `private/garden1-coop-route-319effe-all/`,
`private/garden1-coop-route-319effe-red/` and their matching `.log` files.

Duchess/Just Desserts now has a paired combat adapter, 22 passing native
component cases and six passing production-Arena checks. Those cover both
actors' attacks/grabs, personal healing, original scenes and particle emission.
Full two-client traversal and recovery for that visit are still open.

Dry Landing/Garden now has 21 passing native component cases for personal arrival
resources, scene ownership/reunion, paired lily supports at three update rates,
obstructed/dead riders, independent clocks and saved bridge timing. Full paired
traversal, combat, recovery and internet transition/rejoin remain open.
The ordinary-input route now completes Garden and loads Garden2 with both players
alive in all four leader/scene variants. Personal resources carry over exactly
and arrivals are clear and separate. Jump shots resolve the low-cover combat
stall; separate landing goals and clearing the final platform before crossing
keep the automated players alive. These are single-process paired routes;
internet traversal and recovery remain open. The separate single-player
automated course still fails at its launcher.

The underwater temple now advances its guide and machinery for both swimmers.
Thirteen native component cases pass for individual air, crushing, saved state
and exit ownership. Four full paired swimming routes now pass in the production
Arena, covering both leaders and watched/skipped endings. Both single-player
swimming routes also pass. Full public-internet traversal now passes in both
leader orders, with correct exit ownership and matching player, machinery and
native-enemy state. Both drowning/respawn roles, shared checkpoint restoration
and entrance rejoin now pass too. Focused exit-rejoin segments pass with either
leader from an exact captured gameplay state. Focused native-enemy target switching
and separate collection of the same placed healing item also pass. Later full-route attempts exposed
intermittent trap deaths in the automated swimmer; the earlier passing routes do
not establish repeatable navigation. The fixture retains failures and timings
without screenshots and can reuse an exit checkpoint for faster focused checks.

## Agreed behavior

- Two separate players: blue Alice and the same model with red clothing. Recolor
  clothing at runtime from user-supplied textures; do not distribute edited game assets.
- Enemies choose the nearest living player they can see, respecting their sight
  rules, occlusion and invisibility. Resolve ties consistently on the host.
- The first player to reach a scene triggers it. Original cutscenes show one
  Alice. At the end both players reunite at safe positions near the authored end.
- Lifts and other shared transports that fit both wait until both living players
  are inside their boarding areas. One-person scripted rides need a separate policy
  and must never strand the other player.
- Health, Will, consumables and pickup collection are tracked per player. One
  player's consumable pickup does not remove the other player's entitlement.
- Death starts ten seconds of spectating the survivor. Respawn near that survivor
  with low health. Initial tuning is 25/100 Sanity. If no safe position exists,
  keep spectating and retry; never spawn inside geometry or above a lethal drop.
- If both die, restore an earlier shared checkpoint and place both players safely.
- The main menu, including ESC, offers Co-op with Host and Join. Host makes a link;
  Join accepts that link. Internet joining belongs in the first playable build.
- Two processes on one PC must use independent settings, saves, inputs and cameras,
  communicating through the same network implementation used between PCs.
- Every campaign visit must be playable, including leaves, swimming and transports.

Implementation defaults to validate in playtests: no friendly fire; save ownership
on the host; world progression shared; personal menus do not pause the partner;
respawn timers suspend during cinematic control. Story items and their world effects
must be separated from consumable inventories so quest progress cannot deadlock.

## Architecture

The host owns enemies, damage, pickups, puzzles, scenes, movement validation,
checkpoint restoration and level transitions. A guest sends bounded player inputs;
it cannot upload arbitrary save state or decide damage. Both processes render their
own camera, HUD and character presentation. The world continues when a window loses
focus; the unfocused player's controls become neutral.

Existing fixed-step movement, stable per-visit entity IDs and save snapshots provide
building blocks. Existing map controllers take one player and need explicit two-player
adapters. Merely running two independent worlds and exchanging positions is insufficient.
Host state snapshots and ordered events must reconcile the guest's world and predictions.

Network setup must report progress, handle cancellation, reject expired/malformed links,
limit connection/message sizes, verify compatible builds/data, and clean up owned helper
processes when hosting ends. Invite tokens are session secrets and must not be committed
or printed in public logs. Input must time out to neutral after a disconnect.

The development internet transport uses Cloudflare Quick Tunnels, which support temporary
public URLs without an account, but are explicitly a development service with no uptime
guarantee; this cannot be silently treated as a production service commitment.
See [Quick Tunnels](https://developers.cloudflare.com/tunnel/get-started/quick-tunnels/)
and [WebSockets](https://developers.cloudflare.com/network/websockets/).

## Delivery and verification weighting

Overall percentages are engineering estimates against these fixed weights, not elapsed
time or a count of files. Credit requires working behavior and relevant evidence.

| Area | Weight | Required evidence |
| --- | ---: | --- |
| Foundation and one-PC test setup | 10% | Branch, agreed rules, independent two-process harness, useful diagnostics |
| Internet Host/Join | 15% | Main/ESC menu, real public invite connection, cancellation, disconnect/reconnect, bounded protocol |
| Two players and movement | 15% | Blue/red clothing, separate cameras/input, synchronized walking/jumping/climbing/swimming and moving supports |
| Combat, personal resources and death | 20% | Nearest-visible AI, coherent damage, individual pickups, timed spectating/respawn, two-player wipe/checkpoint |
| Scenes, transport and progression | 20% | First arrival, single-Alice scenes, safe reunion, boarding barriers, shared saves and level changes |
| Full-campaign verification | 20% | Every visit exercised by both roles, leaves/swimming regression, latency/disconnect tests, single-player regressions |

The [campaign matrix](COOP_CAMPAIGN.md) includes all 39 visits. Each requires both trigger orders, a player
dying during traversal, both dying, checkpoint reload, a guest falling behind, and a
disconnect around a transition. The network test harness should inject delay/jitter
and exercise slow readers rather than assuming loopback performance represents the internet.

## Current evidence

- `coop` exists from current public main.
- `src/coop/rules.rs` implements policy decisions and scenario tests. The death
  policy now drives the movement slice below; scene, boarding and targeting
  policies still need campaign integration. All six initial policy
  scenarios pass: countdown, simultaneous death, saved/cinematic countdown,
  visible targeting, scene ownership/reunion and the boarding barrier.
- Authenticated, bounded WebSocket transport runs on background threads, with TLS
  on public invites, one guest slot, reconnect generations and delay/jitter injection.
  Slow readers disconnect without growing an unbounded queue. Invite secrets stay
  in link fragments and authorization headers, never normal logs or command lines.
- `--profile <folder>` isolates settings and saves. The two-process network probe
  uses the same profile resolution, transport and public tunnel as the future UI.
- On 2026-10-23, the current transport passed 681 Rust tests (one unrelated ignored
  test), a two-process loopback run at 60 ms delay + 30 ms jitter per sender, and
  a two-process public internet run at 40 ms + 20 ms per sender. Both exchanged
  16 ordered messages, verified independent profiles/PIDs and observed clean guest
  shutdown. The internet guest joined immediately after invite publication.
- New tunnel hostnames initially failed DNS resolution when published too early.
  Hosting now waits for tunnel registration and a 12-second propagation grace.
  This passed repeated public tests; it is not a universal DNS availability guarantee.
  Joining now retries failed or empty DNS answers once per second within a
  20-second retry budget on the connection worker. Cancel interrupts the wait;
  a cancelled lookup cannot proceed to connect. An operating-system lookup
  already in progress can still take its system timeout. Persistent failure
  returns a clear retry message. Invite authentication and TLS are unchanged.
- Main/ESC Host/Join now opens the development movement/combat slice. Full-campaign
  integration remains unfinished. These transport probes verify
  networking; the later rendered checks exercise two independently controlled Alices.

The next session layer (`protocol.rs`, `session.rs`, `hosting.rs`) checks exact source
and dependency identity plus the mounted game-data fingerprint. Both roles must
acknowledge loading a visit before guest controls become active. Every reload or
transition has a new epoch: old controls cannot leak into the next visit. Held
movement/attacks become neutral after 500 ms without fresh input, while cumulative
jump/use counts preserve short presses across a busy send queue. An incompatible
guest releases the slot so the same host can accept a correct build. Host setup,
helper download and cancellation run off the UI thread.

The development movement viewer and main/ESC co-op menus now use these APIs;
the campaign still needs integration. The source identity is a compatibility check, not an
attestation that a peer is trustworthy. Guest messages cannot upload world state.

The session changes passed 689 Rust tests (one unrelated ignored test), including
real-socket scenarios for readiness, a transition with input still in flight,
incompatible data followed by a successful rejoin, disconnection and loading timeout.
The shared background Host setup also passed separate-process loopback and public
internet probes with the delay/jitter above. The public probe and development
movement viewer both wait for helper shutdown before exiting.
After tightening cancellation and the forged-message regression, all 21 focused
co-op tests passed again. This final focused run includes the latest source changes.

Overall estimate after completing the village, Pandemonium, both Fortress visits, Beyond and return-school verification rows, plus native-rope recovery, Herbaceous components, the Rolling Stones/Icy Reception/Flora/Sanctum/both Caterpillar's Plot shared-world adapters, Pale Realm cast/world components, replicated weapon sparks/trails, Castling combat/world components and Checkmate combat/scene components and Funhouse battle/machinery/scene components: **about 48.1%**. Credited
points are 8/10 for foundation/diagnostics, 7/15 for internet Host/Join,
10.25/15 for two-player movement/presentation, paired school supports, dry/swimming arrival placement, complete delayed village/Pandemonium traversal in separate clients, rendered independent rope controls, Pale Realm's native paired lift, Castling's portal boarding/carry and replicated weapon sparks/trails, and 9.25/20 for native targeting, host-owned
weapons/damage, personal resources, timed respawns and full saved-state team wipes.
The additional 0.5 points credit tested recovery beside a survivor hanging on an
active native rope: both-role unit coverage plus a captured fall/revival through
independent internet clients. This does not cover custom controlled ropes.
The Herbaceous adapter adds 0.25 movement points for its verified native paired
fulcrum and 0.25 combat points for its integrated Ant/Ladybug targeting, personal
damage and preserved impact effects. These are component credits; no campaign
row or independent-client credit is added by the staged checks.
Flora adds 0.25 combat points for its integrated native Ant targeting/personal
hits, paired spawn visibility and visit-specific health-floor handling.
Sanctum adds 0.25 combat points for its verified native paired boss attacks,
owned grabs/throws and personal healing. Pale Realm adds 0.25 combat points for
its verified paired resident targeting/personal attacks, saved attack ownership,
native shields and preserved scripted NPC duels. That remains isolated cast
coverage; the later world milestone does not add further combat credit. Castling
adds 0.25 combat points for the same bounded scope: native personal targeting,
attacks and saved ownership, with preserved faction duels and single-player
routes. That combat slice adds no world, internet or completed-campaign-row credit.
The separate Castling world milestone adds 0.25 movement points for native portal
boarding/atomic carry and 0.25 progression points for shared doors, all original
royal scene combinations and saved moving-exit ownership. It adds no full route,
independent-client or campaign-row credit. Credit also includes
10.5/20 for shared world replication, ordered contacts, the verified theatre
ownership/reunion slice, school boarding barriers, visit history and restart/backup
restoration of shared checkpoints, and native traversal of all six village scenes
and all four Pandemonium scenes with either player arriving first (also verified through public internet clients).
The preceding 0.25 shared-progression points cover Pale Realm's original puzzles,
scenes, levers and shared machinery, alongside its 0.25 movement credit for
two-rider lift boarding/carry. The preceding 0.5 points cover both Caterpillar's Plot visits' verified shared gates,
wall/secret progression and original scene handoffs (0.25 per visit). The preceding
0.25 points cover Sanctum's shared boss-defeat/spike/mushroom progression
and personal slide arrival; the preceding 0.25 cover Flora's shared wave/departure clock and
original ambush ownership/reunion, following 0.25 points each for Icy Reception's
hazards/portal and the Rolling Stones chase adapter. They do not credit a full
paired route, internet play or a completed campaign row.
Full-campaign verification now earns 120/39 points (about 3.08/20), for those six
visits' complete combined evidence. Checkmate adds 0.25 combat points for native
paired King/chess attacks, saved target ownership and personal healing, plus
0.25 progression points for the original three scenes with either actor, watched
or held-skipped, paused/restored, safe reunion or Funhouse departure. The
84 staged native component records add no full-route, internet or campaign-row
credit. Funhouse adds 0.25 movement points for native paired seesaw, pendulum
and tube carry with obstruction/boarding checks, and 0.25 combat points for its
paired battle, personal healing and saved attack/victory components. A further
0.25 progression points cover its 32 original cinematic combinations, safe reunion,
saved cameras and four personal Jacks/ordinary-speech cases. Full traversal,
internet, recovery and rendered effects remain separate requirements. Total
credited points are about 48.08, reported as 48.1%, up from 47.83 before this scene milestone. The earlier 0.25
presentation points require both native visible-pixel comparisons and real
public-client delivery of impact sparks, swing trails and projectile trails.
They do not cover every other effect or count as a completed visit. Updates include
completed milestones so rounding does not conceal progress. Standalone policy
tests receive no campaign credit; 33 visits remain open. The first five historical
passes still need the contact-system repeats described in the campaign matrix.

Pandemonium now uses each replicated player's own rope grip in co-op, while the
single-player controller and saved grip remain unchanged. The host advances its
shared machinery once and includes both living bodies when checking quest-gate
obstructions. Either player can collect the shared Gnome key. The authored
minecart trigger waits for both living players in its boarding volume, retains the
first requester across a snapshot restore, then runs its original one-Alice ride
and reunites the pair at the far landing. Scene takeover releases old rope grips.

`--coop-pandemonium-check` passes staged real-geometry checks for both first-player
orders: simultaneous grabs on one rope, opposing climbs, wire roundtrip of both
grips, personal release, a two-second boarding wait, requester retention, one
complete minecart ride, safe reunion and the partner collecting the key. Evidence:
`private/coop-pandemonium-adapters1.log`, build
`d5443db066562bb26284db40f43d3e55a91805ade8aac6f787de61f0dea6e5aa`.
These first checks use the real host arena with explicitly staged starting positions;
they do not establish a full route, rendered rope fidelity or separate-client play.

The original watched and skipped single-player Pandemonium routes, state/save
checks and the school/village co-op world regressions also pass. The watched route
finishes in 26,139 ticks with nine jumps, 13 throws, one portal and 47.334 Sanity.
Evidence: `private/coop-pandemonium-singleplayer1.log`,
`private/coop-pandemonium-singleplayer-skip1.log`,
`private/coop-pandemonium-singleplayer-state1.log` and
`private/coop-pandemonium-world1.log`. All 734 Rust tests pass (one pre-existing
ignored test). The newer route and presentation evidence follows below.

The co-op renderer now deforms each rope through the players' animated supporting
hands. Its loose tail follows the physical grip, so a hand-placement animation
cannot inject velocity into the strand. Dynamic mesh indices follow the added
hand rings. The two players retain independent checked pendulums; this is not a
coupled two-person load solver. Unit coverage checks player-order independence,
exact hand contacts, finite geometry, pause, either player's release and the
invariance of tail physics under animated hand movement. Attachments are ordered
by body height and samples follow that path, so crossing radial lengths while
swinging cannot place the tail above the lower player. Single-player continues
using its original physical grip and rope shape.

`--coop-pandemonium-route-check` now passes from the normal entrance to the
authored Fortress exit with both first-arrival orders, using ordinary inputs and
no position/resource/world writes. Both runs take 33,993 ticks, retain a personal
Cards pickup each and finish at 60.934 Sanity each without a death or checkpoint
rollback. Each first player owns all four scenes (warning, minecart, Gnome return,
airship), with three safe paired reunions before the outgoing transition. The
original return trigger previously only set a progression flag: it now starts the
existing return cinematic, which the single-player route also explicitly checks.
This result is a single-process two-body simulation, not a network traversal.

Evidence: `private/coop/pandemonium-route6/coop-pandemonium/` and
`private/coop-pandemonium-route6.log`, build
`a9b50ed49c886e859e6a0a9cbcaec0d833fd08d799a4e1f74527fa0626da844b`.
The staged adapters repeat successfully on that build in
`private/coop-pandemonium-rope-native8.log`, with rope/cart captures under
`private/coop/pandemonium-rope8/coop-pandemonium/`. The shared-rope and return-reunion
captures were inspected. The accelerated checks advance the tail every simulation
tick rather than only when drawing an occasional frame.

Earlier failed routes remain under `private/coop/pandemonium-route1` through
`pandemonium-route4`: respectively an incorrect height goal on the sloping return
street, excessive waiting on damaging ground, the disconnected return cutscene,
and a portal check that did not allow time for the restored scene. The driver now
follows horizontal goals on that street, moves both players through the hazard
together and waits for the first portal arrival's scene/reunion. These fixes do
not change movement speed, level damage or the requested first-arrival policy.

All 736 Rust tests pass (one pre-existing ignored). The watched/skipped
single-player routes, cinematic pause/save/skip checks, machinery state checks
and school/village co-op world regressions pass on the preceding
`b5b78ca663f3e3c6fa94aa526cdf5479b95b2a09a1cbbf89cc39eea1ba005b3d` build, in
`private/coop-pandemonium-sp7-{route,skip,cinematic,state,world}.log`. The watched
route takes 26,897 ticks and finishes at 47.334 Sanity; the skipped route takes
25,529 ticks and finishes at 32.334 Sanity. Both retain nine jumps, 13 throws and
one portal. The native single-player machinery/cast/rope render checks also pass
in `private/coop-pandemonium-sp-render7.log`; the rope ascent capture was inspected.
The final co-op native repeat used the authorized PC fallback while Anode was
busy; the preceding full native render checks ran in Anode. All error logs are
empty and all owned game processes closed. The final change only orders multiple
co-op grip attachments; the single-player one-grip path is unchanged. The later
public separate-client traversal, recovery and rejoin evidence follows below.

Run the staged check in Anode after building:

```text
looking-glass.exe --data <base-folder> --coop-pandemonium-check --no-audio --profile private/coop/pandemonium-adapters
looking-glass.exe --data <base-folder> --coop-pandemonium-route-check --no-audio --profile private/coop/pandemonium-route
```

The first public two-process Pandemonium attempt exposed a delayed rope-use race:
Red's ordinary airborne physics caught the rope before her grab press reached the
host, so the old toggle interpreted that press as a release. She fell to damaging
ground and died; this is a failed traversal, retained in
`private/coop/pandemonium-test-6254d8209f784c2ab361f596a8e0b850/` on build
`5c1a8e34ba15f14de44cc192876da422cab3ee147bad0851bbf2b7081fcb9b5f`.

Protocol revision 4 separates cumulative grab and release edges according to the
local player's observed grip and nearby rope. The host consumes each edge once and
only requests the matching ordinary rope action: a late grab cannot release an
existing grip, and a late release cannot attach a detached player. Edge counts
remain bounded and expire under the input watchdog. Dialogue, doors, movement
locks and single-player controls keep their existing paths. The regression test
reproduces automatic attachment followed by a delayed encoded grab, then verifies
an explicit release, duplicate suppression, timeout and counter bounds. All 740
Rust tests pass (one pre-existing ignored) in
`private/coop-pandemonium-network-rope-tests2.log`.

The independent-client driver now supports rope controls, horizontal goals and
local map markers using ordinary inputs. The Pandemonium harness checks shared
rope descent, minecart boarding, all four scene owners/reunions, personal Cards
and the authored Fortress transition. Run it in either first-arrival order:

```text
python tools/test_coop_pandemonium.py --data <base-folder> --first blue --delay-ms 40 --jitter-ms 20
python tools/test_coop_pandemonium.py --data <base-folder> --first red --delay-ms 40 --jitter-ms 20
```

The Blue-first repeat passes through the public invite connection with 40 ms delay
and 20 ms jitter per sender on build
`ec4ccf71f5a634f1c8c03ccbc6924fa3492e9fac1421dd813cd4937c06118a48`.
Both clients agree on all four Blue-owned scenes and three reunion ticks, retain
their actual personal Cards across the Fortress transition, and match complete
player state at 207 identical authoritative ticks. There are no deaths, respawns
or checkpoint resets; Blue arrives with 27.134 Sanity and Red with 21.135.
Shared-rope, landing, boarding-wait and initial Fortress cinematic captures were
inspected. The last capture is during Fortress's arrival sequence; it does not
verify that next visit. Both client processes exit normally and the private invite
is removed. Evidence: `private/coop/pandemonium-test-25726e42ca974a459cf70cd356606c0e/`,
`private/coop-pandemonium-network-blue2.log`. This run used the authorized local
desktop fallback while Anode was busy. Later recovery/rejoin results follow below;
consumable, first-guard and full-route encounter comparison checks now follow in
the encounter section and complete the Pandemonium verification row.

The Red-first repeat also passes on the same build and delay/jitter settings,
running in Anode. Both clients observe all four Red-owned scenes, three reunions,
the delayed cart departure and personal Cards carried into Fortress. They agree
on 89 matching authoritative ticks; Blue arrives with 39.867 Sanity and Red with
23.468, with no deaths, respawns or checkpoint resets. Both rope views, Red's
boarding-wait notice and the initial Fortress cinematic view were inspected.
Both game clients and the test job exit normally (job exit 0), and the invite is
removed. Evidence: `private/coop/pandemonium-test-a2cf6558bee64719af5fda4834594ca6/`,
`private/coop-pandemonium-network-red1.log`. Python compilation and all 19 existing
setup/source-package checks pass. No campaign row is promoted to passed by these
clean traversals alone.

Pandemonium now has separate ordinary-input recovery and fresh-process rejoin
checks. The recovery test follows the real entrance rope to the warning scene,
then sends each player onto the damaging street while the survivor walks on the
supported landing. After both individual recoveries, it sends both low-health
players into the hazard and compares the result with the game's earlier automatic
checkpoint. It never writes health, positions, inventories or shared world state.
Placement is checked on each client's first restored frame, required within one
simulation second, allowing normal horizontal movement during the recorded host
ticks since restoration; resources
and the selected quest/scene completion fields compare exactly. The rejoin check
closes the guest process before the airship exit, starts a new one using the same
invite/profile, compares both players' resources and completed scenes, then
continues through the authored transition.

```text
python tools/test_coop_pandemonium_recovery.py --data <base-folder> --first red --delay-ms 40 --jitter-ms 20
python tools/test_coop_pandemonium.py --data <base-folder> --first blue --rejoin --delay-ms 40 --jitter-ms 20
```

The first recovery attempt verified both ten-second respawns but its shared-wipe
approach snagged scenery. The second attempted approach also snagged scenery;
the test now uses the ordinary route planner to reach a point farther into the
damaging street. The third and fourth runs reached the wipe but a fixed 20-unit
placement assertion did not account for movement before the reports arrived.
The verification now uses the first restored samples and bounds horizontal drift
by the normal 320-unit/second maximum and the exact elapsed host ticks; the
vertical platform tolerance remains 20 units. These diagnostic runs are retained
under `private/coop/`: `pandemonium-recovery-f763a479554d4031b5e41e2a707fc4a7`,
`pandemonium-recovery-13bddb5bb8074e8cb0d65cb96d74041e`,
`pandemonium-recovery-1d4418c09da24a8fbe93c9f757a0a9ee` and
`pandemonium-recovery-3f1b82a621884511bf09cd1daaa1e2c1`.

The fifth recovery run passes on build
`ec4ccf71f5a634f1c8c03ccbc6924fa3492e9fac1421dd813cd4937c06118a48`, over the
public invite with 40 ms delay and 20 ms jitter per sender. Red dies at tick 6,054
and revives at 7,254; Blue dies at 8,962 and revives at 10,162. Both clients agree
on each exact ten-second interval, each 25-Sanity revival 48 units from the
survivor, both moving spectator cameras, retained inventory and the survivor's
unchanged health. The team wipe at tick 11,170 restores the actual automatic
checkpoint saved at tick 1, including full personal resources and the earlier
quest/scene flags. The first host and guest placement samples, respectively 4 and
12 ticks after restoration, have zero horizontal displacement from the saved
positions. Both spectator views and both restored-checkpoint views were inspected.
Both clients and the Anode job exit normally; the invite is removed. Evidence:
`private/coop/pandemonium-recovery-1127245225934d65ac70d7f9af173184/`,
`private/coop-pandemonium-recovery-internet5.log`. This is the entrance-side
hazard/recovery scenario; it ends at the restored checkpoint without an outgoing
transition. The traversal and rejoin tests exercise the later route separately.

The fresh-guest rejoin run also passes on that build and public delay/jitter
settings. Blue leads the full route; Red closes her original process just before
the airship and joins again in a new process. The before/after reports preserve
both players' Sanity, Will, owned/collected items, equipped weapon, school items,
staff components and difficulty, as well as the minecart, key and completed-scene
flags. Both continue into Fortress with their personal Cards (Blue 45.734 / Red
15.668 Sanity), without deaths, respawns or checkpoint resets. All four owners,
three reunions and 126 matching authoritative ticks pass. The final initial
Fortress cinematic views were inspected; both clients and the Anode job exit
normally, and the invite is removed. Evidence:
`private/coop/pandemonium-test-b5965aedc3274cba8065c2c111d4421f/`,
`private/coop-pandemonium-rejoin-internet1.log`. The game source is unchanged from
the preceding 740-test build; this checkpoint extends the independent-client
harnesses and records their evidence. Later focused consumable/guard checks are
recorded below, followed by both successful full-route encounter comparisons.

The native enemy combat foundation now supplies two live targets to each enemy
and keeps damage, knockback and Will drain separate for Blue and Red. Closest
visible selection respects cover, death, invisibility, card-guard sight cones and
club-guard home limits. An attack retains its valid victim while it finishes;
ordinary last-seen pursuit continues when both players take cover. Projectiles
can hit either living body regardless of the enemy's current aim. Clockwork steam
uses separate hit bits so both players can be hit once by the same attack.
The live roster is omitted from saved NPC state, preserving the single-player
save representation. Feedback aggregation preserves both players' results.

The combat-foundation revision passed **703 tests, one unrelated ignored test**.
`Npcs::update_players` advances a cast once and lets either living player reach its
ambush triggers. The current viewer integration adds host-owned weapon simulation,
guest NPC/weapon replication and individual pickup presentation, as described below.
Two summoned demons, boss/controller adapters and shared world checkpoint restoration
remain unfinished. No campaign matrix row is credited by these tests.

The headless `--combat-check` also passes with user-supplied school geometry and
the actual club-guard animation timings. At each of 30, 60 and 144 Hz, the closer
Red Alice receives 60 damage during the first 12-second phase and Blue receives
none. After a guard-state serialization roundtrip and Red's death, the survivor
Blue receives 84 damage in the next phase and Red receives none. Final position
and damage agree across all three frame rates. The existing single-player pursuit,
Blade contact and defeat checks pass alongside this. Reproduce with:

```text
cargo test --locked
cargo build --locked
target/debug/looking-glass.exe --combat-check --data <original-base-folder>
```

### Integration constraints found in the existing engine

- `viewer.rs` owns one player, inventory, character, camera and save record. Add
  explicit per-player state while keeping world time and progression on the host.
- `Menu::run` currently waits in a modal loop. Co-op needs a per-frame menu update
  so one player's settings/menu and window focus cannot freeze the other's world.
- Level controllers advance shared machinery and enemies through single-player
  hooks. Do not call a shared controller twice per tick to process two players;
  split world advancement from per-player contacts and rider displacement.
- Enemy selection must be per enemy, not one globally selected player for the
  whole level. Route damage/impulses to the player that actually received the hit.
- Alice's blouse/leg/inner-mouth regions share a texture. Runtime red clothing
  needs a blue-fabric mask on known clothing textures, preserving skin, apron,
  mouth, hair, and power-up appearances. Never distribute altered original assets.

### Shared world contact adapter

`interaction::coop` separates two players' trigger occupancy/cooldowns from the
shared event runtime. Both living players can receive hazard damage or use a
teleporter independently. Simultaneous contacts with a shared trigger fire it once,
with arrivals ordered by swept contact time, then stable identities for exact ties. Each
activation retains its player owner for scene/exit handling. Shared delayed events
advance once even when both players have contacts. The existing event gates and
one-shot rule history remain authoritative.

Door use shares the single-player implementation. Simultaneous use of the same
paired leaves toggles them once, and the door's swept arc checks both living
bodies. Personal contact history can be serialized beside the interaction snapshot
and cleared for one respawn without rearming shared one-shot events. The original
single-player save representation is unchanged.

The rendered co-op viewer now owns this shared world on the host and replicates
its event history, contacts and story state. Controller/mover adapters and complete
campaign progression still need integration. No campaign-matrix credit is assigned
merely for adding this interface. Its actual-map
check is `--coop-world-check --data <original-base-folder>`; `--world-check` remains
the single-player regression.

The interaction refactor passed 717 Rust tests (one unrelated ignored test).
Eight new cases cover staggered and simultaneous hazard contacts, independent
teleports, both first-arrival orders, competing exits in one update, repeatable
trigger reentry, delayed world time, contact snapshot validation/restoration and
simultaneous door use with either body blocking its swept arc. These are engine
checks and do not establish a completed two-player campaign visit.

The actual-school check passed with either Blue or Red opening the door at 30,
60 and 144 Hz. Both bodies first walk against the closed door for long enough to
stop, remain blocked for another second, then cross after one use. The authored
fatal-fall volume damages both independently; a shared snapshot/contact roundtrip
and resetting Blue's contacts leave Red's cooldown intact. Evidence is in
`private/coop-world-native.log`.

All 39 initial world snapshot hashes, event signatures and byte lengths match the
pre-refactor executable. This was compared independently from the `HASH` records
against `private/coop-world-before-registry.json`: the existing registry comparator
intentionally skips registered controllers, so its summary alone is insufficient
for this claim. Evidence is `private/coop-world-after-registry.log`. These are
single-player initialization/save-format checks, not co-op campaign traversal.

The broad single-player world check exposed an older driver assumption: its steam
floor route allowed two seconds to walk more than 368 units at 104 units/second.
Both the saved pre-refactor executable and this revision stop at the same position
(`y=2637.4326`). The driver now derives its duration from the current walk speed;
its required endpoint, support and collision assertions are unchanged. Failed
pre-fix logs remain in `private/coop-world-before-single.log` and
`private/coop-world-single.log`.
After that driver correction, `--world-check` passes all three school-door rates,
all eight floor supports, both visits' seam crossings at 30/60/144 Hz, and fatal-fall
reset. Each seam route ends at `y=2845.4404`; evidence is
`private/coop-world-single-final.log`. Seven source-package tests also pass.

Live contact dispatch now resolves personal damage/teleport immediately, ending
that body's old sweep before it can consume a later scene or exit. The partner's
contacts remain eligible, and teleport cooldowns survive relocation.

### Live world and school theatre integration

`coop::stage` owns shared interactions, contacts, story, hints and the active scene
owner. Scene ticks advance the original single-Alice controller exactly once,
freeze the partner and hold respawn timers. Both windows use the replicated
original camera, dialogue and level art. Completion finds a supported nearby
position outside hazard volumes before returning control to both players.
Relocations cut camera/body interpolation without being counted as death respawns.
An entrance wipe restores shared world/story/contact state before checking body
placements, together with native NPC and character state.

`--coop-world-check` now also runs the actual theatre for both first-arrival roles:
5,592 fixed ticks each, frozen waiting partner, camera/mover snapshot agreement,
48-unit safe reunion and full shared-state reset. The check passes scene state
through the same compressed host message envelope used on the network.
Single-player `--world-check` and `--school-check` pass; all 39 initial visit
records match the retained pre-refactor baseline independently of the registry's
comparison exemptions. These checks do not establish a completed campaign visit.

The first rendered attempt exposed an envelope problem: the old internally tagged
message buffered integer entity-map keys in a form that could not be restored.
Host messages now use an external tag, with a regression covering real interaction
snapshots. Protocol revision 2 and the exact build identity prevent mixed clients.

The intermittent initial-join failure also reproduced against the retained older
executable: a valid HTTP upgrade delayed by 50, 200 or 500 ms was aborted every time.
Windows accepted sockets inherit the listener's properties
([Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept)); the bounded
synchronous upgrade now explicitly selects blocking mode before authentication,
then returns to nonblocking exchange. The regression forces the inherited mode
even on other platforms. `tools/test_coop_handshake.py` exercises the production
listener with delayed bytes without using game assets. Failed pre-fix evidence:
`private/coop/handshake-test-a241a70853194f21845271a9abdedc0e`.

`tools/test_coop_scene.py --data <original-base-folder> --first blue` starts two
independent rendered clients near the theatre. The chosen player walks into the
authored trigger; neither driver starts a scene directly. Use `--first red` for
the other arrival order, `--local` for loopback, and `--delay-ms` / `--jitter-ms`
for network conditions. It verifies scene ownership, partner immobilization,
camera movement, matching authoritative reunion ticks and nearby living bodies,
and captures the scene and reunion. This is a scene fixture, not a traversal test.

Add `--rejoin --rejoin-at 6` to restart the guest during the camera introduction,
or `--rejoin --rejoin-at 30` during dialogue. Exercise each with `--first blue`
and `--first red`. The test waits for observed host disconnection and checks
that player, scene and world state stay frozen. A fresh guest must inherit the
same owner, scene and dialogue clocks, pending contacts, personal resources and
hidden waiting partner. Both clients then watch the original scene through its
normal completion and must agree on the safe reunion. `--local` scopes the
result to loopback, including the requested artificial delay/jitter.

All four owner/phase combinations are available as one automated suite:
`python tools/test_coop_regressions.py --exe target/release/looking-glass.exe --data <original-base-folder> --only scene-rejoin-blue-intro scene-rejoin-blue-dialogue scene-rejoin-red-intro scene-rejoin-red-dialogue`.
The report records the scene/dialogue clocks and both clients' reunion evidence.
Public invites and 40/20 ms delay/jitter are the defaults; `--local` remains an
explicit loopback-only check. No images are opened by the runner.

The staged entrance is single-file. Before simulation, its collision-checked
front spawn slot is assigned to the requested first arrival. Ordinary walking
and the actual contact trigger still start the scene; no live body, clock or
progress state is moved to force a result. The harness fails early if that
player cannot reach the trigger. Reports, fixture source and failures stay in
ignored private evidence; screenshot inspection is not required.

Rendered checks on 2026-10-23 passed with Blue first on loopback and Red first over
the public TLS tunnel with 40 ms delay plus 0–20 ms jitter per sender. Both windows
observed the same owner and reunion tick, and a 48-unit separation. Captures were
inspected; the final presentation uses the original scene field of view/fades and
suppresses the ordinary gameplay HUD. Evidence under `private/coop/`:
`scene-test-538e7bf8119543b88d43e770ed292147` (Blue, before final presentation tuning),
`scene-test-459a4e14b3724b5ba9ebe5769871a9b9` (Red, final presentation).
The final production-listener regression accepts upgrades at all three delays:
`handshake-test-1449f00e111d4c608db6c5e3dd843f94`.

At the theatre checkpoint, the full Rust suite passed 723 tests (one unrelated ignored test), including
the real entity-map envelope and delayed-upgrade regressions. Source-package (7),
portable setup (6) and Windows setup (6) tests also pass. Final headless evidence:
`private/coop-stage-verified-tests.log` and `private/coop-stage-verified-native.log`.
A subsequent public recovery attempt did not reach gameplay because the guest
could not resolve its newly created tunnel address; retain
`private/coop/play-test-72524584443844d1b1d5ad372ddd5d9c`. The current DNS grace
period does not guarantee immediate invite availability. That remains an internet
hosting reliability limitation; the successful theatre run does not erase it.
The local recovery regression with the same 40/20 ms conditions passed: Red died
at tick 2,077, returned at tick 3,277 with 25 health and 48-unit separation, then
both clients restored the checkpoint at tick 4,261. Shared world restoration did
not change the ten-second timer or spectator/recovery camera behavior. Evidence:
`private/coop/play-test-d49bd2e45c5d402a9256785059bbf298`.
The final local Blade-combat regression also passes with the shared world enabled:
both players attack, both clients see the same defeated guard, Blue remains at
100 health and Red reaches 64. Evidence:
`private/coop/play-test-46f596d015874301b65afbe05631577b`.
All owned game and tunnel processes were gone after testing.

The level-change/load-barrier path is connected and carries each player's own
resources, inventory and death countdown. Full round-trip campaign state caching,
shared saves, controller combat, moving supports beyond the school adapter below, co-op audio,
and live transition/disconnect coverage remain unfinished. Free-play controller
updates outside the school remain pending; one theatre passing cannot
stand in for those systems or for the 39-visit campaign matrix.

## Shared school lifts

The host now advances the school machinery once per world step. A moving support
plans both living riders' positions before committing either body, its own collider
or its motion clock. An obstruction above either rider, or a partner below a
descending support, stops the movement without a partial carry. Jumping bodies
are not dragged along. The single-player path uses the same helper with one body.

The spiral and ordinary elevator wait for both living players before activation
and before leaving either endpoint with a passenger. Empty return trips remain
available. The observatory lift's one-shot contact also waits for both living
players. Pending requests preserve the first living requester across compressed
world/contact snapshots; a dead requester yields to the survivor. Waiting does
not consume the authored trigger or its cooldown. A notice explains the wait.
The observatory check stages an already-open gate; its complete quest/cutscene
route is not yet verified in co-op.

`--coop-world-check` now covers all three boarding contacts in both arrival orders,
with both alive, the waiting partner dead, and the original requester dead. It also
checks the two ordinary lifts through complete cycles at 30/60/144 Hz, keeping
both bodies clear before and after physics, preserving rider offsets, restoring
mid-trip state, holding at both endpoints, returning empty and carrying a sole
living rider. Obstructions over either rider stop both bodies and the lift clock;
removing the obstruction resumes travel. The integrated host world loop passes
both ordinary lifts with Blue first and Red first. Theatre/reunion/wipe checks
still pass. Evidence: `private/coop-lifts-native4.log`.

The first fixture used an outside spiral position that encounters the landing
lip near the top. The safety check correctly stopped both riders; the unobstructed
full-cycle fixture uses the inside position. Earlier fixture setup errors used a
starting point above the elevator deck and an incorrect assumed body height for
the temporary ceiling. Their failed logs remain private; they are not counted as
passing runs.

The full Rust suite passes 726 tests (one unrelated ignored test), including three
atomic carry regressions. Single-player school state/movers, shelf footing at
30/60/144 Hz and continuous school traversal pass. All 39 fresh visit records are
exactly equal to the frozen pre-refactor baseline, including registered controllers.
Evidence: `private/coop-lifts-tests.log`, `private/coop-lifts-school1.log`,
`private/coop-lifts-footing1.log`, `private/coop-lifts-school-route1.log`, and
`private/coop-lifts-registry.json` compared with `private/coop-world-before-registry.json`.

`tools/test_coop_lift.py --data <original-base-folder> --lift elevator --local
--delay-ms 40 --jitter-ms 20` runs two independent rendered clients on an actual
lift. It uses the ordinary Host/Join and contact/movement pipeline, with staged
starting positions and no injected world updates. Wait for the full-cycle check,
capture each window with F12, then close both windows. Omit `--local` for public
TLS; `--lift spiral` selects the rotating lift. This fixture verifies the ride;
arrival-order and obstruction coverage comes from the native checks above.

For automated boarding and guest reconnect checks, use
`tools/test_coop_school_lift.py --data <original-base-folder> --first blue --rejoin`
(and repeat with `--first red`). Both players walk from a staged lower-library
start. The first rider waits for the second before departure. During rotation,
the runner restarts the guest, verifies that the disconnected host holds the
full world and both bodies, then checks the fresh client's platform clock,
pose, rider offsets and personal resources. The ordinary ride must still reach
the upper landing and match sent/restored player and world snapshots. No live
positions, platform state or inventory are written; captures are not inspected.

Both arrival orders are available in one automated report:
`python tools/test_coop_regressions.py --exe target/release/looking-glass.exe --data <original-base-folder> --only lift-rejoin-blue lift-rejoin-red`.
Public internet invites and 40/20 ms delay/jitter are the defaults; `--local`
records a loopback-only result. Reports and original-data evidence stay private.

The delayed loopback elevator check passed on 2026-10-23: both riders reached
696.03125 and returned, with aligned body/mover snapshots in both clients. Both
captures were inspected and both processes exited successfully. Evidence:
`private/coop/lift-test-e98f064766774ad08854c7ba79f3192e`.
The rotating lift also passed through the public TLS tunnel with 40 ms delay plus
0–20 ms jitter per sender. Both riders reached 64.03125, completed the return and
stayed aligned with the shared support in both windows. Captures were inspected
and both clients exited successfully; no game or tunnel helper remained. Evidence:
`private/coop/lift-test-3e9dd3b1787b413d8767094b6d70fb16`. This successful connection
does not remove the previously recorded fresh-hostname DNS limitation.
Every campaign-matrix row remains **Not run**.

## Shared visit history and arrival placement

The host now retains each departed visit's shared controllers, story/hint state and
native NPC state for the lifetime of the session. Canonical first entrances share
one cache key; authored return visits remain separate. Re-entering restores that
world before placing the pair, resets their old contact sweeps, and makes the
restored world the new entrance checkpoint. Individual resources and pickup
entitlements travel with each player's roster entry, never with an old world copy.
Destination geometry, restored collision, arrival positions and art are prepared
before committing the local visit history. A load error still ends the co-op session;
host-owned checkpoints now support save/resume, but failed-transition recovery still ends the session.

Protocol revision 3 adds an ordered guest loading acknowledgement. The host keeps
draining old controls until it receives that acknowledgement, then begins its
synchronous asset load. The queue stays bounded and retains its ordinary stall
deadline; loading does not require increasing either. This addresses the public
travel run in `travel-test-7f597a3d82be4a71a8d81999563481e8`, where late controls
filled the host queue during asset loading and disconnected the guest.

Partner placement accepts clear swimming positions in the survivor's water volume.
Dry placement checks a complete footprint against the local floor plane, including
slopes. Entrance placement can search up to 256 units below an authored elevated
start; ordinary revival remains limited to 32 units down. Walls, narrow ledges,
harmful liquids and hazardous destination triggers remain rejected. These changes
are confined to co-op placement; single-player recovery is unchanged.

`--coop-campaign-check` checks all 39 authored entrances for two clear bodies,
two seconds of collision/swimming physics, world snapshot roundtrip and an entrance
wipe. It deliberately does not advance the level controllers or establish traversal.
All 39 pass (`private/coop-progression-entrances4.log`). The underwater temple and
first garden entrance also pass both death roles: ten seconds of following the
living swimmer, then revival at the same depth with 25 health and fresh air. This
uses real entrance geometry and ordinary movement/death policy, without enemy AI.
The shared-world suite still passes its school lifts, theatre reunion and
revisit-checkpoint rollback (`private/coop-progression-world2.log`).

`tools/test_coop_travel.py --data <base> --reconnect --delay-ms 40 --jitter-ms 20`
uses two rendered clients and a public invite (add `--local` for loopback). Its
explicit `--coop-travel-check` driver opens the first school door with ordinary input,
requests a normal load barrier to the return entrance, disconnects/rejoins the guest,
then requests the first entrance again. It checks exact world/NPC restoration,
separate visit histories, personal resources, paired relocations and clean exits.
The destination requests are staged; this does not exercise the quest-gated exits.

Normal-menu save eligibility has an Anode fixture:
`python tools/test_coop_menu_save_rules.py --exe target/release/looking-glass.exe --data <original-base-folder> --seat-session <Anode-session-id> --case death`.
It uses ordinary school guard damage for both death roles, real ESC/Save input,
unchanged checkpoint files during refusal, exact countdown/camera checks, and
successful saves after revival. Use `--case scene --first blue` or `--first red`
to watch the staged original theatre with that owner: saving is refused during
its introduction and dialogue, then succeeds after safe reunion. These are core
component checks, not a complete school route. Public invites and 40/20 ms
delay/jitter are default; `--local` isolates behavior over loopback.
Forward `ANODE_AGENT_ID` and `ANODE_LEASE_TOKEN` from the active Anode lease to the
test process, and retain that lease throughout the run. The installed Anode CLI
focuses freshly discovered owned game windows when Windows refuses activation
from the background test job; it does not close or modify unrelated seat apps.

Save-file failure and recovery has a separate real-menu fixture:
`python tools/test_coop_menu_storage.py --exe target/release/looking-glass.exe --data <original-base-folder> --seat-session <Anode-session-id>`.
It uses real Windows file locks in isolated profiles, checks visible Save errors
and live clients, releases the locks and retries. Fresh game processes then use
Continue co-op with a damaged primary, join through a new public invite, display
the backup recovery notice and repair the primary without losing the backup.
With both files damaged, the menu reports failure and returns to the exact
suspended solo game, whose controls still work. Three solo returns and the original
solo slot are checked. Add `--local` for loopback isolation; it is not internet proof.
No live gameplay state is injected. An opt-in passive panel-message receipt avoids
image lookups. Primary replacement failure can advance the backup to the previous
valid primary; the fixture checks recoverability, not two-file transaction atomicity.

Host/Join error recovery has a separate real-menu fixture:
`python tools/test_coop_menu_recovery.py --exe target/release/looking-glass.exe --data <original-base-folder> --seat-session <Anode-session-id>`.
It exercises a real failed DNS lookup, an owned HTTP 503 responder, cancellation
of a stalled connection, and failed/cancelled Host setup in an isolated profile.
The final retry must connect both existing game processes through a public invite,
save the shared checkpoint, leave, and restore the suspended solo state exactly.
Only the temporary profile is used for the filesystem fault. The test does not
change network settings or fabricate gameplay state. Internet provider failures
still fail the run; no automatic provisioning retry bypasses their limits.

Loading-barrier reconnects can be tested separately with
`python tools/test_coop_regressions.py --exe target/release/looking-glass.exe --data <original-base-folder> --only travel-rejoin-outbound travel-rejoin-return`.
Public invites and 40/20 ms delay/jitter are the defaults; add `--local` for
loopback. The runner briefly suspends only its owned guest before departure,
observes the host waiting at the actual Loading barrier, verifies frozen world
and player state, then terminates and restarts that guest on the same invite.
It checks the pending destination, paired arrival, personal resources, unchanged
simulation clock during loading, fresh process identity and cached-world return.
No positions, resources, world state or readiness messages are injected. Native
session tests also interrupt after the Loading acknowledgement, after guest
readiness, and after the host finishes loading. These component checks do not
certify quest exits or the full campaign.

On 2026-10-23, the public-invite run `travel-test-9b33f710636c4e03a1a54e81c9830d98`
passed all gameplay assertions at 40 ms delay + 20 ms jitter per sender, including
guest replacement, three visit epochs, exact cached world/NPC restoration and
preserved inventories. Both final views were inspected on the user's desktop after
Anode became occupied. The run's `ready.json` records those gameplay checks. The
manual capture/close stage timed out because the foreground automation's F12 did
not produce game captures; therefore this is not a completely green harness run
or proof of clean voluntary exits. Its cleanup removed the owned clients, tunnel
helper and invite. The earlier loopback run `travel-test-9ec9dd5f7d1c43048b14e2d791f76183`
also passed gameplay assertions but exceeded its manual inspection deadline.
The subsequent public run `travel-test-b41a73f3337d46a2ada6b94db1d03b3a`
passed the entire harness on Anode at the same delay/jitter, including guest
replacement, automatic captures, clean exits and helper/invite cleanup. Both
final captures were inspected. The fixture captures after five seconds in the
returned visit, then closes the guest and host itself; it requires no desktop
input. The full Rust suite at the travel milestone passed 730 tests, with one pre-existing
ignored test (`private/coop-progression-tests3.log`), including the new slow-load
barrier regression. No co-op release or full campaign playthrough is claimed.

## Continuing a single-player game as co-op

Load the solo save normally, then choose **ESC > Co-op > Host > Host this game**.
Blue keeps her current position, movement state, health, earned inventory and
weapon actions. Doors, quests, NPCs, story progress and cached first/return visits
carry over. Red appears in a collision-checked nearby position with her own copy
of the earned resources; Blue's active projectiles and summons are not duplicated.
Pickup history is copied so previously collected items do not refill on import.
From then on, both players collect and spend resources independently.

The original solo session remains suspended and its save files are unchanged.
Co-op uses its separate checkpoint. **Continue co-op** restores that checkpoint;
**Start chapter** deliberately starts at the chapter entrance. Finish active
scenes/conversations and leave fly mode before importing; a dead or obstructed
Alice, or a location with no safe partner placement, produces an error instead
of discarding progress. Load a living save or move somewhere wider and retry.

The reproducible native fixture and internet process test are:

```powershell
python tools/test_coop_import.py --exe target/release/looking-glass.exe --data <local-base>
```

Run it in Anode. It generates a solo save in a fresh isolated profile, proves
exact conversion of Blue/world/NPC/character and cached return state, then hosts
that save through the production loader (`--coop-host <invite-file> --load slot1`).
The guest reconnects in each phase: initial import, both-process restart from the
co-op save, restart after abruptly terminating the host, and damaged-primary
backup recovery. The terminated host must leave both committed files unchanged;
the guest must detect the loss within 25 seconds. Native receipts verify the full
restored state before simulation; both network views and unchanged solo files
are checked without image inspection. The native fixture also checks a swimming
import and refuses to overwrite an existing fixture slot. This is persistence
coverage, not a campaign completion claim.

The native fixture also checks real Windows sharing violations during primary
and backup replacement. Failed saves must preserve both disk files and the
previous in-memory team-wipe point. It verifies stale temporary files are ignored,
a successful save updates the wipe point, and saving after a damaged primary
preserves the last good backup. These are deterministic storage fault cases;
they do not simulate loss of electrical power or exhaust the disk.

## Native menu acceptance

Run `tools/test_coop_menu.py` inside Anode with `--exe`, `--data`, and
`--seat-session <session ID from seat_status>`. It creates isolated profiles,
loads a solo slot through the original menu, and uses native mouse/keyboard
input for Host this game, Join, Save checkpoint, Leave, and Continue co-op.
It also checks batched typing, malformed links, cancelled joins, expired links,
independent inventories, a fresh second invite, and resuming the suspended solo
game. Exact before/after hashes verify that co-op leaves solo state intact.

The opt-in `LOOKING_GLASS_UI_PROBE=1` records the actual button rectangles and
state receipts in the selected profile. It accepts no UI commands. Invite input
is represented by its length and hash, never the token text. The runner reads
clipboard text only after its own Copy invite action changes the clipboard.
Window ownership and the requested Anode session are checked before input.

Internet is the default and a provider failure fails the run. `--local` sets the
explicit `LOOKING_GLASS_UI_PROBE_LOCAL=1` isolation mode; its receipts say
`internet: false`, and it cannot count as public internet acceptance. Ordinary
menu hosting remains online. This test verifies input and state routing without
image inspection; it does not replace visual layout or campaign verification.

## Shared co-op checkpoints

The host owns a separate `coop-checkpoint.json` in its settings/profile folder.
It contains both bodies and individual inventories, character actions/projectiles,
the shared stage and NPC state, and cached first/return visits. The host's ESC
menu can save while both players are alive and outside a cutscene/transition.
Normal sessions also save at the first freely controlled moment of each chapter;
leaving co-op does not silently replace a manual checkpoint. Explicit development
fixtures do not autosave into the user's checkpoint.

Continue co-op restores the saved chapter even if another chapter is selected in
the main menu. Its network epoch starts fresh and a new invite is issued. Red's
body/resources come from the host; the guest does not need a save file. A team
wipe restores the same world, NPCs, character actions and two bodies as the saved
checkpoint, including after restarting both clients. Individual ten-second
respawns still use nearby placement rather than rolling back the world.

The versioned co-op format is separate from single-player format 12. It checks
the game-data fingerprint, checksum, campaign/difficulty, living roster and
structural limits, with a 32 MiB file bound. Scene/character/actor state and body
placement are validated while constructing the replacement scene. A valid old
primary becomes `.json.bak` before atomic replacement; a damaged primary never
replaces that backup. Loading can recover the backup and reports this to the
host. Saving changes the in-memory wipe checkpoint only after the disk commit
succeeds. Invites, tokens, source assets and connection identities are not saved.

`tools/test_coop_checkpoint.py --data <original-base-folder>` runs on Anode with
two independent profiles, a real public invite and 40 ms delay/20 ms jitter per
sender (`--local` selects loopback). It combines the staged three-visit travel
fixture with distinct seeded health/Will values, closes both clients, starts a
new host/guest pair from the save, and forces a team wipe. It then repeats from
a deliberately damaged primary with a valid backup. Exact-state assertions run
before resumed simulation and immediately after the wipe; network snapshots,
clean exits, file preservation and separate single-player sentinels are checked
by the harness. This is persistence evidence, not a completed campaign visit.

On 2026-10-23, the complete public-invite run
`travel-test-0abfe07a7db0413db758682defa42cd9` passed all three phases: three-visit
travel/rejoin/save, fresh-process primary resume/wipe, and fresh-process backup
resume/wipe. The school door remained opened, two cached visits survived, both
clients reported health 83/47, and the native assertions restored the complete
saved players/resources/character/world/NPC state exactly. Dead-team save attempts
were rejected; primary/backup and single-player sentinels remained unchanged.
Both final views were inspected, showing Blue and Red correctly with the saved
health. All clients exited cleanly. Evidence is the folder's `result.json`,
`resume-result.json`, `backup-result.json`, histories, native logs and captures.
The full Rust suite passes 733 tests with one pre-existing ignored test
(`private/coop-checkpoint-tests.log`); the debug build passes
(`private/coop-checkpoint-build.log`). No campaign-matrix row is promoted by this fixture.

A native Anode menu pass then opened Co-op from the single-player village ESC
menu, selected Host → Continue co-op, and restored the saved school with health
83/47 and both cached visits. Clicking Save checkpoint displayed success and
wrote the fresh session epoch to the co-op file; the single-player quick-save
sentinel stayed unchanged. Leave co-op returned to the host chooser and the app
exited with code 0. The same evidence folder contains `menu-result.json`,
`host/coop-continue-menu.png` and `host/coop-menu.png`. No owned game windows or
tunnel helpers remained afterward.

## Windowed movement and combat slice (development only)

The main menu and single-player ESC menu now have a **Co-op** button (keyboard
shortcut C), with **Host** and **Join**. Host creates an internet invite; Copy
invite puts the complete link on the clipboard. Join accepts that link with
Ctrl+V or Paste link. Invalid links stay in the form with an explanation.
Hosting/connecting can be cancelled. Leaving or a connection error returns to
the co-op screen. A suspended single-player session stays in memory and can be
resumed afterward. Host starts at the chapter entrance when there is no co-op
checkpoint. With a checkpoint, it offers **Continue co-op** or **Start chapter**.
Continue restores both players and the shared world and creates a new invite;
the guest joins normally and receives the host's saved Red Alice.

The connected ESC panel has Resume, Save checkpoint (host), Copy invite (host),
and Leave co-op. It is
drawn once per frame, so it does not pause simulation or network updates. An
incompatible guest releases its slot while the host keeps the invite available.
`tools/run_coop_pair.ps1 -Data <original-base-folder> -Menu` opens two independent
main menus to exercise this flow using the public internet connection.

On 2026-10-23, native Anode tests exercised Host from the main menu and Join
from a paused single-player ESC menu, using the copied public invite. The guest
left and rejoined using the retained link after moving. With the host's menu
open, Red moved 105.91 units while Blue stayed still. Ending the host session
returned the guest to the Join screen. Both game processes exited with code 0;
no owned game windows or tunnel helper remained.

The single-player quick save stayed byte-for-byte unchanged during co-op.
After returning to the original school chapter, a newly written quick save
confirmed the same position, health and inventory. This is evidence for session
isolation, not for restoring a shared co-op campaign checkpoint.

The first menu pass also checked invalid links, cancelling Host, and rejection
of an older build followed by a valid join. It exposed two problems, both fixed
and retested: accumulated gameplay text could alter the retained invite, and a
Windows reader holding the optional diagnostic file could end gameplay. Text
input is now cleared at the gameplay/menu boundary; diagnostic writes are best
effort and the live report is removed on exit. The final public run continued
through an explicit three-second read-only lock of that diagnostic file.
Error messages now have their own readable panel.

The menu integration passed 696 Rust tests with one unrelated ignored test.
After the fixes above, all 28 focused co-op tests and the native public menu flow
passed. The automated two-window loopback movement check also passed with the
new menu and diagnostic cleanup in place. Original single-player save files outside the isolated test profile were
not used or changed.

`--coop-host <private-invite-file>` and `--coop-join <private-invite-file>` now open
two actual game clients. Hosting uses the public internet tunnel by default;
`--coop-local` after the host argument selects loopback for local diagnostics.
Both still require the normal `--data`, `--profile`, and optional `--map` arguments.
F8 copies the host invite. ESC opens a small personal overlay; Q leaves the session.
The world continues when a window loses focus and its controls become neutral.

The host advances two independent movement bodies at 120 Hz and sends validated
snapshots at up to 20 Hz. The guest renders the received state; it cannot decide
positions or health. Each client follows its own player, with camera collision and
guest-side visual smoothing. Alice's blue-fabric pixels in the known clothing
textures are recolored for player two at load time, including packed FTX textures.

`world.rs` integrates individual health, falling/liquid damage, the 10-second
spectating policy, safe nearby respawn at 25 health, and a two-body entrance checkpoint
for this slice. Its tests exercise movement independence, death/respawn/wipe and
refusing a spawn over a drop. This is not yet the campaign checkpoint system.

Run `python tools/test_coop_play.py --data <original-base-folder>` in a desktop
session (prefer Anode). It creates independent profiles and two rendered clients,
uses scripted controls through the same session and simulation as normal controls,
checks that both bodies moved and both clients agree, verifies separate cameras,
captures both views and closes its processes/helpers. Add `--local` for loopback.
The test never writes campaign saves or distributes original assets. Its scripted
controls are enabled only by the explicit development flag `--coop-drive`.

Add `--death --death-first red` or `--death --death-first blue` for the staged
guard recovery check. Ordinary movement makes the chosen Alice the nearest target;
the real guard supplies all damage. The survivor moves during the countdown, then
the pair continues until a wipe. The harness records both clients' death clocks,
spawn ticks, camera subjects and native guard state, and checks the ten-second
delay, 25-health nearby respawn, moving spectator camera and shared entrance reset.
Use `--delay-ms 40 --jitter-ms 20` to add outbound network delay per sender.
Both rendered recovery scenarios still cover only the school encounter.

Add `--rejoin` to either death scenario to terminate the guest process during
that player's real countdown, then reconnect with a fresh process and the same
profile/invite. The host must hold its world and life clocks while disconnected.
The fresh guest must inherit the remaining countdown, both personal inventories,
health and body identities, and the correct spectator camera. The ordinary
encounter then continues through the exact 1,200-active-tick revival at 25 health
and a shared checkpoint wipe. Both roles are required: Red dead, and Blue dead
with the surviving Red reconnecting. This check does not credit public joining
when `--local` is supplied. All receipts and the exact Python fixture stay private.

Both death/rejoin cases are available in one automated report:
`python tools/test_coop_regressions.py --exe target/release/looking-glass.exe --data <original-base-folder> --only death-rejoin-blue death-rejoin-red`.
The runner checks build identity, retains failed reconnect snapshots for diagnosis,
and records each death/respawn/checkpoint tick and remaining countdown. Public
invites and 40/20 ms delay/jitter are the defaults. Captures stay private and are
not inspected; the assertions use state and camera receipts.

The Blue-first development driver walks around the stationary Red using observed
waypoint arrivals. Its old straight approach stopped at the player contact radius
and could leave Red as the first victim. The route uses ordinary movement inputs;
player collision, enemy targeting, health and damage rules remain unchanged.

Snapshots identify each new body by its host spawn tick. Guest smoothing and
camera collision history reset when the watched player changes or respawns,
including short-distance respawns and checkpoint resets. Successful placement,
health and the living state are published together. Co-op draws its own spectator
countdown without the single-player retry panel; single-player keeps that panel.

On 2026-10-23, a rendered local recovery run killed Blue first at host tick 2006
and respawned her at tick 3206 with 25 health, 48 units from Red. Blue's camera
followed Red moving during the countdown and switched back at recovery. Both
clients reported the same respawn and the same entrance reset at tick 4208,
including restored guard health, position and action state. Captures confirmed the
spectator view no longer contains the single-player retry panel. Evidence:
`private/coop/play-test-26fd3e2ac8944ab2b33ed7227fdb0d27`.

The public internet recovery check added 40 ms delay and 0-20 ms jitter per sender.
Red died at tick 2077 and respawned at tick 3277 with 25 health, 48 units from Blue.
Her camera followed Blue moving during the countdown. Both clients agreed on the
entrance reset at tick 4261 and the guest acknowledged 1098 inputs. Both processes
exited successfully and the owned tunnel stopped. Evidence:
`private/coop/play-test-55aa3fe401e24789a3fd6f1f5a9268dd`.
The reverse order also passed over the public connection with the same added
delay/jitter: Blue died at tick 2004, respawned at tick 3204 with 25 health and
48-unit separation, and both clients restored the entrance at tick 4213. This
also exercises delayed guest movement while the host spectates. Evidence:
`private/coop/play-test-65f645a093ca4b0b98770c097ceff993`. No game windows or owned
tunnel helper remained after the tests.
The recovery revision passed 709 Rust tests (one unrelated ignored test), plus the
19 source-package and portable/Windows setup tests. The reset tested here restores
the entrance bodies and native guard, not the full campaign world or a saved puzzle.

On 2026-10-23 the first loopback rendered test passed: blue moved 207.14 units and
red 112.97 units (limited by the wall), both clients agreed within one unit, and
the guest acknowledged 256 control messages. The full suite passed 694 tests with
one unrelated test ignored. That first capture exposed packed-texture recoloring
and overlay spacing errors, which were corrected and visually checked in later runs.

The public internet movement run subsequently passed, including independent
cameras, matching authoritative positions, acknowledged guest input, captures and
clean process/helper shutdown. The captures show blue and red clothing together.
An earlier public run disconnected during play. The transport now tolerates bounded
bursts between render frames by pausing reads rather than disconnecting immediately;
the new burst regression and all 27 focused co-op tests pass, including the existing
persistent slow-reader limit. Keep the failed run in the private evidence rather
than treating the first internet attempt as a pass.

Real Anode keyboard/focus testing also passed: Red moved 107.47 units while Blue
stayed still, then Blue moved 104.00 units while Red stayed still. Both clients left
through ESC then Q; the interactive launcher exited successfully with no game
windows left behind. `tools/run_coop_pair.ps1 -Data <original-base-folder>` launches
that interactive pair; add `-Local` for loopback. It creates independent profiles,
waits for the invite, starts the guest and retains private diagnostics.

The final local rendered check passed after the sleeve recoloring and close-camera
visibility fixes. Both clients agreed on positions; Blue moved 208.05 units and Red
112.97 units. The inspected capture shows red sleeves/dress, blue player one,
unchanged skin/apron/hair/boots, and separate readable overlays. Both processes
exited successfully and no game windows remained. This final local run validates
the visual changes after the successful public internet run above.

The native combat pipeline now advances both real weapon implementations and a
single enemy cast on the host. Guest controls can select owned toys and attack;
cumulative button counters preserve short clicks across network send intervals.
Damage, Will, map pickups and enemy essence drops are personal. Each window draws
its own HUD and remaining pickups, and receives the same enemy and weapon state.
Death decisions run once after both players' combat damage resolves. A wipe resets
this slice's entrance bodies, native NPCs and character state; this remains separate
from a complete campaign checkpoint.

Combat snapshots include full animation poses, compressed with bounded zlib packets.
Both compressed and decoded size are capped at 8 MiB; truncated, oversized and trailing
data are rejected. Incoming NPC state must match the loaded cast; model indices stay
local. Character restoration currently uses the save validation/presentation path,
so smoother guest animation and effect continuity still need work.

Add `--combat` to `tools/test_coop_play.py` to exercise a staged school guard using
both players' actual Blade attacks. Only the host receives `--combat-preview`; the
visit handshake tells the guest to load the matching encounter. The harness checks
personal damage during the fight (essence can heal it afterward), attacks by both
players, shared defeat, matching positions, separate initial cameras and clean exits.
The driver first walks Blue to a clear side approach, then waits until both
players are in Blade range before attacking. This keeps the fixture compatible
with player collision and prevents Red from finishing the fight before Blue
arrives. Real guard damage, both attacks and personal health remain required.
For an interactive staged fight, add `-Combat` to `tools/run_coop_pair.ps1` with
the default school map and without `-Menu`.

On 2026-10-23, the rendered staged fight passed both loopback and public internet
tests using two independent processes/profiles and compressed combat snapshots.
Blue remained at 100 health, Red reached 64, both performed two real Blade attacks,
and both clients saw the same guard defeated. Public guest input reached sequence
411; final body positions matched within one unit. Both processes exited successfully
and the owned helper stopped. Captures were visually inspected. Private evidence:
`play-test-b1505039eeb04cfea0d87a16c5c75d09` (local) and
`play-test-ebe842687f2340568aee5b2d0941cc30` (internet), under `private/coop/`.

Earlier unsuccessful runs are retained too: one fighter could not reach the guard,
then both fighters interrupted it before it dealt damage. The final driver waits
for the guard's approach before attacking. The camera assertion also now accounts
for the school back wall retracting two separate camera arms to nearby origins.
These test corrections are not evidence of a change to normal enemy difficulty.

The full Rust suite passed 708 tests (one unrelated ignored test) at both default
concurrency and eight workers. Two earlier default-concurrency runs timed out while
waiting for an initial transport connection; focused and subsequent full runs passed.
Caller locations and connection-state failures are now reported for diagnosis. The
intermittent timeout's cause is not established; retain its private logs rather than
claiming those earlier runs fixed it. The later delayed-upgrade reproduction and
socket-mode fix are recorded in the live-world section above.
Single-player and two-player headless school combat checks
also pass at 30, 60 and 144 Hz.

After combat integration, the ordinary local movement regression also passed:
Blue moved 208.07 units and Red 112.97, with matching body positions and independent
cameras. Evidence is `play-test-98a3ab7d87454f7bb2cba07530c75ec1`. All owned game
processes and the tunnel helper were gone after these checks. Source packaging
tests (7), portable setup tests (6) and Windows setup tests (6) passed as well.

Still missing from this slice: two-demon targeting, co-op audio, remaining scripted enemy/world
controllers, moving supports beyond the school/village adapters, remaining cutscenes, full campaign progression and checkpoint validation,
prediction/latency tuning and campaign validation. Do not use the movement or staged
combat test as evidence of completing any campaign visit.

## Village integration and traversal

The host now advances trigger-owned encounters with the same two-player target
roster as placed NPCs. Their activation, damage, defeat and personal loot join the
shared simulation. Explicit conversations, the actual hallway Blade pickup scene
and shootable world switches enter the same scene-ownership path. Village machinery
advances once for both bodies using the common atomic carry function. The village's
derived hidden/relocated cast is rebuilt after snapshots and checkpoint restoration,
including on the guest; guests do not advance the story or enemy clocks.

An earlier `--coop-village-check` ran the production native arena from an unarmed
village entrance to its authored Pandemonium exit, with each first-arrival order.
Both collected their own actual Blade pickup and completed the six scenes. It also
counted two voice-only lines as cinematic handoffs; the correction and current
six-reunion results are recorded below. Those runs took 22,512 fixed ticks, ending with
the leader at 70 Sanity and the partner at 100, and performs no deaths, checkpoint
resets, test resource grants or test teleports. Only production scenes relocate players.
This is a single-process, two-body traversal, not a separate-process network test
or, by itself, a passed campaign-matrix row. Later network/death/rejoin and combat
evidence completes that row. That earlier native repeat passed with build identity
`06ad4947fcada8789a50aa239b059306b4453fbd2328cf4459ff1f041dccd5c5` in
`private/coop/village-route-2/coop-village/`, including per-reunion captures.
Blue's third/fifth handoffs and Red's climb handoff were visually inspected.
The initial run remains in `private/coop/village-route-1/coop-village/`.
All 734 Rust tests pass, with one pre-existing ignored test
(`private/coop-village-tests3.log`).

The same build also passes the public two-process checkpoint regression with
40 ms delay and 20 ms jitter per sender: travel/rejoin, exact restart/team-wipe
restoration and corrupt-primary backup recovery. Both final views were inspected;
all clients/helpers exited. Evidence:
`private/coop/travel-test-2e029f8371424a16884acb1619a3bbd0/` and
`private/coop-village-checkpoint.log`. This fixture still uses staged school travel,
not a networked village playthrough.

The new `--coop-world-check` village fixture compares 960 shared machinery ticks
against single-player poses and verifies paused clocks and serialized continuation.
It stages both bodies at a clear entrance; it does not claim riding every mechanism.
The original roof/sawmill rider probes found no clear standing positions inside the
map's collision boundaries and are retained in `private/coop-village-world1.log`
through `world3.log`, not counted as successful ride tests. Atomic rider and blocker
behavior remains covered by `moving::tests` and the authored school lift fixtures.
The corrected world fixture passes in `private/coop-village-world4.log`.

The single-player village route remains exactly 17,790 ticks, six jumps, two throws,
zero combat damage and 100 Sanity, matching the frozen pre-change executable. The
single-player machinery and six-scene cinematic checks also pass. Evidence:
`private/coop-village-before-*.log` and `private/coop-village-after-*.log`.

Subsequent separate-client traversal exposed an incorrect distinction in the shared
story clock: the two voice-only Cheshire lines froze an airborne player and caused
an extra reunion. Single-player allows exploration during those lines. Co-op now
advances ordinary speech alongside movement, pauses attacks/enemies while listening,
and reserves single-Alice control and reunion for controller-owned cutscenes. Either
living player can advance ordinary speech; that E press is consumed by dialogue.
Actual cutscene completion also clears grounded running momentum. Automatic saves
continue waiting for speech to finish before attempting a checkpoint.

The corrected native traversal passes both first-arrival orders in 22,297 ticks,
with six cinematic reunions, 35 ticks of observed movement during speech, both
actual Blade pickups and 100 Sanity for both players. Evidence:
`private/coop/village-route-4/coop-village/`, build
`0eb413f6731105a82780dc10d716c793fc04b3d7c3931d11391cbf6e9de9c2ee`.
Red's final cinematic reunion was visually inspected. The school/world fixture
passes in `private/coop-network-speech-world.log`; the unchanged 17,790-tick
single-player route passes in `private/coop-network-speech-singleplayer.log`.
All 734 Rust tests pass (one pre-existing ignored), recorded in
`private/coop-network-route-tests4.log`.

Failed development runs remain in `private/coop-network-village-local1.log` through
`local5.log`. The first attempts exposed driver timing/input differences; later ones
exposed the speech freeze above. The fifth run reached Pandemonium with both clients,
but failed its speech-movement evidence assertion because the driver skipped the
short lines between diagnostic samples. It is not recorded as a passed network test.
The driver now lets ordinary speech play while continuing movement.

The first public Red-leading run (`private/coop-network-village-internet-red1.log`)
also found an interactive client bug: the guest fed an unused local input latch,
which could reject the ninth accumulated button edge. Only the host samples that
latch. Guests now send their cumulative actions directly through the session,
without also filling a second queue that is never consumed. The host's bounded
input validation remains in place.
The repeat in `private/coop-network-village-internet-red2.log` ended on the
driver's approach ramp before reaching the earlier failure; it does not verify
the input fix. The harness now walks that approach through positions recorded by
the single-player route. Full delayed traversal is still pending. The current
build `028486987c95dd8d2ef21a4a148622aa8e9644aeb51fa2bfb7a2ba3f7113f210`
passes all 734 Rust tests (one ignored) in `private/coop-network-route-tests6.log`.

Later public runs separated test-driver failures from game failures. Guest plans
must time their input segments on the local clock; using received simulation ticks
turned network jitter into uneven key holds. Comparing a just-sent segment with
a delayed position also restarted valid plans and duplicated movement. The driver
now waits for its guest sequence to finish and the body to settle before replanning.
The host still checks each segment against its directly observed simulation.
These changes apply only to the explicitly enabled route driver.

The fifth public Red-leading run reached Pandemonium with both players at 100 Sanity,
but failed final verification: scene labels were sampled before the host step while
ownership and bodies were recorded after it. A completed scene could therefore appear
with no owner. Diagnostics now observe all those fields after simulation, record
their frame identity, and the harness asserts that identity matches. The earlier
failed runs remain evidence of development findings, not successful full tests.
The updated build passes 734 Rust tests (one ignored) in
`private/coop-network-route-tests8.log`.

The sixth public run verified consistent diagnostic clocks and correct owners for
the first five scenes, then missed the late climb. Its single long input plan kept
steering after the first missed landing. The harness now follows separate goals at
the actual intermediate landings before continuing the climb, with the same public
transport and delay/jitter settings. The failed run is retained in
`private/coop/village-test-a1044142d0b04cdeb70771de3b6ebe64/`.
The seventh run cleared those landings with both players and recorded all six
correct scene owners, then missed the final descent. The harness now also walks
the supported approach to that drop before planning the landing. Evidence:
`private/coop/village-test-e5eb38fdf0b24efb869e3c71abd6936a/`.

The public Red-leading repeat now passes the full traversal with 40 ms injected
delay and 20 ms jitter per sender. Both independent clients reach Pandemonium at
100 Sanity. Each observes all six correctly owned scenes and six reunions, both
actual personal Blade pickups and 50 moving-speech sample pairs without a voice-only
warp. All 84 matching authoritative ticks agree. Final captures from both clients
were visually inspected; both processes exit cleanly and the tunnel helper stops.
Evidence: `private/coop/village-test-eb20704fbc3547b1ab7bf766699c54a8/`, build
`5cae1ad45d01ad0b9a6f04a8f9a209c9bbff3ff804697cde94dbb3fefe2f5114`, reproduced by
`tools/test_coop_village.py --first red --delay-ms 40 --jitter-ms 20 --data <base-folder>`.
The Blue-leading repeat passes on the same build and network settings in
`private/coop/village-test-064657842c4d466cb6e551e24be6bb98/`: both players finish
at 100 Sanity, all six scene owners/reunions and personal Blades pass, 81 matching
authoritative ticks agree, and ordinary speech includes 49 host/51 guest moving
sample pairs without a warp. Both processes and the helper exit cleanly. Thus both
network arrival orders are verified. At that checkpoint the death/rejoin matrix was
unfinished; the later recovery and combat evidence completes the row. The 19 synthetic
packaging/setup checks also pass.

Run the native check in Anode (it needs a graphics context), with an isolated profile:

```text
looking-glass.exe --data <base-folder> --coop-village-check --no-audio --profile private/coop/village-route
```

The separate-client village harness is `tools/test_coop_village.py`. Run it in
Anode after building, with `--first blue` and `--first red` in separate runs:

```text
python tools/test_coop_village.py --data <base-folder> --local --delay-ms 0 --jitter-ms 0 --first blue
python tools/test_coop_village.py --data <base-folder> --first red --delay-ms 40 --jitter-ms 20
```

Omit `--local` for the public invite path. Each process has its own profile and
receives only local navigation goals through `--coop-route-driver <command-file>`.
Its planner runs against its own observed world; movement, jumps, attacks and
dialogue inputs still use the normal client/session path. Commands expire after
three seconds without an updated local file. The driver never writes player
positions, inventory, health or shared world state. It is explicitly enabled for
testing and is absent from ordinary Host/Join play.

The harness retains scene captures, route plans and authoritative frame histories
under `private/coop/village-test-*/`, then removes the invite and closes only its
own processes. It rejects a death, checkpoint rollback, blocked body, missed scene,
wrong scene owner, missing personal Blade, divergent matching network frame or
missing authored transition. A clean traversal alone still does not complete the
village death/disconnect/checkpoint matrix.

`--rejoin` on the village harness closes the guest near the exit, starts a fresh
guest process using the retained invite, checks both players' health/inventories,
placements and completed scenes, then continues through the authored exit. Its
before/after reports and separate guest logs stay in the private fixture directory.

`tools/test_coop_village_recovery.py` exercises actual falls after both players
collect the Blade. It checks each dead player's camera while the survivor walks,
the exact 1,200-tick countdown, nearby 25-health revival and retained personal
pickups. It then sends both players over the drop and checks shared restoration
against the game's earlier saved checkpoint. This fixture ends at that restored
checkpoint; outgoing traversal and rejoining are checked separately.
Route-driver profiles now use the same automatic checkpoint conditions as ordinary
play. The older staged movement/combat fixtures still opt out of automatic saves.

```text
python tools/test_coop_village.py --data <base-folder> --first blue --rejoin --delay-ms 40 --jitter-ms 20
python tools/test_coop_village_recovery.py --data <base-folder> --first red --delay-ms 40 --jitter-ms 20
```

The first recovery attempt reached the late scene but its direct diagonal movement
hit a barrier before the drop. It timed out without a death and supplies no recovery
evidence (`private/coop/village-test-fc87e73fd03b436caa14dc5ece1de800/`). The revised
test follows the supported approach before walking off the unrailed edge.
The second attempt observed Red's exact 1,200-tick recovery, but the survivor's
8.55-unit walk was shorter than the camera-motion assertion required. The survivor
now walks farther along the supported ramp. The third attempt passed both individual
recoveries and observed a team wipe; its final resource assertion incorrectly
treated the empty current-visit loot ledger, created by the first resumed ordinary
tick, as a changed inventory. The comparison now models only that empty ledger
creation and still compares every resource, award and drop. These failed fixtures
are retained as `village-test-f6163c27d91f45fbb9b41f7b21436584` and
`village-test-3e4d85e9f83643cd99cf26bb6cd4585e` under `private/coop/`.

The fourth recovery run passes over the public invite path with 40 ms delay and
20 ms jitter per sender. Red dies at tick 30,757 and revives at 31,957; Blue dies
at 34,524 and revives at 35,724. Both clients agree on those exact ten-second
intervals, both moving spectator cameras, each 25-Sanity revival within 50 units
of the survivor, retained personal Blades and the survivor's unchanged health.
The later team wipe at tick 37,338 restores the actual automatic checkpoint saved
at tick 2,916: both original resources/placements and the earlier scene/hatch state.
Both restored-checkpoint views were inspected and both clients exit normally.
Evidence: `private/coop/village-test-db59178229354bdbb2011361cf51ca9d/`, build
`f6304f3344c2f6d0340da41879fb00c4c7d37ada0076df301d49ae75a49f331d`.

The first fresh-process village rejoin run preserves both players and completed
scenes, but its guest misses the final descent afterward. It is not a complete
traversal pass (`private/coop/village-test-08f40261814744a28eac815202f0d736/`).
The test now follows the supported upper ledge farther south before crossing
to the lower landing, avoiding a long diagonal drop that grazed its north edge.
The repeat passes the complete Blue-leading traversal, fresh guest rejoin and
authored transition into Pandemonium with the same public delay/jitter settings.
It retains both Blades and completed scenes, all six scene owners/reunions, and
matches both players' authoritative state at 63 sampled ticks. Blue finishes at
100 Sanity and Red at 92; there are no deaths, respawns or checkpoint rollbacks.
Both final views were inspected; both clients and the tunnel helper exit cleanly.
The retained before/after rejoin reports also agree on Will, equipped weapon,
school items, staff components and difficulty for both players in both clients.
Evidence: `private/coop/village-test-49531ea8bd3f460da591f2be6a8c4465/`, on the
same `f6304f3344c2f6d0340da41879fb00c4c7d37ada0076df301d49ae75a49f331d` build.
The source change and harnesses also pass 734 Rust tests (one pre-existing ignored),
19 packaging/setup checks and Python compilation. The later focused checks below
complete the village's consumable/enemy evidence; 37 visits still prevent claiming
full campaign support.

### Public encounter and personal-consumable checks

`tools/test_coop_village_combat.py` and `tools/test_coop_pandemonium_combat.py` use
normal starts, independent processes/profiles and ordinary walking/weapon inputs.
They approach a real guard with attacks disabled, verify that it selects/damages
the nearer visible Alice, swap the players and repeat, defeat it with the Blade,
then observe each player's separate healing pickup. They default to the public
invite connection with 40 ms delay and 20 ms jitter per sender. For example:

```powershell
python tools/test_coop_village_combat.py --data C:\Alice\base --first blue
python tools/test_coop_pandemonium_combat.py --data C:\Alice\base --first red
```

Both pass in Anode on build
`fe1f5fd905c8262e13dc3bae6e86dfef7a57752504c84d2575e52c1d87bd7a0b`:

- Village: `private/coop/village-combat-44b591c1f47d453aa0afd3b9514fb7df/`.
  Both checked guard hits deal 12 only to the selected player; the placed shard
  independently heals each Alice from 88 to 100. All 3,986 comparable sent/restored
  states and 146 coincident full reports agree. Both attack views, the guest's
  retained pickup after Blue collects, and the final healed view were inspected.
- Pandemonium: `private/coop/pandemonium-combat-ac557b2cbaa240d4a61b04df68220399/`.
  The normal rope/cart lead to the first club guard; Red and then Blue are selected
  and take the checked hits while the distant partner remains unchanged. Its drop
  heals Blue 57.733 to 72.733 and then Red 87.667 to 100 independently. All 2,438
  comparable sent/restored states and six coincident full reports agree. Both attack
  views and the guest pickup view were inspected. Later enemies/the exit are outside
  this focused fixture; subsequent full-route repeats complete that coverage.

The opt-in route driver records the guard's host-selected target, distances and
line of sight. Guests do not run AI to produce a second target choice. A bounded
64-entry SHA-256 history records players, lives, checkpoint generation and
`Encounters` state at actual host send/guest restore points. The harness compares
these exact ticks regardless of the two local report clocks; this does not claim
coverage of other NPC/controller fields. It also compares full encounter/player
records when local report ticks coincide. These observations never enter the
input protocol or alter game state. Regular clients without the driver keep no
such history. All 740 Rust tests pass (one pre-existing ignored), and the normal
executable builds successfully.

`tools/check_coop_rewards.py` independently checks both clients' retained histories
for separated collection and healing, unchanged partner resources and retained
personal enemy drops. Explicit `--map-pickup` IDs also check placed consumables.
It verifies the earlier Blue-first and Red-first Pandemonium full-route histories
on build `ec4ccf71f5a634f1c8c03ccbc6924fa3492e9fac1421dd813cd4937c06118a48`:
`private/coop-pandemonium-personal-rewards1.json` and
`private/coop-pandemonium-personal-rewards-red1.json`. Negative checks reject absent
healing, shared removal of enemy drops, and missing guest observations.

Development trials are retained as failures: the first village approach stopped
outside melee range (`village-combat-8271af180b1347ad96aa67325be8c4bc`); its next
drop chase entered the damaging street (`village-combat-fa63edb33a6f497b9619dbb479222605`).
The successful test uses the supported route's placed shard. The first Pandemonium
trial completed its gameplay checks but had only three coincident local report
ticks (`pandemonium-combat-d99e90f584e24341aa3486dabf794c3d`), motivating the bounded
send/restore observations. No health, inventory or enemy behavior was changed to
make these tests pass. Successful clients/helpers close normally and remove invites.

The full Pandemonium route then passes again in both arrival orders on `fe1f5fd9`,
with the same public delay/jitter settings. Blue's run also leaves/rejoins a fresh
guest before the final airship. All seven encounter actors are included in the
comparison, including six combat deaths and both encountered Diamond guards'
projectiles. Blue's run matches 5,997 sent/restored states and 31 local reports;
Red's matches 6,362 and 65. All four scene owners, three safe reunions, delayed
cart boarding, shared rope, retained personal Cards and independently collected
healing rewards pass. Neither run has a death, respawn or wipe. Both arrive in
Fortress alive (Blue-led 46.667/25.134 Sanity; Red-led 30.001/45.201).
Evidence: `private/coop/pandemonium-test-94511445d6bc41e9a4e92a1a50ec4f18/` and
`private/coop/pandemonium-test-c3449c00e8bb41d0b6c7f39bb4729a98/`; each retains an
`encounter-coverage.json` summary. Rope, boarding-wait and initial Fortress
cinematic views were inspected. Both clients/helpers exit and invites are removed.
Combined with the earlier recovery/wipe evidence, Pandemonium is the second passed
campaign row. The initial Fortress arrival images do not verify that next visit.

## Fortress moving-room preparation

The first Fortress controller now advances its moving room once for all living
co-op bodies. It calculates every rider's proposed position before committing;
an obstruction affecting either player holds both riders and the shared room
clock until it clears. Original single-Alice scenes still use their original
controller path. Moving trigger volumes are refreshed with the same poses as in
single player. The split room is a first-arrival environmental trap, not a shared
passenger lift, so its authored trigger does not gain an all-aboard wait.

`--coop-fortress-check` stages two bodies on the real `split_middle` geometry.
On build `bac55928de0966ab179553addb99d0c14888d14eab63f352fb34c906564d374f`,
30/60/144 Hz checks carry both 220 units, preserve pause and serialized restore,
and exactly match one-player carry. Separate obstructions of either rider stop
both and resume after removal. Evidence: `private/coop-fortress-support-native1.log`.
This is a staged support check, not a complete two-client campaign traversal.

Both single-player Fortress routes match the previous `fe1f5fd9` build exactly:
arrival takes 16,020 ticks, 10 jumps, one Blade throw and 22 Cards; return takes
5,343 ticks, three jumps, one Blade throw and 41 Cards. Both finish at 100 Sanity
with no combat damage, flight or recovery. Watched/skipped cinematic outcomes,
pause, mid-scene restore, legacy saves and the one-shot ambush also pass.
Evidence: `private/coop-fortress-sp-comparison1.json`,
`private/coop-fortress-sp-route1.log`, `private/coop-fortress-sp-cinema1.log`.
All 740 Rust tests pass (one pre-existing ignored), and the normal build succeeds.
Full network routes, map-specific recovery, and the later return visit remain
separate requirements; the first Blue-leading network result follows below.
The campaign count remains two of 39 visits.

The separate-client first-visit probe is under development:

```text
python tools/test_coop_fortress.py --data C:\Alice\base --first blue --rejoin
python tools/test_coop_fortress.py --data C:\Alice\base --first red
```

It uses the public invite with 40 ms delay and 20 ms jitter per sender by default.
Each process receives ordinary local goals and controls; the harness reads the
Fortress scene/state, requires independent portal contacts and the completed
split, compares authoritative player/enemy snapshots, and checks optional fresh
guest rejoin before the authored exit. The opt-in driver can now send one ordinary
jump per command and a run modifier without inventing a grounded destination for
an airborne doorway. Neither control changes the network input protocol. Its
legacy-command/rope regressions pass and the normal build succeeds on
`fda9260961ca3342961c3a50c0d71e3ff8a23d820466e92efa3c009cff63cff2`.

Trials 1–8 are failures, not campaign credit. Long timed paths missed landings,
the guest overshot an upper ledge, and walking a gap fell below its sloping landing.
The route now uses supported waypoints, a landing farther onto the upper platform,
and received-position steering for the gap jump. The failed trial summary is
`private/coop-fortress-development-summary.json`; logs are
`private/coop-fortress-internet-blue1.log` through `blue8.log`.
Trials 3 and 4 were stopped after the failure was observed and retain final
observations/captures instead of full histories; the others retain full histories.
All eight have their owned clients/helpers stopped and invite files removed.
Creating `stop-test` inside a running Fortress probe's printed evidence folder
now stops it through its normal cleanup and preserves its partial history.

Trial 7 reached the last raised section with both players at 100/92 Sanity, but
its next navigation search timed out. Its partial checks match 5,114 sent/restored
player/enemy states and 87 full reports including Fortress state. Both clients
observe the Blue-owned entry scene, the reunion at tick 10,020, independent portal
contacts and a completed split. Both sloping-walkway views were inspected.
Evidence: `private/coop/fortress-test-c85881127e254ef2b5969aa41136dd02/partial-checks.json`.
Trial 8's direct jump landed both bodies, but continued exact-marker steering ran
the guest back off the edge. The controller now releases input after a real jump
lands near the destination at the right height, then verifies the body remains
grounded after the delayed inputs settle. These partial results do not verify an
exit, rejoin, recovery matrix or the later return visit.

The Blue-leading full route with fresh guest rejoin now passes on `fda92609`, in
`private/coop/fortress-test-cc6abe54e07e4f2fac11ff3621b1c75d/` (public internet,
40 ms delay + 20 ms jitter per sender). Both clients observe the original entry
scene owned by Blue (the tied automatic arrival), one safe reunion at tick 10,020,
their independent portal contacts, the completed split and the authored exit to
`fortress2` epoch 2. All 3,740 comparable sent/restored player/enemy states match;
50 coincident full reports also match, including Fortress controller state.
The guest exits and rejoins in a fresh process near the final doorway (ticks
24,817/24,901), preserving both players' resources, placements and completed
scene/machinery progress. Both finish alive at 85/80 Sanity with their Blade and
Cards, no respawn and no wipe. The sloping-walkway view and both final views were
inspected. Both clients and the helper close normally and the invite is removed.
The Red-leading route also passes on the same build and delay/jitter settings:
`private/coop/fortress-test-3cf5044c3af64e3e97ed3ef09d70f751/`.
It observes the same original arrival/reunion, independent portal contacts,
completed split and authored exit. All 3,870 comparable sent/restored states and
93 coincident full reports agree. Both arrive alive at 100 Sanity with their
weapons, without a respawn or wipe. The Red sloping-walkway and final arrival
views were inspected; clients/helper exit normally and the invite is removed.
These routes verify both first-visit orders and Blue-leading rejoin. Targeting,
consumables and recovery are checked separately below; the next level and later
Fortress return visit remain separate requirements.

The focused Fortress checks use the same independent profiles, public invite and
ordinary input driver:

```text
python tools/test_coop_fortress_combat.py --data C:\Alice\base --first red
python tools/test_coop_fortress_recovery.py --data C:\Alice\base --first red
```

The opt-in report now includes Boojum selection, visibility, distance and stable
encounter IDs (the two room Boojums have no authored names). It changes neither
normal gameplay nor the network protocol. The combat probe swaps the nearest
player, takes real scream damage, defeats both active Boojums with the Blade,
then checks an available personal healing drop. The recovery probe leaves the
survivor in the courtyard while the other enters the Boojum room, checks both
ten-second spectator/respawn roles, then sends both low-health players into the
fight to verify their actual automatic checkpoint. These checks supply no health,
position, inventory or shared-world writes.

The focused combat probe passes on build
`2271c7db46267916aa0d866bcc99db4b4d94fce2516014274027895145d58ab8` in
`private/coop/fortress-combat-e4a4260346de427c84f93e82e338f187/` (public internet,
40 ms delay + 20 ms jitter). The observed Boojum selects nearer Red, then Blue,
with both visible. The checked damage changes Red 100 to 93, then Blue 100 to 98,
leaving the farther partner unchanged at each check. Both active Boojums are
defeated by ordinary Blade attacks. The same reward heals Blue 87 to 100 while
Red stays at 85 and retains her copy, then heals Red to 100 without changing
Blue. An independent history check confirms separate healing/collection in both
clients. All 2,442 comparable sent/restored states and 78 coincident full enemy/
player reports match. The first two probe attempts are retained as failures:
the first retreated under a lintel that blocked throws; the second approached a
farther health drop and collected the nearer one on the way, becoming too healthy
to collect the chosen drop. Neither contributes a campaign pass.
The Blue damage view, both-player pickup view and final healed Red view were
inspected. Both clients and the helper exited normally; the invite was removed.

Fortress recovery uses inventory at the observed death as its preservation
reference. A real pickup during the preceding fight is allowed; respawning must
retain it. The earlier village and Pandemonium recovery histories still pass
that comparison for both roles in both clients (`private/coop-fortress-recovery-inventory-audit.json`).
The courtyard survivor route uses the ordinary jump planner across its gap.
The first two Fortress recovery attempts are retained as failures: one used the
pre-fight inventory reference, and the other left the survivor hanging from a
ledge after a fall-reset portal. The latter correctly deferred respawning until
a safe platform was available; it does not supply a successful timed recovery.

The complete recovery probe passes on the same `2271c7db` build and public
40 ms + 20 ms connection in
`private/coop/fortress-recovery-6388fe74d6414e87a48f45ee92ceda44/`.
Red dies at tick 15,887 and respawns at 17,087; Blue dies at 19,780 and returns at
20,980. Both have exactly 25 Sanity, stand 48 units from the survivor on clear
ground, retain their death-time inventory and regain their own camera after
following the moving partner. The survivor takes no damage during either check.
Both then die in the actual Boojum room. Tick 24,430 restores the automatic
checkpoint from tick 10,021, including personal resources, earlier placements,
scene/machinery state and the Boojums' original positions with no scream waves.
The first restored reports arrive 22/17 ticks after restoration; movement stays
within the bound derived from those ticks (Blue drifts 15.85/11.52 units, Red 0).
All 3,662 comparable sent/restored states and 88 coincident enemy/player reports
match. The spectator view, revived pair and both restored views were inspected;
both clients/helper close normally and the invite is removed.

Combined with the two full routes, fresh guest rejoin, native paired supports and
single-player regressions above, the first Fortress visit is now passed in the
campaign matrix. Its splitting room is a first-arrival environmental trap, not
an activated passenger lift; a shared boarding barrier is not applicable here.
The opt-in diagnostic build and both route-driver regression tests pass in
`private/coop-fortress-combat-build1.log` and
`private/coop-fortress-combat-driver-tests1.log`. Normal gameplay is unchanged by
the additional read-only Boojum report.

## Beyond the Wall co-op adapters

The shared Beyond controller now advances once per host tick and checks both
living bodies before committing a moving support. Either player can operate the
musical levers; the actual final operator owns the original single-Alice scene,
including when the partner also tries a lever on that tick. Note replay outputs
advance once for the world. Sound playback was added in the follow-up below.

The rolling bridge waits for both living players to reach the stable tiles before
rolling the rear tile away. The Rage pickup waits for both to board the lift,
starts its original transformation scene for the collector, and reunites the
partner on the lift before descent. The other player's copy remains available
and an ordinary jump from the lowered lift can collect it without replaying the
scene. The later cycling elevator waits while only part of the living team is
aboard, including during disembarkation, and cycles normally when empty. Its
saved boarding delay affects only the lift; other machinery keeps running. A
sole survivor does not wait for a dead partner.

Build `c17d9c2b1eedbd621fd56923648b8f38ec4a58bfd1ba1bae5ef39aac4c0e53c8`
passes `--coop-beyond-check` on the authored map geometry. It checks paired
48-unit descent at 30/60/144 Hz, single-player clock/carry parity, pause and JSON
restore, either obstructed rider stopping both carries and resuming after the
obstruction is removed, both boarding orders, saved waits and invalid-delay
rejection, both scene owners, frozen partners, lift reunion and separate pickups.
The native fixture uses staged placements and does not run combat or a complete
route. Evidence: `private/coop-beyond-adapter-build5.log` and
`private/coop-beyond-adapter-check5.log`.

Single-player machinery/cinematics and the power-up suite (including watched,
skipped and legacy-save Rage scenes) pass in `private/coop-beyond-sp-state5.log`
and `private/coop-beyond-sp-items5.log`. The baseline power-up run also passed.
The full single-player route already failed at arch step 12 on the unchanged
`2271c7db` executable. The final build reproduces its entire 192-line output
exactly, including that failure; this is not a route pass. The comparison is
recorded in `private/coop-beyond-sp-comparison5.json`, with both original logs.
Existing co-op school/world checks, the Fortress paired supports and all 39
entrance-only checks pass in `private/coop-beyond-world5.log`,
`private/coop-beyond-fortress5.log` and `private/coop-beyond-entrances5.log`.
The complete Rust suite passes: 740 passed, no failures, one existing ignored
test (`private/coop-beyond-adapter-tests5.log`).

This adapter milestone added no completed campaign row. Full Beyond traversal,
combat and recovery remain open; the focused sound/rejoin evidence follows below.
The overall estimate remains about 41%, with three of 39 visits fully verified.

## Co-op sound and dialogue

The host now publishes a bounded journal of numbered interaction, movement,
weapon, enemy and cinematic sound cues. Each client consumes new cues once,
even when snapshots repeat or intermediate snapshots are skipped. Old cues are
not replayed on joining, travel or checkpoint rollback. Continuous music and
loops are reconstructed from shared state; original story voices follow the
host's line and elapsed position, including cinematic speech holds. Each client
uses its own camera for spatial listening. The single-player/menu mixer is
paused while the co-op viewer owns playback.

The first delayed public sound fixture passed on build
`0944f2141e72b2011c2a77d06e90d71289b5dcd5ed3e1488cb4fe5b44369a9ba`:
`private/coop/beyond-audio-1dbe4744e4984a3394b41ce156a4aead/`.
It stages only the initial feet outside the musical demonstration trigger, then
uses ordinary movement and E inputs through two real clients. Red triggers the
original demonstration; Blue and Red each operate the reset lever. Both clients
receive exactly the same three numbered notes in each sequence. A fresh guest
process rejoins without replaying the completed demonstration, with resources
and puzzle progress intact. The fixture compares 675 actual sent/restored states
at 40 ms delay plus 20 ms jitter per sender and captures both rendered views.

All three app-only WAV recordings contain nonzero output. Cross-correlation
against locally read original note recordings also identifies all three notes
in order in the initial guest's recording (approximately 3.65, 4.73 and 5.75
seconds). This corroborates real mixer output beyond cue metadata; it is not a
subjective listening or full sound-fidelity assessment. Original assets and
recordings remain private. Anode's session has no isolated audio endpoint, so
this fixture used the authorized main desktop; its owned windows were closed.

The four focused journal/voice-cursor tests pass, including collapsed snapshots
at 30/60/144 Hz, join/travel/wipe/expiry boundaries, bounded bursts, malformed
wire fields and dialogue cursor changes. The complete suite on this build
passes with 744 tests, no failures and one existing ignored test. Native
Beyond co-op checks additionally verify the three-note reset order for either
operator. Single-player Beyond state/cinematics and existing co-op world/scene
checks pass. Evidence: `private/coop-audio-tests1.log`,
`private/coop-audio-all-tests1.log`, `private/coop-audio-beyond-native1.log`,
`private/coop-audio-sp-beyond1.log` and `private/coop-audio-world1.log`.
The original audio asset/clock check also passes: 336 referenced files decoded,
38 music cues and 900 ambient emitters; two original missing references are
reported (`alice/idle_move2.wav`, `icewand/icw_idle.wav`). The live musical
fixture reports no missing cues (`private/coop-audio-assets1.log`).

Final build `10aa2112129158e4810cf5856895fd6d32b2eac586afa20a29104acc3f293fc4`
also passes the full 744-test suite and native Beyond checks
(`private/coop-audio-all-tests2.log`, `private/coop-audio-beyond-native2.log`).
This follow-up corrects camera-directed stereo listening and clears reaction
gating/swim ambience on world restore. Its unskipped village opening test passes
in `private/coop/village-voice-0e430b62bac8415a8ecf762320d21940/`: both public
clients observe all six original lines in the same order, one voice start per
line, elapsed positions matching the host story, and held dialogue during the
original camera choreography. It compares 1,136 sent/restored states under the
same delay/jitter. Both 30-second app recordings contain the first Rabbit voice:
correlation with the locally read original is above 0.96. The recordings stop at
the capture limit; the full six-line sequence is verified by runtime observations
and playback logs, not a complete six-line acoustic comparison. The driver now
supports watching dialogue without generating its usual advance presses.

The final build also passes the musical fixture with Blue arriving first,
both reset operators and a fresh guest process:
`private/coop/beyond-audio-d97405d188c34927aa4e24529f05f29f/`
(`private/coop-audio-internet3.log`). It again verifies identical numbered notes,
no historical replay, nonzero output without missing cues, independent app
recordings and matching sent/restored state. Both runs close their owned windows
and remove their invite files. The initial rejected launch (audio capture lacked
the required frame limit) is retained in `private/coop-audio-internet1.log`;
the harness now supplies that limit before launch.

Reproduce the musical fixture with:

```text
python tools/test_coop_audio.py --data <base-folder> --first red
python tools/test_coop_voice.py --data <base-folder>
```

This is a staged musical demonstration/reset/rejoin check, not a full Beyond
route. Remote pain/pickup cue fidelity, menu-to-game audio handoff, rendered Rage
presentation and the remaining campaign traversal/combat/recovery checks still
need verification. No additional campaign visit or percentage point is credited.

## Beyond normal-entrance network route

`tools/test_coop_beyond.py` starts two independent profiles at the normal Beyond
entrance and uses ordinary movement, jumping, attacks and lever inputs. It reads
replicated machinery clocks to time the flipping corridor and moving stairs.
It rejects deaths, checkpoint resets, obstructed bodies and unplanned recovery
portals rather than treating those shortcuts as clean traversal. Each private
run keeps the exact harness source, observations, final reports and captures.

The explicit local test driver now accepts a bounded input-delay estimate for
braking from delayed observations and a moving-step target. These affect only
the driver's ordinary inputs; they do not change player physics, received state
or the host's authority. Other route fixtures keep their existing zero-delay
steering unless they opt in.

The fourth public run, build `496972c8`, reaches arch step 11 from the entrance
with 40 ms delay plus 20 ms jitter per sender. It crosses the flipping corridor,
solves the puzzle with alternating players, completes the rolling walkway and
observes all three Blue-owned scenes. Both players reach the arches with 100
Sanity; the minimum along the route is 96 for Blue and 100 for Red. No player
respawns, and 7,143 actual sent/restored network states agree. Evidence and both
rendered views are retained in
`private/coop/beyond-route-aa5a24b4065b48d7bb126d15146b0f9b/`.
The run stops at the next stair because the driver required a vertical gap below
90 units between steps whose gap is always 102.4. This is not a completed route.

Earlier failed attempts are retained: the delayed guest overshot a jump landing
(`1a1cbed4`), simultaneous automatic door presses prevented passage (`888da19b`),
and delayed walking overshot a platform edge into a recovery portal (`217a1088`).
The harness now anticipates braking, sends the leader through that narrow door
first, and aims farther inside the platform. The moving-step driver uses the
native route's 110-unit launch bound and descending upper-wave timing. The upper
exit uses the native route's running approach, and the final door waits for the
shuffle to settle before being approached. These test adjustments still require
the resulting full route to pass.

Two further repeats stopped at the gap after the third flipping platform. In
`beyond-route-d12f8fe4562c400fb1d66815c43a99a8`, the guest's static plan treated a
passing platform as permanent floor and never jumped. In
`beyond-route-f11fb0abdca14a369a9affd8dba05144`, an outer-harness run-up lasted
too long because it waited on periodic network reports. Both recovery portals
were rejected. The driver now times bounded run-ups alongside local input
sampling: host simulation ticks and the guest's local clock. A regression test
checks that stalled guest snapshots cannot lengthen the run-up. The corridor
gap now uses that ordinary running jump instead of a static-floor plan.

Reproduce with a debug build and locally installed game data:

```text
python tools/test_coop_beyond.py --data <base-folder> --first blue --rejoin
python tools/test_coop_beyond.py --data <base-folder> --first red --rejoin
```

For diagnosis, `--arches-only` stages the initial position at a verified point
outside the rising-room trigger and then supplies ordinary inputs through the
stairs. Its evidence folder and result explicitly identify it as a staged probe;
it cannot complete a campaign row or establish normal-entrance traversal.
`--corridor-only` similarly starts just before the corridor trigger. The two
probe flags are mutually exclusive.
That probe passes on build
`7eec625b6f796e7878748a243126ca79df900c889b7cc00a9f4e61e02f915424` in
`private/coop/beyond-arches-probe-0387775c95634254acd868903865bf8f/`:
both climb the steps, complete the upper running jump and use the authored
portal. There are 1,027 matching sent/restored states under the default delay
and jitter. Both rendered upper-door views were inspected. The first probe
(`20cac92e`) was rejected at startup because its proposed spawn was obstructed;
the successful probe uses a clear point recorded in the normal traversal.

The same build completes the whole corridor from the normal entrance in
`private/coop/beyond-route-3ccb48bd81f34998802d1600e37f7f42/`, with 4,516
matching sent/restored states and no respawns. That repeat stops before the
first musical note: the navigation landing radius left the operator outside
lever reach. The harness now finishes each lever approach with precise ordinary
walking before its single E press.
The next repeat (`beyond-route-4fd969389a574d3997680914aa974503`) clears the
corridor and all three levers, then stops when a Boojum's scream knocks Blue
off the spiral stairs into a recovery portal. The test had treated a nearby
enemy hidden by the stairs as safe. It now fights from the upper landing before
descending, allows nearby Boojums time to approach, and waits for existing
scream projectiles and knockback to settle before departing a combat stop.
The following full-route repeat (`beyond-route-d1833f4c4eca4f698d5a499781c15b5b`)
stops earlier, at another static-planner misstep toward `(2980,3456,-64)`.
The remaining corridor gaps now use explicit ordinary jumps too. The focused
corridor probe passes on the same `7eec625b` build in
`private/coop/beyond-corridor-probe-f26639b408ba470b82d9b3a6498ea8e7/`:
both traverse all flipping-platform gaps and take the authored portal, with
1,709 matching sent/restored states under 40/20 ms delay/jitter. Both rendered
portal views show 100 Sanity and were inspected. This initial placement is
staged. The subsequent normal-entrance traversal below covers the revised
combat stops and final shuffled-door exit.

The Blue-leading normal-entrance run on the same build reaches the authored
`fortress1_start2` return with both players alive in
`private/coop/beyond-route-3ea9b2df6c034cffb7c1b83a3df0ccc9/`.
It has 7,722 matching sent/restored states, all three Blue-owned route scenes,
frozen partners and safe reunions, a solved musical puzzle, walkway stage nine,
the correct final door and no respawns. Minimum Sanity is Blue 96 / Red 98;
both arrive in Fortress at 100. The original harness exits with an error after
the actual traversal: it scheduled its Beyond reconnect check after crossing
the outgoing trigger, then looked for Beyond state in the Fortress visit.
The retained observations pass the extracted `verify_route()` validator in
`audited-route-result.json`; `audited-post-transition-rejoin.json` independently
checks the actual fresh guest's preserved Fortress placement and resources.
This is audited traversal evidence, not a claim that the original harness
exited successfully. The fixture now reconnects at the safe pre-exit landing.

The corrected Red-leading full route exits successfully in
`private/coop/beyond-route-26dba8d47c88465abb0ab7cc4a1eb6bd/`, on the same build
and 40/20 ms public connection. All three scenes belong to Red, all reunions
and traversal invariants pass, and 7,885 sent/restored states match. A second
guest process joins at the safe pre-exit landing with personal resources,
placements and completed puzzles intact. Both reach `fortress1_start2` at
100 Sanity. The rendered guest stair/upper-door/return views were inspected,
including a live Anode observation. The host return capture is orange and
obscures the scene despite valid placement; the route assertions alone did not
detect it. The scoped camera correction and rendered repeat are recorded below.

The separate public combat check passes on `7eec625b` in
`private/coop/beyond-combat-04edc0990c244e69995a04acf6a9d34f/`.
The hallway guard chooses nearer visible Red, then nearer visible Blue. Each
checked hit removes 12 Sanity from its target without hurting the other player.
Both collect their own earned healing drop after ordinary Blade combat: Blue
heals 76 to 91 while Red stays at 88; Red then heals to 100 while Blue stays at
91. There are 505 matching actual sent/restored states and nine coincident full
enemy/player reports. Both target views and the final personal pickup view
were inspected. Reproduce with `tools/test_coop_beyond_combat.py --data <base>`.

Both combat deaths and the shared wipe pass on the same build/settings in
`private/coop/beyond-recovery-6ca7a13ffb634602bc613cff7d893097/`.
Red dies at tick 3,660 and returns at 4,860; Blue dies at 8,492 and returns at
9,692. Each spectates the moving survivor for exactly 1,200 ticks, returns at
25 Sanity about 48 units away, and preserves inventory. A subsequent team wipe
at tick 13,015 restores the actual earlier checkpoint's personal resources,
placement, enemy state and Beyond machinery. There are 1,943 matching
sent/restored states and 89 coincident full enemy/player reports. Spectator,
respawn and checkpoint views were inspected. Both fixtures use normal-entrance
movement and actual enemy damage; neither injects health or world state.
Reproduce recovery with `tools/test_coop_beyond_recovery.py --data <base>`.

The focused passenger probes in `tools/test_coop_beyond_transports.py` use the
same public connection and delay/jitter. `--probe rage` walks from the normal
entrance; `--probe bridge` explicitly stages only the initial bridge approach.
Both accept `--first blue` or `--first red`, and neither claims a full route.
On `7eec625b`, Rage passes with Red first in
`private/coop/beyond-rage-ffb2d9ac54324e4ca8b926673385d316/` (956 matching states)
and Blue first in `private/coop/beyond-rage-fdfee62536404a0d97d6c2633a33f0e6/`
(965). The first player waits for the partner, owns the original scene, then
both descend 48 units together. The other player collects an independent Rage
copy with an ordinary jump; the scene runs once. These state assertions pass,
but inspecting the middle of the scene revealed that its owning Alice was
hidden: the generic co-op cinematic drawing rule hid both gameplay bodies.
The renderer now draws the owner's existing authoritative Rage pose while
keeping the other player hidden. Other cinematic actor drawing is unchanged.
The corrected `cf86b004` public repeats pass with Red first in
`private/coop/beyond-rage-7d58e582d71842dfaae8f083f4998a27/` (936 matching states)
and Blue first in `private/coop/beyond-rage-ca85ba3508fb4818846294edf437d0a2/`
(960). Both clients' middle-scene views were inspected in each run: exactly the
owning Alice is visible in the original transformation pose, with the correct
clothing. All boarding, frozen-partner, descent and independent-copy assertions
also pass on this final build.

The bridge waits for a trailing player, shows the boarding notice, and then
advances once with both safely aboard in both orders. Red's run is
`private/coop/beyond-bridge-5cd7763586a34c51919d81c6feec49c6/` on `7eec625b`
(156 matching states). Blue's run is
`private/coop/beyond-bridge-07f7c0ef71a64c2aa9a796aeade41fbb/` on
`cf86b00492db900ea0f523d75c5bde56c763f11e7890fb1ef74775f1a0c1ad24`
(170). Waiting/boarding views were inspected. Initial probe failures remain
retained: a broad navigation plan could not reach the Rage lift, a walking
approach stopped at its raised lip, and the bridge's combat wait could not
reach a distant enemy across the gap. The final probe uses ordinary approach
points/jumps and keeps real combat active. A Blue bridge setup also started
its partner already on a safe tile; both orders now place the trailing role
on the same rear approach before checking the barrier.

The `cf86b004` build also fixes the obscured Fortress return view. Blue's camera
was inside the visible `portal_back1` volume, which had no camera collider.
That visible portal now blocks cameras without becoming a player obstacle or
changing save state. An independent real-geometry regression checks both
entrance positions at 30/60/144 Hz in `--fortress-cinematic-check`.
The original local two-client reproduction is
`private/coop/fortress-return-camera-0d47b1f15b4e4c0081c7f68b5679dddc/`;
the fixed repeat is `fortress-return-camera-579dd23381c84f05aa0c6937d36aec74/`.
Both fixed entrance views were inspected and are clear while stationary.
The existing watched/skipped/paused/resumed/legacy-save Fortress scene checks
and both native co-op Fortress/Beyond adapter suites also pass. Evidence:
`private/coop-fortress-camera-check7.log`, `private/coop-fortress-adapter-check7.log`
and `private/coop-beyond-adapter-check7.log`.

Final build `cf86b004` passes all 745 Rust tests (one existing ignored test),
debug/release builds and all seven source-package tests. The rendered clients
match the independently calculated source identity. Validation logs are
`private/coop-beyond-validation-tests7.log`, `private/coop-beyond-validation-build7.log`,
`private/coop-beyond-validation-release7.log`, `private/coop-beyond-source-tests7.log`
and `private/coop-beyond-source-identity7.json`. The source-only worktree audit
passes 765 files. No private captures, saves, game data or invite links are
included in the contribution.

Validation for this source build: 745 Rust tests pass with one existing ignored
test, both debug and release builds succeed, and all seven source-package tests
pass. The source identity matches the executable used by the passing stair
probe. Evidence: `private/coop-beyond-route-tests5.log`,
`private/coop-beyond-route-build5.log`, `private/coop-beyond-route-release5.log`,
`private/coop-beyond-source-tests4.log` and `private/coop-beyond-source-identity5.json`.

Internet joining and 40/20 ms delay/jitter are the defaults. Both full traversal
orders, combat/resources and death/wipe have the evidence above.

Cycling-elevator rendered checks now pass in both boarding orders on `cf86b004`:
`private/coop/beyond-elevator-d57b738905834aa6a1ab2d1c730c3c59/` (Red, 811 matching
sent/restored states) and `private/coop/beyond-elevator-da651fab20844150a026e2a640f1ebc2/`
(Blue, 775). `--probe elevator` stages the initial upper return landing inside
the authored activation trigger, then walks down, boards, rides and dismounts
through ordinary controls. It verifies two seconds of boarding and dismounting
wait in each client's own reports, continuing world time, paired ascent, safe
running jumps to the Normal-difficulty outer ledges and the empty lift's resumed
clock. Waiting/dismount/final views were inspected in both orders. Earlier
fixture failures retain a wall-bound approach, an incorrect Easy-only landing,
and a host-only observation deadline that ended before the delayed guest's full
wait. These probes grant transport evidence, not standalone full-route credit.

The additional `--probe late-rage` exposed a real freeze on `cf86b004` in
`private/coop/beyond-late-rage-e16f3f79bbe141f387b6d401f0030397/`: collecting the
box after cycling began changed the lift's phase, obstructed both bodies and
left the original scene at time zero. The stopped run retains those reports.
Build `ef3db23edd407fcac578bb0676a3af94c984bdd9882aab117864395ccbd73989` derives the
cycle's initial descent from whether Rage preceded the return trigger. A later
pickup cannot restart that descent or shift the cycle. Existing saved timestamps
suffice; no save field or format changes. The original early pickup is unchanged.
The phase regression, all 746 Rust tests (one existing ignored), and native
co-op checks pass. Both owners watch and skip the late scene with pause/save/
restore, frozen partners, clear bodies and a supported paired reunion. Original
Rage watch/skip/save/legacy cases on all difficulties and Beyond's other world/
cinematic regressions also pass. Logs: `private/coop-beyond-validation-tests8.log`,
`private/coop-beyond-adapter-check8.log`, `private/coop-beyond-items-check8.log`
and `private/coop-beyond-world-check8.log`.
Both corrected late-pickup public repeats pass under 40/20 ms delay/jitter:
`private/coop/beyond-late-rage-510a52db30ac4a858ec444cd09c10e84/` with Red first
(932 matching states), and `private/coop/beyond-late-rage-0dedcf3a803a4016babffde8b517a11c/`
with Blue first (927). Both check the advancing original scene, a frozen hidden
partner, one reunion and grounded bodies. The middle-scene owner and paired
reunion views were inspected in each run. The rendered clients match the source
identity in `private/coop-beyond-source-identity8.json`. Debug/release builds and
seven source-package tests pass (`private/coop-beyond-validation-build8.log`,
`private/coop-beyond-validation-release8.log`, `private/coop-beyond-source-tests8.log`).
Together with the earlier full routes, combat, recovery and rejoin evidence,
these checks complete Beyond's row: four of 39 visits are fully verified.
Overall weighted credit is about 41.05%, still reported as 41%.

The next visit's traversal now passes in both orders on the same `ef3db23e`
build. `tools/test_coop_fortress_return.py --data <base> --first red --rejoin`
starts at the actual `fortress1_start2` entrance and follows the upper wall with
ordinary navigation, Blade attacks and the running jump into the rocking school
window. `--first blue` repeats the other trigger order. Internet joining with
40/20 ms delay/jitter remains the default; neither run stages a mid-level start
or writes gameplay state. The shared harness now accepts an optional authored
entrance; existing first-visit fixtures retain their previous arguments.

Red's public run is `private/coop/fortress-return-0e72ab5bd10a471fbcf6fe4dc06c87fc/`
(1,047 matching sent/restored states); Blue's is
`private/coop/fortress-return-a66cad4ce4344e68b5ae5169cf15ad47/` (1,064).
Each verifies the proper owner of `Fortress1_Boojum_Attack`, a frozen partner,
one safe reunion, the activated ambush, no respawns/wipes, and a fresh guest
rejoin preserving resources/placement/completed scene state before the outgoing
trigger. Both players enter `skool1_start1` at 100 Sanity in both runs. Reunion,
window approach and both school entrance views were inspected. The scene's
automatic capture is its opening white fade, so a middle-reveal capture still
needs inspection. Earlier fixture attempts stopped while trying to throw the
Blade at a guard from behind the corner parapet; the passing route completes
the ordinary approach before waiting for that fight. Failed evidence remains
in `fortress-return-bcef832fb66d498e96ac997201cce2b1` and
`fortress-return-47dcb81c92b6468d99b36ea883a07a05`.

These are traversal/reconnect results. The return visit's dedicated nearest-
visible targeting, personal consumable and both death/respawn/wipe checks remain
open, together with the middle-reveal view. No fifth-row completion credit is
claimed. Seven source-package tests and Python syntax checks pass in
`private/coop-fortress-return-source-tests1.log`; the game source/build remains
the validated `ef3db23e` above.

The return visit's focused guard and personal-healing check now passes on that
same build in `private/coop/fortress-return-combat-86bb88db6fc24b00b5320f8c0f494b9c/`.
`tools/test_coop_fortress_return_combat.py --data <base>` walks from the authored
entrance with public 40/20 ms delay/jitter. With both players visible, guard
`end_spawn4` selects nearer Red, then nearer Blue. Each checked hit removes seven
Sanity only from the selected player. Ordinary Blade attacks defeat the guard;
Blue collects her earned shard and heals 72 to 97 while Red stays at 79 and
retains her copy. Red then heals 79 to 100 while Blue stays at 97. Both clients
observe those separate collections. All 578 comparable sent/restored states and
eight coincident player/enemy reports match. Both attack views and both healing
views were inspected. The shared combat/recovery helpers now accept an explicit
loot visit, so this return uses `fortress1$return`; first-visit defaults are
unchanged. No game source or save format changed.

`tools/test_coop_fortress_return_recovery.py --data <base>` also passes on the same
build and public delay/jitter in
`private/coop/fortress-return-recovery-15c11dd1a47044339884ce41fd8b686f/`.
Each Alice walks into the actual guard encounter without attacking while the
survivor stays on the upper wall. Red dies/respawns at ticks 3,029/4,229; Blue at
7,055/8,255. Both clients agree on exactly 1,200 ticks, 25-Sanity recovery,
moving spectator cameras, restored personal cameras, retained inventory and
unchanged survivor health. Separation is 49.19 and 48.89 units respectively.
Their subsequent shared wipe at tick 10,518 restores the automatic checkpoint
from tick 1: exact resources, earlier Fortress state, enemy health/activation/
positions/state, and both starting placements within ordinary movement bounds.
All 1,538 comparable sent/restored states and 31 coincident player/enemy reports
match. Both spectator views and both restored checkpoint views were inspected.
Both recovered-player views were also inspected. The combat and recovery
clients/helpers close normally and remove their invites.

The route fixture now explicitly captures the reveal after its original white
fade. The Red-first repeat passes in
`private/coop/fortress-return-53bf1343634e43e19a655a6ffd592774/`, with 1,042 matching
sent/restored states, one reunion at tick 2,911 and fresh guest rejoin before the
school exit. Its middle captures show the original Boojum cutaway in both clients,
with no extra Alice in the shot; both final school views were also inspected.
The reported scene clocks after capture are 2.33 seconds in the host and 2.05 in
the delayed guest. Both finish at 100 Sanity. The original opening-fade captures
remain in the evidence, and the new capture does not change game state.

The Blue-first repeat also passes in
`private/coop/fortress-return-e744240e838d43b6b1776eea5d0c2ede/`: 1,036 matching
sent/restored states, the Blue-owned reveal, reunion at tick 2,906 and a fresh
guest rejoin before the exit. Both middle-reveal views and both final school
views were inspected; scene clocks after capture are 2.21/2.02 seconds. Both
arrive at 100 Sanity without respawns or a wipe. These two completed routes use
fixture SHA-256 `24aac8cc240f3ff41d2ba5423aa3e155f852c9d384470dec16d7ec5878e81d16`.

There is no activated passenger lift or ride on the return route. Fortress's
split-room trigger is first-visit-only (`Fortress::gate`), while the rocking
school is ambient machinery (`Fortress::advance_machinery`), so shared boarding
is not applicable here. Combined traversal, visual, combat, recovery and rejoin
evidence now completes the fifth campaign row. Overall weighted progress is
41.56%, reported as about 42%; 34 visits remain open. The game remains the frozen
`ef3db23e` build with its existing 746 passing Rust tests (one ignored), release
build and native/single-player checks recorded above. This checkpoint adds only
Python fixtures and evidence documentation. Seven source-package tests, Python
syntax checks and the source-boundary audits cover the authored additions.

## First school verification in progress

The first school visit has partial public two-process evidence on build
`4fd9c6cca856cc455099b23286834117ecb693144726781a33103c34bb4fab00`.
The only runtime addition is opt-in reporting of school scene names. The new
`tools/test_coop_school.py` fixture uses ordinary movement and combat inputs from
the normal entrance. Its retained Red-first run reaches both personal Mallets,
four correctly owned scenes with safe reunions, both lifts (including delayed
boarding), and the four-book bridge. All 14,137 comparable network states and
117 coincident full school/player/encounter reports match. Both waiting and
completed lift views, and the middle shelf/book reveals, were inspected.

Evidence is in `private/coop/school-route-1bb1e30f303f4edc9abdeb31367d7952/`.
It remains a failed full-route test: the host's navigation search times out at
the first bridge book after the guest reaches it. The fixture now uses a direct
walk for that small step. No recipe, exit or rejoin success is inferred from the
partial run. Both complete arrival orders
remain outstanding; the total estimate stays at about **42%**, with **5/39**
campaign visits fully verified.

The separate recovery fixture now passes with the optimized executable and the
same public 40/20 ms delay/jitter in
`private/coop/school-recovery-84df4abea6c8431fbd0072b0a61ca189/`.
Both actual guard deaths lead to exactly 1,200 ticks of moving-partner spectating
and revival at 25 Sanity, 48 units from the survivor. The subsequent team wipe
restores the real earlier checkpoint's resources, school/enemy state and starting
positions within ordinary movement bounds. All 1,930 comparable network states
and eight coincident full school reports match. Both spectator, respawn and
checkpoint views were inspected. This proves recovery at the entrance; it does
not establish completion of the school route. Reproduce with
`tools/test_coop_school_recovery.py --data <base> --exe target/release/looking-glass.exe`.

The frozen build passes 746 Rust tests (one existing ignored test), debug and
release builds, the native co-op world check and the native school state/lift
check. The latter covers both lifts at 30/60/144 Hz, waiting/obstruction, book
state contacts and fresh/return restoration; it is not a full two-player route.

The optimized Red-first repeat in
`private/coop/school-route-44d72453baad4b1690465f308c26a03a/` subsequently crosses
all four books, plays the recipe scene and successfully reconnects a fresh guest.
Five owners/reunions, 10,233 network states and 294 full school reports agree.
The run still fails before the exit: the guest overshoots a lower shelf during
the timed descent and dies from the longer fall. The fixture now uses a fresh
theatre guard reward and a slower, observed edge descent. The Mallet approach
is split at the lower stair after another run missed the long planned jump;
the short approach reaches the platform in a focused probe. The essence near
the Mallet is return-only and is not made available on the first visit. The
original recipe cinematic contains one Alice in her original blue clothing,
including when Red is the trigger owner; that costume has not been recolored.

Focused combat and healing now pass on the same build/settings in
`private/coop/school-combat-2b97ba23cace4d9f8b9ee9202be946ea/`.
With both visible, the upper guard selects nearer Red, then nearer Blue; each
checked hit removes seven Sanity only from the selected player. The fresh
theatre-floor guard reward heals Blue 93 to 100 while Red stays at 58 with her
copy retained; Red then heals to 83 without changing Blue's resources. Both
clients observe the separate collections. All 1,662 comparable network states
and 28 full school reports match. Targeting and healing views were inspected;
the clients/helpers close and the invite is removed. Reproduce with
`tools/test_coop_school_combat.py --data <base> --exe target/release/looking-glass.exe`.
The updated full route takes that fresh reward, splits the Mallet approach at
the lower stair, and attempts a slow descent onto the lower library shelf.
Those changes still need a complete repeat. Overall progress remains 42%.

The later library-to-exit checkpoint repeat also passes after replacing the
upper stair navigation leg with a deliberate running jump. Both players reach
the next school alive, with all remaining scene owners, frozen partners,
reunions and a fresh guest reconnect checked. The preceding full repeat still
counts as a failure because Blue fell while navigating that upper gap. Neither
complete first-school arrival order has yet passed; progress remains about 42%.

Second-school co-op machinery now checks both living players before moving a
group or carrying its riders. Native checks cover both moving furniture riders,
either player blocking a pendulum and its bound lamp, pause/save restoration,
and 30/60/144 Hz. Single-player scenes retain their original owner step. These
focused checks do not verify the second-school quest/combat or full co-op route;
the build identities and evidence are recorded in `COOP_CAMPAIGN.md`.

## Repeating a school section from a normal checkpoint

For diagnosis, the school route accepts `--checkpoint-stops theatre balcony library bridge descent`.
At each stop it holds both clients' movement inputs for up to three minutes.
Use the host's ESC menu to **Save checkpoint**, retain a byte-for-byte copy of
that profile's `coop-checkpoint.json`, and resume the menu. Create an empty
`continue-theatre`, `continue-balcony`, `continue-library`, `continue-bridge` or `continue-descent` file in the printed evidence folder to
continue the route. The option is off for unattended full-route tests.

The saved section can then be repeated with:

```text
python tools/test_coop_school_resume.py --data <base> --checkpoint <saved-file> --resume-point balcony --first red --rejoin
```

Use `--resume-point theatre` before the Mallet stair descent,
`--resume-point library` before the upper library stairs,
`--resume-point bridge` before crossing the books, or `--resume-point descent`
for a checkpoint taken after the recipe reunion.
The fixture copies the original save unchanged into a fresh host profile, uses
the game's normal resume path, and checks that both clients retain their
positions, personal resources and school progress. It records the checkpoint's
SHA-256 and source tick. It never edits health, positions or world state, and a
successful segment does not count as a full entrance-to-exit campaign pass.

The bridge-to-exit repeat now passes over the public internet with 40 ms delay
and 20 ms jitter per sender, including a fresh guest reconnect. It exposed and
verified a fix for entering the recipe scene while airborne: after the scene,
the partner can land on safe nearby ground and the owner resumes normal gravity.
Grounded reunions and ordinary death respawns keep their narrower placement rule.
Both players survive the lower shelf descent and reach the next school entrance.
See `COOP_CAMPAIGN.md` for the build, evidence and remaining full-route checks.

School lift support now gives a static landing precedence over a platform below
it or at the same height. This prevents the player who has already stepped off
from blocking the remaining rider or being dragged around by the rotating lift.
Actual body obstructions still stop the mover and every rider together. Native
checks reproduce the observed near-top stop with either player on the floor;
the separate-client route fixture also uses ordinary inward walking to clear
the spiral's outside rim beneath the landing lip.

## Reproducing transport checks

Build with `cargo build --locked`, then run:

```text
python tools/test_coop_network.py --delay-ms 60 --jitter-ms 30
python tools/test_coop_network.py --internet --delay-ms 40 --jitter-ms 20
```

The internet check downloads the official Windows x64 cloudflared 2026.10.2 helper
if absent, verifies its size (55,329,336 bytes) and SHA-256 before execution:
`12c7c887f8aaab779aef1dfa9b60673530a5adb88ecc55dd6df8f56e0b70f199`.
It creates only an owned temporary tunnel to the loopback co-op listener and stops
that helper at the end. It does not install a service or change firewall/router
settings. Each run writes private diagnostic evidence under `private/coop/` and
removes its invite file. These checks need no original game assets.

Quick Tunnels are for development/testing, not an uptime promise. A released build
needs a separately reviewed service choice and updated third-party notices for the
network dependencies/helper. No co-op release has been created.

Local `coop-live.json` diagnostics include guest `network_timing` (sent input
sequence, last acknowledged input round trip, oldest retained pending input age)
and `frame_timing` (render-frame duration, guest polling and world restoration).
Input timing keeps at most 128 send timestamps and resets on a visit transition.
These observations do not change controls, packet ordering or authoritative
simulation. The school fixture retains them in its private history to distinguish
connection delay from slow client updates during a traversal.

The unchanged theatre checkpoint also passes through the school exit with Blue
arriving first on build
`f2ff00f9d7e31835bfc705351276cfba5f631d0657aa3bdd597ade6014ac6d08`.
This two-process public-internet run used 40/20 ms delay/jitter on each sender,
both paired lifts, the upper gap and flying books, the recipe reunion, fresh
guest rejoin and lower-library descent. The final audit compares 10,092 actual
sent/restored states and 192 full school frames; both players remain alive,
ending at 21/77 Sanity without a checkpoint wipe. Guest input round trip was
226 ms median, 254 ms at the 95th percentile and 474 ms maximum. Evidence:
`private/coop/school-resume-ca7c14ed54954d9281665968c4070126`.
This is checkpoint-to-exit evidence only. An earlier return-stair recovery in
the segment means it must not be described as a recovery-free full traversal.
The overall estimate remains about 42%, with 5/39 campaign visits complete.

## Player contact and effect presentation

Co-op no longer hides the partner merely because the two bodies are close.
The host resolves ordinary player contact at 36 units of horizontal separation,
with vertical clearance for different floors. An approaching player yields;
their stationary partner is not pushed. Corrections are checked against walls,
ground support and swimming depth, and leave rope, ledge and scripted motion
under their existing controllers. Dead bodies do not obstruct living players.

Each client now loads the original level emitters, follows the shared world
switches and mechanism positions, and filters collectible emitters using its
own player's inventory. Checkpoint changes reset old particles. Co-op supplies
ambient, weapon and enemy lights, draws Boojum waves, and flushes the sorted
transparent-effect pass before drawing the interface. Previously that final
pass was never flushed, so even queued weapon and scene effects were absent.

The focused two-client check is `tools/test_coop_presence.py`; it uses ordinary
movement at the village entrance after its original opening scene. Diagnostics
in `coop-live.json` include visible-player flags, emitter/live-particle counts,
and rendered/dropped effect counts. Counts complement inspected captures;
they are not proof that every effect in every campaign visit has been reviewed.
Existing full-route fixtures which ask both players to stand on exactly the
same waypoint need separate stopping positions before their next route pass.

Verified build:
`612b892a490afa1b06b6ac182c7323edcf933eef841e294941d508f2e9294980`.
All 755 Rust tests pass (one existing ignored), as does `--coop-world-check`,
including both school lift boarding orders and the exact near-top landing
regression. Logs: `private/coop-presence-{tests,world,release}.log`.
The two-client public-internet presence test passes with 40/20 ms delay/jitter:
`private/coop/presence-test-8d280ef631054daf89433b9ea789b384`.
It compares 614 sent/restored states, checks each approaching player without
moving the stationary partner, confirms enabled models and live particles in
both windows, and observes rendered effect triangles without dropped effects.
The contact and entrance captures were inspected. The two preceding attempts
retain a premature test during the entry scene and an invalid requirement for
visible effects at every camera angle; neither receives a pass. This focused
regression adds no completed campaign visits. The existing traversal evidence
predates player contact and must be repeated before release.

## School route inputs after player contact

The opt-in route report now offers nearby supported stopping points, checked
against walls, hazards and landing height. These are read-only navigation goals;
the fixture reaches them with ordinary inputs. They never move a player directly.
The school fixture gives the second player a separate endpoint, sends players
through narrow stairs in sequence, and clears a landing before the next jump.
Both players still have to touch their own Mallet pickup. Grounding, landing,
health, checkpoint and network-coherence checks remain required.

Build `d4869cdf7f939ddeab026e4c5613e57ef8855e02fa44fe9d5ccc6b71bf40f3aa`
passes 756 Rust tests (one existing ignored), the release build and
`--coop-world-check`. Logs: `private/coop-neighbors-{all-tests,release,world}.log`.
The new placement test covers a broad floor, a narrow platform, walls, hazards
and a lower floor that must not qualify as the same landing.

The delayed public two-client school repeat
`private/coop/school-resume-378c70871dd14f6f9af3636ebd9eebd8` verifies both
directions of the Mallet stairs and both personal pickups, with no deaths,
checkpoint reset or cinematic warp. All 1,708 matching sent/restored states
and 13 coincident full school reports agree. Both finish that partial attempt
at 97/100 Sanity. It then fails the second balcony jump from the offset waiting
spot; this is retained as a failed route, not campaign completion. Subsequent
fixture work uses the intended run-up for each jumper and the wider balcony
as the first player's waiting area. The full route remains open.

The balcony correction passes in the subsequent Anode run
`private/coop/school-resume-5e37391b2d0348c28ea9350c75f589de`, using the unchanged
normal menu checkpoint saved at tick 36,452 in the preceding `86c39c0c...` run.
Both jump landings and the first library approach pass; host/guest landing
captures were inspected. The guest now aims at the actual upper landing rather
than the old offset target over the lower slope. This attempt was stopped later
when enemy knockback moved the leader into the partner's waiting destination;
3,102 matching states and five coincident full reports agree, with both alive
at 83/100 Sanity. The fixture now clears nearby enemies before choosing these
destinations. This remains partial traversal evidence, with no new campaign credit.

Clearing combat alone does not cover later knockback: the `f69cf642...` repeat
still times out against an occupied endpoint. The fixture now reselects a
supported endpoint if the leader moves more than eight units, with a bounded
retry count and unchanged final distance, height and grounding checks. In
`private/coop/school-resume-468abae7fed045bb8b34234eeef32083`, two real knockbacks
cause two recorded replans; both players finish that leg and the following
library-floor legs. All 2,607 matching states and 112 coincident full reports
agree, with no death or checkpoint reset and final Sanity 86/100. The run was
then deliberately stopped to hand Anode to queued work. Lifts and the remaining
route were not reached in this repeat; it adds no completed campaign visit.

### Second-school combat and quest adapter (24 October)

The host now includes School2's legacy Boojums, rescue guards and reinforcements
in weapon contacts and the once-per-world combat update. Their per-player
damage/knockback feedback reaches the selected Alice. Rescue guards continue
attacking the Gnome until provoked. Living players can both collect the story
ingredients and rewards; those quest flags are shared while health, Will and
ordinary inventory remain personal. Scene/dialogue steps advance actor/audio
presentation with quest and combat paused. Single-player retains its wrapper.

Build `a3ce8a69e7cb0f9a27aacc788300bb67dc6c733fc0c70c5f1733512d937fe4be`
passes all 756 Rust tests (one existing ignored test), release compilation,
`--school2-check` and `--coop-world-check`. New staged checks use both player
roles to verify individual damage, dead/concealed-target changes, serialized
save continuation, all four real-map quest contacts, shared quest flags,
personal resources and one quest clock. Barrier insertion protection is tested
with an explicitly blocking fixture using the original enemy-only brush's
volume; production keeps its original collision mask. Evidence is in
`private/coop-school2-combat-{release3,native3,world,tests-final}.log`.
These are component checks. Full rendered second-school co-op traversal,
real-network combat and scene ownership still need verification; row 7 and
the overall estimate (about 42%, five completed visits) remain unchanged.
The complete single-player School2 regression also passes on this build:
53,236 ticks, nine jumps, 23 throws, 149 cards, 170 combat damage and 27 final
Sanity, with both original portals, all quest items and the gated return.
`private/coop-school2-combat-route.log` records the no-cheats/no-recovery run.

### First-school traversal follow-up (24 October)

The collision-aware route now retains airborne braking when releasing run,
checks that delayed walking inputs have actually settled at each paired
endpoint, and stops the joiner near a clear boarding position once the host
starts the lift. It keeps the existing ground/height/distance and both-riders
arrival assertions. In `school-resume-cefce6a0...`, waiting for both clients to
report a fixed world destination kept Red walking after the spiral rotated;
in `0c08caab...`, stopping at the first boarding signal left her too close to
the central column. Both failures remain in private evidence.

The revised boarding passes in `private/coop/school-resume-43cbe3f3d87340a6b71ea483ca6ed55c`
on frozen build `d4869cdf...`: both players wait/ride to upper-floor height,
and both top captures were inspected. Red subsequently falls when the paired
endpoint shortcuts the narrow exit; this is not a successful route. The next
repeat, `private/coop/school-resume-f392612e7c114c5397f9902d577fa37e`, passes
the balcony, library approach and spiral boarding/ride again, then deliberately
stops at a normal menu save before the exit. Its 4,628 matching network states
and 157 coincident full reports agree; both remain alive at 98/100 Sanity.
The unchanged checkpoint at tick 65,443 is retained as `spiral-checkpoint.json`
(SHA-256 `890a3d4a3044949b4c1d87d87acf807b3077169aaa896e3556d4543e1ccf5436`).

The fixture now supports `--resume-point spiral` and a `spiral` checkpoint
stop. Departure follows the narrow exit in turn and recognizes an authored
cutscene reunion, so the waiting partner continues from the safe reunion
point instead of walking back toward the now-departed lift.
Lower-shelf descent also clears the first landing before the partner follows;
that later segment still needs verification. No additional campaign visit is
credited by these partial tests.

Build `a3ce8a69...` restores that unchanged checkpoint and completes the shelf
scene with both players safely reunited on the upper floor. In
`private/coop/school-resume-a6daea30dc114453800d1ed5aa4adba3`, the subsequent
library approach fails: enemy knockback pushes the stationary leader off the
ledge, and the fixture incorrectly chooses a partner destination on that
lower floor. Its 4,610 matching network states and 98 full reports agree,
but it receives no traversal credit and no library checkpoint was saved.
Paired legs now steer back toward the reached goal while fighting and verify
the leader remains near the requested goal and height before and after the
partner follows. The existing strict partner landing checks remain in place.

The corrected repeat, `private/coop/school-resume-4d2e9c6bcada436bbf63b18329256e5a`,
passes the lift departure, shelf scene/reunion and three upper-floor library
legs on the same `a3ce8a69...` build over the public connection with 40/20 ms
delay/jitter. Both remain alive at 70/100 Sanity, on the intended floor at
height 64.03125. All 2,608 matching states and five coincident full reports
agree. The host's normal ESC menu saves a checkpoint at tick 79,790, confirmed
in the UI and retained byte-for-byte as `library-checkpoint.json` (SHA-256
`886ec09f0cad157e3080524af516418f464b2b7b2755eabe264582591e08d757`).
The test then deliberately stops; this is a verified partial segment, not a
completed first-school traversal. Upper stairs, books and descent remain open
on the current player-contact build.

The focused player-contact/effects regression also passes on `a3ce8a69...`
in Anode: `private/coop/presence-test-85c43af39d0d4a8997a601be0b6c430c`.
Both independent clients use the public connection with 40/20 ms delay/jitter;
615 sent/restored states match. Each approaching player stops at 36 units,
the stationary partner stays fixed, both models remain enabled, ambient
particles are live and each client's history includes drawn effect triangles
without dropped batches. Entrance and contact captures were inspected.
This focused pass does not certify every campaign effect. All owned test
processes and windows were closed, and Anode was released afterward.

### Upper library and School2 test controls (24 October)

The upper stair-and-gap fixture now lets each player complete the stairs and
jump in turn, with the first continuing west along the gallery before the
second lands. This avoids placing a waiting body in the take-off point or
the next route waypoint; collision and landing tolerances are unchanged.
The public 40/20 ms repeat on frozen `a3ce8a69...` reaches the flying-book
approach with both alive, after the upper gap, gallery and paired elevator2
boarding/ride. Its normal host-menu checkpoint at tick 105,626 is retained
in `private/coop/school-resume-390422f07ec646cab97cefdcbba89ebb/bridge-checkpoint.json`,
SHA-256 `6ff0424cf634b993c816f4179b6132e1431eaa3de83bf456a094bfc41746d920`.
This checkpoint alone does not establish a completed route.

The opt-in route controller now includes `School2::targets()` when aiming
ordinary attacks. The host arena already included those legacy enemies;
without this addition, a future School2 fixture could walk past enemies that
normal players can fight. Build
`2a7bbf285ac97e7299bc27ee10b161e22e6f6851035608aa0d92a5602a35921c`
passes all 756 tests (one existing ignored), release compilation,
`--school2-check` and `--coop-world-check`; evidence is in
`private/coop-school2-driver-{tests,release,native,world}.log`.
These checks do not replace the pending rendered School2 route.

The book fixture now brings each jumper to the same take-off point in turn,
and anticipates delayed guest braking on the small landing surfaces. The
three shelf descents likewise use the authored path in turn, rather than
giving the partner an offset destination over open air. These are ordinary
input changes; no player positions, health or checkpoint payloads are edited.
On `2a7bbf28...`, `private/coop/school-resume-d44435ba535244dea12798e3413b7d74`
passes all three book jumps, the Blue-owned book scene with Red frozen,
safe reunion and a fresh guest reconnect. Its normal-menu descent checkpoint
has SHA-256 `7b6331a1dc4d15268932e9c76072dc3801710245d7990979119e2fb8641bdeb1`.

The repeat from that unchanged checkpoint,
`private/coop/school-resume-08cd7701e325452a98fe4f517e54879e`, passes the entire
descent and authored exit into `skool2_start1` (epoch 2). Both clients use the
public connection with 40/20 ms delay/jitter; 609 sent/restored states and
17 coincident full reports agree. Both arrive alive at 12/77 Sanity, with no
death, respawn or checkpoint rollback. Both final rendered views were inspected.
This is resumed-segment evidence, not a complete school traversal.

The combined bridge repeat
`private/coop/school-resume-a42391b5188e4f57ac537639a54f7c94` exposed a real
scene handoff failure: the book scene finished, but the partner placement
search rejected the owner's low-clearance landing and kept both players
frozen. Both clients agreed on the stalled state (2,928
matching sent/restored states and 34 coincident full reports); neither died.
A read-only replay of that observed world confirms the raised probe starts
inside the shelf while a level path reaches supported ground beside Alice.

The normal placement probe starts one unit above Alice and searches from
raised candidate positions. Under a shelf, that can intersect geometry even
though her actual body fits. Scene reunion now also tries a level sweep from
the actual body position when the usual samples fail. It retains the same
distance/drop limits, collision sweeps, hazard rejection and full-footprint
support checks. Ordinary revival placement is unchanged. Low-ceiling tests
also reject unsafe landings and intervening walls; the native regression
checks the observed book-edge position for both owners.
That regression runs the full original book scene before testing reunion,
including removal of the book's old collision shape, and confirms that the
previous raised placement search still fails at the observed position.

Build `ced2c8cb5be427ef3329c06889cce29cf7aae7ef04609b1d8544504e869dc99a`
passes 757 tests (one existing ignored), release compilation and
`--coop-world-check`, including the original book scene and safe handoff
with each Alice as owner. Both retain their health, the owner remains at her
original position, and the partner remains frozen until reunion. Evidence:
`private/coop-recipe-handoff-{tests,release,world}.log`. The frozen executable
is `private/coop-verified-ced2c8c.exe`, SHA-256
`289e3f9ebafc61f49e4da1aa7d3d701749c492d268920e96d811b2752b527107`.

The combined public two-client repeat on that build passes from the unchanged
bridge checkpoint through all three book jumps, the original book scene,
reunion, a fresh guest reconnect, all shelf descents and the authored exit
into `skool2_start1` (epoch 2). Both arrive alive at 12/77 Sanity with no death,
respawn or checkpoint rollback. All 1,882 sent/restored states and 35 coincident
full reports agree; both final rendered views were inspected. Evidence:
`private/coop/school-resume-b9776816d3284d4f932228f3a25cddef`.
This repeat enters the book scene while airborne; the native regression above
separately covers the exact grounded position that previously froze.
The resumed segment is complete, while both complete entrance-to-exit arrival
orders remain open. Overall campaign progress remains about 42%.

The focused player-contact/effects repeat on the same build also passes:
`private/coop/presence-test-8556bf1b5162437486550fb981260815` records 597 matching
sent/restored states, 36-unit player separation, an unchanged stationary
partner, both models enabled, live ambient particles and submitted effect
triangles without dropped batches. This is automated presentation evidence;
it does not certify every campaign effect or replace visual review of changed art.

### Automated regression runner

Run `python tools/test_coop_regressions.py --data "C:\path\Alice1\bin\base"`
inside Anode. It runs the native shared-world/scene checks, native School2
checks and two-client player-contact/ambient-effects test. Internet joining
with 40/20 ms delay/jitter is the default. `--exe` selects a frozen build;
`--only world school2` selects the checks that need no desktop.

Add `--school-checkpoint <ordinary-checkpoint.json> --school-resume-point bridge`
to include the books, cutscene, fresh guest reconnect, descent and exit; other
documented school resume points are also supported. Checkpoint bytes are copied
unchanged through the existing resume fixture. `--first red` changes the leader.

Use `--only school-blue school-red` for two complete, independent first-school
entrance-to-exit runs, with Blue then Red arriving first. Each run starts from
the normal entrance and includes a fresh guest reconnect. These cases need no
checkpoint argument; their names select the arrival order independently of
`--first`. They retain the route's scene, lift, personal pickup, no-death and
network checks and each has a 25-minute timeout. Focused combat/recovery checks
remain separate from this traversal coverage.

Use `--only lift-blue lift-red` for quick spiral-lift boarding checks in both
orders. These start two independent clients beside the lower-library lift with
the explicit `--start-at` diagnostic spawn, then use ordinary movement for the
approach and ride. They require a two-second partner wait, both riders reaching
the upper landing, matching network state and no death or wipe. The staged
start makes these component checks; they do not certify school traversal.

`--only stairs-blue stairs-red` similarly checks the full Mallet trip out and
back from an explicit staged upper-landing spawn, including both individual
pickups. On the return each player first walks to the same edge, descends
slowly onto the narrow lower support, and approaches the
upper step before its short jump. This avoids treating the entire descent and
climb as a running navigation plan. Both players must finish grounded on the
upper landing with matching shared-world states and no death/wipe/respawn.
Short school waiting moves request a three-unit stopping radius and a slower
walking input, so a supported spot on a small step is actually reached. Other
walks retain the original eight-unit default. Slow inputs on the narrow stair
supports limit overshoot while delayed feedback is in flight. Planned
navigation and game physics are unchanged.

The runner writes one private `regression-suite-*/summary.json` with bounded
pass/fail output, executable and fixture hashes, elapsed times, network/effect
metrics, evidence locations and explicit coverage scope. It stops at the first
failure, applies per-test time limits and cleans up its owned process tree on
timeout/interruption. Partial or interrupted runs cannot report a suite pass.
Failures include a bounded log excerpt and, when available, each client's last
command, scene, player positions, health and most recent supported stopping
points. This permits routine triage from the summary without opening images or
loading the complete frame history. Missing final evidence is reported without
hiding the original failure.

Prefer these automated assertions and compact reports for routine repeats.
Keep captures in the evidence folder, and inspect images only for a new visual
change, a failed check, or a problem that the current assertions cannot measure.
A focused suite pass does not complete a campaign-matrix row.

New diagnostic builds additionally hash the complete shared stage at each
sent/restored network tick: movers, quest state, scene ownership, contacts and
story progress. School fixtures require at least 20 matching shared-stage hashes
when this field is available, while retaining every available same-tick full
report comparison and the existing player/enemy hash comparisons. Older builds
still require coincident full reports. This makes short component checks
independent of the two local report clocks without weakening state validation.

The first complete runner verification passes all four cases on `ced2c8cb...`:
`private/coop/regression-suite-746c9472e407480682aff113ca65697d/summary.json`.
The native world and School2 checks pass, the presence test records 623 matching
network states and 36-unit player separation, and the bridge-to-School2 segment
records 1,972 matching states and 67 full reports with resources/scenes retained
after guest reconnect. All checks completed in about 261 seconds, using the
public connection with 40/20 ms delay/jitter for the two-client tests. No images
were opened during this suite. A separate empty-data failure probe correctly
returned failure and stopped before the next case; its report is
`private/coop/regression-suite-f99df7c54fc845f6956a6dc8d8722730/summary.json`.

### Full-school fixture with player contact

The first-school controller now moves the waiting player aside when they block
the leader's nearby departure path, including after an original scene reunion.
It uses the existing supported stopping points and ordinary local inputs. The
theatre switchback is traversed one player at a time because its intermediate
steps have no safe side-by-side stopping point. Attacks remain enabled while
climbing past the optional side-stair guard; the fixture no longer waits for a
kill from fixed positions where the Blade strikes the stair edge.

The full Blue-first attempt on `ced2c8cb...` in
`private/coop/school-route-5092902e7c634403a90a9070a618064e` passes the entrance,
theatre, original shelf scene/reunion, stair switchback, both personal Mallet
pickups and the balcony jump. It stops in the lower library when a late enemy
hit moves the waiting leader away from the requested waypoint. Both clients
agree on 6,796 sent/restored states and ten coincident full reports; health is
86/100, with no death, respawn or checkpoint restore. This is failed-route
evidence and receives no complete campaign-row credit.

The controller now keeps steering toward the requested landing while the
partner follows and returns there before replanning after a displacement.
The final height, position and support checks remain unchanged. The follow-up
starts from the unchanged ordinary balcony checkpoint with SHA-256
`59378fe503dfe6982064471df5ea2d3d90acc108a5d00ae533c4f372bc9e35b2`.

That repeat passes the corrected combat waypoint, lower library, shared spiral
lift and its narrow exit. It exposes a separate approach problem at the upper
bookcase: an endpoint supported beside the leader can be unreachable directly
from the partner's side. Straight approaches now record four seconds without
progress, reject that candidate and try another supported spot, retaining the
existing attempt limit and final checks. The bookcase corridor itself is now
traversed in single file, with the leader stepping aside after clearing it.
Passage movement remains aimed at each waypoint until nearby combat finishes.

The final repeat on the same frozen game build passes from the unchanged
spiral-lift checkpoint through the upper library, second paired lift, all three
book jumps, recipe handoff, fresh guest reconnect, shelf descent and authored
exit into `skool2_start1` (epoch 2). Evidence:
`private/coop/school-resume-adaa2520042b4a3b89c4028334de4514`.
All four remaining scenes belong to Blue, with the partner frozen until their
handoffs. Both clients arrive alive at 12/77 Sanity, without death, respawn or
checkpoint rollback; 6,058 sent/restored states and 60 full reports agree.
Two departure clearances use ordinary movement. The final fixture SHA-256 is
`9c04515e642f73ac2bafc0a93027feb2a23f79c0b71f7a1ccbcbde9d012c5e7a`.
No images were opened during these repeats. This completes the resumed segment;
both complete entrance-to-exit arrival orders remain open, and overall progress
remains about 42%.

### Full-route controls and short lift regressions

The next complete Blue-first attempt exposed continued jump planning after a
brief stair landing. Passage controls now switch to damped walking at arrival,
as paired landings already do. Its repeat passed both stair passages, both
personal Mallets, the balcony and every lower-library waypoint before exposing
the lift approach. A rider can touch the lift while overlapping the fixed lower
landing, and requiring the final planned endpoint can continue walking after
rotation starts. The fixture now aligns the first rider, switches the second
from jump planning to walking when boarding starts, and releases movement at
the first actual lift pose change. Production boarding/carrying rules are
unchanged. The waiting, both-rider upper-height and no-death checks remain.

The five-case suite passes on build
`57fd88a789fcc1474432304154723b58707a6579fe335dfbeefa8dab60992657`:
`private/coop/regression-suite-0416f33b7f5947c39af2731d88d4d1dd/summary.json`.
Native world and School2 checks pass, followed by the rendered contact/effects
case (613 matching player/enemy states) and both staged lift orders. Blue-first
passes with 278 matching player/enemy and complete shared-stage states, despite
zero coincident local full reports; Red-first passes with 317 matching states
and 31 coincident full reports. Both lifts wait two seconds for their partner
and carry both players to height 64. The suite takes about 202 seconds; both
lift cases take under 40 seconds each. All 757 Rust tests and the release build
pass (one existing ignored test). No images were opened.

The full-school runner correctly recorded the earlier failed traversal and
stopped; these component passes do not replace it. Complete entrance-to-exit
runs in both arrival orders and current-contact combat/recovery repeats remain
open. Overall progress stays about 42%.

### School Two enemy observations and stair-return repeat

School Two's original quest-owned guards and Boojums now appear in opt-in route
diagnostics alongside the shared encounter actors. Their stable combat IDs,
activation gates, health, selected player, distances and visibility are read
without advancing AI or changing the world. The real-data School2 check verifies
all legacy actors are included, active IDs match combat targets, each enemy
group reports the nearer visible victim in both player roles, dead players are
not visible, and observation leaves the saved world state unchanged.

Build `dfa73f5c13cafaffb5c51a18671aa48e2ef53414462c37f67d1416ede6fd8686`
passes 757 unit tests (one existing ignored), the release build, native School2
and shared-world checks. The focused stair return passes in both orders on the
public connection with 40/20 ms delay/jitter: Blue-first evidence
`private/coop/school-stairs-6f3f540cf8154a69ae18fa1847b4713c` has 672 matching
player/enemy and shared-stage states plus seven coincident full reports;
Red-first evidence `private/coop/school-stairs-d90a7ac236bd405987b7346eea6e9482`
has 653 matching states and two full reports. Both finish at 100/100 Sanity.
They take about 60 and 59 seconds respectively. No images were opened.

These are component results. The preceding full Blue-first attempt failed on
the earlier stair-return controls after 5,214 matching network/shared-stage
states, so it receives no traversal credit. Complete school runs in both arrival
orders, focused combat/recovery repeats and School Two network traversal remain
open; the estimate stays about 42%.

### Automated Mallet trip and compact failure reports

The focused stairs fixture now covers the approach, both personal Mallet
pickups and the return climb. Each player first reaches the stair centre from
their own waiting position. The controller uses small ordinary walking inputs
on the narrow supports and more precise short waiting moves; it never writes
positions, health or inventory after the explicit initial staged spawn.

Build `1d02f54c6a39f5a9943584f9a6c67fb48d60d8fe659b8b78521a5f35f0f2eefe`
passes 757 unit tests (one existing ignored), release compilation and the native
world/School2 suite. That suite is retained in
`private/coop/regression-suite-f1db698a3af24eadb8a63842cfc49061`.
Blue-first completes the expanded route over the public connection with 40/20 ms
delay/jitter in 184 seconds, at 100/100 Sanity: evidence
`private/coop/school-stairs-ddfba6c292f0445a8397631831f12982` contains 2,813
matching player/enemy and shared-stage states and 88 coincident full reports.
Red-first subsequently passes with settled waiting-position corrections in
207 seconds at 100/100 Sanity:
`private/coop/school-stairs-04aef6633c83432bb201adcbdc3b4759` contains 3,106
matching player/enemy and shared-stage states and 66 coincident full reports.
Waiting moves now let delayed inputs drain and make at most three ordinary
corrections, requiring both clients to confirm a grounded position within
three horizontal units and two height units of the supported target.
The final Blue-first repeat passes in 193 seconds at 100/100 Sanity:
`private/coop/school-stairs-04255752c6124e97b34ce37016e666d0` contains 2,823
matching player/enemy and shared-stage states and 88 coincident full reports.
Both final orders pass in the single runner report
`private/coop/regression-suite-0e5a204930b24acc9ab958fad8490321/summary.json`.

Earlier expanded probes exposed unsupported diagonal approaches and delayed
control overshoot. Their failing summaries retain client positions, commands,
last supported targets and a bounded log excerpt automatically. They receive
no traversal credit. No images were opened for these checks. Full first-school
traversal in both arrival orders remains open; the estimate stays about 42%.

### School Two network route under verification

`tools/test_coop_school2.py` follows the native School Two route from the normal
entrance using two independent clients. It covers the personal Demon Dice
altar, the gym lever and bleachers, upper-gym battle, laboratory rescue,
Jumbogrow, both authored portals, greenhouse growth, quest rewards and the
gated return to `skool1_start2`. Inputs use the existing controller; the fixture
does not write positions, health, inventory or quest state. Its complete route
still needs runtime verification in both arrival orders.

The regression runner accepts `--only school2-blue school2-red` for complete
route attempts, with a fresh guest reconnect after the greenhouse. The existing
`--only school2` still means the native School Two checks. Use
`--only dice-blue dice-red` for the partial normal-entrance route through both
personal Dice pickups and the return from the altar. This partial route has
no staged spawn, but does not certify the whole visit. Each case retains exact
build/fixture identities, explicit scope, bounded runtime and failure details.

The fixture observes School Two's original enemy pools and waits for both
ordinary and legacy Boojum waves to clear. Verification requires matching
shared-world states, the expected scene owners, frozen partners and safe
reunions, personal Dice collection and no death/respawn/wipe. The full route
also requires two portal traversals per player, completed quest rewards and
the correct return entry; consumed Jumbogrow is not mistaken for a missing
final reward.

The first-school entrance repeat exposed a waiting target only 2.35 units
beyond a wall contact. The existing final criterion allowed three units, but
the controller still requested two. Those tolerances now agree without
relaxing the final grounded/height checks. That repeat passed the theatre and
healing section, then stopped on the later stair climb. The theatre stair
inputs now align both players at the same approach and use the slower landing
controls; that latest adjustment still needs a complete repeat.

The first School Two entrance probe completed both physical Dice pickups and
the return stairs, but correctly failed its scene check: co-op collection had
omitted the ordinary game's `dice_cat` pickup trigger, so the scene and its
guard ambush never ran. Collection now starts the original scene and binds it
to the first collector. Each player still receives their own Dice; collecting
the second copy cannot replay the scene or replace its owner.

The native `--coop-world-check` regression exercises that path with either
player collecting first. It checks personal inventory, ownership, the frozen
partner, serialized/restored scene state, supported reunion, ambush activation
and no replay on the second collection. Both orders passed on source build
`ea619c58d85f926afd043ed7011fb9b2419de8a8a4da7beee0e8406f5cfdffd9`.
This component stages the collector at the real altar; the two-client route
must independently reach it from the ordinary entrance.

School Two route attempts now write `milestone-dice.json` and
`milestone-gym.json` only after their complete prefix checks pass. Each is
explicitly marked `full_visit: false`. If a later section fails, the suite
reports those verified components alongside the failure, without converting
the attempt into a full-level pass. Routine triage uses these JSON results,
shared-world hashes and bounded log excerpts; saved captures are opened only
when the failure calls for visual inspection.

The same build passed the full native School Two check as well as the co-op
world check (`private/coop/regression-suite-5975a9b630154110b9d70187f436487a`).
The Red-first internet attempt observed the restored scene and ambush in both
clients and both personal pickups, then failed before completing the return
path. Its waiting-point selector had treated supported map geometry as free
of the other Alice. Waiting targets and straight approaches now account for
the partner's occupied space. A recorded-state replay of the failure chooses
the available unoccupied target. Final paired approaches also use slow,
tighter steering so their broad arrival radius cannot stop short on a slope;
the final grounded, distance and height requirements are unchanged. These
controller changes affect automated route input only.

For faster iteration at the altar, `--only altar-red altar-blue` runs a focused
component with an explicit initial spawn on the lower landing before the gap.
It still walks/jumps, collects both personal pickups, runs the scene and
ambush, and returns over the gap. Results identify the staged spawn and never
count as entrance traversal or a full visit. `dice-blue/red` retain the normal
entrance; `school2-blue/red` retain the complete route.

School Two's test controller requests owned Playing Cards when the player can
pay their ordinary Will cost, falling back to the Blade when they cannot.
Other fixtures keep the previous Blade-only default. This uses ordinary
weapon selection and attack inputs, without adding inventory or resources.
The fixture records each player's minimum Will and uses ordinary steering to
hold grounded waiting positions against enemy knockback. Death still fails a
clean traversal, including in staged components.

A shorter Blue-first repeat exposed a genuine handoff lock at a recorded Dice
pickup near the altar edge. The dialogue and camera had completed, but the
partner-placement search's eight directions missed the available supported
space. Scene reunion now tries a finer bounded search if the original search
fails, keeping the owner at their original position. It retains wall sweeps,
full-footprint support, hazard checks, separation and the same drop limits;
ordinary death respawn placement is unchanged. A narrow-landing unit fixture
and both-owner native checks at the recorded map position cover this case.

Build `6205a876066a49c68cb95683de217453de2193d943afb1ee7bdc3436558558c8`
passed 758 unit checks (one existing ignored test), the release build and the
native co-op world suite, including both owners at the recorded altar edge
(`private/coop/regression-suite-94a51d0cf5f644b190b9720c57a80d12`). Its
Blue-first staged network component passed in 64.72 seconds in
`private/coop/regression-suite-9c7e919e3c804389a68d2b86f61641ae`.

The fixture now accepts the second player's actual personal pickup after a
scene reunion instead of forcing another approach when it is already complete.
Both inventories and an observed reunion are required for that branch; the
scene, combat, return and coherence checks still run. An unnecessary attempt
to swap positions on the narrow altar has also been removed.

An intermittent delayed-client opening jump remains unresolved: in
`private/coop/regression-suite-bf553370d4b14ebdaa57bdf6449a6e69`, Red fell
before any Dice scene, and the run correctly failed. Successful selected runs
do not establish robust latency coverage or a complete School Two visit.
Full first-school and School Two routes in both orders remain open. Overall
progress remains about 42%, with five of 39 full campaign visits verified.

With the final fixture, the same `6205a876...` build passed both staged altar
orders over the public internet with 40 ms injected delay and 20 ms jitter:
`private/coop/regression-suite-ccd5f7d9a50d40d18315215b42954577/summary.json`.
Blue-first took 56.94 seconds with 628 matching player/enemy and shared-world
states; Red-first took 53.38 seconds with 558. Both finished at 100/100 health
without death, respawn or checkpoint restoration. Minimum Will was 46/52 and
61/58 respectively, confirming ordinary personal resource expenditure.
These are short, explicitly staged component passes, not full routes or a
resolution of the intermittent delayed-jump failure recorded above. No images
were opened during this work.


### Automated route timing and narrow pickup pads

Comparing the retained failed and passing Red-first Dice plans identified the
opening-jump failure above. Both plans and starting positions were identical.
The failed recording contains a roughly 450 ms render frame on both clients.
The host clamps that frame to 100 ms of simulation, while the guest's old test
controller advanced through its entire wall-clock interval. It discarded the
planned jump before emitting a jump input; the authoritative jump counter stayed
zero. This was an opt-in navigation-controller timing error, not an observed
loss of a transmitted jump.

Guest navigation now accumulates local frame intervals capped at the same
100 ms, retaining the remainder between 200 ms input segments. It cannot consume
multiple unsampled segments in one frame. The host still uses its simulation
ticks. Unit checks reproduce the recorded stall and verify ordinary frame-time
remainders; 760 checks pass, with one existing ignored test. Build
`0ae9a8f3e17841ad0f6853610609a90e82b01d1675c4544078d8b31c267db708`
also passes release compilation. Normal gameplay inputs and the network protocol
are unchanged. This does not establish resilience to every independent client
or server stall.

The first integration repeat passed Red-first in 52.92 seconds but correctly
failed Blue-first when the test controller found no unoccupied supported step
aside on the small altar. Both players were alive at 100 health, and the scene
had finished. Evidence is retained in
`private/coop/regression-suite-21aa4c84ae1046d9a2573c2359b83b62`.
The Dice route now gives its first player an explicit ordinary jump back to the
lower landing when the partner still needs their personal pickup. That frees
the altar before the second approach. It still requires actual personal
pickups, scene ownership and reunion, combat, both returns, live health and
matching network states. No positions or resources are written.

The updated fixture passed both staged altar orders on that build with public
internet joining and 40/20 ms injected delay/jitter:
`private/coop/regression-suite-7de4d71d6003494e8cb6dc8a27bbb3f4/summary.json`.
Blue took 57.30 seconds with 638 matching player/enemy and shared-world states;
Red took 60.22 seconds with 673. Both finished with 100/100 health and no death,
respawn or wipe. Minimum Will was 58/58 and 46/49. Blue verified the partner's
actual pickup after reunion; Red exercised the explicit first-player departure
before the other pickup. These are staged components, not complete visits.
Routine verification consumed compact reports; no images were opened.

The following normal-entrance Red-first attempt failed after 410.77 seconds:
`private/coop/regression-suite-6b52f53b33144064b9b7b2658464fc6c`.
It reached the real Dice altar and began returning up the stairs, then the
fixture waited for an idle reinforcement roughly 600 units away. Both players
remained at 100 health, but Red had nearly exhausted her Will. That melee guard
wakes within 420 units, and the existing native route excludes idle guards from
its immediate combat policy. The co-op fixture was instead stopping for every
visible guard within 700 units. No completed-visit credit is assigned to this run.

School Two's opt-in Cards tactic and combat wait now follow the active-fight
policy: an idle guard does not stop traversal or attract automatic shots before
the route approaches it. Active enemies and live attack waves still block the
combat wait. Gameplay awareness, damage, inventories and resource costs remain
unchanged. Full-route verification is still required.

The targeting filter retains all bodies in collision tests. A new regression
checks that an unselected idle guard still blocks aim at an active foe behind
it, while a visible foe to the side remains selectable. The expanded unit set
passes 761 checks, with one existing ignored test.

Build `1c1491c5ad8ddd1bd67487e89ee4b3e000e7fd008fa4c9a82ca864c23e77023a`
passed the 761 unit checks and release compilation. Its normal-entrance
Red-first repeat passed the Dice-and-return milestone with 7,458 matching
player/enemy and shared-world states, correct Red scene ownership, both personal
pickups and no death, respawn or wipe. Both players had 100 health at that
milestone. Minimum Will was 55 and 1.1217; resources were not replenished by the
fixture. Evidence is `milestone-dice.json` in
`private/coop/school2-route-dd80107b99f746578573fb52a5bc3a05`.

The broader run then operated the gym lever and climbed the bleachers, but
failed after 681.73 seconds on the jump from the upper seating toward
`(1600, -2275, 440)`. Red briefly landed, slid off, took fall damage and could
not plan back up from the gym floor. The recorded jump input was emitted;
this is a separate landing-control problem still under investigation. The
suite remains failed in
`private/coop/regression-suite-d6f977d6fec04faca6112a843c6ad50b`.
There is no gym-completion or full-visit credit. Full School One and School Two
routes in both orders remain open; overall progress stays about 42%, with five
of 39 full campaign visits verified.

Driver failures now print only the command, phase, error, tick, replan count,
remaining steps and expected position. Full enemy pools and network hashes
remain in the retained private reports, avoiding large terminal dumps during
routine automated diagnosis. No images were opened during this work.

### Automated gym landing regression

`tools/test_coop_gym_jump.py` isolates the gym lever, bleacher ascent and upper
jump in two independent clients. Only its initial spawn on the gym floor is
staged. It uses the real lever and normal movement, requires actual jumps from
both players, checks that both settle on supported destinations at 100 health,
and verifies the scene owner and exact shared player/world states. It grants no
resources and cannot certify the gym quest or a complete visit. Run both orders:

```powershell
python tools/test_coop_regressions.py --exe <built-executable> --data <base-directory> --only gym-jump-red gym-jump-blue
```

The opt-in route controller now brakes during a clear final descent and settles
near the goal before reporting arrival. Previously a brief landing within 42
units could cancel correction while Alice still moved rapidly toward an edge.
Two production-physics regressions reproduce recorded landings with delayed
observations and inputs; blocked approaches and ascending flight retain their
planned controls. Normal player physics and the network protocol are unchanged.
Build `1d28e039a07f2e1beafebe66729b6cb7a0b17b1023de0a21fb17225c0a5ffabb`
passes 763 unit checks (one existing ignored test) and release compilation.

Both staged gym orders pass on that build through internet joining with 40 ms
injected delay and 20 ms jitter. Red-first takes 109.12 seconds with 1,545 matching
player/enemy and shared-world states; Blue-first takes 101.95 seconds with 1,357.
Both retain 100/100 health without death, respawn or checkpoint restoration.
Evidence: `private/coop/regression-suite-c443b5ad57c443fbae61f65e6bd440d9`.
The fixture explicitly navigates the obstructed departure walkway before the
second player approaches, rather than demanding another sideways waiting spot.

That suite then fails the Red-first altar case before any landing control runs:
the client emits a jump edge, but the host body walks off the takeoff ledge with
zero actual jumps. This intermittent delivery/takeoff timing failure remains
open; it differs from the earlier unsent-jump render-stall failure. An unchanged
build repeat reaches the altar safely but times out because the reunited partner
occupies the requested final approach (`regression-suite-49f614cc709040b4837daf9fde4348a5`).
The fixture now accepts a completed reunion only when both clients confirm both
actual personal pickups, grounded players and the ended scene, then finishes
combat from their separate positions. It does not grant a pickup or bypass a
failed jump. Selected passing runs do not establish robust latency coverage.

With that fixture correction, Red-first completes the staged altar round trip
in 52.89 seconds: 532 matching player/enemy and shared-world states, 100/100
health and minimum Will 34/37. Blue-first completes both personal pickups and
the reunion, then fails the partner's return takeoff after 44.75 seconds. The
guest's recorded plan contains a jump, but the host body never jumps; landing
control never runs during that return. Both outcomes remain in
`private/coop/regression-suite-44a717d8ddce4ae998847f6be21ef63b`.
The suite is failed, and no full-route repeat or new completed-visit credit is
claimed. The next regression needs a safe takeoff margin under input jitter.

These component checks use compact reports; no images were opened. Overall
completion remains about 42%, with five of 39 full campaign visits verified.

### Remote takeoff margin

The opt-in remote route planner now requires 100 ms of remaining support before
a planned grounded jump. It simulates the preceding ordinary movement input to
reject takeoffs that depend on a jump arriving at the last safe instant. The
host/native planner retains its zero-margin search. Neither gameplay physics,
the input protocol nor the player's jump rules change.

A production-physics regression reproduces a sent-but-missed jump by extending
the last run segment by 12 ticks. The original plan walks off with zero actual
jumps; the revised plan jumps and lands across the gap for every tested delay
from zero through 12 ticks. All 764 unit checks pass, with one existing ignored
test, and release compilation succeeds on build
`bfaaa28997ede41787d23ed3ba1be43f879b9263470e8edadcdbd0dfe6b677f4`.
This bounded fixture does not establish reliability under arbitrary stalls or
network delay. Full route and two-client evidence remains necessary.

The complete single-player School Two route passes on this build with the
unchanged result: 53,236 ticks, nine jumps, 23 throws, 149 Cards, 170 combat
damage and 27 final Sanity, including both portals and the gated school return.
The co-op world and native School Two regressions also pass. Evidence:
`private/coop-takeoff-single-player.log` and
`private/coop/regression-suite-f1ec346e93334c49a232d560920b4a8f`.

The first internet integration reaches the altar and performs the return jump,
but fails because the reunited partner blocks the run-up. Actual movement is
only a few units before takeoff, while the world-only plan expects roughly 100.
This is retained as a failure in
`private/coop/regression-suite-bbcdfe3d48df44a094b79b6585a10080`.
The altar has no reliable sideways passing place. The fixture now lets the
player nearest the return landing leave first, then the other follows through
ordinary movement. Scene ownership still follows the originally requested
first-arrival order; the recorded return order is separate. A blanket request
to step aside at the staged entrance failed without a waiting spot
(`regression-suite-2350524d88dc493ab2b99e964aae8e8b`), before either player moved.

Compact failure reports now include input and actual body jump counters,
velocity, remaining planned steps and network/frame timing. The retained failed
Blue-first altar return verifies that this report distinguishes one sent jump
from zero actual jumps without opening images or dumping the entire history.

The final fixture on this build passes both staged altar orders over internet
joining with 40/20 ms injected delay/jitter: Red-first takes 48.45 seconds with
467 matching player/enemy and shared-world states; Blue-first takes 45.70 seconds
with 427. Both retain 100/100 health with no death, respawn or wipe. Minimum Will
is 55/52 and 46/49 respectively. The recorded return order happens to match the
original scene owner in these two runs; the alternate role override has a local
logic check but has not yet been exercised by a passing rendered run.

Red-first also passes the staged gym in 104.56 seconds at 100/100 health. Blue-first
successfully lands at `(1600, -2275, 440)` but fails the following planned walking
turn around the railing toward `(1750, -2275, 440)`. That plan contains no jump.
The host diverges from the planned corner, discards its remaining steps, slides
off and takes 42 fall damage. The complete suite correctly fails after that case
(77.73 seconds):
`private/coop/regression-suite-99cc9ee187cb4e578108b4b3f46b2219`.
This is a remaining corner/braking problem, not another missing takeoff. No full
School Two repeat or new completed-visit credit is claimed. Progress remains
about 42%, with five of 39 full visits verified. No images were opened.

### Host test input sampling

The host diagnostic controller previously sampled its planned inputs once per
rendered frame, then held them across that frame's physics steps. A 24-tick
segment could overrun its boundary before the next sample, changing a narrow
turn and causing the controller to discard its remaining plan while still
moving. The host now samples from its current body before every fixed physics
step. Human controls, guest input delivery and production movement rules keep
their existing behavior. This applies only to the explicit local route driver.

Build `8f056d4f67ab9af79d2cfd93b2015afe9a47ba29cc306b181da6b4460e0fc41a`
passes all 764 unit checks (one existing ignored test) and release compilation.
The gym regression exercises the affected corner after the upper landing;
two-client results are recorded separately from these unit/build checks.

All four focused internet cases pass on this build with 40/20 ms injected
delay/jitter (`private/coop/regression-suite-61202b59c5c14eb8b7e0e0a43931480d`):

| Component | Seconds | Matching player/enemy and world states | Final health |
| --- | ---: | ---: | --- |
| Gym, Blue first | 103.22 | 1,484 | 100 / 100 |
| Gym, Red first | 104.55 | 1,469 | 100 / 100 |
| Altar, Blue first | 48.00 | 505 | 100 / 100 |
| Altar, Red first | 46.00 | 451 | 100 / 100 |

Both gym cases include the previously failing departure corner. Both altar
cases include individual Dice pickups, the correctly owned scene, ambush and
return. Neither pair dies, respawns or restores a checkpoint. These remain
staged component results, not a complete School Two visit.

The subsequent normal-entrance Red-first repeat passes the complete Dice
section and return, including 7,463 matching player/enemy and shared-world
states and 139 matching full reports. Both players retain 100 health at that
milestone. The full run then fails after 810.23 seconds at the upper gym's
departure toward `(1750, -2275, 440)`: Red remains grounded at
`(1741.645, -2275.219, 440.031)`, about 8.36 units from the requested point,
while the controller reports `settling landing`. Blue waits on the lower floor.
There is no death, respawn or checkpoint reset; final health is 100/70.
This is a different failure from the former host's fall around the railing.
The cause of the remaining guest stall is not yet established.

The failed suite and its verified Dice prefix are retained in
`private/coop/regression-suite-8b87284ebafd4b89ad0115a9ede05ed6` and
`private/coop/school2-route-7fb7f563e3f745918605f57e3f519293`.
No full School Two traversal or new campaign visit is credited. Overall progress
remains about 42%, with five of 39 full visits historically verified and the
later player-contact changes still requiring their campaign repeats. All
checks in this change used compact numerical reports; no images were opened.

### Gym waiting point beside the swinging lamp

A native replay of the failed School Two report reproduces the stationary
guest exactly. Its target `(1750, -2275, 440)` intersects `gym_light1` at that
phase. Removing that lamp only in the private diagnostic clears the body trace;
removing its rod does not. The normal co-op mover safely holds the lamp against
Alice, while the test controller keeps steering toward its occupied target.
The former waypoint is obstructed for 113 of 600 sampled ticks in a full swing.

The fixture now uses `(1680, -2350, 440)` on the adjacent diagonal walkway for
the departure and return. In the recorded world, three ordinary movement
segments reach this point within the original eight-unit arrival radius without
leaving support. The point remains clear and supported through all 600 ticks of
the lamp's cycle. The focused gym test also waits 600 gameplay ticks after both
players arrive and requires the lamp to advance at least 4.9 seconds, detecting
a blocked waiting position even when neither player takes damage.

This changes test waypoints and assertions only. Game source remains build
`8f056d4f67ab9af79d2cfd93b2015afe9a47ba29cc306b181da6b4460e0fc41a`.
Numerical diagnosis and validation are retained in `private/gym-stall-probe.json`,
`private/gym-stall-candidates.json` and `private/gym-stall-detour.json`.
Two-client repeats and the full-route result are recorded separately below.

Both focused internet orders pass with 40/20 ms delay/jitter in
`private/coop/regression-suite-cabdb662c9a64637ac62471f1d9f1783`.
Blue-first takes 105.83 seconds and records 1,443 matching player/enemy and
shared-world states; Red-first takes 111.62 seconds with 1,559 matches. Both
finish at 100/100 health without death, respawn or checkpoint restoration.
The lamp advances 5.015 and 5.174 seconds respectively during the waiting check.
These staged components retain their limited scope; a full entrance-to-return
repeat is a separate requirement.

The normal-entrance Red-first repeat on this unchanged game build fails before
reaching the gym, after 333.22 seconds. Both personal Dice pickups and the
reunion succeed; Red returns upstairs, but Blue falls during the return jump.
The recorded plan contains a valid takeoff and lateral landing approach. The
driver changes from `navigating` to `settling landing` while Blue is still beside
the landing platform, discards the remaining plan, and falls against its side.
This identifies premature landing control as the next investigation; it does
not establish a verified fix for that separate failure.

Evidence is retained in
`private/coop/regression-suite-311b50cd522d4f03868f7a3122e3ad59` and
`private/coop/school2-route-00c91b7e8c8043998d39d0af8547d81e`, with a compact
movement extract in `private/coop-altar-host-fall.json`. No new Dice milestone,
full traversal or campaign credit is claimed from this failed run. Progress
remains about 42%; no images were opened.

### Preserve the sideways approach before landing

A small production-physics regression reproduces the altar return failure with
two platforms and the recorded sequence of eight movement segments. The
unaltered plan lands safely. With the former landing correction, Alice slows
while still beside the destination and falls against its side at the same
relative collision boundary as the recorded full run. The failing check is
retained in `private/coop-lateral-landing-red.log`.

The opt-in route controller now starts airborne landing correction only when
a downward body trace finds support at the intended landing height. Its existing
clear horizontal approach check remains. Grounded settling, player movement,
network input delivery and single-player controls are unchanged. This retains
the planned sideways input until the body is above the destination.

The regression now passes, as do the existing delayed gym-landing and edge
settling checks. Build
`d5225f0f22b34ed8403b1ac4ba81dcb6e59519b1c1b7762fdfd6df21cefc0e14`
passes 765 unit tests with one existing ignored test. Release and two-client
results are recorded separately from this numerical evidence.

The initial two-unit support-height band passes both staged altar orders
(49.67/48.20 seconds, both at 100/100 health), but fails the Blue-first gym
repeat after 112.45 seconds. Red takes 18 fall damage on the earlier bleacher
jump before eventually reaching the upper endpoint. The suite correctly fails
and does not run its remaining Red-first gym case:
`private/coop/regression-suite-29546387ef0f42daaaaa3e5ad5fad4d6`.
The compact movement extract is `private/coop-gym-ascent-damage.json`.

The delayed-landing regression now also covers waypoints 16 units above and
below the actual support, with both immediate and delayed observations/inputs.
The strict band rejects these valid approaches. The support check now uses the
same 35-unit height tolerance as straight route walking, while still requiring
support beneath the body. Both the lateral-gap regression and the expanded gym
checks pass. All 765 unit tests pass on build
`a2aede11dcb6e9518184a9a3e83da883a1c4ca33356977851f86da0df7645530`.

Release compilation succeeds. All four focused internet cases pass with
40/20 ms delay/jitter on this build in
`private/coop/regression-suite-a4d1c082112d4fc7b4e9ae288b473020`:

| Component | Seconds | Matching player/enemy and world states | Final health |
| --- | ---: | ---: | --- |
| Gym, Blue first | 106.16 | 1,522 | 100 / 100 |
| Gym, Red first | 108.34 | 1,493 | 100 / 100 |
| Altar, Blue first | 49.19 | 486 | 100 / 100 |
| Altar, Red first | 46.86 | 447 | 100 / 100 |

Both gym waits permit 5.190 seconds of lamp motion. The Red-first altar case
also exercises the alternate departure order: Blue leaves first after the
reunion, while both clients correctly retain Red as the Dice scene owner.
Neither pair dies, respawns or restores a checkpoint. These remain staged
components; full entrance-to-return traversal is checked separately.

The normal-entrance Red-first repeat on this build passes the previous altar
return failure and verifies the complete Dice prefix: 7,500 matching player/enemy
and world states, 78 full-report matches, both personal pickups and the correct
scene owner. Both also complete the gym ascent. The full run then fails after
707.19 seconds because the fixture requests a paired endpoint on the narrow
diagonal walkway, where no supported side-by-side position exists. Both remain
alive at 100/87 health. Evidence is retained in
`private/coop/regression-suite-c81752fc0ec24a6dbe74a67eca41ae87` and its
`school2-route-159d4c6ac4ec4f8581bda99e821e674e/milestone-dice.json` child.
This failed full run earns no completed-visit credit.

The upper gym fixture now crosses the complete narrow passage one player at a
time before stepping aside at its far end. The focused gym case also exercises
this shared route and the three-Boojum quest, after its existing damage-free
ascent and unobstructed-lamp checks. It requires both players to finish alive,
grounded at the far endpoint, with no death, respawn or checkpoint restoration.
Results for this expanded component are recorded separately below.

Its first Red-first repeat catches a stale test command after the Gnome scene
and fails after 145.62 seconds. The leader crosses the upper walkway safely;
the scene reunites the waiting Blue player at the far end, but the fixture's
old combat-holding goal sends Blue back toward the former waiting point and off
the platform. Evidence:
`private/coop/regression-suite-2181284e692b4fd289daf626ed30f938`, with compact
movement records in `private/coop-gym-walkway-fall.json`.

The School Two fixture now releases the waiting player's positional goal during
each scene, before reunion. The upper-passage continuation also recognises a
partner delivered by that authored reunion and continues from the far endpoint
instead of sending them back to the start. No player state or physics is
changed. The expanded gym case additionally checks all three scene owners.
Both gym orders and both altar orders are repeated after this fixture change.

The next Red-first attempt retains both players at 100 health through the Gnome
reunion, confirming that the stale-goal fall is removed. It stops after 141.12
seconds when the generic passage helper asks the leader to step aside despite
both players already occupying separate safe pads. Evidence:
`private/coop/regression-suite-9b3c7ca24aa045409f9c69d91b896edd`.
The passage now accepts that authored reunion only after the leader reaches the
endpoint and both players are grounded nearby with their Mushroom awarded.
The focused final-position checks include the existing reunion pad eight units
higher and about 110 units from the route endpoint. Neither arrival is staged
or written by the fixture.

The following repeat completes the upper passage and three-Boojum fight at
100/79 health, but its new scene-coverage assertion fires while `OG1_Warp` is
still starting (157.42 seconds,
`private/coop/regression-suite-0e6071894619470a9272ae66b5783a25`).
`gym_finish()` now waits for that departure scene and a grounded reunion before
returning. The Laboratory quest state alone is not treated as scene completion.

### Keep a safe nearby partner after the gym departure

The extended repeat in
`private/coop/regression-suite-660e039adf1d4193adeb36e9b1a0e208` fails after
233.55 seconds with a genuine co-op scene lock. The Gnome's departure animation
is inactive, the story queue is empty and no transition is pending, but Red
still owns the scene. Both players remain alive at 100/11 health. The subsequent
fight and movement are frozen while reunion repeatedly fails.

A native replay restores that exact world and both bodies. It confirms that no
new reunion position passes the collision/support/hazard checks, even when the
diagnostic search extends to 144 units. The existing partner is already on a
supported, harmless pad about 110 units from the owner. Evidence:
`private/coop-gym-reunion-probe.json` and its matching log. The temporary probe
module is removed from public sources.

Scene completion now keeps an existing nearby partner as a fallback when no
new placement is available. Both bodies must be grounded, with at least 40
units of horizontal separation, at most 128 units total separation, no more
than 32 units of height difference, a clear eye-level sight line, and the same
full-footprint support and hazard checks. This preserves an existing position;
it does not move a body through an obstruction or change ordinary revival.

A unit regression rejects overlapping, distant, hazardous, unsupported and
wall-separated fallback positions. The native world check also runs the
recorded Gnome departure with both scene owners and requires the original owner
position, a frozen waiting partner, safe reunion and released controls.
Validation of the new build is recorded below.

All 766 unit tests pass with one existing ignored test on build
`61d9b30356a5a9f2042508160ce3cff410f0c18839fe6690e00190cb57eb1d8f`.
The recorded native departure releases control; its first final assertion
incorrectly required the unchanged scene owner to have a recovery-sized floor
footprint on the narrow original walkway. The check now preserves the original
owner's grounded, unobstructed pose and requires the larger footprint for the
retained partner, including exact retention of that partner's safe position.

The complete single-player School Two route passes unchanged: 53,236 ticks,
nine jumps, 23 throws, 149 Cards, 170 combat damage and 27 final Sanity; both
authored portals, all quest rewards and the gated return pass without cheats,
recovery or resource refills. Evidence:
`private/coop-gym-reunion-single-player.log`.
The expanded gym fixture now enables ordinary attacks for both players after
the ascent, matching the full co-op route instead of leaving its staged partner
passive during the fight.

The final build,
`5365929dc75b9bfb4008ec5b6af406e37c66b329b5258af2dbf046461d0cdf62`,
contains the same runtime changes with the corrected native assertion. Release
compilation and the complete co-op world check pass. Both recorded gym scene
owners release control, preserve the owner's position, and retain the existing
supported partner exactly. Existing scene, transport, restoration, airborne
handoff and cramped-altar checks also pass. Evidence:
`private/coop-gym-reunion-final-world.log` and
`private/coop-gym-reunion-final-release.log`.

The first internet repeat on this final build fails before reaching the Gnome
scene, after 124.14 seconds. Both players pass the ascent and lamp check at full
health. Red then misses the planned jump from `(1710, -2800, 440)` toward
`(1880, -2940, 440)`, falls to the gym floor and loses 42 health. The driver
correctly rejects a new route from that lower floor. Blue remains at 100 health.
No death, respawn or wipe is used to continue the test, and the suite does not
run its remaining cases after this failure.

Evidence is retained in
`private/coop/regression-suite-f514393988a74110a35c8625852bdc8e` and
`private/coop/gym-jump-46f3e2c43d9a463cb83f20c75a70f5e7`; a compact movement and
plan extract is `private/coop-gym-upper-jump-fall.json`. The new scene fix has
passing native coverage for both owners, but no passing internet repeat yet.
The upper-walkway jump is the next controller investigation. Full School Two
traversal remains open, with no additional campaign visit credited. Progress
remains about 42%; all checks in this work used logs/state without opening images.

### Grounded route feedback at the upper gym walkway

Inspection of the recorded plan corrects the previous description: this upper
crossing contains four walking/running segments and **no jump**. A swinging lamp
deflects Red during a diagonal segment, but the timed driver still turns right
after its allotted ticks, before reaching the corner. The resulting path runs
off the walkway. This is an automated-driver failure before the Gnome scene.

A native replay restores the recorded school state and moves the actual lamps.
Across 25 lamp phases and two input/observation delays, the old timed path falls
in five no-delay cases and stalls in one. Following the planned intermediate
positions passes all 50 cases without losing height. The implemented helper
also passes a separate 50-case repeat. Evidence:
`private/coop-upper-feedback-comparison.json`,
`private/coop-upper-feedback-production-results.json` and
`private/coop-upper-feedback-production.log`.

The test driver now replays each candidate plan to identify routes that remain
grounded throughout. These routes use ordinary walking inputs and advance to
the next waypoint only after observing arrival. Plans involving jumps, swimming,
ledge grabs or unsupported steps keep the existing timed controller. Human
movement and single-player controls are unchanged. A portable physics regression
reproduces the old missed turn around a temporary obstacle and requires safe
arrival with both zero and 12 ticks of input/observation delay; it also checks
that jumps and walking off a gap do not select the grounded controller.

Build `7d4ed880ebcf0dd8e808211e97ed61d724fd32db3e6ca3481f7aef095eff81fa`
passes all 767 unit tests, with one existing ignored test, and release
compilation. The first internet attempt fails before gameplay because the
temporary invite does not resolve (`regression-suite-71dbbcd0057f472a9b5fb495fe00203e`).
A fresh invite connects, but the expanded Red-leading gym test times out after
244.67 seconds (`regression-suite-ad80bca3a18648ae9cf67c3657a7beb0`).

Both players remain at 100 health. The live plan meets the lamp at a different
pose from the original replay: Red stops at `(1773.73, -2834.33, 440.03)` and
blocks its rotation while continuing toward an obsolete waypoint. Lamp three's
clock stays at 101.822 seconds while the other lamps continue. The initial
feedback fix prevents this fall but needs a bounded replan when progress stops.
The exact world, bodies and plan are retained in
`private/coop-upper-stalled-replay-input.json`. No campaign row is credited by
these component checks.

The blocked-state replay reproduces the stall with zero and 12 ticks of delay.
After one second without eight units of progress, a fresh collision-aware plan
walks around the lamp and reaches the destination in both cases without losing
height (`private/coop-upper-stalled-replay-results.json` and matching log).
The driver now discards an obstructed walking plan after that interval once the
body is grounded and nearly stopped. Its normal worker plans again from the
current received world. A second portable physics regression introduces an
obstacle after planning and compares indefinite blocking with successful
replanning under both delays. It requires ordinary grounded movement throughout.

Build `80ead548ae83e0c2e3f5457f03fc3e8d7dbde0297ac5f6b13e1449bd915d5b85`
passes both new regressions, all 768 unit tests (one existing ignored), and
release compilation. In the expanded Red-leading internet test, both players
pass the ascent and entire upper walkway at 100 health. This run reaches the
Gnome and receives the Mushroom, but fails after 239.97 seconds in the following
fight. Two Boojums die; the third stays at seven health while both automated
players hold their positions. Final player health is 100/98. No death, respawn,
checkpoint recovery or skipped fight is used to continue.

Evidence: `private/coop/regression-suite-34990af90ee14824bab5c443baf0b7d5`,
`private/coop/gym-jump-5cf6c99f8410476aa86a16e4567682f6`, and the compact saved
input `private/coop-upper-boojum-stall-input.json`. The remaining Boojum is
visible to Red according to the thin sight ray, but projectile clearance and
aiming still need an exact-state check. The full expanded gym test and full
School Two traversal remain open. No campaign credit is added.

Independent remaining cases pass on the same immutable `80ead54` executable:
Blue-leading altar in 47.58 seconds (437 matching shared states), Red-leading
altar in 49.34 seconds (471), and the expanded Blue-leading gym in 162.62 seconds
(2,420 matching shared states and 36 full school frames). Both altar runs finish
at 100/100 health. The Blue-leading gym completes all three Boojums, all three
correctly owned scenes, the Laboratory quest state and grounded reunion, ending
at 77/100 health without death or respawn. Evidence:
`private/coop/regression-suite-4b6fcf0b748f413cb2d6e3553852b63f`.

This establishes both upper-walkway arrival orders and one complete staged gym
order. Red's combat stall and both full School Two traversals still need work.
Progress remains about 42%, with five historical full visits that require the
previously documented contact-system repeats before release. No images were
opened during this investigation; temporary native replay modules are excluded
from the public source.

### Automated combat aim around partial cover

The saved Red gym stall reproduces a projectile-clearance problem. Red's
one-unit centre sight ray reaches the last Boojum, but its eight-unit projectile
path hits the environment. Native Blade and Card flights from the eye hit the
world instead of the target. The single-player route's existing centre/top-centre
aim selection also falls back to that blocked centre at this particular pose.

A scan of points inside the actual hitbox finds two exposed upper-side points.
The real projectile simulation hits the Boojum from those points; the first
requires the Blade's ordinary gravity correction. Evidence:
`private/coop-gym-aim-center-results.json`,
`private/coop-gym-aim-replay-results.json` and
`private/coop-gym-aim-sampling.log`. These eye-origin checks establish the
obstruction and candidate aim; rendered tests still need to verify the actual
weapon launch and complete fight.

The opt-in co-op driver now prefers an eight-unit-clear point, sampling exposed
sides inside the hitbox if the standard point is blocked. All other enemy bodies
remain in the collision context, including guards excluded from target
selection. Blade throws use the existing gravity correction. Human aiming,
weapon rules and single-player controls are unchanged. A portable projectile
regression places both a parapet and a post in the way: centre/top-centre aiming
fails, while the selected upper-side point produces a real Blade hit. The
unselected-guard obstruction regression remains passing.

Build `70209472a80c37f070a62a0c27a48b36432e8d106fe209905940ac0d3b6c6c79`
passes all 769 unit tests (one existing ignored) and release compilation. Its
expanded Red-leading internet gym test passes in 168.44 seconds with both
players at 100 health, 2,591 matching shared states and 73 full school frames.
All three Boojums die, all three scenes have the correct owner, the Laboratory
quest opens, and both players finish grounded after reunion without respawning.
This verifies the rendered combat sequence as well as the earlier native aim
checks. Evidence: `private/coop/regression-suite-bf85294757af4e81a72439bff25a7c04`
and `private/coop/gym-jump-0c78033198fb43d9bf0176ef01d26308`.

The Blue-leading repeat fails earlier, after 101.70 seconds: the guest misses
the final ascent jump while its attack command is false. It lands on the lower
floor at 47 health; the host remains at 100. The recorded ten-segment plan has
four grounded run-up segments, and observed positions drift about 17 units from
the expected run-up before an early loss of jump height. The driver remains in
timed navigation throughout. Exact collision and timing replay is still needed;
this is not a passing gym result. Evidence:
`private/coop/gym-jump-04902cb73e8944ba81b75b060077378e` and
`private/coop-gym-follower-jump-failure.json`.

The full normal Red-leading run on the same `7020947` build passes the entrance,
Dice altar and complete gym sequence in one continuous route. Both verified
component reports are retained in
`private/coop/school2-route-a90e419b177747ea9f575711b8db052e`. No player dies,
respawns or restores a checkpoint; final health is 100/37. The run stops after
773.86 seconds because the laboratory fixture asks both players to wait at a
narrow gym-return waypoint with no separate supported spot. The corrected
fixture sends each player through the existing full return passage in turn.
This remains a partial prefix, not a completed School Two visit.

School Two now requests ordinary host checkpoints at its verified Dice and gym
milestones. The opt-in local driver accepts only a neutral save command, invokes
the same validated save operation as the host menu, and acknowledges success or
failure once per command. No checkpoint request is sent by the guest over the
network. Each fixture retains the original save bytes, tick and SHA-256 beside
its component evidence, without requiring menu input or image inspection.

`tools/test_coop_school2_resume.py --checkpoint <checkpoint-gym.json>` copies an
unchanged gym save into an independent profile and uses the normal co-op resume
path. It checks retained player resources and quest state before running the
laboratory, greenhouse, rewards and exit; `--first red` and `--rejoin` select the
scene owner and reconnect check. Resumed results explicitly cover only that
segment. Full entrance-to-exit routes remain required for campaign credit.
Source `4e42226fed20d08b75e38f0c261caa3cfaf55af1b3dc3bb6cc018bd39ebeb1af`
passes all 770 unit tests (one existing ignored) and release compilation. The
normal Red-leading internet run successfully saves both verified milestones:
Dice at tick 50,229 with 100/100 health, and gym at tick 87,785 with 100/28 health,
all three Boojums defeated and Laboratory active. Both retained file hashes
match their receipts. Both players then complete the corrected gym return
passage. The run stops after 989.97 seconds when Red dies in the next Boojum
encounter; Blue is still at 100 health, waiting behind. This is not a full pass.
Evidence: `private/coop/school2-route-f93a34241b8140fb8a8d967fed77864a`.

The test now lets the healthier player lead that ground-level combat approach,
restoring the requested arrival order before the laboratory scenes. The first
gym-resume attempt exposed an overly strict
test assertion: low Will regenerates normally while waiting for live reports.
The opt-in driver now verifies the complete restored world and both players
exactly before simulation, retaining a `route-resume.json` receipt. The neutral
live-client check allows only the existing difficulty-based Will recovery over
the recorded elapsed ticks.

Final source `844ea959eadefaa1b6131d86f3f4211d89872ccef9ce19d17adcb6b59abec95e`
passes all 770 unit tests (one existing ignored) and release compilation. Its
internet resume test verifies the unchanged gym checkpoint before simulation
and 36 matching shared states afterward. Both players repeat the gym return,
and the healthier-player approach passes the previously fatal Boojum encounter
with health 83/28. The resumed run stops after 403.97 seconds at the laboratory
entrance climb: the guest remains grounded at approximately (-60, -2167, 128)
while trying to reach (-64, -1960, 256), after 14 route plans. Neither player
dies or restores a checkpoint. The stair route still needs investigation;
neither the complete resumed segment nor a complete School Two visit passes.
The passing restore component remains separately recorded in
`private/coop/school2-resume-53cc71a254b448ada847ccb95b23a8a1/milestone-resume.json`.

The laboratory fixture now walks the supported stairs from their wider south
entrance, one player at a time. Native replay reproduces both failed jump plans
with exact timing, but a one-tick variation makes each miss. The new 17-point
native walking route reaches the door in all nine cases (three recorded starts
at three input/observation delays), without jumping or falling, in 32–44 seconds.
The production game and its verified `844ea95` build are unchanged.

The route also saves `checkpoint-laboratory-approach.json` normally before the
climb. Use `test_coop_school2_resume.py --resume-point laboratory-approach` or
the suite's `--school2-resume-point laboratory-approach` with that unchanged file
to avoid repeating the gym return. Both clients verified exact restoration of
the new checkpoint before the Blue-leading test.

Rendered internet tests independently confirm that Red and Blue reach the
upper laboratory doorway by walking. Neither full passage passes: the leading
player dies fighting the entrance Diamond while the partner is still below.
Red starts with 28 health; the Blue repeat starts with 83 and leaves the guard
at 3 health before dying. The remaining combat approach needs both players to
participate or the route's normal evasion/loot tactics; more health alone did
not resolve it. No resources were edited and no deaths were accepted as a clean
traversal. Evidence: `private/coop/school2-resume-e35bf359994042669e0421a334192a53`
(335.70 seconds, gym resume) and
`private/coop/school2-resume-0950299e6cdc4b30afc9d03c4ae107ce`
(91.88 seconds, laboratory-approach resume). This remains component evidence,
not a complete School Two visit.

The fixture now regroups both players at the covered upper landing before the
doorway. It uses short paired approaches, with the healthier player closest to
the guard, then restores the requested scene order before the rescue trigger.
Both players complete the stairs and defeat the entrance guard in two successive
Red-owner internet tests. The first exposes another fixture error: a 20-second
landing timeout expires while the Spice Drops scene still owns movement. The
fixture now waits for the actual scene ending and grounded reunion instead.

The repeat passes the rescue, Red-owned Spice Drops scene, reunion and Jumbogrow
pickup, without death or edited resources. It also saves an unchanged ordinary
`checkpoint-laboratory-entrance.json` after the entrance fight. The run then
fails after 358.27 seconds when Red's slow step aside after the greenhouse portal
times out under fire; the portal transfer itself has completed. Blue remains
in the lower corridor. Greenhouse, reconnection, rewards, exit and the complete
visit remain unverified by this run. Evidence:
`private/coop/school2-resume-f4a0daa42b19416086a0a8a88d0df780` (repeat) and
`private/coop/school2-resume-cd18c27bfc2c4005b61b1e72e43071f2`
(235.47 seconds, earlier scene-wait failure). No screenshots were opened.

School Two now retains ordinary `greenhouse` and `school2-complete` checkpoints
as well. The resume fixture/suite accepts `laboratory-entrance` and `greenhouse`
entry points, validating the saved location, quest and personal story items
before copying the unchanged file into a new profile. Greenhouse resumes
explicitly exclude the already completed Spice Drops scene from their coverage.

The portal test starts the partner's approach immediately after the first
transfer is confirmed, while the first player walks off the arrival pad. Both
actual transfers and supported arrivals are still required. The corridor uses
short paired approaches with the healthier player in front, restoring the
chosen order before the growth scene. Rewards are checked in both inventories;
the test only visits the second pickup position if a reward is still missing.

The final quest checker now verifies the ordered collection, use and consumption
of ingredients, followed by both rewards, in the shared quest and both personal
inventories. Its former assertion incorrectly required ingredients to remain
after mixing consumed them. Rechecking the recorded Red laboratory-entrance
segment passes all history checks, including 8,853 matching sent/restored world
and complete states, 189 coincident reports, both portal directions, Red-owned
scenes, guest reconnection and the authored exit at 100/65 health. Four altered
versions of the real recording (missing Spice, retained ingredients, missing
Red potion, omitted growth phase) are rejected. This is explicitly offline
re-verification; the original process exit remains 1. Evidence:
`private/coop/school2-resume-9209434bfc9e44259feeb2c8c61bf7a0/history-verification.json`.

The final Blue-leading greenhouse-to-exit run passes live in 377.52 seconds,
using the unchanged `844ea95` game build, public internet, 40 ms delay and 20 ms
jitter. Both growth/final scenes belong to Blue, all four portal transfers pass,
and a new guest process preserves resources and progress. Both enter
`skool1_start2` alive at 71/100 health, with no respawn or checkpoint restoration.
All 6,232 comparable sent/restored world and complete states match, along with
84 coincident full reports. Evidence:
`private/coop/school2-resume-fc769a3353a84f5c80859b195cec789e/result.json`.
All owned processes close and no screenshots were opened. The complete School
Two traversal in both orders and the remaining recovery matrix are still open;
these checkpoint segments do not complete a campaign row.

### Compact automated failure evidence

Rendered route fixtures now retain `recent-movement.json`: at most ten seconds
and 80 observations per client from the current visit, containing movement,
health, route phases and network/frame timing. The regression runner reports
peak delays with their ticks and links the last command's saved movement plan.
The complete history remains available separately. The new diagnostic helper
is included in the suite's fixture hashes. No screenshot inspection is needed.

The completed Red-first full-route attempt on unchanged build `844ea95` fails
after 243.73 seconds while returning from the Dice altar. Both players have
their personal Dice, the scene completes, and the guest then falls against the
return landing and dies. This earns no completed component or full-visit credit.
Evidence: `private/coop/regression-suite-1f916c6b56054e5f83b15803a6b819df`
and `private/coop/school2-route-a7728dd8cd934394bfd070dc35e39851`.

The new compact report was checked against that actual recording: 65,435 bytes
instead of its 80,082,766-byte full history. It identifies the guest's 799.6 ms
oldest pending input at tick 25,923 and 667.5 ms round trip at tick 25,965,
alongside `guest/route-plan-140-2.json`. Empty histories, missing timing, visit
changes and the sample limit were also checked. The offline report is labelled
`failure-diagnostic-reverification.json`; the original failed result is retained.

A private headless replay uses the actual map and recorded jump plan. Its full
24-tick run-up and an 18-tick run-up land; shortening it to 12, 6 or zero ticks
misses the same side of the platform, with both immediate and delayed views.
This supports investigating compressed movement during the recorded input
delay spike; it is not an exact packet replay or a verified networking fix.
Nearby take-off candidates also fail some small position/timing variations, so
the shipped route and game controls remain unchanged. The private replay is
not included in production sources. Evidence: `private/coop-altar-compressed-runup.json`
and `private/coop-altar-candidate-grid.log`.

The complete School Two test now permits 1,740 seconds: recorded successful
sections total about 27 minutes. Individual movement, combat and scene
watchdogs remain unchanged. This does not turn partial sections or a timed-out
run into a passing full visit. Estimated total progress remains about 42%.

Fresh internet checks on the unchanged game, with 40/20 ms delay/jitter, pass
in `private/coop/regression-suite-a7c2ead28bca40f69e8d6ac20ee92d71`:
presence/contact/effects in 55.39 seconds, Blue-first altar in 47.30 seconds and
Red-first altar in 49.19 seconds. The presence check confirms 36-unit player
separation, an unchanged stationary partner, both models and rendered particles.
Both altar runs finish at 100/100 health with their correct scene owners and
individual Dice. All three automatically produce the bounded movement report;
the helper's recorded hash matches the tested file. These focused passes do
not clear the separate full-route timing failure above. All owned windows close,
Anode is released, and no screenshots are opened.

### Preserve guest movement through delayed packet bursts

The old host input latch preserves button counts but replaces held movement
with the newest packet in a poll. A new regression sends a run-up, jump and
turn together: the previous implementation immediately uses the final turn,
so the jump never receives its preceding run-up. The failing test is retained
in `private/coop-input-timeline-red.log`. This proves that input-order defect;
it does not reconstruct the exact packets of the earlier full-route failure.

Protocol revision 5 adds relative sampling times to guest controls. A bounded
sender outbox keeps unsent changes during backpressure, merging only identical
controls. The host preserves their intervals on its existing 120 Hz simulation
clock. Late packets cannot skip whole intervals that were never simulated;
fractional rounding is retained so ordinary jitter does not accumulate delay.
Both queues are bounded, counters and clocks are validated, and the existing
500 ms input timeout cancels queued edges as well as held movement. Visit and
connection changes reset the queues. Local controls and saved world formats
are unchanged; guests still cannot provide authoritative positions or state.

Optional reports expose received/applied input sequences, queued sample counts
and buffered movement time. Compact failure summaries retain peaks for these
fields alongside their existing network timings. The new diagnostics are used
to check that a fix for input ordering does not create growing control latency.

Build `3c10e1aee105f1665f0035c81233e6844f6dc6b2259cd8c6e78df44febafc075`
passes 776 unit tests with one existing ignored test and release compilation
with five existing warnings. Six new regressions cover ordered bursts, an
actual physics jump across a gap, sender backpressure, stale-input cancellation,
sustained jitter without accumulating delay, and invalid clocks/counters or
queue floods. Existing reconnection, level-change, single-player and save checks
remain passing. Logs: `private/coop-input-timeline-tests-final.log` and
`private/coop-input-timeline-release.log`. Rendered verification follows separately;
these checks alone do not clear the full School Two failure or complete a visit.

The native shared-world and School Two checks also pass on this build
(`private/coop-input-timeline-world.log`, `private/coop-input-timeline-school2.log`).
The subsequent public internet suite, with 40/20 ms delay/jitter, passes in
`private/coop/regression-suite-ceb670ee82a843ab830a17d5928cc613`:

| Case | Seconds | Matching complete/world states | Final health |
| --- | ---: | ---: | --- |
| Contact and particle effects | 56.03 | 624 complete states | 100 / 100 |
| Staged altar, Blue first | 47.62 | 456 | 100 / 100 |
| Staged altar, Red first | 49.89 | 475 | 100 / 100 |
| Normal entrance through Dice, Red first | 445.27 | 7,445 | 100 / 100 |

The normal route completes both return jumps, the stairs and an ordinary
checkpoint, with the correct scene owner and both personal Dice. It also has
209 coincident matching full reports. Its recorded timing includes a 653.3 ms
round-trip peak and 622.8 ms oldest-pending-input peak. The host retains at most
eight queued samples; reported buffered control time has a 303.7 ms median and
381.0 ms maximum. The sender never accumulates an unsent sample in this run.
These measurements expose the added scheduling delay rather than treating a
successful traversal as proof of low latency. They are retained in
`private/coop/school2-route-9515abce48654916818cf23a1cca0d85/input-timing-summary.json`.

This validates the previously failing route section on the new implementation,
not an exact replay of the earlier packet delivery or a complete campaign visit.
Both full School Two orders and its remaining recovery matrix are still open.
No screenshots were opened; the four tests close their owned windows normally.

### Recover simulation time after brief host stalls

The following full Red-first run fails at the Dice return jump in
`private/coop/regression-suite-a73619bc4ed848188403d73f4c147a13` (256.48 seconds,
exit 1). Its complete history records buffered controls growing from roughly
100 ms to a peak of 872.9 ms. The failure and compact timing summary remain in
`private/coop/school2-route-18bc548d83ff4c85b0a567cf142d0bc5`. This overrides any
inference that the preceding partial-route pass made the section reliable.

The host previously capped each elapsed frame at 100 ms before advancing its
fixed simulation. Time above that cap was lost, while guest input sampling used
elapsed real time. A headless regression with 350, 400 and 200 ms render stalls
reproduces 78 permanently lost ticks (650 ms); its failing output is retained
in `private/coop-clock-red.log`. This is a controlled reproduction of the clock
defect, not a replay of the failed internet run's exact frame or packet timings.

The co-op clock now retains short-frame debt and drains it with at most 12
physics steps per rendered frame. A suspension or accumulated debt reaching
the existing 500 ms input watchdog instead discards old time and cancels both
players' stale controls. Accepted button counters remain consumed, so resumed
traffic cannot replay cancelled jumps, uses, attacks or rope actions. Ordinary
fresh controls resume normally. Presentation keeps its existing elapsed-time
cap; the single-player clock and saved-world formats are unchanged.

Reports include remaining simulation debt, and compact failure reports keep its
peak. Automated regressions cover bounded catch-up with real input scheduling,
long-pause recovery and cancelled-edge handling. Live validation of this change
is recorded separately below; no new full-visit credit is awarded here.

Build `9e65cb769a20883d181664d6d3a42315ddba1b2808d3318fa05c964730f529e2`
passes 779 unit tests (one existing ignored), release compilation (five existing
warnings), and both native shared-world and School Two checks. Logs are
`private/coop-clock-{tests,release,world,school2}.log`. The internet suite with
40/20 ms delay/jitter passes in
`private/coop/regression-suite-e3a9acdb18664f03b94b3e9963472be7`:

| Case | Seconds | Matching complete/world states | Final health |
| --- | ---: | ---: | --- |
| Contact and particle effects | 55.64 | 600 complete states | 100 / 100 |
| Staged altar, Blue first | 47.88 | 428 | 100 / 100 |
| Staged altar, Red first | 47.97 | 446 | 100 / 100 |
| Normal entrance through Dice, Red first | 444.72 | 7,387 | 100 / 100 |

The normal Dice route completes both return jumps and its ordinary checkpoint,
with 153 coincident matching full reports. Its host control buffer has a
17.3 ms median, 45.6 ms 95th percentile and 151.5 ms observed peak; it ends at
26.1 ms. The observed simulation-debt peak is 34.9 ms, draining to 5.3 ms at
the end. These are periodic measurements, not every frame or a guarantee of
latency on other connections. The new `timing-summary.json` is automatically
written alongside bounded recent movement by the common pair fixture, on both
success and failure, and included in regression-suite results. It retains
sample counts, median, 95th percentile, last value and the tick/epoch of each
observed peak. No screenshot inspection is needed to obtain these results.

A controlled Windows process-stall comparison also reproduces the defect in
two real internet-connected clients. The private probe pauses only its own
host child for nominal 350/400/200 ms intervals, always resumes the same process
handle in a `finally` block, and leaves the guest running. After settling,
build `3c10e1a` fails with a 915.5 ms median and 967.8 ms peak control backlog.
Build `9e65cb7` has zero observed backlog across the 15 settled samples, at most
8.1 ms of fractional simulation debt, and then passes the contact/effects checks
with 739 matching complete states. The comparison exits successfully only when
the old-build failure and new-build pass both occur. Evidence:
`private/coop/host-stall-comparison-943ede69f7164f36ab0f4911bd3bfc55`.

The regression is now `tools/test_coop_host_stalls.py`, records its own fixture
hash and measured pause durations, and is included in the standard regression
runner's default selection. To run it alone in Anode:

```text
python tools/test_coop_regressions.py --exe <built-executable> --data <base-directory> --only host-stalls
```

It checks that delay drains after the stalls, then exercises ordinary movement,
contact and particle rendering. It does not award campaign traversal credit.

The final public fixture passes through the runner in 66.61 seconds in
`private/coop/regression-suite-076d44ff74484e358b036acbcd21efbf`, with 790 matching
complete states. Measured pauses are 350.2, 400.2 and 200.5 ms. All 15 settled
samples have zero control backlog; remaining fractional simulation debt peaks
at 7.6 ms. The runner's fixture hash matches the source copy recorded by the
test. All owned game windows close, Anode is released, and no screenshots are
opened. Full School Two traversal in both orders and recovery coverage remain
open; the overall estimate remains approximately 42%.

### Scene checks and shorter automated repeats

`tools/test_coop_fixture_scenes.py` runs eight headless checks without installed
game assets or a desktop. It is also the regression runner's default
`fixture-scenes` case. The checks reproduce a delayed guest arrival report
while the host has already started a cutscene, require both clients to finish
the scene and land before selecting a waiting spot, and preserve failures for
stuck scenes and missing personal pickups. The previous fixture fails the
same regression before the scene-wait fix.

School Two reunion checks also recognize the engine's existing safe-partner
fallback: after a second scene, an already grounded partner can stay on the
same supported nearby pad within 128 units. This allowance requires unchanged
partner coordinates and bounded height/separation; it does not extend the
ordinary 110-unit bound for newly placed partners. Headless cases reject a
displaced, airborne, overly high or distant retained partner, and mismatched
relocation ticks. The native placement test separately checks support, hazards
and walls. Game movement and placement rules are unchanged by these fixture fixes.

The ordinary `checkpoint-dice.json` saved by a normal School Two entrance run
can now repeat just the gym, including its lever, upper platforms, combat and
Gnome scenes. The checkpoint is copied unchanged into a fresh profile and
verified exactly on restore. Select `--school2-resume-point dice`; this ends
at a new ordinary gym checkpoint and always reports a partial segment.
Other School Two resume points retain their existing routes through the exit.

```text
python tools/test_coop_regressions.py --exe <built-executable> --data <base-directory> --only school2-resume --school2-checkpoint <checkpoint-dice.json> --school2-resume-point dice --first red
```

For compact, read-only monitoring without images or complete histories:

```text
python tools/coop_status.py private/coop/regression-suite-<id>
```

The status reports completed cases, the active case, latest route event, player
health, scene and selected network timings. Terminal failures include a bounded
error excerpt and their full log path. Missing live reports are explicitly
unavailable. Observing status never awards a pass or changes a running test.

The fresh automated suite
`private/coop/regression-suite-c118bc8f7ddd4429867ee55a8460b77d` passes on the
unchanged `9e65cb7` runtime with public internet, 40 ms delay and 20 ms jitter:

| Case | Seconds | Matching complete network states | Final Sanity, Blue / Red |
| --- | ---: | ---: | ---: |
| Eight headless fixture checks | 0.16 | Not applicable | Not applicable |
| Staged altar, Blue first | 47.80 | 427 | 100 / 100 |
| Staged altar, Red first | 49.95 | 495 | 100 / 100 |
| Unchanged Dice checkpoint through gym, Red first | 323.30 | 5,259 | 100 / 100 |

The gym repeat has 99 matching full local reports and saves an ordinary gym
checkpoint at tick 85,783. Scene ownership, frozen waiting partners, reunion,
personal Dice, no deaths/wipes and shared state checks pass. Fixture and frozen
executable hashes match after the run. All owned windows close and Anode is
released; no screenshots are opened. Locked Rust tests pass 779 with one
existing ignored test, the release build succeeds, and all seven source-package
tests pass. These are component results; full School Two runs in both orders
and its recovery matrix remain open. Overall completion remains about 42%.

### Return-school reward gates

Build `005d643` connects each player's carried Star and potion to the shared
return-school quest. Either inventory, including a dead teammate's retained
inventory, can supply a story reward. Opening the Star doors consumes all Star
copies; the original potion performance consumes potion copies only from its
Drinking phase onward. Ordinary personal resources are unchanged. The original
single-player inventory method uses the same logic with one inventory.

The completed potion performance now requests the authored Pool of Tears exit
through the shared scene update. Previously that completion was only checked
by contact traversal, which co-op deliberately suspends during a cutscene.
The ordinary contact path retains its existing completion behavior.

The new focused native check in `src/coop/school_return_check.rs` is part of
`--coop-world-check`. It reproduces the missing Star gate before the fix, and
then exposes the missing potion exit. After both corrections, all 16 gate/scene
checks pass: both first arrivals, both reward-holder arrangements, and a living
or dead waiting partner. They check unchanged personal resources, the original
potion scene, its frozen partner, consumption timing and exact destination.
The bodies and later prerequisite are staged; these are not lift traversal or
network completion results. Evidence is in `private/coop-return-exit-check.log`.
All 780 Rust unit tests pass, with one existing ignored test; the release build
and seven source-package tests pass.

At this checkpoint, single-player `--school-return-check` failed with `Lift did not release`
in its staged Star/lift check on both the previous `9e65cb7` and new `005d643`
builds. This was unresolved here and was not passing preservation or
return traversal evidence. Logs are `private/coop-return-single-player-baseline.log`
and `private/coop-return-single-player.log`.

A new full School Two Red-first run on the unchanged `9e65cb7` build passes the
normal Dice and gym components and reaches the laboratory, but times out in the
rescue encounter at 1,309.41 seconds. Both players remain alive; the laboratory
door blocks the leader and the remaining guards have no line of sight. Its
ordinary laboratory checkpoints are retained in
`private/coop/school2-route-b40b02356a1b49aa9a0160e28b630828`. Full traversal is
still unproven. The complete route's internal watchdog now allows 28.5 minutes,
below the runner's 29-minute limit with cleanup time; individual movement,
combat and scene watchdogs remain unchanged.

### Delayed door feedback in automated routes

The opt-in route driver now sends one opening press per observed closed door,
then waits for the opening target to appear in its received world state. A
300 ms timer alone could repeat the toggle while a guest still saw a closed
door. A headless regression reproduces 13 toggles with 600 ms of delayed
feedback before this correction. The ordinary prompt already distinguishes
open and closed doors; the failure was stale feedback, not its wording.

Pending requests survive aim and route-command changes. They clear after an
observed opening, or a visit change, checkpoint restore, teleport or respawn.
Observation continues while the driver is idle so a later deliberate closure
can be opened again. These are read-only world observations and ordinary use
inputs; manual door controls, collision, quest conditions and network protocol
are unchanged. An unconfirmed press is not retried indefinitely; the existing
route watchdog still reports an unsuccessful passage.

Headless coverage uses the real door update with 0, 200, 600 and 1,000 ms of
delayed feedback, both approach directions and a later deliberate closure.
A separate check covers lost aim and each world-relocation boundary. Existing
manual-use and paired-body collision tests also pass. All 782 Rust tests pass
with one existing ignored test. Logs are `private/coop-door-delay-baseline-test.log`,
`private/coop-door-delay-fixed-test.log` and `private/coop-door-all-tests.log`.

The diagnostic repeat on the previous `005d643` runtime crosses the laboratory
door and completes the rescue, then fails at another rotating door during the
greenhouse return. Its unchanged ordinary checkpoint and text observations
are retained in `private/coop/school2-resume-3b127c33aee24971a0442bb0615a9d84`.
This is an intermittent route failure, not a full visit pass. No screenshots
were opened to diagnose or reproduce it.

The next repeat, on `6d66fdd`, passes the laboratory rescue and reaches the
greenhouse pickup before a reconnect assertion fails. The assertion required
Will to be exactly unchanged while the host continued simulating. Red's Will
rises from 8.7375517 to 8.8342390 over 116 ticks, matching Normal difficulty's
0.1-per-second recovery within 0.000021. Blue's above-cap Will is unchanged.
The same comparison on the guest's earlier tick also matches. This evidence
does not convert the stopped route into a pass; its report remains failed in
`private/coop/regression-suite-446b0db2e2434ed083d27ac4ad8198d7`.

The reconnect assertion now permits only the expected simulation-time recovery
for the saved difficulty and health, with a 0.002 floating-point tolerance.
Health, equipment, collected items, selection, quest items and difficulty still
require exact equality; existing position, scene and checkpoint assertions
remain. Both reconnect reports are retained before validation, including on
failure. Four headless cases cover the observed mismatch, recovery caps and
dead players, unexpected resource changes, and invalid times/values. The
default `fixture-scenes` case now runs 12 tests.
These asset-free Python regressions also run in the Windows checks workflow on
every push and pull request, alongside Rust unit tests. They need no desktop,
screenshots, original game files or network host.

The fresh suite `private/coop/regression-suite-839085182f0d452c88621c9c949f8534`
passes on frozen runtime `6d66fdd`: 12 headless cases in 0.17 seconds and the
ordinary greenhouse checkpoint through School Two's exit in 381.47 seconds.
The route uses public internet with 40 ms delay and 20 ms jitter, Red first,
and a fresh guest process at reconnect. It verifies 6,176 matching player and
shared-world states and 152 matching full school reports. Both players finish
with 100 Sanity; quest rewards, scenes, reunion, no deaths/wipes and the authored
return-school entrance pass. The previously blocked greenhouse return door is
crossed with ordinary inputs. The checkpoint is copied unchanged, and frozen
executable/source identities are rechecked after the run. All owned windows
close and Anode is released. No screenshots were opened.

This is a partial checkpoint segment, not a complete School Two visit. Both
full arrival orders, recovery coverage, complete return traversal and later
campaign visits remain open. Overall completion remains approximately 42%.

### Observatory departure and landing regressions

The return observatory lift could not leave its bottom landing: the support
test gave the static floor priority even when the floor and lift were at the
same height. A straight vertical ascent now picks up those riders. A floor
above the lift still takes priority, and a rotating or sideways platform
cannot recapture someone who has already stepped onto a landing. Obstructions
still stop the mover and every rider together.

Two asset-free regressions cover coplanar departure and a rotating lift just
below its landing. They run with the ordinary Rust tests in GitHub checks.
The original departure logic fails the new test. An initially broader fix
failed the existing native spiral-landing check; the narrower correction
passes both arrival orders, as well as the obstruction and paused/restored
moving-support checks. All 784 Rust tests pass, with one existing ignored test.

`--coop-world-check` now includes eight integrated observatory cases: Blue or
Red arrives first, watches or skips the original Star scene, and continues
directly or through repeated snapshot round trips. The lift waits for its
partner; the scene retains its first arrival and freezes the waiting player;
both bodies reach the top separately, without damage, respawn, checkpoint
recovery or potion consumption. Only starting positions and quest rewards are
staged. These checks do not certify an entrance-to-exit or internet traversal.
Evidence is in `private/coop-return-lift-refined-world.log`; all eight cases
and the existing spiral landing checks pass without opening screenshots.

The eleven single-player Star/potion scene cases now pass, including the
actual observatory ascent, watched/skipped scenes, paused clocks and restored
shots. The following full return route exposed a separate entrance navigation
blockage at approximately `(-1895, 2854, -471)`. The unchanged `6d66fdd` build
reproduces the same blockage through `--school-return-chain-check`, so full
return traversal remains a separate open check. Logs are
`private/coop-return-lift-fixed-sp.log` and
`private/coop-return-route-baseline-chain.log`.

The return route at that checkpoint planned ordinary movement and jumps down from the raised
entrance edge instead of holding a straight walking input there. On frozen
runtime `a9eaf40`, it crosses that obstruction and continues through the theatre
approach, but later falls and blocks on the library approach at approximately
`(-1583, 2983, -550)`. `--school-return-check` therefore still exited with failure;
the eleven scene checks pass, but neither complete return traversal nor the
School Two/return chain is certified. See `private/coop-return-lift-final-sp.log`.

The compact runner's `fixture-scenes world` suite passes on that same frozen
executable: 12 Python fixtures in 0.16 seconds and the native paired world
checks in 12.41 seconds. Its machine-readable report is
`private/coop/regression-suite-a29d6576f7cf44309b6a093256621e9a/summary.json`.
The locked release build, seven source-package tests and source audit pass.
Overall completion remains approximately 42%; no campaign row is added by
these focused checks. No desktop or image inspection was needed.

### Return-school route automation

The single-player return replay now uses the existing funded combat driver,
clears active threats at ordinary route stops, and runs through the exposed
library doorway before fighting inside its supported corridor. Legacy Blade
throws and waiting on the doorway edge let Boojum waves knock the replay off
the route. Combat, damage, resources and world collision are unchanged.

Spiral boarding now watches actual support and releases movement once Alice
has landed on the lift. A fixed lower-floor target made the previous replay
chase that point while the lift rotated, repeatedly replanning after falling
off. The new School query only observes support; the replay still supplies
ordinary walking/jumping inputs and fails if it cannot board within its limit.

On frozen runtime `01c1d34`, `--school-return-check` passes all eleven saved
Star/potion cases and both complete return routes. Watched scenes finish in
24,222 ticks and skipped scenes in 22,187. Both routes finish at the authored
Pool entrance with 100 Sanity, after 6 damage, 8 Blade throws and 14 jumps;
normal earned drops restore health. Inventory carryover, spending the Star
and potion, and absence of unintended traversal teleports are asserted.
The compact report is
`private/coop/regression-suite-d9b6ebe78917493bb09c2b7c6e491910/summary.json`
(45 seconds, no desktop or screenshots).

The compact runner now accepts `return-sp` and `return-chain` for headless
single-player checks, and `return-blue` / `return-red` for the two-client
internet route. The co-op fixture starts at the authored return entrance with
the visit's normal loadout, takes narrow passages in turn, checks shared
boarding and both original scenes, and requests an ordinary host checkpoint
at major stops. It compares authoritative player/world states, verifies scene
ownership and frozen partners, and rejects unexpected deaths or checkpoint
recovery. Its intended end is the shared Pool entrance; the new fixture's
existence is not a passing full-visit claim.

`return-resume --return-checkpoint <file> --return-resume-point <stop>` repeats
an unchanged saved segment in fresh profiles. Stops are `entrance`, `theatre`,
`spiral`, `observatory-approach` and `observatory`. Saved bytes are hashed and
copied unchanged; startup checks positions, personal resources and quest
state. Resumed results are explicitly partial. The default public internet
connection, 40 ms delay and 20 ms jitter still apply to rendered cases.

Two additional headless fixture tests cover the completed Star presentation
clock continuing, then capping, while a guest reconnects. Scene restarts,
reward resets and unexpected clock changes still fail. The fixture suite now
contains 14 tests and remains part of the Windows checks workflow.

The actual School Two-to-return single-player chain also passes on `01c1d34`.
Return entry preserves its earned 27 Sanity, 17.17661 Will, selected weapon,
18 collected items, Star and potion. Both watched and skipped return routes
reach the Pool exit, with normal pickups restoring health. The compact suite
`private/coop/regression-suite-9c4e4acd21c54c87a83937f01d9944e0/summary.json`
passes 14 fixture tests, the paired native world checks, and the 328.69-second
chain. All 784 Rust tests and seven source-package tests pass.

The first full two-client return run reaches the theatre but stalls at an
overly precise corner waypoint. Subsequent repeats use its unchanged ordinary
theatre checkpoint, without editing positions or resources. A completed
theatre-to-spiral segment now includes both players traversing the reward loop,
library approach, and shared spiral boarding/ascent over the internet, but the
following upper-landing route still fails in
`private/coop/regression-suite-f551980260704f2caa124be31923fcf8/summary.json`.
That run remains failed, and no complete co-op return visit is credited.

These repeats also exposed a harness stopping problem near a small step: the
guest could stand eight units above a proposed endpoint after reporting
arrival, and the harness retried the same endpoint eight times. It now rejects
that candidate and tries another supported spot, retaining the final ground,
distance and height assertions. Narrow walkways use single-file passages
instead of assuming there is room for two stopping positions at every turn.

A further repeat from the unchanged spiral checkpoint traverses the upper
library and gap, the Star gate, shared observatory boarding and physical ascent,
then preserves scene and personal resource state across a fresh guest rejoin.
That run still fails at the final stairs: its approximate target height was
eleven units below the supported landing. The failed result remains recorded
in `private/coop/regression-suite-e2f8564dc32e40a2a1d5f563e6661e3f/summary.json`.

The exit replay now targets the observed stair landings and walks up the ramp
with ordinary horizontal input, matching the native route. It retains precise
landing checks and does not change collision, progression or player resources.
The unchanged ordinary observatory checkpoint then passes through the potion
scene and the shared Pool entrance on frozen runtime `01c1d34`, with Blue first,
public internet transport, 40 ms added delay and 20 ms jitter. Both players
finish at 100 health without deaths or wipes; 1,610 player states and 1,610
shared-world states match, including five full school frames. Scene ownership,
the frozen partner, spent quest items and the authored destination are checked.
The compact result is
`private/coop/regression-suite-8b066e4cac994efdbb5d6dcff90d7671/summary.json`
(114.73 seconds). This is an observatory-to-Pool segment pass, not a complete
visit or a reconnect test. Full fresh co-op return runs in both arrival orders
and the recovery matrix remain open; total progress stays approximately 42%.
All checks in this work used text reports without opening screenshots.

The next fresh Blue-first return attempt reaches the spiral lift but fails
during its ascent, with both riders alive at 83/95 health. Its ordinary saved
spiral checkpoint is retained in
`private/coop/school-return-e69669c6276b4faebc5235d83393e39e/checkpoint-spiral.json`.
Replaying both that failure and a closer-boarding attempt against the real map
confirms supported riders and a valid next support, but a carried rider's next
position intersects solid shaft geometry. The shared lift correctly stops;
collision and platform support do not need relaxing. The private diagnostic
is `private/return-lift-replay-results.json`, not a complete gameplay test.

The paired lift replay now handles a blocked rotation at any height. It walks
one rider toward an observed supported neighbor with room for their partner,
then releases movement when carrying resumes. Lower-shaft corrections move
outward; upper-lip corrections can move inward. Corrections remain bounded and
use ordinary movement inputs. The old replay only adjusted near the top and
could therefore wait forever at a lower obstruction until its watchdog failed.

Repeating the unchanged spiral checkpoint on `01c1d34` now passes through both
shared lifts, the original Star and potion scenes, fresh guest rejoin, and the
authored Pool entrance. Two lower-lift clearance steps were actually exercised.
Both players remain alive and finish at 98/100 health after ordinary pickups;
5,305 player states, 5,305 shared-world states and 420 full school frames match.
The 325.97-second result is
`private/coop/regression-suite-08030788f20949bd9d1e62d80f1acd27/summary.json`.
This certifies the resumed spiral-to-Pool segment only. Full fresh return runs
in both arrival orders and the recovery checks are still required.

The fresh Red-first attempt clears the entrance and theatre, then fails on the
return from the reward walkway. Its long running plan overshoots the narrow
lower stair supports and leaves Red on the floor below. The return fixture now
uses the first visit's slow walking steps on those supports before planning
the short upper jump. It still completes the passage separately for each
player, with the upper arrival east of the waiting partner.

Repeating the unchanged ordinary theatre checkpoint from that failed run passes
through the whole remaining visit with Red leading. Both shared lifts, Star and
potion scenes, fresh guest rejoin and the authored Pool entrance pass, with no
deaths or wipes. Final health is 100/77; 11,375 player states, 11,375 shared-world
states and 16 full school frames match. The 663.44-second result is
`private/coop/regression-suite-004e37da10d443b5ab572ca3fcea71e8/summary.json`,
using the same frozen `01c1d34` runtime, public internet and 40/20 ms delay/jitter.
This is a theatre-to-Pool checkpoint run, not a fresh full visit or death test.
Total progress remains about 42%; fresh full runs in both orders and the
return recovery matrix remain open.

The return-school recovery checks now pass in both death orders on `01c1d34`.
`tools/test_coop_school_return_recovery.py` starts two clients at the normal
return entrance, leaves one player outside the encounter and walks the other
into actual guard attacks. Both victims spectate their moving partner for
exactly 1,200 host ticks, return with 25 health at a clear position 48 units
away, retain their personal inventory and regain their own camera. A subsequent
both-dead wipe restores the ordinary starting checkpoint's resources, positions,
quest flags, machinery and enemies. The return quest's restored timer is checked
against the actual host ticks since respawn; ordinary resumed ticking must not
be mistaken for retained pre-death progress. Replaying the original failed
comparison also confirms that a stale timer or changed quest reward is rejected.

Both cases pass through the compact runner as `return-recovery-blue` and
`return-recovery-red`, in 89.80 and 96.22 seconds. Evidence is
`private/coop/regression-suite-54669cb7e3c645f284c56d2bfc0fd856/summary.json`:
1,233/1,351 matching player and shared-world states, and 23/61 matching full
school frames. Public internet adds 40 ms delay and 20 ms jitter per sender.
Anode was occupied, so these automated rendered runs used the user's permitted
PC fallback. Both clients closed normally; no screenshots were opened.
This completes the return visit's recovery component, not its full campaign row.
Fresh route attempts still expose automated entrance/corner traversal problems;
the latest proposed corner route remains under test. Overall credit stays 42%.

The completed Red-first recovery recording also supplies return-area targeting
evidence without another playthrough. `tools/check_coop_school_return_targeting.py`
checks every recorded active host guard decision where both players are alive,
visible and separated by more than 30 units in target distance. All 58 Blue-nearer
and 14 Red-nearer samples select the nearer player. Both roles have observed
damage intervals with the farther player's health unchanged. Deliberately
reversing the recorded selections is rejected. Source:
`private/coop/school-return-recovery-0bd3f6a852284ed393f8b32490dfed2c/`;
compact result: `private/coop-return-targeting-recorded.json`.
Target selection is transient host AI state, deliberately omitted from saved
and replicated `Opponents`; the source run separately compares the resulting
player and shared-world states received by the guest. This does not establish
independent collection/healing of the same consumable by both players. Applying
the existing personal-reward checker to the earlier spiral-to-Pool recording
does not provide that evidence, so the pickup check remains open.

A later full-route attempt still fails at the library corner, but its reports
also expose roughly 370–393 ms of retained guest-input delay while round-trip
time is near 220 ms and host simulation debt is under 10 ms. A deterministic
input test reproduces the underlying timing problem: after a brief 250 ms
packet delay, a long idle and a return to 50 ms delivery, new movement still
waits 250 ms. The timeline retained the earlier burst's offset indefinitely.

An unchanged idle can now confirm its timeline through the latest received
sample once the host has actually applied 250 ms of neutral controls. This
requires an empty queue, no pending button edges and no movement, swimming,
vertical input or held attacks. Actual simulation time is not advanced. Short
pauses, active controls and queued actions retain their timing. The regression
now applies fresh movement within its 100 ms bound; tests also retain the
run-up/jump order and the existing delayed crossing of a real physics gap.

Build `eb3b335` passes all 786 Rust tests (one existing ignored test), the locked
optimized build, 14 Python scene fixtures and the native co-op world check.
The two-client public host-stall/contact/effect check passes in 64.91 seconds:
controlled 350/400/200 ms stalls, zero buffered control delay in all 15 settled
samples, less than 7.33 ms simulation debt, 36-unit player separation, stationary
partner unchanged, both Alice models visible and actual particle/effect draws.
All 759 comparable player states match. Evidence:
`private/coop/regression-suite-c29e2b24a5104473a2202d1aec5cc53f/summary.json`.
The preceding attempt failed before gameplay because the fresh invite address
could not be resolved; its failed result is retained in suite
`374926f6e773457287ccb42473344c0a`. Neither result proves the complete return route.

Both return recovery orders pass again on `eb3b335`, in 89.26/95.91 seconds,
including real deaths, moving spectator cameras, exact ten-second/25-health
respawns and the shared saved-state wipe. The runs match 1,224/1,383 player and
world states and 53/35 full school frames. Evidence:
`private/coop/regression-suite-3492bab653ec43ce808973ebd7a122f4/summary.json`.
The new Red-first recording also passes the targeting verifier with 55
Blue-nearer and 15 Red-nearer samples and damage examples for both roles
(`private/coop-return-targeting-eb3b335.json`). No screenshots were opened.
At that checkpoint, full-route verification and independent pickup healing were
still open. The subsequent complete results follow.

### Return-school full traversal in both orders

Both fresh entrance-to-Pool runs pass on `eb3b335`, using separate public internet
clients with 40 ms delay and 20 ms jitter per sender. Blue leads in
`private/coop/school-return-d39328d7967a4f53a567869cc51210c4/` (707.39 seconds),
and Red in `private/coop/school-return-d12b98b34a694686a0ba0a86aa5e9bc6/`
(833.98 seconds). These start at the normal return entrance and use ordinary
movement, attacks and visit loadout; no checkpoint segment or staged position
substitutes for the full route. The fixture now stops on the flat entrance
landing and takes each player through the entire narrow library corner before
bringing the partner forward, avoiding an unsupported paired waiting point.

Both runs verify the library and observatory boarding waits/rides, the intended
single-Alice Star and potion scene owners, frozen waiting partners, safe reunion,
fresh guest rejoin preserving resources/progress, and the authored Pool transition.
Neither run has a death, respawn or checkpoint rollback. Final Blue/Red health is
100/95 and 100/61. All 12,192/14,193 comparable player and world states match, as
do 308/401 coincident full school reports. Median buffered control delay is zero;
95th percentiles are 11.80/8.23 ms, without the former retained idle delay.

The Blue recording also proves both clients observe separate entitlement and
healing from reward `3000003`: Blue 76 to 91, then Red 80 to 95. The uncollected
partner copy remains available. The checker previously rejected the first event
because the waiting player's Will regenerated normally: 0.023335 over 28 host
ticks (0.020001 over 24 guest ticks). It now permits only unchanged Will or the
difficulty-dependent passive recovery over that exact interval. Partner health,
inventory and collection changes, extra refills, depletion and missing copies
still fail. The recorded false failure has a regression test; all 18 fixture
tests pass, including invalid resource changes and recovery caps/difficulties.
Evidence: `private/coop-return-blue-rewards-eb3b335.json`.

Together with the same-build recovery/targeting checks above and recorded
single-player regressions, these results complete the return-school row. The
earlier five rows retain their contact-repeat requirement. Overall estimated
progress is 42.1%, with six combined verification rows and 33 visits still open.
These checks used compact recorded state; no screenshots were opened. The game
runtime is unchanged from the preceding input-delay fix.

### Pool of Tears shared machinery and combat (2026-10-25)

The freely controlled Pool now advances one shared world clock and carries all
living leaf riders together. Its four ride triggers wait for both living players;
waiting does not consume the trigger, and a survivor can depart if the partner
dies. An obstructed leaf holds its own motion while the rest of the level keeps
running. Recycled or newly appearing train leaves cannot carry a body from their
previous inactive position. Original single-Alice performances retain their
existing controller.

Falling rocks now evaluate both living bodies in one update and route damage and
impulse to the struck player. Released pusher ants join the co-op weapon targets
and choose from both players using the existing visibility rules. Environmental
ant deaths are included in the personal reward comparison.

Build `0ca5fa5df6611cd1c35d4daab208df209019f0dab39cc2315ea1d301a9b35ae4`
passes 38 real-data component scenarios through `--coop-pool-check`: four leaves
in both rider orders with pause/restore; all four authored boarding contacts in
both arrival orders with either death or neither, compressed world/contact
restoration and integrated carry; each player beneath an obstructed leaf;
isolated rock damage/impulse for either player; and each nearer ant target followed
by invisibility-driven retargeting. Logs are `private/coop-pool-native.log`.

The same final build passes `--pool-check` (including single-player scenes,
watched/skipped outcomes, saves and leaf/swimmer checks), `--coop-world-check`,
786 Rust tests with one ignored, and the optimized locked build. Source-package
tests pass. These checks run without opening screenshots.

Pool remains **partial**: full ordinary-input two-client traversal, swimming,
scripted rides/reunions, weapon/reward integration, death/wipe and transition
rejoin still require campaign evidence. This component work does not complete
another campaign row; the overall estimate remains **42.1%**.


### Pool network route automation: entrance and bank segments

Runtime `215f76fbc1df1879cf01fdf001530c3a38d522df2a6534ed05082bbfb0ce1fd7`
retains the preceding Pool integration and corrects only opt-in test steering:
speed limits now cap input magnitude without suppressing small final corrections.
The physics regression covers both walk/run and delayed observation/input.
All 787 Rust tests pass (one ignored), as does the locked release build.

`tools/test_coop_pool.py` drives ordinary movement in two independent clients,
using public internet connections with 40 ms delay and 20 ms jitter by default.
It saves normal checkpoints after paired waypoints, records fixture/build hashes,
and can resume unchanged checkpoint files. Supported waiting positions are checked
by actually walking to them. Stalls, deaths and failed body-clearance checks fail
the run; no state relocation, forced progression or health assistance is used.

The opening scene has independent recorded verification in
`private/coop/pool-route-8179b7c94a654cb785a79a077a25c182/opening-verification.json`:
Blue owns the scene, Red waits, reunion completes, both retain 100 health, and
597 comparable authoritative player/world states match. The resumed final
lower-bank segment passes in `pool-route-edc214e490624434a9643a4c4932e46f`, with
146 matching player/world states and both players at 100 health. Its ordinary
`checkpoint-bank1.json` has SHA-256
`21cc06748e3d92ab02fd2c465132e02c636f1f47fb21ff8bf81cf874eeffcfb7`.

`pool-route-880a167c6415495480d16bd33796205d` subsequently crosses the first
high-bank gap with both players and reaches waypoint 6. The independently
checked prefix has 285 matching player/world states, no respawns/wipes and both
players at 100 health (`segment-verification.json`). Its complete run fails later:
Red catches a ledge at waypoint 7 and the original fixture never sends the
additional jump input that starts a pull-up. The fixture now requests that
ordinary input when an observed hanging player has stopped advancing. These
checkpoint segments are partial evidence; they do not complete campaign row 9.

The pull-up repeat in `pool-route-96ccfe0322f449e5ac25ce0b2003736b` passes paired
waypoints 7 and 8: 774 matching player/world states, both players at 100 health,
no respawns/wipes. Its normal waypoint-8 checkpoint has SHA-256
`34e865e9f2ae250fbc20cafffb2eb8c547758e3f4254e47d65c1663380bd2154`.
The later waypoint-9 approach times out after a scene reunion places Blue near
Red's goal. Both remain grounded at 100 health, approximately 36 units apart;
the fixture still needs to move the waiting player away after that reunion.
The complete bank segment remains failed, with its passing prefix recorded
separately. All checks above used logs/state; no screenshots were opened.

### Hollow Hideaway paired machinery and personal hazards (2026-10-25)

The registered level controller now participates in the host's paired movement,
boarding, targeting, weapon-hit and personal-damage paths. The pond advances once
per world tick and carries both living riders on each of its seven paths. Each
triggered leaf waits for all living players. If either rider is obstructed, that
path retains its pose and clock while unrelated paths keep advancing. The second
leaf's initial turn now carries both riders from its authored boarding pose;
saving during a blocked departure retains that pose and the pending departure.

Fish exposure is saved separately for Blue and Red. The first threshold crossing
owns the original bite scene; its partner and support stay still during the
scene. Only the victim dies. The survivor stays at their waiting position while
the victim enters the existing spectator/respawn flow. Fish water is excluded
from automatic respawn/reunion placement. Bound pickups follow the authoritative
pad pose after movement and snapshot restoration, with collection and healing
remaining personal.

Frozen source `6c11e46a8599a474b0a8ab28fbd4b88f991d056b1cb5dbd839dfbdf88f297178`
passes all 33 `--potears2-coop-check` scenarios in
`private/coop-potears2-native6.log`: fourteen paired pad rides, twelve original
boarding contacts (both orders and either death), two blocked-start/save/release
cases, two nearest-visible guard/damage cases, both fish victims with
mid-contact/mid-scene restoration and exact 1,200-tick/25-health respawns, and
moving pickup restoration/personal collection. These are real-data component
fixtures, not a full ordinary-input two-client traversal.

The same production code passed 787 Rust tests (one ignored), the locked release
build, `--coop-world-check`, single-player transport/save/fish checks at
30/60/144 Hz, and both ordinary entrance-to-Just-Desserts routes with the default
and watched ending. The routes finish at 100 health with eleven jumps, two
throws, twenty-four cards and one pickup, without authored teleport assistance.
Those regression logs are `private/coop-potears2-rust-final.log`,
`private/coop-potears2-world.log` and
`private/coop-potears2-sp-{transport,route,watched}.log`, on source `93d29da`;
the final source differs only in the obstruction test's fixture placement.

Hollow remains **partial**. Full network traversal in both arrival orders,
combat/reward integration, shared wipe, final reunion and rejoin-to-exit still
need campaign evidence. No campaign row or percentage is credited for these
component checks. All validation above used state/logs; no screenshots were opened.

### Pool bank continuation and scene-aware route control (2026-10-25)

The route fixture now checks both the local client's and host's grounded state
and stopping speed before choosing a waiting position. A scene reunion only
clears the waiting partner when that partner blocks the next goal. If the
original scene has already placed both players on a narrow bank in distinct
supported positions, the fixture accepts that paired landing. Ordinary bank
wayfinding uses the single-player route's 20-unit horizontal/48-unit vertical
tolerance, followed by a neutral, supported stop; exact waiting spots still
require their tighter position check. No player/world state is written by these
steering changes.

On frozen runtime `6c11e46`, `pool-route-7a8e56f4ecba41b0a4d65eaf9e0a4704`
passes the unchanged waypoint-8 checkpoint through waypoint 9 and the original
rock scene/reunion: 752 matching player/world states, both players at 100 health,
no respawns or wipes (`segment-verification.json`). The later slope attempt
failed because the earlier fixture accepted arrival while the guest was still
moving. With the stopping check corrected,
`pool-route-cff881d68bf64cffb9b7605beda79d2d` passes the resumed waypoint-9 to
bank-2 segment, including waypoints 10 and 11: 752 matching player/world states
and both players at 100 health. Its ordinary bank-2 checkpoint has SHA-256
`b4b734f9611a04cd9b64e190017305b62f6bd257744149ecc3fb3c7be1f430fd`.
Both runs use the public internet path with 40 ms delay and 20 ms jitter.

The same runtime also passes the complete single-player Pool route to Hollow:
56,884 ticks, 26 jumps, seven throws, 72 cards, 57.864 combat damage, two pickups,
three enemy rewards, 100 final health and no authored teleports. Evidence is
`private/coop-pool-route-reference.log`; this headless route does not certify
two-client traversal. Higher-bank fixtures are still under test, and Pool's
full campaign row remains partial. No screenshots were opened.

The higher-bank driver now gives the ordinary jumps a short running approach,
matching the moving takeoffs in the single-player route. After the second rock
scene, both players leave immediately: stopping to save or waiting for one
player leaves the other in the falling rock's path. The fixture waits until
both have reached the following bank before neutralizing their inputs, and
retains distinct supported positions instead of requiring a place exchange.
It validates the observed rock-scene owner and rejects a resume index that
would skip every remaining waypoint.

The prefix of `pool-route-93df380f927e4d79b37ad06cc67c33c7` through waypoints
12/13 has 268 matching player/world states and both players at 100 health.
On the subsequent corrected fixture,
`pool-route-83cf17bb073a4d8d90cc115d76611382` passes from that unchanged
waypoint-13 checkpoint through the running jump, Red-owned second rock scene,
frozen partner/reunion, paired escape and bank-3 waypoint 16. Independent prefix
verification records 900 matching player/world states, both players at 100
health, and no respawns/wipes. Its ordinary waypoint-16 checkpoint has SHA-256
`ee8e17c3e116b4d9939968e1ab6b6b6c1771f6c0d8180945f318725ffe1a3fcf`.
Both prefixes retain `segment-verification.json` alongside their full histories.

The last full attempt **fails later at waypoint 17**: Red stalls near
(-2180, 3104, 1263) approaching (-2128, 3104, 1248); both remain alive at 100
health. Bank 4, later transport/swimming and the full visit remain unverified.
The fixture uses ordinary client input and normal checkpoint saves throughout;
the game runtime is unchanged. Test windows were closed and the Anode lease released.

### Duchess paired encounter integration (2026-10-25)

The host now exposes the Duchess to both players' weapons and advances her
existing attack logic once against the living roster. Target selection respects
visibility and death. A grab retains its captured actor across saved snapshots;
only that Alice loses movement/weapon control, receives bites and is thrown.
Death or interruption releases the hold. Entry restoration happens once per
visit, and respawning does not grant another full heal. The first Jackbomb
collector owns the original intro; the finisher owns the original death/reward
scene and temple transition. Cinematic handoff cannot run a single-player combat
tick before the paired simulation resumes.

Each Alice has separate availability and cooldowns for the four healing items.
The native fixture exposed an existing encounter contact bug: a ray to the
item's base could intersect furniture while its upper pickup volume remained
reachable. Both single-player and co-op now use the existing inventory contact
box/occlusion check. Amount (25) and cooldown (10 seconds) are unchanged.
The co-op viewer draws the boss HUD and personal healing visibility. Duchess
and Pandemonium effects now enter the shared particle pass; the single-player
viewer uses the same pass once, without duplicate emission.

Frozen source identity:
`3353e534f00e815151090698ffb4f1722314d2fe2d003fee1d15a4ea7fcd7354`.
Executable SHA-256:
`fe461f125bce4c77b8a8a24084bf51c2a95b098814f8da5ceab8c682576a048c`.

- `--coop-duchess-check`: 22 real-map cases pass, including both actors' targeting,
  invisibility/death retargeting, isolated melee/bites/throws, paused combat,
  exact saved continuation, all four personal healing items, and watched/skipped
  intro/outro ownership, frozen partner, replica and safe intro reunion.
- `--coop-duchess-arena-check`: six production-Arena cases pass in Anode. Each
  actor's captured controls/weapon stay locked while the partner moves; a real
  Blade input defeats the staged one-health boss and binds the finisher's scene;
  the shared effect pass emits 3,717 particle batches per actor's fixture.
- `--duchess-check`: the ordinary single-player entrance/fight/reward/temple route
  passes in 15,061 ticks, 18 throws, 35 Cards and 67.49947 final Sanity, with
  32 boss attacks and 11 dodges. Its scene, pause and restore contracts also pass.
- `--coop-world-check` passes. Locked release build and all 788 Rust tests pass
  (one unrelated ignored test). Logs are `private/coop-duchess-{native-final,
  arena-final,single-player,world,unit-final}.log`.

These are staged component/Arena checks, not a normal co-op entrance-to-exit
route or a two-client internet run. Full traversal, recovery and rejoin remain
open. No campaign row is completed; the total stays 42.1%. No screenshots were
opened for this work.

### Underwater temple paired machinery (2026-10-25)

The temple previously had no paired controller update. Its guide, traps and air
emitters now advance once per world tick. The machinery hook receives stable
Blue/Red slots, including a dead partner, so crushing affects the correct
player's health and cooldown. Both living swimmers receive the original entry
restoration and shell once; respawning does not grant another full heal.
Existing air remains usable while the Watch stops the world clock.

Both escape positions are proposed before machinery commits. If either swimmer
cannot escape, both positions and the shared trap movement roll back. Damage is
personal even when both contact the same clam. The original single-player clock
logic is shared, with its one-player movement resolution retained. The temple
has falling traps rather than a shared passenger ride; its guide does not wait
for a boarding group.

Frozen source identity:
`f21de58f3278876de885f0e94265ae3739e34ccb01e24920da139eead515d30b`.
Executable SHA-256:
`92aec2f4cc5a2b8edaa0448bca676e13d12aa93361bb484c1268b0653fcdeb90`.

- `--utemple-coop-check`: 13 native cases pass. They cover both swimmers' entry
  grants, air contacts with a stopped clock, visual-only bubble exclusion,
  actual brush swimming and drowning, individual crushing with a living/dead
  partner, simultaneous crushing, blocked movement in either slot, saved
  continuation and invalid cooldown rejection. Both exit owners pass watched
  and skipped original scenes, frozen-partner checks, compressed state
  restoration and the garden transition.
- `--utemple-check` passes the existing water, breath, migration, pause, restore
  and 12 scene-outcome contracts. Both full single-player route checks pass,
  with 154 air contacts, no teleport/drowning bypass and 90 health / 100 Will
  carried into the garden. Watched and skipped times are 173.84 and 166.83 seconds.
- Shared-world, Hollow Hideaway (33 cases) and Duchess (22 cases) regressions
  pass. All six production Duchess Arena checks also pass in Anode, including
  personal control/weapon restrictions, scene ownership and particle emission.
- Locked release and all 788 Rust tests pass (one unrelated ignored test),
  together with seven source-package tests. Native logs use
  `private/utemple-f21de58-*.log`; build/unit logs use `private/coop-utemple-*`.

These component fixtures stage hazard and scene contacts. They do not establish
a complete paired swimming route or internet recovery. Full two-client
traversal, drowning/respawn, shared wipe and transition rejoin remain open, so
the campaign row is partial and the total remains 42.1%. No screenshots were
opened; all reported checks use automated state assertions.

### Full paired temple swimming (2026-10-25)

`--utemple-coop-route-check` now follows the whole visit from its normal paired
entrance using ordinary swimming input. The second Alice follows the guide's
air trail one second behind the leader; each independently navigates current
geometry and anticipated traps. The test runs the production Arena, including
the native cast and personal damage. It does not change positions, resources,
guide events or hazards. Skipped runs call the original cinematic skip path.

All four routes pass on source
`c2a598bd44d1dad38827b95d9342bb78bdf4686944abdcd79aff2437ba799ab4`, executable
SHA-256 `b6cd878b7c1fd05b60c3fc0dbb3e33a98941d0d6a21257b53d01a461fbc58434`.

| Leader | Ending | Ticks | Matching restored world samples | Final Blue / Red health |
| --- | --- | ---: | ---: | --- |
| Blue | Watched | 20,861 | 35 | 30 / 48 |
| Blue | Skipped | 20,020 | 34 | 30 / 48 |
| Red | Watched | 20,873 | 35 | 58 / 30 |
| Red | Skipped | 20,032 | 34 | 58 / 30 |

Every route verifies the intended exit owner, a frozen partner during the scene,
both personal shells, no drowning, no death/respawn or team wipe, clear bodies
during traversal and no free-movement teleport. Minimum remaining air is 15.89
seconds. A second Arena restores compressed snapshots every five simulated
seconds and matches the full world, NPC and character state. Rendering runs
periodically in Anode; no screenshots were captured or opened.

Evidence is `private/utemple-route-c2a598b.log` and four JSON results in
`private/utemple-route-c2a598b/`. The 13 component cases pass on the same build,
as do all 788 Rust tests (one unrelated ignored test), the locked release build
and seven source-package tests. This is full traversal in one process. It does
not yet prove delayed internet traversal, death recovery, shared checkpoint
restoration or transition rejoin. Those remain required before completing the
campaign row; total credit remains 42.1%.

### Temple internet test and ceiling regression (2026-10-25)

`tools/test_coop_utemple.py` runs two independent profiles through ordinary
public Host/Join, with 40 ms delay and 20 ms jitter by default. Each client
plans swimming input from its own received world. It checks personal air and
shells, scene ownership, the frozen partner, survival, clear bodies and the
garden transition. Tick-matched hashes now include native residents alongside
players and shared machinery. The fixture needs no screenshots.

The first run on `0e898b0` failed at temple node 81: an upward launch volume
pinned Red against a shallow ceiling until she drowned. An offline replay of
the captured body and world produced zero displacement for all seven steering
directions, even though a nearby body sweep was clear. Host and guest matched
1,440 authoritative ticks, including native residents, so this was a movement
fault rather than a replicated-state disagreement. Evidence is
`private/coop/temple-route-173be7fcd7394689be83b7a0f66ce620/`.

The existing checked collision-skin separation now also handles tangent
ceiling stalls. Walkable floor handling and the authored launch velocity stay
unchanged. A synthetic regression fails before the fix and passes afterward at
nine translated ceiling positions, checking displacement and collision clearance.
The opt-in `LOOKING_GLASS_TEMPLE_REPLAY_REPORT` diagnostic replays the captured
Red body through `--utemple-coop-check` without modifying a live game or save.
Unset it to run the normal component suite.

Fixed source `44335efa33445207fcd830f672cb8dc42f8697ae50b555e092bf920f4d2460bd`
and executable SHA256
`5052c307f16899a7105457fe701518336574d9426cbbdc05e7233a2edbf53a2c` pass:

- The captured stall replay: all seven steering probes move over 11 units in
  one second and remain clear every tick (`private/utemple-swim-replay-fixed.log`).
- All 790 Rust tests, with one unrelated ignored test, and the locked release build.
- All 13 paired component cases, the existing temple state checks and both
  single-player swimming routes (`private/temple-44335ef--utemple*.log`).
- All four full paired routes in Anode, with the same final health, ticks and
  air margins as the preceding table (`private/temple-44335ef-native.log`).

The public repeat on this build also fails. Red dies near the first falling
spike at about tick 4026/node 30, with no drowning hits, while Blue stays at 100
health. It ends before reaching the previously stuck ceiling, so it cannot
establish a complete internet route. The retained run has 591 matching
authoritative player/world/native-resident samples and no hash mismatches.
Observed guest round-trip latency has a 228 ms median and 274 ms p95; a late
host frame reaches 347 ms. The cause of this earlier death and the test swimmer's
response to those timings remain to be investigated. Red-first was not run
after the Blue-first failure. Evidence is
`private/coop/temple-route-baa86d7d5b20435580efedaf32ebf638/` and
`private/utemple-internet-44335ef-blue.log`.

Internet traversal and recovery/rejoin remain required for campaign completion.
The ceiling fix and repeatable network fixture are smaller completed milestones;
the total remains 42.1%.

### Full temple traversal through separate internet clients (2026-10-25)

Both first-arrival orders now pass on source
`0c9d40b5907d22224b65cc68959853ddbf0318139320848635709e972ba2ff5c`, executable
SHA256 `30be6c7fa50e468b74544c6eb0c63a6416c1c6db38cd32a1b3001015309c3877`.
The original falling column deals 100 damage; its damage and timing are unchanged.
The test swimmer now prices future hazard contact along an entire swept leg,
not just at its endpoint. A synthetic thin-obstacle regression fails before this
change and passes afterward. Difficult searches run on a cloned collision world
in a background worker. Results from old commands, visits, checkpoint generations,
warps or spawns are discarded, as are stale or currently obstructed legs.

The public Host/Join runs use separate processes and profiles, with 40 ms delay
and 20 ms jitter per sender. Each client generates ordinary inputs from its own
received state. Both start at the normal entrance, traverse the entire level,
watch the intended first player's original exit, and reach garden epoch 2 alive.
They retain their own shells, have no drowning, respawn, wipe or traversal warp,
and keep the waiting partner still during the single-Alice scene.

| First arrival | Final Blue / Red health | Matching sent/restored ticks for each of player, world and native-enemy state | Evidence folder under `private/coop/` |
| --- | --- | ---: | --- |
| Blue | 40 / 40 | 3,050 | `temple-route-bf02b1fdcb864bd48eb0e674a2dfb425` |
| Red | 58 / 50 | 3,032 | `temple-route-9a7b7da7eb1d43a69050b01fb96381e5` |

All compared hashes agree. The lowest observed remaining air is about 6.03
seconds in the Blue-led run and 11.43 seconds in the Red-led run. Slow searches
reach 533 ms in the background; preparation remains below 18 ms, with a median
near 3–4 ms. The Blue-led clients' frame medians are about 16 ms and p95 about
24 ms. The largest host frame occurs during garden loading, so these timings
are not a claim that level transitions have no loading pause.

On the same build, all four native paired routes, both single-player swimming
routes, the 13 paired component cases and the temple state checks pass. Native
Blue-led runs finish at 30/48 health; Red-led runs finish at 38/40. All 791 Rust
tests pass, with one unrelated ignored test. Logs use
`private/temple-0c9d40b*`, `private/utemple-internet-0c9d40b-*.log` and
`private/temple-async-*.log`. No screenshots were captured or opened.

`tools/test_coop_utemple_recovery.py` separately tests real drowning, both
spectator roles, 25-health revival after exactly 1,200 ticks, shared checkpoint
restoration and entrance rejoin. It does not relax the clean-route assertions.

The first recovery run ended in an unintended team wipe when the survivor hit
the falling column during the first countdown. That failed evidence remains in
`private/coop/temple-recovery-cfccff4aa3b74ff2b6a66db017ab58d0/`. The fixture no
longer reissues the survivor's already-active follow command when stopping the
victim; doing so discarded its navigation state unnecessarily.

The corrected fixture passes on the same `0c9d40b` executable and public
delay/jitter settings in
`private/coop/temple-recovery-6cf854d537f74d58a7bec8c26fb3d802/`:

- Red drowns at tick 3,719 and respawns at 4,919; Blue drowns at 8,279 and
  respawns at 9,479. Both clients agree on these exact 1,200-tick intervals.
  Each victim's camera follows the moving survivor and returns to that victim
  at revival. Both revive swimming at 25 health, in clear space near the partner,
  with their weapons, collection records and personal shell preserved.
- A subsequent ordinary drowning wipe restores both at tick 12,020. The actual
  saved entrance resources, placements, guide, node, events and machinery state
  return; both have their saved 100 health and the restored world clock resumes.
- A new guest process rejoins the restored entrance without moving either player
  or changing resources or temple progression. All 1,712 comparable sent/restored
  player, world and native-enemy hashes match.

Logs are in `private/utemple-recovery-0c9d40b-repeat.log`; the exact recovery
fixture source is retained with its reports. All owned clients/helpers close
and Anode is released. An entrance rejoin does not establish transition rejoin.
That and the remaining combined-evidence requirements must pass before this row
gains completion credit; the overall estimate remains 42.1%.

### Temple exit rejoin and reusable focused tests

On the same `0c9d40b5907d22224b65cc68959853ddbf0318139320848635709e972ba2ff5c`
build, `tools/test_coop_utemple.py --rejoin` stops by the opened final wall,
defeats the following WaterSnark through ordinary attacks, restarts the guest,
and completes the authored exit. Exact idle resource/position checks run after
combat, while guide/mover checks allow their normal elapsed time.
`--checkpoint PATH` uses an existing isolated exit checkpoint and explicitly
reports an exit segment, not a full traversal. No screenshot is required.

The focused public-internet repeats pass with 40 ms delay/20 ms jitter:

- Red first: `private/coop/temple-route-0634a17a34e346c49dc76ddd1191a671/`,
  189 matching player/world/native-enemy states, 35.4 seconds.
- Blue first: `private/coop/temple-route-6592cfa519464decae6ce060208b3cd4/`,
  210 matching states, 37.2 seconds.

Both preserve personal resources, placement, shells and exit progress across a
fresh guest process, select the intended scene owner, freeze the partner during
the original scene and reach Garden with 48/20 health without deaths or warps
during free traversal. Their exact fixture hash is
`9fae5954e1d0316465c964b0b676b6299bfee5d9bd4cafc1fb34aeba108824c8`.
The checkpoint hash is `6151f88c2cbe3d3d46c06e5640e499fa2b742f21ee2c21eeb4e295b1c5ee0158`.
It packages the unchanged tick-20,007 player/world report from the ordinary-input
internet route `private/coop/temple-route-75b5f382ad98470f92ac6540718dcb18/`;
provenance and exact frame equality are recorded in
`private/temple-exit-checkpoint-provenance.json`. This is captured-state component
evidence, not a claim that an entrance-to-exit reconnect route passed.

Three preceding failures remain recorded: Blue-led traversal died near the
falling panels at tick 10,307 (`temple-route-bc15a42da02541e4a941efbfeeae1223`);
the first Red-led exit attempt saw a legitimate 10-health bite/knockback during
the idle reconnect assertion (`temple-route-75b5f382ad98470f92ac6540718dcb18`);
the next Red-led traversal died near the first columns at tick 4,062
(`temple-route-ef65fe6b666a4b6b9e1aaf9fce5d45a5`). Exact failure reports are retained.
The focused fixture fixes the combat/idle test setup; it does not fix the two
navigation failures or complete the outstanding pickup/targeting requirements.
All 791 Rust tests (one unrelated ignored), locked release build, seven package
tests and Python compilation pass. Overall credit remains **42.1%**.

### Temple personal pickup and native target switching

`tools/test_coop_utemple_interactions.py` passes through the public internet with
40 ms delay/20 ms jitter on build
`2c43012c2b9d76aecd057933efeb922b46977f870316b8ca1bf275db7c05d055`.
It resumes the same exact captured exit checkpoint described above. The new
opt-in driver exposes native Snark selection without running additional AI and
can send ordinary three-dimensional swimming inputs toward a local test goal.
These additions do not change normal gameplay or network state.

In `private/coop/temple-interactions-24b578549f2840b6a59bf2b9df93bdb4/`:

- The original final WaterSnark selects Blue at distances 143.75/291.32, with
  both players visible. Its bite reduces Blue from 58 to 48; Red stays at 20.
- Ordinary swimming moves Blue away and Red closer. The same fish selects Red
  at distances 190.76/92.26; its bite reduces Red from 20 to 10, leaving Blue at 48.
- After defeating it through ordinary attacks, Blue collects placed item
  `utemple:26` and heals 48 to 100 while Red retains her own uncollected copy and
  stays at 10. Red then collects the same placed item and heals to 100 without
  changing Blue's resources. Partner Will changes are limited to normal recovery.
- All 206 comparable authoritative player, world and native-enemy states match.
  There are no deaths, drowning hits, traversal warps or scenes in this focused run.

The run takes 34.9 seconds and produces no screenshots. Its exact fixture hash
is `a8d9f78b9217fc73eaa7997f2b1b92389256339ab03be5d65e0e4d3f8a2bd1a3`.
Both full single-player swim routes, the temple state/migration checks, 13 native
paired component cases, 791 unit tests and the locked release build also pass.

Shared-passenger boarding is not applicable to this visit: `Temple::event`
starts an independently swimming turtle guide, while `motion.rs` controls
falling panels, columns, spikes and clams. There is no activated passenger lift
or ride. Full-route test navigation around the lethal traps remains under
investigation, with the failures retained above; this segment adds no campaign
completion credit by itself. The total remains **42.1%**.

The offline captured-state probe now supports selecting either body and replaying
ordinary steering against the advancing shared machinery. On diagnostic build
`221e5fd899512c4483703d60d6be1f19ce1ddae5815f991b0eca4d8c44e7eec3`, the recorded
panel approach reproduces the lethal contact at ticks 10,306–10,308 with several
steering delays (the live failure was at 10,307). This isolated replay holds the
recorded goal and gives the other player neutral input; it is a diagnostic, not
a full input-stream replay. The shorter forecast samples did not demonstrate the
suspected sampling gap. A subsequent full-route trial with earlier steering also
failed at the panels (`private/coop/temple-route-53780ccd87d946c18b3e95d6cc95d9e2/`,
tick 10,299); that unproven steering change was reverted. Normal gameplay, damage
and the previously verified controller settings are unchanged.

### Temple test swimmer momentum checks

Build `2c7e4e01061b9b762ed95365d1782dd4bc2418860436aa9a160f1debda0a5f68`
adds braking clearance and a one-second prediction using the real movement
controller to the opt-in route planner. It checks both the first detour leg and
direct routes; nearby hazards keep the search in the background worker. Normal
movement, collision hulls, trap damage and resources are unchanged.

A new open-water regression starts with sideways momentum and holds each goal
for half a second. The earlier planner crossed the forecast obstacle at tick 38;
clearance alone and a shorter prediction also failed. The final planner reaches
the destination without crossing it. All 792 active Rust tests, the locked
release build, seven source-package tests, 13 native Temple component cases,
state/migration checks and both full single-player swim routes pass.

All four production-Arena paired routes also pass on that build, with either
leader and watched/skipped endings. Blue-led runs finish at 50/48 Sanity and
Red-led runs at 48/50; each validates 35 watched or 34 skipped exact replicas.
Evidence: `private/temple-2c7e4e0-native.log` and
`private/temple-2c7e4e0---utemple*.log`. The deterministic regression and these
routes do not alone establish repeatability in separate internet clients.

The full local two-client Blue-led route with 40 ms delay/20 ms jitter also
passes in `private/coop/temple-route-2f933100ad5a44fd9dc5d9329e71a576/`:
3,230 matching player/world/native-enemy samples, correct exit ownership,
30/20 final Sanity, individual air, no deaths/respawns/warps, and both arriving
in Garden. Background searches took at most 502 ms; preparation stayed below
4.6 ms. The fixture's printed result now distinguishes local and internet runs;
its saved `internet` field already made that distinction.

Two public-internet attempts on this build instead fail early with Red drowning
at node 23, before the panel section. Their guest reports trail the host by more
than 2,000 simulation ticks and report about 4.3 seconds of round-trip delay.
The previous build `221e5fd` reproduces the same failure and delay in a separate
public run. This comparison does not isolate the cause to the new planner;
public connection behavior still needs investigation. Retained evidence folders:
`temple-route-5c1dd9c5c8e2422fa9acfe4bb034a555`,
`temple-route-f7c6896bf9aa44b89f31ccbfa0bb7a3a`, and
`temple-route-1a85e9d430ba47568f741ad981fe5c4b`, all under `private/coop/`.
No new public pass or campaign completion credit is claimed. Overall: **42.1%**.

### Bound snapshot backlog across the connection

Protocol revision 6 adds receipts for validated host snapshots. The host permits
at most four unconfirmed snapshots across the whole connection, then sends the
current state when space becomes available. Small local channels alone did not
provide this bound: TCP, WebSocket and relay buffers could still accumulate old
states. The host also avoids building snapshots while this window is full.
Controls and session changes retain their ordered delivery.

Receipts must name a snapshot actually sent in the current visit. The last
receipt can be repeated harmlessly; unsent ticks and future visits are rejected.
Old-visit receipts cannot affect a new visit, and travel/rejoin reset the window.
The guest sends receipts independently of player input and coalesces them to
the newest validated state. Both players need the same new build.

A real-socket regression admitted 24 stale snapshots before the fix. It now
holds the four-snapshot limit, resumes at the latest state, and preserves a
movement command and exactly one jump. Additional socket checks cover forged
receipts, duplicates, travel and rejoin. All 795 active Rust tests, the locked
release build and seven source-package tests pass.

Build `f93ea32deceba75950b7e91d91e392585d652a7e0f6bcb6312c00c2fbf33dbca`
has the following public-internet evidence with 40 ms delay/20 ms jitter:

- Blue-led traversal reaches Garden alive at 40/4 Sanity. All 2,925 comparable
  player/world/native-enemy states match, and all 872 host observations respect
  the four-snapshot limit. Median guest round-trip time is 223 ms, p95 251 ms.
  The full route still fails its clean-air check: Red takes three drowning-damage
  ticks near node 76. Evidence: `temple-route-c297b7d9e11d40c584a98395fea02a86`.
- Red-led traversal fails when Blue dies at node 126. Its 2,305 comparable states
  match, the receipt limit holds, and median/p95 round-trip time is 217/249 ms.
  Evidence: `temple-route-d9fa0dbe51d54800bf13e3fee4af6702`.
- The isolated Blue-led exit segment passes fresh-process guest rejoin and the
  original Garden transition, preserving resources/progress at 48/20 Sanity,
  with 194 matching states. Evidence: `temple-route-5f69717cf9ab42ae84759c054ea0e5ca`.

These folders are under `private/coop/`; failures remain recorded as failures.
The public connection is healthy in these runs, but yesterday's older-build
comparison does not establish that this change alone caused today's latency
improvement. The receipt bound is established independently by the regression.
Temple traversal reliability remains open; overall credit stays **42.1%**.

### Garden arrival and shared lily supports

Garden now prepares both living players before its automatic arrival scene.
The normal entrance refill applies once to each; the shared shell story grant
reaches the waiting partner too. A dead partner is not revived by preparation,
and later low-health respawns do not repeat the entry refill. Already initialized
single-player saves retain their resources when entering co-op.

Each of the three free-running lilies advances once per shared update, carrying
its living riders together. A blocked rider holds that pad without holding the
other lilies or the world clock. The optional collapsing bridge has a shared
saved clock and commits proposed rider movement together. These lilies have no
activation or boarding trigger; separate pads can run while players occupy
different parts of the route. Existing single-player saves omit the new optional
clock state; enabling co-op preserves their current machinery poses.

The first focused headless run on source
`33f62542ccbd79664b821582a54813aaa4f4483794ae641bc1e90e580bc9e660`
passes arrival/dead-partner/migration checks, both Rabbit scene owners watched
and skipped, frozen waiting actors, repeated pause/restore, safe reunions and
two separated riders on each lily for 75 seconds. Existing single-player scene
and world checks also pass. Evidence: `private/garden1-coop-native-33f6254.log`,
`private/garden1-scenes-33f6254.log`, `private/garden1-world-33f6254.log`.

The expanded native check additionally covers each pad at 30/60/144 Hz, either
blocked rider preventing partial movement, other pads continuing independently,
exact stopped-clock restoration, resuming after the obstacle clears, dead riders
neither moving nor blocking, and bridge activation/restoration/retirement. Its
obstruction fixtures use measured native movement to place a wall just beyond the
current body; no game physics or damage is changed for these checks. Bridge
coverage here verifies its timeline and saved poses, not a full two-rider route
over the falling pieces.

All 21 expanded cases pass in the final release build with source identity
`35a38d41ac947da40155971c029f1a121b502a90f8eefef4f42853e64983322c`;
evidence is `private/garden1-coop-native-final.log`. All 795 active unit tests
also pass (one remains ignored). No screenshots or desktop interaction were
needed for these checks. The single-player evidence above exercises the same
runtime adapter; subsequent changes only expand/format the check code.

These are component checks, not a full paired Garden traversal. Combat,
consumables, recovery, internet traversal and transition/rejoin still need their
combined evidence. No campaign row gains completion credit; total remains
**42.1%**. Progress reports should name completed components alongside that total.

### Player contact on slopes

The first native paired Garden course captured two ordinary grounded players
overlapping on the initial bank. Both were dry, without rope/ledge/climb or
scripted movement. The contact check required a centre support within two units,
but a box standing on a slope touches the surface at its uphill edge; its centre
can be higher. The horizontal correction also needed to follow the ground slope.

Grounded contact now follows the current surface normal and allows the centre
support depth implied by the collision footprint. Swept clearance, walkable
support and the stationary partner's position remain checked. Removing inward
velocity uses the same surface direction, avoiding an upward impulse when an
uphill approach stops. Flat supports retain the original short centre probe.

A ramp regression fails before the fix at 35.125 units of horizontal separation
(the requirement is 36). Both uphill and downhill approaches pass after the fix,
along with all 796 active unit tests. The native Garden route remains a separate
verification requirement; the earlier attempts are failures, not completed visits.

Final source `4baa8d5f79534e675fafa062435189d8faea0043828e6dd3dd378e3036261aba`
passes the 21 Garden component cases plus an offline replay of the exact native
bank capture: separation increases from 18.434 to 36.000 units with clear bodies,
the stationary partner unchanged and both personal resource states preserved.
The optional replay uses `LOOKING_GLASS_GARDEN_CONTACT_REPORT` with
`--garden1-coop-check`. Evidence: `private/coop-slope-before.log`,
`private/coop-slope-after-unit.log`, `private/coop-slope-native.log`; the original
capture is under `private/garden1-coop-route-73b4109/`.

The new windowed `--garden1-coop-route-check` drives both players with ordinary
controls through the production Arena and records private world/player state.
Its current full attempt on the same runtime (`8474635`, before adding the
offline replay check) fails after Blue loses the first rope and Red dies while
waiting farther ahead. No full-route pass is claimed. The route log/profile are
`private/garden1-coop-route-8474635.log` and `private/garden1-coop-route-8474635/`.
All owned Anode jobs/windows closed and the lease was released; no screenshots
were opened. Overall campaign credit stays **42.1%**.

### Garden automated rope recovery

The paired driver now approaches and grabs again after losing a rope, using
ordinary movement, jump and use inputs. The follower waits with combat available
on banks before the first rope and second launcher instead of hanging exposed
on the second rope. Grounded rope approaches can use the same stuck-jump recovery
as walking. Invalid `LOOKING_GLASS_GARDEN_COOP_FIRST` values now fail explicitly
instead of returning success without running a case.

Build `1be07acfdb754b6860d8e12af9ceb999c6164bc252c904a05761cc1f67385029`
records Blue losing the first grip without a use input at tick 5846, recovering
and intentionally releasing at tick 6438. Both players complete the Rabbit scene
and reunion; Blue reaches step 111 while Red stalls at step 72 and dies. The next
build, `2acee224394243f35a31b057884bdb3e37c4f13a4510c85b4014b5332fb75b6b`,
clears that grounded approach with both players but still fails at tick 19942,
steps 73/83, when Red dies farther along. The high grab before the timed swing
and subsequent drop need investigation. Neither run is a full-route pass.

Evidence: `private/garden1-coop-route-1be07ac.log` and
`private/garden1-coop-route-2acee22.log`, with corresponding private state reports.
The final build passes 796 unit tests (one ignored), the locked release build,
seven source-package tests and the invalid-selector native check. No gameplay
physics, damage, resources or placements were changed. Overall campaign credit
remains **42.1%**; report these intermediate results alongside it.

### Garden grip height and launcher diagnostics

A grab followed directly by a timed swing now first climbs to the approach's
height. Catching that rope high after an obstacle jump previously started the
swing several hundred units above its intended path. Build
`c0f121135c442f8fb2d1cd9c881591a89d68a9129517f25136812e531c9e2a12`
keeps both players alive through step 127, at 68.735/54.440 Sanity; the former
death at step 83 is gone in that run. Hard landings now log speed and health
changes, so fall damage can be distinguished from an unexplained resource loss.

Final fixture build
`3310b54a70a1fd869aa659e802f1fd8dc8b2debf8e195f9ec4a27cfcce4794c3`
also permits grounded obstacle jumps on launcher approaches. Its default course
still fails there. An ordinary-input override aimed at the first launcher's
centre gets both past it, but the second launcher/landing still fails. A fresh
single-player native-cast run also dies at step 131, near the same gap. Steering
and ledge-approach variants have not yet resolved the paired failure. A short
single-player replay from its genuinely earned second-launch checkpoint reaches
a hanging ledge; that focused diagnostic deliberately has no campaign exit and
is not a route pass.

`LOOKING_GLASS_GARDEN_COOP_ROUTE` accepts a diagnostic course JSON without a new
executable build. The fixture saves the exact course in its isolated profile.
It also now requires a real Garden2 load, safe separate arrivals and exact
personal resource carry-over before reporting full success. That new exit gate
has not yet been reached. No gameplay movement, damage or placement code changed.

Evidence: `private/garden1-coop-route-c0f1211.log`,
`private/garden1-coop-route-3310b54*.log`,
`private/garden1-sp-route-3310b54.log`, and
`private/garden1-sp-launch-samples.log`. The final build passes 796 active unit
tests (one ignored) and the locked release build. Full paired and single-player
route failures remain recorded; overall credit stays **42.1%**.

### Garden launcher, rope landing and three complete paired routes

The paired course now has its own authored control file, `coop_steps.json`.
Relative to the single-player course, it clears a larger area before the
launcher, approaches the first mushroom at its trigger centre, steers farther
onto the second landing, explicitly pulls up and slows over the final rope
landing. These change test inputs only. Gameplay movement, enemies, resources
and the single-player control file are unchanged.

Sparse flight logs and exact launch/release captures distinguish steering,
combat knockback and collision. The second launch originally stopped short of
its ledge; a farther inland approach catches it. A single-player replay of an
earned hanging state then showed an enemy hit interrupting the pull-up. Clearing
that enemy before crossing resolves the interruption. A comparison using the
older fixture also defeats the enemy with the larger clear radius: the separate
aim-tracking adjustment is not established as the cause of this improvement.
The fixture preserves pitch and continues target observation during locked
weapon animations while holding traversal controls neutral.

The later rope trace showed full-speed steering overshooting the platform
before Alice descended onto it. A proportional hold at the landing point fixes
that crossing using ordinary controls. Build
`3327b3911b5f3e013566702e0f41e559fc3447d04bb906fad4ae9d8f2308e604`
with the exact saved course completes:

| First player | Scenes | Ticks | Matching replica samples | Final Blue/Red Sanity |
| --- | --- | ---: | ---: | --- |
| Blue | Watched | 31,263 | 53 | 47.735 / 8.223 |
| Red | Watched | 30,071 | 51 | 95.000 / 41.897 |
| Red | Skipped | 23,445 | 40 | 97.914 / 95.000 |

Each pass requires the authored exit, two scene reunions, no death/recovery or
unexpected warp, a real Garden2 load, exact personal-resource carry-over and
clear arrivals at least 36 units apart. Blue-led skipped scenes still stall at
step 62's combat clear. Both players remain alive; the nearby corporal retains
49 health. This is a recorded failure, not a passing four-case suite.

Evidence: `private/garden1-coop-route-3327b39-ropehold.log`,
`private/garden1-coop-route-3327b39-red-ropehold.log`, their isolated profiles,
`private/garden1-coop-route-3327b39-clear.log` (overshoot trace), and
`private/garden1-coop-route-4476ea3-clear-control.log` (comparison).
The diagnostic build passes 796 active unit tests, one ignored test, the locked
release build and seven package tests. No screenshots were used. These three
routes do not complete the combined campaign verification row; total remains
**42.1%**, with this explicit intermediate milestone reported alongside it.

Final default-course build
`fe62901dbae493398c607c62183445d3eeda758a7093cd35fce844f2e13e5a1f`
repeats all three passes with the same ticks, sample counts and resources. Its
Blue-led skipped case repeats the same step-62 failure. Logs are
`private/garden1-coop-route-fe62901-blue.log` and
`private/garden1-coop-route-fe62901-red.log`; both profiles' saved courses match
`coop_steps.json` exactly. Run `--garden1-coop-route-check` with
`LOOKING_GLASS_GARDEN_COOP_FIRST=Blue` or `Red` to select the leader; each selector
runs watched then skipped scenes and stops on the first failure. No route-file
override is needed. The final build passes all 796 active unit tests (one ignored),
the locked release build, seven package tests and the 812-file worktree audit.


### Garden four-route milestone

The automated paired course now uses ordinary jump shots after ten seconds of
stationary firing at the early combat clear. Projectile forecasts showed that
low cover intercepted the original shots even though a thin visibility query
found an exposed target point. The test now logs sparse hypothetical and
in-flight projectile contacts to distinguish blocked shots from missing damage;
these forecasts never change the live world.

At the shared launcher landing, Red aims 80 units to the side of Blue's landing
goal. The prior trace showed the second arrival colliding with the player pulling
up and falling below the catchable ledge. Both still use normal movement and
ledge controls. At the final platforms, the course clears nearby enemies before
jumping into melee range; it can use jump shots after two seconds there.

Trial build `c8ea5303e569f2a6275a564a07aab11acd18cd6ea90e5ada14e073cb68beafab`
with the saved exit-platform course completes all four combinations:

| First player | Scenes | Ticks | Matching replica samples | Final Blue/Red Sanity |
| --- | --- | ---: | ---: | --- |
| Blue | Watched | 31,311 | 53 | 57.735 / 18.440 |
| Blue | Skipped | 24,427 | 41 | 35.549 / 70.000 |
| Red | Watched | 29,541 | 50 | 22.646 / 61.004 |
| Red | Skipped | 23,713 | 40 | 48.288 / 30.329 |

Every pass retains the authored-exit, two-reunion, no-death/recovery/warp,
real-Garden2-load, exact-personal-resource and separate-clear-arrival gates.
Evidence is `private/garden1-coop-route-c8ea530-exitrose.log` and its isolated
profile, including the exact course and terminal states. The course is now the
public default in `coop_steps.json`.
`LOOKING_GLASS_GARDEN_COOP_SCENES=watched|skipped` optionally selects one scene
mode, independently of the existing first-player selector. Omitting both runs
all four cases. Invalid selectors must fail explicitly.

This is a single-process paired traversal milestone, not full independent-client
internet validation. No gameplay movement, health, damage, enemy placement or
single-player course changed. Internet traversal and the Garden recovery matrix
remain open. Overall remains **42.1%**; report **4/4 native routes passed** beside
that figure so this progress is visible. No screenshots were used.

Final default-course build
`74aa21ece12fd0e7262b9fcedbeacf6afa2cc32e1f557c7a63058459cd913aba`
repeats all four passes with exactly the same ticks, sample counts and resources.
No course or case selector override is needed. Its saved course matches the
public file exactly. Evidence is `private/garden1-coop-route-74aa21e-all.log`
and its isolated profile. The invalid scene-selector check also passes.
Validation: locked release build, 796 active unit tests (one ignored), seven
source-package tests and the source worktree audit pass.


### Independent Garden clients and delayed controls

`tools/test_coop_garden.py` runs two processes with separate profiles over the
public internet transport, with 40 ms delay and 20 ms jitter by default. Each
client drives from its own received frame and world. The harness sends local
course commands, never positions, health, inventory or enemy state through the
network. It records player, world and native-enemy state hashes, scene owners,
reunions, personal inventory and the outgoing visit. Death, checkpoint recovery
or an unexplained warp fail this clean-traversal test. Recovery still needs its
own cases.

The local route controller now accepts elapsed observation ticks. Repeated guest
frames retain continuous inputs without repeating jump or rope edges; coalesced
updates cannot skip periodic input boundaries. Remote jump approaches anticipate
travel during input delivery. The authored course can provide `network_pos` for
a delayed-client aiming point while retaining the independently verified native
route. Its first mushroom uses that adjustment to avoid steering past the pad
while airborne. Both clients save the exact course bytes compiled into their
executable; the fixture hashes those bytes rather than the current working tree.

Run the fixture in the background desktop, for example:

```powershell
python tools/test_coop_garden.py --exe target/release/looking-glass.exe --data <local-base> --first blue --scenes skipped
```

`--first red` exchanges the first-arrival role; `--scenes watched` exercises
watched scenes. Public internet is the default; `--local` explicitly selects a
local diagnostic. Scene captures are disabled. Full and previous valid reports,
compact histories and timing summaries are retained in the isolated private
folder on failure.

The initial `8cf3e35` internet run recorded 2,590 identical sent/restored player,
world and native-enemy states but failed the early crossing. The guest's jump
counter advanced after the host body had already left its ledge; the host then
overshot the first mushroom. A subsequent `a40b072` run with anticipated jumps
and the revised aiming point passed both locations, the Rabbit scene and the
later ropes. It failed at step 62 when the team wiped. The last pre-wipe guest
snapshot contains a corporal grenade approaching both players, who had 36.538
and 59.653 Sanity. The harness rejected the automatic checkpoint restart.
These are partial route observations, not a full internet traversal pass.

Evidence: `private/coop/garden-route-ffa9e18a28754f2c91cf8bfcf2afc5bd/`
and `private/coop/garden-route-57493ff6fcbe442f8fe3bbd53293a732/`.
The latter was a diagnostic candidate whose shared aiming-point change also
caused a later native combat failure. The final source confines the aiming
adjustment to the network course. Full internet traversal and the recovery
matrix remain open; overall credit stays **42.1%**.


Finalized native-path build
`235926edfeb6423b88e8416d79b133acf0cb6c24e8eb9a4aa9ef6cfe9d898ce8`
repeats all four native passes with the original ticks, replica counts and
resources. Its internet repeat again passes the early crossing and Rabbit
scene, then fails when Red misses the later rope at step 29. The previous valid
state retains the fall before death; 1,461 sent/restored player, world and enemy
states match. Both saved compiled courses match the source. Evidence is
`private/coop/garden-route-a67829cf32d74109b28fe27c1dd82b35/`.

The independent-client controller now sends explicit grab/release edges for
rope actions. A generic use pressed before a rope prompt existed could arrive
after automatic attachment and toggle that grip off. The regression delivers
such an early grab after attachment through the real input latch, checks that
the grip remains, then checks that an intentional release still works. This
changes test input selection only; the production protocol already supports
those distinct actions. Full internet traversal remains unverified.


Rope-intent build
`b716dc30251ebb8116e45b7ee40a1530fd98aba6ba7aa88257aefda2bcfb0ea5`
passes the locked release build and all 798 active unit tests (one ignored).
Its two-client skipped-scene run passes the previously missed rope and reaches
the step-62 combat clear with both players. It then rejects a simultaneous
wipe: the last valid host report is tick 14,569 with 29.873/70.000 Sanity; tick
14,593 has restored checkpoint generation one. All 2,099 common sent/restored
player, world and native-enemy states match. Both compiled course snapshots
match the recorded source. Evidence, including previous valid full states, is
`private/coop/garden-route-414a5a5a62b647a486b1a9cb253deb42/`.
This remains a failed full route. The next investigation is the clustered
players' response to the corporal's grenades. The four native-route passes
above belong to build `235926e`; the later rope-intent change affects only the
independent-client test controller and its regression test.


### Garden running approaches and short network segments

The previous Red fall at step 72 was preceded by braking at the waypoint before
an airborne rope interception. Source
`45cbce80f5969a142cc6d304e1421fd116b88660b2d3e1b1c4180783a3bf9005`
keeps the opt-in test driver's running approach. Its full public run passes that
crossing, reaches Blue step 93, and fails when Red misses the next rope at step 79.
All 2,377 common player/world/native-enemy states agree. Evidence:
`private/coop/garden-route-dfe072def1c7409f83f00b1759e8685a/`.

The fixture now automatically captures exact authoritative states before the
later crossing. A normal isolated checkpoint exported from its step-71 capture
preserves the recorded 41.592/63 health, positions, inventory and combat state.
The independent-client course can start at explicit steps using ordinary inputs;
this changes the test controller only. Segment results explicitly exclude the
preceding and remaining traversal.

Source `14ecae613acd29a4da3182599da26862c46547b885294e4bcc2593aebd526536`
also keeps momentum before running jumps. Its first segment still failed: the
fixed 170 ms steering estimate lagged actual 220-270 ms input acknowledgements.
That failure remains in `garden-route-d265eeed2d0e4afab200fb4e97ec0a7a`.
The Python fixture now smooths measured round-trip time, with a 50 ms allowance
and the injected-delay estimate as its minimum. Each observation is sampled once;
the final input lead is recorded alongside the exact fixture source and hash.

The same executable then passes both rope crossings in a public checkpoint
segment with 40 ms delay and 20 ms jitter: both begin at step 71; Blue reaches 86
and Red 84, alive at 49.592/37 health, without respawns. All 169 comparable
player/world/native-enemy states match. Final measured input lead is 264 ms.
Evidence: `private/coop/garden-route-c9a199a40d8540e59068022c6eae5e79/`.
The run takes 42.98 seconds including connection setup and closes both clients.
No images were opened. All 803 active Rust tests (one ignored), locked release
build, and seven source-package tests pass.

These reports also reveal an open contact case: ordinary players attached to
the same rope can overlap. The ground-contact resolver intentionally skips
attached bodies; the spaced revival fix does not resolve later climbing contact.
The failing segment records six overlapping samples, including horizontal
spacing 5.51 and vertical spacing 19.72 at tick 16,091. This requires a rope-aware
contact fix and a fresh segment check. No new campaign row is claimed; overall
credit remains 42.6%.

The subsequent full entrance repeat on `14ecae6` with measured timing fails
before those rope segments: Blue dies in combat at step 55 (tick 13,082), while
Red is at step 47 with 100 health. All 1,841 common player/world/native states
match. Evidence: `private/coop/garden-route-64b6dc21d8f646c2ad8a0dc9bc73fc32/`.
This remains a failed full-route attempt; the successful checkpoint segment
cannot replace it. Both clients/helpers close and Anode is released.


### Clockwork paired-transport credit (4 November 2026)

Source `9ce4f98` adds 0.25 movement points for actual native paired lift/floater
boarding/carry, blocked-rider rollback and separate sinking supports. The 38
staged checks also exercise personal levers and their saved owners, but no
additional progression credit is assigned before complete paired cinematic
handoffs are verified. Movement is now 10.5/15; the other category totals remain
unchanged. Total is about 48.33, reported as 48.3%. Six campaign rows retain
combined evidence; 33 remain open, plus earlier contact-system repeats.


### Clockwork paired-scene verification credit (4 November 2026)

Source `ec48aa7` passes 40 staged scene/speech cases: native activation with
either Alice, watched/held-skipped cinematics, living/dead waiting partners,
pause/compressed restore, safe reunion and independently moving explorers
through ordinary speech. All 38 machinery records still pass, and single-player
11+1 records exactly match baseline. See [Clockwork evidence](HATTER1.md).
This adds 0.25 scenes/progression points, now 10.75/20. The total is 48.58,
reported as **48.6%**. Six campaign visits retain combined evidence; 33 remain
open, plus earlier contact-system repeats. Closing doors, the final transport,
continuous two-client Clockwork traversal and recovery/rejoin remain unverified.


## Underwater countdown reconnect fixture

`tools/test_coop_utemple_countdown.py` extends the normal temple recovery test.
It restarts only the owned guest process during each player's actual drowning
countdown, leaving the survivor's ordinary swim command running. While the host
waits, its complete player/life frame and shared world must remain unchanged.
The replacement guest must preserve the remaining countdown, drowned body and
breath state, both personal inventories and swimming equipment. Its first
playing frame is compared with the host's exact matching tick using full-body,
world and native-enemy hashes; normal air-bubble refills remain real simulation.

The test then requires the original ten-second spectator/revival checks, safe
25-health swimming placement with fresh air, a shared saved-checkpoint wipe and
an entrance rejoin. Camera assertions compare the actual rendered body and its
recorded interpolation inputs, including the cut back to the revived player.
No health, player, air or progression state is injected. These are recovery
components, not completion of the temple route or the all-level campaign goal.

Repeat the passing core case in Anode (it covers BOTH dead-player roles):

```powershell
python tools/test_coop_regressions.py --exe target/release/looking-glass.exe --data <original-base-folder> --only swim-countdown-blue --delay-ms 40 --jitter-ms 20
```

Public invites are the default; use `--local` only for a loopback baseline.
Add `swim-countdown-red` to test the reversed route order. That case currently
wipes near the first column before reconnecting and remains a failed route
check; the two-order suite is not a complete pass. See `COOP_PROGRESS.md`.
The suite writes private machine-readable results and retains failed attempts.
It does not inspect screenshots. Each case records its fixture source hashes,
client/build identities and the full before/held/after reconnect snapshots.
