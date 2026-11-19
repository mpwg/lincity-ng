# Umsetzung der 3D-Stadt

Die Marktstraße und der Blick von der Flussbrücke zeigen dasselbe spielbare Viertel. Der Rust-Kern liest und schreibt bestehende Spielstände und entscheidet über alle Baubefehle. Das Godot-Frontend erhält Kartenraster, Grundstücksanker, Kartenebenen und Betriebszustand über die native Erweiterung. [Freigegebenes Konzept](../specs/2026-11-19-godot-desktop-design.md)

1. Das Flussviertel durch reguläre Landschafts- und Bauaktionen aus „Good Times“ herstellen. Beide Blickrichtungen als Kamerapositionen festlegen.
2. Markthalle, Wohngebäude, Fabriken und Straßenfahrzeuge in Blender ausarbeiten. Weitere Gelände-, Gebäude- und Anschlussvarianten reproduzierbar als GLB erzeugen. Maßstab, Auswahlkörper und drei Detailstufen im Katalog prüfen.
3. Gebäude und Ausstattung nach Kartenfeldern gruppieren, Fassaden zur Straße drehen, Anschlussänderungen über Bereichsgrenzen verfolgen. Rust liefert Betrieb und Fahrbewegungen.
4. Bauleiste, Zeitsteuerung, Minikarte, Gebäudeauskunft, Handel, Hilfe und Spielstanddialoge in der Stadtansicht integrieren. Kamera, Tastaturfokus und Eingaben bei geöffneten Dialogen prüfen.
5. Den Referenzmarkt aus vier Richtungen auswählen, eine Straße über die Bereichsgrenze bauen und abreißen, Kosten und Betriebswechsel prüfen. Das Viertel laden, weiterspielen, speichern und erneut öffnen.
6. Markt und Flussbrücke im Spiel bei 1920 × 1080 vergleichen. „Rocket 98“ bei laufender Simulation mit 100 Aufwärmbildern und je 400 Bildern in Nah- und Fernsicht messen. Das Linux-Paket mit vollständigen Lizenztexten in einem frischen Verzeichnis starten und prüfen.
