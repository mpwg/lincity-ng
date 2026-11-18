# RustCity Core

RustCity Core entsteht als eigenständige Rust-Bibliothek neben dem C++-Core.
Referenz ist d4346dd34fafea3384360b593fa1d59ffa2b6324. Fachliche Unterschiede
werden gegen den [bestehenden Vertrag](../../../lincity-core/parity/PARITY-CONTRACT.md)
geprüft. Die historische Desktop-Referenz dient weiterhin ihrem eigenen Vergleich.

## Reihenfolge und Abnahme

1. C++ frisch in Debug und Release bauen, CTest und Sanitizer ausführen und
   die sechs Startszenarien mit der historischen Referenz vergleichen.
2. Waren, Inventare, Finanzen, Perioden und Historien gegen die versionierten
   Referenzfälle prüfen. Jede Portierung beginnt mit einem fehlschlagenden Test.
3. Kartenfelder, Gebäudeidentitäten, Belegung und Aktionsvorschau portieren.
   Abriss und Neubau müssen alte Verbindungen und Identitäten entwerten.
   Eine erste durchgängige Kette umfasst Solarkraftwerk und Umspannwerk.
4. Alle registrierten Gebäudegruppen, Transport, Fahrzeuge, Dienste, Umwelt,
   Weltgenerierung, Kalender, Ereignisse und Animation portieren. Lade- und
   Speicherpfade mit jedem Abschnitt mitführen. Die öffentliche Rust-API bietet
   instanzgebundene Städte, Vorschau, Aktionen, Tick, Animation, Kartenausschnitt,
   Berichte und geordnete Ereignisse. Zwei Städte müssen unabhängig laufen.
5. Sechs Szenarien mit und ohne Verkehr für jeweils 1.201 Tage vergleichen:
   14.424 Tageszustände mit Tag null und 48 vollständige Spielstandvergleiche.
   Zusätzlich generierte Städte, alle Verkehrsmodi, native Zufallsauswahl,
   Versionen 2130 und 2140 und beidseitige Spielstandfortsetzung prüfen.
   Jede Funktion erhält einen Nachweis. Alle bisherigen Tests bleiben ausführbar.
6. Erst nach der Headless-Abnahme die alte SDL-Oberfläche anbinden.
   LINCITY_CORE_BACKEND wählt beim Bauen cpp (Standard) oder rust.
   Die FFI gehört zum entfernbaren Adapter. In beiden GUI-Builds über echte
   Eingaben bauen, abreißen, handeln, Geschwindigkeiten umschalten, Minikarte,
   Reports, Animation und Meldungen prüfen und über Kreuz weiterspielen.
7. CTest, Cargo-Tests, Clippy, Formatierung und Dokumentation prüfen, FFI mit
   Miri und einem C++-Lebenszyklustest kontrollieren. Abschließend die
   eigenständige Crate ohne Desktop- und FFI-Anbindung prüfen.

Die neue GUI selbst ist ein späteres Projekt. Die dauerhafte Konsumentenschnittstelle
ist die sichere Rust-API. Der bisherige Desktop-Build benötigt weiterhin nur C++.
