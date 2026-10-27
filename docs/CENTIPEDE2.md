# Centipede arena (`centipede2`)

The visit is owned by `levels/centipede2`. It uses the supplied BSP, animation clips,
camera tracks, voices, models and particle declarations. Original scripts are not executed.

## Progression

- Arrival plays the ant escort/shove and teeth sequence. Watching or skipping commits
  the same slide position, open teeth and return barrier, without refilling resources.
  The slide temporarily prevents damage from reducing existing sanity below 30; the
  authored arena trigger removes that protection. It cannot heal or resurrect Alice.
- The Centipede has three stages, with two successful weak-point reactions per stage.
  Its animated armor boxes absorb hits. The underside at `tag_target` accepts damage
  during the crush animation before frame 25; fire remains ineffective. One attack
  window produces at most one reaction, regardless of splash damage or pellet count.
- Attacks include approach/contact, crush, acid spit, Larvae, charging, and the final
  stage's grab/toss. Acid splits into six smaller damaging projectiles on impact.
  Larvae use the shared leap, attach, suck, detach, death and freezing behavior.
- The sixth reaction plays `death_start`, three thrashes and the falling-spike scene.
  All entities sharing each spike target name move together. The original inactive
  tip movements remain inactive; every tip is retired before the climbing geometry
  appears. The settled steps support jumping, ledge grabs and pull-ups.
- Climb to the mushroom and enter its trigger. Cheshire's two conversations, eating,
  camera cuts and growth play before departure to `wforest$wforest_start1`.
  The old ground-level `c2_changelevel` volume stays disabled, even after victory.
- Medium essence cycles through the three authored stations, ten seconds after each
  collection. Demon Dice allies are dismissed in this boss arena. Recovery after the
  intro returns to the accessible arena rather than behind the closed barrier.

No boss defeat, pickup, dialogue, or departure can be committed by an unrelated or
out-of-order completion callback. Failed destination loads retry the committed exit.
Inventory, turtle-shell ownership, sanity and Will carry into the next map.

## Persistence and checks

The versioned controller snapshot owns the stage, weak-point hits, deterministic
combat clock/random state, Larvae, projectiles, grab, spike timing, essence station,
dialogue phase and exit transaction. Controller-less saves restart this changed visit
at the entrance while preserving carried resources. Pausing holds simulation clocks.

- `--centipede2-check`: weak-point boundaries, fire/armor rejection, repeated hits,
  attack timing at 30/60/144 Hz, Larvae, grab release, pause, scene skipping, death
  gating, spike completion and exit retry.
- `--centipede2-route-check`: Normal with watched intro and Hard with skipped intro;
  production movement and weapon input, all six weak-point hits, jumping/climbing,
  both conversations, growth and actual next-map loading. No victory writes or
  position warps are used to complete the route. The authored scenes own their warps.
- `--centipede2-render-check`: native frame captures of arrival, weak point, acid,
  Larvae, grab, spikes, climbing route and growth.
- `--centipede2-save-write` and `--centipede2-save-read`: eleven native Store/Restored
  fixtures, including the grabbed player, active Larvae, falling spikes, dialogue and
  a committed exit; the reader runs in a separate process. A controller-less bypass
  save also verifies safe entrance migration without changing carried resources.

## Implementation boundaries

Movement steering, fragment spread, grab interpolation and cosmetic impact fade are
independent bounded implementations. The malformed original second-Larva launch
token is treated as 250 units/s. Live Larvae are capped at 16 and acid carriers at 128;
expired slots are reused. Damage traces use the animated tag boxes; player separation
is handled independently so a solid enclosing box cannot hide the weak point.
The damage window, stage count, attack frames, spike groups/distances and exit timing
come from local asset facts. This is not a claim of identical native-engine physics.

Private research extracts, captures, test logs and local builds remain under `private/`.

## Co-op adapter

The paired adapter advances one boss clock and chooses the closest visible living
Alice. Acid and melee contacts consider both living bodies, while a grab binds
only its victim's movement and throw. The other Alice retains her controls.
Larvae use the existing paired targeting and personal damage paths. Boss holds
test the partner's space as well as world clearance, and the co-op grab uses the
existing player model rather than adding the single-player scene puppet.

Each player has a separate rotating essence station and ten-second cooldown;
the local view shows that player's available station. Slide health protection is
removed per player at the authored arena trigger. Saved paired state includes
the boss target/grab, personal healing, arrival flags and swept trigger positions.
Older single-player snapshots omit these fields and retain their existing path.

