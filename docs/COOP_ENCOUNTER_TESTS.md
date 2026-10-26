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
or `one_dead` (one player dies without a team wipe). A null expectation prints
`OBSERVED`, never `PASS`. A wipe must restore the complete encounter checkpoint
and both players' exact saved resources. Scenes and level transitions are outside
this short check's scope and fail explicitly.

The isolated profile receives `coop-encounter-result.json`, with the tested
build, captured build, report and fixture hashes, health/movement samples and
each assertion result. An assertion failure returns a failing exit code after
saving the result. Even a passing reproduction of a known wipe is not a passing
clean route; the result explicitly retains `full_route_passed: false`.

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
