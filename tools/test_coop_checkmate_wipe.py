"""Verify an ordinary Checkmate combat wipe against an unchanged shared save.

Two clients walk into the northern encounter without attacking. Only normal
input and checkpoint loading are used; no resources or progression are set.
"""
import argparse
import hashlib
import json
from pathlib import Path

from coop_recovery import verify_presence
from test_coop_checkmate import CheckmatePair
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from test_coop_play import distance
from test_coop_village import verify_rejoin_stats


def verify_wipe_trace(rows, saved, build):
    """Prove the exact common spawn boundary, separately from individual revival."""
    previous, event, count = None, None, 0
    for row in rows:
        count += 1
        assert row['serial'] == count and row['version'] == 1, 'Missing/reordered wipe trace'
        assert row['build'] == build and row['epoch'] == saved['visit']['epoch'], 'Wrong wipe trace scope'
        generation = row['checkpoint_generation']
        assert generation in (0, 1), 'Unexpected additional team wipe'
        assert all(not p['god'] and not p['notarget'] for p in row['players']), 'Wipe used cheats'
        if previous:
            assert row['tick'] > previous['tick'], 'Wipe clock moved backwards'
            pending = any(isinstance(p['life'], dict) or p['life'] == 'AwaitingSafeSpawn'
                          for p in previous['players'])
            assert row['tick']-previous['tick'] <= (1 if pending else 120), 'Missing wipe boundary ticks'
        if generation == 0:
            assert event is None, 'Wipe generation moved backwards'
            assert all(p['spawned_at'] == old['spawned_at']
                       for p, old in zip(row['players'], saved['frame']['players'])), 'Individual revival preceded this wipe'
        elif event is None:
            assert previous and previous['checkpoint_generation'] == 0, 'Missing pre-wipe state'
            assert row['tick'] == previous['tick']+1, 'Missing exact wipe boundary'
            waiting = [p for p in previous['players'] if isinstance(p['life'], dict)]
            assert waiting and any(0 < p['life']['Spectating']['ticks_left'] <= 1200 for p in waiting), 'No survivor countdown before wipe'
            assert not row['cinematic'] and not row['spawn_held'], 'Wipe restored inside a controlled scene'
            for body, old in zip(row['players'], saved['frame']['players']):
                assert body['life'] == 'Alive' and body['spawned_at'] == row['tick'], 'Wipe did not restore both together'
                assert body['feet'] == old['player']['feet'], 'Wrong exact checkpoint placement'
                assert body['warped_at'] == old['warped_at'], 'Wipe changed scene stamp'
                assert body['stats'] == old['stats'], 'Wipe changed saved personal resources'
                assert body['health'] == old['stats']['sanity'] > 0, 'Wrong checkpoint health'
                geometry = body['geometry']
                assert geometry and all(geometry[k] for k in ('clear', 'valid', 'safe')), 'Unsafe checkpoint restore'
                assert geometry['supported'] or geometry['swimming'] or geometry['rope'], 'Unsupported checkpoint restore'
            event = dict(tick=row['tick'], generation=1, exact_players=True)
        previous = row
    assert event is not None, 'No shared wipe was recorded'
    return dict(**event, trace_rows=count)


def verify_restore(saved, report, role):
    frame = report['frame']
    assert frame['checkpoint_generation'] == 1 and report['visit'] == saved['visit']
    stamps = [b['spawned_at'] for b in frame['players']]
    assert stamps[0] == stamps[1] > saved['frame']['tick'], 'Wrong shared spawn stamp'
    elapsed = frame['tick']-stamps[0]
    assert 0 <= elapsed <= 120, 'Missed restored client boundary'
    assert frame['lives']['states'] == ['Alive', 'Alive']
    assert report['camera_player'] == ('Blue' if role == 'host' else 'Red'), 'Wipe left spectator camera active'
    for body, old in zip(frame['players'], saved['frame']['players']):
        verify_rejoin_stats(old['stats'], body['stats'], elapsed)
        assert distance(body['player']['feet'], old['player']['feet']) <= 2+104*elapsed/120, 'Wrong restored client placement'
    old = saved['frame']['combat']['stage']['world']['levels']['rchess1']
    new = report['combat']['stage']['world']['levels']['rchess1']
    for field in ('ambushed', 'beheaded', 'phase', 'checkpoint', 'exit', 'essence'):
        assert old[field] == new[field], 'Wipe changed shared progression: '+field
    assert old['boss']['health'] == new['boss']['health'], 'Wipe changed saved King health'
    def pieces(state):
        result = {entity: (piece['kind'], piece['variant'], piece['health']) for entity, piece in state['pieces']}
        assert len(result) == len(state['pieces']), 'Duplicate native piece'
        return result
    assert pieces(old) == pieces(new), 'Wipe did not restore native pieces'
    return stamps[0]


