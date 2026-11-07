# Queen finale co-op

Build `fa8ddf1` integrates Heart of Darkness, the last campaign controller
without an explicit paired update. The normal paired campaign pass remains open.

Both Queen forms advance once and select a visible living opponent through the
shared targeting policy. Native attack events, projectile collisions, splash,
beams and melee use stable player slots. A telekinetic grab controls and damages
only its victim; the partner retains their camera and can fight. Death releases
the capture and allows survivor targeting.

Healing stations have separate saved cycles, cooldowns and local presentation.
Initial and second-arena refills are tracked individually. Importing an existing
save preserves the current cycle instead of treating a rejoin as a refill.
Each player's platform contacts are tracked separately, so two arrivals do not
cancel the Queen's platform selection.

The original introduction, birth, death and ending run on the shared scene
clock, with one actor and safe reunion at each arena handoff. Co-op now consumes
level-requested checkpoints after scenes finish and both players are alive.
The checkpoint writer acknowledges requests only after its disk commit. Failed
automatic writes retain the request and retry with a delay.

Completion is included in the ordinary replicated world. The host waits for
receipt of its completed frame before closing, with a five-second fallback for
a lost connection. Each client plays the original ending film and credits and
returns to the menu. The completed world does not overwrite the playable
checkpoint; Host/Resume replays the final encounter. The film/credits interface
still needs a hands-on review.

## Verification

`--qlair-coop-check` passes eleven component groups on `fa8ddf1` in Anode
(session 4, exit 0, 2.73 seconds). It requires a graphics context to construct
real character save state, but takes no screenshots and requires no input.
It covers personal healing and saved cooldowns, both grab victims and survivor
retargeting, both projectile recipients and splash, individual platform contacts,
native projectile animation events in both forms, the original Red-owned
introduction, birth/reunion/refill, an actual co-op disk checkpoint and exact
restoration, and the shared death/ending lifecycle.

The existing solo finale check and movie decode check pass on `2a75c26`;
the only later differences are the paired fixture's graphics-context wrapper
and command registration. Their passing evidence is reused. The native registry
passes on `fa8ddf1`, building all 39 visits twice deterministically; no historical
fingerprint baseline was supplied.

The initial gameplay batch `c9b0b9f` passed 943 Rust tests, including the real
transport receipt check, plus 450 Python co-op guards and seven package checks.
The final bounded impact-validation correction is covered by the native checks;
the full Rust run predates that correction. Python guards and package checks
also pass after the correction.

The first paired run found an existing save/replication bug: native comet/jack
impact marks can have radius 300, above the old validation limit of 256. The
bounded limit now permits those authored effects. A subsequent test-only panic
came from constructing GPU-backed character state without a graphics context;
the final fixture runs in Anode and passes. Failed receipts remain private.

This is component evidence, not a complete two-client Queen combat route,
rendered film/credits acceptance or an internet finale-disconnect test. Earlier
core import/resume/rejoin evidence remains applicable to unchanged paths.
