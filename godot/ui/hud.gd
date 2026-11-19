extends CanvasLayer

signal tool_changed(id: int)
signal speed_changed(speed: int)
signal overlay_changed(active: bool)
signal focus_requested(cell: Vector2i)
signal save_requested(path: String)
signal load_requested(path: String)
signal new_requested()
signal new_settings(climate: int, side: int, village: bool, trees: bool)
signal layer_selected(id: int, commodity: int)
signal survey_requested()
signal launch_requested()
signal trade_changed(id: int, buy: bool, sell: bool)
signal report_requested(production: bool)

const Catalog = preload("res://ui/catalog.gd")
const Minimap = preload("res://ui/minimap.gd")
const PAPER := Color("#f1f2df")
const MUTED := Color("#b1c6bb")
const ACCENT := Color("#d9eb98")
const PANEL := Color("#1c302bee")
const MONTHS := ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"]
const LAYERS := ["Stadtplan", "Luftbelastung", "Arbeitsmarkt", "Ernährung", "Stromversorgung", "Feuerwehr", "Sport", "Gesundheit", "Kohlevorräte", "Verkehr", "Warenbestand"]
const GOODS := ["Lebensmittel", "Arbeitskräfte", "Kohle", "Waren", "Erz", "Stahl", "Abfall", "Niederspannung", "Hochspannung", "Wasser"]
const ACCOUNTS := {
	"Income tax": "Einkommensteuer", "Coal tax": "Kohlesteuer", "Goods tax": "Warensteuer", "Exports": "Exporte",
	"Construction": "Bauvorhaben", "Coal survey": "Kohleerkundung", "Imports": "Importe", "Unemployment": "Arbeitslosenhilfe",
	"Transport": "Verkehr", "Windmills": "Windmühlen", "Universities": "Universitäten", "Recycling": "Recycling",
	"Deaths": "Sterbefälle", "Health": "Gesundheit", "Rockets": "Raketen", "Schools": "Schulen",
	"Fire stations": "Feuerwehr", "Sports": "Sport", "Interest": "Zinsen",
}
const SCENARIOS := {
	"Beach.scn.gz": "Am Strand", "Rocket_98.scn.gz": "Raketenprogramm",
	"bad_times.scn.gz": "Schlechte Zeiten", "extreme_arid.scn.gz": "Trockengebiet",
	"extreme_wetland.scn.gz": "Feuchtgebiet", "good_times.scn.gz": "Gute Zeiten",
}

var surface := Control.new()
var treasury := Label.new()
var inhabitants := Label.new()
var date := Label.new()
var speed_picker := OptionButton.new()
var category_picker := OptionButton.new()
var palette: PanelContainer
var tool_list := HBoxContainer.new()
var inspector: PanelContainer
var inspected_group := -1
var map := Minimap.new()
var details := RichTextLabel.new()
var preview := Label.new()
var layers := OptionButton.new()
var commodities := OptionButton.new()
var production := CheckButton.new()
var trade_button := Button.new()
var launch_button := Button.new()
var trade_dialog := AcceptDialog.new()
var trade_list := VBoxContainer.new()
var report_dialog := AcceptDialog.new()
var report_body := RichTextLabel.new()
var survey_dialog := ConfirmationDialog.new()
var launch_dialog := ConfirmationDialog.new()
var events := PackedStringArray()
var current_rules: Array = []
var current_report := -1
var save_dialog := FileDialog.new()
var load_dialog := FileDialog.new()
var new_dialog := ConfirmationDialog.new()
var scenarios := OptionButton.new()
var climate := OptionButton.new()
var extent := SpinBox.new()
var village := CheckBox.new()
var trees := CheckBox.new()
var help_dialog := preload("res://ui/help_book.gd").new()
var catalog := {}
var categories := {}
var chosen_tool := -1
var city := {}

func _ready() -> void:
	add_child(surface)
	surface.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	surface.mouse_filter = Control.MOUSE_FILTER_IGNORE
	surface.theme = _theme()
	_top_bar()
	_palette()
	_overview()
	_footer()
	_dialogs()
	_refresh_categories()

func _box(color: Color, padding_x: int, padding_y: int, radius: int = 5) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = color
	style.set_corner_radius_all(radius)
	style.content_margin_left = padding_x
	style.content_margin_right = padding_x
	style.content_margin_top = padding_y
	style.content_margin_bottom = padding_y
	return style

