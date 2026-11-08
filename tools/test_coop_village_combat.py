"""Check the authored village guard and personal health pickup over the internet.

Two independent clients walk the ordinary route, take real enemy damage, swap
the nearest visible player, defeat the guard with the Blade, and collect their
own health shards. This is an encounter check, not a complete campaign-visit result.
"""
import argparse
import json
from pathlib import Path

from test_coop_play import ROOT
from test_coop_village import Pair
from check_coop_rewards import personal_rewards


class VillageCombat(Pair):
    evidence_prefix = 'village-combat-'
    guard_name = 'guard_rabbit'
    reward_source = 3000000
    map_pickup = 'gvillage:14'
    pickup_goal = (-2544, 3096, 24)

    def guard(self, role='host'):
        return next(g for g in self.reports[role]['route_driver']['guards'] if g['name'] == self.guard_name)

    def stats(self, role='host'):
        return [b['stats'] for b in self.reports[role]['frame']['players']]

    def capture(self, label):
        for role, old in list(self.commands.items()):
            self.set(role, old['goal'], old['navigate'], capture=True)
        self.until(lambda: all(self.reports[r]['route_driver']['captured'] == self.commands[r]['id']
                               for r in self.commands), 30)
        self.evidence.setdefault('captures', {})[label] = {r: c['id'] for r, c in self.commands.items()}

    def selected(self, index):
        guard = self.guard()
        return (guard['active'] and guard['health'] > 0 and guard['state'] in ('Chase', 'Alert')
                and guard['selected_player'] == index and guard['line_of_sight'][index]
                and guard['distance'][index] + 30 < guard['distance'][1-index])

    def check_target(self, index):
        self.until(lambda: self.selected(index), 25)
        before = [s['sanity'] for s in self.stats()]
        decision = dict(self.guard())
        self.until(lambda: self.stats()[index]['sanity'] < before[index], 25)
        after = [s['sanity'] for s in self.stats()]
        assert after[1-index] == before[1-index], 'Guard damaged the distant partner'
        self.evidence.setdefault('targeting', []).append({'player': index, 'decision': decision,
            'before_health': before, 'after_health': after, 'tick': self.reports['host']['frame']['tick']})
        self.capture('guard-targets-' + ('blue' if index == 0 else 'red'))
        print('PASS guard selected and damaged nearest visible player', index, flush=True)

    def pickups(self):
        visit = getattr(self,'loot_visit',self.map_name + '$first')
        drop = self.map_pickup or f'drop:{visit}:{self.reward_source}'
        initial = self.stats()
        collected = [drop in s['collected'] for s in initial]
        assert not all(collected), 'Both copies collected before the isolation check'
        for index, role in enumerate(('host', 'guest')):
            if collected[index]:
                continue
            before = self.stats()
            if self.map_pickup:
                goal = self.pickup_goal
            else:
                loot = before[index]['loot'][visit]['drops']
                goal = next(d['origin'] for d in loot if d['id'] == self.reward_source)
            self.set(role, goal, navigate=True)
            self.until(lambda: drop in self.stats()[index]['collected'], 35)
            self.set(role)
            after = self.stats()
            assert after[index]['sanity'] > before[index]['sanity'], 'Personal pickup did not heal its collector'
            for field in ('sanity', 'will', 'owned', 'collected'):
                assert after[1-index][field] == before[1-index][field], 'Collection changed the other player: ' + field
            if not collected[1-index] and not self.map_pickup:
                assert any(d['id'] == self.reward_source for d in after[1-index]['loot'][visit]['drops']), 'Other copy disappeared'
            collected[index] = True
            self.evidence.setdefault('pickups', []).append({'player': index, 'pickup': drop,
                'before_health': [s['sanity'] for s in before], 'after_health': [s['sanity'] for s in after],
                'other_copy_previously_collected': collected[1-index], 'other_resources_unchanged': True})
            self.capture('personal-pickup-' + role)
        self.wait(.5)
        assert all(drop in s['collected'] for s in self.stats('guest')), 'Guest did not receive both personal ledgers'
        print('PASS personal', self.map_name, 'consumables: separate entitlement and healing.', flush=True)

    def prepare_pickups(self):
        # The guard's small drop falls into the damaging street. Use the authored
        # health shard on the normal supported route, keeping both observers safe.
        self.go((-2808, 3520, -96), True)
        self.go((-2744, 3324, -104))
        self.go((-2628, 3220, 16), True)

    def verify_coherence(self):
        by_tick = {(r['epoch'], r['tick']): r for r in self.history['host']}
        compared = 0
        for row in self.history['guest']:
            host = by_tick.get((row['epoch'], row['tick']))
            if host:
                assert row['players'] == host['players'], 'Players differ at the same authoritative tick'
                assert row['encounters'] == host['encounters'], 'Enemy state differs at the same authoritative tick'
                compared += 1
        self.verify_network_states()
        self.evidence['matching_enemy_and_player_frames'] = compared

    def encounter(self, near, far, retreat):
        first = 'host' if self.args.first == 'blue' else 'guest'
        second = 'guest' if first == 'host' else 'host'
        index = 0 if first == 'host' else 1
        self.set(first, near, navigate=True)
        self.check_target(index)
        self.set(first, far, navigate=True)
        self.set(second, near, navigate=True)
        self.check_target(1-index)
        # Retreat from the drop before defeating the guard, leaving each copy
        # available for an independently observed collection.
        self.set(second, retreat, navigate=True)
        self.set(first, far, navigate=True, attack=True)
        self.until(lambda: self.guard()['health'] == 0, 25)
        for role in self.commands:
            self.set(role, attack=False)
        self.wait(.5)
        self.capture('guard-defeated')
        self.prepare_pickups()
        self.pickups()
        self.evidence['personal_rewards'] = personal_rewards(self.history, getattr(self,'loot_visit',self.map_name + '$first'),
            [self.map_pickup] if self.map_pickup else [])
        self.verify_coherence()
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps({'passed': True, 'build': self.build,
            'internet': not self.args.local, 'first': self.args.first,
            'delay_ms': self.args.delay_ms, 'jitter_ms': self.args.jitter_ms, 'evidence': self.evidence,
            'scope': self.map_name + ' guard targeting, real damage, personal consumables and synchronized enemy state; no exit/death/rejoin claim'}, indent=2))

    def run(self):
        for role in self.commands:
            self.set(role, attack=False)
        self.start_pair()
        self.route_to_guard()
        self.encounter((-2804, 3392, -104), (-2800, 3872, -96), (-2804, 3700, -104))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = VillageCombat(args)
    try:
        pair.run()
    finally:
        pair.close()
