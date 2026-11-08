"""Check Pandemonium's first club guard and personal rewards in two clients.

Uses the ordinary entrance rope and shared minecart. Real damage, nearest-player
switching, separate healing drops and matching enemy snapshots are verified over
the public internet invite by default. Full routes/recovery/rejoin are separate.
"""
import argparse
from pathlib import Path

from test_coop_play import ROOT
from test_coop_pandemonium import PandemoniumPair
from test_coop_village_combat import VillageCombat


class PandemoniumCombat(VillageCombat, PandemoniumPair):
    evidence_prefix = 'pandemonium-combat-'
    guard_name = 'airship_cardguard1'
    map_pickup = None

    def prepare_pickups(self):
        pass  # This guard's reward is supported on the ordinary ramp.

    def wait_for_minecart(self):
        # The route's long settling wait normally permits auto-combat. Here the
        # test takes control as soon as the real ride/cutscene finishes.
        for role in self.commands:
            self.set(role)
        self.until(lambda: self.pand()['cart'] == 'Finished'
                   and all(r['combat']['stage']['owner'] is None for r in self.reports.values()), 90)

    def run(self):
        for role in self.commands:
            self.set(role, attack=False)
        self.start_pair()
        self.rope()
        self.minecart()
        self.encounter((-3264, -552, 440), (-3184, -152, 376), (-3232, -304, 405))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='red')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = PandemoniumCombat(args)
    try:
        pair.run()
    finally:
        pair.close()
