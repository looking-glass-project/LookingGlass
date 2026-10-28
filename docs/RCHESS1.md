# Red King — Checkmate in Red

The `rchess1` controller restores the Red King battle and the surrounding visit.
It reads models, animations, map markers and camera paths from the local data;
reviewed Rust controls the encounter. Original scripts and AI programs never execute.

## Playable behavior

- The castle's ten ambush groups use the existing pawn, knight, rook and bishop
  combat, including group-five suppression, reactions, deaths and one-time rewards.
- The White Queen execution uses the original cameras, separate head/body models,
  blade movement and spectator departure. The upper route and arena drawbridge
  remain available at their appropriate points.
- The King appears during his introduction, then fights with 1,300 health,
  a damage-once melee strike, four-pulse beam, bouncing explosive balls,
  diamond shots and an incoming-projectile counter with a seeking ball.
  Emission uses animated `tag_ball` and `tag_barrel` transforms.
- Pain reactions, directional deaths, frozen death, a boss meter and the three
  cycling medium essence positions are supported. Dice allies are dismissed
  for the arena; the shared Watch slowdown remains effective.
- Defeat waits for the King's death and shrink before bringing back the platform,
  transforming the pawn into the Queen, revealing the royal pieces and showing
  the Hatter's strike and the fall into `funhouse$funhouse_start1`.
- Scenes can be watched or skipped. The arena writes a checkpoint request;
  reload preserves battle state. Exit requests are committed once and retry
  safely if destination loading fails.

## Persistence and bounds

The controller's version-one payload is inside the existing save envelope. It
stores phases, scene clocks, bridge/platform state, all chess receivers, boss
health/action, recoil, shield recovery, projectiles, essence position/cooldown
and exit status. Restores reject malformed or inconsistent state before mutation.
Boss simulation runs at 120 Hz with at most 16 shots and four beams. Finished
shots and their effects expire; death clears live hazards. Pause and dead-player
states do not advance the encounter.

Pre-controller saves use the shared visit upgrade path and restart this visit's
new encounter state at a safe entry. Previously defeated generic chess receivers
are not imported into the new controller; start a fresh visit for this playtest.

## Fidelity limits

The original AI's descending `HEALTH` tests are interpreted as lower bounds:
beam at 1,000–1,300 health, balls at 500–999, diamonds below 500. This is a
compatibility choice that makes all three authored branches reachable; native
condition semantics have not been confirmed. Local steering, random attack
selection, pain cooldown, shield direction/window and projectile steering are
bounded approximations. Bodies are non-solid to Alice. Explosion radius follows
the shared damage-plus-60 convention, with cover checks and no doubled direct hit.

The beam uses a depth-tested line/core effect rather than the original beam
shader. Models and attached effects use the production material queue. Spectator
exits, royal-piece movement, Alice's walking and intermediate camera fades are
approximations. The Hatter uses his authored portal mark after unbinding so his
strike reaches Alice. Optional asylum scenery is rendered, but this is not a
complete restoration of every optional scene or original head-tracking behavior.
Audible mixing has not been verified; native checks run without audio in Anode.

## Verification

`--rchess1-check` checks source tags/clips, hidden/dead target gates, attacks at
30/60/144 Hz, pause, restore, health stages, melee contact, counter activation,
frozen death, invalid saves and the one-shot exit.

`LOOKING_GLASS_KING_ARENA=1 --rchess1-route-check` starts at an explicitly
staged bridge entrance (set the environment variable before running the flag).
`LOOKING_GLASS_WATCH_SCENES=1` watches scenes instead of skipping. Both variants
pass and are only evidence for the boss segment. They defeat the King with ordinary
Blade/Cards inputs, taking real damage and collecting essence, then arrived
safely in the Funhouse with all four ranged/counter branches used (95 sanity left).

The separate `--rchess1-route-check` is an incomplete full-visit replay. It reaches
the swim, rope climb, third Demon Die, execution, and northern castle route through
normal movement and combat. It currently dies during the northern rook fight;
a version that runs past the rooks instead dies on the final drop with one sanity.
The full approach is therefore **not verified as an uninterrupted playthrough**.
These failures are retained for follow-up instead of refilling resources or disabling
enemies. Private movement variants can use `LOOKING_GLASS_KING_ROUTE_INPUT`; those
files only express navigation, movement and fighting, never world-state edits.

`--rchess1-render-check` captures twelve staged native views, including the
execution, bridge, attacks, frozen death, Queen and Hatter ending. Setting
`LOOKING_GLASS_KING_SAVE=write` then `read` in separate native processes tests
seven real Store snapshots from approach through committed exit. These are
explicit fixtures, not claims of uninterrupted gameplay. Evidence is local in
`private/rchess1/`.

