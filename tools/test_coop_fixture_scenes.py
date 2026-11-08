"""Headless regression for a scene starting between client arrival reports."""
import copy
import unittest
from types import SimpleNamespace

from check_coop_rewards import personal_rewards
from test_coop_school import SchoolPair
from test_coop_school2 import verify_reunion
from test_coop_school_return import SchoolReturnPair
from test_coop_village import verify_rejoin_stats


class SceneRace(SchoolPair):
    def __init__(self, stuck=False, pickup=True):
        self.args = SimpleNamespace(first='red')
        self.commands = {'host': None, 'guest': None}
        self.evidence = {}
        self.phase = 0
        self.started = False
        self.stuck = stuck
        self.pickup = pickup
        self.asides = []
        self.reports = {}
        for role in self.commands:
            self.reports[role] = {'combat': {'stage': {'owner': None}},
                'frame': {'tick': 0, 'players': [
                    {'warped_at': 0, 'player': {'feet': [0, 0, 0], 'grounded': True}},
                    {'warped_at': 0, 'player': {'feet': [50, 0, 0], 'grounded': True}}]}}

    def set(self, role, goal=None, *args, **kwargs):
        previous = self.commands[role]
        self.commands[role] = goal
        if role == self.first and previous is not None and goal is None and not self.started:
            self.started = True
            # Host starts the scene after the guest's earlier arrival report.
            self.reports['host']['combat']['stage']['owner'] = 'Red'

    def reached(self, role):
        return True

    def clear_departure(self, goal):
        pass

    def secure(self, **kwargs):
        pass

    def advance(self):
        self.phase += 1
        for report in self.reports.values():
            report['combat']['stage']['owner'] = 'Red' if self.stuck or self.phase == 1 else None
            report['frame']['tick'] = self.phase
            for body in report['frame']['players']:
                body['warped_at'] = 1
                body['player']['grounded'] = self.phase >= 3

    def wait(self, seconds):
        for role in self.commands:
            self.set(role)
        self.advance()

    def until(self, predicate, seconds=120):
        for _ in range(8):
            if predicate():
                return
            self.advance()
        raise AssertionError('Scene did not settle')

    def move_aside(self, role):
        assert all(r['combat']['stage']['owner'] is None for r in self.reports.values()), 'Waiting spot queried during scene'
        self.asides.append(role)

    def run_passage(self):
        self.passage([((50, 0, 0), False)], partner_done=lambda: self.pickup and self.phase >= 1)


class PassageSceneTest(unittest.TestCase):
    def test_host_scene_waits_for_both_clients_and_grounded_reunion(self):
        pair = SceneRace()
        pair.run_passage()
        self.assertGreaterEqual(pair.phase, 3)
        self.assertEqual(pair.asides, [])
        self.assertEqual(len(pair.evidence['passage_partner_completed']), 1)

    def test_scene_timeout_still_fails(self):
        with self.assertRaisesRegex(AssertionError, 'Scene did not settle'):
            SceneRace(stuck=True).run_passage()

    def test_finished_scene_does_not_replace_missing_personal_pickup(self):
        pair = SceneRace(pickup=False)
        pair.run_passage()
        self.assertEqual(pair.asides, ['guest'])
        self.assertNotIn('passage_partner_completed', pair.evidence)


