"""Reject revival credit without the intended ordinary lava death and survivor."""
import copy
import unittest

from test_coop_jlair1 import CuriosityPair
from test_coop_jlair1_recovery import CuriosityRecovery, DEATH_GOALS, verify_lava_revival, verify_survivor_motion


class CuriosityRecoveryEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.command = dict(id=7,goal=DEATH_GOALS['host'],horizontal=True,run=True,
                            navigate=False,jump=False,attack=False,capture=False,finish=False)
        self.starting = dict(host=[0,0,100],guest=[100,0,100])
        self.event = dict(player=0,epoch=1,death_tick=20,spawn_tick=1220,active_ticks=1200,health=25)
        def row(tick,health):
            return dict(tick=tick,driver=dict(command=7),players=[
                dict(feet=[0,100,0],health=health,spawned_at=0,grounded=False,immersion=dict(kind='Lava')),
                dict(feet=[100,0,100],health=100,spawned_at=0,grounded=True,immersion=dict(kind='Dry'))])
        self.history = {r:[row(10,1),row(21,0)] for r in ('host','guest')}

    def test_revival_requires_the_intended_ordinary_lava_walk(self):
        verify_lava_revival([self.event],self.history,'host',self.command,self.starting)
        bad = copy.deepcopy(self.history); bad['host'][0]['players'][0]['immersion']['kind']='Dry'
        with self.assertRaisesRegex(AssertionError,'walk into lava'):
            verify_lava_revival([self.event],bad,'host',self.command,self.starting)
        with self.assertRaisesRegex(AssertionError,'unexpected controls'):
            verify_lava_revival([self.event],self.history,'host',dict(self.command,attack=True),self.starting)
        for events in ([],[dict(self.event,player=1)],[self.event,self.event]):
            with self.assertRaisesRegex(AssertionError,'Wrong or repeated'):
                verify_lava_revival(events,self.history,'host',self.command,self.starting)

    def test_survivor_must_remain_alive_and_on_the_waiting_pad(self):
        for change in (dict(health=0),dict(spawned_at=5),dict(grounded=False),dict(feet=[200,0,100])):
            bad=copy.deepcopy(self.history);bad['guest'][1]['players'][1].update(change)
            with self.assertRaises(AssertionError):
                verify_lava_revival([self.event],bad,'host',self.command,self.starting)

    def test_clean_traversal_still_rejects_death_and_respawn(self):
        pair=CuriosityPair.__new__(CuriosityPair)
        report=dict(frame=dict(checkpoint_generation=0,players=[dict(stats=dict(sanity=100),spawned_at=0)]))
        pair.validate_lives('host',report)
        report['frame']['players'][0]['spawned_at']=20
        with self.assertRaisesRegex(AssertionError,'silently respawned'): pair.validate_lives('host',report)
        report['frame']['players'][0]['stats']['sanity']=0
        with self.assertRaisesRegex(AssertionError,'player died'): pair.validate_lives('host',report)

    def test_spectator_must_follow_an_actual_acknowledged_survivor_walk(self):
        event=dict(self.event,player=1)
        goal=[48,0,100]
        selected=dict(tick=30,epoch=1,driver=dict(neighbors=[[goal],[]]))
        arrived=dict(tick=80,epoch=1,map='jlair1',owner=None,
            driver=dict(command=5,reached=True,clear=[True,True]),
            players=[dict(feet=goal,health=100,grounded=True,spawned_at=0)])
        history=dict(host=[selected,arrived],guest=[dict(tick=30,camera_subject_feet=[0,0,100]),
            dict(tick=80,camera_subject_feet=goal)])
        receipt=dict(step=65,role='host',goal=goal,epoch=1,tick=80,feet=goal,command=5)
        entry=dict(role='host',selected_tick=30,goal=goal,arrival=receipt)
        verify_survivor_motion(entry,history,event)
        bad=copy.deepcopy(history);bad['guest'][1]['camera_subject_feet']=[0,0,100]
        with self.assertRaisesRegex(AssertionError,'moving survivor'): verify_survivor_motion(entry,bad,event)
        bad=copy.deepcopy(history);bad['host'][1]['driver']['command']=6
        with self.assertRaisesRegex(AssertionError,'Unacknowledged'): verify_survivor_motion(entry,bad,event)
        with self.assertRaisesRegex(AssertionError,'spectator interval'):
            verify_survivor_motion(entry,history,dict(event,spawn_tick=80))

    def test_targeted_revival_cannot_hide_survivor_death_or_team_wipe(self):
        pair=CuriosityRecovery.__new__(CuriosityRecovery);pair.survivor='guest'
        report=dict(frame=dict(checkpoint_generation=0,players=[dict(stats=dict(sanity=0),spawned_at=0),
            dict(stats=dict(sanity=100),spawned_at=0)]))
        pair.validate_lives('host',report)
        report['frame']['checkpoint_generation']=1
        with self.assertRaisesRegex(AssertionError,'team wipe'): pair.validate_lives('host',report)
        report['frame']['checkpoint_generation']=0;report['frame']['players'][1]['stats']['sanity']=0
        with self.assertRaisesRegex(AssertionError,'Survivor died'): pair.validate_lives('host',report)


if __name__ == '__main__': unittest.main()
