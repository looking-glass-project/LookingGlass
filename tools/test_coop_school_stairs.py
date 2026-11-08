"""Check the Mallet trip out and back from a staged landing in two clients.

The explicit --start-at diagnostic spawn is the only staging. Traversal uses
ordinary inputs and retains the real collision, gravity, health and networking.
This component test does not certify the complete school visit. Run in Anode.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_school import SchoolPair
from test_coop_play import ROOT, distance


class SchoolStairsPair(SchoolPair):
    evidence_prefix = 'school-stairs-'
    start_at = (-2304, 2800, -256)

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'stairs-fixture-source.py').write_bytes(source)
        self.evidence['stairs_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def run(self):
        self.start_pair()
        self.wait(1)
        self.collect_mallet()
        self.go((-2272, 2440, -256))
        self.return_stairs()
        for role in self.reports:
            for body in self.bodies(role):
                assert body['player']['grounded'] and abs(body['player']['feet'][2] + 256) < 2
                assert distance(body['player']['feet'][:2], (-2304, 2800)) < 60
        self.verify_coherence()
        self.evidence['final_health'] = [b['stats']['sanity'] for b in self.bodies()]
        assert all(r['visit']['map'] == 'skool1' and r['visit']['epoch'] == 1 for r in self.reports.values())
        assert all(b['spawned_at'] == 0 for rows in self.history.values() for row in rows for b in row['players'])
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps(dict(
            passed=True, build=self.build, internet=not self.args.local, first=self.args.first,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Staged Mallet approach, individual pickups and return stairs; not campaign traversal'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = SchoolStairsPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
