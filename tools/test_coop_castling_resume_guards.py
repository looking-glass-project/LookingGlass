"""Reject lost progression and reset actors in the Castling rejoin evidence."""
import copy
import unittest

from test_coop_castling_resume import verify_castling_rejoin


class RejoinGuards(unittest.TestCase):
    def report(self):
        castle = dict(queen=True, queen_release=10, king=True, portal=40,
            scene=None, pending=None, fired=['queen', 'king'], age=100,
            cast=[dict(entity=1, instance=0, model='rook', white=False, counted=True,
                       piece=dict(kind='Rook', variant=0, health=20))])
        return dict(combat=dict(stage=dict(owner=None, world=dict(levels=dict(wchess2=castle)),
                                          story=dict(completed=2, seen=['queen', 'king']))))

    def test_idle_clocks_can_advance(self):
        before, after = self.report(), self.report()
        after['combat']['stage']['world']['levels']['wchess2']['age'] += 2
        verify_castling_rejoin(before, after)

    def test_royal_progress_cannot_reset(self):
        before = self.report()
        for field, value in [('queen', False), ('king', False), ('portal', 50), ('queen_release', 12),
                             ('pending', 'King'), ('fired', ['queen'])]:
            after = copy.deepcopy(before)
            after['combat']['stage']['world']['levels']['wchess2'][field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_castling_rejoin(before, after)

    def test_story_cannot_forget_completed_scenes(self):
        before = self.report()
        for field, value in [('completed', 1), ('seen', ['queen'])]:
            after = copy.deepcopy(before)
            after['combat']['stage']['story'][field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_castling_rejoin(before, after)

    def test_actors_cannot_reset_or_duplicate(self):
        before = self.report()
        for mutation in ('health', 'identity', 'duplicate'):
            after = copy.deepcopy(before)
            actors = after['combat']['stage']['world']['levels']['wchess2']['cast']
            if mutation == 'health':
                actors[0]['piece']['health'] = 200
            elif mutation == 'identity':
                actors[0]['instance'] += 1
            else:
                actors.append(copy.deepcopy(actors[0]))
            with self.subTest(mutation=mutation), self.assertRaises(AssertionError):
                verify_castling_rejoin(before, after)


if __name__ == '__main__':
    unittest.main()
