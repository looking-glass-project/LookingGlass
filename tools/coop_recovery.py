"""Validate exact host recovery evidence and the independently rendered views.

These checks permit individual revival. They deliberately reject team wipes,
missing ticks, bypassed timers and incomplete recovery instead of treating them
as a successful route. Team checkpoint restoration has its own fixtures.
"""
import copy
import json
import math


def distance(a, b):
    return math.sqrt(sum((x-y)**2 for x, y in zip(a, b)))


def state(life):
    if isinstance(life, dict):
        assert set(life) == {'Spectating'}, 'Invalid life state'
        return 'Spectating', life['Spectating']['ticks_left']
    assert life in ('Alive', 'AwaitingSafeSpawn'), 'Invalid life state'
    return life, None


def verify_trace(rows, build, require_revival=False):
    previous, pending, events = None, {}, []
    count = 0
    for row in rows:
        count += 1
        assert row['version'] == 1 and row['serial'] == count, 'Missing/reordered recovery trace row'
        assert row['build'] == build, 'Recovery trace belongs to another build'
        assert row['checkpoint_generation'] == 0, 'Unexpected team wipe; checkpoint recovery needs separate proof'
        epoch, tick = row['epoch'], row['tick']
        if previous:
            assert (epoch, tick) > (previous['epoch'], previous['tick']), 'Recovery clock moved backwards'
            if epoch != previous['epoch']:
                assert not pending, 'Level transition bypassed a pending revival'
            elif pending:
                assert tick == previous['tick'] + 1, 'Missing exact host recovery tick'
            else:
                assert tick-previous['tick'] <= 120, 'Recovery trace lost its live heartbeat'
        for index, body in enumerate(row['players']):
            assert not body['god'] and not body['notarget'], 'Recovery used cheats'
            kind, left = state(body['life'])
            assert (body['health'] > 0) == (kind == 'Alive'), 'Health/life state disagree'
            if kind == 'Spectating':
                assert type(left) is int and 0 < left <= 1200, 'Invalid ten-second countdown'
                if index not in pending:
                    assert left == 1200 and body['stats'] is not None, 'Missing exact death boundary'
                    pending[index] = dict(epoch=epoch, death_tick=tick, active_ticks=0,
                        paused_ticks=0, wait_ticks=0, left=left, stats=body['stats'])
                else:
                    death = pending[index]
                    assert death['left'] is not None, 'Expired countdown restarted'
                    delta = 0 if row['cinematic'] else 1
                    assert left == death['left']-delta, 'Countdown did not follow the host simulation clock'
                    death['active_ticks'] += delta
                    death['paused_ticks'] += 1-delta
                    death['left'] = left
            elif kind == 'AwaitingSafeSpawn':
                assert index in pending, 'Pending spawn has no recorded death'
                death = pending[index]
                if death['left'] is not None:
                    assert death['left'] == 1 and not row['cinematic'], 'Spawn wait began before the timer expired'
                    death['active_ticks'] += 1
                    death['left'] = None
                else:
                    death['wait_ticks'] += 1
                assert death['active_ticks'] == 1200, 'Spawn wait bypassed ten seconds'
            elif index in pending:
                death = pending.pop(index)
                assert not row['cinematic'] and not row['spawn_held'], 'Revived inside a controlled sequence'
                if death['left'] is not None:
                    assert death['left'] == 1, 'Revived before the timer expired'
                    death['active_ticks'] += 1
                else:
                    death['wait_ticks'] += 1
                assert death['active_ticks'] == 1200, 'Revival did not wait ten simulation seconds'
                assert tick-death['death_tick'] == 1200+death['paused_ticks']+death['wait_ticks'], 'Unaccounted revival time'
                assert body['spawned_at'] == tick and body['health'] == 25, 'Wrong exact spawn tick or initial health'
                expected = copy.deepcopy(death['stats'])
                expected['sanity'] = 25
                # Arena advances uncollected world drops even while Alice is dead.
                # Carried items, weapons, will and personal timers stay exact.
                if 'loot' in expected:
                    expected['loot'] = body['stats'].get('loot')
                assert body['stats'] == expected, 'Revival changed personal resources'
                g = body['geometry']
                assert g and g['clear'] and g['safe'] and g['valid'], 'Revival placement is obstructed or hazardous'
                assert g['supported'] or g['swimming'] or g['rope'], 'Revival has no supported placement'
                partner = row['players'][1-index]
                gap = [a-b for a, b in zip(body['feet'], partner['feet'])]
                assert partner['health'] > 0 and distance(body['feet'], partner['feet']) <= 128, 'Revival is not near a living survivor'
                assert distance(gap[:2], [0, 0]) >= 35.95 or abs(gap[2]) >= 55.95, 'Revival overlaps its survivor'
                events.append(dict(player=index, epoch=epoch, death_tick=death['death_tick'],
                    spawn_tick=tick, health=25, active_ticks=1200,
                    paused_ticks=death['paused_ticks'], wait_ticks=death['wait_ticks']))
            elif previous and epoch == previous['epoch']:
                assert body['spawned_at'] == previous['players'][index]['spawned_at'], 'Respawn has no recorded death'
        previous = row
    assert previous is not None and not pending, 'Recovery trace is empty or ends during a death'
    assert events or not require_revival, 'Route did not exercise a revival'
    return dict(revivals=events, trace_rows=count)


