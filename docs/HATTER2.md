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

The native component milestone brought estimated total completion to 49.3%.
The network evidence below now verifies a fresh full route, but the remaining
recovery/reconnect matrix keeps this campaign row partial and adds no percentage.

## Independent-client route and recovery

`tools/test_coop_hatter2.py` drives two real processes with isolated profiles using
ordinary movement, funded weapons and normal saves. Internet hosting is the
default, with 40 ms added delay and 20 ms jitter. No position, health, inventory
or progression writes are used. Screenshots were not inspected.

The fresh V5 run on native source `3298768` exits successfully after the complete
entrance-to-exit route. Both players cross the bridge, fight the 2,600-health
Hatter through his original three malfunctions, collect their own blade/Watch,
ride the lift and jump to the upper landing. Red owns the watched five-line
Gryphon scene; Blue stays hidden and frozen. Both reach `jlair1`, finish its
original entry scene and reunite with clear, grounded, separated bodies.

The closed evidence has **10 personal waypoint receipts and 6,805 matching
player/world/native states** (6,797 before process shutdown). Blue dies at tick
23,061 and revives at 24,261: exactly 1,200 active ticks, 25 health, preserved
personal resources, safe placement and the correct spectator/returning camera.
The complete host trace has 1,580 records. Hatter victory health is 50/33. After
the next level's original entry scene it is 100/33: that controller currently
grants its arrival refill to Blue, so this does not claim paired `jlair1` support.

A separate resumed victory run verifies Blue's watched Gryphon ownership and
actual shared arrival, with 2,169 matching states. An earlier fresh prefix
through the Watch landing has 6,209 matching states and two exact Blue revivals.
Those separate runs retain their narrower scopes; they are not fresh full runs.

Both clients submit environmental particles and effect batches in the fresh
run: maximum live puffs 1,198/1,279, submitted batches 3,280/3,505, zero dropped
batches. This is renderer/state delivery evidence, not a pixel comparison.

Evidence is retained under `private/coop/`:

- Fresh V5: `hatter2-route-d0cef301eb6e41309869328fda67da5e/`, successful
  `result.json` and independently replayed `verified-full-route.json`.
  Observations SHA-256:
  `0aa46d0af818c5b38241829e4ea78cde11cfa986c241a3d8fd3ddf7fd1ca14aa`.
- Blue ending: `hatter2-route-95033e47a6b5459ab133896c6ef6591e/verified-ending.json`.
- Earlier fresh prefix: `hatter2-route-a396b442fd9049aa954d5b1bf4618d9b/verified-prefix.json`.
  Its ordinary victory checkpoint is tick 30,675, SHA-256
  `cc79f82879fba7f88c424b854b49d0562d68b478fa759c399fc70e46885b2c86`.

Retained failures distinguish test defects from gameplay: the first strict run
stopped on a real death; recovery mode now proves the full timer and resources.
Later runs exposed Python's comparison against the native f32 door cap, a missing
forward movement after the exit approach, and the scene verifier rejecting the
committed `Done` loading state. All are corrected without changing the game.
The full-history replay also scopes combat to its original visit, so a revival
stamp carried into the next epoch is not misclassified as a new death.

Nine Python verifier guards and seven source-package checks pass. The exact
frozen V5 verifier and the current verifier both replay the retained milestones.
The strengthened ending audit measures the physical tail to the first destination
frame, excluding the next entry scene: 1,305/1,306 ticks, above the original
1,302-tick minimum. The next level's dialogue cannot mask an early departure.
Native source is unchanged from the 905-test/release/83-component milestone;
those builds were not repeated for these Python-only changes.

For another full run, supply the executable's printed native build identity:

```powershell
python tools/test_coop_hatter2.py --exe <exe> --build <native-build-hash> --data <base-folder> --first red --allow-revival
python -m unittest discover -s tools -p test_coop_hatter2_guards.py
```

`--stop-after combat` records only the entrance/fight prefix. `--checkpoint` accepts
an unchanged normal victory checkpoint alongside its original observations,
history and host recovery trace; its SHA, resume receipt and prior revivals must
verify. Resumed runs cannot claim fresh completion. `--local` is an explicit
local-network alternative. Team wipes fail this runner pending dedicated proof.

### Targeted recovery and guest restart

`tools/test_coop_hatter2_recovery.py` adds two bounded internet cases on the same
unchanged native source `3298768`, with 40 ms delay and 20 ms jitter:

