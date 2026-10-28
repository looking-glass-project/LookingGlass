# Castling â€” Wchess2

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
it again. Continuous native traversal, independent internet clients and focused
recovery/rejoin checks are recorded below.

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

The independent-client evidence below extends these native results. Abrupt
disconnect, skipped network scenes and broader effects coverage remain open.
Its campaign row and the overall weighted estimate remain unchanged.

### Independent internet traversal checks

`tools/test_coop_castling.py` starts two independent clients and profiles through
the public invite path. Each client derives ordinary controls from its received
world. The fixture checks both full routes, personal cameras, separation, original
scene ownership and frozen partners, exact replicated state and Checkmate arrival.
Death, shared rollback, unscripted warps and cheats invalidate the route. Optional
`--save-before` steps use the ordinary pause-menu checkpoint operation. The default
adds 40 ms delay and 20 ms jitter; `--first blue|red` selects the first player.
Five Python negative-control tests ensure the presence checks reject overlap,
wrong cameras, unscripted warps and moving scene partners.

The first Blue-led public run on source `04dd45c` passes steps 0â€“7, then stalls at
step 8 with both players at 100 health. The delayed follower repeatedly reverses
around an old trail sample while the leader waits. All 4,078 common player,
world and native-resident hashes match; 1,251 host and 1,227 guest presence samples
pass. This is a failed full route, with no additional campaign credit. Reports,
the normal checkpoint before step 2 and the failed prefix checks are retained in
`private/coop/castling-route-0bbdde512884436d9146b211e95286a3`.

The diagnostic controller now reduces steering gain with input delay and follows
farther along recorded ground paths only after checking body clearance, continuous
support, liquids and hazards. A native-physics test starts from the captured
position and velocity, delays both observations and inputs, reproduces the old
oscillation and verifies settling at four delay levels. Separate negative cases
reject gaps, walls, hazardous paths and airborne shortcuts. All 836 active unit
tests pass, with one existing ignored test. Production movement is unchanged.

The fresh Blue-led public internet repeat on source
`e54bc16d3ef2e2a9a90f9d711565d18e4e661a3c2ff152d852da7d97015a0f53`
passes all **203 legs** with 40 ms delay and 20 ms jitter. Both original scenes
belong to Blue, the partner stays frozen, both players remain separated during
gameplay and both arrive in Checkmate at 75/100 health. All **8,749 common player,
world and native-resident hashes** match, and 2,570 host/2,549 guest presence samples
pass. Normal checkpoints before steps 2, 42, 110, 186 and 202 are retained alongside
the reports under `private/coop/castling-route-048ce8d9a3d94f7ba36ef842980980bb`.
Both processes exit successfully; the run takes 554 seconds without screenshot
inspection.

The fresh **Red-led run on the same frozen source also passes all 203 legs**,
with both original scenes owned by Red and both players at 100 health throughout.
Its 7,680 common player/world/native-resident hashes match, and 2,236 host/2,221
guest presence samples pass. Both clients reach Checkmate and exit successfully
in 482 seconds. Evidence and the same five normal checkpoints are retained under
`private/coop/castling-route-ed4b32971db5400d9be6d63aabb04798`. Both leader orders
therefore pass the full watched-scene route over the public internet. Skipped
scenes over the network, death/wipe recovery and broader rendered effects remain
separate requirements; no completed-row credit is assigned yet.

### Checkpoint resume and guest rejoin

`tools/test_coop_castling_resume.py` copies an unchanged ordinary checkpoint into
an isolated host profile, requires the native exact-state restoration receipt,
and traverses the selected course segment. With `--rejoin`, the guest exits
gracefully and a new process joins the same host. The checks require unchanged
positions, personal resources, royal progression, completed scenes and native
actor identities/health, plus at least 20 matching player/world/native states
after the new guest joins. Four negative-control tests reject progression loss,
forgotten scenes, actor resets and duplicate residents. A passing resumed segment
does not count as a fresh full route or as an abrupt network-loss test.

The initial Red-led exit replay on `e54bc16` preserved the reconnect state but
failed to board the portal. The delayed controller's endpoint floor probe could
look across the gap onto the far platform and omit the jump. Red fell to the
lower floor, lost 17 health and stalled. The failure remains recorded under
`private/coop/castling-resume-5120abe42d1b422789d0bc93de704d36`.

