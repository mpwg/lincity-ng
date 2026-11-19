extends SceneTree

var errors := 0

func check(condition: bool, description: String) -> void:
	if not condition:
		printerr(description)
		errors += 1

func _initialize() -> void:
	call_deferred("_test")

func _test() -> void:
	var core := CityWorld.new()
	if not core.has_method("history"):
		check(false, "CityWorld liefert keinen Statistikverlauf.")
		quit(1)
		return
	var fixture := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-history", "xml")
	fixture.store_string("""<lc-game ldsv-version="2130"><globals>
	<world_side_len>32</world_side_len><total_time>399</total_time><tech_level>12000</tech_level>
	<total_money>1700</total_money><monthgraph_pop>29 28 27</monthgraph_pop>
	<monthgraph_ppool>22 20 10</monthgraph_ppool><monthgraph_starve>0 10 20</monthgraph_starve>
	<monthgraph_nojobs>1 2 3</monthgraph_nojobs>
	<pbar id="1"><diff>200</diff><data>7000 7250 7500</data></pbar>
	<pbar id="2"><diff>-45</diff><data>-326</data></pbar>
	<pbar id="3"><diff>40</diff><data>660</data></pbar>
	<pbar id="9"><diff>19</diff><data>275</data></pbar>
	<pbar id="14"><diff>20</diff><data>910</data></pbar>
	</globals><map/></lc-game>""")
	fixture.flush()
	check(core.load_city(fixture.get_path_absolute()) == "", "Die historische Stadt konnte nicht geladen werden.")
	var data: Dictionary = core.call("history")
	check(data.last_month == 2, "Der laufende April darf nicht als abgeschlossen erscheinen.")
	check(data.series.population.is_empty() and data.series.unemployed.is_empty(), "Historische Diagrammindizes werden als Personen angezeigt.")
	check(data.series.balance[0] == -326 and data.series.balance[1] == -281, "Historischer Kassenstand fehlt oder wurde durch den aktuellen Kontostand ersetzt.")
	check(is_nan(data.series.balance[2]) and data.series.births.is_empty(), "Fehlende Werte des Altspielstands wurden als Null ausgegeben.")
	check(data.series.tech[0] == 0.75 and data.series.pollution[0] == 275, "Technik oder Luftverschmutzung fehlt.")
	check(data.series.occupancy[0] == 91.0 and data.commodities[0]["values"][0] == 66.0, "Alte Wohnraum- und Warenquoten wurden falsch umgerechnet.")
	check(data.sustainability.map(func(criterion: Dictionary): return criterion.goal) == [180, 180, 60, 60, 120, 60], "Ein Nachhaltigkeitskriterium fehlt oder hat den falschen Zielwert.")
	check(core.step() == "", "Der Monatswechsel konnte nicht ausgeführt werden.")
	var advanced: Dictionary = core.call("history")
	check(advanced.last_month == 3 and advanced.series.tech[1] == 0.75, "Ein neuer Monat wurde nicht an der richtigen Stelle ergänzt.")
	check(is_finite(advanced.series.population[0]) and is_nan(advanced.series.population[1]), "Alte Diagrammindizes wurden als nachträgliche Messwerte übernommen.")
	check(advanced.series.births[0] == 0.0 and is_nan(advanced.series.births[1]), "Eine neue echte Null oder die alte Lücke ging verloren.")
	var saved := "user://lincity-history-%d.gz" % Time.get_ticks_usec()
	check(core.save_city(saved) == "" and core.load_city(saved) == "", "Der Verlauf konnte nicht erneut geladen werden.")
	if FileAccess.file_exists(saved):
		DirAccess.remove_absolute(saved)
	var reloaded: Dictionary = core.call("history")
	check(reloaded.series.births[0] == 0.0 and is_nan(reloaded.series.births[1]), "Der Speicherzyklus verwechselte einen fehlenden Wert mit Null.")
	await _test_dialog(data)
	_test_person_days(core)
	var scenario := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-opening-history", "gz")
	scenario.store_buffer(FileAccess.get_file_as_bytes("res://scenarios/am_fluss.scn.gz"))
	scenario.flush()
	check(core.load_city(scenario.get_path_absolute()) == "", "Das Einstiegsszenario konnte nicht geladen werden.")
	var opening: Dictionary = core.history()
	check(opening.series.population.is_empty() and is_finite(opening.series.balance[0]) and is_nan(opening.series.balance[2]), "Das Einstiegsszenario zeigt Diagrammindizes oder Platzhalter als Messwerte.")
	check(core.new_city(0, 50, false, false) == "", "Neue Stadt konnte nicht angelegt werden.")
	var fresh: Dictionary = core.call("history")
	check(fresh.last_month == -1 and fresh.series.population.is_empty(), "Eine neue Stadt zeigt fiktive Monatswerte.")
	if errors == 0:
		print("OK: Monatsgrenze, historischer Spielstand, Lücken, Lagerquoten, Speicherzyklus und Verlaufsdialog.")
	quit(1 if errors else 0)

