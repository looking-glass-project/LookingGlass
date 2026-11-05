"""Anode: travel, save, close both clients, resume, wipe, and recover a backup.

Uses separate real clients and ordinary host-owned checkpoints. The fixture
seeds distinct resources after returning to the first school visit, then tests
restoration exactly before the resumed simulation and at a later team wipe.
This is a focused persistence fixture, not a campaign playthrough.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
from test_coop_play import ROOT, read
import test_coop_travel


def resume(args, folder, backup):
    suffix = 'backup' if backup else 'resume'
    invite = folder / ('invite-' + suffix + '.txt')
    saved = folder / 'host' / 'coop-checkpoint.json'
    backup_path = saved.with_suffix('.json.bak')
    original = saved.read_bytes()
    previous = backup_path.read_bytes()
    expected = json.loads(previous if backup else original)['payload']
    if backup:
        saved.write_bytes(b'{"interrupted checkpoint write')
    expected_primary = saved.read_bytes()
    verified = folder / 'host' / 'coop-checkpoint-verified.json'
    verified.unlink(missing_ok=True)
    processes, logs = {}, []
    history = {'host': [], 'guest': []}
    sentinel = {}
    try:
        for role in ('host', 'guest'):
            profile = folder / role
            single_player = profile / 'saves' / 'quick.json'
            single_player.parent.mkdir(exist_ok=True)
            single_player.write_text('Single-player save sentinel: ' + role)
            sentinel[role] = (single_player, single_player.read_bytes())
            (profile / 'coop-screenshot.png').unlink(missing_ok=True)
            log = (folder / (role + '-' + suffix + '.log')).open('w')
            logs.append(log)
            command = [str(args.exe.resolve()), '--data', str(args.data.resolve()), '--map', 'gvillage',
                '--no-audio', '--profile', str(profile), '--coop-host' if role == 'host' else '--coop-join',
                str(invite), '--coop-checkpoint-check', '--coop-delay-ms', str(args.delay_ms),
                '--coop-jitter-ms', str(args.jitter_ms)]
            if role == 'host':
                command += ['--coop-resume'] + (['--coop-local'] if args.local else [])
            process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
                creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
            processes[role] = process
            if role == 'host':
                deadline = time.monotonic() + 150
                while not invite.exists():
                    assert process.poll() is None, f'Host failed: {folder / (role + "-" + suffix + ".log")}'
                    assert time.monotonic() < deadline, 'Resume invite timeout'
                    time.sleep(.1)
        deadline = time.monotonic() + 90
        while any(p.poll() is None for p in processes.values()):
            assert time.monotonic() < deadline, 'Resume clients did not finish'
            for role, process in processes.items():
                assert process.poll() in (None, 0), f'{role} failed: inspect {folder / (role + "-" + suffix + ".log")}'
                report = read(folder / role / 'coop-live.json')
                if report and report['pid'] == process.pid and report['playing']:
                    frame = report['frame']
                    if not history[role] or frame['tick'] != history[role][-1]['tick']:
                        history[role].append({'tick': frame['tick'], 'epoch': report['visit']['epoch'],
                            'map': report['visit']['map'], 'entry': report['visit']['entry'],
                            'health': [b['stats']['sanity'] for b in frame['players']],
                            'will': [b['stats']['will'] for b in frame['players']],
                            'generation': frame['checkpoint_generation'],
                            'used': report['combat']['stage']['world']['used'],
                            'completed': report['completed_visits']})
            time.sleep(.05)
        assert all(p.returncode == 0 for p in processes.values()), f'Resume failed: inspect {folder}'
        proof = read(verified)
        assert proof and proof['passed'] and proof['recovered_backup'] == backup, 'Missing exact restoration proof'
        for role, rows in history.items():
            assert rows and any(r['generation'] == expected['frame']['checkpoint_generation'] + 1 for r in rows), f'{role} missed wipe'
            assert all(r['epoch'] == 1 and r['map'] == expected['visit']['map'] and r['entry'] == expected['visit']['entry'] for r in rows)
            assert all(r['health'] == [83, 47] for r in rows), f'{role} lost personal health'
            assert all(r['used'] == expected['frame']['combat']['stage']['world']['used'] for r in rows), 'Opened door was lost'
            assert (folder / role / 'coop-screenshot.png').exists(), f'{role} did not capture'
            assert sentinel[role][0].read_bytes() == sentinel[role][1], 'Single-player save changed'
        assert all(r['completed'] == 2 for r in history['host']), 'Cached visits did not survive restart'
        assert saved.read_bytes() == expected_primary and backup_path.read_bytes() == previous, 'Resume or dead-team save changed the files'
        assert not (folder / 'guest' / 'coop-checkpoint.json').exists(), 'Guest wrote a host checkpoint'
        (folder / (suffix + '-result.json')).write_text(json.dumps({'passed': True, 'backup': backup,
            'internet': not args.local, 'saved_sha256': hashlib.sha256(original).hexdigest()}, indent=2))
        print('PASS', suffix, ': exact full-state restore, team wipe, both network views and separate single-player saves.', flush=True)
    finally:
        (folder / (suffix + '-history.json')).write_text(json.dumps(history, indent=2))
        for process in processes.values():
            if process.poll() is None:
                subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True)
                process.wait(timeout=10)
        for log in logs:
            log.close()
        invite.unlink(missing_ok=True)
        if backup:
            saved.write_bytes(original)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Network test delays must be between 0 and 500 milliseconds')
    args.reconnect = True
    args.checkpoint = True
    folder = test_coop_travel.run(args)
    resume(args, folder, backup=False)
    resume(args, folder, backup=True)
