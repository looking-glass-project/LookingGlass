"""Run two rendered co-op movement/combat clients in a desktop session.

This is a development gameplay slice, not a campaign completion check. Run in
Anode to leave the user's desktop free. Public internet hosting is the default.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError):
        return None


def distance(a, b):
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))


def recovery_report(history, role, initial, first_victim):
    """Verify host-clock timing using observed deaths and authoritative spawn ticks."""
    last_death = [None, None]
    respawns = {}
    wipe = None
    local = 0 if role == 'host' else 1
    followed = []
    for report in history:
        tick = report['tick']
        players = report['players']
        if report['checkpoint_generation']:
            if wipe is None:
                assert report['checkpoint_generation'] == 1
                assert all(p['health'] == 100 for p in players), 'Checkpoint did not restore both players health'
                assert players[0]['spawned_at'] == players[1]['spawned_at'] > 0
                assert report['lives']['states'] == ['Alive', 'Alive']
                for body, old in zip(players, initial['frame']['players']):
                    assert distance(body['feet'], old['player']['feet']) < 8, 'Checkpoint did not restore entrance positions'
                old_guard = initial['combat']['npcs']['actors'][0]['guard']
                assert report['guard']['health'] == old_guard['health']
                assert distance(report['guard']['feet'], old_guard['feet']) < 5, 'Enemy position was not restored with the players'
                assert report['guard']['time'] < 1, 'Enemy action was not restarted'
                wipe = players[0]['spawned_at']
            continue
        for i, (body, life) in enumerate(zip(players, report['lives']['states'])):
            if body['health'] <= 0:
                assert isinstance(life, dict) and 'Spectating' in life, 'Dead player did not enter the countdown'
                remaining = life['Spectating']['ticks_left']
                assert 0 < remaining <= 1200
                death_tick = tick - (1200 - remaining)
                assert last_death[i] in (None, death_tick), 'Death countdown restarted or skipped time'
                last_death[i] = death_tick
                if i == local:
                    partner = 1 - i
                    assert report['camera_player'] == ('Blue', 'Red')[partner], 'Dead player did not spectate the survivor'
                    subject = report['camera_subject_feet']
                    assert distance(report['camera_target'][:2], subject[:2]) < 1, 'Spectator camera did not follow its rendered subject'
                    assert distance(subject, players[partner]['feet']) < 24, 'Spectator body drifted away from authoritative survivor'
                    if local == first_victim:
                        followed.append(report['camera_target'])
            elif body['spawned_at'] and (i, body['spawned_at']) not in respawns:
                assert last_death[i] is not None, 'Respawn without an observed death'
                assert body['spawned_at'] - last_death[i] == 1200, 'Respawn did not wait exactly ten host simulation seconds'
                assert body['health'] == 25, f"Respawn health was {body['health']}, expected 25"
                assert life == 'Alive', 'Respawn health and life state were not published together'
                separation = distance(body['feet'], players[1 - i]['feet'])
                assert 40 <= separation <= 110, f'Unsafe or distant respawn: {separation}'
                if i == local:
                    assert report['camera_player'] == ('Blue', 'Red')[i], 'Camera did not return to the respawned player'
                    assert distance(report['camera_target'][:2], body['feet'][:2]) < 1, 'Respawn camera blended from old body'
                respawns[(i, body['spawned_at'])] = {'player': i, 'death_tick': last_death[i],
                    'spawn_tick': body['spawned_at'], 'health': body['health'], 'separation': separation}
                last_death[i] = None
    assert respawns, 'No ten-second respawn was observed before the wipe'
    assert next(iter(respawns))[0] == first_victim, 'Encounter killed the wrong first victim'
    if local == first_victim:
        assert followed and max(distance(p, followed[0]) for p in followed) > 10, 'Spectator camera never followed a moving survivor'
    assert wipe is not None, 'No shared checkpoint restore was observed'
    return {'respawns': list(respawns.values()), 'checkpoint_tick': wipe}


def verify_countdown_rejoin(before, after, victim, old_pid, *, expected_generation=0):
    """A fresh guest must inherit the running life cycle, not create a new body."""
    from test_coop_village import verify_rejoin_stats
    assert after['pid'] != old_pid, 'Guest process did not restart'
    assert before['build'] == after['build'] and before['visit'] == after['visit'], 'Rejoin changed build or visit'
    old, new = before['frame'], after['frame']
    elapsed = new['tick'] - old['tick']
    assert elapsed >= 0, 'Rejoin moved the clock backwards'
    assert new['checkpoint_generation'] == old['checkpoint_generation'] == expected_generation, 'Rejoin silently wiped'
    life = lambda frame: frame['lives']['states'][victim]
    assert isinstance(life(old), dict) and 'Spectating' in life(old)
    assert isinstance(life(new), dict) and 'Spectating' in life(new), 'Guest did not rejoin during countdown'
    before_left, after_left = (life(f)['Spectating']['ticks_left'] for f in (old, new))
    assert 0 < after_left <= before_left <= 1200
    assert before_left - after_left == elapsed, 'Rejoin reset or skipped the death countdown'
    for previous, current in zip(old['players'], new['players']):
        # The living partner can take ordinary enemy damage after play resumes.
        # Inventory/Will and body identity must still survive the guest restart.
        stats = dict(current['stats'], sanity=previous['stats']['sanity'])
        verify_rejoin_stats(previous['stats'], stats, elapsed)
        assert current['spawned_at'] == previous['spawned_at'] and current['warped_at'] == previous['warped_at'], 'Rejoin replaced a body'
    assert new['players'][victim]['stats']['sanity'] == 0
    assert 0 < new['players'][1-victim]['stats']['sanity'] <= old['players'][1-victim]['stats']['sanity'], 'Rejoin healed or killed the survivor'
    assert after['camera_player'] == ('Blue' if victim == 1 else 'Red'), 'Rejoined guest watched the wrong player'
    return dict(death_tick=old['tick']-(1200-before_left), before_tick=old['tick'], after_tick=new['tick'],
        remaining_before=before_left, remaining_after=after_left, old_guest_pid=old_pid, new_guest_pid=after['pid'])


def run(exe, data, local, combat=False, death=False, delay=0, jitter=0, death_first='red', rejoin=False):
    assert not rejoin or death, 'Countdown rejoin needs the death fixture'
    folder = ROOT / 'private' / 'coop' / ('play-test-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    print('Evidence:', folder, flush=True)
    source = Path(__file__).read_bytes()
    (folder / 'fixture-source.py').write_bytes(source)
    invite = folder / 'invite.txt'
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    processes = []
    active = {}
    logs = []
    log_paths = {}
    initial = {}
    latest = {}
    history = {role: [] for role in ('host', 'guest')}
    minimum_health = {role: [100., 100.] for role in ('host', 'guest')}
    reconnect = None
    build = None
    try:
        def start(role, reconnecting=False):
            profile = folder / role
            profile.mkdir(exist_ok=reconnecting)
            log_paths[role] = folder / (role + ('-rejoin' if reconnecting else '') + '.log')
            log = log_paths[role].open('w')
            logs.append(log)
            args = [str(exe), '--data', str(data), '--map', 'skool1', '--no-audio',
                    '--profile', str(profile), '--coop-host' if role == 'host' else '--coop-join',
                    str(invite), '--coop-drive']
            if local and role == 'host':
                args.append('--coop-local')
            if (combat or death) and role == 'host':
                args.append('--combat-preview')
            if death:
                args += ['--coop-death-check', death_first]
            args += ['--coop-delay-ms', str(delay), '--coop-jitter-ms', str(jitter)]
            process = subprocess.Popen(args, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, creationflags=flags)
            processes.append(process)
            active[role] = process
            return process
        host = start('host')
        deadline = time.monotonic() + 150
        while not invite.exists():
            assert host.poll() is None, (folder / 'host.log').read_text()
            assert time.monotonic() < deadline, 'Host invite timeout'
            time.sleep(.05)
        guest = start('guest')
        deadline = time.monotonic() + 120
        while any(p.poll() is None for p in active.values()):
            assert time.monotonic() < deadline, 'Gameplay test timeout'
            for role, process in active.items():
                assert process.poll() in (None, 0), log_paths[role].read_text()
            for role in ('host', 'guest'):
                report = read(folder / role / 'coop-live.json')
                if report and report['pid'] == active[role].pid and report['playing']:
                    assert build is None or report['build'] == build, 'Clients used different builds'
                    build = report['build']
                    initial.setdefault(role, report)
                    latest[role] = report
                    minimum_health[role] = [min(old, body['stats']['sanity']) for old, body in
                        zip(minimum_health[role], report['frame']['players'])]
                    if death and (not history[role] or history[role][-1]['tick'] != report['frame']['tick']):
                        frame = report['frame']
                        history[role].append({'tick': frame['tick'], 'checkpoint_generation': frame['checkpoint_generation'],
                            'lives': frame['lives'], 'camera_player': report['camera_player'], 'camera_target': report['camera_target'],
                            'camera_subject_feet': report['camera_subject_feet'],
                            'guard': report['combat']['npcs']['actors'][0]['guard'],
                            'players': [{'health': b['stats']['sanity'], 'feet': b['player']['feet'], 'spawned_at': b['spawned_at']}
                                for b in frame['players']]})
            if rejoin and reconnect is None and 'host' in latest:
                victim = 0 if death_first == 'blue' else 1
                state = latest['host']['frame']['lives']['states'][victim]
                if isinstance(state, dict) and 600 <= state.get('Spectating', {}).get('ticks_left', 0) <= 1000:
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
                    # An observed inactive host must hold its full world and
                    # life clocks while waiting, including the dead body.
                    time.sleep(.4)
                    held = read(folder / 'host' / 'coop-live.json')
                    assert held and not held['playing'] and held['frame'] == before['frame']
                    assert held['combat'] == before['combat'], 'Disconnected host advanced the world'
                    guest = start('guest', reconnecting=True)
                    join_deadline = time.monotonic() + 60
                    while True:
                        after = read(folder / 'guest' / 'coop-live.json')
                        assert guest.poll() is None, 'Replacement guest failed: ' + log_paths['guest'].read_text()
                        if after and after['pid'] == guest.pid and after['playing']:
                            break
                        assert time.monotonic() < join_deadline, 'Guest rejoin timeout'
                        time.sleep(.05)
                    rejoin_record = dict(passed=False, before=before, held=held, after=after)
                    (folder / 'rejoin.json').write_text(json.dumps(rejoin_record, indent=2))
                    reconnect = verify_countdown_rejoin(before, after, victim, old_pid)
                    rejoin_record.update(passed=True, evidence=reconnect)
                    (folder / 'rejoin.json').write_text(json.dumps(rejoin_record, indent=2))
                    print('PASS fresh guest rejoined during countdown; paused world, inventory and timer preserved', flush=True)
            time.sleep(.05)
        assert host.returncode == 0, (folder / 'host.log').read_text()
        assert guest.returncode == 0, log_paths['guest'].read_text()
        assert not rejoin or reconnect is not None, 'Countdown reconnect was not exercised'
        assert set(latest) == {'host', 'guest'}, 'Both clients must receive a playing state'
        assert latest['host']['pid'] != latest['guest']['pid']
        assert latest['host']['camera_player'] == 'Blue'
        assert latest['guest']['camera_player'] == 'Red'
        positions = lambda r: [p['player']['feet'] for p in r['frame']['players']]
        before = positions(initial['host'])
        after = positions(latest['host'])
        replicated = positions(latest['guest'])
        moved = [distance(a, b) for a, b in zip(before, after)]
        recovery = None
        if death:
            recovery = {role: recovery_report(history[role], role, initial[role], 0 if death_first == 'blue' else 1)
                        for role in ('host', 'guest')}
            assert recovery['host'] == recovery['guest'], 'Clients disagreed about death/respawn/checkpoint history'
        elif combat:
            guard = lambda r: r['combat']['npcs']['actors'][0]['guard']['health']
            assert guard(initial['host']) > 0, 'Encounter did not begin with a living guard'
            assert guard(latest['host']) == guard(latest['guest']) == 0, 'Both clients must see the defeated guard'
            # Defeating the guard creates an essence pickup which can heal the
            # victim before the final frame. Verify the actual fight history.
            health = minimum_health['host']
            assert 0 < min(health) < 100 and max(health) == 100, f'Enemy damage was not personal: {health}'
            assert minimum_health['guest'] == health, 'Guest did not see the same personal damage'
            assert all(c['projectiles']['shots'] > 0 for c in latest['host']['combat']['characters']), 'Both players must attack through their real weapon path'
        else:
            assert all(d > 5 for d in moved), f'Both independently driven players must move: {moved}'
        assert all(distance(a, b) < 1 for a, b in zip(after, replicated)), 'Guest diverged from the host'
        assert latest['guest']['frame']['acknowledged_input'] > 0, 'No guest controls acknowledged'
        cameras = initial if combat or death else latest
        # The school back wall retracts both camera arms to the same Y plane.
        # Their origins remain distinct, but can be only a few units apart even
        # when they follow players standing 48 units apart. Role checks above
        # also verify that each camera watches its own body.
        assert distance(cameras['host']['camera'], cameras['guest']['camera']) > .1, 'Both clients used the same camera origin'
        for role in ('host', 'guest'):
            assert (folder / role / 'coop-screenshot.png').exists(), 'Missing rendered capture'
            if death:
                for stage in ('spectating', 'respawn', 'checkpoint'):
                    assert (folder / role / ('coop-' + stage + '.png')).exists(), 'Missing recovery capture: ' + stage
        result = {'passed': True, 'internet': not local, 'combat': combat, 'death': death, 'recovery': recovery, 'rejoin': reconnect,
                  'build': build, 'evidence': {'recovery': recovery, 'countdown_rejoin': reconnect,
                                             'minimum_health': minimum_health},
                  'fixture_sha256': hashlib.sha256(source).hexdigest(),
                  'delay_ms': delay, 'jitter_ms': jitter, 'death_first': death_first if death else None, 'moved': moved,
                  'initial': initial, 'final': latest, 'minimum_health': minimum_health,
                  'scope': 'native enemy death, timed spectating/respawn and entrance checkpoint; campaign not validated' if death else
                    'native enemy, weapon and personal damage; campaign not validated' if combat else 'two-player movement and replication; campaign not validated'}
        (folder / 'result.json').write_text(json.dumps(result, indent=2))
        print('PASS two rendered processes:', 'ten-second respawn, spectator cameras and shared entrance checkpoint.' if death else
            'shared combat and personal damage.' if combat else 'independent movement/cameras and matching host/guest state.')
        return folder
    finally:
        (folder / 'last-reports.json').write_text(json.dumps(latest, indent=2))
        (folder / 'diagnostic.json').write_text(json.dumps({'initial': initial, 'latest': latest, 'minimum_health': minimum_health,
            'exit_codes': [p.poll() for p in processes]}, indent=2))
        if death:
            (folder / 'lifecycle.json').write_text(json.dumps(history, indent=2))
        for process in processes:
            if process.poll() is None:
                if os.name == 'nt':
                    subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True)
                else:
                    process.terminate()
                process.wait(timeout=10)
        for log in logs:
            log.close()
        invite.unlink(missing_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    scenario = parser.add_mutually_exclusive_group()
    scenario.add_argument('--combat', action='store_true', help='Staged school guard, using real Blade attacks')
    scenario.add_argument('--death', action='store_true', help='School guard encounter: real deaths, moving spectator, respawn and wipe')
    parser.add_argument('--death-first', choices=('red', 'blue'), default='red')
    parser.add_argument('--rejoin', action='store_true', help='Restart the guest during the real death countdown')
    parser.add_argument('--delay-ms', type=int, default=0)
    parser.add_argument('--jitter-ms', type=int, default=0)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Network test delays must be between 0 and 500 milliseconds')
    if args.rejoin and not args.death:
        parser.error('--rejoin requires --death')
    run(args.exe.resolve(), args.data.resolve(), args.local, args.combat, args.death, args.delay_ms, args.jitter_ms, args.death_first, args.rejoin)
