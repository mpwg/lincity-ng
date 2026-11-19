"""Export and verify the Windows x64 Godot test package from Linux."""

import argparse
from pathlib import Path
import re
import shutil
from tempfile import TemporaryDirectory
from zipfile import ZIP_DEFLATED, ZipFile

from package_godot_linux import PROJECT, run
from rust_notices import check_notices

PRESET = "Windows x64"
EXE = "LinCity-NG.exe"
EXTENSION = "liblincity_godot.release.dll"
SYSTEM_DLLS = frozenset("""
    advapi32 avrt bcrypt bcryptprimitives crypt32 d3d12 dwmapi dwrite dxgi gdi32 imm32 iphlpapi
    kernel32 kernelbase msvcrt ntdll ole32 oleaut32 opengl32 psapi rpcrt4 shcore
    shell32 shlwapi ucrtbase uiautomationcore user32 userenv version winmm ws2_32
    wsock32
""".split())


def windows_dll(name):
    return name.startswith(("api-ms-win-", "ext-ms-win-")) or name.removesuffix(".dll") in SYSTEM_DLLS


def pe_info(binary):
    result = run(["llvm-objdump", "-p", binary])
    if "file format coff-x86-64" not in result:
        raise SystemExit(f"{binary.name} ist keine Windows-x64-Binärdatei.")
    return result


def bundle_dlls(stage, directories):
    available = {}
    for directory in directories:
        for file in directory.glob("*.dll"):
            if file.is_file():
                available.setdefault(file.name.lower(), file)
    packaged = {file.name.lower() for file in stage.glob("*.dll")}
    pending = [stage / EXE, *stage.glob("*.console.exe"), *stage.glob("*.dll")]
    while pending:
        binary = pending.pop()
        info = pe_info(binary)
        for name in re.findall(r"^\s*DLL Name: (\S+)", info, re.MULTILINE):
            key = name.lower()
            if key in packaged or windows_dll(key):
                continue
            if key not in available:
                raise SystemExit(f"{binary.name} benötigt die fehlende Windows-DLL {name}.")
            destination = stage / key
            shutil.copyfile(available[key], destination)
            packaged.add(key)
            pending.append(destination)
    if "lincity_library_init" not in pe_info(stage / EXTENSION):
        raise SystemExit("Die Windows-DLL exportiert den Godot-Einstiegspunkt nicht.")
    print("x64-DLL-Abhängigkeiten geprüft:", ", ".join(sorted(packaged)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--godot", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="Pfad des neuen ZIP-Archivs")
    parser.add_argument("--dll-path", type=Path, action="append", default=[])
    parser.add_argument("--license", type=Path, action="append", default=[])
    args = parser.parse_args()
    target = args.output.resolve()
    if target.suffix.lower() != ".zip" or target.exists():
        parser.error("--output muss auf eine neue ZIP-Datei zeigen.")
    check_notices()
    target.parent.mkdir(parents=True, exist_ok=True)
    with TemporaryDirectory(prefix=".lincity-windows-", dir=target.parent) as scratch:
        stage = Path(scratch) / "LinCity-NG-Windows-x64"
        stage.mkdir()
        run([args.godot, "--headless", "--path", PROJECT, "--import"])
        run([args.godot, "--headless", "--path", PROJECT, "--export-release", PRESET, stage / EXE])
        for required in [EXE, EXTENSION, "LinCity-NG.pck"]:
            if not (stage / required).is_file():
                raise SystemExit(f"Export unvollständig: {required} fehlt.")
        bundle_dlls(stage, args.dll_path)

        licenses = stage / "licenses"
        shutil.copytree(PROJECT / "licenses", licenses)
        for license_file in args.license:
            shutil.copyfile(license_file, licenses / license_file.name)
        (stage / "LIESMICH.txt").write_text(
            "LinCity-NG für Windows 10 oder neuer (x64)\n\n"
            "Das gesamte ZIP entpacken und LinCity-NG.exe öffnen. Die EXE, "
            "die PCK-Datei und die DLLs bleiben zusammen im Paketordner. "
            "Der Testbuild hat keine Authenticode-Signatur.\n"
            "Lizenzinformationen liegen im Verzeichnis licenses und im Spielmenü.\n",
            encoding="utf-8",
        )
        archive_path = Path(scratch) / target.name
        with ZipFile(archive_path, "w", ZIP_DEFLATED) as archive:
            for file in sorted(stage.rglob("*")):
                archive.write(file, file.relative_to(stage.parent))
        with ZipFile(archive_path) as archive:
            broken = archive.testzip()
            if broken is not None:
                raise SystemExit(f"ZIP-Prüfung fehlgeschlagen: {broken}")
        archive_path.rename(target)
    print(target)


if __name__ == "__main__":
    main()
