"""Reject incomplete or unobserved personal Funhouse traversal claims."""
import copy
import unittest

from test_coop_funhouse import FunhousePair, verify_arrivals, verify_switch_receipt, verify_holds, waiting_point, clock_threats, following_point, verify_following, nearby_drops, verify_loot, shooting_neighbor, cards_preference
from test_coop_funhouse import fulcrum_waiting_point, fulcrum_balance_goal, fulcrums_reset
from test_coop_funhouse import resident_boojum_threats


def fixture():
    steps = [dict(mode='wait', pos=[0, 0, 0]), dict(pos=[100, 100, 0])]
    receipts, history = [], {}
    for slot, role in enumerate(('host', 'guest')):
        bodies = [dict(feet=[100, 100 + 48*i, 0], grounded=True, health=100, spawned_at=0) for i in (0, 1)]
        bodies[slot]['feet'] = [100, 100, 0]
        bodies[1-slot]['feet'] = [100, 148, 0]
        history[role] = [dict(tick=10+slot, owner=None, players=bodies)]
        receipts.append(dict(step=1, role=role, goal=[100, 100, 0], tick=10+slot, feet=[100, 100, 0]))
    return steps, [0, 1], receipts, history


class PersonalArrivalGuards(unittest.TestCase):
    def test_clear_includes_resident_boojums_and_remaining_screams(self):
        boojum = dict(active=True, health=56, feet=[1315, -3764, 352], waves=[])
        report = dict(frame=dict(players=[dict(player=dict(feet=[1920, -3600, 0])),
                                         dict(player=dict(feet=[1920, -2164, 0]))]),
                      route_driver=dict(boojums=[dict(id=0, name='resident_boojum_0', active=True,
                                                     health=56, distance=[702, 1700], line_of_sight=[True, False])]),
                      combat=dict(npcs=dict(actors=[dict(resident=dict(active=True, awake=True,
                                                                       body=dict(Boojum=boojum)))])))
        self.assertEqual(resident_boojum_threats(report, 'host'), [0])
        self.assertEqual(resident_boojum_threats(report, 'guest'), [])
        report['route_driver']['boojums'][0]['line_of_sight'][0] = False
        self.assertEqual(resident_boojum_threats(report, 'host'), [])
        report['route_driver']['boojums'][0]['line_of_sight'][0] = True
        boojum['health'] = 0
        self.assertEqual(resident_boojum_threats(report, 'host'), [])
        boojum['waves'] = [dict(position=[1920, -3550, 42])]
        self.assertEqual(resident_boojum_threats(report, 'host'), [0])
        report['route_driver']['boojums'] = []
        with self.assertRaises(AssertionError):
            resident_boojum_threats(report, 'host')

    def test_tilted_arrival_needs_the_named_native_support(self):
        steps = [dict(pos=[0, 0, 0]) for _ in range(102)]
        steps[101] = dict(pos=[1920, -3400, 0])
        receipts, history = [], {}
        for slot, role in enumerate(('host', 'guest')):
            target = [1920, -3400, -178]
            body = dict(feet=target, grounded=True, health=100, spawned_at=0)
            native = dict(entity=460, requested=steps[101]['pos'], goal=target, supported=[True, True])
            history[role] = [dict(tick=10+slot, owner=None, players=[body, body],
                                 driver=dict(command=7+slot, surface=native))]
            receipts.append(dict(step=101, role=role, goal=steps[101]['pos'], feet=target, target=target,
                                 tick=10+slot, command=7+slot, surface=460))
        verify_arrivals(steps, [101], receipts, history)
        for mutation in ('lower_floor', 'wrong_board', 'wrong_goal', 'unacknowledged'):
            wrong = copy.deepcopy(history)
            driver = wrong['guest'][0]['driver']
            if mutation == 'lower_floor': driver['surface']['supported'][1] = False
            elif mutation == 'wrong_board': driver['surface']['entity'] = 459
            elif mutation == 'wrong_goal': driver['surface']['goal'][2] -= 80
            else: driver['command'] += 1
            with self.assertRaises(AssertionError):
                verify_arrivals(steps, [101], receipts, wrong)

    def test_final_seesaw_is_included_in_reset_gate(self):
        state = dict(fulcrums=[[0, 0] for _ in range(63)])
        self.assertTrue(fulcrums_reset([state, state]))
        tilted = copy.deepcopy(state)
        tilted['fulcrums'][62] = [-45, 0]
        self.assertFalse(fulcrums_reset([state, tilted]))
        self.assertFalse(fulcrums_reset([tilted, state]))

    def test_waiter_balances_on_a_supported_lateral_pivot_approach(self):
        points = [[1974.9, -3601.7, .03125], [1926.9, -3553.7, .03125], [1960.9, -3567.7, .03125]]
        anchor = fulcrum_waiting_point(points, [1920, -3600, -16])
        self.assertEqual(anchor, points[0])
        self.assertEqual(fulcrum_balance_goal(anchor, 0), [1974.9, -3600, 0])
        for angle in (-45, -10, 10, 45):
            goal = fulcrum_balance_goal(anchor, angle)
            self.assertEqual(goal[0], anchor[0])
            self.assertGreater((goal[1] + 3600) * angle, 0)
            self.assertLessEqual(abs(goal[1] + 3600), 220)
        with self.assertRaises(AssertionError):
            fulcrum_waiting_point(points[1:], [1920, -3600, -16])

    def test_moving_approach_requires_native_window_and_acknowledged_walk(self):
        steps = [dict(pos=[0, 0, 0]) for _ in range(96)]
        steps[95] = dict(pos=[1920, -2650, 0])
        receipts, history = [], {}
        for slot, role in enumerate(('host', 'guest')):
            body = dict(feet=[1920, -2634, -14], grounded=True, health=100, spawned_at=0)
            history[role] = [dict(tick=10+slot, owner=None, players=[body, body],
                                 driver=dict(command=7+slot, phase='walking', expected=None))]
            receipts.append(dict(step=95, role=role, goal=steps[95]['pos'], feet=body['feet'],
                                 tick=10+slot, command=7+slot, crossing=True))
        verify_arrivals(steps, [95], receipts, history)
        for field, value in [('feet', [1920, -2634, -100]), ('grounded', False)]:
            wrong = copy.deepcopy(history)
            claimed = copy.deepcopy(receipts)
            wrong['guest'][0]['players'][1][field] = value
            if field == 'feet':
                claimed[1]['feet'] = value
            with self.assertRaises(AssertionError):
                verify_arrivals(steps, [95], claimed, wrong)
        wrong = copy.deepcopy(history)
        wrong['guest'][0]['driver']['phase'] = 'planning'
        with self.assertRaises(AssertionError):
            verify_arrivals(steps, [95], receipts, wrong)
        wrong = copy.deepcopy(history)
        wrong['guest'][0]['driver']['command'] += 1
        with self.assertRaises(AssertionError):
            verify_arrivals(steps, [95], receipts, wrong)

    def test_resumed_course_retains_the_last_authored_cards_choice(self):
        steps = [dict(pos=[0, 0, 0]), dict(mode='cards', pos=[0, 0, 0]),
                 dict(pos=[200, 0, 0]), dict(mode='cards', pos=[1, 0, 0])]
        self.assertTrue(cards_preference(steps, 1))
        self.assertFalse(cards_preference(steps, 3))
        self.assertTrue(cards_preference(steps, 4))

    def test_authored_cards_preference_changes_only_requested_controls(self):
        pair = FunhousePair.__new__(FunhousePair)
        pair.serial, pair.lead, pair.commands = 0, .3, {}
        pair.args = type('Args', (), {'skip_scenes': False})()
        for enabled in (False, True):
            pair.prefer_cards = enabled
            pair.set('host', [400, 0, 0], run=True)
            self.assertEqual(pair.commands['host']['prefer_cards'], enabled)
            self.assertTrue(pair.commands['host']['attack'])
            self.assertEqual(pair.commands['host']['goal'], [400, 0, 0])

    def test_drop_selection_uses_personal_live_nearby_inventory(self):
        drop = dict(id=32, origin=[699, 2239, 3], grade='Large', remaining=7.9)
        body = dict(player=dict(feet=[800, 2293, 0]), stats=dict(sanity=3, will=.6,
            collected=[], loot={'funhouse$first': dict(drops=[drop])}))
        self.assertEqual(nearby_drops(body)[0]['key'], 'drop:funhouse$first:32')
        for change in ('collected', 'expired', 'distant', 'full'):
            test = copy.deepcopy(body)
            if change == 'collected':
                test['stats']['collected'].append('drop:funhouse$first:32')
            elif change == 'expired':
                test['stats']['loot']['funhouse$first']['drops'][0]['remaining'] = 0
            elif change == 'distant':
                test['player']['feet'] = [0, 0, 0]
            else:
                test['stats'].update(sanity=100, will=100)
            self.assertEqual(nearby_drops(test), [])

    def test_drop_verification_rejects_partner_credit_and_vanishing_pickups(self):
        key = 'drop:funhouse$first:32'
        old = dict(health=3, collected=[], stats=dict(loot={'funhouse$first': dict(
            drops=[dict(id=32, origin=[699, 2239, 3], remaining=7.9)])}))
        new = copy.deepcopy(old)
        new.update(health=53, collected=[key])
        receipts = [dict(role='host', key=key, origin=[699, 2239, 3], start_tick=1, tick=2)]
        history = dict(host=[dict(tick=1, owner=None, players=[old, old]),
                             dict(tick=2, owner=None, players=[new, old])])
        verify_loot(receipts, history)
        for bodies in ([old, new], [old, old]):
            bad = copy.deepcopy(history)
            bad['host'][1]['players'] = bodies
            with self.assertRaises(AssertionError):
                verify_loot(receipts, bad)

    def test_following_waits_for_space_and_uses_an_observed_footing(self):
        trail = [dict(tick=100, feet=[0, 0, 0]), dict(tick=200, feet=[96, 0, 0])]
        self.assertIsNone(following_point(trail[:1], [64, 0, 0], [-60, 0, 0]))
        self.assertEqual(following_point(trail, [192, 0, 0], [0, 0, 0]), trail[1])
        self.assertEqual(following_point(trail, [192, 0, 0], [-60, 0, 0]), trail[0])

    def test_clockwork_fists_and_awake_enemies_prevent_false_clear(self):
        report = dict(frame=dict(players=[dict(player=dict(feet=[0, 0, 0]))]*2),
                      combat=dict(npcs=dict(actors=[dict(clock=dict(active=True, health=400,
                          phase='Fast', feet=[650, 0, 0], fists=[]))])))
        self.assertEqual(clock_threats(report, 'host'), [0])
        clock = report['combat']['npcs']['actors'][0]['clock']
        clock['phase'] = 'Idle'
        self.assertEqual(clock_threats(report, 'host'), [])
        clock.update(feet=[900, 0, 0], fists=[dict(victim=5000002)])
        self.assertEqual(clock_threats(report, 'guest'), [0])
        clock['health'] = 0
        self.assertEqual(clock_threats(report, 'guest'), [0], 'A dead Automaton can still have a live fist')
        clock['fists'] = []
        self.assertEqual(clock_threats(report, 'guest'), [])

    def test_follower_receipts_reject_invented_or_dead_footings(self):
        _, _, receipts, history = fixture()
        entry = dict(leader='host', follower='guest', tick=10, feet=receipts[0]['feet'])
        verify_following([entry], history)
        for change in (dict(tick=99), dict(feet=[0, 0, 0]), dict(follower='host')):
            with self.assertRaises(AssertionError):
                verify_following([dict(entry, **change)], history)
        history['host'][0]['players'][0]['health'] = 0
        with self.assertRaises(AssertionError):
            verify_following([entry], history)

    def test_death_suspends_inputs_until_the_game_revives_the_player(self):
        pair = FunhousePair.__new__(FunhousePair)
        pair.serial, pair.lead = 0, .3
        pair.args = type('Args', (), {'skip_scenes': False})()
        pair.allow_revival, pair.suspended, pair.commands = True, {}, {}
        pair.survivor_wait, pair.moving_role = {}, None
        body = dict(stats=dict(sanity=0), player=dict(feet=[100, 0, 0]), spawned_at=0)
        pair.reports = dict(host=dict(frame=dict(players=[body])))
        pair.set('host', [400, 0, 0], run=True)
        original = copy.deepcopy(pair.reports)
        pair.resume_after_death()
        self.assertEqual(pair.reports, original)
        self.assertIsNone(pair.commands['host']['goal'])
        self.assertFalse(pair.commands['host']['attack'])
        self.assertTrue(pair.commands['host']['recovery_trace'])
        neutral_id = pair.commands['host']['id']
        pair.resume_after_death()
        self.assertEqual(pair.commands['host']['id'], neutral_id)
        body['stats']['sanity'], body['spawned_at'] = 25, 1200
        body['player']['feet'] = [200, 48, 0]
        original = copy.deepcopy(pair.reports)
        pair.resume_after_death()
        self.assertEqual(pair.reports, original)
        self.assertEqual(pair.commands['host']['goal'], [400, 0, 0])
        self.assertTrue(pair.commands['host']['attack'])
        self.assertGreater(pair.commands['host']['id'], neutral_id)
        self.assertFalse(pair.suspended)

    def test_waiting_survivor_can_dodge_then_resumes_the_parked_goal(self):
        pair = FunhousePair.__new__(FunhousePair)
        pair.serial, pair.lead = 0, .3
        pair.args = type('Args', (), {'skip_scenes': False})()
        pair.allow_revival, pair.suspended, pair.commands = True, {}, {}
        pair.survivor_wait, pair.moving_role = {}, 'host'
        bodies = [dict(stats=dict(sanity=0)), dict(stats=dict(sanity=70))]
        pair.reports = {role: dict(frame=dict(players=bodies)) for role in ('host', 'guest')}
        pair.set('host', [400, 0, 0], run=True)
        pair.set('guest', [300, 48, 0], run=True)
        pair.resume_after_death()
        self.assertIsNone(pair.commands['guest']['goal'])
        self.assertTrue(pair.commands['guest']['attack'] and pair.commands['guest']['combat_tactics'])
        self.assertEqual(bodies[1]['stats']['sanity'], 70)
        bodies[0]['stats']['sanity'] = 25
        pair.resume_after_death()
        self.assertEqual(pair.commands['guest']['goal'], [300, 48, 0])
        self.assertFalse(pair.survivor_wait)

    def test_waiting_point_keeps_clearance_after_arrival_tolerance(self):
        # Captured step 34: the old target 41.6 units from the waypoint let
        # Alice stop just 34.4 units away, inside the partner's destination.
        at, incoming, goal = [802.83, 1493.52, .03], [757.70, 1492.52, .03], [800, 1500, 0]
        unsafe, safe = [802.83, 1541.52, .03], [802.83, 1445.52, .03]
        self.assertEqual(waiting_point([unsafe, safe], at, incoming, goal), safe)
        with self.assertRaises(AssertionError):
            waiting_point([unsafe], at, incoming, goal)

    def test_both_observed_arrivals_pass(self):
        verify_arrivals(*fixture())

    def test_missing_or_duplicate_partner_cannot_pass(self):
        for duplicate in (False, True):
            args = fixture()
            args[2].pop()
            if duplicate:
                args[2].append(copy.deepcopy(args[2][0]))
            with self.assertRaises(AssertionError):
                verify_arrivals(*args)

    def test_unobserved_relocated_or_dead_arrivals_cannot_pass(self):
        for field, value in [('feet', [1000, 100, 0]), ('grounded', False), ('health', 0), ('spawned_at', 10)]:
            args = fixture()
            args[3]['guest'][0]['players'][1][field] = value
            with self.assertRaises(AssertionError):
                verify_arrivals(*args)
        args = fixture()
        args[2][1]['tick'] = 99
        with self.assertRaises(AssertionError):
            verify_arrivals(*args)

    def test_neutral_save_request_clears_previous_attack(self):
        # The first real run rejected its save because inherited attack input
        # survived a neutral command. Exercise that exact command sequence.
        pair = FunhousePair.__new__(FunhousePair)
        pair.serial, pair.lead = 0, .3
        pair.args = type('Args', (), {'skip_scenes': False})()
        pair.commands = {}
        pair.set('host', [100, 100, 0])
        self.assertTrue(pair.commands['host']['attack'])
        pair.set('host')
        pair.commands['host']['checkpoint'] = True
        self.assertFalse(pair.commands['host']['attack'])
        self.assertIsNone(pair.commands['host']['goal'])

    def test_jump_arrival_requires_a_new_observed_jump(self):
        args = fixture()
        args[0][1]['mode'] = 'jump'
        for receipt in args[2]:
            role = receipt['role']
            slot = 0 if role == 'host' else 1
            earlier = copy.deepcopy(args[3][role][0])
            earlier['tick'] = 9
            earlier['players'][slot]['jumps'] = 0
            args[3][role].insert(0, earlier)
            args[3][role][-1]['players'][slot]['jumps'] = 1
            receipt.update(start_tick=9, jumps_before=0, jumps_after=1)
        verify_arrivals(*args)
        # A supported 14-unit offset is inside the original jump controller's
        # 20-unit limit; the exact boundary must still fail.
        args[3]['guest'][-1]['players'][1]['feet'] = [114, 100, 0]
        args[2][1]['feet'] = [114, 100, 0]
        verify_arrivals(*args)
        args[3]['guest'][-1]['players'][1]['feet'] = [120, 100, 0]
        args[2][1]['feet'] = [120, 100, 0]
        with self.assertRaises(AssertionError):
            verify_arrivals(*args)
        args[3]['guest'][-1]['players'][1]['feet'] = [100, 100, 0]
        args[2][1]['feet'] = [100, 100, 0]
        args[3]['guest'][-1]['players'][1]['jumps'] = 0
        with self.assertRaises(AssertionError):
            verify_arrivals(*args)


