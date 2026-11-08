"""Require actual, separate player arrivals in the audible puzzle fixture."""
import copy
import unittest

from test_coop_audio import RESET_GOALS,verify_reset_approaches


def evidence():
    history={};entries=[]
    for index,role in enumerate(('host','guest')):
        players=[dict(feet=list(RESET_GOALS[r]),health=100,grounded=True) for r in ('host','guest')]
        history[role]=[dict(tick=10,epoch=1,map='fortress2',owner=None,players=players,
                            driver=dict(command=index+1,reached=True,clear=[True,True]))]
        entries.append(dict(role=role,goal=list(RESET_GOALS[role]),tick=10,command=index+1,feet=players[index]['feet']))
    return entries,history


class AudioApproachGuards(unittest.TestCase):
    def test_both_original_separated_approaches_are_required(self):
        entries,history=evidence();verify_reset_approaches(entries,history)
        with self.assertRaisesRegex(AssertionError,'Missing'):verify_reset_approaches(entries[:1],history)
        entries[1]['goal']=entries[0]['goal'][:]
        with self.assertRaisesRegex(AssertionError,'Changed'):verify_reset_approaches(entries,history)

    def test_rejects_unobserved_unsafe_or_missed_arrivals(self):
        for change in ('command','reached','clear','grounded','missed','scene'):
            entries,history=evidence();row=history['guest'][0]
            if change=='command':row['driver']['command']+=1
            elif change=='reached':row['driver']['reached']=False
            elif change=='clear':row['driver']['clear'][1]=False
            elif change=='grounded':row['players'][1]['grounded']=False
            elif change=='missed':row['players'][1]['feet'][1]+=15
            else:row['owner']='Blue'
            with self.assertRaises(AssertionError):verify_reset_approaches(entries,history)

    def test_rejects_partner_overlap_even_if_goal_was_reached(self):
        entries,history=evidence()
        history['host'][0]['players'][1]['feet']=copy.deepcopy(history['host'][0]['players'][0]['feet'])
        with self.assertRaisesRegex(AssertionError,'overlaps'):verify_reset_approaches(entries,history)


if __name__=='__main__':unittest.main()
