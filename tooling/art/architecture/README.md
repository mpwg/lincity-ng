# Stadtarchitektur

Die Markthalle, zwei Industrieanlagen und 18 Wohnvarianten sind eigens für LinCity-NG modelliert. Die beiden Stadtansichten im gestalterischen Konzept dienen als Vorlagen für Bauform und Farbgebung. Sämtliche Geometrien und Texturen entstanden für dieses Projekt. Für die Modelle und Bilder gelten die beiden Lizenzen in [COPYING-data.txt](../../../COPYING-data.txt).

[Stadtarchitektur.blend](Stadtarchitektur.blend) enthält die editierbaren Modelle und den gepackten Texturatlas. Jede Collection entspricht einer Baugruppe und Variante. Beim Öffnen ist die Markthalle sichtbar. Die anderen Collections lassen sich im Outliner einblenden. Die drei sichtbaren Detailstufen heißen Near, Mid und Far. Pick enthält die Auswahlgeometrie. In beiden Industrieanlagen bezeichnet Smoke die Schornsteinmündung.

## Export

Im Repository-Stamm diesen Befehl mit Blender 4.5 LTS ausführen. Er erzeugt den Blender-Katalog und die Dateien in `godot/art/architecture/` aus der parametrischen Quelle neu.

```sh
blender --background --python tooling/art/architecture/model.py
```

Die GLBs verweisen über einen relativen Bildpfad auf denselben JPEG-Atlas. Beim Import in Godot 4.5.1 entstehen eine Textur und eine Szene pro Variante. Eine Simulationszelle entspricht einer Godot-Einheit. Die Gebäudemitte bildet den Ursprung. Die Straßenfront liegt in Godot an der positiven Z-Seite.

Der mitgelieferte PNG-Atlas genügt für den Export. Zum Ändern der gemalten Materialfelder stehen `atlas.py` und `layout.py` zur Verfügung. Den Atlas mit Python 3, Pillow und NumPy neu erzeugen und anschließend den Blender-Export wiederholen.

```sh
python3 tooling/art/architecture/atlas.py
```
