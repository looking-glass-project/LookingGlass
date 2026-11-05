"""Play About Face in two independent clients using ordinary network inputs.

Fresh entry, the original Hatter fight, personal rewards, paired lift and Gryphon
exit are separate recorded milestones. A stopped/resumed run never claims a full
visit. No positions, resources or encounter progress are written by this runner.
"""
import argparse
import hashlib
import json
import re
import struct
import time
import zipfile
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from coop_recovery import read_trace, verify_presence, verify_trace, verify_views


ROLES = ('host', 'guest')
SCENE = 'Hatter2_Gryphon_Cinema1'
CORNERS = ([6080, -240, 0], [6080, 600, 0], [5260, 600, 0], [5260, -240, 0])
DOOR_OPEN = struct.unpack('<f', struct.pack('<f', 1.4))[0]


def door_open(state):
    # JSON preserves the exact native f32 cap, which is below Python's 1.4.
    return state['phase'] == 'ExitReady' and state['door'] >= DOOR_OPEN


def completed_trace_prefix(path, through=None):
    lines = path.read_bytes().splitlines(keepends=True)
    if lines and not lines[-1].endswith(b'\n'): lines.pop()
    rows = [json.loads(line) for line in lines]
    return rows if through is None else [r for r in rows if (r['epoch'], r['tick']) <= through]


def verify_victory_resume(path, saved, build):
    """A resumed nonzero spawn stamp needs the original exact recovery proof."""
    folder = path.parent
    observations = json.loads((folder / 'observations.json').read_text())
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    assert any(c['name'] == 'victory' and c['sha256'] == digest and c['tick'] == saved['frame']['tick']
        for c in observations['checkpoints']), 'Checkpoint is not the recorded ordinary victory save'
    boundary = saved['visit']['epoch'], saved['frame']['tick']
    trace = completed_trace_prefix(folder / 'host/recovery-trace.jsonl', boundary)
    assert trace and boundary[1] - 120 <= trace[-1]['tick'] <= boundary[1], 'Missing checkpoint trace heartbeat'
    recovery = verify_trace(trace, build)
    history = {role:[r for r in rows if (r['epoch'], r['tick']) <= boundary]
        for role, rows in json.loads((folder / 'history.json').read_text()).items()}
    combat = verify_combat(history, recovery['revivals'])
    assert combat == observations['combat'], 'Original fight proof changed'
    verify_views(recovery['revivals'], history)
    for slot, body in enumerate(saved['frame']['players']):
        assert body['stats']['sanity'] > 0
        assert body['spawned_at'] == 0 or any(e['player'] == slot and e['epoch'] == boundary[0]
            and e['spawn_tick'] == body['spawned_at'] for e in recovery['revivals']), 'Unverified checkpoint revival'
    return recovery


def map_markers(base):
    raw = None
    for path in sorted(base.glob('*.pk3'), key=lambda p: p.name.lower()):
        with zipfile.ZipFile(path) as pack:
            for item in pack.infolist():
                if item.filename.replace('\\', '/').lower() == 'maps/hatter2.bsp':
                    assert item.file_size <= 128 * 1024 * 1024
                    raw = pack.read(item)
    assert raw and raw[:8] == b'FAKK*\0\0\0', 'Expected native About Face map'
    offset, length = struct.unpack_from('<II', raw, 12 + 14 * 8)
    assert offset >= 172 and offset + length <= len(raw)
    entities = [dict(re.findall(r'"([^"\n]*)"\s*"([^"\n]*)"', body))
                for body in re.findall(r'\{([^{}]*)\}', raw[offset:offset + length].decode())]
    xyz = lambda e: [float(v) for v in e['origin'].split()]
    return dict(sha256=hashlib.sha256(raw).hexdigest(),
        healing=[xyz(e) for e in entities if e.get('targetname', '').startswith('help_me')],
        blade=next(xyz(e) for e in entities if e.get('classname') == 'ambient_eyestaff_blade'),
        watch=next(dict(key=f'hatter2:{i}', at=xyz(e)) for i, e in enumerate(entities)
                   if e.get('classname') == 'Item_WeaponPickup_DeadtimeWatch'))


