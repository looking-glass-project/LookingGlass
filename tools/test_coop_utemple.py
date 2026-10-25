"""Swim the temple through two independent clients using ordinary network inputs.

Public internet with delay/jitter is the default. The clients independently use
the verified native swimmer on their own received worlds. No state or resource
writes; recovery and rejoin are separate requirements from this clean route.
"""
import argparse
import hashlib
import json
import time
from pathlib import Path

from test_coop_play import ROOT
from test_coop_village import Pair


class TemplePair(Pair):
    map_name = 'utemple'
    next_map = 'garden1'
    evidence_prefix = 'temple-route-'

    def __init__(self, args):
        super().__init__(args)
        self.deadline = time.monotonic() + 900
        source = Path(__file__).read_bytes()
        (self.folder / 'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()
        self.last_node = -1
        checkpoint = getattr(args, 'checkpoint', None)
        if checkpoint:
            saved = checkpoint.read_bytes()
            payload = json.loads(saved)['payload']
            assert payload['visit']['map'] == self.map_name and payload['visit']['epoch'] == 1
            assert payload['frame']['combat']['stage']['world']['levels']['utemple']['open']
            (self.folder / 'host' / 'coop-checkpoint.json').write_bytes(saved)
            self.resume_checkpoint = True
            self.evidence['checkpoint_sha256'] = hashlib.sha256(saved).hexdigest()

    def set(self, role, *args, follow=False, **kwargs):
        kwargs['attack'] = False
        super().set(role, *args, **kwargs)
        self.commands[role].update(
            temple_follow=self.args.first.title() if follow else None,
            input_lead=min(1.5, .05 + 2 * (self.args.delay_ms + self.args.jitter_ms) / 1000)
                if role == 'guest' else 0,
            watch_dialogue=True)

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Temple route stopped; evidence retained')
        super().pump()
        for role, report in self.reports.items():
            row = self.history[role][-1]
            temple = report['combat']['stage']['world']['levels'].get('utemple')
            row['temple'] = {key: temple[key] for key in ('clock', 'node', 'guide', 'open', 'scene')} if temple else None
            row['breath'] = [body['player']['breath'] for body in report['frame']['players']]
        if 'host' in self.reports:
            row = self.history['host'][-1]
            temple = row['temple']
            if temple and temple['node'] // 20 > self.last_node:
                self.last_node = temple['node'] // 20
                print('Temple node', temple['node'], 'health',
                      [p['health'] for p in row['players']], 'tick', row['tick'], flush=True)

    def until(self, predicate, seconds=240):
        deadline = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= deadline:
                raise AssertionError('Temple timeout: ' + json.dumps({role: {
                    'tick': report['frame']['tick'], 'map': report['visit']['map'],
                    'phase': report['route_driver']['phase'], 'expected': report['route_driver']['expected'],
                    'feet': [b['player']['feet'] for b in report['frame']['players']],
                    'health': [b['stats']['sanity'] for b in report['frame']['players']]
                } for role, report in self.reports.items()}))
            self.pump()

    def verify(self):
        for role, rows in self.history.items():
            temple_rows = [row for row in rows if row['map'] == self.map_name and row['epoch'] == 1]
            assert temple_rows and max(row['temple']['node'] for row in temple_rows) == 160
            if not getattr(self.args, 'checkpoint', None):
                assert temple_rows[0]['temple']['node'] == 0, 'Full route did not start at the entrance'
            owners = {row['owner'] for row in temple_rows if row['owner'] is not None}
            assert owners == {self.args.first.title()}, f'{role}: wrong exit owner {owners}'
            assert any(row['temple']['scene'] is not None for row in temple_rows)
            for row in temple_rows:
                assert all(b['hits'] == 0 for b in row['breath']), f'{role}: a swimmer drowned'
                if row['temple']['guide'] is not None:
                    assert all(p['stats']['turtle_air'] for p in row['players'])
                if row['owner'] is None:
                    assert all(p['warped_at'] == 0 for p in row['players']), 'Traversal silently teleported'
            for old, row in zip(temple_rows, temple_rows[1:]):
                if row['owner'] is not None and row['owner'] == old['owner']:
                    partner = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][partner]['feet'] == old['players'][partner]['feet'], 'Scene moved waiting partner'
            assert all(p['spawned_at'] == 0 for row in rows for p in row['players']), 'Traversal silently respawned'
            assert any(row['map'] == self.next_map and row['epoch'] == 2 for row in rows)
            final = self.reports[role]['frame']['players']
            assert all(b['stats']['turtle_air'] and b['stats']['sanity'] > 0 for b in final)
            self.evidence[role] = dict(owners=sorted(owners), final_health=[b['stats']['sanity'] for b in final],
                minimum_air=[20 - max(row['breath'][i]['submerged'] for row in temple_rows) for i in range(2)])
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')

    def verify_rejoined_world(self, before, report):
        old = before['combat']['stage']['world']['levels']['utemple']
        current = report['combat']['stage']['world']['levels']['utemple']
        elapsed = (report['frame']['tick'] - before['frame']['tick']) / 120
        assert old['open'] and current['open'], 'Rejoin closed the authored exit'
        assert 157 <= old['node'] <= current['node'] <= 160
        for field in ('guide', 'initialized', 'scene'):
            assert current[field] == old[field], f'Rejoin changed temple {field}'
        assert current['scene'] is None, 'Rejoin triggered the exit scene'
        assert abs(current['clock'] - old['clock'] - elapsed) < .001
        for event, fired in old['events'].items():
            assert current['events'].get(event) == fired, 'Rejoin reset an authored event'
        for name, age in old['movers'].items():
            assert abs(current['movers'][name] - min(60, age + elapsed)) < .01, \
                f'Rejoin changed mover {name} beyond normal elapsed time'
        for body, prior in zip(report['frame']['players'], before['frame']['players']):
            assert body['stats']['turtle_air'] == prior['stats']['turtle_air']

    def clear_exit_enemy(self):
        # The final WaterSnark can follow the swimmers to the exit. Defeat it
        # with ordinary attacks before asserting that idle rejoin changes no
        # health or position; an enemy bite is legitimate damage and knockback.
        def fish(report):
            actors = report['combat']['npcs']['actors']
            return next(a['resident']['body']['Snark'] for a in actors
                if a['spawn']['model'] == 'c_snark'
                and a['spawn']['origin'] == [-1648, 3840, 264])
        for role in self.commands:
            self.set(role)
            self.commands[role]['attack'] = True
        self.until(lambda: all(fish(report)['health'] <= 0 for report in self.reports.values()), 15)
        self.evidence['exit_enemy_defeated'] = True
        print('PASS temple exit enemy defeated through ordinary attacks in both clients.', flush=True)

    def run(self):
        self.start_pair()
        for role in self.commands:
            self.set(role, follow=True)
        if getattr(self.args, 'rejoin', False):
            # Stop through normal input as the turtle opens the last wall,
            # before either Alice reaches the exit scene trigger.
            self.until(lambda: self.reports['host']['combat']['stage']['world']['levels']['utemple']['open'], 480)
            self.clear_exit_enemy()
            self.rejoin_guest()
            self.evidence['rejoin']['near_exit'] = True
            for role in self.commands:
                self.set(role, follow=True)
        self.until(self.arrived, 480)
        self.verify()
        # Finish through the normal client command; automatic captures are not
        # needed for this state-assertion run.
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=2)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, first=self.args.first, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope=('Checkpoint-based temple exit segment; not a full traversal' if getattr(self.args, 'checkpoint', None)
                else 'Full two-client temple traversal') +
                (' with a fresh guest rejoin before the outgoing transition' if getattr(self.args, 'rejoin', False) else '') +
                '; death/wipe remain separate'), indent=2))
        print('PASS ' + ('local' if self.args.local else 'internet') + ' temple' +
            (' exit segment' if getattr(self.args, 'checkpoint', None) else ' full traversal') +
            ': original exit, personal air, matching residents/world and garden arrival.', flush=True)

    def close(self):
        planning = {}
        for role, rows in self.history.items():
            samples = {row['driver']['swim_planning']['requested_tick']: row['driver']['swim_planning']
                for row in rows if row['driver'].get('swim_planning')}
            def timing(field):
                values = sorted(sample[field] for sample in samples.values())
                return dict(median=values[len(values)//2], p95=values[95*(len(values)-1)//100],
                            maximum=values[-1]) if values else None
            planning[role] = dict(searches=len(samples),
                accepted=sum(sample['accepted'] for sample in samples.values()),
                prepare_ms=timing('prepare_ms'), search_ms=timing('search_ms'))
        self.evidence['swim_planning'] = planning
        super().close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true', help='Restart the guest beside the opened exit before completing the route')
    parser.add_argument('--checkpoint', type=Path, help='Use an existing temple exit checkpoint for an isolated rejoin/exit test; does not verify full traversal')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if args.checkpoint and not args.rejoin:
        parser.error('--checkpoint requires --rejoin')
    pair = TemplePair(args)
    try:
        pair.run()
    except Exception as error:
        # Retain the exact failing observations before live report files advance
        # during process cleanup. Original data stays in the ignored test folder.
        (pair.folder / 'failure-report.json').write_text(json.dumps(dict(
            passed=False, error=str(error), build=pair.build, reports=pair.reports), indent=2))
        raise
    finally:
        pair.close()
