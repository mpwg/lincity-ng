# Befunde zum Verhaltensvergleich mit LinCity-NG

Stand 18. November 2026. Grundlage sind der Originalquellstand `21796de5` und die 178 lokal erfassten GitHub-Issues des Originalprojekts (29 offen, 149 geschlossen). Titel und Beschreibungen wurden nach den unten aufgeführten Abweichungen durchsucht. Bei einschlägigen Treffern wurden auch Kommentare und bei #399 der angehängte cppcheck-Bericht geprüft. Der Issue-Status bezieht sich auf diesen Bestand. [Originalquellstand](https://github.com/lincity-ng/lincity-ng/tree/21796de530d68b30881c6b0f6a6a9d6bbfb1e31a) · [Issue-Tracker](https://github.com/lincity-ng/lincity-ng/issues)

- **Gebäudereihenfolge.** Die zufällige Reihenfolge bei Handel und Aktualisierung ist gewollt. Der Autor von #231 (geschlossen) beschreibt sie ausdrücklich. Das Original mischt Iteratoren eines nach Zeigern sortierten Sets. Bei anderer Speicherbelegung kann dieselbe Permutation andere Gebäude treffen. Für eine Abhängigkeit von Speicheradressen als Spielregel fehlt ein Beleg. Ein Replay muss die tatsächlich verarbeiteten Gebäude identifizieren. [#231](https://github.com/lincity-ng/lincity-ng/issues/231) · [Gebäudemenge](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/world.hpp#L162) · [Tagesschritt](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/simulate.cpp#L230-L247)
- **Zufallsquellen.** Das Original würfelt beispielsweise Handelsfreigaben mit `rand()` aus. Für das Mischen verwendet es einen eigenen Generator, initialisiert aus `std::random_device` und Uhrzeit. Ein einzelner C-Zufallsseed garantiert daher keinen identischen Tagesablauf. Für die exakte Folge oder die Wahl beider Generatoren fand sich keine Vorgabe. #103 (geschlossen) behandelt eine reproduzierbare Build- und Testumgebung. [Handelsfreigaben und Mischung](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/simulate.cpp#L218-L237) · [Generator](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/util/randutil.cpp#L29-L44) · [#103](https://github.com/lincity-ng/lincity-ng/issues/103)
- **Windmühlenkosten.** Die alte Windmühle soll je vier Tage eine Geldeinheit Betriebskosten verursachen. Das statische lokale `paid` kann darüber hinaus eine Zahlung von Welt A für Welt B wirksam machen, selbst nach einem Stadtwechsel im selben Prozess. Das widerspricht der Buchung auf dem Konto der jeweiligen Welt. Kein direkter Issue-Treffer. #31 (offen) vergleicht Solar- und Windkraftwerke hinsichtlich ihrer Wirtschaftlichkeit. [Kostensatz](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/modules/windmill.hpp#L51-L52) · [Zahlungsprüfung](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/modules/windmill.cpp#L77-L93) · [#31](https://github.com/lincity-ng/lincity-ng/issues/31)
- **Abrisskostenvorschau.** Bei einem Klick auf ein Marktfeld zeigt die Statuszeile den Preis des Geländes. Abgebucht wird die Gebühr für den gesamten Markt. Bei einer Flächenauswahl zählt die Vorschau jede erlaubte Zelle auf der unveränderten Karte. Die Ausführung läuft in umgekehrter Reihenfolge und verändert die Karte mit jedem Abriss. Die korrigierte Vorschau zählt den Markt in Ausführungsreihenfolge einmal und schätzt für weitere Zellen den freigelegten Bodenpreis. Diese Summe bleibt eine Schätzung, denn eine abgerissene Erzmine kann Wasser hinterlassen und eine abgerissene Hütte Feuer. #224 und #294 (beide geschlossen) betreffen die Abrisssperre und die Warnung beim unfertigen Monument. Keiner beschreibt den Preisfehler. [Vorschau](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity-ng/GameView.cpp#L1413-L1452) · [Preisauskunft](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity-ng/UserOperation.cpp#L192-L199) · [Erzmine](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/modules/oremine.cpp#L203-L219) · [Hütte](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/modules/shanty.cpp#L262-L276) · [Flächenausführung](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity-ng/GameView.cpp#L685-L696) · [Abbuchung](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/lintypes.cpp#L310-L315) · [#224](https://github.com/lincity-ng/lincity-ng/issues/224) · [#294](https://github.com/lincity-ng/lincity-ng/issues/294)
- **Koordinate außerhalb der Karte.** `OutsideMapMessage` erhält die angefragte Position und verwirft sie. Dadurch liefern `getPoint()` und der Diagnosetext für jede Position den Standardwert (0, 0). Das ist ein Fehler im Meldungsinhalt. Der cppcheck-Anhang zu #399 (offen) erwähnt an dieser Stelle lediglich den Konstruktor ohne `explicit`. [Konstruktor und Zugriff](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/messages.hpp#L351-L362) · [Diagnosetext](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/messages.cpp#L104-L106) · [#399](https://github.com/lincity-ng/lincity-ng/issues/399)
- **Gespeicherter Kartenausschnitt.** Das Original liest beide Koordinaten von `recentPoint`, platziert danach die Gebäude und überschreibt den Punkt bei jeder Platzierung. Bei gesetztem Speicherwert und Gebäuden wird deshalb die letzte Gebäudeposition angezeigt. In Version 2130 speichern `main_screen_originx/y` die beiden Koordinaten der Bildschirmmitte. Die damalige Ansicht setzt beim Laden genau diesen Kartenpunkt ins Bildzentrum. #172 (geschlossen durch PR #227) beschreibt falsche Kamerapositionen beim Start und Laden. Das Ticket benennt diesen Überschreibvorgang nicht. [Lader](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/xmlloadsave.cpp#L1042-L1075) · [Bildschirmmitte in 2.13.1](https://github.com/lincity-ng/lincity-ng/blob/lincity-ng-2.13.1/src/lincity-ng/GameView.cpp#L242-L253) · [Älteres Format](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/xmlloadsave.cpp#L798-L799) · [Platzierung](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/lintypes.cpp#L295) · [Kamera](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity-ng/Game.cpp#L190) · [#172](https://github.com/lincity-ng/lincity-ng/issues/172) · [PR #227](https://github.com/lincity-ng/lincity-ng/pull/227)
- **Negative Einnahmen in den Statistiken.** Bei einer Steuereinnahme erhöht sich die Kasse. Der Statistikzähler sinkt um denselben Betrag, und die Finanzansicht dreht das Vorzeichen für Anzeige und Nettosumme wieder um. Diese Buchungskonvention gehört zum Vergleichsverhalten. #337 (geschlossen) erklärt jährliche Steuerzahlungen bei laufenden Ausgaben. #28 (geschlossen) behandelt einen vermuteten Überlauf bei hohen Beträgen. Beide sind vom Vorzeichen der Statistik zu unterscheiden. [Buchung](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/engine.cpp#L43-L52) · [Finanzansicht](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity-ng/Mps.cpp#L398-L420) · [#337, Erklärung](https://github.com/lincity-ng/lincity-ng/issues/337#issuecomment-3959318233) · [#28, Abschluss](https://github.com/lincity-ng/lincity-ng/issues/28#issuecomment-1837037246)
- **Ausgaben und Kreditgrenze.** Das Original lehnt eine Zahlung unter null ohne Kredit bzw. unter −2 Milliarden mit Kredit durch eine Ausnahme ab. `World::expense` wirft ein `std::exception_ptr`-Objekt. Die Windmühle versucht ihrerseits, das Meldungsobjekt zu fangen. Beide Stellen müssen den Ausnahmetyp `OutOfMoneyMessage::Exception` verwenden. Deshalb kann ein abgelehnter Betriebskostenabzug den Tagesschritt abbrechen. #220 (offen) erörtert ausdrücklich den Betriebsstopp bei fehlendem Geld. Ein Simulationsabbruch ist dort nicht vorgesehen. [Zahlung](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/engine.cpp#L54-L65) · [Windmühle](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/modules/windmill.cpp#L77-L93) · [#220, Vorschlag und Grenzen](https://github.com/lincity-ng/lincity-ng/issues/220#issuecomment-2466413124)
- **Überlauf und Nullsaldo-Meldung.** Die Geldfunktionen berechnen Addition bzw. Subtraktion in einem vorzeichenbehafteten `int`, bevor sie die Grenze prüfen. Ein Überlauf ergibt in C++ undefiniertes Verhalten. Einnahmen sollen am Maximum gedeckelt, Ausgaben jenseits des Limits abgelehnt werden. Nach erfolgreicher Abbuchung vergleicht die Warnbedingung denselben bereits aktualisierten Saldo gleichzeitig mit `> 0` und `<= 0`. Die beabsichtigte Meldung beim Übergang von Guthaben zu null oder Schulden kann so nie erscheinen. #28 und #220 thematisieren Überläufe. Im geprüften Bestand gibt es keinen direkten Bericht zur unerreichbaren Warnung. [Geldfunktionen](https://github.com/lincity-ng/lincity-ng/blob/21796de5/src/lincity/engine.cpp#L44-L65) · [#28](https://github.com/lincity-ng/lincity-ng/issues/28) · [#220, Einschätzung zur Erreichbarkeit](https://github.com/lincity-ng/lincity-ng/issues/220#issuecomment-2468887080)


