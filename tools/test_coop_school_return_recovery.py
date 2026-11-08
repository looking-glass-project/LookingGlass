"""Return-school combat deaths, moving spectator views and shared checkpoint wipe.

Both internet clients start at the authored return entrance. Each player walks
into a live guard encounter while the survivor stays on the entrance approach.
Only ordinary inputs are supplied; no damage or player/world state is injected.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path

from test_coop_play import ROOT, distance, read
from test_coop_school_recovery import SchoolRecovery
from test_coop_school_return import SchoolReturnPair
from test_coop_village_recovery import Recovery


class SchoolReturnRecovery(Recovery, SchoolReturnPair):
    evidence_prefix = 'school-return-recovery-'
    loot_visit = 'skool1$return'
    death_goal = (-2104, 3136, -586)
    death_navigate = True

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'fixture-recovery-source.py').write_bytes(source)
        self.evidence['recovery_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def approach_fall(self, roles):
        for role in roles:
            self.set(role, (-1888, 3290, -649), speed_limit=40)
            self.until(lambda: self.reached(role), 40)
            self.set(role)

    def survivor_goal(self, victim_index):
        survivor = 1 - victim_index
        feet = self.bodies()[survivor]['player']['feet']
        victim = self.bodies()[victim_index]['player']['feet']
        candidates = [p for p in self.reports['host']['route_driver']['neighbors'][survivor]
                      if abs(p[2] - feet[2]) < 8 and distance(p, feet) >= 32]
        assert candidates, 'No supported entrance step for the moving spectator subject'
        # These are observed, clear walking neighbors. Prefer stepping away
        # from the live fight while providing a moving camera subject.
        return max(candidates, key=lambda p: distance(p, victim))

    def verify_wiped_world(self, report, saved):
        # The locked return quest's timer advances during ordinary play too.
        # Verify its restored value plus the actual resumed host ticks, then
        # retain the first-school checks for every other saved quest field.
        elapsed = report['frame']['tick'] - report['frame']['players'][0]['spawned_at']
        assert 0 <= elapsed <= 120, 'Restore report arrived too late to verify the return clock'
        expected = copy.deepcopy(saved)
        old = expected['frame']['combat']['stage']['world']['school']['return_visit']
        current = report['combat']['stage']['world']['school']['return_visit']
        assert old['phase'] == 'Locked' and old.get('scene') is None
        assert abs(current['time'] - (old['time'] + elapsed / 120)) < .00001, \
            'Wipe did not restore the return quest clock before ordinary ticking'
        old['time'] = current['time']
        SchoolRecovery.verify_wiped_world(self, report, expected)

    wipe = SchoolRecovery.wipe

    def run(self):
        for role in self.commands:
            self.set(role, attack=False)
        self.start_pair()
        checkpoint = self.folder / 'host' / 'coop-checkpoint.json'
        self.until(lambda: read(checkpoint) is not None, 150)
        saved = read(checkpoint)['payload']
        assert saved['visit']['map'] == self.map_name and saved['visit']['entry'] == self.entry
        assert saved['frame']['checkpoint_generation'] == 0
        (self.folder / 'starting-checkpoint.json').write_text(json.dumps(saved))
        for victim in ('guest', 'host') if self.args.first == 'red' else ('host', 'guest'):
            self.fall_and_respawn(victim)
        self.wipe(saved)
        self.verify_coherence()
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, first=self.args.first, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Both return-school combat death/spectator/respawn roles and shared checkpoint wipe; no full route'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='red')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    parser.set_defaults(rejoin=False, checkpoint=None, resume_point=None)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = SchoolReturnRecovery(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
