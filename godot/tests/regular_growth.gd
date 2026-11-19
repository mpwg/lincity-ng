extends SceneTree

const MARKET := 6
const POTTERY := 29
const MONUMENT := 25
const SCHOOL := 26
const GOODS := 3
const INVENTORY_LAYER := 10
const NO_CELL := Vector2i(-1, -1)
const NEW_CITY_ATTEMPTS := 3
const MAX_RESEARCH_DAYS := 15_000

func _initialize() -> void:
	call_deferred("_run")

func _market(state: Dictionary) -> Vector2i:
	var side := int(state.side)
	for i in state.tiles.size():
		if state.tiles[i] == MARKET and state.anchors[i] == i:
			return Vector2i(i % side, floori(float(i) / side))
	return NO_CELL

func _market_site(core: Object, group: int, market: Vector2i) -> Vector2i:
	for distance in range(2, 8):
		for y in range(-distance, distance + 1):
			for x in range(-distance, distance + 1):
				if maxi(absi(x), absi(y)) != distance:
					continue
				var site := market + Vector2i(x, y)
				if core.preview(group, site).allowed:
					return site
	return NO_CELL

func _tool(game, id: int) -> Button:
	var hud = game.hud
	if hud.catalog_panel.visible:
		hud.close_catalog()
	for category: String in hud.categories:
		if id in hud.categories[category]:
			hud.category_buttons[category].pressed.emit()
			for button: Button in hud.tool_list.get_children():
				if button.get_meta("tool_id") == id:
					return button
	return null

func _build(game, group: int, site: Vector2i) -> bool:
	game.camera.center_on(site)
	var position: Vector2 = game.camera.unproject_position(Vector3(site.x + 0.5, 0, site.y + 0.5))
	var move := InputEventMouseMotion.new()
	move.position = position
	root.push_input(move, true)
	for down: bool in [true, false]:
		var click := InputEventMouseButton.new()
		click.button_index = MOUSE_BUTTON_LEFT
		click.position = position
		click.pressed = down
		root.push_input(click, true)
	return game.state.tiles[site.y * int(game.state.side) + site.x] == group

func _advance(core: Object, days: int) -> String:
	for day in days:
		var error: String = core.step()
		if not error.is_empty():
			return error
	return ""

func _new_village(game) -> void:
	var hud = game.hud
	if hud.session_menu.visible:
		hud.session_menu.buttons.new_game.pressed.emit()
	else:
		hud._menu_action(0)
	hud.scenarios.select(0)
	hud.climate.select(2)
	hud.extent.value = 50
	hud.village.button_pressed = true
	hud.trees.button_pressed = false
	hud.new_dialog.confirmed.emit()
	hud.new_dialog.hide()
	hud.speed_buttons[4].pressed.emit()

