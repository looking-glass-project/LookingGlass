# Automated core save and recovery checks

`tools/test_coop_core.py` runs the twelve focused save/recovery groups used by
the core readiness checklist. It complements `tools/test_coop_regressions.py`
and the main/ESC menu lifecycle tests. A passing selection does not certify
all core behavior or any complete campaign visit.

Run it inside Anode with a stable built executable and the player's original
base-data folder. Pass the actual session ID reported by `seat_status`. For
`save-death` and either `save-scene-*` case, forward the current lease's
`ANODE_AGENT_ID` and `ANODE_LEASE_TOKEN` into `seat_exec`'s environment. These
are checked before any game starts. The runner does not acquire another lease,
change desktop settings, or take screenshots for inspection.

For example, run the two save families:

```powershell
python tools/test_coop_core.py --exe target/release/looking-glass.exe --data "<original-base-folder>" --seat-session 4 --only import checkpoint
```

Replace `4` with the observed session. Internet invites are the default. Add
`--local` to explicitly select loopback; every case receipt must match that
choice. Both clients use independent processes and profiles. Delay/jitter
defaults to 40/20 ms per sender and can be set with `--delay-ms`/`--jitter-ms`.
The storage-menu fixture uses its normal menu connection without injected
delay; its effective values are recorded separately.

| Case | Required coverage |
| --- | --- |
| `travel-return` | Guest restart at a return-visit loading barrier |
| `import` | Solo conversion, fresh co-op resume, abrupt host loss, resume after loss, backup and guest rejoins |
| `checkpoint` | Shared travel/cache, fresh-process primary/backup load and team wipes |
| `storage` | Locked-file failures, retry, backup repair and damaged-save feedback through the menu |
| `save-death` | Refused menu save during both countdowns, successful safe retries |
| `save-scene-blue`, `save-scene-red` | Refused introduction/dialogue saves and successful post-scene save |
| `lift-blue`, `lift-red` | Both boarding orders and guest restart during the rotating lift |
| `swim` | Both underwater countdown reconnects, revivals and team wipe in the Blue-led route |
| `rope` | Both countdown reconnects and rope-adjacent revivals |
| `fulcrum` | Both countdown reconnects, moving-platform revivals and passive carry |

Without `--only`, all twelve cases are selected. The full selection requires
`--rope-checkpoint` and `--fulcrum-checkpoint` pointing to unchanged ordinary
gameplay checkpoints captured by the corresponding route fixtures. The saves
and original assets remain private and are not bundled with the source.
If a worktree's SchoolPair helper has unrelated edits, `--school-fixture`
can select a saved Python module for the lift cases. It is loaded through an
owned wrapper; the worktree file is not replaced. Its path/hash is recorded.

The runner writes `private/coop/core-suite-<id>/summary.json` plus per-case logs.
It records the executable hash, runtime build identity, fixture hashes, input
checkpoint hashes, effective delays, evidence folders and result-file hashes.
It requires all expected phase results to pass and rejects missing files,
wrong network scope, inconsistent builds or evidence folders that predate the
current invocation. It detects source, input or executable changes during a run.

It stops on the first failure, including provisioning rate limits, and never
retries automatically. Timeouts and interruption close only the owned test
process tree. Failed and incomplete cases remain failures in the report; a
partial selection is listed explicitly. Continue with `--only` for unfinished
cases after resolving their cause, keeping earlier reports for their evidence.

Guard tests need no game assets or desktop:

```powershell
python -m unittest discover -s tools -p test_coop_core_guards.py
```
