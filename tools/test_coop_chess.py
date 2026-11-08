"""Traverse Pale Realm in two independent clients through the public connection.

The fixture chooses the next authored step after both clients finish the current
one. Each client derives its own controls from its received world. No positions,
resources, puzzle answers or progression are written into authoritative state.
Watched scenes only; skipped scenes and recovery require separate evidence.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair


def verify_spike_exit_progress(route, steps):
    """A completed timed corridor must not be retraced on the following leg."""
    for row in route:
        index = (row['driver'].get('chess_course') or {}).get('step')
        if index is None or index == 0 or steps[index - 1]['mode'] != 'spikes':
            continue
        previous = steps[index - 1]
        direction = [b - a for a, b in zip(previous['hold'][:2], previous['pos'][:2])]
        length = sum(v * v for v in direction) ** .5
        assert length > 0, 'Timed corridor has no crossing direction'
        for player in row['players']:
            along = sum((p - goal) * d / length for p, goal, d in
                        zip(player['feet'][:2], previous['pos'][:2], direction))
            assert along >= -80.5, 'Follower retraced the completed spike corridor'


class ChessPair(Pair):
    map_name = 'wchess1'
    entry = 'wchess1_start1'
    next_map = 'wchess2'
    evidence_prefix = 'chess-route-'
    capture_scenes = False

    def __init__(self, args):
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.guest_lead = self.minimum_lead
        self.lead_tick = None
        super().__init__(args)
        self.deadline = time.monotonic() + 1800
        self.course_hashes = {}
        self.steps = None
        self.last_valid = {}
        self.completed = []
        source = Path(__file__).read_bytes()
        (self.folder / 'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def set(self, role, *args, step=None, **kwargs):
        kwargs['attack'] = False
        super().set(role, *args, **kwargs)
        self.commands[role].update(
            chess_follow=self.args.first.title() if step is not None else None,
            chess_step=step,
            input_lead=self.guest_lead if role == 'guest' else 0,
            watch_dialogue=True)

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Pale Realm test stopped; evidence retained')
        super().pump()
        guest = self.reports.get('guest')
        if guest and guest['frame']['tick'] != self.lead_tick:
            self.lead_tick = guest['frame']['tick']
            rtt = (guest.get('network_timing') or {}).get('round_trip_ms')
            if rtt is not None and 0 <= rtt < 10000:
                observed = min(1.5, .05 + rtt / 1000)
                self.guest_lead = max(self.minimum_lead, .8 * self.guest_lead + .2 * observed)
                self.commands['guest']['input_lead'] = self.guest_lead
                self.evidence['last_measured_input_lead'] = self.guest_lead
        for role, report in self.reports.items():
            course = report['route_driver'].get('chess_course')
            if course and role not in self.course_hashes:
                raw = (self.folder / role / 'wchess1-coop-course.json').read_bytes()
                self.course_hashes[role] = hashlib.sha256(raw).hexdigest()
                assert len(set(self.course_hashes.values())) == 1, 'Clients use different courses'
                self.steps = json.loads(raw)
                (self.folder / 'course.json').write_bytes(raw)
                self.evidence['course_sha256'] = self.course_hashes[role]
            saved = report['combat']['stage']['world']['levels'].get('wchess1')
            row = self.history[role][-1]
            row['chess'] = {k: saved[k] for k in ('bishop_done', 'knight_done', 'bell', 'water',
                'board', 'pull', 'elevator', 'coop', 'scene', 'pending', 'age')} if saved else None
            row['presentation'] = report.get('presentation')
            if saved:
                self.last_valid[role] = report

    def until(self, predicate, seconds=240):
        end = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= end:
                raise AssertionError('Pale Realm timeout: ' + json.dumps({role: dict(
                    tick=r['frame']['tick'], map=r['visit']['map'],
                    course=r['route_driver'].get('chess_course'),
                    feet=[p['player']['feet'] for p in r['frame']['players']],
                    health=[p['stats']['sanity'] for p in r['frame']['players']])
                    for role, r in self.reports.items()}))
            self.pump()

    def record_damage(self, role, report):
        previous = self.last_valid.get(role)
        if previous and any(b['stats']['sanity'] < old['stats']['sanity']
                            for old, b in zip(previous['frame']['players'], report['frame']['players'])):
            path = self.folder / f'damage-{role}-{report["frame"]["tick"]}.json'
            if not path.exists():
                path.write_text(json.dumps(dict(before=previous, after=report)))

    def validate_report(self, role, report):
        self.record_overlap(role, report)
        self.record_damage(role, report)
        super().validate_report(role, report)

    def record_overlap(self, role, report):
        """Keep the first complete contact failure for a native replay."""
        stage = report['combat']['stage']
        saved = stage['world']['levels'].get(self.map_name)
        if (report['visit']['map'] != self.map_name or report['visit']['epoch'] != 1
                or saved is None or stage['owner'] is not None
                or self.interactive_owner({'chess': saved}) is not None):
            return
        bodies = report['frame']['players']
        if any(b['stats']['sanity'] <= 0 for b in bodies):
            return
        a, b = [body['player']['feet'] for body in bodies]
        if distance(a[:2], b[:2]) >= 35.95 or abs(a[2] - b[2]) >= 55.95:
            return
        path = self.folder / f'first-overlap-{role}.json'
        if not path.exists():
            path.write_text(json.dumps(dict(before=self.last_valid.get(role),
                after=report, peers=self.reports)))
        # The unchanged final presence assertion decides the test result.

    def close(self):
        # Keep native enemy outcomes on successful runs too; reaching a course
        # boundary alone must not be mistaken for defeating every opponent.
        try:
            (self.folder / 'last-reports.json').write_text(json.dumps(self.reports))
        finally:
            super().close()

    @staticmethod
    def interactive_owner(row):
        saved = row['chess']
        paired = saved['coop'] or {}
        # The saved actor remains after a puzzle. Only an active board/reunion
        # hides the partner; ordinary play must still check both bodies.
        return paired.get('actor') if saved['board'] is not None or paired.get('reunion') else None

    def verify_presence(self, route, role):
        verify_spike_exit_progress(route, self.steps)
        for row in route:
            if row['owner'] is None and self.interactive_owner(row) is None:
                a, b = [p['feet'] for p in row['players']]
                assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2]-b[2]) >= 55.95, 'Players overlap'
                assert row['camera_player'] == ('Blue' if role == 'host' else 'Red')
        for old, row in zip(route, route[1:]):
            changed = any(a['warped_at'] != b['warped_at'] for a, b in zip(old['players'], row['players']))
            assert not changed or old['owner'] is not None or row['owner'] is not None \
                or self.interactive_owner(old) is not None, 'Unscripted Pale Realm warp'
            actor = self.interactive_owner(row)
            if actor is not None and actor == self.interactive_owner(old):
                assert row['players'][1-actor]['feet'] == old['players'][1-actor]['feet'], 'Puzzle moved waiter'

    def save_checkpoint(self, step):
        """Retain an unchanged normal save for a later focused segment test."""
        for role in self.commands:
            self.set(role)
        self.wait(.5)
        self.commands['host']['checkpoint'] = True
        command = self.commands['host']['id']
        self.until(lambda: (self.reports['host']['route_driver'].get('checkpoint') or {}).get('command') == command, 15)
        receipt = self.reports['host']['route_driver']['checkpoint']
        assert receipt['epoch'] == 1 and receipt['error'] is None, receipt
        saved = (self.folder / 'host' / 'coop-checkpoint.json').read_bytes()
        payload = json.loads(saved)['payload']
        assert payload['visit']['map'] == self.map_name and payload['visit']['epoch'] == 1
        assert payload['frame']['tick'] == receipt['tick'] and payload['frame']['checkpoint_generation'] == 0
        path = self.folder / f'checkpoint-before-{step:03d}.json'
        path.write_bytes(saved)
        self.evidence.setdefault('checkpoints', {})[str(step)] = dict(
            file=path.name, sha256=hashlib.sha256(saved).hexdigest(), **receipt)
        print('PASS ordinary checkpoint before step', step, flush=True)

    def verify(self):
        assert self.steps and self.completed == list(range(len(self.steps))), 'Incomplete course'
        assert len(self.course_hashes) == 2
        self.evidence['completed_steps'] = self.completed
        for role, rows in self.history.items():
            route = [r for r in rows if r['map'] == self.map_name and r['epoch'] == 1]
            assert route
            self.verify_presence(route, role)
            owners = {}
            for row in route:
                if row['scene'] and row['owner']:
                    owners.setdefault(row['scene'], set()).add(row['owner'])
            assert len(owners) == 5, f'Missing original scenes: {owners}'
            for scene, actors in owners.items():
                assert actors == {'Blue' if scene == 'cinema_wchess1_intro_thread' else self.args.first.title()}, \
                    f'Wrong scene actor for {scene}: {actors}'
            for old, row in zip(route, route[1:]):
                if row['owner'] is not None and old['owner'] == row['owner'] and old['scene'] == row['scene']:
                    partner = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][partner]['feet'] == old['players'][partner]['feet'], 'Scene moved waiter'
            assert all(r['checkpoint_generation'] == 0 and all(p['health'] > 0 and p['spawned_at'] == 0
                and not p['stats']['god'] and not p['stats']['notarget'] for p in r['players']) for r in rows)
            assert all(any(r['players'][i]['swimming'] for r in route) for i in (0, 1)), 'Missing paired swim'
            last = self.last_valid[role]['combat']['stage']['world']['levels']['wchess1']
            assert last['bishop_done'] and last['knight_done'] and last['water'] is not None
            assert self.reports[role]['visit']['entry'] == 'wchess2_start1'
            assert any(r['map'] == self.next_map and r['epoch'] == 2 for r in rows)
            for before, after in zip(route[-1]['players'], self.reports[role]['frame']['players']):
                for field in ('sanity', 'owned', 'collected', 'school_items', 'staff_component'):
                    assert before['stats'][field] == after['stats'][field], 'Arrival changed ' + field
            self.evidence[role] = dict(owners={k: sorted(v) for k, v in owners.items()}, ticks=route[-1]['tick'])
        assert self.processes['host'].pid != self.processes['guest'].pid
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')

    def run(self):
        self.start_pair()
        for role in self.commands:
            self.set(role, step=0)
        self.until(lambda: len(self.course_hashes) == 2, 20)
        assert all(0 < step < len(self.steps) for step in self.args.save_before), 'Invalid checkpoint step'
        for index, step in enumerate(self.steps):
            if index in self.args.save_before:
                self.save_checkpoint(index)
            (self.folder / f'before-step-{index:03d}.json').write_text(json.dumps(self.reports['host']))
            if index:
                for role in self.commands:
                    self.set(role, step=index)
            self.until(lambda: self.arrived() or all(self.reached(role) for role in self.commands), 240)
            self.completed.append(index)
            print('PASS Pale Realm step', index, step['mode'], 'tick', self.reports['host']['frame']['tick'],
                  'health', [p['stats']['sanity'] for p in self.reports['host']['frame']['players']], flush=True)
            if self.arrived():
                break
        self.until(self.arrived, 60)
        self.verify()
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=2)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route_passed=True,
            build=self.build, internet=not self.args.local, first=self.args.first,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Full ordinary-input Pale Realm route with watched scenes; recovery, rejoin and skipped scenes remain separate'), indent=2))
        print('PASS full Pale Realm ' + ('local' if self.args.local else 'internet') + ' traversal', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--save-before', nargs='*', type=int, default=[],
                        help='Retain ordinary co-op saves before these steps for later segment tests')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = ChessPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, last_valid=pair.last_valid, completed=pair.completed)))
        raise
    finally:
        pair.close()
