extends SceneTree

# Draw explicitly so an occluded desktop window still renders every sample.
# Reading the framebuffer waits for the GPU and adds a conservative transfer cost.
const WARMUP := 100
const SAMPLES := 400
const ROCKET_98_START_DAY := 793565
var game
var camera
var intervals: Array[float] = []
var last_usec := 0
var frame := 0
var start_day := 0
var phase := 0
var distances := [16.0, 110.0]
var max_p95 := 0.0
var draw_calls := 0
var primitives := 0

func _initialize() -> void:
	call_deferred("_start")

func _start() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("Für die GPU-Messung ist ein echter Grafiktreiber erforderlich.")
		quit(2)
		return
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	RenderingServer.render_loop_enabled = false
	game = load("res://city.tscn").instantiate()
	root.add_child(game)
	current_scene = game
	root.size = Vector2i(1920, 1080)
	root.content_scale_size = Vector2i(1920, 1080)
	game._load_city("res://opening/Rocket_98.scn.gz")
	if game.state.get("time", -1) != ROCKET_98_START_DAY:
		printerr("Messung braucht den unveränderten Referenzstand Rocket 98.")
		quit(2)
		return
	camera = game.camera
	camera.center_on(Vector2i(47, 48))
	start_day = game.state.time
	game.speed = 4
	camera.distance = distances[0]
	camera.update_pose()
	print("GPU %s | %s | %s | %s | %d×%d Felder" % [
		RenderingServer.get_video_adapter_name(), RenderingServer.get_current_rendering_method(),
		root.size, Engine.get_version_info().string, game.state.side, game.state.side])
	process_frame.connect(_measure)

func _measure() -> void:
	RenderingServer.force_draw(false)
	var frame_image := root.get_texture().get_image()
	if frame_image == null or frame_image.get_size() != Vector2i(1920, 1080):
		printerr("Unerwartete Bildgröße.")
		quit(2)
		return
	var now := Time.get_ticks_usec()
	if frame > WARMUP:
		intervals.append((now - last_usec) / 1000.0)
		draw_calls = maxi(draw_calls, root.get_render_info(Viewport.RENDER_INFO_TYPE_VISIBLE, Viewport.RENDER_INFO_DRAW_CALLS_IN_FRAME))
		primitives = maxi(primitives, root.get_render_info(Viewport.RENDER_INFO_TYPE_VISIBLE, Viewport.RENDER_INFO_PRIMITIVES_IN_FRAME))
	last_usec = now
	camera.azimuth += TAU / SAMPLES
	camera.update_pose()
	frame += 1
	if intervals.size() < SAMPLES:
		return
	if game.state.time <= start_day:
		printerr("Die Simulation ist während der Kameradrehung stehen geblieben.")
		quit(2)
		return
	start_day = game.state.time
	if draw_calls < 1 or primitives < 1000:
		printerr("Keine gezeichnete Stadt. Fenster sichtbar halten oder --display-driver x11 verwenden.")
		quit(2)
		return
	intervals.sort()
	var p95 := intervals[ceili(intervals.size() * 0.95) - 1]
	max_p95 = maxf(max_p95, p95)
	print("DISTANCE %.1f | n=%d | median=%.2f ms | p95=%.2f ms | p99=%.2f ms | max=%.2f ms | draw_calls_max=%d | primitives_max=%d" %
		[distances[phase], SAMPLES, intervals[SAMPLES / 2], p95, intervals[ceili(SAMPLES * 0.99) - 1], intervals[-1], draw_calls, primitives])
	phase += 1
	if phase == distances.size():
		quit(0 if max_p95 <= 33.33 else 1)
		return
	draw_calls = 0
	primitives = 0
	camera.distance = distances[phase]
	frame = 0
	intervals.clear()