Use `tools/launchers/Launch-Red-King.cmd` for the installed private candidate and separate
playtest saves. The default player launcher and existing saves are untouched.

Validation on 30 September 2026: 627 unit tests pass, including four new King
regressions; the shared chess audit passes. Native captures and all seven
separate-process Store cases pass. Audio was disabled.


## Co-op component validation, 28 October 2026

Frozen source `26b913484fee7be2cba4ca7a06ee98b4ec1accb408923c99873ab813783388cb`
adds one shared Checkmate clock and two personal healing cycles. The King and
the native pawn, knight, bishop and rook select visible living players; melee,
beam and projectile contacts damage the body they hit. Saved attack ownership
survives restoration. Explosions can reach both players without double-hitting
the direct victim, respect walls, and ignore dead bodies. A seeker retains its
saved target and becomes ballistic if that player dies.

The three original scenes retain one actor. Trigger contacts bind the first
arrival; the victory scene binds the nearest living survivor after combat
damage has been applied. Entering that scene during combat would allow the next
shared event to bind the previous speaker first, so activation and ownership
now happen together in the contact phase. The other Alice remains frozen and
hidden until the existing safe reunion or Funhouse transition.

`--rchess1-coop-check` passes **84 records** against the original local map and
animations: 40 target-selection cases; eight chess-piece and four King attack
cases; two personal-healing cases; three clock-rate cases; legacy/rejected-save
and dead-team-victory guards; 24 scene combinations; and the aggregate result.
Scenes cover both owners, watched or owner-held skipped, living or dead partner,
pause and save restoration. The waiting partner cannot skip the owner's scene.
Per-view healing presentation uses that player's saved location and cooldown.

Four additional unit regressions cover projectile body interception/splash,
walls/dead bodies, seeker target restoration and legacy/invalid target saves.
Both complete single-player logs, `--rchess1-check` and
`--rchess1-route-check`, match the pre-adapter baseline exactly. The latter now
passes from the normal entrance through the King and into Funhouse; the older
September route failures above remain historical evidence. The broader
82 shared-world and 43 campaign entrance/recovery checks also pass. All 847 active
unit tests and the release build pass; one existing unit remains ignored.

These co-op checks use staged actors and contacts. **A continuous two-client
Checkmate playthrough is still unverified**, as are its public-internet,
recovery/rejoin and complete rendered-effects gates. They add 0.25 combat and
0.25 progression points to the weighted estimate, with no completed campaign row.
Evidence remains local under `private/coop-rchess1-build.json` and
`private/coop-rchess1-native59.log`. No screenshots were needed for these checks.

## Independent-client automation follow-up

The first public-internet attempt passed its first approach waypoint, then Blue
died beside a pawn while waiting for Red. The pawn remained at 75 health: the
generic test driver omitted targets owned by native campaign controllers. Its
attack controls now include those targets and retain non-hostile bodies for
occlusion without selecting them for attack. Gameplay damage and resources are
unchanged. The retained failure is not a full-route pass.

Frozen source `fe1505f81cabc4ad87fe2f5069e80e4114363e4796dfa78fef991f4065e7cf72`
passes 94 native records. Ten added checks exercise the same combat-control
function used by the independent clients against the original King and four
chess families, from each player slot. They require normal Cards aim/contact
and no firing through an intervening wall. All 847 active units, the release
build, both unchanged single-player logs, 82 shared-world checks and 43
campaign entrance/recovery checks pass. One existing unit remains ignored.
These diagnostics add no campaign completion credit. A fresh full two-client
replay is pending. Evidence: `private/coop-rchess1-native62.log`.

## Replaying the two-client approach

`tools/test_coop_checkmate.py` drives two independent profiles through normal
Host/Join invites, with public internet transport by default and 40 ms delay /
20 ms jitter. It retains commands, exact build identity, executable and fixture
hashes, normal checkpoints and state comparisons without opening screenshots.
A failed run cannot emit a full-route pass.

The optional `--checkpoint PATH --start-step 7 --last-step 10 --save-before 10`
mode restores the unchanged pre-rope checkpoint and verifies the native exact
resume receipt before replaying that segment. Segment results explicitly say
`full_route: false`; fresh entrance-to-Funhouse traversal remains a separate
requirement. Without checkpoint/segment options, the runner attempts all 45
approach steps, the original scenes, King fight and Funhouse arrival.

