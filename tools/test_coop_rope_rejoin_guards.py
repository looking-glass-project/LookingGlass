"""Reject lost rope attachments, changed bodies and advancing disconnected worlds."""
import copy
import unittest

from coop_rope_rejoin import verify_rope_rejoin
from test_coop_countdown_guards import reports


def hanging(victim):
    before, after = reports(victim)
    before['playing'], after['playing'] = False, True
    for report in (before, after):
        report['combat'] = {'rope_clock': 12}
        for index, body in enumerate(report['frame']['players']):
            body['player'] = dict(feet=[index*72, 0, -100],
                rope=None if index == victim else dict(id=353, length=577., rise=0))
    return before, copy.deepcopy(before), after


class RopeRejoinGuards(unittest.TestCase):
    def test_both_death_roles_preserve_rope_and_clock(self):
        for victim in (0, 1):
            proof = verify_rope_rejoin(*hanging(victim), victim, 2)
            self.assertEqual(proof['remaining_before'] - proof['remaining_after'], 20)
            self.assertEqual(proof['rope_before'], proof['rope_after'])

    def test_lost_changed_or_climbing_grip_is_rejected(self):
        for fault in ('lost', 'id', 'length', 'rise'):
            before, held, after = hanging(1)
            player = after['frame']['players'][0]['player']
            if fault == 'lost': player['rope'] = None
            else: player['rope'][fault] += 1
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_rope_rejoin(before, held, after, 1, 2)

    def test_disconnected_world_and_dead_body_must_not_move(self):
        for fault in ('body', 'world', 'tick'):
            before, held, after = hanging(0)
            if fault == 'body': after['frame']['players'][0]['player']['feet'][0] += 1
            elif fault == 'world': held['combat']['rope_clock'] += 1
            else: held['frame']['tick'] += 1
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_rope_rejoin(before, held, after, 0, 2)

    def test_missing_or_late_fresh_guest_is_rejected(self):
        for fault in ('waiting', 'old_pid', 'late'):
            before, held, after = hanging(1)
            if fault == 'waiting': after['playing'] = False
            elif fault == 'old_pid': after['pid'] = 2
            else:
                after['frame']['tick'] = 321
                after['frame']['lives']['states'][1]['Spectating']['ticks_left'] = 879
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_rope_rejoin(before, held, after, 1, 2)


if __name__ == '__main__':
    unittest.main()
