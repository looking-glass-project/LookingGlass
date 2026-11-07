# Caterpillar's Plot — both visits

The `wforest` and `wforest-return` registrations own separate states in the existing
`wforest$first` and `wforest$return` save slots. Original data is read locally; no
script bodies, dialogue text or assets are distributed with this implementation.

## First visit

Run `tools/launchers/Launch-Caterpillars-Plot.cmd`. The arrival Cat scene starts once. Touch the
Staff altar on the pillar to collect the Staff component and play the gate scene.
The component is a saved quest item, not the usable Eye Staff. The three cavegate
brushes lower together. Either western scene volume starts the same Caterpillar
conversation, which raises the chessgate. The chess approach plays its Cat scene,
and the gated exit leads to `wchess1$wchess1_start1`.

The Humpty button and Blunderbuss pickup are unavailable here. The hedge route
cannot transition. As in the supplied scripts, the Caterpillar remains hidden
during this conversation; the five voice beats and four camera paths still play.
Watched and skipped scenes commit the same progression. Cat and Alice puppets
have a single owner, and grounded handoffs are checked against live collision.

## Return visit

Run `tools/launchers/Launch-Caterpillars-Plot-Return.cmd`. The cavegate and four chesswall brushes
seal the first route, and its level-change trigger is disabled. The working Eye
Staff is granted if missing. Humpty performs his ambient cigar loop. Touch the
button beside him: it retracts for two seconds, then the secret door rises for
four. The optional Blunderbuss altar starts the Cat's appearance, line and departure.

The wall warning plays once, with the voice identified as the Gryphon rather than
Alice. The wall has 100 health and accepts Eye Staff beam, projectile and splash
damage; other weapons cannot open it. Private inspection of the original wall's
flag-8 damage branch and Staff damage discriminator confirmed this filter.
Partial damage persists. Breaking the wall suppresses an unplayed warning,
removes collision, starts two bursts, seven moving brush fragments and fifteen
model fragments, then opens the two hedge doors. Alice has brief scripted damage
protection during the collapse. Only the completed return route can request
`hedge1$hedge1_start1`; failed transitions can retry without replaying the scene.

The two essence spawns alternate after pickup with a ten-second delay. The optional
secret does not gate the hedge exit. Wall burst particles and deterministic small
rubble trajectories approximate the original cannon-driven destruction effect.

## Persistence and compatibility

Controller version 1 saves scene identity, line cursor, shot clock, completed
scenes, Staff pickup, mover clocks, secret progress, wall damage/destruction,
essence cycle and exit commitment. Cross-visit or inconsistent state is rejected
before replacing live state. Pauses freeze these clocks. Older saves without the
controller use the shared upgrade path to rearm formerly pending progression
threads and recover obstructed positions. Previously acquired inventory is kept;
an early Blunderbuss from an older build is not confiscated.

The additive `staff_component` and `scripted_immunity` inventory fields default
for older saves; the overall save format remains 12. Eye Staff damage retains
the existing generic enemy reaction while carrying a distinct wall discriminator.
Other enemy families remain under their existing owners.

## Verification

- `--wforest-check` and `--wforest-return-check`: watched/skipped scenes at 30, 60
  and 144 Hz, dialogue counts, pause/restore, grounded handoffs, duplicate-scene
  rejection, visit isolation, wrong exits, weapon filtering, partial wall damage,
  early destruction and transition retry.
- `--wforest-route-check`: continuous movement from first entry through the altar,
  cave, Caterpillar clearing, chess approach and exit.
- `--wforest-return-route-check`: the optional Humpty reward route and the mandatory
  return route, including actual Eye Staff input and the hedge transition.
  These navigation checks disable enemy targeting; they are not combat balance tests.
- `--wforest-render-check` / `--wforest-return-render-check`: native captures of
  the altar, scenes, Humpty, collapse and open hedge doors.
- `LOOKING_GLASS_SAVE_CASE=wforest-` with `--save-check-write` then
  `--save-check-read` in separate processes: moving Staff gate, half-open secret
  with damaged wall, and destruction in flight. Exact restored state and continued
  simulation are compared.

