# Mirror Image (`funhouse`)

The October 2026 performance update and original mirror behavior are described in [MIRRORS.md](MIRRORS.md).

Visit 22; one controller owns machinery, clock progression, Tweedles and scene completion. Save envelope remains 12. Controller payload version 1; hit IDs use the existing 8,200,000 reservation. Source: `src/levels/funhouse.rs` and its companion modules.

## Implemented behavior

- Intro commits cell 2 once and withdraws the entrance stand after the handoff.
- Eight cell bits, shootable mirror/cell clocks, closet glass and existing encounter activation, rotating tube, gear and gas-door sequence.
- All 20 rotating doors admit the player: 4096 means automatic closing, 64 player proximity, and 16 excludes Actors. The older locked-cell assumption in v23 was incorrect. Native evidence: private research `phase2/TRIGGERS_AND_DOORS.md`.
- Weighted fulcrums, difficulty-specific pit handling, suction/teeth cycles and three pendulums. Movable geometry carries riders with upright body clearance.
- Arena floors are present and solid before entry. Both named Tweedles retain distinct identities, health, animations, attacks and miniature caps. Only the two named deaths count toward the five-second ending gate.
- Hatter scene, saved dialogue/camera clocks, physical floor collapse and one retriable exit to `hatter1$hatter1_start1`. Skipping dialogue retains the physical ending tail and removes live miniatures.
- Existing map and model identifiers resolve against local installed data at runtime; no original dialogue is stored here.

## Verification and publication

The co-op playable audit on 7 November connects 20 ordinary save/resume
sections from the entrance through 156 recorded course actions, the actual
Tweedles defeat, original ending and both players' Hatter1 arrival. Rechecking
the captured movement/combat/scene assertions yields 18,637 matching state
samples and two verified ordinary revivals. Checkpoint hashes and native
exact-resume receipts connect every section. This credits historical
build-specific playability, not a fresh uninterrupted latest-build run;
the older single-player candidate below retains its original scope.

Playtest candidate **6C9850B5**, based on Labyrinthine Revenge **D654E222** and retaining its earlier releases. Root `Launch.cmd` selects this candidate; `tools/launchers/Launch-Mirror-Image.cmd` opens a fresh visit with separate saves. This is a restored-systems playtest, **not a completed full-route certification**.

- Headless contracts pass: idempotent clock mask, real arena-floor traces, pause, machinery restoration, knife-frame damage, miniature caps, defeat/exit transactions, both door flags/directions, Easy rescue versus Normal fatal-fall volumes, weighted fulcrum carry/jump, a complete pendulum period and native radial-damage falloff.
- Nine native Store cases pass in independent writer/reader processes. Each compares the same 20-second owner/world/player future. The legacy case uses an actual pre-controller interaction snapshot with a consumed arena trigger; migration rearms it and restarts at the entrance without changing resources or inventory.
- Thirteen staged native renders pass and were inspected. The map references `textures/notexture`; it is the sole unresolved material identifier. Arrival pause, quicksave and held-Enter skipping were exercised using real keyboard input.
- 632 unit tests and root compilation pass. All 38 other visit fingerprints and event signatures match D654E222. Funhouse's new controller changes its own fingerprint and event signature; existing rule identifiers, visit order, save envelope and hit ranges remain unchanged. The 39-visit exit graph passes.
- The Normal native checkpoint lineage reaches all clocks, crosses the rotating tube and weighted fulcrums, and reaches the pendulum bank with live enemies, ordinary weapons, real loot, no invulnerability and no recovery teleports. The full combat course remains open: Boojum knockback defeats the route driver at the pendulums. Neither the movement-only probe nor staged arena fixtures are full-route proof.
- A separately labelled staged arena fixture restores the native placed cast and fights both bosses with ordinary attacks. The watched ending enters Hatter1 alive with exactly carried 100 Sanity/100 Will. A fresh process resumes the earned 4.958-second defeat-wait checkpoint, skips the Hatter scene and reaches the same required outcome. The staged arena placement is not evidence that the incoming route passed.
- Owner-only arena diagnostics start with 30 Sanity, collect both authored essence pads by jumping from their ramps, kill both named bosses and complete the watched/skipped Hatter ending at 100 Sanity. This does not cover the surrounding placed NPC cast or the incoming course.

Use `--funhouse-check`, `--funhouse-route-check`, `--funhouse-skip-route-check`, and `--funhouse-render-check`. The route checks are currently diagnostic; a complete pass is still required. `--funhouse-traversal-probe` disables the placed NPC cast and deliberately reports failure at its end. Private inputs, logs, source manifest and publication metadata are under `private/funhouse-work/`.

## Fidelity limits

AI selector probabilities and fulcrum return/damping frequency are bounded interpretations where native details remain unresolved. Detailed head tracking, death sinking and some scene gestures need further comparison. Child waypoint walks, the split player-clip, propeller loops and some exact actor navigation remain fidelity follow-ups. Audio identifiers and cue timing are implemented; listening has not been verified on the isolated test desktop.

## Entrance presentation correction (2026-10-01)

The six `begin_lamps` fixtures now receive the original World_Init roll of
270 degrees and render alongside the existing BSP flame surfaces. Previously
the unsupported placement command deferred their geometry, leaving floating fire.
The hanging lamps and chandelier retain their original models and materials.

The unnamed passable screen behind the starting stand now blocks camera sweeps
only. Player and weapon collision retain the original non-solid flag. Both the
follow camera and its cutscene return use this extra occlusion; it is rebuilt
by the normal interaction synchronization after entry or restore.

The headless fixture checks all six lamps, production synchronization, and
watched/skipped/restored arrival returns at 30, 60 and 144 Hz. Native entrance
captures cover 36 return frames and two views of the lamps. These checks run
as part of `--funhouse-check` and `--funhouse-render-check`.

## Co-op battle components (28 October 2026)

Source `ffaea18a57d1d15ac129ce72630ad9973c0e2f74ff5783c75fb127d3b9e9f486`
passes 63 staged native cases through `--funhouse-coop-check`: both Tweedle
families and miniatures choose the nearest visible living player; knife hits,
grenade contact/splash and ground pounds damage the actual personal bodies.
Healing locations and cooldowns are individual, including joining during an old
save's cooldown. Saved attacks continue exactly, invalid personal states are
rejected, and victory selects the living actor after damage. Both original
victory completions remain saveable and hide healing after the ending.

All 855 active unit tests pass (one pre-existing ignored test); the locked release
build succeeds. The eight single-player component records match the preceding
baseline, excluding only two measured rebuild-performance timings. The original
single-player full-route baseline already fails at step 142; no new full-route
pass is claimed. At that battle-only checkpoint, shared machinery and the broader scene,
traversal, internet, recovery and rendered-effects checks remained open. This component check does not complete a campaign visit.

## Co-op machinery follow-up (28 October 2026)

Source `74a6a2519a35e800e99756ca69976522d0adf98bf9c700c72d6df34df745dc28`
passes 35 native cases through `--funhouse-coop-world-check`, plus the preceding
63 battle cases and all 855 active units. One shared motion update now handles
all living players. Passive fulcrums use their average loading position. Both
riders complete a full pendulum period and the tube's complete turn, remaining
clear and separate. Tube departure waits at its authored trigger for all living
players; saved contact order keeps the first living requester. Doors react to
either player and hold their own leaf if the other blocks it, without stopping
unrelated clocks. Any blocked platform carry rolls back the shared step and both
bodies before either is committed. Both dead-player slots match the one-player
machinery result exactly; a dead team leaves the world frozen.

The automatic gas scene binds the living player nearest the gas machinery in
the contact phase. Ordinary Jack speech keeps exploration running, and Stage
advances bound cinematics once. Full watched/skipped scene and safe-reunion
combinations still need verification. The entrance stand keeps its authored
withdrawal/drop behavior; it is not treated as a passenger lift.

The first machinery run passed 32 records but failed because the tube fixture
placed its riders inside the floor. The corrected fixture uses a native body
sweep and verifies actual tube support before testing movement. Both attempts
are retained privately. The existing single-player component gameplay log is
identical, excluding two measured benchmark timings. An Anode replay also
matches every single-player route line through its pre-existing step-142 death,
including the same final coordinates. The route remains failed; no full co-op
route, independent internet, recovery or rendered-effects pass is claimed.
The broader 94 Checkmate, 82 shared-world and 43 campaign checks also pass with
unchanged Checkmate single-player logs. No images were opened.

## Co-op scene follow-up (28 October 2026)

Source `e1b17cb10c40c705ff419449fb123fa8b15fe3d729ca6fea10f9436a1db61777` passes 37 records through
`--funhouse-coop-scene-check`: 32 cinematic combinations (Arrival, Gas, Tweedles
and Hatter, both actors, watched/skipped, living/dead partners) and four real
personal Jacks pickup cases. Cinematics keep the waiting partner frozen; held
partner input cannot skip them. Compressed saves preserve scene state, camera
eye/target/up, field of view and world poses. Invalid/paused time leaves both
bodies and the scene unchanged. A dead partner stays dead. Hatter skipping keeps
its 6.5-second floor/exit tail and requests the original next visit.
Co-op retains its actor after committing the exit until the normal scene update
delivers the request, so a dead partner cannot cause premature control release.

The arrival stand fits one Alice. Its completed scene now opts into a bounded,
swept reunion search on the adjacent lower corridor, with full-footprint support
and hazard rejection. The actor retains the authored endpoint and the stand's
normal withdrawal. Jacks collection now starts ordinary shared speech in the
co-op contact phase using the actual pickup identity from local data; it does
not freeze exploration. The partner retains an individual pickup and cannot
replay completed speech.

