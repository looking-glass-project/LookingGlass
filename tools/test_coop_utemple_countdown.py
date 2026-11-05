"""Verify a shared wipe, then guest restarts during both underwater death countdowns.

Run in Anode. The existing temple recovery fixture supplies ordinary swimming,
real drowning, camera/countdown assertions and an initial saved-checkpoint wipe.
Only the owned guest process is terminated; no live state or resources are edited.
"""
import argparse
import hashlib
import json
from pathlib import Path
import time

from coop_swim_rejoin import verify_swim_rejoin, verify_first_frame
from test_coop_play import ROOT, read
from test_coop_utemple_recovery import TempleRecovery


class CountdownTemple(TempleRecovery):
    evidence_prefix = 'temple-countdown-'

    def __init__(self, args):
        super().__init__(args)
        self.evidence['countdown_rejoins'] = {}
        self.evidence['sources'] = {}
        for name in ('test_coop_utemple_countdown.py', 'coop_swim_rejoin.py',
                     'test_coop_utemple_recovery.py', 'test_coop_utemple.py',
                     'test_coop_village.py', 'test_coop_play.py', 'coop_recovery.py'):
            source = (ROOT / 'tools' / name).read_bytes()
            (self.folder / name).write_bytes(source)
            self.evidence['sources'][name] = hashlib.sha256(source).hexdigest()

    def live(self, role, predicate, seconds, label):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            process = self.processes[role]
            assert process.poll() is None, f'{role} exited during {label}: {self.log_paths[role]}'
            report = read(self.folder / role / 'coop-live.json')
            if report and report['pid'] == process.pid and predicate(report):
                return report
            self.pump()
        raise AssertionError('Timed out waiting for ' + label)

    def on_drowned(self, victim):
        index = 0 if victim == 'host' else 1
        self.until(lambda: 700 <= self.reports['host']['frame']['lives']['states'][index]['Spectating']['ticks_left'] <= 1000, 10)
        guest = self.processes['guest']
        old_pid = guest.pid
        assert guest.poll() is None
        guest.kill()
        guest.wait(timeout=10)
        self.processes.pop('guest')
        self.reports.pop('guest')
        before = self.live('host', lambda r: not r['playing'], 15, 'host waiting after guest loss')
        end = time.monotonic() + .4
        while time.monotonic() < end:
            self.pump()
        held = self.live('host', lambda r: not r['playing'], 5, 'held underwater state')
        path = self.folder / (victim + '-countdown-rejoin.json')
        record = dict(passed=False, before=before, held=held)
        path.write_text(json.dumps(record))
        assert before['frame'] == held['frame'] and before['combat'] == held['combat'], 'Waiting host advanced underwater state'
        self.start('guest')
        after = self.live('guest', lambda r: r['playing'], 60, 'fresh underwater guest')
        record['after'] = after
        path.write_text(json.dumps(record))
        proof = verify_swim_rejoin(before, held, after, index, old_pid,
                                  expected_generation=self.generation)
        key = after['frame']['epoch'], after['frame']['tick']
        matching_host = self.live('host', lambda r: any((s['epoch'], s['tick']) == key
            for s in r['route_driver']['network_states']), 5, 'exact first-frame host evidence')
        record['matching_host'] = matching_host
        path.write_text(json.dumps(record))
        proof['first_frame'] = verify_first_frame(matching_host, after)
        self.evidence['countdown_rejoins'][victim] = proof
        record.update(passed=True, evidence=proof)
        path.write_text(json.dumps(record))
        # Replace stale pre-disconnect observations before the original recovery
        # verifier continues. The driver's movement commands remain unchanged.
        self.pump()
        assert self.reports['guest']['pid'] != old_pid
        print('PASS underwater countdown guest restart:', victim,
              {k: proof[k] for k in ('before_tick', 'after_tick', 'remaining_before', 'remaining_after')}, flush=True)

    def start_pair(self):
        super().start_pair()
        invite = self.invite.read_text().strip()
        assert invite.startswith('http://127.0.0.1:' if self.args.local else 'https://') and '/#lg-coop=1.' in invite, 'Wrong invite network scope'

    def run(self):
        super().run()
        assert set(self.evidence['countdown_rejoins']) == {'host', 'guest'}
        result = read(self.folder / 'result.json')
        result['scope'] = 'Both real underwater countdown guest restarts, exact first-frame/body/air/world transfer, normal revival and saved temple wipe; no full route'
        result['evidence'] = self.evidence
        (self.folder / 'result.json').write_text(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not all(0 <= n <= 500 for n in (args.delay_ms, args.jitter_ms)):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = CountdownTemple(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failure-report.json').write_text(json.dumps(dict(passed=False,
            error=str(error), build=pair.build, reports=pair.reports), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
