# Godot-Desktop veröffentlichen

1. Die Änderungen aus `docs/changes/` in `CHANGELOG.md` übernehmen und die Versionsnummer in `CMakeLists.txt` aktualisieren.
2. Den Rust-Kern und den Godot-Desktop prüfen. Dazu eine neue Stadt starten, einen Spielstand speichern und in einem neuen Prozess laden. [Testbefehle](GODOT_DESKTOP.md#prüfung)
3. Die native Erweiterung mit `GODOTCPP_TARGET=template_release` für jede Zielplattform separat bauen und mit der passenden Godot-Exportvorlage paketieren. [Linux](GODOT_DESKTOP.md#linux-export) · [macOS](GODOT_DESKTOP.md#apple-silicon-testbuild) · [Windows](GODOT_DESKTOP.md#windows-x64-testbuild)
4. Die entpackten Pakete prüfen. Spielstart, Ladepfade, gespeicherte Stadt und Lizenzansicht kontrollieren.
5. Den Release-Stand mit `lincity-ng-<version>` taggen. Die geprüften Pakete und die Versionshinweise dem GitHub-Release zuordnen.
