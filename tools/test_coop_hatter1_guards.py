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
from test_coop_hatter1 import next_door_footing, verify_door_waiting
from test_coop_hatter1 import verify_first_portal,verify_clock_portal,verify_clockwork_exit
from test_coop_hatter1 import chair_return,verify_chair_departures,verify_chair_hops
from test_coop_hatter1 import ClockworkPair,ChairDeath,verify_chair_retries


class ChairDeparture(unittest.TestCase):
    def setUp(self):
        self.steps=[{} for _ in range(139)];self.steps[138]=dict(nav=[0,0,100])
        self.arrivals=[dict(step=138,role=role,tick=tick) for role,tick in (('host',100),('guest',300))]
        self.receipts=[dict(step=138,role=role,approach_tick=start,selected_tick=start,before_tick=before,
            after_tick=flight,arrival_tick=end,goal=goal,command=cmd)
            for role,start,before,flight,end,goal,cmd in [('host',10,100,120,200,[-200,0,0],11),('guest',200,300,320,400,[200,0,0],21)]]
        self.history={}
        for role in ('host','guest'):
            rows=[]
            for tick in (10,100,120,200,300,320,400):
                players=[]
                for slot,start,flight,end,x in ((0,100,120,200,-200),(1,300,320,400,200)):
                    at=[x,0,0] if tick<start or tick>=end else [0,0,100 if tick<flight else 130]
                    players.append(dict(feet=at,health=100,grounded=tick!=flight,ledge=None,immersion=dict(level=0),
                        jumps=int(tick>=flight),warped_at=0,spawned_at=0))
                command=(10 if tick<120 else 11) if role=='host' else (20 if tick<320 else 21)
                events=[]
                for slot,t in ((0,110),(1,310)):
                    if tick>=t:
                        before=[[[0,0,100],True,0]]*2;after=copy.deepcopy(before);after[slot][1:]=[False,1]
                        events.append(dict(kind='jump',epoch=1,tick=t,input=[slot==0,slot==1],before=before,after=after))
                rows.append(dict(tick=tick,owner=None,players=players,clockwork=dict(cubes=[False,False,tick>=100,False]),
                    driver=dict(command=command,reached=tick in (100,200,300,400),clear=[True,True],input_events=events)))
            self.history[role]=rows

    def check(self):verify_chair_departures(self.receipts,self.steps,self.arrivals,self.history,[self.receipts[0]['step']])

    def test_both_players_jump_back_to_observed_dry_ground(self):self.check()

    def test_return_uses_the_approach_floor_before_climbing_the_chair_side(self):
        # A later supported point can be under an overhang. Returning to that
        # same XY from above leaves Alice on the roof, unable to reach its Z.
        side=copy.deepcopy(self.history['host'][0]);side['tick']=60
        side['players'][0]['feet']=[-100,0,40]
        self.history['host'].insert(1,side)
        self.check()

    def test_missing_partner_departure_is_rejected(self):
        self.receipts.pop()
        with self.assertRaisesRegex(AssertionError,'Missing personal'):self.check()

    def test_hazardous_return_footing_is_rejected(self):
        self.history['host'][0]['players'][0]['immersion']['level']=1
        with self.assertRaisesRegex(AssertionError,'no observed dry'):self.check()

    def test_partner_occupied_return_footing_is_rejected(self):
        self.history['host'][1]['players'][1]['feet']=[-200,0,0]
        with self.assertRaisesRegex(AssertionError,'no observed dry'):self.check()

    def test_an_unobserved_return_target_is_rejected(self):
        self.receipts[0]['goal']=[-210,0,0]
        with self.assertRaisesRegex(AssertionError,'not the observed'):self.check()

    def test_liquid_at_final_landing_is_rejected(self):
        self.history['host'][3]['players'][0]['immersion']['level']=1
        with self.assertRaisesRegex(AssertionError,'land safely outside liquid'):self.check()

    def test_landing_above_the_return_floor_is_rejected(self):
        self.history['host'][3]['players'][0]['feet']=[-200,0,33.31]
        with self.assertRaisesRegex(AssertionError,'return missed'):self.check()

    def test_missing_authoritative_takeoff_is_rejected(self):
        for row in self.history['host']:row['driver']['input_events']=[]
        with self.assertRaisesRegex(AssertionError,'Actual takeoff unobserved'):self.check()

    def test_a_repeated_jump_is_rejected(self):
        self.history['host'][3]['players'][0]['jumps']=2
        with self.assertRaisesRegex(AssertionError,'repeated the jump'):self.check()

    def test_a_recovery_warp_is_rejected(self):
        self.history['host'][3]['players'][0]['warped_at']=180
        with self.assertRaisesRegex(AssertionError,'scene, warp or revival'):self.check()

    def test_an_unreplicated_puzzle_contact_is_rejected(self):
        for row in self.history['guest']:row['clockwork']['cubes'][2]=False
        with self.assertRaisesRegex(AssertionError,'contact not replicated'):self.check()

    def projected(self):
        self.steps += [{},dict(nav=[0,0,100])]
        for r in self.receipts:
            r['step']=140;r['goal'][2]=33.31
        for r in self.arrivals:r['step']=140
        for rows in self.history.values():
            for row in rows:
                row['clockwork']['cubes']=[row['tick']>=100,False,False,False]
                row['driver']['return_landings']=[[[-200,0,33.31]],[[200,0,33.31]]]
                for slot,end in ((0,200),(1,400)):
                    if row['tick']>=end:row['players'][slot]['feet'][2]=33.31

    def test_projected_upper_floor_requires_its_own_actual_landing(self):
        self.projected();self.check()
        self.history['host'][3]['players'][0]['feet'][2]=0
        with self.assertRaisesRegex(AssertionError,'return missed'):self.check()

    def test_projected_return_requires_the_native_observation(self):
        self.projected()
        for row in self.history['host']:row['driver']['return_landings']=[[],[]]
        with self.assertRaisesRegex(AssertionError,'no observed dry'):self.check()

    def test_a_projected_point_far_from_the_actual_approach_is_rejected(self):
        self.projected()
        for row in self.history['host']:row['driver']['return_landings'][0]=[[-300,0,33.31]]
        with self.assertRaisesRegex(AssertionError,'no observed dry'):self.check()

    def shared_floor(self):
        self.projected()
        self.receipts[0].update(goal=[248,0,33.31],selected_player=1)
        for role,rows in self.history.items():
            for row in rows:
                if row['tick']>=200:row['players'][0]['feet']=[248,0,33.31]
                if role=='host':row['driver']['return_landings']=[[],[[248,0,33.31]]]

    def test_native_floor_near_partner_is_available_when_own_projection_is_missing(self):
        self.shared_floor();self.check()

    def test_shared_floor_source_must_be_recorded(self):
        self.shared_floor();del self.receipts[0]['selected_player']
        with self.assertRaisesRegex(AssertionError,'observed safe footing'):self.check()

    def test_shared_floor_still_requires_clearance_from_the_waiting_player(self):
        self.shared_floor()
        for row in self.history['host']:row['driver']['return_landings'][1]=[[232,0,33.31]]
        with self.assertRaisesRegex(AssertionError,'no observed dry'):self.check()


