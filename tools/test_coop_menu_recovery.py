"""Anode: real Host/Join failure, cancel and retry without restarting the game.

DNS/HTTP faults use an invalid hostname or an owned loopback responder. Final
hosting/joining uses a real public invite. Failures never rewrite gameplay state;
the host setup fault is a temporary file in this run's isolated profile only.
"""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import time

from test_coop_menu import Run, wait
from test_coop_play import ROOT, read


class RecoveryRun(Run):
    def host_children(self):
        query = f"@(Get-CimInstance Win32_Process -Filter 'ParentProcessId = {self.processes['host'].pid}' | Select-Object ProcessId,Name,CreationDate) | ConvertTo-Json -Compress"
        checked = subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-Command',query],
            capture_output=True, text=True, check=True, timeout=15, creationflags=subprocess.CREATE_NO_WINDOW)
        result = json.loads(checked.stdout or '[]')
        return result if isinstance(result, list) else [result]

    def result_after(self, role, count, error=None):
        def result():
            rows = self.events(role, 'coop-result')
            return rows[-1] if len(rows) > count else None
        receipt = wait(result, 90, role + ' connection result')
        if error is None:
            assert receipt['error'] is None, receipt
        else:
            assert receipt['error'] and error in receipt['error'], receipt
        self.button(role, 'Paste link' if role == 'guest' else 'Host this game')
        return receipt

    def failed_dns(self):
        link = 'https://looking-glass-invite.invalid/#lg-coop=1.' + 'a'*64
        self.join_field(link)
        count = len(self.events('guest', 'coop-result'))
        started = time.monotonic()
        self.click('guest', 'Join')
        self.result_after('guest', count, "Cannot resolve the invite's internet address")
        assert self.events('guest', 'invite-input')[-1]['sha256'] == hashlib.sha256(link.encode()).hexdigest()
        self.evidence['dns_failure_seconds'] = round(time.monotonic()-started, 3)
        print('PASS real DNS failure returns to Join with the invite retained', flush=True)

    def handshake_fault(self, reject):
        with socket.socket() as listener:
            listener.bind(('127.0.0.1', 0))
            listener.listen(1)
            listener.settimeout(5)
            self.join_field(f'http://127.0.0.1:{listener.getsockname()[1]}/#lg-coop=1.' + 'b'*64)
            count = len(self.events('guest', 'coop-result'))
            self.click('guest', 'Join')
            connection, _ = listener.accept()
            with connection:
                connection.settimeout(5)
                header = b''
                while not header.endswith(b'\r\n\r\n'):
                    chunk = connection.recv(1)
                    assert chunk, 'Guest closed before requesting its handshake'
                    header += chunk
                    assert len(header) < 8192, 'Unbounded handshake headers'
                if reject:
                    body = b'private responder detail must not reach the menu'
                    connection.sendall(b'HTTP/1.1 503 Service Unavailable\r\nConnection: close\r\nContent-Length: '
                                       + str(len(body)).encode() + b'\r\n\r\n' + body)
                    receipt = self.result_after('guest', count, 'HTTP 503')
                    assert body.decode() not in receipt['error']
                    self.evidence['http_rejection'] = True
                else:
                    started = time.monotonic()
                    self.click('guest', 'Cancel')
                    connection.settimeout(1.5)
                    try:
                        assert connection.recv(1) == b'', 'Cancelled guest kept sending data'
                    except ConnectionResetError:
                        pass
                    self.result_after('guest', count)
                    self.evidence['cancelled_handshake_seconds'] = round(time.monotonic()-started, 3)
        print('PASS HTTP rejection returns to Join' if reject else 'PASS Cancel closes the actual pending connection', flush=True)

    def host_setup_recovery(self, solo):
        children_before = self.host_children()
        obstacle = self.folder / 'host/coop'
        assert not obstacle.exists()
        obstacle.write_bytes(b'Owned setup-failure fixture; remove before retry')
        try:
            count = len(self.events('host', 'coop-result'))
            self.click('host', 'Host this game')
            # A real filesystem error before tunnel launch, not a fabricated
            # menu result or an intentionally rate-limited public request.
            receipt = wait(lambda: self.events('host', 'coop-result')[count:], 30, 'host setup failure')[0]
            assert receipt['error'] is not None
            self.button('host', 'Host this game')
        finally:
            obstacle.unlink()
        assert solo.read_bytes() == self.solo_bytes
        assert not (self.folder / 'host/coop-checkpoint.json').exists(), 'Failed setup wrote a co-op checkpoint'
        print('PASS failed Host setup returns to the same menu with the solo save intact', flush=True)

        count = len(self.events('host', 'coop-result'))
        self.click('host', 'Host this game')
        self.click('host', 'Cancel')
        self.result_after('host', count)
        # The cancelled host must finish its own helper cleanup before retry.
        children_after = self.host_children()
        self.evidence['host_children'] = dict(before=children_before, after_cancel=children_after)
        (self.folder / 'host-children.json').write_text(json.dumps(self.evidence['host_children'], indent=2))
        # Windows may give the game its own conhost process at startup. That
        # unchanged baseline is not a tunnel/download helper created by Host.
        assert all(child in children_before for child in children_after), 'Cancelled host left a new child helper running'
        assert solo.read_bytes() == self.solo_bytes
        assert not (self.folder / 'host/coop-checkpoint.json').exists(), 'Cancelled setup wrote a co-op checkpoint'
        self.evidence['host_failure_and_cancel'] = True
        print('PASS Cancel Host closes its helper and allows another attempt', flush=True)

    def run(self):
        self.evidence = {}
        for name in ('test_coop_menu_recovery.py', 'test_coop_menu.py'):
            source = (ROOT / 'tools' / name).read_bytes()
            (self.folder / name).write_bytes(source)
            self.evidence[name + '_sha256'] = hashlib.sha256(source).hexdigest()
        with (self.folder / 'prepare.log').open('w') as log:
            subprocess.run([str(self.args.exe.resolve()), '--data', str(self.args.data.resolve()),
                '--no-audio', '--profile', str(self.folder / 'host'), '--coop-import-check'],
                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=120, check=True,
                creationflags=subprocess.CREATE_NO_WINDOW)
        solo = self.folder / 'host/saves/slot1.json'
        self.solo_bytes = solo.read_bytes()
        self.start('host')
        self.start('guest')
        original_pids = {role: p.pid for role,p in self.processes.items()}
        self.click('guest', 'Action(Coop)')
        self.click('guest', 'Join')
        self.failed_dns()
        self.handshake_fault(reject=True)
        self.handshake_fault(reject=False)
        self.click('host', 'Page(LoadSave)')
        self.click('host', 'Slot(0)')
        self.click('host', 'LoadSelection')
        self.screen('host', 'solo-paused')
        self.key('host', 0x1b)
        self.click('host', 'Action(Coop)')
        self.click('host', 'Host')
        self.host_setup_recovery(solo)
        self.click('host', 'Host this game')
        self.copy_join()
        self.connected(read(self.folder / 'host/coop-import-expected.json'))
        self.key('host', 0x1b)
        self.click('host', 'Save checkpoint')
        saved = wait(lambda: self.events('host', 'checkpoint-save'))[-1]
        assert saved['error'] is None
        checkpoint = self.folder / 'host/coop-checkpoint.json'
        checkpoint_bytes = checkpoint.read_bytes()
        self.leave()
        assert solo.read_bytes() == self.solo_bytes
        assert checkpoint.read_bytes() == checkpoint_bytes
        assert original_pids == {role: p.pid for role,p in self.processes.items()}
        assert all(p.poll() is None for p in self.processes.values())
        assert len(self.events('host', 'solo-after-coop')) == 1
        self.evidence.update(original_pids=original_pids, solo_sha256=hashlib.sha256(self.solo_bytes).hexdigest(),
                             unchanged_solo_return=self.events('host', 'solo-after-coop')[-1]['sha256'])
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, internet=True,
            native_input=True, build=self.state('host')['build'], evidence=self.evidence,
            scope='Host/Join failure, cancellation and public retry from a loaded solo save; not campaign certification'), indent=2))
        print('PASS same running games recover through public Host/Join, save/leave and exact solo return', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--seat-session', type=int, required=True)
    args = parser.parse_args()
    args.local = False
    run = RecoveryRun(args)
    try:
        run.run()
    finally:
        run.close()
