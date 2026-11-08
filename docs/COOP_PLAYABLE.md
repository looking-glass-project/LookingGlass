# Co-op playable milestone

The user has prioritized reaching a **100% playable co-op version quickly**,
with a separate polishing stage afterward. This changes the order and depth of
verification; the full campaign and requested gameplay features remain in scope.

## Required for the playable milestone

- All 39 campaign visits support two players through their normal progression,
  including swimming, leaves, moving supports, bosses and the ending.
- Host and Join are available from the menu, with a working public invite link
  and two independent client processes/profiles for testing on one PC.
- Blue and red Alice have separate controls, cameras, health and consumables;
  enemies can target the closest visible living player.
- First arrival activates scenes; original single-Alice scenes reunite both
  players afterward. Shared transports wait for all living players to board.
- Death uses survivor spectating, a ten-second timer and safe low-health revival;
  a team wipe restores a shared checkpoint with both players.
- Earlier single-player saves can become co-op games, and earlier co-op saves
  can be resumed and joined again. Basic disconnect/rejoin must work.
- Existing single-player behavior and save compatibility remain usable.

## Work order

1. Core controller adapters are integrated through the Queen finale on
   `c8a041e`. Resolve normal-play blockers found by the consolidated campaign pass.
2. Build once per coherent batch and run focused checks for the changed behavior.
   Reuse passing evidence for unchanged systems. Fix build errors and failures
   that prevent normal play before expanding test coverage.
3. Run a consolidated campaign playability pass. Accelerated production-Arena
   runs with two players can establish normal visit progression and exits;
   separate-process internet tests establish transport and core flows. Exercise
   essential save, death and rejoin behavior at representative points.
4. Make the playable build available for hands-on testing. Publishing a release
   or merging still requires the user's later request.

The core acceptance evidence is tracked in COOP_CORE_TESTS.md. A visit is not
proven playable by having an adapter alone: its normal progression must work.
The broader historical matrix in COOP_CAMPAIGN.md remains useful evidence, but
its exhaustive combinations do not all gate the playable milestone.

A continuous chain of normal save/resume segments can establish a playable
route when unchanged checkpoint hashes connect the entrance to the actual exit
and all progression is covered. Audit that continuity before crediting it;
isolated staged tests or disconnected passing segments are insufficient. A
second uninterrupted run of an already verified save chain belongs to polishing.

## Polishing stage

Defer exhaustive frame-rate/difficulty combinations, every scene owner and skip
timing, repeated full traversals of unchanged levels, rare save/network failure
combinations, complete per-visit recovery matrices, visual/audio comparisons
and presentation refinements. Promote a deferred item immediately if it causes
a normal-play blocker, data loss, or a missing requested core feature.

Report playable readiness and the polishing backlog separately. Do not label
the game 100% playable while a level or requested core flow remains blocked.

## Playable milestone complete — 8 November 2026

**100% of the agreed playable scope is accepted on `coop`.**
[Start co-op or run two clients on one PC](COOP_QUICKSTART.md).

| Playable milestone | Accepted points |
| --- | ---: |
| Requested core controls, Host/Join, resources, scenes, death, saves and rejoining | 40/40 |
| Normal paired progression through all 39 campaign visits | 55/55 |
| Actual public two-client ending movie and credits handoff | 5/5 |
| **Playable readiness** | **100/100** |

Frozen `cddd820` completes the final public test from the unchanged, normally
earned Queen2 checkpoint. Both clients finish actual combat and present the
ending movie and credits, using the public invite path with 40 ms delay and
20 ms jitter. Ordinary focus/resume and held Enter skip inputs take each film
to credits. **3,445 matching sent/restored player, world and NPC states** agree.
The original checkpoint and host's playable checkpoint remain byte-for-byte
unchanged. Evidence: `private/coop-playable-finale-public-verified.json`.

The build passes **951 active Rust tests** (five ignored), the release build,
source audits and seven packaging checks. The final Pool default reproduces
its accepted route exactly. The thirteen earlier public save/rejoin groups
remain accepted; both solo-save conversion and co-op continuation are covered.
The final changes add test inputs and presentation logs, not combat assistance.

