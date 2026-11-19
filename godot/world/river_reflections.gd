extends SubViewport
## The mirror has its own world. Only above-ground city geometry enters its color buffer.
const SILHOUETTE := preload("river_mirror.gdshader")
const RESOLUTION := 0.36
const DISTANCE := 58.0

var eye := Camera3D.new()
var districts := {}
var materials := {}
var enabled := false
var dirty := true

func _init() -> void:
	world_3d = World3D.new()
	transparent_bg = true
	render_target_update_mode = SubViewport.UPDATE_DISABLED
	positional_shadow_atlas_size = 0
	gui_disable_input = true
	audio_listener_enable_3d = false
	add_child(eye)
	eye.current = true

func clear_district(id: Vector2i) -> void:
	if districts.has(id):
		districts[id].free()
		districts.erase(id)
		dirty = true

func add_batch(id: Vector2i, asset, transforms: Array[Transform3D], foliage: bool) -> void:
	if not districts.has(id):
		var district := Node3D.new()
		district.scale = Vector3(1,-1,1)
		add_child(district)
		districts[id] = district
	var geometry := MultiMesh.new()
	geometry.transform_format = MultiMesh.TRANSFORM_3D
	var level := 1 if foliage else 2
	geometry.mesh = asset.visual[level]
	geometry.instance_count = transforms.size()
	for i in transforms.size():
		geometry.set_instance_transform(i,transforms[i])
	var batch := MultiMeshInstance3D.new()
	batch.multimesh = geometry
	batch.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	var atlas: Texture2D = asset.materials[level].get_shader_parameter("atlas")
	if not materials.has(atlas):
		var material := ShaderMaterial.new()
		material.shader = SILHOUETTE
		material.set_shader_parameter("leaf_spray",asset.materials[level].get_shader_parameter("leaf_spray"))
		if atlas:
			material.set_shader_parameter("has_atlas",true)
			material.set_shader_parameter("atlas",atlas)
		materials[atlas] = material
	batch.material_override = materials[atlas]
	districts[id].add_child(batch)
	dirty = true

func sync(camera: Camera3D, show_reflections: bool) -> bool:
	var was_enabled := enabled
	enabled = show_reflections and camera.global_position.y < DISTANCE
	if enabled:
		var target := Vector2i(camera.get_viewport().get_visible_rect().size*RESOLUTION).max(Vector2i(32,32))
		if dirty or not was_enabled or target != size or eye.global_transform != camera.global_transform or eye.fov != camera.fov:
			size = target
			eye.global_transform = camera.global_transform
			eye.fov = camera.fov
			eye.keep_aspect = camera.keep_aspect
			eye.near = camera.near
			eye.far = DISTANCE + 15.0
			render_target_update_mode = SubViewport.UPDATE_ONCE
			dirty = false
	elif was_enabled:
		render_target_update_mode = SubViewport.UPDATE_DISABLED
	return was_enabled != enabled
