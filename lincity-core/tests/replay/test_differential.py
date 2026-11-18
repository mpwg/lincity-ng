"""Ensure differential replay compares the candidate's actual command observations."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

REFERENCE = os.environ.get("LINCITY_REFERENCE_REPLAY")


class DifferentialReplay(unittest.TestCase):
    @unittest.skipUnless(os.name == "posix", "executable Python fixture uses a shebang")
    def test_changed_action_result_names_scenario_profile_day_and_field(self):
        if not REFERENCE:
            self.skipTest("set LINCITY_REFERENCE_REPLAY to the built C++ core_replay")
        with tempfile.TemporaryDirectory() as scratch:
            candidate = Path(scratch) / "candidate"
            candidate.write_text(
                f"#!{sys.executable}\n"
                "from pathlib import Path\n"
                "import subprocess, sys\n"
                f"subprocess.run([{str(REFERENCE)!r}, *sys.argv[1:]], check=True)\n"
                "path = Path(sys.argv[-1]) / 'state.tsv'\n"
                "lines = path.read_text().splitlines(keepends=True)\n"
                "key = '1\\taction[0].query(0,8)\\tmoney_delta\\t'\n"
                "i = next(i for i, line in enumerate(lines) if line.startswith(key))\n"
                "lines[i] = key + '17\\n'\n"
                "path.write_text(''.join(lines))\n"
            )
            candidate.chmod(0o755)
            result = subprocess.run(
                [sys.executable, "-B", Path(__file__).with_name("check.py"), REFERENCE,
                 "differential", "--candidate", candidate, "--quick"],
                capture_output=True, text=True,
            )
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("good_times (no-traffic)", result.stderr)
        self.assertIn("day 1 action[0].query(0,8) money_delta: 0 != 17", result.stderr)


if __name__ == "__main__":
    REFERENCE = Path(sys.argv.pop(1)).resolve()
    unittest.main()