`--centipede2-coop-check` stages native-map combat, grab/save, healing and original
intro handoff cases. Independent internet clients, broader rendered effects and
rejoin remain open; production Arena and full native route results follow.

`--centipede2-coop-arena-check` runs ten focused cases in the production two-player
game loop with a native window. It covers both grab victims, movement/weapon
locking, personal acid damage, funded Cards hits on the animated weak point,
native acid deaths and timed revival, a paired checkpoint wipe, and a separate
guest Arena restoring compressed snapshots while both views submit particle
batches. It reports assertions and render-queue counts without image inspection.
Full traversal and separate-process internet/rejoin checks remain distinct gates.

The ten production Arena cases pass on source
`ead0be326db7169fb9ad48c815e924ba96c3118f98e653ba30949f1636efb7e0`.
Both grab victims retain isolated control locks while the partner moves; funded
Cards hit the native animated weak point, and acid changes only the struck
player's health. Both actual acid-death cases follow the survivor for 1,200 ticks
before reviving at 25 health with personal Will preserved. A simultaneous acid
wipe restores both bodies/resources and the saved boss. Thirty compressed
host/guest snapshots match exactly, and both particle passes draw 8,287 batches
with zero drops. The check completes in Anode in about 19 seconds without image
inspection. Evidence: `private/coop-centipede2-arena-ead0be3.log`.

`--centipede2-coop-route-check` adds an ordinary-input route through the complete
visit, using the same climbing waypoints as the single-player check. It checks
body spacing, frozen scene partners, original exit ownership, guest restoration
and carried resources at the next visit. Failures retain the complete shared
state and 240 recent input frames in the isolated settings directory. Optional
selectors are `LOOKING_GLASS_SANCTUM_COOP_FIRST=Blue|Red` and
`LOOKING_GLASS_SANCTUM_COOP_SCENES=watched|skipped`; the default runs all four
combinations. Its full-route results are recorded separately from staged cases.

The first Blue/watched run on source `9628160` defeats the boss through six
ordinary Cards hits, with both players at 51 health and two scene reunions.
It then fails at tick 8,324 while both players pull onto the first ledge:
horizontal spacing is about 28 units and vertical spacing falls below 56.
Both bodies were excluded from ordinary separation because their pull-up paths
were active. This is a real contact defect, not a reason to loosen the check.
The captured state and recent inputs are in the isolated
`private/coop-centipede2-route-9628160/` profile; the run log is
`private/coop-centipede2-route-blue-watched-9628160.log`.

Source `0edf86ad6e95e25db8e88c19ff3edfa0af06abb969c394ca765c9decc3f74969`
fixes that contact by pausing only an approaching climber's path and pose while
their partner occupies it. Personal timers still advance; the partner is not
shoved off the ledge. The shared continuous-contact calculation includes exact
vertical boundary contact. Both-role regression cases cover the recorded
pull-up geometry, stationary low-step blockers, saved waits and continuation
when the partner moves or dies. All 812 active unit tests pass (one existing
ignored), along with the locked release build.

All **four ordinary-input paired routes pass** on this source: Blue watched
12,716 ticks, Blue skipped 10,526, Red watched 12,675 and Red skipped 10,485.
Each lands six funded boss hits, keeps both bodies alive without respawn or
wipe, preserves scene ownership, and enters `wforest` with carried resources.
The leading player finishes at 51 health; the trailing player takes a later
climbing fall to 36 health and remains behind until the scene/level handoff.
Thus this proves an entrance-to-exit session with the first-arrival rule, not
that the trailing player independently completes every climb. Across the four
routes, 80 sampled compressed guest Arena states match exactly. Evidence:
`private/coop-centipede2-route-all-0edf86a.log` and the four full-state reports in
`private/coop-centipede2-route-0edf86a/`. The matrix takes about 17 seconds in
Anode; no images are inspected.

The same source repeats all 28 native component cases and ten production Arena
cases successfully, including the two actual particle passes (8,287 batches
each, zero drops), individual death recovery and checkpoint wipe. The existing
single-player component log is identical to baseline, as is the route's stdout;
the captured route log additionally includes the known Hard-route error on
stderr. Normal still passes and Hard fails at the same position. Logs end in
`components-0edf86a.log`, `arena-0edf86a.log`, `single-0edf86a.log` and
`single-route-0edf86a.log` under the `private/coop-centipede2-` prefix.
Separate-process internet traversal and reconnect checks remain open, so the
campaign row remains partial and the weighted estimate stays 44.6%.

