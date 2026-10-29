"""Reject incomplete combat/scene/next-map evidence."""
import copy
import unittest

from test_coop_funhouse_boss import SCENE, verify_boss_exit


def fixture():
    bodies = [dict(feet=[i*48, 0, 0], health=80-i*20, grounded=True, spawned_at=0,
                   warped_at=0, stats=dict(owned=[1, i]), collected=[str(i)]) for i in range(2)]
    rows = []
    sequence = [(100, None, None), (200, None, None)]
    sequence += [(300+i*120, i, None) for i in range(8)]
    sequence += [(1200, 7, 0), (1512, 7, 2.6), (1950, 7, 6.25), (1980, None, None)]
    for tick, line, tail in sequence:
        arrival, active = tick == 1980, line is not None
        scene = dict(clock=dict(line=line, time=10+(tail or 0)), ending=10 if tail is not None else None)
        rows.append(dict(tick=tick, epoch=2 if arrival else 1, map='hatter1' if arrival else 'funhouse',
            checkpoint_generation=0, scene=SCENE if active else None, owner='Blue' if active else None,
            players=copy.deepcopy(bodies), presentation=dict(visible_players=[True, not active]),
            driver=dict(clear=[True, True]), funhouse=dict(fight=True, bosses=[dict(health=100 if tick == 100 else 0), dict(health=0)],
            scene=scene if active else None, floor=1000 if tail is not None and tail >= 2.5 else None)))
    history = {}
    for slot, peer in enumerate(('host', 'guest')):
        history[peer] = copy.deepcopy(rows)
        for row in history[peer]:
            row.update(camera_player=('Blue', 'Red')[slot], camera_subject_feet=bodies[slot]['feet'], camera_target=bodies[slot]['feet'])
    return history


class BossExitGuards(unittest.TestCase):
    def test_complete_watched_exit(self):
        result = verify_boss_exit(fixture())
        self.assertEqual([r['tail_ticks'] for r in result], [780, 780])

    def test_empty_cast_before_arena_is_not_a_boss_defeat(self):
        history = fixture()
        for peer, rows in history.items():
            earlier = copy.deepcopy(rows[0])
            earlier['tick'] = 1
            earlier['funhouse'].update(fight=False, bosses=[])
            rows.insert(0, earlier)
        self.assertEqual([r['defeated_tick'] for r in verify_boss_exit(history)], [200, 200])
        for rows in history.values():
            for row in rows:
                row['funhouse']['bosses'] = []
        with self.assertRaises(AssertionError):
            verify_boss_exit(history)

    def test_skipping_still_requires_physical_tail_and_next_map(self):
        rows = fixture()
        for peer in rows:
            rows[peer] = [r for r in rows[peer] if r['scene'] is None or r['funhouse']['scene']['clock']['line'] == 7]
        verify_boss_exit(rows, skipped=True)
        rows['guest'] = [r for r in rows['guest'] if r['tick'] != 1950]
        with self.assertRaises(AssertionError):
            verify_boss_exit(rows, skipped=True)

    def test_missing_or_unsafe_evidence_fails(self):
        for mistake in ('no_fight', 'living_boss', 'line', 'owner', 'visible', 'moved', 'health',
                        'inventory', 'early_floor', 'no_floor', 'short_tail', 'no_exit', 'wrong_map',
                        'blocked', 'unsupported', 'overlap', 'far', 'hidden', 'lost_item'):
            history = fixture()
            rows = history['guest']
            active = [r for r in rows if r['scene'] == SCENE]
            after = rows[-1]
            if mistake == 'no_fight': rows[0]['funhouse']['bosses'][0]['health'] = 0
            elif mistake == 'living_boss': active[0]['funhouse']['bosses'][0]['health'] = 1
            elif mistake == 'line': active[1]['funhouse']['scene']['clock']['line'] = 0
            elif mistake == 'owner':
                for r in active: r['owner'] = 'Red'; r['presentation']['visible_players'] = [False, False]
            elif mistake == 'visible': active[1]['presentation']['visible_players'][1] = True
            elif mistake == 'moved': active[1]['players'][1]['feet'][0] += 1
            elif mistake == 'health': active[1]['players'][1]['health'] -= 1
            elif mistake == 'inventory': active[1]['players'][1]['collected'] = []
            elif mistake == 'early_floor': rows[-4]['funhouse']['floor'] = 1
            elif mistake == 'no_floor': rows[-2]['funhouse']['floor'] = None
            elif mistake == 'short_tail': after['tick'] = 1978
            elif mistake == 'no_exit': rows.pop()
            elif mistake == 'wrong_map': after['map'] = 'funhouse'
            elif mistake == 'blocked': after['driver']['clear'][1] = False
            elif mistake == 'unsupported': after['players'][1]['grounded'] = False
            elif mistake == 'overlap': after['players'][1]['feet'] = [0, 0, 0]
            elif mistake == 'far': after['players'][1]['feet'] = [1000, 0, 0]
            elif mistake == 'hidden': after['presentation']['visible_players'][0] = False
            elif mistake == 'lost_item': after['players'][1]['stats']['owned'] = []
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_boss_exit(history)


if __name__ == '__main__':
    unittest.main()
