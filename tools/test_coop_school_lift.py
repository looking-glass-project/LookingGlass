"""Exercise spiral-lift boarding from a staged lower-library start.

Two independent clients use the real internet connection and ordinary movement.
Only the initial spawn is staged, through --start-at; no live positions, health,
quest state or platform state are written. This is a component test, not school
traversal evidence. Run inside Anode or another dedicated desktop session.
"""
import argparse
import hashlib
import json
import math
import time
from pathlib import Path

from test_coop_school import SchoolPair
from test_coop_play import ROOT, read, distance
from test_coop_village import verify_rejoin_stats


def lift_object(report):
    return next(o for o in report['combat']['stage']['world']['school']['objects']
                if o['name'] == 'library_spiral_lift')


def rider_offset(feet, pose):
    assert all(abs(a) < .001 for a in pose['angles'][:2]), 'Expected a vertical-axis lift'
    x, y, z = [a-b for a,b in zip(feet, pose['origin'])]
    angle = math.radians(pose['angles'][2])
    return [math.cos(angle)*x + math.sin(angle)*y,
            -math.sin(angle)*x + math.cos(angle)*y, z]


def verify_lift_rejoin(before, after, old_pid):
    assert after['pid'] != old_pid, 'Guest process did not restart'
    assert after['build'] == before['build'] and after['visit'] == before['visit'], 'Rejoin changed build or visit'
    old, new = before['frame'], after['frame']
    elapsed = new['tick'] - old['tick']
    assert elapsed >= 0, 'Rejoin moved the clock backwards'
    assert old['checkpoint_generation'] == new['checkpoint_generation'] == 0, 'Rejoin silently wiped'
    for r in (before, after):
        assert r['combat']['stage']['owner'] is None, 'Rejoin started a scene'
    previous, current = lift_object(before), lift_object(after)
    assert -260 < previous['pose']['origin'][2] < 40, 'Reconnect did not occur during the ride'
    assert previous['enabled'] and previous['solid'] and current['enabled'] and current['solid'], 'Rejoin disabled the lift'
    assert previous['motion'] and current['motion'], 'Rejoin lost the lift motion'
    for field in ('keys', 'repeat'):
        assert current['motion'][field] == previous['motion'][field], 'Rejoin replaced the lift motion'
    advance = current['motion']['time'] - previous['motion']['time']
    assert 0 <= advance <= elapsed / 120 + .003, 'Rejoin restarted or skipped lift time'
    clocks = [r['combat']['stage']['world']['school']['clock'] for r in (before, after)]
    assert abs(clocks[1]-clocks[0]-elapsed/120) < .006, 'Rejoin changed the shared world clock'
    for obj in (previous, current):
        # The captured authored motion must still explain the actual pose.
        motion = obj['motion']
        start, end = next((a,b) for a,b in zip(motion['keys'], motion['keys'][1:]) if motion['time'] <= b[0])
        blend = max(0, min(1, (motion['time']-start[0])/(end[0]-start[0])))
        for field in ('origin', 'angles'):
            expected = [a+(b-a)*blend for a,b in zip(start[1][field], end[1][field])]
            assert distance(obj['pose'][field], expected) < .03, 'Lift pose does not match its motion clock'
    offsets = []
    for a,b in zip(old['players'], new['players']):
        verify_rejoin_stats(a['stats'], b['stats'], elapsed)
        assert a['stats']['loot'] == b['stats']['loot'], 'Rejoin changed personal loot'
        assert a['spawned_at'] == b['spawned_at'] == 0, 'Rejoin respawned a rider'
        assert a['warped_at'] == b['warped_at'] == 0, 'Rejoin teleported a rider'
        assert a['player']['grounded'] and b['player']['grounded'], 'Rejoin detached a rider'
        old_offset = rider_offset(a['player']['feet'], previous['pose'])
        new_offset = rider_offset(b['player']['feet'], current['pose'])
        assert distance(old_offset, new_offset) < .2, 'Rejoin changed a rider position relative to the lift'
        offsets.append(dict(before=old_offset, after=new_offset))
    return dict(before_tick=old['tick'], after_tick=new['tick'], motion_before=previous['motion']['time'],
        motion_after=current['motion']['time'], rider_offsets=offsets,
        old_guest_pid=old_pid, new_guest_pid=after['pid'])


