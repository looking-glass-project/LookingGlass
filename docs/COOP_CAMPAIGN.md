# Co-op campaign verification matrix

Source: `campaign::route()` in `src/campaign.rs`. Return visits are separate requirements.
The village, Pandemonium, both Fortress visits, Beyond and return school have the complete combined co-op evidence below.
Existing single-player route tests do not supply evidence for two-player play.
Record build identities and reproducible evidence before changing a row to passed.

The first five passed rows retain the build-specific evidence recorded below. Player contact
was added afterward; earlier full traversals must be repeated with separate
waiting/landing positions before release. The focused contact/effect checks
in `COOP.md` do not substitute for those campaign repeats.
The return-school row was verified with the contact system enabled.

Each visit requires both host/guest roles and both first-arrival orders, coherent enemies
and individual consumables, single-player cutscenes with safe reunion, shared-transport
boarding, a real gameplay death followed by timed spectating/respawn, a simultaneous wipe
and checkpoint reload, a trailing guest, and disconnect/rejoin near a transition.
Where a feature is absent, record why it is not applicable instead of silently omitting it.
The finale must verify completion for both players in place of an outgoing transition.

Repeat traversal and combat with latency/jitter. Keep separate single-player/save
regression evidence. The leaves, swimming, moving supports and scripted rides need
rendered two-process playtests as well as headless checks.

Entrance-only evidence: `--coop-campaign-check` passes all 39 visits for paired
placement, two seconds of arrival physics, shared-world snapshot restoration and
entrance wipe placement. Both underwater death roles also pass at the temple and
first garden entrance. This is preparation for the traversal checks below, not
completion of a campaign row. Evidence: `private/coop-progression-entrances4.log`.

Persistence-only evidence: the public two-process fixture
`private/coop/travel-test-0abfe07a7db0413db758682defa42cd9/` passes travel/rejoin,
save/close/restart, exact shared checkpoint/team-wipe restoration and backup
recovery with distinct player resources. It uses staged school transitions;
school quest-gated exits and complete level traversal remain unverified by that fixture.

Village partial traversal: `--coop-village-check` now reaches the authored exit
from the unarmed entrance with both native bodies and both first-arrival orders.
It verifies both real Blade pickups, all six scenes, six safe cinematic reunions,
continued movement during ordinary speech and no deaths/wipes. Evidence:
`private/coop/village-route-4/coop-village/`, build
`0eb413f6731105a82780dc10d716c793fc04b3d7c3931d11391cbf6e9de9c2ee`.
The former eight-reunion result also treated two voice-only lines as cutscenes;
that behavior has been corrected to match single-player exploration.
That result is a single-process simulation. A subsequent public two-process
Red-leading traversal also passes with 40 ms delay and 20 ms jitter per sender:
six correctly owned scenes/reunions, both personal Blades, 50 moving-speech sample
pairs in each client, 84 identical matching authoritative ticks, and both players
entering Pandemonium at 100 Sanity. Both final views were inspected and the clients
and helper exited cleanly. Evidence:
`private/coop/village-test-eb20704fbc3547b1ab7bf766699c54a8/`, build
`5cae1ad45d01ad0b9a6f04a8f9a209c9bbff3ff804697cde94dbb3fefe2f5114`.
The Blue-leading network repeat also passes on that build and those settings in
`private/coop/village-test-064657842c4d466cb6e551e24be6bb98/`: both finish at 100
Sanity, all six owners/reunions and personal Blades pass, 81 matching authoritative
ticks agree, and both clients observe movement during ordinary speech without a warp.
Both real fall/death roles and the shared saved-checkpoint wipe now pass in a
separate public run with the same delay/jitter. Each player spectates the moving
survivor for exactly 1,200 ticks, revives at 25 Sanity with their Blade preserved,
and regains their own camera. A subsequent team wipe restores the actual earlier
automatic checkpoint's resources, placements and scene progress. Both restored
views were inspected. Evidence:
`private/coop/village-test-db59178229354bdbb2011361cf51ca9d/`, build
`f6304f3344c2f6d0340da41879fb00c4c7d37ada0076df301d49ae75a49f331d`.
The village has ambient moving machinery and a shootable hatch, but no activated
shared passenger lift or ride (`Village::event` / `Village::advance_players`);
the shared-transport boarding requirement is not applicable to this visit.
Fresh-process guest rejoin followed by the authored exit now passes on the same
build in `private/coop/village-test-49531ea8bd3f460da591f2be6a8c4465/` (Blue leads,
40 ms delay + 20 ms jitter). Both personal Blades and completed scenes survive;
all six owners/reunions and 63 matching authoritative player-state samples pass.
Both enter Pandemonium alive (Blue 100 / Red 92 Sanity), without a respawn or wipe.
Both final views were inspected and all owned processes/helpers closed.
The remaining enemy/consumable requirements now pass on build
`fe1f5fd905c8262e13dc3bae6e86dfef7a57752504c84d2575e52c1d87bd7a0b`, in
`private/coop/village-combat-44b591c1f47d453aa0afd3b9514fb7df/` (public internet,
40 ms delay + 20 ms jitter per sender). Its sole live guard selects nearer Blue,
then nearer Red, with both visible. Each checked hit removes 12 Sanity from the
selected player and leaves the distant partner unchanged. After defeating it
with ordinary Blade attacks, both follow the normal ledges to the placed health
shard: Blue heals 88 to 100 while Red stays at 88 and retains her pickup; Red then
heals to 100 without changing Blue's resources. Both clients observe separate
collection. All 3,986 comparable sent/restored player, lives, checkpoint and
encounter states match; 146 coincident full local reports also agree. Both attack
views, the guest's still-available pickup, and the final healed view were inspected.
Both clients/helpers close normally and the invite is removed. Combined with the
earlier full-route/recovery/rejoin evidence, this completes the village row. The
latest runtime changes only add opt-in test observations; they change neither
single-player behavior nor the previously verified scene/recovery mechanics.

Pandemonium partial traversal: the normal-start native two-body route now reaches
the authored Fortress exit with either player first. Both runs verify all four
scene owners, three safe reunions, personal Cards and no deaths/wipes; both finish
at 60.934 Sanity in 33,993 ticks. The Gnome return scene's original trigger now
starts its existing timeline. Shared-rope hand attachments and reunion captures
were inspected; staged grab/climb/release and minecart boarding checks also pass.
Evidence: `private/coop/pandemonium-route6/coop-pandemonium/`,
`private/coop-pandemonium-route6.log`, `private/coop-pandemonium-rope-native8.log`,
build `a9b50ed49c886e859e6a0a9cbcaec0d833fd08d799a4e1f74527fa0626da844b`.
Watched/skipped single-player routes, cinematic pause/save/skip, machinery state,
school/village world regressions and 736 Rust tests pass. This is still a
single-process route; the later separate-client evidence follows below.

