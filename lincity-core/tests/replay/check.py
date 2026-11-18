"""Exercise the versioned replay corpus and allocator-independent scenario replay."""

import argparse
import os
from pathlib import Path
import subprocess
import tempfile

from itertools import zip_longest

from compare import compare, compare_day, read_days

HERE = Path(__file__).resolve().parent


def run(executable, mode, tape, city, days, actions, directory, allocator=None):
    env = os.environ.copy()
    if allocator:
        env["LD_PRELOAD"] = str(allocator)
    return subprocess.run(
        [executable, mode, tape, city, "no-traffic", str(days), actions, directory],
        env=env, capture_output=True, text=True,
    )


def require_success(result):
    if result.returncode:
        raise RuntimeError(result.stderr or f"child exited {result.returncode}")


def small_case(executable, root):
    city = HERE.parent / "fixtures" / "blacksmith-2140.xml"
    tape = HERE / "decisions-v1.txt"
    target = root / "actual"
    require_success(run(executable, "replay", tape, city, 5, HERE / "actions.txt", target))
    actual = target / "state.tsv"
    with (HERE / "state-v1.tsv").open() as oracle, actual.open() as observed:
        for day, (a, b) in enumerate(zip_longest(read_days(oracle), read_days(observed))):
            compare_day(day, a, b)

    # A plausible, single-unit production error must identify the first affected tick.
    bad = root / "modified"
    bad.mkdir()
    for save in target.glob("day-*.gz"):
        (bad / save.name).write_bytes(save.read_bytes())
    lines = actual.read_text().splitlines(keepends=True)
    key = "1\tbuilding(12,12)\tgoods\t"
    index = next(i for i, line in enumerate(lines) if line.startswith(key))
    lines[index] = key + str(int(lines[index][len(key):]) + 1) + "\n"
    (bad / actual.name).write_text("".join(lines))
    try:
        compare(target, bad)
    except ValueError as error:
        if "day 1 building(12,12) goods:" not in str(error):
            raise
    else:
        raise AssertionError("changed goods escaped comparison")

    broken = root / "truncated.tape"
    broken.write_bytes(tape.read_bytes().splitlines(keepends=True)[0])
    result = run(executable, "replay", broken, city, 5, HERE / "actions.txt", root / "invalid")
    if result.returncode != 1 or "simulation trace" not in result.stderr:
        raise AssertionError(f"truncated tape produced unexpected result {result.returncode}: {result.stderr}")


def scenario(executable, root, allocator, days):
    city = HERE.parents[1] / "opening" / "good_times.scn.gz"
    tape = root / "decisions"
    before, after = root / "reference", root / "candidate"
    require_success(run(executable, "record", tape, city, days, "-", before))
    require_success(run(executable, "replay", tape, city, days, "-", after, allocator))
    count = compare(before, after)
    if count != days + 1:
        raise ValueError(f"expected {days + 1} daily observations, got {count}")
    print(f"good_times: {count} daily observations matched")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    parser.add_argument("case", choices=("small", "scenario"))
    parser.add_argument("--allocator", type=Path)
    parser.add_argument("--days", type=int, default=3)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory() as tmp:
        if args.case == "small":
            small_case(args.executable.resolve(), Path(tmp))
        else:
            scenario(args.executable.resolve(), Path(tmp), args.allocator, args.days)