class ChairRecovery(unittest.TestCase):
    def setUp(self):
        self.events=[dict(player=1,epoch=1,death_tick=200,spawn_tick=1400)]
        self.attempts=[dict(step=141,role='guest',start_tick=100,previous_spawn=0,
            death_seen_tick=250,spawn_tick=1400,receipts=dict(arrivals=[dict(tick=190)]))]
        self.arrivals=[dict(step=141,role='guest',tick=1500)]
        self.departures=[dict(step=141,role='guest',approach_tick=1410,before_tick=1500)]
        self.history={'guest':[dict(tick=250,players=[{},dict(health=0,spawned_at=0)]),
            dict(tick=1500,players=[{},dict(health=25,spawned_at=1400)])]}

    def check(self):verify_chair_retries(self.attempts,self.events,self.arrivals,self.departures,self.history)

    def test_verified_revival_precedes_a_new_personal_approach(self):self.check()

    def test_unverified_revival_cannot_authorize_a_retry(self):
        self.events=[]
        with self.assertRaisesRegex(AssertionError,'own verified revival'):self.check()

    def test_spawning_at_partner_cannot_replace_a_new_approach(self):
        self.departures[0]['approach_tick']=100
        with self.assertRaisesRegex(AssertionError,'replaced the personal'):self.check()

    def test_one_death_cannot_authorize_two_attempts(self):
        self.attempts*=2
        with self.assertRaisesRegex(AssertionError,'own verified revival'):self.check()

    def test_unaccounted_death_is_not_hidden(self):
        self.events.append(dict(player=0,epoch=1,death_tick=300,spawn_tick=1500))
        with self.assertRaisesRegex(AssertionError,'outside a recorded'):self.check()

    def test_retry_requires_an_observed_dead_body(self):
        self.history['guest'][0]['players'][1]['health']=1
        with self.assertRaisesRegex(AssertionError,'original dead body'):self.check()

    def test_runtime_archives_failed_receipts_and_repeats_the_original_goal(self):
        for role,slot in (('host',0),('guest',1)):
            with self.subTest(role=role):
                pair=ClockworkPair.__new__(ClockworkPair)
                pair.step=141;pair.allow_revival=True;pair.serial=0;pair.lead=.2;pair.commands={}
                pair.evidence={k:[] for k in ('arrivals','ledge_pulls','chair_departures','yields','failed_chair_attempts')}
                goal=[-608,2272,212.03125];calls=[]
                bodies=[dict(stats=dict(sanity=40),spawned_at=0,player=dict(feet=[0,0,64])) for _ in range(2)]
                pair.reports={r:dict(frame=dict(tick=100,players=bodies),route_driver=dict(command=0)) for r in ('host','guest')}
                def move(who,at,navigate,yield_partner=None):
                    self.assertEqual((who,at),(role,goal));calls.append(at)
                    pair.set(role,at,navigate=navigate)
                    pair.reports[role]['route_driver']['command']=pair.commands[role]['id']
                    if len(calls)==1:
                        pair.evidence['arrivals'].append(dict(step=141,role=role,tick=190))
                        pair.evidence['chair_departures'].append(dict(step=141,role=role,before_tick=190))
                        bodies[slot]['stats']['sanity']=0;pair.reports[role]['frame']['tick']=200
                        raise ChairDeath(role)
                    self.assertFalse(pair.evidence['arrivals'],'Respawn was recorded as a personal arrival')
                    self.assertEqual(bodies[slot]['player']['feet'],[50,0,64])
                    pair.reports[role]['frame']['tick']=1500;bodies[slot]['player']['feet']=at
                def until(predicate,seconds):
                    self.assertIsNone(pair.commands[role]['goal']);self.assertFalse(pair.commands[role]['attack'])
                    self.assertTrue(pair.commands['host']['recovery_trace']);self.assertFalse(pair.commands['guest']['recovery_trace'])
                    self.assertFalse(predicate())
                    bodies[slot].update(spawned_at=1400,stats=dict(sanity=25),player=dict(feet=[50,0,64]))
                    for report in pair.reports.values():report['frame']['tick']=1400
                    self.assertTrue(predicate())
                def leave(who,at,start):
                    self.assertEqual(start,1400)
                    pair.evidence['chair_departures'].append(dict(step=141,role=who,approach_tick=start,before_tick=1500))
                pair.move=move;pair.until=until;pair.leave_chair=leave
                pair.traverse_chair(role,goal,True,None)
                self.assertEqual(len(calls),2);self.assertIsNone(pair.moving_role)
                self.assertEqual(len(pair.evidence['arrivals']),1)
                failed=pair.evidence['failed_chair_attempts'][0]
                self.assertEqual(failed['receipts']['arrivals'][0]['tick'],190)
                self.assertEqual(failed['spawn_tick'],1400)


