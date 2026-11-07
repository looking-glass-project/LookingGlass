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
3. Run a consolidated campaign playability pass with two clients. Record each
   visit's progression and exit; exercise the essential save, death and rejoin
   flows at representative points.
4. Make the playable build available for hands-on testing. Publishing a release
   or merging still requires the user's later request.

The remaining controller audit is tracked in COOP_CORE_TESTS.md. A visit is not
proven playable by having an adapter alone: its normal progression must work.
The broader historical matrix in COOP_CAMPAIGN.md remains useful evidence, but
its exhaustive combinations do not all gate the playable milestone.

## Polishing stage

Defer exhaustive frame-rate/difficulty combinations, every scene owner and skip
timing, repeated full traversals of unchanged levels, rare save/network failure
combinations, complete per-visit recovery matrices, visual/audio comparisons
and presentation refinements. Promote a deferred item immediately if it causes
a normal-play blocker, data loss, or a missing requested core feature.

Report playable readiness and the polishing backlog separately. Do not label
the game 100% playable while a level or requested core flow remains blocked.
