"""Test a Pale Realm segment from an unchanged ordinary co-op save.

This uses two independent clients and the normal resume path. A passing segment
never counts as a complete entrance-to-exit traversal.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_chess import ChessPair
from test_coop_play import ROOT
from test_coop_village import verify_rejoin_stats


def verify_resume_receipt(receipt, saved, build):
    """Require the native exact comparison made before any physics advances."""
    assert receipt['verified'] is True, 'Native resume was not verified'
    assert receipt['build'] == build, 'Resume receipt belongs to another build'
    assert receipt['epoch'] == saved['visit']['epoch'], 'Resume receipt changed epoch'
    assert receipt['tick'] == saved['frame']['tick'], 'Resume receipt changed checkpoint tick'


def verify_resume_body(current, old, elapsed):
    # A captured body can still have momentum. By the first network report it
    # has advanced normally, so position equality belongs to the native receipt.
    assert elapsed >= 0, 'Resume observation precedes the checkpoint'
    for stamp in ('spawned_at', 'warped_at'):
        assert current[stamp] == old[stamp], 'Resume unexpectedly changed ' + stamp
    verify_rejoin_stats(old['stats'], current['stats'], elapsed)


class ChessResume(ChessPair):
    evidence_prefix = 'chess-resume-'
    resume_checkpoint = True

    def __init__(self, args):
        raw = args.checkpoint.read_bytes()
        self.saved = json.loads(raw)['payload']
        assert self.saved['visit']['map'] == self.map_name and self.saved['visit']['epoch'] == 1
        assert self.saved['frame']['checkpoint_generation'] == 0
        super().__init__(args)
        (self.folder / 'host' / 'coop-checkpoint.json').write_bytes(raw)
        self.evidence.update(checkpoint_sha256=hashlib.sha256(raw).hexdigest(),
                             checkpoint_tick=self.saved['frame']['tick'],
                             resumed_steps=[args.start_step, args.last_step])
        source = Path(__file__).read_bytes()
        (self.folder / 'resume-fixture-source.py').write_bytes(source)
        self.evidence['resume_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def run(self):
        self.start_pair()
        receipt = json.loads((self.folder / 'host' / 'route-resume.json').read_text())
        verify_resume_receipt(receipt, self.saved, self.build)
        self.evidence['exact_resume_receipt'] = receipt
        for role, report in self.reports.items():
            assert report['visit'] == self.saved['visit'], 'Resume changed visit'
            elapsed = report['frame']['tick'] - self.saved['frame']['tick']
            for current, old in zip(report['frame']['players'], self.saved['frame']['players']):
                verify_resume_body(current, old, elapsed)
        self.after_resume()
        for role in self.commands:
            self.set(role, step=self.args.start_step)
        self.until(lambda: len(self.course_hashes) == 2, 20)
        assert 0 < self.args.start_step <= self.args.last_step < len(self.steps)
        for index in range(self.args.start_step, self.args.last_step + 1):
            if index in self.args.save_before:
                self.save_checkpoint(index)
            (self.folder / f'before-step-{index:03d}.json').write_text(json.dumps(self.reports['host']))
            if index > self.args.start_step or index in self.args.save_before:
                for role in self.commands:
                    self.set(role, step=index)
            self.until(lambda: self.arrived() or all(self.reached(role) for role in self.commands))
            self.completed.append(index)
            print('PASS resumed Pale Realm step', index, flush=True)
            if self.arrived():
                break
        assert self.completed == list(range(self.args.start_step, self.args.last_step + 1))
        for role, rows in self.history.items():
            route = [r for r in rows if r['map'] == self.map_name and r['epoch'] == 1]
            self.verify_presence(route, role)
            assert all(p['health'] > 0 and p['spawned_at'] == old['spawned_at']
                       and not p['stats']['god'] and not p['stats']['notarget']
                       for row in route for p, old in zip(row['players'], self.saved['frame']['players']))
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')
        self.evidence['completed_steps'] = self.completed
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=self.reports[role]['visit']['epoch'])
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route_passed=False,
            build=self.build, internet=not self.args.local, first=self.args.first,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Resumed segment only; no full-level completion credit'), indent=2))
        print('PASS resumed Pale Realm segment; not a complete traversal', flush=True)

    def after_resume(self):
        """Optional lifecycle check after validating the unchanged checkpoint."""


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--start-step', type=int, required=True)
    parser.add_argument('--last-step', type=int, required=True)
    parser.add_argument('--save-before', nargs='*', type=int, default=[])
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if not (0 < args.start_step <= args.last_step < 1000):
        parser.error('Choose an ordered, nonzero course segment')
    pair = ChessResume(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, last_valid=pair.last_valid, completed=pair.completed)))
        raise
    finally:
        pair.close()
