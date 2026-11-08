"""Traverse School Two with two independent clients and ordinary inputs.

Public internet is the default. No live position, inventory, health or quest writes.
--stop-after explicitly selects a partial entrance component, never a full visit.
--stage-at-altar opts into a staged initial spawn for a shorter component.
Run rendered tests in Anode; retained captures need not be opened for routine QA.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path

from test_coop_school import SchoolPair
from test_coop_village import Pair
from test_coop_play import ROOT, distance


def verify_reunion(old, row):
    """Match normal placement or the existing safe-partner fallback."""
    a, b = row['players']
    assert a['warped_at'] == b['warped_at'] > 0, 'Scene did not reunite both presentation tracks'
    travel = max(0, row['tick'] - a['warped_at']) * 640 / 120
    separation = distance(a['feet'], b['feet'])
    if separation <= 110 + travel:
        return 'placed'
    # placement::reunion_with_partner can retain an already supported nearby
    # pad when a second scene has no new landing. Only that unchanged partner
    # earns its 128-unit limit; a newly displaced body keeps the normal bound.
    other = 1 if old['owner'] == 'Blue' else 0
    before, after = old['players'][other], row['players'][other]
    assert before['grounded'] and all(p['grounded'] for p in row['players']), 'Retained reunion is airborne'
    assert before['feet'] == after['feet'], 'Distant reunion displaced the waiting partner'
    assert abs(a['feet'][2] - b['feet'][2]) <= 32 + travel, 'Retained reunion changed landing height'
    assert distance(a['feet'][:2], b['feet'][:2]) >= 40 - travel, 'Retained reunion overlaps the owner'
    assert separation <= 128 + travel, 'Retained reunion is too far from the owner'
    return 'retained'


def verify_quest_completion(rows):
    """Check collection and consumption in order, including both inventories."""
    ingredients = {'mushroom', 'spice', 'lollipop', 'jumbogrow'}
    phases = [
        ('Jumbogrow', {'mushroom', 'spice', 'jumbogrow'}, set()),
        ('Growing', {'mushroom', 'spice'}, {'jumbogrow'}),
        ('Lollipop', {'mushroom', 'spice', 'lollipop'}, {'jumbogrow'}),
        ('FinalDialogue', {'mushroom', 'spice', 'lollipop'}, {'jumbogrow'}),
        ('Rewards', set(), ingredients),
        ('Complete', {'potion', 'star'}, ingredients),
    ]
    remaining = iter(rows)
    evidence = {}
    for phase, present, consumed in phases:
        for row in remaining:
            quest = row['school']['quest']
            inventories = [quest['items']] + [b['stats']['school_items'] for b in row['players']]
            if quest['stage'] == phase and all(
                    all(items[k] for k in present) and all(not items[k] for k in consumed)
                    for items in inventories):
                evidence[phase] = row['tick']
                break
        else:
            raise AssertionError(f'Missing ordered quest evidence for {phase}: collection or consumption differs')
    return evidence


class School2Pair(SchoolPair):
    map_name = 'skool2'
    next_map = 'skool1'
    evidence_prefix = 'school2-route-'
    expected_scenes = {'dice_cat', 'extendBleachers', 'Old_Gnome_Mushroom',
                       'OG1_Warp', 'Old_Gnome_SpiceDrops', 'Skool2_GrowLollypop',
                       'Skool2_LastGnome_Cinema'}
    # The former point (1750, -2275, 440) lies inside gym_light1 during part
    # of its swing. Wait on the supported diagonal walkway beside the lamp.
    gym_exit = (1680, -2350, 440)

    @property
    def first(self):
        # A narrow-pad departure may use a different walking order, without
        # changing the requested scene owners checked against args.first.
        return getattr(self, '_departure_first', None) or super().first

    def __init__(self, args):
        super().__init__(args)
        # The complete route takes about 27 minutes. Keep its internal limit
        # below the runner's 29-minute limit, leaving time for cleanup instead
        # of inheriting the common pair fixture's 25-minute cutoff.
        if not args.stage_at_altar and args.stop_after is None:
            self.deadline += 210
        if args.stage_at_altar:
            self.start_at = (-992, -2816, -192)
            self.evidence['staged_initial_spawn'] = self.start_at
        source = Path(__file__).read_bytes()
        (self.folder / 'school2-fixture-source.py').write_bytes(source)
        self.evidence['school2_fixture_sha256'] = hashlib.sha256(source).hexdigest()
        self.evidence['portals'] = []

    def set(self, role, *args, use_aim=None, **kwargs):
        super().set(role, *args, **kwargs)
        self.commands[role]['use_aim'] = use_aim
        self.commands[role]['prefer_cards'] = True

    def school(self, role='host'):
        return self.reports[role]['combat']['stage']['world']['school2']

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('School Two test stopped; partial evidence retained')
        # Use the common network recorder; School One's first_cinema has a
        # different schema. Complete shared-world hashes also cover School Two.
        Pair.pump(self)
        if 'host' in self.reports:
            report = self.reports['host']
            owner = report['combat']['stage']['owner']
            if owner is not None:
                waiting = 'guest' if owner == 'Blue' else 'host'
                old = self.commands[waiting]
                if old['goal'] is not None:
                    # secure() holds a waiting body at its current position.
                    # A scene will reunite it elsewhere: release that old goal
                    # during the scene, before it can walk back off the pad.
                    self.set(waiting, capture=old['capture'], epoch=old['epoch'])
                    self.evidence.setdefault('scene_goal_clearances', []).append(dict(
                        role=waiting, goal=old['goal'], scene=report['route_driver']['scene'],
                        tick=report['frame']['tick']))
        for role, report in self.reports.items():
            if self.history[role]:
                self.history[role][-1]['school'] = report['combat']['stage']['world']['school2']
                self.history[role][-1]['network_timing'] = report.get('network_timing')
                self.history[role][-1]['frame_timing'] = report.get('frame_timing')

    def secure(self, keep_movement=False, radius=700):
        if self.arrived():
            return
        # The altar ambush can push an idle partner off its narrow platform.
        # Fight with ordinary steering back toward each grounded waiting spot,
        # just as the moving player holds their goal; never pin/write a body.
        for role in self.commands:
            body = self.bodies()[0 if role == 'host' else 1]
            if body['player']['grounded'] and (not keep_movement or self.commands[role]['goal'] is None):
                self.set(role,body['player']['feet'],arrival_radius=3.0)
        self.hold(.5)

        def safe():
            for report in self.reports.values():
                driver = report['route_driver']
                # An idle guard outside its engagement range is not a fight
                # to wait for at this waypoint. Continue the ordinary route;
                # it will wake on approach. Active foes and waves still block.
                foes = [e for e in driver['guards'] if e['state'] != 'Idle'] + driver['boojums']
                if any(e['active'] and e['health'] > 0 and any(v and d < radius
                       for v, d in zip(e['line_of_sight'], e['distance'])) for e in foes):
                    return False
                world = report['combat']['stage']['world']
                cast = world['encounters']
                if cast and any(a['enemy'].get('Boojum', {}).get('waves') for a in cast['actors']):
                    return False
                if world['school2'] and any(b['waves'] for b in world['school2']['boojums']):
                    return False
            return True

        self.until(safe, 90)

    def dice(self):
        stairs = [(-600, -2560, 0), (-800, -2560, 0), (-872, -2496, 0),
                  (-1000, -2496, -48), (-1120, -2496, -80), (-1120, -2608, -112),
                  (-1120, -2736, -176), (-1120, -2816, -192), (-992, -2816, -192)]
        if not self.args.stage_at_altar:
            self.go((-64, -3100, 0))
            self.legs([((-320, -2800, 0), False), ((-600, -2560, 0), False)])
            self.passage([(p, False) for p in stairs], speed_limit=20)
        altar = (-906, -3170, -192)
        self.passage([(altar, True)],partner_done=lambda:
            all('skool2:44' in b['stats']['collected'] for r in self.reports for b in self.bodies(r)),
            first_exit=[(stairs[-1], True)])
        self.wait(.5)
        assert all('skool2:44' in b['stats']['collected'] for b in self.bodies()), 'Personal Demon Dice missing'
        # Both have collected, either after reunion or individual approaches.
        # Leave from their supported positions without a needless swap across
        # the occupied altar on this narrow platform.
        return_path = [(stairs[-1], True)]
        if not self.args.stage_at_altar:
            return_path += [(p, False) for p in reversed(stairs[:-1])]
        # Reunion may place the original scene owner behind their partner on
        # this narrow pad. Let the front player leave first: there is no safe
        # sideways passing place, and contact would spoil a planned run-up.
        original_order = getattr(self, '_departure_first', None)
        nearest = min(('host','guest'), key=lambda role:
            distance(self.bodies()[0 if role == 'host' else 1]['player']['feet'],stairs[-1]))
        self._departure_first = nearest
        self.evidence['dice_return_first'] = nearest
        try:
            self.passage(return_path, speed_limit=20)
        finally:
            self._departure_first = original_order
        if not self.args.stage_at_altar:
            self.legs([((-320, -2800, 0), False), ((-64, -3100, 0), False)])
        print('PASS School Two personal Dice and return from altar', flush=True)

    def gym(self):
        self.legs([(p, False) for p in [(-64, -3100, 0), (250, -2800, 0),
                   (480, -2560, 0), (800, -2560, 0), (1150, -2560, 0),
                   (1030, -2560, 0), (1030, -2880, 0), (1200, -2880, 0),
                   (1470, -2880, 0), (1470, -3040, 0), (2150, -3040, 0), (2150, -2000, 0)]])
        self.gym_ascent()
        self.gym_finish()

    def gym_finish(self):
        # The diagonal walkway has no supported side-by-side waiting spot.
        # Cross the whole upper route before stepping aside for the partner.
        for role in self.commands:
            self.set(role, attack=True)
        endpoint = (2077, -3090, 440)
        first_index = 0 if self.first == 'host' else 1
        def reunited_at_gnome():
            # The Gnome scene normally brings the waiting player to a separate
            # supported pad. That is already a valid arrival; neither body
            # needs to squeeze sideways or repeat the old walkway afterward.
            return all(
                distance(r['frame']['players'][first_index]['player']['feet'][:2], endpoint[:2]) < 8
                and all(b['stats']['school_items']['mushroom'] and b['player']['grounded']
                        and distance(b['player']['feet'][:2], endpoint[:2]) < 128
                        and abs(b['player']['feet'][2]-endpoint[2]) < 16
                        for b in r['frame']['players']) for r in self.reports.values())
        self.passage([(self.gym_exit, True), ((1472, -2560, 440), False),
                      ((1710, -2800, 440), False), ((1880, -2940, 440), True),
                      (endpoint, False)], after_reunion=[(endpoint, False)],
                     partner_done=reunited_at_gnome)
        self.secure(radius=1400)
        self.until(lambda: all(self.school(r)['quest']['stage'] == 'Laboratory' for r in self.reports), 45)
        # Laboratory opens at the start of OG1_Warp. Wait for the actual scene
        # and grounded reunion before moving on or verifying scene coverage.
        self.wait(.5)
        self.until(lambda: all(r['combat']['stage']['owner'] is None
                   and all(b['player']['grounded'] for b in r['frame']['players'])
                   for r in self.reports.values()), 90)
        print('PASS School Two gym lever, bleachers and three Boojums', flush=True)

    def gym_ascent(self):
        self.set(self.first, (2163, -1859, 64), navigate=True)
        self.until(lambda: self.reached(self.first), 45)
        self.set(self.first, (2163, -1859, 64))
        self.until(lambda: self.reached(self.first), 20)
        self.wait(.5)
        player = self.bodies(self.first)[0 if self.first == 'host' else 1]['player']
        # Same authored lever and eye convention as the native route; ordinary E.
        aim = [2244-player['feet'][0], -1859-player['feet'][1], 90-player['feet'][2]-48]
        self.set(self.first, attack=False,
                 use_aim=(math.atan2(aim[1], aim[0]), math.atan2(aim[2], math.hypot(*aim[:2]))))
        self.until(lambda: all(r['combat']['stage']['world']['gym']['used'] for r in self.reports.values()), 15)
        self.set(self.first, attack=True)
        self.wait(7)
        self.go((2150, -2080, 0), navigate=True)
        self.passage([((2150, -2080, 0), False), ((2400, -2150, 160), False),
                      ((2440, -2150, 192), False), ((2270, -2040, 250), True),
                      ((1950, -2016, 400), False), ((1710, -2000, 448), False),
                      ((1600, -2275, 440), True)],
                     first_exit=[(self.gym_exit, True)])

    def laboratory(self):
        # The return walkway has no paired waiting spot. Send each player
        # through the whole passage before the other leaves the reunion pad.
        self.passage([(p, True) for p in [(1880, -2940, 440), (1710, -2800, 440),
                      (1472, -2560, 440), self.gym_exit, (1600, -2275, 440),
                      (1710, -2000, 448), (1950, -2016, 400), (2270, -2040, 250),
                      (2440, -2150, 192), (2400, -2150, 160), (2150, -2080, 0)]])
        # The gym fight can leave its scene owner hurt. Share the next combat
        # approach with the healthier partner instead of making the same body
        # absorb every encounter while the other waits behind. Restore the
        # requested arrival order before entering the laboratory scenes.
        original_order = getattr(self, '_departure_first', None)
        health = [b['stats']['sanity'] for b in self.bodies()]
        self._departure_first = 'host' if health[0] > health[1] else 'guest' if health[1] > health[0] else self.first
        self.evidence['laboratory_approach_first'] = dict(role=self.first, health=health)
        try:
            self.legs([(p, False) for p in [(2150, -3040, 0), (1470, -3040, 0),
                       (1470, -2880, 0), (1440, -2880, 0), (1040, -2880, 0),
                       (1040, -2560, 0), (480, -2560, 0), (250, -2290, 0), (-64, -2060, 0)]])
        finally:
            self._departure_first = original_order
        self.save_checkpoint('laboratory-approach')
        self.laboratory_from_stairs()

    def laboratory_from_stairs(self):
        # The shortest search plan jumps onto the side of these curved stairs
        # and pulls up a ledge. A single tick of input jitter can miss it. Walk
        # around to the wide entrance and follow the supported stair centres;
        # pass through one at a time so neither player blocks a narrow tread.
        self.passage([(p, False) for p in [(-64, -2200, 0), (-416, -2200, 0),
                      (-416, -2480, 0), (-272, -2448, 48), (-192, -2400, 80),
                      (-128, -2336, 112), (-96, -2272, 128), (-32, -2272, 136),
                      (64, -2368, 184), (192, -2464, 240), (256, -2528, 256),
                      (416, -2528, 256), (432, -2400, 256), (288, -2208, 256),
                      (160, -2112, 256)]])
        # Regroup behind the doorway before waking its ranged guard. Holding
        # one Alice at the exposed door while the partner climbs makes her
        # absorb the entire fight. Approach together in short ordinary legs,
        # with the healthier player nearest the guard. Stay south of the
        # rescue trigger (y=-1232), then restore the requested scene order.
        original_order = getattr(self, '_departure_first', None)
        health = [b['stats']['sanity'] for b in self.bodies()]
        self._departure_first = 'host' if health[0] >= health[1] else 'guest'
        self.evidence['laboratory_guard_first'] = dict(role=self.first, health=health)
        try:
            for point in [(32, -1984, 256), (-64, -1744, 256), (-64, -1640, 256)]:
                self.go(point, fight=False)
            self.secure()
            assert all(self.school(r)['quest']['stage'] == 'Laboratory' for r in self.reports)
        finally:
            self._departure_first = original_order
        print('PASS School Two paired laboratory stairs and entrance fight', flush=True)
        self.save_checkpoint('laboratory-entrance')
        self.laboratory_rescue()

    def laboratory_rescue(self):
        self.go((-64, -1270, 256))
        self.wait(8)
        self.secure()
        # The rescue fight starts the Spice Drops scene after reaching this
        # point. Wait for its actual ending: go()'s short final landing check
        # cannot complete while cinematic control owns the player.
        self.set(self.partner)
        self.set(self.first, (-64, -1020, 256))
        self.until(lambda: all(r['route_driver']['scene'] == 'Old_Gnome_SpiceDrops'
                              for r in self.reports.values()), 90)
        for role in self.commands:
            self.set(role)
        self.until(lambda: all(self.school(r)['quest']['stage'] == 'Jumbogrow'
                   and self.reports[r]['combat']['stage']['owner'] is None
                   and all(b['player']['grounded'] for b in self.bodies(r))
                   for r in self.reports), 90)
        self.go((-64, -1020, 256))
        self.legs([((40, -930, 256), False), ((150, -600, 256), False), ((36, -452, 256), False)])
        self.until(lambda: all(self.school(r)['quest']['items']['jumbogrow'] for r in self.reports), 15)
        print('PASS School Two rescue, Spice Drops and Jumbogrow', flush=True)
        self.save_checkpoint('greenhouse')

    def portal(self, goal, clearance):
        def transfer_done(role, before):
            index = 0 if role == 'host' else 1
            return all(self.bodies(r)[index]['warped_at'] > before for r in self.reports)

        def record(role):
            index = 0 if role == 'host' else 1
            self.evidence['portals'].append(dict(role=role, goal=goal,
                warped_at=self.bodies()[index]['warped_at'], feet=self.bodies()[index]['player']['feet']))

        before = {role: self.bodies()[0 if role == 'host' else 1]['warped_at']
                  for role in (self.first, self.partner)}
        self.set(self.partner)
        self.set(self.first, goal)
        self.until(lambda: transfer_done(self.first, before[self.first]), 30)
        record(self.first)
        # Bring the partner through while the first player walks off the pad.
        # Waiting for a precise clearance first strands a hurt player in the
        # arrival fight for several extra seconds. Normal portal placement and
        # player collision still choose separate supported arrivals.
        self.set(self.first, clearance)
        self.set(self.partner, goal)
        self.until(lambda: transfer_done(self.partner, before[self.partner]), 30)
        record(self.partner)
        self.set(self.partner)
        self.until(lambda: self.reached(self.first) and all(
            b['player']['grounded'] for r in self.reports for b in self.bodies(r)), 30)
        index = 0 if self.first == 'host' else 1
        self.evidence.setdefault('portal_clearances', []).append(dict(
            role=self.first, goal=clearance, feet=self.bodies()[index]['player']['feet'],
            tick=self.reports['host']['frame']['tick']))

    def greenhouse(self):
        self.legs([(p, False) for p in [(150, -600, 256), (40, -930, 256),
                   (-64, -1270, 256), (-64, -1504, 256)]])
        self.portal((-312, -1504, 256), (-1120, -2540, 512))
        original_order = getattr(self, '_departure_first', None)
        health = [b['stats']['sanity'] for b in self.bodies()]
        self._departure_first = 'host' if health[0] >= health[1] else 'guest'
        self.evidence['greenhouse_corridor_first'] = dict(role=self.first, health=health)
        try:
            # Approach the corridor encounter with both players before waiting
            # for it to clear. The doorway can expose a distant guard without
            # giving the first player an effective shot from that stopping spot.
            for point in [(-1120, -2540, 512), (-1470, -2560, 512), (-1740, -2560, 512)]:
                self.go(point, fight=False)
            self.secure()
            self.legs([(p, False) for p in [(-2070, -2560, 512), (-2320, -2560, 512),
                       (-2640, -2560, 512), (-2780, -2560, 512)]])
        finally:
            self._departure_first = original_order
        self.go((-3010, -2550, 488), navigate=True)
        self.wait(7)
        if not all(self.school(r)['quest']['items']['lollipop'] for r in self.reports):
            self.go((-3010, -2550, 504), navigate=True)
        self.until(lambda: all(self.school(r)['quest']['items']['lollipop'] for r in self.reports), 15)
        self.go((-2780, -2560, 480), navigate=True)
        self.go((-2680, -2560, 512), navigate=True)
        self.secure()
        if self.args.rejoin:
            self.rejoin_guest()
        self.legs([(p, False) for p in [(-2320, -2560, 512), (-2070, -2560, 512),
                   (-1740, -2560, 512), (-1470, -2560, 512), (-1120, -2540, 512)]])
        self.portal((-1112, -2280, 512), (-64, -1504, 256))

    def rewards(self):
        self.legs([((-64, -1504, 256), False), ((-64, -1020, 256), False), ((0, -820, 256), False)])
        self.wait(9)
        self.until(lambda: all(self.school(r)['quest']['stage'] in ('Rewards', 'Complete') for r in self.reports), 45)
        self.legs([((-28, -695, 256), False)])
        # Both rewards can be collected during the first paired approach.
        # Do not force an extra crossing of the occupied condenser pad after
        # the quest has already confirmed both rewards for both players.
        complete = lambda: all(self.school(r)['quest']['stage'] == 'Complete'
            and all(b['stats']['school_items']['potion'] and b['stats']['school_items']['star']
                    for b in self.bodies(r)) for r in self.reports)
        if not complete():
            self.go((0, -760, 256))
        self.until(complete, 15)
        self.save_checkpoint('school2-complete')
        self.go((-64, -1270, 256))
        self.until(self.arrived, 90)

    def verify_rejoined_world(self, before, report):
        old = before['combat']['stage']['world']['school2']
        new = report['combat']['stage']['world']['school2']
        for field in ('stage', 'items'):
            assert new['quest'][field] == old['quest'][field], f'Rejoin changed School Two quest {field}'
        assert new['dice_guards'] == old['dice_guards']
        assert [o['enabled'] for o in new['objects']] == [o['enabled'] for o in old['objects']]
        assert [o['motion'] for o in new['objects']] == [o['motion'] for o in old['objects']]

    def verify(self, partial=None, completed_scenes=frozenset()):
        expected = {'dice_cat'} if partial == 'dice' else (
            {'dice_cat', 'extendBleachers', 'Old_Gnome_Mushroom', 'OG1_Warp'} if partial == 'gym' else self.expected_scenes)
        expected = expected - completed_scenes
        assert self.captures == expected, f'Unexpected scene coverage: {self.captures}'
        for role, rows in self.history.items():
            visit = [r for r in rows if r['epoch'] == 1]
            owners = {s: {r['owner'] for r in visit if r['scene'] == s} for s in expected}
            assert all(ids == {self.args.first.title()} for ids in owners.values()), owners
            for old, row in zip(visit, visit[1:]):
                if old['owner'] is not None and row['owner'] == old['owner']:
                    other = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][other]['feet'] == old['players'][other]['feet'], 'Partner moved during School Two scene'
                if old['owner'] is not None and row['owner'] is None:
                    kind = verify_reunion(old, row)
                    self.evidence.setdefault('verified_reunions', {}).setdefault(role, {})[row['tick']] = kind
            assert all(r['checkpoint_generation'] == 0 for r in rows)
            assert all(b['spawned_at'] == 0 and b['health'] > 0 for r in rows for b in r['players'])
            assert all('skool2:44' in b['stats']['collected'] for b in self.bodies(role))
            if partial:
                assert self.reports[role]['visit']['map'] == 'skool2' and self.reports[role]['visit']['epoch'] == 1
            if not partial:
                assert self.reports[role]['visit']['entry'] == 'skool1_start2'
                self.evidence.setdefault('quest_timeline', {})[role] = verify_quest_completion(visit)
                assert sum(p['role'] == role for p in self.evidence['portals']) == 2
            self.evidence[role] = dict(owners={s: sorted(ids) for s, ids in owners.items()})
        self.evidence['minimum_will'] = [min(r['players'][i]['stats']['will']
            for r in self.history['host']) for i in range(2)]
        self.verify_coherence()

    def milestone(self, phase):
        # Preserve a verified prefix when a later part of the full route fails.
        # This never creates a full-route result or bypasses its final checks.
        self.wait(.5)
        self.until(lambda: all(r['combat']['stage']['owner'] is None for r in self.reports.values()), 180)
        self.verify(phase)
        self.save_checkpoint(phase)
        (self.folder / f'milestone-{phase}.json').write_text(json.dumps(dict(
            passed=True, full_visit=False, phase=phase, build=self.build,
            scope=self.partial_scope(phase),
            first=self.args.first, internet=not self.args.local, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence), indent=2))
        print('PASS verified School Two component', phase, flush=True)

    def partial_scope(self, phase):
        return ('Staged School Two Dice altar: gap, personal pickups, scene, ambush and return; not entrance or full traversal'
                if self.args.stage_at_altar else f'School Two entrance through {phase}; partial component only')

    def run(self):
        self.start_pair()
        self.wait(1)
        self.dice()
        self.milestone('dice')
        if self.args.stop_after != 'dice':
            self.gym()
            self.milestone('gym')
            if self.args.stop_after != 'gym':
                self.laboratory()
                self.greenhouse()
                self.rewards()
        if not self.args.stop_after:
            self.verify()
        self.evidence['final_health'] = [b['stats']['sanity'] for b in self.bodies()]
        self.finish_pair(1 if self.args.stop_after else 2)
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, first=self.args.first, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope=self.partial_scope(self.args.stop_after) if self.args.stop_after else
                  'School Two traversal, scenes, personal Dice, quest and gated return; combat/recovery matrix separate'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true')
    parser.add_argument('--stop-after', choices=('dice', 'gym'))
    parser.add_argument('--stage-at-altar', action='store_true',
                        help='Focused component with staged initial spawn before the altar gap; requires --stop-after dice')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if args.rejoin and args.stop_after:
        parser.error('Reconnect is checked later in the full route')
    if args.stage_at_altar and args.stop_after != 'dice':
        parser.error('--stage-at-altar requires --stop-after dice; never a full traversal')
    pair = School2Pair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