This is the playable milestone, not a claim that every bug or fidelity gap is
resolved. Polishing retains the broader frame-rate/difficulty/device matrix,
scene-owner and skip permutations, per-visit recovery/network edge cases,
visual/audio comparisons and hands-on tuning. A normal-play blocker or data
loss found there takes priority. The historical full matrix remains at 81.5%
as a separate record; it is not the playable milestone's denominator.

## Pool of Tears normal paired route accepted

Frozen `dcdc1d2` passes the final missing normal visit in **67,887 ticks**,
with **566 exact compressed replicas**, four scene reunions and actual Hollow
Hideaway arrival. Both players finish alive at **63.39/25 health**. Three deaths
use normal recovery: each revival occurs after exactly **1,200 active ticks**
at 25 health. There are no team wipes, health floors, cheats or forced progression.

The input pilot takes narrow banks in single file, leaves room for boarding,
and steers/jumps around the third leaf's obstacles using collision queries.
The rope approach uses ordinary movement and E; one player's death there is
recovered normally. This proves a playable route with the requested recovery,
not a flawless deathless traverse. Gameplay physics, scenes and resources are
unchanged. Evidence is pinned in `private/coop-playable-pool-course-verified.json`.
The standard check on `cddd820` exactly matches the accepted commands, players,
world, scenes, revivals and arrival. Its pin is
`private/coop-playable-pool-default-verified.json`; repeat with
`tools/test_coop_native_route.py --map potears1` and local game data.

Normal paired progression now covers **39/39 campaign visits**. The public
save/rejoin core retains its thirteen accepted groups on unchanged `0b5bca3`.
The remaining playable acceptance item is the actual two-client movie/credits
handoff; broader recovery and presentation combinations are polishing.

## Hollow Hideaway normal paired route accepted

Frozen `5fd9df4` passes the complete normal paired visit in **33,897 ticks**,
with **283 exact compressed replicas**, the original Bill scene and reunion,
and actual Just Desserts arrival. Both players finish alive at **78.70/26 health**.
Red dies once and revives at **25 health after exactly 1,200 active ticks**;
there are no team wipes, health floors, cheats or forced progression.

The input pilot takes narrow banks in single file, boards both large leaves,
and steers around obstacles using swept collision queries. Both players make
an ordinary diagonal jump from lily four to the house bank. Lily five's carry
retains its existing component evidence; its alternative landing route is not
claimed by this run. Combat, world machinery, scenes and saves are unchanged.

The accepted course is pinned in `private/coop-playable-hollow-course-verified.json`.
The default diagnostic on `b88c6c7` exactly matches the accepted commands, players,
world and arrival; its pin is `private/coop-playable-hollow-default-verified.json`.
Repeat with
`tools/test_coop_native_route.py --map potears2`, local game data and a new profile.
All 951 active Rust tests pass on `20e7894`; subsequent changes affect only
diagnostic inputs and geometry inspection. Source/package checks pass.

Normal route coverage is now **38/39 visits**, or **81.0%** under the existing
overall weighting. Pool of Tears is the remaining normal visit. The actual
two-client ending film/credits handoff is still required. The thirteen public
save/rejoin core groups remain accepted on unchanged `0b5bca3`.

## School Two normal paired route accepted

Frozen `b925e5c` passes the complete main quest in **53,696 ticks**, with
**448 exact compressed replicas**, **six reunions** and both players finishing
at **100 health**. The run includes the gym lever and bleachers, Gnome battle,
laboratory stairs and rescue, Jumbogrow, both personal portal journeys, greenhouse
growth, lollipop, potion and star rewards, and actual `skool1_start2` arrival.
There are no deaths, revivals, team wipes, health floors, cheats or forced quest
state. Optional Dice collection remains outside this main-quest route; the
normal return fills the missing campaign weapon baseline.

The run exposed a real post-battle reunion lock when the scene owner stood on
the gym's narrow outer trim and the partner had already descended. If ordinary
reunion fails there, co-op now places both players on verified supported ground
near the original scene's Alice mark. Both landings must fit before either body
changes. Personal resources, solo scenes and the save schema are unchanged.
The failing and fixed normal runs have identical inputs, completed steps and
positions through the scene trigger at tick 15,453; the fixed pair continues.

