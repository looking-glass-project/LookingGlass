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

The remaining controller audit is tracked in COOP_CORE_TESTS.md. A visit is not
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
normal-route acceptance, followed by Herbaceous Border, Rolling Stones, Icy Reception, Fungiferous Flora, both Caterpillar's Plot visits, Pale Realm, Airborne Terror and Mystifying Madness above. Ten visits remain outside credited normal route
coverage; all have paired component evidence.

Remaining normal-route coverage is first Skool, School Two, all three Pool
visits, Majestic Maze, Water Logged, Labyrinthine Revenge, Machinations and
the Queen finale.

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
