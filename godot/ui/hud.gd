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
signal inspection_closed
signal modal_opened

const Catalog = preload("res://ui/catalog.gd")
const Minimap = preload("res://ui/minimap.gd")
const PAPER := Color("#f5f2e9")
const MUTED := Color("#c4c8c0")
const ACCENT := Color("#e3c78a")
const PANEL := Color("#202321ee")
const EDGE := Color("#73766a")
const QUICK_TOOLS := [
	[-1, "Untersuchen", "inspect"], [11, "Straße", "road"], [4, "Wohnen", "housing"],
	[6, "Markt", "market"], [-2, "Abriss", "demolish"],
]
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
var speed_buttons := {}
var quick_buttons := {}
var catalog_button: Button
var catalog_dialog := AcceptDialog.new()
var preview_panel: PanelContainer
var category_picker := OptionButton.new()
var palette: PanelContainer
var tool_list := GridContainer.new()
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
var credits_dialog := preload("res://ui/credits.gd").new()
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
	theme.set_color("default_color", "RichTextLabel", PAPER)
	theme.set_constant("separation", "VBoxContainer", 8)
	theme.set_constant("separation", "HBoxContainer", 8)
	for type in ["Button", "OptionButton", "MenuButton"]:
		for state in ["normal", "hover", "pressed", "hover_pressed", "focus"]:
			var fill: Color = {"normal": Color("#343835"), "hover": Color("#4a514a"), "pressed": Color("#505849"), "hover_pressed": Color("#56604e"), "focus": Color.TRANSPARENT}[state]
			var style := _box(fill, 9, 4, 4)
			style.set_border_width_all(2 if state == "focus" else 1)
			style.border_color = ACCENT if state in ["pressed", "hover_pressed", "focus"] else Color("#656a60")
			theme.set_stylebox(state, type, style)
		theme.set_color("font_color", type, PAPER)
	var dialog_style := _box(Color("#262b28"), 16, 14, 5)
	theme.set_stylebox("panel", "AcceptDialog", dialog_style)
	theme.set_stylebox("panel", "PopupMenu", _box(Color("#262b28"), 6, 6))
	theme.set_stylebox("panel", "PopupPanel", dialog_style)
	var frame := _box(Color("#363d38"), 6, 6)
	frame.expand_margin_top = 32
	theme.set_stylebox("embedded_border", "Window", frame)
	theme.set_stylebox("embedded_unfocused_border", "Window", frame)
	theme.set_color("title_color", "Window", PAPER)
	return theme

func _panel(preset: Control.LayoutPreset, bounds: Rect2) -> PanelContainer:
	var panel := PanelContainer.new()
	surface.add_child(panel)
	panel.set_anchors_and_offsets_preset(preset)
	panel.offset_left = bounds.position.x
	panel.offset_top = bounds.position.y
	panel.offset_right = bounds.end.x
	panel.offset_bottom = bounds.end.y
	var skin := _box(PANEL, 6, 5, 6)
	skin.set_border_width_all(1)
	skin.border_color = EDGE
	skin.shadow_color = Color("#10141066")
	skin.shadow_size = 3
	panel.add_theme_stylebox_override("panel", skin)
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

func _symbol(id: String) -> Texture2D:
	return load("res://ui/symbols/%s.svg" % id)

func _metric(row: HBoxContainer, caption: String, value: Label, symbol: String) -> void:
	var icon := TextureRect.new()
	icon.texture = _symbol(symbol)
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	icon.custom_minimum_size = Vector2(22, 22)
	row.add_child(icon)
	row.add_child(_label(caption, 14, MUTED))
	value.text = "—"
	value.add_theme_font_size_override("font_size", 15)
	row.add_child(value)

