# Castling — Wchess2

Single registered owner for campaign index 20. Entrance `wchess2_start1`; exit `rchess1$rchess1_start1`. Hit IDs remain in the reserved range starting at 8,000,000. Save envelope 12 and existing trigger identities are unchanged.

## Doors and world

All 32 rotating leaves and the small sliding secret use the visit owner. Rendered and colliding poses agree. The shared door system retains stable save entries but disables its duplicate colliders, including after restore. The 4096 bit is not a lock; 64 enables player proximity, 16 excludes actors, and explicit script locks remain separate. Door proximity uses the native 60/60/8 expansion; a nearby accepted activator renews the authored wait. Direct use retains facing, reach and wall checks for the manual secret. Castle front doors open during the Queen scene.

Native door findings: constructor 0x100631b0, linking 0x10062800, proximity 0x10064690, use 0x10063c40, completion 0x100623b0. Confidence: high for flag meanings and expansion; medium for exact simultaneous leaf linking and collision reversal fidelity.

## Royal scenes

`cinema_queen_abduction_thread` uses both supplied camera tracks, ten parent waypoints, bound Queen/abductors, knight approaches, fades and moving red platform. Dead chess props retain their frozen source frames until removal. Watching retains Alice's captured approach; skipping lands at `alice_queen_dest1`. Both remove the same cast and platform, open the castle and release the two knights/pawn spawns after the closing delay.

`cinema_king_thread` restores Alice/King approaches, rook movement, the authored camera cuts and holds, eleven dialogue calls, supplied voice/lip bindings, pawn spline/shrink/pickup presentation and weapon-change performance. The pawn scene does not grant an inventory weapon. Both completion paths land at `alice_king_dest2`, make the King solid, remove the specified battle/side actors, enable late ambush groups and start the rook paths to the portal. Dialogue is divided at the authored pawn presentation, retaining source call order and one-second gaps. Saved phase, shot, line and actor clocks resume without restarting rewards or scene completion.

The portal is hidden and non-solid before the King finishes. Its supplied four-node loop drives the brush, collision, rider carry and bound exit trigger together. The exit is one-shot per current visit and can retry a failed destination load. A restored committed exit is delivered again until the destination actually loads.

## Encounters and compatibility

Placed and spawned chess pieces belong to this owner; the generic NPC renderer and combat fallback defer. Explicit duels select the scripted opponent. White battle pieces retain 10,000 health; royal rooks retain their health floor. Side ambush alternatives exclude one another; battle two disables earlier side triggers. Broken source references are preserved as inactive follow-ups rather than invented kill gates: neither battle counter blocks the route, group four's swapped spawner names do not create actors, and the unreferenced third battle stays dormant.

Pre-controller saves restart this visit at its entrance with carried resources retained. Previously consumed scene/exit triggers are rearmed behind the new gates. Normal saves retain doors, actors, scene clocks, portal epoch and exit commitment. No source dialogue or script prose is stored here.

## Verification

### Paired combat components

Source `025b004a49b70f3b5e6fd03aeae9bd0946e60e0b4dd489df4105f455b185999d`
adds two personal targets to this visit's native chess cast. Ordinary red pieces
choose the closest visible living Alice; explicit faction duels keep their
scripted opponents. Attack damage, impulses and will drain return to the selected
player. Saved target ownership keeps an in-progress attack aimed at the same
player after restoration. Older single-player actor saves omit the optional
target field, and invalid targets fail validation before modifying state.

`--wchess2-coop-cast-check` passes **48 component cases** using the real roster
and animation timing with controlled collision fixtures: all four enemy families
and both player roles, hidden/dead/occluded targets, paused combat, personal
attacks and sounds, saved committed bishop beams, faction duels, knight shields,
royal health floors, one-shot battle groups and cinematic combat freeze.
All **834 active unit tests** pass, with one existing ignored test. The complete
single-player component, watched-route and skipped-route logs match the
pre-change `d7bbc99` baselines. Evidence is retained in the private Castling build
manifest and `coop-wchess2-*-025b004.log` files.

Those checks verify combat components only. The shared world adapter below now
owns scripted cast movement once per world update; paired combat does not advance
it again. Continuous native traversal is verified below; independent internet
clients and recovery/rejoin remain open.

### Paired machinery and royal scenes

The paired adapter advances the original door, cast, Queen-release, fade and
portal clocks once. Either living player can activate proximity doors or use the
manual secret. A moving door checks its swept leaf against both living bodies,
including intermediate rotations; an obstruction holds the last safe pose and
reopens a closing leaf. Dead players do not block machinery.

