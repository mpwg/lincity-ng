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
signal modal_closed
signal quit_requested
signal history_requested
signal pause_requested

const Catalog = preload("res://ui/catalog.gd")
const Minimap = preload("res://ui/minimap.gd")
const PAPER := Color("#f5f2e9")
const MUTED := Color("#c4c8c0")
const ACCENT := Color("#e3c78a")
const PANEL := Color("#202321ee")
const EDGE := Color("#73766a")
const MIN_WINDOW_SIZE := preload("res://ui/settings.gd").MIN_WINDOW_SIZE
const MIN_UI_SCALE := 0.75
const COMPACT_LAYOUT_MIN_WIDTH := 1320
const COMPACT_SIDEBAR_WIDTH := 296
const SUPPLY_REPORT := 16
const QUICK_TOOLS := [
	[-1, "Untersuchen", "inspect"], [11, "Straße", "road"], [4, "Wohnen", "housing"],
	[6, "Markt", "market"], [-2, "Abriss", "demolish"],
]
const MONTHS := ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"]
const LAYERS := ["Stadtplan", "Luftbelastung", "Arbeitsmarkt", "Ernährung", "Stromversorgung", "Feuerwehr", "Sport", "Gesundheit", "Kohlevorräte", "Verkehr", "Warenbestand"]
const GOODS := ["Lebensmittel", "Arbeitskräfte", "Kohle", "Waren", "Erz", "Stahl", "Abfall", "Niederspannung", "Hochspannung", "Wasser"]
const LAYER_KEYS := {KEY_QUOTELEFT: 0, KEY_1: 3, KEY_2: 2, KEY_3: 4, KEY_4: 5,
	KEY_5: 6, KEY_6: 7, KEY_7: 9, KEY_8: 1, KEY_9: 8, KEY_0: 10}
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

const QUICK_SAVE := "user://Schnellspeicherung.gz"
const OUTCOMES := {
	"economy": ["Sieg durch Nachhaltigkeit", "Deine Stadt hat alle Nachhaltigkeitsziele erreicht. Ihre Wirtschaft kann auf Dauer bestehen."],
	"evacuation": ["Sieg durch Evakuierung", "Du hast die letzten Einwohnerinnen und Einwohner erfolgreich ins All evakuiert."],
	"loss": ["Stadt verloren", "Die Bevölkerung deiner Stadt ist gestorben."],
}
const ROCKET_EVENTS := {
	"rocket_ready": "Eine Rakete ist startbereit.",
	"rocket_crash": "Die Rakete ist abgestürzt.",
	"rocket_success": "Der Testflug war erfolgreich.",
	"rocket_evacuation": "1.000 Menschen wurden evakuiert.",
}

var outcome_dialog := AcceptDialog.new()
var demolition_dialog := ConfirmationDialog.new()
var quit_dialog := ConfirmationDialog.new()
var history_dialog := preload("res://ui/history.gd").new()
var settings_dialog := preload("res://ui/settings_dialog.gd").new()
var settings: RefCounted
var surface := Control.new()
var game_surface := Control.new()
var session_menu := preload("res://ui/session_menu.gd").new()
var treasury := Label.new()
var inhabitants := Label.new()
var technology := Label.new()
var supply_button: Button
var warning_panel: PanelContainer
var warning_text := Label.new()
var warning_focus: Button
var warning_cell := Vector2i(-1, -1)
var date := Label.new()
var speed_buttons := {}
var quick_buttons := {}
var stats_panel: PanelContainer
var clock_panel: PanelContainer
var catalog_panel: PanelContainer
var category_row := HBoxContainer.new()
var category_second_row := HBoxContainer.new()
var category_buttons := {}
var active_category := ""
var catalog_heading := Label.new()
var preview_panel: PanelContainer
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
var city_report := preload("res://ui/city_report.gd").new()
var report_export: Button
var report_save_dialog := FileDialog.new()
var report_export_status := Label.new()
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
var compact_layout := false

func _ready() -> void:
	add_child(surface)
	surface.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	surface.mouse_filter = Control.MOUSE_FILTER_IGNORE
	surface.theme = _theme()
	surface.add_child(game_surface)
	game_surface.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	game_surface.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_top_bar()
	_palette()
	_overview()
	_footer()
	_dialogs()
	_refresh_categories()
	surface.add_child(session_menu)
	get_window().size_changed.connect(_scale_interface)
	surface.resized.connect(_layout_interface)
	_scale_interface()

