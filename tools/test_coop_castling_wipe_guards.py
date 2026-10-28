"""Reject plausible but incorrect Castling checkpoint restoration evidence."""
import copy
import unittest

from test_coop_chess_wipe_guards import fixture as body_fixture
from test_coop_castling_wipe import verify_restore


def fixture():
    saved, report = body_fixture()
    castle = dict(queen=True, queen_release=10, king=False, portal=None, fired=['queen'],
                  cast=[dict(entity=698, instance=0, piece=dict(health=150))])
    for visit in (saved['visit'], report['visit']):
        visit['map'] = 'wchess2'
    for combat in (saved['frame']['combat'], report['combat']):
        combat['stage']['world']['levels'] = dict(wchess2=copy.deepcopy(castle))
    return saved, report


class CastlingWipeGuards(unittest.TestCase):
    def test_saved_pair_and_bounded_walk_after_restore_pass(self):
        saved, report = fixture()
        self.assertEqual(verify_restore(saved, report, 'host'), 2000)
        report['frame']['tick'] += 12
        for body in report['frame']['players']:
            body['stats']['will'] += .01
            body['player']['feet'][0] += 10
        verify_restore(saved, report, 'host')

    def test_personal_resources_cannot_reset_or_merge(self):
        for field, value in [('sanity', 100), ('owned', [1, 0]), ('collected', [])]:
            saved, report = fixture()
            report['frame']['players'][0]['stats'][field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_restore(saved, report, 'host')

    def test_world_and_native_encounter_must_restore(self):
        for mutation in ('royals', 'events', 'enemy'):
            saved, report = fixture()
            castle = report['combat']['stage']['world']['levels']['wchess2']
            if mutation == 'royals':
                castle['king'] = True
            elif mutation == 'events':
                castle['fired'] = []
            else:
                castle['cast'][0]['piece']['health'] = 0
            with self.subTest(mutation=mutation), self.assertRaises(AssertionError):
                verify_restore(saved, report, 'host')

    def test_wrong_respawn_time_place_visit_and_camera_fail(self):
        for mutation in ('split', 'stale', 'placement', 'visit', 'camera'):
            saved, report = fixture()
            if mutation == 'split':
                report['frame']['players'][1]['spawned_at'] += 1
            elif mutation == 'stale':
                report['frame']['tick'] += 121
            elif mutation == 'placement':
                report['frame']['players'][1]['player']['feet'][0] += 100
            elif mutation == 'visit':
                report['visit']['map'] = 'rchess1'
            else:
                report['camera_player'] = 'Red'
            with self.subTest(mutation=mutation), self.assertRaises(AssertionError):
                verify_restore(saved, report, 'host')


if __name__ == '__main__':
    unittest.main()