## Stand der Korrekturen im C++-Core

Die Tests prüfen die angefragte Koordinate bei einer Ablehnung, den Abrisspreis
auf Haupt- und Nebenfeld eines Markts, getrennte Windmühlenzahlungen in zwei
Welten und den weiteren Betrieb am Kreditlimit. Bei einer erfolgreichen
Buchung von positivem Guthaben auf null oder ins Minus erscheint einmalig eine
Meldung. Geldgrenzen und die Sättigung der betroffenen Buchungskonten sind durch
Grenzwertfälle belegt.

Der Lader stellt einen ausdrücklich gespeicherten `recentPoint` in Format
2140 nach der Gebäudeplatzierung wieder her. Ohne diesen Eintrag gilt weiterhin
die letzte Platzierung. Für historische 2130-Szenarien bildet der unveränderte
Originalvergleich weiterhin das damalige Ladeverhalten ab. Der Zahlungszyklus
der Windmühlen beginnt beim Laden wie beim früheren Start in einem neuen
Prozess. Das Speicherformat enthält dafür kein Feld.

Bei der Brückenplatzierung kann der allgemeine Bauplatztest zunächst einen
belegten Platz melden, den die spezifische Brückenregel anschließend zulässt.
Der Rückgabewert entscheidet über die Freigabe. Der Trace erfasst einen
Ablehnungsgrund ausschließlich bei einer abgelehnten Vorschau.

