"""Play Checkmate in two independent internet clients using ordinary inputs.

Run in Anode. The existing local controller plans from each client's received
world. This runner writes commands and normal checkpoint requests only; it
never changes player positions, resources, enemies or progression. No images.
"""
import argparse
import hashlib
import json
import re
import time
from pathlib import Path

from test_coop_play import ROOT, distance
from test_coop_village import Pair
from test_coop_chess import ChessPair
from test_coop_castling import verify_presence
from test_coop_chess_resume import verify_resume_body, verify_resume_receipt
from coop_recovery import read_trace, verify_views, verify_northern_coverage
from coop_recovery import verify_presence as verify_recovery_presence


HEALING = [(3968, 3392, 0), (3968, 2624, 0), (4352, 3008, 0)]
CORNERS = [(3940, 2630, 0), (4470, 2630, 0), (4470, 3370, 0), (3940, 3370, 0)]
SCENES = {'cinema_beheading_thread', 'cinema_king_battle_thread', 'cinema_king_killed_thread'}


def verify_beheading(rows, first, role, presence=verify_presence):
    scene = [r for r in rows if r['scene'] == 'cinema_beheading_thread']
    assert scene, 'Original beheading scene was not observed'
    assert {r['owner'] for r in scene} == {first.title()}, 'Wrong beheading owner'
    partner = 1 if first == 'blue' else 0
    assert all(not r['presentation']['visible_players'][partner] for r in scene), 'Scene displayed the waiting partner'
    presence(rows, role)
    final = rows[-1]
    assert final['owner'] is None and final['checkmate']['beheaded'], 'Beheading did not finish'
    # The follow camera can legitimately hide its own body against a wall.
    # Each client must still draw the other player after the scripted hold.
    remote = 1 if role == 'host' else 0
    assert final['presentation']['visible_players'][remote], 'Partner did not reappear after the scene'
    assert all(p['health'] > 0 for p in final['players']), 'Scene ended with a dead player'
    return dict(owner=first.title(), scene_samples=len(scene), reunited=True)


def course():
    source = (ROOT / 'src/levels/rchess1/route_input.rs').read_text()
    match = re.search(r'r#"(.*?)"#', source, re.S)
    assert match, 'Missing authored Checkmate input course'
    result = json.loads(match[1])
    assert len(result) == 45 and result[7:10] == [
        {'input': [1, 0, 0, 1, 1, 0]}, {'input': [8, 0, 0, 1, 0, 0]},
        {'input': [.6, 1, 0, 1, 1, 0]}], 'Review changed rope course before replay'
    return result


