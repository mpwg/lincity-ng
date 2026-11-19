extends Camera3D

signal moved

var focus := Vector3.ZERO
var azimuth := deg_to_rad(28.0)
var elevation := deg_to_rad(44.0)
var distance := 16.0
var side := 100
var orbiting := false
var panning := false

func center_on(cell: Vector2i) -> void:
	focus = Vector3(cell.x + 0.5, 0, cell.y + 0.5)
	update_pose()

func update_pose() -> void:
	focus.x = clampf(focus.x, 0.5, side - 0.5)
	focus.z = clampf(focus.z, 0.5, side - 0.5)
	elevation = clampf(elevation, deg_to_rad(20), deg_to_rad(82))
	distance = clampf(distance, 4.0, side * 1.8)
	var horizontal := distance * cos(elevation)
	position = focus + Vector3(sin(azimuth) * horizontal, distance * sin(elevation), cos(azimuth) * horizontal)
	look_at(focus, Vector3.UP)
	moved.emit()

func ground_at(screen: Vector2) -> Vector3:
	var origin := project_ray_origin(screen)
	var ray := project_ray_normal(screen)
	if ray.y > -0.0001:
		return Vector3.INF
	return origin - ray * (origin.y / ray.y)

func tile_at(screen: Vector2) -> Vector2i:
	var point := ground_at(screen)
	if not point.is_finite() or point.x < 0 or point.z < 0 or point.x >= side or point.z >= side:
		return Vector2i(-1, -1)
	return Vector2i(floori(point.x), floori(point.z))

func input_camera(event: InputEvent) -> bool:
	if event is InputEventMouseButton:
		match event.button_index:
			MOUSE_BUTTON_RIGHT:
				orbiting = event.pressed
			MOUSE_BUTTON_MIDDLE:
				panning = event.pressed
			MOUSE_BUTTON_WHEEL_UP, MOUSE_BUTTON_WHEEL_DOWN:
				if event.pressed:
					distance *= 0.85 if event.button_index == MOUSE_BUTTON_WHEEL_UP else 1.0 / 0.85
					update_pose()
			_:
				return false
		return true
	if event is InputEventMouseMotion:
		if orbiting:
			azimuth -= event.relative.x * 0.007
			elevation += event.relative.y * 0.006
		elif panning:
			var before := ground_at(event.position - event.relative)
			var after := ground_at(event.position)
			if before.is_finite() and after.is_finite():
				focus += before - after
		else:
			return false
		update_pose()
		return true
	if event is InputEventKey and event.pressed:
		var offset := Vector2.ZERO
		match event.keycode:
			KEY_Q: azimuth -= PI / 12
			KEY_E: azimuth += PI / 12
			KEY_W, KEY_UP: offset.y -= 1
			KEY_S, KEY_DOWN: offset.y += 1
			KEY_A, KEY_LEFT: offset.x -= 1
			KEY_D, KEY_RIGHT: offset.x += 1
			_: return false
		offset = offset.rotated(-azimuth) * distance * 0.085
		focus += Vector3(offset.x, 0, offset.y)
		update_pose()
		return true
	return false

# Both views look into the same playable district, in map coordinates.
func riverside_view(bridge: bool) -> void:
	if bridge:
		focus = Vector3(80.65, 0, 30.25)
		azimuth = deg_to_rad(75)
		elevation = deg_to_rad(30)
		distance = 10.8
	else:
		focus = Vector3(76.85, 0, 29.95)
		azimuth = deg_to_rad(-8)
		elevation = deg_to_rad(31)
		distance = 8.6
	update_pose()
