"""Both countdown reconnects and moving-platform revivals from a normal save.

Use an unchanged Funhouse checkpoint with both players on the second fulcrum.
Ordinary running input sends each victim off its side. The survivor walks along
the board, tipping it through revival; the revived player stays neutral to prove
passive carry. No health, bodies, clocks, enemies or saves are modified.
"""
import argparse
import hashlib
import json
from pathlib import Path
import time

from coop_fulcrum_recovery import verify_fulcrum_rejoin, verify_fulcrum_recovery
from coop_recovery import read_trace, verify_presence
from coop_swim_rejoin import verify_first_frame
from test_coop_play import ROOT, read
from test_coop_village import Pair


class FulcrumCountdown(Pair):
    map_name = 'funhouse'
    entry = 'funhouse_start1'
    resume_checkpoint = True
    evidence_prefix = 'fulcrum-countdown-'
    capture_scenes = False

    def __init__(self, args):
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.lead, self.lead_tick = self.minimum_lead, None
        self.victim = None
        super().__init__(args)
        raw = args.checkpoint.read_bytes()
        seed = json.loads(raw)['payload']
        assert seed['visit']['map'] == self.map_name and seed['visit']['epoch'] == 1
        assert seed['frame']['checkpoint_generation'] == 0
        assert all(b['stats']['sanity'] > 0 and b['player']['grounded'] for b in seed['frame']['players'])
        (self.folder / 'host/coop-checkpoint.json').write_bytes(raw)
        self.evidence.update(checkpoint_sha256=hashlib.sha256(raw).hexdigest(),
            executable_sha256=hashlib.sha256(args.exe.read_bytes()).hexdigest(), countdown_rejoins={}, neutral_commands={})
        for name in ('test_coop_fulcrum_countdown.py', 'coop_fulcrum_recovery.py',
                     'coop_recovery.py', 'coop_swim_rejoin.py', 'test_coop_play.py', 'test_coop_village.py'):
            data = (ROOT / 'tools' / name).read_bytes()
            (self.folder / name).write_bytes(data)
            self.evidence.setdefault('sources', {})[name] = hashlib.sha256(data).hexdigest()

    def set(self, role, *args, surface=None, **kwargs):
        kwargs['attack'] = False
        super().set(role, *args, **kwargs)
        self.commands[role].update(surface=surface, recovery_trace=role == 'host',
                                  input_lead=self.lead if role == 'guest' else 0)

    def validate_report(self, role, report):
        assert report['visit']['map'] == self.map_name
        assert report['frame']['checkpoint_generation'] == 0, 'Unexpected team wipe'
        assert report['combat']['stage']['owner'] is None, 'Unexpected scene'
        assert not report['route_driver'].get('recovery_trace_error')
        assert all(not b['stats']['god'] and not b['stats']['notarget'] for b in report['frame']['players'])
        if self.victim is not None:
            assert report['frame']['players'][1-self.victim]['stats']['sanity'] > 0, 'Survivor died during recovery'

    def validate_clear(self, role, report):
        assert all(c for c, b in zip(report['route_driver']['clear'], report['frame']['players'])
                   if b['stats']['sanity'] > 0), 'Obstructed living body'

    def pump(self):
        super().pump()
        guest = self.reports.get('guest')
        if guest and guest['frame']['tick'] != self.lead_tick:
            self.lead_tick = guest['frame']['tick']
            rtt = (guest.get('network_timing') or {}).get('round_trip_ms')
            if rtt is not None and 0 <= rtt < 10000:
                self.lead = max(self.minimum_lead, .8*self.lead + .2*min(1.5, .05+rtt/1000))
                self.commands['guest']['input_lead'] = self.lead
        for role, report in self.reports.items():
            row = self.history[role][-1]
            # Keep camera inputs from the same first observation of this tick.
            row.setdefault('presentation', report['presentation'])
            row.setdefault('fulcrum', report['combat']['stage']['world']['levels'][self.map_name]['fulcrums'][62])
            verify_presence([row], role)

    def until(self, predicate, seconds=30):
        end = time.monotonic() + seconds
        while not predicate():
            assert time.monotonic() < end, 'Timed out; complete reports retained in evidence'
            self.pump()

    def live(self, role, predicate, seconds, label):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            process = self.processes[role]
            assert process.poll() is None, role + ' exited during ' + label
            report = read(self.folder / role / 'coop-live.json')
            if report and report['pid'] == process.pid and predicate(report):
                return report
            self.pump()
        raise AssertionError('Timed out waiting for ' + label)

    def reconnect(self, victim):
        guest = self.processes.pop('guest')
        old_pid = guest.pid
        assert guest.poll() is None
        guest.kill()
        guest.wait(timeout=10)
        self.reports.pop('guest')
        before = self.live('host', lambda r: not r['playing'], 15, 'waiting platform host')
        end = time.monotonic() + .4
        while time.monotonic() < end:
            self.pump()
        held = self.live('host', lambda r: not r['playing'], 5, 'held platform world')
        record = dict(passed=False, before=before, held=held)
        path = self.folder / (('blue', 'red')[victim] + '-rejoin.json')
        path.write_text(json.dumps(record))
        assert before['frame'] == held['frame'] and before['combat'] == held['combat']
        self.start('guest')
        after = self.live('guest', lambda r: r['playing'], 60, 'replacement platform guest')
        record['after'] = after
        path.write_text(json.dumps(record))
        proof = verify_fulcrum_rejoin(before, held, after, victim, old_pid)
        key = after['frame']['epoch'], after['frame']['tick']
        host = self.live('host', lambda r: any((s['epoch'], s['tick']) == key
            for s in r['route_driver']['network_states']), 5, 'first platform-frame host evidence')
        record['matching_host'] = host
        proof['first_frame'] = verify_first_frame(host, after)
        self.evidence['countdown_rejoins'][str(victim)] = proof
        record.update(passed=True, evidence=proof)
        path.write_text(json.dumps(record))
        self.pump()
        print('PASS moving-support countdown guest restart:', victim,
            proof['remaining_before'], '->', proof['remaining_after'], flush=True)

    def exercise(self, victim):
        self.victim = victim
        role, survivor = ('host', 'guest')[victim], ('guest', 'host')[victim]
        frame = lambda: self.reports['host']['frame']
        body = lambda: frame()['players'][victim]
        old_spawn = body()['spawned_at']
        x = frame()['players'][1-victim]['player']['feet'][0]
        self.set(survivor, [x, -3450, 0], surface=460, run=True)
        self.until(lambda: self.reached(survivor), 15)
        self.until(lambda: self.history['host'][-1]['fulcrum'][0] <= -44, 12)
        feet = body()['player']['feet']
        self.set(role, [1640, feet[1], feet[2]], run=True)
        self.until(lambda: body()['stats']['sanity'] <= 0, 15)
        self.set(role)
        self.evidence['neutral_commands'][str(victim)] = self.commands[role]['id']
        self.set(survivor, [x, -3750, 0], surface=460, run=True)
        self.until(lambda: self.reached(survivor), 12)
        self.reconnect(victim)
        left = lambda: frame()['lives']['states'][victim]['Spectating']['ticks_left']
        self.until(lambda: left() <= 600, 10)
        # Reverse load half way through the countdown so the board is still
        # rotating when the timer expires. Goals project onto the native brush.
        self.set(survivor, [x, -3450, 0], surface=460, run=True)
        self.until(lambda: all(r['frame']['players'][victim]['spawned_at'] > old_spawn for r in self.reports.values()), 15)
        spawn = body()['spawned_at']
        self.until(lambda: all(r['frame']['tick'] >= spawn+180 for r in self.reports.values()), 5)
        print('Observed platform revival:', victim, spawn, flush=True)

    def run(self):
        self.start_pair()
        invite = self.invite.read_text().strip()
        assert invite.startswith('http://127.0.0.1:' if self.args.local else 'https://') and '/#lg-coop=1.' in invite
        for victim in (1, 0):
            self.exercise(victim)
        for key, metric in (('sha256', 'matching_sent_and_restored_states'),
                            ('world_sha256', 'matching_world_states'), ('native_sha256', 'matching_native_enemies')):
            self.verify_network_states(key, metric)
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        trace = read_trace(self.folder / 'host/recovery-trace.jsonl', self.build, require_revival=True)
        assert [e['player'] for e in trace['revivals']] == [1, 0]
        self.evidence['exact_recovery'] = trace
        self.evidence['moving_support_recovery'] = [verify_fulcrum_recovery(e, self.history,
            self.evidence['neutral_commands'][str(e['player'])]) for e in trace['revivals']]
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            evidence=self.evidence, full_route_passed=False,
            scope='Both countdown reconnects, exact moving-platform revivals and passive carry from a captured gameplay checkpoint'), indent=2))
        print('PASS both moving-platform revivals and passive carry', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not all(0 <= n <= 500 for n in (args.delay_ms, args.jitter_ms)):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = FulcrumCountdown(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failure-report.json').write_text(json.dumps(dict(passed=False,
            error=str(error), build=pair.build, reports=pair.reports, evidence=pair.evidence)))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
