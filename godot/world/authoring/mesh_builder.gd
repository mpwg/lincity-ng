extends RefCounted
## Vertices use clockwise winding in Godot. The helpers accept outward-facing
## counterclockwise faces and keep normals, UVs and winding together.

const PICK := 3
var detail := 0
var vertex_count := 0
var surface := SurfaceTool.new()

func _init() -> void:
	surface.begin(Mesh.PRIMITIVE_TRIANGLES)

func triangle(a: Vector3, b: Vector3, c: Vector3, color: Color, style: int = 0) -> void:
	var normal := (b - a).cross(c - a).normalized()
	var horizontal := absf(normal.y) > 0.7
	for vertex: Vector3 in [a, c, b]:
		surface.set_normal(normal)
		surface.set_color(color)
		surface.set_uv2(Vector2(style, 0))
		if horizontal:
			surface.set_uv(Vector2(vertex.x, vertex.z))
		elif absf(normal.x) > absf(normal.z):
			surface.set_uv(Vector2(vertex.z, vertex.y))
		else:
			surface.set_uv(Vector2(vertex.x, vertex.y))
		surface.add_vertex(vertex)
		vertex_count += 1

func quad(a: Vector3, b: Vector3, c: Vector3, d: Vector3, color: Color, style: int = 0) -> void:
	triangle(a, b, c, color, style)
	triangle(a, c, d, color, style)

func floor_rect(x: float, z: float, w: float, d: float, y: float, color: Color, style: int = 0) -> void:
	if detail == PICK and y < .012: return
	if detail == 2 and y > 0.0 and minf(w,d) < .075: return
	var a := Vector3(x - w / 2, y, z - d / 2)
	quad(a, a + Vector3(0, 0, d), a + Vector3(w, 0, d), a + Vector3(w, 0, 0), color, style)

func box(x: float, z: float, w: float, d: float, y: float, h: float, color: Color, wall_style: int = 0) -> void:
	if detail > 0 and minf(w,d) < [.0,.035,.12,.19][detail] and h < .85: return
	var a := Vector3(x - w / 2, y, z - d / 2)
	var b := a + Vector3(w, 0, 0)
	var c := b + Vector3(0, 0, d)
	var e := a + Vector3(0, 0, d)
	var up := Vector3.UP * h
	quad(a, a + up, b + up, b, color, wall_style)
	quad(b, b + up, c + up, c, color, wall_style)
	quad(c, c + up, e + up, e, color, wall_style)
	quad(e, e + up, a + up, a, color, wall_style)
	quad(a + up, e + up, c + up, b + up, color)

func roof(x: float, z: float, w: float, d: float, y: float, h: float, color: Color, hip: float = 0.0) -> void:
	var a := Vector3(x - w / 2, y, z - d / 2)
	var b := a + Vector3(w, 0, 0)
	var c := b + Vector3(0, 0, d)
	var e := a + Vector3(0, 0, d)
	var l := Vector3(a.x + hip, y + h, z)
	var r := Vector3(b.x - hip, y + h, z)
	quad(a, l, r, b, color, 3)
	quad(e, c, r, l, color, 3)
	triangle(a, e, l, color, 3)
	triangle(b, r, c, color, 3)

func cylinder(x: float, z: float, y: float, bottom: float, top: float, h: float, color: Color, sides: int = 10, style: int = 0) -> void:
	if detail > 0:
		if maxf(top,bottom) < [.0,.03,.085,.11][detail] and h < .8: return
		sides = mini(sides, 7 if detail == 1 else 5)
	var center := Vector3(x, y + h, z)
	for i in sides:
		var n := Vector3(cos(i * TAU / sides), 0, sin(i * TAU / sides))
		var m := Vector3(cos((i + 1) * TAU / sides), 0, sin((i + 1) * TAU / sides))
		var a := Vector3(x, y, z) + n * bottom
		var b := Vector3(x, y, z) + m * bottom
		quad(a, center + n * top, center + m * top, b, color, style)
		if top > 0.0:
			triangle(center, center + m * top, center + n * top, color)

