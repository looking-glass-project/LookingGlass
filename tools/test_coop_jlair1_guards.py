"""Reject false personal arrival and scene evidence without running the game."""
import copy
import unittest

from test_coop_jlair1 import verify_arrival, verify_course


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


if __name__ == '__main__': unittest.main()
