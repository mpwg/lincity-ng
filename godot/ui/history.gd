extends AcceptDialog

const Graph = preload("res://ui/history_graph.gd")
const COLOURS := [Color("#e8c882"), Color("#84cdef"), Color("#f29b8a"), Color("#cba9ef")]
const CHARTS := {
	"people": [["population", "Einwohner"], ["homeless", "Wohnungslose"], ["unemployed", "Arbeitslose"], ["starving", "Hungernde"]],
	"vital": [["births", "Geburten"], ["deaths", "Sterbefälle"], ["unnatural_deaths", "Davon unnatürlich"]],
	"housing": [["occupancy", "Wohnraumbelegung"]],
	"balance": [["balance", "Kassenstand"]],
	"tech": [["tech", "Technologiestand"]],
	"pollution": [["pollution", "Luftverschmutzung"]],
}
const CRITERION_HELP := [
	"Kohle- und Erzvorkommen schonen.",
	"Ohne Hafenimporte und -exporte auskommen.",
	"Den Kassenstand von Jahr zu Jahr halten oder steigern.",
	"Eine ausreichend große Bevölkerung stabil halten.",
	"Ein ausreichend hohes Technologieniveau weitgehend halten oder steigern.",
	"Die relevanten Gebäude mit Feuerwehrschutz abdecken.",
]

var _tabs := TabBar.new()
var _timeline := VBoxContainer.new()
var _metrics := OptionButton.new()
var _period := OptionButton.new()
var _date := OptionButton.new()
var _axis := Label.new()
var _values := RichTextLabel.new()
var _graph := Graph.new()
var _sustainability := VBoxContainer.new()
var _data: Dictionary = {}
var _last := -1
var _first := 0
var _month := -1

func _ready() -> void:
	title = "Stadtstatistik"
	ok_button_text = "Schließen"
	min_size = Vector2i(630, 430)
	var layout := VBoxContainer.new()
	add_child(layout)
	_tabs.name = "HistoryTabs"
	_tabs.focus_mode = Control.FOCUS_ALL
	for tab in ["Bevölkerung", "Finanzen", "Technologie", "Umwelt", "Waren", "Nachhaltigkeit"]:
		_tabs.add_tab(tab)
	_style_tabs()
	layout.add_child(_tabs)
	layout.add_child(_timeline)
	_timeline.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var controls := HBoxContainer.new()
	_timeline.add_child(controls)
	controls.add_child(_metrics)
	_metrics.accessibility_name = "Kennzahl"
	controls.add_child(_period)
	_period.accessibility_name = "Zeitraum"
	_period.add_item("Letzte 12 Monate", 12)
	_period.add_item("Letzte 5 Jahre", 60)
	_period.add_item("Gesamter Verlauf", 0)
	_period.select(0)
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	controls.add_child(spacer)
	var month_label := Label.new()
	month_label.text = "Monat"
	controls.add_child(month_label)
	_date.name = "HistoryMonths"
	_date.accessibility_name = "Abgeschlossener Monat"
	controls.add_child(_date)
	_timeline.add_child(_axis)
	_timeline.add_child(_graph)
	_graph.custom_minimum_size = Vector2(0, 190)
	_graph.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_graph.accessibility_name = "Monatsdiagramm"
	_timeline.add_child(_values)
	_values.name = "HistoryValues"
	_values.custom_minimum_size = Vector2(0, 43)
	_values.scroll_active = false
	_values.selection_enabled = true
	layout.add_child(_sustainability)
	_sustainability.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_tabs.tab_changed.connect(_show_tab)
	_metrics.item_selected.connect(func(_index: int): _refresh())
	_period.item_selected.connect(func(_index: int): _refresh_dates())
	_date.item_selected.connect(func(_index: int): _select_month(_date.get_selected_id()))
	_graph.month_selected.connect(_select_month)
	_show_tab(_tabs.current_tab)
	_build_sustainability()

func set_history(data: Dictionary) -> void:
	var first_open := _data.is_empty()
	var latest := int(data.get("last_month", -1))
	if _last != latest:
		_month = latest
	_data = data
	_last = latest
	if is_node_ready():
		var series: Dictionary = _data.get("series", {})
		if first_open and series.get("population", PackedFloat64Array()).is_empty() and not series.get("balance", PackedFloat64Array()).is_empty():
			_tabs.current_tab = 1
		else:
			_show_tab(_tabs.current_tab)
		_build_sustainability()

func _show_tab(_tab: int) -> void:
	_timeline.visible = _tabs.current_tab != 5
	_sustainability.visible = not _timeline.visible
	_metrics.clear()
	_metrics.visible = _tabs.current_tab in [0, 4]
	if _tabs.current_tab == 0:
		for label in ["Einwohner", "Geburten und Sterbefälle", "Wohnraum"]:
			_metrics.add_item(label)
	elif _tabs.current_tab == 4:
		for commodity: Dictionary in _data.get("commodities", []):
			_metrics.add_item(commodity.name)
	if _metrics.item_count > 0:
		_metrics.select(0)
	_refresh_dates()

