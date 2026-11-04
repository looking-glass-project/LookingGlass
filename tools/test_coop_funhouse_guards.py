"""Reject incomplete or unobserved personal Funhouse traversal claims."""
import copy
import unittest

from test_coop_funhouse import FunhousePair, verify_arrivals, verify_switch_receipt, verify_holds, waiting_point, clock_threats, following_point, verify_following, nearby_drops, verify_loot, shooting_neighbor, cards_preference
from test_coop_funhouse import fulcrum_waiting_point, fulcrum_balance_goal, fulcrums_reset
from test_coop_funhouse import resident_boojum_threats
from test_coop_funhouse import verify_exit_arrival
from test_coop_funhouse import forward_waiting_point, verify_forward_waits
from test_coop_funhouse import ice_preference
from test_coop_funhouse import verify_waiting_footings
from test_coop_funhouse import verify_pendulum_course
from test_coop_funhouse import verify_tweedle_handoff
from test_coop_funhouse import verify_input_jumps
from test_coop_funhouse import verify_gas_handoff


def gas_fixture(owner='Blue', skipped=False):
    entry = dict(step=90, scene='GasOpenCine', owner=owner, skipped=skipped,
                 starts=dict(host=100, guest=100), ends=dict(host=1220, guest=1220))
    bodies = [dict(feet=[48*i, 0, 0], health=100, grounded=True, spawned_at=i,
                   warped_at=10, stats=dict(owned=[1, i]), collected=[str(i)]) for i in range(2)]
    rows = []
    for tick, clock in ((100, .025), (1180, 9.025), (1220, None)):
        after = clock is None
        players = copy.deepcopy(bodies)
        if after:
            for p in players:
                p['warped_at'] = 1213
                p['feet'][0] += 100
        rows.append(dict(tick=tick, scene=None if after else entry['scene'], owner=None if after else owner,
            players=players, presentation=dict(visible_players=[after, after]), driver=dict(clear=[True, True]),
            funhouse=dict(cells=255, gas=after, scene=None if after else dict(clock=dict(time=clock)))))
    return entry, {peer: copy.deepcopy(rows) for peer in ('host', 'guest')}


class GasSceneGuards(unittest.TestCase):
    def test_gate_waits_for_pending_scene_and_accepts_scene_observed_during_startup(self):
        for already_finished in (False, True):
            entry, history = gas_fixture()
            pair = FunhousePair.__new__(FunhousePair)
            pair.step, pair.saved, pair.evidence = 90, None, {}
            pair.commands = dict(host={}, guest={})
            pair.args = type('Args', (), dict(skip_scenes=False))()
            commands = []
            pair.set = lambda role, **kwargs: commands.append((role, kwargs))
            pair.retain = lambda name: None
            pair.state = lambda role: pair.reports[role]['funhouse']
            def reports(index):
                return {peer: dict(funhouse=rows[index]['funhouse'],
                    route_driver=dict(scene=rows[index]['scene']), combat=dict(stage=dict(owner=rows[index]['owner'])),
                    frame=dict(tick=rows[index]['tick'], players=[dict(player=dict(grounded=p['grounded']),
                        stats=dict(sanity=p['health'])) for p in rows[index]['players']])) for peer, rows in history.items()}
            pair.reports = reports(-1 if already_finished else 0)
            if not already_finished:
                for r in pair.reports.values():
                    r['route_driver']['scene'] = None
                    r['combat']['stage']['owner'] = None
            pair.history = history if already_finished else dict(host=[], guest=[])
            def until(predicate, seconds):
                if not already_finished:
                    self.assertFalse(predicate(), 'An idle gap before gas starts is not completion')
                    pair.reports = reports(0)
                    self.assertFalse(predicate(), 'An active scene is not completion')
                    pair.reports, pair.history = reports(-1), history
                self.assertTrue(predicate())
            pair.until = until
            with self.subTest(already_finished=already_finished):
                pair.gas_scene()
                self.assertEqual(pair.evidence['gas_handoff'], entry)
                self.assertEqual(commands, [('host', dict(attack=False)), ('guest', dict(attack=False))])

    def test_saved_completion_does_not_claim_a_new_scene(self):
        pair = FunhousePair.__new__(FunhousePair)
        pair.step, pair.evidence, pair.reports = 90, {}, dict(host={}, guest={})
        pair.saved = dict(frame=dict(combat=dict(stage=dict(world=dict(levels=dict(funhouse=dict(gas=True)))))))
        pair.state = lambda role: dict(gas=True)
        pair.gas_scene()
        self.assertEqual(pair.evidence, {})

    def test_watched_scene_requires_both_clients_and_real_shared_reunion(self):
        for owner in ('Blue', 'Red'):
            verify_gas_handoff(*gas_fixture(owner))

    def test_missing_peer_ownership_visibility_movement_and_resources_fail(self):
        for mistake in ('peer', 'scene', 'owner', 'visible', 'moving', 'health', 'inventory', 'puzzle', 'early_flag'):
            entry, history = gas_fixture()
            row = history['guest'][1]
            if mistake == 'peer': history.pop('guest')
            elif mistake == 'scene': history['guest'] = [history['guest'][-1]]
            elif mistake == 'owner': row['owner'] = 'Red'
            elif mistake == 'visible': row['presentation']['visible_players'][1] = True
            elif mistake == 'moving': row['players'][1]['feet'][0] += 1
            elif mistake == 'health': row['players'][1]['health'] -= 1
            elif mistake == 'inventory': row['players'][1]['stats']['owned'] = [1, 0]
            elif mistake == 'puzzle': row['funhouse']['cells'] = 127
            else: row['funhouse']['gas'] = True
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_gas_handoff(entry, history)

    def test_watched_scene_cannot_be_shortened_or_miss_its_start_or_end(self):
        for mistake in ('short', 'start', 'end'):
            entry, history = gas_fixture()
            if mistake == 'short':
                for p in history['guest'][-1]['players']: p['warped_at'] = 1200
            elif mistake == 'start': history['guest'].pop(0)
            else: history['guest'].pop(1)
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_gas_handoff(entry, history)

    def test_skip_allows_short_duration_but_still_requires_scene_and_reunion(self):
        entry, history = gas_fixture(skipped=True)
        for rows in history.values():
            rows.pop(1)
            for p in rows[-1]['players']: p['warped_at'] = 130
        verify_gas_handoff(entry, history)
        history['guest'][-1]['funhouse']['gas'] = False
        with self.assertRaises(AssertionError): verify_gas_handoff(entry, history)

    def test_missing_unsafe_or_inconsistent_reunion_fails(self):
        for mistake in ('missing', 'warp', 'peer_warp', 'ground', 'clear', 'overlap', 'far', 'hidden', 'health', 'inventory', 'spawn'):
            entry, history = gas_fixture()
            row = history['guest'][-1]
            if mistake == 'missing': history['guest'].pop()
            elif mistake == 'warp': row['players'][1]['warped_at'] = 10
            elif mistake == 'peer_warp':
                for p in row['players']: p['warped_at'] += 1
            elif mistake == 'ground': row['players'][1]['grounded'] = False
            elif mistake == 'clear': row['driver']['clear'][1] = False
            elif mistake == 'overlap': row['players'][1]['feet'] = list(row['players'][0]['feet'])
            elif mistake == 'far': row['players'][1]['feet'][0] += 200
            elif mistake == 'hidden': row['presentation']['visible_players'][0] = False
            elif mistake == 'health': row['players'][1]['health'] -= 1
            elif mistake == 'inventory': row['players'][1]['stats']['owned'] = [1, 0]
            else: row['players'][1]['spawned_at'] += 1
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_gas_handoff(entry, history)


