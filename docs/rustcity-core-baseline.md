# C++-Referenz für RustCity Core

Am 18. November 2026 wurde der Simulationsstand
`d4346dd34fafea3384360b593fa1d59ffa2b6324` unter Linux x86-64 aus frischen
Bauverzeichnissen geprüft.

| Konfiguration | Ergebnis |
| --- | --- |
| Debug, eigenständiger Core, GCC 16.2.1 | 29 von 29 CTests bestanden |
| Release, Hauptprojekt ohne GUI, GCC 16.2.1 | 29 von 29 CTests bestanden |
| Debug, Clang 23.1.5, ASan und UBSan mit Leckerkennung | 29 von 29 CTests bestanden, keine Sanitizer-Meldungen |
| Historische Desktop-Referenz, GCC 16.2.1 | 48 Spielstände und 12 Meldungsfolgen stimmten überein |

Der historische Vergleich lud Beach, Rocket_98, bad_times, extreme_arid,
extreme_wetland und good_times jeweils mit und ohne Fahrzeugverkehr.
Pro Lauf verglich er die dekomprimierten XML-Daten an den Tagen 0, 1, 100 und
1201 sowie die geordnete Meldungsfolge. Beide Zufallsgeneratoren starteten mit
Seed 714. Das Vergleichsprogramm baut den historischen Commit
`21796de530d68b30881c6b0f6a6a9d6bbfb1e31a` mit den dokumentierten Korrekturen
für Bahnübergänge und Kohlesteuer. Es entfernt beim XML-Vergleich das zusätzliche
Attribut `diffusion-active="1"` aus der aktuellen Ausgabe. Der CTest
`core.characterization.saves` prüft die Fortsetzung der Diffusion nach dem Laden.
[Vergleichsvertrag](../lincity-core/parity/PARITY-CONTRACT.md)

## Wiederholung

Benötigt werden CMake, Ninja, ein C++17-Compiler, pkg-config, zlib, libxml2,
libxml++ 5.0 und fmt. Für dieselben 29 Tests müssen Python 3 und jemalloc
vorhanden sein. Der historische Lauf benötigt zusätzlich den historischen
Git-Commit, die SDL3-Desktopabhängigkeiten und xsltproc. Geprüft wurde mit
CMake 4.4.5, Python 3.14.10, libxml2 2.15.4 und libxml++ 5.4.0.
[Abhängigkeiten](DEPENDENCIES.md)

Die Befehle laufen vom Repository-Stamm aus. Auf dem Prüfhost setzt
`source build/local-deps/env.sh` die Suchpfade für den vorhandenen lokalen
Abhängigkeitsprefix. Bei systemweit installierten Abhängigkeiten genügen
die regulären Suchpfade.

```sh
base=build/rustcity-baseline
cmake -S lincity-core -B "$base/debug" -G Ninja -DBUILD_TESTING=ON -DCMAKE_BUILD_TYPE=Debug
cmake --build "$base/debug" -j3
ctest --test-dir "$base/debug" -j3 --output-on-failure

cmake -S . -B "$base/release" -G Ninja -DLINCITY_BUILD_GUI=OFF -DBUILD_TESTING=ON -DCMAKE_BUILD_TYPE=Release
cmake --build "$base/release" -j3
ctest --test-dir "$base/release" -j3 --output-on-failure
python3 -B tooling/regression/compare_scenarios.py "$base/release"

CC=clang CXX=clang++ cmake -S lincity-core -B "$base/sanitizers" -G Ninja \
  -DBUILD_TESTING=ON -DCMAKE_BUILD_TYPE=Debug \
  '-DCMAKE_CXX_FLAGS=-fsanitize=address,undefined -fno-omit-frame-pointer'
cmake --build "$base/sanitizers" -j3
ASAN_OPTIONS=detect_leaks=1:halt_on_error=1 UBSAN_OPTIONS=print_stacktrace=1:halt_on_error=1 \
  ctest --test-dir "$base/sanitizers" -j3 --output-on-failure
```