The pilot uses ordinary E for manual doors and walks the moving greenhouse
doorway. It ignores optional enemy drops with no supported approach. These are
test input changes, without altered loot, combat or progression. The standard
command's players, world, commands and arrival exactly match the accepted course
on `61d7ab5`. Evidence: `private/coop-playable-school2-default-verified.json`;
repeat with `tools/test_coop_native_route.py --map skool2` and local game data.

All **951 active Rust tests** pass on the gameplay-fix build `2acd182`; later
changes affect diagnostic inputs only. The original School Two checks also
pass watched/skipped scenes, pause/restore, mover/rider behavior and reward
gates. Source/package audits pass. Normal coverage is **37/39 visits**, with
the two swimming visits remaining, or **80.5%** overall. The rendered two-client
ending handoff remains separate; broader recovery permutations are polishing.

## Duchess normal paired route accepted

Frozen `a4f954c` passes the fresh Just Desserts route in **12,502 ticks**:
actual Jackbomb pickup, original introduction, living boss combat, turtle
rescue and the real temple departure. **105 exact compressed replicas** agree;
both players finish alive at **100/25 health**, with no deaths, revivals, team
wipes, health floors, cheats or forced progression. The watched introduction
reunites the pair; the original outro carries both into the next visit.

Temple arrival preserves personal resources and pickup history, grants the
original breathing-shell flag and fills the partner's missing Jackbomb campaign
milestone. The first attempt reached the exit but failed the diagnostic's
missing shell-grant allowance; the corrected check passes without changing
gameplay. The pilot now includes the legacy combat targets used by production.

Evidence: `private/coop-playable-duchess-default-verified.json`. Repeat with
`tools/test_coop_native_route.py --map potears3`, local data and a fresh profile.
All **950 active Rust tests** pass on the preceding diagnostic build `148a733`;
the final build changes only arrival expectations and flag registration.
Normal coverage is **36/39 visits**, with **3 remaining**, or **80.0%** overall.
This accelerated Arena check leaves internet/rendered-camera evidence separate.

## Majestic Maze normal paired route accepted

Frozen `845bb52` passes the complete normal paired Maze course: **33,270 ticks**,
**278 exact compressed replicas**, both personal portal arrivals, the real
child's escort and plate hold, and actual Mystifying Madness arrival. Both
players finish alive at **98/100 health**, with their personal resources carried
exactly. There are no deaths, revivals, team wipes, cheats or forced quest state.

The automated inputs use Cards/Ice, single-file narrow legs, ordinary walks to
regain sight of the child, and close spacing beyond the final plate. The real
gate must open before departure. The run also exposed and fixed the blocked
portal retry described below. The passing course and exact source/executable
are pinned in `private/coop-playable-majestic-maze-default-verified.json`.
Repeat with `tools/test_coop_native_route.py --map hedge1`, local game data
and a fresh profile. The default check exactly matches the accepted course
inputs, final players, shared world and onward arrival.

This accelerated Arena run proves normal paired progression and replicated
world state. Internet and rendered-camera evidence remains separate; broader
per-visit recovery permutations belong to polishing. Normal coverage is now
**35/39 visits**, **4 remaining**, or **79.4%** overall.

## Blocked portal retry fixed

A normal Maze run exposed a shared-portal failure: when the first player stood
at the destination, the second player's attempt was rejected safely, but the
contact became latched during its cooldown and stopped retrying. Blocked
personal portals now retain a pending contact until the destination clears or
the player leaves. The cooldown still applies, successful arrivals clear the
pending contact, and pending retries survive save/restore using the existing
contact field. Solo contacts are unchanged.

Frozen `1774b6a` passes all **950 active Rust tests**, including both-player
cooldown, restored retry, successful completion and cancellation coverage.
The complete normal paired Maze route now passes on `845bb52`, including
both portal arrivals. The portal fix itself adds no separate campaign credit.

## First school normal save chain accepted

The first Skool Daze visit now has verified normal entrance-to-exit coverage
through four connected public-invite segments on `59cc04b`, `d4869cd` and
`ced2c8c`. The unchanged theatre, balcony and spiral saves connect each segment
by exact file hash. Recorded starting and ending positions/resources agree
with those checkpoints; both clients retain ordinary gameplay throughout.

