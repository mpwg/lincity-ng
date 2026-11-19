extends RefCounted

const BUILDING_COUNTS := [
	["Wohnhäuser", [4, 36, 37, 38, 39, 40]], ["Märkte", [6]], ["Bauernhöfe", [5]],
	["Brunnen", [41]], ["Windmühlen", [24]], ["Denkmäler", [25]], ["Schulen", [26]],
	["Universitäten", [13]], ["Feuerwachen", [30]], ["Parks", [19, 50]],
	["Sportplätze", [31]], ["Gesundheitszentren", [22]], ["Mülldeponien", [16]],
	["Behelfsunterkünfte", [33]], ["Windkraftanlagen", [49]], ["Kohlekraftwerke", [10]],
	["Solarkraftwerke", [2]], ["Umspannwerke", [3]], ["Stromleitungen", [1]], ["Häfen", [17]],
	["Feldwege und Feldwegbrücken", [7, 46]], ["Straßen und Straßenbrücken", [11, 47]],
	["Gleise und Eisenbahnbrücken", [9, 48]], ["Töpfereien", [29]], ["Schmieden", [27]],
	["Textilmühlen", [28]], ["Leichtindustrie", [12]], ["Schwerindustrie", [18]],
	["Recyclinganlagen", [20]], ["Kohlebergwerke", [8]], ["Erzbergwerke", [15]],
	["Kommunen", [14]], ["Raketenstartplätze", [23]],
]

func statistics(city: Dictionary, date: String, number: Callable) -> PackedStringArray:
	var lines := PackedStringArray()
	var scenario := str(city.get("scenario", ""))
	if not scenario.is_empty():
		lines.append("%s   %s" % [tr("Ausgangsszenario"), scenario])
	lines.append("%s   %s" % [tr("Spielzeit"), date])
	lines.append("%s   %s $" % [tr("Stadtkasse"), number.call(int(city.get("money", 0)))])
	lines.append("%s   %s" % [tr("Nachhaltigkeit"), tr("erreicht") if city.get("sustainable", false) else tr("offen")])
	lines.append("%s   %.2f %%" % [tr("Technologie"), float(city.get("tech_percent", 0.0))])
	lines.append("%s   %.2f %%" % [tr("Höchste Technologie"), float(city.get("highest_tech_percent", 0.0))])
	lines.append("")
	for pair in [
		["population", "Bevölkerung"], ["population_record", "Höchste Einwohnerzahl"],
		["homeless", "Wohnungssuchende"], ["unemployment", "Arbeitslose"],
		["births", "Geburten"], ["starvation_deaths", "Tote durch Hunger"],
		["pollution_deaths", "Tote durch Umweltbelastung"], ["evacuated", "Evakuierte"],
		["rockets", "Gestartete Raketen"], ["successful_rockets", "Erfolgreiche Starts"],
	]:
		lines.append("%s   %s" % [tr(pair[1]), number.call(int(city.get(pair[0], 0)))])
	lines.append("%s   %.2f" % [tr("Arbeitslosigkeit in Personenjahren"), float(city.get("unemployment_years", 0.0))])
	lines.append("")
	lines.append(tr("Bauwerke und Verkehrswege"))
	var counts: PackedInt32Array = city.get("building_counts", PackedInt32Array())
	for pair in BUILDING_COUNTS:
		var count := 0
		for group: int in pair[1]:
			if group < counts.size():
				count += counts[group]
		lines.append("%s   %s" % [tr(pair[0]), number.call(count)])
	return lines

func supply(inventory: Array, goods: Array, number: Callable) -> PackedStringArray:
	var lines := PackedStringArray()
	lines.append(tr("Bestand / Kapazität"))
	lines.append("")
	for index in mini(inventory.size(), goods.size()):
		var amount := int(inventory[index].amount)
		var capacity := int(inventory[index].capacity)
		var stock := "—"
		if capacity > 0:
			stock = "%s / %s   (%.1f %%)" % [number.call(amount), number.call(capacity), amount * 100.0 / capacity]
		lines.append("%s   %s" % [tr(goods[index]), stock])
	return lines
