"""Stage two independent clients on a school lift using ordinary Host/Join.

Run inside Anode; no synthetic world updates or position messages are injected.
The real contact starts the lift and both clients remain idle through a full
cycle. Once verified, capture and close both windows to finish. Boarding arrival
orders and obstructions have separate native checks under --coop-world-check.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
import uuid

from test_coop_play import ROOT, read, distance

LIFTS = {
    'spiral': ('library_spiral_lift', '144 4360 -263.96', -264., 64.),
    'elevator': ('elevator2', '-276 4360 400.04', 400., 696.),
}


def verify(rows, lift):
    name, _, bottom, top = LIFTS[lift]
    assert len(rows) > 80, 'Insufficient lift history'
    assert rows[-1]['tick'] - rows[0]['tick'] >= 3000, 'A full cycle was not sampled'
    assert all(r['owner'] is None and r['generation'] == 0 for r in rows), 'Unexpected scene or wipe'
    high = [max(r['feet'][i][2] for r in rows) for i in range(2)]
    assert all(h >= top - 1 for h in high), f'Both players did not reach the top: {high}'
    peak = next(i for i, r in enumerate(rows) if min(p[2] for p in r['feet']) >= top - 1)
    assert any(max(p[2] for p in r['feet']) <= bottom + 1 for r in rows[peak + 1:]), 'Both riders did not return'
    initial_offsets = [p[2] - rows[0]['lift'][2] for p in rows[0]['feet']]
    for row in rows:
        assert all(h > 0 for h in row['health']), 'Rider died'
        assert 40 <= distance(*row['feet']) <= 80, 'Riders drifted apart or overlapped'
        assert all(abs(row['feet'][i][2] - row['lift'][2] - initial_offsets[i]) < .15 for i in range(2)), 'Body and lift snapshots are out of sync'
    return {'name': name, 'highest': high, 'samples': len(rows), 'last_tick': rows[-1]['tick']}


def run(args):
    folder = ROOT / 'private' / 'coop' / ('lift-test-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    invite = folder / 'invite.txt'
    history = {role: [] for role in ('host', 'guest')}
    processes, logs = [], []
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    try:
        def start(role):
            profile = folder / role
            profile.mkdir()
            log = (folder / (role + '.log')).open('w')
            logs.append(log)
            command = [str(args.exe.resolve()), '--data', str(args.data.resolve()), '--map', 'skool1', '--no-audio',
                       '--profile', str(profile), '--start-at', LIFTS[args.lift][1],
                       '--coop-host' if role == 'host' else '--coop-join', str(invite),
                       '--coop-delay-ms', str(args.delay_ms), '--coop-jitter-ms', str(args.jitter_ms)]
            if role == 'host' and args.local:
                command.append('--coop-local')
            process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, creationflags=flags)
            processes.append(process)
            return process
        host = start('host')
        deadline = time.monotonic() + 150
        while not invite.exists():
            assert host.poll() is None, (folder / 'host.log').read_text()
            assert time.monotonic() < deadline, 'Invite timeout'
            time.sleep(.1)
        start('guest')
        print('Evidence:', folder, flush=True)
        deadline = time.monotonic() + 180
        while not all(rows and rows[-1]['tick'] - rows[0]['tick'] >= 3120 for rows in history.values()):
            assert time.monotonic() < deadline, 'Lift timeout'
            for role, process in zip(history, processes):
                assert process.poll() is None, (folder / (role + '.log')).read_text()
                report = read(folder / role / 'coop-live.json')
                if not report or not report['playing']:
                    continue
                frame, stage = report['frame'], report['combat']['stage']
                if history[role] and history[role][-1]['tick'] == frame['tick']:
                    continue
                lift = next(o for o in stage['world']['school']['objects'] if o['name'] == LIFTS[args.lift][0])
                history[role].append({'tick': frame['tick'], 'generation': frame['checkpoint_generation'], 'owner': stage['owner'],
                    'lift': lift['pose']['origin'], 'feet': [p['player']['feet'] for p in frame['players']],
                    'health': [p['stats']['sanity'] for p in frame['players']]})
            time.sleep(.05)
        results = {role: verify(rows, args.lift) for role, rows in history.items()}
        assert all(abs(a - b) < .1 for a, b in zip(results['host']['highest'], results['guest']['highest'])), 'Guest missed lift top'
        (folder / 'ready.json').write_text(json.dumps(results, indent=2))
        print('Rides validated. Capture and close both game windows to finish.', flush=True)
        deadline = time.monotonic() + 90
        while any(p.poll() is None for p in processes):
            assert time.monotonic() < deadline, 'Close both windows after inspecting the lift'
            time.sleep(.1)
        assert all(p.returncode == 0 for p in processes), 'Game did not exit cleanly'
        for role in history:
            assert (folder / role / 'coop-screenshot.png').exists(), f'Capture {role} with F12 before closing'
        (folder / 'result.json').write_text(json.dumps({'passed': True, 'lift': args.lift, 'internet': not args.local,
            'delay_ms': args.delay_ms, 'jitter_ms': args.jitter_ms, 'results': results,
            'scope': 'staged two-rider lift cycle only; not a campaign traversal'}, indent=2))
        print('PASS two rendered clients: shared lift, two riders, full cycle, synchronized world/body snapshots and clean exits.', flush=True)
    finally:
        (folder / 'history.json').write_text(json.dumps(history, indent=2))
        for p in processes:
            if p.poll() is None:
                subprocess.run(['taskkill', '/PID', str(p.pid), '/T', '/F'], capture_output=True)
                p.wait(timeout=10)
        for log in logs:
            log.close()
        invite.unlink(missing_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--lift', choices=LIFTS, default='elevator')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=0)
    parser.add_argument('--jitter-ms', type=int, default=0)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Network test delays must be between 0 and 500 milliseconds')
    run(args)
