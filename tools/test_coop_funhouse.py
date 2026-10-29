"""Play Funhouse's opening, mirror and cell machinery in independent clients.

This runner covers the authored entrance through step 117. It sends only ordinary
inputs and normal save requests, including actual Blade throws at switches.
Every movement waypoint needs a separate arrival receipt for each player.
Full traversal, recovery, machinery and the exit remain separate work.
"""
import argparse
import hashlib
import json
import math
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair
from test_coop_chess import ChessPair
from test_coop_castling import verify_presence
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from coop_recovery import read_trace, verify_views
from coop_recovery import verify_presence as verify_recovery_presence


def sloped_arrival(step, goal):
    # The authored native steer() accepts 52 vertical units. This descent's
    # actual floor is near z69 at its z112 marker; keep the stricter eight-unit
    # horizontal gate and require a real supported body, not a planner estimate.
    return step == 116 and goal == [150, -4520, 112]


def verify_arrivals(steps, completed, receipts, history, allow_revival=False):
    """Require each claimed arrival to match an immutable client observation."""
    observed = {role: {row['tick']: row for row in rows} for role, rows in history.items()}
    expected = {step for step in completed if steps[step].get('mode', '') in ('', 'jump')}
    assert {r['step'] for r in receipts} == expected, 'Missing or extra movement receipts'
    for step in expected:
        entries = [r for r in receipts if r['step'] == step]
        assert len(entries) == 2 and {r['role'] for r in entries} == {'host', 'guest'}, 'Missing personal arrival'
        for receipt in entries:
            if receipt.get('native_exit'):
                verify_exit_arrival(steps, receipt, history)
                continue
            role = receipt['role']
            row = observed[role].get(receipt['tick'])
            assert row and row['owner'] is None, 'Arrival was not observed during free play'
            body = row['players'][0 if role == 'host' else 1]
            goal = steps[step]['pos']
            assert receipt['goal'] == goal and receipt['feet'] == body['feet'], 'Arrival differs from observed body'
            assert body['grounded'] and body['health'] > 0 and (allow_revival or body['spawned_at'] == 0), 'Invalid arrival body'
            jump = steps[step].get('mode') == 'jump'
            surface = receipt.get('surface')
            if surface is not None:
                assert step in (101, 103, 104) and surface == 460, 'Unknown moving-surface waypoint'
                native = row['driver'].get('surface')
                assert native and native['entity'] == surface and native['requested'] == goal, 'No observed native surface goal'
                assert native['supported'][0 if role == 'host' else 1], 'Player is not supported by the intended board'
                assert row['driver']['command'] == receipt['command'], 'Surface command was not acknowledged'
                assert receipt['target'] == native['goal'], 'Surface goal differs from native observation'
                goal = native['goal']
            # The authored jump controller (funhouse/route.rs) accepts these
            # supported landing bounds. Corridor walks keep their tighter gate.
            crossing = receipt.get('crossing', False)
            if crossing:
                goals = {95: [1920, -2650, 0], 97: [1920, -3400, 0]}
                assert step in goals and steps[step] == dict(pos=goals[step]), 'Unknown moving-platform approach'
                assert row['driver']['command'] == receipt['command'], 'No acknowledged crossing command'
                assert row['driver']['phase'] in ('walking', 'settling landing', 'reached'), 'No observed crossing input'
            radius, height = (20, 52) if jump or crossing else (8, 35)
            if sloped_arrival(step, steps[step]['pos']):
                height = 52
                assert row['driver']['command'] == receipt['command'], 'Slope command was not acknowledged'
                assert row['driver']['phase'] in ('walking', 'reached'), 'No observed slope approach input'
            assert distance(body['feet'][:2], goal[:2]) < radius and abs(body['feet'][2] - goal[2]) < height, 'Missed waypoint'
            if jump:
                earlier = observed[role].get(receipt['start_tick'])
                assert earlier and earlier['players'][0 if role == 'host' else 1]['jumps'] == receipt['jumps_before'], 'Jump start was not observed'
                assert body['jumps'] == receipt['jumps_after'], 'Jump count differs from observed body'
                assert receipt['jumps_after'] > receipt['jumps_before'], 'Jump waypoint has no actual jump'


def verify_exit_arrival(steps, receipt, history):
    """Match a brief native landing to actual sent AND restored body evidence."""
    role, step, tick = receipt['role'], receipt['step'], receipt['tick']
    assert step in (103, 104, 105), 'Unknown continuous exit waypoint'
    course = receipt['course']
    assert course['reached'] and course['step'] == 106, 'Native exit is incomplete'
    assert [r['step'] for r in course['receipts']] == [103, 104, 105], 'Native exit skipped a waypoint'
    assert any(row['driver']['command'] == receipt['command']
               and row['driver'].get('funhouse_exit') == course for row in history[role]), 'Exit course was not observed for its command'
    native = next(r for r in course['receipts'] if r['step'] == step)
    assert native['tick'] == tick and native['feet'] == receipt['feet'], 'Exit receipt differs from native observation'
    assert receipt['goal'] == steps[step]['pos'], 'Exit target differs from authored waypoint'
    states = {}
    for peer, rows in history.items():
        states[peer] = {}
        for row in rows:
            for state in row['driver']['network_states']:
                key = state['epoch'], state['tick']
                old = states[peer].setdefault(key, state)
                assert old == state, 'Per-tick network evidence changed'
    key = receipt['epoch'], tick
    start_key = receipt['epoch'], course['start_tick']
    for at in (key, start_key):
        assert all(at in states[p] for p in ('host', 'guest')), 'Missing sent or restored exit tick'
        a, b = states['host'][at], states['guest'][at]
        assert all(a[f] == b[f] for f in ('sha256', 'world_sha256', 'native_sha256', 'movement')), 'Exit tick differs between clients'
    slot = 0 if role == 'host' else 1
    body = states[role][key]['movement'][slot]
    start = states[role][start_key]['movement'][slot]
    assert start['jumps'] == course['start_jumps'] and start['spawned_at'] == course['spawned_at'], 'Exit start differs from actual body'
    assert body['feet'] == receipt['feet'] and body['grounded'] and body['health'] > 0, 'Invalid exit arrival body'
    assert body['spawned_at'] == start['spawned_at'] and body['warped_at'] == start['warped_at'], 'Exit used a relocation or respawn'
    goal = receipt['goal']
    radius, height = 20, 52
    if step != 105:
        surface = native['surface']
        assert surface and surface['entity'] == 460 and surface['requested'] == goal, 'Wrong exit support'
        assert surface['supported'][slot], 'Exit rider lacks own board support'
        goal, radius, height = surface['goal'], 8, 35
    else:
        assert native['surface'] is None and body['jumps'] > course['start_jumps'], 'Exit has no actual upper jump'
    assert distance(body['feet'][:2], goal[:2]) < radius and abs(body['feet'][2] - goal[2]) < height, 'Missed exit waypoint'


