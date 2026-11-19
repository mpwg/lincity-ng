extends SceneTree

const GameSettings = preload("res://ui/settings.gd")
const SettingsDialog = preload("res://ui/settings_dialog.gd")
const GameAudio = preload("res://audio/game_audio.gd")

var errors := 0

func _initialize() -> void:
	call_deferred("_test")

func check(ok: bool, message: String) -> void:
	if not ok:
		printerr(message)
		errors += 1

func _test() -> void:
	var path := "user://settings-test-%s.cfg" % OS.get_process_id()
	var settings := GameSettings.new(path)
	check(settings.load_from_disk() == ERR_FILE_NOT_FOUND, "Eine fehlende Konfiguration wurde nicht erkannt.")
	check(settings.values.map_side == 100 and settings.values.music_enabled, "Die erste Sitzung enthält keine benutzbaren Vorgaben.")
	var initial: Dictionary = settings.values.duplicate()
	var dialog := SettingsDialog.new()
	var audio := GameAudio.new()
	root.add_child(audio)
	audio.set_technology_percent(2.0)
	root.add_child(dialog)
	dialog.setup(settings, audio)
	dialog.popup_centered(Vector2i(590, 480))
	await process_frame
	check(root.get_last_exclusive_window() == dialog, "Die Spieleinstellungen sind nicht modal.")
	check(dialog.get_node("%WindowSize").disabled == (DisplayServer.get_name() == "Wayland"), "Die Fenstergröße ist auf diesem Grafiktreiber falsch freigegeben.")
	var playlist: HBoxContainer = dialog.get_node("%Playlist")
	var previous: Button = playlist.get_child(0)
	var track: Label = playlist.get_child(1)
	var next: Button = playlist.get_child(2)
	check(track.text == "Robert van Herk - City Blues", "Der aktuell gewählte Musiktitel fehlt.")
	next.pressed.emit()
	check(track.text == "Robert van Herk - Architectural Contemplations", "Die Schaltfläche für den nächsten Titel aktualisiert die Anzeige nicht.")
	previous.pressed.emit()
	check(track.text == "Robert van Herk - City Blues", "Die Schaltfläche für den vorherigen Titel aktualisiert die Anzeige nicht.")
	_set_values(dialog)
	dialog.get_cancel_button().pressed.emit()
	await process_frame
	check(not dialog.visible and settings.values == initial and not FileAccess.file_exists(path), "Abbrechen hat den Dialog nicht geschlossen oder Änderungen gespeichert.")
	dialog.popup_centered(Vector2i(590, 480))
	check(dialog.get_node("%MapSide").value == 100, "Das Menü zeigt weiterhin die abgebrochenen Änderungen.")
	_set_values(dialog)
	var applied: Array[bool] = []
	dialog.settings_applied.connect(func(): applied.append(true))
	dialog.get_ok_button().pressed.emit()
	check(applied == [true] and not dialog.visible, "Das Menü meldet erfolgreich gespeicherte Änderungen nicht.")
	var restarted := GameSettings.new(path)
	check(restarted.load_from_disk() == OK, "Die gespeicherte Konfiguration konnte nicht erneut geöffnet werden.")
	check(restarted.values.fullscreen and not restarted.values.shadows and not restarted.values.reflections and restarted.values.msaa == 0, "Grafikeinstellungen wurden nicht wiederhergestellt.")
	check(restarted.values.map_climate == 2 and restarted.values.map_side == 225 and not restarted.values.map_village and not restarted.values.map_trees, "Vorgaben für neue Städte wurden nicht wiederhergestellt.")
	check(not restarted.values.music_enabled and is_equal_approx(restarted.values.music_volume, 0.7) and not restarted.values.effects_enabled and is_equal_approx(restarted.values.effects_volume, 0.35), "Toneinstellungen wurden nicht wiederhergestellt.")
	var was_minimized := root.mode == Window.MODE_MINIMIZED
	restarted.apply_display(root)
	check(root.msaa_3d == Viewport.MSAA_DISABLED, "Die 3D-Ansicht hat die neue Kantenglättung nicht übernommen.")
	if DisplayServer.get_name() != "headless":
		await create_timer(0.25).timeout
		check(root.mode == Window.MODE_FULLSCREEN, "Das Spiel ist nicht im Vollbild.")
	check(restarted.save_changes({"fullscreen": false, "window_size": Vector2i(1280, 720)}) == OK and settings.load_from_disk() == OK and settings.values.window_size == Vector2i(1280, 720), "Die gewählte Fenstergröße wurde nicht wiederhergestellt.")
	if DisplayServer.get_name() != "headless":
		settings.apply_display(root)
		await create_timer(0.25).timeout
		check(root.mode == Window.MODE_WINDOWED or (was_minimized and root.mode == Window.MODE_MINIMIZED), "Der Wechsel aus dem Vollbild funktioniert nicht (Modus %d)." % root.mode)
		if DisplayServer.get_name() != "Wayland":
			check(root.size == Vector2i(1280, 720), "Der Wechsel aus dem Vollbild übernimmt die gewählte Fenstergröße nicht (%s)." % str(root.size))
	check(restarted.values.map_side == 225, "Eine einzelne Änderung hat andere Einstellungen überschrieben.")
	var corrupt := ConfigFile.new()
	corrupt.set_value("display", "window_size", Vector2i(-1, 2))
	corrupt.set_value("display", "fullscreen", "false")
	corrupt.set_value("audio", "music_volume", 37.0)
	corrupt.set_value("new_city", "map_side", 225)
	check(corrupt.save(path) == OK and settings.load_from_disk() == OK, "Die teilweise ungültige Konfiguration konnte nicht gelesen werden.")
	check(settings.values.window_size == Vector2i(1440, 900) and not settings.values.fullscreen and settings.values.music_volume == 0.5 and settings.values.map_side == 225, "Ungültige Werte wurden übernommen oder gültige Werte verworfen.")
	var unwritable := GameSettings.new(path.path_join("settings.cfg"))
	check(unwritable.save_changes({"map_side": 225}) != OK and unwritable.values.map_side == 100, "Ein fehlgeschlagener Schreibvorgang verändert die aktiven Einstellungen.")
	dialog.free()
	audio.free()
	await _test_city_menu(path, restarted)
	DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
	if errors == 0:
		print("OK: Spieleinstellungen über Menü, Vollbild, Musikwechsel, Speichern, erneutes Laden und Vorgaben für neue Städte.")
	quit(1 if errors else 0)

