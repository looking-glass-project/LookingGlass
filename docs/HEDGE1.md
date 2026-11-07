# Majestic Maze: child, pressure plate and departure

`hedge1$hedge1_start1` uses the existing saved companion actors. Crossing BSP 25 starts the Muzzle child's run to `seek_end`. Its body presses plate 30, sinking brush 31 by four units and opening leaves 32/33 by 120 units at 400 units per second. Crossing BSP 527 sends that child through `seek_middle` to `seek_tele_path`, removes it and activates the chase child. The empty plate lets the gates close.

The chase child approaches Alice when she is visible within 300 units, walks inside 150, runs farther away and stops at 60. Its own contact with the monsters-only Hold volume, while approaching, starts the held pose. Continued occupancy refreshes Hold and the plate. Alice and enemy bodies can press the ordinary plate, but Alice cannot invoke Hold or complete the escort alone. The player and companion systems feed their real body contacts to the same controller; there is no second child or decorative gate.

The gate uses fixed steps, source brush collision and the same saved fractions for rendering and collision. It waits rather than crushing Alice. The plate carries a standing player. Pause freezes its clocks. Child movement and animation remain in the existing NPC snapshot; the new `hedge1` controller saves the gate, occupancy and exit state. An older NPC-only save keeps compatible actor states and rearms only the formerly pending Seek contacts. Invalid positions inside newly restored solids use the existing safe entrance recovery.

The exit is BSP 29, beyond the double gate, to `tower1$tower1_start1`. Source inspection confirms the entire `SeekCine` function and its map call are inside a block comment in `maps/hedge1.scr` lines 131–153. No cinematic is invented or required. The physical exit requires the chase child holding the plate and the leaves open. Departure is delivered once, and a failed destination load can retry.

The `t30` portal marker lies in the solid `teleporter2_1red` patch (world surface 549). A map-specific correction places Alice 16 units in the marker's facing direction, at `(4584,2648,2056)`, after checking full-body clearance. Surrounding collision stays intact. This is a documented landing approximation.

The native enemy system retains ownership of placed enemies and ambush groups. The seven untargeted actors in the sealed precache room remain in their original snapshot slots but are hidden and inactive. This preserves identities for the live enemies and their drops.

Deliberate differences and remaining fidelity work:

- Only the chase child's own occupancy can hold it. The original thread also renames that child when an unrelated monster touches the volume, potentially stranding it elsewhere.
- Long-delay repeat ambush spawning, hedge-top launch trajectories, puffball motion and gate sound fidelity retain their existing approximations. No new voiced scene is supplied by this map.
- The follower keeps up to 32 recent visible positions of Alice to round corners with collision-checked steps. Obstructed trail fragments are skipped only when the whole supported path to a later remembered point is clear, with retries limited to four per second. The saved trail can continue briefly after losing sight; it never learns unseen positions and clears when Alice is beyond 300 units. This is the bounded navigation approximation used in place of the original engine's path search.

Checks:

- `--hedge1-check`: supplied-map source bindings, closed/open collision, plate occupancy, escort gate, pause, restoration, retry and shared companion checks at 30/60/144 updates per second.
- `--hedge1-route-check`: normal entrance, native combat, child demonstration, handoff, portal, return escort and strict resource-preserving departure to tower1. Uses ordinary movement and paid weapon attacks.
- `--hedge1-save-write` / `--hedge1-save-read`: real save files from route checkpoints, exact state restoration and identical continuation in a fresh process.
- `--hedge1-render-check`: a route checkpoint supplied through `LOOKING_GLASS_MAZE_CAPTURE`; optional look target `LOOKING_GLASS_MAZE_LOOK`.

Private diagnostics and checkpoints live under `private/hedge1`; no original assets or script bodies are redistributed.

## Co-op integration, 6 November 2026

`c6f580e` supplies an explicit paired machinery update. Either living Alice can
press the plate; both riders are carried in the same fixed step. All candidate
bodies must be clear before either moves, and a closing gate waits for either
living player. Dead bodies are excluded. This four-unit pressure plate retains
its ordinary occupancy behavior; it is not a passenger ride that waits for a
second boarder. Only the real child holding its own volume can enable the exit.