def switched(state, thread):
    if thread == 'MirrorTrigger':
        return state['mirror']
    assert thread.startswith('cell') and len(thread) == 5 and thread[4] in '12345678', 'Unknown authored switch'
    return bool(state['cells'] & (1 << (int(thread[4:]) - 1)))


def verify_switch_receipt(entry, history):
    rows = history[entry['role']]
    commanded = [r for r in rows if r['driver']['command'] == entry['command']]
    assert any(r['driver']['phase'] == 'shooting switch' and r['driver']['expected'] is not None
               for r in commanded), 'No observed active switch target'
    assert any(r['driver']['phase'] == 'switch completed' and r['driver']['reached']
               for r in commanded), 'No acknowledged switch completion'
    for role, states in history.items():
        relevant = [r for r in states if r['tick'] >= entry['before_ticks'][role]]
        assert relevant and not switched(relevant[0]['funhouse'], entry['thread']), 'Switch was already active'
        assert any(switched(r['funhouse'], entry['thread']) for r in relevant), 'Switch did not reach both clients'
    slot = 0 if entry['role'] == 'host' else 1
    observed = {row['tick']: row for row in rows}
    for movement in entry.get('repositions', []):
        before, after = observed.get(movement['before']), observed.get(movement['after'])
        assert before and after, 'Unobserved shooting approach'
        assert movement['goal'] in before['driver']['neighbors'][slot], 'Invented shooting footing'
        body = after['players'][slot]
        assert body['health'] > 0 and body['grounded'], 'Unsupported shooting approach'
        assert distance(body['feet'][:2], movement['goal'][:2]) < 8, 'Missed shooting approach'


def shooting_neighbor(neighbors, home, other, target, tried):
    choices = [p for p in neighbors if distance(p, home) <= 96
               and distance(p[:2], other[:2]) >= 48
               and all(distance(p, old) >= 24 for old in tried)]
    return min(choices, key=lambda p: distance(p, target)) if choices else None


def verify_holds(steps, completed, holds, history):
    expected = {i for i in completed if steps[i].get('mode') == 'hold'}
    assert {e['step'] for e in holds} == expected, 'Missing or extra timed holds'
    for index in expected:
        covered = []
        for entry in (e for e in holds if e['step'] == index):
            assert entry['goal'] == steps[index]['pos'] and entry['frames'] == steps[index]['frames']
            for role, start in entry['starts'].items():
                covered.append(role)
                rows = [r for r in history[role] if start <= r['tick'] <= entry['ends'][role]]
                assert all(r['driver']['command'] == entry['commands'][role]
                           and r['driver']['phase'] == 'holding toward goal'
                           and r['driver']['expected'] == entry['goal'] for r in rows), 'Timed hold was interrupted'
                assert rows and rows[-1]['tick'] - rows[0]['tick'] >= entry['frames'], 'Timed hold lacked observed simulation ticks'
        assert sorted(covered) == ['guest', 'host'], 'Each player must perform the timed hold once'


def waiting_point(neighbors, at, incoming, goal):
    # Reserve the eight-unit arrival tolerance as well as the 36-unit body
    # spacing. A target only 40 units away may stop the body inside the lane.
    candidates = [p for p in neighbors
                  if distance(p[:2], goal[:2]) >= 48 and distance(p[:2], incoming[:2]) >= 48]
    assert candidates, 'No safe ordinary walking space beside waypoint'
    direction = [goal[i] - incoming[i] for i in range(2)]
    length = max(1, distance(goal[:2], incoming[:2]))
    def score(point):
        offset = [point[i] - at[i] for i in range(2)]
        lateral = abs(offset[0]*direction[1] - offset[1]*direction[0]) / length
        return lateral + .1*distance(point[:2], incoming[:2])
    return max(candidates, key=score)


def forward_waiting_point(steps, index, goal):
    if index + 1 >= len(steps):
        return None
    next_step = steps[index + 1]
    if next_step.get('mode', '') != '':
        return None
    point = next_step['pos']
    return point if 72 <= distance(point[:2], goal[:2]) <= 512 and abs(point[2]-goal[2]) <= 52 else None


def verify_forward_waits(entries, steps, history):
    observed = {role: {row['tick']: row for row in rows} for role, rows in history.items()}
    for entry in entries:
        role, index = entry['role'], entry['step']
        slot = 0 if role == 'host' else 1
        assert entry['goal'] == forward_waiting_point(steps, index, steps[index]['pos']), 'Waiting skipped the next walking waypoint'
        before = observed[role].get(entry['start_tick'])
        after = observed[role].get(entry['tick'])
        assert before and after and after['tick'] > before['tick'], 'Forward waiting was not observed'
        a, b = before['players'][slot], after['players'][slot]
        assert a['health'] > 0 and b['health'] > 0 and a['spawned_at'] == b['spawned_at'], 'Forward waiting used death'
        assert a['warped_at'] == b['warped_at'] and after['owner'] is None and b['grounded'], 'Forward waiting used relocation or lacked support'
        assert distance(b['feet'][:2], entry['goal'][:2]) < 8 and abs(b['feet'][2]-entry['goal'][2]) < 35, 'Missed forward waiting point'
        assert distance(b['feet'][:2], steps[index]['pos'][:2]) >= 36, 'Forward waiter blocks the previous waypoint'


