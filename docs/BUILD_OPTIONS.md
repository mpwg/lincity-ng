# Build-Optionen

`cargo godot-build` baut die native Erweiterung `lincity_godot` mit dem Rust-Simulationskern und legt die Bibliothek in `godot/bin/` ab. Godot lädt sie beim Start des Editors oder des Spiels aus den Quellen. Standardmäßig verwendet der Build das optimierte Cargo-Profil `release`.

Der Cargo-Workspace im Stammverzeichnis verwaltet den Simulationskern, die Godot-Erweiterung und das Build-Werkzeug mit einer gemeinsamen `Cargo.lock`. Die Erweiterung baut auf `godot-rust` und der GDExtension-API von Godot 4.5 auf. Das Godot-Projekt verwendet den Editor 4.5.1. [Abhängigkeiten](DEPENDENCIES.md)

- `--export` legt die Bibliothek unter dem Dateinamen für Godots Release-Exportvorlagen ab. Der Editor-Build und die Exportbibliothek können nebeneinander in `godot/bin/` liegen. Das vollständige Paket erstellen die plattformspezifischen Skripte in `tooling/release/`.
- `--debug` baut mit dem unoptimierten Cargo-Profil `dev` und erzeugt Debugsymbole.
- `--target TRIPLE` gibt das Rust-Ziel für den Build der Erweiterung an. Den Linker und dessen Optionen für Cross-Builds kannst du über Cargo konfigurieren.

[Beispielbefehle für Editor und Release-Exporte](GODOT_DESKTOP.md)

`cargo test --workspace --locked` führt alle Rust-Tests aus. Den Simulationskern kannst du mit `cargo test --locked -p rustcity-core` separat prüfen.

## Vergleich mit dem archivierten C++-Kern

Für die optionalen Kreuztests brauchst du eine separate Arbeitskopie des Archivbranches. Setze `archive` auf den absoluten Pfad zu diesem Checkout und führe die folgenden Befehle vom Stammverzeichnis des aktuellen Godot-Projekts aus.
[Archivbranch und historische Vergleichsergebnisse](https://github.com/mpwg/lincity-ng/blob/archive/legacy-2d-cpp/docs/rustcity-core-paritaet.md)

```sh
archive=/absoluter/pfad/zum/archiv
cmake -S "$archive/lincity-core" -B "$archive/build" -DBUILD_TESTING=ON -DCMAKE_BUILD_TYPE=Release
cmake --build "$archive/build" --target core_replay core_generation_reference
RUSTCITY_CPP_ARCHIVE="$archive" \
  RUSTCITY_CPP_REPLAY="$archive/build/tests/core_replay" \
  RUSTCITY_CPP_GENERATOR="$archive/build/parity/core_generation_reference" \
  cargo test --locked -p rustcity-core -- --ignored --skip generated_village_wins_a_sustainable_economy
```
