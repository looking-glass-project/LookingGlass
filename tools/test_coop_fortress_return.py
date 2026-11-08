"""Traverse the authored Fortress return with two delayed internet clients.

Starts at fortress1_start2, then supplies only ordinary movement, jumps and Blade
attacks. The reveal, reunion, ambush and school exit use their original triggers.
Death/recovery and focused enemy/resource checks remain separate requirements.
"""
import argparse
import hashlib
import json
import math
import time
from pathlib import Path

from test_coop_fortress import FortressPair
from test_coop_play import ROOT, distance


class FortressReturnPair(FortressPair):
    entry = 'fortress1_start2'
    next_map = 'skool1'
    evidence_prefix = 'fortress-return-'

    def __init__(self,args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def set(self,role,*args,**kwargs):
        super().set(role,*args,**kwargs)
        self.commands[role]['input_lead'] = min(1.5,.05+2*(self.args.delay_ms+self.args.jitter_ms)/1000) if role == 'guest' else 0

    def reached(self,role):
        return super().reached(role) and self.bodies(role)[0 if role == 'host' else 1]['player']['grounded']

    def until(self,predicate,seconds=90):
        end = time.monotonic()+seconds
        while not predicate():
            assert time.monotonic() < end, 'Fortress return timeout: '+str({role:dict(
                tick=r['frame']['tick'],phase=r['route_driver']['phase'],command=self.commands[role],
                feet=[b['player']['feet'] for b in r['frame']['players']]) for role,r in self.reports.items()})
            self.pump()

    def secure(self):
        self.wait(.5)
        def safe():
            for r in self.reports.values():
                driver = r['route_driver']
                # Walk closer to a distant ground guard along the authored
                # wall route; the Blade cannot hit it from the previous ledge.
                for kind,radius in [('guards',450),('boojums',900)]:
                    if any(e['active'] and e['health'] > 0 and any(visible and distance < radius
                           for visible,distance in zip(e['line_of_sight'],e['distance']))
                           for e in driver[kind]):
                        return False
                cast = r['combat']['stage']['world']['encounters']
                if cast and any(a['enemy'].get('Boojum',{}).get('waves') for a in cast['actors']):
                    return False
            return True
        self.until(safe,60)
        self.wait(.5)

    def route(self):
        self.wait(1)
        assert all(self.fortress(r)['returning'] and self.reports[r]['combat']['stage']['owner'] is None for r in self.reports)
        self.capture_pair('return-entrance')
        for goal in [(-4300,2250,510),(-3800,2250,405),(-3350,2250,390),
                     (-3060,2250,376),(-3060,2520,376)]:
            self.go(goal,True)
        # The corner parapet blocks hand-height Blade throws from the earlier
        # wall. Complete the ordinary approach before waiting for this fight.
        self.secure()
        self.set(self.partner)
        self.set(self.first,(-3060,2660,376))
        # The automatic scene-entry capture sees the original opening white
        # fade. Also capture the actual reveal after that fade has cleared.
        self.until(lambda: all(self.fortress(r)['cinema']['beat'] == 'Boojum'
                               and 1.75 <= self.fortress(r)['cinema']['time'] < 4.5
                               for r in self.reports),30)
        self.capture_pair('boojum-middle')
        self.evidence['middle_reveal_times'] = {r:self.fortress(r)['cinema']['time'] for r in self.reports}
        assert all(1.5 <= t < 4.5 for t in self.evidence['middle_reveal_times'].values())
        self.until(lambda: all(r['combat']['stage']['owner'] is None for r in self.reports.values()),30)
        self.go((-3060,2660,376))
        self.until(lambda: all(r['combat']['stage']['owner'] is None for r in self.reports.values()),30)
        self.secure()
        self.capture_pair('boojum-reunion')
        for goal in [(-3060,2780,376),(-3060,3340,376),(-3060,3890,376),
                     (-3620,3900,398),(-3810,3850,407),(-3840,3788,452)]:
            self.go(goal,True)
            self.secure()
        self.go((-3840,3788,452),horizontal=True)
        self.capture_pair('school-exit-approach')
        if self.args.rejoin:
            self.rejoin_guest()
        self.set(self.partner)
        for _ in range(5):
            # Aim at the original exit volume on the gently rocking school,
            # using the received shared clock, as the native route does.
            roll = math.radians(6+3*math.sin(self.fortress()['age']*math.tau/12))
            pitch = math.radians(3)
            x,y,z = 188,510,672
            y,z = math.cos(roll)*y-math.sin(roll)*z,math.sin(roll)*y+math.cos(roll)*z
            x,z = math.cos(pitch)*x+math.sin(pitch)*z,-math.sin(pitch)*x+math.cos(pitch)*z
            self.set(self.first,(-4060+x,3088+y,-184+z),run=True,jump=True)
            self.hold(1.5)
            if self.arrived():
                break
        assert self.arrived(), 'Authored school exit was not reached'

    def verify(self):
        scene = 'Fortress1_Boojum_Attack'
        assert self.captures == {scene}, self.captures
        for role,rows in self.history.items():
            visit = [r for r in rows if r['epoch'] == 1]
            owners = {r['owner'] for r in visit if r['scene'] == scene}
            assert owners == {self.args.first.title()}, owners
            reunions = set()
            for old,row in zip(visit,visit[1:]):
                if old['owner'] is not None and row['owner'] == old['owner']:
                    other = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][other]['feet'] == old['players'][other]['feet'], 'Partner moved during the reveal'
                if old['owner'] is not None and row['owner'] is None:
                    a,b = row['players']
                    assert a['warped_at'] == b['warped_at'] > 0
                    assert distance(a['feet'],b['feet']) <= 110 + max(0,row['tick']-a['warped_at'])*640/120
                    reunions.add(a['warped_at'])
            assert len(reunions) == 1, reunions
            assert any(r['fortress']['shutters'] for r in visit)
            actors = visit[-1]['encounters']['actors']
            ambush = [a for a in actors if a['name'].startswith('s1_booj')]
            assert len(ambush) == 2 and all(a['active'] for a in ambush)
            assert all(b['spawned_at'] == 0 for row in rows for b in row['players'])
            assert self.reports[role]['visit']['entry'] == 'skool1_start1'
            self.evidence[role] = dict(scene_owner=sorted(owners),reunions=sorted(reunions),ambush_activated=True)
        self.verify_network_states()

    def run(self):
        self.start_pair()
        self.route()
        self.verify()
        self.finish_pair(2)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,build=self.build,
            internet=not self.args.local,first=self.args.first,delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms,evidence=self.evidence,
            scope='Authored Fortress return traversal; combat/resources and recovery matrix remain separate'),indent=2))
        print('PASS two-process Fortress return: original reveal, reunion, ambush and school exit.',flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=ROOT/'target/debug/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--first',choices=('blue','red'),default='blue')
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--rejoin',action='store_true')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = FortressReturnPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
