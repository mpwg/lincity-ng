extends Node3D

const NO_CELL := Vector2i(-1, -1)
const OPENING := "res://scenarios/am_fluss.scn.gz"
const TRANSPORT := [7, 9, 11]
const BUILD_COLOR := Color(0.25, 1.0, 0.67, 0.9)
const INVALID_COLOR := Color(1, 0.28, 0.18, 0.95)

@onready var camera = $Camera
@onready var view = $World
@onready var hud = $HUD
var core: Object
var state: Dictionary
var tools_by_id := {}
var tool := -1
var speed := 1
var elapsed := 0.0
var traffic_time := 0.0
var traffic_sample := 0.0
var layer := 0
var commodity := 0
var start := NO_CELL
var hover := NO_CELL
var selected := NO_CELL
var production := false
var alternate := false
var restoring_camera := false

func _ready() -> void:
	if not ClassDB.class_exists("CityWorld"):
		hud.show_info("Die native Erweiterung fehlt. Bitte das CMake-Ziel lincity_godot bauen.")
		set_process(false)
		return
	core = ClassDB.instantiate("CityWorld")
	hud.tool_changed.connect(_select_tool)
	hud.inspection_closed.connect(func(): selected = NO_CELL)
	hud.modal_opened.connect(_cancel_pointer)
	hud.speed_changed.connect(func(value: int): speed = value; elapsed = 0)
	hud.layer_selected.connect(_set_layer)
	hud.focus_requested.connect(camera.center_on)
	hud.save_requested.connect(_save_city)
	hud.load_requested.connect(_load_city)
	hud.new_requested.connect(_new_city)
	hud.new_settings.connect(_new_city)
	hud.survey_requested.connect(func(): _after_action(core.survey_coal()))
	hud.launch_requested.connect(func(): _after_action(core.launch_rocket(selected)))
	hud.trade_changed.connect(func(id: int, buy: bool, sell: bool): _after_action(core.set_trade(selected, id, buy, sell)))
	hud.report_requested.connect(func(value: bool): production = value; _inspect())
	camera.moved.connect(_camera_moved)
	get_window().focus_exited.connect(_cancel_pointer)
	var opening := OPENING
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--city="):
			opening = arg.trim_prefix("--city=")
	_load_city(opening)
	if opening == OPENING and not state.is_empty():
		camera.riverside_view("--bridge-view" in OS.get_cmdline_user_args())

func _new_city(climate := 0, side := 100, village := true, trees := true) -> void:
	var error: String = core.new_city(climate, side, village, trees)
	_after_open(error)

func _load_city(path: String) -> void:
	var physical := ProjectSettings.globalize_path(path)
	# Native readers take filesystem paths. Built-in scenarios may also live in a PCK.
	if path.begins_with("res://"):
		var bytes := FileAccess.get_file_as_bytes(path)
		if bytes.is_empty():
			hud.show_info("Das Szenario konnte nicht gelesen werden.")
			return
		var temp := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity", "gz")
		if not temp or not temp.store_buffer(bytes):
			hud.show_info("Das Szenario konnte nicht geöffnet werden.")
			return
		temp.flush()
		_after_open(core.load_city(temp.get_path_absolute()))
		return
	_after_open(core.load_city(physical))

func _save_city(path: String) -> void:
	core.set_focus(Vector2i(floori(camera.focus.x), floori(camera.focus.z)))
	var error: String = core.save_city(ProjectSettings.globalize_path(path))
	hud.show_info("Stadt gespeichert." if error.is_empty() else error)

func _after_open(error: String) -> void:
	if not error.is_empty():
		hud.show_info(error)
		return
	selected = NO_CELL
	traffic_time = 0.0
	traffic_sample = 0.0
	$Traffic.update_commuters(PackedVector3Array())
	_select_tool(-1)
	elapsed = 0
	var available: Array = core.tools()
	tools_by_id.clear()
	for item in available:
		tools_by_id[int(item.id)] = item
	hud.setup(available)
	restoring_camera = true
	_refresh()
	restoring_camera = false
	camera.side = state.side
	camera.center_on(state.focus)
	hud.show_info("")

func _refresh() -> void:
	state = core.snapshot()
	if layer != 0:
		state.overlay_colors = core.overlay(layer, commodity)
	state.overlay_id = layer
	state.finances = core.finances()
	view.update_city(state, layer != 0)
	$Traffic.update_map(state)
	state.focus = Vector2i(floori(camera.focus.x), floori(camera.focus.z)) if not restoring_camera else state.focus
	hud.update_city(state)
	_inspect()
	_update_preview()
	for message in core.take_messages():
		hud.show_event(str(message))

func _set_layer(id: int, product: int) -> void:
	layer = id
	commodity = product
	if not state.is_empty():
		_refresh()

func _process(delta: float) -> void:
	if state.is_empty() or speed <= 0 or hud.has_modal():
		return
	traffic_time += delta * speed
	traffic_sample += delta
	if traffic_sample >= 0.05:
		$Traffic.update_commuters(core.advance_traffic(int(traffic_time * 1000)))
		traffic_sample = fmod(traffic_sample, 0.05)
	elapsed += delta * speed
	if elapsed < 0.25:
		return
	elapsed = fmod(elapsed, 0.25)
	var error: String = core.step()
	if not error.is_empty():
		speed = 0
		hud.set_speed(0)
		hud.show_info(error)
	else:
		_refresh()

