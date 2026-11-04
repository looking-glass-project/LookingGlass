"""Reject incomplete or unobserved Clockwork route receipts."""
import copy
import json
import unittest
from pathlib import Path
from test_coop_hatter1 import DESCENT, GEAR_APPROACH, GEAR_EXIT_APPROACH, parking_target, partner_contact, yield_target, verify_yields, verify_arrivals, verify_descent, verify_ledge_pulls, verify_gear_boarding, verify_gear_exit
from test_coop_hatter1 import recorded_arrival, next_stair_footing, verify_stair_waiting
from test_coop_hatter1 import next_corridor_footing, verify_corridor_waiting
from test_coop_hatter1 import validate_course_range, verify_gap_hold


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


class CorridorWaiting(unittest.TestCase):
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
