# Parität von RustCity Core

Vergleichsgrundlage ist LinCity Core bei `d4346dd34fafea3384360b593fa1d59ffa2b6324`.
Die Spielstände des C++-Kerns, das Rust-Programm und der historische
Desktop-Vergleich haben jeweils eigene Einstiegspunkte.
[Referenzstand](rustcity-core-baseline.md) · [Replay-Vertrag](../lincity-core/parity/PARITY-CONTRACT.md)

## Prüfung je Funktionsbereich

| Bereich | Überprüfter Umfang und Testpfad |
| --- | --- |
| Alle 43 Gebäudegruppen | Die initialisierte C++-Registry liefert IDs, Metadaten und Warenregeln je Gruppe. Der Rust-Test vergleicht das Registry-Transkript. [Katalog](../rustcity-core/src/building_kind.rs) |
| Waren, Inventare, Konten, Perioden und Historien | Referenzfälle für zehn Warennamen, Kapazitäten und Promillewerte, getrennte Monats- und Jahresabschlüsse sowie historische Reihen. [Regelfälle](../lincity-core/parity/README.md) · [Statistiktests](../rustcity-core/src/stats.rs) |
| Kartenfelder, Bau, Fluten, Abriss, Evakuieren | Randkoordinaten, Belegung des gesamten Grundrisses, Kosten, Kreditgrenze, Kontentrennung und ungültig gewordene Gebäude-IDs. Die Referenzschmiede übt alle fünf Aktionsarten aus. Zwölf C++/Rust-Fälle prüfen zusätzlich Flussverbindungen, gespeicherte Evakuierungssperren, Bauvoraussetzungen, Verbrauchssteuern und Haushaltsgrenzen bis zu den beiden Extremwerten von 32-Bit-Ganzzahlen. [Kreuztest](../lincity-core/tests/replay/check_continuation.py) [Aktionstests](../rustcity-core/tests/terrain_actions.rs) · [Gebäudeidentität](../rustcity-core/src/buildings.rs) |
| Solarkraft, Umspannwerk, Stromleitungen, Kohlekraft und beide Windkraftwerke | Produktion, Spannungsumwandlung, Leitungskosten, Handel vor Aktualisierung, Abriss eines Nachbarn, Windkosten und Rauchentwicklung werden in einzelnen Fällen geprüft. [Referenzkette](../rustcity-core/src/energy.rs) · [Kraftwerke](../rustcity-core/src/power.rs) · [Netztest](../rustcity-core/tests/transport.rs) |
| Schmiede, Mühle, Töpferei und Biobauernhof | Produktion und Monatsgrenzen, Wahl des Brennstoffs, Arbeitsunterbrechung, Bewässerung sowie Zufallsauswahl bei der Platzierung. Für die Schmiede besteht ein Vergleich mit C++ über fünf Ticks samt Aktionen und Spielständen. [Werkstätten](../rustcity-core/src/workshop.rs) · [Farm](../rustcity-core/tests/farm.rs) · [CLI-Test](../rustcity-core/tests/replay_cli.rs) |
| Kohle- und Erzbergwerk, Leicht- und Schwerindustrie, Recycling und Deponie | Die C++-Referenz verzeichnet Tage, Bargeld, Umweltzustand, Reserven, Evakuierung, Vorräte und Produktionszähler. Weitere Tests betreffen Stilllegung, verbleibende Erzvorkommen und Abrisssperren. [Referenzprogramm](../lincity-core/parity/production_reference.cpp) · [Fälle](../rustcity-core/src/production/cases/) · [Integrationstests](../rustcity-core/tests/industry_actions.rs) |
| Sechs Wohngebiete, Baracke, Schule, Universität, Feuerwehr, Gesundheitsdienst, Sportplatz, Denkmal, Brunnen und beide Parktypen | Die Gruppen teilen sich Zustands- und Aktualisierungsregeln. Gezielte Tests betreffen versorgte Wohnhäuser, getrennte Bildungsbudgets und die Warenentnahme der Baracke. [Regeln und Tests](../rustcity-core/src/city/civic/) · [Gruppenzuordnung](../rustcity-core/src/city/civic.rs) |
| Markt, Hafen, Steuern und Stadthaushalt | Kauf- und Verkaufsregeln, Handel mit entfernten Partnern, Importabbuchung, Exportreste und Reihenfolge des Jahresabschlusses. [Finanztests](../rustcity-core/tests/finance.rs) |
| Wege, Straßen, Schienen und drei Brückentypen | Gruppenwahl beim Bau auf Wasser, Kosten, Netzanbindung und Warenaustausch. Ein Straßennetz mit 96 Feldern prüft Pendlerfahrten unter allen drei Verkehrsmodi. Frame-Referenzen decken Kurven, Brücken und Bahnübergänge ab. [Aktionsdaten](../lincity-core/tests/replay/state-v1.tsv) · [Transporttests](../rustcity-core/tests/transport.rs) · [Fahrzeugreferenz](../lincity-core/parity/transport_reference.cpp) |
| Entscheidungen und Besuchsreihenfolge | Separate Zufallsquellen je Stadt, native Aufnahme und Wiedergabe, Rohwert vor C-Modulo, benannte Verteilungen, Platzierungs-IDs und A-/D-/E-Reihenfolgen. Tests manipulieren IDs, Zeit, Anzahl, Ereignistyp, Wahlausgang und Dateiende. [Tape-Tests](../rustcity-core/tests/decision_tape.rs) |
| Umwelt, Ereignisse und Stadtinstanzen | Nachwachsen freiliegender Wüste, verschmutzte Rand- und Innenfelder, Warteschlange beim Übergang in Schulden, Spielfortsetzung und unabhängige Karten. [Umwelttests](../rustcity-core/tests/environment.rs) · [Ökologiefall](../rustcity-core/tests/farm.rs) |
| Format 2130, Format 2140 und Fortsetzung | Migration alter Konten, Historien, Gebäude, XML und gzip. Schmiede (2140) und Beach (2130) werden je fünf Ticks bis zur Übergabe gespielt. Beide Kerne speichern und spielen die Zwischenstände des jeweils anderen Kerns sieben weitere Ticks. [Persistenztests](../rustcity-core/tests/persistence.rs) · [Mehrtägiger Kreuztest](../lincity-core/tests/replay/check_continuation.py) |
| Generierte Städte | Für Fluss-, Wüsten-, gemäßigte und Sumpfkarten laufen Kombinationen mit und ohne Wald sowie mit und ohne Dorf in 50×50 und 100×100. Vier weitere Fälle prüfen die Mindestgrößen 13 und 26 sowie zwei Küsten bei Größe 50. C++ und Rust nahmen abwechselnd auf und spielten das jeweils andere Tape ab. Alle 72 daraus gespeicherten Kartenpaare stimmten einschließlich Gelände, Höhen, Wasserständen, Rohstoffreserven und Dorfgebäuden überein. [Kreuztest](../rustcity-core/tests/world_generation.rs) · [C++-Referenzprogramm](../lincity-core/parity/generation_reference.cpp) |
| Kommune, Feuer und Raketenrampe | Gezielte Gebäude- und Replaytests ergänzen die Szenarien. Die Langzeitläufe prüfen auch Wechselwirkungen mit Handel, Bevölkerung und Umwelt. [Kommunen](../rustcity-core/src/city/commune.rs) · [Feuer](../rustcity-core/src/city/civic.rs) · [Raketenstarts](../rustcity-core/tests/rocket.rs) |
| Historischer SDL-Desktop | Zwei kompilierte Backends mit getrennten Startkennungen, reale Maus- und Tastatureingaben, Bildprüfung und Fortsetzung der Spielstände in beiden Richtungen. [GUI-Test](../tooling/regression/desktop_replay.py) |

