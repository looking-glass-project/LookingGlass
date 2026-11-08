"""Anode: real-menu save failure/retry, backup continuation and corrupt-save feedback.

Uses native input and passive receipts, separate processes and isolated profiles.
Only this run's checkpoint files are locked or damaged. Public invites are the
default; --local isolates storage/UI behavior and never counts as internet proof.
"""
import argparse
from contextlib import contextmanager
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
from pathlib import Path
import subprocess

from test_coop_menu import Run, wait
from test_coop_play import ROOT, read


@contextmanager
def deny_replacement(path, profile):
    """Permit reading an existing owned file but deny write/delete sharing."""
    assert path.resolve().is_relative_to(profile.resolve()) and path.is_file()
    kernel = c.WinDLL('kernel32', use_last_error=True)
    kernel.CreateFileW.argtypes = [w.LPCWSTR, w.DWORD, w.DWORD, c.c_void_p,
                                  w.DWORD, w.DWORD, w.HANDLE]
    kernel.CreateFileW.restype = w.HANDLE
    kernel.CloseHandle.argtypes, kernel.CloseHandle.restype = [w.HANDLE], w.BOOL
    handle = kernel.CreateFileW(str(path), 0x80000000, 1, None, 3, 0, None)
    if handle == c.c_void_p(-1).value:
        raise c.WinError(c.get_last_error())
    try:
        yield
    finally:
        assert kernel.CloseHandle(handle), 'Could not release test file lock'


