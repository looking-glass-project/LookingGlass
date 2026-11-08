"""Reject incomplete or invented Curiosity ending evidence without game assets."""
import copy
import struct
import unittest

from test_coop_jlair1_ending import SCENE, verify_scene, verify_pickup, verify_watch, verify_exit, verify_climb, verify_reunion_resources


def player(at, health=80):
    stats=dict(sanity=health,owned=[1]*10,collected=['jlair1:15'],selected=0,will=100.,
               powers=dict(stopped=0.,recharge=0.),staff_component=0,school_items=[],difficulty='Normal')
    return dict(feet=at,health=health,grounded=True,spawned_at=0,warped_at=0,stats=stats,collected=stats['collected'],
                immersion=dict(kind='Dry'),jumps=0,ledge=None,climb=None)


def scene_rows(first='blue'):
    owner=first.title();partner=1 if first=='blue' else 0
    rows=[]
    for i in range(15):
        rows.append(dict(map='jlair1',epoch=1,tick=i+1,owner=owner,scene=SCENE,camera_player=owner,
            presentation=dict(visible_players=[False,False]),driver=dict(clear=[True,True]),
            players=[player([0,0,0]),player([100,0,0])],curiosity=dict(scene=dict(line=i),caterpillar=False)))
    after=copy.deepcopy(rows[-1]);after.update(tick=20,owner=None,scene=None)
    after['curiosity']=dict(scene=None,caterpillar=True)
    after['players'][1-partner]['feet']=[10,20,30];after['players'][partner]['feet']=[58,20,30]
    for p in after['players']:p['warped_at']=20
    rows.append(after)
    return {role:copy.deepcopy(rows) for role in ('host','guest')}


