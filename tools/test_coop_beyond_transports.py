"""Exercise Beyond passenger boarding through two real internet clients.

The Rage probe walks from the normal entrance. Other probes stage only their
initial placement on an approach, then use ordinary inputs. None of the probes is
a full campaign route; no machinery, resources or scene state is injected.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_beyond import BeyondPair
from test_coop_play import ROOT


class BeyondTransports(BeyondPair):
    def __init__(self,args):
        self.evidence_prefix = 'beyond-' + args.probe + '-'
        if args.probe == 'bridge':
            self.start_at = (1456,576,0)
        elif args.probe in ('elevator','late-rage'):
            # The actual upper return landing contains the authored LastStart
            # trigger. No machinery or completed-puzzle state is injected.
            self.start_at = (-160,4576,384)
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-transports-source.py').write_bytes(source)
        self.evidence['transports_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def index(self,role):
        return 0 if role == 'host' else 1

    def waiting_notice(self,role):
        index = self.index(role)
        return all(r['combat']['stage']['notices'][index]
                   and 'waiting for' in r['combat']['stage']['notices'][index]['text'].lower()
                   for r in self.reports.values())

    def rage(self):
        for goal in [(-750,4128,0),(-750,3800,0),(-560,3800,0)]:
            self.go(goal,together=True)
        self.secure()
        for goal in [(-160,3800,0),(-160,3680,0)]:
            self.set(self.first,goal)
            self.until(lambda: self.reached(self.first),15)
        self.set(self.first,(-160,3552,36),jump=True)
        self.until(lambda: self.reached(self.first),15)
        self.wait(2)
        a,other = self.index(self.first),self.index(self.partner)
        start = [b['player']['feet'] for b in self.bodies()]
        assert all(self.beyond(r).get('rage_started') is None for r in self.reports)
        assert all('fortress2:37' not in b['stats']['collected'] for b in self.bodies())
        assert self.waiting_notice(self.first), 'Missing all-board notice'
        self.capture_pair('rage-waits-for-partner')
        for goal in [(-96,3800,0),(-96,3680,0)]:
            self.set(self.partner,goal)
            self.until(lambda: self.reached(self.partner),15)
        self.set(self.partner,(-96,3552,36),jump=True)
        self.until(lambda: all(r['combat']['stage']['owner'] is not None for r in self.reports.values()),30)
        for r in self.reports.values():
            assert r['combat']['stage']['owner'] == self.args.first.title()
        self.hold(5)
        self.capture_pair('rage-scene-middle')
        self.scene_done()
        self.wait(5)
        for r in self.reports.values():
            bodies = r['frame']['players']
            assert all(b['player']['grounded'] and abs(b['player']['feet'][2]-start[a][2]+48) < .2 for b in bodies)
            assert 'fortress2:37' in bodies[a]['stats']['collected']
            assert 'fortress2:37' not in bodies[other]['stats']['collected']
            assert bodies[a]['warped_at'] == bodies[other]['warped_at'] > 0
        self.capture_pair('rage-paired-descent')
        # Reach the uncollected personal copy with the same ordinary jump used
        # in the native adapter check. The first player holds still on the lift.
        goal = tuple(self.bodies()[a]['player']['feet'])
        self.set(self.partner,goal)
        self.until(lambda: self.reached(self.partner),15)
        self.set(self.partner,(goal[0],goal[1],goal[2]+64),jump=True)
        self.until(lambda: all('fortress2:37' in b['stats']['collected'] for b in self.bodies()),10)
        self.wait(1)
        assert self.captures == {'Fortress2_Rage_Pickup'}
        for role,rows in self.history.items():
            owners = {r['owner'] for r in rows if r['scene'] == 'Fortress2_Rage_Pickup'}
            assert owners == {self.args.first.title()}
            for old,row in zip(rows,rows[1:]):
                if old['owner'] and row['owner']:
                    assert row['players'][other]['feet'] == old['players'][other]['feet'], 'Waiting partner moved in Rage scene'
            assert sum(bool(old['owner']) and not row['owner'] for old,row in zip(rows,rows[1:])) == 1, 'Rage scene replayed'
            assert all(b['stats']['powers']['rage'] > 0 for b in self.bodies(role))
        self.capture_pair('both-personal-rage-copies')
        self.evidence['rage'] = dict(first=self.args.first,waited_for_partner=True,
            scene_count=1,paired_descent=48,second_personal_copy_by_ordinary_jump=True)
        print('PASS Rage boarding wait, first-owner scene, paired descent and separate personal copies.',flush=True)

    def bridge(self):
        # The far-side guard cannot be reached with the Blade from this staged
        # approach. Keep real combat active while checking the boarding barrier.
        self.wait(1)
        initial = self.beyond()['walkway']
        assert initial == 2
        # The paired spawn offsets Red onto the edge of a stable tile. Put the
        # trailing role on the same verified rear approach in either order.
        self.set(self.partner,(1456,576,0))
        self.until(lambda: self.reached(self.partner),15)
        self.set(self.first,(1552,608,0))
        self.until(lambda: self.reached(self.first),20)
        self.set(self.first,(1638,636,0))
        self.until(lambda: self.reached(self.first),20)
        self.wait(2)
        for r in self.reports:
            assert self.beyond(r)['walkway'] == initial and self.beyond(r)['slide'] is None
        assert self.waiting_notice(self.first), 'Missing bridge boarding notice'
        self.capture_pair('bridge-waits-for-partner')
        self.set(self.partner,(1552,608,0))
        self.until(lambda: self.beyond()['walkway'] == 3 and self.beyond()['slide'] is None,12)
        self.wait(.5)
        assert all(self.beyond(r)['walkway'] == 3 for r in self.reports)
        self.capture_pair('bridge-both-aboard')
        self.evidence['bridge'] = dict(first=self.args.first,waited_for_partner=True,one_completed_move=True)
        print('PASS rolling bridge waits for trailing player, then advances with both safely aboard.',flush=True)

    def elevator_wait(self,label,rider):
        index = self.index(rider)
        before = {role:dict(age=self.beyond(role)['age'],wait=self.beyond(role).get('lift_wait',0),
                    z=self.bodies(role)[index]['player']['feet'][2]) for role in self.reports}
        for role in self.commands:
            self.set(role)
        # Each delayed client must actually report two seconds of observation;
        # reaching a host-only deadline can leave the guest's last report short.
        self.until(lambda: all(self.beyond(role)['age'] >= old['age']+2
                               for role,old in before.items()),10)
        for role,old in before.items():
            now = self.beyond(role)
            elapsed = now['age']-old['age']
            assert elapsed > 1.8, 'Boarding barrier stopped the shared world clock'
            assert abs(now.get('lift_wait',0)-old['wait']-elapsed) < .1, 'Lift moved during a partial boarding'
            assert abs(self.bodies(role)[index]['player']['feet'][2]-old['z']) < .1, 'Waiting rider moved vertically'
        assert self.waiting_notice(rider)
        self.capture_pair(label)
        self.evidence[label] = before

    def elevator_board(self):
        self.until(lambda: all(self.beyond(r)['last_started'] is not None for r in self.reports),10)
        self.capture_pair('upper-return-trigger')
        for goal in [(-352,4380,0),(-352,3800,0)]:
            self.go(goal,horizontal=True,together=True)
        self.secure()
        self.capture_pair('lower-lift-approach')
        # Stay to either side of the uncollected Rage pickup. Its separate
        # original transformation/descent is covered by the Rage probe.
        for role,x in [(self.first,-224),(self.partner,-96)]:
            self.set(role,(x,3800,0))
            self.until(lambda: self.reached(role),15)
            self.set(role,(x,3680,0))
            self.until(lambda: self.reached(role),15)
            self.set(role)
        self.set(self.first,(-224,3552,16),horizontal=True)
        self.until(lambda: self.reached(self.first),20)
        self.elevator_wait('elevator-waits-for-partner',self.first)
        self.set(self.partner,(-96,3552,16),horizontal=True)
        self.until(lambda: self.reached(self.partner),20)
        self.wait(.2)

    def elevator(self):
        self.elevator_board()
        self.until(lambda: min(b['player']['feet'][2] for b in self.bodies()) > 200,12)
        assert abs(self.bodies()[0]['player']['feet'][2]-self.bodies()[1]['player']['feet'][2]) < .1
        self.capture_pair('elevator-paired-rise')
        self.until(lambda: min(b['player']['feet'][2] for b in self.bodies()) > 360,12)
        # Normal difficulty omits the two short Easy-only side bridges. Jump
        # the actual gap to the broad outer landing, with a short ordinary runup.
        self.set(self.first,(-544,3552,384),jump=True,run=True,
                 runup=dict(millis=100,wish=[-1,0]))
        self.until(lambda: self.reached(self.first),15)
        self.elevator_wait('elevator-waits-for-disembarking-partner',self.partner)
        self.set(self.partner,(224,3552,384),jump=True,run=True,
                 runup=dict(millis=100,wish=[1,0]))
        self.until(lambda: self.reached(self.partner),15)
        self.wait(.5)
        before = {role:dict(age=self.beyond(role)['age'],wait=self.beyond(role).get('lift_wait',0)) for role in self.reports}
        self.until(lambda: all(self.beyond(role)['age'] >= old['age']+2
                               for role,old in before.items()),10)
        for role,old in before.items():
            now = self.beyond(role)
            assert now['age'] > old['age']+1.8 and now.get('lift_wait',0) == old['wait'], 'Empty lift did not resume its clock'
            assert now.get('rage_started') is None, 'Cycling lift probe accidentally collected Rage'
            assert all(b['player']['grounded'] and abs(b['player']['feet'][2]-384) < .2 for b in self.bodies(role))
        self.capture_pair('elevator-both-disembarked')
        self.evidence['elevator'] = dict(first=self.args.first,authored_start_trigger=True,
            boarding_wait=True,paired_rise=True,disembarking_wait=True,empty_clock_resumed=True)
        print('PASS cycling elevator: authored activation, boarding wait, paired rise, disembarking wait and empty return clock.',flush=True)

    def late_rage(self):
        self.elevator_board()
        self.set(self.first,(-160,3552,16),horizontal=True)
        self.until(lambda: all(r['combat']['stage']['owner'] is not None for r in self.reports.values()),15)
        assert all(r['combat']['stage']['owner'] == self.args.first.title() for r in self.reports.values())
        self.hold(5)
        assert all(self.beyond(r)['cinema']['time'] > 4 for r in self.reports), 'Late Rage froze its original scene clock'
        self.capture_pair('late-rage-scene-middle')
        self.scene_done()
        self.wait(1)
        assert all(b['player']['grounded'] for r in self.reports for b in self.bodies(r))
        assert all('fortress2:37' in self.bodies(r)[self.index(self.first)]['stats']['collected'] for r in self.reports)
        for role,rows in self.history.items():
            assert sum(bool(old['owner']) and not row['owner'] for old,row in zip(rows,rows[1:])) == 1
            other = self.index(self.partner)
            for old,row in zip(rows,rows[1:]):
                if old['owner'] and row['owner']:
                    assert old['players'][other]['feet'] == row['players'][other]['feet'], 'Hidden late-Rage partner moved'
        self.capture_pair('late-rage-reunion')
        self.evidence['late_rage'] = dict(owner=self.args.first,cycling_before_pickup=True,grounded_reunion=True)
        print('PASS late Rage collected during an already cycling elevator ride and safe paired reunion.',flush=True)

    def run(self):
        self.start_pair()
        self.section = self.args.probe
        getattr(self,self.args.probe.replace('-','_'))()
        self.verify_network_states()
        assert all(b['spawned_at'] == 0 for rows in self.history.values() for r in rows for b in r['players'])
        self.finish_pair(1)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,build=self.build,
            internet=not self.args.local,first=self.args.first,delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms,evidence=self.evidence,
            scope=('Normal entrance to Rage pickup and lift descent' if self.args.probe == 'rage'
                   else 'Staged ' + self.args.probe + ' approach and ordinary boarding inputs') + '; no full-route credit'),indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=ROOT/'target/debug/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--probe',choices=('rage','bridge','elevator','late-rage'),required=True)
    parser.add_argument('--first',choices=('blue','red'),default='red')
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    parser.set_defaults(arches_only=False,corridor_only=False,rejoin=False)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = BeyondTransports(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
