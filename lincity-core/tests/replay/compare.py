"""Compare replay outputs in tick order, including the first differing field."""

import argparse
import gzip
from itertools import groupby, zip_longest
from pathlib import Path
import xml.etree.ElementTree as ET


def read_days(stream):
    def rows():
        for number, line in enumerate(stream, 1):
            try:
                day, entity, field, value = line.rstrip("\n").split("\t")
                yield int(day), (entity, field), value
            except ValueError as error:
                raise ValueError(f"invalid observation at {stream.name}:{number}") from error

    for expected, (day, group) in enumerate(groupby(rows(), lambda row: row[0])):
        if day != expected:
            raise ValueError(f"{stream.name}: expected day {expected}, got {day}")
        fields = {}
        for _, key, value in group:
            if key in fields:
                raise ValueError(f"day {day} {key}: duplicate observation")
            fields[key] = value
        yield fields


def checkpoint(path):
    with gzip.open(path) as stream:
        root = ET.parse(stream).getroot()
    # The save writer enumerates constructions by pointer. Match map entries
    # by their persisted identity while preserving ordered histories and queues.
    world_map = root.find("map")
    if world_map is None:
        raise ValueError(f"{path}: missing map")
    world_map[:] = sorted(world_map, key=lambda e: (e.tag, e.get("map-y", ""), e.get("map-x", "")))
    return root


def xml_diff(left, right, path):
    if left.tag != right.tag:
        return f"{path} tag: {left.tag} != {right.tag}"
    if left.attrib != right.attrib:
        return f"{path} attributes: {left.attrib} != {right.attrib}"
    if (left.text or "").strip() != (right.text or "").strip():
        return f"{path}: {left.text!r} != {right.text!r}"
    if len(left) != len(right):
        return f"{path} children: {len(left)} != {len(right)}"
    for index, (a, b) in enumerate(zip(left, right)):
        position = f"({a.get('map-x')},{a.get('map-y')})" if "map-x" in a.attrib else f"[{index}]"
        delta = xml_diff(a, b, f"{path}/{a.tag}{position}")
        if delta:
            return delta
    return None


def compare_day(day, a, b):
    if a is None or b is None:
        raise ValueError(f"day {day}: missing observations")
    if a != b:
        for entity, field in dict.fromkeys((*a, *b)):
            key = entity, field
            if a.get(key) != b.get(key):
                raise ValueError(f"day {day} {entity} {field}: "
                                 f"{a.get(key, '<missing>')} != {b.get(key, '<missing>')}")


def compare(reference, candidate):
    count = 0
    with (reference / "state.tsv").open() as left, (candidate / "state.tsv").open() as right:
        for day, (a, b) in enumerate(zip_longest(read_days(left), read_days(right))):
            compare_day(day, a, b)
            count += 1
            name = f"day-{day}.gz"
            x, y = reference / name, candidate / name
            if x.exists() != y.exists():
                raise ValueError(f"day {day}: missing checkpoint {name}")
            if x.exists():
                delta = xml_diff(checkpoint(x), checkpoint(y), "save")
                if delta:
                    raise ValueError(f"day {day} {delta}")
    if not count:
        raise ValueError("empty observation files")
    required = {f"day-{d}.gz" for d in (0, 1, 100, count - 1) if d < count}
    for directory in (reference, candidate):
        found = {p.name for p in directory.glob("day-*.gz")}
        if found != required:
            raise ValueError(f"{directory}: checkpoint set differs: {sorted(found ^ required)}")
    return count


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    try:
        print(f"{compare(args.reference, args.candidate)} daily observations matched")
    except (OSError, ValueError, ET.ParseError) as error:
        parser.exit(1, f"{error}\n")