func _theme() -> Theme:
	var theme := Theme.new()
	theme.default_font_size = 15
	theme.set_color("font_color", "Label", PAPER)
	theme.set_constant("separation", "VBoxContainer", 7)
	theme.set_constant("separation", "HBoxContainer", 10)
	for type in ["Button", "OptionButton", "MenuButton"]:
		theme.set_stylebox("normal", type, _box(Color("#30463e"), 10, 5))
		theme.set_stylebox("hover", type, _box(Color("#405c50"), 10, 5))
		theme.set_stylebox("pressed", type, _box(Color("#53705a"), 10, 5))
		theme.set_color("font_color", type, PAPER)
		theme.set_color("font_pressed_color", type, ACCENT)
	return theme

func _panel(preset: Control.LayoutPreset, bounds: Rect2) -> PanelContainer:
	var panel := PanelContainer.new()
	surface.add_child(panel)
	panel.set_anchors_and_offsets_preset(preset)
	panel.offset_left = bounds.position.x
	panel.offset_top = bounds.position.y
	panel.offset_right = bounds.end.x
	panel.offset_bottom = bounds.end.y
	panel.add_theme_stylebox_override("panel", _box(PANEL, 12, 10))
	return panel

func _label(value: String, font_size: int, color: Color = PAPER) -> Label:
	var label := Label.new()
	label.text = value
	label.add_theme_font_size_override("font_size", font_size)
	label.add_theme_color_override("font_color", color)
	return label

func _button(title: String, callback: Callable) -> Button:
	var button := Button.new()
	button.text = title
	button.pressed.connect(callback)
	return button

func _metric(row: HBoxContainer, caption: String, value: Label, width: int) -> void:
	var column := VBoxContainer.new()
	column.add_theme_constant_override("separation", 0)
	column.custom_minimum_size.x = width
	row.add_child(column)
	column.add_child(_label(caption, 12, MUTED))
	value.add_theme_font_size_override("font_size", 17)
	value.text = "—"
	column.add_child(value)

func _top_bar() -> void:
	var panel := _panel(Control.PRESET_TOP_WIDE, Rect2(10, 8, -20, 60))
	var row := HBoxContainer.new()
	panel.add_child(row)
	var title := _label("LinCity NG", 26, ACCENT)
	title.custom_minimum_size.x = 150
	row.add_child(title)
	_metric(row, "Stadtkasse", treasury, 112)
	_metric(row, "Bevölkerung", inhabitants, 92)
	_metric(row, "Kalender", date, 140)
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	spacer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(spacer)
	row.add_child(speed_picker)
	speed_picker.tooltip_text = "Simulationsgeschwindigkeit"
	speed_picker.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	for speed in [0, 1, 2, 4]:
		speed_picker.add_item("Pause" if speed == 0 else "%d× Tempo" % speed, speed)
	speed_picker.select(1)
	speed_picker.item_selected.connect(func(index: int): speed_changed.emit(speed_picker.get_item_id(index)))
	var menu := MenuButton.new()
	menu.text = "Spiel ▾"
	menu.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	menu.flat = false
	row.add_child(menu)
	var popup := menu.get_popup()
	popup.add_item("Neue Stadt oder Szenario …", 0)
	popup.add_item("Spielstand laden …", 1)
	popup.add_item("Spielstand speichern …", 2)
	popup.add_separator()
	popup.add_item("Stadtstatistik", 4)
	popup.add_item("Jahresabschluss", 5)
	popup.add_item("Ereignisse", 6)
	popup.add_item("Kohlevorkommen erkunden …", 7)
	popup.add_separator()
	popup.add_item("Spielhilfe", 3)
	popup.id_pressed.connect(_menu_action)

func _palette() -> void:
	palette = _panel(Control.PRESET_BOTTOM_LEFT, Rect2(10, -124, 610, 116))
	surface.resized.connect(_fit_palette)
	var layout := VBoxContainer.new()
	palette.add_child(layout)
	var row := HBoxContainer.new()
	layout.add_child(row)
	var category := VBoxContainer.new()
	category.custom_minimum_size.x = 142
	row.add_child(category)
	category.add_child(_label("Baupalette", 15, ACCENT))
	category.add_child(category_picker)
	category_picker.item_selected.connect(func(_index: int): _refresh_tools())
	var scroll := ScrollContainer.new()
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	row.add_child(scroll)
	scroll.add_child(tool_list)
	tool_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tool_list.add_theme_constant_override("separation", 5)
	layout.add_child(preview)
	preview.add_theme_font_size_override("font_size", 12)
	preview.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	preview.add_theme_color_override("font_color", MUTED)

