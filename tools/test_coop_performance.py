"""Measure two rendered clients on Anode, optionally exercising native guest input."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
import uuid
from test_coop_play import ROOT, read

def process_cpu_seconds(process):
    """Separate actual CPU consumption from wall-clock stalls on the shared PC."""
    if os.name != 'nt':
        return None
    import ctypes as c
    from ctypes import wintypes as w
    times = [w.FILETIME() for _ in range(4)]
    api = c.WinDLL('kernel32', use_last_error=True).GetProcessTimes
    api.argtypes = [w.HANDLE] + [c.POINTER(w.FILETIME)] * 4
    api.restype = w.BOOL
    if not api(w.HANDLE(int(process._handle)), *(c.byref(t) for t in times)):
        raise c.WinError(c.get_last_error())
    return sum((t.dwHighDateTime << 32) | t.dwLowDateTime for t in times[2:]) / 10_000_000

class Exercise:
    """Ordinary keyboard/mouse input, only in the explicitly selected test seat."""
    def __init__(self, seat_session):
        from test_coop_menu import Desktop, c, w
        session = w.DWORD()
        assert c.WinDLL('kernel32').ProcessIdToSessionId(os.getpid(), c.byref(session))
        assert seat_session is not None and session.value == seat_session and session.value != 0, 'Wrong desktop session'
        self.desktop, self.held, self.started = Desktop(), None, None
        self.hwnd, self.events = None, 0

    def arrange(self, processes):
        from test_coop_menu import c, w, wait
        user = self.desktop.user
        receipts, x = {}, 8
        for role, process in processes.items():
            assert process.poll() is None, 'Owned client exited before arrangement'
            hwnd = wait(lambda: self.desktop.window(process.pid), label=role + ' window')
            rect = w.RECT()
            assert user.GetWindowRect(hwnd, c.byref(rect))
            width, height = rect.right-rect.left, rect.bottom-rect.top
            assert x+width <= user.GetSystemMetrics(0) and 8+height <= user.GetSystemMetrics(1), 'Use a larger Anode display for two unobscured clients'
            assert user.SetWindowPos(hwnd, None, x, 8, 0, 0, 0x15)  # move only, keep size/z-order/focus
            assert user.GetWindowRect(hwnd, c.byref(rect))
            receipts[role] = [rect.left, rect.top, rect.right, rect.bottom]
            x += width+8
        assert receipts['host'][2] <= receipts['guest'][0], 'Test windows overlap'
        return receipts

    def key(self, key, down):
        from test_coop_menu import Input, Payload, Key
        self.desktop.send([Input(1, Payload(key=Key(0, self.desktop.user.MapVirtualKeyW(key, 0), 8 if down else 10, 0, 0)))])

    def update(self, process):
        from test_coop_menu import Input, Payload, Mouse
        if self.started is None:
            self.hwnd = self.desktop.focus(process)
            self.started = time.monotonic()
        assert self.desktop.user.GetForegroundWindow() == self.hwnd, 'Guest lost focus or left gameplay; inspect host.log and guest.log'
        elapsed = time.monotonic() - self.started
        key = (0x57, None, 0x53, None)[int(elapsed / .75) % 4]
        if key != self.held:
            if self.held is not None:
                self.key(self.held, False)
            self.held = key
            if key is not None:
                self.key(key, True)
        self.desktop.send([Input(0, Payload(mouse=Mouse(2 if int(elapsed / 3) % 2 else -2, 0, 0, 1, 0, 0)))])
        self.events += 1

    def release(self):
        if self.held is not None:
            self.key(self.held, False)
            self.held = None

def network_summary(rows):
    result = {}
    for key in ('round_trip_ms', 'oldest_pending_ms', 'buffered_control_ms',
                'queued_input_samples', 'unacknowledged_frames',
                'snapshot_age_ms', 'last_snapshot_gap_ms'):
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
    desktop = Exercise(args.seat_session) if args.exercise_guest or args.tile_windows else None
    exercise = desktop if args.exercise_guest else None
    placement = None
    latest_guest = None
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
        if args.tile_windows:
            placement = desktop.arrange(processes)
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
                        samples[role].append(dict(timing=report['timing'], network=report['network'], scene=report['scene'],
                            process_cpu_seconds=process_cpu_seconds(processes[role]),
                            work=report.get('work'), local_view=report.get('local_view'), renderer=report.get('renderer'), elapsed=time.monotonic()-started))
                        if role == 'guest':
                            latest_guest = report
                    continue
                marker = (report['frame']['tick'], report['frame_timing']['frame_ms'])
                if previous.get(role) == marker:
                    continue
                previous[role] = marker
                samples[role].append(dict(tick=report['frame']['tick'], timing=report['frame_timing'],
                    network=report['network_timing'], snapshot_bytes=len(json.dumps(report['combat'])),
                    scene=report['combat']['stage']['owner']))
            if exercise and started and time.monotonic()-started > 10 and latest_guest and not latest_guest['scene']:
                exercise.update(processes['guest'])
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
            first, last = rows[0].get('network'), rows[-1].get('network')
            if first and last and 'received_frames' in first:
                summary[role]['delivered_snapshots'] = last['received_frames'] - first['received_frames']
                summary[role]['delivered_bytes'] = last['received_frame_bytes'] - first['received_frame_bytes']
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
        input_result = None
        if exercise:
            exercise.release()
            views = [r['local_view'] for r in samples['guest'] if r.get('local_view') and not r['scene']]
            assert exercise.events > 100 and len(views) > 3, 'Insufficient post-cutscene input exercise'
            yaw_span = max(v['yaw'] for v in views) - min(v['yaw'] for v in views)
            movement = max(sum((a-b)**2 for a,b in zip(v['feet'],views[0]['feet']))**.5 for v in views)
            assert yaw_span > .05, 'Native mouse input did not turn the camera'
            assert movement > 2, 'Native movement did not move guest Alice'
            input_result = dict(mouse_events=exercise.events, yaw_span=yaw_span, movement=movement)
        result = dict(exe=str(args.exe.resolve()), map=args.map, local=args.local, input_exercise=input_result, windows=placement,
            delay_ms=args.delay_ms, jitter_ms=args.jitter_ms, metrics_only=args.metrics_only,
            summary=summary, samples=samples)
        (folder / 'result.json').write_text(json.dumps(result, indent=2))
        print('Evidence:', folder, flush=True)
        print(json.dumps(summary, indent=2), flush=True)
    finally:
        if exercise:
            exercise.release()
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
    p.add_argument('--exercise-guest', action='store_true', help='Move and turn with native input after the opening cutscene')
    p.add_argument('--tile-windows', action='store_true', help='Keep both clients unobscured on a sufficiently wide Anode display')
    p.add_argument('--seat-session', type=int, help='Required Anode session ID when exercising input')
    args = p.parse_args()
    assert not args.exercise_guest or args.metrics_only, 'Native input exercise requires compact metrics'
    run(args)
