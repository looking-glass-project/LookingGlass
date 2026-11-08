"""Verify return-school guard targeting from a completed recovery recording.

The host owns AI selection; that transient choice is not serialized to guests.
Check the actual host decisions while both living players are visible, plus
damage to each nearer player with the farther partner's health unchanged.
The source run must also have passed its sent/restored player/world comparisons.
This reads evidence only and does not claim full traversal or personal pickups.
"""
import argparse
import json
from pathlib import Path


def targeting(history):
    rows = []
    seen = set()
    for row in history['host']:
        key = (row['epoch'], row['tick'])
        if key not in seen:
            rows.append(row)
            seen.add(key)
    counts = [0, 0]
    hits = {}
    for before, after in zip(rows, rows[1:]):
        if before['owner'] is not None or any(p['health'] <= 0 for p in before['players']):
            continue
        for guard in before['driver']['guards']:
            if not (guard['active'] and guard['health'] > 0 and
                    guard['state'] in ('Chase', 'Alert', 'Attack') and all(guard['line_of_sight'])):
                continue
            nearer = min(range(2), key=lambda i: guard['distance'][i])
            if guard['distance'][nearer] + 30 >= guard['distance'][1 - nearer]:
                continue
            assert guard['selected_player'] == nearer, 'Guard did not select the nearer visible living player'
            counts[nearer] += 1
            if before['epoch'] != after['epoch'] or not 0 < after['tick'] - before['tick'] <= 60:
                continue
            old = [p['health'] for p in before['players']]
            now = [p['health'] for p in after['players']]
            if now[nearer] < old[nearer] and now[1 - nearer] == old[1 - nearer]:
                hits.setdefault(nearer, dict(tick=before['tick'], next_tick=after['tick'],
                    guard=guard['id'], distance=guard['distance'], before_health=old,
                    after_health=now, both_players_visible=True))
    assert all(counts), 'Both nearer-player roles were not observed'
    assert set(hits) == {0, 1}, 'Both nearer players need observed damage with the farther partner unchanged'
    return dict(host_decision_samples=counts, damage_examples=hits)


def verify(folder):
    result = json.loads((folder / 'result.json').read_text())
    assert result['passed'], 'Source run did not pass'
    saved = json.loads((folder / 'starting-checkpoint.json').read_text())
    assert saved['visit']['map'] == 'skool1' and saved['visit']['entry'] == 'skool1_start2'
    for field in ('matching_sent_and_restored_states', 'matching_shared_world_states'):
        assert result['evidence'][field] > 0, 'Source lacks matching player/world state evidence'
    history = json.loads((folder / 'history.json').read_text())
    return dict(build=result['build'], source=str(folder), targeting=targeting(history),
                scope='Return-school host guard selection and damage in both roles; no full route or pickup claim')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('folder', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.folder), indent=2))
