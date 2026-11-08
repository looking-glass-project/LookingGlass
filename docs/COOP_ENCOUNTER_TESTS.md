# Short co-op encounter reproductions

`--coop-encounter-check` loads a captured authoritative report into the production
combat simulation, then supplies ordinary player inputs. Use it for repeatable
combat, movement and checkpoint investigations before another full route run.
It does not reproduce original network delivery or establish full-level support.
Original assets and captured reports belong in ignored `private/`.

Set `LOOKING_GLASS_COOP_ENCOUNTER` to a fixture JSON file and run the check in the
background desktop with `--data <local-base> --no-audio --profile <private-folder>`.
The fixture names a report relative to itself and an optional JSON pointer:

```json
{
  "report": "failure-report.json",
  "pointer": "/previous_reports/host",
  "cases": [
    {"name": "neutral baseline", "ticks": 240, "expected": "team_wipe"},
    {"name": "move apart", "ticks": 240, "expected": null,
     "players": [
       [{"start": 0, "ticks": 80, "wish": [1, 0], "run": true}],
       [{"start": 0, "ticks": 80, "wish": [-1, 0], "run": true, "jump": true}]
     ]}
  ]
}
```

A direct `coop-live.json` or Garden `encounter-*.json` uses an empty pointer.
Each tick is 1/120 second. Inputs are ordered, non-overlapping intervals; a jump
occurs only at the interval's first tick. Unspecified controls are neutral and
preserve the player's view direction. Health, positions, inventory and enemies
are restored exactly once at the start and are then owned by normal gameplay.

Every case runs twice and must produce identical recorded outcomes and final
state hashes. Expectations are `team_wipe`, `both_alive` (neither player dies),
`one_dead` (one player dies without a team wipe), or `revived` (at least one
player revives exactly 1,200 ticks after death, without a team wipe). A null expectation prints
`OBSERVED`, never `PASS`. A wipe must restore the complete encounter checkpoint
and both players' exact saved resources. Scenes and level transitions are outside
this short check's scope and fail explicitly.

The isolated profile receives `coop-encounter-result.json`, with the tested
build, captured build, report and fixture hashes, health/movement samples and
each assertion result. An assertion failure returns a failing exit code after
saving the result. Even a passing reproduction of a known wipe is not a passing
clean route; the result explicitly retains `full_route_passed: false`.

For focused diagnosis, `"capture_final": true` also retains the final player
frame and combat snapshot in each outcome. This lets a script check enemy
health and weapon emissions after an input experiment. Those snapshots must
match across the identical repeated runs; the option does not add a gameplay
assertion or change a null expectation into a pass.

## Independent-client recovery segments

An optional `"export_checkpoint": true` fixture field saves the captured current
visit through the normal checkpoint store. It requires a host report with zero
completed campaign visits and a fresh isolated profile without a checkpoint.
The exported checkpoint retains exact current players, resources and combat;
it does not reconstruct omitted campaign history. Never use a personal profile.

`tools/test_coop_rope_revival.py --exe <tested-exe> --data <local-base>
--checkpoint <exported-checkpoint>` starts two independent clients through the
public internet transport, with 40 ms delay and 20 ms jitter by default. Inputs
remain neutral, allowing the captured falling player to die naturally. It checks
the spectator camera, exact ten-second revival, 25 health, separate valid rope
grips, personal items and matching player/world/enemy state. Use `--victim blue`
for a capture in which Blue dies; the default is Red. No screenshots are requested.

Native replay records every revival and camera/life-state change. Its resource
comparison excludes world loot, whose drop lifetimes continue while a player is
dead; the focused Roster regression checks exact full resource preservation at
the respawn boundary. Both roles are covered by that regression, including 120
subsequent movement ticks. Placement rejects blocked, hazardous, disabled and
controller-managed ropes. The fallback supports active native ropes only;
custom level-controlled ropes still need their own recovery checks.

## Garden evidence

