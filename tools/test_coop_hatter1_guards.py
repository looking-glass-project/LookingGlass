"""Reject incomplete or unobserved Clockwork route receipts."""
import copy
import unittest
from test_coop_hatter1 import parking_target, verify_arrivals


class Arrivals(unittest.TestCase):
    def setUp(self):
        self.steps = [dict(nav=[10, 20, 30]), dict(wait=1)]
        self.receipts = []
        self.history = {}
        for slot, role in enumerate(('host', 'guest')):
            body = dict(feet=[10,20,30], health=75, spawned_at=0, grounded=True)
            row = dict(tick=100+slot, owner=None, map='hatter1', epoch=1, players=[copy.deepcopy(body),copy.deepcopy(body)],
                       driver=dict(command=5+slot, reached=True, expected=[10,20,30]))
            self.history[role] = [row]
            self.receipts.append(dict(step=0,role=role,tick=100+slot,goal=[10,20,30],feet=[10,20,30],command=5+slot))

    def check(self):
        verify_arrivals(self.steps, [0,1], self.receipts, self.history)

    def test_complete_personal_receipts(self):
        self.check()

    def test_missing_second_player(self):
        self.receipts.pop()
        with self.assertRaisesRegex(AssertionError, 'Missing personal'): self.check()

    def test_unobserved_tick(self):
        self.receipts[1]['tick'] += 1
        with self.assertRaisesRegex(AssertionError, 'not observed'): self.check()

    def test_unacknowledged_command(self):
        self.history['guest'][0]['driver']['command'] += 1
        with self.assertRaisesRegex(AssertionError, 'not acknowledged'): self.check()

    def test_wrong_requested_goal(self):
        self.receipts[0]['goal'][0] += 1
        with self.assertRaisesRegex(AssertionError, 'differs'): self.check()

    def test_planner_point_is_not_the_requested_goal(self):
        self.history['host'][0]['driver']['expected'] = None
        self.history['guest'][0]['driver']['expected'] = [9, 19, 30]
        self.check()

    def test_airborne_or_revived_arrival(self):
        for field, value in [('grounded',False), ('spawned_at',99), ('health',0)]:
            with self.subTest(field=field):
                row = self.history['guest'][0]['players'][1]
                old = row[field]; row[field] = value
                with self.assertRaisesRegex(AssertionError, 'Invalid arrival'): self.check()
                row[field] = old

    def test_missed_position(self):
        self.history['host'][0]['players'][0]['feet'][0] += 9
        self.receipts[0]['feet'][0] += 9
        with self.assertRaisesRegex(AssertionError, 'Missed personal'): self.check()

    def test_cinematic_receipt(self):
        self.history['host'][0]['owner'] = 'Blue'
        with self.assertRaisesRegex(AssertionError, 'not observed'): self.check()


class Parking(unittest.TestCase):
    def test_far_side_preferred_over_incoming_doorway(self):
        candidates = [[-3295,2701,192], [-3261,2783,192], [-3329,2783,192]]
        self.assertEqual(parking_target(candidates, [-3295,2749,192], [-3331,2994,192], [-3296,2750,192]),
                         (candidates[0], True))

    def test_near_ring_is_only_an_approach(self):
        candidates = [[45, 0, 0], [20, 20, 0]]
        self.assertEqual(parking_target(candidates, [3,0,0], [-80,0,0], [0,0,0]), ([45,0,0], False))

    def test_second_ring_clears_arrival_lane(self):
        self.assertEqual(parking_target([[90,0,0]], [45,0,0], [-80,0,0], [0,0,0]), ([90,0,0], True))

    def test_no_forward_progress_is_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'No observed walking'):
            parking_target([[2,0,0]], [3,0,0], [-80,0,0], [0,0,0])

    def test_partner_space_remains_reserved(self):
        with self.assertRaisesRegex(AssertionError, 'No observed walking'):
            parking_target([[60,0,0]], [0,0,0], [80,0,0], [0,0,0])


if __name__ == '__main__': unittest.main()
