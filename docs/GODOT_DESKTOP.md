# LinCity-NG in 3D spielen

Im Hauptmenü lässt sich eine neue Stadt anlegen, ein Szenario auswählen, ein Spielstand laden oder die zuletzt gesicherte Stadt fortsetzen. Das Flussviertel ist als „Kleinstadt am Fluss“ unter „Neues Spiel“ verfügbar. Godot 4.5.1 zeichnet die frei drehbare Stadt mit OpenGL. RustCity verwaltet Simulation, Baukosten, Handel und Spielstände.

„Spiel → Pausenmenü“ hält die Simulation an. „Fortfahren“ setzt die laufende Stadt fort. Beim Wechsel zum Hauptmenü und beim regulären Beenden sichert das Spiel die Stadt für „Fortsetzen“. Ein Fehler beim Sichern wird im Pausenmenü angezeigt. Unter „Spiel laden“ lassen sich auch Spielstände der klassischen Oberfläche öffnen.

## Start aus den Quellen

Cargo baut den Rust-Simulationskern und die Rust-GDExtension für Godot. Die Erweiterung stellt GDScript die Klasse `CityWorld` bereit. Führe die Befehle vom Stammverzeichnis des Repositorys aus. [Abhängigkeiten](DEPENDENCIES.md)

```sh
cargo godot-build
godot --headless --path godot --import
godot --path godot
```

Mit `godot --path godot -- --bridge-view` startet das Flussviertel direkt von der Brückenseite. Mit `--city=/absoluter/pfad/stadt.gz` hinter dem doppelten Bindestrich startet Godot direkt in diesem Spielstand.

| Aktion | Bedienung |
|---|---|
| Blick drehen und neigen | Rechte Maustaste ziehen, Q und E zum Drehen |
| Verschieben und zoomen | Mittlere Maustaste ziehen, WASD oder Pfeiltasten, Mausrad zum Zoomen |
| Untersuchen, Straße, Wohnen, Markt, Abriss | Umschalt+1 bis Umschalt+5 oder Bauleiste. B schaltet den Abriss um. |
| Weitere Bauwerkzeuge | Kategorien in der Bauleiste zeigen Grundpreis, Grundfläche und nötige Technologie. |
| Kartenebenen | Tasten 1 bis 0 oder Auswahl unter der Minikarte. Die Akzenttaste links von 1 öffnet den Stadtplan. |
| Verkehrsweg bauen | Linke Maustaste ziehen, Strg wechselt den Knick im Verlauf |
| Pausenmenü | Esc oder „Spiel → Pausenmenü“ |
| Hilfe und Spielstände | F1 für die Hilfe, F9 zum Schnellladen, F12 zum Schnellspeichern |
| Verlauf und Siegziele | Spiel → Verlauf und Nachhaltigkeit |
| Anzeige, Ton und Startvorgaben | Spiel → Einstellungen |

Die Nachhaltigkeitsansicht zeigt den Fortschritt aller sechs Siegkriterien. Die Verlaufsdiagramme lesen die abgeschlossenen Monate aus dem Spielstand und zeigen fehlende ältere Aufzeichnungen als Lücken. Ein erfolgreicher letzter Evakuierungsflug und eine nachhaltige Wirtschaft öffnen jeweils einen eigenen Siegdialog.

Die Stadtleiste zeigt den Technologiestand und öffnet die aktuelle Warenversorgung. „Spiel → Stadtstatistik“ zeigt unter anderem Gebäudezahlen, Evakuierte und Ergebnisse der Raketenstarts und kann den Bericht exportieren. Die Bauvorschau zeigt nur die Kosten der Felder, die in der angezeigten Reihenfolge tatsächlich gebaut werden können.

Eine gezogene Bau- oder Abrissauswahl umfasst höchstens 4.096 Felder. Bei einer größeren Auswahl meldet die Vorschau die Grenze. Bau und Abriss lassen sich dann abschnittsweise ausführen.

Unter „Einstellungen“ lassen sich Musik und Effekte getrennt regeln sowie der Fahrzeugverkehr ausblenden. Im Schnelllauf fährt die Simulation so viele Tage pro Bild wie das Zeitbudget erlaubt. Das Startargument `--mute` hinter dem doppelten Bindestrich schaltet die Sitzung stumm. Die Wahl zwischen Deutsch und Englisch wird mit dem nächsten Start für Oberfläche, Hilfe und Meldungen wirksam. Unter X11 lässt sich die Fensterauflösung im Menü wählen. Unter Wayland wird die Fenstergröße über den Fenstermanager geändert. Vollbild steht auf beiden Plattformen zur Verfügung.

