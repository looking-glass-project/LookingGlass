"""Reject incomplete or unobserved Clockwork route receipts."""
import copy
import json
import unittest
from pathlib import Path
from test_coop_hatter1 import DESCENT, GEAR_APPROACH, GEAR_EXIT_APPROACH, parking_target, partner_contact, yield_target, verify_yields, verify_arrivals, verify_descent, verify_ledge_pulls, verify_gear_boarding, verify_gear_exit
from test_coop_hatter1 import recorded_arrival, next_stair_footing, verify_stair_waiting
from test_coop_hatter1 import next_corridor_footing, verify_corridor_waiting
from test_coop_hatter1 import validate_course_range, verify_gap_hold
from test_coop_hatter1 import recorded_node, verify_node_resolutions
from test_coop_hatter1 import verify_scene_handoff, verify_contact_handoff
from test_coop_hatter1 import verify_doorway_approaches


class DoorwayApproaches(unittest.TestCase):
    def setUp(self):
        self.steps=[{} for _ in range(117)]; self.steps[109]={'nav':[-4896,1632,251.03125]}
        self.steps[116]={'walk':[-4896,1664,240.03125]}
        self.entities=[{'classname':'info_pathnode'} for _ in range(94)]
        for i in (92,93): self.entities[i]={'classname':'func_rotatingdoor'}
        goal=self.steps[109]['nav']
        self.receipt=dict(step=107,role='host',goal=goal,tick=100,command=9,feet=goal,end_tick=304)
        body=dict(feet=goal,grounded=True,health=100,spawned_at=0,warped_at=50)
        first=dict(tick=100,players=[body],driver=dict(command=9,reached=True,clear=[True]),clockwork=dict(doors=[.3,.3]))
        last=copy.deepcopy(first);last.update(tick=304);last['clockwork']['doors']=[1.,1.]
        lead=copy.deepcopy(first);lead['tick']=80;lead['driver']['command']=8
        lead['players'][0]['feet']=self.steps[116]['walk']
        self.receipt['lead_in']=dict(goal=self.steps[116]['walk'],tick=80,command=8,feet=self.steps[116]['walk'])
        self.history={'host':[lead,first,last]}

    def check(self): verify_doorway_approaches([self.receipt],self.steps,self.history,self.entities)

    def test_observed_centered_approach_and_opening(self): self.check()

    def test_unacknowledged_arrival(self):
        self.history['host'][1]['driver']['reached']=False
        with self.assertRaisesRegex(AssertionError,'acknowledged'): self.check()

    def test_partly_open_neighbor(self):
        self.history['host'][-1]['clockwork']['doors'][1]=.35
        with self.assertRaisesRegex(AssertionError,'open fully'): self.check()

    def test_insufficient_hold(self):
        self.receipt['end_tick']=303; self.history['host'][-1]['tick']=303
        with self.assertRaisesRegex(AssertionError,'too short'): self.check()

    def test_warp_is_not_walking(self):
        self.history['host'][-1]['players'][0]['warped_at']=300
        with self.assertRaisesRegex(AssertionError,'warped'): self.check()

    def test_leaving_the_waiting_footing(self):
        self.history['host'][-1]['players'][0]['feet']=[-4896,1610,251.03125]
        with self.assertRaisesRegex(AssertionError,'left its footing'): self.check()

    def test_wrong_staging_goal(self):
        self.receipt['goal']=[0,0,0]
        with self.assertRaisesRegex(AssertionError,'differs from the course'): self.check()

    def test_wrong_lead_in(self):
        self.receipt['lead_in']['goal']=[0,0,0]
        with self.assertRaisesRegex(AssertionError,'centered doorway lead-in'): self.check()


