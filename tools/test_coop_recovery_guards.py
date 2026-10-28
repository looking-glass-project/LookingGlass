"""Reject incomplete, early, unsafe or incorrectly rendered recovery evidence."""
import copy
import unittest
import tempfile
from pathlib import Path

from coop_recovery import verify_trace, verify_views, verify_northern_coverage
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
