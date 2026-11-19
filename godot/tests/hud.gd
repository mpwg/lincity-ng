extends SceneTree

var errors := 0

func _initialize() -> void:
	call_deferred("_test")

func check(ok: bool, message: String) -> void:
	if not ok:
		printerr(message)
		errors += 1

func key(code: Key) -> InputEventKey:
	var event := InputEventKey.new()
	event.keycode = code
	event.pressed = true
	return event

func _test() -> void:
	var game = preload("res://city.tscn").instantiate()
	root.add_child(game)
	game.set_process(false)
	var hud = game.hud
	if game.state.is_empty():
		printerr("Referenzstadt konnte nicht geladen werden.")
		quit(1)
		return
	var core: Object = game.core
	await process_frame
	game._process(0.01)
	check(is_equal_approx(game.elapsed, 0.01), "Die Simulation steht trotz geschlossenem Dialog.")
	var all := {}
	for index in hud.category_picker.item_count:
		hud.category_picker.select(index)
		hud.category_picker.item_selected.emit(index)
		for button: Button in hud.tool_list.get_children():
			var id: int = button.get_meta("tool_id")
			check(not all.has(id), "Doppelter Katalogeintrag %d." % id)
			all[id] = true
	for tool: Dictionary in core.tools():
		check(all.has(int(tool.id)), "Werkzeug fehlt im Katalog %s." % tool.name)
	var speed_log: Array[int] = []
	hud.speed_changed.connect(func(value: int): speed_log.append(value))
	for speed: int in [0, 1, 2, 4]:
		hud.speed_buttons[speed].pressed.emit()
	check(speed_log == [0, 1, 2, 4], "Fehlerhafte Zuordnung der Zeitsteuerung.")
	for i in hud.QUICK_TOOLS.size():
		root.push_input(key(KEY_1 + i))
		check(hud.chosen_tool == hud.QUICK_TOOLS[i][0], "Falsches Werkzeug für Taste %d." % (i + 1))
	root.push_input(key(KEY_B))
	await process_frame
	check(root.get_last_exclusive_window() == hud.catalog_dialog, "Baukatalog ist nicht modal.")
	var before: float = game.elapsed
	game._process(0.01)
	check(is_equal_approx(game.elapsed, before), "Die Simulation läuft bei geöffnetem Dialog.")
	var bearing: float = game.camera.azimuth
	game._unhandled_input(key(KEY_Q))
	check(game.camera.azimuth == bearing, "Die Kamera dreht sich hinter dem Dialog.")
	var click_world := InputEventMouseButton.new()
	click_world.button_index = MOUSE_BUTTON_LEFT
	click_world.pressed = true
	game._unhandled_input(click_world)
	check(game.start == game.NO_CELL, "Klick im Dialog beginnt einen Bauvorgang.")
	hud._shortcut_input(key(KEY_1))
	check(hud.chosen_tool == -2, "Werkzeugwechsel hinter offenem Dialog.")
	var option: Button = hud.tool_list.get_child(0)
	var id: int = option.get_meta("tool_id")
	option.pressed.emit()
	check(hud.chosen_tool == id and not hud.catalog_dialog.visible, "Katalogauswahl wurde nicht übernommen.")
	root.push_input(key(KEY_F1))
	check(hud.help_dialog.visible, "F1 öffnet die Hilfe nicht.")
	hud.help_dialog.hide()
	hud.show_info("Markt")
	hud.set_inspection(6, [{"id": 0, "buy": true, "sell": false}])
	check(hud.inspector.visible and hud.trade_button.visible, "Handelsregeln fehlen im Feldbericht.")
	root.push_input(key(KEY_ESCAPE))
	check(not hud.inspector.visible and not hud.trade_button.visible, "Der Feldbericht bleibt geöffnet.")
	var target: Array[Vector2i] = []
	hud.focus_requested.connect(func(cell: Vector2i): target.append(cell))
	var click := InputEventMouseButton.new()
	click.button_index = MOUSE_BUTTON_LEFT
	click.pressed = true
	click.position = hud.map.map_rect().get_center()
	hud.map._gui_input(click)
	var middle := floori(hud.map.side / 2.0)
	check(target == [Vector2i(middle, middle)], "Minikartenmitte und Raster stimmen nicht überein.")
	hud.set_focus(Vector2i(middle, middle), 0.0)
	hud.map._gui_input(key(KEY_LEFT))
	check(target.back() == Vector2i(middle - 1, middle), "Pfeiltaste navigiert nicht auf der Minikarte.")
	for pixels in [Vector2i(1440, 900), Vector2i(1920, 1080)]:
		root.size = pixels
		await process_frame
		await process_frame
		var bounds: Rect2 = hud.palette.get_global_rect()
		check(hud.surface.get_rect().encloses(bounds), "Bauleiste außerhalb der Spielfläche bei %s." % pixels)
		check(not bounds.intersects(hud.map.get_global_rect()), "Bauleiste überlappt die Minikarte bei %s." % pixels)
	if errors == 0:
		print("OK: Bauleiste, vollständiger Katalog, Zeitsteuerung, exklusive Dialoge, Tastatur, Minikarte und HUD-Größen.")
	game.free()
	quit(1 if errors else 0)
