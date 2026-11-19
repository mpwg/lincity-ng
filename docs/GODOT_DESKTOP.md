# LinCity-NG in 3D spielen

Das Spiel startet am Markt des Flussviertels. Die Stadt lässt sich aus jedem Winkel betrachten und bearbeiten. Godot 4.5.1 zeichnet die Stadt mit OpenGL. RustCity verwaltet Simulation, Baukosten, Handel und Spielstände. Die Auswahl des SDL-Backends gilt nur für die klassische Oberfläche.

## Start aus den Quellen

Für den Linux-Build werden ein C++20-Compiler für die Godot-Anbindung, CMake, Cargo und die Kernbibliotheken des Projekts benötigt. CMake verwendet die im Projekt festgelegte Version von godot-cpp. Die vorbereiteten GLB-Dateien sind im Repository enthalten. Zum Spielen genügt der Godot-Editor mit der passenden Erweiterung. [Abhängigkeiten](DEPENDENCIES.md)

```sh
cmake -S . -B build/godot-editor -DLINCITY_BUILD_GUI=OFF -DLINCITY_BUILD_GODOT=ON -DBUILD_TESTING=OFF -DCMAKE_BUILD_TYPE=Release
cmake --build build/godot-editor --target lincity_godot --parallel
godot --headless --path godot --import
godot --path godot
```

Mit `godot --path godot -- --bridge-view` öffnet sich dieselbe Stadt von der Flussseite. Mit `--city=/absoluter/pfad/stadt.gz` hinter dem doppelten Bindestrich lädt Godot einen anderen Spielstand.

| Aktion | Bedienung |
|---|---|
| Blick drehen und neigen | Rechte Maustaste ziehen, Q und E zum Drehen |
| Verschieben und zoomen | Mittlere Maustaste ziehen, WASD oder Pfeiltasten, Mausrad zum Zoomen |
| Untersuchen, Straße, Wohnen, Markt, Abriss | Tasten 1 bis 5 oder Bauleiste |
| Weitere Werkzeuge und Kartenebenen | B für den Baukatalog, Auswahl unter der Minikarte für die Kartenebene |
| Verkehrsweg bauen | Linke Maustaste ziehen, Strg wechselt den Knick im Verlauf |
| Hilfe und Spielstände | F1 für die Hilfe, Menü „Spiel“ zum Speichern und Laden |

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
cmake -S . -B build/godot-template-release -DLINCITY_BUILD_GUI=OFF -DLINCITY_BUILD_GODOT=ON -DBUILD_TESTING=OFF -DCMAKE_BUILD_TYPE=Release -DGODOTCPP_TARGET=template_release
cmake --build build/godot-template-release --target lincity_godot --parallel
python3 tooling/release/package_godot_linux.py --godot godot --libxmlpp-source /pfad/zum/libxml++-5.4.0.tar.xz --output build/godot-linux-final --graphics
```

Das Quellarchiv muss zur eingebundenen libxml++-Version passen. Die Paketierung verwendet Python, patchelf und ldd. Sie prüft Spielstart, PCK, Bibliotheken, Credits und Spielstand aus einem frischen Arbeitsverzeichnis. Die Datei `LIESMICH.txt` im Paket nennt die erforderlichen Linux-Systembibliotheken.

## Prüfung

```sh
godot --headless --path godot -s res://tests/model_catalog.gd
godot --headless --path godot -s res://tests/hud.gd
godot --headless --path godot -s res://tests/acceptance.gd
godot --path godot -s res://tests/benchmark.gd
```

Der Benchmark lädt „Rocket 98“, startet die Simulation und dreht die Kamera bei 1920 × 1080 Pixeln. Nach je 100 Aufwärmbildern werden 400 Bilder der Nahansicht und 400 der Stadtübersicht gemessen. Der GPU-Test verlangt höchstens 33,33 ms am 95. Perzentil in beiden Ansichten. Für die beiden festen Marktansichten den Ordner für die Aufnahmen über `LINCITY_CAPTURE_DIR` setzen und `res://tests/riverside_views.gd` mit einem Grafiktreiber starten.

## Aufnahmen aus dem Linux-Export

Beide Aufnahmen zeigen denselben Spielstand während einer Arbeitsschicht der Leichtindustrie. Die Marktstraße führt zur Halle mit den drei grünen Markisen. Von der Brücke aus sind ihre Seitenfassade, der Fabrikhof und die Spiegelung des Schornsteins sichtbar. Die grüne Kontur zeigt das mit dem Werkzeug „Untersuchen“ anvisierte Feld.

![Marktstraße im laufenden Linux-Spiel](screenshots/godot/marktstrasse.webp)

![Blick von der Flussbrücke auf Markt und Fabrik](screenshots/godot/flussbruecke.webp)
