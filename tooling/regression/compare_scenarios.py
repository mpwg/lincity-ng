"""Compare current and historical simulation checkpoints on this toolchain."""

import gzip
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[2]
REFERENCE = "21796de530d68b30881c6b0f6a6a9d6bbfb1e31a"


def run(*args, **kwargs):
    return subprocess.run(args, cwd=ROOT, check=True, **kwargs)


def prepare_reference(directory):
    source = directory / "reference"
    source.mkdir()
    archive = directory / "reference.tar"
    with archive.open("wb") as dst:
        run("git", "archive", REFERENCE, stdout=dst)
    with tarfile.open(archive) as tar:
        tar.extractall(source, filter="data")
    archive.unlink()

    desktop = source / "src" / "lincity-ng"
    shutil.copyfile(Path(__file__).with_name("scenario_trace.cpp"), desktop / "main.cpp")
    shutil.copyfile(ROOT / "lincity-core/lincity/frame_counts.inc", desktop / "frame_counts.inc")
    # The pinned source already links desktop code to the simulation. Retain
    # the application helpers while providing a separate entry point for tracing.
    original = subprocess.check_output(
        ["git", "show", f"{REFERENCE}:src/lincity-ng/main.cpp"], cwd=ROOT
    )
    (desktop / "original_main.cpp").write_bytes(
        b"#define main lincity_reference_main\n" + original
    )
    with (desktop / "CMakeLists.txt").open("a") as cmake:
        cmake.write("""
target_sources(lincity-ng PRIVATE original_main.cpp)
target_compile_definitions(lincity-ng PRIVATE LINCITY_REFERENCE_TRACE)
""")
    with (source / "src/lincity/CMakeLists.txt").open("a") as cmake:
        cmake.write("""
target_link_libraries(lincity_lib PUBLIC
  $<IF:$<TARGET_EXISTS:SDL3_mixer::SDL3_mixer>,SDL3_mixer::SDL3_mixer,SDL3_mixer::SDL3_mixer-static>)
""")
    # Explicitly seed the original generator without changing its algorithm.
    header = source / "src/util/randutil.hpp"
    header.write_text(header.read_text().replace(
        "  result_type operator()();",
        "  result_type operator()();\n  void seed(result_type value) { base_urbg.seed(value); }",
    ))
    build = directory / "reference-build"
    run("cmake", "-S", source, "-B", build, "-DCMAKE_BUILD_TYPE=Release")
    run("cmake", "--build", build, "--parallel", "6", "--target", "lincity-ng")
    return build / "bin" / ("lincity-ng.exe" if os.name == "nt" else "lincity-ng")


def compare(reference, current, directory):
    checkpoints = ("day-0.gz", "day-1.gz", "day-100.gz", "day-1201.gz", "messages.txt")
    for profile in ("no-traffic", "traffic"):
        for scenario in sorted((ROOT / "lincity-core/opening").glob("*.scn.gz")):
            before = directory / f"{scenario.name}.{profile}.reference"
            after = directory / f"{scenario.name}.{profile}.current"
            before.mkdir()
            after.mkdir()
            run(reference, scenario, before, profile)
            run(current, scenario, after, profile)
            for checkpoint in checkpoints:
                read = gzip.open if checkpoint.endswith(".gz") else open
                with read(before / checkpoint, "rb") as a, read(after / checkpoint, "rb") as b:
                    if a.read() != b.read():
                        raise SystemExit(f"{scenario.name} ({profile}): {checkpoint} differs")
            print(f"PASS {scenario.name} ({profile}, four saves and message sequence)", flush=True)


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: python tooling/regression/compare_scenarios.py BUILD_DIR")
    core_build = Path(sys.argv[1]).resolve()
    storage = ROOT / "build"
    storage.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="scenario-parity-", dir=storage) as scratch:
        os.environ["TMPDIR"] = scratch
        directory = Path(scratch)
        run("cmake", "--build", core_build, "--target", "core_scenario_trace")
        reference = prepare_reference(directory)
        current = core_build / "tooling/regression" / (
            "core_scenario_trace.exe" if os.name == "nt" else "core_scenario_trace"
        )
        compare(reference, current, directory)


if __name__ == "__main__":
    main()
