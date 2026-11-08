# Fungiferous Flora (`centipede1`)

The visit is owned by `levels/centipede1`. It uses the local map, camera track,
voice/subtitle data, animation clips and models without executing original scripts.

- The Centipede stays hidden during exploration. The first voice-over leaves Alice
  free to move. The second Demon Die remains an ordinary, persistent altar pickup.
- Crossing the ambush trigger starts the runners' rush, followed by the reveal,
  yawn/snarl animation, dialogue, fades and return to play. The first two seconds
  cannot be skipped. Scene AI pauses with the scene; dead runners stay dead.
- Watching and skipping commit the same encounter: living runners activate, the
  Centipede disappears, Ant waves start, and a ten-second departure countdown begins.
  The watched version's delay and actor policy are used for both paths. The two
  extra Ants present only in the original skip branch are omitted from both.
- Both spawn chains forward past points visible from Alice, spawning at the first
  occluded point. Facing direction does not alter visibility. The shared live-brave
  cap also gates the killer chain, preserving the original loop's quirk. Four wave
  attempts fit the countdown at 0, 3.1, 6.2 and 9.3 seconds, with up to eight new
  soldiers. Ordinary Ant combat supplies projectiles, melee, damage, death and loot.
- During the countdown, damage cannot reduce existing sanity below ten. This never
  heals or resurrects Alice and is recalculated by the current visit after loading.
- The mushroom lid is visible, solid and eligible for ledge support. The authored
  forest fog distance is applied. Existing Ladybug, plant, Antlion and Snark behavior
  remains with the shared enemy controllers.
- Departure requests `centipede2$centipede2_start1` once, with a bounded retry after
  a failed load. Inventory and resources carry into the Sanctum intro. Older saves
  without this controller restart at the forest entrance with resources preserved.

Use the regular `Launch.cmd` and select Fungiferous Flora, or use
`tools/launchers/Launch-Fungiferous-Flora.cmd` for direct entry into the selected local build.

## Verification commands

- `--centipede1-check`: watched/skip equivalence, 30/60/144 Hz timing, wave ownership,
  damage, pause, malformed snapshots and exit retries.
- `--centipede1-route-check` / `--centipede1-skip-route-check`: ordinary movement and
  weapons on Normal and Hard from the entrance through the altar and ambush into
  the Sanctum intro. A checkpoint during live countdown combat must continue with
  identical actors, resources and departure after restoring.
- `--centipede1-render-check`: forest, runner rush, Centipede reveal, waves and lid.
- `--centipede1-save-write` / `--centipede1-save-read`: native saves for exploration,
  speech, rush, reveal, outro, countdown, active/dead wave actors and committed exit;
  fresh-process restore and migration from a previously consumed ambush trigger.

The headless route includes the visit-owned Ants and world hazards. The other enemy
families use the native viewer's shared NPC simulation and its existing family checks.
Spawn chains and soldier slots are bounded; exact original AI physics is not claimed.
Research extracts, captures, saves and local builds remain under `private/`.

The installed build passes all four Normal/Hard watched/skipped walking routes,
including continued countdown combat after restoring and the Sanctum intro.
Eight native save cases, legacy recovery and five native scene captures pass.
The shared suite passes 616 unit tests; the neighboring Centipede2 and Wforest
routes and existing weapon, swimming and enemy checks also pass.

## Paired adapter

Co-op now advances one shared exploration/wave/departure clock and gives the
level-owned Ants both living player targets. Native Ant behavior chooses the
nearest visible eligible player and routes hits to that player's feedback.
The original wave budget and shared live-brave cap still govern both chains.

Wave candidates must be hidden from every living player. The first co-op wave
is chosen on the first free update after the ambush reunion, so it uses both
players' final positions. Dead players do not block spawns. Single-player keeps
its existing immediate first wave and one-observer selection.

The actual triggering player supplies the original scene's Alice pose. An
optional saved co-op owner preserves that binding; older saves omit it. The
countdown's existing health floor is recomputed for each player without healing
low-health players or reviving dead ones. Shared departure carries resources
individually, while the next visit clears the previous visit's temporary health
floor before applying its own rules.

`--centipede1-coop-check` stages shared-clock, native Ant targeting/projectile,
spawn visibility/cap and original ambush owner/reunion cases, including a dead
partner. These checks are separate from full paired traversal, independent
internet clients, rendered effects and the death/wipe/rejoin matrix.

Source `e3d05551a704149b7a3e1fd352237379851b59141c892fb9daa11375ab38ae52`
passes **31 native co-op cases**: nine 30/60/144 Hz clock/wave comparisons with
both players or either dead partner, a dead-team freeze, four paired spawn-view
cases, the live-brave cap/ordinary kill, six targeting cases, two projectile
victims and eight scene-owner/watch/skip/dead-partner cases. The locked release
build and 809 active unit tests pass, including the health-floor transition
regression; one existing test remains ignored.

The first native run caught a skipped-scene failure on source `31b8a9b`: after
skip committed the scene, the old scene update selected a wave before reunion.
The controller now leaves the co-op Countdown phase entirely to the paired free
update, so waiting for safe reunion consumes none of the ten seconds. Both
watched and skipped cases assert zero waves and zero elapsed countdown time at
handoff, then the full shared countdown and one departure into the Sanctum.

Evidence: `private/coop-centipede1-components-e3d0555.log`,
`private/coop-centipede1-unit2.log` and the failed
`private/coop-centipede1-components-31b8a9b.log`. The existing single-player
component suite passes with a complete log identical to the `b7ba057` baseline
(`private/coop-centipede1-single-e3d0555.log`). Both watched/skipped route commands
also pass on Normal and Hard through the Sanctum intro, with complete logs
identical to baseline (`private/coop-centipede1-route-e3d0555.log` and
`private/coop-centipede1-skip-e3d0555.log`). No images were viewed. These results
do not complete the campaign row.

## Normal paired playable route

Frozen `906f8c7` passes `--centipede1-coop-route-check` through actual Sanctum
arrival in 12,471 ticks with 104 exact replicas and one watched ambush reunion.
Both players arrive alive at 10/47.57 health, without deaths or wipes. Blue
collects the second Die; Red receives the existing campaign weapon grant on
arrival. All other personal stats carry exactly, and both temporary health
floors clear. Normal gameplay was unchanged.

The final native run succeeds; the corrected Python verifier accepts its saved
report, including the final scripted departure taking ownership on the exit
tick. The earlier failed verifier result remains preserved. Evidence is pinned
in `private/coop-playable-flora-route-verified.json`. Repeat with
`tools/test_coop_native_route.py --map centipede1` and its required executable,
data and fresh-profile arguments. Internet and rendered presentation are
covered separately.
