"""Real Fortress return combat deaths, spectator cameras and checkpoint wipe.

Two independent internet clients walk from the authored return entrance. The
survivor stays on the upper wall while the other takes actual guard damage.
Only ordinary movement inputs are supplied; no health or world-state writes.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_fortress_return import FortressReturnPair
from test_coop_play import ROOT, read, distance
from test_coop_village_combat import VillageCombat
from test_coop_village_recovery import Recovery


class FortressReturnRecovery(Recovery,FortressReturnPair):
    evidence_prefix = 'fortress-return-recovery-'
    loot_visit = 'fortress1$return'
    death_goal = (-3060,2520,376)
    death_navigate = False
    survivor_navigate = True

    def __init__(self,args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-recovery-source.py').write_bytes(source)
        self.evidence['recovery_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def approach_fall(self,roles):
        for goal in [(-4300,2250,510),(-3800,2250,405),(-3350,2250,390),(-3060,2250,376)]:
            for role in roles:
                self.set(role,goal,navigate=True)
            self.until(lambda: all(self.reached(role) for role in roles),40)

    def survivor_goal(self,victim_index):
        first_index = 1 if self.args.first == 'red' else 0
        return (-4300,2250,510) if victim_index == first_index else (-3800,2250,405)

    def verify_wiped_world(self,report,saved):
        old = saved['frame']['combat']['stage']['world']
        now = report['combat']['stage']['world']
        for field,value in old['fortress'].items():
            if field != 'age':
                assert now['fortress'][field] == value, f'Wipe retained Fortress {field}'
        for before,after in zip(old['encounters']['actors'],now['encounters']['actors']):
            assert after['active'] == before['active'], 'Wipe retained a later enemy activation'
            for kind in ('Guard','Boojum'):
                if kind not in before['enemy']:
                    continue
                a,b = before['enemy'][kind],after['enemy'][kind]
                assert a['health'] == b['health'] and a['state'] == b['state'], 'Wipe retained combat damage/state'
                assert distance(a['feet'],b['feet']) < 1, 'Wipe retained a pursuing enemy position'
                if kind == 'Boojum':
                    assert b['waves'] == [], 'Wipe retained a scream projectile'

    def wipe(self,saved):
        assert all(b['stats']['sanity'] == 25 for b in self.bodies())
        old = saved['frame']['combat']['stage']['world']['encounters']['actors']
        now = self.reports['host']['combat']['stage']['world']['encounters']['actors']
        assert any(a['enemy']['Guard'] != b['enemy']['Guard']
                   for a,b in zip(old,now) if 'Guard' in a['enemy']), 'Guard encounter never advanced beyond checkpoint'
        self.approach_fall(['host','guest'])
        self.cause_wipe(saved)

    def run(self):
        for role in self.commands:
            self.set(role,attack=False)
        self.start_pair()
        checkpoint = self.folder/'host'/'coop-checkpoint.json'
        self.until(lambda: read(checkpoint) is not None,150)
        saved = read(checkpoint)['payload']
        assert saved['visit']['map'] == self.map_name and saved['visit']['entry'] == self.entry
        assert saved['frame']['checkpoint_generation'] == 0
        (self.folder/'starting-checkpoint.json').write_text(json.dumps(saved))
        for victim in ('guest','host') if self.args.first == 'red' else ('host','guest'):
            self.fall_and_respawn(victim)
        self.wipe(saved)
        VillageCombat.verify_coherence(self)
        self.finish_pair(1)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,internet=not self.args.local,
            build=self.build,first=self.args.first,delay_ms=self.args.delay_ms,jitter_ms=self.args.jitter_ms,
            evidence=self.evidence,scope='Both actual Fortress return combat death/spectator/respawn roles and shared saved-checkpoint wipe; no full route'),indent=2))


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
    pair = FortressReturnRecovery(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
