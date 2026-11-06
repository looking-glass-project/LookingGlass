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
can still be rate-limited externally; campaign-specific effects/audio and each visit's
recovery cases remain separate work.

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

## Spectator audio listener

The separate `tools/test_coop_audio_listener.py` fixture checks the production
audio listener during both players' ordinary underwater deaths, ten-second
revivals, fresh guest rejoins and a shared checkpoint wipe. Run it in Anode:

```powershell
python tools/test_coop_audio_listener.py --exe <frozen-executable> --build <full-build-identity> --data "<original-base-folder>"
```

Internet invites and 40/20 ms delay/jitter are the defaults; `--local` selects
loopback. It records which player's health, swimming ambience and underwater
filter the mixer uses, and requires that player to match the actual camera
subject. Both clients must cover normal play, spectating and revived play.
The inherited recovery checks also require actual drowning, exact revival
timing, safe placement and preserved resources after fresh guest restarts.

`audio-listeners.json` records the observations; `recovery-result.json` retains
the underlying recovery proof. The final `result.json` exists only after the
additional listener checks pass. This check runs without an audio endpoint and
sets `speaker_output_verified: false`: it validates mixer inputs, not audible
speaker output. The existing `test_coop_audio.py` and `test_coop_voice.py` fixtures
require real audio output and app-only recordings for their separate checks.

Asset-free guard tests are available through `test_coop_audio_listener_guards.py`.

## Live co-op audio output

`tools/test_coop_audio.py` exercises the staged Beyond musical demonstration,
both reset-lever operators and a fresh guest rejoin. Run once with `--first blue`
and once with `--first red`. The lever approaches use separate destinations;
both players must arrive alive, grounded, clear and separated while ordinary
combat remains active. Three ordered notes must match on both clients for each
demonstration/reset. Rejoining must not replay the completed demonstration.

`tools/test_coop_voice.py` watches the village opening without skipping. It
requires all six authored voice identifiers in order, exactly one start per
line, increasing playback time, the host's dialogue clock and cinematic holds.
The stricter verifier rejects partial playback even when both clients omit the
same lines. Its asset-free guards cover missing/reordered lines, restarted
voices, clock drift and absent watched/held playback.

Both fixtures take `--exe <frozen-executable> --data "<original-base-folder>"`.
Public invites and 40/20 ms delay/jitter are the defaults. These checks require
an actual audio output endpoint. If Anode has none, use an available desktop
with audio. They capture only the game's own mix, with independent profiles
and processes; no microphone or system loopback is recorded.

On `66ebd44`, both note orders and the unskipped opening pass over public
invites. All eight app recordings contain 44.1 kHz stereo output. A separate
private waveform audit corroborates the initial notes on both clients, the
first reset in both fresh-guest captures, and the first Rabbit voice on both
clients. Recordings stop at 30 seconds, so the later resets and remaining five
voice lines rely on runtime cue/clock evidence. This does not establish full
campaign audio coverage or subjective sound quality. The saved observations
also pass the final six-line verifier; the original live fixtures remain frozen.

## Remaining controller integration

The focused save/rejoin milestone does not mean every level's mechanics have
co-op adapters. The source audit on 6 November found that the remaining late
controllers (`jlair2`, `tower1`, `hedge2`, `tower2`, `hedge3`, `tower3`,
`grounds1`, `grounds2`, `facade`, `keep`, `qlair`) still use the default paired
hooks. Those hooks do not advance the single-player machinery or combat twice;
they are intentionally empty until a controller has an explicit shared update.
The Jabberwock controller serves both `jlair2` and `grounds1` and also lacks those
adapters. These are implementation gaps, not simply missing route-test receipts.

Majestic Maze now has its explicit shared update. Its component checks are
`--hedge1-coop-check` and `--hedge1-coop-cast-check` (the latter needs a window,
so run it in Anode). They cover both pressure-plate actors, shared clock/carry,
blocked movement, dead-body exclusion, the real NPC contact pipeline, either
escort trigger actor, restored NPC state and automatic exit retry. They do not
prove a complete two-client escort route or its death/rejoin matrix. Continuing
core work should add the missing controllers before resuming broad traversal.