Evidence and captures are retained privately under `private/wforest-visits`.
Project-wide strict Clippy currently reports unrelated existing warnings; no
WForest diagnostics were reported.

## Co-op adapter

Both visits have an explicit paired update for the cave/chess gates, Humpty's
secret door, wall destruction and hedge doors. The world clock advances once;
each living rider is checked before the shared mover commits. An obstructed
rider pauses the movement for everyone, and dead bodies do not stop it.
These gates are passages, not shared passenger lifts.

The Staff altar chooses the earliest living swept contact and binds its original
single-Alice scene to that player. The quest component is shared progression;
health, Will, usable weapons and ordinary pickups remain personal. On the return
visit, each player has their own alternating essence station and ten-second
cooldown, with the local view displaying that player's available station.
Saved paired fields are optional, leaving existing single-player saves intact.

The Staff scene ends on a narrow pillar. Its completed co-op handoff explicitly
permits a partner landing within 256 horizontal units and 256 units below the
actor when the usual nearby search fails. The actor stays at the original
handoff; the fallback retains clear body paths, full footprint support and hazard
rejection. Other scenes and ordinary death revival retain their smaller search.

`--wforest-coop-check` stages native checks for shared mover clocks at
30/60/144 Hz, individual healing, saved altar sweeps, blocked/living/dead riders,
both scene owners, watched/skipped scenes, frozen partners and safe reunion.
This fixture is image-free. Full traversal, independent internet clients,
rendered effects and the gameplay death/wipe/rejoin matrix remain separate gates;
component runtime results must be recorded before crediting this adapter.

Source `2990c668baa25eeece66fa585dcca9f88ec8d65facc94a12b0564d2181e9155e`
passes all **62 staged co-op cases** in about five seconds without a window or
image inspection. Six compare native shared gate/destruction clocks with the
single-player controller at 30/60/144 Hz, including pause and saved continuation.
Eight cover both individual healing slots and both saved Staff arrival slots,
with a living or dead partner. Forty run the five original scenes with either
owner, watched/skipped outcomes and a living/dead waiting partner; they verify
compressed state restoration, frozen partners, personal resources and reunion.
The other eight isolate the native chess-gate mover/collider to test both rider
slots, obstruction and dead-body exclusion. Its top is inside the authored
tunnel ceiling, so these isolated carry tests do not claim a playable gate ride.
Evidence: `private/coop-wforest-components-2990c66.log`.

The scene matrix exposed an actual paired handoff deadlock on source `ed0a5c1`:
the Staff scene and cave gate completed, but the original pillar position left
no landing within the usual reunion search. The actor remained at
`(5358, 1182, 216.03125)` and the scene owner never released. The bounded,
Staff-perch-only fallback above resolves all four living-partner Staff variants;
the captured failing state remains in
`private/coop-wforest-staff-perch-ed0a5c1.json`. It does not alter ordinary death
revival, other scene searches, or single-player handoffs. Full paired traversal,
combat, independent internet clients, rendered effects and recovery remain open
for both visits.

The same source passes **813 active unit tests** (one existing ignored test),
including bounded perch placement rejecting walls, hazards and invalid limits
while ordinary revival retains its previous range. The locked release build
passes. Both single-player component suites and both full route logs are
identical to pre-adapter source `bb4d83b` after decoding/line-ending normalization.
Evidence: `private/coop-wforest-unit-final.log`, `private/coop-wforest-build5.log`
and the four `private/coop-wforest-{first,return}[-route]-2990c66.log` files.
The paired suite does not exercise native enemies, actual optional Blunderbuss
collection or an independent rendered guest. Both campaign rows stay partial;
the verified shared progression adds 0.25 points per visit, taking the weighted
overall estimate from 44.6% to **45.1%**.

The integrated build passed 616 unit tests, both scene contracts, both walking
routes (including Humpty's optional reward), and the Centipede2, Garden4, Queen,
swimming, world, loadout, event and heavy-weapon checks. The older all-map
`--story-check` still stops at `garden2/Garden2_Cat_End`, identically to the prior
published build. WForest's own voice and trigger checks pass independently.

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