func _test_city_menu(path: String, settings: GameSettings) -> void:
	check(settings.save_changes({"map_side": 75, "shadows": false, "reflections": false}) == OK, "Die Spielkonfiguration für den Neustart konnte nicht geschrieben werden.")
	var scene: PackedScene = load("res://city.tscn")
	var game = scene.instantiate()
	game.settings = GameSettings.new(path)
	root.add_child(game)
	game.set_process(false)
	check(not game.get_node("Sun").shadow_enabled and not game.view.reflections_enabled, "Die Stadt hat die Grafikvorgaben beim Start nicht übernommen.")
	var hud = game.hud
	var game_menu: MenuButton = hud.find_children("*", "MenuButton", true, false).front()
	var popup := game_menu.get_popup()
	for index in popup.item_count:
		if popup.get_item_text(index).begins_with("Einstellungen"):
			popup.id_pressed.emit(popup.get_item_id(index))
			break
	check(hud.settings_dialog.visible and hud.has_modal(), "Das Spielmenü öffnet die Einstellungen nicht modal.")
	hud.settings_dialog.get_cancel_button().pressed.emit()
	await process_frame
	popup.id_pressed.emit(0)
	check(hud.new_dialog.visible and hud.extent.value == 75 and hud.climate.selected == 2 and not hud.village.button_pressed and not hud.trees.button_pressed, "Das Neuspielmenü übernimmt die gespeicherten Stadtvorgaben nicht.")
	game.free()

func _set_values(dialog: Window) -> void:
	dialog.get_node("%Fullscreen").button_pressed = true
	dialog.get_node("%Shadows").button_pressed = false
	dialog.get_node("%Reflections").button_pressed = false
	dialog.get_node("%Antialiasing").select(0)
	dialog.get_node("%MapClimate").select(2)
	dialog.get_node("%MapSide").value = 225
	dialog.get_node("%MapVillage").button_pressed = false
	dialog.get_node("%MapTrees").button_pressed = false
	dialog.get_node("%MusicEnabled").button_pressed = false
	dialog.get_node("%MusicVolume").value = 70
	dialog.get_node("%EffectsEnabled").button_pressed = false
	dialog.get_node("%EffectsVolume").value = 35
