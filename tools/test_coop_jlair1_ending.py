"""Resume Burning Curiosity's ordinary ending with independent internet clients.

Watched Caterpillar ownership/reunion, personal Meta-essence and Watch use,
native climbs and actual paired travel are separate verified milestones.
An unchanged ordinary checkpoint is required; this never claims a full visit.
"""
import argparse
import copy
import hashlib
import json
import re
import struct
import zipfile
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import verify_rejoin_stats
from test_coop_jlair1 import CuriosityPair, ROLES, verify_arrival, verify_jump_flight
from test_coop_chess_resume import verify_resume_receipt, verify_resume_body

SCENE = 'JLair1_Caterpillar_Cinema1'
NAV_STEPS = tuple(range(77, 86)) + tuple(range(87, 91)) + tuple(range(98, 107))
ARRIVAL_STEPS = NAV_STEPS + (86, 92, 97)
# The live planner at 97 performs the ordinary jump/pull from the approach.
# The strict course separately checks the optional pickup, Watch and exact
# single-player setup marks; both bodies need not queue on that narrow mark.
PLAYABLE_OMITTED = frozenset((86, 90, 91, 92, 93, 94, 95, 96))
UPPER_LEDGE = [-2784., 3980., 2960.03125]


def verify_reunion_resources(before, after, ticks):
    """Model only ordinary idle ticks after the authoritative reunion stamp."""
    assert type(ticks) is int and 0 <= ticks <= 120, 'Unbounded reunion observation gap'
    verify_rejoin_stats(before, after, ticks)
    expected=copy.deepcopy(before)
    expected['will']=after['will']  # Already checked against normal difficulty regeneration.
    f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
    dt=f32(1/120)
    loot=expected.get('loot',{}).get('jlair1$first')
    if loot:
        smaller={'Super':'Large','Large':'Medium','Medium':'Small'}
        duration={'Large':10.,'Medium':20.,'Small':40.}
        for _ in range(ticks):
            remaining=[]
            for drop in loot['drops']:
                drop['remaining']=f32(drop['remaining']-dt)
                while drop['remaining']<=0 and drop['grade'] in smaller:
                    drop['grade']=smaller[drop['grade']]
                    drop['remaining']=f32(drop['remaining']+duration[drop['grade']])
                if drop['remaining']>0:remaining.append(drop)
            loot['drops']=remaining
    assert after==expected, 'Reunion changed resources beyond ordinary idle ticks'


def markers(base):
    raw = None
    for path in sorted(base.glob('*.pk3'), key=lambda p: p.name.lower()):
        with zipfile.ZipFile(path) as pack:
            for item in pack.infolist():
                if item.filename.replace('\\', '/').lower() == 'maps/jlair1.bsp':
                    assert item.file_size <= 128 * 1024 * 1024
                    raw = pack.read(item)
    assert raw and raw[:8] == b'FAKK*\0\0\0', 'Expected native Curiosity map'
    offset, length = struct.unpack_from('<II', raw, 12 + 14 * 8)
    assert offset >= 172 and offset + length <= len(raw)
    entities = [dict(re.findall(r'"([^"\n]*)"\s*"([^"\n]*)"', body))
                for body in re.findall(r'\{([^{}]*)\}', raw[offset:offset + length].decode())]
    xyz = lambda e: [float(v) for v in e['origin'].split()]
    assert entities[120]['classname'] == 'Item_MetaEssense-Large'
    assert entities[15]['classname'] == 'Item_WeaponPickup_Jacks'
    assert entities[99]['targetname'] == 'alice_cater_pos1'
    # The original route's pickup is placed on this observed ordinary floor.
    goal = xyz(entities[120]); goal[2] += .03125
    return dict(map_sha256=hashlib.sha256(raw).hexdigest(), health_key='jlair1:120',
                jacks_key='jlair1:15', health_goal=goal, scene_endpoint=xyz(entities[99]))


