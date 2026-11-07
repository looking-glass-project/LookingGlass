"""Run normal paired Garden Arena courses and verify exact native recovery.

Run in Anode (a graphics context is required). Uses isolated profiles and logs,
without screenshots. Public-invite and rendered-camera checks remain separate.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from coop_recovery import read_trace


def verify(profile, first, skip, require_revival):
    prefix = f'garden1-{first}-{str(skip).lower()}'
    report = json.loads((profile / f'{prefix}-route.json').read_text())
    assert report.get('full_route_passed') and report.get('allow_revival'), 'Native course did not pass'
    assert report.get('network_waypoints'), 'Course did not use the corrected network waypoints'
    assert report['first'] == first and report['skip'] == skip, 'Wrong course selection'
    assert report['frame']['checkpoint_generation'] == 0, 'Route team-wiped'
    assert report['world']['stage']['transition'] == ['garden2', 'garden2_start1'], 'Missing original exit'
    arrival = report['arrival']
    assert arrival['visit']['map'] == 'garden2' and arrival['visit']['entry'] == 'garden2_start1'
    for old, new in zip(report['frame']['players'], arrival['frame']['players']):
        assert old['stats'] == new['stats'] and new['stats']['sanity'] > 0, 'Arrival lost personal resources'
        assert not old['stats']['god'] and not old['stats']['notarget'], 'Route used cheats'
    recovery = read_trace(profile / f'{prefix}-recovery.jsonl', report['build'], require_revival)
    joins = json.loads((profile / f'{prefix}-rejoins.json').read_text())
    assert joins == report['revival_rejoins'], 'Continuation records disagree'
    assert len(joins) == len(recovery['revivals']), 'Missing or extra course continuation'
    for join, event in zip(joins, recovery['revivals']):
        assert (join['player'], join['tick']) == (event['player'], event['spawn_tick'])
        assert join['to_step'] == join['survivor_step'], 'Continuation passed the survivor'
    assert report['reunions'] == 2 and report['samples'] > 0, 'Missing scenes or replica checks'
    return dict(build=report['build'], ticks=report['frame']['tick'], replicas=report['samples'],
        steering_lead=report['steering_lead'],
        health=[p['stats']['sanity'] for p in report['frame']['players']],
        revival_rejoins=joins, recovery=recovery,
        scope='Normal paired production Arena route and exact recovery; internet and camera checks are separate')


def verify_visit(profile, map_name, first, skip, require_revival):
    if map_name == 'garden1':
        return verify(profile, first, skip, require_revival)
    prefix = f'{map_name}-{first}-{str(skip).lower()}' if map_name == 'garden2' else map_name
    report_name = f'{prefix}-passed.json' if map_name == 'garden2' else f'{map_name}-coop-route.json'
    report = json.loads((profile / report_name).read_text())
    assert report['full_route_passed'], 'Native course did not pass'
    assert report['frame']['checkpoint_generation'] == 0, 'Route team-wiped'
    next_map = {'garden2': 'garden3', 'garden3': 'garden4', 'garden4': 'centipede1'}[map_name]
    assert report['world']['stage']['transition'] == [next_map, f'{next_map}_start1']
    arrival = report['arrival']
    assert arrival['visit']['map'] == next_map and arrival['visit']['entry'] == f'{next_map}_start1'
    for old, new in zip(report['frame']['players'], arrival['frame']['players']):
        assert old['stats'] == new['stats'] and new['stats']['sanity'] > 0, 'Arrival lost personal resources'
        assert not old['stats']['god'] and not old['stats']['notarget'], 'Route used cheats'
    recovery = read_trace(profile / f'{prefix}-recovery.jsonl', report['build'], require_revival)
    joins = report['revival_rejoins']
    assert len(joins) == len(recovery['revivals']), 'Missing or extra course continuation'
    for join, event in zip(joins, recovery['revivals']):
        assert (join['player'], join['tick']) == (event['player'], event['spawn_tick'])
    assert report['samples'] > 0 and report['reunions'] >= {'garden2': 4, 'garden3': 1, 'garden4': 2}[map_name]
    if map_name == 'garden2':
        assert report['first'] == first and report['skip'] == skip and report['allow_revival']
        pickups = ['garden2:57' in p['stats']['collected'] for p in report['frame']['players']]
        assert pickups == report['personal_mallet'] and any(pickups), 'Missing or inconsistent Mallet pickup'
    if map_name == 'garden4':
        pickups = ['garden4:42' in p['stats']['collected'] for p in report['frame']['players']]
        assert pickups == report['personal_wand'] and any(pickups), 'Missing or inconsistent Ice Wand pickup'
    return dict(build=report['build'], ticks=report['frame']['tick'], replicas=report['samples'],
        health=[p['stats']['sanity'] for p in report['frame']['players']],
        revival_rejoins=joins, recovery=recovery, personal_mallet=report.get('personal_mallet'),
        personal_wand=report.get('personal_wand'),
        scope='Normal paired production Arena route and exact recovery; internet and camera checks are separate')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--profile', type=Path, required=True, help='New directory for this run')
    parser.add_argument('--map', choices=['garden1', 'garden2', 'garden3', 'garden4'], default='garden1')
    parser.add_argument('--first', choices=['Blue', 'Red'],
        help='Garden1 defaults to Red, Garden2 to Blue; ice currently uses Blue and the chase its authored arrival')
    parser.add_argument('--scenes', choices=['watched', 'skipped'], default='watched')
    parser.add_argument('--require-revival', action='store_true')
    parser.add_argument('--steering-lookahead', type=float, default=0.,
        help='Velocity lookahead in seconds for ordinary test inputs (0 to 0.5)')
    args = parser.parse_args()
    args.first = args.first or ('Blue' if args.map in ('garden2', 'garden4') else 'Red')
    if args.map == 'garden4' and args.first != 'Blue':
        parser.error('The ice course currently starts with Blue leading')
    if args.map in ('garden3', 'garden4') and args.scenes != 'watched':
        parser.error('The chase and ice courses currently watch their original scenes')
    exe, data, profile = args.exe.resolve(), args.data.resolve(), args.profile.resolve()
    assert exe.is_file() and data.is_dir(), 'Executable or game data is missing'
    profile.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    for key in ('LOOKING_GLASS_GARDEN_COOP_ROUTE', 'LOOKING_GLASS_GARDEN2_COOP_ROUTE', 'LOOKING_GLASS_GARDEN3_ROUTE', 'LOOKING_GLASS_GARDEN4_ROUTE'):
        env.pop(key, None)
    env_prefix = 'LOOKING_GLASS_GARDEN' if args.map == 'garden1' else 'LOOKING_GLASS_GARDEN2'
    env.update({f'{env_prefix}_COOP_FIRST': args.first, f'{env_prefix}_COOP_SCENES': args.scenes,
        f'{env_prefix}_COOP_RECOVERY': 'allowed', 'LOOKING_GLASS_GARDEN_COOP_STEERING': str(args.steering_lookahead)})
    result = dict(passed=False, map=args.map, executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest())
    try:
        with (profile / 'output.log').open('w', encoding='utf-8') as log:
            run = subprocess.run([str(exe), '--data', str(data), '--profile', str(profile),
                '--no-audio', f'--{args.map}-coop-route-check'], env=env, stdout=log,
                stderr=subprocess.STDOUT, timeout=1200)
        result['exit_code'] = run.returncode
        assert run.returncode == 0, f'Native Garden route exited {run.returncode}'
        result.update(verify_visit(profile, args.map, args.first, args.scenes == 'skipped', args.require_revival))
        result['passed'] = True
    except (AssertionError, OSError, ValueError, subprocess.TimeoutExpired) as error:
        result['error'] = str(error)
    (profile / 'result.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(result), flush=True)
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