func _overview() -> void:
	var panel := _panel(Control.PRESET_BOTTOM_RIGHT, Rect2(-198, -223, 188, 215))
	panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	var column := VBoxContainer.new()
	panel.add_child(column)
	column.add_child(_label("Stadtübersicht", 17))
	column.add_child(map)
	map.custom_minimum_size = Vector2(138, 138)
	map.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	map.tooltip_text = "Klicken, um diesen Kartenausschnitt anzuzeigen"
	map.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	map.cell_selected.connect(func(cell: Vector2i): focus_requested.emit(cell))
	for layer_name in LAYERS: layers.add_item(layer_name)
	column.add_child(layers)
	layers.tooltip_text = "Kartenebene auswählen"
	for good in GOODS: commodities.add_item(good)
	column.add_child(commodities)
	commodities.hide()
	layers.item_selected.connect(func(id: int):
		commodities.visible = id == 10
		overlay_changed.emit(id == 1)
		layer_selected.emit(id, commodities.selected))
	commodities.item_selected.connect(func(good: int): layer_selected.emit(layers.selected, good))

func _footer() -> void:
	inspector = _panel(Control.PRESET_TOP_RIGHT, Rect2(-320, 76, 310, 224))
	inspector.hide()
	var column := VBoxContainer.new()
	inspector.add_child(column)
	var headings := HBoxContainer.new()
	column.add_child(headings)
	headings.add_child(_label("Feldbericht", 17))
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	headings.add_child(spacer)
	headings.add_child(production)
	production.text = "Produktion"
	production.toggled.connect(func(active: bool): report_requested.emit(active))
	column.add_child(details)
	details.size_flags_vertical = Control.SIZE_EXPAND_FILL
	details.selection_enabled = true
	details.add_theme_color_override("default_color", PAPER)
	details.add_theme_font_size_override("normal_font_size", 14)
	details.text = "Mit „Untersuchen“ ein Feld auf der Karte auswählen."

	column.add_child(trade_button)
	trade_button.text = "Handel einstellen …"
	trade_button.pressed.connect(_open_trade)
	column.add_child(launch_button)
	launch_button.text = "Rakete starten …"
	launch_button.pressed.connect(func(): launch_dialog.popup_centered())
	set_inspection(-1, [])

func _dialogs() -> void:
	for dialog: FileDialog in [save_dialog, load_dialog]:
		surface.add_child(dialog)
		dialog.access = FileDialog.ACCESS_FILESYSTEM
		dialog.transient = true
		dialog.current_dir = ProjectSettings.globalize_path("user://")
		dialog.add_filter("*.gz", "LinCity-Spielstände")
	save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	save_dialog.title = "Stadt speichern"
	save_dialog.current_file = "Meine Stadt.gz"
	save_dialog.file_selected.connect(func(path: String): save_requested.emit(path))
	load_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	load_dialog.title = "Stadt laden"
	load_dialog.file_selected.connect(func(path: String): load_requested.emit(path))
	surface.add_child(new_dialog)
	new_dialog.title = "Neue Stadt"
	new_dialog.ok_button_text = "Starten"
	new_dialog.cancel_button_text = "Abbrechen"
	var content := VBoxContainer.new()
	new_dialog.add_child(content)
	content.add_child(_label("Ausgangslage wählen", 19))
	content.add_child(scenarios)
	scenarios.add_item("Neue Landschaft")
	scenarios.add_item("Kleinstadt am Fluss")
	scenarios.set_item_metadata(1, "res://scenarios/am_fluss.scn.gz")
	for file: String in SCENARIOS:
		scenarios.add_item(SCENARIOS[file])
		scenarios.set_item_metadata(scenarios.item_count - 1, "res://opening/" + file)
	var options := VBoxContainer.new()
	content.add_child(options)
	options.add_child(_label("Landschaft", 14, MUTED))
	options.add_child(climate)
	for region in ["Flusslandschaft", "Wüste", "Gemäßigte Zone", "Sumpfland"]: climate.add_item(region)
	options.add_child(_label("Kantenlänge in Feldern", 14, MUTED))
	options.add_child(extent)
	extent.min_value = 50
	extent.max_value = 10000
	extent.step = 25
	extent.value = 100
	village.text = "Mit Startdorf"
	village.button_pressed = true
	options.add_child(village)
	trees.text = "Mit Bäumen"
	trees.button_pressed = true
	options.add_child(trees)
	scenarios.item_selected.connect(func(index: int): options.visible = index == 0)
	content.add_child(_label("Die aktuelle Stadt wird ersetzt.", 14, MUTED))
	new_dialog.confirmed.connect(func():
		if scenarios.selected == 0:
			if new_settings.has_connections():
				new_settings.emit(climate.selected, int(extent.value), village.button_pressed, trees.button_pressed)
			else:
				new_requested.emit()
		else:
			load_requested.emit(str(scenarios.get_selected_metadata())))
	surface.add_child(help_dialog)
	surface.add_child(report_dialog)
	report_dialog.add_child(report_body)
	report_body.selection_enabled = true
	report_body.add_theme_font_size_override("normal_font_size", 16)
	report_dialog.ok_button_text = "Schließen"
	surface.add_child(trade_dialog)
	trade_dialog.title = "Handelsregeln"
	trade_dialog.ok_button_text = "Schließen"
	trade_dialog.add_child(trade_list)
	surface.add_child(survey_dialog)
	survey_dialog.title = "Kohlevorkommen erkunden"
	survey_dialog.dialog_text = "Die Erkundung kostet 1.000.000 $. Danach zeigt die Kohlekarte die Lagerstätten."
	survey_dialog.ok_button_text = "Erkunden"
	survey_dialog.cancel_button_text = "Abbrechen"
	survey_dialog.confirmed.connect(func(): survey_requested.emit())
	surface.add_child(launch_dialog)
	launch_dialog.title = "Raketenstart"
	launch_dialog.dialog_text = "Soll die vorbereitete Rakete starten?"
	launch_dialog.ok_button_text = "Starten"
	launch_dialog.cancel_button_text = "Abbrechen"
	launch_dialog.confirmed.connect(func(): launch_requested.emit())

