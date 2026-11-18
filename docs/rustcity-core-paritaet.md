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
| Kartenfelder, Bau, Fluten, Abriss, Evakuieren | Randkoordinaten, Belegung des gesamten Grundrisses, Kosten, Kreditgrenze, Kontentrennung und ungültig gewordene Gebäude-IDs. Die Referenzschmiede übt alle fünf Aktionsarten aus. [Aktionstests](../rustcity-core/tests/terrain_actions.rs) · [Gebäudeidentität](../rustcity-core/src/buildings.rs) |
| Solarkraft, Umspannwerk, Stromleitungen, Kohlekraft und beide Windkraftwerke | Produktion, Spannungsumwandlung, Leitungskosten, Handel vor Aktualisierung, Abriss eines Nachbarn, Windkosten und Rauchentwicklung werden in einzelnen Fällen geprüft. [Referenzkette](../rustcity-core/src/energy.rs) · [Kraftwerke](../rustcity-core/src/power.rs) · [Netztest](../rustcity-core/tests/transport.rs) |
| Schmiede, Mühle, Töpferei und Biobauernhof | Produktion und Monatsgrenzen, Wahl des Brennstoffs, Arbeitsunterbrechung, Bewässerung sowie Zufallsauswahl bei der Platzierung. Für die Schmiede besteht ein Vergleich mit C++ über fünf Ticks samt Aktionen und Spielständen. [Werkstätten](../rustcity-core/src/workshop.rs) · [Farm](../rustcity-core/tests/farm.rs) · [CLI-Test](../rustcity-core/tests/replay_cli.rs) |
| Kohle- und Erzbergwerk, Leicht- und Schwerindustrie, Recycling und Deponie | Die C++-Referenz verzeichnet Tage, Bargeld, Umweltzustand, Reserven, Evakuierung, Vorräte und Produktionszähler. Weitere Tests betreffen Stilllegung, verbleibende Erzvorkommen und Abrisssperren. [Referenzprogramm](../lincity-core/parity/production_reference.cpp) · [Fälle](../rustcity-core/src/production/cases/) · [Integrationstests](../rustcity-core/tests/industry_actions.rs) |
| Sechs Wohngebiete, Baracke, Schule, Universität, Feuerwehr, Gesundheitsdienst, Sportplatz, Denkmal, Brunnen und beide Parktypen | Die Gruppen teilen sich Zustands- und Aktualisierungsregeln. Gezielte Tests betreffen versorgte Wohnhäuser, getrennte Bildungsbudgets und die Warenentnahme der Baracke. [Regeln und Tests](../rustcity-core/src/city/civic/) · [Gruppenzuordnung](../rustcity-core/src/city/civic.rs) |
| Markt, Hafen, Steuern und Stadthaushalt | Kauf- und Verkaufsregeln, Handel mit entfernten Partnern, Importabbuchung, Exportreste und Reihenfolge des Jahresabschlusses. [Finanztests](../rustcity-core/tests/finance.rs) |
| Wege, Straßen, Schienen und drei Brückentypen | Gruppenwahl beim Bau auf Wasser, Kosten, Netzanbindung und Warenaustausch. Gesonderte Prüfung der Fahrzeugwahl, Wegfindung und Animation in belebten Städten bleibt erforderlich. [Aktionsdaten](../lincity-core/tests/replay/state-v1.tsv) · [Transporttests](../rustcity-core/tests/transport.rs) |
| Entscheidungen und Besuchsreihenfolge | Separate Zufallsquellen je Stadt, native Aufnahme und Wiedergabe, Rohwert vor C-Modulo, benannte Verteilungen, Platzierungs-IDs und A-/D-/E-Reihenfolgen. Tests manipulieren IDs, Zeit, Anzahl, Ereignistyp, Wahlausgang und Dateiende. [Tape-Tests](../rustcity-core/tests/decision_tape.rs) |
| Umwelt, Ereignisse und Stadtinstanzen | Nachwachsen freiliegender Wüste, verschmutzte Rand- und Innenfelder, Warteschlange beim Übergang in Schulden, Spielfortsetzung und unabhängige Karten. [Umwelttests](../rustcity-core/tests/environment.rs) · [Ökologiefall](../rustcity-core/tests/farm.rs) |
| Format 2130, Format 2140 und Fortsetzung | Migration alter Konten, Historien, Gebäude, XML und gzip. Ein zusätzlicher Test startet den C++-Runner und vergleicht den Folgetag aus einem Rust-Spielstand. [Persistenztests](../rustcity-core/tests/persistence.rs) · [Kreuztest](../rustcity-core/src/save/tests.rs) |
| Generierte Städte | Für Fluss-, Wüsten-, gemäßigte und Sumpfkarten laufen Kombinationen mit und ohne Wald sowie mit und ohne Dorf in 50×50 und 100×100. Vier weitere Fälle prüfen die Mindestgrößen 13 und 26 sowie zwei Küsten bei Größe 50. C++ und Rust nahmen abwechselnd auf und spielten das jeweils andere Tape ab. Alle 72 daraus gespeicherten Kartenpaare stimmten einschließlich Gelände, Höhen, Wasserständen, Rohstoffreserven und Dorfgebäuden überein. [Kreuztest](../rustcity-core/tests/world_generation.rs) · [C++-Referenzprogramm](../lincity-core/parity/generation_reference.cpp) |
| Kommune, Feuer, Raketenrampe und Verkehrsmodus „reduziert“ | Die Szenarien prüfen einzelne Zustände dieser Bereiche. Für die Gesamtabnahme sind eigenständige Verhaltensbelege sowie Läufe aller Verkehrsmodi erforderlich. |
| SDL-Anbindung, FFI und eigenständige Crate | Die Desktop-Abnahme setzt erfolgreiche Headless-Parität voraus. Der [Abnahmeplan](superpowers/plans/2026-11-18-rustcity-core.md) benennt die Tests für Build-Auswahl, Eingaben, Kreuzladen und Lebenszyklus. |

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