The Blue-first public internet repeat passes with 40 ms delay and 20 ms jitter per
sender on build `ec4ccf71f5a634f1c8c03ccbc6924fa3492e9fac1421dd813cd4937c06118a48`.
Both players use the same entrance rope, the cart waits for its partner, all four
Blue-owned scenes and three reunions pass, both collect/retain personal Cards,
and 207 matching authoritative samples agree. Both arrive in Fortress alive
(Blue 27.134 / Red 21.135 Sanity) without respawns or checkpoint resets. Rope,
landing, boarding-wait and initial Fortress cinematic views were inspected; both
processes exit cleanly. Evidence:
`private/coop/pandemonium-test-25726e42ca974a459cf70cd356606c0e/`.
This caught and fixed a delayed grab that otherwise released an automatic rope
grip. Protocol revision 4 preserves grab/release intent, with bounds/watchdog and
actual-physics regression coverage; all 740 Rust tests pass.
The Red-first public repeat also passes on that build/settings in
`private/coop/pandemonium-test-a2cf6558bee64719af5fda4834594ca6/`, running in Anode:
four Red-owned scenes, three reunions, delayed boarding, both personal Cards and
89 matching authoritative ticks. Both arrive alive (Blue 39.867 / Red 23.468),
without respawns or checkpoint resets. Rope, boarding-wait and initial Fortress
cinematic views were inspected; both clients and the job exit normally.
Both entrance-side hazard deaths now pass over the same public connection and
delay/jitter settings on that build, in
`private/coop/pandemonium-recovery-1127245225934d65ac70d7f9af173184/`.
Red's death/respawn ticks are 6,054/7,254 and Blue's are 8,962/10,162: both clients
agree on exactly 1,200 ticks, moving spectator cameras, 25-Sanity recovery 48 units
from the survivor, retained inventory and the survivor's unchanged health. The
team wipe at tick 11,170 restores the automatic checkpoint from tick 1, with full
saved resources, initial placements and earlier quest/scene flags. Both first
restored samples have zero horizontal displacement from the saved placements.
Spectator/checkpoint views were inspected and both clients exit normally. This
fixture ends at the restored checkpoint.
Fresh-process guest rejoin near the airship followed by the authored exit also
passes on the same build/settings in
`private/coop/pandemonium-test-b5965aedc3274cba8065c2c111d4421f/` (Blue leads).
Both personal resources and completed cart/key/scene flags survive; all four
owners, three reunions and 126 matching authoritative ticks pass. Both retain
Cards and arrive alive (Blue 45.734 / Red 15.668 Sanity), with no respawn or wipe.
The initial Fortress cinematic views were inspected; both processes exit normally.
Both clients' retained histories also verify individual consumable rewards in the
Blue-first rejoin run and the Red-first route above. `tools/check_coop_rewards.py`
requires separate collection and healing, unchanged partner resources, and an
uncollected partner copy remaining in that player's loot ledger. Both clients
observe the same reward IDs. In the Blue-first run, the first guard's drop heals
Blue from 93.667 to 100, then Red from 84.533 to 99.533; collecting either copy
does not change the other player's resources. The later ranged guard's drop also
passes. Evidence: `private/coop-pandemonium-personal-rewards1.json` and
`private/coop-pandemonium-personal-rewards-red1.json`, on the same `ec4ccf71` build.
The first club guard now also passes a focused public-internet encounter check
on build `fe1f5fd905c8262e13dc3bae6e86dfef7a57752504c84d2575e52c1d87bd7a0b`,
in `private/coop/pandemonium-combat-ac557b2cbaa240d4a61b04df68220399/`.
Both Alices are visible; the guard selects and damages nearer Red, then nearer
Blue after their ordinary movement swaps positions. The distant partner's health
stays unchanged during each checked hit. After the Blade defeats the guard,
Blue heals 57.733 to 72.733 while Red retains her copy and resources; Red then
heals 87.667 to 100 without changing Blue's resources. Both clients independently
observe these grants. All 2,438 comparable sent/restored player, lives, checkpoint
and encounter states match; six coincident full local reports match as well.
Both target views and the guest pickup view were inspected, and both clients exit
normally. The normal entrance rope/cart are included, but later guards and the
exit are outside this focused fixture. The later full-route repeats below complete
the encounter comparison and the Pandemonium row.

Both full public routes now pass with the new encounter/player comparison on the
same `fe1f5fd9` build, again with 40 ms delay and 20 ms jitter:

- Blue leads: `private/coop/pandemonium-test-94511445d6bc41e9a4e92a1a50ec4f18/`.
  All four scenes belong to Blue, with reunions at ticks 2,701/14,409/34,878.
  A fresh guest rejoins near the exit (ticks 38,251/38,334), retaining both players'
  resources and completed scenes. All 5,997 comparable sent/restored states and
  31 coincident local reports match. Both reach Fortress alive, at 46.667/25.134 Sanity.
- Red leads: `private/coop/pandemonium-test-c3449c00e8bb41d0b6c7f39bb4729a98/`.
  All four scenes belong to Red, with reunions at 2,756/14,621/34,994. All 6,362
  comparable sent/restored states and 65 coincident local reports match. Both reach
  Fortress alive, at 30.001/45.201 Sanity.

Both retain personal Cards and independently collected healing rewards. The full
encounter records include all seven authored guards, six normal combat deaths,
and projectile attacks from both encountered Diamond guards. The distant return
guard stays alive, as on the ordinary route. Neither run uses respawns or wipes.
Rope, boarding-wait and initial Fortress cinematic views were inspected; those
arrival views do not verify the Fortress visit. Both client pairs and helpers
exit normally, and both invites are removed. Coverage summaries are retained as
`encounter-coverage.json` inside each fixture.

The first Fortress visit now has complete combined evidence (details and failed
development attempts are in `docs/COOP.md`): native paired split-room carry and
single-player route/cinematic regressions on `bac55928`; Blue-leading full public
route with fresh guest rejoin (`fortress-test-cc6abe54e07e4f2fac11ff3621b1c75d`)
and Red-leading full route (`fortress-test-3cf5044c3af64e3e97ed3ef09d70f751`)
on `fda92609`; focused Boojum targeting/personal healing
(`fortress-combat-e4a4260346de427c84f93e82e338f187`) and both combat-death
roles plus shared checkpoint restore (`fortress-recovery-6388fe74d6414e87a48f45ee92ceda44`)
on `2271c7db`. All public runs use 40 ms delay + 20 ms jitter and compare actual
sent/restored player/enemy state. The newer builds add opt-in diagnostics and
ordinary-input test controls; gameplay is unchanged from the paired-support build.
Its entry scene starts automatically with Blue as owner even when Red leads the
subsequent route. The split room is an environmental trap; passenger boarding is
not applicable. The later return visit and full Beyond the Wall traversal remain unverified.

