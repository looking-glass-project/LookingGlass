"""Reject plausible-looking but incorrect revival evidence."""
import copy
import unittest

from test_coop_chess_recovery import verify_revives


def sample(tick, left=None, spawned=0):
    red = 0 if tick == 1000 else 128
    blue = red-48
    dead = left is not None
    stats = dict(owned=[1, 1], collected=['personal-item'], school_items={}, staff_component=False)
    return dict(tick=tick, checkpoint_generation=0,
                lives=dict(states=[{'Spectating': dict(ticks_left=left)} if dead else 'Alive', 'Alive']),
                players=[dict(health=0 if dead else 25, spawned_at=spawned, feet=[blue, 0, 0], stats=copy.deepcopy(stats)),
                         dict(health=50, spawned_at=0, feet=[red, 0, 0], stats=copy.deepcopy(stats))],
                camera_player='Red' if dead else 'Blue',
                camera_target=[red if dead else blue, 0, 48],
                camera_subject_feet=[red if dead else blue, 0, 0])


def timeline():
    return [sample(1000, 1200), sample(1400, 800), sample(2200, spawned=2200)]


class RevivalGuards(unittest.TestCase):
    def test_real_countdown_moving_camera_and_low_health_revive(self):
        self.assertEqual(verify_revives(timeline(), 'host', [0, 0]),
                         [dict(player=0, death_tick=1000, spawn_tick=2200, health=25)])

    def test_countdown_must_keep_its_original_death_tick(self):
        rows = timeline()
        rows[1]['lives']['states'][0]['Spectating']['ticks_left'] = 799
        with self.assertRaisesRegex(AssertionError, 'death tick'):
            verify_revives(rows, 'host', [0, 0])

    def test_early_or_full_health_revive_is_rejected(self):
        for field, value in [('spawned_at', 2199), ('health', 100)]:
            with self.subTest(field=field):
                rows = timeline()
                rows[-1]['players'][0][field] = value
                with self.assertRaises(AssertionError):
                    verify_revives(rows, 'host', [0, 0])

    def test_wrong_spectator_or_lost_inventory_is_rejected(self):
        rows = timeline()
        rows[0]['camera_player'] = 'Blue'
        with self.assertRaisesRegex(AssertionError, 'camera'):
            verify_revives(rows, 'host', [0, 0])
        rows = timeline()
        rows[-1]['players'][0]['stats']['owned'] = [1, 0]
        with self.assertRaisesRegex(AssertionError, 'personal'):
            verify_revives(rows, 'host', [0, 0])

    def test_missing_death_or_revive_does_not_pass(self):
        for rows in [timeline()[-1:], timeline()[:2]]:
            with self.assertRaises(AssertionError):
                verify_revives(rows, 'host', [0, 0])


if __name__ == '__main__':
    unittest.main()