func _play_growth(game) -> String:
	_new_village(game)
	var core: Object = game.core
	var hud = game.hud
	if game.state.is_empty() or int(game.state.time) != 0 or hud.session_menu.visible:
		return "Das Hauptmenü hat keine neue spielbare Stadt erzeugt."
	var market := _market(game.state)
	if market == NO_CELL:
		return "Das erzeugte Dorf hat keinen Markt."
	var initial_population := int(game.state.population)
	var school_tool := _tool(game, SCHOOL)
	if not school_tool or not school_tool.disabled:
		return "Die Schule ist am ersten Tag nicht gesperrt."
	var required_tech := int(school_tool.get_meta("required_tech"))
	var monument := _market_site(core, MONUMENT, market)
	var monument_tool := _tool(game, MONUMENT)
	if monument == NO_CELL or not monument_tool or monument_tool.disabled:
		return "Das Startdorf hat keinen verfügbaren Standort für ein Denkmal."
	monument_tool.pressed.emit()
	if game.tool != MONUMENT or not _build(game, MONUMENT, monument):
		return "Das Denkmal ließ sich über den Baukatalog nicht errichten."

	# Simulate the ordinary native day without rebuilding the display after every day.
	var progress: Dictionary = game.state
	while int(progress.tech) < required_tech and int(progress.time) < MAX_RESEARCH_DAYS:
		var error := _advance(core, 100)
		if not error.is_empty():
			return error
		progress = core.snapshot()
	game._refresh()
	if int(progress.tech) < required_tech or int(progress.population) <= initial_population:
		return "Kein frühes Wachstum auf dieser Karte (%d Einwohner, Technologie %d nach Tag %d)." % [
			progress.population, progress.tech, progress.time]
	var school := _market_site(core, SCHOOL, market)
	school_tool = _tool(game, SCHOOL)
	if school == NO_CELL or not school_tool or school_tool.disabled:
		return "Schule trotz erreichter Technologie weiterhin gesperrt."
	school_tool.pressed.emit()
	if game.tool != SCHOOL or not _build(game, SCHOOL, school):
		return "Die freigeschaltete Schule ließ sich nicht bauen."
	# A second pottery keeps the education supply independent of the first one's location.
	var pottery := _market_site(core, POTTERY, market)
	if pottery != NO_CELL and int(game.state.money) > 5000:
		var pottery_tool := _tool(game, POTTERY)
		if not pottery_tool or pottery_tool.disabled:
			return "Die Töpferei fehlt im Baukatalog."
		pottery_tool.pressed.emit()
		if game.tool != POTTERY or not _build(game, POTTERY, pottery):
			return "Die zweite Töpferei ließ sich nicht errichten."
	# The field report rounds research to 0.1%, so it needs several productive months.
	var error := _advance(core, 2400)
	if not error.is_empty():
		return error
	game._refresh()
	game.selected = school
	game._inspect()
	var lesson_label := String(TranslationServer.translate(&"Lessons learned"))
	var lessons := RegEx.create_from_string(lesson_label + "\\s+([0-9.]+)%").search(hud.details.text)
	if not hud.inspector.visible or not lessons or float(lessons.get_string(1)) <= 0.0:
		return "Der Feldbericht zeigt keine tatsächlichen Forschungsleistungen der Schule:\n" + hud.details.text
	hud.layers.select(INVENTORY_LAYER)
	hud.layers.item_selected.emit(INVENTORY_LAYER)
	hud.commodities.select(GOODS)
	hud.commodities.item_selected.emit(GOODS)
	var side := int(game.state.side)
	var colors: PackedColorArray = game.state.get("overlay_colors", PackedColorArray())
	if game.state.overlay_id != INVENTORY_LAYER or colors.size() != side * side:
		return "Die Warenversorgung lässt sich nicht auf der Karte prüfen."
	var school_color := colors[school.y * side + school.x]
	if is_equal_approx(school_color.r, school_color.g) and is_equal_approx(school_color.g, school_color.b):
		return "Die Warenversorgung lässt sich nicht auf der Karte prüfen."
	if int(game.state.inventory[GOODS].amount) <= 0:
		return "Die Stadt weist nach dem Schulbau keinen Warenbestand mehr aus."
	var day := int(game.state.time)
	hud.speed_buttons[1].pressed.emit()
	game._process(0.12)
	if int(game.state.time) <= day or hud.outcome_dialog.visible:
		return "Die Stadt spielt nach dem Schulbau nicht regulär weiter."
	print("OK: frisch erzeugtes Dorf, Forschungsstufe erreicht an Tag %d, Schule versorgt und bis Tag %d weitergespielt." % [
		progress.time, game.state.time])
	return ""

func _run() -> void:
	var game = preload("res://main.tscn").instantiate()
	root.add_child(game)
	game.set_process(false)
	for attempt in NEW_CITY_ATTEMPTS:
		var error := _play_growth(game)
		if error.is_empty():
			game.free()
			quit()
			return
		printerr("Gemäßigtes Startdorf %d: %s" % [attempt + 1, error])
	game.free()
	quit(1)