def verify_arrival(receipt, history, revivals=()):
    role = receipt['role']; slot = ROLES.index(role)
    row = next(r for r in history[role] if (r['epoch'], r['tick']) == (receipt['epoch'], receipt['tick']))
    body = row['players'][slot]
    assert row['map'] == 'hatter2' and row['owner'] is None, 'Arrival outside ordinary About Face play'
    assert row['driver']['command'] == receipt['command'] and row['driver']['reached'], 'Unacknowledged arrival'
    assert body['health'] > 0 and body['grounded'] and row['driver']['clear'][slot], 'Invalid arrival body'
    assert body['spawned_at'] == 0 or any(e['player'] == slot and e['epoch'] == row['epoch']
        and e['spawn_tick'] == body['spawned_at'] <= row['tick'] for e in revivals), 'Arrival silently respawned'
    assert body['feet'] == receipt['feet'], 'Arrival body changed'
    assert distance(body['feet'][:2], receipt['goal'][:2]) < 8 and abs(body['feet'][2] - receipt['goal'][2]) < 35, 'Missed waypoint'


def verify_combat(history, revivals=()):
    assert set(history) == set(ROLES), 'Missing independent combat client'
    evidence = []
    for role, rows in history.items():
        # A later visit carries the previous revival stamp in a new epoch.
        # Combat proof belongs to this visit; ending/trace checks cover onward play.
        rows = [r for r in rows if r['map'] == 'hatter2' and r['epoch'] == 1]
        fight = [r for r in rows if r['map'] == 'hatter2' and r['hatter']['phase'] == 'Fight']
        won = [r for r in rows if r['map'] == 'hatter2' and r['hatter']['phase'] == 'Victory']
        assert fight and fight[0]['hatter']['boss']['health'] > 0 and won, 'Missing actual boss fight and defeat'
        assert won[0]['hatter']['boss']['health'] <= 0 and won[0]['tick'] > fight[0]['tick'], 'Invalid boss defeat'
        for r in rows:
            for slot, p in enumerate(r['players']):
                if p['health'] <= 0:
                    assert any(e['player'] == slot and e['epoch'] == r['epoch'] and
                        e['death_tick'] <= r['tick'] < e['spawn_tick'] for e in revivals), 'Unverified combat death'
                if p['spawned_at'] != 0:
                    assert any(e['player'] == slot and e['epoch'] == r['epoch'] and
                        e['spawn_tick'] == p['spawned_at'] <= r['tick'] for e in revivals), 'Unverified combat revival'
        for slot in range(2):
            assert any(r['players'][slot]['feet'] != fight[0]['players'][slot]['feet'] for r in fight), 'Stationary combat participant'
        evidence.append(dict(peer=role, start=fight[0]['tick'], defeat=won[0]['tick'],
            health=[p['health'] for p in won[0]['players']]))
    return evidence


def verify_ending(history, first):
    assert set(history) == set(ROLES), 'Missing independent ending client'
    evidence = []
    for role, rows in history.items():
        scene = [r for r in rows if r['owner'] is not None and r['map'] == 'hatter2']
        active = [r for r in scene if r['hatter']['phase'] == 'Gryphon']
        assert active and all(r['owner'] == first.title() for r in scene), 'Missing or changing Gryphon actor'
        assert all(r['scene'] == SCENE for r in active), 'Wrong original scene'
        assert {r['hatter']['line'] for r in active} == set(range(5)), 'Missing watched Gryphon lines'
        for r in scene:
            assert r['hatter']['phase'] == 'Gryphon' or valid_ending_tail(r, active[-1]['tick']), 'Unverified ending tail'
        partner = 1 if first == 'blue' else 0
        for r in scene:
            assert not r['presentation']['visible_players'][partner], 'Scene exposed partner'
            assert r['players'][partner]['feet'] == scene[0]['players'][partner]['feet'], 'Scene moved partner'
            for old, new in zip(scene[0]['players'], r['players']):
                assert old['health'] == new['health'] > 0 and old['spawned_at'] == new['spawned_at'], 'Scene changed life'
                assert old['collected'] == new['collected'] and old['stats']['owned'] == new['stats']['owned'], 'Scene changed inventory'
        after_talk = next((r for r in active if r['hatter']['talk_done']), None)
        transition = next((r for r in rows if r['map'] == 'jlair1' and r['epoch'] == 2), None)
        arrival = next((r for r in rows if r['map'] == 'jlair1' and r['epoch'] == 2 and r['owner'] is None), None)
        assert after_talk and transition and arrival and transition['tick'] > active[-1]['tick'], 'No actual onward arrival'
        tail_ticks = verify_tail_duration(after_talk, transition)
        assert all(p['grounded'] and p['health'] > 0 for p in arrival['players']) and all(arrival['driver']['clear']), 'Unsafe arrival'
        assert distance(arrival['players'][0]['feet'], arrival['players'][1]['feet']) <= 160, 'Onward pair not reunited'
        for old, new in zip(scene[0]['players'], arrival['players']):
            assert old['collected'] == new['collected'] and old['stats']['owned'] == new['stats']['owned'], 'Exit lost inventory'
        verify_presence(rows, role)
        evidence.append(dict(peer=role, owner=first.title(), scene_tick=active[0]['tick'],
            transition_tick=transition['tick'], arrival_tick=arrival['tick'], tail_ticks=tail_ticks))
    return evidence