class SceneHandoffs(unittest.TestCase):
    def setUp(self):
        self.entry=dict(step=67,scene='Port_Cine',role='host',trigger_command=10,
                        starts={'host':120,'guest':120},ends={'host':1900,'guest':1900})
        self.levers=[dict(step=66,lever=3,role='host',command=10)]
        stats=dict(sanity=75.,will=20.,difficulty='Normal',owned=[1,1],collected=[],
                   selected=1,school_items={},staff_component=False)
        bodies=[dict(feet=[x,0,0],grounded=True,health=75.,stats=copy.deepcopy(stats),spawned_at=0,warped_at=0)
                for x in (0,80)]
        rows=[]
        for tick,t in ((120,0),(960,7),(1872,14.6)):
            rows.append(dict(tick=tick,scene='Port_Cine',owner='Blue',camera_player='Blue',
                clockwork=dict(port=None,coop=dict(actor=0),scene=dict(kind='Port',time=t,line=0,skip=None)),
                players=copy.deepcopy(bodies),driver=dict(command=30,clear=[True,True]),
                presentation=dict(visible_players=[False,False])))
        after=copy.deepcopy(rows[-1]);after.update(tick=1900,scene=None,owner=None)
        after['clockwork'].update(port=20.,scene=None)
        after['presentation']['visible_players']=[True,True]
        for i,b in enumerate(after['players']):b.update(feet=[100+i*48,100,0],warped_at=1890)
        rows.append(after)
        self.history={peer:copy.deepcopy(rows) for peer in ('host','guest')}

    def check(self): verify_scene_handoff(self.entry,self.history,self.levers)

    def test_watched_port_and_shared_reunion(self): self.check()

    def test_red_lever_user_owns_scene(self):
        self.entry['role']='guest';self.levers[0]['role']='guest'
        for rows in self.history.values():
            for row in rows[:-1]:
                row.update(owner='Red',camera_player='Red');row['clockwork']['coop']['actor']=1
        self.check()

    def test_wrong_lever_user(self):
        self.levers[0]['role']='guest'
        with self.assertRaisesRegex(AssertionError,'lever user'): self.check()

    def test_partner_must_remain_hidden(self):
        self.history['guest'][1]['presentation']['visible_players'][1]=True
        with self.assertRaisesRegex(AssertionError,'displayed'): self.check()

    def test_partner_must_remain_frozen(self):
        self.history['host'][1]['players'][1]['feet'][0]+=1
        with self.assertRaisesRegex(AssertionError,'moved the waiting'): self.check()

    def test_scene_cannot_change_personal_inventory(self):
        self.history['guest'][1]['players'][0]['stats']['owned'][0]=0
        with self.assertRaisesRegex(AssertionError,'personal resources'): self.check()

    def test_no_revival_during_scene(self):
        self.history['host'][1]['players'][1]['spawned_at']=960
        with self.assertRaisesRegex(AssertionError,'revived or warped'): self.check()

    def test_skip_is_not_watched_evidence(self):
        self.history['guest'][1]['clockwork']['scene']['skip']=.1
        with self.assertRaisesRegex(AssertionError,'skipped'): self.check()

    def test_full_port_duration_required(self):
        for peer,rows in self.history.items():
            rows[1]['tick']=300;rows[2]['tick']=480;rows[3]['tick']=600
            for body in rows[3]['players']:body['warped_at']=590
            self.entry['ends'][peer]=600
        with self.assertRaisesRegex(AssertionError,'ended early'): self.check()

    def test_scene_beginning_must_be_observed(self):
        self.history['host'][0]['clockwork']['scene']['time']=2
        with self.assertRaisesRegex(AssertionError,'start was not observed'): self.check()

    def test_missing_reunion(self):
        self.history['guest'].pop()
        with self.assertRaisesRegex(AssertionError,'Missing completed'): self.check()

    def test_mismatched_shared_stamp(self):
        self.history['guest'][-1]['players'][1]['warped_at']-=1
        with self.assertRaisesRegex(AssertionError,'shared reunion stamp'): self.check()

    def test_overlapping_reunion(self):
        self.history['host'][-1]['players'][1]['feet']=[110,100,0]
        with self.assertRaisesRegex(AssertionError,'overlaps'): self.check()

    def test_obstructed_reunion(self):
        self.history['guest'][-1]['driver']['clear'][1]=False
        with self.assertRaisesRegex(AssertionError,'Unsafe'): self.check()

    def test_reunion_cannot_hide_partner(self):
        self.history['guest'][-1]['presentation']['visible_players'][0]=False
        with self.assertRaisesRegex(AssertionError,'remains hidden'): self.check()

    def hare(self):
        self.entry.update(step=60,scene='Hatter1_Cinema1',trigger_command=30)
        for peer,rows in self.history.items():
            initial=rows[0];after=rows[-1];active=[]
            for line in range(12):
                time=0 if line==0 else 31+line
                row=copy.deepcopy(initial);row.update(tick=120+time*120,scene='Hatter1_Cinema1')
                row['clockwork']=dict(hare=None,coop=dict(actor=0),scene=dict(kind='Hare',time=time,line=line,skip=None))
                active.append(row)
            after.update(tick=5290);after['clockwork']=dict(hare=43.,scene=None)
            for body in after['players']:body['warped_at']=5280
            self.entry['ends'][peer]=5290;self.history[peer]=active+[after]

    def test_contact_scene_watches_every_hare_line(self):
        self.hare();self.check()

    def test_hare_cannot_omit_a_dialogue_line(self):
        self.hare();self.history['guest'].pop(6)
        with self.assertRaisesRegex(AssertionError,'missed dialogue'): self.check()

    def test_hare_must_retain_triggering_movement(self):
        self.hare();self.entry['trigger_command']=12
        with self.assertRaisesRegex(AssertionError,'trigger movement'): self.check()

    def gryphon(self):
        self.entry.update(step=112,scene='Hatter1_Cinema2',trigger_command=30)
        for peer,rows in self.history.items():
            initial=rows[0];after=rows[-1];active=[]
            for line in range(7):
                time=0 if line==0 else 5.5+line
                row=copy.deepcopy(initial);row.update(tick=120+time*120,scene='Hatter1_Cinema2')
                row['clockwork']=dict(gryphon=None,coop=dict(actor=0),scene=dict(kind='Gryphon',time=time,line=line,skip=None))
                active.append(row)
            after.update(tick=1580);after['clockwork']=dict(gryphon=12.,scene=None)
            for body in after['players']:body['warped_at']=1570
            self.entry['ends'][peer]=1580;self.history[peer]=active+[after]

    def test_gryphon_watches_every_line(self):
        self.gryphon();self.check()

    def test_red_can_own_gryphon(self):
        self.gryphon();self.entry['role']='guest'
        for rows in self.history.values():
            for row in rows[:-1]:
                row.update(owner='Red',camera_player='Red');row['clockwork']['coop']['actor']=1
        self.check()

    def test_gryphon_missing_line(self):
        self.gryphon();self.history['guest'].pop(4)
        with self.assertRaisesRegex(AssertionError,'missed dialogue'): self.check()

    def test_gryphon_requires_original_opening(self):
        self.gryphon()
        for row in self.history['host'][:-1]:row['clockwork']['scene']['time']=.1
        with self.assertRaisesRegex(AssertionError,'opening action'): self.check()

    def test_gryphon_requires_triggering_command(self):
        self.gryphon();self.entry['trigger_command']=999
        with self.assertRaisesRegex(AssertionError,'trigger movement'): self.check()

    def test_gryphon_contact_can_replace_its_approach(self):
        self.gryphon();steps=[{} for _ in range(113)];steps[112]=dict(walk=[0,0,0])
        self.entry['interrupted_goal']=[0,0,0]
        verify_contact_handoff(self.entry,steps,[],self.history)

    def test_gryphon_reunion_is_not_a_personal_arrival(self):
        self.gryphon();steps=[{} for _ in range(113)];steps[112]=dict(walk=[0,0,0])
        self.entry['interrupted_goal']=[0,0,0]
        with self.assertRaisesRegex(AssertionError,'actual course approach'):
            verify_contact_handoff(self.entry,steps,[dict(step=112,role='guest')],self.history)

    def contact(self):
        self.hare();self.steps=[{} for _ in range(63)];self.steps[60]=dict(nav=[0,0,0])
        self.entry['interrupted_goal']=[0,0,0]

    def test_actual_scene_replaces_an_interrupted_contact_approach(self):
        self.contact();verify_contact_handoff(self.entry,self.steps,[],self.history)

    def test_missing_partner_scene_cannot_replace_arrival(self):
        self.contact();self.history['guest'].pop()
        with self.assertRaisesRegex(AssertionError,'Missing completed'):
            verify_contact_handoff(self.entry,self.steps,[],self.history)

    def test_teleport_cannot_be_labelled_as_personal_arrival(self):
        self.contact()
        with self.assertRaisesRegex(AssertionError,'actual course approach'):
            verify_contact_handoff(self.entry,self.steps,[dict(step=60,role='guest')],self.history)

    def test_contact_cannot_replace_an_unrelated_course_goal(self):
        self.contact();self.entry['interrupted_goal']=[100,0,0]
        with self.assertRaisesRegex(AssertionError,'actual course approach'):
            verify_contact_handoff(self.entry,self.steps,[],self.history)

    def parking_contact(self):
        self.contact();point=[0,48,0]
        row=copy.deepcopy(self.history['host'][0]);row['tick']=100;row['driver']['neighbors']=[[point],[]]
        self.history['host'].insert(0,row)
        self.entry['parking_approach']=dict(step=60,role='host',goal=point,ready=True,selected_tick=100,candidates=[point])
        self.entry['interrupted_goal']=point
        self.receipts=[dict(step=60,role='host',tick=90)]

    def test_scene_interrupts_observed_parking_after_first_arrival(self):
        self.parking_contact();verify_contact_handoff(self.entry,self.steps,self.receipts,self.history)

    def test_interrupted_parking_is_not_a_completed_handoff(self):
        self.parking_contact();self.entry['parking_approach']['tick']=110
        with self.assertRaisesRegex(AssertionError,'must not claim arrival'):
            verify_contact_handoff(self.entry,self.steps,self.receipts,self.history)

    def test_scene_parking_still_requires_observed_native_neighbor(self):
        self.parking_contact();self.entry['parking_approach']['candidates']=[]
        with self.assertRaisesRegex(AssertionError,'observed neighbor'):
            verify_contact_handoff(self.entry,self.steps,self.receipts,self.history)