func _scale_interface() -> void:
	var window := get_window()
	var design_size := Vector2i(
		ProjectSettings.get_setting("display/window/size/viewport_width"),
		ProjectSettings.get_setting("display/window/size/viewport_height"))
	var readable_size := Vector2i(Vector2(window.size.max(MIN_WINDOW_SIZE)) / MIN_UI_SCALE)
	var viewport_size := design_size.min(readable_size)
	if viewport_size.x > 0 and viewport_size.y > 0 and window.content_scale_size != viewport_size:
		window.content_scale_size = viewport_size
	_layout_interface()

func _layout_interface() -> void:
	var use_compact := surface.size.x < COMPACT_LAYOUT_MIN_WIDTH
	if compact_layout == use_compact:
		return
	compact_layout = use_compact
	_layout_categories()
	tool_list.columns = 2 if compact_layout else 3
	for button: Button in quick_buttons.values():
		button.add_theme_font_size_override("font_size", 15 if compact_layout else 13)
	for button: Button in speed_buttons.values():
		button.add_theme_font_size_override("font_size", 15 if compact_layout else 12)
	for button: Button in tool_list.get_children():
		_layout_tool_button(button)
	details.add_theme_font_size_override("normal_font_size", 15 if compact_layout else 14)
	preview.add_theme_font_size_override("font_size", 15 if compact_layout else 13)
	preview.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART if compact_layout else TextServer.AUTOWRAP_OFF
	preview_panel.grow_vertical = Control.GROW_DIRECTION_BEGIN if compact_layout else Control.GROW_DIRECTION_END
	warning_text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART if compact_layout else TextServer.AUTOWRAP_OFF
	warning_text.size_flags_horizontal = Control.SIZE_EXPAND_FILL if compact_layout else Control.SIZE_FILL
	supply_button.clip_text = compact_layout
	supply_button.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS if compact_layout else TextServer.OVERRUN_NO_TRIMMING
	supply_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL if compact_layout else Control.SIZE_FILL
	supply_button.tooltip_text = supply_button.text if compact_layout else tr("Warenversorgung")
	if compact_layout:
		_place_panel(stats_panel, Control.PRESET_TOP_WIDE, Rect2(10, 9, -20, 36))
		_place_panel(clock_panel, Control.PRESET_TOP_LEFT, Rect2(10, 56, 0, 36))
		clock_panel.grow_horizontal = Control.GROW_DIRECTION_END
		_place_compact_left(warning_panel, Control.PRESET_TOP_WIDE, 103, 141)
		_place_panel(inspector, Control.PRESET_RIGHT_WIDE, Rect2(-286, 56, 276, -310))
		_place_compact_left(palette, Control.PRESET_BOTTOM_WIDE, -190, -10)
		_place_compact_left(catalog_panel, Control.PRESET_BOTTOM_WIDE, -432, -200)
		_place_compact_left(preview_panel, Control.PRESET_BOTTOM_WIDE, -236, -200)
	else:
		_place_panel(stats_panel, Control.PRESET_TOP_LEFT, Rect2(10, 9, 0, 36))
		_place_panel(clock_panel, Control.PRESET_TOP_RIGHT, Rect2(-10, 9, 0, 36))
		clock_panel.grow_horizontal = Control.GROW_DIRECTION_BEGIN
		_place_panel(warning_panel, Control.PRESET_TOP_LEFT, Rect2(10, 57, 0, 38))
		_place_panel(inspector, Control.PRESET_TOP_RIGHT, Rect2(-332, 61, 322, 276))
		_place_panel(palette, Control.PRESET_CENTER_BOTTOM, Rect2(0, -123, 0, 113))
		_place_panel(catalog_panel, Control.PRESET_CENTER_BOTTOM, Rect2(-290, -367, 580, 235))
		_place_panel(preview_panel, Control.PRESET_CENTER_BOTTOM, Rect2(0, -174, 0, 35))

func _place_compact_left(panel: PanelContainer, preset: Control.LayoutPreset, top: float, bottom: float) -> void:
	_place_panel(panel, preset, Rect2(10, top, -COMPACT_SIDEBAR_WIDTH - 10, bottom - top))

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
	game_surface.add_child(panel)
	_place_panel(panel, preset, bounds)
	var skin := _box(PANEL, 6, 5, 6)
	skin.set_border_width_all(1)
	skin.border_color = EDGE
	skin.shadow_color = Color("#10141066")
	skin.shadow_size = 3
	panel.add_theme_stylebox_override("panel", skin)
	return panel