func _refresh_dates() -> void:
	var span := 0
	for values: PackedFloat64Array in _data.get("series", {}).values():
		span = maxi(span, _observed_span(values))
	for commodity: Dictionary in _data.get("commodities", []):
		span = maxi(span, _observed_span(commodity["values"]))
	var period := _period.get_selected_id()
	if period > 0:
		span = mini(span, period)
	_first = _last - span + 1
	_date.clear()
	for month in range(_first, _last + 1):
		_date.add_item(Graph.month_text(month), month)
	_date.disabled = _date.item_count == 0
	if _date.disabled:
		_date.text = "–"
		_month = -1
	else:
		_month = clampi(_month, _first, _last)
		_date.select(_month - _first)
	_refresh()

func _observed_span(values: PackedFloat64Array) -> int:
	for index in range(values.size() - 1, -1, -1):
		if is_finite(values[index]):
			return index + 1
	return 0

func _select_month(month: int) -> void:
	_month = month
	_date.select(month - _first)
	_refresh()

func _refresh() -> void:
	var display: Array[Dictionary] = []
	var chart := ""
	var unit := ""
	match _tabs.current_tab:
		0:
			chart = ["people", "vital", "housing"][maxi(_metrics.selected, 0)]
			unit = "%" if chart == "housing" else ""
			_axis.text = {"people": "Personen im Monatsmittel", "vital": "Personen pro Monat", "housing": "Belegung des Wohnraums (%)"}[chart]
		1:
			chart = "balance"
			unit = "$"
			_axis.text = "Kassenstand zum Monatsende ($)"
		2:
			chart = "tech"
			unit = "%"
			_axis.text = "Technologiestand zum Monatsende (%)"
		3:
			chart = "pollution"
			_axis.text = "Luftverschmutzung auf der gesamten Karte"
		4:
			unit = "%"
			_axis.text = "Lagerfüllgrad im Monatsmittel (%)"
			if _metrics.selected >= 0:
				var commodity: Dictionary = _data.commodities[_metrics.selected]
				display.append({"label": commodity.name, "values": commodity["values"], "color": COLOURS[0]})
	var available: Dictionary = _data.get("series", {})
	for definition: Array in CHARTS.get(chart, []):
		display.append({"label": definition[1], "values": available.get(definition[0], PackedFloat64Array()), "color": COLOURS[display.size()]})
	_graph.update_series(display, _first, _last, _month)
	_values.clear()
	if _month < 0:
		_values.add_text("Für diese Stadt sind noch keine abgeschlossenen Monatswerte verfügbar.")
		return
	for index in display.size():
		var series := display[index]
		if index > 0:
			_values.add_text("\n" if index == 2 else "    ")
		var sample := Graph.sample(series, _last - _month)
		_values.push_color(series.color)
		_values.add_text(series.label + "  ")
		_values.pop()
		_values.add_text((Graph.number_text(sample) + (" " + unit if unit != "" else "")) if is_finite(sample) else "Keine Daten")

func _build_sustainability() -> void:
	for row in _sustainability.get_children():
		_sustainability.remove_child(row)
		row.queue_free()
	var status := Label.new()
	status.text = "Nachhaltigkeitsziel erreicht" if _data.get("sustainable", false) else "Fortschritt zum Nachhaltigkeitssieg"
	_sustainability.add_child(status)
	var grid := GridContainer.new()
	grid.columns = 3
	grid.add_theme_constant_override("h_separation", 12)
	grid.add_theme_constant_override("v_separation", 22)
	_sustainability.add_child(grid)
	var criteria: Array = _data.get("sustainability", [])
	for index in criteria.size():
		var criterion: Dictionary = criteria[index]
		var name_label := Label.new()
		name_label.text = criterion.name
		name_label.tooltip_text = CRITERION_HELP[index]
		name_label.mouse_filter = Control.MOUSE_FILTER_STOP
		grid.add_child(name_label)
		var progress := ProgressBar.new()
		progress.name = "SustainabilityCriterion%d" % index
		progress.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		progress.custom_minimum_size.y = 14
		progress.show_percentage = false
		progress.max_value = criterion.goal
		progress.value = criterion.years
		progress.accessibility_name = criterion.name
		progress.tooltip_text = CRITERION_HELP[index]
		progress.add_theme_stylebox_override("background", _box(Color("#353d37")))
		progress.add_theme_stylebox_override("fill", _box(Color("#cfb77f")))
		grid.add_child(progress)
		var years := Label.new()
		years.text = "%d / %d Jahre" % [criterion.years, criterion.goal]
		grid.add_child(years)

func _style_tabs() -> void:
	_tabs.add_theme_stylebox_override("tab_unselected", _box(Color("#343835"), 9, 7))
	_tabs.add_theme_stylebox_override("tab_hovered", _box(Color("#4a514a"), 9, 7))
	_tabs.add_theme_stylebox_override("tab_selected", _box(Color("#505849"), 9, 7))
	_tabs.add_theme_color_override("font_selected_color", Color("#f0d69e"))

func _box(fill: Color, horizontal: float = 0.0, vertical: float = 0.0) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = fill
	style.set_corner_radius_all(4)
	style.content_margin_left = horizontal
	style.content_margin_right = horizontal
	style.content_margin_top = vertical
	style.content_margin_bottom = vertical
	return style
