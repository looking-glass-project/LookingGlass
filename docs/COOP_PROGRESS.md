# Co-op progress

Updated 29 October 2026. Branch: `coop`. **Estimated overall completion: 48.1%.**
The older 42% reports are historical. This percentage measures delivered and
verified behavior against the full co-op scope; it is not a running test's progress.

Six of the 39 campaign visits have combined verification evidence; 33 remain open.
The earlier contact-system repeats are still required. The remaining scope includes
full paired traversal, both scene owners, deaths and checkpoint recovery, reconnects,
shared transports, swimming/leaves and single-player regressions across the campaign.

| Area | Credited points | Maximum |
| --- | ---: | ---: |
| Foundation and independent client diagnostics | 8 | 10 |
| Internet Host/Join | 7 | 15 |
| Two players, movement and presentation | 10.25 | 15 |
| Combat, individual resources and death | 9.25 | 20 |
| Scenes, transport and progression | 10.5 | 20 |
| Full campaign verification | 3.08 | 20 |
| **Total** | **48.08, rounded to 48.1** | **100** |

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

Current focus: the connected Funhouse fulcrums, steps 95-98. Both players have
observed first-platform jumps, and Blue has reached the far bank. A combined
crossing remains unverified. Earlier runs lose Blue while waiting. The latest
run keeps both alive at 81/100 health, verifies both first jumps, then fails because
Red is about 178 units below the second landing. The test-controller arrival check
is corrected; the remaining second approach has exposed a third loaded fulcrum missing from
the reset check. The waiting position and connected-mover gate need correction. These failed full-section runs receive no campaign completion credit.

Tests run as two independent clients through the public network path on Anode,
with injected delay and jitter. Routine diagnosis uses state and log assertions.
Screenshots are not needed for each run. The current source also has 865 passing
active Rust tests and native Funhouse battle, machinery and scene checks; those
component results do not prove a full campaign route.

See the [fixed weighting and detailed history](COOP.md#delivery-and-verification-weighting),
[campaign matrix](COOP_CAMPAIGN.md) and [Funhouse evidence](FUNHOUSE.md). Future
updates should state the overall percentage, the new bounded milestone, and the
next unresolved acceptance check even when the percentage does not change.
