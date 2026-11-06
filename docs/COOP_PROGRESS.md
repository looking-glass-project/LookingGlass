# Co-op progress

Updated 6 November 2026. Branch: `coop`. **Estimated overall completion: 57.8%.**
The older 42% reports are historical. This percentage measures delivered and
verified behavior against the full co-op scope; it is not a running test's progress.

The focused save/rejoin milestone passes on frozen `0b5bca3`, including
adding player 2 to a solo save and resuming an earlier co-op save. All thirteen
selected core groups have public-invite passes, covering personal resources,
fresh processes, rejoining, backup recovery, shared wipes and unchanged solo
saves. Both requested save paths are included in the
[core acceptance checklist](COOP_CORE_TESTS.md#current-saverejoin-acceptance).
The overall all-level goal remains active. Majestic Maze now has its missing
shared plate/gate and escort integration, with native component evidence on
`c6f580e`. Both Jabberwock encounters now have shared combat and personal healing
on `7c3ca24`, verified by 58 staged component cases. Build `ccd6cd1` adds their
paired scenes, personal Eye Staff reward, shared bridge and exit handling, with
55 further world/scene cases, including both post-battle walking orders across
the bridge and through the actual castle exit. Nine other late controllers
still lack explicit co-op hooks. These are implementation gaps
before full traversal testing. Campaign-specific
verification, including the unfinished Burning Curiosity route, stays separate.

The fresh pre-change checkpoint run on 5 November passed staged shared travel,
guest reconnect, complete primary-save restoration and team-wipe restoration.
The backup attempt restored the world before simulation but its guest could
not resolve the new public invite hostname; that attempt is a failed internet
check, not a backup/network pass. Solo-save continuation passes native
conversion and all four paired internet phases on build `7994cc2`. That earlier
foundation point credited the missing import feature. The current additional
Internet Host/Join point credits completed public core consolidation on `0b5bca3`;
no additional campaign visit is credited.

Six of the 39 campaign visits have combined verification evidence; 33 remain open.
Earlier contact-system repeats are still required. Remaining work includes full
paired traversal, both scene owners, deaths and checkpoint recovery, reconnects,
shared transports, swimming/leaves and single-player regressions across the campaign.

| Area | Credited points | Maximum |
| --- | ---: | ---: |
| Foundation and independent client diagnostics | 9.5 | 10 |
| Internet Host/Join | 10.75 | 15 |
| Two players, movement and presentation | 11 | 15 |
| Combat, individual resources and death | 11 | 20 |
| Scenes, transport and progression | 12.5 | 20 |
| Full campaign verification | 3.08 | 20 |
| **Total** | **57.83, rounded to 57.8** | **100** |

## Implemented features and remaining work

The percentage is a conservative, manually weighted readiness estimate, not
the fraction of code written or the current position in the story. Burning
Curiosity is visit 26 of 39, but the work consists of separate level and
checkpoint tests. Earlier visits still have outstanding acceptance cases.

The working features below have evidence in covered scenarios; they are not
yet verified across every level and failure condition.

| Area | Implemented and demonstrated | Remaining work |
| --- | --- | --- |
| One-PC setup | `coop` branch; independent game processes, profiles/saves and automated state/log checks | Finish reproducible campaign coverage and repeat it on the final build |
| Host / Join | Main/ESC controls, public internet menu flow, authenticated transport, prompt handshake cancellation, failure/retry recovery and countdown/cutscene/moving-lift/loading-barrier reconnect cases | Other transport/recovery cases and public provisioning/DNS availability |
| Players and presentation | Blue/red Alice, separate input/cameras, body separation, replicated weapon effects | Broader contact/effect checks and reliable traversal through all swimming, leaves, ropes and moving supports |
| Combat and resources | Nearest-visible targeting, individual damage, inventory and consumables in integrated levels | Remaining enemies/bosses and level-specific rewards, with both players as the target/collector |
| Death and saves | Exact ten-second countdown, moving spectator camera, safe 25-health revival; shared checkpoint wipes, public save/leave/continue and storage-error/backup menu flow, public save refusal/retry during deaths/scenes, public guest restart during either player's countdown | Each visit's recovery matrix and remaining transport/death combinations |
| Scenes and machinery | First-arrival ownership, original single-Alice scenes, hidden partner and safe reunion; public intro/dialogue reconnects with either theatre owner; all-living boarding on integrated rides | Remaining level adapters, both scene owners, watched/skipped cases, transports and exits |
| Regression checks | 940 active Rust tests; 450 Python guards; Maze and Jabberwock paired components and exact solo route logs, plus build-specific weapon-effect/audio and Curiosity baseline checks | Full campaign verification and required earlier contact-system repeats on the final build |

Campaign status from `COOP_CAMPAIGN.md`:

- **6 visits have combined verification evidence:** Dementia/opening village,
  Pandemonium, Fortress of Doors (first visit), Beyond the Wall, Fortress of
  Doors (return visit), and Skool Daze (return visit). The first five retain
  historical build-specific passes and still require contact-system repeats.
- **24 visits have partial evidence.** Some already have complete internet
  traversals, including Wholly Morel Ground, Castling and About Face, but still
  have outstanding reliability, recovery, reconnect or scene cases. Others
  currently have components or checkpoint segments only.
- **9 visits are marked Not run for level-specific co-op verification:**
  Airborne Terror, Mystifying Madness,
  Water Logged, Labyrinthine Revenge, Machinations, Battle Royale,
  Ascension, Castle Keep and Heart of Darkness. Broad entrance checks cover
  all 39 visits; they do not complete these nine level checks.

See the [full campaign matrix](COOP_CAMPAIGN.md). Future updates should separate
working features, partial route milestones and completed verification visits.

### Jabberwock combat and healing checkpoint

The follow-up on `ccd6cd1` passes 21 world/reward cases and 34 Stage/scene cases:
both original introductions and finales, both scene owners, watched/skipped,
pause/restoration, frozen partners, dead partners and safe reunion. Each Alice
collects their own Eye Staff and ordinary personal Will refill; departure waits
for both collections and retries automatically after a failed transition.
The drawbridge begins after reunion, carries living riders atomically and stops
if either body is obstructed. Both ordinary-input post-battle crossing orders
reach the castle's real exit trigger. The 58 combat/healing cases also pass;
all four original solo check/route logs exactly match `7c3ca24`.
This adds 0.5 scene/progression points, with no full campaign verification points.
The remaining scope includes full two-client combat-to-exit routes, rendered
effects and the encounter-specific death/wipe/rejoin matrix. Core integration
continues with the nine controllers listed above.

Build `7c3ca24` adds one shared combat clock, closest-visible living targets,
stable attack ownership across saves, physical damage to either body and
individual eight-second healing cycles in both encounters. Fifty-eight native
cases cover both slots, cover, dead/hidden players, six Lair wave targets,
30/60/144 Hz solo combat parity, Pocket Watch rules, compressed continuation,
solo-save conversion and invalid-save rejection. Both existing solo checks and
both production-input solo routes have byte-identical logs to frozen `c6f580e`.
This credits 0.5 combat/resource points. It adds no full campaign verification
points and no two-client route, rejoin, death/recovery or rendered-effects pass.
The scene, reward and exit hooks are covered by the follow-up described above.
On `7c3ca24`, the full Rust suite passes with four test workers. Its default-concurrency run
hit the previously observed 750 ms socket-cancellation timeout; that unchanged
test also passes alone. Both receipts are retained, and the intermittent
cancellation deadline remains a test-reliability issue to investigate.

On `ccd6cd1`, the same cancellation test fails both in the four-worker suite
(939 passed, one failed) and in isolation. A separate instrumented reproduction
observes Windows rejecting shutdown on the duplicated connecting socket with
`WSAENOTCONN`, while the worker remains Connecting. This is now tracked as a
connection cancellation bug; the Jabberwock component and solo checks all pass.

## Core readiness priority

The focused checklist is now consolidated on `0b5bca3`, including public
internet runs. These checks remain separate from the 39-visit campaign matrix.

- Verified on the initial import build: existing co-op primary resume and team
  wipe; loaded solo progress converted into two players; manual co-op saves;
  fresh host/guest restarts; guest rejoining imported, resumed and backup sessions;
  unchanged solo saves; cached first/return visits; host-owned guest resources.
- Also verified on prior build `a9b55a5`: swimming import and the complete
  import/rejoin/resume/backup internet suite. The separate team-wipe persistence
  suite passed travel/reconnect and primary resume/wipe on the same build; its
  backup guest hit the DNS failure again, so that whole internet run did not pass.
- Verified on build `7994cc2`: real Windows primary/backup replacement failures
  preserve both committed files and the previous in-memory team-wipe state;
  stale temporary files are ignored; a successful save updates the wipe point;
  saving after primary corruption preserves the last good backup. Abrupt host
  loss leaves the save intact, ends the diagnostic guest connection, and permits
  fresh host/guest resume and rejoin over the internet.
- Verified through real menu input on loopback: load solo, Host this game, Join,
  Save checkpoint, Leave, exact suspended-solo return, ordinary solo movement,
  expired-link feedback and Continue co-op with a fresh invite. Two solo returns
  preserve the full state hash. Invalid/batched input and Cancel also pass.
- Verified on build `0b32c79`, with delayed loopback clients: a fresh guest
  reconnects during either player's death countdown. The waiting host holds its
  world/life clocks; personal resources, body identity and remaining time survive.
  Both cases continue through exact ten-second revival and a shared checkpoint wipe.
- Verified on build `4b3760a`: the complete real-input menu flow over public
  internet invites, including two exact suspended-solo returns and a fresh
  invite for Continue co-op. The provider responded after an hour without retries.
- Also verified on `4b3760a`: guest process restart during the theatre introduction
  and dialogue, with either player owning the watched scene. All four loopback
  cases preserve clocks, ownership, starting resources and the hidden partner,
  then complete the original scene and safely reunite both clients.
- Verified on `4b3760a` through public invites: guest restart during the rotating
  spiral lift, with either player boarding first. The lift waits for its second
  rider, holds its full world/body state during disconnection and completes the
  ride after rejoining with both attachments and starting resources preserved.
- Verified on `9921a3e`: guest restart while the host waits at the normal loading
  barrier, both outbound and returning to a cached school visit. Both cases pass
  over loopback and public invites with 40/20 ms delay/jitter, retaining personal
  resources, visit history and cached world state. Native socket tests cover four
  interrupted loading stages, including either client's readiness boundary.
- Verified on `0c5e717`: real-menu recovery from DNS failure, HTTP rejection,
  cancelled Join and failed/cancelled Host setup, followed by a successful public
  join, checkpoint save, leave and exact solo return in the same game processes.
  Cancel now closes an established but stalled TLS/WebSocket handshake socket.
- Also verified on `0c5e717` through public invites: both death-countdown guest
  restarts and all four theatre owner/introduction-or-dialogue reconnect cases,
  each with 40/20 ms delay/jitter. The same frozen build completes exact revival,
  shared checkpoint recovery, watched scenes and safe reunions on both clients.
- Verified on `c5cb7fc`: real-menu primary/backup write failures, successful retry,
  fresh-process backup continuation/repair and both-corrupt feedback. Loopback and
  public-invite runs preserve all three suspended solo returns and the solo slot.
- Also verified on `c5cb7fc`: normal-menu save refusal during either player's
  countdown and both theatre owners' introduction/dialogue, followed by successful
  safe retries. All three public cases use the same final fixture and 40/20 ms
  delay/jitter; timers, cameras, original scenes and saved files remain correct.
- Verified on `c5cb7fc`: public guest restart during either player's underwater
  death countdown in the Blue-led recovery route, with 40/20 ms delay/jitter.
  Exact first-frame body/air/world/enemy agreement, both revivals and shared wipe
  pass. The Red-led attempt wipes near the first column before reconnecting;
  it remains failed route evidence, not a second passing order.
- Verified on `c5cb7fc`: public guest restart during either death countdown
  beside a rope survivor, followed by both exact ten-second, 25-health rope
  revivals. The survivor uses ordinary climbing and attacks against live enemies;
  no health, bodies or enemy state are edited.
- Verified on `c5cb7fc`: public guest restart during either death countdown
  beside a survivor on Funhouse's tilting platform. Both exact ten-second,
  25-health revivals occur while the brush moves; neutral revived bodies remain
  supported and are carried with it. First-frame player/world/enemy hashes agree.
- Earlier consolidation on `c5cb7fc` passed both save families, fresh-process
  resume/rejoin, host loss, backup and team-wipe restoration, plus the normal
  public menu and failure/cancel/retry flows, plus player contact/ambient effects.
  Its next network case failed invite DNS before gameplay, leaving network,
  scene and lift repeats open at that point. Those focused repeats now pass on
  `0b5bca3`; see the current consolidation checkpoint below.
  External DNS/provisioning availability remains a limitation despite verified
  error/cancel/retry handling.

### Core acceptance checkpoint

This separates completed feature checks from the remaining core work. A passed
row does not certify all 39 campaign visits or erase the build-specific limits
in the evidence below.

| Core area | Current evidence | Next required check |
| --- | --- | --- |
| Existing saves | Solo import and co-op resume, individual resources, cached visits, fresh processes, unchanged solo files, normal-menu refusal/retry while dead or in a scene; fixed-build import/resume/backup/host-loss/wipe repeat passes | Core save cases covered; each campaign visit still needs its recovery matrix |
| Storage failures | Real locked-file failures, retry, corrupt-primary backup recovery/repair, both-corrupt feedback; public menu tests | Covered for these cases; electrical power loss and disk exhaustion are not simulated |
| Host/Join lifecycle | Public main/ESC flow, authenticated invite preflight, fresh invites, invalid/expired links, DNS/HTTP failure, Cancel, retry, exact solo return on `0b5bca3` | Public provisioning can still rate-limit new sessions |
| Rejoining | Public passes on the frozen core builds for idle gameplay, either player's ground/underwater/rope/moving-platform countdown, either theatre owner, rotating lift and both loading barriers | Per-visit matrix; underwater proof uses the Blue-led route and does not close the Red-led trap-repeatability gap |
| Combat/death | Shared targets, personal resources, exact countdown, spectator camera, 25-health reunion, shared wipe and moving-platform revival/carry; focused current-build public consolidation passes | Level-specific enemies and recovery matrices |
| Presentation | Both Alice models, contact separation and replicated ambient/weapon effects; `66ebd44` repeats weapon rendering/network checks, verifies survivor-camera audio inputs, shared-mixer output and actual two-client puzzle/dialogue audio | Campaign-specific effects/audio; subjective sound quality is not certified by these automated checks |
| Consolidation | `0b5bca3`: 934 native tests, both public menu flows, eight public network cases, and all thirteen selected core groups pass over public invites and locally | Full campaign coverage; provider rate limits remain an external service limitation |

## Maze core integration, 6 November 2026

Source inspection found three implementation gaps: Maze machinery inherited an
empty co-op update, the Arena did not supply native NPC contacts to movers, and
the escort child only sampled the first living player. Frozen `c6f580e` fixes
these together. Either living player can press the plate and trigger the escort.
The child follows the closest visible living Alice, retaining only that actor's
observed path; changing targets clears the previous path. The selected slot is
saved and replicated, with an optional field for older saves.

The plate/gates advance once per shared tick. Both living riders move only after
all destinations are clear; a closing gate waits for either player. Dead bodies
neither press nor ride the plate. The child's own Hold still controls departure,
so two Alices cannot bypass the escort. A pending failed exit retries once from
the shared update without requiring an E press. Solo play uses the same mover
implementation with one body.

All twelve supplied-map component cases and seven production-cast cases pass:
both player slots at 30/60/144 Hz, solo clock parity, pause, snapshot continuation,
blocked/released carry, dead-body exclusion, both gate blockers, actual Arena
NPC contacts before/after restore, both native escort triggers and one-shot
automatic exit retry. Two new asset-free escort regressions also pass. Broader
shared-world and all 39 entrance/four underwater checks pass on this executable.

The full ordinary solo route reaches tower1 in 28,351 simulation ticks at
95.2152 health and 99.2 Will. All eleven JSON checkpoints/reports, including the
destination arrival, exactly match a fresh `66ebd44` baseline; the route logs
also have identical hashes. All 940 active Rust tests pass (five ignored), the
locked release builds, and 450 Python guards/seven package tests pass. The first
unit run retained a registration-list failure and a transient socket-cancel
timeout; after fixing the registration list, the unchanged network test passes
alone and in the final full suite. No timing assertion was relaxed.

Evidence: `private/coop-maze-build.json`, `private/coop-maze-components.log`,
`private/coop-maze-native-c6f580e/`, `private/coop-maze-solo-before/` and
`private/coop-maze-solo-comparison.json`. Anode is released and owned windows
are closed. No screenshots were inspected. This adds **0.25 progression points**
for a working core adapter, bringing the estimate to **56.8%**. Maze becomes
Partial; there is no new complete campaign visit. Its full two-client route,
combat/pickups and death/wipe/rejoin matrix remain unverified. The next priority
is the [remaining controller integration](COOP_CORE_TESTS.md#remaining-controller-integration).

## Core internet consolidation completed, 5 November 2026

After nearly two hours without a provisioning request, public hosting became
available again. All thirteen focused core groups now have passing internet
evidence on the byte-identical `0b5bca3` executable. The initial cohort
`private/coop/core-suite-2f95fa5c8bb94289ab303a2848908566/summary.json` passed
ten groups and failed the swimming fixture's final team-wipe setup. Its failure
is retained. The later cohort
`private/coop/core-suite-b09c7ef47ce043a1a6a6b603aef07bc2/summary.json` passes
swimming, rope and tilting-platform recovery. All recorded phase/result hashes,
network scopes and native identities were checked when combining these results.

The passing coverage includes solo import, primary/backup resume, host loss,
normal menu recovery, locked-file failures, save refusal during deaths/scenes,
both lift boarding orders, loading transitions, underwater/rope/platform
countdown rejoins, exact revivals and shared checkpoint restoration. Both
clients use separate processes and profiles; diagnostic cases use 40/20 ms
delay/jitter, while storage and host-loss menus use ordinary connections.

The swimming failure was a test setup issue: stopping both players immediately
after individual revivals left their air supplies far apart. They alternated
deaths and revivals without ever being dead together. A later attempt to refill
after those revivals also failed when a player drowned before reaching the
guide. The fixture now checks fresh air from ordinary gameplay and performs
the shared wipe first, followed by entrance rejoin and both individual recovery
roles. No health, air, position, death rule or native code was changed.
The countdown verifier now accepts an explicit expected checkpoint generation,
preserving its default zero and rejecting unexpected generation changes.
The intermediate hard-coded-zero assertion failure remains recorded too.

The final fixture passes locally in 121.19 seconds and over public invites in
141.91 seconds. The two remaining public groups pass in 51.34 seconds (rope)
and 83.94 seconds (tilting platform). All 427 Python guards and seven package
tests pass. The earlier 934-test Rust result still applies to the unchanged
native build. No screenshots were inspected. Anode was released after cleanup.

This completes the focused core milestone, not the full game. Campaign status
remains six combined, twenty-one partial and twelve not run. The known Red-led
Temple route gap, broader effects/audio and per-visit acceptance cases remain.
The weighted estimate rises one Internet Host/Join point to **56.6%**.

## Live two-client audio, 6 November 2026

The remaining focused audible co-op check now passes on frozen `66ebd44`.
Two public-invite puzzle runs cover Blue-first and Red-first demonstrations,
both reset-lever operators and a fresh guest rejoin in 71.97/70.95 seconds.
Each client receives the same three notes once per sequence; the fresh guest
does not replay the completed demonstration. A separate 83.47-second public
opening-scene run watches all six dialogue lines once, in order, on both clients.
All three cases use independent processes/profiles and 40/20 ms delay/jitter.

These runs use the PC's output endpoint because Anode has no audio endpoint.
All eight app-only recordings contain real 44.1 kHz stereo mixer output.
Private cross-correlation against the locally supplied original sounds finds
the three ordered note onsets in both initial-client captures and in both
fresh-guest captures (scores 0.769-0.833), and the first Rabbit voice on both
clients (0.964/0.975). Capture length is capped at 30 seconds: later reset
sequences and the other five voice lines are runtime cue/clock evidence, not
full acoustic comparisons. No microphone/system audio or model image lookups
are used. This is bounded output verification, not subjective sound fidelity.

Two earlier attempts remain failed evidence. The original audio fixture sent
both bodies to one point and stalled against correct player separation. The
first separated-goal revision disabled defensive attacks and a player died.
The final fixture retains ordinary combat and waits for both distinct, safe
arrivals before using the original lever. No native physics, health or enemy
rules changed. New guards reject unsafe/missing arrivals and partner overlap.
The voice verifier now requires all six opening lines rather than a three-line
minimum; all 308 saved observations per client pass the stricter verifier.

Evidence: `private/coop-live-audio-combat-manifest.json` and
`private/coop-live-audio-verified.json`. All 33 recorded artifact hashes, frozen
fixture hashes and executable identity were independently rechecked. The final
validators are recorded separately from the unmodified live receipts. All 450
Python guards and seven package tests pass; the prior 938 active Rust tests and
locked release result apply to the unchanged native source. Owned test windows
are closed. Both save families remain covered by the separate thirteen-group
`0b5bca3` milestone. Overall remains **56.6%**, with no campaign visit credited.

## Spectator audio and weapon effects, 6 November 2026

On frozen build `66ebd44`, co-op audio now uses the same player as the camera
for health reactions, swimming ambience and the underwater filter. Previously,
a dead player's submerged body could keep the dry survivor's view muffled.
Changing the subject clears only personal reaction/swim state, so switching
between different health values cannot invent a pain cry. Shared world sounds
and dialogue continue. Native tests cover either player dying, both dry/wet
survivor combinations and return to the revived player.

The public-invite recovery fixture passes in 140.86 seconds with 40/20 ms
delay/jitter. It exercises an ordinary shared wipe, both real drowning roles,
both fresh-guest countdown rejoins and exact ten-second revivals. The listener
matches the camera in 550 host and 545 guest observations, including 51/48
spectator observations. The guest runs in four fresh processes. An independent
audit also checks each recorded first rejoined frame. The final stricter verifier
requires revival after the observed death in the same checkpoint generation;
the saved observations pass with 3/191 post-revival rows. Earlier shared-wipe
spawns cannot substitute for that coverage.

The live fixture's two swimmers remain submerged, so the mixed dry/wet cases
are native-test evidence. Anode has no isolated audio endpoint: this run checks
the actual mixer inputs and sets `speaker_output_verified: false`; it does not
claim a new audible-output or sound-fidelity pass.

Separate audio-only regressions then pass on the PC's output device, using the
same frozen executable and isolated files, without opening game windows. They
measure fourteen gameplay intervals and seven liquid/reaction intervals in
22.73/18.66 seconds. App-only 44.1 kHz stereo captures contain 22.35/18.25 seconds
of output. Mute and pause are silent; expected audible phases have nonzero
output without clipping, and no requested audio files are missing. These are
production shared-mixer regressions, not an audible two-client co-op run or a
subjective sound-quality assessment. Evidence is
`private/coop-listener-audio-output-66ebd44/`.

On the same executable, all six native effect pixel comparisons pass: sparks,
swing trails and projectile trails for both skins remain visible and match
across three repeated restores each. A separate 26.34-second public combat
check receives all three effect types and both emitting players on both clients,
with 63 matching player/world/native states. These automated checks inspect
rendered pixels and network state without model image lookups.

Evidence: `private/coop/audio-listener-3a0fff842e8146808293fb8b7562b522/`,
`private/coop/weapon-effects-261091c7703548b98832633b1a17998c/` and
`private/coop-listener-verified.json`. Original live receipts and fixture copies
remain unchanged; the audit records the tightened validator's hash separately.
All 938 active Rust tests pass (five ignored), the locked release builds, 443
Python guards and seven package tests pass. Owned windows are closed and Anode
is released. The thirteen-group save/rejoin milestone remains on its separate
`0b5bca3` build. Overall remains **56.6%**; no full campaign visit is credited.

## Save coverage audit and bounded ending evidence, 6 November 2026

The thirteen public core groups and all nineteen phase-result hashes remain
intact on frozen `0b5bca3`, including both solo-to-co-op import and previous
co-op resume/rejoin. The frozen executable hash also matches. No core test was
rerun or additional readiness point credited by this evidence audit.

The already-running Curiosity ending attempt on `ddaf73b` completes Blue's
watched Caterpillar scene, safe reunion and paired points 77-82. Its independent
bounded audit verifies twelve personal arrivals and 5,219 matching
player/world/native states. The complete attempt fails at point 83 when Blue
does not settle on an intermediate ledge within the navigation timeout.
Watch activation, the later personal Meta-essence, final climb and exit remain
unverified by this live fixture. The earlier first attempt also remains failed;
its boundary assertion incorrectly rejected normal post-scene regeneration and
loot aging. The corrected verifier permits only changes explained by the
authoritative reunion tick and normal fixed-step rules.

The new ending fixture and opt-in ordinary Watch input have 936 passing native
tests (five ignored), a locked release build, 439 Python guards and seven
package tests. Four native comparison logs preserve the existing baseline,
including the known cast-free point-54 failure. No screenshots were inspected;
owned game windows are closed and Anode is released. Full campaign status and
the **56.6%** estimate are unchanged. See [ending evidence](JLAIR1.md#resumed-ending-fixture-6-november-2026).

## Remote approach directions, 6 November 2026

The opt-in remote route search now retains the direction used to arrive at a
search cell. Its delayed-turn safety check continues that preceding input;
merging arrivals from different directions discarded potentially safe paths.
The native reproduction failed after 100,002 states from the settled guest
position, only 1.56 units from an earlier successful start. Keeping the incoming
direction finds a route from both positions and the exact later live failure.
The search limit, twelve-tick takeoff reserve, eight-unit first-landing support
and normal player physics remain unchanged. Zero-margin native routes retain
their previous state equivalence.

The optional native `remote_curiosity_ascent_retains_approach_directions` test
replays each selected input through actual movement/collision physics at all
three recorded starts. It requires actual jumps, clear bodies, dry travel,
grounded arrivals within 22 units, supported first contacts, and the full late
input interval at grounded turns/takeoffs. The test passes explicitly on frozen source `c893e48`; all
935 active Rust tests pass (five ignored), the locked release builds, 427 Python
guards and seven package tests pass. All four Curiosity native comparison logs
match their baselines, including the known cast-free point-54 failure. That
failed comparison remains a failure, not a completed traversal.

The paired fixture retains all four original ascent guides and adds an observed
higher ledge before the unchanged original point 65. On the earlier `1a6b3ea`
runtime, both players reached all five guides and Blue reached point 65, but
Red's final search failed. Its bounded independent proof verifies ten guide
arrivals, 1,845 matching player/world/native states and 108 recovery rows without
revivals. That attempt remains failed and adds no paired point or visit credit.

The current `c893e48` public-invite run completes the resumed **65-76** segment,
with both players reaching every original goal: 24 personal arrivals and all
ten guidance arrivals. Independent processes/profiles use 40 ms delay and
20 ms jitter. The frozen-fixture audit rechecks exact checkpoint restoration,
ordered arrivals, observed waiting bays, player/camera presence, saved checkpoint
hashes, 7,419 matching player/world/native-enemy states and 431 recovery rows.
Both finish alive at 92/77.67 health without revival. Ordinary checkpoints
before 65, 66, 70, 74, 76 and 77 are retained.

Evidence is `private/coop/jlair1-route-a6b6947a160e4842bf28b4bcfd91fd74/`, including
`verified-direction-segment.json`. The formerly stalled paired point 65 is now
verified. This is one completed segment, explicitly `full_route: false`; a fresh
continuous repeat, the ending, both scene owners, recovery/rejoin combinations
and complete campaign consolidation remain open. No state or checkpoint bytes
were edited to progress. No screenshots were inspected; both test clients
exited normally, owned windows are closed, and Anode is released.
Overall remains 56.6%, with six combined, 21 partial and 12 unrun visits.

## Automated takeoff timing, 6 November 2026

The opt-in route controller now consumes elapsed guest frame time when a
movement step expires. Its old 100 ms cap could hold a run-up too long while
the host continued simulating, issuing the next jump after the player had
already left the ledge. The new clock advances once per sampled frame and
gives the next input its full interval. A native Curiosity regression models
a 300 ms guest stall: the old clock jumps too late and records zero actual
jumps; the corrected clock records one grounded takeoff. This reproduces a
failure mode, not the exact timing of the earlier live capture.

Frozen source `1a6b3ea` passes 935 active Rust tests (four ignored), the explicit
asset-dependent takeoff regression, locked release, 427 Python guards and
seven package checks. Its Curiosity native comparison logs match their prior
baselines, including the known cast-free point-54 failure. Only the automated
route driver changes; normal player input, physics, networking and saves do not.

The original four-guide public fixture now records both lower approaches and
all eight personal guide arrivals, plus Blue's original upper-shelf arrival.
A bounded audit verifies 1,709 matching player/world/native states and no
revivals through the last guide receipt. The complete run still fails: Red's
final ascent exhausts 100,001 search states. A separate final-guide adjustment
also fails after the eight guide receipts (992 matching states in its bounded
audit); that adjustment has been reverted. Both failed runs remain recorded.

The focused save/rejoin milestone remains complete on `0b5bca3`; these campaign
tests do not replace or expand that build-specific evidence. No paired point-65
or completed campaign visit is credited. Overall stays **56.6%**. See the
[Curiosity evidence](JLAIR1.md#automated-takeoff-clock-6-november-2026).

## Campaign work resumed, 5 November 2026

With the focused core gate complete, Burning Curiosity testing resumes on
unchanged build `0b5bca3`. Its 26 paired native cases plus summary, seven solo
components and full native combat-route output match the earlier baselines
byte for byte. The separate cast-free route retains its known step-54 failure.
A first long native comparison ended with Windows `AppHangB1`; the isolated
Anode repeat completed and matched. The failed attempt remains recorded.

The resumed internet climb still fails at point 65. Blue reaches all four
guidance points and the upper shelf; Red reaches the first guide, then misses
the next landing and falls. A private crossing prototype verifies Blue's real
run-up, jump and landing, but the guest disconnects before attempting it. The
transport log does not establish the disconnection's cause. A delayed local
attempt then fails on Red's earlier approach: the host records the jump input
only after she is already airborne below the ledge, so no actual jump occurs.
Neither prototype is promoted to the checked-in route fixture or counted as a
paired climb pass. See `JLAIR1.md` for evidence and the next controller issue.

No game code, player physics or save behavior changed in this investigation.
The previously deferred supported-landing fix remains in the current build,
but these failures happen before its target ascent can be verified for Red.
Campaign counts and the **56.6%** overall estimate remain unchanged.

## Archived solo save import, 5 November 2026

The original Village solo autosave from the 9 October movement-fix work now
passes the full import suite on unchanged build `0b5bca3`. Before its first
guest connects, the runner compares Blue's complete body, resources, camera,
character, world interactions, story, hints and NPC state with the archived
file. No differences remain after accounting for equivalent JSON spellings
of native 32-bit floats. Red receives the same earned starting resources at a
separate safe position. The source file remains unchanged.

All four connected phases pass: import, fresh-process resume, resume after
abrupt host loss, and backup recovery. Each also restarts and rejoins the guest.
The final checked-in runner result is
`private/coop/core-suite-1902de13b88844d696b763337e1c3da2/summary.json`, case
`import-test-beaa7e9d6c5c48a3b24c6f5590fe02f9`, taking 33.44 seconds. This is
loopback evidence with 40/20 ms delay/jitter, not a new public-invite pass or
proof of every older save format or campaign visit.

The initial probe exposed a test teardown race: sending Finish to both clients
together could close the host before the guest read its own command, yielding
a normal host-ended error. The import runner now waits for a successful guest
exit before finishing the host, retaining failure checks on both processes.
The default generated school/swimming preparation and all four network phases
also pass with this ordering: `core-suite-e31f227be06442acaa20edd48fa9e279`,
case `import-test-7516ff8eb9aa42c7aab58aa12dbf4615`, taking 34.95 seconds.
The failed probe is retained separately and receives no passing-suite credit.

`--source-save` makes this reproducible with an isolated copy of an existing
quiet save; the core runner records and checks its hash. Five new guards reject
partial resets, stale observations, lost fields and altered values, and cover
the shutdown order. All 424 Python guards and seven source-package tests pass.
No native gameplay code changed. Overall completion remains **55.6%**.

## Abrupt host loss through the menus, 5 November 2026

`tools/test_coop_menu_host_loss.py` closes a gap between diagnostic host-loss
coverage and the normal player-facing menus. It passes on unchanged build
`0b5bca3` through the checked-in core runner, using two rendered processes,
native input and passive state receipts. This run is **loopback only**.

After loading independent solo saves, the host imports its game and writes a
primary checkpoint and backup through the menu. Terminating only that owned
host process tree produces the guest's visible disconnect explanation in
0.969 seconds. The surviving guest returns to its exact suspended solo state,
accepts ordinary movement, then enters Join again through ESC. A fresh host
process uses Continue co-op and creates a different invite; the same guest
process rejoins with saved resources, positions, used world interactions and
cached visit count preserved. Both players then return to their own solo
states. The two solo files, primary checkpoint and backup retain their bytes.

Evidence: `private/coop/core-suite-06283cf9f6d948608ab058b5323ba07d/summary.json`
and `private/coop/menu-test-b045e7cc7f74431eb05c4addb2f000de/`. The full check
took 29.75 seconds and inspected no screenshots. It is now selectable as
`host-loss-menu` in the thirteen-case core runner. Its public-invite repeat
remains open alongside the previous twelve core cases. No native files changed,
no campaign visit was added, and overall completion remains **55.6%**.

## Authenticated invite readiness, 5 November 2026

Build `0b5bca3` checks its prospective public invite before exposing it. After
helper registration and the existing startup grace, a background worker uses
the same DNS/TCP/TLS and authenticated endpoint as Join. Only the expected reply
to a fresh random challenge succeeds. The host remains waiting with no guest
slot consumed, generation change or gameplay started. The menu shows
"Checking your invite..." while this runs. Transient failures retry within a
60-second budget; timeout/helper failure returns an error, and Cancel closes
an in-progress handshake socket and releases the owned helper.

All **934 active Rust tests** pass (three ignored); the locked release, 410
Python guards and seven source-package tests pass. Added coverage uses real
loopback sockets for slot preservation, invalid authentication/protocol/path,
malformed and replayed challenges, ordinary upgrades, transient failure,
deadline/helper failure and cancellation during HTTP/TLS setup. The initial
malformed-path test accidentally used an HTTP rather than WebSocket scheme;
that fixture was corrected. A subsequent full run exposed a cancellation test
whose socket deadline began before Cancel was dispatched. It now synchronizes
that action, checks Cancel's own duration and separately requires prompt EOF.
The final full suite passes; earlier failed logs remain in private evidence.

Public tests on the byte-identical executable pass:

- `network-test-266666220b5748b7a853cdcf92648c5e`: independent processes, isolated
  profiles, 16 ordered exchanges, 40/20 ms delay/jitter and clean disconnect.
- `menu-test-e38c28f83d344935b8a0509c58743d5d`: real main/ESC solo import,
  save/leave, expired-link feedback, Continue co-op with a fresh invite and two
  exact solo returns (66.27 s).
- `menu-test-7e9a518372294142abc5c4995a780efe`: DNS/HTTP errors, cancelled Join,
  failed/cancelled Host, successful public retry in the same processes, save,
  leave and exact solo return (63.80 s).
- `host-stall-test-49c9bc9570904e1a8a0f1e82b2b0954d`: three controlled stalls,
  recovered input timing, contact separation, both models and ambient effects;
  680 matching states, zero settled input backlog and under 7 ms simulation
  debt with 40/20 ms delay/jitter (65.81 s).

All case folders are under `private/coop/`. The retained manifest is
`private/coop/invite-readiness-0b5bca3-20261105/manifest.json`. The executable SHA
is `32f9559ff0c7c3d38722eda021cfdddf81fb366298e54db43f0eb42f33b2d2df`.
The network cohort `regression-suite-643f10bb62464b169308cecb98e2f48c` passes
8/9 selected cases: host stalls, both ground countdown reconnects, all four
scene-owner/introduction-or-dialogue reconnects, and outbound loading-barrier
rejoin. The return-barrier case fails before gameplay in 1.25 s because the
invite provider limits new sessions; no new link is published. Its retained
folder is `travel-test-511a4bf7b477499da9779c898278a95b`. This is a failed public
suite, not a completed core consolidation. Remaining checks completed locally
without retrying the rate-limited provider; local results carry no internet
credit and older save/recovery passes keep their original build identities.

This adds 0.25 Internet Host/Join points for verified positive route readiness:
**55.58 total, reported as 55.6%**. Campaign counts remain 6 combined / 21 partial /
12 not run. The check proves reachability from the host at setup time; it cannot
guarantee another computer's DNS cache or continued provider availability.
[Cloudflare Quick Tunnels](https://developers.cloudflare.com/tunnel/get-started/quick-tunnels/)
remain a development service without an uptime guarantee.

## Reproducible core runner, 5 November 2026

The twelve save/recovery groups are now available through the checked-in
`tools/test_coop_core.py`; see [core test instructions](COOP_CORE_TESTS.md).
It preflights the desktop and required focus lease before starting games,
validates every expected phase receipt, rejects mismatched network scope,
reused evidence and mixed builds, and detects changed source/input/executable
files. It stops on the first failure without automatically retrying a provider
error. Optional saved SchoolPair modules preserve unrelated worktree edits.

Nine focused runner guards pass, bringing the Python guard total to **419**;
seven package checks pass. The final collector also validates all twelve
retained local cases plus one fresh local import case. That audit is receipt
validation, not twelve additional gameplay passes. The fresh end-to-end run is
`private/coop/core-suite-b159c71b2d3044ac859ef30c51ef1611/summary.json`, with
`import-test-f6eb3d468ec945f083115f1ac85fe962`: all four connected save phases,
guest rejoins, abrupt host loss and unchanged solo files pass in 34.53 seconds.
The later malformed-metadata checks were validated against its retained results;
the report keeps the runner hash actually used during execution.

One bounded public return-barrier retry after approximately twenty minutes
without new provisioning requests still fails before gameplay in 1.38 seconds:
`ready-core-suite-83fa108fb74d41cf8aaac3f9500a3c86/summary.json`, evidence
`travel-test-c3b2192d4e144945a2869077d0791ec5`. The service again reports session
rate limiting; the retained helper log gives no retry time. No further public
request was made in this turn. Earlier public passes remain valid, and the
remaining twelve public groups stay open. All native code and frozen executable
bytes remain `0b5bca3`; no game rebuild was needed. Anode is released and overall
completion stays **55.6%**.

## Frozen-build local core completion, 5 November 2026

The same `0b5bca3` executable completes all **12 selected local core cases** with
40/20 ms delay/jitter where supported. These use separate client processes and
profiles. They supplement the public checks above; they do not count as public
internet passes or full campaign verification.

| Local group | Verified result |
| --- | --- |
| Return loading barrier | Guest restart while returning to a cached visit; preserved history/resources and shared transition |
| Solo import and co-op resume | Four fresh connected phases: import, resume, resume after abrupt host loss and backup; guest rejoin in each; exact native restore and unchanged solo files |
| Persistent shared checkpoint | Cached travel/rejoin, fresh-process primary and backup restoration, exact team-wipe recovery and personal resources |
| Storage errors | Locked primary/backup save failures, successful retry, fresh-process backup continuation and repair, both-corrupt feedback and normal solo return |
| Save eligibility (three cases) | Real-menu refusal during either death countdown and both theatre owners' introduction/dialogue, with successful safe retries |
| Rotating lift (two cases) | Both boarding orders, guest restart during motion, exact held state, preserved rider attachments and completed ride |
| Swimming recovery | Both underwater countdown reconnects and exact revivals, moving spectator and saved team wipe in the Blue-led route |
| Rope and moving-platform recovery (two cases) | Both countdown reconnects and exact safe revivals; moving-platform passive carry after revival |

Private batch reports:
`ready-core-suite-bfd11efc07d54ebd810277b719b6df65/summary.json` retains the first
four passes and a failed menu-test launch: the caller omitted its active Anode
focus lease, so the fixture stopped before its first Save action. The caller
now checks and forwards that lease. `ready-core-suite-da1d0126c45d4404b8fe3904e879b9f9/summary.json`
completes only the eight remaining cases. Both retain their actual wrapper and
fixture hashes. The aggregate readiness manifest records all twelve case
folders and result hashes; no successful case was repeated to hide a failure.

The swim case covers both victims in the Blue-led recovery route. It does not
close the separately recorded Red-led trap-navigation repeatability gap.
Rope and fulcrum fixtures resume unchanged ordinary gameplay checkpoints;
these are representative recovery checks, not entrance-to-exit traversals.
The travel fixture now prints its evidence folder before launching Host so
future provisioning failures are discoverable automatically. This logging-only
change follows these runs; their original fixture identities are retained.

The next public work is exactly these twelve local cases on this frozen build.
The eight public network passes and both public menu passes need no duplicate
run absent another relevant change. Public rate limiting remains unresolved;
all owned windows are closed and Anode is released. Overall completion stays
**55.6%** and the full campaign goal remains active.

## Fixed-build core checkpoint, 5 November 2026

The consolidated save repeat passes on the byte-identical `c5cb7fc` executable,
through public invites with 40/20 ms delay/jitter. This explicitly covers both
user-requested starting points: a loaded solo save converted to co-op, and an
earlier co-op session resumed later.

- `import-test-25f44f88ee764fac9c4716bd7e652a3f`: import, resume, abrupt host loss,
  fresh resume after loss and corrupt-primary backup recovery. Every one of the
  four connected phases includes a fresh guest rejoin. Native pre-simulation
  restoration, individual inventory/health/Will, cached visit state and unchanged
  solo files pass. The host-loss guest detects disconnection in 6.08 seconds.
- `travel-test-8a21edb2af184e1d879ed389766f6fc9`: shared travel/cache/rejoin, primary
  save restoration after both processes exit, and backup restoration. Both
  resumed cases also pass exact shared team-wipe recovery, individual resources
  and solo-save isolation. The combined save job exits zero in 233.89 seconds.
- `menu-test-0adb79d7780c41d2a1fb87bf4c1a426c`: real public main/ESC Host/Join,
  manual checkpoint, leave, expired-link feedback, Continue co-op with a new
  invite, normal solo movement and both exact suspended-solo returns (70.77 s).
- `menu-test-2924520df8824e9299d1dc5dcf8273b1`: real DNS/HTTP failures, Join and
  Host cancellation, failed hosting and subsequent public success in the same
  running processes, followed by save/leave and exact solo return (64.11 s).

All folders above are under `private/coop/`. The consolidated manifest is
`private/coop/core-consolidation-c5cb7fc-20261105/manifest.json`. It retains hashes,
receipts and failed attempts. The first menu repeat, `menu-test-91b25e76465a429b866877a4973919cf`,
passed its first co-op save/leave but opened Co-op when the test intended Resume.
It remains failed evidence. The harness now focuses before reading button
coordinates, checks the actual pointer and owned window before clicking, and
records every click. The subsequent two public menu cases pass with these checks.
The isolated window probe reproduced a successful Resume; it does not establish
the exact cause of the earlier mistargeted input. No game behavior changed.

Existing `c5cb7fc` storage-error, menu save-eligibility, underwater, rope and
fulcrum passes retain their own fixture identities; they need no duplicate run
just to share a report. Their build/source identities were audited at consolidation
start. The later menu input checks refine automation without changing save or
recovery assertions.

The network group `regression-suite-a0a95a8d4a564e028bbabd17f3ae1959` is **failed**,
not a completed core pass. Its first case, `presence-test-4233e0afbace44678f2fb6482c989e1a`,
passes contact separation, stationary-partner stability, both visible Alice
models and ambient particle/effect rendering on both clients (55.91 s). The next
case, `host-stall-test-e61fa43063ac4a15a80af365fcbd6187`, fails after 45.39 s because
the guest cannot resolve the new public invite address. It never reaches gameplay
or suspends the host. The remaining eight selected cases were not run, and both
lift orders remain separate work. The provider failure has not been rerun away.

Hosting currently waits for tunnel registration plus a 12-second grace period;
Join has bounded DNS retries. Host setup does not yet positively verify its
public address resolves before publishing the invite. That is the next concrete
readiness investigation; extending a fixed delay alone would not establish
service availability. All owned clients are closed and Anode is released.
This checkpoint adds no campaign visit or percentage point: overall completion
stays **55.3%**, with core consolidation incomplete.

## Moving-platform countdown recovery, 5 November 2026

`tools/test_coop_fulcrum_countdown.py` resumes an unchanged normal Funhouse save
with both players on the second fulcrum. Each victim runs off the side through
ordinary input. The survivor walks along the board to change its load. A fresh
guest rejoins during each countdown, then each revived player stays neutral
while the moving brush carries it. No native rules, bodies, health, enemy state
or saved payloads were changed. The native executable remains `c5cb7fc` with SHA
`07885bd0d8eb477aefe3d751d25f8e9fe0cf9f5f8819a337ba78f3ad87627f97`.

The final public case uses 40 ms delay and 20 ms jitter and passes in 60.03 seconds:
`private/coop/regression-suite-ac593d2aed8b4a609042bc88e74a4267/summary.json`;
case `private/coop/fulcrum-countdown-388308598d6f479083fea2574959ea6b/`.
All six retained source hashes match the final fixture. The unchanged checkpoint
SHA is `a61e0db3cbabc7347a0b3dd2fdd5826d29fbeffaf3de764b75b74f4cb8a628bd`.

| Public victim | Death | Revival | Rejoin ticks | Remaining countdown | Passive carry |
| --- | ---: | ---: | --- | --- | --- |
| Red | 88029 | 89229 | 88235 to 88247 | 994 to 982 | 17.72 units over 131 ticks; 7.11 degrees |
| Blue | 89899 | 91099 | 90262 to 90263 | 837 to 836 | 33.92 units over 120 ticks; 13.95 degrees |

The closed host trace has 2,415 records, both exact 1,200-tick/25-health revivals,
no unsafe-spawn delay, preserved personal resources and clear, safe, valid
supported placement. Observations bracket each spawn while the brush is rotating
and identify both bodies on that same native support. Both cameras, separation,
neutral-body carry, held disconnected world and exact first-frame agreement pass,
with 426 matching player/world/native-enemy states.

The initial local case `fulcrum-countdown-17dd7c4eb1c2415bb0e5ec478c4ca1b2`
passes in 40.74 seconds. The first public attempt
`fulcrum-countdown-4bd8c53398ab4419bb9e9e9436f03318` fails: unpredicted guest
steering overshoots a goal, causing an extra death. That failed evidence remains.
The final fixture uses the same measured-RTT steering lead as the existing
Funhouse route and fails immediately if its intended survivor dies. The public
pass follows that concrete fixture correction; no gameplay behavior was weakened.
The earlier spiral-lift probe stopped because a living player on the landing
correctly prevents shared departure; it supplies no moving-support recovery proof.

Four new guard tests reject stationary/wrong supports, driven or unsupported
carry, stale reconnects and advancing held worlds. All 410 Python guards and
seven package tests pass; the source worktree audit passes. No screenshots were
inspected. All owned clients close and Anode is released after verification.
This adds 0.25 combat/recovery and 0.25 transport point, giving **55.3%** overall.
The campaign matrix stays at 6 combined, 21 partial and 12 not run. The remaining
core milestone is the consolidated fixed-build run, beginning with solo import,
existing co-op saves, fresh restarts, backup recovery and rejoining.

## Rope countdown reconnect and revival, 5 November 2026

`tools/test_coop_rope_countdown.py` passes on unchanged build `c5cb7fc`, locally
and through a public invite with 40 ms delay and 20 ms jitter. It resumes the
unchanged Garden capture with Red falling and Blue attached to rope 353. After
Red returns, it exercises Blue's death while Red remains on the rope. The
survivor uses ordinary climb/weapon controls against the still-active rifle ant.
Only the owned guest process is terminated and restarted; bodies, health,
resources and enemies remain under normal simulation.

Both countdown reconnects preserve the held world, dead body, survivor's rope
ID/grip, personal resources and remaining time. The guest's first playing frame
matches the exact host tick in all three hash domains. A closed host trace then
proves both exact spawn boundaries: 1,200 ticks after death, 25 health, clear and
hazard-safe valid rope placement, approximately 72 units from the living partner,
with carried resources intact. Both rendered views independently verify the
spectator camera, its return and current non-overlap while the partner climbs.

| Public victim | Death tick | Revival tick | Rejoin ticks | Remaining countdown |
| --- | ---: | ---: | --- | --- |
| Red | 16926 | 18126 | 17471 to 17473 | 655 to 653 |
| Blue | 18174 | 19374 | 18400 to 18401 | 974 to 973 |

Public suite: `private/coop/regression-suite-fa1a0eba9c264a1ba59381572a7cc2cf/summary.json`
(48.00 seconds). Its passing case is
`private/coop/rope-countdown-0cc60c5a60034bed9857e1031b1fdded/`, with 2,403 exact
host trace rows and 290 matching player/world/native-enemy states. All seven
retained fixture source hashes match the current files. The checkpoint hash is
`1782b54ba1795b9f66119aa301d5299a84f86e1621c4db9b6299bf503540f52d`.
The local pass is `private/coop/rope-countdown-42c6a20ba71f4cf3899526ac96d621a0/`
(28.64 seconds, 363 matching states, both exact revivals). The final public
fixture reads its trace after closing the host rather than while it is writing.

Failed local attempts remain retained. `3c0acb83546947d6ac322dc59611f700` and
`beafec60ec3646d9a8b6ca51a5a72bb9` lose the low-health survivor to rifle fire;
`2bdb622727ac4be9927a962f44bf03aa` reaches a spawn wait after the survivor loses
her grip. Ordinary return fire alone did not solve those exposed waits. The
fixture now alternates ordinary climbing during Blue's remaining countdown.
`530f6fa043ae4584b6c244024261b028` observes both revivals but fails an old fixed
64-unit distance rule after the survivor has climbed closer. That rule remains
for the original neutral fixture; this moving case instead requires exact host
spawn geometry, current body separation and speed-bounded later observations.
No native game rule or enemy strength was changed to produce the pass.

The old Blue-victim internet attempt retained the same Red-falling checkpoint
as the old Red case and failed before gameplay; it supplies no Blue-role proof.
The new sequence supplies actual Blue death/revival evidence. No screenshots
were inspected. All owned clients close and Anode is released.

Four new guard tests reject changed grips/bodies, advancing held worlds and
stale or late guest evidence. All 406 Python guards and seven package tests pass.
The native executable is byte-identical to the previous 928-active-test/locked-
release build. This adds 0.25 combat/recovery point, giving 54.8% overall. It is a
captured recovery component, not a new completed campaign visit; the matrix
remains 6 combined, 21 partial and 12 not run.

## Underwater countdown reconnect, 5 November 2026

The new `tools/test_coop_utemple_countdown.py` uses ordinary swimming and actual
drowning, restarting only its owned guest process during each victim's countdown.
It adds both player-order cases to the regression runner and six rejection guards.
Native build `c5cb7fc` is unchanged; all evidence uses the same frozen executable.

The Blue-led route passes locally and through a public invite with 40 ms delay
and 20 ms jitter. Both Blue and Red are tested as the dead player. The waiting
host holds its entire player/life frame and world; the fresh guest inherits the
remaining timer, body identity, air, shells and personal resources. All three
first-frame hash domains match the host at the exact guest tick. Both players
revive after exactly 1,200 ticks with 25 health and fresh air in clear water near
the survivor. A subsequent team wipe restores the saved entrance for both, and
a further guest restart preserves that restored state.

| Public victim | Death tick | Revival tick | Rejoin ticks | Remaining countdown |
| --- | ---: | ---: | --- | --- |
| Red | 3474 | 4674 | 3712 to 3713 | 962 to 961 |
| Blue | 8039 | 9239 | 8571 to 8572 | 668 to 667 |

Public evidence: `private/coop/temple-countdown-9af51a1dd65f428da12c197dec1892c5/`
(150.25 seconds, 1,563 matching full-body/world/native-enemy states, shared wipe
at tick 13,320). Local evidence:
`private/coop/temple-countdown-63951454af72477bb751d1d0ea3f84ea/`
(107.38 seconds, 1,743 matches, shared wipe at tick 11,227). Both results retain
matching fixture sources and full before/held/after reconnect snapshots.

The two-order public suite is **failed**, not a complete pass:
`private/coop/regression-suite-9bab4afd60af4923ac35506c1e8e5e55/summary.json`.
Its Red-led attempt (`temple-countdown-c6aa18e081054012997d2b0d362a0634`) wipes at
tick 4,011 before the first reconnect. The surviving Red is near the first
column with full health/air just before the wipe. This remains part of the
already-open temple trap-repeatability work; no damage or collision rule was
weakened, and the attempt is not counted as reconnect evidence.

An earlier local attempt (`temple-countdown-e5a6d5ba396948f9a4798dc36f78665f`)
passed its first reconnect, then failed a fixed 32-unit camera/network-distance
assertion. The recovery fixture now retains coherent presentation samples and
uses the existing verifier for exact interpolation inputs and the actual drawn
camera subject. It also checks fresh air at revival. The failed attempt is
retained; no native camera behavior changed. No screenshots were inspected.

The 0.25 internet point credits the new underwater reconnect coverage for both
victims, not another campaign visit or a reliable Red-led temple route. Current
checks: 402 Python guards, seven package tests and source audits pass; the locked
928-active-test native suite and release build pass. Campaign status remains
6 combined, 21 partial and 12 not run. Overall completion is 54.6%.

## Saving during deaths and scenes, 5 November 2026

`tools/test_coop_menu_save_rules.py` verifies the real ESC/Save checkpoint UI on
unchanged native build `c5cb7fc`. Three public cases pass with 40 ms delay and
20 ms jitter per sender, sharing the exact final fixture identity. The fixture
checks the public invite scope rather than inferring it from its arguments.

The death case uses normal guard damage in the first school entrance. Save is
refused during each player's countdown, with the displayed reason and both disk
files checked. The attempt neither resets nor skips the life timer. Both clients
then observe moving-partner spectating, exactly 1,200 active ticks to revival,
25 health and 48-unit separation. Save succeeds after each revival, with the
new file's tick matching the menu receipt. The pair agrees on 1,259 transmitted
game, world and enemy states. Public evidence:
`private/coop/menu-death-save-9361ba6310024cb4845451149caba201/result.json`
(137.89 seconds; Red dies first).

| Public victim | Death tick | Revival tick | Refused Save tick | Successful Save tick |
| --- | ---: | ---: | ---: | ---: |
| Red | 5932 | 7132 | 6208 | 7285 |
| Blue | 11857 | 13057 | 12173 | 13277 |

Each theatre owner is tested during the camera introduction and active dialogue.
The refused saves preserve the committed files. The original scene continues,
its waiting partner stays hidden/frozen, and both clients agree on safe reunion.
Save then succeeds with both reunited bodies and no remaining scene owner.

| Public theatre owner | Reunion tick | Successful Save tick | Evidence directory |
| --- | ---: | ---: | --- |
| Blue | 5848 | 5931 | `private/coop/menu-test-339f436e11c943919ab590ef1ab8eaa8/` |
| Red | 5873 | 5923 | `private/coop/menu-test-6bfdcc75f63a4d6ab2cb788cebb63c1a/` |

The Blue/Red public scene cases take 83.12/79.44 seconds. Local baselines also
pass: `private/coop/menu-death-save-d6972cc826a84e46bf7762b5564e1fbe/`
(both deaths, Blue first) and `private/coop/menu-test-040f6b8d0e8f444a879ddfefe63ba6cd/`
(Blue theatre). The final fixture adds explicit invite/scene-phase assertions;
all three public results independently match its current source hashes.

Earlier local attempts remain failed evidence. Two stopped at the background
job's denied window activation, one at a straight-line approach blocked by the
partner, one after the first safe retry because the survivor had not moved, and
one on the second approach across the revived body. The fixture now uses Anode's
owned-window focus action, separate approach lanes and a distinct survivor
waypoint. These corrections do not change collision, damage, revival or save
rules; the original recovery/camera assertions remain intact. No screenshots
were inspected, and unrelated seat windows were left open.

Current checks: 396 Python guards, seven package tests and source audits pass.
The frozen executable retains its earlier 928-active-test and locked-release
validation; no native source changed in this pass. The 0.25 foundation point
credits normal-menu save eligibility and retry coverage. Campaign status stays
6 combined, 21 partial and 12 not run; overall completion is 54.3%.

## Menu storage recovery, 5 November 2026

Build `c5cb7fc` passes the new `tools/test_coop_menu_storage.py` through native
menu input on loopback and public internet invites. The only native change is
an opt-in passive receipt of the message actually passed to the co-op panel.
No gameplay or checkpoint behavior was changed to make these cases pass.

The fixture loads an existing solo slot, hosts it, joins, then holds real Windows
read handles that deny replacement of its isolated backup and primary files in
turn. Each Save checkpoint action reports failure visibly, keeps the committed
primary intact, removes its temporary file and lets both clients keep advancing.
After the handles close, Save succeeds in the same processes.

Backup replacement happens before primary replacement. If the latter fails,
the backup can contain the still-valid previous primary; it need not retain its
older bytes. This test uses distinct saved states and checks that exact behavior.
The earlier native two-file equality assertion used identical initial snapshots;
it did not establish atomicity across two distinct files.

After both games exit, only the fixture's primary is damaged. Fresh host/guest
processes use Continue co-op and a new public invite, display the backup recovery
notice, retain the two players' resources and repair the primary on Save. The
valid backup remains intact. A third host start with both files damaged reports
the read failure, preserves those files, returns the suspended solo state exactly
and still accepts ordinary solo movement. All three solo returns have matching
full-state hashes; the original solo slot retains its bytes throughout.

- Loopback: `private/coop/menu-test-14610712fb634ea28ae54edc9dd84e1b/result.json`
  (37.28 seconds).
- Public invites: `private/coop/menu-test-f05962a090824d3e840dfdc375d75e67/result.json`
  (74.17 seconds). This menu test uses normal network conditions, without injected
  delay/jitter; it is separate from the earlier delayed reconnect matrices.

Both runs pass on their first attempt. No screenshots were inspected. Locked
release, 928 active Rust tests (three existing ignored), 396 Python guards, seven
source-package tests and source audits pass. The 0.25 foundation point credits
the normal-menu storage recovery evidence. No campaign visit is added; overall
completion is 54.1% and core work remains ahead of campaign traversal.

## Public countdown and cutscene reconnects, 5 November 2026

Six automated cases pass over public invites on unchanged native build `0c5e717`,
with 40 ms delay and 20 ms jitter per sender. The guest is a fresh process after
each interruption. Retained snapshots prove that the inactive host holds its full
frame and world; the replacement inherits the running timer or scene state.

| Death case | Death tick | Revival tick | Health / separation | Shared checkpoint tick |
| --- | ---: | ---: | --- | ---: |
| Blue dies; living Red reconnects | 2008 | 3208 | 25 / 48 units | 4201 |
| Red dies and reconnects | 2077 | 3277 | 25 / 48 units | 4261 |

Both clients agree on the exact 1,200-active-tick revival and later team wipe.
The dead player's camera follows the moving survivor and returns to its own body
after revival. Personal resources and body identities survive reconnection.
Suite: `private/coop/regression-suite-5a62b7313b014e2b99d617bce86e760f/summary.json`.

| Theatre owner / restart phase | Reconnect ticks | Authoritative reunion tick |
| --- | --- | ---: |
| Blue / introduction | 1017 to 1019 | 5848 |
| Blue / dialogue | 3891 to 3892 | 5846 |
| Red / introduction | 1036 to 1037 | 5870 |
| Red / dialogue | 3921 to 3923 | 5872 |

All four preserve the owner, scene cues, pending contacts, personal resources,
and hidden/frozen waiting partner. Introduction reconnects keep dialogue queued;
dialogue reconnects retain the current line and advance it only by active ticks.
Both clients watch the original scene to completion and agree on a 48-unit
reunion without respawn or checkpoint markers.
Suite: `private/coop/regression-suite-1f0f564f3329490c9bef83cc1b82e8ca/summary.json`.

The regression runner now exposes both death cases and all four scene cases with
compact timer/reunion metrics and build identity checks. Test fixtures retain
before/held/after snapshots, including failure evidence, without opening images.
All 928 active Rust tests, 396 Python guards, seven source-package tests and the
locked release build pass. No native gameplay change was needed in this pass.

The added 0.5 point credits public verification: 0.25 for death recovery and 0.25
for the scene matrix. These staged school scenarios do not complete a campaign
visit, every map's recovery behavior, skipped scenes or audible voice playback.

## Connection failure and cancellation recovery, 5 November 2026

The real-socket regression reproduced a Join cancellation bug: the UI returned
while the connection worker retained an unanswered handshake until its network
timeout. Build `0c5e717` registers the connected socket for cancellation and
shuts it down when the peer is stopped or dropped. The temporary duplicate is
released on both successful and failed handshakes. Cancellation tests cover
plain WebSocket and TLS setup; a separate HTTP rejection test retains its peer
object and still observes the failed socket close. DNS lookups and TCP connect
calls remain on background workers; cancellation cannot interrupt an OS lookup
already in progress, but a cancelled worker cannot enter gameplay afterward.

The automated real-menu run passed in
`private/coop/menu-test-d2faef5612b645839af27401e5876f24/`:

- An invalid hostname exhausts DNS retries and returns to Join with its link
  retained (23.39 seconds in this run).
- An owned HTTP 503 responder returns a useful error without exposing its body.
- Cancel closes the actual stalled handshake connection; the complete measured
  click/return check took 0.328 seconds.
- A temporary filesystem obstruction in the isolated host profile produces a
  genuine setup failure, returns to Host and leaves the solo save untouched.
- Cancelling another Host attempt stops its helper. Before/after process records
  show only the same pre-existing game console child, with no new helper left.
- The same host and guest processes then connect through a public invite, retain
  the loaded resources/world, save a checkpoint and leave. The manual co-op save
  and original solo slot remain unchanged; suspended solo state hashes match.

The first menu attempt passed the Join checks but stopped at an overly broad
cleanup assertion that rejected any child process, including the game's own
console. The revised check records the baseline and rejects new remaining
children; that complete rerun passes. Original evidence remains private.

All 928 active Rust tests, 396 Python guards, seven source-package tests and the
locked release build pass. The added 0.5 point credits the cancellation fix and
the verified failure-to-public-retry flow. This does not claim that the external
invite provider or every DNS resolver will always be available. No campaign
visit is newly completed and no screenshots were inspected.

## Loading-barrier reconnect evidence, 5 November 2026

Build `9921a3e` adds passive session-phase/pending-visit diagnostics and a native
socket regression; normal gameplay and loading rules are unchanged. The automated
travel runner suspends only its own guest before departure, observes the host
in Loading with the next visit pending, and verifies a frozen full frame/world.
It terminates that guest, waits for the host to load its pending destination and
hold there without advancing the simulation, then starts a fresh guest on the
same invite. Both continue through all three visit epochs and clean exits.

| Public-invite case | Held simulation tick | Fresh guest tick | Result |
| --- | ---: | ---: | --- |
| Outbound to the return-school entrance | 1440 | 1442 | Pass |
| Return to the cached first entrance | 2041 | 2043 | Pass |

Public suite: `private/coop/regression-suite-5618659c836c4efea039031063f49ea8/summary.json`.
Loopback suite: `private/coop/regression-suite-aa4d01e85f8b49d1b4d646295d0d4399/summary.json`.
Both use 40 ms delay and 20 ms jitter per sender. The checks retain health, Will,
earned weapon copies, pickup/drop history and paired relocation markers, with
exact cached world/NPC restoration on return. The destination's normal campaign
weapon grants are required exactly; arbitrary additions or resource resets fail.
The first local attempt stopped on an overly strict no-grants assertion. Its
evidence was retained and the oracle corrected before the complete passing runs.

Native tests interrupt before Enter reaches the guest, after Loading, after guest
Ready and after the host finishes loading. They verify that the guest slot is
released, the pending visit is retained, both readiness requirements still apply,
and old controls cannot move the rejoined player. Seven oracle guard tests cover
missed barriers, advancing clocks, stale processes, lost resources, wrong
visits, duplicated history and body resets. All 926 active Rust tests, 396 Python
guards, seven source-package tests and the locked release build pass.

These are staged transitions between visits of the school map, using real load
barriers and independent clients. They do not certify quest exits, all map pairs,
or another campaign visit. The added 0.25 point credits loading-interruption
coverage only. No screenshots were inspected.

## Moving-lift reconnect evidence, 5 November 2026

The automated `lift-rejoin-blue` and `lift-rejoin-red` regression cases pass
over public internet invites on build `4b3760a`, with 40 ms delay and 20 ms jitter
per sender. Both players walk from the staged lower-library start. The first
rider waits two shared-world seconds without advancing the lift pose or motion
clock. After the second boards, both ride the original rotating platform.

The runner terminates only its guest process during the ascent, verifies the
host has observed disconnection, and checks that the full frame and combat/world
snapshot stay frozen. It then starts a new guest on the same profile and invite.
The fresh client retains the authored motion keys, phase and matching platform
pose. Each rider retains their position in the lift's rotating coordinate system,
grounding, identity and starting resources. No live position, health, inventory
or platform value is written. The normal ride must still finish at the upper
landing and pass the complete sent/restored player and shared-world comparisons.

| First boarder | Duration | Matching player/world states | Final health |
| --- | ---: | ---: | --- |
| Blue | 84.27 seconds | 857 / 857 | 100 / 100 |
| Red | 88.20 seconds | 920 / 920 | 100 / 100 |

Blue-first reconnect advances from ticks 6099 to 6101, with platform time
2.2500017 to 2.2666686 seconds. Red-first advances from ticks 6632 to 6633,
with platform time 2.3916702 to 2.4000037 seconds. Maximum measured rider-offset
drift across either restart is below 0.00025 units. Both pairs reach height 64.
The source game is unchanged; these are new automation and verification cases.

Suite: `private/coop/regression-suite-f6fda7b53f104b29b1d16882a74c7cf3/summary.json`.
Cases: `private/coop/school-lift-da48bc0032364d7d81027f1cd26a5a53/` and
`private/coop/school-lift-5911f19b32d4434d984c9c26453fb46a/`.
The preceding Blue-first loopback baseline and reconnect also pass in
`school-lift-2a306facf0484287b2cfc161ffb2a112/` and
`school-lift-93658396311b4ddf9dd5b827ac71bff5/`, under `private/coop/`.

Five new guards reject stale processes, resets/skips, invalid motion poses,
changed resources/body identity, detached riders and falsely crediting a
stationary entrance reconnect as a ride. An unchanged world-space player
position is explicitly rejected when the platform rotates/translates beneath it.
The locked release, 925 active Rust tests, 389 Python guards and seven package
tests pass. No screenshots were inspected. The added 0.25 Host/Join points credit
the verified moving-ride reconnects; no campaign visit is added. Overall: 52.6%.

## Cutscene reconnect and public menu evidence, 5 November 2026

Build `4b3760a` passes four `tools/test_coop_scene.py --local --rejoin` cases
with 40 ms delay and 20 ms jitter per sender. Each terminates the guest during
the original theatre, observes the host stop, verifies a held full frame and
combat/world snapshot, and joins with a fresh process. No scene is skipped.

| Owner | Restart requested at scene time | Rejoin active ticks | Shared reunion tick | Separation |
| --- | ---: | ---: | ---: | ---: |
| Blue | 6 seconds | 13 | 5848 | 48 units |
| Red | 30 seconds | 13 | 5864 | 48 units |
| Blue | 30 seconds | 14 | 5846 | 48 units |
| Red | 6 seconds | 14 | 5862 | 48 units |

The six-second cases retain queued dialogue during the camera introduction;
the thirty-second cases retain the same active line and its elapsed time.
The scene clock advances by exactly the active host ticks, within float
tolerance. Ownership, pending contacts, starting personal resources and body
identity survive; the waiting partner remains frozen and hidden in both clients.
Both clients watch the complete scene and agree on the normal reunion.
Four guard tests reject resets, stale PIDs, changed resources/contacts, early
dialogue, replacement bodies, moved partners and incorrect visibility.

Evidence under `private/coop/`, in table order:
`scene-test-fe7e1caf696442ae815c15196c0adf80/`,
`scene-test-ac93e19fe7cd45ec89f60eb3b0cd4f0f/`,
`scene-test-6c647ad26dfc4337b13b758f287ef07f/`,
`scene-test-0e3e8f4f435d4509b8d289c63a0362db/`.

The old Blue-first fixture was blocked by the waiting Red in its single-file
approach. Before simulation, the fixture now assigns the collision-checked
front spawn slot to the requested owner and records that initial checkpoint.
The actual walking/contact still starts the scene. Normal spawning, collision,
scene rules and saves are unchanged. The stalled attempt remains in
`scene-test-f144fa012a1c42e290e6d1b8222fd59c/`; two harness failures involving
an omitted pre-scene field and queued dialogue remain in `scene-test-ca2da77805ba40b59ec0c0530cca57e3/`
and `scene-test-1fd35a937eb944539efea32501163dee/`. The reader and oracle now
match those actual serialized states. An earlier Red reconnect on `0b32c79`
passes in `scene-test-0578514c1d51406389913f254d5a12b5/`.

After more than an hour without public provisioning attempts, the final build
passes `tools/test_coop_menu.py` with `internet: true` and real Windows input:
`private/coop/menu-test-5194337850bb40e1bf2fbfae0f52487e/`. It loads a solo slot,
hosts that current game through ESC, joins a guest, saves, leaves and returns
to the exact suspended solo state. It then verifies expired-link feedback,
Continue co-op with a new public invite and a second exact solo return. Source
solo bytes stay unchanged. Batched typing, invalid input and Cancel also pass.
This completes the previously open public menu case; it does not prove that
the earlier DNS failures or provider rate limit cannot recur.

The locked release, 925 active Rust tests, 384 Python guards and seven source
package tests pass. No screenshots were inspected. The added 0.25 Host/Join
points credit the cutscene reconnect cases, and 0.5 credits the complete public
menu flow. No campaign visit is added. Overall: 52.3%.

## Countdown reconnect evidence, 5 November 2026

Build `0b32c79` passes both `tools/test_coop_play.py --local --death --rejoin`
roles with 40 ms delay and 20 ms jitter per sender. The runner terminates only
its guest process during a real guard-inflicted death, waits for the host to
observe the loss, checks the full paused frame and combat state, then starts a
new guest with the same profile and invite. The fresh process must inherit both
players' inventories, health, body identities and the correct camera. No health,
position or clock is written to create the result.

| First victim | Death tick | Revival tick | Health / separation | Shared wipe tick |
| --- | ---: | ---: | --- | ---: |
| Blue | 2008 | 3208 | 25 / 48 units | 4203 |
| Red | 2077 | 3277 | 25 / 48 units | 4261 |

Both clients agree on these events. Blue's reconnect retains 847 then 839 ticks
with eight active ticks elapsed; Red's retains 891 then 877 with 14 active ticks
elapsed. Evidence: `private/coop/play-test-8a35b8e7f545481e9ad50f19dcf5c04f/`
and `private/coop/play-test-f9c88e2556c4491baa9d0ee5836f6b9c/` respectively.
Four guard tests reject stale processes, rewound/reset clocks, replaced bodies,
wrong cameras, changed resources and unexpected healing. The lifecycle verifier
also rejects a countdown reset between samples.

The first Blue attempt on `8c02fd0` failed because the old straight-line test
approach stopped at Red's contact radius, leaving Red as the nearer victim.
That failure remains in `private/coop/play-test-3f9923da110c4425836934a832e65d2f/`.
The development driver now walks around Red through observed waypoint arrivals;
normal movement, collision, AI and damage rules are unchanged. The earlier Red
baseline and reconnect pass on `8c02fd0` remain separate historical evidence.

Build `c57d7df` also passes all 82 native `--coop-world-check` records, including
shared boarding/transport, personal pickups, both theatre owners, safe reunion
and checkpoint restoration. These are focused core checks, not campaign routes.

The staged combat repeat initially failed on `c57d7df`: Blue was again blocked
behind Red, so only Red attacked. Evidence is retained in
`private/coop/play-test-909f0165682245ba8917924ba1e28364/`. The combat driver now
uses a clear side approach and waits for both players to reach Blade range.
Build `0b32c79` passes the unchanged combat assertions with two real attacks from
each player, a shared guard defeat and minimum health 100/64 on both clients.
Evidence: `private/coop/play-test-2beed43adfdc4c0ea3866f76cc95f35b/`.
Both driver corrections are restricted to explicit development scenarios.

The final `0b32c79` contact/ambient-effects run passes 612 matching sent/restored
states, both directions of approach, stationary-partner preservation and a
36-unit contact distance. Both models remain visible and both clients render
ambient effects without dropped batches. Evidence:
`private/coop/presence-test-a55f8bfb1d2b4523b2b14fb5bb18528d/`.
All new paired checks use loopback with 40/20 ms delay/jitter; no screenshots
were inspected. The locked release, 925 active Rust tests, 380 Python guards
and seven source-package tests pass. The unchanged native world checks above
were run on `c57d7df`, before the combat-only driver correction.

The added 0.25 Host/Join points credit the newly verified countdown reconnect
cases. They do not resolve public invite provisioning/DNS, finish core acceptance
or complete another campaign visit. Overall: 51.6%.

## Menu acceptance evidence, 5 November 2026

Build `8c02fd0` passes `tools/test_coop_menu.py --local` with real Windows mouse
and keyboard input in Anode. Evidence:
`private/coop/menu-test-d6691e95748e4e9583c1d6368a779c57/`.
The original main menu loads the solo slot; ESC hosts that current game. Both
clients retain the expected positions, resources and opened door. The ordinary
co-op menu saves and leaves without replacing the manual checkpoint. Returning
to solo preserves its entire serialized state hash; ordinary movement then
works. Continue co-op restores the shared save independently of that changed
solo position, issues a new invite and returns to the second suspended solo
state unchanged. The source solo slot retains its bytes throughout.

The runner also verifies batched invite typing, malformed-link feedback,
cancelling a pending handshake, an expired session link, and menu recovery after
the host leaves. A real typing bug was fixed: Macroquad returns a character
batch newest-first, so the invite field now restores event order. Passive,
opt-in reports provide actual button bounds and state receipts; no screenshots
were inspected. A harness focus failure was corrected by activating only its
owned game windows; the passing full run supersedes those harness failures.

The public attempt on the same final executable fails at provisioning with
HTTP 429: `private/coop/menu-test-74dfab0b5f304251a516993ff98b456a/`.
It verifies the improved retry-later error and intact solo return, but cannot
credit the full public menu flow. The preceding public attempt on `e0fe7f6`
failed the same way. No automatic retry loop or provider-limit workaround was
added. Prior successful internet persistence tests remain build-specific evidence.

The additional 0.25 Host/Join points credit verified menu routing, input and
error/cancel handling. They do not credit public menu connectivity, resolve DNS
reliability, complete core acceptance or add a campaign visit. Overall: 51.3%.

## Storage and host-loss evidence, 5 November 2026

Build `7994cc2` passes the extended `tools/test_coop_import.py` suite over public
internet invites with 40 ms delay and 20 ms jitter per sender. Evidence:
`private/coop/import-test-6d6f290f6632436784c49ab6e67a5e9d/`.
The native storage receipt records two real sharing-violation failures and two
exact team-wipe restores of the old checkpoint. It also verifies stale temporary
files, successful checkpoint replacement and backup preservation after corruption.
These cases do not simulate electrical power loss or disk exhaustion.

The harness terminates the resumed host and its helper without a graceful exit.
The guest reports the disconnect in 6.33 seconds; both committed files retain
their bytes. Fresh processes then resume the save, rejoin, and recover the backup.
All four network phases retain the two inventories, shared world and solo files.
This CLI diagnostic evidence does not yet verify the normal menu's error panel,
return to the suspended solo game, or subsequent hosting through ESC.

The earlier invite DNS failure did not recur in six fresh native game sessions
or six headless transport sessions. Its cause remains unproven and no resolver
fix is claimed. These successful retries do not close intermittent reliability.
A separate full multi-visit checkpoint rerun on `7994cc2` passes travel/rejoin,
then fails before the resumed host can publish an invite: the temporary tunnel
provider returns HTTP 429 during provisioning. Evidence:
`private/coop/travel-test-1bbdc0c80f44484fbaf8edd1441b4542/`.
That run remains failed; repeated new-session setup hit a real provider limit.
No immediate internet retry or rate-limit workaround is used.
The same multi-visit checkpoint passes separate primary and backup resume/wipe
checks on loopback with the same 40/20 ms conditions:
`private/coop/storage-resume-local-32dbde14645349219084727e4fe31ceb/`.
This isolates persistence from the provider failure; it is not an internet pass.

That storage milestone left the total at 51.1%; it strengthened core evidence
without completing the menu/connection milestone or adding a campaign visit.

## Solo continuation evidence, 5 November 2026

Build `a9b55a5` passes `tools/test_coop_import.py` over public internet invites
with 40 ms delay and 20 ms jitter per sender. Evidence:
`private/coop/import-test-9e0363c7644545c8a8313b8c27a48b5a/`.
The native fixture verifies Blue's full body, view, resources, actions and world,
the distinct cached return visit, and safe Red placement. A separate native
swimming case retains Blue's body/air supply and places Red at the same depth.
Each network phase includes a guest process restart using the current invite;
both host and guest restart between primary and backup sessions. The solo slot
and guest sentinel remain unchanged. No image inspection was needed.

The initial build `08033a5` also passed the same three network phases. A later
fixture attempt started in Garden's opening cinematic and correctly refused
import; the swimming fixture now uses freely controlled Temple gameplay. The
observed DNS failure remains an open reliability item despite successful import
runs. The separate final checkpoint run
`private/coop/travel-test-c2fb379c358d42c4b059f91bea7b1ea3/` passes staged travel,
guest reconnect and primary resume/team wipe; its backup guest failed DNS
resolution. The runner now reports that failure immediately. A separate loopback
backup/team-wipe test passes with 40/20 ms conditions and unchanged files:
`private/coop/backup-local-a747d7b0866a4aaf840cc3174b2c852a/`. This isolates save
correctness from internet setup; it is not an internet backup/wipe pass.

Validation: 923 active Rust tests (three existing ignored), locked release,
376 Python guards, seven source-package tests and source audits. This change
does not finish all core acceptance work.

## Evidence history

The dated and build-specific results below retain historical percentages;
the current overall estimate is the one at the top of this document.

Latest native milestone: Burning Curiosity source `56160ff` passes **26 staged
cases plus one summary**. Each living player receives the original arrival
grant once and retains their own two essence pickups. The sinking platform waits
for both living riders, carries them at 30/60/144 Hz, transfers them onto the
uneven floor independently and returns when empty. A blocked return rolls back
the platform and both bodies. Original arrival and first-entrant Caterpillar
scenes pass watched/skipped, living/dead partner and compressed-save cases.

The locked release passes seven single-player component checks and the full
native combat route; both outputs match the previous baseline exactly. The
separate cast-free traversal still has its pre-existing step-54 lava-jump
failure, also unchanged. These results add **0.75 points** under the existing
component weighting: 0.25 resources, 0.25 movement and 0.25 progression.
No additional campaign visit is counted as complete. Next is Burning Curiosity's
full route with two independent internet clients, followed by recovery and
reconnect checks. See [Burning Curiosity evidence](JLAIR1.md).

Current validation: **919 active Rust tests** pass (three ignored), along with
the locked release, twenty traversal and five recovery guards for Curiosity
(376 guards across the Python suite), seven package checks and
the 912-file source worktree audit. Negative tests cover partial boarding,
floating bodies, marginal ledges, damaging-liquid navigation and stationary
bodies that retain collision velocity, alongside valid sloped landings.
No images were inspected for these milestones.

Burning Curiosity's first fresh internet segment now passes **steps 0-3** with
eight personal arrivals, watched entry, both early lava crossings and 2,133
matching player/world/native states. Both survive without revival at 98.75/95.42
health; a normal checkpoint before step 4 is retained. The longer approach and
full route remain open. This segment adds no percentage or completed-visit credit.
A later resumed attempt has independently verified evidence through **4-20**:
34 personal arrivals, five forward waiting walks and 5,582 matching states, with
no deaths. That longer attempt fails at guest planning for step 21; its successful
prefix is bounded separately. This also adds no percentage or completed visit.
Source `ca4cc26` then corrects the automated planner's floor-plane checks, keeping
its eight-unit support margin and requiring actual body contact. The previously
stalled climb now passes in a separate resumed **21-23** internet run: six personal
arrivals, one forward waiting walk and 2,224 matching states. Both finish at
100 health without revival; a normal checkpoint before 24 is retained. Native
component and single-player baseline outputs remain identical.
Later bounded **24-25** evidence adds four personal arrivals and 1,563 matching
states. Two attempts fail at 26 under fire-spitter attacks. The runner now allows
ordinary combat dodging before parking and uses the original route's Cards
preference. A resumed **26-29** run passes eight personal arrivals, the fight and
1,374 matching states, ending at 50/96 health without death or revival. Its normal
checkpoint before 30 is retained. A further attempt verifies **30-31** separately
(four arrivals, 352 matching states), then fails the forward climb from 32 toward
33 when Red falls into lava. A separate retry verifies **30-32** with six
arrivals and 783 matching states, taking a different ascent before failing
parking at 33; climb reliability remains open. Forward waiting now uses actual
native dry bays instead of treating the next route point as a safe stopping
place. Bounded **33-37** evidence has ten arrivals and 1,309 matching states.
A separate bounded **38** replay uses ordinary jumps onto the original hot
waypoint and back onto dry ground: two arrivals and 462 matching states,
93.75/41.67 health, no deaths or revivals. Its longer attempt fails at 39 when
the planner walks into lava. Those full attempts stay failed. Source `259d4f5`
adds optional liquid avoidance and corrects blocked-walk detection to use actual
position progress instead of retained collision velocity. A resumed **39-42**
internet replay now passes eight personal arrivals and **1,335 matching states**,
ending at 100/56.67 health with no deaths or revivals. The normal checkpoint
before 43 is retained. A subsequent remote three-jump sequence fails at 43.
Source `5c4a693` stops timed sequences after the first planned landing and settles
the actual body before planning onward. The runner also follows successive
original approach points when a narrow slope has no nearby waiting bay, with
separate personal receipts for every intermediate point. Bounded **43-45**
evidence then passes six arrivals and **869 matching states**, ending at
100/71.67 health without deaths or revivals. That longer attempt fails at 46
when a new plan starts during an unfinished ledge pull. Its normal checkpoint
before 46 is retained. Source `c7576ca` then waits for active ledge pulls to
finish before starting another route search. A separate **46-49** internet run
passes eight personal arrivals and **1,017 matching states** at 100/46.67 health,
with no deaths or revivals. A separate resumed **50-52** segment passes six
arrivals and **699 matching states** with no further damage, death or revival.
The normal checkpoint before 53 retains both players beside the raised platform.
Separate evidence covers approach points 0-52; a fresh continuous repeat is
still required. The later platform checks below do not complete the remaining
route, recovery or reconnect coverage.
A separate **53** internet segment on `9f1acd0` now passes both personal
arrivals and actual single-rider waiting: Red has full native deck contact,
Blue remains off the platform, and it stays raised for at least 120 ticks in
both clients. There are **317 matching player/world/native states**, no new
damage, death or revival, and a normal checkpoint before 54. This build only
adds read-only platform contact diagnostics; the native platform cases and
single-player outputs remain identical.
The paired platform crossing now passes in a closed **54-only** segment:
both players reach the opposite bank alive, Blue waits there for Red, a normal
checkpoint before 55 is saved, and 152 player/world/native states match.
A longer **50-54** repeat then passes ten personal arrivals, actual boarding,
waiting and carry, both ordinary jumps and stable bank waiting. All **1,145**
comparable player/world/native states match; both retain **100/46.67 health**
without damage, death or revival. The current checkpoint before 55 is retained.
The crossing destination is explicitly the flat bank `[-7208,768,1632.03125]`,
instead of the single-player fixture's sloping point 54. That point is not a
game trigger. The original course file is retained and the changed crossing
goal is recorded; this is no claim of landing at the old coordinate. Only
normal controls are used, with native collision, resources and level behavior
unchanged. The subsequent route evidence is recorded below. No screenshots
were used.

Post-platform evidence now covers **55-64 in separate segments**. A closed
55-57 internet run passes six personal arrivals and 572 matching states with
100/46.67 health and no new damage, death or revival. A later run has a verified
58-63 prefix (12 arrivals, 1,326 matching states), then fails at 64. On source
`0871d86`, a verified 64 prefix passes both arrivals and 255 matching states,
then times out at 65. These bounded audits retain the parent failures.

The diagnostic driver can finish a supported grounded approach before planning
its jump; this is exercised by Blue at 64. Forward waiting now rejects fleeting
fast landings. A grounded lava-planning failure can trigger a recorded ordinary
jump or automatic climb back to recent safe footing. Other driver failures
remain fatal, and a recovered planning error requires a verified escape receipt.

Blue reaches 65 through an additional ordinary approach, but paired 65 remains
failed: after a verified lava escape, the next jump falls into deep lava. A
combined 55-64 repeat also fails at 60 while Red is swimming in lava; the
current grounded escape action cannot recover that state. Continuous route
reliability, 65 onward, a fresh full visit, recovery and reconnect remain open.
The latest native components and single-player outputs still match baseline;
no gameplay physics, resources or level progress were changed by these controls.
Total remains **50.1%**; no extra completed visit is credited.

Latest network milestone: a **fresh full About Face route passes** in two
independent internet clients on source `3298768`, using ordinary controls with
40 ms delay and 20 ms jitter. Ten personal arrivals, both reward pickups, the
shared lift, Red-owned watched Gryphon exit and actual shared `jlair1` arrival
are verified. The closed evidence has **6,805 matching player/world/native
states**. Blue's death at 23,061 revives at 24,261 with exactly 25 health and
verified resources, placement and spectator/returning cameras. A separate
resumed ending verifies Blue ownership with 2,169 matching states.

Latest recovery milestone: Red also revives after exactly 1,200 active ticks
at 25 health, with resources, safe placement and cameras verified across 1,087
matching states. A fresh guest process rejoins an unchanged normal victory save
with personal resources, pickup ownership and shared progress preserved: 77
matching states, including 58 after rejoining. Timed drops age normally.

Latest wipe milestone: both players die through ordinary combat and return
together at tick 7,060 to the normal checkpoint saved at 3,421, at 100/100 health.
Source `116fc99` adds an opt-in exact restore receipt before another tick can
change the world. Characters, enemies, pickups, shared progression and personal
resources match the actual saved checkpoint. Both clients agree on 1,058 states,
including 27 after restoration; the closed 917-row trace verifies the boundary.

All 905 active Rust tests, the locked release, 83 native cases plus three
summaries, 16 About Face Python guards and seven package checks pass. The
single-player component and full-route outputs still match the baseline exactly.
Renderer counters from the earlier full route show particles/effects in both
clients with zero dropped batches; no images were inspected. Combat/transport/
transition reconnects and remaining ending cases keep this visit partial.
The remaining About Face checks stay open in the matrix. This recovery milestone
retained the then-current **49.3%** estimate and six combined visits.

Earlier native milestone: About Face source `3298768` passes **83 staged cases
plus three summaries** for combat, personal resources, both lifts and the original
scenes. The bridge waits for both players to clear it; lifts enforce boarding and
carry living riders together, including obstruction rollback and catch-up from a
supported destination. Each player retains their own rewards. Legacy saves and
low-health revivals keep their health. Gryphon scenes bind the first entrant's
pose on the trigger frame and preserve the partner through watched/skipped play.

**905 active Rust tests** (three ignored), the locked release, seven package
checks and the 900-file source worktree audit pass. Both single-player checks,
including the full input route, match the unchanged baseline exactly. No images
were inspected. Under the existing component weighting this adds **0.75 points**:
0.25 combat/resources, 0.25 movement and 0.25 progression. The total then was **49.3%**.

No campaign row is completed by staged components. Six of 39 retain combined
verification evidence. About Face's remaining recovery/reconnect and
ending cases stay open. Clockwork's full-route and
recovery gaps also remain. See [About Face evidence](HATTER2.md).

Latest completed traversal sections: resumed **144-145** and **146-149** pass.
Both players enter the lowered clock portal through ordinary jump/climb input,
then personally reach the final lever approach. The first arrival leaves the
portal exit clear for the second. This section has **361 matching
player/world/native states** and retains the normal checkpoint before 146.

Red then uses the final lever and owns the watched 30-second Stop scene. Blue
stays hidden and still; both reunite safely. After the original 20-second wait,
Red approaches and enters the original exit, moving both players into About Face.
The shared transition has its own receipt, rather than claiming that both
players walked to the exit waypoint. This section has **1,122 matching states**,
including **27 after arrival**, with health preserved at **26.93/23** and personal
resource, camera and body-clearance checks passing. No new deaths occur.

Earlier resumed 141 verifies Red's exact 10-second revival and a new personal
chair approach; 142-143 verifies onward waiting. All **210 Clockwork Python
guards**, **15 generic recovery guards** and seven package checks pass. Those Clockwork runs used native
source `84c8eee`; its previous 905 active Rust tests, 72 paired/driver checks and
exact single-player 11+1 baseline were not rerun for those Python-only changes.

Remaining Clockwork work: fresh full-route, remaining scene-owner, recovery and reconnect verification.
These checkpoint segments through the exit do not complete the full visit's
verification matrix or add another campaign row.
See [Clockwork evidence](HATTER1.md).

Earlier diagnostic work: source `84c8eee` detects an actual off-course landing
before sending later jumps from the old plan. It also reports nearby dry,
fully supported return targets from native geometry, without moving either
body. The chair fixture requires the real jump and landing on that target.

The new resumed **140** replay records both personal chair arrivals, one Red
pull-up and Blue's safe return, with **2,312 matching player/world/native
states**. It still **fails**: Red reaches the chair with about **8.53 health**
and dies landing after the return jump. Her approach already incurred normal
damage, including slime. No rescue warp or revival occurs before either arrival.
The earlier replay timed out during a corrected route search; the bounded
approach allowance is now 180 seconds, while departure remains 30 seconds.
Neither failed replay earns a completed paired step or campaign visit.

Validation passes **905 active Rust tests** (three ignored), the locked release,
**72 native paired/driver checks**, the exact single-player **11+1** baseline,
**169 Python guards** and seven source-package checks. The 40 scene cases still
belong to earlier source `04d3b0c`. A survivable third-chair approach/departure,
the fourth chair and later route remain open. Overall stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Previously completed traversal section: resumed **138** now passes for both players.
Each reaches the original chair waypoint, then uses an observed native jump to
return to dry ground before the partner approaches. The run verifies the puzzle
contact, two personal arrivals, two departures, one observed Red pull-up and
**1,828 matching player/world/native states**. Both survive at **67/67.73 health**;
ordinary slime and combat damage is retained. A normal checkpoint before 139 is
saved. The earlier failed waiting arrangements remain recorded separately.

The next requested 139-143 run completes **139**. Its separately audited section
adds two arrivals, two departures and **1,741 matching states**, ending at
**54/43.53 health**. Step 140 remains unverified: Blue's selected return point
is beneath a higher landing, and Red's delayed jump route triggers the original
rescue portal before a new search times out. The runtime now enforces the same
return-height check as the final verifier, so it stops at that mismatch earlier.

The fixture returns to the first nearby dry footing from the actual approach.
A reproduced selection bug chose a later point beneath the chair rim, leaving
Blue above her target; its regression now rejects that choice. All **166 Python
guards** and seven source-package checks pass. Native source remains `99a66d6`,
with **901 active Rust tests** (three ignored), the locked release, **72 native
paired/driver checks** and the exact single-player **11+1** baseline. The previous
**40 scene cases** belong to source `04d3b0c`; they were not repeated for these
diagnostic changes. The remaining chairs, second portal, final scene/exit and
full-route/recovery/reconnect repeats remain open. No additional campaign visit
is credited; overall stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier completed traversal section: Clockwork's resumed **136-137** portal section passes
with two personal approach arrivals, two input-driven native climbs, separate
portal crossings, one waiting handoff and **202 matching player/world/native
states**. Both players remain alive at **80/100 health**; the leader takes two
ordinary hits while waiting beyond the portal. No health is reset for the test.

The preceding **128-135** doorway section is separately audited with ten
personal arrivals, five waiting handoffs, three timed holds and **562 matching
states**, ending at 100/100 health. The first player observes both rotating
leaves opening and walks forward before the partner approaches. The original
route's later wait and both players' waypoint arrivals remain required.

A reproduced portal bug could place one Alice directly inside the other.
An occupied exit now waits for the living partner to move clear. The test
controller also preserves a consumed jump across teleports, while allowing
new commands and new visits to jump normally. The retained failed portal run
is rejected by the verifier for its repeated jump at the destination.

Frozen source `04d3b0c` passes **897 active Rust tests** (three ignored), the
locked release, **72 native paired/driver checks**, the unchanged single-player
**11+1** baseline and **40 staged scene cases**. All **151 Python guards** and
seven source-package checks pass. The following **138-143** attempt times out
at the first chair approach after an incomplete remote ledge pull; it earns no
completed steps. Both players remain alive at 80/90 health. The chair section, second portal, final
scene/exit, full-route and recovery/reconnect repeats remain open. No additional
complete campaign visit is credited; overall remains **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **109-116** section passes with
eight personal arrivals, the watched Blue-owned Gryphon scene, four waiting
handoffs, three timed holds and **1,353 matching player/world/native states**.
A separate Red-owned Gryphon replay of **112-113** passes with **960 matching
states**. Both runs finish at 100/100 health and verify all seven dialogue
stages, hidden/frozen partners, preserved resources and safe shared reunions.

The next requested **117-136** run stops at 128 because its narrow approach
offers no usable sideways waiting position. Its separately audited **117-127**
section passes with 22 arrivals, 20 pathnode projections, 11 waiting handoffs
and **1,514 matching states**, also at 100/100 health. A normal checkpoint
before 128 is available for the next retry. The original failure is retained.

Native source remains `11c5ba2`; all **40 staged paired scene cases** now pass
on this exact build, including both owners, watched/skipped scenes, pause/restore
and living/dead partners. All **131 Python guards** pass. No further complete
campaign visit is credited. Later traversal, portals, the final scene,
recovery/reconnects and full-route repeats remain open; overall stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **107-108** doorway section passes
on source `11c5ba2`, with four personal arrivals, two observed door approaches,
one forward handoff, one waiting handoff and **692 matching player/world/native
states**. Both players finish at 100 health, and an ordinary checkpoint before
109 is saved. The runner lines up at authored waiting points and observes both
rotating leaves opening before continuing to the original waypoint. Its precise
walking handoff now checks walls while permitting normal step-up clearance.

All **893 active Rust tests**, the locked release, **72 native checks**, the
unchanged single-player **11+1** baseline, **124 Python guards** and seven
source-package checks pass. Earlier unsuccessful doorway replays are retained.
This completes the previously stalled section, not the whole visit. Later
Clockwork scenes/traversal, recovery/reconnects and campaign repeats remain open;
overall completion stays **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's completed resumed **99-106** section
verifies 16 personal arrivals, two pathnode projections, seven forward handoffs,
one waiting handoff and **1,845 matching player/world/native states**, with both
players at 100 health. The requested 99-108 run times out at 107 while the leader
approaches its next waiting point. It stops about 16 units short with the native
controller reporting `settling landing`; this remains the next investigation.
The original failure and the separately audited completed section are retained.

This batch also verifies **81-93** (2,421 matching states, 46/85 health) and
**91-103** (3,181 matching states, 100/100 health), each scoped separately from
its original longer-run failure. The test controller now coordinates narrow
stairs and winding paths, permits ordinary defensive movement while waiting on
static ground, and bounds an unsuccessful melee approach before allowing a
normal Blade throw. Source `cefabf5` passes **891 active Rust tests**, the locked
release, **72 native checks** and the exact single-player **11+1** baseline;
all **116 Python guards** pass. No additional complete visit is credited.
Later machinery/scenes, recovery/reconnects and full campaign repeats remain
open. Overall stays **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **68-81** section passes with
26 personal arrivals, 16 pathnode projections, 13 waiting handoffs, one timed
hold and **2,294 matching player/world/native states**. Both finish alive at
16.53/100 health. The combat test controller now requires supported footing
under its full stance through delayed movement and braking. The regression
fails on the old controller and passes with the correction; the real internet
replay also passes the fight that previously killed Red. Source `894292f`
passes **890 active Rust tests**, the release, **72 native checks**, the exact
single-player **11+1** baseline and **110 Python guards**. Ordinary checkpoints
allow targeted retries near this fight and the next route section.

The preceding requested 68-108 run remains failed at 77, with a separate scoped
audit crediting only 68-76 and 1,237 matching states. Later Clockwork traversal,
recovery, reconnects and complete campaign repeats remain the bottleneck. No
further complete campaign visit is credited; the estimate stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **60-67** section passes with
nine personal arrivals, two native scene handoffs, four waiting handoffs, one
lever, two timed holds and **2,971 matching player/world/native states**. Blue
owns the watched Hare scene and Red owns the delayed Port scene. Separate
retained runs verify Red's Hare scene (**1,791 matching states**) and Blue's
Port lever/scene (**378 matching states**). Both stay at 88/78 health, and the
scene checks require hidden/frozen partners, preserved resources and safe shared
reunions. Scripted reunions are recorded separately from walking arrivals.
All **110 Python guards** pass; native source remains `6f589de`. An intervening
startup failed DNS resolution; the next new internet session passed. Later
Clockwork sections, skipped-scene/recovery/reconnect repeats and full traversal
remain open. Overall completion stays **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **44-59** section passes with
32 personal arrivals, 32 independent pathnode projections, 16 waiting handoffs
and **3,565 matching player/world/native states**. Both finish alive at 88/78
health. The parking controller now takes bounded, observed sideways detours
around corners; the previously failed corner at 52 passes through ordinary
movement. A normal checkpoint before 60 is retained. Source `6f589de` passes
**889 active Rust tests**, the locked release, **72 native checks** and the exact
single-player **11+1** baseline; **85 Python guards** pass. This is resumed
section evidence, not fresh traversal or a completed campaign visit. Clockwork's
later machinery/scenes, recovery, reconnects and complete repeats remain open.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **43-51** section verifies 18
personal arrivals, 16 independent pathnode projections, nine waiting handoffs
and **2,634 matching player/world/native states**. Both finish alive at 88/100
health. The requested 43-59 run stops at 52 because its parking selector cannot
take a short sideways detour around a corner. The original failure is retained;
the completed section has a separate scoped audit. Source `6f589de` passes
**889 active Rust tests**, the locked release, **72 native checks** and the exact
single-player **11+1** baseline. The parking fix passes **85 Python guards**.
No additional complete campaign visit is credited.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **36-42** gap sequence passes
through independent internet clients: six personal arrivals, two actual gap
jumps, two preparation holds, one waiting handoff and **232 matching
player/world/native states**. Both finish alive at 100/99.867 health. A normal
checkpoint beyond the gap is retained for the next section. Remote plans now
require supported first landings and retain subsequent jumps; direct airborne
steering preserves enough speed to cross the gap before settling. Source
`a07517e` passes **887 active Rust tests**, the locked release, **71 native
paired/driver checks** and the exact single-player **11+1** baseline. This is
resumed segment evidence; full Clockwork and campaign repeats remain open.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **25-35** section verifies 16
personal arrivals, seven corridor handoffs, two timed holds, one lever and
**1,073 matching player/world/native states**, ending with both alive at 89/100
health. This is a scoped audit of a longer run that stopped at the gap approach.
The next retry exposed a delayed landing failure before that gap. An earlier
braking handoff now passes **886 active Rust tests**, including a regression
that reproduces the old fall. The new live retry verifies **33-34**, including
the previously failed landing, with four personal arrivals, two handoffs and
**237 matching states**. It then fails at 35 with Red falling during a different
jump approach; the gap sequence remains untested. Full Clockwork stays open.
The locked release, 68 paired native checks and exact single-player 11+1
baseline pass, as do **72 Python guards**. Total completion stays **48.6%**,
with six fully verified campaign visits.

Earlier automation result: Clockwork's resumed **18-24** section verifies 12
personal arrivals, all three stair handoffs, one timed hold and **454 matching
player/world/native states**, with both players at 89 health. The leader moves
to the next tread before the follower arrives. The runner also waits for command
acknowledgments retained in its history; **59 Python guards** pass. The requested
18-38 run stops at step 25 while selecting a corridor waiting position. Its
completed section is audited separately, and a normal checkpoint before 25 is
retained. Full Clockwork remains open; total completion stays **48.6%**.

Earlier automation result: Clockwork's resumed gear exit, **step 17**, now verifies
both actual jumps and personal landings, with **245 matching player/world/native
states** and both players alive at 89 health. Route planning now holds a stable
position against gear carry using ordinary controls. The exit test jumps from
the verified gear waypoint because a static walking route can lose support as
the gear turns. Source `cf5d551` passes **885 active Rust tests**, the locked
release, **68 native paired checks**, **50 Python guards**, and the unchanged
single-player **11+1** baseline. The resumed exit is separate from fresh traversal
and does not complete another campaign visit. Overall remains **48.6%**.

Earlier automation result: Clockwork's resumed **14-16** segment now verifies both
players boarding the rotating gear, with two actual jumps, four personal course
arrivals, one timed hold and **447 matching player/world/native states**. The
first player holds its waiting point within 2.76 units while the partner boards;
both remain at 89 health. The test driver compensates for observed gear motion
using ordinary controls. Source `623fece` passes **883 active Rust tests**, the
locked release, **68 native paired checks**, **43 Python guards**, and the exact
single-player **11+1** baseline. The native gear checks include both rider orders
and delayed observations/inputs. The original requested 14-17 run still fails
at the exit with repeated planning; it remains marked failed. This resumed
segment does not extend the fresh traversal or complete another campaign visit.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's fresh **0-15** prefix verifies 24 personal
course arrivals, eight intermediate arrivals, four jumps, one pull-up, two timed
holds, two lever activations and one ordinary partner yield. All **2,997** compared
player/world/native states match, with both players alive at 89 health. The
runner now detects stalled contact even when attempted velocity remains high,
stops the follower, walks the waiting player to an observed clear position and
resumes the follower. **38 Python guards** pass. Native source stays `dbd12c8`.
The requested 0-17 run still fails at step 16 while repeatedly replanning on the
rotating gear; its scoped audit excludes that failed segment. No new complete
campaign visit is credited. Overall completion remains **48.6%**.

Earlier automation result: Clockwork's fresh entrance (steps 0-1) passes with
either player going first on source `dbd12c8`. The two internet runs verify
697 and 693 matching states, each with four course arrivals, eight intermediate
arrivals, four actual jumps and one native pull-up. Both players finish at 89
health. A delayed final-steering correction and precise waiting-footing arrival
address the observed overshoot and blocked pull-up. All 881 active Rust tests,
the locked release, 27 Python guards and seven source-package tests pass.
A longer fresh run independently verifies steps 0-5 and 981 matching states,
then stops at step 6 with both alive: the parked first player blocks the
follower's planned approach. That coordination failure is retained. No additional
campaign visit is credited, and the overall estimate remains **48.6%**.

Earlier automation milestone: Clockwork now has a fresh two-client internet
prefix runner with separate personal arrivals, supported waiting moves, normal
lever inputs and checkpoints, and network-state comparisons. Read-only door
observations let its normal E input work with Clockwork-owned door groups.
Waiting positions prefer the far side of a doorway; parking on the incoming
side had pinned the follower against an open door's tip. Source `f6b1a4f`
passes **60 native machinery/door records**, all **40 scene cases**, and the
unchanged single-player **11+1** baseline. Four new local door cases cover
both actors and the proximity-open starting state, with ordinary partner yield.
The route runner now has 27 receipt/parking/jump/pull-up guards. Source `4fd024c`
adds grounded navigation to the optional test driver and passes 880 active Rust
tests and the locked release build. The component results above remain attributed
to `f6b1a4f`; the new build has not passed the live prefix.
This is additional automation and component coverage, not another completed
campaign visit. Total credit remains **48.6%**. See [scope and retained failures](HATTER1.md).
The earlier fresh retry died at step 1. Observed entrance takeoffs and landings,
ordinary ledge pull-up input, and precise centering now pass a fresh **0-5**
prefix: ten course arrivals, eight intermediate personal arrivals, four actual
jumps, one native pull-up, one timed hold and 872 matching player/world/native
states. Both finish that prefix alive at 89 health. The run then stops at step 6,
where the original single-player marker cannot satisfy the stricter co-op
arrival radius. A repeat passes 0-5 again, then Blue reaches the adjusted step-6
target before Red falls on the approach. The latest run on `4fd024c` connects
over the internet but stops at step 1: Red walks beyond the lower entrance
footing while the test driver is settling the next takeoff position. Its second
jump has not occurred. This is a failed route test, not a hung process or a
completed visit. Failures remain retained; full Clockwork remains open.

Earlier fix: Clockwork's gear barrier and laboratory door now wait for all
living players to pass their authored closing approach. The laboratory lever
alcove does not count as inside. Source `a007099` passes **18 local door cases**
with either first player, paused/compressed waiting-state restore, either/both
players dead and the outside-lever case. All 38 earlier machinery records,
40 scene cases and the unchanged single-player 11+1 baseline also pass.
The gear-door course begins at its open doorway; traversal/waiting on the
rotating great gear remains unverified. Total credit stays **48.6%** and the
completed campaign count is unchanged.

Earlier verification milestone: Clockwork's new native scene suite passes all
**40 cases**: four cinematics with either owner, watched/held-skipped play and
living/dead partners, plus both ordinary speech scenes with either initiator
and living/dead partners. It checks actual contact/lever activation, frozen
waiting positions, pause/compressed restore, camera and authored pose stability,
owner-only skipping, safe separated reunion and personal resource preservation.
Ordinary speech advances once while both living players move independently.
Source `ec48aa7` also passes all 38 machinery checks; the single-player 11+1
records still match the baseline exactly. This adds **0.25 scene/progression
points** for complete staged handoff coverage, not a full campaign visit.

Earlier implementation milestone: Clockwork (`hatter1`) now explicitly advances
one shared machinery clock, carries both living riders, waits for boarding,
and binds lever actions and their delayed scenes to the actual user. Source
`9ce4f98` passes **38 native paired component checks** covering lift and floater
carry, blocked-rider rollback, saved boarding contacts with either requester,
separate sinking pillars, all five levers with either actor, blocked approaches
and owner-specific death cleanup. The original single-player 11+1 component
records match the pre-change baseline exactly. All 879 active Rust tests, the
locked release, source-package seven and source-worktree 890 checks pass.

That adapter added **0.25 movement points**, using the same bounded credit as
earlier native paired-transport milestones. Full paired handoffs are now checked
separately above. Clockwork still needs rotating-gear/exit cases, continuous
two-client traversal, recovery and reconnects. Internet and completed-campaign
credit are unchanged. See [Clockwork coverage](HATTER1.md).

Funhouse's best verified fresh prefix remains **0-88** on source `f9e7029`:
122 personal arrivals, 27 own pickups, 14 paired holds, eight switch shots,
13,767 matching states and four exact ten-second revivals. Both finish at 100
health. The gas-scene fixture correction separately passes unchanged checkpoint
**89-94**, including the watched scene, safe shared reunion, six arrivals, two
holds and 578 matching states. That segment does not extend the fresh-run proof.

The latest fresh repeat with that correction fails at **36 after a team wipe**.
Its independently verified **0-34** prefix has 52 arrivals, six own pickups,
two holds, one switch and 4,788 matching states. The host trace shows both players
restored together at the normal checkpoint with 97 health; full two-client wipe
verification remains open. The failure is retained. No combat rule, resource,
position or team-wipe guard was weakened. Full Funhouse traversal remains open.

Two resumed mirror 29-32 runs also pass, including exact actual-jump evidence.
Conservative range rejection removes distant-enemy aiming work from the test
driver; its measured host p95 cost drops from 39.5 to 1.1 ms per frame on the same
PC. The earlier 0-37 prefix had no control resets; the latest run records one
early suspension. Gameplay physics, combat rules and
the control-reset safeguard are unchanged. **877 active Rust tests**, the locked
release, Funhouse battle63/world46/scene37/single-player8 and Checkmate94 pass;
single-player component output matches baseline except two benchmark timings.

Separate verified work: Funhouse's second personal pad and descent now pass
steps **151-155** with eight personal arrivals, both actual jumps and 396 matching
network states. Each player gains their own second 25-health/Will pickup.
A separate resumed combat run defeats both Tweedles, watches all eight Hatter
scene lines, preserves the hidden/frozen partner and the original floor-collapse
tail, then reaches **Hatter1 with both players alive**. It has 1,178 matching
Funhouse states plus four retained matching states in the new visit. These are
checkpoint segments; the fresh full route and remaining campaign checks stay open.

Recent verified milestones, which do not yet complete another campaign visit:

- Checkmate has separate fresh Blue-led watched-scene and Red-led skipped-scene
  internet passes, a natural team wipe and a guest reconnect pass. Other recovery
  cases remain open. Six native GPU comparisons verify both skins' weapon effects;
  real clients also receive both players' weapon effect types.
- Funhouse's return corridor, steps 35-42, passes 16 personal arrivals and 1,175
  matching player/world/native states, including Red's exact ten-second revival
  at 25 health and each player's own essence pickups.
- Funhouse machinery, steps 46-88, passes 50 personal arrivals, 11 paired holds,
  seven switch shots and 3,633 matching states. Steps 89-94 have separate scoped
  arrival, hold and pickup evidence.

Latest milestone: Funhouse steps **95-103** now have 12 verified personal arrivals,
five personal essence pickups and 1,139 matching player/world/native network states.
Both finish at 100 health, without deaths. The test uses ordinary counterbalancing,
actual native brush-support checks and visibility-aware resident enemy checks.

Latest upper-exit milestone: both players now pass steps **103-109** on source
`88b98f0`, with 12 personal arrivals, two 200-tick holds, two own pickups and
719 matching player/world/native states. A separate resumed **107-117** run
passes 22 personal arrivals, six own pickups and 1,524 matching states, without
new deaths. These are separate checkpoint segments, not a fresh full route.

Source `94488e1`, verifies the narrow descent **111-116**, including
12 personal arrivals, one actual forward waiting move, three own pickups and
1,010 matching states. Red dies naturally and revives after exactly 1,200 active
ticks at 25 health, with safe placement, preserved resources and verified camera
handoff/return. The co-op recovery logger now uses the actual slope-aware placement
rules; its old flat-ground test falsely rejected supported sloped revivals.

The waypoint-117 stall is resolved in the test controller: it now finishes a
nearby clear, supported approach within the existing walking height and arrival
bounds. A separate **116-121** prefix has six personal arrivals, three safe
waiting handoffs, one own pickup and 422 matching states, ending at 100/65 health.

Source `c8f333c` then verifies **121-125**: ten personal arrivals, five safe waiting
handoffs, two own pickups and **1,033 matching states**, ending at 90/30 health
with no deaths. The dodge controller now predicts network delay and braking;
the earlier version could walk the guest off the ledge before taking damage.

A further **121-125** replay passes with **1,079 matching states**, ten personal
arrivals, five safe waiting handoffs and two own pickups. Both finish at 70 health
without deaths. Normal checkpoints before 125 and 126 are retained, allowing
the next crossing test to resume without repeating the earlier fights.

The narrow bank and pendulums now pass as a coordinated **126-134** sequence on
source `c79ac85`: **14 personal arrivals**, all six individual pendulum landings,
both bank jump holds and **1,041 matching states**. Red crosses first and waits
off the landing route before Blue follows. Both finish at 70 health without
deaths; a normal checkpoint before 135 is retained. The final jump waits for a
reachable departure window, including platform movement during network delay.

The next **135-138** segment now passes on the same native source: six personal
corridor arrivals, three supported waiting handoffs, Red's watched Tweedles scene
and safe shared reunion, with **1,092 matching states**. Both observe all nine
scene lines; Blue remains hidden and frozen, then both return at 70 health with
their personal resources intact. A normal checkpoint before 139 is retained.
The entrance step is verified by its scene trigger and reunion: the scene closes
the door before either player can reach the old marker inside it.

A later resumed attempt independently verifies the **139-143 prefix**: ten
personal arrivals, five supported waiting handoffs and **505 matching states**,
ending at 60/70 health without deaths. It then wipes at 144: Blue dies while
waiting below the climb, and Red dies before Blue's revival. A normal checkpoint
before 144 is retained; the complete 139-146 run failed.

The climb now passes as a coordinated **144-146** sequence: six personal
arrivals, two observed following footings, one safe final waiting handoff and
**203 matching states**. Both move and jump normally, with the follower keeping
space while the leader brakes. Neither dies; final health is about 6.8/59.5.

A subsequent **147-150** replay passes both actual pad jumps, four personal
bridge arrivals, the paired 900-tick hold and **479 matching states**. Jump
durations are measured after the game acknowledges each actual jump, excluding
command-delivery delay. Each player collects their own arena essence: each gets
25 health and Will, with no shared consumption or partner healing. Both survive
at about 31.8/38.1 health. A normal checkpoint before 151 is retained.

Next are the opposite pad jump and descent from **151**, the remaining boss
fight, later scenes and exit. Fresh full traversal and the opposite crossing
order remain unverified. These segments add no completed campaign row.
Overall stays **48.1%**.

Tests run as two independent clients through the public network path on Anode,
with injected delay and jitter. Routine diagnosis uses state and log assertions.
Screenshots are not needed for each run. The current source also has 875 passing
active Rust tests and native Funhouse battle, machinery and scene checks; those
component results do not prove a full campaign route.

See the [fixed weighting and detailed history](COOP.md#delivery-and-verification-weighting),
[campaign matrix](COOP_CAMPAIGN.md) and [Funhouse evidence](FUNHOUSE.md). Future
updates should state the overall percentage, the new bounded milestone, and the
next unresolved acceptance check even when the percentage does not change.