def shot_fixture():
    def row(tick, active, phase='idle', command=None, reached=False, expected=None):
        return dict(tick=tick, funhouse=dict(mirror=active, cells=2),
                    driver=dict(command=command, phase=phase, reached=reached, expected=expected))
    entry = dict(role='host', command=7, thread='MirrorTrigger', before_ticks=dict(host=10, guest=10))
    history = dict(host=[row(10, False), row(11, False, 'shooting switch', 7, expected=[10, 20, 30]),
                         row(12, True, 'switch completed', 7, True)],
                   guest=[row(10, False), row(12, True)])
    return entry, history


class SwitchReceiptGuards(unittest.TestCase):
    def test_shooting_approach_stays_near_home_and_clear_of_partner(self):
        home, other, target = [-1232, -52, 0], [-1230, -88, 0], [-1043, -16, 288]
        safe, blocked, far = [-1184, -52, 0], [-1232, -100, 0], [-1088, -52, 0]
        self.assertEqual(shooting_neighbor([safe, blocked, far], home, other, target, [home]), safe)
        self.assertIsNone(shooting_neighbor([safe, blocked, far], home, other, target, [home, safe]))

    def test_shooting_receipt_rejects_an_invented_approach(self):
        entry, history = shot_fixture()
        point = [-1184, -52, 0]
        entry['repositions'] = [dict(before=10, after=11, goal=point)]
        history['host'][0]['driver']['neighbors'] = [[point], []]
        history['host'][1]['players'] = [dict(health=100, grounded=True, feet=point), {}]
        verify_switch_receipt(entry, history)
        history['host'][0]['driver']['neighbors'][0] = []
        with self.assertRaises(AssertionError):
            verify_switch_receipt(entry, history)

    def test_observed_target_and_both_shared_states_pass(self):
        verify_switch_receipt(*shot_fixture())

    def test_missing_target_completed_command_or_partner_state_fails(self):
        for mutation in ('target', 'completed', 'partner', 'already'):
            entry, history = shot_fixture()
            if mutation == 'target':
                history['host'].pop(1)
            elif mutation == 'completed':
                history['host'][-1]['driver']['reached'] = False
            elif mutation == 'partner':
                history['guest'][-1]['funhouse']['mirror'] = False
            else:
                history['guest'][0]['funhouse']['mirror'] = True
            with self.assertRaises(AssertionError):
                verify_switch_receipt(entry, history)


