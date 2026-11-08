# Water Logged

## Normal paired route (2026-11-07)

Frozen `b278fdc` completes the ordinary co-op course through all three flushers,
the underwater passages and actual Labyrinthine Revenge arrival. Both players
finish at 55 health with their resources carried exactly. The run records
28,247 ticks, 236 matching compressed replicas and the original arrival/reunion.
Red revives twice and Blue once, each after exactly 1,200 active ticks at
25 health. No team wipe, cheats or world-progression staging is used.

Run `tools/test_coop_native_route.py --map tower2 --exe <frozen-executable>
--data <base-folder> --profile <new-folder>` in Anode. The native entry point is
`--tower2-coop-route-check`. The diagnostic uses normal Blade/Cards, the owned
Watch, independent fan-gap timing and single-file swimming through the final
narrow inlet. It follows actual footing after a vacated tray springs back.
No gameplay or save behavior changes. The locked release, all 949 active Rust
tests and source/package checks pass. Exact evidence is pinned privately in
`coop-playable-waterlogged-default-verified.json`; public-invite, presentation
and additional recovery permutations retain their separate evidence/backlog.

## Co-op core integration (2026-11-06)

Build `d06ded3c2ec57c42176358b727ad37c3e9e4fa4b28ab710653bc3269d4f4d553`
adds the shared machinery adapter. Water, lids and fans advance once per world
step. Either living Alice can weigh down a flusher, with the original boolean
weight/acceleration rule. Every supported living rider moves atomically; if
either path or proposed body is blocked, neither rider moves and the affected
brush rolls back. Dead bodies neither ride nor obstruct the machinery.

Each Alice has her own four current cooldowns and pending fan-hit flags.
Blades retain their original angular sweep and lethal contact damage, delivered
only to the struck player through the normal co-op feedback path. Either
living player's Watch freezes water/lids/fans, while weighted trays and native
current pulses retain their ordinary behavior. Shell preparation and air-source
refresh apply independently to living players. Revival clears the returning
player's stale current waits and pending hits.

The optional paired state preserves existing solo saves and their stage-derived
machinery. A pending solo fan hit belongs to Blue, while both players inherit
the saved current waits without a reset. Compressed restores preserve moving
machinery and individual timers; invalid timers or missing paired machinery
are rejected before changing live state. The original single-Alice arrival
camera/story keeps the partner frozen and uses the shared safe reunion.

`--tower2-coop-check` passes 66 supplied-map cases. These include both riders
and either dead partner on all three trays, both obstructed-rider slots, all
three lids, both victims and dead-body variants for all five fans, all four
currents in either arrival order, and personal air refresh. Both ordinary-input
swimming orders cross the live drain with exact saved continuation. Twelve
Stage introduction variants cover both owners, watched/early/late skip,
pause/restore and dead partners; six staged pressure courses use real body
physics and native contacts to activate every flusher as either Alice.
The original 16-case solo check produces byte-identical output to `291af36`.

These checks are headless and stage the entry prerequisites of each component
course. They do not certify full two-client traversal, native combat through
the whole visit, rendered effects, or the per-visit rejoin/death/wipe matrix.
Build `0e5f061` adds placement exclusion around each fan's complete rotation,
derived from native local model bounds and the full player body. Checkpoints,
solo conversion, reunion and revival use the existing hazard predicate;
ordinary movement and lethal blade-contact damage remain unchanged. Watch
and the saved blade angle cannot temporarily remove this exclusion.

The paired check now passes 75 cases. Five new native fan cases reject
currently clear gaps for either checkpoint slot and sample a complete
rotation. Four staged revival cases cover both players, the real 1200-tick
countdown, spectator selection, compressed countdown continuation, safe
25-health placement and waiting for a safe location after the timer expires.
Both bodies remain clear through a full fan cycle after Watch is released.
The held case stages the survivor's later position; it is not a traversal or
internet reconnect test. The solo 16-case log still exactly matches `291af36`.
All 941 active Rust tests, 450 Python guards and seven package checks pass.

## Existing single-player behavior and evidence

