"""Two rendered clients open a door, change visits and return to the cached world.

Run in Anode. The --coop-travel-check fixture requests ordinary load barriers;
it deliberately does not claim to traverse the quest-gated school exits. With
--reconnect, close/restart the guest after arrival at the second visit. With
--reconnect-at outbound/return, interrupt the actual loading barrier instead.
Both clients capture their final views and exit themselves; desktop input is
unnecessary.
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
from coop_travel_rejoin import LoadingReconnect


def run(args):
    folder = ROOT / 'private' / 'coop' / ('travel-test-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    print('Evidence:', folder, flush=True)
    invite = folder / 'invite.txt'
    processes, logs = [], []
    active, history = {}, {'host': [], 'guest': []}
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    disconnected = False
    reconnect_at = getattr(args, 'reconnect_at', 'arrival')
    loading = LoadingReconnect(folder, 2 if reconnect_at == 'outbound' else 3) if reconnect_at != 'arrival' else None
    sources = {}
    for name in ('test_coop_travel.py', 'coop_travel_rejoin.py'):
        source = (ROOT / 'tools' / name).read_bytes()
        (folder / name).write_bytes(source)
        sources[name] = hashlib.sha256(source).hexdigest()
    build = None
    try:
        def start(role, replacement=False):
            profile = folder / role
            profile.mkdir(exist_ok=True)
            log = (folder / (role + ('-rejoin' if replacement else '') + '.log')).open('w')
            logs.append(log)
            command = [str(args.exe.resolve()), '--data', str(args.data.resolve()), '--map', 'skool1', '--no-audio',
                       '--profile', str(profile), '--coop-host' if role == 'host' else '--coop-join', str(invite),
                       '--coop-travel-check', '--coop-delay-ms', str(args.delay_ms), '--coop-jitter-ms', str(args.jitter_ms)]
            if role == 'host' and args.local:
                command.append('--coop-local')
            if getattr(args, 'checkpoint', False):
                command.append('--coop-checkpoint-check')
            process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, creationflags=flags)
            processes.append(process)
            active[role] = process
            return process
        host = start('host')
        deadline = time.monotonic() + 150
        while not invite.exists():
            assert host.poll() is None, (folder / 'host.log').read_text()
            assert time.monotonic() < deadline, 'Invite timeout'
            time.sleep(.1)
        start('guest')
        deadline = time.monotonic() + 180
        while True:
            assert time.monotonic() < deadline, 'Travel timeout'
            for role, process in list(active.items()):
                assert process.poll() is None, f'{role} exited: inspect {folder}'
                report = read(folder / role / 'coop-live.json')
                if not report or report['pid'] != process.pid:
                    continue
                frame, visit = report['frame'], report['visit']
                assert build is None or report['build'] == build, 'Clients used different builds'
                build = report['build']
                if role == 'host' and loading and loading.observe(report, active, start):
                    disconnected = True
                    break
                key = (visit['epoch'], frame['tick'], report['playing'])
                if history[role] and tuple(history[role][-1]['key']) == key:
                    continue
                shared = report['combat']['stage']['world']
                history[role].append({'key': key, 'entry': visit['entry'], 'used': shared['used'],
                    'health': [b['stats']['sanity'] for b in frame['players']],
                    'weapons': [b['stats']['owned'] for b in frame['players']],
                    'collected': [b['stats']['collected'] for b in frame['players']],
                    'generation': frame['checkpoint_generation'], 'completed': report['completed_visits'],
                    'spawned_at': [b['spawned_at'] for b in frame['players']],
                    'warped_at': [b['warped_at'] for b in frame['players']]})
                if role == 'host' and visit['epoch'] == 2 and args.reconnect and not loading and not disconnected:
                    guest = active['guest']
                    guest.kill()
                    guest.wait(timeout=10)
                    disconnected = True
                    # Wait for the production listener to observe the lost peer;
                    # the existing invite then restores the same authoritative Red.
                    time.sleep(.5)
                    start('guest', replacement=True)
                    break
            if all(rows and rows[-1]['key'][0] == 3 and rows[-1]['key'][2] for rows in history.values()):
                starts = [next(r['key'][1] for r in rows if r['key'][0] == 3 and r['key'][2]) for rows in history.values()]
                if all(rows[-1]['key'][1] - first >= 360 for rows, first in zip(history.values(), starts)):
                    break
            time.sleep(.05)
        for role, rows in history.items():
            for epoch in (1, 2, 3):
                assert any(r['key'][0] == epoch and r['key'][2] for r in rows), f'{role} did not play visit {epoch}'
            first = [r for r in rows if r['key'][0] == 1 and r['key'][2]]
            returned = [r for r in rows if r['key'][0] == 3 and r['key'][2]]
            assert max(r['used'] for r in first) > 0, 'Door was never opened through player input'
            assert all(r['used'] == first[-1]['used'] for r in returned), 'Revisit lost the shared door state'
            assert all(r['generation'] == 0 and all(s == 0 for s in r['spawned_at']) for r in rows), 'Travel reported a wipe or respawn'
            if getattr(args, 'checkpoint', False):
                assert returned[-1]['health'] == [83, 47], 'Checkpoint fixture did not seed distinct resources'
            else:
                assert all(r['health'] == first[0]['health'] for r in returned), 'Travel changed personal health'
            assert all(all(new >= old for new, old in zip(r['weapons'][i], first[-1]['weapons'][i]))
                       and set(first[-1]['collected'][i]).issubset(r['collected'][i])
                       for r in returned for i in range(2)), 'Travel lost a personal weapon or pickup entitlement'
            assert all(r['warped_at'][0] == r['warped_at'][1] and r['warped_at'][0] > 0 for r in returned), 'Players did not relocate together'
        assert history['host'][-1]['completed'] == 2, 'First and return visit histories were conflated'
        assert 'cached shared world and NPC state restored exactly' in (folder / 'host.log').read_text(), 'No full restore evidence'
        assert not (args.reconnect or loading) or disconnected
        (folder / 'ready.json').write_text(json.dumps({'reconnected': disconnected, 'passed': True}))
        print('Travel validated. Waiting for automatic captures and clean client exits.', flush=True)
        deadline = time.monotonic() + 30
        while any(p.poll() is None for p in active.values()):
            assert time.monotonic() < deadline, 'Travel clients did not finish automatically'
            time.sleep(.1)
        assert all(p.returncode == 0 for p in active.values()), 'Final clients did not exit cleanly'
        assert all((folder / role / 'coop-screenshot.png').exists() for role in active), 'A client did not capture its final view'
        (folder / 'result.json').write_text(json.dumps({'passed': True, 'internet': not args.local,
            'build': build, 'fixture_sha256': sources, 'reconnect_at': reconnect_at,
            'evidence': {'loading_rejoin': loading.evidence['summary']} if loading else {},
            'reconnected': disconnected, 'delay_ms': args.delay_ms, 'jitter_ms': args.jitter_ms,
            'scope': 'staged travel/cache/epoch fixture; quest-gated exits and campaign completion not verified'}, indent=2))
        print('PASS shared visit history, return-visit separation, paired transitions, restored world and clean final exits.', flush=True)
    finally:
        (folder / 'history.json').write_text(json.dumps(history, indent=2))
        try:
            if loading:
                loading.close()
        finally:
            for p in processes:
                if p.poll() is None:
                    # The host owns its tunnel helper too. Keep failure cleanup
                    # scoped to this still-live child and its descendants.
                    subprocess.run(['taskkill', '/PID', str(p.pid), '/T', '/F'], capture_output=True)
                    p.wait(timeout=10)
        for log in logs:
            log.close()
        invite.unlink(missing_ok=True)
    return folder


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--reconnect', action='store_true')
    parser.add_argument('--reconnect-at', choices=('arrival', 'outbound', 'return'), default='arrival')
    parser.add_argument('--delay-ms', type=int, default=0)
    parser.add_argument('--jitter-ms', type=int, default=0)
    args = parser.parse_args()
    if args.reconnect_at != 'arrival' and os.name != 'nt':
        parser.error('Loading-barrier interruption requires Windows process suspension')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Network test delays must be between 0 and 500 milliseconds')
    run(args)
