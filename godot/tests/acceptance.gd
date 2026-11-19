extends SceneTree

var errors := 0

func check(condition: bool, description: String) -> void:
	if not condition:
		printerr(description)
		errors += 1

func _initialize() -> void:
	call_deferred("_test")

func _test() -> void:
	var game = load("res://city.tscn").instantiate()
	root.add_child(game)
	game.speed = 0
	if game.state.is_empty():
		printerr("Kein Spielstand geladen.")
		quit(1)
		return
	var snapshot: Dictionary = game.state
	var side := int(snapshot.side)
	var i := 28 * side + 76
	if i >= snapshot.tiles.size() or snapshot.tiles[i] != 6:
		i = -1
	check(i >= 0, "Der Referenzstand enthält keinen Markt.")
	if i < 0:
		quit(1)
		return
	var anchor: int = snapshot.anchors[i]
	var market := Vector2i(anchor % side, floori(float(anchor) / side))
	game.camera.center_on(market)
	var target := Vector3(market.x + 0.5, 0, market.y + 0.5)
	for angle in [0, 90, 180, 270]:
		game.camera.azimuth = deg_to_rad(angle)
		game.camera.update_pose()
		var screen: Vector2 = game.camera.unproject_position(target)
		check(game.camera.tile_at(screen) == market, "Falsches Baufeld bei %s°" % angle)
		var roof := target + Vector3(0.5, 0.38, 0.5)
		screen = game.camera.unproject_position(roof)
		var picked: Vector2i = game.view.pick_building(game.camera.project_ray_origin(screen), game.camera.project_ray_normal(screen))
		check(picked == market, "Marktauswahl bei %s° ergibt %s, erwartet %s" % [angle, picked, market])
	var cell := Vector2i(-1, -1)
	for y in range(1, side - 3):
		for x in range(1, side - 3):
			var p := Vector2i(x, y)
			if x % 12 != 11 or snapshot.tiles[y * side + x] == 21 or snapshot.tiles[y * side + x + 1] == 21:
				continue
			if game.core.preview(11, p).allowed and game.core.preview(11, p + Vector2i.RIGHT).allowed:
				cell = p
				break
		if cell.x >= 0: break
	check(cell.x >= 0, "Keine baubare Straße gefunden.")
	if cell.x >= 0:
		var cells: Array = game.selection(cell, cell + Vector2i.RIGHT, 11, 1, false)
		check(cells.size() == 2, "Falsche Straßenlänge.")
		var money := int(game.core.snapshot().money)
		var total := 0
		for p in cells:
			total += int(game.core.preview(11, p).cost)
			check(game.core.apply(11, p) == "", "Straßenbau fehlgeschlagen.")
		var result: Dictionary = game.core.snapshot()
		check(result.money == money - total, "Abbuchung weicht von der Vorschau ab.")
		check(result.tiles[cell.y * side + cell.x] == 11 and result.tiles[cell.y * side + cell.x + 1] == 11, "Straße über Bereichsgrenze fehlt.")
		game._set_layer(1, 0)
		check(game.state.overlay_colors.size() == side * side, "Kartenebene ist unvollständig.")
		game._set_layer(0, 0)
		var path := "user://Städtchen-%d.gz" % Time.get_ticks_usec()
		check(game.core.save_city(path) == "", "Speichern fehlgeschlagen.")
		check(FileAccess.file_exists(path), "Spielstand mit Umlaut fehlt.")
		check(game.core.save_city(path) == "", "Überschreiben fehlgeschlagen.")
		check(game.core.load_city(path) == "", "Laden fehlgeschlagen.")
		if FileAccess.file_exists(path):
			DirAccess.remove_absolute(path)
		var loaded: Dictionary = game.core.snapshot()
		check(loaded.tiles == result.tiles and loaded.money == result.money, "Spielstand hat sich beim Laden verändert.")
		check(game.core.step() == "", "Fortsetzen nach dem Laden fehlgeschlagen.")
		check(game.core.apply(-2, cell) == "", "Straßenabriss fehlgeschlagen.")
		game._refresh()
		check(game.view.tiles[cell.y * side + cell.x] != 11, "Abgerissene Straße wird noch angezeigt.")
	_test_factory_status(game.core)
	_test_rocket_and_coal(game)
	if errors == 0:
		print("OK: Referenzmarkt aus vier Richtungen, Bereichsgrenze, Abriss, Kartenebenen, Betriebswechsel, Raketenstart und Spielstand.")
	quit(1 if errors else 0)