class LedgeHandoffCommands(unittest.TestCase):
    def test_only_planned_chair_approaches_wait_for_an_explicit_pullup(self):
        pair=ClockworkPair.__new__(ClockworkPair)
        pair.serial=0;pair.commands={};pair.lead=.17
        for role in ('host','guest'):
            for step in (-1,0,136,137,138,139,140,141,142):
                pair.step=step
                pair.set(role,[0,0,64],navigate=True)
                self.assertEqual(pair.commands[role]['ledge_handoff'],step in (138,139,140,141))
                self.assertEqual(pair.commands[role]['landing_feedback'],step in (140,141))
                pair.set(role,[0,0,64],jump=True)
                self.assertFalse(pair.commands[role]['ledge_handoff'])
                self.assertFalse(pair.commands[role]['landing_feedback'])
                pair.set(role,attack=False)
                self.assertFalse(pair.commands[role]['ledge_handoff'])

    def test_completed_pull_resumes_routing_and_combat_before_arrival(self):
        for role,slot,step in ((r,s,n) for r,s in (('host',0),('guest',1)) for n in (138,140)):
            with self.subTest(role=role,step=step):
                pair=ClockworkPair.__new__(ClockworkPair)
                pair.serial=0;pair.commands={};pair.lead=.2;pair.step=step
                pair.evidence=dict(ledge_pulls=[]);pair.history=dict(host=[],guest=[])
                pair.reports={};goal=[100,0,100]
                def observe(tick,climbs,grounded,ledge,at,reached=False):
                    body=dict(feet=at,climbs=climbs,grounded=grounded,ledge=ledge,jumps=int(step==140 and tick>=500),
                        immersion=dict(level=0),health=100,warped_at=0,spawned_at=0)
                    partner=dict(feet=[-100,0,100],climbs=1,grounded=True,ledge=None)
                    players=[partner,partner];players[slot]=body
                    driver=dict(command=pair.commands.get(role,{}).get('id',0),reached=reached,
                        ledge_pull_clear=[True,True],clear=[True,True],input_events=[])
                    if tick>=200:
                        driver['input_events']=[dict(kind='jump',epoch=1,tick=110,input=[slot==0,slot==1],
                            before=[[[0,0,0],False,0]]*2,after=[[[0,0,0],False,0]]*2)]
                    if step==140 and tick>=500:
                        before=[[[36,0,100],True,0]]*2;after=copy.deepcopy(before);after[slot][1:]=[False,1]
                        driver['input_events'].append(dict(kind='jump',epoch=1,tick=450,input=[slot==0,slot==1],before=before,after=after))
                    row=dict(tick=tick,epoch=1,owner=None,players=players,driver=driver)
                    for peer in pair.history:pair.history[peer].append(copy.deepcopy(row))
                    pair.reports[role]=dict(frame=dict(tick=tick,epoch=1,players=[dict(player=p) for p in players]),
                        route_driver=driver,combat=dict(stage=dict(owner=None)))
                def until(predicate,seconds=90):
                    self.assertFalse(predicate())
                    pull_command=pair.commands[role]['id']
                    self.assertTrue(pair.commands[role]['jump'])
                    self.assertFalse(pair.commands[role]['attack'])
                    observe(200,0,False,dict(pulling=True),[0,0,70])
                    self.assertFalse(predicate())
                    hold_command=pull_command
                    if step==140:
                        held=pair.commands[role];hold_command=held['id']
                        self.assertGreater(hold_command,pull_command)
                        self.assertIsNone(held['goal'])
                        self.assertFalse(held['navigate'] or held['jump'] or held['attack'])
                        observe(250,0,False,dict(pulling=True),[0,0,80])
                        self.assertFalse(predicate())
                        self.assertEqual(pair.evidence['ledge_pulls'][0]['hold_ack_tick'],250)
                    # Grounding or the partner's climb cannot complete this pull.
                    observe(300,0,True,None,[36,0,100])
                    self.assertFalse(predicate())
                    self.assertEqual(pair.commands[role]['id'],hold_command)
                    self.assertNotIn('after_tick',pair.evidence['ledge_pulls'][0])
                    observe(400,1,True,None,[36,0,100])
                    self.assertFalse(predicate())
                    resumed=pair.commands[role]
                    self.assertGreater(resumed['id'],pull_command)
                    self.assertEqual(resumed['goal'],goal)
                    if step==140:
                        self.assertTrue(resumed['jump'])
                        self.assertFalse(resumed['navigate'] or resumed['attack'])
                        observe(500,1,False,None,[80,0,180])
                        self.assertFalse(predicate())
                    else:
                        self.assertTrue(resumed['navigate'] and resumed['attack'])
                        self.assertFalse(resumed['jump'])
                    self.assertEqual(resumed['arrival_radius'],6)
                    self.assertEqual(pair.evidence['ledge_pulls'][0]['after_tick'],400)
                    observe(600,1,True,None,goal,reached=True)
                    self.assertTrue(predicate())
                pair.until=until
                observe(100,0,False,dict(pulling=False,top=100,forward=[1,0]),[0,0,40])
                command=pair.move(role,goal,navigate=True,radius=6)
                self.assertEqual(command,pair.commands[role]['id'])
                self.assertEqual(pair.evidence['ledge_pulls'][0]['after_tick'],400)

                if step==140:
                    steps=[{} for _ in range(141)];steps[140]=dict(nav=goal)
                    verify_chair_hops(pair.evidence['ledge_pulls'],steps,[dict(step=140,role=role,tick=600,command=command)],pair.history)


