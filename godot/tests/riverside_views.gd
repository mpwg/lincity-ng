extends SceneTree
## Capture the two design views of one paused city on the real renderer.
## Set LINCITY_CAPTURE_DIR to an existing directory for the two PNGs.

func _initialize() -> void:
	call_deferred("_capture")

func _capture() -> void:
	var directory := OS.get_environment("LINCITY_CAPTURE_DIR")
	if not DirAccess.dir_exists_absolute(directory) or DisplayServer.get_name() == "headless":
		printerr("Bildaufnahme braucht einen Grafiktreiber und LINCITY_CAPTURE_DIR.")
		quit(2)
		return
	var game = load("res://city.tscn").instantiate()
	root.add_child(game)
	current_scene = game
	root.size = Vector2i(1920, 1080)
	root.content_scale_size = root.size
	game.speed = 0
	if game.state.is_empty():
		quit(1)
		return
	# Advance the real opening to the first working shift and freeze both views
	# on the same tick. The smokestack is driven by actual production.
	var mill: int = 27 * int(game.state.side) + 80
	for day in 64:
		if game.state.operating[mill] != 0:
			break
		if not game.core.step().is_empty():
			quit(1)
			return
		game.state = game.core.snapshot()
	if game.state.operating[mill] == 0:
		printerr("Die Fabrik am Fluss hat keine Waren produziert.")
		quit(1)
		return
	game._refresh()
	game.hud.set_speed(0)
	for bridge in [false, true]:
		game.camera.riverside_view(bridge)
		# Use the live inspector at the same reproducible road location for both views.
		var lane := Vector3(84.5, .12, 30.5) if bridge else Vector3(76.5, .025, 32.5)
		game._point_at(game.camera.unproject_position(lane))
		await process_frame
		await process_frame
		RenderingServer.force_draw(false)
		var image := root.get_texture().get_image()
		var filename := "flussbruecke.png" if bridge else "marktstrasse.png"
		if image == null or image.save_png(directory.path_join(filename)) != OK:
			quit(1)
			return
	quit()
