"""Check that Pale Realm route evidence cannot hide free-play regressions."""
import copy
import unittest

from test_coop_chess import ChessPair


def row(board=None):
    return dict(owner=None, camera_player='Blue', driver=dict(chess_course=dict(step=0)),
                chess=dict(board=board, coop=dict(actor=0, reunion=False)),
                players=[dict(feet=[0, 0, 0], warped_at=0),
                         dict(feet=[48, 0, 0], warped_at=0)])


class PresenceGuards(unittest.TestCase):
    def setUp(self):
        self.pair = object.__new__(ChessPair)
        self.pair.steps = [dict(mode='walk')]

    def test_saved_actor_does_not_exempt_free_play_overlap(self):
        sample = row()
        self.pair.verify_presence([sample], 'host')
        for actor in (0, 1):
            sample['chess']['coop']['actor'] = actor
            sample['players'][1]['feet'] = [20, 0, 0]
            with self.assertRaisesRegex(AssertionError, 'Players overlap'):
                self.pair.verify_presence([sample], 'host')

    def test_saved_actor_does_not_exempt_wrong_camera(self):
        with self.assertRaises(AssertionError):
            self.pair.verify_presence([row()], 'guest')

    def test_free_play_warp_is_rejected_but_puzzle_reunion_is_allowed(self):
        before = row()
        after = copy.deepcopy(before)
        after['players'][0].update(feet=[500, 0, 0], warped_at=12)
        with self.assertRaisesRegex(AssertionError, 'Unscripted Pale Realm warp'):
            self.pair.verify_presence([before, after], 'host')
        before['chess']['board'] = dict(piece='Bishop')
        self.pair.verify_presence([before, after], 'host')

    def test_active_puzzle_keeps_the_other_player_frozen(self):
        before = row(dict(piece='Bishop'))
        after = copy.deepcopy(before)
        self.pair.verify_presence([before, after], 'host')
        after['players'][1]['feet'][0] += 1
        with self.assertRaisesRegex(AssertionError, 'Puzzle moved waiter'):
            self.pair.verify_presence([before, after], 'host')


if __name__ == '__main__':
    unittest.main()