class EndingGuards(unittest.TestCase):
    def test_watched_scene_requires_both_clients_and_all_lines(self):
        for first in ('blue','red'):
            h=scene_rows(first);self.assertEqual(len(verify_scene(h,first,[10,20,30])),2)
            with self.assertRaisesRegex(AssertionError,'independent'):verify_scene(dict(host=h['host']),first,[10,20,30])
            h['guest'].pop(7)
            with self.assertRaisesRegex(AssertionError,'Missing watched'):verify_scene(h,first,[10,20,30])

    def test_scene_rejects_wrong_owner_visible_partner_or_resource_changes(self):
        for change in ('owner','visible','position','stats','camera'):
            h=scene_rows();r=h['guest'][5]
            if change=='owner':r['owner']='Red'
            elif change=='visible':r['presentation']['visible_players'][1]=True
            elif change=='position':r['players'][1]['feet'][0]+=1
            elif change=='stats':r['players'][0]['stats']['will']-=1
            elif change=='camera':r['camera_player']='Red'
            with self.assertRaises(AssertionError):verify_scene(h,'blue',[10,20,30])

    def test_reunion_requires_original_endpoint_support_and_separation(self):
        for change in ('endpoint','overlap','ground','liquid','clear'):
            h=scene_rows();r=h['host'][-1]
            if change=='endpoint':r['players'][0]['feet'][0]+=1
            elif change=='overlap':r['players'][1]['feet']=r['players'][0]['feet'][:]
            elif change=='ground':r['players'][1]['grounded']=False
            elif change=='liquid':r['players'][1]['immersion']['kind']='Lava'
            elif change=='clear':r['driver']['clear'][1]=False
            with self.assertRaises(AssertionError):verify_scene(h,'blue',[10,20,30])

    def test_reunion_allows_only_tick_bounded_regeneration_and_loot_aging(self):
        before=player([0,0,0])['stats'];before['will']=7.
        before['loot']={'jlair1$first':dict(awarded=[28],drops=[dict(id=28,grade='Small',origin=[10,20,30],remaining=24.)])}
        after=copy.deepcopy(before);f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
        for _ in range(22):
            after['will']=f32(after['will']+f32(f32(1/120)*f32(.1)))
            after['loot']['jlair1$first']['drops'][0]['remaining']=f32(after['loot']['jlair1$first']['drops'][0]['remaining']-f32(1/120))
        verify_reunion_resources(before,after,22)
        bad=copy.deepcopy(after);bad['will']+=.1
        with self.assertRaises(AssertionError):verify_reunion_resources(before,bad,22)
        bad=copy.deepcopy(after);bad['loot']['jlair1$first']['drops']=[]
        with self.assertRaises(AssertionError):verify_reunion_resources(before,bad,22)
        with self.assertRaisesRegex(AssertionError,'Unbounded'):verify_reunion_resources(before,after,121)

    def pickup_history(self):
        first=dict(map='jlair1',epoch=1,tick=1,players=[player([0,0,0]),player([100,0,0],70)])
        second=copy.deepcopy(first);second['tick']=2
        second['players'][0]['health']=100;second['players'][0]['collected'].append('jlair1:120')
        second['players'][0]['stats']['sanity']=100
        third=copy.deepcopy(second);third['tick']=3
        third['players'][1]['health']=100;third['players'][1]['collected'].append('jlair1:120')
        third['players'][1]['stats']['sanity']=100
        return {r:copy.deepcopy([first,second,third]) for r in ('host','guest')},[p['stats'] for p in first['players']]

    def test_pickup_requires_each_personal_collection_and_healing(self):
        h,b=self.pickup_history();self.assertEqual(len(verify_pickup(h,'jlair1:120',b)),4)
        h['guest'].pop()
        with self.assertRaisesRegex(AssertionError,'Missing personal'):verify_pickup(h,'jlair1:120',b)

    def test_pickup_rejects_shared_consumption_or_shared_healing(self):
        for field in ('health','collected','will'):
            h,b=self.pickup_history()
            if field=='health':h['host'][1]['players'][1]['health']=71
            elif field=='collected':h['host'][1]['players'][1]['collected'].append('jlair1:120')
            else:
                h['host'][0]['players'][1]['stats']['will']=50.
                h['host'][1]['players'][1]['stats']['will']=75.
            with self.assertRaises(AssertionError):verify_pickup(h,'jlair1:120',b)

    def test_essence_can_restore_will_at_full_health(self):
        h,b=self.pickup_history()
        for rows in h.values():
            for i,row in enumerate(rows):
                row['players'][0]['health']=row['players'][0]['stats']['sanity']=100
                row['players'][0]['stats']['will']=50. if i==0 else 75.
        self.assertEqual(len(verify_pickup(h,'jlair1:120',b)),4)

    def test_essence_rejects_no_benefit_or_passive_regeneration_only(self):
        for gain in (0.,.00083333):
            h,b=self.pickup_history()
            for rows in h.values():
                for i,row in enumerate(rows):
                    row['players'][0]['health']=row['players'][0]['stats']['sanity']=100
                    row['players'][0]['stats']['will']=7. if i==0 else 7.+gain
            with self.assertRaisesRegex(AssertionError,'did not restore'):verify_pickup(h,'jlair1:120',b)

    def watch_history(self):
        before=dict(epoch=1,tick=1,players=[player([0,0,0]),player([100,0,0])])
        after=copy.deepcopy(before);after.update(tick=2,driver=dict(command=5,phase='using watch'))
        after['players'][0]['stats'].update(selected=9,will=99.,powers=dict(stopped=20.,recharge=60.))
        return dict(host=[before,after]),dict(role='host',before=1,after=2,command=5)

    def test_watch_requires_actual_activation_cost_and_command(self):
        h,r=self.watch_history();verify_watch(r,h)
        for field,value in [('will',100.),('selected',0),('powers',dict(stopped=0.,recharge=0.))]:
            h,r=self.watch_history();h['host'][-1]['players'][0]['stats'][field]=value
            with self.assertRaises(AssertionError):verify_watch(r,h)
        h,r=self.watch_history();r['command']=6
        with self.assertRaisesRegex(AssertionError,'Unobserved'):verify_watch(r,h)

    def test_watch_cannot_activate_the_partner_inventory(self):
        h,r=self.watch_history();h['host'][-1]['players'][1]['stats']['powers']['stopped']=20.
        with self.assertRaisesRegex(AssertionError,'partner inventory'):verify_watch(r,h)

    def test_exit_requires_real_second_visit_and_personal_inventory(self):
        h=scene_rows();saved=[dict(stats=copy.deepcopy(p['stats'])) for p in h['host'][-1]['players']]
        for rows in h.values():
            after=copy.deepcopy(rows[-1]);after.update(map='jlair2',epoch=2,tick=30);rows.append(after)
        self.assertEqual(len(verify_exit(h,saved)),2)
        h['guest'][-1]['players'][1]['stats']['owned'][5]=0
        with self.assertRaisesRegex(AssertionError,'Travel changed owned'):verify_exit(h,saved)

    def test_climb_requires_observed_jump_ledge_and_original_landing(self):
        row=dict(map='jlair1',epoch=1,tick=1,owner=None,driver=dict(command=7,reached=True,clear=[True,True]),
                 players=[player([0,0,0]),player([100,0,0])])
        mid=copy.deepcopy(row);mid['tick']=2;mid['players'][0].update(jumps=1,grounded=False,ledge=dict(pulling=False))
        end=copy.deepcopy(row);end['tick']=3;end['players'][0].update(jumps=1,feet=[0,0,100])
        h=dict(host=[row,mid,end]);receipt=dict(role='host',before=1,after=3)
        arrival=dict(role='host',step=97,epoch=1,tick=3,command=7,goal=[0,0,100],feet=[0,0,100])
        verify_climb(receipt,h,[arrival])
        mid['players'][0]['ledge']=None
        with self.assertRaisesRegex(AssertionError,'ledge catch'):verify_climb(receipt,h,[arrival])

    def test_exit_allows_only_bounded_observed_settling_after_scene(self):
        h=scene_rows();saved=[dict(stats=copy.deepcopy(p['stats'])) for p in h['host'][-1]['players']]
        for rows in h.values():
            released=copy.deepcopy(rows[-1]);released.update(map='jlair2',epoch=2,tick=30)
            released['players'][0]['grounded']=False
            landed=copy.deepcopy(released);landed['tick']=60;landed['players'][0]['grounded']=True
            rows.extend((released,landed))
        self.assertTrue(all(r['ready']==60 for r in verify_exit(h,saved)))
        for invalid in ('late','blocked','airborne'):
            broken=copy.deepcopy(h);row=broken['guest'][-1]
            if invalid=='late':row['tick']=151
            elif invalid=='blocked':row['driver']['clear'][0]=False
            else:row['players'][0]['grounded']=False
            with self.assertRaises(AssertionError):verify_exit(broken,saved)


if __name__=='__main__':unittest.main()
