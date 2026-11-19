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
		var tmp := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-test", "gz")
		var path := tmp.get_path_absolute()
		check(game.core.save_city(path) == "", "Speichern fehlgeschlagen.")
		check(game.core.load_city(path) == "", "Laden fehlgeschlagen.")
		var loaded: Dictionary = game.core.snapshot()
		check(loaded.tiles == result.tiles and loaded.money == result.money, "Spielstand hat sich beim Laden verändert.")
		check(game.core.step() == "", "Fortsetzen nach dem Laden fehlgeschlagen.")
		check(game.core.apply(-2, cell) == "", "Straßenabriss fehlgeschlagen.")
		game._refresh()
		check(game.view.tiles[cell.y * side + cell.x] != 11, "Abgerissene Straße wird noch angezeigt.")
	_test_factory_status(game.core)
	if errors == 0:
		print("OK: Referenzmarkt aus vier Richtungen, Bereichsgrenze, Abriss, Kartenebenen, Betriebswechsel und Spielstand.")
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
