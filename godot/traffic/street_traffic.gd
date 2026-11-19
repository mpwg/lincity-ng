extends Node3D
## Parked cars follow developed streets. Rust supplies the lane positions of commuters.

const PAINTS := 3
const ROAD := 11
const ROAD_BRIDGE := 47
const DEVELOPED := [4, 6, 12, 18, 36, 37, 38, 39, 40]
var moving: MultiMesh
var parked: Array[MultiMesh] = []
var tiles := PackedInt32Array()
var side := 0

func _ready() -> void:
	for paint in PAINTS:
		var scene: PackedScene = load("res://art/traffic/car_%d.glb" % paint)
		var model := scene.instantiate()
		var body := model.find_child("Body", true, false) as MeshInstance3D
		if paint == 1:
			moving = _batch(body.mesh)
		parked.append(_batch(body.mesh))
		model.free()

func _batch(mesh: Mesh) -> MultiMesh:
	var batch := MultiMesh.new()
	batch.transform_format = MultiMesh.TRANSFORM_3D
	batch.mesh = mesh
	var instance := MultiMeshInstance3D.new()
	instance.multimesh = batch
	add_child(instance)
	return batch

func update_map(snapshot: Dictionary) -> void:
	var groups: PackedInt32Array = snapshot.tiles
	if groups == tiles and snapshot.side == side:
		return
	side = snapshot.side
	tiles = groups
	var lots := [[], [], []]
	for y in range(1, side - 1):
		for x in range(1, side - 1):
			var i := y * side + x
			if tiles[i] != ROAD:
				continue
			var seed_value := absi(hash(Vector2i(x, y)))
			if seed_value % 5 > 1:
				continue
			var along_x := tiles[i-1] == ROAD and tiles[i+1] == ROAD
			var along_z := tiles[i-side] == ROAD and tiles[i+side] == ROAD
			if along_x == along_z:
				continue
			var across := side if along_x else 1
			var curb := -1 if tiles[i-across] in DEVELOPED else 1
			if tiles[i + across * curb] not in DEVELOPED:
				continue
			var offset := Vector3(0, 0, curb * .29) if along_x else Vector3(curb * .29, 0, 0)
			var yaw := PI * .5 if along_x else 0.0
			if (along_x and curb < 0) or (along_z and curb > 0):
				yaw += PI
			lots[(seed_value / 5) % PAINTS].append(Transform3D(Basis(Vector3.UP,yaw), Vector3(x+.5,.032,y+.5) + offset))
	for paint in PAINTS:
		_set_transforms(parked[paint], lots[paint])

func update_commuters(poses: PackedVector3Array) -> void:
	var cars: Array[Transform3D] = []
	for i in poses.size():
		var p := poses[i]
		var x := floori(p.x)
		var z := floori(p.z)
		if x < 0 or z < 0 or x >= side or z >= side:
			continue
		var ground := .115 if tiles[z * side + x] in [46, ROAD_BRIDGE] else .032
		var heading := Basis(Vector3.UP, p.y * PI * .25)
		# Simulation order can change as commuters finish. All moving commuters
		# share one livery so a still-visible car keeps its colour.
		cars.append(Transform3D(heading, Vector3(p.x, ground, p.z)))
	_set_transforms(moving, cars)

func _set_transforms(batch: MultiMesh, transforms: Array) -> void:
	if transforms.size() > batch.instance_count:
		batch.instance_count = transforms.size()
	batch.visible_instance_count = transforms.size()
	for i in transforms.size():
		batch.set_instance_transform(i, transforms[i])