These are staged native components. Fresh simultaneous arrival normally ties
to Blue; Red's arrival case explicitly restores an alternate owner. Tweedles
uses the real trigger; Gas and Hatter use staged puzzle/post-battle conditions
and the production owner-selection phase. Hatter uses native damage to create
valid dead enemies. No full route, rendered visibility/effects, internet or
recovery pass is claimed. The previous single-player route still fails at step
142; that historical trace is not a new route pass.

The first scene attempt stalled at the opening reunion and was terminated; the
next passed 28 cases but rejected eight invalid Hatter fixtures whose health had
been zeroed without the normal death transition. The third passed 32 cases but
exposed four dead-partner Hatter cases that released control before delivering
the committed exit. The fourth confirmed that an ordinary free-world tick does
not deliver it. Co-op now keeps scene control through the automatic request.
All attempts remain private.
The corrected run passes all 36 cases plus its summary record, together with 63
battle and 35 machinery records. Single-player component gameplay is identical
to baseline, excluding only two benchmark timings. The 855 active unit tests
pass with the final production changes (one existing ignored test).
The 94 Checkmate, 82 shared-world and 43 campaign checks pass on this build;
Checkmate single-player component and full-route logs remain identical.
No screenshots were opened.

## Independent-client opening traversal (28 October 2026)

Source `b897af27cf0b7ade07ed4f6a28626e400efad84b519faf337c7dbeadebb6ce29`
passes the fresh watched entrance through authored steps **0-28** over the public
internet, using two processes and separate profiles with 40 ms delay and 20 ms
jitter. This is the opening corridor, before the mirror switch and machinery.
The original Blue-owned introduction is observed in both clients (179 / 178
samples), hides the partner and rejoins the players safely afterward. Strict
body separation, personal cameras, visible partners and no-respawn checks pass.

Each Alice personally reaches all 22 movement waypoints. The 44 arrival receipts
match immutable client observations, including actual supported feet. The mover
walks to a nearby collision-checked waiting point before the other player enters
the same waypoint; there are no body, health or progression writes. Return legs
let the player already ahead move first, recording the actual order. Six
ordinary combat-clear steps and the initial wait also pass. Blue collects a
normal health drop; the pair finishes at **86 / 75 health**, with no deaths.
There are **2,734** matching sent/restored player, world and native-resident states.

Evidence is `private/coop/funhouse-route-2e4c32d4c9754287910d29f369b32dea/`:
`result.json`, `history.json`, fixture/dependency hashes, per-step reports and
unchanged normal checkpoints before steps 7, 28 and 29. The last checkpoint has
SHA-256 `18ce8375a8989c5db04164d0e2e95fb4e70757e347900cd85cefe54e19a373ae`.
The numeric test ran in Anode in 255.592 seconds; all owned windows were closed
and the desktop lease released. No screenshots were opened.

The reusable command is:

```powershell
python tools/test_coop_funhouse.py --data "C:\path\to\Alice\base" --first blue --last-step 28 --save-before 7 28
```

`--checkpoint` together with `--start-step` requires an unchanged normal co-op
checkpoint and verifies the exact resume receipt. That path is available for
future segment tests; the run above starts at the entrance and does not claim a
resume pass. The runner has since been extended through step 88, but supported
commands do not establish successful traversal. Full paired traversal, the cell
switches, tube/gear/pendulum routes, later scenes,
recovery/rejoin, rendered environment effects and the exit remain unverified.
The existing single-player full-route failure at step 142 remains separate.

The first attempt on `afe78ef`, folder
`funhouse-route-0604eff526164b3c8028b3cf49aeaa46`, reached steps 0-6 but failed its
checkpoint request because the diagnostic command retained attack input. It
remains a failed run. Neutral commands now clear that input. The same attempt
exposed delayed stopping oscillation: the new regression fails on the old
controller while still reversing at speed after 2,400 simulated ticks. Scaling
the diagnostic steering correction with input delay fixes both walking and
running across four tested delay queues; zero-delay steering is unchanged.
This does not alter human movement, gameplay physics or saves.

All **858 active Rust tests** pass (one existing ignored test), the locked release
build succeeds, and current native checks repeat **63 battle / 35 machinery /
37 scene / 8 single-player component** records. The single-player component log
matches baseline apart from the two benchmark timings. Four Python guards reject
missing/duplicate personal arrivals, unobserved or invalid bodies, and the old
non-neutral save request. The overall estimate remains **48.1%**; no completed
campaign row is added for this opening segment.

## Independent-client mirror sequence (29 October 2026)

Source `b703a7ac1ee5cd7481da6efc6e7806b1cc4b0b074b4c04ba3ba2e1ceb4a7c6fd`
verifies authored steps **29-32** from the unchanged normal checkpoint above.
Both independent public-internet clients use 40 ms delay and 20 ms jitter. The
small lower perch requires the players to go one at a time. Both make actual
downward and return jumps, and Blue throws the owned Blade at the active mirror
target. Both clients observe the switch changing; both players perform the
authored 120-tick directional hold (157 and 167 observed ticks respectively).
All four jump arrivals match supported bodies and increased jump counters.
There are **615** matching player/world/native states in this scoped segment.

The evidence is `private/coop-funhouse-mirror-prefix-b703a7a.json`, extracted
through the normal checkpoint at tick 31989 from the immutable history in
`private/coop/funhouse-route-21d88baebe1447e0bba12a81d640772c/`.
Checkpoint-before-033 SHA-256 is
`6060c01f178dcbae5ee01c6f14a6aa0511ef76ad27f93f3d71d641028de25a64`.
The original requested 29-42 run **remains failed at step 35**. Enemy hits pushed
the waiting Red Alice into Blue's destination. Neither player died. The next
test resumes from the normal post-mirror checkpoint and makes the waiting player
defend and return to her safe waiting point using ordinary movement controls.

Earlier attempts exposed test-controller assumptions: the perch had no adjacent
waiting space, a previous-tick reached flag did not prove current arrival, an
authored directional hold was mistaken for a destination, and the generic
eight-unit arrival radius rejected a valid jump landing. Jump verification now
uses the original route's grounded 20-unit horizontal / 52-unit vertical bounds
and still requires a newly observed jump. Walks retain the stricter eight-unit
bound. No health, position or progression is forced; failed runs are retained.

New diagnostic commands emit ordinary Blade aiming/throwing and timed steering.
Switch completion requires observing the same active target before it disappears;
missing or changed targets fail. Explicitly avoiding puzzle switches in generic
combat is opt-in, preserving existing callers. Holds do not claim arrival.
The locked build passes **861 active Rust tests** (one existing ignored test),
63 battle / 35 machinery / 37 scene / 8 single-player component records, and
unchanged single-player component gameplay apart from two benchmark timings.
Ten Python guards cover personal arrivals, actual jumps, switch transitions,
uninterrupted timed holds and waiting clearance including stopping tolerance. No screenshots were opened. This segment earns no
completed campaign row; the overall estimate remains **48.1%**.

The first post-mirror replay (`funhouse-route-0884aa9ed4374dd5b7a81e1ca1866af0`)
failed at step 34 because a waiting target 41.6 units from the destination let
the body stop only 34.4 units away. Waiting-point selection now reserves both
body clearance and the eight-unit stopping tolerance; the captured geometry
has a regression guard. The second replay
(`funhouse-route-d1422e6ff9254213b771cb8cf7107481`) passes steps 33-35, but Red
dies during step 36. It remains failed. These attempts do not establish a
complete return-corridor pass or earn further progress credit.

## Natural combat revival and retained route failure (29 October 2026)

The independent-client replay on the same frozen `b703a7a` build now exercises
the game's normal revival in Funhouse. Starting from an unchanged checkpoint
before step 35, both players personally complete steps 35 and 36. Red dies at
host tick **35382** and respawns beside Blue at **36582**: exactly **1,200** active
ticks, **25 health**, no cinematic pause or placement delay. The exact native
trace verifies preserved personal resources and safe, supported, separate spawn
placement. The recorded spectator and returning-player views pass; the scoped
presence check includes 160 host and 152 guest observations.

The replay then fails at step 37 after both players die. At tick **37831**, the
host restores both players simultaneously with their exact saved positions,
health (83/72), inventory and scene stamps. This is exact host-boundary evidence,
not verification of both clients' subsequent play or all restored world state.
The original run remains failed; no full-corridor or completed-visit credit is
claimed. The evidence and verifier hashes are retained in
`private/coop-funhouse-recovery-prefix-b703a7a.json`, derived from
`private/coop/funhouse-route-fd5b2cbdcad64d1b8d02a5d991f4666c/`.
The unchanged input checkpoint SHA-256 is
`ab47a181651b5026a07b1aa0695b3be6fc39e0b1fa72f29e39eb1b37452b0399`.

The damaging enemies are Clockwork Automatons. The captured state contains an
active punch attacker and a homing fist targeting Red. Earlier clear checks
covered guards, Boojums, Snarks and native level actors but omitted these NPCs.
The fixture now checks awake nearby Automatons and remaining fists, including
fists still live after their owner's death. Long walking legs let the partner
follow actual recorded, supported footings with a gap; every authored waypoint
still needs each player's separate arrival. Parking retains steering through
the handoff, removing the neutral interval in which punches pushed a waiting
body into the destination. These are test-controller changes only.

`--allow-revival` opts into exact host tracing and the shared recovery verifier;
`--require-revival` additionally requires a real death and completed revival.
The default remains strict. During death the fixture suspends input, then
resumes the requested controls after the game's own spawn. It never writes
bodies, health, timers or progress, and it does not omit missed waypoints.
An incomplete countdown, incorrect spawn, missing rendered evidence or team
wipe fails this route mode. Fourteen Funhouse guards, thirteen shared recovery
guards and seven source-package tests pass. No Rust source changed in this
follow-up, so the frozen build's 861-unit and native-check results are unchanged.

