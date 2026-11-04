"""Drive Clockwork prefixes in independent clients through the real network.

Each player must earn a separate waypoint receipt. The first stops at a native
clear/supported neighbor to let the second arrive. The initial runner supports
nav/walk/wait/lever steps and a paired gap crossing adapted from steps 36-42;
other timed inputs, portals and scene handoffs remain explicit future work.
An explicit checkpoint can resume a separately labelled segment,
with an exact native restore receipt. The runner never edits authoritative
positions, resources or progression.
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
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt

LEVERS = (376, 378, 289, 694, 44)
THREADS = ('Gear_Bridge_One', 'Gear_Bridge_Two', 'Open_ClockRoom_Door', 'Open_Port', 'stop_clock')
# Supported footings observed in an ordinary successful entrance descent.
DESCENT = [[-2848.6, 3228.5, 320.03125], [-3070.9, 2936.8, 176.03125],
           [-3231.4, 2929.1, 128.03125], [-3296, 2960, 192.03125]]
GEAR_APPROACH = [-4373.3784,3304.2549,208.03125]
GEAR_EXIT_APPROACH = [-4512,3008,208.03125]
CORRIDOR_HANDOFFS = (25,26,27,28,33,34,35)


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


def recorded_arrival(row, report, command):
    """Wait for an acknowledgment retained alongside the authoritative frame."""
    return bool(row and (row['epoch'],row['tick']) == (report['frame']['epoch'],report['frame']['tick'])
        and row['driver']['command'] == command and row['driver']['reached'])


def next_stair_footing(steps, step):
    assert step in (21,22,23), 'Forward waiting is limited to the stair flight'
    goal=steps[step]['walk'];following=steps[step+1]
    assert set(following)=={'walk'}, 'Next stair footing must be an ordinary walking step'
    point=following['walk']
    assert 48 <= distance(goal[:2],point[:2]) <= 192 and 0 <= point[2]-goal[2] <= 48, 'Invalid next stair footing'
    return point


def verify_stair_waiting(receipts, steps, arrivals, history):
    verify_forward_waiting(receipts,steps,arrivals,history,next_stair_footing,'Stair',2)


def next_corridor_footing(steps, step):
    assert step in CORRIDOR_HANDOFFS, 'Forward navigation is limited to the laboratory corridor'
    assert set(steps[step])=={'nav'} and set(steps[step+1])=={'nav'}, 'Corridor handoff cannot skip a non-navigation action'
    goal,point=steps[step]['nav'],steps[step+1]['nav']
    assert 48 <= distance(goal[:2],point[:2]) <= 256 and abs(point[2]-goal[2]) < 16, 'Invalid next corridor footing'
    return point


def verify_corridor_waiting(receipts, steps, arrivals, history):
    verify_forward_waiting(receipts,steps,arrivals,history,next_corridor_footing,'Corridor',8)


def verify_forward_waiting(receipts, steps, arrivals, history, target, label, radius):
    for receipt in receipts:
        step,role=receipt['step'],receipt['role'];slot=0 if role=='host' else 1
        goal=target(steps,step)
        assert receipt['goal']==goal, f'{label} waiting target differs from the course'
        pair=[r for r in arrivals if r['step']==step]
        assert len(pair)==2 and pair[0]['role']==role, f'{label} waiter must be the first arrival'
        rows={r['tick']:r for r in history[role]}
        before,end=rows[receipt['before_tick']],rows[receipt['tick']]
        assert pair[0]['tick']<=before['tick']<=end['tick']<pair[1]['tick'], f'{label} handoff out of order'
        start=before['players'][slot];original=next(iter(steps[step].values()))
        assert start['grounded'] and distance(start['feet'][:2],original[:2])<radius and abs(start['feet'][2]-original[2])<35, f'{label} waiter did not center first'
        body=end['players'][slot]
        assert body['health']>0 and body['spawned_at']==0 and body['grounded'], f'Invalid {label} waiting body'
        assert end['driver']['command']==receipt['command'] and end['driver']['reached'], f'{label} waiting arrival unacknowledged'
        assert body['feet']==receipt['feet'] and distance(body['feet'][:2],goal[:2])<8 and abs(body['feet'][2]-goal[2])<35, f'{label} waiting arrival missed'


def validate_gap_course(steps):
    assert [next(iter(s)) for s in steps[36:44]] == ['nav','walk','wait','input','input','input','nav','nav'], 'Review changed gap course'
    assert steps[38]['wait']==0.25 and [steps[i]['input'] for i in (39,40,41)] == [
        [0.1,0,1,0,0,0],[0.35,0,1,0,1,0],[0.65,0,-1,0,0,0]], 'Review changed gap inputs'
    approach,landing,waiting=steps[37]['walk'],steps[42]['nav'],steps[43]['nav']
    assert abs(approach[0]-landing[0])<1 and 200<landing[1]-approach[1]<300 and abs(landing[2]-approach[2])<1, 'Review changed gap geometry'
    assert distance(landing[:2],waiting[:2])>=64, 'Gap waiter needs landing clearance'


def validate_course_range(steps, start, last):
    if start<=42 and last>=36:
        assert start<=36 and last>=42, 'The paired gap needs the complete 36-42 sequence'
        validate_gap_course(steps)
    assert all(next(iter(steps[i])) in ('nav','walk','wait','lever') or 36<=i<=42
               for i in range(start,last+1)), 'Requested prefix needs an unimplemented input type'


def verify_gap_hold(hold, goal, history):
    assert hold['seconds']==0.25 and hold['end_tick']-hold['start_tick']>=30, 'Gap preparation wait ended early'
    slot=0 if hold['role']=='host' else 1
    rows=[r for r in history[hold['role']] if hold['start_tick']<=r['tick']<=hold['end_tick']]
    assert rows and rows[0]['tick']==hold['start_tick'] and rows[-1]['tick']==hold['end_tick'], 'Gap preparation wait unobserved'
    assert all(r['players'][slot]['grounded'] and distance(r['players'][slot]['feet'][:2],goal[:2])<8
               and abs(r['players'][slot]['feet'][2]-goal[2])<35 for r in rows), 'Gap preparation left the takeoff footing'


def verify_gap_sequence(evidence, steps, history):
    validate_gap_course(steps)
    approach,landing,waiting=steps[37]['walk'],steps[42]['nav'],steps[43]['nav']
    jumps=evidence['gap_jumps'];holds=evidence['gap_holds'];parks=evidence['gap_waiting']
    verify_jump_crossing(jumps,landing,approach,history,'Gap')
    assert len(holds)==2 and {h['role'] for h in holds}=={'host','guest'}, 'Missing personal gap preparation wait'
    for hold in holds:
        verify_gap_hold(hold,approach,history)
        jump=next(j for j in jumps if j['role']==hold['role'])
        assert hold['end_tick']<=jump['before_tick'], 'Gap jump preceded preparation'
        slot=0 if jump['role']=='host' else 1
        rows={r['tick']:r for r in history[jump['role']]}
        assert rows[jump['before_tick']]['players'][slot]['warped_at']==rows[jump['arrival_tick']]['players'][slot]['warped_at'], 'Gap used a recovery warp'
    assert len(parks)==1, 'Missing first-player gap handoff'
    park=parks[0];role=park['role'];slot=0 if role=='host' else 1
    pair=[r for r in evidence['arrivals'] if r['step']==36]
    assert len(pair)==2 and pair[0]['role']==role, 'Gap waiter must cross first'
    first=next(j for j in jumps if j['role']==role);other=next(j for j in jumps if j['role']!=role)
    assert first['arrival_tick']<=park['before_tick']<=park['tick']<pair[1]['tick'], 'Gap handoff out of order'
    assert park['goal']==waiting, 'Gap waiting target differs from the course'
    row=next(r for r in history[role] if r['tick']==park['tick']);body=row['players'][slot]
    assert row['driver']['command']==park['command'] and row['driver']['reached'], 'Gap waiting arrival unacknowledged'
    assert body['grounded'] and body['feet']==park['feet'] and distance(body['feet'][:2],waiting[:2])<8 and abs(body['feet'][2]-waiting[2])<35, 'Gap waiting arrival missed'
    assert all(r['players'][slot]['grounded'] and distance(r['players'][slot]['feet'][:2],landing[:2])>=48
               for r in history[role] if park['tick']<=r['tick']<=other['arrival_tick']), 'Gap waiter blocked the landing'


def verify_descent(arrivals, jumps, history):
    verify_arrivals([dict(nav=p) for p in DESCENT], list(range(len(DESCENT))), arrivals, history)
    assert len(jumps) == 4 and {(j['role'],j['leg']) for j in jumps} == {
        (r,leg) for r in ('host','guest') for leg in (0,1)}, 'Missing personal entrance jump'
    for jump in jumps: verify_grounded_jump(jump,history)


def verify_grounded_jump(jump, history):
    slot = 0 if jump['role'] == 'host' else 1
    rows = history[jump['role']]
    before = next(r for r in rows if r['tick'] == jump['before_tick'])
    after = next(r for r in rows if r['tick'] == jump['after_tick'])
    count = before['players'][slot]['jumps']
    assert before['players'][slot]['grounded'] and after['players'][slot]['jumps'] == count + 1, 'Movement needs one actual jump'
    assert any(before['tick'] < r['tick'] <= after['tick'] and r['driver']['command'] == jump['command']
               for r in rows), 'Jump command unobserved'
    # Actual input/takeoff events are recorded by the authoritative host,
    # including Red's events. The guest supplies its own command receipt.
    events = [e for r in history['host'] for e in r['driver'].get('input_events', []) if e['kind'] == 'jump'
              and before['tick'] <= e['tick'] < after['tick'] and e['epoch'] == 1]
    assert any(e['input'][slot] and e['before'][slot][1] and not e['after'][slot][1]
               and e['before'][slot][2] == count and e['after'][slot][2] == count + 1 for e in events), 'Actual takeoff unobserved'


def verify_gear_boarding(boardings, goal, history):
    verify_jump_crossing(boardings,goal,GEAR_APPROACH,history)


def verify_gear_exit(exits, goal, history):
    verify_jump_crossing(exits,goal,GEAR_EXIT_APPROACH,history)


def verify_jump_crossing(boardings, goal, approach, history, label='Gear'):
    assert len(boardings) == 2 and {r['role'] for r in boardings} == {'host','guest'}, f'Missing personal {label} crossing'
    for receipt in boardings:
        verify_grounded_jump(receipt,history)
        slot = 0 if receipt['role'] == 'host' else 1
        rows = {r['tick']:r for r in history[receipt['role']]}
        start, flight, end = [rows[receipt[k]] for k in ('before_tick','after_tick','arrival_tick')]
        assert start['driver']['command'] == receipt['approach_command'] and start['driver']['reached'], f'{label} approach unacknowledged'
        start_feet=start['players'][slot]['feet']
        assert distance(start_feet[:2],approach[:2]) < 2 and abs(start_feet[2]-approach[2]) < 35, f'{label} run-up started elsewhere'
        assert not flight['players'][slot]['grounded'], f'{label} flight unobserved'
        body = end['players'][slot]
        assert body['health'] > 0 and body['spawned_at'] == 0 and body['grounded'], f'Invalid {label} arrival body'
        assert end['driver']['command'] == receipt['arrival_command'] and end['driver']['reached'], f'{label} arrival unacknowledged'
        assert distance(body['feet'][:2],goal[:2]) < 8 and abs(body['feet'][2]-goal[2]) < 35, f'{label} arrival missed'


def yield_target(candidates, at, incoming, goal, via=None):
    """Choose an observed walking neighbor clear of the follower's approach."""
    route = [incoming] + ([via] if via is not None else []) + [goal]
    def segment_gap(point, a, b):
        line = [b[i]-a[i] for i in range(2)]
        length2 = sum(v*v for v in line)
        t = max(0, min(1, sum((point[i]-a[i])*line[i] for i in range(2))/length2)) if length2 else 0
        return distance(point[:2], [a[i]+t*line[i] for i in range(2)])
    def clearance(point):
        return min(segment_gap(point,a,b) for a,b in zip(route,route[1:]))
    usable = [p for p in candidates if distance(p[:2],incoming[:2]) >= 64 and clearance(p) >= 48
              and sum((p[i]-at[i])*(at[i]-incoming[i]) for i in range(2)) >= 0]
    assert usable, 'No observed supported yield clear of the follower'
    return max(usable, key=lambda p: (clearance(p),distance(p[:2],incoming[:2])))