def verify_scene(history, first, endpoint):
    assert set(history)==set(ROLES), 'Missing independent scene client'
    owner = first.title(); partner = 1 if first == 'blue' else 0
    receipts = []
    for role, rows in history.items():
        active = [r for r in rows if r['map'] == 'jlair1' and r['owner'] is not None]
        assert active and all(r['owner'] == owner and r['scene'] == SCENE for r in active), 'Wrong Caterpillar actor or scene'
        assert {r['curiosity']['scene']['line'] for r in active} == set(range(15)), 'Missing watched Caterpillar lines'
        for row in active:
            assert row['camera_player'] == owner, 'Scene camera followed partner'
            assert not row['presentation']['visible_players'][partner], 'Scene exposed partner'
            assert row['players'][partner]['feet'] == active[0]['players'][partner]['feet'], 'Scene moved hidden partner'
            assert all(p['stats'] == old['stats'] and p['health'] > 0 and p['spawned_at'] == 0
                       for p, old in zip(row['players'], active[0]['players'])), 'Scene changed personal resources'
        after = next((r for r in rows if r['map'] == 'jlair1' and r['tick'] > active[-1]['tick']
                      and r['owner'] is None and r['curiosity']['caterpillar']), None)
        assert after and after['curiosity']['scene'] is None, 'Missing completed Caterpillar scene'
        assert all(p['grounded'] and p['health'] > 0 and p['immersion']['kind']=='Dry'
                   for p in after['players']) and all(after['driver']['clear']), 'Unsafe reunion'
        actor = after['players'][1-partner]; other = after['players'][partner]
        assert distance(actor['feet'][:2], endpoint[:2]) < .1, 'Original scene endpoint changed'
        assert distance(actor['feet'], other['feet']) <= 128, 'Partner did not reunite'
        assert distance(actor['feet'][:2], other['feet'][:2]) >= 35.95 or abs(actor['feet'][2]-other['feet'][2]) >= 55.95, 'Reunion overlaps'
        warp=actor['warped_at']
        assert other['warped_at']==warp and active[-1]['tick']<warp<=after['tick'], 'Missing authoritative reunion stamp'
        for p,old in zip(after['players'],active[0]['players']):
            verify_reunion_resources(old['stats'],p['stats'],after['tick']-warp)
        receipts.append(dict(peer=role, owner=owner, start=active[0]['tick'], reunion=warp,
                             end=after['tick'], ordinary_ticks=after['tick']-warp))
    return receipts


def verify_pickup(history, key, baseline):
    assert set(history)==set(ROLES), 'Missing independent pickup client'
    receipts = []
    for role, rows in history.items():
        rows = [r for r in rows if r['map'] == 'jlair1']
        for slot in range(2):
            assert key not in baseline[slot]['collected'], 'Meta-essence was already consumed in checkpoint'
            index = next((i for i, row in enumerate(rows) if key in row['players'][slot]['collected']), None)
            assert index is not None and index > 0, 'Missing personal Meta-essence collection'
            before, after = rows[index-1], rows[index]
            old, new = before['players'][slot], after['players'][slot]
            ticks=after['tick']-before['tick']
            assert 0 < ticks <= 120, 'Unbounded pickup observation gap'
            assert old['health']==old['stats']['sanity'] and new['health']==new['stats']['sanity'], 'Inconsistent pickup health'
            assert 0 < old['health'] <= new['health'] <= 100 and 0 <= new['stats']['will'] <= 100, 'Invalid pickup resources'
            # Meta-essence restores either resource. Full health still permits
            # collection for Will; passive regeneration alone is not evidence.
            rate={'Easy':.3,'Normal':.1,'Hard':0,'Nightmare':0}[old['stats']['difficulty']]
            will_gain=new['stats']['will']-old['stats']['will']
            assert new['health'] > old['health'] or will_gain > ticks/120*rate+.002, 'Pickup did not restore collector resources'
            assert before['players'][1-slot]['health'] == after['players'][1-slot]['health'], 'Pickup changed partner health'
            assert before['players'][1-slot]['collected'] == after['players'][1-slot]['collected'], 'Pickup consumed partner item'
            verify_rejoin_stats(before['players'][1-slot]['stats'],after['players'][1-slot]['stats'],ticks)
            receipts.append(dict(peer=role, player=slot, tick=after['tick'], health=[old['health'],new['health']],
                                 will=[old['stats']['will'],new['stats']['will']]))
    return receipts


