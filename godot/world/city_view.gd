extends Node3D

const Models = preload("model_catalog.gd")
const RiverReflections = preload("river_reflections.gd")
const SURFACE = preload("city_surface.gdshader")
const CHUNK := 12
const DIRS := [Vector2i.UP, Vector2i.RIGHT, Vector2i.DOWN, Vector2i.LEFT]
const MISS := Vector2i(-1, -1)
const CLEAR := Color(0, 0, 0, 0)

var side := 0
var tiles := PackedInt32Array()
var anchors := PackedInt32Array()
var _pollution := PackedInt32Array()
var _colors := PackedColorArray()
var _overlay := false
var _catalog := Models.new()
var _material := ShaderMaterial.new()
var _overlay_map: ImageTexture
var _rocket_frames := {}
var _flights := {}
var _operating := PackedByteArray()
var _smoke_mesh := QuadMesh.new()
var _chunks: Dictionary = {}
var _water_mask: ImageTexture
var _mirror: RiverReflections

class Chunk:
	var root := Node3D.new()
	var batches: Array[Batch] = []
	var bounds := AABB()
	var level := -1
	var smoke := MultiMeshInstance3D.new()

class Batch:
	var data := MultiMesh.new()
	var node := MultiMeshInstance3D.new()
	var cells := PackedInt32Array()
	var key: int
	var asset: Models.Asset
	var transforms: Array[Transform3D] = []

func _init() -> void:
	_material.shader = SURFACE
	_material.set_shader_parameter("wave_slopes",load("res://world/river_slopes.png"))
	_material.set_shader_parameter("leaf_spray",load("res://art/city/leaf_spray.png"))
	var smoke_material := ShaderMaterial.new()
	smoke_material.shader = preload("smoke.gdshader")
	_smoke_mesh.material = smoke_material

func _ready() -> void:
	if DisplayServer.get_name() == "headless": return
	_mirror = RiverReflections.new()
	add_child(_mirror)
	_catalog.set_map_parameter("reflection_map",_mirror.get_texture(),_material)

func update_city(snapshot: Dictionary, overlay: bool = false) -> void:
	var new_side: int = snapshot.get("side", 0)
	var groups: PackedInt32Array = snapshot.get("tiles", PackedInt32Array())
	var roots: PackedInt32Array = snapshot.get("anchors", PackedInt32Array())
	if groups.size() != new_side * new_side or roots.size() != groups.size():
		return
	var emissions: PackedInt32Array = snapshot.get("pollution", PackedInt32Array())
	var colors: PackedColorArray = snapshot.get("overlay_colors", PackedColorArray())
	var recolor := overlay != _overlay or (overlay and (_colors != colors or (colors.is_empty() and emissions != _pollution)))
	var dirty := {}
	var smoke_dirty := {}
	var operating: PackedByteArray = snapshot.get("operating",PackedByteArray())
	if side != new_side:
		for id: Vector2i in _chunks:
			_chunks[id].root.free()
			if _mirror: _mirror.clear_district(id)
		_chunks.clear()
		for z in range(0, new_side, CHUNK):
			for x in range(0, new_side, CHUNK):
				dirty[_chunk_id(x,z)] = true
	elif tiles != groups or anchors != roots:
		for i in groups.size():
			if tiles[i] == groups[i] and anchors[i] == roots[i]:
				continue
			var x := i % side
			var z := floori(i / float(side))
			if (tiles[i] in [21,46,47,48]) != (groups[i] in [21,46,47,48]):
				# The reflection district includes one tile beyond each corner.
				for dx in [-1,1]:
					for dz in [-1,1]:
						if _inside(x+dx,z+dz): dirty[_chunk_id(x+dx,z+dz)] = true
			dirty[_chunk_id(x,z)] = true
			# Rebuild neighbours' road masks as well as the building anchor.
			for r in [anchors[i], roots[i]]:
				if r >= 0:
					dirty[_chunk_id(r % side, floori(r / float(side)))] = true
			for d: Vector2i in DIRS:
				if _inside(x+d.x,z+d.y):
					dirty[_chunk_id(x+d.x,z+d.y)] = true
					# A neighbouring facade can be owned by a different chunk.
					var neighbour: int = (z+d.y)*side+x+d.x
					for r in [anchors[neighbour],roots[neighbour]]:
						if r >= 0: dirty[_chunk_id(r%side,floori(r/float(side)))] = true
	if new_side != side or groups != tiles:
		_update_water_mask(groups, new_side)
	if operating != _operating and new_side == side:
		for i in groups.size():
			var old_active := i < _operating.size() and _operating[i] != 0
			var new_active := i < operating.size() and operating[i] != 0
			if old_active != new_active and groups[i] in [10,12,18]:
				smoke_dirty[_chunk_id(i%side,floori(i/float(side)))] = true
	var frames: Dictionary = snapshot.get("rocket_frames", {})
	if side == new_side and frames != _rocket_frames:
		for cell: int in frames:
			if mini(frames[cell],5) != mini(_rocket_frames.get(cell,-1),5):
				dirty[_chunk_id(cell%side,floori(cell/float(side)))] = true
	_rocket_frames = frames
	_operating = operating
	side = new_side
	tiles = groups
	anchors = roots
	_pollution = emissions
	_colors = colors
	_overlay = overlay
	for id: Vector2i in dirty:
		_build_chunk(id)
	for id: Vector2i in smoke_dirty:
		if not dirty.has(id):
			_update_smoke(_chunks[id])
	_update_flights()
	if recolor or (_overlay and not _overlay_map):
		_update_overlay()
	_catalog.set_map_parameter("overlay_active",_overlay,_material)