func _place_panel(panel: PanelContainer, preset: Control.LayoutPreset, bounds: Rect2) -> void:
	panel.set_anchors_preset(preset)
	panel.offset_left = bounds.position.x
	panel.offset_top = bounds.position.y
	panel.offset_right = bounds.end.x
	panel.offset_bottom = bounds.end.y

func _label(value: String, font_size: int, color: Color = PAPER) -> Label:
	var label := Label.new()
	label.text = tr(value)
	label.add_theme_font_size_override("font_size", font_size)
	label.add_theme_color_override("font_color", color)
	return label

func _button(title: String, callback: Callable) -> Button:
	var button := Button.new()
	button.text = tr(title)
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
	stats_panel = _panel(Control.PRESET_TOP_LEFT, Rect2(10, 9, 0, 36))
	var row := HBoxContainer.new()
	stats_panel.add_child(row)
	_metric(row, "Bevölkerung", inhabitants, "people")
	row.add_child(VSeparator.new())
	_metric(row, "Stadtkasse", treasury, "coins")
	row.add_child(VSeparator.new())
	row.add_child(_label("Technologie", 14, MUTED))
	technology.text = "—"
	row.add_child(technology)
	supply_button = _button("Warenversorgung", _open_report.bind(SUPPLY_REPORT))
	supply_button.tooltip_text = tr("Warenversorgung")
	supply_button.tooltip_text = tr("Warenversorgung")
	supply_button.accessibility_name = tr("Warenversorgung")
	row.add_child(supply_button)
	_warning_banner()
	clock_panel = _panel(Control.PRESET_TOP_RIGHT, Rect2(-10, 9, 0, 36))
	clock_panel.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	var clock := HBoxContainer.new()
	clock.add_theme_constant_override("separation", 4)
	clock_panel.add_child(clock)
	var speeds := ButtonGroup.new()
	for item in [[0, "Ⅱ", "Pause"], [1, "▶", "Normales Tempo"], [2, "▶▶", "Hohes Tempo"], [4, "▶▶▶", "Schnelllauf"]]:
		var speed: int = item[0]
		var button := _button(item[1], speed_changed.emit.bind(speed))
		button.tooltip_text = tr(item[2])
		button.add_theme_font_size_override("font_size", 12)
		button.button_group = speeds
		button.toggle_mode = true
		button.button_pressed = speed == 1
		speed_buttons[speed] = button
		clock.add_child(button)
	clock.add_child(VSeparator.new())
	date.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	date.custom_minimum_size.x = 110
	date.tooltip_text = tr("Spielzeit")
	clock.add_child(date)
	var menu := MenuButton.new()
	menu.text = tr("Spiel")
	menu.flat = false
	clock.add_child(menu)
	var popup := menu.get_popup()
	popup.add_item(tr("Pausenmenü (Esc)"), 15)
	popup.add_separator()
	popup.add_item(tr("Neue Stadt oder Szenario …"), 0)
	popup.add_item(tr("Spielstand laden …"), 1)
	popup.add_item(tr("Spielstand speichern …"), 2)
	popup.add_item(tr("Schnellladen (F9)"), 12)
	popup.add_item(tr("Schnellspeichern (F12)"), 13)
	popup.add_separator()
	popup.add_item(tr("Stadtstatistik"), 4)
	popup.add_item(tr("Jahresabschluss"), 5)
	popup.add_item(tr("Verlauf und Nachhaltigkeit …"), 10)
	popup.add_item(tr("Ereignisse"), 6)
	popup.add_item(tr("Kohlevorkommen erkunden …"), 7)
	popup.add_separator()
	popup.add_item(tr("Spielhilfe"), 3)
	popup.add_item(tr("Mitwirkende und Lizenzen …"), 8)
	popup.add_separator()
	popup.add_item(tr("Einstellungen …"), 9)
	popup.add_item(tr("Beenden …"), 14)
	popup.id_pressed.connect(_menu_action)