The chain covers all six scenes, personal Mallets, both lifts, the flying-book
bridge, recipe handoff and actual School Two arrival. Both players finish alive
at **12/77 health**, with **15,852 matching network states** and **309 matching
full school reports**. The recorded histories pass the full route assertions
without deaths, respawns or checkpoint rollbacks. This is connected normal
save/resume evidence, not a new uninterrupted run on the latest build.

The private audit is `private/coop-playable-school-chain-audit.json` and can be
rechecked with `private/audit-school-playable-chain.py`. Broader owner/recovery
permutations remain polishing work. Coverage is **34/39 visits**, **5 remaining**,
or **78.9%** overall.

## Queen normal paired route accepted

Frozen `ef316ac` passes a fresh normal paired Heart of Darkness run:
**34,711 ticks**, **291 exact compressed replicas**, both Queen forms defeated,
the original introduction/birth/death/ending sequence, and both players alive
at **100 health**. The first battle ends at about 74/71 health; the original
second-arena refill restores both players. No death, team wipe, cheats, forced
boss damage or progression staging is used.

At tick 18,660 the real second-arena checkpoint is written and restored into a
separate Arena/roster, with exact player and shared-world state. Both replicas
reach the actual durable ending state, with no map transition. The normal
weapons, enemy attacks and production scene/recovery pipeline remain active.
All 949 active Rust tests and source/package checks pass. Evidence is pinned in
`private/coop-playable-queen-default-verified.json`.

Repeat with `tools/test_coop_native_route.py --map qlair`, local data and a fresh
profile. This proves normal paired combat, scenes, checkpoint restoration and
ending state. The rendered two-client film/credits handoff remains a separate
acceptance item; it is not claimed by this accelerated Arena check.
Coverage is **33/39 visits**, **6 remaining**, or **78.4%** overall.

## Machinations accepted

Frozen `fcf4979` passes the normal paired course through actual Royal Rage
arrival: **36,116 ticks**, **301 exact compressed replicas**, and both players
alive at **22/25 health**, with personal resources carried exactly. Five normal
revivals each take exactly ten active seconds and return at 25 health. There is
no team wipe, progression staging, cheat state or changed gameplay balance.

The diagnostic uses live lift/pedal phases, precise pillar landings, single-file
wheel boarding and independent movement off the final climbing gear. It omits
the solo route's deliberate fall demonstrations; normal recovery remains active.
Gameplay, saves and networking are unchanged. All 949 active Rust tests and
source/package checks pass. Frozen source, executable, arrival and recovery
evidence is pinned in `private/coop-playable-machinations-default-verified.json`.

Repeat with `tools/test_coop_native_route.py --map tower3`, local data and a
fresh profile. Internet and rendered-camera evidence remains separate.
Coverage is **32/39 visits**, **7 remaining**, or **77.9%** overall.

## Labyrinthine Revenge accepted

Frozen `714bd1f` passes the normal paired course through actual Machinations
arrival: **138,944 ticks**, **1,158 exact compressed replicas**, and both
players alive at **95/25 health** with their personal resources carried exactly.
Eight normal revivals each take exactly ten active seconds and return at
25 health. There is no team wipe, progression staging or altered combat balance.

The diagnostic follows live gear poses, individual waypoint and hammer clocks,
ordinary Watch/Staff/Ice controls, and safe regrouping before the exit. Its
combat wait uses normal Will regeneration. Gameplay, saves and networking are
unchanged. All 949 active Rust tests and source/package checks pass. Exact
source, executable, arrival and recovery evidence is pinned in
`private/coop-playable-labyrinth-default-verified.json`.

Repeat with `tools/test_coop_native_route.py --map hedge3`, local data and a
fresh profile. Internet and rendered-camera evidence remains separate.
Coverage is **31/39 visits**, **8 remaining**, or **77.4%** overall.

## Water Logged accepted

`b278fdc` passes the normal paired water course, all three flushers, live fan
passages and final drain into actual Labyrinthine Revenge. The 28,247-tick run
checks 236 exact replicas and finishes with both players at 55 health. Three
ordinary revivals each take exactly ten active seconds. The automated controls
use the owned Watch, per-player fan-gap timing and a single-file final inlet;
gameplay and combat rules are unchanged. All 949 active Rust tests pass.
Repeat with `tools/test_coop_native_route.py --map tower2`, local data and a
fresh profile. Coverage is **30/39 visits**, **9 remaining**, or **76.9%** overall.
Internet and rendered-camera evidence remain separate.