def fulcrum_waiting_point(neighbors, pivot):
    """Use an observed lateral footing beside the final seesaw's pivot."""
    candidates = [p for p in neighbors if abs(p[1] - pivot[1]) < 8
                  and 48 <= abs(p[0] - pivot[0]) <= 96 and abs(p[2] - pivot[2]) < 35]
    assert candidates, 'No supported lateral waiting point at the seesaw pivot'
    return min(candidates, key=lambda p: abs(p[1] - pivot[1]))


def fulcrum_balance_goal(anchor, angle):
    """Normal steering on entity 460; its centre is not a stationary bank."""
    assert math.isfinite(angle) and abs(angle) <= 45
    offset = max(-220, min(220, 20 * angle))
    radians = math.radians(angle)
    # Stay inside the native board footprint, following its tilted top plane.
    height = -16 + math.tan(radians) * offset + 16 / math.cos(radians)
    return [anchor[0], -3600 + offset, height]


def fulcrums_reset(states):
    return all(max(abs(v) for v in state['fulcrums'][index]) < 1
               for state in states for index in (31, 61, 62))


def clock_threats(report, role):
    """Awake nearby Automatons and their in-flight fists from the real snapshot."""
    feet = report['frame']['players'][0 if role == 'host' else 1]['player']['feet']
    threats = []
    for index, actor in enumerate(report['combat']['npcs']['actors']):
        clock = actor.get('clock')
        if not clock or not clock['active']:
            continue
        if clock['fists'] or (clock['health'] > 0 and clock['phase'] != 'Idle' and distance(feet, clock['feet']) <= 700):
            threats.append(index)
    return threats


def resident_boojum_threats(report, role):
    """Resident Boojums are separate from the generic encounter diagnostics."""
    slot = 0 if role == 'host' else 1
    feet = report['frame']['players'][slot]['player']['feet']
    catalog = {b['id']: b for b in report['route_driver']['boojums']
               if b['name'] == 'resident_boojum_' + str(b['id'])}
    visible = {b['id'] for b in catalog.values() if b['active'] and b['health'] > 0
               and b['line_of_sight'][slot] and b['distance'][slot] < 1200}
    threats = []
    for index, actor in enumerate(report['combat']['npcs']['actors']):
        resident = actor.get('resident') or {}
        boojum = resident.get('body', {}).get('Boojum')
        if not boojum:
            continue
        assert index in catalog, 'Native resident Boojum visibility diagnostics are missing'
        live = resident.get('active') and resident.get('awake') and boojum['active'] and boojum['health'] > 0
        if (live and index in visible
                or any(distance(feet, wave['position']) < 1200 for wave in boojum['waves'])):
            threats.append(index)
    return threats


def nearby_drops(body):
    """Only this player's live, uncollected essence near their current footing."""
    stats, feet = body['stats'], body['player']['feet']
    if stats['sanity'] <= 0 or stats['sanity'] >= 100 and stats['will'] >= 100:
        return []
    visit = 'funhouse$first'
    drops = []
    for drop in stats.get('loot', {}).get(visit, {}).get('drops', []):
        key = f'drop:{visit}:{drop["id"]}'
        if (key not in stats['collected'] and drop['remaining'] > 0
                and distance(feet[:2], drop['origin'][:2]) <= 256
                and abs(feet[2] - drop['origin'][2]) < 35):
            drops.append(dict(key=key, **drop))
    return sorted(drops, key=lambda d: distance(feet, d['origin']))


def cards_preference(steps, before):
    enabled = True
    for step in steps[:before]:
        if step.get('mode') == 'cards':
            assert step['pos'][0] in (0, 1), 'Invalid authored Cards preference'
            enabled = bool(step['pos'][0])
    return enabled


def verify_loot(receipts, history):
    for receipt in receipts:
        role, key = receipt['role'], receipt['key']
        slot = 0 if role == 'host' else 1
        observed = {row['tick']: row for row in history[role]}
        before, after = observed.get(receipt['start_tick']), observed.get(receipt['tick'])
        assert before and after and after['tick'] > before['tick'], 'Unobserved collection'
        old, new = before['players'][slot], after['players'][slot]
        assert key not in old['collected'] and key in new['collected'], 'No personal collection transition'
        assert after['owner'] is None and new['health'] > 0, 'Invalid collection body'
        drops = old['stats'].get('loot', {}).get('funhouse$first', {}).get('drops', [])
        assert any(f'drop:funhouse$first:{d["id"]}' == key and d['origin'] == receipt['origin']
                   and d['remaining'] > 0 for d in drops), 'No observed live personal drop'


def following_point(trail, leader, follower):
    # Follow actual supported positions, leaving a body-length margin even
    # while the next delayed input is in transit. Never invent a ledge offset.
    return next((entry for entry in trail
                 if distance(entry['feet'], leader) >= 96
                 and distance(entry['feet'], follower) >= 24), None)


def verify_following(footings, history):
    observed = {role: {row['tick']: row for row in rows} for role, rows in history.items()}
    for entry in footings:
        role = entry['leader']
        assert entry['follower'] != role, 'Follower followed itself'
        row = observed[role].get(entry['tick'])
        assert row and row['owner'] is None, 'Following position was not observed in free play'
        body = row['players'][0 if role == 'host' else 1]
        assert body['health'] > 0 and body['grounded'] and body['feet'] == entry['feet'], 'Following position was not a living supported body'


