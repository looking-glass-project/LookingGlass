"""Reject stationary, driven, unsupported and stale moving-support evidence."""
import copy
import unittest

from coop_fulcrum_recovery import verify_fulcrum_rejoin, verify_fulcrum_recovery
from test_coop_countdown_guards import reports


def reconnect(victim):
    before, after = reports(victim)
    before['playing'], after['playing'] = False, True
    for r in (before, after):
        r['combat'] = dict(tilt=-30)
        for i, b in enumerate(r['frame']['players']):
            b['player'] = dict(feet=[i*48, 0, 0], grounded=i != victim, swimming=False, rope=None)
    return before, copy.deepcopy(before), after


def recovery(victim):
    event = dict(player=victim, epoch=1, death_tick=0, spawn_tick=1200, paused_ticks=0, wait_ticks=0)
    history = dict(host=[], guest=[])
    for tick, angle, z in ((1195, -5, 0), (1210, -6, 2), (1230, -8, 5), (1270, -12, 15), (1310, -16, 25)):
        for local, role in enumerate(('host', 'guest')):
            alive = tick >= 1200
            bodies = [dict(feet=[i*48, 0, z], health=25 if i == victim else 100,
                spawned_at=1200 if i == victim and alive else 0, warped_at=0,
                grounded=True, velocity=[0, 0, 0]) for i in range(2)]
            if not alive:
                bodies[victim].update(health=0, feet=[500, 0, -100])
            watched = 1-victim if local == victim and not alive else local
            subject = bodies[watched]['feet']
            history[role].append(dict(tick=tick, epoch=1, owner=None, checkpoint_generation=0,
                players=bodies, camera_player=('Blue', 'Red')[watched], camera_target=subject,
                camera_subject_feet=subject, fulcrum=[angle, 0],
                driver=dict(command=7 if local == victim else 8, phase='idle' if local == victim else 'reached',
                    clear=[True, True], surface=dict(entity=460, supported=[True, True]))))
    return event, history, 7


class FulcrumGuards(unittest.TestCase):
    def test_either_reconnect_and_passive_platform_revival(self):
        for victim in (0, 1):
            self.assertEqual(verify_fulcrum_rejoin(*reconnect(victim), victim, 2)['remaining_after'], 980)
            proof = verify_fulcrum_recovery(*recovery(victim))
            self.assertEqual(proof['carry_ticks'], 80)
            self.assertEqual(proof['carried_distance'], 20)

    def test_rejoin_cannot_advance_held_world_or_move_dead_body(self):
        for fault in ('world', 'dead_body', 'lost_support', 'old_process'):
            before, held, after = reconnect(1)
            if fault == 'world': held['combat']['tilt'] += 1
            elif fault == 'dead_body': after['frame']['players'][1]['player']['feet'][0] += 1
            elif fault == 'lost_support': after['frame']['players'][0]['player']['grounded'] = False
            else: after['pid'] = 2
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_fulcrum_rejoin(before, held, after, 1, 2)

    def test_stationary_platform_and_missing_spawn_boundary_fail(self):
        for fault in ('stationary', 'limit', 'missing_before', 'wrong_platform', 'lost_spawn_support'):
            event, history, command = recovery(1)
            if fault == 'stationary': history['host'][1]['fulcrum'][0] = -5
            elif fault == 'limit': history['host'][0]['fulcrum'][0] = -45
            elif fault == 'missing_before': history['host'].pop(0)
            elif fault == 'wrong_platform': history['host'][1]['driver']['surface']['entity'] = 123
            else: history['host'][1]['driver']['surface']['supported'][1] = False
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_fulcrum_recovery(event, history, command)

    def test_carry_requires_neutral_clear_grounded_motion_on_same_brush(self):
        for fault in ('short', 'driven', 'velocity', 'obstructed', 'airborne', 'unsupported', 'stationary_body'):
            event, history, command = recovery(0)
            row = history['host'][3]
            if fault == 'short': history['host'] = history['host'][:3]
            elif fault == 'driven': row['driver']['phase'] = 'walking'
            elif fault == 'velocity': row['players'][0]['velocity'][0] = 1
            elif fault == 'obstructed': row['driver']['clear'][0] = False
            elif fault == 'airborne': row['players'][0]['grounded'] = False
            elif fault == 'unsupported': history['guest'][3]['driver']['surface']['supported'][0] = False
            else: history['host'][4]['players'][0]['feet'] = history['host'][2]['players'][0]['feet']
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_fulcrum_recovery(event, history, command)


if __name__ == '__main__':
    unittest.main()
