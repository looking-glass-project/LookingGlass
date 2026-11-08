# Co-op progress

Updated 8 November 2026. Branch: `coop`. **Playable readiness: 100%. Historical full-matrix completion: 81.5%.**
The [100% playable co-op milestone](COOP_PLAYABLE.md) is complete. The next
stage is polishing; normal progression and requested core flows are accepted. The historical exhaustive
matrix below also tracks polishing work and does not all gate playability.

Playable scoring now assigns 40 points to requested core flows, 55 to normal
campaign coverage and 5 to the two-client ending handoff. All 100 points
are accepted, including both rendered movies and credits. The historical full-matrix
score below retains deferred polishing work and is reported separately.

The older 42% reports are historical. This percentage measures delivered and
verified behavior against the full co-op scope; it is not a running test's progress.

The focused save/rejoin milestone passes on frozen `0b5bca3`, including
adding player 2 to a solo save and resuming an earlier co-op save. All thirteen
selected core groups have public-invite passes, covering personal resources,
fresh processes, rejoining, backup recovery, shared wipes and unchanged solo
saves. Both requested save paths are included in the
[core acceptance checklist](COOP_CORE_TESTS.md#current-saverejoin-acceptance).
The agreed all-level playable goal is complete. Majestic Maze now has its missing
shared plate/gate and escort integration, with native component evidence on
`c6f580e`. Both Jabberwock encounters now have shared combat and personal healing
on `7c3ca24`, verified by 58 staged component cases. Build `ccd6cd1` adds their
paired scenes, personal Eye Staff reward, shared bridge and exit handling, with
55 further world/scene cases, including both post-battle walking orders across
the bridge and through the actual castle exit. Water Logged now has its shared
water machinery, personal currents/fan hits and original introduction on
`d06ded3`, with 66 native paired cases. Follow-up `0e5f061` adds fan-safe
checkpoint/revival placement and nine native recovery checks. The [machinery batch](COOP_MACHINERY.md) now integrates Mystifying Madness,
Labyrinthine Revenge and Machinations. The [castle batch](COOP_CASTLE.md) adds Battle Royale, Ascension and Castle
Keep. The [Queen finale](COOP_QUEEN.md) now closes the last missing explicit
paired adapter on `c8a041e`. All 39 visits have component evidence; normal paired
campaign progression is now accepted for every visit. Burning
Curiosity now has a complete verified normal save chain.

The fresh pre-change checkpoint run on 5 November passed staged shared travel,
guest reconnect, complete primary-save restoration and team-wipe restoration.
The backup attempt restored the world before simulation but its guest could
not resolve the new public invite hostname; that attempt is a failed internet
check, not a backup/network pass. Solo-save continuation passes native
conversion and all four paired internet phases on build `7994cc2`. That earlier
foundation point credited the missing import feature. The current additional
Internet Host/Join point credits completed public core consolidation on `0b5bca3`;
no additional campaign visit is credited.

All 39 visits now have credited normal paired route evidence. Six retain
the historical combined designation; the broader matrix still labels 33 as
partial because it includes polishing cases. The next stage is polishing and hands-on playtesting. Exhaustive scene-owner, recovery, network and presentation permutations
remain in the polishing backlog.

| Area | Credited points | Maximum |
| --- | ---: | ---: |
| Foundation and independent client diagnostics | 9.5 | 10 |
| Internet Host/Join | 10.75 | 15 |
| Two players, movement and presentation | 11.75 | 15 |
| Combat, individual resources and death | 13.5 | 20 |
| Scenes, transport and progression | 16 | 20 |
| Campaign playability | 20 | 20 |
| **Historical full-matrix total** | **81.5** | **100** |

## Playable milestone complete — 8 November 2026

**100% of the agreed playable scope is accepted on `coop`.**
[Start co-op or run two clients on one PC](COOP_QUICKSTART.md).

| Playable milestone | Accepted points |
| --- | ---: |
| Requested core controls, Host/Join, resources, scenes, death, saves and rejoining | 40/40 |
| Normal paired progression through all 39 campaign visits | 55/55 |
| Actual public two-client ending movie and credits handoff | 5/5 |
| **Playable readiness** | **100/100** |

Frozen `cddd820` completes the final public test from the unchanged, normally
earned Queen2 checkpoint. Both clients finish actual combat and present the
ending movie and credits, using the public invite path with 40 ms delay and
20 ms jitter. Ordinary focus/resume and held Enter skip inputs take each film
to credits. **3,445 matching sent/restored player, world and NPC states** agree.
The original checkpoint and host's playable checkpoint remain byte-for-byte
unchanged. Evidence: `private/coop-playable-finale-public-verified.json`.

The build passes **951 active Rust tests** (five ignored), the release build,
source audits and seven packaging checks. The final Pool default reproduces
its accepted route exactly. The thirteen earlier public save/rejoin groups
remain accepted; both solo-save conversion and co-op continuation are covered.
The final changes add test inputs and presentation logs, not combat assistance.

This is the playable milestone, not a claim that every bug or fidelity gap is
resolved. Polishing retains the broader frame-rate/difficulty/device matrix,
scene-owner and skip permutations, per-visit recovery/network edge cases,
visual/audio comparisons and hands-on tuning. A normal-play blocker or data
loss found there takes priority. The historical full matrix remains at 81.5%
as a separate record; it is not the playable milestone's denominator.

## Pool of Tears normal paired route accepted

Frozen `dcdc1d2` passes the final missing normal visit in **67,887 ticks**,
with **566 exact compressed replicas**, four scene reunions and actual Hollow
Hideaway arrival. Both players finish alive at **63.39/25 health**. Three deaths
use normal recovery: each revival occurs after exactly **1,200 active ticks**
at 25 health. There are no team wipes, health floors, cheats or forced progression.

The input pilot takes narrow banks in single file, leaves room for boarding,
and steers/jumps around the third leaf's obstacles using collision queries.
The rope approach uses ordinary movement and E; one player's death there is
recovered normally. This proves a playable route with the requested recovery,
not a flawless deathless traverse. Gameplay physics, scenes and resources are
unchanged. Evidence is pinned in `private/coop-playable-pool-course-verified.json`.
The standard check on `cddd820` exactly matches the accepted commands, players,
world, scenes, revivals and arrival. Its pin is
`private/coop-playable-pool-default-verified.json`; repeat with
`tools/test_coop_native_route.py --map potears1` and local game data.

Normal paired progression now covers **39/39 campaign visits**. The public
save/rejoin core retains its thirteen accepted groups on unchanged `0b5bca3`.
The remaining playable acceptance item is the actual two-client movie/credits
handoff; broader recovery and presentation combinations are polishing.

## Hollow Hideaway normal paired route accepted

Frozen `5fd9df4` passes the complete normal paired visit in **33,897 ticks**,
with **283 exact compressed replicas**, the original Bill scene and reunion,
and actual Just Desserts arrival. Both players finish alive at **78.70/26 health**.
Red dies once and revives at **25 health after exactly 1,200 active ticks**;
there are no team wipes, health floors, cheats or forced progression.

The input pilot takes narrow banks in single file, boards both large leaves,
and steers around obstacles using swept collision queries. Both players make
an ordinary diagonal jump from lily four to the house bank. Lily five's carry
retains its existing component evidence; its alternative landing route is not
claimed by this run. Combat, world machinery, scenes and saves are unchanged.

The accepted course is pinned in `private/coop-playable-hollow-course-verified.json`.
The default diagnostic on `b88c6c7` exactly matches the accepted commands, players,
world and arrival; its pin is `private/coop-playable-hollow-default-verified.json`.
Repeat with
`tools/test_coop_native_route.py --map potears2`, local game data and a new profile.
All 951 active Rust tests pass on `20e7894`; subsequent changes affect only
diagnostic inputs and geometry inspection. Source/package checks pass.

Normal route coverage is now **38/39 visits**, or **81.0%** under the existing
overall weighting. Pool of Tears is the remaining normal visit. The actual
two-client ending film/credits handoff is still required. The thirteen public
save/rejoin core groups remain accepted on unchanged `0b5bca3`.

## School Two normal paired route accepted

Frozen `b925e5c` passes the complete main quest in **53,696 ticks**, with
**448 exact compressed replicas**, **six reunions** and both players finishing
at **100 health**. The run includes the gym lever and bleachers, Gnome battle,
laboratory stairs and rescue, Jumbogrow, both personal portal journeys, greenhouse
growth, lollipop, potion and star rewards, and actual `skool1_start2` arrival.
There are no deaths, revivals, team wipes, health floors, cheats or forced quest
state. Optional Dice collection remains outside this main-quest route; the
normal return fills the missing campaign weapon baseline.

The run exposed a real post-battle reunion lock when the scene owner stood on
the gym's narrow outer trim and the partner had already descended. If ordinary
reunion fails there, co-op now places both players on verified supported ground
near the original scene's Alice mark. Both landings must fit before either body
changes. Personal resources, solo scenes and the save schema are unchanged.
The failing and fixed normal runs have identical inputs, completed steps and
positions through the scene trigger at tick 15,453; the fixed pair continues.

The pilot uses ordinary E for manual doors and walks the moving greenhouse
doorway. It ignores optional enemy drops with no supported approach. These are
test input changes, without altered loot, combat or progression. The standard
command's players, world, commands and arrival exactly match the accepted course
on `61d7ab5`. Evidence: `private/coop-playable-school2-default-verified.json`;
repeat with `tools/test_coop_native_route.py --map skool2` and local game data.

All **951 active Rust tests** pass on the gameplay-fix build `2acd182`; later
changes affect diagnostic inputs only. The original School Two checks also
pass watched/skipped scenes, pause/restore, mover/rider behavior and reward
gates. Source/package audits pass. Normal coverage is **37/39 visits**, with
the two swimming visits remaining, or **80.5%** overall. The rendered two-client
ending handoff remains separate; broader recovery permutations are polishing.

## Duchess normal paired route accepted

Frozen `a4f954c` passes the fresh Just Desserts route in **12,502 ticks**:
actual Jackbomb pickup, original introduction, living boss combat, turtle
rescue and the real temple departure. **105 exact compressed replicas** agree;
both players finish alive at **100/25 health**, with no deaths, revivals, team
wipes, health floors, cheats or forced progression. The watched introduction
reunites the pair; the original outro carries both into the next visit.

Temple arrival preserves personal resources and pickup history, grants the
original breathing-shell flag and fills the partner's missing Jackbomb campaign
milestone. The first attempt reached the exit but failed the diagnostic's
missing shell-grant allowance; the corrected check passes without changing
gameplay. The pilot now includes the legacy combat targets used by production.

Evidence: `private/coop-playable-duchess-default-verified.json`. Repeat with
`tools/test_coop_native_route.py --map potears3`, local data and a fresh profile.
All **950 active Rust tests** pass on the preceding diagnostic build `148a733`;
the final build changes only arrival expectations and flag registration.
Normal coverage is **36/39 visits**, with **3 remaining**, or **80.0%** overall.
This accelerated Arena check leaves internet/rendered-camera evidence separate.

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

## First school normal save chain accepted

The first Skool Daze visit now has verified normal entrance-to-exit coverage
through four connected public-invite segments on `59cc04b`, `d4869cd` and
`ced2c8c`. The unchanged theatre, balcony and spiral saves connect each segment
by exact file hash. Recorded starting and ending positions/resources agree
with those checkpoints; both clients retain ordinary gameplay throughout.

The chain covers all six scenes, personal Mallets, both lifts, the flying-book
bridge, recipe handoff and actual School Two arrival. Both players finish alive
at **12/77 health**, with **15,852 matching network states** and **309 matching
full school reports**. The recorded histories pass the full route assertions
without deaths, respawns or checkpoint rollbacks. This is connected normal
save/resume evidence, not a new uninterrupted run on the latest build.

The private audit is `private/coop-playable-school-chain-audit.json` and can be
rechecked with `private/audit-school-playable-chain.py`. Broader owner/recovery
permutations remain polishing work. Coverage is **34/39 visits**, **5 remaining**,
or **78.9%** overall.

## Queen normal paired finale completed

Frozen `ef316ac` passes a fresh normal paired Heart of Darkness run:
**34,711 ticks**, **291 exact compressed replicas**, both Queen forms defeated,
the original introduction/birth/death/ending sequence, and both players alive
at **100 health**. The first battle ends at about 74/71 health; the original
second-arena refill restores both players. No death, team wipe, cheats, forced
boss damage or progression staging is used.

At tick 18,660 the real second-arena checkpoint is written and restored into a
separate Arena/roster, with exact player and shared-world state. Both replicas
reach the actual durable ending state, with no map transition. The normal
weapons, enemy attacks and production scene/recovery pipeline remain active.
All 949 active Rust tests and source/package checks pass. Evidence is pinned in
`private/coop-playable-queen-default-verified.json`.

Repeat with `tools/test_coop_native_route.py --map qlair`, local data and a fresh
profile. This proves normal paired combat, scenes, checkpoint restoration and
ending state. The rendered two-client film/credits handoff remains a separate
acceptance item; it is not claimed by this accelerated Arena check.
Coverage is **33/39 visits**, **6 remaining**, or **78.4%** overall.

## Machinations playable route completed

Frozen `fcf4979` passes the normal paired course through actual Royal Rage
arrival: **36,116 ticks**, **301 exact compressed replicas**, and both players
alive at **22/25 health**, with personal resources carried exactly. Five normal
revivals each take exactly ten active seconds and return at 25 health. There is
no team wipe, progression staging, cheat state or changed gameplay balance.

The diagnostic uses live lift/pedal phases, precise pillar landings, single-file
wheel boarding and independent movement off the final climbing gear. It omits
the solo route's deliberate fall demonstrations; normal recovery remains active.
Gameplay, saves and networking are unchanged. All 949 active Rust tests and
source/package checks pass. Frozen source, executable, arrival and recovery
evidence is pinned in `private/coop-playable-machinations-default-verified.json`.

Repeat with `tools/test_coop_native_route.py --map tower3`, local data and a
fresh profile. Internet and rendered-camera evidence remains separate.
Coverage is **32/39 visits**, **7 remaining**, or **77.9%** overall.

## Labyrinthine Revenge playable route completed

Frozen `714bd1f` passes the normal paired course through actual Machinations
arrival: **138,944 ticks**, **1,158 exact compressed replicas**, and both
players alive at **95/25 health** with their personal resources carried exactly.
Eight normal revivals each take exactly ten active seconds and return at
25 health. There is no team wipe, progression staging or altered combat balance.

The diagnostic follows live gear poses, individual waypoint and hammer clocks,
ordinary Watch/Staff/Ice controls, and safe regrouping before the exit. Its
combat wait uses normal Will regeneration. Gameplay, saves and networking are
unchanged. All 949 active Rust tests and source/package checks pass. Exact
source, executable, arrival and recovery evidence is pinned in
`private/coop-playable-labyrinth-default-verified.json`.

Repeat with `tools/test_coop_native_route.py --map hedge3`, local data and a
fresh profile. Internet and rendered-camera evidence remains separate.
Coverage is **31/39 visits**, **8 remaining**, or **77.4%** overall.

## Water Logged playable route completed

Frozen `b278fdc` passes the normal paired water course into actual Labyrinthine
Revenge arrival: **28,247 ticks**, **236 exact compressed replicas**, the original
arrival/reunion, all three flushers and the final drain. Both arrive alive at
**55 health**, with personal resources carried exactly. Two Red revivals and
one Blue revival each take exactly 1,200 active ticks and return at 25 health.
There is no team wipe, progression staging or altered combat balance.

The new diagnostic reuses the solo course with ordinary Blade/Cards, Watch,
swimming and per-player fan-gap timing. It keeps the leader moving through
stair landings and reads actual footing after the vacated tray springs back.
The narrow final inlet uses a single-file swim sequence: one swimmer clears
it while the partner waits on the broad platform, then the partner follows.
No gameplay, save or network behavior changed in this batch.

All **949 active Rust tests**, the locked release and source/package checks
pass. The default route passes in about 54 seconds in Anode; its source,
executable, arrival and exact recovery evidence are pinned in
`private/coop-playable-waterlogged-default-verified.json`. Repeat with
`tools/test_coop_native_route.py --map tower2`, local data and a fresh profile.
The unchanged public save/rejoin and controller component evidence is reused;
this Arena pass does not add internet or rendered-camera coverage.
Coverage is **30/39 visits**, with **9 remaining**, or **76.9%** overall.

## Mystifying Madness playable route completed

Frozen `f9d8893` passes the normal paired maze course into actual Water Logged
arrival: **53,709 ticks**, **448 exact compressed replicas**, both lever scenes
and both personal breathing shells. Both players arrive alive at **80/70.59
health**, with personal resources carried exactly and no team wipe or staged
progression. Five Blue revivals each count exactly 1,200 active ticks; the last
also waits eight ticks for safe footing.

The route exposed converging low pull-ups that paused both players forever.
The contact handler now releases one climb without moving either body, allowing
ordinary steering and gravity to clear the contact. Its asset-free regression
verifies backing away and retrying in both orientations, without overlap,
penetration or health changes. All **949 active Rust tests**, the locked release
and seven package checks pass. The affected Airborne Terror route also passes.

The diagnostic reuses the solo course, adds water path planning and takes turns
at the narrow bank. It uses ordinary air steering to leave the upper combat
ledge. Combat and progression rules remain unchanged. Evidence is pinned in
`private/coop-playable-maze-route-verified.json`. Use
`tools/test_coop_native_route.py --map hedge2` with a frozen executable, local
data and a new profile. Internet and rendered-camera evidence remain separate.
Coverage is **29/39 visits**, with **10 remaining**, or **76.4%** overall.
The final default course on `cfcb764` reproduces the same ticks, replicas,
resources and recovery events, pinned in
`private/coop-playable-maze-default-verified.json`.

## Airborne Terror playable route completed

Frozen `fb8a476` passes the normal paired route into actual Mystifying Madness
arrival: **17,513 ticks**, **146 exact compressed replicas**, the original
arrival scene, all four face phases and the upper updraft course. Both arrive
alive at **21/62 health**, with personal resources carried exactly and no wipe
or progression staging. Blue revives twice at 25 health after exactly 1,200
active ticks, followed by 966 and 68 ticks waiting for safe survivor footing.

The route exposed a real wall-contact bug: horizontal retreat could rewind
airborne height while gravity continued accumulating. The fix preserves clear
vertical travel. An asset-free regression fails before the fix and passes for
both player slots, rising and falling. The changed collision code also passes
Battle Royale on `5c28c48` (30,447 ticks, three exact ten-second revivals).
The test controls now take turns at three narrow landings and use independent
waypoint sequences for the final updrafts. No combat or recovery rules changed.

Repeat with `tools/test_coop_native_route.py --map tower1` and the frozen
executable, local data and a fresh profile in Anode. Exact source, executable
and report hashes are pinned in `private/coop-playable-tower1-route-verified.json`.
Internet and rendered-camera evidence remain separate. Coverage is **28/39
visits**, with **11 remaining**, or **75.9%** overall.

## Pale Realm playable route completed

The existing normal Blue-led, watched course passes on frozen `841dcd6`:
**68,146 ticks**, **113 exact compressed replicas**, both chess puzzles, the
shared lift, timed spikes, both levers and all five original scenes. Both
players reach actual Castling arrival alive at **92/100 health**, with unchanged
personal resources, no deaths, wipes, overlap or unscripted warps.

No native code changed. A small public wrapper runs the existing course with a
fresh profile and verifies its receipt. Native source, executable, course and
report hashes are pinned in `private/coop-playable-pale-route-verified.json`;
the run is in `private/coop-pale-playable-1/`. The earlier failed public-client
routes remain failed evidence. This pass proves normal Arena progression and
its native arrival assertions; internet, rendered presentation and recovery
remain covered by separate core evidence and the polishing backlog.

The existing locked release and Rust checks are reused for unchanged native
source; Python compilation, seven package checks and the source audit pass.
Coverage is **27/39 visits**, leaving **12**, or **75.3%** overall.

## Both Caterpillar's Plot routes completed

The first visit passes on `841dcd6`: **33,869 ticks**, **283
exact replicas**, four watched scene reunions and actual Pale Realm arrival.
Both arrive alive at **99.00/100.00 health**, with
unchanged personal resources, the shared Staff component and no deaths or wipes.
The return passes on `d199822`: **9,720 ticks**, **81 exact
replicas, the original wall scene and paid Staff destruction, then actual maze
arrival at **59/100 health**. Its optional Blunderbuss is supplied by the existing
campaign arrival grant; other personal resources carry exactly.

The automated first-visit controls now spend ordinary Will on ranged cards and
approach the exit passage without projecting its marker onto the roof. An
earlier knife-only run wiped, and the roof search was cancelled; neither is
credited. A subsequent calibration reached the correct exit but exposed a
verifier mistake: only the return uses the controller's timed exit flag. The
final default run checks the first visit's actual gated transition and arrival.
Gameplay, enemy damage and progression rules are unchanged.

The batch passes 947 active Rust tests and the 39-visit deterministic registry
check on initial `d199822`, followed by the diagnostic refinements, final locked
release and native route. Source/package checks pass. Evidence is pinned in
`private/coop-playable-forest-first-verified.json` and
`private/coop-playable-forest-return-verified.json`. Existing internet/core and
rendered-camera evidence remain separate. Coverage is **26/39 visits**, leaving
**13**; estimated overall completion is **74.8%**.

## Fungiferous Flora playable route completed

Frozen `906f8c7` reaches actual Sanctum arrival through the normal paired course:
**12,471 ticks, 104 exact compressed replicas**, one watched ambush reunion,
and both players alive at **10/47.57 health**, without deaths, wipes or staging.
Personal health, consumables and pickup history carry exactly. Blue collects
the optional second Die; Red receives the existing campaign weapon grant on
arrival. Both temporary countdown health floors clear at the new visit.

The new course reuses the solo waypoints and the existing native paired driver.
Gameplay is unchanged. Its verifier now accounts for the normal arrival grant
and the final scripted departure taking ownership after the watched reunion.
The final native run exits successfully; the corrected verifier accepts that
saved evidence without rerunning the unchanged simulation. The initial failed
checks remain recorded. Evidence: `private/coop-flora-playable-2/` and
`private/coop-playable-flora-route-verified.json`.

The same frozen build repeats Battle Royale successfully: 30,506 ticks, 255
exact replicas and both ten-second revivals, now checked against 2,637 exact
recovery trace rows. It preserves the previous outcome and onward resources.
Validation includes 947 active Rust tests, the final locked release and source/
package checks. Internet and rendered-camera evidence remain separate and are
reused for unchanged systems. Coverage is **24/39 visits**, leaving **15**.

## Icy Reception playable route completed

Frozen `cd89026` passes the fresh paired normal route through the actual
Fungiferous Flora arrival: **21,406 ticks**, **35 exact compressed Arena
replicas**, both personal Ice Wands, both players' two vent rises and two
original watched scene reunions. Red activates the marble scene first; Blue
later activates the Caterpillar scene. Both arrive alive at about **23.29/88
health**, with their complete personal resources preserved, without a team wipe,
cheats, body staging or forced progression.

Blue dies at tick 2,348 and revives at 4,389 with 25 health. The trace verifies
exactly **1,200 active countdown ticks**, plus **841 paused cutscene ticks**,
safe placement and unchanged personal resources. The test then continues from
the survivor's observed waypoint. This credits normal progression with recovery,
not a flawless run or new rendered-spectator/internet evidence.

The new automated course reuses the solo waypoints and normal inputs. Earlier
attempts exposed driver limitations: it cut planned landings short, stopped
before a partner could reach a reunion, and omitted the pull-up input after a
ledge grab. The final driver handles these normal states with bounded waits.
Gameplay physics, hazards, combat and revival rules did not change.

The batch passes **947 active Rust tests** on initial `84b284a`, followed by the
diagnostic control refinements and the final locked release. That initial build
also passes the existing Normal/Easy solo routes into Flora. Source/package
checks pass. Evidence is pinned in `private/coop-icy-playable-4/` and
`private/coop-playable-icy-route-verified.json`. Public-invite/core evidence is
reused for unchanged systems. Coverage is **23/39 visits**, leaving **16**;
estimated overall completion is **73.3%**.

## Rolling Stones playable route completed

Frozen `7710f92` fixes a normal co-op blocker: the follower could toggle a shared
chase gate back, trapping both players before the marble arrived. The four
one-way chase markers now consult existing saved dispatch history and fire once
for the shared co-op world. Solo repeatability, save format and rule identities
are unchanged. Rejected contacts do not consume the shared event.

The fresh paired route now reaches actual Icy Reception arrival in **10,699
ticks**, with **89 matching compressed Arena replicas**, the original watched
arrival/reunion, both mushroom launches and unchanged personal resources.
Both players finish at **86 health**, with no deaths, revivals, wipes or cheats.
Fast exit need not wait for the marble's later ice callback or every gate latch;
the separate shared-clock component checks retain that coverage.

Validation: **947 active Rust tests**, locked release, the existing 11 native
chase component cases plus eight leader/trigger restore cases and a solo toggle
check. The full Herbaceous route also repeats successfully on this build:
33,190 ticks, 56 replicas and five exact ten-second revivals. Source/package
checks pass. These are production-Arena routes; existing public-invite and
rendered-camera evidence remains separate and is reused for unchanged systems.

Evidence: `private/coop-rolling-playable-3/`,
`private/coop-playable-rolling-route-verified.json`, and
`private/coop-herbaceous-gates-regression-1/`. Normal paired coverage is now
**22/39 visits**, leaving **17**. Overall estimate: **72.8%**.

## Herbaceous Border playable route completed

Frozen `4bb7dac` passes the fresh Blue-led paired route through actual Rolling
Stones arrival: **33,190 ticks, 56 exact compressed Arena replicas and four
watched scene reunions**. Five deaths recover after exactly 1,200 active ticks
at 25 health, with safe placement and preserved resources. Both arrive alive
at 25/46 health, with their full personal inventories unchanged, without a wipe
or gameplay-state staging. The ledge approach uses ordinary planned controls.

This is playable-route acceptance with recovery, not complete pickup coverage:
Blue collects the Mallet; Red revives beyond its approach and misses it. The
playable check records this ownership and accepts the actual progression and
unchanged individual inventories. Strict mode still requires both pickups.
The prior attempt was rejected under that stricter pickup requirement; no
weapon, health, damage or world-progression rule was changed to obtain this pass.

Evidence: `private/coop-herbaceous-playable-8/` and the pinned source/executable
audit `private/coop-playable-herbaceous-route-verified.json`. The batch passed
946 Rust tests, 15 recovery guards and seven packaging checks. The final release
also passes; the full unit run precedes the last diagnostic steering/acceptance
refinements. Existing core/internet evidence is reused for unchanged systems.
Coverage is now **21/39 visits**, leaving 18 normal routes.

Earlier Rolling Stones paired attempts failed at approach 16 before a
marble wipe. The later shared-gate fix above resolves that blocker. Its current solo fast course passes
through Icy Reception; the subsequent optional paced course dies earlier.
These are distinct outcomes, not a passing combined solo command.

## Dry Landing playable route completed

Frozen `5c1b2dd` passes the fresh Red-led normal paired Garden route through
actual Garden2 arrival: **29,719 ticks and 50 exact compressed Arena replicas**.
Both players traverse the course, watch the original scenes and reunite twice.
Both arrive alive at approximately 54.88/62.18 health with every personal
resource preserved; this run has no deaths, revivals, team wipes or cheats.

The native driver now supports ordinary revival, discards pre-death inputs and
continues beside the actual survivor. Failed Blue-led runs exercise that path
but do not establish a complete Blue-led route. The default public native runner
selects the passing Red-led order. No gameplay movement or combat balance changed.
The repeated Blue approach/slope impact and its later crossing failures remain
follow-up investigations; the two new small slope fixtures pass but do not
reproduce that native impact or establish a fix.

Validation: 946 Rust tests and the locked release pass, alongside 15 recovery
guards and seven source-package tests. Native evidence is in
`private/coop-garden-revival-native-7/`; the pinned audit is
`private/coop-playable-garden-route-verified.json`. This establishes the normal
paired Arena route, not a new internet or rendered spectator-camera check.
Existing independent-client core acceptance remains applicable. Coverage is now
**20/39 visits**, leaving 19 normal routes.

## Castle Keep playable route completed

Frozen `b22c2b4` passes the default normal paired Castle Keep course through
actual Queen arrival: **80,098 ticks and 668 exact compressed Arena replicas**.
Ordinary controls activate all three mirror scenes, climb the steam column
three times and shoot the three correct portraits. Both players reach each
upper landing. Red later dies on a descent and revives beside Blue after
exactly 1,200 ticks, with no team wipe, cheats or staged progression.

This route exposed a real resource bug: fresh guard waves reused earlier
enemies' reward IDs, preventing later healing essence from dropping. Each
spawned group now has a saved reward generation; existing version-2 saves
default to generation zero and retain their original IDs. The native fixture
verifies three defeated waves, six fresh personal drops per player, duplicate
suppression and world/loot reloads, including old saves without the new field.

The first walker continues past the final gate to trigger the original
single-Alice scene. The waiting partner stays frozen, then the game reunites
both for the actual Queen arrival. Both arrive alive with all personal resources
unchanged: approximately **64.47/53.47 health and 99/99 Will**. The test driver
now supports ordinary lever use and portrait shots, single-file cornering and
room for both bodies at a shared landing; gameplay movement is unchanged.

Validation: **944 Rust tests passed**, locked release build, solo and paired
Keep component checks, seven source-package tests and source audit. Tests,
components and the repeated Ascension route use `fe24cb8`; only the Keep
diagnostic's final corridor commands changed afterward. The final Battle Royale
regression also passes again at 30,506 ticks, 255 replicas and actual Ascension
arrival. Ascension repeats at 63,444 ticks and 529 replicas with its shared lift.

Private evidence: `coop-keep-native-7-keep`, `coop-keep-native-7-grounds2`,
`coop-keep-native-6-facade`, `coop-keep-contract-6` and
`coop-playable-keep-route-verified.json`. This is normal production Arena and
compressed-replica evidence, separate from unchanged public-invite/core tests.
It does not establish rendered spectator cameras or independent internet
clients for this route. Earlier failed/cancelled attempts receive no credit.

This adds Castle Keep: `61.5 + 20 * 19 / 39 = 71.24`; **20 visits remain**.

## Ascension playable route completed

Frozen `4d8acf0` passes the default normal paired Ascension course through the
actual Castle Keep arrival: **63,444 ticks and 529 exact compressed Arena
replicas**. Both players enter both steam columns. Both finish the first climb;
Red's second climb includes normal death and revival beside Blue on the landing.
All five Red deaths recover after exactly 1,200 ticks (ten seconds), with no
team wipe, cheats, staged progress or position writes.

The automated players clear narrow pickup platforms in sequence. Navigation
waits for a grounded landing before turning, and uses nearby partner spacing.
These are diagnostic input changes; gameplay collision and movement are
unchanged. The first rider waits 151 ticks while the partner approaches. Actual
deck contacts confirm both aboard before the Blue-owned original lift scene.
The hidden partner stays frozen during the scene. The normal 4.5-second ride
commits the exit and places both living players in Castle Keep, preserving all
personal resources: 88/88 health and about 62.80/87.60 Will.

An earlier shared-driver regression stalled Battle Royale at a waypoint.
Its proven nearby-marker navigation is now kept separately from Ascension's
landing-aware course. The final default Battle Royale repeat again passes
**30,506 ticks, 255 exact replicas, all 30 recurring guards defeated and actual
Ascension arrival**, matching its prior successful route.

Validation on the final source: **944 Rust tests passed**, locked release build,
seven source-package tests and source audit. Private artifacts are
`coop-ascension-native-9-facade`, `coop-ascension-native-9-grounds2` and
`coop-playable-ascension-route-verified.json`. This is production Arena and
compressed-replica evidence; unchanged public-invite/core checks remain separate.
It does not establish rendered spectator cameras or separate internet clients
for this route. Earlier failed/cancelled attempts receive no route credit.

This adds Ascension: `61.5 + 20 * 18 / 39 = 70.73`; **21 visits remain**.

## Battle Royale playable route completed

`57aa72b` passes the default accelerated paired Battle Royale route from its
normal entrance to actual Ascension arrival. The production Arena runs both
players' ordinary navigation, weapons, Watch, damage, personal loot and revival.
All eight recurring companies reach their normal counts, totaling 30 defeated
guards. The original introduction reunites both players. No world progression,
health, damage outcome or reward is staged.

The fresh default run takes about 19 seconds for 30,506 simulation ticks and
255 matching compressed arena replicas. Blue dies at tick 4,433 and revives at
5,633; Red dies at 10,746 and revives at 11,946. Both countdowns are exactly ten
seconds, without a team wipe. Both arrive alive with unchanged personal stats:
17.76/100 health and 81.20/100 Will. Internet transport and spectator rendering
remain covered by their separate core evidence, rather than this native run.

The first attempt stopped when two automated approach paths blocked each other
at a redundant solo turn marker. The normal path to the following marker works
for both. That tested sequence is now the default; no game collision, enemy or
damage rules changed. The reusable runner accepts optional ordinary-input
command files, allowing later route adjustments without recompilation.

All 944 active Rust tests passed on the initial runner build; only the default
diagnostic waypoints changed afterward. The final locked release and fresh
default native route pass, as do seven package checks and the source audit.
This credits one normal route: `61.5 + 20 * 17 / 39 = 70.22`; 22 remain.

## Royal Rage playable route completed

`4f1d1ea` extends the accelerated normal paired runner to Royal Rage. Both
players use their normal entrance inventories and ordinary movement/weapon
controls to defeat the 2,000-health Jabberwock, finish the original introduction
and Gryphon farewell, wait for the drawbridge, cross to the opened gate and
trigger the actual Battle Royale arrival. No combat outcome or progression is
staged. Both finish alive at 100/72.27 health without revival or team wipe.

The route covers 18,802 simulation ticks and 157 exact compressed arena
replicas. Both players reach the gate before the leader activates the exit;
the next map receives their unchanged personal resources. The shared runner's
Lair regression also passes all 24,348 ticks and 203 replicas, preserving both
Eye Staff rewards and forest arrival. These are normal production-Arena routes;
independent-process public-invite/core evidence remains separate.

All 944 active Rust tests, the locked release build, seven package checks and
source audit pass. Gameplay code is unchanged. This credits Royal Rage once:
`61.5 + 20 * 16 / 39 = 69.71`; 23 normal paired visits remain.

## Jabberwock survival playable route completed

`bfdf5b3` adds an accelerated two-player production-Arena route from the normal
Jabberwock's Lair entrance through the original introduction, 90-second fight,
30/60-second waves, original outro, both personal Eye Staff pickups and actual
`wforest_start2` arrival. Ordinary controls drive both players; combat damage,
healing and collision remain active. Both finish alive at 100/75 health without
revivals or a team wipe, and their carried resources match the forest arrival.

The run covers 24,348 simulation ticks in about eight seconds and checks 203
exact compressed arena replicas. It uses Anode's graphics context for the
production characters but does not render or inspect screenshots. This is a
fresh normal paired simulation, not an independent-process internet run;
existing public-invite/core-flow evidence remains separate. This faster route
method can cover normal progression while exhaustive network permutations stay
in polishing. The first attempt's final assertion incorrectly expected wave
actors to survive outro cleanup; the final check instead records their actual
three/six-actor appearances at ticks 3,600 and 7,200.

All 944 active Rust tests, the locked release build, seven package checks and
source audit pass. Gameplay code is unchanged; the new command and its registry
declarations are diagnostic additions. This credits one normal route:
`61.5 + 20 * 15 / 39 = 69.19`; 24 visits remain.

## Burning Curiosity playable route completed

Twenty-four connected ordinary save/resume sections now cover the entrance,
original watched Caterpillar scene, both players' climbs and actual paired
Jabberwock arrival. Their unchanged saved bytes and native resume receipts
connect the historical approach to the new upper route and exit on `40d06c0`.
The retained sections contain 43,586 matching player/world/native state samples.
This is a verified save chain, not a new uninterrupted run of the entire level.

The upper test now lets one player finish the narrow ledges before the partner
follows. Each uses an ordinary continuous jump beyond the sloped lip; actual
damage remains active during the short upper crossing. The original next-level
scene releases Alice above the floor, so the arrival check now observes her
normal landing within one second rather than requiring immediate grounding.
Both clients pass the final exit check, with 1,383 matching states. Earlier
failed attempts remain recorded as failures; only their separately verified
prefixes contribute to the connected chain.

All 38 focused Curiosity guards and seven package checks pass. Native code is
unchanged from the already tested build. Optional pickup/Watch choreography,
extra scene-owner permutations and repeated full traversals remain polishing.
That milestone added one normal-route credit: `61.5 + 20 * 14 / 39 = 68.68`,
leaving 25 visits before the Jabberwock survival pass above.

## Connected Clockwork and Funhouse routes

Clockwork's retained normal save chain now verifies all 150 recorded course
actions from the entrance to About Face across 26 connected sections. A missing
14-16 prefix was replayed through its frozen verifier and actual saved boundary,
closing the hash connection to the remaining route. Funhouse similarly verifies
20 connected sections, 156 recorded course actions, actual boss defeat, the
original ending and both players arriving in Hatter1. Rechecking its individual
edges confirms 18,637 matching state samples and two exact ordinary revivals.

Every outgoing checkpoint matches the next section's input byte-for-byte,
and native resume receipts confirm restoration before simulation. These
historical build-specific routes satisfy the playable save-chain criterion;
they are not new uninterrupted runs on the latest executable. Three Clockwork
receipts with missing or mismatched observation hashes were excluded from the
selected chain. This adds two normal-route credits: `61.5 + 20 * 13 / 39 = 68.17`.

Current component checks on `42139c0` pass all 41 Clockwork scene groups and
all 150 Funhouse combat/world/scene groups. The Clockwork movement check exposed
a regression in diagnostic gap steering: the stricter full-footprint check
intended for discarding a planned jump also changed direct steering's damping.
`40d06c0` separates those decisions while retaining the stricter planner handoff.
The existing actual-map gap probe passes all six delays, and the complete
Clockwork movement check now passes all 72 groups. Normal player controls are
unchanged. All 944 active Rust tests, the locked release build, seven package
checks and the source audit pass. These focused checks do not claim fresh
full routes for the other visits.

Burning Curiosity's earlier approach and climb segments are now connected to
the completed upper route and actual exit described above. See the
[continuation evidence](JLAIR1.md#playable-route-continuation-7-november-2026).

## Playable route consolidation

UTemple now completes a fresh public-internet route to Garden with four verified
ordinary revivals and 2,813 matching state samples. Complete historical two-client
routes for Sanctum, Castling, Checkmate and About Face were also audited and
credited under the playable milestone. This adds five normal-route credits to
the existing six: `61.5 + 20 * 11 / 39 = 67.14`. Four credits reuse earlier
passes; this is not a claim that five levels were newly implemented or that
those four were rerun on the latest build.

Garden's strict test stopped at a normal combat death. Its automation now keeps
time while spectating, and both Garden and UTemple offer verified-revival route
checks in the batch runner. Build `42139c0` passes 944 active Rust tests; the
fifteen recovery guards and seven source-package checks also pass. Garden's
retry verifies three individual revivals and continued traversal beside the
survivor, then fails at a late combat team wipe. Its full route remains
uncredited; improving the automated combat route is separate from the already
passing core recovery milestone. See [current route scope and
commands](COOP_PLAYABLE.md#normal-route-acceptance-7-november).

## Latest finale batch

Build `c8a041e` adds paired Queen combat, individual healing, personal grabs,
shared phases and durable completion. Level-requested co-op checkpoints are now
saved after the scene handoff; completion is sent to the guest before the host
leaves for the original film/credits. The twelve focused native groups pass,
including ordinary initial-area and requested second-arena disk saves and exact restore. Native solo
finale, registry and movie decoding also pass. The initial batch passed 943 Rust
tests; final focused native checks cover the subsequent impact-radius and checkpoint-acknowledgement fixes.
450 Python guards and seven package checks pass.

This closes the last explicit controller gap and adds 1.5 combat and 1.5
scene/progression points, reaching 64.58 (reported as 64.6%). It does not credit
a new full paired route. See [scope and evidence](COOP_QUEEN.md). The next work
is normal paired campaign progression; film/credits visual review and exhaustive
permutations are deferred to polishing.

## Latest castle batch

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

This adds 0.25 movement, 0.5 combat and 0.75 scene/progression points:
61.58, reported as 61.6%. No full campaign visit is newly credited.

## Latest machinery batch

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

This adds 0.5 movement, 0.25 personal-hazard and 0.5 scene/progression
points: 60.08, reported as 60.1%. No full campaign visit is newly credited.

## Implemented features and remaining work

The percentage is a conservative, manually weighted readiness estimate, not
the fraction of code written or the current position in the story. Burning
Curiosity is visit 26 of 39, but the work consists of separate level and
checkpoint tests. Earlier visits still have outstanding acceptance cases.

The working features below have evidence in covered scenarios; they are not
yet verified across every level and failure condition.

| Area | Implemented and demonstrated | Remaining work |
| --- | --- | --- |
| One-PC setup | `coop` branch; independent game processes, profiles/saves and automated state/log checks | Finish missing campaign routes; repeat affected checks when behavior changes |
| Host / Join | Main/ESC controls, public internet menu flow, authenticated transport, prompt handshake cancellation, failure/retry recovery and countdown/cutscene/moving-lift/loading-barrier reconnect cases | Other transport/recovery cases and public provisioning/DNS availability |
| Players and presentation | Blue/red Alice, separate input/cameras, body separation, replicated weapon effects | Broader contact/effect checks and reliable traversal through all swimming, leaves, ropes and moving supports |
| Combat and resources | Nearest-visible targeting, individual damage, inventory and consumables; all explicit paired boss adapters integrated | Verify remaining full combat routes and fix normal-play blockers |
| Death and saves | Exact ten-second countdown, moving spectator camera, safe 25-health revival; shared checkpoint wipes, solo import, earlier co-op resume, public save/leave/continue and storage-error/backup menu flow | Representative recovery during remaining routes; exhaustive per-visit matrices deferred to polishing |
| Scenes and machinery | First-arrival ownership, original single-Alice scenes, hidden partner and safe reunion; public scene reconnects and all-living boarding; adapters through the ending | Verify remaining normal transports/exits and the final film/credits handoff; exhaustive owner/skip permutations deferred |
| Regression checks | 949 active Rust tests on `b278fdc`; existing native registry/component checks and build-specific source guards, weapon-effect/audio and solo baselines | Missing normal campaign routes and focused checks for subsequent fixes |

Campaign status from `COOP_CAMPAIGN.md`:

- **6 visits have combined verification evidence:** Dementia/opening village,
  Pandemonium, Fortress of Doors (first visit), Beyond the Wall, Fortress of
  Doors (return visit), and Skool Daze (return visit). The first five retain
  historical build-specific passes and still require contact-system repeats.
- **33 visits have partial evidence under the historical broader matrix.** Some already have complete internet
  traversals, including Wholly Morel Ground, Castling and About Face, but still
  have outstanding reliability, recovery, reconnect or scene cases. Others
  currently have components or checkpoint segments only.
- **No visit remains without paired component evidence.** Heart of Darkness's
  adapter is integrated. Component coverage does not complete the full campaign
  playability pass: 36 normal paired routes are credited, with 3 remaining.

See the [full campaign matrix](COOP_CAMPAIGN.md). Future updates should separate
working features, partial route milestones and completed verification visits.

### Airborne Terror playable integration

Build `3e96892` adds the shared co-op face clock. Each gust advances once
per world step and drives both players, enemy pushes and the existing face
animation/effects state. Normal contacts can activate and repeat all four
faces, including the overlapping middle triggers. Shared Watch time now
uses only living owners and the latest overlapping expiry; machinery,
combat, client particle clocks and audio use that same calculation.

`--tower1-coop-check` passes 41 native cases: three clock/parity cases,
twelve trigger/repeat cases, twelve ordinary player-push cases, twelve
original arrival variants and both exit-trigger owners. Compressed state
continuation is covered. The ten-case solo log is byte-identical to the
pre-change `0e5f061` baseline. The production code passes 942 Rust tests;
the final native run follows fixture-only corrections. Existing saves keep
their format and shared face phases.

These staged components establish core integration. A normal paired route
through the level remains part of the playable campaign pass. Detailed
per-visit recovery/rejoin, visual/audio comparison and difficulty/scene
permutations belong to the later polishing stage unless they expose a
normal-play blocker. See [playable milestone](COOP_PLAYABLE.md).

This adds 0.25 scene/progression points and changes Airborne Terror from
Not run to Partial: six combined, twenty-six partial and seven not run.
The estimate is 58.83 (58.8%). Remaining controller integration and a
consolidated playable campaign pass take priority over expanding the
already-passing component matrices.

### Water Logged safe placement checkpoint

Build `0e5f061` adds a conservative full-body exclusion for the complete
rotation of all five native fans. It applies through the common checkpoint,
solo-conversion, reunion and revival hazard predicate, even during Watch.
Actual blade-contact damage and ordinary movement collision are unchanged.
All 75 paired native cases pass, including nine new fan-boundary/checkpoint
and both-player timed revival cases. The latter verify spectator selection,
1200 simulation ticks, compressed countdown continuation, 25-health revival
and waiting until a safe nearby location becomes available. The held case
stages the survivor's later location; it does not certify escape traversal
or an internet rejoin. The 16-case solo log remains byte-identical to
`291af36`; 941 Rust tests, 450 Python guards and seven package checks pass.

This closes a core recovery gap and adds 0.25 combat/death points, bringing
the weighted estimate to 58.58 (58.6%). No campaign visit is marked complete;
full Water Logged combat/traversal, rendered effects and per-visit network,
rejoin/death/wipe coverage remain open. The failed initial fixture used an
invalid Watch timer combination and is retained separately; the passing
fixture activates Watch through the normal game method.

### Water Logged core integration checkpoint

Build `d06ded3` adds one shared machinery update, atomic living-rider carry,
either-player weighted flusher controls and separate current cooldowns and fan
hits. Both living players receive the original shell/air preparation; air
sources refill only their own contact. Either living player's Watch freezes
the original water/lid/fan clocks while weighted trays retain their normal
motion. Dead players neither weigh down trays nor block fans or receive pushes.
Optional paired state preserves old solo stages, pending damage ownership and
remaining cooldowns; malformed state is rejected before replacing live state.

The headless paired check passes 66 cases: 48 machinery/hazard/swimming cases,
12 original-introduction combinations, and six physical flusher-trigger cases.
Both swimming orders cross the live drain using ordinary controls and preserve
an exact saved future. Either Alice activates all three native flusher triggers
through normal body physics. Both scene owners, early/late skips, watched,
paused/restored playback, dead partners and safe reunion are covered. The
original 16-case solo check has byte-identical output to frozen `291af36`.

This adds 0.5 scene/progression points and makes Water Logged Partial, with no
full campaign verification points. Full two-client traversal with native combat,
rendered effects and recovery/rejoin acceptance remain open. The subsequent
safe-placement checkpoint above closes the fan-sweep revival exclusion gap;
it does not complete the visit's full recovery matrix.

### Jabberwock core integration checkpoint

The follow-up on `ccd6cd1` passes 21 world/reward cases and 34 Stage/scene cases:
both original introductions and finales, both scene owners, watched/skipped,
pause/restoration, frozen partners, dead partners and safe reunion. Each Alice
collects their own Eye Staff and ordinary personal Will refill; departure waits
for both collections and retries automatically after a failed transition.
The drawbridge begins after reunion, carries living riders atomically and stops
if either body is obstructed. Both ordinary-input post-battle crossing orders
reach the castle's real exit trigger. The 58 combat/healing cases also pass;
all four original solo check/route logs exactly match `7c3ca24`.
This adds 0.5 scene/progression points, with no full campaign verification points.
The remaining scope includes full two-client combat-to-exit routes, rendered
effects and the encounter-specific death/wipe/rejoin matrix. Core integration
continues with the eight controllers listed above.

Build `7c3ca24` adds one shared combat clock, closest-visible living targets,
stable attack ownership across saves, physical damage to either body and
individual eight-second healing cycles in both encounters. Fifty-eight native
cases cover both slots, cover, dead/hidden players, six Lair wave targets,
30/60/144 Hz solo combat parity, Pocket Watch rules, compressed continuation,
solo-save conversion and invalid-save rejection. Both existing solo checks and
both production-input solo routes have byte-identical logs to frozen `c6f580e`.
This credits 0.5 combat/resource points. It adds no full campaign verification
points and no two-client route, rejoin, death/recovery or rendered-effects pass.
The scene, reward and exit hooks are covered by the follow-up described above.
On `7c3ca24`, the full Rust suite passes with four test workers. Its default-concurrency run
hit the previously observed 750 ms socket-cancellation timeout; that unchanged
test also passes alone. Both receipts are retained; the later investigation
and fix follow below.

On `ccd6cd1`, the same cancellation test fails both in the four-worker suite
(939 passed, one failed) and in isolation. A separate instrumented reproduction
observes Windows rejecting shutdown on the duplicated connecting socket with
`WSAENOTCONN`, while the worker remains Connecting. This is now tracked as a
connection cancellation bug; the Jabberwock component and solo checks all pass.

Build `291af36` fixes cancellation by sharing one socket handle between the
handshake worker and Cancel instead of duplicating it. The existing 750 ms
deadline is unchanged. All 940 active Rust tests pass at default concurrency,
and the production test binary passes 128 repeats of the eight cancellation
variants (1,024 cases: Join/readiness, plain/TLS, Cancel/drop). The earlier
failure logs are retained. The frozen release also passes 16 ordered exchanges
through a fresh public HTTPS invite, using separate host/guest processes and
profiles with 40/20 ms delay/jitter, followed by clean disconnect and helper
shutdown. All seven Jabberwock component/scene/solo logs exactly match `ccd6cd1`.
This transport check does not repeat the earlier gameplay/save acceptance suite.
The repair adds no percentage or campaign-visit credit.

## Core readiness priority

The focused checklist is now consolidated on `0b5bca3`, including public
internet runs. These checks remain separate from the 39-visit campaign matrix.

- Verified on the initial import build: existing co-op primary resume and team
  wipe; loaded solo progress converted into two players; manual co-op saves;
  fresh host/guest restarts; guest rejoining imported, resumed and backup sessions;
  unchanged solo saves; cached first/return visits; host-owned guest resources.
- Also verified on prior build `a9b55a5`: swimming import and the complete
  import/rejoin/resume/backup internet suite. The separate team-wipe persistence
  suite passed travel/reconnect and primary resume/wipe on the same build; its
  backup guest hit the DNS failure again, so that whole internet run did not pass.
- Verified on build `7994cc2`: real Windows primary/backup replacement failures
  preserve both committed files and the previous in-memory team-wipe state;
  stale temporary files are ignored; a successful save updates the wipe point;
  saving after primary corruption preserves the last good backup. Abrupt host
  loss leaves the save intact, ends the diagnostic guest connection, and permits
  fresh host/guest resume and rejoin over the internet.
- Verified through real menu input on loopback: load solo, Host this game, Join,
  Save checkpoint, Leave, exact suspended-solo return, ordinary solo movement,
  expired-link feedback and Continue co-op with a fresh invite. Two solo returns
  preserve the full state hash. Invalid/batched input and Cancel also pass.
- Verified on build `0b32c79`, with delayed loopback clients: a fresh guest
  reconnects during either player's death countdown. The waiting host holds its
  world/life clocks; personal resources, body identity and remaining time survive.
  Both cases continue through exact ten-second revival and a shared checkpoint wipe.
- Verified on build `4b3760a`: the complete real-input menu flow over public
  internet invites, including two exact suspended-solo returns and a fresh
  invite for Continue co-op. The provider responded after an hour without retries.
- Also verified on `4b3760a`: guest process restart during the theatre introduction
  and dialogue, with either player owning the watched scene. All four loopback
  cases preserve clocks, ownership, starting resources and the hidden partner,
  then complete the original scene and safely reunite both clients.
- Verified on `4b3760a` through public invites: guest restart during the rotating
  spiral lift, with either player boarding first. The lift waits for its second
  rider, holds its full world/body state during disconnection and completes the
  ride after rejoining with both attachments and starting resources preserved.
- Verified on `9921a3e`: guest restart while the host waits at the normal loading
  barrier, both outbound and returning to a cached school visit. Both cases pass
  over loopback and public invites with 40/20 ms delay/jitter, retaining personal
  resources, visit history and cached world state. Native socket tests cover four
  interrupted loading stages, including either client's readiness boundary.
- Verified on `0c5e717`: real-menu recovery from DNS failure, HTTP rejection,
  cancelled Join and failed/cancelled Host setup, followed by a successful public
  join, checkpoint save, leave and exact solo return in the same game processes.
  Cancel now closes an established but stalled TLS/WebSocket handshake socket.
- Also verified on `0c5e717` through public invites: both death-countdown guest
  restarts and all four theatre owner/introduction-or-dialogue reconnect cases,
  each with 40/20 ms delay/jitter. The same frozen build completes exact revival,
  shared checkpoint recovery, watched scenes and safe reunions on both clients.
- Verified on `c5cb7fc`: real-menu primary/backup write failures, successful retry,
  fresh-process backup continuation/repair and both-corrupt feedback. Loopback and
  public-invite runs preserve all three suspended solo returns and the solo slot.
- Also verified on `c5cb7fc`: normal-menu save refusal during either player's
  countdown and both theatre owners' introduction/dialogue, followed by successful
  safe retries. All three public cases use the same final fixture and 40/20 ms
  delay/jitter; timers, cameras, original scenes and saved files remain correct.
- Verified on `c5cb7fc`: public guest restart during either player's underwater
  death countdown in the Blue-led recovery route, with 40/20 ms delay/jitter.
  Exact first-frame body/air/world/enemy agreement, both revivals and shared wipe
  pass. The Red-led attempt wipes near the first column before reconnecting;
  it remains failed route evidence, not a second passing order.
- Verified on `c5cb7fc`: public guest restart during either death countdown
  beside a rope survivor, followed by both exact ten-second, 25-health rope
  revivals. The survivor uses ordinary climbing and attacks against live enemies;
  no health, bodies or enemy state are edited.
- Verified on `c5cb7fc`: public guest restart during either death countdown
  beside a survivor on Funhouse's tilting platform. Both exact ten-second,
  25-health revivals occur while the brush moves; neutral revived bodies remain
  supported and are carried with it. First-frame player/world/enemy hashes agree.
- Earlier consolidation on `c5cb7fc` passed both save families, fresh-process
  resume/rejoin, host loss, backup and team-wipe restoration, plus the normal
  public menu and failure/cancel/retry flows, plus player contact/ambient effects.
  Its next network case failed invite DNS before gameplay, leaving network,
  scene and lift repeats open at that point. Those focused repeats now pass on
  `0b5bca3`; see the current consolidation checkpoint below.
  External DNS/provisioning availability remains a limitation despite verified
  error/cancel/retry handling.

### Core acceptance checkpoint

This separates completed feature checks from the remaining core work. A passed
row does not certify all 39 campaign visits or erase the build-specific limits
in the evidence below.

| Core area | Current evidence | Next required check |
| --- | --- | --- |
| Existing saves | Solo import and co-op resume, individual resources, cached visits, fresh processes, unchanged solo files, normal-menu refusal/retry while dead or in a scene; fixed-build import/resume/backup/host-loss/wipe repeat passes | Core save cases covered; each campaign visit still needs its recovery matrix |
| Storage failures | Real locked-file failures, retry, corrupt-primary backup recovery/repair, both-corrupt feedback; public menu tests | Covered for these cases; electrical power loss and disk exhaustion are not simulated |
| Host/Join lifecycle | Public main/ESC flow, authenticated invite preflight, fresh invites, invalid/expired links, DNS/HTTP failure, Cancel, retry, exact solo return on `0b5bca3` | Public provisioning can still rate-limit new sessions |
| Rejoining | Public passes on the frozen core builds for idle gameplay, either player's ground/underwater/rope/moving-platform countdown, either theatre owner, rotating lift and both loading barriers | Per-visit matrix; underwater proof uses the Blue-led route and does not close the Red-led trap-repeatability gap |
| Combat/death | Shared targets, personal resources, exact countdown, spectator camera, 25-health reunion, shared wipe and moving-platform revival/carry; focused current-build public consolidation passes | Level-specific enemies and recovery matrices |
| Presentation | Both Alice models, contact separation and replicated ambient/weapon effects; `66ebd44` repeats weapon rendering/network checks, verifies survivor-camera audio inputs, shared-mixer output and actual two-client puzzle/dialogue audio | Campaign-specific effects/audio; subjective sound quality is not certified by these automated checks |
| Consolidation | `0b5bca3`: 934 native tests, both public menu flows, eight public network cases, and all thirteen selected core groups pass over public invites and locally | Full campaign coverage; provider rate limits remain an external service limitation |

## Maze core integration, 6 November 2026

Source inspection found three implementation gaps: Maze machinery inherited an
empty co-op update, the Arena did not supply native NPC contacts to movers, and
the escort child only sampled the first living player. Frozen `c6f580e` fixes
these together. Either living player can press the plate and trigger the escort.
The child follows the closest visible living Alice, retaining only that actor's
observed path; changing targets clears the previous path. The selected slot is
saved and replicated, with an optional field for older saves.

The plate/gates advance once per shared tick. Both living riders move only after
all destinations are clear; a closing gate waits for either player. Dead bodies
neither press nor ride the plate. The child's own Hold still controls departure,
so two Alices cannot bypass the escort. A pending failed exit retries once from
the shared update without requiring an E press. Solo play uses the same mover
implementation with one body.

All twelve supplied-map component cases and seven production-cast cases pass:
both player slots at 30/60/144 Hz, solo clock parity, pause, snapshot continuation,
blocked/released carry, dead-body exclusion, both gate blockers, actual Arena
NPC contacts before/after restore, both native escort triggers and one-shot
automatic exit retry. Two new asset-free escort regressions also pass. Broader
shared-world and all 39 entrance/four underwater checks pass on this executable.

The full ordinary solo route reaches tower1 in 28,351 simulation ticks at
95.2152 health and 99.2 Will. All eleven JSON checkpoints/reports, including the
destination arrival, exactly match a fresh `66ebd44` baseline; the route logs
also have identical hashes. All 940 active Rust tests pass (five ignored), the
locked release builds, and 450 Python guards/seven package tests pass. The first
unit run retained a registration-list failure and a transient socket-cancel
timeout; after fixing the registration list, the unchanged network test passes
alone and in the final full suite. No timing assertion was relaxed.

Evidence: `private/coop-maze-build.json`, `private/coop-maze-components.log`,
`private/coop-maze-native-c6f580e/`, `private/coop-maze-solo-before/` and
`private/coop-maze-solo-comparison.json`. Anode is released and owned windows
are closed. No screenshots were inspected. This adds **0.25 progression points**
for a working core adapter, bringing the estimate to **56.8%**. Maze becomes
Partial; there is no new complete campaign visit. Its full two-client route,
combat/pickups and death/wipe/rejoin matrix remain unverified. The next priority
is the [remaining controller integration](COOP_CORE_TESTS.md#remaining-controller-integration).

## Core internet consolidation completed, 5 November 2026

After nearly two hours without a provisioning request, public hosting became
available again. All thirteen focused core groups now have passing internet
evidence on the byte-identical `0b5bca3` executable. The initial cohort
`private/coop/core-suite-2f95fa5c8bb94289ab303a2848908566/summary.json` passed
ten groups and failed the swimming fixture's final team-wipe setup. Its failure
is retained. The later cohort
`private/coop/core-suite-b09c7ef47ce043a1a6a6b603aef07bc2/summary.json` passes
swimming, rope and tilting-platform recovery. All recorded phase/result hashes,
network scopes and native identities were checked when combining these results.

The passing coverage includes solo import, primary/backup resume, host loss,
normal menu recovery, locked-file failures, save refusal during deaths/scenes,
both lift boarding orders, loading transitions, underwater/rope/platform
countdown rejoins, exact revivals and shared checkpoint restoration. Both
clients use separate processes and profiles; diagnostic cases use 40/20 ms
delay/jitter, while storage and host-loss menus use ordinary connections.

The swimming failure was a test setup issue: stopping both players immediately
after individual revivals left their air supplies far apart. They alternated
deaths and revivals without ever being dead together. A later attempt to refill
after those revivals also failed when a player drowned before reaching the
guide. The fixture now checks fresh air from ordinary gameplay and performs
the shared wipe first, followed by entrance rejoin and both individual recovery
roles. No health, air, position, death rule or native code was changed.
The countdown verifier now accepts an explicit expected checkpoint generation,
preserving its default zero and rejecting unexpected generation changes.
The intermediate hard-coded-zero assertion failure remains recorded too.

The final fixture passes locally in 121.19 seconds and over public invites in
141.91 seconds. The two remaining public groups pass in 51.34 seconds (rope)
and 83.94 seconds (tilting platform). All 427 Python guards and seven package
tests pass. The earlier 934-test Rust result still applies to the unchanged
native build. No screenshots were inspected. Anode was released after cleanup.

This completes the focused core milestone, not the full game. Campaign status
remains six combined, twenty-one partial and twelve not run. The known Red-led
Temple route gap, broader effects/audio and per-visit acceptance cases remain.
The weighted estimate rises one Internet Host/Join point to **56.6%**.

## Live two-client audio, 6 November 2026

The remaining focused audible co-op check now passes on frozen `66ebd44`.
Two public-invite puzzle runs cover Blue-first and Red-first demonstrations,
both reset-lever operators and a fresh guest rejoin in 71.97/70.95 seconds.
Each client receives the same three notes once per sequence; the fresh guest
does not replay the completed demonstration. A separate 83.47-second public
opening-scene run watches all six dialogue lines once, in order, on both clients.
All three cases use independent processes/profiles and 40/20 ms delay/jitter.

These runs use the PC's output endpoint because Anode has no audio endpoint.
All eight app-only recordings contain real 44.1 kHz stereo mixer output.
Private cross-correlation against the locally supplied original sounds finds
the three ordered note onsets in both initial-client captures and in both
fresh-guest captures (scores 0.769-0.833), and the first Rabbit voice on both
clients (0.964/0.975). Capture length is capped at 30 seconds: later reset
sequences and the other five voice lines are runtime cue/clock evidence, not
full acoustic comparisons. No microphone/system audio or model image lookups
are used. This is bounded output verification, not subjective sound fidelity.

Two earlier attempts remain failed evidence. The original audio fixture sent
both bodies to one point and stalled against correct player separation. The
first separated-goal revision disabled defensive attacks and a player died.
The final fixture retains ordinary combat and waits for both distinct, safe
arrivals before using the original lever. No native physics, health or enemy
rules changed. New guards reject unsafe/missing arrivals and partner overlap.
The voice verifier now requires all six opening lines rather than a three-line
minimum; all 308 saved observations per client pass the stricter verifier.

Evidence: `private/coop-live-audio-combat-manifest.json` and
`private/coop-live-audio-verified.json`. All 33 recorded artifact hashes, frozen
fixture hashes and executable identity were independently rechecked. The final
validators are recorded separately from the unmodified live receipts. All 450
Python guards and seven package tests pass; the prior 938 active Rust tests and
locked release result apply to the unchanged native source. Owned test windows
are closed. Both save families remain covered by the separate thirteen-group
`0b5bca3` milestone. Overall remains **56.6%**, with no campaign visit credited.

## Spectator audio and weapon effects, 6 November 2026

On frozen build `66ebd44`, co-op audio now uses the same player as the camera
for health reactions, swimming ambience and the underwater filter. Previously,
a dead player's submerged body could keep the dry survivor's view muffled.
Changing the subject clears only personal reaction/swim state, so switching
between different health values cannot invent a pain cry. Shared world sounds
and dialogue continue. Native tests cover either player dying, both dry/wet
survivor combinations and return to the revived player.

The public-invite recovery fixture passes in 140.86 seconds with 40/20 ms
delay/jitter. It exercises an ordinary shared wipe, both real drowning roles,
both fresh-guest countdown rejoins and exact ten-second revivals. The listener
matches the camera in 550 host and 545 guest observations, including 51/48
spectator observations. The guest runs in four fresh processes. An independent
audit also checks each recorded first rejoined frame. The final stricter verifier
requires revival after the observed death in the same checkpoint generation;
the saved observations pass with 3/191 post-revival rows. Earlier shared-wipe
spawns cannot substitute for that coverage.

The live fixture's two swimmers remain submerged, so the mixed dry/wet cases
are native-test evidence. Anode has no isolated audio endpoint: this run checks
the actual mixer inputs and sets `speaker_output_verified: false`; it does not
claim a new audible-output or sound-fidelity pass.

Separate audio-only regressions then pass on the PC's output device, using the
same frozen executable and isolated files, without opening game windows. They
measure fourteen gameplay intervals and seven liquid/reaction intervals in
22.73/18.66 seconds. App-only 44.1 kHz stereo captures contain 22.35/18.25 seconds
of output. Mute and pause are silent; expected audible phases have nonzero
output without clipping, and no requested audio files are missing. These are
production shared-mixer regressions, not an audible two-client co-op run or a
subjective sound-quality assessment. Evidence is
`private/coop-listener-audio-output-66ebd44/`.

On the same executable, all six native effect pixel comparisons pass: sparks,
swing trails and projectile trails for both skins remain visible and match
across three repeated restores each. A separate 26.34-second public combat
check receives all three effect types and both emitting players on both clients,
with 63 matching player/world/native states. These automated checks inspect
rendered pixels and network state without model image lookups.

Evidence: `private/coop/audio-listener-3a0fff842e8146808293fb8b7562b522/`,
`private/coop/weapon-effects-261091c7703548b98832633b1a17998c/` and
`private/coop-listener-verified.json`. Original live receipts and fixture copies
remain unchanged; the audit records the tightened validator's hash separately.
All 938 active Rust tests pass (five ignored), the locked release builds, 443
Python guards and seven package tests pass. Owned windows are closed and Anode
is released. The thirteen-group save/rejoin milestone remains on its separate
`0b5bca3` build. Overall remains **56.6%**; no full campaign visit is credited.

## Save coverage audit and bounded ending evidence, 6 November 2026

The thirteen public core groups and all nineteen phase-result hashes remain
intact on frozen `0b5bca3`, including both solo-to-co-op import and previous
co-op resume/rejoin. The frozen executable hash also matches. No core test was
rerun or additional readiness point credited by this evidence audit.

The already-running Curiosity ending attempt on `ddaf73b` completes Blue's
watched Caterpillar scene, safe reunion and paired points 77-82. Its independent
bounded audit verifies twelve personal arrivals and 5,219 matching
player/world/native states. The complete attempt fails at point 83 when Blue
does not settle on an intermediate ledge within the navigation timeout.
Watch activation, the later personal Meta-essence, final climb and exit remain
unverified by this live fixture. The earlier first attempt also remains failed;
its boundary assertion incorrectly rejected normal post-scene regeneration and
loot aging. The corrected verifier permits only changes explained by the
authoritative reunion tick and normal fixed-step rules.

The new ending fixture and opt-in ordinary Watch input have 936 passing native
tests (five ignored), a locked release build, 439 Python guards and seven
package tests. Four native comparison logs preserve the existing baseline,
including the known cast-free point-54 failure. No screenshots were inspected;
owned game windows are closed and Anode is released. Full campaign status and
the **56.6%** estimate are unchanged. See [ending evidence](JLAIR1.md#resumed-ending-fixture-6-november-2026).

## Remote approach directions, 6 November 2026

The opt-in remote route search now retains the direction used to arrive at a
search cell. Its delayed-turn safety check continues that preceding input;
merging arrivals from different directions discarded potentially safe paths.
The native reproduction failed after 100,002 states from the settled guest
position, only 1.56 units from an earlier successful start. Keeping the incoming
direction finds a route from both positions and the exact later live failure.
The search limit, twelve-tick takeoff reserve, eight-unit first-landing support
and normal player physics remain unchanged. Zero-margin native routes retain
their previous state equivalence.

The optional native `remote_curiosity_ascent_retains_approach_directions` test
replays each selected input through actual movement/collision physics at all
three recorded starts. It requires actual jumps, clear bodies, dry travel,
grounded arrivals within 22 units, supported first contacts, and the full late
input interval at grounded turns/takeoffs. The test passes explicitly on frozen source `c893e48`; all
935 active Rust tests pass (five ignored), the locked release builds, 427 Python
guards and seven package tests pass. All four Curiosity native comparison logs
match their baselines, including the known cast-free point-54 failure. That
failed comparison remains a failure, not a completed traversal.

The paired fixture retains all four original ascent guides and adds an observed
higher ledge before the unchanged original point 65. On the earlier `1a6b3ea`
runtime, both players reached all five guides and Blue reached point 65, but
Red's final search failed. Its bounded independent proof verifies ten guide
arrivals, 1,845 matching player/world/native states and 108 recovery rows without
revivals. That attempt remains failed and adds no paired point or visit credit.

The current `c893e48` public-invite run completes the resumed **65-76** segment,
with both players reaching every original goal: 24 personal arrivals and all
ten guidance arrivals. Independent processes/profiles use 40 ms delay and
20 ms jitter. The frozen-fixture audit rechecks exact checkpoint restoration,
ordered arrivals, observed waiting bays, player/camera presence, saved checkpoint
hashes, 7,419 matching player/world/native-enemy states and 431 recovery rows.
Both finish alive at 92/77.67 health without revival. Ordinary checkpoints
before 65, 66, 70, 74, 76 and 77 are retained.

Evidence is `private/coop/jlair1-route-a6b6947a160e4842bf28b4bcfd91fd74/`, including
`verified-direction-segment.json`. The formerly stalled paired point 65 is now
verified. This is one completed segment, explicitly `full_route: false`; a fresh
continuous repeat, the ending, both scene owners, recovery/rejoin combinations
and complete campaign consolidation remain open. No state or checkpoint bytes
were edited to progress. No screenshots were inspected; both test clients
exited normally, owned windows are closed, and Anode is released.
Overall remains 56.6%, with six combined, 21 partial and 12 unrun visits.

## Automated takeoff timing, 6 November 2026

The opt-in route controller now consumes elapsed guest frame time when a
movement step expires. Its old 100 ms cap could hold a run-up too long while
the host continued simulating, issuing the next jump after the player had
already left the ledge. The new clock advances once per sampled frame and
gives the next input its full interval. A native Curiosity regression models
a 300 ms guest stall: the old clock jumps too late and records zero actual
jumps; the corrected clock records one grounded takeoff. This reproduces a
failure mode, not the exact timing of the earlier live capture.

Frozen source `1a6b3ea` passes 935 active Rust tests (four ignored), the explicit
asset-dependent takeoff regression, locked release, 427 Python guards and
seven package checks. Its Curiosity native comparison logs match their prior
baselines, including the known cast-free point-54 failure. Only the automated
route driver changes; normal player input, physics, networking and saves do not.

The original four-guide public fixture now records both lower approaches and
all eight personal guide arrivals, plus Blue's original upper-shelf arrival.
A bounded audit verifies 1,709 matching player/world/native states and no
revivals through the last guide receipt. The complete run still fails: Red's
final ascent exhausts 100,001 search states. A separate final-guide adjustment
also fails after the eight guide receipts (992 matching states in its bounded
audit); that adjustment has been reverted. Both failed runs remain recorded.

The focused save/rejoin milestone remains complete on `0b5bca3`; these campaign
tests do not replace or expand that build-specific evidence. No paired point-65
or completed campaign visit is credited. Overall stays **56.6%**. See the
[Curiosity evidence](JLAIR1.md#automated-takeoff-clock-6-november-2026).

## Campaign work resumed, 5 November 2026

With the focused core gate complete, Burning Curiosity testing resumes on
unchanged build `0b5bca3`. Its 26 paired native cases plus summary, seven solo
components and full native combat-route output match the earlier baselines
byte for byte. The separate cast-free route retains its known step-54 failure.
A first long native comparison ended with Windows `AppHangB1`; the isolated
Anode repeat completed and matched. The failed attempt remains recorded.

The resumed internet climb still fails at point 65. Blue reaches all four
guidance points and the upper shelf; Red reaches the first guide, then misses
the next landing and falls. A private crossing prototype verifies Blue's real
run-up, jump and landing, but the guest disconnects before attempting it. The
transport log does not establish the disconnection's cause. A delayed local
attempt then fails on Red's earlier approach: the host records the jump input
only after she is already airborne below the ledge, so no actual jump occurs.
Neither prototype is promoted to the checked-in route fixture or counted as a
paired climb pass. See `JLAIR1.md` for evidence and the next controller issue.

No game code, player physics or save behavior changed in this investigation.
The previously deferred supported-landing fix remains in the current build,
but these failures happen before its target ascent can be verified for Red.
Campaign counts and the **56.6%** overall estimate remain unchanged.

## Archived solo save import, 5 November 2026

The original Village solo autosave from the 9 October movement-fix work now
passes the full import suite on unchanged build `0b5bca3`. Before its first
guest connects, the runner compares Blue's complete body, resources, camera,
character, world interactions, story, hints and NPC state with the archived
file. No differences remain after accounting for equivalent JSON spellings
of native 32-bit floats. Red receives the same earned starting resources at a
separate safe position. The source file remains unchanged.

All four connected phases pass: import, fresh-process resume, resume after
abrupt host loss, and backup recovery. Each also restarts and rejoins the guest.
The final checked-in runner result is
`private/coop/core-suite-1902de13b88844d696b763337e1c3da2/summary.json`, case
`import-test-beaa7e9d6c5c48a3b24c6f5590fe02f9`, taking 33.44 seconds. This is
loopback evidence with 40/20 ms delay/jitter, not a new public-invite pass or
proof of every older save format or campaign visit.

The initial probe exposed a test teardown race: sending Finish to both clients
together could close the host before the guest read its own command, yielding
a normal host-ended error. The import runner now waits for a successful guest
exit before finishing the host, retaining failure checks on both processes.
The default generated school/swimming preparation and all four network phases
also pass with this ordering: `core-suite-e31f227be06442acaa20edd48fa9e279`,
case `import-test-7516ff8eb9aa42c7aab58aa12dbf4615`, taking 34.95 seconds.
The failed probe is retained separately and receives no passing-suite credit.

`--source-save` makes this reproducible with an isolated copy of an existing
quiet save; the core runner records and checks its hash. Five new guards reject
partial resets, stale observations, lost fields and altered values, and cover
the shutdown order. All 424 Python guards and seven source-package tests pass.
No native gameplay code changed. Overall completion remains **55.6%**.

## Abrupt host loss through the menus, 5 November 2026

`tools/test_coop_menu_host_loss.py` closes a gap between diagnostic host-loss
coverage and the normal player-facing menus. It passes on unchanged build
`0b5bca3` through the checked-in core runner, using two rendered processes,
native input and passive state receipts. This run is **loopback only**.

After loading independent solo saves, the host imports its game and writes a
primary checkpoint and backup through the menu. Terminating only that owned
host process tree produces the guest's visible disconnect explanation in
0.969 seconds. The surviving guest returns to its exact suspended solo state,
accepts ordinary movement, then enters Join again through ESC. A fresh host
process uses Continue co-op and creates a different invite; the same guest
process rejoins with saved resources, positions, used world interactions and
cached visit count preserved. Both players then return to their own solo
states. The two solo files, primary checkpoint and backup retain their bytes.

Evidence: `private/coop/core-suite-06283cf9f6d948608ab058b5323ba07d/summary.json`
and `private/coop/menu-test-b045e7cc7f74431eb05c4addb2f000de/`. The full check
took 29.75 seconds and inspected no screenshots. It is now selectable as
`host-loss-menu` in the thirteen-case core runner. Its public-invite repeat
remains open alongside the previous twelve core cases. No native files changed,
no campaign visit was added, and overall completion remains **55.6%**.

## Authenticated invite readiness, 5 November 2026

Build `0b5bca3` checks its prospective public invite before exposing it. After
helper registration and the existing startup grace, a background worker uses
the same DNS/TCP/TLS and authenticated endpoint as Join. Only the expected reply
to a fresh random challenge succeeds. The host remains waiting with no guest
slot consumed, generation change or gameplay started. The menu shows
"Checking your invite..." while this runs. Transient failures retry within a
60-second budget; timeout/helper failure returns an error, and Cancel closes
an in-progress handshake socket and releases the owned helper.

All **934 active Rust tests** pass (three ignored); the locked release, 410
Python guards and seven source-package tests pass. Added coverage uses real
loopback sockets for slot preservation, invalid authentication/protocol/path,
malformed and replayed challenges, ordinary upgrades, transient failure,
deadline/helper failure and cancellation during HTTP/TLS setup. The initial
malformed-path test accidentally used an HTTP rather than WebSocket scheme;
that fixture was corrected. A subsequent full run exposed a cancellation test
whose socket deadline began before Cancel was dispatched. It now synchronizes
that action, checks Cancel's own duration and separately requires prompt EOF.
The final full suite passes; earlier failed logs remain in private evidence.

Public tests on the byte-identical executable pass:

- `network-test-266666220b5748b7a853cdcf92648c5e`: independent processes, isolated
  profiles, 16 ordered exchanges, 40/20 ms delay/jitter and clean disconnect.
- `menu-test-e38c28f83d344935b8a0509c58743d5d`: real main/ESC solo import,
  save/leave, expired-link feedback, Continue co-op with a fresh invite and two
  exact solo returns (66.27 s).
- `menu-test-7e9a518372294142abc5c4995a780efe`: DNS/HTTP errors, cancelled Join,
  failed/cancelled Host, successful public retry in the same processes, save,
  leave and exact solo return (63.80 s).
- `host-stall-test-49c9bc9570904e1a8a0f1e82b2b0954d`: three controlled stalls,
  recovered input timing, contact separation, both models and ambient effects;
  680 matching states, zero settled input backlog and under 7 ms simulation
  debt with 40/20 ms delay/jitter (65.81 s).

All case folders are under `private/coop/`. The retained manifest is
`private/coop/invite-readiness-0b5bca3-20261105/manifest.json`. The executable SHA
is `32f9559ff0c7c3d38722eda021cfdddf81fb366298e54db43f0eb42f33b2d2df`.
The network cohort `regression-suite-643f10bb62464b169308cecb98e2f48c` passes
8/9 selected cases: host stalls, both ground countdown reconnects, all four
scene-owner/introduction-or-dialogue reconnects, and outbound loading-barrier
rejoin. The return-barrier case fails before gameplay in 1.25 s because the
invite provider limits new sessions; no new link is published. Its retained
folder is `travel-test-511a4bf7b477499da9779c898278a95b`. This is a failed public
suite, not a completed core consolidation. Remaining checks completed locally
without retrying the rate-limited provider; local results carry no internet
credit and older save/recovery passes keep their original build identities.

This adds 0.25 Internet Host/Join points for verified positive route readiness:
**55.58 total, reported as 55.6%**. Campaign counts remain 6 combined / 21 partial /
12 not run. The check proves reachability from the host at setup time; it cannot
guarantee another computer's DNS cache or continued provider availability.
[Cloudflare Quick Tunnels](https://developers.cloudflare.com/tunnel/get-started/quick-tunnels/)
remain a development service without an uptime guarantee.

## Reproducible core runner, 5 November 2026

The twelve save/recovery groups are now available through the checked-in
`tools/test_coop_core.py`; see [core test instructions](COOP_CORE_TESTS.md).
It preflights the desktop and required focus lease before starting games,
validates every expected phase receipt, rejects mismatched network scope,
reused evidence and mixed builds, and detects changed source/input/executable
files. It stops on the first failure without automatically retrying a provider
error. Optional saved SchoolPair modules preserve unrelated worktree edits.

Nine focused runner guards pass, bringing the Python guard total to **419**;
seven package checks pass. The final collector also validates all twelve
retained local cases plus one fresh local import case. That audit is receipt
validation, not twelve additional gameplay passes. The fresh end-to-end run is
`private/coop/core-suite-b159c71b2d3044ac859ef30c51ef1611/summary.json`, with
`import-test-f6eb3d468ec945f083115f1ac85fe962`: all four connected save phases,
guest rejoins, abrupt host loss and unchanged solo files pass in 34.53 seconds.
The later malformed-metadata checks were validated against its retained results;
the report keeps the runner hash actually used during execution.

One bounded public return-barrier retry after approximately twenty minutes
without new provisioning requests still fails before gameplay in 1.38 seconds:
`ready-core-suite-83fa108fb74d41cf8aaac3f9500a3c86/summary.json`, evidence
`travel-test-c3b2192d4e144945a2869077d0791ec5`. The service again reports session
rate limiting; the retained helper log gives no retry time. No further public
request was made in this turn. Earlier public passes remain valid, and the
remaining twelve public groups stay open. All native code and frozen executable
bytes remain `0b5bca3`; no game rebuild was needed. Anode is released and overall
completion stays **55.6%**.

## Frozen-build local core completion, 5 November 2026

The same `0b5bca3` executable completes all **12 selected local core cases** with
40/20 ms delay/jitter where supported. These use separate client processes and
profiles. They supplement the public checks above; they do not count as public
internet passes or full campaign verification.

| Local group | Verified result |
| --- | --- |
| Return loading barrier | Guest restart while returning to a cached visit; preserved history/resources and shared transition |
| Solo import and co-op resume | Four fresh connected phases: import, resume, resume after abrupt host loss and backup; guest rejoin in each; exact native restore and unchanged solo files |
| Persistent shared checkpoint | Cached travel/rejoin, fresh-process primary and backup restoration, exact team-wipe recovery and personal resources |
| Storage errors | Locked primary/backup save failures, successful retry, fresh-process backup continuation and repair, both-corrupt feedback and normal solo return |
| Save eligibility (three cases) | Real-menu refusal during either death countdown and both theatre owners' introduction/dialogue, with successful safe retries |
| Rotating lift (two cases) | Both boarding orders, guest restart during motion, exact held state, preserved rider attachments and completed ride |
| Swimming recovery | Both underwater countdown reconnects and exact revivals, moving spectator and saved team wipe in the Blue-led route |
| Rope and moving-platform recovery (two cases) | Both countdown reconnects and exact safe revivals; moving-platform passive carry after revival |

Private batch reports:
`ready-core-suite-bfd11efc07d54ebd810277b719b6df65/summary.json` retains the first
four passes and a failed menu-test launch: the caller omitted its active Anode
focus lease, so the fixture stopped before its first Save action. The caller
now checks and forwards that lease. `ready-core-suite-da1d0126c45d4404b8fe3904e879b9f9/summary.json`
completes only the eight remaining cases. Both retain their actual wrapper and
fixture hashes. The aggregate readiness manifest records all twelve case
folders and result hashes; no successful case was repeated to hide a failure.

The swim case covers both victims in the Blue-led recovery route. It does not
close the separately recorded Red-led trap-navigation repeatability gap.
Rope and fulcrum fixtures resume unchanged ordinary gameplay checkpoints;
these are representative recovery checks, not entrance-to-exit traversals.
The travel fixture now prints its evidence folder before launching Host so
future provisioning failures are discoverable automatically. This logging-only
change follows these runs; their original fixture identities are retained.

The next public work is exactly these twelve local cases on this frozen build.
The eight public network passes and both public menu passes need no duplicate
run absent another relevant change. Public rate limiting remains unresolved;
all owned windows are closed and Anode is released. Overall completion stays
**55.6%** and the full campaign goal remains active.

## Fixed-build core checkpoint, 5 November 2026

The consolidated save repeat passes on the byte-identical `c5cb7fc` executable,
through public invites with 40/20 ms delay/jitter. This explicitly covers both
user-requested starting points: a loaded solo save converted to co-op, and an
earlier co-op session resumed later.

- `import-test-25f44f88ee764fac9c4716bd7e652a3f`: import, resume, abrupt host loss,
  fresh resume after loss and corrupt-primary backup recovery. Every one of the
  four connected phases includes a fresh guest rejoin. Native pre-simulation
  restoration, individual inventory/health/Will, cached visit state and unchanged
  solo files pass. The host-loss guest detects disconnection in 6.08 seconds.
- `travel-test-8a21edb2af184e1d879ed389766f6fc9`: shared travel/cache/rejoin, primary
  save restoration after both processes exit, and backup restoration. Both
  resumed cases also pass exact shared team-wipe recovery, individual resources
  and solo-save isolation. The combined save job exits zero in 233.89 seconds.
- `menu-test-0adb79d7780c41d2a1fb87bf4c1a426c`: real public main/ESC Host/Join,
  manual checkpoint, leave, expired-link feedback, Continue co-op with a new
  invite, normal solo movement and both exact suspended-solo returns (70.77 s).
- `menu-test-2924520df8824e9299d1dc5dcf8273b1`: real DNS/HTTP failures, Join and
  Host cancellation, failed hosting and subsequent public success in the same
  running processes, followed by save/leave and exact solo return (64.11 s).

All folders above are under `private/coop/`. The consolidated manifest is
`private/coop/core-consolidation-c5cb7fc-20261105/manifest.json`. It retains hashes,
receipts and failed attempts. The first menu repeat, `menu-test-91b25e76465a429b866877a4973919cf`,
passed its first co-op save/leave but opened Co-op when the test intended Resume.
It remains failed evidence. The harness now focuses before reading button
coordinates, checks the actual pointer and owned window before clicking, and
records every click. The subsequent two public menu cases pass with these checks.
The isolated window probe reproduced a successful Resume; it does not establish
the exact cause of the earlier mistargeted input. No game behavior changed.

Existing `c5cb7fc` storage-error, menu save-eligibility, underwater, rope and
fulcrum passes retain their own fixture identities; they need no duplicate run
just to share a report. Their build/source identities were audited at consolidation
start. The later menu input checks refine automation without changing save or
recovery assertions.

The network group `regression-suite-a0a95a8d4a564e028bbabd17f3ae1959` is **failed**,
not a completed core pass. Its first case, `presence-test-4233e0afbace44678f2fb6482c989e1a`,
passes contact separation, stationary-partner stability, both visible Alice
models and ambient particle/effect rendering on both clients (55.91 s). The next
case, `host-stall-test-e61fa43063ac4a15a80af365fcbd6187`, fails after 45.39 s because
the guest cannot resolve the new public invite address. It never reaches gameplay
or suspends the host. The remaining eight selected cases were not run, and both
lift orders remain separate work. The provider failure has not been rerun away.

Hosting currently waits for tunnel registration plus a 12-second grace period;
Join has bounded DNS retries. Host setup does not yet positively verify its
public address resolves before publishing the invite. That is the next concrete
readiness investigation; extending a fixed delay alone would not establish
service availability. All owned clients are closed and Anode is released.
This checkpoint adds no campaign visit or percentage point: overall completion
stays **55.3%**, with core consolidation incomplete.

## Moving-platform countdown recovery, 5 November 2026

`tools/test_coop_fulcrum_countdown.py` resumes an unchanged normal Funhouse save
with both players on the second fulcrum. Each victim runs off the side through
ordinary input. The survivor walks along the board to change its load. A fresh
guest rejoins during each countdown, then each revived player stays neutral
while the moving brush carries it. No native rules, bodies, health, enemy state
or saved payloads were changed. The native executable remains `c5cb7fc` with SHA
`07885bd0d8eb477aefe3d751d25f8e9fe0cf9f5f8819a337ba78f3ad87627f97`.

The final public case uses 40 ms delay and 20 ms jitter and passes in 60.03 seconds:
`private/coop/regression-suite-ac593d2aed8b4a609042bc88e74a4267/summary.json`;
case `private/coop/fulcrum-countdown-388308598d6f479083fea2574959ea6b/`.
All six retained source hashes match the final fixture. The unchanged checkpoint
SHA is `a61e0db3cbabc7347a0b3dd2fdd5826d29fbeffaf3de764b75b74f4cb8a628bd`.

| Public victim | Death | Revival | Rejoin ticks | Remaining countdown | Passive carry |
| --- | ---: | ---: | --- | --- | --- |
| Red | 88029 | 89229 | 88235 to 88247 | 994 to 982 | 17.72 units over 131 ticks; 7.11 degrees |
| Blue | 89899 | 91099 | 90262 to 90263 | 837 to 836 | 33.92 units over 120 ticks; 13.95 degrees |

The closed host trace has 2,415 records, both exact 1,200-tick/25-health revivals,
no unsafe-spawn delay, preserved personal resources and clear, safe, valid
supported placement. Observations bracket each spawn while the brush is rotating
and identify both bodies on that same native support. Both cameras, separation,
neutral-body carry, held disconnected world and exact first-frame agreement pass,
with 426 matching player/world/native-enemy states.

The initial local case `fulcrum-countdown-17dd7c4eb1c2415bb0e5ec478c4ca1b2`
passes in 40.74 seconds. The first public attempt
`fulcrum-countdown-4bd8c53398ab4419bb9e9e9436f03318` fails: unpredicted guest
steering overshoots a goal, causing an extra death. That failed evidence remains.
The final fixture uses the same measured-RTT steering lead as the existing
Funhouse route and fails immediately if its intended survivor dies. The public
pass follows that concrete fixture correction; no gameplay behavior was weakened.
The earlier spiral-lift probe stopped because a living player on the landing
correctly prevents shared departure; it supplies no moving-support recovery proof.

Four new guard tests reject stationary/wrong supports, driven or unsupported
carry, stale reconnects and advancing held worlds. All 410 Python guards and
seven package tests pass; the source worktree audit passes. No screenshots were
inspected. All owned clients close and Anode is released after verification.
This adds 0.25 combat/recovery and 0.25 transport point, giving **55.3%** overall.
The campaign matrix stays at 6 combined, 21 partial and 12 not run. The remaining
core milestone is the consolidated fixed-build run, beginning with solo import,
existing co-op saves, fresh restarts, backup recovery and rejoining.

## Rope countdown reconnect and revival, 5 November 2026

`tools/test_coop_rope_countdown.py` passes on unchanged build `c5cb7fc`, locally
and through a public invite with 40 ms delay and 20 ms jitter. It resumes the
unchanged Garden capture with Red falling and Blue attached to rope 353. After
Red returns, it exercises Blue's death while Red remains on the rope. The
survivor uses ordinary climb/weapon controls against the still-active rifle ant.
Only the owned guest process is terminated and restarted; bodies, health,
resources and enemies remain under normal simulation.

Both countdown reconnects preserve the held world, dead body, survivor's rope
ID/grip, personal resources and remaining time. The guest's first playing frame
matches the exact host tick in all three hash domains. A closed host trace then
proves both exact spawn boundaries: 1,200 ticks after death, 25 health, clear and
hazard-safe valid rope placement, approximately 72 units from the living partner,
with carried resources intact. Both rendered views independently verify the
spectator camera, its return and current non-overlap while the partner climbs.

| Public victim | Death tick | Revival tick | Rejoin ticks | Remaining countdown |
| --- | ---: | ---: | --- | --- |
| Red | 16926 | 18126 | 17471 to 17473 | 655 to 653 |
| Blue | 18174 | 19374 | 18400 to 18401 | 974 to 973 |

Public suite: `private/coop/regression-suite-fa1a0eba9c264a1ba59381572a7cc2cf/summary.json`
(48.00 seconds). Its passing case is
`private/coop/rope-countdown-0cc60c5a60034bed9857e1031b1fdded/`, with 2,403 exact
host trace rows and 290 matching player/world/native-enemy states. All seven
retained fixture source hashes match the current files. The checkpoint hash is
`1782b54ba1795b9f66119aa301d5299a84f86e1621c4db9b6299bf503540f52d`.
The local pass is `private/coop/rope-countdown-42c6a20ba71f4cf3899526ac96d621a0/`
(28.64 seconds, 363 matching states, both exact revivals). The final public
fixture reads its trace after closing the host rather than while it is writing.

Failed local attempts remain retained. `3c0acb83546947d6ac322dc59611f700` and
`beafec60ec3646d9a8b6ca51a5a72bb9` lose the low-health survivor to rifle fire;
`2bdb622727ac4be9927a962f44bf03aa` reaches a spawn wait after the survivor loses
her grip. Ordinary return fire alone did not solve those exposed waits. The
fixture now alternates ordinary climbing during Blue's remaining countdown.
`530f6fa043ae4584b6c244024261b028` observes both revivals but fails an old fixed
64-unit distance rule after the survivor has climbed closer. That rule remains
for the original neutral fixture; this moving case instead requires exact host
spawn geometry, current body separation and speed-bounded later observations.
No native game rule or enemy strength was changed to produce the pass.

The old Blue-victim internet attempt retained the same Red-falling checkpoint
as the old Red case and failed before gameplay; it supplies no Blue-role proof.
The new sequence supplies actual Blue death/revival evidence. No screenshots
were inspected. All owned clients close and Anode is released.

Four new guard tests reject changed grips/bodies, advancing held worlds and
stale or late guest evidence. All 406 Python guards and seven package tests pass.
The native executable is byte-identical to the previous 928-active-test/locked-
release build. This adds 0.25 combat/recovery point, giving 54.8% overall. It is a
captured recovery component, not a new completed campaign visit; the matrix
remains 6 combined, 21 partial and 12 not run.

## Underwater countdown reconnect, 5 November 2026

The new `tools/test_coop_utemple_countdown.py` uses ordinary swimming and actual
drowning, restarting only its owned guest process during each victim's countdown.
It adds both player-order cases to the regression runner and six rejection guards.
Native build `c5cb7fc` is unchanged; all evidence uses the same frozen executable.

The Blue-led route passes locally and through a public invite with 40 ms delay
and 20 ms jitter. Both Blue and Red are tested as the dead player. The waiting
host holds its entire player/life frame and world; the fresh guest inherits the
remaining timer, body identity, air, shells and personal resources. All three
first-frame hash domains match the host at the exact guest tick. Both players
revive after exactly 1,200 ticks with 25 health and fresh air in clear water near
the survivor. A subsequent team wipe restores the saved entrance for both, and
a further guest restart preserves that restored state.

| Public victim | Death tick | Revival tick | Rejoin ticks | Remaining countdown |
| --- | ---: | ---: | --- | --- |
| Red | 3474 | 4674 | 3712 to 3713 | 962 to 961 |
| Blue | 8039 | 9239 | 8571 to 8572 | 668 to 667 |

Public evidence: `private/coop/temple-countdown-9af51a1dd65f428da12c197dec1892c5/`
(150.25 seconds, 1,563 matching full-body/world/native-enemy states, shared wipe
at tick 13,320). Local evidence:
`private/coop/temple-countdown-63951454af72477bb751d1d0ea3f84ea/`
(107.38 seconds, 1,743 matches, shared wipe at tick 11,227). Both results retain
matching fixture sources and full before/held/after reconnect snapshots.

The two-order public suite is **failed**, not a complete pass:
`private/coop/regression-suite-9bab4afd60af4923ac35506c1e8e5e55/summary.json`.
Its Red-led attempt (`temple-countdown-c6aa18e081054012997d2b0d362a0634`) wipes at
tick 4,011 before the first reconnect. The surviving Red is near the first
column with full health/air just before the wipe. This remains part of the
already-open temple trap-repeatability work; no damage or collision rule was
weakened, and the attempt is not counted as reconnect evidence.

An earlier local attempt (`temple-countdown-e5a6d5ba396948f9a4798dc36f78665f`)
passed its first reconnect, then failed a fixed 32-unit camera/network-distance
assertion. The recovery fixture now retains coherent presentation samples and
uses the existing verifier for exact interpolation inputs and the actual drawn
camera subject. It also checks fresh air at revival. The failed attempt is
retained; no native camera behavior changed. No screenshots were inspected.

The 0.25 internet point credits the new underwater reconnect coverage for both
victims, not another campaign visit or a reliable Red-led temple route. Current
checks: 402 Python guards, seven package tests and source audits pass; the locked
928-active-test native suite and release build pass. Campaign status remains
6 combined, 21 partial and 12 not run. Overall completion is 54.6%.

## Saving during deaths and scenes, 5 November 2026

`tools/test_coop_menu_save_rules.py` verifies the real ESC/Save checkpoint UI on
unchanged native build `c5cb7fc`. Three public cases pass with 40 ms delay and
20 ms jitter per sender, sharing the exact final fixture identity. The fixture
checks the public invite scope rather than inferring it from its arguments.

The death case uses normal guard damage in the first school entrance. Save is
refused during each player's countdown, with the displayed reason and both disk
files checked. The attempt neither resets nor skips the life timer. Both clients
then observe moving-partner spectating, exactly 1,200 active ticks to revival,
25 health and 48-unit separation. Save succeeds after each revival, with the
new file's tick matching the menu receipt. The pair agrees on 1,259 transmitted
game, world and enemy states. Public evidence:
`private/coop/menu-death-save-9361ba6310024cb4845451149caba201/result.json`
(137.89 seconds; Red dies first).

| Public victim | Death tick | Revival tick | Refused Save tick | Successful Save tick |
| --- | ---: | ---: | ---: | ---: |
| Red | 5932 | 7132 | 6208 | 7285 |
| Blue | 11857 | 13057 | 12173 | 13277 |

Each theatre owner is tested during the camera introduction and active dialogue.
The refused saves preserve the committed files. The original scene continues,
its waiting partner stays hidden/frozen, and both clients agree on safe reunion.
Save then succeeds with both reunited bodies and no remaining scene owner.

| Public theatre owner | Reunion tick | Successful Save tick | Evidence directory |
| --- | ---: | ---: | --- |
| Blue | 5848 | 5931 | `private/coop/menu-test-339f436e11c943919ab590ef1ab8eaa8/` |
| Red | 5873 | 5923 | `private/coop/menu-test-6bfdcc75f63a4d6ab2cb788cebb63c1a/` |

The Blue/Red public scene cases take 83.12/79.44 seconds. Local baselines also
pass: `private/coop/menu-death-save-d6972cc826a84e46bf7762b5564e1fbe/`
(both deaths, Blue first) and `private/coop/menu-test-040f6b8d0e8f444a879ddfefe63ba6cd/`
(Blue theatre). The final fixture adds explicit invite/scene-phase assertions;
all three public results independently match its current source hashes.

Earlier local attempts remain failed evidence. Two stopped at the background
job's denied window activation, one at a straight-line approach blocked by the
partner, one after the first safe retry because the survivor had not moved, and
one on the second approach across the revived body. The fixture now uses Anode's
owned-window focus action, separate approach lanes and a distinct survivor
waypoint. These corrections do not change collision, damage, revival or save
rules; the original recovery/camera assertions remain intact. No screenshots
were inspected, and unrelated seat windows were left open.

Current checks: 396 Python guards, seven package tests and source audits pass.
The frozen executable retains its earlier 928-active-test and locked-release
validation; no native source changed in this pass. The 0.25 foundation point
credits normal-menu save eligibility and retry coverage. Campaign status stays
6 combined, 21 partial and 12 not run; overall completion is 54.3%.

## Menu storage recovery, 5 November 2026

Build `c5cb7fc` passes the new `tools/test_coop_menu_storage.py` through native
menu input on loopback and public internet invites. The only native change is
an opt-in passive receipt of the message actually passed to the co-op panel.
No gameplay or checkpoint behavior was changed to make these cases pass.

The fixture loads an existing solo slot, hosts it, joins, then holds real Windows
read handles that deny replacement of its isolated backup and primary files in
turn. Each Save checkpoint action reports failure visibly, keeps the committed
primary intact, removes its temporary file and lets both clients keep advancing.
After the handles close, Save succeeds in the same processes.

Backup replacement happens before primary replacement. If the latter fails,
the backup can contain the still-valid previous primary; it need not retain its
older bytes. This test uses distinct saved states and checks that exact behavior.
The earlier native two-file equality assertion used identical initial snapshots;
it did not establish atomicity across two distinct files.

After both games exit, only the fixture's primary is damaged. Fresh host/guest
processes use Continue co-op and a new public invite, display the backup recovery
notice, retain the two players' resources and repair the primary on Save. The
valid backup remains intact. A third host start with both files damaged reports
the read failure, preserves those files, returns the suspended solo state exactly
and still accepts ordinary solo movement. All three solo returns have matching
full-state hashes; the original solo slot retains its bytes throughout.

- Loopback: `private/coop/menu-test-14610712fb634ea28ae54edc9dd84e1b/result.json`
  (37.28 seconds).
- Public invites: `private/coop/menu-test-f05962a090824d3e840dfdc375d75e67/result.json`
  (74.17 seconds). This menu test uses normal network conditions, without injected
  delay/jitter; it is separate from the earlier delayed reconnect matrices.

Both runs pass on their first attempt. No screenshots were inspected. Locked
release, 928 active Rust tests (three existing ignored), 396 Python guards, seven
source-package tests and source audits pass. The 0.25 foundation point credits
the normal-menu storage recovery evidence. No campaign visit is added; overall
completion is 54.1% and core work remains ahead of campaign traversal.

## Public countdown and cutscene reconnects, 5 November 2026

Six automated cases pass over public invites on unchanged native build `0c5e717`,
with 40 ms delay and 20 ms jitter per sender. The guest is a fresh process after
each interruption. Retained snapshots prove that the inactive host holds its full
frame and world; the replacement inherits the running timer or scene state.

| Death case | Death tick | Revival tick | Health / separation | Shared checkpoint tick |
| --- | ---: | ---: | --- | ---: |
| Blue dies; living Red reconnects | 2008 | 3208 | 25 / 48 units | 4201 |
| Red dies and reconnects | 2077 | 3277 | 25 / 48 units | 4261 |

Both clients agree on the exact 1,200-active-tick revival and later team wipe.
The dead player's camera follows the moving survivor and returns to its own body
after revival. Personal resources and body identities survive reconnection.
Suite: `private/coop/regression-suite-5a62b7313b014e2b99d617bce86e760f/summary.json`.

| Theatre owner / restart phase | Reconnect ticks | Authoritative reunion tick |
| --- | --- | ---: |
| Blue / introduction | 1017 to 1019 | 5848 |
| Blue / dialogue | 3891 to 3892 | 5846 |
| Red / introduction | 1036 to 1037 | 5870 |
| Red / dialogue | 3921 to 3923 | 5872 |

All four preserve the owner, scene cues, pending contacts, personal resources,
and hidden/frozen waiting partner. Introduction reconnects keep dialogue queued;
dialogue reconnects retain the current line and advance it only by active ticks.
Both clients watch the original scene to completion and agree on a 48-unit
reunion without respawn or checkpoint markers.
Suite: `private/coop/regression-suite-1f0f564f3329490c9bef83cc1b82e8ca/summary.json`.

The regression runner now exposes both death cases and all four scene cases with
compact timer/reunion metrics and build identity checks. Test fixtures retain
before/held/after snapshots, including failure evidence, without opening images.
All 928 active Rust tests, 396 Python guards, seven source-package tests and the
locked release build pass. No native gameplay change was needed in this pass.

The added 0.5 point credits public verification: 0.25 for death recovery and 0.25
for the scene matrix. These staged school scenarios do not complete a campaign
visit, every map's recovery behavior, skipped scenes or audible voice playback.

## Connection failure and cancellation recovery, 5 November 2026

The real-socket regression reproduced a Join cancellation bug: the UI returned
while the connection worker retained an unanswered handshake until its network
timeout. Build `0c5e717` registers the connected socket for cancellation and
shuts it down when the peer is stopped or dropped. The temporary duplicate is
released on both successful and failed handshakes. Cancellation tests cover
plain WebSocket and TLS setup; a separate HTTP rejection test retains its peer
object and still observes the failed socket close. DNS lookups and TCP connect
calls remain on background workers; cancellation cannot interrupt an OS lookup
already in progress, but a cancelled worker cannot enter gameplay afterward.

The automated real-menu run passed in
`private/coop/menu-test-d2faef5612b645839af27401e5876f24/`:

- An invalid hostname exhausts DNS retries and returns to Join with its link
  retained (23.39 seconds in this run).
- An owned HTTP 503 responder returns a useful error without exposing its body.
- Cancel closes the actual stalled handshake connection; the complete measured
  click/return check took 0.328 seconds.
- A temporary filesystem obstruction in the isolated host profile produces a
  genuine setup failure, returns to Host and leaves the solo save untouched.
- Cancelling another Host attempt stops its helper. Before/after process records
  show only the same pre-existing game console child, with no new helper left.
- The same host and guest processes then connect through a public invite, retain
  the loaded resources/world, save a checkpoint and leave. The manual co-op save
  and original solo slot remain unchanged; suspended solo state hashes match.

The first menu attempt passed the Join checks but stopped at an overly broad
cleanup assertion that rejected any child process, including the game's own
console. The revised check records the baseline and rejects new remaining
children; that complete rerun passes. Original evidence remains private.

All 928 active Rust tests, 396 Python guards, seven source-package tests and the
locked release build pass. The added 0.5 point credits the cancellation fix and
the verified failure-to-public-retry flow. This does not claim that the external
invite provider or every DNS resolver will always be available. No campaign
visit is newly completed and no screenshots were inspected.

## Loading-barrier reconnect evidence, 5 November 2026

Build `9921a3e` adds passive session-phase/pending-visit diagnostics and a native
socket regression; normal gameplay and loading rules are unchanged. The automated
travel runner suspends only its own guest before departure, observes the host
in Loading with the next visit pending, and verifies a frozen full frame/world.
It terminates that guest, waits for the host to load its pending destination and
hold there without advancing the simulation, then starts a fresh guest on the
same invite. Both continue through all three visit epochs and clean exits.

| Public-invite case | Held simulation tick | Fresh guest tick | Result |
| --- | ---: | ---: | --- |
| Outbound to the return-school entrance | 1440 | 1442 | Pass |
| Return to the cached first entrance | 2041 | 2043 | Pass |

Public suite: `private/coop/regression-suite-5618659c836c4efea039031063f49ea8/summary.json`.
Loopback suite: `private/coop/regression-suite-aa4d01e85f8b49d1b4d646295d0d4399/summary.json`.
Both use 40 ms delay and 20 ms jitter per sender. The checks retain health, Will,
earned weapon copies, pickup/drop history and paired relocation markers, with
exact cached world/NPC restoration on return. The destination's normal campaign
weapon grants are required exactly; arbitrary additions or resource resets fail.
The first local attempt stopped on an overly strict no-grants assertion. Its
evidence was retained and the oracle corrected before the complete passing runs.

Native tests interrupt before Enter reaches the guest, after Loading, after guest
Ready and after the host finishes loading. They verify that the guest slot is
released, the pending visit is retained, both readiness requirements still apply,
and old controls cannot move the rejoined player. Seven oracle guard tests cover
missed barriers, advancing clocks, stale processes, lost resources, wrong
visits, duplicated history and body resets. All 926 active Rust tests, 396 Python
guards, seven source-package tests and the locked release build pass.

These are staged transitions between visits of the school map, using real load
barriers and independent clients. They do not certify quest exits, all map pairs,
or another campaign visit. The added 0.25 point credits loading-interruption
coverage only. No screenshots were inspected.

## Moving-lift reconnect evidence, 5 November 2026

The automated `lift-rejoin-blue` and `lift-rejoin-red` regression cases pass
over public internet invites on build `4b3760a`, with 40 ms delay and 20 ms jitter
per sender. Both players walk from the staged lower-library start. The first
rider waits two shared-world seconds without advancing the lift pose or motion
clock. After the second boards, both ride the original rotating platform.

The runner terminates only its guest process during the ascent, verifies the
host has observed disconnection, and checks that the full frame and combat/world
snapshot stay frozen. It then starts a new guest on the same profile and invite.
The fresh client retains the authored motion keys, phase and matching platform
pose. Each rider retains their position in the lift's rotating coordinate system,
grounding, identity and starting resources. No live position, health, inventory
or platform value is written. The normal ride must still finish at the upper
landing and pass the complete sent/restored player and shared-world comparisons.

| First boarder | Duration | Matching player/world states | Final health |
| --- | ---: | ---: | --- |
| Blue | 84.27 seconds | 857 / 857 | 100 / 100 |
| Red | 88.20 seconds | 920 / 920 | 100 / 100 |

Blue-first reconnect advances from ticks 6099 to 6101, with platform time
2.2500017 to 2.2666686 seconds. Red-first advances from ticks 6632 to 6633,
with platform time 2.3916702 to 2.4000037 seconds. Maximum measured rider-offset
drift across either restart is below 0.00025 units. Both pairs reach height 64.
The source game is unchanged; these are new automation and verification cases.

Suite: `private/coop/regression-suite-f6fda7b53f104b29b1d16882a74c7cf3/summary.json`.
Cases: `private/coop/school-lift-da48bc0032364d7d81027f1cd26a5a53/` and
`private/coop/school-lift-5911f19b32d4434d984c9c26453fb46a/`.
The preceding Blue-first loopback baseline and reconnect also pass in
`school-lift-2a306facf0484287b2cfc161ffb2a112/` and
`school-lift-93658396311b4ddf9dd5b827ac71bff5/`, under `private/coop/`.

Five new guards reject stale processes, resets/skips, invalid motion poses,
changed resources/body identity, detached riders and falsely crediting a
stationary entrance reconnect as a ride. An unchanged world-space player
position is explicitly rejected when the platform rotates/translates beneath it.
The locked release, 925 active Rust tests, 389 Python guards and seven package
tests pass. No screenshots were inspected. The added 0.25 Host/Join points credit
the verified moving-ride reconnects; no campaign visit is added. Overall: 52.6%.

## Cutscene reconnect and public menu evidence, 5 November 2026

Build `4b3760a` passes four `tools/test_coop_scene.py --local --rejoin` cases
with 40 ms delay and 20 ms jitter per sender. Each terminates the guest during
the original theatre, observes the host stop, verifies a held full frame and
combat/world snapshot, and joins with a fresh process. No scene is skipped.

| Owner | Restart requested at scene time | Rejoin active ticks | Shared reunion tick | Separation |
| --- | ---: | ---: | ---: | ---: |
| Blue | 6 seconds | 13 | 5848 | 48 units |
| Red | 30 seconds | 13 | 5864 | 48 units |
| Blue | 30 seconds | 14 | 5846 | 48 units |
| Red | 6 seconds | 14 | 5862 | 48 units |

The six-second cases retain queued dialogue during the camera introduction;
the thirty-second cases retain the same active line and its elapsed time.
The scene clock advances by exactly the active host ticks, within float
tolerance. Ownership, pending contacts, starting personal resources and body
identity survive; the waiting partner remains frozen and hidden in both clients.
Both clients watch the complete scene and agree on the normal reunion.
Four guard tests reject resets, stale PIDs, changed resources/contacts, early
dialogue, replacement bodies, moved partners and incorrect visibility.

Evidence under `private/coop/`, in table order:
`scene-test-fe7e1caf696442ae815c15196c0adf80/`,
`scene-test-ac93e19fe7cd45ec89f60eb3b0cd4f0f/`,
`scene-test-6c647ad26dfc4337b13b758f287ef07f/`,
`scene-test-0e3e8f4f435d4509b8d289c63a0362db/`.

The old Blue-first fixture was blocked by the waiting Red in its single-file
approach. Before simulation, the fixture now assigns the collision-checked
front spawn slot to the requested owner and records that initial checkpoint.
The actual walking/contact still starts the scene. Normal spawning, collision,
scene rules and saves are unchanged. The stalled attempt remains in
`scene-test-f144fa012a1c42e290e6d1b8222fd59c/`; two harness failures involving
an omitted pre-scene field and queued dialogue remain in `scene-test-ca2da77805ba40b59ec0c0530cca57e3/`
and `scene-test-1fd35a937eb944539efea32501163dee/`. The reader and oracle now
match those actual serialized states. An earlier Red reconnect on `0b32c79`
passes in `scene-test-0578514c1d51406389913f254d5a12b5/`.

After more than an hour without public provisioning attempts, the final build
passes `tools/test_coop_menu.py` with `internet: true` and real Windows input:
`private/coop/menu-test-5194337850bb40e1bf2fbfae0f52487e/`. It loads a solo slot,
hosts that current game through ESC, joins a guest, saves, leaves and returns
to the exact suspended solo state. It then verifies expired-link feedback,
Continue co-op with a new public invite and a second exact solo return. Source
solo bytes stay unchanged. Batched typing, invalid input and Cancel also pass.
This completes the previously open public menu case; it does not prove that
the earlier DNS failures or provider rate limit cannot recur.

The locked release, 925 active Rust tests, 384 Python guards and seven source
package tests pass. No screenshots were inspected. The added 0.25 Host/Join
points credit the cutscene reconnect cases, and 0.5 credits the complete public
menu flow. No campaign visit is added. Overall: 52.3%.

## Countdown reconnect evidence, 5 November 2026

Build `0b32c79` passes both `tools/test_coop_play.py --local --death --rejoin`
roles with 40 ms delay and 20 ms jitter per sender. The runner terminates only
its guest process during a real guard-inflicted death, waits for the host to
observe the loss, checks the full paused frame and combat state, then starts a
new guest with the same profile and invite. The fresh process must inherit both
players' inventories, health, body identities and the correct camera. No health,
position or clock is written to create the result.

| First victim | Death tick | Revival tick | Health / separation | Shared wipe tick |
| --- | ---: | ---: | --- | ---: |
| Blue | 2008 | 3208 | 25 / 48 units | 4203 |
| Red | 2077 | 3277 | 25 / 48 units | 4261 |

Both clients agree on these events. Blue's reconnect retains 847 then 839 ticks
with eight active ticks elapsed; Red's retains 891 then 877 with 14 active ticks
elapsed. Evidence: `private/coop/play-test-8a35b8e7f545481e9ad50f19dcf5c04f/`
and `private/coop/play-test-f9c88e2556c4491baa9d0ee5836f6b9c/` respectively.
Four guard tests reject stale processes, rewound/reset clocks, replaced bodies,
wrong cameras, changed resources and unexpected healing. The lifecycle verifier
also rejects a countdown reset between samples.

The first Blue attempt on `8c02fd0` failed because the old straight-line test
approach stopped at Red's contact radius, leaving Red as the nearer victim.
That failure remains in `private/coop/play-test-3f9923da110c4425836934a832e65d2f/`.
The development driver now walks around Red through observed waypoint arrivals;
normal movement, collision, AI and damage rules are unchanged. The earlier Red
baseline and reconnect pass on `8c02fd0` remain separate historical evidence.

Build `c57d7df` also passes all 82 native `--coop-world-check` records, including
shared boarding/transport, personal pickups, both theatre owners, safe reunion
and checkpoint restoration. These are focused core checks, not campaign routes.

The staged combat repeat initially failed on `c57d7df`: Blue was again blocked
behind Red, so only Red attacked. Evidence is retained in
`private/coop/play-test-909f0165682245ba8917924ba1e28364/`. The combat driver now
uses a clear side approach and waits for both players to reach Blade range.
Build `0b32c79` passes the unchanged combat assertions with two real attacks from
each player, a shared guard defeat and minimum health 100/64 on both clients.
Evidence: `private/coop/play-test-2beed43adfdc4c0ea3866f76cc95f35b/`.
Both driver corrections are restricted to explicit development scenarios.

The final `0b32c79` contact/ambient-effects run passes 612 matching sent/restored
states, both directions of approach, stationary-partner preservation and a
36-unit contact distance. Both models remain visible and both clients render
ambient effects without dropped batches. Evidence:
`private/coop/presence-test-a55f8bfb1d2b4523b2b14fb5bb18528d/`.
All new paired checks use loopback with 40/20 ms delay/jitter; no screenshots
were inspected. The locked release, 925 active Rust tests, 380 Python guards
and seven source-package tests pass. The unchanged native world checks above
were run on `c57d7df`, before the combat-only driver correction.

The added 0.25 Host/Join points credit the newly verified countdown reconnect
cases. They do not resolve public invite provisioning/DNS, finish core acceptance
or complete another campaign visit. Overall: 51.6%.

## Menu acceptance evidence, 5 November 2026

Build `8c02fd0` passes `tools/test_coop_menu.py --local` with real Windows mouse
and keyboard input in Anode. Evidence:
`private/coop/menu-test-d6691e95748e4e9583c1d6368a779c57/`.
The original main menu loads the solo slot; ESC hosts that current game. Both
clients retain the expected positions, resources and opened door. The ordinary
co-op menu saves and leaves without replacing the manual checkpoint. Returning
to solo preserves its entire serialized state hash; ordinary movement then
works. Continue co-op restores the shared save independently of that changed
solo position, issues a new invite and returns to the second suspended solo
state unchanged. The source solo slot retains its bytes throughout.

The runner also verifies batched invite typing, malformed-link feedback,
cancelling a pending handshake, an expired session link, and menu recovery after
the host leaves. A real typing bug was fixed: Macroquad returns a character
batch newest-first, so the invite field now restores event order. Passive,
opt-in reports provide actual button bounds and state receipts; no screenshots
were inspected. A harness focus failure was corrected by activating only its
owned game windows; the passing full run supersedes those harness failures.

The public attempt on the same final executable fails at provisioning with
HTTP 429: `private/coop/menu-test-74dfab0b5f304251a516993ff98b456a/`.
It verifies the improved retry-later error and intact solo return, but cannot
credit the full public menu flow. The preceding public attempt on `e0fe7f6`
failed the same way. No automatic retry loop or provider-limit workaround was
added. Prior successful internet persistence tests remain build-specific evidence.

The additional 0.25 Host/Join points credit verified menu routing, input and
error/cancel handling. They do not credit public menu connectivity, resolve DNS
reliability, complete core acceptance or add a campaign visit. Overall: 51.3%.

## Storage and host-loss evidence, 5 November 2026

Build `7994cc2` passes the extended `tools/test_coop_import.py` suite over public
internet invites with 40 ms delay and 20 ms jitter per sender. Evidence:
`private/coop/import-test-6d6f290f6632436784c49ab6e67a5e9d/`.
The native storage receipt records two real sharing-violation failures and two
exact team-wipe restores of the old checkpoint. It also verifies stale temporary
files, successful checkpoint replacement and backup preservation after corruption.
These cases do not simulate electrical power loss or disk exhaustion.

The harness terminates the resumed host and its helper without a graceful exit.
The guest reports the disconnect in 6.33 seconds; both committed files retain
their bytes. Fresh processes then resume the save, rejoin, and recover the backup.
All four network phases retain the two inventories, shared world and solo files.
This CLI diagnostic evidence does not yet verify the normal menu's error panel,
return to the suspended solo game, or subsequent hosting through ESC.

The earlier invite DNS failure did not recur in six fresh native game sessions
or six headless transport sessions. Its cause remains unproven and no resolver
fix is claimed. These successful retries do not close intermittent reliability.
A separate full multi-visit checkpoint rerun on `7994cc2` passes travel/rejoin,
then fails before the resumed host can publish an invite: the temporary tunnel
provider returns HTTP 429 during provisioning. Evidence:
`private/coop/travel-test-1bbdc0c80f44484fbaf8edd1441b4542/`.
That run remains failed; repeated new-session setup hit a real provider limit.
No immediate internet retry or rate-limit workaround is used.
The same multi-visit checkpoint passes separate primary and backup resume/wipe
checks on loopback with the same 40/20 ms conditions:
`private/coop/storage-resume-local-32dbde14645349219084727e4fe31ceb/`.
This isolates persistence from the provider failure; it is not an internet pass.

That storage milestone left the total at 51.1%; it strengthened core evidence
without completing the menu/connection milestone or adding a campaign visit.

## Solo continuation evidence, 5 November 2026

Build `a9b55a5` passes `tools/test_coop_import.py` over public internet invites
with 40 ms delay and 20 ms jitter per sender. Evidence:
`private/coop/import-test-9e0363c7644545c8a8313b8c27a48b5a/`.
The native fixture verifies Blue's full body, view, resources, actions and world,
the distinct cached return visit, and safe Red placement. A separate native
swimming case retains Blue's body/air supply and places Red at the same depth.
Each network phase includes a guest process restart using the current invite;
both host and guest restart between primary and backup sessions. The solo slot
and guest sentinel remain unchanged. No image inspection was needed.

The initial build `08033a5` also passed the same three network phases. A later
fixture attempt started in Garden's opening cinematic and correctly refused
import; the swimming fixture now uses freely controlled Temple gameplay. The
observed DNS failure remains an open reliability item despite successful import
runs. The separate final checkpoint run
`private/coop/travel-test-c2fb379c358d42c4b059f91bea7b1ea3/` passes staged travel,
guest reconnect and primary resume/team wipe; its backup guest failed DNS
resolution. The runner now reports that failure immediately. A separate loopback
backup/team-wipe test passes with 40/20 ms conditions and unchanged files:
`private/coop/backup-local-a747d7b0866a4aaf840cc3174b2c852a/`. This isolates save
correctness from internet setup; it is not an internet backup/wipe pass.

Validation: 923 active Rust tests (three existing ignored), locked release,
376 Python guards, seven source-package tests and source audits. This change
does not finish all core acceptance work.

## Evidence history

The dated and build-specific results below retain historical percentages;
the current overall estimate is the one at the top of this document.

Latest native milestone: Burning Curiosity source `56160ff` passes **26 staged
cases plus one summary**. Each living player receives the original arrival
grant once and retains their own two essence pickups. The sinking platform waits
for both living riders, carries them at 30/60/144 Hz, transfers them onto the
uneven floor independently and returns when empty. A blocked return rolls back
the platform and both bodies. Original arrival and first-entrant Caterpillar
scenes pass watched/skipped, living/dead partner and compressed-save cases.

The locked release passes seven single-player component checks and the full
native combat route; both outputs match the previous baseline exactly. The
separate cast-free traversal still has its pre-existing step-54 lava-jump
failure, also unchanged. These results add **0.75 points** under the existing
component weighting: 0.25 resources, 0.25 movement and 0.25 progression.
No additional campaign visit is counted as complete. Next is Burning Curiosity's
full route with two independent internet clients, followed by recovery and
reconnect checks. See [Burning Curiosity evidence](JLAIR1.md).

Current validation: **919 active Rust tests** pass (three ignored), along with
the locked release, twenty traversal and five recovery guards for Curiosity
(376 guards across the Python suite), seven package checks and
the 912-file source worktree audit. Negative tests cover partial boarding,
floating bodies, marginal ledges, damaging-liquid navigation and stationary
bodies that retain collision velocity, alongside valid sloped landings.
No images were inspected for these milestones.

Burning Curiosity's first fresh internet segment now passes **steps 0-3** with
eight personal arrivals, watched entry, both early lava crossings and 2,133
matching player/world/native states. Both survive without revival at 98.75/95.42
health; a normal checkpoint before step 4 is retained. The longer approach and
full route remain open. This segment adds no percentage or completed-visit credit.
A later resumed attempt has independently verified evidence through **4-20**:
34 personal arrivals, five forward waiting walks and 5,582 matching states, with
no deaths. That longer attempt fails at guest planning for step 21; its successful
prefix is bounded separately. This also adds no percentage or completed visit.
Source `ca4cc26` then corrects the automated planner's floor-plane checks, keeping
its eight-unit support margin and requiring actual body contact. The previously
stalled climb now passes in a separate resumed **21-23** internet run: six personal
arrivals, one forward waiting walk and 2,224 matching states. Both finish at
100 health without revival; a normal checkpoint before 24 is retained. Native
component and single-player baseline outputs remain identical.
Later bounded **24-25** evidence adds four personal arrivals and 1,563 matching
states. Two attempts fail at 26 under fire-spitter attacks. The runner now allows
ordinary combat dodging before parking and uses the original route's Cards
preference. A resumed **26-29** run passes eight personal arrivals, the fight and
1,374 matching states, ending at 50/96 health without death or revival. Its normal
checkpoint before 30 is retained. A further attempt verifies **30-31** separately
(four arrivals, 352 matching states), then fails the forward climb from 32 toward
33 when Red falls into lava. A separate retry verifies **30-32** with six
arrivals and 783 matching states, taking a different ascent before failing
parking at 33; climb reliability remains open. Forward waiting now uses actual
native dry bays instead of treating the next route point as a safe stopping
place. Bounded **33-37** evidence has ten arrivals and 1,309 matching states.
A separate bounded **38** replay uses ordinary jumps onto the original hot
waypoint and back onto dry ground: two arrivals and 462 matching states,
93.75/41.67 health, no deaths or revivals. Its longer attempt fails at 39 when
the planner walks into lava. Those full attempts stay failed. Source `259d4f5`
adds optional liquid avoidance and corrects blocked-walk detection to use actual
position progress instead of retained collision velocity. A resumed **39-42**
internet replay now passes eight personal arrivals and **1,335 matching states**,
ending at 100/56.67 health with no deaths or revivals. The normal checkpoint
before 43 is retained. A subsequent remote three-jump sequence fails at 43.
Source `5c4a693` stops timed sequences after the first planned landing and settles
the actual body before planning onward. The runner also follows successive
original approach points when a narrow slope has no nearby waiting bay, with
separate personal receipts for every intermediate point. Bounded **43-45**
evidence then passes six arrivals and **869 matching states**, ending at
100/71.67 health without deaths or revivals. That longer attempt fails at 46
when a new plan starts during an unfinished ledge pull. Its normal checkpoint
before 46 is retained. Source `c7576ca` then waits for active ledge pulls to
finish before starting another route search. A separate **46-49** internet run
passes eight personal arrivals and **1,017 matching states** at 100/46.67 health,
with no deaths or revivals. A separate resumed **50-52** segment passes six
arrivals and **699 matching states** with no further damage, death or revival.
The normal checkpoint before 53 retains both players beside the raised platform.
Separate evidence covers approach points 0-52; a fresh continuous repeat is
still required. The later platform checks below do not complete the remaining
route, recovery or reconnect coverage.
A separate **53** internet segment on `9f1acd0` now passes both personal
arrivals and actual single-rider waiting: Red has full native deck contact,
Blue remains off the platform, and it stays raised for at least 120 ticks in
both clients. There are **317 matching player/world/native states**, no new
damage, death or revival, and a normal checkpoint before 54. This build only
adds read-only platform contact diagnostics; the native platform cases and
single-player outputs remain identical.
The paired platform crossing now passes in a closed **54-only** segment:
both players reach the opposite bank alive, Blue waits there for Red, a normal
checkpoint before 55 is saved, and 152 player/world/native states match.
A longer **50-54** repeat then passes ten personal arrivals, actual boarding,
waiting and carry, both ordinary jumps and stable bank waiting. All **1,145**
comparable player/world/native states match; both retain **100/46.67 health**
without damage, death or revival. The current checkpoint before 55 is retained.
The crossing destination is explicitly the flat bank `[-7208,768,1632.03125]`,
instead of the single-player fixture's sloping point 54. That point is not a
game trigger. The original course file is retained and the changed crossing
goal is recorded; this is no claim of landing at the old coordinate. Only
normal controls are used, with native collision, resources and level behavior
unchanged. The subsequent route evidence is recorded below. No screenshots
were used.

Post-platform evidence now covers **55-64 in separate segments**. A closed
55-57 internet run passes six personal arrivals and 572 matching states with
100/46.67 health and no new damage, death or revival. A later run has a verified
58-63 prefix (12 arrivals, 1,326 matching states), then fails at 64. On source
`0871d86`, a verified 64 prefix passes both arrivals and 255 matching states,
then times out at 65. These bounded audits retain the parent failures.

The diagnostic driver can finish a supported grounded approach before planning
its jump; this is exercised by Blue at 64. Forward waiting now rejects fleeting
fast landings. A grounded lava-planning failure can trigger a recorded ordinary
jump or automatic climb back to recent safe footing. Other driver failures
remain fatal, and a recovered planning error requires a verified escape receipt.

Blue reaches 65 through an additional ordinary approach, but paired 65 remains
failed: after a verified lava escape, the next jump falls into deep lava. A
combined 55-64 repeat also fails at 60 while Red is swimming in lava; the
current grounded escape action cannot recover that state. Continuous route
reliability, 65 onward, a fresh full visit, recovery and reconnect remain open.
The latest native components and single-player outputs still match baseline;
no gameplay physics, resources or level progress were changed by these controls.
Total remains **50.1%**; no extra completed visit is credited.

Latest network milestone: a **fresh full About Face route passes** in two
independent internet clients on source `3298768`, using ordinary controls with
40 ms delay and 20 ms jitter. Ten personal arrivals, both reward pickups, the
shared lift, Red-owned watched Gryphon exit and actual shared `jlair1` arrival
are verified. The closed evidence has **6,805 matching player/world/native
states**. Blue's death at 23,061 revives at 24,261 with exactly 25 health and
verified resources, placement and spectator/returning cameras. A separate
resumed ending verifies Blue ownership with 2,169 matching states.

Latest recovery milestone: Red also revives after exactly 1,200 active ticks
at 25 health, with resources, safe placement and cameras verified across 1,087
matching states. A fresh guest process rejoins an unchanged normal victory save
with personal resources, pickup ownership and shared progress preserved: 77
matching states, including 58 after rejoining. Timed drops age normally.

Latest wipe milestone: both players die through ordinary combat and return
together at tick 7,060 to the normal checkpoint saved at 3,421, at 100/100 health.
Source `116fc99` adds an opt-in exact restore receipt before another tick can
change the world. Characters, enemies, pickups, shared progression and personal
resources match the actual saved checkpoint. Both clients agree on 1,058 states,
including 27 after restoration; the closed 917-row trace verifies the boundary.

All 905 active Rust tests, the locked release, 83 native cases plus three
summaries, 16 About Face Python guards and seven package checks pass. The
single-player component and full-route outputs still match the baseline exactly.
Renderer counters from the earlier full route show particles/effects in both
clients with zero dropped batches; no images were inspected. Combat/transport/
transition reconnects and remaining ending cases keep this visit partial.
The remaining About Face checks stay open in the matrix. This recovery milestone
retained the then-current **49.3%** estimate and six combined visits.

Earlier native milestone: About Face source `3298768` passes **83 staged cases
plus three summaries** for combat, personal resources, both lifts and the original
scenes. The bridge waits for both players to clear it; lifts enforce boarding and
carry living riders together, including obstruction rollback and catch-up from a
supported destination. Each player retains their own rewards. Legacy saves and
low-health revivals keep their health. Gryphon scenes bind the first entrant's
pose on the trigger frame and preserve the partner through watched/skipped play.

**905 active Rust tests** (three ignored), the locked release, seven package
checks and the 900-file source worktree audit pass. Both single-player checks,
including the full input route, match the unchanged baseline exactly. No images
were inspected. Under the existing component weighting this adds **0.75 points**:
0.25 combat/resources, 0.25 movement and 0.25 progression. The total then was **49.3%**.

No campaign row is completed by staged components. Six of 39 retain combined
verification evidence. About Face's remaining recovery/reconnect and
ending cases stay open. Clockwork's full-route and
recovery gaps also remain. See [About Face evidence](HATTER2.md).

Latest completed traversal sections: resumed **144-145** and **146-149** pass.
Both players enter the lowered clock portal through ordinary jump/climb input,
then personally reach the final lever approach. The first arrival leaves the
portal exit clear for the second. This section has **361 matching
player/world/native states** and retains the normal checkpoint before 146.

Red then uses the final lever and owns the watched 30-second Stop scene. Blue
stays hidden and still; both reunite safely. After the original 20-second wait,
Red approaches and enters the original exit, moving both players into About Face.
The shared transition has its own receipt, rather than claiming that both
players walked to the exit waypoint. This section has **1,122 matching states**,
including **27 after arrival**, with health preserved at **26.93/23** and personal
resource, camera and body-clearance checks passing. No new deaths occur.

Earlier resumed 141 verifies Red's exact 10-second revival and a new personal
chair approach; 142-143 verifies onward waiting. All **210 Clockwork Python
guards**, **15 generic recovery guards** and seven package checks pass. Those Clockwork runs used native
source `84c8eee`; its previous 905 active Rust tests, 72 paired/driver checks and
exact single-player 11+1 baseline were not rerun for those Python-only changes.

Remaining Clockwork work: fresh full-route, remaining scene-owner, recovery and reconnect verification.
These checkpoint segments through the exit do not complete the full visit's
verification matrix or add another campaign row.
See [Clockwork evidence](HATTER1.md).

Earlier diagnostic work: source `84c8eee` detects an actual off-course landing
before sending later jumps from the old plan. It also reports nearby dry,
fully supported return targets from native geometry, without moving either
body. The chair fixture requires the real jump and landing on that target.

The new resumed **140** replay records both personal chair arrivals, one Red
pull-up and Blue's safe return, with **2,312 matching player/world/native
states**. It still **fails**: Red reaches the chair with about **8.53 health**
and dies landing after the return jump. Her approach already incurred normal
damage, including slime. No rescue warp or revival occurs before either arrival.
The earlier replay timed out during a corrected route search; the bounded
approach allowance is now 180 seconds, while departure remains 30 seconds.
Neither failed replay earns a completed paired step or campaign visit.

Validation passes **905 active Rust tests** (three ignored), the locked release,
**72 native paired/driver checks**, the exact single-player **11+1** baseline,
**169 Python guards** and seven source-package checks. The 40 scene cases still
belong to earlier source `04d3b0c`. A survivable third-chair approach/departure,
the fourth chair and later route remain open. Overall stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Previously completed traversal section: resumed **138** now passes for both players.
Each reaches the original chair waypoint, then uses an observed native jump to
return to dry ground before the partner approaches. The run verifies the puzzle
contact, two personal arrivals, two departures, one observed Red pull-up and
**1,828 matching player/world/native states**. Both survive at **67/67.73 health**;
ordinary slime and combat damage is retained. A normal checkpoint before 139 is
saved. The earlier failed waiting arrangements remain recorded separately.

The next requested 139-143 run completes **139**. Its separately audited section
adds two arrivals, two departures and **1,741 matching states**, ending at
**54/43.53 health**. Step 140 remains unverified: Blue's selected return point
is beneath a higher landing, and Red's delayed jump route triggers the original
rescue portal before a new search times out. The runtime now enforces the same
return-height check as the final verifier, so it stops at that mismatch earlier.

The fixture returns to the first nearby dry footing from the actual approach.
A reproduced selection bug chose a later point beneath the chair rim, leaving
Blue above her target; its regression now rejects that choice. All **166 Python
guards** and seven source-package checks pass. Native source remains `99a66d6`,
with **901 active Rust tests** (three ignored), the locked release, **72 native
paired/driver checks** and the exact single-player **11+1** baseline. The previous
**40 scene cases** belong to source `04d3b0c`; they were not repeated for these
diagnostic changes. The remaining chairs, second portal, final scene/exit and
full-route/recovery/reconnect repeats remain open. No additional campaign visit
is credited; overall stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier completed traversal section: Clockwork's resumed **136-137** portal section passes
with two personal approach arrivals, two input-driven native climbs, separate
portal crossings, one waiting handoff and **202 matching player/world/native
states**. Both players remain alive at **80/100 health**; the leader takes two
ordinary hits while waiting beyond the portal. No health is reset for the test.

The preceding **128-135** doorway section is separately audited with ten
personal arrivals, five waiting handoffs, three timed holds and **562 matching
states**, ending at 100/100 health. The first player observes both rotating
leaves opening and walks forward before the partner approaches. The original
route's later wait and both players' waypoint arrivals remain required.

A reproduced portal bug could place one Alice directly inside the other.
An occupied exit now waits for the living partner to move clear. The test
controller also preserves a consumed jump across teleports, while allowing
new commands and new visits to jump normally. The retained failed portal run
is rejected by the verifier for its repeated jump at the destination.

Frozen source `04d3b0c` passes **897 active Rust tests** (three ignored), the
locked release, **72 native paired/driver checks**, the unchanged single-player
**11+1** baseline and **40 staged scene cases**. All **151 Python guards** and
seven source-package checks pass. The following **138-143** attempt times out
at the first chair approach after an incomplete remote ledge pull; it earns no
completed steps. Both players remain alive at 80/90 health. The chair section, second portal, final
scene/exit, full-route and recovery/reconnect repeats remain open. No additional
complete campaign visit is credited; overall remains **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **109-116** section passes with
eight personal arrivals, the watched Blue-owned Gryphon scene, four waiting
handoffs, three timed holds and **1,353 matching player/world/native states**.
A separate Red-owned Gryphon replay of **112-113** passes with **960 matching
states**. Both runs finish at 100/100 health and verify all seven dialogue
stages, hidden/frozen partners, preserved resources and safe shared reunions.

The next requested **117-136** run stops at 128 because its narrow approach
offers no usable sideways waiting position. Its separately audited **117-127**
section passes with 22 arrivals, 20 pathnode projections, 11 waiting handoffs
and **1,514 matching states**, also at 100/100 health. A normal checkpoint
before 128 is available for the next retry. The original failure is retained.

Native source remains `11c5ba2`; all **40 staged paired scene cases** now pass
on this exact build, including both owners, watched/skipped scenes, pause/restore
and living/dead partners. All **131 Python guards** pass. No further complete
campaign visit is credited. Later traversal, portals, the final scene,
recovery/reconnects and full-route repeats remain open; overall stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **107-108** doorway section passes
on source `11c5ba2`, with four personal arrivals, two observed door approaches,
one forward handoff, one waiting handoff and **692 matching player/world/native
states**. Both players finish at 100 health, and an ordinary checkpoint before
109 is saved. The runner lines up at authored waiting points and observes both
rotating leaves opening before continuing to the original waypoint. Its precise
walking handoff now checks walls while permitting normal step-up clearance.

All **893 active Rust tests**, the locked release, **72 native checks**, the
unchanged single-player **11+1** baseline, **124 Python guards** and seven
source-package checks pass. Earlier unsuccessful doorway replays are retained.
This completes the previously stalled section, not the whole visit. Later
Clockwork scenes/traversal, recovery/reconnects and campaign repeats remain open;
overall completion stays **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's completed resumed **99-106** section
verifies 16 personal arrivals, two pathnode projections, seven forward handoffs,
one waiting handoff and **1,845 matching player/world/native states**, with both
players at 100 health. The requested 99-108 run times out at 107 while the leader
approaches its next waiting point. It stops about 16 units short with the native
controller reporting `settling landing`; this remains the next investigation.
The original failure and the separately audited completed section are retained.

This batch also verifies **81-93** (2,421 matching states, 46/85 health) and
**91-103** (3,181 matching states, 100/100 health), each scoped separately from
its original longer-run failure. The test controller now coordinates narrow
stairs and winding paths, permits ordinary defensive movement while waiting on
static ground, and bounds an unsuccessful melee approach before allowing a
normal Blade throw. Source `cefabf5` passes **891 active Rust tests**, the locked
release, **72 native checks** and the exact single-player **11+1** baseline;
all **116 Python guards** pass. No additional complete visit is credited.
Later machinery/scenes, recovery/reconnects and full campaign repeats remain
open. Overall stays **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **68-81** section passes with
26 personal arrivals, 16 pathnode projections, 13 waiting handoffs, one timed
hold and **2,294 matching player/world/native states**. Both finish alive at
16.53/100 health. The combat test controller now requires supported footing
under its full stance through delayed movement and braking. The regression
fails on the old controller and passes with the correction; the real internet
replay also passes the fight that previously killed Red. Source `894292f`
passes **890 active Rust tests**, the release, **72 native checks**, the exact
single-player **11+1** baseline and **110 Python guards**. Ordinary checkpoints
allow targeted retries near this fight and the next route section.

The preceding requested 68-108 run remains failed at 77, with a separate scoped
audit crediting only 68-76 and 1,237 matching states. Later Clockwork traversal,
recovery, reconnects and complete campaign repeats remain the bottleneck. No
further complete campaign visit is credited; the estimate stays **48.6%**.
See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **60-67** section passes with
nine personal arrivals, two native scene handoffs, four waiting handoffs, one
lever, two timed holds and **2,971 matching player/world/native states**. Blue
owns the watched Hare scene and Red owns the delayed Port scene. Separate
retained runs verify Red's Hare scene (**1,791 matching states**) and Blue's
Port lever/scene (**378 matching states**). Both stay at 88/78 health, and the
scene checks require hidden/frozen partners, preserved resources and safe shared
reunions. Scripted reunions are recorded separately from walking arrivals.
All **110 Python guards** pass; native source remains `6f589de`. An intervening
startup failed DNS resolution; the next new internet session passed. Later
Clockwork sections, skipped-scene/recovery/reconnect repeats and full traversal
remain open. Overall completion stays **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **44-59** section passes with
32 personal arrivals, 32 independent pathnode projections, 16 waiting handoffs
and **3,565 matching player/world/native states**. Both finish alive at 88/78
health. The parking controller now takes bounded, observed sideways detours
around corners; the previously failed corner at 52 passes through ordinary
movement. A normal checkpoint before 60 is retained. Source `6f589de` passes
**889 active Rust tests**, the locked release, **72 native checks** and the exact
single-player **11+1** baseline; **85 Python guards** pass. This is resumed
section evidence, not fresh traversal or a completed campaign visit. Clockwork's
later machinery/scenes, recovery, reconnects and complete repeats remain open.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **43-51** section verifies 18
personal arrivals, 16 independent pathnode projections, nine waiting handoffs
and **2,634 matching player/world/native states**. Both finish alive at 88/100
health. The requested 43-59 run stops at 52 because its parking selector cannot
take a short sideways detour around a corner. The original failure is retained;
the completed section has a separate scoped audit. Source `6f589de` passes
**889 active Rust tests**, the locked release, **72 native checks** and the exact
single-player **11+1** baseline. The parking fix passes **85 Python guards**.
No additional complete campaign visit is credited.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **36-42** gap sequence passes
through independent internet clients: six personal arrivals, two actual gap
jumps, two preparation holds, one waiting handoff and **232 matching
player/world/native states**. Both finish alive at 100/99.867 health. A normal
checkpoint beyond the gap is retained for the next section. Remote plans now
require supported first landings and retain subsequent jumps; direct airborne
steering preserves enough speed to cross the gap before settling. Source
`a07517e` passes **887 active Rust tests**, the locked release, **71 native
paired/driver checks** and the exact single-player **11+1** baseline. This is
resumed segment evidence; full Clockwork and campaign repeats remain open.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's resumed **25-35** section verifies 16
personal arrivals, seven corridor handoffs, two timed holds, one lever and
**1,073 matching player/world/native states**, ending with both alive at 89/100
health. This is a scoped audit of a longer run that stopped at the gap approach.
The next retry exposed a delayed landing failure before that gap. An earlier
braking handoff now passes **886 active Rust tests**, including a regression
that reproduces the old fall. The new live retry verifies **33-34**, including
the previously failed landing, with four personal arrivals, two handoffs and
**237 matching states**. It then fails at 35 with Red falling during a different
jump approach; the gap sequence remains untested. Full Clockwork stays open.
The locked release, 68 paired native checks and exact single-player 11+1
baseline pass, as do **72 Python guards**. Total completion stays **48.6%**,
with six fully verified campaign visits.

Earlier automation result: Clockwork's resumed **18-24** section verifies 12
personal arrivals, all three stair handoffs, one timed hold and **454 matching
player/world/native states**, with both players at 89 health. The leader moves
to the next tread before the follower arrives. The runner also waits for command
acknowledgments retained in its history; **59 Python guards** pass. The requested
18-38 run stops at step 25 while selecting a corridor waiting position. Its
completed section is audited separately, and a normal checkpoint before 25 is
retained. Full Clockwork remains open; total completion stays **48.6%**.

Earlier automation result: Clockwork's resumed gear exit, **step 17**, now verifies
both actual jumps and personal landings, with **245 matching player/world/native
states** and both players alive at 89 health. Route planning now holds a stable
position against gear carry using ordinary controls. The exit test jumps from
the verified gear waypoint because a static walking route can lose support as
the gear turns. Source `cf5d551` passes **885 active Rust tests**, the locked
release, **68 native paired checks**, **50 Python guards**, and the unchanged
single-player **11+1** baseline. The resumed exit is separate from fresh traversal
and does not complete another campaign visit. Overall remains **48.6%**.

Earlier automation result: Clockwork's resumed **14-16** segment now verifies both
players boarding the rotating gear, with two actual jumps, four personal course
arrivals, one timed hold and **447 matching player/world/native states**. The
first player holds its waiting point within 2.76 units while the partner boards;
both remain at 89 health. The test driver compensates for observed gear motion
using ordinary controls. Source `623fece` passes **883 active Rust tests**, the
locked release, **68 native paired checks**, **43 Python guards**, and the exact
single-player **11+1** baseline. The native gear checks include both rider orders
and delayed observations/inputs. The original requested 14-17 run still fails
at the exit with repeated planning; it remains marked failed. This resumed
segment does not extend the fresh traversal or complete another campaign visit.
Overall completion remains **48.6%**. See [Clockwork evidence](HATTER1.md).

Earlier automation result: Clockwork's fresh **0-15** prefix verifies 24 personal
course arrivals, eight intermediate arrivals, four jumps, one pull-up, two timed
holds, two lever activations and one ordinary partner yield. All **2,997** compared
player/world/native states match, with both players alive at 89 health. The
runner now detects stalled contact even when attempted velocity remains high,
stops the follower, walks the waiting player to an observed clear position and
resumes the follower. **38 Python guards** pass. Native source stays `dbd12c8`.
The requested 0-17 run still fails at step 16 while repeatedly replanning on the
rotating gear; its scoped audit excludes that failed segment. No new complete
campaign visit is credited. Overall completion remains **48.6%**.

Earlier automation result: Clockwork's fresh entrance (steps 0-1) passes with
either player going first on source `dbd12c8`. The two internet runs verify
697 and 693 matching states, each with four course arrivals, eight intermediate
arrivals, four actual jumps and one native pull-up. Both players finish at 89
health. A delayed final-steering correction and precise waiting-footing arrival
address the observed overshoot and blocked pull-up. All 881 active Rust tests,
the locked release, 27 Python guards and seven source-package tests pass.
A longer fresh run independently verifies steps 0-5 and 981 matching states,
then stops at step 6 with both alive: the parked first player blocks the
follower's planned approach. That coordination failure is retained. No additional
campaign visit is credited, and the overall estimate remains **48.6%**.

Earlier automation milestone: Clockwork now has a fresh two-client internet
prefix runner with separate personal arrivals, supported waiting moves, normal
lever inputs and checkpoints, and network-state comparisons. Read-only door
observations let its normal E input work with Clockwork-owned door groups.
Waiting positions prefer the far side of a doorway; parking on the incoming
side had pinned the follower against an open door's tip. Source `f6b1a4f`
passes **60 native machinery/door records**, all **40 scene cases**, and the
unchanged single-player **11+1** baseline. Four new local door cases cover
both actors and the proximity-open starting state, with ordinary partner yield.
The route runner now has 27 receipt/parking/jump/pull-up guards. Source `4fd024c`
adds grounded navigation to the optional test driver and passes 880 active Rust
tests and the locked release build. The component results above remain attributed
to `f6b1a4f`; the new build has not passed the live prefix.
This is additional automation and component coverage, not another completed
campaign visit. Total credit remains **48.6%**. See [scope and retained failures](HATTER1.md).
The earlier fresh retry died at step 1. Observed entrance takeoffs and landings,
ordinary ledge pull-up input, and precise centering now pass a fresh **0-5**
prefix: ten course arrivals, eight intermediate personal arrivals, four actual
jumps, one native pull-up, one timed hold and 872 matching player/world/native
states. Both finish that prefix alive at 89 health. The run then stops at step 6,
where the original single-player marker cannot satisfy the stricter co-op
arrival radius. A repeat passes 0-5 again, then Blue reaches the adjusted step-6
target before Red falls on the approach. The latest run on `4fd024c` connects
over the internet but stops at step 1: Red walks beyond the lower entrance
footing while the test driver is settling the next takeoff position. Its second
jump has not occurred. This is a failed route test, not a hung process or a
completed visit. Failures remain retained; full Clockwork remains open.

Earlier fix: Clockwork's gear barrier and laboratory door now wait for all
living players to pass their authored closing approach. The laboratory lever
alcove does not count as inside. Source `a007099` passes **18 local door cases**
with either first player, paused/compressed waiting-state restore, either/both
players dead and the outside-lever case. All 38 earlier machinery records,
40 scene cases and the unchanged single-player 11+1 baseline also pass.
The gear-door course begins at its open doorway; traversal/waiting on the
rotating great gear remains unverified. Total credit stays **48.6%** and the
completed campaign count is unchanged.

Earlier verification milestone: Clockwork's new native scene suite passes all
**40 cases**: four cinematics with either owner, watched/held-skipped play and
living/dead partners, plus both ordinary speech scenes with either initiator
and living/dead partners. It checks actual contact/lever activation, frozen
waiting positions, pause/compressed restore, camera and authored pose stability,
owner-only skipping, safe separated reunion and personal resource preservation.
Ordinary speech advances once while both living players move independently.
Source `ec48aa7` also passes all 38 machinery checks; the single-player 11+1
records still match the baseline exactly. This adds **0.25 scene/progression
points** for complete staged handoff coverage, not a full campaign visit.

Earlier implementation milestone: Clockwork (`hatter1`) now explicitly advances
one shared machinery clock, carries both living riders, waits for boarding,
and binds lever actions and their delayed scenes to the actual user. Source
`9ce4f98` passes **38 native paired component checks** covering lift and floater
carry, blocked-rider rollback, saved boarding contacts with either requester,
separate sinking pillars, all five levers with either actor, blocked approaches
and owner-specific death cleanup. The original single-player 11+1 component
records match the pre-change baseline exactly. All 879 active Rust tests, the
locked release, source-package seven and source-worktree 890 checks pass.

That adapter added **0.25 movement points**, using the same bounded credit as
earlier native paired-transport milestones. Full paired handoffs are now checked
separately above. Clockwork still needs rotating-gear/exit cases, continuous
two-client traversal, recovery and reconnects. Internet and completed-campaign
credit are unchanged. See [Clockwork coverage](HATTER1.md).

Funhouse's best verified fresh prefix remains **0-88** on source `f9e7029`:
122 personal arrivals, 27 own pickups, 14 paired holds, eight switch shots,
13,767 matching states and four exact ten-second revivals. Both finish at 100
health. The gas-scene fixture correction separately passes unchanged checkpoint
**89-94**, including the watched scene, safe shared reunion, six arrivals, two
holds and 578 matching states. That segment does not extend the fresh-run proof.

The latest fresh repeat with that correction fails at **36 after a team wipe**.
Its independently verified **0-34** prefix has 52 arrivals, six own pickups,
two holds, one switch and 4,788 matching states. The host trace shows both players
restored together at the normal checkpoint with 97 health; full two-client wipe
verification remains open. The failure is retained. No combat rule, resource,
position or team-wipe guard was weakened. Full Funhouse traversal remains open.

Two resumed mirror 29-32 runs also pass, including exact actual-jump evidence.
Conservative range rejection removes distant-enemy aiming work from the test
driver; its measured host p95 cost drops from 39.5 to 1.1 ms per frame on the same
PC. The earlier 0-37 prefix had no control resets; the latest run records one
early suspension. Gameplay physics, combat rules and
the control-reset safeguard are unchanged. **877 active Rust tests**, the locked
release, Funhouse battle63/world46/scene37/single-player8 and Checkmate94 pass;
single-player component output matches baseline except two benchmark timings.

Separate verified work: Funhouse's second personal pad and descent now pass
steps **151-155** with eight personal arrivals, both actual jumps and 396 matching
network states. Each player gains their own second 25-health/Will pickup.
A separate resumed combat run defeats both Tweedles, watches all eight Hatter
scene lines, preserves the hidden/frozen partner and the original floor-collapse
tail, then reaches **Hatter1 with both players alive**. It has 1,178 matching
Funhouse states plus four retained matching states in the new visit. These are
checkpoint segments; the fresh full route and remaining campaign checks stay open.

Recent verified milestones, which do not yet complete another campaign visit:

- Checkmate has separate fresh Blue-led watched-scene and Red-led skipped-scene
  internet passes, a natural team wipe and a guest reconnect pass. Other recovery
  cases remain open. Six native GPU comparisons verify both skins' weapon effects;
  real clients also receive both players' weapon effect types.
- Funhouse's return corridor, steps 35-42, passes 16 personal arrivals and 1,175
  matching player/world/native states, including Red's exact ten-second revival
  at 25 health and each player's own essence pickups.
- Funhouse machinery, steps 46-88, passes 50 personal arrivals, 11 paired holds,
  seven switch shots and 3,633 matching states. Steps 89-94 have separate scoped
  arrival, hold and pickup evidence.

Latest milestone: Funhouse steps **95-103** now have 12 verified personal arrivals,
five personal essence pickups and 1,139 matching player/world/native network states.
Both finish at 100 health, without deaths. The test uses ordinary counterbalancing,
actual native brush-support checks and visibility-aware resident enemy checks.

Latest upper-exit milestone: both players now pass steps **103-109** on source
`88b98f0`, with 12 personal arrivals, two 200-tick holds, two own pickups and
719 matching player/world/native states. A separate resumed **107-117** run
passes 22 personal arrivals, six own pickups and 1,524 matching states, without
new deaths. These are separate checkpoint segments, not a fresh full route.

Source `94488e1`, verifies the narrow descent **111-116**, including
12 personal arrivals, one actual forward waiting move, three own pickups and
1,010 matching states. Red dies naturally and revives after exactly 1,200 active
ticks at 25 health, with safe placement, preserved resources and verified camera
handoff/return. The co-op recovery logger now uses the actual slope-aware placement
rules; its old flat-ground test falsely rejected supported sloped revivals.

The waypoint-117 stall is resolved in the test controller: it now finishes a
nearby clear, supported approach within the existing walking height and arrival
bounds. A separate **116-121** prefix has six personal arrivals, three safe
waiting handoffs, one own pickup and 422 matching states, ending at 100/65 health.

Source `c8f333c` then verifies **121-125**: ten personal arrivals, five safe waiting
handoffs, two own pickups and **1,033 matching states**, ending at 90/30 health
with no deaths. The dodge controller now predicts network delay and braking;
the earlier version could walk the guest off the ledge before taking damage.

A further **121-125** replay passes with **1,079 matching states**, ten personal
arrivals, five safe waiting handoffs and two own pickups. Both finish at 70 health
without deaths. Normal checkpoints before 125 and 126 are retained, allowing
the next crossing test to resume without repeating the earlier fights.

The narrow bank and pendulums now pass as a coordinated **126-134** sequence on
source `c79ac85`: **14 personal arrivals**, all six individual pendulum landings,
both bank jump holds and **1,041 matching states**. Red crosses first and waits
off the landing route before Blue follows. Both finish at 70 health without
deaths; a normal checkpoint before 135 is retained. The final jump waits for a
reachable departure window, including platform movement during network delay.

The next **135-138** segment now passes on the same native source: six personal
corridor arrivals, three supported waiting handoffs, Red's watched Tweedles scene
and safe shared reunion, with **1,092 matching states**. Both observe all nine
scene lines; Blue remains hidden and frozen, then both return at 70 health with
their personal resources intact. A normal checkpoint before 139 is retained.
The entrance step is verified by its scene trigger and reunion: the scene closes
the door before either player can reach the old marker inside it.

A later resumed attempt independently verifies the **139-143 prefix**: ten
personal arrivals, five supported waiting handoffs and **505 matching states**,
ending at 60/70 health without deaths. It then wipes at 144: Blue dies while
waiting below the climb, and Red dies before Blue's revival. A normal checkpoint
before 144 is retained; the complete 139-146 run failed.

The climb now passes as a coordinated **144-146** sequence: six personal
arrivals, two observed following footings, one safe final waiting handoff and
**203 matching states**. Both move and jump normally, with the follower keeping
space while the leader brakes. Neither dies; final health is about 6.8/59.5.

A subsequent **147-150** replay passes both actual pad jumps, four personal
bridge arrivals, the paired 900-tick hold and **479 matching states**. Jump
durations are measured after the game acknowledges each actual jump, excluding
command-delivery delay. Each player collects their own arena essence: each gets
25 health and Will, with no shared consumption or partner healing. Both survive
at about 31.8/38.1 health. A normal checkpoint before 151 is retained.

Next are the opposite pad jump and descent from **151**, the remaining boss
fight, later scenes and exit. Fresh full traversal and the opposite crossing
order remain unverified. These segments add no completed campaign row.
Overall stays **48.1%**.

Tests run as two independent clients through the public network path on Anode,
with injected delay and jitter. Routine diagnosis uses state and log assertions.
Screenshots are not needed for each run. The current source also has 875 passing
active Rust tests and native Funhouse battle, machinery and scene checks; those
component results do not prove a full campaign route.

See the [fixed weighting and detailed history](COOP.md#delivery-and-verification-weighting),
[campaign matrix](COOP_CAMPAIGN.md) and [Funhouse evidence](FUNHOUSE.md). Future
updates should state the overall percentage, the new bounded milestone, and the
next unresolved acceptance check even when the percentage does not change.
