extends RefCounted
## Shared by the GLB baker, the renderer and catalog checks.
const SIZE := [1,1,4,2,3,4,2,1,4,1,4,1,3,3,4,4,4,4,4,1,2,1,2,4,2,2,2,2,2,2,2,2,1,2,1,1,3,3,3,3,3,2,1,1,1,1,1,1,1,2,1]
const SHORE := 51
const AVENUE := 52
const NETWORKS := [1,7,9,11,46,47,48]
const VARIED := [4,19,36,37,38,39,40,43,44,45]
const AUTHORED := [4,6,12,18,36,37,38,39,40]
const UNSELECTABLE := [0,19,21,35,42,43,44,45,SHORE,AVENUE]

static func variant_count(group: int) -> int:
	if group == 23: return 6
	if group in [SHORE,AVENUE]: return 2
	if group in NETWORKS: return 16
	return 3 if group in VARIED else 1

static func path(group: int, variant: int) -> String:
	if group == SHORE: return "res://art/city/shore_%d.glb" % variant
	if group == AVENUE: return "res://art/city/avenue_%d.glb" % variant
	var directory := "architecture" if group in AUTHORED else "city"
	return "res://art/%s/g%02d_v%02d.glb" % [directory, group, variant]

class Asset:
	var visual: Array[Mesh] = []
	var materials: Array[Material] = []
	var bounds := AABB()
	var pick: TriangleMesh
	var pick_bounds := AABB()
	var outlets := PackedVector3Array()
	var flight: Mesh

var _assets := {}
var _materials := {}

func asset(group: int, variant: int, surface: ShaderMaterial) -> Asset:
	var key := group * 16 + variant
	if _assets.has(key): return _assets[key]
	var file := path(group,variant)
	var scene := load(file) as PackedScene
	assert(scene != null, file)
	var root := scene.instantiate() as Node3D
	var result := Asset.new()
	for level in ["Near","Mid","Far"]:
		var part := root.find_child(level,true,false) as MeshInstance3D
		assert(part != null, "%s %s" % [file,level])
		var mesh := part.mesh
		var imported := mesh.surface_get_material(0) as BaseMaterial3D
		var texture: Texture2D = imported.albedo_texture if imported else null
		var material := surface
		if texture or group in [46,47,48]:
			var material_key: Variant = texture if texture else "bridge"
			if not _materials.has(material_key):
				var custom := surface.duplicate() as ShaderMaterial
				custom.set_shader_parameter("bridge",group in [46,47,48])
				if texture:
					custom.set_shader_parameter("use_atlas",true)
					custom.set_shader_parameter("atlas",texture)
				_materials[material_key] = custom
			material = _materials[material_key]
		result.materials.append(material)
		result.visual.append(mesh)
		result.bounds = mesh.get_aabb() if result.visual.size() == 1 else result.bounds.merge(mesh.get_aabb())
	var pick := root.find_child("Pick",true,false) as MeshInstance3D
	if pick:
		result.pick_bounds = pick.mesh.get_aabb().grow(.005)
		result.pick = TriangleMesh.new()
		result.pick.create_from_faces(pick.mesh.get_faces())
	for marker in root.find_children("Smoke*","Node3D",true,false):
		result.outlets.append(marker.position)
	var flight := root.get_node_or_null("Flight") as MeshInstance3D
	if flight: result.flight = flight.mesh
	root.free()
	_assets[key] = result
	return result

func set_map_parameter(parameter: StringName, value: Variant, surface: ShaderMaterial) -> void:
	surface.set_shader_parameter(parameter,value)
	for material: ShaderMaterial in _materials.values():
		material.set_shader_parameter(parameter,value)
