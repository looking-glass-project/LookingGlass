"""First-school nearest-visible guard damage and personal essence healing.

Both independent public clients take real damage, exchange the closer-player
role, then collect separate copies of a fresh theatre guard's earned essence.
No resource, position, enemy or quest state is supplied by the fixture.
"""
import argparse
import hashlib
import json
from pathlib import Path

from check_coop_rewards import personal_rewards
from test_coop_play import ROOT
from test_coop_school import SchoolPair
from test_coop_village_combat import VillageCombat


class SchoolCombat(VillageCombat,SchoolPair):
    evidence_prefix = 'school-combat-'
    guard_name = 'cgd_top1'
    # The upper guard's drop expires before the stairs are accessible. Use the
    # newly defeated theatre-floor guard immediately after the original scene.
    reward_source = 3000019
    map_pickup = None

    def __init__(self,args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-combat-source.py').write_bytes(source)
        self.evidence['combat_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def selected(self,index):
        guard = self.guard()
        return (guard['active'] and guard['health'] > 0 and guard['state'] in ('Chase','Alert','Attack')
                and guard['selected_player'] == index and all(guard['line_of_sight'])
                and guard['distance'][index]+30 < guard['distance'][1-index])

    def theatre_healing(self):
        self.until(lambda: all(next(g for g in r['route_driver']['guards']
                                   if g['name'] == 'play_guard1')['health'] == 0
                               for r in self.reports.values()),20)
        self.pickups()

    def run(self):
        for role in self.commands:
            self.set(role,attack=False)
        self.start_pair()
        for role in self.commands:
            self.set(role,(-1888,2220,-512))
        self.until(lambda: all(self.reached(r) for r in self.commands),30)
        for role in (self.first,self.partner):
            index = 0 if role == 'host' else 1
            self.set(role,(-1888,2298,-512))
            self.until(lambda: self.reached(role),20)
            self.check_target(index)
            self.set(role,(-1888,2220,-512))
            self.until(lambda: self.reached(role),20)
            self.wait(2)
        for role in self.commands:
            self.set(role,(-1888,2298,-512),attack=True)
        self.until(lambda: self.guard()['health'] == 0,35)
        self.wait(1.5)
        self.capture('school-upper-guard-defeated')
        self.theatre(upstairs=False)
        for role in self.commands:
            self.set(role,attack=False)
        self.evidence['personal_rewards'] = personal_rewards(self.history,'skool1$first')
        SchoolPair.verify_coherence(self)
        self.finish_pair(1)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,build=self.build,
            internet=not self.args.local,first=self.args.first,delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms,evidence=self.evidence,
            scope='First-school nearest-visible guard damage in both roles and separate earned healing; no full traversal'),indent=2))
        print('PASS school nearest-visible guard damage and personal healing.',flush=True)


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
    pair = SchoolCombat(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