func _test_person_days(core: Object) -> void:
	var fixture := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-person-days", "xml")
	fixture.store_string("""<lc-game ldsv-version="2140"><globals><total_time>399</total_time><stats><history>
	<pop><index_0>900</index_0><index_1>800</index_1><index_2>700</index_2></pop>
	<nojobs><index_0>150</index_0><index_1>250</index_1><index_2>350</index_2></nojobs>
	<starve><index_0>0</index_0><index_2>20</index_2></starve>
	</history></stats></globals><map size="32"/></lc-game>""")
	fixture.flush()
	check(core.load_city(fixture.get_path_absolute()) == "", "Der Spielstand mit gemessenen Personentagen konnte nicht geladen werden.")
	var data: Dictionary = core.history()
	check(data.series.population == PackedFloat64Array([9.0, 8.0, 7.0]), "Einwohner je Monat wurden in falscher Reihenfolge oder Einheit angezeigt.")
	check(data.series.unemployed[0] == 1.5 and data.series.starving[0] == 0.0, "Monatsmittel oder echte Nullwerte fehlen.")
	check(is_nan(data.series.starving[1]) and data.series.starving[2] == 0.2, "Eine Lücke mitten in einer Datenreihe wurde als Null ausgegeben.")

func _test_dialog(data: Dictionary) -> void:
	var script: Script = load("res://ui/history.gd")
	if script == null:
		check(false, "Der Verlaufsdialog fehlt.")
		return
	var dialog: AcceptDialog = script.new()
	root.add_child(dialog)
	dialog.call("set_history", data)
	dialog.popup_centered(Vector2i(760, 470))
	await process_frame
	var tabs := dialog.find_child("HistoryTabs", true, false) as TabBar
	var dates := dialog.find_child("HistoryMonths", true, false) as OptionButton
	var values := dialog.find_child("HistoryValues", true, false) as RichTextLabel
	check(tabs.tab_count == 6 and dates.get_selected_id() == 2 and dates.get_item_text(dates.selected) == "März 0", "Statistikbereiche oder Monatsauswahl fehlen.")
	check(dialog.get_contents_minimum_size().y <= 470, "Der Statistikdialog ist höher als sein vorgesehener Platz.")
	tabs.current_tab = 1
	check("−326 $" in values.get_parsed_text(), "Der gewählte Monat zeigt keinen historischen Kassenstand.")
	dates.select(0)
	dates.item_selected.emit(0)
	check("Keine Daten" in values.get_parsed_text(), "Der älteste Monat zeigt einen vorgetäuschten Kassenstand.")
	tabs.current_tab = 5
	var fire := dialog.find_child("SustainabilityCriterion5", true, false) as ProgressBar
	check(fire != null and fire.is_visible_in_tree() and fire.max_value == 60.0, "Der Fortschritt für Brandschutz ist nicht sichtbar.")
	check(data.sustainability[1].name == "Handel über den Hafen", "Umlaute in Nachhaltigkeitskriterien sind falsch kodiert.")
	dialog.queue_free()
