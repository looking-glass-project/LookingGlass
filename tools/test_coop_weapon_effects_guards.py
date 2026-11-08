"""Reject incomplete effect observations instead of accepting ambient particles."""
import copy
import unittest
from test_coop_weapon_effects import verify_effects


def history():
    row = dict(weapon_effects=[dict(sparks=12, trail=8, flight_trails=0),
                               dict(sparks=0, trail=0, flight_trails=4)])
    return {role: [copy.deepcopy(row)] for role in ('host', 'guest')}


class EffectsGuards(unittest.TestCase):
    def test_both_clients_observe_all_types_and_both_players(self):
        evidence = verify_effects(history())
        self.assertEqual(evidence['guest'][1]['flight_trails'], 4)

    def test_one_client_missing_a_type_fails(self):
        rows = history()
        rows['guest'][0]['weapon_effects'][0]['sparks'] = 0
        with self.assertRaisesRegex(AssertionError, 'missing effect type'):
            verify_effects(rows)

    def test_one_player_or_one_client_is_insufficient(self):
        rows = history()
        rows['guest'][0]['weapon_effects'][0]['flight_trails'] = 4
        rows['guest'][0]['weapon_effects'][1]['flight_trails'] = 0
        with self.assertRaisesRegex(AssertionError, 'missing emitting player'):
            verify_effects(rows)
        with self.assertRaisesRegex(AssertionError, 'Missing client'):
            verify_effects({'host': history()['host']})


if __name__ == '__main__':
    unittest.main()