func _palette() -> void:
	palette = _panel(Control.PRESET_CENTER_BOTTOM, Rect2(0, -123, 0, 113))
	palette.grow_horizontal = Control.GROW_DIRECTION_BOTH
	palette.grow_vertical = Control.GROW_DIRECTION_BEGIN
	var tools := VBoxContainer.new()
	tools.add_theme_constant_override("separation", 5)
	palette.add_child(tools)
	var row := HBoxContainer.new()
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	row.add_theme_constant_override("separation", 4)
	tools.add_child(row)
	for item in QUICK_TOOLS:
		var id: int = item[0]
		var button := _dock_button(item[1], item[2], _select_tool.bind(id))
		button.toggle_mode = true
		button.tooltip_text = tr("%s (Umschalt+%d)") % [Catalog.tool_name(id), quick_buttons.size() + 1]
		row.add_child(button)
		quick_buttons[id] = button
	category_row.alignment = BoxContainer.ALIGNMENT_CENTER
	category_row.add_theme_constant_override("separation", 4)
	tools.add_child(category_row)
	category_second_row.alignment = BoxContainer.ALIGNMENT_CENTER
	category_second_row.add_theme_constant_override("separation", 4)
	tools.add_child(category_second_row)
	category_second_row.hide()
	catalog_panel = _panel(Control.PRESET_CENTER_BOTTOM, Rect2(-290, -367, 580, 235))
	catalog_panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	var catalog_layout := VBoxContainer.new()
	catalog_panel.add_child(catalog_layout)
	catalog_layout.add_child(catalog_heading)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	catalog_layout.add_child(scroll)
	scroll.add_child(tool_list)
	tool_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	tool_list.columns = 3
	catalog_panel.hide()
	preview_panel = _panel(Control.PRESET_CENTER_BOTTOM, Rect2(0, -174, 0, 35))
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

func _toggle_category(category: String) -> void:
	if catalog_panel.visible and active_category == category:
		close_catalog()
		return
	active_category = category
	catalog_heading.text = tr(category)
	_refresh_tools()
	catalog_panel.show()
	preview_panel.hide()
	_sync_selection()

func close_catalog() -> void:
	catalog_panel.hide()
	_sync_selection()

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
	map.tooltip_text = tr("Zum Ziel klicken oder mit den Pfeiltasten navigieren")
	map.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	map.cell_selected.connect(func(cell: Vector2i): focus_requested.emit(cell))
	for layer_name in LAYERS: layers.add_item(tr(layer_name))
	column.add_child(layers)
	layers.tooltip_text = tr("Kartenebene auswählen")
	for good in GOODS: commodities.add_item(tr(good))
	column.add_child(commodities)
	commodities.hide()
	layers.item_selected.connect(_activate_layer)
	commodities.item_selected.connect(func(good: int): layer_selected.emit(layers.selected, good))

func _activate_layer(id: int) -> void:
	layers.select(id)
	commodities.visible = id == 10
	overlay_changed.emit(id == 1)
	layer_selected.emit(id, commodities.selected)

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
	close.tooltip_text = tr("Feldbericht schließen (Esc)")
	headings.add_child(close)
	production.text = tr("Produktion")
	production.toggled.connect(func(active: bool): report_requested.emit(active))
	column.add_child(details)
	details.size_flags_vertical = Control.SIZE_EXPAND_FILL
	details.selection_enabled = true
	details.add_theme_color_override("default_color", PAPER)
	details.add_theme_font_size_override("normal_font_size", 14)
	details.text = tr("Mit „Untersuchen“ ein Feld auf der Karte auswählen.")

	column.add_child(trade_button)
	trade_button.text = tr("Handel einstellen …")
	trade_button.pressed.connect(_open_trade)
	column.add_child(launch_button)
	launch_button.text = tr("Rakete starten …")
	launch_button.pressed.connect(func(): launch_dialog.popup_centered())
	set_inspection(-1, [])

func _close_inspection() -> void:
	set_inspection(-1, [])
	inspection_closed.emit()

