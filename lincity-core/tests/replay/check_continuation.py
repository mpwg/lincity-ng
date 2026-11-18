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


def action_contracts(cpp, rust, directory):
    # Group numbers and flags are part of the versioned save and replay formats.
    def check(name, globals_xml, map_xml, commands, days=1):
        city = directory / f"{name}.xml"
        city.write_text(f'<lc-game ldsv-version="2140"><globals>{globals_xml}</globals>'
                        f'<map size="32">{map_xml}</map></lc-game>')
        actions = directory / f"{name}.txt"
        actions.write_text("\n".join(commands))
        replay_pair(cpp, rust, city, actions, directory / name, days, "no-traffic")

    check("river", "", '<MapTile group="21" map-x="2" map-y="4"><flags>0x02000000</flags></MapTile>'
          '<MapTile group="21" map-x="0" map-y="4"/><MapTile group="21" map-x="0" map-y="3"/>',
          ["1 flood 1 4", "2 bulldoze 1 4"], 2)
    check("cash", "<total_money>-2147483648</total_money>",
          '<Construction group="6" map-x="2" map-y="2"/><MapTile group="42" map-x="9" map-y="9"/>',
          ["1 query 2 2", "1 evacuate 2 2", "1 bulldoze 9 9", "1 flood 12 12"])
    check("evacuation", "", '<Construction group="25" map-x="2" map-y="2">'
          '<flags>0x300</flags><completion>100</completion></Construction>'
          '<Construction group="7" map-x="9" map-y="9"><flags>0</flags></Construction>',
          ["1 evacuate 3 3", "1 evacuate 9 9"])
    check("prerequisites", "<total_money>10000000</total_money><tech_level>1000000</tech_level>",
          '<MapTile group="0" map-x="16" map-y="16"><ore>999</ore></MapTile>'
          '<MapTile group="0" map-x="19" map-y="19"><ore>1</ore></MapTile>',
          ["1 build 4 4 13", "1 build 4 4 41", "1 build 4 4 15", "1 build 4 4 17",
           "1 build 16 16 15", "1 build 30 30 6"])
    for opening in (-10_000, -1_998_000_000, -2_147_483_648, 2_147_483_647):
        check(f"interest-{opening}", f"<total_money>{opening}</total_money>"
              "<total_time>1198</total_time><people_pool>7</people_pool>", "", [])
    check("construction-cost", "<tech_level>1000000</tech_level><total_money>3100000</total_money>",
          "", ["1 build 8 8 23", "1 build 8 8 11", "1 build 8 8 7"])
    check("wind-low", "<tech_level>30000</tech_level><total_money>1000000</total_money>",
          "", ["1 build 4 4 49", "1 build 10 10 24"])
    check("wind-high", "<tech_level>450000</tech_level><total_money>1000000</total_money>",
          "", ["1 build 4 4 24", "1 build 10 10 49"])
    check("taxes", "<total_time>1198</total_time>",
          '<Construction group="11" map-x="12" map-y="12"><labor>5120</labor>'
          '<coal>512</coal><goods>10240</goods><food>8192</food></Construction>'
          '<Construction group="27" map-x="10" map-y="12"/>'
          '<Construction group="4" map-x="12" map-y="9"/>', [], 2)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cpp", type=Path)
    parser.add_argument("rust", type=Path)
    parser.add_argument("case", choices=("continuation", "traffic", "actions"))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="rustcity-cross-") as scratch:
        case = {"traffic": traffic_modes, "actions": action_contracts, "continuation": continuation}[args.case]
        case(args.cpp.resolve(), args.rust.resolve(), Path(scratch))
