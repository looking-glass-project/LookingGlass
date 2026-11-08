"""Run separate host/guest processes with isolated profiles through real sockets.

Add --internet to route through a real public HTTPS tunnel. This tests transport,
profile isolation and cleanup; it does not certify gameplay or campaign support.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]


def run(exe, internet=False, delay=0, jitter=0, join_delay=0):
    folder = ROOT / 'private' / 'coop' / ('network-test-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    invite = folder / 'invite.txt'
    common = ['--invite-file', str(invite), '--seconds', '90', '--delay-ms', str(delay), '--jitter-ms', str(jitter)]
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    with (folder / 'host.log').open('w') as host_log, (folder / 'guest.log').open('w') as guest_log:
        host = subprocess.Popen([str(exe), '--coop-probe-host', '--profile', str(folder / 'host'), *common,
                                 *(['--internet'] if internet else [])], cwd=ROOT, stdout=host_log, stderr=subprocess.STDOUT, creationflags=flags)
        guest = None
        try:
            deadline = time.monotonic() + 150
            while not invite.exists():
                if host.poll() is not None:
                    raise RuntimeError('Host failed before creating an invite: ' + (folder / 'host.log').read_text())
                if time.monotonic() > deadline:
                    raise TimeoutError('Host invite did not become ready')
                time.sleep(.05)
            time.sleep(join_delay)
            guest = subprocess.Popen([str(exe), '--coop-probe-join', '--profile', str(folder / 'guest'), *common],
                                     cwd=ROOT, stdout=guest_log, stderr=subprocess.STDOUT, creationflags=flags)
            assert guest.wait(timeout=100) == 0, (folder / 'guest.log').read_text()
            assert host.wait(timeout=10) == 0, (folder / 'host.log').read_text()
            reports = [json.loads((folder / role / 'result.json').read_text()) for role in ('host', 'guest')]
            assert all(report['passed'] and report['exchanges'] == 16 for report in reports)
            assert reports[0]['pid'] != reports[1]['pid']
            assert reports[0]['peer_pid'] == reports[1]['pid'] and reports[1]['peer_pid'] == reports[0]['pid']
            assert all(report['internet'] == internet for report in reports)
            for role, report in zip(('host', 'guest'), reports):
                settings = json.loads((folder / role / 'coop-probe-settings.json').read_text())
                saves = json.loads((folder / role / 'saves' / 'coop-probe-profile.json').read_text())
                assert settings == saves == {'role': role, 'pid': report['pid']}
            print(f'PASS {"internet" if internet else "loopback"}: two processes, isolated profiles, 16 exchanges, {delay} ms delay + {jitter} ms jitter per sender; initial join delay {join_delay}s.')
            print('Private evidence:', folder)
        finally:
            for process in (guest, host):
                if process is not None and process.poll() is None:
                    # Kill only this test's descendants, including its owned tunnel.
                    if os.name == 'nt':
                        subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True)
                    else:
                        process.terminate()
                    process.wait(timeout=10)
            invite.unlink(missing_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--internet', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=0)
    parser.add_argument('--jitter-ms', type=int, default=0)
    parser.add_argument('--join-delay-seconds', type=int, default=0, choices=range(61))
    args = parser.parse_args()
    run(args.exe.resolve(), args.internet, args.delay_ms, args.jitter_ms, args.join_delay_seconds)
