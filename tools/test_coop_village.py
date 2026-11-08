"""Drive a complete village visit in two real client processes/profiles.

Each client receives only local test goals, plans from its own view, and sends
ordinary inputs through the real session. No position, damage or item writes.
Run in Anode; internet hosting is default. Full death/rejoin matrix is separate.
"""
import argparse
import json
import math
import os
from pathlib import Path
import subprocess
import time
import uuid

from test_coop_play import ROOT, read, distance
from coop_diagnostics import recent_movement, timing_summary


def verify_rejoin_stats(old, current, elapsed_ticks):
    """Idle reconnect permits only the game's normal simulation-time Will recovery."""
    assert type(elapsed_ticks) is int and elapsed_ticks >= 0, 'Rejoin moved the simulation clock backwards'
    for field in ('sanity', 'owned', 'collected', 'selected', 'school_items', 'staff_component', 'difficulty'):
        assert current[field] == old[field], f'Rejoin changed personal {field}'
    rate, cap = {'Easy': (.3, 100), 'Normal': (.1, 10), 'Hard': (0, 0), 'Nightmare': (0, 0)}[old['difficulty']]
    before, after = old['will'], current['will']
    assert math.isfinite(before) and math.isfinite(after), 'Rejoin produced invalid Will'
    expected = min(cap, before + elapsed_ticks / 120 * rate) if old['sanity'] > 0 and before < cap else before
    # Fixed-step f32 accumulation differs slightly from the analytic value.
    assert abs(after - expected) < .002, f'Rejoin changed personal Will beyond normal recovery: {before} -> {after}, expected {expected}'


