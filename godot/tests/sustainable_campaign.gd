extends SceneTree
## Play the recorded build commands in the frontend and wait for a real victory.
## Pass the freshly generated start.gz and the action TSV as user arguments.

const DAYS_PER_YEAR := 1200
const MAX_RESEARCH_WAIT := 5 * DAYS_PER_YEAR
const MAX_CAMPAIGN_DAY := 321 * DAYS_PER_YEAR

func _initialize() -> void:
	call_deferred("_run")

func _fail(reason: String) -> void:
	printerr(reason)
	quit(1)

func _choose(game: Node, id: int) -> bool:
	if game.tool == id:
		return true
	for category: String in game.hud.categories:
		if id not in game.hud.categories[category]:
			continue
		game.hud.category_buttons[category].pressed.emit()
		for button: Button in game.hud.tool_list.get_children():
			if button.get_meta("tool_id") == id and not button.disabled:
				button.pressed.emit()
				return game.tool == id
	return false

func _point_and_build(game: Node, cell: Vector2i) -> void:
	game.camera.center_on(cell)
	var position: Vector2 = game.camera.unproject_position(Vector3(cell.x + 0.5, 0, cell.y + 0.5))
	var move := InputEventMouseMotion.new()
	move.position = position
	root.push_input(move, true)
	for down: bool in [true, false]:
		var click := InputEventMouseButton.new()
		click.button_index = MOUSE_BUTTON_LEFT
		click.position = position
		click.pressed = down
		root.push_input(click, true)

func _advance(game: Node, days: int) -> String:
	# Advance the same native day as the speed controls, without redrawing each day.
	for _day in days:
		var error: String = game.core.step()
		if not error.is_empty():
			return error
	return ""

func _run() -> void:
	var args := OS.get_cmdline_user_args()
	var paths: Array[String] = []
	for argument: String in args:
		if argument != "--mute":
			paths.append(argument)
	if paths.size() != 2 or not FileAccess.file_exists(paths[0]) or not FileAccess.file_exists(paths[1]):
		_fail("Erwartet werden der generierte Startspielstand und die Kampagnen-TSV.")
		return
	root.size = Vector2i(1440, 900)
	var game := (load("res://main.tscn") as PackedScene).instantiate()
	root.add_child(game)
	game.set_process(false)
	await process_frame
	if game.state.size() != 0 or not game.hud.session_menu.visible:
		_fail("Das Spiel startet ohne Hauptmenü.")
		return
	game.hud.session_menu.buttons.load_game.pressed.emit()
	game.hud.load_dialog.current_path = paths[0]
	game.hud.load_dialog.get_ok_button().pressed.emit()
	await process_frame
	await process_frame
	if game.state.get("time") != 0 or game.hud.session_menu.visible:
		_fail("Am Spielstart war keine neu erzeugte Stadt verfügbar.")
		return
	game.hud.speed_buttons[4].pressed.emit()

	var day := 0
	var commands := FileAccess.get_file_as_string(paths[1]).split("\n", false)
	if commands.is_empty():
		_fail("Die Kampagne enthält keine Bauaktionen.")
		return
	for raw_line in commands:
		var fields := raw_line.split("\t")
		if fields.size() != 5 or fields[1] != "build":
			_fail("Ungültige Kampagnenaktion: " + raw_line)
			return
		var when := maxi(day, int(fields[0]))
		var error := _advance(game, when - day)
		if not error.is_empty():
			_fail(error)
			return
		if day != when:
			day = when
			game._refresh()
			print("Ausbau Tag ", day, " Kasse ", game.state.money, " Einwohner ", game.state.population, " Technologie ", game.state.tech)
		var cell := Vector2i(int(fields[2]), int(fields[3]))
		var id := int(fields[4])
		if not game.tools_by_id.has(id):
			_fail("Unbekanntes Kampagnenwerkzeug %d." % id)
			return
		# Loading a save starts a new random stream. A technology threshold can be
		# reached later, so wait for it through ordinary simulation days.
		var tech: int = game.tools_by_id[id].get("required_tech", 0)
		var wait_until := day + MAX_RESEARCH_WAIT
		while game.state.tech < tech and day < wait_until:
			error = _advance(game, 100)
			if not error.is_empty():
				_fail(error)
				return
			day += 100
			game._refresh()
		if not _choose(game, id):
			_fail("Werkzeug %s fehlt an Tag %d bei Technologie %d" % [id, day, game.state.tech])
			return
		_point_and_build(game, cell)
		if int(game.state.tiles[cell.y * int(game.state.side) + cell.x]) != id:
			_fail("Bau %d an Tag %d bei %s fehlgeschlagen: %s" % [id, day, cell, game.hud.details.text])
			return
		await process_frame
	print("Alle Bauaktionen ausgeführt, Stadt simuliert bis zum Sieg.")
	while day < MAX_CAMPAIGN_DAY:
		var error := _advance(game, DAYS_PER_YEAR)
		if not error.is_empty():
			_fail(error)
			return
		day += DAYS_PER_YEAR
		game._refresh()
		if game.hud.outcome_dialog.visible:
			if not game.state.sustainable or game.hud.outcome_dialog.title != game.hud.tr("Sieg durch Nachhaltigkeit"):
				_fail("Unerwarteter Spielausgang: " + game.hud.outcome_dialog.title)
				return
			print("SIEG über Godot bis Tag ", day, " nach ", commands.size(), " Bauaktionen: ",
				game.state.population, " Einwohner, Technologie ", game.state.tech)
			game.hud.outcome_dialog.get_ok_button().pressed.emit()
			await process_frame
			game.hud.speed_buttons[1].pressed.emit()
			game._process(0.12)
			if game.hud.outcome_dialog.visible or game.state.time <= day:
				_fail("Nach dem Siegdialog läuft die Stadt nicht weiter.")
				return
			quit()
			return
		if day % (10 * DAYS_PER_YEAR) == 0:
			print("Stadt Tag ", day, " Kasse ", game.state.money, " Einwohner ", game.state.population, " Technologie ", game.state.tech)
		await process_frame
	_fail("Die Kampagne löste keinen Nachhaltigkeitssieg aus.")
