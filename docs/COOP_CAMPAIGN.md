# Co-op campaign verification matrix

The user now prioritizes [100% playable coverage](COOP_PLAYABLE.md), with
exhaustive polishing afterward. This historical matrix retains detailed
verification gaps; those combinations do not all gate the playable build.

First Skool Daze now has a verified four-segment normal save chain, including
all six scenes, both lifts, personal Mallets and actual School Two arrival.
Unchanged checkpoint hashes join 15,852 matching network states. The Queen
normal paired ending state is also accepted. Normal paired coverage is
**39/39 visits**, with **no normal routes remaining**. Playable readiness is
**100%**, including the public two-client ending; the historical full matrix
remains a separate polishing record at **81.5%**.
Hollow Hideaway now passes its normal paired route on `5fd9df4`: 33,897 ticks,
283 exact replicas, both large leaf rides, the Bill reunion and actual arrival.
Red also revives at 25 health after exactly ten seconds. Pool of Tears now
passes on `dcdc1d2`: 67,887 ticks, 566 replicas, four reunions and actual onward
arrival. Three ordinary deaths each revive after exactly ten active seconds.
School Two now passes its normal main quest on `b925e5c`: 53,696 ticks,
448 exact replicas, six reunions, both personal rewards and actual return arrival.
Both players finish at 100 health. See `COOP_PLAYABLE.md`.
Just Desserts now passes its normal paired pickup, boss fight, rescue and temple
arrival on `a4f954c`: 12,502 ticks, 105 exact replicas, both alive at 100/25.
Majestic Maze also passes its complete normal paired escort, held gate and
actual onward arrival on `845bb52` (33,270 ticks, both alive at 98/100 health). Rendered finale film/credits
handoff now passes on `cddd820`, with 3,445 matching public network states and
both clients presenting their credits. The broader historical statuses
below also include deferred polishing cases.

Airborne Terror core update: `3e96892` passes 41 paired component cases
and an unchanged ten-case solo baseline. The machinery batch `11d47f8` adds
paired component evidence for hedge2, hedge3 and tower3. Current status totals:
six combined and thirty-three partial after the finale batch
`c8a041e`. See [finale evidence](COOP_QUEEN.md). All normal paired routes are now accepted for the playable milestone.

Source: `campaign::route()` in `src/campaign.rs`. Return visits are separate requirements.
The village, Pandemonium, both Fortress visits, Beyond and return school have the complete combined co-op evidence below.
Existing single-player route tests do not supply evidence for two-player play.
Record build identities and reproducible evidence before changing a row to passed.

The first five passed rows retain the build-specific evidence recorded below. Player contact
was added afterward; earlier full traversals must be repeated with separate
waiting/landing positions before release. The focused contact/effect checks
in `COOP.md` do not substitute for those campaign repeats.
The return-school row was verified with the contact system enabled.

Core Water Logged update, 6 November 2026: `d06ded3` adds shared machinery,
personal current waits/fan hits and the original paired introduction. Sixty-six
native cases pass, including both ordinary swimming orders, all six player/
flusher activations and twelve Stage scene combinations. The solo 16-case log
exactly matches `291af36`. At that checkpoint Water Logged became Partial; totals were six
combined, twenty-five partial and eight not run. Full two-client combat and
traversal, rendered effects and recovery/rejoin remain open. Follow-up `0e5f061`
adds full fan-sweep placement exclusion and nine native checkpoint/revival
cases (75 paired cases total); full per-visit recovery remains unverified.

Core progression update, 6 November 2026: `ccd6cd1` adds Jabberwock paired
scenes, personal Eye Staff reward, shared drawbridge and departure. Twenty-one
world/reward cases and thirty-four Stage/scene cases pass, including both native
post-battle bridge/gate/exit walking orders. All 58 combat cases also pass; four
solo check/route logs exactly match `7c3ca24`. No full two-client combat route or
per-encounter recovery matrix is claimed. Totals at that checkpoint were 6 combined, 24 partial,
9 not run.

Core combat update, 6 November 2026: `7c3ca24` adds shared Jabberwock combat
and individual healing in `jlair2` and `grounds1`. Fifty-eight staged native
cases pass, including both target/collector slots and saved continuation. Both
solo check/route logs exactly match `c6f580e`. Victory scenes, Eye Staff,
bridge/exit and two-client route/recovery remain open. Both rows are now Partial:
totals at that checkpoint were six combined, twenty-four partial and nine not run.

Core integration update, 6 November 2026: `c6f580e` adds Majestic Maze's shared
plate/gate clock, atomic rider movement, either-player escort triggers and real
NPC occupancy. Twelve map component cases and seven production-cast cases pass;
the ordinary solo route and eleven saved checkpoints/reports exactly match the
previous build. Maze is now Partial, leaving six combined, twenty-two partial
and eleven not run. This supplies no full two-client route/recovery evidence.