func _test_factory_status(core: Object) -> void:
	# The detached factory has exactly one day's ore and no supplier.
	var fixture := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-factory", "xml")
	fixture.store_string("""<lc-game ldsv-version="2140"><globals><total_time>1</total_time></globals>
	<map size="40"><Construction group="12" map-x="4" map-y="4">
	<labor>400</labor><ore>125</ore></Construction></map></lc-game>""")
	fixture.flush()
	check(core.load_city(fixture.get_path_absolute()) == "", "Testfabrik konnte nicht geladen werden.")
	var index := 4 * 40 + 4
	for expected in [0, 1, 0]:
		var state: Dictionary = core.snapshot()
		check(state.operating.size() == 1600 and state.operating[index] == expected, "Falscher Betriebszustand am Fabrikanker.")
		check(state.tiles[index] == 12, "Betriebswechsel hat den Gebäudetyp verändert.")
		check(core.step() == "", "Fabrik konnte nicht simuliert werden.")

func _test_rocket_and_coal(game: Node) -> void:
	# A stocked pad needs one construction step. The isolated plant has exactly
	# one day's coal at base technology, so its plume must disappear on day two.
	var fixture := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-launch", "xml")
	fixture.store_string("""<lc-game ldsv-version="2140"><globals><total_time>1</total_time>
	<tech_level>1000000</tech_level><rockets_launched>10</rockets_launched>
	<total_money>1000000</total_money></globals><map size="40">
	<Construction group="23" map-x="4" map-y="4"><steps>4999</steps>
	<labor>200</labor><steel>240</steel><goods>10000</goods></Construction>
	<Construction group="10" map-x="16" map-y="4"><tech>0</tech><labor>100</labor>
	<coal>44</coal></Construction></map></lc-game>""")
	fixture.flush()
	game._load_city(fixture.get_path_absolute())
	var pad := 4*40+4
	var plant := 4*40+16
	game.selected = Vector2i(5,5)
	game._inspect()
	check(game.state.rocket_frames.get(pad) == 3 and game.hud.launch_button.disabled, "Rakete im Bau ist startbereit.")
	check(_pad_variant(game) == 3, "Bauzustand fehlt am Raketenmodell.")
	check(game.state.operating[plant] == 0, "Kohlekraftwerk vor der ersten Produktion aktiv.")
	check(game.core.step() == "", "Fertigstellung der Rakete fehlgeschlagen.")
	game._refresh()
	check(game.state.operating[plant] == 1, "Keine Schornsteinfahne am produzierenden Kohlekraftwerk.")
	check(game.state.rocket_frames.get(pad) == 4 and _pad_variant(game) == 4 and not game.hud.launch_button.disabled, "Fertige Rakete nicht sichtbar oder anwählbar.")
	game.hud.launch_requested.emit()
	check(game.state.rocket_frames.get(pad) == 5 and _pad_variant(game) == 5, "Start trennt Rakete und Rampe nicht.")
	check(game.view._flights.has(pad), "Flugmodell fehlt.")
	game.speed = 1
	game._process(.1)
	if game.view._flights.has(pad):
		check(game.view._flights[pad].position.y > 0, "Rakete hebt nicht ab.")
	for i in 15:
		game._process(.1)
	game.speed = 0
	check(game.state.operating[plant] == 0, "Kohlekraftwerk ohne Brennstoff zeigt Rauch.")
	check(game.state.rocket_frames.get(pad) == 7 and game.state.rockets == 11, "Start wurde nicht genau einmal abgeschlossen.")
	check(_pad_variant(game) == 5 and game.view._flights.is_empty() and game.hud.launch_button.disabled, "Verbrauchter Startplatz zeigt weiterhin eine Rakete.")

func _pad_variant(game: Node) -> int:
	for batch in game.view._chunks[Vector2i.ZERO].batches:
		if floori(batch.key/16.0) == 23:
			return batch.key%16
	return -1
