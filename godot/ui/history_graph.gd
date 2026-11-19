extends Control

signal month_selected(month: int)

const BACKGROUND := Color("#1b201d")
const GRID := Color("#414943")
const AXIS := Color("#b8c0b7")
const SELECTION := Color("#f0d69e")
const MONTHS := ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"]

var _series: Array[Dictionary] = []
var _first := 0
var _last := -1
var _selected := -1

func _ready() -> void:
	resized.connect(queue_redraw)

func update_series(series: Array[Dictionary], first: int, last: int, selected: int) -> void:
	_series = series
	_first = first
	_last = last
	_selected = selected
	queue_redraw()

static func month_text(month: int) -> String:
	@warning_ignore("integer_division")
	return "%s %d" % [MONTHS[month % 12], month / 12]

static func number_text(value: float) -> String:
	var magnitude := absf(value)
	var format := "%.10f" if magnitude < 0.01 else "%.2f"
	var parts := (format % magnitude).split(".")
	var digits := parts[0]
	for i in range(digits.length() - 3, 0, -3):
		digits = digits.insert(i, ".")
	var fraction := parts[1].rstrip("0")
	return ("−" if value < 0.0 else "") + digits + ("," + fraction if fraction != "" else "")

static func sample(series: Dictionary, index: int) -> float:
	var values: PackedFloat64Array = series["values"]
	return values[index] if index >= 0 and index < values.size() else NAN

func _plot() -> Rect2:
	return Rect2(Vector2(104, 16), Vector2(maxf(1.0, size.x - 120), maxf(1.0, size.y - 49)))

func _x(month: int, plot: Rect2) -> float:
	if _first == _last:
		return plot.get_center().x
	return lerpf(plot.position.x, plot.end.x, float(month - _first) / float(_last - _first))

func _gui_input(event: InputEvent) -> void:
	if _last < _first or not event is InputEventMouseButton:
		return
	if event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		var plot := _plot()
		if plot.has_point(event.position):
			var fraction: float = (event.position.x - plot.position.x) / plot.size.x
			month_selected.emit(roundi(lerpf(_first, _last, fraction)))
			accept_event()

func _draw() -> void:
	var font := get_theme_default_font()
	var plot := _plot()
	draw_rect(Rect2(Vector2.ZERO, size), BACKGROUND)
	var minimum := 0.0
	var maximum := 0.0
	var found := false
	for series in _series:
		for month in range(_first, _last + 1):
			var value := sample(series, _last - month)
			if is_finite(value):
				found = true
				minimum = minf(minimum, value)
				maximum = maxf(maximum, value)
	if not found:
		var message := "Keine Messwerte für diesen Zeitraum"
		var width := font.get_string_size(message, HORIZONTAL_ALIGNMENT_LEFT, -1, 16).x
		draw_string(font, Vector2((size.x - width) * 0.5, size.y * 0.5), message, HORIZONTAL_ALIGNMENT_LEFT, -1, 16, AXIS)
		return
	# Round outward to a readable scale and retain the zero line for debt.
	var step := _tick_step((maximum - minimum) / 4.0)
	minimum = floorf(minimum / step) * step
	maximum = maxf(minimum + step, ceilf(maximum / step) * step)
	var ticks := roundi((maximum - minimum) / step)
	for i in range(ticks + 1):
		var level := minimum + step * i
		var height := lerpf(plot.end.y, plot.position.y, float(i) / float(ticks))
		draw_line(Vector2(plot.position.x, height), Vector2(plot.end.x, height), AXIS if is_zero_approx(level) else GRID)
		var label := number_text(snappedf(level, step))
		var width := font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, 12).x
		draw_string(font, Vector2(plot.position.x - width - 8, height + 4), label, HORIZONTAL_ALIGNMENT_LEFT, -1, 12, AXIS)
	var intervals := mini(4, _last - _first)
	for i in range(intervals + 1):
		var month := _first if intervals == 0 else roundi(lerpf(_first, _last, float(i) / float(intervals)))
		@warning_ignore("integer_division")
		var label := "%02d/%d" % [month % 12 + 1, month / 12]
		var center := _x(month, plot)
		var width := font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, 12).x
		var left := clampf(center - width / 2.0, 4.0, size.x - width - 4.0)
		draw_string(font, Vector2(left, plot.end.y + 20), label, HORIZONTAL_ALIGNMENT_LEFT, -1, 12, AXIS)
	var selected_x := _x(_selected, plot)
	draw_dashed_line(Vector2(selected_x, plot.position.y), Vector2(selected_x, plot.end.y), SELECTION, 1.0, 4.0)
	for series in _series:
		var previous := Vector2.INF
		for month in range(_first, _last + 1):
			var value := sample(series, _last - month)
			if not is_finite(value):
				previous = Vector2.INF
				continue
			var point := Vector2(_x(month, plot), lerpf(plot.end.y, plot.position.y, inverse_lerp(minimum, maximum, value)))
			if previous.is_finite():
				draw_line(previous, point, series.color, 2.0, true)
			draw_circle(point, 4.0 if month == _selected else 2.0, series.color, true, -1.0, true)
			previous = point

func _tick_step(rough: float) -> float:
	if rough == 0.0:
		return 1.0
	var base := pow(10.0, floorf(log(rough) / log(10.0)))
	for multiple in [1.0, 2.0, 5.0]:
		if rough <= base * multiple:
			return base * multiple
	return base * 10.0
