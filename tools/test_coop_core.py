"""Run focused co-op save/recovery checks in an isolated desktop.

Public invites are the default; --local explicitly selects loopback. This
complements test_coop_regressions.py and the main/ESC menu lifecycle fixtures.
It stops on the first failure, retains partial evidence, and never retries a
provider error or opens screenshots. It does not certify campaign completion.
"""
import argparse
import ctypes
from dataclasses import dataclass
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import uuid

from test_coop_regressions import ROOT, EVIDENCE, digest, evidence_from_log, failure_from_log, stop_owned_process


@dataclass(frozen=True)
class Case:
    script: str
    extra: tuple = ()
    results: tuple = ('result.json',)
    seconds: int = 420
    menu: bool = False


CASES = {
    'travel-return': Case('test_coop_travel.py', ('--reconnect-at', 'return')),
    'import': Case('test_coop_import.py', results=('import-result.json', 'resume-result.json',
        'resume-after-loss-result.json', 'backup-result.json', 'host-loss-result.json'), seconds=600),
    'checkpoint': Case('test_coop_checkpoint.py', results=('result.json', 'resume-result.json',
        'backup-result.json'), seconds=600),
    'storage': Case('test_coop_menu_storage.py', menu=True),
    'host-loss-menu': Case('test_coop_menu_host_loss.py', menu=True),
    'save-death': Case('test_coop_menu_save_rules.py', ('--case', 'death'), menu=True),
    'save-scene-blue': Case('test_coop_menu_save_rules.py', ('--case', 'scene', '--first', 'blue'), menu=True),
    'save-scene-red': Case('test_coop_menu_save_rules.py', ('--case', 'scene', '--first', 'red'), menu=True),
    'lift-blue': Case('test_coop_school_lift.py', ('--first', 'blue', '--rejoin')),
    'lift-red': Case('test_coop_school_lift.py', ('--first', 'red', '--rejoin')),
    'swim': Case('test_coop_utemple_countdown.py', ('--first', 'blue'), seconds=600),
    'rope': Case('test_coop_rope_countdown.py'),
    'fulcrum': Case('test_coop_fulcrum_countdown.py'),
}


def collect_results(log, case, internet, previous_build=None, preexisting=()):
    """Require every phase, the requested network scope and one native build."""
    folder = evidence_from_log(log)
    if folder in preexisting:
        raise ValueError('Fixture reused evidence that existed before this invocation')
    builds, results = set(), {}

    def read_object(path):
        value = json.loads(path.read_text())
        if not isinstance(value, dict):
            raise ValueError(f'{path.name} is not a result object')
        return value

    def add_build(value):
        if not isinstance(value, str) or not re.fullmatch(r'[0-9a-f]{64}', value):
            raise ValueError('Missing or inconsistent native build identities')
        builds.add(value)

    for name in case.results:
        path = folder / name
        result = read_object(path)
        if result.get('passed') is not True:
            raise ValueError(f'{name} is not a passing result')
        # Host-loss is a subordinate process/save receipt; the four connected
        # import phases each independently report their network scope.
        if name != 'host-loss-result.json' and result.get('internet') is not internet:
            raise ValueError(f'{name} does not prove the requested network scope')
        details = result.get('evidence', {})
        if not isinstance(details, dict):
            raise ValueError(f'{name} has invalid evidence metadata')
        build = result.get('build') or details.get('build')
        if build is not None:
            add_build(build)
        results[name] = digest(path)
    for role in ('host', 'guest'):
        path = folder / role / 'coop-live.json'
        if path.exists():
            add_build(read_object(path).get('build'))
    restored = folder / 'host/route-resume.json'
    if restored.exists():
        receipt = read_object(restored)
        if receipt.get('verified') is not True:
            raise ValueError('Native resume receipt was not verified')
        add_build(receipt.get('build'))
    if len(builds) != 1 or not all(isinstance(b, str) and re.fullmatch(r'[0-9a-f]{64}', b) for b in builds):
        raise ValueError('Missing or inconsistent native build identities')
    build = builds.pop()
    if previous_build is not None and build != previous_build:
        raise ValueError('Core cases used different native builds')
    return dict(evidence=str(folder), build=build, results=results)


