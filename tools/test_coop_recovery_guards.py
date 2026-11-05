"""Reject incomplete, early, unsafe or incorrectly rendered recovery evidence."""
import copy
import unittest
import tempfile
from pathlib import Path

from coop_recovery import verify_trace, verify_views, verify_northern_coverage,verify_presence
from test_coop_checkmate import CheckmatePair


def body(health=50, life='Alive', spawn=0, x=0, full=False):
    return dict(health=health, life=life, spawned_at=spawn, warped_at=0,
                feet=[x, 0, 0], god=False, notarget=False,
                stats=dict(sanity=health, owned=[1, 1], collected=['mine'], will=7) if full else None,
                geometry=dict(clear=True, safe=True, valid=True, supported=True, swimming=False, rope=False) if full else None)


def trace(paused=0, waiting=0):
    rows = []
    def append(blue, cinematic=False):
        tick = len(rows)
        rows.append(dict(version=1, serial=tick+1, build='test', epoch=1, tick=tick,
                         map='rchess1', checkpoint_generation=0, cinematic=cinematic, spawn_held=False,
                         players=[blue, body(x=48)]))
    append(body(full=True))
    append(body(0, {'Spectating': dict(ticks_left=1200)}, full=True))
    for left in range(1199, 0, -1):
        append(body(0, {'Spectating': dict(ticks_left=left)}))
        if left == 700:
            for _ in range(paused):
                append(body(0, {'Spectating': dict(ticks_left=left)}), True)
    for _ in range(waiting):
        append(body(0, 'AwaitingSafeSpawn'))
    append(body(25, spawn=len(rows), full=True))
    return rows


