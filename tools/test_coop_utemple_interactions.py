"""Focused internet WaterSnark targeting and personal placed-pickup checks.

Resume an isolated checkpoint reached through ordinary gameplay near the temple
exit. All movement, attacks and collection use ordinary client inputs. This
checks the exit chamber only and does not establish full-level traversal.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path

from test_coop_play import ROOT
from test_coop_utemple import TemplePair
from test_coop_village import verify_rejoin_stats


class TempleInteractions(TemplePair):
    evidence_prefix = 'temple-interactions-'

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'interactions-fixture-source.py').write_bytes(source)
        self.evidence['interactions_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def set(self, role, *args, swim=False, **kwargs):
        super().set(role, *args, **kwargs)
        self.commands[role]['swim'] = swim

    def enemy(self):
        return next(enemy for enemy in self.reports['host']['route_driver']['snarks'] if enemy['id'] == 3)

    def selected(self, index):
        enemy = self.enemy()
        return enemy['active'] and enemy['health'] > 0 and all(enemy['line_of_sight']) \
            and enemy['selected_player'] == index and enemy['distance'][index] < enemy['distance'][1-index]

    def observe_hit(self, index):
        self.until(lambda: self.selected(index), 10)
        before = copy.deepcopy(self.reports['host'])
        starting = before['frame']['players']
        target = copy.deepcopy(self.enemy())
        self.until(lambda: self.reports['host']['frame']['players'][index]['stats']['sanity']
                   < starting[index]['stats']['sanity'], 5)
        after = self.reports['host']
        bodies = after['frame']['players']
        assert bodies[index]['stats']['sanity'] == starting[index]['stats']['sanity'] - 10
        assert bodies[1-index]['stats']['sanity'] == starting[1-index]['stats']['sanity']
        assert self.enemy()['selected_player'] == index
        name = 'blue' if index == 0 else 'red'
        (self.folder / f'{name}-target-hit.json').write_text(json.dumps(dict(before=before, after=after), indent=2))
        self.evidence[name + '_target'] = dict(target=target, before_tick=before['frame']['tick'],
            hit_tick=after['frame']['tick'], health_before=starting[index]['stats']['sanity'],
            health_after=bodies[index]['stats']['sanity'], partner_health=bodies[1-index]['stats']['sanity'])
        print('PASS nearest visible living target and isolated 10-health bite:', name, flush=True)

    def move(self, role, goal):
        self.set(role, goal, swim=True, run=True)
        self.until(lambda: self.reached(role), 10)
        self.set(role)

    def pickup(self, role):
        index = 0 if role == 'host' else 1
        item = 'utemple:26'
        before = copy.deepcopy(self.reports['host'])
        bodies = before['frame']['players']
        assert item not in bodies[index]['stats']['collected']
        self.set(role, (-1704, 3696, 170), swim=True, run=True)
        self.until(lambda: all(item in report['frame']['players'][index]['stats']['collected']
                              for report in self.reports.values()), 10)
        self.set(role)
        for report in self.reports.values():
            current = report['frame']['players']
            own, partner = current[index], current[1-index]
            assert set(own['stats']['collected']) - set(bodies[index]['stats']['collected']) == {item}
            assert own['stats']['sanity'] > bodies[index]['stats']['sanity']
            verify_rejoin_stats(bodies[1-index]['stats'], partner['stats'],
                               report['frame']['tick'] - before['frame']['tick'])
        (self.folder / f'{role}-pickup.json').write_text(json.dumps(dict(before=before, after=self.reports), indent=2))
        self.evidence[role + '_pickup'] = dict(item=item,
            health_before=bodies[index]['stats']['sanity'],
            health_after=self.reports['host']['frame']['players'][index]['stats']['sanity'])
        print('PASS independent placed metaessence collection:', role, flush=True)

    def run(self):
        self.start_pair()
        self.observe_hit(0)
        # Separate paths let Red approach the same fish as Blue retreats.
        self.set('host', (-1870, 3420, 480), swim=True, run=True)
        self.set('guest', (-1660, 3260, 480), swim=True, run=True)
        self.observe_hit(1)
        self.clear_exit_enemy()
        self.move('host', (-1710, 3610, 300))
        self.pickup('host')
        self.move('host', (-1780, 3696, 170))
        self.move('guest', (-1710, 3610, 300))
        self.pickup('guest')
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')
        for role, rows in self.history.items():
            assert all(row['owner'] is None and row['map'] == 'utemple' for row in rows)
            assert all(b['spawned_at'] == 0 and b['warped_at'] == 0 for row in rows for b in row['players'])
            assert all(b['hits'] == 0 for row in rows for b in row['breath'])
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            evidence=self.evidence, scope='Captured-checkpoint exit chamber: both nearest-visible targets, '
            'personal native damage and separate collection of the same placed metaessence; no full traversal'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    args.first, args.rejoin = 'red', True
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = TempleInteractions(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failure-report.json').write_text(json.dumps(dict(
            passed=False, error=str(error), build=pair.build, reports=pair.reports), indent=2))
        raise
    finally:
        pair.close()
