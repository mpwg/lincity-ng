extends Node3D

const GameSettings = preload("res://ui/settings.gd")
const GameAudio = preload("res://audio/game_audio.gd")
const NO_CELL := Vector2i(-1, -1)
const OPENING := "res://scenarios/am_fluss.scn.gz"
const SESSION_SAVE := "user://Fortsetzung.scn.gz"
const TRANSPORT := [7, 9, 11]
const CONFIRM_DEMOLITION := [21, 25, 33]
const BUILD_COLOR := Color(0.25, 1.0, 0.67, 0.9)
const INVALID_COLOR := Color(1, 0.28, 0.18, 0.95)

@onready var camera = $Camera
@onready var view = $World
@onready var hud = $HUD
@export var start_in_menu := false
var core: Object
var settings := GameSettings.new()
var audio := GameAudio.new()
var state: Dictionary
var tools_by_id := {}
var tool := -1
var speed := 1
var elapsed := 0.0
var animation_time := 0.0
var animation_sample := 0.0
var layer := 0
var commodity := 0
var start := NO_CELL
var hover := NO_CELL
var selected := NO_CELL
var production := false
var alternate := false
var restoring_camera := false
var quitting := false
var pending_demolition: Array[Vector2i] = []

func _ready() -> void:
	add_child(audio)
	if not ClassDB.class_exists("CityWorld"):
		hud.show_info("Die native Erweiterung fehlt. Bitte das CMake-Ziel lincity_godot bauen.")
		set_process(false)
		return
	core = ClassDB.instantiate("CityWorld")
	settings.load_from_disk()
	hud.settings = settings
	hud.settings_dialog.setup(settings, audio)
	hud.settings_dialog.settings_applied.connect(_apply_settings.bind(false))
	hud.modal_opened.connect(audio.play_effect.bind(&"WindowOpen"))
	hud.modal_closed.connect(audio.play_effect.bind(&"WindowClose"))
	hud.quit_requested.connect(_quit)
	hud.pause_requested.connect(_open_pause)
	hud.session_menu.action_requested.connect(_session_action)
	hud.history_requested.connect(func():
		hud.history_dialog.set_history(core.history())
		hud.history_dialog.popup_centered(Vector2i(760, 490)))
	get_tree().auto_accept_quit = false
	_apply_settings(true)
	hud.tool_changed.connect(func(id: int):
		_select_tool(id)
		audio.play_effect(&"Click"))
	hud.inspection_closed.connect(_close_inspection)
	hud.modal_opened.connect(_cancel_pointer)
	hud.demolition_dialog.confirmed.connect(func():
		var cells := pending_demolition
		pending_demolition = []
		_apply_build(-2, cells))
	hud.demolition_dialog.canceled.connect(func(): pending_demolition = [])
	hud.speed_changed.connect(func(value: int): speed = value; elapsed = 0)
	hud.layer_selected.connect(_set_layer)
	hud.focus_requested.connect(camera.center_on)
	hud.save_requested.connect(_save_city)
	hud.load_requested.connect(_load_city)
	hud.new_requested.connect(_new_city)
	hud.new_settings.connect(_new_city)
	hud.survey_requested.connect(func(): _after_action(core.survey_coal()))
	hud.launch_requested.connect(func():
		var error: String = core.launch_rocket(selected)
		_after_action(error)
		if error.is_empty(): audio.play_effect(&"RocketTakeoff"))
	hud.trade_changed.connect(func(id: int, buy: bool, sell: bool): _after_action(core.set_trade(selected, id, buy, sell)))
	hud.report_requested.connect(func(value: bool): production = value; _inspect())
	camera.moved.connect(_camera_moved)
	get_window().focus_exited.connect(_cancel_pointer)
	var args := OS.get_cmdline_user_args()
	var opening := "" if start_in_menu and "--bridge-view" not in args else OPENING
	for arg in args:
		if arg.begins_with("--city="):
			opening = arg.trim_prefix("--city=")
	if start_in_menu:
		_open_main_menu()
	if not opening.is_empty():
		_load_city(opening)

func _open_main_menu() -> void:
	hide()
	hud.open_session_menu(false, FileAccess.file_exists(SESSION_SAVE))

func _open_pause() -> void:
	if state.is_empty():
		return
	_cancel_pointer()
	hud.open_session_menu(true, false)

func _session_action(action: StringName) -> void:
	match action:
		&"resume":
			hud.close_session_menu()
		&"main_menu":
			if _save_session():
				state = {}
				_open_main_menu()
		&"new_game": hud._menu_action(0)
		&"load_game": hud._menu_action(1)
		&"save_game": hud._menu_action(2)
		&"continue_game": _load_city(SESSION_SAVE)
		&"settings": hud._menu_action(9)
		&"help": hud._menu_action(3)
		&"credits": hud._menu_action(8)
		&"quit": hud._menu_action(14)