class CheckmateWipe(CheckmatePair):
    def __init__(self, args):
        self.dying, self.restored, self.observed_death = False, {}, set()
        super().__init__(args)
        raw = Path(__file__).read_bytes()
        (self.folder / 'wipe-fixture-source.py').write_bytes(raw)
        self.evidence['wipe_fixture_sha256'] = hashlib.sha256(raw).hexdigest()

    def validate_report(self, role, report):
        if not self.dying:
            return super().validate_report(role, report)
        frame = report['frame']
        assert frame['checkpoint_generation'] in (0, 1), 'Unexpected additional team wipe'
        assert report['route_driver'].get('recovery_trace_error') is None, 'Host trace failed'
        assert all(not b['stats']['god'] and not b['stats']['notarget'] for b in frame['players'])
        if report['combat']['stage']['owner'] is None and all(b['stats']['sanity'] > 0 for b in frame['players']):
            a, b = [body['player']['feet'] for body in frame['players']]
            assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2]-b[2]) >= 55.95, 'Living players overlap'
        if frame['checkpoint_generation'] == 0:
            if any(b['stats']['sanity'] <= 0 for b in frame['players']):
                self.observed_death.add(role)
        elif role not in self.restored:
            (self.folder / f'restored-{role}.json').write_text(json.dumps(report))
            self.restored[role] = verify_restore(self.saved, report, role)
            for client in self.commands:
                self.set(client, attack=False)

    def run(self):
        self.start_pair()
        receipt = json.loads((self.folder / 'host' / 'route-resume.json').read_text())
        verify_resume_receipt(receipt, self.saved, self.build)
        self.evidence['exact_resume_receipt'] = receipt
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit']
            elapsed = report['frame']['tick']-self.saved['frame']['tick']
            for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                verify_resume_body(body, old, elapsed)
        self.dying = True
        self.step = 33
        self.set('host', [1792, 4480, 392], navigate=True, attack=False)
        self.set('guest', [1728, 4480, 392], navigate=True, attack=False)
        self.until(lambda: len(self.restored) == 2, 180)
        assert self.observed_death == {'host', 'guest'}, 'Both clients must observe the combat death'
        assert self.restored['host'] == self.restored['guest'], 'Clients disagree on restore tick'
        self.wait(.5)
        for role, rows in self.history.items():
            for generation in (0, 1):
                group = [r for r in rows if r['checkpoint_generation'] == generation]
                assert group, 'Missing client history across wipe'
                verify_presence(group, role, generation=generation)
        for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'),
                              ('native_sha256', 'matching_native_residents')]:
            self.verify_network_states(field, metric)
        for role in ('guest', 'host'):
            self.set(role, finish=True, attack=False)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        with (self.folder / 'host' / 'recovery-trace.jsonl').open() as source:
            self.evidence['wipe'] = verify_wipe_trace((json.loads(line) for line in source), self.saved, self.build)
        assert self.evidence['wipe']['tick'] == self.restored['host']
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route=False,
            internet=not self.args.local, build=self.build, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Real Checkmate combat wipe, exact saved players and both clients restored; no full-route credit'), indent=2))
        print('PASS Checkmate real team wipe and shared checkpoint restore in both clients', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    args.first, args.start_step, args.last_step, args.save_before = 'blue', 33, 33, []
    args.skip_scenes, args.allow_revival, args.require_revival, args.through_exit = False, True, False, False
    pair = CheckmateWipe(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, evidence=pair.evidence)))
        raise
    finally:
        pair.close()
