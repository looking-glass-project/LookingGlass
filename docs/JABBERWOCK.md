# Jabberwock: both major battles

## Co-op scenes, rewards and exit (2026-11-06)

Build `ccd6cd15eecf026f4d9694b5a69a2f9382f994c396c4278493b8401b8e3aaf5d`
adds the remaining paired progression hooks. The Lair finale uses the normal
trigger ordering after the survival timer opens its gate, retaining a waiting
first entrant. Royal Rage advances its original death/fall sequence once, then
binds the finale to the nearest living Alice. Both keep the original single
Alice camera/performance, hide and freeze the partner, and use the normal safe
reunion. Watched, skipped, paused and restored outcomes preserve personal stats.

The Lair Eye Staff and its local model/effect availability are personal. Each
living player collects once and receives the ordinary weapon pickup's own Will
refill. The first collection neither consumes nor refills the partner's copy.
The departure countdown starts after both collections; a dead partner who has
not collected can do so after revival. Failed departure retries without another
use press. Optional validated world state preserves older solo and paired saves.

Royal Rage's drawbridge is a crossing that lowers from vertical, not a passenger
lift. Its clock starts after the cinematic reunion. Living riders move together
only when both proposed positions and paths are clear; an obstruction stops
the brush without moving either rider. Dead bodies do not block or ride it.
Either living Alice can open the castle door. Shared epilogue clocks advance
once, and duplicate contacts do not restart them.

`--jlair2-coop-world-check` passes 21 supplied-map cases.
`--jlair2-coop-scene-check` passes 32 scene combinations plus two ordinary-input
post-battle walks across the bridge, through the gate and into the real exit
trigger, with either player leading. These fixtures stage battle prerequisites;
they do not bypass prerequisites during ordinary play. All 58 earlier combat
cases also pass. The two original solo checks and two solo routes produce
byte-identical logs to `7c3ca24`. Full two-client combat-to-exit traversal,
rendered effects and per-encounter death/wipe/rejoin acceptance remain open.

## Co-op combat and personal healing (2026-11-06)

The host now advances each boss and its waves once, retaining the original
120 Hz battle schedule and boss immunity/health rules. The boss chooses the
nearest visible living Alice; committed attacks retain their target until it
becomes ineligible. Melee, breath, beam and spiral collision use both living
bodies, and a spiral can damage the other player with its ordinary visible
splash without damaging its direct victim twice. Dive contact can hit both
overlapping bodies once. Each of the six Lair wave actors retains its target
slot across a save. A living player's Pocket Watch stops the same spirals and
waves as solo play; it does not freeze the boss or breath packets.

Healing is individual, including the current pickup location, eight-second
cooldown and local art visibility. Importing a solo save initializes both
players from that save's current cycle/cooldown. Collection cannot heal the
partner, consume their pickup or revive a dead player. Optional validated
co-op state preserves existing solo snapshots and compressed co-op restores.

On frozen source `7c3ca2407b8bd1378499a813103a913152e67a7a2292404622bf77bf1671a8b3`,
`--jlair2-coop-check` passes 58 staged cases using the supplied rigs across both
maps. It runs headlessly without screenshots. The two existing solo checks and
two production-input solo routes pass with byte-identical logs to `c6f580e`.
Private manifests retain executable/source hashes and per-check receipts.

That build was a combat/resource milestone; the follow-up above adds paired
victory scenes, Eye Staff reward, drawbridge and exit progression. No full paired route,
internet rejoin, encounter death/recovery or rendered-effects pass is claimed.

## Eye altar visibility correction (2026-10-01)

The placed Eye Staff altar emitted its purple plume during the introduction and
survival fight even though the encounter hid the reward model. The encounter now
suppresses that ambient copy by its BSP entity identity. Its own reward art is
the only altar effect, starting at the authored reveal and ending on collection.
The reward model and effect share the same visibility predicate. The second
Jabberwock encounter has no Eye altar and keeps its ordinary ambience.

