"""Guard the reconnect verifier against plausible progression/health resets."""
import copy
import unittest

from test_coop_chess_rejoin import ChessRejoin, verify_chess_rejoin


def report():
    level = dict(intro=True, bishop_done=True, knight_done=True, knight_gate=True,
                 knight_opened=12., bell=8., spikes=3., spikes_off=5., water=18.,
                 secret=False, lever_started=[12., 18.], fired=[1, 2], age=30.,
                 board=None, pending=None, scene=None, coop=None,
                 cast=[dict(entity=20, instance=0, model='pawn', white=False, counted=True,
                            piece=dict(kind='Pawn', variant=2, health=0., time=4.))])
    return dict(combat=dict(stage=dict(owner=None, story=dict(completed=3, seen=['intro', 'bishop', 'knight']),
                                      world=dict(levels=dict(wchess1=level)))))


class RejoinGuards(unittest.TestCase):
    def network_pair(self, ticks):
        pair = object.__new__(ChessRejoin)
        pair.evidence = {'rejoin': {'after_tick': 100, 'epoch': 1}}
        states = [dict(epoch=1, tick=tick, sha256=str(tick)) for tick in ticks]
        pair.history = {role: [dict(driver=dict(network_states=copy.deepcopy(states)))]
                        for role in ('host', 'guest')}
        return pair

    def test_old_matching_frames_cannot_pass_the_rejoined_client_check(self):
        pair = self.network_pair(range(20))
        with self.assertRaisesRegex(AssertionError, 'after the new guest joined'):
            pair.verify_network_states()

    def test_new_matching_frames_pass_and_divergence_fails(self):
        pair = self.network_pair(list(range(20)) + list(range(100, 120)))
        pair.verify_network_states()
        self.assertEqual(pair.evidence['rejoin_matching_sent_and_restored_states'], 20)
        pair.history['guest'][0]['driver']['network_states'][-1]['sha256'] = 'wrong'
        with self.assertRaises(AssertionError):
            pair.verify_network_states()

    def test_clock_and_animation_can_advance_with_progress_and_health_preserved(self):
        before = report()
        after = copy.deepcopy(before)
        level = after['combat']['stage']['world']['levels']['wchess1']
        level['age'] += 2
        level['cast'][0]['piece']['time'] += 2
        verify_chess_rejoin(before, after)

    def test_rejects_progression_reset_and_new_puzzle(self):
        for field, value in [('bishop_done', False), ('knight_done', False), ('water', None),
                             ('knight_opened', 0.), ('board', {}), ('coop', {'reunion': True})]:
            with self.subTest(field=field):
                before, after = report(), report()
                after['combat']['stage']['world']['levels']['wchess1'][field] = value
                with self.assertRaises(AssertionError):
                    verify_chess_rejoin(before, after)

    def test_rejects_forgotten_events_and_scenes(self):
        for kind in ('event', 'scene', 'count'):
            before, after = report(), report()
            if kind == 'scene':
                after['combat']['stage']['story']['seen'].pop()
            elif kind == 'count':
                after['combat']['stage']['story']['completed'] = 0
            else:
                after['combat']['stage']['world']['levels']['wchess1']['fired'].pop()
            with self.assertRaises(AssertionError):
                verify_chess_rejoin(before, after)

    def test_rejects_enemy_respawn_and_identity_changes(self):
        for mutation in ('health', 'missing', 'duplicate', 'model'):
            with self.subTest(mutation=mutation):
                before, after = report(), report()
                cast = after['combat']['stage']['world']['levels']['wchess1']['cast']
                if mutation == 'health': cast[0]['piece']['health'] = 100.
                elif mutation == 'missing': cast.clear()
                elif mutation == 'duplicate': cast.append(copy.deepcopy(cast[0]))
                else: cast[0]['model'] = 'bishop'
                with self.assertRaises(AssertionError):
                    verify_chess_rejoin(before, after)


if __name__ == '__main__':
    unittest.main()
