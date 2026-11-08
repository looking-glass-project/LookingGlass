"""Traverse Castling in independent internet clients using ordinary local inputs.

Run in Anode. Each process uses its own received world, profile and command file.
The fixture advances course steps after both clients finish; it never writes
positions, resources, enemy health or script state. No screenshots are required.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair
from test_coop_chess import ChessPair


def verify_presence(route, role):
    for row in route:
        if row['owner'] is None:
            a, b = [p['feet'] for p in row['players']]
            assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2]-b[2]) >= 55.95, 'Players overlap'
            assert row['camera_player'] == ('Blue' if role == 'host' else 'Red'), 'Wrong personal camera'
    for old, row in zip(route, route[1:]):
        warped = any(a['warped_at'] != b['warped_at'] for a, b in zip(old['players'], row['players']))
        assert not warped or old['owner'] is not None or row['owner'] is not None, 'Unscripted Castling warp'
        if row['owner'] is not None and old['owner'] == row['owner'] and old['scene'] == row['scene']:
            partner = 1 if row['owner'] == 'Blue' else 0
            assert row['players'][partner]['feet'] == old['players'][partner]['feet'], 'Scene moved waiter'


class CastlingPair(Pair):
    map_name = 'wchess2'
    entry = 'wchess2_start1'
    next_map = 'rchess1'
    evidence_prefix = 'castling-route-'
    capture_scenes = False

    def __init__(self, args):
        self.minimum_lead = min(1.5, .05 + 2*(args.delay_ms + args.jitter_ms)/1000)
        self.guest_lead = self.minimum_lead
        self.lead_tick = None
        super().__init__(args)
        self.deadline = time.monotonic() + 1800
        self.course_hashes, self.last_valid = {}, {}
        self.steps, self.completed = None, []
        source = Path(__file__).read_bytes()
        (self.folder / 'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def set(self, role, *args, step=None, **kwargs):
        kwargs['attack'] = False
        super().set(role, *args, **kwargs)
        self.commands[role].update(castling_follow=self.args.first.title() if step is not None else None,
            castling_step=step, input_lead=self.guest_lead if role == 'guest' else 0, watch_dialogue=True)

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Castling test stopped; evidence retained')
        super().pump()
        guest = self.reports.get('guest')
        if guest and guest['frame']['tick'] != self.lead_tick:
            self.lead_tick = guest['frame']['tick']
            rtt = (guest.get('network_timing') or {}).get('round_trip_ms')
            if rtt is not None and 0 <= rtt < 10000:
                self.guest_lead = max(self.minimum_lead, .8*self.guest_lead + .2*min(1.5, .05+rtt/1000))
                self.commands['guest']['input_lead'] = self.guest_lead
                self.evidence['last_measured_input_lead'] = self.guest_lead
        for role, report in self.reports.items():
            course = report['route_driver'].get('castling_course')
            if course and role not in self.course_hashes:
                raw = (self.folder / role / 'wchess2-coop-course.json').read_bytes()
                self.course_hashes[role] = hashlib.sha256(raw).hexdigest()
                assert len(set(self.course_hashes.values())) == 1, 'Clients use different courses'
                self.steps = json.loads(raw)
                (self.folder / 'course.json').write_bytes(raw)
                self.evidence['course_sha256'] = self.course_hashes[role]
            saved = report['combat']['stage']['world']['levels'].get(self.map_name)
            row = self.history[role][-1]
            row['castling'] = {k: saved[k] for k in ('queen', 'king', 'portal', 'scene', 'pending', 'age')} if saved else None
            row['presentation'] = report.get('presentation')
            if saved:
                self.last_valid[role] = report

    def validate_report(self, role, report):
        stage = report['combat']['stage']
        bodies = report['frame']['players']
        if report['visit']['map'] == self.map_name and report['visit']['epoch'] == 1:
            previous = self.last_valid.get(role)
            if previous and any(b['stats']['sanity'] < a['stats']['sanity'] for a, b in zip(previous['frame']['players'], bodies)):
                path = self.folder / f'damage-{role}-{report["frame"]["tick"]}.json'
                if not path.exists():
                    path.write_text(json.dumps(dict(before=previous, after=report)))
            a, b = [body['player']['feet'] for body in bodies]
            if stage['owner'] is None and all(p['stats']['sanity'] > 0 for p in bodies) \
                    and distance(a[:2], b[:2]) < 35.95 and abs(a[2]-b[2]) < 55.95:
                path = self.folder / f'first-overlap-{role}.json'
                if not path.exists():
                    path.write_text(json.dumps(dict(before=previous, after=report,
                        peer=self.reports.get('guest' if role == 'host' else 'host'))))
        super().validate_report(role, report)

    def until(self, predicate, seconds=240):
        end = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= end:
                raise AssertionError('Castling timeout: ' + json.dumps({role: dict(
                    tick=r['frame']['tick'], map=r['visit']['map'], course=r['route_driver'].get('castling_course'),
                    feet=[p['player']['feet'] for p in r['frame']['players']],
                    health=[p['stats']['sanity'] for p in r['frame']['players']]) for role, r in self.reports.items()}))
            self.pump()

    # This is the ordinary pause-menu checkpoint operation, with its native receipt.
    save_checkpoint = ChessPair.save_checkpoint

    def verify(self):
        assert self.steps and len(self.steps) == 203 and self.completed == list(range(len(self.steps))), 'Incomplete course'
        assert len(self.course_hashes) == 2
        self.evidence['completed_steps'] = self.completed
        for role, rows in self.history.items():
            route = [r for r in rows if r['map'] == self.map_name and r['epoch'] == 1]
            assert route
            verify_presence(route, role)
            owners = {}
            for row in route:
                if row['scene'] and row['owner']:
                    owners.setdefault(row['scene'], set()).add(row['owner'])
            assert set(owners) == {'cinema_queen_abduction_thread', 'cinema_king_thread'}, f'Missing original scenes: {owners}'
            assert all(actors == {self.args.first.title()} for actors in owners.values()), f'Wrong first actor: {owners}'
            assert all(r['checkpoint_generation'] == 0 and all(p['health'] > 0 and p['spawned_at'] == 0
                and not p['stats']['god'] and not p['stats']['notarget'] for p in r['players']) for r in rows), 'Death/wipe or cheats'
            last = self.last_valid[role]['combat']['stage']['world']['levels'][self.map_name]
            assert last['queen'] and last['king'] and last['portal'] is not None
            assert self.reports[role]['visit']['entry'] == 'rchess1_start1'
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
            print('PASS Castling step', index, step['mode'], 'tick', self.reports['host']['frame']['tick'],
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
            scope='Full ordinary-input Castling route with watched scenes; recovery, rejoin and skipped scenes remain separate'), indent=2))
        print('PASS full Castling ' + ('local' if self.args.local else 'internet') + ' traversal', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--save-before', nargs='*', type=int, default=[])
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = CastlingPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, last_valid=pair.last_valid, completed=pair.completed)))
        raise
    finally:
        pair.close()
