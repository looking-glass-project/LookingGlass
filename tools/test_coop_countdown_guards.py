"""Keep countdown-rejoin acceptance from passing resets or stale client state."""
import copy
import unittest

from test_coop_play import recovery_report, verify_countdown_rejoin


def reports(victim=1):
    stats = dict(sanity=60, will=15, owned=[True, False], collected=['personal'],
        selected=0, school_items=[], staff_component=False, difficulty='Normal')
    bodies = [dict(stats=copy.deepcopy(stats), spawned_at=0, warped_at=0) for _ in range(2)]
    bodies[victim]['stats']['sanity'] = 0
    states = ['Alive', 'Alive']
    states[victim] = {'Spectating': {'ticks_left': 1000}}
    before = dict(pid=1, build='test', visit={'map': 'skool1'},
        frame=dict(tick=200, checkpoint_generation=0, players=bodies, lives={'states': states}))
    after = copy.deepcopy(before)
    after.update(pid=3, camera_player='Blue' if victim == 1 else 'Red')
    after['frame']['tick'] = 220
    after['frame']['lives']['states'][victim]['Spectating']['ticks_left'] = 980
    return before, after


class CountdownGuards(unittest.TestCase):
    def test_restart_preserves_either_victim_and_allows_real_survivor_damage(self):
        for victim in (0, 1):
            before, after = reports(victim)
            after['frame']['players'][1-victim]['stats']['sanity'] -= 10
            evidence = verify_countdown_rejoin(before, after, victim, 2)
            self.assertEqual(evidence['remaining_before']-evidence['remaining_after'], 20)
            self.assertEqual(evidence['death_tick'], 0)

    def test_stale_pid_clock_reset_wipe_and_replaced_body_are_rejected(self):
        for fault in ('pid', 'clock', 'countdown', 'alive', 'wipe', 'spawn', 'camera', 'visit', 'health'):
            before, after = reports()
            frame = after['frame']
            if fault == 'pid': after['pid'] = 2
            elif fault == 'clock': frame['tick'] = 199
            elif fault == 'countdown': frame['lives']['states'][1]['Spectating']['ticks_left'] = 1000
            elif fault == 'alive': frame['lives']['states'][1] = 'Alive'
            elif fault == 'wipe': frame['checkpoint_generation'] = 1
            elif fault == 'spawn': frame['players'][1]['spawned_at'] = 220
            elif fault == 'camera': after['camera_player'] = 'Red'
            elif fault == 'health': frame['players'][0]['stats']['sanity'] = 100
            else: after['visit'] = {'map': 'gvillage'}
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_countdown_rejoin(before, after, 1, 2)

    def test_personal_inventory_cannot_reset_on_either_body(self):
        for player in (0, 1):
            for field, value in [('will', 100), ('owned', [False, True]), ('collected', [])]:
                before, after = reports()
                after['frame']['players'][player]['stats'][field] = value
                with self.subTest(player=player, field=field), self.assertRaisesRegex(AssertionError, 'Rejoin changed'):
                    verify_countdown_rejoin(before, after, 1, 2)

    def test_lifecycle_checker_rejects_a_timer_reset_between_samples(self):
        history = []
        for tick in (1, 2):
            history.append(dict(tick=tick, checkpoint_generation=0,
                players=[dict(health=60, spawned_at=0), dict(health=0, spawned_at=0)],
                lives=dict(states=['Alive', {'Spectating': {'ticks_left': 1200}}])))
        with self.assertRaisesRegex(AssertionError, 'countdown restarted'):
            recovery_report(history, 'host', {}, 1)


if __name__ == '__main__':
    unittest.main()