def hold_fixture():
    goal = [665, 775, -104]
    steps = [dict(mode='hold', pos=goal, frames=120)]
    entries = [dict(step=0, goal=goal, frames=120, starts=dict(host=100, guest=100),
                    ends=dict(host=220, guest=220), commands=dict(host=1, guest=2))]
    history = {role: [dict(tick=tick, driver=dict(command=command, phase='holding toward goal', expected=goal))
                      for tick in (100, 160, 220)] for role, command in [('host', 1), ('guest', 2)]}
    return steps, [0], entries, history


class HoldGuards(unittest.TestCase):
    def test_both_players_hold_toward_the_authored_point_for_the_required_ticks(self):
        verify_holds(*hold_fixture())

    def test_short_interrupted_or_missing_partner_hold_fails(self):
        for mutation in ('short', 'interrupted', 'partner', 'direction'):
            args = hold_fixture()
            if mutation == 'short':
                args[2][0]['ends']['guest'] = 219
            elif mutation == 'interrupted':
                args[3]['guest'][1]['driver']['phase'] = 'scene'
            elif mutation == 'partner':
                args[2][0]['starts'].pop('guest')
            else:
                args[3]['host'][-1]['driver']['expected'] = [0, 0, 0]
            with self.assertRaises(AssertionError):
                verify_holds(*args)


if __name__ == '__main__':
    unittest.main()
