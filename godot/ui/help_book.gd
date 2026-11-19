extends AcceptDialog

const HOME := "desktop"
const ALIASES := {"evacutate": "evacuate", "farms": "farm", "indistryh": "industryh", "indistryl": "industryl", "mills": "mill", "msg-transport": "msb-transport", "rivers": "river", "schools": "school"}
const INTRO := """[font_size=24]Die Stadt verwalten[/font_size]

Ein Markt verteilt Lebensmittel, Güter und Arbeitskräfte in seinem Einzugsgebiet. Platziere Wohngebiete, einen Bauernhof und einen Brunnen in der Nähe. Wege verbinden die Stadt mit weiteren Betrieben.

[b]Bauen und untersuchen[/b]
Die Tasten 1 bis 5 wählen Untersuchen, Straße, Wohnen, Markt und Abriss. Im Baukatalog (B) findest du alle Werkzeuge nach Kategorie. Esc schließt den Feldbericht oder wechselt zu „Untersuchen“. Mit F1 öffnest du die Spielhilfe. F12 speichert schnell, F9 lädt diese Schnellspeicherung. Klicke zum Bauen auf ein Feld. Ziehe einen Feldweg, eine Straße oder eine Bahnlinie mit gedrückter linker Maustaste. Strg wechselt die Richtung der Kurve. Mit „Untersuchen“ öffnest du den Feldbericht und erhältst Zugang zu Produktion, Handelsregeln oder Raketenstart.

[b]Kamera und Kartenebenen[/b]
Drehe die Ansicht mit gedrückter rechter Maustaste oder mit Q und E. Verschiebe sie mit der mittleren Maustaste, WASD oder den Pfeiltasten. Das Mausrad zoomt. Ein Klick in die Minikarte springt zum Ziel. Unter der Minikarte wählst du die Kartenebene.

[b]Statistik und Spielziele[/b]
Unter „Spiel → Verlauf und Nachhaltigkeit“ findest du Monatsdiagramme und die sechs Kriterien für den Sieg durch eine nachhaltige Wirtschaft. Du kannst das Spiel auch durch die vollständige Evakuierung per Rakete gewinnen. Anzeige, Musik, Effekte und Vorgaben für neue Städte stehen unter „Spiel → Einstellungen“.

[b]LinCity-Handbuch[/b]
[url=tutorial-basics]Erste Schritte[/url]    [url=market]Märkte[/url]    [url=transport]Verkehr[/url]    [url=index]Alle Themen[/url]

Die folgenden Originalartikel beschreiben auch die Bedienung der klassischen Oberfläche."""
var article := RichTextLabel.new()
var back := Button.new()
var history: Array[String] = []
var current := ""
var whitespace := RegEx.new()

func _ready() -> void:
	title = "Spielhilfe"
	ok_button_text = "Schließen"
	whitespace.compile("\\s+")
	var layout := VBoxContainer.new()
	add_child(layout)
	var actions := HBoxContainer.new()
	layout.add_child(actions)
	back.text = "Zurück"
	actions.add_child(back)
	back.pressed.connect(func(): _open(history.pop_back(), false))
	for link in [["Steuerung", HOME], ["Themenverzeichnis", "index"]]:
		var button := Button.new()
		actions.add_child(button)
		button.text = link[0]
		button.pressed.connect(_open.bind(link[1]))
	layout.add_child(article)
	article.size_flags_vertical = Control.SIZE_EXPAND_FILL
	article.bbcode_enabled = true
	article.selection_enabled = true
	article.meta_clicked.connect(func(link): _open(str(link)))
	_open(HOME)

func _open(link: String, remember: bool = true) -> void:
	link = ALIASES.get(link, link)
	if not link.is_valid_filename(): return
	if remember and not current.is_empty() and current != link:
		history.append(current)
	current = link
	back.disabled = history.is_empty()
	article.clear()
	if link == HOME:
		article.append_text(INTRO)
	else:
		_render_xml(link)
	article.scroll_to_line(0)

func _render_xml(link: String) -> void:
	var parser := XMLParser.new()
	if parser.open("res://ui/help_de/%s.xml" % link) != OK:
		article.add_text("Dieser Artikel ist nicht verfügbar.")
		return
	var style := ""
	while parser.read() == OK:
		match parser.get_node_type():
			XMLParser.NODE_ELEMENT:
				match parser.get_node_name():
					"p":
						style = parser.get_named_attribute_value_safe("style")
						if style in ["htitle", "hsubtitle"]:
							article.push_font_size(24 if style == "htitle" else 19)
					"a": article.push_meta(parser.get_named_attribute_value_safe("href"))
					"b": article.push_bold()
					"i": article.push_italics()
					"li": article.add_text("•  ")
					"img": _illustration(parser.get_named_attribute_value_safe("src"))
			XMLParser.NODE_TEXT:
				var value := parser.get_node_data()
				if not value.strip_edges().is_empty():
					article.add_text(whitespace.sub(value, " ", true))
			XMLParser.NODE_ELEMENT_END:
				match parser.get_node_name():
					"p":
						if style in ["htitle", "hsubtitle"]: article.pop()
						article.add_text("\n\n")
					"li": article.add_text("\n")
					"a", "b", "i": article.pop()

func _illustration(path: String) -> void:
	var name_only := path.get_file()
	var resource := "res://ui/icons/" + name_only
	if not ResourceLoader.exists(resource): resource = "res://ui/help_images/" + name_only
	if not ResourceLoader.exists(resource): return
	var image: Texture2D = load(resource)
	var scale_factor := minf(1, minf(500.0 / image.get_width(), 220.0 / image.get_height()))
	article.add_image(image, roundi(image.get_width() * scale_factor), roundi(image.get_height() * scale_factor))
	article.add_text("\n")