## Eigenständige Quellbäume

Der Rust-Core enthält die zehn Referenzdateien für seine regulären Tests in
`tests/fixtures`. Sie stammen unverändert aus den C++-Szenarien, den XML-Tests
und den versionierten Replay- und Paritätsfällen. Damit lassen sich die
Bibliothek, der Replay-Runner und ihre Tests aus einer allein kopierten Crate
bauen. Die Kopie des Abschlussstands bestand die Format-, Typ-, Dokumentations-
und Clippy-Prüfung sowie je 95 Tests in Debug und Release. Die vier expliziten
Referenztests bestanden separat mit den C++-Programmen und dem Vergleichsprüfer
aus dem gemeinsamen Repository. Sie decken das Kreuzladen, den Langzeitlauf des
Wohn- und Versorgungsviertels, CLI-Diagnosen und die Landschaftserzeugung ab.
[Fixtures](../rustcity-core/tests/fixtures/)

Für die Prüfung einer Kopie mit Rust und Cargo 1.99.0:

```sh
cargo fmt --check
cargo check --locked --all-targets
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --release
```

## Szenariovergleich

Das C++-Programm zeichnet die Entscheidungen auf. Rust lädt dieselbe Stadt,
führt dieselben sechs Aktionen aus und konsumiert das Tape. Der Prüfer vergleicht
alle protokollierten Tagesfelder und Meldungen sowie die vollständigen,
normalisierten XML-Spielstände an den relativen Tagen 0, 1, 100 und 1201.
Jedes Szenario und Verkehrspaar startet eigene Prozesse und temporäre Dateien.
Der erste Unterschied eines Paares beendet dessen Prüfung.