func _dialogs() -> void:
	surface.add_child(history_dialog)
	surface.add_child(settings_dialog)
	surface.add_child(outcome_dialog)
	outcome_dialog.ok_button_text = tr("Weiterbauen")
	surface.add_child(demolition_dialog)
	demolition_dialog.title = tr("Teuren Abriss bestätigen")
	demolition_dialog.ok_button_text = tr("Abreißen")
	demolition_dialog.cancel_button_text = tr("Abbrechen")
	surface.add_child(quit_dialog)
	quit_dialog.title = tr("Spiel beenden")
	quit_dialog.dialog_text = tr("LinCity-NG schließen?")
	quit_dialog.ok_button_text = tr("Beenden")
	quit_dialog.cancel_button_text = tr("Abbrechen")
	quit_dialog.confirmed.connect(quit_requested.emit)
	for dialog: FileDialog in [save_dialog, load_dialog]:
		surface.add_child(dialog)
		dialog.access = FileDialog.ACCESS_FILESYSTEM
		dialog.transient = true
		dialog.current_dir = ProjectSettings.globalize_path("user://")
		dialog.add_filter("*.gz", tr("LinCity-Spielstände"))
	save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	save_dialog.title = tr("Stadt speichern")
	save_dialog.current_file = "Meine Stadt.gz"
	save_dialog.file_selected.connect(func(path: String): save_requested.emit(path))
	load_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	load_dialog.title = tr("Stadt laden")
	load_dialog.file_selected.connect(func(path: String): load_requested.emit(path))
	surface.add_child(new_dialog)
	new_dialog.title = tr("Neue Stadt")
	new_dialog.ok_button_text = tr("Starten")
	new_dialog.cancel_button_text = tr("Abbrechen")
	var content := VBoxContainer.new()
	new_dialog.add_child(content)
	content.add_child(_label("Ausgangslage wählen", 19))
	content.add_child(scenarios)
	scenarios.add_item(tr("Neue Landschaft"))
	scenarios.add_item(tr("Kleinstadt am Fluss"))
	scenarios.set_item_metadata(1, "res://scenarios/am_fluss.scn.gz")
	for file: String in SCENARIOS:
		scenarios.add_item(tr(SCENARIOS[file]))
		scenarios.set_item_metadata(scenarios.item_count - 1, "res://opening/" + file)
	var options := VBoxContainer.new()
	content.add_child(options)
	options.add_child(_label("Landschaft", 14, MUTED))
	options.add_child(climate)
	for region in ["Flusslandschaft", "Wüste", "Gemäßigte Zone", "Sumpfland"]: climate.add_item(tr(region))
	options.add_child(_label("Kantenlänge in Feldern", 14, MUTED))
	options.add_child(extent)
	extent.min_value = 50
	extent.max_value = 10000
	extent.step = 25
	extent.value = 100
	village.text = tr("Mit Startdorf")
	village.button_pressed = true
	options.add_child(village)
	trees.text = tr("Mit Bäumen")
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
	var report_content := VBoxContainer.new()
	report_dialog.add_child(report_content)
	report_content.add_child(report_body)
	report_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	report_body.selection_enabled = true
	report_body.add_theme_font_size_override("normal_font_size", 16)
	report_dialog.ok_button_text = tr("Schließen")
	report_content.add_child(report_export_status)
	report_export_status.hide()
	report_export = report_dialog.add_button(tr("Statistik exportieren …"), true)
	report_export.pressed.connect(func(): report_save_dialog.popup_centered_ratio(0.75))
	report_dialog.add_child(report_save_dialog)
	report_save_dialog.title = tr("Stadtstatistik exportieren")
	report_save_dialog.theme = surface.theme
	report_save_dialog.transient = true
	report_save_dialog.exclusive = true
	report_save_dialog.access = FileDialog.ACCESS_FILESYSTEM
	report_save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	report_save_dialog.add_filter("*.txt", tr("Textdateien"))
	report_save_dialog.current_file = tr("Stadtstatistik.txt")
	report_save_dialog.file_selected.connect(_export_report)
	surface.add_child(trade_dialog)
	trade_dialog.title = tr("Handelsregeln")
	trade_dialog.ok_button_text = tr("Schließen")
	trade_dialog.add_child(trade_list)
	surface.add_child(survey_dialog)
	survey_dialog.title = tr("Kohlevorkommen erkunden")
	survey_dialog.dialog_text = tr("Die Erkundung kostet 1.000.000 $. Danach zeigt die Kohlekarte die Lagerstätten.")
	survey_dialog.ok_button_text = tr("Erkunden")
	survey_dialog.cancel_button_text = tr("Abbrechen")
	survey_dialog.confirmed.connect(func(): survey_requested.emit())
	surface.add_child(launch_dialog)
	launch_dialog.title = tr("Raketenstart")
	launch_dialog.dialog_text = tr("Soll die vorbereitete Rakete starten?")
	launch_dialog.ok_button_text = tr("Starten")
	launch_dialog.cancel_button_text = tr("Abbrechen")
	launch_dialog.confirmed.connect(func(): launch_requested.emit())
	for dialog: Window in [save_dialog, load_dialog, new_dialog, report_dialog, help_dialog, credits_dialog, trade_dialog, survey_dialog, launch_dialog, outcome_dialog, demolition_dialog, quit_dialog, settings_dialog, history_dialog]:
		dialog.theme = surface.theme
		dialog.transient = true
		dialog.exclusive = true
		dialog.about_to_popup.connect(func(): modal_opened.emit())
		dialog.visibility_changed.connect(func():
			if not dialog.visible: modal_closed.emit())

