extends SceneTree

var failures := 0

func _initialize() -> void:
	call_deferred("_run")

func _check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		printerr(message)

func _run() -> void:
	TranslationServer.set_locale("de")
	var game = preload("res://city.tscn").instantiate()
	root.add_child(game)
	game.set_process(false)
	var fixture := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-statistik", "xml")
	var city_xml := """<lc-game ldsv-version="2140"><globals>
		<given_scene>Referenzstadt</given_scene><total_time>1202</total_time>
		<total_money>%d</total_money><people_pool>42</people_pool>
		<tech_level>400000</tech_level><rockets_launched>8</rockets_launched>
		<rockets_launched_success>5</rockets_launched_success>
		<stats><highest_tech_level>550000</highest_tech_level><population>
		<evacuated_total>123</evacuated_total><max_pop_ever>97</max_pop_ever>
		</population></stats></globals><map size="40">
		<Construction group="6" map-x="4" map-y="4"><food>128</food></Construction>
		<Construction group="6" map-x="9" map-y="4"><food>32</food></Construction>
		</map></lc-game>"""
	fixture.store_string(city_xml % -1999999999)
	fixture.flush()
	game._load_city(fixture.get_path_absolute())
	var state: Dictionary = game.state
	if state.get("side") != 40:
		printerr("Teststadt konnte nicht geladen werden.")
		game.free()
		quit(1)
		return
	_check(state.get("evacuated") == 123 and state.get("successful_rockets") == 5, "Historische Stadtstatistik fehlt.")
	_check(state.get("highest_tech_percent") == 55.0, "Der Technologierekord wurde falsch skaliert.")
	var inventory: Array = state.get("inventory", [])
	_check(inventory.size() == 10, "Nicht alle stadtweiten Warenbestände sind verfügbar.")
	if inventory.size() == 10:
		_check(inventory[0].amount == 160 and inventory[0].capacity == 524288, "Bestände beider Märkte wurden nicht genau einmal erfasst.")
	var counts: PackedInt32Array = state.get("building_counts", PackedInt32Array())
	_check(counts.size() > 6 and counts[6] == 2, "Die Statistik zählt Gebäudegrundrisse mehrfach oder gar nicht.")

	var hud = game.hud
	hud._menu_action(4)
	var report: String = hud.report_body.text
	_check("Evakuierte" in report and "123" in report and "Erfolgreiche Starts" in report, "Historische Details fehlen im Spielerbericht.")
	_check("Märkte" in report, "Gebäudezahlen fehlen im Spielerbericht.")
	var tech: Label = hud.get("technology")
	_check(tech != null and tech.is_visible_in_tree() and "40" in tech.text, "Die Technologie ist während des Spiels nicht sichtbar.")
	var export_button: Button = hud.get("report_export")
	var export_dialog: FileDialog = hud.get("report_save_dialog")
	_check(export_button != null and export_button.is_visible_in_tree(), "Der Statistikexport ist nicht erreichbar.")
	if export_button and export_dialog:
		var output := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-statistik-export", "txt")
		export_button.pressed.emit()
		_check(export_dialog.visible, "Der Statistikexport öffnet keine Dateiauswahl.")
		export_dialog.file_selected.emit(output.get_path_absolute())
		_check("123" in FileAccess.get_file_as_string(output.get_path_absolute()), "Der Export schreibt die Stadtstatistik nicht in die gewünschte Datei.")
		export_dialog.hide()
	hud.report_dialog.hide()

	var supply: Button = hud.get("supply_button")
	_check(supply != null and supply.is_visible_in_tree(), "Die Warenversorgung ist während des Spiels nicht erreichbar.")
	if supply:
		_check(supply.text.contains("Lebensmittel") and supply.text.contains("Wasser"),
			"Die Versorgung mit Lebensmitteln und Wasser ist während des Bauens nicht sichtbar.")
		supply.pressed.emit()
		_check("Lebensmittel" in hud.report_body.text and "160" in hud.report_body.text, "Das Warenfenster zeigt keinen aktuellen Bestand.")
		hud.report_dialog.hide()

	_check(not game.core.survey_coal().is_empty(), "Kohleerkundung hätte am Kreditlimit scheitern müssen.")
	var warnings: Array = game.core.take_events()
	_check(warnings.size() == 1 and warnings[0].get("kind") == "out_of_credit", "Unerwartete Ereignisse am Kreditlimit: %s" % str(warnings))
	for event in warnings:
		hud.show_event(event)
	var warning: PanelContainer = hud.get("warning_panel")
	_check(warning != null and warning.is_visible_in_tree(), "Finanzwarnung erscheint nicht sichtbar.")
	var funded := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-statistik-kredit", "xml")
	funded.store_string(city_xml % 500000)
	funded.flush()
	game._load_city(funded.get_path_absolute())
	if warning:
		_check(not warning.visible, "Beim Stadtwechsel bleiben alte Warnungen sichtbar.")
	_check(game.core.survey_coal().is_empty(), "Die Kohleerkundung auf Kredit ist fehlgeschlagen.")
	warnings = game.core.take_events()
	_check(warnings.size() == 1 and warnings[0].get("kind") == "out_of_money", "Der Übergang ins Minus meldet keine leere Stadtkasse.")
	var target: Array[Vector2i] = []
	hud.focus_requested.connect(func(cell: Vector2i): target.append(cell))
	hud.show_event({"kind": "fire", "cell": Vector2i(4, 4), "group": 6})
	var locate: Button = hud.get("warning_focus")
	if locate:
		locate.pressed.emit()
	_check(target == [Vector2i(4, 4)], "Die Feuerwarnung führt nicht zum betroffenen Gebäude.")
	if failures == 0:
		print("OK: sichtbare Technologie, Warenversorgung, Stadtstatistik, Export und Warnungen.")
	game.free()
	quit(1 if failures else 0)