func _menu_action(id: int) -> void:
	match id:
		0: new_dialog.popup_centered(Vector2i(365, 180))
		1: load_dialog.popup_centered_ratio(0.75)
		2: save_dialog.popup_centered_ratio(0.75)
		3: help_dialog.popup_centered(Vector2i(660, 540))
		4, 5, 6:
			current_report = id
			_refresh_report()
			report_dialog.popup_centered(Vector2i(510, 450))
		7:
			if city.get("coal_survey_done", false):
				show_info("Die Kohlevorkommen sind bereits erkundet. Wähle die Kartenebene „Kohlevorräte“.")
			else:
				survey_dialog.popup_centered()

func setup(tools: Array) -> void:
	chosen_tool = -1
	events.clear()
	production.set_pressed_no_signal(false)
	set_inspection(-1, [])
	catalog.clear()
	catalog[-1] = {"id": -1, "name": "Untersuchen", "size": 1}
	for tool: Dictionary in tools:
		catalog[int(tool.id)] = tool
	if is_node_ready():
		_refresh_categories()

func _refresh_categories() -> void:
	categories.clear()
	var listed := []
	for category: String in Catalog.CATEGORIES:
		var available := []
		for id: int in Catalog.CATEGORIES[category]:
			if catalog.has(id):
				available.append(id)
				listed.append(id)
		if not available.is_empty():
			categories[category] = available
	for id: int in catalog:
		if not listed.has(id):
			if not categories.has("Weitere"): categories["Weitere"] = []
			categories["Weitere"].append(id)
	category_picker.clear()
	for category: String in categories:
		category_picker.add_item(category)
	_refresh_tools()

func _refresh_tools() -> void:
	for child in tool_list.get_children():
		tool_list.remove_child(child)
		child.queue_free()
	if category_picker.selected < 0: return
	var category := category_picker.get_item_text(category_picker.selected)
	var group := ButtonGroup.new()
	for id: int in categories[category]:
		var tool: Dictionary = catalog[id]
		var name_de := Catalog.tool_name(id, str(tool.get("name", id)))
		var button := _button(name_de, _select_tool.bind(id))
		if Catalog.TOOLS.has(id):
			button.icon = load("res://ui/icons/%s.png" % Catalog.TOOLS[id][1])
			button.add_theme_constant_override("icon_max_width", 26)
		button.toggle_mode = true
		button.button_group = group
		button.button_pressed = id == chosen_tool
		button.alignment = HORIZONTAL_ALIGNMENT_CENTER
		button.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
		button.vertical_icon_alignment = VERTICAL_ALIGNMENT_TOP
		button.add_theme_font_size_override("font_size", 13)
		button.custom_minimum_size.x = maxf(70, button.get_theme_font("font").get_string_size(name_de, HORIZONTAL_ALIGNMENT_LEFT, -1, 13).x + 24)
		button.tooltip_text = "%s · %d × %d Felder" % [name_de, tool.get("size", 1), tool.get("size", 1)]
		tool_list.add_child(button)
	_fit_palette()

func _fit_palette() -> void:
	var preferred := tool_list.get_combined_minimum_size().x + 183
	palette.offset_right = 10 + minf(maxf(preferred, 610), maxf(200, surface.size.x - 216))

func _select_tool(id: int) -> void:
	chosen_tool = id
	if id != -1: inspector.hide()
	show_preview("")
	tool_changed.emit(id)

