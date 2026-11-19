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
	root.size = Vector2i(1920, 1080)
	root.content_scale_size = root.size
	var game = load("res://city.tscn").instantiate()
	root.add_child(game)
	current_scene = game
	game.speed = 0
	if game.state.is_empty():
		quit(1)
		return
	for bridge in [false, true]:
		game.camera.riverside_view(bridge)
		await process_frame
		await process_frame
		RenderingServer.force_draw(false)
		var image := root.get_texture().get_image()
		var filename := "flussbruecke.png" if bridge else "marktstrasse.png"
		if image == null or image.save_png(directory.path_join(filename)) != OK:
			quit(1)
			return
	quit()
