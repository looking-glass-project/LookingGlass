"""Reject air/body resets and unsupported claims of first-frame agreement."""
import copy
import unittest
from unittest.mock import patch

from coop_swim_rejoin import verify_swim_rejoin, verify_first_frame
from test_coop_countdown_guards import reports
from test_coop_utemple import TemplePair
from test_coop_utemple_recovery import TempleRecovery, wipe_air_ready


def swimming(victim=1):
    before, after = reports(victim)
    before['playing'], after['playing'] = False, True
    for report in (before, after):
        report['frame']['epoch'] = 1
        report['combat'] = {'guide_clock': 20.}
        for index, body in enumerate(report['frame']['players']):
            body['stats']['turtle_air'] = True
            body['player'] = dict(feet=[index*48, 0, -100], swimming=True,
                immersion={'level': 3}, breath=dict(shell=True, submerged=24 if index == victim else 5,
                                                    hits=9 if index == victim else 0, next_hurt=24.5))
    after['frame']['players'][1-victim]['player']['breath']['submerged'] += 20/120
    return before, copy.deepcopy(before), after


class SwimRejoinGuards(unittest.TestCase):
    def test_rejoin_after_known_wipe_still_rejects_an_unexpected_generation(self):
        before, held, after = swimming()
        for report in (before, held, after):
            report['frame']['checkpoint_generation'] = 1
        verify_swim_rejoin(before, held, after, 1, 2, expected_generation=1)
        with self.assertRaisesRegex(AssertionError, 'silently wiped'):
            verify_swim_rejoin(before, held, after, 1, 2)
        after['frame']['checkpoint_generation'] = 2
        with self.assertRaisesRegex(AssertionError, 'silently wiped'):
            verify_swim_rejoin(before, held, after, 1, 2, expected_generation=1)

    def test_wipe_waits_for_both_air_supplies_instead_of_alternating_revivals(self):
        _, _, report = swimming()
        for body in report['frame']['players']:
            body['stats']['sanity'] = 25
            body['player']['breath'].update(submerged=.1, hits=0, next_hurt=0)
        self.assertTrue(wipe_air_ready(report))
        for index in (0, 1):
            stale = copy.deepcopy(report)
            # Captured failure: one fresh revival and a partner with over
            # fourteen seconds already spent underwater never died together.
            stale['frame']['players'][index]['player']['breath']['submerged'] = 14.15
            self.assertFalse(wipe_air_ready(stale))

    def test_wipe_preparation_rejects_dead_dry_or_stale_clients(self):
        for fault in ('dead', 'dry', 'surface', 'shell', 'damage', 'waiting'):
            with self.subTest(fault=fault):
                _, _, report = swimming()
                for body in report['frame']['players']:
                    body['stats']['sanity'] = 25
                    body['player']['breath'].update(submerged=.1, hits=0)
                body = report['frame']['players'][0]
                if fault == 'dead': body['stats']['sanity'] = 0
                elif fault == 'dry': body['player']['swimming'] = False
                elif fault == 'surface': body['player']['immersion']['level'] = 2
                elif fault == 'shell': body['player']['breath']['shell'] = False
                elif fault == 'damage': body['player']['breath']['hits'] = 1
                elif fault == 'waiting': report['playing'] = False
                self.assertFalse(wipe_air_ready(report))

    def test_repeated_tick_keeps_matching_camera_interpolation_inputs(self):
        pair = TempleRecovery.__new__(TempleRecovery)
        first = dict(player_feet=[[0, 0, 0], [64, 0, 0]])
        pair.reports = {'guest': dict(presentation=first)}
        pair.history = {'guest': [dict(tick=50, camera_subject_feet=[64, 0, 0])]}
        with patch.object(TemplePair, 'pump'):
            pair.pump()
            pair.reports['guest']['presentation'] = dict(player_feet=[[0, 0, 0], [72, 0, 0]])
            pair.pump()
        self.assertEqual(pair.history['guest'][0]['presentation'], first)
        self.assertEqual(pair.history['guest'][0]['camera_subject_feet'], [64, 0, 0])

    def test_both_drowned_roles_keep_body_air_and_countdown(self):
        for victim in (0, 1):
            proof = verify_swim_rejoin(*swimming(victim), victim, 2)
            self.assertEqual(proof['remaining_before']-proof['remaining_after'], 20)

    def test_waiting_host_cannot_advance_bodies_or_world(self):
        for field in ('frame', 'combat'):
            before, held, after = swimming()
            if field == 'frame': held['frame']['tick'] += 1
            else: held['combat']['guide_clock'] += 1
            with self.subTest(field=field), self.assertRaisesRegex(AssertionError, 'Disconnected host advanced'):
                verify_swim_rejoin(before, held, after, 1, 2)

    def test_drowned_body_cannot_refill_move_or_reset_drowning_history(self):
        for fault in ('air', 'hits', 'feet'):
            before, held, after = swimming()
            player = after['frame']['players'][1]['player']
            if fault == 'air': player['breath']['submerged'] = 0
            elif fault == 'hits': player['breath']['hits'] = 0
            else: player['feet'][0] += 1
            with self.subTest(fault=fault), self.assertRaisesRegex(AssertionError, 'drowned body'):
                verify_swim_rejoin(before, held, after, 1, 2)

    def test_swimming_air_capacity_and_fresh_observation_are_required(self):
        for fault in ('swimming', 'level', 'equipment', 'shell', 'late'):
            before, held, after = swimming()
            body = after['frame']['players'][0]
            if fault == 'swimming': body['player']['swimming'] = False
            elif fault == 'level': body['player']['immersion']['level'] = 2
            elif fault == 'equipment': body['stats']['turtle_air'] = False
            elif fault == 'shell': body['player']['breath']['shell'] = False
            else:
                after['frame']['tick'] = 321
                after['frame']['lives']['states'][1]['Spectating']['ticks_left'] = 879
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_swim_rejoin(before, held, after, 1, 2)

    def test_first_frame_must_have_matching_body_world_and_enemy_evidence(self):
        _, host, guest = swimming()
        sample = dict(epoch=1, tick=220, sha256='a'*64, world_sha256='b'*64, native_sha256='c'*64)
        for report in (host, guest):
            report['route_driver'] = {'network_states': [copy.deepcopy(sample)]}
        self.assertEqual(verify_first_frame(host, guest)['tick'], 220)
        for field in ('sha256', 'world_sha256', 'native_sha256', 'tick'):
            damaged = copy.deepcopy(guest)
            damaged['route_driver']['network_states'][0][field] = 221 if field == 'tick' else 'd'*64
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_first_frame(host, damaged)


if __name__ == '__main__':
    unittest.main()