func _after_action(error: String) -> void:
	_refresh()
	if not error.is_empty():
		hud.show_info(error)

func _select_tool(id: int) -> void:
	tool = id
	start = NO_CELL
	hover = NO_CELL
	$Preview.mesh = null

func _inspect() -> void:
	if selected == NO_CELL or state.is_empty():
		return
	var i: int = selected.y * state.side + selected.x
	var group: int = state.tiles[i]
	hud.show_info(core.inspect_tile(selected, production))
	hud.set_inspection(group, core.trade_rules(selected) if group in [6, 17] else [], production)

func _camera_moved() -> void:
	if state.is_empty():
		return
	hud.set_focus(Vector2i(floori(camera.focus.x), floori(camera.focus.z)), camera.azimuth)
	_point_at(get_viewport().get_mouse_position())

func _cancel_pointer() -> void:
	start = NO_CELL
	camera.panning = false
	camera.orbiting = false
	hover = NO_CELL
	$Preview.mesh = null

func _input(event: InputEvent) -> void:
	# A release over the toolbar ends a gesture without placing anything beneath it.
	if event is InputEventMouseButton and not event.pressed and get_viewport().gui_get_hovered_control() != null:
		_cancel_pointer()

func _unhandled_input(event: InputEvent) -> void:
	if state.is_empty() or hud.has_modal():
		return
	if camera.input_camera(event):
		get_viewport().set_input_as_handled()
		return
	if event is InputEventKey and event.keycode == KEY_CTRL:
		alternate = event.pressed
		_update_preview()
	if event is InputEventMouseMotion:
		_point_at(event.position)
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		_point_at(event.position)
		if event.pressed:
			start = hover
		elif start != NO_CELL and hover != NO_CELL:
			if tool == -1:
				selected = hover
				production = false
				_inspect()
			else:
				var error := ""
				var cells := selection(start, hover, tool, int(tools_by_id.get(tool, {}).get("size", 1)), alternate)
				cells.reverse()
				for cell in cells:
					if cells.size() == 1 or core.preview(tool, cell).allowed:
						error = core.apply(tool, cell)
						if not error.is_empty():
							break
				_after_action(error)
			start = NO_CELL
		else:
			start = NO_CELL
		_update_preview()

func _point_at(screen: Vector2) -> void:
	var target: Vector2i = camera.tile_at(screen)
	if tool == -1:
		var hit: Vector2i = view.pick_building(camera.project_ray_origin(screen), camera.project_ray_normal(screen))
		if hit != NO_CELL:
			target = hit
	if target != hover:
		hover = target
		_update_preview()

static func selection(first: Vector2i, last: Vector2i, id: int, footprint: int, turn: bool) -> Array[Vector2i]:
	var cells: Array[Vector2i] = []
	if footprint != 1 or id in [-1, -3] or first == NO_CELL:
		cells.append(last)
		return cells
	var dir := (last - first).sign()
	if id in TRANSPORT:
		var current := first
		cells.append(current)
		var corner := Vector2i(first.x, last.y) if turn else Vector2i(last.x, first.y)
		while current != corner:
			current += (corner - current).sign()
			cells.append(current)
		while current != last:
			current += (last - current).sign()
			cells.append(current)
	elif id == -2 and not turn:
		cells.append(last)
	else:
		for y in range(absi(last.y - first.y) + 1):
			for x in range(absi(last.x - first.x) + 1):
				cells.append(first + Vector2i(x, y) * dir)
	return cells

func _update_preview() -> void:
	$Preview.mesh = null
	if hover == NO_CELL or state.is_empty() or tool == -1:
		hud.show_preview("")
		return
	var mesh := ImmediateMesh.new()
	mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	var amount := 0
	var cells := selection(start, hover, tool, int(tools_by_id.get(tool, {}).get("size", 1)), alternate)
	var reason := ""
	for cell in cells:
		var rule: Dictionary = core.preview(tool, cell)
		var extent := float(rule.size)
		var anchor := cell
		if tool in [-2, -3]:
			var index: int = state.anchors[cell.y * state.side + cell.x]
			if index >= 0:
				anchor = Vector2i(index % int(state.side), floori(float(index) / state.side))
				extent = float(tools_by_id.get(int(state.tiles[index]), {}).get("size", 1))
		mesh.surface_set_color(BUILD_COLOR if rule.allowed else INVALID_COLOR)
		if rule.allowed:
			amount += int(rule.cost)
		else:
			reason = str(rule.get("reason", "Hier nicht möglich."))
		var p := Vector3(anchor.x, 0.06, anchor.y)
		var corners := [p, p + Vector3(extent, 0, 0), p + Vector3(extent, 0, extent), p + Vector3(0, 0, extent)]
		for i in 4:
			mesh.surface_add_vertex(corners[i])
			mesh.surface_add_vertex(corners[(i + 1) % 4])
	mesh.surface_end()
	$Preview.mesh = mesh
	hud.show_preview(reason if cells.size() == 1 and not reason.is_empty() else "Kosten  %s $" % amount)