Bei schmalen Fenstern bleibt die Schrift größer. Bauleiste, Feldbericht und Minikarte nutzen getrennte Bereiche. Die Anordnung wurde bei 640 × 480 Pixeln mit deutschen und englischen Texten geprüft.

## Assets und Referenzviertel

Eine Kartenzelle entspricht einer Godot-Welteinheit. Der Modellkatalog verbindet die Gruppen des Spielkerns mit den Gebäude-, Verkehrs- und Geländevarianten. Beim Platzieren bestimmt die Straßenanbindung die Vorderseite. Für das Bild entscheidende Fassaden, Dächer und Rückseiten sind ausmodelliert. Auswahlkörper und vereinfachte Fernmodelle werden separat exportiert. Die Bau- und Terrainmodelle werden lokal erzeugt und mit dem Projekt verteilt. [Blender-Quellen und Export der Architektur](../tooling/art/architecture/README.md) · [Katalog und Modellzuordnung](../godot/world/model_catalog.gd)

Die folgenden Befehle erzeugen die übrigen Katalogmodelle beziehungsweise den Spielstand aus „Good Times“ neu. Der Spielstand entsteht durch dieselben Landschafts- und Bauaktionen wie im laufenden Spiel.

```sh
godot --headless --path godot -s res://world/authoring/bake_models.gd
godot --headless --path godot -s ../tooling/art/build_riverside.gd
```

## Linux-Export

Die Linux-Releasevorlage muss zur installierten Godot-Version passen. Der Editor und das exportierte Spiel erhalten jeweils die passende native Bibliothek. Zusätzlich zum Editor-Build die Release-Erweiterung bauen und das Paket in einem neuen Zielverzeichnis erstellen.

```sh
cargo godot-build --export
python3 tooling/release/package_godot_linux.py --godot godot --output build/godot-linux-final --graphics
```

Die Paketierung verwendet Python, `ldd` und `readelf`. Sie prüft Spielstart, PCK, Bibliotheken, Credits und Spielstand aus einem frischen Arbeitsverzeichnis. Die Datei `LIESMICH.txt` im Paket nennt die erforderlichen Linux-Systembibliotheken.

## Prüfung

Baue die Editor-Bibliothek und importiere das Projekt vor dem ersten Godot-Test wie oben beschrieben.

```sh
cargo test --workspace --locked
godot --headless --path godot -s res://tests/model_catalog.gd
godot --headless --path godot -s res://tests/session.gd -- --mute
godot --headless --path godot -s res://tests/construction_batch.gd -- --mute
godot --headless --path godot -s res://tests/city_information.gd -- --mute
godot --headless --path godot -s res://tests/language.gd -- --mute
godot --headless --path godot -s res://tests/regular_growth.gd -- --mute
godot --headless --path godot -s res://tests/hud.gd -- --mute
godot --headless --path godot -s res://tests/acceptance.gd -- --mute
godot --headless --path godot -s res://tests/outcomes.gd -- --mute
godot --headless --path godot -s res://tests/history.gd -- --mute
godot --headless --path godot -s res://tests/audio.gd
godot --headless --path godot -s res://tests/settings.gd
godot --path godot -s res://tests/benchmark.gd
```

Der Benchmark lädt „Rocket 98“, startet die Simulation und dreht die Kamera bei 1920 × 1080 Pixeln. Nach je 100 Aufwärmbildern werden 400 Bilder der Nahansicht und 400 der Stadtübersicht gemessen. Der GPU-Test verlangt höchstens 33,33 ms am 95. Perzentil in beiden Ansichten. Für die beiden festen Marktansichten den Ordner für die Aufnahmen über `LINCITY_CAPTURE_DIR` setzen und `res://tests/riverside_views.gd` mit einem Grafiktreiber starten.

## Aufnahmen aus dem Linux-Export

Beide Aufnahmen zeigen denselben Spielstand während einer Arbeitsschicht der Leichtindustrie. Die Marktstraße führt zur Halle mit den drei grünen Markisen. Von der Brücke aus sind ihre Seitenfassade, der Fabrikhof und die Spiegelung des Schornsteins sichtbar. Die grüne Kontur zeigt das mit dem Werkzeug „Untersuchen“ anvisierte Feld.

![Marktstraße im laufenden Linux-Spiel](screenshots/godot/marktstrasse.webp)

![Blick von der Flussbrücke auf Markt und Fabrik](screenshots/godot/flussbruecke.webp)

## Prüfung der Rust-GDExtension

