"""Exercise the versioned replay corpus and allocator-independent scenario replay."""

import argparse
import os
from pathlib import Path
import subprocess
import tempfile

from itertools import zip_longest
from xml.etree.ElementTree import ParseError

from compare import compare, compare_day, read_days

HERE = Path(__file__).resolve().parent


def run(executable, mode, tape, city, days, actions, directory, allocator=None, profile="no-traffic"):
    env = os.environ.copy()
    if allocator:
        env["LD_PRELOAD"] = str(allocator)
    return subprocess.run(
        [executable, mode, tape, city, profile, str(days), actions, directory],
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

    records = tape.read_text().splitlines()
    first_pair = next(i for i, r in enumerate(records) if r.startswith("A ") and r.split()[2] == "2")
    duplicate = records.copy()
    parts = duplicate[first_pair].split()
    parts[-1] = parts[-2]
    duplicate[first_pair] = " ".join(parts)
    for name, damaged, diagnostic in (
        ("truncated", records[:1], "unexpected event"),
        ("duplicate", duplicate, "duplicate or unknown construction visit"),
        ("trailing", [*records, "Z"], "trailing data"),
    ):
        broken = root / f"{name}.tape"
        broken.write_text("\n".join(damaged) + "\n")
        result = run(executable, "replay", broken, city, 5, HERE / "actions.txt", root / name)
        if result.returncode != 1 or diagnostic not in result.stderr:
            raise AssertionError(f"{name} tape produced unexpected result {result.returncode}: {result.stderr}")


def compare_run(reference, candidate, root, city, days, actions, profile, allocator=None):
    tape = root / "decisions"
    before, after = root / "reference", root / "candidate"
    require_success(run(reference, "record", tape, city, days, actions, before, profile=profile))
    require_success(run(candidate, "replay", tape, city, days, actions, after, allocator, profile))
    count = compare(before, after)
    if count != days + 1:
        raise ValueError(f"expected {days + 1} daily observations, got {count}")
    return count, len(list(before.glob("day-*.gz")))


def differential(reference, candidate, actions, quick):
    scenarios = ("good_times",) if quick else (
        "Beach", "Rocket_98", "bad_times", "extreme_arid", "extreme_wetland", "good_times",
    )
    days = 5 if quick else 1201
    total_days = total_saves = 0
    for name in scenarios:
        city = HERE.parents[1] / "opening" / (name + ".scn.gz")
        for profile in ("no-traffic", "traffic"):
            try:
                with tempfile.TemporaryDirectory(prefix="lincity-differential-") as tmp:
                    count, saves = compare_run(reference, candidate, Path(tmp), city, days,
                                               actions, profile)
            except (OSError, EOFError, ParseError, RuntimeError, ValueError) as error:
                raise ValueError(f"{name} ({profile}): {error}") from error
            print(f"{name} ({profile}): {count} daily observations, {saves} saves matched",
                  flush=True)
            total_days += count
            total_saves += saves
    print(f"Total: {total_days} daily observations, {total_saves} saves matched")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path, help="reference core_replay executable")
    parser.add_argument("case", choices=("small", "scenario", "differential"))
    parser.add_argument("--allocator", type=Path)
    parser.add_argument("--profile", choices=("traffic", "no-traffic"))
    parser.add_argument("--days", type=int)
    parser.add_argument("--candidate", type=Path, help="candidate for differential replay")
    parser.add_argument("--quick", action="store_true", help="compare good_times for five days")
    parser.add_argument("--actions", type=Path, help="ordered actions for differential replay")
    args = parser.parse_args()
    reference = args.executable.resolve()
    if args.case == "differential":
        if not args.candidate:
            parser.error("differential requires --candidate")
        if args.allocator or args.profile or args.days is not None:
            parser.error("--allocator, --profile and --days belong to scenario replay")
        try:
            differential(reference, args.candidate.resolve(), args.actions or HERE / "actions.txt",
                         args.quick)
        except ValueError as error:
            parser.exit(1, f"{error}\n")
    else:
        if args.candidate or args.quick or args.actions:
            parser.error("--candidate, --quick and --actions belong to differential replay")
        with tempfile.TemporaryDirectory() as tmp:
            if args.case == "small":
                small_case(reference, Path(tmp))
            else:
                city = HERE.parents[1] / "opening" / "good_times.scn.gz"
                count, _ = compare_run(reference, reference, Path(tmp), city,
                                       args.days if args.days is not None else 3, "-",
                                       args.profile or "no-traffic", args.allocator)
                print(f"good_times: {count} daily observations matched")
