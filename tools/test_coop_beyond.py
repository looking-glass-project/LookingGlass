"""Drive Beyond the Wall from its normal entrance with two real clients.

Only ordinary movement, jump, attack and E inputs are supplied. Machinery clocks
and poses are read from each client's replicated world; there are no gameplay
state writes, forced portals or resource refills. Optional focused probes stage
their initial placement outside a section and grant no full-route credit.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import time

from test_coop_pandemonium import PandemoniumPair
from test_coop_play import ROOT, distance


class BeyondPair(PandemoniumPair):
    map_name = 'fortress2'
    next_map = 'fortress1'
    evidence_prefix = 'beyond-route-'

    def __init__(self, args):
        if args.arches_only:
            self.start_at = (5280,-4364,0)
            self.evidence_prefix = 'beyond-arches-probe-'
        elif args.corridor_only:
            self.start_at = (1500,3360,0)
            self.evidence_prefix = 'beyond-corridor-probe-'
        self.allow_warp = False
        self.previous_reports = {}
        self.section = 'opening'
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def validate_report(self, role, report):
        super().validate_report(role, report)
        old = self.previous_reports.get(role)
        if old and old['frame']['epoch'] == report['frame']['epoch'] == 1:
            if not self.allow_warp and old['combat']['stage']['owner'] is None and report['combat']['stage']['owner'] is None:
                assert [b['warped_at'] for b in old['frame']['players']] == [b['warped_at'] for b in report['frame']['players']], \
                    f'Unexpected recovery portal during {self.section}'
        self.previous_reports[role] = report

    def set(self, role, *args, use_aim=None, beyond_step=None, runup=None, **kwargs):
        super().set(role, *args, **kwargs)
        self.commands[role].update(use_aim=use_aim, beyond_step=beyond_step, runup=runup)
        self.commands[role]['input_lead'] = min(1.5, .05 + 2*(self.args.delay_ms+self.args.jitter_ms)/1000) if role == 'guest' else 0

    def until(self, predicate, seconds=60):
        end = time.monotonic() + seconds
        while not predicate():
            assert time.monotonic() < end, f'{self.section} timeout: ' + str({
                role: dict(tick=r['frame']['tick'], phase=r['route_driver']['phase'],
                           command=self.commands[role], feet=[b['player']['feet'] for b in r['frame']['players']])
                for role,r in self.reports.items()})
            self.pump()

    def beyond(self, role='host'):
        return self.reports[role]['combat']['stage']['world']['beyond']

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Beyond route stopped; partial evidence retained')
        super().pump()
        for role, report in self.reports.items():
            if self.history[role]:
                self.history[role][-1]['beyond'] = report['combat']['stage']['world']['beyond']

    def reached(self, role):
        return super().reached(role) and self.bodies(role)[0 if role == 'host' else 1]['player']['grounded']

    def scene_done(self):
        self.until(lambda: all(r['combat']['stage']['owner'] is None for r in self.reports.values()), 120)

    def secure(self):
        self.wait(.5)
        start = self.reports['host']['frame']['tick']
        def safe():
            for report in self.reports.values():
                driver = report['route_driver']
                for enemy in driver['guards'] + driver['boojums']:
                    if enemy['active'] and enemy['health'] > 0 and any(
                            visible and dist < 1000 for visible, dist in zip(enemy['line_of_sight'], enemy['distance'])):
                        return False
                # Nearby Boojums can be temporarily hidden by the spiral.
                # Give them time to approach while both players are on broad
                # floor, as the native route does, instead of leaving at the
                # first loss of line of sight.
                if self.reports['host']['frame']['tick'] - start < 1800 and any(
                        e['active'] and e['health'] > 0 and min(e['distance']) < 1300
                        for e in driver['boojums']):
                    return False
                cast = report['combat']['stage']['world']['encounters']
                if cast and any(a['enemy'].get('Boojum',{}).get('waves') for a in cast['actors']):
                    return False
            return True
        self.until(safe)
        self.wait(.5)
        self.until(lambda: all(b['player']['grounded'] and math.hypot(*b['player']['velocity'][:2]) < 5
                               for b in self.bodies()), 10)

    def phase(self, field, phase, period, minimum=0):
        for role in self.commands:
            self.set(role)
        now = self.beyond()['age'] - self.beyond()[field]
        # As in the native route, an observation a fraction of a second after
        # the requested phase is usable; do not wait an entire extra cycle.
        if now >= minimum and (now-phase) % period < .25:
            return
        due = phase + max(0, math.ceil((max(now,minimum)-phase)/period)) * period
        self.until(lambda: self.beyond()['age'] - self.beyond()[field] >= due, period + 5)

    def jump(self, goal, runup=0, runup_axis=None):
        previous = [b['player']['jumps'] for b in self.bodies()]
        for role in self.commands:
            approach = None
            if runup:
                index = 0 if role == 'host' else 1
                feet = self.bodies()[index]['player']['feet']
                delta = [goal[i]-feet[i] if runup_axis is None or i == runup_axis else 0 for i in range(2)]
                length = math.hypot(*delta)
                assert length > 0
                approach = dict(millis=round(runup*1000),wish=[d/length for d in delta])
            self.set(role, goal, run=True, jump=True,runup=approach)
        def landed():
            return all(b['player']['jumps'] > old and b['player']['grounded']
                       and distance(b['player']['feet'],goal) < 60 for b,old in zip(self.bodies(),previous))
        self.until(landed, 20)
        self.wait(.25)
        assert landed(), 'A player did not remain on the jump landing'

    def portal(self, goal):
        self.allow_warp = True
        old = [b['warped_at'] for b in self.bodies()]
        for role in (self.first,self.partner):
            index = 0 if role == 'host' else 1
            if self.bodies()[index]['warped_at'] == old[index]:
                self.set(role,goal,horizontal=True,run=True)
                self.until(lambda: self.arrived() or self.bodies()[index]['warped_at'] > old[index], 25)
                self.set(role)
                self.hold(.3)
                self.scene_done()
        self.wait(.2)
        self.allow_warp = False

    def hall(self):
        for goal, nav in [((-750,4128,0),False),((-750,3800,0),False),
                          ((-944,3360,0),True),((-560,3800,0),True),((450,3800,0),False),
                          ((608,4128,0),True),((450,3800,0),True),((450,3360,0),False),
                          ((850,3360,0),False),((1500,3360,-8),False),((1660,3360,-64),False)]:
            # Let the leading player operate this narrow door. Two delayed
            # bots pressing E on the same closed-door observation can otherwise
            # toggle it open and closed on successive host ticks.
            self.go(goal,nav,together=goal[0] < 1000)
        self.secure()
        self.capture_pair('hall-cleared')

    def corridor(self):
        self.phase('long_started',8.883,88.5)
        self.jump((1800,3380,-64))
        self.phase('long_started',12.3,88.5)
        self.go((1880,3392,-64),together=True)
        self.phase('long_started',18.801,88.5)
        self.jump((2100,3380,-64))
        self.phase('long_started',27.794,88.5)
        self.jump((2340,3392,-64))
        self.phase('long_started',36.805,88.5)
        self.go((2352,3398,-64),together=True)
        # A static navigation plan can mistake a passing flipping platform for
        # a permanent floor here. Use an ordinary running jump over the gap.
        self.jump((2664,3504,-64),runup=.15,runup_axis=0)
        self.phase('long_started',46.490,88.5)
        self.go((2760,3488,-64),together=True)
        self.phase('long_started',55.954,88.5)
        self.jump((2980,3456,-64))
        self.phase('long_started',66.552,88.5)
        self.jump((3150,3580,-64))
        self.phase('long_started',75.341,88.5)
        self.jump((3350,3560,-64))
        self.jump((3540,3520,-64))
        self.go((3680,3520,-64),together=True)
        self.capture_pair('corridor-crossed')
        self.portal((4050,3520,-64))

    def puzzle(self):
        self.scene_done()
        self.go((1008,627,0),True)
        self.secure()
        self.go((1300,790,-328),True)
        self.go((1000,352,-333),True)
        self.secure()
        for count,(role,y) in enumerate([(self.first,465),(self.partner,593),(self.first,529)],1):
            self.set(role,(1320,y,-336),True)
            self.until(lambda: self.reached(role))
            # Navigation accepts a broad landing radius. Finish the approach
            # precisely so the single E press reaches this individual lever.
            self.set(role,(1320,y,-336))
            self.until(lambda: self.reached(role))
            self.set(role,use_aim=(0,0),attack=False)
            self.until(lambda: self.beyond()['notes'] == count, 10)
            self.set(role,attack=True)
        self.scene_done()
        assert self.beyond()['solved']
        self.secure()
        self.go((1300,790,-328),True)
        for x,y in [(1390,850),(1435,945),(1380,1040),(1300,1050),(1180,960),
                    (1160,870),(1080,790),(990,710),(1010,620),(1200,506)]:
            self.go((x,y,0),horizontal=True)
        self.secure()
        self.go((1300,576,0),True)
        self.go((1456,576,0))
        self.secure()
        self.go((1552,608,0))
        self.capture_pair('puzzle-solved')

    def walkway(self):
        expected = self.beyond()['walkway']
        for x,y in [(1638,636),(1734,668),(1830,700),(1926,732),(2022,764),(2118,796),(2214,828)]:
            self.go((x,y,0),together=True)
            expected += 1
            self.until(lambda: self.beyond()['walkway'] >= expected and self.beyond()['slide'] is None, 8)
        self.go((2320,864,0),together=True)
        self.secure()
        self.jump((2500,896,0))
        self.secure()
        self.go((2860,896,0),True,together=True)
        for x in [3160,3450,3800]:
            self.go((x,896,0),together=True)
        self.capture_pair('rolling-walkway-crossed')
        self.portal((4100,896,0))

    def arches(self):
        self.go((5280,-4260,0))
        self.scene_done()
        self.until(lambda: self.beyond()['age'] - self.beyond()['raised'] > 19.4, 45)
        self.go((5280,-4040,0),together=True)
        for step in [15,13,12,11,7,6,5,3,1]:
            for role in self.commands:
                self.set(role,beyond_step=step)
            self.until(lambda: all(self.reached(r) for r in self.commands), 20)
            print('PASS moving stair',step,[b['player']['feet'] for b in self.bodies()],flush=True)
        self.go((5280,-3040,0),horizontal=True,together=True)
        self.phase('raised',1.78,4)
        # The last gap needs the same short running approach as the native
        # route. Both clients send ordinary run inputs before the jump edge.
        self.jump((5280,-2810,576),runup=.15)
        self.go((5280,-2800,576),together=True)
        self.capture_pair('arches-crossed')
        self.portal((5280,-2500,576))

    def finish_route(self):
        self.scene_done()
        self.secure()
        def doors_ready():
            state = self.beyond()
            return state['last_started'] is not None and state['age'] - state['last_started'] >= [15,12.8,8.5][state['last_cycle']]
        self.until(doors_ready, 45)
        for goal in [(-544,4200,384),(-544,4070,384)]:
            self.go(goal,True)
        if self.args.rejoin:
            self.rejoin_guest()
        self.go((-544,3950,384),True)
        self.set(self.first,(-536,3820,384),horizontal=True,run=True)
        self.until(self.arrived, 30)

    def verify_rejoined_world(self, before, report):
        old, now = (r['combat']['stage']['world']['beyond'] for r in (before,report))
        assert old is not None and now is not None, 'Reconnect fixture must stop before leaving Beyond'
        for field in ['solved','notes','walkway','last_open']:
            assert old[field] == now[field], f'Rejoin changed {field}'

    def run(self):
        self.start_pair()
        if self.args.arches_only or self.args.corridor_only:
            kind = 'arches' if self.args.arches_only else 'corridor'
            self.section = 'staged ' + kind + ' probe'
            if self.args.corridor_only:
                self.go((1660,3360,-64),together=True)
                self.until(lambda: self.beyond()['long_started'] is not None,10)
            getattr(self,kind)()
            self.scene_done()
            self.verify_network_states()
            self.finish_pair(1)
            (self.folder/'result.json').write_text(json.dumps(dict(passed=True,build=self.build,
                internet=not self.args.local,first=self.args.first,delay_ms=self.args.delay_ms,
                jitter_ms=self.args.jitter_ms,evidence=self.evidence,
                scope='Staged start outside the ' + kind + '; ordinary section traversal only, not a campaign route'),indent=2))
            print('PASS staged',kind,'probe; no full-route credit.',flush=True)
            return
        for section in ['hall','corridor','puzzle','walkway','arches','finish_route']:
            self.section = section
            getattr(self,section)()
        self.verify_route()
        self.finish_pair(2)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,internet=not self.args.local,
            build=self.build,first=self.args.first,delay_ms=self.args.delay_ms,jitter_ms=self.args.jitter_ms,
            scope='Beyond normal-entrance traversal in two clients; recovery matrix remains separate',evidence=self.evidence),indent=2))
        print('PASS Beyond entrance to Fortress return in two clients.',flush=True)

    def verify_route(self):
        """Validate live or retained observations without changing game state."""
        self.verify_network_states()
        assert all(r['visit']['entry'] == 'fortress1_start2' for r in self.reports.values())
        assert all(b['spawned_at'] == 0 for rows in self.history.values() for r in rows for b in r['players']), 'Route respawned a player'
        expected = {'Start_GetSmart','End_GetSmart','ARCH_CAM_MOVE'}
        assert self.captures == expected, f'Missing scene views: {self.captures}'
        for role,rows in self.history.items():
            owners = {}
            for old,row in zip(rows,rows[1:]):
                if row['epoch'] != 1:
                    continue
                if row['scene']:
                    owners.setdefault(row['scene'],set()).add(row['owner'])
                if row['owner'] and row['owner'] == old['owner']:
                    partner = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][partner]['feet'] == old['players'][partner]['feet'], 'Waiting partner moved during scene'
                if old['owner'] and not row['owner']:
                    assert row['players'][0]['warped_at'] == row['players'][1]['warped_at'], 'Missing paired reunion'
                    assert distance(row['players'][0]['feet'],row['players'][1]['feet']) <= 110
            assert owners == {scene:{self.args.first.title()} for scene in expected}, owners
            assert any(r['beyond'] and r['beyond']['solved'] and r['beyond']['walkway'] == 9 and r['beyond']['last_open'] for r in rows)
            self.evidence[role] = {'scene_owners':{name:sorted(ids) for name,ids in owners.items()}}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=ROOT/'target/debug/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--first',choices=('blue','red'),default='blue')
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--rejoin',action='store_true')
    probes = parser.add_mutually_exclusive_group()
    probes.add_argument('--arches-only',action='store_true',help='Staged diagnostic start outside the arches; no full-route claim')
    probes.add_argument('--corridor-only',action='store_true',help='Staged diagnostic start outside the corridor; no full-route claim')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    args=parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair=BeyondPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
