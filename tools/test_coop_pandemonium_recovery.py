"""Exercise real Pandemonium hazard deaths, both spectator roles and team wipe.

Two independent public clients, ordinary inputs only. The game supplies all
damage and the automatic shared checkpoint. Clean traversal and rejoining are
covered separately by test_coop_pandemonium.py.
"""
import argparse
import json
from pathlib import Path

from test_coop_play import ROOT, read
from test_coop_pandemonium import PandemoniumPair
from test_coop_village_recovery import Recovery


class PandemoniumRecovery(Recovery, PandemoniumPair):
    evidence_prefix = 'pandemonium-recovery-'
    death_goal = (-3720, 1812, -384)
    death_navigate = True

    def approach_fall(self, roles):
        # The supported entrance-side landing overlooks the damaging street.
        # Use the same ordinary planner as the verified full traversal.
        for role in roles:
            self.set(role, (-3920,2300,-280), navigate=True)
        self.until(lambda: all(self.reached(role) for role in roles), 90)
        # The following leg also plans around the roof edges: a direct diagonal
        # can snag scenery from some valid landings. Its destination is out on
        # the damaging street, away from a safe platform within goal tolerance.

    def survivor_goal(self, victim_index):
        return (-4300,2560,-264) if victim_index == 1 else (-4152,2544,-280)

    def verify_wiped_world(self, report, saved):
        expected = saved['frame']['combat']['stage']['world']['pandemonium']
        actual = report['combat']['stage']['world']['pandemonium']
        for field in ('cart','rides','key','returned','leaving','departure','exit_sent','dialogue_done'):
            assert actual[field] == expected[field], f'Wipe did not restore Pandemonium {field}'
        for field in ('beat','warning_done','return_done'):
            assert actual['cinema'][field] == expected['cinema'][field], f'Wipe did not restore Pandemonium scene {field}'

    def wipe(self, saved):
        self.go((-4152,2544,-280), together=True)
        self.wait(.5)
        assert all(b['stats']['sanity'] == 25 for b in self.bodies())
        assert self.pand()['cinema']['warning_done']
        assert not saved['frame']['combat']['stage']['world']['pandemonium']['cinema']['warning_done']
        self.approach_fall(['host','guest'])
        self.cause_wipe(saved)

    def run(self):
        self.start_pair()
        checkpoint = self.folder / 'host' / 'coop-checkpoint.json'
        self.until(lambda: read(checkpoint) is not None, 120)
        saved = read(checkpoint)['payload']
        assert saved['visit']['map'] == self.map_name and saved['frame']['checkpoint_generation'] == 0
        (self.folder / 'starting-checkpoint.json').write_text(json.dumps(saved))
        self.rope()
        for goal in [(-3920,2300,-280),(-4152,2544,-280)]:
            self.go(goal, True)
        self.wait(10)
        assert self.pand()['cinema']['warning_done']
        for victim in ('guest','host') if self.args.first == 'red' else ('host','guest'):
            self.fall_and_respawn(victim)
        self.wipe(saved)
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps({'passed': True, 'internet': not self.args.local,
            'build': self.build, 'first': self.args.first, 'delay_ms': self.args.delay_ms,
            'jitter_ms': self.args.jitter_ms, 'evidence': self.evidence,
            'scope': 'both real Pandemonium hazard death/spectator/respawn roles and saved-checkpoint wipe; no outgoing transition'}, indent=2))


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
    pair = PandemoniumRecovery(args)
    try:
        pair.run()
    finally:
        pair.close()