func _update_flights() -> void:
	for cell: int in _flights.keys():
		if _rocket_frames.get(cell, -1) != 5:
			_flights[cell].free()
			_flights.erase(cell)
	for cell: int in _rocket_frames:
		if _rocket_frames[cell] != 5: continue
		if not _flights.has(cell):
			var vehicle := MeshInstance3D.new()
			vehicle.mesh = _catalog.asset(23,5,_material).flight
			vehicle.position = Vector3(cell%side+2.0,0,floori(cell/float(side))+2.0)
			vehicle.set_meta("age",0.0)
			add_child(vehicle)
			_flights[cell] = vehicle
		_flights[cell].visible = not _overlay

func advance_rockets(delta: float) -> void:
	for vehicle: MeshInstance3D in _flights.values():
		var age := float(vehicle.get_meta("age")) + delta
		vehicle.position.y = 16.0 * age * age
		vehicle.set_meta("age",age)

func _update_overlay() -> void:
	if not _overlay or side <= 0: return
	var colors := PackedByteArray()
	colors.resize(tiles.size()*4)
	for i in tiles.size():
		var color := _color_at(i)
		var offset := i*4
		colors[offset] = color.r8
		colors[offset+1] = color.g8
		colors[offset+2] = color.b8
		colors[offset+3] = color.a8
	var image := Image.create_from_data(side,side,false,Image.FORMAT_RGBA8,colors)
	if _overlay_map and _overlay_map.get_width() == side:
		_overlay_map.update(image)
	else:
		_overlay_map = ImageTexture.create_from_image(image)
		_catalog.set_map_parameter("overlay_map",_overlay_map,_material)

func _update_water_mask(groups: PackedInt32Array, width: int) -> void:
	if width < 1: return
	var pixels := PackedByteArray()
	pixels.resize(groups.size()*2)
	for i in groups.size():
		if groups[i] in [21,46,47,48]: pixels[i*2] = 255
		if groups[i] == 42: pixels[i*2+1] = 255
	var image := Image.create_from_data(width,width,false,Image.FORMAT_RG8,pixels)
	if _water_mask and _water_mask.get_width() == width:
		_water_mask.update(image)
	else:
		_water_mask = ImageTexture.create_from_image(image)
		_catalog.set_map_parameter("water_mask",_water_mask,_material)
	_catalog.set_map_parameter("map_width",float(width),_material)

func _chunk_id(x: int, z: int) -> Vector2i:
	return Vector2i(floori(x / float(CHUNK)), floori(z / float(CHUNK)))

func _inside(x: int, z: int) -> bool:
	return x >= 0 and z >= 0 and x < side and z < side

func _transport_kind(g: int) -> int:
	if g in [9,48]: return 9
	if g in [7,11,46,47]: return 11
	return g

func _mask(x: int,z: int,g: int) -> int:
	var result := 0
	for i in 4:
		var d: Vector2i = DIRS[i]
		if _inside(x+d.x,z+d.y):
			var other := tiles[(z+d.y)*side+x+d.x]
			if _transport_kind(other)==_transport_kind(g) or (g==1 and other in [2,3,10,49]):
				result |= (1<<i)
	return result

