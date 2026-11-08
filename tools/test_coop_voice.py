"""Watch the unskipped village opening in two real internet clients.

Checks replicated dialogue positions and that repeated snapshots do not restart
voices. Records each app's actual mix; no microphone or system audio is captured.
This is an opening-scene fixture, not another complete campaign verification.
"""
import argparse
import json
from pathlib import Path
import wave

from test_coop_village import Pair, ROOT


# The entry registry has one Rabbit line followed by five introductory lines.
# These are asset identifiers only; dialogue and recordings stay with the user.
OPENING_VOICES = [
    'sound/character/white_rabbit/vo/rbt000.wav',
    'sound/character/alice/vo/alcz1001.wav',
    'sound/character/cheshire_cat/vo/catz101.wav',
    'sound/character/alice/vo/alcz1002.wav',
    'sound/character/cheshire_cat/vo/catz102.wav',
    'sound/character/cheshire_cat/vo/catz103.wav',
]


def verify_opening_voices(rows):
    """Require the whole watched opening, with one start per authored line."""
    order = []
    previous = None
    for row in rows:
        if row['path'] and (previous is None or previous['path'] != row['path']):
            order.append(row['path'])
        assert row['starts'] == len(order), 'Missed or restarted a voice'
        if previous is not None and previous['path'] == row['path']:
            assert row['elapsed'] >= previous['elapsed'], 'Dialogue clock ran backwards'
        if row['story'] is not None:
            assert abs(row['story'][2] - row['elapsed']) < .001, 'Audio does not follow the host dialogue clock'
        previous = row
    assert order == OPENING_VOICES, 'Incomplete or reordered opening dialogue'
    assert any(row['running'] and row['elapsed'] > 1 for row in rows), 'No watched spoken line'
    assert any(not row['running'] and row['path'] for row in rows), 'No cinematic speech hold observed'
    return order


class VoicePair(Pair):
    evidence_prefix = 'village-voice-'

    def __init__(self, args):
        self.voice_rows = {'host': [], 'guest': []}
        super().__init__(args)

    def set(self, role, *args, **kwargs):
        super().set(role, *args, **kwargs)
        self.commands[role]['watch_dialogue'] = True

    def validate_report(self, role, report):
        super().validate_report(role, report)
        audio = report['audio']
        stage = report['combat']['stage']
        row = dict(tick=report['frame']['tick'], starts=audio['voice_starts'],
                   path=audio['voice_path'], elapsed=audio['voice_elapsed'],
                   running=stage['voice_running'], story=stage['story']['current'])
        rows = self.voice_rows[role]
        if rows and rows[-1]['tick'] == row['tick']:
            return
        if rows and rows[-1]['path'] == row['path']:
            assert row['starts'] == rows[-1]['starts'], 'Repeated snapshot restarted a voice'
            assert row['elapsed'] >= rows[-1]['elapsed'], 'Dialogue clock ran backwards'
        if row['story'] is not None:
            assert abs(row['story'][2] - row['elapsed']) < .001, 'Audio does not follow the host dialogue clock'
        rows.append(row)

    def run(self):
        self.start_pair()
        self.until(lambda: all('entry' in r['combat']['stage']['world']['village']['cinema']['done']
                              and r['combat']['stage']['owner'] is None for r in self.reports.values()), 180)
        line_orders = {}
        for role, rows in self.voice_rows.items():
            order = verify_opening_voices(rows)
            audio = self.reports[role]['audio']
            assert audio['output_frames'] > 44100 and audio['peak'] > .001 and audio['missing'] == 0
            line_orders[role] = order
        assert line_orders['host'] == line_orders['guest'], 'Players heard different dialogue order'
        self.evidence['line_orders'] = line_orders
        self.evidence['dialogue_not_skipped'] = True
        self.verify_network_states()
        self.finish_pair(1)
        recordings = {}
        for path in sorted(self.folder.glob('*.wav')):
            with wave.open(str(path), 'rb') as wav:
                assert wav.getnchannels() == 2 and wav.getframerate() == 44100 and wav.getnframes() > 44100
                recordings[path.name] = {'frames': wav.getnframes(), 'sample_rate': wav.getframerate()}
        assert len(recordings) == 2
        self.evidence['recordings'] = recordings
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, internet=not self.args.local,
            build=self.build, delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            scope='unskipped village opening dialogue in two clients; not a campaign route',
            evidence=self.evidence), indent=2))
        print('PASS unskipped opening voices: same line order, one start per line, host clock and cinematic holds', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    args.audio = True
    args.first = 'blue'
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = VoicePair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps({'passed': False, 'error': str(error)}, indent=2))
        raise
    finally:
        (pair.folder / 'voices.json').write_text(json.dumps(pair.voice_rows, indent=2))
        pair.close()