class ChairRimHop(unittest.TestCase):
    def setUp(self):
        # The pulled-up rim and goal from the run that walked into slime before
        # reaching the cushion. A new hop must prove flight above that gap.
        self.goal=[-928,2264,212.03125]
        self.steps=[{} for _ in range(141)];self.steps[140]=dict(nav=self.goal)
        self.hop=dict(role='guest',before_tick=100,after_tick=150,arrival_tick=250,command=14,goal=self.goal)
        self.pulls=[dict(step=140,role='guest',after_tick=100,chair_hop=self.hop)]
        self.arrivals=[dict(step=140,role='guest',tick=250,command=14)]
        self.history={}
        for role in ('host','guest'):
            rows=[]
            for tick,at in ((100,[-855.11,2223,224.15625]),(150,[-887,2240,290]),(250,[-920.625,2264.687,225.365])):
                red=dict(feet=at,grounded=tick!=150,ledge=None,jumps=12+int(tick>=150),
                    health=20.53,immersion=dict(level=0),warped_at=184476,spawned_at=0)
                blue=dict(red,feet=[-842,2455,64],grounded=True)
                events=[]
                if tick>=150:
                    events=[dict(kind='jump',epoch=1,tick=120,input=[False,True],
                        before=[[[0,0,0],True,12]]*2,after=[[[0,0,0],True,12],[[0,0,0],False,13]])]
                rows.append(dict(tick=tick,owner=None,players=[blue,red],
                    driver=dict(clear=[True,True],command=14,input_events=events)))
            self.history[role]=rows

    def check(self):verify_chair_hops(self.pulls,self.steps,self.arrivals,self.history)

    def test_actual_hop_from_the_recorded_rim_to_the_cushion(self):self.check()

    def test_old_walk_after_pull_cannot_replace_the_hop(self):
        del self.pulls[0]['chair_hop']
        with self.assertRaisesRegex(AssertionError,'own observed hop'):self.check()

    def test_no_airborne_observation_rejects_the_hop(self):
        self.history['guest'][1]['players'][1]['grounded']=True
        with self.assertRaisesRegex(AssertionError,'flight unobserved'):self.check()

    def test_a_hop_that_enters_slime_is_rejected(self):
        self.history['guest'][1]['players'][1]['immersion']['level']=1
        with self.assertRaisesRegex(AssertionError,'entered liquid'):self.check()