Vom Repository-Stamm aus mit einem konfigurierten Standalone-Build in
`build/core` ausführen:

```sh
cmake --build build/core --target core_replay
cargo build --manifest-path rustcity-core/Cargo.toml --release --bin rustcity-replay
python3 -B lincity-core/tests/replay/check.py build/core/tests/core_replay differential \
  --candidate rustcity-core/target/release/rustcity-replay --keep-going
```

### Messung am 18. November 2026

Alle zwölf Kombinationen stimmten bis einschließlich Tag 1201 mit der
C++-Referenz überein. Geprüft wurden **14.424 Tageszustände und 48 vollständige
XML-Spielstände**. Jede Tabellenzelle gibt die Anzahl übereinstimmender
Tageszustände und Spielstände an.

| Szenario | Ohne Verkehr | Mit Verkehr |
| --- | --- | --- |
| Beach | 1.202 / 4 | 1.202 / 4 |
| Rocket_98 | 1.202 / 4 | 1.202 / 4 |
| bad_times | 1.202 / 4 | 1.202 / 4 |
| extreme_arid | 1.202 / 4 | 1.202 / 4 |
| extreme_wetland | 1.202 / 4 | 1.202 / 4 |
| good_times | 1.202 / 4 | 1.202 / 4 |

Die Abschlussmessung verwendete die Quellen von
`fe8008e6570428589cd39465f63da0b2b50b0da7`. Das Rust-Programm wurde
aus der allein kopierten Crate als Linux-Release-Binary gebaut. Seine
SHA-256-Prüfsumme war
`ddb7d3bbeaae774ed71c63c134f47e9a354d998827ac8d3f3427c935947375d8`.
Das C++-Referenzprogramm hatte die SHA-256-Prüfsumme
`5d529ecc96056d61ab92922aff48312b96f75e3825c8f8f350ca0489ed7957bf`.

Für die Schmiede in Format 2140 bestand außerdem der Vergleich in beiden
Richtungen. Rust spielte das versionierte C++-Tape ab. Anschließend zeichnete
Rust ein neues Tape auf, das der C++-Runner abspielte. Beide Vergleiche
bestätigten sechs Tageszustände und die drei vollständigen Spielstände an
den Tagen 0, 1 und 5.

## C++-Abnahme nach der Entkopplung des Desktops

Die allein kopierte C++-Bibliothek mit `Map::discardConstruction` bestand alle
30 Debug-CTests ohne benachbarten Rust-Quellbaum. Mit beiden Quellbäumen
bestanden unter GCC 16.2.1 im Debug- und Release-Build jeweils 43 CTests.
Clang 23.1.5 bestand alle 43 CTests mit AddressSanitizer, UndefinedBehaviorSanitizer
und aktivierter Leckerkennung.
Der erneute Vergleich gegen den historischen Desktop-Commit
`21796de530d68b30881c6b0f6a6a9d6bbfb1e31a` bestätigte erneut die vier
Spielstände sowie die Meldungsfolge für jedes der zwölf Szenario-/Verkehrspaare.
[Referenzstand und Befehle](rustcity-core-baseline.md)

Die Seeerzeugung zieht zuerst die y- und danach die x-Koordinate. Diese Reihenfolge
stand früher implizit in zwei C++-Funktionsargumenten. GCC und Clang werteten
sie unterschiedlich aus. Die explizite Reihenfolge erhält die historischen
GCC-Ziehungen und ermöglicht denselben Referenztest unter Clang.
[Referenzgenerator](../lincity-core/lincity/init_game.cpp)

## Weltgenerierung wiederholen

Der Generator benötigt die C++-Referenz nur für den Kreuztest. Die öffentliche
Rust-API bietet `City::generate` für die native Auswahl und
`City::generate_with_decisions` für einen gesetzten Seed, Aufnahme oder Wiedergabe.

```sh
cmake --build build/core --target core_generation_reference
RUSTCITY_CPP_GENERATOR="$PWD/build/core/parity/core_generation_reference" \
  cargo test --manifest-path rustcity-core/Cargo.toml --release --test world_generation -- --include-ignored
```

## Spielstandübergabe und Verkehr

