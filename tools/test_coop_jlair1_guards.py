"""Reject false personal arrival and scene evidence without running the game."""
import copy
import unittest
from types import SimpleNamespace
from test_coop_jlair1 import forward_waiting_target, verify_recovered_planning_errors
from test_coop_jlair1 import SHELF_APPROACH, verify_shelf_approaches

from test_coop_jlair1 import verify_arrival, verify_course, verify_combat_wait, waiting_target, escape_target, hot_approach_target, verify_jump_flight, verify_forward_approaches, verify_platform_wait, verify_platform_carry, verify_platform_climb, verify_bank_wait, PLATFORM_BANK, PLATFORM_PARK, CuriosityPair


class CuriosityEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.goal = [10, 20, 30]
        self.receipt = dict(step=0, role='host', epoch=1, tick=10, command=4,
                            goal=self.goal, feet=list(self.goal))
        self.row = dict(map='jlair1', epoch=1, tick=10, owner=None,
            driver=dict(command=4, reached=True, clear=[True, True]),
            players=[dict(health=100, grounded=True, spawned_at=0, feet=list(self.goal)),
                     dict(health=100, grounded=True, spawned_at=0, feet=[100, 20, 30])])
        self.history = dict(host=[self.row], guest=[copy.deepcopy(self.row)])

    def test_partner_receipt_cannot_claim_local_arrival(self):
        verify_arrival(self.receipt, self.history)
        bad = dict(self.receipt, role='guest')
        with self.assertRaisesRegex(AssertionError, 'Arrival body changed'):
            verify_arrival(bad, self.history)

    def test_falling_revival_or_unacknowledged_body_cannot_pass(self):
        for field, value in [('grounded', False), ('spawned_at', 100)]:
            history = copy.deepcopy(self.history)
            history['host'][0]['players'][0][field] = value
            with self.assertRaises(AssertionError): verify_arrival(self.receipt, history)
        self.row['driver']['command'] = 3
        with self.assertRaisesRegex(AssertionError, 'Unacknowledged'):
            verify_arrival(self.receipt, self.history)

    def test_course_requires_both_personal_arrivals(self):
        with self.assertRaisesRegex(AssertionError, 'Missing personal arrival'):
            verify_course([dict(pos=self.goal)], [0], [self.receipt], self.history, 0, 0)
        guest = dict(self.receipt, role='guest')
        self.history['guest'][0]['players'][1]['feet'] = list(self.goal)
        verify_course([dict(pos=self.goal)], [0], [self.receipt, guest], self.history, 0, 0)
        with self.assertRaisesRegex(AssertionError, 'Duplicate'):
            verify_course([dict(pos=self.goal)], [0], [self.receipt, guest, guest], self.history, 0, 0)

    def test_shelf_guidance_requires_each_personal_footing_in_order(self):
        entries=[];history={role:[] for role in ('host','guest')}
        lower=[dict(step=65,role=r,tick=1) for r in history]
        upper=[dict(step=65,role=r,tick=20*(len(SHELF_APPROACH)+1)) for r in history]
        for slot,role in enumerate(history):
            for index,goal in enumerate(SHELF_APPROACH):
                tick=20*(index+1)
                row=copy.deepcopy(self.row);row['tick']=tick
                row['players'][slot]['feet']=list(goal);row['players'][slot]['spawned_at']=0
                history[role].append(row)
                entries.append(dict(self.receipt,step=65,role=role,index=index,goal=goal,feet=list(goal),tick=tick))
        verify_shelf_approaches(entries,history,[65],lower,upper)
        for changed in (entries[:-1],entries+[entries[0]]):
            with self.assertRaisesRegex(AssertionError,'Incomplete or repeated'):
                verify_shelf_approaches(changed,history,[65],lower,upper)
        early=[dict(e,tick=30) for e in upper]
        with self.assertRaisesRegex(AssertionError,'reordered or bypassed'):
            verify_shelf_approaches(entries,history,[65],lower,early)

    def test_platform_crossing_requires_both_actual_bank_arrivals(self):
        steps = [dict(pos=[0,0,0]) for _ in range(55)]
        steps[54] = dict(pos=[-7168,788,1615.0443], mode='jump')
        receipts = []
        for slot, role in enumerate(('host','guest')):
            self.history[role][0]['players'][slot]['feet'] = list(PLATFORM_BANK)
            receipts.append(dict(self.receipt, step=54, role=role, goal=PLATFORM_BANK, feet=list(PLATFORM_BANK)))
        verify_course(steps, [54], receipts, self.history, 54, 54)
        receipts[0]['goal'] = steps[54]['pos']
        with self.assertRaisesRegex(AssertionError, 'Waypoint changed'):
            verify_course(steps, [54], receipts, self.history, 54, 54)

    def test_ordinary_navigation_still_moves_both_players_and_parks_the_first(self):
        pair = CuriosityPair.__new__(CuriosityPair)
        pair.args = SimpleNamespace(first='blue'); pair.step = 1; pair.approach_goals = {}
        pair.evidence = dict(arrivals=[], approaches=[], parking=[])
        bodies = {r:dict(player=dict(feet=p)) for r,p in [('host',[0,0,0]),('guest',[-100,0,0])]}
        pair.body = lambda role: bodies[role]
        pair.clear = lambda: False
        pair.escape_hazard = lambda role: None
        pair.set = lambda *args, **kwargs: None
        pair.reports = dict(host=dict(frame=dict(tick=1), route_driver=dict(neighbors=[[[48,0,0]],[]])))
        moved = []
        def move(role, goal, navigate=False, **kwargs):
            moved.append((role, goal, navigate)); bodies[role]['player']['feet'] = list(goal)
            return dict(role=role, goal=goal)
        pair.move = move
        pair.go([0,0,0])
        self.assertEqual(moved, [('host',[0,0,0],True),('host',[48,0,0],False),('guest',[0,0,0],True)])
        self.assertEqual([r['role'] for r in pair.evidence['arrivals']], ['host','guest'])

    def test_combat_wait_needs_both_acknowledgements_and_living_players(self):
        history = {r:[dict(copy.deepcopy(self.row), tick=t) for t in (10, 40, 80)] for r in ('host','guest')}
        command = dict(id=4, epoch=1, goal=None, navigate=False, attack=True,
                       combat_tactics=True, prefer_cards=True, prefer_ice=False)
        entry = dict(start={r:10 for r in history}, end={r:80 for r in history},
                     commands={r:dict(command) for r in history})
        verify_combat_wait(entry, history)
        bad = copy.deepcopy(history)
        bad['guest'][-1]['driver']['command'] = 3
        with self.assertRaisesRegex(AssertionError, 'Unacknowledged'):
            verify_combat_wait(entry, bad)
        bad = copy.deepcopy(history)
        bad['host'][1]['players'][0]['health'] = 0
        with self.assertRaisesRegex(AssertionError, 'lost a player'):
            verify_combat_wait(entry, bad)

    def test_exhausted_waiting_ring_does_not_revisit_or_bypass_a_free_bay(self):
        at, incoming, goal = [0,48,0], [-160,-320,0], [0,0,0]
        # The only same-height neighbor leads straight back to the visited spot.
        assert waiting_target([[0,0,0]], at, incoming, goal, [[0,0,0]]) is None
        assert waiting_target([incoming], at, incoming, goal, []) is None
        chosen = waiting_target([[0,0,0],[0,96,0]], at, incoming, goal, [[0,0,0]])
        self.assertEqual(chosen, ([0,96,0], True))

    def test_hazard_exit_requires_a_recent_native_candidate_clear_of_partner(self):
        before = dict(tick=600, players=[dict(feet=[0,0,0]),dict(feet=[100,0,0])])
        row = dict(tick=500, driver=dict(neighbors=[[[60,0,0],[-60,0,0]],[]]))
        self.assertEqual(escape_target([row],0,before), (500,[-60,0,0]))
        row['tick'] = 100
        self.assertIsNone(escape_target([row],0,before))
        row['tick'] = 601
        self.assertIsNone(escape_target([row],0,before))
        row['tick'] = 500; row['driver']['neighbors'][0] = [[200,0,0],[60,0,0]]
        self.assertIsNone(escape_target([row],0,before))

    def test_hot_takeoff_requires_an_observed_high_pad_with_room(self):
        row = dict(players=[dict(feet=[0,0,0]),dict(feet=[80,0,32])],
                   driver=dict(neighbors=[[[80,0,32],[-80,0,0],[-160,0,32]],[]]))
        self.assertIsNone(hot_approach_target(row,0,[0,0,0]))
        row['driver']['neighbors'][0].append([-80,0,32])
        self.assertEqual(hot_approach_target(row,0,[0,0,0]),[-80,0,32])

    def test_hot_crossing_requires_a_real_jump_and_acknowledged_flight(self):
        def row(tick, jumps, grounded):
            return dict(tick=tick,driver=dict(command=3),players=[dict(jumps=jumps,grounded=grounded)])
        before, flight, after = row(10,2,True),row(20,3,False),row(30,3,True)
        verify_jump_flight(before,flight,after,0,3)
        with self.assertRaisesRegex(AssertionError,'left the ground'):
            verify_jump_flight(before,row(20,3,True),after,0,3)
        with self.assertRaisesRegex(AssertionError,'Missing ordinary jump'):
            verify_jump_flight(before,row(20,2,False),after,0,3)
        with self.assertRaisesRegex(AssertionError,'acknowledged'):
            verify_jump_flight(before,flight,after,0,4)

    def test_forward_waiting_requires_each_intermediate_personal_arrival(self):
        steps = [dict(pos=[0,0,0]), dict(pos=self.goal), dict(pos=[90,100,30])]
        entry = dict(step=0, role='host', navigation_start=1, selected_tick=20,
                     navigation_command=5, navigation_goal=steps[2]['pos'], passed_goals=[self.receipt])
        verify_forward_approaches(entry, steps, self.history)
        with self.assertRaisesRegex(AssertionError, 'skipped an original'):
            verify_forward_approaches(dict(entry, passed_goals=[]), steps, self.history)
        bad = dict(entry, passed_goals=[dict(self.receipt, role='guest')])
        with self.assertRaisesRegex(AssertionError, 'owner'):
            verify_forward_approaches(bad, steps, self.history)
        self.row['driver']['command'] = 3
        with self.assertRaisesRegex(AssertionError, 'Unacknowledged'):
            verify_forward_approaches(entry, steps, self.history)

    def test_platform_wait_rejects_partial_boarding_and_early_departure(self):
        history = {r:[dict(copy.deepcopy(self.row), tick=t) for t in (10,70,130)] for r in ('host','guest')}
        for rows in history.values():
            for row in rows:
                row['driver']['curiosity_platform'] = dict(full=[False,True], riders=[False,True], sink=0, descending=False)
        entry = dict(rider='guest', start={r:10 for r in history}, end={r:130 for r in history})
        verify_platform_wait(entry, history)
        bad = copy.deepcopy(history); bad['guest'][1]['driver']['curiosity_platform']['full'][1] = False
        with self.assertRaisesRegex(AssertionError, 'fully boarded'):
            verify_platform_wait(entry, bad)
        bad = copy.deepcopy(history); bad['host'][1]['driver']['curiosity_platform']['sink'] = 1
        with self.assertRaisesRegex(AssertionError, 'departed without'):
            verify_platform_wait(entry, bad)

    def test_bank_wait_rejects_falling_drift_and_damage(self):
        history = {r:[dict(copy.deepcopy(self.row), tick=t) for t in (10,40,80)] for r in ('host','guest')}
        for rows in history.values():
            for row in rows:
                row['players'][0].update(feet=list(PLATFORM_PARK), immersion=dict(kind='Dry'))
        entry = dict(role='host', start={r:10 for r in history}, end={r:80 for r in history})
        verify_bank_wait(entry, history)
        for change in (dict(grounded=False), dict(health=99), dict(feet=[0,0,0])):
            bad = copy.deepcopy(history); bad['guest'][1]['players'][0].update(change)
            with self.assertRaises(AssertionError): verify_bank_wait(entry, bad)

    def test_platform_carry_rejects_motion_without_rider_contact_or_body_descent(self):
        history = dict(host=[dict(copy.deepcopy(self.row), tick=t) for t in (10,40)])
        for row, sink in zip(history['host'], (10,35)):
            row['driver']['curiosity_platform'] = dict(sink=sink, descending=True, riders=[True,False])
            row['players'][0]['feet'][2] = 100-sink
        entry = dict(rider='host', before=10, after=40)
        verify_platform_carry(entry, history)
        initial = copy.deepcopy(history)
        initial['host'][0]['driver']['curiosity_platform'].update(sink=0, descending=False)
        initial['host'][0]['players'][0]['feet'][2] = 100
        verify_platform_carry(entry, initial)
        bad = copy.deepcopy(history); bad['host'][1]['driver']['curiosity_platform']['riders'][0] = False
        with self.assertRaisesRegex(AssertionError, 'actual platform contact'):
            verify_platform_carry(entry, bad)
        bad = copy.deepcopy(history); bad['host'][1]['players'][0]['feet'][2] = 90
        with self.assertRaisesRegex(AssertionError, 'did not follow'):
            verify_platform_carry(entry, bad)

    def test_platform_climb_requires_native_action_and_completed_landing(self):
        rows = [dict(copy.deepcopy(self.row), tick=t) for t in (10,20,30)]
        for row, count in zip(rows, (2,3,3)):
            row['players'][0].update(climb=None, climbs=count, jumps=7)
        rows[1]['players'][0].update(climb=dict(end=self.goal), grounded=False)
        history = dict(host=rows)
        entry = dict(role='host', before=10, active=20, after=30, command=4)
        verify_platform_climb(entry, history)
        bad = copy.deepcopy(history); bad['host'][1]['players'][0]['climb'] = None
        with self.assertRaisesRegex(AssertionError, 'never completed'):
            verify_platform_climb(entry, bad)
        bad = copy.deepcopy(history); bad['host'][2]['players'][0]['climbs'] = 2
        with self.assertRaisesRegex(AssertionError, 'ordinary climb'):
            verify_platform_climb(entry, bad)


    def test_forward_bay_rejects_a_fleeting_fast_landing(self):
        row = dict(players=[dict(feet=[0,0,0], velocity=[0,0,0], grounded=True,
                                immersion=dict(kind='Dry')), dict(feet=[-100,0,0])],
                   driver=dict(clear=[True,True], neighbors=[[[48,0,0]],[]]))
        self.assertEqual(forward_waiting_target(row,0,[0,0,0]), ([48,0,0],True))
        for change in (dict(velocity=[266,41,0]), dict(grounded=False), dict(immersion=dict(kind='Lava'))):
            bad = copy.deepcopy(row); bad['players'][0].update(change)
            self.assertIsNone(forward_waiting_target(bad,0,[0,0,0]))

    def test_only_grounded_liquid_planning_failure_can_attempt_escape(self):
        pair = CuriosityPair.__new__(CuriosityPair)
        report = dict(route_driver=dict(error='No route after 1 physics states'),
            frame=dict(players=[dict(stats=dict(sanity=50),
                player=dict(grounded=True,immersion=dict(kind='Lava')))]))
        self.assertTrue(pair.recoverable_driver_error('host',report))
        bad = copy.deepcopy(report); bad['route_driver']['error'] = 'Invalid command'
        self.assertFalse(pair.recoverable_driver_error('host',bad))
        for change in (dict(grounded=False), dict(immersion=dict(kind='Dry'))):
            bad = copy.deepcopy(report); bad['frame']['players'][0]['player'].update(change)
            self.assertFalse(pair.recoverable_driver_error('host',bad))
        bad = copy.deepcopy(report); bad['frame']['players'][0]['stats']['sanity'] = 0
        self.assertFalse(pair.recoverable_driver_error('host',bad))

    def test_planning_error_cannot_pass_without_its_own_completed_escape(self):
        history = dict(host=[dict(tick=10, driver=dict(error='No route after 1 physics states'))])
        exits = [dict(role='host', before_tick=10, tick=80)]
        verify_recovered_planning_errors(history, exits)
        for invalid in ([], [dict(exits[0], role='guest')], [dict(exits[0], before_tick=500, tick=600)]):
            with self.assertRaisesRegex(AssertionError, 'no verified escape'):
                verify_recovered_planning_errors(history, invalid)

    def test_forward_waiting_after_platform_cannot_cross_section_boundaries(self):
        steps = [dict(pos=self.goal) for _ in range(78)]
        entry = dict(step=55, role='host', navigation_start=1, selected_tick=20,
                     navigation_command=5, navigation_goal=self.goal,
                     passed_goals=[dict(self.receipt, step=55)])
        verify_forward_approaches(entry, steps, self.history)
        for step in (52, 53, 54, 76):
            with self.assertRaisesRegex(AssertionError, 'navigation section'):
                verify_forward_approaches(dict(entry, step=step, passed_goals=[]), steps, self.history)


if __name__ == '__main__': unittest.main()