func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST and is_node_ready():
		_quit()

func _quit() -> void:
	if quitting:
		return
	if not state.is_empty() and not _save_session():
		return
	quitting = true
	if audio.is_inside_tree():
		audio.apply_settings({"music_enabled": false, "effects_enabled": false})
		await get_tree().create_timer(0.15).timeout
	get_tree().quit()

func _apply_settings(on_start: bool) -> void:
	settings.apply_display(get_window(), on_start)
	$Sun.shadow_enabled = settings.values.shadows
	view.reflections_enabled = settings.values.reflections
	audio.apply_settings(settings.values)

func _new_city(climate := 0, side := 100, village := true, trees := true) -> void:
	var error: String = core.new_city(climate, side, village, trees)
	_after_open(error)

func _load_city(path: String) -> void:
	var physical := ProjectSettings.globalize_path(path)
	# Native readers take filesystem paths. Built-in scenarios may also live in a PCK.
	if path.begins_with("res://"):
		var bytes := FileAccess.get_file_as_bytes(path)
		if bytes.is_empty():
			_show_session_error(tr("Das Szenario konnte nicht gelesen werden."))
			return
		var temp := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity", "gz")
		if not temp or not temp.store_buffer(bytes):
			_show_session_error(tr("Das Szenario konnte nicht geöffnet werden."))
			return
		temp.flush()
		var error: String = core.load_city(temp.get_path_absolute())
		_after_open(error)
		if error.is_empty() and path == OPENING:
			camera.riverside_view("--bridge-view" in OS.get_cmdline_user_args())
		return
	_after_open(core.load_city(physical))

func _save_city(path: String) -> void:
	var error := _write_city(path)
	_show_session_error(tr("Stadt gespeichert.") if error.is_empty() else error)

func _write_city(path: String) -> String:
	core.set_focus(Vector2i(floori(camera.focus.x), floori(camera.focus.z)))
	return core.save_city(ProjectSettings.globalize_path(path))

func _save_session() -> bool:
	var path := ProjectSettings.globalize_path(SESSION_SAVE)
	var temporary := "%s.%d.tmp" % [path, OS.get_process_id()]
	var error := _write_city(temporary)
	if error.is_empty() and DirAccess.rename_absolute(temporary, path) != OK:
		error = tr("Die Datei für den Fortsetzungsstand konnte nicht ersetzt werden.")
	if error.is_empty():
		return true
	if FileAccess.file_exists(temporary):
		DirAccess.remove_absolute(temporary)
	if not hud.session_menu.visible:
		_open_pause()
	hud.session_menu.show_message(tr("Der Fortsetzungsstand konnte nicht gespeichert werden. %s") % error)
	return false

func _show_session_error(message: String) -> void:
	if hud.session_menu.visible:
		hud.session_menu.show_message(message)
	else:
		hud.show_info(message)

func _after_open(error: String) -> void:
	if not error.is_empty():
		_show_session_error(error)
		return
	show()
	hud.close_session_menu()
	selected = NO_CELL
	pending_demolition = []
	hud.events.clear()
	hud.outcome_dialog.hide()
	animation_time = 0.0
	animation_sample = 0.0
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
	audio.set_technology_percent(float(state.tech) / 10000.0)
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
	for event: Dictionary in core.take_events():
		hud.show_event(event)
		if event.get("kind") == "rocket_crash":
			audio.play_effect(&"RocketExplosion")

func _set_layer(id: int, product: int) -> void:
	layer = id
	commodity = product
	if not state.is_empty():
		_refresh()

func _process(delta: float) -> void:
	if quitting or state.is_empty() or speed <= 0 or hud.has_modal() or hud.session_menu.visible:
		return
	view.advance_rockets(delta)
	animation_time += delta
	animation_sample += delta
	if animation_sample >= 0.1:
		$Traffic.update_commuters(core.advance_animation(int(animation_time * 1000)))
		animation_sample = fmod(animation_sample, 0.1)
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

func _close_inspection() -> void:
	selected = NO_CELL
	_update_preview()

func _inspect() -> void:
	if selected == NO_CELL or state.is_empty():
		return
	var i: int = selected.y * state.side + selected.x
	var group: int = state.tiles[i]
	hud.show_info(core.inspect_tile(selected, production))
	hud.set_inspection(group, core.trade_rules(selected) if group in [6, 17] else [], production)
	if group == 23:
		hud.launch_button.disabled = int(state.rocket_frames.get(state.anchors[i], 0)) != 4

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
	if quitting or state.is_empty() or hud.has_modal() or hud.session_menu.visible:
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
				audio.play_tile_effect(core.tile_resource(selected))
				production = false
				_inspect()
			else:
				var cells := _action_cells()
				cells.reverse()
				_commit_build(tool, cells)
			start = NO_CELL
		else:
			start = NO_CELL
		_update_preview()

