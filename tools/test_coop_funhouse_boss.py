"""Fight the remaining Tweedles normally, watch the ending and verify Hatter1 arrival.

Resume an unchanged normal arena checkpoint. This is a bounded combat/exit test,
not a claim that the earlier Funhouse route or every repeated combat marker ran.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_funhouse import FunhousePair
from test_coop_play import ROOT, distance
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from coop_recovery import read_trace, verify_views, verify_presence


SCENE = 'Funhouse_Hatter_Cinema1'


def verify_boss_exit(history, skipped=False):
    """Require observed defeat, original solo scene/tail and actual safe next visit."""
    owners, endings = [], []
    for peer in ('host', 'guest'):
        rows = history[peer]
        arena = [r for r in rows if r['map'] == 'funhouse' and r['epoch'] == 1]
        combat = [r for r in arena if r['funhouse'].get('fight') and len(r['funhouse']['bosses']) == 2]
        assert combat and any(b['health'] > 0 for b in combat[0]['funhouse']['bosses']), 'No live starting boss'
        defeated = [r for r in combat if all(b['health'] <= 0 for b in r['funhouse']['bosses'])]
        assert defeated, 'Both bosses must actually die'
        active = [r for r in arena if r['scene'] == SCENE and r['owner'] is not None]
        assert active and len({r['owner'] for r in active}) == 1, 'Missing or changing ending actor'
        owner = active[0]['owner']
        assert owner in ('Blue', 'Red')
        owners.append(owner)
        partner = 1 if owner == 'Blue' else 0
        before = active[0]
        assert defeated[0]['tick'] <= before['tick'], 'Scene preceded boss defeat'
        for row in active:
            assert len(row['funhouse']['bosses']) == 2 and all(b['health'] <= 0 for b in row['funhouse']['bosses']), 'Missing or living boss during ending'
            assert not row['presentation']['visible_players'][partner], 'Ending displayed partner'
            assert row['players'][partner]['feet'] == before['players'][partner]['feet'], 'Ending moved partner'
            for old, body in zip(before['players'], row['players']):
                assert body['health'] == old['health'] > 0 and body['spawned_at'] == old['spawned_at'], 'Ending changed life/health'
                assert body['stats']['owned'] == old['stats']['owned'] and body['collected'] == old['collected'], 'Ending changed personal inventory'
        if not skipped:
            assert {r['funhouse']['scene']['clock']['line'] for r in active} == set(range(8)), 'Missing watched ending lines'
        tail = [r for r in active if r['funhouse']['scene']['ending'] is not None]
        assert tail, 'Missing original physical ending'
        duration = lambda r: r['funhouse']['scene']['clock']['time'] - r['funhouse']['scene']['ending']
        assert any(duration(r) < 2.4 and r['funhouse']['floor'] is None for r in tail), 'Missing pre-collapse tail'
        assert any(duration(r) >= 6 and r['funhouse']['floor'] is not None for r in tail), 'Ending cut its physical tail short'
        for row in tail:
            if duration(row) >= 2.6:
                assert row['funhouse']['floor'] is not None, 'Floor never collapsed'
            elif duration(row) < 2.4:
                assert row['funhouse']['floor'] is None, 'Floor collapsed too early'
        arrival = [r for r in rows if r['map'] == 'hatter1' and r['epoch'] == 2]
        assert arrival, 'No actual Hatter1 transition'
        end_tick = tail[0]['tick'] - round(duration(tail[0])*120)
        assert arrival[0]['tick'] - end_tick >= 779, 'Exit skipped the 6.5-second physical tail'
        after = next((r for r in arrival if r['owner'] is None and all(p['grounded'] for p in r['players'])), None)
        assert after is not None and all(after['driver']['clear']), 'Missing supported clear destination'
        a, b = after['players']
        assert all(p['health'] > 0 for p in after['players']), 'Dead destination player'
        assert distance(a['feet'][:2], b['feet'][:2]) >= 35.95 or abs(a['feet'][2]-b['feet'][2]) >= 55.95, 'Overlapping destination players'
        assert distance(a['feet'], b['feet']) <= 160, 'Destination separated partners'
        assert after['presentation']['visible_players'][1 if peer == 'host' else 0], 'Destination partner hidden'
        for old, body in zip(before['players'], after['players']):
            assert body['stats']['owned'] == old['stats']['owned'] and body['collected'] == old['collected'], 'Transition lost personal inventory'
        verify_presence(rows, peer)
        endings.append(dict(peer=peer, owner=owner, defeated_tick=defeated[0]['tick'],
            scene_tick=before['tick'], tail_ticks=arrival[0]['tick']-end_tick,
            arrival_tick=after['tick'], health=[p['health'] for p in after['players']]))
    assert len(set(owners)) == 1, 'Clients disagree on ending owner'
    return endings


class BossPair(FunhousePair):
    evidence_prefix = 'funhouse-boss-'

    def __init__(self, args):
        super().__init__(args)
        raw = Path(__file__).read_bytes()
        (self.folder / 'fixture-dependencies' / Path(__file__).name).write_bytes(raw)
        self.evidence['fixture_sha256'][Path(__file__).name] = hashlib.sha256(raw).hexdigest()
        self.evidence['combat_inputs'] = []

    def state(self, role):
        report = self.reports[role]
        return report['combat']['stage']['world']['levels'].get('funhouse', {}) if report['visit']['map'] == 'funhouse' else {}

    def fighting(self):
        return all(r['visit']['map'] == 'funhouse' and r['combat']['stage']['owner'] is None
                   for r in self.reports.values())

    def fight(self):
        # These authored floor points are ordinary movement/attack inputs. A
        # boss defeat may interrupt any leg; no uncompleted arrival is credited.
        cursors = dict(host=156, guest=158)
        for role in self.commands:
            self.set(role, attack=True)
        end = time.monotonic() + 180
        while self.fighting():
            assert time.monotonic() < end, 'Boss fight timed out'
            for role in self.commands:
                state = self.state(role)
                if all(b['health'] <= 0 for b in state['bosses']):
                    if self.commands[role]['attack']:
                        self.set(role, attack=False)
                    continue
                if self.body(role)['stats']['sanity'] <= 0:
                    continue
                if self.commands[role]['goal'] is None or self.reached(role):
                    self.step = cursors[role]
                    self.set(role, self.steps[self.step]['pos'], run=True, attack=True)
                    self.evidence['combat_inputs'].append(dict(role=role, step=self.step,
                        tick=self.reports[role]['frame']['tick'], command=dict(self.commands[role])))
                    cursors[role] = 156 + (cursors[role] - 155) % 12
            self.pump()
        assert all(r['visit']['map'] == 'funhouse' for r in self.reports.values()), 'Unobserved ending scene'
        for role in self.commands:
            self.set(role, attack=False)
        self.until(lambda: all(r['route_driver']['scene'] == SCENE for r in self.reports.values()), 20)
        self.retain('bosses-defeated')
        print('PASS both Tweedles defeated; watching original ending', flush=True)

    def run(self):
        self.start_pair()
        receipt = json.loads((self.folder / 'host' / 'route-resume.json').read_text())
        verify_resume_receipt(receipt, self.saved, self.build)
        self.evidence['exact_resume_receipt'] = receipt
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit']
            elapsed = report['frame']['tick'] - self.saved['frame']['tick']
            for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                verify_resume_body(body, old, elapsed)
        self.fight()
        self.finish_ending()

    def finish_ending(self, full_route=False):
        if full_route:
            assert self.saved is None and self.args.start_step == 0
            assert self.evidence.get('verified_pre_arena_steps') == list(range(139)), 'Missing full personal pre-arena traversal'
        self.until(self.arrived, 180)
        for role in self.commands:
            self.set(role, epoch=2, attack=False)
        self.until(lambda: all(r['combat']['stage']['owner'] is None and all(b['player']['grounded']
            for b in r['frame']['players']) for r in self.reports.values()), 20)
        self.evidence['ending'] = verify_boss_exit(self.history, self.args.skip_scenes)
        for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'),
                              ('native_sha256', 'matching_native_residents')]:
            self.verify_network_states(field, metric)
        self.retain('hatter1-arrival')
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=2, attack=False)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        recovery = read_trace(self.folder / 'host/recovery-trace.jsonl', self.build)
        recovery['rendered_revivals'] = verify_views(recovery['revivals'], self.history)
        self.evidence['recovery'] = recovery
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route=full_route,
            through_exit=True, build=self.build, internet=not self.args.local, skip_scenes=self.args.skip_scenes,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            scope=('Fresh Funhouse traversal, combat, original ending and Hatter1 arrival' if full_route else
                   'Resumed Funhouse arena combat, original ending and actual Hatter1 arrival'),
            evidence=self.evidence), indent=2))
        print('PASS fresh full Funhouse route through Hatter1' if full_route else
              'PASS Funhouse boss ending and paired Hatter1 arrival; full route remains open', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--skip-scenes', action='store_true')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    args.first, args.start_step, args.last_step = 'blue', 156, 167
    args.save_before, args.allow_revival, args.require_revival = [], True, False
    pair = BossPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error), build=pair.build)))
        raise
    finally:
        pair.close()
