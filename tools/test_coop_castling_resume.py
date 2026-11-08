"""Resume an unchanged Castling checkpoint, optionally rejoin, then traverse a segment.

Both clients use the normal save/invite paths and ordinary inputs. A guest
rejoin starts a new process. Segment evidence never counts as a full traversal.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_castling import CastlingPair, verify_presence
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from test_coop_play import ROOT


def verify_castling_rejoin(before, report):
    old_stage, stage = before['combat']['stage'], report['combat']['stage']
    old, new = [s['world']['levels']['wchess2'] for s in (old_stage, stage)]
    assert old['queen'] and old['king'] and old['portal'] is not None, 'Rejoin must follow both royal scenes'
    assert old_stage['owner'] is None and stage['owner'] is None, 'Rejoin entered a scene'
    for state in (old, new):
        assert state['scene'] is None and state['pending'] is None, 'Rejoin started a royal handoff'
    for field in ('queen', 'queen_release', 'king', 'portal'):
        assert new[field] == old[field], 'Rejoin changed Castling progression: ' + field
    assert set(old['fired']) <= set(new['fired']), 'Rejoin forgot fired events'
    assert stage['story']['completed'] >= old_stage['story']['completed'], 'Rejoin reset completed scenes'
    assert set(old_stage['story']['seen']) <= set(stage['story']['seen']), 'Rejoin forgot seen scenes'

    def residents(state):
        result = {}
        for actor in state['cast']:
            key = actor['entity'], actor['instance']
            assert key not in result, 'Duplicate native resident'
            piece = actor['piece']
            result[key] = (actor['model'], actor['white'], actor['counted'],
                           piece['kind'], piece['variant'], piece['health'])
        return result
    # The retained exit approach is outside active combat. Idle animation and
    # portal clocks may advance, but actors and their health must not reset.
    assert residents(old) == residents(new), 'Rejoin changed native residents or health'


class CastlingResume(CastlingPair):
    evidence_prefix = 'castling-resume-'
    resume_checkpoint = True

    def __init__(self, args):
        raw = args.checkpoint.read_bytes()
        self.saved = json.loads(raw)['payload']
        assert self.saved['visit']['map'] == self.map_name and self.saved['visit']['epoch'] == 1
        assert self.saved['frame']['checkpoint_generation'] == 0
        super().__init__(args)
        (self.folder / 'host' / 'coop-checkpoint.json').write_bytes(raw)
        self.evidence.update(checkpoint_sha256=hashlib.sha256(raw).hexdigest(),
                             checkpoint_tick=self.saved['frame']['tick'])
        source = Path(__file__).read_bytes()
        (self.folder / 'resume-fixture-source.py').write_bytes(source)
        self.evidence['resume_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def verify_rejoined_world(self, before, report):
        verify_castling_rejoin(before, report)

    def verify_network_states(self, field='sha256', metric='matching_sent_and_restored_states'):
        super().verify_network_states(field, metric)
        if 'rejoin' not in self.evidence:
            return
        event = self.evidence['rejoin']
        states = {role: {(s['epoch'], s['tick']): s[field]
                         for row in rows for s in row['driver']['network_states']
                         if s['epoch'] == event['epoch'] and s['tick'] >= event['after_tick']}
                  for role, rows in self.history.items()}
        shared = states['host'].keys() & states['guest'].keys()
        assert len(shared) >= 20, 'Too few matching states after rejoin'
        assert all(states['host'][key] == states['guest'][key] for key in shared), 'Rejoined client state differs'
        self.evidence['rejoin_' + metric] = len(shared)

    def run(self):
        self.start_pair()
        receipt = json.loads((self.folder / 'host' / 'route-resume.json').read_text())
        verify_resume_receipt(receipt, self.saved, self.build)
        self.evidence['exact_resume_receipt'] = receipt
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit'], 'Resume changed visit'
            elapsed = report['frame']['tick'] - self.saved['frame']['tick']
            for current, old in zip(report['frame']['players'], self.saved['frame']['players']):
                verify_resume_body(current, old, elapsed)
        if self.args.rejoin:
            self.rejoin_guest()
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
            print('PASS resumed Castling step', index, flush=True)
            if self.arrived():
                break
        assert self.completed == list(range(self.args.start_step, self.args.last_step + 1))
        if self.args.last_step == len(self.steps)-1:
            self.until(self.arrived, 60)
        for role, rows in self.history.items():
            route = [r for r in rows if r['map'] == self.map_name and r['epoch'] == 1]
            verify_presence(route, role)
            assert all(r['checkpoint_generation'] == 0 and all(p['health'] > 0
                and p['spawned_at'] == old['spawned_at'] and not p['stats']['god'] and not p['stats']['notarget']
                for p, old in zip(r['players'], self.saved['frame']['players'])) for r in rows), 'Death, wipe or cheats'
            if self.args.last_step == len(self.steps)-1:
                assert self.reports[role]['visit']['entry'] == 'rchess1_start1'
                for before, after in zip(route[-1]['players'], self.reports[role]['frame']['players']):
                    for field in ('sanity', 'owned', 'collected', 'school_items', 'staff_component'):
                        assert before['stats'][field] == after['stats'][field], 'Arrival changed ' + field
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')
        self.evidence['completed_steps'] = self.completed
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=self.reports[role]['visit']['epoch'])
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route_passed=False,
            build=self.build, internet=not self.args.local, first=self.args.first, rejoin=self.args.rejoin,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Unchanged checkpoint resume, optional fresh guest rejoin and segment traversal; no full-route credit'), indent=2))
        print('PASS resumed Castling segment' + (' with a fresh guest rejoin' if self.args.rejoin else ''), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--start-step', type=int, required=True)
    parser.add_argument('--last-step', type=int, required=True)
    parser.add_argument('--save-before', nargs='*', type=int, default=[])
    parser.add_argument('--rejoin', action='store_true')
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 < args.start_step <= args.last_step <= 202):
        parser.error('Choose a nonzero segment ending at or before step 202')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = CastlingResume(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, last_valid=pair.last_valid, completed=pair.completed)))
        raise
    finally:
        pair.close()
