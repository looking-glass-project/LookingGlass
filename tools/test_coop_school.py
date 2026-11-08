"""Traverse the first school visit through two independent internet clients.

The authored theatre, library, flying books, lifts and recipe exit are reached
with ordinary movement/attack inputs. No positions, resources or quest state
are written. Death, focused combat and optional secret checks remain separate.
"""
import argparse
import copy
import hashlib
import json
import time
from pathlib import Path

from test_coop_pandemonium import PandemoniumPair
from test_coop_play import ROOT, distance


class SchoolPair(PandemoniumPair):
    map_name = 'skool1'
    next_map = 'skool2'
    evidence_prefix = 'school-route-'

    def __init__(self,args):
        self.middle_commands = {}
        self.middle_done = set()
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def set(self,role,*args,runup=None,speed_limit=None,arrival_radius=8.0,**kwargs):
        super().set(role,*args,**kwargs)
        self.commands[role]['input_lead'] = min(1.5,.05+2*(self.args.delay_ms+self.args.jitter_ms)/1000) if role == 'guest' else 0
        self.commands[role]['runup'] = runup
        self.commands[role]['speed_limit'] = speed_limit
        self.commands[role]['arrival_radius'] = arrival_radius

    def school(self,role='host'):
        return self.reports[role]['combat']['stage']['world']['school']

    def pump(self):
        if (self.folder/'stop-test').exists():
            raise RuntimeError('School test stopped; partial evidence retained')
        super().pump()
        for role,report in self.reports.items():
            if self.history[role]:
                self.history[role][-1]['school'] = report['combat']['stage']['world']['school']
                self.history[role][-1]['network_timing'] = report.get('network_timing')
                self.history[role][-1]['frame_timing'] = report.get('frame_timing')
        if len(self.reports) == 2 and not self.arrived():
            scene = self.reports['host']['route_driver']['scene']
            if scene and scene == self.reports['guest']['route_driver']['scene']:
                threshold = 20 if scene == 'Theatre_Cinematic' else 2.5
                if scene not in self.middle_commands and all(
                        self.school(r)['first_cinema']['time'] >= threshold for r in self.reports):
                    pending = {}
                    for role,old in list(self.commands.items()):
                        self.set(role,old['goal'],old['navigate'],capture=True,
                                 horizontal=old['horizontal'],run=old['run'],jump=old['jump'])
                        pending[role] = self.commands[role]['id']
                    self.middle_commands[scene] = pending
            for scene,pending in self.middle_commands.items():
                if scene not in self.middle_done and all((self.folder/r/f'route-capture-{i}.png').exists()
                                                         for r,i in pending.items()):
                    self.middle_done.add(scene)
                    self.evidence.setdefault('middle_captures',{})[scene] = pending

    def reached(self,role):
        return super().reached(role) and self.bodies(role)[0 if role == 'host' else 1]['player']['grounded']

    def move_aside(self,role):
        index = 0 if role == 'host' else 1
        anchor = self.bodies()[index]['player']['feet']
        incoming = self.bodies()[1-index]['player']['feet']
        candidates = self.reports['host']['route_driver']['neighbors'][index]
        def clears_partner(point):
            if min(abs(anchor[2]-incoming[2]),abs(point[2]-incoming[2])) >= 56:
                return True
            # Neighbors describe supported world geometry, not occupancy by
            # the other player. Keep both the endpoint and straight approach
            # outside their body; allow an existing close contact to move away.
            if distance(point[:2],incoming[:2]) < 40:
                return False
            delta = [point[i]-anchor[i] for i in range(2)]
            length2 = sum(v*v for v in delta)
            along = max(0,min(1,sum((incoming[i]-anchor[i])*delta[i]
                                   for i in range(2))/length2)) if length2 else 0
            nearest = [anchor[i]+along*delta[i] for i in range(2)]
            return distance(nearest,incoming[:2]) >= min(39,distance(anchor[:2],incoming[:2]))-.1
        candidates = [p for p in candidates if clears_partner(p)]
        assert candidates, f'No supported waiting spot near {anchor}'
        # Prefer stepping sideways so the other player can pass in either
        # direction, including when returning from a personal pickup.
        approach = [anchor[i]-incoming[i] for i in range(2)]
        endpoint = max(candidates,key=lambda p:(abs(
            (p[0]-anchor[0])*approach[1]-(p[1]-anchor[1])*approach[0]),distance(p,incoming)))
        self.evidence.setdefault('waiting_spots',[]).append(dict(role=role,anchor=anchor,goal=endpoint,
            tick=self.reports['host']['frame']['tick']))
        for _ in range(3):
            self.set(role,endpoint,speed_limit=20,arrival_radius=3.0)
            self.until(lambda:self.reached(role),15)
            self.set(role)
            # An arrival report can precede the last in-flight walking input.
            # Let it drain, then correct from the settled position if needed.
            timing = self.reports.get(role,{}).get('network_timing') or {}
            self.wait(max(.4,min(3,(timing.get('round_trip_ms') or 0)/1000+.2)))
            if all(r['frame']['players'][index]['player']['grounded'] and
                   distance(r['frame']['players'][index]['player']['feet'][:2],endpoint[:2]) < 3 and
                   abs(r['frame']['players'][index]['player']['feet'][2]-endpoint[2]) < 2
                   for r in self.reports.values()):
                return
        raise AssertionError(f'{role}: did not settle at supported waiting spot {endpoint}')

    def clear_departure(self,goal):
        index = 0 if self.first == 'host' else 1
        a,b = [body['player'] for body in self.bodies()]
        leader,partner = (a,b) if index == 0 else (b,a)
        if not (leader['grounded'] and partner['grounded']):
            return
        start,other = leader['feet'],partner['feet']
        delta = [goal[i]-start[i] for i in range(2)]
        length = distance(goal[:2],start[:2])
        if length < 8 or abs(start[2]-other[2]) >= 56:
            return
        offset = [other[i]-start[i] for i in range(2)]
        along = sum(d*p for d,p in zip(delta,offset))/length
        across_squared = sum(p*p for p in offset)-along*along
        if 0 < along < min(length+36,96) and across_squared < 36*36:
            # A scene may safely reunite the pair in single file. The waiting
            # player can step aside using normal input before the leader walks
            # onward; the route controller must not push through their body.
            self.set(self.first)
            self.wait(.4)
            self.move_aside(self.partner)
            self.evidence.setdefault('departure_clearances',[]).append(dict(
                goal=goal,tick=self.reports['host']['frame']['tick']))

    def go(self,goal=None,navigate=False,marker=None,horizontal=False,together=False,fight=True):
        assert not together and marker is None, 'School paired endpoints require an explicit ordinary route goal'
        if self.arrived():
            return
        self.set(self.partner)
        first_index = 0 if self.first == 'host' else 1
        for _ in range(4):
            self.clear_departure(goal)
            warped = self.bodies()[first_index]['warped_at']
            self.set(self.first,goal,navigate,horizontal=horizontal)
            self.until(lambda: self.arrived() or self.reached(self.first) or (
                self.bodies()[first_index]['warped_at'] > warped and
                all(r['combat']['stage']['owner'] is None for r in self.reports.values())))
            if self.arrived() or self.reached(self.first):
                break
            self.set(self.first)
            self.wait(.4)
        else:
            raise AssertionError('Too many scene handoffs on one school leg')
        if self.arrived():
            return
        # Freeze the leader before choosing a second supported endpoint. Both
        # clients still walk normally, with unchanged collision and landing rules.
        self.set(self.first)
        self.wait(.4)
        # Fight while steering back toward the reached goal. Neutral movement
        # lets successive enemy hits push Alice off a narrow landing before
        # the partner follows. These remain ordinary movement/attack inputs.
        self.set(self.first,goal,horizontal=horizontal)
        if fight:
            self.secure(keep_movement=True)
        if self.arrived():
            return
        self.until(lambda:self.reached(self.first),20)
        self.hold(.4)
        first_index = 0 if self.first == 'host' else 1
        partner_index = 1-first_index
        def leader_at_goal(report):
            player = report['frame']['players'][first_index]['player']
            return (player['grounded'] and distance(player['feet'][:2],goal[:2]) < 42 and
                    (horizontal or abs(player['feet'][2]-goal[2]) < 35))
        self.until(lambda:all(leader_at_goal(r) for r in self.reports.values()),20)
        rejected = []
        for attempt in range(8):
            self.set(self.partner)
            # A projectile already in flight can arrive after secure() has
            # finished. Keep ordinary steering toward the requested landing
            # while the partner follows, and return there before replanning.
            self.clear_departure(goal)
            self.set(self.first,goal,horizontal=horizontal)
            self.until(lambda:self.arrived() or self.reached(self.first),20)
            if self.arrived():
                return
            anchor = self.bodies()[first_index]['player']['feet']
            start = self.bodies()[partner_index]['player']['feet']
            candidates = self.reports['host']['route_driver']['neighbors'][first_index]
            candidates = [p for p in candidates if all(distance(p,old) > 8 for old in rejected)]
            assert candidates, f'No supported paired endpoint near {anchor}; route must move the leader onward'
            endpoint = min(candidates,key=lambda p: distance(p,start))
            assert 47.9 <= distance(endpoint[:2],anchor[:2]) <= 48.1 and abs(endpoint[2]-anchor[2]) <= 18.1
            self.evidence.setdefault('paired_endpoints',[]).append(dict(goal=goal,leader=self.first,
                anchor=anchor,partner_goal=endpoint,attempt=attempt,tick=self.reports['host']['frame']['tick']))
            def displaced():
                return distance(self.bodies()[first_index]['player']['feet'],anchor) > 8
            # In-flight attacks can still move the leader. Replan from the new
            # supported position instead of walking into their occupied slot.
            for planned,refine in ((navigate,False),(False,True)):
                # The broad arrival radius can stop partway down a slope,
                # outside the unchanged two-unit final height tolerance.
                # Refine with ordinary slow walking before releasing input.
                self.set(self.partner,endpoint,planned,horizontal=False,
                    speed_limit=20 if refine else None,arrival_radius=3.0 if refine else 8.0)
                best = distance(self.bodies()[partner_index]['player']['feet'],endpoint)
                progress_tick = self.reports['host']['frame']['tick']
                def approached():
                    nonlocal best,progress_tick
                    if self.arrived() or displaced() or self.reached(self.partner):
                        return True
                    remaining = distance(self.bodies()[partner_index]['player']['feet'],endpoint)
                    if remaining < best-2:
                        best = remaining
                        progress_tick = self.reports['host']['frame']['tick']
                    return not planned and self.reports['host']['frame']['tick']-progress_tick >= 480
                self.until(approached)
                if self.arrived():
                    return
                if displaced():
                    break
                if not self.reached(self.partner):
                    # A safe endpoint beside the leader can be behind a
                    # bookcase from the partner's approach. Try a different
                    # supported endpoint after four seconds without progress.
                    rejected.append(endpoint)
                    self.evidence.setdefault('blocked_pair_approaches',[]).append(dict(
                        goal=goal,endpoint=endpoint,feet=self.bodies()[partner_index]['player']['feet'],
                        tick=self.reports['host']['frame']['tick']))
                    break
            self.set(self.partner)
            self.wait(.4)
            settled = all(
                r['frame']['players'][partner_index]['player']['grounded'] and
                distance(r['frame']['players'][partner_index]['player']['feet'][:2],endpoint[:2]) < 8 and
                abs(r['frame']['players'][partner_index]['player']['feet'][2]-endpoint[2]) < 2
                for r in self.reports.values())
            # A late walking input can carry the guest past an endpoint that
            # was reached in its previous report. Correct from the settled
            # position, retaining the same final distance/height checks.
            if not displaced() and settled and all(leader_at_goal(r) for r in self.reports.values()):
                break
            if not displaced() and not settled:
                # A nearby step can leave the body on a different height even
                # after both approaches report arrival. Repeating this same
                # endpoint cannot satisfy the unchanged final height check.
                rejected.append(endpoint)
                self.evidence.setdefault('unsettled_pair_endpoints',[]).append(dict(
                    goal=goal,endpoint=endpoint,feet=self.bodies()[partner_index]['player']['feet'],
                    tick=self.reports['host']['frame']['tick']))
        else:
            raise AssertionError('Pair did not settle at a safe endpoint after eight attempts')
        # The final straight walk keeps navigation's broad arrival from passing
        # an adjacent stair or an occupied stopping point as the destination.
        for report in self.reports.values():
            bodies = report['frame']['players']
            assert leader_at_goal(report), 'Leader left the requested route landing'
            assert bodies[partner_index]['player']['grounded']
            assert distance(bodies[partner_index]['player']['feet'][:2],endpoint[:2]) < 8
            assert abs(bodies[partner_index]['player']['feet'][2]-endpoint[2]) < 2
        print('PASS paired school leg',goal,'partner',endpoint,flush=True)

    def until(self,predicate,seconds=120):
        end = time.monotonic()+seconds
        while not predicate():
            assert time.monotonic() < end, 'School timeout: '+str({role:dict(
                tick=r['frame']['tick'],phase=r['route_driver']['phase'],command=self.commands[role],
                feet=[b['player']['feet'] for b in r['frame']['players']]) for role,r in self.reports.items()})
            self.pump()

    def secure(self,keep_movement=False):
        if self.arrived():
            return
        if keep_movement:
            self.hold(.5)
        else:
            self.wait(.5)
        def safe():
            for r in self.reports.values():
                if any(e['active'] and e['health'] > 0 and any(v and d < 700
                       for v,d in zip(e['line_of_sight'],e['distance']))
                       for e in r['route_driver']['guards']+r['route_driver']['boojums']):
                    return False
                cast = r['combat']['stage']['world']['encounters']
                if cast and any(a['enemy'].get('Boojum',{}).get('waves') for a in cast['actors']):
                    return False
            return True
        self.until(safe,60)

    def legs(self,goals):
        for goal,navigate in goals:
            self.go(goal,navigate)
            self.secure()

    def passage(self,goals,after_reunion=None,speed_limit=None,partner_done=None,first_exit=None):
        # Narrow stairs have no same-height waiting spot at each bend. Let one
        # player finish the passage and step aside before the other follows
        # exactly the same ordinary route. An authored scene may already have
        # transported the waiting partner beyond the passage.
        partner_index = 0 if self.partner == 'host' else 1
        old_warp = self.bodies()[partner_index]['warped_at']
        def completed_reunion():
            return partner_done is not None and partner_done() and all(
                r['frame']['players'][partner_index]['warped_at'] > old_warp
                and r['combat']['stage']['owner'] is None
                and all(b['player']['grounded'] for b in r['frame']['players'])
                for r in self.reports.values())
        def traverse(role,role_goals):
            for goal,navigate in role_goals:
                self.set(role,goal,navigate,speed_limit=speed_limit if not navigate else None)
                final_pickup = role == self.first and role_goals is goals and goal == goals[-1][0]
                self.until(lambda:self.reached(role) or (final_pickup and completed_reunion()))
                if final_pickup and completed_reunion():
                    # Both actual pickups and a completed, grounded reunion
                    # satisfy this component. Do not push toward the original
                    # point when the scene placed the other body across it.
                    self.secure()
                    return
                # Keep damped ordinary steering during combat rather than
                # restarting a jump plan when delayed movement leaves its pad.
                self.set(role,goal,speed_limit=speed_limit)
                if role == self.first:
                    self.clear_departure(goal)
                    self.set(role,goal,speed_limit=speed_limit)
                self.secure(keep_movement=True)
                if role == self.first:
                    # A cutscene can reunite the waiting partner across this
                    # landing's final approach. Clear it with ordinary input
                    # before refining the owner's arrival after combat.
                    self.clear_departure(goal)
                    self.set(role,goal,speed_limit=speed_limit)
                self.until(lambda:self.reached(role),20)
                self.set(role)
                self.evidence.setdefault('passage_stops',[]).append(dict(role=role,goal=goal,
                    feet=self.bodies()[0 if role == 'host' else 1]['player']['feet'],
                    tick=self.reports['host']['frame']['tick']))

        for role in (self.first,self.partner):
            other = 'guest' if role == 'host' else 'host'
            self.set(other)
            role_goals = goals
            if role == self.partner and after_reunion is not None and self.bodies()[partner_index]['warped_at'] > old_warp:
                self.evidence.setdefault('passage_reunions',[]).append(dict(role=role,
                    warped_at=self.bodies()[partner_index]['warped_at'],
                    feet=self.bodies()[partner_index]['player']['feet']))
                role_goals = after_reunion
            traverse(role,role_goals)
            if role == self.first:
                if partner_done is not None:
                    # A delayed client can report arrival just before the
                    # host starts the scene at this endpoint. Drain ordinary
                    # movement and wait for BOTH scene reports before judging
                    # reunion or asking for a supported sideways waiting spot.
                    self.wait(.5)
                    self.until(lambda: all(r['combat']['stage']['owner'] is None
                        and all(b['player']['grounded'] for b in r['frame']['players'])
                        for r in self.reports.values()), 90)
                # A scene may already have delivered the waiting player's
                # personal pickup. A caller can verify that outcome directly
                # instead of forcing a second approach across an occupied pad.
                if completed_reunion():
                    self.evidence.setdefault('passage_partner_completed',[]).append(dict(
                        role=self.partner,warped_at=self.bodies()[partner_index]['warped_at'],goals=goals))
                    break
                if first_exit:
                    # A small pickup pad may have no space to step aside.
                    # Let the first player leave by an explicit ordinary route
                    # before the partner approaches their personal pickup.
                    traverse(role,first_exit)
                    self.evidence.setdefault('passage_first_exits',[]).append(dict(
                        role=role,goals=first_exit,feet=self.bodies()[0 if role == 'host' else 1]['player']['feet']))
                    # The explicit exit already supplies a separate waiting
                    # place. Do not demand another sideways step on a narrow
                    # walkway when the partner's landing is safely behind it.
                    assert distance(first_exit[-1][0],goals[-1][0]) >= 80
                    continue
                self.move_aside(role)
        print('PASS school passage',goals,flush=True)

    def lift(self,name,first_goal,second_goal,top):
        def obj(role='host'):
            return next(o for o in self.school(role)['objects'] if o['name'] == name)
        self.set(self.partner)
        self.set(self.first,first_goal,navigate=True)
        self.until(lambda: self.reached(self.first),40)
        self.set(self.first,first_goal)
        self.until(lambda: self.reached(self.first),20)
        self.set(self.first)
        initial = {r:copy.deepcopy(obj(r)) for r in self.reports}
        clocks = {r:self.school(r)['clock'] for r in self.reports}
        self.until(lambda: all(self.school(r)['clock'] >= t+2 for r,t in clocks.items()),10)
        for role,before in initial.items():
            assert obj(role)['pose'] == before['pose'], 'Lift left before both players boarded'
            assert obj(role)['motion'] == before['motion'], 'Lift timer advanced while partner absent'
        self.capture_pair(name+'-waiting')
        self.set(self.partner,second_goal,navigate=True)
        # Boarding starts the shared clock before the second rider reaches the
        # planned endpoint. Stop on that host signal: following a fixed world
        # coordinate after rotation begins can walk the delayed guest off.
        # The second player may board well before that endpoint. Requiring
        # proximity to it keeps the jump plan running despite confirmed
        # boarding; the shared clock and both grounded heights are sufficient.
        self.until(lambda: obj()['motion'] != initial['host']['motion'] and
                   all(b['player']['grounded'] and b['player']['feet'][2] >= first_goal[2]-2
                       for b in self.bodies()),40)
        # A body can touch the lift while still overlapping the fixed lower
        # landing. Finish boarding with damped walking, not the jump plan, and
        # release it at the first actual platform movement. Carrying remains
        # blocked safely until both bodies have cleared the fixed landing.
        self.set(self.partner,second_goal)
        self.until(lambda: obj()['pose'] != initial['host']['pose'],20)
        self.set(self.partner)
        self.evidence.setdefault('lift_boarding',[]).append(dict(name=name,
            tick=self.reports['host']['frame']['tick'],motion_time=obj()['motion']['time'],
            feet=[b['player']['feet'] for b in self.bodies()]))
        def at_top():
            return all(b['player']['feet'][2] >= top-1 and b['player']['grounded']
                       for r in self.reports for b in self.bodies(r))
        last_time = obj()['motion']['time']
        last_move = self.reports['host']['frame']['tick']
        adjustments = 0
        deadline = time.monotonic()+40
        while not at_top():
            assert time.monotonic() < deadline, f'{name}: did not reach the upper landing'
            self.pump()
            current = obj()
            if current['motion']['time'] != last_time:
                last_time = current['motion']['time']
                last_move = self.reports['host']['frame']['tick']
            if (name == 'library_spiral_lift' and adjustments < 8 and
                    self.reports['host']['frame']['tick']-last_move >= 120):
                # A rider can meet the central shaft during rotation or the
                # upper landing lip near the top. The shared mover correctly
                # waits. Walk to an observed supported neighbor with room for
                # the other body, then release input as carrying resumes.
                center = current['pose']['origin'][:2]
                starts = [b['player']['feet'] for b in self.bodies()]
                candidates = []
                upper = current['pose']['origin'][2] > top-32
                for index, feet in enumerate(starts):
                    radius = distance(feet[:2], center)
                    for point in self.reports['host']['route_driver']['neighbors'][index]:
                        gain = distance(point[:2], center)-radius
                        if (abs(point[2]-feet[2]) < 2 and distance(point,starts[1-index]) >= 40
                                and (gain > 4 or upper and gain < -4)):
                            candidates.append(((-gain if upper else gain), index, point))
                assert candidates, f'{name}: no supported clearance step for riders at {starts}'
                _, index, target = max(candidates, key=lambda c:c[0])
                role = 'host' if index == 0 else 'guest'
                self.evidence.setdefault('lift_clearance',[]).append(dict(
                    name=name,tick=last_move,feet=starts,motion_time=last_time,
                    role=role,target=target))
                self.set(role,target)
                self.until(lambda: obj()['motion']['time'] > last_time+.1 or self.reached(role),10)
                self.set(role)
                adjustments += 1
                last_time = obj()['motion']['time']
                last_move = self.reports['host']['frame']['tick']
        for role in self.commands:
            self.set(role)
        self.capture_pair(name+'-top')
        self.evidence.setdefault('lifts',[]).append(dict(name=name,waiting_seconds=2,
            both_reached_height=top,first=self.args.first))
        print('PASS school paired boarding and ride:',name,flush=True)

    def lower_library_landing(self):
        # A full-speed, timed plan can overshoot this narrow shelf under jitter.
        # Walk slowly off its edge, then steer back toward the lower shelf from
        # observed motion. These are ordinary inputs, including real fall damage.
        for role in (self.first,self.partner):
            other = 'guest' if role == 'host' else 'host'
            index = 0 if role == 'host' else 1
            self.set(other)
            self.set(role,(56,2780,384))
            self.until(lambda: self.reached(role),25)
            self.set(role,(96,2780,384))
            # Turn back at the first loss of footing. Waiting for a two-unit
            # fall can miss that first report and lose most of the air time.
            self.until(lambda: not self.bodies(role)[index]['player']['grounded'],15)
            self.set(role,(-32,2780,64))
            self.until(lambda: self.reached(role),20)
            assert abs(self.bodies(role)[index]['player']['feet'][2]-64) < 2, 'Missed the lower library shelf'
            self.set(role)
            if role == self.first:
                self.wait(.4)
                self.move_aside(role)
        self.capture_pair('lower-library-landing')
        self.secure()

    def mallet_landing(self):
        # The altar is across the stairwell, not along a continuous balcony.
        # Stop on the lower stair before the short jump to the altar platform.
        # Walk down its centre: a broad navigation stop can leave running
        # momentum carrying the delayed guest off the side of the stairs.
        self.go((-2304,2800,-256))
        # The second player waits beside this landing. Align them on the
        # stair centre before descending instead of cutting diagonally across
        # its unsupported side from their separate waiting spot.
        self.passage([((-2304,2800,-256),False),((-2304,2624,-342),False),((-2304,2544,-352),False),
                      ((-2272,2464,-256),True)],speed_limit=20)

    def checkpoint_stop(self,label):
        if label not in getattr(self.args,'checkpoint_stops',[]):
            return
        for role in self.commands:
            self.set(role)
        self.wait(.5)
        (self.folder/f'ready-for-{label}.json').write_text(json.dumps(dict(
            tick=self.reports['host']['frame']['tick'],build=self.build)))
        print(f'Ready for ordinary menu checkpoint at {label}; create continue-{label} to proceed.',flush=True)
        self.until(lambda: (self.folder/f'continue-{label}').exists(),180)

    def save_checkpoint(self, label):
        """Ask the host to save normally, then retain the unchanged disk file."""
        epoch = self.reports['host']['visit']['epoch']
        for role in self.commands:
            self.set(role, attack=False, epoch=epoch)
        self.wait(.5)
        self.commands['host']['checkpoint'] = True
        command = self.commands['host']['id']
        self.until(lambda: (self.reports['host']['route_driver'].get('checkpoint') or {}).get('command') == command, 15)
        receipt = self.reports['host']['route_driver']['checkpoint']
        assert receipt['epoch'] == epoch and receipt['error'] is None, receipt
        saved = (self.folder/'host'/'coop-checkpoint.json').read_bytes()
        payload = json.loads(saved)['payload']
        assert payload['visit']['epoch'] == epoch and payload['frame']['tick'] == receipt['tick']
        assert payload['frame']['checkpoint_generation'] == self.reports['host']['frame']['checkpoint_generation']
        path = self.folder/f'checkpoint-{label}.json'
        path.write_bytes(saved)
        self.evidence.setdefault('checkpoints', {})[label] = dict(
            file=path.name, sha256=hashlib.sha256(saved).hexdigest(), **receipt)
        for role in self.commands:
            self.set(role, attack=True, epoch=epoch)
        print(f'PASS ordinary checkpoint retained at {label}', flush=True)

    def balcony_gap(self):
        # A short running approach followed by feedback steering is less brittle
        # than replaying a timed sequence of sideways inputs across this gap.
        for role in (self.first,self.partner):
            # Waiting beside the partner can put Alice on the adjacent sloping
            # stair. Each jumper must first walk to the authored run-up point.
            self.set(role,(-1664,3248,-256))
            self.until(lambda:self.reached(role),20)
            self.wait(.4)
            # Both aim at the actual upper landing. The old guest-only target
            # at y=3072 is over the lower slope, so anticipatory steering can
            # stop there before reaching the balcony.
            goal = (-1664,3040,-256)
            index = 0 if role == 'host' else 1
            jumps = self.bodies()[index]['player']['jumps']
            self.set(role,goal,run=True,jump=True,runup=dict(millis=200,wish=[0,-1]))
            # Air steering takes time to shed running speed. Include braking
            # time as well as network delivery when aiming at this short ledge.
            self.commands[role]['input_lead'] += .30
            def landed():
                body = self.bodies()[index]['player']
                return (body['jumps'] > jumps and body['grounded'] and
                        abs(body['feet'][2]-goal[2]) < 20 and distance(body['feet'],goal) < 45)
            # Reports may skip the brief descending portion under load. A
            # confirmed landing is also a valid time to release run; requiring
            # an already-past airborne frame would time out on a safe crossing.
            self.until(lambda: self.bodies()[index]['player']['jumps'] > jumps
                       and ((not self.bodies()[index]['player']['grounded']
                             and self.bodies()[index]['player']['velocity'][2] < 0)
                            or landed()),8)
            # Release run before touchdown: grounded acceleration is much
            # stronger and a delayed running correction can cross the ledge.
            self.set(role,goal)
            # Releasing run does not remove airborne momentum. Keep the same
            # braking lead until touchdown instead of abruptly shortening it.
            self.commands[role]['input_lead'] += .30
            self.until(landed,20)
            self.set(role)
            self.wait(.5)
            assert landed(), f'{role}: did not remain on the balcony landing'
            if role == self.first:
                # This landing is too narrow for a supported sideways step.
                # Continue along the ordinary route to the wider balcony.
                waiting = copy.deepcopy(self.bodies()[1-index]['player']['feet'])
                self.set(role,(-1480,3008,-256),navigate=True)
                self.until(lambda:self.reached(role),30)
                self.wait(.4)
                assert self.bodies()[1-index]['player']['feet'] == waiting, 'Balcony waiting player moved'
        self.capture_pair('balcony-gap')
        self.secure()

    def theatre_healing(self):
        self.until(lambda: all(next(g for g in r['route_driver']['guards']
                                   if g['name'] == 'play_guard1')['health'] == 0
                               for r in self.reports.values()),20)
        drops = self.bodies()[0]['stats']['loot']['skool1$first']['drops']
        reward = next(d for d in drops if d['id'] == 3000019)
        self.go(reward['origin'],fight=False)
        self.move_aside(self.first)
        self.set(self.partner,reward['origin'])
        self.until(lambda:self.reached(self.partner),20)
        self.wait(.4)
        self.capture_pair('theatre-earned-healing')

    def theatre(self,upstairs=True):
        # The entrance spawns the pair in single file. Clear the waiting
        # player's supported spot before asking the leader to walk north;
        # otherwise the Blue-first route walks directly into Red's body.
        self.move_aside(self.partner)
        self.legs([((-1888,2290,-512),False),((-2340,2368,-480),True)])
        self.set(self.partner)
        self.set(self.first,(-2560,2368,-480))
        self.until(lambda: all(r['combat']['stage']['owner'] is not None for r in self.reports.values()),30)
        self.until(lambda: all(r['combat']['stage']['owner'] is None for r in self.reports.values()),120)
        self.capture_pair('theatre-reunion')
        # The original scene lands both players on the lower theatre floor.
        # Align with the stair opening, collect the fresh nearby reward and
        # walk both players through the stairs before waiting on combat. The
        # guard on the side stair is visible but these fixed firing positions
        # strike its lip; defeating that optional guard is not a route gate.
        # Attacks stay enabled throughout the ordinary climb.
        self.go((-3336,2368,-672),fight=False)
        self.theatre_healing()
        if not upstairs:
            return
        self.go((-3336,2368,-672),fight=False)
        for goal,navigate in [((-2560,2368,-480),False),((-2340,2368,-480),True)]:
            self.go(goal,navigate,fight=False)
        # The switchback has no supported side-by-side stopping point. Finish
        # it and clear the landing before the second player takes the stairs.
        self.passage([((-2340,2368,-480),False),((-2256,2544,-480),False),
                      ((-2304,2624,-342),True),((-2304,2800,-256),True)],speed_limit=20)
        self.capture_pair('theatre-and-shelf-reunion')

    def route(self):
        self.wait(1)
        self.capture_pair('school-entrance')
        self.theatre()
        self.checkpoint_stop('theatre')
        self.after_theatre()

    def after_theatre(self):
        self.collect_mallet()
        self.go((-2272,2440,-256))
        self.return_stairs()
        # The far end is a narrow run-up, so each player approaches it only
        # when it is their turn to jump in balcony_gap().
        self.go((-2304,3248,-256))
        self.checkpoint_stop('balcony')
        self.balcony_gap()
        self.library_route()

    def collect_mallet(self):
        self.mallet_landing()
        # The essence beside the Mallet belongs to the return visit; the first
        # visit uses the fresh theatre reward above for ordinary healing.
        self.go((-2274,2334,-256))
        self.secure()
        self.move_aside(self.first)
        self.set(self.partner,(-2274,2334,-256))
        self.until(lambda:self.reached(self.partner),20)
        self.wait(.4)
        assert all('skool1:78' in b['stats']['collected'] for b in self.bodies()), 'Both players must collect their own Mallet'

    def return_stairs(self):
        # Bring each player from their own waiting spot to the same altar edge.
        # Walk the drop and lower stair before planning the short upper jump;
        # a running plan for the whole return overshoots its narrow supports.
        self.passage([((-2272,2440,-256),False),((-2304,2544,-352),False),
                      ((-2304,2624,-342),False),((-2304,2720,-334),False),
                      ((-2304,2800,-256),True)],speed_limit=20)

    def library_route(self):
        self.legs([((-1480,3008,-256),True),
                   ((-208,3008,-256),False),((-208,3456,-256),False),
                   ((-208,3968,-256),False),((208,3968,-288),False),
                   ((208,3520,-288),False),((700,3520,-264),True),
                   ((1000,3360,-256),False),((944,3440,-256),False),
                   ((700,3440,-256),False),((208,3440,-288),False),
                   ((208,4160,-264),False),((144,4230,-288),False)])
        self.lift('library_spiral_lift',(144,4360,-264),(192,4360,-264),64)
        self.checkpoint_stop('spiral')
        self.after_spiral_lift()

    def after_spiral_lift(self):
        # Leave the rotating platform through its narrow authored landing in
        # turn. A sideways partner endpoint cuts diagonally across open air.
        departure = tuple(self.bodies()[0 if self.first == 'host' else 1]['player']['feet'])
        self.passage([(departure,False),((272,4160,64),False),((700,4096,64),False)],
                     after_reunion=[((700,4096,64),False)])
        self.legs([((884,4056,64),False),((0,4096,64),False)])
        # The bookcase blocks a diagonal approach to the supported spots west
        # of this landing. Clear the straight corridor before the partner follows.
        self.passage([((0,3500,64),False)])
        self.checkpoint_stop('library')
        self.upper_library_route()

    def upper_library_route(self):
        # Clear the west edge of the bookcase before turning toward the stairs.
        self.legs([((-180,3500,64),False),((-180,3300,64),False),((-140,3300,64),False)])
        self.upper_library_gap()
        self.legs([((-464,3520,384),False),
                   ((-464,3800,384),False),((-464,4096,384),False),((-276,4160,384),False)])
        self.lift('elevator2',(-276,4360,400),(-228,4360,400),696)
        self.legs([((-276,4160,704),False),((-212,3860,704),False),((-64,3968,704),False),
                   ((-64,3520,704),False),((-160,3520,704),False)])
        self.wait(8.5)
        self.capture_pair('flying-books-bridge')
        self.checkpoint_stop('bridge')
        self.book_bridge()

    def upper_library_gap(self):
        # The stair top and west landing are separated by open space. Use a
        # deliberate running jump from the settled lip; a broad navigation
        # tolerance can end at a different stair edge and plan a fall below it.
        goal = (-16,3496,384)
        self.wait(.5)
        for role in (self.first,self.partner):
            index = 0 if role == 'host' else 1
            # Keep the waiting player below the stairs until the first has
            # crossed. A nearby paired stop can occupy the take-off point.
            self.set(role,(208,3300,400),navigate=True)
            self.until(lambda:self.reached(role),40)
            self.set(role,(208,3300,400))
            self.secure(keep_movement=True)
            self.until(lambda:self.reached(role),20)
            self.wait(.4)
            # The stair lip has no flat paired waiting slot. Approach this
            # shared run-up only when it is this player's turn to jump.
            self.set(role,(192,3344,400))
            self.until(lambda:self.reached(role),20)
            self.wait(.4)
            jumps = self.bodies()[index]['player']['jumps']
            self.set(role,goal,run=True,jump=True)
            def landed():
                p = self.bodies()[index]['player']
                return (p['jumps'] > jumps and p['grounded'] and
                        abs(p['feet'][2]-goal[2]) < 4 and distance(p['feet'],goal) < 45)
            self.until(landed,15)
            self.set(role,goal)
            self.until(lambda: self.reached(role),10)
            self.set(role)
            self.wait(.5)
            assert landed(), f'{role}: left the upper library landing'
            if role == self.first:
                # Continue west along the gallery before the partner lands.
                # A generic sideways wait leaves the leader behind the second
                # jumper, whose body then blocks the next westbound waypoint.
                self.set(role,(-128,3504,384))
                self.until(lambda:self.reached(role),15)
                self.wait(.4)
                assert abs(self.bodies()[index]['player']['feet'][2]-384) < 2
        self.capture_pair('upper-library-gap')

    def book_jump(self,approach,goal,guest_braking=.30):
        # The smaller books have no side-by-side waiting space at the lip.
        # Let each jumper walk from their landing to the take-off point in turn.
        for role in (self.first,self.partner):
            index = 0 if role == 'host' else 1
            # The second player's waiting slot is farther from the next book.
            # Walk to the same take-off point after the first has cleared it.
            self.set(role,approach)
            self.until(lambda:self.reached(role),20)
            self.wait(.4)
            jumps = self.bodies()[index]['player']['jumps']
            self.set(role,goal,run=True,jump=True,speed_limit=220)
            # Airborne momentum needs braking before a delayed guest receives
            # the landing report; a neutral input alone does not slow the jump.
            braking_lead = guest_braking if role == 'guest' else 0
            self.commands[role]['input_lead'] += braking_lead
            def landed():
                p = self.bodies()[index]['player']
                return (p['jumps'] > jumps and p['grounded'] and
                        abs(p['feet'][2]-goal[2]) < 5 and distance(p['feet'],goal) < 60)
            self.until(landed,15)
            self.set(role,goal)
            self.commands[role]['input_lead'] += braking_lead
            self.until(lambda: self.reached(role),10)
            self.set(role)
            self.wait(.5)
            assert landed(), f'{role}: left the book after landing'
            if role == self.first:
                self.move_aside(role)
        self.capture_pair('book-landing-'+str(goal[1]))

    def book_bridge(self):
        # The first book joins this landing with a four-unit step. Walk onto it
        # directly; a jump search from this narrow edge can spend its entire
        # search budget exploring the shelves below the bridge.
        self.go((-150,3396,708))
        self.book_jump((-130,3334,708),(-40,3186,724))
        self.book_jump((20,3158,724),(136,3014,716))
        # The last gap is longer; retain more forward momentum until reaching
        # its near edge, then brake on the broader final book.
        self.book_jump((160,2990,716),(224,2796,716),guest_braking=.10)
        self.legs([((204,2450,704),True)])
        self.wait(4)
        self.capture_pair('recipe-reunion')
        if self.args.rejoin:
            self.rejoin_guest()
        self.checkpoint_stop('descent')
        self.descent()

    def descent(self):
        # Descend the authored shelves in turn. Sideways paired endpoints can
        # change the next drop into a diagonal route over open air.
        self.passage([((104,2736,624),True),((0,2824,564),True),((-32,2780,384),True)])
        self.lower_library_landing()
        self.legs([((200,2776,-288),True)])
        self.until(self.arrived,90)

    def verify_rejoined_world(self,before,report):
        old = before['combat']['stage']['world']['school']
        new = report['combat']['stage']['world']['school']
        for field in ('books','returning','theatre','library','shelves','recipe','secret_open','secret_claimed'):
            assert new[field] == old[field], f'Rejoin changed school {field}'
        assert new['first_cinema']['beat'] == old['first_cinema']['beat']

    def verify_coherence(self):
        by_tick = {(r['epoch'],r['tick']):r for r in self.history['host']}
        compared = 0
        for row in self.history['guest']:
            other = by_tick.get((row['epoch'],row['tick']))
            if other:
                for field in ('players','owner','school','encounters'):
                    assert row[field] == other[field], f'Authoritative school {field} differs'
                compared += 1
        # Short component runs need not produce coincident local report clocks.
        # New builds record the complete shared world at each sent/restored
        # network tick; require those comparisons instead of relying on luck.
        if any('world_sha256' in state for rows in self.history.values() for row in rows
               for state in row['driver']['network_states']):
            self.verify_network_states('world_sha256', 'matching_shared_world_states')
        else:
            assert compared > 0, 'No coincident full school reports were observed'
        self.evidence['matching_full_school_frames'] = compared
        self.verify_network_states()

    def verify(self):
        expected = {'Theatre_Cinematic','Skool1_OG_MoveShelf','shelf_cinematic',
                    'Skool1_Book_Win','book_cinematic','Book_Ingredients_Exit'}
        assert self.captures == expected, self.captures
        for role,rows in self.history.items():
            visit = [r for r in rows if r['epoch'] == 1]
            owners = {scene:{r['owner'] for r in visit if r['scene'] == scene} for scene in expected}
            assert all(ids == {self.args.first.title()} for ids in owners.values()), owners
            reunions = set()
            for old,row in zip(visit,visit[1:]):
                if old['owner'] is not None and row['owner'] == old['owner']:
                    other = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][other]['feet'] == old['players'][other]['feet'], 'Partner moved during school scene'
                if old['owner'] is not None and row['owner'] is None:
                    a,b = row['players']
                    assert a['warped_at'] == b['warped_at'] > 0
                    assert distance(a['feet'],b['feet']) <= 110+max(0,row['tick']-a['warped_at'])*640/120
                    reunions.add(a['warped_at'])
            assert all(r['checkpoint_generation'] == 0 for r in rows), 'Route silently restored a checkpoint'
            assert all(b['spawned_at'] == 0 and b['health'] > 0 for r in rows for b in r['players']), \
                'Route silently died or respawned'
            assert any(r['school']['books'] == ['Bridge']*4 for r in visit)
            assert any(r['school']['recipe'] for r in visit)
            assert all('skool1:78' in b['stats']['collected'] for b in self.bodies(role)), 'Personal Mallet pickup missing'
            assert self.reports[role]['visit']['entry'] == 'skool2_start1'
            self.evidence[role] = dict(owners={s:sorted(ids) for s,ids in owners.items()},reunions=sorted(reunions))
        self.verify_coherence()

    def run(self):
        self.start_pair()
        self.route()
        self.verify()
        self.finish_pair(2)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,build=self.build,
            internet=not self.args.local,first=self.args.first,delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms,evidence=self.evidence,
            scope='First school traversal, scenes, paired lifts and recipe exit; focused combat and recovery remain separate'),indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=ROOT/'target/debug/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--first',choices=('blue','red'),default='red')
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--rejoin',action='store_true')
    parser.add_argument('--checkpoint-stops',nargs='*',choices=('theatre','balcony','spiral','library','bridge','descent'),default=[],
                        help='Hold neutral inputs for up to 180 seconds for a normal menu checkpoint; create continue-<stop> in the evidence folder to resume')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = SchoolPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