## Befunde aus den zusätzlichen Abdeckungstests

- Eine Überflutung am Kreditlimit hinterließ trotz Ablehnung Wasser auf der
  Karte. Die Ausgabe wird jetzt vor der Geländeveränderung gebucht. Der Test
  prüft Karte, Konto und Änderungsindikatoren nach der Ablehnung.
- Ein weiterer erfolgreicher Raketenstart nach Spielende konnte in einer
  leeren Stadt bei der Evakuierung hängen bleiben. Die Evakuierung endet jetzt
  auch bei bereits gesetztem Spielende, wenn sie keine Einwohner mehr findet.
  Das Originalticket #241 betrifft die Wiederverwendung der Startrampe.
  [#241](https://github.com/lincity-ng/lincity-ng/issues/241)
- Gespeicherte Marktregeln mit unterschiedlichen Kauf- und Verkaufswerten
  gingen beim Laden verloren. Der Lader prüft jetzt das Präfix vor der
  Umwandlung in einen Wahrheitswert und weist dem ausgewählten Feld den
  gelesenen Wert zu. Die sechs historischen Szenarien enthalten nur Märkte mit
  symmetrischen Einstellungen. Ein neuer Spielstand prüft beide Richtungen.
- Die Schule erfasst laufende Kosten zunächst im Ausgabenkonto und schließt
  die Periode an Tag 100 ab. Die Universität belastet die Kasse direkt und
  schließt an Tag 99 ab. Das Monument benötigt mehr Arbeiterbestand als sein
  Tagesverbrauch und wechselt nach Erreichen von 100 Prozent erst im nächsten
  Update in den abgeschlossenen Zustand. Diese Eigenschaften sind
  als Referenzverhalten in den neuen Tests erfasst.
- Die Erzmine prüfte bei der Rücklagerung weniger Arbeitseinheiten als sie
  anschließend abzog. Die Prüfung verwendet jetzt dieselben 209 Einheiten
  wie die Buchung. Das verwandte Originalticket #69 beschreibt eine andere
  Eigenschaft, die mögliche Rücklagerung über den natürlichen Erzvorrat.
  [#69](https://github.com/lincity-ng/lincity-ng/issues/69)
- Die Kohlemine behielt nach erfolgloser Suche eine Position außerhalb des
  Abbaugebiets. Beide Suchpfade setzen die Position jetzt zurück. Der
  Regressionstest prüft wiederholt erfolglose Versuche und den nächsten
  erfolgreichen Versuch innerhalb des Gebiets.
- Der 2140-Lader schnitt die vier erweiterbaren Bevölkerungshistorien bei
  190 Monaten ab. Er übernimmt nun fortlaufend gespeicherte Werte darüber
  hinaus. Die Länge fester Reihen wie des Geldverlaufs bleibt begrenzt.
- In einer vollständig bebauten Stadt kann die Standortsuche für ein
  Elendsviertel ohne Ergebnis enden. Beide Suchzweige erkennen diesen
  Zustand jetzt. Der Tageswechsel und die vorhandene Bebauung bleiben
  erhalten.
- Die Verbindung eines Sees am Kartenrand mit einem Fluss bezog beim
  Weiterlaufen auch Nachbarn außerhalb der Karte ein. Der Test prüft
  jetzt, dass das Wasser am Rand angeschlossen wird und die Suche dort endet.
- Die Zinsberechnung zum Jahresende negierte den Kontostand vor der
  Division. Für den kleinsten darstellbaren Kontostand konnte das
  überlaufen. Der Test prüft die Begrenzung extremer Kontostände und
  daneben die normale Zinsbuchung sowie die jährlichen Handelsfreigaben.
- Speicherfehler nach erfolgreichem Öffnen werden jetzt auch über die
  Gzip-Rückrufe gemeldet. Der Linux-Test verwendet dafür das vorhandene
  Zeichengerät `/dev/full`. Die Fehlererkennung beim abschließenden Schreiben
  hängt damit nicht von `xmlTextWriterClose` ab.

Originalticket #369 nennt Abstürze beim Bau von Kohleminen, Häfen und bei
Flussarbeiten. Der dort untersuchte Stacktrace führt zur Behandlung einer
Hafenablehnung in der Desktop-Oberfläche. Der aktuelle Desktop-Code erkennt
diese Meldung bereits. Die beiden hier abgedeckten Randfehler bei Kohlemine
und Fluss sind durch eigene Tests belegt. Für die übrigen Abstürze liefert das
Ticket keine bestätigte Ursache. #159 beschreibt das Anlegen eines Flusses
ohne vorhandenen Flussanschluss. [#369](https://github.com/lincity-ng/lincity-ng/issues/369) · [#159](https://github.com/lincity-ng/lincity-ng/issues/159)
