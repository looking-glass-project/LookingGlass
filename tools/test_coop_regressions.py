"""Run focused co-op regressions and write a compact machine-readable report.

Run in Anode when selecting rendered tests (presence or paired routes/recovery). Native tests
can run without a desktop. Screenshots remain in private evidence folders and
are never opened by this runner. A pass is not full campaign certification.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'private' / 'coop'


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def stop_owned_process(process):
    if process.poll() is not None:
        return
    if os.name == 'nt':
        subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
    else:
        os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        if os.name != 'nt':
            os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=10)


def evidence_from_log(log):
    match = re.search(r'^Evidence: (.+)$', log.read_text(errors='replace'), re.MULTILINE)
    if not match:
        raise ValueError('Rendered test did not report an evidence folder')
    folder = Path(match.group(1).strip()).resolve()
    if not folder.is_relative_to(EVIDENCE.resolve()):
        raise ValueError('Test evidence is outside private/coop')
    return folder


def result_from_log(log):
    folder = evidence_from_log(log)
    result = json.loads((folder / 'result.json').read_text())
    if result.get('passed') is not True or not re.fullmatch(r'[0-9a-f]{64}', result.get('build', '')):
        raise ValueError('Rendered test has no valid passing result/build identity')
    fields = ('matching_sent_and_restored_states', 'matching_shared_world_states', 'matching_full_school_frames',
              'contact_distance', 'stationary_partner_unchanged',
              'both_models_and_particles_visible', 'checkpoint_sha256', 'resume_point', 'lifts',
              'minimum_will', 'final_health', 'staged_initial_spawn', 'host_stalls',
              'host_recovery', 'guest_recovery', 'wipe', 'loading_rejoin', 'recovery',
              'countdown_rejoin', 'countdown_rejoins', 'revivals', 'exact_recovery', 'moving_support_recovery',
              'minimum_health', 'scene_rejoin', 'scene_reunion')
    metrics = {key: result['evidence'][key] for key in fields if key in result['evidence']}
    rejoin = result['evidence'].get('rejoin')
    if rejoin:
        if 'rider_offsets' in rejoin:
            metrics['lift_rejoin'] = {key: rejoin[key] for key in
                ('before_tick', 'after_tick', 'motion_before', 'motion_after', 'rider_offsets')}
        else:
            metrics['rejoin_preserved_resources_and_scenes'] = rejoin['resources_and_completed_scenes_preserved']
    timing = folder / 'timing-summary.json'
    if timing.exists():
        metrics['timing_summary'] = json.loads(timing.read_text())
    return dict(build=result['build'], evidence=str(folder), scope=result['scope'],
                full_route_passed=result.get('full_route_passed'), mode=result.get('mode'),
                first=result.get('first'), metrics=metrics)


def failure_from_log(log, rendered):
    """Keep enough terminal evidence to triage without opening images/history."""
    result = {'log_tail': '\n'.join(log.read_text(errors='replace').splitlines()[-18:])[-6000:]}
    if not rendered:
        return result
    try:
        folder = evidence_from_log(log)
        result['evidence'] = str(folder)
        components = []
        for path in sorted(folder.glob('milestone-*.json')):
            milestone = json.loads(path.read_text())
            if milestone.get('passed') is True and milestone.get('full_visit') is False:
                components.append({k: milestone.get(k) for k in ('phase', 'scope', 'build', 'first')})
        if components:
            result['completed_components'] = components
        reports = json.loads((folder / 'last-reports.json').read_text())
        result['clients'] = {}
        for role in ('host', 'guest'):
            if role not in reports:
                continue
            report = reports[role]
            route = report.get('route_driver') or {}
            state = dict(visit=report.get('visit'), tick=report['frame']['tick'],
                         route={k: route.get(k) for k in
                                ('command', 'phase', 'reached', 'error', 'scene', 'replans',
                                 'jumps', 'plan_remaining', 'expected')},
                         players=[dict(feet=b['player']['feet'], grounded=b['player']['grounded'],
                                       velocity=b['player']['velocity'], jumps=b['player']['jumps'],
                                       health=b['stats']['sanity'], spawned_at=b['spawned_at'])
                                  for b in report['frame']['players']],
                         network_timing=report.get('network_timing'), frame_timing=report.get('frame_timing'))
            command = folder / role / 'route-command.json'
            if command.exists():
                state['last_command'] = json.loads(command.read_text())
            result['clients'][role] = state
        observations = folder / 'observations.json'
        if observations.exists():
            data = json.loads(observations.read_text())
            result['last_stops'] = {k: data[k][-1] for k in
                                   ('waiting_spots', 'paired_endpoints', 'passage_stops') if data.get(k)}
        recent = folder / 'recent-movement.json'
        if recent.exists():
            data = json.loads(recent.read_text())
            result['recent_movement'] = dict(file=str(recent), clients={
                role: {key: value[key] for key in ('epoch', 'first_tick', 'last_tick', 'samples', 'peaks')}
                for role, value in data['clients'].items()})
        timing = folder / 'timing-summary.json'
        if timing.exists():
            result['timing_summary'] = dict(file=str(timing), **json.loads(timing.read_text()))
        for role, state in result['clients'].items():
            command = state['route'].get('command')
            state['plans'] = [p.relative_to(folder).as_posix() for p in
                              sorted((folder / role).glob(f'route-plan-{command}-*.json'))]
    except (OSError, ValueError, KeyError, TypeError) as error:
        # A crash or timeout may leave no final report. Preserve the test's
        # original failure and log instead of making missing evidence a pass.
        result['diagnostic_error'] = str(error)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--only', nargs='+', choices=('fixture-scenes', 'world', 'school2', 'presence', 'host-stalls', 'school',
                                                   'school-blue', 'school-red', 'lift-blue', 'lift-red',
                                                   'lift-rejoin-blue', 'lift-rejoin-red',
                                                   'travel-rejoin-outbound', 'travel-rejoin-return',
                                                   'death-rejoin-blue', 'death-rejoin-red',
                                                   'swim-countdown-blue', 'swim-countdown-red',
                                                   'rope-countdown', 'fulcrum-countdown',
                                                   'scene-rejoin-blue-intro', 'scene-rejoin-blue-dialogue',
                                                   'scene-rejoin-red-intro', 'scene-rejoin-red-dialogue',
                                                   'stairs-blue', 'stairs-red', 'school2-blue', 'school2-red',
                                                   'dice-blue', 'dice-red', 'altar-blue', 'altar-red',
                                                   'gym-jump-blue', 'gym-jump-red', 'school2-resume',
                                                   'return-sp', 'return-chain', 'return-blue', 'return-red', 'return-resume',
                                                   'return-recovery-blue', 'return-recovery-red',
                                                   'temple-playable', 'garden-playable'),
                        help='school and school2-resume resume checkpoints; school-blue/red and school2-blue/red are full routes; lift/stairs/altar/gym-jump are staged; dice is a partial School Two entrance route')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    parser.add_argument('--school-checkpoint', type=Path)
    parser.add_argument('--rope-checkpoint', type=Path,
                        help='Unchanged captured Blue-on-rope/Red-falling checkpoint for rope-countdown')
    parser.add_argument('--fulcrum-checkpoint', type=Path,
                        help='Unchanged captured second-fulcrum checkpoint for fulcrum-countdown')
    parser.add_argument('--school2-checkpoint', type=Path)
    parser.add_argument('--return-checkpoint', type=Path)
    parser.add_argument('--return-resume-point', choices=('entrance', 'theatre', 'spiral', 'observatory-approach', 'observatory'))
    parser.add_argument('--school2-resume-point', choices=('dice', 'gym', 'laboratory-approach', 'laboratory-entrance', 'greenhouse'), default='gym',
                        help='dice repeats only dice-to-gym; later checkpoints continue to exit')
    parser.add_argument('--school-resume-point', choices=('theatre', 'balcony', 'spiral', 'library', 'bridge', 'descent'))
    parser.add_argument('--first', choices=('blue', 'red'), default='blue',
                        help='Arrival order for the resumed school case; full cases name their own order')
    args = parser.parse_args()
    args.exe, args.data = args.exe.resolve(), args.data.resolve()
    if args.school2_checkpoint:
        args.school2_checkpoint = args.school2_checkpoint.resolve()
        if not args.school2_checkpoint.is_file():
            parser.error('The unchanged ordinary School Two checkpoint must exist')
    if bool(args.return_checkpoint) != bool(args.return_resume_point):
        parser.error('Provide both --return-checkpoint and --return-resume-point')
    if args.return_checkpoint:
        args.return_checkpoint = args.return_checkpoint.resolve()
        if not args.return_checkpoint.is_file():
            parser.error('The unchanged ordinary return-school checkpoint must exist')
    if not args.exe.is_file() or not args.data.is_dir():
        parser.error('--exe must be a built executable and --data an existing game-data directory')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if bool(args.school_checkpoint) != bool(args.school_resume_point):
        parser.error('Provide both --school-checkpoint and --school-resume-point')
    if args.school_checkpoint:
        args.school_checkpoint = args.school_checkpoint.resolve()
        if not args.school_checkpoint.is_file():
            parser.error('The unchanged ordinary co-op checkpoint must exist')
    selected = list(dict.fromkeys(args.only or ['fixture-scenes', 'world', 'school2', 'presence', 'host-stalls'] +
                                 (['school'] if args.school_checkpoint else [])))
    if 'school' in selected and not args.school_checkpoint:
        parser.error('The school regression needs --school-checkpoint and --school-resume-point')
    if 'rope-countdown' in selected:
        if not args.rope_checkpoint or not args.rope_checkpoint.is_file():
            parser.error('The rope countdown check needs an existing --rope-checkpoint')
        args.rope_checkpoint = args.rope_checkpoint.resolve()
    if 'fulcrum-countdown' in selected:
        if not args.fulcrum_checkpoint or not args.fulcrum_checkpoint.is_file():
            parser.error('The moving-support countdown check needs an existing --fulcrum-checkpoint')
        args.fulcrum_checkpoint = args.fulcrum_checkpoint.resolve()
    if 'school2-resume' in selected and not args.school2_checkpoint:
        parser.error('The school2-resume regression needs --school2-checkpoint')
    if 'return-resume' in selected and not args.return_checkpoint:
        parser.error('The return-resume regression needs --return-checkpoint and --return-resume-point')
    common = ['--exe', str(args.exe), '--data', str(args.data),
              '--delay-ms', str(args.delay_ms), '--jitter-ms', str(args.jitter_ms)]
    if args.local:
        common.append('--local')
    cases = {
        'fixture-scenes': ([sys.executable, str(ROOT / 'tools/test_coop_fixture_scenes.py')], 30, False),
        'world': ([str(args.exe), '--data', str(args.data), '--coop-world-check'], 180, False),
        'school2': ([str(args.exe), '--data', str(args.data), '--school2-check'], 180, False),
        'return-sp': ([str(args.exe), '--data', str(args.data), '--school-return-check'], 600, False),
        'return-chain': ([str(args.exe), '--data', str(args.data), '--school-return-chain-check'], 600, False),
        'presence': ([sys.executable, str(ROOT / 'tools/test_coop_presence.py'), *common], 300, True),
        'host-stalls': ([sys.executable, str(ROOT / 'tools/test_coop_host_stalls.py'), *common], 300, True),
        'temple-playable': ([sys.executable, str(ROOT / 'tools/test_coop_utemple.py'), *common,
                             '--first', args.first, '--allow-revival'], 960, True),
        'garden-playable': ([sys.executable, str(ROOT / 'tools/test_coop_garden.py'), *common,
                             '--first', args.first, '--allow-revival'], 1260, True),
    }
    if args.rope_checkpoint:
        cases['rope-countdown'] = ([sys.executable, str(ROOT / 'tools/test_coop_rope_countdown.py'),
            *common, '--checkpoint', str(args.rope_checkpoint)], 300, True)
    if args.fulcrum_checkpoint:
        cases['fulcrum-countdown'] = ([sys.executable, str(ROOT / 'tools/test_coop_fulcrum_countdown.py'),
            *common, '--checkpoint', str(args.fulcrum_checkpoint)], 300, True)
    for direction in ('outbound', 'return'):
        cases['travel-rejoin-' + direction] = ([sys.executable, str(ROOT / 'tools/test_coop_travel.py'), *common,
                                               '--reconnect-at', direction], 300, True)
    for first in ('blue', 'red'):
        cases['swim-countdown-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_utemple_countdown.py'),
            *common, '--first', first], 600, True)
        for phase, seconds in (('intro', 6), ('dialogue', 30)):
            cases[f'scene-rejoin-{first}-{phase}'] = ([sys.executable, str(ROOT / 'tools/test_coop_scene.py'), *common,
                '--first', first, '--rejoin', '--rejoin-at', str(seconds)], 420, True)
        cases['death-rejoin-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_play.py'), *common,
                                          '--death', '--death-first', first, '--rejoin'], 300, True)
        cases['return-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school_return.py'), *common,
                                    '--first', first, '--rejoin'], 1560, True)
        cases['return-recovery-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school_return_recovery.py'),
                                             *common, '--first', first], 600, True)
        cases['school-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school.py'), *common,
                                    '--first', first, '--rejoin'], 1500, True)
        cases['lift-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school_lift.py'), *common,
                                  '--first', first], 300, True)
        cases['lift-rejoin-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school_lift.py'), *common,
                                         '--first', first, '--rejoin'], 300, True)
        cases['stairs-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school_stairs.py'), *common,
                                    '--first', first], 300, True)
        # Recorded entrance/gym, return and later segments total about 27
        # minutes. Allow the complete route to finish; per-leg/fight/scene
        # watchdogs still reject stalled movement and encounters promptly.
        cases['school2-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school2.py'), *common,
                                     '--first', first, '--rejoin'], 1740, True)
        cases['dice-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school2.py'), *common,
                                  '--first', first, '--stop-after', 'dice'], 900, True)
        cases['altar-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school2.py'), *common,
                                   '--first', first, '--stop-after', 'dice', '--stage-at-altar'], 300, True)
        cases['gym-jump-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_gym_jump.py'), *common,
                                      '--first', first], 300, True)
    if args.school_checkpoint:
        cases['school'] = ([sys.executable, str(ROOT / 'tools/test_coop_school_resume.py'), *common,
                           '--checkpoint', str(args.school_checkpoint), '--resume-point', args.school_resume_point,
                           '--first', args.first, '--rejoin'], 1200, True)
    if args.return_checkpoint:
        cases['return-resume'] = ([sys.executable, str(ROOT / 'tools/test_coop_school_return.py'), *common,
                                 '--checkpoint', str(args.return_checkpoint), '--resume-point', args.return_resume_point,
                                 '--first', args.first, '--rejoin'], 1560, True)
    if args.school2_checkpoint:
        cases['school2-resume'] = ([sys.executable, str(ROOT / 'tools/test_coop_school2_resume.py'), *common,
                                  '--checkpoint', str(args.school2_checkpoint),
                                  '--resume-point', args.school2_resume_point,
                                  '--first', args.first, '--rejoin'], 1200, True)
    folder = EVIDENCE / ('regression-suite-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    report_path = folder / 'summary.json'
    report = dict(format=1, status='running', passed=None, scope='Focused regressions; not full campaign certification',
                  executable=str(args.exe), executable_sha256=digest(args.exe),
                  internet=not args.local, delay_ms=args.delay_ms, jitter_ms=args.jitter_ms,
                  selected=selected, fixtures={p.name: digest(p) for p in sorted([
                      *(ROOT / 'tools').glob('test_coop_*.py'), ROOT / 'tools/coop_diagnostics.py',
                      ROOT / 'tools/coop_travel_rejoin.py', ROOT / 'tools/coop_swim_rejoin.py',
                      ROOT / 'tools/coop_rope_rejoin.py', ROOT / 'tools/coop_recovery.py'])},
                  cases=[])

    def save():
        temporary = folder / 'summary.tmp'
        temporary.write_text(json.dumps(report, indent=2))
        temporary.replace(report_path)

    save()
    print('Report:', report_path, flush=True)
    interrupted = False
    build = None
    for name in selected:
        command, timeout, rendered = cases[name]
        log = folder / (name + '.log')
        case = dict(name=name, passed=False, log=str(log), timeout_seconds=timeout)
        started = time.monotonic()
        print('RUN', name, flush=True)
        try:
            with log.open('w') as output:
                process = subprocess.Popen(command, cwd=ROOT, stdout=output, stderr=subprocess.STDOUT,
                    creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    start_new_session=os.name != 'nt')
                try:
                    case['exit_code'] = process.wait(timeout=timeout)
                except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
                    interrupted = isinstance(error, KeyboardInterrupt)
                    stop_owned_process(process)
                    raise RuntimeError('Interrupted' if interrupted else 'Test exceeded its time limit') from error
            if case['exit_code'] != 0:
                raise RuntimeError('Test failed; see its private log')
            if rendered:
                case.update(result_from_log(log))
                if name in ('temple-playable', 'garden-playable') and case['full_route_passed'] is not True:
                    raise ValueError('Playable route stopped before a fresh-entry full traversal')
                if build is not None and build != case['build']:
                    raise ValueError('Rendered tests used different game builds')
                build = case['build']
            # Detect accidental replacement of the executable during a suite.
            if digest(args.exe) != report['executable_sha256']:
                raise ValueError('The executable changed during the suite')
            case['passed'] = True
        except (OSError, ValueError, RuntimeError, KeyError, subprocess.SubprocessError) as error:
            case['error'] = str(error)
            if log.exists():
                case['failure'] = failure_from_log(log, rendered)
        case['seconds'] = round(time.monotonic() - started, 2)
        report['cases'].append(case)
        save()
        print('PASS' if case['passed'] else 'FAIL', name, f"({case['seconds']}s)", flush=True)
        if not case['passed']:
            break
    report.update(status='interrupted' if interrupted else 'complete',
                  passed=len(report['cases']) == len(selected) and all(c['passed'] for c in report['cases']),
                  build=build)
    save()
    print('PASS suite' if report['passed'] else 'FAIL suite', '-', report_path, flush=True)
    return 130 if interrupted else 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