Rope controls use independent per-player phases. The follower swims to the base
before grabbing; the leader completes its own climb, jump-off and landing
without waiting underwater for the follower. The diagnostic input format accepts
an optional bounded `wish` vector while climbing and an optional `jump` on
release. Both default to the previous behavior. The game still handles the
actual pendulum, jump, collision, damage and resource consumption.

## Verified rope segment and remaining limits

Frozen source `e7dc397035d7605e7ea953c78902c2c13b4786153bb1f242a6e2247bd9ce67c9`
passes 848 active unit tests (one existing ignored), the release build, all 94
Checkmate native records, 82 shared-world and 43 campaign entrance/recovery
checks. Both complete single-player logs remain identical to baseline. The new
rope regression uses real movement physics: swing from a vertical strand,
then require a jump-off to reach an elevated platform where a plain drop fails.

The two-client Blue-led local replay now passes steps 7 through 10 with 40 ms
delay and 20 ms jitter. It restores the unchanged checkpoint from the earlier
public-internet water approach, checks the native exact resume receipt, and
uses ordinary swim, grip, climb, jump, ledge pull-up and movement inputs. Both
players land with 92/100 health, remain clear of solid geometry and each other,
and reach the onward waypoint. All three replicated state hashes match at
243 sampled ticks. A normal checkpoint is saved before step 10. The result is
explicitly `full_route: false` and `internet: false`.

Earlier attempts are retained as failures: an enemy hit interrupted a pull-up,
a follower re-grab needed another deliberate release, and waiting positions
blocked the onward goal or met the original map geometry. The final runner
parks players along the platform centre with enough clearance. No collision,
health, enemy strength or progression checks were relaxed.

Two public-internet rope replays failed during invite hostname resolution,
before gameplay. They do not establish a traversal failure or an internet pass.
Internet retry/reliability, the full fresh route in both arrival orders, recovery
and rendered-effects coverage remain open. This segment adds no campaign-row
or percentage credit. Local evidence: `private/coop-rchess1-native65.log` and
`private/coop/checkmate-route-4be55339a1224afc92509f1925227e52/result.json`.

## Public-internet rope replay

Frozen source `8beb04fa32c42d17c9bd7e8af72e058a8eb794032feb507be41ef0a1eee356df`
adds bounded, cancellable DNS retries to the guest connection worker. All 851
active units, the release build and the same 94/82/43 native check groups pass;
both single-player logs remain identical. The DNS regressions exercise failed
and empty answers followed by a real reachable listener, persistent failure,
and cancellation during the retry wait or a lookup that returns an address.
A fresh relay probe also resolves through both the system and provider resolver
at 12 seconds and returns the expected HTTPS response; this does not establish
the cause of the two older invite-resolution failures.

The actual public-internet replay of the unchanged pre-rope checkpoint passes
steps 7-10, with 40 ms delay and 20 ms jitter, 92/100 health, body clearance and
240 matching player/world/native hashes. Its normal pre-step-10 checkpoint
supports the next approach replay. It remains a segment, not a fresh full-route
pass, and adds no campaign completion credit. Evidence:
`private/coop/checkmate-route-9f0cbdef6ec04beb9867f74afa856652/result.json` and
`private/coop-rchess1-native66.log`. Further traversal/recovery remains open.

## Visibility-aware combat waits

The next internet replay restored the normal upper-platform checkpoint and
passed step 10, but its proximity-only clear predicate then waited for Knight
464 on the lower floor. Both players stayed alive at 92/100 health; the run was
stopped with its reports retained. A nearby enemy behind the platform cannot
be cleared by standing still above it. The original single-player route also
uses occlusion when deciding which threats to fight.

The opt-in diagnostic report now includes each player's nearest visible native
hostile within 700 units. It uses native hostility and the same aiming/occlusion
context as ordinary attack controls, retaining other bodies as obstructions.
The runner requires a current acknowledged report from each client before
ending a combat wait. It does not change enemies, damage or progression.

Frozen source `d12189f270267e3ad5d5f9ebd21c1d1d2c617f943b49dafbb2df54e91a26192c`
passes all 851 active units, the release build, 94 Checkmate native records,
82 shared-world and 43 campaign entrance/recovery checks. The ten native driver
cases now also require a visible threat, no threat through a wall, and no threat
outside the clear radius for the King and four chess families from either slot.
Both single-player baseline logs remain identical.

The subsequent public-internet replay passes steps 10-26 from the unchanged
normal checkpoint, with 40 ms delay and 20 ms jitter. Both clients pass presence
and body-clearance checks and match all three state hashes at 3,444 ticks.
Blue's lowest health is 80 and Red's is 100; normal pickups restore both to 100
at the finish. The original beheading scene is observed in 110 host and 112
guest samples, always owned by Blue. Red stays still and hidden during the
scene, then both players reappear alive. A normal checkpoint before step 12 is
retained. The runner now explicitly verifies scene ownership, waiting-partner
visibility and reunion when that segment includes the approach to beheading;
that verifier also passes against the retained run's complete history.

