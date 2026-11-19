extends Control

signal action_requested(action: StringName)

const MAIN_ACTIONS := {
	&"new_game": "Neues Spiel",
	&"load_game": "Spiel laden",
	&"continue_game": "Fortsetzen",
	&"settings": "Einstellungen",
	&"help": "Spielhilfe",
	&"credits": "Mitwirkende und Lizenzen",
	&"quit": "Beenden",
}
const PAUSE_ACTIONS := {
	&"resume": "Fortfahren",
	&"save_game": "Spiel speichern",
	&"main_menu": "Zum Hauptmenü",
}

var buttons := {}
var in_game := false
var _backdrop := ColorRect.new()
var _title := Label.new()
var _message := Label.new()

func _ready() -> void:
	set_anchors_and_offsets_preset(PRESET_FULL_RECT)
	add_child(_backdrop)
	_backdrop.set_anchors_and_offsets_preset(PRESET_FULL_RECT)
	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(PRESET_FULL_RECT)
	add_child(center)
	var card := PanelContainer.new()
	card.custom_minimum_size.x = 320
	var style := StyleBoxFlat.new()
	style.bg_color = Color("#232a25")
	style.border_color = Color("#868c75")
	style.set_border_width_all(1)
	style.set_corner_radius_all(8)
	style.content_margin_left = 24
	style.content_margin_right = 24
	style.content_margin_top = 18
	style.content_margin_bottom = 18
	card.add_theme_stylebox_override("panel", style)
	center.add_child(card)
	var rows := VBoxContainer.new()
	rows.add_theme_constant_override("separation", 5)
	card.add_child(rows)
	_title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_title.add_theme_font_size_override("font_size", 28)
	rows.add_child(_title)
	for actions: Dictionary in [PAUSE_ACTIONS, MAIN_ACTIONS]:
		for action: StringName in actions:
			var button := Button.new()
			button.name = action
			button.text = actions[action]
			button.custom_minimum_size.y = 30
			button.pressed.connect(action_requested.emit.bind(action))
			buttons[action] = button
			rows.add_child(button)
	_message.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_message.custom_minimum_size.x = 270
	_message.add_theme_color_override("font_color", Color("#ffb5a1"))
	rows.add_child(_message)
	hide()

func open(pause_game: bool, has_save: bool) -> void:
	in_game = pause_game
	_title.text = tr("Spiel pausiert") if in_game else "LinCity-NG"
	_backdrop.color = Color("#18211bad") if in_game else Color("#171e19")
	for action: StringName in PAUSE_ACTIONS:
		buttons[action].visible = in_game
	for action: StringName in [&"new_game", &"load_game", &"continue_game"]:
		buttons[action].visible = not in_game
	buttons.continue_game.disabled = not has_save
	show_message("")
	show()
	if in_game:
		buttons.resume.grab_focus()
	elif has_save:
		buttons.continue_game.grab_focus()
	else:
		buttons.new_game.grab_focus()

func show_message(message: String) -> void:
	_message.text = message
	_message.visible = not message.is_empty()
