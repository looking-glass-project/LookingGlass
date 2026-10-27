"""Leave and rejoin Pale Realm after its puzzles, then traverse the normal exit.

Starts from an unchanged normal checkpoint. The guest exits gracefully and a new
process rejoins the same host using its invite. This does not simulate packet
loss or supply full entrance-to-exit traversal evidence.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_chess_resume import ChessResume
from test_coop_play import ROOT


def verify_chess_rejoin(before, report):
    old_stage, stage = before['combat']['stage'], report['combat']['stage']
    old = old_stage['world']['levels']['wchess1']
    new = stage['world']['levels']['wchess1']
    assert old['bishop_done'] and old['knight_done'] and old['water'] is not None, 'Rejoin must follow both puzzles and water release'
    assert old_stage['owner'] is None and stage['owner'] is None, 'Rejoin entered a scene'
    for state in (old, new):
        assert state['board'] is None and state['pending'] is None and state['scene'] is None
        assert not (state['coop'] or {}).get('reunion'), 'Rejoin entered a puzzle handoff'
    for field in ('intro', 'bishop_done', 'knight_done', 'knight_gate', 'knight_opened',
                  'bell', 'spikes', 'spikes_off', 'water', 'secret', 'lever_started'):
        assert new[field] == old[field], 'Rejoin changed Pale Realm progression: ' + field
    assert set(old['fired']) <= set(new['fired']), 'Rejoin forgot fired level events'
    assert stage['story']['completed'] >= old_stage['story']['completed'], 'Rejoin reset the completed-scene count'
    assert set(old_stage['story']['seen']) <= set(stage['story']['seen']), 'Rejoin forgot seen scenes'
    # Animation, world age and idle enemy clocks can advance normally. Identity
    # and health cannot reset while both players stand idle after the fights.
    def residents(state):
        result = {}
        for actor in state['cast']:
            key = (actor['entity'], actor['instance'])
            assert key not in result, 'Duplicate native resident'
            piece = actor['piece']
            result[key] = (actor['model'], actor['white'], actor['counted'],
                           None if piece is None else (piece['kind'], piece['variant'], piece['health']))
        return result
    assert residents(new) == residents(old), 'Rejoin changed native residents or their health'


class ChessRejoin(ChessResume):
    evidence_prefix = 'chess-rejoin-'

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'rejoin-fixture-source.py').write_bytes(source)
        self.evidence['rejoin_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def after_resume(self):
        self.rejoin_guest()

    def verify_rejoined_world(self, before, report):
        verify_chess_rejoin(before, report)

    def verify_network_states(self, field='sha256', metric='matching_sent_and_restored_states'):
        super().verify_network_states(field, metric)
        after = self.evidence['rejoin']['after_tick']
        epoch = self.evidence['rejoin']['epoch']
        states = {role: {(s['epoch'], s['tick']): s[field]
                         for row in rows for s in row['driver']['network_states']
                         if s['epoch'] == epoch and s['tick'] >= after}
                  for role, rows in self.history.items()}
        shared = states['host'].keys() & states['guest'].keys()
        assert len(shared) >= 20, 'Too few matching states after the new guest joined'
        assert all(states['host'][k] == states['guest'][k] for k in shared), 'Rejoined client state differs: ' + field
        self.evidence['rejoin_' + metric] = len(shared)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--start-step', type=int, default=151)
    parser.add_argument('--last-step', type=int, default=160)
    parser.add_argument('--save-before', nargs='*', type=int, default=[])
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if not (0 < args.start_step <= args.last_step < 1000):
        parser.error('Choose an ordered, nonzero course segment')
    pair = ChessRejoin(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, last_valid=pair.last_valid, completed=pair.completed)))
        raise
    finally:
        pair.close()
