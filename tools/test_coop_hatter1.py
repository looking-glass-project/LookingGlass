"""Drive Clockwork prefixes in independent clients through the real network.

Each player must earn a separate waypoint receipt. The first stops at a native
clear/supported neighbor to let the second arrive. The initial runner supports
nav/walk/node/wait/lever steps, watched Hare/Port/Gryphon/Stop handoffs and a paired gap
crossing adapted from steps 36-42, sequential first-portal jumps at 136-137,
chair contacts/departures at 138-141 and the lowered clock portal at 144.
A scene-triggered reunion is recorded
separately from personal walking arrivals. The first entrant at the original
final exit moves both players into About Face; that shared transition has its
own receipt rather than two claimed personal exit-waypoint arrivals.
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
from test_coop_village import Pair, verify_rejoin_stats
from test_coop_funhouse import waiting_point, combat_pending
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from coop_recovery import read_trace, verify_views, verify_presence as verify_recovery_presence

LEVERS = (376, 378, 289, 694, 44)
THREADS = ('Gear_Bridge_One', 'Gear_Bridge_Two', 'Open_ClockRoom_Door', 'Open_Port', 'stop_clock')
# Supported footings observed in an ordinary successful entrance descent.
DESCENT = [[-2848.6, 3228.5, 320.03125], [-3070.9, 2936.8, 176.03125],
           [-3231.4, 2929.1, 128.03125], [-3296, 2960, 192.03125]]
GEAR_APPROACH = [-4373.3784,3304.2549,208.03125]
GEAR_EXIT_APPROACH = [-4512,3008,208.03125]
CORRIDOR_HANDOFFS = (25,26,27,28,33,34,35,100,101,102,103,104,105,106,107)
STAIR_HANDOFFS = (21,22,23,81,82,83,84,85)
DOOR_HANDOFFS = (128,)
SCENES = {'Hatter1_Cinema1': ('Hare','hare',(60,61,62)), 'Port_Cine': ('Port','port',(67,)),
          'Hatter1_Cinema2': ('Gryphon','gryphon',(112,)), 'Hatter1_End': ('Stop','stopped',(146,))}
CONTACT_SCENES = {'Hatter1_Cinema1': (60,62), 'Hatter1_Cinema2': (112,)}


class SceneCompleted(Exception):
    """The native scene has replaced the in-progress approach with a reunion."""
    def __init__(self, entry): self.entry=entry


class ChairDeath(Exception):
    """An actual death interrupts this player's current chair attempt."""


def scene_finished(clock, flag):
    return clock[flag] is True if flag=='stopped' else clock[flag] is not None


def verify_scene_handoff(entry, history, levers):
    scene=entry['scene'];kind,flag,steps=SCENES[scene]
    assert entry['step'] in steps and entry['role'] in ('host','guest'), 'Unexpected Clockwork scene handoff'
    owner='Blue' if entry['role']=='host' else 'Red';slot=0 if owner=='Blue' else 1;partner=1-slot
    assert set(history)==set(entry['starts'])==set(entry['ends'])=={'host','guest'}, 'Scene needs both clients'
    if kind in ('Port','Stop'):
        step,lever=(66,3) if kind=='Port' else (146,4)
        used=[r for r in levers if r['step']==step and r['lever']==lever]
        assert len(used)==1 and used[0]['role']==entry['role'], 'Scene owner differs from the lever user'
    warps=[]
    for peer,rows in history.items():
        rows=[r for r in rows if entry['starts'][peer]<=r['tick']<=entry['ends'][peer]]
        active=[r for r in rows if r['scene']==scene and r['owner'] is not None]
        assert active and {r['owner'] for r in active}=={owner}, 'Missing or inconsistent scene owner'
        before=active[0]
        assert before['clockwork']['scene']['time']<=.5, 'Scene start was not observed'
        if scene in CONTACT_SCENES and peer==entry['role']:
            assert before['driver']['command']==entry['trigger_command'], 'Scene trigger movement was not observed'
        for row in active:
            clock=row['clockwork'];native=clock['scene']
            assert native['kind']==kind and native['skip'] is None and not scene_finished(clock,flag), 'Scene skipped or completed early'
            assert clock['coop']['actor']==slot and row['camera_player']==owner, 'Scene lost its actor or camera'
            assert not row['presentation']['visible_players'][partner], 'Scene displayed the waiting partner'
            assert row['players'][partner]['feet']==before['players'][partner]['feet'], 'Scene moved the waiting partner'
            for old,body in zip(before['players'],row['players']):
                assert body['stats']==old['stats'] and body['health']==old['health']>0, 'Scene changed personal resources'
                assert body['spawned_at']==old['spawned_at'] and body['warped_at']==old['warped_at'], 'Scene revived or warped a player early'
        if kind in ('Hare','Gryphon'):
            count,opening=(12,31) if kind=='Hare' else (7,5.5)
            assert {r['clockwork']['scene']['line'] for r in active}==set(range(count)), f'Watched {kind} scene missed dialogue'
            assert max(r['clockwork']['scene']['time'] for r in active)>=opening, f'{kind} scene ended before its opening action'
        reunited=[r for r in rows if r['tick']>active[-1]['tick'] and r['owner'] is None and r['scene'] is None
                  and scene_finished(r['clockwork'],flag)]
        assert reunited, 'Missing completed scene and reunion'
        after=reunited[0];a,b=after['players'];warp=a['warped_at']
        assert warp==b['warped_at'] and before['tick']<warp<=after['tick'] and all(warp>p['warped_at'] for p in before['players']), 'Missing shared reunion stamp'
        if kind=='Port':
            assert warp-before['tick']+before['clockwork']['scene']['time']*120>=14.75*120-2, 'Watched Port scene ended early'
            assert max(r['clockwork']['scene']['time'] for r in active)>=14.4, 'Port scene ending was not observed'
        if kind=='Stop':
            assert warp-before['tick']+before['clockwork']['scene']['time']*120>=30*120-2, 'Watched Stop scene ended early'
            assert max(r['clockwork']['scene']['time'] for r in active)>=29.5, 'Stop scene ending was not observed'
        assert all(p['grounded'] and p['health']>0 for p in after['players']) and all(after['driver']['clear']), 'Unsafe scene reunion'
        assert distance(a['feet'][:2],b['feet'][:2])>=35.95 or abs(a['feet'][2]-b['feet'][2])>=55.95, 'Scene reunion overlaps players'
        assert distance(a['feet'][:2],b['feet'][:2])<=256 and abs(a['feet'][2]-b['feet'][2])<=256, 'Scene reunion separates partners'
        for old,body in zip(before['players'],after['players']):
            assert body['health']==old['health'] and body['spawned_at']==old['spawned_at'], 'Reunion changed player life'
            verify_rejoin_stats(old['stats'],body['stats'],after['tick']-warp)
        assert after['presentation']['visible_players'][1 if peer=='host' else 0], 'Reunited partner remains hidden'
        warps.append(warp)
    assert len(set(warps))==1, 'Clients disagree on the reunion'


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


def verify_contact_handoff(entry, steps, receipts, history):
    assert entry['step'] in CONTACT_SCENES.get(entry['scene'],()), 'Only an expected native contact replaces a route approach'
    verify_scene_handoff(entry,history,[])
    step=entry['step'];goal=next(iter(steps[step].values()));pair=[r for r in receipts if r['step']==step]
    park=entry.get('parking_approach')
    if park:
        assert len(pair)==1 and pair[0]['role']==entry['role'], 'Scene parking needs the first personal arrival'
        assert park['step']==step and park['role']==entry['role'] and 'tick' not in park, 'Interrupted parking must not claim arrival'
        row=next(r for r in history[entry['role']] if r['tick']==park['selected_tick'])
        slot=0 if entry['role']=='host' else 1
        assert park['goal'] in row['driver']['neighbors'][slot]==park['candidates'], 'Scene approach was not an observed neighbor'
        assert pair[0]['tick']<=park['selected_tick']<entry['starts'][entry['role']], 'Scene parking preceded its waypoint arrival'
        assert entry['interrupted_goal']==park['goal'], 'Scene movement differs from its parking approach'
    else:
        assert not pair and entry['interrupted_goal']==goal, 'Contact must interrupt the actual course approach'


def verify_arrivals(steps, completed, receipts, history, scene_handoffs=(), revivals=()):
    expected = {i for i in completed if next(iter(steps[i])) in ('nav', 'walk')}
    contacts=[r for r in scene_handoffs if 'interrupted_goal' in r]
    assert len(contacts)==len({r['step'] for r in contacts}) and {r['step'] for r in contacts}<=expected, 'Unexpected contact handoff'
    assert {r['step'] for r in receipts}|{r['step'] for r in contacts} == expected, 'Missing or extra waypoint receipts'
    observed = {role: {r['tick']: r for r in rows} for role, rows in history.items()}
    for index in expected:
        pair = [r for r in receipts if r['step'] == index]
        contact=next((r for r in contacts if r['step']==index),None)
        if contact: verify_contact_handoff(contact,steps,receipts,history)
        else: assert len(pair) == 2 and {r['role'] for r in pair} == {'host', 'guest'}, 'Missing personal arrival'
        goal = next(iter(steps[index].values()))
        for receipt in pair:
            row = observed[receipt['role']].get(receipt['tick'])
            assert row and row['owner'] is None and row['map'] == 'hatter1' and row['epoch'] == 1, 'Arrival not observed in free Clockwork play'
            slot=0 if receipt['role']=='host' else 1;body = row['players'][slot]
            known_spawn=body['spawned_at']==0 or any(e['player']==slot and e['epoch']==row['epoch']
                and e['spawn_tick']==body['spawned_at']<=row['tick'] for e in revivals)
            assert body['health'] > 0 and known_spawn and body['grounded'], 'Invalid arrival body'
            assert receipt['goal'] == goal and receipt['feet'] == body['feet'], 'Receipt differs from observed body'
            assert row['driver']['command'] == receipt['command'] and row['driver']['reached'], 'Arrival command not acknowledged'
            assert distance(body['feet'][:2], goal[:2]) < 8 and abs(body['feet'][2] - goal[2]) < 35, 'Missed personal waypoint'