Build `bad5ddf53efc4c7ce039ddea3f3319bc387e810b60fe4f5f668416e5a0416d39`
repeats the captured late grenade state with neutral controls and four movement
alternatives. All five reproduce a team wipe in four or five simulation ticks;
both exact player states/resources and the full combat checkpoint are restored.
Every outcome matches its independent repeat. This shows that this late capture
has too little reaction time for those inputs; it does not prove that earlier
avoidance is impossible. Evidence is retained in the private
`coop-encounter-bad5ddf-late` profile.

A separate internet run ended while Red climbed the first rope. Its last valid
state produces a 31.999992-unit grip after two ordinary movement ticks, below
the 32-unit save/network minimum. The focused regression fails before the fix.
The rope solver now bounds the recomputed grip distance, preserving strict
validation. From the original captured state, a 120-tick climb passes twice with
every frame valid and both players alive at their original 98.344/100 health.
Evidence is in `coop-encounter-bad5ddf-rope`. No enemy, health, inventory or
traversal state is edited to obtain these results.

The same build passes all 801 active unit tests (one ignored) and all four full
native paired Garden routes. Blue watched/skipped finish at 31,311/24,427 ticks;
Red watched/skipped finish at 29,541/23,713 ticks. Completion times, replica
counts and personal resources exactly match the previously verified routes.
This preserves the native traversal milestone while internet traversal and the
rest of the Garden recovery matrix remain open.

The subsequent independent internet run passes the rope section, Rabbit scene
and early combat clear, then fails when Red falls at step 72 while Blue reaches
step 73. All 2,515 common player, world and native-enemy states match. It retains
an earlier combat snapshot for further investigation. Evidence is
`private/coop/garden-route-6263674b01254359bc36a0f36df8838f/`.

Replaying its last living pair with neutral controls reproduces Red's fall at
tick 16,926. The countdown reaches `AwaitingSafeSpawn` while Blue hangs over the
pit; no safe nearby placement is found. Blue then dies and the encounter
checkpoint restores at tick 18,167. The expected single-death case correctly
fails, saves its result and returns a nonzero exit code. This is an open recovery
case, not evidence of a completed safe respawn. Its two identical runs are saved
in `coop-encounter-bad5ddf-fall`.


## Native-rope recovery fix

Build `10063748f92ba15532639080045f3d10b9ff4e55b80c202a6f76427b52ee4a92`
adds a clear, separate grip on the survivor's active native rope when nearby
supported ground or water is unavailable. The survivor stays in place. Candidate
grips retain the survivor's angular motion and pass full body-sweep, hazard,
rope-length and saved-state validation. No safe candidate means continued spectating.

Two independent public clients pass the captured natural fall with 40 ms delay
and 20 ms jitter per sender. Red dies at tick 16,926 and revives at tick 18,126
with 25 health, about 72 units from Blue on rope 353. The guest camera follows
moving Blue during the countdown and returns to Red. Personal items are retained;
142 common player, world and native-enemy states match. Both clients close
normally without requesting screenshots. Evidence:
`private/coop/rope-revival-4e8ad303ecd349b2b8cdf652de4ba69d/`.

The first native replay assertion incorrectly compared world-drop timers across
an ordinary simulation tick as if they were carried inventory. That failed run
is retained in `coop-encounter-1006374-revival`; its isolated checkpoint export
passed. The corrected comparison excludes these advancing world drops, while
the Roster regression checks exact full Stats preservation at the spawn boundary.
An attempted Blue-victim internet repeat in `rope-revival-ca46d00d6e914a218d7edcc419da5567`
failed before gameplay with HTTP 530. It supplies no Blue-role gameplay evidence.
This is a recovery-segment milestone, not a full Garden traversal or campaign pass.

The corrected replay on source
`e9c87b1f2fb5a8bb81d8f54f239e91fbefadad5f28eed1c56038338fb2e56285`
passes the 1,210-tick fall/countdown/revival case twice with identical outcomes,
including valid placement and exact carried resources at the spawn tick. This
source differs from the internet-tested build only in the replay's resource
assertion. Evidence: `private/coop-encounter-e9c87b1-revival/`.