def verify_yields(yields, steps, history):
    counts = {}
    for receipt in yields:
        role, follower = receipt['role'], receipt['follower']
        assert {role,follower} == {'host','guest'}, 'Yield needs two different players'
        key = receipt['step'], follower
        counts[key] = counts.get(key,0)+1
        assert counts[key] <= 3, 'Unbounded partner yielding'
        slot, other = (0,1) if role == 'host' else (1,0)
        selected = next(r for r in history[role] if r['tick'] == receipt['selected_tick'])
        paused = next(r for r in history[follower] if r['tick'] == receipt['paused_tick'])
        assert paused['driver']['command'] == receipt['pause_command'], 'Follower pause not acknowledged'
        assert paused['players'][other]['grounded'] and math.hypot(*paused['players'][other]['velocity'][:2]) < 20, 'Follower did not stop'
        candidates = selected['driver']['neighbors'][slot]
        assert receipt['candidates'] == candidates, 'Yield candidates were not observed'
        goal = next(iter(steps[receipt['step']].values()))
        assert receipt['goal'] == yield_target(candidates,selected['players'][slot]['feet'],
            selected['players'][other]['feet'],goal,receipt['via']), 'Yield target does not clear the approach'
        arrived = next(r for r in history[role] if r['tick'] == receipt['tick'])
        body = arrived['players'][slot]
        assert body['health'] > 0 and body['spawned_at'] == 0 and body['grounded'], 'Invalid yield body'
        assert body['feet'] == receipt['feet'] and arrived['driver']['command'] == receipt['command'] and arrived['driver']['reached'], 'Yield arrival not acknowledged'
        assert distance(body['feet'][:2],receipt['goal'][:2]) < 8 and abs(body['feet'][2]-receipt['goal'][2]) < 35, 'Yield did not arrive'


