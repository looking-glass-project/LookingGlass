"""Interrupt a real visit-loading barrier without writing game or world state."""
import ctypes
from ctypes import wintypes
import json
import time

from test_coop_play import read, distance
from test_coop_village import verify_rejoin_stats


def current_report(path):
    deadline = time.monotonic() + 5
    while True:
        report = read(path)
        if report is not None:
            return report
        assert time.monotonic() < deadline, 'Live report remained unreadable'
        time.sleep(.025)


def verify_barrier(before, held, epoch):
    for report in (before, held):
        assert report['role'] == 'host' and report['session_phase'] == 'Loading'
        assert not report['playing'] and report['can_load'] is False
        assert report['visit']['epoch'] == report['frame']['epoch'] == epoch - 1
        assert report['pending_visit']['epoch'] == epoch, 'Not waiting for the requested visit'
    for field in ('pid', 'build', 'visit', 'pending_visit', 'frame', 'combat', 'completed_visits'):
        assert before[field] == held[field], f'Loading barrier advanced {field}'


def verify_waiting_arrival(before, arrived):
    assert arrived['role'] == 'host' and arrived['pid'] == before['pid']
    assert arrived['build'] == before['build']
    assert arrived['session_phase'] == 'Waiting' and not arrived['playing']
    assert arrived['pending_visit'] is None and arrived['visit'] == before['pending_visit']
    old, new = before['frame'], arrived['frame']
    assert new['epoch'] == arrived['visit']['epoch'] and new['tick'] == old['tick'], 'Loading advanced simulation time'
    assert new['checkpoint_generation'] == old['checkpoint_generation'] == 0, 'Loading restored a wipe point'
    assert new['lives'] == old['lives'], 'Loading changed life/countdown state'
    assert arrived['completed_visits'] == before['completed_visits'] + 1, 'Departure lost or duplicated visit history'
    assert arrived['visit']['map'] == 'skool1', 'This oracle covers the staged school visits'
    baseline = [1,1,1,0,0,0,1,0,0,0] if arrived['visit']['entry'] == 'skool1_start2' else [1,1,0,0,0,0,0,0,0,0]
    for previous, current in zip(old['players'], new['players']):
        assert len(previous['stats']['owned']) == len(baseline)
        # Normal destination loadouts fill earned campaign milestones. Require
        # exactly those grants, with all prior copies/resources still intact.
        expected = dict(previous['stats'], owned=[max(a,b) for a,b in zip(previous['stats']['owned'], baseline)])
        assert current['stats'] == expected, 'Loading changed personal resources beyond destination grants'
        assert current['spawned_at'] == previous['spawned_at'] == 0
        assert current['warped_at'] == old['tick'], 'Players did not arrive together'
    assert distance(*(b['player']['feet'] for b in new['players'])) >= 35, 'Arrival overlapped the players'


def verify_guest_arrival(waiting, guest, old_pid):
    assert guest['role'] == 'guest' and guest['pid'] != old_pid, 'Guest did not restart'
    assert guest['playing'] and guest['session_phase'] == 'Playing'
    assert guest['build'] == waiting['build'] and guest['visit'] == waiting['visit']
    old, new = waiting['frame'], guest['frame']
    assert new['epoch'] == old['epoch'] and new['tick'] >= old['tick']
    assert new['checkpoint_generation'] == old['checkpoint_generation'] == 0
    assert new['lives'] == old['lives']
    for previous, current in zip(old['players'], new['players']):
        verify_rejoin_stats(previous['stats'], current['stats'], new['tick'] - old['tick'])
        loot = dict(previous['stats'].get('loot', {}))
        key = 'skool1$return' if guest['visit']['entry'] == 'skool1_start2' else 'skool1$first'
        # The first simulation step may register an empty drop journal for the
        # new visit. Existing journals and actual drops cannot reset or change.
        expected_loot = dict(loot)
        expected_loot.setdefault(key, dict(awarded=[], drops=[]))
        assert current['stats'].get('loot', {}) in (loot, expected_loot), 'Rejoin changed personal drops'
        for field in ('spawned_at', 'warped_at'):
            assert current[field] == previous[field], f'Rejoin replaced body {field}'
    return dict(epoch=new['epoch'], entry=guest['visit']['entry'],
                held_tick=old['tick'], rejoined_tick=new['tick'],
                old_pid=old_pid, new_pid=guest['pid'])


