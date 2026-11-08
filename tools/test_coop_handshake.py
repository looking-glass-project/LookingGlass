"""Exercise delayed real HTTP upgrade bytes against the production host listener."""
import argparse
import base64
import json
import os
from pathlib import Path
import socket
import subprocess
import time
from urllib.parse import urlsplit
import uuid

ROOT = Path(__file__).resolve().parents[1]


def run(exe):
    folder = ROOT / 'private' / 'coop' / ('handshake-test-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    invite = folder / 'invite.txt'
    results = []
    with (folder / 'host.log').open('w') as log:
        process = subprocess.Popen([str(exe), '--coop-probe-host', '--profile', str(folder / 'host'),
            '--invite-file', str(invite), '--seconds', '30'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        try:
            deadline = time.monotonic() + 10
            while not invite.exists():
                assert process.poll() is None, 'Host ended before creating an invite'
                assert time.monotonic() < deadline, 'Invite timeout'
                time.sleep(.02)
            link = urlsplit(invite.read_text().strip())
            token = link.fragment.removeprefix('lg-coop=1.')
            assert len(token) == 64
            for delay in (.05, .2, .5):
                with socket.create_connection((link.hostname, link.port), timeout=3) as stream:
                    # The TCP connection exists before the HTTP bytes arrive.
                    # A nonblocking accepted socket must not abort the upgrade.
                    time.sleep(delay)
                    request = (f'GET /coop HTTP/1.1\r\nHost: {link.netloc}\r\nUpgrade: websocket\r\n'
                        f'Connection: Upgrade\r\nSec-WebSocket-Version: 13\r\n'
                        f'Sec-WebSocket-Key: {base64.b64encode(os.urandom(16)).decode()}\r\n'
                        f'Sec-WebSocket-Protocol: lookingglass-coop-v1\r\nAuthorization: Bearer {token}\r\n\r\n')
                    try:
                        stream.sendall(request.encode())
                        response = stream.recv(4096).split(b'\r\n', 1)[0].decode(errors='replace')
                    except OSError as error:
                        response = type(error).__name__
                    results.append({'delay_seconds': delay, 'response': response})
                time.sleep(.1)
            passed = all(r['response'].startswith('HTTP/1.1 101 ') for r in results)
            (folder / 'result.json').write_text(json.dumps({'passed': passed, 'attempts': results}, indent=2))
            print('Evidence:', folder)
            assert passed, results
            print('PASS delayed authenticated upgrade at 50, 200 and 500 ms')
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=10)
            invite.unlink(missing_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    run(parser.parse_args().exe.resolve())