def verify_watch(receipt, history):
    slot = ROLES.index(receipt['role'])
    rows = history[receipt['role']]
    before = next(r for r in rows if r['tick'] == receipt['before'] and r['epoch'] == 1)
    after = next(r for r in rows if r['tick'] == receipt['after'] and r['epoch'] == 1)
    old, new = [r['players'][slot]['stats'] for r in (before,after)]
    assert old['owned'][9] > 0 and old['powers']['stopped'] == 0 and old['powers']['recharge'] <= 0, 'Watch unavailable before input'
    assert after['driver']['command'] == receipt['command'] and after['driver']['phase'] == 'using watch', 'Unobserved Watch input'
    assert new['selected'] == 9 and new['powers']['stopped'] > 0 and new['powers']['recharge'] > 0, 'Watch did not activate normally'
    assert abs(old['will'] - new['will'] - 1) < .01, 'Wrong Watch cost'
    assert new['owned'] == old['owned'] and new['collected'] == old['collected'], 'Watch changed inventory'
    assert after['players'][1-slot]['stats']['powers']['stopped'] <= before['players'][1-slot]['stats']['powers']['stopped'], 'Watch activated partner inventory'


def verify_exit(history, saved):
    assert set(history)==set(ROLES), 'Missing independent exit client'
    receipts = []
    for role, rows in history.items():
        before = [r for r in rows if r['map'] == 'jlair1'][-1]
        entered = next((r for r in rows if r['map'] == 'jlair2' and r['epoch'] == 2), None)
        released = next((r for r in rows if r['map'] == 'jlair2' and r['epoch'] == 2 and r['owner'] is None), None)
        ready = next((r for r in rows if released and r['map'] == 'jlair2' and r['epoch'] == 2
                      and r['owner'] is None and 0 <= r['tick']-released['tick'] <= 120
                      and all(r['driver']['clear']) and all(p['grounded'] and p['health'] > 0 for p in r['players'])), None)
        assert before['curiosity']['caterpillar'] and entered and ready, 'Missing real onward travel'
        assert all(ready['driver']['clear']) and all(p['grounded'] and p['health'] > 0 for p in ready['players']), 'Unsafe onward pair'
        for slot, p in enumerate(entered['players']):
            for field in ('sanity','owned','collected','staff_component','school_items','difficulty'):
                assert p['stats'][field] == before['players'][slot]['stats'][field], f'Travel changed {field}'
            assert p['stats']['owned'][5] == saved[slot]['stats']['owned'][5] > 0, 'Travel lost personal Jacks'
        receipts.append(dict(peer=role, entered=entered['tick'], ready=ready['tick']))
    return receipts


def verify_climb(receipt, history, arrivals):
    role=receipt['role'];slot=ROLES.index(role)
    rows=[r for r in history[role] if r['epoch']==1 and receipt['before']<=r['tick']<=receipt['after']]
    assert rows and rows[0]['tick']==receipt['before'] and rows[-1]['tick']==receipt['after'], 'Missing climb interval'
    assert rows[0]['players'][slot]['grounded'], 'Climb has no grounded approach'
    assert any(r['players'][slot]['ledge'] is not None or r['players'][slot]['climb'] is not None for r in rows), 'Missing ordinary ledge catch'
    landing=next((r for r in arrivals if r['role']==role and r['step']==97),None)
    assert landing and receipt['before']<landing['tick']<=receipt['after'], 'Missing original climb landing'
    verify_arrival(landing,history)
    at=next(r for r in rows if r['tick']==landing['tick'])
    assert at['players'][slot]['jumps']>rows[0]['players'][slot]['jumps'], 'Climb lacks actual jump'