The native check now includes world particles alongside encounter art. It proves
the old ungated plume, absence during intro/fight, reveal timing, collection and
restored visibility. All 27 weapon altar placements were audited: 24 ordinary
pickups follow availability/collection, and the three scripted Staff component
altars follow their owners. Hatter's blade and the forest Staff already had the
required controls; the forest return hides its former Staff altar. Complete boss
routes and separate save writer/reader checks also pass. Evidence and the frozen
release source are in `private/jabber-altar-fix/`.

## Fire direction correction (2026-10-01)

Ground and flying fire now launch along the animated mouth attachment's forward
axis. Previously, the mouth supplied only the start point: the carrier aimed
at Alice and gained a distance-based upward velocity, so the fire could leave
sideways or climb while the head faced another way. Each new packet now follows
the head's pitch, turn and animation sweep. Emitted packets continue their own
trajectory as the head turns; their gravity and collision behavior are retained.

The bright mouth jet also uses the animated attachment's local velocity axes;
its previous world-space velocity could point behind the head. This applies to
the breath emitter only, preserving the smokestacks and other ambient effects.

The separate aimed spiral attack, attack timing, speed, damage, lifetime and
save structure remain unchanged. Existing in-flight projectiles retain their
saved velocities. Both encounters share this fix.

Verification uses the original rig at every ground/flying emission frame,
three body headings and four target positions: 144 launch cases per encounter.
It also checks fire damage, blocked shots, gravity, spiral aiming and exact
saved continuation. Both complete encounter routes, native ground/flying
captures and separate save writer/reader processes pass. Evidence and the
compiled source snapshot are in `private/jabber-fire-fix/`.

The `jlair2` and `grounds1` registrations use the shared actor in
`src/jabberwock/`. They have separate progression, scene and save state. Models,
animations, speech, cameras, brush geometry and authored markers are read from
the user's installed original data; no original assets are distributed here.

## Playable encounters

- **Jabberwock's Lair (`jlair2`)**: original introduction, invulnerable ground
  boss, fire/melee/eye attacks, a 90-second survival clock and two groups of
  three Jabberspawn at 30 and 60 seconds. Gryphon interrupts the battle, the
  Jabberwock loses its eye, Alice collects the visible Eye Staff reward, and
  progression returns to `wforest$wforest_start2`.
- **Royal Rage (`grounds1`)**: Gryphon/Jabberwock introduction and sky changes,
  2,000-health boss, takeoff, flight attacks and dives, landing at 1,000 health,
  ground pursuit/attacks, pain and death. The Gryphon farewell leads to the
  lowering drawbridge, Cheshire's advice, the gnome/guard scene, opening castle
  doors and `grounds2$grounds2_start1`.

Both battles use ordinary player movement, weapons, damage, difficulty scaling,
essence healing and recovery. The boss ignores the Pocket Watch as in its data;
Jabberspawn and spiral projectiles pause. The survival timer still runs. Combat
advances in fixed 120 Hz steps, checks cover/solid surfaces, and bounds waves,
projectiles, attached effects and dynamic lights. Reward and exit state are
latched; failed transitions can retry without duplicate grants.

Scene skipping uses the same checked handoff positions and progression as the
watched scenes. Versioned saves retain boss action, aim, health, shots, waves,
survival clock, dialogue progress, reward, bridge, door and epilogue state.
Invalid saves are rejected before mutation. Old saves without these owners
restart the encounter at its reviewed entry.

Shared fixes forward animation sound cues and Will drain from level controllers
and restore Royal Rage's initial and cinematic sky viewpoints. A shared animation
loop boundary fix prevents a floating-point frame index crash encountered in
Royal Rage's live introduction; its regression test exercises the exact boundary.

## Validation

Use the normal original-data setup described in INSTALL.md. Headless checks:

```powershell
cargo test --locked
looking-glass.exe --jlair2-check --no-audio
looking-glass.exe --grounds1-check --no-audio
looking-glass.exe --jlair2-route-check --no-audio
looking-glass.exe --grounds1-route-check --no-audio
$env:JABBER_SKIP='1'
looking-glass.exe --jlair2-route-check --no-audio
looking-glass.exe --grounds1-route-check --no-audio
Remove-Item Env:JABBER_SKIP
```

