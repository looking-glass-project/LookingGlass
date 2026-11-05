"""Verify a natural fall and rope revival from an isolated captured checkpoint.

Two independent profiles use the public internet transport by default. Inputs
stay neutral: the saved falling player dies through ordinary physics. This is
a recovery segment, not a clean traversal or full-campaign completion check.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

from coop_recovery import verify_presence
from test_coop_play import ROOT, distance
from test_coop_village import Pair


class RopeRevivalPair(Pair):
    evidence_prefix = 'rope-revival-'
    capture_scenes = False
    resume_checkpoint = True

    def __init__(self, args):
        self.seed = json.loads(args.checkpoint.read_text())['payload']
        self.map_name = self.seed['visit']['map']
        self.entry = self.seed['visit']['entry']
        self.victim = 0 if args.victim == 'blue' else 1
        self.deaths, self.revivals, self.followed = {}, {}, []
        super().__init__(args)
        shutil.copyfile(args.checkpoint, self.folder / 'host' / 'coop-checkpoint.json')
        self.evidence['checkpoint_sha256'] = hashlib.sha256(args.checkpoint.read_bytes()).hexdigest()
        (self.folder / 'fixture-source.py').write_bytes(Path(__file__).read_bytes())

    def set(self, role, *args, **kwargs):
        kwargs['attack'] = False
        super().set(role, *args, **kwargs)

    def validate_report(self, role, report):
        frame = report['frame']
        assert frame['checkpoint_generation'] == self.seed['frame']['checkpoint_generation'], 'Unexpected team wipe'
        assert report['visit']['map'] == self.map_name
        assert all(not b['stats']['god'] for b in frame['players'])
        # Spectating follows the rendered rope body, including ordinary network
        # smoothing. Check the exact render inputs, not a fixed network gap.
        verify_presence([dict(checkpoint_generation=frame['checkpoint_generation'],
            owner=report['combat']['stage']['owner'],
            camera_player=report['camera_player'], camera_target=report['camera_target'],
            camera_subject_feet=report['camera_subject_feet'], presentation=report['presentation'],
            players=[dict(feet=b['player']['feet'], health=b['stats']['sanity'],
                spawned_at=b['spawned_at'], warped_at=b['warped_at']) for b in frame['players']])],
            role, generation=self.seed['frame']['checkpoint_generation'])
        local = 0 if role == 'host' else 1
        body = frame['players'][self.victim]
        partner = frame['players'][1 - self.victim]
        life = frame['lives']['states'][self.victim]
        if body['stats']['sanity'] <= 0:
            assert isinstance(life, dict) and 'Spectating' in life, 'Safe rope revival missed its countdown'
            left = life['Spectating']['ticks_left']
            assert 0 < left <= 1200
            death = frame['tick'] - (1200 - left)
            assert self.deaths.setdefault(role, death) == death, 'Death clock changed'
            if local == self.victim:
                assert report['camera_player'] == ('Blue', 'Red')[1 - self.victim]
                subject = report['camera_subject_feet']
                assert distance(report['camera_target'][:2], subject[:2]) < 1
                self.followed.append(report['camera_target'])
        elif body['spawned_at'] > self.seed['frame']['players'][self.victim]['spawned_at']:
            if role in self.revivals:
                return
            assert role in self.deaths, 'Revival without an observed countdown'
            assert body['spawned_at'] - self.deaths[role] == 1200
            assert life == 'Alive' and body['stats']['sanity'] == 25
            rope = body['player']['rope']
            assert rope is not None and partner['player']['rope'] is not None
            assert rope['id'] == partner['player']['rope']['id']
            gap = distance(body['player']['feet'], partner['player']['feet'])
            if getattr(self, 'exact_recovery_trace', False):
                # The survivor may keep climbing after the exact spawn tick.
                # Presence above verifies current non-overlap; the host trace
                # must separately prove geometry at the actual spawn boundary.
                elapsed = frame['tick'] - body['spawned_at']
                assert 0 <= elapsed <= 120
                assert gap <= 120 + 1000 * elapsed / 120, 'Rope bodies moved beyond normal speed'
            else:
                assert 64 <= gap <= 120, f'Overlapping or distant rope revival: {gap}'
            saved = self.seed['frame']['players'][self.victim]['stats']
            for key in ('owned', 'collected', 'selected', 'turtle_air', 'difficulty'):
                assert body['stats'][key] == saved[key], f'Revival changed personal {key}'
            if local == self.victim:
                assert report['camera_player'] == ('Blue', 'Red')[self.victim]
                assert distance(report['camera_target'][:2], body['player']['feet'][:2]) < 1
            self.revivals[role] = dict(death_tick=self.deaths[role], spawn_tick=body['spawned_at'],
                health=body['stats']['sanity'], separation=gap, rope=rope['id'])
            print(role, 'observed rope revival', self.revivals[role], flush=True)

    def run(self):
        self.start_pair()
        self.until(lambda: len(self.revivals) == 2, 45)
        assert len(set(self.deaths.values())) == 1, 'Clients disagree on death tick'
        assert len({r['spawn_tick'] for r in self.revivals.values()}) == 1
        assert self.followed and max(distance(p, self.followed[0]) for p in self.followed) > 10
        assert self.reports['host']['pid'] != self.reports['guest']['pid']
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_enemies')
        self.evidence['revivals'] = self.revivals
        # Close normally without requesting screenshots.
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            victim=self.args.victim, evidence=self.evidence, full_route_passed=False,
            scope='Captured current-visit recovery segment: spectator camera, exact countdown, rope revival and state agreement'), indent=2))
        print('PASS independent clients: spectator camera and ten-second rope revival.', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--victim', choices=('blue', 'red'), default='red')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = RopeRevivalPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failure-report.json').write_text(json.dumps(dict(passed=False,
            error=str(error), reports=pair.reports, deaths=pair.deaths, revivals=pair.revivals), indent=2))
        raise
    finally:
        pair.close()