class Pair:
    map_name = 'gvillage'
    next_map = 'pandemonium'
    evidence_prefix = 'village-test-'
    capture_scenes = True

    def __init__(self, args):
        self.args = args
        self.folder = ROOT / 'private' / 'coop' / (self.evidence_prefix + uuid.uuid4().hex)
        self.folder.mkdir(parents=True)
        self.invite = self.folder / 'invite.txt'
        self.processes, self.logs, self.commands, self.reports = {}, {}, {}, {}
        self.starts = {'host': 0, 'guest': 0}
        self.log_paths = {}
        self.history = {'host': [], 'guest': []}
        self.captures = set()
        self.capture_commands = {}
        self.serial = 0
        self.heartbeat = 0
        self.build = None
        self.evidence = {}
        self.deadline = time.monotonic() + 1500
        for role in self.history:
            (self.folder / role).mkdir()
            self.set(role)

    def set(self, role, goal=None, navigate=False, capture=False, finish=False, epoch=1,
            horizontal=False, rope=None, marker=None, attack=None, run=False, jump=False):
        self.serial += 1
        if attack is None:
            attack = self.commands.get(role, {}).get('attack', True)
        self.commands[role] = dict(id=self.serial, heartbeat=1, epoch=epoch,
                                  goal=goal, navigate=navigate, attack=attack,
                                  capture=capture, finish=finish, horizontal=horizontal, rope=rope, marker=marker,
                                  run=run, jump=jump)

    def start(self, role):
        assert role not in self.processes or self.processes[role].poll() is not None
        profile = self.folder / role
        if role in self.logs:
            self.logs[role].close()
        suffix = f'-rejoin{self.starts[role]}' if self.starts[role] else ''
        self.starts[role] += 1
        self.log_paths[role] = self.folder / (role + suffix + '.log')
        log = self.log_paths[role].open('w')
        self.logs[role] = log
        args = [str(self.args.exe.resolve()), '--data', str(self.args.data.resolve()), '--map', self.map_name,
                '--no-audio', '--profile', str(profile), '--coop-host' if role == 'host' else '--coop-join',
                str(self.invite), '--coop-route-driver', str(profile / 'route-command.json'),
                '--coop-delay-ms', str(self.args.delay_ms), '--coop-jitter-ms', str(self.args.jitter_ms)]
        if getattr(self, 'entry', None) is not None:
            args += ['--entry', self.entry]
        if getattr(self.args, 'audio', False):
            args.remove('--no-audio')
            args += ['--audio-capture', str(self.folder / (role + suffix + '.wav')), '--frames', '36000']
        if role == 'host' and getattr(self, 'start_at', None) is not None:
            args += ['--start-at', ' '.join(map(str, self.start_at))]
        if role == 'host' and self.args.local:
            args += ['--coop-local']
        if role == 'host' and getattr(self, 'resume_checkpoint', False):
            args += ['--coop-resume']
        self.processes[role] = subprocess.Popen(args, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)

    def pump(self):
        assert time.monotonic() < self.deadline, self.map_name + ' route timed out'
        self.heartbeat += 1
        for role, command in self.commands.items():
            command['heartbeat'] = self.heartbeat
            target = self.folder / role / 'route-command.json'
            temp = target.with_suffix('.tmp')
            temp.write_text(json.dumps(command))
            try:
                temp.replace(target)
            except PermissionError:
                pass  # Next heartbeat retries; the client neutralises stale input.
        for role, process in self.processes.items():
            assert process.poll() in (None, 0), f'{role} exited: {self.log_paths[role]}'
            report = read(self.folder / role / 'coop-live.json')
            if not report or report['pid'] != process.pid or not report['playing']:
                continue
            self.reports[role] = report
            frame, stage = report['frame'], report['combat']['stage']
            driver = report['route_driver']
            if not driver or driver['error'] and not self.recoverable_driver_error(role, report):
                # Keep terminal failures small. Full reports (including enemy
                # pools and network hashes) remain in the private evidence.
                details = {k: driver.get(k) for k in ('command', 'phase', 'error',
                    'observed_tick', 'replans', 'plan_remaining', 'expected')} if driver else None
                raise AssertionError(f'{role} driver failed: {details}')
            assert (driver['observed_epoch'], driver['observed_tick']) == (frame['epoch'], frame['tick']), 'Driver observed a different frame'
            if self.build is None:
                self.build = report['build']
            assert self.build == report['build'], 'Client build identities differ'
            self.validate_report(role, report)
            if stage['owner'] is None:
                self.validate_clear(role, report)
            row = {'tick': frame['tick'], 'epoch': frame['epoch'], 'map': report['visit']['map'],
                   'network_timing': report.get('network_timing'), 'frame_timing': report.get('frame_timing'),
                   'checkpoint_generation': frame['checkpoint_generation'], 'lives': frame['lives'],
                   'camera_player': report['camera_player'], 'camera_target': report['camera_target'],
                   'camera_subject_feet': report['camera_subject_feet'],
                   'weapon_effects': [{key: len(c['projectiles'].get(key, []))
                                       for key in ('sparks', 'trail', 'flight_trails')}
                                      for c in report['combat']['characters']],
                   'encounters': stage['world']['encounters'],
                   'owner': stage['owner'], 'scene': driver['scene'], 'driver': driver,
                   'players': [{'feet': b['player']['feet'], 'health': b['stats']['sanity'],
                       'stats': b['stats'],
                       'velocity': b['player']['velocity'], 'grounded': b['player']['grounded'],
                       'swimming': b['player']['swimming'], 'jumps': b['player']['jumps'],
                       'rope': b['player']['rope'],
                       'collected': b['stats']['collected'], 'warped_at': b['warped_at'],
                       'spawned_at': b['spawned_at']} for b in frame['players']]}
            rows = self.history[role]
            if not rows or (rows[-1]['epoch'], rows[-1]['tick']) != (row['epoch'], row['tick']):
                rows.append(row)
        # Capture both views of each actual scene without changing their goals.
        if self.capture_scenes and len(self.reports) == 2:
            scene = self.reports['host']['route_driver']['scene']
            if scene and scene not in self.captures and self.reports['guest']['route_driver']['scene'] == scene:
                pending = self.capture_commands.setdefault(scene, {})
                for role in self.commands:
                    captured = pending.get(role)
                    if captured and (self.folder / role / f'route-capture-{captured}.png').exists():
                        continue
                    if captured == self.commands[role]['id']:
                        continue
                    old = self.commands[role]
                    self.set(role, old['goal'], old['navigate'], capture=True, epoch=old['epoch'],
                             horizontal=old['horizontal'],rope=old['rope'],marker=old['marker'],
                             run=old['run'],jump=old['jump'])
                    pending[role] = self.commands[role]['id']
                if all(role in pending and (self.folder / role / f'route-capture-{pending[role]}.png').exists()
                       for role in self.commands):
                    self.captures.add(scene)
        time.sleep(.05)

    def recoverable_driver_error(self, role, report):
        return False

    def validate_report(self, role, report):
        assert all(b['stats']['sanity'] > 0 for b in report['frame']['players']), f'{role}: player died'
        assert report['frame']['checkpoint_generation'] == 0, 'Unexpected team wipe during clean traversal'

    def validate_clear(self, role, report):
        assert all(report['route_driver']['clear']), f'{role}: obstructed body'

    def until(self, predicate, seconds=240):
        end = time.monotonic() + seconds
        while not predicate():
            assert time.monotonic() < end, f'Leg timeout: {self.commands}; last drivers: ' + str({
                r: (d['frame']['tick'], d['route_driver'], [b['player']['feet'] for b in d['frame']['players']])
                for r, d in self.reports.items()})
            self.pump()

    def arrived(self):
        return len(self.reports) == 2 and all(d['visit']['map'] == self.next_map and d['visit']['epoch'] == 2
                                             for d in self.reports.values())

    def reached(self, role):
        report = self.reports.get(role)
        if not report:
            return False
        driver = report['route_driver']
        return driver['command'] == self.commands[role]['id'] and driver['reached'] and report['combat']['stage']['owner'] is None

    def go(self, goal, navigate=False):
        if self.arrived():
            return
        first = 'host' if self.args.first == 'blue' else 'guest'
        partner = 'guest' if first == 'host' else 'host'
        self.set(partner)
        self.set(first, goal, navigate)
        self.until(lambda: self.arrived() or self.reached(first))
        if not self.arrived():
            self.set(partner, goal, navigate)
            self.until(lambda: self.arrived() or all(self.reached(r) for r in self.commands))
        print('PASS network', self.map_name, 'leg', goal, flush=True)

    def wait(self, seconds):
        if self.arrived():
            return
        for role in self.commands:
            self.set(role)
        tick = self.reports['host']['frame']['tick']
        self.until(lambda: self.arrived() or self.reports['host']['frame']['tick'] >= tick + seconds * 120)

    def start_pair(self):
        print('Evidence:', self.folder, flush=True)
        self.pump()
        self.start('host')
        self.until(self.invite.exists, 150)
        self.start('guest')
        self.until(lambda: len(self.reports) == 2, 120)

    def route_to_guard(self):
        self.wait(6)
        for goal in [(-4180,5900,16),(-4192,5416,-52),(-4412,5328,-52),(-4780,5392,-120),(-4968,5424,-120)]: self.go(goal)
        self.wait(6)
        # This is a walkable approach, not a running jump. Use the recorded
        # single-player route's intermediate ground points so delayed steering
        # does not turn a wide stopping tolerance into an unnecessary water fall.
        for goal in [(-4994,5398,-120),(-5039,5353,-120),(-5054,5295,-120),(-5085,5245,-107),(-5100,5200,-120)]: self.go(goal)
        self.go((-5100,4780,-56), True)
        self.wait(6)
        for goal in [(-4784,4784,-120),(-4512,4800,-96),(-4288,4808,-136),(-3984,4712,-176),(-3972,4332,-160),(-3960,4056,-224),(-3968,3840,-272),(-3776,3832,-208),(-3544,3856,-136)]: self.go(goal)
        self.wait(15)
        for goal in [(-3208,4040,-56),(-2800,3872,-96)]: self.go(goal, True)

    def route_to_cliff(self):
        self.route_to_guard()
        self.go((-2808,3520,-96), True)
        # Confirm each real landing before the next jump. A single open-loop
        # plan for the whole climb can keep steering over a drop after network
        # timing makes its first jump miss the small intermediate ledge.
        self.go((-2744,3324,-104))
        self.go((-2628,3220,16), True)
        self.go((-2680,3028,68), True)
        self.go((-2936,3024,128))
        self.wait(8)

    def route_to_exit(self):
        # Walk the supported approach before committing to the final drop. These
        # points follow the planner's recorded ground path; a long running plan
        # magnifies a late first turn into a missed landing several steps later.
        for goal in [(-2960,2998,138),(-2976,2940,164),(-2976,2876,184),
                     (-2976,2780,184),(-2976,2720,184),(-2976,2660,184)]: self.go(goal)
        # Continue along the supported ledge before crossing to the lower one.
        # A long diagonal drop from y=2840 clips the lower landing's north edge
        # when network timing shifts an airborne turn by a few input samples.
        self.go((-3112,2584,40), True)
        self.go((-3312,2440,48))
        self.until(self.arrived, 120)

    def rejoin_guest(self):
        self.wait(.5)
        before = self.reports['host']
        (self.folder / 'before-rejoin.json').write_text(json.dumps(before))
        assert before['combat']['stage']['owner'] is None
        checkpoint = before['frame']['checkpoint_generation']
        previous_pid = self.processes['guest'].pid
        self.set('guest', finish=True)
        self.until(lambda: self.processes['guest'].poll() is not None, 30)
        assert self.processes['guest'].returncode == 0
        self.set('guest')
        self.reports.pop('guest')
        self.pump()
        self.start('guest')
        self.until(lambda: 'guest' in self.reports and self.reports['guest']['pid'] != previous_pid, 120)
        self.wait(.5)
        (self.folder / 'after-rejoin.json').write_text(json.dumps(self.reports))
        for report in self.reports.values():
            frame = report['frame']
            assert report['visit'] == before['visit'] and frame['checkpoint_generation'] == checkpoint
            assert frame['tick'] >= before['frame']['tick']
            self.verify_rejoined_world(before, report)
            for body, old in zip(frame['players'], before['frame']['players']):
                verify_rejoin_stats(old['stats'], body['stats'], frame['tick'] - before['frame']['tick'])
                assert body['spawned_at'] == old['spawned_at'] and body['warped_at'] == old['warped_at']
                assert distance(body['player']['feet'], old['player']['feet']) < 2, 'Rejoin relocated a player'
        self.evidence['rejoin'] = {'guest_processes': self.starts['guest'], 'epoch': before['visit']['epoch'],
            'before_tick': before['frame']['tick'], 'after_tick': self.reports['guest']['frame']['tick'],
            'resources_and_completed_scenes_preserved': True}
        print('PASS', self.map_name, 'guest left and rejoined at the same position with personal resources and scene progress intact.', flush=True)

    def verify_rejoined_world(self, before, report):
        assert report['combat']['stage']['world']['village']['cinema']['done'] == before['combat']['stage']['world']['village']['cinema']['done']

    def finish_pair(self, epoch):
        for role in self.commands:
            self.set(role, capture=True, epoch=epoch)
        self.until(lambda: all(self.reports[r]['route_driver']['captured'] == self.commands[r]['id'] for r in self.commands), 30)
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=epoch)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0

    def verify_network_states(self, field='sha256', metric='matching_sent_and_restored_states'):
        states = {}
        for role, rows in self.history.items():
            states[role] = {}
            for row in rows:
                for state in row['driver']['network_states']:
                    key = (state['epoch'], state['tick'])
                    previous = states[role].setdefault(key, state[field])
                    assert previous == state[field], f'An authoritative {field} tick changed after being recorded'
        shared = states['host'].keys() & states['guest'].keys()
        assert len(shared) >= 20, 'Too few matching sent/restored network states'
        assert all(states['host'][key] == states['guest'][key] for key in shared), f'Sent and restored {field} states disagree'
        self.evidence[metric] = len(shared)

    def run(self):
        self.start_pair()
        self.route_to_cliff()
        if self.args.rejoin:
            self.rejoin_guest()
        self.route_to_exit()
        self.verify()
        self.finish_pair(2)
        (self.folder / 'result.json').write_text(json.dumps({'passed': True, 'internet': not self.args.local,
            'build': self.build, 'evidence': self.evidence,
            'first': self.args.first, 'delay_ms': self.args.delay_ms, 'jitter_ms': self.args.jitter_ms,
            'scope': 'full village traversal in separate clients' +
                (' with guest leave/rejoin near the exit; death matrix remains separate' if self.args.rejoin
                 else '; death/rejoin matrix remains separate')}, indent=2))
        print('PASS two-process village: six scenes, personal Blade pickups, clean traversal and authored network transition.', flush=True)

    def verify(self):
        expected = {'entry','knife_cat','Torchgnome1_Dialog','Torchgnome2_Dialog','Torchgnome3_Dialog_part2','Torchgnome4_Dialog'}
        assert self.captures == expected, f'Missing paired scene captures: {expected - self.captures}'
        for role, rows in self.history.items():
            owners = {}
            reunions = set()
            moving_speech = 0
            for row in rows:
                if row['scene'] and row['epoch'] == 1:
                    owners.setdefault(row['scene'], set()).add(row['owner'])
                if row['epoch'] == 1 and row['owner'] is None and row['players'][0]['warped_at'] > 0:
                    warp = row['players'][0]['warped_at']
                    assert warp == row['players'][1]['warped_at'], f'{role}: players observed different scene handoffs'
                    # A report may arrive after ordinary movement has resumed.
                    allowance = 110 + max(0, row['tick'] - warp) * 640 / 120
                    assert distance(row['players'][0]['feet'], row['players'][1]['feet']) <= allowance
                    reunions.add(warp)
            for old, row in zip(rows, rows[1:]):
                if row['epoch'] == old['epoch'] == 1 and row['driver']['speech'] and old['driver']['speech']:
                    assert all(a['warped_at'] == b['warped_at'] for a, b in zip(old['players'], row['players'])), 'Voice-only line teleported a player'
                    moving_speech += any(distance(a['feet'], b['feet']) > 1 for a, b in zip(old['players'], row['players']))
                if row['epoch'] == old['epoch'] == 1 and row['owner'] is not None and row['owner'] == old['owner']:
                    partner = 1 if row['owner'] == 'Blue' else 0
                    if row['players'][partner]['warped_at'] == old['players'][partner]['warped_at']:
                        assert row['players'][partner]['feet'] == old['players'][partner]['feet'], f'{role}: waiting partner moved in a scene'
            assert set(owners) == expected, f'{role} did not observe every village scene: {owners}'
            assert all(owners[s] == {'Blue' if s == 'entry' else self.args.first.title()} for s in expected), owners
            assert len(reunions) == 6, f'{role}: expected six cinematic handoffs: {reunions}'
            assert moving_speech > 0, f'{role}: no movement observed during ordinary speech'
            assert any(r['map'] == 'pandemonium' and r['epoch'] == 2 for r in rows)
            assert any(r['epoch'] == 1 and all(r['driver']['blade_pickups']) for r in rows), f'{role} missed an actual personal Blade pickup'
            assert all(b['spawned_at'] == 0 for r in rows for b in r['players']), 'Traversal silently respawned a player'
            self.evidence[role] = {'scenes': len(owners), 'reunions': len(reunions), 'moving_speech_samples': moving_speech}
        host = {(r['epoch'],r['tick']): r for r in self.history['host']}
        compared = 0
        for guest in self.history['guest']:
            other = host.get((guest['epoch'],guest['tick']))
            if other:
                assert other['players'] == guest['players'] and other['owner'] == guest['owner'], 'Authoritative client states disagree'
                assert other['encounters'] == guest['encounters'], 'Authoritative enemy states disagree'
                compared += 1
        self.verify_network_states()
        self.evidence['matching_authoritative_ticks'] = compared
        print('Compared', compared, 'matching authoritative ticks.', flush=True)

    def close(self):
        (self.folder / 'recent-movement.json').write_text(json.dumps(recent_movement(self.history)))
        (self.folder / 'timing-summary.json').write_text(json.dumps(timing_summary(self.history)))
        (self.folder / 'history.json').write_text(json.dumps(self.history))
        (self.folder / 'observations.json').write_text(json.dumps(self.evidence, indent=2))
        (self.folder / 'captures.json').write_text(json.dumps(self.capture_commands, indent=2))
        for process in self.processes.values():
            if process.poll() is None:
                subprocess.run(['taskkill','/PID',str(process.pid),'/T','/F'], capture_output=True)
                process.wait(timeout=10)
        for log in self.logs.values():
            log.close()
        self.invite.unlink(missing_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue','red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true', help='Leave/rejoin the guest near the authored village exit')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500): parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = Pair(args)
    try:
        pair.run()
    finally:
        pair.close()
