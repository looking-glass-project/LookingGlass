"""Verify a real two-player combat wipe and shared normal checkpoint restore.

Both clients approach the saved Pawn encounter using ordinary movement without
attacking. Native encounter replay separately checks the exact atomic restore;
this test verifies the state subsequently observed over the public connection.
No health, enemy, inventory or position values are changed by the fixture.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_chess_resume import ChessResume
from test_coop_play import ROOT, distance
from test_coop_village import verify_rejoin_stats


def verify_restore(saved, report, role):
    frame = report['frame']
    assert frame['checkpoint_generation'] == saved['frame']['checkpoint_generation']+1
    stamps = [b['spawned_at'] for b in frame['players']]
    assert stamps[0] == stamps[1] and stamps[0] > saved['frame']['tick'], 'Wipe did not respawn both together'
    elapsed = frame['tick']-stamps[0]
    assert 0 <= elapsed <= 120, 'Missed the checkpoint restoration boundary'
    assert frame['lives']['states'] == ['Alive', 'Alive']
    assert report['visit'] == saved['visit'], 'Wipe restored another visit'
    assert report['camera_player'] == ('Blue' if role == 'host' else 'Red'), 'Wipe left spectator camera active'
    subject = report['camera_subject_feet']
    assert distance(report['camera_target'][:2], subject[:2]) < 1, 'Wipe camera missed its rendered subject'
    assert distance(subject, frame['players'][0 if role == 'host' else 1]['player']['feet']) < 24, 'Wipe camera followed the other body'
    for body, old in zip(frame['players'], saved['frame']['players']):
        verify_rejoin_stats(old['stats'], body['stats'], elapsed)
        # Inputs can remain in flight briefly after the shared restore. Bound
        # displacement by ordinary walking during the observed elapsed ticks.
        assert distance(body['player']['feet'], old['player']['feet']) <= 2+104*elapsed/120, 'Wrong checkpoint placement'
    original = saved['frame']['combat']['stage']['world']['levels']['wchess1']
    current = report['combat']['stage']['world']['levels']['wchess1']
    for field in ('bishop_done', 'knight_done', 'fired', 'water'):
        assert current[field] == original[field], 'Wipe changed shared progression: '+field
    health = lambda state: [(a['entity'], a['instance'], a['piece']['health']) for a in state['cast']]
    assert health(current) == health(original), 'Wipe did not restore the native encounter'
    return stamps[0]


class ChessWipe(ChessResume):
    evidence_prefix = 'chess-wipe-'

    def __init__(self, args):
        self.dying = False
        self.restored = {}
        self.observed_death = set()
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'wipe-fixture-source.py').write_bytes(source)
        self.evidence['wipe_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def validate_report(self, role, report):
        if not self.dying:
            return super().validate_report(role, report)
        self.record_damage(role, report)
        frame = report['frame']
        assert all(not b['stats']['god'] and not b['stats']['notarget'] for b in frame['players'])
        assert frame['checkpoint_generation'] in (0, 1), 'Unexpected additional team wipe'
        if frame['checkpoint_generation'] == 0:
            if any(b['stats']['sanity'] <= 0 for b in frame['players']):
                self.observed_death.add(role)
        elif role not in self.restored:
            self.restored[role] = verify_restore(self.saved, report, role)
            for client in self.commands:
                self.set(client)

    def run(self):
        self.start_pair()
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit']
            elapsed = report['frame']['tick']-self.saved['frame']['tick']
            for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                assert distance(body['player']['feet'], old['player']['feet']) < 2
                verify_rejoin_stats(old['stats'], body['stats'], elapsed)
        self.dying = True
        for role in self.commands:
            self.set(role, goal=[2312, 1440, 256])
        self.until(lambda: len(self.restored) == 2, 60)
        assert self.observed_death == {'host', 'guest'}, 'Missing observed combat death before restore'
        assert self.restored['host'] == self.restored['guest'], 'Clients disagree about shared respawn tick'
        self.wait(.3)
        for role, rows in self.history.items():
            for row in rows:
                if all(p['health'] > 0 for p in row['players']):
                    self.verify_presence([row], role)
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')
        self.evidence['restore_ticks'] = self.restored
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True,
            full_route_passed=False, build=self.build, internet=not self.args.local,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Real combat team wipe and shared checkpoint restore; no full-route credit'), indent=2))
        print('PASS real team wipe and shared checkpoint restore in both clients', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    args.first, args.start_step, args.last_step, args.save_before = 'blue', 126, 126, []
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = ChessWipe(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False,
            error=str(error), build=pair.build, reports=pair.reports, evidence=pair.evidence)))
        raise
    finally:
        pair.close()
