"""Resume a real Castling segment across a deliberate host-process suspension.

Run in Anode. Only this fixture's own host process is suspended, always resumed
in finally. Both clients keep their ordinary controls and the real network path.
"""
import argparse
import ctypes
import hashlib
import json
import statistics
import time
from pathlib import Path

from test_coop_castling_resume import CastlingResume
from test_coop_play import ROOT


def suspend_owned_process(process, seconds):
    assert process.poll() is None, 'Test host exited before suspension'
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    native = ctypes.WinDLL('ntdll')
    kernel.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_uint32]
    kernel.OpenProcess.restype = ctypes.c_void_p
    kernel.CloseHandle.argtypes = [ctypes.c_void_p]
    for fn in (native.NtSuspendProcess, native.NtResumeProcess):
        fn.argtypes = [ctypes.c_void_p]
        fn.restype = ctypes.c_long
    handle = kernel.OpenProcess(0x0800, False, process.pid)
    if not handle:
        raise ctypes.WinError(ctypes.get_last_error())
    suspended = False
    began = time.monotonic()
    try:
        status = native.NtSuspendProcess(handle)
        assert status == 0, f'Could not suspend owned test host: {status}'
        suspended = True
        time.sleep(seconds)
    finally:
        if suspended:
            status = native.NtResumeProcess(handle)
            if status != 0:
                process.terminate()
                raise RuntimeError(f'Could not resume owned test host: {status}')
        kernel.CloseHandle(handle)
    return time.monotonic() - began


def verify_suspension(event):
    assert event and .6 <= event['seconds'] < 2, 'Missing bounded host suspension'
    assert event['reset'] > event['before_reset'], 'Host did not reset suspended controls'
    assert event['ack_tick'] >= event['before_tick'], 'Invalid reset acknowledgement'
    assert event['verified_tick'] - event['ack_tick'] >= 240, 'Too little post-resume simulation'
    samples = event['settled_samples']
    assert len(samples) >= 5 and samples[-1][0]-samples[0][0] >= 96, 'Insufficient recovery window'
    assert all(tick-event['ack_tick'] >= 120 for tick, _ in samples), 'Recovery sampled before settling'
    assert statistics.median(value for _, value in samples) < 100, 'Suspended controls left lasting input delay'


class CastlingSuspend(CastlingResume):
    evidence_prefix = 'castling-suspend-'

    def __init__(self, args):
        self.suspension = None
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'suspend-fixture-source.py').write_bytes(source)
        self.evidence['suspend_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def pump(self):
        super().pump()
        if len(self.reports) != 2:
            return
        host, guest = self.reports['host'], self.reports['guest']
        timing, tick = host['network_timing'], host['frame']['tick']
        if self.suspension is None:
            course = guest['route_driver'].get('castling_course')
            if not course or course['step'] != self.args.suspend_step or course['ticks'] < 12:
                return
            if sum(x*x for x in course['movement']['wish']) < .0001:
                return
            assert host['combat']['stage']['owner'] is None, 'Suspension must test ordinary movement'
            (self.folder / 'before-suspension.json').write_text(json.dumps(host))
            self.suspension = dict(before_tick=tick, before_reset=timing['control_reset'])
            self.suspension['seconds'] = suspend_owned_process(self.processes['host'], .65)
            print('Suspended and resumed owned host for', self.suspension['seconds'], 'seconds', flush=True)
            return
        event = self.suspension
        if timing['control_reset'] > event['before_reset'] and not timing['awaiting_controls']:
            if 'ack_tick' not in event:
                event.update(reset=timing['control_reset'], ack_tick=tick, settled_samples=[])
            samples = event['settled_samples']
            if tick-event['ack_tick'] >= 120 and 'verified_tick' not in event \
                    and (not samples or tick != samples[-1][0]):
                samples.append([tick, timing['buffered_control_ms']])
            if tick-event['ack_tick'] >= 240 and len(samples) >= 5 and 'verified_tick' not in event:
                event.update(verified_tick=tick,
                             buffered_ms=statistics.median(value for _, value in samples))
                verify_suspension(event)
                (self.folder / 'after-suspension.json').write_text(json.dumps(host))
                self.evidence['suspension'] = event

    def verify_network_states(self, field='sha256', metric='matching_sent_and_restored_states'):
        verify_suspension(self.suspension)
        super().verify_network_states(field, metric)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='red')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    parser.add_argument('--start-step', type=int, default=124)
    parser.add_argument('--last-step', type=int, default=129)
    parser.add_argument('--suspend-step', type=int, default=127)
    args = parser.parse_args()
    if not 0 < args.start_step <= args.suspend_step < args.last_step <= 202:
        parser.error('Suspension must be inside a nonzero Castling segment')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    args.save_before, args.rejoin = [], False
    pair = CastlingSuspend(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, suspension=pair.suspension)))
        raise
    finally:
        pair.close()
