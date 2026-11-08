# Co-op weapon effect verification

The host produced impact sparks, swing trails and projectile trails, but omitted
those transient buffers from its character snapshot. Guest restoration cleared
them, leaving those effects absent on the second client. The snapshot now carries
their world positions, remaining life, colors and trail type. Guests restore the
buffers without rerunning attacks, charging resources or simulating hits.

The fields are optional for older saves and omitted when empty. Validation caps
sparks/projectile trails at 256 each and swing trails at 32, matching their runtime
bounds, and rejects non-finite geometry or invalid life/color values. Reset and
death still clear the buffers. Two Rust regressions cover legacy saves, exact
round trips, invalid values and oversized buffers.

## Automated native rendering

`--blade-cards-render-check` now includes six focused replica-effect cases. For
each Alice skin, funded ordinary attacks create a swing, a projectile trail and
an impact against a wall. The test renders each effect through `Character`,
serializes/restores it three times and compares the resulting pixels. A negative
control removes only that effect buffer and requires a visible image difference.
This prevents an empty or missing effect from passing merely because both images
are equally empty. No model image inspection is required.

On source `871bb0bd73a825b16198fa4bc849099d070dc2305cde233ace35631afe2fb590`,
all six cases pass: **776 swing-trail pixels, 1,265 projectile-trail pixels and
184 spark pixels** for each Alice. Repeated restored images match within one
channel value of GPU rounding. The existing Blade/Cards native checks pass too.
Evidence: `private/coop-weapon-effects-native18.log` and
`private/coop-weapon-effects18/coop-weapon-effects.json`.

## Independent internet clients

`tools/test_coop_weapon_effects.py` resumes an unchanged ordinary Pale Realm
encounter checkpoint. Each client issues its own normal combat inputs. The test
waits until both clients have observed all three effect types and effects from
both players, then verifies presence and matching player/world/native hashes.
It makes no claim about winning that encounter or traversing the level. Three
guard tests reject a missing effect type, emitting player or client.

The public run with **40 ms delay and 20 ms jitter per sender** passes in 25.9
seconds, with **59 matching player/world/native states**. Both clients observe
Blue and Red sparks/swing trails and Red projectile trails. Native rendering
separately covers all three effects for both skins. Peak counts need not match
between clients because their diagnostic observations occur at different ticks;
the compared same-tick authoritative hashes do match.

Evidence: `private/coop/weapon-effects-1d31e1cdade24812bf4166582de60479/` and
`private/coop-weapon-effects-internet18.log`. Histories retain per-player effect
counts, and the exact fixture source is copied into the evidence folder.

The frozen executable is `private/coop-wchess1-world-871bb0b.exe`, SHA-256
`a686a078af1193e69eee1d607d9205baff59cdac0ce989e20c1a373385ea6a0a`.
Locked release build and **822 active Rust tests** pass (one existing ignored).
Logs: `private/coop-weapon-effects-{build18,unit18}.log`.

This verifies the three shared transient buffers. Other weapon, enemy, attached
emitter and environmental effects still need their own coverage; the result
does not certify every particle in every campaign visit.

## Checkmate repeat on 28 October 2026

Source `afe78ef49fcd940d037fb41d0c62d346b55d4fe1fe556668ab65af78db158156`
passes the same six GPU cases again: 776 swing-trail pixels, 1,265 projectile-trail
pixels and 184 spark pixels for each Alice, with three matching restores each.
An isolated native profile contains `coop-weapon-effects.json`; the numerical
checks require no model image inspection. Local evidence is under
`private/coop-rope-combat-render-afe78ef/`.

The fresh Red-led Checkmate internet route on that build also passes the existing
effect-delivery verifier for all three types and both emitting players in both
clients. Evidence is the retained route
`checkmate-recovery-route-6e982980bd2a45169c7e1ba34372af1a`, with a supplemental
history/verifier hash record at `private/coop-checkmate-effects-afe78ef.json`.
This adds current Checkmate weapon coverage. It does not extend the claim to
native King attacks, attached emitters or all environmental particles.