def require_desktop(session, needs_lease):
    if os.name != 'nt':
        raise ValueError('These rendered core checks require Windows and an isolated desktop')
    actual = ctypes.c_ulong()
    if not ctypes.windll.kernel32.ProcessIdToSessionId(os.getpid(), ctypes.byref(actual)):
        raise ValueError('Could not verify the current desktop session')
    if actual.value != session or not session:
        raise ValueError('Run inside the Anode session reported by seat_status')
    if needs_lease and not all(os.environ.get(key) for key in ('ANODE_AGENT_ID', 'ANODE_LEASE_TOKEN')):
        raise ValueError('Forward the active Anode agent and lease for menu-save focus')


def school_runner(folder, fixture):
    """Optional immutable SchoolPair snapshot; never replace a worktree file."""
    wrapper = folder / 'school-runner.py'
    wrapper.write_text(
        'import importlib.util,runpy,sys\n'
        f'sys.path.insert(0,{str(ROOT / "tools")!r})\n'
        f'spec=importlib.util.spec_from_file_location("test_coop_school",{str(fixture)!r})\n'
        'module=importlib.util.module_from_spec(spec)\n'
        'sys.modules[spec.name]=module\n'
        'spec.loader.exec_module(module)\n'
        f'sys.argv[0]={str(ROOT / "tools/test_coop_school_lift.py")!r}\n'
        'runpy.run_path(sys.argv[0],run_name="__main__")\n', encoding='utf-8')
    return wrapper


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--seat-session', type=int, required=True)
    parser.add_argument('--only', nargs='+', choices=CASES, help=f'Default: all {len(CASES)} cases')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    parser.add_argument('--rope-checkpoint', type=Path)
    parser.add_argument('--fulcrum-checkpoint', type=Path)
    parser.add_argument('--source-save', type=Path, help='Optional existing quiet solo save for the import case')
    parser.add_argument('--school-fixture', type=Path, help='Optional saved SchoolPair Python module for lift checks')
    args = parser.parse_args()
    selected = list(dict.fromkeys(args.only or CASES))
    args.exe, args.data = args.exe.resolve(), args.data.resolve()
    if not args.exe.is_file() or not args.data.is_dir():
        parser.error('Provide a built executable and an existing original base-data folder')
    if not all(0 <= n <= 500 for n in (args.delay_ms, args.jitter_ms)):
        parser.error('Delay and jitter must be between 0 and 500 ms')
    for name in ('rope', 'fulcrum'):
        path = getattr(args, name + '_checkpoint')
        if name in selected and (not path or not path.is_file()):
            parser.error(f'{name} requires --{name}-checkpoint with an unchanged normal gameplay save')
    if args.school_fixture and (not args.school_fixture.is_file() or args.school_fixture.suffix != '.py'):
        parser.error('--school-fixture must be an existing Python module')
    if args.source_save and ('import' not in selected or not args.source_save.is_file()):
        parser.error('--source-save requires the import case and an existing solo save file')
    try:
        require_desktop(args.seat_session, any(name.startswith('save-') for name in selected))
    except ValueError as error:
        parser.error(str(error))
    folder = EVIDENCE / ('core-suite-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    executable_hash = digest(args.exe)
    fixtures = {p.name: digest(p) for p in sorted((ROOT / 'tools').glob('*coop*.py'))}
    report = dict(status='running', passed=False, internet=not args.local, executable=str(args.exe),
        executable_sha256=executable_hash, selected=selected, fixtures=fixtures, cases=[],
        scope='Focused saves, menu eligibility, reconnect and recovery; not full campaign verification')
    if args.source_save:
        args.source_save = args.source_save.resolve()
        report['source_save'] = dict(path=str(args.source_save), sha256=digest(args.source_save))
    if args.school_fixture:
        args.school_fixture = args.school_fixture.resolve()
        report['school_fixture'] = dict(path=str(args.school_fixture), sha256=digest(args.school_fixture))
        lift = school_runner(folder, args.school_fixture)
    else:
        lift = ROOT / 'tools/test_coop_school_lift.py'
    for name in ('rope', 'fulcrum'):
        if name in selected:
            path = getattr(args, name + '_checkpoint').resolve()
            report[name + '_checkpoint'] = dict(path=str(path), sha256=digest(path))

    watched = {ROOT / 'tools' / name: value for name, value in fixtures.items()}
    for key in ('school_fixture', 'rope_checkpoint', 'fulcrum_checkpoint', 'source_save'):
        if key in report:
            watched[Path(report[key]['path'])] = report[key]['sha256']

    def unchanged():
        if digest(args.exe) != executable_hash:
            raise ValueError('The executable changed during this suite')
        for path, expected in watched.items():
            if digest(path) != expected:
                raise ValueError(f'Test source or input changed during this suite: {path.name}')

    def save():
        temporary = folder / 'summary.tmp'
        temporary.write_text(json.dumps(report, indent=2))
        temporary.replace(folder / 'summary.json')

    save()
    print('Report:', folder / 'summary.json', flush=True)
    build, interrupted = None, False
    for name in selected:
        case = CASES[name]
        script = lift if name.startswith('lift-') else ROOT / 'tools' / case.script
        command = [sys.executable, str(script), '--exe', str(args.exe), '--data', str(args.data), *case.extra]
        if args.local:
            command.append('--local')
        if case.menu:
            command.extend(['--seat-session', str(args.seat_session)])
        normal_menu = name in ('storage', 'host-loss-menu')
        if not normal_menu:
            command.extend(['--delay-ms', str(args.delay_ms), '--jitter-ms', str(args.jitter_ms)])
        if name in ('rope', 'fulcrum'):
            command.extend(['--checkpoint', report[name + '_checkpoint']['path']])
        if name == 'import' and args.source_save:
            command.extend(['--source-save', str(args.source_save)])
        log = folder / (name + '.log')
        item = dict(name=name, passed=False, log=str(log), timeout_seconds=case.seconds,
            delay_ms=0 if normal_menu else args.delay_ms,
            jitter_ms=0 if normal_menu else args.jitter_ms)
        report['cases'].append(item)
        save()
        before = {p.resolve() for p in EVIDENCE.iterdir() if p.is_dir()}
        started = time.monotonic()
        print('RUN', name, flush=True)
        try:
            unchanged()
            with log.open('x') as output:
                process = subprocess.Popen(command, cwd=ROOT, stdout=output, stderr=subprocess.STDOUT,
                    creationflags=subprocess.CREATE_NO_WINDOW)
                try:
                    item['exit_code'] = process.wait(timeout=case.seconds)
                except (KeyboardInterrupt, subprocess.TimeoutExpired) as error:
                    interrupted = isinstance(error, KeyboardInterrupt)
                    stop_owned_process(process)
                    raise RuntimeError('Interrupted' if interrupted else 'Core case exceeded its time limit') from error
            if item['exit_code'] != 0:
                raise RuntimeError('Core case failed; inspect the private log')
            item.update(collect_results(log, case, not args.local, build, before))
            build = item['build']
            unchanged()
            item['passed'] = True
        except (OSError, ValueError, RuntimeError, KeyError, TypeError, subprocess.SubprocessError) as error:
            item['error'] = str(error)
            if log.exists():
                item['failure'] = failure_from_log(log, True)
        item['seconds'] = round(time.monotonic() - started, 2)
        save()
        print('PASS' if item['passed'] else 'FAIL', name, item['seconds'], flush=True)
        if not item['passed']:
            break
    passed = len(report['cases']) == len(selected) and all(c['passed'] for c in report['cases'])
    report.update(status='interrupted' if interrupted else 'complete' if passed else 'failed',
        build=build, passed=passed)
    save()
    return 130 if interrupted else 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