class RecordedArrival(unittest.TestCase):
    def test_same_tick_old_command_is_not_a_recorded_arrival(self):
        row=dict(epoch=1,tick=27568,driver=dict(command=141,reached=True))
        report=dict(frame=dict(epoch=1,tick=27568))
        self.assertFalse(recorded_arrival(row,report,142))

    def test_waits_for_recorded_reached_flag(self):
        row=dict(epoch=1,tick=27568,driver=dict(command=142,reached=False))
        report=dict(frame=dict(epoch=1,tick=27568))
        self.assertFalse(recorded_arrival(row,report,142))
        row['driver']['reached']=True
        self.assertTrue(recorded_arrival(row,report,142))

    def test_old_frame_cannot_supply_the_receipt(self):
        row=dict(epoch=1,tick=27568,driver=dict(command=142,reached=True))
        report=dict(frame=dict(epoch=1,tick=27583))
        self.assertFalse(recorded_arrival(row,report,142))


class NodeResolutions(unittest.TestCase):
    def setUp(self):
        self.source=[{'node':0}];self.effective=[{'nav':[10.,20.,.03125]}]
        self.entities=[dict(classname='info_pathnode',origin='10 20 40')]
        self.receipts=[];self.history={}
        for index,role in enumerate(('host','guest')):
            projection=dict(command=100+index,epoch=1,tick=110+index,entity=0,origin=[10.,20.,40.],feet=[10.,20.,.03125])
            self.history[role]=[dict(tick=110+index,epoch=1,map='hatter1',owner=None,
                driver=dict(command=100+index,reached=False,phase='observing pathnode',pathnode=copy.deepcopy(projection)))]
            self.receipts.append(dict(step=0,role=role,tick=110+index,command=100+index,projection=projection))

    def check(self): verify_node_resolutions(self.source,self.effective,[0],self.receipts,self.history,self.entities)

    def change(self,key,value):
        self.receipts[1]['projection'][key]=copy.deepcopy(value)
        self.history['guest'][0]['driver']['pathnode'][key]=copy.deepcopy(value)

    def test_two_independent_native_projections(self): self.check()

    def test_missing_partner(self):
        self.receipts.pop()
        with self.assertRaisesRegex(AssertionError,'Missing personal'): self.check()

    def test_wrong_acknowledgment(self):
        self.history['guest'][0]['driver']['command']+=1
        with self.assertRaisesRegex(AssertionError,'unacknowledged'): self.check()

    def test_projection_is_not_an_arrival(self):
        self.history['guest'][0]['driver']['reached']=True
        with self.assertRaisesRegex(AssertionError,'must not claim arrival'): self.check()

    def test_future_projection(self):
        self.change('tick',200)
        with self.assertRaisesRegex(AssertionError,'different pathnode'): self.check()

    def test_different_native_footings(self):
        self.change('feet',[10.,20.,5.])
        with self.assertRaisesRegex(AssertionError,'Clients disagree'): self.check()

    def test_wrong_node_identity(self):
        self.change('entity',1)
        with self.assertRaisesRegex(AssertionError,'different pathnode'): self.check()

    def test_non_pathnode_entity(self):
        self.entities[0]['classname']='info_null'
        with self.assertRaisesRegex(AssertionError,'Wrong native'): self.check()

    def test_projection_outside_drop_bound(self):
        self.change('feet',[10.,20.,-300.])
        with self.assertRaisesRegex(AssertionError,'footing bounds'): self.check()

    def test_unretained_projection_is_not_accepted(self):
        row=self.history['host'][0]
        report=dict(frame=dict(epoch=1,tick=row['tick']),route_driver=dict(pathnode=self.receipts[0]['projection']))
        self.assertTrue(recorded_node(row,report,100,0))
        row['driver']['pathnode']['tick']-=1
        self.assertFalse(recorded_node(row,report,100,0))


