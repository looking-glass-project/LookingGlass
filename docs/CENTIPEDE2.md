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
intro handoff cases. Full paired boss traversal, production Arena integration,
independent internet clients, rendered effects and recovery/rejoin remain open.

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
