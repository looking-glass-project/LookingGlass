"""Verify an ordinary Castling combat death, moving spectator and timed revival.

Start from the unchanged normal checkpoint before step 77. The chosen player
approaches the knights without attacking; the partner retreats along the cleared
route. No player, enemy, inventory or progression values are written by the test.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_castling import verify_presence
from test_coop_castling_resume import CastlingResume
from test_coop_chess_recovery import verify_revives
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from test_coop_play import ROOT


class CastlingRecovery(CastlingResume):
    evidence_prefix = 'castling-recovery-'

    def __init__(self, args):
        self.recovering = False
        self.victim = ('blue', 'red').index(args.victim)
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'recovery-fixture-source.py').write_bytes(source)
        self.evidence['recovery_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def validate_report(self, role, report):
        if not self.recovering:
            return super().validate_report(role, report)
        frame = report['frame']
        assert frame['checkpoint_generation'] == 0, 'Team wiped during individual recovery'
        assert frame['players'][1-self.victim]['stats']['sanity'] > 0, 'Survivor died'
        assert all(not b['stats']['god'] and not b['stats']['notarget'] for b in frame['players'])
        previous = self.last_valid.get(role)
        if previous and any(b['stats']['sanity'] < a['stats']['sanity']
                            for a, b in zip(previous['frame']['players'], frame['players'])):
            path = self.folder / f'damage-{role}-{frame["tick"]}.json'
            if not path.exists():
                path.write_text(json.dumps(dict(before=previous, after=report)))

    def dead(self):
        return self.reports['host']['frame']['players'][self.victim]['stats']['sanity'] <= 0

    def run(self):
        self.start_pair()
        receipt = json.loads((self.folder / 'host' / 'route-resume.json').read_text())
        verify_resume_receipt(receipt, self.saved, self.build)
        self.evidence['exact_resume_receipt'] = receipt
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit']
            elapsed = report['frame']['tick'] - self.saved['frame']['tick']
            for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                verify_resume_body(body, old, elapsed)
        self.recovering = True
        victim_role, survivor_role = ('host', 'guest')[self.victim], ('host', 'guest')[1-self.victim]
        for role in self.commands:
            self.set(role)
        survivor = self.reports['host']['frame']['players'][1-self.victim]['player']['feet']
        # Blue starts in front of Red on this ramp. Pass in the clear eastern
        # lane before retreating uphill, keeping both solid bodies separate.
        retreat = [[-128, -1056, 157.25]]
        if self.victim == 1:
            retreat = [[-80, survivor[1], survivor[2]], [-80, -1056, 157.25]] + retreat
        for goal in retreat:
            self.set(survivor_role, goal=goal, navigate=True)
            self.until(lambda: self.reached(survivor_role) or self.dead(), 30)
            if self.dead():
                break
        if not self.dead():
            self.set(victim_role, goal=[-128, -640, 64.03125], navigate=True)
            self.until(self.dead, 120)
        (self.folder / 'death.json').write_text(json.dumps(self.reports))
        body = self.reports['host']['frame']['players'][self.victim]
        old_spawn = body['spawned_at']
        self.set(victim_role)
        # Walk the survivor through known cleared ramp/landing points while the
        # dead client's camera follows. Continue a retreat already in progress.
        retreats = [[-128, -1056, 157.25], [-128, -1088, 167.125], [-128, -1152, 186.8125],
                    [-128, -1184, 192.03125], [-128, -1312, 192.03125],
                    [-256, -1312, 192.03125], [-256, -1344, 192.03125], [-480, -1344, 192.03125]]
        y = self.reports['host']['frame']['players'][1-self.victim]['player']['feet'][1]
        retreats = [p for p in retreats if p[1] < y-8]
        assert retreats, 'No cleared spectator retreat remains'
        self.set(survivor_role, goal=retreats.pop(0), navigate=True)
        end = self.reports['host']['frame']['tick'] + 1800
        while True:
            self.pump()
            report = self.reports[victim_role]
            body = report['frame']['players'][self.victim]
            if body['spawned_at'] > old_spawn and body['stats']['sanity'] > 0:
                break
            assert self.reports['host']['frame']['tick'] < end, 'Revival did not finish'
            if self.reached(survivor_role) and retreats:
                self.set(survivor_role, goal=retreats.pop(0), navigate=True)
        self.wait(.3)
        initial = [b['spawned_at'] for b in self.saved['frame']['players']]
        routes = {role: [r for r in rows if r['epoch'] == 1 and r['map'] == self.map_name]
                  for role, rows in self.history.items()}
        events = {role: verify_revives(rows, role, initial) for role, rows in routes.items()}
        assert events['host'] == events['guest'], 'Clients disagree about revival timing'
        assert all(event['player'] == self.victim for event in events['host']), 'Wrong player revived'
        for role, rows in routes.items():
            for row in rows:
                if all(p['health'] > 0 for p in row['players']):
                    verify_presence([row], role)
            for before, after in zip(rows, rows[1:]):
                if all(p['health'] > 0 for r in (before, after) for p in r['players']) \
                        and all(a['spawned_at'] == b['spawned_at'] for a, b in zip(before['players'], after['players'])):
                    verify_presence([before, after], role)
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')
        self.evidence.update(recovery=events, victim=self.victim)
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route_passed=False,
            build=self.build, internet=not self.args.local, victim=self.args.victim,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Real combat death, moving spectator camera and ten-second 25-health revival; no team-wipe/full-route credit'), indent=2))
        print('PASS Castling combat death, moving spectator and ten-second 25-health revival', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--victim', choices=('blue', 'red'), required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    args.first, args.start_step, args.last_step, args.save_before, args.rejoin = 'blue', 77, 77, [], False
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = CastlingRecovery(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, last_valid=pair.last_valid, evidence=pair.evidence)))
        raise
    finally:
        pair.close()
