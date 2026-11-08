"""Reject a suspension check without a fresh handshake and bounded recovery."""
import unittest
from test_coop_castling_suspend import verify_suspension


class SuspensionGuards(unittest.TestCase):
    def fixture(self):
        return dict(seconds=.65, before_reset=0, reset=1, before_tick=100,
                    ack_tick=120, verified_tick=360,
                    settled_samples=[[tick, 12] for tick in range(240,361,24)])

    def test_complete_recovery(self):
        verify_suspension(self.fixture())

    def test_no_reset_or_no_real_suspension(self):
        for change in (dict(seconds=.1), dict(reset=0), dict(ack_tick=99)):
            event=self.fixture();event.update(change)
            with self.assertRaises(AssertionError):verify_suspension(event)

    def test_incomplete_or_delayed_recovery(self):
        for change in (dict(verified_tick=200), dict(settled_samples=[[300, 12]]),
                       dict(settled_samples=[[tick, 1000] for tick in range(240,361,24)])):
            event=self.fixture();event.update(change)
            with self.assertRaises(AssertionError):verify_suspension(event)


if __name__ == '__main__':
    unittest.main()
