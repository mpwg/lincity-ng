"""Report measured GCC coverage and uncovered LinCity Core source locations."""

import argparse
from collections import defaultdict
import json
from pathlib import Path
import subprocess


CORE = Path(__file__).resolve().parents[1]
SOURCE_DIRECTORIES = (CORE / "lincity", CORE / "util")


def collect(build, gcov):
    lines, functions, branches = (defaultdict(dict) for _ in range(3))
    for obj in sorted(build.rglob("*.gcno")):
        # Restrict instrumentation to real core targets. Header code inlined
        # by those targets counts, platform and third-party headers do not.
        if not any(obj.is_relative_to(build / source.name) for source in SOURCE_DIRECTORIES):
            continue
        result = subprocess.run([gcov, "-jbt", str(obj)], capture_output=True, text=True, check=True)
        report = json.loads(result.stdout)
        for source in report["files"]:
            path = (Path(report["current_working_directory"]) / source["file"]).resolve()
            if not any(path.is_relative_to(directory) for directory in SOURCE_DIRECTORIES):
                continue
            name = str(path.relative_to(CORE))
            for function in source["functions"]:
                key = function["start_line"], function["demangled_name"]
                count = function["execution_count"]
                functions[name][key] = functions[name].get(key, 0) + count
            for line in source["lines"]:
                key = line["line_number"]
                lines[name][key] = lines[name].get(key, 0) + line["count"]
                for index, branch in enumerate(line["branches"], 1):
                    # Even identical inlined expressions may acquire different
                    # edges in different translation units. Keep native edges.
                    if not branch["throw"]:
                        edge = key, str(obj.relative_to(build)), index
                        branches[name][edge] = branches[name].get(edge, 0) + branch["count"]
    if not lines:
        raise ValueError("no instrumented core sources found. Use a standalone GCC coverage build")
    return lines, functions, branches


def summarize(entries):
    total = sum(map(len, entries.values()))
    covered = sum(value > 0 for source in entries.values() for value in source.values())
    percent = 100 * covered / total if total else 100
    return f"{covered}/{total} ({percent:.2f}%)"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("--gcov", default="gcov", help="gcov matching the GCC used for the build")
    parser.add_argument("--details", action="store_true", help="list uncovered locations")
    args = parser.parse_args()
    try:
        lines, functions, branches = collect(args.build.resolve(), args.gcov)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"{error}\n")
    for label, entries in (
        ("Lines", lines), ("Functions", functions), ("Branches excluding throw edges", branches)
    ):
        print(f"{label}: {summarize(entries)}")
        if args.details:
            for path, source in sorted(entries.items()):
                for location, count in sorted(source.items()):
                    if count == 0:
                        print(f"  {path}:" + ": ".join(map(str, location if isinstance(location, tuple) else (location,))))


if __name__ == "__main__":
    main()