class FunhousePair(Pair):
    map_name, entry, next_map = 'funhouse', 'funhouse_start1', 'hatter1'
    evidence_prefix, capture_scenes = 'funhouse-route-', False

    def __init__(self, args):
        self.allow_revival = getattr(args, 'allow_revival', False)
        self.suspended = {}
        self.survivor_wait = {}
        self.moving_role = None
        self.prefer_cards = True
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.lead, self.lead_tick = self.minimum_lead, None
        self.step, self.completed = -1, []
        self.first = 'host' if args.first == 'blue' else 'guest'
        super().__init__(args)
        self.deadline = time.monotonic() + 1200
        raw = (ROOT / 'src/levels/funhouse/route_steps.json').read_bytes()
        self.steps = json.loads(raw)
        self.prefer_cards = cards_preference(self.steps, args.start_step)
        assert len(self.steps) == 1077, 'Review changed authored course'
        assert all(s.get('mode', '') in ('', 'wait', 'clear', 'jump', 'shot', 'hold', 'cards')
                   for s in self.steps[args.start_step:args.last_step + 1])
        (self.folder / 'course.json').write_bytes(raw)
        self.evidence.update(course_sha256=hashlib.sha256(raw).hexdigest(),
            executable_sha256=hashlib.sha256(args.exe.read_bytes()).hexdigest(), arrivals=[])
        dependencies = self.folder / 'fixture-dependencies'
        dependencies.mkdir()
        for name in ('test_coop_funhouse.py', 'test_coop_play.py', 'test_coop_village.py',
                     'test_coop_chess.py', 'test_coop_castling.py', 'test_coop_chess_resume.py', 'coop_diagnostics.py', 'coop_recovery.py'):
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

    def set(self, role, *args, shoot_switch=None, hold_goal=False, runup=None, surface=None, funhouse_exit=False, **kwargs):
        kwargs.setdefault('attack', bool(args and args[0] is not None or kwargs.get('goal') is not None))
        super().set(role, *args, **kwargs)
        self.commands[role].update(input_lead=self.lead if role == 'guest' else 0,
            watch_dialogue=True, skip_scene=self.args.skip_scenes, prefer_cards=getattr(self, 'prefer_cards', True),
            avoid_switches=True, shoot_switch=shoot_switch, hold_goal=hold_goal,
            combat_tactics=True, runup=runup, surface=surface, funhouse_exit=funhouse_exit)
        if getattr(self, 'allow_revival', False):
            self.commands[role]['recovery_trace'] = role == 'host'

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
        if self.allow_revival:
            assert report['frame']['checkpoint_generation'] == 0, 'Unexpected team wipe; separate restore proof required'
            assert report['route_driver'].get('recovery_trace_error') is None, 'Host recovery trace failed'
        else:
            super().validate_report(role, report)
        bodies = report['frame']['players']
        assert all(not b['stats']['god'] and not b['stats']['notarget'] for b in bodies), 'Cheated route'
        assert self.allow_revival or all(b['spawned_at'] == 0 for b in bodies), 'Strict route silently revived a player'
        if report['combat']['stage']['owner'] is None and all(b['stats']['sanity'] > 0 for b in bodies):
            a, b = [p['player']['feet'] for p in bodies]
            assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2] - b[2]) >= 55.95, 'Players overlap'
            assert report['presentation']['visible_players'][1 if role == 'host' else 0], 'Free-play partner hidden'

    def validate_clear(self, role, report):
        assert all(clear for clear, body in zip(report['route_driver']['clear'], report['frame']['players'])
                   if body['stats']['sanity'] > 0), f'{role}: obstructed living body'

    def resume_after_death(self):
        if not self.allow_revival:
            return
        for role in self.reports:
            command = self.commands[role]
            if self.body(role)['stats']['sanity'] <= 0:
                if command.get('goal') is not None or command.get('attack') or command.get('shoot_switch'):
                    self.suspended[role] = dict(command)
                    self.set(role, attack=False)
            elif role in self.suspended:
                # This restores only the requested input after the game's real
                # spawn. The body, timer, resources and spawn remain untouched.
                pending = self.suspended.pop(role)
                self.serial += 1
                pending.update(id=self.serial, input_lead=self.lead if role == 'guest' else 0)
                self.commands[role] = pending
        alive = [role for role in self.reports if self.body(role)['stats']['sanity'] > 0]
        if len(alive) == 1 and len(self.reports) == 2:
            survivor = alive[0]
            command = self.commands[survivor]
            if survivor != self.moving_role and command.get('goal') is not None and not command.get('hold_goal'):
                # A waiting survivor must be able to dodge through the ten-second
                # countdown. The native tactic predicts safe ordinary movement.
                self.survivor_wait[survivor] = dict(command)
                self.set(survivor, attack=True)
        elif len(alive) == len(self.reports):
            for role, pending in self.survivor_wait.items():
                self.serial += 1
                pending.update(id=self.serial, input_lead=self.lead if role == 'guest' else 0)
                self.commands[role] = pending
            self.survivor_wait.clear()

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Funhouse stopped; evidence retained')
        super().pump()
        self.resume_after_death()
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
        balance = getattr(self, 'fulcrum_balance', None)
        if balance:
            role = balance['role']
            body = self.body(role)
            assert body['stats']['sanity'] > 0 and body['spawned_at'] == balance['spawn'], 'Waiting rider left the seesaw through death'
            angle = self.state(role)['fulcrums'][62][0]
            goal = fulcrum_balance_goal(balance['anchor'], angle)
            old = self.commands[role].get('goal')
            if old is None or distance(old, goal) > 2:
                self.set(role, goal, run=True, attack=True)
                self.evidence.setdefault('fulcrum_balance', []).append(dict(
                    role=role, tick=self.reports[role]['frame']['tick'], angle=angle,
                    anchor=balance['anchor'], goal=goal, command=self.commands[role]['id']))

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

    def move(self, role, goal, navigate=False, jump=False, follow=None, crossing=False, running_jump=False, surface=None):
        assert surface is None or not (navigate or jump or crossing or running_jump or follow)
        jumps_before = self.body(role)['player']['jumps']
        start_tick = self.reports[role]['frame']['tick']
        # The native Funhouse steer() accepts 20/52 on its tilting approach.
        # Requiring a slower eight-unit stop lets the loaded tip sink away.
        radius, height = (20, 52) if jump or crossing else (8, 35)
        slope = sloped_arrival(self.step, goal) and not (jump or crossing or surface is not None)
        if slope:
            height = 52
        runup = None
        if running_jump:
            assert jump and not navigate and not crossing
            feet = self.body(role)['player']['feet']
            span = distance(feet[:2], goal[:2])
            assert span > 1
            runup = dict(millis=150, wish=[(goal[i]-feet[i])/span for i in range(2)])
        self.set(role, goal, navigate=navigate, run=True, jump=jump, runup=runup, surface=surface)
        best = (distance(self.body(role)['player']['feet'], goal), self.reports[role]['frame']['tick'])
        ledge_jump = None
        trail, following = [], None
        def done():
            nonlocal best, navigate, ledge_jump, following
            body, tick = self.body(role)['player'], self.reports[role]['frame']['tick']
            if self.body(role)['stats']['sanity'] <= 0:
                return False
            if follow and self.body(follow)['stats']['sanity'] > 0 and not self.busy():
                if body['grounded'] and (not trail or distance(trail[-1]['feet'], body['feet']) >= 96):
                    trail.append(dict(tick=tick, feet=list(body['feet'])))
                follower = self.body(follow)['player']['feet']
                if following is not None and distance(follower, following['feet']) < 24:
                    trail[:] = [entry for entry in trail if entry['tick'] > following['tick']]
                    following = None
                if following is None:
                    following = following_point(trail, body['feet'], follower)
                    if following is not None:
                        self.set(follow, following['feet'], navigate=True, run=True)
                        self.evidence.setdefault('following_footings', []).append(dict(
                            leader=role, follower=follow, **following))
            target = goal
            if surface is not None:
                native = self.reports[role]['route_driver'].get('surface')
                if not native or native['entity'] != surface or native['requested'] != goal:
                    return False
                target = native['goal']
                if not native['supported'][0 if role == 'host' else 1]:
                    return False
            at_goal = distance(body['feet'][:2], target[:2]) < radius and abs(body['feet'][2] - target[2]) < height
            acknowledged = self.reports[role]['route_driver']['command'] == self.commands[role]['id']
            arrived = acknowledged and body['jumps'] > jumps_before if jump else self.reached(role)
            if crossing:
                driver = self.reports[role]['route_driver']
                arrived = acknowledged and driver['phase'] in ('walking', 'settling landing', 'reached')
            if slope:
                arrived = acknowledged and self.reports[role]['route_driver']['phase'] in ('walking', 'reached')
            if self.busy() or arrived and body['grounded'] and at_goal:
                return True
            if surface is None and not crossing and body.get('ledge') and not body['ledge']['pulling'] and (ledge_jump is None or tick - ledge_jump > 120):
                self.set(role, goal, run=True, jump=True)
                ledge_jump = tick
                navigate = False
            gap = distance(body['feet'], goal)
            if gap < best[0] - 2:
                best = gap, tick
            elif surface is None and not crossing and not navigate and tick - best[1] > 240 and body['grounded']:
                # Match the authored route's normal path-planning fallback.
                navigate = True
                self.set(role, goal, navigate=True, run=True)
            return False
        self.moving_role = role
        try:
            self.until(done)
        finally:
            self.moving_role = None
        assert not self.busy(), 'Unexpected scene during opening corridor movement'
        body = self.body(role)['player']
        assert body['grounded'], 'Waypoint has no supported landing'
        target = self.reports[role]['route_driver']['surface']['goal'] if surface is not None else goal
        assert distance(body['feet'][:2], target[:2]) < radius and abs(body['feet'][2] - target[2]) < height
        if jump:
            assert body['jumps'] > jumps_before, 'No actual jump at authored jump step'
        return dict(start_tick=start_tick, jumps_before=jumps_before, jumps_after=body['jumps'])

    def park(self, role, other, goal, surface=None):
        """Walk to a native clear/supported neighbor, keeping the approach open."""
        slot = 0 if role == 'host' else 1
        at, incoming = self.body(role)['player']['feet'], self.body(other)['player']['feet']
        # Queue the narrow slope in the direction of travel. Its exact arrival
        # can have no supported lateral neighbor; do not improvise an offset.
        forward = self.step == 111 and surface is None
        if forward:
            parking = forward_waiting_point(self.steps, self.step, goal)
            assert parking is not None, 'No authored forward waiting point'
        else:
            try:
                parking = waiting_point(self.reports[role]['route_driver']['neighbors'][slot], at, incoming, goal)
            except AssertionError:
                parking = forward_waiting_point(self.steps, self.step, goal) if surface is None else None
                if parking is None:
                    raise
                forward = True
        start_tick = self.reports[role]['frame']['tick']
        self.move(role, parking, surface=surface)
        assert distance(self.body(role)['player']['feet'][:2], goal[:2]) >= 36
        if forward:
            self.evidence.setdefault('forward_waits', []).append(dict(role=role, step=self.step, goal=parking,
                start_tick=start_tick, tick=self.reports[role]['frame']['tick']))
        # Continue defending this waiting point. An enemy hit can otherwise
        # push the idle waiter into the partner's destination and block it.
        # Keep steering through the handoff; a neutral settling interval lets
        # incoming punches knock the player back into the lane before it ends.
        self.set(role, parking, run=True, attack=True, surface=surface)

    def go(self, goal, jump=False, surface=None):
        # On a return along a narrow hall, the player already in front leads.
        # Record the real order; do not make the bodies pass through each other.
        self.until(lambda: all(self.body(r)['stats']['sanity'] > 0 for r in self.commands))
        first = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], goal)
                    + (0 if r == self.first else 16))
        other = 'guest' if first == 'host' else 'host'
        self.set(other, attack=True)
        for role in (first, other):
            # On the narrow descent a follower can occupy the leader's target
            # after a death/respawn. Queue personal arrivals and parking there.
            follow = other if surface is None and role == first and not jump and not 111 <= self.step <= 117 and distance(self.body(role)['player']['feet'], goal) > 192 else None
            self.record_arrival(role, goal, jump, follow=follow, surface=surface)
            if role == first:
                self.park(first, other, goal, surface=surface)
        self.set(other, attack=True)

    def record_arrival(self, role, goal, jump=False, follow=None, crossing=False, running_jump=False, surface=None):
        movement = self.move(role, goal, jump=jump, follow=follow, crossing=crossing, running_jump=running_jump, surface=surface)
        report = self.reports[role]
        self.evidence['arrivals'].append(dict(step=self.step, role=role, goal=goal,
            tick=report['frame']['tick'], feet=self.body(role)['player']['feet'],
            command=report['route_driver']['command'], crossing=crossing, surface=surface,
            target=report['route_driver']['surface']['goal'] if surface is not None else goal, **movement))

    def hold_roles(self, roles, goal, frames):
        for role in roles:
            self.set(role, goal, run=True, hold_goal=True)
        def sampled():
            return all(self.history[r][-1]['driver']['command'] == self.commands[r]['id']
                       and self.history[r][-1]['driver']['phase'] == 'holding toward goal' for r in roles)
        self.until(sampled)
        starts = {r: self.history[r][-1]['tick'] for r in roles}
        self.until(lambda: all(self.reports[r]['frame']['tick'] >= starts[r] + frames + int(self.lead*120)
                               for r in roles) and sampled(), max(90, frames/120 + 60))
        self.evidence.setdefault('holds', []).append(dict(step=self.step, goal=goal, frames=frames, starts=starts,
            commands={r: self.commands[r]['id'] for r in roles},
            ends={r: self.reports[r]['frame']['tick'] for r in roles}))

    def hold(self, goal, frames):
        # Authored holds press toward a point for a fixed duration, including
        # points inside walls. They are directional inputs, not arrival goals.
        self.hold_roles(list(self.commands), goal, frames)

    def shoot(self, thread, role=None):
        assert all(not switched(self.state(r), thread) for r in self.commands), 'Puzzle switch activated before its shot command'
        before = {r: self.reports[r]['frame']['tick'] for r in self.commands}
        vantage = self.steps[self.step - 1]['pos']
        role = role or min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], vantage))
        for r in self.commands:
            self.set(r)
        home = self.body(role)['player']['feet']
        tried, repositions = [home], []
        other = 'guest' if role == 'host' else 'host'
        for attempt in range(5):
            self.set(role, shoot_switch=thread)
            command = self.commands[role]['id']
            start = self.reports[role]['frame']['tick']
            self.until(lambda: self.reached(role) or self.reports[role]['frame']['tick'] - start >= 960, 25)
            if self.reached(role):
                break
            assert attempt < 4, 'Switch unreachable from tested ordinary shooting positions'
            report = self.reports[role]
            target = report['route_driver']['expected']
            assert target is not None, 'No active switch target to approach'
            slot = 0 if role == 'host' else 1
            point = shooting_neighbor(report['route_driver']['neighbors'][slot], home,
                                      self.body(other)['player']['feet'], target, tried)
            assert point is not None, 'No safe nearby shooting approach'
            tick = report['frame']['tick']
            self.move(role, point)
            tried.append(point)
            repositions.append(dict(before=tick, after=self.reports[role]['frame']['tick'], goal=point))
        self.until(lambda: all(switched(self.state(r), thread) for r in self.commands)
                   and any(r['driver']['command'] == command and r['driver']['phase'] == 'switch completed'
                           and r['driver']['reached'] for r in self.history[role][-8:]), 20)
        entry = dict(step=self.step, role=role, thread=thread, command=command, before_ticks=before,
                     repositions=repositions)
        verify_switch_receipt(entry, self.history)
        self.evidence.setdefault('switches', []).append(entry)
        self.set(role)

    def mirror_course(self):
        """Queue the narrow mirror perch; both players traverse it personally."""
        other = 'guest' if self.first == 'host' else 'host'
        assert [self.steps[i].get('mode') for i in range(29, 33)] == ['jump', 'shot', 'hold', 'jump']
        assert self.steps[30]['thread'] == 'MirrorTrigger'
        returning = self.steps[32]['pos']
        # Clear the original top landing before the first player jumps back.
        # This uses the waiting player's observed supported neighbors.
        self.park(other, self.first, returning)
        for role in (self.first, other):
            # Parking is outside the authored takeoff position. Return to the
            # original upper approach before each player's downwards jump.
            self.move(role, self.steps[28]['pos'])
            for index in (29, 31, 32):
                self.step = index
                step = self.steps[index]
                if index == 31:
                    self.hold_roles([role], step['pos'], step['frames'])
                else:
                    self.record_arrival(role, step['pos'], jump=step['mode'] == 'jump')
                self.retain(f'mirror-{role}-{index:03d}')
                if index == 29 and role == self.first:
                    self.step = 30
                    self.shoot(self.steps[30]['thread'], role)
            if role == self.first:
                self.park(role, other, returning)
            else:
                self.set(role, attack=True)
        self.completed.extend(range(29, 33))
        print('PASS Funhouse mirror sequence 29..32: both personal jumps/holds; one actual shared switch', flush=True)

    def fulcrum_course(self):
        """Queue the first boards, then balance the final seesaw for the partner."""
        assert self.steps[95] == dict(pos=[1920, -2650, 0])
        assert self.steps[96] == dict(pos=[1920, -2950, 0], mode='jump')
        assert self.steps[97] == dict(pos=[1920, -3400, 0])
        assert self.steps[98] == dict(pos=[1920, -3600, -16])
        self.until(lambda: all(self.body(r)['stats']['sanity'] > 0 for r in self.commands))
        first = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], self.steps[95]['pos'])
                    + (0 if r == self.first else 16))
        other = 'guest' if first == 'host' else 'host'
        waiting = list(self.body(other)['player']['feet'])
        self.set(other, waiting, run=True, attack=True)
        for role in (first, other):
            if role == other:
                # All three boards matter. The final board is occupied, so
                # the first rider actively balances it using normal inputs.
                self.until(lambda: fulcrums_reset([self.state(r) for r in self.commands]), 30)
            for index in (95, 96, 97, 98):
                self.step = index
                self.record_arrival(role, self.steps[index]['pos'], jump=index == 96,
                                    crossing=index in (95, 97), running_jump=index == 96)
            if role == first:
                spawn = self.body(first)['spawned_at']
                self.clear((first,))
                self.collect_loot((first,))
                assert self.body(first)['stats']['sanity'] > 0 and self.body(first)['spawned_at'] == spawn, 'First rider must remain on the far side'
                # Step 98 is the final seesaw's pivot, not a stationary bank.
                # Unconstrained defensive movement here tips Red's approach.
                self.move(first, self.steps[98]['pos'])
                assert self.body(first)['spawned_at'] == spawn, 'Waiting rider died before reaching the pivot'
                slot = 0 if first == 'host' else 1
                anchor = fulcrum_waiting_point(self.reports[first]['route_driver']['neighbors'][slot], self.steps[98]['pos'])
                self.fulcrum_balance = dict(role=first, anchor=anchor, spawn=spawn)
                self.set(first, fulcrum_balance_goal(anchor, self.state(first)['fulcrums'][62][0]), run=True, attack=True)
            else:
                self.set(role, attack=True)
        self.fulcrum_balance = None
        self.set(first, attack=True)
        self.completed.extend([95, 96, 97, 98])
        print('PASS Funhouse fulcrums 95..98: both personal approaches, exit jumps and final pivot arrivals', flush=True)

    def seesaw_exit(self):
        """Keep the waiting rider forward until the other reaches the upper ledge."""
        assert self.steps[103] == dict(pos=[1920, -3820, 150])
        assert self.steps[104] == dict(pos=[1920, -3860, 195])
        assert self.steps[105] == dict(pos=[1920, -4050, 256], mode='jump')
        assert self.steps[106] == dict(pos=[1920, -4050, 256], mode='hold', frames=200)
        first = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], self.steps[103]['pos'])
                    + (0 if r == self.first else 16))
        other = 'guest' if first == 'host' else 'host'
        spawn = self.body(first)['spawned_at']
        self.set(other, list(self.body(other)['player']['feet']), surface=460, run=True, attack=True)
        for role in (first, other):
            # Load the front using actual movement. A lateral offset keeps the
            # waiting partner's footing free during the first rider's setup.
            x = 1920
            if role == first:
                x += 64 if self.body(role)['player']['feet'][0] >= x else -64
            self.move(role, [x, -3400, 0], surface=460)
            self.until(lambda: self.state(role)['fulcrums'][62][0] <= -43, 30)
            self.step = 103
            self.set(role, funhouse_exit=True, attack=True)
            self.moving_role = role
            try:
                self.until(lambda: self.reached(role), 70)
            finally:
                self.moving_role = None
            report = self.reports[role]
            course = report['route_driver']['funhouse_exit']
            assert course and course['reached'] and course['step'] == 106
            for native in course['receipts']:
                self.evidence['arrivals'].append(dict(step=native['step'], role=role,
                    goal=self.steps[native['step']]['pos'], tick=native['tick'], feet=native['feet'],
                    epoch=report['frame']['epoch'], command=report['route_driver']['command'],
                    native_exit=True, course=course))
            self.step = 106
            self.hold_roles([role], self.steps[106]['pos'], self.steps[106]['frames'])
            self.retain(f'upper-seesaw-{role}')
            if role == first:
                self.park(first, other, self.steps[105]['pos'])
        assert self.body(first)['stats']['sanity'] > 0 and self.body(first)['spawned_at'] == spawn, 'Upper waiting rider died during the handoff'
        self.completed.extend(range(103, 107))
        print('PASS Funhouse seesaw exit 103..106: both personal approaches, upper jumps and timed holds', flush=True)

    def clear(self, roles=None):
        roles = tuple(self.commands) if roles is None else tuple(roles)
        for role in roles:
            self.set(role, attack=True)
        since = None
        def cleared():
            nonlocal since
            quiet = True
            for role in roles:
                report = self.reports[role]
                driver, slot = report['route_driver'], 0 if role == 'host' else 1
                quiet &= driver['command'] == self.commands[role]['id'] and driver['native_threats'][slot] is None
                quiet &= not clock_threats(report, role)
                quiet &= not resident_boojum_threats(report, role)
                for kind in ('guards', 'boojums', 'snarks'):
                    quiet &= not any(e['active'] and e['health'] > 0 and e['distance'][slot] <= 700
                                     and e['line_of_sight'][slot] for e in driver[kind])
            tick = self.reports['host']['frame']['tick']
            since = (tick if since is None else since) if quiet else None
            return since is not None and tick - since >= 120
        self.until(cleared, 150)

    def collect_loot(self, roles=None):
        # Killing a foe without walking over its essence leaves both test
        # players starved. Each uses their own received inventory and inputs.
        roles = tuple(self.commands) if roles is None else tuple(roles)
        attempted = set()
        for _ in range(8):
            choices = [(self.body(role)['stats']['sanity'], role, drop)
                       for role in roles for drop in nearby_drops(self.body(role))
                       if (role, drop['key']) not in attempted]
            if not choices or self.busy():
                break
            _, role, drop = min(choices, key=lambda item: item[0])
            key, goal = drop['key'], drop['origin']
            attempted.add((role, key))
            other = 'guest' if role == 'host' else 'host'
            if self.body(other)['stats']['sanity'] > 0:
                if distance(self.body(other)['player']['feet'][:2], goal[:2]) < 64:
                    self.park(other, role, goal)
                else:
                    self.set(other, self.body(other)['player']['feet'], run=True, attack=True)
            # Parking can collect that player's copy incidentally. Re-read
            # before claiming a separate deliberate collection by this player.
            if key in self.body(role)['stats']['collected']:
                continue
            start = self.reports[role]['frame']['tick']
            self.set(role, goal, navigate=True, run=True)
            def done():
                stats = self.body(role)['stats']
                live = stats.get('loot', {}).get('funhouse$first', {}).get('drops', [])
                return (key in stats['collected'] or not any(d['id'] == drop['id'] for d in live)
                        or self.reports[role]['frame']['tick'] - start >= 1200)
            self.moving_role = role
            try:
                self.until(done, 35)
            finally:
                self.moving_role = None
            if key in self.body(role)['stats']['collected']:
                self.evidence.setdefault('loot', []).append(dict(role=role, key=key, origin=goal,
                    start_tick=start, tick=self.reports[role]['frame']['tick']))
                print('PASS personal essence', role, key, 'health', self.body(role)['stats']['sanity'], flush=True)
            self.set(role, attack=True)

    save_checkpoint = ChessPair.save_checkpoint

    def verify(self):
        assert self.completed == list(range(self.args.start_step, self.args.last_step + 1))
        verify_arrivals(self.steps, self.completed, self.evidence['arrivals'], self.history, self.allow_revival)
        verify_following(self.evidence.get('following_footings', []), self.history)
        verify_forward_waits(self.evidence.get('forward_waits', []), self.steps, self.history)
        verify_loot(self.evidence.get('loot', []), self.history)
        verify_holds(self.steps, self.completed, self.evidence.get('holds', []), self.history)
        shots = self.evidence.get('switches', [])
        assert [e['step'] for e in shots] == [i for i in self.completed if self.steps[i].get('mode') == 'shot']
        for entry in shots:
            verify_switch_receipt(entry, self.history)
        for role, rows in self.history.items():
            (verify_recovery_presence if self.allow_revival else verify_presence)(rows, role)
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
            if self.step in self.completed:
                continue
            self.retain(f'before-{self.step:03d}')
            if self.step in self.args.save_before:
                self.save_checkpoint(self.step)
            step = self.steps[self.step]
            mode = step.get('mode', '')
            if self.step == 29 and self.args.last_step >= 32:
                self.mirror_course()
                continue
            if self.step == 95:
                self.fulcrum_course()
                continue
            if self.step == 103:
                self.seesaw_exit()
                continue
            if mode == 'wait':
                self.wait(step['frames'] / 120)
            elif mode == 'clear':
                self.clear()
            elif mode == 'shot':
                self.shoot(step['thread'])
            elif mode == 'hold':
                self.hold(step['pos'], step['frames'])
            elif mode == 'cards':
                assert step['pos'][0] in (0, 1), 'Invalid authored Cards preference'
                self.prefer_cards = bool(step['pos'][0])
            else:
                self.go(step['pos'], jump=mode == 'jump', surface=460 if self.step in (101, 103, 104) else None)
            if self.step in (107, 109, 112):
                # Defeat the visible upper-corridor attackers before walking
                # onto the narrow descent. Moving-goal steering cannot dodge
                # their punches/waves. These are normal, resource-costed fights.
                previous = self.prefer_cards
                self.prefer_cards = True
                try:
                    self.clear()
                finally:
                    self.prefer_cards = previous
            self.completed.append(self.step)
            if mode in ('', 'clear'):
                self.collect_loot()
            print('PASS Funhouse step', self.step, mode or 'walk', 'health',
                  [self.body(r)['stats']['sanity'] for r in self.commands], flush=True)
        self.save_checkpoint(self.args.last_step + 1)
        self.verify()
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        self.evidence['completed_steps'] = self.completed
        if self.allow_revival:
            recovery = read_trace(self.folder / 'host/recovery-trace.jsonl', self.build, self.args.require_revival)
            recovery['rendered_revivals'] = verify_views(recovery['revivals'], self.history)
            self.evidence['recovery'] = recovery
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route=False,
            through_exit=False, build=self.build, internet=not self.args.local,
            first=self.args.first, skip_scenes=self.args.skip_scenes,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            scope=f'Funhouse steps {self.args.start_step}..{self.args.last_step}; both personal arrivals',
            mode='verified-individual-revival' if self.allow_revival else 'strict-no-death',
            fresh_entrance=self.saved is None,
            evidence=self.evidence), indent=2))
        print('PASS Funhouse segment; full route remains open', flush=True)

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
    parser.add_argument('--last-step', type=int, default=88)
    parser.add_argument('--start-step', type=int, default=0)
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--save-before', nargs='*', type=int, default=[])
    parser.add_argument('--skip-scenes', action='store_true')
    parser.add_argument('--allow-revival', action='store_true')
    parser.add_argument('--require-revival', action='store_true')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if args.require_revival and not args.allow_revival:
        parser.error('--require-revival requires --allow-revival')
    if not 0 <= args.start_step <= args.last_step <= 117 or args.last_step < 1:
        parser.error('The runner currently supports steps 0..117')
    if bool(args.checkpoint) != (args.start_step > 0):
        parser.error('A later start requires an unchanged checkpoint; a fresh run starts at zero')
    if 29 < args.start_step < 33 or 28 < args.last_step < 32:
        parser.error('The queued mirror sequence must include steps 29..32 together')
    if 95 < args.start_step < 99 or 94 < args.last_step < 98:
        parser.error('The queued fulcrum crossing must include steps 95..98 together')
    if 103 < args.start_step < 107 or 102 < args.last_step < 106:
        parser.error('The queued seesaw exit must include steps 103..106 together')
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
