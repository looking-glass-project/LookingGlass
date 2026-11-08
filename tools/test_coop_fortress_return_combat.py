"""Real Fortress return guard targeting and individual healing over the internet.

Both clients walk from the authored return entrance, exchange the closest-player
role, take actual enemy damage, then use ordinary Blade attacks and pickups.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_fortress_return import FortressReturnPair
from test_coop_village_combat import VillageCombat
from check_coop_rewards import personal_rewards
from test_coop_play import ROOT


class FortressReturnCombat(VillageCombat,FortressReturnPair):
    evidence_prefix = 'fortress-return-combat-'
    loot_visit = 'fortress1$return'
    guard_name = 'end_spawn4'
    reward_source = 3000004
    map_pickup = None

    def __init__(self,args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-combat-source.py').write_bytes(source)
        self.evidence['combat_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def selected(self,index):
        enemy = self.guard()
        return (enemy['active'] and enemy['health'] > 0 and enemy['state'] in ('Chase','Alert','Attack')
                and enemy['selected_player'] == index and all(enemy['line_of_sight'])
                and enemy['distance'][index]+30 < enemy['distance'][1-index])

    def run(self):
        for role in self.commands:
            self.set(role,attack=False)
        self.start_pair()
        for goal in [(-4300,2250,510),(-3800,2250,405),(-3350,2250,390)]:
            self.go(goal,True)
        first,other = self.first,self.partner
        index = 0 if first == 'host' else 1
        # Move around the parapet before asking the guard to hit either player.
        for goal in [(-3060,2250,376),(-3060,2520,376)]:
            self.set(first,goal,navigate=True)
            self.until(lambda: self.reached(first),30)
        self.check_target(index)
        self.set(first,(-3350,2250,390),navigate=True)
        self.until(lambda: self.reached(first),30)
        for goal in [(-3060,2250,376),(-3060,2520,376)]:
            self.set(other,goal,navigate=True)
            self.until(lambda: self.reached(other),30)
        self.check_target(1-index)
        self.set(first,(-3350,2250,390),navigate=True,attack=False)
        self.set(other,(-3060,2520,376),attack=True)
        self.until(lambda: self.guard()['health'] == 0,30)
        self.wait(1.5)
        for role in self.commands:
            self.set(role,attack=False)
        self.capture('return-guard-defeated')
        self.pickups()
        self.evidence['personal_rewards'] = personal_rewards(self.history,self.loot_visit,[])
        self.verify_coherence()
        self.finish_pair(1)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,build=self.build,
            internet=not self.args.local,first=self.args.first,delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms,evidence=self.evidence,
            scope='Return-visit nearest-visible guard damage in both roles and personal earned healing; no full route or recovery'),indent=2))
        print('PASS Fortress return: nearest-visible guard damage in both roles and separate healing.',flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=ROOT/'target/debug/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--first',choices=('blue','red'),default='red')
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    parser.set_defaults(rejoin=False)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = FortressReturnCombat(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
