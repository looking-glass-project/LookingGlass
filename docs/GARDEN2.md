# Herbaceous Border

Visit 14, `garden2$garden2_start1`, has one registry owner. Save envelope 12,
route order and existing trigger keys are unchanged. New Ant receiver keys use
`garden2/ant/<entity>`; targets occupy the reserved 7,300,000 range.

The arrival uses the supplied Rabbit, miniature Rabbit/Hatter, Alice and Cheshire
assets and camera tracks. Dialogue is read from the user's data at runtime. The
script's `sit_smile_close` request maps to the model's `sit_smile_shut` clip.
Missing references remain unresolved rather than creating invented entities.

Watched and skipped scenes share their completion transaction:

| Scene | Required result |
| --- | --- |
| `Garden2_Start` | Dead Rabbit visible, live Rabbits and first Cat removed, Hatter ready at `rabbit_tiny_pos1`; Ladybug 154 active and 599 queued after 2.1 seconds; Alice at `alice_pos_squish2`, yaw 135 |
| `Collapse_Bridge1` / `Collapse_Bridge2` | Six first-bridge pieces removed from drawing and collision; both entry triggers gated; Alice's exact saved approach pose, zero velocity; underworld fog |
| `Collapse_Second_Bridge1` | Twelve pieces removed; Alice at `fakeplayer_bridge_pos2` on solid fulcrum 86; difficulty-filtered `t131` Ant activation |
| `Garden2_Cat_End` | Last Cat removed, both triggers gated; Alice's saved pose retained |

The bridge meshes and colliders use the same translated/rotated pose. Authored
piece delays, 2,400-unit drops and 55-degree rotations run on saved scene clocks.
The central fulcrum dips under the rider, up to 12 degrees at 2 degrees/second,
and returns toward level when unloaded. Its saved pose supplies both collision
and rendering; checked rider movement cannot push Alice into a wall. Entrance
rocks and the four difficulty pieces are restored; the latter are omitted on
Hard/Nightmare. Scene-local 0.5
and 0.8 rates never alter the global physics step or enemy timers.

Older controller-less visits continue without replaying arrival or granting
resources. Trigger history is inspected before pending rules are rearmed. Old
bridge events are retired, and the current player's route position can retire
bridges behind her. Restored solids use the existing invalid-position recovery;
ordinary valid saves retain their position. New scenes save their home, cast,
camera/phase and dialogue clocks and derive world geometry from the same state.

All seven spawned Ladybugs start their own patrol, including delayed spawns and
active older saves. Duplicate spawn names remain separate entity identities.
The underworld Corporal's missing ambush now waits one second, runs toward its
authored position with collision checks, then resumes combat with its specified
sight range. Both the delay and run survive saving. Ladybugs leave normal essence
loot. Placed Antlions, Bloodroses and Mushrooms use the production resident cast.

Two movement gaps blocked the actual route: an ascending jump from shallow slime
was being cancelled by swimming, and both hands missed diagonal ledges because
the square player body reaches farther toward them. The corrected checks retain
liquid damage, body clearance, two-hand support and saved mid-pull continuation.

`--garden2-route-check` runs fresh watched and skipped routes on Normal with the
native enemy cast, real damage, ordinary inputs and authored pickups. It crosses
both collapsing bridges, the southeast slime bank and thermal chain, the Mallet
detour, fulcrum and launch ledge, the northern tunnel, final Cat, two cavern vines,
Cards ledges, diagonal exit steps, surface vine and final rabbit hole. It requires
the bridge pieces to be gone and the real exit to load `garden3$garden3_start1`
with identical carried resources. The checked input goals are in
`src/levels/garden2/route_steps.json`; no private calibration file is required.

Thirteen marked moments per route compare the running and disk-restored future,
including the fulcrum, updrafts and three vine grips. The route then continues
from those disk saves. `--garden2-check` additionally exercises every scene's
watched/skipped handoff, legacy migration, all Ladybug patrols, the Corporal run,
and fulcrum rider/restore behavior. `--garden2-render-check` captures six authored
shots. Six native save fixtures cover arrival, both falling bridges and the final
Cat, with separate writer and reader processes. `tools/launchers/Launch-Herbaceous-Border.cmd`
uses separate playtest saves; the usual root launcher uses the selected build.

