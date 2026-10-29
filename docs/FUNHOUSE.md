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