Beyond the Wall now has native adapter evidence on build
`c17d9c2b1eedbd621fd56923648b8f38ec4a58bfd1ba1bae5ef39aac4c0e53c8`
(`private/coop-beyond-adapter-check5.log`). Staged checks on the actual map cover
paired Rage-lift carry at 30/60/144 Hz, either rider blocking both proposed carries,
rolling-bridge boarding, the later elevator's saved boarding wait without freezing
other machinery, survivor-only use, and either player's Rage pickup/scene/reunion
and musical-lever operation. Each player retains an individual Rage pickup.
These are focused fixtures, not entrance-to-exit routes or rendered/network proof.
The subsequent sound fixture on build `0944f214` adds public two-process evidence
for Red's musical demonstration, both reset-lever operators, identical numbered
notes, actual mixer recordings and a fresh guest rejoin without old-note replay
(`private/coop/beyond-audio-1dbe4744e4984a3394b41ce156a4aead/`). It compares 675
sent/restored states with 40 ms delay plus 20 ms jitter per sender. The start is
staged outside the musical trigger. Complete traversal/combat, death/wipe and
rejoin-to-exit remain open, so the completed count stays three of 39.
The final `10aa2112` build also passes the same musical fixture with Blue first
(`private/coop/beyond-audio-d97405d188c34927aa4e24529f05f29f/`) and a separate
unskipped six-line village opening with both clients' actual audio output
(`private/coop/village-voice-0e430b62bac8415a8ecf762320d21940/`). These focused
checks add sound evidence without granting another full campaign row.

Normal-entrance network traversal now reaches arch step 11 on build
`496972c89d1bfb2d2f9b3518f8f90e929ebb6e8ef62e09df31fb95650c5d57da`
in `private/coop/beyond-route-aa5a24b4065b48d7bb126d15146b0f9b/`.
Both independent clients cross the flipping corridor, solve the musical puzzle
with alternating operators, cross all nine rolling-walkway stages and take the
two authored portals using ordinary inputs. All three scenes have Blue as owner;
both players reach the arches at 100 Sanity, with no respawns. There are 7,143
matching actual sent/restored states under 40 ms delay plus 20 ms jitter.
The run stops on the next stair command: its test controller required less than
90 units of vertical separation between steps that remain 102.4 units apart.
This is partial traversal evidence, not an exit, rejoin or recovery pass.
The focused delayed public stair probe subsequently passes on build
`7eec625b6f796e7878748a243126ca79df900c889b7cc00a9f4e61e02f915424`:
`private/coop/beyond-arches-probe-0387775c95634254acd868903865bf8f/`.
Both players climb the moving steps, make the running jump to the upper door
and take the authored portal, with 1,027 matching sent/restored states and both
rendered upper-door views inspected. Its initial placement is staged outside
the rising-room trigger, so it adds no normal-entrance/full-row credit.
On that same build, a focused public corridor probe passes all flipping-platform
gaps and the authored portal in
`private/coop/beyond-corridor-probe-f26639b408ba470b82d9b3a6498ea8e7/`, with
1,709 matching states and both rendered 100-Sanity portal views inspected.
Its initial placement is staged before the corridor trigger. Subsequent full
repeats have not reached the exit: one stops at lever range, one at a Boojum
knockback on the spiral stairs, and one at a later corridor navigation gap.
The harness now uses precise lever approaches, broader combat waits and explicit
corridor jumps. The subsequent Blue-leading run reaches the actual Fortress
return in `private/coop/beyond-route-3ea9b2df6c034cffb7c1b83a3df0ccc9/` on
`7eec625b`: all three Blue-owned scenes, frozen partners, safe reunions, puzzle,
walkway, final door and 7,722 matching sent/restored states pass without respawns.
The original harness fails after successful travel because its reconnect check
still expects Beyond state. The retained route passes the same extracted route
validator (`audited-route-result.json`); the separate post-transition reconnect
audit also passes. The fixture now reconnects before the outgoing trigger.

Combat and individual earned healing pass on that build under 40/20 ms public
delay/jitter in `private/coop/beyond-combat-04edc0990c244e69995a04acf6a9d34f/`.
The guard selects nearer visible Red and then Blue, damages only each target,
and both collect separate healing copies. All 505 compared sent/restored states
and nine coincident full enemy/player reports match; target/pickup views were
inspected. Both real combat deaths and the shared earlier-checkpoint wipe pass
in `private/coop/beyond-recovery-6ca7a13ffb634602bc613cff7d893097/`: each role
spectates for exactly 1,200 ticks and returns at 25 Sanity, 48 units from the
survivor, with inventory preserved. The wipe restores personal resources,
placement, enemies and machinery. All 1,943 compared sent/restored states and
89 coincident full enemy/player reports match; recovery views were inspected.
The corrected Red-leading full repeat passes on the same build/settings in
`private/coop/beyond-route-26dba8d47c88465abb0ab7cc4a1eb6bd/`: all three Red-owned
scenes, safe reunions, no respawns, 7,885 matching sent/restored states and a
fresh guest reconnect before the exit. Both arrive at `fortress1_start2` at
100 Sanity. Guest stair/upper-door/return views were inspected, including live
Anode observation. Blue's obscured return capture exposed a missing camera
collider on the visible return portal. Build `cf86b004` corrects that camera
collision; the focused two-client repeat `fortress-return-camera-579dd23381c84f05aa0c6937d36aec74`
has clear stationary entrance views. Both native body positions also pass at
30/60/144 Hz, alongside the original Fortress cinematic/save regressions.

Focused public transport checks pass Rage boarding, ownership, a single scene,
paired 48-unit descent and individual power copies in both orders on `7eec625b`
(`beyond-rage-ffb2d9ac54324e4ca8b926673385d316`, Red; `beyond-rage-fdfee62536404a0d97d6c2633a33f0e6`, Blue).
They walk from the entrance and compare 956/965 sent/restored states. Their
middle-scene captures exposed a hidden owner body; `cf86b004` now draws its
authoritative Rage pose instead of hiding every gameplay body during a scene.
Rolling-bridge waiting/boarding passes under 40/20 ms public delay/jitter with
Red first (`beyond-bridge-5cd7763586a34c51919d81c6feec49c6`, `7eec625b`, 156 states)
and Blue first (`beyond-bridge-07f7c0ef71a64c2aa9a796aeade41fbb`, `cf86b004`, 170).
Those bridge probes stage the initial approach; both waiting/boarding views
were inspected. Final `cf86b004` Rage repeats pass with Red first in
`beyond-rage-7d58e582d71842dfaae8f083f4998a27` (936 matching states) and Blue first
in `beyond-rage-ca85ba3508fb4818846294edf437d0a2` (960). Both clients' middle-scene
views were inspected in each run: the owning Alice is visible with the correct
clothing and transformation pose, and the partner is hidden. The final build
passes 745 Rust tests, debug/release builds, seven source-package tests and
the Fortress cinematic plus native Fortress/Beyond co-op regressions.
Cycling-elevator public checks now pass with both first riders on `cf86b004`,
under the same 40/20 ms delay/jitter: `beyond-elevator-d57b738905834aa6a1ab2d1c730c3c59`
(Red, 811 matching states) and `beyond-elevator-da651fab20844150a026e2a640f1ebc2`
(Blue, 775). These staged upper-return approaches use the authored activation
trigger, then ordinary descent, boarding, riding and running dismount jumps.
They verify boarding wait, paired rise, dismount wait and empty return clock;
waiting/dismount/final views were inspected. Normal's actual outer landing is
used; the short side bridges exist only on Easy.

