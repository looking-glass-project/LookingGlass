"""Check Fortress combat deaths, both spectator roles and the shared checkpoint.

The survivor waits in the courtyard while the other enters the actual Boojum
room. Only ordinary local inputs are supplied; health and checkpoint state are
never edited. The final paired fight exercises a normal both-dead restore.
"""
import argparse
import json
from pathlib import Path

from test_coop_play import ROOT, read, distance
from test_coop_fortress import FortressPair
from test_coop_village_recovery import Recovery
from test_coop_village_combat import VillageCombat


class FortressRecovery(Recovery, FortressPair):
    evidence_prefix = 'fortress-recovery-'
    death_goal = (-360,-2410,-192)
    death_navigate = False
    survivor_navigate = True

    def approach_fall(self, roles):
        for role in roles:
            self.set(role, (-3230,3050,0), navigate=True)
        self.until(lambda: all(self.reached(role) for role in roles), 40)
        for role in roles:
            self.portal(role)

    def survivor_goal(self, victim_index):
        # Retrace a verified courtyard landing. Straight walking here can fall
        # through a gap, take a reset portal and leave the survivor hanging from
        # a ledge, where the game correctly waits for a safe respawn platform.
        return (-3230,2730,0) if victim_index == 1 else (-3380,2700,0)

    def verify_wiped_world(self, report, saved):
        expected = saved['frame']['combat']['stage']['world']['fortress']
        actual = report['combat']['stage']['world']['fortress']
        for field in expected:
            if field != 'age':
                assert actual[field] == expected[field], f'Wipe did not restore Fortress {field}'
        old_actors = saved['frame']['combat']['stage']['world']['encounters']['actors']
        new_actors = report['combat']['stage']['world']['encounters']['actors']
        for old, new in zip(old_actors, new_actors):
            if 'Boojum' in old['enemy']:
                before, after = old['enemy']['Boojum'], new['enemy']['Boojum']
                assert before['health'] == after['health'] and before['state'] == after['state']
                assert distance(before['feet'], after['feet']) < 1, 'Wipe retained the pursuing Boojum position'
                assert after['waves'] == [], 'Wipe retained a scream projectile'

    def wipe(self, saved):
        assert all(b['stats']['sanity'] == 25 for b in self.bodies())
        old = saved['frame']['combat']['stage']['world']['encounters']['actors']
        current = self.reports['host']['combat']['stage']['world']['encounters']['actors']
        assert any(distance(a['enemy']['Boojum']['feet'], b['enemy']['Boojum']['feet']) > 100
                   for a, b in zip(old, current) if 'Boojum' in a['enemy']), 'Encounter did not progress beyond checkpoint'
        for goal in [(-3230,2730,0), (-3230,3050,0)]:
            self.go(goal, navigate=True, together=True)
        # Both arrive together, before either fragile player begins the fight.
        warps = [b['warped_at'] for b in self.bodies()]
        self.allowed_deaths = {0, 1}
        for role in self.commands:
            self.set(role, (-3098,3068,0), horizontal=True)
        self.until(lambda: all(b['warped_at'] != old for b, old in zip(self.bodies(), warps)), 30)
        self.cause_wipe(saved)

    def run(self):
        for role in self.commands:
            self.set(role, attack=False)
        self.start_pair()
        checkpoint = self.folder / 'host' / 'coop-checkpoint.json'
        self.until(lambda: read(checkpoint) is not None, 150)
        saved = read(checkpoint)['payload']
        assert saved['visit']['map'] == self.map_name and saved['frame']['checkpoint_generation'] == 0
        (self.folder / 'starting-checkpoint.json').write_text(json.dumps(saved))
        self.arrival_to_courtyard()
        for victim in ('guest','host') if self.args.first == 'red' else ('host','guest'):
            self.fall_and_respawn(victim)
        self.wipe(saved)
        VillageCombat.verify_coherence(self)
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps({'passed': True, 'internet': not self.args.local,
            'build': self.build, 'first': self.args.first, 'delay_ms': self.args.delay_ms,
            'jitter_ms': self.args.jitter_ms, 'evidence': self.evidence,
            'scope': 'both real Fortress combat death/spectator/respawn roles and saved-checkpoint wipe; no outgoing transition'}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue','red'), default='red')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = FortressRecovery(args)
    try:
        pair.run()
    finally:
        pair.close()
