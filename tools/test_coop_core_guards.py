"""Prevent incomplete or mismatched core receipts from becoming suite passes."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from test_coop_core import CASES, collect_results, school_runner


class CoreReceiptGuards(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.folder = self.root / 'new-evidence'
        self.folder.mkdir()
        self.log = self.root / 'case.log'
        self.log.write_text('Evidence: ' + str(self.folder) + '\n')
        self.boundary = patch('test_coop_regressions.EVIDENCE', self.root)
        self.boundary.start()
        self.addCleanup(self.boundary.stop)
        self.build = 'a' * 64

    def write(self, name, data):
        path = self.folder / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(data))
        return path

    def imported(self):
        for name in CASES['import'].results:
            self.write(name, dict(passed=True, **({} if name == 'host-loss-result.json' else {'internet': True})))
        for role in ('host', 'guest'):
            self.write(role + '/coop-live.json', dict(build=self.build))

    def test_missing_final_backup_or_host_loss_cannot_pass(self):
        self.imported()
        for name in ('backup-result.json', 'host-loss-result.json'):
            with self.subTest(name=name):
                path = self.folder / name
                raw = path.read_bytes()
                path.unlink()
                with self.assertRaises(FileNotFoundError):
                    collect_results(self.log, CASES['import'], True)
                path.write_bytes(raw)

    def test_failed_or_non_boolean_receipts_are_rejected(self):
        self.imported()
        for passed in (False, 'true', 1, None):
            with self.subTest(passed=passed):
                self.write('resume-result.json', dict(passed=passed, internet=True))
                with self.assertRaisesRegex(ValueError, 'not a passing result'):
                    collect_results(self.log, CASES['import'], True)

    def test_each_connected_phase_must_prove_requested_network_scope(self):
        self.imported()
        for scope in (False, None, 'true', 1):
            with self.subTest(scope=scope):
                self.write('resume-after-loss-result.json', dict(passed=True, internet=scope))
                with self.assertRaisesRegex(ValueError, 'network scope'):
                    collect_results(self.log, CASES['import'], True)

    def test_client_and_case_builds_cannot_be_mixed(self):
        self.imported()
        self.write('guest/coop-live.json', dict(build='b' * 64))
        with self.assertRaisesRegex(ValueError, 'inconsistent native build'):
            collect_results(self.log, CASES['import'], True)
        self.write('guest/coop-live.json', dict(build=self.build))
        with self.assertRaisesRegex(ValueError, 'different native builds'):
            collect_results(self.log, CASES['import'], True, 'b' * 64)
        for role in ('host', 'guest'):
            (self.folder / role / 'coop-live.json').unlink()
        with self.assertRaisesRegex(ValueError, 'Missing or inconsistent'):
            collect_results(self.log, CASES['import'], True)

    def test_old_or_outside_evidence_is_not_a_fresh_pass(self):
        self.imported()
        with self.assertRaisesRegex(ValueError, 'reused evidence'):
            collect_results(self.log, CASES['import'], True, preexisting={self.folder})
        self.log.write_text('Evidence: ' + str(self.root.parent) + '\n')
        with self.assertRaisesRegex(ValueError, 'outside private/coop'):
            collect_results(self.log, CASES['import'], True)

    def test_complete_import_and_menu_storage_receipts_keep_their_hashes(self):
        self.imported()
        result = collect_results(self.log, CASES['import'], True)
        self.assertEqual(result['build'], self.build)
        self.assertEqual(set(result['results']), set(CASES['import'].results))
        self.assertTrue(all(len(value) == 64 for value in result['results'].values()))
        self.write('result.json', dict(passed=True, internet=False, evidence=dict(build=self.build)))
        result = collect_results(self.log, CASES['storage'], False, self.build)
        self.assertEqual(result['build'], self.build)

    def test_native_resume_identity_survives_live_report_cleanup(self):
        self.imported()
        for role in ('host', 'guest'):
            (self.folder / role / 'coop-live.json').unlink()
        self.write('host/route-resume.json', dict(verified=True, build=self.build))
        self.assertEqual(collect_results(self.log, CASES['import'], True)['build'], self.build)
        self.write('host/route-resume.json', dict(verified=False, build=self.build))
        with self.assertRaisesRegex(ValueError, 'resume receipt was not verified'):
            collect_results(self.log, CASES['import'], True)

    def test_malformed_metadata_becomes_a_reported_failure(self):
        self.imported()
        for value in ([], None, dict(passed=True, internet=True, evidence=None),
                      dict(passed=True, internet=True, build=['invalid'])):
            with self.subTest(value=value):
                self.write('resume-result.json', value)
                with self.assertRaises(ValueError):
                    collect_results(self.log, CASES['import'], True)

    def test_school_snapshot_preserves_arguments_and_unrelated_worktree_module(self):
        tools = self.root / 'tools'
        tools.mkdir()
        unrelated = tools / 'test_coop_school.py'
        unrelated.write_text('raise RuntimeError("Unrelated worktree module was loaded")\n')
        original = unrelated.read_bytes()
        snapshot = self.root / "saved helper's module.py"
        snapshot.write_text('identity="saved fixture"\n')
        (tools / 'test_coop_school_lift.py').write_text(
            'import json,sys,test_coop_school\n'
            'print(json.dumps([test_coop_school.identity,sys.argv[1:]]))\n')
        with patch('test_coop_core.ROOT', self.root):
            wrapper = school_runner(self.folder, snapshot)
        result = subprocess.run([sys.executable, str(wrapper), '--first', 'red'],
            capture_output=True, text=True, check=True, timeout=10)
        self.assertEqual(json.loads(result.stdout), ['saved fixture', ['--first', 'red']])
        self.assertEqual(unrelated.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
