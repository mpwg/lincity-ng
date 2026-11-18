"""Check XML warning text and the build's assertion policy in a subprocess."""

import argparse
import resource
import signal
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable")
    args = parser.parse_args()
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    for kind, diagnostic in (
        ("element", "warning: skipping unexpected element <tile>"),
        ("attribute", 'warning: skipping unexpected attribute obsolete="west" in <tile>'),
    ):
        result = subprocess.run([args.executable, kind], capture_output=True, text=True)
        status = {"assertions": -signal.SIGABRT, "warnings": 0}.get(result.stdout.strip())
        if status is None or result.returncode != status or diagnostic not in result.stderr:
            parser.exit(1, f"{kind}: exit {result.returncode}\n{result.stdout}{result.stderr}")


if __name__ == "__main__":
    main()