The portal waits while one rider boards and the living partner remains outside
its boarding area. Carry is atomic: a blocked rider holds the shared slab and
both bodies, then both resume when the obstruction clears. Its bound exit also
waits for all living players, preserves the first waiting owner through save/load,
and transfers ownership if that player dies. Ordinary exits without an explicit
transport guard retain their existing first-arrival behavior.

Registered trigger volumes now follow their controllers after either paired
update path and immediately after restoration. Previously the paired path could
move Castling's visible platform while leaving its exit volume at the editor pose.
The native boundary checks distinguish those two positions.

Source `ac3a080c59aa00cd300a918fed0c38ebe78e51019b9b12637c59a67c3bd5aa1f`
passes **45 staged native cases** with `--wchess2-coop-world-check`: three
single-player clock comparisons (30/60/144 Hz), four paired portal rides with
living/dead partners, two blocked-rider/resumption cases, ten door cases, sixteen
royal scene combinations, six saved waiting-exit cases and four moving-volume
checks across both update paths and restoration. Scene cases cross the original
approach triggers with either Alice first, watch or skip both original scenes,
keep the partner frozen, save mid-scene and verify supported, separated reunion
with personal resources retained. These are staged components, not a full route.
The final repeat also passes all **48 cast cases**, **834 active unit tests**
(one existing ignored), **82 shared-world cases**, **39 campaign entrance/four
underwater-recovery cases**, and **119 Pale Realm world cases**. All three complete
Castling single-player logs match the `d7bbc99` baseline. Build identity and binary
hash are retained in `private/coop-wchess2-world-build.json`; individual logs use
the suffix `ac3a080`. No new complete campaign row is credited by this milestone.

### Continuous paired traversal

`--wchess2-coop-route-check` runs the production two-player Arena from the real
Castling entrance through all **203 route legs** into a loaded Checkmate visit.
It uses ordinary movement, jump, door-use, weapon and scene-skip inputs. Waypoints
steer the players; the driver never places bodies, grants resources, kills actors
or changes script state. Blocked approaches use short movement plans against the
current geometry, allowing later proximity doors to open on approach. At the
portal, players jump from the ledge to enter the shared boarding area together.

Source `a80ef8fa922d13e923c7135463d0fb12ec48c78725834f11c9eebe26d32e8db7`
passes all four continuous runs on Normal difficulty:

| First player | Scenes | Simulation ticks | Exact guest-world comparisons | Final Blue / Red health |
| --- | --- | ---: | ---: | ---: |
| Blue | Watched | 38,775 | 64 | 75 / 100 |
| Red | Watched | 38,761 | 64 | 100 / 75 |
| Blue | Skipped | 26,071 | 43 | 95 / 100 |
| Red | Skipped | 26,057 | 43 | 100 / 95 |

Every run verifies both original royal scenes belong to the first player, the
partner stays frozen during each scene, both reunions complete, and the players
remain separated during gameplay. Death, checkpoint rollback, unscripted warps,
god mode and notarget invalidate the run. Compressed world snapshots restore
exactly into a separate guest Arena. Checkmate loads with clear, separated
entrance positions and each player's exact personal resources retained. All
**834 active unit tests** also pass on this source, with one existing ignored test.

The default command checks all four combinations. Optional selectors are
`LOOKING_GLASS_CASTLING_COOP_FIRST=Blue|Red` and
`LOOKING_GLASS_CASTLING_COOP_SCENES=watched|skipped`. Set
`LOOKING_GLASS_SETTINGS_DIR` to an isolated output directory. Each route writes
before-leg states, its recent input history and a final passed/failed record.
Evidence is retained under `private/coop/castling-route-a80ef8f-*`, including
the frozen executable identity and full logs. Verification used state/log checks
without screenshot inspection.

These runs establish continuous native traversal, not independent network play.
Internet latency, disconnect/rejoin, death/wipe recovery and broader effects
coverage remain open for Castling. Its campaign row and the overall weighted
estimate therefore remain unchanged.

`--wchess2-check` exercises real approach triggers, both royal outcomes, dialogue, pause, save futures, moving-platform support, bound exits, ambush alternatives and migration. `--wchess2-route-check` and `--wchess2-route-skip-check` traverse from the real entrance with Normal combat into a loaded Checkmate visit. Route checkpoints are private diagnostics; they do not change player saves. `--wchess2-render-check` captures eight staged views. Native save fixtures are `wchess2-queen-mid`, `wchess2-king-mid`, and `wchess2-exit-open`.

Use `tools/launchers/Launch-Castling.cmd` for a fresh visit with separate playtest saves. Release identity, exact test results and limits are recorded in `private/wchess2/WORKLOG.md`.

Native pathfinding, turn-rate details and precise sub-frame camera/animation phase remain fidelity comparisons. Static sound/voice validation and silent native rendering do not establish an audible side-by-side comparison.
