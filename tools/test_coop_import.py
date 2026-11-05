"""Anode: host a solo save, rejoin, save, close both games, resume and recover backup.

Uses separate rendered processes/profiles and the real internet path by default.
Native receipts verify complete restoration before simulation; sampled network
state verifies both clients retain progress. No desktop input or image inspection.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import uuid

from test_coop_play import ROOT, read


class Pair:
    def __init__(self, args, folder, phase, resume=False):
        self.args, self.folder, self.phase, self.resume = args, folder, phase, resume
        self.invite = folder / (phase + '-invite.txt')
        self.processes, self.logs, self.commands, self.reports = {}, [], {}, {}
        self.history = {'host': [], 'guest': []}
        self.heartbeat = 0

    def command(self, role, **values):
        old = self.commands.get(role, {})
        self.commands[role] = dict(id=old.get('id', 0) + 1, heartbeat=1, epoch=1,
            goal=None, navigate=False, attack=False, capture=False, finish=False, **values)

    def pump(self, allow_exit=False):
        self.heartbeat += 1
        for role, command in self.commands.items():
            command['heartbeat'] = self.heartbeat
            path = self.folder / role / 'core-command.json'
            temporary = path.with_suffix('.tmp')
            temporary.write_text(json.dumps(command))
            try:
                temporary.replace(path)
            except PermissionError:
                pass
        for role, process in self.processes.items():
            code = process.poll()
            assert code is None or allow_exit and code == 0, (
                f'{role} exited ({code}); inspect {self.folder}/*{self.phase}*.log')
            report = read(self.folder / role / 'coop-live.json')
            if report and report['pid'] == process.pid and report['playing']:
                self.reports[role] = report
                frame = report['frame']
                row = dict(tick=frame['tick'], epoch=report['visit']['epoch'],
                    map=report['visit']['map'], entry=report['visit']['entry'],
                    health=[p['stats']['sanity'] for p in frame['players']],
                    will=[p['stats']['will'] for p in frame['players']],
                    owned=[p['stats']['owned'] for p in frame['players']],
                    collected=[p['stats']['collected'] for p in frame['players']],
                    feet=[p['player']['feet'] for p in frame['players']],
                    used=report['combat']['stage']['world']['used'],
                    generation=frame['checkpoint_generation'], completed=report['completed_visits'])
                if not self.history[role] or self.history[role][-1]['tick'] != row['tick']:
                    self.history[role].append(row)

    def until(self, condition, timeout=90, allow_exit=False):
        deadline = time.monotonic() + timeout
        while True:
            self.pump(allow_exit)
            if condition():
                return
            assert time.monotonic() < deadline, f'{self.phase} timed out: {self.folder}'
            time.sleep(.05)

    def start(self, role, rejoin=False):
        profile = self.folder / role
        profile.mkdir(exist_ok=True)
        self.command(role)
        self.reports.pop(role, None)
        log = (self.folder / f'{role}-{self.phase}{"-rejoin" if rejoin else ""}.log').open('w')
        self.logs.append(log)
        command = [str(self.args.exe.resolve()), '--data', str(self.args.data.resolve()),
            '--profile', str(profile), '--map', 'gvillage', '--no-audio',
            '--coop-host' if role == 'host' else '--coop-join', str(self.invite),
            '--coop-route-driver', str(profile / 'core-command.json'),
            '--coop-delay-ms', str(self.args.delay_ms), '--coop-jitter-ms', str(self.args.jitter_ms)]
        if role == 'host':
            command += ['--coop-resume'] if self.resume else ['--load', 'slot1']
            if self.args.local:
                command.append('--coop-local')
        self.processes[role] = subprocess.Popen(command, cwd=ROOT, stdout=log,
            stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)

    def connect(self):
        (self.folder / 'host' / 'route-resume.json').unlink(missing_ok=True)
        self.start('host')
        self.until(self.invite.exists, 150)
        self.start('guest')
        self.until(lambda: len(self.reports) == 2)
        receipt = read(self.folder / 'host' / 'route-resume.json')
        assert receipt and receipt['verified'], 'Missing exact pre-simulation restore receipt'

    def rejoin(self):
        process = self.processes.pop('guest')
        subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True, check=True)
        process.wait(timeout=10)
        time.sleep(.5)
        self.start('guest', rejoin=True)
        self.until(lambda: 'guest' in self.reports)

    def checkpoint(self):
        self.command('host', checkpoint=True)
        request = self.commands['host']['id']
        self.until(lambda: (self.reports['host']['route_driver'].get('checkpoint') or {}).get('command') == request)
        assert self.reports['host']['route_driver']['checkpoint']['error'] is None
        self.command('host')

    def verify(self, expected):
        start = {role: report['frame']['tick'] for role, report in self.reports.items()}
        self.until(lambda: all(self.reports[r]['frame']['tick'] >= tick + 120 for r, tick in start.items()))
        players = expected['frame']['players']
        for role, rows in self.history.items():
            assert rows and all(r['epoch'] == 1 and r['map'] == expected['visit']['map']
                and r['entry'] == expected['visit']['entry'] for r in rows), 'Wrong resumed visit'
            assert all(r['generation'] == expected['frame']['checkpoint_generation'] for r in rows), 'Unexpected wipe'
            assert all(r['health'] == [p['stats']['sanity'] for p in players] for r in rows), 'Health reset'
            assert all(r['will'] == [p['stats']['will'] for p in players] for r in rows), 'Will reset'
            assert all(r['owned'] == [p['stats']['owned'] for p in players] for r in rows), 'Earned weapons lost'
            assert all(r['collected'] == [p['stats']['collected'] for p in players] for r in rows), 'Pickup progress lost'
            assert all(r['used'] == expected['frame']['combat']['stage']['world']['used'] for r in rows), 'Door reset'
            assert all(sum((a-b)**2 for a, b in zip(r['feet'][0], players[0]['player']['feet'])) < 1
                for r in rows), 'Blue returned to entrance or moved unexpectedly'
        assert all(r['completed'] == 1 for r in self.history['host']), 'Cached history lost'
        assert not (self.folder / 'guest' / 'coop-checkpoint.json').exists(), 'Guest wrote host checkpoint'

    def finish(self):
        for role in self.processes:
            self.command(role)
            self.commands[role]['finish'] = True
        self.until(lambda: all(p.poll() is not None for p in self.processes.values()), 30, allow_exit=True)

    def crash_host(self):
        """Abrupt process loss after a committed save; no graceful shutdown hook."""
        saved = self.folder / 'host' / 'coop-checkpoint.json'
        backup = saved.with_suffix('.json.bak')
        before = {path: path.read_bytes() for path in (saved, backup)}
        started = time.monotonic()
        host = self.processes['host']
        subprocess.run(['taskkill', '/PID', str(host.pid), '/T', '/F'],
            capture_output=True, check=True)
        host.wait(timeout=10)
        # CLI diagnostic launches surface the normal session error as exit 1.
        # The menu's error panel and return path need a separate UI acceptance test.
        code = self.processes['guest'].wait(timeout=25)
        log = (self.folder / f'guest-{self.phase}-rejoin.log').read_text()
        expected = ('disconnected', 'stopped responding', 'connection cannot keep up',
            'Connection lost', 'session has ended', 'Player left')
        assert code == 1 and 'Viewer failed:' in log and any(s in log for s in expected), (
            f'Guest did not report host loss: {log[-500:]}')
        assert all(path.read_bytes() == raw for path, raw in before.items()), 'Host loss changed saved files'
        (self.folder / 'host-loss-result.json').write_text(json.dumps(dict(passed=True,
            seconds=round(time.monotonic()-started, 3), guest_exit=code,
            saved_sha256=hashlib.sha256(before[saved]).hexdigest()), indent=2))
        print('PASS abrupt host loss: guest detected disconnect; committed save and backup unchanged', flush=True)

    def close(self):
        for process in self.processes.values():
            if process.poll() is None:
                subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True)
                process.wait(timeout=10)
        for log in self.logs:
            log.close()
        self.invite.unlink(missing_ok=True)
        (self.folder / f'{self.phase}-history.json').write_text(json.dumps(self.history, indent=2))


def run(args):
    folder = ROOT / 'private' / 'coop' / ('import-test-' + uuid.uuid4().hex)
    (folder / 'host').mkdir(parents=True)
    (folder / 'guest' / 'saves').mkdir(parents=True)
    sentinel = folder / 'guest' / 'saves' / 'slot1.json'
    sentinel.write_text('Guest solo save must remain unchanged')
    print('Evidence:', folder, flush=True)
    with (folder / 'prepare.log').open('w') as log:
        subprocess.run([str(args.exe.resolve()), '--data', str(args.data.resolve()), '--no-audio',
            '--profile', str(folder / 'host'), '--coop-import-check'], cwd=ROOT,
            stdout=log, stderr=subprocess.STDOUT, timeout=120, check=True)
    storage = read(folder / 'host' / 'coop-write-check' / 'verified.json')
    assert storage and storage['passed'], 'Missing native storage-failure proof'
    if os.name == 'nt':
        assert storage['failed_replacements'] == storage['failed_write_wipes'] == 2
    source = folder / 'host' / 'saves' / 'slot1.json'
    unchanged = {source: source.read_bytes(), sentinel: sentinel.read_bytes()}
    expected = read(folder / 'host' / 'coop-import-expected.json')
    checkpoint = folder / 'host' / 'coop-checkpoint.json'
    for phase in ('import', 'resume', 'resume-after-loss', 'backup'):
        previous = checkpoint.read_bytes() if checkpoint.exists() else None
        if phase != 'import':
            expected = json.loads(previous)['payload']
        if phase == 'backup':
            expected = json.loads(checkpoint.with_suffix('.json.bak').read_bytes())['payload']
            checkpoint.write_bytes(b'{"interrupted checkpoint write')
        before = checkpoint.read_bytes() if checkpoint.exists() else None
        pair = Pair(args, folder, phase, resume=phase != 'import')
        try:
            pair.connect()
            pair.rejoin()
            pair.verify(expected)
            if phase == 'import':
                pair.checkpoint()
                pair.checkpoint()
            else:
                assert checkpoint.read_bytes() == before, 'Resume overwrote checkpoint'
            if phase == 'resume':
                pair.crash_host()
            else:
                pair.finish()
            assert all(path.read_bytes() == raw for path, raw in unchanged.items()), 'Solo save changed'
            print('PASS', phase, ': exact native restore, two network views, guest rejoin, personal inventory and solo-save isolation', flush=True)
            (folder / f'{phase}-result.json').write_text(json.dumps(dict(passed=True,
                internet=not args.local, source_sha256=hashlib.sha256(unchanged[source]).hexdigest()), indent=2))
        finally:
            pair.close()
            if phase == 'backup':
                checkpoint.write_bytes(previous)
    return folder


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not all(0 <= n <= 500 for n in (args.delay_ms, args.jitter_ms)):
        parser.error('Network delays must be between 0 and 500 ms')
    run(args)