class FirstPortal(unittest.TestCase):
    def setUp(self):
        self.steps=json.loads((Path(__file__).resolve().parents[1]/'src/levels/hatter1/route_steps.json').read_text())
        self.entities=[{} for _ in range(132)]
        self.entities[45]=dict(classname='trigger_teleport',target='tel3',origin='-2720 2272 320')
        self.entities[131]=dict(classname='func_teleportdest',targetname='tel3',origin='-768 3232 72')
        goal=self.steps[136]['nav'];park=[-720,3232,64]
        self.evidence=dict(portal_jumps=[dict(role='host',before_tick=100,command=10,warp=130,arrival_tick=150),
            dict(role='guest',before_tick=300,command=20,warp=330,arrival_tick=350)],
            portal_waiting=[dict(role='host',selected_tick=150,tick=200,goal=park,candidates=[park],feet=park,command=11)])
        events=[]
        for slot,tick in ((0,110),(1,310)):
            before=[[goal,True,0] for _ in range(2)];after=copy.deepcopy(before);after[slot][1:]=[False,1]
            events.append(dict(kind='jump',tick=tick,input=[slot==0,slot==1],before=before,after=after))
        self.history={}
        for role in ('host','guest'):
            rows=[]
            for tick in (100,120,132,150,200,300,320,332,350):
                bodies=[]
                for slot,warp in ((0,130),(1,330)):
                    at=list(goal);grounded=True
                    if tick<warp and tick>=(120 if slot==0 else 320):at=[-2740,2272,280];grounded=False
                    if tick>=warp:at=[-768,3232,64 if tick>=(150 if slot==0 else 350) else 72];grounded=tick>=(150 if slot==0 else 350)
                    if slot==0 and tick>=200:at=park
                    bodies.append(dict(feet=at,grounded=grounded,spawned_at=0,warped_at=warp if tick>=warp else 5,health=100,climbs=0))
                command=(10 if tick<200 else 11) if role=='host' else (20 if tick>=300 else 0)
                rows.append(dict(tick=tick,owner=None,players=copy.deepcopy(bodies),driver=dict(command=command,
                    reached=tick==200,clear=[True,True],neighbors=[[park],[]],input_events=[e for e in events if e['tick']<=tick])))
            self.history[role]=rows

    def check(self):verify_first_portal(self.evidence,self.steps,self.history,self.entities)

    def test_two_actual_jumps_and_replicated_personal_warps(self):self.check()

    def climb(self):
        for rows in self.history.values():
            for row in rows:
                for event in row['driver']['input_events']:
                    slot=0 if event['input'][0] else 1
                    event['after'][slot][2]=event['before'][slot][2]
                    event['after'][slot][0][2]=event['before'][slot][0][2]+2
                for slot,warp in ((0,130),(1,330)):
                    if warp-20<=row['tick']<warp:row['players'][slot]['climbs']=1

    def test_native_climb_from_the_same_jump_input_is_a_takeoff(self):
        self.climb();self.check()

    def test_upward_motion_without_the_native_climb_is_not_a_takeoff(self):
        self.climb()
        for row in self.history['host']:row['players'][0]['climbs']=0
        with self.assertRaisesRegex(AssertionError,'takeoff unobserved'):self.check()

    def test_a_climb_without_jump_input_is_not_a_takeoff(self):
        self.climb()
        for row in self.history['host']:row['driver']['input_events']=[]
        with self.assertRaisesRegex(AssertionError,'takeoff unobserved'):self.check()

    def test_a_jump_must_not_replay_at_the_destination(self):
        event=copy.deepcopy(self.history['host'][1]['driver']['input_events'][0]);event['tick']=130
        self.history['host'][3]['driver']['input_events'].append(event)
        with self.assertRaisesRegex(AssertionError,'replayed at the destination'):self.check()

    def test_partial_portal_sequence_is_not_supported(self):
        for start,last in ((136,136),(137,137)):
            with self.subTest(start=start),self.assertRaisesRegex(AssertionError,'complete 136-137'):
                validate_course_range(self.steps,start,last)

    def test_changed_portal_input_requires_review(self):
        self.steps[137]['input'][4]=0
        with self.assertRaisesRegex(AssertionError,'changed portal jump'):self.check()

    def test_a_warp_without_jump_is_not_a_crossing(self):
        for row in self.history['host']:row['driver']['input_events']=[]
        with self.assertRaisesRegex(AssertionError,'takeoff unobserved'):self.check()

    def test_both_clients_must_observe_the_same_warp(self):
        self.history['guest'][2]['players'][0]['warped_at']=131
        with self.assertRaisesRegex(AssertionError,'warp stamp'):self.check()

    def test_wrong_destination_is_rejected(self):
        self.history['guest'][2]['players'][0]['feet']=[0,0,72]
        with self.assertRaisesRegex(AssertionError,'wrong destination'):self.check()

    def test_scene_cannot_replace_portal_contact(self):
        self.history['host'][2]['owner']='Blue'
        with self.assertRaisesRegex(AssertionError,'revival or a scene'):self.check()

    def test_waiter_must_leave_the_destination_clear(self):
        at=[-768,3232,64];receipt=self.evidence['portal_waiting'][0]
        receipt.update(goal=at,candidates=[at],feet=at)
        self.history['host'][3]['driver']['neighbors'][0]=[at]
        self.history['host'][4]['players'][0]['feet']=at
        with self.assertRaisesRegex(AssertionError,'obstructs the arrival'):self.check()