def valid_ending_tail(row, last_active):
    state = row['hatter']
    return (state['phase'] == 'Done' and row['scene'] is None and row['tick'] > last_active
        and state['exit']['committed'] and state['talk_done'] and state['after_talk'] >= 10.85)


def verify_tail_duration(after_talk, transition):
    # Measure to the first destination frame, not the end of its entry scene.
    # Otherwise that later dialogue could conceal a shortened Gryphon tail.
    start = after_talk['tick'] - round(after_talk['hatter']['after_talk'] * 120)
    ticks = transition['tick'] - start
    assert ticks >= 1302, 'Exit skipped its original physical tail'
    return ticks


class AboutFacePair(Pair):
    map_name, next_map = 'hatter2', 'jlair1'
    evidence_prefix, capture_scenes = 'hatter2-route-', False

    def __init__(self, args):
        self.phase = 'starting'
        self.revivals = []
        self.prior_revivals = []
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.lead, self.lead_tick = self.minimum_lead, None
        super().__init__(args)
        self.markers = map_markers(args.data)
        self.evidence.update(executable_sha256=hashlib.sha256(args.exe.read_bytes()).hexdigest(),
            markers=self.markers, arrivals=[], combat_inputs=[], milestones=[], fresh=args.checkpoint is None)
        deps = self.folder / 'fixture-dependencies'; deps.mkdir()
        for name in ('test_coop_hatter2.py', 'test_coop_village.py', 'test_coop_play.py',
                     'test_coop_chess_resume.py', 'test_coop_chess.py', 'coop_diagnostics.py', 'coop_recovery.py'):
            raw = (Path(__file__).parent / name).read_bytes()
            (deps / name).write_bytes(raw)
            self.evidence.setdefault('fixture_sha256', {})[name] = hashlib.sha256(raw).hexdigest()
        self.saved = None
        if args.checkpoint:
            raw = args.checkpoint.read_bytes()
            self.saved = json.loads(raw)['payload']
            assert self.saved['visit']['map'] == self.map_name and self.saved['visit']['epoch'] == 1
            assert self.saved['frame']['checkpoint_generation'] == 0
            self.resume_checkpoint = True
            (self.folder / 'starting-checkpoint.json').write_bytes(raw)
            (self.folder / 'host/coop-checkpoint.json').write_bytes(raw)
            self.evidence['checkpoint_sha256'] = hashlib.sha256(raw).hexdigest()
            recovery = verify_victory_resume(args.checkpoint, self.saved, args.build)
            self.prior_revivals = recovery['revivals']
            self.revivals = list(self.prior_revivals)
            self.evidence['prior_recovery'] = recovery

    def set(self, role, *args, **kwargs):
        kwargs.setdefault('attack', False)
        super().set(role, *args, **kwargs)
        self.commands[role].update(input_lead=self.lead if role == 'guest' else 0,
            watch_dialogue=True, combat_tactics=True, prefer_cards=True, prefer_ice=False,
            avoid_switches=True, recovery_trace=role == 'host')

    def body(self, role):
        return self.reports[role]['frame']['players'][ROLES.index(role)]

    def state(self, role):
        return self.reports[role]['combat']['stage']['world']['levels'].get(self.map_name)

    def validate_report(self, role, report):
        assert report['frame']['checkpoint_generation'] == 0, 'Team wipe requires separate checkpoint proof'
        if not (self.args.allow_revival and self.phase == 'Hatter fight'):
            assert all(b['stats']['sanity'] > 0 for b in report['frame']['players']), 'Death outside combat recovery scope'
        assert report['build'] == self.args.build, 'Executable differs from frozen build'
        assert (report['visit']['map'], report['visit']['epoch']) in ((self.map_name, 1), (self.next_map, 2))
        assert all(not b['stats']['god'] and not b['stats']['notarget']
                   for b in report['frame']['players']), 'Cheated traversal'
        assert self.args.allow_revival or all(b['spawned_at'] == 0 for b in report['frame']['players']), 'Unexpected revival'
        assert report['route_driver'].get('recovery_trace_error') is None
        if report['combat']['stage']['owner'] is None and all(p['stats']['sanity'] > 0 for p in report['frame']['players']):
            a, b = [p['player']['feet'] for p in report['frame']['players']]
            assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2] - b[2]) >= 55.95, 'Living players overlap'

    def validate_clear(self, role, report):
        assert all(clear for clear, body in zip(report['route_driver']['clear'], report['frame']['players'])
                   if body['stats']['sanity'] > 0), 'Obstructed living player'

    def pump(self):
        assert not (self.folder / 'stop-test').exists(), 'About Face test stopped'
        super().pump()
        for role, report in self.reports.items():
            row = self.history[role][-1]
            # Only enrich the same immutable frame retained by Pair.
            assert (row['epoch'], row['tick']) == (report['frame']['epoch'], report['frame']['tick'])
            row.setdefault('hatter', self.state(role))
            row.setdefault('presentation', report['presentation'])
            row.setdefault('visit_entry', report['visit']['entry'])
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
                details = {r: dict(tick=d['frame']['tick'], phase=(self.state(r) or {}).get('phase'),
                    feet=[p['player']['feet'] for p in d['frame']['players']],
                    driver={k:d['route_driver'].get(k) for k in ('command', 'phase', 'error', 'reached')})
                    for r, d in self.reports.items()}
                raise AssertionError(f'{self.phase} timed out: {details}')
            self.pump()

    def reached(self, role):
        if not super().reached(role) or not self.history[role]: return False
        row = self.history[role][-1]
        return row['driver']['command'] == self.commands[role]['id'] and row['driver']['reached']

    def move_pair(self, goals, *, navigate=False, jump=False, attack=False):
        for role, goal in zip(ROLES, goals):
            self.set(role, goal, navigate=navigate, jump=jump, attack=attack, run=True)
        self.until(lambda: all(self.reached(r) for r in ROLES))
        for role, goal in zip(ROLES, goals):
            row = self.history[role][-1]
            receipt = dict(role=role, goal=goal, feet=list(self.body(role)['player']['feet']),
                epoch=row['epoch'], tick=row['tick'], command=self.commands[role]['id'], phase=self.phase)
            verify_arrival(receipt, self.history, self.revivals)
            self.evidence['arrivals'].append(receipt)
            self.set(role)

    def mark(self, name):
        self.evidence['milestones'].append(dict(name=name, ticks={r:d['frame']['tick'] for r,d in self.reports.items()}))
        (self.folder / f'{name}.json').write_text(json.dumps(self.reports))
        print('PASS', name, 'health', [p['stats']['sanity'] for p in self.reports['host']['frame']['players']], flush=True)

    def checkpoint(self, name):
        for role in ROLES: self.set(role)
        self.commands['host']['checkpoint'] = True
        command = self.commands['host']['id']
        self.until(lambda: (self.reports['host']['route_driver'].get('checkpoint') or {}).get('command') == command, 30)
        receipt = self.reports['host']['route_driver']['checkpoint']
        assert receipt['epoch'] == 1 and receipt['error'] is None, receipt
        raw = (self.folder / 'host/coop-checkpoint.json').read_bytes()
        payload = json.loads(raw)['payload']
        assert payload['visit']['map'] == self.map_name and payload['frame']['tick'] == receipt['tick']
        (self.folder / f'checkpoint-{name}.json').write_bytes(raw)
        self.evidence.setdefault('checkpoints', []).append(dict(name=name, sha256=hashlib.sha256(raw).hexdigest(), tick=payload['frame']['tick']))
        self.set('host')

    def approach(self):
        self.phase = 'bridge approach'
        self.move_pair([[5640, -400, 0], [5688, -400, 0]], navigate=True)
        self.phase = 'arena approach'
        self.move_pair([[5640, 160, 0], [5688, 160, 0]], navigate=True)
        self.until(lambda: all(self.state(r)['phase'] == 'Fight' for r in ROLES), 120)
        self.mark('fresh-arena-entry')

    def fight(self):
        self.phase = 'Hatter fight'
        cursors = dict(host=0, guest=2)
        healing = dict.fromkeys(ROLES, False)
        spawned = {r:self.body(r)['spawned_at'] for r in ROLES}
        started = time.monotonic(); printed = -1
        while not all(self.state(r)['phase'] == 'Victory' and all(p['stats']['sanity'] > 0
                      for p in self.reports[r]['frame']['players']) for r in ROLES):
            assert time.monotonic() - started < 700, 'Hatter fight exceeded ten minutes of gameplay'
            for role in ROLES:
                state = self.state(role)
                body = self.body(role)
                if body['stats']['sanity'] <= 0:
                    if self.commands[role]['attack'] or self.commands[role]['goal'] is not None: self.set(role)
                    continue
                if state['phase'] != 'Fight':
                    if self.commands[role]['attack']: self.set(role)
                    continue
                if body['spawned_at'] != spawned[role]:
                    spawned[role] = body['spawned_at']
                    cursors[role] = min(range(4), key=lambda i: distance(body['player']['feet'], CORNERS[i]))
                    self.set(role)
                slot = ROLES.index(role)
                items = state['coop']['items']
                wants_heal = body['stats']['sanity'] < 75 and items['ready'][slot]
                heal = self.markers['healing'][items['index'][slot] if items['easy'][slot] else -1]
                # Both can consume their own copy from separated points inside
                # its native pickup box; they never converge on the same spot.
                heal_goal = [heal[0] + (-24 if slot == 0 else 24), heal[1], 0]
                if wants_heal != healing[role] or self.commands[role]['goal'] is None or self.reached(role):
                    if not wants_heal and self.reached(role): cursors[role] = (cursors[role] + 1) % 4
                    healing[role] = wants_heal
                    goal = heal_goal if wants_heal else CORNERS[cursors[role]]
                    self.set(role, goal, attack=True, run=True)
                    self.evidence['combat_inputs'].append(dict(role=role, tick=self.reports[role]['frame']['tick'], command=dict(self.commands[role])))
            self.pump()
            interval = int((time.monotonic() - started) / 20)
            if interval != printed:
                printed = interval
                print('COMBAT', self.state('host')['cycle'], self.state('host')['boss']['health'],
                    [p['stats']['sanity'] for p in self.reports['host']['frame']['players']], flush=True)
        for role in ROLES: self.set(role)
        # Let the exact host trace publish its next heartbeat after the last
        # revival. Only a currently incomplete final JSON line may be omitted;
        # the finished run is checked again against the closed, complete file.
        tick = self.reports['host']['frame']['tick']
        self.until(lambda: self.reports['host']['frame']['tick'] >= tick + 240, 10)
        recovery = verify_trace(completed_trace_prefix(self.folder / 'host/recovery-trace.jsonl'), self.build)
        self.revivals = recovery['revivals']
        verify_views(self.revivals, self.history)
        self.evidence['combat'] = verify_combat(self.history, self.revivals)
        self.evidence['combat_recovery'] = recovery
        self.mark('hatter-defeated')
        self.checkpoint('victory')

    def rewards_and_lift(self):
        self.phase = 'personal blade'
        blade = self.markers['blade']
        self.move_pair([[blade[0]-24, blade[1], 0], [blade[0]+24, blade[1], 0]], navigate=True)
        self.until(lambda: all('hatter2:11' in self.body(r)['stats']['collected'] for r in ROLES))
        self.mark('personal-blades')
        self.phase = 'lift approach'
        self.move_pair([[5544, 828, 0], [5592, 828, 0]], navigate=True)
        self.phase = 'paired lift'
        for role, x in zip(ROLES, (5544, 5592)):
            self.set(role, [x, 988, 8], jump=True, horizontal=True)
        self.until(lambda: all(self.body(r)['player']['grounded'] and self.body(r)['player']['feet'][2] >= 398 for r in ROLES), 30)
        self.mark('paired-upper-deck')
        self.phase = 'Watch landing'
        self.move_pair([[5640, 1152, 392], [5688, 1152, 392]], jump=True)
        self.until(lambda: all(self.markers['watch']['key'] in self.body(r)['stats']['collected'] for r in ROLES), 20)
        self.mark('personal-watches')
        self.until(lambda: all(door_open(self.state(r)) for r in ROLES), 120)
        self.checkpoint('exit-ready')

    def ending(self):
        self.phase = 'Gryphon approach'
        first = 'host' if self.args.first == 'blue' else 'guest'
        self.set(first, [5664, 1234, 392], navigate=True, run=True)
        active = lambda: any(d['combat']['stage']['owner'] is not None for d in self.reports.values())
        self.until(lambda: active() or self.reached(first))
        if not active():
            row = self.history[first][-1]
            approach = dict(role=first, goal=[5664, 1234, 392], feet=list(self.body(first)['player']['feet']),
                epoch=row['epoch'], tick=row['tick'], command=self.commands[first]['id'])
            verify_arrival(approach, self.history, self.revivals)
            self.evidence['gryphon_approach'] = approach
            # Match the original SP route's ordinary forward continuation after
            # its planned approach. Reaching the nearby marker is not a portal.
            self.set(first, [5664, 1300, 392], run=True)
            self.evidence['gryphon_forward_command'] = dict(self.commands[first])
            self.until(active, 15)
        for role in ROLES: self.set(role)
        self.phase = 'Gryphon scene'
        self.until(self.arrived, 180)
        for role in ROLES: self.set(role, epoch=2)
        self.until(lambda: all(d['combat']['stage']['owner'] is None and all(p['player']['grounded']
            for p in d['frame']['players']) for d in self.reports.values()), 30)
        self.evidence['ending'] = verify_ending(self.history, self.args.first)
        self.mark('jlair1-arrival')

    def run(self):
        self.start_pair()
        if self.saved is None:
            self.approach()
            self.fight()
        else:
            # No entry/fight credit is granted by an unchanged victory resume.
            receipt = json.loads((self.folder / 'host/route-resume.json').read_text())
            verify_resume_receipt(receipt, self.saved, self.build)
            self.evidence['exact_resume_receipt'] = receipt
            for report in self.reports.values():
                elapsed = report['frame']['tick'] - self.saved['frame']['tick']
                for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                    verify_resume_body(body, old, elapsed)
            assert all(self.state(r)['phase'] == 'Victory' for r in ROLES), 'Resume requires normal victory checkpoint'
        if self.args.stop_after == 'full':
            self.rewards_and_lift()
            self.ending()
        for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'), ('native_sha256', 'matching_native_states')]:
            self.verify_network_states(field, metric)
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=2 if self.args.stop_after == 'full' else 1)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        recovery = read_trace(self.folder / 'host/recovery-trace.jsonl', self.build)
        assert self.prior_revivals + recovery['revivals'] == self.revivals, 'Recovery after the verified fight'
        recovery['rendered_revivals'] = verify_views(recovery['revivals'], self.history)
        for role, rows in self.history.items():
            if self.args.stop_after == 'combat': verify_presence(rows, role)
        self.evidence['recovery'] = recovery
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, full_route=self.saved is None and self.args.stop_after == 'full',
            stop_after=self.args.stop_after, evidence=self.evidence), indent=2))
        print('PASS About Face', self.args.stop_after, 'fresh' if self.saved is None else 'resumed', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--allow-revival', action='store_true', help='Verify exact individual fight deaths and revivals; team wipes still fail')
    parser.add_argument('--stop-after', choices=('combat', 'full'), default='full')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not all(0 <= v <= 500 for v in (args.delay_ms, args.jitter_ms)): parser.error('Delay/jitter must be 0..500 ms')
    if args.checkpoint and args.stop_after != 'full': parser.error('A resumed victory must continue toward the exit')
    pair = AboutFacePair(args)
    try: pair.run()
    except BaseException as error:
        pair.evidence.update(passed=False, failed_phase=pair.phase, error=str(error),
            process_exit_codes_before_cleanup={r:p.poll() for r,p in pair.processes.items()})
        raise
    finally: pair.close()


if __name__ == '__main__': main()
