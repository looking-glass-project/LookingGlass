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
