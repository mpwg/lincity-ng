extends AcceptDialog

const FILES := {
	"Projekt und Spieldaten": "Hinweise.txt",
	"Mitwirkende": "creditslist.xml",
	"LinCity-NG · GPL": "LinCity-GPL.txt",
	"Spieldaten · GPL / CC BY-SA": "Spieldaten.txt",
	"Godot Engine und Drittkomponenten": "",
	"Rust-Abhängigkeiten": "Rust-Bibliotheken.txt",
}
var sections := OptionButton.new()
var body := TextEdit.new()

func _ready() -> void:
	title = tr("Mitwirkende und Lizenzen")
	ok_button_text = tr("Schließen")
	var layout := VBoxContainer.new()
	add_child(layout)
	layout.add_child(sections)
	for section: String in FILES:
		sections.add_item(tr(section))
		sections.set_item_metadata(sections.item_count - 1, FILES[section])
	sections.item_selected.connect(_show_section)
	layout.add_child(body)
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.editable = false
	body.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	body.accessibility_name = tr("Lizenztext")
	sections.accessibility_name = tr("Dokument auswählen")
	_show_section(0)

func _show_section(index: int) -> void:
	var file: String = sections.get_item_metadata(index)
	match file:
		"": body.text = engine_notices()
		"creditslist.xml": body.text = contributors()
		_: body.text = FileAccess.get_file_as_string("res://licenses/" + file)
	body.scroll_vertical = 0

static func contributors() -> String:
	var parser := XMLParser.new()
	var lines := PackedStringArray()
	if parser.open("res://licenses/creditslist.xml") != OK: return ""
	var line := PackedStringArray()
	while parser.read() == OK:
		match parser.get_node_type():
			XMLParser.NODE_ELEMENT:
				if parser.get_node_name() == "p":
					line.clear()
					if parser.get_named_attribute_value_safe("style") == "section": lines.append("")
			XMLParser.NODE_TEXT:
				var value := parser.get_node_data().strip_edges()
				if not value.is_empty(): line.append(value)
			XMLParser.NODE_ELEMENT_END:
				if parser.get_node_name() == "p": lines.append(" ".join(line))
	return "\n".join(lines)

static func engine_notices() -> String:
	var lines := PackedStringArray(["Godot Engine " + Engine.get_version_info().string, Engine.get_license_text()])
	for component: Dictionary in Engine.get_copyright_info():
		lines.append("\n" + str(component.name))
		for part: Dictionary in component.parts:
			lines.append(", ".join(part.files))
			lines.append("\n".join(part.copyright))
			lines.append(part.license)
	var licenses := Engine.get_license_info()
	for license_name: String in licenses:
		lines.append("\n" + license_name)
		lines.append(licenses[license_name])
	return "\n".join(lines)