The diagnostic driver now samples the whole approach and accounts for acceleration
while input travels. A native movement test uses the captured starting position,
the level's ledge bounds and delayed observation/input queues: the old controls
omit the jump, while the new controls jump once and land. Its simplified deck
allows step-up recovery, so the real arched-platform repeat remains the stronger
evidence. Production physics, boarding guards and progression are unchanged.

Source `c1a202c287c47d15cf3ac5fd3b8b44bc0ea9a6d870bffbf03de000c3f6c1fa45`
passes the real exit replay from both unchanged step-202 checkpoints with public
internet joining and 40/20 ms delay/jitter:

| Saved leader | Exact restored tick | Matching states / after rejoin | Arrival Blue / Red health |
| --- | ---: | ---: | ---: |
| Blue | 61,967 | 55 / 35 | 75 / 100 |
| Red | 54,206 | 49 / 29 | 100 / 100 |

The counts apply separately to player, world and native-resident hashes. Both
players make one ordinary jump, retain their health and arrive in Checkmate;
both processes exit successfully. Evidence is in
`private/coop/castling-resume-52741abcb7624bafab98c0b855c6b724` (Blue) and
`private/coop/castling-resume-94aaab2959fb4ab586bfc9135b4c7648` (Red). All **837
active unit tests** pass on this source, with one existing ignored test. These
checks use state/log evidence without screenshot inspection. The following
section records the separate death, timed revival and team-wipe checks.

### Real combat recovery checks

`tools/test_coop_castling_recovery.py` resumes an unchanged normal checkpoint
before leg 77. The selected player approaches the knights without attacking;
the survivor retreats through the already cleared route with ordinary movement.
The fixture verifies the dead player's moving spectator camera, exactly 1,200
ticks before revival, 25 health, safe nearby placement and retained personal
inventory. It checks both clients and all three replicated-state hash families.
Five existing negative controls reject incorrect recovery evidence.

On source `c1a202c`, the Blue death at tick 21,269 revives at 22,469; the Red death
at 21,605 revives at 22,805. Both roles pass, with 302 and 367 matching states
respectively, over public internet joining with 40/20 ms delay/jitter. Evidence
is retained under `private/coop/castling-recovery-687f2dcfbaf14e8e86c24ed7cada84bc`
and `private/coop/castling-recovery-599b939d022649d9a12778fddc4a1c18`.

`tools/test_coop_castling_wipe.py` sends both players into the same encounter
without attacks. It requires shared checkpoint restoration, original individual
resources, royal progression, actor health and each client's own camera. Four
negative controls reject incorrect restoration evidence. The first two live runs
restore both saved players but **fail** the strict presence check: approaching
the partner beside a sloped wall permits overlap. Captured full states are kept
in `private/coop/castling-wipe-b5d5b43354c643eba83045ef8c7b9888`; an offline native
reproduction also overlaps after 14 physics ticks. Both old runs remain failed.

Source `62fe393291e2fbcd92d0e7a7f9176125a192b1f15da7ddac7d75184e13d37975`
repairs that collision defect. If a wall blocks the outward correction, Alice
can slide along its tangent while retaining her ground plane or airborne height.
The correction must still restore personal space and pass swept clearance,
support and immersion checks. Nearly perpendicular corrections are rejected to
avoid magnifying the displacement. The stationary partner remains unchanged.
The captured native approach now passes 120 physics ticks in both slots; the
old code failed on tick 14. A unit regression also rejects wall/corner crossings.

The unchanged public-client wipe fixture now passes: both clients restore at
tick 21,489, all 164 common player/world/native-state hashes match, and all living
presence samples pass. The saved individual Will values differ and are preserved
within ordinary regeneration bounds. Evidence is in
`private/coop/castling-wipe-23ff8a9120f9498c89ec6813889819f9`. All 838 active Rust
tests pass, with one existing ignored test; 82 shared-world, 43 campaign entrance/
recovery and 119 Pale Realm checks also pass. All three complete Castling
single-player check logs match their baseline. These focused checks add no
completed campaign-row credit; full-route repeats after this collision change,
skipped network scenes and broader effects coverage remain separate work.

