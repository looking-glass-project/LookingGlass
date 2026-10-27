"""Verify a real combat death, spectator camera and timed revival in Pale Realm.

The approach starts from an unchanged normal checkpoint. Combat uses ordinary
course controls; after a death the survivor retreats along already visited
waypoints. An explicit victim keeps fighting while the partner retreats first.
An optional continuation traverses the remaining course after revival. Neither
mode counts as a complete entrance-to-exit route.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_chess_resume import ChessResume
from test_coop_play import ROOT, distance
from test_coop_village import verify_rejoin_stats


def verify_revives(rows, role, initial_spawns):
    deaths, events, followed = {}, {}, {}
    local = 0 if role == 'host' else 1
    for row in rows:
        assert row['checkpoint_generation'] == 0, 'Unexpected team wipe in revival test'
        for index, (body, life) in enumerate(zip(row['players'], row['lives']['states'])):
            if body['health'] <= 0:
                assert isinstance(life, dict) and 'Spectating' in life, 'Dead player has no countdown'
                left = life['Spectating']['ticks_left']
                assert 0 < left <= 1200, 'Invalid revival countdown'
                death_tick = row['tick'] - (1200-left)
                if index in deaths:
                    assert deaths[index][0] == death_tick, 'Countdown changed its death tick'
                else:
                    deaths[index] = (death_tick, body['stats'])
                if index == local:
                    assert row['camera_player'] == ('Blue', 'Red')[1-index], 'Dead camera did not follow partner'
                    subject = row['camera_subject_feet']
                    assert distance(row['camera_target'][:2], subject[:2]) < 1, 'Spectator camera missed rendered subject'
                    assert distance(subject, row['players'][1-index]['feet']) < 24, 'Spectator subject drifted from survivor'
                    followed.setdefault(index, []).append(row['camera_target'])
            elif body['spawned_at'] != initial_spawns[index] and (index, body['spawned_at']) not in events:
                assert index in deaths, 'Revival has no observed death'
                death_tick, old = deaths.pop(index)
                assert body['spawned_at'] - death_tick == 1200, 'Revival did not wait exactly ten seconds'
                assert body['health'] == 25 and life == 'Alive', 'Revival did not publish 25 health and alive state together'
                gap = distance(body['feet'], row['players'][1-index]['feet'])
                assert 40 <= gap <= 110, 'Revival was not safely near the survivor'
                for field in ('owned', 'collected', 'school_items', 'staff_component'):
                    assert body['stats'][field] == old[field], 'Revival changed personal ' + field
                if index == local:
                    assert row['camera_player'] == ('Blue', 'Red')[index], 'Camera did not return after revival'
                    assert distance(row['camera_target'][:2], body['feet'][:2]) < 1, 'Revival camera blended from old body'
                    points = followed.get(index, [])
                    assert points and max(distance(point, points[0]) for point in points) > 10, 'Spectator never followed a moving survivor'
                events[index, body['spawned_at']] = dict(player=index, death_tick=death_tick,
                                                       spawn_tick=body['spawned_at'], health=body['health'])
    assert events and not deaths, 'No completed revival was observed'
    return list(events.values())


class ChessRecovery(ChessResume):
    evidence_prefix = 'chess-recovery-'

    def __init__(self, args):
        self.recovering = False
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'recovery-fixture-source.py').write_bytes(source)
        self.evidence['recovery_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def validate_report(self, role, report):
        if not self.recovering:
            return super().validate_report(role, report)
        self.record_damage(role, report)
        assert report['frame']['checkpoint_generation'] == 0, 'Team wiped before individual revival'
        assert all(not b['stats']['god'] and not b['stats']['notarget'] for b in report['frame']['players'])

    def run(self):
        self.start_pair()
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit']
            elapsed = report['frame']['tick'] - self.saved['frame']['tick']
            for current, old in zip(report['frame']['players'], self.saved['frame']['players']):
                assert distance(current['player']['feet'], old['player']['feet']) < 2
                verify_rejoin_stats(old['stats'], current['stats'], elapsed)
        for role in self.commands:
            # Load the course without accidentally beginning the target fight
            # before the survivor's deliberate retreat has been arranged.
            self.set(role, step=min(self.args.start_step, self.args.death_step-1))
        self.until(lambda: len(self.course_hashes) == 2, 20)
        assert 0 < self.args.start_step <= self.args.death_step < len(self.steps)
        for index in range(self.args.start_step, self.args.death_step):
            if index in self.args.save_before:
                self.save_checkpoint(index)
            if index > self.args.start_step or index in self.args.save_before:
                for role in self.commands:
                    self.set(role, step=index)
            self.until(lambda: all(self.reached(role) for role in self.commands))
            self.completed.append(index)
            print('PASS recovery approach step', index, flush=True)
        if self.args.death_step in self.args.save_before:
            self.save_checkpoint(self.args.death_step)
        self.recovering = True
        requested = None if self.args.victim == 'auto' else ('blue', 'red').index(self.args.victim)
        if requested is not None:
            for role in self.commands:
                self.set(role)
            survivor_role = ('host', 'guest')[1-requested]
            at = self.reports['host']['frame']['players'][1-requested]['player']['feet']
            # Pass around the waiting partner before retreating east. A
            # straight reverse path can make the two solid bodies block one
            # another in this doorway. Every waypoint uses normal movement.
            side_y = 1368 if requested == 1 else 1456
            for goal in ([at[0], side_y, at[2]], [2592, 1392 if requested == 1 else 1456, 256]):
                self.set(survivor_role, goal=goal, navigate=True)
                self.until(lambda: self.reached(survivor_role), 30)
            self.set(survivor_role, goal=self.steps[122]['pos'], navigate=True)
            self.set(('host', 'guest')[requested], step=self.args.death_step)
        else:
            for role in self.commands:
                self.set(role, step=self.args.death_step)
        self.until(lambda: any(b['stats']['sanity'] <= 0 for b in self.reports['host']['frame']['players'])
                   or all(self.reached(role) for role in self.commands), 180)
        bodies = self.reports['host']['frame']['players']
        assert any(b['stats']['sanity'] <= 0 for b in bodies), 'Encounter finished without a death; no revival evidence'
        victim = next(i for i, b in enumerate(bodies) if b['stats']['sanity'] <= 0)
        assert requested is None or victim == requested, 'Encounter killed a different player than requested'
        survivor = 1-victim
        assert bodies[survivor]['stats']['sanity'] > 0, 'No survivor to spectate'
        victim_role, survivor_role = ('host', 'guest')[victim], ('host', 'guest')[survivor]
        old_spawn = bodies[victim]['spawned_at']
        # Reverse through known dry approaches while the dead client's own
        # course remains selected, exercising its spectator/resume clock.
        retreats = [self.steps[i]['pos'] for i in (124, 122)]
        assert all(self.steps[i]['mode'] == 'walk' for i in (124, 122))
        self.set(survivor_role, goal=retreats[0], navigate=True)
        end = self.reports['host']['frame']['tick'] + 1800
        while True:
            self.pump()
            report = self.reports[victim_role]
            body = report['frame']['players'][victim]
            if body['spawned_at'] > old_spawn and body['stats']['sanity'] > 0 \
                    and report['route_driver']['phase'] != 'spectating':
                break
            assert self.reports['host']['frame']['tick'] < end, 'Revival did not finish'
            if self.reached(survivor_role) and len(retreats) > 1:
                retreats.pop(0)
                self.set(survivor_role, goal=retreats[0], navigate=True)
        self.wait(.3)
        if self.args.continue_to is not None:
            # The subsequent traversal remains strict: both players must
            # actually swim/walk it, without another death skipping a section.
            self.recovering = False
            for index in range(self.args.death_step, self.args.continue_to+1):
                if index in self.args.save_before:
                    self.save_checkpoint(index)
                (self.folder / f'before-tail-step-{index:03d}.json').write_text(json.dumps(self.reports['host']))
                for role in self.commands:
                    self.set(role, step=index)
                self.until(lambda: self.arrived() or all(self.reached(role) for role in self.commands))
                self.completed.append(index)
                print('PASS continued route step', index, flush=True)
                if self.arrived():
                    break
            assert self.completed == list(range(self.args.start_step, self.args.continue_to+1))
            if self.args.continue_to == len(self.steps)-1:
                self.until(self.arrived, 60)
                for role, rows in self.history.items():
                    route = [r for r in rows if r['epoch'] == 1 and r['map'] == self.map_name]
                    assert all(any(r['players'][i]['swimming'] for r in route) for i in (0, 1)), 'Missing paired swim'
                    assert self.reports[role]['visit']['entry'] == 'wchess2_start1'
                    for before, after in zip(route[-1]['players'], self.reports[role]['frame']['players']):
                        for field in ('sanity', 'owned', 'collected', 'school_items', 'staff_component'):
                            assert before['stats'][field] == after['stats'][field], 'Arrival changed ' + field
        initial_spawns = [b['spawned_at'] for b in self.saved['frame']['players']]
        routes = {role: [r for r in rows if r['epoch'] == 1 and r['map'] == self.map_name]
                  for role, rows in self.history.items()}
        recovery = {role: verify_revives(rows, role, initial_spawns) for role, rows in routes.items()}
        assert recovery['host'] == recovery['guest'], 'Clients disagree about revival timing'
        for role, rows in routes.items():
            for row in rows:
                if all(p['health'] > 0 for p in row['players']):
                    self.verify_presence([row], role)
            for before, after in zip(rows, rows[1:]):
                if all(p['health'] > 0 for r in (before, after) for p in r['players']) \
                        and all(a['spawned_at'] == b['spawned_at'] for a, b in zip(before['players'], after['players'])):
                    self.verify_presence([before, after], role)
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')
        self.evidence.update(recovery=recovery, victim=victim, completed_steps=self.completed)
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=self.reports[role]['visit']['epoch'])
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route_passed=False,
            build=self.build, internet=not self.args.local, first=self.args.first,
            requested_victim=self.args.victim, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, continue_to=self.args.continue_to, evidence=self.evidence,
            scope='Real combat death and ten-second revival, with optional remaining-course traversal; no team-wipe or full-route credit'), indent=2))
        print('PASS real combat death, moving spectator camera and ten-second 25-health revival', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--start-step', type=int, default=100)
    parser.add_argument('--death-step', type=int, choices=[126], default=126)
    parser.add_argument('--continue-to', type=int,
                        help='Continue the course through this step after revival; later traversal must remain death-free')
    parser.add_argument('--save-before', nargs='*', type=int, default=[126])
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--victim', choices=('auto', 'blue', 'red'), default='auto',
                        help='Keep this player fighting while the partner retreats before the death')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    args.last_step = args.continue_to if args.continue_to is not None else args.death_step
    if not (0 < args.start_step <= args.death_step < 1000):
        parser.error('Choose an ordered, nonzero course segment')
    if args.continue_to is not None and not (args.death_step <= args.continue_to <= 160):
        parser.error('Continuation must end between the target encounter and the final course step')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = ChessRecovery(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, last_valid=pair.last_valid, completed=pair.completed)))
        raise
    finally:
        pair.close()
