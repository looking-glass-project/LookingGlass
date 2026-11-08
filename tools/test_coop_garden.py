"""Traverse Garden in independent clients using ordinary inputs over the internet.

Each process runs the same authored course from its own received world. Reports
and state hashes replace screenshots. Optional individual revival is checked
against the exact host countdown and both views; wipes/rejoins remain separate.
"""
import argparse
import hashlib
import json
import shutil
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair
from coop_recovery import read_trace, verify_views


class GardenPair(Pair):
    map_name = 'garden1'
    next_map = 'garden2'
    entry = 'garden1_start1'
    evidence_prefix = 'garden-route-'
    capture_scenes = False

    def __init__(self, args):
        self.allow_revival = getattr(args, 'allow_revival', False)
        self.course_starts = {role: args.start_step for role in ('host', 'guest')}
        self.observed_spawns = {}
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.guest_lead = self.minimum_lead
        self.lead_tick = None
        super().__init__(args)
        self.deadline = time.monotonic() + 1200
        self.progress = None
        self.last_valid = {}
        source = Path(__file__).read_bytes()
        (self.folder / 'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()
        self.course_hashes = {}
        self.encounter_snapshots = set()
        if args.checkpoint:
            seed = json.loads(args.checkpoint.read_text())['payload']
            assert seed['visit']['map'] == self.map_name and seed['visit']['epoch'] == 1
            assert seed['visit']['entry'] == self.entry
            self.resume_checkpoint = True
            shutil.copyfile(args.checkpoint, self.folder / 'host' / 'coop-checkpoint.json')
            self.evidence['checkpoint_sha256'] = hashlib.sha256(args.checkpoint.read_bytes()).hexdigest()
            self.evidence['start_tick'] = seed['frame']['tick']
            self.evidence['start_step'] = args.start_step

    def set(self, role, *args, follow=False, **kwargs):
        kwargs['attack'] = False
        super().set(role, *args, **kwargs)
        self.commands[role].update(
            garden_follow=self.args.first.title() if follow else None,
            garden_start=self.course_starts[role] if follow else None,
            input_lead=self.guest_lead if role == 'guest' else 0,
            watch_dialogue=self.args.scenes == 'watched')
        self.commands[role]['recovery_trace'] = self.allow_revival and role == 'host'

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Garden route stopped; evidence retained')
        super().pump()
        guest = self.reports.get('guest')
        if guest and guest['frame']['tick'] != self.lead_tick:
            self.lead_tick = guest['frame']['tick']
            round_trip = (guest.get('network_timing') or {}).get('round_trip_ms')
            if round_trip is not None and 0 <= round_trip < 10000:
                # Include actual relay travel, not just the injected delay.
                # Smooth fresh acknowledgements; a repeated report is one sample.
                observed = min(1.5, .05 + round_trip / 1000)
                self.guest_lead = max(self.minimum_lead, .8 * self.guest_lead + .2 * observed)
                self.commands['guest']['input_lead'] = self.guest_lead
                self.history['guest'][-1]['test_input_lead'] = self.guest_lead
                self.evidence['last_measured_input_lead'] = self.guest_lead
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
            row['presentation'] = report.get('presentation')
            garden = report['combat']['stage']['world']['levels'].get('garden1')
            row['garden'] = {key: garden[key] for key in ('arrived', 'rabbit_done', 'shell')} if garden else None
            if garden:
                row['garden']['rabbit_contact'] = garden['world']['rabbit']
        if self.allow_revival:
            self.follow_survivor_after_revival()
        if len(self.reports) == 2:
            courses = [self.reports[r]['route_driver']['garden_course'] for r in ('host', 'guest')]
            progress = tuple(None if c is None else c['step'] // 10 for c in courses)
            if progress != self.progress:
                self.progress = progress
                print('Garden steps', [None if c is None else c['step'] for c in courses],
                      'health', [b['stats']['sanity'] for b in self.reports['host']['frame']['players']],
                      'tick', self.reports['host']['frame']['tick'], flush=True)

    def follow_survivor_after_revival(self):
        # A real revival puts Alice beside her partner, often beyond a one-way
        # jump. Continue from the survivor's observed course position instead
        # of steering back toward the launch point where the dead player was.
        for role, report in self.reports.items():
            if report['visit']['map'] != self.map_name:
                continue
            slot = 0 if role == 'host' else 1
            body = report['frame']['players'][slot]
            spawn = body['spawned_at']
            old = self.observed_spawns.setdefault(role, spawn)
            self.observed_spawns[role] = spawn
            if spawn == old or not self.commands[role].get('garden_follow'):
                continue
            assert report['visit']['map'] == self.map_name and spawn > old, 'Unexpected Garden revival boundary'
            partner = 'guest' if role == 'host' else 'host'
            witnessed = [r for r in self.history[partner] if r['map'] == self.map_name
                and r['epoch'] == report['frame']['epoch'] and r['tick'] <= spawn
                and r['driver'].get('garden_course') is not None]
            assert witnessed, 'No survivor course evidence before revival'
            source = witnessed[-1]
            step = source['driver']['garden_course']['step']
            before = report['route_driver']['garden_course']['step']
            self.course_starts[role] = step
            self.set(role, follow=True)
            event = dict(role=role, player=slot, epoch=report['frame']['epoch'], spawn_tick=spawn,
                         from_step=before, to_step=step, survivor=partner, survivor_tick=source['tick'])
            self.evidence.setdefault('revival_rejoins', []).append(event)
            print('Garden verified-revival follow', role, before, '->', step, 'spawn', spawn, flush=True)

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
        if self.allow_revival:
            assert report['frame']['checkpoint_generation'] == 0, 'Unexpected team wipe; separate checkpoint verification required'
            assert report['route_driver'].get('recovery_trace_error') is None, 'Host recovery trace failed'
        else:
            super().validate_report(role, report)
        for body in report['frame']['players']:
            assert self.allow_revival or body['spawned_at'] == 0, f'{role}: traversal respawned a player'
            assert not body['stats']['god'] and not body['stats']['notarget'], f'{role}: cheats enabled'
        players = [b['player'] for b in report['frame']['players']]
        if report['combat']['stage']['owner'] is None and any(p['rope'] for p in players):
            left, right = [p['feet'] for p in players]
            horizontal = distance(left[:2], right[:2])
            vertical = abs(left[2] - right[2])
            observed = self.evidence.setdefault('rope_spacing_samples', {})
            observed[role] = observed.get(role, 0) + 1
            if horizontal < 35.95 and vertical < 55.95:
                (self.folder / f'{role}-rope-overlap.json').write_text(json.dumps(report))
                raise AssertionError(f'{role}: rope bodies overlap at tick {report["frame"]["tick"]}: '
                                     f'horizontal={horizontal:.3f}, vertical={vertical:.3f}')
        self.last_valid[role] = report
        # Keep the real state before a combat failure, rather than only the last
        # already-doomed frame. These are private, read-only replay fixtures.
        course = report['route_driver']['garden_course']
        # Keep an exact world/player state before the later rope approach, so
        # a failure can be investigated without replaying the whole visit.
        if role == 'host' and course and course['step'] in (69, 71, 72, 75):
            marker = f'rope-step-{course["step"]}'
            if marker not in self.encounter_snapshots:
                (self.folder / f'encounter-{marker}.json').write_text(json.dumps(report))
                self.encounter_snapshots.add(marker)
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
        if self.args.checkpoint:
            self.until(lambda: all((r['route_driver']['garden_course'] or {}).get('step', 0)
                >= self.args.end_step for r in self.reports.values()), 120)
            assert all(r['visit']['map'] == self.map_name for r in self.reports.values())
            self.verify_network_states()
            self.verify_network_states('world_sha256', 'matching_world_states')
            self.verify_network_states('native_sha256', 'matching_native_residents')
            self.evidence['end_steps'] = {role: r['route_driver']['garden_course']['step']
                for role, r in self.reports.items()}
            self.evidence['end_tick'] = self.reports['host']['frame']['tick']
            self.evidence['final_health'] = [b['stats']['sanity'] for b in self.reports['host']['frame']['players']]
        else:
            self.until(self.arrived, 900)
            self.verify()
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=1 if self.args.checkpoint else 2)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        if self.allow_revival:
            recovery = read_trace(self.folder / 'host' / 'recovery-trace.jsonl', self.build,
                                  getattr(self.args, 'require_revival', False))
            recovery['rendered_revivals'] = verify_views(recovery['revivals'], self.history)
            for follow in self.evidence.get('revival_rejoins', []):
                assert any(e['player'] == follow['player'] and e['epoch'] == follow['epoch']
                           and e['spawn_tick'] == follow['spawn_tick'] for e in recovery['revivals']), \
                    'Course continuation has no exact native revival'
                witness = next(r for r in self.history[follow['survivor']]
                               if r['epoch'] == follow['epoch'] and r['tick'] == follow['survivor_tick'])
                assert witness['tick'] <= follow['spawn_tick'] and \
                    witness['driver']['garden_course']['step'] == follow['to_step'], 'Continuation passed its survivor'
            self.evidence['recovery'] = recovery
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            mode='verified-individual-revival' if self.allow_revival else 'strict-no-death',
            internet=not self.args.local, first=self.args.first, scenes=self.args.scenes,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            full_route_passed=not bool(self.args.checkpoint),
            scope='Captured Garden course segment; preceding traversal and later route excluded'
                if self.args.checkpoint else 'Full independent two-client Garden traversal; ' +
                ('individual revivals verified, wipes/rejoin remain separate' if self.allow_revival
                 else 'recovery/rejoin remain separate')), indent=2))
        print('PASS Garden checkpoint segment.' if self.args.checkpoint else
            'PASS Garden full two-client traversal, owned scenes, personal inventory and Garden2 arrival.', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--scenes', choices=('watched', 'skipped'), default='watched')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--start-step', type=int)
    parser.add_argument('--end-step', type=int)
    parser.add_argument('--allow-revival', action='store_true', help='Allow and verify ordinary individual revivals; team wipes remain separate')
    parser.add_argument('--require-revival', action='store_true', help='Require an exact ten-second/25-health revival with spectator evidence')
    args = parser.parse_args()
    if args.require_revival and not args.allow_revival:
        parser.error('--require-revival requires --allow-revival')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if args.checkpoint:
        if args.start_step is None or args.end_step is None or not 26 <= args.start_step < args.end_step:
            parser.error('A checkpoint segment needs explicit ordered start/end steps after the Rabbit scene')
    elif args.start_step is not None or args.end_step is not None:
        parser.error('Course start/end steps require an isolated checkpoint')
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
