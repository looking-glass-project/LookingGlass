"""Reject a different checkpoint, partial restores and stale spectator views."""
import copy
import unittest

from test_coop_hatter2_wipe import verify_wipe_receipt, verify_restored_view


def fixture():
    stats = dict(sanity=100, will=100, difficulty='Normal', owned=['Blade'], collected={},
                 selected='Blade', school_items=0, staff_component=False)
    level = dict(phase='Fight', blade=True, watch=False, checkpoint=True, bridge=1,
                 exit=dict(committed=False), coop=dict(items=dict(blade=[True, True])), boss=dict(health=2600))
    bodies = [dict(player=dict(feet=[i * 48, 0, 0]), stats=copy.deepcopy(stats),
                   spawned_at=0, warped_at=0) for i in range(2)]
    stage = dict(world=dict(levels=dict(hatter2=level)))
    saved = dict(visit=dict(map='hatter2', epoch=1), frame=dict(tick=100, players=bodies,
                 combat=dict(stage=stage)))
    report = copy.deepcopy(saved)
    report['frame'].update(tick=1001, checkpoint_generation=1, lives=dict(states=['Alive', 'Alive']))
    for body in report['frame']['players']: body['spawned_at'] = 1000
    report.update(combat=copy.deepcopy(saved['frame']['combat']), camera_player='Blue')
    receipt = dict(passed=True, build='test', previous_generation=0, generation=1,
                   tick=1000, recovered_backup=False, checkpoint=copy.deepcopy(saved))
    return saved, report, receipt


class AboutFaceWipeGuards(unittest.TestCase):
    def test_exact_checkpoint_receipt_cannot_be_reused_for_other_state_or_tick(self):
        saved, _, receipt = fixture()
        verify_wipe_receipt(receipt, saved, 'test', 1000)
        for key, value in [('passed', False), ('build', 'other'), ('tick', 999),
                           ('generation', 2), ('previous_generation', 1), ('recovered_backup', True)]:
            bad = copy.deepcopy(receipt); bad[key] = value
            with self.subTest(key=key), self.assertRaises(AssertionError):
                verify_wipe_receipt(bad, saved, 'test', 1000)
        bad = copy.deepcopy(receipt); bad['checkpoint']['frame']['players'][1]['stats']['sanity'] = 25
        with self.assertRaises(AssertionError): verify_wipe_receipt(bad, saved, 'test', 1000)

    def test_client_must_restore_both_players_and_their_own_camera(self):
        saved, report, _ = fixture()
        self.assertEqual(verify_restored_view(saved, report, 'host'), 1000)
        for mutate in (lambda r:r.update(camera_player='Red'),
                       lambda r:r['frame']['players'][1].update(spawned_at=999),
                       lambda r:r['frame']['players'][1]['stats'].update(sanity=25),
                       lambda r:r['frame']['players'][1]['player'].update(feet=[900, 0, 0]),
                       lambda r:r['frame'].update(tick=1201)):
            bad = copy.deepcopy(report); mutate(bad)
            with self.assertRaises(AssertionError): verify_restored_view(saved, bad, 'host')

    def test_client_cannot_keep_post_save_rewards_or_defeated_boss(self):
        saved, report, _ = fixture()
        for mutate in (lambda s:s.update(watch=True), lambda s:s.update(phase='Victory'),
                       lambda s:s['boss'].update(health=0),
                       lambda s:s['coop']['items'].update(blade=[True, False])):
            bad = copy.deepcopy(report); mutate(bad['combat']['stage']['world']['levels']['hatter2'])
            with self.assertRaises(AssertionError): verify_restored_view(saved, bad, 'host')


if __name__ == '__main__': unittest.main()