The longer neutral-input survival experiment remains a failed case. After Red's
successful revival, Blue dies at tick 18,167. Red takes two 10-health hits, loses
the grip, then dies at tick 18,970 before Blue's countdown finishes. The team
checkpoint restores exactly. This does not establish safe continued idle survival
under attack or a Blue-victim internet pass; its failing expectation and nonzero
exit status remain recorded. Full traversal/combat work is still required.


## Garden traversal segments

The Garden fixture captures host reports at steps 69, 71, 72 and 75. Export an
isolated checkpoint from one of those reports using the encounter fixture above,
then run:

```powershell
python tools/test_coop_garden.py --exe <tested-exe> --data <local-base> --checkpoint <isolated-checkpoint> --start-step 71 --end-step 84 --first blue --scenes skipped
```

The start/end steps are explicit test controls and must match the captured
position. The host restores the unmodified checkpoint and replicates it to the guest.
Both keep ordinary combat and traversal, and stop after passing the requested
course point.
The result retains the checkpoint, executable, embedded course and Python fixture
identities, measured input lead, health and matching network-state counts. It
always records `full_route_passed: false`. Omitting checkpoint/start/end continues
to run and verify the full original entrance-to-Garden2 route.

See the Garden segment evidence and the separate unresolved same-rope overlap
case in `COOP.md`; traversal success alone does not prove collision or complete
campaign support.


## Both rope countdown reconnects

`tools/test_coop_rope_countdown.py` resumes the unchanged Blue-on-rope/Red-falling
checkpoint and exercises both deaths sequentially through ordinary gameplay.
The survivor climbs and attacks while the other player is dead. A fresh guest
must inherit the remaining countdown and rope attachment. Exact host recovery
traces are mandatory in this moving case: a later client sample can legitimately
have a smaller gap because the survivor has continued climbing after revival.
The original neutral fixture retains its fixed-distance assertion.

Run the automated public case in Anode:

```powershell
python tools/test_coop_regressions.py --exe target/release/looking-glass.exe --data <local-base> --rope-checkpoint <unchanged-exported-checkpoint> --only rope-countdown --delay-ms 40 --jitter-ms 20
```

Use `--local` only for a loopback baseline. The input checkpoint must contain
living Blue on a native rope and living Red without a grip; this test does not
swap players or alter the save. On build `c5cb7fc`, the public case passes both
exact 1,200-tick, 25-health revivals, safe separate grips, both rendered cameras,
full first-frame agreement and 290 matching body/world/native states. See
`COOP_PROGRESS.md` for the retained passes and failed preparation attempts.
This recovery segment does not establish full Garden traversal.

## Moving-platform countdown reconnects

`tools/test_coop_fulcrum_countdown.py` resumes a normal, unchanged Funhouse
checkpoint with both players on the second fulcrum. Each player runs off its
side in turn. The survivor tips the board using ordinary walking, and the owned
guest process is restarted during each countdown. An exact host trace verifies
ten seconds, 25 health, personal resources and safe supported placement. Both
client views must show the correct spectator camera and separate bodies.
The board must be moving across each spawn boundary, and each revived player
must remain supported and travel with it while receiving neutral input.

```powershell
python tools/test_coop_regressions.py --exe <tested-exe> --data <local-base> --fulcrum-checkpoint <unchanged-fulcrum-save> --only fulcrum-countdown --delay-ms 40 --jitter-ms 20
```

Public invites are the default; `--local` is a separate baseline. The fixture
uses measured round-trip time to anticipate its ordinary steering commands.
It rejects an unintended survivor death, extra revival, stationary support,
driven carry and mismatched first-frame state. Source `c5cb7fc` passes both roles
over the internet with 426 matching body/world/native-enemy states. See
`COOP_PROGRESS.md` for the pass and retained failed preparation attempts.
This component does not complete Funhouse or another campaign visit.