Bei der Umstellung in Commit `7071ed51` bestanden 105 Rust-Tests und 15 Godot-Prüfungen. Für den Vergleich mit der vorherigen Anbindung wurden alle 24 Methoden von `CityWorld` mit sieben Spielständen auf Deutsch und Englisch erfasst. Die 1.144 Vergleichswerte stimmten überein. Beide Versionen konnten die Spielstände der jeweils anderen laden.

Der Linux-Export bestand den Pakettest mit OpenGL und die Nachhaltigkeitskampagne mit 181 Bauaktionen. Unter Wine bestand das Windows-Paket den Pakettest und setzte die Stadt in einem zweiten Prozess fort. Beim macOS-ARM64-Paket wurden Architektur, Ladepfade und Ad-hoc-Signatur geprüft. Der Start auf Apple Silicon steht aus. [Commit der Umstellung](https://github.com/mpwg/lincity-ng/commit/7071ed51fdd579d1b1006fceb938e3e18eecd338)

## Grafikmessungen vor der Portierung vom 19. November 2026

Das separat entpackte Linux-Release wurde auf einer GeForce RTX 3060 Ti mit Treiber 620.70.06 geprüft. Die folgende Messung verwendet das Szenario „Rocket 98“ mit 100 × 100 Feldern, laufender Rust-Simulation und vollständigem HUD. Jede Stichprobe wartet auf das gezeichnete Bild einschließlich Rücklesen des Framebuffers.

| Renderer | Nahansicht Median | Nahansicht P95 | Stadtübersicht Median | Stadtübersicht P95 |
|---|---:|---:|---:|---:|
| RTX 3060 Ti, gepacktes Release | 6,59 ms | 12,26 ms | 7,64 ms | 12,67 ms |
| Mesa llvmpipe, Ryzen 5 5600, Editorlauf | 606,89 ms | 676,64 ms | 355,24 ms | 370,64 ms |

Das Release erreicht auf der RTX das Ziel von 33,33 ms in beiden Ansichten. Auf dem Testrechner steht eine physische GPU zur Verfügung. Der zusätzliche Softwarelauf mit Mesa 26.2.6 auf der CPU überschreitet das Ziel. Die Leistung auf einer schwächeren physischen GPU ist offen.

Die Katalogprüfung erfasst 185 Varianten mit drei Detailstufen und separaten Auswahlkörpern für anwählbare Modelle. Die Spielprüfung deckt den Markt aus vier Kamerarichtungen, Straßenbau und Abriss über eine Bereichsgrenze, Kosten, Kartenebenen, Rauch bei laufender Produktion, Raketenstart und das Weiterspielen eines gespeicherten Stands ab. Der entpackte Export wurde zusätzlich mit echten Mauseingaben, allen Lizenzansichten und einem regulären Grafikstart aus einem frischen Verzeichnis geprüft.

## Prüfung der erweiterten Oberfläche vor der Portierung

Im separat entpackten Linux-Paket wurden alle sieben Startstädte und die Kartenerzeugung in allen vier Klimazonen geprüft. Die Bedienprüfung umfasst Handel, normale und schnelle Spielstände sowie die Kostenbestätigung beim Abriss. Die Einstellungen wurden unter X11 und Wayland getestet. Für den Verlauf wurden gemessene Daten, echte Nullwerte, Lücken aus Altspielständen und ein erneuter Speicherzyklus geprüft.

Die beiden Siege und die Niederlage wurden mit je einem vorbereiteten Spielstand über den tatsächlichen Simulationsschritt und den Spielausgangsdialog geprüft. Der aktualisierte Linux-Export erreicht in „Rocket 98“ auf der RTX 3060 Ti bei 1920 × 1080 ein P95 von 11,16 ms in der Nahansicht und 13,28 ms in der Stadtübersicht.

Die Fassung mit Hauptmenü wurde im entpackten Linux-Paket mit OpenGL geprüft. Der Pakettest wählt das Flussviertel im Menü, untersucht den Markt aus vier Richtungen, zieht eine Straße, prüft Vorschau und Abbuchung, speichert und setzt die Stadt in einem neuen Prozess fort. Ein zusätzlicher X11-Test lädt einen klassischen Spielstand über den Ladedialog. Der Wachstumstest beginnt in einer neu erzeugten Landschaft, baut ein Denkmal und schaltet eine Schule frei. Er prüft die Warenversorgung der Schule und dass sie tatsächlich forscht. Die getrennten Spielausgangstests decken beide Siege und die Niederlage ab.

Im damaligen Editorlauf mit der RTX 3060 Ti und laufender Stadt in „Rocket 98“ betrug das P95 bei 1920 × 1080 17,41 ms in der Nahansicht und 19,34 ms in der Stadtübersicht.

## Vollständige Nachhaltigkeitskampagne

Die Kampagne beginnt in einem neu erzeugten 100 × 100 großen Sumpfgebiet mit Startdorf und dem Startwert 48. Mit 181 regulären Bauaktionen erreicht der Rust-Kern den Nachhaltigkeitssieg an Tag 297.600. Beim Sieg hat die Stadt 6.832 Einwohner, keine Hungernden und 19.435.272 $ in der Kasse.

Der Godot-Test öffnet den unveränderten Startstand über das Hauptmenü und spielt dieselben Bauaktionen über Bauleiste und Zeigereingaben ab. Er verwendet den regulären Simulationsschritt und überspringt das Zeichnen der Zwischenbilder. Im Linux-Export mit Rust-GDExtension erschien der Nachhaltigkeitssieg an Tag 298.800 bei 6.733 Einwohnern. Nach dem Schließen des Dialogs lief die Stadt weiter. Ein neuer Zufallsverlauf nach dem Laden kann den Siegtermin verschieben.

```sh
RUSTCITY_CAMPAIGN_ARTIFACTS="$PWD/build/sustainable-campaign" cargo test --locked --release -p rustcity-core --test sustainable_campaign -- --ignored --nocapture
godot --path godot -s res://tests/sustainable_campaign.gd -- --mute "$PWD/build/sustainable-campaign/start.gz" "$PWD/rustcity-core/tests/fixtures/sustainable_campaign.tsv"
```

## Apple-Silicon-Testbuild

Das Paketierskript erzeugt `LinCity-NG.app` für macOS 15 oder neuer auf Apple Silicon. Die App lässt sich aus dem Paketordner öffnen und ist ad hoc signiert. Falls Gatekeeper den ersten Start blockiert, kann die App unter „Systemeinstellungen → Datenschutz & Sicherheit“ über „Dennoch öffnen“ freigegeben werden.

Die Linux-Prüfung umfasst Architektur, Ladepfade, Signatur und Archivintegrität. Der Bedienlauf auf Apple Silicon steht aus.

Der Cross-Build auf Linux verwendet Clang, LLD, das macOS-15.5-SDK und Rusts Ziel `aarch64-apple-darwin`. Baue zuvor mit `cargo godot-build` die Erweiterung für den Editor auf dem Build-Rechner.

```sh
rustup target add aarch64-apple-darwin
env SDKROOT=/pfad/MacOSX15.5.sdk MACOSX_DEPLOYMENT_TARGET=15.0 \
  CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=clang \
  CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS="-C link-arg=--target=arm64-apple-macos15.0 -C link-arg=-fuse-ld=lld" \
  cargo godot-build --export --target aarch64-apple-darwin
python3 tooling/release/package_godot_macos.py --godot /pfad/godot \
  --template /pfad/macos.zip --output build/godot-macos-final
```

[Das Paketierskript](../tooling/release/package_godot_macos.py) benötigt Godot 4.5.1, dessen `macos.zip`-Vorlage, LLVM-Werkzeuge und rcodesign. Es extrahiert ARM64 aus der offiziellen Universalvorlage, prüft die Ladepfade und signiert die fertige App.

## Windows-x64-Testbuild

Das ZIP für Windows 10 und 11 auf x64 vollständig entpacken und `LinCity-NG.exe` öffnen. PCK und DLLs bleiben neben der EXE. Die EXE hat keine Authenticode-Signatur.

Das entpackte Paket wurde unter Wine auf Hauptmenü, Fortsetzen, Bauvorschau, HUD und Spielstand geprüft. Der Bedienlauf auf einem nativen Windows-System steht aus.

Für den Cross-Build auf Linux LLVM-MinGW mit UCRT und das Rust-Ziel `x86_64-pc-windows-gnullvm` verwenden. Baue zuvor mit `cargo godot-build` die Erweiterung für den Editor auf dem Build-Rechner.

```sh
rustup target add x86_64-pc-windows-gnullvm
env CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER=/pfad/llvm-mingw/bin/x86_64-w64-mingw32-clang \
  cargo godot-build --export --target x86_64-pc-windows-gnullvm
python3 tooling/release/package_godot_windows.py --godot /pfad/godot \
  --output build/LinCity-NG-Windows-x64.zip \
  --dll-path /pfad/llvm-mingw/x86_64-w64-mingw32/bin --license /pfad/llvm-mingw/LICENSE.TXT
```

[Das Paketierskript](../tooling/release/package_godot_windows.py) verwendet Godot 4.5.1 mit der installierten Windows-x64-Exportvorlage und `llvm-objdump`. Der LLVM-MinGW-Build benötigt `libunwind.dll`. Das Skript übernimmt diese DLL aus `--dll-path` und legt den Lizenztext aus `--license` bei.