The earlier continued-following run
`funhouse-route-fa2369815bbc4f4b97e3cdd29cc1bd10` still failed at step 36 with
Red dead. The first recovery-enabled attempt
`funhouse-route-3c591bd67e694b0994b1d4bcaab9ff33` failed at step 35 during the
neutral parking interval. Neither is reclassified. Overall stays **48.1%**.


## Ordinary-input combat automation (29 October 2026)

Source `c946266` adds an opt-in `combat_tactics` command for the local diagnostic
controller. It reuses the route combat planner: actual Blade melee volume takes
priority over Cards, and observed target velocity leads thrown attacks. A
repeated received frame does not alter that velocity estimate. Waypoint changes
retain it; actual relocation or a new epoch resets it. Living partners remain
occluders and cannot become attack targets. Old fixtures keep their default
controls, and human gameplay is unchanged.

When waiting without a movement goal, the controller predicts normal movement
on a cloned player before choosing a short defensive strafe or close approach.
It rejects walls, unsupported drops, hazards and approaches into the partner.
When Cards are unfunded and an attacker is nearby, it retains the Blade for
ordinary melee rather than starting the long throw recovery. The real weapon
hit test still decides whether a swing reaches the enemy. A waiting survivor
may temporarily leave the parking goal during the partner's countdown, then
resume it after the game's own revival. No body, resource or progression state
is written by these tactics.

All **865 active Rust tests** pass (one ignored), including actual moving-target
projectile contact, melee occlusion, safe predicted movement and retaining the
Blade for an approaching Automaton. The locked release build passes. Native
Funhouse checks repeat **63 battle / 35 world / 37 scene** records, and
Checkmate repeats **94**. All eight single-player component records match the
baseline gameplay log, excluding only its two benchmark durations. The Python
suites pass **15 Funhouse / 13 shared recovery / 7 source-package** tests.

The unchanged Normal checkpoint before step 35 was replayed with independent
public-internet clients, 40 ms delay and 20 ms jitter. Both personally reached
steps 35 and 36, but the run still failed at step **37** after both died. It is
retained as a failure in
`private/coop/funhouse-route-3cbc004991fd47c981ee1fd7bdfb81b9/`.
The preceding `e96a9d3` tactic build also failed at 37; neither is a corridor
pass. The frozen final executable SHA-256 is
`d5b4422a06cf2808eb9a442be085c3346c143b4ccf7adefd19f0805458374b39`.
Further combat-route coordination is needed before the machinery segment can
be reached and verified from this checkpoint. Overall remains **48.1%**.


## Return corridor and personal essence (29 October 2026)

The same frozen `c946266` executable now passes **steps 35-42** from the unchanged
Normal checkpoint in two independent public-internet clients, with 40 ms delay
and 20 ms jitter. All **16 personal arrivals**, supported following footings and
**1,175 matching player/world/native states** pass. Red dies naturally at tick
**34887** and revives at **36087**: exactly **1,200 active ticks**, **25 health**,
preserved resources and safe placement. Recorded spectator/return views pass.
The final health is **Blue 100 / Red 26**.

