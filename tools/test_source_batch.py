"""The source audit preserves Git's exact bytes without per-file processes."""
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import check_source


class IndexBlobTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = tempfile.TemporaryDirectory(prefix="looking-glass-source-")
        cls.root = Path(cls.directory.name)
        subprocess.run(["git", "init", "--quiet"], cwd=cls.root, check=True)
        cls.payloads = [b"", b"fn main() {}\n", b"\x00binary\npayload\n"]
        cls.blobs = [subprocess.check_output(["git", "hash-object", "-w", "--stdin"],
                     cwd=cls.root, input=data).decode().strip() for data in cls.payloads]

    @classmethod
    def tearDownClass(cls):
        cls.directory.cleanup()

    def test_exact_empty_text_and_binary_bytes_with_constant_process_count(self):
        entries = [(f"src/check-{n}.rs", self.blobs[n % 3]) for n in range(60)]
        with patch.object(subprocess, "Popen", wraps=subprocess.Popen) as processes:
            found = list(check_source.index_blobs(entries, self.root))
        self.assertEqual(processes.call_count, 2)
        self.assertEqual(found, [(path, len(self.payloads[n % 3]), self.payloads[n % 3])
                                 for n, (path, _) in enumerate(entries)])

    def test_oversized_blob_is_skipped_without_losing_later_records(self):
        order = [2, 0, 1]
        entries = [(f"src/check-{n}.rs", self.blobs[n]) for n in order]
        with patch.object(check_source, "size_limit", return_value=14):
            found = list(check_source.index_blobs(entries, self.root))
        self.assertEqual(found, [(path, len(self.payloads[n]),
                                 self.payloads[n] if len(self.payloads[n]) <= 14 else None)
                                for (path, _), n in zip(entries, order)])


if __name__ == "__main__":
    unittest.main()
