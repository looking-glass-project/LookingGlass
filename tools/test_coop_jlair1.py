"""Drive Burning Curiosity's approach in two independent internet clients.

Both players earn their own native waypoint receipts and walk to supported
waiting positions. The platform extension checks actual boarding, waiting and
ordinary crossing jumps; it does not establish a full visit. Resumes use unchanged ordinary checkpoints.
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
PLATFORM_LEFT = [-7048, 698, 1632]
PLATFORM_RIGHT = [-7000, 698, 1632]
PLATFORM_BANK = [-7208, 768, 1632.03125]
PLATFORM_PARK = [-7256, 768, 1632.03125]
NAV_SECTIONS = ((0, 52), (55, 76), (77, 85), (87, 90), (98, 106))
# Observed ordinary footings along the ascent from 66 back to the upper shelf.
# Separate short searches avoid a remote search exhausting its state limit.
# The previous direct plan caught the bank edge. This observed higher ledge
# gives the guest a supported descending approach.
SHELF_APPROACH = ([-6020,3210,1996.4535], [-6236.352,3366.2947,2000.03125],
                  [-6313.79,3264.695,2088.03125], [-6381.0264,3216.554,2088.03125],
                  [-6474.3486,3274.1404,2208.03125])


def forward_limit(step):
    for first, last in NAV_SECTIONS:
        if first <= step < last:
            return last
    raise AssertionError('No forward waiting beyond this navigation section')


def waiting_target(candidates, at, incoming, goal, visited):
    # A narrow slope may offer only the footing we just left. Every branch of
    # parking_target needs an unvisited neighbor clear of the incoming body.
    # When that ring is exhausted, look for a safe bay along the next approach.
    available = [p for p in candidates if distance(p[:2], at[:2]) >= 16
                 and distance(p[:2], incoming[:2]) >= 48
                 and all(distance(p[:2], old[:2]) >= 16 for old in visited)]
    return parking_target(candidates, at, incoming, goal, visited) if available else None


def escape_target(rows, slot, before):
    """Use a recent native safe neighbor; never plan while standing in lava."""
    at = before['players'][slot]['feet']; other = before['players'][1-slot]['feet']
    candidates = [(r['tick'], p) for r in rows if 0 <= before['tick'] - r['tick'] <= 480
        for p in r['driver']['neighbors'][slot]
        if 48 <= distance(p[:2], at[:2]) <= 160 and p[2] <= at[2] + 80
        and distance(p[:2], other[:2]) >= 48]
    return min(candidates, key=lambda item: (item[1][2] < at[2] + 20, distance(item[1], at), -item[0])) if candidates else None


def forward_waiting_target(row, slot, goal):
    body = row['players'][slot]
    if (not body['grounded'] or body['immersion']['kind'] != 'Dry'
            or distance(body['velocity'], [0,0,0]) >= 20 or not row['driver']['clear'][slot]):
        return None
    return waiting_target(row['driver']['neighbors'][slot], body['feet'],
                          row['players'][1-slot]['feet'], goal, [])


def hot_approach_target(row, slot, goal):
    other = row['players'][1-slot]['feet']
    candidates = [p for p in row['driver']['neighbors'][slot]
        if 64 <= distance(p[:2], goal[:2]) <= 96 and p[2] >= goal[2] + 20
        and distance(p[:2], other[:2]) >= 48]
    return min(candidates, key=lambda p: distance(p, goal)) if candidates else None


def replannable_intermediate(row, slot):
    body = row['players'][slot]
    return (row['owner'] is None and row['driver']['phase'] == 'settling intermediate landing'
            and row['driver']['clear'][slot] and body['grounded']
            and body['immersion']['kind'] == 'Dry' and body['health'] > 0)


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
        # The single-player route's point 54 is a sloping transit point, not a
        # game trigger. For paired crossing, land on the actual adjacent bank.
        goal = PLATFORM_BANK if receipt['step'] == 54 else steps[receipt['step']]['pos']
        assert receipt['goal'] == goal, 'Waypoint changed'
        verify_arrival(receipt, history)


def verify_shelf_approaches(entries, history, completed, approaches, arrivals):
    expected = {(role, index) for role in ROLES for index in range(len(SHELF_APPROACH))} if 65 in completed else set()
    assert {(e['role'],e['index']) for e in entries} == expected and len(entries) == len(expected), 'Incomplete or repeated shelf approach'
    for entry in entries:
        assert entry['step'] == 65 and entry['goal'] == SHELF_APPROACH[entry['index']], 'Changed shelf approach'
        verify_arrival(entry, history)
    if 65 in completed:
        for role in ROLES:
            lower = next(e for e in approaches if e['step'] == 65 and e['role'] == role)
            upper = next(e for e in arrivals if e['step'] == 65 and e['role'] == role)
            ordered = sorted((e for e in entries if e['role'] == role),key=lambda e:e['index'])
            ticks = [lower['tick']] + [e['tick'] for e in ordered] + [upper['tick']]
            assert all(a < b for a,b in zip(ticks,ticks[1:])), 'Shelf approach reordered or bypassed the final ascent'


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


def verify_jump_flight(before, flight, arrival, slot, command):
    assert before['tick'] < flight['tick'] < arrival['tick'], 'Jump flight was not between takeoff and landing'
    assert before['players'][slot]['grounded'] and arrival['players'][slot]['grounded']
    assert flight['driver']['command'] == arrival['driver']['command'] == command, 'Jump command was not acknowledged'
    assert not flight['players'][slot]['grounded'], 'Jump never left the ground'
    assert flight['players'][slot]['jumps'] == arrival['players'][slot]['jumps'] == before['players'][slot]['jumps'] + 1, 'Missing ordinary jump'


def verify_forward_approaches(entry, steps, history):
    passed = entry.get('passed_goals', [])
    final = entry['step'] + len(passed) + 1
    assert final <= forward_limit(entry['step']) and entry['navigation_goal'] == steps[final]['pos'], 'Forward waiting skipped an original approach'
    tick, command = entry['navigation_start'], -1
    for offset, receipt in enumerate(passed, 1):
        assert receipt['role'] == entry['role'] and receipt['step'] == entry['step'], 'Wrong forward approach owner'
        assert receipt['goal'] == steps[entry['step'] + offset]['pos'], 'Forward waiting skipped an original approach'
        assert tick < receipt['tick'] < entry['selected_tick'], 'Forward approaches were not walked in order'
        assert command < receipt['command'] < entry['navigation_command'], 'Forward approach command was not observed in order'
        verify_arrival(receipt, history)
        tick, command = receipt['tick'], receipt['command']


def verify_platform_wait(entry, history):
    slot = ROLES.index(entry['rider'])
    for role in ROLES:
        start, end = entry['start'][role], entry['end'][role]
        rows = [r for r in history[role] if start <= r['tick'] <= end]
        assert end - start >= 120 and rows and rows[0]['tick'] == start and rows[-1]['tick'] == end, 'Missing platform wait interval'
        for row in rows:
            platform = row['driver']['curiosity_platform']
            assert platform['full'][slot] and not platform['riders'][1-slot], 'Wait did not have one fully boarded player'
            assert platform['sink'] == 0 and not platform['descending'], 'Platform departed without the partner'
            assert all(p['health'] > 0 and p['spawned_at'] == 0 for p in row['players']) and all(row['driver']['clear']), 'Unsafe platform wait'


def verify_platform_carry(entry, history):
    slot = ROLES.index(entry['rider'])
    before, after = [next(r for r in history['host'] if r['tick'] == entry[k]) for k in ('before', 'after')]
    a, b = before['driver']['curiosity_platform'], after['driver']['curiosity_platform']
    assert before['tick'] < after['tick'] and 0 <= a['sink'] < b['sink'] <= 128, 'No observed platform descent'
    assert (a['sink'] == 0 or a['descending']) and b['descending'] and a['riders'][slot] and b['riders'][slot], 'Missing actual platform contact'
    delta = before['players'][slot]['feet'][2] - after['players'][slot]['feet'][2]
    assert abs(delta - (b['sink'] - a['sink'])) < .2, 'Rider did not follow the actual platform'
    assert all(p['health'] > 0 and p['spawned_at'] == 0 for p in after['players']) and all(after['driver']['clear']), 'Unsafe carry'


def verify_platform_climb(entry, history):
    slot = ROLES.index(entry['role'])
    before, active, after = [next(r for r in history[entry['role']] if r['tick'] == entry[k]) for k in ('before', 'active', 'after')]
    assert before['tick'] < active['tick'] < after['tick'], 'Missing climb interval'
    assert active['driver']['command'] == entry['command'], 'Unacknowledged climb input'
    a, b, c = [r['players'][slot] for r in (before, active, after)]
    assert b['climb'] is not None and c['climb'] is None and c['grounded'], 'Climb never completed'
    assert b['climbs'] == c['climbs'] == a['climbs'] + 1 and c['jumps'] == a['jumps'], 'Missing ordinary climb'
    assert distance(c['feet'][:2], b['climb']['end'][:2]) < 8 and abs(c['feet'][2] - b['climb']['end'][2]) < 35, 'Missed climb landing'


def verify_bank_wait(entry, history):
    slot = ROLES.index(entry['role'])
    for peer in ROLES:
        rows = [r for r in history[peer] if entry['start'][peer] <= r['tick'] <= entry['end'][peer]]
        assert rows and rows[0]['tick'] == entry['start'][peer] and rows[-1]['tick'] == entry['end'][peer]
        assert rows[-1]['tick'] - rows[0]['tick'] >= 60, 'Missing bank waiting interval'
        for r in rows:
            body = r['players'][slot]
            assert body['grounded'] and r['driver']['clear'][slot] and body['immersion']['kind'] == 'Dry', 'Unstable bank waiting body'
            assert body['health'] == rows[0]['players'][slot]['health'] > 0 and body['spawned_at'] == 0
            assert distance(body['feet'], PLATFORM_PARK) < 8, 'Waiting player left the bank'


def verify_recovered_planning_errors(history, exits):
    for role, rows in history.items():
        for row in rows:
            error = row['driver'].get('error')
            if error is None: continue
            assert error.startswith('No route after '), 'Unrecoverable driver failure'
            assert any(e['role'] == role and abs(e['before_tick'] - row['tick']) <= 240
                       and row['tick'] < e['tick'] for e in exits), 'Planning error has no verified escape'


class CuriosityPair(Pair):
    map_name, entry, next_map = 'jlair1', 'jlair1_start1', 'jlair2'
    evidence_prefix, capture_scenes = 'jlair1-route-', False

    def __init__(self, args):
        self.step, self.completed, self.last_valid = -1, [], {}
        self.navigation_timeout = getattr(args, 'navigation_timeout', 90)
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.lead, self.lead_tick = self.minimum_lead, None
        super().__init__(args)
        raw = (ROOT / 'src/levels/jlair1/route_steps.json').read_bytes()
        self.steps = json.loads(raw)
        assert len(self.steps) == 108 and all(s['mode'] == 'nav' for s in self.steps[:53]), 'Review changed original course'
        assert all(s['mode'] == 'nav' for s in self.steps[55:77]), 'Review changed post-platform course'
        # The upper shelf is approached around its eastern side. The original
        # single-player path passes this lower point on the way up; both
        # players still earn separate ordered arrivals at 65 and then 66.
        self.approach_goals = {3: self.steps[2]['pos'], 65: self.steps[66]['pos']}
        (self.folder / 'course.json').write_bytes(raw)
        self.evidence.update(course_sha256=hashlib.sha256(raw).hexdigest(),
            executable_sha256=hashlib.sha256(args.exe.read_bytes()).hexdigest(),
            arrivals=[], parking=[], approaches=[], shelf_approaches=[], forward_waiting=[], hazard_exits=[], hazard_jumps=[], combat_waits=[],
            platform_moves=[], platform_waits=[], platform_carries=[], platform_jumps=[], platform_climbs=[], bank_waits=[],
            crossing_exit_goal=PLATFORM_BANK,
            navigation_timeout=self.navigation_timeout,
            completed=self.completed, fresh=args.checkpoint is None,
            scope='Approach/platform segment only; ending and full visit remain unverified')
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
        self.commands[role]['avoid_liquid_damage'] = bool(self.commands[role]['navigate']
            and self.commands[role]['goal'] != self.steps[38]['pos'])
        self.commands[role]['settle_landings'] = self.commands[role]['navigate']

    def body(self, role):
        return self.reports[role]['frame']['players'][ROLES.index(role)]

    def recoverable_driver_error(self, role, report):
        driver = report['route_driver']
        body = report['frame']['players'][ROLES.index(role)]
        return (isinstance(driver['error'], str) and driver['error'].startswith('No route after ')
            and body['stats']['sanity'] > 0 and body['player']['grounded']
            and body['player']['immersion']['kind'] in ('Lava', 'Slime'))

    def validate_lives(self, role, report):
        super().validate_report(role, report)
        assert all(p['spawned_at'] == 0 for p in report['frame']['players']), 'Route silently respawned'

    def validate_report(self, role, report):
        previous = self.last_valid.get(role)
        if previous and any(b['stats']['sanity'] < old['stats']['sanity']
                            for b, old in zip(report['frame']['players'], previous['frame']['players'])):
            path = self.folder / f'damage-{role}-{report["frame"]["tick"]}.json'
            if not path.exists(): path.write_text(json.dumps(dict(before=previous, after=report)))
        self.validate_lives(role, report)
        assert report['build'] == self.args.build, 'Executable differs from frozen build'
        assert (report['visit']['map'], report['visit']['epoch']) == (self.map_name, 1), 'Unexpected visit during segment'
        assert all(not p['stats']['god'] and not p['stats']['notarget']
                   for p in report['frame']['players']), 'Cheated route'
        assert report['route_driver'].get('recovery_trace_error') is None
        if report['combat']['stage']['owner'] is None and all(p['stats']['sanity'] > 0 for p in report['frame']['players']):
            a, b = [p['player']['feet'] for p in report['frame']['players']]
            assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2] - b[2]) >= 55.95, 'Living players overlap'
        self.last_valid[role] = report

    def pump(self):
        assert not (self.folder / 'stop-test').exists(), 'Burning Curiosity test stopped'
        super().pump()
        for role, report in self.reports.items():
            row = self.history[role][-1]
            assert (row['epoch'], row['tick']) == (report['frame']['epoch'], report['frame']['tick'])
            row.setdefault('curiosity', report['combat']['stage']['world']['levels'].get(self.map_name))
            row.setdefault('presentation', report['presentation'])
            for observed, body in zip(row['players'], report['frame']['players']):
                observed.setdefault('ledge', body['player'].get('ledge'))
                observed.setdefault('climb', body['player'].get('climb'))
                observed.setdefault('climbs', body['player']['climbs'])
                observed.setdefault('immersion', body['player']['immersion'])
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

    def move(self, role, goal, navigate=False, radius=8, recover_liquid=True):
        if navigate and goal == self.steps[38]['pos']:
            return self.hot_crossing(role, goal)
        for _ in range(4):
            self.set(role, goal, navigate=navigate, run=True, radius=radius)
            resting = None
            def arrived():
                nonlocal resting
                body = self.body(role)['player']
                if body['grounded'] and (self.reached(role) or recover_liquid and body['immersion']['kind'] in ('Lava', 'Slime')):
                    return True
                row = self.history[role][-1]; slot = ROLES.index(role)
                if navigate and replannable_intermediate(row, slot):
                    at = row['players'][slot]['feet']
                    if resting is None or distance(at, resting[1]) >= 2:
                        resting = (row['tick'], list(at))
                    elif row['tick'] - resting[0] >= 240:
                        # An approximate intermediate point can lie just beyond
                        # a slope's reachable resting position. Replan from the
                        # actual grounded body; keep the final arrival unchanged.
                        self.evidence.setdefault('navigation_replans', []).append(
                            dict(step=self.step, role=role, start=resting[0], tick=row['tick'], feet=at))
                        self.set(role, goal, navigate=True, run=True, radius=radius)
                        print('REPLAN stationary intermediate', self.step, role, flush=True)
                        resting = None
                else:
                    resting = None
                return False
            self.until(arrived, self.navigation_timeout if navigate else 90)
            if self.reached(role):
                return self.receipt(role, goal)
            # A live jump can miss a ledge. Use a recently observed safe pad
            # and an ordinary jump before planning again in damaging liquid.
            self.escape_hazard(role)
        raise AssertionError('Movement repeatedly fell into damaging liquid')

    def hot_crossing(self, role, goal):
        slot = ROLES.index(role); choice = None
        self.set(role, goal, navigate=True, run=True)
        navigation = self.commands[role]['id']
        def found():
            nonlocal choice
            row = self.history[role][-1]
            if row['driver']['command'] != navigation or not row['players'][slot]['grounded']: return False
            pad = hot_approach_target(row, slot, goal)
            if pad is not None:
                choice = row['tick'], pad
                return True
            assert row['players'][slot]['immersion']['kind'] == 'Dry', 'Hot approach reached lava before a safe takeoff'
            return False
        self.until(found, self.navigation_timeout)
        selected_tick, pad = choice
        approach = self.move(role, pad, radius=2)
        before = self.history[role][-1]
        assert before['players'][slot]['immersion']['kind'] == 'Dry'
        count = before['players'][slot]['jumps']
        self.set(role, goal, run=True, jump=True)
        command = self.commands[role]['id']
        self.until(lambda: self.body(role)['player']['jumps'] == count + 1 and not self.body(role)['player']['grounded'], 15)
        flight = self.history[role][-1]['tick']
        self.until(lambda: self.reached(role) and self.body(role)['player']['grounded'], 20)
        receipt = self.receipt(role, goal)
        self.evidence['hazard_jumps'].append(dict(step=self.step, role=role, selected_tick=selected_tick,
            navigation_command=navigation, approach=approach, command=command, flight_tick=flight, arrival=receipt))
        return receipt

    def escape_hazard(self, role):
        slot = ROLES.index(role); before = self.history[role][-1]
        if before['players'][slot]['immersion']['kind'] not in ('Lava', 'Slime'): return
        selected = escape_target(self.history[role], slot, before)
        assert selected is not None, 'No recently observed safe footing beside the hot waypoint'
        tick, goal = selected
        count = before['players'][slot]['jumps']
        self.set(role, goal, run=True, jump=True, radius=2)
        command = self.commands[role]['id']
        self.until(lambda: (self.body(role)['player']['jumps'] == count + 1 and not self.body(role)['player']['grounded'])
                   or self.body(role)['player'].get('climb') is not None, 15)
        if self.body(role)['player'].get('climb') is not None:
            climb = dict(role=role, before=before['tick'], active=self.history[role][-1]['tick'], command=command)
            self.set(role, attack=False)
            self.until(lambda: self.body(role)['player'].get('climb') is None and self.body(role)['player']['grounded'], 15)
            climb['after'] = self.history[role][-1]['tick']
            verify_platform_climb(climb, self.history)
            assert self.body(role)['player']['immersion']['kind'] == 'Dry', 'Hazard climb did not reach dry ground'
            entry = self.move(role, goal, radius=2)
            entry['climb'] = climb
        else:
            flight = self.history[role][-1]['tick']
            self.until(lambda: self.reached(role) and self.body(role)['player']['grounded'], 20)
            entry = self.receipt(role, goal)
            entry['flight_tick'] = flight
        entry.update(before_tick=before['tick'], selected_tick=tick)
        assert self.body(role)['player']['immersion']['kind'] == 'Dry', 'Hazard exit did not reach dry ground'
        self.evidence['hazard_exits'].append(entry)
        print('PASS ordinary hazard exit', self.step, role, flush=True)

    def forward_bay(self, role, goal):
        last = forward_limit(self.step)
        start = self.history[role][-1]['tick']
        passed = []
        for next_step in range(self.step + 1, last + 1):
            next_goal = self.steps[next_step]['pos']
            for _ in range(4):
                self.set(role, next_goal, navigate=True, run=True)
                command = self.commands[role]['id']; choice = None
                def found():
                    nonlocal choice
                    row = self.history[role][-1]; slot = ROLES.index(role)
                    if row['driver']['command'] != command or not row['players'][slot]['grounded']: return False
                    if row['players'][slot]['immersion']['kind'] in ('Lava', 'Slime'): return True
                    selected = forward_waiting_target(row, slot, goal)
                    if selected is not None and selected[1]:
                        choice = row['tick'], selected[0]
                        return True
                    return self.reached(role)
                self.until(found, self.navigation_timeout)
                if self.body(role)['player']['immersion']['kind'] in ('Lava', 'Slime'):
                    self.escape_hazard(role)
                    continue
                break
            else: raise AssertionError('Forward waiting repeatedly entered damaging liquid')
            if choice is not None: break
            # A narrow slope can end at another narrow slope. Walk every
            # original intermediate point before searching farther ahead.
            # These are separate approaches, not completed paired waypoints.
            passed.append(self.receipt(role, next_goal))
            assert self.body(role)['player']['immersion']['kind'] == 'Dry', 'Forward approach cannot wait in damaging liquid'
        else: raise AssertionError('No observed waiting bay within this navigation section')
        tick, park = choice
        entry = self.move(role, park, radius=2)
        entry.update(selected_tick=tick, navigation_start=start, navigation_command=command,
                     navigation_goal=next_goal, passed_goals=passed)
        self.evidence['forward_waiting'].append(entry)
        return park

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
            if self.step == 65:
                for index, footing in enumerate(SHELF_APPROACH):
                    receipt = self.move(role, footing, navigate=True, radius=2)
                    self.evidence['shelf_approaches'].append(dict(receipt,index=index))
                    print('PASS ordinary shelf approach', role, index, flush=True)
            entry = self.move(role, goal, navigate=True)
            self.escape_hazard(role)
            if self.clear():
                entry = self.move(role, goal, navigate=True)
                self.escape_hazard(role)
            self.evidence['arrivals'].append(entry)
            if role == first:
                visited = []
                for _ in range(8):
                    report = self.reports[role]; slot = ROLES.index(role)
                    at = list(self.body(role)['player']['feet'])
                    candidates = report['route_driver']['neighbors'][slot]
                    selection = waiting_target(candidates, at, self.body(other)['player']['feet'], goal, visited)
                    if selection is None:
                        # The next original point may itself be in shallow lava.
                        # Follow its approach only until a native safe bay is
                        # observed, then leave room for the incoming player.
                        park = self.forward_bay(role, goal)
                        break
                    park, ready = selection
                    visited.append(at)
                    entry = dict(selected_tick=report['frame']['tick'], candidates=candidates, ready=ready)
                    entry.update(self.move(role, park, radius=2))
                    self.evidence['parking'].append(entry)
                    if ready: break
                else: raise AssertionError('No clear waiting position for the second player')
                self.set(role, park, run=True)
        self.set(other)

    def platform_roles(self):
        first = 'host' if self.args.first == 'blue' else 'guest'
        return first, ROLES[1-ROLES.index(first)]

    def wait_for_partner(self, rider):
        slot = ROLES.index(rider)
        def aboard():
            return all(r['route_driver']['curiosity_platform']['full'][slot]
                       and not r['route_driver']['curiosity_platform']['riders'][1-slot]
                       for r in self.reports.values())
        self.until(aboard, 15)
        entry = dict(rider=rider, start={r:self.history[r][-1]['tick'] for r in ROLES})
        self.until(lambda: all(self.history[r][-1]['tick'] >= entry['start'][r] + 120 for r in ROLES), 10)
        entry['end'] = {r:self.history[r][-1]['tick'] for r in ROLES}
        verify_platform_wait(entry, self.history)
        self.evidence['platform_waits'].append(entry)
        print('PASS shared platform waits for partner', flush=True)

    def approach_platform(self):
        first, other = self.platform_roles()
        start = list(self.body(first)['player']['feet'])
        self.set(other)
        self.evidence['arrivals'].append(self.move(first, self.steps[53]['pos'], navigate=True))
        self.evidence['platform_moves'].append(self.move(first, start, navigate=True))
        self.set(first, start, run=True)
        self.evidence['arrivals'].append(self.move(other, self.steps[53]['pos'], navigate=True))
        self.evidence['platform_moves'].append(self.move(other, PLATFORM_RIGHT, radius=2))
        self.set(other, PLATFORM_RIGHT, horizontal=True, run=True, radius=2)
        self.wait_for_partner(other)

    def platform_jump(self, role):
        slot = ROLES.index(role)
        for _ in range(2):
            before = self.history[role][-1]
            assert before['players'][slot]['grounded'], 'Platform jump has no grounded takeoff'
            assert before['players'][slot]['immersion']['kind'] == 'Dry', 'Platform takeoff already reached lava'
            self.set(role, PLATFORM_BANK, run=True, jump=True)
            runup = distance(before['players'][slot]['feet'][:2], PLATFORM_RIGHT[:2]) < 8
            if runup:
                delta = [PLATFORM_BANK[i] - before['players'][slot]['feet'][i] for i in range(2)]
                length = distance(delta, [0,0])
                self.commands[role]['runup'] = dict(millis=150, wish=[v/length for v in delta])
            command = self.commands[role]['id']; count = before['players'][slot]['jumps']
            self.until(lambda: (self.body(role)['player']['jumps'] == count + 1 and not self.body(role)['player']['grounded'])
                       or self.body(role)['player'].get('climb') is not None, 10)
            if self.body(role)['player']['jumps'] == count + 1:
                flight = self.history[role][-1]['tick']
                break
            # Space may start the normal automatic bank climb. Finish that
            # action without walking off its landing, then press jump again.
            entry = dict(role=role, before=before['tick'], active=self.history[role][-1]['tick'], command=command)
            self.set(role)
            self.until(lambda: self.body(role)['player'].get('climb') is None and self.body(role)['player']['grounded'], 10)
            entry['after'] = self.history[role][-1]['tick']
            verify_platform_climb(entry, self.history)
            self.evidence['platform_climbs'].append(entry)
        else: raise AssertionError('Platform approach did not produce an ordinary jump after climbing')
        self.until(lambda: self.reached(role) and self.body(role)['player']['grounded'], 20)
        receipt = self.receipt(role, PLATFORM_BANK)
        self.evidence['arrivals'].append(receipt)
        self.evidence['platform_jumps'].append(dict(role=role, before=before['tick'], flight=flight, arrival=receipt))

    def cross_platform(self):
        first, other = self.platform_roles(); slot = ROLES.index(first)
        self.set(other, PLATFORM_RIGHT, horizontal=True, run=True, radius=2)
        self.wait_for_partner(other)
        self.set(first, PLATFORM_LEFT, horizontal=True, run=True, radius=2)
        def boarded():
            row = self.history['host'][-1]; p = row['driver']['curiosity_platform']
            return p['descending'] and p['sink'] > 0 and p['riders'][slot] and row['players'][slot]['grounded']
        self.until(boarded, 20)
        after = self.history['host'][-1]
        # Use already recorded contact to prove carry. Waiting for another
        # report here lowered the player into lava before sending the jump.
        prior = [r for r in self.history['host'][:-1] if 0 < after['tick'] - r['tick'] <= 60
                 and r['driver']['curiosity_platform']['riders'][slot]
                 and r['driver']['curiosity_platform']['sink'] < after['driver']['curiosity_platform']['sink']]
        assert prior, 'No preceding platform contact observation'
        entry = dict(rider=first, before=prior[-1]['tick'], after=after['tick'])
        verify_platform_carry(entry, self.history)
        self.evidence['platform_carries'].append(entry)
        self.platform_jump(first)
        # This narrow authored bank has no full-margin respawn neighbors.
        # Walk to its collision-probed waiting point and verify actual dry,
        # clear, grounded contact throughout the partner's crossing instead.
        self.evidence['platform_moves'].append(self.move(first, PLATFORM_PARK, radius=2))
        self.set(first, PLATFORM_PARK, run=True, radius=2)
        entry = dict(role=first, start={r:self.history[r][-1]['tick'] for r in ROLES})
        self.platform_jump(other)
        entry['end'] = {r:self.history[r][-1]['tick'] for r in ROLES}
        verify_bank_wait(entry, self.history)
        self.evidence['bank_waits'].append(entry)

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
        self.verify_navigation_evidence()

    def verify_navigation_evidence(self):
        verify_shelf_approaches(self.evidence['shelf_approaches'], self.history, self.completed,
                               self.evidence['approaches'], self.evidence['arrivals'])
        for entry in self.evidence['combat_waits']: verify_combat_wait(entry, self.history)
        for entry in self.evidence['platform_moves']: verify_arrival(entry, self.history)
        for entry in self.evidence['platform_waits']: verify_platform_wait(entry, self.history)
        for entry in self.evidence['platform_carries']: verify_platform_carry(entry, self.history)
        for entry in self.evidence['platform_climbs']: verify_platform_climb(entry, self.history)
        for entry in self.evidence['bank_waits']: verify_bank_wait(entry, self.history)
        for entry in self.evidence['platform_jumps']:
            role = entry['role']; receipt = entry['arrival']
            before, flight = [next(r for r in self.history[role] if r['tick'] == t)
                             for t in (entry['before'], entry['flight'])]
            after = next(r for r in self.history[role] if r['tick'] == receipt['tick'])
            assert receipt['step'] == 54 and receipt['goal'] == PLATFORM_BANK
            verify_jump_flight(before, flight, after, ROLES.index(role), receipt['command'])
        if 53 in self.completed: assert self.evidence['platform_waits'], 'Missing boarding wait'
        if 54 in self.completed:
            assert len(self.evidence['platform_carries']) == 1 and {e['role'] for e in self.evidence['platform_jumps']} == set(ROLES), 'Missing shared crossing'
        expected = {(step, role) for step in self.completed if step in self.approach_goals for role in ROLES}
        assert {(r['step'], r['role']) for r in self.evidence['approaches']} == expected
        assert len(self.evidence['approaches']) == len(expected), 'Duplicated approach receipt'
        for entry in self.evidence['approaches']:
            assert entry['goal'] == self.approach_goals[entry['step']]
            verify_arrival(entry, self.history)
        for entry in self.evidence['forward_waiting']:
            verify_forward_approaches(entry, self.steps, self.history)
            selected = next(r for r in self.history[entry['role']] if r['epoch'] == 1 and r['tick'] == entry['selected_tick'])
            slot = ROLES.index(entry['role'])
            start = next(r for r in self.history[entry['role']] if r['epoch'] == 1 and r['tick'] == entry['navigation_start'])
            visited = [next(r for r in self.history[entry['role']] if r['tick'] == p['selected_tick'])['players'][slot]['feet']
                       for p in self.evidence['parking'] if p['step'] == entry['step'] and p['role'] == entry['role']
                       and p['selected_tick'] < entry['navigation_start']]
            assert waiting_target(start['driver']['neighbors'][slot], start['players'][slot]['feet'],
                start['players'][1-slot]['feet'], self.steps[entry['step']]['pos'], visited) is None, 'Forward wait bypassed an available bay'
            assert selected['driver']['command'] == entry['navigation_command'] and start['tick'] < selected['tick'], 'Forward approach unobserved'
            assert forward_waiting_target(selected, slot, self.steps[entry['step']]['pos']) == (entry['goal'], True), 'Unobserved or unsettled forward bay'
            verify_arrival(entry, self.history)
        for entry in self.evidence['hazard_exits']:
            role = entry['role']; slot = ROLES.index(role)
            before = next(r for r in self.history[role] if r['tick'] == entry['before_tick'])
            assert before['players'][slot]['immersion']['kind'] in ('Lava', 'Slime')
            assert escape_target(self.history[role], slot, before) == (entry['selected_tick'], entry['goal']), 'Unobserved hazard exit'
            verify_arrival(entry, self.history)
            after = next(r for r in self.history[role] if r['tick'] == entry['tick'])
            assert after['players'][slot]['immersion']['kind'] == 'Dry'
            if 'climb' in entry:
                climb = entry['climb']
                assert climb['role'] == role and climb['before'] == entry['before_tick'] and climb['after'] <= entry['tick']
                verify_platform_climb(climb, self.history)
            else:
                flight = next(r for r in self.history[role] if r['tick'] == entry['flight_tick'])
                verify_jump_flight(before, flight, after, slot, entry['command'])
        verify_recovered_planning_errors(self.history, self.evidence['hazard_exits'])
        for entry in self.evidence['hazard_jumps']:
            role = entry['role']; slot = ROLES.index(role); goal = self.steps[38]['pos']
            assert entry['arrival']['goal'] == goal
            assert all(entry[k]['role'] == role and entry[k]['step'] == 38 for k in ('approach', 'arrival'))
            selected = next(r for r in self.history[role] if r['tick'] == entry['selected_tick'])
            assert entry['step'] == 38 and selected['driver']['command'] == entry['navigation_command']
            assert hot_approach_target(selected, slot, goal) == entry['approach']['goal']
            verify_arrival(entry['approach'], self.history)
            verify_arrival(entry['arrival'], self.history)
            before = next(r for r in self.history[role] if r['tick'] == entry['approach']['tick'])
            flight = next(r for r in self.history[role] if r['tick'] == entry['flight_tick'])
            assert before['players'][slot]['immersion']['kind'] == 'Dry'
            after = next(r for r in self.history[role] if r['tick'] == entry['arrival']['tick'])
            verify_jump_flight(before, flight, after, slot, entry['command'])
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
            if self.step == 53: self.approach_platform()
            elif self.step == 54: self.cross_platform()
            else: self.go(self.steps[self.step]['pos'])
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
    parser.add_argument('--navigation-timeout', type=int, default=90,
                        help='Bounded seconds per navigation leg, including asynchronous route search')
    args = parser.parse_args()
    if not 0 <= args.start_step <= args.last_step <= 76: parser.error('This approach/platform segment supports steps 0..76')
    if (args.start_step > 0) != (args.checkpoint is not None): parser.error('A resumed segment requires its unchanged checkpoint and positive start step')
    if not all(0 <= v <= 500 for v in (args.delay_ms, args.jitter_ms)): parser.error('Delay/jitter must be 0..500 ms')
    if not 30 <= args.navigation_timeout <= 600: parser.error('Navigation timeout must be 30..600 seconds')
    pair = CuriosityPair(args)
    try: pair.run()
    except BaseException as error:
        pair.evidence.update(passed=False, failed_step=pair.step, error=str(error),
            process_exit_codes_before_cleanup={r:p.poll() for r,p in pair.processes.items()})
        raise
    finally: pair.close()


if __name__ == '__main__': main()
