# Castle co-op integration

Build `073ed9f` adds Battle Royale, Ascension and Castle Keep's core co-op
adapters. Reinforcements spawn on one shared clock and use the existing
nearest-visible living-player targeting. Spawned castle guards now preserve
each player's damage and impulses. Ascension waits for all living players on
the lift, remembers the first arrival and delivers one shared Keep transition.
Castle Keep advances its puzzle and hallway clocks once, accepts either player's
lever input, checks both bodies during movement and delivers its final exit.

The three focused native paired checks pass, covering the original Battle Royale
arrival/reunion, wave cadence/caps/landings and both damage victims; both lift
boarding orders, saved ride/departure and both gate blockers; and Keep's arrival,
Red-owned mirror scene, all three correct portraits, saved progression, heart
gate, final Queen departure and all four guard groups targeting either player.
The three existing solo component checks and the native registry check pass.
The gameplay code passes all 942 active Rust tests, 450 Python co-op guards and
seven package checks. The final Keep fixture uses normal entry-story setup;
the original fixture disabled that setup and failed before the arrival. Only
that fixture changed after the first build, so the unchanged paired, solo and
registry passes are reused after verifying the source-file hashes.

These are staged components, not full normal-input paired traversals. Earlier
public save/import/rejoin evidence is reused for unchanged systems. Full paired
campaign progression remains required; exhaustive permutations and rendered
comparisons belong to the polishing stage unless they reveal a play blocker.

Frozen native source identity: `073ed9ff887951c18bd8f8d32dbcdf5c36bee8ebc4bbbbf3e4b824f651d14e31`.
Reproduce headlessly with `--grounds2-coop-check`, `--facade-coop-check`,
`--keep-coop-check`, their matching solo `--<map>-check` flags and
`--registry-check`, with original data supplied through `--data`. Private
receipts retain executable/source/log hashes and command results. Existing
save versions are retained, including Castle Keep controller version 2.