func _top_bar() -> void:
	var stats := _panel(Control.PRESET_TOP_LEFT, Rect2(10, 9, 0, 36))
	var row := HBoxContainer.new()
	stats.add_child(row)
	_metric(row, "Bevölkerung", inhabitants, "people")
	row.add_child(VSeparator.new())
	_metric(row, "Stadtkasse", treasury, "coins")
	var controls := _panel(Control.PRESET_TOP_RIGHT, Rect2(-10, 9, 0, 36))
	controls.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	var clock := HBoxContainer.new()
	clock.add_theme_constant_override("separation", 4)
	controls.add_child(clock)
	var speeds := ButtonGroup.new()
	for item in [[0, "Ⅱ", "Pause"], [1, "▶", "Einfaches Tempo"], [2, "▶▶", "Doppeltes Tempo"], [4, "▶▶▶", "Vierfaches Tempo"]]:
		var speed: int = item[0]
		var button := _button(item[1], speed_changed.emit.bind(speed))
		button.tooltip_text = item[2]
		button.add_theme_font_size_override("font_size", 12)
		button.button_group = speeds
		button.toggle_mode = true
		button.button_pressed = speed == 1
		speed_buttons[speed] = button
		clock.add_child(button)
	clock.add_child(VSeparator.new())
	date.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	date.custom_minimum_size.x = 110
	date.tooltip_text = "Spielzeit"
	clock.add_child(date)
	var menu := MenuButton.new()
	menu.text = "Spiel"
	menu.flat = false
	clock.add_child(menu)
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
	popup.add_item("Mitwirkende und Lizenzen …", 8)
	popup.id_pressed.connect(_menu_action)

func _palette() -> void:
	palette = _panel(Control.PRESET_CENTER_BOTTOM, Rect2(0, -90, 0, 80))
	palette.grow_horizontal = Control.GROW_DIRECTION_BOTH
	palette.grow_vertical = Control.GROW_DIRECTION_BEGIN
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	palette.add_child(row)
	for item in QUICK_TOOLS:
		var id: int = item[0]
		var button := _dock_button(item[1], item[2], _select_tool.bind(id))
		button.toggle_mode = true
		button.tooltip_text = "%s (%d)" % [Catalog.tool_name(id), quick_buttons.size() + 1]
		row.add_child(button)
		quick_buttons[id] = button
	row.add_child(VSeparator.new())
	catalog_button = _dock_button("Baukatalog", "catalog", _open_catalog)
	catalog_button.tooltip_text = "Alle Bauwerkzeuge (B)"
	row.add_child(catalog_button)
	preview_panel = _panel(Control.PRESET_CENTER_BOTTOM, Rect2(0, -142, 0, 30))
	preview_panel.grow_horizontal = Control.GROW_DIRECTION_BOTH
	preview_panel.add_child(preview)
	preview.add_theme_font_size_override("font_size", 13)
	preview_panel.hide()

func _dock_button(caption: String, symbol: String, action: Callable) -> Button:
	var button := _button(caption, action)
	button.icon = _symbol(symbol)
	button.add_theme_constant_override("icon_max_width", 40)
	button.add_theme_font_size_override("font_size", 13)
	button.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
	button.vertical_icon_alignment = VERTICAL_ALIGNMENT_TOP
	button.custom_minimum_size = Vector2(84, 64)
	return button

func _open_catalog() -> void:
	catalog_dialog.popup_centered(Vector2i(560, 320))
	category_picker.grab_focus()

func _overview() -> void:
	var panel := _panel(Control.PRESET_BOTTOM_RIGHT, Rect2(-182, -212, 172, 202))
	panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	var column := VBoxContainer.new()
	panel.add_child(column)
	column.add_theme_constant_override("separation", 4)
	column.add_child(map)
	map.custom_minimum_size = Vector2(154, 154)
	map.focus_mode = Control.FOCUS_ALL
	map.focus_entered.connect(map.queue_redraw)
	map.focus_exited.connect(map.queue_redraw)
	map.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	map.tooltip_text = "Zum Ziel klicken oder mit den Pfeiltasten navigieren"
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
	inspector = _panel(Control.PRESET_TOP_RIGHT, Rect2(-332, 61, 322, 276))
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
	var close := _button("×", _close_inspection)
	close.tooltip_text = "Feldbericht schließen (Esc)"
	headings.add_child(close)
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

