"""Measure two rendered clients on Anode; no input, screenshots or game-state edits."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
import uuid
from test_coop_play import ROOT, read

def network_summary(rows):
    result = {}
    for key in ('round_trip_ms', 'oldest_pending_ms', 'buffered_control_ms',
                'queued_input_samples', 'unacknowledged_frames'):
        values = sorted(r['network'][key] for r in rows
                        if r.get('network') and r['network'].get(key) is not None)
        if values:
            result[key] = dict(median=values[len(values)//2],
                p95=values[int(len(values)*.95)], maximum=values[-1])
    return result

def run(args):
    folder = ROOT / 'private/coop' / ('performance-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    invite = folder / 'invite.txt'
    processes, logs, samples = {}, [], {'host': [], 'guest': []}
    try:
        for role in samples:
            profile = folder / role
            profile.mkdir()
            log = (folder / (role + '.log')).open('w')
            logs.append(log)
            command = [str(args.exe.resolve()), '--data', str(args.data.resolve()), '--map', args.map,
                '--no-audio', '--profile', str(profile), '--coop-' + ('host' if role == 'host' else 'join'),
                str(invite), '--coop-delay-ms', str(args.delay_ms), '--coop-jitter-ms', str(args.jitter_ms)]
            if args.local and role == 'host':
                command.append('--coop-local')
            processes[role] = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
                env=dict(os.environ, LOOKING_GLASS_COOP_DIAGNOSTICS='0' if args.metrics_only else '1',
                         LOOKING_GLASS_COOP_METRICS='1'),
                creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
            deadline = time.monotonic() + 90
            while role == 'host' and not invite.exists():
                assert processes[role].poll() is None, 'Host exited; inspect ' + str(folder)
                assert time.monotonic() < deadline, 'Invite timeout'
                time.sleep(.05)
        started, deadline = None, time.monotonic() + 90
        previous = {}
        while True:
            exits = {role: p.poll() for role, p in processes.items()}
            assert all(code is None for code in exits.values()), f'Client exited: {exits}; inspect {folder}'
            for role in samples:
                report = read(folder / role / ('coop-performance.json' if args.metrics_only else 'coop-live.json'))
                if not report or not report['playing']:
                    continue
                started = started or time.monotonic()
                if args.metrics_only:
                    marker = json.dumps(report, sort_keys=True)
                    if previous.get(role) != marker:
                        previous[role] = marker
                        samples[role].append(dict(timing=report['timing'], network=report['network'], scene=report['scene']))
                    continue
                marker = (report['frame']['tick'], report['frame_timing']['frame_ms'])
                if previous.get(role) == marker:
                    continue
                previous[role] = marker
                samples[role].append(dict(tick=report['frame']['tick'], timing=report['frame_timing'],
                    network=report['network_timing'], snapshot_bytes=len(json.dumps(report['combat'])),
                    scene=report['combat']['stage']['owner']))
            if started and time.monotonic() - started >= args.seconds:
                break
            assert time.monotonic() < deadline + args.seconds, 'No paired gameplay'
            time.sleep(.03)
        summary = {}
        for role, rows in samples.items():
            assert len(rows) >= (min(10, args.seconds / 2) if args.metrics_only else 20), 'Insufficient samples'
            timings = {}
            for key in rows[0]['timing']:
                values = sorted(r['timing'][key] for r in rows)
                timings[key] = dict(median=values[len(values)//2], p95=values[int(len(values)*.95)], maximum=values[-1])
            summary[role] = dict(samples=len(rows), timings=timings,
                network=network_summary(rows),
                scene_samples=sum(bool(r['scene']) for r in rows))
            if args.metrics_only:
                assert not (folder / role / 'coop-live.json').exists(), 'Full-world diagnostics enabled in normal gameplay'
                summary[role]['total_frames'] = sum(r['timing']['frames'] for r in rows)
                summary[role]['frames_over_50_ms'] = sum(r['timing']['over_50_ms'] for r in rows)
                for name, in_scene in [('cutscene', True), ('after_cutscene', False)]:
                    subset = [r for r in rows if bool(r['scene']) == in_scene]
                    summary[role][name] = dict(windows=len(subset),
                        network=network_summary(subset),
                        frames=sum(r['timing']['frames'] for r in subset),
                        frames_over_50_ms=sum(r['timing']['over_50_ms'] for r in subset),
                        worst_ms=max((r['timing']['max_ms'] for r in subset), default=None))
            else:
                summary[role]['snapshot_bytes'] = max(r['snapshot_bytes'] for r in rows)
        result = dict(exe=str(args.exe.resolve()), map=args.map, local=args.local,
            delay_ms=args.delay_ms, jitter_ms=args.jitter_ms, metrics_only=args.metrics_only,
            summary=summary, samples=samples)
        (folder / 'result.json').write_text(json.dumps(result, indent=2))
        print('Evidence:', folder, flush=True)
        print(json.dumps(summary, indent=2), flush=True)
    finally:
        (folder / 'samples.json').write_text(json.dumps(samples, indent=2))
        (folder / 'exits.json').write_text(json.dumps({role: p.poll() for role, p in processes.items()}))
        for p in processes.values():
            if p.poll() is None:
                subprocess.run(['taskkill', '/PID', str(p.pid), '/T', '/F'], capture_output=True)
                p.wait(timeout=10)
        for log in logs:
            log.close()
        invite.unlink(missing_ok=True)

if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--exe', type=Path, required=True)
    p.add_argument('--data', type=Path, required=True)
    p.add_argument('--map', default='gvillage')
    p.add_argument('--seconds', type=float, default=25)
    p.add_argument('--delay-ms', type=int, default=100)
    p.add_argument('--jitter-ms', type=int, default=30)
    p.add_argument('--local', action='store_true')
    p.add_argument('--metrics-only', action='store_true', help='Normal gameplay with compact per-second FPS diagnostics only')
    run(p.parse_args())