class ClockPortal(unittest.TestCase):
    def setUp(self):
        source=FirstPortal();source.setUp()
        self.steps=source.steps;self.entities=source.entities
        self.evidence=dict(clock_portal_jumps=source.evidence['portal_jumps'],clock_portal_waiting=source.evidence['portal_waiting'])
        self.history=source.history
        self.entities[125]=dict(classname='trigger_teleport',targetname='clock_teleport',target='end',origin='-768 1892 544')
        self.entities[124]=dict(classname='func_teleportdest',targetname='end',origin='-2144 2272 192')
        takeoff=self.steps[142]['nav'];park=[-2096,2272,192]
        receipt=self.evidence['clock_portal_waiting'][0];receipt.update(goal=park,candidates=[park],feet=park)
        for rows in self.history.values():
            for row in rows:
                row['clockwork']=dict(cubes=[True]*4,clock=1.,age=10.)
                row['driver']['neighbors']=[[park],[]]
                for slot,body in enumerate(row['players']):
                    old=body['feet']
                    if old[0]==-2800:body['feet']=list(takeoff)
                    elif old[0]==-2740:body['feet']=[-768,1980,150]
                    elif old[0]==-768:body['feet']=[-2144,2272,192 if body['grounded'] else 200]
                    else:body['feet']=park
                    if slot==1:body['spawned_at']=80
                for event in row['driver']['input_events']:
                    for b in event['before']+event['after']:b[0]=list(takeoff)

    def check(self):return verify_clock_portal(self.evidence,self.steps,self.history,self.entities)

    def test_lowered_clock_has_two_personal_crossings_after_an_earlier_revival(self):
        self.assertEqual(self.check(),{(1,0,130),(1,1,330)})

    def test_unfinished_puzzle_and_lowering_are_rejected(self):
        for mode in ('chair','clock','time'):
            self.setUp();state=self.history['host'][0]['clockwork']
            if mode=='chair':state['cubes'][3]=False
            elif mode=='clock':state['clock']=None
            else:state['age']=5.9
            with self.subTest(mode=mode),self.assertRaisesRegex(AssertionError,'before puzzle lowering'):self.check()

    def test_original_high_portal_position_is_not_the_lowered_contact(self):
        self.entities[125]['origin']='-768 1892 900'
        with self.assertRaisesRegex(AssertionError,'lowered clock portal position'):self.check()

    def test_rescue_teleport_cannot_replace_source_contact(self):
        self.history['host'][1]['players'][0]['feet']=[0,0,0]
        with self.assertRaisesRegex(AssertionError,'source contact'):self.check()

    def test_revival_cannot_replace_second_portal(self):
        self.history['guest'][7]['players'][1]['spawned_at']=330
        with self.assertRaisesRegex(AssertionError,'revival or a scene'):self.check()

    def test_unobserved_takeoff_cannot_authorize_warp(self):
        for row in self.history['host']:row['driver']['input_events']=[]
        with self.assertRaisesRegex(AssertionError,'takeoff unobserved'):self.check()

    def test_wrong_destination_is_rejected(self):
        self.history['guest'][2]['players'][0]['feet']=[-768,3232,64]
        with self.assertRaisesRegex(AssertionError,'wrong destination'):self.check()

    def test_waiting_handoff_must_precede_second_crossing(self):
        self.evidence['clock_portal_waiting'][0]['tick']=320
        with self.assertRaisesRegex(AssertionError,'out of order'):self.check()

    def test_already_clear_dry_arrival_can_be_held_without_an_extra_step(self):
        receipt=self.evidence['clock_portal_waiting'][0];receipt['source']='arrival'
        body=self.history['host'][3]['players'][0]
        body.update(feet=receipt['goal'],immersion=dict(level=0))
        self.check()
        body['feet']=[-2140,2272,192]
        with self.assertRaisesRegex(AssertionError,'dry supported arrival'):self.check()

    def test_occupied_arrival_cannot_be_used_as_a_waiting_point(self):
        receipt=self.evidence['clock_portal_waiting'][0]
        receipt.update(source='arrival',goal=[-2140,2272,192])
        self.history['host'][3]['players'][0].update(feet=receipt['goal'],immersion=dict(level=0))
        with self.assertRaisesRegex(AssertionError,'insufficient waiting clearance'):self.check()

    def test_original_onward_waypoint_can_clear_a_small_portal_platform(self):
        receipt=self.evidence['clock_portal_waiting'][0];goal=self.steps[145]['nav']
        receipt.update(source='onward',goal=goal,feet=goal)
        self.history['host'][4]['players'][0]['feet']=goal
        self.check()
        receipt['goal']=[-2480,2400,256]
        with self.assertRaisesRegex(AssertionError,'original onward waypoint'):self.check()


