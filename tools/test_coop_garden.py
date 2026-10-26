"""Traverse Garden in independent clients using ordinary inputs over the internet.

Each process runs the same authored course from its own received world. Reports
and state hashes replace screenshots. Clean traversal does not establish the
separate death, checkpoint and reconnect requirements.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair


class GardenPair(Pair):
    map_name = 'garden1'
    next_map = 'garden2'
    entry = 'garden1_start1'
    evidence_prefix = 'garden-route-'
    capture_scenes = False

    def __init__(self, args):
        super().__init__(args)
        self.deadline = time.monotonic() + 1200
        self.progress = None
        self.last_valid = {}
        source = Path(__file__).read_bytes()
        (self.folder / 'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()
        self.course_hashes = {}
        self.encounter_snapshots = set()

    def set(self, role, *args, follow=False, **kwargs):
        kwargs['attack'] = False
        super().set(role, *args, **kwargs)
        self.commands[role].update(
            garden_follow=self.args.first.title() if follow else None,
            input_lead=min(1.5, .05 + 2 * (self.args.delay_ms + self.args.jitter_ms) / 1000)
                if role == 'guest' else 0,
            watch_dialogue=self.args.scenes == 'watched')

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Garden route stopped; evidence retained')
        super().pump()
        for role, report in self.reports.items():
            if role not in self.course_hashes and report['route_driver']['garden_course']:
                # Read the bytes embedded in the tested executable, which may
                # be frozen before the working tree receives another change.
                course = (self.folder / role / 'garden1-coop-course.json').read_bytes()
                self.course_hashes[role] = hashlib.sha256(course).hexdigest()
                assert len(set(self.course_hashes.values())) == 1, 'Clients use different courses'
                (self.folder / 'course.json').write_bytes(course)
                self.evidence['course_sha256'] = self.course_hashes[role]
            row = self.history[role][-1]
            garden = report['combat']['stage']['world']['levels'].get('garden1')
            row['garden'] = {key: garden[key] for key in ('arrived', 'rabbit_done', 'shell')} if garden else None
            if garden:
                row['garden']['rabbit_contact'] = garden['world']['rabbit']
        if len(self.reports) == 2:
            courses = [self.reports[r]['route_driver']['garden_course'] for r in ('host', 'guest')]
            progress = tuple(None if c is None else c['step'] // 10 for c in courses)
            if progress != self.progress:
                self.progress = progress
                print('Garden steps', [None if c is None else c['step'] for c in courses],
                      'health', [b['stats']['sanity'] for b in self.reports['host']['frame']['players']],
                      'tick', self.reports['host']['frame']['tick'], flush=True)

    def until(self, predicate, seconds=240):
        deadline = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= deadline:
                raise AssertionError('Garden timeout: ' + json.dumps({role: {
                    'tick': report['frame']['tick'], 'map': report['visit']['map'],
                    'course': report['route_driver']['garden_course'],
                    'feet': [b['player']['feet'] for b in report['frame']['players']],
                    'health': [b['stats']['sanity'] for b in report['frame']['players']]
                } for role, report in self.reports.items()}))
            self.pump()

    def validate_report(self, role, report):
        super().validate_report(role, report)
        for body in report['frame']['players']:
            assert body['spawned_at'] == 0, f'{role}: traversal respawned a player'
            assert not body['stats']['god'], f'{role}: invulnerability enabled'
        self.last_valid[role] = report
        # Keep the real state before a combat failure, rather than only the last
        # already-doomed frame. These are private, read-only replay fixtures.
        course = report['route_driver']['garden_course']
        if role == 'host' and course and course['step'] == 62:
            markers = ['clear-start']
            for actor, saved in enumerate(report['combat']['npcs']['actors']):
                ant = saved.get('ant')
                for shot in (ant or {}).get('shots', []):
                    if shot['grenade']:
                        birth = round(report['frame']['tick'] - shot['age'] * 120)
                        markers.append(f'grenade-{actor}-{birth}')
            for marker in markers:
                if marker in self.encounter_snapshots or len(self.encounter_snapshots) >= 16:
                    continue
                path = self.folder / f'encounter-{marker}.json'
                path.write_text(json.dumps(report))
                self.encounter_snapshots.add(marker)

    def verify(self):
        for role, rows in self.history.items():
            garden = [row for row in rows if row['map'] == self.map_name and row['epoch'] == 1]
            assert garden and not garden[0]['garden']['arrived'], 'Route did not start at the original arrival'
            scenes = {row['scene'] for row in garden if row['scene']}
            assert scenes == {'Garden1_Start', 'Garden1_Rabbit_Cinema1'}, (role, scenes)
            owners = {row['owner'] for row in garden if row['scene'] == 'Garden1_Rabbit_Cinema1'}
            assert owners == {self.args.first.title()}, f'{role}: wrong Rabbit scene owner {owners}'
            assert garden[-1]['garden']['rabbit_done'] and garden[-1]['garden']['rabbit_contact'] is not None
            changes = [set(), set()]
            for old, row in zip(garden, garden[1:]):
                if row['owner'] is not None and row['owner'] == old['owner']:
                    partner = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][partner]['feet'] == old['players'][partner]['feet'], 'Scene moved waiting player'
                for i, player in enumerate(row['players']):
                    if player['warped_at'] != old['players'][i]['warped_at']:
                        changes[i].add(player['warped_at'])
                        assert old['owner'] is not None or any(
                            row['garden'][key] and not old['garden'][key] for key in ('arrived', 'rabbit_done')
                        ), 'Unexplained traversal teleport'
                if old['owner'] is not None and row['owner'] is None:
                    assert distance(row['players'][0]['feet'], row['players'][1]['feet']) >= 36, 'Reunion overlaps players'
            assert all(len(warps) == 2 for warps in changes), f'{role}: expected two scene reunions, got {changes}'
            arrived = [row for row in rows if row['map'] == self.next_map and row['epoch'] == 2]
            assert arrived, f'{role}: did not load Garden2'
            final = self.reports[role]['frame']['players']
            assert distance(final[0]['player']['feet'], final[1]['player']['feet']) >= 36
            for i, body in enumerate(final):
                assert body['stats']['turtle_air'] and body['stats']['sanity'] > 0
                for key in ('owned', 'collected', 'turtle_air'):
                    assert arrived[0]['players'][i]['stats'][key] == garden[-1]['players'][i]['stats'][key], \
                        f'{role}: transition lost player {i} personal {key}'
            self.evidence[role] = dict(scenes=sorted(scenes), rabbit_owner=sorted(owners),
                reunion_ticks=[sorted(warps) for warps in changes],
                final_health=[b['stats']['sanity'] for b in final])
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')

    def run(self):
        self.start_pair()
        for role in self.commands:
            self.set(role, follow=True)
        self.until(self.arrived, 900)
        self.verify()
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=2)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, first=self.args.first, scenes=self.args.scenes,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Full independent two-client Garden traversal; recovery/rejoin remain separate'), indent=2))
        print('PASS Garden full two-client traversal, owned scenes, personal inventory and Garden2 arrival.', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--scenes', choices=('watched', 'skipped'), default='watched')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = GardenPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failure-report.json').write_text(json.dumps(dict(
            passed=False, error=str(error), build=pair.build, reports=pair.reports,
            previous_reports=pair.last_valid), indent=2))
        raise
    finally:
        pair.close()