Der Mehrtages-Kreuztest führt die sechs versionierten Aktionen für die Schmiede
und Beach aus. Er vergleicht das Aufwärmen und prüft anschließend dieselbe
gespeicherte Stadt aus jeder Schreibimplementierung mit dem jeweils anderen
Reader. Alle vier Fortsetzungen über sieben Ticks stimmten an acht Tagen und
an den drei Speicherpunkten der Fortsetzung überein.

Für den Verkehrstest starten 96 verbundene Straßenfelder mit wechselnden
Arbeitskräftevorräten. Über 20 Ticks stimmten je Profil 21 Tageszustände und
drei vollständige Spielstände überein. Im aufgezeichneten Lauf erschienen
0 Richtungsentscheidungen ohne Verkehr, 19 mit reduziertem und 30 mit normalem
Verkehr. Alle aufgezeichneten Richtungen wurden im Rust-Kern validiert.

Wenn beide Quellbäume, Cargo und Python vorhanden sind, führt der reguläre
CTest-Lauf die Kreuztests aus. Für einen gezielten Lauf nach einem CMake-Build:

```sh
ctest --test-dir build/core -R '^rustcity[.]cross[.]' --output-on-failure
```

## Lebensdauer am Desktop-Adapter

Alle sechs Rust-Adaptertests bestanden. Zwei davon liefen zusätzlich unter Miri.
Einer gibt zuerst die Stadt und danach zwei unabhängige Kartenansichten frei.
Der andere speichert eine Stadt, gibt ihren Handle frei und lädt die Datei in
einen neuen Handle. Die beiden C++-Adaptertests prüfen Aufrufe durch die echte
Sprachgrenze, einschließlich Gebäudeabriss und Darstellung. Sie bestanden
auch je vier Durchläufe mit AddressSanitizer, LeakSanitizer und UBSan.
Der integrierte Desktop-Build bestand alle 45 CTests.

## SDL-Desktop und wechselseitige Spielstandfortsetzung

Der GUI-Test startet die installierte Oberfläche für jeden Lauf auf einem
isolierten X-Server mit eigener Konfiguration und eigenen Spielständen.
Er kontrolliert den Backendnamen im Startprotokoll. Bildschirmfotos und
Texterkennung belegen den Raketenbereitschaftsdialog, den Bericht des Marktes
und seine Handelssteuerung. Bildausschnitte prüfen die Feueranimation und die
Umschaltung der Minikarte auf Luftverschmutzung.

Am 18. November 2026 bestanden beide Desktops die Prüfung. Bis zur ersten Pause
erhielt der Markt im C++-Lauf 456 und im Rust-Lauf 224 Einheiten Lebensmittel
vom angrenzenden Weg. Insgesamt blieben jeweils 760 Einheiten erhalten. Anschließend wurde
der Einkauf per Mausklick deaktiviert, ein Weg gebaut, abgerissen und durch
Schnellladen wiederhergestellt. Die Spielstände belegten außerdem Stillstand
bei Pause und unterschiedliche Tagesfortschritte bei normalem und schnellem
Tempo. Rust setzte die im C++-Desktop gespeicherte Stadt von Tag 65 bis 81
fort, C++ die im Rust-Desktop gespeicherte Stadt von Tag 70 bis 86. In
beiden Fortsetzungen wurde der übernommene Weg abgerissen und der Markt für
Lebensmitteleinkäufe wieder geöffnet. Die neu gebauten Programme hatten folgende
SHA-256-Prüfsummen.

| Programm | SHA-256 |
| --- | --- |
| LinCity Core (C++) | `37f1a7b3f9dc3b054edbfecb3a9164f9d94492a7dc651c681151e254a579d8e8` |
| RustCity Core | `5f17b104b54096a697d7ff5a4b55a0bc2c1e057040682e6b07dda4fb5ad33a5d` |

Der Test benötigt Python mit Pillow/XCB, Xvfb, xvfb-run, xauth, xdotool und
Tesseract mit englischen oder deutschen Sprachdaten. Das Zielverzeichnis
muss neu sein. Für projektlokale X11-Werkzeuge akzeptiert der Test zusätzlich
`--tools-prefix PFAD` mit den Unterverzeichnissen `bin` und `lib`.

```sh
python3 -B tooling/regression/desktop_replay.py \
  --cpp build/desktop/bin/lincity-ng \
  --rust build/rustcity-desktop/bin/lincity-ng \
  --assets build/desktop/share/lincity-ng \
  --artifacts build/desktop-regression
```

Jeder der vier Läufe hinterlegt Screenshots, gzip-Spielstände, das
Desktop-Protokoll und eine `results.json` mit SHA-256 des Programms und den
beobachteten Werten. [GUI-Test](../tooling/regression/desktop_replay.py)
