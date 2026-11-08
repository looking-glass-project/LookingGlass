"""Reject false traversal/combat credit in the About Face network verifier."""
import copy
import unittest

from test_coop_hatter2 import DOOR_OPEN, door_open, valid_ending_tail, verify_arrival, verify_combat, verify_ending, verify_tail_duration


def body(x):
    return dict(feet=[x, 160, .03125], health=50, grounded=True, spawned_at=0)


def row(tick, phase='Fight', health=2600):
    return dict(epoch=1, tick=tick, map='hatter2', owner=None,
        hatter=dict(phase=phase, boss=dict(health=health)),
        players=[body(5640 + tick), body(5688 + tick)],
        driver=dict(command=7, reached=True, clear=[True, True]))


class AboutFaceGuards(unittest.TestCase):
    def test_exit_tail_ends_at_transition_not_the_later_entry_scene(self):
        after = dict(tick=1300, hatter=dict(after_talk=2.5))
        self.assertEqual(verify_tail_duration(after, dict(tick=2302)), 1302)
        with self.assertRaises(AssertionError): verify_tail_duration(after, dict(tick=2301))

    def test_scene_without_a_name_requires_the_completed_native_exit(self):
        tail = dict(tick=2000, scene=None, hatter=dict(phase='Done', exit=dict(committed=True),
                    talk_done=True, after_talk=10.866666793823242))
        self.assertTrue(valid_ending_tail(tail, 1990))
        for mutate in (lambda r:r.update(tick=1989),
                       lambda r:r['hatter'].update(phase='ExitReady'),
                       lambda r:r['hatter'].update(after_talk=10.8),
                       lambda r:r['hatter'].update(talk_done=False),
                       lambda r:r['hatter']['exit'].update(committed=False)):
            bad = copy.deepcopy(tail); mutate(bad)
            self.assertFalse(valid_ending_tail(bad, 1990))

    def test_a_single_client_cannot_prove_shared_completion(self):
        history = dict(host=[row(1), row(2), row(3, 'Victory', 0)])
        with self.assertRaises(AssertionError): verify_combat(history)
        with self.assertRaises(AssertionError): verify_ending(history, 'blue')

    def test_a_carried_stamp_in_the_next_visit_is_not_a_new_fight_revival(self):
        history = {role:[row(1), row(2), row(3, 'Victory', 0)] for role in ('host', 'guest')}
        for rows in history.values():
            onward = row(4); onward.update(map='jlair1', epoch=2, hatter=None)
            onward['players'][0]['spawned_at'] = 1
            rows.append(onward)
        self.assertEqual(len(verify_combat(history)), 2)

    def test_native_door_cap_is_accepted_without_accepting_an_unfinished_door(self):
        self.assertLess(DOOR_OPEN, 1.4)
        self.assertTrue(door_open(dict(phase='ExitReady', door=DOOR_OPEN)))
        self.assertFalse(door_open(dict(phase='ExitReady', door=1.39)))
        self.assertFalse(door_open(dict(phase='WatchCat', door=DOOR_OPEN)))

    def test_arrival_requires_the_players_own_observed_body(self):
        history = dict(host=[row(1)], guest=[row(1)])
        receipt = dict(role='guest', epoch=1, tick=1, command=7,
                       goal=[5689, 160, 0], feet=[5689, 160, .03125])
        verify_arrival(receipt, history)
        for mutate in (lambda h: h['guest'][0]['driver'].update(command=6),
                       lambda h: h['guest'][0]['players'][1].update(grounded=False),
                       lambda h: h['guest'][0]['players'][1].update(feet=h['host'][0]['players'][0]['feet']),
                       lambda h: h['guest'][0].update(owner='Blue')):
            bad = copy.deepcopy(history); mutate(bad)
            with self.assertRaises(AssertionError): verify_arrival(receipt, bad)

    def test_only_an_exact_verified_revival_allows_a_later_arrival(self):
        history = dict(guest=[row(1400)])
        history['guest'][0]['players'][1]['spawned_at'] = 1300
        receipt = dict(role='guest', epoch=1, tick=1400, command=7,
            goal=[7088, 160, 0], feet=[7088, 160, .03125])
        event = dict(player=1, epoch=1, death_tick=100, spawn_tick=1300)
        verify_arrival(receipt, history, [event])
        for events in ([], [dict(event, player=0)], [dict(event, spawn_tick=1301)], [dict(event, epoch=2)]):
            with self.assertRaises(AssertionError): verify_arrival(receipt, history, events)

    def test_defeat_requires_both_clients_and_actual_living_start(self):
        history = {role:[row(1), row(2), row(3, 'Victory', 0)] for role in ('host', 'guest')}
        self.assertEqual(len(verify_combat(history)), 2)
        for mutate in (lambda h: h['guest'].pop(),
                       lambda h: h['guest'][-1]['hatter']['boss'].update(health=15),
                       lambda h: h['host'][0]['hatter']['boss'].update(health=0),
                       lambda h: h['host'][1]['players'][1].update(feet=h['host'][0]['players'][1]['feet'])):
            bad = copy.deepcopy(history); mutate(bad)
            with self.assertRaises(AssertionError): verify_combat(bad)

    def test_combat_death_cannot_disappear_from_the_evidence(self):
        event = dict(player=0, epoch=1, death_tick=100, spawn_tick=1300)
        history = {role:[row(1), row(100), row(1300), row(1500, 'Victory', 0)] for role in ('host', 'guest')}
        for rows in history.values():
            rows[1]['players'][0]['health'] = 0
            for r in rows[2:]: r['players'][0].update(health=25, spawned_at=1300)
        verify_combat(history, [event])
        for events in ([], [dict(event, player=1)], [dict(event, spawn_tick=1299)], [dict(event, death_tick=101)]):
            with self.assertRaises(AssertionError): verify_combat(history, events)


if __name__ == '__main__': unittest.main()
