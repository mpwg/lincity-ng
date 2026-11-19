# Godot-Desktop veröffentlichen

1. Die Änderungen aus `docs/changes/` in `CHANGELOG.md` übernehmen und `config/version` in `godot/project.godot` aktualisieren.
2. Alle Rust-Crates mit `cargo test --workspace --locked` prüfen. Die Editor-Bibliothek mit `cargo godot-build` bereitstellen und die Godot-Tests ausführen. [Testbefehle](GODOT_DESKTOP.md#prüfung)
3. Die Lizenzliste mit `python3 tooling/release/rust_notices.py` aktualisieren. Mit `cargo godot-build --export` die Erweiterung für die jeweilige Zielplattform in `godot/bin/` bereitstellen. Anschließend mit dem passenden Skript und der zu Godot 4.5.1 passenden Exportvorlage paketieren. [Linux](GODOT_DESKTOP.md#linux-export) · [macOS](GODOT_DESKTOP.md#apple-silicon-testbuild) · [Windows](GODOT_DESKTOP.md#windows-x64-testbuild)
4. Die entpackten Pakete prüfen. Spielstart, Ladepfade, gespeicherte Stadt und Lizenzansicht kontrollieren.
5. Den Release-Stand mit `lincity-ng-<version>` taggen. Die geprüften Pakete und die Versionshinweise dem GitHub-Release zuordnen.