func _bank_direction(x: int,z: int) -> int:
	for i in DIRS.size():
		var neighbour: Vector2i = Vector2i(x,z)+DIRS[i]
		if _inside(neighbour.x,neighbour.y) and tiles[neighbour.y*side+neighbour.x] == 21:
			return i
	return -1

func _near_water(start: Vector2i) -> bool:
	for z in range(maxi(start.y-1,0),mini(side,start.y+CHUNK+1)):
		for x in range(maxi(start.x-1,0),mini(side,start.x+CHUNK+1)):
			if tiles[z*side+x] in [21,46,47,48]: return true
	return false

func _build_chunk(id: Vector2i) -> void:
	if _chunks.has(id):
		var old: Chunk = _chunks[id]
		old.root.free()
		if _mirror: _mirror.clear_district(id)
	var chunk := Chunk.new()
	_chunks[id] = chunk
	chunk.root.name = "District_%d_%d" % [id.x,id.y]
	var start := id * CHUNK
	var grouped := {}
	for z in range(start.y,mini(side,start.y+CHUNK)):
		for x in range(start.x,mini(side,start.x+CHUNK)):
			var i := z*side+x
			if anchors[i]>=0 and anchors[i]!=i: continue
			var g := tiles[i]
			if g < 0 or g >= Models.SIZE.size(): g=0
			var variant := 0
			if g == 23:
				variant = mini(_rocket_frames.get(i,0),5)
			elif g in Models.NETWORKS:
				variant = _mask(x,z,g)
			elif g in Models.VARIED:
				variant = absi(hash(Vector2i(x,z))) % 3
			var key := g*16+variant
			if not grouped.has(key): grouped[key]=[]
			grouped[key].append(i)
			# Keep decorative batches out of selection. Trees need a straight sidewalk,
			# with spacing stable when adjacent chunks are rebuilt.
			var decoration := -1
			if g in [0,19,42,43,44,45] and _bank_direction(x,z) >= 0:
				decoration = Models.SHORE
			elif g == 11 and variant in [5,10] and (x+z)%3 == 0:
				decoration = Models.AVENUE
			if decoration >= 0:
				var decoration_key := decoration*16+(absi(hash(i))%2)
				if not grouped.has(decoration_key): grouped[decoration_key]=[]
				grouped[decoration_key].append(i)
	var has_bounds := false
	var river_district := _mirror != null and _near_water(start)
	for key: int in grouped:
		var group := floori(key/16.0)
		var batch := Batch.new()
		batch.key = key
		batch.asset = _catalog.asset(group,key%16,_material)
		batch.cells = PackedInt32Array(grouped[key])
		batch.data.transform_format = MultiMesh.TRANSFORM_3D
		batch.data.mesh = batch.asset.visual[0]
		batch.data.instance_count = batch.cells.size()
		var half_size: float = Models.SIZE[group]*.5 if group < Models.SIZE.size() else .5
		for j in batch.cells.size():
			var i := batch.cells[j]
			var x := i%side
			var z := floori(i/float(side))
			var basis := Basis.IDENTITY
			if group in [4,6,36,37,38,39,40]:
				basis = _frontage(x,z,Models.SIZE[group])
			elif group in [19,43,44,45]:
				basis = Basis(Vector3.UP,(absi(hash(i))%4)*PI*.5)
			var position_3d := Vector3(x+half_size,0,z+half_size)
			if group == Models.SHORE:
				basis = Basis(Vector3.UP,(2-_bank_direction(x,z))*PI*.5)
			elif group == Models.AVENUE:
				var axis := _mask(x,z,11)
				var tree_side := 1.0 if (floori((x+z)/3.)%2 == 0) else -1.0
				basis = Basis(Vector3.UP,0. if axis == 5 else PI*.5)
				position_3d += basis*Vector3(.438*tree_side,0,0)
			var transform := Transform3D(basis,position_3d)
			batch.data.set_instance_transform(j,transform)
			batch.transforms.append(transform)
			var bounds := transform * batch.asset.bounds
			chunk.bounds = chunk.bounds.merge(bounds) if has_bounds else bounds
			has_bounds = true
		var node := batch.node
		node.material_override = batch.asset.materials[0]
		node.multimesh = batch.data
		if group in [0,21,35,42]:
			node.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		chunk.root.add_child(node)
		chunk.batches.append(batch)
		if river_district and batch.asset.bounds.size.y > .21 and group != Models.SHORE:
			_mirror.add_batch(id,batch.asset,batch.transforms,group in [19,43,44,45,50,Models.AVENUE])
	chunk.root.add_child(chunk.smoke)
	chunk.smoke.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_update_smoke(chunk)
	add_child(chunk.root)
	var camera := get_viewport().get_camera_3d()
	if camera: _set_detail(chunk,camera.global_position)