class ClockworkExit(unittest.TestCase):
    def setUp(self):
        self.steps=json.loads((Path(__file__).resolve().parents[1]/'src/levels/hatter1/route_steps.json').read_text())
        self.entry=dict(role='host',approach_tick=100,approach_command=7,exit_command=8,arrival_ticks=dict(host=200,guest=200))
        players=[dict(feet=list(self.steps[148]['nav']) if i==0 else [-2500,2300,256],health=25+i,
            grounded=True,spawned_at=80 if i else 0,stats=dict(sanity=25+i,will=30+i,difficulty='Normal',selected=i,
                owned=[1,i],collected=[str(i)],school_items={},staff_component=False)) for i in range(2)]
        clock=dict(stopped=True,levers=[1]*5,cubes=[True]*4,hare=1,port=1,gryphon=1)
        approach=dict(map='hatter1',epoch=1,tick=100,owner=None,players=players,clockwork=clock,
                      driver=dict(command=7,reached=True,clear=[True,True]))
        contact=copy.deepcopy(approach);contact.update(tick=120);contact['players'][0]['feet']=[-2528,1780,192];contact['driver']['command']=8
        arrival=copy.deepcopy(contact);arrival.update(map='hatter2',epoch=2,tick=200,visit_entry=None,checkpoint_generation=0)
        for i,b in enumerate(arrival['players']):b['feet']=[5664+i*48,-1088,0]
        arrival['driver']['network_states']=[dict(epoch=2,tick=t,sha256=str(t),world_sha256='w'+str(t),native_sha256='n'+str(t)) for t in range(180,200)]
        self.history={r:copy.deepcopy([approach,contact,arrival]) for r in ('host','guest')}

    def check(self):return verify_clockwork_exit(self.entry,self.steps,self.history)

    def test_first_entrant_transitions_both_players_with_individual_resources(self):
        self.assertEqual(self.check(),20)

    def test_closed_clock_cannot_authorize_an_exit(self):
        self.history['host'][1]['clockwork']['stopped']=False
        with self.assertRaisesRegex(AssertionError,'bypassed Clockwork'):self.check()

    def test_exit_needs_its_own_ordinary_approach_and_contact(self):
        self.history['host'][0]['players'][0]['feet']=[0,0,0]
        with self.assertRaisesRegex(AssertionError,'personally reached'):self.check()
        self.setUp();self.history['guest'][1]['players'][0]['feet']=[0,0,0]
        with self.assertRaisesRegex(AssertionError,'contact was not approached'):self.check()

    def test_partner_must_arrive_in_about_face(self):
        self.history['guest'][-1]['map']='other'
        with self.assertRaisesRegex(AssertionError,'shared About Face'):self.check()

    def test_transition_cannot_refill_health_or_replace_personal_inventory(self):
        self.history['host'][-1]['players'][1]['health']=100
        with self.assertRaisesRegex(AssertionError,'reset a player'):self.check()
        self.setUp();self.history['guest'][-1]['players'][1]['stats']['owned']=[9,9]
        with self.assertRaisesRegex(AssertionError,'personal owned'):self.check()

    def test_new_map_requires_matching_native_states(self):
        self.history['guest'][-1]['driver']['network_states'][0]['native_sha256']='wrong'
        with self.assertRaisesRegex(AssertionError,'replication incomplete'):self.check()

    def test_exit_cannot_refill_will(self):
        self.history['guest'][-1]['players'][1]['stats']['will']=100
        with self.assertRaisesRegex(AssertionError,'personal Will'):self.check()

    def test_final_approach_requires_the_shared_exit_sequence(self):
        for start,last in ((146,148),(148,148),(149,149)):
            with self.subTest(start=start,last=last),self.assertRaisesRegex(AssertionError,'complete 148-149'):
                validate_course_range(self.steps,start,last)


