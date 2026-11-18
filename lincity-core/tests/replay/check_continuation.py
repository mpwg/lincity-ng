"""Check C++/Rust checkpoint handoff and all three commuter traffic settings."""

import argparse
from pathlib import Path
import tempfile

from check import HERE, require_success, run
from compare import compare


def replay_pair(reference, candidate, city, actions, root, days, profile):
    root.mkdir()
    tape = root / "decisions"
    before, after = root / "recorded", root / "replayed"
    require_success(run(reference, "record", tape, city, days, actions, before, profile=profile))
    require_success(run(candidate, "replay", tape, city, days, actions, after, profile=profile))
    count = compare(before, after)
    if count != days + 1:
        raise ValueError(f"{city}: expected {days + 1} states, got {count}")
    return before / f"day-{days}.gz", after / f"day-{days}.gz", tape


def continuation(cpp, rust, directory):
    for name, city in (
        ("smithy-2140", HERE.parent / "fixtures" / "blacksmith-2140.xml"),
        ("beach-2130", HERE.parents[1] / "opening" / "Beach.scn.gz"),
    ):
        start = directory / name
        cpp_save, rust_save, _ = replay_pair(cpp, rust, city, HERE / "actions.txt", start,
                                              5, "no-traffic")
        for origin, save, recorder, reader in (
            ("cpp", cpp_save, cpp, rust), ("rust", rust_save, rust, cpp),
        ):
            replay_pair(recorder, reader, save, "-", start / origin, 7, "no-traffic")
            print(f"{name} {origin} checkpoint: eight daily states and three saves matched")


def traffic_modes(cpp, rust, directory):
    # Alternating labor stocks on a connected road grid create enough real
    # commuter trips to exercise both spawning and direction selection.
    roads = []
    for y in range(3, 9):
        for x in range(3, 19):
            labor = 5120 if (x + y) % 2 else 0
            roads.append(f'<Construction group="11" map-x="{x}" map-y="{y}">'
                         f'<labor>{labor}</labor></Construction>')
    city = directory / "commuters.xml"
    city.write_text('<lc-game ldsv-version="2140"><globals/><map size="24">'
                    + "".join(roads) + "</map></lc-game>")
    for profile in ("no-traffic", "reduced", "traffic"):
        _, _, tape = replay_pair(cpp, rust, city, "-", directory / profile, 20, profile)
        with tape.open() as stream:
            headings = sum(line.startswith("S vehicle.heading ") for line in stream)
        if bool(headings) != (profile != "no-traffic"):
            raise ValueError(f"{profile}: unexpected number of commuter heading choices: {headings}")
        print(f"{profile}: 21 daily states, three saves and {headings} commuter headings matched")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cpp", type=Path)
    parser.add_argument("rust", type=Path)
    parser.add_argument("case", choices=("continuation", "traffic"))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="rustcity-cross-") as scratch:
        case = traffic_modes if args.case == "traffic" else continuation
        case(args.cpp.resolve(), args.rust.resolve(), Path(scratch))
