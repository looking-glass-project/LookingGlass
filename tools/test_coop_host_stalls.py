"""Automate brief host stalls, recovery and contact/effects in two real clients.

Run in Anode on Windows. Only the fixture-owned host process is paused;
its guest keeps running through the real network connection. No state writes.
"""
import argparse
import ctypes
from ctypes import wintypes
import json
import hashlib
import os
from pathlib import Path
import statistics
import time

from test_coop_presence import PresencePair


class StallPair(PresencePair):
    evidence_prefix = 'host-stall-test-'

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'host-stall-fixture-source.py').write_bytes(source)
        self.evidence['host_stall_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def run(self):
        super().run()
        path = self.folder/'result.json'
        result = json.loads(path.read_text())
        result['scope'] = 'Controlled host stalls, input recovery, contact and ambient effects; no campaign traversal'
        path.write_text(json.dumps(result, indent=2))
        print('PASS host recovered from controlled stalls before contact/effects checks.', flush=True)

    def start_pair(self):
        super().start_pair()
        self.until(lambda: all('entry' in r['combat']['stage']['world']['village']['cinema']['done']
                               and r['combat']['stage']['owner'] is None
                               for r in self.reports.values()), 120)
        self.wait(1)
        process = self.processes['host']
        assert process.poll() is None
        # Popen's owned process handle identifies this exact child even if it
        # exits. No process-name lookup or PID reuse can affect another app.
        ntdll = ctypes.WinDLL('ntdll')
        suspend, resume = ntdll.NtSuspendProcess, ntdll.NtResumeProcess
        for call in (suspend, resume):
            call.argtypes = [wintypes.HANDLE]
            call.restype = ctypes.c_long
        handle = wintypes.HANDLE(int(process._handle))
        events = []
        for duration in (.35, .4, .2):
            assert process.poll() is None
            before = self.reports['host']['frame']['tick']
            started = time.perf_counter()
            status = suspend(handle)
            assert status >= 0, 'Could not suspend owned host'
            try:
                time.sleep(duration)
            finally:
                # Always resume the same handle before any assertion or read.
                status = resume(handle)
            assert status >= 0, 'Could not resume owned host'
            events.append(dict(requested_ms=duration*1000,
                               measured_ms=(time.perf_counter()-started)*1000,
                               tick_before=before))
            self.wait(1.5)
        first = len(self.history['host'])
        self.wait(3)
        settled = self.history['host'][first:]
        buffer = [r['network_timing']['buffered_control_ms'] for r in settled]
        debt = [r['frame_timing']['simulation_debt_ms'] for r in settled
                if 'simulation_debt_ms' in r['frame_timing']]
        assert len(buffer) >= 8, 'Insufficient post-stall samples'
        self.evidence['host_stalls'] = dict(events=events, settled_samples=len(buffer),
            buffered_median_ms=statistics.median(buffer), buffered_peak_ms=max(buffer),
            remaining_debt_peak_ms=max(debt) if debt else None)
        (self.folder/'host-stalls.json').write_text(json.dumps(self.evidence['host_stalls'],indent=2))
        assert statistics.median(buffer) < 200, f'Permanent input backlog after stalls: {statistics.median(buffer)} ms'
        assert len(debt) >= 8, 'Build has no simulation-debt diagnostics'
        assert max(debt) < 10, f'Host did not drain simulation debt: {max(debt)} ms'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,required=True)
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    args=parser.parse_args()
    if os.name != 'nt':
        parser.error('The host-stall process fixture requires Windows')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    args.first,args.rejoin='blue',False
    pair=StallPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
