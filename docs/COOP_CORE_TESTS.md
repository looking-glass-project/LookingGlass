# Automated core save and recovery checks

`tools/test_coop_core.py` runs thirteen focused save/recovery groups used by
the core readiness checklist. It complements `tools/test_coop_regressions.py`
and the main/ESC menu lifecycle tests. A passing selection does not certify
all core behavior or any complete campaign visit.

## Current save/rejoin acceptance

Both requested meanings of an earlier game are covered by the public-invite
consolidation on frozen build `0b5bca3`:

| Starting game | Verified behavior |
| --- | --- |
| Existing single-player save | Load the solo game, host from its current progress, add player 2, save co-op, then resume and rejoin with fresh processes. The original solo slot stays unchanged. |
| Earlier co-op save | Continue the shared checkpoint with both players' personal resources and cached visits, rejoin after disconnection or host loss, recover from the previous backup, and restore the shared checkpoint after a team wipe. |

The full focused milestone contains thirteen passing public core groups and
nineteen phase results on the same executable. Their recorded hashes and the
frozen executable were rechecked on 6 November 2026. This was an evidence audit,
not a new gameplay run. Normal-menu save refusal during death/scene sequences,
safe retries and locked-file failures are also covered. See the current
[core acceptance checkpoint](COOP_PROGRESS.md#core-acceptance-checkpoint) for
the build-specific evidence and remaining limits.

These passes establish the focused save/rejoin milestone. Invite provisioning
can still be rate-limited externally; broader effects/audio and each campaign
visit's recovery cases remain separate work.

## Running the checks

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
The storage and host-loss menu fixtures use normal connections without injected
delay; their effective values are recorded separately.

| Case | Required coverage |
| --- | --- |
| `travel-return` | Guest restart at a return-visit loading barrier |
| `import` | Solo conversion, fresh co-op resume, abrupt host loss, resume after loss, backup and guest rejoins |
| `checkpoint` | Shared travel/cache, fresh-process primary/backup load and team wipes |
| `storage` | Locked-file failures, retry, backup repair and damaged-save feedback through the menu |
| `host-loss-menu` | Abrupt host loss, guest error/solo return, fresh-host Continue and same-process guest rejoin |
| `save-death` | Refused menu save during both countdowns, successful safe retries |
| `save-scene-blue`, `save-scene-red` | Refused introduction/dialogue saves and successful post-scene save |
| `lift-blue`, `lift-red` | Both boarding orders and guest restart during the rotating lift |
| `swim` | Both underwater countdown reconnects, revivals and team wipe in the Blue-led route |
| `rope` | Both countdown reconnects and rope-adjacent revivals |
| `fulcrum` | Both countdown reconnects, moving-platform revivals and passive carry |

The swimming fixture performs its shared wipe and entrance rejoin before the
individual deaths. It waits for both swimmers to have fresh air from normal
gameplay before stopping their inputs; different air reserves after individual
revivals can otherwise cause alternating deaths without a team wipe. Later
countdown checks require the known restored checkpoint generation and still
reject any unexpected wipe. No game state is edited to arrange these cases.

Without `--only`, all thirteen cases are selected. The full selection requires
`--rope-checkpoint` and `--fulcrum-checkpoint` pointing to unchanged ordinary
gameplay checkpoints captured by the corresponding route fixtures. The saves
and original assets remain private and are not bundled with the source.
If a worktree's SchoolPair helper has unrelated edits, `--school-fixture`
can select a saved Python module for the lift cases. It is loaded through an
owned wrapper; the worktree file is not replaced. Its path/hash is recorded.

For `import`, optional `--source-save <solo-save.json>` tests a copy of an
existing solo save. Choose a living Alice in a quiet, safe location: this
strict check requires preserved positions/resources while clients reconnect.
Before the guest joins, it compares Blue's entire body, inventory, camera,
character, world, story, hints and NPC state against the original file. It
allows only different decimal spellings of the same native 32-bit float.
State-changing migrations fail this preservation check. The original file is
never used as a writable profile and its hash is checked throughout the run.
The import, resume, resume-after-loss and backup phases still run. This option replaces
the generated school/swimming preparation and its storage-failure fixture;
retain those default-case results as separate coverage.

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
