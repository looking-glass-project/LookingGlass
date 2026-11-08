"""Run a fresh normal paired Pale Realm Arena course without screenshots.

This accelerated check covers gameplay and compressed replicas. Independent
internet clients and rendered spectator cameras have separate checks.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def verify(profile, first):
    report = json.loads((profile / f'wchess1-{first}-false-passed.json').read_text())
    assert report['full_route_passed'] and report['course_passed'] and report['default_course']
    assert report['first'] == first and report['skip'] is False and report['samples'] > 0
    assert len(report['owners']) == 5 and report['frame']['checkpoint_generation'] == 0
    assert report['world']['stage']['transition'] == ['wchess2', 'wchess2_start1']
    saved = report['world']['stage']['world']['levels']['wchess1']
    assert saved['bishop_done'] and saved['knight_done']
    for player in report['frame']['players']:
        assert player['stats']['sanity'] > 0 and player['spawned_at'] == 0
        assert not player['stats']['god'] and not player['stats']['notarget']
    return dict(build=report['build'], ticks=report['frame']['tick'], replicas=report['samples'],
        course_sha256=report['course_sha256'], scenes=report['owners'],
        health=[p['stats']['sanity'] for p in report['frame']['players']],
        scope='Normal paired Arena course with native Castling arrival assertions; separate internet/camera evidence')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--profile', type=Path, required=True)
    parser.add_argument('--first', choices=['Blue', 'Red'], default='Blue')
    args = parser.parse_args()
    exe, data, profile = args.exe.resolve(), args.data.resolve(), args.profile.resolve()
    assert exe.is_file() and data.is_dir()
    profile.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env.pop('LOOKING_GLASS_CHESS_COOP_COURSE', None)
    env['LOOKING_GLASS_CHESS_COOP_FIRST'] = args.first
    env['LOOKING_GLASS_CHESS_COOP_SCENES'] = 'watched'
    result = dict(passed=False, first=args.first, executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest())
    try:
        with (profile / 'output.log').open('w', encoding='utf-8') as log:
            run = subprocess.run([str(exe), '--data', str(data), '--profile', str(profile),
                '--no-audio', '--wchess1-coop-route-check'], env=env, stdout=log,
                stderr=subprocess.STDOUT, timeout=1500)
        result['exit_code'] = run.returncode
        assert run.returncode == 0, f'Native Pale Realm course exited {run.returncode}'
        result.update(verify(profile, args.first))
        result['passed'] = True
    except (AssertionError, OSError, ValueError, subprocess.TimeoutExpired) as error:
        result['error'] = str(error)
    (profile / 'result.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(result), flush=True)
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
