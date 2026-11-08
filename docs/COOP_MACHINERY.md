# Late-campaign co-op machinery

Build `11d47f8` integrates Mystifying Madness, Labyrinthine Revenge and
Machinations. Both players can use the maze levers and receive their own bubble
air. Shared doors and machinery advance once, carry living riders together and
reject blocked movement for either body. Crushing is delivered to the actual
victim. Machinations keeps separate fall checkpoints and waits for all living
riders at shared lift landings; empty lifts continue cycling.

All three focused paired native checks and the three existing solo component
checks pass. Paired coverage includes use inputs, individual breathing, door
obstruction, shared clocks, two riders, lift boarding/dead exclusion, personal
checkpoint contacts and saved continuations. The native registry check also
passes. The Rust run passed 941 tests with one command-registration failure;
the allowed-command table was corrected and the same validation passes in the
native registry check. Existing 450 Python co-op guards and seven package checks
pass. This is component evidence, not a complete paired route or a new internet
save/rejoin run. The prior passing core save/rejoin evidence remains unchanged.

Normal paired campaign progression is still required for the playable milestone.
Exhaustive scene/recovery permutations and visual comparisons remain polishing
work unless a normal-play blocker appears.

Frozen native source identity: `11d47f8420cdd5fdc40619555900a3fe5c6662319c7c4d56c1697c61ba2ca579`.
Reproduce with `--hedge2-coop-check`, `--hedge3-coop-check`,
`--tower3-coop-check`, their corresponding solo `--<map>-check` flags,
and `--registry-check`, supplying the original data through `--data`.
These checks are headless and use no screenshots. Private receipts retain
the executable hash, source-file hashes, command results and log hashes.
