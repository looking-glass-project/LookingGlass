"""Traverse the first Fortress visit with two independent internet clients.

Only ordinary local movement/attack goals are supplied. Portals, the splitting
room, cutscenes and travel use their authored triggers. Death/recovery and the
later return visit are separate requirements, not implied by this route.
"""
import argparse
import json
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_pandemonium import PandemoniumPair


class FortressPair(PandemoniumPair):
    map_name = 'fortress1'
    next_map = 'fortress2'
    evidence_prefix = 'fortress-test-'

    def until(self, predicate, seconds=90):
        # Short, confirmed legs should not leave the partner idle for minutes
        # after a missed landing. Startup/cinematic calls supply their own limit.
        return super().until(predicate, seconds)

    def fortress(self, role='host'):
        return self.reports[role]['combat']['stage']['world']['fortress']

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Fortress test stopped; partial evidence retained')
        super().pump()
        for role, report in self.reports.items():
            if self.history[role]:
                self.history[role][-1]['fortress'] = report['combat']['stage']['world']['fortress']

    def arrival_to_courtyard(self):
        self.until(lambda: all(self.fortress(r)['cinema']['arrival_done'] and
                              self.reports[r]['combat']['stage']['owner'] is None
                              for r in self.reports), 120)
        self.wait(.5)
        self.capture_pair('arrival-reunion')
        for goal in [(-3500,2140,0),(-3600,2430,0),(-3500,2550,48),
                     (-3450,2600,48),(-3380,2700,0),(-3230,2730,0),(-3230,3050,0)]:
            self.go(goal, True)

    def portal(self, role):
        index = 0 if role == 'host' else 1
        warp = self.bodies()[index]['warped_at']
        self.set(role, (-3098,3068,0), horizontal=True)
        self.until(lambda: self.bodies()[index]['warped_at'] != warp, 30)
        self.set(role)

    def arrival_and_portal(self):
        self.arrival_to_courtyard()
        for role in (self.first, self.partner):
            self.portal(role)
        self.wait(1)
        self.capture_pair('both-through-portal')
        print('PASS network Fortress arrival and independent portal contacts.', flush=True)

    def jump_to(self, goal):
        self.set(self.partner)
        for role in (self.first, self.partner):
            index = 0 if role == 'host' else 1
            jumps = self.bodies()[index]['player']['jumps']
            self.set(role, goal, run=True, jump=True)
            def landed(radius):
                body = self.bodies()[index]['player']
                return (body['jumps'] > jumps and body['grounded'] and
                        abs(body['feet'][2] - goal[2]) < 12 and distance(body['feet'],goal) < radius)
            # A jump lands on a platform, not an exact marker. Continuing to
            # chase that marker can turn around and run back off the edge.
            self.until(lambda: landed(80), 20)
            self.set(role)
            self.hold(.5)
            assert landed(100), f'{role}: did not remain on the landing after releasing input'

    def split_room(self):
        self.go((-360,-2410,-192), True)
        self.go((-400,-2240,-192), True)
        self.wait(12)
        assert all(self.fortress(r)['split'] == 1 for r in self.reports)
        self.capture_pair('split-room-complete')
        # Follow the supported lower perimeter and confirm each real landing.
        # A long input sequence can miss a landing under network timing and enter
        # the authored fall-reset portal before the next grounded replan.
        for goal in [(-308,-1953,-192),(-147,-1637,-192),(-45,-1373,-160),
                     (-20,-1208,-64),(-150,-1180,32)]:
            self.go(goal, True)
        # This upper ledge is walkable. Steer from the received position along
        # its supported straight sections instead of replaying a running turn.
        for goal in [(-150,-1248,32),(-400,-1248,32),(-444,-1279,32),
                     (-450,-1330,32),(-436,-1360,32)]:
            self.go(goal)
        # Steer this jump from each received position, rather than replaying a
        # timed turn that can overshoot the narrow ramp with delayed inputs.
        self.jump_to((-516,-1513,9))
        self.capture_pair('both-on-sloping-walkway')
        for goal in [(-516,-1577,12),(-516,-1641,16),
                     (-486,-1693,19),(-472,-1751,22),(-448,-1810,32),
                     (-350,-1833,28),(-158,-1833,31),(-50,-1878,34),(-30,-1900,36),
                     (34,-1951,39),(34,-2079,46),(-11,-2188,51),(-11,-2259,55)]:
            self.go(goal)
        self.jump_to((0,-2451,32))
        for goal in [(-150,-2630,32),(-300,-2610,32)]:
            self.go(goal, True)
        print('PASS network Fortress split room and upper route.', flush=True)

    def exit(self):
        if self.args.rejoin:
            self.rejoin_guest()
        self.set(self.partner)
        # The doorway is above the last landing. Press the ordinary run/jump
        # controls toward it; a grounded navigation destination is inappropriate.
        for _ in range(5):
            self.set(self.first, (-360,-2680,132), run=True, jump=True)
            self.hold(1.5)
            if self.arrived():
                return
        assert self.arrived(), 'Fortress exit was not reached with ordinary running jumps'

    def verify_rejoined_world(self, before, report):
        old = before['combat']['stage']['world']['fortress']
        new = report['combat']['stage']['world']['fortress']
        for field in old:
            if field != 'age':
                assert new[field] == old[field], f'Rejoin changed Fortress {field}'

    def verify(self):
        expected = {'Fortress1_Start'}
        assert self.captures == expected, f'Missing paired scene captures: {self.captures}'
        for role, rows in self.history.items():
            owners, reunions = {}, set()
            for row in rows:
                if row['epoch'] != 1:
                    continue
                if row['scene']:
                    owners.setdefault(row['scene'], set()).add(row['owner'])
                warp = row['players'][0]['warped_at']
                if row['owner'] is None and warp > 0 and warp == row['players'][1]['warped_at']:
                    allowance = 110 + max(0, row['tick'] - warp) * 640 / 120
                    assert distance(row['players'][0]['feet'], row['players'][1]['feet']) <= allowance
                    reunions.add(warp)
            for old, row in zip(rows, rows[1:]):
                if row['epoch'] == old['epoch'] == 1 and row['owner'] is not None and row['owner'] == old['owner']:
                    partner = 1 if row['owner'] == 'Blue' else 0
                    if row['players'][partner]['warped_at'] == old['players'][partner]['warped_at']:
                        assert row['players'][partner]['feet'] == old['players'][partner]['feet'], 'Waiting partner moved in a scene'
            # Arrival starts on loading this visit; tied entry gives Blue ownership.
            assert owners == {'Fortress1_Start': {'Blue'}}, owners
            assert len(reunions) == 1, reunions
            assert any(r['epoch'] == 1 and r['fortress']['split'] == 1 for r in rows)
            for index in range(2):
                assert len({r['players'][index]['warped_at'] for r in rows if r['epoch'] == 1}) >= 3, 'Missing personal portal warp'
            assert any(r['map'] == self.next_map and r['epoch'] == 2 for r in rows)
            assert all(b['spawned_at'] == 0 for r in rows for b in r['players']), 'Route silently respawned a player'
            assert all(b['stats']['owned'][i] > 0 for b in self.bodies(role) for i in (0,1)), 'Entry weapons lost'
            self.evidence[role] = {'scenes': {name: sorted(ids) for name, ids in owners.items()}, 'reunions': sorted(reunions)}
        host = {(r['epoch'],r['tick']): r for r in self.history['host']}
        compared = 0
        for guest in self.history['guest']:
            other = host.get((guest['epoch'],guest['tick']))
            if other:
                for field in ('players','owner','encounters','fortress'):
                    assert other[field] == guest[field], f'Authoritative {field} disagree'
                compared += 1
        self.verify_network_states()
        self.evidence['matching_authoritative_ticks'] = compared

    def run(self):
        self.start_pair()
        self.arrival_and_portal()
        self.split_room()
        self.exit()
        self.verify()
        self.finish_pair(2)
        (self.folder / 'result.json').write_text(json.dumps({'passed': True, 'internet': not self.args.local,
            'build': self.build, 'evidence': self.evidence, 'first': self.args.first,
            'delay_ms': self.args.delay_ms, 'jitter_ms': self.args.jitter_ms,
            'scope': 'first Fortress traversal in separate clients; death/return-visit matrix remains separate'}, indent=2))
        print('PASS two-process Fortress arrival: scene, portal, split room and authored exit.', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue','red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = FortressPair(args)
    try:
        pair.run()
    finally:
        pair.close()