class ReunionEvidenceTest(unittest.TestCase):
    def frames(self, owner='Red'):
        bodies = [dict(feet=[114, 0, 8], grounded=True, warped_at=50),
                  dict(feet=[0, 0, 0], grounded=True, warped_at=50)]
        if owner == 'Blue':
            bodies.reverse()
        return (dict(owner=owner, players=copy.deepcopy(bodies)),
                dict(owner=None, tick=50, players=copy.deepcopy(bodies)))

    def test_existing_grounded_pad_is_valid_for_either_owner(self):
        for owner in ('Blue', 'Red'):
            self.assertEqual(verify_reunion(*self.frames(owner)), 'retained')

    def test_distant_new_placement_is_rejected(self):
        old, row = self.frames()
        old['players'][0]['feet'][0] -= 10
        with self.assertRaisesRegex(AssertionError, 'displaced'):
            verify_reunion(old, row)

    def test_retained_pad_still_has_height_ground_and_distance_limits(self):
        for point, grounded in [([129, 0, 0], True), ([114, 0, 33], True), ([114, 0, 8], False)]:
            old, row = self.frames()
            for frame in (old, row):
                frame['players'][0].update(feet=point, grounded=grounded)
            with self.assertRaises(AssertionError):
                verify_reunion(old, row)

    def test_normal_placement_and_sampled_movement_remain_valid(self):
        old, row = self.frames()
        row['players'][0]['feet'] = [48, 0, 0]
        self.assertEqual(verify_reunion(old, row), 'placed')
        row['players'][0]['feet'] = [114, 0, 8]
        row['tick'] += 1
        self.assertEqual(verify_reunion(old, row), 'placed')

    def test_mismatched_relocation_ticks_are_rejected(self):
        old, row = self.frames()
        row['players'][0]['warped_at'] -= 1
        with self.assertRaisesRegex(AssertionError, 'both presentation tracks'):
            verify_reunion(old, row)


class RejoinResourcesTest(unittest.TestCase):
    def stats(self, **changes):
        return dict(dict(sanity=100, will=8.73755168914795, difficulty='Normal',
            owned=[0, 1], collected=['blade'], selected=0, school_items={'star': True},
            staff_component=False), **changes)

    def test_observed_normal_recovery_during_fresh_guest_start(self):
        verify_rejoin_stats(self.stats(), self.stats(will=8.83423900604248), 116)
        verify_rejoin_stats(self.stats(), self.stats(will=8.824236869812012), 104)

    def test_caps_difficulty_and_dead_players_preserve_resources(self):
        for old, after in [(self.stats(will=9.99), 10), (self.stats(will=52), 52),
                           (self.stats(difficulty='Easy', will=99.9), 100),
                           (self.stats(difficulty='Hard'), 8.73755168914795),
                           (self.stats(difficulty='Nightmare'), 8.73755168914795),
                           (self.stats(sanity=0), 8.73755168914795)]:
            verify_rejoin_stats(old, dict(old, will=after), 120)

    def test_resets_losses_and_other_resource_changes_still_fail(self):
        for changes in [dict(will=100), dict(will=8), dict(will=9.5), dict(sanity=90),
                        dict(owned=[0]), dict(collected=[]), dict(school_items={})]:
            with self.assertRaises(AssertionError):
                verify_rejoin_stats(self.stats(), self.stats(**changes), 116)

    def test_invalid_clock_and_will_are_rejected(self):
        for ticks in [-1, 1.5]:
            with self.assertRaises(AssertionError):
                verify_rejoin_stats(self.stats(), self.stats(), ticks)
        for value in [float('nan'), float('inf')]:
            with self.assertRaises(AssertionError):
                verify_rejoin_stats(self.stats(), self.stats(will=value), 116)


