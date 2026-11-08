"""Reject stale restore proof and unexpected lifecycle/resource changes."""
import copy
import unittest
from unittest.mock import patch

from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from test_coop_chess import verify_spike_exit_progress


class ResumeGuards(unittest.TestCase):
    def test_captured_backtracking_is_rejected_after_crossing(self):
        steps = [dict(mode='spikes', hold=[-784, 400, 256], pos=[-1120, 384, 224]), dict(mode='walk')]
        row = dict(driver={'chess_course': {'step': 1}}, players=[{'feet': [-1050.9, 399, 224]},
                                                               {'feet': [-1181.9, 404, 192]}])
        verify_spike_exit_progress([row], steps)
        row['players'][0]['feet'] = [-716.13, 399, 256]
        with self.assertRaises(AssertionError):
            verify_spike_exit_progress([row], steps)

    def test_only_exact_build_epoch_and_tick_receipt_is_accepted(self):
        saved = {'visit': {'epoch': 1}, 'frame': {'tick': 14400}}
        good = dict(verified=True, build='current', epoch=1, tick=14400)
        verify_resume_receipt(good, saved, 'current')
        for field, bad in [('verified', False), ('verified', 1), ('build', 'old'),
                           ('epoch', 2), ('tick', 14401)]:
            with self.subTest(field=field, bad=bad), self.assertRaises(AssertionError):
                verify_resume_receipt(dict(good, **{field: bad}), saved, 'current')
        with self.assertRaises(KeyError):
            verify_resume_receipt({}, saved, 'current')

    def test_momentum_does_not_hide_respawn_or_teleport(self):
        old = dict(spawned_at=0, warped_at=120, stats={}, player={'feet': [0, 0, 0]})
        current = copy.deepcopy(old)
        current['player']['feet'] = [0, 6, 0]
        with patch('test_coop_chess_resume.verify_rejoin_stats') as resources:
            verify_resume_body(current, old, 60)
            resources.assert_called_once_with(old['stats'], current['stats'], 60)
            for stamp in ('spawned_at', 'warped_at'):
                bad = dict(current, **{stamp: old[stamp] + 1})
                with self.subTest(stamp=stamp), self.assertRaises(AssertionError):
                    verify_resume_body(bad, old, 60)
            with self.assertRaises(AssertionError):
                verify_resume_body(current, old, -1)

    def test_resource_failure_still_fails_the_resume(self):
        body = dict(spawned_at=0, warped_at=0, stats={})
        with patch('test_coop_chess_resume.verify_rejoin_stats', side_effect=AssertionError('changed health')):
            with self.assertRaises(AssertionError):
                verify_resume_body(body, body, 60)


if __name__ == '__main__':
    unittest.main()