Geprüft wurde ein festgehaltenes Linux-Release-Binary mit SHA-256
`f4ee90a4fcba0f0664eee01b3d66dba1ea54396e073b16e0ee9f7e0cf629a924`.
Alle zwölf Kombinationen wurden gestartet. Beach erreichte Tag 1201 in beiden
Verkehrsprofilen. Insgesamt bestanden damit **2 von 12 Läufen mit 2.404
Tageszuständen und 8 vollständigen Spielständen**.

| Szenario | Ohne Verkehr | Mit Verkehr |
| --- | --- | --- |
| Beach | 1.202 Tageszustände und 4 Spielstände gleich | 1.202 Tageszustände und 4 Spielstände gleich |
| Rocket_98 | Ereignis 3880, Rust erwartet P, Tape enthält C | Gleicher Unterschied |
| bad_times | Ereignis 864, Rust erwartet P, Tape enthält C | Gleicher Unterschied |
| extreme_arid | Ereignis 42, Rust erwartet D, Tape enthält C | Gleicher Unterschied |
| extreme_wetland | Ereignis 42, Rust erwartet D, Tape enthält C | Gleicher Unterschied |
| good_times | Ereignis 856, Rust erwartet P, Tape enthält C | Gleicher Unterschied |

P steht für Platzierung, D für den täglichen Besuch und C für eine Ziehung
aus dem C-Zufallsgenerator. Bei einem Tape-Abbruch werden dem Prüfer für
dieses Szenario keine erfolgreichen Tage oder Spielstände gutgeschrieben.

Für die Schmiede in Format 2140 bestand außerdem der Vergleich in beiden
Richtungen. Rust spielte das versionierte C++-Tape ab. Anschließend zeichnete
Rust ein neues Tape auf, das der C++-Runner abspielte. Beide Vergleiche
bestätigten sechs Tageszustände und die drei vollständigen Spielstände an
den Tagen 0, 1 und 5.

## Weltgenerierung wiederholen

Der Generator benötigt die C++-Referenz nur für den Kreuztest. Die öffentliche
Rust-API bietet `City::generate` für die native Auswahl und
`City::generate_with_decisions` für einen gesetzten Seed, Aufnahme oder Wiedergabe.

```sh
cmake --build build/core --target core_generation_reference
RUSTCITY_CPP_GENERATOR="$PWD/build/core/parity/core_generation_reference" \
  cargo test --manifest-path rustcity-core/Cargo.toml --release --test world_generation -- --include-ignored
```