def input_jump_fixture(step=147):
    steps = [dict(pos=[0, 0, 0], mode='wait') for _ in range(step+1)]
    pad = [656, -2560, 160] if step == 147 else [-552, -2536, 160]
    steps[146 if step == 147 else 149] = dict(pos=pad)
    steps[step] = dict(pos=[.001, 0, 0], mode='inputjump', frames=120)
    entries, history = [], {}
    for slot, role in enumerate(('host', 'guest')):
        start, command = 100+slot*300, 9+slot
        body = dict(feet=list(pad), grounded=True, health=70,
                    jumps=4, spawned_at=0, warped_at=5)
        before = dict(tick=start, owner=None, players=[copy.deepcopy(body), copy.deepcopy(body)],
                      driver=dict(command=command-1))
        after = copy.deepcopy(before)
        after.update(tick=start+160, driver=dict(command=command))
        after['players'][slot].update(feet=[pad[0]+.3, pad[1], 256], jumps=5)
        active = copy.deepcopy(after)
        active['tick'] = start+20
        active['players'][slot]['grounded'] = False
        history[role] = [before, active, after]
        entries.append(dict(step=step, role=role, start_tick=start, input_start_tick=start+20, tick=start+160,
                            command=command, jumps_before=4, jumps_after=5))
    return steps, [step], entries, history


class TimedJumpGuards(unittest.TestCase):
    def test_each_player_needs_an_actual_jump_from_the_pad_and_landing(self):
        for step in (147, 151):
            with self.subTest(step=step):
                verify_input_jumps(*input_jump_fixture(step))

    def test_second_pad_cannot_reuse_first_pad_evidence(self):
        steps, completed, entries, history = input_jump_fixture(151)
        history['guest'][0]['players'][1]['feet'] = [656, -2560, 160]
        with self.assertRaises(AssertionError):
            verify_input_jumps(steps, completed, entries, history)

    def test_missing_short_unacknowledged_or_relocated_jumps_fail(self):
        for mistake in ('missing', 'short', 'no_jump', 'extra_jump', 'airborne', 'command', 'start_command', 'warp', 'death', 'wrong_pad'):
            steps, completed, entries, history = input_jump_fixture()
            before, active, after = history['guest']
            if mistake == 'missing': entries.pop()
            elif mistake == 'short': after['tick'] = entries[1]['tick'] = active['tick']+119
            elif mistake == 'no_jump': after['players'][1]['jumps'] = entries[1]['jumps_after'] = 4
            elif mistake == 'extra_jump': after['players'][1]['jumps'] = entries[1]['jumps_after'] = 6
            elif mistake == 'airborne': after['players'][1]['grounded'] = False
            elif mistake == 'command': after['driver']['command'] = 99
            elif mistake == 'start_command': active['driver']['command'] = 99
            elif mistake == 'warp': after['players'][1]['warped_at'] += 1
            elif mistake == 'death': after['players'][1]['health'] = 0
            else: before['players'][1]['feet'][0] += 8
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_input_jumps(steps, completed, entries, history)


def scene_fixture(role='host'):
    owner = 'Blue' if role == 'host' else 'Red'
    entry = dict(step=138, role=role, scene='Funhouse_Tweedle_Cinema1', skipped=False,
                 starts=dict(host=99, guest=99), ends=dict(host=310, guest=310))
    bodies = [dict(feet=[48*i, 0, 0], health=70, grounded=True, spawned_at=i, warped_at=10,
                   stats=dict(owned=[1, i]), collected=[str(i)]) for i in range(2)]
    rows = []
    for tick, line in [(100 + 15*i, i) for i in range(9)] + [(310, None)]:
        after = line is None
        players = copy.deepcopy(bodies)
        if after:
            for p in players:
                p['feet'][0] += 100
                p['warped_at'] = 300
        rows.append(dict(tick=tick, scene=None if after else entry['scene'], owner=None if after else owner,
            players=players, presentation=dict(visible_players=[after, after]), driver=dict(clear=[True, True]),
            funhouse=dict(fight=after, scene=None if after else dict(clock=dict(line=line)))))
    return entry, {peer: copy.deepcopy(rows) for peer in ('host', 'guest')}