class PersonalRewardIsolationTest(unittest.TestCase):
    visit = 'skool1$return'
    reward = 'drop:skool1$return:3000003'

    def history(self):
        def stats(health, will, collected=False):
            return dict(sanity=health, will=will, difficulty='Normal', owned=[0, 1],
                        collected=[self.reward] if collected else [],
                        loot={self.visit: dict(drops=[] if collected else [dict(id=3000003)])})
        # Recorded Blue collection interval: 28 ticks of ordinary Normal Will
        # recovery in the waiting Red player, followed by her own healing.
        rows = [dict(epoch=1, tick=tick, players=[dict(stats=s) for s in pair])
                for tick, pair in [
                    (74283, [stats(76, 30), stats(80, 1.7975634336471558)]),
                    (74311, [stats(91, 30, True), stats(80, 1.820898413658142)]),
                    (74339, [stats(91, 30, True), stats(95, 1.8442333936691284, True)])]]
        return dict(host=rows, guest=copy.deepcopy(rows))

    def test_passive_will_does_not_hide_separate_healing(self):
        result = personal_rewards(self.history(), self.visit)
        self.assertEqual(result['shared_reward_ids'], [self.reward])
        for observations in result['observations'].values():
            self.assertFalse(observations[self.reward][0]['partner_already_collected'])
            self.assertTrue(observations[self.reward][1]['partner_already_collected'])

    def test_shared_resources_loss_and_unearned_will_are_rejected(self):
        for changes in [dict(sanity=95), dict(will=2.5), dict(will=1),
                        dict(will=float('nan')), dict(owned=[0]),
                        dict(collected=[self.reward]), dict(difficulty='Easy')]:
            with self.subTest(changes=changes):
                history = self.history()
                history['host'][1]['players'][1]['stats'].update(changes)
                with self.assertRaises(AssertionError):
                    personal_rewards(history, self.visit)

    def test_missing_partner_copy_and_missing_guest_proof_are_rejected(self):
        history = self.history()
        history['host'][1]['players'][1]['stats']['loot'][self.visit]['drops'] = []
        with self.assertRaises(AssertionError):
            personal_rewards(history, self.visit)
        history = self.history()
        history['guest'] = history['guest'][:2]
        with self.assertRaises(AssertionError):
            personal_rewards(history, self.visit)

    def test_passive_recovery_respects_caps_and_difficulty(self):
        for difficulty, before, after, valid in [
                ('Normal', 9.99, 10, True), ('Normal', 30, 30, True),
                ('Easy', 99.99, 100, True), ('Hard', 4, 4, True),
                ('Normal', 9.99, 10.001, False), ('Hard', 4, 4.001, False),
                ('Nightmare', 4, 4.02333498, False)]:
            with self.subTest(difficulty=difficulty, before=before, after=after):
                history = self.history()
                for rows in history.values():
                    for i, row in enumerate(rows):
                        row['players'][1]['stats'].update(
                            difficulty=difficulty, will=before if i == 0 else after)
                if valid:
                    personal_rewards(history, self.visit)
                else:
                    with self.assertRaises(AssertionError):
                        personal_rewards(history, self.visit)


class ReturnRejoinTest(unittest.TestCase):
    def report(self, tick, time):
        school = {name: 0 for name in ('books', 'returning', 'theatre', 'library', 'shelves', 'recipe')}
        school['return_visit'] = dict(phase='Observatory', star=False, potion=True,
            scene=dict(active=False, time=time, beat='Star', doors_closed=True, skipped=False))
        return dict(frame=dict(tick=tick), combat=dict(stage=dict(world=dict(school=school))))

    def verify(self, old, new):
        SchoolReturnPair.verify_rejoined_world(None, old, new)

    def test_completed_scene_clock_continues_and_caps_during_rejoin(self):
        self.verify(self.report(100, 12.5), self.report(220, 13.5))
        self.verify(self.report(100, 29.5), self.report(220, 30))
        self.verify(self.report(100, 30), self.report(220, 30))

    def test_scene_restart_reward_reset_and_wrong_clock_are_rejected(self):
        old = self.report(100, 12.5)
        for field, value in [('active', True), ('skipped', True), ('time', 0), ('time', 14.5)]:
            new = self.report(220, 13.5)
            new['combat']['stage']['world']['school']['return_visit']['scene'][field] = value
            with self.assertRaises(AssertionError):
                self.verify(old, new)
        for field, value in [('phase', 'Rising'), ('star', True), ('potion', False)]:
            new = self.report(220, 13.5)
            new['combat']['stage']['world']['school']['return_visit'][field] = value
            with self.assertRaises(AssertionError):
                self.verify(old, new)
        with self.assertRaises(AssertionError):
            self.verify(old, self.report(99, 12.5))


if __name__ == '__main__':
    unittest.main()
