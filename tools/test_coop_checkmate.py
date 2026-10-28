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


HEALING = [(3968, 3392, 0), (3968, 2624, 0), (4352, 3008, 0)]
CORNERS = [(3940, 2630, 0), (4470, 2630, 0), (4470, 3370, 0), (3940, 3370, 0)]
SCENES = {'cinema_beheading_thread', 'cinema_king_battle_thread', 'cinema_king_killed_thread'}


def verify_beheading(rows, first, role):
    scene = [r for r in rows if r['scene'] == 'cinema_beheading_thread']
    assert scene, 'Original beheading scene was not observed'
    assert {r['owner'] for r in scene} == {first.title()}, 'Wrong beheading owner'
    partner = 1 if first == 'blue' else 0
    assert all(not r['presentation']['visible_players'][partner] for r in scene), 'Scene displayed the waiting partner'
    verify_presence(rows, role)
    final = rows[-1]
    assert final['owner'] is None and final['checkmate']['beheaded'], 'Beheading did not finish'
    assert all(final['presentation']['visible_players']), 'Players did not reappear after the scene'
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
                     'test_coop_castling.py', 'test_coop_chess_resume.py', 'coop_diagnostics.py'):
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
            attack = goal is not None
        if navigate:
            kwargs['run'] = False  # Planned controls carry their own running input.
        self.swim_requested[role] = swim
        swimming = role in self.reports and self.body(role)['player']['swimming']
        super().set(role, goal, navigate, attack=attack, **kwargs)
        self.commands[role].update(swim=swim and swimming, prefer_cards=True, watch_dialogue=True,
            skip_scene=self.args.skip_scenes, input_lead=self.lead if role == 'guest' else 0,
            arrival_radius=8)

    def body(self, role):
        return self.reports[role]['frame']['players'][0 if role == 'host' else 1]

    def state(self, role='host'):
        return self.reports[role]['combat']['stage']['world']['levels'].get(self.map_name)

    def busy(self):
        return any(r['combat']['stage']['owner'] is not None for r in self.reports.values())

    def retain(self, name):
        (self.folder / (name + '.json')).write_text(json.dumps(dict(
            step=self.step, completed=self.completed, commands=self.commands, reports=self.reports)))

    def validate_report(self, role, report):
        try:
            super().validate_report(role, report)
            bodies = report['frame']['players']
            assert all(p['spawned_at'] == 0 and not p['stats']['god'] and not p['stats']['notarget'] for p in bodies), 'Respawn or cheats'
            if report['combat']['stage']['owner'] is None:
                a, b = [p['player']['feet'] for p in bodies]
                assert distance(a[:2], b[:2]) >= 35.95 or abs(a[2]-b[2]) >= 55.95, 'Players overlap'
        except Exception:
            (self.folder / ('failure-' + role + '.json')).write_text(json.dumps(dict(
                before=self.last_valid.get(role), after=report, commands=self.commands, step=self.step)))
            raise
        if report['visit']['map'] == self.map_name:
            self.last_valid[role] = report

    def pump(self):
        if (self.folder / 'stop-test').exists():
            raise RuntimeError('Checkmate stopped; evidence retained')
        for role, requested in self.swim_requested.items():
            if requested and role in self.reports:
                # Walk/fall into water first; the existing swim command requires
                # an already swimming body and never changes its movement mode.
                self.commands[role]['swim'] = self.body(role)['player']['swimming']
        super().pump()
        guest = self.reports.get('guest')
        if guest:
            rtt = (guest.get('network_timing') or {}).get('round_trip_ms')
            if rtt is not None and 0 <= rtt < 10000:
                self.lead = max(self.minimum_lead, .8*self.lead + .2*min(1.5, .05+rtt/1000))
                self.commands['guest']['input_lead'] = self.lead
        for role, report in self.reports.items():
            row = self.history[role][-1]
            row['checkmate'] = self.state(role)
            row['presentation'] = report.get('presentation')

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
        """Leader takes the next waypoint; partner follows its preceding footing."""
        self.scene_end()
        if self.arrived():
            return
        previous = self.body(self.first)['player']['feet']
        active = [self.first]
        if distance(previous, goal) >= 96 and distance(self.body(self.partner)['player']['feet'], previous) > 48:
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
        self.until(lambda: self.arrived() or self.busy() or all(at_waypoint(r) for r in active))
        self.scene_end()

    def rope(self):
        # Each client advances on its own acknowledged state. A slow follower
        # must not keep the leader climbing through its exit, and an underwater
        # follower must swim to the base before switching to dry approach input.
        phases = {self.first: 'approach', self.partner: 'waiting'}
        left_rope = dict.fromkeys(phases, False)
        # Wait along the platform's clear centre line. Sideways waiting spots
        # either meet the original walls or leave the partner blocking step 10.
        parking = {self.first: [-1536, 1408, 256], self.partner: [-1600, 1408, 256]}
        events = self.evidence.setdefault('rope_events', [])
        self.set(self.partner, [-1648, 1408, -486], swim=True, run=True)
        self.set(self.first, rope=dict(action='approach', goal=[-1728, 1408]))

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
                elif current == 'base' and distance(body['feet'], [-1728, 1408, -486]) < 18:
                    self.set(role, rope=dict(action='approach', goal=[-1728, 1408]))
                    phase(role, 'approach')
                elif current == 'approach' and body['rope'] is not None:
                    self.set(role, rope=dict(action='climb', rise=1))
                    phase(role, 'climb')
                elif current == 'hold' or (current == 'climb' and body['feet'][2] >= 190):
                    assert body['rope'] is not None, 'Lost grip before the rope exit'
                    if role == self.first or self.body(self.first)['player']['feet'][0] > -1660:
                        if role == self.first:
                            self.set(role, rope=dict(action='release', goal=[-1504, 1408], jump=True))
                            left_rope[role] = False
                            phase(role, 'release')
                        else:
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
                elif current == 'parking' and self.reached(role):
                    phase(role, 'done')
            return all(value == 'done' for value in phases.values())

        self.until(advance, 60)
        assert all(self.body(r)['player']['grounded'] and self.body(r)['player']['feet'][2] > 250
                   for r in self.commands), 'Both players must finish the native rope landing'
        self.retain('rope-finished')

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

    def verify(self):
        assert self.completed == list(range(len(self.steps))), 'Incomplete approach'
        for role, rows in self.history.items():
            route = [r for r in rows if r['map'] == self.map_name and r['epoch'] == 1]
            verify_presence(route, role)
            owners = {}
            for row in route:
                if row['scene'] and row['owner']:
                    owners.setdefault(row['scene'], set()).add(row['owner'])
            assert set(owners) == SCENES, f'Missing scenes: {owners}'
            for scene in SCENES - {'cinema_king_killed_thread'}:
                assert owners[scene] == {self.args.first.title()}, 'Wrong first arrival'
            assert len(owners['cinema_king_killed_thread']) == 1, 'Victory owner changed'
            final = self.last_valid[role]['combat']['stage']['world']['levels'][self.map_name]
            assert final['boss']['health'] == 0 and final['beheaded']
            assert self.reports[role]['visit']['entry'] == 'funhouse_start1'
            assert all(any(r['players'][i]['rope'] is not None for r in route) for i in (0, 1)), 'Both players must climb the native rope'
            self.evidence[role] = dict(owners={k: sorted(v) for k, v in owners.items()}, samples=len(route))
        for field, metric in [('sha256', 'matching_player_states'), ('world_sha256', 'matching_world_states'), ('native_sha256', 'matching_native_residents')]:
            self.verify_network_states(field, metric)

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
        for index in range(self.args.start_step, self.args.last_step + 1):
            step = self.steps[index]
            self.step = index
            self.retain(f'before-{index:03d}')
            if index in self.args.save_before:
                self.save_checkpoint(index)
            if index == 7:
                self.rope()
            elif index in (8, 9):
                pass  # Completed by independently acknowledged rope controls above.
            elif 'nav' in step:
                self.go(step['nav'], navigate=True)
            elif 'steer' in step:
                self.go(step['steer'][:3], swim=index in (4, 5, 6))
            elif 'clear' in step:
                self.clear()
            elif 'beheading' in step:
                self.until(lambda: self.busy() or all(self.state(r)['beheaded'] for r in self.commands), 30)
                self.scene_end()
            else:
                raise AssertionError('Unknown Checkmate route input')
            if index == 26 and self.args.start_step < 26:
                self.evidence['beheading'] = {role: verify_beheading(rows, self.args.first, role)
                                             for role, rows in self.history.items()}
            self.completed.append(index)
            print('PASS Checkmate approach', index, 'health', [self.body(r)['stats']['sanity'] for r in self.commands], flush=True)
        full_route = self.saved is None and self.args.last_step == len(self.steps)-1
        if full_route:
            self.step = len(self.steps)
            self.go([3936, 2944, 0], navigate=True)
            self.boss()
            self.verify()
        else:
            assert self.completed == list(range(self.args.start_step, self.args.last_step+1))
            for role, rows in self.history.items():
                verify_presence(rows, role)
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
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True, full_route=full_route,
            internet=not self.args.local, build=self.build, first=self.args.first,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence), indent=2))
        print('PASS full independent Checkmate route through Funhouse' if full_route else
              f'PASS independent Checkmate segment {self.args.start_step}..{self.args.last_step}; not a full route', flush=True)

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
    parser.add_argument('--checkpoint', type=Path)
    parser.add_argument('--start-step', type=int, default=0)
    parser.add_argument('--last-step', type=int, default=44)
    parser.add_argument('--save-before', nargs='*', type=int, default=[7, 12, 27, 44])
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if not (0 <= args.start_step <= args.last_step < 45) or args.start_step in (8, 9):
        parser.error('Choose an ordered segment within 0..44, starting before the whole rope procedure')
    if (args.start_step > 0) != bool(args.checkpoint):
        parser.error('A later start requires an unchanged checkpoint; an entrance run starts at zero')
    pair = CheckmatePair(args)
    try:
        pair.run()
    except BaseException as error:
        pair.evidence['error'] = repr(error)
        pair.retain('failed-result')
        raise
    finally:
        pair.close()
