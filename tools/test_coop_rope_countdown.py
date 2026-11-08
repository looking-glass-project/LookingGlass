"""Reconnect during both death countdowns beside a moving rope survivor.

Resume an unchanged captured checkpoint with Red falling and Blue on a rope.
After Red revives, Blue releases through ordinary E input; enemies stay active.
The survivor climbs and uses normal attacks against rifle fire. Only the owned
guest process is restarted. No bodies, resources or enemy health are edited.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import time

from coop_rope_rejoin import verify_rope_rejoin
from coop_recovery import read_trace
from coop_swim_rejoin import verify_first_frame
from test_coop_play import ROOT, read, distance
from test_coop_rope_revival import RopeRevivalPair


class RopeCountdown(RopeRevivalPair):
    evidence_prefix = 'rope-countdown-'
    exact_recovery_trace = True

    def __init__(self, args):
        self.climbing_survivor = False
        self.climb_direction = -1
        super().__init__(args)
        bodies = self.seed['frame']['players']
        assert bodies[0]['player']['rope'] is not None and bodies[1]['player']['rope'] is None, 'Need the captured Blue-rope/Red-fall checkpoint'
        assert all(b['stats']['sanity'] > 0 for b in bodies)
        self.evidence['countdown_rejoins'] = {}
        self.evidence['revivals'] = {}
        self.evidence['sources'] = {}
        for name in ('test_coop_rope_countdown.py', 'test_coop_rope_revival.py',
                     'coop_rope_rejoin.py', 'coop_swim_rejoin.py', 'coop_recovery.py',
                     'test_coop_play.py', 'test_coop_village.py'):
            data = (ROOT / 'tools' / name).read_bytes()
            (self.folder / name).write_bytes(data)
            self.evidence['sources'][name] = hashlib.sha256(data).hexdigest()

    def set(self, role, *args, **kwargs):
        super().set(role, *args, **kwargs)
        self.commands[role]['recovery_trace'] = role == 'host'

    def live(self, role, predicate, seconds, label):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            process = self.processes[role]
            assert process.poll() is None, f'{role} exited during {label}'
            report = read(self.folder / role / 'coop-live.json')
            if report and report['pid'] == process.pid and predicate(report):
                return report
            self.pump()
        raise AssertionError('Timed out waiting for ' + label)

    def guard_rope(self, role, rise=0, wish=(0, 0)):
        self.set(role, rope=dict(action='climb', rise=rise, wish=list(wish)))
        self.commands[role].update(attack=True, combat_tactics=True, avoid_switches=True)

    def pump(self):
        super().pump()
        if self.climbing_survivor and 'host' in self.reports:
            grip = self.reports['host']['frame']['players'][1]['player']['rope']
            if grip is not None:
                direction = self.climb_direction
                if grip['length'] >= 300:
                    direction = 1
                elif grip['length'] <= 120:
                    direction = -1
                if self.commands['guest'].get('rope', {}).get('rise') != direction:
                    self.guard_rope('guest', rise=direction)
                self.climb_direction = direction

    def reconnect(self):
        victim = ('host', 'guest')[self.victim]
        guest = self.processes.pop('guest')
        old_pid = guest.pid
        assert guest.poll() is None
        guest.kill()
        guest.wait(timeout=10)
        self.reports.pop('guest')
        before = self.live('host', lambda r: not r['playing'], 15, 'waiting rope host')
        end = time.monotonic() + .4
        while time.monotonic() < end:
            self.pump()
        held = self.live('host', lambda r: not r['playing'], 5, 'held rope world')
        path = self.folder / (victim + '-countdown-rejoin.json')
        record = dict(passed=False, before=before, held=held)
        path.write_text(json.dumps(record))
        assert before['frame'] == held['frame'] and before['combat'] == held['combat'], 'Waiting rope world advanced'
        self.start('guest')
        after = self.live('guest', lambda r: r['playing'], 60, 'replacement rope guest')
        record['after'] = after
        path.write_text(json.dumps(record))
        proof = verify_rope_rejoin(before, held, after, self.victim, old_pid)
        key = after['frame']['epoch'], after['frame']['tick']
        host = self.live('host', lambda r: any((s['epoch'], s['tick']) == key
            for s in r['route_driver']['network_states']), 5, 'first rope-frame host evidence')
        record['matching_host'] = host
        path.write_text(json.dumps(record))
        proof['first_frame'] = verify_first_frame(host, after)
        self.evidence['countdown_rejoins'][victim] = proof
        record.update(passed=True, evidence=proof)
        path.write_text(json.dumps(record))
        self.pump()
        print('PASS rope countdown guest restart:', victim,
              {k: proof[k] for k in ('before_tick', 'after_tick', 'remaining_before', 'remaining_after')}, flush=True)

    def exercise(self, victim):
        self.victim = victim
        self.deaths, self.revivals, self.followed = {}, {}, []
        role = ('host', 'guest')[victim]
        if victim == 0:
            player = self.reports['host']['frame']['players'][0]['player']
            self.guard_rope('guest', wish=(.25, 0))
            self.set('host', rope=dict(action='release', goal=player['feet'][:2]))
        else:
            # The captured low grip is exposed to a living rifle ant. Climb
            # through normal controls before waiting; do not disable its fire.
            self.guard_rope('host', rise=1)
            self.until(lambda: self.reports['host']['frame']['players'][0]['player']['rope']['length'] <= 200, 8)
            self.guard_rope('host')
            self.until(lambda: self.reports['host']['frame']['players'][0]['player']['rope']['rise'] == 0, 3)
        self.until(lambda: len(self.deaths) == 2, 30)
        # Stop the release input before revival can attach a new grip.
        self.set(role)
        self.until(lambda: 300 <= self.reports['host']['frame']['lives']['states'][victim]['Spectating']['ticks_left'] <= 1000, 10)
        self.reconnect()
        # After proving the unchanged neutral grip at reconnect, keep the
        # surviving player moving vertically against ordinary rifle fire.
        self.climbing_survivor = victim == 0
        self.until(lambda: len(self.revivals) == 2, 35)
        self.climbing_survivor = False
        assert len(set(self.deaths.values())) == 1
        assert len({r['spawn_tick'] for r in self.revivals.values()}) == 1
        assert self.followed and max(distance(p, self.followed[0]) for p in self.followed) > 10, 'Spectator did not follow a moving rope'
        self.evidence['revivals'][role] = copy.deepcopy(self.revivals)

    def run(self):
        self.start_pair()
        invite = self.invite.read_text().strip()
        assert invite.startswith('http://127.0.0.1:' if self.args.local else 'https://') and '/#lg-coop=1.' in invite
        for victim in (1, 0):
            self.exercise(victim)
        for key, name in (('sha256', 'matching_sent_and_restored_states'),
                          ('world_sha256', 'matching_world_states'),
                          ('native_sha256', 'matching_native_enemies')):
            self.verify_network_states(key, name)
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        # Read only after the host has closed its trace, avoiding a partial
        # concurrently-written final row.
        trace = read_trace(self.folder / 'host/recovery-trace.jsonl', self.build, require_revival=True)
        assert [event['player'] for event in trace['revivals']] == [1, 0]
        assert all(event['wait_ticks'] == 0 and event['paused_ticks'] == 0 for event in trace['revivals'])
        self.evidence['exact_recovery'] = trace
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            evidence=self.evidence, full_route_passed=False,
            scope='Both death countdown guest restarts and ten-second rope revivals from a captured gameplay checkpoint'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    args.victim = 'red'
    if not all(0 <= n <= 500 for n in (args.delay_ms, args.jitter_ms)):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = RopeCountdown(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failure-report.json').write_text(json.dumps(dict(passed=False,
            error=str(error), build=pair.build, reports=pair.reports, evidence=pair.evidence), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
