"""Reject audio evidence that follows the wrong body or omits recovery phases."""
import copy
import unittest

from test_coop_audio_listener import verify_listener,verify_listener_history


def row(role,dead=False,revived=False):
    local=('host','guest').index(role);subject=1-local if dead else local
    players=[dict(health=100.,swimming=False,submerged=False,spawned_at=0) for _ in range(2)]
    players[local].update(health=0. if dead else 25. if revived else 100.,
                          swimming=True,submerged=True,spawned_at=1200 if revived else 0)
    body=players[subject]
    return dict(role=role,pid=local+1,epoch=1,generation=0,tick=1300 if revived else 100 if dead else 10,
                camera_player=('Blue','Red')[subject],players=players,
                audio=dict(listener_player=('Blue','Red')[subject],listener_health=body['health'],
                           listener_swimming=body['swimming'],listener_submerged=body['submerged']))


class ListenerGuards(unittest.TestCase):
    def history(self):
        return {r:[row(r),row(r,dead=True),row(r,revived=True)] for r in ('host','guest')}

    def test_both_death_roles_follow_the_survivor_and_return(self):
        proof=verify_listener_history(self.history())
        self.assertEqual(set(proof),{'host','guest'})
        self.assertTrue(all(p['different_immersion']==1 for p in proof.values()))

    def test_dead_underwater_body_cannot_muffle_a_dry_survivor(self):
        for role in ('host','guest'):
            for field,value in [('listener_player','Blue' if role=='host' else 'Red'),
                                ('listener_health',0.),('listener_swimming',True),('listener_submerged',True)]:
                bad=row(role,dead=True);bad['audio'][field]=value
                with self.assertRaises(AssertionError):verify_listener(bad)

    def test_needs_both_clients_and_all_listener_phases(self):
        history=self.history()
        with self.assertRaisesRegex(AssertionError,'independent'):verify_listener_history(dict(host=history['host']))
        for missing in range(3):
            bad=copy.deepcopy(history);bad['guest'].pop(missing)
            # A revived row also covers ordinary alive listening. Drop both for that case.
            if missing==0:bad['guest']=[row('guest',dead=True)]
            with self.assertRaisesRegex(AssertionError,'coverage'):verify_listener_history(bad)

    def test_an_earlier_shared_wipe_cannot_stand_in_for_revival(self):
        for change in ('earlier_spawn','new_checkpoint'):
            history=self.history()
            if change=='earlier_spawn':history['guest'][2]['players'][1]['spawned_at']=50
            else:history['guest'][2]['generation']=1
            with self.assertRaisesRegex(AssertionError,'coverage'):verify_listener_history(history)


if __name__=='__main__':unittest.main()