func _number(value: int) -> String:
	var digits := str(absi(value))
	var i := digits.length() - 3
	while i > 0:
		digits = digits.insert(i, ".")
		i -= 3
	return ("−" if value < 0 else "") + digits

func update_city(snapshot: Dictionary) -> void:
	city = snapshot
	var money := int(snapshot.get("money", 0))
	treasury.text = "%s $" % _number(money)
	treasury.add_theme_color_override("font_color", Color("#ffab95") if money < 0 else PAPER)
	inhabitants.text = _number(int(snapshot.get("population", 0)))
	var day := int(snapshot.get("time", 0))
	date.text = "%s %d" % [MONTHS[(day / 100) % 12], day / 1200]
	map.update_map(snapshot)
	if report_dialog.visible: _refresh_report()

func show_info(text: String) -> void:
	if text.is_empty():
		inspector.hide()
		return
	if chosen_tool == -1 or inspected_group < 0: inspector.show()
	if details.text != text:
		details.text = text

func show_preview(text: String) -> void:
	preview.text = text if not text.is_empty() else "%s   ·   Mausrad zum Zoomen   ·   Rechts ziehen zum Drehen" % Catalog.tool_name(chosen_tool, "Bauen")

func set_speed(speed: int) -> void:
	speed_picker.select(speed_picker.get_item_index(speed))

func set_focus(cell: Vector2i) -> void:
	map.focus = cell
	map.queue_redraw()

func set_inspection(group: int, rules: Array, show_production: bool = false) -> void:
	inspected_group = group
	current_rules = rules
	if inspector and group < 0: inspector.hide()
	production.set_pressed_no_signal(show_production)
	trade_button.visible = not rules.is_empty()
	launch_button.visible = group == 23
	production.visible = group >= 0

func _open_trade() -> void:
	for child in trade_list.get_children():
		trade_list.remove_child(child)
		child.queue_free()
	var grid := GridContainer.new()
	grid.columns = 3
	grid.add_theme_constant_override("h_separation", 20)
	trade_list.add_child(grid)
	for caption in ["Ware", "Annehmen", "Abgeben"]:
		grid.add_child(_label(caption, 14, MUTED))
	for rule: Dictionary in current_rules:
		var id := int(rule.id)
		grid.add_child(_label(GOODS[id], 15))
		var buy := CheckBox.new()
		var sell := CheckBox.new()
		grid.add_child(buy)
		grid.add_child(sell)
		buy.button_pressed = rule.buy
		sell.button_pressed = rule.sell
		buy.toggled.connect(func(active: bool): trade_changed.emit(id, active, sell.button_pressed))
		sell.toggled.connect(func(active: bool): trade_changed.emit(id, buy.button_pressed, active))
	trade_dialog.popup_centered()

func show_event(text: String) -> void:
	if text.is_empty(): return
	events.append(text)
	if events.size() > 256: events.remove_at(0)
	show_info(text)
	if report_dialog.visible and current_report == 6: _refresh_report()

func _refresh_report() -> void:
	var lines := PackedStringArray()
	match current_report:
		4:
			report_dialog.title = "Stadtstatistik"
			lines.append("Bevölkerung   %s" % inhabitants.text)
			for pair in [["homeless", "Wohnungssuchende"], ["unemployment", "Arbeitslose"], ["births", "Geburten"], ["deaths", "Tote durch Hunger und Umweltbelastung"], ["rockets", "Gestartete Raketen"]]:
				lines.append("%s   %s" % [pair[1], _number(int(city.get(pair[0], 0)))])
			lines.append("Technologie   %.2f %%" % (float(city.get("tech", 0)) / 10000.0))
			lines.append("Nachhaltigkeit   %s" % ("erreicht" if city.get("sustainable", false) else "offen"))
		5:
			report_dialog.title = "Jahresabschluss · Vorjahr"
			var finances: Dictionary = city.get("finances", {})
			for category: String in ["income", "expenses"]:
				lines.append("Einnahmen" if category == "income" else "Ausgaben")
				var total := 0
				var accounts: Dictionary = finances.get(category, {})
				for item: String in accounts:
					total += int(accounts[item])
					lines.append("    %s   %s $" % [ACCOUNTS.get(item, item), _number(accounts[item])])
				lines.append("Summe   %s $\n" % _number(total))
		6:
			report_dialog.title = "Ereignisse"
			for i in range(events.size() - 1, -1, -1): lines.append(events[i])
			if lines.is_empty(): lines.append("Bisher sind keine Ereignisse eingegangen.")
	report_body.text = "\n".join(lines)