- Fresh entry and ordinary combat damage kill Red at tick 6,171. Blue survives;
  Red revives at 7,371 with 25 health after exactly 1,200 active ticks. The closed
  1,253-row trace and both views prove preserved resources, safe placement and
  spectator/returning cameras. There are 1,087 matching player/world/native
  states. Final health is 80/25. No full-route claim is made for this case.
- An unchanged normal victory checkpoint resumes with its original provenance.
  The guest exits and a different process rejoins: personal resources, pickup
  ownership, defeated encounter and completed scenes are preserved. There are
  77 matching player/world/native states, including 58 after the new guest
  rejoins. Both players retain 50/33 health. This covers a graceful restart at
  idle victory; combat, moving machinery and transition reconnects remain open.

The first restart attempt exposed an overly strict test precondition requiring
no timed loot. The corrected verifier predicts the normal native float clocks,
cooldowns, drop aging and expiry, then compares the entire state exactly. It
rejects reset progress, changed pickup masks and incorrect drop removal. No
gameplay change was needed. Thirteen About Face verifier guards and seven
source-package checks pass; retained closed evidence replays successfully.

The successful cases retain `result.json`, frozen fixture dependencies and
`verified-recovery.json` under `private/coop/`:

- Red revival: `hatter2-recovery-df3e49a78b3c48ddb03f951f3573a01b/`.
  Observations SHA-256:
  `5ccdab0ee4bb5773c7ffe990d0608ec1a540337107157e2d971dbbc99743b91a`.
- Guest restart: `hatter2-recovery-aa1a6a92859c4f37bf3ae1a1d18c61de/`.
  Observations SHA-256:
  `0d86516bce0a2ac8af230a2571f648a103a5be4c5ddb9cf3fefbd341cfd48b65`.

```powershell
python tools/test_coop_hatter2_recovery.py --exe <exe> --build <native-build-hash> --data <base-folder> --case revival --victim red
python tools/test_coop_hatter2_recovery.py --exe <exe> --build <native-build-hash> --data <base-folder> --case rejoin --checkpoint <normal-victory-checkpoint>
python -m unittest discover -s tools -p 'test_coop_hatter2*guards.py'
```

### Ordinary combat team wipe

`tools/test_coop_hatter2_wipe.py` passes a fresh two-client internet run on source
`116fc99`, with 40 ms delay and 20 ms jitter. Both enter the arena normally and
save at tick 3,421. They take ordinary damage without attacking; Red dies first,
then Blue dies with 339 ticks left on Red's countdown. At tick 7,060 both return
to their exact checkpoint positions and personal resources, at 100/100 health.
The closed 917-row recovery trace verifies one common spawn boundary, valid
supported placement and no intervening individual revival. Both client views
return to their own cameras. There are 1,058 matching player/world/native
states, including 27 after restoration.

The opt-in route driver now checks the entire saved character/world snapshot,
both bodies and inventories, visit and campaign progress immediately after an
ordinary wipe, before another simulation tick. It writes `route-wipe-<tick>.json`
to the isolated host profile. This receipt observes the normal restore and never
sets damage, loads a checkpoint or changes gameplay. The Python check binds the
receipt to the actual checkpoint bytes and the closed trace's spawn tick.

Evidence: `private/coop/hatter2-wipe-bd9cd4af90f34b63a8238253bb5ac1de/`, including
successful `result.json` and replayed `verified-wipe.json`. Checkpoint SHA-256:
`ba3898cd05bebd87a80f35019736ffc7a9eb8e33947c16aa3f571f48e1d0a679`.
Observations SHA-256:
`b2ab954f8095d38353edd971d8380898d2bfa0471f19d659ff53de4b8607f851`.

The new native identity passes 905 active Rust tests (three ignored), the locked
release and the same 83 paired cases plus three summaries. Single-player 8+1
outputs match the unchanged baseline exactly. Sixteen About Face Python guards
and seven package checks pass. This targeted case does not claim a full route.

```powershell
python tools/test_coop_hatter2_wipe.py --exe <exe> --build <native-build-hash> --data <base-folder>
```

Remaining: disconnect/rejoin around combat, transports and transition, live
skipped/dead-partner ending cases, and broader campaign/contact regressions.
Six of 39 visits retain combined verification evidence; total remains 49.3%.

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