class StairWaiting(unittest.TestCase):
    def setUp(self):
        self.steps=[{} for _ in range(23)]
        self.steps[21]=dict(walk=[-4848,2688,256.03125])
        self.steps[22]=dict(walk=[-4848,2592,288.03125])
        self.receipt=dict(step=21,role='host',goal=self.steps[22]['walk'],before_tick=110,tick=200,
                          command=172,feet=list(self.steps[22]['walk']))
        self.arrivals=[dict(step=21,role='host',tick=100),dict(step=21,role='guest',tick=300)]
        self.history={'host':[]}
        for tick,point in [(110,self.steps[21]['walk']),(200,self.steps[22]['walk'])]:
            body=dict(feet=list(point),health=89,spawned_at=0,grounded=True)
            self.history['host'].append(dict(tick=tick,players=[body,body],driver=dict(command=172,reached=True)))

    def check(self): verify_stair_waiting([self.receipt],self.steps,self.arrivals,self.history)

    def test_first_player_walks_to_next_footing_before_second_arrives(self): self.check()

    def test_wrong_course_target(self):
        self.receipt['goal']=[0,0,0]
        with self.assertRaisesRegex(AssertionError,'differs'): self.check()

    def test_missing_waiting_acknowledgment(self):
        self.history['host'][-1]['driver']['command']+=1
        with self.assertRaisesRegex(AssertionError,'unacknowledged'): self.check()

    def test_waiter_must_move_before_follower_arrives(self):
        self.arrivals[1]['tick']=150
        with self.assertRaisesRegex(AssertionError,'out of order'): self.check()

    def test_cannot_skip_a_nonwalking_step(self):
        self.steps[22]=dict(lever=2)
        with self.assertRaisesRegex(AssertionError,'ordinary walking'): self.check()

    def test_not_a_general_route_skip(self):
        with self.assertRaisesRegex(AssertionError,'limited'): next_stair_footing(self.steps,20)

    def test_return_flight_uses_each_next_authored_navigation_step(self):
        steps=json.loads((Path(__file__).resolve().parents[1]/'src/levels/hatter1/route_steps.json').read_text())
        for step in range(81,86):
            with self.subTest(step=step):
                self.assertEqual(next_stair_footing(steps,step),steps[step+1]['nav'])

    def test_return_flight_needs_observed_forward_arrival(self):
        self.steps=[{} for _ in range(84)]
        self.steps[82]=dict(nav=[-6240,3616,128.03125])
        self.steps[83]=dict(nav=[-6112,3616,160.03125])
        self.receipt.update(step=82,goal=self.steps[83]['nav'],feet=list(self.steps[83]['nav']))
        for arrival in self.arrivals: arrival['step']=82
        for row,step in zip(self.history['host'],(82,83)):
            row['players']=[dict(feet=list(self.steps[step]['nav']),health=16.53,spawned_at=0,grounded=True) for _ in range(2)]
        self.check()
        self.history['host'][-1]['driver']['reached']=False
        with self.assertRaisesRegex(AssertionError,'unacknowledged'): self.check()

    def test_return_flight_cannot_skip_an_action_or_unbounded_rise(self):
        steps=[{} for _ in range(84)];steps[82]=dict(nav=[-6240,3616,128.03125])
        steps[83]=dict(wait=1)
        with self.assertRaises(AssertionError): next_stair_footing(steps,82)
        steps[83]=dict(nav=[-6112,3616,256.03125])
        with self.assertRaisesRegex(AssertionError,'Invalid next stair'): next_stair_footing(steps,82)


