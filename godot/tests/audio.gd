extends SceneTree

const AUDIO_SCRIPT := "res://audio/game_audio.gd"

var errors := 0

func _initialize() -> void:
	call_deferred("_test")

func _test() -> void:
	var script := load(AUDIO_SCRIPT) as GDScript
	if script == null:
		printerr("Audiosteuerung fehlt.")
		quit(1)
		return
	var audio = script.new()
	root.add_child(audio)
	var music: AudioStreamPlayer = audio.get_node("Music")
	var effects: AudioStreamPlayer = audio.get_node("Effects")
	var titles: Array[String] = []
	audio.track_changed.connect(func(title: String): titles.append(title))
	if "--mute" in OS.get_cmdline_user_args():
		audio.apply_settings({"music_enabled": true, "effects_enabled": true})
		audio.play_effect(&"Click")
		_check(not music.playing and not effects.playing, "Der Start mit --mute spielt Audio.")
		audio.free()
		if errors == 0:
			print("OK: --mute schaltet Musik und Effekte für den Spielstart stumm.")
		quit(1 if errors else 0)
		return
	for track: Dictionary in audio._playlist:
		_check(ResourceLoader.exists(track.path, "AudioStream"), "Fehlender Titel: " + track.path)
	for groups: Dictionary in [audio._samples, audio._tile_samples]:
		for paths: Array in groups.values():
			for path: String in paths:
				_check(ResourceLoader.exists(path, "AudioStream"), "Fehlender Effekt: " + path)
	audio.apply_settings({"music_enabled": false, "effects_enabled": false})
	audio.play_effect(&"Click")
	_check(not music.playing and not effects.playing, "Stumm geschalteter Start spielt Audio.")
	_check(audio.current_track_title() == "Robert van Herk - City Blues", "Der klassische Starttitel fehlt.")

	audio.apply_settings({"music_enabled": true, "music_volume": 0.5, "effects_enabled": true, "effects_volume": 0.25})
	_check(music.playing and music.stream is AudioStreamOggVorbis, "Hintergrundmusik startet nicht.")
	_check(is_equal_approx(music.volume_linear, 0.5), "Musiklautstärke falsch.")
	audio.next_track()
	_check(titles.is_empty(), "Ein Titel wurde vor seiner Technologiestufe gewählt.")
	audio.set_technology_percent(2.0)
	audio.next_track()
	_check(audio.current_track_title() == "Robert van Herk - Architectural Contemplations", "Der zweite Titel fehlt.")
	audio.next_track()
	_check(audio.current_track_title() == "Pronobozo - Lincity", "Der dritte Titel fehlt.")
	audio.previous_track()
	_check(audio.current_track_title() == "Robert van Herk - Architectural Contemplations", "Der vorige Titel fehlt.")
	audio.next_track()
	music.finished.emit()
	_check(audio.current_track_title() == "Robert van Herk - City Blues", "Die Wiedergabeliste wird nicht wiederholt.")
	_check(titles.size() == 5, "Die Titelanzeige wurde nicht bei jedem Titelwechsel aktualisiert.")
	audio.next_track()
	audio.set_technology_percent(0.0)
	_check(audio.current_track_title() == "Robert van Herk - City Blues", "Ein Spielstand mit weniger Technologie behielt einen gesperrten Titel.")

	audio.play_effect(&"Build")
	_check(effects.playing and effects.stream.resource_path.get_file().begins_with("Build"), "Baugeräusch fehlt.")
	_check(is_equal_approx(effects.volume_linear, 0.25), "Effektlautstärke falsch.")
	audio.play_effect(&"Raze")
	_check(effects.stream.resource_path.get_file().begins_with("Raze"), "Abrissgeräusch fehlt.")
	audio.play_tile_effect("MarketMed")
	_check(effects.stream.resource_path.get_file() == "MarketMed1.wav", "Marktvariante fehlt.")
	audio.play_tile_effect("RailBridge")
	_check(effects.stream.resource_path.get_file().begins_with("RailTrain"), "Eisenbahnbrücke verwendet kein Eisenbahngeräusch.")
	audio.play_tile_effect("Desert")
	_check(effects.stream.resource_path.get_file().begins_with("RailTrain"), "Stummes Wüstenfeld spielt ein Geräusch.")

	audio.apply_settings({"music_enabled": false, "effects_enabled": true})
	_check(not music.playing and effects.playing, "Musikschalter beeinflusst Effekte.")
	audio.play_effect(&"RocketTakeoff")
	_check(effects.stream.resource_path.get_file() == "RocketTakeoff1.wav", "Raketenstartgeräusch fehlt.")
	audio.apply_settings({"music_enabled": true, "effects_enabled": false})
	_check(music.playing and not effects.playing, "Effektschalter beeinflusst Musik.")
	audio.play_effect(&"Click")
	_check(not effects.playing, "Effekt startet im stumm geschalteten Zustand.")
	audio.apply_settings({"music_enabled": false, "effects_enabled": false})
	audio.queue_free()
	# Let the audio mixer release its stopped playbacks before the process exits.
	await create_timer(0.15).timeout
	if errors == 0:
		print("OK: Klassische Wiedergabeliste, Technologiestufen, Titelanzeige, Geräusche und getrennte Lautstärken.")
	quit(1 if errors else 0)

func _check(ok: bool, message: String) -> void:
	if not ok:
		printerr(message)
		errors += 1
