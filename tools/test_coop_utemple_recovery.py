"""Both real drowning/respawn roles, a saved temple wipe and internet rejoin.

Run in Anode. Only ordinary swim or neutral inputs are used. Each victim leaves
the guide's air trail while the other keeps swimming; no health or position edits.
This is separate from the clean full-route test and produces no screenshots.
"""
import argparse
import copy
import json
from pathlib import Path

from coop_recovery import verify_presence
from test_coop_play import ROOT, distance, read
from test_coop_utemple import TemplePair


def wipe_air_ready(report):
    """Both swimmers must refill naturally before attempting a simultaneous wipe.

    A just-revived player has fresh air while their partner may be almost out.
    Stopping both at that point can produce alternating deaths forever: each
    revives before the other drowns. No body or breath state is changed here.
    """
    return report['playing'] and all(
        body['stats']['sanity'] > 0 and body['player']['swimming']
        and body['player']['immersion']['level'] == 3
        and body['player']['breath']['shell']
        and body['player']['breath']['hits'] == 0
        and 0 <= body['player']['breath']['submerged'] <= 1
        for body in report['frame']['players'])


class TempleRecovery(TemplePair):
    evidence_prefix = 'temple-recovery-'

    def __init__(self, args):
        super().__init__(args)
        self.allowed_deaths = set()
        self.generation = 0
        self.wiping = False
        (self.folder / 'recovery-fixture-source.py').write_bytes(Path(__file__).read_bytes())

    def pump(self):
        super().pump()
        for role, report in self.reports.items():
            # A repeated simulation tick can have a newer rendered position.
            # Keep the inputs belonging to the camera sample already recorded.
            self.history[role][-1].setdefault('presentation', report['presentation'])

    def validate_report(self, role, report):
        frame = report['frame']
        assert report['visit']['map'] == 'utemple' and frame['epoch'] == 1
        assert frame['checkpoint_generation'] in ({0, 1} if self.wiping else {self.generation})
        for index, body in enumerate(frame['players']):
            assert body['stats']['sanity'] > 0 or index in self.allowed_deaths, f'Unexpected death: {index}'

    def on_drowned(self, victim):
        """Optional network interruption after both clients observe the death."""
        pass

    def drown_and_respawn(self, victim):
        index = 0 if victim == 'host' else 1
        before = self.reports['host']['frame']
        starts = {role: len(rows) for role, rows in self.history.items()}
        self.allowed_deaths = {index}
        self.set(victim)
        # The survivor is already following. Reissuing that command would
        # discard an in-flight path search at an unrelated player's death.
        self.until(lambda: all(r['frame']['players'][index]['stats']['sanity'] <= 0
                               for r in self.reports.values()), 90)
        assert all(r['frame']['players'][index]['player']['breath']['hits'] > 0
                   for r in self.reports.values()), 'Victim did not actually drown'
        self.on_drowned(victim)
        self.until(lambda: all(r['frame']['players'][index]['spawned_at'] > before['players'][index]['spawned_at']
                               and r['frame']['players'][index]['stats']['sanity'] > 0
                               for r in self.reports.values()), 35)
        self.allowed_deaths = set()
        self.set(victim, follow=True)
        results = {}
        for role, history in self.history.items():
            rows = history[starts[role]:]
            dead = [row for row in rows if row['players'][index]['health'] <= 0]
            assert dead, f'{role} did not observe drowning'
            death_ticks, followed = set(), []
            for row in dead:
                assert row['owner'] is None
                life = row['lives']['states'][index]
                assert isinstance(life, dict) and 'Spectating' in life
                remaining = life['Spectating']['ticks_left']
                assert 0 < remaining <= 1200
                death_ticks.add(row['tick'] - (1200 - remaining))
                if role == victim:
                    assert 'player_feet' in row['presentation'], 'Missing rendered spectator evidence'
                    verify_presence([row], role, generation=self.generation)
                    followed.append(row['camera_target'])
            assert len(death_ticks) == 1, 'Spectator countdown drifted'
            death_tick = death_ticks.pop()
            spawned = next(row for row in rows
                if row['players'][index]['spawned_at'] > before['players'][index]['spawned_at'])
            body = spawned['players'][index]
            assert body['spawned_at'] == death_tick + 1200, 'Respawn did not wait exactly ten seconds'
            assert body['health'] == 25 and spawned['lives']['states'][index] == 'Alive'
            elapsed = spawned['tick'] - body['spawned_at']
            assert 0 <= elapsed <= 120
            separation = distance(body['feet'], spawned['players'][1-index]['feet'])
            assert 36 - .01 <= separation <= 110 + 440 * elapsed / 120, 'Unsafe or distant swim respawn'
            assert all(spawned['driver']['clear']) and body['swimming']
            breath = spawned['breath'][index]
            assert breath['hits'] == 0 and breath['next_hurt'] == 0, 'Revived swimmer retained drowning damage'
            assert 0 <= breath['submerged'] <= elapsed / 120 + .001, 'Revived swimmer did not get fresh air'
            for actual, old in zip(spawned['players'], dead[0]['players']):
                for field in ('owned', 'collected', 'turtle_air'):
                    assert actual['stats'][field] == old['stats'][field], f'Respawn changed personal {field}'
            if role == victim:
                verify_presence([spawned], role, generation=self.generation)
                assert max(distance(p, followed[0]) for p in followed) > 10
            results[role] = dict(death_tick=death_tick, spawn_tick=body['spawned_at'],
                                 health=body['health'], separation=separation, elapsed_ticks=elapsed)
        assert results['host']['death_tick'] == results['guest']['death_tick']
        assert results['host']['spawn_tick'] == results['guest']['spawn_tick']
        self.evidence[victim + '_recovery'] = results
        print('PASS temple drowning, moving spectator and ten-second swim respawn:', victim, results, flush=True)

    def verify_rejoined_world(self, before, report):
        old = before['combat']['stage']['world']['levels']['utemple']
        current = report['combat']['stage']['world']['levels']['utemple']
        for field in ('guide', 'events', 'movers', 'node', 'open', 'scene', 'initialized'):
            assert current[field] == old[field], f'Rejoin changed temple {field}'
        elapsed = (report['frame']['tick'] - before['frame']['tick']) / 120
        assert abs(current['clock'] - old['clock'] - elapsed) < .001
        for body, prior in zip(report['frame']['players'], before['frame']['players']):
            assert body['stats']['turtle_air'] == prior['stats']['turtle_air']

    def wipe(self, saved):
        # Both existing follow commands remain active until the authored bubble
        # trail gives them comparable air. Do not reset path searches or edit
        # health/breath to manufacture a team wipe.
        self.until(lambda: all(wipe_air_ready(r) for r in self.reports.values()), 60)
        prepared = copy.deepcopy(self.reports)
        (self.folder / 'wipe-prepared.json').write_text(json.dumps(prepared))
        self.evidence['wipe_preparation'] = {role: dict(tick=r['frame']['tick'],
            air_used=[b['player']['breath']['submerged'] for b in r['frame']['players']],
            health=[b['stats']['sanity'] for b in r['frame']['players']])
            for role, r in prepared.items()}
        self.allowed_deaths = {0, 1}
        self.wiping = True
        for role in self.commands:
            self.set(role)
        self.until(lambda: all(r['frame']['checkpoint_generation'] == 1 for r in self.reports.values()), 100)
        self.generation = 1
        self.wiping = False
        self.allowed_deaths = set()
        proof = {}
        old_temple = saved['frame']['combat']['stage']['world']['levels']['utemple']
        for role, report in self.reports.items():
            row = next(row for row in self.history[role] if row['checkpoint_generation'] == 1)
            frame = report['frame']
            assert frame['lives']['states'] == ['Alive', 'Alive']
            assert frame['players'][0]['spawned_at'] == frame['players'][1]['spawned_at'] > 0
            for body, old in zip(frame['players'], saved['frame']['players']):
                expected = copy.deepcopy(old['stats'])
                if frame['tick'] > body['spawned_at']:
                    expected['loot'].setdefault('utemple$first', dict(awarded=[], drops=[]))
                assert body['stats'] == expected, 'Wipe changed saved personal resources'
            for body, old in zip(row['players'], saved['frame']['players']):
                elapsed = row['tick'] - body['spawned_at']
                assert 0 <= elapsed <= 120
                assert distance(body['feet'], old['player']['feet']) <= 2 + 220 * elapsed / 120
            temple = report['combat']['stage']['world']['levels']['utemple']
            for field in ('guide', 'events', 'movers', 'node', 'open', 'scene', 'initialized'):
                assert temple[field] == old_temple[field], f'Wipe retained later temple {field}'
            elapsed = (frame['tick'] - frame['players'][0]['spawned_at']) / 120
            assert abs(temple['clock'] - old_temple['clock'] - elapsed) <= 2 / 120
            assert all(report['route_driver']['clear'])
            proof[role] = dict(spawn_tick=frame['players'][0]['spawned_at'],
                               restored_clock=temple['clock'], health=[b['stats']['sanity'] for b in frame['players']])
        assert proof['host']['spawn_tick'] == proof['guest']['spawn_tick']
        self.evidence['wipe'] = proof
        print('PASS temple both-dead saved checkpoint restore:', proof, flush=True)

    def run(self):
        self.start_pair()
        checkpoint = self.folder / 'host' / 'coop-checkpoint.json'
        self.until(lambda: read(checkpoint) is not None, 30)
        saved = read(checkpoint)['payload']
        assert saved['visit']['map'] == 'utemple' and saved['frame']['checkpoint_generation'] == 0
        assert saved['frame']['combat']['stage']['world']['levels']['utemple']['guide'] is None
        (self.folder / 'starting-checkpoint.json').write_text(json.dumps(saved))
        for role in self.commands:
            self.set(role, follow=True)
        self.until(lambda: self.reports['host']['combat']['stage']['world']['levels']['utemple']['node'] >= 3)
        # Exercise the shared wipe before individual revivals introduce very
        # different air reserves. The restored checkpoint then supplies both
        # ordinary players for the separate spectator/rejoin/respawn cases.
        self.wipe(saved)
        self.rejoin_guest()
        for role in self.commands:
            self.set(role, follow=True)
        self.until(lambda: self.reports['host']['combat']['stage']['world']['levels']['utemple']['node'] >= 3)
        for victim in ('guest', 'host') if self.args.first == 'blue' else ('host', 'guest'):
            self.drown_and_respawn(victim)
        self.verify_network_states()
        self.verify_network_states('world_sha256', 'matching_world_states')
        self.verify_network_states('native_sha256', 'matching_native_residents')
        for role in ('guest', 'host'):
            self.set(role, finish=True)
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, build=self.build,
            internet=not self.args.local, first=self.args.first, delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Both drowning/spectator/respawn roles, shared saved temple wipe and entrance rejoin; no exit'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = TempleRecovery(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failure-report.json').write_text(json.dumps(dict(
            passed=False, error=str(error), build=pair.build, reports=pair.reports), indent=2))
        raise
    finally:
        pair.close()
