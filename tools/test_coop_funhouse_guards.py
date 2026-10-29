"""Reject incomplete or unobserved personal Funhouse traversal claims."""
import copy
import unittest

from test_coop_funhouse import FunhousePair, verify_arrivals


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


if __name__ == '__main__':
    unittest.main()
