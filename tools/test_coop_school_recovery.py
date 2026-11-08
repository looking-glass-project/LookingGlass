"""Real first-school guard deaths, timed spectating and shared checkpoint recovery.

Each player walks from the ordinary entrance into the live upper-guard sightline.
The survivor remains in the entrance passage. No damage or state is injected.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_school import SchoolPair
from test_coop_play import ROOT, read, distance
from test_coop_village_recovery import Recovery


class SchoolRecovery(Recovery,SchoolPair):
    evidence_prefix = 'school-recovery-'
    # Stand inside the upper guard's firing range without moving under the
    # balcony. At y=2290, braking can stop a body just outside firing range.
    death_goal = (-1888,2298,-512)

    def __init__(self,args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-recovery-source.py').write_bytes(source)
        self.evidence['recovery_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def approach_fall(self,roles):
        for role in roles:
            self.set(role,(-1888,2160,-512))
        self.until(lambda: all(self.reached(role) for role in roles),25)

    def survivor_goal(self,victim_index):
        # The first respawn is beside the first survivor. A different depth on
        # the second pass ensures the new survivor actually walks on camera.
        return (-1920,2048,-512) if victim_index == (1 if self.args.first == 'red' else 0) else (-1860,1856,-512)

    def verify_wiped_world(self,report,saved):
        old = saved['frame']['combat']['stage']['world']
        now = report['combat']['stage']['world']
        for field,value in old['school'].items():
            if field not in ('clock','objects'):
                assert now['school'][field] == value, f'Wipe retained school {field}'
        assert len(now['school']['objects']) == len(old['school']['objects'])
        for before,after in zip(old['school']['objects'],now['school']['objects']):
            for field in ('name','enabled','solid'):
                assert before[field] == after[field], 'Wipe retained a later school object state'
            if before['motion'] is None:
                assert after == before, 'Wipe retained a later school movement'
            else:
                assert after['motion']['keys'] == before['motion']['keys']
                assert after['motion']['repeat'] == before['motion']['repeat']
        assert len(now['encounters']['actors']) == len(old['encounters']['actors'])
        for before,after in zip(old['encounters']['actors'],now['encounters']['actors']):
            assert before['name'] == after['name'] and before['active'] == after['active']
            for kind in ('Guard','Boojum'):
                if kind not in before['enemy']:
                    continue
                a,b = before['enemy'][kind],after['enemy'][kind]
                assert a['health'] == b['health'] and a['state'] == b['state']
                assert distance(a['feet'],b['feet']) < 1, 'Wipe retained a pursuing enemy position'
                assert not b.get('shots',[]) and not b.get('waves',[]), 'Wipe retained enemy projectiles'

    def wipe(self,saved):
        assert all(b['stats']['sanity'] == 25 for b in self.bodies())
        old = saved['frame']['combat']['stage']['world']['encounters']['actors']
        now = self.reports['host']['combat']['stage']['world']['encounters']['actors']
        assert any(a != b for a,b in zip(old,now)), 'Encounter never advanced beyond checkpoint'
        self.cause_wipe(saved)

    def run(self):
        for role in self.commands:
            self.set(role,attack=False)
        self.start_pair()
        checkpoint = self.folder/'host'/'coop-checkpoint.json'
        self.until(lambda: read(checkpoint) is not None,150)
        saved = read(checkpoint)['payload']
        assert saved['visit']['map'] == self.map_name and saved['frame']['checkpoint_generation'] == 0
        (self.folder/'starting-checkpoint.json').write_text(json.dumps(saved))
        for victim in ('guest','host') if self.args.first == 'red' else ('host','guest'):
            self.fall_and_respawn(victim)
        self.wipe(saved)
        self.verify_coherence()
        self.finish_pair(1)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,internet=not self.args.local,
            build=self.build,first=self.args.first,delay_ms=self.args.delay_ms,jitter_ms=self.args.jitter_ms,
            evidence=self.evidence,scope='Both first-school combat death/spectator/respawn roles and shared saved-checkpoint wipe; no full route'),indent=2))


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
    pair = SchoolRecovery(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
