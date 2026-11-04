"""Drive a fresh Clockwork prefix in independent clients through the real network.

Each player must earn a separate waypoint receipt. The first stops at a native
clear/supported neighbor to let the second arrive. The initial runner supports
nav/walk/wait/lever steps; timed inputs, portals and scene handoffs remain explicit
future work. It never edits authoritative positions, resources or progression.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import struct
import time
import zipfile

from test_coop_play import ROOT, distance
from test_coop_village import Pair
from test_coop_funhouse import waiting_point, combat_pending

LEVERS = (376, 378, 289, 694, 44)
THREADS = ('Gear_Bridge_One', 'Gear_Bridge_Two', 'Open_ClockRoom_Door', 'Open_Port', 'stop_clock')
# Supported footings observed in an ordinary successful entrance descent.
DESCENT = [[-2848.6, 3228.5, 320.03125], [-3070.9, 2936.8, 176.03125],
           [-3231.4, 2929.1, 128.03125], [-3296, 2960, 192.03125]]


def map_entities(base):
    """Read the same override-resolved local map as the native asset mount."""
    raw = None
    for path in sorted((p for p in base.iterdir() if p.suffix.lower() == '.pk3'), key=lambda p: p.name.lower()):
        with zipfile.ZipFile(path) as pack:
            for item in pack.infolist():
                if item.filename.replace('\\', '/').lower() == 'maps/hatter1.bsp':
                    assert item.file_size <= 128 * 1024 * 1024, 'Map exceeds native asset limit'
                    raw = pack.read(item)
    assert raw and raw[:8] == b'FAKK*\0\0\0' and len(raw) >= 172, 'Expected native FAKK42 Clockwork'
    offset, length = struct.unpack_from('<II', raw, 12 + 14 * 8)
    assert offset >= 172 and offset + length <= len(raw), 'Invalid map entity bounds'
    text = raw[offset:offset + length].decode().rstrip('\0')
    entities = [dict(re.findall(r'"([^"\n]*)"\s*"([^"\n]*)"', body))
                for body in re.findall(r'\{([^{}]*)\}', text)]
    assert len(entities) == 982, 'Review changed Clockwork layout'
    for entity, thread in zip(LEVERS, THREADS):
        assert entities[entity]['move_thread'] == thread, 'Wrong native lever'
    return entities, hashlib.sha256(raw).hexdigest()


def verify_arrivals(steps, completed, receipts, history):
    expected = {i for i in completed if next(iter(steps[i])) in ('nav', 'walk')}
    assert {r['step'] for r in receipts} == expected, 'Missing or extra waypoint receipts'
    observed = {role: {r['tick']: r for r in rows} for role, rows in history.items()}
    for index in expected:
        pair = [r for r in receipts if r['step'] == index]
        assert len(pair) == 2 and {r['role'] for r in pair} == {'host', 'guest'}, 'Missing personal arrival'
        goal = next(iter(steps[index].values()))
        for receipt in pair:
            row = observed[receipt['role']].get(receipt['tick'])
            assert row and row['owner'] is None and row['map'] == 'hatter1' and row['epoch'] == 1, 'Arrival not observed in free Clockwork play'
            body = row['players'][0 if receipt['role'] == 'host' else 1]
            assert body['health'] > 0 and body['spawned_at'] == 0 and body['grounded'], 'Invalid arrival body'
            assert receipt['goal'] == goal and receipt['feet'] == body['feet'], 'Receipt differs from observed body'
            assert row['driver']['command'] == receipt['command'] and row['driver']['reached'], 'Arrival command not acknowledged'
            assert distance(body['feet'][:2], goal[:2]) < 8 and abs(body['feet'][2] - goal[2]) < 35, 'Missed personal waypoint'


def parking_target(candidates, at, incoming, goal):
    """Take another observed walking step if the first neighbor ring is too near."""
    direction = [goal[i] - incoming[i] for i in range(2)]
    def ahead(point):
        return sum((point[i] - goal[i]) * direction[i] for i in range(2)) > 8 * math.hypot(*direction)
    usable = [p for p in candidates if distance(p[:2], goal[:2]) >= 48 and distance(p[:2], incoming[:2]) >= 48]
    forward = [p for p in usable if ahead(p)]
    if forward:
        return waiting_point(forward, at, incoming, goal), True
    farther = [p for p in candidates if distance(p[:2], incoming[:2]) >= 48
               and distance(p[:2], goal[:2]) >= distance(at[:2], goal[:2]) + 16]
    forward = [p for p in farther if ahead(p)]
    if forward:
        # A short forward step may need another ring before it leaves the
        # arrival lane clear. Do that before considering an incoming-side spot.
        return max(forward, key=lambda p: distance(p[:2], goal[:2])), False
    if usable:
        return waiting_point(usable, at, incoming, goal), True
    assert farther, 'No observed walking approach to a waiting position'
    return max(farther, key=lambda p: distance(p[:2], goal[:2])), False


def verify_descent(arrivals, jumps, history):
    verify_arrivals([dict(nav=p) for p in DESCENT], list(range(len(DESCENT))), arrivals, history)
    assert len(jumps) == 4 and {(j['role'],j['leg']) for j in jumps} == {
        (r,leg) for r in ('host','guest') for leg in (0,1)}, 'Missing personal entrance jump'
    for jump in jumps:
        slot = 0 if jump['role'] == 'host' else 1
        rows = history[jump['role']]
        before = next(r for r in rows if r['tick'] == jump['before_tick'])
        after = next(r for r in rows if r['tick'] == jump['after_tick'])
        count = before['players'][slot]['jumps']
        assert before['players'][slot]['grounded'] and after['players'][slot]['jumps'] == count + 1, 'Descent needs one actual jump'
        assert any(before['tick'] < r['tick'] <= after['tick'] and r['driver']['command'] == jump['command']
                   for r in rows), 'Entrance jump command unobserved'
        # Actual input/takeoff events are recorded by the authoritative host,
        # including Red's events. The guest supplies its own command receipt.
        events = [e for r in history['host']
                  for e in r['driver'].get('input_events', []) if e['kind'] == 'jump'
                  and before['tick'] <= e['tick'] < after['tick'] and e['epoch'] == 1]
        assert any(e['input'][slot] and e['before'][slot][1] and not e['after'][slot][1]
                   and e['before'][slot][2] == count and e['after'][slot][2] == count + 1 for e in events), 'Actual descent takeoff unobserved'


def verify_ledge_pulls(pulls, history):
    for pull in pulls:
        slot = 0 if pull['role'] == 'host' else 1
        rows = history[pull['role']]
        before = next(r for r in rows if r['tick'] == pull['before_tick'])
        after = next(r for r in rows if r['tick'] == pull['after_tick'])
        a, b = before['players'][slot], after['players'][slot]
        assert a['ledge'] and not a['ledge']['pulling'], 'Pull-up did not begin at an observed ledge'
        assert b['climbs'] == a['climbs'] + 1 and b['grounded'] and b['ledge'] is None, 'Native pull-up did not finish'
        assert any(before['tick'] < r['tick'] <= after['tick'] and r['driver']['command'] == pull['command']
                   and r['players'][slot]['ledge'] and r['players'][slot]['ledge']['pulling'] for r in rows), 'Pull-up animation unobserved'
        assert any(e['kind'] == 'jump' and e['epoch'] == 1 and before['tick'] <= e['tick'] < after['tick']
                   and e['input'][slot] and not e['before'][slot][1]
                   and e['before'][slot][2] == e['after'][slot][2]
                   for r in history['host'] for e in r['driver'].get('input_events', [])), 'Pull-up input unobserved'


class ClockworkPair(Pair):
    map_name, entry, next_map = 'hatter1', 'hatter1_start1', 'hatter2'
    evidence_prefix, capture_scenes = 'hatter1-route-', False

    def __init__(self, args):
        self.step, self.completed, self.last_valid = -1, [], {}
        self.first = 'host' if args.first == 'blue' else 'guest'
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.lead, self.lead_tick = self.minimum_lead, None
        super().__init__(args)
        self.deadline = time.monotonic() + 1500
        raw = (ROOT / 'src/levels/hatter1/route_steps.json').read_bytes()
        self.steps = json.loads(raw)
        assert len(self.steps) == 150 and 0 <= args.last_step < len(self.steps), 'Review changed course'
        assert all(next(iter(s)) in ('nav', 'walk', 'wait', 'lever') for s in self.steps[:args.last_step + 1]), 'Requested prefix needs an unimplemented input type'
        # The native SP course accepts a 22-unit planning radius. At this
        # boundary its marker cannot be approached within our eight-unit
        # arrival bound. Use the separately observed clear, grounded footing.
        assert self.steps[6] == dict(nav=[-3904,2750,208.03125]), 'Review changed entrance marker'
        adjustments = [dict(step=6, original=self.steps[6]['nav'], goal=[-3920,2750,208.03125])]
        assert distance(adjustments[0]['original'], adjustments[0]['goal']) < 22
        self.steps[6] = dict(nav=adjustments[0]['goal'])
        self.entities, map_sha = map_entities(args.data)
        (self.folder / 'course.json').write_bytes(raw)
        effective = json.dumps(self.steps, indent=2).encode()
        (self.folder / 'effective-course.json').write_bytes(effective)
        self.evidence.update(course_sha256=hashlib.sha256(raw).hexdigest(), map_sha256=map_sha,
            effective_course_sha256=hashlib.sha256(effective).hexdigest(), waypoint_adjustments=adjustments,
            executable_sha256=hashlib.sha256(args.exe.read_bytes()).hexdigest(),
            arrivals=[], levers=[], holds=[], parking=[], descent_arrivals=[], descent_jumps=[], ledge_pulls=[],
            completed=self.completed, scope='Fresh Clockwork prefix only')
        dependencies = self.folder / 'fixture-dependencies'; dependencies.mkdir()
        for name in ('test_coop_hatter1.py', 'test_coop_village.py', 'test_coop_play.py', 'test_coop_chess.py',
                     'test_coop_funhouse.py', 'test_coop_castling.py', 'test_coop_chess_resume.py', 'coop_diagnostics.py', 'coop_recovery.py'):
            raw = (Path(__file__).parent / name).read_bytes()
            (dependencies / name).write_bytes(raw)
            self.evidence.setdefault('fixture_sha256', {})[name] = hashlib.sha256(raw).hexdigest()

    def set(self, role, *args, use_aim=None, **kwargs):
        kwargs.setdefault('attack', True)
        if use_aim is not None: kwargs['attack'] = False
        super().set(role, *args, **kwargs)
        self.commands[role].update(input_lead=self.lead if role == 'guest' else 0,
            use_aim=use_aim, watch_dialogue=True, combat_tactics=True,
            prefer_cards=False, prefer_ice=True, avoid_switches=True)

    def body(self, role):
        return self.reports[role]['frame']['players'][0 if role == 'host' else 1]

    def state(self, role):
        return self.reports[role]['combat']['stage']['world']['levels'][self.map_name]

    def validate_report(self, role, report):
        previous = self.last_valid.get(role)
        if previous and any(b['stats']['sanity'] < old['stats']['sanity']
                            for b, old in zip(report['frame']['players'], previous['frame']['players'])):
            path = self.folder / f'damage-{role}-{report["frame"]["tick"]}.json'
            if not path.exists(): path.write_text(json.dumps(dict(before=previous, after=report)))
        super().validate_report(role, report)
        assert report['build'] == self.args.build, 'Executable differs from the frozen build'
        assert report['visit']['map'] == self.map_name and report['visit']['epoch'] == 1, 'Unexpected visit during prefix'
        assert all(b['spawned_at'] == 0 for b in report['frame']['players']), 'Prefix silently respawned a player'
        if report['combat']['stage']['owner'] is None:
            a, b = [x['player']['feet'] for x in report['frame']['players']]
            if distance(a[:2], b[:2]) < 35.95 and abs(a[2] - b[2]) < 55.95:
                (self.folder / f'overlap-{role}.json').write_text(json.dumps(report))
                raise AssertionError('Living players overlap')
        self.last_valid[role] = report

    def pump(self):
        assert not (self.folder / 'stop-test').exists(), 'Clockwork test stopped'
        super().pump()
        guest = self.reports.get('guest')
        if guest and guest['frame']['tick'] != self.lead_tick:
            self.lead_tick = guest['frame']['tick']
            rtt = (guest.get('network_timing') or {}).get('round_trip_ms')
            if rtt is not None and 0 <= rtt < 10000:
                self.lead = max(self.minimum_lead, .8 * self.lead + .2 * min(1.5, .05 + rtt / 1000))
                self.commands['guest']['input_lead'] = self.lead
                self.evidence['last_measured_input_lead'] = self.lead
        for role, report in self.reports.items():
            self.history[role][-1]['clockwork'] = self.state(role)
            for observed, body in zip(self.history[role][-1]['players'], report['frame']['players']):
                observed.update(ledge=body['player'].get('ledge'), climbs=body['player']['climbs'])

    def until(self, predicate, seconds=90):
        end = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= end:
                summary = {r: dict(tick=d['frame']['tick'], feet=[b['player']['feet'] for b in d['frame']['players']],
                    driver={k:d['route_driver'].get(k) for k in ('command','phase','error','reached','replans','expected')})
                    for r, d in self.reports.items()}
                raise AssertionError(f'Clockwork step {self.step} timed out: {summary}')
            self.pump()

    def move(self, role, goal, navigate=False, jump=False, radius=8):
        self.set(role, goal, navigate=navigate, run=True, jump=jump)
        self.commands[role]['arrival_radius'] = radius
        command = self.commands[role]['id']
        pullup = None
        best = (distance(self.body(role)['player']['feet'], goal), self.reports[role]['frame']['tick'])
        def done():
            nonlocal best, navigate, pullup
            body = self.body(role)['player']; report = self.reports[role]
            assert report['combat']['stage']['owner'] is None, 'Unexpected cinematic in ordinary prefix movement'
            gap = distance(body['feet'], goal); tick = report['frame']['tick']
            if self.reached(role) and body['grounded'] and distance(body['feet'][:2], goal[:2]) < radius and abs(body['feet'][2]-goal[2]) < 35:
                return True
            hang = body.get('ledge')
            if hang and not hang['pulling'] and pullup is None and goal[2] >= hang['top'] - 35 and sum(
                    (goal[i]-body['feet'][i])*hang['forward'][i] for i in range(2)) > 0:
                self.set(role, goal, run=True, jump=True)
                self.commands[role]['arrival_radius'] = radius
                pullup = dict(role=role, before_tick=tick, command=self.commands[role]['id'])
                self.evidence['ledge_pulls'].append(pullup)
            if gap < best[0] - 2: best = gap, tick
            elif not navigate and not jump and tick - best[1] > 240 and body['grounded']:
                navigate = True; self.set(role, goal, navigate=True, run=True)
                self.commands[role]['arrival_radius'] = radius
            return False
        self.until(done)
        if pullup is not None: pullup['after_tick'] = self.reports[role]['frame']['tick']
        return command

    def descend(self, role):
        for index, point in enumerate(DESCENT):
            before = self.reports[role]['frame']['tick']
            jumping = index % 2 == 1
            command = self.move(role, point, navigate=not jumping, jump=jumping, radius=2 if index >= 2 else 8)
            report = self.reports[role]
            self.evidence['descent_arrivals'].append(dict(step=index, role=role, goal=point,
                tick=report['frame']['tick'], feet=list(self.body(role)['player']['feet']),
                command=report['route_driver']['command']))
            if jumping:
                self.evidence['descent_jumps'].append(dict(role=role, leg=index//2, command=command,
                    before_tick=before, after_tick=report['frame']['tick']))
        print('PASS ordinary entrance descent', role, flush=True)

    def clear(self):
        if not any(combat_pending(self.reports[r], r) for r in self.commands): return
        for role in self.commands: self.set(role, attack=True)
        quiet = None
        def done():
            nonlocal quiet
            tick = self.reports['host']['frame']['tick']
            if any(combat_pending(self.reports[r], r) for r in self.commands): quiet = None
            elif quiet is None: quiet = tick
            return quiet is not None and tick - quiet >= 60
        self.until(done, 120)

    def go(self, goal, navigate=False):
        first = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], goal) + (0 if r == self.first else 16))
        other = 'guest' if first == 'host' else 'host'
        self.set(other, attack=True)
        for role in (first, other):
            if self.step == 1: self.descend(role)
            self.move(role, goal, navigate)
            report = self.reports[role]
            self.evidence['arrivals'].append(dict(step=self.step, role=role, goal=goal,
                tick=report['frame']['tick'], feet=list(self.body(role)['player']['feet']),
                command=report['route_driver']['command']))
            if role == first:
                slot = 0 if role == 'host' else 1
                if not report['route_driver']['neighbors'][slot]:
                    # An eight-unit arrival can stop on the edge of a small
                    # footing. Center by walking before asking for neighbors.
                    self.move(role, goal, radius=2)
                for attempt in range(3):
                    report = self.reports[role]
                    candidates = report['route_driver']['neighbors'][slot]
                    park, ready = parking_target(candidates, self.body(role)['player']['feet'],
                                                 self.body(other)['player']['feet'], goal)
                    self.evidence['parking'].append(dict(step=self.step, role=role, goal=park, ready=ready,
                        selected_tick=report['frame']['tick'], candidates=candidates))
                    # Stopping eight units short on the entrance's small
                    # waiting footing can obstruct the partner's pull-up.
                    self.move(role, park, radius=2 if self.step == 1 else 8)
                    self.evidence['parking'][-1].update(tick=self.reports[role]['frame']['tick'],
                        command=self.reports[role]['route_driver']['command'], feet=list(self.body(role)['player']['feet']))
                    if ready: break
                else: raise AssertionError('Could not walk far enough to leave the arrival lane clear')
                self.set(role, park, run=True)
        self.set(other, attack=True)

    def lever(self, index):
        self.clear()
        target = [float(v) for v in self.entities[LEVERS[index]]['origin'].split()]
        target[2] += 20
        role = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], target))
        feet = self.body(role)['player']['feet']
        # Movement's native eye offset is 48 units; this is an ordinary use aim.
        eye = [feet[0], feet[1], feet[2] + 48]
        delta = [b-a for a,b in zip(eye, target)]
        assert math.sqrt(sum(v*v for v in delta)) < 130, 'Lever operator needs an ordinary closer approach'
        aim = [math.atan2(delta[1], delta[0]), math.atan2(delta[2], math.hypot(*delta[:2]))]
        assert self.state(role)['levers'][index] is None, 'Lever already consumed'
        self.set(role, use_aim=aim)
        command = self.commands[role]['id']
        self.until(lambda: self.state(role)['levers'][index] is not None and self.state(role).get('pull') is None, 15)
        self.evidence['levers'].append(dict(step=self.step, lever=index, role=role, command=command,
            tick=self.reports[role]['frame']['tick'], time=self.state(role)['levers'][index]))
        self.set(role, attack=True)

    def save_checkpoint(self, step):
        for role in self.commands: self.set(role, attack=False)
        start = self.reports['host']['frame']['tick']
        self.until(lambda: self.reports['host']['frame']['tick'] >= start + 60)
        self.commands['host']['checkpoint'] = True
        command = self.commands['host']['id']
        self.until(lambda: (self.reports['host']['route_driver'].get('checkpoint') or {}).get('command') == command, 15)
        receipt = self.reports['host']['route_driver']['checkpoint']
        assert receipt['epoch'] == 1 and receipt['error'] is None, receipt
        raw = (self.folder / 'host/coop-checkpoint.json').read_bytes()
        saved = json.loads(raw)['payload']
        assert saved['visit']['map'] == self.map_name and saved['visit']['epoch'] == 1
        assert saved['frame']['tick'] == receipt['tick'] and saved['frame']['checkpoint_generation'] == 0
        path = self.folder / f'checkpoint-before-{step:03d}.json'; path.write_bytes(raw)
        self.evidence.setdefault('checkpoints', {})[str(step)] = dict(file=path.name, sha256=hashlib.sha256(raw).hexdigest(), **receipt)
        for role in self.commands: self.set(role)
        print('PASS ordinary checkpoint before step', step, flush=True)

    def verify(self):
        assert self.completed == list(range(self.args.last_step + 1)), 'Incomplete requested prefix'
        verify_arrivals(self.steps, self.completed, self.evidence['arrivals'], self.history)
        if 1 in self.completed:
            verify_descent(self.evidence['descent_arrivals'], self.evidence['descent_jumps'], self.history)
        verify_ledge_pulls(self.evidence['ledge_pulls'], self.history)
        expected = [i for i in self.completed if 'lever' in self.steps[i]]
        assert [r['step'] for r in self.evidence['levers']] == expected, 'Missing lever receipts'
        for receipt in self.evidence['levers']:
            row = next(r for r in self.history[receipt['role']] if r['tick'] == receipt['tick'])
            assert row['clockwork']['levers'][receipt['lever']] == receipt['time'] and row['clockwork'].get('pull') is None
            assert row['driver']['command'] == receipt['command'], 'Lever command not observed'
        assert [r['step'] for r in self.evidence['holds']] == [i for i in self.completed if 'wait' in self.steps[i]], 'Missing timed wait'
        for hold in self.evidence['holds']:
            assert hold['end_tick'] - hold['start_tick'] >= hold['seconds'] * 120, 'Timed wait ended early'
        for park in self.evidence['parking']:
            selected = next(r for r in self.history[park['role']] if r['tick'] == park['selected_tick'])
            slot = 0 if park['role'] == 'host' else 1
            assert park['goal'] in selected['driver']['neighbors'][slot] == park['candidates'], 'Parking was not a native supported neighbor'
            row = next(r for r in self.history[park['role']] if r['tick'] == park['tick'])
            body = row['players'][slot]
            assert body['grounded'] and body['feet'] == park['feet'] and row['driver']['command'] == park['command'], 'Parking arrival unobserved'
            assert distance(body['feet'][:2], park['goal'][:2]) < 8 and abs(body['feet'][2]-park['goal'][2]) < 35
            if park['ready']:
                goal = next(iter(self.steps[park['step']].values()))
                assert distance(park['goal'][:2], goal[:2]) >= 48, 'Waiting target obstructs the arrival lane'
        assert [p['step'] for p in self.evidence['parking'] if p['ready']] == [i for i in self.completed
            if next(iter(self.steps[i])) in ('nav','walk')], 'Missing completed parking handoff'
        for role, rows in self.history.items():
            assert rows and all(r['map'] == self.map_name and r['epoch'] == 1 and r['checkpoint_generation'] == 0 for r in rows)
            assert all(p['health'] > 0 and p['spawned_at'] == 0 for r in rows for p in r['players']), 'Prefix contains a death'
        for field, metric in (('sha256','matching_sent_and_restored_states'), ('world_sha256','matching_world_states'), ('native_sha256','matching_native_states')):
            self.verify_network_states(field, metric)
        self.evidence.update(build=self.build, passed=True)

    def run(self):
        self.start_pair()
        for self.step, step in enumerate(self.steps[:self.args.last_step + 1]):
            print('Clockwork step', self.step, step, flush=True)
            self.clear()
            if self.step in (0, 1, 9, 14, 18, 25, 33): self.save_checkpoint(self.step)
            key, value = next(iter(step.items()))
            if key in ('nav', 'walk'): self.go(value, key == 'nav')
            elif key == 'lever': self.lever(value)
            elif key == 'wait':
                start = self.reports['host']['frame']['tick']; self.wait(value)
                self.evidence['holds'].append(dict(step=self.step, start_tick=start, end_tick=self.reports['host']['frame']['tick'], seconds=value))
            else: raise AssertionError('Unimplemented course action')
            self.completed.append(self.step)
            print('PASS Clockwork prefix step', self.step, flush=True)
        self.save_checkpoint(self.args.last_step + 1)
        self.verify()
        print('PASS fresh Clockwork prefix', self.completed[0], self.completed[-1], self.evidence['matching_sent_and_restored_states'], 'matching states', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue','red'), default='blue')
    parser.add_argument('--last-step', type=int, default=17)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not all(0 <= v <= 500 for v in (args.delay_ms, args.jitter_ms)): parser.error('Delay/jitter must be 0..500 ms')
    pair = ClockworkPair(args)
    try: pair.run()
    except BaseException as error:
        pair.evidence.update(passed=False, failed_step=pair.step, error=str(error))
        raise
    finally: pair.close()


if __name__ == '__main__': main()
