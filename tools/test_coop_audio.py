"""Staged Beyond musical-puzzle audio in two public internet clients.

Only the initial placement is staged. Real movement, E presses, scene clocks,
sound output and guest rejoin run through production code. Not a full map route.
Run this headed fixture in Anode, with the supplied original game data.
"""
import argparse
import json
from pathlib import Path
import wave

from test_coop_village import Pair, ROOT
from test_coop_play import distance

NOTES = [f'sound/ambience/special/door_machine_tone{i}.wav' for i in (3, 4, 5)]
RESET_GOALS = {'host': (1200, 457, 0), 'guest': (1200, 537, 0)}


def verify_reset_approaches(entries, history):
    assert len(entries)==2 and {e['role'] for e in entries}=={'host','guest'}, 'Missing personal lever approach'
    for entry in entries:
        role=entry['role'];slot=0 if role=='host' else 1
        assert entry['goal']==list(RESET_GOALS[role]), 'Changed lever approach'
        row=next(r for r in history[role] if r['tick']==entry['tick'] and r['epoch']==1)
        body=row['players'][slot];other=row['players'][1-slot]
        assert row['map']=='fortress2' and row['owner'] is None, 'Approach is not ordinary play'
        assert row['driver']['command']==entry['command'] and row['driver']['reached'], 'Unobserved lever approach'
        assert body['health']>0 and body['grounded'] and all(row['driver']['clear']), 'Unsafe lever approach'
        assert body['feet']==entry['feet'] and distance(body['feet'][:2],entry['goal'][:2])<8 and abs(body['feet'][2]-entry['goal'][2])<8, 'Missed lever approach'
        assert distance(body['feet'][:2],other['feet'][:2])>=35.95 or abs(body['feet'][2]-other['feet'][2])>=55.95, 'Lever approach overlaps partner'