Exact original quake noise, actor approach avoidance and translucent facial
deformation remain fidelity limits. The route proof covers the described Normal
path, not every optional branch or difficulty. Native checks run muted. No
original script or subtitle prose is stored.

## Co-op integration

The host advances the native fulcrum, fade, spawn delays and Corporal ambush once
per world tick. The fulcrum uses the mean position of living supported riders;
opposite riders balance it. It commits the platform and all riders together or
rejects an obstructed move. A dead teammate contributes no weight and is not
carried. This passive weight-driven support has no triggered passenger departure
to gate on boarding. The single-player controller keeps its original behavior.

Owned Ants and Ladybugs receive both living/visible player targets and route
damage to separate player slots. Their native cast identities, attacks and loot
remain shared. Short Ladybug explosion bursts now survive snapshot serialization
with bounded count, position and age validation. Older saves omit this optional
field and load with no pending bursts; the save-envelope version is unchanged.

`--garden2-coop-check` provides staged headless checks, with optional group
selection via `LOOKING_GLASS_GARDEN2_COOP_CASE=platform|combat|scenes|contact`. On source
`0601aeb39bc98a7828a398e3c59f6fc5ef637ab54ab5fe343994a079cc45f8a2`,
44 cases pass: six platform cases, 20 cast/impact cases and 18 scene cases.
These include 30/60/144 Hz paired carry, balancing, dead-partner single-player
parity, pause/save, targeting and isolated impacts in both slots, bounded shared
spawn/ambush clocks, impact snapshot restoration, and original triggers with
either first arrival, frozen partners, watched/skipped scenes and safe reunion.
The automatic arrival is simultaneous and uses Blue as the existing tie-break.

The Cat fixtures now approach trigger 52 from the supported route waypoint;
the former trigger-53 approach fell into a gap before contact in both single-
and two-player checks. Existing `--garden2-check` passes on the same build,
including all 45 scene variants, legacy migration, patrols and single-rider
fulcrum checks. Logs: `private/coop-garden2-components-0601aeb.log` and
`private/coop-garden2-single-0601aeb.log`. All 808 active Rust tests pass, including
impact replication/expiry and old-save loading. No images were viewed.

This does not establish full co-op traversal, independent-client internet play,
rendered guest effects, or this visit's death/wipe/rejoin matrix. The campaign row
remains partial.

## Ordinary-input paired route

`--garden2-coop-route-check` runs the production co-op Arena against the existing
108-step authored route. Each player chooses ordinary movement and attacks;
enemy targeting, resources, physics, scene ownership and reunion remain gameplay.
Both initial input orders run with watched and skipped scenes. The driver checks
frozen partners, free-play body spacing, periodic compressed guest snapshot
restoration, and unchanged personal resources on the authored Garden3 transition.
Any death, recovery, unexplained warp or stalled waypoint fails the attempt.
The windowed check uses Anode; results need no screenshot inspection.

Set `LOOKING_GLASS_SETTINGS_DIR` to a fresh private profile. That directory receives
the exact course, marked waypoint states and a final passed/failed report for
each attempt, including the full frame/world and the last 240 input ticks. A
failed attempt does not stop the other three variants. These are native paired
checks, not independent network clients or a rendered-effects check.

First run on source
`da5e242bd7751ff0a691cacf7923bc7c73bd3ebec80c766b0431a583b966e7ad`
fails all four variants around the first thermal climb. With Blue starting,
Blue dies at steps `[22,25]`, ticks 13,422 watched / 4,847 skipped, while Red
retains 90 Sanity. With Red starting, Red dies at steps `[23,23]`, ticks 13,708
watched / 5,133 skipped, while Blue retains 33 Sanity. Watched/skipped pairs
reach the same outcomes, shifted by scene time. The preceding bridge and slime
bank states are captured, with no earlier spacing or replica mismatch reported.
The failure's gameplay-versus-driver cause is not established yet.

Evidence: `private/coop-garden2-route-da5e242.log` and
`private/garden2-coop-route-da5e242/`. The four attempts finish in 43.6 seconds,
with 808 active unit tests, the locked release build and seven source-package
checks passing. No images were inspected. No full-route or campaign credit is
awarded; total estimated co-op progress remains 43.1%.