The routes use Normal difficulty, real combat/movement, healing and the ordinary
exit/arrival path; neither god mode nor notarget is enabled. Contract fixtures
separately test 30/60/144 Hz timing, pause, Pocket Watch policy, invulnerability,
half-health landing, completion gates, corrupt saves and scene asset references.

Run `--jlair2-render-check` and `--grounds1-render-check` on a graphics-capable
desktop to capture representative scenes, attacks, reward, flight and death.
Captures remain under `private/jabberwock/captures`. The checks assert no dropped
transparent submissions. These staged captures are separate from route proof.

For native save checks, run each render flag with
`LOOKING_GLASS_JABBER_SAVE=write`, then run it again in a new process with
`LOOKING_GLASS_JABBER_SAVE=read`. Each checks introduction, combat, outro,
reward/drawbridge and completed state (plus Royal Rage ground and death states)
through the production Store and full
restore path, including actor art reconstruction. Saves remain private.

The integrated project passed all 621 unit tests on 30 September 2026. Both
full routes passed with scenes watched and skipped, and all 12 native save
fixtures passed in fresh reader processes. The release build and both launchers
also passed smoke checks. Clippy on the source snapshot completed with 22
existing warnings and none in the new encounter modules; a global
`-D warnings` run remains blocked by those baseline warnings.

## Co-op playable survival route, 7 November 2026

Run `--jlair2-coop-route-check --no-audio` in Anode with an isolated profile.
This accelerated check uses both production characters, the full Arena pipeline
and ordinary movement/weapon inputs. It starts at the normal entrance with the
normal level inventory; it does not seed a battle timer, scene outcome, reward,
player position or health. Original scenes play without skipping. A separately
restored arena checks compressed snapshots once per simulated second.

Build `bfdf5b3` passes the full 24,348-tick route in about eight seconds: the
90-second battle, wave appearances at 30/60 seconds, two scene reunions, both
personal Eye Staff collections and actual forest-return placement. All 203
arena replicas match. Both players finish alive at 100/75 health without a
revival or checkpoint rollback, and the arrival preserves personal resources.
The initial final assertion wrongly required wave actors after outro cleanup;
the final check verifies their observed appearances during combat instead.

The report is `jlair2-coop-route.json` in the chosen profile. It includes the
build identity, phases, wave timings, replicas, final players/world, destination
and recent inputs, including on a failed attempt. No screenshots are captured.
This proves normal paired progression, not independent-process internet
delivery. Internet/core-flow evidence is tracked separately. The 944 active
Rust tests, locked release build, seven package checks and source audit pass.

## Co-op playable Royal Rage route, 7 November 2026

`--grounds1-coop-route-check --no-audio` uses the same accelerated production
Arena runner from the normal Royal Rage entrance. Both players fight with
their actual starting weapons and resources; the fixture does not seed boss
damage, defeat, scenes or bridge/gate progression. It watches the original
introduction and farewell, waits for the drawbridge to lower, walks each player
across a separate lane to the gate, then activates the actual castle exit.

Build `4f1d1ea` passes 18,802 simulation ticks and 157 matching compressed
arena replicas. The boss finishes dead and landed; both scene reunions pass.
Blue and Red reach the gate at ticks 18,429 and 18,430, before departure.
Both arrive at `grounds2_start1` with unchanged personal resources, alive at
100/72.27 health and without revival or checkpoint rollback. The profile report
is `grounds1-coop-route.json`. The shared Lair regression remains unchanged at
24,348 ticks, 203 replicas and 100/75 health. Both runs capture no screenshots.
They prove normal paired progression, with internet transport covered separately.

## Remaining original-game parity limits

These are playable reconstructions, not a claim of frame-perfect original AI.
Native steering, beam pulse cadence, dive/jump decisions and projectile bounce
response are approximations where script/TIKI data delegates to unavailable
engine behavior. Authored camera paths, animation clips and dialogue are used,
but portions of aerial choreography and root-motion alignment remain approximate.
The gnome's falling guard uses bounded kinematic motion. The moving drawbridge
pauses if it would embed Alice instead of reproducing the original crushing
damage. Original game audio assets are wired in; automated runs use no audio.