A further late-Rage pickup probe exposed a phase-reset freeze on `cf86b004`
(`beyond-late-rage-e16f3f79bbe141f387b6d401f0030397`). `ef3db23e` now preserves a
cycle already started before the pickup, using the existing saved timestamps.
Both public repeats pass: `beyond-late-rage-510a52db30ac4a858ec444cd09c10e84` (Red,
932 matching states) and `beyond-late-rage-0dedcf3a803a4016babffde8b517a11c` (Blue,
927). Scene-owner and paired reunion views were inspected; the original scene
advances, the hidden partner stays still and both resume supported on the lift.
Native watch/skip/pause/save/restore cases pass for both owners, alongside
original Rage/legacy/difficulty and Beyond world/cinematic regressions, all 746
Rust tests (one ignored), debug/release builds and seven source-package tests.
Logs and full identities are in `COOP.md`. Combined with the full traversal,
combat, resource, death/wipe and reconnect evidence above, Beyond's row passes.

Fortress return traversal passes on `ef3db23e` with Red first in
`private/coop/fortress-return-0e72ab5bd10a471fbcf6fe4dc06c87fc/` (1,047 matching
sent/restored states) and Blue first in
`private/coop/fortress-return-a66cad4ce4344e68b5ae5169cf15ad47/` (1,064).
Both use the authored return entrance, public 40/20 ms delay/jitter and ordinary
movement/combat/jump inputs. The reveal has the correct first owner, a frozen
partner and one safe reunion; the ambush activates. A second guest process
rejoins before the school window with resources and scene state preserved.
Both reach `skool1_start1` at 100 Sanity without death or checkpoint reset.
Reunion/window/school entrance views were inspected. The initial scene captures
are the opening white fade. The route now also captures the actual reveal after
the fade, verified in both clients and both trigger orders on the same build:
`private/coop/fortress-return-53bf1343634e43e19a655a6ffd592774/` (Red, 1,042 matching
states) and `private/coop/fortress-return-e744240e838d43b6b1776eea5d0c2ede/` (Blue,
1,036). Both original Boojum cutaway views, without an extra Alice, and both final
school views were inspected. Both repeats include fresh guest rejoin and end
at 100 Sanity without recovery. Reproduce with
`tools/test_coop_fortress_return.py --data <base> --first red --rejoin` (also Blue).

Focused return combat passes in
`private/coop/fortress-return-combat-86bb88db6fc24b00b5320f8c0f494b9c/` on that build
and public 40/20 ms delay/jitter. With both players visible, `end_spawn4` targets
nearer Red, then nearer Blue; each checked seven-Sanity hit leaves the distant
partner unchanged. After ordinary Blade combat, Blue's earned shard heals 72 to
97 while Red stays at 79 with her copy available. Red's copy then heals 79 to 100
while Blue stays at 97. Both clients observe independent collections under the
return ledger `fortress1$return`. All 578 comparable sent/restored states and
eight coincident player/enemy reports agree; attack/healing views were inspected.
Reproduce with `tools/test_coop_fortress_return_combat.py --data <base>`.

Both actual guard-death roles and the saved-checkpoint wipe pass in
`private/coop/fortress-return-recovery-15c11dd1a47044339884ce41fd8b686f/` on that build
and connection settings. Red's death/respawn ticks are 3,029/4,229 and Blue's are
7,055/8,255: exactly 1,200 ticks, moving spectator cameras, 25 Sanity, safe nearby
placement (49.19/48.89 units), retained inventory and unchanged survivor health.
The wipe at tick 10,518 restores the real checkpoint from tick 1, including exact
personal resources, earlier scene/machinery and enemy state, and both starting
placements within ordinary movement bounds. All 1,538 comparable sent/restored
states and 31 coincident player/enemy reports agree. Both spectator, respawn and
checkpoint views were inspected. Reproduce with
`tools/test_coop_fortress_return_recovery.py --data <base>`.

Return shared boarding is N/A: the split-room trigger is first-visit-only in
`Fortress::gate`; the rocking school is ambient machinery, not an activated
passenger transport. All four new jobs exit normally, close their clients/helpers
and remove their invites. Combined evidence completes the fifth row; 34 remain
open. Only Python fixtures/documentation changed after the frozen game's existing
Rust, release and native/single-player verification; source-package/syntax/source
boundary checks cover these additions.

First school partial traversal: build
`4fd9c6cca856cc455099b23286834117ecb693144726781a33103c34bb4fab00`
adds opt-in school scene observations without changing gameplay. A Red-first,
public internet run with 40 ms delay and 20 ms jitter reaches the four-book
bridge, both personal Mallets, and both library lifts from the normal entrance.
Each lift waits for its absent partner for two observed school-clock seconds,
then carries both players to the upper landing. Waiting messages and completed
rides were visually inspected. The theatre, moving shelf, shelf reveal and
book-bridge reveal each have Red as owner, a frozen partner and a safe reunion.
Both middle shelf/book reveal views were inspected; no extra Alice is present.
All 14,137 comparable sent/restored states and 117 coincident full school,
encounter and player reports agree. Neither player dies or resets the checkpoint.
Evidence: `private/coop/school-route-1bb1e30f303f4edc9abdeb31367d7952/`, including
`partial-audit.json`. This run fails at the test driver's navigation search onto
the first bridge book; it does not verify the recipe, exit, or guest rejoin.
The route fixture now walks onto that first book before planning the remaining
gaps. Both complete arrival orders
remain required before the row can pass. Existing 746 Rust tests (one ignored),
debug/release builds, native co-op world and school state/lift checks pass on this
build. No additional campaign completion credit is assigned yet.

A subsequent Red-first optimized run crosses all four books and reaches the
recipe scene, then rejoins a fresh guest with positions, personal resources and
school progress retained (ticks 66,317/66,401). Five Red-owned scenes, five safe
reunions, 10,233 matching sent/restored states and 294 coincident full school
reports pass. Evidence: `private/coop/school-route-44d72453baad4b1690465f308c26a03a/`.
Its recipe middle view shows the original single Alice and Cheshire Cat; the
authored cinematic Alice retains the original blue costume even with Red owning
the scene. The full route still fails: the guest's timed descent overshoots the
64-unit shelf and falls from 384 to the bottom floor, dying with 44 Sanity. The
fixture now collects a fresh theatre guard reward and uses a slow, observed edge
descent to that shelf. The exit remains unverified by this run. A further route
attempt times out on the long planned jump to the Mallet platform. Splitting that
approach at the lower stair reaches the platform in the later focused probe
`private/coop/school-combat-da20a3a5b1ef498dae4720537d5c4567/`; its healing check
still fails because `mana_skool1` is return-only, as required by
`inventory::pickups_for_visit`. The fixture preserves that restriction and now
uses an actual fresh enemy reward after the theatre. The changed descent and
complete route still need a clean repeat.