def read_trace(path, build, require_revival=False):
    with path.open(encoding='utf-8') as source:
        return verify_trace((json.loads(line) for line in source), build, require_revival)


def verify_presence(rows, role, *, generation=0, portal_warps=()):
    """Portal exceptions must first pass the route's source/destination verifier."""
    local = 0 if role == 'host' else 1
    for row in rows:
        assert row['checkpoint_generation'] == generation, 'Unexpected team wipe'
        if row['owner'] is not None:
            continue
        a, b = row['players']
        if a['health'] > 0 and b['health'] > 0:
            assert distance(a['feet'][:2], b['feet'][:2]) >= 35.95 or abs(a['feet'][2]-b['feet'][2]) >= 55.95, 'Players overlap'
        watched = local if row['players'][local]['health'] > 0 else 1-local
        assert row['camera_player'] == ('Blue', 'Red')[watched], 'Wrong personal/survivor camera'
        subject = row['camera_subject_feet']
        assert distance(row['camera_target'][:2], subject[:2]) < 1, 'Camera missed its rendered subject'
        presentation = row.get('presentation') or {}
        if 'player_feet' in presentation:
            # The guest draws an interpolated body, which can legitimately lag
            # a running authoritative body by more than 24 units. Verify that
            # actual drawn position against its recorded interpolation inputs.
            rendered = presentation['player_feet']
            previous = presentation['previous_players']
            alpha = max(0, min(1, presentation['elapsed'] * 20))
            for i, body in enumerate(row['players']):
                expected = body['feet']
                if previous:
                    old, spawn, warp, alive = previous[i]
                    if spawn == body['spawned_at'] and warp == body['warped_at'] \
                            and alive == (body['health'] > 0) and distance(old, expected) < 128:
                        expected = [a + (b-a)*alpha for a, b in zip(old, expected)]
                assert distance(rendered[i], expected) < .01, 'Rendered body does not match interpolation inputs'
            assert distance(subject, rendered[watched]) < .01, 'Camera missed its rendered player'
        else:
            assert distance(subject, row['players'][watched]['feet']) < 24, 'Camera subject drifted from its player'
    for old, row in zip(rows, rows[1:]):
        if old['epoch'] != row['epoch']:
            continue
        for slot,(a,b) in enumerate(zip(old['players'],row['players'])):
            if a['warped_at']==b['warped_at'] or old['owner'] is not None or row['owner'] is not None:continue
            assert (row['epoch'],slot,b['warped_at']) in portal_warps \
                and old['tick']<b['warped_at']<=row['tick'], 'Unscripted route warp'
        if row['owner'] is not None and row['owner'] == old['owner'] and row['scene'] == old['scene']:
            other = 1 if row['owner'] == 'Blue' else 0
            assert row['players'][other]['feet'] == old['players'][other]['feet'], 'Scene moved waiting partner'


def verify_views(events, history):
    for event in events:
        role = ('host', 'guest')[event['player']]
        rows = [r for r in history[role] if r['epoch'] == event['epoch'] and r['owner'] is None]
        dead = [r for r in rows if event['death_tick'] <= r['tick'] < event['spawn_tick']]
        alive = [r for r in rows if r['tick'] >= event['spawn_tick'] and r['players'][event['player']]['health'] > 0]
        assert dead and alive, 'Missing rendered spectator or returning-player evidence'
        for row in dead + alive:
            verify_presence([row], role)
    return len(events)


def verify_northern_coverage(events, skips):
    for role in ('host', 'guest'):
        actual = [e['step'] for e in events[role]]
        assert actual == sorted(set(actual)), 'Northern waypoints were duplicated or reordered'
        omitted = [step for e in skips if e['role'] == role for step in e['steps']]
        assert sorted(actual + omitted) == list(range(33, 45)), 'Incomplete/duplicate personal northern coverage'
    for skip in skips:
        other = 'guest' if skip['role'] == 'host' else 'host'
        witnessed = {e['step'] for e in events[other]}
        assert skip['steps'] and set(skip['steps']) <= witnessed, 'Revival skipped an untraversed team waypoint'