class CheckmatePair(Pair):
    map_name, entry, next_map = 'rchess1', 'rchess1_start1', 'funhouse'
    evidence_prefix, capture_scenes = 'checkmate-route-', False

    def __init__(self, args):
        self.allow_revival = getattr(args, 'allow_revival', False)
        if self.allow_revival:
            self.evidence_prefix = 'checkmate-recovery-route-'
        self.minimum_lead = min(1.5, .05 + 2 * (args.delay_ms + args.jitter_ms) / 1000)
        self.lead = self.minimum_lead
        self.last_valid, self.completed = {}, []
        self.swim_requested = {}
        self.step = -1
        self.first = 'host' if args.first == 'blue' else 'guest'
        self.partner = 'guest' if self.first == 'host' else 'host'
        super().__init__(args)
        self.deadline = time.monotonic() + 1600
        self.steps = course()
        raw = Path(__file__).read_bytes()
        (self.folder / 'fixture-source.py').write_bytes(raw)
        (self.folder / 'course.json').write_text(json.dumps(self.steps))
        self.evidence['fixture_sha256'] = hashlib.sha256(raw).hexdigest()
        self.evidence['executable_sha256'] = hashlib.sha256(args.exe.read_bytes()).hexdigest()
        dependencies = self.folder / 'fixture-dependencies'
        dependencies.mkdir()
        self.evidence['fixture_dependencies'] = {}
        for name in ('test_coop_play.py', 'test_coop_village.py', 'test_coop_chess.py',
                     'test_coop_castling.py', 'test_coop_chess_resume.py', 'coop_diagnostics.py', 'coop_recovery.py'):
            raw = (Path(__file__).parent / name).read_bytes()
            (dependencies / name).write_bytes(raw)
            self.evidence['fixture_dependencies'][name] = hashlib.sha256(raw).hexdigest()
        self.saved = None
        if args.checkpoint:
            raw = args.checkpoint.read_bytes()
            self.saved = json.loads(raw)['payload']
            assert self.saved['visit']['map'] == self.map_name and self.saved['visit']['epoch'] == 1
            assert self.saved['frame']['checkpoint_generation'] == 0
            self.resume_checkpoint = True
            (self.folder / 'host' / 'coop-checkpoint.json').write_bytes(raw)
            self.evidence.update(checkpoint_sha256=hashlib.sha256(raw).hexdigest(),
                                 checkpoint_tick=self.saved['frame']['tick'])

    def set(self, role, goal=None, navigate=False, *, swim=False, attack=None, **kwargs):
        if attack is None:
            attack = goal is not None or kwargs.get('rope') is not None
        if navigate:
            # Run the exposed late approach; timed jumps and final landing
            # braking retain their own inputs in the native controller.
            kwargs['run'] = self.step >= 27
        self.swim_requested[role] = swim
        swimming = role in self.reports and self.body(role)['player']['swimming']
        super().set(role, goal, navigate, attack=attack, **kwargs)
        self.commands[role].update(swim=swim and swimming, prefer_cards=True, watch_dialogue=True,
            skip_scene=self.args.skip_scenes, input_lead=self.lead if role == 'guest' else 0,
            arrival_radius=8)
        if self.allow_revival:
            self.commands[role]['recovery_trace'] = role == 'host'

    def body(self, role):
        return self.reports[role]['frame']['players'][0 if role == 'host' else 1]

    def state(self, role='host'):
        return self.reports[role]['combat']['stage']['world']['levels'].get(self.map_name)

    def busy(self):
        return any(r['combat']['stage']['owner'] is not None for r in self.reports.values())

    def retain(self, name):
        (self.folder / (name + '.json')).write_text(json.dumps(dict(
            step=self.step, completed=self.completed, commands=self.commands,
            reports=self.reports, evidence=self.evidence)))

    def validate_report(self, role, report):
        try:
            if self.allow_revival:
                assert report['frame']['checkpoint_generation'] == 0, 'Unexpected team wipe; separate checkpoint verification required'
                assert report['route_driver'].get('recovery_trace_error') is None, 'Host recovery trace failed: ' + str(report['route_driver'].get('recovery_trace_error'))
            else:
                super().validate_report(role, report)
            bodies = report['frame']['players']
            assert all(not p['stats']['god'] and not p['stats']['notarget'] for p in bodies), 'Route used cheats'
            assert self.allow_revival or all(p['spawned_at'] == 0 for p in bodies), 'Strict route respawned a player'
            if report['combat']['stage']['owner'] is None and all(p['stats']['sanity'] > 0 for p in bodies):
                a, b = [p['player']['feet'] for p in bodies]
                assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2]-b[2]) >= 55.95, 'Players overlap'
                if report['visit']['map'] == self.map_name and report['route_driver']['scene'] is None:
                    remote = 1 if role == 'host' else 0
                    assert report['presentation']['visible_players'][remote], 'Partner hidden during free Checkmate play'
                    center = list(bodies[remote]['player']['feet'])
                    center[2] += 32
                    if distance(report['camera'], center) < 48:
                        samples = self.evidence.setdefault('close_partner_samples', {})
                        samples[role] = samples.get(role, 0) + 1
        except Exception:
            (self.folder / ('failure-' + role + '.json')).write_text(json.dumps(dict(
                before=self.last_valid.get(role), after=report, commands=self.commands, step=self.step)))
            raise
        if report['visit']['map'] == self.map_name:
            self.last_valid[role] = report

    def validate_clear(self, role, report):
        if not self.allow_revival:
            return super().validate_clear(role, report)
        assert all(clear for clear, body in zip(report['route_driver']['clear'], report['frame']['players'])
                   if body['stats']['sanity'] > 0), f'{role}: obstructed living body'

    def verify_presence(self, rows, role):
        (verify_recovery_presence if self.allow_revival else verify_presence)(rows, role)

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Checkmate stopped; evidence retained')
        for role, requested in self.swim_requested.items():
            if requested and role in self.reports:
                # Walk/fall into water first; the existing swim command requires
                # an already swimming body and never changes its movement mode.
                self.commands[role]['swim'] = self.body(role)['player']['swimming']
        super().pump()
        self.clear_dead_northern_commands()
        guest = self.reports.get('guest')
        if guest:
            rtt = (guest.get('network_timing') or {}).get('round_trip_ms')
            if rtt is not None and 0 <= rtt < 10000:
                self.lead = max(self.minimum_lead, .8*self.lead + .2*min(1.5, .05+rtt/1000))
                self.commands['guest']['input_lead'] = self.lead
        for role, report in self.reports.items():
            row = self.history[role][-1]
            # Several render frames may share a network tick. Keep these
            # fields with the camera sample first stored by Pair.pump().
            row.setdefault('checkmate', self.state(role))
            row.setdefault('presentation', report.get('presentation'))

    def clear_dead_northern_commands(self):
        # Deliver neutral input during the countdown. A revived body may be
        # beside its partner on another floor, where its old goal is unreachable.
        # northern_course assigns the next goal after verifying the spawn stamp.
        if not self.allow_revival or not 33 <= self.step <= 44:
            return
        for role in self.reports:
            command = self.commands[role]
            if self.body(role)['stats']['sanity'] <= 0 and (
                    command.get('goal') is not None or command.get('navigate')
                    or command.get('rope') is not None or command.get('attack')):
                self.set(role, attack=False)

    def until(self, predicate, seconds=120):
        end = time.monotonic() + seconds
        while not predicate():
            if time.monotonic() >= end:
                self.retain('stalled')
                raise AssertionError('Checkmate stalled at step ' + str(self.step) + ': ' + json.dumps({
                    role: dict(tick=r['frame']['tick'], phase=r['route_driver']['phase'],
                        feet=self.body(role)['player']['feet'], health=self.body(role)['stats']['sanity'])
                    for role, r in self.reports.items()}))
            self.pump()

    def scene_end(self):
        if not self.busy():
            return
        for role in self.commands:
            self.set(role)
        self.until(lambda: self.arrived() or not self.busy(), 180)

    def go(self, goal, navigate=False, swim=False):
        """Follow real footings, advancing the partner during long late legs."""
        self.scene_end()
        if self.arrived():
            return
        previous = self.body(self.first)['player']['feet']
        trail = [list(previous)]
        follow_trail = navigate and self.step >= 27
        active = [self.first]
        follow_previous = distance(previous, goal) >= 96 and distance(self.body(self.partner)['player']['feet'], previous) > 48
        deferred_follower = follow_previous and not follow_trail
        if follow_previous and not deferred_follower:
            self.set(self.partner, previous, navigate, swim=swim, run=True)
            active.append(self.partner)
        else:
            self.set(self.partner, attack=True)
        self.set(self.first, goal, navigate, swim=swim, run=True)
        def at_waypoint(role):
            # The authored swim approach stops within 18 units (check.rs).
            # Its transitional -450 goal is ten units above the native swim
            # surface; requiring the general controller's eight-unit radius
            # leaves Alice treading water forever. Still require real swimming
            # and a matching acknowledged command, with unchanged body checks.
            body = self.body(role)['player']
            driver = self.reports[role]['route_driver']
            command = self.commands[role]
            return self.reached(role) or (swim and body['swimming']
                and driver['command'] == command['id']
                and distance(body['feet'], command['goal']) < 18)
        def arrived_together():
            nonlocal deferred_follower
            if self.arrived() or self.busy():
                return True
            leader = self.body(self.first)['player']
            if deferred_follower and distance(leader['feet'], previous) >= 64:
                # The delayed leader may first backtrack to line up a step.
                # Keep their starting lane clear until they actually leave it.
                self.set(self.partner, previous, navigate, swim=swim, run=True)
                active.append(self.partner)
                deferred_follower = False
            if follow_trail:
                feet = leader['feet']
                if leader['grounded'] and distance(trail[-1], feet) >= 96:
                    trail.append(list(feet))
                # These are observed supported positions, never invented
                # offsets on an unknown ledge. Keep a gap behind the leader.
                target = next((p for p in reversed(trail) if distance(p, feet) >= 64), None)
                if target is not None and (self.partner not in active or at_waypoint(self.partner)):
                    old = self.commands[self.partner]['goal']
                    if old is None or distance(old, target) > 48:
                        self.set(self.partner, target, navigate=True, run=True)
                        if self.partner not in active:
                            active.append(self.partner)
                        self.evidence['following_footings'] = self.evidence.get('following_footings', 0) + 1
            return not deferred_follower and all(at_waypoint(r) for r in active)
        self.until(arrived_together)
        self.scene_end()

    def rope(self):
        # Each client advances on its own acknowledged state. A slow follower
        # must not keep the leader climbing through its exit, and an underwater
        # follower must swim to the base before switching to dry approach input.
        phases = {self.first: 'base', self.partner: 'waiting'}
        left_rope = dict.fromkeys(phases, False)
        # Wait along the platform's clear centre line. Sideways waiting spots
        # either meet the original walls or leave the partner blocking step 10.
        parking = {self.first: [-1536, 1408, 256], self.partner: [-1600, 1408, 256]}
        events = self.evidence.setdefault('rope_events', [])
        self.set(self.partner, [-1648, 1408, -486], swim=True, run=True)
        # Saving can leave the leader drifting a little away from the base.
        # Both slots need the same real swim approach before dry rope controls.
        self.set(self.first, [-1728, 1408, -486], swim=True, run=True)

        def phase(role, value):
            phases[role] = value
            events.append(dict(role=role, phase=value, tick=self.reports[role]['frame']['tick'],
                               feet=self.body(role)['player']['feet']))

        def advance():
            for role in (self.first, self.partner):
                body = self.body(role)['player']
                current = phases[role]
                if current == 'release':
                    if body['rope'] is None:
                        left_rope[role] = True
                    elif left_rope[role]:
                        # Only an observed release followed by a new grip allows
                        # another E edge; delayed acknowledgement alone does not.
                        self.set(role, rope=dict(action='climb', rise=0, wish=[1, 0]))
                        phase(role, 'swing')
                        continue
                if current in ('climb', 'hold', 'swing', 'release') and body.get('ledge') is not None:
                    self.set(role, parking[role], run=True, jump=True)
                    phase(role, 'pulling')
                elif current in ('climb', 'hold', 'swing') and body['rope'] is None:
                    # A normal enemy hit can knock Alice off. Re-grab using
                    # ordinary controls; death/warp/overlap checks remain strict.
                    self.set(role, rope=dict(action='approach', goal=[-1728, 1408]))
                    phase(role, 'approach')
                elif current == 'waiting' and self.body(self.first)['player']['feet'][2] > -380:
                    self.set(role, [-1728, 1408, -486], swim=True, run=True)
                    phase(role, 'base')
                elif current == 'base' and (body['rope'] is not None or distance(body['feet'], [-1728, 1408, -486]) < 18):
                    self.set(role, rope=dict(action='approach', goal=[-1728, 1408]))
                    phase(role, 'approach')
                elif current == 'approach' and body['rope'] is not None:
                    self.set(role, rope=dict(action='climb', rise=1))
                    phase(role, 'climb')
                elif current == 'hold' or (current == 'climb' and body['feet'][2] >= 190):
                    assert body['rope'] is not None, 'Lost grip before the rope exit'
                    if role == self.first or self.body(self.first)['player']['feet'][0] > -1660:
                        # Build horizontal swing before either slot releases.
                        # A delayed leader's vertical jump from the rope centre
                        # can hit the landing face before crossing its edge.
                        self.set(role, rope=dict(action='climb', rise=0, wish=[1, 0]))
                        phase(role, 'swing')
                    elif current != 'hold':
                        self.set(role, rope=dict(action='climb', rise=0))
                        phase(role, 'hold')
                elif current == 'swing' and body['feet'][0] > -1716:
                    assert body['rope'] is not None, 'Lost grip during rope swing'
                    self.set(role, rope=dict(action='release', goal=[-1504, 1408], jump=True))
                    left_rope[role] = False
                    phase(role, 'release')
                elif current in ('release', 'pulling') and body['grounded']:
                    assert body['feet'][2] > 250, 'Rope jump missed the upper landing'
                    self.set(role, parking[role], navigate=True)
                    phase(role, 'parking')
                elif current == 'parking' and body['grounded'] and distance(body['feet'], parking[role]) < 24:
                    # This is a waiting area, not a precise trigger. A delayed
                    # steering correction can orbit an eight-unit point forever.
                    # Release movement normally and require its acknowledgement
                    # plus a supported, stationary body before continuing.
                    self.set(role, attack=False)
                    phase(role, 'braking')
                elif current == 'braking' and body['grounded']:
                    if distance(body['feet'], parking[role]) > 32:
                        self.set(role, parking[role], navigate=True, attack=False)
                        phase(role, 'parking')
                    elif self.reports[role]['route_driver']['command'] == self.commands[role]['id'] \
                            and distance(body['velocity'][:2], [0, 0]) < 1:
                        phase(role, 'done')
            return all(value == 'done' for value in phases.values())

        self.until(advance, 60)
        assert all(self.body(r)['player']['grounded'] and self.body(r)['player']['feet'][2] > 250
                   for r in self.commands), 'Both players must finish the native rope landing'
        self.retain('rope-finished')

    def northern_course(self):
        """Both clients traverse every late waypoint without waiting in combat."""
        # Regroup on the clear approach lane. Starting seventy units south
        # gave the delayed client four extra turns around the wall at x=1200.
        self.set(self.first, [1024, 4512, 384], navigate=True, run=True)
        self.set(self.partner, [960, 4512, 384], navigate=True, run=True)
        self.until(lambda: all(self.reached(r) for r in self.commands), 60)
        self.retain('northern-regrouped')
        progress = dict.fromkeys(self.commands, 33)
        events = self.evidence.setdefault('northern_steps', {r: [] for r in self.commands})
        skips = self.evidence.setdefault('revival_skips', [])
        spawned = {r: self.body(r)['spawned_at'] for r in self.commands}
        parked = None

        def command(role):
            index = progress[role]
            step = self.steps[index]
            goal = step.get('nav', step.get('steer'))[:3]
            self.set(role, goal, navigate='nav' in step, run=True)

        for role in self.commands:
            command(role)

        def advance():
            nonlocal parked
            assert not self.arrived() and not self.busy(), 'Northern course was interrupted by a scene'
            for role in (self.first, self.partner):
                index = progress[role]
                personal = self.body(role)
                if personal['stats']['sanity'] <= 0:
                    continue
                if personal['spawned_at'] != spawned[role]:
                    assert self.allow_revival, 'Strict northern route revived a player'
                    spawned[role] = personal['spawned_at']
                    other = self.partner if role == self.first else self.first
                    if progress[other] > index:
                        skipped = list(range(index, progress[other]))
                        assert set(skipped) <= {e['step'] for e in events[other]}, 'Revival would skip untraversed team ground'
                        skips.append(dict(role=role, epoch=self.reports[role]['frame']['epoch'],
                                          spawn_tick=spawned[role], steps=skipped))
                        progress[role] = progress[other]
                        index = progress[role]
                        print('RECOVERY northern waypoints carried by survivor', role, skipped, flush=True)
                    if index <= 44:
                        command(role)
                    elif parked == role:
                        self.set(role, [3376, 2944, 0], navigate=True, run=True)
                    else:
                        self.set(role, attack=True)
                if index > 44:
                    continue
                body = self.body(role)['player']
                driver = self.reports[role]['route_driver']
                target = self.commands[role]
                delta = [a-b for a, b in zip(body['feet'], target['goal'])]
                # The northern ground waypoints are travel points, not
                # landings: advance while grounded and within 24 units rather
                # than standing still under repeated knockback. The descent
                # and final bridge (43/44) retain settled-arrival checks.
                passing = 33 <= index <= 42 and body['grounded'] and abs(delta[2]) < 35 \
                    and distance(delta[:2], [0, 0]) < 24 \
                    and driver['command'] == target['id'] \
                    and driver['observed_tick'] == self.reports[role]['frame']['tick']
                if not (self.reached(role) or passing):
                    continue
                events[role].append(dict(step=index, tick=self.reports[role]['frame']['tick'],
                    feet=list(body['feet']), passing=passing))
                print('PASS northern waypoint', role, index, flush=True)
                progress[role] += 1
                if index < 44:
                    command(role)
                elif parked is None:
                    # Continue a short distance along the authored bridge
                    # approach so the second player can reach the same point.
                    parked = role
                    self.set(role, [3376, 2944, 0], navigate=True, run=True)
                else:
                    self.set(role, attack=True)
            self.step = min(min(progress.values()), 44)
            return all(i == 45 for i in progress.values()) and self.reached(parked) \
                and all(self.body(r)['stats']['sanity'] > 0 for r in self.commands)

        self.until(advance, 240)
        verify_northern_coverage(events, skips)
        for role in self.commands:
            self.set(role, attack=True)
        self.completed.extend(range(33, 45))
        self.retain('northern-finished')

    def clear(self):
        for role in self.commands:
            self.set(role, attack=True)
        def cleared():
            state = self.state()
            if not state or self.busy():
                return True
            # A live Knight on the floor below is nearby but cannot be fought
            # through the platform. Use native hostile selection and actual
            # world/body occlusion, as the single-player clear route does.
            # Missing diagnostics or an unacknowledged command cannot pass.
            return all(report['route_driver']['command'] == self.commands[role]['id']
                and report['route_driver']['observed_tick'] == report['frame']['tick']
                and report['route_driver']['native_threats'][0 if role == 'host' else 1] is None
                for role, report in self.reports.items())
        self.until(cleared, 120)
        self.scene_end()

    save_checkpoint = ChessPair.save_checkpoint

    def boss(self):
        corner = {'host': 0, 'guest': 2}
        chosen = {}
        logged_tick = None
        damage_progress = None
        end = time.monotonic() + 600
        while not self.arrived():
            assert time.monotonic() < end, 'King fight/outro timed out'
            if self.busy():
                self.scene_end()
                continue
            for role in self.commands:
                state = self.state(role)
                if not state or state['phase'] != 'Fight':
                    continue
                body = self.body(role)
                slot = 0 if role == 'host' else 1
                personal = state.get('coop')
                assert personal, 'King has no personal pickup state'
                if body['stats']['sanity'] < 65 or body['stats']['will'] < 5:
                    at = HEALING[personal['essence'][slot]]
                    goal = [at[0] + (-20 if slot == 0 else 20), at[1], at[2]]
                else:
                    if distance(body['player']['feet'][:2], CORNERS[corner[role]][:2]) < 55:
                        corner[role] = (corner[role]+1) % 4
                    goal = list(CORNERS[corner[role]])
                if chosen.get(role) != goal:
                    self.set(role, goal, run=True, attack=True)
                    chosen[role] = goal
            self.pump()
            host_tick = self.reports['host']['frame']['tick']
            state = self.state()
            if state and state['phase'] == 'Fight':
                health = state['boss']['health']
                if damage_progress is None or health < damage_progress[0]:
                    damage_progress = (health, host_tick)
                assert host_tick - damage_progress[1] <= 120 * 120, (
                    'King fight made no damage progress for 120 simulation seconds: '
                    + json.dumps(state['boss']))
            tick = host_tick // 1200
            if tick != logged_tick:
                state = self.state()
                if state:
                    print('King progress', state['phase'], 'boss health', state['boss']['health'],
                          'players', [self.body(r)['stats']['sanity'] for r in self.commands], flush=True)
                logged_tick = tick

    def verify(self):
        assert self.completed == list(range(self.args.start_step, len(self.steps))), 'Incomplete requested approach'
        expected_scenes = set(SCENES)
        if self.saved:
            initial = self.saved['frame']['combat']['stage']['world']['levels'][self.map_name]
            if initial['beheaded']:
                expected_scenes.remove('cinema_beheading_thread')
        for role, rows in self.history.items():
            route = [r for r in rows if r['map'] == self.map_name and r['epoch'] == 1]
            self.verify_presence(route, role)
            owners = {}
            for row in route:
                if row['scene'] and row['owner']:
                    owners.setdefault(row['scene'], set()).add(row['owner'])
            assert set(owners) == expected_scenes, f'Missing scenes: {owners}'
            for scene in expected_scenes - {'cinema_king_killed_thread'}:
                assert owners[scene] == {self.args.first.title()}, 'Wrong first arrival'
            assert len(owners['cinema_king_killed_thread']) == 1, 'Victory owner changed'
            final = self.last_valid[role]['combat']['stage']['world']['levels'][self.map_name]
            assert final['boss']['health'] == 0 and final['beheaded']
            assert self.reports[role]['visit']['entry'] == 'funhouse_start1'
            if 7 in self.completed:
                assert all(any(r['players'][i]['rope'] is not None for r in route) for i in (0, 1)), 'Both players must climb the native rope'
            self.evidence[role] = dict(owners={k: sorted(v) for k, v in owners.items()}, samples=len(route))
        for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'), ('native_sha256', 'matching_native_residents')]:
            self.verify_network_states(field, metric)

    def after_resume(self):
        pass

    def run(self):
        self.start_pair()
        if self.saved:
            receipt = json.loads((self.folder / 'host' / 'route-resume.json').read_text())
            verify_resume_receipt(receipt, self.saved, self.build)
            self.evidence['exact_resume_receipt'] = receipt
            for report in self.reports.values():
                assert report['visit'] == self.saved['visit'], 'Resume changed visit'
                elapsed = report['frame']['tick'] - self.saved['frame']['tick']
                for current, old in zip(report['frame']['players'], self.saved['frame']['players']):
                    verify_resume_body(current, old, elapsed)
            self.after_resume()
        for index in range(self.args.start_step, self.args.last_step + 1):
            if index in self.completed:
                continue
            step = self.steps[index]
            self.step = index
            self.retain(f'before-{index:03d}')
            if index in self.args.save_before:
                self.save_checkpoint(index)
            if index == 25 and all(self.state(r)['beheaded'] for r in self.commands):
                # Either approach can enter the real trigger before the final
                # pre-scene waypoint. The original scene has already placed
                # both players beyond it; walking back can block the leader
                # against the reunited partner. Require the observed scene,
                # owner and reunion before accepting that obsolete waypoint.
                proof = {role: verify_beheading(rows, self.args.first, role, self.verify_presence)
                         for role, rows in self.history.items()}
                self.evidence.setdefault('scene_superseded_steps', []).append(
                    dict(step=index, scene='cinema_beheading_thread', proof=proof))
            elif index == 33 and self.args.last_step == 44:
                self.northern_course()
                continue
            elif index == 7:
                self.rope()
            elif index in (8, 9):
                pass  # Completed by independently acknowledged rope controls above.
            elif 'nav' in step:
                self.go(step['nav'], navigate=True)
            elif 'steer' in step:
                # The narrow doorway needs a planned line from either client's
                # actual approach position; straight steering can meet its edge.
                self.go(step['steer'][:3], navigate=index == 22, swim=index in (4, 5, 6))
            elif 'clear' in step:
                self.clear()
            elif 'beheading' in step:
                self.until(lambda: self.busy() or all(self.state(r)['beheaded'] for r in self.commands), 30)
                self.scene_end()
            else:
                raise AssertionError('Unknown Checkmate route input')
            if index == 26 and self.args.start_step < 26:
                self.evidence['beheading'] = {role: verify_beheading(rows, self.args.first, role, self.verify_presence)
                                             for role, rows in self.history.items()}
            self.completed.append(index)
            print('PASS Checkmate approach', index, 'health', [self.body(r)['stats']['sanity'] for r in self.commands], flush=True)
        full_route = self.saved is None and self.args.last_step == len(self.steps)-1
        through_exit = full_route or self.args.through_exit
        if through_exit:
            self.step = len(self.steps)
            # A revival can leave the partner directly ahead on the bridge.
            # Walk them aside before following the leader's observed footings;
            # standing still here blocks the leader against the contact body.
            self.set(self.first, attack=True)
            aside = list(self.body(self.partner)['player']['feet'])
            aside[1] += 64
            self.set(self.partner, aside, navigate=True)
            self.until(lambda: self.reached(self.partner) or self.busy(), 45)
            self.go([3936, 2944, 0], navigate=True)
            self.boss()
            self.verify()
        else:
            assert self.completed == list(range(self.args.start_step, self.args.last_step+1))
            for role, rows in self.history.items():
                self.verify_presence(rows, role)
                if 7 in self.completed:
                    assert all(any(row['players'][i]['rope'] is not None for row in rows) for i in (0, 1))
            for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'),
                                  ('native_sha256', 'matching_native_residents')]:
                self.verify_network_states(field, metric)
        for role in ('guest', 'host'):
            self.set(role, finish=True, epoch=self.reports[role]['visit']['epoch'])
            self.until(lambda: self.processes[role].poll() is not None, 30)
            assert self.processes[role].returncode == 0
        self.evidence['completed_steps'] = self.completed
        if self.allow_revival:
            recovery = read_trace(self.folder / 'host' / 'recovery-trace.jsonl', self.build, self.args.require_revival)
            recovery['rendered_revivals'] = verify_views(recovery['revivals'], self.history)
            for skip in self.evidence.get('revival_skips', []):
                player = 0 if skip['role'] == 'host' else 1
                assert any(e['player'] == player and e['epoch'] == skip['epoch'] and e['spawn_tick'] == skip['spawn_tick']
                           for e in recovery['revivals']), 'Waypoint skip has no verified native revival'
            self.evidence['recovery'] = recovery
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route=full_route,
            mode='verified-individual-revival' if self.allow_revival else 'strict-no-death',
            through_exit=through_exit,
            internet=not self.args.local, build=self.build, first=self.args.first,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence), indent=2))
        print('PASS full independent Checkmate route through Funhouse' if full_route else
              ('PASS independent Checkmate checkpoint route through Funhouse; not a full route' if through_exit else
               f'PASS independent Checkmate segment {self.args.start_step}..{self.args.last_step}; not a full route'), flush=True)

    def close(self):
        try:
            self.retain('last-reports')
        finally:
            super().close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--skip-scenes', action='store_true')
    parser.add_argument('--allow-revival', action='store_true', help='Verify ordinary individual revivals during the route; team wipes remain a separate gate')
    parser.add_argument('--require-revival', action='store_true', help='Require an actual death, survivor camera and exact ten-second/25-health revival')
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--start-step', type=int, default=0)
    parser.add_argument('--last-step', type=int, default=44)
    parser.add_argument('--through-exit', action='store_true', help='Continue the final approach segment through the King and Funhouse; resumed runs remain partial')
    parser.add_argument('--save-before', nargs='*', type=int, default=[7, 12, 27, 33])
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if args.require_revival and not args.allow_revival:
        parser.error('--require-revival requires --allow-revival')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if not (0 <= args.start_step <= args.last_step < 45) or args.start_step in (8, 9):
        parser.error('Choose an ordered segment within 0..44, starting before the whole rope procedure')
    if (args.start_step > 0) != bool(args.checkpoint):
        parser.error('A later start requires an unchanged checkpoint; an entrance run starts at zero')
    if args.through_exit and args.last_step != 44:
        parser.error('Continuing through the exit requires the final approach step, 44')
    if args.start_step <= 33 and args.last_step == 44 and any(34 <= i <= 44 for i in args.save_before):
        parser.error('The northern course runs continuously; save before step 33 instead')
    pair = CheckmatePair(args)
    try:
        pair.run()
    except BaseException as error:
        pair.evidence['error'] = repr(error)
        pair.retain('failed-result')
        raise
    finally:
        pair.close()