func _commit_build(id: int, cells: Array[Vector2i]) -> void:
	if id == -2:
		var cost := 0
		var confirm := false
		for cell in cells:
			var rule: Dictionary = core.preview(id, cell)
			if rule.allowed:
				cost += int(rule.cost)
				confirm = confirm or int(state.tiles[cell.y * state.side + cell.x]) in CONFIRM_DEMOLITION
		if confirm:
			pending_demolition = cells
			hud.demolition_dialog.dialog_text = "Der Abriss kostet %s $. Möchtest du fortfahren?" % hud._number(cost)
			hud.demolition_dialog.popup_centered(Vector2i(410, 165))
			return
	_apply_build(id, cells)

func _apply_build(id: int, cells: Array[Vector2i]) -> void:
	var error := ""
	var last := NO_CELL
	for cell in cells:
		if cells.size() == 1 or core.preview(id, cell).allowed:
			error = core.apply(id, cell)
			if not error.is_empty():
				break
			last = cell
	_after_action(error)
	if last != NO_CELL:
		if id == -2:
			audio.play_effect(&"Raze")
		else:
			audio.play_effect(&"Build")

func _point_at(screen: Vector2) -> void:
	var target: Vector2i = camera.tile_at(screen)
	if tool == -1:
		var hit: Vector2i = view.pick_building(camera.project_ray_origin(screen), camera.project_ray_normal(screen))
		if hit != NO_CELL:
			target = hit
	if target != hover:
		hover = target
		_update_preview()

func _action_cells() -> Array[Vector2i]:
	var fields := selection(start, hover, tool, int(tools_by_id.get(tool, {}).get("size", 1)), alternate)
	if tool != -2:
		return fields
	var targets: Array[Vector2i] = []
	var visited := {}
	var side: int = state.side
	for field in fields:
		var index := field.y * side + field.x
		var anchor := int(state.anchors[index]) if state.anchors[index] >= 0 else index
		if not visited.has(anchor):
			targets.append(Vector2i(anchor % side, floori(float(anchor) / side)))
			visited[anchor] = true
	return targets

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

# Draw a thin ribbon around the actual footprint. Its width follows zoom so
# the selected field remains visible in both the street view and the overview.
func _outline(mesh: ImmediateMesh, cell: Vector2i, extent: float, color: Color) -> void:
	var inset := clampf(camera.distance * .0022, .016, .10)
	var outer := PackedVector3Array()
	var inner := PackedVector3Array()
	for unit: Vector2 in [Vector2.ZERO, Vector2.RIGHT, Vector2.ONE, Vector2.DOWN]:
		outer.append(Vector3(cell.x + unit.x * extent, .055, cell.y + unit.y * extent))
		inner.append(Vector3(cell.x + inset + unit.x * (extent - 2 * inset), .055,
			cell.y + inset + unit.y * (extent - 2 * inset)))
	mesh.surface_set_color(color)
	for i in 4:
		var j := (i + 1) % 4
		for point in [outer[i], outer[j], inner[j], outer[i], inner[j], inner[i]]:
			mesh.surface_add_vertex(point)

func _update_preview() -> void:
	$Preview.mesh = null
	if state.is_empty():
		return
	var mesh := ImmediateMesh.new()
	if tool == -1:
		hud.show_preview("")
		var field := selected if selected != NO_CELL else hover
		if field == NO_CELL:
			return
		var anchor: int = state.anchors[field.y * state.side + field.x]
		if anchor < 0:
			anchor = field.y * state.side + field.x
		var cell := Vector2i(anchor % int(state.side), floori(float(anchor) / state.side))
		var size := float(view.Models.SIZE[state.tiles[anchor]])
		mesh.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
		_outline(mesh, cell, size, BUILD_COLOR)
		mesh.surface_end()
		$Preview.mesh = mesh
		return
	if hover == NO_CELL:
		hud.show_preview("")
		return
	mesh.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	var amount := 0
	var cells := _action_cells()
	var reason := ""
	for cell in cells:
		var rule: Dictionary = core.preview(tool, cell)
		var extent := float(rule.size)
		var anchor := cell
		if tool in [-2, -3]:
			var index: int = state.anchors[cell.y * state.side + cell.x]
			if index >= 0:
				anchor = Vector2i(index % int(state.side), floori(float(index) / state.side))
				extent = float(view.Models.SIZE[state.tiles[index]])
		if rule.allowed:
			amount += int(rule.cost)
		else:
			reason = str(rule.get("reason", "Hier nicht möglich."))
		_outline(mesh, anchor, extent, BUILD_COLOR if rule.allowed else INVALID_COLOR)
	mesh.surface_end()
	$Preview.mesh = mesh
	hud.show_preview(reason if cells.size() == 1 and not reason.is_empty() else "Kosten  %s $" % amount)
