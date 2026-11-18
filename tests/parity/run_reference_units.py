"""Build and exercise the actual pinned C++ logic with Git and a C++20 compiler."""
import os
from pathlib import Path
import subprocess
import tempfile

REFERENCE = "6f6ffe2c8c5558b3d68ed6b4b8f68721817c3792"
HERE = Path(__file__).resolve().parent
SOURCES = ("commodities.cpp", "commodities.hpp", "stats.cpp", "stats.hpp", "groups.hpp")


def main():
    workspace = HERE.parent.parent / "build"
    workspace.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="reference-units-", dir=workspace) as tmp:
        build = Path(tmp)
        for name in SOURCES:
            source = subprocess.check_output(
                ["git", "show", f"{REFERENCE}:src/lincity/{name}"], cwd=HERE
            )
            (build / name).write_bytes(source)
        binary = build / "reference-units"
        subprocess.run(
            [os.environ.get("CXX", "c++"), "-std=c++20", "-I", tmp,
             HERE / "reference_units.cpp", build / "commodities.cpp",
             build / "stats.cpp", "-o", binary], env=dict(os.environ, TMPDIR=tmp), check=True
        )
        subprocess.run([binary], check=True)


if __name__ == "__main__":
    main()