class TweedleSceneGuards(unittest.TestCase):
    def test_closed_entrance_requires_verified_scene_not_fictitious_arrivals(self):
        entry, history = scene_fixture('guest')
        steps = [dict(pos=[0, 0, 0], mode='wait') for _ in range(139)]
        steps[138] = dict(pos=[32, -2800, -64])
        verify_arrivals(steps, [138], [], history, True, [entry])
        for mistake in ('missing', 'duplicate', 'wrong_goal', 'extra_arrival', 'not_completed', 'unsafe'):
            course, proof, rows = copy.deepcopy(steps), [copy.deepcopy(entry)], copy.deepcopy(history)
            completed, arrivals = [138], []
            if mistake == 'missing': proof = []
            elif mistake == 'duplicate': proof *= 2
            elif mistake == 'wrong_goal': course[138]['pos'][0] += 1
            elif mistake == 'extra_arrival': arrivals = [dict(step=138, role='guest')]
            elif mistake == 'not_completed': completed = []
            else: rows['host'][-1]['players'][0]['grounded'] = False
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_arrivals(course, completed, arrivals, rows, True, proof)

    def test_either_first_entrant_has_a_hidden_frozen_partner_and_shared_reunion(self):
        for role in ('host', 'guest'):
            verify_tweedle_handoff(*scene_fixture(role))

    def test_missing_scene_bad_owner_visible_or_moving_partner_fail(self):
        for mistake in ('missing', 'owner', 'visible', 'moving', 'dialogue'):
            entry, history = scene_fixture()
            if mistake == 'missing': history['guest'] = [history['guest'][-1]]
            elif mistake == 'owner': history['guest'][0]['owner'] = 'Red'
            elif mistake == 'visible': history['guest'][0]['presentation']['visible_players'][1] = True
            elif mistake == 'moving': history['guest'][1]['players'][1]['feet'][0] += 1
            else: history['guest'][1]['funhouse']['scene']['clock']['line'] = 7
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_tweedle_handoff(entry, history)

    def test_reunion_requires_real_warp_safe_bodies_and_personal_inventory(self):
        for mistake in ('warp', 'ground', 'overlap', 'clear', 'hidden', 'inventory', 'spawn'):
            entry, history = scene_fixture()
            row = history['guest'][-1]
            if mistake == 'warp': row['players'][1]['warped_at'] = 10
            elif mistake == 'ground': row['players'][1]['grounded'] = False
            elif mistake == 'overlap': row['players'][1]['feet'] = list(row['players'][0]['feet'])
            elif mistake == 'clear': row['driver']['clear'][1] = False
            elif mistake == 'hidden': row['presentation']['visible_players'][0] = False
            elif mistake == 'inventory': row['players'][1]['stats']['owned'] = [1, 0]
            else: row['players'][1]['spawned_at'] += 1
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_tweedle_handoff(entry, history)

    def test_original_actor_may_settle_briefly_but_cannot_drift_or_fall_away(self):
        def landing():
            entry, history = scene_fixture('guest')
            for rows in history.values():
                airborne = copy.deepcopy(rows[-1])
                airborne['tick'] = 305
                airborne['players'][1]['grounded'] = False
                airborne['players'][1]['feet'][2] = 13.2
                rows.insert(-1, airborne)
            return entry, history
        verify_tweedle_handoff(*landing())
        for mistake in ('deep_fall', 'drift', 'partner', 'late', 'damage'):
            entry, history = landing()
            rows = history['guest']
            if mistake == 'deep_fall': rows[-2]['players'][1]['feet'][2] = 33
            elif mistake == 'drift': rows[-1]['players'][1]['feet'][0] += 2
            elif mistake == 'partner': rows[-2]['players'][0]['grounded'] = False
            elif mistake == 'late': rows[-1]['tick'] = entry['ends']['guest'] = 400
            else: rows[-1]['players'][1]['health'] = 69
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_tweedle_handoff(entry, history)


def pendulum_fixture(role='host'):
    steps = [dict(pos=[0, 0, 0]) for _ in range(133)]
    steps[128] = dict(pos=[-288, -3500, -32], mode='holdjump', frames=1200)
    for step in range(129, 132):
        steps[step] = dict(pos=[0, 0, 0], mode='pendulum', thread=f'pend{step-128}')
    steps[132] = dict(pos=[640, -3500, 56], mode='jump')
    slot = 0 if role == 'host' else 1
    receipts, states = [], []
    previous = 9
    for step in range(128, 133):
        start, end = previous + 1, previous + (1201 if step == 128 else 80)
        feet = [-288, -3500, -32] if step == 128 else [640, -3500, 56] if step == 132 else [100*(step-128), -3500, 0]
        support = None if step in (128, 132) else dict(entity=246-step, name=f'pend{step-128}',
            goal=list(feet), supported=[slot == 0, slot == 1])
        receipts.append(dict(step=step, start_tick=start, start_jumps=step-128, tick=end, feet=list(feet), support=support))
        for tick, jumps in [(start, step-128), (end, step-127)]:
            body = dict(feet=list(feet), health=70, grounded=True, jumps=jumps, spawned_at=5, warped_at=0)
            states.append(dict(epoch=1, tick=tick, sha256=str(tick), world_sha256=str(tick),
                native_sha256=str(tick), movement=[dict(body), dict(body)]))
        previous = end
    course = dict(step=133, reached=True, receipts=receipts)
    entry = dict(role=role, epoch=1, command=77, course=course)
    history = {peer: [dict(owner=None, driver=dict(command=77 if peer == role else 78,
        funhouse_pendulums=course if peer == role else None, network_states=copy.deepcopy(states)))] for peer in ('host', 'guest')}
    return steps, entry, history


