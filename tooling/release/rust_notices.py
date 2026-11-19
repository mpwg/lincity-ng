"""Regenerate the Linux desktop's Rust copyright and license notices from Cargo."""

from collections import defaultdict
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "lincity-legacy/rust-adapter/Cargo.toml"
TARGET = "x86_64-unknown-linux-gnu"


def cargo(*args):
    return subprocess.check_output(
        ["cargo", *args, "--locked", "--manifest-path", MANIFEST], text=True
    )


def notices(target=TARGET):
    metadata = json.loads(cargo("metadata", "--format-version", "1", "--filter-platform", target))
    packages = {(p["name"], p["version"]): p for p in metadata["packages"] if p["source"]}
    tree = cargo("tree", "--target", target, "-e", "normal", "--prefix", "none", "--format", "{p}")
    notices_by_text = defaultdict(list)
    for line in sorted(set(tree.splitlines())):
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
        if not license_files:
            raise ValueError(f"Kein Lizenztext für {name} {version}")
        for file in license_files:
            notices_by_text[file.read_text().strip()].append(f"{name} {version} · {file.name}")
    platform = "Linux x86_64" if target == TARGET else target
    blocks = [f"Rust-Abhängigkeiten von LinCity-NG für {platform}"]
    for text, labels in notices_by_text.items():
        blocks.extend(["\n".join(labels), text])
    return "\n\n".join(blocks) + "\n"


if __name__ == "__main__":
    (ROOT / "godot/licenses/Rust-Bibliotheken.txt").write_text(notices())
