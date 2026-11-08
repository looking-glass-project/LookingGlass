"""The regular launcher must not select an obsolete private playtest build."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


@unittest.skipUnless(os.name == 'nt', 'Windows launcher')
class LauncherTests(unittest.TestCase):
    def check_launch(self, packaged):
        with tempfile.TemporaryDirectory(prefix='Looking Glass launcher ') as temp:
            root = Path(temp)
            shutil.copyfile(ROOT / 'Launch.cmd', root / 'Launch.cmd')
            old = root / 'private/playtest'
            old.mkdir(parents=True)
            (old / 'Play-Latest.cmd').write_text('@echo obsolete-playtest\n@exit /b 37\n')
            target = root if packaged else root / 'target/release'
            target.mkdir(parents=True, exist_ok=True)
            # A harmless OS executable stands in for the engine. Its successful
            # output proves which launcher branch ran without starting a game.
            shutil.copyfile(Path(os.environ['SystemRoot']) / 'System32/cmd.exe',
                            target / 'looking-glass.exe')
            result = subprocess.run(['cmd.exe', '/d', '/c', 'Launch.cmd /d /c echo current-build'],
                                    cwd=root, stdin=subprocess.DEVNULL,
                                    capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn('current-build', result.stdout)
            self.assertNotIn('obsolete-playtest', result.stdout)

    def test_source_uses_current_release(self):
        self.check_launch(False)

    def test_package_uses_bundled_engine(self):
        self.check_launch(True)