class StorageRun(Run):
    def checkpoint_save(self, failed=False):
        count = len(self.events('host', 'checkpoint-save'))
        self.click('host', 'Save checkpoint')
        receipt = wait(lambda: self.events('host', 'checkpoint-save')[count:],
                       label='new menu save result')[0]
        assert (receipt['error'] is not None) == failed, receipt
        message = ('Checkpoint was not saved: ' + receipt['error']) if failed else 'Co-op checkpoint saved for both players.'
        wait(lambda: self.events('host', 'coop-panel-message')[-1] == message,
             label='displayed checkpoint feedback')
        return receipt

    def open_loaded_solo(self):
        self.click('host', 'Page(LoadSave)')
        self.click('host', 'Slot(0)')
        self.click('host', 'LoadSelection')
        self.screen('host', 'solo-paused')
        self.key('host', 0x1b)
        self.click('host', 'Action(Coop)')
        self.click('host', 'Host')

    def pair(self):
        self.start('host')
        self.start('guest')
        assert self.state('host')['build'] == self.state('guest')['build'], 'Mixed native builds'
        self.evidence['build'] = self.state('host')['build']
        self.evidence.setdefault('processes', []).append({r: p.pid for r, p in self.processes.items()})
        self.open_loaded_solo()
        self.click('guest', 'Action(Coop)')
        self.click('guest', 'Join')

    def retain_and_close(self, phase):
        self.close()
        for role in ('host', 'guest'):
            path = self.folder / (role + '.log')
            if path.exists():
                (self.folder / (phase + '-' + role + '.log')).write_bytes(path.read_bytes())
        self.logs = []

    def snapshots(self, phase):
        reports = {r: read(self.folder / r / 'coop-live.json') for r in ('host', 'guest')}
        assert all(report and report['playing'] for report in reports.values()), 'Session stopped during save failure'
        (self.folder / (phase + '.json')).write_text(json.dumps(reports))
        return reports

    def failed_writes(self, checkpoint, backup):
        baseline = checkpoint.read_bytes()
        self.evidence['failed_writes'] = []
        for path in (backup, checkpoint):
            old_backup = backup.read_bytes()
            before = self.snapshots(path.suffix + '-before')
            with deny_replacement(path, self.folder / 'host'):
                receipt = self.checkpoint_save(failed=True)
                assert checkpoint.read_bytes() == baseline, 'Failed save replaced the committed checkpoint'
                # Backup replacement precedes primary replacement. If the latter
                # fails, the backup may now contain the still-valid old primary.
                expected_backup = old_backup if path == backup else baseline
                assert backup.read_bytes() == expected_backup, 'Failed save lost the last good backup'
                assert not list((self.folder / 'host').glob('coop-checkpoint.*.tmp')), 'Failed save leaked a temporary file'
                wait(lambda: all((read(self.folder / r / 'coop-live.json') or {}).get('frame', {}).get('tick', 0)
                                 > before[r]['frame']['tick'] for r in ('host', 'guest')),
                     label='both clients advancing after failed save')
                after = self.snapshots(path.suffix + '-after')
                self.evidence['failed_writes'].append(dict(file=path.name, receipt=receipt,
                    ticks={r: [before[r]['frame']['tick'], after[r]['frame']['tick']] for r in before},
                    backup_matches_previous_primary=backup.read_bytes() == baseline))
            print('PASS menu save failure, visible feedback and live session:', path.name, flush=True)
        self.checkpoint_save()
        assert checkpoint.read_bytes() != baseline, 'Retry did not save the advanced game'
        assert backup.read_bytes() == baseline
        self.evidence['save_retry'] = True
        print('PASS released file locks -> successful menu save without restarting either game', flush=True)

    def run(self):
        for name in ('test_coop_menu_storage.py', 'test_coop_menu.py'):
            source = (ROOT / 'tools' / name).read_bytes()
            (self.folder / name).write_bytes(source)
            self.evidence[name + '_sha256'] = hashlib.sha256(source).hexdigest()
        with (self.folder / 'prepare.log').open('w') as log:
            subprocess.run([str(self.args.exe.resolve()), '--data', str(self.args.data.resolve()),
                '--no-audio', '--profile', str(self.folder / 'host'), '--coop-import-check'],
                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=120, check=True,
                creationflags=subprocess.CREATE_NO_WINDOW)
        solo = self.folder / 'host/saves/slot1.json'
        solo_bytes = solo.read_bytes()
        checkpoint = self.folder / 'host/coop-checkpoint.json'
        backup = checkpoint.with_suffix('.json.bak')
        self.pair()
        self.click('host', 'Host this game')
        self.first_link = self.copy_join()
        self.connected(read(self.folder / 'host/coop-import-expected.json'))
        self.key('host', 0x1b)
        self.checkpoint_save()
        assert checkpoint.is_file() and backup.is_file()
        self.failed_writes(checkpoint, backup)
        saved, previous = checkpoint.read_bytes(), backup.read_bytes()
        assert saved != previous, 'Recovery fixture needs distinct primary and backup states'
        self.leave()
        assert checkpoint.read_bytes() == saved and backup.read_bytes() == previous
        assert solo.read_bytes() == solo_bytes
        self.evidence['first_solo_return'] = self.events('host', 'solo-after-coop')[-1]
        self.retain_and_close('initial')

        # Mutate only our isolated disk fixture, after both game processes exit.
        checkpoint.write_bytes(b'{damaged primary')
        self.pair()
        self.click('host', 'Continue co-op')
        self.button('host', 'Copy invite', 150)
        wait(lambda: 'Recovered the previous co-op checkpoint from its backup.'
             in self.events('host', 'coop-panel-message'), label='visible backup recovery notice')
        assert self.copy_join() != self.first_link, 'Restart reused an expired invite'
        self.connected(json.loads(previous)['payload'])
        self.snapshots('backup-connected')
        self.key('host', 0x1b)
        self.checkpoint_save()
        assert backup.read_bytes() == previous, 'Repairing primary replaced the good backup with damaged bytes'
        repaired = checkpoint.read_bytes()
        assert json.loads(repaired)['format'] == 'looking-glass-coop'
        self.leave()
        assert checkpoint.read_bytes() == repaired and backup.read_bytes() == previous
        assert solo.read_bytes() == solo_bytes
        self.evidence['second_solo_return'] = self.events('host', 'solo-after-coop')[-1]
        self.evidence.update(backup_recovery=True, repaired_primary=True)
        print('PASS fresh processes -> Continue co-op backup notice -> join -> save repaired primary -> exact solo return', flush=True)
        self.retain_and_close('backup')

        checkpoint.write_bytes(b'{damaged primary')
        backup.write_bytes(b'{damaged backup')
        self.start('host')
        self.open_loaded_solo()
        count = len(self.events('host', 'coop-result'))
        self.click('host', 'Continue co-op')
        receipt = wait(lambda: self.events('host', 'coop-result')[count:], label='corrupt-save menu result')[0]
        assert 'Could not read the co-op checkpoint' in (receipt['error'] or '') and 'backup:' in receipt['error'], receipt
        wait(lambda: self.events('host', 'coop-message')[-1] == 'Could not continue co-op: ' + receipt['error'],
             label='visible corrupt-save feedback')
        self.button('host', 'Host this game')
        assert checkpoint.read_bytes() == b'{damaged primary' and backup.read_bytes() == b'{damaged backup'
        self.click('host', 'Back')
        self.click('host', 'Back')
        self.screen('host', 'menu-Main')
        before, after = self.events('host', 'solo-before-coop')[-1], self.events('host', 'solo-after-coop')[-1]
        assert before['sha256'] == after['sha256'], 'Failed Continue changed the suspended solo game'
        assert solo.read_bytes() == solo_bytes
        self.evidence.update(both_corrupt_receipt=receipt, third_solo_return=after,
                             source_sha256=hashlib.sha256(solo_bytes).hexdigest())
        self.click('host', 'Action(Resume)')
        self.screen('host', 'solo-play')
        self.key('host', 0x57, hold=.3)
        self.key('host', 0x1b)
        self.click('host', 'Action(Coop)')
        assert self.events('host', 'solo-before-coop')[-1]['feet'] != before['feet'], 'Solo controls did not recover after failed Continue'
        self.click('host', 'Back')
        self.screen('host', 'menu-Main')
        self.evidence['solo_controls_after_failure'] = True
        print('PASS both saves damaged -> menu error -> unchanged suspended solo -> ordinary solo movement', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--seat-session', type=int, required=True)
    parser.add_argument('--local', action='store_true')
    run = StorageRun(parser.parse_args())
    run.evidence = {}
    result = dict(passed=False, internet=not run.args.local, native_input=True, evidence=run.evidence)
    try:
        run.run()
        result['passed'] = True
    except Exception as error:
        result['error'] = str(error)
        raise
    finally:
        run.close()
        (run.folder / 'result.json').write_text(json.dumps(result, indent=2))
