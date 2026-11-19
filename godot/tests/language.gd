extends SceneTree

const GameSettings = preload("res://ui/settings.gd")
const SettingsDialog = preload("res://ui/settings_dialog.gd")
const HelpBook = preload("res://ui/help_book.gd")

var failures := 0

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		printerr(message)
		failures += 1

func _run() -> void:
	var path := "user://language-test-%s.cfg" % OS.get_process_id()
	var settings := GameSettings.new(path)
	settings.load_from_disk()
	_check(settings.values.language == "de", "Eine neue Konfiguration startet auf Deutsch.")
	settings.apply_language()
	var dialog := SettingsDialog.new()
	root.add_child(dialog)
	dialog.setup(settings, null)
	dialog.popup_centered()
	var option := dialog.get_node_or_null("%Language") as OptionButton
	_check(option != null, "Die Sprache kann in den Einstellungen gewählt werden.")
	if option:
		option.select(1)
		dialog.get_ok_button().pressed.emit()
	var restarted := GameSettings.new(path)
	_check(restarted.load_from_disk() == OK and restarted.values.language == "en", "Die gewählte Sprache wird gespeichert.")
	_check(TranslationServer.get_locale() == "de", "Die laufende Sitzung wechselt ihre Sprache erst nach dem Neustart.")
	var entry: PackedScene = load(ProjectSettings.get_setting("application/run/main_scene"))
	var game := entry.instantiate()
	game.settings = restarted
	root.add_child(game)
	_check(TranslationServer.get_locale() == "en", "Englisch muss beim nächsten Start aktiv sein.")
	_check(game.hud.session_menu.buttons.new_game.text == "New game", "Das Hauptmenü muss auf Englisch erscheinen.")
	_check(game.hud.new_dialog.title == "New city", "Der Dialog im Spiel muss auf Englisch erscheinen.")
	game.free()
	var english_dialog := SettingsDialog.new()
	root.add_child(english_dialog)
	_check(english_dialog.title == "Settings", "Der Einstellungstitel muss auf Englisch erscheinen.")
	var help := HelpBook.new()
	root.add_child(help)
	_check(help.article.get_parsed_text().contains("Managing your city"), "Die Einführung muss auf Englisch erscheinen.")
	help._open("index")
	_check(help.article.get_parsed_text().contains("Blacksmiths"), "Englisch muss das englische Themenverzeichnis laden.")
	_test_simulation_text("No city loaded.", "Elevation", "Market", "This space is occupied.")
	_check(restarted.save_changes({"language": "unsupported"}) == OK, "Ungültige Sprachangaben müssen abgefangen werden.")
	restarted.apply_language()
	_check(TranslationServer.get_locale() == "de", "Ungültige Sprachangaben müssen Deutsch wählen.")
	var german_help := HelpBook.new()
	root.add_child(german_help)
	_check(german_help.article.get_parsed_text().contains("Die Stadt verwalten"), "Die Einführung muss auf Deutsch erscheinen.")
	german_help._open("index")
	_check(german_help.article.get_parsed_text().contains("Schmieden"), "Deutsch muss das deutsche Themenverzeichnis laden.")
	_test_simulation_text("Keine Stadt geladen.", "Höhe", "Markt", "Dieser Platz ist belegt.")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
	if failures == 0:
		print("OK: Sprachauswahl, Speicherung, Hilfe und Spielmeldungen auf Deutsch und Englisch")
	quit(1 if failures else 0)

func _test_simulation_text(no_city: String, elevation: String, market_name: String, occupied: String) -> void:
	var core := CityWorld.new()
	_check(core.step() == no_city, "Die Fehlermeldung der Erweiterung entspricht nicht der gewählten Sprache.")
	var fixture := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-language", "gz")
	fixture.store_buffer(FileAccess.get_file_as_bytes("res://scenarios/am_fluss.scn.gz"))
	fixture.flush()
	if core.load_city(fixture.get_path_absolute()) != "":
		_check(false, "Das Referenzszenario konnte zum Sprachtest nicht geladen werden.")
		return
	var state := core.snapshot()
	var side: int = state.side
	var empty := (state.tiles as PackedInt32Array).find(0)
	var market := (state.tiles as PackedInt32Array).find(6)
	if empty < 0 or market < 0:
		_check(false, "Das Referenzszenario braucht einen Markt und ein unbebautes Feld.")
		return
	var ground_cell := Vector2i(empty % side, floori(float(empty) / side))
	var market_cell := Vector2i(market % side, floori(float(market) / side))
	_check(elevation in core.inspect_tile(ground_cell), "Der Geländefeldbericht entspricht nicht der gewählten Sprache.")
	_check(core.inspect_tile(market_cell).begins_with(market_name), "Der Gebäudebericht entspricht nicht der gewählten Sprache.")
	_check(occupied in core.preview(6, market_cell).get("reason", ""), "Die Bauwarnung entspricht nicht der gewählten Sprache.")
