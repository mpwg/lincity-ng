"""Regenerate the desktop's Rust copyright and license notices from Cargo."""

from collections import defaultdict
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "Cargo.toml"
NOTICE = ROOT / "godot/licenses/Rust-Bibliotheken.txt"
TARGETS = (
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-gnullvm",
    "x86_64-pc-windows-msvc",
)
GODOT_RUST_LICENSE = Path(__file__).with_name("licenses") / "godot-rust-0.5.5.txt"
GODOT_RUST_SOURCE = "https://github.com/godot-rust/gdext/tree/v0.5.5"


def cargo(*args):
    return subprocess.check_output(
        ["cargo", *args, "--locked", "--manifest-path", MANIFEST], text=True
    )


def notices():
    metadata = json.loads(cargo("metadata", "--format-version", "1"))
    packages = {(p["name"], p["version"]): p for p in metadata["packages"] if p["source"]}
    entries = set()
    for target in TARGETS:
        tree = cargo("tree", "--package", "rustcity-godot", "--target", target,
                     "-e", "normal", "--prefix", "none", "--format", "{p}")
        entries.update(tree.splitlines())
    notices_by_text = defaultdict(list)
    for line in sorted(entries):
        name, version, *_ = line.split()
        key = (name, version.removeprefix("v"))
        if key not in packages:
            continue
        package = packages.pop(key)
        folder = Path(package["manifest_path"]).parent
        license_files = sorted(
            f for f in folder.iterdir() if f.is_file()
            and f.name.upper().startswith(("LICENSE", "LICENCE", "COPYRIGHT", "COPYING", "NOTICE", "UNLICENSE"))
        )
        if not license_files and version == "v0.5.5" and name in {
            "godot", "godot-core", "godot-cell", "godot-ffi", "godot-macros",
        }:
            # The published crates omit the repository's top-level License.txt.
            text = GODOT_RUST_LICENSE.read_text(encoding="utf-8").strip()
            notices_by_text[text].append(f"{name} {version} · {GODOT_RUST_SOURCE}")
            continue
        if not license_files:
            raise ValueError(f"Kein Lizenztext für {name} {version}")
        for file in license_files:
            notices_by_text[file.read_text(encoding="utf-8").strip()].append(f"{name} {version} · {file.name}")
    blocks = ["Rust-Abhängigkeiten von LinCity-NG für Linux, macOS und Windows"]
    for text, labels in notices_by_text.items():
        blocks.extend(["\n".join(labels), text])
    return "\n\n".join(blocks) + "\n"


def check_notices():
    if notices() != NOTICE.read_text(encoding="utf-8"):
        raise SystemExit("Die Lizenzliste ist veraltet. Bitte tooling/release/rust_notices.py ausführen.")


if __name__ == "__main__":
    NOTICE.write_text(notices(), encoding="utf-8")