Both real combat revivals also pass again on `62fe393`: Blue dies at 21,194 and
revives at 22,394; Red dies at 21,614 and revives at 22,814. Each returns with
25 health after exactly 1,200 ticks, with matching moving spectator cameras,
personal resources and safe placement. The three hash families match at 295 and
358 common states. Repeat evidence is under
`private/coop/castling-recovery-e8ad78548f3f48269232b663df31ec35` and
`private/coop/castling-recovery-2d1dad8ebb13409e9ca433765dadd9d2`.

### Current route repeat and held scene skip

The post-collision source `62fe393` Blue-led internet repeat completes all 203
legs, both original royal scenes and the Checkmate arrival at 75/100 health.
All 8,793 common player/world/native-state hashes match; presence checks pass.
Evidence is `private/coop/castling-route-512e8f087ff2442390f6b7709c0fc1fc`.

The corresponding Red-led repeat **fails at leg 128**, after completing 0–127
and both scenes. Both players remain alive at 100 health; 9,378 common hashes
and all 2,623 host / 2,606 guest presence samples match. Its retained failure
reports roughly 1,029 ms of queued host controls versus 206 ms guest round-trip
time. A stale steering estimate is a hypothesis, not an established cause.
Evidence remains `private/coop/castling-route-4671ddbc50904c51be0c32b81cbe82a1`.
This failure does not inherit the older build's full-route pass.

Source `592728d2811460b3cb556b5d6a23c34bc5171a2617698d55181c27050ac03799`
adds the original held Enter/controller-A scene skip to co-op. Only the living
scene owner may skip, after a fresh 0.65-second hold. The host runs the original
skip and safe reunion path; a waiting partner's held input cannot end the scene.
Both clients receive the same skip notice. Protocol revision 7 carries the held
control; it expires with other controls, and active holds retain their sampled
duration. Saved games do not retain partial holds.

The native royal scene checks now use actual held inputs rather than directly
calling the skip function. Both owners, both scenes, watched/skipped outcomes,
living/dead partners and restored scenes pass. All 839 active unit tests pass
(one existing ignored), as do 82 shared-world, 43 campaign entrance/recovery and
119 Pale Realm cases. All three complete single-player logs match their baseline.
`tools/test_coop_castling_skip.py` now passes all four independent internet
cases with 40 ms delay / 20 ms jitter. The partner holds skip alone for at least
180 ticks, then the owner holds it; both clients must acknowledge the identical
skip tick and safely reunite with unchanged health, inventory and selection.
Will stays frozen during cinematic ticks and resumes normal recovery afterward.
The fixture sends neutral controls through the handoff so automatic combat
cannot switch weapons or spend Will before the assertions inspect the result.

| Owner / scene | Partner hold / owner hold / skip ticks | Matching states |
| --- | --- | ---: |
| Blue / Queen | 6,392 / 6,584 / 6,672 | 83 |
| Blue / King | 25,818 / 26,009 / 26,110 | 65 |
| Red / Queen | 5,835 / 6,026 / 6,130 | 112 |
| Red / King | 22,757 / 22,949 / 23,054 | 87 |

Counts apply separately to player, world and native-resident hashes. Both clients
exit cleanly in each case. Evidence folders under `private/coop/` are
`castling-skip-d6f9498414ba4589a9329909d7152761`,
`castling-skip-a29f3a6a4b46464287d0bed8ecd50968`,
`castling-skip-8aa76fe1980649d2bdd84fa77dd52379` and
`castling-skip-6190c31ba2ec401e83e5cf2430278c84`, respectively.
Their isolated checkpoints are exact offline exports of retained pre-scene
authoritative snapshots through the normal checkpoint store; they are not normal
pause-menu saves or evidence of a new full route. The earlier failed fixtures
remain retained: one stopped outside the Queen trigger, one allowed automatic
weapon selection immediately after the scene, and one incorrectly expected Will
recovery throughout the cutscene. Three negative controls reject early,
inconsistent or incomplete skip evidence. No images are inspected.

### Recovery after a suspended host

The retained Red failure narrows the input-delay onset to a 560.78 ms host
frame at tick 36,236, during leg 124. The existing simulation watchdog cancels
controls for that frame, but later network polls admit more controls sampled
during the suspension. At tick 36,254, a coalesced unchanged hold already has
1,089.7 ms of unsimulated sampled time. Later movement changes retain roughly
one second of extra delay through leg 128, even after network round-trip time
returns to about 206 ms. The retained sequence is
`private/coop-castling-red53-suspension.json`.

