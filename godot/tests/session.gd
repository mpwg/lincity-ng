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

func _run() -> void:
	var session := ProjectSettings.globalize_path("user://Fortsetzung.scn.gz")
	if FileAccess.file_exists(session) or DirAccess.dir_exists_absolute(session):
		printerr("Für den Test ist ein frisches Benutzerdatenverzeichnis erforderlich.")
		quit(2)
		return
	var game := _launch()
	await process_frame
	if not game.hud.get("session_menu"):
		_check(false, "Das reguläre Spiel startet ohne Hauptmenü.")
		quit(1)
		return
	var menu: Control = game.hud.session_menu
	_check(menu.visible and game.state.is_empty(), "Beim ersten Start darf keine Stadt bereits laufen.")
	_check(menu.buttons.continue_game.disabled, "Fortsetzen ist ohne Spielstand verfügbar.")
	menu.buttons.new_game.pressed.emit()
	_check(game.hud.new_dialog.visible, "Neues Spiel öffnet die Szenarioauswahl nicht.")
	game.hud.scenarios.select(1)
	game.hud.new_dialog.confirmed.emit()
	game.hud.new_dialog.hide()
	await process_frame
	_check(not menu.visible and game.state.side == 100, "Das gewählte Szenario startet nicht.")
	game.speed = 0
	var day: int = game.state.time
	var money: int = game.state.money
	game.hud._menu_action(15)
	_check(menu.visible, "Das Pausenmenü lässt sich nicht öffnen.")
	menu.buttons.resume.pressed.emit()
	_check(not menu.visible and game.state.time == day, "Fortfahren ersetzt den Stand.")
	game.hud._menu_action(15)
	_check(DirAccess.make_dir_absolute(session) == OK, "Speicherfehler konnte nicht vorbereitet werden.")
	menu.buttons.main_menu.pressed.emit()
	_check(menu.in_game and game.state.time == day, "Bei einem Speicherfehler geht die laufende Stadt verloren.")
	_check(menu.get("_message").visible, "Der Speicherfehler wird im Menü nicht angezeigt.")
	_check(DirAccess.remove_absolute(session) == OK, "Vorbereiteter Speicherfehler konnte nicht entfernt werden.")
	menu.buttons.main_menu.pressed.emit()
	_check(menu.visible and game.state.is_empty(), "Der Rückweg ins Hauptmenü fehlt.")
	_check(not menu.buttons.continue_game.disabled, "Der Fortsetzungsstand fehlt.")
	game.free()
	game = _launch()
	await process_frame
	game.hud.session_menu.buttons.continue_game.pressed.emit()
	_check(game.state.get("time") == day and game.state.get("money") == money, "Fortsetzen lädt den gesicherten Zustand nicht.")
	game.free()
	await process_frame
	if failures == 0:
		print("OK: Hauptmenü, Szenariowahl, Pause, Rückweg und Fortsetzen.")
	quit(1 if failures else 0)