class EndingPair(CuriosityPair):
    evidence_prefix = 'jlair1-ending-'

    def __init__(self, args):
        super().__init__(args)
        self.markers = markers(args.data)
        self.evidence.update(scope='Resumed ending only; earlier continuous traversal and full visit remain unverified',
                             markers=self.markers, watches=[], climbs=[], climb_approaches=[], scene=[], exit=[])
        for name in ('test_coop_jlair1_ending.py',):
            raw = (Path(__file__).parent/name).read_bytes()
            (self.folder/'fixture-dependencies'/name).write_bytes(raw)
            self.evidence['fixture_sha256'][name] = hashlib.sha256(raw).hexdigest()
        assert self.saved, 'Ending requires an ordinary checkpoint'
        watched = self.saved['frame']['combat']['stage']['world']['levels']['jlair1']['caterpillar']
        assert watched == (args.start_step > 77), 'Checkpoint scene state does not match requested ending segment'
        self.evidence.update(start_step=args.start_step, optional_pickup_checked=not args.playable,
                             watch_checked=not args.playable, climb_checked=args.start_step<=97,
                             playable=args.playable)
        assert all(self.markers['jacks_key'] in p['stats']['collected'] and p['stats']['owned'][5] > 0
                   for p in self.saved['frame']['players']), 'Checkpoint lacks earned personal Jacks'

    def validate_report(self, role, report):
        if report['visit']['map'] == self.map_name:
            return super().validate_report(role, report)
        self.validate_lives(role, report)
        assert (report['visit']['map'],report['visit']['epoch']) == (self.next_map,2), 'Unexpected ending destination'
        assert report['build'] == self.args.build and report['route_driver'].get('recovery_trace_error') is None
        assert all(not p['stats']['god'] and not p['stats']['notarget'] for p in report['frame']['players'])

    def scene(self):
        first = 'host' if self.args.first == 'blue' else 'guest'
        for role in ROLES: self.set(role, attack=False)
        self.set(first, self.steps[77]['pos'], navigate=True, run=True, attack=False)
        self.until(lambda: any(r['combat']['stage']['owner'] is not None for r in self.reports.values()), self.navigation_timeout)
        for role in ROLES: self.set(role, attack=False)
        self.until(lambda: all(r['combat']['stage']['owner'] is None and r['combat']['stage']['world']['levels']['jlair1']['caterpillar']
                              for r in self.reports.values()), 240)
        self.evidence['scene'] = verify_scene(self.history, self.args.first, self.markers['scene_endpoint'])
        print('PASS watched Caterpillar and reunion', self.args.first, flush=True)

    def use_watches(self):
        self.clear()
        for role in ROLES: self.set(role, attack=False)
        start = self.reports['host']['frame']['tick']
        self.until(lambda: self.reports['host']['frame']['tick'] >= start+120, 10)
        for role in ROLES:
            before = self.history[role][-1]['tick']
            self.set(role, attack=False);self.commands[role]['use_watch'] = True
            command = self.commands[role]['id']
            self.until(lambda: self.body(role)['stats']['powers']['stopped'] > 0, 20)
            receipt = dict(role=role,before=before,after=self.history[role][-1]['tick'],command=command)
            verify_watch(receipt,self.history);self.evidence['watches'].append(receipt)
            self.set(role,attack=False)
        print('PASS both personal Watch activations',flush=True)

    def climb(self):
        # The native course aligns before jumping and pulling up this lip.
        # The live planner supplies ordinary jump/grab inputs from each actual
        # body; completion needs both the original landing and an observed climb.
        before={r:self.history[r][-1] for r in ROLES}
        if self.args.playable:
            first = min(ROLES, key=lambda r: distance(self.body(r)['player']['feet'], self.steps[90]['pos']))
            other = ROLES[1-ROLES.index(first)]
            self.set(other, attack=False)
            for role in (first, other):
                # Keep the original approach, but let each player clear the
                # narrow takeoff before bringing the other onto it.
                for point in (90, 92):
                    entry = self.move(role, self.steps[point]['pos'], navigate=True)
                    self.evidence['climb_approaches'].append(dict(entry, point=point))
                self.evidence['arrivals'].append(self.move(role, self.steps[97]['pos'], navigate=True))
                if role == first:
                    entry = self.move(role, self.steps[98]['pos'], navigate=True)
                    self.evidence['climb_approaches'].append(dict(entry, point=98))
                    self.set(role, self.steps[98]['pos'], run=True)
            self.set(other)
        else:
            self.go(self.steps[97]['pos'])
        for role in ROLES:
            receipt=dict(role=role,before=before[role]['tick'],after=self.history[role][-1]['tick'],
                         strategy='Ordinary planned jump and ledge pull to original landing')
            verify_climb(receipt,self.history,self.evidence['arrivals'])
            self.evidence['climbs'].append(receipt)

    def ending_exit(self):
        for role in ROLES:self.set(role,attack=False)
        first=(min(ROLES,key=lambda r:distance(self.body(r)['player']['feet'],self.steps[106]['pos']))
               if self.args.playable else ('host' if self.args.first=='blue' else 'guest'))
        self.set(first,self.steps[107]['pos'],run=True,attack=False)
        self.until(self.arrived,45)
        for role in ROLES:self.set(role,epoch=2,attack=False)
        # The original arrival scene releases Alice above the floor. Observe
        # her ordinary short fall before checking the shared playable arrival.
        self.until(lambda: all(r['combat']['stage']['owner'] is None and all(r['route_driver']['clear'])
                   and all(p['player']['grounded'] for p in r['frame']['players']) for r in self.reports.values()),180)
        self.evidence['exit']=verify_exit(self.history,self.saved['frame']['players'])
        print('PASS actual paired Jabberwock arrival',flush=True)

    def upper_route(self):
        # Parking beside each narrow takeoff can put the leader under a lip.
        # Follow the original consecutive approaches, then wait by the exit
        # while the partner follows. These are static ledges, not a shared lift.
        first=min(ROLES,key=lambda r:distance(self.body(r)['player']['feet'],self.steps[98]['pos']))
        other=ROLES[1-ROLES.index(first)]
        self.evidence['upper_leader']=first
        self.set(other)
        for role in (first,other):
            for self.step in range(98,106):
                (self.folder/f'before-upper-{role}-{self.step:03d}.json').write_text(json.dumps(self.reports))
                if self.step==100:
                    # The original path jumps onto the sloped lip before
                    # walking up to 100. Its grounded planner can miss this
                    # short approach; use the same ordinary jump explicitly.
                    approach=self.move(role,self.steps[99]['pos'],radius=2)
                    before=self.history[role][-1];slot=ROLES.index(role)
                    self.set(role,UPPER_LEDGE,run=True,jump=True)
                    command=self.commands[role]['id']
                    self.until(lambda:self.body(role)['player']['jumps']==before['players'][slot]['jumps']+1
                               and not self.body(role)['player']['grounded'],15)
                    flight=self.history[role][-1]
                    # Aim beyond the sloped lip so normal forward input
                    # continues uphill without a delayed mid-jump handoff.
                    self.until(lambda:self.reached(role) and self.body(role)['player']['grounded'],30)
                    arrival=self.receipt(role,UPPER_LEDGE)
                    verify_jump_flight(before,flight,self.history[role][-1],slot,command)
                    self.evidence.setdefault('upper_jumps',[]).append(dict(role=role,approach=approach,
                        before=before['tick'],flight=flight['tick'],arrival=arrival))
                # Keep following this short upper crossing through incidental
                # liquid contact; actual damage and deaths remain validated.
                self.evidence['arrivals'].append(self.move(role,self.steps[self.step]['pos'],
                    navigate=self.step!=100,recover_liquid=False))
                print('PASS upper approach',role,self.step,flush=True)
            if role==first:
                self.step=106
                self.evidence['arrivals'].append(self.move(role,self.steps[106]['pos'],navigate=True))
                self.set(role,self.steps[106]['pos'],run=True)
            else:self.set(role)
        self.completed.extend(range(98,107))
        self.checkpoint(107)

    def verify(self):
        expected = [step for step in range(self.args.start_step,108) if not (self.args.playable and step in PLAYABLE_OMITTED)]
        assert self.completed == expected, 'Incomplete requested ending'
        arrival_steps = [step for step in ARRIVAL_STEPS if step in expected]
        pairs={(step,role) for step in arrival_steps for role in ROLES
               if not (self.args.playable and step==106 and role!=self.evidence.get('upper_leader'))}
        assert {(r['step'],r['role']) for r in self.evidence['arrivals']} == pairs, 'Missing personal ending arrivals'
        assert len(self.evidence['arrivals']) == len(pairs), 'Duplicated ending arrivals'
        if self.args.playable and self.args.start_step<=98:
            assert self.evidence.get('upper_leader') in ROLES, 'Missing observed upper-route leader'
            jumps=self.evidence.get('upper_jumps',[])
            assert len(jumps)==2 and {r['role'] for r in jumps}==set(ROLES), 'Missing upper ledge jumps'
            for entry in jumps:
                assert entry['approach']['goal']==self.steps[99]['pos'] and entry['arrival']['goal']==UPPER_LEDGE
                verify_arrival(entry['approach'],self.history);verify_arrival(entry['arrival'],self.history)
                rows={r['tick']:r for r in self.history[entry['role']]}
                verify_jump_flight(rows[entry['before']],rows[entry['flight']],rows[entry['arrival']['tick']],
                                   ROLES.index(entry['role']),entry['arrival']['command'])
        for entry in self.evidence['arrivals']:
            goal=self.markers['health_goal'] if entry['step']==86 else self.steps[entry['step']]['pos']
            assert entry['goal']==goal
            verify_arrival(entry,self.history)
        if self.args.start_step == 77:
            self.evidence['scene']=verify_scene(self.history,self.args.first,self.markers['scene_endpoint'])
        else:
            assert self.evidence['scene'] == [], 'Resumed segment cannot claim a watched scene'
        if not self.args.playable:
            self.evidence['pickup']=verify_pickup(self.history,self.markers['health_key'],[p['stats'] for p in self.saved['frame']['players']])
        if not self.args.playable:
            assert {r['role'] for r in self.evidence['watches']}==set(ROLES) and len(self.evidence['watches'])==2
        else:
            assert self.evidence['watches']==[], 'Playable route cannot claim separate Watch coverage'
        for receipt in self.evidence['watches']:verify_watch(receipt,self.history)
        if self.args.start_step<=97:
            assert {r['role'] for r in self.evidence['climbs']}==set(ROLES) and len(self.evidence['climbs'])==2
        else:
            assert self.evidence['climbs']==[] and self.evidence['climb_approaches']==[], 'Upper continuation cannot claim an earlier climb'
        for receipt in self.evidence['climbs']:verify_climb(receipt,self.history,self.evidence['arrivals'])
        if self.args.playable and self.args.start_step<=97:
            entries=self.evidence['climb_approaches']
            assert len(entries)==5 and {(e['role'],e['point']) for e in entries if e['point']!=98}=={(r,p) for r in ROLES for p in (90,92)}
            assert sum(e['point']==98 for e in entries)==1
            for entry in entries:
                assert entry['goal']==self.steps[entry['point']]['pos']
                verify_arrival(entry,self.history)
        self.evidence['exit']=verify_exit(self.history,self.saved['frame']['players'])
        self.verify_navigation_evidence()

    def run(self):
        self.start_pair()
        receipt=json.loads((self.folder/'host/route-resume.json').read_text())
        verify_resume_receipt(receipt,self.saved,self.build);self.evidence['exact_resume_receipt']=receipt
        for report in self.reports.values():
            for body,old in zip(report['frame']['players'],self.saved['frame']['players']):
                verify_resume_body(body,old,report['frame']['tick']-self.saved['frame']['tick'])
        self.step=self.args.start_step
        if self.step == 77:self.scene()
        for self.step in range(self.args.start_step,108):
            if self.args.playable and self.step in PLAYABLE_OMITTED:continue
            if self.args.playable and 98<self.step<=106:continue
            if self.step in self.args.save_before:self.checkpoint(self.step)
            (self.folder/f'before-step-{self.step:03d}.json').write_text(json.dumps(self.reports))
            if self.args.playable and self.step==98:
                self.upper_route();continue
            elif self.step in NAV_STEPS:self.go(self.steps[self.step]['pos'])
            elif self.step==86:
                self.go(self.markers['health_goal'])
                self.evidence['pickup']=verify_pickup(self.history,self.markers['health_key'],[p['stats'] for p in self.saved['frame']['players']])
            elif self.step==91:self.use_watches()
            elif self.step==92:self.go(self.steps[92]['pos'])
            elif self.step==93:self.wait(.4)
            elif self.step in (94,95,96):continue  # Verified together by the native ledge replay at 97.
            elif self.step==97:
                self.climb()
                if not self.args.playable:self.completed.extend((94,95,96))
            elif self.step==107:self.ending_exit()
            self.completed.append(self.step)
            print('PASS ending step',self.step,flush=True)
        for role in ('guest','host'):
            self.set(role,finish=True,epoch=2,attack=False)
            self.until(lambda:self.processes[role].poll() is not None,30)
            assert self.processes[role].returncode==0
        self.verify()
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,full_route=False,internet=not self.args.local,build=self.build,evidence=self.evidence),indent=2))
        print('PASS resumed Burning Curiosity ending',self.args.first,flush=True)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,required=True);parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--build',required=True);parser.add_argument('--checkpoint',type=Path,required=True)
    parser.add_argument('--first',choices=('blue','red'),default='blue');parser.add_argument('--local',action='store_true')
    parser.add_argument('--start-step',type=int,choices=(77,78,83,86,90,98,107),default=77,
                        help='Resume a normal checkpoint; post-scene segments do not claim watched-scene coverage')
    parser.add_argument('--playable',action='store_true',help='Use ordinary planned climbs; omit optional healing and separate Watch choreography')
    parser.add_argument('--delay-ms',type=int,default=40);parser.add_argument('--jitter-ms',type=int,default=20)
    parser.add_argument('--navigation-timeout',type=int,default=240)
    parser.add_argument('--save-before',type=int,nargs='*',default=[78,83,86,87,90,92,98,106])
    args=parser.parse_args();args.last_step=107
    if args.start_step>=90 and not args.playable:parser.error('Upper continuation requires --playable; the earlier optional pickup is outside this segment')
    if not all(0<=v<=500 for v in (args.delay_ms,args.jitter_ms)):parser.error('Delay/jitter must be0..500ms')
    if not 30<=args.navigation_timeout<=600:parser.error('Navigation timeout must be30..600s')
    pair=EndingPair(args)
    try:pair.run()
    except BaseException as error:
        pair.evidence.update(passed=False,failed_step=pair.step,error=str(error),process_exit_codes_before_cleanup={r:p.poll() for r,p in pair.processes.items()});raise
    finally:pair.close()


if __name__=='__main__':main()