`tools/test_coop_sanctum.py` drives two independent rendered clients through the
public invite path, using 40 ms delay and 20 ms jitter per sender by default.
`--first blue|red` selects the intended mushroom-scene arrival; the follower
waits below the last platform rather than racing the chosen leader. Each client
plans from its own received world and sends ordinary controls. Delayed clients
settle at climb waypoints and send a pull-up jump only before pulling begins,
so stale jump edges do not cause an extra hop after landing. `--rejoin` uses
the normal host save operation after defeating the boss, leaves/restarts the
guest, and checks personal resources and completed progress before continuing.
The saved checkpoint and failure reports remain in its isolated private profile.
This fixture uses state assertions and matching sent/restored hashes, with no
image captures; runtime evidence is recorded separately from its availability.

Source `bb4d83b492c995093eab574ae752955c0ecd539cad5f3032de0cd8d2a5f71890`
passes a **complete Blue-first internet route with a fresh guest rejoin**. Both
independent clients use the public invite path with 40 ms delay and 20 ms jitter
per sender; this run sets `--input-lead 0.04 --first blue --rejoin`. It watches
the entrance, lands all six funded boss hits, saves through the normal host
operation, restarts the guest, completes the mushroom scenes with Blue as owner,
and enters `wforest` alive at 51/49.5 health. All 1,874 matching player, world and
native-resident states agree. Both clients exit successfully. Evidence:
`private/coop/sanctum-route-a90c37b7f8d54d69ae30417854abf9d1/result.json`.
The exact fixture source/hash, normal checkpoint, histories and logs are retained
there. No images were captured or inspected.

The same source repeats **all four native paired routes successfully**, with the
same tick counts and outcomes as `0edf86a`; evidence is
`private/coop-centipede2-route-all-bb4d83b.log`. All 812 active unit tests and the
locked release build pass. The network driver is opt-in test automation: it
reads the client's own received world and sends ordinary controls without
changing gameplay physics, positions, resources or progression.

Red-first internet traversal is **still open**. One earlier full session reached
the next visit but Blue overtook the intended leader, so it fails the Red-owner
assertion. Subsequent captured-checkpoint climb tests stop automatically when
Red's delayed steering stalls, including a waypoint blocked by the follower
after Red falls and returns. The saved checkpoint is authentic; `--checkpoint`
results are explicitly segment-only and never count as entrance/boss traversal.
The latest failing segment is retained in
`private/coop/sanctum-route-66dacc6d323442deb41abb77b812f312/`.
Further work must improve the ordinary-input controller without weakening
collision or scene-owner assertions. Separate-client combat/recovery, broader
effects and both players independently finishing every climb remain unverified.
The campaign row stays partial and the overall estimate stays **44.6%**.

Source `993d722cbcdd1ad904d6ff66269e3a5fbf5ee54c245e816959bedebe3ea94c9b`
passes **28 native co-op cases**: eight target/acid/melee/grab cases, six attack
timing cases at 30/60/144 Hz, two personal healing cases, eight watched/skipped
intro handoffs with either owner and a living/dead partner, two individual slide
arrival cases, and two boss-defeat/mushroom progression cases. The latter stage
six native weak-point reactions, then run the actual death/thrash/spike clocks,
saved scene continuation, reunion, a short walk into the original mushroom
trigger, both conversations and the committed exit. They do not traverse the
complete fight or climbing route. The first exit fixture incorrectly started
at the acting marker outside the trigger; the corrected fixture walks into it.

All **810 active unit tests** pass, with one existing ignored test. The new
unit regression checks that a boss drag cannot cross or end inside the living
partner; a dead partner does not obstruct it. The locked release build passes.
Evidence: `private/coop-centipede2-components-993d722.log`,
`private/coop-centipede2-unit.log` and `private/coop-centipede2-build2.log`.
No images were viewed and no desktop session was used.

Before this adapter, source `e3d0555` passes the existing component check and the
Normal watched route (12,039 ticks, 13 Sanity, followed by the real `wforest`
arrival). The Hard skipped route fails during combat at
`(-3332.1218, 3946.9988, 2848.0313)`. Logs:
`private/coop-centipede2-baseline-e3d0555.log` and
`private/coop-centipede2-route-baseline-e3d0555.log`. This is the regression
baseline; earlier full-route claims do not establish a current Hard-route pass.
The final adapter's complete component and route logs are identical to that
baseline: `private/coop-centipede2-single-993d722.log` and
`private/coop-centipede2-route-993d722.log`. The current Hard-route failure remains
open; it was not introduced or hidden by the paired adapter.
