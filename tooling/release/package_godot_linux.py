"""Export a relocatable Linux x86_64 desktop from the built Godot extension."""

import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

from rust_notices import ROOT, notices

PROJECT = ROOT / "godot"
PROGRAM = "LinCity-NG.x86_64"
EXTENSION = "liblincity_godot.release.so"


def run(arguments, **options):
    result = subprocess.run(arguments, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, **options)
    if result.returncode or re.search(r"^(?:SCRIPT )?ERROR:", result.stdout, re.MULTILINE):
        raise SystemExit(f"{arguments[0]} fehlgeschlagen ({result.returncode})\n{result.stdout}")
    return result.stdout


def dependencies(elf, env=None):
    output = run(["ldd", elf], env=env)
    if "not found" in output:
        raise SystemExit(output)
    return {name: Path(path) for name, path in re.findall(
        r"^\s*(\S+)\s+=>\s+(/.*?)\s+\(0x", output, re.MULTILINE)}


def test_package(stage, args):
    clean_env = os.environ.copy()
    for key in ["LD_LIBRARY_PATH", "LD_PRELOAD", "GODOT_PROJECT_PATH"]:
        clean_env.pop(key, None)
    linked = dependencies(stage / PROGRAM, clean_env)
    linked.update(dependencies(stage / EXTENSION, clean_env))
    system_libs = []
    for name, path in linked.items():
        if any(path.resolve().is_relative_to(root) for root in ["/usr/lib", "/usr/lib64", "/lib", "/lib64"]):
            system_libs.append(name)
        else:
            raise SystemExit(f"Abhängigkeit außerhalb des Pakets und der Systembibliotheken: {path}")
    elf_files = [stage / PROGRAM, stage / EXTENSION]
    version_info = run(["readelf", "--version-info", *elf_files])
    requirements = {}
    for abi, version in re.findall(r"Name: (GLIBC|GLIBCXX)_([0-9.]+)", version_info):
        parts = tuple(int(v) for v in version.split("."))
        requirements[abi] = max(parts, requirements.get(abi, ()))
    minimum = ", ".join(k + "_" + ".".join(map(str, v)) for k, v in sorted(requirements.items()))
    (stage / "LIESMICH.txt").write_text(
        "LinCity-NG für Linux x86_64\n\n"
        "Starte LinCity-NG.x86_64 im entpackten Verzeichnis. Die ausführbare Datei, das PCK "
        "und die Bibliotheken müssen zusammenbleiben. Spielstände landen im persönlichen Datenverzeichnis.\n\n"
        "Voraussetzungen sind ein x86_64-Linux mit glibc, X11 oder Wayland und einem Treiber für "
        f"OpenGL 3.3. Die Binärdateien erfordern {minimum}. Diese Ausgabe verwendet folgende "
        "Systembibliotheken einschließlich deren "
        "üblicher Laufzeitabhängigkeiten:\n" + "\n".join(sorted(system_libs)) +
        "\n\nIm Menü Spiel und unter licenses stehen die Mitwirkenden und Lizenztexte.\n"
    )
    with tempfile.TemporaryDirectory() as scratch:
        clean_env["XDG_DATA_HOME"] = scratch
        clean_env["XDG_CONFIG_HOME"] = scratch
        clean_env["XDG_CACHE_HOME"] = scratch
        program = stage / PROGRAM
        display = [] if args.graphics else ["--headless"]
        result = run([program, *display, "--rendering-method", "gl_compatibility",
                      "--script", Path(__file__).with_name("check_godot_package.gd").resolve(),
                      "--", stage / "licenses", "--mute"], cwd=scratch, env=clean_env, timeout=120)
        if not (stage / "licenses/Godot.txt").is_file():
            raise SystemExit(f"Paketprüfung wurde nicht vollständig ausgeführt.\n{result}")
        run([program, *display, "--rendering-method", "gl_compatibility",
             "--script", Path(__file__).with_name("check_godot_package.gd").resolve(),
             "--", "--continue", "--mute"], cwd=scratch, env=clean_env, timeout=120)
        run([program, *display, "--rendering-method", "gl_compatibility", "--quit-after", "12", "--", "--mute"],
            cwd=scratch, env=clean_env, timeout=120)
        print("Linux-Paket mit Hauptmenü, Spielstand, Neustart und Credits geprüft.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--godot", default="godot", help="Godot-Editor mit Linux-Exportvorlage")
    parser.add_argument("--output", type=Path, required=True, help="Neues Paketverzeichnis")
    parser.add_argument("--graphics", action="store_true", help="Export im OpenGL-Fenster testen")
    args = parser.parse_args()
    target = args.output.resolve()
    if target.exists() and (not target.is_dir() or any(target.iterdir())):
        parser.error("Das Ziel muss neu oder leer sein.")
    if notices() != (PROJECT / "licenses/Rust-Bibliotheken.txt").read_text():
        raise SystemExit("Die Lizenzliste ist veraltet. Bitte tooling/release/rust_notices.py ausführen.")
    target.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".lincity-export-", dir=target.parent) as scratch:
        stage = Path(scratch) / "LinCity-NG"
        stage.mkdir()
        run([args.godot, "--headless", "--path", PROJECT, "--import"])
        run([args.godot, "--headless", "--path", PROJECT, "--export-release", "Linux Desktop", stage / PROGRAM])
        shutil.copytree(PROJECT / "licenses", stage / "licenses")
        test_package(stage, args)
        stage.rename(target)
    print(target)


if __name__ == "__main__":
    main()
