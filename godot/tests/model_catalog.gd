extends SceneTree

const Catalog = preload("res://world/model_catalog.gd")
const View = preload("res://world/city_view.gd")
var failures := 0

func _initialize() -> void:
	call_deferred("_run")

func check(ok: bool, message: String) -> void:
	if not ok:
		printerr(message)
		failures += 1

func _run() -> void:
	var catalog := Catalog.new()
	var material := ShaderMaterial.new()
	material.shader = View.SURFACE
	var count := 0
	for group in Catalog.AVENUE+1:
		for variant in Catalog.variant_count(group):
			var file := Catalog.path(group,variant)
			if not ResourceLoader.exists(file):
				check(false,"Fehlendes Modell: "+file)
				continue
			var model := catalog.asset(group,variant,material)
			check(model.visual.size() == 3, "Detailebenen: "+file)
			var near_count := 0
			for mesh in model.visual:
				check(mesh.get_surface_count() == 1,"Material-Batching: "+file)
				var data: Array = mesh.surface_get_arrays(0)
				check(data[Mesh.ARRAY_TEX_UV] != null and data[Mesh.ARRAY_COLOR] != null,"UV und Farbe: "+file)
				var index_count: int = data[Mesh.ARRAY_INDEX].size() if data[Mesh.ARRAY_INDEX] != null else data[Mesh.ARRAY_VERTEX].size()
				check(index_count >= 3,"Leeres Netz: "+file)
				if near_count == 0: near_count = index_count
				check(index_count <= near_count,"Fernmodell hat mehr Dreiecke als Nahmodell: "+file)
			if group in Catalog.AUTHORED:
				check(model.materials[0].get_shader_parameter("atlas") != null,"Fehlender Fassadenatlas: "+file)
				check(model.visual[2].get_faces().size() < model.visual[0].get_faces().size(),"Unwirksames Fernmodell: "+file)
			check((model.pick == null) == (group in Catalog.UNSELECTABLE),"Auswahlgeometrie: "+file)
			check(model.bounds.size.x > .01 and model.bounds.size.z > .01,"Modellgrenzen: "+file)
			check(model.outlets.is_empty() == (group not in [10,12,18]),"Rauchaustritt: "+file)
			count += 1
	var view := View.new()
	root.add_child(view)
	const SIDE := 24
	var tiles := PackedInt32Array()
	tiles.resize(SIDE*SIDE)
	var anchors := PackedInt32Array()
	anchors.resize(tiles.size())
	anchors.fill(-1)
	var active := PackedByteArray()
	active.resize(tiles.size())
	var market := Vector2i(11,3)
	var works := Vector2i(2,14)
	for building in [[market,6,2],[works,10,4]]:
		var anchor: Vector2i = building[0]
		for dz in building[2]:
			for dx in building[2]:
				var index: int = (anchor.y+dz)*SIDE+anchor.x+dx
				tiles[index] = building[1]
				anchors[index] = anchor.y*SIDE+anchor.x
	var state := {"side": SIDE, "tiles":tiles, "anchors":anchors,"operating":active}
	view.update_city(state)
	var market_chunk: View.Chunk = view._chunks[Vector2i.ZERO]
	var works_chunk: View.Chunk = view._chunks[Vector2i(0,1)]
	check(works_chunk.smoke.multimesh == null,"Rauch bei stillstehendem Betrieb")
	var road := tiles.duplicate()
	road[market.y*SIDE+market.x+2] = 11
	state.tiles = road
	view.update_city(state)
	var rotated: View.Chunk = view._chunks[Vector2i.ZERO]
	check(rotated != market_chunk,"Straßenbau hinter der Chunkgrenze aktualisiert den Markt nicht")
	for batch in rotated.batches:
		if batch.key == 6*16:
			check((batch.transforms[0].basis*Vector3.BACK).is_equal_approx(Vector3.RIGHT),"Falsche Marktausrichtung")
			check(rotated.bounds.encloses(batch.transforms[0]*batch.asset.bounds),"Modell liegt außerhalb des Chunks")
			view._set_detail(rotated,Vector3(200,200,200))
			check(batch.data.mesh == batch.asset.visual[2],"Fernsicht verwendet falsches Netz")
	active = active.duplicate()
	active[works.y*SIDE+works.x] = 1
	state.operating = active
	view.update_city(state)
	check(works_chunk == view._chunks[Vector2i(0,1)] and works_chunk.smoke.multimesh != null,"Rauchwechsel ohne Bauänderung")
	state.operating = PackedByteArray()
	view.update_city(state)
	check(works_chunk.smoke.multimesh == null,"Rauch trotz inaktivem Gebäude")
	view.free()
	print("Geprüft: %d GLB-Varianten, Ausrichtung, Modellgrenzen, Detailebenen und Betriebsrauch." % count)
	quit(1 if failures else 0)
