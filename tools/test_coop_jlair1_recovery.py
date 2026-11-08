"""Verify a real lava death and revival from an unchanged Curiosity checkpoint.

Two independent internet clients use only normal walking inputs. The exact host
trace proves the 1,200 active-tick countdown, 25-health revival and unchanged
personal resources; both rendered views must follow the right living player.
This targeted case does not establish a route, team wipe or reconnect pass.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_jlair1 import CuriosityPair, ROLES, verify_arrival
from test_coop_play import distance
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from coop_recovery import read_trace, verify_presence, verify_views


DEATH_GOALS = {'host': [-6240,2800,1792], 'guest': [-6240,2670,1792]}


def verify_lava_revival(events, history, victim, command, starting_feet):
    slot = ROLES.index(victim); survivor = 1-slot
    assert len(events) == 1 and events[0]['player'] == slot, 'Wrong or repeated revival'
    event = events[0]
    assert event['epoch'] == 1 and event['active_ticks'] == 1200 and event['health'] == 25
    assert command['goal'] == DEATH_GOALS[victim] and command['horizontal'] and command['run']
    assert not any(command[k] for k in ('navigate','jump','attack','capture','finish')), 'Death used unexpected controls'
    approach = [r for r in history[victim] if r['tick'] < event['death_tick']
                and r['driver']['command'] == command['id']
                and r['players'][slot]['health'] > 0 and r['players'][slot]['immersion']['kind'] == 'Lava']
    assert approach and any(distance(r['players'][slot]['feet'],starting_feet[victim]) > 48 for r in approach), 'No ordinary walk into lava'
    dead = [r for r in history[victim] if event['death_tick'] <= r['tick'] < event['spawn_tick']]
    assert dead and all(r['players'][slot]['health'] <= 0 and r['players'][slot]['immersion']['kind'] == 'Lava' for r in dead), 'Missing lava death observations'
    for rows in history.values():
        for row in rows:
            body = row['players'][survivor]
            assert body['health'] > 0 and body['spawned_at'] == 0, 'Survivor died or revived'
            assert body['grounded'] and body['immersion']['kind'] == 'Dry', 'Survivor lost safe footing'
            assert distance(body['feet'],starting_feet[ROLES[survivor]]) < 72, 'Survivor left the waiting area'
    return dict(victim=victim, death_goal=DEATH_GOALS[victim], lava_tick=approach[0]['tick'], **event)


def verify_survivor_motion(entry, history, event):
    role = entry['role']; slot = ROLES.index(role)
    assert slot != event['player'], 'Dead player substituted for moving survivor'
    selected = next(r for r in history[role] if r['tick'] == entry['selected_tick'])
    assert entry['goal'] in selected['driver']['neighbors'][slot], 'Unobserved survivor waiting pad'
    arrival = entry['arrival']
    assert arrival['role'] == role and arrival['goal'] == entry['goal']
    assert event['death_tick'] < selected['tick'] < arrival['tick'] < event['spawn_tick'], 'Survivor movement missed spectator interval'
    verify_arrival(arrival, history)
    views = [r for r in history[ROLES[event['player']]] if event['death_tick'] <= r['tick'] < event['spawn_tick']]
    assert views and max(distance(r['camera_subject_feet'],views[0]['camera_subject_feet']) for r in views) > 20, 'Spectator never followed a moving survivor'


class CuriosityRecovery(CuriosityPair):
    evidence_prefix = 'jlair1-recovery-'

    def __init__(self, args):
        self.victim = 'host' if args.victim == 'blue' else 'guest'
        self.survivor = 'guest' if self.victim == 'host' else 'host'
        super().__init__(args)
        self.step = 65
        raw = Path(__file__).read_bytes()
        (self.folder/'fixture-dependencies'/Path(__file__).name).write_bytes(raw)
        self.evidence['fixture_sha256'][Path(__file__).name] = hashlib.sha256(raw).hexdigest()
        self.evidence.update(case='lava revival', victim=args.victim,
            scope='Targeted ordinary lava death and revival; no route, wipe or rejoin credit')

    def validate_lives(self, role, report):
        assert report['frame']['checkpoint_generation'] == 0, 'Targeted revival became a team wipe'
        survivor = report['frame']['players'][ROLES.index(self.survivor)]
        assert survivor['stats']['sanity'] > 0 and survivor['spawned_at'] == 0, 'Survivor died or revived'

    def validate_clear(self, role, report):
        for slot, body in enumerate(report['frame']['players']):
            if body['stats']['sanity'] > 0:
                assert report['route_driver']['clear'][slot], 'Living body is obstructed'

    def run(self):
        for role in ROLES: self.set(role, attack=False)
        self.start_pair()
        receipt = json.loads((self.folder/'host/route-resume.json').read_text())
        verify_resume_receipt(receipt, self.saved, self.build)
        self.evidence['exact_resume_receipt'] = receipt
        for report in self.reports.values():
            for body, old in zip(report['frame']['players'], self.saved['frame']['players']):
                verify_resume_body(body, old, report['frame']['tick']-self.saved['frame']['tick'])
        starting = {role:list(self.body(role)['player']['feet']) for role in ROLES}
        self.evidence['starting_feet'] = starting
        self.set(self.survivor, starting[self.survivor], run=True, radius=2, attack=False)
        self.set(self.victim, DEATH_GOALS[self.victim], horizontal=True, run=True, attack=False)
        self.evidence['death_command'] = dict(self.commands[self.victim])
        slot = ROLES.index(self.victim)
        self.until(lambda: any(r['frame']['players'][slot]['stats']['sanity'] <= 0 for r in self.reports.values()), 45)
        self.set(self.victim, attack=False)
        print('Observed ordinary lava death', self.victim, self.reports['host']['frame']['tick'], flush=True)
        observed = self.reports['host']['frame']['tick']
        self.until(lambda: self.reports['host']['frame']['tick'] >= observed + 120, 5)
        selected = self.history[self.survivor][-1]; survivor_slot = ROLES.index(self.survivor)
        at = selected['players'][survivor_slot]['feet']
        candidates = [p for p in selected['driver']['neighbors'][survivor_slot] if 40 <= distance(p,at) <= 64]
        assert candidates, 'No observed safe survivor walk during countdown'
        goal = min(candidates,key=lambda p:abs(p[2]-at[2]))
        self.set(self.survivor, goal, radius=2, attack=False)
        self.until(lambda: self.reached(self.survivor) and self.body(self.survivor)['player']['grounded'], 6)
        self.evidence['survivor_motion'] = dict(role=self.survivor, selected_tick=selected['tick'],
            goal=goal, arrival=self.receipt(self.survivor,goal))
        self.until(lambda: all(r['frame']['players'][slot]['spawned_at'] > 0
            and r['frame']['players'][slot]['stats']['sanity'] > 0 for r in self.reports.values()), 45)
        spawn_tick = self.body(self.victim)['spawned_at']
        self.until(lambda: self.reports['host']['frame']['tick'] >= spawn_tick + 120, 15)
        assert all(self.body(r)['player']['grounded'] and self.body(r)['player']['immersion']['kind'] == 'Dry' for r in ROLES), 'Revival did not settle safely'
        self.checkpoint(65)
        for role in ('guest','host'):
            self.set(role, finish=True, attack=False)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        recovery = read_trace(self.folder/'host/recovery-trace.jsonl', self.build, require_revival=True)
        self.evidence['lava_revival'] = verify_lava_revival(recovery['revivals'], self.history,
            self.victim, self.evidence['death_command'], starting)
        verify_survivor_motion(self.evidence['survivor_motion'],self.history,recovery['revivals'][0])
        recovery['rendered_revivals'] = verify_views(recovery['revivals'], self.history)
        for role, rows in self.history.items(): verify_presence(rows, role)
        for field, metric in [('sha256','matching_player_states'),('world_sha256','matching_world_states'),('native_sha256','matching_native_states')]:
            self.verify_network_states(field, metric)
        self.evidence['recovery'] = recovery
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True, full_route=False,
            internet=not self.args.local, build=self.build, case='lava revival', victim=self.args.victim,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence),indent=2))
        print('PASS Burning Curiosity ordinary lava revival', self.args.victim, flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--victim', choices=('blue','red'), required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not all(0 <= v <= 500 for v in (args.delay_ms,args.jitter_ms)): parser.error('Delay/jitter must be 0..500 ms')
    args.first = 'blue'
    pair = CuriosityRecovery(args)
    try: pair.run()
    except BaseException as error:
        pair.evidence.update(passed=False,error=str(error),process_exit_codes_before_cleanup={r:p.poll() for r,p in pair.processes.items()})
        raise
    finally: pair.close()


if __name__ == '__main__': main()