func _menu_action(id: int) -> void:
	match id:
		0:
			new_dialog.dialog_text = "" if session_menu.visible and not session_menu.in_game else tr("Die aktuelle Stadt wird ersetzt.")
			if settings:
				climate.select(settings.values.map_climate)
				extent.value = settings.values.map_side
				village.button_pressed = settings.values.map_village
				trees.button_pressed = settings.values.map_trees
			new_dialog.popup_centered(Vector2i(365, 180))
		1: load_dialog.popup_centered_ratio(0.75)
		2: save_dialog.popup_centered_ratio(0.75)
		3: help_dialog.popup_centered(Vector2i(660, 540))
		8: credits_dialog.popup_centered(Vector2i(710, 550))
		9: settings_dialog.popup_centered(Vector2i(590, 405))
		10: history_requested.emit()
		12:
			if FileAccess.file_exists(QUICK_SAVE):
				load_requested.emit(QUICK_SAVE)
			else:
				show_info(tr("Es gibt noch keine Schnellspeicherung."))
		13: save_requested.emit(QUICK_SAVE)
		14: quit_dialog.popup_centered(Vector2i(380, 155))
		15: pause_requested.emit()
		4, 5, 6:
			_open_report(id)
		7:
			if city.get("coal_survey_done", false):
				show_info(tr("Die Kohlevorkommen sind bereits erkundet. Wähle die Kartenebene „Kohlevorräte“."))
			else:
				survey_dialog.popup_centered()

func setup(tools: Array) -> void:
	chosen_tool = -1
	events.clear()
	if warning_panel:
		warning_panel.hide()
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
	close_catalog()
	category_buttons.clear()
	for row: HBoxContainer in [category_row, category_second_row]:
		for child in row.get_children():
			row.remove_child(child)
			child.queue_free()
	for category: String in categories:
		var button := _button(category, _toggle_category.bind(category))
		button.toggle_mode = true
		category_buttons[category] = button
		category_row.add_child(button)
	_layout_categories()
	for id: int in quick_buttons:
		quick_buttons[id].disabled = not catalog.has(id)
	_sync_selection()

func _layout_categories() -> void:
	var index := 0
	for button: Button in category_buttons.values():
		var row := category_second_row if compact_layout and index >= 4 else category_row
		if button.get_parent() != row:
			var had_focus := button.has_focus()
			button.reparent(row, false)
			row.show()
			if had_focus:
				button.grab_focus()
		index += 1
	category_second_row.visible = category_second_row.get_child_count() > 0

func _refresh_tools() -> void:
	for child in tool_list.get_children():
		tool_list.remove_child(child)
		child.queue_free()
	var group := ButtonGroup.new()
	for id: int in categories[active_category]:
		var tool: Dictionary = catalog[id]
		var tool_name := Catalog.tool_name(id, str(tool.get("name", id)))
		var button := _button(tool_name, _select_tool.bind(id))
		if Catalog.TOOLS.has(id):
			button.icon = load("res://ui/icons/%s.png" % Catalog.TOOLS[id][1])
			button.add_theme_constant_override("icon_max_width", 26)
		button.toggle_mode = true
		button.button_group = group
		button.button_pressed = id == chosen_tool
		button.alignment = HORIZONTAL_ALIGNMENT_CENTER
		button.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
		button.vertical_icon_alignment = VERTICAL_ALIGNMENT_TOP
		var footprint := int(tool.get("size", 1))
		var tech := int(tool.get("required_tech", 0))
		var cost := int(tool.get("base_cost", 0))
		var footprint_label := tr("%d × %d Felder") % [footprint, footprint]
		var cost_label := tr("Grundpreis %s $") % _number(cost) if cost > 0 else tr("Preis je nach Feld")
		if id in [-1, -3]:
			cost_label = tr("Kostenlos")
		var tech_label := tr("%.1f %% Technologie") % (tech / 10000.0)
		button.set_meta("tool_details", [tool_name, footprint_label, cost_label, tech_label])
		_layout_tool_button(button)
		button.set_meta("required_tech", tech)
		button.disabled = int(city.get("tech", 0)) < tech
		tool_list.add_child(button)
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.set_meta("tool_id", id)