class CorridorWaiting(unittest.TestCase):
    def test_narrow_return_path_uses_the_next_authored_point(self):
        steps=json.loads((Path(__file__).resolve().parents[1]/'src/levels/hatter1/route_steps.json').read_text())
        for step in range(100,108):
            with self.subTest(step=step):
                self.assertEqual(next_corridor_footing(steps,step),steps[step+1]['nav'])

    def test_return_handoff_cannot_skip_actions_or_large_height_changes(self):
        steps=[{} for _ in range(106)];steps[104]=dict(nav=[-5440,1824,216.03125])
        steps[105]=dict(wait=1)
        with self.assertRaisesRegex(AssertionError,'non-navigation'): next_corridor_footing(steps,104)
        steps[105]=dict(nav=[-5312,1792,312.03125])
        with self.assertRaisesRegex(AssertionError,'Invalid next corridor'): next_corridor_footing(steps,104)

    def setUp(self):
        self.steps=[{} for _ in range(27)]
        self.steps[25]=dict(nav=[-5120,2544,320.03125])
        self.steps[26]=dict(nav=[-5240,2408,320.03125])
        self.receipt=dict(step=25,role='guest',goal=self.steps[26]['nav'],before_tick=110,tick=200,
                          command=55,feet=list(self.steps[26]['nav']))
        self.arrivals=[dict(step=25,role='guest',tick=100),dict(step=25,role='host',tick=300)]
        self.history={'guest':[]}
        for tick,point in [(110,self.steps[25]['nav']),(200,self.steps[26]['nav'])]:
            body=dict(feet=list(point),health=89,spawned_at=0,grounded=True)
            self.history['guest'].append(dict(tick=tick,players=[body,body],driver=dict(command=55,reached=True)))

    def check(self): verify_corridor_waiting([self.receipt],self.steps,self.arrivals,self.history)

    def test_separate_forward_arrival_before_follower(self): self.check()

    def test_missing_arrival_acknowledgment(self):
        self.history['guest'][-1]['driver']['reached']=False
        with self.assertRaisesRegex(AssertionError,'unacknowledged'): self.check()

    def test_cannot_skip_a_lever_or_wait(self):
        for key in ('lever','wait'):
            with self.subTest(key=key):
                self.steps[26]={key:2}
                with self.assertRaisesRegex(AssertionError,'non-navigation'): self.check()

    def test_forward_motion_is_bounded(self):
        self.steps[26]['nav'][0]-=500
        with self.assertRaisesRegex(AssertionError,'Invalid next corridor'): self.check()

    def test_other_sections_cannot_use_this_handoff(self):
        with self.assertRaisesRegex(AssertionError,'limited'): next_corridor_footing(self.steps,20)


class GapCourse(unittest.TestCase):
    def setUp(self):
        self.steps=json.loads((Path(__file__).resolve().parents[1]/'src/levels/hatter1/route_steps.json').read_text())

    def test_complete_paired_sequence_is_accepted(self): validate_course_range(self.steps,33,42)

    def test_partial_paired_sequence_is_rejected(self):
        for start,last in [(36,38),(37,42),(40,42)]:
            with self.subTest(start=start,last=last),self.assertRaisesRegex(AssertionError,'complete 36-42'):
                validate_course_range(self.steps,start,last)

    def test_changed_authored_jump_requires_review(self):
        self.steps[40]['input'][4]=0
        with self.assertRaisesRegex(AssertionError,'changed gap inputs'): validate_course_range(self.steps,36,42)

    def test_unrelated_input_does_not_gain_support(self):
        self.steps[25]={'input':[0.1,1,0,0,0,0]}
        with self.assertRaisesRegex(AssertionError,'unimplemented'): validate_course_range(self.steps,25,35)


class GapHold(unittest.TestCase):
    def setUp(self):
        self.goal=[-5080,2795,320.03125]
        self.hold=dict(role='host',seconds=0.25,start_tick=100,end_tick=130)
        self.history={'host':[dict(tick=t,players=[dict(feet=list(self.goal),grounded=True)]) for t in (100,130)]}

    def check(self): verify_gap_hold(self.hold,self.goal,self.history)

    def test_supported_preparation_wait(self): self.check()

    def test_short_wait(self):
        self.hold['end_tick']=129
        with self.assertRaisesRegex(AssertionError,'ended early'): self.check()

    def test_airborne_wait(self):
        self.history['host'][-1]['players'][0]['grounded']=False
        with self.assertRaisesRegex(AssertionError,'left the takeoff'): self.check()

    def test_wrong_floor(self):
        self.history['host'][-1]['players'][0]['feet'][2]-=100
        with self.assertRaisesRegex(AssertionError,'left the takeoff'): self.check()