Core recovery update, 5 November 2026: unchanged build `c5cb7fc` passes both
public countdown reconnects and exact ten-second, 25-health revivals beside a
survivor on Funhouse's moving fulcrum. Neutral revived bodies remain supported
and are carried with the brush. Both cameras, resources, safe separation and
426 matching player/world/native-enemy states pass. See `COOP_PROGRESS.md` for
the retained final case and failed fixture preparation. This recovery component
adds no completed campaign visit; the consolidated core suite remains next.

On 27 October 2026, source `685a05d` repeats all 39 entrance checks plus four
underwater recovery cases, all 82 shared-world checks, five Fortress adapter
checks and 15 Beyond adapter checks successfully. Beyond's old Rage test assumed
walking through the first collector; it now has that player walk aside using
normal input. Logs are `private/coop-wchess1-campaign-685a05d.log`,
`private/coop-wchess1-shared-world-685a05d.log` and
`private/coop-wchess1-{fortress,beyond}-685a05d.log`. These are native
component/entrance repeats; the full independent-client campaign repeats above
remain open. The same build's Pale Realm internet run completes steps 0–70,
then stalls at a ledge on step 71; it does not complete a campaign row.

A later repeat on source `8499731` passes the same **39 entrances plus four
underwater recovery cases, 82 shared-world cases, five Fortress cases and 15
Beyond cases**, including the newer landing-contact behavior. Logs are
`private/coop-broad-{campaign,world,fortress,beyond}-8499731.log`. A fresh
Blue-led Pale Realm internet run passes 0–29, then exposes a test-driver lift
handoff stall; its 5,281 matching states and 1,570 host / 1,556 guest presence
observations pass. These results add no full-campaign credit.

Source `b945e0e` fixes first-arrival ownership when a shared trigger becomes
eligible after both players have entered it. Waiting order survives saves and
is removed when a player leaves, dies or relocates. All 82 shared-world and
39 entrance/four underwater-recovery cases pass again, as do 117 Pale Realm
world/scene/puzzle cases and 832 active Rust tests. The captured Red-led Knight
approach/puzzle/exit segment passes through independent internet clients with
677 matching states. The strict owner check remains intact. These component
and segment results do not replace the full-route matrix below.