Both school guard-death roles and the saved-checkpoint wipe now pass on the same
build and public delay/jitter settings, using the optimized executable in
`private/coop/school-recovery-84df4abea6c8431fbd0072b0a61ca189/`.
Red dies/respawns at ticks 3,969/5,169 and Blue at 9,270/10,470. Both clients
agree on exactly 1,200 ticks, cameras following the moving survivor, 25 Sanity,
48-unit separation, retained inventory and unchanged survivor health. A team
wipe at tick 12,727 restores the actual tick-1 checkpoint: exact personal
resources, school puzzle flags, machinery definitions and enemy health/state,
plus starting positions within normal movement bounds. All 1,930 comparable
sent/restored states and eight coincident full school/player/encounter reports
agree. Both spectator, respawn and checkpoint views were inspected. The fixture
closes both clients/helpers and removes the invite. Reproduce with
`tools/test_coop_school_recovery.py --data <base> --exe target/release/looking-glass.exe`.
Earlier recovery attempts are retained as failures: an edge position outside
firing range, an occluded balcony position, and a spectator movement target too
close to the preceding respawn. These were fixture corrections, with no changes
to combat damage, recovery timing or the saved world.

Focused school combat and individual healing now pass in
`private/coop/school-combat-2b97ba23cace4d9f8b9ee9202be946ea/` on the same build,
using the optimized executable and public 40/20 ms delay/jitter. With both
players visible, `cgd_top1` selects nearer Red, then nearer Blue; each checked
seven-Sanity hit leaves the distant partner unchanged. The fresh `play_guard1`
reward after the theatre heals Blue from 93 to 100 while Red stays at 58 and
retains her copy. Red then heals to 83 while Blue stays at 100. Both clients
observe independent collection and unchanged partner resources. All 1,662
comparable sent/restored states and 28 coincident full school/player/encounter
reports agree. Targeting and both healing views were inspected; both clients and
their helpers exit normally and the invite is removed. Reproduce with
`tools/test_coop_school_combat.py --data <base> --exe target/release/looking-glass.exe`.
The upper guard's reward expires before reaching it after the theatre, so this
fixture uses the fresh theatre-floor reward without changing item lifetimes.
Full clean traversal and both first-arrival orders remain open; 5/39 rows pass.

Checkpoint-based school diagnosis now uses an ordinary host-menu save from
`private/coop/school-route-29377935da334bbda58fa3d525d4be26/`: tick 30,631,
Blue/Red Sanity 96/90, SHA-256
`d231c8614579512f1a331be2d84a38c040a16c804159b61617050aba844adfad`.
The saved file was copied unchanged into fresh profiles and loaded through the
normal resume path. In `private/coop/school-resume-42df416b6dc243c18361b6a78e8bea4c/`,
both clients retain positions, personal resources and puzzle progress; both clear
the balcony gap and repeat the two lifts' waiting/ride checks. The balcony landing
was visually inspected. All 5,907 comparable sent/restored states and 58 coincident
full school reports match. That resumed run still fails on the second-to-third
book jump, where Red falls and dies; it gives no descent, exit or full-route credit.
Its `partial-audit.json` records the limited scope. The preceding full repeat and
three shorter balcony probes remain recorded as failures, not successful routes.

The opt-in local route controller now accepts a bounded analogue input speed cap
for the next book-jump repeat. This does not change player physics or network
authority. Build `bd57c4843ded26e3102c046a2f3f0c3e485a5006e11e4160623f6a033d08e285`
passes 746 Rust tests (one existing ignored test) and the optimized build. Its
new book sequence passes all three gaps for both players in
`private/coop/school-resume-4d348a4e85d942dd85073256e2b80164/`, with Blue/Red
Sanity unchanged at 82/76. The third and fourth book landings were visually
inspected. All 9,321 comparable network states and 270 coincident full school
reports match. This run exposes a game bug after the recipe scene: Red entered
while airborne, 46.67 units above the floor. The finished scene's 32-unit partner
placement search could not find that floor, but holding the scene also froze
Red's gravity, leaving both players stuck. The run times out and remains a
failure, with no descent, exit or full-route credit.

An unchanged host-menu checkpoint before the bridge is retained from that run:
tick 73,578, Blue/Red Sanity 82/76, SHA-256
`d271632b9cde459d80998e315bc03f22974ac2c8d3997b396e318576aeeb7324`.
It allows the airborne scene handoff and remaining descent to be repeated through
the normal resume path without editing either player's state.

Build `7f7637c10d8dd0cdea57cdfbe3369145fe3e96dbe39664a1adbb576379fd3c51`
fixes that handoff: when the scene owner is airborne, partner placement may search
up to 256 units down for a supported, unobstructed, non-hazardous landing. A
grounded owner's reunion and ordinary death respawns keep the 32-unit limit.
All 747 Rust tests pass (one existing ignored test), as do the optimized build
and native co-op world check, including the actual recipe platform with both
owners. The resumed bridge-to-exit internet run passes in
`private/coop/school-resume-e9785d16088944ac80929212f68b3227/`: Red's airborne
handoff releases at tick 79,695, the fresh guest rejoins at 80,462/80,543, both
reach the lower shelf and enter `skool2_start1` alive at 59/53 Sanity. Both scenes
belong to Red; the waiting partner stays frozen; no death, respawn or checkpoint
rollback occurs. The completed-history audit finds 2,134 matching network states
and 49 coincident full school reports. The reunion, lower landing, exit scene and
next-level view were inspected. This is a resumed segment, not a full-route pass.
The resume fixture now checks scene ownership, frozen partners, common reunion
ticks and absence of silent death/checkpoint recovery as well as coherence.

Two subsequent full repeats remain failures. In
`private/coop/school-route-c65dca262e5e4ef88007faf0691f0bfd/`, both players land
across the balcony, but the fixture misses the brief descending report and waits
for an already-past frame. It now also accepts a confirmed landing, retaining the
same destination and stability checks. The next repeat,
`private/coop/school-route-2bc554dfc699446f945cd393433e974c/`, passes that crossing
and the first paired lift but stops at the diagonal approach to the library stairs.
Blue remains against the corner while Red has arrived. Its revised waypoints go
around the west side of the bookcase before approaching the stairs. The
respective audits find 4,272/102 matching network/full-school states and
10,263/92, with no deaths or respawns. Neither run completes the level.