class PendulumCourseGuards(unittest.TestCase):
    def test_both_players_need_the_full_hold_and_all_actual_jumps(self):
        for role in ('host', 'guest'):
            verify_pendulum_course(*pendulum_fixture(role))

    def test_wrong_support_unsent_ticks_and_skipped_stages_fail(self):
        for mistake in ('support', 'name', 'brush', 'peer', 'command', 'duration', 'short_hold', 'stage'):
            steps, entry, history = pendulum_fixture()
            if mistake == 'support': entry['course']['receipts'][1]['support']['supported'][0] = False
            elif mistake == 'name': entry['course']['receipts'][1]['support']['name'] = 'pend2'
            elif mistake == 'brush': entry['course']['receipts'][1]['support']['entity'] = 116
            elif mistake == 'peer': history['guest'][0]['driver']['network_states'].pop(3)
            elif mistake == 'command': entry['command'] += 1
            elif mistake == 'duration': steps[128]['frames'] = 1201
            elif mistake == 'short_hold':
                entry['course']['receipts'][0]['start_tick'] += 2
                for rows in history.values():
                    rows[0]['driver']['network_states'][0]['tick'] += 2
            else: entry['course']['receipts'].pop(1)
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_pendulum_course(steps, entry, history)

    def test_matching_peers_do_not_excuse_death_relocation_or_fake_jumps(self):
        for field, value in [('grounded', False), ('health', 0), ('spawned_at', 12), ('warped_at', 12), ('jumps', 0)]:
            steps, entry, history = pendulum_fixture()
            for rows in history.values():
                rows[0]['driver']['network_states'][3]['movement'][0][field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_pendulum_course(steps, entry, history)


class ForwardWaitingGuards(unittest.TestCase):
    def test_waiter_must_still_be_alive_supported_and_at_its_stopping_point(self):
        entry = dict(role='host', tick=10, command=7, goal=[0, 0, 0])
        row = dict(tick=10, owner=None, driver=dict(command=7),
                   players=[dict(feet=[0, 0, 0], health=100, grounded=True)])
        verify_waiting_footings([entry], dict(host=[row]))
        for change in ('fall', 'dead', 'lower_floor', 'displaced', 'command'):
            bad = copy.deepcopy(row)
            if change == 'fall': bad['players'][0]['grounded'] = False
            if change == 'dead': bad['players'][0]['health'] = 0
            if change == 'lower_floor': bad['players'][0]['feet'][2] = -36
            if change == 'displaced': bad['players'][0]['feet'][0] = 24
            if change == 'command': bad['driver']['command'] = 8
            with self.assertRaises(AssertionError):
                verify_waiting_footings([entry], dict(host=[bad]))

    def test_ice_preference_restores_only_prior_authored_switches(self):
        steps = [dict(pos=[0, 0, 0]), dict(mode='ice', pos=[1, 0, 0]),
                 dict(mode='cards', pos=[0, 0, 0]), dict(mode='ice', pos=[0, 0, 0])]
        self.assertFalse(ice_preference(steps, 1))
        self.assertTrue(ice_preference(steps, 2))
        self.assertTrue(ice_preference(steps, 3))
        self.assertFalse(ice_preference(steps, 4))
        with self.assertRaises(AssertionError):
            ice_preference([dict(mode='ice', pos=[2, 0, 0])], 1)

    def test_waiting_cannot_skip_a_jump_hold_or_distant_waypoint(self):
        for step in (dict(pos=[120, 0, 0], mode='jump'), dict(pos=[120, 0, 0], mode='hold'),
                     dict(pos=[600, 0, 0]), dict(pos=[120, 0, 80])):
            self.assertIsNone(forward_waiting_point([dict(pos=[0, 0, 0]), step], 0, [0, 0, 0]))

    def test_forward_wait_requires_actual_supported_arrival_without_relocation(self):
        steps = [dict(pos=[0, 0, 0]), dict(pos=[120, 0, 0])]
        entry = dict(role='host', step=0, goal=[120, 0, 0], start_tick=10, tick=20)
        def history():
            body = dict(feet=[0, 0, 0], health=100, spawned_at=0, warped_at=0, grounded=True)
            return dict(host=[dict(tick=10, owner=None, players=[body]),
                              dict(tick=20, owner=None, players=[dict(body, feet=[120, 0, 0])])])
        verify_forward_waits([entry], steps, history())
        for field, value in [('health', 0), ('spawned_at', 15), ('warped_at', 15), ('grounded', False), ('feet', [128, 0, 0])]:
            rows=history();rows['host'][1]['players'][0][field]=value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_forward_waits([entry], steps, rows)


def exit_fixture(role='host'):
    steps = [{} for _ in range(106)]
    steps[103], steps[104], steps[105] = dict(pos=[1920, -3820, 150]), dict(pos=[1920, -3860, 195]), dict(pos=[1920, -4050, 256], mode='jump')
    slot = 0 if role == 'host' else 1
    course = dict(step=106, reached=True, start_tick=10, start_jumps=0, spawned_at=0, receipts=[])
    states = []
    for tick, point in [(10, [1920, -3400, -170]), (11, [1920, -3820, 100]), (12, [1920, -3860, 140]), (13, [1920, -4050, 256])]:
        body = dict(feet=point, velocity=[0, 0, 0], health=100, grounded=True, jumps=int(tick == 13), spawned_at=0, warped_at=0)
        bodies = [dict(body, feet=[1968, -3400, -170]), dict(body, feet=[1968, -3400, -170])]
        bodies[slot] = body
        states.append(dict(epoch=1, tick=tick, sha256=str(tick), world_sha256=str(tick), native_sha256=str(tick), movement=bodies))
        if tick > 10:
            step = tick + 92
            course['receipts'].append(dict(step=step, tick=tick, feet=point, surface=None if tick == 13 else
                dict(entity=460, requested=steps[step]['pos'], goal=point, supported=[slot == 0, slot == 1])))
    history = {peer: [dict(driver=dict(command=99 if peer == role else 100,
        funhouse_exit=course if peer == role else None, network_states=copy.deepcopy(states)))] for peer in ('host', 'guest')}
    entries = [dict(step=n['step'], role=role, epoch=1, tick=n['tick'], feet=n['feet'], goal=steps[n['step']]['pos'], command=99, course=course) for n in course['receipts']]
    return steps, entries, history


class ContinuousExitGuards(unittest.TestCase):
    def test_both_roles_need_real_sent_and_restored_movement_ticks(self):
        for role in ('host', 'guest'):
            steps, receipts, history = exit_fixture(role)
            for receipt in receipts:
                verify_exit_arrival(steps, receipt, history)

    def test_wrong_support_missing_peer_or_unobserved_receipt_cannot_pass(self):
        for mistake in ('board', 'support', 'peer', 'unsent', 'command', 'receipt'):
            steps, receipts, history = exit_fixture()
            receipt = receipts[0]
            if mistake == 'board':
                receipt['course']['receipts'][0]['surface']['entity'] = 459
            elif mistake == 'support':
                receipt['course']['receipts'][0]['surface']['supported'][0] = False
            elif mistake == 'peer':
                history['guest'][0]['driver']['network_states'].pop(1)
            elif mistake == 'unsent':
                for rows in history.values():
                    rows[0]['driver']['network_states'].pop(1)
            elif mistake == 'command':
                receipt['command'] = 101
            else:
                receipt['tick'] = 14
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_exit_arrival(steps, receipt, history)

    def test_shared_but_invalid_body_or_fake_jump_still_fails(self):
        for field, value in [('grounded', False), ('health', 0), ('spawned_at', 12), ('warped_at', 12), ('jumps', 0)]:
            steps, receipts, history = exit_fixture()
            for rows in history.values():
                rows[0]['driver']['network_states'][-1]['movement'][0][field] = value
            with self.subTest(field=field), self.assertRaises(AssertionError):
                verify_exit_arrival(steps, receipts[-1], history)


def fixture():
    steps = [dict(mode='wait', pos=[0, 0, 0]), dict(pos=[100, 100, 0])]
    receipts, history = [], {}
    for slot, role in enumerate(('host', 'guest')):
        bodies = [dict(feet=[100, 100 + 48*i, 0], grounded=True, health=100, spawned_at=0) for i in (0, 1)]
        bodies[slot]['feet'] = [100, 100, 0]
        bodies[1-slot]['feet'] = [100, 148, 0]
        history[role] = [dict(tick=10+slot, owner=None, players=bodies)]
        receipts.append(dict(step=1, role=role, goal=[100, 100, 0], tick=10+slot, feet=[100, 100, 0]))
    return steps, [0, 1], receipts, history


class PersonalArrivalGuards(unittest.TestCase):
    def test_recorded_waypoint136_uses_the_native_marker_height(self):
        _, _, receipts, history = fixture()
        steps = [dict(mode='wait', pos=[0, 0, 0]) for _ in range(137)]
        steps[136] = dict(pos=[400, -3000, -16])
        for slot, entry in enumerate(receipts):
            feet = [399.99704, -3000.48706, -52.053833]
            entry.update(step=136, goal=steps[136]['pos'], feet=list(feet), command=9)
            row = history[entry['role']][0]
            row['players'][slot]['feet'] = list(feet)
            row['driver'] = dict(command=9, phase='reached')
        verify_arrivals(steps, [136], receipts, history)
        for mistake in ('height', 'horizontal', 'airborne', 'command', 'planning'):
            entries, rows = copy.deepcopy(receipts), copy.deepcopy(history)
            body, driver = rows['host'][0]['players'][0], rows['host'][0]['driver']
            if mistake == 'height': body['feet'][2] = -68
            elif mistake == 'horizontal': body['feet'][0] = 408
            elif mistake == 'airborne': body['grounded'] = False
            elif mistake == 'command': driver['command'] = 8
            else: driver['phase'] = 'planning'
            entries[0]['feet'] = list(body['feet'])
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_arrivals(steps, [136], entries, rows)

    def test_recorded_far_bank_slope_keeps_native_height_and_tight_horizontal_bounds(self):
        def captured():
            _, _, receipts, history = fixture()
            steps = [dict(mode='wait', pos=[0, 0, 0]) for _ in range(135)]
            steps[134] = dict(pos=[450, -3270, 30])
            for slot, entry in enumerate(receipts):
                feet = [450.520172, -3270.575928, -7.039044]
                entry.update(step=134, goal=steps[134]['pos'], feet=list(feet), command=7)
                row = history[entry['role']][0]
                row['players'][slot]['feet'] = list(feet)
                row['driver'] = dict(command=7, phase='reached')
            return steps, [134], receipts, history
        verify_arrivals(*captured())
        for mistake in ('lower_floor', 'horizontal', 'airborne', 'command', 'planning'):
            args = captured()
            row, entry = args[3]['host'][0], args[2][0]
            if mistake == 'lower_floor': row['players'][0]['feet'][2] = -22
            elif mistake == 'horizontal': row['players'][0]['feet'][0] = 458
            elif mistake == 'airborne': row['players'][0]['grounded'] = False
            elif mistake == 'command': row['driver']['command'] = 6
            else: row['driver']['phase'] = 'planning'
            entry['feet'] = list(row['players'][0]['feet'])
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_arrivals(*args)

    def test_sloped_marker_matches_native_height_but_keeps_personal_support(self):
        def slope_fixture(step=116):
            _, _, receipts, history = fixture()
            goal, feet = [150, -4520, 112], [150, -4520, 69.0878]
            steps = [dict(mode='wait', pos=[0, 0, 0]) for _ in range(117)]
            steps[step] = dict(pos=goal)
            for slot, receipt in enumerate(receipts):
                receipt.update(step=step, goal=goal, feet=list(feet), command=7)
                row = history[receipt['role']][0]
                row['players'][slot]['feet'] = list(feet)
                row['driver'] = dict(command=7, phase='walking')
            return steps, [step], receipts, history
        verify_arrivals(*slope_fixture())
        for mistake in ('floor', 'horizontal', 'airborne', 'command', 'planning', 'other_step'):
            args = slope_fixture(115 if mistake == 'other_step' else 116)
            receipt = args[2][0]; row = args[3]['host'][0]; body = row['players'][0]
            if mistake in ('floor', 'horizontal'):
                body['feet'][2 if mistake == 'floor' else 0] = 60 if mistake == 'floor' else 158
                receipt['feet'] = list(body['feet'])
            elif mistake == 'airborne':
                body['grounded'] = False
            elif mistake == 'command':
                row['driver']['command'] = 6
            elif mistake == 'planning':
                row['driver']['phase'] = 'planning'
            with self.subTest(mistake=mistake), self.assertRaises(AssertionError):
                verify_arrivals(*args)

    def test_clear_includes_resident_boojums_and_remaining_screams(self):
        boojum = dict(active=True, health=56, feet=[1315, -3764, 352], waves=[])
        report = dict(frame=dict(players=[dict(player=dict(feet=[1920, -3600, 0])),
                                         dict(player=dict(feet=[1920, -2164, 0]))]),
                      route_driver=dict(boojums=[dict(id=0, name='resident_boojum_0', active=True,
                                                     health=56, distance=[702, 1700], line_of_sight=[True, False])]),
                      combat=dict(npcs=dict(actors=[dict(resident=dict(active=True, awake=True,
                                                                       body=dict(Boojum=boojum)))])))
        self.assertEqual(resident_boojum_threats(report, 'host'), [0])
        self.assertEqual(resident_boojum_threats(report, 'guest'), [])
        report['route_driver']['boojums'][0]['line_of_sight'][0] = False
        self.assertEqual(resident_boojum_threats(report, 'host'), [])
        report['route_driver']['boojums'][0]['line_of_sight'][0] = True
        boojum['health'] = 0
        self.assertEqual(resident_boojum_threats(report, 'host'), [])
        boojum['waves'] = [dict(position=[1920, -3550, 42])]
        self.assertEqual(resident_boojum_threats(report, 'host'), [0])
        report['route_driver']['boojums'] = []
        with self.assertRaises(AssertionError):
            resident_boojum_threats(report, 'host')

    def test_tilted_arrival_needs_the_named_native_support(self):
        steps = [dict(pos=[0, 0, 0]) for _ in range(102)]
        steps[101] = dict(pos=[1920, -3400, 0])
        receipts, history = [], {}
        for slot, role in enumerate(('host', 'guest')):
            target = [1920, -3400, -178]
            body = dict(feet=target, grounded=True, health=100, spawned_at=0)
            native = dict(entity=460, requested=steps[101]['pos'], goal=target, supported=[True, True])
            history[role] = [dict(tick=10+slot, owner=None, players=[body, body],
                                 driver=dict(command=7+slot, surface=native))]
            receipts.append(dict(step=101, role=role, goal=steps[101]['pos'], feet=target, target=target,
                                 tick=10+slot, command=7+slot, surface=460))
        verify_arrivals(steps, [101], receipts, history)
        for mutation in ('lower_floor', 'wrong_board', 'wrong_goal', 'unacknowledged'):
            wrong = copy.deepcopy(history)
            driver = wrong['guest'][0]['driver']
            if mutation == 'lower_floor': driver['surface']['supported'][1] = False
            elif mutation == 'wrong_board': driver['surface']['entity'] = 459
            elif mutation == 'wrong_goal': driver['surface']['goal'][2] -= 80
            else: driver['command'] += 1
            with self.assertRaises(AssertionError):
                verify_arrivals(steps, [101], receipts, wrong)

    def test_final_seesaw_is_included_in_reset_gate(self):
        state = dict(fulcrums=[[0, 0] for _ in range(63)])
        self.assertTrue(fulcrums_reset([state, state]))
        tilted = copy.deepcopy(state)
        tilted['fulcrums'][62] = [-45, 0]
        self.assertFalse(fulcrums_reset([state, tilted]))
        self.assertFalse(fulcrums_reset([tilted, state]))

    def test_waiter_balances_on_a_supported_lateral_pivot_approach(self):
        points = [[1974.9, -3601.7, .03125], [1926.9, -3553.7, .03125], [1960.9, -3567.7, .03125]]
        anchor = fulcrum_waiting_point(points, [1920, -3600, -16])
        self.assertEqual(anchor, points[0])
        self.assertEqual(fulcrum_balance_goal(anchor, 0), [1974.9, -3600, 0])
        for angle in (-45, -10, 10, 45):
            goal = fulcrum_balance_goal(anchor, angle)
            self.assertEqual(goal[0], anchor[0])
            self.assertGreater((goal[1] + 3600) * angle, 0)
            self.assertLessEqual(abs(goal[1] + 3600), 220)
        with self.assertRaises(AssertionError):
            fulcrum_waiting_point(points[1:], [1920, -3600, -16])

    def test_moving_approach_requires_native_window_and_acknowledged_walk(self):
        steps = [dict(pos=[0, 0, 0]) for _ in range(96)]
        steps[95] = dict(pos=[1920, -2650, 0])
        receipts, history = [], {}
        for slot, role in enumerate(('host', 'guest')):
            body = dict(feet=[1920, -2634, -14], grounded=True, health=100, spawned_at=0)
            history[role] = [dict(tick=10+slot, owner=None, players=[body, body],
                                 driver=dict(command=7+slot, phase='walking', expected=None))]
            receipts.append(dict(step=95, role=role, goal=steps[95]['pos'], feet=body['feet'],
                                 tick=10+slot, command=7+slot, crossing=True))
        verify_arrivals(steps, [95], receipts, history)
        for field, value in [('feet', [1920, -2634, -100]), ('grounded', False)]:
            wrong = copy.deepcopy(history)
            claimed = copy.deepcopy(receipts)
            wrong['guest'][0]['players'][1][field] = value
            if field == 'feet':
                claimed[1]['feet'] = value
            with self.assertRaises(AssertionError):
                verify_arrivals(steps, [95], claimed, wrong)
        wrong = copy.deepcopy(history)
        wrong['guest'][0]['driver']['phase'] = 'planning'
        with self.assertRaises(AssertionError):
            verify_arrivals(steps, [95], receipts, wrong)
        wrong = copy.deepcopy(history)
        wrong['guest'][0]['driver']['command'] += 1
        with self.assertRaises(AssertionError):
            verify_arrivals(steps, [95], receipts, wrong)

    def test_resumed_course_retains_the_last_authored_cards_choice(self):
        steps = [dict(pos=[0, 0, 0]), dict(mode='cards', pos=[0, 0, 0]),
                 dict(pos=[200, 0, 0]), dict(mode='cards', pos=[1, 0, 0])]
        self.assertTrue(cards_preference(steps, 1))
        self.assertFalse(cards_preference(steps, 3))
        self.assertTrue(cards_preference(steps, 4))

    def test_authored_cards_preference_changes_only_requested_controls(self):
        pair = FunhousePair.__new__(FunhousePair)
        pair.serial, pair.lead, pair.commands = 0, .3, {}
        pair.args = type('Args', (), {'skip_scenes': False})()
        for enabled in (False, True):
            pair.prefer_cards = enabled
            pair.set('host', [400, 0, 0], run=True)
            self.assertEqual(pair.commands['host']['prefer_cards'], enabled)
            self.assertTrue(pair.commands['host']['attack'])
            self.assertEqual(pair.commands['host']['goal'], [400, 0, 0])

    def test_drop_selection_uses_personal_live_nearby_inventory(self):
        drop = dict(id=32, origin=[699, 2239, 3], grade='Large', remaining=7.9)
        body = dict(player=dict(feet=[800, 2293, 0]), stats=dict(sanity=3, will=.6,
            collected=[], loot={'funhouse$first': dict(drops=[drop])}))
        self.assertEqual(nearby_drops(body)[0]['key'], 'drop:funhouse$first:32')
        for change in ('collected', 'expired', 'distant', 'full'):
            test = copy.deepcopy(body)
            if change == 'collected':
                test['stats']['collected'].append('drop:funhouse$first:32')
            elif change == 'expired':
                test['stats']['loot']['funhouse$first']['drops'][0]['remaining'] = 0
            elif change == 'distant':
                test['player']['feet'] = [0, 0, 0]
            else:
                test['stats'].update(sanity=100, will=100)
            self.assertEqual(nearby_drops(test), [])

    def test_drop_verification_rejects_partner_credit_and_vanishing_pickups(self):
        key = 'drop:funhouse$first:32'
        old = dict(health=3, collected=[], stats=dict(loot={'funhouse$first': dict(
            drops=[dict(id=32, origin=[699, 2239, 3], remaining=7.9)])}))
        new = copy.deepcopy(old)
        new.update(health=53, collected=[key])
        receipts = [dict(role='host', key=key, origin=[699, 2239, 3], start_tick=1, tick=2)]
        history = dict(host=[dict(tick=1, owner=None, players=[old, old]),
                             dict(tick=2, owner=None, players=[new, old])])
        verify_loot(receipts, history)
        for bodies in ([old, new], [old, old]):
            bad = copy.deepcopy(history)
            bad['host'][1]['players'] = bodies
            with self.assertRaises(AssertionError):
                verify_loot(receipts, bad)

    def test_following_waits_for_space_and_uses_an_observed_footing(self):
        trail = [dict(tick=100, feet=[0, 0, 0]), dict(tick=200, feet=[96, 0, 0])]
        self.assertIsNone(following_point(trail[:1], [64, 0, 0], [-60, 0, 0]))
        self.assertEqual(following_point(trail, [192, 0, 0], [0, 0, 0]), trail[1])
        self.assertEqual(following_point(trail, [192, 0, 0], [-60, 0, 0]), trail[0])

    def test_clockwork_fists_and_awake_enemies_prevent_false_clear(self):
        report = dict(frame=dict(players=[dict(player=dict(feet=[0, 0, 0]))]*2),
                      combat=dict(npcs=dict(actors=[dict(clock=dict(active=True, health=400,
                          phase='Fast', feet=[650, 0, 0], fists=[]))])))
        self.assertEqual(clock_threats(report, 'host'), [0])
        clock = report['combat']['npcs']['actors'][0]['clock']
        clock['phase'] = 'Idle'
        self.assertEqual(clock_threats(report, 'host'), [])
        clock.update(feet=[900, 0, 0], fists=[dict(victim=5000002)])
        self.assertEqual(clock_threats(report, 'guest'), [0])
        clock['health'] = 0
        self.assertEqual(clock_threats(report, 'guest'), [0], 'A dead Automaton can still have a live fist')
        clock['fists'] = []
        self.assertEqual(clock_threats(report, 'guest'), [])

    def test_follower_receipts_reject_invented_or_dead_footings(self):
        _, _, receipts, history = fixture()
        entry = dict(leader='host', follower='guest', tick=10, feet=receipts[0]['feet'])
        verify_following([entry], history)
        for change in (dict(tick=99), dict(feet=[0, 0, 0]), dict(follower='host')):
            with self.assertRaises(AssertionError):
                verify_following([dict(entry, **change)], history)
        history['host'][0]['players'][0]['health'] = 0
        with self.assertRaises(AssertionError):
            verify_following([entry], history)

    def test_death_suspends_inputs_until_the_game_revives_the_player(self):
        pair = FunhousePair.__new__(FunhousePair)
        pair.serial, pair.lead = 0, .3
        pair.args = type('Args', (), {'skip_scenes': False})()
        pair.allow_revival, pair.suspended, pair.commands = True, {}, {}
        pair.survivor_wait, pair.moving_role = {}, None
        body = dict(stats=dict(sanity=0), player=dict(feet=[100, 0, 0]), spawned_at=0)
        pair.reports = dict(host=dict(frame=dict(players=[body])))
        pair.set('host', [400, 0, 0], run=True)
        original = copy.deepcopy(pair.reports)
        pair.resume_after_death()
        self.assertEqual(pair.reports, original)
        self.assertIsNone(pair.commands['host']['goal'])
        self.assertFalse(pair.commands['host']['attack'])
        self.assertTrue(pair.commands['host']['recovery_trace'])
        neutral_id = pair.commands['host']['id']
        pair.resume_after_death()
        self.assertEqual(pair.commands['host']['id'], neutral_id)
        body['stats']['sanity'], body['spawned_at'] = 25, 1200
        body['player']['feet'] = [200, 48, 0]
        original = copy.deepcopy(pair.reports)
        pair.resume_after_death()
        self.assertEqual(pair.reports, original)
        self.assertEqual(pair.commands['host']['goal'], [400, 0, 0])
        self.assertTrue(pair.commands['host']['attack'])
        self.assertGreater(pair.commands['host']['id'], neutral_id)
        self.assertFalse(pair.suspended)

    def test_waiting_survivor_can_dodge_then_resumes_the_parked_goal(self):
        pair = FunhousePair.__new__(FunhousePair)
        pair.serial, pair.lead = 0, .3
        pair.args = type('Args', (), {'skip_scenes': False})()
        pair.allow_revival, pair.suspended, pair.commands = True, {}, {}
        pair.survivor_wait, pair.moving_role = {}, 'host'
        bodies = [dict(stats=dict(sanity=0)), dict(stats=dict(sanity=70))]
        pair.reports = {role: dict(frame=dict(players=bodies)) for role in ('host', 'guest')}
        pair.set('host', [400, 0, 0], run=True)
        pair.set('guest', [300, 48, 0], run=True)
        pair.resume_after_death()
        self.assertIsNone(pair.commands['guest']['goal'])
        self.assertTrue(pair.commands['guest']['attack'] and pair.commands['guest']['combat_tactics'])
        self.assertEqual(bodies[1]['stats']['sanity'], 70)
        bodies[0]['stats']['sanity'] = 25
        pair.resume_after_death()
        self.assertEqual(pair.commands['guest']['goal'], [300, 48, 0])
        self.assertFalse(pair.survivor_wait)

    def test_waiting_point_keeps_clearance_after_arrival_tolerance(self):
        # Captured step 34: the old target 41.6 units from the waypoint let
        # Alice stop just 34.4 units away, inside the partner's destination.
        at, incoming, goal = [802.83, 1493.52, .03], [757.70, 1492.52, .03], [800, 1500, 0]
        unsafe, safe = [802.83, 1541.52, .03], [802.83, 1445.52, .03]
        self.assertEqual(waiting_point([unsafe, safe], at, incoming, goal), safe)
        with self.assertRaises(AssertionError):
            waiting_point([unsafe], at, incoming, goal)

    def test_both_observed_arrivals_pass(self):
        verify_arrivals(*fixture())

    def test_missing_or_duplicate_partner_cannot_pass(self):
        for duplicate in (False, True):
            args = fixture()
            args[2].pop()
            if duplicate:
                args[2].append(copy.deepcopy(args[2][0]))
            with self.assertRaises(AssertionError):
                verify_arrivals(*args)

    def test_unobserved_relocated_or_dead_arrivals_cannot_pass(self):
        for field, value in [('feet', [1000, 100, 0]), ('grounded', False), ('health', 0), ('spawned_at', 10)]:
            args = fixture()
            args[3]['guest'][0]['players'][1][field] = value
            with self.assertRaises(AssertionError):
                verify_arrivals(*args)
        args = fixture()
        args[2][1]['tick'] = 99
        with self.assertRaises(AssertionError):
            verify_arrivals(*args)

    def test_neutral_save_request_clears_previous_attack(self):
        # The first real run rejected its save because inherited attack input
        # survived a neutral command. Exercise that exact command sequence.
        pair = FunhousePair.__new__(FunhousePair)
        pair.serial, pair.lead = 0, .3
        pair.args = type('Args', (), {'skip_scenes': False})()
        pair.commands = {}
        pair.set('host', [100, 100, 0])
        self.assertTrue(pair.commands['host']['attack'])
        pair.set('host')
        pair.commands['host']['checkpoint'] = True
        self.assertFalse(pair.commands['host']['attack'])
        self.assertIsNone(pair.commands['host']['goal'])

    def test_jump_arrival_requires_a_new_observed_jump(self):
        args = fixture()
        args[0][1]['mode'] = 'jump'
        for receipt in args[2]:
            role = receipt['role']
            slot = 0 if role == 'host' else 1
            earlier = copy.deepcopy(args[3][role][0])
            earlier['tick'] = 9
            earlier['players'][slot]['jumps'] = 0
            args[3][role].insert(0, earlier)
            args[3][role][-1]['players'][slot]['jumps'] = 1
            receipt.update(start_tick=9, jumps_before=0, jumps_after=1)
        verify_arrivals(*args)
        # A supported 14-unit offset is inside the original jump controller's
        # 20-unit limit; the exact boundary must still fail.
        args[3]['guest'][-1]['players'][1]['feet'] = [114, 100, 0]
        args[2][1]['feet'] = [114, 100, 0]
        verify_arrivals(*args)
        args[3]['guest'][-1]['players'][1]['feet'] = [120, 100, 0]
        args[2][1]['feet'] = [120, 100, 0]
        with self.assertRaises(AssertionError):
            verify_arrivals(*args)
        args[3]['guest'][-1]['players'][1]['feet'] = [100, 100, 0]
        args[2][1]['feet'] = [100, 100, 0]
        args[3]['guest'][-1]['players'][1]['jumps'] = 0
        with self.assertRaises(AssertionError):
            verify_arrivals(*args)


def shot_fixture():
    def row(tick, active, phase='idle', command=None, reached=False, expected=None):
        return dict(tick=tick, funhouse=dict(mirror=active, cells=2),
                    driver=dict(command=command, phase=phase, reached=reached, expected=expected))
    entry = dict(role='host', command=7, thread='MirrorTrigger', before_ticks=dict(host=10, guest=10))
    history = dict(host=[row(10, False), row(11, False, 'shooting switch', 7, expected=[10, 20, 30]),
                         row(12, True, 'switch completed', 7, True)],
                   guest=[row(10, False), row(12, True)])
    return entry, history


class SwitchReceiptGuards(unittest.TestCase):
    def test_shooting_approach_stays_near_home_and_clear_of_partner(self):
        home, other, target = [-1232, -52, 0], [-1230, -88, 0], [-1043, -16, 288]
        safe, blocked, far = [-1184, -52, 0], [-1232, -100, 0], [-1088, -52, 0]
        self.assertEqual(shooting_neighbor([safe, blocked, far], home, other, target, [home]), safe)
        self.assertIsNone(shooting_neighbor([safe, blocked, far], home, other, target, [home, safe]))

    def test_switch_retry_prefers_visible_footing_without_relaxing_safety(self):
        home, other, target = [-1648, -54, 0], [-1648, -90, 0], [-1837, -18, 288]
        closer, visible, far = [-1717, -25, 0], [-1676, 22, 0], [-1900, 22, 0]
        neighbors = [closer, visible, far, other]
        self.assertEqual(shooting_neighbor(neighbors, home, other, target, [home],
                                          [visible, far, other]), visible)
        self.assertEqual(shooting_neighbor(neighbors, home, other, target, [home, visible],
                                          [visible, far, other]), closer)

    def test_shooting_receipt_rejects_an_invented_approach(self):
        entry, history = shot_fixture()
        point = [-1184, -52, 0]
        entry['repositions'] = [dict(before=10, after=11, goal=point)]
        history['host'][0]['driver']['neighbors'] = [[point], []]
        history['host'][1]['players'] = [dict(health=100, grounded=True, feet=point), {}]
        verify_switch_receipt(entry, history)
        history['host'][0]['driver']['neighbors'][0] = []
        with self.assertRaises(AssertionError):
            verify_switch_receipt(entry, history)

    def test_observed_target_and_both_shared_states_pass(self):
        verify_switch_receipt(*shot_fixture())

    def test_missing_target_completed_command_or_partner_state_fails(self):
        for mutation in ('target', 'completed', 'partner', 'already'):
            entry, history = shot_fixture()
            if mutation == 'target':
                history['host'].pop(1)
            elif mutation == 'completed':
                history['host'][-1]['driver']['reached'] = False
            elif mutation == 'partner':
                history['guest'][-1]['funhouse']['mirror'] = False
            else:
                history['guest'][0]['funhouse']['mirror'] = True
            with self.assertRaises(AssertionError):
                verify_switch_receipt(entry, history)


def hold_fixture():
    goal = [665, 775, -104]
    steps = [dict(mode='hold', pos=goal, frames=120)]
    entries = [dict(step=0, goal=goal, frames=120, starts=dict(host=100, guest=100),
                    ends=dict(host=220, guest=220), commands=dict(host=1, guest=2))]
    history = {role: [dict(tick=tick, driver=dict(command=command, phase='holding toward goal', expected=goal))
                      for tick in (100, 160, 220)] for role, command in [('host', 1), ('guest', 2)]}
    return steps, [0], entries, history


class HoldGuards(unittest.TestCase):
    def test_both_players_hold_toward_the_authored_point_for_the_required_ticks(self):
        verify_holds(*hold_fixture())

    def test_short_interrupted_or_missing_partner_hold_fails(self):
        for mutation in ('short', 'interrupted', 'partner', 'direction'):
            args = hold_fixture()
            if mutation == 'short':
                args[2][0]['ends']['guest'] = 219
            elif mutation == 'interrupted':
                args[3]['guest'][1]['driver']['phase'] = 'scene'
            elif mutation == 'partner':
                args[2][0]['starts'].pop('guest')
            else:
                args[3]['host'][-1]['driver']['expected'] = [0, 0, 0]
            with self.assertRaises(AssertionError):
                verify_holds(*args)


if __name__ == '__main__':
    unittest.main()
