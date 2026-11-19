extends ConfirmationDialog

signal settings_applied

const GameSettings = preload("res://ui/settings.gd")
const RESOLUTIONS := [
	Vector2i(640, 480), Vector2i(800, 600), Vector2i(1024, 768),
	Vector2i(1280, 720), Vector2i(1280, 800), Vector2i(1366, 768),
	Vector2i(1440, 900), Vector2i(1600, 900), Vector2i(1920, 1080),
	Vector2i(1920, 1200), Vector2i(2560, 1440), Vector2i(3840, 2160),
]

var _settings: GameSettings
var _audio
var _fullscreen := CheckButton.new()
var _resolution := OptionButton.new()
var _msaa := OptionButton.new()
var _shadows := CheckButton.new()
var _reflections := CheckButton.new()
var _music := CheckButton.new()
var _music_volume := HSlider.new()
var _effects := CheckButton.new()
var _effects_volume := HSlider.new()
var _track := Label.new()
var _climate := OptionButton.new()
var _side := SpinBox.new()
var _village := CheckButton.new()
var _trees := CheckButton.new()
var _save_error := Label.new()

func _ready() -> void:
	title = "Einstellungen"
	ok_button_text = "Übernehmen"
	cancel_button_text = "Abbrechen"
	dialog_hide_on_ok = false
	transient = true
	exclusive = true
	var content := VBoxContainer.new()
	add_child(content)
	var tabs := TabContainer.new()
	tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	content.add_child(tabs)
	var display := _tab(tabs, "Grafik")
	_row(display, "Vollbild", _fullscreen, "Fullscreen")
	_row(display, "Fenstergröße", _resolution, "WindowSize")
	if not GameSettings.can_set_window_size():
		_resolution.tooltip_text = "Unter Wayland änderst du die Größe am Fensterrand."
		_resolution.accessibility_description = _resolution.tooltip_text
	_row(display, "Kantenglättung", _msaa, "Antialiasing")
	for item: String in GameSettings.MSAA_MODES:
		_msaa.add_item(item)
	_row(display, "Schatten", _shadows, "Shadows")
	_row(display, "Wasserspiegelungen", _reflections, "Reflections")
	_fullscreen.toggled.connect(_update_resolution_control)

	var audio := _tab(tabs, "Ton")
	_row(audio, "Hintergrundmusik", _music, "MusicEnabled")
	_volume_row(audio, "Musiklautstärke", _music_volume, "MusicVolume")
	var playlist := HBoxContainer.new()
	var previous := _track_button("‹", "Vorheriger Titel", _skip_track.bind(-1))
	var next := _track_button("›", "Nächster Titel", _skip_track.bind(1))
	playlist.add_child(previous)
	_track.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_track.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_track.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	playlist.add_child(_track)
	playlist.add_child(next)
	_row(audio, "Musiktitel", playlist, "Playlist")
	_row(audio, "Geräusche", _effects, "EffectsEnabled")
	_volume_row(audio, "Geräuschlautstärke", _effects_volume, "EffectsVolume")
	about_to_popup.connect(func():
		var audio_available: bool = is_instance_valid(_audio) and not _audio.current_track_title().is_empty()
		previous.disabled = not audio_available
		next.disabled = not audio_available)

	var map := _tab(tabs, "Neue Stadt")
	_row(map, "Landschaft", _climate, "MapClimate")
	for item: String in GameSettings.CLIMATES:
		_climate.add_item(item)
	_row(map, "Kantenlänge in Feldern", _side, "MapSide")
	_side.min_value = GameSettings.MAP_SIDE_MIN
	_side.max_value = GameSettings.MAP_SIDE_MAX
	_side.step = GameSettings.MAP_SIDE_STEP
	_row(map, "Mit Startdorf", _village, "MapVillage")
	_row(map, "Mit Bäumen", _trees, "MapTrees")
	_save_error.add_theme_color_override("font_color", Color("#ffab95"))
	_save_error.hide()
	content.add_child(_save_error)
	about_to_popup.connect(_populate)
	confirmed.connect(_save)

func setup(settings: GameSettings, audio: Node) -> void:
	if is_instance_valid(_audio):
		_audio.track_changed.disconnect(_show_track)
	_settings = settings
	_audio = audio
	if is_instance_valid(_audio):
		_audio.track_changed.connect(_show_track)

func _tab(tabs: TabContainer, caption: String) -> GridContainer:
	var container := MarginContainer.new()
	container.name = caption
	for side: String in ["left", "right", "top", "bottom"]:
		container.add_theme_constant_override("margin_" + side, 16)
	tabs.add_child(container)
	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 24)
	grid.add_theme_constant_override("v_separation", 12)
	container.add_child(grid)
	return grid

