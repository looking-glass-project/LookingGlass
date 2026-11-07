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

Nineteen of 39 visits now have credited normal paired route evidence. The six
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

Garden's fresh strict run stopped at a normal combat death. The automated
controller now keeps its observation clock current during spectating and
discards pre-death movement plans. `42139c0` contains only that test-driver
change relative to `c8a041e`; UTemple gameplay is unchanged. The next attempt
exposed a test-controller backtrack after revival beyond a one-way jump. The
Python harness now continues from the survivor's last witnessed course step,
linked to the exact native revival. Three such revivals/continuations pass a
bounded audit. The subsequent full attempt still fails after both players die
in the late encounter at step 154 and restore the initial checkpoint. A Garden
full-route pass is still required; no team-wipe certification is inferred from
that failed route. Twenty visits remain outside credited normal
route coverage; all have paired component evidence.

Remaining normal-route coverage is first Skool, School Two, all three Pool
visits, all four Garden visits, Centipede One, both Wood visits, Pale Realm,
the six Maze/Tower visits and the Queen finale.

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
