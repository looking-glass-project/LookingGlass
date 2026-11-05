"""Drive Burning Curiosity's approach in two independent internet clients.

Both players earn their own native waypoint receipts and walk to supported
waiting positions. This first course segment ends before the sinking platform;
it does not establish a full visit. Resumes use unchanged ordinary checkpoints.
No positions, health, items or progression are written by the runner.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair
from test_coop_hatter1 import parking_target, recorded_arrival
from test_coop_funhouse import combat_pending
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from coop_recovery import read_trace, verify_presence


ROLES = ('host', 'guest')
ARRIVAL = 'JLair1_Start'


def verify_arrival(receipt, history):
    role = receipt['role']; slot = ROLES.index(role)
    row = next(r for r in history[role] if (r['epoch'], r['tick']) == (receipt['epoch'], receipt['tick']))
    body = row['players'][slot]
    assert row['map'] == 'jlair1' and row['epoch'] == 1 and row['owner'] is None, 'Arrival outside ordinary visit'
    assert row['driver']['command'] == receipt['command'] and row['driver']['reached'], 'Unacknowledged arrival'
    assert body['health'] > 0 and body['grounded'] and row['driver']['clear'][slot], 'Invalid arrival body'
    assert body['spawned_at'] == 0, 'Route silently respawned'
    assert body['feet'] == receipt['feet'], 'Arrival body changed'
    assert distance(body['feet'][:2], receipt['goal'][:2]) < 8 and abs(body['feet'][2] - receipt['goal'][2]) < 35, 'Missed waypoint'


def verify_course(steps, completed, arrivals, history, start, last):
    assert completed == list(range(start, last + 1)), 'Incomplete requested course'
    assert {(r['step'], r['role']) for r in arrivals} == {(s, r) for s in completed for r in ROLES}, 'Missing personal arrival'
    assert len(arrivals) == 2 * len(completed), 'Duplicate personal arrival'
    for receipt in arrivals:
        assert receipt['goal'] == steps[receipt['step']]['pos'], 'Waypoint changed'
        verify_arrival(receipt, history)


def verify_entry(history):
    for role, rows in history.items():
        active = [r for r in rows if r['scene'] == ARRIVAL and r['owner'] is not None]
        assert active and all(r['owner'] == 'Blue' for r in active), 'Missing original arrival owner'
        for r in active:
            assert r['players'][1]['feet'] == active[0]['players'][1]['feet'], 'Arrival moved waiting partner'
            assert not r['presentation']['visible_players'][1], 'Arrival exposed waiting partner'
            assert all(p['health'] == 100 for p in r['players']), 'Fresh arrival grant changed'
        after = next(r for r in rows if r['tick'] > active[-1]['tick'] and r['owner'] is None)
        assert after['curiosity']['arrived'] and after['curiosity']['scene'] is None, 'Arrival did not finish'
        assert all(p['grounded'] and p['health'] == 100 for p in after['players']) and all(after['driver']['clear']), 'Unsafe arrival'
        assert distance(after['players'][0]['feet'], after['players'][1]['feet']) <= 128, 'Arrival did not reunite the pair'


def verify_combat_wait(entry, history):
    assert set(entry['start']) == set(entry['end']) == set(entry['commands']) == set(ROLES)
    for role in ROLES:
        start, end, command = entry['start'][role], entry['end'][role], entry['commands'][role]
        assert end - start >= 60, 'Combat wait has no settled observation interval'
        assert command['epoch'] == 1 and command['goal'] is None and not command['navigate']
        assert command['attack'] and command['combat_tactics'] and command['prefer_cards'] and not command['prefer_ice']
        rows = [r for r in history[role] if start <= r['tick'] <= end]
        assert rows and rows[0]['tick'] == start and rows[-1]['tick'] == end, 'Missing combat wait boundary'
        observed = [r for r in rows if r['driver']['command'] == command['id']]
        assert len(observed) >= 2 and observed[-1] is rows[-1], 'Unacknowledged combat wait'
        assert all(r['epoch'] == 1 and r['map'] == 'jlair1' and r['owner'] is None
            and all(r['driver']['clear']) and all(p['health'] > 0 and p['spawned_at'] == 0 for p in r['players'])
            for r in rows), 'Combat wait changed visit or lost a player'


class CuriosityPair(Pair):
    map_name, entry, next_map = 'jlair1', 'jlair1_start1', 'jlair2'
    evidence_prefix, capture_scenes = 'jlair1-route-', False

    def __init__(self, args):
        self.step, self.completed, self.last_valid = -1, [], {}
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.lead, self.lead_tick = self.minimum_lead, None
        super().__init__(args)
        raw = (ROOT / 'src/levels/jlair1/route_steps.json').read_bytes()
        self.steps = json.loads(raw)
        assert len(self.steps) == 108 and all(s['mode'] == 'nav' for s in self.steps[:53]), 'Review changed original course'
        self.approach_goals = {3: self.steps[2]['pos']}
        (self.folder / 'course.json').write_bytes(raw)
        self.evidence.update(course_sha256=hashlib.sha256(raw).hexdigest(),
            executable_sha256=hashlib.sha256(args.exe.read_bytes()).hexdigest(),
            arrivals=[], parking=[], approaches=[], forward_waiting=[], combat_waits=[], completed=self.completed, fresh=args.checkpoint is None,
            scope='Approach segment only; shared lava crossing, ending and full visit remain unverified')
        deps = self.folder / 'fixture-dependencies'; deps.mkdir()
        for name in ('test_coop_jlair1.py', 'test_coop_hatter1.py', 'test_coop_village.py', 'test_coop_play.py',
                     'test_coop_funhouse.py', 'test_coop_castling.py', 'test_coop_chess.py',
                     'test_coop_chess_resume.py', 'coop_diagnostics.py', 'coop_recovery.py'):
            data = (Path(__file__).parent / name).read_bytes()
            (deps / name).write_bytes(data)
            self.evidence.setdefault('fixture_sha256', {})[name] = hashlib.sha256(data).hexdigest()
        self.saved = None
        if args.checkpoint:
            raw = args.checkpoint.read_bytes(); self.saved = json.loads(raw)['payload']
            assert self.saved['visit']['map'] == self.map_name and self.saved['visit']['epoch'] == 1
            assert self.saved['frame']['checkpoint_generation'] == 0
            assert all(p['spawned_at'] == 0 and p['stats']['sanity'] > 0 for p in self.saved['frame']['players'])
            self.resume_checkpoint = True
            (self.folder / 'starting-checkpoint.json').write_bytes(raw)
            (self.folder / 'host/coop-checkpoint.json').write_bytes(raw)
            self.evidence['checkpoint_sha256'] = hashlib.sha256(raw).hexdigest()

    def set(self, role, *args, radius=8, **kwargs):
        kwargs.setdefault('attack', True)
        super().set(role, *args, **kwargs)
        self.commands[role].update(input_lead=self.lead if role == 'guest' else 0,
            arrival_radius=radius, watch_dialogue=True, combat_tactics=True,
            prefer_cards=False, prefer_ice=True, avoid_switches=True, recovery_trace=role == 'host')

    def body(self, role):
        return self.reports[role]['frame']['players'][ROLES.index(role)]

    def validate_report(self, role, report):
        previous = self.last_valid.get(role)
        if previous and any(b['stats']['sanity'] < old['stats']['sanity']
                            for b, old in zip(report['frame']['players'], previous['frame']['players'])):
            path = self.folder / f'damage-{role}-{report["frame"]["tick"]}.json'
            if not path.exists(): path.write_text(json.dumps(dict(before=previous, after=report)))
        super().validate_report(role, report)
        assert report['build'] == self.args.build, 'Executable differs from frozen build'
        assert (report['visit']['map'], report['visit']['epoch']) == (self.map_name, 1), 'Unexpected visit during segment'
        assert all(not p['stats']['god'] and not p['stats']['notarget'] and p['spawned_at'] == 0
                   for p in report['frame']['players']), 'Cheated or silently respawned route'
        assert report['route_driver'].get('recovery_trace_error') is None
        if report['combat']['stage']['owner'] is None:
            a, b = [p['player']['feet'] for p in report['frame']['players']]
            assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2] - b[2]) >= 55.95, 'Living players overlap'
        self.last_valid[role] = report

    def pump(self):
        assert not (self.folder / 'stop-test').exists(), 'Burning Curiosity test stopped'
        super().pump()
        for role, report in self.reports.items():
            row = self.history[role][-1]
            assert (row['epoch'], row['tick']) == (report['frame']['epoch'], report['frame']['tick'])
            row.setdefault('curiosity', report['combat']['stage']['world']['levels'][self.map_name])
            row.setdefault('presentation', report['presentation'])
        guest = self.reports.get('guest')
        if guest and guest['frame']['tick'] != self.lead_tick:
            self.lead_tick = guest['frame']['tick']
            rtt = (guest.get('network_timing') or {}).get('round_trip_ms')
            if rtt is not None and 0 <= rtt < 10000:
                self.lead = max(self.minimum_lead, .8 * self.lead + .2 * min(1.5, .05 + rtt / 1000))
                self.commands['guest']['input_lead'] = self.lead

    def until(self, predicate, seconds=90):
        end = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= end:
                details = {r: dict(tick=d['frame']['tick'], feet=[p['player']['feet'] for p in d['frame']['players']],
                    driver={k:d['route_driver'].get(k) for k in ('command', 'phase', 'error', 'reached', 'expected')})
                    for r, d in self.reports.items()}
                raise AssertionError(f'Burning Curiosity step {self.step} timed out: {details}')
            self.pump()

    def reached(self, role):
        return super().reached(role) and recorded_arrival(self.history[role][-1], self.reports[role], self.commands[role]['id'])

    def receipt(self, role, goal):
        row = self.history[role][-1]
        entry = dict(step=self.step, role=role, goal=list(goal), epoch=row['epoch'], tick=row['tick'],
            feet=list(self.body(role)['player']['feet']), command=self.commands[role]['id'])
        verify_arrival(entry, self.history)
        return entry

    def move(self, role, goal, navigate=False, radius=8):
        self.set(role, goal, navigate=navigate, run=True, radius=radius)
        self.until(lambda: self.reached(role) and self.body(role)['player']['grounded'])
        return self.receipt(role, goal)

    def clear(self):
        if not any(combat_pending(self.reports[r], r) for r in ROLES): return False
        entry = dict(step=self.step, start={r:self.reports[r]['frame']['tick'] for r in ROLES})
        # A fixed waiting goal prevents the ordinary combat driver from dodging.
        # Resolve currently visible threats before parking on a narrow slope;
        # both clients still spend their own resources and use normal weapons.
        for role in ROLES:
            self.set(role, attack=True)
            # Moving fire-spitters can evade the short Ice stream. Use the
            # original route's homing Cards preference during a stationary
            # fight, with the normal Blade fallback when Will runs out.
            self.commands[role].update(prefer_cards=True, prefer_ice=False)
        entry['commands'] = {r:dict(self.commands[r]) for r in ROLES}
        self.evidence['combat_waits'].append(entry)
        quiet = None
        def done():
            nonlocal quiet
            tick = self.reports['host']['frame']['tick']
            if any(combat_pending(self.reports[r], r) for r in ROLES): quiet = None
            elif quiet is None: quiet = tick
            return quiet is not None and tick - quiet >= 60
        self.until(done, 120)
        entry['end'] = {r:self.reports[r]['frame']['tick'] for r in ROLES}
        print('PASS ordinary combat wait', self.step, 'health', [self.body(r)['stats']['sanity'] for r in ROLES], flush=True)
        return True

    def go(self, goal):
        self.clear()
        # The early crossing starts on a narrow ledge. Return from the waiting
        # bay to the original approach before asking either client to plan the
        # jumps; an offset starting line produced a different lava-side landing.
        approach = self.approach_goals.get(self.step)
        first = min(ROLES, key=lambda r: distance(self.body(r)['player']['feet'], goal)
                    + (0 if r == ('host' if self.args.first == 'blue' else 'guest') else 16))
        if approach is not None:
            first = min(ROLES, key=lambda r: distance(self.body(r)['player']['feet'], approach))
        other = 'guest' if first == 'host' else 'host'
        self.set(other)
        for role in (first, other):
            if approach is not None:
                self.evidence['approaches'].append(self.move(role, approach, navigate=True))
            entry = self.move(role, goal, navigate=True)
            if self.clear():
                entry = self.move(role, goal, navigate=True)
            self.evidence['arrivals'].append(entry)
            if role == first:
                visited = []
                for _ in range(8):
                    report = self.reports[role]; slot = ROLES.index(role)
                    at = list(self.body(role)['player']['feet'])
                    candidates = report['route_driver']['neighbors'][slot]
                    if not candidates:
                        # Steep narrow slopes need not have same-height bays.
                        # Continue by ordinary navigation to the next authored
                        # footing, then let the partner reach this one. Record
                        # that real walk separately; it does not award the
                        # partner an arrival at either point.
                        assert self.step < 52, 'No tested forward waiting footing before the shared platform'
                        next_goal = self.steps[self.step + 1]['pos']
                        selected_tick = report['frame']['tick']
                        entry = self.move(role, next_goal, navigate=True)
                        entry['selected_tick'] = selected_tick
                        self.evidence['forward_waiting'].append(entry)
                        park = next_goal
                        break
                    park, ready = parking_target(candidates, at, self.body(other)['player']['feet'], goal, visited)
                    visited.append(at)
                    entry = dict(selected_tick=report['frame']['tick'], candidates=candidates, ready=ready)
                    entry.update(self.move(role, park, radius=2))
                    self.evidence['parking'].append(entry)
                    if ready: break
                else: raise AssertionError('No clear waiting position for the second player')
                self.set(role, park, run=True)
        self.set(other)

    def checkpoint(self, step):
        for role in ROLES: self.set(role, attack=False)
        start = self.reports['host']['frame']['tick']
        self.until(lambda: self.reports['host']['frame']['tick'] >= start + 60)
        self.commands['host']['checkpoint'] = True; command = self.commands['host']['id']
        self.until(lambda: (self.reports['host']['route_driver'].get('checkpoint') or {}).get('command') == command, 20)
        receipt = self.reports['host']['route_driver']['checkpoint']
        assert receipt['epoch'] == 1 and receipt['error'] is None, receipt
        raw = (self.folder / 'host/coop-checkpoint.json').read_bytes(); saved = json.loads(raw)['payload']
        assert saved['visit']['map'] == self.map_name and saved['frame']['tick'] == receipt['tick']
        path = self.folder / f'checkpoint-before-{step:03d}.json'; path.write_bytes(raw)
        self.evidence.setdefault('checkpoints', {})[str(step)] = dict(file=path.name, sha256=hashlib.sha256(raw).hexdigest(), **receipt)
        for role in ROLES: self.set(role)
        print('PASS ordinary checkpoint before step', step, flush=True)

    def verify(self):
        verify_course(self.steps, self.completed, self.evidence['arrivals'], self.history, self.args.start_step, self.args.last_step)
        for entry in self.evidence['combat_waits']: verify_combat_wait(entry, self.history)
        expected = {(step, role) for step in self.completed if step in self.approach_goals for role in ROLES}
        assert {(r['step'], r['role']) for r in self.evidence['approaches']} == expected
        assert len(self.evidence['approaches']) == len(expected), 'Duplicated approach receipt'
        for entry in self.evidence['approaches']:
            assert entry['goal'] == self.approach_goals[entry['step']]
            verify_arrival(entry, self.history)
        for entry in self.evidence['forward_waiting']:
            assert entry['goal'] == self.steps[entry['step'] + 1]['pos']
            selected = next(r for r in self.history[entry['role']] if r['epoch'] == 1 and r['tick'] == entry['selected_tick'])
            assert selected['driver']['neighbors'][ROLES.index(entry['role'])] == [], 'Forward wait bypassed an available bay'
            verify_arrival(entry, self.history)
        for entry in self.evidence['parking']:
            selected = next(r for r in self.history[entry['role']] if r['epoch'] == 1 and r['tick'] == entry['selected_tick'])
            assert entry['candidates'] == selected['driver']['neighbors'][ROLES.index(entry['role'])]
            assert entry['goal'] in entry['candidates'], 'Waiting target was not an observed safe neighbor'
            verify_arrival(entry, self.history)
        if self.saved is None: verify_entry(self.history)
        for role, rows in self.history.items(): verify_presence(rows, role)
        for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'), ('native_sha256', 'matching_native_states')]:
            self.verify_network_states(field, metric)
        recovery = read_trace(self.folder / 'host/recovery-trace.jsonl', self.build)
        assert not recovery['revivals'], 'Clean segment contains a revival'
        self.evidence['recovery'] = recovery

    def run(self):
        self.start_pair()
        if self.saved:
            receipt = json.loads((self.folder / 'host/route-resume.json').read_text())
            verify_resume_receipt(receipt, self.saved, self.build)
            self.evidence['exact_resume_receipt'] = receipt
            for report in self.reports.values():
                for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                    verify_resume_body(body, old, report['frame']['tick'] - self.saved['frame']['tick'])
        else:
            self.until(lambda: all(r['combat']['stage']['owner'] is None for r in self.reports.values()), 150)
            verify_entry(self.history)
        for self.step in range(self.args.start_step, self.args.last_step + 1):
            if self.step in self.args.save_before: self.checkpoint(self.step)
            (self.folder / f'before-step-{self.step:03d}.json').write_text(json.dumps(self.reports))
            self.go(self.steps[self.step]['pos'])
            self.completed.append(self.step)
            print('PASS personal approach step', self.step, 'health', [self.body(r)['stats']['sanity'] for r in ROLES], flush=True)
        self.checkpoint(self.args.last_step + 1)
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        self.verify()
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route=False,
            internet=not self.args.local, build=self.build, evidence=self.evidence), indent=2))
        print('PASS Burning Curiosity approach segment', self.args.start_step, self.args.last_step, flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--start-step', type=int, default=0)
    parser.add_argument('--last-step', type=int, default=52)
    parser.add_argument('--save-before', type=int, nargs='*', default=[0, 4, 8, 15, 23, 33, 41, 49, 52])
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not 0 <= args.start_step <= args.last_step <= 52: parser.error('This approach segment supports steps 0..52')
    if (args.start_step > 0) != (args.checkpoint is not None): parser.error('A resumed segment requires its unchanged checkpoint and positive start step')
    if not all(0 <= v <= 500 for v in (args.delay_ms, args.jitter_ms)): parser.error('Delay/jitter must be 0..500 ms')
    pair = CuriosityPair(args)
    try: pair.run()
    except BaseException as error:
        pair.evidence.update(passed=False, failed_step=pair.step, error=str(error),
            process_exit_codes_before_cleanup={r:p.poll() for r,p in pair.processes.items()})
        raise
    finally: pair.close()


if __name__ == '__main__': main()
