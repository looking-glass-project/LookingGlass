"""Anode: abrupt host loss, unchanged solo return, menu Continue and guest rejoin.

Only the owned host process tree is terminated. Uses native menu input and
passive receipts, independent profiles, and no screenshots or gameplay writes.
Public invites are the default; --local is explicitly loopback-only evidence.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

from test_coop_menu import Run, wait
from test_coop_play import ROOT, read


class HostLossRun(Run):
    def loaded(self, role):
        self.click(role, 'Page(LoadSave)')
        self.click(role, 'Slot(0)')
        self.click(role, 'LoadSelection')
        self.screen(role, 'solo-paused')
        self.key(role, 0x1b)
        self.click(role, 'Action(Coop)')

    def save(self):
        count = len(self.events('host', 'checkpoint-save'))
        self.click('host', 'Save checkpoint')
        receipt = wait(lambda: self.events('host', 'checkpoint-save')[count:],
                       label='new menu save receipt')[0]
        assert receipt['error'] is None, receipt
        return receipt

    def connected(self, expected):
        super().connected(expected)
        reports = {r: read(self.folder / r / 'coop-live.json') for r in ('host', 'guest')}
        for role, report in reports.items():
            assert report['pid'] == self.processes[role].pid, 'Stale process report'
            assert report['build'] == self.build, 'Mixed native builds'
            assert report['visit']['entry'] == expected['visit']['entry'], 'Wrong saved visit'
            assert report['frame']['checkpoint_generation'] == expected['frame']['checkpoint_generation']
        assert reports['host']['completed_visits'] == 1, 'Cached history lost'
        return reports

    def guest_solo_return(self):
        self.click('guest', 'Back')
        self.button('guest', 'Join')
        self.click('guest', 'Back')
        self.screen('guest', 'menu-Main')
        before = self.events('guest', 'solo-before-coop')[-1]
        after = self.events('guest', 'solo-after-coop')[-1]
        assert before['sha256'] == after['sha256'], 'Guest lost its suspended solo game'
        return after['sha256']

    def run(self):
        evidence = {}
        for name in ('test_coop_menu_host_loss.py', 'test_coop_menu.py'):
            source = (ROOT / 'tools' / name).read_bytes()
            (self.folder / name).write_bytes(source)
            evidence[name + '_sha256'] = hashlib.sha256(source).hexdigest()
        with (self.folder / 'prepare.log').open('w') as log:
            subprocess.run([str(self.args.exe.resolve()), '--data', str(self.args.data.resolve()),
                '--no-audio', '--profile', str(self.folder / 'host'), '--coop-import-check'],
                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=120, check=True,
                creationflags=subprocess.CREATE_NO_WINDOW)
        solo = self.folder / 'host/saves/slot1.json'
        guest_solo = self.folder / 'guest/saves/slot1.json'
        guest_solo.write_bytes(solo.read_bytes())
        unchanged = {path: path.read_bytes() for path in (solo, guest_solo)}

        def intact():
            assert all(path.read_bytes() == raw for path, raw in unchanged.items()), 'Saved files changed'
            assert not (self.folder / 'guest/coop-checkpoint.json').exists(), 'Guest wrote a shared save'

        for role in ('host', 'guest'):
            self.start(role)
            self.loaded(role)
        self.build = self.state('host')['build']
        original = {role: p.pid for role, p in self.processes.items()}
        self.click('guest', 'Join')
        self.click('host', 'Host')
        self.click('host', 'Host this game')
        first_link = self.copy_join()
        self.connected(read(self.folder / 'host/coop-import-expected.json'))
        self.key('host', 0x1b)
        self.save()
        self.save()  # A real previous checkpoint must survive along with the primary.
        checkpoint = self.folder / 'host/coop-checkpoint.json'
        backup = checkpoint.with_suffix('.json.bak')
        unchanged.update({path: path.read_bytes() for path in (checkpoint, backup)})
        guest_results = len(self.events('guest', 'coop-result'))
        started = time.monotonic()
        host = self.processes['host']
        subprocess.run(['taskkill', '/PID', str(host.pid), '/T', '/F'],
                       capture_output=True, check=True, timeout=15)
        host.wait(timeout=10)
        receipt = wait(lambda: self.events('guest', 'coop-result')[guest_results:],
                       30, 'guest menu host-loss result')[0]
        assert receipt['error'] and any(term in receipt['error'].lower() for term in
            ('disconnected', 'stopped responding', 'connection', 'session has ended', 'player left')), receipt
        wait(lambda: self.events('guest', 'coop-message')[-1] == 'Could not continue co-op: ' + receipt['error'],
             label='displayed guest disconnect explanation')
        self.button('guest', 'Paste link')
        evidence['disconnect'] = dict(seconds=round(time.monotonic() - started, 3), receipt=receipt)
        evidence['guest_solo_returns'] = [self.guest_solo_return()]
        intact()
        print('PASS abrupt host loss -> visible guest explanation -> exact suspended-solo return', flush=True)

        # The surviving guest can play solo, then rejoin from ESC without restarting.
        self.click('guest', 'Action(Resume)')
        self.screen('guest', 'solo-play')
        self.key('guest', 0x57, hold=.3)
        self.key('guest', 0x1b)
        self.click('guest', 'Action(Coop)')
        before = self.events('guest', 'solo-before-coop')
        assert before[-1]['feet'] != before[-2]['feet'], 'Guest solo movement stopped after host loss'
        self.click('guest', 'Join')
        # Preserve the crashed process's log before starting the fresh host.
        self.logs[0].close()
        (self.folder / 'host-before-loss.log').write_bytes((self.folder / 'host.log').read_bytes())
        self.start('host')
        self.loaded('host')
        self.click('host', 'Host')
        self.click('host', 'Continue co-op')
        second_link = self.copy_join()
        assert second_link != first_link, 'Restart reused the previous invite'
        reports = self.connected(json.loads(unchanged[checkpoint])['payload'])
        (self.folder / 'resumed-views.json').write_text(json.dumps(reports))
        intact()
        assert self.processes['host'].pid != original['host'], 'Host process did not restart'
        assert self.processes['guest'].pid == original['guest'], 'Guest process was restarted'
        self.leave()
        evidence['guest_solo_returns'].append(self.guest_solo_return())
        intact()
        assert len(self.events('guest', 'solo-after-coop')) == 2
        evidence.update(original_pids=original, final_pids={r: p.pid for r, p in self.processes.items()},
            unchanged_files={str(p.relative_to(self.folder)): hashlib.sha256(raw).hexdigest()
                             for p, raw in unchanged.items()},
            invite_hashes=[hashlib.sha256(link.encode()).hexdigest() for link in (first_link, second_link)])
        (self.folder / 'result.json').write_text(json.dumps(dict(passed=True,
            internet=not self.args.local, native_input=True, build=self.build, evidence=evidence,
            scope='Menu recovery after abrupt host loss, fresh-host resume, same-guest rejoin and solo isolation'), indent=2))
        print('PASS fresh host Continue -> same guest rejoins -> saved resources/world -> both solo returns', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--seat-session', type=int, required=True)
    parser.add_argument('--local', action='store_true')
    run = HostLossRun(parser.parse_args())
    try:
        run.run()
    finally:
        run.close()