The production Arena now feeds native companion and enemy body contacts into
the shared mover step, including after restoration. Either player can trigger
the child's demonstration/handoff. The follower chooses the nearest visible
living player and clears its remembered path when that slot changes. The slot
is an optional saved field; existing solo guide state remains compatible.
Pending failed departures retry once through the shared update without a use
press, and pause cannot emit the retry.

- `--hedge1-coop-check`: twelve native map cases for both slots at 30/60/144 Hz,
  solo clock parity, pause/restore, blocked/released carry, dead-body exclusion,
  both closing-gate blockers, child ownership and departure retry.
- `--hedge1-coop-cast-check`: seven windowed production cases for actual Arena
  contacts before/after restore, both native trigger actors, exact NPC replica
  state and the shared automatic exit retry. Run in Anode; no image lookup is
  required.

Both checks pass, as do 940 active Rust tests, the shared-world checks and all
39 entrances/four underwater recovery cases. A fresh full solo route on this
build exactly matches a fresh `66ebd44` baseline: 28,351 ticks, one teleport,
one transition, 20 loot collections, no hazard/lost ticks, and final resources
95.2152 Sanity/99.2 Will. All eleven JSON checkpoints/reports and the route log
match exactly. Evidence is in `private/coop-maze-native-c6f580e/` and
`private/coop-maze-solo-before/`. Full independent-client traversal, personal
combat/rewards and the death/wipe/rejoin matrix remain open.

Verified on 2026-10-01:

- The full default native route starts at the ordinary entrance on Normal difficulty and reaches `tower1$tower1_start1` in 25,580 simulation ticks. It completes the demonstration, handoff, portal, return escort, Hold and physical departure. It collects 17 essence drops and finishes with 48.196 Sanity and 98.400 Will; the tower arrival preserves both exactly. There are zero hazard/lost ticks, zero jumps, one authored teleport and exactly one transition. God mode, notarget and minimum-health assistance are disabled.
- All 637 unit tests pass, including saved corner navigation, recovery after doubling back, and the narrowly scoped precache identity upgrade. All targets compile. Supplied-map gate/companion checks pass at 30/60/144 Hz.
- Real save files for the demonstration, moving leaves, handoff, following and holding restore exactly in a fresh process and continue identically for 240 ticks. A pre-controller save fixture retains compatible live actors and resources, deactivates the seven precache actors and recovers an old player position inside a newly solid leaf at the entrance.
- Native closed-gate and held-plate captures were visually inspected; the scene reports zero fallback images. Audio was disabled during these runs, so this is not an audio-fidelity result.

## Blocked portal retry fixed

A normal Maze run exposed a shared-portal failure: when the first player stood
at the destination, the second player's attempt was rejected safely, but the
contact became latched during its cooldown and stopped retrying. Blocked
personal portals now retain a pending contact until the destination clears or
the player leaves. The cooldown still applies, successful arrivals clear the
pending contact, and pending retries survive save/restore using the existing
contact field. Solo contacts are unchanged.

Frozen `1774b6a` passes all **950 active Rust tests**, including both-player
cooldown, restored retry, successful completion and cancellation coverage.
The complete normal paired Maze route now passes on `845bb52`, including
both portal arrivals. The portal fix itself adds no separate campaign credit.

## Majestic Maze normal paired route accepted

Frozen `845bb52` passes the complete normal paired Maze course: **33,270 ticks**,
**278 exact compressed replicas**, both personal portal arrivals, the real
child's escort and plate hold, and actual Mystifying Madness arrival. Both
players finish alive at **98/100 health**, with their personal resources carried
exactly. There are no deaths, revivals, team wipes, cheats or forced quest state.

The automated inputs use Cards/Ice, single-file narrow legs, ordinary walks to
regain sight of the child, and close spacing beyond the final plate. The real
gate must open before departure. The run also exposed and fixed the blocked
portal retry described below. The passing course and exact source/executable
are pinned in `private/coop-playable-majestic-maze-default-verified.json`.
Repeat with `tools/test_coop_native_route.py --map hedge1`, local game data
and a fresh profile. The default check exactly matches the accepted course
inputs, final players, shared world and onward arrival.

This accelerated Arena run proves normal paired progression and replicated
world state. Internet and rendered-camera evidence remains separate; broader
per-visit recovery permutations belong to polishing. Normal coverage is now
**35/39 visits**, **4 remaining**, or **79.4%** overall.
