extends Control

signal cell_selected(cell: Vector2i)

const BASE = Color("#91a46c")
const FOREST = Color("#486e4a")
const WATER = Color("#427f99")
const ROADS = Color("#666b62")
const BUILDINGS = Color("#c4ac86")
const HOMES = Color("#9ab88a")

var side := 0
var focus := Vector2i(-1, -1)
var bearing := 0.0
var texture: ImageTexture
var last_tiles := PackedInt32Array()
var last_colors := PackedColorArray()

func update_map(snapshot: Dictionary) -> void:
	side = int(snapshot.get("side", 0))
	focus = snapshot.get("focus", Vector2i(-1, -1))
	var tiles: PackedInt32Array = snapshot.get("tiles", PackedInt32Array())
	if side < 1 or tiles.size() != side * side:
		texture = null
		queue_redraw()
		return
	var colors: PackedColorArray = snapshot.get("overlay_colors", PackedColorArray())
	var use_colors := colors.size() == tiles.size() and int(snapshot.get("overlay_id", 0)) != 0
	if texture and texture.get_width() == side and last_tiles == tiles and last_colors == colors:
		queue_redraw()
		return
	last_tiles = tiles
	last_colors = colors
	var data := PackedByteArray()
	data.resize(tiles.size() * 3)
	var palette := PackedColorArray()
	palette.resize(51)
	palette.fill(BUILDINGS)
	for group in [0, 5]: palette[group] = BASE
	for group in [43, 44, 45, 19, 50]: palette[group] = FOREST
	for group in [7, 9, 11, 46, 47, 48]: palette[group] = ROADS
	for group in [4, 36, 37, 38, 39, 40]: palette[group] = HOMES
	palette[21] = WATER
	palette[34] = Color("#ff5739")
	palette[42] = Color("#c4af79")
	for i in tiles.size():
		var group := tiles[i]
		var color := palette[group] if group >= 0 and group < palette.size() else BASE
		if use_colors: color = colors[i]
		data[i * 3] = color.r8
		data[i * 3 + 1] = color.g8
		data[i * 3 + 2] = color.b8
	var image := Image.create_from_data(side, side, false, Image.FORMAT_RGB8, data)
	if texture and texture.get_width() == side:
		texture.update(image)
	else:
		texture = ImageTexture.create_from_image(image)
	queue_redraw()

func map_rect() -> Rect2:
	var extent := Vector2.ONE * minf(size.x, size.y)
	return Rect2((size - extent) / 2.0, extent)

func _draw() -> void:
	if texture == null: return
	var rect := map_rect()
	draw_texture_rect(texture, rect, false)
	if focus.x >= 0 and focus.y >= 0 and focus.x < side and focus.y < side:
		var point := rect.position + (Vector2(focus) + Vector2(0.5, 0.5)) * rect.size / float(side)
		var direction := Vector2(-sin(bearing), -cos(bearing))
		var flank := Vector2(-direction.y, direction.x)
		var marker := PackedVector2Array([point + direction * 7.0, point - direction * 4.0 + flank * 4.0, point - direction * 4.0 - flank * 4.0])
		draw_colored_polygon(marker, Color("#ffedb3"))
		marker.append(marker[0])
		draw_polyline(marker, Color("#343b32"), 1.5, true)
	if has_focus(): draw_rect(rect, Color("#e3c78a"), false, 2.0)

func _gui_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and side > 0:
		var move := Vector2i.ZERO
		match event.keycode:
			KEY_UP: move = Vector2i.UP
			KEY_DOWN: move = Vector2i.DOWN
			KEY_LEFT: move = Vector2i.LEFT
			KEY_RIGHT: move = Vector2i.RIGHT
			_: return
		cell_selected.emit((focus + move).clampi(0, side - 1))
		accept_event()
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		var rect := map_rect()
		if side > 0 and rect.has_point(event.position):
			cell_selected.emit(Vector2i((event.position - rect.position) / rect.size * side))
			accept_event()