The intermediate straight approach also hits the bookcase in
`private/coop/school-resume-6d2f7fdb207f4777832d82a02da3e985/`; this repeat is
stopped with its partial evidence retained. It supplies an unchanged ordinary
host-menu library checkpoint, tick 61,982, Blue/Red Sanity 89/83, SHA-256
`f96e477ef6a290a7e354c55ea387c14bae02dad2ef4db1e3f1df9ffac05218bb`.
The optional `library` checkpoint/resume point allows short repeats from there.
The west-side approach then passes for both players in
`private/coop/school-resume-d56f4a7e33b2404a903b4f794b24ade2/` on build
`59cc04b2bad4238d013635b739de9807fdc6a092845181fac604e99f99504f7f`, along with
the upper climb/lift, all book jumps, airborne recipe reunion and fresh guest
rejoin at ticks 82,552/82,624. All 3,322 comparable network states and 76 full
school reports match. The lower descent still fails: the guest's first airborne
report has feet at 383.8, just above the fixture's old 382 threshold; the next
report is already at 219.3, too late to steer back onto the shelf. The fixture
now turns at the first loss of footing. The failed run remains partial evidence,
and gives no exit or full-route credit.

The corrected descent then passes from the unchanged tick-83,155 checkpoint
(`fade6224a03e45906fdc2dfd460ce6e8328a878e4bb8330727ff7083a2fad3f5`)
in both orders on that final build and the same public 40/20 ms delay/jitter:
Red leads in `private/coop/school-resume-d64d8527339c48a0b18eeccbbcf1fe5c/`
(467 matching network states, two full school reports), and Blue leads in
`private/coop/school-resume-e41a6b3885d64ff88b37ba7232837f11/`
(483 matching network states, three full school reports). Both players reach the
lower shelf and `skool2_start1` at 59/53 Sanity, without death, respawn or checkpoint
rollback. Each exit scene has the intended owner and a frozen partner. The lower
landings and next-level view were inspected. These are resumed descent passes;
the complete first-school route still needs both clean entrance-to-exit repeats.

The fresh Red-first repeat in
`private/coop/school-route-969188a6bd4d41d5897d9ec836916f5d/` passes the west
bookcase approach but fails on the next upper-library leg. Blue's test navigator
walks off the stair edge, falls to the lower floor and dies on a subsequent
attempt. The failure is retained; 8,464 matching network states and 90 full
school reports agree before it. The fixture now approaches a fixed stair lip
and uses an explicit ordinary running jump to the upper landing.

That revised route passes from the unchanged library checkpoint through the
exit in `private/coop/school-resume-cfcc10a29d594d7a87e2a72a5aec3f0f/`, using
build `59cc04b2bad4238d013635b739de9807fdc6a092845181fac604e99f99504f7f`, Red
first and public 40/20 ms delay/jitter. Both make the upper gap, paired second
lift, book crossing and lower shelf. The airborne recipe reunion and fresh
guest rejoin at ticks 83,135/83,210 retain progress and personal resources.
Both enter `skool2_start1` at 59/46 Sanity without death, respawn or checkpoint
rollback. All 3,622 matching network states and 174 full school reports agree;
all three remaining scene owners and frozen partners pass. The upper landing,
lower shelf and next-level views were inspected. Fixture SHA-256:
`54b0d7aff5f95c453e8e90d37ec2a6426b1cb534554b1f295c36547925b0c2e1`.
This is a library-to-exit segment, not either required full-route repeat.

The next Blue-first full repeat,
`private/coop/school-route-ee1cadfdb8dc4f09bdb732a11fe949c2/`, times out before
the Mallet: Red reaches the lower stair within the navigator's tolerance but
drifts off its side while stopping. Both remain alive at 97/95 Sanity; all
5,972 matching network states and 351 full school reports agree. This remains
a failed route. The fixture now attempts a centred ordinary walk down those
stairs, and an optional `theatre` menu checkpoint/resume point supports shorter
repeats from just before that descent.

The revised Mallet descent passes for both players in the next Blue-first full
run, `private/coop/school-route-88dc45f1a8a5499b8b7cd2ff6e6d19d4/`. Both
collect their Mallets and cross the balcony. The run still fails: the spiral
lift stops at motion time 6.841648, rider height 55.3747, before the 64-unit
landing. Both remain alive at 83/93 Sanity. All 8,807 matching network states
and 112 full school reports agree. Native lift checks already distinguish the
outer rim beneath the upper landing lip from clear rider positions; the next
fixture repeat tests walking inward if that obstruction holds the lift.
No full-route credit is awarded.

That run also supplies an unchanged ordinary host-menu checkpoint before the
Mallet stairs, tick 26,629, Blue/Red Sanity 97/100, in `theatre-checkpoint.json`.
Its SHA-256 is
`1034fb722534c35e4971ece22162b2853a664a0730082859ad8bfe0583cf0629`.
The successful menu save and return to play were inspected; checkpoint contents
are not edited by the repeat fixture.

Two theatre-checkpoint repeats, `school-resume-a618799f5256491e917e5b7d7004609d`
and `school-resume-09e4e7a02397448787c733e0032e2e0c`, reproduce a separate
landing defect after ordinary inward walking clears the outer lip. Red reaches
the static floor at height 64 while Blue remains on the lift at 61.2968;
the lift clock stops at 6.94998. Both remain alive at 83/93 Sanity. The latter
run has 4,276 matching network states and five coincident full school reports.
The downwards support probe incorrectly still counts the static-floor player
as riding the lower lift. The new regression fails on that old behavior;
support selection now gives the nearer walkable floor precedence. A separate
regression retains obstruction handling when a moving platform would actually
hit that player. These failed repeats receive no traversal credit.

Build `b21fc09969b22ae75d53b25a7d9a2a608feac17bd51411006971ee9dda7bce78`
fixes both lower-lift and equal-height support ambiguity. The native regression
starts at the exact observed stopped pose with both player orders: the landed
body stays fixed, the other reaches the top, and neither enters solid geometry.
`--coop-world-check` passes, including existing boarding, obstruction, village
mover and scene/checkpoint checks (`private/coop-landing2-world.log`). The full
single-player school route also passes in 41,593 ticks at 77 Sanity, with no
flight, warps or recovery (`private/coop-landing2-sp-route.log`). These native
results do not replace the outstanding full two-client traversals.
The release build and 750 Rust tests also pass, with one existing ignored test
(`private/coop-landing2-release.log`, `private/coop-landing2-tests.log`).

The next full Blue run, `school-route-862883d3dd5742c8919c21f5e9878be4`,
was stopped before reaching either lift. Red continued down the Mallet stair
after the test released walking, fell to the lower floor, and could not complete
the subsequent planned jump. Both remained at 100 Sanity; 4,733 network states
and 129 full school reports agree. Guest input was marked idle at tick 31,622,
but the host still showed the old walking velocity through tick 31,925. This
needs timing investigation and receives no traversal credit. The full-route
verifier now explicitly rejects any observed death or checkpoint restoration.

