"""Anode: real ESC Save during deaths/scenes, followed by a safe retry.

Death uses ordinary first-school guard damage and movement inputs. Scene uses
the existing staged theatre approach, then watches the original scene. Neither
case writes health, position, world or save state. Public invites are default.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

from test_coop_menu import Run, Desktop, wait
from test_coop_play import ROOT, read
from test_coop_scene import verify as verify_scene
from test_coop_village_recovery import Recovery


class AnodeDesktop(Desktop):
    def focus(self, process):
        assert process.poll() is None, 'Owned game exited'
        hwnd = wait(lambda: self.window(process.pid), label='owned game window')
        if self.user.GetForegroundWindow() != hwnd:
            # The seat agent can focus our window even when Windows refuses
            # SetForegroundWindow from a background command job. Reuse the
            # caller's lease; never acquire another identity or touch other apps.
            assert os.environ.get('ANODE_AGENT_ID') and os.environ.get('ANODE_LEASE_TOKEN'), 'Forward the active Anode agent/lease to this test'
            cli = shutil.which('anode') or str(Path(os.environ['LOCALAPPDATA']) / 'Programs/Anode/anode.exe')
            def call(*arguments):
                return subprocess.run([cli, *arguments], capture_output=True, text=True,
                    check=True, timeout=10, creationflags=subprocess.CREATE_NO_WINDOW)
            windows = json.loads(call('windows', '--pid', str(process.pid), '--json').stdout)['windows']
            owned = [window for window in windows if window['pid'] == process.pid and window['className'] == 'MINIQUADAPP']
            assert len(owned) == 1 and process.poll() is None, 'Ambiguous or expired game window'
            call('window', owned[0]['windowId'], 'focus')
            wait(lambda: self.user.GetForegroundWindow() == hwnd, 3, 'Anode game focus')
        return hwnd


class Menus(Run):
    """Use observed native buttons in an already-running owned pair."""
    def __init__(self, args, folder, processes, pump):
        session = w.DWORD()
        assert c.WinDLL('kernel32').ProcessIdToSessionId(os.getpid(), c.byref(session))
        assert session.value == args.seat_session and session.value != 0, 'Wrong desktop session'
        self.args, self.folder, self.processes = args, folder, processes
        self.desktop, self.pump = AnodeDesktop(), pump

    def until(self, predicate, label):
        def sample():
            self.pump()
            return predicate()
        return wait(sample, 10, label)

    def save(self, expected_error=None):
        if self.state('host').get('screen') == 'coop-play':
            self.key('host', 0x1b)
        count = len(self.events('host', 'checkpoint-save'))
        self.click('host', 'Save checkpoint')
        receipt = self.until(lambda: self.events('host', 'checkpoint-save')[count:], 'new Save result')[0]
        if expected_error is None:
            assert receipt['error'] is None, receipt
            message = 'Co-op checkpoint saved for both players.'
        else:
            assert expected_error in (receipt['error'] or ''), receipt
            message = 'Checkpoint was not saved: ' + receipt['error']
        self.until(lambda: self.events('host', 'coop-panel-message')[-1] == message, 'displayed Save feedback')
        return receipt


def save_files(folder):
    return {name: (folder / 'host' / name).read_bytes() if (folder / 'host' / name).exists() else None
            for name in ('coop-checkpoint.json', 'coop-checkpoint.json.bak')}


def verify_invite_scope(path, local):
    link = path.read_text().strip()
    assert link.startswith('http://127.0.0.1:' if local else 'https://') and '/#lg-coop=1.' in link, 'Wrong invite network scope'


def retain_sources(folder):
    names = ['test_coop_menu_save_rules.py', 'test_coop_menu.py', 'test_coop_scene.py',
             'test_coop_village_recovery.py', 'test_coop_village.py']
    hashes = {}
    for name in names:
        data = (ROOT / 'tools' / name).read_bytes()
        (folder / name).write_bytes(data)
        hashes[name] = hashlib.sha256(data).hexdigest()
    return hashes


class DeathSave(Recovery):
    map_name, next_map = 'skool1', 'skool2'
    evidence_prefix = 'menu-death-save-'
    capture_scenes = False
    death_goal = (-1888, 2298, -512)

    def __init__(self, args):
        super().__init__(args)
        self.menu = Menus(args, self.folder, self.processes, self.pump)
        self.evidence['sources'] = retain_sources(self.folder)

    def set(self, role, *args, **kwargs):
        super().set(role, *args, **kwargs)
        # Same measured steering lead as the school traversal fixture, without
        # importing its unrelated local edits or running the full route.
        self.commands[role]['input_lead'] = min(1.5, .05 + 2*(self.args.delay_ms+self.args.jitter_ms)/1000) if role == 'guest' else 0
        self.commands[role]['arrival_radius'] = 8.

    def start(self, role):
        profile = self.folder / role
        self.log_paths[role] = self.folder / (role + '.log')
        self.logs[role] = self.log_paths[role].open('w')
        command = [str(self.args.exe.resolve()), '--data', str(self.args.data.resolve()), '--map', self.map_name,
                   '--no-audio', '--profile', str(profile), '--coop-host' if role == 'host' else '--coop-join',
                   str(self.invite), '--coop-route-driver', str(profile / 'route-command.json'),
                   '--coop-delay-ms', str(self.args.delay_ms), '--coop-jitter-ms', str(self.args.jitter_ms)]
        if role == 'host' and self.args.local:
            command.append('--coop-local')
        self.processes[role] = subprocess.Popen(command, cwd=ROOT, stdout=self.logs[role],
            stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW,
            env=dict(os.environ, LOOKING_GLASS_UI_PROBE='1'))

    def approach_fall(self, roles):
        assert len(roles) == 1
        victim = roles[0]
        partner = 'guest' if victim == 'host' else 'host'
        index = 0 if victim == 'host' else 1
        feet = self.reports['host']['frame']['players'][index]['player']['feet']
        y = feet[1] + 64
        # Keep two lanes in the wider entrance. The survivor/just-revived body
        # blocks a straight line; walk around it rather than through its collider.
        self.set(victim, (feet[0], y, -512))
        self.until(lambda: self.reached(victim), 15)
        self.set(victim, (-1856, y, -512))
        self.until(lambda: self.reached(victim), 15)
        self.set(partner, (-1920, 2048, -512))
        self.until(lambda: self.reached(partner), 15)
        for goal in ((-1856, 2160, -512), (-1888, 2160, -512)):
            self.set(victim, goal)
            self.until(lambda: self.reached(victim), 20)

    def survivor_goal(self, victim_index):
        return (-1920, 1960, -512) if victim_index == (1 if self.args.first == 'red' else 0) else (-1860, 1856, -512)

    def capture_pair(self, label):
        # Reuse the recovery verifier's stable countdown/respawn checkpoints,
        # recording menu input/results instead of requesting image captures.
        before_files = save_files(self.folder)
        before = self.reports['host']
        dead = label.endswith('-spectating')
        receipt = self.menu.save('Both players must be alive' if dead else None)
        after = self.reports['host']
        if dead:
            assert save_files(self.folder) == before_files, 'Refused save changed a checkpoint file'
            index = 0 if label.startswith('host-') else 1
            for report in (before, after):
                assert report['frame']['players'][index]['stats']['sanity'] == 0
                life = report['frame']['lives']['states'][index]
                assert isinstance(life, dict) and 'Spectating' in life
            a, b = before['frame'], after['frame']
            assert a['lives']['states'][index]['Spectating']['ticks_left'] - b['lives']['states'][index]['Spectating']['ticks_left'] == b['tick'] - a['tick'], 'Save attempt changed countdown'
        else:
            saved = read(self.folder / 'host/coop-checkpoint.json')['payload']
            assert saved['frame']['tick'] == receipt['tick']
            assert saved['frame']['lives']['states'] == ['Alive', 'Alive']
            assert all(b['stats']['sanity'] > 0 for b in saved['frame']['players'])
            assert save_files(self.folder) != before_files, 'Safe retry did not update checkpoint'
        (self.folder / (label + '.json')).write_text(json.dumps(dict(before=before, after=after, receipt=receipt)))
        self.evidence.setdefault('menu_saves', {})[label] = receipt
        self.menu.click('host', 'Resume')
        print('PASS real-menu', label, 'refused safely' if dead else 'saved after revival', flush=True)

    def run(self):
        for role in self.commands:
            self.set(role, attack=False)
        self.start_pair()
        verify_invite_scope(self.invite, self.args.local)
        self.until(lambda: read(self.folder / 'host/coop-checkpoint.json') is not None, 30)
        self.evidence['initial_save'] = self.menu.save()
        self.menu.click('host', 'Resume')
        # Red starts at the front of the narrow entrance. Clear that section
        # first, then give both players ordinary, separated approach positions.
        self.set('guest', (-1920, 2048, -512) if self.args.first == 'blue' else (-1856, 2160, -512))
        self.until(lambda: self.reached('guest'), 25)
        for goal in ((-1888, 1950, -512),
                     (-1856, 2000, -512) if self.args.first == 'blue' else (-1920, 2048, -512)):
            self.set('host', goal)
            self.until(lambda: self.reached('host'), 25)
        for victim in ('guest', 'host') if self.args.first == 'red' else ('host', 'guest'):
            self.fall_and_respawn(victim)
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_shared_world_states')
        self.verify_network_states('native_sha256', 'matching_native_enemies')
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0


class SceneSave(Run):
    def __init__(self, args):
        super().__init__(args)
        self.invite = self.folder / 'invite.txt'
        self.reports, self.history = {}, {'host': [], 'guest': []}
        self.evidence = {'sources': retain_sources(self.folder)}
        self.build = None
        self.menu = Menus(args, self.folder, self.processes, self.pump)

    def start(self, role):
        log = (self.folder / (role + '.log')).open('w')
        self.logs.append(log)
        command = [str(self.args.exe.resolve()), '--data', str(self.args.data.resolve()), '--map', 'skool1',
                   '--no-audio', '--profile', str(self.folder / role),
                   '--coop-host' if role == 'host' else '--coop-join', str(self.invite),
                   '--coop-scene-check', self.args.first, '--coop-delay-ms', str(self.args.delay_ms),
                   '--coop-jitter-ms', str(self.args.jitter_ms)]
        if role == 'host' and self.args.local:
            command.append('--coop-local')
        self.processes[role] = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
            creationflags=subprocess.CREATE_NO_WINDOW, env=dict(os.environ, LOOKING_GLASS_UI_PROBE='1'))

    def pump(self):
        for role, process in self.processes.items():
            assert process.poll() in (None, 0), f'{role} failed: inspect its log'
            report = read(self.folder / role / 'coop-live.json')
            if not report or report['pid'] != process.pid or not report['playing']:
                continue
            assert self.build in (None, report['build']), 'Mixed native builds'
            self.build = report['build']
            self.reports[role] = report
            frame, stage = report['frame'], report['combat']['stage']
            rows = self.history[role]
            if rows and rows[-1]['tick'] == frame['tick']:
                continue
            rows.append(dict(tick=frame['tick'], checkpoint_generation=frame['checkpoint_generation'],
                owner=stage['owner'], camera=report['camera'], scene_time=(stage['world']['school'].get('first_cinema') or {}).get('time'),
                visible_players=report['presentation']['visible_players'],
                players=[dict(feet=b['player']['feet'], health=b['stats']['sanity'],
                              spawned_at=b['spawned_at'], warped_at=b['warped_at']) for b in frame['players']]))

    def until(self, predicate, timeout, label):
        def sample():
            self.pump()
            return predicate()
        return wait(sample, timeout, label)

    def run(self):
        self.start('host')
        self.until(self.invite.exists, 150, 'host invite')
        verify_invite_scope(self.invite, self.args.local)
        self.button('host', 'Save checkpoint')
        self.evidence['initial_save'] = self.menu.save()
        saved = save_files(self.folder)
        self.start('guest')
        self.until(lambda: len(self.reports) == 2, 90, 'both clients playing')
        for seconds in (6, 30):
            self.until(lambda: self.reports['host']['combat']['stage']['owner'] == self.args.first.title()
                and self.reports['host']['combat']['stage']['world']['school']['first_cinema']['time'] >= seconds,
                55, 'original scene phase')
            before = self.reports['host']
            line = before['combat']['stage']['story']['current']
            assert (line is None) == (seconds == 6), 'Save attempt missed the intended introduction/dialogue phase'
            receipt = self.menu.save('Wait until the scene or chapter transition finishes')
            assert save_files(self.folder) == saved, 'Scene save refusal changed committed files'
            after = self.reports['host']
            assert before['combat']['stage']['owner'] == after['combat']['stage']['owner'] == self.args.first.title()
            (self.folder / f'scene-save-{seconds}.json').write_text(json.dumps(dict(before=before, after=after, receipt=receipt)))
            self.evidence.setdefault('refused_scene_saves', []).append(receipt)
            print('PASS real-menu save refused during', self.args.first, 'theatre phase', seconds, flush=True)
        self.until(lambda: all(r['combat']['stage']['owner'] is None and
            all(b['warped_at'] > 0 for b in r['frame']['players']) for r in self.reports.values()),
            65, 'original scene completion and safe reunion')
        self.evidence['safe_save'] = self.menu.save()
        checkpoint = read(self.folder / 'host/coop-checkpoint.json')['payload']
        assert checkpoint['frame']['tick'] == self.evidence['safe_save']['tick']
        assert all(b['warped_at'] > 0 for b in checkpoint['frame']['players'])
        assert checkpoint['frame']['combat']['stage']['owner'] is None
        assert save_files(self.folder) != saved
        self.evidence['scene'] = {r: verify_scene(rows, self.args.first) for r, rows in self.history.items()}
        assert self.evidence['scene']['host']['reunion_tick'] == self.evidence['scene']['guest']['reunion_tick']
        print('PASS real-menu Save succeeds after watched scene and safe reunion:', self.args.first, flush=True)
        self.until(lambda: all(p.poll() is not None for p in self.processes.values()), 15, 'normal fixture exit')
        assert all(p.returncode == 0 for p in self.processes.values())


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--seat-session', type=int, required=True)
    parser.add_argument('--case', choices=('death', 'scene'), required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not all(0 <= n <= 500 for n in (args.delay_ms, args.jitter_ms)):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    run = DeathSave(args) if args.case == 'death' else SceneSave(args)
    result = dict(passed=False, internet=not args.local, native_input=True, case=args.case,
                  first=args.first, delay_ms=args.delay_ms, jitter_ms=args.jitter_ms, evidence=run.evidence)
    try:
        run.run()
        result['passed'] = True
    except Exception as error:
        result['error'] = str(error)
        raise
    finally:
        result['build'] = run.build
        (run.folder / 'last-reports.json').write_text(json.dumps(run.reports))
        (run.folder / 'history.json').write_text(json.dumps(run.history))
        run.close()
        (run.folder / 'result.json').write_text(json.dumps(result, indent=2))
