# Mad Hatter — About Face

The `hatter2` controller owns the Mad Hatter battle and its onward route. It uses the user's local models, animations, effects, voices and camera paths. No original script or AI program executes.

## Co-op native adapter

Native source `3298768` implements shared combat/world clocks, personal rewards,
paired machinery and the original scenes. Each Alice receives the original
arrival refill once. Loading older co-op state preserves current health, and
preparation never resurrects a dead player or overwrites a low-health revival.

The blade, healing and Clockwork drops have personal consumption and view state.
Taking a reward does not consume or hide the partner's copy. Difficulty controls
each healing cooldown; an active cooldown can finish after victory, as in
single-player. Shared blade/Watch progress opens the encounter's existing gates.

The closing bridge waits until all living bodies clear its edge. The lower lift
waits for boarding; departures require full footprints on the deck, with an
exception for a partner already on the nearby supported destination landing.
Both living riders move in one transaction, and an obstruction rolls machinery
and riders back together. Empty upper decks return to the lower boarding stop.

Both Cat conversations keep players free to explore. The first Gryphon entrant
owns the original single-Alice scene. Its initial pose is bound on the trigger
frame, before rendering, and the partner stays frozen through the watched or
owner-skipped scene. Both paths request the original shared `jlair1` departure.
A scene exit request alone does not verify the next map's two-client arrival.

On 5 November 2026 the native checks pass **83 staged cases plus three summaries**:

- `--hatter2-coop-combat-check`: 29 cases for shared clocks, nearest visible
  living targets, personal melee/projectiles/tea, saved syringe steering, Watch
  behavior, reinforcement targets and the original boss breakdown.
- `--hatter2-coop-check`: 38 cases for clock parity, personal arrival/resources,
  legacy save migration, death/low-health handling, cooldown and drop persistence,
  shared reward gates, both lifts, boarding, obstruction rollback and catch-up.
- `--hatter2-coop-scene-check`: 16 cases using actual Stage contacts with staged
  prerequisites: both Gryphon owners, watched/skipped scenes, living/dead waiting
  partners, trigger-frame actor poses, pause/compressed restore and exit requests,
  plus both ordinary Cat conversations with free exploration.

All **905 active Rust tests** pass (three ignored), as does the locked release.
The existing `--hatter2-check` and `--hatter2-route-check` logs match the unchanged
`84c8eee` baseline exactly, including the full single-player input route. Seven
package checks and the source worktree audit also pass. No images were inspected.
Evidence: `private/hatter2-coop-3298768/manifest.json`; world/reward log SHA-256
`25381f26b7cc72c03480f7e2a17dc3c0fb68a6e1b909484c5ea1bdafd2d9250c`.

Earlier failed runs are retained: the migration/landing checks were expanded,
and the empty-return fixture originally lacked verified player footing. The final
run asserts clear supported entrance positions, full deck footprints and actual
landing support; these requirements were not relaxed to pass the tests.

Full two-client traversal, the actual shared onward arrival, rendered/network
delivery, deaths, checkpoint recovery and reconnects remain unverified. These
component checks add 0.25 points each to combat/resources, movement and progression
under the existing weighting, bringing total estimated co-op completion to 49.3%.
They do not complete another campaign visit.

## Single-player implementation

- The arrival bridge is solid immediately and closes after Alice crosses it. Arrival restores resources once and disables the Cheshire summon and Demon Dice allies.
- The centre blade is a distinct quest component, recorded in persistent pickup history. It does not grant the completed Eye Staff.
- Cheshire's first appearance starts the four-stage, 96-second clock cycle. The minute and hour hands move, gongs mark the stages, and the Hatter shrinks, changes between tower and arena, then grows again.
- The boss has 2,600 health, locomotion, damage reactions, directional cane/slap attacks, ballistic cups and homing syringes emitted from the animated attachment tags. Melee contact and projectile release use the reviewed source animation frames. Winding up a melee strike commits his facing so Alice can dodge.
- East and west Clockworks emerge through the timed doors and launch into the arena. They share the existing Clockwork combat controller. The tea phase clears surviving reinforcements; their death rewards expire and cannot be collected twice. At most two reinforcements, four drops and 24 Hatter projectiles exist.
- The final 100 health starts the breakdown: start animation, three malfunction clips, then the final collapse. This works on the tower as well as in the arena. The supplied animation events control the cane, hat, surfaces and attached sparks.
- Victory stops the arena clock and starts the alternating reward platforms. Rider movement and platform collision use the same transforms; blocked machinery rolls back rather than embedding Alice.
- Collecting both the blade and the Pocket Watch enables Cheshire's second appearance, the exit doors, the teleport and the Gryphon escape. The scene includes the opening cage, dialogue, camera paths, Alice's riding attachment and the breakaway ceiling. Watching or skipping commits the same transactional `jlair1$jlair1_start1` departure.
- Encounter state, phase timers, projectiles, reinforcement deaths, rewards, moving supports, dialogue progress and exit delivery are saved. Older controller-less visits restart at the entrance while retaining carried resources.

## Deliberate approximations

Syringe steering uses a bounded 90 degrees/second interpretation of the supplied seeker setting; its native conversion has not been recovered. Cups use the supplied speed and gravity with a simple ballistic aim correction. Cups and syringes apply direct contact damage; their secondary explosion-effect radial damage is not reproduced. The tea impact uses a reviewed 300-unit radius; its native default radius is not supplied in the effect definition. The malformed timer model supplies timing only, not a mesh. The moving discs use 112-by-112 support boxes around their walkable tops; the tall decorative rods do not form solid walls. The patch-only pendulum is animated but has no additional solid collision. Ceiling debris follows bounded scripted motions and is cosmetic. Camera and path playback use the engine's existing independent spline implementation. These are playable compatibility behaviors, not a claim of frame-perfect original parity.

## Verification commands

The clock floor retains its authored opaque lightmap base before its detail,
face and pulsing glow layers. Removing that base caused black triangular seams
and overlapping numerals. Both scripted Cheshire placements now receive the
normal actor floor placement (arena cover z=0, Watch perch z=396) and use the lit,
mouth-animated dissolve renderer. This is derived when loading the map, including
existing saves; no save-format migration is needed.

`--hatter2-check` checks clock timing at 30/60/144 Hz, reinforcement limits, pause, state restoration, death while on the tower, reward gates and rejected saves. `--hatter2-route-check` uses production movement and weapon inputs through the battle and departure. Set `LOOKING_GLASS_WATCH_SCENES=1` to watch instead of skipping the dialogue scenes during this route.

`--hatter2-render-check` creates native staged captures under `private/hatter2/captures`. With `LOOKING_GLASS_HATTER_SAVE=write` or `read`, it instead exercises native Store snapshots in separate processes. These staged checks are distinct from the input-driven route. Run native checks on the hidden test desktop with `--no-audio` when that desktop has no audio device.

The headless check also verifies both Cheshire support heights. Native captures
include the complete clock floor, its close Cheshire conversation and the Watch
Cat on the ledge.

This work covers the About Face boss encounter. It does not claim completion of every puzzle and cinematic in the preceding `hatter1` map.
