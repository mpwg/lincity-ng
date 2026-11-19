"""Export, relocate and verify the Apple Silicon Godot test app."""

import argparse
import os
from pathlib import Path
import plistlib
import re
import shutil
import tempfile

from package_godot_linux import PROJECT, run
from rust_notices import notices

MIN_MACOS = (15, 0)


def dependencies(binary):
    # otool -L also prints the library's own install name. Read load commands
    # instead, so this works equally for executables and dynamic libraries.
    output = run(["otool", "-l", binary])
    return re.findall(r"cmd LC_(?:LOAD|LOAD_WEAK|REEXPORT)_DYLIB\s+cmdsize \d+\s+name (.+) \(offset \d+\)", output)


def system_library(name):
    return name.startswith(("/usr/lib/", "/System/Library/"))


def copy_licenses(source, destination):
    for parent in source.parents:
        if parent.parent.parent.name != "Cellar":
            continue
        names = sorted(p for p in parent.iterdir() if p.is_file() and
                       p.name.upper().startswith(("COPYING", "LICENSE", "COPYRIGHT", "NOTICE")))
        for item in names:
            shutil.copyfile(item, destination / f"{parent.parent.name}-{item.name}")
        break


def relocate(extension, frameworks, library_paths, licenses):
    pending = [(extension, extension)]
    copied = {}
    while pending:
        source, bundled = pending.pop()
        for name in dependencies(bundled):
            if system_library(name):
                continue
            if name.startswith("@rpath/"):
                paths = [folder / name.removeprefix("@rpath/")
                         for folder in [source.parent, *library_paths]]
            elif name.startswith("@loader_path/"):
                paths = [source.parent / name.removeprefix("@loader_path/")]
            else:
                paths = [Path(name)]
            original = next((path.resolve() for path in paths if path.is_file()), None)
            if original is None:
                raise SystemExit(f"{source}: Bibliothek {name} fehlt.")
            filename = Path(name).name
            target = frameworks / filename
            if filename in copied and copied[filename] != original:
                raise SystemExit(f"Zwei verschiedene Bibliotheken heißen {filename}.")
            if filename not in copied:
                shutil.copyfile(original, target)
                run(["chmod", "u+w", target])
                run(["install_name_tool", "-id", f"@rpath/{filename}", target])
                copy_licenses(original, licenses)
                copied[filename] = original
                pending.append((original, target))
            run(["install_name_tool", "-change", name, f"@loader_path/{filename}", bundled])
    return [frameworks / name for name in copied]


def verify_binary(binary):
    archs = run(["lipo", "-archs", binary]).strip().split()
    if "arm64" not in archs:
        raise SystemExit(f"{binary.name} enthält keinen arm64-Code.")
    info = run(["vtool", "-arch", "arm64", "-show-build", binary])
    for version in re.findall(r"minos (\d+(?:\.\d+)*)", info):
        parts = tuple(map(int, version.split(".")))
        if (parts + (0, 0))[:3] > (*MIN_MACOS, 0):
            raise SystemExit(f"{binary.name} erfordert macOS {version}.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--godot", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--library-path", type=Path, action="append", default=[])
    parser.add_argument("--libxmlpp-source", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists():
        parser.error("Das Ziel muss neu sein.")

    output.mkdir(parents=True)
    app = output / "LinCity-NG.app"
    run([args.godot, "--headless", "--path", PROJECT, "--import"])
    run([args.godot, "--headless", "--path", PROJECT, "--export-release", "macOS Apple Silicon", app])
    contents = app / "Contents"
    frameworks = contents / "Frameworks"
    extension = frameworks / "liblincity_godot.release.dylib"
    if not extension.is_file():
        raise SystemExit("Das exportierte App-Bundle enthält die Rust-Anbindung nicht.")
    with (contents / "Info.plist").open("rb") as stream:
        executable = contents / "MacOS" / plistlib.load(stream)["CFBundleExecutable"]
    if run(["lipo", "-archs", executable]).strip() != "arm64":
        raise SystemExit("Die exportierte Anwendung ist kein reiner ARM64-Build.")

    licenses = output / "licenses"
    shutil.copytree(PROJECT / "licenses", licenses)
    (licenses / "Rust-Bibliotheken.txt").write_text(notices("aarch64-apple-darwin"))
    shutil.copyfile(args.libxmlpp_source, output / "libxml++-5.4.0.tar.xz")
    dylibs = relocate(extension, frameworks, args.library_path, licenses)
    for binary in [executable, *frameworks.glob("*.dylib")]:
        verify_binary(binary)
    for binary in [extension, *dylibs]:
        for name in dependencies(binary):
            if system_library(name):
                continue
            if not name.startswith("@loader_path/") or not (frameworks / Path(name).name).is_file():
                raise SystemExit(f"{binary.name} hat eine externe Abhängigkeit: {name}")
    for binary in frameworks.glob("*.dylib"):
        run(["codesign", "--force", "--sign", "-", binary])
    run(["codesign", "--force", "--sign", "-", app])
    run(["codesign", "--verify", "--deep", "--strict", app])

    (output / "LIESMICH.txt").write_text(
        "LinCity-NG für Apple Silicon, macOS 15 oder neuer\n\n"
        "Das ZIP entpacken und LinCity-NG.app öffnen. Das Testpaket ist ad hoc "
        "signiert. Wenn Gatekeeper den Start blockiert, einmal öffnen und danach "
        "unter Systemeinstellungen → Datenschutz & Sicherheit „Dennoch öffnen“ wählen.\n"
        "Lizenzinformationen liegen im Verzeichnis licenses und im Spielmenü.\n"
    )
    with tempfile.TemporaryDirectory() as scratch:
        env = {key: value for key, value in os.environ.items()
               if not key.startswith("DYLD_")}
        for script, arguments in [
            (Path(__file__).with_name("check_godot_package.gd"), [licenses]),
            (PROJECT / "tests/outcomes.gd", []),
            (PROJECT / "tests/history.gd", []),
            (PROJECT / "tests/audio.gd", []),
        ]:
            result = run([executable, "--headless", "--script", script, "--",
                          *arguments, *(["--mute"] if script.stem != "audio" else [])],
                         cwd=scratch, env=env, timeout=120)
            print(result, end="")
    print(output)


if __name__ == "__main__":
    main()
