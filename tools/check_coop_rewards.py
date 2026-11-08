"""Verify personal consumable healing from both clients' recorded real play.

Requires separated collection, healing for each collector, no pickup-driven
partner resource change, and the uncollected partner's reward in its own ledger.
Only ordinary difficulty-dependent Will regeneration may change the partner's
resources during the measured interval; healing or an extra refill still fails.
Explicit --map-pickup IDs can also prove separate collection of placed health items.
This checks recorded gameplay; it never supplies or changes a game state.
"""
import argparse
import json
import math
from pathlib import Path


def personal_rewards(history, visit, map_pickups=()):
    evidence = {}
    prefix = 'drop:' + visit + ':'
    for role in ('host', 'guest'):
        events = {}
        rows = history[role]
        for before, after in zip(rows, rows[1:]):
            if before['epoch'] != after['epoch'] or not 0 < after['tick'] - before['tick'] <= 60:
                continue
            for index in range(2):
                previous = before['players'][index]['stats']
                current = after['players'][index]['stats']
                other_before = before['players'][1-index]['stats']
                other_after = after['players'][1-index]['stats']
                for reward in set(current['collected']) - set(previous['collected']):
                    drop = reward.startswith(prefix)
                    if not (drop or reward in map_pickups) or current['sanity'] <= previous['sanity']:
                        continue
                    if any(other_before[f] != other_after[f] for f in ('sanity', 'owned', 'collected', 'difficulty')):
                        continue
                    old_will, new_will = other_before['will'], other_after['will']
                    if not all(math.isfinite(w) for w in (old_will, new_will)):
                        continue
                    if new_will != old_will:
                        # Stats::update / Difficulty::regen run on simulation
                        # time even while the partner waits. Check that exact
                        # allowance, not a general tolerance for resource drift.
                        rate, cap = {'Easy': (.3, 100), 'Normal': (.1, 10),
                                     'Hard': (0, 0), 'Nightmare': (0, 0)}[other_before['difficulty']]
                        expected = (min(cap, old_will + (after['tick'] - before['tick']) / 120 * rate)
                                    if other_before['sanity'] > 0 and old_will < cap else old_will)
                        if (expected == old_will or new_will < old_will or new_will > cap
                                or abs(new_will - expected) > .002):
                            continue  # Fixed-step f32 rounding, never a shot/refill.
                    already = reward in other_before['collected']
                    if drop and not already and not any(d['id'] == int(reward[len(prefix):])
                            for d in other_after['loot'].get(visit, {}).get('drops', [])):
                        continue
                    events.setdefault(reward, {})[index] = {'tick': after['tick'],
                        'before_health': previous['sanity'], 'after_health': current['sanity'],
                        'partner_already_collected': already, 'partner_resources_unchanged': True,
                        'partner_passive_will_gain': new_will - old_will,
                        'partner_copy_retained': not already, 'source': 'enemy' if drop else 'map'}
        pairs = {reward: players for reward, players in events.items()
                 if len(players) == 2 and players[0]['tick'] != players[1]['tick']
                 and {p['partner_already_collected'] for p in players.values()} == {False, True}}
        assert pairs, f'{role}: no independently collected and healing reward pair for {visit}'
        evidence[role] = pairs
    shared = set(evidence['host']) & set(evidence['guest'])
    assert shared, 'Clients did not observe the same personal reward pair'
    return {'visit': visit, 'shared_reward_ids': sorted(shared), 'observations': evidence}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('folder', type=Path)
    parser.add_argument('--visit', required=True)
    parser.add_argument('--map-pickup', action='append', default=[])
    args = parser.parse_args()
    result = json.loads((args.folder / 'result.json').read_text())
    assert result['passed'], 'Source run did not pass'
    history = json.loads((args.folder / 'history.json').read_text())
    print(json.dumps({'build': result['build'], 'source': str(args.folder),
                      'personal_rewards': personal_rewards(history, args.visit, args.map_pickup)}, indent=2))
