extends RefCounted

const CONFIG_PATH := "user://settings.cfg"
const MIN_WINDOW_SIZE := Vector2i(640, 480)
const MAX_WINDOW_SIZE := Vector2i(16384, 16384)
const MAP_SIDE_MIN := 50
const MAP_SIDE_MAX := 10000
const MAP_SIDE_STEP := 25
const CLIMATES := ["Flusslandschaft", "Wüste", "Gemäßigte Zone", "Sumpfland"]
const MSAA_MODES := ["Aus", "2× MSAA", "4× MSAA", "8× MSAA"]
const LANGUAGES := {"de": "Deutsch", "en": "English"}
const DEFAULTS := {
	"language": "de",
	"fullscreen": false,
	"window_size": Vector2i(1440, 900),
	"msaa": Viewport.MSAA_2X,
	"shadows": true,
	"reflections": true,
	"vehicles": true,
	"music_enabled": true,
	"music_volume": 0.5,
	"effects_enabled": true,
	"effects_volume": 1.0,
	"map_climate": 0,
	"map_side": 100,
	"map_village": true,
	"map_trees": true,
}
const SECTIONS := {
	"general": ["language"],
	"display": ["fullscreen", "window_size", "msaa", "shadows", "reflections", "vehicles"],
	"audio": ["music_enabled", "music_volume", "effects_enabled", "effects_volume"],
	"new_city": ["map_climate", "map_side", "map_village", "map_trees"],
}

var values := DEFAULTS.duplicate()
var _path: String

func _init(path := CONFIG_PATH) -> void:
	_path = path

func load_from_disk() -> Error:
	var config := ConfigFile.new()
	var result := config.load(_path)
	values = DEFAULTS.duplicate()
	if result == OK:
		for section: String in SECTIONS:
			for key: String in SECTIONS[section]:
				values[key] = _valid_value(key, config.get_value(section, key, DEFAULTS[key]))
	return result

func save_changes(changes: Dictionary) -> Error:
	var updated := values.duplicate()
	var config := ConfigFile.new()
	for section: String in SECTIONS:
		for key: String in SECTIONS[section]:
			updated[key] = _valid_value(key, changes.get(key, updated[key]))
			config.set_value(section, key, updated[key])
	var target := ProjectSettings.globalize_path(_path)
	var temporary := "%s.%s.tmp" % [target, OS.get_process_id()]
	var result := config.save(temporary)
	if result == OK:
		result = DirAccess.rename_absolute(temporary, target)
	if result == OK:
		values = updated
	elif FileAccess.file_exists(temporary):
		DirAccess.remove_absolute(temporary)
	return result

func apply_language() -> void:
	TranslationServer.set_locale(values.language)

func apply_display(window: Window, respect_launch_options := false) -> void:
	window.msaa_3d = values.msaa
	if DisplayServer.get_name() == "headless":
		return
	if respect_launch_options:
		for argument in OS.get_cmdline_args():
			if argument in ["-f", "--fullscreen", "-w", "--windowed", "-m", "--maximized", "--resolution", "--position"]:
				return
	if values.fullscreen:
		window.mode = Window.MODE_FULLSCREEN
		return
	if window.mode in [Window.MODE_FULLSCREEN, Window.MODE_EXCLUSIVE_FULLSCREEN]:
		window.mode = Window.MODE_WINDOWED
	if not can_set_window_size():
		return
	var usable := DisplayServer.screen_get_usable_rect(window.current_screen)
	var target: Vector2i = values.window_size
	if usable.has_area():
		target = target.min(usable.size)
	if window.size != target:
		window.mode = Window.MODE_WINDOWED
		window.size = target
		if usable.has_area():
			window.position = usable.position + (usable.size - target) / 2

static func can_set_window_size() -> bool:
	return DisplayServer.get_name() != "Wayland"

static func _valid_value(key: String, raw: Variant) -> Variant:
	var fallback: Variant = DEFAULTS[key]
	if typeof(raw) != typeof(fallback):
		return fallback
	match key:
		"language":
			if not LANGUAGES.has(raw): return fallback
		"window_size":
			if raw != raw.clamp(MIN_WINDOW_SIZE, MAX_WINDOW_SIZE): return fallback
		"msaa":
			if raw < 0 or raw >= MSAA_MODES.size(): return fallback
		"music_volume", "effects_volume":
			if not is_finite(raw) or raw < 0.0 or raw > 1.0: return fallback
		"map_climate":
			if raw < 0 or raw >= CLIMATES.size(): return fallback
		"map_side":
			if raw < MAP_SIDE_MIN or raw > MAP_SIDE_MAX or raw % MAP_SIDE_STEP != 0: return fallback
	return raw