func beam(a: Vector3, b: Vector3, radius: float, color: Color) -> void:
	if detail > 0 and radius < [.0,.012,.026,.05][detail]: return
	var axis := (b - a).normalized()
	var u := axis.cross(Vector3.FORWARD if absf(axis.z) < 0.85 else Vector3.RIGHT).normalized() * radius
	var v := axis.cross(u).normalized() * radius
	var ring: Array[Vector3] = [u + v, -u + v, -u - v, u - v]
	for i in 4:
		var j := (i + 1) % 4
		quad(a + ring[i], a + ring[j], b + ring[j], b + ring[i], color)
	quad(b + ring[0], b + ring[1], b + ring[2], b + ring[3], color)

func finish(material: Material) -> ArrayMesh:
	if vertex_count == 0: return null
	surface.index()
	surface.set_material(material)
	return surface.commit()

## Uneven rings and blended normals keep the sunlit tips and shaded skirt continuous.
func crown(x: float,z: float,y: float,r: float,h: float,color: Color,phase: float) -> void:
	var center := Vector3(x,y+h*.48,z)
	var sides: int = [9,7,5][detail]
	var rings := [Vector2(.36,-.44),Vector2(.89,-.27),Vector2(1.0,.02),Vector2(.78,.31),Vector2(.29,.47)]
	var points := PackedVector3Array()
	for band in rings.size():
		var ring: Vector2 = rings[band]
		for i in sides:
			var a: float = TAU*i/sides+phase+sin(band*1.73)*.19
			var lobe: float = 1.0+.13*sin(a*3.0+phase)+.07*sin(i*4.17+band*1.41)
			points.append(center+Vector3(cos(a)*r*ring.x*lobe,h*(ring.y+.047*sin(2.0*a+band)),sin(a)*r*ring.x*lobe))
	for band in rings.size()-1:
		for i in sides:
			var j: int = (i+1)%sides
			var base: int = band*sides
			for k in [base+i,base+sides+j,base+sides+i,base+i,base+j,base+sides+j]:
				_leaf_vertex(points[k],center,r,h,color)
	for i in sides:
		var j: int = (i+1)%sides
		for point in [center+Vector3.DOWN*h*.49,points[j],points[i]]:
			_leaf_vertex(point,center,r,h,color)
		var top: int = (rings.size()-1)*sides
		for point in [points[top+i],points[top+j],center+Vector3.UP*h*.5]:
			_leaf_vertex(point,center,r,h,color)

func _leaf_vertex(point: Vector3, center: Vector3, r: float, h: float, color: Color) -> void:
	var relative := point-center
	var normal := (relative/Vector3(r*r,h*h*.25,r*r)).normalized()
	surface.set_normal(normal)
	surface.set_color(color.darkened(.12*(1.0-normal.y)))
	surface.set_uv2(Vector2(8,0))
	surface.set_uv(Vector2(point.x,point.y))
	surface.add_vertex(point)
	vertex_count += 1

## A two-sided spray of individual twigs sits around the solid crown. UVs
## always follow the stem, independent of the outward-facing normal.
func leaf_spray(base: Vector3, outward: Vector3, width: float, height: float, color: Color) -> void:
	var right := Vector3.UP.cross(outward).normalized()*width*.5
	var top := base+Vector3.UP*height+outward*height*.31
	var points := [base-right,base+right,top+right,top-right]
	var uv := [Vector2(0,1),Vector2(1,1),Vector2(1,0),Vector2(0,0)]
	for j in [0,2,1,0,3,2,0,1,2,0,2,3]:
		surface.set_normal((outward+Vector3.UP*.53).normalized())
		surface.set_color(color)
		surface.set_uv2(Vector2(11,0))
		surface.set_uv(uv[j])
		surface.add_vertex(points[j])
		vertex_count += 1
