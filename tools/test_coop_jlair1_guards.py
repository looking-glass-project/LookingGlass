"""Reject false personal arrival and scene evidence without running the game."""
import copy
import unittest

from test_coop_jlair1 import verify_arrival, verify_course, verify_combat_wait


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


if __name__ == '__main__': unittest.main()
