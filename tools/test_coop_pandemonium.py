"""Traverse Pandemonium through two independent clients and ordinary inputs.

Uses the public internet invite by default, including configurable delay/jitter.
No position, health, inventory, quest or shared-world writes. The death matrix is
separate; --rejoin checks a fresh guest before the exit. Original map markers are read
by each client from its own installed assets, never copied into the harness.
"""
import argparse
import json
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair


class PandemoniumPair(Pair):
    map_name = 'pandemonium'
    next_map = 'fortress1'
    evidence_prefix = 'pandemonium-test-'

    @property
    def first(self):
        return 'host' if self.args.first == 'blue' else 'guest'

    @property
    def partner(self):
        return 'guest' if self.first == 'host' else 'host'

    def pand(self, role='host'):
        return self.reports[role]['combat']['stage']['world']['pandemonium']

    def bodies(self, role='host'):
        return self.reports[role]['frame']['players']

    def hold(self, seconds):
        tick = self.reports['host']['frame']['tick']
        self.until(lambda: self.arrived() or self.reports['host']['frame']['tick'] >= tick + seconds * 120)

    def capture_pair(self, label):
        for role, old in list(self.commands.items()):
            self.set(role, old['goal'], old['navigate'], capture=True, epoch=old['epoch'],
                     horizontal=old['horizontal'], rope=old['rope'], marker=old['marker'],
                     run=old['run'], jump=old['jump'])
        self.until(lambda: all(self.reports[r]['route_driver']['captured'] == self.commands[r]['id']
                               for r in self.commands), 30)
        self.evidence.setdefault('captures', {})[label] = {r: c['id'] for r, c in self.commands.items()}

    def go(self, goal=None, navigate=False, marker=None, horizontal=False, together=False):
        if self.arrived():
            return
        self.set(self.partner, goal if together else None, navigate if together else False,
                 marker=marker if together else None, horizontal=horizontal)
        self.set(self.first, goal, navigate, marker=marker, horizontal=horizontal)
        if not together:
            self.until(lambda: self.arrived() or self.reached(self.first))
            if not self.arrived():
                self.set(self.partner, goal, navigate, marker=marker, horizontal=horizontal)
        self.until(lambda: self.arrived() or all(self.reached(r) for r in self.commands))
        print('PASS network', self.map_name, 'leg', goal if marker is None else f'marker {marker}', flush=True)

    def rope(self):
        self.go((-3400, 2190, -32))
        for role in (self.first, self.partner):
            self.set(role, rope={'action': 'approach', 'goal': [-3552, 2032]})
            self.until(lambda: self.reached(role), 30)
        self.wait(.3)
        for role in self.reports:
            grips = [b['player']['rope'] for b in self.bodies(role)]
            assert all(grips) and grips[0]['id'] == grips[1]['id'], f'{role}: not sharing the entrance rope'
        before = [b['player']['feet'][2] for b in self.bodies()]
        for role in self.commands:
            self.set(role, rope={'action': 'climb', 'rise': -1})
        self.hold(1)
        self.wait(.3)
        after = [b['player']['feet'][2] for b in self.bodies()]
        assert all(a < b - 30 for a, b in zip(after, before)), (before, after)
        assert all(b['player']['rope'] for b in self.bodies())
        self.evidence['shared_rope'] = {'before_height': before, 'after_height': after}
        self.capture_pair('both-on-rope')
        for role in self.commands:
            self.set(role, rope={'action': 'release', 'goal': [-3920, 2300]})
        self.until(lambda: all(self.reached(r) for r in self.commands), 30)
        assert all(b['player']['rope'] is None and b['player']['grounded'] for b in self.bodies())
        self.capture_pair('rope-landing')
        print('PASS network shared rope: both grab, descend, release and land.', flush=True)

    def minecart(self):
        for goal in [(-3920,2300,-280),(-4152,2544,-280)]:
            self.go(goal, True)
        self.wait(10)
        for goal in [(-4500,2560,-264),(-5000,2560,-240)]:
            self.go(goal)
        for goal in [(-5024,2200,-160),(-5024,2080,-133),(-5024,1840,-92),(-5024,1600,-25)]:
            self.go(goal, True)
        self.set(self.partner)
        self.set(self.first, (-5024,1368,32))
        self.until(lambda: self.reached(self.first))
        self.hold(2)
        assert self.pand()['cart'] == 'Waiting' and self.pand()['rides'] == 0
        for role, report in self.reports.items():
            stage = report['combat']['stage']
            assert stage['owner'] is None and stage['world']['pandemonium']['cart'] == 'Waiting'
            notice = stage['notices'][0 if self.first == 'host' else 1]
            assert notice and notice['until'] > report['frame']['tick'], f'{role}: no boarding notice'
        self.evidence['minecart_waited_for_partner'] = True
        self.capture_pair('waiting-for-minecart-partner')
        self.set(self.partner, (-5024,1368,32))
        self.until(lambda: self.pand()['cart'] != 'Waiting')
        self.wait_for_minecart()
        assert self.pand()['cart'] == 'Finished' and self.pand()['rides'] == 1
        print('PASS network minecart: waits for both, one original ride.', flush=True)

    def wait_for_minecart(self):
        self.wait(65)

    def key_and_return(self):
        self.go((-3264,-560,440))
        self.wait(8)
        self.go((-3378,-1040,424), True)
        self.go((-3378,-1234,448))
        self.wait(3)
        self.go((-3412,-1036,440))
        for marker in [598,601,599,600,602,603,604,605,606,607,608,609]:
            self.go(marker=marker)
        self.set(self.partner)
        self.set(self.first, (-4312,-2528,536))
        self.until(lambda: self.pand()['key'])
        self.wait(3)
        for marker in range(610,620):
            self.go(marker=marker)
        warped = [b['warped_at'] for b in self.bodies()]
        for role in (self.first, self.partner):
            index = 0 if role == 'host' else 1
            self.set(role, (-3674,-2276,672), horizontal=True)
            self.until(lambda: self.bodies()[index]['warped_at'] != warped[index], 40)
            self.set(role)
        self.wait(3)
        for goal in [(-3600,1100,0),(-3600,1700,0)]:
            self.go(goal, horizontal=True, together=True)
        for goal in [(-3800,1900,-384),(-3920,2300,-280)]:
            self.go(goal, True, together=True)
        print('PASS network Gnome key and return portal.', flush=True)

    def exit(self):
        for goal in [(-3968,2560,-232),(-3968,2624,-232)]:
            self.go(goal)
        if self.args.rejoin:
            self.rejoin_guest()
        self.set(self.partner)
        self.set(self.first, (-3968,2900,-232))
        self.until(lambda: self.pand()['departure'])
        self.wait(1)
        self.until(self.arrived, 120)

    def verify_rejoined_world(self, before, report):
        old = before['combat']['stage']['world']['pandemonium']
        new = report['combat']['stage']['world']['pandemonium']
        for field in ('cart','rides','key','returned','leaving','departure','exit_sent'):
            assert new[field] == old[field], f'Rejoin changed Pandemonium {field}'
        for field in ('beat','warning_done','return_done'):
            assert new['cinema'][field] == old['cinema'][field], f'Rejoin changed Pandemonium scene {field}'

    def verify(self):
        expected = {'Elder_Gnome1_Warn_Thread','Minecart_Thread','alice_leave','Pand_End_Ship'}
        assert self.captures == expected, f'Missing paired scene captures: {expected - self.captures}'
        for role, rows in self.history.items():
            owners, reunions = {}, set()
            for row in rows:
                if row['scene'] and row['epoch'] == 1:
                    owners.setdefault(row['scene'], set()).add(row['owner'])
                if row['epoch'] == 1 and row['owner'] is None:
                    warp = row['players'][0]['warped_at']
                    # Before the return scene, the first portal contact warps
                    # only its owner. A reunion has a shared nonzero stamp.
                    if warp > 0 and warp == row['players'][1]['warped_at']:
                        allowance = 110 + max(0, row['tick'] - warp) * 640 / 120
                        assert distance(row['players'][0]['feet'], row['players'][1]['feet']) <= allowance
                        reunions.add(warp)
            for old, row in zip(rows, rows[1:]):
                if row['epoch'] == old['epoch'] == 1 and row['owner'] is not None and row['owner'] == old['owner']:
                    partner = 1 if row['owner'] == 'Blue' else 0
                    if row['players'][partner]['warped_at'] == old['players'][partner]['warped_at']:
                        assert row['players'][partner]['feet'] == old['players'][partner]['feet'], f'{role}: waiting partner moved in a scene'
            assert set(owners) == expected, f'{role}: missing Pandemonium scene: {owners}'
            assert all(v == {self.args.first.title()} for v in owners.values()), owners
            assert len(reunions) == 3, f'{role}: expected three scene reunions: {reunions}'
            assert any(r['map'] == self.next_map and r['epoch'] == 2 for r in rows)
            assert any(r['epoch'] == 1 and all(r['driver']['cards_pickups']) for r in rows), f'{role}: missing personal Cards pickup'
            assert all(b['stats']['owned'][1] > 0 for b in self.bodies(role)), f'{role}: Cards lost across transition'
            assert all(b['spawned_at'] == 0 for r in rows for b in r['players']), 'Traversal silently respawned a player'
            self.evidence[role] = {'scenes': {name: sorted(ids) for name, ids in owners.items()},
                                   'reunions': sorted(reunions)}
        host = {(r['epoch'],r['tick']): r for r in self.history['host']}
        compared = 0
        for guest in self.history['guest']:
            other = host.get((guest['epoch'],guest['tick']))
            if other:
                assert other['players'] == guest['players'] and other['owner'] == guest['owner'], 'Authoritative client states disagree'
                assert other['encounters'] == guest['encounters'], 'Authoritative enemy states disagree'
                compared += 1
        self.verify_network_states()
        self.evidence['matching_authoritative_ticks'] = compared
        print('Compared', compared, 'matching authoritative ticks.', flush=True)

    def run(self):
        self.start_pair()
        self.wait(.5)
        self.rope()
        self.minecart()
        self.key_and_return()
        self.exit()
        self.verify()
        self.finish_pair(2)
        (self.folder / 'result.json').write_text(json.dumps({'passed': True, 'internet': not self.args.local,
            'build': self.build, 'evidence': self.evidence,
            'first': self.args.first, 'delay_ms': self.args.delay_ms, 'jitter_ms': self.args.jitter_ms,
            'scope': 'full Pandemonium traversal in separate clients' +
                (' with fresh guest rejoin before the exit; death matrix remains separate' if self.args.rejoin
                 else '; death/rejoin matrix remains separate')}, indent=2))
        print('PASS two-process Pandemonium: shared rope, waiting cart, four scenes, personal Cards and authored network transition.', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue','red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true', help='Leave/rejoin the guest before the authored airship exit')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = PandemoniumPair(args)
    try:
        pair.run()
    finally:
        pair.close()