def parking_target(candidates, at, incoming, goal, visited=(), prefer_forward=True):
    """Find a waiting spot with bounded, observed steps around nearby corners."""
    candidates = [p for p in candidates if distance(p[:2], at[:2]) >= 16
                  and all(distance(p[:2], old[:2]) >= 16 for old in visited)]
    direction = [goal[i] - incoming[i] for i in range(2)]
    def ahead(point):
        return sum((point[i] - goal[i]) * direction[i] for i in range(2)) > 8 * math.hypot(*direction)
    usable = [p for p in candidates if distance(p[:2], goal[:2]) >= 48 and distance(p[:2], incoming[:2]) >= 48]
    if usable and not prefer_forward:
        return waiting_point(usable, at, incoming, goal), True
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
    if farther:
        return max(farther, key=lambda p: distance(p[:2], goal[:2])), False
    # A corner may need a lateral step before the next native neighbor ring
    # clears the arrival lane. Keep the incoming body reserved and avoid
    # revisiting prior positions; this step never counts as ready parking.
    detours = [p for p in candidates if distance(p[:2], incoming[:2]) >= 48]
    assert detours, 'No observed walking approach to a waiting position'
    return max(detours, key=lambda p: distance(p[:2], goal[:2])), False


def recorded_arrival(row, report, command):
    """Wait for an acknowledgment retained alongside the authoritative frame."""
    return bool(row and (row['epoch'],row['tick']) == (report['frame']['epoch'],report['frame']['tick'])
        and row['driver']['command'] == command and row['driver']['reached'])


def recorded_node(row, report, command, entity):
    projection=report['route_driver'].get('pathnode')
    return bool(row and projection and projection['command']==command and projection['entity']==entity
        and (row['epoch'],row['tick'])==(report['frame']['epoch'],report['frame']['tick'])
        and row['driver']['command']==command and row['driver'].get('pathnode')==projection)


def verify_node_resolutions(source, effective, completed, receipts, history, entities):
    expected={i for i in completed if 'node' in source[i]}
    assert {r['step'] for r in receipts}==expected, 'Missing or extra pathnode projections'
    for step in expected:
        pair=[r for r in receipts if r['step']==step]
        assert len(pair)==2 and {r['role'] for r in pair}=={'host','guest'}, 'Missing personal pathnode projection'
        entity=source[step]['node'];native=entities[entity]
        assert native['classname']=='info_pathnode', 'Wrong native pathnode class'
        origin=[float(v) for v in native['origin'].split()]
        assert len(origin)==3 and all(math.isfinite(v) and abs(v)<1_000_000 for v in origin), 'Invalid pathnode origin'
        for receipt in pair:
            row=next(r for r in history[receipt['role']] if r['tick']==receipt['tick'])
            projection=receipt['projection'];goal=projection['feet']
            assert row['owner'] is None and row['map']=='hatter1' and row['epoch']==projection['epoch']==1, 'Pathnode projection outside free play'
            assert row['driver']['command']==receipt['command']==projection['command'] and row['driver'].get('pathnode')==projection, 'Pathnode projection unacknowledged'
            assert not row['driver']['reached'] and row['driver']['phase']=='observing pathnode', 'Projection must not claim arrival'
            assert projection['entity']==entity and projection['tick']<=row['tick'] and distance(projection['origin'],origin)<.001, 'Projection names a different pathnode'
            assert len(goal)==3 and all(math.isfinite(v) for v in goal) and distance(goal[:2],origin[:2])<.001 and origin[2]-208<=goal[2]<=origin[2]+40, 'Projection outside native footing bounds'
            assert set(effective[step])=={'nav'} and distance(effective[step]['nav'],goal)<.01, 'Clients disagree about the pathnode footing'


def next_stair_footing(steps, step):
    assert step in STAIR_HANDOFFS, 'Forward waiting is limited to the stair flights'
    key,label,rise=('walk','walking',48) if step<81 else ('nav','navigation',64)
    following=steps[step+1]
    assert set(steps[step])==set(following)=={key}, f'Next stair footing must be an ordinary {label} step'
    goal,point=steps[step][key],following[key]
    assert 48 <= distance(goal[:2],point[:2]) <= 192 and 0 <= point[2]-goal[2] <= rise, 'Invalid next stair footing'
    return point


def verify_stair_waiting(receipts, steps, arrivals, history):
    verify_forward_waiting(receipts,steps,arrivals,history,next_stair_footing,'Stair',2)


def next_corridor_footing(steps, step):
    assert step in CORRIDOR_HANDOFFS, 'Forward navigation is limited to the laboratory and return corridors'
    assert set(steps[step])=={'nav'} and set(steps[step+1])=={'nav'}, 'Corridor handoff cannot skip a non-navigation action'
    goal,point=steps[step]['nav'],steps[step+1]['nav']
    height=abs(point[2]-goal[2])
    assert 48 <= distance(goal[:2],point[:2]) <= 256 and (height<=64 if step>=100 else height<16), 'Invalid next corridor footing'
    return point


def verify_corridor_waiting(receipts, steps, arrivals, history):
    verify_forward_waiting(receipts,steps,arrivals,history,next_corridor_footing,'Corridor',8)


def next_door_footing(steps, step):
    assert step in DOOR_HANDOFFS, 'Door handoff is limited to the return doorway'
    assert set(steps[step])=={'walk'} and steps[step+1]=={'wait':1.7} and set(steps[step+2])=={'walk'}, 'Door handoff must retain its opening wait'
    start,goal=steps[step]['walk'],steps[step+2]['walk']
    assert 48<=distance(start[:2],goal[:2])<=192 and abs(goal[2]-start[2])<16, 'Invalid door waiting footing'
    return goal


def verify_door_waiting(receipts, steps, arrivals, history, entities):
    verify_forward_waiting(receipts,steps,arrivals,history,next_door_footing,'Door',8)
    slots=doorway_slots(entities,(4,5))
    for receipt in receipts:
        hold=receipt['opening_hold'];role=receipt['role'];slot=0 if role=='host' else 1
        first_arrival=next(r for r in arrivals if r['step']==receipt['step'] and r['role']==role)
        assert first_arrival['tick']<=hold['start_tick'], 'Door opening wait preceded arrival'
        rows=[r for r in history[role] if hold['start_tick']<=r['tick']<=hold['end_tick']]
        assert rows and rows[0]['tick']==hold['start_tick'] and rows[-1]['tick']==hold['end_tick']==receipt['before_tick'], 'Door opening hold unobserved'
        assert hold['end_tick']-hold['start_tick']>=204, 'Door opening wait too short'
        goal=steps[receipt['step']]['walk'];first=rows[0]['players'][slot]
        for row in rows:
            body=row['players'][slot]
            assert body['grounded'] and body['health']>0 and distance(body['feet'][:2],goal[:2])<8 and abs(body['feet'][2]-goal[2])<35, 'Door opening wait left its footing'
            assert body['warped_at']==first['warped_at'] and body['spawned_at']==0, 'Door opening wait warped or revived'
        assert all(rows[-1]['clockwork']['doors'][k]>=.999 for k in slots), 'Door crossing began before both leaves opened'


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
    if last>=148:
        assert start<=148 and last==149, 'The paired exit needs the complete 148-149 sequence'
    if start<=42 and last>=36:
        assert start<=36 and last>=42, 'The paired gap needs the complete 36-42 sequence'
        validate_gap_course(steps)
    if start<=137 and last>=136:
        assert start<=136 and last>=137, 'The paired portal needs the complete 136-137 sequence'
        validate_first_portal_course(steps)
    assert all(next(iter(steps[i])) in ('nav','walk','node','wait','lever') or 36<=i<=42
               or i==137 or (i==144 and steps[i]=={'portal':[-768,1962,184]})
               for i in range(start,last+1)), 'Requested prefix needs an unimplemented input type'


def validate_first_portal_course(steps):
    assert steps[136]=={'nav':[-2800,2272,256.03125]} and steps[137]=={'input':[1,1,0,0,1,0]}, 'Review changed portal jump course'


def verify_first_portal(evidence, steps, history, entities):
    validate_first_portal_course(steps)
    portal,dest=entities[45],entities[131]
    assert portal['classname']=='trigger_teleport' and dest['classname']=='func_teleportdest' and portal['target']==dest['targetname'], 'Wrong native first portal'
    origin=[float(v) for v in portal['origin'].split()];destination=[float(v) for v in dest['origin'].split()]
    jumps=evidence['portal_jumps'];parks=evidence['portal_waiting']
    verify_portal_crossings(jumps,parks,steps[136]['nav'],origin,destination,history)
    assert all(next(r for r in history[j['role']] if r['tick']==j['before_tick'])['players'][0 if j['role']=='host' else 1]['spawned_at']==0
               for j in jumps), 'First portal resumed an unverified revival'


