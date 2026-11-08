"""Assertions for an underwater countdown interrupted by a fresh guest process."""
from test_coop_play import verify_countdown_rejoin


def verify_swim_rejoin(before, held, after, victim, old_pid, *, expected_generation=0):
    assert not before['playing'] and not held['playing'], 'Host never waited for its missing partner'
    assert held['pid'] == before['pid'] and held['build'] == before['build']
    assert held['visit'] == before['visit']
    assert held['frame'] == before['frame'] and held['combat'] == before['combat'], 'Disconnected host advanced underwater state'
    assert after['playing'], 'Replacement guest never entered gameplay'
    result = verify_countdown_rejoin(before, after, victim, old_pid,
                                    expected_generation=expected_generation)
    elapsed = after['frame']['tick'] - before['frame']['tick']
    assert elapsed <= 120, 'First rejoined observation is too late to verify swimming state'
    for old, new in zip(before['frame']['players'], after['frame']['players']):
        assert old['player']['swimming'] and new['player']['swimming'], 'Rejoin lost a swimming body'
        assert old['player']['immersion']['level'] == new['player']['immersion']['level'] == 3, 'Rejoin left the underwater volume'
        assert old['stats']['turtle_air'] == new['stats']['turtle_air'], 'Rejoin changed personal air equipment'
        assert old['player']['breath']['shell'] == new['player']['breath']['shell'], 'Rejoin reset air capacity'
    old_dead = before['frame']['players'][victim]['player']
    new_dead = after['frame']['players'][victim]['player']
    assert old_dead['breath']['hits'] > 0, 'Countdown was not caused by drowning'
    assert new_dead == old_dead, 'Rejoin moved or refilled the drowned body before revival'
    result['breath_before'] = [b['player']['breath'] for b in before['frame']['players']]
    result['breath_after'] = [b['player']['breath'] for b in after['frame']['players']]
    return result


def verify_first_frame(host, guest):
    """Match the guest's complete first playing state to that exact host tick.

    Air bubbles may refill a living swimmer on an ordinary resumed tick. Matching
    the recorded full-body hash checks the real host result, rather than allowing
    an arbitrary breath reset or approximating the bubble collision geometry.
    """
    assert host['build'] == guest['build'] and host['visit'] == guest['visit']
    key = guest['frame']['epoch'], guest['frame']['tick']
    rows = []
    for report in (host, guest):
        matches = [s for s in report['route_driver']['network_states'] if (s['epoch'], s['tick']) == key]
        assert len(matches) == 1, 'Missing exact first-frame network evidence'
        rows.append(matches[0])
    for field in ('sha256', 'world_sha256', 'native_sha256'):
        assert rows[0][field] == rows[1][field], f'Rejoined first-frame {field} differs from the host'
    return dict(epoch=key[0], tick=key[1], **{field: rows[0][field]
        for field in ('sha256', 'world_sha256', 'native_sha256')})