The missing step was collecting essence. The failed replay retained a large
Automaton drop for each player near the end of step 35, yet the runner left
Blue at three health and continued to the next fight. It now walks each player
over nearby live drops from that player's own received inventory. The successful
run verifies three deliberate pickup transitions (Blue's and Red's first
Automaton essence, plus Red's second); incidental collection still uses the
normal game rules. The checker rejects expired or distant candidates, a
partner's collection credit and disappearance without a personal pickup event.
No resources, bodies or progression are assigned by the fixture.

Evidence: `private/coop/funhouse-route-19d78fb6fb2442b7beff2da6dc4a1908/`.
The resulting normal checkpoint before step 43 has SHA-256
`bdf435a23f89705e21fb1467ee8d968197b008092ef57d13b173af07bb117ddc`.
The original failed runs remain failed; this is a new successful resumed segment.

The first machinery attempt from that checkpoint verifies ten personal arrivals
and three paired timed holds across **43-50**, then fails to hit **cell4 at 51**.
Its retained evidence is `funhouse-route-f885234d3ac4461d859a62d0e8b748e7`.
The scoped receipt checks pass independently, but the complete machinery run
remains failed. Its normal checkpoint before 46 is retained. Later cell switches,
shared transports, pendulums, scenes, combat and the exit remain open.

The runner now also accepts the original Cards preference toggles through step
117; that extends its supported command vocabulary, not verified coverage.
Twenty-one Funhouse guards, thirteen shared recovery guards and seven package tests
pass. The frozen executable and 865 active Rust/native-check results are unchanged.
These segment results do not complete a campaign row: overall stays **48.1%**.


## Cell-section traversal (29 October 2026)

A subsequent independent-client internet run passes **46-88**, starting from
the unchanged normal checkpoint before 46. It verifies **50 personal movement
arrivals**, **11 paired timed holds**, **seven requested cell-switch shots** and
**3,633 matching player/world/native states** with 40 ms delay and 20 ms jitter.
There are no deaths; final health is **100/90**. Each switch was observed active
before its command, acknowledged after the actual Blade shot, and received by
both clients. This covers the requested cell4/3/1/6/5/7/8 sequence in this segment,
not a new fresh-start proof of every cell.

The fixture can retry a missed switch shot from a native collision/support/
hazard-checked neighboring footing. It waits eight active simulation seconds
before trying another position, remains within 96 units of the original footing,
keeps at least 48 units from the partner, and uses ordinary walking. A receipt
must match the observed native candidate and actual grounded arrival. Cell3
needed two such approaches in this replay. Cell4 completed without repositioning
in the new run; the previous cell4 timeout remains recorded rather than being
reclassified as a pass. No weapon or collision rule was changed.

Evidence: `private/coop/funhouse-route-e389cb0861f04f65a74961ccc397d027/`.
The resulting normal checkpoint before 89 has SHA-256
`47b0a987b66128627550e7694bcd2ed79872dfd97cf8b0176f5d9aeb8ae7f851`.
Together with the earlier scoped 43-50 proof, these retained segments cover the
tested opening machinery, but do not establish the full route, later pendulums,
all recovery/rejoin paths or rendered environment effects. Overall remains
**48.1%**, six combined campaign rows and 33 open.


The next attempt, `funhouse-route-304c8d36ea9e438f9273bf8cdbf29e7e`, starts from
that normal checkpoint and fails at **95**. Scoped checks for **89-94** pass
six personal arrivals, two paired holds and three deliberate personal pickup
receipts. Both players remain at 100 health at the failure. Blue reached 95,
then parked on the tilting platform; Red could no longer satisfy the authored
waypoint height. Both end grounded on its 45-degree slope, about 137-141 units
below the nominal waypoint. The original run remains failed. The next route
work is to queue the fulcrum crossing and its exit jump while preserving both
personal receipts, rather than widening the arrival-height check. No full
89-117 pass, new campaign-row credit or percentage increase is claimed.


## Connected fulcrum investigation (29 October 2026)

The queued runner now lets one player traverse both connected fulcrums and reach
step 98 before the partner starts. Parking at 96 had left the first player on
the second fulcrum. Only the two named approaches, 95 and 97, use the native
single-player `steer()` bounds of 20 horizontal / 52 vertical units, with an
acknowledged ordinary walking command and an observed living, grounded body.
Other walks retain 8/35 bounds; jump 96 still requires a real jump and supported
landing. A 150 ms ordinary run-up precedes that jump. No positions, resources,
physics or progression are forced.

The attempts are retained individually; none is a full connected-course pass:

| Evidence folder suffix | Observed outcome |
| --- | --- |
| `cffa6eac2cdc4fda8254efddabacb39a` | Red passes through the native approach window but never satisfies the old tighter stop. A normal checkpoint before 95 is retained. |
| `de111de4fd674a1699d21d583327c953` | Both approach/jump receipts at 95-96 pass; parking on the second fulcrum precedes a failure at 97. |
| `970ca03f58c946729375cbdc57a8b07b` | Blue reaches 98, then is knocked off while waiting; Red fails at 96. |
| `fe0d0a6d97c74f0bbe0075963c9c1076` | Blue reaches 98 and collects personal essence; both first jumps pass, but the subsequent waiting/crossing sequence wipes at 97. |
| `99e2b76b4a0940c0a8e0f00bbbaf90e4` | Automation error: a new check requires `expected`, which ordinary walking leaves empty. Both remain alive at the timeout. The check is corrected to use the acknowledged command and actual body. |
| `739ff3ffec674c558fa8ff45ffe65246` | Four personal approach/jump receipts at 95-96 pass independent verification. Blue traverses 95-98 and remains alive; Red stalls below 97. |

All folders are under ignored `private/coop/funhouse-route-<suffix>/`. The latest
run uses unchanged executable/source `c946266` and the ordinary checkpoint
SHA-256 `89ae40cc26b6308ccfea800b3876a4a75d043b360b6078293b095ec685a3b14e`.
Its captured fixture hashes match the tested source. It takes 137.969 seconds,
returns failure, and leaves both players alive at 81/100 health. Red is grounded
near `[1920, -3400, -178.295]`, outside the required landing height. The current
arrival guard rejects this outcome. The queue currently checks saved mover
indices 31 and 61 for reset. A full scan of the captured state reveals another
loaded fulcrum, index 62, which reaches -45 degrees while Blue waits near
`[1967, -3563, -15]`. The reset gate therefore misses a relevant mover. The next
check must establish the bank's actual support and keep the waiting player off
all connected fulcrums before admitting the partner.

After collecting essence, the first player now walks back to the authored bank
before choosing a supported waiting neighbor and using ordinary defensive
controls. The latest run avoids the earlier waiting death; this single result
does not establish general combat safety. The full step-99 clear remains required
for both players. Scope verification covers 95-96 only, not the entire queue.

The 22 Funhouse receipt guards, 13 recovery guards and seven source-package tests
pass. No Rust source or game binary changed; the previously frozen 865 active
Rust tests and native component results remain the applicable evidence. No
screenshots were opened. Overall remains **48.1%**, six combined campaign rows
and 33 open, with the earlier contact repeats still required.


## Live support and resident threats (29 October 2026)

Geometry inspection corrects the earlier "far bank" description: step 98 is the
pivot of native fulcrum entity 460, saved mover index 62. The connected course
uses indices 31, 61 and 62. A waiting Alice who drifts away from that pivot loads
the last board and lowers the partner's approach. The test now chooses an
observed lateral footing and uses ordinary walking inputs to counter its tilt.
It changes no body positions, health, inventory, physics or level progression.

A first counterbalanced run on `c946266`, evidence folder suffix
`22650502fc9f4448bb73c58308d8358d`, verifies all eight personal arrivals in 95-98
and 400 matching player/world/native states before step 99. It subsequently
fails at 101 because fixed world height does not describe the tilted board.
Its old clear predicate did not include resident Boojums, so its printed clear
message is not sufficient combat proof.

Source `0d43ade17911a0182e2dbe1903885bc7ac813ba7f1bfe16863643b3f95585402`
adds local read-only surface diagnostics. A named live fulcrum projects a requested
horizontal walking goal onto its transformed collision brush. Arrival requires
an acknowledged ordinary walking command, a living grounded body, and an actual
support trace against that same brush. The recorded projected goal and body must
match the immutable client observation. A lower floor, an airborne body, another
board or an unknown entity cannot satisfy the check. This applies to the explicit
101/103/104 moving-board goals; ordinary corridor/jump checks keep their prior bounds.

The diagnostic also includes resident Boojums, with each player's actual native
line of sight and distance. The test previously examined generic encounter Boojums
and level-owned enemies but omitted NPC residents 50/51. A replay on `603f022`,
folder suffix `f721b7cde9ac461386938d3c72466ab6`, therefore left them attacking after
a premature clear; Blue died and revived before the waiting handoff. The new
handoff rejects that hidden revival. A distance-only correction in
`d2a5bf6a02b24b54af288f67b624ddae` kills 50/51/52 but waits for resident 65, outside
the current sight lines. The final predicate requires native visibility for live
resident threats within their 1,200-unit perception range and separately waits
for nearby in-flight waves, including waves left by a dead enemy. Missing native
resident diagnostics fail explicitly. These failed attempts remain retained.

The subsequent public-internet run, folder suffix
`cfdd7049f9864402b41e755906407db4`, verifies **95-103** with 40 ms delay and
20 ms jitter. Independent checks confirm **12 personal arrivals**, **five own
essence pickups** and **1,139 matching player/world/native states** before 104.
Both players remain alive and finish at **100/100 health**. Residents 50/51/52 are
dead; resident 65 remains alive with both native line-of-sight flags false.
Both 101 and 103 arrivals are verified on the real tilted support, including
Red's 101 arrival near height -173 and her later 103 arrival near height 10.
These are supported board positions, not a widened claim for a lower floor.

The run returns failure after 92.799 seconds at **104**: both riders moving to
the rear tip the board down, and its projected footprint no longer supports the
next target. Both remain alive. The diagnostic correctly rejects that goal.
The ordinary checkpoint before 103 has SHA-256
`d51582ee7d438652ad491a64677636e266bde6e185471afac7a20c8aec7b1f00`.
The next route must coordinate the upper exit, letting one rider reach the upper
ledge before both load the rear. No complete 95-117 or full Funhouse pass is claimed.

All evidence folders above are under ignored `private/coop/funhouse-route-<suffix>/`.
The frozen executable SHA-256 is
`c27cb1150bab1b94671ac2367ca1a0997ce59dceea854953554697b386107eaa`.
It passes **866 active Rust tests** (one ignored), the locked release build,
**63 battle / 41 machinery / 37 scene / 8 single-player component** records and
**94 Checkmate** records. Six new native world cases cover flat and +/-45-degree
boards with either player, query purity, and lower-floor/airborne/wrong-support
rejection. Single-player component gameplay matches the baseline, excluding only
two measured benchmark durations. The **26 Funhouse guards**, **13 recovery guards**
and **seven source-package tests** pass. No screenshots were opened.
Overall remains **48.1%**, six combined campaign rows and 33 open; earlier contact
repeats, full traversal, recovery/rejoin combinations and broader effects remain required.


### Queued upper-exit timing, 29 October 2026

The normal checkpoint before 103 was replayed on the same frozen `0d43ade`
build, with 40 ms delay and 20 ms jitter. The first attempt, evidence
`46c2865748d543aa87eb71e914aa7d38`, still asks for 104 before the real brush
supports it and fails. Waiting for the board to reach -30 degrees instead lets
Blue complete both supported approaches, an actual upper jump and the 200-tick
hold. The waiting partner stays forward as a counterweight until Blue lands.

Evidence `d397e9ce9bea45acbc7d0bc4368d5028` independently verifies Blue's
103/104/105 receipts at ticks 87781/88397/88898 and the 106 hold from 88922 to
89171. All **296 shared player/world/native states** match. Both players remain
alive at **100/100 health**, with unchanged spawn stamps. This is deliberately
a single-rider result: the full personal-arrival verifier rejects the incomplete
pair. The run fails after 40.943 seconds when Red reaches 103 with the board
already at -19.56 degrees, after the raised approach window has closed.

The guest controller begins slowing more than 200 units before its target,
while its own weight is lowering the tip. A separate reduced-braking experiment,
`73c1596059bd4151861b3dba802997f4`, oscillates around the target and fails after
55.350 seconds, with Red down to 13 health. That experiment was reverted;
all eight current fixture dependency hashes match the retained 100/100 run.
No production physics, collision, networking or arrival acceptance was changed.

Next: make the guest's ordinary input sequence continuous and correctly timed
through the approach and jump, then require both personal upper-exit receipts.
The full 103-117 segment, Funhouse visit and campaign acceptance remain open.
Overall remains **48.1%**. No screenshots were opened for these runs.


### Paired upper exit and descending corridor, 29 October 2026

The opt-in `NetworkExit` test controller now sequences the final seesaw approach
with ordinary inputs using each client's own world observations. The counterweighted
rider waits for the tip; the solo rider runs continuously through the approach.
The controller records support and jump receipts only on frames actually sent or
restored. The verifier matches their body evidence and all three state hashes
against both peers. It rejects unsent ticks, wrong support, death/relocation,
missing receipts and fabricated jumps. This does not change game physics, saves
or the network wire format.

The first candidate, source `59b6e68`, failed honestly: receipts from intermediate
physics ticks did not cross the network, and Red missed the approach during a
network stall. That evidence remains in `83b9ab360ba54ff99a750cb782ac315f`.
Moving receipt capture to actual sent/restored frames produced source
`88b98f001452fba62a556c2741fd4ec3c9b8f9c6c92f4df68eb73f9f14a856f4`, executable
SHA-256 `ee40c9688250ee8d5784902140ef0e4e97bbba4465599a591aa1f2f74a0bc42e`.

Evidence `17e3d27ea01649928c7605680fc78c3b` independently verifies **103-109**:
12 personal arrivals, two 200-tick holds, two own essence pickups and 719 matching
player/world/native states. Both remain alive through this scope at 80/55 health.
Blue's exit receipts are ticks 87760/88373/88834; Red's are 89450/89469/90086.
The full run later wipes at 110 under normal enemy attacks, so it is not a full
103-117 pass. A normal checkpoint before 107 is retained with SHA-256
`39c819dc78d9ca0340759a8884a359c2c1984e4212ffa18f5da5d52ea360bf9a`.

Ordinary combat pauses at 107/109 and each player's normal essence collection
allow a later resumed **107-117** pass in `ee5ac69bdcf944d5adf062037d8306ac`:
22 personal arrivals, six own pickups and 1,524 matching states, final health
68/63 and no new deaths. Step 116 keeps strict eight-unit horizontal arrival but
uses the original native route's 52-unit height tolerance: its marker is at 112
while the actual supported sloping floor is near 69. Other walking steps retain
35 units. Guards reject airborne, wrong-command and outside-bound arrivals.
Earlier failures at the slope (`8557e1d4eb53497dbd28f16d36abfad9`) and at a
landing with no safe lateral neighbor (`32437fe35de04cacbebab6657c356892`) remain.

A narrow landing can instead queue forward to the next authored walking point.
This requires an actual ordinary move and observed supported arrival; it does not
credit the next route step prematurely. The targeted replay
`1933f7036020423d82e0472eafba4b61` proves this wait but later stalls after deaths:
the follower follows the leader's pre-respawn trail into her new destination.
The runner now queues personal arrivals through 111-117 and pauses for normal
combat at 112. Its old recovery audit also rejects two sloped revivals because it
uses a flat-ground probe. That failed audit is retained, not retroactively changed.

Source `94488e16adcb68dd601dd235df4e068d02b182e50986c947b2f7c0ca3c15379f`
(executable SHA-256
`bedcc403edaef7cfc8c189b1546db126c3ab3ed608a8cec7e69002eb04633d07`)
corrects the read-only recovery logger to use co-op's actual slope/edge placement
rules plus actual ground contact. Native checks reproduce both recorded slope
positions and reject floating and airborne bodies. Placement behavior itself is
unchanged. It passes **870 active units** (one ignored), locked release,
**63 battle / 43 world / 37 scene / 8 single-player / 94 Checkmate** records.
Single-player gameplay matches baseline, excluding only two benchmark timings.

On this build, `5ed98c7b581345e88b1e32aa17d08343` independently verifies
**111-116**: 12 personal arrivals, one forward wait, three own pickups and
1,010 matching player/world/native states. Red dies at 100737 and revives at
101937: exactly 1,200 active ticks, 25 health, preserved resources, safe supported
placement and verified spectator/return cameras. The scoped end health is 100/65.
The full replay stops at 117 after 186.695 seconds, with Red's navigation worker
still planning from approximately [60.45, -4475.49, 64.03] toward [32, -4460, 96].
Both are alive at the stop, 100/50 health. The failure is retained; the earlier
107-117 success cannot substitute for a reliable repeat of this newer fixture.

All runs use independent clients through the public internet with 40 ms injected
delay and 20 ms jitter. **32 Funhouse guards / 13 recovery guards / seven package
tests** pass. Current fixture dependencies match the captured latest run. No
screenshots were opened. Next: resolve the 117 navigation stall, then weapons and
pendulums from 118 onward. Full fresh traversal and remaining acceptance cases
are open; overall remains **48.1%**, six combined visits and 33 open.


### Delayed navigation and combat on the descent, 29 October 2026

The recorded waypoint-117 stop is about 32 horizontal units from its target but
also 32 below the approximate marker. It falls outside the test controller's
42-unit spherical handoff, while a grounded body on that floor can never satisfy
the planner's first 22-unit search. A new regression first fails on that recorded
offset. The controller now finishes a nearby walk only with a clear body sweep,
continuous floor support and the existing 35-unit walking height bound. Arrival
still requires eight horizontal units. Unit and real-map cases cover zero, 100 ms
and 200 ms observation/input delays, walls, unsupported edges and excessive height.

The runner also reads the authored Ice preference on resume and can send normal
Ice Wand primary inputs when the individual player owns it, has sufficient Will,
is dry, and has a visible target within 360 units. Tests reject missing funds or
ownership, water, excessive range, walls and a blocking partner. Input selection
does not modify inventory; the actual weapon simulation spends the resources.

Source `8a6352f3478d0f24a2442a2cc867b1a561af81dd44e656c0cd94a47ec3b61ba5`
passes 872 active units and battle63/world46/scene37/SP8/Checkmate94. The first
replay (`8beb1841f0484574a829a48f8f0d7d3d`) reaches 117 but Blue is knocked off
while waiting; the run later wipes. A normal fight on the wider footing before
117 resolves that handoff. Prefix **116-121** in
`657d979b12444c33a53edc2c66ac47be` independently passes six personal arrivals,
three safe waiting handoffs, one own pickup and 422 matching player/world/native
states. Both finish the scope alive and grounded at 100/65 health. The normal
checkpoint before 121 has SHA-256
`de7e1c3ce9b7441f8241bed94e1613b28b02504c0e4dc43d0b26034d66ba19c9`.
The full run later wipes at 124 and is not promoted to a segment pass.

The waiting-footing audit rejects a partner who has fallen, died or left its
supported stopping point. An acknowledged walking command may bring a displaced
but still supported waiter back; the runner observes that return within 15 seconds
before continuing. It also pauses normal static-corridor movement to fight newly
visible enemies, then resumes both ordinary movement commands. Attempts
`8036d07fb6db43c898ed866d5632cd4f` and `830da397dd5c456f81d7ffa9ec5194c1`
remain failed: the former catches an unsettled waiter, and the latter wipes at 123.
The latter receives actual paid primary Ice emission intervals for both players
on both clients (host sample counts 50/19; guest 45/18). This is replicated-state
evidence from a failed run, not a rendered-particle comparison or route pass.

That last trace isolates a second input-controller defect: Red keeps 65 health
while her dodge takes her off the bank. Its old 24-tick input plus 48-tick braking
prediction ignored the roughly 200-230 ms round trip. The revised prediction
includes in-flight motion, delayed observations/inputs and the braking tail;
shorter analogue sidesteps are available when a full-speed one is unsafe.
A regression simulates 1,200 ticks on a narrow platform for zero, 100 ms and
200 ms observation/input delays, requiring continuous ground contact and useful
movement. Authoritative player physics, enemy behavior and save formats are unchanged.

Frozen source `c8f333c7757edb6a01e20018f4a5dee8b582f9e055058e29f6b5eaf4a09ded57`
has executable SHA-256
`62a23773a8c042fbfeae05c9f6a3fbf8245b9378256a9965fc436b6b537fb68a`.
It passes **873 active units** (one ignored), locked release,
**63 battle / 46 world / 37 scene / 8 single-player / 94 Checkmate** records.
Single-player gameplay matches baseline except two measured benchmark timings.

On this source, `87c4d6f9ce4943b2a06e4160d8ea7f9b` independently verifies
**121-125**: ten personal arrivals, five safe waiting handoffs, two own pickups
and **1,033 matching player/world/native states**, with final health 90/30 and no
deaths or revivals. No paid Ice emission is claimed for this prefix. At 126 the
full run stops: Blue is at approximately [-287.17, -3545.22, -30.94] and Red waits
at [-287.83, -3509.23, -31.97], exactly the required 36-unit separation. The real
neighbor diagnostics offer only forward/backward points on this narrow bank.
Moving the leader backward into this lane blocks the follower's approach to
[-288, -3460, 0]. The bank and subsequent pendulums require a coordinated sequence;
player separation and personal arrival checks must remain intact.

The Python suites pass **34 Funhouse / 13 recovery / seven source-package** tests.
All live tests use independent clients over the public internet with 40 ms injected
delay and 20 ms jitter. No screenshots were opened. The full visit remains open,
and overall progress remains **48.1%**, six combined visits and 33 open.


### Normal descent checkpoints, 29 October 2026

Run `9123ca8df2dc4a2e877aa4d64424a3d5` fails when it requests the normal
checkpoint while Red is dead. The menu correctly refuses. The runner now waits
for the ordinary revival, when enabled, before requesting that same save;
shared wipes still fail validation. This branch of the runner remains without
a successful live death-before-save replay.

The next run, `8263668a24884f27b056502c2c86522a`, passes the complete requested
**121-125** segment on the same frozen `c8f333c` source. It verifies ten personal
arrivals, five safe waiting handoffs, two own essence pickups, one forward wait
and **1,079 matching player/world/native states**. Eight normal combat pauses
occur. Both finish at 70 health, with no new deaths or revivals. No paid Ice
emission intervals occur in this replay, so it adds no particle-rendering proof.

Normal checkpoints are retained before 125 (SHA-256
`24dafa706db3b14f0ee3092b39adb5b9e0c3e8554d27cb3849646b4fad7d1fd1`)
and before 126 (SHA-256
`7c95be3c71f9786703969f876e8e372fd5958876a04bccb4bde9728deb09e831`).
Both bodies are alive in each save. All eight captured Python fixture dependency
hashes match the tested current files; source and executable identities match the
frozen build. These checkpoints let the next bank/pendulum test resume through
the normal restore path. The narrow-bank coordination at 126 and the full visit
remain open; there is no additional campaign-row or percentage credit.


### Queued bank and pendulums, 29 October 2026

Waiting beside waypoint 126 blocks the only supported approach lane. The runner
now sends one player through the bank, three pendulums and far-side jump before
bringing the other across. It uses ordinary walking, running and jump inputs;
the authoritative physics, enemies, shared collision and saves are unchanged.
Each short platform landing is recorded only at an actual sent/restored frame.
Its verification requires both peers' matching player/world/native state and
movement, an actual personal jump, a living grounded body, and native contact
with the intended moving brush (117, 116 and 115). Death or relocation cannot
complete the sequence. Each player also performs the full 1,200-tick bank jump
hold and both fixed far-side jumps.

On source `d5372a1`, run `43bcd21d53d24392a62cb4c0591bb252` independently verifies
Red's 128-132 sequence, but fails because the far landing has no safe parking
neighbor. The next attempt, `e03b6ac0ebc94a60965d620133648c9a`, also completes Red's
next jump, then stalls at 134. The grounded body is approximately
[450.52, -3270.58, -7.04] beneath its [450, -3270, 30] marker. This specific
sloped marker now uses the original native route's 52-unit height bound while
retaining eight horizontal units, real ground contact and an acknowledged walking
command. Guards reject an unrelated lower floor, horizontal miss or airborne body.
These failed paired runs retain only the independently checked personal scope.

Attempt `d8ff8b10319840499ab4fbcc1f523360` reaches the third pendulum but exposes
an unsafe immediate exit jump. While the guest's input travels, the platform
carries Red from about [363.02, -3624.14, 55.81] to
[360.46, -3708.85, 72.16], outside ordinary jump range of the bank. Red dies.
The normal revival is independently verified at ticks 117964-119164: exactly
1,200 active ticks, 25 health, preserved resources, supported placement and the
spectator/return camera transition. That recovery does not make the crossing pass.

The exit controller now forecasts platform carry through the input delay and
short running start. It waits on the actual moving platform until the bank is
within the ordinary ballistic jump range, then runs and jumps. Captured unsafe
departures fail its new regression. Source
`c79ac854cc6e1facbc5618f288f0b07be737c9a7830bc4c67abaa350b0536dd4`, executable SHA-256
`8d8583df9a276009cfb83d8b6fff5d1a6e93831160e1c7ab7bbc637e58dfa15b`, passes
**875 active Rust units** (one ignored), locked release,
**63 battle / 46 world / 37 scene / 8 single-player / 94 Checkmate** records.
Single-player component gameplay matches baseline except two benchmark timings.

The paired run `7c9bdf3c69a54dc2bc6af7237b4a0b9c` then passes **126-134** with
**14 personal arrivals**, all six individual pendulum landings, the two bank
jump holds and **1,041 matching player/world/native states**. Red's bank hold
lasts 1,201 observed ticks with eight actual jumps; Blue's lasts 1,200 with ten.
The final waiting-footing check confirms Red stays alive and supported off the
incoming route. Both finish at 70 health, with no deaths or new spawns. A separate
audit repeats the receipt checks and matches 1,052 retained states including the
additional normal client-exit observations after the runner's verification point.
All eight captured Python dependencies match the tested current files.

The normal checkpoint before 135 has SHA-256
`ed78029b571b35fa4f6763e642d5fde669734329412dbcf6207ff29e96b1ca50`.
The run uses two independent processes over the public internet with 40 ms delay
and 20 ms jitter. **38 Funhouse / 13 recovery / seven source-package** guards pass.
No screenshots were opened, and no new particle-rendering claim is made. The next
scope starts at 135. Fresh full traversal, the opposite crossing order and the
remaining visit acceptance cases are open; overall remains **48.1%**.


### Co-op corridor and Tweedles entrance, 29 October 2026

The unchanged `c79ac85` native build now passes a separate **135-138** public
internet segment in run `d061b4b894894e24bf961d18abc956ab`, with 40 ms delay and
20 ms jitter. There are **six personal arrivals**, three supported waiting
handoffs and **1,092 matching player/world/native states**. An independent audit
also matches all 1,096 retained states, including normal shutdown observations.
Both remain at 70 health throughout, without new deaths or spawns.

Red is the first entrant and owns the original watched Tweedles scene. Both peers
observe all nine dialogue lines, with 223 host and 221 guest active-scene samples.
Blue is hidden and stationary during the scene. The shared reunion occurs at tick
129228; both are grounded by host tick 129253 and guest tick 129264. They remain
48 units apart, body-clear and visible, with unchanged health and personal
inventory. The original actor's brief descent is checked separately: at most
32 vertical units within 60 ticks, with no horizontal drift, damage or revival;
the partner stays grounded. The normal checkpoint before 139 has SHA-256
`986689438ce34fe5d022df6b9623b916fb0da9c9b0078dc9b66b14b0cce3bfc7`.

Three test-controller corrections precede this pass. The exact authored static
walking markers now use the native single-player route's 52-unit vertical bound,
retaining eight horizontal units, actual ground contact and acknowledged walking
input; moving-brush projections and jump checks retain their existing bounds.
A captured grounded step-136 approach was 36.05 units below its approximate marker,
which the previous 35-unit checker incorrectly rejected. The scene verifier now
observes the original actor's bounded gravity landing before testing the completed
reunion. Finally, the scene entrance is a trigger/handoff milestone: it closes
before the old step-138 marker can be reached. The runner no longer sends both
players back into that closed doorway after the verified reunion. Every other
movement step still requires separate personal arrival evidence.

Earlier run `1fdac9f4ac1545c48e1d5821135f250f` has scoped watched-scene proof only;
its verifier stopped before saving. Run `edd6ba75bd23454a94efe5a5e024f274` saves
the valid reunion but fails the attempted return to the closed entrance. Neither
is promoted to a full segment pass. Guards reject missing/duplicate/unsafe scene
proof, fictitious entrance arrivals, wrong targets, airborne or unsupported
bodies, missing dialogue, changed resources, overlap and unsafe landing drift.
**44 Funhouse guards**, 13 recovery guards and seven source-package tests pass.
The frozen native identity and executable hash are unchanged; its existing
875-unit and battle63/world46/scene37/single-player8/Checkmate94 checks apply.
No screenshots were opened and no new particle-rendering claim is made.

A further resumed run, `a33042a4fad84486b78f3c7b8fa73276`, independently verifies
only its **139-143 prefix**: ten personal arrivals, five safe waiting handoffs and
**505 matching player/world/native states**, ending at Blue 60 / Red 70 health
without deaths or new spawns. The audit cuts off at the retained before-144
reports (host 133119, guest 133110); later failure observations are not counted.

The complete 139-146 attempt fails at 144. Blue dies while waiting below the
climb; Red reaches the 144 target but subsequently dies before Blue's revival,
causing a normal team wipe that the traversal verifier correctly refuses.
No full climb, boss defeat or post-wipe continuation is claimed. The normal
checkpoint before 144 has SHA-256
`0c6cbac7cb306a23541a1adf04eaf9f03b6c0d58e889ea2f0d5dfbe0b80e7222`.
The next scope is coordinated movement and survival at **144**. These remain
checkpoint segments, not fresh full traversal or a completed campaign visit.
Overall stays **48.1%**.


### Co-op arena climb, upper bridge and personal essence, 29 October 2026

The test controller now coordinates the ordinary **144-146** ramp movements.
Both players advance while fighting. Subsequent following goals are actual
grounded leader positions, selected at least 96 units behind the current leader
while they share a pending target. The follower still visits every authored waypoint itself. Ordinary jumps reduce exposure
to ground attacks. The first finisher clears the upper destination using a
native supported neighbor. No positions, resources, enemy damage or combat state
are changed by the test controller.

The first concurrent attempt, `a9dce1d06dce44c2bd27e39e4289308b`, records all six
arrivals but fails its final unacknowledged waiting command. The wait now requires
the actual native command receipt. Run `56d8fef2587d4e168897f9487eccb572` exposes
crowding while both brake at 144, and Blue dies. The spaced walking attempt
`f7b74150213e44329721eef7649fdaca` advances both past 144 but still loses Blue to
attacks. These remain failed runs, not completed climb evidence.

Run `cfe35e7dc64f4b178ad0bb1d6ab282be` then passes **144-146** with six personal
arrivals, two observed follower footings, one safe final waiting handoff and
**203 matching player/world/native states** (206 including shutdown). Blue makes
four ordinary jumps and Red three. Neither dies or relocates. Final health is
Blue **6.8084** / Red **59.4828**. The normal checkpoint before 147 has SHA-256
`0f85623177527c1dc48d5334bbf70f0e0da5ff879b8dd9df3125a69235366ab4`.
The low final health remains part of the recorded test state.

The next bridge sequence is queued so each player uses the actual takeoff pad,
performs one ordinary jump and independently crosses both upper waypoints. A
nearby goal above the body requests the jump; native physics determines whether
an upper landing exists. The receipt requires the correct grounded pad start,
a real personal jump count, at least the authored 120 ticks, an acknowledged
landing and no death, scene or relocation during the action. The first player
clears the far landing before the other uses the pad.

Run `119ad58422a94b80896d476eb4b20725` passes **147-149**: two such pad jumps,
four personal bridge arrivals, a safe far-side waiting handoff and **342 matching
states** (345 including shutdown). Blue's observed jump interval is 140 ticks,
Red's 184; each has exactly one actual jump. These early intervals start at
command issue; the final replay below excludes delivery delay by starting after
an acknowledged actual jump. Both peers independently observe
personal arena-essence generations change from [0,0] to [1,0], then [1,1].
Blue gains exactly 25 health (6.8084 to 31.8084) and Red separately gains 25
(59.4828 to 84.4828), with corresponding personal Will gains. Each pickup flips
only its owner's next pad and starts only their cooldown; the partner receives
no health or essence-sized Will increase. Red subsequently takes normal damage.
Both survive at **31.8084 / 38.2388**. The normal checkpoint before 150 is
`9617deb0f9efbb2140d8f681e9ea9cab19d2cc32a03652b0a97cf0eef7a38339`.

Run `7823bebb917d4b94b6bf53c34d7def27` separately passes the **150** hold: both
acknowledged holding commands remain active for at least 900 ticks (951 host,
965 guest). There are **141 matching states** (144 including shutdown), unchanged
health and no deaths. Both next personal essence pads are ready; this run does
not claim a cooldown transition because both were already ready at its first
observation. Boss health remains 125.8698 / 480.775, so defeat is not claimed.
The normal checkpoint before 151 has SHA-256
`a88d9f8eead038ccc9332204b6ea6ad0ccfbabbae11d2a68d88f45a3f055692a`.

The final combined **147-150** replay, `487f9a93e20e45f3b8305e70b6e4e8c2`, uses
the stricter jump timer. Blue's acknowledged jump is observed at 134755 and its
landing/hold endpoint at 134890 (135 ticks); Red's are 135838 and 135977 (139).
Command delivery cannot satisfy the 120-tick input period. The run passes both
personal jumps, all four arrivals, the final waiting handoff, and the paired
hold for 942 host / 957 guest ticks. There are **479 matching states** (483
including shutdown), no deaths or relocations and final health **31.8084 /
38.0889**. A separate audit again verifies both personal 25-health/Will pickups,
the unchanged partner resources and independent pad/cooldown state. Both next
pads are ready. The latest normal checkpoint before 151 has SHA-256
`d4f5826ce9786017958bb58a29a53b1e7823b355e231b75d0f09963a558bb366`.

These successful runs use unchanged source `c79ac85`, separate public-internet
clients, 40 ms injected delay and 20 ms jitter. Independent retained-state audits
repeat the receipts and player/world/native hashes; each test's captured fixture
hashes matched its current files when audited. **46 Funhouse guards**, 13 recovery
guards and seven source-package tests pass. Native source and executable remain
unchanged, retaining the earlier 875-unit and 63/46/37/8/94 component results.
No screenshots were opened; particle simulation observations are not a new
rendered-effects claim. These resumed segments do not prove fresh full traversal,
the opposite order or the later boss ending. The next checkpoint is **151**;
its later descent/ending evidence follows below. No completed campaign row or
percentage is added. Overall remains **48.1%**.


### Second pad, descent and boss ending

Run `bdb6d4d0e5374eeb9c1da6e1e9ae6a9c` passes resumed **151-155** on the same
frozen source: eight personal movement arrivals, both acknowledged actual pad
jumps (120/130 active ticks) and **396 matching player/world/native states**
(399 retained including shutdown). Each player's own second essence generation
advances from one to two, gaining 25 health and Will while leaving the partner's
health, pad side and readiness unchanged. Both survive at about **56.8/63.1**
health, with no deaths or relocations. The normal checkpoint before 156 has
SHA-256 `0f78b5e2449dc45431352b93fdb44a6f84d090854e213d879821747d34099dc1`.

The dedicated `tools/test_coop_funhouse_boss.py` resumes that unchanged checkpoint.
Run `2aec79c1680e44dfb0d068cbf7429e2e` uses six ordinary movement/attack commands
to finish the remaining Tweedle. Both bosses reach zero health. Blue owns the
original Hatter ending; both clients observe all eight dialogue lines, with Red
hidden and stationary, personal inventories/health preserved, and the original
floor-collapse tail. The observed ending-to-arrival intervals are 781/785 ticks,
covering the original 6.5 seconds. Both actually enter **Hatter1, epoch 2**, alive,
grounded, clear and separated at the same 56.8/63.1 health.

The boss run verifies **1,178 matching player/world/native states** in Funhouse;
the independent retained audit also checks **four matching states in Hatter1**.
No deaths/revivals occur in either run. All eight descent and nine boss fixture
hashes match their captured sources, and both independent audits pass. The boss
controller stops on actual victory/transition: interrupted combat legs are never
credited as personal waypoint arrivals. It does not force enemy health, progression
or player placement. Native source/executable are unchanged; the previous native
checks still apply. **50 Funhouse guards**, 13 recovery guards and seven package
tests pass, including rejection of absent defeat/dialogue/tail/exit, visible or
moving partners, changed inventories, unsafe arrivals and wrong second-pad proof.

These two independent checkpoint segments do not constitute a fresh full route.
Fresh traversal, the opposite lead order and ending owner, skipped ending, remaining
recovery/reconnect cases and rendered environment-effects checks remain open.
Six campaign rows retain combined evidence; overall stays **48.1%**. No screenshots
were opened for these runs.


### Fresh-route integration and mirror timing frontier

`tools/test_coop_funhouse_full.py` now joins the authored entrance through 138
with the arena fight and original ending. Both players must have their own
corridor/machinery/pendulum arrival evidence, gas must be open and all eight cell
bits set. The runner starts with new profiles and rejects a resumed entrance.
Actual boss victory can interrupt optional arena healing movements; unfinished
waypoints never become completed arrivals. This runner has **no full-route pass**.

Fresh run `4a98cc596f78481abffb2988f9ce2cd1` on `c79ac85` has independently audited
**0-28** evidence: 44 personal arrivals, seven own pickups and **2,763 matching
player/world/native states**, ending at 100/100 health without deaths. Its normal
checkpoint before 29 has SHA-256
`eb33096775f6d5bec151668ccc67ee25c59b08b2b63fd868d69a767f110776c7`.
It fails at the mirror switch. The later cell-switch reposition policy had
shortened the shooting wait to eight seconds on a perch that takes twelve seconds
to turn. A fourteen-second retry still fails; restoring the mirror fixture's
original bounded 90-second real-Blade wait activates the switch. The cell-switch
reposition policy is unchanged and no switch is forced.

The next retry, `391c4a4d04ae43838584998d294bba64`, activates the mirror and returns
Blue, but Red's padded 120-tick hold lasts 204 observed ticks and walks off the
perch. Red dies at 35,509 and revives at 36,709: exactly 1,200 active ticks, 25
health, with preserved resources and verified spectator/return cameras. This
is a failed crossing, not an arrival proof. Removing the extra hold padding alone
still fails. One separate retry times out while connecting the guest before play.

Source `1f789e9` adds delay prediction and lower feedback gain only to the opt-in
route driver's held movement. Local zero-delay input is unchanged. A native
physics regression reproduces the old controller's fall with delayed observation
and input queues; the corrected controller stays supported at 0/6/17/36-tick
queue delays. **876 active Rust tests** (one ignored), the locked release build,
Funhouse **battle63/world46/scene37/single-player8** and **Checkmate94** pass.
Single-player component output matches baseline except two measured timings.
**51 Funhouse guards**, 13 recovery guards and seven source-package tests pass.

Live run `fce3ab07947249fcb8b10d25cbe21c5b` keeps Red alive/grounded on the rotating
perch through the 90-second hold, but stops on an incorrectly added takeoff-point
arrival check. That check was removed: the authored action is directional input,
not point arrival. Living, unchanged-spawn, grounded and minimum-duration checks
remain; the following bank jump still requires its own actual jump and arrival.
The next run, `d30e89298f6a4202aa59e22e36fa3fc5`, completes the hold but **fails the
return jump**. Red's jump counter does not increase before the fall; her later
25-health revival on the bank is rejected. The next work is to trace the requested
jump against authoritative input and movement timing. Neither run is a mirror or
full-route pass. No screenshots were opened. Overall stays **48.1%**, with six
combined campaign rows and the existing remaining scope.


### Mirror input timing and driver cost (4 November 2026)

The failed `d30e892...` run's retained reports show the host control-reset counter
rising from two to three as Red's requested jump counter increases, after the
host accumulates 386 ms of simulation debt. Those reports do not identify the
exact consumption tick, so cancellation is a supported candidate cause rather
than proof of a grounding fault. New opt-in diagnostics retain up to 32 host
jump/reset events, actual before/after physics, received/applied/pending jump
counters and driver/simulation/send/draw/observation timings. These fields never
change gameplay state, network frames or saves.

Diagnostic source `46cc79b` passes resumed mirror 29-32 in run
`f3c6ad9e7d0947fc85930c60832f9070`: four personal arrivals, two held approaches,
one actual shared switch, 789 matching states and no deaths. Red's real return
jump is applied at tick 34,979. Both finish at 97 health; no reset occurs.
The host's driver costs 10.42 ms median and 39.45 ms p95 per rendered frame.

Source `1839b9f` rejects only attack candidates whose whole hit box lies beyond
the existing weapon range (with a conservative one-unit margin). Every body
remains in the collision context and velocity tracker. Closing and Ice queries
also reject impossible ranges before tracing. A regression compares the original
unfiltered choices across range edges, tall exposed bosses, walls, the partner
and funded/dry attacks. Production combat, movement and the suspension barrier
are unchanged.

Run `84482faac50a40fbad9bf378ce283f6c` passes the same segment: four personal
arrivals, two holds, the actual switch, 816 matching states (823 retained),
52 recovery rows, no deaths and 97/97 health. Normal checkpoint33 SHA-256 is
`509aa1cc7b99cb83356d21ecdb35081a0d6ef9bd3f899d0883bf801028e35b33`.
The driver costs 0.26 ms median and 1.07 ms p95; frame p95 is 30.21 ms versus
90.80 ms in the preceding run. Both used the authorized local-PC fallback while
Anode was occupied, independent public-internet clients, 40 ms delay/20 ms
jitter, and the unchanged earned checkpoint29. No screenshots were opened.
These measurements support the driver optimization; they are not an Anode
repeatability claim or a complete campaign visit.

All 877 active Rust tests (one ignored), the locked release, Funhouse
battle63/world46/scene37/single-player8, Checkmate94, 51 Funhouse guards and seven
source-package tests pass. Single-player output differs only in two benchmark
timings. A fresh full-route retry is running; no full-route pass is claimed.
Overall remains 48.1%, with six combined campaign visits and the existing open
contact, recovery, opposite-order and full-route requirements.


### Fresh prefix and return-corridor combat (4 November 2026)

Fresh run `bbfc4d741b41498c9ebd2a9f6e6413ae` on `1839b9f` now passes the complete
prefix verifier through **0-37**: 58 personal arrivals, seven own pickups, two
holds, the real mirror switch, watched opening ownership/reunion and 5,011
matching player/world/native states. No one dies before the boundary. Both
have 22 health by then; the following step38 causes a team wipe. The generation
change correctly fails the full route. No control reset occurs in this run.

The fixture was continuing long waypoint walks while two active Automatons
attacked, preventing its no-goal dodge tactic from operating. The existing
pause-and-fight behavior now also applies to grounded return-corridor steps
35-42. It uses ordinary combat inputs and collects each player's own available
drops before restoring the requested waypoint. Enemy stats, weapon costs,
physics, deaths and checkpoint verification are unchanged.

Retry `2ba972d51d6047bab39655f490d734f0` uses the unchanged checkpoint35 earned by
that fresh run (SHA268f8ae13799bda709253c3469bbbed4a83364dcd4e3b20387af9971ef2120d4).
It passes **35-42**: 16 personal arrivals, six personal essence pickups, 2,636
matching states (2,643 retained), 165 recovery rows, no deaths and final health
100/80 (minimum30/30). It saves normal checkpoint43 with SHA-256
`dbed01198f3b48c81a0f4f5f71e08556f48d8217685fcf81d327e800c7cd6dd6`.
The independent retained audit passes, including all captured/current fixture
hashes. The 51 Funhouse guards pass. Both runs use independent public-internet
clients on the authorized PC fallback while Anode is occupied, with 40 ms delay
and 20 ms jitter. No screenshots were opened. The earlier fresh prefix and this
resumed corridor are not a combined full-run pass; that remains the next test.
Overall remains 48.1%, with no new completed campaign visit.


### Combined fresh prefix and four recoveries (4 November 2026)

Run `f4d6dc8aa1304a3d89ca3e6845124746` uses frozen native source `1839b9f`
and the final corridor fixture from `fbee005`. It verifies fresh **0-55**:
82 personal arrivals, 16 own pickups, seven paired holds, two actual switch
shots and 10,511 matching player/world/native states. The original opening,
mirror and corrected return corridor pass together. The complete run remains
FAILED at switch step 56 after five ordinary shooting positions; both players
remain alive at 75/95 health. The exact aiming cause is not yet established.

The independent retained prefix audit calls the full Funhouse verifier against
the saved before-56 boundary and bounded histories. Every captured fixture
matches current source. Its 5,412-row recovery trace proves four natural
revivals (Blue three times, Red once), each exactly 1,200 active ticks later at
25 health, with clear safe supported placement, preserved personal resources,
and independently rendered spectator/returning-player cameras. It verifies the
unchanged checkpoint43 SHA-256
`0fa861781383932bdbb807c3b0af282dfb64dea71531e6d41c44c7215f40b860`.

Proof is retained locally as `verified-recovery-prefix-0-55.json`. The run uses
independent internet clients, 40 ms delay/20 ms jitter and the authorized PC
fallback while Anode is occupied. One early input suspension is recorded; this
is not a zero-reset or Anode-repeatability claim. No screenshots were opened.
No campaign row is added. Overall remains 48.1%; next is the focused step-56
switch failure, then fresh full traversal and the remaining campaign cases.


### Visible shooting approaches (4 November 2026)

The cell3 failure is now reproduced as a geometry problem: native rotating door
entity 160 crosses the line from the nearer western retry positions when fully
open. A northern supported position has a projectile-width sight line at door
fractions 0, 0.5, 0.8 and 1. The fixture's nearest-only choice could repeatedly select
blocked positions. The normal aiming, weapon and door rules are unchanged.

`Report.switch_approaches` now filters the existing safe neighboring footings
through native world/target/partner occlusion. This runs only on the 200 ms
report observation, not every physics tick. The Python fixture prefers those
positions while retaining its 96-unit home bound, 48-unit partner clearance,
ordinary walking and actual movement/shot receipts. No destination is applied
as an authoritative placement, and no switch is activated by the fixture.

Source `a797a001fca2e86050999ac1dcbdc08eecea6521ef98cf39d45ed33bd630a198`,
frozen executable SHA-256
`becf1b7f6faba2785a85548c4cbcb473edc16cf90eff0fd0e9f5ae8c0a9c5af5`, passes
878 active Rust tests/one ignored, the locked release, native Funhouse
battle 63/world 50/scene 37/single-player 8, Checkmate 94, 52 Funhouse guards and
seven source-package tests. SP component output matches the baseline except
two benchmark timings. The four new world cases prove actual supported footing
and sight lines at the door angles; these are component checks only.

Anode run `0a0202ca99b741cc9d612b156a17bfe8` resumes the unchanged BLUE63
checkpoint43 through independent internet clients with 40 ms delay/20 ms jitter.
It passes 43-56 with 14 personal arrivals, five paired holds, two real switch shots,
1,753 matching states (1,760 retained), 105 recovery rows and no new deaths.
Both finish at 75/95 health. The independent retained audit checks captured and
current fixture hashes, movement/shot receipts, network agreement and recovery.
Its choice audit shows two visible approaches differ from the old nearest-only
choice; the third uses the same now-visible point. This exercises the correction.
Normal checkpoint57 SHA-256 is
`74d95b04cce602661f70cbce8131d6e6f9841414c1ac26af0baf77621ddc76c3`.

For further headless diagnosis, `LOOKING_GLASS_FUNHOUSE_SWITCH_PROBE` may point
to a local JSON object with `world` (an Interactions snapshot), `thread`, and
`feet` (at most 64 finite positions) while running `--funhouse-coop-world-check`.
It returns `diagnostic_only` sight-line and eye-origin Blade trajectory samples.
It does not advance a live game or prove a real animated-hand shot, movement,
NPC combat or traversal. Keep retained snapshots and original assets private.

No screenshots were opened. The combined fresh full route remains unverified;
this scoped retry adds no campaign row. Overall remains 48.1%.


### Separate connection-setup timeout (4 November 2026)

Fresh run `6c46a1aa2c7a4024a2464b1db0795967` on `a797a00` failed before paired
reports or gameplay. The guest exited with the old generic connection/load
timeout. The host's first ready report was created about 1.3 seconds after the
guest launched, about 14 seconds before the failure. No guest phase was retained,
so the exact external network delay is not proven.

Source inspection found an independent startup mismatch: Guest initialized its
connection deadline with the 15-second identity-check limit, while DNS retries
alone allow 20 seconds, followed by TCP/TLS setup. Initial setup now has its own
60-second ceiling. Once connected, the original 15-second identity deadline
starts afresh; the 120-second load deadline remains. Errors distinguish initial
connection from host check/load. A controlled pending-worker test passes beyond
21 seconds, checks both connection and handshake expiry, and proves timeout and
cancellation stop the worker without real waiting.

Source `f9e702908fcaa1e7cd7713cd3364d0e5ea24ac1881948e8c7b3315f3b0e21c24`,
frozen executable SHA-256
`67079bbc64910eb0841daf22f98b75448ba2adfd6b51cec374e320de068e43f7`, passes
879 active Rust tests/one ignored, the locked release, Funhouse native
battle63/world50/scene37/single-player8 and Checkmate94. SP component output
matches baseline except two benchmark timings; worktree888/package7 also pass.

Fresh retry `1b9f5c22091942ccbd68466a497bfef9` uses two independent internet
clients with 40 ms delay/20 ms jitter on the authorized PC fallback while Anode
is occupied. Both have connected and entered the original opening at 100/100
health. The full run is still in progress; no route pass is claimed. Overall
remains 48.1%, with six combined visits and the remaining campaign checks open.


### Fresh puzzle traversal and expected gas cinematic (4 November 2026)

The `1b9f5c22091942ccbd68466a497bfef9` run has ended with a full-run FAILURE
at 90: the fixture rejects an automatic scene during ordinary waypoint movement.
Native `pair_actor` correctly begins `GasOpenCine` one second after `all_cells`;
its original sequence completes after 9.3 seconds. The failing host frame is
98554, owner Blue, scene clock 0.025. The guest is 18 ticks behind and has not yet
received the scene. This is not evidence of a lost guest activation: the fixture
aborts before waiting for replication. No production scene change is inferred.

The full retained prefix verifier passes **0-88**, using the clean before 89
boundary (host 98463/guest 98459): 122 arrivals, 27 own pickups, 14 holds, eight switch
shots and 13,767 matching player/world/native states. Captured and current
fixture hashes match. A 5,585-row exact recovery trace verifies four natural
revivals (Red once, Blue three times), each after 1,200 active ticks at 25 health,
with supported safe non-overlapping placement, preserved personal resources
and independently rendered spectator/returning cameras. Both end at 100 health.
Proof is retained locally as `verified-recovery-prefix-0-88.json`.

The attempted 0-89 audit fails its requirement to end outside a cinematic; that
guard is preserved. The full run remains failed. Its ordinary checkpoint 89 has
verified SHA-256
`0b8e43ed48d9faece5c8d332ad8e06935b6632a345e58f197e0cdf5ead30436f`
and can support a focused real-input continuation. Next is an explicit gas-scene
wait and shared-reunion receipt before walking, then another full traversal.
No screenshots were opened. No campaign row is added; overall stays 48.1%.


### Explicit gas-scene wait and checkpoint replay (4 November 2026)

The fixture now waits for the scheduled Gas scene and both clients' completed
reunion before step 90, rather than treating the idle second before activation
as completion. Scene observations retained during checkpoint startup also count.
A checkpoint that already contains completed gas progress does not claim a new
scene. Unexpected scenes during ordinary movement still fail.

Independent internet clients on unchanged native source `f9e7029`, with 40 ms
delay/20 ms jitter, pass steps **89-94** from the unchanged normal checkpoint
listed above. Evidence: `funhouse-route-35465cdbbef443c88c64d6afc7f28f7f`.
Each client retains 50 active Gas observations, owner Blue, covering clocks
0.125-9.208 and 0.108-9.267 seconds. Both observe the actual reunion at tick 99668:
hidden/frozen waiting partner during the scene; grounded, clear, non-overlapping
living bodies afterward; matching warp stamps and preserved health/inventory.
All six personal walking arrivals and two paired holds pass, with 578 matching
player/world/native states (581 retained in the independent audit). Both players
remain at 100 health, with no deaths. The 38-row exact recovery trace and captured
fixture hashes also verify. Checkpoint before 95 has SHA-256
`deafb88161ad447b80c197a83ea583dad651db99dfa133b39534e2f5c1aef63f`.

All 55 Funhouse test guards pass, including the pending-scene gap, startup
observation, missing peer/owner, early completion, unsafe reunion and resource
corruption cases. The native executable is unchanged; its earlier locked build,
879 active Rust tests and native checks still apply. Source-package seven and
worktree 888 checks pass. No screenshot was opened; both clients close normally.

This is a watched Blue-owner checkpoint segment, not a fresh complete route or
live skipped/Red-owner proof. The earlier fresh run stays failed with verified
0-88; next is a fresh full repeat. No campaign row is added; total stays 48.1%.


### Fresh gas-gate repeat ends in combat wipe (4 November 2026)

Fresh run `6a1f420c41554ee980d55d16e6ae98ad` on unchanged native source
`f9e7029` and the gas-gate fixture fails at step 36 after both players die.
It uses independent internet clients with 40 ms delay/20 ms jitter on the
authorized PC fallback. The strict team-wipe guard is retained. The host
observes generation 1, both players spawned at tick 44577 with their saved
97 health near the normally earned checkpoint before 35. The last validated
guest report is still before that restore. Full two-client restoration is
therefore not claimed from this failed run.

An independent audit verifies only fresh **0-34**: 52 personal arrivals,
six own pickups, two paired holds, one switch shot and 4,788 matching states.
The bounded 288-row recovery trace has no deaths in that prefix; both end at
97 health. Captured/current fixture hashes match. The normal checkpoint before
35 has SHA-256
`4288107db30cd66b9c1ce3df868f5e51b306b297b40a7d1c1acf0fffe48d5d7b`.
The earlier 0-88 fresh prefix and separate gas 89-94 segment retain their own
evidence. They cannot be joined into a complete-run pass. No game combat,
resources, placement or recovery rule was altered to pass this repeat.
