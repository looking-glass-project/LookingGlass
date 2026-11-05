"""A restart must preserve the actual personal/native state and new snapshots."""
import copy
import unittest

from test_coop_hatter2_recovery import passive_victory_state, verify_rejoined_world, verify_restart_states


def report(tick, clock):
    state = dict(phase='Victory', lifts=0, door=0, drops=[], clock=clock, time=clock,
        remainder=0, boss=dict(health=0, action='Gone'),
        coop=dict(items=dict(blade=[True, False], wait=[0, 0], ready=[False, False], drop_taken=[])))
    return dict(frame=dict(tick=tick), combat=dict(stage=dict(owner=None,
        world=dict(levels=dict(hatter2=state)), story=dict(seen=['hatter_cat'], completed=1))))


class RestartGuards(unittest.TestCase):
    def test_restart_does_not_reset_personal_or_shared_progress(self):
        before, after = report(1000, 50), report(1120, 51)
        after['combat']['stage']['world']['levels']['hatter2'] = passive_victory_state(
            before['combat']['stage']['world']['levels']['hatter2'], 120)
        verify_rejoined_world(before, after)
        for mutate in (lambda r:r['combat']['stage']['world']['levels']['hatter2']['boss'].update(health=1),
                       lambda r:r['combat']['stage']['world']['levels']['hatter2']['coop']['items'].update(blade=[True, True]),
                       lambda r:r['combat']['stage']['world']['levels']['hatter2'].update(clock=0),
                       lambda r:r['combat']['stage']['world']['levels']['hatter2'].update(clock=55),
                       lambda r:r['combat']['stage']['story'].update(seen=[])):
            bad = copy.deepcopy(after); mutate(bad)
            with self.assertRaises(AssertionError): verify_rejoined_world(before, bad)

    def test_expired_drop_keeps_the_surviving_drops_personal_mask(self):
        before, after = report(1000, 50), report(1120, 51)
        old = before['combat']['stage']['world']['levels']['hatter2']
        old['drops'] = [[[10, 0, 20], 69.5], [[20, 0, 20], 10]]
        old['coop']['items']['drop_taken'] = [[True, False], [False, True]]
        expected = passive_victory_state(old, 120)
        self.assertEqual(expected['coop']['items']['drop_taken'], [[False, True]])
        self.assertEqual([p for p, _ in expected['drops']], [[20, 0, 20]])
        after['combat']['stage']['world']['levels']['hatter2'] = expected
        verify_rejoined_world(before, after)
        for mutate in (lambda s:s['coop']['items'].update(drop_taken=[[True, False]]),
                       lambda s:s['drops'][0].__setitem__(1, 0),
                       lambda s:s.update(drops=[])):
            bad = copy.deepcopy(after); mutate(bad['combat']['stage']['world']['levels']['hatter2'])
            with self.assertRaises(AssertionError): verify_rejoined_world(before, bad)

    def test_moving_machinery_cannot_be_claimed_as_an_idle_restart(self):
        before, after = report(1000, 50), report(1120, 51)
        for r in (before, after): r['combat']['stage']['world']['levels']['hatter2']['lifts'] = 1
        with self.assertRaises(AssertionError): verify_rejoined_world(before, after)

    def test_pre_restart_snapshots_do_not_prove_the_new_guest(self):
        def history(start):
            return {role:[dict(driver=dict(network_states=[dict(epoch=1, tick=start+i, sha256=str(i))
                for i in range(20)]))] for role in ('host', 'guest')}
        self.assertEqual(verify_restart_states(history(200), 200, 'sha256'), 20)
        with self.assertRaises(AssertionError): verify_restart_states(history(100), 200, 'sha256')
        bad = history(200); bad['guest'][0]['driver']['network_states'][0]['sha256'] = 'changed'
        with self.assertRaises(AssertionError): verify_restart_states(bad, 200, 'sha256')
        with self.assertRaises(AssertionError): verify_restart_states(dict(host=history(200)['host']), 200, 'sha256')


if __name__ == '__main__': unittest.main()