func _update_smoke(chunk: Chunk) -> void:
	var origins := PackedVector3Array()
	for batch in chunk.batches:
		if batch.asset.outlets.is_empty(): continue
		for j in batch.cells.size():
			var i := batch.cells[j]
			if i >= _operating.size() or _operating[i] == 0: continue
			for outlet in batch.asset.outlets:
				origins.append(batch.transforms[j] * outlet)
	if origins.is_empty():
		chunk.smoke.multimesh = null
		return
	var particles := MultiMesh.new()
	particles.transform_format = MultiMesh.TRANSFORM_3D
	particles.use_custom_data = true
	particles.mesh = _smoke_mesh
	particles.instance_count = origins.size()*5
	var bounds := AABB(origins[0],Vector3.ZERO)
	for j in origins.size():
		bounds = bounds.expand(origins[j])
		for puff in 5:
			var index := j*5+puff
			particles.set_instance_transform(index,Transform3D(Basis.IDENTITY,origins[j]))
			particles.set_instance_custom_data(index,Color(fmod(puff*.20+j*.37,1),0,0,0))
	particles.custom_aabb = bounds.grow(1.7)
	chunk.smoke.multimesh = particles

func _process(_delta: float) -> void:
	var camera := get_viewport().get_camera_3d()
	if not camera: return
	if _mirror and _mirror.sync(camera,not _overlay):
		_catalog.set_map_parameter("reflection_active",_mirror.enabled,_material)
	for chunk: Chunk in _chunks.values():
		_set_detail(chunk,camera.global_position)

func _set_detail(chunk: Chunk, eye: Vector3) -> void:
	var distance := eye.distance_to(eye.clamp(chunk.bounds.position,chunk.bounds.end))
	var level := 0 if distance < 30 else (1 if distance < 66 else 2)
	if chunk.level == level: return
	for batch in chunk.batches:
		batch.data.mesh = batch.asset.visual[level]
		batch.node.material_override = batch.asset.materials[level]
	chunk.smoke.visible = level < 2
	chunk.level = level

func _frontage(x: int,z: int,s: int) -> Basis:
	var best := -1
	var orientation := 0
	for i in 4:
		var count := 0
		for lane in s:
			var pos: Vector2i
			match i:
				0: pos = Vector2i(x+lane,z+s)
				1: pos = Vector2i(x+s,z+lane)
				2: pos = Vector2i(x+lane,z-1)
				3: pos = Vector2i(x-1,z+lane)
			if _inside(pos.x,pos.y) and tiles[pos.y*side+pos.x] in [7,11,46,47]:
				count += 1
		if count>best:
			best=count
			orientation=i
	return Basis(Vector3.UP,orientation*PI*.5)

func _color_at(index: int) -> Color:
	if not _overlay: return CLEAR
	if _colors.size()==tiles.size(): return _colors[index]
	if _pollution.size()!=tiles.size() or _pollution[index]<=0: return CLEAR
	var intensity := clampf(_pollution[index]/200.0,0,1)
	var color := Color("#e7bd54").lerp(Color("#bc4939"),intensity)
	color.a=.42+.55*intensity
	return color

func pick_building(origin: Vector3, direction: Vector3) -> Vector2i:
	var best := INF
	var cell := MISS
	for c: Chunk in _chunks.values():
		if c.bounds.intersects_ray(origin,direction)==null: continue
		for b in c.batches:
			if not b.asset.pick: continue
			var bounds := b.asset.pick_bounds
			for j in b.cells.size():
				var tr := b.transforms[j]
				var inverse := tr.inverse()
				var local_origin := inverse*origin
				var local_dir := inverse.basis*direction
				if bounds.intersects_ray(local_origin,local_dir)==null: continue
				var hit := b.asset.pick.intersect_ray(local_origin,local_dir)
				if hit.is_empty(): continue
				var distance := local_origin.distance_squared_to(hit.position)
				if distance<best:
					best=distance
					cell=Vector2i(b.cells[j]%side,floori(b.cells[j]/float(side)))
	return cell
