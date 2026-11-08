"""Run a normal paired production-Arena course and verify its saved evidence.

Use Anode for the windowed executable. This checks gameplay and compressed
replicas; public internet transport and rendered cameras have separate checks.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from coop_recovery import read_trace

EXITS = {'centipede1': 'centipede2', 'grounds2': 'facade', 'facade': 'keep', 'keep': 'qlair',
         'wforest': 'wchess1', 'wforest-return': 'hedge1', 'tower1': 'hedge2', 'hedge2': 'tower2',
         'tower2': 'hedge3', 'hedge3': 'tower3', 'tower3': 'grounds1', 'qlair': None, 'hedge1': 'tower1',
         'potears3': 'utemple', 'skool2': 'skool1', 'potears2': 'potears3', 'potears1': 'potears2'}


def verify(profile, map_name):
    report_map = map_name.split('-')[0]
    report = json.loads((profile / f'{report_map}-coop-route.json').read_text())
    assert report['full_route_passed'], report.get('error')
    assert report['replica_samples'] > 0 and report['frame']['checkpoint_generation'] == 0
    if map_name == 'qlair':
        assert report['ending_ready'] and report['arrival'] is None
        assert report['world']['stage']['transition'] is None
        queen = report['world']['stage']['world']['levels']['qlair']
        assert queen['phase'] == 'Done' and queen['queen1'] <= 0 and queen['queen2'] < 1000
        assert [p['phase'] for p in report['phases']] == ['Intro', 'Queen1', 'Birth', 'Queen2', 'Death', 'Ending', 'Done']
        assert report['checkpoint']['phase'] == 'Queen2' and report['checkpoint']['exact_restore']
        assert report['health_floors'] == [0, 0]
        assert all(p['stats']['sanity'] > 0 and not p['stats']['god'] and not p['stats']['notarget']
                   for p in report['frame']['players'])
    else:
        next_map = EXITS[map_name]
        next_entry = None if map_name == 'potears3' else f'{next_map}_start1'
        if map_name == 'skool2':
            next_entry = 'skool1_start2'
        assert report['world']['stage']['transition'] == [next_map, next_entry]
        arrival = report['arrival']
        assert arrival['visit']['map'] == next_map and arrival['visit']['entry'] == next_entry
        assert arrival['health_floors'] == [0, 0], 'Arrival retained the previous visit health floor'
        for old, new in zip(report['frame']['players'], arrival['frame']['players']):
            expected = dict(old['stats'], owned=list(old['stats']['owned']))
            if map_name == 'centipede1':
                # Sanctum supplies the campaign's second Die if a player missed it.
                expected['owned'][6] = max(expected['owned'][6], 2)
            if map_name == 'wforest-return':
                # The return's optional Blunderbuss is the next campaign milestone.
                expected['owned'][8] = max(expected['owned'][8], 1)
            if map_name == 'potears3':
                # Temple entry grants the returned breathing shell and fills
                # a missing Jackbomb milestone for the non-collecting partner.
                expected['turtle_air'] = True
                expected['owned'][3] = max(expected['owned'][3], 1)
            if map_name == 'skool2':
                expected['owned'][6] = max(expected['owned'][6], 1)
            assert expected == new['stats'], 'Arrival changed resources beyond its campaign weapon grant'
            assert new['stats']['sanity'] > 0 and not old['stats']['god'] and not old['stats']['notarget']
    recovery = read_trace(profile / report['recovery_log'], report['build'])
    revivals = {(r['player'], r['spawn_tick']) for r in recovery['revivals']}
    recorded = {(r['slot'], r['tick']) for r in report['life_events'] if r['alive'] and r['spawned_at'] == r['tick']}
    assert revivals == recorded, 'Native life events disagree with exact revival trace'
    dice = None
    if map_name == 'centipede1':
        dice = ['centipede1:299' in p['stats']['collected'] for p in report['frame']['players']]
        assert any(dice) and report['reunions'] == 1, 'Flora missed its pickup or watched ambush reunion'
        # Done is a scripted departure state, so it takes scene ownership again
        # on the final exit tick; it does not run a second watched cutscene.
        assert len(report['scene_starts']) == 2
        assert report['scene_starts'][-1]['tick'] == report['frame']['tick'] - 1
        assert report['health_floors'] == [10, 10]
    if report_map == 'wforest':
        entry = 'wforest_start2' if map_name.endswith('-return') else 'wforest_start1'
        assert report['visit']['map'] == 'wforest' and report['visit']['entry'] == entry
    if map_name == 'potears3':
        boss = report['world']['stage']['world']['duchess']
        assert boss['stage'] == 'Complete' and boss['health'] == 0
        assert boss['action'] == 'Death' and boss['shell_returned'] and boss['attacks'] > 0
        assert any('potears3:21' in b['stats']['collected'] for b in report['frame']['players'])
        assert report['reunions'] == 1 and len(report['scene_starts']) == 2
    if map_name == 'skool2':
        quest = report['world']['stage']['world']['school2']['quest']
        assert quest['stage'] == 'Complete'
        assert all(b['stats']['school_items']['potion'] and b['stats']['school_items']['star']
                   for b in report['frame']['players'])
    if map_name == 'potears2':
        pond = report['world']['stage']['world']['levels']['potears2']
        assert all(t is not None for t in pond['pond']['leaves']) and pond['pond']['ladies']
        assert pond['fish']['attack'] is None and pond['gate_time'] == 2
        assert pond['scene']['phase'] == 'Done' and all(g['health'] <= 0 for g in pond['guards'])
        assert report['health_floors'] == [0, 0]
    if map_name == 'potears1':
        pool = report['world']['stage']['world']['pool']
        assert all(t is not None for t in pool['rides'] + pool['turtle'])
        assert all(pool['cinema']['done']) and pool['cinema']['exit_sent']
        assert report['health_floors'] == [0, 0]
    return dict(build=report['build'], ticks=report['frame']['tick'], replicas=report['replica_samples'],
        reunions=report['reunions'], health=[p['stats']['sanity'] for p in report['frame']['players']],
        personal_second_die=dice, recovery=recovery,
        scope='Normal paired Arena route and exact recovery; internet and rendered-camera evidence is separate')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--profile', type=Path, required=True, help='New isolated directory')
    parser.add_argument('--map', choices=EXITS, required=True)
    args = parser.parse_args()
    exe, data, profile = args.exe.resolve(), args.data.resolve(), args.profile.resolve()
    assert exe.is_file() and data.is_dir(), 'Executable or game data is missing'
    profile.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env.pop('LOOKING_GLASS_COOP_NATIVE_ROUTE', None)
    env.pop('LOOKING_GLASS_CENTIPEDE1_ROUTE', None)
    env.pop('LOOKING_GLASS_WFOREST_ROUTE', None)
    result = dict(passed=False, map=args.map, executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest())
    try:
        with (profile / 'output.log').open('w', encoding='utf-8') as log:
            run = subprocess.run([str(exe), '--data', str(data), '--profile', str(profile),
                '--no-audio', f'--{args.map}-coop-route-check'], env=env, stdout=log,
                stderr=subprocess.STDOUT, timeout=1500)
        result['exit_code'] = run.returncode
        assert run.returncode == 0, f'Native route exited {run.returncode}'
        result.update(verify(profile, args.map))
        result['passed'] = True
    except (AssertionError, OSError, ValueError, subprocess.TimeoutExpired) as error:
        result['error'] = str(error)
    (profile / 'result.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(result), flush=True)
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