class DoorWaiting(unittest.TestCase):
    def setUp(self):
        self.steps=[{} for _ in range(131)]
        self.steps[128]={'walk':[-3508,2272,192.03125]}
        self.steps[129]={'wait':1.7};self.steps[130]={'walk':[-3352,2272,192.03125]}
        self.entities=[{'classname':'info_pathnode'} for _ in range(6)]
        for i in (4,5):self.entities[i]={'classname':'func_rotatingdoor'}
        self.arrivals=[dict(step=128,role='host',tick=90),dict(step=128,role='guest',tick=600)]
        self.receipt=dict(step=128,role='host',goal=self.steps[130]['walk'],before_tick=304,tick=500,
            command=9,feet=self.steps[130]['walk'],opening_hold=dict(start_tick=100,end_tick=304))
        self.history={'host':[]}
        for tick,point in ((100,self.steps[128]['walk']),(304,self.steps[128]['walk']),(500,self.steps[130]['walk'])):
            self.history['host'].append(dict(tick=tick,players=[dict(feet=point,health=100,grounded=True,spawned_at=0,warped_at=50)],
                driver=dict(command=9,reached=True),clockwork=dict(doors=[1.,1.])))

    def check(self): verify_door_waiting([self.receipt],self.steps,self.arrivals,self.history,self.entities)

    def test_wait_then_walk_clear_before_follower_arrives(self): self.check()

    def test_authored_wait_cannot_be_removed(self):
        self.steps[129]={'lever':3}
        with self.assertRaisesRegex(AssertionError,'opening wait'): self.check()

    def test_short_opening_wait(self):
        self.receipt['opening_hold']['start_tick']=101;self.history['host'][0]['tick']=101
        with self.assertRaisesRegex(AssertionError,'too short'): self.check()

    def test_opening_must_follow_personal_arrival(self):
        self.arrivals[0]['tick']=101
        with self.assertRaisesRegex(AssertionError,'preceded arrival'): self.check()

    def test_both_leaves_must_open(self):
        self.history['host'][1]['clockwork']['doors'][1]=.9
        with self.assertRaisesRegex(AssertionError,'both leaves'): self.check()

    def test_waiter_must_stay_on_the_approach(self):
        self.history['host'][0]['players'][0]['feet']=[-3600,2272,192.03125]
        with self.assertRaisesRegex(AssertionError,'left its footing'): self.check()

    def test_reunion_is_not_a_door_crossing(self):
        self.history['host'][1]['players'][0]['warped_at']=250
        with self.assertRaisesRegex(AssertionError,'warped'): self.check()

    def test_handoff_cannot_skip_arbitrary_route_sections(self):
        with self.assertRaisesRegex(AssertionError,'limited'): next_door_footing(self.steps,127)


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

    def stop(self):
        self.entry.update(step=146,scene='Hatter1_End')
        self.levers[0].update(step=146,lever=4)
        for peer,rows in self.history.items():
            for row,t in zip(rows[:-1],(0,15,29.6)):
                row.update(tick=120+round(t*120),scene='Hatter1_End')
                row['clockwork']=dict(stopped=False,coop=dict(actor=0),scene=dict(kind='Stop',time=t,line=0,skip=None))
            rows[-1]['tick']=3730;rows[-1]['clockwork']=dict(stopped=True,scene=None)
            for body in rows[-1]['players']:body['warped_at']=3720
            self.entry['ends'][peer]=3730

    def test_watched_stop_and_shared_reunion(self):
        self.stop();self.check()

    def test_false_stop_flag_cannot_claim_completion(self):
        self.stop();self.history['guest'][-1]['clockwork']['stopped']=False
        with self.assertRaisesRegex(AssertionError,'Missing completed scene'):self.check()

    def test_stop_requires_its_full_thirty_seconds(self):
        self.stop()
        for peer,rows in self.history.items():
            rows[2]['tick']=2400;rows[2]['clockwork']['scene']['time']=19
            rows[-1]['tick']=2530
            for body in rows[-1]['players']:body['warped_at']=2520
            self.entry['ends'][peer]=2530
        with self.assertRaisesRegex(AssertionError,'Stop scene ended early'):self.check()

    def test_stop_belongs_to_the_final_lever_user(self):
        self.stop();self.levers[0]['role']='guest'
        with self.assertRaisesRegex(AssertionError,'lever user'):self.check()

    def test_stop_partner_must_remain_hidden(self):
        self.stop();self.history['guest'][1]['presentation']['visible_players'][1]=True
        with self.assertRaisesRegex(AssertionError,'displayed'):self.check()

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

    def test_exact_verified_spawn_allows_an_independent_later_arrival(self):
        self.history['guest'][0]['players'][1]['spawned_at']=99
        verify_arrivals(self.steps,[0,1],self.receipts,self.history,revivals=[dict(player=1,epoch=1,spawn_tick=99)])

    def test_other_players_revival_cannot_authorize_this_arrival(self):
        self.history['guest'][0]['players'][1]['spawned_at']=99
        with self.assertRaisesRegex(AssertionError,'Invalid arrival'):
            verify_arrivals(self.steps,[0,1],self.receipts,self.history,revivals=[dict(player=0,epoch=1,spawn_tick=99)])

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

    def test_chair_pull_requires_its_own_clear_native_destination(self):
        self.pulls[0]['requires_clear']=True
        self.history['guest'][0]['driver']['ledge_pull_clear']=[True,False]
        with self.assertRaisesRegex(AssertionError,'destination was not clear'):self.check()
        self.history['guest'][0]['driver']['ledge_pull_clear']=[False,True]
        self.check()

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
    def test_clock_approach_uses_an_observed_ready_side_point_before_the_forward_pocket(self):
        candidates=[[-771.6982421875,2110.91796875,64.03125],
            [-771.6982421875,2014.91796875,64.03125],[-737.7571411132812,2096.859130859375,64.03125]]
        at=[-771.6982421875,2062.91796875,64.03125]
        incoming=[-895.1575317382812,2450.95361328125,64.03125];goal=[-768,2056,64.03125]
        self.assertEqual(parking_target(candidates,at,incoming,goal),(candidates[1],False))
        self.assertEqual(parking_target(candidates,at,incoming,goal,prefer_forward=False),(candidates[2],True))

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
