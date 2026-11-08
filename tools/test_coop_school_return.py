"""Traverse the return school visit through two independent internet clients.

Starts at the authored return entrance with its normal visit loadout. Drives
ordinary movement, combat, shared boarding and the original Star/potion scenes.
No live position, health, inventory or quest writes. A full campaign chain and
the death/checkpoint matrix remain separate. Run rendered tests in Anode.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_school import SchoolPair
from test_coop_school2 import School2Pair, verify_reunion
from test_coop_village import Pair, verify_rejoin_stats


class SchoolReturnPair(SchoolPair):
    entry = 'skool1_start2'
    next_map = 'potears1'
    evidence_prefix = 'school-return-'

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'return-fixture-source.py').write_bytes(source)
        self.evidence['return_fixture_sha256'] = hashlib.sha256(source).hexdigest()
        self.saved = None
        self.resume_checkpoint = args.checkpoint is not None
        if self.resume_checkpoint:
            saved = args.checkpoint.read_bytes()
            payload = json.loads(saved)['payload']
            assert payload['visit']['map'] == self.map_name and payload['visit']['entry'] == self.entry
            assert payload['visit']['epoch'] == 1 and payload['frame']['checkpoint_generation'] == 0
            assert all(b['spawned_at'] == 0 and b['stats']['sanity'] > 0 for b in payload['frame']['players'])
            (self.folder / 'host' / 'coop-checkpoint.json').write_bytes(saved)
            self.saved = payload
            self.evidence.update(checkpoint_sha256=hashlib.sha256(saved).hexdigest(),
                                 resume_point=args.resume_point)

    def set(self, role, *args, **kwargs):
        super().set(role, *args, **kwargs)
        self.commands[role]['prefer_cards'] = True

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Return-school test stopped; partial evidence retained')
        # The return visit has a different scene clock from School One.
        Pair.pump(self)
        for role, report in self.reports.items():
            if self.history[role]:
                self.history[role][-1]['school'] = report['combat']['stage']['world']['school']
        if 'host' in self.reports:
            report = self.reports['host']
            owner = report['combat']['stage']['owner']
            if owner is not None:
                waiting = 'guest' if owner == 'Blue' else 'host'
                old = self.commands[waiting]
                if old['goal'] is not None:
                    self.set(waiting, capture=old['capture'], epoch=old['epoch'])

    # Both return and School Two need to wait for active foes and in-flight
    # waves, while an idle guard outside engagement range must not deadlock QA.
    secure = School2Pair.secure

    def state(self, role='host'):
        return self.school(role)['return_visit']

    def verify_rejoined_world(self, before, report):
        old = before['combat']['stage']['world']['school']
        new = report['combat']['stage']['world']['school']
        for field in ('books', 'returning', 'theatre', 'library', 'shelves', 'recipe'):
            assert old[field] == new[field], f'Rejoin changed return school {field}'
        for field in ('phase', 'star', 'potion'):
            assert old['return_visit'][field] == new['return_visit'][field], f'Rejoin changed return quest {field}'
        old_scene, new_scene = old['return_visit'].get('scene'), new['return_visit'].get('scene')
        if old_scene is None:
            assert new_scene is None, 'Resume started a return scene'
            return
        assert not old_scene['active'] and not new_scene['active'], 'Rejoin restarted the Star scene'
        assert {k: v for k, v in old_scene.items() if k != 'time'} == \
               {k: v for k, v in new_scene.items() if k != 'time'}, 'Rejoin changed completed Star scene'
        elapsed = report['frame']['tick'] - before['frame']['tick']
        assert elapsed >= 0 and abs(new_scene['time'] - min(30, old_scene['time'] + elapsed / 120)) < .005, \
            'Rejoin changed the continuing Star presentation clock'

    def entrance(self):
        # Finish on the flat lower landing. The old intermediate target on
        # the slope can keep delayed steering correcting back and forth;
        # this supported landing is on the recorded ordinary walking route.
        self.passage([((-1888, 3290, -649), False),
                      ((-1860, 2752, -512), True),
                      ((-1888, 2290, -512), True)], speed_limit=40)
        self.save_checkpoint('entrance')

    def theatre(self):
        self.go((-2340, 2368, -480), True)
        # The bottom stair has no separate supported waiting spot. Finish the
        # whole narrow passage before bringing the partner through it.
        self.passage([((-2256, 2544, -480), True),
                      ((-2304, 2624, -342), True), ((-2304, 2800, -256), True)])
        self.save_checkpoint('theatre')

    def library(self):
        # The reward detour is another narrow walkway. Each player completes
        # the loop to the broad passage before the other follows.
        # Descend and climb the observed lower supports before the short
        # upper jump, as on the first visit. A running plan for the whole
        # return can overshoot these narrow stairs on the delayed guest.
        # Finish east of the waiting player before continuing past them.
        self.passage([((-2272, 2440, -256), True), ((-2304, 2544, -352), False),
                      ((-2304, 2624, -342), False), ((-2304, 2720, -334), False),
                      ((-2256, 2800, -256), True), ((-2304, 3248, -256), False)],
                     speed_limit=20)
        # These intermediate supports come from the successful native route.
        # Complete the whole corner in turn, then fight inside the corridor.
        # Pairing at its near end puts the partner on an adjacent outer slope;
        # that is not a stable waiting place for the following planned turn.
        self.passage([((-1664, 3248, -256), False),
                      ((-1673, 3026, -250), True), ((-1477, 3020, -256), True),
                      ((-1200, 3008, -256), True)], first_exit=[((-1000, 3008, -256), False)])
        self.legs([((-208, 3008, -256), False), ((-208, 3968, -256), False),
                   ((208, 3968, -288), False), ((208, 4160, -264), False),
                   ((144, 4230, -288), False)])
        self.save_checkpoint('spiral')

    def upper_library(self):
        self.lift('library_spiral_lift', (144, 4360, -264), (192, 4360, -264), 64)
        # Use the first-school fixture's supported single-file lift exit; a
        # sideways endpoint at (272,4160) lies over the lower library floor.
        departure = tuple(self.bodies()[0 if self.first == 'host' else 1]['player']['feet'])
        self.passage([(departure, False), ((272, 4160, 64), False), ((700, 4096, 64), False)])
        self.legs([((884, 4056, 64), False), ((0, 4096, 64), False)])
        self.passage([((0, 3500, 64), False)])
        self.legs([((-180, 3500, 64), False), ((-180, 3300, 64), False), ((-140, 3300, 64), False)])
        self.passage([((208, 3300, 400), True), ((412, 3535, 384), True)])
        self.legs([((688, 4112, 384), True), ((692, 4170, 384), False)])
        self.until(lambda: all(self.state(r)['phase'] == 'Open' for r in self.reports), 15)
        assert all(not b['stats']['school_items']['star'] and b['stats']['school_items']['potion']
                   for r in self.reports for b in self.bodies(r)), 'Star consumption changed personal potion'
        self.save_checkpoint('observatory-approach')

    def observatory(self):
        def lift(role='host'):
            return next(o for o in self.school(role)['objects'] if o['name'] == 'observatory_lift')
        self.set(self.partner)
        self.set(self.first, (668, 4370, 384), navigate=True)
        self.until(lambda: self.reached(self.first), 45)
        self.set(self.first)
        before = {r: copy.deepcopy(lift(r)) for r in self.reports}
        clocks = {r: self.school(r)['clock'] for r in self.reports}
        self.until(lambda: all(self.school(r)['clock'] >= t + 2 for r, t in clocks.items()), 10)
        assert all(lift(r)['pose'] == old['pose'] and lift(r)['motion'] == old['motion']
                   and self.state(r)['phase'] == 'Open' for r, old in before.items()), \
            'Observatory departed before its living partner boarded'
        self.set(self.partner, (716, 4370, 384), navigate=True)
        self.until(lambda: all(r['route_driver']['scene'] == 'Skool1_OLift_Up'
                               for r in self.reports.values()), 45)
        for role in self.commands:
            self.set(role)
        self.until(lambda: all(self.state(r)['phase'] == 'Observatory'
                               and self.reports[r]['combat']['stage']['owner'] is None
                               and all(b['player']['grounded'] and b['player']['feet'][2] > 1015
                                       for b in self.bodies(r)) for r in self.reports), 45)
        self.evidence.setdefault('lifts', []).append(dict(name='observatory_lift',
            waiting_seconds=2, both_reached_height=1016, first=self.args.first))
        assert all(not b['stats']['school_items']['star'] and b['stats']['school_items']['potion']
                   for r in self.reports for b in self.bodies(r))
        self.save_checkpoint('observatory')
        if self.args.rejoin:
            self.rejoin_guest()
        print('PASS paired observatory wait, Star scene, physical ascent and reunion', flush=True)

    def exit_walk(self):
        self.legs([((692, 4160, 1024), False), ((692, 3860, 1024), False)])
        for goal in ((692, 3500, 1024), (-280, 3500, 1024), (-280, 3860, 1024)):
            self.go(goal, horizontal=True)
        # Reach the authored potion contact first. Its single-Alice performance
        # takes the waiting partner through the normal shared level transition.
        self.set(self.partner)
        # Use observed supported stair landings, then walk up the ramp like
        # the native route. Its old approximate goal height was below the
        # actual floor and cannot satisfy precise network landing checks.
        for goal, navigate, horizontal in (
                ((-272, 4077, 1215), True, False),
                ((-280, 4210, 1280), False, True),
                ((202, 4231, 1280), True, False),
                ((340, 4396, 1344), True, False)):
            self.set(self.first, goal, navigate=navigate, horizontal=horizontal)
            self.until(lambda: self.reached(self.first) or self.arrived()
                       or self.reports['host']['route_driver']['scene'] == 'observatory_exit_cinematic', 60)
            if self.arrived() or self.reports['host']['route_driver']['scene'] == 'observatory_exit_cinematic':
                break
        for role in self.commands:
            self.set(role)
        self.until(self.arrived, 90)

    def verify(self):
        expected = {'Skool1_OLift_Up', 'observatory_exit_cinematic'}
        if self.args.resume_point == 'observatory':
            expected.remove('Skool1_OLift_Up')
        assert self.captures == expected, f'Missing return scenes: {self.captures}'
        for role, rows in self.history.items():
            visit = [r for r in rows if r['epoch'] == 1]
            owners = {s: {r['owner'] for r in visit if r['scene'] == s} for s in expected}
            assert all(ids == {self.args.first.title()} for ids in owners.values()), owners
            reunions = set()
            for old, row in zip(visit, visit[1:]):
                if old['owner'] is not None and row['owner'] == old['owner']:
                    waiting = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][waiting]['feet'] == old['players'][waiting]['feet'], 'Waiting Alice moved in return scene'
                if old['owner'] is not None and row['owner'] is None:
                    verify_reunion(old, row)
                    reunions.add(old['scene'])
            if 'Skool1_OLift_Up' in expected:
                assert 'Skool1_OLift_Up' in reunions, 'Star scene handoff was not observed'
            assert all(r['checkpoint_generation'] == 0 for r in rows), 'Route silently restored a checkpoint'
            assert all(b['spawned_at'] == 0 and b['health'] > 0 for r in rows for b in r['players']), 'Unexpected death/respawn'
            assert self.reports[role]['visit']['entry'] == 'potears1_start1'
            assert all(not b['stats']['school_items']['star'] and not b['stats']['school_items']['potion']
                       for b in self.bodies(role)), 'Exit left unspent quest rewards'
            self.evidence[role] = dict(owners={s: sorted(ids) for s, ids in owners.items()})
        self.verify_coherence()

    def run(self):
        self.start_pair()
        self.wait(1)
        if self.saved:
            before = dict(frame=self.saved['frame'], combat=self.saved['frame']['combat'])
            for report in self.reports.values():
                assert report['visit'] == self.saved['visit']
                for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                    assert distance(body['player']['feet'], old['player']['feet']) < 2, 'Resume moved a player'
                    verify_rejoin_stats(old['stats'], body['stats'], report['frame']['tick'] - self.saved['frame']['tick'])
                self.verify_rejoined_world(before, report)
        else:
            assert all(b['stats']['school_items']['star'] and b['stats']['school_items']['potion']
                       for r in self.reports for b in self.bodies(r)), 'Return visit lacks its normal earned quest prerequisites'
        steps = [(None, self.entrance), ('entrance', self.theatre), ('theatre', self.library),
                 ('spiral', self.upper_library), ('observatory-approach', self.observatory), ('observatory', self.exit_walk)]
        start = next(i for i, (name, _) in enumerate(steps) if name == self.args.resume_point)
        for _, step in steps[start:]:
            step()
        self.verify()
        self.evidence['final_health'] = [b['stats']['sanity'] for b in self.bodies()]
        self.finish_pair(2)
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, first=self.args.first, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope=(f'Resumed return-school {self.args.resume_point} to Pool exit only; not a complete visit' if self.saved else
                   'Return-school entrance to Pool exit, paired lifts, original scenes and rejoin; normal visit loadout, campaign chain and death matrix separate')), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true')
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--resume-point', choices=('entrance', 'theatre', 'spiral', 'observatory-approach', 'observatory'))
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if bool(args.checkpoint) != bool(args.resume_point):
        parser.error('Provide both --checkpoint and --resume-point for a partial repeat')
    pair = SchoolReturnPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