A separate small-packet public echo probe at 40/20 ms measures 156–188 ms round
trip (median 172 ms), in `private/coop/rtt-probe-cc2bce2ca4814751b30061a9a12cdca1`.
The ordinary two-process transport probe also passes in
`private/coop/network-test-626abcf67dad4732aced57c0b7dc6037`.
Build `f2ff00f9d7e31835bfc705351276cfba5f631d0657aa3bdd597ade6014ac6d08`
adds passive bounded input-acknowledgement and per-frame timing diagnostics to
measure this under actual school load. Its release build and 750 tests pass
(one existing ignored); the rendered timing repeat is still pending.

Second-school machinery preparation was first implemented on build
`59cc04b2bad4238d013635b739de9807fdc6a092845181fac604e99f99504f7f`.
The gym lever uses the original activation rules for either player and assigns
the original scene to its operator. Outside scenes, each extending bleacher step
checks both living bodies before committing its movement and rider positions.
`--gym-check` passes the actual-map lever/hold checks for both owners and, at
30/60/144 Hz, either blocking player, resuming after clearance, snapshot
restoration, both physical staircase climbs and two riders carried on an exposed
moving tread. Evidence: `private/coop-gym-native4.log`; optimized build:
`private/coop-gym-release4.log`. All 747 Rust tests pass on this final source
(one existing ignored test), in `private/coop-gym-tests2.log`.
The unchanged older build also fails the former
two-second single-player climb deadline; the check now waits up to four seconds
for the same 190-unit landing at the current ordinary walking speed.

The complete single-player second-school route passes with the same gameplay
changes on build `61df60e526e460e90cdc15b08337f7e84ce2cd585209d312de71a717e196f3cf`
(subsequent changes adjust only the native check): 53,236 ticks, nine jumps,
both portals, all ingredients/rewards and the return to `skool1_start2`, ending
at 27 Sanity without cheats/recovery/refills. Evidence:
`private/coop-gym-school2-route1.log`. The co-op world regression also passes on
that build in `private/coop-gym-world1.log`. These are focused machinery and
single-player checks. They did not yet cover paired pendulums or full rendered
second-school traversal.

The remaining school-two machinery now has an explicit paired adapter on build
`5c2a87b56cddb3c5d321ef3765fbbc5c76ce52550b1385b8bee0e7084fa2f5ec`.
Each moving group plans both bodies before committing any pose, clock or rider
position; a pendulum and its bound lamp stop together when either player blocks
them. The single-player wrapper retains the original one-owner scene step.
Native `--school2-check` checks pass for cabinet/books riders in both player
orders at 30/60/144 Hz, paused motion, snapshot continuation, either-player
pendulum obstruction/resume and the production adapter without a duplicate
quest-clock step. Existing school-two scene, potion and save checks also pass.
Evidence: `private/coop-school2-movers-native1.log`. All 747 Rust tests pass
(one existing ignored test), as do the optimized build, paired gym checks and
co-op world regression: `private/coop-school2-movers-tests1.log`,
`private/coop-school2-movers-release1.log`, `private/coop-school2-movers-gym1.log`
and `private/coop-school2-movers-world1.log`. Co-op quest/combat and full rendered
second-school traversal remain unfinished; row 7 receives no completion credit.
The complete single-player second-school route also passes on this exact build,
in `private/coop-school2-movers-route1.log`: 53,236 ticks, nine jumps, 23 throws,
149 cards, 170 combat damage and 27 final Sanity, with both authored portals,
all ingredients/rewards and the gated `skool1_start2` return. No cheats,
recovery or refills are used. This confirms the single-player regression check
for the paired school machinery as well as the earlier gym adapter.

The later second-school combat/quest adapter is implemented on build
`a3ce8a69e7cb0f9a27aacc788300bb67dc6c733fc0c70c5f1733512d937fe4be`.
It connects legacy enemy hits and per-player damage to the host arena, advances
the quest once per world tick, accepts either living player's story-item
contacts, and keeps the Gnome rescue guards' original pre-provocation target.
Staged native checks pass for both victims/collectors, concealed/dead target
changes, save continuation, individual resources, shared story rewards and
paused quest clocks. All 756 Rust tests, native School2 checks, release build
and co-op world regression pass. This is integration/component evidence only;
the full rendered/network School2 route and row 7 remain open. See
`private/coop-school2-combat-{native3,world,tests-final,release3}.log`.
The full single-player route also passes unchanged on this exact build:
53,236 ticks, nine jumps, 23 throws, 149 cards, 170 combat damage, 27 Sanity,
both original portals and the gated return, without cheats or recovery
(`private/coop-school2-combat-route.log`).