class Arrivals(unittest.TestCase):
    def setUp(self):
        self.steps = [dict(nav=[10, 20, 30]), dict(wait=1)]
        self.receipts = []
        self.history = {}
        for slot, role in enumerate(('host', 'guest')):
            body = dict(feet=[10,20,30], health=75, spawned_at=0, grounded=True)
            row = dict(tick=100+slot, owner=None, map='hatter1', epoch=1, players=[copy.deepcopy(body),copy.deepcopy(body)],
                       driver=dict(command=5+slot, reached=True, expected=[10,20,30]))
            self.history[role] = [row]
            self.receipts.append(dict(step=0,role=role,tick=100+slot,goal=[10,20,30],feet=[10,20,30],command=5+slot))

    def check(self):
        verify_arrivals(self.steps, [0,1], self.receipts, self.history)

    def test_complete_personal_receipts(self):
        self.check()

    def test_missing_second_player(self):
        self.receipts.pop()
        with self.assertRaisesRegex(AssertionError, 'Missing personal'): self.check()

    def test_unobserved_tick(self):
        self.receipts[1]['tick'] += 1
        with self.assertRaisesRegex(AssertionError, 'not observed'): self.check()

    def test_unacknowledged_command(self):
        self.history['guest'][0]['driver']['command'] += 1
        with self.assertRaisesRegex(AssertionError, 'not acknowledged'): self.check()

    def test_wrong_requested_goal(self):
        self.receipts[0]['goal'][0] += 1
        with self.assertRaisesRegex(AssertionError, 'differs'): self.check()

    def test_planner_point_is_not_the_requested_goal(self):
        self.history['host'][0]['driver']['expected'] = None
        self.history['guest'][0]['driver']['expected'] = [9, 19, 30]
        self.check()

    def test_airborne_or_revived_arrival(self):
        for field, value in [('grounded',False), ('spawned_at',99), ('health',0)]:
            with self.subTest(field=field):
                row = self.history['guest'][0]['players'][1]
                old = row[field]; row[field] = value
                with self.assertRaisesRegex(AssertionError, 'Invalid arrival'): self.check()
                row[field] = old

    def test_missed_position(self):
        self.history['host'][0]['players'][0]['feet'][0] += 9
        self.receipts[0]['feet'][0] += 9
        with self.assertRaisesRegex(AssertionError, 'Missed personal'): self.check()

    def test_cinematic_receipt(self):
        self.history['host'][0]['owner'] = 'Blue'
        with self.assertRaisesRegex(AssertionError, 'not observed'): self.check()


