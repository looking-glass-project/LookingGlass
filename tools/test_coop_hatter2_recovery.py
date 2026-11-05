"""Exercise About Face combat revival or a guest restart without state writes.

Revival begins at the normal entrance and uses actual Hatter damage. Rejoin uses
an unchanged ordinary victory checkpoint and its existing recovery provenance.
Neither bounded case claims a full entrance-to-exit route or team-wipe coverage.
"""
import argparse
import copy
import hashlib
import json
import struct
import time
from pathlib import Path

from test_coop_hatter2 import AboutFacePair, ROLES, CORNERS
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from test_coop_play import distance
from coop_recovery import read_trace, verify_presence, verify_views


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def passive_victory_state(state, ticks):
    """Predict only normal idle clocks, personal cooldowns and expiring drops."""
    assert type(ticks) is int and 0 <= ticks <= 36000
    result = copy.deepcopy(state)
    dt = f32(1 / 120)
    items = result['coop']['items']
    assert len(result['drops']) == len(items['drop_taken'])
    for _ in range(ticks):
        result['clock'] = min(f32(result['clock'] + dt), 86400)
        result['time'] = min(f32(result['time'] + dt), 3600)
        for slot in range(2):
            if items['wait'][slot] > 0:
                items['wait'][slot] = max(f32(items['wait'][slot] - dt), 0)
                if items['wait'][slot] == 0: items['ready'][slot] = True
        result['remainder'] += dt
        while result['remainder'] + 1e-9 >= 1 / 120:
            result['remainder'] = max(result['remainder'] - 1 / 120, 0)
            retained = [(point, f32(age + dt), mask) for (point, age), mask in zip(result['drops'], items['drop_taken'])
                        if f32(age + dt) < 70]
            result['drops'] = [[p, age] for p, age, _ in retained]
            items['drop_taken'] = [mask for _, _, mask in retained]
    return result


def verify_rejoined_world(before, report):
    old_stage, stage = before['combat']['stage'], report['combat']['stage']
    old, new = (s['world']['levels']['hatter2'] for s in (old_stage, stage))
    assert old_stage['owner'] is None and stage['owner'] is None, 'Restart entered a scene'
    assert old['phase'] == new['phase'] == 'Victory', 'Restart changed the defeated encounter'
    assert old['lifts'] == old['door'] == 0, 'Idle victory restart requires stopped machinery'
    elapsed = report['frame']['tick'] - before['frame']['tick']
    assert passive_victory_state(old, elapsed) == new, 'Restart changed native progress/resources beyond normal clocks and drop expiry'
    assert stage['story']['seen'] == old_stage['story']['seen'], 'Restart forgot or invented a conversation'
    assert stage['story']['completed'] == old_stage['story']['completed'], 'Restart changed completed conversations'


def verify_restart_states(history, after, field):
    states = {role:{(s['epoch'], s['tick']):s[field] for row in rows
                    for s in row['driver']['network_states'] if s['epoch'] == 1 and s['tick'] >= after}
              for role, rows in history.items()}
    assert set(states) == set(ROLES), 'Restart needs both independent clients'
    shared = states['host'].keys() & states['guest'].keys()
    assert len(shared) >= 20, 'Too few matching states from the new guest'
    assert all(states['host'][k] == states['guest'][k] for k in shared), 'Restarted guest differs from the host'
    return len(shared)


