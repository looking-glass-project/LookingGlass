"""Reject rider/clock resets in the real rotating-lift reconnect verifier."""
import copy
import math
import unittest

from test_coop_school_lift import lift_object, verify_lift_rejoin


def reports():
    start = dict(origin=[208, 4288, -268], angles=[0, 0, 0])
    end = dict(origin=[208, 4288, 60], angles=[0, 0, -540])
    keys = [[0, start], [1, start], [7, end], [12, end]]
    def report(t, tick, pid):
        angle = -(t-1)*90
        z = -268+(t-1)*328/6
        pose = dict(origin=[208, 4288, z], angles=[0, 0, angle])
        obj = dict(name='library_spiral_lift', enabled=True, solid=True,
            pose=pose, motion=dict(keys=copy.deepcopy(keys), repeat=True, time=t))
        bodies = []
        for i,y in enumerate((-30, 30)):
            radians = math.radians(angle)
            feet = [208-80*math.cos(radians)-y*math.sin(radians),
                    4288-80*math.sin(radians)+y*math.cos(radians), z+4.03125]
            stats = dict(sanity=100-i*20, will=30-i*15, owned=[1,1-i], collected=[str(i)],
                selected=0, school_items={}, staff_component=False, difficulty='Normal', loot={})
            bodies.append(dict(stats=stats, player=dict(feet=feet, grounded=True), spawned_at=0, warped_at=0))
        return dict(pid=pid, build='fixture', visit=dict(map='skool1', epoch=1),
            frame=dict(tick=tick, checkpoint_generation=0, players=bodies),
            combat=dict(stage=dict(owner=None, world=dict(school=dict(clock=48+t, objects=[obj])))))
    return report(2,100,1), report(2.1,112,3)


class LiftGuards(unittest.TestCase):
    def test_rotation_and_translation_preserve_both_rider_offsets(self):
        before, after = reports()
        result = verify_lift_rejoin(before, after, 2)
        self.assertAlmostEqual(result['motion_after']-result['motion_before'], .1)
        for row in result['rider_offsets']:
            for a,b in zip(row['before'], row['after']):
                self.assertAlmostEqual(a,b)

    def test_unchanged_world_position_is_not_attachment_to_a_moving_lift(self):
        for player in (0,1):
            before, after = reports()
            after['frame']['players'][player]['player']['feet'] = before['frame']['players'][player]['player']['feet']
            with self.subTest(player=player), self.assertRaisesRegex(AssertionError, 'relative to the lift'):
                verify_lift_rejoin(before, after, 2)

    def test_stale_process_clock_motion_pose_and_world_resets_fail(self):
        for fault in ('pid','clock','wipe','visit','owner','reset','skip','keys','pose','disabled','world_clock'):
            before, after = reports()
            obj = lift_object(after)
            if fault == 'pid': after['pid'] = 2
            elif fault == 'clock': after['frame']['tick'] = 99
            elif fault == 'wipe': after['frame']['checkpoint_generation'] = 1
            elif fault == 'visit': after['visit']['epoch'] = 2
            elif fault == 'owner': after['combat']['stage']['owner'] = 'Red'
            elif fault == 'reset': obj['motion']['time'] = 0
            elif fault == 'skip': obj['motion']['time'] = 3
            elif fault == 'keys': obj['motion']['keys'][0][0] = .5
            elif fault == 'pose': obj['pose']['origin'][2] += 10
            elif fault == 'disabled': obj['solid'] = False
            else: after['combat']['stage']['world']['school']['clock'] = 0
            with self.subTest(fault=fault), self.assertRaises(AssertionError):
                verify_lift_rejoin(before, after, 2)

    def test_personal_resources_and_rider_identity_cannot_reset(self):
        for player in (0,1):
            for fault in ('health','will','inventory','loot','spawn','warp','grounded'):
                before, after = reports()
                body = after['frame']['players'][player]
                if fault == 'health': body['stats']['sanity'] = 25
                elif fault == 'will': body['stats']['will'] = 100
                elif fault == 'inventory': body['stats']['collected'] = []
                elif fault == 'loot': body['stats']['loot'] = {'unexpected': []}
                elif fault == 'spawn': body['spawned_at'] = 100
                elif fault == 'warp': body['warped_at'] = 100
                else: body['player']['grounded'] = False
                with self.subTest(player=player,fault=fault), self.assertRaises(AssertionError):
                    verify_lift_rejoin(before, after, 2)

    def test_stationary_entrance_reconnect_cannot_count_as_a_ride(self):
        before, after = reports()
        lift_object(before)['pose']['origin'][2] = -268
        with self.assertRaisesRegex(AssertionError, 'during the ride'):
            verify_lift_rejoin(before, after, 2)


if __name__ == '__main__':
    unittest.main()
