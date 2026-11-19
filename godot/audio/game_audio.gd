extends Node

signal track_changed(title: String)

const MUSIC_DIR := "res://audio/music/"
const EFFECT_DIR := "res://audio/sounds/"

var _music := AudioStreamPlayer.new()
var _effects := AudioStreamPlayer.new()
var _playlist: Array[Dictionary] = []
var _samples := {}
var _tile_samples := {}
var _track_index := -1
var _technology := 0.0
var _music_enabled := false
var _effects_enabled := false
var _muted_for_session := false

func _ready() -> void:
	_music.name = "Music"
	_effects.name = "Effects"
	add_child(_music)
	add_child(_effects)
	_music.finished.connect(_advance_music)
	_muted_for_session = "--mute" in OS.get_cmdline_user_args()
	_load_music_theme()
	_load_sound_groups()
	_select_track(_first_available_track())

func apply_settings(options: Dictionary) -> void:
	_music.volume_linear = clampf(float(options.get("music_volume", 0.5)), 0.0, 1.0)
	_effects.volume_linear = clampf(float(options.get("effects_volume", 1.0)), 0.0, 1.0)
	_music_enabled = bool(options.get("music_enabled", true)) and not _muted_for_session
	_effects_enabled = bool(options.get("effects_enabled", true)) and not _muted_for_session
	if not _effects_enabled:
		_effects.stop()
	if not _music_enabled:
		_music.stop()
	elif _track_index >= 0 and not _music.playing:
		_play_track()

func set_technology_percent(value: float) -> void:
	_technology = snappedf(value, 0.1)
	if _track_index == -1 or not _is_available(_track_index):
		_select_track(_first_available_track())

func current_track_title() -> String:
	return _playlist[_track_index].title if _track_index >= 0 else ""

func next_track() -> void:
	for index in range(_track_index + 1, _playlist.size()):
		if _is_available(index):
			_select_track(index)
			return

func previous_track() -> void:
	for index in range(_track_index - 1, -1, -1):
		if _is_available(index):
			_select_track(index)
			return
	if _track_index >= 0:
		_select_track(_track_index)

func play_effect(id: StringName) -> void:
	_play_sample(_samples.get(String(id), []))

func play_tile_effect(sprite: String) -> void:
	_play_sample(_tile_samples.get(sprite, []))

func _advance_music() -> void:
	for offset in range(1, _playlist.size() + 1):
		var index := posmod(_track_index + offset, _playlist.size())
		if _is_available(index):
			_select_track(index)
			return

func _play_track() -> void:
	_music.stream = load(_playlist[_track_index].path) as AudioStream
	if _music.stream != null:
		_music.play()

func _select_track(index: int) -> void:
	var changed := index != _track_index
	_track_index = index
	if _track_index == -1:
		_music.stop()
	elif _music_enabled:
		_play_track()
	if changed:
		track_changed.emit(current_track_title())

func _first_available_track() -> int:
	for index in _playlist.size():
		if _is_available(index):
			return index
	return -1

func _is_available(index: int) -> bool:
	var track := _playlist[index]
	return _technology >= track.minimum and (track.maximum == 0.0 or _technology <= track.maximum)

func _play_sample(paths: Array) -> void:
	if not _effects_enabled or paths.is_empty():
		return
	_effects.stream = load(paths.pick_random()) as AudioStream
	if _effects.stream != null:
		_effects.play()

func _load_music_theme() -> void:
	var parser := XMLParser.new()
	if parser.open(MUSIC_DIR + "default.xml") != OK:
		push_error("Could not open the LinCity-NG music playlist.")
		return
	while parser.read() == OK:
		if parser.get_node_type() == XMLParser.NODE_ELEMENT and parser.get_node_name() == "song":
			_playlist.append({
				"title": parser.get_named_attribute_value("title"),
				"path": MUSIC_DIR + parser.get_named_attribute_value("filename"),
				"minimum": float(parser.get_named_attribute_value_safe("lowest-tech-level")),
				"maximum": float(parser.get_named_attribute_value_safe("highest-tech-level")),
			})

func _load_sound_groups() -> void:
	var parser := XMLParser.new()
	if parser.open(EFFECT_DIR + "sounds.xml") != OK:
		push_error("Could not open the LinCity-NG sound groups.")
		return
	var resources: Array[String] = []
	while parser.read() == OK:
		if parser.get_node_type() == XMLParser.NODE_ELEMENT_END and parser.get_node_name() == "group":
			resources.clear()
		if parser.get_node_type() != XMLParser.NODE_ELEMENT:
			continue
		match parser.get_node_name():
			"resourceID":
				resources.append(parser.get_named_attribute_value("name"))
			"sound":
				var file := parser.get_named_attribute_value("file")
				var path := EFFECT_DIR + file
				if resources.is_empty():
					var id := file.get_basename().rstrip("0123456789")
					_append_sample(_samples, id, path)
				else:
					for resource_id in resources:
						_append_sample(_tile_samples, resource_id, path)
	_samples["WindowOpen"] = [EFFECT_DIR + "WindowOpen.wav"]
	_samples["WindowClose"] = [EFFECT_DIR + "WindowClose.wav"]

func _append_sample(groups: Dictionary, id: String, path: String) -> void:
	if not groups.has(id):
		groups[id] = []
	groups[id].append(path)
