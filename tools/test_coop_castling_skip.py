"""Verify owner-only held scene skipping through two independent clients.

Resume a retained pre-scene checkpoint and traverse the scene's ordinary course
leg. The waiting player holds skip first; only the owner's later hold may end it.
No world, player, resource or progression values are changed by this fixture.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_castling_resume import CastlingResume
from test_coop_play import ROOT
from test_coop_village import verify_rejoin_stats


def verify_skip_event(event):
    assert event['owner_request']-event['partner_request'] >= 180, 'Partner-only hold was too short'
    assert set(event['acknowledged']) == {'host', 'guest'}, 'Missing client skip acknowledgement'
    stamps = list(event['acknowledged'].values())
    assert stamps[0] == stamps[1], 'Clients disagree about skip tick'
    assert 78 <= stamps[0]-event['owner_request'] <= 480, 'Skip was early or stalled'
    assert event['reunited'] == ['guest', 'host'], 'Missing safe shared handoff'


class CastlingSkip(CastlingResume):
    evidence_prefix = 'castling-skip-'

    def __init__(self, args):
        self.scene_name = 'cinema_queen_abduction_thread' if args.scene == 'queen' else 'cinema_king_thread'
        self.skip_event, self.scene_bodies = None, None
        self.reunited = set()
        self.resume_steps = {}
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'skip-fixture-source.py').write_bytes(source)
        self.evidence['skip_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def pump(self):
        super().pump()
        if len(self.reports) != 2:
            return
        host = self.reports['host']
        tick = host['frame']['tick']
        owner = self.args.first.title()
        owner_role = 'host' if owner == 'Blue' else 'guest'
        partner_role = 'guest' if owner_role == 'host' else 'host'
        if self.skip_event is None:
            if not all(r['route_driver']['scene'] == self.scene_name for r in self.reports.values()):
                return
            assert all(r['combat']['stage']['owner'] == owner for r in self.reports.values()), 'Wrong scene owner'
            self.scene_bodies = host['frame']['players']
            self.skip_event = dict(scene=self.scene_name, owner=owner, partner_request=tick,
                                   owner_request=None, acknowledged={}, reunited=[])
            # Stop the route's automatic combat controls while inspecting the
            # handoff. Otherwise it can legitimately select/fire a weapon on
            # the first free tick before the next report captures the reunion.
            self.resume_steps = {role: command['castling_step'] for role, command in self.commands.items()}
            for role in self.commands:
                self.set(role)
            self.commands[partner_role]['skip_scene'] = True
            return
        event = self.skip_event
        if event['owner_request'] is None:
            assert host['route_driver']['scene'] == self.scene_name, 'Waiting player skipped scene'
            assert host['combat']['stage']['owner'] == owner, 'Waiting player changed scene owner'
            if tick-event['partner_request'] >= 180:
                event['owner_request'] = tick
                self.commands[owner_role]['skip_scene'] = True
            return
        assert tick-event['owner_request'] <= 600 or len(self.reunited) == 2, 'Scene skip did not reunite players'
        for role, report in self.reports.items():
            stage = report['combat']['stage']
            notice = stage['notices'][0 if owner == 'Blue' else 1]
            if notice and notice['text'] == 'Scene skipped':
                event['acknowledged'][role] = notice['until']-240
            if role in event['acknowledged'] and stage['owner'] is None and role not in self.reunited:
                assert report['route_driver']['scene'] is None, 'Skipped scene still active'
                assert stage['world']['levels']['wchess2'][self.args.scene], 'Skip lost royal progression'
                for old, body in zip(self.scene_bodies, report['frame']['players']):
                    # Personal recovery pauses during cinematic ticks. Only
                    # free simulation ticks after the authoritative skip may
                    # regenerate Will before this report reaches the fixture.
                    verify_rejoin_stats(old['stats'], body['stats'],
                                        report['frame']['tick']-event['acknowledged'][role])
                (self.folder / f'skipped-{role}.json').write_text(json.dumps(report))
                self.reunited.add(role)
        event['reunited'] = sorted(self.reunited)
        if len(self.reunited) == 2 and 'skip' not in self.evidence:
            self.evidence['skip'] = event
            for role, step in self.resume_steps.items():
                self.set(role, step=step)

    def reached(self, role):
        if self.skip_event is not None and len(self.reunited) != 2:
            return False
        return super().reached(role)

    def verify_network_states(self, field='sha256', metric='matching_sent_and_restored_states'):
        assert self.skip_event, 'Requested scene was never observed'
        verify_skip_event(self.skip_event)
        super().verify_network_states(field, metric)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--scene', choices=('queen', 'king'), required=True)
    parser.add_argument('--first', choices=('blue', 'red'), required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    args.start_step = 29 if args.scene == 'queen' else 109
    # The walking arrival tolerance at leg 29 can stop just outside the Queen
    # trigger after a resume. Continue the existing course through its doorway;
    # scene observation and acknowledgement remain mandatory for this check.
    args.last_step = 31 if args.scene == 'queen' else args.start_step
    args.save_before, args.rejoin = [], False
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = CastlingSkip(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error),
            build=pair.build, reports=pair.reports, evidence=pair.evidence, skip=pair.skip_event)))
        raise
    finally:
        pair.close()