Source `59dc7e0` repairs contact at a stair support boundary: the final retreat
may return to the actual previous supported position after full clearance and
support checks. The captured native approach passes in both slot orders, all
834 active Rust tests pass, and the 119 Pale Realm, 82 shared-world and 39
entrance/four underwater-recovery checks pass again. Public-client 129–145
segments pass with either leader (668/890 matching states). The preceding fresh
run reached the next level but failed its overlap assertion; it remains failed.
Fresh full repeats and the remaining campaign matrix are still required.

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
| 6 | `skool1` | `Initial` | Partial: Red-first public traversal through five scenes, both paired lifts, book crossing and recipe; fresh guest rejoin, personal Mallets and coherent full school snapshots; nearest-visible guard damage in both roles, separate healing, both combat death/spectator/respawn roles and saved-checkpoint wipe pass. Red library-to-exit and Blue theatre-to-exit checkpoint segments pass, including the explicit upper gap jump, airborne reunion and lower shelf. Four hash-connected normal public-invite segments now establish first-school entrance-to-exit playability (15,852 matching network states, 309 full reports, both alive at 12/77 health). This is historical save-chain evidence, not an uninterrupted latest-build repeat; broader arrival-order permutations remain polishing work. |
| 7 | `skool2` | `Initial` | Partial under the historical exhaustive matrix. Normal main-quest paired route now passes on `b925e5c`: 53,696 ticks, 448 exact replicas, six reunions, both potion/star rewards and actual `skool1_start2` arrival at 100/100 health. Includes the fixed gym reunion softlock. Existing native/public component and checkpoint evidence is retained; broader owner/recovery permutations are polishing. See `COOP_PLAYABLE.md`. |
| 8 | `skool1` | `skool1_start2` | Passed on `eb3b335` (combined evidence in COOP.md): fresh entrance-to-Pool public routes in both arrival orders with delay/jitter, both shared lift waits/rides, original scene owners/reunions, trailing partner and fresh guest rejoin; both real death orders, moving spectator cameras, exact ten-second/25-health respawns and shared saved-checkpoint restoration; nearer-visible guard selection/damage in both roles; separate entitlement/healing from the same enemy reward and matching full player/world states. Contact enabled; starts with the normal return-visit loadout. |
| 9 | `potears1` | `Initial` | Partial: shared leaf boarding/carry, rock damage and ant-targeting components pass. Public two-client opening and checkpoint segments reach bank 3 (waypoint 16), including the high-bank jumps, both rock scenes with Red owning them, safe reunions and a paired escape from the falling rock, with matching player/world states and both alive. The latest complete attempt fails at waypoint 17. Full two-client traversal/recovery remains open (see COOP.md). |
| 10 | `potears2` | `Initial` | Partial: 33 native cases pass for seven paired pad paths, both leaf boarding orders with either death, atomic blocked departure/save/release, nearest-visible guards and isolated damage, both personal fish deaths with exact ten-second/25-health recovery, and moving personal pickups. Single-player transport and both complete route endings pass. Normal paired Arena progression is now accepted on `5fd9df4`, including both large leaf rides, real ten-second revival, the Bill scene/reunion and actual Just Desserts arrival (33,897 ticks, 283 exact replicas). The pair jumps normally from lily four to the shore. Full per-visit network/recovery permutations remain polishing work (see COOP_PLAYABLE.md). |
| 11 | `potears3` | `Initial` | Partial: 22 native paired boss/scene/healing cases and six production-Arena hit/grab/particle cases pass on source `3353e534f00e815151090698ffb4f1722314d2fe2d003fee1d15a4ea7fcd7354`; single-player full route passes. Normal paired Arena route now passes on `a4f954c`: 12,502 ticks, 105 exact replicas, actual temple arrival and both alive at 100/25. Broader two-client/recovery permutations remain polishing. See `COOP_PLAYABLE.md`. |
| 12 | `utemple` | `Initial` | Partial: full public-internet traversal passes in both first-arrival orders on `0c9d40b5907d22224b65cc68959853ddbf0318139320848635709e972ba2ff5c`, with 40 ms delay/20 ms jitter, correct exit ownership, a frozen partner, personal shells/air, clear living bodies and both clients reaching the garden without respawns or traversal warps. Blue-led health 40/40 with 3,050 matching player/world/native-enemy states; Red-led health 58/50 with 3,032 matches. Both real drowning/spectator/1,200-tick/25-health revival roles, a saved team wipe and entrance rejoin pass separately, with 1,712 matching states. All four native paired routes, 13 component cases and both single-player routes pass. Focused exit-rejoin segments now pass with both leaders from an exact captured gameplay checkpoint (189/210 matching states), followed by the authored Garden exit. These are segment checks; later full-route repeats exposed intermittent trap deaths in the test swimmer. Native WaterSnark target switching/isolated damage and each player collecting the same placed metaessence now pass in a focused public segment (206 matching states). The turtle is a free-swimming guide; no shared passenger ride exists. On `c5cb7fc`, the Blue-led public recovery route also passes guest restart during BOTH underwater death roles, exact air/body/world transfer, revival and shared wipe (1,563 matching states). The Red-led countdown attempt wipes near the first column before reconnecting; it is failed route evidence. Route repeatability around lethal traps remains open; see `COOP.md`. |
| 13 | `garden1` | `Initial` | Partial: 21 native component cases and historical 4/4 paired routes into Garden2. After adding native-rope player contact, both watched routes pass but both skipped variants fail (late combat/fall); the historical 4/4 result does not validate that change. A captured two-rope public segment passes with 115 spacing checks and 156 matching player/world/native states. Captured fall recovery also verifies exact ten-second/25-health revival beside a survivor on a native rope. Custom controlled ropes, full independent two-client traversal and the remaining recovery/rejoin matrix stay open; see `COOP_ENCOUNTER_TESTS.md` and the current Garden results in `COOP.md`. On `c5cb7fc`, an unchanged captured rope segment passes public guest restarts during BOTH death countdowns, with both exact ten-second/25-health safe rope revivals, rendered cameras and 290 matching states. The survivor uses ordinary climbing and attacks against live rifle fire; this is not a full-route repeat. |
| 14 | `garden2` | `Initial` | Partial: explicit paired fulcrum and owned Ant/Ladybug adapter; 46 staged native cases pass on source `c6a4ddc5632c8f9dec60303122e6629c2ce5eaa0cfa982d231d55420349e32c9`, covering both riders/dead partners, shared clocks, targeting/personal impacts, original watched/skipped scenes with either owner and safe reunion, impact-effect snapshot restoration and the captured bank overlap in both player slots. The identical diagnostic course clears that overlap but still loses Blue near the thermal. Earlier single-player component checks pass. Full co-op traversal, internet clients, rendered guest effects and death/wipe/rejoin remain open; see `GARDEN2.md`. |
| 15 | `garden3` | `Initial` | Normal paired route accepted on `7710f92`: shared gate double-toggle fixed; fresh production-Arena entrance through actual Icy Reception arrival, 10,699 ticks, 89 exact replicas, original watched reunion, both mushroom launches, 86/86 health, no deaths/wipes and exact onward personal resources. Eight real-map saved trigger/order cases and unchanged solo toggle pass, alongside the original 11 shared-clock/contact/scene components. Historical exhaustive status remains Partial: per-visit internet, rendered presentation and full recovery/rejoin permutations remain polishing work. See `GARDEN3.md`. |
| 16 | `garden4` | `Initial` | Normal paired route accepted on `cd89026`: fresh production-Arena entrance through actual Fungiferous Flora arrival, 21,406 ticks, 35 exact replicas, both personal Ice Wands, both vent rises per player and two watched original scene reunions. Blue revives after 1,200 active ticks plus 841 paused cutscene ticks; both arrive alive with exact personal resources, no wipe/cheats/staging. Existing 44 shared-clock/hazard/scene components remain applicable. Normal/Easy solo routes and 947 active Rust tests pass in the initial batch. Historical exhaustive status remains Partial: broader per-visit internet, rendered presentation and recovery/rejoin permutations remain polishing. See `GARDEN4.md`. |
| 17 | `centipede1` | `Initial` | Normal paired route accepted on `906f8c7`: 12,471 ticks, 104 exact replicas, one watched ambush reunion and actual Sanctum arrival with both alive, no deaths/wipes/staging. Personal resources carry exactly apart from the existing second-Die arrival grant to Red; temporary health floors clear. Existing 31 staged component cases remain applicable. Broader internet, presentation and recovery permutations remain polishing; see `CENTIPEDE1.md`. |
| 18 | `centipede2` | `Initial` | Partial: paired boss targeting/attacks/grabs, personal healing and slide protection, and shared defeat/spike/mushroom progression. 28 staged native cases pass on source `993d722cbcdd1ad904d6ff66269e3a5fbf5ee54c245e816959bedebe3ea94c9b`, covering both victim/scene-owner roles, watched/skipped intro with a living/dead partner, saved grabs and scene continuation, individual arrivals and the original mushroom exit. Ten production Arena cases additionally pass on `ead0be326db7169fb9ad48c815e924ba96c3118f98e653ba30949f1636efb7e0`: both personal grab/control and native hit roles, actual acid-death/spectator/1,200-tick revival, paired checkpoint wipe, and 30 exact guest snapshots with 8,287 rendered particle batches per side and no drops. All four native entrance-to-exit paired routes pass on `0edf86a` after fixing simultaneous pull-up overlap, with either leader and watched/skipped intro; both survive and reach the next visit (the follower falls during the later climb and rejoins through the first-arrival scene/level handoff). All 28 component and ten Arena cases repeat successfully on that source. Source `bb4d83b` repeats all four native routes and passes a full Blue-first public-internet route with 40/20 ms delay/jitter, a normal post-boss save and fresh guest rejoin, correct scene ownership, both alive at the next visit, and 1,874 matching player/world/native states. Red-first internet traversal still fails in the delayed climb controller; separate-client combat/recovery, broader effects and independent completion of both climbing routes remain open. See `CENTIPEDE2.md`. |
| 19 | `wforest` | `Initial` | Normal paired route accepted on `841dcd6`: 33,869 ticks, 283 exact replicas, four watched scene reunions, shared Staff component and actual Pale Realm arrival with both alive, no deaths/wipes/staging. Personal resources carry exactly. Existing 62 paired component cases across both visits remain applicable. Broader per-visit internet, rendered presentation and recovery matrices remain polishing; see `WFOREST.md`. |
| 20 | `wchess1` | `Initial` | Normal paired Blue-led watched route accepted on `841dcd6`: 68,146 ticks, 113 exact replicas, both chess puzzles, shared lift, timed spikes, both levers and all five original scenes through actual Castling arrival. Both alive at 92/100 health with exact personal resources, no deaths/wipes/overlap/unscripted warps. Existing paired components and separate core internet evidence remain applicable; broader per-visit presentation/recovery permutations remain polishing. See `WCHESS1.md`. |
| 21 | `wchess2` | `Initial` | Partial: 48 native cast cases and 45 staged world/scene cases pass on source `ac3a080c59aa00cd300a918fed0c38ebe78e51019b9b12637c59a67c3bd5aa1f`, covering personal targeting/attacks, shared doors, portal boarding/carry, original scenes and saved moving exits. Source `a80ef8fa922d13e923c7135463d0fb12ec48c78725834f11c9eebe26d32e8db7` now passes the continuous 203-leg native Arena route in all four first-player/scene combinations, both reunions, 214 exact guest-world comparisons and resource-preserving Checkmate arrival. All 834 active units pass on that source. The preceding component repeat also passes 82 shared-world, 43 campaign entrance/recovery and 119 Pale Realm cases, with three complete single-player logs identical to baseline. Source `e54bc16` additionally passes all 203 legs in both Blue-led and Red-led public internet runs with 40/20 ms delay/jitter, both original scenes, Checkmate arrivals at 75/100 and 100/100 health, and 8,749/7,680 matching player/world/native states. Source `c1a202c` also passes exact normal-checkpoint restore, graceful fresh-guest rejoin and portal exit from both saved leader roles, with 55/49 matching states and 35/29 after rejoin; all 837 active units pass. Real combat revival now passes for both victims on that source (1,200 ticks, 25 health, moving spectator camera and 302/367 matching states). The shared wipe exposed a pre-death sloped-wall overlap. Source `62fe393` fixes it and passes the unchanged wipe (164 matching states), both captured native contact slots, 838 active units and the broad 82/43/119 world/entrance/Pale Realm checks; three single-player logs match baseline. The post-fix Blue full route passes with 8,793 matching states; the Red repeat passes 0�127 and both scenes but stalls at leg 128 (9,378 matching states and no observed overlap). Source `592728d` adds owner-only held Enter/A scene skipping and passes 839 active units plus native scene/broad checks; all four independent internet skip cases pass (both scenes and owners), with 83/65/112/87 matching states, partner rejection and preserved personal resources. Source `cc2388e` adds an ordered reset after host suspension: 843 active units and the broad native checks pass, plus a real forced-pause internet replay through legs 124�129 (145 matching states, both 100 health, 36.68 ms median settled input backlog). The fresh full Red-led route on the same `cc2388e` build now passes all 203 legs, both Red-owned scenes and Checkmate arrival at 100/100 health, with 6,519 matching states and passing presence checks. Broader rendered effects and remaining campaign gates stay open; no completed-row credit. See `WCHESS2.md`. |
| 22 | `rchess1` | `Initial` | Partial: source `e11e75c` passes 94 native Checkmate records, 82 shared-world cases and 43 campaign entrance/recovery cases; complete single-player component and route logs still match baseline. A fresh Blue-led watched-scenes internet run passes from the entrance through the King into Funhouse with 5,765 matching player/world/native states and a verified 1,200-tick, 25-health Red revival. Blue traversed the northern steps Red missed while dead; those are explicitly revival-carried coverage, not personal traversal by Red. Grounded co-op recoil fixes the King rising out of attack range under sustained paired hits. The separate natural combat wipe passes with exact saved players, restored native enemies/progression and 229 matching states. A fresh guest reconnect from the completed first scene continues through the King into Funhouse with 3,410 matching post-rejoin states and an exact ten-second Red revival. Both clients receive all three weapon effect types from both players. Source `afe78ef` now also passes the fresh Red-led skipped-scenes full route through Funhouse, with 5,777 matching states and all three Red-owned scenes. Red's exact 1,200-tick, 25-health revival is verified; Blue traverses northern steps 38-44 while she is dead. Six native GPU weapon-effect positive/negative comparisons pass for both skins and repeated restores. Other recovery/reconnect cases and broader native King/environment effect visibility remain open. See `RCHESS1.md`. |
| 23 | `funhouse` | `Initial` | Partial: source `b897af2` passes a fresh public-internet opening through steps 0-28, with both personal arrivals at all 22 movement waypoints, watched intro/reunion, no deaths and 2,734 matching player/world/native states. Normal checkpoints before 7/28/29 are retained. Native battle63/world35/scene37 records repeat successfully and single-player component gameplay is unchanged. The earlier full single-player route still fails at step 142. Source `b703a7a` separately verifies the resumed mirror steps 29-32: both personal down/up jumps, an actual Blade shot, both timed holds and 615 matching states. The longer 29-42 attempt remains failed at step 35 after enemy knockback blocked the approach. Revised waiting controls pass steps 33-35 from the normal mirror checkpoint, but that replay fails when Red dies at 36; no corridor pass is claimed. A later recovery-enabled replay reaches step 37 and verifies Red's exact 1,200-tick, 25-health revival, preserved inventory and spectator/return cameras. It then wipes; the exact host restore matches both saved bodies/resources, but post-wipe client continuation remains unverified. Source `c946266` adds opt-in ordinary melee/led-throw/defensive movement to the automated controller; 865 active Rust tests, native battle63/world35/scene37 and Checkmate94 pass, with identical single-player component gameplay. That initial paired replay still wipes at step 37. After adding ordinary collection of each player's live dropped essence, a resumed 35-42 run passes all 16 personal arrivals, 1,175 matching player/world/native states and Red's exact ten-second, 25-health revival with resource/placement/camera verification. The normal checkpoint before 43 is retained. The first machinery attempt has scoped 43-50 arrival/hold proof but fails cell4 at 51. A subsequent resumed 46-88 run passes 50 personal arrivals, 11 paired timed holds, all seven requested cell-switch shots and 3,633 matching states without deaths; final health is 100/90 and a normal checkpoint before 89 is retained. Cell3 uses two verified ordinary walks to native safe neighboring shooting footings. The next attempt has scoped 89-94 arrival/hold/pickup proof, then stalls at 95 with both players alive on a tilted fulcrum. A later queued run independently verifies all four personal arrival/jump receipts at 95-96; Blue also traverses 97-98 and survives waiting on the bank. Red stalls at 97 about 178 units below the required landing. Both remain alive at 81/100 health; the full connected crossing and 89-117 segment remain failed. Source `0d43ade` subsequently verifies 95-103 with 12 personal arrivals, five own essence pickups and 1,139 matching states; both finish at 100 health without deaths. Ordinary counterbalancing handles the third seesaw, native brush-support diagnostics validate tilted arrivals, and the corrected clear check includes visible resident Boojums and remaining attacks. The run stops at 104 when the rear target has no support after both riders move back; a normal checkpoint before 103 is retained. Source `88b98f0` then verifies both riders through 103-109: 12 personal arrivals, two timed holds, two own pickups and 719 matching states. A separate 107-117 replay passes 22 arrivals, six own pickups and 1,524 matching states without new deaths. Current source `94488e1` verifies 111-116 with 12 arrivals, one actual forward waiting move, three own pickups, 1,010 matching states and Red's exact 1,200-tick/25-health revival with resources, slope support and camera proof. The 117 navigation stall is resolved: a separate `8a6352f` prefix verifies 116-121 with six arrivals, three safe waiting handoffs, one own pickup and 422 matching states at 100/65 health. Source `c8f333c` verifies 121-125 with ten arrivals, five safe waiting handoffs, two own pickups and 1,033 matching states, no deaths and final health 90/30. Delay-aware ordinary dodge prediction fixes the test controller stepping off the ledge. At 126 the first waiting player blocks the narrow bank; the rear player stops at the required 36-unit separation. That attempt left bank/pendulum coordination open. A further resumed 121-125 run passes ten arrivals, five safe waiting handoffs, two own pickups and 1,079 matching states, without deaths and with both at 70 health; normal checkpoints before 125 and 126 are retained. Source `c79ac85` now passes the queued 126-134 sequence: 14 personal arrivals, all six individual pendulum landings, both bank jump holds, one safe far-side waiting handoff and 1,041 matching states. Red crosses first and clears the narrow landing before Blue follows; both finish at 70 health without deaths. The final jump waits for ordinary range after forecasting network-delayed platform carry. A normal checkpoint before 135 is retained. The next 135-138 segment passes six personal corridor arrivals, three safe waiting handoffs and Red's watched Tweedles scene/reunion, with all nine lines observed by both peers and 1,092 matching states. Both retain 70 health and personal resources; a normal checkpoint before 139 is retained. The entrance closes during the scene, so step 138 requires actual first-entrant ownership and safe reunion instead of fictitious arrivals inside the closed door. A further 139-143 prefix independently passes ten arrivals, five waiting handoffs and 505 matching states, with no deaths and final health 60/70. That longer attempt wipes at 144 while the players take turns on the climb; the full 139-146 run remains failed, and a normal checkpoint before 144 is retained. The subsequent coordinated 144-146 climb passes six personal arrivals, ordinary jumps and observed follower spacing with 203 matching states and no deaths, ending at 6.8/59.5 health. A subsequent 147-150 replay passes both actual pad jumps, four personal arrivals, the paired 900-tick hold and 479 matching states. Jump duration starts after acknowledgment of the actual jump. Both collect their own 25-health/Will arena essence without healing or consuming the partner's copy, and survive at 31.8/38.1 health; a normal checkpoint before 151 is retained. The resumed 151-155 descent then passes both actual second-pad jumps, eight personal arrivals and 396 matching states. Both collect their own second 25-health/Will pickup and survive at 56.8/63.1 health. A separate resumed boss run defeats both Tweedles, verifies Blue's watched eight-line Hatter scene with Red hidden/frozen and the original floor-collapse tail, then reaches Hatter1 with both players alive/grounded/clear/separated and personal inventory intact. It has 1,178 matching Funhouse states plus four retained matching next-visit states. Fresh full traversal, opposite order/ending ownership, skipped ending and remaining recovery remain open. The latest 875 active units and battle63/world46/scene37/Checkmate94 checks pass, with unchanged single-player component gameplay; 51 Funhouse guards pass. A later fresh `c79ac85` run independently verifies 0-28 with 44 arrivals, seven pickups and 2,763 matching states, then fails at the mirror. The new full-route runner has no full pass. Source `1f789e9` passes 876 active Rust tests plus the native component checks and fixes delayed held-movement prediction in the opt-in test controller; single-player component output remains unchanged. The actual guest mirror return jump remains failed and is not replaced by revival-at-bank evidence. These separate segments add no completed-visit or percentage credit. Fresh full paired traversal, the opposite crossing order, later shared transports/scenes/combat, remaining recovery and rendered environment effects remain open. See `FUNHOUSE.md`. |
| 24 | `hatter1` | `Initial` | Partial: staged paired machinery/scene checks and separate internet traversal segments through step 137. Both Gryphon owners pass. Scoped 128-135 adds 562 matching states at 100/100 health; resumed first portal 136-137 adds two personal crossings and 202 matching states at 80/100 health. Portal occupancy and repeated diagnostic jump regressions are fixed on source 04d3b0c. Resumed paired chair step 138 now passes on source 99a66d6 with two personal arrivals, two native jump departures to dry ground and 1828 matching states at 67/67.73 health. Separately audited step 139 adds two arrivals/departures and 1741 matching states at 54/43.53 health. Source 84c8eee adds diagnostic landing feedback and supported return targets. A subsequent Python fixture releases movement during the actual pull-up before jumping from the rim to the cushion. Resumed paired 140 now passes with two personal arrivals, one held Red pull, one rim hop, two dry return jumps and 3004 matching states at 42.93/6.53 health. The original failed attempts are retained. Resumed paired 141 now passes with two final personal arrivals/departures and 4229 matching states; Red revives after exactly 1200 active ticks at 25 health, then repeats her own approach. Resumed 142-143 adds two arrivals, one supported waiting handoff, the full wait and 426 matching states with no new death, ending at 26.93/23 health. Resumed 144-145 verifies both lowered-clock portal crossings and final lever approaches with 361 matching states. Resumed 146-149 verifies Red-owned watched Stop, shared reunion, original wait and Red-triggered shared About Face arrival with 1122 matching states, including 27 in the destination; health remains 26.93/23 with personal resources preserved. The exit is a shared transition, not two claimed personal waypoint arrivals. These checkpoint segments do not complete a full visit; fresh routes, remaining scene owners, other recovery and reconnects remain open. See [Clockwork evidence](HATTER1.md). |
| 25 | `hatter2` | `Initial` | Partial: 83 native cases pass. A fresh full two-client internet route now passes with 10 personal arrivals, shared lift, individual rewards, Red-owned watched Gryphon exit, actual paired jlair1 arrival and 6805 matching player/world/native states. Blue revives after exactly 1200 active ticks at 25 health with resources, placement and camera proof. A separate resumed Blue-owned ending has 2169 matching states. Both clients submit particles/effects with zero dropped batches; no pixel comparison is claimed. A separate exact Red revival adds 1087 matching states. A fresh guest process rejoins the unchanged victory checkpoint with personal resources and shared progress preserved: 77 matching states, 58 after rejoining. A fresh ordinary combat team wipe on source 116fc99 restores both players together at tick 7060 to the normal checkpoint at 3421, with exact native world/character/resource proof and 1058 matching states, 27 after restoration. Combat/transport/transition reconnects, live skipped/dead-partner endings and wider contact regressions remain open. See [evidence](HATTER2.md). |
| 26 | `jlair1` | `Initial` | Partial: source 56160ff passes 26 staged native cases for personal arrival/essence, shared platform boarding/carry/uneven-floor handoffs/obstruction and original arrival/Caterpillar scenes, including either Caterpillar owner, watched/skipped play, living/dead partners and compressed saves. Single-player components and full native combat route match the prior baseline; the cast-free step-54 failure is unchanged. A fresh two-client internet 0-3 segment passes with eight personal arrivals, watched entry, both early lava crossings and 2133 matching player/world/native states, without deaths; a normal checkpoint before 4 is retained. A separately bounded resumed 4-20 prefix has 34 personal arrivals and 5582 matching states, but its longer attempt fails guest planning at 21. Source ca4cc26 corrects the automated slope-support check; a separate resumed 21-23 run then passes six arrivals and 2224 matching states at 100/100 health without revival. A normal checkpoint before 24 is retained. Later bounded 24-25 evidence has four arrivals and 1563 matching states. After two failed combat attempts at 26, ordinary dodging and Cards-based combat waits allow a resumed 26-29 run to pass eight arrivals and 1374 matching states at 50/96 health without revival, retaining normal checkpoint 30. A further attempt verifies 30-31 separately (four arrivals, 352 matching states), then fails during the forward climb from 32 toward 33. A retry verifies 30-32 (six arrivals, 783 matching states) by a different path; climb reliability remains open. Observed dry waiting bays allow bounded 33-37 evidence (ten arrivals, 1309 matching states). A separate bounded 38 replay verifies two hot-waypoint jumps and two dry returns (two arrivals, 462 matching states), finishing at 93.75/41.67 health without revival. The longer attempt fails at 39 navigating through lava. These bounded prefixes do not pass their full attempts. Source 259d4f5 adds optional liquid avoidance and displacement-based obstruction detection; a separate resumed 39-42 internet segment then passes eight arrivals and 1335 matching states, ending at 100/56.67 health without death or revival and retaining normal checkpoint 43. Source 5c4a693 separates successive jumps with actual settled landings, and ordered forward approaches find bays beyond narrow slopes. Bounded 43-45 evidence has six arrivals and 869 matching states at 100/71.67 health without revival. That longer attempt fails at 46 after planning during an unfinished ledge pull; normal checkpoint 46 is retained. Source c7576ca waits for active ledge pulls to finish before planning; a separate 46-49 internet run passes eight arrivals and 1017 matching states at 100/46.67 health without death or revival, retaining normal checkpoint 50. A separate 50-52 segment passes six arrivals and 699 matching states with unchanged health and no revival, retaining normal checkpoint 53 beside the raised platform. A later c893e48 resumed public 65-76 segment passes all 24 personal arrivals and ten ascent-guide arrivals, with 7419 matching player/world/native states and 431 recovery rows. Both finish at 92/77.67 health without revival, retaining ordinary checkpoint 77. The formerly stalled paired point 65 is now verified. Separate evidence covers approach points 0-52 and 65-76; a fresh continuous approach repeat and full two-client internet traversal, rendered pickup delivery, death/wipe recovery and reconnects remain open. See [evidence](JLAIR1.md). |
| 27 | `jlair2` | `Initial` | Partial: explicit paired combat, personal healing/reward, scenes and exit integration. On `ccd6cd1`, 58 combat, 21 world/reward and 34 Stage/scene cases pass across both encounters, including both post-battle bridge/gate/exit walks. Both owners, watched/skipped/paused/restored scenes, dead partners, safe reunion and personal collections are covered. Four solo check/route logs exactly match `7c3ca24`. Full two-client combat routes, rendered effects and per-encounter rejoin/death/wipe acceptance remain open; see `JABBERWOCK.md`. |
| 28 | `wforest` | `wforest_start2` | Normal paired return route accepted on `d199822`: 9,720 ticks, 81 exact replicas, original wall-warning reunion, paid Staff wall destruction and actual maze arrival with both alive at 59/100 health. No deaths/wipes/staging; resources carry apart from the normal optional Blunderbuss arrival grant. Existing 62 paired components remain applicable. Optional reward collection and broader internet/presentation/recovery matrices remain polishing; see `WFOREST.md`. |
| 29 | `hedge1` | `Initial` | Partial: `c6f580e` passes 12 native paired plate/gate cases and seven production-cast cases, covering either player, shared carry/clock, obstruction, dead-body exclusion, pause/restore, child-owned Hold, real NPC contacts, both escort trigger actors, replica state and automatic exit retry. The full solo route and all eleven checkpoints/reports exactly match the baseline. Frozen `845bb52` now passes the normal paired Arena escort, both personal portals, real child-held gate and actual tower1 arrival: 33,270 ticks, 278 exact replicas, both alive at 98/100 health and exact resource carry, with no deaths or wipes. A blocked-portal cooldown retry bug is fixed and covered by the 950-test suite. Independent-client escort/recovery permutations remain separate polishing evidence; see HEDGE1.md. |
| 30 | `tower1` | `Initial` | Partial: `3e96892` adds the shared face-gust clock and living-owner Watch timing. All 41 native paired cases pass: clock parity, both players' trigger/push contacts, occupied repeats, saved continuation, original arrival variants and both actual exit-trigger owners. The ten-case solo log exactly matches `0e5f061`. Normal paired route `fb8a476` passes 17,513 ticks, 146 exact replicas, the watched arrival and upper updrafts into actual Hedge2 with both alive at 21/62 health and two exact ten-second revivals. Exhaustive recovery/rejoin and presentation checks remain polishing. See `TOWER1.md`. |
| 31 | `hedge2` | `Initial` | Normal paired route passes on `f9d8893`: both lever scenes, personal shells, five exact ten-second revivals and actual Water Logged arrival, 53,709 ticks/448 replicas. Broader recovery, internet and presentation permutations remain partial. See [current progress](COOP_PROGRESS.md#mystifying-madness-playable-route-completed). |
| 32 | `tower2` | `Initial` | Partial: source `d06ded3` adds shared water/machinery, atomic living-rider carry, personal current waits/fan hits and original scene integration. All 66 native paired cases pass, including both ordinary swimming orders with saved continuation, 12 Stage introduction variants and six physical flusher activations through native contacts. The original 16-case solo log is byte-identical to `291af36`. Follow-up `0e5f061` adds full fan-sweep placement exclusion and nine native checkpoint/timed-revival cases (75 paired cases total). Normal paired route `b278fdc` passes all three flushers, fan passages and actual Hedge3 arrival: 28,247 ticks, 236 exact replicas, three exact ten-second revivals and both alive at 55 health. Broader per-visit internet, presentation and recovery permutations remain polishing; see `TOWER2.md`. |
| 33 | `hedge3` | `Initial` | Partial: paired machinery/personal-contact components and existing solo component check pass on `11d47f8`. Full paired route remains open. See [machinery batch](COOP_MACHINERY.md). |
| 34 | `tower3` | `Initial` | Partial in the broader matrix; normal paired route passes on `fcf4979` through actual Royal Rage arrival, with 301 exact replicas, both alive and five ten-second revivals. Per-visit network/presentation permutations remain polishing. See [playable evidence](COOP_PLAYABLE.md). |
| 35 | `grounds1` | `Initial` | Partial: explicit paired combat, personal healing/reward, scenes and exit integration. On `ccd6cd1`, 58 combat, 21 world/reward and 34 Stage/scene cases pass across both encounters, including both post-battle bridge/gate/exit walks. Both owners, watched/skipped/paused/restored scenes, dead partners, safe reunion and personal collections are covered. Four solo check/route logs exactly match `7c3ca24`. Full two-client combat routes, rendered effects and per-encounter rejoin/death/wipe acceptance remain open; see `JABBERWOCK.md`. |
| 36 | `grounds2` | `Initial` | Partial: focused paired progression/combat and existing solo components pass on `073ed9f`. Full paired route remains open. See [castle batch](COOP_CASTLE.md). |
| 37 | `facade` | `Initial` | Partial: focused paired progression/combat and existing solo components pass on `073ed9f`. Full paired route remains open. See [castle batch](COOP_CASTLE.md). |
| 38 | `keep` | `Initial` | Partial: focused paired progression/combat and existing solo components pass on `073ed9f`. Full paired route remains open. See [castle batch](COOP_CASTLE.md). |
| 39 | `qlair` | `Initial` | Normal paired combat/ending route passes on `ef316ac`: both forms, original scenes, real second-arena checkpoint and exact restore, 291 exact replicas and both alive at the durable ending. Rendered film/credits and broader per-visit recovery/rejoin permutations remain open. See [Queen evidence](COOP_QUEEN.md). |
