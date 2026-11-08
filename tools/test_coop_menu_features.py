"""Anode acceptance: difficulty, both Tab/console overlays, and shared chapter travel."""
import argparse
import json
from pathlib import Path
from test_coop_menu import Run, wait, read

class Features(Run):
    def paired(self, epoch, entry, difficulty):
        def ready():
            reports = [read(self.folder / role / 'coop-live.json') for role in ('host', 'guest')]
            if all(r and r['playing'] and r['visit']['epoch'] == epoch for r in reports):
                return reports
        reports = wait(ready, 90, 'shared chapter loaded')
        for report in reports:
            visit = report['visit']
            assert (visit['map'], visit['entry'], visit['difficulty']) == ('skool1', entry, difficulty), visit
            assert all(p['stats']['difficulty'] == difficulty for p in report['frame']['players'])
        return reports

    def run(self):
        self.start('host')
        self.start('guest')
        self.click('host', 'Action(Coop)')
        self.click('host', 'Host')
        self.click('host', 'Start chapter')
        self.screen('host', 'coop-chapters')
        for _ in range(5):
            self.key('host', 0x28)
        self.key('host', 0x44)  # Normal -> Hard in the existing chapter chooser.
        self.key('host', 0x0D)
        self.button('host', 'Copy invite', 150)
        self.click('guest', 'Action(Coop)')
        self.click('guest', 'Join')
        self.copy_join()
        self.paired(1, None, 'Hard')
        wait(lambda: all((read(self.folder / role / 'coop-live.json') or {}).get('presentation', {}).get('aim_glow')
            for role in ('host', 'guest')), label='both aim glow renderers')
        if self.args.capture:
            for role in ('host', 'guest'):
                self.key(role, 0x7B)
                wait(lambda: (self.folder / role / 'coop-screenshot.png').exists(), label=role + ' aim capture')
        print('PASS host chapter and difficulty reach both players', flush=True)
        # The guest can browse without changing the shared world.
        self.key('guest', 0x09)
        self.screen('guest', 'coop-chapters')
        self.key('guest', 0x1B)
        self.screen('guest', 'coop-play')
        for role in ('host', 'guest'):
            self.key(role, 0xC0)
            self.screen(role, 'coop-console')
            self.desktop.type_burst(self.processes[role], 'status')
            self.key(role, 0x0D)
            wait(lambda: 'Console: status' in (self.folder / (role + '.log')).read_text(errors='replace'),
                 label=role + ' console command')
            self.key(role, 0x1B)
            self.screen(role, 'coop-play')
        print('PASS Tab and typed console commands on both clients', flush=True)
        self.key('host', 0x09)
        self.screen('host', 'coop-chapters')
        # First school -> second school -> returning school visit.
        self.key('host', 0x28)
        self.key('host', 0x28)
        self.key('host', 0x44)
        self.key('host', 0x0D)
        self.paired(2, 'skool1_start2', 'Nightmare')
        print('PASS host Tab travel moves both clients to the chosen entrance and difficulty', flush=True)
        (self.folder / 'features-result.json').write_text(json.dumps(dict(passed=True,
            internet=not self.args.local, cases=['host difficulty', 'guest Tab', 'both consoles',
                'shared chapter', 'return entrance', 'shared difficulty']), indent=2))

if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--exe', type=Path, required=True)
    p.add_argument('--data', type=Path, required=True)
    p.add_argument('--seat-session', type=int, required=True)
    p.add_argument('--local', action='store_true')
    p.add_argument('--capture', action='store_true')
    run = Features(p.parse_args())
    try:
        run.run()
    finally:
        run.close()