## Mystifying Madness accepted

`f9d8893` passes the normal paired maze course through actual Water Logged
arrival: 53,709 ticks, 448 exact replicas, both lever scenes and both breathing
shells. Both finish alive at 80/70.59 health. Five ordinary revivals each count
ten active seconds; the last also waits eight ticks for safe footing.
A simultaneous low-climb lock is fixed, with a passing movement regression,
949 active unit tests and an affected Airborne Terror route pass. The diagnostic
uses ordinary swimming and single-file controls at the narrow exit bank.
Use `tools/test_coop_native_route.py --map hedge2` with local data and a fresh
profile. Coverage is **29/39 visits**, **10 remaining**, or **76.4%** overall.
The final default course on `cfcb764` reproduces the calibrated result exactly.
Internet and rendered-camera evidence remain separate.

## Airborne Terror accepted

`fb8a476` passes the full normal paired wind course into Mystifying Madness:
17,513 ticks, 146 exact replicas, one watched reunion and both players alive
at 21/62 health. Two Blue revivals each count exactly ten active seconds and
then wait for safe footing. A real airborne wall-contact freeze is fixed;
its regression and the affected Battle Royale route pass. The final updraft
controls advance each player's own waypoints without mid-air waiting.
Use `tools/test_coop_native_route.py --map tower1` with the frozen executable,
local data and a new profile. Coverage is **28/39 visits**, **11 remaining**,
or **75.9%** overall. Internet/camera evidence remains separate.

## Pale Realm accepted

The existing Blue-led watched course passes on `841dcd6`: 68,146 ticks, 113
exact replicas, both chess puzzles, the shared lift, timed spikes, both levers
and all five scenes through actual Castling arrival. Both players finish alive
at 92/100 health, with exact personal resources and no deaths, wipes, overlap
or unscripted warps. No native code changed. Repeat with
`tools/test_coop_chess_native.py --exe <frozen-executable> --data <base-directory> --profile <new-directory>`
in Anode. Internet and rendered-camera evidence remain separate. Coverage is
**27/39 visits**, with **12** remaining and **75.3%** overall.

## Both Caterpillar's Plot visits accepted

The first visit (`841dcd6`) reaches Pale Realm in 33,869 ticks with
283 exact replicas and four watched scene reunions. The return
(`d199822`) reaches the maze in 9,720 ticks with 81 replicas
after its original wall scene and ordinary Staff destruction. Both courses
finish with two living players, without deaths, wipes or progression staging.
Personal resources carry correctly, including the return's normal optional
weapon arrival grant. The automated controls changed; gameplay did not.
Coverage is **26/39 visits**, with **13** remaining and **74.8%** overall.
Use `tools/test_coop_native_route.py --map wforest` or `--map wforest-return`
with the required frozen executable, local data and fresh-profile arguments.
Internet/camera evidence is reused separately; optional pickup completion and
exhaustive scene/recovery permutations remain polishing.

## Fungiferous Flora accepted

`906f8c7` passes the fresh paired course into actual Sanctum arrival: 12,471
ticks, 104 exact replicas, one watched ambush reunion and no deaths or wipes.
Both arrive alive with preserved personal resources, allowing the existing
second-Die campaign grant for Red, who missed that optional pickup. The old
visit's temporary health floors clear. The saved native evidence passes the
corrected verifier; gameplay was unchanged. Coverage is **24/39 visits**, with
**15** remaining and **73.8%** overall. Repeat with
`tools/test_coop_native_route.py --map centipede1 --exe <frozen-executable> --data <base-directory> --profile <new-directory>`
in Anode. Separate internet/camera evidence is reused.

## Icy Reception accepted

`cd89026` passes a fresh normal paired route into actual Fungiferous Flora:
21,406 ticks, 35 exact Arena replicas, both personal Ice Wands, both vent rises
for each player and two watched scene reunions. Blue dies and revives after
exactly 1,200 active ticks, with 841 cutscene ticks correctly paused. Both arrive
alive with unchanged personal resources; no wipe, cheats or world staging.
The final automated controls handle planned landings and actual ledge pull-ups.
This credits normal progression with recovery; existing internet/camera evidence
remains separate. Coverage is **23/39 visits**, **16** remaining, **73.3%** overall.

