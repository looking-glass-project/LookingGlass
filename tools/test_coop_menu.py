"""Anode-only Windows acceptance test of the real main/ESC co-op menus.

Uses native input against observed button rectangles and passive UI receipts.
No gameplay commands, synthetic state changes, image lookups or test-only menu
actions. Public internet hosting is the default; provider failures fail the run.
--local isolates menu behavior and is never credited as internet verification.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import time
import uuid

from test_coop_play import ROOT, read


class Key(c.Structure):
    _fields_ = [('vk', w.WORD), ('scan', w.WORD), ('flags', w.DWORD),
                ('time', w.DWORD), ('extra', c.c_size_t)]


class Mouse(c.Structure):
    _fields_ = [('dx', w.LONG), ('dy', w.LONG), ('data', w.DWORD),
                ('flags', w.DWORD), ('time', w.DWORD), ('extra', c.c_size_t)]


class Payload(c.Union):
    _fields_ = [('key', Key), ('mouse', Mouse)]


class Input(c.Structure):
    _fields_ = [('kind', w.DWORD), ('payload', Payload)]


def wait(condition, timeout=30, label='UI response'):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        value = condition()
        if value:
            return value
        time.sleep(.05)
    raise AssertionError('Timed out waiting for ' + label)


class Desktop:
    def __init__(self):
        self.user = c.WinDLL('user32', use_last_error=True)
        self.kernel = c.WinDLL('kernel32', use_last_error=True)
        self.callback = c.WINFUNCTYPE(w.BOOL, w.HWND, w.LPARAM)
        declarations = {
            'EnumWindows': ([self.callback, w.LPARAM], w.BOOL),
            'GetWindowThreadProcessId': ([w.HWND, c.POINTER(w.DWORD)], w.DWORD),
            'IsWindowVisible': ([w.HWND], w.BOOL),
            'SetForegroundWindow': ([w.HWND], w.BOOL),
            'GetForegroundWindow': ([], w.HWND),
            'ClientToScreen': ([w.HWND, c.POINTER(w.POINT)], w.BOOL),
            'SetCursorPos': ([c.c_int, c.c_int], w.BOOL),
            'GetCursorPos': ([c.POINTER(w.POINT)], w.BOOL),
            'GetWindowRect': ([w.HWND, c.POINTER(w.RECT)], w.BOOL),
            'SetWindowPos': ([w.HWND, w.HWND, c.c_int, c.c_int, c.c_int, c.c_int, w.UINT], w.BOOL),
            'WindowFromPoint': ([w.POINT], w.HWND),
            'MapVirtualKeyW': ([w.UINT, w.UINT], w.UINT),
            'SendInput': ([w.UINT, c.POINTER(Input), c.c_int], w.UINT),
            'OpenClipboard': ([w.HWND], w.BOOL), 'CloseClipboard': ([], w.BOOL),
            'GetClipboardData': ([w.UINT], w.HANDLE),
            'GetClipboardSequenceNumber': ([], w.DWORD),
            'PostMessageW': ([w.HWND, w.UINT, w.WPARAM, w.LPARAM], w.BOOL),
        }
        for name, (args, result) in declarations.items():
            function = getattr(self.user, name)
            function.argtypes, function.restype = args, result
        self.kernel.GlobalLock.argtypes, self.kernel.GlobalLock.restype = [w.HANDLE], c.c_void_p
        self.kernel.GlobalUnlock.argtypes = [w.HANDLE]

    def window(self, pid):
        found = []
        @self.callback
        def visit(hwnd, _):
            owner = w.DWORD()
            self.user.GetWindowThreadProcessId(hwnd, c.byref(owner))
            if owner.value == pid and self.user.IsWindowVisible(hwnd):
                found.append(hwnd)
            return True
        self.user.EnumWindows(visit, 0)
        return found[0] if found else None

    def focus(self, process):
        assert process.poll() is None, 'Owned game exited'
        hwnd = wait(lambda: self.window(process.pid), label='owned game window')
        self.user.SetForegroundWindow(hwnd)
        if self.user.GetForegroundWindow() != hwnd and os.environ.get('ANODE_LEASE_TOKEN'):
            # Job processes can be denied foreground activation. Ask the seat
            # to focus this freshly observed owned window using the same lease.
            def anode(*args):
                return json.loads(subprocess.check_output(['anode', *args, '--json'],
                    creationflags=subprocess.CREATE_NO_WINDOW, timeout=15))
            windows = anode('windows', '--pid', str(process.pid))['windows']
            window = next(w for w in windows if w['className'] == 'MINIQUADAPP')
            anode('window', window['windowId'], 'focus')
            wait(lambda: self.user.GetForegroundWindow() == hwnd, 3, 'owned window focus')
        if self.user.GetForegroundWindow() != hwnd:
            # Windows may reject programmatic foreground activation. Raise our
            # own window and click its observed title bar, as a user would.
            rect, client = w.RECT(), w.POINT(0, 0)
            assert self.user.GetWindowRect(hwnd, c.byref(rect))
            assert self.user.ClientToScreen(hwnd, c.byref(client))
            assert client.y - rect.top > 8, 'UI fixture requires a windowed game'
            assert self.user.SetWindowPos(hwnd, w.HWND(-1), 0, 0, 0, 0, 0x53)
            try:
                point = w.POINT((rect.left+rect.right)//2, (rect.top+client.y)//2)
                assert self.user.WindowFromPoint(point) == hwnd, 'Owned title bar is obscured'
                assert self.user.SetCursorPos(point.x, point.y)
                self.send([Input(0, Payload(mouse=Mouse(0, 0, 0, 2, 0, 0)))])
                time.sleep(.08)
                self.send([Input(0, Payload(mouse=Mouse(0, 0, 0, 4, 0, 0)))])
            finally:
                self.user.SetWindowPos(hwnd, w.HWND(-2), 0, 0, 0, 0, 0x13)
        wait(lambda: self.user.GetForegroundWindow() == hwnd, 3, 'owned window focus')
        return hwnd

    def send(self, events):
        array = (Input * len(events))(*events)
        assert self.user.SendInput(len(array), array, c.sizeof(Input)) == len(array), 'Native input failed'

    def key(self, process, vk, hold=.08, control=False):
        self.focus(process)
        keys = ([0x11] if control else []) + [vk]
        for key in keys:
            extended = int(key in (0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2D, 0x2E))
            self.send([Input(1, Payload(key=Key(0, self.user.MapVirtualKeyW(key, 0), 8 | extended, 0, 0)))])
        time.sleep(hold)
        for key in reversed(keys):
            extended = int(key in (0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2D, 0x2E))
            self.send([Input(1, Payload(key=Key(0, self.user.MapVirtualKeyW(key, 0), 10 | extended, 0, 0)))])
        time.sleep(.12)

    def type_burst(self, process, text):
        self.focus(process)
        events = []
        for char in text:
            events.extend(Input(1, Payload(key=Key(0, ord(char), flags, 0, 0))) for flags in (4, 6))
        self.send(events)

    def click(self, process, rect):
        hwnd = self.focus(process)
        x, y, width, height = rect
        point = w.POINT(round(x + width / 2), round(y + height / 2))
        assert self.user.ClientToScreen(hwnd, c.byref(point))
        assert self.user.SetCursorPos(point.x, point.y)
        time.sleep(.1)
        actual = w.POINT()
        assert self.user.GetCursorPos(c.byref(actual)), 'Could not verify the test pointer'
        assert (actual.x, actual.y) == (point.x, point.y), (
            f'Test pointer moved or was clipped: requested {(point.x, point.y)}, actual {(actual.x, actual.y)}')
        assert self.user.GetForegroundWindow() == hwnd and self.user.WindowFromPoint(point) == hwnd, 'Test button is outside the focused owned window'
        self.send([Input(0, Payload(mouse=Mouse(0, 0, 0, 2, 0, 0)))])
        time.sleep(.08)
        self.send([Input(0, Payload(mouse=Mouse(0, 0, 0, 4, 0, 0)))])
        time.sleep(.15)
        return dict(pid=process.pid, rect=rect, screen_point=[point.x, point.y],
                    actual_pointer=[actual.x, actual.y])

    def copied_text(self, sequence):
        # Read only after our Copy invite action changes this session's clipboard.
        wait(lambda: self.user.GetClipboardSequenceNumber() != sequence, label='copied invite')
        wait(lambda: self.user.OpenClipboard(None), 3, 'clipboard availability')
        try:
            handle = self.user.GetClipboardData(13)
            pointer = self.kernel.GlobalLock(handle)
            assert pointer, 'No copied text'
            try:
                return c.wstring_at(pointer)
            finally:
                self.kernel.GlobalUnlock(handle)
        finally:
            self.user.CloseClipboard()


class Run:
    def __init__(self, args):
        assert os.name == 'nt', 'Run this test in the Windows Anode desktop'
        session = w.DWORD()
        assert c.WinDLL('kernel32').ProcessIdToSessionId(os.getpid(), c.byref(session))
        assert session.value == args.seat_session and session.value != 0, 'Wrong desktop session'
        self.args, self.desktop = args, Desktop()
        self.folder = ROOT / 'private/coop' / ('menu-test-' + uuid.uuid4().hex)
        self.processes, self.logs = {}, []
        for role in ('host', 'guest'):
            (self.folder / role / 'saves').mkdir(parents=True)
        print('Evidence:', self.folder, flush=True)

    def start(self, role):
        log = (self.folder / (role + '.log')).open('w')
        self.logs.append(log)
        self.processes[role] = subprocess.Popen([str(self.args.exe.resolve()), '--data',
            str(self.args.data.resolve()), '--no-audio', '--profile', str(self.folder / role)],
            cwd=ROOT, env=dict(os.environ, LOOKING_GLASS_UI_PROBE='1',
                LOOKING_GLASS_UI_PROBE_LOCAL='1' if self.args.local else '0'), stdout=log,
            stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW)
        self.screen(role, 'menu-Main')

    def state(self, role):
        assert self.processes[role].poll() is None, f'{role} exited; inspect its log'
        report = read(self.folder / role / 'ui-live.json')
        return report if report and report['pid'] == self.processes[role].pid else {}

    def screen(self, role, screen):
        return wait(lambda: self.state(role).get('screen') == screen, label=role + ' ' + screen)

    def button(self, role, label, timeout=30):
        def find():
            if role == 'host' and label == 'Copy invite':
                messages = self.events(role, 'coop-message')
                assert not messages or not messages[-1].startswith('Could not continue co-op:'), messages[-1]
            return next((b for b in self.state(role).get('buttons', []) if b['label'] == label), None)
        return wait(find, timeout, role + ' button ' + label)

    def click(self, role, label, timeout=30):
        # Focus can change the window's native geometry. Read the passive
        # button receipt afterwards, and verify the actual target before input.
        self.desktop.focus(self.processes[role])
        receipt = self.desktop.click(self.processes[role], self.button(role, label, timeout)['rect'])
        with (self.folder / 'ui-input.jsonl').open('a') as output:
            output.write(json.dumps(dict(role=role, label=label, **receipt)) + '\n')

    def key(self, role, vk, **kwargs):
        self.desktop.key(self.processes[role], vk, **kwargs)

    def events(self, role, event):
        path = self.folder / role / 'ui-events.jsonl'
        if not path.exists():
            return []
        rows = []
        for line in path.read_text().splitlines():
            try:
                row = json.loads(line)
            except ValueError:
                continue  # A concurrent append may not have completed its last line.
            if row['event'] == event and row['pid'] == self.processes[role].pid:
                rows.append(row['value'])
        return rows

    def join_field(self, text):
        self.key('guest', 0x41, control=True)
        self.desktop.type_burst(self.processes['guest'], text)
        digest = hashlib.sha256(text.encode()).hexdigest()
        wait(lambda: self.events('guest', 'invite-input')[-1]['sha256'] == digest,
            label='ordered typed invite')

    def connected(self, expected):
        def ready():
            reports = [read(self.folder / role / 'coop-live.json') for role in ('host', 'guest')]
            return reports if all(r and r['playing'] for r in reports) else None
        reports = wait(ready, 90, 'both clients playing')
        for report in reports:
            assert report['visit']['map'] == expected['visit']['map']
            for body, saved in zip(report['frame']['players'], expected['frame']['players']):
                for field in ('sanity', 'will', 'owned', 'collected'):
                    assert body['stats'][field] == saved['stats'][field], 'Menu session lost resources'
                assert sum((a-b)**2 for a,b in zip(body['player']['feet'],saved['player']['feet'])) < 1
            assert report['combat']['stage']['world']['used'] == expected['frame']['combat']['stage']['world']['used']

    def copy_join(self):
        sequence = self.desktop.user.GetClipboardSequenceNumber()
        self.click('host', 'Copy invite', 150)
        link = self.desktop.copied_text(sequence)
        origin = 'http://127.0.0.1:' if self.args.local else 'https://'
        assert link.startswith(origin) and '/#lg-coop=1.' in link, 'Wrong invite network scope'
        self.key('guest', 0x41, control=True)
        self.key('guest', 0x56, control=True)
        wait(lambda: self.events('guest', 'invite-input')[-1]['sha256'] == hashlib.sha256(link.encode()).hexdigest(),
            label='pasted host invite')
        self.click('guest', 'Join')
        return link

    def leave(self):
        if self.state('host')['screen'] == 'coop-play':
            self.key('host', 0x1b)
        self.click('host', 'Leave co-op')
        self.button('host', 'Continue co-op')
        wait(lambda: self.events('guest', 'coop-message')[-1].startswith('Could not continue co-op:'),
            25, 'guest host-left menu error')
        self.click('host', 'Back')
        self.button('host', 'Host')
        self.click('host', 'Back')
        self.screen('host', 'menu-Main')
        before = self.events('host', 'solo-before-coop')[-1]
        after = self.events('host', 'solo-after-coop')[-1]
        assert before['sha256'] == after['sha256'], 'Co-op changed the suspended solo game'

    def run(self):
        with (self.folder / 'prepare.log').open('w') as log:
            subprocess.run([str(self.args.exe.resolve()), '--data', str(self.args.data.resolve()),
                '--no-audio', '--profile', str(self.folder / 'host'), '--coop-import-check'],
                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=120, check=True)
        source = self.folder / 'host/saves/slot1.json'
        source_bytes = source.read_bytes()
        self.start('host')
        self.start('guest')
        self.click('guest', 'Action(Coop)')
        self.click('guest', 'Join')
        self.join_field('invalid-coop-link')
        self.click('guest', 'Join')
        wait(lambda: self.events('guest', 'coop-message')[-1] == 'Paste the complete invite link from the host.')
        # Accept a TCP connection that never completes its WebSocket handshake.
        # The normal connecting screen must remain cancellable.
        with socket.socket() as pending:
            pending.bind(('127.0.0.1', 0))
            pending.listen(1)
            self.join_field(f'http://127.0.0.1:{pending.getsockname()[1]}/#lg-coop=1.' + 'a'*64)
            self.click('guest', 'Join')
            self.click('guest', 'Cancel')
            self.button('guest', 'Paste link')
        print('PASS Join UI: batched typing, malformed-link feedback and cancelled connection', flush=True)
        self.click('host', 'Page(LoadSave)')
        self.click('host', 'Slot(0)')
        self.click('host', 'LoadSelection')
        self.screen('host', 'solo-paused')
        self.key('host', 0x1b)
        self.click('host', 'Action(Coop)')
        self.click('host', 'Host')
        self.click('host', 'Host this game')
        first_link = self.copy_join()
        self.connected(read(self.folder / 'host/coop-import-expected.json'))
        self.key('host', 0x1b)
        self.click('host', 'Save checkpoint')
        receipt = wait(lambda: self.events('host', 'checkpoint-save'))[-1]
        assert receipt['error'] is None, receipt
        checkpoint = self.folder / 'host/coop-checkpoint.json'
        saved = checkpoint.read_bytes()
        self.leave()
        assert checkpoint.read_bytes() == saved, 'Leaving replaced the manual co-op save'
        assert source.read_bytes() == source_bytes, 'Co-op changed the solo slot'
        print('PASS loaded solo -> ESC Host this game -> menu save/leave -> exact solo return', flush=True)
        # The expired link remains in the guest field. It must fail back
        # into that same menu, allowing replacement with the next session link.
        old_results = len(self.events('guest', 'coop-result'))
        self.click('guest', 'Join')
        wait(lambda: len(self.events('guest', 'coop-result')) > old_results and
            self.events('guest', 'coop-result')[-1]['error'] is not None and
            any(b['label'] == 'Paste link' for b in self.state('guest').get('buttons', [])) and
            self.events('guest', 'coop-message')[-1].startswith('Could not continue co-op:'),
            90, 'expired invite feedback')
        self.click('host', 'Action(Resume)')
        self.screen('host', 'solo-play')
        self.key('host', 0x57, hold=.3)
        self.key('host', 0x1b)
        self.click('host', 'Action(Coop)')
        before = self.events('host', 'solo-before-coop')
        assert before[-1]['feet'] != before[-2]['feet'], 'Returned solo game did not accept movement'
        self.click('host', 'Host')
        self.click('host', 'Continue co-op')
        assert self.copy_join() != first_link, 'Rehosting reused an expired invite'
        self.connected(json.loads(saved)['payload'])
        self.leave()
        assert source.read_bytes() == source_bytes, 'Rehosting changed the solo slot'
        assert len(self.events('host', 'solo-after-coop')) == 2
        result = dict(passed=True, internet=not self.args.local, native_input=True,
            source_sha256=hashlib.sha256(source_bytes).hexdigest(),
            build=self.state('host')['build'], solo_returns=2,
            cases=['ordered typing', 'malformed link', 'cancel join', 'load solo through menu',
                   'host current', 'save checkpoint', 'leave', 'unchanged solo return',
                   'solo movement', 'expired invite', 'continue co-op', 'fresh invite', 'rejoin'])
        (self.folder / 'result.json').write_text(json.dumps(result, indent=2))
        print('PASS expired invite -> new session -> Continue co-op -> second exact solo return', flush=True)

    def close(self):
        for process in self.processes.values():
            if process.poll() is None:
                hwnd = self.desktop.window(process.pid)
                if hwnd:
                    self.desktop.user.PostMessageW(hwnd, 0x10, 0, 0)
                try:
                    process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True)
                    process.wait(timeout=10)
        for log in self.logs:
            log.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--seat-session', type=int, required=True, help='Session ID reported by Anode seat_status')
    parser.add_argument('--local', action='store_true', help='Loopback UI isolation; never counts as internet proof')
    args = parser.parse_args()
    run = Run(args)
    try:
        run.run()
    finally:
        run.close()
