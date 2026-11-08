"""Bounded movement evidence for diagnosing rendered runs without images."""
import math
import statistics


TIMINGS = (('network_timing', 'round_trip_ms'), ('network_timing', 'oldest_pending_ms'),
           ('network_timing', 'queued_input_samples'), ('network_timing', 'buffered_control_ms'),
           ('frame_timing', 'frame_ms'), ('frame_timing', 'simulation_debt_ms'))


def timing_summary(history):
    """Observed timing statistics across the whole run, without gameplay rows.

    Reports sample frames periodically, so these are observed peaks, not a
    guarantee that every transient stall was recorded. They never award a pass.
    """
    clients = {}
    for role, rows in history.items():
        metrics = {}
        for group, key in TIMINGS:
            observations = [(row, (row.get(group) or {}).get(key)) for row in rows]
            observations = [(row, value) for row, value in observations
                            if type(value) in (float, int) and math.isfinite(value)]
            if not observations:
                continue
            row, peak = max(observations, key=lambda pair: pair[1])
            values = sorted(value for _, value in observations)
            metrics[key] = dict(samples=len(values), median=statistics.median(values),
                p95=values[math.ceil(len(values) * .95) - 1], last=observations[-1][1],
                peak=dict(epoch=row['epoch'], tick=row['tick'], value=peak))
        clients[role] = metrics
    return dict(format=1, scope='Periodically observed timings; not a pass/fail result', clients=clients)


def recent_movement(history, seconds=10, limit=80):
    """Copy only the current visit's recent controls, bodies and timing.

    Full histories retain world snapshots and hashes. This intentionally small
    companion is safe to print, and does not classify a failure or infer a cause.
    """
    clients = {}
    for role, rows in history.items():
        if not rows:
            continue
        last = rows[-1]
        recent = []
        for row in reversed(rows):
            if row['epoch'] != last['epoch'] or row['tick'] < last['tick'] - seconds * 120:
                break
            driver = row.get('driver') or {}
            recent.append(dict(tick=row['tick'], owner=row.get('owner'), scene=row.get('scene'),
                route={k: driver.get(k) for k in ('command', 'phase', 'jumps', 'replans',
                                                  'plan_remaining', 'expected')},
                players=[{k: body.get(k) for k in ('feet', 'velocity', 'grounded', 'health',
                                                   'jumps', 'warped_at', 'spawned_at')}
                         for body in row['players']],
                network_timing=row.get('network_timing'), frame_timing=row.get('frame_timing')))
            if len(recent) >= limit:
                break
        recent.reverse()
        peaks = {}
        for group, key in TIMINGS:
            values = [(r['tick'], (r.get(group) or {}).get(key)) for r in recent]
            values = [(tick, value) for tick, value in values
                      if isinstance(value, (float, int)) and math.isfinite(value)]
            if values:
                tick, value = max(values, key=lambda pair: pair[1])
                peaks[key] = dict(tick=tick, value=value)
        clients[role] = dict(epoch=last['epoch'], first_tick=recent[0]['tick'],
                             last_tick=recent[-1]['tick'], samples=len(recent), peaks=peaks, rows=recent)
    return dict(format=1, window_seconds=seconds, maximum_samples_per_client=limit, clients=clients)