## Rolling Stones accepted

`7710f92` fixes the follower's repeated chase-gate activation and passes a fresh
paired entrance-to-Icy Reception route: 10,699 ticks, 89 exact Arena replicas,
both players at 86 health, no deaths or wipes, and unchanged onward resources.
Eight real-map trigger/order restore cases verify the saved one-way gate policy;
the unchanged solo toggle still repeats. The full Herbaceous route also passes
on this build with its five exact ten-second revivals. Internet/camera evidence
is reused separately. Normal paired coverage is **22/39 visits**, with **17**
remaining; estimated overall completion is **72.8%**.

## Herbaceous Border accepted

`4bb7dac` passes the fresh Blue-led route through actual Rolling Stones arrival:
33,190 ticks, 56 matching Arena replicas, four original scene reunions and five
exact ten-second revivals. Both arrive alive with unchanged personal resources.
Red misses the optional Mallet after revival; the playable gate records that
personal inventory, while strict mode still requires both weapon pickups. This
does not change gameplay rewards or progression. Complete optional pickup
coverage belongs to polish. Repeat with the native runner's `--map garden2`.
Normal paired coverage is **21/39 visits**, with 18 remaining.

## Dry Landing accepted

`5c1b2dd` passes the fresh Red-led paired Garden course with both players alive
through the actual Garden2 arrival. The 29,719-tick run covers both original
scene reunions and 50 matching compressed Arena replicas, without deaths or
team wipes, and preserves all personal resources at arrival. Run
`tools/test_coop_garden_native.py --exe <frozen-executable> --data <base-directory> --profile <new-directory>`
in Anode to repeat that order. The failed Blue-led attempts remain follow-up
work; they do not invalidate the demonstrated playable route or count as passes.
This brings normal paired coverage to **20/39 visits**, with 19 remaining.
See [current evidence](COOP_PROGRESS.md#dry-landing-playable-route-completed).

## Castle Keep accepted

`b22c2b4` passes the fresh default paired Castle Keep route: all three lever
selections and portrait shots, three steam climbs, personal earned essence,
the original final scene and actual Queen arrival. The 80,098-tick run includes
one exact ten-second revival and no team wipe. A repeated-wave reward-ID bug
is fixed with old-save compatibility. Both arrive alive with all personal
resources preserved. Battle Royale and Ascension regressions also pass.
This brings normal paired coverage to **19/39 visits**, with 20 remaining.
See [current evidence](COOP_PROGRESS.md#castle-keep-playable-route-completed).

## Ascension accepted

`4d8acf0` passes the fresh default paired route from Ascension's normal entrance
through its steam climbs, upper crossing, shared lift and actual Castle Keep
arrival. The 63,444-tick run includes five exact ten-second Red revivals and no
team wipe. Both enter each steam column; the second climb uses ordinary revival
recovery for Red, so independent flawless climbs are not claimed. The lift waits
for both riders, and both arrive alive with all personal resources preserved.
The final shared-helper regression repeats the earlier Battle Royale pass.
This brings normal paired coverage to **18/39 visits**, with 21 remaining.
See [current evidence](COOP_PROGRESS.md#ascension-playable-route-completed).

## Normal-route acceptance, 7 November

Twenty-three of 39 visits now have credited normal paired route evidence. The six
historically combined visits remain credited. The playable milestone also
accepts complete ordinary-input routes in UTemple, Sanctum, Castling,
Checkmate and About Face without requiring every polishing permutation.
Clockwork and Funhouse are now also credited through verified normal save
chains: 26 connected sections cover Clockwork's entrance, all 150 course
actions and its About Face exit; 20 sections cover Funhouse's entrance,
156 recorded course actions, the actual Tweedles defeat and the Hatter1 exit.
Each handoff uses the identical saved bytes and a native exact-resume receipt.
Funhouse's edges were rechecked against their frozen movement/combat/scene
assertions, with 18,637 matching state samples and two verified revivals.
These are historical build-specific routes, not fresh uninterrupted runs on
the latest executable. Disconnected or unverifiable receipts were excluded.

Burning Curiosity is also credited through 24 connected normal save/resume
sections, including the original Caterpillar scene, both players' climbs and
actual paired Jabberwock arrival. There are 43,586 matching state samples across
the retained sections. The new upper route and exit use `40d06c0`; earlier
approach evidence remains tied to its historical builds. Optional pickup/Watch
choreography and another uninterrupted full run are deferred to polishing.

Jabberwock's Lair now passes a fresh accelerated two-player production-Arena
route on `bfdf5b3`: original scenes, 90-second survival, both three-enemy waves,
both personal Eye Staff pickups and the actual forest return. The 24,348-tick
run checks 203 compressed arena replicas and finishes at 100/75 health with no
revival or wipe. It is normal gameplay simulation, not a staged component case
or a new internet check. Independent-process public-invite and core-flow
evidence remains required and is already recorded separately.

Royal Rage also passes its normal paired route on `4f1d1ea`: the full boss
battle and original scenes, lowered drawbridge, both walkers reaching the gate,
opened gate, actual Battle Royale arrival and unchanged personal resources.
The 18,802-tick run checks 157 compressed arena replicas and finishes at
100/72.27 health without revival or wipe. The Lair route also passes again on
this shared runner. No gameplay balance or progression rules were changed.

Battle Royale's default normal paired route passes on `57aa72b`: the original
arrival, all 30 recurring guards defeated, personal combat/loot, two exact
ten-second revivals without a team wipe and actual Ascension arrival. Both
players retain their personal resources across the transition. The accelerated
30,506-tick run checks 255 compressed arena replicas in about 19 seconds.
This credits normal progression with recovery, not a flawless or internet run.

UTemple now passes a fresh public-internet entrance-to-Garden run on `c8a041e`,
with 40 ms injected delay and 20 ms jitter. Both clients agree on 2,813
player/world/native state samples. Four actual deaths complete the exact
ten-second countdown, survivor camera and safe 25-health revival; both players
finish alive. This proves normal progress with recovery, not a flawless swim.

The retained full-route histories for Sanctum (`bb4d83b`), Castling (`cc2388e`),
Checkmate (`afe78ef`) and About Face (`3298768`) were re-audited for both clients'
fresh entrances, actual next-map exits, no team-wipe resets and no cheats.
Their original build-specific passes are reused, not described as fresh runs
of the latest executable. Earlier route details and limitations remain in
COOP_CAMPAIGN.md. Sanctum does not prove both players independently finish
every climb; Checkmate includes verified revival past some partner waypoints.

Earlier Garden's fresh strict run stopped at a normal combat death. The automated
controller now keeps its observation clock current during spectating and
discards pre-death movement plans. `42139c0` contains only that test-driver
change relative to `c8a041e`; UTemple gameplay is unchanged. The next attempt
exposed a test-controller backtrack after revival beyond a one-way jump. The
Python harness now continues from the survivor's last witnessed course step,
linked to the exact native revival. Three such revivals/continuations pass a
bounded audit. The subsequent full attempt still fails after both players die
in the late encounter at step 154 and restore the initial checkpoint. A Garden
full-route pass was still required at that point; no team-wipe certification is
inferred from that failed route. The later Red-led native pass above now meets
normal-route acceptance, followed by Herbaceous Border, Rolling Stones, Icy Reception, Fungiferous Flora, both Caterpillar's Plot visits, Pale Realm, Airborne Terror, Mystifying Madness, Water Logged, Labyrinthine Revenge, Machinations and the Queen finale, connected first-school chain and Majestic Maze above. Four visits remain outside credited normal route
coverage; all have paired component evidence.

Remaining normal-route coverage is School Two and all three Pool visits. The rendered finale film/credits handoff is also
still required for the playable milestone.

Run the two fresh-entry route checks together in Anode:

```powershell
python tools/test_coop_regressions.py --exe <frozen-executable> --data <base-directory> --only temple-playable garden-playable
```

Public invites are the default. `--first red` selects Red's arrival order.
The runner retains logs and JSON results, never opens screenshots, and stops
on the first failure. `--allow-revival` in the underlying route scripts checks
exact host recovery traces and both rendered views; it does not ignore deaths,
enable cheats, accept team wipes or grant checkpoint segments full-route credit.
The old strict no-death modes remain available for polishing.
