"""Play Funhouse's opening corridor in independent internet clients.

This runner covers the authored entrance through step 28, before the mirror
switch and machinery. It sends only ordinary inputs and normal save requests.
Every movement waypoint needs a separate arrival receipt for each player.
Full traversal, recovery, machinery and the exit remain separate work.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair
from test_coop_chess import ChessPair
from test_coop_castling import verify_presence
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt


def verify_arrivals(steps, completed, receipts, history):
    """Require each claimed arrival to match an immutable client observation."""
    observed = {role: {row['tick']: row for row in rows} for role, rows in history.items()}
    expected = {step for step in completed if not steps[step].get('mode')}
    assert {r['step'] for r in receipts} == expected, 'Missing or extra movement receipts'
    for step in expected:
        entries = [r for r in receipts if r['step'] == step]
        assert len(entries) == 2 and {r['role'] for r in entries} == {'host', 'guest'}, 'Missing personal arrival'
        for receipt in entries:
            role = receipt['role']
            row = observed[role].get(receipt['tick'])
            assert row and row['owner'] is None, 'Arrival was not observed during free play'
            body = row['players'][0 if role == 'host' else 1]
            goal = steps[step]['pos']
            assert receipt['goal'] == goal and receipt['feet'] == body['feet'], 'Arrival differs from observed body'
            assert body['grounded'] and body['health'] > 0 and body['spawned_at'] == 0, 'Invalid arrival body'
            assert distance(body['feet'][:2], goal[:2]) < 8 and abs(body['feet'][2] - goal[2]) < 35, 'Missed waypoint'


class FunhousePair(Pair):
    map_name, entry, next_map = 'funhouse', 'funhouse_start1', 'hatter1'
    evidence_prefix, capture_scenes = 'funhouse-route-', False

    def __init__(self, args):
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.lead, self.lead_tick = self.minimum_lead, None
        self.step, self.completed = -1, []
        self.first = 'host' if args.first == 'blue' else 'guest'
        super().__init__(args)
        self.deadline = time.monotonic() + 1200
        raw = (ROOT / 'src/levels/funhouse/route_steps.json').read_bytes()
        self.steps = json.loads(raw)
        assert len(self.steps) == 1077, 'Review changed authored course'
        assert all(s.get('mode', '') in ('', 'wait', 'clear')
                   for s in self.steps[args.start_step:args.last_step + 1])
        (self.folder / 'course.json').write_bytes(raw)
        self.evidence.update(course_sha256=hashlib.sha256(raw).hexdigest(),
            executable_sha256=hashlib.sha256(args.exe.read_bytes()).hexdigest(), arrivals=[])
        dependencies = self.folder / 'fixture-dependencies'
        dependencies.mkdir()
        for name in ('test_coop_funhouse.py', 'test_coop_play.py', 'test_coop_village.py',
                     'test_coop_chess.py', 'test_coop_castling.py', 'test_coop_chess_resume.py', 'coop_diagnostics.py'):
            raw = (Path(__file__).parent / name).read_bytes()
            (dependencies / name).write_bytes(raw)
            self.evidence.setdefault('fixture_sha256', {})[name] = hashlib.sha256(raw).hexdigest()
        self.saved = None
        if args.checkpoint:
            raw = args.checkpoint.read_bytes()
            self.saved = json.loads(raw)['payload']
            assert self.saved['visit']['map'] == self.map_name and self.saved['visit']['epoch'] == 1
            assert self.saved['frame']['checkpoint_generation'] == 0
            self.resume_checkpoint = True
            (self.folder / 'host' / 'coop-checkpoint.json').write_bytes(raw)
            self.evidence.update(checkpoint_sha256=hashlib.sha256(raw).hexdigest(),
                                 checkpoint_tick=self.saved['frame']['tick'])

    def set(self, role, *args, **kwargs):
        kwargs.setdefault('attack', bool(args and args[0] is not None or kwargs.get('goal') is not None))
        super().set(role, *args, **kwargs)
        self.commands[role].update(input_lead=self.lead if role == 'guest' else 0,
            watch_dialogue=True, skip_scene=self.args.skip_scenes, prefer_cards=True)

    def body(self, role):
        return self.reports[role]['frame']['players'][0 if role == 'host' else 1]

    def busy(self):
        return any(r['combat']['stage']['owner'] is not None for r in self.reports.values())

    def state(self, role):
        return self.reports[role]['combat']['stage']['world']['levels'][self.map_name]

    def retain(self, name):
        (self.folder / (name + '.json')).write_text(json.dumps(dict(step=self.step,
            completed=self.completed, reports=self.reports, commands=self.commands, evidence=self.evidence)))

    def validate_report(self, role, report):
        super().validate_report(role, report)
        bodies = report['frame']['players']
        assert all(not b['stats']['god'] and not b['stats']['notarget'] for b in bodies), 'Cheated route'
        assert all(b['spawned_at'] == 0 for b in bodies), 'Strict route silently revived a player'
        if report['combat']['stage']['owner'] is None:
            a, b = [p['player']['feet'] for p in bodies]
            assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2] - b[2]) >= 55.95, 'Players overlap'
            assert report['presentation']['visible_players'][1 if role == 'host' else 0], 'Free-play partner hidden'

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Funhouse stopped; evidence retained')
        super().pump()
        guest = self.reports.get('guest')
        if guest and guest['frame']['tick'] != self.lead_tick:
            self.lead_tick = guest['frame']['tick']
            rtt = (guest.get('network_timing') or {}).get('round_trip_ms')
            if rtt is not None and 0 <= rtt < 10000:
                self.lead = max(self.minimum_lead, .8*self.lead + .2*min(1.5, .05+rtt/1000))
                self.commands['guest']['input_lead'] = self.lead
        for role, report in self.reports.items():
            row = self.history[role][-1]
            row.setdefault('presentation', report['presentation'])
            row.setdefault('funhouse', self.state(role))

    def until(self, predicate, seconds=90):
        end = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= end:
                self.retain('stalled')
                raise AssertionError('Funhouse stalled at step ' + str(self.step) + ': ' + json.dumps({
                    role: dict(tick=r['frame']['tick'], phase=r['route_driver']['phase'],
                        feet=self.body(role)['player']['feet'], health=self.body(role)['stats']['sanity'])
                    for role, r in self.reports.items()}))
            self.pump()

    def scene_end(self):
        for role in self.commands:
            self.set(role)
        self.until(lambda: not self.busy(), 120)

    def move(self, role, goal, navigate=False):
        self.set(role, goal, navigate=navigate, run=True)
        best = (distance(self.body(role)['player']['feet'], goal), self.reports[role]['frame']['tick'])
        def done():
            nonlocal best, navigate
            if self.reached(role) or self.busy():
                return True
            body, tick = self.body(role)['player'], self.reports[role]['frame']['tick']
            gap = distance(body['feet'], goal)
            if gap < best[0] - 2:
                best = gap, tick
            elif not navigate and tick - best[1] > 240 and body['grounded']:
                # Match the authored route's normal path-planning fallback.
                navigate = True
                self.set(role, goal, navigate=True, run=True)
            return False
        self.until(done)
        assert not self.busy(), 'Unexpected scene during opening corridor movement'
        body = self.body(role)['player']
        assert body['grounded'], 'Waypoint has no supported landing'
        assert distance(body['feet'][:2], goal[:2]) < 8 and abs(body['feet'][2] - goal[2]) < 35

    def park(self, role, other, goal):
        """Walk to a native clear/supported neighbor, keeping the approach open."""
        slot = 0 if role == 'host' else 1
        at, incoming = self.body(role)['player']['feet'], self.body(other)['player']['feet']
        direction = [goal[i] - incoming[i] for i in range(2)]
        length = max(1, distance(goal[:2], incoming[:2]))
        candidates = [p for p in self.reports[role]['route_driver']['neighbors'][slot]
                      if distance(p[:2], goal[:2]) >= 40 and distance(p[:2], incoming[:2]) >= 40]
        assert candidates, 'No safe ordinary walking space beside waypoint'
        def score(point):
            offset = [point[i] - at[i] for i in range(2)]
            lateral = abs(offset[0]*direction[1] - offset[1]*direction[0]) / length
            return lateral + .1*distance(point[:2], incoming[:2])
        parking = max(candidates, key=score)
        self.move(role, parking)
        self.set(role, attack=True)
        self.until(lambda: self.reports[role]['route_driver']['command'] == self.commands[role]['id']
                   and distance(self.body(role)['player']['velocity'][:2], [0, 0]) < 1, 20)
        assert distance(self.body(role)['player']['feet'][:2], goal[:2]) >= 36

    def go(self, goal):
        # On a return along a narrow hall, the player already in front leads.
        # Record the real order; do not make the bodies pass through each other.
        first = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], goal)
                    + (0 if r == self.first else 16))
        other = 'guest' if first == 'host' else 'host'
        self.set(other, attack=True)
        for role in (first, other):
            self.move(role, goal)
            report = self.reports[role]
            self.evidence['arrivals'].append(dict(step=self.step, role=role, goal=goal,
                tick=report['frame']['tick'], feet=self.body(role)['player']['feet'],
                command=report['route_driver']['command']))
            if role == first:
                self.park(first, other, goal)
        self.set(other, attack=True)

    def clear(self):
        for role in self.commands:
            self.set(role, attack=True)
        since = None
        def cleared():
            nonlocal since
            quiet = True
            for role, report in self.reports.items():
                driver, slot = report['route_driver'], 0 if role == 'host' else 1
                quiet &= driver['command'] == self.commands[role]['id'] and driver['native_threats'][slot] is None
                for kind in ('guards', 'boojums', 'snarks'):
                    quiet &= not any(e['active'] and e['health'] > 0 and e['distance'][slot] <= 700
                                     and e['line_of_sight'][slot] for e in driver[kind])
            tick = self.reports['host']['frame']['tick']
            since = (tick if since is None else since) if quiet else None
            return since is not None and tick - since >= 120
        self.until(cleared, 150)

    save_checkpoint = ChessPair.save_checkpoint

    def verify(self):
        assert self.completed == list(range(self.args.start_step, self.args.last_step + 1))
        verify_arrivals(self.steps, self.completed, self.evidence['arrivals'], self.history)
        for role, rows in self.history.items():
            verify_presence(rows, role)
            if not self.saved:
                opening = [r for r in rows if r['scene'] == 'Funhouse_Start']
                assert opening and {r['owner'] for r in opening} == {'Blue'}, 'Missing original opening owner'
                assert all(not r['presentation']['visible_players'][1] for r in opening), 'Opening displayed partner'
                assert any(r['players'][1]['warped_at'] for r in rows), 'No real opening reunion'
            assert rows[-1]['owner'] is None and rows[-1]['funhouse']['arrived'], 'Opening did not finish'
        for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'),
                              ('native_sha256', 'matching_native_residents')]:
            self.verify_network_states(field, metric)

    def run(self):
        self.start_pair()
        if self.saved:
            receipt = json.loads((self.folder / 'host' / 'route-resume.json').read_text())
            verify_resume_receipt(receipt, self.saved, self.build)
            self.evidence['exact_resume_receipt'] = receipt
            for report in self.reports.values():
                assert report['visit'] == self.saved['visit']
                elapsed = report['frame']['tick'] - self.saved['frame']['tick']
                for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                    verify_resume_body(body, old, elapsed)
        self.scene_end()
        for self.step in range(self.args.start_step, self.args.last_step + 1):
            self.retain(f'before-{self.step:03d}')
            if self.step in self.args.save_before:
                self.save_checkpoint(self.step)
            step = self.steps[self.step]
            mode = step.get('mode', '')
            if mode == 'wait':
                self.wait(step['frames'] / 120)
            elif mode == 'clear':
                self.clear()
            else:
                self.go(step['pos'])
            self.completed.append(self.step)
            print('PASS Funhouse step', self.step, mode or 'walk', 'health',
                  [self.body(r)['stats']['sanity'] for r in self.commands], flush=True)
        self.save_checkpoint(self.args.last_step + 1)
        self.verify()
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        self.evidence['completed_steps'] = self.completed
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route=False,
            through_exit=False, build=self.build, internet=not self.args.local,
            first=self.args.first, skip_scenes=self.args.skip_scenes,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            scope=f'Funhouse steps {self.args.start_step}..{self.args.last_step}; both personal arrivals; no deaths',
            fresh_entrance=self.saved is None,
            evidence=self.evidence), indent=2))
        print('PASS Funhouse opening segment; full route remains open', flush=True)

    def close(self):
        try:
            self.retain('last-reports')
        finally:
            super().close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--last-step', type=int, default=28)
    parser.add_argument('--start-step', type=int, default=0)
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--save-before', nargs='*', type=int, default=[])
    parser.add_argument('--skip-scenes', action='store_true')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not 0 <= args.start_step <= args.last_step <= 28 or args.last_step < 1:
        parser.error('The opening runner currently supports steps 0..28')
    if bool(args.checkpoint) != (args.start_step > 0):
        parser.error('A later start requires an unchanged checkpoint; a fresh run starts at zero')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = FunhousePair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False,
            error=str(error), build=pair.build, step=pair.step, completed=pair.completed)))
        raise
    finally:
        pair.close()
