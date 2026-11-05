"""Verify a fresh guest inherits a rope survivor and an active death timer."""
from test_coop_play import verify_countdown_rejoin


def verify_rope_rejoin(before, held, after, victim, old_pid):
    assert not before['playing'] and not held['playing']
    assert before['pid'] == held['pid'] and before['build'] == held['build']
    assert before['visit'] == held['visit']
    assert before['frame'] == held['frame'] and before['combat'] == held['combat'], 'Waiting host advanced the rope world'
    assert after['playing'], 'Replacement guest did not resume'
    proof = verify_countdown_rejoin(before, after, victim, old_pid)
    assert proof['after_tick'] - proof['before_tick'] <= 120, 'First rope observation arrived too late'
    old, new = (r['frame']['players'] for r in (before, after))
    assert new[victim]['player'] == old[victim]['player'], 'Rejoin moved the dead body'
    grip = old[1-victim]['player']['rope']
    resumed = new[1-victim]['player']['rope']
    assert grip is not None and resumed is not None, 'Rejoin lost the survivor rope attachment'
    assert resumed['id'] == grip['id'], 'Rejoin attached the survivor to another rope'
    assert abs(resumed['length'] - grip['length']) < .001, 'Rejoin changed the survivor grip length'
    assert resumed['rise'] == grip['rise'] == 0, 'Neutral rope survivor started climbing'
    proof.update(rope_before=grip, rope_after=resumed)
    return proof
