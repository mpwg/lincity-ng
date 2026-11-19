extends SceneTree

# Rebuild the playable opening from the distributed scenario using game commands.
const SOURCE := "res://opening/good_times.scn.gz"
const OUTPUT := "res://scenarios/am_fluss.scn.gz"
const ROAD := 11
const DEMOLISH := -2
var city: Object
var failures := 0

func _initialize() -> void:
	city = ClassDB.instantiate("CityWorld")
	if not city.load_city(SOURCE).is_empty():
		quit(1)
		return
	# Broaden the natural bend into a river with two recognizable banks.
	for y in range(19, 44):
		var bank := roundi(88.0 - (y - 19) * .41)
		var width := mini(4, mini(y - 18, 44 - y))
		for x in range(bank, bank + width + 1):
			var cell := Vector2i(x,y)
			var state: Dictionary = city.snapshot()
			var index: int = y * state.side + x
			if state.tiles[index] == 21:
				continue
			if state.tiles[index] not in [0,35,42,43,44,45]:
				city.apply(DEMOLISH, cell)
			if city.snapshot().tiles[index] != 21:
				var error: String = city.apply(-4, cell)
				if not error.is_empty():
					printerr("Flussbett %s: %s" % [cell, error])
					failures += 1
	# The transverse street crosses the existing course of the river.
	for x in range(65, 90):
		place(ROAD, Vector2i(x, 30), 1)
	for y in range(17, 42):
		place(ROAD, Vector2i(72, y), 1)
	for y in range(31, 41):
		place(ROAD, Vector2i(76, y), 1)
	for x in range(65, 77):
		place(ROAD, Vector2i(x, 40), 1)
	for x in range(66, 84):
		place(ROAD, Vector2i(x, 23), 1)

	place(6, Vector2i(76, 28), 2)  # Market front on the cross street.
	place(12, Vector2i(80, 27), 3) # Boiler house at the river bend.
	for y in [31, 34, 37]:
		place(36, Vector2i(73, y), 3)
		if y < 37:
			place(37, Vector2i(77, y), 3)
	for y in [17, 20, 24, 27, 31, 34, 37]:
		place(4, Vector2i(69, y), 3)
	for x in [66, 73, 76, 79]:
		place(36, Vector2i(x, 20), 3)
		place(37, Vector2i(x, 24), 3)
	place(4, Vector2i(73, 27), 3)
	# Continue the existing street into a mixed neighbourhood behind the market.
	for x in range(59, 84):
		place(ROAD, Vector2i(x, 16), 1)
	for y in range(16, 31):
		place(ROAD, Vector2i(65, y), 1)
	for x in [66, 69, 73, 76, 79]:
		place(4 if x % 2 else 39, Vector2i(x, 13), 3)
	place(36, Vector2i(66, 17), 3)
	place(39, Vector2i(62, 20), 3)
	place(26, Vector2i(73, 17), 2)
	place(22, Vector2i(77, 17), 2)
	place(13, Vector2i(80, 17), 3)
	for x in [68, 72, 76, 80]:
		place(5, Vector2i(x, 8), 4)
	# Connect the bridge and the suppliers to the established districts.
	for y in range(30, 34):
		place(ROAD, Vector2i(89, y), 1)
	for y in range(31, 36):
		place(ROAD, Vector2i(68, y), 1)
	for y in range(12, 16):
		place(ROAD, Vector2i(72, y), 1)
	place(15, Vector2i(82, 19), 4)
	place(41, Vector2i(75, 18), 2)
	for x in range(78, 80):
		for y in range(28, 30):
			var index: int = y * 100 + x
			if city.snapshot().tiles[index] in [43,44,45]:
				city.apply(DEMOLISH, Vector2i(x,y))
	if failures == 0:
		# Let deliveries reach the newly built mill, market and homes.
		for day in 24:
			var error: String = city.step()
			if not error.is_empty():
				printerr(error)
				quit(1)
				return
		city.set_focus(Vector2i(76, 31))
		var error: String = city.save_city(OUTPUT)
		if not error.is_empty():
			printerr(error)
			failures += 1
	quit(1 if failures else 0)

func place(group: int, cell: Vector2i, size: int) -> void:
	var snapshot: Dictionary = city.snapshot()
	var at: int = cell.y * snapshot.side + cell.x
	if snapshot.tiles[at] == group and snapshot.anchors[at] == at:
		return
	for y in range(cell.y, cell.y + size):
		for x in range(cell.x, cell.x + size):
			var i: int = y * snapshot.side + x
			# Leave the river bed in place for the core's bridge resolver.
			if snapshot.tiles[i] in [0, 21, 35, 42, 43, 44, 45]:
				continue
			var pos := Vector2i(x, y)
			var current: Dictionary = city.snapshot()
			if current.tiles[i] == 47 and group == ROAD:
				return
			if current.tiles[i] == snapshot.tiles[i]:
				city.apply(DEMOLISH, pos)
	var error: String = city.apply(group, cell)
	if not error.is_empty():
		printerr("Gruppe %d auf %s: %s" % [group, cell, error])
		failures += 1
