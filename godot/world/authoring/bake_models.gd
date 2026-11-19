extends SceneTree
## Rebuild with Godot --headless --path godot --script res://world/authoring/bake_models.gd
const Catalog = preload("../model_catalog.gd")
const Models = preload("town_models.gd")

func _initialize() -> void:
	var models := Models.new()
	var material := StandardMaterial3D.new()
	material.vertex_color_use_as_albedo = true
	for group in Catalog.AVENUE + 1:
		if group in Catalog.AUTHORED: continue
		for variant in Catalog.variant_count(group):
			var model := Node3D.new()
			model.name = "Group%dVariant%d" % [group,variant]
			for detail in 4:
				if detail == 3 and group in Catalog.UNSELECTABLE: continue
				var mesh := models.make(group, variant, material, detail)
				if not mesh:
					if detail == 3: continue
					printerr("Leeres sichtbares Netz: ",group,"/",variant,"/",detail)
					model.free()
					quit(1)
					return
				var node := MeshInstance3D.new()
				node.name = ["Near","Mid","Far","Pick"][detail]
				mesh.resource_name = node.name
				node.mesh = mesh
				model.add_child(node)
				node.owner = model
			if group == 10:
				var marker := Marker3D.new()
				marker.name = "Smoke"
				marker.position = Vector3(1.28,3.37,1.08)
				model.add_child(marker)
				marker.owner = model
			var document := GLTFDocument.new()
			var state := GLTFState.new()
			var status := document.append_from_scene(model,state)
			if status == OK:
				status = document.write_to_filesystem(state,ProjectSettings.globalize_path(Catalog.path(group,variant)))
			model.free()
			if status != OK:
				printerr("GLB-Export fehlgeschlagen: ",group,"/",variant)
				quit(1)
				return
	quit()