`tower2$tower2_start1` retains save envelope 12 and controller state version 1.
One owner drives the three weighted flushers, three lids, five fans, moving
liquid and the opening Cheshire scene. Rendered and collision poses come from
the same saved machinery state.

The initial surface is 624. Each ordered flusher raises it 256 units over five
seconds: 880, 1136, then 1392. The first two lids open to +35 degrees in six
seconds; the last opens to -35 in five. Opening a later lid closes the preceding
one to 90 degrees over eight seconds. Open lids sway by two degrees on a saved
ten-second cycle. The exit requires the third rise and an open final lid.

The authored flusher speed controls acceleration, rather than a fixed sinking
speed. Riders depress each tray by up to 64 units; vacated trays return with
damping. Their depth and velocity persist. Trays continue during the Watch;
water, lids, fans and enemies freeze. Swept collision checks carry riders and
reject machinery motion that would embed Alice. The five fans rotate on their
authored axes and rates, with lethal 1,000-point damage on actual blade contact.

The four drain currents retain their native velocity projection and inherited
0.2-second trigger wait. Previously the projection ran every physics tick,
removing Alice's ability to steer away from the drain. Each current's remaining
wait now persists with the tank state; other maps' force volumes are unchanged.

The introduction uses the original camera track and Cat clips, runtime-loaded
dialogue, appearance/disappearance sounds and fades. Watched, skipped and saved
playback return control at the supported arrival pose. Entry grants the Reptile
Shell. Four underwater air emitters refresh its twenty-second breath allowance;
their overlapping server lifetimes leave a continuous local refresh volume.
The existing particle emitters remain visible.

The native resident cast contains seven placed Snarks and twenty spawned Snarks
in five distinct waves (3, 6, 5, 4 and 2). Duplicate target names retain separate
entity identities. Their ordinary swimming, tongue/projectile combat, damage,
death and loot remain active. There are no invented enemy-kill exit gates.

Old rising-water saves preserve their water stage and skip the newly added
introduction. Missing machinery state is derived from the saved stage clocks.
Controller-less legacy imports use the established campaign recovery path.

`--tower2-check` verifies introduction variants, all machinery, Watch behavior,
legacy stage migration, air refresh and save continuation at 30/60/144 Hz.
`--tower2-cast-check` exercises all five native trigger volumes, distinct wave
receivers, exact saved futures and no resurrection on retrigger/load.
`--tower2-render-check` captures each water height and three introduction shots.
Four native save fixtures cover introduction, first rise, second-stage machinery
and the final opened shaft, using separate writer and reader processes.

The second climb exposed a beveled pipe rim that rejected both hand probes.
The ledge check can now reach across a narrow connected bevel to a horizontal
cap. It still rejects empty gaps and requires both hands and a collision-clear
pull-up. The asset check includes a real jump and saved mid-pull continuation.

`--tower2-route-check` completes the whole Normal route from fresh arrival with
the introduction watched and skipped. Ordinary movement reaches all three
flushers through the live currents, fans and native Snark combat, collects the
large essence, then dives through the real exit into `hedge3$hedge3_start1`.
Both runs finish at 75 sanity with about 13.9 seconds of air remaining and no
drowning damage. Each run reloads ten disk checkpoints and checks identical
two-second futures, including rising water, air sources, the fan approach and
the final dive. No god mode, notarget, teleport or staged progression is used.
All five waves are checked independently; this route does not require every
optional encounter to be cleared. This is standalone native route validation,
not certification of the complete campaign chain or every difficulty.

`tools/launchers/Launch-Water-Logged.cmd` starts a separate playtest save directory through the
regular root launcher. The normal launcher includes these changes as well.

Fidelity limits: the two-degree sinusoidal lid sway is a bounded reconstruction,
not a verified port of the native pendulum integrator. Flusher acceleration is
resampled from the researched 20 Hz law. Air refresh uses the same model-bounds
approximation as the restored temple bubbles; exact native bounds setup order
is unverified. Fan motion has no additional synthetic blur. Native checks run
muted; original script and subtitle prose are not stored.