class LoadingReconnect:
    def __init__(self, folder, epoch):
        self.folder, self.epoch = folder, epoch
        self.suspended = None
        self.started = None
        self.evidence = None

    def observe(self, report, active, start):
        if self.evidence is not None:
            return False
        tick = report['frame']['tick']
        if self.suspended is None:
            if report['visit']['epoch'] != self.epoch - 1 or tick < (1200 if self.epoch == 2 else 1740):
                return False
            assert report['playing'] and report['pending_visit'] is None, 'Missed pre-transition suspension window'
            process = active['guest']
            assert process.poll() is None
            ntdll = ctypes.WinDLL('ntdll')
            self.resume = ntdll.NtResumeProcess
            suspend = ntdll.NtSuspendProcess
            for call in (suspend, self.resume):
                call.argtypes, call.restype = [wintypes.HANDLE], ctypes.c_long
            self.handle = wintypes.HANDLE(int(process._handle))
            assert suspend(self.handle) >= 0, 'Could not suspend owned guest'
            self.suspended, self.started = process, time.monotonic()
            print('Guest suspended before visit', self.epoch, 'at tick', tick, flush=True)
            return False
        assert time.monotonic() - self.started < 12, 'Host did not reach the loading barrier'
        if report['session_phase'] != 'Loading' or report['playing']:
            return False
        path = self.folder / 'host' / 'coop-live.json'
        time.sleep(.5)
        held = current_report(path)
        verify_barrier(report, held, self.epoch)
        self.evidence = dict(passed=False, barrier=report, held=held)
        self.save()
        old_pid = self.suspended.pid
        # The Popen handle owns this exact child; no process-name/PID lookup.
        self.suspended.kill()
        self.suspended.wait(timeout=10)
        self.suspended = None
        del active['guest']
        deadline = time.monotonic() + 30
        while True:
            assert active['host'].poll() is None, 'Host exited during loading'
            waiting = read(path)
            if waiting and waiting['pid'] == active['host'].pid and waiting['session_phase'] == 'Waiting' and waiting['visit']['epoch'] == self.epoch:
                break
            assert time.monotonic() < deadline, 'Host did not finish pending load after guest loss'
            time.sleep(.05)
        self.evidence['waiting'] = waiting
        self.save()
        verify_waiting_arrival(held, waiting)
        time.sleep(.5)
        still = current_report(path)
        for field in ('frame', 'combat', 'visit', 'completed_visits', 'session_phase'):
            assert still[field] == waiting[field], f'Waiting host advanced {field}'
        replacement = start('guest', replacement=True)
        deadline = time.monotonic() + 60
        while True:
            assert replacement.poll() is None, 'Replacement guest exited'
            after = read(self.folder / 'guest' / 'coop-live.json')
            if after and after['pid'] == replacement.pid and after['playing']:
                break
            assert time.monotonic() < deadline, 'Replacement guest did not resume pending visit'
            time.sleep(.05)
        self.evidence['after'] = after
        self.save()
        self.evidence['summary'] = verify_guest_arrival(waiting, after, old_pid)
        self.evidence['passed'] = True
        self.save()
        print('PASS loading-barrier guest restart for visit', self.epoch, flush=True)
        return True

    def save(self):
        (self.folder / 'loading-rejoin.json').write_text(json.dumps(self.evidence, indent=2))

    def close(self):
        if self.suspended is not None and self.suspended.poll() is None:
            assert self.resume(self.handle) >= 0, 'Could not resume owned guest during cleanup'
        self.suspended = None
