"""Verify Fortress Boojum targeting and personal rewards in two internet clients.

Uses normal movement and Blade inputs from the original arrival. Both players
take real scream damage, then independently collect an earned healing drop.
Full traversal, rejoining and recovery are separate checks.
"""
import argparse
import json
from pathlib import Path

from test_coop_play import ROOT
from test_coop_fortress import FortressPair
from test_coop_village_combat import VillageCombat
from check_coop_rewards import personal_rewards


class FortressCombat(VillageCombat, FortressPair):
    evidence_prefix = 'fortress-combat-'
    map_pickup = None

    def guard(self, role='host'):
        return next(b for b in self.reports[role]['route_driver']['boojums']
                    if b['id'] == self.reward_source)

    def selected(self, index):
        enemy = self.guard()
        return (enemy['active'] and enemy['health'] > 0 and enemy['state'] in ('Fly', 'Scream')
                and enemy['selected_player'] == index and enemy['line_of_sight'][index]
                and enemy['distance'][index] + 30 < enemy['distance'][1-index])

    def run(self):
        for role in self.commands:
            self.set(role, attack=False)
        self.start_pair()
        self.arrival_and_portal()
        near, far = (-360,-2410,-192), (64,-2660,-192)
        index = 0 if self.first == 'host' else 1
        self.set(self.first, near, navigate=True)
        self.check_target(index)
        self.set(self.first, far, navigate=True)
        self.set(self.partner, near, navigate=True)
        self.check_target(1-index)
        # Fight from the open floor, away from the portal's low lintel, which can
        # block the Blade's hand-height throw even with a clear eye-level sight.
        self.set(self.first, (-400,-2430,-192), navigate=True, attack=True)
        self.set(self.partner, far, navigate=True, attack=False)
        self.until(lambda: all(b['health'] == 0 for b in self.reports['host']['route_driver']['boojums']
                               if b['active']), 60)
        self.wait(1.5)
        for role in self.commands:
            self.set(role, attack=False)
        self.capture('boojums-defeated')
        # The first player may collect a drop on the way to the second kill.
        # Prefer that independently collected award if the partner's copy still
        # exists. A fully healed player cannot collect a second health drop.
        stats = self.stats()
        visit = self.map_name + '$first'
        ledgers = [s['loot'][visit]['drops'] for s in stats]
        available = []
        for identity in {d['id'] for drops in ledgers for d in drops}:
            drop = f'drop:{visit}:{identity}'
            collected = [drop in s['collected'] for s in stats]
            remaining = [next((d for d in drops if d['id'] == identity), None) for drops in ledgers]
            if all(done or (award and s['sanity'] < 100) for done, award, s in zip(collected, remaining, stats)):
                collector = next(i for i, done in enumerate(collected) if not done)
                origin = remaining[collector]['origin']
                feet = self.bodies()[collector]['player']['feet']
                travel = sum((a-b)**2 for a, b in zip(origin, feet))
                available.append((any(collected), -travel, identity))
        assert available, 'No separate healing entitlement can be checked after combat'
        self.reward_source = max(available)[2]
        self.pickups()
        self.evidence['personal_rewards'] = personal_rewards(self.history, self.map_name + '$first', [])
        self.verify_coherence()
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps({'passed': True, 'build': self.build,
            'internet': not self.args.local, 'first': self.args.first,
            'delay_ms': self.args.delay_ms, 'jitter_ms': self.args.jitter_ms, 'evidence': self.evidence,
            'scope': 'Fortress Boojum targeting, real scream damage, personal healing and synchronized enemies; no recovery/exit claim'}, indent=2))
        print('PASS two-process Fortress Boojum targeting and individual consumables.', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue','red'), default='red')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = FortressCombat(args)
    try:
        pair.run()
    finally:
        pair.close()