Protocol revision 8 adds an ordered reset exchange after the host's simulation
watchdog fires. Until the guest's reply, the host validates and discards old
controls. The guest clears unsent old samples and replies with its latest button
counters; those counters are consumed without executing suspended actions.
Only subsequent fresh inputs drive Alice. Normal sampled movement intervals,
run-ups and jumps retain their existing timing. Repeated suspension while
waiting does not queue duplicate resets, and transitions/reconnections clear
the reset state.

All 843 active unit tests pass, with one existing ignored test. New real-session
tests cover old inputs spanning many bounded receive polls, cancelled suspended
jumps, one execution of a fresh jump, transitions during the exchange and
rejection of unsolicited/future reset replies. A native timeline regression
retains the original duration of a resumed run-up before its jump.
Source `cc2388ea85f7055571872c815217cdd910084ddba4edb7201f0d5b07ca3782de`
also passes the independent internet forced-suspension check with 40/20 ms
injected delay/jitter. `tools/test_coop_castling_suspend.py` restores an exact
offline export of the retained pre-leg-124 snapshot (tick 35,974), uses ordinary
controls through legs 124–129, and suspends only its own host process during
active ramp movement. The host is always resumed in a finally block.

In the passing run, suspension lasts approximately 640 ms at tick 36,855. The
ordered reset is acknowledged by tick 36,952. Six settled measurements from
37,096–37,216 show a median buffered control time of 36.68 ms (peak 92.17 ms),
well below the original persistent one-second backlog. Both players finish at
100 health, all 145 common player/world/native-state hashes match, all 79 host /
70 guest presence samples pass, and both processes exit cleanly. Evidence is
`private/coop/castling-suspend-d51b88e085574cc0b9bb88684ba170ef`.

The fixture requires at least 240 post-acknowledgement simulation ticks and at
least five distinct settled samples spanning 96 ticks; their median must remain
below 100 ms. Three negative controls reject absent resets, inadequate sampling
and a persistent 1,000 ms backlog. One earlier run cleared the backlog but failed
a single-sample 100 ms assertion on a 116.52 ms jitter spike while relay round-trip
time was about 1.3 seconds. That failure remains retained; the repeated passing
run above uses the sustained-window criterion. Two earlier fixtures missed the
pause trigger, and a separate attempt failed invite DNS resolution before play;
none is counted as passing.

The same build passes 82 shared-world, 43 campaign entrance/recovery and 119 Pale
Realm cases, the Castling native cast/world checks, and all three single-player
logs remain identical to baseline. The subsequent fresh full Red-led repeat on
the same frozen `cc2388e` build passes all 203 legs over the public internet path
with 40/20 ms injected delay/jitter. Both royal scenes belong to Red; both
players reach Checkmate at 100 health. All 6,519 common player, world and native
resident states match. Presence checks pass for 2,798 host and 2,544 guest
samples; minimum health during the route is 100/30. The retained evidence is
`private/coop/castling-route-76622a44563446b98cd81a28b69eec0a/verification-summary.json`.
This closes the failed leg-128 route repeat. Broader effects checks and the
remaining campaign gates still prevent completed-row credit.

`--wchess2-check` exercises real approach triggers, both royal outcomes, dialogue, pause, save futures, moving-platform support, bound exits, ambush alternatives and migration. `--wchess2-route-check` and `--wchess2-route-skip-check` traverse from the real entrance with Normal combat into a loaded Checkmate visit. Route checkpoints are private diagnostics; they do not change player saves. `--wchess2-render-check` captures eight staged views. Native save fixtures are `wchess2-queen-mid`, `wchess2-king-mid`, and `wchess2-exit-open`.

Use `tools/launchers/Launch-Castling.cmd` for a fresh visit with separate playtest saves. Release identity, exact test results and limits are recorded in `private/wchess2/WORKLOG.md`.

Native pathfinding, turn-rate details and precise sub-frame camera/animation phase remain fidelity comparisons. Static sound/voice validation and silent native rendering do not establish an audible side-by-side comparison.
