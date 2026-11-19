extends SceneTree

func _initialize() -> void:
	call_deferred("_check")

func _check() -> void:
	var license_dir := OS.get_cmdline_user_args()[0]
	var game = load("res://city.tscn").instantiate()
	root.add_child(game)
	current_scene = game
	game.speed = 0
	if game.state.is_empty() or game.state.get("side", 0) < 1:
		printerr("Die Startstadt fehlt im Spielpaket.")
		quit(1)
		return
	var ok := true
	var credits = game.hud.credits_dialog
	game.hud._menu_action(8)
	ok = ok and game.hud.has_modal()
	for i in credits.sections.item_count:
		credits._show_section(i)
		ok = ok and credits.body.text.length() > 100
	for item in [["Godot.txt", credits.engine_notices()], ["Mitwirkende.txt", credits.contributors()]]:
		var text_file := FileAccess.open(license_dir.path_join(item[0]), FileAccess.WRITE)
		if text_file == null or not text_file.store_string(item[1]): ok = false
	credits.hide()
	ok = ok and _play(game)
	var before: Dictionary = game.core.snapshot()
	var tmp := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-release", "gz")
	ok = ok and tmp != null
	if tmp:
		var path := tmp.get_path_absolute()
		ok = ok and game.core.save_city(path).is_empty()
		ok = ok and game.core.load_city(path).is_empty()
		var after: Dictionary = game.core.snapshot()
		ok = ok and after.tiles == before.tiles and after.money == before.money
		ok = ok and game.core.step().is_empty()
	if not ok: printerr("Spielstart, Lizenzen, Bedienung oder Spielstandprüfung fehlgeschlagen.")
	else: print("Spielpaket geprüft.")
	quit(0 if ok else 1)

func _click(position: Vector2, down: bool) -> void:
	var click := InputEventMouseButton.new()
	click.button_index = MOUSE_BUTTON_LEFT
	click.position = position
	click.pressed = down
	root.push_input(click)

func _play(game) -> bool:
	var side: int = game.state.side
	var market := Vector2i(76, 28)
	game.camera.center_on(market)
	for angle in [0, 90, 180, 270]:
		game.camera.azimuth = deg_to_rad(angle)
		game.camera.update_pose()
		var screen: Vector2 = game.camera.unproject_position(Vector3(market.x + 1, 0.38, market.y + 1))
		_click(screen, true)
		_click(screen, false)
		if game.selected != market or not game.hud.inspector.visible:
			printerr("Marktauswahl fehlgeschlagen bei %d°." % angle)
			return false
		game.hud._close_inspection()
	var first := game.NO_CELL as Vector2i
	for y in range(1, side - 1):
		for x in range(1, side - 2):
			var cell := Vector2i(x, y)
			var tile_index := y * side + x
			if game.state.tiles[tile_index] in [11, 21] or game.state.tiles[tile_index + 1] in [11, 21]: continue
			if game.core.preview(11, cell).allowed and game.core.preview(11, cell + Vector2i.RIGHT).allowed:
				first = cell
				break
		if first != game.NO_CELL: break
	if first == game.NO_CELL: return false
	var second := first + Vector2i.RIGHT
	game.camera.center_on(first)
	var a: Vector2 = game.camera.unproject_position(Vector3(first.x + 0.5, 0, first.y + 0.5))
	var b: Vector2 = game.camera.unproject_position(Vector3(second.x + 0.5, 0, second.y + 0.5))
	var money: int = game.state.money
	var cost: int = game.core.preview(11, first).cost + game.core.preview(11, second).cost
	game.hud.quick_buttons[11].pressed.emit()
	_click(a, true)
	var move := InputEventMouseMotion.new()
	move.position = b
	move.button_mask = MOUSE_BUTTON_MASK_LEFT
	root.push_input(move)
	if not game.hud.preview.text.contains(str(cost)):
		printerr("Die Vorschau der gezogenen Straße zeigt falsche Kosten.")
		return false
	_click(b, false)
	var i := first.y * side + first.x
	if game.state.tiles[i] != 11 or game.state.tiles[i + 1] != 11 or game.state.money != money - cost:
		printerr("Straßenfelder oder Baukosten stimmen nicht mit der Vorschau überein.")
		return false
	game.hud.quick_buttons[-2].pressed.emit()
	var demolition_cost: int = game.core.preview(-2, first).cost
	_click(a, true)
	_click(a, false)
	if game.state.tiles[i] == 11 or game.state.tiles[i + 1] != 11 or game.state.money != money - cost - demolition_cost:
		printerr("Straßenabriss oder Abbuchung fehlgeschlagen.")
		return false
	return true
