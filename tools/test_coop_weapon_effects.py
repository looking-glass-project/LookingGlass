"""Check ordinary combat effect delivery through two independent public clients.

Use an unchanged Pale Realm encounter checkpoint and ordinary course controls.
Native GPU tests separately compare visible pixels before/after the same restore.
This fixture requires all three transient effect types and both emitting players
to be observed by each client, plus matching authoritative network-state hashes.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_chess_resume import ChessResume
from test_coop_play import ROOT
from test_coop_play import distance
from test_coop_village import verify_rejoin_stats


def verify_effects(history):
    evidence = {}
    for role, rows in history.items():
        peaks = [{key: 0 for key in ('sparks', 'trail', 'flight_trails')} for _ in range(2)]
        for row in rows:
            for peak, effects in zip(peaks, row['weapon_effects']):
                for key in peak:
                    peak[key] = max(peak[key], effects[key])
        assert all(any(p[key] > 0 for p in peaks) for key in peaks[0]), role + ': missing effect type'
        assert all(any(p.values()) for p in peaks), role + ': missing emitting player'
        evidence[role] = peaks
    assert set(evidence) == {'host', 'guest'}, 'Missing client evidence'
    return evidence


class WeaponEffects(ChessResume):
    evidence_prefix = 'weapon-effects-'

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'effects-fixture-source.py').write_bytes(source)
        self.evidence['effects_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def effects_ready(self):
        try:
            verify_effects(self.history)
            return True
        except AssertionError:
            return False

    def run(self):
        self.start_pair()
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit']
            elapsed = report['frame']['tick'] - self.saved['frame']['tick']
            for current, old in zip(report['frame']['players'], self.saved['frame']['players']):
                assert distance(current['player']['feet'], old['player']['feet']) < 2
                verify_rejoin_stats(old['stats'], current['stats'], elapsed)
        for role in self.commands:
            self.set(role, step=self.args.start_step)
        self.until(self.effects_ready, 60)
        for role in self.commands:
            self.set(role)
        self.wait(.3)
        self.evidence['weapon_effects'] = verify_effects(self.history)
        for role, rows in self.history.items():
            self.verify_presence(rows, role)
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route_passed=False,
            build=self.build, internet=not self.args.local, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Effect delivery during ordinary combat; no encounter-victory or traversal credit'), indent=2))
        print('PASS both clients received weapon sparks, swing/projectile trails and both emitters', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--start-step', type=int, default=126)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    args.save_before = []
    args.last_step = args.start_step
    if not (0 < args.start_step < 161):
        parser.error('Choose a nonzero course step')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = WeaponEffects(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False,
            error=str(error), build=pair.build, reports=pair.reports, evidence=pair.evidence)))
        raise
    finally:
        pair.close()
