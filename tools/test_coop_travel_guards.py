"""Ensure loading-rejoin evidence rejects a missed barrier or lost player state."""
import copy
import unittest

from coop_travel_rejoin import verify_barrier, verify_waiting_arrival, verify_guest_arrival


def reports():
    bodies = []
    for i in range(2):
        bodies.append(dict(stats=dict(sanity=83-i*36, will=20-i*10, owned=[1,1,0,0,0,0,i+1,0,0,0],
            collected=[str(i)], selected=0, school_items={}, staff_component=False,
            difficulty='Normal', loot={}), spawned_at=0, warped_at=0,
            player=dict(feet=[i*48,0,0])))
    before = dict(role='host', pid=1, build='fixture', session_phase='Loading', playing=False,
        can_load=False, visit=dict(epoch=1, map='skool1', entry=None),
        pending_visit=dict(epoch=2, map='skool1', entry='skool1_start2'), completed_visits=0,
        frame=dict(epoch=1, tick=1440, checkpoint_generation=0, players=bodies,
                   lives=dict(states=['Alive','Alive'])), combat=dict(door='open'))
    waiting = copy.deepcopy(before)
    waiting.update(session_phase='Waiting', can_load=True, visit=before['pending_visit'].copy(),
                   pending_visit=None, completed_visits=1)
    waiting['frame']['epoch'] = 2
    for body in waiting['frame']['players']:
        body['warped_at'] = 1440
        body['stats']['owned'][2] = 1
    guest = copy.deepcopy(waiting)
    guest.update(role='guest', pid=3, playing=True, session_phase='Playing')
    guest['frame']['tick'] += 12
    return before, waiting, guest


class TravelGuards(unittest.TestCase):
    def test_real_barrier_and_fresh_guest_keep_both_resources(self):
        before, waiting, guest = reports()
        verify_barrier(before, copy.deepcopy(before), 2)
        verify_waiting_arrival(before, waiting)
        self.assertEqual(verify_guest_arrival(waiting, guest, 2)['epoch'], 2)

    def test_after_arrival_is_not_loading_barrier_coverage(self):
        for field, value in [('session_phase','Playing'), ('playing',True), ('can_load',True),
                             ('pending_visit',dict(epoch=3))]:
            before, _, _ = reports()
            before[field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_barrier(before, before, 2)

    def test_loading_must_hold_entire_world_and_bodies(self):
        for field in ('frame','combat','completed_visits','pid','build'):
            before, _, _ = reports()
            held = copy.deepcopy(before)
            if field == 'frame': held['frame']['tick'] += 1
            elif field == 'combat': held['combat']['door'] = 'closed'
            else: held[field] = 'changed'
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_barrier(before, held, 2)

    def test_arrival_cannot_advance_clock_lose_history_or_respawn(self):
        for fault in ('tick','epoch','wipe','history','overlap','life','warp','spawn'):
            before, waiting, _ = reports()
            if fault == 'tick': waiting['frame']['tick'] += 1
            elif fault == 'epoch': waiting['visit']['epoch'] += 1
            elif fault == 'wipe': waiting['frame']['checkpoint_generation'] = 1
            elif fault == 'history': waiting['completed_visits'] = 0
            elif fault == 'overlap': waiting['frame']['players'][1]['player']['feet'] = [0,0,0]
            elif fault == 'life': waiting['frame']['lives']['states'][1] = 'Dead'
            else: waiting['frame']['players'][1][fault+'ed_at' if fault=='spawn' else 'warped_at'] = 20
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_waiting_arrival(before, waiting)

    def test_each_players_inventory_health_and_will_must_survive(self):
        for index in (0,1):
            for field, value in [('sanity',100), ('will',100), ('owned',[]), ('collected',[]), ('loot',{'reset':True})]:
                before, waiting, guest = reports()
                waiting['frame']['players'][index]['stats'][field] = value
                with self.subTest(index=index,field=field), self.assertRaises(AssertionError):
                    verify_waiting_arrival(before, waiting)
                _, waiting, guest = reports()
                guest['frame']['players'][index]['stats'][field] = value
                with self.subTest(index=index,field=field), self.assertRaises(AssertionError):
                    verify_guest_arrival(waiting, guest, 2)

    def test_rejoin_cannot_use_stale_process_visit_or_body(self):
        for fault in ('pid','visit','tick','warp','spawn','wipe','phase'):
            _, waiting, guest = reports()
            if fault == 'pid': guest['pid'] = 2
            elif fault == 'visit': guest['visit']['epoch'] = 1
            elif fault == 'tick': guest['frame']['tick'] = 0
            elif fault == 'wipe': guest['frame']['checkpoint_generation'] = 1
            elif fault == 'phase': guest['session_phase'] = 'Loading'
            else: guest['frame']['players'][1]['spawned_at' if fault=='spawn' else 'warped_at'] = 20
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_guest_arrival(waiting, guest, 2)

    def test_only_destination_grants_and_empty_new_drop_journal_are_allowed(self):
        before, waiting, guest = reports()
        verify_waiting_arrival(before, waiting)
        guest['frame']['players'][1]['stats']['loot']['skool1$return'] = dict(awarded=[],drops=[])
        verify_guest_arrival(waiting, guest, 2)
        guest['frame']['players'][1]['stats']['loot']['skool1$return']['awarded'] = [99]
        with self.assertRaises(AssertionError):
            verify_guest_arrival(waiting, guest, 2)
        for slot in (0,6,9):
            before, waiting, _ = reports()
            waiting['frame']['players'][1]['stats']['owned'][slot] = 0 if slot < 9 else 1
            with self.subTest(slot=slot), self.assertRaises(AssertionError):
                verify_waiting_arrival(before, waiting)


if __name__ == '__main__':
    unittest.main()
