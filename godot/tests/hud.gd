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
	var speed_log: Array[int] = []
	hud.speed_changed.connect(func(value: int): speed_log.append(value))
	for speed: int in [0, 1, 2, 4]:
		hud.speed_buttons[speed].pressed.emit()
	check(speed_log == [0, 1, 2, 4], "Fehlerhafte Zuordnung der Zeitsteuerung.")
	check(game.selection(Vector2i(1, 1), Vector2i(98, 98), -2, 1, true).is_empty(),
		"Ein sehr großer Abriss wird ohne Begrenzung vorbereitet.")
	for shortcut: Key in [KEY_1, KEY_7, KEY_QUOTELEFT]:
		root.push_input(key(shortcut))
		check(hud.layers.selected == {KEY_1: 3, KEY_7: 9, KEY_QUOTELEFT: 0}[shortcut],
			"Die klassische Kartenebene fehlt für Taste %d." % shortcut)
	root.push_input(key(KEY_B))
	check(hud.chosen_tool == -2, "B schaltet das Abrisswerkzeug nicht ein.")
	root.push_input(key(KEY_B))
	check(hud.chosen_tool == -1, "B schaltet das Abrisswerkzeug nicht aus.")
	check(hud.get("category_buttons") != null, "Direkte Baukategorien fehlen.")
	if hud.get("category_buttons") == null:
		quit(1)
		return
	var all := {}
	for category: String in hud.categories:
		hud.category_buttons[category].pressed.emit()
		check(hud.catalog_panel.visible and not hud.has_modal(), "Baukategorie blockiert das Spiel.")
		check(hud.tool_list.get_child_count() == hud.categories[category].size(),
			"Direkte Baukategorie zeigt falsche Einträge.")
		for button: Button in hud.tool_list.get_children():
			var tool_id: int = button.get_meta("tool_id")
			check(not all.has(tool_id), "Doppelter Katalogeintrag %d." % tool_id)
			all[tool_id] = true
			check(button.text.contains("Felder") and button.text.contains("Technologie"),
				"Angaben zum Werkzeug fehlen.")
	for tool: Dictionary in core.tools():
		check(all.has(int(tool.id)), "Werkzeug fehlt im Katalog %s." % tool.name)
	var option: Button = hud.tool_list.get_child(0)
	var id: int = option.get_meta("tool_id")
	option.pressed.emit()
	check(hud.chosen_tool == id and not hud.catalog_panel.visible, "Katalogauswahl wurde nicht übernommen.")
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
	for pixels in [Vector2i(640, 480), Vector2i(1440, 900), Vector2i(1920, 1080)]:
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