class SchoolLiftPair(SchoolPair):
    evidence_prefix = 'school-lift-'
    start_at = (144, 4230, -288)

    def __init__(self, args):
        self.rejoin_started = False
        self.rejoin_done = False
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'lift-fixture-source.py').write_bytes(source)
        self.evidence['lift_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def pump(self):
        super().pump()
        if not getattr(self.args, 'rejoin', False) or self.rejoin_started or len(self.reports) != 2:
            return
        obj = lift_object(self.reports['host'])
        idle = all(command['goal'] is None and self.reports[role]['route_driver']['command'] == command['id']
                   for role,command in self.commands.items())
        settled = all(b['player']['grounded'] and math.hypot(*b['player']['velocity'][:2]) < .5 for b in self.bodies())
        if obj['motion'] and -220 < obj['pose']['origin'][2] < 0 and idle and settled:
            self.rejoin_started = True
            self.rejoin_during_ride()

    def rejoin_during_ride(self):
        guest = self.processes['guest']
        old_pid = guest.pid
        assert guest.poll() is None, 'Guest ended before the ride reconnect'
        guest.kill()
        guest.wait(timeout=10)
        self.processes.pop('guest')
        self.reports.pop('guest')
        deadline = time.monotonic()+15
        while True:
            self.pump()  # Keep ordinary control heartbeats alive for the host.
            before = read(self.folder/'host'/'coop-live.json')
            assert self.processes['host'].poll() is None, 'Host ended when the guest disconnected'
            if before and before['pid'] == self.processes['host'].pid and not before['playing']:
                break
            assert time.monotonic() < deadline, 'Host did not observe guest loss'
        deadline = time.monotonic()+.5
        while time.monotonic() < deadline:
            self.pump()
        held = read(self.folder/'host'/'coop-live.json')
        assert held and not held['playing'] and held['frame'] == before['frame'], 'Disconnected host moved the riders'
        assert held['combat'] == before['combat'], 'Disconnected host advanced the lift/world'
        self.start('guest')
        self.until(lambda: 'guest' in self.reports and self.reports['guest']['pid'] != old_pid, 60)
        after = self.reports['guest']
        receipt = dict(passed=False, before=before, after=after)
        (self.folder/'rejoin.json').write_text(json.dumps(receipt, indent=2))
        self.evidence['rejoin'] = verify_lift_rejoin(before, after, old_pid)
        receipt.update(passed=True, evidence=self.evidence['rejoin'])
        (self.folder/'rejoin.json').write_text(json.dumps(receipt, indent=2))
        self.rejoin_done = True
        print('PASS fresh guest rejoined during rotating lift; world held and rider attachments/resources preserved', flush=True)

    def run(self):
        self.start_pair()
        self.wait(1)
        self.go(self.start_at)
        self.lift('library_spiral_lift', (144, 4360, -264), (192, 4360, -264), 64)
        assert not getattr(self.args, 'rejoin', False) or self.rejoin_done, 'Moving-lift reconnect was not exercised'
        self.verify_coherence()
        self.evidence['final_health'] = [b['stats']['sanity'] for b in self.bodies()]
        assert all(r['visit']['map'] == 'skool1' and r['visit']['epoch'] == 1
                   for r in self.reports.values()), 'Lift test unexpectedly changed level'
        assert all(b['spawned_at'] == 0 for r in self.history.values() for row in r for b in row['players'])
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps(dict(
            passed=True, build=self.build, internet=not self.args.local,
            first=self.args.first, delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            evidence=self.evidence,
            scope='Staged spiral-lift approach, paired boarding and ride' +
                (' with guest restart during motion' if self.rejoin_done else '') +
                '; not campaign traversal'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true', help='Restart the guest during the rotating lift ride')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = SchoolLiftPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
