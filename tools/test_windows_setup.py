"""Exercise the guided installer's worker using local synthetic data and HTTP."""
from contextlib import contextmanager
import hashlib
import http.server
import json
import os
from pathlib import Path
import shutil
import subprocess
import threading
import time
import unittest
import uuid

from test_setup import ROOT, make_data, make_language, setup_environment

TEST_ROOT = ROOT / 'private' / ('guided-setup-tests-' + uuid.uuid4().hex)


@contextmanager
def server(payload, slow=False):
    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def do_GET(self):
            self.send_response(200)
            self.send_header('Content-Length', str(len(payload)))
            self.end_headers()
            try:
                for i in range(0, len(payload), 1024):
                    self.wfile.write(payload[i:i + 1024])
                    self.wfile.flush()
                    if slow:
                        time.sleep(0.02)
            except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
                pass
    srv = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    thread = threading.Thread(target=srv.serve_forever, daemon=True)
    thread.start()
    try:
        yield f'http://127.0.0.1:{srv.server_port}/fixture.7z'
    finally:
        srv.shutdown()
        srv.server_close()


@unittest.skipUnless(os.name == 'nt', 'Windows installer')
class GuidedSetupTests(unittest.TestCase):
    def setUp(self):
        self.root = TEST_ROOT / self._testMethodName / 'Jeu & preview [test]'
        (self.root / 'tools').mkdir(parents=True)
        shutil.copyfile(ROOT / 'Setup.cmd', self.root / 'Setup.cmd')
        self.script = (ROOT / 'tools/windows_setup.ps1').read_text(encoding='utf-8')
        self.data = self.root.parent / 'Données & source'
        make_data(self.data)
        self.job = self.root / 'private/job'
        self.job.mkdir(parents=True)
        self.config = self.root / 'private/data-path.txt'
        self.config.write_text('old-data-path\n')
        self.save = self.root / 'private/saves/keep.txt'
        self.save.parent.mkdir()
        self.save.write_text('keep my progress')

    def start(self, source='', download=False, script=None, language=None, action='setup', confirmed=None):
        # Only the test copy can use loopback HTTP and synthetic download hashes.
        path = self.root / 'tools/windows_setup.ps1'
        path.write_text(script or self.script, encoding='utf-8')
        jobfile = self.job / 'job.json'
        job = dict(directory=str(self.job), source=str(source), download=download,
                   language=language, action=action)
        if confirmed is not None:
            job['downloadConfirmed'] = confirmed
        jobfile.write_text(json.dumps(job))
        return subprocess.Popen(['powershell.exe', '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', str(path),
                                 '-Mode', 'Worker', '-JobFile', str(jobfile)], cwd=self.root,
                                 env=setup_environment(), stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def diagnostics(self, output=''):
        parts = [output]
        for name in ('status.json', 'setup.log'):
            path = self.job / name
            if path.exists():
                parts.append(f'{name}:\n{path.read_text(encoding="utf-8-sig", errors="replace")}')
        return '\n'.join(parts)

    def finish(self, proc, expected):
        stdout, stderr = proc.communicate(timeout=30)
        self.assertEqual(proc.returncode, expected,
                         self.diagnostics((stdout + stderr).decode(errors='replace')))
        self.assertEqual(self.save.read_text(), 'keep my progress')

    def test_folder_worker_handles_literal_paths(self):
        self.finish(self.start(self.data), 0)
        imported = self.root / self.config.read_text(encoding='utf-8').strip()
        self.assertTrue(imported.is_relative_to(self.root / 'private/game-data'))
        self.assertEqual((imported / 'pak0.pk3').read_bytes(), (self.data / 'Alice1/bin/base/pak0.pk3').read_bytes())

    def test_setup_actions_remain_visible_on_short_displays(self):
        # Construct the actual form without showing a window or starting setup.
        # A 620-pixel client area fits a 720p desktop with its taskbar/title bar.
        probe = r'''
    foreach ($height in @(620, 760)) {
        $form.ClientSize = New-Object Drawing.Size(660, $height)
        $form.PerformLayout()
        foreach ($button in @($go, $applyLanguage, $cancel)) {
            $top = $button.Top
            $parent = $button.Parent
            while ($parent -ne $form) { $top += $parent.Top; $parent = $parent.Parent }
            if ($top -lt 0 -or ($top + $button.Height) -gt $form.ClientSize.Height) {
                throw "Action '$($button.Text)' is outside the $height-pixel client area."
            }
        }
    }
'''
        marker = '    [void]$form.ShowDialog()'
        self.assertEqual(self.script.count(marker), 1)
        script = self.script.replace(marker, probe)
        # Make errors observable instead of opening a modal error dialog.
        script = script.replace(
            "[void][Windows.Forms.MessageBox]::Show($_.Exception.Message, 'Looking Glass - Setup')",
            'Write-Error $_.Exception.Message')
        path = self.root / 'tools/windows_setup.ps1'
        path.write_text(script, encoding='utf-8')
        result = subprocess.run(
            ['powershell.exe', '-NoProfile', '-STA', '-ExecutionPolicy', 'Bypass', '-File', str(path)],
            cwd=self.root, env=setup_environment(), capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, (result.stdout + result.stderr).decode(errors='replace'))

    def test_worker_switches_language_using_saved_packs(self):
        german = self.data / 'Alice1/bin/base/pak4_german.pk3'
        make_language(german, 'Schritt')
        self.finish(self.start(self.data, language='de'), 0)
        active = self.root / self.config.read_text().strip()
        pointer = self.config.read_bytes()
        core = {p.name: p.read_bytes() for p in active.glob('*.pk3') if not p.name.startswith('pak4_')}
        self.assertEqual((active / 'pak4_german.pk3').read_bytes(), german.read_bytes())
        self.finish(self.start(language='en', action='language'), 0)
        self.assertFalse((active / 'pak4_german.pk3').exists())
        cache = self.root / 'private/game-data/languages'
        self.assertEqual((cache / 'pak4_german.pk3').read_bytes(), german.read_bytes())
        self.finish(self.start(language='de', action='language'), 0)
        self.assertEqual((active / 'pak4_german.pk3').read_bytes(), german.read_bytes())
        self.assertFalse((active / 'pak4_english.pk3').exists())
        self.assertEqual(self.config.read_bytes(), pointer)
        self.assertEqual({p.name: p.read_bytes() for p in active.glob('*.pk3') if not p.name.startswith('pak4_')}, core)

    def test_invalid_input_keeps_previous_config(self):
        self.finish(self.start(self.root / 'missing.7z'), 1)
        self.assertEqual(self.config.read_text(), 'old-data-path\n')

    def test_cancellation_before_work_keeps_previous_config(self):
        (self.job / 'cancel').write_text('cancel')
        self.finish(self.start(self.data), 1)
        self.assertEqual(self.config.read_text(), 'old-data-path\n')

    def test_download_requires_explicit_confirmation_before_any_work(self):
        # Old jobs, declined consent, and truthy JSON strings must all fail closed.
        for consent in (None, False, 'true', 'false', 1):
            with self.subTest(consent=consent):
                self.finish(self.start(download=True, confirmed=consent), 1)
                state = json.loads((self.job / 'status.json').read_text())
                self.assertIn('Download not confirmed', state['message'])
                self.assertFalse((self.root / 'private/downloads').exists())
                self.assertEqual(self.config.read_text(), 'old-data-path\n')

    def test_empty_local_choice_does_not_fall_back_to_download(self):
        self.finish(self.start(), 1)
        self.assertFalse((self.root / 'private/downloads').exists())
        self.assertEqual(self.config.read_text(), 'old-data-path\n')

    def test_language_download_requires_confirmation_without_changing_installation(self):
        self.finish(self.start(self.data, language='en'), 0)
        pointer = self.config.read_bytes()
        active = self.root / self.config.read_text().strip()
        packs = {p.name: p.read_bytes() for p in active.glob('*.pk3')}
        for consent in (None, False, 'true'):
            with self.subTest(consent=consent):
                self.finish(self.start(download=True, language='de', action='language', confirmed=consent), 1)
                state = json.loads((self.job / 'status.json').read_text())
                self.assertIn('Download not confirmed', state['message'])
                self.assertFalse((self.root / 'private/downloads').exists())
                self.assertEqual(self.config.read_bytes(), pointer)
                self.assertEqual({p.name: p.read_bytes() for p in active.glob('*.pk3')}, packs)

    def test_non_boolean_download_choice_is_rejected(self):
        self.finish(self.start(download='false', confirmed=True), 1)
        self.assertFalse((self.root / 'private/downloads').exists())
        self.assertEqual(self.config.read_text(), 'old-data-path\n')

    def test_setup_defaults_to_local_files_even_when_none_are_found(self):
        self.check_ui_defaults(False)

    def test_setup_detects_local_files_without_selecting_download(self):
        self.check_ui_defaults(True)

    def check_ui_defaults(self, with_data):
        if with_data:
            make_data(self.root)
        # Construct the real WinForms controls without displaying a window. Limit
        # discovery to the fixture's empty user profile, never the user's Downloads.
        checks = r"""
    if (-not $local.Checked -or $download.Checked) { throw 'Download selected by default' }
    if ($go.Enabled -ne ($choices.Items.Count -gt 0)) { throw 'Incorrect initial Set up state' }
    $download.Checked = $true
    if (-not $go.Enabled) { throw 'Explicit download selection disabled' }
    $local.Checked = $true
    if ($go.Enabled -ne ($choices.Items.Count -gt 0)) { throw 'Incorrect local Set up state' }
    Write-Output 'UI defaults passed'
"""
        script = self.script.replace('[void]$form.ShowDialog()', checks)
        # Surface failures instead of showing a modal error in an unattended test.
        script = script.replace("[void][Windows.Forms.MessageBox]::Show($_.Exception.Message, 'Looking Glass - Setup')",
                                'Write-Error $_.Exception.Message')
        path = self.root / 'tools/windows_setup.ps1'
        path.write_text(script, encoding='utf-8')
        env = dict(os.environ, USERPROFILE=str(self.root / 'empty-profile'))
        result = subprocess.run(['powershell.exe', '-NoProfile', '-STA', '-ExecutionPolicy', 'Bypass',
                                 '-File', str(path)], env=env, capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, (result.stdout + result.stderr).decode(errors='replace'))
        self.assertIn(b'UI defaults passed', result.stdout)
        self.assertFalse((self.root / 'private/setup-jobs').exists())
        self.assertFalse((self.root / 'private/downloads').exists())

    def download_script(self, url, payload, wrong_hash=False):
        s = self.script.replace('https://archive.org/download/alice_202106/Alice1_2011_vanilla.7z', url)
        s = s.replace('977811158L', str(len(payload)) + 'L')
        s = s.replace('b253bb2c9c875f1838a711d0fe4f12f4bc2c6207db3baaE6431af962eca8705d',
                      '0' * 64 if wrong_hash else hashlib.sha256(payload).hexdigest())
        return s.replace("$response.ResponseUri.Scheme -ne 'https'", "$response.ResponseUri.Scheme -ne 'http'")

    def test_failed_integrity_check_does_not_publish_download(self):
        payload = b'synthetic bad archive' * 1024
        with server(payload) as url:
            self.finish(self.start(download=True, confirmed=True, script=self.download_script(url, payload, wrong_hash=True)), 1)
        state = json.loads((self.job / 'status.json').read_text())
        self.assertIn('integrity', state['message'], self.diagnostics())
        self.assertEqual(self.config.read_text(), 'old-data-path\n')
        self.assertFalse(list((self.root / 'private/downloads').iterdir()))

    def test_download_cancellation_keeps_previous_install(self):
        payload = b'x' * (1024 * 1024)
        with server(payload, slow=True) as url:
            proc = self.start(download=True, confirmed=True, script=self.download_script(url, payload))
            deadline = time.monotonic() + 10
            while not list((self.root / 'private/downloads').glob('*.partial')):
                if proc.poll() is not None or time.monotonic() > deadline:
                    proc.kill()
                    stdout, stderr = proc.communicate(timeout=30)
                    self.fail(self.diagnostics('Worker did not begin the synthetic download\n'
                                              + (stdout + stderr).decode(errors='replace')))
                time.sleep(0.05)
            (self.job / 'cancel').write_text('cancel')
            self.finish(proc, 1)
        self.assertEqual(self.config.read_text(), 'old-data-path\n')
        self.assertFalse(list((self.root / 'private/downloads').iterdir()))

    def test_download_and_import(self):
        extractor = shutil.which('7z') or str(Path(os.environ.get('ProgramFiles', 'C:/Program Files')) / '7-Zip/7z.exe')
        if not Path(extractor).is_file():
            self.skipTest('7-Zip needed to create the synthetic fixture')
        archive = self.root.parent / 'fixture.7z'
        subprocess.run([extractor, 'a', '-t7z', str(archive), 'Alice1'], cwd=self.data, capture_output=True, check=True)
        payload = archive.read_bytes()
        with server(payload) as url:
            self.finish(self.start(download=True, confirmed=True, script=self.download_script(url, payload)), 0)
        self.assertTrue((self.root / self.config.read_text().strip() / 'pak0.pk3').is_file())
        self.assertEqual((self.root / 'private/downloads/Alice1_2011_vanilla.7z').read_bytes(), payload)

    def test_confirmed_download_imports_only_the_missing_language(self):
        self.finish(self.start(self.data, language='en'), 0)
        pointer = self.config.read_bytes()
        active = self.root / self.config.read_text().strip()
        core = {p.name: p.read_bytes() for p in active.glob('*.pk3') if not p.name.startswith('pak4_')}
        german = self.data / 'Alice1/bin/base/pak4_german.pk3'
        make_language(german, 'Schritt')
        extractor = shutil.which('7z') or str(Path(os.environ.get('ProgramFiles', 'C:/Program Files')) / '7-Zip/7z.exe')
        if not Path(extractor).is_file():
            self.skipTest('7-Zip needed to create the synthetic fixture')
        archive = self.root.parent / 'language-fixture.7z'
        subprocess.run([extractor, 'a', '-t7z', str(archive), 'Alice1'], cwd=self.data, capture_output=True, check=True)
        payload = archive.read_bytes()
        with server(payload) as url:
            self.finish(self.start(download=True, language='de', action='language', confirmed=True,
                                   script=self.download_script(url, payload)), 0)
        self.assertEqual(self.config.read_bytes(), pointer)
        self.assertEqual({p.name: p.read_bytes() for p in active.glob('*.pk3') if not p.name.startswith('pak4_')}, core)
        self.assertEqual((active / 'pak4_german.pk3').read_bytes(), german.read_bytes())
        self.assertFalse((active / 'pak4_english.pk3').exists())


if __name__ == '__main__':
    unittest.main()