func _layout_tool_button(button: Button) -> void:
	var format := "%s\n%s\n%s\n%s" if compact_layout else "%s\n%s · %s\n%s"
	button.text = format % button.get_meta("tool_details")
	button.tooltip_text = button.text
	button.add_theme_font_size_override("font_size", 15 if compact_layout else 13)

func _sync_selection() -> void:
	for id: int in quick_buttons:
		quick_buttons[id].set_pressed_no_signal(id == chosen_tool)
	for button: Button in tool_list.get_children():
		button.set_pressed_no_signal(button.get_meta("tool_id") == chosen_tool)
	for category: String in category_buttons:
		var selected: bool = active_category == category if catalog_panel.visible else chosen_tool in categories[category] and not quick_buttons.has(chosen_tool)
		category_buttons[category].set_pressed_no_signal(selected)

func _select_tool(id: int) -> void:
	if not catalog.has(id): return
	chosen_tool = id
	close_catalog()
	if id != -1: _close_inspection()
	_sync_selection()
	show_preview("")
	tool_changed.emit(id)

func has_modal() -> bool:
	var active := get_window().get_last_exclusive_window()
	return active != null and active != get_window()

func open_session_menu(in_game: bool, has_save: bool) -> void:
	close_catalog()
	game_surface.hide()
	session_menu.open(in_game, has_save)

func close_session_menu() -> void:
	session_menu.hide()
	game_surface.show()

func _shortcut_input(event: InputEvent) -> void:
	if has_modal() or not event is InputEventKey: return
	if not event.pressed or event.echo: return
	if session_menu.visible:
		if event.keycode == KEY_ESCAPE and session_menu.in_game:
			session_menu.action_requested.emit(&"resume")
			get_viewport().set_input_as_handled()
		return
	var layer_key: Key = KEY_QUOTELEFT if event.physical_keycode == KEY_QUOTELEFT else event.keycode
	if not event.shift_pressed and LAYER_KEYS.has(layer_key):
		_activate_layer(LAYER_KEYS[layer_key])
		get_viewport().set_input_as_handled()
		return
	match event.keycode:
		KEY_1, KEY_2, KEY_3, KEY_4, KEY_5:
			if not event.shift_pressed: return
			_select_tool(QUICK_TOOLS[event.keycode - KEY_1][0])
		KEY_B: _select_tool(-1 if chosen_tool == -2 else -2)
		KEY_F1: _menu_action(3)
		KEY_F9: _menu_action(12)
		KEY_F12: _menu_action(13)
		KEY_ESCAPE:
			if catalog_panel.visible: close_catalog()
			elif inspector.visible: _close_inspection()
			else: pause_requested.emit()
		_: return
	get_viewport().set_input_as_handled()

func _number(value: int) -> String:
	var digits := str(absi(value))
	var separator := "." if TranslationServer.get_locale().begins_with("de") else ","
	var i := digits.length() - 3
	while i > 0:
		digits = digits.insert(i, separator)
		i -= 3
	return ("−" if value < 0 else "") + digits

func update_city(snapshot: Dictionary) -> void:
	if catalog_panel.visible and snapshot.get("tech", 0) != city.get("tech", 0):
		for button: Button in tool_list.get_children():
			button.disabled = int(snapshot.get("tech", 0)) < int(button.get_meta("required_tech"))
	city = snapshot
	var money := int(snapshot.get("money", 0))
	treasury.text = "%s $" % _number(money)
	treasury.add_theme_color_override("font_color", Color("#ffab95") if money < 0 else PAPER)
	inhabitants.text = _number(int(snapshot.get("population", 0)))
	technology.text = "%.2f %%" % float(snapshot.get("tech_percent", 0.0))
	var inventory: Array = snapshot.get("inventory", [])
	if inventory.size() == GOODS.size():
		var essentials := PackedStringArray()
		for i in [0, 9]:
			var stock: Dictionary = inventory[i]
			var fill := "—" if stock.capacity <= 0 else "%.0f %%" % (100.0 * stock.amount / stock.capacity)
			essentials.append("%s %s" % [tr(GOODS[i]), fill])
		supply_button.text = " · ".join(essentials)
		if compact_layout:
			supply_button.tooltip_text = supply_button.text
	var day := int(snapshot.get("time", 0))
	date.text = "%s %d" % [tr(MONTHS[(day / 100) % 12]), day / 1200]
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
	preview_panel.visible = not text.is_empty() and not catalog_panel.visible
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

