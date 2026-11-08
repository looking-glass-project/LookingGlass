"""Play Funhouse's opening, mirror and cell machinery in independent clients.

This runner covers the authored entrance through step 155. It sends only ordinary
inputs and normal save requests, including actual Blade throws at switches.
Every movement waypoint needs a separate arrival receipt for each player; the
arena entrance instead verifies its first-entrant scene and shared reunion.
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


AUTHORED_WALK_GOALS = {i: step['pos'] for i, step in enumerate(json.loads(
    (ROOT / 'src/levels/funhouse/route_steps.json').read_text())) if not step.get('mode')}


def authored_walk_arrival(step, goal):
    # Native steer() uses 52 vertical units for these approximate markers.
    # Bind that rule to the exact authored walk, retaining eight horizontal
    # units, actual ground contact and an acknowledged walking command.
    # Dynamic brush goals keep their own stricter projected support bounds.
    return AUTHORED_WALK_GOALS.get(step) == goal


def verify_arrivals(steps, completed, receipts, history, allow_revival=False, scene_handoffs=()):
    """Require each claimed arrival to match an immutable client observation."""
    scene_steps = {138} if 138 in completed else set()
    assert [e['step'] for e in scene_handoffs] == sorted(scene_steps), 'Missing or extra arena scene handoff'
    for entry in scene_handoffs:
        assert steps[entry['step']] == dict(pos=[32, -2800, -64]), 'Unknown scene entrance waypoint'
        verify_tweedle_handoff(entry, history)
    observed = {role: {row['tick']: row for row in rows} for role, rows in history.items()}
    expected = {step for step in completed if step not in scene_steps
                and steps[step].get('mode', '') in ('', 'jump', 'pendulum')}
    assert {r['step'] for r in receipts} == expected, 'Missing or extra movement receipts'
    for step in expected:
        entries = [r for r in receipts if r['step'] == step]
        assert len(entries) == 2 and {r['role'] for r in entries} == {'host', 'guest'}, 'Missing personal arrival'
        for receipt in entries:
            if receipt.get('native_pendulums'):
                verify_pendulum_course(steps, receipt, history)
                native = next(r for r in receipt['course']['receipts'] if r['step'] == step)
                assert step in range(129, 133) and native['tick'] == receipt['tick'] and native['feet'] == receipt['feet']
                assert receipt['goal'] == steps[step]['pos']
                continue
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
            if not (jump or crossing or surface is not None) and authored_walk_arrival(step, steps[step]['pos']):
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


def verify_pendulum_course(steps, entry, history):
    role, epoch, course = entry['role'], entry['epoch'], entry['course']
    assert course and course['reached'] and course['step'] == 133, 'Incomplete pendulum course'
    assert [r['step'] for r in course['receipts']] == list(range(128, 133)), 'Skipped pendulum stage'
    assert any(row['driver']['command'] == entry['command'] and row['owner'] is None
               and row['driver'].get('funhouse_pendulums') == course for row in history[role]), 'Unobserved pendulum command'
    states = {peer: {} for peer in history}
    for peer, rows in history.items():
        for row in rows:
            for state in row['driver']['network_states']:
                key = state['epoch'], state['tick']
                assert states[peer].setdefault(key, state) == state, 'Changed network evidence'
    slot = 0 if role == 'host' else 1
    previous = 0
    original = None
    for native in course['receipts']:
        step, start, end = native['step'], native['start_tick'], native['tick']
        assert previous <= start < end, 'Pendulum stages overlap or have no observed duration'
        previous = end
        for tick in (start, end):
            key = epoch, tick
            assert all(key in states[peer] for peer in ('host', 'guest')), 'Missing sent/restored pendulum tick'
            a, b = states['host'][key], states['guest'][key]
            assert all(a[f] == b[f] for f in ('sha256', 'world_sha256', 'native_sha256', 'movement')), 'Pendulum peers differ'
        before = states[role][epoch, start]['movement'][slot]
        body = states[role][epoch, end]['movement'][slot]
        if original is None:
            original = before
        assert before['jumps'] == native['start_jumps'] and body['jumps'] > before['jumps'], 'No actual personal jump'
        assert body['feet'] == native['feet'] and body['health'] > 0 and body['grounded'], 'Invalid pendulum landing body'
        for state in states[role].values():
            if state['epoch'] == epoch and start <= state['tick'] <= end:
                p = state['movement'][slot]
                assert p['health'] > 0 and all(p[f] == original[f] for f in ('spawned_at', 'warped_at')), 'Crossing used death or relocation'
        if step == 128:
            assert steps[step] == dict(pos=[-288, -3500, -32], mode='holdjump', frames=1200)
            assert before['grounded'] and distance(before['feet'][:2], steps[step]['pos'][:2]) < 20 and abs(before['feet'][2] + 32) < 35, 'Jump hold did not start on the bank'
            assert end - start >= steps[step]['frames'] and native['support'] is None, 'Incomplete bank jump hold'
            assert distance(body['feet'][:2], steps[step]['pos'][:2]) < 20 and abs(body['feet'][2] + 32) < 35
        elif step < 132:
            support = native['support']
            assert steps[step]['mode'] == 'pendulum' and support and support['name'] == steps[step]['thread'], 'Wrong pendulum'
            assert support['entity'] == {129: 117, 130: 116, 131: 115}[step], 'Wrong pendulum brush'
            assert support['supported'][slot] and abs(body['feet'][0] - support['goal'][0]) < 65, 'Missing own moving-platform support'
        else:
            assert steps[step] == dict(pos=[640, -3500, 56], mode='jump') and native['support'] is None
            assert distance(body['feet'][:2], steps[step]['pos'][:2]) < 20 and abs(body['feet'][2] - 56) < 52, 'Missed far bank'


def switched(state, thread):
    if thread == 'MirrorTrigger':
        return state['mirror']
    assert thread.startswith('cell') and len(thread) == 5 and thread[4] in '12345678', 'Unknown authored switch'
    return bool(state['cells'] & (1 << (int(thread[4:]) - 1)))


def verify_tweedle_handoff(entry, history):
    assert entry['step'] == 138 and entry['scene'] == 'Funhouse_Tweedle_Cinema1'
    owner = 'Blue' if entry['role'] == 'host' else 'Red'
    partner = 1 if owner == 'Blue' else 0
    warps = []
    for peer, rows in history.items():
        rows = [r for r in rows if entry['starts'][peer] <= r['tick'] <= entry['ends'][peer]]
        active = [r for r in rows if r['scene'] == entry['scene'] and r['owner'] is not None]
        assert active and {r['owner'] for r in active} == {owner}, 'Wrong or missing first-entrant scene owner'
        before = active[0]
        for row in active:
            assert not row['presentation']['visible_players'][partner], 'Scene displayed the waiting partner'
            assert row['players'][partner]['feet'] == before['players'][partner]['feet'], 'Scene moved the waiting partner'
            for old, body in zip(before['players'], row['players']):
                assert body['health'] == old['health'] > 0 and body['spawned_at'] == old['spawned_at'], 'Scene damaged or revived a player'
                assert body['stats']['owned'] == old['stats']['owned'] and body['collected'] == old['collected'], 'Scene changed personal inventory'
        if not entry['skipped']:
            assert {r['funhouse']['scene']['clock']['line'] for r in active} == set(range(9)), 'Watched scene did not observe every dialogue line'
        reunited = [r for r in rows if r['tick'] > active[-1]['tick'] and r['owner'] is None
                    and r['funhouse']['fight'] and all(p['warped_at'] > old['warped_at']
                        for p, old in zip(r['players'], before['players']))]
        assert reunited, 'No actual post-scene reunion'
        after = next((r for r in reunited if all(p['grounded'] for p in r['players'])), None)
        assert after is not None, 'Scene bodies never settled on the ground'
        first = reunited[0]
        warp = first['players'][0]['warped_at']
        assert after['tick']-warp <= 60, 'Scene landing took too long to settle'
        # The original actor landing is above the arena floor. Ordinary gravity
        # settles it; the partner is already placed on supported ground.
        # Observe that short descent rather than treating an airborne frame as
        # the completed reunion, and reject drift, deep falls or damage.
        for row in (r for r in reunited if r['tick'] <= after['tick']):
            assert all(row['driver']['clear']) and row['players'][partner]['grounded'], 'Obstructed or unsupported reunion partner'
            for p, initial, old in zip(row['players'], first['players'], before['players']):
                assert p['warped_at'] == warp and p['spawned_at'] == old['spawned_at'] and p['health'] == old['health'], 'Landing changed life or health'
                assert distance(p['feet'][:2], initial['feet'][:2]) < 1 and 0 <= initial['feet'][2]-p['feet'][2] <= 32, 'Landing drifted or fell too far'
        a, b = after['players']
        assert a['warped_at'] == b['warped_at'], 'Scene reunion stamps differ'
        assert all(p['grounded'] and p['health'] > 0 for p in after['players']) and all(after['driver']['clear']), 'Unsafe scene reunion'
        assert distance(a['feet'][:2], b['feet'][:2]) >= 35.95 or abs(a['feet'][2]-b['feet'][2]) >= 55.95, 'Scene reunited overlapping players'
        assert distance(a['feet'], b['feet']) <= 110 + max(0, after['tick']-a['warped_at'])*640/120, 'Scene separated the partners'
        for old, body in zip(before['players'], after['players']):
            assert body['spawned_at'] == old['spawned_at'], 'Reunion silently revived a player'
            assert body['stats']['owned'] == old['stats']['owned'] and body['collected'] == old['collected'], 'Reunion changed personal inventory'
        assert after['presentation']['visible_players'][1 if peer == 'host' else 0], 'Reunited partner remains hidden'
        warps.append(a['warped_at'])
    assert len(warps) == 2 and len(set(warps)) == 1, 'Clients disagree on the reunion'


def verify_gas_handoff(entry, history):
    """Require both clients to observe the automatic scene and its real reunion."""
    assert entry['step'] == 90 and entry['scene'] == 'GasOpenCine'
    assert entry['owner'] in ('Blue', 'Red') and set(history) == {'host', 'guest'}
    partner = 1 if entry['owner'] == 'Blue' else 0
    warps = []
    for peer, rows in history.items():
        rows = [r for r in rows if entry['starts'][peer] <= r['tick'] <= entry['ends'][peer]]
        active = [r for r in rows if r['scene'] == entry['scene'] and r['owner'] is not None]
        assert active and {r['owner'] for r in active} == {entry['owner']}, 'Missing or inconsistent gas owner'
        before = active[0]
        assert before['funhouse']['scene']['clock']['time'] <= .5, 'Gas scene start was not observed'
        for row in active:
            assert row['funhouse']['cells'] == 255 and not row['funhouse']['gas'], 'Gas scene bypassed the puzzle or completed early'
            assert not row['presentation']['visible_players'][partner], 'Gas scene displayed the waiting partner'
            assert row['players'][partner]['feet'] == before['players'][partner]['feet'], 'Gas scene moved the waiting partner'
            for old, body in zip(before['players'], row['players']):
                assert body['health'] == old['health'] > 0 and body['spawned_at'] == old['spawned_at'], 'Gas scene damaged or revived a player'
                assert body['stats']['owned'] == old['stats']['owned'] and body['collected'] == old['collected'], 'Gas scene changed personal inventory'
        reunited = [r for r in rows if r['tick'] > active[-1]['tick'] and r['owner'] is None
                    and r['funhouse']['gas'] and r['scene'] is None]
        assert reunited, 'Missing completed gas scene and reunion'
        after = reunited[0]
        a, b = after['players']
        warp = a['warped_at']
        assert warp == b['warped_at'] and all(warp > p['warped_at'] for p in before['players']), 'Missing actual shared gas reunion'
        if not entry['skipped']:
            # Native Gas finishes at 9.3 seconds. Reports are sampled, so use
            # the actual reunion tick plus the first observed scene clock.
            watched = warp - before['tick'] + before['funhouse']['scene']['clock']['time'] * 120
            assert watched >= 9.3 * 120 - 2, 'Watched gas scene finished early'
            assert max(r['funhouse']['scene']['clock']['time'] for r in active) >= 9., 'Gas scene ending was not observed'
        assert all(p['grounded'] and p['health'] > 0 for p in after['players']) and all(after['driver']['clear']), 'Unsafe gas reunion'
        assert distance(a['feet'][:2], b['feet'][:2]) >= 35.95 or abs(a['feet'][2]-b['feet'][2]) >= 55.95, 'Gas reunion overlaps players'
        assert distance(a['feet'], b['feet']) <= 110, 'Gas reunion separated the partners'
        for old, body in zip(before['players'], after['players']):
            assert body['health'] == old['health'] and body['spawned_at'] == old['spawned_at'], 'Gas reunion changed life or health'
            assert body['stats']['owned'] == old['stats']['owned'] and body['collected'] == old['collected'], 'Gas reunion changed personal inventory'
        assert after['presentation']['visible_players'][1 if peer == 'host' else 0], 'Gas reunion left the partner hidden'
        warps.append(warp)
    assert len(set(warps)) == 1, 'Clients disagree on gas reunion'


def verify_input_jumps(steps, completed, receipts, history):
    expected = {s for s in completed if steps[s].get('mode') == 'inputjump'}
    assert {r['step'] for r in receipts} == expected, 'Missing or extra timed jump'
    for step in expected:
        assert step in (147, 151) and steps[step] == dict(mode='inputjump', pos=[.001, 0, 0], frames=120)
        entries = [r for r in receipts if r['step'] == step]
        assert len(entries) == 2 and {r['role'] for r in entries} == {'host', 'guest'}, 'Missing personal timed jump'
        for entry in entries:
            rows = {r['tick']: r for r in history[entry['role']]}
            before, after = rows.get(entry['start_tick']), rows.get(entry['tick'])
            assert before and after and before['owner'] is None and after['owner'] is None, 'Unobserved timed jump'
            active = rows.get(entry['input_start_tick'])
            assert active and active['driver']['command'] == entry['command'], 'Jump input was not acknowledged'
            slot = 0 if entry['role'] == 'host' else 1
            a, b = before['players'][slot], after['players'][slot]
            origin = steps[146 if step == 147 else 149]['pos']
            assert a['grounded'] and distance(a['feet'][:2], origin[:2]) < 8 and abs(a['feet'][2]-origin[2]) < 52, 'Jump did not start on its personal pad'
            assert entry['jumps_before'] == a['jumps'] and entry['jumps_after'] == b['jumps'] == a['jumps']+1, 'Timed jump lacks its actual jump'
            assert before['tick'] <= active['tick'] and active['players'][slot]['jumps'] == a['jumps']+1, 'No observed active jump'
            assert after['tick']-active['tick'] >= steps[step]['frames'], 'Timed jump ended too early'
            assert after['driver']['command'] == entry['command'] and b['grounded'], 'Jump has no acknowledged landing'
            for row in (r for t, r in rows.items() if before['tick'] <= t <= after['tick']):
                p = row['players'][slot]
                assert row['owner'] is None and p['health'] > 0 and all(p[k] == a[k] for k in ('spawned_at', 'warped_at')), 'Timed jump used death or relocation'


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


def shooting_neighbor(neighbors, home, other, target, tried, visible=()):
    choices = [p for p in neighbors if distance(p, home) <= 96
               and distance(p[:2], other[:2]) >= 48
               and all(distance(p, old) >= 24 for old in tried)]
    # A closer point can put a swinging door across the shot. Prefer a native
    # sight-line observation, retaining the ordinary safe-footing bounds.
    return min(choices, key=lambda p: (p not in visible, distance(p, target))) if choices else None


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


def verify_waiting_footings(entries, history):
    observed = {role: {row['tick']: row for row in rows} for role, rows in history.items()}
    for entry in entries:
        row = observed[entry['role']].get(entry['tick'])
        assert row and row['owner'] is None, 'Waiting footing was not observed in free play'
        body = row['players'][0 if entry['role'] == 'host' else 1]
        assert body['health'] > 0 and body['grounded'], 'Waiting partner fell or died during handoff'
        assert distance(body['feet'][:2], entry['goal'][:2]) < 24 and abs(body['feet'][2] - entry['goal'][2]) < 35, 'Waiting partner left the supported stopping point'
        assert row['driver']['command'] == entry['command'], 'Waiting command was not acknowledged'


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


def combat_pending(report, role):
    driver, slot = report['route_driver'], 0 if role == 'host' else 1
    return (driver['native_threats'][slot] is not None or bool(clock_threats(report, role))
            or bool(resident_boojum_threats(report, role))
            or any(e['active'] and e['health'] > 0 and e['distance'][slot] <= 700
                   and e['line_of_sight'][slot] for kind in ('guards', 'boojums', 'snarks')
                   for e in driver[kind]))


def cards_preference(steps, before):
    enabled = True
    for step in steps[:before]:
        if step.get('mode') == 'cards':
            assert step['pos'][0] in (0, 1), 'Invalid authored Cards preference'
            enabled = bool(step['pos'][0])
    return enabled


def ice_preference(steps, before):
    enabled = False
    for step in steps[:before]:
        if step.get('mode') == 'ice':
            assert step['pos'][0] in (0, 1), 'Invalid authored Ice preference'
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
        self.prefer_ice = ice_preference(self.steps, args.start_step)
        assert len(self.steps) == 1077, 'Review changed authored course'
        assert all(s.get('mode', '') in ('', 'wait', 'clear', 'jump', 'shot', 'hold', 'cards', 'ice', 'holdjump', 'pendulum', 'inputjump')
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

    def set(self, role, *args, shoot_switch=None, hold_goal=False, runup=None, surface=None, funhouse_exit=False, funhouse_pendulums=False, **kwargs):
        kwargs.setdefault('attack', bool(args and args[0] is not None or kwargs.get('goal') is not None))
        super().set(role, *args, **kwargs)
        self.commands[role].update(input_lead=self.lead if role == 'guest' else 0,
            watch_dialogue=True, skip_scene=self.args.skip_scenes, prefer_cards=getattr(self, 'prefer_cards', True),
            prefer_ice=getattr(self, 'prefer_ice', False),
            avoid_switches=True, shoot_switch=shoot_switch, hold_goal=hold_goal,
            combat_tactics=True, runup=runup, surface=surface, funhouse_exit=funhouse_exit, funhouse_pendulums=funhouse_pendulums)
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
            # Paid primary-emission intervals are native weapon state, retained
            # separately for each player on both clients (not a visibility claim).
            row.setdefault('ice_primary_pulses', [sum(not pulse[2] for pulse in
                c['projectiles'].get('ice', {}).get('pulses', [])) for c in report['combat']['characters']])
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

    def tweedle_scene(self, role):
        assert self.step == 138, 'Unexpected scene outside the arena entrance'
        scene = 'Funhouse_Tweedle_Cinema1'
        owner = 'Blue' if role == 'host' else 'Red'
        starts = {r: report['frame']['tick'] for r, report in self.reports.items()}
        for r in self.commands:
            self.set(r)
        self.until(lambda: all(r['route_driver']['scene'] == scene and r['combat']['stage']['owner'] == owner
                               for r in self.reports.values()), 20)
        self.scene_end()
        self.until(lambda: all(r['combat']['stage']['owner'] is None and all(b['player']['grounded']
                               and b['stats']['sanity'] > 0 for b in r['frame']['players']) for r in self.reports.values()), 20)
        entry = dict(step=self.step, role=role, scene=scene, skipped=self.args.skip_scenes, starts=starts,
                     ends={r: report['frame']['tick'] for r, report in self.reports.items()})
        verify_tweedle_handoff(entry, self.history)
        self.evidence.setdefault('scene_handoffs', []).append(entry)
        self.retain('after-tweedle-scene')
        self.save_checkpoint(139)
        print('PASS Tweedles scene', owner, 'skipped' if self.args.skip_scenes else 'watched', 'and shared reunion', flush=True)

    def gas_scene(self):
        # The final cell switch schedules this scene one second later. Waiting
        # only for "not busy" can return before it starts. A resumed checkpoint
        # may instead see it during startup; retain that actual history too.
        assert self.step == 90
        if self.gas_already_saved():
            assert all(self.state(role)['gas'] for role in self.reports), 'Restored gas completion was lost'
            return  # This segment does not claim a scene completed in an earlier run.
        for role in self.commands:
            self.set(role, attack=False)
        def complete():
            for role, report in self.reports.items():
                assert report['route_driver']['scene'] in (None, 'GasOpenCine'), 'Unexpected scene before gas reunion'
                if not self.state(role)['gas'] or report['combat']['stage']['owner'] is not None:
                    return False
                if not all(b['player']['grounded'] and b['stats']['sanity'] > 0 for b in report['frame']['players']):
                    return False
            return len(self.reports) == 2
        self.until(complete, 60)
        active = {peer: [r for r in rows if r['scene'] == 'GasOpenCine' and r['owner'] is not None]
                  for peer, rows in self.history.items()}
        assert all(active.values()), 'Each client must observe the gas scene'
        entry = dict(step=90, scene='GasOpenCine', owner=active['host'][0]['owner'], skipped=self.args.skip_scenes,
                     starts={peer: rows[0]['tick'] for peer, rows in active.items()},
                     ends={peer: r['frame']['tick'] for peer, r in self.reports.items()})
        verify_gas_handoff(entry, self.history)
        self.evidence['gas_handoff'] = entry
        self.retain('after-gas-scene')
        print('PASS gas scene', entry['owner'], 'skipped' if entry['skipped'] else 'watched', 'and shared reunion', flush=True)

    def gas_already_saved(self):
        return bool(self.saved and self.saved['frame']['combat']['stage']['world']['levels'][self.map_name]['gas'])

    def tweedle_entry(self):
        # The entrance triggers before the authored marker is reached, then
        # closes behind the original actor. Verify the first-entrant scene
        # and shared reunion; walking both players back into that closed door
        # is neither an arrival requirement nor normal progression.
        assert self.step == 138 and not self.busy()
        assert all(not self.state(r)['fight'] for r in self.commands), 'Arena scene already completed; resume at 139'
        goal = self.steps[self.step]['pos']
        role = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], goal)
                   + (0 if r == self.first else 16))
        for peer in self.commands:
            self.set(peer)
        self.set(role, goal, run=True)
        self.until(self.busy)
        self.tweedle_scene(role)

    def move(self, role, goal, navigate=False, jump=False, follow=None, crossing=False, running_jump=False, surface=None):
        assert surface is None or not (navigate or jump or crossing or running_jump or follow)
        jumps_before = self.body(role)['player']['jumps']
        start_tick = self.reports[role]['frame']['tick']
        # The native Funhouse steer() accepts 20/52 on its tilting approach.
        # Requiring a slower eight-unit stop lets the loaded tip sink away.
        radius, height = (20, 52) if jump or crossing else (8, 35)
        slope = authored_walk_arrival(self.step, goal) and not (jump or crossing or surface is not None)
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
            assert not self.busy(), 'Unexpected scene during ordinary waypoint movement'
            if (35 <= self.step <= 42 or 116 <= self.step <= 127 or 135 <= self.step <= 146) and body['grounded'] and not self.state(role)['fight']:
                threats = {r: combat_pending(self.reports[r], r) for r in self.commands}
                if any(threats.values()):
                    # Fight newly visible corridor/ledge attackers before
                    # movement-goal steering carries both players past them.
                    # No-goal combat enables normal dodge/weapon inputs.
                    waiting = {r: dict(self.commands[r]) for r in self.commands if r != role}
                    self.clear()
                    if 35 <= self.step <= 42:
                        self.collect_loot()
                        # The nested pickup walk ends its own moving-role
                        # scope. This waypoint is still in progress afterwards.
                        self.moving_role = role
                    self.evidence.setdefault('combat_pauses', []).append(dict(
                        step=self.step, role=role, start_tick=tick,
                        tick=self.reports[role]['frame']['tick'], threats=threats))
                    for waiter, old in waiting.items():
                        if old.get('goal') is not None:
                            self.set(waiter, old['goal'], navigate=old['navigate'], run=old.get('run', False),
                                     attack=old['attack'], hold_goal=old.get('hold_goal', False), surface=old.get('surface'))
                    self.set(role, goal, navigate=navigate, run=True, jump=jump, runup=runup, surface=surface)
                    body, tick = self.body(role)['player'], self.reports[role]['frame']['tick']
                    best = distance(body['feet'], goal), tick
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
            if arrived and body['grounded'] and at_goal:
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
        return parking

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
            follow = other if surface is None and role == first and not jump and not 111 <= self.step <= 127 and distance(self.body(role)['player']['feet'], goal) > 192 else None
            self.record_arrival(role, goal, jump, follow=follow, surface=surface)
            if role == first:
                parking = self.park(first, other, goal, surface=surface)
        if 111 <= self.step <= 127 or 135 <= self.step <= 146:
            # An earlier personal arrival is not a safe handoff if the waiter
            # has since been knocked off the ledge while the partner approaches.
            def waiter_settled():
                body = self.body(first)
                feet = body['player']['feet']
                assert body['stats']['sanity'] > 0 and body['player']['grounded'], 'Waiting partner fell or died during handoff'
                assert abs(feet[2] - parking[2]) < 35, 'Waiting partner fell below the supported stopping point'
                return (distance(feet[:2], parking[:2]) < 24
                        and self.reports[first]['route_driver']['command'] == self.commands[first]['id'])
            # A hit can displace a still-supported waiter while its existing
            # walking command brings it back. Observe that return, not a timer.
            self.until(waiter_settled, 15)
            entry = dict(step=self.step, role=first, goal=parking,
                         command=self.commands[first]['id'], tick=self.reports[first]['frame']['tick'])
            verify_waiting_footings([entry], self.history)
            self.evidence.setdefault('waiting_footings', []).append(entry)
        self.set(other, attack=True)

    def record_arrival(self, role, goal, jump=False, follow=None, crossing=False, running_jump=False, surface=None):
        movement = self.move(role, goal, jump=jump, follow=follow, crossing=crossing, running_jump=running_jump, surface=surface)
        report = self.reports[role]
        self.evidence['arrivals'].append(dict(step=self.step, role=role, goal=goal,
            tick=report['frame']['tick'], feet=self.body(role)['player']['feet'],
            command=report['route_driver']['command'], crossing=crossing, surface=surface,
            target=report['route_driver']['surface']['goal'] if surface is not None else goal, **movement))

    def arena_ramp(self, first_step, last_step):
        """Keep both players moving through an ordinary boss-arena ramp."""
        assert (first_step, last_step) in ((144, 146), (153, 155))
        assert self.step == first_step and self.args.last_step >= last_step
        assert all(self.state(r)['fight'] for r in self.commands) and not self.busy()
        starts = {r: self.reports[r]['frame']['tick'] for r in self.commands}
        lives = {r: (self.body(r)['spawned_at'], self.body(r)['warped_at']) for r in self.commands}
        pending, issued, parking = {r: first_step for r in self.commands}, {}, None
        leader = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], self.steps[first_step]['pos']))
        follower = 'guest' if leader == 'host' else 'host'
        trail, following = [], None

        def issue(role):
            step = pending[role]
            # Ordinary ramp jumps keep the moving players above incoming
            # ground-level attacks; every waypoint still needs a real landing.
            self.set(role, self.steps[step]['pos'], run=True, jump=True)
            issued[role] = dict(start_tick=self.reports[role]['frame']['tick'],
                                jumps_before=self.body(role)['player']['jumps'])

        issue(leader)
        initial = dict(tick=self.reports[leader]['frame']['tick'], feet=list(self.body(leader)['player']['feet']))
        assert self.body(leader)['player']['grounded'], 'Ramp leader has no initial footing'
        self.set(follower, initial['feet'], run=True, jump=True)
        self.evidence.setdefault('following_footings', []).append(dict(leader=leader, follower=follower, **initial))

        def follow_ramp():
            nonlocal following
            if pending[follower] > last_step:
                return
            goal = self.steps[pending[follower]]['pos']
            if pending[follower] < pending[leader]:
                if self.commands[follower]['goal'] != goal:
                    issue(follower)
                following = None
                return
            # Do not crowd the leader while it brakes at the same waypoint.
            # Advance through actual grounded breadcrumbs at least 96 units
            # behind it, then visit the authored target after it has moved on.
            ahead = self.body(leader)['player']
            if ahead['grounded'] and (not trail or distance(trail[-1]['feet'], ahead['feet']) >= 64):
                trail.append(dict(tick=self.reports[leader]['frame']['tick'], feet=list(ahead['feet'])))
            feet = self.body(follower)['player']['feet']
            if following is not None and distance(feet, following['feet']) < 24:
                following = None
            if following is None:
                candidates = [p for p in trail if distance(p['feet'], goal) < distance(feet, goal)-16]
                following = following_point(candidates, ahead['feet'], feet)
                if following is not None:
                    self.set(follower, following['feet'], run=True)
                    self.evidence.setdefault('following_footings', []).append(dict(
                        leader=leader, follower=follower, **following))

        def climbed():
            nonlocal parking
            assert not self.busy(), 'Unexpected scene during arena climb'
            for role in self.commands:
                body = self.body(role)
                assert body['stats']['sanity'] > 0 and (body['spawned_at'], body['warped_at']) == lives[role], 'Arena climb interrupted by death or relocation'
                step = pending[role]
                if step > last_step:
                    continue
                goal, player = self.steps[step]['pos'], body['player']
                driver = self.reports[role]['route_driver']
                if (self.commands[role]['goal'] == goal and role in issued
                        and driver['command'] == self.commands[role]['id'] and driver['phase'] in ('walking', 'reached')
                        and player['grounded'] and distance(player['feet'][:2], goal[:2]) < 8
                        and abs(player['feet'][2]-goal[2]) < 52):
                    self.evidence['arrivals'].append(dict(step=step, role=role, goal=goal,
                        tick=self.reports[role]['frame']['tick'], feet=player['feet'], command=driver['command'],
                        crossing=False, surface=None, target=goal, jumps_after=player['jumps'], **issued[role]))
                    pending[role] += 1
                    print('PASS arena climb personal arrival', role, step, flush=True)
                    if pending[role] <= last_step:
                        if role == leader or pending[role] < pending[leader]:
                            issue(role)
                        else:
                            self.set(role, list(player['feet']), run=True)
            follow_ramp()
            # Only stop after the full ramp. Clear the final destination while
            # the other player continues, using a real supported neighboring point.
            if parking is None:
                first = next((r for r in self.commands if pending[r] > last_step), None)
                if first is not None:
                    other = 'guest' if first == 'host' else 'host'
                    self.step = last_step
                    parking = first, self.park(first, other, self.steps[last_step]['pos'])
            return all(step > last_step for step in pending.values())

        self.until(climbed)
        role, goal = parking
        self.until(lambda: self.body(role)['player']['grounded']
                   and distance(self.body(role)['player']['feet'][:2], goal[:2]) < 24
                   and abs(self.body(role)['player']['feet'][2]-goal[2]) < 35
                   and self.reports[role]['route_driver']['command'] == self.commands[role]['id'], 15)
        for peer, rows in self.history.items():
            for row in (r for r in rows if r['tick'] >= starts[peer]):
                for slot, who in enumerate(('host', 'guest')):
                    p = row['players'][slot]
                    assert p['health'] > 0 and (p['spawned_at'], p['warped_at']) == lives[who], 'Arena climb used a death or relocation'
        footing = dict(step=last_step, role=role, goal=goal, command=self.commands[role]['id'],
                       tick=self.reports[role]['frame']['tick'])
        verify_waiting_footings([footing], self.history)
        self.evidence.setdefault('waiting_footings', []).append(footing)
        self.completed.extend(range(first_step, last_step+1))
        print('PASS paired arena ramp', first_step, last_step, 'both personal arrivals without deaths', flush=True)

    def hold_roles(self, roles, goal, frames):
        for role in roles:
            self.set(role, goal, run=True, hold_goal=True)
        def sampled():
            return all(self.history[r][-1]['driver']['command'] == self.commands[r]['id']
                       and self.history[r][-1]['driver']['phase'] == 'holding toward goal' for r in roles)
        self.until(sampled)
        starts = {r: self.history[r][-1]['tick'] for r in roles}
        # The mirror's short run-up must finish while the player can still jump
        # from its perch. Its timer already starts after command observation;
        # adding the adaptive network lead extended 120 ticks to over 200 and
        # walked the guest over the edge before the next jump was requested.
        padding = 0 if self.step == 31 else int(self.lead*120)
        spawns = {r: self.body(r)['spawned_at'] for r in roles}
        def held():
            if self.step == 31:
                assert all(self.body(r)['stats']['sanity'] > 0 and self.body(r)['spawned_at'] == spawns[r]
                           for r in roles), 'Mirror run-up lost its rider'
                # This is directional input on a turning support, not point
                # arrival. Require support before the next actual jump; its
                # ordinary bank landing still needs a personal arrival receipt.
                if any(not self.body(r)['player']['grounded'] for r in roles):
                    return False
            return all(self.reports[r]['frame']['tick'] >= starts[r] + frames + padding
                       for r in roles) and sampled()
        self.until(held, max(90, frames/120 + 60))
        self.evidence.setdefault('holds', []).append(dict(step=self.step, goal=goal, frames=frames, starts=starts,
            commands={r: self.commands[r]['id'] for r in roles},
            ends={r: self.reports[r]['frame']['tick'] for r in roles}))

    def timed_jump(self, role):
        assert self.step in (147, 151) and self.steps[self.step]['mode'] == 'inputjump'
        body = self.body(role)['player']
        start, jumps = self.reports[role]['frame']['tick'], body['jumps']
        # A nearby goal above the body requests one ordinary jump, even when
        # horizontal movement is nearly zero. It cannot change position or
        # invent an upper landing; the native collision still decides that.
        goal = [body['feet'][0]+.32, body['feet'][1], body['feet'][2]+100]
        self.set(role, goal, run=True, jump=True)
        command = self.commands[role]['id']
        self.until(lambda: self.reports[role]['route_driver']['command'] == command
                   and self.body(role)['player']['jumps'] > jumps, 20)
        input_start = self.reports[role]['frame']['tick']
        def landed():
            p = self.body(role)
            assert p['stats']['sanity'] > 0, 'Player died during timed jump'
            return (self.reports[role]['route_driver']['command'] == command
                    and self.reports[role]['frame']['tick']-input_start >= self.steps[self.step]['frames']
                    and p['player']['jumps'] > jumps and p['player']['grounded'])
        self.until(landed, 20)
        self.evidence.setdefault('input_jumps', []).append(dict(step=self.step, role=role, command=command,
            start_tick=start, input_start_tick=input_start, tick=self.reports[role]['frame']['tick'], jumps_before=jumps,
            jumps_after=self.body(role)['player']['jumps']))

    def upper_bridge(self):
        assert self.step == 147 and self.args.last_step >= 149
        first = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], self.steps[146]['pos']))
        other = 'guest' if first == 'host' else 'host'
        for role in (first, other):
            # The second player uses the same actual takeoff pad only after
            # the first has crossed and cleared the far landing.
            self.step = 146
            self.move(role, self.steps[146]['pos'])
            self.step = 147
            self.timed_jump(role)
            for self.step in (148, 149):
                self.record_arrival(role, self.steps[self.step]['pos'])
            if role == first:
                parking = self.park(first, other, self.steps[149]['pos'])
        self.until(lambda: self.reports[first]['route_driver']['command'] == self.commands[first]['id']
                   and self.body(first)['player']['grounded']
                   and distance(self.body(first)['player']['feet'][:2], parking[:2]) < 24
                   and abs(self.body(first)['player']['feet'][2]-parking[2]) < 35, 15)
        footing = dict(step=149, role=first, goal=parking, command=self.commands[first]['id'],
                       tick=self.reports[first]['frame']['tick'])
        verify_waiting_footings([footing], self.history)
        self.evidence.setdefault('waiting_footings', []).append(footing)
        verify_input_jumps(self.steps, [147], self.evidence.get('input_jumps', []), self.history)
        self.completed.extend(range(147, 150))
        print('PASS upper bridge 147..149: both actual jumps and personal crossings', flush=True)

    def hold(self, goal, frames):
        # Authored holds press toward a point for a fixed duration, including
        # points inside walls. They are directional inputs, not arrival goals.
        self.hold_roles(list(self.commands), goal, frames)

    def left_pad(self):
        assert self.step == 151 and self.args.last_step >= 152
        pad = self.steps[149]['pos']
        first = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], pad))
        other = 'guest' if first == 'host' else 'host'
        # The preceding paired hold can leave the partner in the approach lane.
        # Move it to an observed clear neighbor before the first personal jump.
        self.park(other, first, pad)
        for role in (first, other):
            self.step = 149
            self.move(role, pad)
            self.step = 151
            self.timed_jump(role)
            self.step = 152
            self.record_arrival(role, self.steps[152]['pos'])
            if role == first:
                parking = self.park(first, other, self.steps[152]['pos'])
        self.until(lambda: self.reports[first]['route_driver']['command'] == self.commands[first]['id']
                   and self.body(first)['player']['grounded']
                   and distance(self.body(first)['player']['feet'][:2], parking[:2]) < 24
                   and abs(self.body(first)['player']['feet'][2]-parking[2]) < 35, 15)
        footing = dict(step=152, role=first, goal=parking, command=self.commands[first]['id'],
                       tick=self.reports[first]['frame']['tick'])
        verify_waiting_footings([footing], self.history)
        self.evidence.setdefault('waiting_footings', []).append(footing)
        verify_input_jumps(self.steps, [151], [e for e in self.evidence['input_jumps'] if e['step'] == 151], self.history)
        self.completed.extend((151, 152))
        print('PASS left pad 151..152: both actual jumps and personal departures', flush=True)

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
            self.until(lambda: self.reports[role]['route_driver']['command'] == command, 20)
            start = self.reports[role]['frame']['tick']
            if thread == 'MirrorTrigger':
                # Keep the original mirror fixture's bounded shooting wait.
                # The 12-second turning perch and Blade cooldown can require
                # several passes. Its small support has no safe neighboring
                # footing; the later cell-switch reposition policy does not fit.
                self.until(lambda: self.reached(role), 90)
            else:
                self.until(lambda: self.reached(role) or self.reports[role]['frame']['tick'] - start >= 960, 30)
            if self.reached(role):
                break
            assert attempt < 4, 'Switch unreachable from tested ordinary shooting positions'
            report = self.reports[role]
            target = report['route_driver']['expected']
            assert target is not None, 'No active switch target to approach'
            slot = 0 if role == 'host' else 1
            point = shooting_neighbor(report['route_driver']['neighbors'][slot], home,
                                      self.body(other)['player']['feet'], target, tried,
                                      report['route_driver']['switch_approaches'][slot])
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

    def pendulum_course(self):
        assert self.steps[126] == dict(pos=[-288, -3460, 0])
        first = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], self.steps[126]['pos']))
        other = 'guest' if first == 'host' else 'host'
        waiting = list(self.body(other)['player']['feet'])
        spawns = {r: self.body(r)['spawned_at'] for r in self.commands}
        self.set(other, waiting, run=True, attack=True)
        for role in (first, other):
            self.step = 126
            self.record_arrival(role, self.steps[126]['pos'])
            self.step = 127
            self.clear((role,))
            self.step = 128
            self.move(role, self.steps[128]['pos'])
            self.set(role, funhouse_pendulums=True, attack=True)
            self.moving_role = role
            try:
                self.until(lambda: self.reached(role), 240)
            finally:
                self.moving_role = None
            report = self.reports[role]
            course = report['route_driver']['funhouse_pendulums']
            entry = dict(role=role, epoch=report['frame']['epoch'], command=report['route_driver']['command'], course=course)
            self.evidence.setdefault('pendulum_courses', []).append(entry)
            for native in course['receipts']:
                if native['step'] != 128:
                    self.evidence['arrivals'].append(dict(entry, step=native['step'], tick=native['tick'],
                        feet=native['feet'], goal=self.steps[native['step']]['pos'], native_pendulums=True))
            self.step = 132
            self.retain(f'pendulums-{role}')
            for step in (133, 134):
                self.step = step
                self.record_arrival(role, self.steps[step]['pos'], jump=step == 133)
            if role == first:
                # The leader only waits after leaving the shared narrow bank.
                parking = self.park(first, other, self.steps[134]['pos'])
        assert all(self.body(r)['stats']['sanity'] > 0 and self.body(r)['spawned_at'] == spawns[r]
                   for r in self.commands), 'Pendulum crossing used a death or revival'
        def waiter_settled():
            body = self.body(first)
            assert body['stats']['sanity'] > 0 and body['player']['grounded'], 'Far-bank waiter fell or died'
            feet = body['player']['feet']
            return distance(feet[:2], parking[:2]) < 24 and abs(feet[2]-parking[2]) < 35
        self.until(waiter_settled, 15)
        footing = dict(step=134, role=first, goal=parking, command=self.commands[first]['id'],
                       tick=self.reports[first]['frame']['tick'])
        verify_waiting_footings([footing], self.history)
        self.evidence.setdefault('waiting_footings', []).append(footing)
        self.completed.extend(range(126, 135))
        print('PASS Funhouse bank/pendulums 126..134: both personal jumps and supported landings', flush=True)

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
                quiet &= not combat_pending(report, role)
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

    def save_checkpoint(self, step):
        # The ordinary menu correctly refuses a save while either player is
        # dead. Let the real revival finish before requesting that same save;
        # a team wipe still fails validate_report instead of becoming a resume.
        if self.allow_revival:
            self.until(lambda: all(self.body(role)['stats']['sanity'] > 0 for role in self.commands), 30)
        ChessPair.save_checkpoint(self, step)

    def verify(self, last_step=None):
        last_step = self.args.last_step if last_step is None else last_step
        assert self.completed == list(range(self.args.start_step, last_step + 1))
        verify_arrivals(self.steps, self.completed, self.evidence['arrivals'], self.history,
                        self.allow_revival, self.evidence.get('scene_handoffs', []))
        verify_following(self.evidence.get('following_footings', []), self.history)
        verify_forward_waits(self.evidence.get('forward_waits', []), self.steps, self.history)
        verify_waiting_footings(self.evidence.get('waiting_footings', []), self.history)
        verify_input_jumps(self.steps, self.completed, self.evidence.get('input_jumps', []), self.history)
        verify_loot(self.evidence.get('loot', []), self.history)
        if 90 in self.completed and not self.gas_already_saved():
            assert self.evidence.get('gas_handoff'), 'Missing gas scene handoff'
            verify_gas_handoff(self.evidence['gas_handoff'], self.history)
        courses = self.evidence.get('pendulum_courses', [])
        assert len(courses) == (2 if 128 in self.completed else 0), 'Missing personal pendulum course'
        if courses:
            assert {e['role'] for e in courses} == {'host', 'guest'}
        for entry in courses:
            verify_pendulum_course(self.steps, entry, self.history)
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
        self.evidence['ice_primary_views'] = {role: [sum(row.get('ice_primary_pulses', [0, 0])[slot] > 0
            for row in rows) for slot in range(2)] for role, rows in self.history.items()}

    def play_steps(self, first_step, last_step):
        for self.step in range(first_step, last_step + 1):
            if self.step in self.completed:
                continue
            if self.step == 90:
                self.gas_scene()
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
            if self.step == 126:
                self.pendulum_course()
                continue
            if self.step == 103:
                self.seesaw_exit()
                continue
            if self.step == 144:
                self.arena_ramp(144, 146)
                continue
            if self.step == 147:
                self.upper_bridge()
                continue
            if self.step == 151:
                self.left_pad()
                continue
            if self.step == 153:
                self.arena_ramp(153, 155)
                continue
            if self.step == 138:
                self.tweedle_entry()
            elif mode == 'wait':
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
            elif mode == 'ice':
                assert step['pos'][0] in (0, 1), 'Invalid authored Ice preference'
                self.prefer_ice = bool(step['pos'][0])
            else:
                self.go(step['pos'], jump=mode == 'jump', surface=460 if self.step in (101, 103, 104) else None)
            if self.step in (107, 109, 112, 116):
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
            print('PASS Funhouse step', self.step, 'scene entrance' if self.step == 138 else mode or 'walk', 'health',
                  [self.body(r)['stats']['sanity'] for r in self.commands], flush=True)

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
        self.play_steps(self.args.start_step, self.args.last_step)
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
            scope=f'Funhouse steps {self.args.start_step}..{self.args.last_step}; personal movement arrivals and scene handoffs',
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
    if not 0 <= args.start_step <= args.last_step <= 155 or args.last_step < 1:
        parser.error('The runner currently supports steps 0..155')
    if bool(args.checkpoint) != (args.start_step > 0):
        parser.error('A later start requires an unchanged checkpoint; a fresh run starts at zero')
    if 29 < args.start_step < 33 or 28 < args.last_step < 32:
        parser.error('The queued mirror sequence must include steps 29..32 together')
    if 95 < args.start_step < 99 or 94 < args.last_step < 98:
        parser.error('The queued fulcrum crossing must include steps 95..98 together')
    if 103 < args.start_step < 107 or 102 < args.last_step < 106:
        parser.error('The queued seesaw exit must include steps 103..106 together')
    if 126 < args.start_step < 135 or 125 < args.last_step < 134:
        parser.error('The queued bank and pendulums must include steps 126..134 together')
    if 144 < args.start_step < 147 or 143 < args.last_step < 146:
        parser.error('The paired arena climb must include steps 144..146 together')
    if 147 < args.start_step < 150 or 146 < args.last_step < 149:
        parser.error('The queued upper bridge must include steps 147..149 together')
    if 151 < args.start_step < 153 or 150 < args.last_step < 152:
        parser.error('The queued left pad must include steps 151..152 together')
    if 153 < args.start_step < 156 or 152 < args.last_step < 155:
        parser.error('The paired arena descent must include steps 153..155 together')
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