def verify_clock_portal(evidence, steps, history, entities):
    assert steps[142]=={'nav':[-768,2056,64.03125]} and steps[144]=={'portal':[-768,1962,184]}, 'Review changed clock portal course'
    portal,dest=entities[125],entities[124]
    assert portal['classname']=='trigger_teleport' and portal['targetname']=='clock_teleport' \
        and dest['classname']=='func_teleportdest' and portal['target']==dest['targetname'], 'Wrong native clock portal'
    base=[float(v) for v in portal['origin'].split()]
    origin=[base[0],base[1]+70,base[2]-360]
    assert origin==steps[144]['portal'], 'Wrong lowered clock portal position'
    jumps=evidence['clock_portal_jumps'];parks=evidence['clock_portal_waiting']
    for jump in jumps:
        for rows in history.values():
            approach=[r for r in rows if jump['before_tick']<=r['tick']<=jump['warp']]
            assert approach and all(all(r['clockwork']['cubes']) and r['clockwork']['clock'] is not None
                and r['clockwork']['age']-r['clockwork']['clock']>=5 for r in approach), 'Clock portal entered before puzzle lowering'
    destination=[float(v) for v in dest['origin'].split()]
    verify_portal_crossings(jumps,parks,steps[142]['nav'],origin,destination,history,onward=steps[145]['nav'])
    return {(1,0 if j['role']=='host' else 1,j['warp']) for j in jumps}


def verify_portal_crossings(jumps, parks, takeoff, origin, destination, history, onward=None):
    assert len(jumps)==2 and {j['role'] for j in jumps}=={'host','guest'}, 'Missing personal portal crossing'
    for jump in jumps:
        role=jump['role'];slot=0 if role=='host' else 1;warps=[]
        own=next(r for r in history[role] if r['tick']==jump['before_tick'])
        before=own['players'][slot]
        assert before['grounded'] and distance(before['feet'][:2],takeoff[:2])<8 \
            and abs(before['feet'][2]-takeoff[2])<35, 'Portal jump did not start on the takeoff point'
        assert any(jump['before_tick']<=r['tick']<=jump['warp'] and r['driver']['command']==jump['command'] for r in history[role]), 'Portal jump command unobserved'
        events=[e for r in history['host'] for e in r['driver'].get('input_events',[])]
        assert any(e['kind']=='jump' and jump['before_tick']<=e['tick']<jump['warp'] and e['input'][slot]
            and e['before'][slot][1] and not e['after'][slot][1]
            and (e['after'][slot][2]==e['before'][slot][2]+1 or
                 (e['after'][slot][2]==e['before'][slot][2] and e['after'][slot][0][2]>e['before'][slot][0][2]
                  and any(e['tick']<=r['tick']<jump['warp'] and not r['players'][slot]['grounded']
                          and r['players'][slot]['climbs']==before['climbs']+1 for r in history[role])))
            for e in events), 'Portal takeoff unobserved'
        assert not any(e['kind']=='jump' and jump['warp']<=e['tick']<=jump['arrival_tick'] and e['input'][slot]
                       for e in events), 'Portal jump replayed at the destination'
        for peer,rows in history.items():
            changed=[r for r in rows if r['tick']>=jump['before_tick'] and r['players'][slot]['warped_at']>before['warped_at']]
            assert changed, 'Missing replicated portal warp'
            after=changed[0];body=after['players'][slot];warp=body['warped_at']
            assert warp==jump['warp'] and warp>jump['before_tick'] and after['tick']-warp<=60, 'Wrong portal warp stamp'
            assert body['spawned_at']==before['spawned_at'] and body['health']>0 and after['owner'] is None, 'Portal was replaced by revival or a scene'
            assert distance(body['feet'][:2],destination[:2])<64 and abs(body['feet'][2]-destination[2])<128, 'Portal reached the wrong destination'
            approach=[r for r in rows if jump['before_tick']<=r['tick']<warp]
            assert approach and distance(approach[-1]['players'][slot]['feet'][:2],origin[:2])<180 and abs(approach[-1]['players'][slot]['feet'][2]-origin[2])<192, 'Portal source contact unobserved'
            warps.append(warp)
        assert len(set(warps))==1, 'Clients disagree on personal portal warp'
        end=next(r for r in history[role] if r['tick']==jump['arrival_tick']);body=end['players'][slot]
        assert body['warped_at']==jump['warp'] and body['grounded'] and end['driver']['clear'][slot], 'Portal landing is not safe'
    assert len(parks)==1, 'Missing portal waiting handoff'
    park=parks[0];first=jumps[0];second=jumps[1];role=first['role'];slot=0 if role=='host' else 1
    assert park['role']==role and first['arrival_tick']<=park['selected_tick']<park['tick']<second['before_tick'], 'Portal handoff out of order'
    selected=next(r for r in history[role] if r['tick']==park['selected_tick'])
    if park.get('source')=='arrival':
        body=selected['players'][slot]
        assert park['goal']==body['feet'] and body['grounded'] and selected['driver']['clear'][slot] \
            and body['immersion']['level']==0, 'Portal waiting point was not a dry supported arrival'
        assert distance(park['goal'][:2],destination[:2])>=48, 'Portal arrival has insufficient waiting clearance'
    elif park.get('source')=='onward':
        assert onward is not None and park['goal']==onward, 'Portal waiting must use the original onward waypoint'
    else:
        assert park['goal'] in selected['driver']['neighbors'][slot]==park['candidates'], 'Portal waiting point was not observed'
    arrived=next(r for r in history[role] if r['tick']==park['tick']);body=arrived['players'][slot]
    assert arrived['driver']['command']==park['command'] and arrived['driver']['reached'] and body['grounded'], 'Portal waiting arrival unacknowledged'
    assert body['feet']==park['feet'] and distance(body['feet'][:2],park['goal'][:2])<8, 'Portal waiting arrival missed'
    assert abs(body['feet'][2]-park['goal'][2])<35 and arrived['driver']['clear'][slot], 'Portal waiting arrival obstructed or wrong height'
    assert body['warped_at']==first['warp'] and body['spawned_at']==selected['players'][slot]['spawned_at'], 'Portal waiting used another warp or revival'
    assert distance(body['feet'][:2],destination[:2])>=35.95 or abs(body['feet'][2]-destination[2])>=55.95, 'Portal waiter obstructs the arrival'


def verify_clockwork_exit(entry, steps, history):
    assert steps[148]=={'nav':[-2528,1848,192.03125]} and steps[149]=={'walk':[-2528,1764,192.03125]}, 'Review changed Clockwork exit course'
    role=entry['role'];slot=0 if role=='host' else 1
    approach=next(r for r in history[role] if r['epoch']==1 and r['tick']==entry['approach_tick'])
    body=approach['players'][slot]
    assert approach['map']=='hatter1' and approach['owner'] is None and body['grounded'] and body['health']>0 \
        and distance(body['feet'][:2],steps[148]['nav'][:2])<8 and abs(body['feet'][2]-steps[148]['nav'][2])<35, 'Exit approach was not personally reached'
    assert approach['driver']['command']==entry['approach_command'] and approach['driver']['reached'], 'Exit approach command unobserved'
    snapshots={}
    for peer,rows in history.items():
        source=[r for r in rows if r['map']=='hatter1' and r['epoch']==1]
        arrived=[r for r in rows if r['map']=='hatter2' and r['epoch']==2]
        assert source and arrived and source[-1]['tick']<arrived[0]['tick'], 'Missing ordered shared About Face transition'
        assert all(r['visit_entry'] is None for r in arrived), 'Wrong About Face entrance'
        last=source[-1];clock=last['clockwork']
        assert clock['stopped'] and all(t is not None for t in clock['levers']) and all(clock['cubes']) \
            and all(clock[f] is not None for f in ('hare','port','gryphon')), 'Exit bypassed Clockwork progression'
        assert distance(last['players'][slot]['feet'][:2],steps[149]['walk'][:2])<64 \
            and abs(last['players'][slot]['feet'][2]-steps[149]['walk'][2])<64, 'Native exit contact was not approached'
        if peer==role:assert last['driver']['command']==entry['exit_command'], 'Exit movement command unobserved'
        first=arrived[0]
        assert first['tick']==entry['arrival_ticks'][peer] and first['checkpoint_generation']==0, 'Unrecorded arrival or checkpoint wipe'
        assert all(p['health']>0 and p['grounded'] for p in first['players']) and all(first['driver']['clear']), 'Unsafe About Face arrival'
        for old,new in zip(last['players'],first['players']):
            assert new['spawned_at']==old['spawned_at'] and new['health']==old['health'], 'Transition revived or reset a player'
            verify_rejoin_stats(old['stats'],new['stats'],first['tick']-last['tick'])
        snapshots[peer]={(s['epoch'],s['tick']):(s['sha256'],s['world_sha256'],s['native_sha256'])
            for r in arrived for s in r['driver']['network_states'] if s['epoch']==2}
    shared=snapshots['host'].keys()&snapshots['guest'].keys()
    assert len(shared)>=20 and all(snapshots['host'][k]==snapshots['guest'][k] for k in shared), 'About Face state replication incomplete'
    return len(shared)


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


def chair_return(rows, slot, start_tick, before_tick, goal, projected=False):
    """Prefer this player's observed floor, then native floor near the partner."""
    before=next(r for r in rows if r['tick']==before_tick)
    partner=before['players'][1-slot]['feet']
    # Later supports may be partway up the chair beneath its rim. Preserve the
    # earlier approach floor rather than targeting an overhang from above.
    for source_slot in (slot,1-slot) if projected else (slot,):
        for row in rows:
            if not start_tick<=row['tick']<before_tick:continue
            body=row['players'][source_slot]
            if not (body['grounded'] and body.get('ledge') is None and body['health']>0
                    and row['driver']['clear'][source_slot] and body['immersion']['level']==0):continue
            candidates=row['driver'].get('return_landings',[[],[]])[source_slot] if projected else [body['feet']]
            for at in candidates:
                if distance(at[:2],body['feet'][:2])<=48.1 and 56<=goal[2]-at[2]<=256 \
                        and 64<=distance(at[:2],goal[:2])<=384 \
                        and (distance(at[:2],partner[:2])>=48 or abs(at[2]-partner[2])>=64):
                    return row,at,source_slot
    raise AssertionError('Chair approach has no observed dry return footing clear of the partner')


