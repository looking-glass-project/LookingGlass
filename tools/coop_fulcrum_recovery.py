"""Require a real moving support at revival and passive carry afterwards."""
from coop_recovery import distance, verify_views
from test_coop_play import verify_countdown_rejoin


def verify_fulcrum_rejoin(before, held, after, victim, old_pid):
    assert not before['playing'] and not held['playing'] and after['playing']
    assert before['pid'] == held['pid'] and before['build'] == held['build']
    assert before['visit'] == held['visit']
    assert before['frame'] == held['frame'] and before['combat'] == held['combat'], 'Disconnected platform world advanced'
    proof = verify_countdown_rejoin(before, after, victim, old_pid)
    assert proof['after_tick'] - proof['before_tick'] <= 120, 'First platform observation arrived too late'
    assert before['frame']['players'][victim]['player'] == after['frame']['players'][victim]['player'], 'Rejoin moved the dead body'
    for report in (before, after):
        body = report['frame']['players'][1-victim]['player']
        assert body['grounded'] and body['rope'] is None and not body['swimming'], 'Rejoin lost the grounded survivor'
    return proof


def verify_fulcrum_recovery(event, history, neutral_command):
    victim, spawn = event['player'], event['spawn_tick']
    survivor = ('guest', 'host')[victim]
    revived = ('host', 'guest')[victim]
    assert event['paused_ticks'] == event['wait_ticks'] == 0, 'Safe moving support missed exact revival'
    rows = [r for r in history[survivor] if r['epoch'] == event['epoch']]
    before = [r for r in rows if spawn-40 <= r['tick'] < spawn]
    after = [r for r in rows if spawn <= r['tick'] <= spawn+40]
    assert before and after, 'Missing platform observations around the spawn tick'
    a, b = before[-1], after[0]
    angles = [r['fulcrum'][0] for r in (a, b)]
    assert all(abs(v) < 44.5 for v in angles) and abs(angles[1]-angles[0]) > .1, 'Platform was stationary at revival'
    for row in (a, b):
        support = row['driver']['surface']
        assert support and support['entity'] == 460 and support['supported'][1-victim], 'Survivor left the actual platform'
    assert all(b['driver']['surface']['supported']), 'Revived player did not join the same platform'
    support_rows = [r for r in rows if spawn+30 <= r['tick'] <= spawn+180]
    assert support_rows and all(r['driver']['surface'] and r['driver']['surface']['entity'] == 460
        and all(r['driver']['surface']['supported']) for r in support_rows), 'Riders left the same platform during carry'
    # The revived player receives neutral input throughout this interval. Its
    # body must move with the real brush while remaining grounded and clear.
    carry = [r for r in history[revived] if spawn+30 <= r['tick'] <= spawn+180]
    assert len(carry) >= 3 and carry[-1]['tick']-carry[0]['tick'] >= 60, 'Missing passive carry interval'
    for row in carry:
        body = row['players'][victim]
        assert row['driver']['command'] == neutral_command and row['driver']['phase'] == 'idle', 'Revived player was driven during carry'
        assert body['spawned_at'] == spawn and body['health'] == 25 and body['grounded'], 'Revived rider lost its body or footing'
        assert row['driver']['clear'][victim] and distance(body['velocity'][:2], [0, 0]) < .1, 'Carry used movement or an obstructed body'
    first, last = carry[0], carry[-1]
    movement = distance(first['players'][victim]['feet'], last['players'][victim]['feet'])
    tilt = abs(last['fulcrum'][0]-first['fulcrum'][0])
    assert tilt > 2 and movement > 5, 'Revived rider did not follow platform motion'
    verify_views([event], history)
    return dict(player=victim, spawn_tick=spawn, spawn_angles=angles,
        carry_ticks=last['tick']-first['tick'], carry_degrees=tilt, carried_distance=movement)
