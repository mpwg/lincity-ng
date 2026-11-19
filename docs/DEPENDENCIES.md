# Abhängigkeiten

Zum Bauen und Spielen aus den Quellen brauchst du die folgenden Programme und Bibliotheken.

- Godot 4.5.1 mit dem OpenGL-Kompatibilitätsrenderer zum Importieren und Starten des Projekts.
- CMake ab 3.21, Git, Python 3 und ein C++20-Compiler für die native Godot-Erweiterung. CMake lädt die festgelegte Version von godot-cpp beim ersten Konfigurieren.
- Rust und Cargo mit Unterstützung für die Edition 2024.
- fmt ab 9.0 mit den Entwicklungsdateien der Zielplattform.

Für Exporte kommen die zur Godot-Version passenden Exportvorlagen und plattformspezifische Werkzeuge hinzu. [Spielstart und Exporte](GODOT_DESKTOP.md)

Der Rust-Kern lässt sich separat mit Rust und Cargo bauen und testen.
