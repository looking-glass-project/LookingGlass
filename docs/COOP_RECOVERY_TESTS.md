# Recovery during route tests

`tools/test_coop_checkmate.py` keeps its strict zero-death mode by default.
`--allow-revival` runs a separately labelled route that permits individual
revival only after verifying the actual host simulation and both client views.
`--require-revival` additionally requires a real death and completed revival;
a route without a death cannot then pass as recovery evidence.

Run the two headed clients in Anode, using a build containing the recovery
trace. Both processes have independent profiles and use the real internet path
unless `--local` is explicitly selected. For example:

```powershell
python tools/test_coop_checkmate.py --exe target/release/looking-glass.exe --data C:/path/to/base --allow-revival --require-revival
```

The existing checkpoint/segment options still work. A resumed run remains
partial even if it reaches the level exit. The harness sends ordinary movement,
weapon and menu checkpoint requests; it does not alter bodies, health, enemies
or progression. Team wipes still fail this route mode. The separate checkpoint
recovery fixtures must prove shared restoration; no silent restart is allowed.

The host's opt-in `recovery-trace.jsonl` records each simulation tick during a
death, plus exact death/respawn resource and geometry snapshots. Healthy play
records one heartbeat per simulation second. Serial numbers and build identity
detect missing or mismatched evidence. Recording uses immutable game references,
does not add fields to network frames or saved games, and refuses to overwrite
an existing trace. A write failure is reported to the harness without ending
normal gameplay. Ordinary players and strict tests do not enable this trace.

The verifier requires 1,200 countdown ticks, accounts explicitly for cinematic
pauses and waiting for a safe spawn, checks the exact initial 25 health and
unchanged personal resources, and rejects a blocked, hazardous, unsupported,
overlapping or distant spawn. It separately checks the dead player's rendered
camera follows the survivor and returns after revival. Later damage need not
leave the next sampled client report at exactly 25 health: that value is proved
at the exact host spawn tick.

New builds report both drawn player positions, the previous presentation state
and the frame duration. The recovery camera check verifies the exact native
interpolation from these inputs and follows the drawn player. This permits
normal visual smoothing behind a newer network position without treating an
arbitrary distance as proof. Death, revival and scene-warp stamps must cut that
interpolation. Older captured reports retain their original distance check.

On Checkmate's northern course, a revived Alice may legitimately appear beside
a partner who is farther along. The result records her skipped waypoint IDs and
requires that the surviving partner actually traversed each one. Every skip
must match a verified native revival. It never counts those waypoints as a
personal traversal by the revived player, and cannot skip untraversed team
ground. The result's mode and personal coverage remain explicit.

Initial observer rollout on 28 October 2026: the native trace observer's read-only/exact-tick
unit check passes within all 856 active units (one existing ignored test).
Python guards cover bad clocks, missing events, unsafe placement, changed
resources, camera failures and invalid waypoint coverage. Funhouse's 63 battle,
35 machinery, 37 scene and eight single-player component records still pass;
its single-player gameplay output is unchanged except measured benchmark times.
The recovery-aware Checkmate northern checkpoint replay now passes in Anode
with two independent internet clients, 40 ms input delay and 20 ms jitter.
Source `df108e9`, evidence `checkmate-recovery-route-101c78ca689f4175855901a8dcbc9eaa`:
Blue traversed waypoints 33-44, Red traversed 33-38, then died at tick 27600
and revived at tick 28800 with exactly 25 health. Her spectator camera followed
Blue and returned to Red. Safe placement, unchanged carried resources, and all
three network-state comparisons passed. Red's remaining six waypoints are
explicitly recorded as carried by revival beside Blue, who traversed them.

The first PC fallback attempt is retained as a failure: its obsolete movement
goal tried to send Red back to an unreachable upper platform after revival.
The runner now clears that command during the countdown and assigns the next
goal only after checking the real spawn stamp and survivor's route coverage.
The resource verifier follows the existing encounter check: carried resources
must stay exact, while uncollected world-drop records can change as the arena
continues updating. This includes normal drop lifetimes and new enemy drops;
those records are not carried inventory.

All 56 Python co-op guard tests pass, including stale-goal handling and rejection
of altered personal inventory or will. This is a resumed segment, not a full
Checkmate route or a team-wipe test. It adds no completed campaign visit or
percentage credit. Overall progress remains approximately 48.1%.

The later Checkmate shared-wipe fixture is `tools/test_coop_checkmate_wipe.py`.
It resumes an unchanged northern checkpoint, sends ordinary movement without
attacks, and checks an actual combat wipe. Its separate trace verifier requires
one common spawn tick, exact saved positions, full personal resources and safe
geometry. Both clients must observe the death and restore, including native
piece health and shared progression. Generation one is accepted only by this
explicit wipe check; ordinary route presence checks still reject a wipe.
On source `e11e75c`, checkpoint tick 23611 restores at 25055 with 613 trace rows
and 229 matching player/world/native states. Evidence is recorded in `RCHESS1.md`.

`tools/test_coop_checkmate_rejoin.py` resumes the checkpoint after the first
scene, gracefully exits the guest, starts a fresh guest process on the same
invite and continues through the King and Funhouse. It checks unchanged personal
resources/positions, completed scenes and native piece health, then requires
at least 20 matching states after reconnect in each of the three state domains.
This is a graceful-reconnect fixture; it does not claim abrupt network-loss
coverage. On source `e11e75c`, the Blue-led case passes through Funhouse with
3,410 matching post-rejoin states and another exact ten-second, 25-health Red
revival. The shared recovery guard suite now has 64 passing tests.
