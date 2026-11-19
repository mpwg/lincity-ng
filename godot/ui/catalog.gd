extends RefCounted

# Schlüssel entsprechen den Baugruppen des Simulationskerns.
const CATEGORIES = {
	"Werkzeuge": [-1, -2, -3, -4],
	"Wohnen": [4, 36, 37, 38, 39, 40, 33],
	"Versorgung": [41, 5, 6, 14, 19, 50, 25],
	"Verkehr": [7, 11, 9, 17, 23, 46, 47, 48],
	"Energie": [1, 3, 24, 49, 10, 2],
	"Wirtschaft": [27, 29, 28, 15, 8, 12, 18, 20, 16],
	"Öffentliches": [26, 13, 22, 30, 31],
}
const TOOLS = {
	-1: ["Untersuchen", "pointer"], -2: ["Abreißen", "bulldozer"],
	-3: ["Evakuieren", "broom"], -4: ["Wasser anlegen", "water"],
	1: ["Stromleitung", "powerlines"], 2: ["Solarkraftwerk", "solarpower"],
	3: ["Umspannwerk", "substation"], 4: ["Einfach · locker", "residence-lotec-lo"],
	5: ["Bauernhof", "farm"], 6: ["Markt", "market"], 7: ["Feldweg", "tracks"],
	8: ["Kohlebergwerk", "coalmine"], 9: ["Eisenbahn", "rails2"],
	10: ["Kohlekraftwerk", "coalpower"], 11: ["Straße", "road"],
	12: ["Leichtindustrie", "lightindustry"], 13: ["Universität", "university"],
	14: ["Kommune", "commune"], 15: ["Erzbergwerk", "oremine"],
	16: ["Mülldeponie", "trash"], 17: ["Hafen", "port"],
	18: ["Schwerindustrie", "heavyindustry2"], 19: ["Park", "tree"],
	20: ["Recyclinganlage", "recycling"], 21: ["Wasser", "water"],
	22: ["Gesundheitszentrum", "health"], 23: ["Raketenstartplatz", "rocket"],
	24: ["Windmühle", "windmills"], 25: ["Denkmal", "monument"],
	26: ["Schule", "book"], 27: ["Schmiede", "blacksmith"],
	28: ["Textilmühle", "mills"], 29: ["Töpferei", "pottery"],
	30: ["Feuerwache", "fire"], 31: ["Sportplatz", "sports"],
	33: ["Behelfsunterkunft", "residence-lotec-lo"],
	36: ["Einfach · mittel", "residence-lotec-med"], 37: ["Einfach · dicht", "residence-lotec-high"],
	38: ["Modern · locker", "residence-hightec-lo"], 39: ["Modern · mittel", "residence-hightec-med"],
	40: ["Modern · dicht", "residence-hightec-high"], 41: ["Brunnen", "waterwell"],
	46: ["Feldwegbrücke", "tracks"], 47: ["Straßenbrücke", "road"], 48: ["Eisenbahnbrücke", "rails2"],
	49: ["Windkraftanlage", "windmills"], 50: ["Park mit Teich", "water"],
}

static func tool_name(id: int, fallback: String = "") -> String:
	return TranslationServer.translate(TOOLS[id][0] if TOOLS.has(id) else fallback)
