"""Export and verify the Apple Silicon test app on a Linux build host."""

import argparse
import os
from pathlib import Path
import plistlib
import re
import shutil
import subprocess
from tempfile import TemporaryDirectory
from zipfile import ZipFile

from package_godot_linux import PROJECT, run
from rust_notices import check_notices

MIN_MACOS = (15, 0)


def export(godot, template, app):
    with TemporaryDirectory(prefix="lincity-macos-") as scratch:
        root = Path(scratch)
        templates = root / "godot" / "export_templates" / "4.5.1.stable"
        templates.mkdir(parents=True)
        macos = "macos_template.app/Contents/MacOS/"
        with ZipFile(template) as original, ZipFile(templates / "macos.zip", "w") as thin_zip:
            release = macos + "godot_macos_release.arm64"
            try:
                original.getinfo(release)
            except KeyError:
                # The official 4.5.1 archive only ships the universal binary.
                universal = original.extract(macos + "godot_macos_release.universal", root)
                thin = root / "godot-arm64"
                run(["llvm-lipo", universal, "-thin", "arm64", "-output", thin])
                thin_zip.write(thin, release)
            for member in original.infolist():
                if member.filename.startswith(macos + "godot_macos_") and member.filename != release:
                    continue
                thin_zip.writestr(member, original.read(member))
        env = {**os.environ, "XDG_DATA_HOME": str(root)}
        run([godot, "--headless", "--path", PROJECT, "--import"], env=env)
        run([godot, "--headless", "--path", PROJECT, "--export-release",
             "macOS Apple Silicon", app], env=env)


def dependencies(binary):
    # otool -L also prints the library's own install name. Read load commands
    # instead, so this works equally for executables and dynamic libraries.
    output = run(["llvm-otool", "-l", binary])
    return re.findall(r"cmd LC_(?:LOAD|LOAD_WEAK|REEXPORT)_DYLIB\s+cmdsize \d+\s+name (.+) \(offset \d+\)", output)


def system_library(name):
    return name.startswith(("/usr/lib/", "/System/Library/"))


def verify_binary(binary):
    archs = run(["llvm-lipo", "-archs", binary]).strip().split()
    if "arm64" not in archs:
        raise SystemExit(f"{binary.name} enthält keinen arm64-Code.")
    info = run(["llvm-otool", "-arch", "arm64", "-l", binary])
    for version in re.findall(r"minos (\d+(?:\.\d+)*)", info):
        parts = tuple(map(int, version.split(".")))
        if (parts + (0, 0))[:3] > (*MIN_MACOS, 0):
            raise SystemExit(f"{binary.name} erfordert macOS {version}.")


def verify_signature(signer, binary):
    arguments = ["--config-file", "/dev/null", binary]
    signature = run([signer, "print-signature-info", *arguments])
    if "CodeSignatureFlags(ADHOC" not in signature:
        raise SystemExit(f"{binary.name} hat keine Ad-hoc-Signatur.")
    if "ADHOC | RUNTIME" in signature and "MAIN_BINARY" in signature:
        if "com.apple.security.cs.disable-library-validation" not in signature:
            raise SystemExit("Die Ad-hoc-App darf ihre mitgelieferte Erweiterung nicht laden.")
    verification = subprocess.run([signer, "verify", *arguments],
                                  stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if verification.returncode:
        # rcodesign 0.29 treats the empty CMS wrapper of an ad-hoc signature as
        # an error. Its Mach-O and Code Directory checks still run.
        diagnostics = {line for line in verification.stdout.splitlines()
                       if line and not line.startswith("(the verify command is known to be buggy")}
        if "cms: null" not in signature or diagnostics != {
            "CMS error: missing further values (at position 0)",
            "Error: problems reported during verification",
        }:
            raise SystemExit(f"{binary.name}: {verification.stdout}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--godot", type=Path, required=True)
    parser.add_argument("--template", type=Path, required=True, help="macos.zip der Godot-Exportvorlagen")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--rcodesign", default="rcodesign")
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists():
        parser.error("Das Ziel muss neu sein.")
    check_notices()

    output.mkdir(parents=True)
    app = output / "LinCity-NG.app"
    export(args.godot, args.template, app)
    contents = app / "Contents"
    frameworks = contents / "Frameworks"
    extension = frameworks / "liblincity_godot.release.dylib"
    if not extension.is_file():
        raise SystemExit("Das exportierte App-Bundle enthält die Rust-Anbindung nicht.")
    run(["llvm-install-name-tool", "-id", f"@rpath/{extension.name}", extension])
    with (contents / "Info.plist").open("rb") as stream:
        executable = contents / "MacOS" / plistlib.load(stream)["CFBundleExecutable"]
    if run(["llvm-lipo", "-archs", executable]).strip() != "arm64":
        raise SystemExit("Die exportierte Anwendung ist kein reiner ARM64-Build.")
    symbols = run(["llvm-nm", "--defined-only", "--extern-only", extension])
    if not re.search(r"\b_lincity_library_init$", symbols, re.MULTILINE):
        raise SystemExit("Die macOS-Bibliothek exportiert den Godot-Einstiegspunkt nicht.")

    licenses = output / "licenses"
    shutil.copytree(PROJECT / "licenses", licenses)
    for binary in [executable, *frameworks.glob("*.dylib")]:
        verify_binary(binary)
    for name in dependencies(extension):
        if not system_library(name):
            raise SystemExit(f"{extension.name} hat eine externe Abhängigkeit: {name}")
    run([args.rcodesign, "sign", "--config-file", "/dev/null", app])
    for binary in [executable, *frameworks.glob("*.dylib")]:
        verify_signature(args.rcodesign, binary)
    if not (contents / "_CodeSignature" / "CodeResources").is_file():
        raise SystemExit("Das App-Bundle hat keine signierte Ressourcentabelle.")
    if not (contents / "Resources" / f"{app.stem}.pck").is_file():
        raise SystemExit("Die exportierte Anwendung enthält keine Spieldaten.")

    (output / "LIESMICH.txt").write_text(
        "LinCity-NG für Apple Silicon, macOS 15 oder neuer\n\n"
        "Das ZIP entpacken und LinCity-NG.app öffnen. Das Testpaket ist ad hoc "
        "signiert. Wenn Gatekeeper den Start blockiert, einmal öffnen und danach "
        "unter Systemeinstellungen → Datenschutz & Sicherheit „Dennoch öffnen“ wählen.\n"
        "Lizenzinformationen liegen im Verzeichnis licenses und im Spielmenü.\n"
    )
    print(output)


if __name__ == "__main__":
    main()
