extends SceneTree
## Play saved cities through their final simulation action and exercise the outcome dialog.

var failures := 0

func _initialize() -> void:
	call_deferred("_run")

func _check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		printerr(message)

func _run() -> void:
	var game = load("res://city.tscn").instantiate()
	root.add_child(game)
	game.set_process(false)
	for scenario in [
		["evacuation", "Sieg durch Evakuierung", """<total_time>1</total_time><people_pool>10</people_pool>
			<tech_level>1000000</tech_level><rockets_launched>10</rockets_launched>
			<rockets_launched_success>5</rockets_launched_success>""",
			"""<Construction group="23" map-x="4" map-y="4"><tech>1000000</tech>
			<steps>5000</steps><stage>awaiting</stage></Construction>"""],
		["economy", "Sieg durch Nachhaltigkeit", """<total_time>1199</total_time><people_pool>5100</people_pool>
			<tech_level>900000</tech_level><stats><population><population_mtd>510000</population_mtd></population>
			<sustainability><mining_flag>1</mining_flag><trade_flag>1</trade_flag>
			<mining_years>179</mining_years><trade_years>179</trade_years><money_years>59</money_years>
			<old_population>510000</old_population><population_years>59</population_years>
			<old_tech>900000</old_tech><tech_years>119</tech_years><fire_years>60</fire_years>
			</sustainability></stats>""", ""],
		["loss", "Stadt verloren", "<total_time>98</total_time><people_pool>0</people_pool>", ""],
	]:
		var fixture := FileAccess.create_temp(FileAccess.WRITE_READ, "lincity-outcome", "xml")
		fixture.store_string("""<lc-game ldsv-version="2140"><globals><total_money>1000000</total_money>
			%s</globals><map size="40">%s</map></lc-game>""" % [scenario[2], scenario[3]])
		fixture.flush()
		game._load_city(fixture.get_path_absolute())
		_check(game.state.side == 40, "Fixture konnte nicht geladen werden.")
		game.speed = 1
		if scenario[0] == "evacuation":
			game.selected = Vector2i(4, 4)
			game._inspect()
			game.hud.launch_button.pressed.emit()
			game.hud.launch_dialog.confirmed.emit()
			game.hud.launch_dialog.hide()
			for i in 20:
				game._process(.1)
		else:
			game._process(.25)
		var dialog: AcceptDialog = game.hud.get("outcome_dialog")
		_check(dialog != null and dialog.visible and dialog.title == scenario[1], "Spielausgang fehlt: " + scenario[0])
		if dialog:
			var day: int = game.state.time
			game._process(.5)
			_check(game.state.time == day, "Simulation läuft hinter dem Dialog.")
			dialog.hide()
			game._process(.25)
			_check(game.state.time > day and not dialog.visible, "Weiterbauen ist nicht möglich.")
	if failures == 0:
		print("Spielausgänge geprüft: Evakuierung, Nachhaltigkeit und Niederlage.")
	quit(1 if failures else 0)
