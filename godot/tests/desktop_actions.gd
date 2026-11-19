extends SceneTree
## Exercise the classic shortcuts and costly field removal in a fresh user directory.

var failures := 0

func _initialize() -> void:
	call_deferred("_run")

func _check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		printerr(message)

func _key(code: Key) -> void:
	var event := InputEventKey.new()
	event.keycode = code
	event.pressed = true
	root.push_input(event)

func _run() -> void:
	var game = load("res://city.tscn").instantiate()
	root.add_child(game)
	game.set_process(false)
	# Use an isolated XDG_DATA_HOME for this test, to keep the player's quick-save intact.
	if FileAccess.file_exists("user://Schnellspeicherung.gz"):
		printerr("Für den Test ist ein frisches Benutzerdatenverzeichnis erforderlich.")
		quit(2)
		return
	var brief: Dictionary = game.state
	_key(KEY_F12)
	_check(FileAccess.file_exists("user://Schnellspeicherung.gz"), "F12 speichert nicht.")
	game._new_city(0, 50, false, false)
	_key(KEY_F9)
	_check(game.state.side == brief.side and game.state.money == brief.money, "F9 lädt nicht.")
	var target := Vector2i(-1, -1)
	for i in game.state.tiles.size():
		if game.state.tiles[i] == 21:
			var p := Vector2i(i % game.state.side, floori(float(i) / game.state.side))
			if game.core.preview(-2, p).allowed:
				target = p
				break
	_check(target.x >= 0, "Kein abreißbares Wasserfeld für die Prüfung.")
	if target.x >= 0 and game.has_method("_commit_build"):
		var cells: Array[Vector2i] = [target]
		var money: int = game.state.money
		var cost: int = game.core.preview(-2, target).cost
		game._commit_build(-2, cells)
		_check(game.hud.demolition_dialog.visible and game.state.money == money, "Teurer Abriss startet vor Bestätigung.")
		game.hud.demolition_dialog.canceled.emit()
		game.hud.demolition_dialog.hide()
		_check(game.state.money == money, "Abbruch belastet die Stadtkasse.")
		game._commit_build(-2, cells)
		game.hud.demolition_dialog.confirmed.emit()
		game.hud.demolition_dialog.hide()
		_check(game.state.money == money - cost, "Bestätigter Abriss wird nicht wie angekündigt abgerechnet: %s − %s = %s." % [money, cost, game.state.money])
	else:
		_check(false, "Abrissbestätigung fehlt.")
	if failures == 0:
		print("OK: Schnellspeichern, Schnellladen und bestätigter Abriss.")
	quit(1 if failures else 0)
