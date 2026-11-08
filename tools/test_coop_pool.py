"""Ordinary-input Pool of Tears route through two independent rendered clients.

Public internet and delay/jitter are the defaults. The current entrance/bank
segments are partial campaign evidence; never count them as a completed visit.
Run in Anode. Saves and diagnostic state stay in ignored private directories.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_pandemonium import PandemoniumPair


class PoolPair(PandemoniumPair):
    map_name = 'potears1'
    entry = 'potears1_start1'
    next_map = 'potears2'
    evidence_prefix = 'pool-route-'

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()
        self.deadline = time.monotonic() + 900
        self.saved = None
        self.resume_checkpoint = args.checkpoint is not None
        if self.resume_checkpoint:
            saved = args.checkpoint.read_bytes()
            self.saved = json.loads(saved)['payload']
            assert self.saved['visit']['map'] == self.map_name and self.saved['visit']['entry'] == self.entry
            assert self.saved['visit']['epoch'] == 1 and self.saved['frame']['checkpoint_generation'] == 0
            assert all(b['spawned_at'] == 0 and b['stats']['sanity'] > 0 for b in self.saved['frame']['players'])
            (self.folder / 'host' / 'coop-checkpoint.json').write_bytes(saved)
            self.evidence.update(checkpoint_sha256=hashlib.sha256(saved).hexdigest(), resume_index=args.resume_index)

    def set(self, role, *args, speed_limit=None, arrival_radius=6., **kwargs):
        super().set(role, *args, **kwargs)
        self.commands[role].update(
            input_lead=min(1.5, .05 + 2 * (self.args.delay_ms + self.args.jitter_ms) / 1000)
                if role == 'guest' else 0,
            speed_limit=speed_limit, arrival_radius=arrival_radius,
            watch_dialogue=True)

    def pool(self, role='host'):
        return self.reports[role]['combat']['stage']['world']['pool']

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Pool run stopped; partial evidence retained')
        super().pump()
        for role, report in self.reports.items():
            row = self.history[role][-1]
            row['pool'] = report['combat']['stage']['world']['pool']
            if row['pool']:
                row['scene'] = row['pool']['cinema']['beat']

    def until(self, predicate, seconds=90):
        deadline = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= deadline:
                raise AssertionError('Pool leg timeout: ' + json.dumps({
                    role: dict(tick=report['frame']['tick'],
                        command=self.commands[role],
                        phase=report['route_driver']['phase'],
                        feet=[b['player']['feet'] for b in report['frame']['players']],
                        health=[b['stats']['sanity'] for b in report['frame']['players']],
                        owner=report['combat']['stage']['owner'])
                    for role, report in self.reports.items()}))
            self.pump()

    def settled(self, role):
        index = 0 if role == 'host' else 1
        return self.reached(role) and all(
            self.bodies(view)[index]['player']['grounded']
            and distance(self.bodies(view)[index]['player']['velocity'][:2], [0, 0]) < 12
            for view in (role, 'host'))

    def move_aside(self, role):
        index = 0 if role == 'host' else 1
        landing = self.bodies()[index]['player']['feet']
        incoming = self.bodies()[1-index]['player']['feet']
        tried = []
        for _ in range(4):
            anchor = self.bodies()[index]['player']['feet']
            neighbors = self.reports['host']['route_driver']['neighbors'][index]
            def clear(point):
                if not 40 <= distance(point[:2], landing[:2]) <= 96 \
                        or any(distance(point, old) < 24 for old in tried):
                    return False
                if min(abs(anchor[2]-incoming[2]), abs(point[2]-incoming[2])) >= 56:
                    return True
                if distance(point[:2], incoming[:2]) < 40:
                    return False
                delta = [point[i]-anchor[i] for i in range(2)]
                length = sum(v*v for v in delta)
                along = max(0, min(1, sum((incoming[i]-anchor[i])*delta[i]
                    for i in range(2)) / length)) if length else 0
                closest = [anchor[i]+along*delta[i] for i in range(2)]
                return distance(closest, incoming[:2]) >= min(39, distance(anchor[:2], incoming[:2]))-.1
            neighbors = [p for p in neighbors if clear(p)]
            assert neighbors, f'No supported waiting spot near {anchor}'
            approach = [landing[i]-incoming[i] for i in range(2)]
            goal = max(neighbors, key=lambda p: abs((p[0]-landing[0])*approach[1] -
                                                   (p[1]-landing[1])*approach[0]))
            tried.append(goal)
            observation = dict(role=role, anchor=anchor, goal=goal, reached=False)
            self.evidence.setdefault('waiting_spots', []).append(observation)
            self.set(role, goal, speed_limit=80, arrival_radius=1.)
            progress, checked = anchor, self.reports['host']['frame']['tick']
            deadline = time.monotonic() + 10
            while not self.settled(role) and time.monotonic() < deadline:
                self.pump()
                tick = self.reports['host']['frame']['tick']
                actual = self.bodies()[index]['player']['feet']
                if tick >= checked + 120:
                    if distance(actual, progress) < .5:
                        break
                    progress, checked = actual, tick
            self.set(role)
            self.hold(.5)
            actual = self.bodies()[index]['player']['feet']
            if self.bodies()[index]['player']['grounded'] and distance(actual[:2], goal[:2]) < 3 \
                    and abs(actual[2]-goal[2]) < 2:
                observation['reached'] = True
                return
            # A supported endpoint need not have a walkable straight approach.
            # Reobserve nearby ground after a blocked attempt; never relocate
            # either body or accept an obstructed/unsupported waiting point.
        raise AssertionError(f'No reachable waiting spot near {landing}')

    def go_one(self, role, goal, jump=False, runup=None):
        self.set(role, goal, run=True, jump=jump)
        if runup:
            self.commands[role]['runup'] = runup
        index = 0 if role == 'host' else 1
        previous = self.bodies()[index]['player']['feet']
        checked = self.reports['host']['frame']['tick']
        scene_owner = self.reports['host']['combat']['stage']['owner']
        def at_goal():
            # These authored wayfinding targets use the native route's
            # twenty-unit tolerance; some sit just past a bank's solid edge.
            return all(self.reports[view]['combat']['stage']['owner'] is None
                and self.bodies(view)[index]['player']['grounded']
                and distance(self.bodies(view)[index]['player']['feet'][:2], goal[:2]) < 20
                and abs(self.bodies(view)[index]['player']['feet'][2] - goal[2]) < 48
                for view in (role, 'host'))
        deadline = time.monotonic() + 60
        while not self.settled(role):
            assert time.monotonic() < deadline, f'Pool approach stalled for {role} at {goal}'
            self.pump()
            owner = self.reports['host']['combat']['stage']['owner']
            if scene_owner is not None and owner is None:
                # Reunion can place the waiting partner on this approach's
                # destination. Clear them with ordinary supported walking
                # before resuming; do this once per observed scene exit.
                scene_owner = None
                other = 'guest' if role == 'host' else 'host'
                self.set(role)
                self.set(other)
                if at_goal() and all(distance(self.bodies(view)[index]['player']['velocity'][:2], [0, 0]) < 12
                        for view in (role, 'host')):
                    return
                self.hold(.5)
                other_index = 1-index
                other_feet = self.bodies()[other_index]['player']['feet']
                if not at_goal() and distance(other_feet[:2], goal[:2]) < 40 \
                        and abs(other_feet[2]-goal[2]) < 56:
                    self.move_aside(other)
                    self.evidence.setdefault('reunion_clearances', []).append(
                        dict(role=role, tick=self.reports['host']['frame']['tick'], goal=goal))
                self.set(role, goal, run=True)
                previous = self.bodies()[index]['player']['feet']
                checked = self.reports['host']['frame']['tick']
                deadline = time.monotonic() + 60
                continue
            scene_owner = owner
            if at_goal():
                self.set(role)
                self.hold(.8)
                if at_goal() and all(distance(self.bodies(view)[index]['player']['velocity'][:2], [0, 0]) < 12
                        for view in (role, 'host')):
                    return
                self.set(role, goal, run=True, speed_limit=80)
            report = self.reports[role]
            player = self.bodies()[index]['player']
            tick = self.reports['host']['frame']['tick']
            if tick >= checked + 120:
                # The single-player Pool pilot also jumps when its bank
                # approach stops advancing. Request that normal input once,
                # then wait for the resulting movement before deciding again.
                hanging = player.get('ledge') and not player['ledge']['pulling']
                if report['route_driver']['phase'] == 'walking' and (player['grounded'] or hanging) \
                        and distance(player['feet'][:2], goal[:2]) > 20 \
                        and distance(player['feet'], previous) < 18:
                    self.set(role, goal, run=True, jump=True)
                    self.evidence.setdefault('bank_jumps', []).append(dict(
                        role=role, tick=tick, feet=player['feet'], goal=goal,
                        ledge_pull=bool(hanging)))
                previous, checked = player['feet'], tick

    def go_pair(self, goal, jump=False, takeoff=None, runup=None, together=False):
        warped = [b['warped_at'] for b in self.bodies()]
        if together:
            # Both must leave the boulder's path after its scene. Let the
            # actual first arrival take a waiting spot while the other follows.
            for role in self.commands:
                self.set(role, goal, run=True)
            def near(role, radius=20):
                body = self.bodies()[0 if role == 'host' else 1]['player']
                return body['grounded'] and distance(body['feet'][:2], goal[:2]) < radius \
                    and abs(body['feet'][2]-goal[2]) < 48
            self.until(lambda: any(near(role) for role in self.commands)
                and all(near(role, 64) for role in self.commands), 30)
            lead = next(role for role in self.commands if near(role))
            follow = 'guest' if lead == 'host' else 'host'
            for role in self.commands:
                self.set(role)
            self.hold(.8)
            self.go_one(lead, goal)
            self.set(lead)
            bodies = self.bodies()
            if all(b['player']['grounded']
                   and distance(b['player']['feet'][:2], goal[:2]) < 64
                   and abs(b['player']['feet'][2]-goal[2]) < 48
                   and distance(b['player']['velocity'][:2], [0, 0]) < 12 for b in bodies) \
                    and distance(bodies[0]['player']['feet'][:2], bodies[1]['player']['feet'][:2]) >= 36:
                self.evidence.setdefault('paired_escapes', []).append(dict(
                    goal=goal, tick=self.reports['host']['frame']['tick'],
                    feet=[b['player']['feet'] for b in bodies]))
                print('PASS Pool both leave boulder path onto the bank', goal, flush=True)
                return
            self.move_aside(lead)
            self.go_one(follow, goal)
            self.set(follow)
            print('PASS Pool paired boulder escape', goal, flush=True)
            return
        self.set(self.partner)
        if takeoff:
            other = 0 if self.partner == 'host' else 1
            if distance(self.bodies()[other]['player']['feet'][:2], takeoff[:2]) < 40:
                self.move_aside(self.partner)
            self.go_one(self.first, takeoff)
        self.go_one(self.first, goal, jump, runup)
        self.set(self.first)
        # An original scene can finish with both already on this bank, in
        # distinct supported positions. Do not make them exchange places on
        # a narrow landing merely to touch the same wayfinding coordinate.
        bodies = self.bodies()
        if all(b['warped_at'] != old and b['player']['grounded']
               and distance(b['player']['feet'][:2], goal[:2]) < 96
               and abs(b['player']['feet'][2]-goal[2]) < 48
               for b, old in zip(bodies, warped)) \
                and distance(bodies[0]['player']['feet'][:2], bodies[1]['player']['feet'][:2]) >= 36:
            self.evidence.setdefault('paired_reunions', []).append(dict(
                goal=goal, tick=self.reports['host']['frame']['tick'],
                feet=[b['player']['feet'] for b in bodies]))
            print('PASS Pool paired scene landing', goal, flush=True)
            return
        self.move_aside(self.first)
        if takeoff:
            self.go_one(self.partner, takeoff)
        self.go_one(self.partner, goal, jump, runup)
        self.set(self.partner)
        print('PASS Pool paired waypoint', goal, flush=True)

    def save_checkpoint(self, label):
        for role in self.commands:
            self.set(role, attack=False)
        self.wait(.5)
        self.commands['host']['checkpoint'] = True
        command = self.commands['host']['id']
        self.until(lambda: (self.reports['host']['route_driver'].get('checkpoint') or {})
            .get('command') == command, 15)
        receipt = self.reports['host']['route_driver']['checkpoint']
        assert receipt['epoch'] == 1 and receipt['error'] is None, receipt
        saved = (self.folder / 'host' / 'coop-checkpoint.json').read_bytes()
        payload = json.loads(saved)['payload']
        assert payload['frame']['tick'] == receipt['tick'] and payload['visit']['map'] == self.map_name
        path = self.folder / f'checkpoint-{label}.json'
        path.write_bytes(saved)
        self.evidence.setdefault('checkpoints', {})[label] = dict(file=path.name,
            sha256=hashlib.sha256(saved).hexdigest(), **receipt)
        for role in self.commands:
            self.set(role, attack=True)
        print('PASS Pool ordinary checkpoint retained:', label, flush=True)

    def verify(self):
        for role, rows in self.history.items():
            if not self.saved:
                assert any(r['owner'] == 'Blue' and r['scene'] == 'Entry' for r in rows), \
                    f'{role}: opening scene not observed'
            for before, after in zip(rows, rows[1:]):
                if before['owner'] == after['owner'] and after['owner'] in ('Blue', 'Red'):
                    partner = 1 if after['owner'] == 'Blue' else 0
                    assert before['players'][partner]['feet'] == after['players'][partner]['feet'], \
                        f'{role}: waiting partner moved during a scene'
            assert all(r['owner'] == self.args.first.title() for r in rows
                if r['scene'] in ('Boulder1', 'Boulder2') and r['owner'] is not None), \
                f'{role}: rock scene did not belong to the first arrival'
            assert all(b['spawned_at'] == 0 for r in rows for b in r['players']), 'Unexpected respawn'
            assert self.pool(role)['cinema']['done'][0], f'{role}: opening scene unfinished'
            if self.args.stop_after in ('bank2', 'bank3', 'bank4'):
                assert self.pool(role)['cinema']['done'][1], f'{role}: first falling-rock scene unfinished'
            if self.args.stop_after in ('bank3', 'bank4'):
                assert self.pool(role)['cinema']['done'][2], f'{role}: second falling-rock scene unfinished'
            assert self.reports[role]['combat']['stage']['owner'] is None
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_sent_and_restored_world_states')

    def run(self):
        self.start_pair()
        self.until(lambda: all(self.pool(role)['cinema']['done'][0] and
            self.reports[role]['combat']['stage']['owner'] is None for role in self.reports), 180)
        self.wait(1)
        label = 'restored_positions' if self.saved else 'entrance_handoff'
        self.evidence[label] = [b['player']['feet'] for b in self.bodies()]
        print('PASS Pool start', label, self.evidence[label], flush=True)
        if self.args.stop_after in ('bank1', 'bank2', 'bank3', 'bank4'):
            waypoints = [(-3160, 3096, 178.13634), (-3008, 3352, 107.15195),
                         (-2912, 3536, 55.28264), (-2768, 3568, 37.2)]
            if self.args.stop_after in ('bank2', 'bank3', 'bank4'):
                waypoints += [(-2496, 3504, 100), (-2496, 2944, 277.28308),
                              (-2440, 2240, 500), (-2360, 2816, 720),
                              (-2360, 3248, 756), (-2376, 3744, 760), (-2344, 3936, 948)]
            if self.args.stop_after in ('bank3', 'bank4'):
                waypoints += [(-2240, 3936, 990), (-2256, 3892, 960),
                              (-2232, 3344, 1216), (-2128, 3168, 1272), (-1968, 3072, 1280)]
            if self.args.stop_after == 'bank4':
                waypoints += [(-2128, 3104, 1248), (-2150, 3020, 1248),
                              (-2180, 2900, 1294), (-2216, 2832, 1328), (-2208, 2700, 1360),
                              (-2208, 2464, 1472), (-2208, 2432, 1472),
                              (-2168, 2256, 1520), (-2136, 2512, 1632)]
            jumps = {
                5: ((-2778, 3568, 37.2), dict(millis=150, wish=[1, 0])),
                14: ((-2256, 3910, 960), dict(millis=150, wish=[0, -1])),
                22: ((-2208, 2719, 1360), dict(millis=150, wish=[0, -1])),
                24: ((-2208, 2450, 1472), dict(millis=150, wish=[0, -1])),
            }
            for index, goal in enumerate(waypoints, 1):
                if index <= (self.args.resume_index or 0):
                    continue
                # The bank gap needs a running takeoff from the path, not a
                # standing diagonal jump from the partner's waiting position.
                takeoff, runup = jumps.get(index, (None, None))
                self.go_pair(goal, jump=index in jumps, takeoff=takeoff, runup=runup,
                    together=index == 15)
                if index == 14:
                    # The original rock keeps falling after this scene. Leave
                    # immediately; save the pair once both reach the next bank.
                    continue
                if index in (9, 16):
                    self.wait(20 if index == 9 else 16)
                self.save_checkpoint(f'bank-path{index}')
            self.save_checkpoint(self.args.stop_after)
        self.wait(1)
        self.verify()
        self.evidence['final_health'] = [b['stats']['sanity'] for b in self.bodies()]
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, first=self.args.first, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope=f'Pool {"resumed segment" if self.saved else "entrance"} through {self.args.stop_after} only; full visit remains unverified'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--stop-after', choices=('entrance', 'bank1', 'bank2', 'bank3', 'bank4'), default='bank1')
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--resume-index', type=int, choices=range(1, 26))
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if bool(args.checkpoint) != bool(args.resume_index):
        parser.error('Supply both the unchanged ordinary checkpoint and its completed waypoint index')
    if args.resume_index and args.resume_index >= {'entrance': 0, 'bank1': 4,
            'bank2': 11, 'bank3': 16, 'bank4': 25}[args.stop_after]:
        parser.error('The resumed route must contain at least one unfinished waypoint')
    pair = PoolPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
