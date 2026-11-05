"""Reject resets and stale reports in the native cutscene reconnect verifier."""
import copy
import unittest

from test_coop_scene import verify, verify_scene_rejoin


def reports(first='blue', time=6):
    body = dict(stats=dict(sanity=100, will=60, owned=[1, 1], collected=['personal']),
        player=dict(feet=[0, 0, 0], velocity=[0, 0, 0]), spawned_at=0, warped_at=0)
    stage = dict(owner=first.title(), contacts=dict(waiting=['fixture']),
        world=dict(school=dict(first_cinema=dict(beat='Theatre', time=time, home=[0, 0, 0]))),
        story=dict(current=[dict(id='fixture'), 1, 5] if time == 30 else None,
            seen=['fixture'], queue=[] if time == 30 else [dict(id='fixture')], completed=0, finished=[], exit=None))
    before = dict(pid=1, build='fixture', visit=dict(map='skool1'),
        frame=dict(tick=1000, checkpoint_generation=0, players=[copy.deepcopy(body), copy.deepcopy(body)]),
        combat=dict(stage=stage), presentation=dict(visible_players=[False, False]))
    after = copy.deepcopy(before)
    after['pid'] = 3
    after['frame']['tick'] += 12
    after['combat']['stage']['world']['school']['first_cinema']['time'] += .1
    if time == 30:
        after['combat']['stage']['story']['current'][2] += .1
    return before, after


def history():
    rows = []
    for i in range(21):
        rows.append(dict(tick=i*24, checkpoint_generation=0, owner='Blue',
            camera=[i*10, 0, 0], scene_time=i*.2, visible_players=[False, False],
            players=[dict(feet=[x, 0, 0], health=100, spawned_at=0, warped_at=0) for x in (0, 48)]))
    reunited = copy.deepcopy(rows[-1])
    reunited.update(tick=500, owner=None)
    for body in reunited['players']:
        body['warped_at'] = 500
    rows.append(reunited)
    return rows


class SceneGuards(unittest.TestCase):
    def test_both_owners_rejoin_during_introduction_and_dialogue(self):
        for owner in ('blue', 'red'):
            for time in (6, 30):
                before, after = reports(owner, time)
                result = verify_scene_rejoin(before, after, owner, 2)
                self.assertEqual(result['after_tick']-result['before_tick'], 12)

    def test_stale_process_and_scene_or_dialogue_resets_fail(self):
        for fault in ('pid', 'clock', 'wipe', 'owner', 'scene', 'cue', 'story', 'line', 'dialogue', 'contacts', 'visible'):
            before, after = reports(time=30)
            stage = after['combat']['stage']
            if fault == 'pid': after['pid'] = 2
            elif fault == 'clock': after['frame']['tick'] = 999
            elif fault == 'wipe': after['frame']['checkpoint_generation'] = 1
            elif fault == 'owner': stage['owner'] = 'Red'
            elif fault == 'scene': stage['world']['school']['first_cinema']['time'] = 0
            elif fault == 'cue': stage['world']['school']['first_cinema']['skipped'] = True
            elif fault == 'story': stage['story']['completed'] = 1
            elif fault == 'line': stage['story']['current'][1] = 0
            elif fault == 'dialogue': stage['story']['current'][2] = 0
            elif fault == 'contacts': stage['contacts']['waiting'] = []
            else: after['presentation']['visible_players'][1] = True
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_scene_rejoin(before, after, 'blue', 2)

    def test_personal_resources_body_identity_and_waiting_pose_survive(self):
        for player in (0, 1):
            for fault in ('health', 'inventory', 'spawn', 'warp'):
                before, after = reports()
                body = after['frame']['players'][player]
                if fault == 'health': body['stats']['sanity'] = 25
                elif fault == 'inventory': body['stats']['collected'] = []
                elif fault == 'spawn': body['spawned_at'] = 1000
                else: body['warped_at'] = 1000
                with self.subTest(player=player, fault=fault), self.assertRaises(AssertionError):
                    verify_scene_rejoin(before, after, 'blue', 2)
        before, after = reports()
        after['frame']['players'][1]['player']['feet'][0] = 5
        with self.assertRaisesRegex(AssertionError, 'waiting partner'):
            verify_scene_rejoin(before, after, 'blue', 2)
        before, after = reports()
        after['combat']['stage']['story']['current'] = [dict(id='fixture'), 0, .1]
        with self.assertRaisesRegex(AssertionError, 'dialogue'):
            verify_scene_rejoin(before, after, 'blue', 2)

    def test_full_history_rejects_a_restarted_clock_or_visible_partner(self):
        self.assertEqual(verify(history(), 'blue')['reunion_tick'], 500)
        for field, value in [('scene_time', 0), ('visible_players', [False, True])]:
            rows = history()
            rows[10][field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify(rows, 'blue')


if __name__ == '__main__':
    unittest.main()