func _row(grid: GridContainer, caption: String, widget: Control, id: String) -> void:
	var label := Label.new()
	label.text = caption
	label.custom_minimum_size.x = 150
	grid.add_child(label)
	grid.add_child(widget)
	widget.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	widget.accessibility_name = caption
	widget.name = id
	widget.owner = self
	widget.unique_name_in_owner = true

func _volume_row(grid: GridContainer, caption: String, slider: HSlider, id: String) -> void:
	var meter := HBoxContainer.new()
	slider.max_value = 100
	slider.step = 1
	slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	slider.accessibility_name = caption
	meter.add_child(slider)
	var percent := Label.new()
	percent.custom_minimum_size.x = 42
	percent.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	meter.add_child(percent)
	slider.value_changed.connect(func(value: float): percent.text = "%d %%" % int(value))
	percent.text = "0 %"
	_row(grid, caption, meter, id + "Row")
	slider.name = id
	slider.owner = self
	slider.unique_name_in_owner = true

func _track_button(glyph: String, caption: String, action: Callable) -> Button:
	var button := Button.new()
	button.text = glyph
	button.tooltip_text = caption
	button.accessibility_name = caption
	button.custom_minimum_size.x = 38
	button.pressed.connect(action)
	return button

func _skip_track(direction: int) -> void:
	if not is_instance_valid(_audio):
		return
	if direction < 0:
		_audio.previous_track()
	else:
		_audio.next_track()

func _show_track(track: String) -> void:
	_track.text = "Kein Musiktitel verfügbar" if track.is_empty() else track
	_track.tooltip_text = track

func _populate() -> void:
	if _settings == null:
		return
	var options := _settings.values
	_fullscreen.set_pressed_no_signal(options.fullscreen)
	_update_resolution_control(options.fullscreen)
	_populate_resolutions(options.window_size)
	_msaa.select(options.msaa)
	_shadows.button_pressed = options.shadows
	_reflections.button_pressed = options.reflections
	_music.button_pressed = options.music_enabled
	_music_volume.value = options.music_volume * 100.0
	_effects.button_pressed = options.effects_enabled
	_effects_volume.value = options.effects_volume * 100.0
	_show_track(_audio.current_track_title() if is_instance_valid(_audio) else "")
	_climate.select(options.map_climate)
	_side.value = options.map_side
	_village.button_pressed = options.map_village
	_trees.button_pressed = options.map_trees
	_save_error.hide()

func _update_resolution_control(fullscreen: bool) -> void:
	_resolution.disabled = fullscreen or not GameSettings.can_set_window_size()

func _populate_resolutions(saved_size: Vector2i) -> void:
	_resolution.clear()
	var sizes: Array[Vector2i] = []
	var window := get_parent().get_window()
	var available := DisplayServer.screen_get_usable_rect(window.current_screen).size
	for preset: Vector2i in RESOLUTIONS:
		if available == Vector2i.ZERO or preset == preset.min(available):
			sizes.append(preset)
	if window.mode == Window.MODE_WINDOWED and window.size == window.size.clamp(GameSettings.MIN_WINDOW_SIZE, GameSettings.MAX_WINDOW_SIZE):
		if window.size not in sizes:
			sizes.append(window.size)
	if saved_size not in sizes:
		sizes.append(saved_size)
	sizes.sort()
	for size_option: Vector2i in sizes:
		_resolution.add_item("%d × %d" % [size_option.x, size_option.y])
		var i := _resolution.item_count - 1
		_resolution.set_item_metadata(i, size_option)
		if size_option == saved_size:
			_resolution.select(i)

func _save() -> void:
	if _settings == null:
		return
	var result := _settings.save_changes({
		"fullscreen": _fullscreen.button_pressed,
		"window_size": _resolution.get_selected_metadata(),
		"msaa": _msaa.selected,
		"shadows": _shadows.button_pressed,
		"reflections": _reflections.button_pressed,
		"music_enabled": _music.button_pressed,
		"music_volume": _music_volume.value / 100.0,
		"effects_enabled": _effects.button_pressed,
		"effects_volume": _effects_volume.value / 100.0,
		"map_climate": _climate.selected,
		"map_side": int(_side.value),
		"map_village": _village.button_pressed,
		"map_trees": _trees.button_pressed,
	})
	if result != OK:
		_save_error.text = "Die Einstellungen konnten nicht gespeichert werden."
		_save_error.show()
		return
	hide()
	settings_applied.emit()
