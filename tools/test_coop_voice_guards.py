"""Reject partial, repeated or unsynchronized opening-dialogue evidence."""
import unittest

from test_coop_voice import OPENING_VOICES, verify_opening_voices


def watched_opening():
    rows = [dict(path=None, starts=0, elapsed=0, running=False, story=None)]
    for index, path in enumerate(OPENING_VOICES):
        for elapsed, running in ((0, True), (2, True), (3, False)):
            rows.append(dict(path=path, starts=index+1, elapsed=elapsed,
                             running=running, story=['entry', index, elapsed]))
    return rows


class OpeningVoiceGuards(unittest.TestCase):
    def test_complete_watched_opening(self):
        self.assertEqual(verify_opening_voices(watched_opening()), OPENING_VOICES)

    def test_partial_or_reordered_lines_cannot_pass(self):
        rows = watched_opening()
        with self.assertRaisesRegex(AssertionError, 'Incomplete'):
            verify_opening_voices(rows[:10])  # The old three-line minimum passed.
        for row in rows:
            if row['path'] == OPENING_VOICES[0]: row['path'] = OPENING_VOICES[1]
            elif row['path'] == OPENING_VOICES[1]: row['path'] = OPENING_VOICES[0]
        with self.assertRaisesRegex(AssertionError, 'reordered'):
            verify_opening_voices(rows)

    def test_restarts_and_clock_changes_cannot_pass(self):
        for defect in ('count', 'restart', 'rewind', 'host_clock'):
            rows = watched_opening()
            if defect == 'count': rows[2]['starts'] += 1
            elif defect == 'restart': rows.insert(2, dict(path=None, starts=1, elapsed=0, running=False, story=None))
            elif defect == 'rewind':
                rows[3]['elapsed'] = 1
                rows[3]['story'][2] = 1
            else: rows[2]['story'][2] += .5
            with self.subTest(defect=defect), self.assertRaises(AssertionError):
                verify_opening_voices(rows)

    def test_unwatched_or_unheld_dialogue_cannot_pass(self):
        for running in (True, False):
            rows = watched_opening()
            for row in rows: row['running'] = running
            with self.subTest(running=running), self.assertRaises(AssertionError):
                verify_opening_voices(rows)


if __name__ == '__main__': unittest.main()
