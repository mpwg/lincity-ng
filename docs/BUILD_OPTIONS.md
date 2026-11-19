# Build-Optionen

CMake baut die native Erweiterung `lincity_godot` und dabei den Rust-Simulationskern. Die Optionen werden beim Konfigurieren mit `-D` gesetzt.

- `CMAKE_BUILD_TYPE` wählt bei Generatoren wie Ninja und Unix Makefiles die Build-Konfiguration. Die Befehle in der Dokumentation verwenden `Release`. Für Debugging stehen auch `Debug` und `RelWithDebInfo` zur Verfügung.
- `GODOTCPP_TARGET` wählt die Zielversion der Godot-Bindings. Der dokumentierte Editor-Build verwendet den Standard `template_debug`. Setze `template_release` in einem eigenen Build-Verzeichnis, wenn die Erweiterung für ein Release-Paket gebaut wird.
- `LINCITY_GODOT_OUTPUT_DIR` setzt das Ausgabeverzeichnis der Erweiterung. Standardmäßig schreibt CMake nach `godot/bin/`, wo das Godot-Projekt und die Paketierskripte die Bibliothek erwarten.
- `CMAKE_TOOLCHAIN_FILE` und `CMAKE_PREFIX_PATH` geben bei Bedarf die Toolchain und die Suchpfade für Bibliotheken der Zielplattform an.

[Beispielbefehle für Editor und Release-Exporte](GODOT_DESKTOP.md)

Die Rust-Tests benötigen keine CMake-Konfiguration. Führe sie vom Stammverzeichnis mit `cargo test --locked --manifest-path rustcity-core/Cargo.toml` aus.

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
  cargo test --locked --manifest-path rustcity-core/Cargo.toml -- --ignored --skip generated_village_wins_a_sustainable_economy
```
