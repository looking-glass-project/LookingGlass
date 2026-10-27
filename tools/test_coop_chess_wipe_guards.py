"""Check that shared-wipe evidence rejects a plausible but wrong restore."""
import copy
import unittest

from test_coop_chess_wipe import verify_restore


def fixture():
    stats = dict(sanity=68, owned=[1, 1], collected=['personal'], selected=0,
                 school_items={}, staff_component=False, difficulty='Normal', will=1.)
    bodies = [dict(stats=copy.deepcopy(stats), player=dict(feet=[i*48, 0, 0]), spawned_at=0) for i in (0, 1)]
    bodies[1]['stats']['sanity'] = 50
    chess = dict(bishop_done=True, knight_done=True, fired=['bell'], water=None,
                 cast=[dict(entity=72, instance=0, piece=dict(health=8))])
    combat = dict(stage=dict(world=dict(levels=dict(wchess1=chess))))
    saved = dict(visit=dict(map='wchess1', epoch=1),
                 frame=dict(tick=1000, checkpoint_generation=0, players=bodies, combat=combat))
    frame = copy.deepcopy(saved['frame'])
    frame.update(tick=2000, checkpoint_generation=1, lives=dict(states=['Alive', 'Alive']))
    for b in frame['players']:
        b['spawned_at'] = 2000
    report = dict(frame=frame, combat=copy.deepcopy(combat), visit=copy.deepcopy(saved['visit']),
                  camera_player='Blue', camera_target=[0, 0, 48], camera_subject_feet=[0, 0, 0])
    return saved, report


class WipeGuards(unittest.TestCase):
    def test_exact_restore_and_bounded_walking_recovery_pass(self):
        saved, report = fixture()
        self.assertEqual(verify_restore(saved, report, 'host'), 2000)
        report['frame']['tick'] += 12
        for body in report['frame']['players']:
            body['stats']['will'] += .01
            body['player']['feet'][0] += 10
        verify_restore(saved, report, 'host')

    def test_wrong_health_or_inventory_is_rejected(self):
        for field, value in [('sanity', 100), ('owned', [1, 0]), ('collected', [])]:
            with self.subTest(field=field):
                saved, report = fixture()
                report['frame']['players'][0]['stats'][field] = value
                with self.assertRaises(AssertionError):
                    verify_restore(saved, report, 'host')

    def test_wrong_visit_or_enemy_restore_is_rejected(self):
        saved, report = fixture()
        report['visit']['map'] = 'wchess2'
        with self.assertRaisesRegex(AssertionError, 'visit'):
            verify_restore(saved, report, 'host')
        saved, report = fixture()
        report['combat']['stage']['world']['levels']['wchess1']['cast'][0]['piece']['health'] = 0
        with self.assertRaisesRegex(AssertionError, 'native encounter'):
            verify_restore(saved, report, 'host')

    def test_separate_respawns_or_wrong_positions_are_rejected(self):
        saved, report = fixture()
        report['frame']['players'][1]['spawned_at'] += 1
        with self.assertRaisesRegex(AssertionError, 'together'):
            verify_restore(saved, report, 'host')
        saved, report = fixture()
        report['frame']['players'][1]['player']['feet'][0] += 100
        with self.assertRaisesRegex(AssertionError, 'placement'):
            verify_restore(saved, report, 'host')

    def test_stale_observation_or_wrong_camera_is_rejected(self):
        saved, report = fixture()
        report['frame']['tick'] += 121
        with self.assertRaisesRegex(AssertionError, 'boundary'):
            verify_restore(saved, report, 'host')
        saved, report = fixture()
        report['camera_subject_feet'] = [48, 0, 0]
        report['camera_target'] = [48, 0, 48]
        with self.assertRaisesRegex(AssertionError, 'other body'):
            verify_restore(saved, report, 'host')


if __name__ == '__main__':
    unittest.main()
