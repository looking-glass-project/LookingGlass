"""Reject incomplete, early or inconsistent scene-skip evidence."""
import copy
import unittest
from test_coop_castling_skip import verify_skip_event


class SkipGuards(unittest.TestCase):
    def fixture(self):
        return dict(partner_request=100, owner_request=280,
                    acknowledged=dict(host=380, guest=380), reunited=['guest', 'host'])

    def test_owner_hold_after_partner_only_interval(self):
        verify_skip_event(self.fixture())

    def test_early_skip_and_missing_partner_test_fail(self):
        for change in (dict(owner_request=150), dict(acknowledged=dict(host=300, guest=300))):
            event=self.fixture();event.update(change)
            with self.assertRaises(AssertionError): verify_skip_event(event)

    def test_missing_or_disagreeing_clients_and_reunion_fail(self):
        for change in (dict(acknowledged=dict(host=380)),
                       dict(acknowledged=dict(host=380, guest=381)), dict(reunited=['host'])):
            event=copy.deepcopy(self.fixture());event.update(change)
            with self.assertRaises(AssertionError): verify_skip_event(event)


if __name__ == '__main__':
    unittest.main()