For focused repeats, `LOOKING_GLASS_GARDEN2_COOP_FIRST=Blue|Red` and
`LOOKING_GLASS_GARDEN2_COOP_SCENES=watched|skipped` select one variant; omitted
selectors still run all four. `LOOKING_GLASS_GARDEN2_COOP_ROUTE` can supply an
explicit diagnostic course, copied unchanged into the private result profile.
The default remains the original authored route.

The first thermal capture contains Blue being pulled toward the nearby live
Mushroom, with large outward velocity and an active knockback timer while the
driver suppresses attacks during the climb. A normal-input bank-fight experiment
on source `3c996ca2c4df7458b3812aeed5621da4cf367e3c5d7cf19a064eb482415a721b`
did not fix the route. Blue-start variants instead find a landing overlap at
ticks 11,313 / 2,738 (watched/skipped), spacing vector
`[34.128418,6.940918,-1.7967529]`; Red-start variants still lose Red near the
thermal at ticks 14,325 / 5,750, steps `[24,24]`, with Blue at 19.59982 Sanity.
All reports retain `full_route_passed=false`. The experimental 109-step course
and state captures remain in `private/garden2-coop-bank-3c996ca/`, with log
`private/coop-garden2-bank-3c996ca.log`. Its four variants finished in 22.9 seconds
without image inspection. These failures remain open; the experiment is not
the default course and is not credited as a gameplay fix.

## Captured bank contact

The saved Blue/skipped bank overlap now has a headless native regression in
the `contact` group. It uses the exact captured positions in both player slots,
then repeats movement/contact for 240 ticks, checking body clearance, valid
player state, stationary-partner position and unchanged personal resources.

On source `39fbf21`, the original tangent correction fails at tick zero: the
body sweep reports floor contact at fraction zero even though its destination
and both ground-support probes are valid. The slope normal is approximately
`[0.040770,0.057757,0.997498]`. This isolates a collision-skin/rounding issue,
separate from the earlier centre-support probe fix.

The correction now retries a grounded non-swimming body blocked by a walkable
floor with one collision skin of vertical clearance. It then repeats the entire
path, destination-body and centre-support checks. A wall, unsupported edge or
embedded starting body still rejects the move. Rope handling and single-player
movement are unchanged.

Source `c6a4ddc5632c8f9dec60303122e6629c2ce5eaa0cfa982d231d55420349e32c9`
passes both captured contact cases and all 44 earlier component cases (46 total),
plus 808 active unit tests. The locked release build, seven source-package tests
and worktree audit pass. Native evidence: `private/coop-bank-contact-native-c6a4ddc.log`.

The unchanged experimental 109-step course, Blue first with scenes skipped,
now passes the former tick-2,738 overlap. It later fails at tick 5,112 when Blue
dies near the thermal, at steps `[23,28]` with health `[0,90]`. Nine compressed
replica comparisons pass before that failure. Its final frame is tick 5,113;
`full_route_passed` remains false. Evidence:
`private/coop-bank-contact-route-c6a4ddc.log` and
`private/garden2-coop-bank-contact-c6a4ddc/`. This fixes the captured bank contact;
it does not fix the thermal traversal or complete the visit.


## Playable paired route, 7 November 2026

Frozen `4bb7dac` passes the normal Blue-led watched route into Rolling Stones
with both players alive: 33,190 ticks, 56 matching Arena replicas, four scene
reunions and five exact 1,200-active-tick revivals at 25 health. All personal
resources carry unchanged into the next map; final health is 25/46.

The native test chooses a planned route around the blocked approach at
[800,-180,-1120]. No movement, combat or progression behavior changes.
`LOOKING_GLASS_GARDEN2_COOP_RECOVERY=allowed` accepts ordinary revival and
records personal weapon ownership. Blue collects the Mallet; Red misses it
after revival. This proves playable progression with recovery, not both
optional pickups; strict mode retains the original two-pickup requirement.
Repeat with `tools/test_coop_garden_native.py --map garden2` and isolated
`--exe`, `--data` and `--profile` paths. Native simulation is separate from
independent-process internet and rendered spectator-camera acceptance.
See COOP_PROGRESS.md for tests, evidence and remaining scope.