class AudioPair(Pair):
    map_name = 'fortress2'
    evidence_prefix = 'beyond-audio-'
    def __init__(self, args):
        self.start_at = (960, 280, 20)
        self.heard = {}
        self.audio_reports = {}
        super().__init__(args)

    def set(self, role, *args, use_aim=None, **kwargs):
        super().set(role, *args, **kwargs)
        self.commands[role]['use_aim'] = use_aim

    def validate_report(self, role, report):
        super().validate_report(role, report)
        audio = report['audio']
        assert len({c['sequence'] for c in audio['recent']}) == len(audio['recent']), 'A sound played twice'
        key = f'{role}:{report["pid"]}'
        seen = self.heard.setdefault(key, {})
        for cue in audio['recent']:
            previous = seen.setdefault(cue['sequence'], cue)
            assert previous == cue, 'A numbered sound changed after playback'
        self.audio_reports[key] = audio

    def notes(self, role):
        key = f'{role}:{self.reports[role]["pid"]}'
        return [cue for _, cue in sorted(self.heard.get(key, {}).items()) if cue['path'] in NOTES]

    def sequence(self, before):
        self.until(lambda: all(len(self.notes(r)) >= before[r] + 3 for r in self.commands), 30)
        pair = {}
        for role in self.commands:
            cues = self.notes(role)[before[role]:]
            assert [cue['path'] for cue in cues] == NOTES, (role, cues)
            pair[role] = cues
        assert pair['host'] == pair['guest'], 'Players did not hear the same numbered notes'
        self.evidence.setdefault('sequences', []).append(pair)

    def verify_rejoined_world(self, before, report):
        old, now = (r['combat']['stage']['world']['beyond'] for r in (before, report))
        for key in ('puzzle_started', 'solved', 'notes', 'lever_used', 'walkway', 'step_slots', 'last_open'):
            assert now[key] == old[key], f'Rejoin changed {key}'

    def operate(self, role):
        before = {r: len(self.notes(r)) for r in self.commands}
        self.set(role, use_aim=(0, 0), attack=False)
        owner = 'Blue' if role == 'host' else 'Red'
        self.until(lambda: self.reports['host']['combat']['stage']['owner'] == owner
                   and self.reports['host']['combat']['stage']['world']['beyond']['cinema']['beat'] == 'Reset', 10)
        self.sequence(before)
        self.until(lambda: all(r['combat']['stage']['owner'] is None for r in self.reports.values()), 15)
        self.wait(.5)
        print('PASS original reset scene and ordered notes for', owner, flush=True)

    def approach_reset_lever(self):
        # Both positions can see/reach the original lever. An 80-unit gap lets
        # each player settle without asking contact physics to overlap bodies.
        first='host' if self.args.first=='blue' else 'guest'
        entries=[]
        for role in self.commands:self.set(role,attack=True)
        for role in (first,'guest' if first=='host' else 'host'):
            self.set(role,RESET_GOALS[role],navigate=True,attack=True)
            self.until(lambda:self.reached(role),60)
        # Keep both goals active while ordinary combat/knockback settles.
        self.until(lambda:all(self.reached(role) for role in self.commands),60)
        for role in (first,'guest' if first=='host' else 'host'):
            row=self.history[role][-1];slot=0 if role=='host' else 1
            entries.append(dict(role=role,goal=list(RESET_GOALS[role]),tick=row['tick'],
                                command=self.commands[role]['id'],feet=row['players'][slot]['feet']))
        verify_reset_approaches(entries,self.history)
        self.evidence['reset_approaches']=entries
        print('PASS both separated reset-lever approaches',flush=True)

    def run(self):
        self.start_pair()
        self.wait(1)
        before = {r: len(self.notes(r)) for r in self.commands}
        assert before == {'host': 0, 'guest': 0}, 'Fixture started inside the demonstration trigger'
        first = 'host' if self.args.first == 'blue' else 'guest'
        self.set(first, (960, 376, 0), attack=False)
        self.until(lambda: self.reports['host']['combat']['stage']['world']['beyond']['puzzle_started'], 20)
        assert self.reports['host']['combat']['stage']['owner'] == self.args.first.title()
        self.sequence(before)
        self.until(lambda: all(r['combat']['stage']['owner'] is None for r in self.reports.values()), 20)
        self.rejoin_guest()
        assert not self.notes('guest'), 'A new guest replayed the old demonstration notes'
        self.approach_reset_lever()
        self.operate('host')
        self.operate('guest')
        for role, report in self.reports.items():
            audio = report['audio']
            assert audio['output_frames'] > 44100 and audio['peak'] > .001, f'{role}: no real audio output'
            assert audio['resolved'] >= len(self.notes(role)) and audio['missing'] == 0, f'{role}: unresolved sound cues'
        self.verify_network_states()
        self.finish_pair(1)
        recordings = {}
        for path in sorted(self.folder.glob('*.wav')):
            with wave.open(str(path), 'rb') as wav:
                frames = wav.getnframes()
                assert wav.getnchannels() == 2 and wav.getframerate() == 44100 and frames > 44100
                recordings[path.name] = {'frames': frames, 'sample_rate': wav.getframerate()}
        assert len(recordings) == 3, 'Missing per-process app audio capture'
        self.evidence['recordings'] = recordings
        (self.folder / 'audio.json').write_text(json.dumps(self.audio_reports, indent=2))
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, internet=not self.args.local,
            build=self.build, first=self.args.first, delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            scope='staged musical demonstration, both reset operators and fresh guest rejoin; not a campaign route',
            evidence=self.evidence), indent=2))
        print('PASS co-op audio: same notes once on both clients, both operators, fresh guest, real mixer output and app captures', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    args.audio = True
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500): parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = AudioPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps({'passed':False, 'error':str(error)},indent=2))
        raise
    finally:
        (pair.folder / 'audio.json').write_text(json.dumps(pair.audio_reports, indent=2))
        pair.close()