class RecoveryPair(AboutFacePair):
    evidence_prefix = 'hatter2-recovery-'

    def __init__(self, args):
        super().__init__(args)
        raw = Path(__file__).read_bytes()
        (self.folder / 'fixture-dependencies' / Path(__file__).name).write_bytes(raw)
        self.evidence['fixture_sha256'][Path(__file__).name] = hashlib.sha256(raw).hexdigest()
        self.evidence['case'] = args.case
        self.evidence['victim'] = args.victim if args.case == 'revival' else None
        self.survivor = 'host' if args.victim == 'red' else 'guest'
        self.recovering = False

    def validate_report(self, role, report):
        super().validate_report(role, report)
        assert report['visit']['map'] == self.map_name and report['visit']['epoch'] == 1
        if self.recovering:
            body = report['frame']['players'][ROLES.index(self.survivor)]
            assert body['stats']['sanity'] > 0 and body['spawned_at'] == 0, 'Survivor died during targeted revival'

    def verify_rejoined_world(self, before, report):
        verify_rejoined_world(before, report)

    def revived_in_both_views(self, victim):
        slot = ROLES.index(victim)
        return all(d['frame']['players'][slot]['spawned_at'] > 0 and
                   d['frame']['players'][slot]['stats']['sanity'] > 0 for d in self.reports.values())

    def revival(self):
        self.approach()
        self.checkpoint('combat')
        self.phase = 'Hatter fight'  # Only this phase permits exact individual recovery.
        self.recovering = True
        victim = 'guest' if self.survivor == 'host' else 'host'
        self.set(victim, [5664, 384, 0], run=True, attack=False)
        cursor = 0 if self.survivor == 'host' else 2
        self.set(self.survivor, CORNERS[cursor], run=True, attack=True)
        started = time.monotonic(); seen_dead = False; printed = -1
        while not self.revived_in_both_views(victim):
            assert time.monotonic() - started < 240, 'Targeted combat revival timed out'
            body = self.body(victim)
            if body['stats']['sanity'] <= 0 and not seen_dead:
                seen_dead = True
                self.set(victim)
                print('Observed', self.args.victim, 'combat death', self.reports['host']['frame']['tick'], flush=True)
            runner = self.body(self.survivor)
            if distance(runner['player']['feet'][:2], CORNERS[cursor][:2]) < 70:
                cursor = (cursor + 1) % 4
                self.set(self.survivor, CORNERS[cursor], run=True, attack=True)
            self.pump()
            interval = int((time.monotonic()-started)/20)
            if interval != printed:
                printed = interval
                print('RECOVERY', self.reports['host']['frame']['tick'],
                    [b['stats']['sanity'] for b in self.reports['host']['frame']['players']], flush=True)
        assert seen_dead, 'Revival lacks an observed combat death'
        self.mark('targeted-revival')

    def rejoin(self):
        receipt = json.loads((self.folder / 'host/route-resume.json').read_text())
        verify_resume_receipt(receipt, self.saved, self.build)
        self.evidence['exact_resume_receipt'] = receipt
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit']
            elapsed = report['frame']['tick'] - self.saved['frame']['tick']
            for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                verify_resume_body(body, old, elapsed)
        self.phase = 'victory guest restart'
        old_pid = self.processes['guest'].pid
        self.rejoin_guest()
        new_pid = self.processes['guest'].pid
        assert new_pid != old_pid and self.reports['guest']['pid'] == new_pid, 'Guest process was not replaced'
        self.evidence['rejoin'].update(old_pid=old_pid, new_pid=new_pid)
        # Continue sampling long enough to require real post-restart snapshots.
        start = self.reports['host']['frame']['tick']
        self.until(lambda: self.reports['host']['frame']['tick'] >= start + 360, 15)
        self.mark('victory-guest-rejoined')

    def run(self):
        self.start_pair()
        if self.args.case == 'revival': self.revival()
        else: self.rejoin()
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        recovery = read_trace(self.folder / 'host/recovery-trace.jsonl', self.build,
                              require_revival=self.args.case == 'revival')
        if self.args.case == 'revival':
            slot = 1 if self.args.victim == 'red' else 0
            assert all(e['player'] == slot for e in recovery['revivals']), 'Unexpected survivor revival'
            recovery['rendered_revivals'] = verify_views(recovery['revivals'], self.history)
        else:
            assert not recovery['revivals'], 'Idle restart caused a new death/revival'
        for role, rows in self.history.items(): verify_presence(rows, role)
        for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'),
                              ('native_sha256', 'matching_native_states')]:
            self.verify_network_states(field, metric)
            if self.args.case == 'rejoin':
                self.evidence['rejoin_' + metric] = verify_restart_states(self.history, self.evidence['rejoin']['after_tick'], field)
        self.evidence['recovery'] = recovery
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route=False,
            build=self.build, internet=not self.args.local, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, case=self.args.case, evidence=self.evidence,
            scope='Targeted ordinary combat revival' if self.args.case == 'revival'
                  else 'Graceful guest process restart at unchanged victory; no packet-loss or moving-transport claim'), indent=2))
        print('PASS About Face', self.args.case, flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--case', choices=('revival', 'rejoin'), required=True)
    parser.add_argument('--victim', choices=('blue', 'red'), default='red')
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if (args.case == 'rejoin') != (args.checkpoint is not None): parser.error('Only the rejoin case requires a normal victory checkpoint')
    if not all(0 <= v <= 500 for v in (args.delay_ms, args.jitter_ms)): parser.error('Delay/jitter must be 0..500 ms')
    args.first, args.allow_revival, args.stop_after = 'blue', True, 'combat'
    pair = RecoveryPair(args)
    try: pair.run()
    except BaseException as error:
        pair.evidence.update(passed=False, failed_phase=pair.phase, error=str(error),
            process_exit_codes_before_cleanup={r:p.poll() for r,p in pair.processes.items()})
        raise
    finally: pair.close()


if __name__ == '__main__': main()