def verify_chair_departures(receipts, steps, arrivals, history, completed):
    expected={i for i in completed if 138<=i<=141}
    assert {(r['step'],r['role']) for r in receipts}=={(i,r) for i in expected for r in ('host','guest')} \
        and len(receipts)==2*len(expected), 'Missing personal chair departure'
    for receipt in receipts:
        step=receipt['step'];role=receipt['role'];slot=0 if role=='host' else 1
        rows=history[role];by_tick={r['tick']:r for r in rows}
        before=by_tick[receipt['before_tick']];end=by_tick[receipt['arrival_tick']]
        arrival=next(r for r in arrivals if r['step']==step and r['role']==role)
        assert arrival['tick']==receipt['before_tick'], 'Chair departure did not follow its personal arrival'
        selected,at,source=chair_return(rows,slot,receipt['approach_tick'],receipt['before_tick'],steps[step]['nav'],step in (140,141))
        assert selected['tick']==receipt['selected_tick'] and receipt['goal']==at \
            and receipt.get('selected_player',slot)==source, 'Chair return was not the observed safe footing'
        verify_grounded_jump(receipt,history)
        flight=by_tick[receipt['after_tick']]
        assert not flight['players'][slot]['grounded'], 'Chair departure flight unobserved'
        body=end['players'][slot]
        assert body['health']>0 and body['grounded'] and body.get('ledge') is None and end['driver']['clear'][slot] \
            and body['immersion']['level']==0, 'Chair departure did not land safely outside liquid'
        assert end['driver']['command']==receipt['command'] and end['driver']['reached'], 'Chair return command unacknowledged'
        assert distance(body['feet'][:2],receipt['goal'][:2])<8 and abs(body['feet'][2]-receipt['goal'][2])<16, 'Chair return missed'
        assert body['jumps']==before['players'][slot]['jumps']+1, 'Chair departure repeated the jump'
        for row in rows:
            if receipt['before_tick']<=row['tick']<=receipt['arrival_tick']:
                b=row['players'][slot];old=before['players'][slot]
                assert row['owner'] is None and b['health']>0 and b['warped_at']==old['warped_at'] and b['spawned_at']==old['spawned_at'], 'Chair departure used a scene, warp or revival'
        cube={138:2,139:1,140:0,141:3}[step]
        assert all(any(r['tick']>=receipt['before_tick'] and r['clockwork']['cubes'][cube] for r in rows) for rows in history.values()), 'Chair contact not replicated'
    for step in expected:
        pair=[r for r in receipts if r['step']==step]
        assert pair[0]['arrival_tick']<pair[1]['before_tick'], 'Follower arrived before the chair was cleared'


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
    # Reserve the same 48-unit lane around the stopped body and its route.
    # Requiring 64 only at the body rejects usable supported corner steps;
    # native personal space remains 36 units throughout the actual movement.
    usable = [p for p in candidates if distance(p[:2],incoming[:2]) >= 48 and clearance(p) >= 48
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
        if pull.get('requires_clear'): assert before['driver']['ledge_pull_clear'][slot], 'Pull-up destination was not clear'
        assert b['climbs'] == a['climbs'] + 1 and b['grounded'] and b['ledge'] is None, 'Native pull-up did not finish'
        assert any(before['tick'] < r['tick'] <= after['tick'] and r['driver']['command'] == pull['command']
                   and r['players'][slot]['ledge'] and r['players'][slot]['ledge']['pulling'] for r in rows), 'Pull-up animation unobserved'
        assert any(e['kind'] == 'jump' and e['epoch'] == 1 and before['tick'] <= e['tick'] < after['tick']
                   and e['input'][slot] and not e['before'][slot][1]
                   and e['before'][slot][2] == e['after'][slot][2]
                   for r in history['host'] for e in r['driver'].get('input_events', [])), 'Pull-up input unobserved'
        if pull.get('step') in (140,141):
            assert 'hold_command' in pull and 'hold_ack_tick' in pull, 'Chair pull needs an acknowledged neutral hold'
            held=next(r for r in rows if r['tick']==pull['hold_ack_tick'])
            requested=next(r for r in rows if r['tick']==pull['hold_tick'])
            assert requested['driver']['command']==pull['command'] and requested['players'][slot]['ledge']['pulling'], 'Chair hold preceded the actual pull'
            assert pull['before_tick']<pull['hold_tick']<held['tick']<after['tick'], 'Chair hold was not applied during the pull'
            assert held['driver']['command']==after['driver']['command']==pull['hold_command'] \
                and held['players'][slot]['ledge']['pulling'], 'Chair hold did not persist through the pull'


def chair_hop_required(step, body, goal, clear):
    # The nearby cushion is separated from this pulled-up rim by harmful liquid.
    # Jump across that gap instead of dropping into it while asking for a route.
    return step in (140,141) and body['grounded'] and body.get('ledge') is None and clear \
        and body['immersion']['level']==0 and 16<=distance(body['feet'][:2],goal[:2])<=160 \
        and abs(body['feet'][2]-goal[2])<35


def verify_chair_hops(pulls, steps, arrivals, history):
    for pull in pulls:
        step=pull.get('step')
        if step not in (140,141):continue
        role=pull['role'];slot=0 if role=='host' else 1
        rows=history[role];by_tick={r['tick']:r for r in rows}
        before=by_tick[pull['after_tick']];goal=steps[step]['nav']
        required=chair_hop_required(step,before['players'][slot],goal,before['driver']['clear'][slot])
        hop=pull.get('chair_hop')
        assert bool(hop)==required, 'Nearby chair cushion requires its own observed hop'
        if not hop:continue
        assert hop['role']==role and hop['before_tick']==pull['after_tick'] and hop['goal']==goal, 'Chair hop must start at its completed pull'
        verify_grounded_jump(hop,history)
        assert not by_tick[hop['after_tick']]['players'][slot]['grounded'], 'Chair hop flight unobserved'
        arrival=next(a for a in arrivals if a['step']==step and a['role']==role)
        assert arrival['tick']==hop['arrival_tick'] and arrival['command']==hop['command'], 'Chair hop must finish at its personal arrival'
        end=by_tick[hop['arrival_tick']];body=end['players'][slot]
        assert body['grounded'] and body.get('ledge') is None and end['driver']['clear'][slot], 'Chair hop did not land clearly'
        assert body['jumps']==before['players'][slot]['jumps']+1, 'Chair hop repeated its jump'
        assert distance(body['feet'][:2],goal[:2])<8 and abs(body['feet'][2]-goal[2])<35, 'Chair hop missed its cushion'
        for row in rows:
            if hop['before_tick']<=row['tick']<=hop['arrival_tick']:
                body=row['players'][slot];old=before['players'][slot]
                assert row['owner'] is None and body['health']>0 and body['immersion']['level']==0, 'Chair hop entered liquid or lost its living body'
                assert body['warped_at']==old['warped_at'] and body['spawned_at']==old['spawned_at'], 'Chair hop warped or revived'


def verify_chair_retries(attempts, revivals, arrivals, departures, history):
    seen=set()
    for attempt in attempts:
        role=attempt['role'];slot=0 if role=='host' else 1;step=attempt['step']
        assert step in (138,139,140,141), 'Recovery retry outside the chair sequence'
        event=next((e for e in revivals if e['player']==slot and e['spawn_tick']==attempt['spawn_tick']),None)
        assert event and (slot,event['spawn_tick']) not in seen, 'Chair retry lacks its own verified revival'
        seen.add((slot,event['spawn_tick']))
        assert attempt['start_tick']<event['death_tick']<=attempt['death_seen_tick']<event['spawn_tick'], 'Chair retry has no intervening death'
        rows=history[role];dead=next(r for r in rows if r['tick']==attempt['death_seen_tick'])
        assert dead['players'][slot]['health']==0 and dead['players'][slot]['spawned_at']==attempt['previous_spawn'], 'Chair retry did not observe the original dead body'
        assert all(a['tick']<=attempt['death_seen_tick'] for a in attempt['receipts']['arrivals']), 'Abandoned arrival follows the death'
        arrival=next(a for a in arrivals if a['step']==step and a['role']==role)
        departure=next(d for d in departures if d['step']==step and d['role']==role)
        assert event['spawn_tick']<=departure['approach_tick']<arrival['tick']==departure['before_tick'], 'Revival replaced the personal chair approach'
        arrived=next(r for r in rows if r['tick']==arrival['tick'])
        assert arrived['players'][slot]['spawned_at']>=event['spawn_tick'], 'Completed arrival belongs to the abandoned life'
    assert seen=={(e['player'],e['spawn_tick']) for e in revivals}, 'Recovery occurred outside a recorded chair retry'


def doorway_slots(entities, doors=(92,93)):
    objects=[i for i,e in enumerate(entities) if e.get('classname') in
             ('script_object','func_door','func_rotatingdoor','func_sinkobject')]
    assert all(entities[i]['classname']=='func_rotatingdoor' for i in doors), 'Review changed doorway'
    return [objects.index(i) for i in doors]


def verify_doorway_approaches(receipts, steps, history, entities):
    slots=doorway_slots(entities)
    for receipt in receipts:
        assert receipt['step'] in (107,108), 'Unexpected doorway approach'
        goal=steps[109]['nav']
        assert receipt['goal']==goal, 'Doorway approach differs from the course'
        role=receipt['role']; slot=0 if role=='host' else 1
        rows=history[role]
        arrived=next(r for r in rows if r['tick']==receipt['tick'])
        body=arrived['players'][slot]
        assert arrived['driver']['command']==receipt['command'] and arrived['driver']['reached'], 'Doorway approach not acknowledged'
        assert body['grounded'] and body['health']>0 and arrived['driver']['clear'][slot], 'Invalid doorway footing'
        assert body['feet']==receipt['feet'] and distance(body['feet'][:2],goal[:2])<8 and abs(body['feet'][2]-goal[2])<35, 'Doorway approach missed'
        end=next(r for r in rows if r['tick']==receipt['end_tick'])
        lead=receipt['lead_in']
        lead_row=next(r for r in rows if r['tick']==lead['tick'])
        lead_body=lead_row['players'][slot]
        assert lead['goal']==steps[116]['walk'] and lead_row['tick']<arrived['tick'], 'Missing centered doorway lead-in'
        assert lead_row['driver']['command']==lead['command'] and lead_row['driver']['reached'], 'Doorway lead-in not acknowledged'
        assert lead_body['grounded'] and lead_body['health']>0 and lead_row['driver']['clear'][slot], 'Invalid doorway lead-in footing'
        assert lead_body['feet']==lead['feet'] and distance(lead_body['feet'][:2],lead['goal'][:2])<8 and abs(lead_body['feet'][2]-lead['goal'][2])<35, 'Doorway lead-in missed'
        assert end['tick']-arrived['tick']>=204, 'Doorway opening hold too short'
        for row in rows:
            if lead_row['tick']<=row['tick']<=end['tick']:
                b=row['players'][slot]
                assert b['spawned_at']==lead_body['spawned_at']==0 and b['warped_at']==lead_body['warped_at'], 'Doorway approach warped or revived'
                if row['tick']>=arrived['tick']:
                    assert b['grounded'] and distance(b['feet'][:2],goal[:2])<8, 'Doorway hold left its footing'
        assert all(end['clockwork']['doors'][k]>=.999 for k in slots), 'Doorway did not open fully'


class ClockworkPair(Pair):
    map_name, entry, next_map = 'hatter1', 'hatter1_start1', 'hatter2'
    evidence_prefix, capture_scenes = 'hatter1-route-', False

    def __init__(self, args):
        self.allow_revival=getattr(args,'allow_revival',False)
        self.moving_role=None
        self.step, self.completed, self.last_valid = -1, [], {}
        self.first = 'host' if args.first == 'blue' else 'guest'
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.lead, self.lead_tick = self.minimum_lead, None
        super().__init__(args)
        self.deadline = time.monotonic() + 1500
        raw = (ROOT / 'src/levels/hatter1/route_steps.json').read_bytes()
        self.steps = json.loads(raw)
        self.source_steps=json.loads(raw)
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
            arrivals=[], levers=[], holds=[], parking=[], stair_waiting=[], corridor_waiting=[], yields=[], descent_arrivals=[], descent_jumps=[], ledge_pulls=[], gear_boarding=[], gear_exits=[], gap_jumps=[], gap_holds=[], gap_waiting=[], node_resolutions=[],
            scene_handoffs=[], doorway_approaches=[], door_waiting=[], portal_jumps=[], portal_waiting=[], clock_portal_jumps=[], clock_portal_waiting=[], chair_departures=[], failed_chair_attempts=[], port_player=args.port_player, completed=self.completed, scope='Fresh Clockwork prefix only')
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
        self.commands[role]['ledge_handoff']=138<=self.step<=141 and self.commands[role]['navigate']
        self.commands[role]['landing_feedback']=self.step in (140,141) and self.commands[role]['navigate']
        if getattr(self,'allow_revival',False):self.commands[role]['recovery_trace']=role=='host'

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
        if getattr(self,'allow_revival',False):
            assert report['frame']['checkpoint_generation']==0, 'Unexpected team wipe; separate restore proof required'
            assert report['route_driver'].get('recovery_trace_error') is None, 'Host recovery trace failed'
        else:super().validate_report(role, report)
        assert report['build'] == self.args.build, 'Executable differs from the frozen build'
        allowed={(self.map_name,1)}
        if self.step==149 and self.args.last_step==149:allowed.add((self.next_map,2))
        assert (report['visit']['map'],report['visit']['epoch']) in allowed, 'Unexpected visit during prefix'
        assert all(not b['stats']['god'] and not b['stats']['notarget'] for b in report['frame']['players']), 'Cheated route'
        assert getattr(self,'allow_revival',False) or all(b['spawned_at'] == 0 for b in report['frame']['players']), 'Prefix silently respawned a player'
        if report['combat']['stage']['owner'] is None and all(b['stats']['sanity']>0 for b in report['frame']['players']):
            a, b = [x['player']['feet'] for x in report['frame']['players']]
            if distance(a[:2], b[:2]) < 35.95 and abs(a[2] - b[2]) < 55.95:
                (self.folder / f'overlap-{role}.json').write_text(json.dumps(report))
                raise AssertionError('Living players overlap')
        self.last_valid[role] = report

    def validate_clear(self, role, report):
        assert all(clear for clear,body in zip(report['route_driver']['clear'],report['frame']['players'])
            if body['stats']['sanity']>0), f'{role}: obstructed living body'

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
            self.history[role][-1]['clockwork'] = self.state(role) if report['visit']['map']==self.map_name else None
            self.history[role][-1]['visit_entry'] = report['visit']['entry']
            self.history[role][-1].setdefault('presentation',report['presentation'])
            for observed, body in zip(self.history[role][-1]['players'], report['frame']['players']):
                observed.update(ledge=body['player'].get('ledge'), climbs=body['player']['climbs'],immersion=body['player']['immersion'])
        role=getattr(self,'moving_role',None)
        if getattr(self,'allow_revival',False) and role is not None:
            if self.body(role)['stats']['sanity']<=0:raise ChairDeath(role)
            other='guest' if role=='host' else 'host'
            assert self.body(other)['stats']['sanity']>0, 'Waiting partner died; this chair attempt cannot complete'

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
        if self.step in (107,108) and goal==self.steps[108]['nav'] and distance(self.body(role)['player']['feet'][:2],goal[:2])>=radius:
            # Approach the rotating pair along the centered, authored waiting
            # waypoint. Cutting diagonally into its left leaf blocks the swing.
            # These are ordinary inputs; retain the original arrival afterwards.
            lead_goal=self.steps[116]['walk']
            self.move(role,lead_goal,navigate=True)
            report=self.reports[role]
            lead=dict(goal=lead_goal,tick=report['frame']['tick'],command=report['route_driver']['command'],
                feet=list(self.body(role)['player']['feet']))
            approach=self.steps[109]['nav']
            self.move(role,approach)
            report=self.reports[role]
            receipt=dict(step=self.step,role=role,goal=approach,tick=report['frame']['tick'],
                command=report['route_driver']['command'],feet=list(self.body(role)['player']['feet']),lead_in=lead)
            self.set(role,approach,run=True,attack=False)
            slots=doorway_slots(self.entities)
            self.until(lambda:self.reports[role]['frame']['tick']>=receipt['tick']+204
                and all(self.state(role)['doors'][k]>=.999 for k in slots),15)
            receipt['end_tick']=self.reports[role]['frame']['tick']
            verify_doorway_approaches([receipt],self.steps,self.history,self.entities)
            self.evidence['doorway_approaches'].append(receipt)
        self.set(role, goal, navigate=navigate, run=True, jump=jump)
        self.commands[role]['arrival_radius'] = radius
        command = self.commands[role]['id']
        pullup = None
        chair_hop = None
        yielded = 0
        cinematic = None
        best = (distance(self.body(role)['player']['feet'], goal), self.reports[role]['frame']['tick'])
        def done():
            nonlocal best, navigate, pullup, chair_hop, yielded, command, cinematic
            body = self.body(role)['player']; report = self.reports[role]
            if chair_hop and 'after_tick' not in chair_hop:
                before=next(r for r in self.history[role] if r['tick']==chair_hop['before_tick'])
                if body['jumps']==before['players'][0 if role=='host' else 1]['jumps']+1 and not body['grounded']:
                    chair_hop['after_tick']=report['frame']['tick']
            if report['combat']['stage']['owner'] is not None:
                assert self.step in CONTACT_SCENES.get(report['route_driver']['scene'],()), 'Unexpected cinematic in ordinary prefix movement'
                assert report['combat']['stage']['owner']==('Blue' if role=='host' else 'Red'), 'Moving player did not own the contact scene'
                command=report['route_driver']['command']
                cinematic=report['route_driver']['scene']
                return True
            gap = distance(body['feet'], goal); tick = report['frame']['tick']
            if pullup is not None:
                slot=0 if role=='host' else 1
                before=next(r for r in self.history[role] if r['tick']==pullup['before_tick'])
                hang=body.get('ledge')
                if self.step in (140,141) and hang and hang['pulling']:
                    if 'hold_command' not in pullup and report['route_driver']['command']==pullup['command']:
                        # The native animation completes without held movement.
                        # Release the approach now, before network delay can walk
                        # the finished body off the narrow rim ahead of its hop.
                        self.set(role,attack=False)
                        pullup.update(hold_tick=tick,hold_command=self.commands[role]['id'])
                    elif report['route_driver']['command']==pullup.get('hold_command'):
                        pullup.setdefault('hold_ack_tick',tick)
                if body['climbs']==before['players'][slot]['climbs']+1 and body['grounded'] and body.get('ledge') is None:
                    pullup['after_tick']=tick
                    verify_ledge_pulls([pullup],self.history)
                    completed_pull=pullup
                    pullup=None
                    hop=chair_hop_required(self.step,body,goal,report['route_driver']['clear'][slot]) if self.step in (140,141) else False
                    navigate=not hop
                    self.set(role,goal,navigate=navigate,run=True,jump=hop,attack=not hop)
                    self.commands[role]['arrival_radius']=radius
                    command=self.commands[role]['id']
                    if hop:
                        chair_hop=dict(role=role,before_tick=tick,goal=goal,command=command)
                        completed_pull['chair_hop']=chair_hop
                    best=gap,tick
                    return False
            if self.reached(role) and body['grounded'] and distance(body['feet'][:2], goal[:2]) < radius and abs(body['feet'][2]-goal[2]) < 35:
                if chair_hop:
                    assert 'after_tick' in chair_hop, 'Chair hop reached without observed flight'
                    chair_hop['arrival_tick']=tick
                return True
            hang = body.get('ledge')
            needs_clear=138<=self.step<=141
            if hang and not hang['pulling'] and pullup is None and goal[2] >= hang['top'] - 35 and sum(
                    (goal[i]-body['feet'][i])*hang['forward'][i] for i in range(2)) > 0 \
                    and (not needs_clear or report['route_driver']['ledge_pull_clear'][0 if role=='host' else 1]):
                self.set(role, goal, run=True, jump=True,attack=not needs_clear)
                self.commands[role]['arrival_radius'] = radius
                navigate=False; command=self.commands[role]['id']
                pullup = dict(step=self.step,role=role, before_tick=tick, command=command,requires_clear=needs_clear)
                self.evidence['ledge_pulls'].append(pullup)
            if gap < best[0] - 2: best = gap, tick
            elif yield_partner is not None and not jump and pullup is None and chair_hop is None and tick-best[1] > 240:
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
            if not navigate and not jump and pullup is None and chair_hop is None and tick - best[1] > 240 and body['grounded']:
                navigate = True; self.set(role, goal, navigate=True, run=True)
                self.commands[role]['arrival_radius'] = radius
                command=self.commands[role]['id']
            return False
        # These longer approaches may need another asynchronous physics search
        # after a corrected landing. Bound that computation separately from
        # the short departure/landing checks; all movement proof stays required.
        self.until(done,seconds=180 if self.step in (140,141) and navigate else 90)
        if cinematic:
            entry=self.watch_scene(cinematic,role,command)
            entry['interrupted_goal']=goal
            raise SceneCompleted(entry)
        assert pullup is None, 'Arrival did not observe a completed native pull-up'
        return command

    def watch_scene(self, scene, role, trigger_command):
        kind,flag,steps=SCENES[scene]
        assert self.step in steps and not any(r['scene']==scene for r in self.evidence['scene_handoffs']), 'Unexpected repeated scene'
        owner='Blue' if role=='host' else 'Red'
        for peer in self.commands: self.set(peer,attack=False)
        self.until(lambda:all(r['route_driver']['scene']==scene and r['combat']['stage']['owner']==owner
                             for r in self.reports.values()),15)
        starts={peer:next(r['tick'] for r in rows if r['scene']==scene and r['owner'] is not None)
                for peer,rows in self.history.items()}
        self.until(lambda:all(r['combat']['stage']['owner'] is None and scene_finished(self.state(peer),flag)
            and all(b['player']['grounded'] for b in r['frame']['players']) for peer,r in self.reports.items()),240)
        entry=dict(step=self.step,scene=scene,role=role,trigger_command=trigger_command,starts=starts,
                   ends={peer:r['frame']['tick'] for peer,r in self.reports.items()})
        verify_scene_handoff(entry,self.history,self.evidence['levers'])
        self.evidence['scene_handoffs'].append(entry)
        print('PASS watched Clockwork scene',kind,owner,'and shared reunion',flush=True)
        return entry

    def wait_port(self, seconds):
        assert self.step==67
        lever=next(r for r in self.evidence['levers'] if r['step']==66 and r['lever']==3)
        start=self.reports['host']['frame']['tick']
        for peer in self.commands: self.set(peer,attack=False)
        self.until(lambda:any(r['route_driver']['scene']=='Port_Cine' for r in self.reports.values()),20)
        self.watch_scene('Port_Cine',lever['role'],lever['command'])
        self.until(lambda:self.reports['host']['frame']['tick']>=start+seconds*120)

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
        self.wait_for_partner(role,park)

    def leave_chair(self, role, goal, approach_tick):
        before=self.reports[role];slot=0 if role=='host' else 1
        selected,at,source=chair_return(self.history[role],slot,approach_tick,before['frame']['tick'],goal,self.step in (140,141))
        park=list(at)
        receipt=dict(step=self.step,role=role,approach_tick=approach_tick,
            selected_tick=selected['tick'],selected_player=source,before_tick=before['frame']['tick'],goal=park)
        count=self.body(role)['player']['jumps']
        # Leave the sinking, harmful cushion immediately. A separate ordinary
        # jump returns to observed dry ground, freeing the chair for the partner.
        self.set(role,park,run=True,jump=True,attack=False)
        receipt['command']=self.commands[role]['id']
        self.evidence['chair_departures'].append(receipt)
        self.until(lambda:self.reports[role]['route_driver']['command']==receipt['command']
            and self.body(role)['player']['jumps']==count+1 and not self.body(role)['player']['grounded'],15)
        receipt['after_tick']=self.reports[role]['frame']['tick']
        self.until(lambda:self.reached(role) and self.body(role)['player']['grounded']
            and self.body(role)['player']['immersion']['level']==0
            and abs(self.body(role)['player']['feet'][2]-park[2])<16,30)
        receipt['arrival_tick']=self.reports[role]['frame']['tick']
        self.wait_for_partner(role,park)
        print('PASS ordinary chair departure',self.step,role,flush=True)

    def wait_for_partner(self, role, park):
        # These return stairs and rooms are static ground. Once clear of the
        # follower, let ordinary combat controls dodge instead of continually
        # walking back into an enemy's punches. Moving machinery still needs
        # its existing position hold.
        if 81<=self.step<=108: self.set(role,attack=True)
        else: self.set(role,park,run=True)

    def resolve_node(self, entity):
        assert self.entities[entity]['classname']=='info_pathnode', 'Expected an authored pathnode'
        for role in self.commands:
            self.set(role,attack=False)
            self.commands[role]['pathnode']=entity
        self.until(lambda:all(recorded_node(self.history[r][-1] if self.history[r] else None,
            self.reports[r],self.commands[r]['id'],entity) for r in self.commands),15)
        receipts=[]
        for role in self.commands:
            report=self.reports[role]
            receipts.append(dict(step=self.step,role=role,command=self.commands[role]['id'],tick=report['frame']['tick'],
                projection=report['route_driver']['pathnode']))
        goal=list(receipts[0]['projection']['feet'])
        self.steps[self.step]={'nav':goal}
        verify_node_resolutions(self.source_steps,self.steps,[self.step],receipts,self.history,self.entities)
        self.evidence['node_resolutions'].extend(receipts)
        effective=json.dumps(self.steps,indent=2).encode()
        (self.folder/'effective-course.json').write_bytes(effective)
        self.evidence['effective_course_sha256']=hashlib.sha256(effective).hexdigest()
        return goal

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
        try: self.go_without_scene(goal,navigate)
        except SceneCompleted as finished:
            entry=finished.entry
            assert entry['step']==self.step and self.step in CONTACT_SCENES.get(entry['scene'],()), 'Unexpected route handoff'
            if self.evidence['parking'] and 'tick' not in self.evidence['parking'][-1]:
                entry['parking_approach']=self.evidence['parking'].pop()
            verify_contact_handoff(entry,self.steps,self.evidence['arrivals'],self.history)
            # The partner's scripted reunion is recorded as a scene handoff,
            # never as an independently walked waypoint or completed parking.

    def cross_first_portal(self):
        validate_first_portal_course(self.steps)
        goal=self.steps[136]['nav'];first=self.first;other='guest' if first=='host' else 'host'
        self.set(other,attack=True)
        for role in (first,other):
            self.move(role,goal,navigate=True,yield_partner=other if role==first else first)
            report=self.reports[role]
            self.evidence['arrivals'].append(dict(step=136,role=role,goal=goal,tick=report['frame']['tick'],
                feet=list(self.body(role)['player']['feet']),command=report['route_driver']['command']))
            old_warp=self.body(role)['warped_at'];feet=self.body(role)['player']['feet']
            jump=dict(role=role,before_tick=report['frame']['tick'])
            self.set(role,[feet[0]+512,goal[1],feet[2]],run=True,jump=True,attack=False)
            jump['command']=self.commands[role]['id']
            self.until(lambda:self.body(role)['warped_at']>old_warp,15)
            jump['warp']=self.body(role)['warped_at'];self.set(role,attack=False)
            slot=0 if role=='host' else 1
            self.until(lambda:self.body(role)['player']['grounded']
                and math.hypot(*self.body(role)['player']['velocity'][:2])<20
                and all(r['frame']['players'][slot]['warped_at']==jump['warp'] for r in self.reports.values()),15)
            jump['arrival_tick']=self.reports[role]['frame']['tick'];self.evidence['portal_jumps'].append(jump)
            if role==first:
                report=self.reports[role];at=list(self.body(role)['player']['feet'])
                candidates=report['route_driver']['neighbors'][slot]
                # Walk away from the actual arrival before the other player
                # enters. The next source approach is deliberately sequential.
                destination=[float(v) for v in self.entities[131]['origin'].split()]
                park,ready=parking_target(candidates,at,destination,destination,[])
                assert ready, 'Portal arrival has no observed clear waiting point'
                receipt=dict(role=role,goal=park,selected_tick=report['frame']['tick'],candidates=candidates)
                self.move(role,park)
                receipt.update(tick=self.reports[role]['frame']['tick'],command=self.reports[role]['route_driver']['command'],
                    feet=list(self.body(role)['player']['feet']))
                self.evidence['portal_waiting'].append(receipt);self.set(role,park,run=True)
        verify_first_portal(self.evidence,self.steps,self.history,self.entities)

    def cross_clock_portal(self):
        assert self.steps[144]=={'portal':[-768,1962,184]}, 'Review changed clock portal course'
        takeoff=self.steps[142]['nav'];goal=self.steps[144]['portal']
        first=min(self.commands,key=lambda r:distance(self.body(r)['player']['feet'],takeoff))
        other='guest' if first=='host' else 'host'
        self.set(other,attack=True)
        for role in (first,other):
            self.move(role,takeoff,navigate=True,yield_partner=other if role==first else first)
            report=self.reports[role];slot=0 if role=='host' else 1
            old_warp=self.body(role)['warped_at']
            jump=dict(role=role,before_tick=report['frame']['tick'],commands=[])
            self.set(role,goal,run=True,jump=True,attack=False)
            jump['command']=self.commands[role]['id'];jump['commands'].append(jump['command'])
            last_jump=report['frame']['tick']
            def entered():
                nonlocal last_jump
                if self.body(role)['warped_at']>old_warp:return True
                report=self.reports[role];tick=report['frame']['tick'];body=self.body(role)['player']
                assert all(b['stats']['sanity']>0 for b in report['frame']['players']), 'Player died during clock portal approach'
                assert tick-jump['before_tick']<=1800, 'Clock portal input exceeded the original approach bound'
                hang=body.get('ledge')
                if report['route_driver']['command']==self.commands[role]['id'] and tick-last_jump>=40 \
                        and (body['grounded'] or (hang and not hang['pulling'])):
                    self.set(role,goal,run=True,jump=True,attack=False)
                    jump['commands'].append(self.commands[role]['id']);last_jump=tick
                return False
            self.until(entered,45)
            jump['warp']=self.body(role)['warped_at'];self.set(role,attack=False)
            self.until(lambda:self.body(role)['player']['grounded']
                and math.hypot(*self.body(role)['player']['velocity'][:2])<20
                and all(r['frame']['players'][slot]['warped_at']==jump['warp'] for r in self.reports.values()),15)
            jump['arrival_tick']=self.reports[role]['frame']['tick'];self.evidence['clock_portal_jumps'].append(jump)
            if role==first:
                report=self.reports[role];at=list(self.body(role)['player']['feet'])
                candidates=report['route_driver']['neighbors'][slot]
                destination=[float(v) for v in self.entities[124]['origin'].split()]
                source='neighbor'
                if distance(at[:2],destination[:2])>=48 and self.body(role)['player']['immersion']['level']==0:
                    park,ready,source=at,True,'arrival'
                elif any(distance(p[:2],destination[:2])>=48 and distance(p[:2],at[:2])>=16 for p in candidates):
                    park,ready=parking_target(candidates,at,destination,destination,[],prefer_forward=False)
                else:
                    # There may be no one-step waiting point on the small
                    # arrival platform. Walk the original onward route before
                    # inviting the second player through the portal.
                    park,ready,source=self.steps[145]['nav'],True,'onward'
                assert ready, 'Clock portal arrival has no observed clear waiting point'
                receipt=dict(role=role,goal=park,source=source,selected_tick=report['frame']['tick'],candidates=candidates)
                self.move(role,park,navigate=source=='onward')
                receipt.update(tick=self.reports[role]['frame']['tick'],command=self.reports[role]['route_driver']['command'],
                    feet=list(self.body(role)['player']['feet']))
                self.evidence['clock_portal_waiting'].append(receipt);self.set(role,park,run=True)
        verify_clock_portal(self.evidence,self.steps,self.history,self.entities)

    def go_without_scene(self, goal, navigate=False):
        first = self.first if self.step in (60,112) else min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], goal) + (0 if r == self.first else 16))
        if 81<=self.step<=108:
            # Either nearby player can lead this static return route. Avoid
            # repeatedly sending an injured Alice into the next room alone
            # just because her preceding waiting point is slightly closer.
            nearby=[r for r in self.commands if distance(self.body(r)['player']['feet'],goal)
                    <=distance(self.body(first)['player']['feet'],goal)+128]
            healthier=max(nearby,key=lambda r:self.body(r)['stats']['sanity'])
            if self.body(healthier)['stats']['sanity']>=self.body(first)['stats']['sanity']+20:
                first=healthier
        other = 'guest' if first == 'host' else 'host'
        self.set(other, attack=True)
        if self.step == 17:
            self.set(other,list(self.body(other)['player']['feet']),run=True)
        for role in (first, other):
            if 138<=self.step<=141:
                self.traverse_chair(role,goal,navigate,first if role==other else None)
                continue
            if self.step == 1: self.descend(role)
            if self.step == 16: self.board_gear(role,goal)
            if self.step == 17: self.exit_gear(role,goal)
            self.move(role, goal, navigate, yield_partner=first if role == other else other if self.step in (60,112) or 81<=self.step<=108 else None)
            report = self.reports[role]
            self.evidence['arrivals'].append(dict(step=self.step, role=role, goal=goal,
                tick=report['frame']['tick'], feet=list(self.body(role)['player']['feet']),
                command=report['route_driver']['command']))
            if role == first:
                slot = 0 if role == 'host' else 1
                if self.step in DOOR_HANDOFFS:
                    # Give the opening its authored wait, then walk through to
                    # make room for the follower. The ordinary wait step still
                    # runs for the pair afterwards; neither arrival is skipped.
                    park=next_door_footing(self.steps,self.step)
                    hold=dict(start_tick=report['frame']['tick'])
                    self.set(role,goal,run=True)
                    slots=doorway_slots(self.entities,(4,5))
                    self.until(lambda:self.reports[role]['frame']['tick']>=hold['start_tick']+204
                        and all(self.state(role)['doors'][k]>=.999 for k in slots),15)
                    hold['end_tick']=self.reports[role]['frame']['tick']
                    self.forward_wait(role,park,False,'door_waiting')
                    self.evidence['door_waiting'][-1]['opening_hold']=hold
                    continue
                if self.step in CORRIDOR_HANDOFFS:
                    self.forward_wait(role,next_corridor_footing(self.steps,self.step),True,'corridor_waiting')
                    continue
                if self.step in STAIR_HANDOFFS or not report['route_driver']['neighbors'][slot]:
                    # An eight-unit arrival can stop on the edge of a small
                    # footing. Center by walking before asking for neighbors.
                    self.move(role, goal, radius=2)
                if self.step in STAIR_HANDOFFS:
                    self.forward_wait(role,next_stair_footing(self.steps,self.step),navigate,'stair_waiting')
                    continue
                visited = []
                for attempt in range(8):
                    report = self.reports[role]
                    candidates = report['route_driver']['neighbors'][slot]
                    at = list(self.body(role)['player']['feet'])
                    park, ready = parking_target(candidates, self.body(role)['player']['feet'],
                                                 self.body(other)['player']['feet'], goal, visited,
                                                 prefer_forward=self.step!=142)
                    visited.append(at)
                    self.evidence['parking'].append(dict(step=self.step, role=role, goal=park, ready=ready,
                        selected_tick=report['frame']['tick'], candidates=candidates))
                    # Stopping eight units short on the entrance's small
                    # waiting footing can obstruct the partner's pull-up.
                    self.move(role, park, radius=2 if self.step == 1 else 8)
                    self.evidence['parking'][-1].update(tick=self.reports[role]['frame']['tick'],
                        command=self.reports[role]['route_driver']['command'], feet=list(self.body(role)['player']['feet']))
                    if ready: break
                else: raise AssertionError('Could not walk far enough to leave the arrival lane clear')
                self.wait_for_partner(role,park)
        if 138<=self.step<=141:
            verify_chair_hops(self.evidence['ledge_pulls'],self.steps,self.evidence['arrivals'],self.history)
            verify_chair_departures(self.evidence['chair_departures'],self.steps,self.evidence['arrivals'],self.history,self.completed+[self.step])
        self.set(other, attack=True)

    def traverse_chair(self, role, goal, navigate, yield_partner):
        for attempt in range(3):
            start=self.reports[role]['frame']['tick'];spawn=self.body(role)['spawned_at']
            lengths={k:len(self.evidence[k]) for k in ('arrivals','ledge_pulls','chair_departures','yields')}
            self.moving_role=role
            try:
                self.move(role,goal,navigate,yield_partner=yield_partner)
                report=self.reports[role]
                self.evidence['arrivals'].append(dict(step=self.step,role=role,goal=goal,
                    tick=report['frame']['tick'],feet=list(self.body(role)['player']['feet']),
                    command=report['route_driver']['command']))
                self.leave_chair(role,goal,start)
                return
            except ChairDeath:
                assert self.allow_revival, 'Strict chair traversal cannot revive'
                failed=dict(step=self.step,role=role,start_tick=start,previous_spawn=spawn,
                    death_seen_tick=self.reports[role]['frame']['tick'],receipts={})
                for key,count in lengths.items():
                    failed['receipts'][key]=self.evidence[key][count:]
                    del self.evidence[key][count:]
                self.evidence['failed_chair_attempts'].append(failed)
                assert attempt<2, 'Chair failed after three actual attempts'
                self.moving_role=None
                other='guest' if role=='host' else 'host'
                self.set(role,attack=False);self.set(other,attack=True)
                self.until(lambda:all(all(b['stats']['sanity']>0 for b in r['frame']['players'])
                    for r in self.reports.values()) and self.body(role)['spawned_at']>spawn,45)
                failed['spawn_tick']=self.body(role)['spawned_at']
                print('Observed native revival; retrying personal chair',self.step,role,flush=True)
            finally:self.moving_role=None

    def lever(self, index):
        self.clear()
        target = [float(v) for v in self.entities[LEVERS[index]]['origin'].split()]
        target[2] += 20
        role = min(self.commands, key=lambda r: distance(self.body(r)['player']['feet'], target))
        if index==3 and self.args.port_player:
            role='host' if self.args.port_player=='blue' else 'guest'
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
        if index==4:
            self.watch_scene('Hatter1_End',role,command)
        self.set(role, attack=True)

    def leave_level(self):
        goal=self.steps[148]['nav']
        role=min(self.commands,key=lambda r:distance(self.body(r)['player']['feet'],goal))
        other='guest' if role=='host' else 'host'
        self.set(other,attack=True)
        self.move(role,goal,navigate=True,yield_partner=other)
        entry=dict(role=role,approach_tick=self.reports[role]['frame']['tick'],
                   approach_command=self.reports[role]['route_driver']['command'])
        self.step=149
        self.set(role,self.steps[149]['walk'],run=True,attack=False)
        entry['exit_command']=self.commands[role]['id']
        self.until(self.arrived,60)
        entry['arrival_ticks']={peer:next(r['tick'] for r in rows if r['map']==self.next_map and r['epoch']==2)
                                for peer,rows in self.history.items()}
        start=self.reports['host']['frame']['tick']
        for peer in self.commands:self.set(peer,attack=False,epoch=2)
        self.until(lambda:all(r['frame']['tick']>=start+180 for r in self.reports.values()),15)
        entry['matching_arrival_states']=verify_clockwork_exit(entry,self.steps,self.history)
        self.evidence['exit_handoff']=entry

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
        revivals=[]
        if getattr(self,'allow_revival',False):
            recovery=read_trace(self.folder/'host/recovery-trace.jsonl',self.build,self.args.require_revival)
            recovery['rendered_revivals']=verify_views(recovery['revivals'],self.history)
            verify_chair_retries(self.evidence['failed_chair_attempts'],recovery['revivals'],
                self.evidence['arrivals'],self.evidence['chair_departures'],self.history)
            self.evidence['recovery']=recovery
            revivals=list(recovery['revivals'])
            if self.saved:
                revivals += [dict(player=i,epoch=self.saved['frame']['epoch'],spawn_tick=b['spawned_at'])
                    for i,b in enumerate(self.saved['frame']['players'])]
        exited=self.evidence.get('exit_handoff')
        regular=[i for i in self.completed if not exited or i<148]
        if exited:
            assert self.completed[-2:]==[148,149], 'Incomplete requested exit sequence'
            verify_clockwork_exit(exited,self.steps,self.history)
        verify_arrivals(self.steps, regular, self.evidence['arrivals'], self.history,self.evidence['scene_handoffs'],revivals)
        verify_node_resolutions(self.source_steps,self.steps,self.completed,self.evidence['node_resolutions'],self.history,self.entities)
        for entry in self.evidence['scene_handoffs']:
            assert entry['step'] in self.completed, 'Scene handoff outside completed course'
            verify_scene_handoff(entry,self.history,self.evidence['levers'])
        names=[entry['scene'] for entry in self.evidence['scene_handoffs']]
        assert len(names)==len(set(names)), 'Duplicate scene handoff'
        if 62 in self.completed and self.args.start_step<=60:
            assert names.count('Hatter1_Cinema1')==1, 'Missing Hare scene handoff'
        if 67 in self.completed: assert names.count('Port_Cine')==1, 'Missing Port scene handoff'
        if 112 in self.completed: assert names.count('Hatter1_Cinema2')==1, 'Missing Gryphon scene handoff'
        if 146 in self.completed: assert names.count('Hatter1_End')==1, 'Missing Stop scene handoff'
        if 1 in self.completed:
            verify_descent(self.evidence['descent_arrivals'], self.evidence['descent_jumps'], self.history)
        verify_ledge_pulls(self.evidence['ledge_pulls'], self.history)
        verify_chair_hops(self.evidence['ledge_pulls'],self.steps,self.evidence['arrivals'],self.history)
        verify_chair_departures(self.evidence['chair_departures'],self.steps,self.evidence['arrivals'],self.history,self.completed)
        verify_yields(self.evidence['yields'], self.steps, self.history)
        verify_stair_waiting(self.evidence['stair_waiting'],self.steps,self.evidence['arrivals'],self.history)
        verify_corridor_waiting(self.evidence['corridor_waiting'],self.steps,self.evidence['arrivals'],self.history)
        verify_door_waiting(self.evidence['door_waiting'],self.steps,self.evidence['arrivals'],self.history,self.entities)
        verify_doorway_approaches(self.evidence['doorway_approaches'],self.steps,self.history,self.entities)
        if 16 in self.completed: verify_gear_boarding(self.evidence['gear_boarding'],self.steps[16]['nav'],self.history)
        if 17 in self.completed: verify_gear_exit(self.evidence['gear_exits'],self.steps[17]['nav'],self.history)
        if 36 in self.completed: verify_gap_sequence(self.evidence,self.steps,self.history)
        if 136 in self.completed: verify_first_portal(self.evidence,self.steps,self.history,self.entities)
        portal_warps=verify_clock_portal(self.evidence,self.steps,self.history,self.entities) if 144 in self.completed else set()
        expected = [i for i in self.completed if 'lever' in self.steps[i]]
        assert [r['step'] for r in self.evidence['levers']] == expected, 'Missing lever receipts'
        for receipt in self.evidence['levers']:
            if receipt['lever']==3 and self.args.port_player:
                assert receipt['role']==('host' if self.args.port_player=='blue' else 'guest'), 'Wrong requested Port lever user'
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
        interrupted={r['step'] for r in self.evidence['scene_handoffs'] if 'interrupted_goal' in r}
        assert sorted([p['step'] for p in self.evidence['parking'] if p['ready']]
            +[p['step'] for kind in ('stair_waiting','corridor_waiting','door_waiting') for p in self.evidence[kind]]) == [i for i in regular
            if next(iter(self.steps[i])) in ('nav','walk') and i not in (36,37,42,136,138,139,140,141) and i not in interrupted], 'Missing completed parking handoff'
        for role, rows in self.history.items():
            allowed={(self.map_name,1)}|({(self.next_map,2)} if exited else set())
            assert rows and all((r['map'],r['epoch']) in allowed and r['checkpoint_generation']==0 for r in rows)
            if getattr(self,'allow_revival',False):
                verify_recovery_presence(rows,role,portal_warps=portal_warps)
                assert all(p['health']>0 for p in rows[-1]['players']), 'Recovery route ends with a dead player'
            else:assert all(p['health'] > 0 and p['spawned_at'] == 0 for r in rows for p in r['players']), 'Prefix contains a death'
        for field, metric in (('sha256','matching_sent_and_restored_states'), ('world_sha256','matching_world_states'), ('native_sha256','matching_native_states')):
            self.verify_network_states(field, metric)
        self.evidence.update(build=self.build, passed=True,
            mode='verified-individual-revival' if getattr(self,'allow_revival',False) else 'strict-no-death')

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
            if self.step in (0, 1, 9, 14, 17, 18, 25, 33, 36, 43, 44, 51, 52, 60, 64, 66, 68, 76, 79, 81, 91, 99, 107, 109, 112, 114, 128, 131, 136, 139, 140, 141, 142, 143): self.save_checkpoint(self.step)
            if self.step==36:
                self.cross_corridor_gap()
                self.completed.extend(range(36,43))
                print('PASS paired gap sequence 36-42',flush=True)
                continue
            if self.step==136:
                self.cross_first_portal()
                self.completed.extend((136,137))
                print('PASS paired portal sequence 136-137',flush=True)
                continue
            if self.step==144:
                self.cross_clock_portal()
                self.completed.append(144)
                print('PASS paired clock portal 144',flush=True)
                continue
            if self.step==148 and self.args.last_step==149:
                self.leave_level()
                self.completed.extend((148,149))
                print('PASS original exit approach and shared About Face arrival',flush=True)
                break
            key, value = next(iter(step.items()))
            if key in ('nav', 'walk'): self.go(value, key == 'nav')
            elif key=='node': self.go(self.resolve_node(value),True)
            elif key == 'lever': self.lever(value)
            elif key == 'wait':
                start = self.reports['host']['frame']['tick']
                if self.step==67: self.wait_port(value)
                else: self.wait(value)
                self.evidence['holds'].append(dict(step=self.step, start_tick=start, end_tick=self.reports['host']['frame']['tick'], seconds=value))
            else: raise AssertionError('Unimplemented course action')
            self.completed.append(self.step)
            print('PASS Clockwork prefix step', self.step, flush=True)
        if not self.evidence.get('exit_handoff'):self.save_checkpoint(self.args.last_step + 1)
        self.verify()
        print('PASS', 'resumed' if self.saved else 'fresh', 'Clockwork prefix', self.completed[0], self.completed[-1], self.evidence['matching_sent_and_restored_states'], 'matching states', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue','red'), default='blue')
    parser.add_argument('--port-player', choices=('blue','red'), help='Choose the Port lever user through ordinary input')
    parser.add_argument('--last-step', type=int, default=17)
    parser.add_argument('--start-step', type=int, default=0)
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--allow-revival', action='store_true', help='Verify individual recovery and retry interrupted chair attempts')
    parser.add_argument('--require-revival', action='store_true')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not 0 <= args.start_step <= args.last_step < 150: parser.error('Require 0 <= start-step <= last-step < 150')
    if (args.start_step == 0) != (args.checkpoint is None): parser.error('A nonzero start-step requires a checkpoint; fresh runs start at zero')
    if not all(0 <= v <= 500 for v in (args.delay_ms, args.jitter_ms)): parser.error('Delay/jitter must be 0..500 ms')
    if args.require_revival and not args.allow_revival:parser.error('--require-revival requires --allow-revival')
    if args.allow_revival and not 138<=args.start_step<=args.last_step<=149:parser.error('Recovery continuation currently covers final segments 138..149')
    if args.last_step==149 and args.start_step>148:parser.error('The exit requires its ordinary approach at step 148')
    pair = ClockworkPair(args)
    try: pair.run()
    except BaseException as error:
        pair.evidence['process_exit_codes_before_cleanup'] = {role: process.poll() for role, process in pair.processes.items()}
        pair.evidence.update(passed=False, failed_step=pair.step, error=str(error))
        raise
    finally: pair.close()


if __name__ == '__main__': main()