| # | Map | Entrance | Co-op evidence |
| ---: | --- | --- | --- |
| 1 | `gvillage` | `Initial` | Passed (combined evidence above): both arrival orders, all scenes/reunions, nearest-visible guard damage in both roles, individual health/Blade pickups, matching encounter/player states, both death/spectator/respawn roles, saved-checkpoint wipe, trailing partner and fresh guest rejoin-to-exit; shared boarding N/A |
| 2 | `pandemonium` | `Initial` | Passed (combined evidence above): native and delayed public routes in both orders, rope/cart, four scenes/three reunions, personal Cards/healing rewards, nearest-visible guard damage in both roles, full-route encounter/player comparison, both hazard death/respawn roles, saved-checkpoint wipe, trailing partner and fresh guest rejoin-to-exit |
| 3 | `fortress1` | `Initial` | Passed (combined evidence above): both delayed public route orders, original scene/reunion, independent portals, paired split supports, upper jumps and exit, fresh guest rejoin, nearest-visible Boojum selection/damage in both roles, separate healing, coherent enemies, both combat death/spectator/respawn roles and saved-checkpoint wipe; shared boarding N/A |
| 4 | `fortress2` | `Initial` | Passed (combined evidence above): full traversal in both orders (Blue audited, Red clean exit), fresh guest rejoin before exit; both Rage, bridge and cycling-elevator boarding orders; late Rage on the cycling lift; notes/reset/rejoin with audio; nearest-visible guard damage and separate healing; both combat death/spectator/respawn roles and saved-checkpoint wipe; clear return camera and visible Rage owner |
| 5 | `fortress1` | `fortress1_start2` | Passed (combined evidence above): both delayed public full traversals to school, correct reveal owners/frozen partners/reunions and inspected middle views, ambush activation, fresh guest rejoin before exit; nearest-visible guard damage in both roles, individual healing, both actual death/spectator/respawn roles and saved-checkpoint wipe; shared boarding N/A |
| 6 | `skool1` | `Initial` | Partial: Red-first public traversal through five scenes, both paired lifts, book crossing and recipe; fresh guest rejoin, personal Mallets and coherent full school snapshots; nearest-visible guard damage in both roles, separate healing, both combat death/spectator/respawn roles and saved-checkpoint wipe pass. Red library-to-exit and Blue theatre-to-exit checkpoint segments pass, including the explicit upper gap jump, airborne reunion and lower shelf. Both complete entrance-to-exit arrival orders remain open. |
| 7 | `skool2` | `Initial` | Open. Native/component checks cover gym machinery, targeting/damage, pickups and paused clocks. Normal Red-first Dice/gym milestones and both ordinary laboratory stair climbs have internet evidence. On `844ea95`, the recorded Red laboratory-entrance-to-exit segment passes corrected offline verification; a fresh Blue greenhouse-to-exit segment passes live, including both portal directions, scene ownership/reunion, guest rejoin, ordered ingredient consumption/rewards and the return to `skool1_start2`. These are checkpoint segments, not full visits. Both complete traversal orders, the remaining recovery matrix and any earlier contact-system repeats remain required. Detailed build-specific evidence is in COOP.md. |
| 8 | `skool1` | `skool1_start2` | Passed on `eb3b335` (combined evidence in COOP.md): fresh entrance-to-Pool public routes in both arrival orders with delay/jitter, both shared lift waits/rides, original scene owners/reunions, trailing partner and fresh guest rejoin; both real death orders, moving spectator cameras, exact ten-second/25-health respawns and shared saved-checkpoint restoration; nearer-visible guard selection/damage in both roles; separate entitlement/healing from the same enemy reward and matching full player/world states. Contact enabled; starts with the normal return-visit loadout. |
| 9 | `potears1` | `Initial` | Partial: shared leaf boarding/carry, rock damage and ant-targeting components pass. Public two-client opening and checkpoint segments reach bank 3 (waypoint 16), including the high-bank jumps, both rock scenes with Red owning them, safe reunions and a paired escape from the falling rock, with matching player/world states and both alive. The latest complete attempt fails at waypoint 17. Full two-client traversal/recovery remains open (see COOP.md). |
| 10 | `potears2` | `Initial` | Partial: 33 native cases pass for seven paired pad paths, both leaf boarding orders with either death, atomic blocked departure/save/release, nearest-visible guards and isolated damage, both personal fish deaths with exact ten-second/25-health recovery, and moving personal pickups. Single-player transport and both complete route endings pass. Full two-client traversal/combat/rewards/wipe/rejoin remains open (see COOP.md). |
| 11 | `potears3` | `Initial` | Partial: 22 native paired boss/scene/healing cases and six production-Arena hit/grab/particle cases pass on source `3353e534f00e815151090698ffb4f1722314d2fe2d003fee1d15a4ea7fcd7354`; single-player full route passes. Full two-client co-op route and recovery matrix remain open. See Duchess integration in `COOP.md`. |
| 12 | `utemple` | `Initial` | Partial: full public-internet traversal passes in both first-arrival orders on `0c9d40b5907d22224b65cc68959853ddbf0318139320848635709e972ba2ff5c`, with 40 ms delay/20 ms jitter, correct exit ownership, a frozen partner, personal shells/air, clear living bodies and both clients reaching the garden without respawns or traversal warps. Blue-led health 40/40 with 3,050 matching player/world/native-enemy states; Red-led health 58/50 with 3,032 matches. Both real drowning/spectator/1,200-tick/25-health revival roles, a saved team wipe and entrance rejoin pass separately, with 1,712 matching states. All four native paired routes, 13 component cases and both single-player routes pass. Focused exit-rejoin segments now pass with both leaders from an exact captured gameplay checkpoint (189/210 matching states), followed by the authored Garden exit. These are segment checks; later full-route repeats exposed intermittent trap deaths in the test swimmer. Native WaterSnark target switching/isolated damage and each player collecting the same placed metaessence now pass in a focused public segment (206 matching states). The turtle is a free-swimming guide; no shared passenger ride exists. Route repeatability around lethal traps remains open; see `COOP.md`. |
| 13 | `garden1` | `Initial` | Partial: 21 native component cases and historical 4/4 paired routes into Garden2. After adding native-rope player contact, both watched routes pass but both skipped variants fail (late combat/fall); the historical 4/4 result does not validate that change. A captured two-rope public segment passes with 115 spacing checks and 156 matching player/world/native states. Captured fall recovery also verifies exact ten-second/25-health revival beside a survivor on a native rope. Custom controlled ropes, full independent two-client traversal and the remaining recovery/rejoin matrix stay open; see `COOP_ENCOUNTER_TESTS.md` and the current Garden results in `COOP.md`. |
| 14 | `garden2` | `Initial` | Partial: explicit paired fulcrum and owned Ant/Ladybug adapter; 44 staged native cases pass on source `0601aeb39bc98a7828a398e3c59f6fc5ef637ab54ab5fe343994a079cc45f8a2`, covering both riders/dead partners, shared clocks, targeting/personal impacts, original watched/skipped scenes with either owner and safe reunion, plus impact-effect snapshot restoration. Existing single-player component checks pass. Full co-op traversal, internet clients, rendered guest effects and death/wipe/rejoin remain open; see `GARDEN2.md`. |
| 15 | `garden3` | `Initial` | Partial: explicit shared marble/gate/collapse update; 11 staged native cases pass on source `6df84313f78ae427eea47d108cf006607c5b1c5658e13fa4073e2d796dae8827`, including 30/60/144 Hz single-player clock parity, pause/save, all marble callbacks, both personal victim/dead-partner slots and watched/skipped arrival reunion. Single-player fast routes pass; older walking-pace checks fail identically before and after the adapter. Full co-op traversal, internet clients, rendered effects and recovery/rejoin remain open; see `GARDEN3.md`. |
| 16 | `garden4` | `Initial` | Not run |
| 17 | `centipede1` | `Initial` | Not run |
| 18 | `centipede2` | `Initial` | Not run |
| 19 | `wforest` | `Initial` | Not run |
| 20 | `wchess1` | `Initial` | Not run |
| 21 | `wchess2` | `Initial` | Not run |
| 22 | `rchess1` | `Initial` | Not run |
| 23 | `funhouse` | `Initial` | Not run |
| 24 | `hatter1` | `Initial` | Not run |
| 25 | `hatter2` | `Initial` | Not run |
| 26 | `jlair1` | `Initial` | Not run |
| 27 | `jlair2` | `Initial` | Not run |
| 28 | `wforest` | `wforest_start2` | Not run |
| 29 | `hedge1` | `Initial` | Not run |
| 30 | `tower1` | `Initial` | Not run |
| 31 | `hedge2` | `Initial` | Not run |
| 32 | `tower2` | `Initial` | Not run |
| 33 | `hedge3` | `Initial` | Not run |
| 34 | `tower3` | `Initial` | Not run |
| 35 | `grounds1` | `Initial` | Not run |
| 36 | `grounds2` | `Initial` | Not run |
| 37 | `facade` | `Initial` | Not run |
| 38 | `keep` | `Initial` | Not run |
| 39 | `qlair` | `Initial` | Not run |
