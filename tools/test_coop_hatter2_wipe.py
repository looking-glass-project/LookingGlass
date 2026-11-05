"""Verify a fresh About Face combat wipe against its normal shared checkpoint.

Both clients enter the arena and take ordinary enemy damage without attacking.
An opt-in native receipt checks the complete restored state before another tick;
the closed recovery trace and client views prove the same shared spawn boundary.
"""
import argparse
import hashlib
import json
from pathlib import Path

from coop_recovery import verify_presence
from test_coop_checkmate_wipe import verify_wipe_trace
from test_coop_hatter2 import AboutFacePair, ROLES
from test_coop_hatter2_recovery import verify_restart_states
from test_coop_play import distance
from test_coop_village import verify_rejoin_stats


def verify_wipe_receipt(receipt, saved, build, tick):
    assert receipt['passed'] is True and receipt['build'] == build, 'Wrong native restore proof'
    assert receipt['previous_generation'] == 0 and receipt['generation'] == 1, 'Wrong wipe generation'
    assert receipt['tick'] == tick > saved['frame']['tick'], 'Wrong exact wipe tick'
    assert receipt['recovered_backup'] is False, 'Unexpected backup recovery'
    assert receipt['checkpoint'] == saved, 'Wipe restored a different checkpoint'


def verify_restored_view(saved, report, role):
    frame = report['frame']
    assert report['visit'] == saved['visit'] and frame['checkpoint_generation'] == 1
    stamps = [p['spawned_at'] for p in frame['players']]
    assert stamps[0] == stamps[1] > saved['frame']['tick'], 'Players did not respawn together'
    elapsed = frame['tick'] - stamps[0]
    assert 0 <= elapsed <= 120, 'Missed restored client boundary'
    assert frame['lives']['states'] == ['Alive', 'Alive']
    assert report['camera_player'] == ('Blue' if role == 'host' else 'Red'), 'Wipe left spectator camera active'
    for body, old in zip(frame['players'], saved['frame']['players']):
        verify_rejoin_stats(old['stats'], body['stats'], elapsed)
        assert body['warped_at'] == old['warped_at'], 'Wipe changed scene stamp'
        assert distance(body['player']['feet'], old['player']['feet']) <= 2 + 208 * elapsed / 120, 'Wrong checkpoint placement'
    old = saved['frame']['combat']['stage']['world']['levels']['hatter2']
    new = report['combat']['stage']['world']['levels']['hatter2']
    for field in ('phase', 'blade', 'watch', 'checkpoint', 'bridge', 'exit'):
        assert old[field] == new[field], 'Wipe changed saved shared progress: ' + field
    assert old['coop']['items'] == new['coop']['items'], 'Wipe changed saved personal rewards'
    assert old['boss']['health'] == new['boss']['health'], 'Wipe did not restore saved Hatter health'
    return stamps[0]


class AboutFaceWipe(AboutFacePair):
    evidence_prefix = 'hatter2-wipe-'

    def __init__(self, args):
        self.dying, self.restored, self.observed_death = False, {}, set()
        super().__init__(args)
        for name in (Path(__file__).name, 'test_coop_checkmate_wipe.py', 'test_coop_checkmate.py',
                     'test_coop_hatter2_recovery.py'):
            raw = (Path(__file__).parent / name).read_bytes()
            (self.folder / 'fixture-dependencies' / name).write_bytes(raw)
            self.evidence['fixture_sha256'][name] = hashlib.sha256(raw).hexdigest()

    def validate_report(self, role, report):
        if not self.dying:
            return super().validate_report(role, report)
        frame = report['frame']
        assert report['build'] == self.args.build and report['visit'] == self.saved['visit']
        assert frame['checkpoint_generation'] in (0, 1), 'Unexpected second team wipe'
        assert report['route_driver'].get('recovery_trace_error') is None
        assert all(not b['stats']['god'] and not b['stats']['notarget'] for b in frame['players'])
        if frame['checkpoint_generation'] == 0:
            if any(b['stats']['sanity'] <= 0 for b in frame['players']):
                self.observed_death.add(role)
        elif role not in self.restored:
            (self.folder / f'restored-{role}.json').write_text(json.dumps(report))
            self.restored[role] = verify_restored_view(self.saved, report, role)
            for client in ROLES: self.set(client)

    def run(self):
        self.start_pair()
        self.approach()
        self.checkpoint('combat')
        self.saved = json.loads((self.folder / 'checkpoint-combat.json').read_text())['payload']
        self.phase, self.dying = 'ordinary combat team wipe', True
        self.set('host', [5640, 384, 0], run=True)
        self.set('guest', [5688, 384, 0], run=True)
        self.until(lambda: len(self.restored) == 2, 180)
        assert self.observed_death == set(ROLES), 'Both clients must observe the combat death'
        assert self.restored['host'] == self.restored['guest'], 'Clients disagree on the wipe tick'
        self.wait(1.5)
        self.mark('shared-checkpoint-restored')
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        rows = [json.loads(line) for line in (self.folder / 'host/recovery-trace.jsonl').read_text().splitlines()]
        wipe = verify_wipe_trace(rows, self.saved, self.build)
        assert wipe['tick'] == self.restored['host']
        receipt = json.loads((self.folder / f'host/route-wipe-{wipe["tick"]}.json').read_text())
        verify_wipe_receipt(receipt, self.saved, self.build, wipe['tick'])
        self.evidence.update(wipe=wipe, exact_native_restore=True)
        for role, history in self.history.items():
            for generation in (0, 1):
                group = [r for r in history if r['checkpoint_generation'] == generation]
                assert group
                verify_presence(group, role, generation=generation)
        for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'),
                              ('native_sha256', 'matching_native_states')]:
            self.verify_network_states(field, metric)
            self.evidence['restored_' + metric] = verify_restart_states(self.history, wipe['tick'], field)
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route=False,
            build=self.build, internet=not self.args.local, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Fresh ordinary combat team wipe; exact shared checkpoint and both client returns'), indent=2))
        print('PASS About Face real team wipe and exact checkpoint restore', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not all(0 <= v <= 500 for v in (args.delay_ms, args.jitter_ms)): parser.error('Delay/jitter must be 0..500 ms')
    args.first, args.allow_revival, args.stop_after, args.checkpoint = 'blue', True, 'combat', None
    pair = AboutFaceWipe(args)
    try: pair.run()
    except BaseException as error:
        pair.evidence.update(passed=False, failed_phase=pair.phase, error=str(error),
            process_exit_codes_before_cleanup={r:p.poll() for r,p in pair.processes.items()})
        raise
    finally: pair.close()


if __name__ == '__main__': main()
