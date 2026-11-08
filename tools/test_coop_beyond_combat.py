"""Beyond guard targeting and personal healing in two internet clients.

Walk from the normal entrance, swap the closest visible player, take real guard
damage, defeat the guard with the Blade, then collect each personal healing drop.
No health, inventory, placement or encounter-state injection is used.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_beyond import BeyondPair
from test_coop_play import ROOT
from test_coop_village_combat import VillageCombat


class BeyondCombat(VillageCombat, BeyondPair):
    evidence_prefix = 'beyond-combat-'
    reward_source = 3000008
    map_pickup = None

    def __init__(self,args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-combat-source.py').write_bytes(source)
        self.evidence['combat_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def guard(self,role='host'):
        return next(g for g in self.reports[role]['route_driver']['guards'] if g['id'] == self.reward_source)

    def prepare_pickups(self):
        # The hallway's earned drops land on its broad, supported floor.
        self.wait(1)

    def run(self):
        for role in self.commands:
            self.set(role,attack=False)
        self.start_pair()
        self.section = 'hallway guard encounter'
        self.go((-750,4128,0),together=True)
        self.go((-750,3800,0),together=True)
        self.encounter(near=(-160,3800,0),far=(-650,3800,0),retreat=(-500,3800,0))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=ROOT/'target/debug/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--first',choices=('blue','red'),default='red')
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    parser.set_defaults(arches_only=False,corridor_only=False,rejoin=False)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = BeyondCombat(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