class RecoveryGuards(unittest.TestCase):
    def portal_rows(self):
        rows=[]
        for tick,x,warp in ((10,0,0),(20,1000,15)):
            rows.append(dict(epoch=1,tick=tick,checkpoint_generation=0,owner=None,scene=None,
                camera_player='Blue',camera_target=[x,0,48],camera_subject_feet=[x,0,0],
                players=[dict(health=50,feet=[x,0,0],warped_at=warp),dict(health=50,feet=[48,0,0],warped_at=0)]))
        return rows

    def test_portal_presence_needs_the_exact_independently_verified_boundary(self):
        verify_presence(self.portal_rows(),'host',portal_warps={(1,0,15)})
        for allowed in ((),{(1,1,15)},{(1,0,14)},{(2,0,15)}):
            with self.subTest(allowed=allowed),self.assertRaisesRegex(AssertionError,'Unscripted route warp'):
                verify_presence(self.portal_rows(),'host',portal_warps=allowed)

    def test_portal_permission_does_not_allow_a_backdated_stamp_or_overlap(self):
        rows=self.portal_rows();rows[1]['players'][0]['warped_at']=5
        with self.assertRaisesRegex(AssertionError,'Unscripted route warp'):
            verify_presence(rows,'host',portal_warps={(1,0,5)})
        rows=self.portal_rows();rows[1]['players'][1]['feet']=[1000,0,0]
        with self.assertRaisesRegex(AssertionError,'overlap'):
            verify_presence(rows,'host',portal_warps={(1,0,15)})

    def test_exact_revive_with_cinematic_pause_and_safe_spawn_wait(self):
        result = verify_trace(trace(13, 9), 'test', True)
        self.assertEqual(result['revivals'], [dict(player=0, epoch=1, death_tick=1,
            spawn_tick=1223, health=25, active_ticks=1200, paused_ticks=13, wait_ticks=9)])

    def test_missing_ticks_and_counter_jumps_fail(self):
        for mode in ('missing', 'counter', 'pause'):
            rows = trace()
            if mode == 'missing':
                del rows[800]
            elif mode == 'counter':
                rows[800]['players'][0]['life']['Spectating']['ticks_left'] += 1
            else:
                rows[800]['cinematic'] = True
            with self.subTest(mode=mode), self.assertRaises(AssertionError):
                verify_trace(rows, 'test', True)

    def test_early_full_health_unsafe_and_resource_changing_revives_fail(self):
        for mode in ('health', 'tick', 'blocked', 'hazard', 'inventory', 'overlap', 'distant', 'sequence'):
            rows = trace(); end = rows[-1]; blue = end['players'][0]
            if mode == 'health': blue['health'] = 100
            elif mode == 'tick': blue['spawned_at'] -= 1
            elif mode == 'blocked': blue['geometry']['clear'] = False
            elif mode == 'hazard': blue['geometry']['safe'] = False
            elif mode == 'inventory': blue['stats']['owned'] = [1, 2]
            elif mode == 'overlap': blue['feet'] = [48, 0, 0]
            elif mode == 'distant': blue['feet'] = [-1000, 0, 0]
            else: end['spawn_held'] = True
            with self.subTest(mode=mode), self.assertRaises(AssertionError):
                verify_trace(rows, 'test', True)

    def test_world_drop_clocks_can_advance_without_changing_inventory(self):
        rows = trace()
        rows[1]['players'][0]['stats']['loot'] = {'visit': {'drops': [{'remaining': 12}]}}
        rows[-1]['players'][0]['stats']['loot'] = {'visit': {'drops': [{'remaining': 2}]}}
        verify_trace(rows, 'test', True)
        for field, value in [('will', 99), ('collected', ['theirs']), ('owned', [9, 9])]:
            broken = copy.deepcopy(rows)
            broken[-1]['players'][0]['stats'][field] = value
            with self.subTest(field=field), self.assertRaisesRegex(AssertionError, 'resources'):
                verify_trace(broken, 'test', True)

    def test_dead_northern_input_is_cleared_once_and_other_modes_are_unchanged(self):
        pair = CheckmatePair.__new__(CheckmatePair)
        pair.allow_revival, pair.step = True, 39
        pair.commands = {'guest': dict(goal=[1, 2, 3], navigate=True, attack=True)}
        pair.reports = {'guest': dict(frame=dict(players=[{}, dict(stats=dict(sanity=0))]))}
        calls = []
        def neutral(role, **kwargs):
            calls.append((role, kwargs))
            pair.commands[role] = dict(goal=None, navigate=False, rope=None, **kwargs)
        pair.set = neutral
        pair.clear_dead_northern_commands()
        pair.clear_dead_northern_commands()
        self.assertEqual(calls, [('guest', dict(attack=False))])
        for enabled, step, health in [(False, 39, 0), (True, 32, 0), (True, 39, 25)]:
            pair.allow_revival, pair.step = enabled, step
            pair.reports['guest']['frame']['players'][1]['stats']['sanity'] = health
            pair.commands['guest']['goal'] = [1, 2, 3]
            pair.clear_dead_northern_commands()
            self.assertEqual(len(calls), 1)

    def test_wipe_missing_death_and_transition_shortcuts_fail(self):
        for mode in ('wipe', 'missing_death', 'transition', 'truncated', 'build'):
            rows = trace()
            if mode == 'wipe': rows[-1]['checkpoint_generation'] = 1
            elif mode == 'missing_death': rows[1]['players'][0]['life']['Spectating']['ticks_left'] = 1199
            elif mode == 'transition': rows[-1]['epoch'] = 2
            elif mode == 'truncated': rows.pop()
            else: rows[-1]['build'] = 'other'
            with self.subTest(mode=mode), self.assertRaises(AssertionError):
                verify_trace(rows, 'test', True)

    def test_no_death_can_pass_a_route_but_cannot_claim_recovery(self):
        rows = trace()[:1]
        self.assertEqual(verify_trace(rows, 'test')['revivals'], [])
        with self.assertRaisesRegex(AssertionError, 'did not exercise'):
            verify_trace(rows, 'test', True)

    def test_only_actually_traversed_team_waypoints_can_be_carried_by_revival(self):
        events = dict(host=[dict(step=i) for i in range(33, 45)], guest=[dict(step=i) for i in range(33, 39)])
        skips = [dict(role='guest', steps=list(range(39, 45)))]
        verify_northern_coverage(events, skips)
        broken = copy.deepcopy(events); broken['host'].pop(6)
        with self.assertRaises(AssertionError): verify_northern_coverage(broken, skips)
        broken = copy.deepcopy(events); broken['host'].reverse()
        with self.assertRaises(AssertionError): verify_northern_coverage(broken, skips)

    def test_rendered_camera_must_follow_then_return(self):
        def row(tick, health, watched):
            return dict(epoch=1, tick=tick, checkpoint_generation=0, owner=None, scene=None,
                camera_player=watched, camera_target=[48 if health == 0 else 0, 0, 48],
                camera_subject_feet=[48 if health == 0 else 0, 0, 0],
                players=[dict(health=health, feet=[0, 0, 0]), dict(health=50, feet=[48, 0, 0])])
        events = [dict(player=0, epoch=1, death_tick=1, spawn_tick=1201)]
        history = dict(host=[row(2, 0, 'Red'), row(1202, 20, 'Blue')])
        self.assertEqual(verify_views(events, history), 1)
        history['host'][0]['camera_player'] = 'Blue'
        with self.assertRaisesRegex(AssertionError, 'camera'): verify_views(events, history)
        with self.assertRaisesRegex(AssertionError, 'Missing rendered'):
            verify_views(events, dict(host=[row(2, 0, 'Red')]))

    def test_camera_uses_actual_interpolated_body_and_cuts_at_revival(self):
        row = dict(epoch=1, tick=50, checkpoint_generation=0, owner=None,
            camera_player='Red', camera_target=[64, 0, 48], camera_subject_feet=[64, 0, 0],
            players=[dict(health=50, feet=[0, 0, 0], spawned_at=0, warped_at=0),
                     dict(health=25, feet=[100, 0, 0], spawned_at=0, warped_at=0)],
            presentation=dict(player_feet=[[0, 0, 0], [64, 0, 0]], elapsed=.005,
                previous_players=[[[0, 0, 0], 0, 0, True], [[60, 0, 0], 0, 0, True]]))
        from coop_recovery import verify_presence
        verify_presence([row], 'guest')
        broken = copy.deepcopy(row); broken['presentation']['player_feet'][1][0] += 1
        with self.assertRaisesRegex(AssertionError, 'interpolation'): verify_presence([broken], 'guest')
        broken = copy.deepcopy(row); broken['camera_subject_feet'][0] += .5
        with self.assertRaisesRegex(AssertionError, 'rendered player'): verify_presence([broken], 'guest')
        row['players'][1]['spawned_at'] = 50
        with self.assertRaisesRegex(AssertionError, 'interpolation'): verify_presence([row], 'guest')
        row['camera_target'][0] = row['camera_subject_feet'][0] = 100
        row['presentation']['player_feet'][1][0] = 100
        verify_presence([row], 'guest')

    def test_duplicate_network_tick_keeps_one_coherent_render_sample(self):
        from unittest.mock import patch
        from test_coop_village import Pair
        with tempfile.TemporaryDirectory() as folder:
            pair = CheckmatePair.__new__(CheckmatePair)
            pair.folder, pair.allow_revival = Path(folder), False
            pair.swim_requested = {}
            first = dict(player_feet=[[0, 0, 0], [64, 0, 0]])
            pair.reports = {'guest': dict(presentation=first)}
            pair.history = {'guest': [dict(tick=50, camera_subject_feet=[64, 0, 0])]}
            pair.state = lambda role: dict(phase='Fight')
            with patch.object(Pair, 'pump'):
                pair.pump()
                pair.reports['guest']['presentation'] = dict(player_feet=[[0, 0, 0], [72, 0, 0]])
                pair.pump()
            self.assertEqual(pair.history['guest'][0]['presentation'], first)
            self.assertEqual(pair.history['guest'][0]['camera_subject_feet'], [64, 0, 0])

    def test_follower_keeps_delayed_leaders_backtrack_lane_clear(self):
        pair = CheckmatePair.__new__(CheckmatePair)
        pair.first, pair.partner, pair.step = 'guest', 'host', 23
        pair.commands = {}
        pair.reports = {r: dict(route_driver={}) for r in ('host', 'guest')}
        bodies = {'guest': dict(player=dict(feet=[300, 2432, 256])),
                  'host': dict(player=dict(feet=[200, 2432, 256]))}
        pair.body = bodies.__getitem__
        pair.scene_end = lambda: None
        pair.arrived = pair.busy = lambda: False
        pair.reached = lambda role: False
        def command(role, goal=None, navigate=False, **kwargs):
            pair.commands[role] = dict(goal=goal, navigate=navigate, **kwargs)
        pair.set = command
        def advance(predicate):
            self.assertFalse(predicate())
            self.assertIsNone(pair.commands['host']['goal'])
            bodies['guest']['player']['feet'] = [274, 2432, 256]
            self.assertFalse(predicate())
            self.assertIsNone(pair.commands['host']['goal'])
            bodies['guest']['player']['feet'] = [370, 2432, 256]
            self.assertFalse(predicate())
            self.assertEqual(pair.commands['host']['goal'], [300, 2432, 256])
            pair.reached = lambda role: True
            self.assertTrue(predicate())
        pair.until = advance
        pair.go([576, 2496, 256], navigate=True)

    def test_recovery_mode_preserves_strict_mode_and_living_body_checks(self):
        with tempfile.TemporaryDirectory() as folder:
            pair = CheckmatePair.__new__(CheckmatePair)
            pair.folder, pair.last_valid, pair.commands, pair.step = Path(folder), {}, {}, 33
            report = dict(visit=dict(map='rchess1'), combat=dict(stage=dict(owner=None)),
                route_driver=dict(scene=None, clear=[False, True], recovery_trace_error=None),
                frame=dict(checkpoint_generation=0, players=[
                    dict(stats=dict(sanity=0, god=False, notarget=False), spawned_at=0),
                    dict(stats=dict(sanity=50, god=False, notarget=False), spawned_at=0)]))
            pair.allow_revival = False
            with self.assertRaisesRegex(AssertionError, 'died'): pair.validate_report('host', report)
            pair.allow_revival = True
            pair.validate_report('host', report)
            pair.validate_clear('host', report)
            report['route_driver']['clear'][1] = False
            with self.assertRaisesRegex(AssertionError, 'living body'): pair.validate_clear('host', report)
            for field in ('wipe', 'cheat', 'trace'):
                broken = copy.deepcopy(report)
                if field == 'wipe': broken['frame']['checkpoint_generation'] = 1
                elif field == 'cheat': broken['frame']['players'][1]['stats']['god'] = True
                else: broken['route_driver']['recovery_trace_error'] = 'disk full'
                with self.subTest(field=field), self.assertRaises(AssertionError):
                    pair.validate_report('host', broken)


if __name__ == '__main__':
    unittest.main()
