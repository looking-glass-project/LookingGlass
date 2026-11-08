"""Repeat a school route segment from an unchanged ordinary co-op checkpoint.

Save through the host's ESC menu at an optional school fixture checkpoint stop.
This copies the resulting file byte-for-byte into a fresh test profile and uses
the normal --coop-resume path. A pass verifies only the resumed segment, never
the complete campaign visit. No positions, health or world state are edited.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_school import SchoolPair
from test_coop_play import ROOT, distance


class SchoolResume(SchoolPair):
    evidence_prefix = 'school-resume-'
    resume_checkpoint = True

    def __init__(self,args):
        saved = args.checkpoint.read_bytes()
        payload = json.loads(saved)['payload']
        assert payload['visit']['map'] == 'skool1' and payload['visit']['epoch'] == 1
        assert payload['frame']['checkpoint_generation'] == 0
        super().__init__(args)
        (self.folder/'host'/'coop-checkpoint.json').write_bytes(saved)
        self.saved = payload
        self.evidence['checkpoint_sha256'] = hashlib.sha256(saved).hexdigest()
        self.evidence['checkpoint_tick'] = payload['frame']['tick']
        self.evidence['resume_point'] = args.resume_point
        source = Path(__file__).read_bytes()
        (self.folder/'resume-fixture-source.py').write_bytes(source)
        self.evidence['resume_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def verify_segment(self):
        expected = {'Book_Ingredients_Exit'}
        if self.args.resume_point != 'descent':
            expected.add('book_cinematic')
        if self.args.resume_point in ('theatre','balcony','spiral','library'):
            expected.add('Skool1_Book_Win')
        if self.args.resume_point in ('theatre','balcony','spiral'):
            expected.add('shelf_cinematic')
        assert self.captures == expected, self.captures
        for role,rows in self.history.items():
            visit = [r for r in rows if r['epoch'] == 1]
            owners = {scene:{r['owner'] for r in visit if r['scene'] == scene} for scene in expected}
            assert all(ids == {self.args.first.title()} for ids in owners.values()), owners
            assert all(r['checkpoint_generation'] == self.saved['frame']['checkpoint_generation']
                       for r in visit), 'Segment silently restored a checkpoint'
            assert all(b['spawned_at'] == old['spawned_at'] and b['health'] > 0
                       for r in visit for b,old in zip(r['players'],self.saved['frame']['players'])), \
                'Segment silently died or respawned'
            reunions,airborne = set(),[]
            for old,row in zip(visit,visit[1:]):
                if old['owner'] is not None and row['owner'] == old['owner']:
                    partner = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][partner]['feet'] == old['players'][partner]['feet'], \
                        'Partner moved during school scene'
                if old['owner'] is not None and row['owner'] is None:
                    a,b = row['players']
                    assert a['warped_at'] == b['warped_at'] > 0
                    assert distance(a['feet'],b['feet']) <= 110+max(0,row['tick']-a['warped_at'])*640/120
                    reunions.add(a['warped_at'])
                    owner = 0 if old['owner'] == 'Blue' else 1
                    if not old['players'][owner]['grounded']:
                        airborne.append(dict(owner=old['owner'],tick=a['warped_at'],
                                             original_feet=old['players'][owner]['feet']))
            assert self.reports[role]['visit']['entry'] == 'skool2_start1'
            self.evidence[role] = dict(owners={s:sorted(ids) for s,ids in owners.items()},
                                      reunions=sorted(reunions),airborne_handoffs=airborne)
        self.verify_coherence()

    def run(self):
        self.start_pair()
        self.wait(.5)
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit']
            assert report['frame']['tick'] >= self.saved['frame']['tick']
            for current,old in zip(report['frame']['players'],self.saved['frame']['players']):
                assert distance(current['player']['feet'],old['player']['feet']) < 2
                for field in ('sanity','will','owned','collected','school_items'):
                    assert current['stats'][field] == old['stats'][field], f'Resume changed {field}'
            school = report['combat']['stage']['world']['school']
            old = self.saved['frame']['combat']['stage']['world']['school']
            for field in ('books','returning','theatre','library','shelves','recipe'):
                assert school[field] == old[field], f'Resume changed school {field}'
        self.capture_pair('resumed-checkpoint')
        if self.args.resume_point == 'theatre':
            self.after_theatre()
        elif self.args.resume_point == 'balcony':
            self.balcony_gap()
            self.library_route()
        elif self.args.resume_point == 'spiral':
            self.after_spiral_lift()
        elif self.args.resume_point == 'library':
            self.upper_library_route()
        elif self.args.resume_point == 'bridge':
            self.book_bridge()
        else:
            self.descent()
        assert self.arrived()
        self.verify_segment()
        self.finish_pair(2)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,build=self.build,
            internet=not self.args.local,first=self.args.first,delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms,evidence=self.evidence,
            scope='Resumed school segment only; not a complete entrance-to-exit traversal'),indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=ROOT/'target/release/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--checkpoint',type=Path,required=True)
    parser.add_argument('--resume-point',choices=('theatre','balcony','spiral','library','bridge','descent'),required=True)
    parser.add_argument('--first',choices=('blue','red'),default='red')
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--rejoin',action='store_true')
    parser.add_argument('--checkpoint-stops',nargs='*',choices=('theatre','balcony','spiral','library','bridge','descent'),default=[])
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = SchoolResume(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
