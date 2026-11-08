"""Negative controls for the public Castling route's presence evidence."""
import copy
import unittest

from test_coop_castling import verify_presence


class PresenceGuards(unittest.TestCase):
    def row(self):
        return dict(owner=None, scene=None, camera_player='Blue', players=[
            dict(feet=[0, 0, 0], warped_at=0), dict(feet=[48, 0, 0], warped_at=0)])

    def test_clear_pair_and_scripted_reunion(self):
        before = self.row()
        scene = copy.deepcopy(before)
        scene.update(owner='Blue', scene='cinema_king_thread')
        after = self.row()
        after['players'][1].update(feet=[60, 0, 0], warped_at=1)
        verify_presence([before, scene, after], 'host')

    def test_actual_overlap_fails(self):
        row = self.row()
        row['players'][1]['feet'] = [20, 0, 0]
        with self.assertRaisesRegex(AssertionError, 'overlap'):
            verify_presence([row], 'host')

    def test_unscripted_warp_fails(self):
        before, after = self.row(), self.row()
        after['players'][1]['warped_at'] = 1
        with self.assertRaisesRegex(AssertionError, 'Unscripted'):
            verify_presence([before, after], 'host')

    def test_scene_cannot_move_partner(self):
        before = self.row()
        before.update(owner='Blue', scene='cinema_king_thread')
        after = copy.deepcopy(before)
        after['players'][1]['feet'][0] += 1
        with self.assertRaisesRegex(AssertionError, 'waiter'):
            verify_presence([before, after], 'host')

    def test_wrong_personal_camera_fails(self):
        with self.assertRaisesRegex(AssertionError, 'camera'):
            verify_presence([self.row()], 'guest')


if __name__ == '__main__':
    unittest.main()