Evidence: `private/coop/checkmate-route-4c37fcefa3cc4e2b9872c45b3dae7d16/`,
including `result.json` and `beheading-verification.json`. This is still a
checkpoint segment, not a fresh entrance-to-Funhouse pass. The later approach,
King fight, opposite arrival order, skip/recovery and broader effects checks
remain open. No campaign-row or percentage credit is added.

## Fresh-route camera visibility regression

The first fresh Blue-led internet route on `d12189f` passes steps 0-25 but fails
the new reunion verifier at step 26. Both players are alive at 100 health and
48 units apart after the completed beheading scene. In Blue's view, Red's body
is only 46.86 units from the follow camera; the renderer's 48-unit close-camera
cutoff incorrectly hides Red even though there is no active scene. This is a
rendering failure, not evidence of overlapping physics bodies or a route pass.
The captured run remains under
`private/coop/checkmate-route-488409831b76417ea86b3b754647e3f2/`.

Close-camera hiding now applies only to the watched player. The other player's
body retains its normal draw eligibility, with cinematic and native scripted
hiding still respected. A regression uses the captured camera and body positions
and preserves own-body hiding for cramped or spectating cameras. The route
verifier checks the remote player in each client's view after reunion and
throughout free Checkmate play. Its own body may legitimately be hidden by the
follow camera. Rechecking the retained histories still accepts the earlier
segment and rejects the fresh run's actual remote disappearance. All 852 active
unit tests pass. The next build also passes the release build, 94 native
Checkmate records, unchanged single-player baselines and the 82/43 broader
checks. A fresh internet attempt passes the rope section and all observed
free-play partner visibility checks, including 20 host and 30 guest samples
with the other body within 48 units of the camera. It then disconnects during
the pre-step-12 checkpoint operation, before reaching the reunion again.


The retained disconnect is under
`private/coop/checkmate-route-15bdcf3bceb94e119bd1844be841d475/`. The host reports
that incoming updates were not consumed quickly enough; the transport used a
two-second queue deadline. The save file was committed at tick 5758, but the
runner had not yet received its acknowledgement. Exact save duration was not
recorded, so a slow filesystem write is not established as the sole cause.

The transport now gives a blocked consumer the existing 15-second connection
grace, retaining the same four-message channel and one held incoming message.
A three-second stall regression requires all eight packets to arrive in order
and the connection to remain usable. A persistent stall still disconnects within
the bounded timeout. Diagnostic checkpoint acknowledgements now include
`save_ms`; save format, validation and disk-before-memory atomicity are unchanged.
All 852 active units pass, including both backpressure cases. Frozen source
`999049c05976d6187c24eacaabf78c9a302ad8162a3522511a32f137ced51c5e` passes the
release build and the same native and single-player checks. Its fresh full
internet replay passes steps 0-33, including the beheading/reunion and later
platforms, before Blue dies during the step-34 ambush. The failed run is retained;
no completed campaign row is claimed.

For focused later failures, `--through-exit` with final approach step 44 lets
an unchanged checkpoint replay continue through the King and Funhouse, while
verifying the remaining scenes. Such a result always remains `full_route: false`.
Only a fresh normal-entrance run can produce a full-route pass. The new checkpoint
exit mode has not yet completed a real boss/exit replay.


The source-69 fresh run is retained under
`private/coop/checkmate-route-977e10f98a6744a2aa0d7bbbe2920a1d/`. The render
assertion passes all observed free-play frames, including 103 host and 23 guest
near-partner camera samples. The Blue-owned beheading is observed in 119/118
samples with a hidden, stationary partner; both reappear at 100 health. Normal
checkpoints before steps 7, 12 and 27 take approximately 49, 37 and 19 ms. These
fast saves do not reproduce the earlier unexplained stall.

At step 34, Blue is cornered near two active Pawns and dies; Red remains at 100
health farther back on the route. The retained pre-death state has Blue at 20
health and 0.56 Will, with a walking controller replanning around the corner.
The diagnostic grounded-route controller currently walks rather than runs,
while the authored single-player course keeps moving past these optional
northern ambushes. Movement speed, coordination and attack tactics need review;
enemy damage and death checks remain intact. The normal pre-step-27 checkpoint
and the failed reports allow a focused replay. `prefix-verification.json`
records the scene checks separately and explicitly retains the death and
`full_route_passed: false`. The King, exit and fresh full traversal remain open.