def partner_contact(player, waiter):
    # A blocked body's attempted velocity can remain nonzero. The caller
    # establishes stalled positional progress before considering this contact.
    return player['grounded'] and waiter['grounded'] and abs(player['feet'][2]-waiter['feet'][2]) < 56 \
        and distance(player['feet'][:2],waiter['feet'][:2]) < 40 \
        and math.hypot(*waiter['velocity'][:2]) < 20


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
        assert len(self.steps) == 150 and 0 <= args.start_step <= args.last_step < len(self.steps), 'Review changed course'
        validate_course_range(self.steps,args.start_step,args.last_step)
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
            arrivals=[], levers=[], holds=[], parking=[], stair_waiting=[], corridor_waiting=[], yields=[], descent_arrivals=[], descent_jumps=[], ledge_pulls=[], gear_boarding=[], gear_exits=[], gap_jumps=[], gap_holds=[], gap_waiting=[],
            completed=self.completed, scope='Fresh Clockwork prefix only')
        if args.start_step<=36<=args.last_step:
            self.evidence['gap_adaptation']='Steps 36-42 run separately per player: original approach, preparation wait and run-up, then actual jump and feedback steering to the original landing. This does not claim identical replay of the single-player timed air inputs.'
        dependencies = self.folder / 'fixture-dependencies'; dependencies.mkdir()
        for name in ('test_coop_hatter1.py', 'test_coop_village.py', 'test_coop_play.py', 'test_coop_chess.py',
                     'test_coop_funhouse.py', 'test_coop_castling.py', 'test_coop_chess_resume.py', 'coop_diagnostics.py', 'coop_recovery.py'):
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
            (self.folder / 'starting-checkpoint.json').write_bytes(raw)
            (self.folder / 'host' / 'coop-checkpoint.json').write_bytes(raw)
            self.evidence.update(checkpoint_sha256=hashlib.sha256(raw).hexdigest(),
                checkpoint_tick=self.saved['frame']['tick'],
                scope=f'Resumed Clockwork steps {args.start_step}..{args.last_step}; not fresh traversal')

    def set(self, role, *args, use_aim=None, **kwargs):
        kwargs.setdefault('attack', True)
        if use_aim is not None: kwargs['attack'] = False
        super().set(role, *args, **kwargs)
        self.commands[role].update(input_lead=self.lead if role == 'guest' else 0,
            use_aim=use_aim, watch_dialogue=True, combat_tactics=True,
            prefer_cards=False, prefer_ice=True, avoid_switches=True)

    def body(self, role):
        return self.reports[role]['frame']['players'][0 if role == 'host' else 1]

    def reached(self, role):
        # Pair retains one row per authoritative tick. A new command can be
        # acknowledged against that same tick; wait for a retained observation
        # instead of issuing the next command before its receipt is recorded.
        return super().reached(role) and recorded_arrival(
            self.history[role][-1] if self.history[role] else None,self.reports[role],self.commands[role]['id'])

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

    def move(self, role, goal, navigate=False, jump=False, radius=8, yield_partner=None):
        self.set(role, goal, navigate=navigate, run=True, jump=jump)
        self.commands[role]['arrival_radius'] = radius
        command = self.commands[role]['id']
        pullup = None
        yielded = 0
        best = (distance(self.body(role)['player']['feet'], goal), self.reports[role]['frame']['tick'])
        def done():
            nonlocal best, navigate, pullup, yielded, command
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
            elif yield_partner is not None and not jump and pullup is None and tick-best[1] > 240:
                peer = self.body(yield_partner)['player']
                if partner_contact(body,peer):
                    assert yielded < 3, 'Partner still obstructs the approach after three yields'
                    via = report['route_driver']['expected']
                    self.set(role, attack=False)
                    pause = self.commands[role]['id']
                    self.until(lambda: self.reports[role]['route_driver']['command'] == pause
                        and self.body(role)['player']['grounded']
                        and math.hypot(*self.body(role)['player']['velocity'][:2]) < 20, 15)
                    selected = self.reports[yield_partner]
                    slot, other = (0,1) if yield_partner == 'host' else (1,0)
                    candidates = selected['route_driver']['neighbors'][slot]
                    park = yield_target(candidates,selected['frame']['players'][slot]['player']['feet'],
                        selected['frame']['players'][other]['player']['feet'],goal,via)
                    receipt = dict(step=self.step,role=yield_partner,follower=role,goal=park,via=via,
                        selected_tick=selected['frame']['tick'],candidates=candidates,
                        pause_command=pause,paused_tick=self.reports[role]['frame']['tick'])
                    self.evidence['yields'].append(receipt)
                    self.move(yield_partner,park)
                    receipt.update(tick=self.reports[yield_partner]['frame']['tick'],
                        command=self.reports[yield_partner]['route_driver']['command'],
                        feet=list(self.body(yield_partner)['player']['feet']))
                    self.set(yield_partner,park,run=True)
                    self.set(role,goal,navigate=navigate,run=True)
                    self.commands[role]['arrival_radius'] = radius
                    command = self.commands[role]['id']
                    best = distance(self.body(role)['player']['feet'],goal), self.reports[role]['frame']['tick']
                    yielded += 1
                    print('PASS ordinary partner yield',self.step,yield_partner,flush=True)
            if not navigate and not jump and tick - best[1] > 240 and body['grounded']:
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

    def board_gear(self, role, goal):
        # The successful native approach runs south before jumping onto the
        # rotor. A static walking plan can instead step into the gap as it turns.
        self.cross_gear(role,goal,GEAR_APPROACH,True,250,[-0.196116135,-0.980580676],'gear_boarding')

    def exit_gear(self, role, goal):
        # Return to the observed supported waypoint, then jump west before the
        # rotating edge. A static walking route across it can lose its support.
        self.cross_gear(role,goal,GEAR_EXIT_APPROACH,False,150,[-1,0],'gear_exits')

    def cross_gear(self, role, goal, approach, navigate, millis, wish, evidence_key):
        self.move(role,approach,navigate=navigate,radius=2)
        before = self.reports[role]
        count = self.body(role)['player']['jumps']
        self.set(role,goal,run=True,jump=True)
        self.commands[role]['runup'] = dict(millis=millis,wish=wish)
        receipt = dict(role=role,before_tick=before['frame']['tick'],
            approach_command=before['route_driver']['command'],command=self.commands[role]['id'])
        self.evidence[evidence_key].append(receipt)
        self.until(lambda:self.reports[role]['route_driver']['command'] == receipt['command']
            and self.body(role)['player']['jumps'] == count+1 and not self.body(role)['player']['grounded'],15)
        receipt['after_tick'] = self.reports[role]['frame']['tick']
        # Keep ordinary air steering, then let the supported carry correction
        # settle at the original world-space waypoint. Never issue a second jump.
        self.move(role,goal)
        receipt.update(arrival_tick=self.reports[role]['frame']['tick'],
            arrival_command=self.reports[role]['route_driver']['command'])

    def forward_wait(self, role, park, navigate, evidence_key):
        receipt=dict(step=self.step,role=role,goal=park,before_tick=self.reports[role]['frame']['tick'])
        self.move(role,park,navigate=navigate)
        receipt.update(tick=self.reports[role]['frame']['tick'],command=self.reports[role]['route_driver']['command'],
            feet=list(self.body(role)['player']['feet']))
        self.evidence[evidence_key].append(receipt)
        self.set(role,park,run=True)

    def cross_corridor_gap(self):
        first=min(self.commands,key=lambda r:distance(self.body(r)['player']['feet'],self.steps[36]['nav']))
        other='guest' if first=='host' else 'host'
        self.set(other,attack=True)
        for role in (first,other):
            for step,key in ((36,'nav'),(37,'walk')):
                goal=self.steps[step][key]
                self.move(role,goal,navigate=key=='nav',radius=2 if step==37 else 8)
                report=self.reports[role]
                self.evidence['arrivals'].append(dict(step=step,role=role,goal=goal,tick=report['frame']['tick'],
                    feet=list(self.body(role)['player']['feet']),command=report['route_driver']['command']))
            start=self.reports[role]['frame']['tick']
            self.until(lambda:self.reports[role]['frame']['tick']>=start+30)
            self.evidence['gap_holds'].append(dict(role=role,seconds=0.25,start_tick=start,end_tick=self.reports[role]['frame']['tick']))
            self.cross_gear(role,self.steps[42]['nav'],self.steps[37]['walk'],False,100,[0,1],'gap_jumps')
            report=self.reports[role]
            self.evidence['arrivals'].append(dict(step=42,role=role,goal=self.steps[42]['nav'],tick=report['frame']['tick'],
                feet=list(self.body(role)['player']['feet']),command=report['route_driver']['command']))
            if role==first: self.forward_wait(role,self.steps[43]['nav'],True,'gap_waiting')
        # Preserve every observed timestamp; group personal receipts by course
        # step after both complete their sequential crossing.
        self.evidence['arrivals'].sort(key=lambda r:r['step'])

    def go(self, goal, navigate=False):
        first = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], goal) + (0 if r == self.first else 16))
        other = 'guest' if first == 'host' else 'host'
        self.set(other, attack=True)
        if self.step == 17:
            self.set(other,list(self.body(other)['player']['feet']),run=True)
        for role in (first, other):
            if self.step == 1: self.descend(role)
            if self.step == 16: self.board_gear(role,goal)
            if self.step == 17: self.exit_gear(role,goal)
            self.move(role, goal, navigate, yield_partner=first if role == other else None)
            report = self.reports[role]
            self.evidence['arrivals'].append(dict(step=self.step, role=role, goal=goal,
                tick=report['frame']['tick'], feet=list(self.body(role)['player']['feet']),
                command=report['route_driver']['command']))
            if role == first:
                slot = 0 if role == 'host' else 1
                if self.step in CORRIDOR_HANDOFFS:
                    self.forward_wait(role,next_corridor_footing(self.steps,self.step),True,'corridor_waiting')
                    continue
                if self.step in (21,22,23) or not report['route_driver']['neighbors'][slot]:
                    # An eight-unit arrival can stop on the edge of a small
                    # footing. Center by walking before asking for neighbors.
                    self.move(role, goal, radius=2)
                if self.step in (21,22,23):
                    self.forward_wait(role,next_stair_footing(self.steps,self.step),False,'stair_waiting')
                    continue
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
        assert self.completed == list(range(getattr(self.args,'start_step',0), self.args.last_step + 1)), 'Incomplete requested prefix'
        verify_arrivals(self.steps, self.completed, self.evidence['arrivals'], self.history)
        if 1 in self.completed:
            verify_descent(self.evidence['descent_arrivals'], self.evidence['descent_jumps'], self.history)
        verify_ledge_pulls(self.evidence['ledge_pulls'], self.history)
        verify_yields(self.evidence['yields'], self.steps, self.history)
        verify_stair_waiting(self.evidence['stair_waiting'],self.steps,self.evidence['arrivals'],self.history)
        verify_corridor_waiting(self.evidence['corridor_waiting'],self.steps,self.evidence['arrivals'],self.history)
        if 16 in self.completed: verify_gear_boarding(self.evidence['gear_boarding'],self.steps[16]['nav'],self.history)
        if 17 in self.completed: verify_gear_exit(self.evidence['gear_exits'],self.steps[17]['nav'],self.history)
        if 36 in self.completed: verify_gap_sequence(self.evidence,self.steps,self.history)
        expected = [i for i in self.completed if 'lever' in self.steps[i]]
        assert [r['step'] for r in self.evidence['levers']] == expected, 'Missing lever receipts'
        for receipt in self.evidence['levers']:
            row = next(r for r in self.history[receipt['role']] if r['tick'] == receipt['tick'])
            assert row['clockwork']['levers'][receipt['lever']] == receipt['time'] and row['clockwork'].get('pull') is None
            assert row['driver']['command'] == receipt['command'], 'Lever command not observed'
        assert [r['step'] for r in self.evidence['holds']] == [i for i in self.completed if 'wait' in self.steps[i] and i!=38], 'Missing timed wait'
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
        assert sorted([p['step'] for p in self.evidence['parking'] if p['ready']]
            +[p['step'] for kind in ('stair_waiting','corridor_waiting') for p in self.evidence[kind]]) == [i for i in self.completed
            if next(iter(self.steps[i])) in ('nav','walk') and i not in (36,37,42)], 'Missing completed parking handoff'
        for role, rows in self.history.items():
            assert rows and all(r['map'] == self.map_name and r['epoch'] == 1 and r['checkpoint_generation'] == 0 for r in rows)
            assert all(p['health'] > 0 and p['spawned_at'] == 0 for r in rows for p in r['players']), 'Prefix contains a death'
        for field, metric in (('sha256','matching_sent_and_restored_states'), ('world_sha256','matching_world_states'), ('native_sha256','matching_native_states')):
            self.verify_network_states(field, metric)
        self.evidence.update(build=self.build, passed=True)

    def run(self):
        self.start_pair()
        if self.saved:
            receipt = json.loads((self.folder / 'host' / 'route-resume.json').read_text())
            verify_resume_receipt(receipt,self.saved,self.build)
            self.evidence['exact_resume_receipt'] = receipt
            for report in self.reports.values():
                assert report['visit'] == self.saved['visit']
                elapsed = report['frame']['tick']-self.saved['frame']['tick']
                for body,old in zip(report['frame']['players'],self.saved['frame']['players']):
                    verify_resume_body(body,old,elapsed)
            print('PASS exact shared checkpoint resume',self.saved['frame']['tick'],flush=True)
        for self.step in range(self.args.start_step,self.args.last_step+1):
            if self.step in self.completed: continue
            step = self.steps[self.step]
            print('Clockwork step', self.step, step, flush=True)
            self.clear()
            if self.step in (0, 1, 9, 14, 17, 18, 25, 33, 36): self.save_checkpoint(self.step)
            if self.step==36:
                self.cross_corridor_gap()
                self.completed.extend(range(36,43))
                print('PASS paired gap sequence 36-42',flush=True)
                continue
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
        print('PASS', 'resumed' if self.saved else 'fresh', 'Clockwork prefix', self.completed[0], self.completed[-1], self.evidence['matching_sent_and_restored_states'], 'matching states', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue','red'), default='blue')
    parser.add_argument('--last-step', type=int, default=17)
    parser.add_argument('--start-step', type=int, default=0)
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not 0 <= args.start_step <= args.last_step < 150: parser.error('Require 0 <= start-step <= last-step < 150')
    if (args.start_step == 0) != (args.checkpoint is None): parser.error('A nonzero start-step requires a checkpoint; fresh runs start at zero')
    if not all(0 <= v <= 500 for v in (args.delay_ms, args.jitter_ms)): parser.error('Delay/jitter must be 0..500 ms')
    pair = ClockworkPair(args)
    try: pair.run()
    except BaseException as error:
        pair.evidence.update(passed=False, failed_step=pair.step, error=str(error))
        raise
    finally: pair.close()


if __name__ == '__main__': main()
