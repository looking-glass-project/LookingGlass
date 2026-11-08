"""Guard preserved-save comparisons and deterministic client shutdown ordering."""
import copy
from pathlib import Path
from types import SimpleNamespace
import unittest

from test_coop_import import Pair, imported_before_play, same_saved_state


class SavedImportGuards(unittest.TestCase):
    def fixture(self):
        game = dict(current='gvillage$first', player=dict(feet=[0., 0., 0.]),
            stats=dict(sanity=83., will=61., collected=['pickup-a']),
            view=dict(yaw=1.2, pitch=.3), character=dict(clock=.2),
            campaign=dict(completed=[], levels={'gvillage$first': dict(map='gvillage', entry=None,
                interactions=dict(used=['door-a']), story=dict(seen=['intro']),
                hints=dict(seen=[]), npcs=dict(dead=['guard-a']))}))
        body = dict(player=game['player'], stats=game['stats'], yaw=1.2, pitch=.3)
        report = dict(playing=False, completed_visits=0, visit=dict(map='gvillage', entry=None),
            frame=dict(tick=0, players=[copy.deepcopy(body), copy.deepcopy(body)]),
            combat=dict(stage=dict(world=dict(used=['door-a']), story=dict(seen=['intro']),
                hints=dict(seen=[])), npcs=dict(dead=['guard-a']), characters=[dict(clock=.2), {}]))
        report['frame']['players'][1]['player']['feet'] = [48., 0., 0.]
        return game, report

    def test_native_float_spelling_is_allowed_without_rounding_gameplay_changes(self):
        same_saved_state(dict(yaw=1.2000000476837158), dict(yaw=1.2), 'camera')
        for actual in (1.2001, float('nan'), float('inf'), True, None):
            with self.subTest(actual=actual), self.assertRaises(AssertionError):
                same_saved_state(1.2, actual, 'camera')
        with self.assertRaises(AssertionError):
            same_saved_state(16777217, 16777216, 'integer event counter')

    def test_existing_save_preserves_bodies_world_and_completed_visits(self):
        game, report = self.fixture()
        before = copy.deepcopy(report)
        expected = imported_before_play(game, report)
        self.assertEqual(report, before)
        self.assertEqual(expected['campaign']['completed'], [])
        self.assertEqual(expected['frame']['combat'], report['combat'])

    def test_partial_reset_or_stale_report_cannot_be_credited_as_import(self):
        for field in ('position', 'health', 'camera', 'NPC', 'world', 'story', 'playing', 'tick',
                      'completed', 'overlap', 'visit'):
            with self.subTest(field=field):
                game, report = self.fixture()
                blue, red = report['frame']['players']
                if field == 'position': blue['player']['feet'][0] = 100.
                elif field == 'health': red['stats']['sanity'] = 100.
                elif field == 'camera': blue['pitch'] = 0.
                elif field == 'NPC': report['combat']['npcs']['dead'] = []
                elif field == 'world': report['combat']['stage']['world']['used'] = []
                elif field == 'story': report['combat']['stage']['story']['seen'] = []
                elif field == 'playing': report['playing'] = True
                elif field == 'tick': report['frame']['tick'] = 12
                elif field == 'completed': report['completed_visits'] = 1
                elif field == 'overlap': red['player']['feet'] = [0., 0., 0.]
                elif field == 'visit': report['visit']['entry'] = 'another-visit'
                with self.assertRaises(AssertionError):
                    imported_before_play(game, report)

    def test_missing_fields_and_boolean_integer_aliases_do_not_pass(self):
        for expected, actual in (({'owned': [True]}, {'owned': [1]}), ({'jumps': 2}, {}),
                                 ([1, 2], [1]), ({'will': 5.}, {'will': 5., 'unexpected': 1})):
            with self.subTest(actual=actual), self.assertRaises(AssertionError):
                same_saved_state(expected, actual, 'save')

    def test_host_finish_is_not_sent_until_guest_exits(self):
        pair = Pair(SimpleNamespace(), Path('unused-profile'), 'shutdown')
        codes = dict(host=None, guest=None)
        pair.processes = {role: SimpleNamespace(poll=lambda r=role: codes[r]) for role in codes}
        waits = []
        def until(condition, timeout, allow_exit):
            self.assertTrue(allow_exit)
            if not waits:
                self.assertTrue(pair.commands['guest']['finish'])
                self.assertNotIn('host', pair.commands)
                codes['guest'] = 0
                waits.append('guest')
            else:
                self.assertEqual(codes['guest'], 0)
                self.assertTrue(pair.commands['host']['finish'])
                codes['host'] = 0
                waits.append('host')
            self.assertTrue(condition())
        pair.until = until
        pair.finish()
        self.assertEqual(waits, ['guest', 'host'])


if __name__ == '__main__':
    unittest.main()