class Descent(unittest.TestCase):
    def setUp(self):
        self.history, self.arrivals, self.jumps = {}, [], []
        for slot, role in enumerate(('host','guest')):
            rows = []
            for index, goal in enumerate(DESCENT):
                tick, command = 100 + index * 100 + slot, 5 + index * 2 + slot
                body = dict(feet=list(goal), health=100, spawned_at=0, grounded=True, jumps=2+(index+1)//2)
                count = 2+index//2
                event = dict(kind='jump', epoch=1, tick=tick-50, input=[slot==0,slot==1],
                             before=[[goal,True,count],[goal,True,count]],
                             after=[[goal,False,count+1],[goal,False,count+1]])
                rows.append(dict(tick=tick,owner=None,map='hatter1',epoch=1,
                    players=[copy.deepcopy(body),copy.deepcopy(body)],
                    driver=dict(command=command,reached=True,input_events=[event] if index%2 else [])))
                self.arrivals.append(dict(step=index,role=role,tick=tick,goal=list(goal),feet=list(goal),command=command))
                if index%2:
                    self.jumps.append(dict(role=role,leg=index//2,command=command,before_tick=tick-100,after_tick=tick))
            self.history[role] = rows
        for index in (1,3):
            self.history['host'][index]['driver']['input_events'] += self.history['guest'][index]['driver']['input_events']
            self.history['guest'][index]['driver']['input_events'] = []

    def check(self):
        verify_descent(self.arrivals,self.jumps,self.history)

    def test_four_host_observed_jumps_with_personal_receipts(self): self.check()

    def test_missing_partner_jump(self):
        self.jumps.pop()
        with self.assertRaisesRegex(AssertionError,'Missing personal'): self.check()

    def test_no_actual_jump(self):
        self.history['guest'][1]['players'][1]['jumps'] = 2
        with self.assertRaisesRegex(AssertionError,'one actual jump'): self.check()

    def test_input_without_takeoff_is_rejected(self):
        self.history['host'][1]['driver']['input_events'][1]['after'][1][1] = True
        with self.assertRaisesRegex(AssertionError,'takeoff unobserved'): self.check()

    def test_wrong_command_event_is_rejected(self):
        self.jumps[2]['command'] += 100
        with self.assertRaisesRegex(AssertionError,'command unobserved'): self.check()

    def test_guest_event_does_not_replace_authoritative_event(self):
        event = self.history['host'][1]['driver']['input_events'].pop()
        self.history['guest'][1]['driver']['input_events'] = [event]
        with self.assertRaisesRegex(AssertionError,'takeoff unobserved'): self.check()

    def test_event_uses_pre_step_tick(self):
        self.history['host'][1]['driver']['input_events'][0]['tick'] = 100
        self.check()


class LedgePull(unittest.TestCase):
    def setUp(self):
        self.pulls = [dict(role='guest',before_tick=100,after_tick=400,command=9)]
        rows = []
        for tick,ledge,climbs,grounded in [(100,dict(pulling=False),0,False),
                (200,dict(pulling=True),0,False),(400,None,1,True)]:
            body = dict(ledge=ledge,climbs=climbs,grounded=grounded)
            rows.append(dict(tick=tick,players=[copy.deepcopy(body),body],
                driver=dict(command=8 if tick==100 else 9,input_events=[])))
        self.history = dict(guest=rows,host=copy.deepcopy(rows))
        self.history['host'][1]['driver']['input_events'] = [dict(kind='jump',epoch=1,tick=110,input=[False,True],
            before=[[[0,0,0],True,0],[[0,0,0],False,1]],after=[[[0,0,0],True,0],[[0,0,0],False,1]])]

    def check(self): verify_ledge_pulls(self.pulls,self.history)

    def test_observed_native_pull_up(self): self.check()

    def test_missing_start_ledge(self):
        self.history['guest'][0]['players'][1]['ledge'] = None
        with self.assertRaisesRegex(AssertionError,'observed ledge'): self.check()

    def test_no_completed_climb(self):
        self.history['guest'][2]['players'][1]['climbs'] = 0
        with self.assertRaisesRegex(AssertionError,'did not finish'): self.check()

    def test_missing_pull_animation(self):
        self.history['guest'][1]['players'][1]['ledge']['pulling'] = False
        with self.assertRaisesRegex(AssertionError,'animation unobserved'): self.check()

    def test_missing_authoritative_input(self):
        self.history['host'][1]['driver']['input_events'] = []
        with self.assertRaisesRegex(AssertionError,'input unobserved'): self.check()


class GearBoarding(unittest.TestCase):
    def setUp(self):
        self.goal=[-4512,3008,208.03125]
        self.history,self.receipts={},[]
        events=[]
        for slot,role in enumerate(('host','guest')):
            rows=[]
            for index,(feet,grounded,jumps) in enumerate([(GEAR_APPROACH,True,2),
                    ([GEAR_APPROACH[0],GEAR_APPROACH[1],213],False,3),(self.goal,True,3)]):
                body=dict(feet=list(feet),grounded=grounded,jumps=jumps,health=89,spawned_at=0)
                rows.append(dict(tick=100*(index+1)+slot,players=[copy.deepcopy(body),copy.deepcopy(body)],
                    driver=dict(command=6+index,reached=index!=1,input_events=[])))
            self.history[role]=rows
            events.append(dict(kind='jump',epoch=1,tick=150+slot,input=[slot==0,slot==1],
                before=[[GEAR_APPROACH,True,2],[GEAR_APPROACH,True,2]],
                after=[[GEAR_APPROACH,False,3],[GEAR_APPROACH,False,3]]))
            self.receipts.append(dict(role=role,before_tick=100+slot,after_tick=200+slot,
                arrival_tick=300+slot,approach_command=6,command=7,arrival_command=8))
        self.history['host'][1]['driver']['input_events']=events

    def check(self): verify_gear_boarding(self.receipts,self.goal,self.history)

    def test_both_actual_jumps_and_arrivals(self): self.check()

    def test_missing_partner(self):
        self.receipts.pop()
        with self.assertRaisesRegex(AssertionError,'Missing personal'): self.check()

    def test_wrong_run_up_footing(self):
        self.history['guest'][0]['players'][1]['feet'][0]+=3
        with self.assertRaisesRegex(AssertionError,'started elsewhere'): self.check()

    def test_run_up_on_wrong_floor(self):
        self.history['guest'][0]['players'][1]['feet'][2]-=100
        with self.assertRaisesRegex(AssertionError,'started elsewhere'): self.check()

    def test_missing_host_takeoff(self):
        self.history['host'][1]['driver']['input_events'][1]['input'][1]=False
        with self.assertRaisesRegex(AssertionError,'takeoff unobserved'): self.check()

    def test_wrong_arrival_command(self):
        self.receipts[1]['arrival_command']+=100
        with self.assertRaisesRegex(AssertionError,'arrival unacknowledged'): self.check()


class GearExit(GearBoarding):
    def setUp(self):
        super().setUp()
        self.goal=[-4752,3008,208.03125]
        for rows in self.history.values():
            for body in rows[0]['players']: body['feet']=list(GEAR_EXIT_APPROACH)
            for body in rows[-1]['players']: body['feet']=list(self.goal)

    def check(self): verify_gear_exit(self.receipts,self.goal,self.history)


class Yielding(unittest.TestCase):
    def test_room_corner_can_yield_with_supported_body_clearance(self):
        candidates=[[-6321.2949,2547.0112,320.03125],[-6287.3540,2532.9524,320.03125],
                    [-6355.2358,2532.9524,320.03125]]
        self.assertEqual(yield_target(candidates,[-6321.2949,2499.0112,320.03125],
            [-6295.3403,2523.9585,320.03125],[-6304,2512,320.03125]),candidates[2])

    def test_blocked_body_can_retain_attempted_velocity(self):
        body = dict(feet=[-3919.305,2799.345,208],velocity=[39.46,-114.75,0],grounded=True)
        waiter = dict(feet=[-3953.348,2787.638,208],velocity=[0,0,0],grounded=True)
        self.assertTrue(partner_contact(body,waiter))

    def test_moving_partner_is_not_a_waiting_obstruction(self):
        body = dict(feet=[0,0,0],velocity=[100,0,0],grounded=True)
        waiter = dict(feet=[36,0,0],velocity=[30,0,0],grounded=True)
        self.assertFalse(partner_contact(body,waiter))

    def test_airborne_or_separate_bodies_do_not_trigger_yield(self):
        body = dict(feet=[0,0,0],velocity=[100,0,0],grounded=True)
        for feet,grounded in [([36,0,0],False),([36,0,60],True),([80,0,0],True)]:
            with self.subTest(feet=feet,grounded=grounded):
                self.assertFalse(partner_contact(body,dict(feet=feet,velocity=[0,0,0],grounded=grounded)))

    def test_recorded_step_six_contact_has_a_clear_supported_yield(self):
        candidates = [[-4001.0681,2787.666,208],[-3953.0681,2835.666,208],
            [-3953.0681,2739.666,208],[-3987.0093,2821.6072,208],[-3987.0093,2753.7249,208]]
        self.assertEqual(yield_target(candidates,[-3953.0681,2787.666,208],
            [-3919.2285,2799.9497,208],[-3920,2750,208],[-3937.9297,2760.5012,208]),candidates[3])

    def test_walking_across_the_follower_is_rejected(self):
        with self.assertRaisesRegex(AssertionError,'No observed'):
            yield_target([[-80,60,0]],[36,0,0],[0,0,0],[100,0,0])

    def test_parking_in_the_planned_approach_is_rejected(self):
        with self.assertRaisesRegex(AssertionError,'No observed'):
            yield_target([[80,10,0]],[36,0,0],[0,0,0],[100,0,0])

    def test_no_synthetic_neighbor_is_invented(self):
        with self.assertRaisesRegex(AssertionError,'No observed'):
            yield_target([],[36,0,0],[0,0,0],[100,0,0])


class YieldReceipts(unittest.TestCase):
    def setUp(self):
        body = dict(feet=[36,0,0],velocity=[0,0,0],health=89,spawned_at=0,grounded=True)
        incoming = dict(body,feet=[0,0,0])
        selected = dict(tick=100,players=[body,incoming],driver=dict(command=6,neighbors=[[[84,0,0]],[]]))
        arrived = copy.deepcopy(selected)
        arrived.update(tick=200)
        arrived['players'][0]['feet'] = [84,0,0]
        arrived['driver'].update(command=8,reached=True)
        paused = copy.deepcopy(selected);paused['driver']['command'] = 7
        self.history = dict(host=[selected,arrived],guest=[paused])
        self.receipts = [dict(step=0,role='host',follower='guest',goal=[84,0,0],via=[0,-50,0],
            selected_tick=100,candidates=[[84,0,0]],pause_command=7,paused_tick=100,
            tick=200,command=8,feet=[84,0,0])]

    def check(self): verify_yields(self.receipts,[dict(nav=[0,-100,0])],self.history)

    def test_observed_stop_and_yield(self): self.check()

    def test_unacknowledged_follower_pause(self):
        self.history['guest'][0]['driver']['command'] += 1
        with self.assertRaisesRegex(AssertionError,'pause not acknowledged'): self.check()

    def test_unobserved_neighbor(self):
        self.receipts[0]['candidates'] = []
        with self.assertRaisesRegex(AssertionError,'not observed'): self.check()

    def test_unacknowledged_yield_arrival(self):
        self.history['host'][-1]['driver']['reached'] = False
        with self.assertRaisesRegex(AssertionError,'arrival not acknowledged'): self.check()


class Parking(unittest.TestCase):
    def test_corner_detour_can_temporarily_remain_inside_arrival_radius(self):
        at=[-6239.9565,3350.7563,48.03125]
        goal=[-6208,3328,48.03125];incoming=[-6215.8867,3167.8276,48.03125]
        candidates=[[-6191.9565,3350.7563,48.03125],[-6239.9565,3302.7563,48.03125],
                    [-6206.0156,3316.8152,48.03125]]
        prior=[[-6210.9932,3321.53,48.03125]]
        point,ready=parking_target(candidates,at,incoming,goal,prior)
        self.assertEqual(point,candidates[1])
        self.assertFalse(ready)
        self.assertEqual(parking_target([[-6287.9565,3302.7563,48.03125]],point,incoming,goal,prior+[at]),
                         ([-6287.9565,3302.7563,48.03125],True))

    def test_detour_cannot_return_to_an_observed_position(self):
        with self.assertRaisesRegex(AssertionError,'No observed walking'):
            parking_target([[30,0,0]], [0,30,0], [-100,0,0], [0,0,0], [[30,1,0]])

    def test_detour_still_reserves_partner_space(self):
        with self.assertRaisesRegex(AssertionError,'No observed walking'):
            parking_target([[30,0,0]], [0,30,0], [35,0,0], [0,0,0])

    def test_short_forward_step_preferred_over_incoming_doorway(self):
        candidates = [[-3296.3,2705.6,192],[-3262.3,2787.5,192],[-3330.2,2787.5,192]]
        self.assertEqual(parking_target(candidates,[-3296.3,2753.6,192],[-3326,2989,192],[-3296,2750,192]),
                         (candidates[0],False))

    def test_far_side_preferred_over_incoming_doorway(self):
        candidates = [[-3295,2701,192], [-3261,2783,192], [-3329,2783,192]]
        self.assertEqual(parking_target(candidates, [-3295,2749,192], [-3331,2994,192], [-3296,2750,192]),
                         (candidates[0], True))

    def test_near_ring_is_only_an_approach(self):
        candidates = [[45, 0, 0], [20, 20, 0]]
        self.assertEqual(parking_target(candidates, [3,0,0], [-80,0,0], [0,0,0]), ([45,0,0], False))

    def test_second_ring_clears_arrival_lane(self):
        self.assertEqual(parking_target([[90,0,0]], [45,0,0], [-80,0,0], [0,0,0]), ([90,0,0], True))

    def test_no_forward_progress_is_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'No observed walking'):
            parking_target([[2,0,0]], [3,0,0], [-80,0,0], [0,0,0])

    def test_partner_space_remains_reserved(self):
        with self.assertRaisesRegex(AssertionError, 'No observed walking'):
            parking_target([[60,0,0]], [0,0,0], [80,0,0], [0,0,0])


if __name__ == '__main__': unittest.main()
