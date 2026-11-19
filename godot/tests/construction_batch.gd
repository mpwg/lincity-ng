extends SceneTree

var failures := 0

func _initialize() -> void:
	call_deferred("_run")

func _check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		printerr(message)

func _run() -> void:
	var world: Object = ClassDB.instantiate("CityWorld")
	if not world.has_method("preview_batch") or not world.has_method("apply_batch"):
		printerr("Die gemeinsame Bauvorschau und Ausführung fehlen.")
		quit(1)
		return
	var path := "user://construction-batch.xml"
	var file := FileAccess.open(path, FileAccess.WRITE)
	file.store_string("""<lc-game ldsv-version="2140"><globals><total_money>-1999999998</total_money></globals>
		<map size="16"><Construction group="6" map-x="4" map-y="4"/></map></lc-game>""")
	file.close()
	var error: String = world.load_city(path)
	DirAccess.remove_absolute(path)
	if not error.is_empty():
		printerr(error)
		quit(1)
		return
	var tools: Array = world.tools()
	var track: Dictionary = tools.filter(func(item: Dictionary) -> bool: return item.id == 7)[0]
	var road: Dictionary = tools.filter(func(item: Dictionary) -> bool: return item.id == 11)[0]
	_check(track.base_cost == 1 and road.required_tech == 50000, "Baukatalog zeigt falsche Grunddaten.")
	var targets: Array[Vector2i] = [Vector2i(5, 5), Vector2i(8, 8), Vector2i(9, 8), Vector2i(10, 8)]
	var before: Dictionary = world.snapshot()
	var preview: Array = world.preview_batch(7, targets)
	_check(not preview[0].allowed and not preview[0].reason.is_empty(), "Belegtes Feld wurde freigegeben.")
	_check(preview[1].allowed and preview[1].cost == 1 and preview[2].allowed, "Bezahlbare Bauaktion wurde blockiert.")
	_check(not preview[3].allowed and preview[3].warning == "out_of_credit", "Bauvorschau überschreitet Kreditrahmen.")
	_check(world.snapshot().tiles == before.tiles and world.snapshot().money == before.money, "Vorschau hat die Stadt geändert.")
	_check(world.apply_batch(7, targets) == preview, "Ausführung weicht von der Bauvorschau ab.")
	var after: Dictionary = world.snapshot()
	_check(after.money == -2000000000 and after.tiles[8 * after.side + 9] == 7, "Bauaktion wurde falsch abgerechnet oder projiziert.")
	if failures == 0:
		print("OK: Bauvorschau, belegte Felder, Kreditrahmen und Ausführung stimmen überein.")
	quit(1 if failures else 0)
