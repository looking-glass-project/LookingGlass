"""Real village falls, both spectator roles and a shared saved-checkpoint wipe.

Run in Anode. The normal route driver supplies only movement/attack inputs. No
damage, resource, position or world-state injection is used. The test finishes
at the restored village checkpoint; clean traversal/rejoin tests cover the exit.
"""
import argparse
import copy
import json
from pathlib import Path

from test_coop_play import ROOT, distance, read
from test_coop_village import Pair


class Recovery(Pair):
    death_goal = (-3155, 2670, -384)
    death_navigate = False
    survivor_navigate = False

    def __init__(self, args):
        super().__init__(args)
        self.allowed_deaths = set()
        self.expected_generation = 0
        self.allow_wipe = False
        self.wipe_seen = False
        self.recovery_captures = {}

    def validate_report(self, role, report):
        frame = report['frame']
        for i, body in enumerate(frame['players']):
            assert body['stats']['sanity'] > 0 or i in self.allowed_deaths, f'Unexpected death of player {i}'
        allowed = {0, 1} if self.allow_wipe else {self.expected_generation}
        assert frame['checkpoint_generation'] in allowed, 'Unexpected checkpoint generation'
        if self.allow_wipe and frame['checkpoint_generation'] == 1 and not self.wipe_seen:
            self.wipe_seen = True
            # Stop the hazardous movement as soon as the restored frame arrives.
            for player in self.commands:
                self.set(player)

    def capture_pair(self, label):
        for role, old in list(self.commands.items()):
            self.set(role, old['goal'], old['navigate'], capture=True, epoch=old['epoch'],
                     horizontal=old['horizontal'], rope=old['rope'], marker=old['marker'])
        self.until(lambda: all(self.reports[r]['route_driver']['captured'] == self.commands[r]['id']
                               for r in self.commands), 30)
        self.recovery_captures[label] = {r: f'{r}/route-capture-{self.commands[r]["id"]}.png'
                                        for r in self.commands}

    def approach_fall(self, roles):
        # The diagonal from the scene landing hits a barrier. Follow the
        # supported approach before walking off the final unrailed drop.
        for goal in [(-2960,2998,138),(-2976,2940,164),(-2976,2876,184),(-2996,2840,184)]:
            for role in roles:
                self.set(role, goal)
            self.until(lambda: all(self.reached(role) for role in roles), 40)

    def survivor_goal(self, victim_index):
        return (-2880 if victim_index == 1 else -2936, 3024, 128)

    def fall_and_respawn(self, victim):
        index = 0 if victim == 'host' else 1
        partner = 'guest' if victim == 'host' else 'host'
        self.wait(.5)
        self.approach_fall([victim])
        before = self.reports['host']['frame']
        starts = {r: len(rows) for r, rows in self.history.items()}
        self.allowed_deaths = {index}
        self.set(victim, self.death_goal, navigate=self.death_navigate)
        self.set(partner)
        self.until(lambda: all(r['frame']['players'][index]['stats']['sanity'] <= 0
                               for r in self.reports.values()), 90)
        self.set(victim)
        # Ordinary movement provides a moving spectator subject. Some maps need
        # their normal jump planner to cross a gap on this route.
        self.set(partner, self.survivor_goal(index), navigate=self.survivor_navigate)
        dead_at = self.reports['host']['frame']['tick']
        self.until(lambda: self.reports['host']['frame']['tick'] >= dead_at + 180)
        self.capture_pair(f'{victim}-spectating')
        self.until(lambda: all(r['frame']['players'][index]['spawned_at'] > before['players'][index]['spawned_at']
                               and r['frame']['players'][index]['stats']['sanity'] > 0
                               for r in self.reports.values()), 25)
        self.allowed_deaths = set()
        self.wait(.5)
        self.capture_pair(f'{victim}-respawned')
        results = {}
        for role, history in self.history.items():
            rows = history[starts[role]:]
            dead = [r for r in rows if r['players'][index]['health'] <= 0]
            assert dead, f'{role} never observed the death'
            death_ticks = set()
            followed = []
            for row in dead:
                assert row['owner'] is None, 'Unexpected cinematic during recovery probe'
                life = row['lives']['states'][index]
                assert isinstance(life, dict) and 'Spectating' in life, 'Missing ten-second countdown'
                remaining = life['Spectating']['ticks_left']
                assert 0 < remaining <= 1200
                death_ticks.add(row['tick'] - (1200 - remaining))
                if role == victim:
                    assert row['camera_player'] == ('Red' if index == 0 else 'Blue')
                    assert distance(row['camera_target'][:2], row['camera_subject_feet'][:2]) < 1
                    assert distance(row['camera_subject_feet'], row['players'][1-index]['feet']) < 32
                    followed.append(row['camera_target'])
            assert len(death_ticks) == 1, 'Countdown drifted from the host clock'
            death_tick = death_ticks.pop()
            spawned = next(r for r in rows if r['players'][index]['spawned_at'] > before['players'][index]['spawned_at'])
            body = spawned['players'][index]
            assert body['spawned_at'] == death_tick + 1200, 'Respawn did not wait exactly ten simulation seconds'
            assert body['health'] == 25 and spawned['lives']['states'][index] == 'Alive'
            separation = distance(body['feet'], spawned['players'][1-index]['feet'])
            assert 40 <= separation <= 110, f'Unsafe or distant respawn: {separation}'
            assert all(spawned['driver']['clear'])
            if role == victim:
                assert spawned['camera_player'] == ('Blue' if index == 0 else 'Red')
                assert distance(spawned['camera_target'][:2], body['feet'][:2]) < 1
                assert max(distance(p, followed[0]) for p in followed) > 10, 'Spectator did not follow the moving partner'
            # The ordinary approach/fight can collect a real item. Preservation
            # starts at death, not at the earlier movement command: otherwise a
            # legitimate pickup would be mistaken for a respawn inventory bug.
            for actual, old in zip(spawned['players'], dead[0]['players']):
                for field in ('owned', 'collected'):
                    assert actual['stats'][field] == old['stats'][field], f'Death changed personal {field}'
            assert spawned['players'][1-index]['health'] == before['players'][1-index]['stats']['sanity']
            results[role] = {'death_tick': death_tick, 'spawn_tick': body['spawned_at'],
                             'health': body['health'], 'separation': separation,
                             'collected_at_death': dead[0]['players'][index]['stats']['collected']}
        assert results['host']['death_tick'] == results['guest']['death_tick']
        assert results['host']['spawn_tick'] == results['guest']['spawn_tick']
        self.evidence[victim + '_recovery'] = results
        print('PASS', self.map_name, 'real traversal death and ten-second spectator/25-health respawn:', victim, results, flush=True)

    def wipe(self, saved):
        self.go((-2936, 3024, 128))
        self.wait(.5)
        assert all(b['stats']['sanity'] == 25 for b in self.reports['host']['frame']['players'])
        done = self.reports['host']['combat']['stage']['world']['village']['cinema']['done']
        original = saved['frame']['combat']['stage']['world']['village']['cinema']['done']
        assert set(done) > set(original), 'World did not progress beyond the saved checkpoint'
        self.approach_fall(['host', 'guest'])
        self.cause_wipe(saved)

    def verify_wiped_world(self, report, saved):
        village = report['combat']['stage']['world']['village']
        expected = saved['frame']['combat']['stage']['world']['village']
        assert village['cinema']['done'] == expected['cinema']['done'], 'Wipe retained later scene completions'
        assert village['hatch_open'] == expected['hatch_open'], 'Wipe did not restore the shared hatch'

    def cause_wipe(self, saved):
        self.allowed_deaths = {0, 1}
        self.allow_wipe = True
        for role in self.commands:
            self.set(role, self.death_goal, navigate=self.death_navigate)
        self.until(lambda: all(r['frame']['checkpoint_generation'] == 1 for r in self.reports.values()), 90)
        self.expected_generation = 1
        self.allow_wipe = False
        self.allowed_deaths = set()
        results = {}
        placements = {}
        for role, report in self.reports.items():
            frame = report['frame']
            restored = next(r for r in self.history[role] if r['checkpoint_generation'] == 1)
            assert frame['lives']['states'] == ['Alive', 'Alive']
            assert frame['players'][0]['spawned_at'] == frame['players'][1]['spawned_at'] > 0
            for body, old in zip(frame['players'], saved['frame']['players']):
                expected_stats = copy.deepcopy(old['stats'])
                if frame['tick'] > body['spawned_at']:
                    # loot::tick_with creates the current visit's empty ledger
                    # on the first resumed ordinary tick. This adds no award or
                    # item. Every resource, award and drop still compares exactly.
                    expected_stats['loot'].setdefault(getattr(self,'loot_visit',self.map_name + '$first'), {'awarded': [], 'drops': []})
                assert body['stats'] == expected_stats, 'Wipe did not restore saved personal resources exactly'
            # Even the first received frame can follow resumed held movement.
            # Bound horizontal drift by ordinary maximum run speed and the known
            # host ticks since restore, not an arbitrary fixed latency allowance.
            samples = []
            for body, old in zip(restored['players'], saved['frame']['players']):
                elapsed = restored['tick'] - body['spawned_at']
                assert 0 <= elapsed <= 120, 'First restored report is too late to verify checkpoint placement'
                horizontal = distance(body['feet'][:2], old['player']['feet'][:2])
                allowance = max(20, 2 + 320 * elapsed / 120)
                assert horizontal <= allowance, 'Wipe missed the saved player placement'
                assert abs(body['feet'][2] - old['player']['feet'][2]) < 20, 'Wipe restored a different platform'
                samples.append({'elapsed_ticks': elapsed, 'horizontal_distance': horizontal,
                                'maximum_distance': allowance})
            placements[role] = samples
            self.verify_wiped_world(report, saved)
            assert all(report['route_driver']['clear'])
            results[role] = frame['players'][0]['spawned_at']
        assert results['host'] == results['guest'], 'Clients disagree about the team wipe'
        self.evidence['wipe'] = {'tick': results['host'], 'saved_tick': saved['frame']['tick'],
            'saved_resources_positions_and_scene_progress_restored': True,
            'first_restored_placement_samples': placements}
        self.capture_pair('checkpoint-restored')
        print('PASS', self.map_name, 'both-dead checkpoint restoration:', self.evidence['wipe'], flush=True)

    def run(self):
        self.start_pair()
        checkpoint = self.folder / 'host' / 'coop-checkpoint.json'
        self.until(lambda: read(checkpoint) is not None, 120)
        saved = read(checkpoint)['payload']
        assert saved['visit']['map'] == 'gvillage' and saved['frame']['checkpoint_generation'] == 0
        (self.folder / 'starting-checkpoint.json').write_text(json.dumps(saved))
        self.route_to_cliff()
        assert all(self.reports['host']['route_driver']['blade_pickups'])
        for victim in ('guest', 'host') if self.args.first == 'red' else ('host', 'guest'):
            self.fall_and_respawn(victim)
        self.wipe(saved)
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps({'passed': True, 'internet': not self.args.local,
            'build': self.build, 'first': self.args.first, 'delay_ms': self.args.delay_ms,
            'jitter_ms': self.args.jitter_ms, 'evidence': self.evidence,
            'scope': 'both real village death/spectator/respawn roles and shared saved-checkpoint wipe; no outgoing transition'}, indent=2))

    def close(self):
        (self.folder / 'recovery-captures.json').write_text(json.dumps(self.recovery_captures, indent=2))
        super().close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='red')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = Recovery(args)
    try:
        pair.run()
    finally:
        pair.close()