func _close_inspection() -> void:
	set_inspection(-1, [])
	inspection_closed.emit()

func _dialogs() -> void:
	surface.add_child(catalog_dialog)
	catalog_dialog.title = "Baukatalog"
	catalog_dialog.ok_button_text = "Schließen"
	var catalog_layout := VBoxContainer.new()
	catalog_dialog.add_child(catalog_layout)
	catalog_layout.add_child(category_picker)
	category_picker.tooltip_text = "Werkzeugkategorie"
	category_picker.item_selected.connect(func(_index: int): _refresh_tools())
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	catalog_layout.add_child(scroll)
	scroll.add_child(tool_list)
	tool_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	tool_list.columns = 3
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
	surface.add_child(credits_dialog)
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
	for dialog: Window in [catalog_dialog, save_dialog, load_dialog, new_dialog, report_dialog, help_dialog, credits_dialog, trade_dialog, survey_dialog, launch_dialog]:
		dialog.theme = surface.theme
		dialog.transient = true
		dialog.exclusive = true
		dialog.about_to_popup.connect(func(): modal_opened.emit())

func _menu_action(id: int) -> void:
	match id:
		0: new_dialog.popup_centered(Vector2i(365, 180))
		1: load_dialog.popup_centered_ratio(0.75)
		2: save_dialog.popup_centered_ratio(0.75)
		3: help_dialog.popup_centered(Vector2i(660, 540))
		8: credits_dialog.popup_centered(Vector2i(710, 550))
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
	for id: int in quick_buttons:
		quick_buttons[id].disabled = not catalog.has(id)
	_sync_selection()
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
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.set_meta("tool_id", id)

func _sync_selection() -> void:
	for id: int in quick_buttons:
		quick_buttons[id].set_pressed_no_signal(id == chosen_tool)
	for button: Button in tool_list.get_children():
		button.set_pressed_no_signal(button.get_meta("tool_id") == chosen_tool)
	catalog_button.text = "Baukatalog" if chosen_tool in quick_buttons else Catalog.tool_name(chosen_tool, "Baukatalog")
	catalog_button.icon = _symbol("catalog") if chosen_tool in quick_buttons or not Catalog.TOOLS.has(chosen_tool) else load("res://ui/icons/%s.png" % Catalog.TOOLS[chosen_tool][1])

func _select_tool(id: int) -> void:
	if not catalog.has(id): return
	chosen_tool = id
	if catalog_dialog.visible: catalog_dialog.hide()
	if id != -1: _close_inspection()
	_sync_selection()
	show_preview("")
	tool_changed.emit(id)

func has_modal() -> bool:
	var active := get_window().get_last_exclusive_window()
	return active != null and active != get_window()

func _shortcut_input(event: InputEvent) -> void:
	if has_modal() or not event is InputEventKey: return
	if not event.pressed or event.echo: return
	match event.keycode:
		KEY_1, KEY_2, KEY_3, KEY_4, KEY_5:
			_select_tool(QUICK_TOOLS[event.keycode - KEY_1][0])
		KEY_B: _open_catalog()
		KEY_F1: _menu_action(3)
		KEY_ESCAPE:
			if inspector.visible: _close_inspection()
			else: _select_tool(-1)
		_: return
	get_viewport().set_input_as_handled()

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
	inspector.show()
	if details.text != text:
		details.text = text

func show_preview(text: String) -> void:
	preview_panel.visible = not text.is_empty()
	preview.text = text

func set_speed(speed: int) -> void:
	if speed_buttons.has(speed): speed_buttons[speed].button_pressed = true

func set_focus(cell: Vector2i, bearing: float) -> void:
	map.focus = cell
	map.bearing = bearing
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
