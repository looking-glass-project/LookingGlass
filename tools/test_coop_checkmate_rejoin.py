"""Rejoin Checkmate after its first scene, then play through the King and exit.

Uses an unchanged ordinary checkpoint and a newly started guest process on the
same invite. This proves graceful reconnect, not abrupt packet-loss recovery.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_checkmate import CheckmatePair


def verify_checkmate_rejoin(before, report):
    old_stage, stage = before['combat']['stage'], report['combat']['stage']
    old, new = [s['world']['levels']['rchess1'] for s in (old_stage, stage)]
    assert old['beheaded'], 'Rejoin must follow the original beheading scene'
    assert old_stage['owner'] is None and stage['owner'] is None, 'Rejoin entered a scene'
    for field in ('beheaded', 'ambushed', 'phase', 'checkpoint', 'exit', 'essence'):
        assert old[field] == new[field], 'Rejoin changed Checkmate progression: '+field
    assert old['coop']['essence'] == new['coop']['essence'], 'Rejoin changed personal healing use'
    assert old['boss']['health'] == new['boss']['health'], 'Rejoin reset King health'
    assert stage['story']['completed'] == old_stage['story']['completed'], 'Rejoin replayed a scene'
    assert stage['story']['seen'] == old_stage['story']['seen'], 'Rejoin changed seen scenes'
    def pieces(state):
        result = {entity: (p['kind'], p['variant'], p['health']) for entity, p in state['pieces']}
        assert len(result) == len(state['pieces']), 'Duplicate native Checkmate piece'
        return result
    assert pieces(old) == pieces(new), 'Rejoin reset native pieces or their health'


class CheckmateRejoin(CheckmatePair):
    def __init__(self, args):
        super().__init__(args)
        raw = Path(__file__).read_bytes()
        (self.folder / 'rejoin-fixture-source.py').write_bytes(raw)
        self.evidence['rejoin_fixture_sha256'] = hashlib.sha256(raw).hexdigest()

    def after_resume(self):
        self.rejoin_guest()

    def verify_rejoined_world(self, before, report):
        verify_checkmate_rejoin(before, report)

    def verify_network_states(self, field='sha256', metric='matching_player_states'):
        super().verify_network_states(field, metric)
        event = self.evidence['rejoin']
        states = {role: {(s['epoch'], s['tick']): s[field]
                        for row in rows for s in row['driver']['network_states']
                        if s['epoch'] == event['epoch'] and s['tick'] >= event['after_tick']}
                  for role, rows in self.history.items()}
        shared = states['host'].keys() & states['guest'].keys()
        assert len(shared) >= 20, 'Too few independently matched post-rejoin states'
        assert all(states['host'][key] == states['guest'][key] for key in shared), 'Rejoined client differs: '+field
        self.evidence['rejoin_'+metric] = len(shared)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    args.start_step, args.last_step, args.save_before = 27, 44, [33]
    args.skip_scenes, args.allow_revival, args.require_revival, args.through_exit = False, True, False, True
    pair = CheckmateRejoin(args)
    try:
        pair.run()
    except BaseException as error:
        pair.evidence['error'] = repr(error)
        pair.retain('failed-result')
        raise
    finally:
        pair.close()
