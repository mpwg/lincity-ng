extends SceneTree

var failures := 0

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		failures += 1
		printerr(message)

func _launch() -> Node:
	var scene := load(str(ProjectSettings.get_setting("application/run/main_scene"))) as PackedScene
	var game := scene.instantiate()
	root.add_child(game)
	return game

func _click(control: Control) -> void:
	var position := control.get_global_rect().get_center()
	var move := InputEventMouseMotion.new()
	move.position = position
	root.push_input(move, true)
	for down: bool in [true, false]:
		var click := InputEventMouseButton.new()
		click.button_index = MOUSE_BUTTON_LEFT
		click.position = position
		click.pressed = down
		root.push_input(click, true)

func _key(code: Key) -> void:
	var event := InputEventKey.new()
	event.keycode = code
	event.pressed = true
	root.push_input(event)

func _run() -> void:
	var session := ProjectSettings.globalize_path("user://Fortsetzung.scn.gz")
	if FileAccess.file_exists(session) or DirAccess.dir_exists_absolute(session):
		printerr("Für den Test ist ein frisches Benutzerdatenverzeichnis erforderlich.")
		quit(2)
		return
	var game := _launch()
	await process_frame
	await process_frame
	if not game.hud.get("session_menu"):
		_check(false, "Das reguläre Spiel startet ohne Hauptmenü.")
		quit(1)
		return
	var menu: Control = game.hud.session_menu
	_check(menu.visible and game.state.is_empty(), "Beim ersten Start darf keine Stadt bereits laufen.")
	_check(menu.buttons.continue_game.disabled, "Fortsetzen ist ohne Spielstand verfügbar.")
	_click(menu.buttons.new_game)
	_check(game.hud.new_dialog.visible, "Neues Spiel öffnet die Szenarioauswahl nicht.")
	game.hud.scenarios.select(1)
	game.hud.new_dialog.confirmed.emit()
	game.hud.new_dialog.hide()
	await process_frame
	_check(not menu.visible and game.state.side == 100, "Das gewählte Szenario startet nicht.")
	game.speed = 0
	var day: int = game.state.time
	var money: int = game.state.money
	_key(KEY_CTRL)
	_key(KEY_ESCAPE)
	_check(menu.visible, "Das Pausenmenü lässt sich nicht öffnen.")
	_key(KEY_ESCAPE)
	_check(not menu.visible and game.state.time == day, "Fortfahren ersetzt den Stand.")
	_check(not game.alternate, "Ein Tastaturmodifikator bleibt nach der Pause aktiv.")
	_key(KEY_ESCAPE)
	await process_frame
	_check(DirAccess.make_dir_absolute(session) == OK, "Speicherfehler konnte nicht vorbereitet werden.")
	_click(menu.buttons.main_menu)
	_check(menu.in_game and game.state.time == day, "Bei einem Speicherfehler geht die laufende Stadt verloren.")
	_check(menu.get("_message").visible, "Der Speicherfehler wird im Menü nicht angezeigt.")
	_check(DirAccess.remove_absolute(session) == OK, "Vorbereiteter Speicherfehler konnte nicht entfernt werden.")
	_click(menu.buttons.main_menu)
	_check(menu.visible and game.state.is_empty(), "Der Rückweg ins Hauptmenü fehlt.")
	_check(not menu.buttons.continue_game.disabled, "Der Fortsetzungsstand fehlt.")
	game.free()
	game = _launch()
	await process_frame
	await process_frame
	_click(game.hud.session_menu.buttons.continue_game)
	_check(game.state.get("time") == day and game.state.get("money") == money, "Fortsetzen lädt den gesicherten Zustand nicht.")
	_key(KEY_ESCAPE)
	await process_frame
	_click(game.hud.session_menu.buttons.main_menu)
	await process_frame
	_click(game.hud.session_menu.buttons.load_game)
	var dialog: FileDialog = game.hud.load_dialog
	_check(dialog.visible, "Das Hauptmenü öffnet den Ladedialog nicht.")
	dialog.current_path = ProjectSettings.globalize_path("res://opening/good_times.scn.gz")
	dialog.get_ok_button().pressed.emit()
	await process_frame
	_check(not game.state.is_empty() and not game.hud.session_menu.visible and game.state.get("time") != day,
		"Der klassische Spielstand wird aus dem Hauptmenü nicht geladen.")
	game.free()
	await process_frame
	if failures == 0:
		print("OK: Hauptmenü, Szenariowahl, Pause, Rückweg und Fortsetzen.")
	quit(1 if failures else 0)