func show_event(event: Dictionary) -> void:
	var kind: String = event.get("kind", "")
	var outcome: Array = OUTCOMES.get(kind, [])
	var text := str(tr(ROCKET_EVENTS[kind])) if ROCKET_EVENTS.has(kind) else str(event.get("text", ""))
	var warning := ""
	var cell := Vector2i(-1, -1)
	match kind:
		"fire":
			cell = event.get("cell", cell)
			var building := Catalog.tool_name(int(event.get("group", -1)))
			if building.is_empty():
				warning = tr("Feuer bei Feld (%d, %d).") % [cell.x, cell.y]
			else:
				warning = tr("Feuer bei Feld (%d, %d) · %s") % [cell.x, cell.y, building]
		"out_of_money":
			warning = tr("Die Stadtkasse ist leer.")
		"out_of_credit":
			warning = tr("Das verfügbare Kreditlimit reicht nicht aus.")
	if not outcome.is_empty():
		outcome_dialog.title = tr(outcome[0])
		outcome_dialog.dialog_text = tr(outcome[1])
		outcome_dialog.popup_centered(Vector2i(460, 170))
		text = "%s – %s" % [outcome_dialog.title, outcome_dialog.dialog_text]
	if not warning.is_empty():
		text = warning
		warning_cell = cell
		warning_text.text = warning
		warning_focus.visible = cell.x >= 0 and cell.y >= 0
		warning_panel.show()
	if text.is_empty(): return
	events.append(text)
	if events.size() > 256: events.remove_at(0)
	if warning.is_empty(): show_info(text)
	if report_dialog.visible and current_report == 6: _refresh_report()

func _warning_banner() -> void:
	warning_panel = _panel(Control.PRESET_TOP_LEFT, Rect2(10, 57, 0, 38))
	var skin := _box(Color("#3b241c"), 9, 5, 6)
	skin.set_border_width_all(1)
	skin.border_color = Color("#e79a75")
	warning_panel.add_theme_stylebox_override("panel", skin)
	var row := HBoxContainer.new()
	warning_panel.add_child(row)
	warning_text.add_theme_color_override("font_color", Color("#ffe4d6"))
	row.add_child(warning_text)
	warning_focus = _button("Zum Ort", func(): focus_requested.emit(warning_cell))
	row.add_child(warning_focus)
	var dismiss := _button("×", warning_panel.hide)
	dismiss.tooltip_text = tr("Warnung schließen")
	row.add_child(dismiss)
	warning_panel.hide()

func _open_report(id: int) -> void:
	current_report = id
	report_export_status.hide()
	_refresh_report()
	report_dialog.popup_centered(Vector2i(510, 450))

func _export_report(path: String) -> void:
	var file := FileAccess.open(path, FileAccess.WRITE)
	var written := false
	if file:
		written = file.store_string(report_dialog.title + "\n\n" + report_body.text + "\n")
		file.flush()
		written = written and file.get_error() == OK
	report_export_status.text = tr("Stadtstatistik exportiert.") if written else tr("Die Statistikdatei konnte nicht geschrieben werden.")
	report_export_status.show()

func _refresh_report() -> void:
	var lines := PackedStringArray()
	report_export.visible = current_report == 4
	match current_report:
		4:
			report_dialog.title = tr("Stadtstatistik")
			lines = city_report.statistics(city, date.text, _number)
		SUPPLY_REPORT:
			report_dialog.title = tr("Warenversorgung")
			lines = city_report.supply(city.get("inventory", []), GOODS, _number)
		5:
			report_dialog.title = tr("Jahresabschluss · Vorjahr")
			var finances: Dictionary = city.get("finances", {})
			for category: String in ["income", "expenses"]:
				lines.append(tr("Einnahmen") if category == "income" else tr("Ausgaben"))
				var total := 0
				var accounts: Dictionary = finances.get(category, {})
				for item: String in accounts:
					total += int(accounts[item])
					lines.append("    %s   %s $" % [tr(ACCOUNTS.get(item, item)), _number(accounts[item])])
				lines.append(tr("Summe   %s $") % _number(total) + "\n")
		6:
			report_dialog.title = tr("Ereignisse")
			for i in range(events.size() - 1, -1, -1): lines.append(events[i])
			if lines.is_empty(): lines.append(tr("Bisher sind keine Ereignisse eingegangen."))
	report_body.text = "\n".join(lines)
