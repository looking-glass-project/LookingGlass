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
        return self.reached(role) and self.bodies(role)[0 if role == 'host' else 1]['player']['grounded']

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
        deadline = time.monotonic() + 60
        while not self.settled(role):
            assert time.monotonic() < deadline, f'Pool approach stalled for {role} at {goal}'
            self.pump()
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

    def go_pair(self, goal, jump=False, takeoff=None, runup=None):
        self.set(self.partner)
        if takeoff:
            other = 0 if self.partner == 'host' else 1
            if distance(self.bodies()[other]['player']['feet'][:2], takeoff[:2]) < 40:
                self.move_aside(self.partner)
            self.go_one(self.first, takeoff)
        self.go_one(self.first, goal, jump, runup)
        self.set(self.first)
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
            assert all(b['spawned_at'] == 0 for r in rows for b in r['players']), 'Unexpected respawn'
            assert self.pool(role)['cinema']['done'][0], f'{role}: opening scene unfinished'
            if self.args.stop_after == 'bank2':
                assert self.pool(role)['cinema']['done'][1], f'{role}: first falling-rock scene unfinished'
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
        if self.args.stop_after in ('bank1', 'bank2'):
            waypoints = [(-3160, 3096, 178.13634), (-3008, 3352, 107.15195),
                         (-2912, 3536, 55.28264), (-2768, 3568, 37.2)]
            if self.args.stop_after == 'bank2':
                waypoints += [(-2496, 3504, 100), (-2496, 2944, 277.28308),
                              (-2440, 2240, 500), (-2360, 2816, 720),
                              (-2360, 3248, 756), (-2376, 3744, 760), (-2344, 3936, 948)]
            for index, goal in enumerate(waypoints, 1):
                if index <= (self.args.resume_index or 0):
                    continue
                # The bank gap needs a running takeoff from the path, not a
                # standing diagonal jump from the partner's waiting position.
                self.go_pair(goal, jump=index == 5,
                    takeoff=(-2778, 3568, 37.2) if index == 5 else None,
                    runup=dict(millis=150, wish=[1, 0]) if index == 5 else None)
                if index == 9:
                    self.wait(20)
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
    parser.add_argument('--stop-after', choices=('entrance', 'bank1', 'bank2'), default='bank1')
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--resume-index', type=int, choices=range(1, 12))
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if bool(args.checkpoint) != bool(args.resume_index):
        parser.error('Supply both the unchanged ordinary checkpoint and its completed waypoint index')
    pair = PoolPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
