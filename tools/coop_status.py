"""Print a bounded, read-only regression status without images or full histories.

Usage: python tools/coop_status.py private/coop/regression-suite-<id>
This observes evidence only; it never creates a pass or changes a running test.
"""
import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def log_tail(path, limit=8192):
    with path.open('rb') as stream:
        stream.seek(0, 2)
        stream.seek(max(0, stream.tell() - limit))
        return stream.read(limit).decode('utf-8', errors='replace')


def summarize(suite):
    summary = json.loads((suite / 'summary.json').read_text())
    result = dict(status=summary['status'], passed=summary['passed'], report=str(suite / 'summary.json'), cases=[])
    for case in summary['cases']:
        item = {k: case[k] for k in ('name', 'passed', 'seconds') if k in case}
        metrics = case.get('metrics', {})
        for key in ('final_health', 'matching_sent_and_restored_states'):
            if key in metrics:
                item[key] = metrics[key]
        if not case['passed']:
            failure = case.get('failure', {})
            item['error'] = failure.get('log_tail', case.get('error', 'Unknown failure'))[-800:]
            item['log'] = case['log']
        result['cases'].append(item)
    if summary['status'] != 'running':
        return result
    done = {case['name'] for case in summary['cases']}
    active = next((name for name in summary['selected'] if name not in done), None)
    if active is None:
        return result
    result['active'] = active
    log = suite / (active + '.log')
    if not log.exists():
        return result
    marks = [line for line in log_tail(log).splitlines()
             if line.startswith(('PASS ', 'FAIL ', 'AssertionError:', 'RuntimeError:'))]
    result['last_event'] = marks[-1][:240] if marks else None
    with log.open(errors='replace') as stream:
        header = stream.read(4096)
    match = re.search(r'^Evidence: (.+)$', header, re.MULTILINE)
    if not match:
        return result
    folder = Path(match.group(1).strip()).resolve()
    if not folder.is_relative_to((ROOT / 'private/coop').resolve()):
        raise ValueError('Reported evidence is outside private/coop')
    result['clients'] = {}
    for role in ('host', 'guest'):
        try:
            report = json.loads((folder / role / 'coop-live.json').read_text())
            driver = report['route_driver']
            state = dict(tick=report['frame']['tick'], phase=driver['phase'], scene=driver['scene'],
                         health=[b['stats']['sanity'] for b in report['frame']['players']])
            timing = report.get('network_timing') or {}
            for key in ('buffered_control_ms', 'round_trip_ms'):
                if timing.get(key) is not None:
                    state[key] = round(timing[key], 1)
            result['clients'][role] = state
        except (OSError, ValueError, KeyError, TypeError):
            # Reports can be between writes/startup; unavailable is not a pass.
            result['clients'][role] = {'status': 'report unavailable'}
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('suite', type=Path)
    args = parser.parse_args()
    try:
        print(json.dumps(summarize(args.suite.resolve())))
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(2, f'Cannot read regression status: {error}\n')
