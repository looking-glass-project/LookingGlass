"""Two real clients walk into the authored school theatre and reunite afterward.

This starts near that trigger; it is not a school traversal/completion test.
Run in Anode. Internet transport is the default; --local is for faster diagnosis.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import uuid

from test_coop_play import ROOT, read, distance


def verify(history, first):
    assert history, 'No playable state observed'
    actor = 0 if first == 'blue' else 1
    partner = 1 - actor
    active = [r for r in history if r['owner'] is not None]
    assert len(active) > 20, 'Original scene was not observed over time'
    assert all(r['owner'] == first.title() for r in active), 'Wrong scene owner'
    waiting = active[0]['players'][partner]['feet']
    assert all(distance(r['players'][partner]['feet'], waiting) < .01 for r in active), 'Waiting partner moved during scene'
    assert all(not r['visible_players'][partner] for r in active), 'Waiting partner was rendered during scene'
    for before, after in zip(active, active[1:]):
        elapsed = (after['tick'] - before['tick']) / 120
        assert elapsed > 0 and abs(after['scene_time'] - before['scene_time'] - elapsed) < .003, 'Scene clock restarted or skipped time'
    assert max(distance(r['camera'], active[0]['camera']) for r in active) > 100, 'Original scene camera did not move'
    reunited = [r for r in history if r['owner'] is None and all(b['warped_at'] > 0 for b in r['players'])]
    assert reunited, 'Both players did not reunite after the scene'
    first_reunion = reunited[0]
    players = first_reunion['players']
    assert players[0]['warped_at'] == players[1]['warped_at']
    assert all(b['spawned_at'] == 0 and b['health'] > 0 for b in players), 'Reunion was misreported as death/respawn'
    separation = distance(players[0]['feet'], players[1]['feet'])
    assert 40 <= separation <= 110, f'Reunion separation {separation}'
    assert all(r['checkpoint_generation'] == 0 for r in history), 'Scene unexpectedly wiped'
    return {'reunion_tick': players[0]['warped_at'], 'separation': separation,
            'scene_first_tick': active[0]['tick'], 'scene_last_tick': active[-1]['tick']}


def verify_scene_rejoin(before, after, first, old_pid):
    actor = 0 if first == 'blue' else 1
    partner = 1 - actor
    assert after['pid'] != old_pid, 'Guest process did not restart'
    assert after['build'] == before['build'] and after['visit'] == before['visit'], 'Rejoin changed build or visit'
    old, new = before['frame'], after['frame']
    elapsed = new['tick'] - old['tick']
    assert elapsed >= 0, 'Rejoin moved the clock backwards'
    assert new['checkpoint_generation'] == old['checkpoint_generation'] == 0, 'Rejoin silently wiped'
    old_stage, new_stage = (r['combat']['stage'] for r in (before, after))
    assert new_stage['owner'] == old_stage['owner'] == first.title(), 'Rejoin changed scene owner'
    old_scene, new_scene = (s['world']['school']['first_cinema'] for s in (old_stage, new_stage))
    assert new_scene['beat'] == old_scene['beat'] == 'Theatre', 'Rejoin ended or replaced the scene'
    assert abs(new_scene['time'] - old_scene['time'] - elapsed / 120) < .003, 'Rejoin reset or skipped scene time'
    assert {k:v for k,v in new_scene.items() if k != 'time'} == {k:v for k,v in old_scene.items() if k != 'time'}, 'Rejoin changed scene cues'
    old_story, new_story = old_stage['story'], new_stage['story']
    assert {k:v for k,v in new_story.items() if k != 'current'} == {k:v for k,v in old_story.items() if k != 'current'}, 'Rejoin changed story progress'
    old_line, new_line = old_story['current'], new_story['current']
    # Both chosen restart points avoid a line boundary. The original theatre
    # holds dialogue until its camera introduction reaches 22.8 seconds.
    if old_line is None:
        assert old_scene['time'] < 22.8 and new_line is None, 'Rejoin started dialogue during its camera introduction'
    else:
        assert new_line is not None and new_line[:2] == old_line[:2], 'Rejoin changed dialogue line'
        expected = elapsed / 120 if old_scene['time'] >= 22.8 else 0
        assert abs(new_line[2] - old_line[2] - expected) < .003, 'Rejoin restarted or advanced dialogue incorrectly'
    assert new_stage['contacts'] == old_stage['contacts'], 'Rejoin changed pending scene contacts'
    for previous, current in zip(old['players'], new['players']):
        assert current['stats'] == previous['stats'], 'Rejoin changed personal resources'
        assert current['spawned_at'] == previous['spawned_at'] == 0, 'Rejoin respawned a body'
        assert current['warped_at'] == previous['warped_at'] == 0, 'Rejoin prematurely reunited players'
    assert new['players'][partner]['player'] == old['players'][partner]['player'], 'Rejoin moved the waiting partner'
    assert not after['presentation']['visible_players'][partner], 'Rejoined guest rendered the waiting partner'
    return dict(before_tick=old['tick'], after_tick=new['tick'], scene_before=old_scene['time'],
        scene_after=new_scene['time'], dialogue_before=old_line[2] if old_line else None,
        dialogue_after=new_line[2] if new_line else None,
        owner=first, old_guest_pid=old_pid, new_guest_pid=after['pid'])


def run(args):
    folder = ROOT / 'private' / 'coop' / ('scene-test-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    print('Evidence:', folder, flush=True)
    source = Path(__file__).read_bytes()
    (folder / 'fixture-source.py').write_bytes(source)
    invite = folder / 'invite.txt'
    history = {role: [] for role in ('host', 'guest')}
    processes, logs = [], []
    active, log_paths, latest = {}, {}, {}
    rejoined = None
    build = None
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    try:
        def start(role, reconnecting=False):
            profile = folder / role
            profile.mkdir(exist_ok=reconnecting)
            log_paths[role] = folder / (role + ('-rejoin' if reconnecting else '') + '.log')
            log = log_paths[role].open('w')
            logs.append(log)
            command = [str(args.exe.resolve()), '--data', str(args.data.resolve()), '--map', 'skool1', '--no-audio',
                       '--profile', str(profile), '--coop-host' if role == 'host' else '--coop-join', str(invite),
                       '--coop-scene-check', args.first, '--coop-delay-ms', str(args.delay_ms), '--coop-jitter-ms', str(args.jitter_ms)]
            if role == 'host' and args.local:
                command.append('--coop-local')
            p = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, creationflags=flags)
            processes.append(p)
            active[role] = p
            return p
        host = start('host')
        deadline = time.monotonic() + 150
        while not invite.exists():
            assert host.poll() is None, (folder / 'host.log').read_text()
            assert time.monotonic() < deadline, 'Invite timeout'
            time.sleep(.1)
        guest = start('guest')
        deadline = time.monotonic() + 240
        while any(p.poll() is None for p in active.values()):
            assert time.monotonic() < deadline, 'Scene timeout'
            for role, process in active.items():
                assert process.poll() in (None, 0), log_paths[role].read_text()
            for role in history:
                report = read(folder / role / 'coop-live.json')
                if not report or report['pid'] != active[role].pid or not report['playing']:
                    continue
                assert build is None or report['build'] == build, 'Clients used different builds'
                build = report['build']
                latest[role] = report
                frame, stage = report['frame'], report['combat']['stage']
                if history[role] and history[role][-1]['tick'] == frame['tick']:
                    continue
                history[role].append({'tick': frame['tick'], 'checkpoint_generation': frame['checkpoint_generation'],
                    'owner': stage['owner'], 'camera': report['camera'], 'target': report['camera_target'],
                    'scene_time': (stage['world']['school'].get('first_cinema') or {}).get('time'),
                    'visible_players': report['presentation']['visible_players'],
                    'players': [{'feet': p['player']['feet'], 'health': p['stats']['sanity'],
                        'spawned_at': p['spawned_at'], 'warped_at': p['warped_at']} for p in frame['players']]})
            if 'host' in latest:
                assert latest['host']['frame']['tick'] < 1800 or any(r['owner'] is not None for r in history['host']), 'Requested player did not reach the staged scene trigger'
            if args.rejoin and rejoined is None and 'host' in latest:
                stage = latest['host']['combat']['stage']
                scene = stage['world']['school'].get('first_cinema')
                if stage['owner'] == args.first.title() and scene and scene['beat'] == 'Theatre' and scene['time'] >= args.rejoin_at:
                    old_pid = guest.pid
                    guest.kill()
                    guest.wait(timeout=10)
                    disconnect_deadline = time.monotonic() + 15
                    while True:
                        before = read(folder / 'host' / 'coop-live.json')
                        assert host.poll() is None, 'Host ended when the guest disconnected'
                        if before and before['pid'] == host.pid and not before['playing']:
                            break
                        assert time.monotonic() < disconnect_deadline, 'Host did not observe guest loss'
                        time.sleep(.05)
                    time.sleep(.4)
                    held = read(folder / 'host' / 'coop-live.json')
                    assert held and not held['playing'] and held['frame'] == before['frame'], 'Disconnected host advanced player state'
                    assert held['combat'] == before['combat'], 'Disconnected host advanced scene/world state'
                    guest = start('guest', reconnecting=True)
                    join_deadline = time.monotonic() + 60
                    while True:
                        after = read(folder / 'guest' / 'coop-live.json')
                        assert guest.poll() is None, 'Replacement guest failed: ' + log_paths['guest'].read_text()
                        if after and after['pid'] == guest.pid and after['playing']:
                            (folder / 'rejoin.json').write_text(json.dumps(dict(passed=False, before=before, held=held, after=after), indent=2))
                            rejoined = verify_scene_rejoin(before, after, args.first, old_pid)
                            break
                        assert time.monotonic() < join_deadline, 'Guest rejoin timeout'
                        time.sleep(.05)
                    (folder / 'rejoin.json').write_text(json.dumps(dict(passed=True,
                        evidence=rejoined, before=before, held=held, after=after), indent=2))
                    print('PASS fresh guest rejoined during scene; owner, clocks, resources and hidden partner preserved', flush=True)
            time.sleep(.05)
        assert not args.rejoin or rejoined is not None, 'Scene reconnect was not exercised'
        for role, process in active.items():
            assert process.returncode == 0, log_paths[role].read_text()
            for capture in ('coop-scene.png', 'coop-screenshot.png'):
                assert (folder / role / capture).exists(), f'Missing {role} {capture}'
        results = {role: verify(reports, args.first) for role, reports in history.items()}
        assert results['host']['reunion_tick'] == results['guest']['reunion_tick'], 'Guest did not see the authoritative reunion'
        assert abs(results['host']['separation'] - results['guest']['separation']) < .1
        (folder / 'result.json').write_text(json.dumps({'passed': True, 'first': args.first,
            'build': build, 'fixture_sha256': hashlib.sha256(source).hexdigest(),
            'evidence': {'scene_rejoin': rejoined, 'scene_reunion': results},
            'rejoin': rejoined, 'rejoin_at': args.rejoin_at if args.rejoin else None,
            'internet': not args.local, 'delay_ms': args.delay_ms, 'jitter_ms': args.jitter_ms,
            'results': results, 'scope': 'school theatre fixture only; campaign not validated'}, indent=2))
        print('PASS two rendered clients: first-arrival theatre ownership, single scene camera, frozen partner, safe reunion.')
        return folder
    except Exception as error:
        (folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (folder / 'last-reports.json').write_text(json.dumps(latest, indent=2))
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
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true', help='Restart the guest during the original cutscene')
    parser.add_argument('--rejoin-at', type=int, choices=(6, 30), default=6,
        help='Scene seconds before restarting: camera introduction (6) or dialogue (30)')
    parser.add_argument('--delay-ms', type=int, default=0)
    parser.add_argument('--jitter-ms', type=int, default=0)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Network test delays must be between 0 and 500 milliseconds')
    run(args)
