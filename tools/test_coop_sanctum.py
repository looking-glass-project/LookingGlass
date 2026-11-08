"""Play Sanctum through two independent internet clients using ordinary inputs.

Run in Anode. Each client follows its received boss/body state; the harness
never writes positions, health, inventory or progression. Reports replace images.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair


class SanctumPair(Pair):
    map_name = 'centipede2'
    entry = 'centipede2_start1'
    next_map = 'wforest'
    evidence_prefix = 'sanctum-route-'
    capture_scenes = False

    def __init__(self, args):
        super().__init__(args)
        self.deadline = time.monotonic() + 900
        source = Path(__file__).read_bytes()
        (self.folder / 'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()
        self.evidence['guest_input_lead_seconds'] = self.commands['guest']['input_lead']
        self.last_milestone = None
        self.leader_progress = None
        if args.checkpoint:
            saved = args.checkpoint.read_bytes()
            payload = json.loads(saved)['payload']
            assert payload['visit']['map'] == self.map_name and payload['visit']['epoch'] == 1
            stage = payload['frame']['combat']['stage']['world']['levels']['centipede2']
            assert stage['phase'] == 'Climb' and stage['battle']['hits'] == 6
            (self.folder / 'host' / 'coop-checkpoint.json').write_bytes(saved)
            self.resume_checkpoint = True
            self.evidence['checkpoint_sha256'] = hashlib.sha256(saved).hexdigest()

    def set(self, role, *args, follow=False, **kwargs):
        kwargs['attack'] = False
        super().set(role, *args, **kwargs)
        self.commands[role].update(
            sanctum_follow=self.args.first.title() if follow else None,
            input_lead=(self.args.input_lead if self.args.input_lead is not None else
                min(1.5, .05 + 2 * (self.args.delay_ms + self.args.jitter_ms) / 1000)) if role == 'guest' else 0,
            watch_dialogue=True)

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Sanctum route stopped; evidence retained')
        super().pump()
        for role, report in self.reports.items():
            row = self.history[role][-1]
            saved = report['combat']['stage']['world']['levels'].get('centipede2')
            row['sanctum'] = dict(phase=saved['phase'], hits=saved['battle']['hits'],
                                  battle_stage=saved['battle']['stage']) if saved else None
            row['presentation'] = report.get('presentation')
            leader = 'host' if self.args.first == 'blue' else 'guest'
            course = row['driver'].get('sanctum_course')
            if role == leader:
                if saved and saved['phase'] == 'Climb' and course and self.commands[role].get('sanctum_follow'):
                    step = course['steps'][0 if leader == 'host' else 1]
                    if self.leader_progress is None or self.leader_progress[0] != step:
                        self.leader_progress = (step, row['tick'])
                    assert row['tick']-self.leader_progress[1] < 45*120, f'Leader stalled at climb step {step}; full reports retained'
                else:
                    self.leader_progress = None
        if 'host' in self.reports:
            row = self.history['host'][-1]
            saved = row['sanctum']
            milestone = (row['map'], saved['phase'], saved['hits']) if saved else (row['map'],)
            if milestone != self.last_milestone:
                self.last_milestone = milestone
                print('Sanctum', milestone, 'tick', row['tick'], 'health',
                      [p['health'] for p in row['players']], flush=True)

    def until(self, predicate, seconds=240):
        deadline = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= deadline:
                raise AssertionError('Sanctum timeout: ' + json.dumps({role: dict(
                    tick=r['frame']['tick'], map=r['visit']['map'],
                    course=r['route_driver'].get('sanctum_course'),
                    feet=[b['player']['feet'] for b in r['frame']['players']],
                    health=[b['stats']['sanity'] for b in r['frame']['players']])
                    for role, r in self.reports.items()}))
            self.pump()

    def verify_rejoined_world(self, before, report):
        old = before['combat']['stage']['world']['levels']['centipede2']
        current = report['combat']['stage']['world']['levels']['centipede2']
        assert old['phase'] == current['phase'] == 'Climb'
        assert old['battle']['hits'] == current['battle']['hits'] == 6
        assert old['battle']['stage'] == current['battle']['stage'] == 3
        assert current['battle']['action'] == 'Dead'
        for field in ('essence', 'exit', 'checkpoint'):
            assert current[field] == old[field], 'Rejoin changed Sanctum ' + field
        assert report['combat']['stage']['owner'] is None

    def save_after_boss(self):
        self.wait(.5)
        self.commands['host']['checkpoint'] = True
        command = self.commands['host']['id']
        self.until(lambda: (self.reports['host']['route_driver'].get('checkpoint') or {}).get('command') == command, 15)
        receipt = self.reports['host']['route_driver']['checkpoint']
        assert receipt['epoch'] == 1 and receipt['error'] is None, receipt
        saved = (self.folder / 'host' / 'coop-checkpoint.json').read_bytes()
        payload = json.loads(saved)['payload']
        assert payload['frame']['tick'] == receipt['tick'] and payload['visit']['map'] == self.map_name
        assert payload['frame']['combat']['stage']['world']['levels']['centipede2']['phase'] == 'Climb'
        (self.folder / 'checkpoint-after-boss.json').write_bytes(saved)
        self.evidence['saved_after_boss'] = dict(sha256=hashlib.sha256(saved).hexdigest(), **receipt)

    def verify(self):
        for role, rows in self.history.items():
            route = [r for r in rows if r['map'] == self.map_name and r['epoch'] == 1]
            assert route
            if getattr(self.args, 'checkpoint', None):
                assert route[0]['sanctum']['phase'] == 'Climb' and all(r['sanctum']['hits'] == 6 for r in route)
            else:
                assert route[0]['sanctum']['hits'] == 0
                assert any(r['sanctum']['phase'] == 'Slide' for r in route), 'Missed entrance slide'
                assert {r['sanctum']['hits'] for r in route} >= set(range(7)), 'Boss reactions missing'
            owners = {r['owner'] for r in route if r['sanctum']['phase'] in ('Greeting', 'Eat', 'Grow')}
            assert owners == {self.args.first.title()}, f'Wrong mushroom owner: {owners}'
            free = [r for r in route if r['owner'] is None]
            assert free and all(r['camera_player'] == ('Blue' if role == 'host' else 'Red') for r in free), \
                'Gameplay camera followed the wrong living player'
            assert all(r['checkpoint_generation'] == 0 and all(p['health'] > 0 and p['spawned_at'] == 0
                       and not p['stats']['god'] and not p['stats']['notarget'] for p in r['players']) for r in rows)
            for r in route:
                if r['owner'] is None:
                    a, b = [p['feet'] for p in r['players']]
                    assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2]-b[2]) >= 55.95, 'Players overlap'
            for old, row in zip(route, route[1:]):
                if row['owner'] is not None and row['owner'] == old['owner']:
                    partner = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][partner]['feet'] == old['players'][partner]['feet'], 'Scene moved waiting partner'
                if row['owner'] is None and old['owner'] is None:
                    assert [p['warped_at'] for p in row['players']] == [p['warped_at'] for p in old['players']], 'Unexplained traversal warp'
            assert any(r['map'] == self.next_map and r['epoch'] == 2 for r in rows)
            final = self.reports[role]['frame']['players']
            # Arrival clears visit-scoped health protection but carries personal resources.
            for old, body in zip(route[-1]['players'], final):
                for field in ('sanity', 'owned', 'collected', 'school_items', 'staff_component'):
                    assert body['stats'][field] == old['stats'][field], 'Arrival changed ' + field
            self.evidence[role] = dict(mushroom_owner=sorted(owners),
                final_health=[b['stats']['sanity'] for b in final], route_ticks=route[-1]['tick'])
        assert self.processes['host'].pid != self.processes['guest'].pid
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')

    def run(self):
        self.start_pair()
        for role in self.commands:
            self.set(role, follow=True)
        if self.args.rejoin:
            self.until(lambda: all(r['combat']['stage']['world']['levels']['centipede2']['phase'] == 'Climb'
                                   for r in self.reports.values()), 360)
            self.save_after_boss()
            self.rejoin_guest()
            self.evidence['rejoin']['after_boss_before_climb'] = True
            for role in self.commands:
                self.set(role, follow=True)
        self.until(self.arrived, 480)
        self.verify()
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=2)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, first=self.args.first, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope=('Captured-checkpoint Sanctum climb-to-exit segment; not a full traversal' if self.args.checkpoint
                else 'Full two-client Sanctum entrance-to-exit session') +
                (' with a fresh guest rejoin after boss defeat' if self.args.rejoin else '') +
                '; death/wipe and both players independently finishing every climb remain separate'), indent=2))
        print('PASS Sanctum ' + ('local' if self.args.local else 'internet') +
              (' checkpoint segment' if self.args.checkpoint else ' full traversal') +
              ': scene ownership, matching states and next-level arrival.', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true')
    parser.add_argument('--checkpoint', type=Path, help='Replay a real saved post-boss checkpoint; segment evidence only')
    parser.add_argument('--input-lead', type=float, help='Override the guest test controller velocity anticipation in seconds')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if args.input_lead is not None and not 0 <= args.input_lead <= 1.5:
        parser.error('Input lead must be between 0 and 1.5 seconds')
    pair = SanctumPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports)))
        raise
    finally:
        pair.close()
