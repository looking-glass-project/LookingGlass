"""Negative controls for Checkmate's real shared-wipe evidence."""
import copy
import unittest

from coop_recovery import verify_presence
from test_coop_checkmate_wipe import verify_wipe_trace
from test_coop_checkmate_rejoin import verify_checkmate_rejoin


def fixture():
    saved = dict(visit=dict(epoch=1), frame=dict(players=[
        dict(player=dict(feet=[i*48, 0, 0]), stats=dict(sanity=100, owned=[i+1]),
             spawned_at=0, warped_at=20) for i in range(2)]))
    def row(serial, tick, generation):
        return dict(version=1, serial=serial, tick=tick, checkpoint_generation=generation,
            build='fixture', epoch=1, cinematic=False, spawn_held=False, players=[
                dict(life='Alive', health=100, feet=copy.deepcopy(old['player']['feet']),
                     stats=copy.deepcopy(old['stats']), spawned_at=tick if generation else 0,
                     warped_at=20, god=False, notarget=False,
                     geometry=dict(clear=True, valid=True, safe=True, supported=True, swimming=False, rope=False))
                for old in saved['frame']['players']])
    rows = [row(1, 100, 0), row(2, 101, 0), row(3, 102, 1)]
    rows[1]['players'][0].update(health=0, life=dict(Spectating=dict(ticks_left=1200)))
    return saved, rows


class CheckmateWipeGuards(unittest.TestCase):
    def test_exact_saved_players_return_at_one_shared_boundary(self):
        saved, rows = fixture()
        self.assertEqual(verify_wipe_trace(rows, saved, 'fixture'),
                         dict(tick=102, generation=1, exact_players=True, trace_rows=3))

    def test_wrong_resources_placement_or_spawn_cannot_pass(self):
        for field, value in [('stats', dict(sanity=100, owned=[9])), ('feet', [96, 0, 0]),
                             ('spawned_at', 101), ('warped_at', 21), ('health', 25),
                             ('life', dict(Spectating=dict(ticks_left=1))), ('god', True)]:
            saved, rows = fixture()
            rows[-1]['players'][1][field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_wipe_trace(rows, saved, 'fixture')

    def test_missing_death_boundary_and_individual_revive_are_rejected(self):
        for case in ('gap', 'serial', 'no_death', 'no_wipe', 'repeat', 'individual', 'scope', 'unsafe'):
            saved, rows = fixture()
            if case == 'gap': rows[-1]['tick'] += 1
            elif case == 'serial': rows[-1]['serial'] += 1
            elif case == 'no_death': rows[1]['players'][0].update(life='Alive', health=100)
            elif case == 'no_wipe': rows.pop()
            elif case == 'repeat': rows[-1]['checkpoint_generation'] = 2
            elif case == 'individual': rows[1]['players'][1]['spawned_at'] = 101
            elif case == 'scope': rows[-1]['build'] = 'other'
            elif case == 'unsafe': rows[-1]['players'][0]['geometry']['safe'] = False
            with self.subTest(case=case), self.assertRaises(AssertionError):
                verify_wipe_trace(rows, saved, 'fixture')

    def test_route_presence_still_rejects_wipes_unless_explicitly_checked(self):
        row = dict(checkpoint_generation=1, owner=None, camera_player='Blue',
            camera_target=[0, 0, 48], camera_subject_feet=[0, 0, 0],
            players=[dict(health=100, feet=[0, 0, 0]), dict(health=100, feet=[48, 0, 0])])
        with self.assertRaisesRegex(AssertionError, 'Unexpected team wipe'):
            verify_presence([row], 'host')
        verify_presence([row], 'host', generation=1)

    def test_rejoin_preserves_native_residents_and_completed_progress(self):
        level = dict(beheaded=True, ambushed=[1, 3], phase='Northern', checkpoint=True,
            exit=dict(committed=False), essence=0, coop=dict(essence=[0, 1]), boss=dict(health=1300),
            pieces=[[23, dict(kind='Pawn', variant=0, health=0)],
                    [24, dict(kind='Rook', variant=1, health=50)]])
        before = dict(combat=dict(stage=dict(owner=None, world=dict(levels=dict(rchess1=level)),
            story=dict(completed=1, seen=['beheading']))))
        verify_checkmate_rejoin(before, copy.deepcopy(before))
        for case in ('scene', 'progress', 'healing', 'king', 'piece', 'replayed'):
            after = copy.deepcopy(before)
            stage = after['combat']['stage']; current = stage['world']['levels']['rchess1']
            if case == 'scene': stage['owner'] = 'Red'
            elif case == 'progress': current['ambushed'].clear()
            elif case == 'healing': current['coop']['essence'][1] = 0
            elif case == 'king': current['boss']['health'] = 1400
            elif case == 'piece': current['pieces'][0][1]['health'] = 100
            elif case == 'replayed': stage['story']['completed'] += 1
            with self.subTest(case=case), self.assertRaises(AssertionError):
                verify_checkmate_rejoin(before, after)


if __name__ == '__main__':
    unittest.main()
