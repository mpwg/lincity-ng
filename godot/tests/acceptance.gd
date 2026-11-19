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
	var i: int = snapshot.tiles.find(6)
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
		var tmp := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-test", "gz")
		var path := tmp.get_path_absolute()
		check(game.core.save_city(path) == "", "Speichern fehlgeschlagen.")
		check(game.core.load_city(path) == "", "Laden fehlgeschlagen.")
		var loaded: Dictionary = game.core.snapshot()
		check(loaded.tiles == result.tiles and loaded.money == result.money, "Spielstand hat sich beim Laden verändert.")
		check(game.core.step() == "", "Fortsetzen nach dem Laden fehlgeschlagen.")
	if errors == 0:
		print("OK: vier Kamerarichtungen, Bauvorschau, Kosten, Spielstand und Fortsetzung.")
	quit(1 if errors else 0)
