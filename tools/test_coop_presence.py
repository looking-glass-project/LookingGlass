"""Two-client contact/ambient-effects regression, using ordinary movement inputs.

This is a focused entrance test, not campaign traversal credit. Internet hosting
and 40/20 ms delay/jitter are the defaults. All evidence stays in private/coop.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_pandemonium import PandemoniumPair
from test_coop_play import ROOT, distance


class PresencePair(PandemoniumPair):
    map_name = 'gvillage'
    evidence_prefix = 'presence-test-'

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()
        for role in self.commands:
            self.set(role, attack=False)

    def pump(self):
        super().pump()
        for role, report in self.reports.items():
            if self.history[role]:
                self.history[role][-1]['presentation'] = report.get('presentation')

    def run(self):
        self.start_pair()
        self.until(lambda: all('entry' in r['combat']['stage']['world']['village']['cinema']['done']
                               and r['combat']['stage']['owner'] is None for r in self.reports.values()),120)
        self.wait(1)
        self.capture_pair('ambient-after-entry')
        initial = [b['player']['feet'] for b in self.bodies()]
        for moving in ('guest', 'host'):
            index = 1 if moving == 'guest' else 0
            other = 1-index
            fixed = self.bodies()[other]['player']['feet']
            self.set(moving, fixed, run=True, attack=False)
            self.until(lambda: all(35.9 <= distance(b['frame']['players'][0]['player']['feet'][:2],
                                                    b['frame']['players'][1]['player']['feet'][:2]) <= 36.2
                                   for b in self.reports.values()), 15)
            self.hold(2)
            self.capture_pair(moving+'-contact')
            for role, report in self.reports.items():
                bodies = report['frame']['players']
                assert distance(bodies[other]['player']['feet'], fixed) < .1, 'Stationary partner was pushed'
                assert 35.9 <= distance(bodies[0]['player']['feet'][:2], bodies[1]['player']['feet'][:2]) <= 36.2
                visual = report['presentation']
                assert all(visual['visible_players']), f'{role}: nearby Alice disappeared'
                assert visual['particles'][2] > 0, f'{role}: no ambient particles'
                assert visual['effect_batches'][1] == 0, f'{role}: effect pass dropped batches'
                # Contact can turn the camera toward a bare wall. Require real
                # rendered effects in this client's history, not every angle.
                assert any((row.get('presentation') or {}).get('effect_batches',[0])[0] > 0
                           for row in self.history[role]), f'{role}: effects were never drawn'
            self.set(moving, initial[index], attack=False)
            self.until(lambda: self.reached(moving), 15)
            self.wait(.5)
        assert all(row['checkpoint_generation'] == 0 and all(p['health'] > 0 and p['spawned_at'] == 0
                   for p in row['players']) for rows in self.history.values() for row in rows)
        self.verify_network_states()
        self.evidence['contact_distance'] = 36
        self.evidence['stationary_partner_unchanged'] = True
        self.evidence['both_models_and_particles_visible'] = True
        self.finish_pair(1)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,build=self.build,
            internet=not self.args.local,delay_ms=self.args.delay_ms,jitter_ms=self.args.jitter_ms,
            evidence=self.evidence,scope='Focused player contact and ambient effects only'),indent=2))
        print('PASS both clients: player contact, stationary partner, ambient particles and effect pass.',flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=ROOT/'target/release/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    args = parser.parse_args()
    args.first, args.rejoin = 'blue', False
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = PresencePair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
