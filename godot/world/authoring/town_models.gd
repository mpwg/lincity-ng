extends RefCounted

const Builder = preload("mesh_builder.gd")
const STONE := Color("#c9bc9f")
const PLASTER := Color("#e1c998")
const HONEY := Color("#cb9d69")
const BRICK := Color("#aa6448")
const RED_TILE := Color("#a8573b")
const SLATE := Color("#606769")
const IRON := Color("#535f5c")
const TIMBER := Color("#786449")
const GREEN := Color("#647e4e")
const GRASS := Color("#a1ac70")
const WATER := Color("#58969b")
const SIZE := preload("../model_catalog.gd").SIZE

var b: Builder

func make(group: int, variant: int, material: Material, detail: int = 0) -> ArrayMesh:
	b = Builder.new()
	b.detail = detail
	var s: float = SIZE[group] if group < SIZE.size() else 1.0
	match group:
		51: _shore(variant)
		52: _avenue(variant)
		0, 35, 42:
			b.floor_rect(0,0,1,1,0,GRASS,4)
		1: _powerline(variant)
		2:
			_yard(s)
			for x in 4:
				for z in 5:
					var px := -1.45 + x * 0.97
					var pz := -1.55 + z * 0.78
					b.box(px,pz,0.08,0.13,0,0.35,IRON)
					var panel := Color("#526a7c")
					b.quad(Vector3(px-.41,.23,pz-.26),Vector3(px-.41,.43,pz+.26),Vector3(px+.41,.43,pz+.26),Vector3(px+.41,.23,pz-.26),panel)
		3:
			_yard(s)
			_house(-.41,-.22,.65,.88,.46,PLASTER,SLATE)
			for z: float in [-.45,.16,.72]:
				b.box(.48,z,.56,.2,0,.21,IRON)
				for x: float in [.28,.48,.68]:
					b.cylinder(x,z,.2,.04,.04,.51,Color("#8a9c93"),6)
		4,36,37,38,39,40: _residences(group,variant)
		5: _farm()
		6: _market()
		7,9,11,46,47,48: _transport(group,variant)
		8,15:
			b.floor_rect(0,0,s,s,.003,Color("#aa9f87"),6)
			_quarry(.64,.61)
			for i in 3:
				b.box(-1.53+i*.55,-1.59,.41,.39,.008,.27,[TIMBER,IRON,BRICK][i])
			_fence(Vector3(-1.84,0,-1.84),Vector3(1.68,0,-1.84),6)
			if group == 8:
				for x: float in [-1.53,-.70]:
					b.beam(Vector3(x,0,-.93),Vector3(-1.1,1.65,-.79),.065,IRON)
				b.box(-1.10,-.8,.66,.54,1.60,.22,TIMBER)
				_house(-.84,1.28,1.55,.68,.44,BRICK,SLATE)
				for z: float in [-1.02,-.59]:
					b.beam(Vector3(-1.53,.02,z),Vector3(-1.1,1.57,z),.043,IRON)
					b.beam(Vector3(-.71,.02,z),Vector3(-1.1,1.57,z),.043,IRON)
					b.beam(Vector3(-1.41,.46,z),Vector3(-.89,1.03,z),.023,IRON)
					b.beam(Vector3(-.79,.46,z),Vector3(-1.28,1.03,z),.023,IRON)
				for dx: float in [-.11,.11]:
					b.beam(Vector3(-1.05+dx,1.65,-.63),Vector3(-.94+dx,.54,1.08),.016,IRON)
			else:
				_crane(-1.12,-.82,1.2)
		10,12,18: _industry(group)
		13,22,26,30: _civic(group,s)
		14:
			_yard(s,true)
			for z: float in [-1.15,0.0,1.15]:
				_house(-.93,z,1.5,.74,.46,PLASTER,RED_TILE)
				_tree(.95,z,.91,1)
		16:
			_yard(s)
			for i in 9:
				b.cylinder(-1.15+(i%3)*1.08,-1.1+floori(i/3.0)*1.05,.02,.69,.16,.36+.18*(i%3),[STONE,TIMBER,SLATE][i%3],7)
		17:
			_yard(s)
			_house(-.70,-.65,2.15,1.05,.69,HONEY,SLATE,2)
			_crane(.99,.56,1.62)
			for i in 5:
				b.box(-1.40+i*.48,.91,.41,.71,0,.39,[BRICK,TIMBER,GREEN][i%3])
		19,43,44,45,50:
			b.floor_rect(0,0,1,1,0,GRASS,4)
			if group == 50:
				b.cylinder(.03,.05,.006,.37,.36,.01,WATER,12)
				_tree(-.30,-.30,.50,0)
			else:
				_tree(-.12,.08,.82+.08*(variant%3),0 if group in [19,43] else group-43)
				if group == 45:
					_tree(.27,-.25,.63,1)
				if group == 19:
					b.box(.29,-.24,.26,.09,.07,.07,TIMBER)
		20:
			_yard(s)
			_house(0,-.37,1.52,.83,.58,Color("#99a494"),SLATE,2)
			for i in 3:
				b.box(-.55+.55*i,.53,.36,.42,0,.28,[GREEN,HONEY,IRON][i])
		21: b.floor_rect(0,0,1,1,0,WATER,5)
		23:
			_yard(s)
			b.cylinder(0,0,0,.98,.98,.13,STONE)
			if variant in [1,2,3,4]:
				_rocket(variant)
			if variant == 0:
				for x: float in [-.6,0,.6]:
					b.box(x,.66,.46,.24,.13,.18,IRON)
			_crane(-.94,-.72,2.85)
		24,49: _windmill(group)
		25:
			_yard(s)
			for i in 3:
				b.box(0,0,1.12-i*.24,1.12-i*.24,i*.13,.13,STONE)
			b.cylinder(0,0,.39,.23,.14,1.23,STONE,8)
			b.cylinder(0,0,1.62,.15,0,.28,Color("#63877a"),8)
		27,28,29:
			_yard(s)
			_house(-.20,-.11,1.22,.99,.64,BRICK if group==27 else HONEY,RED_TILE if group==29 else SLATE,2)
			_stack(.53,-.46,.95 if group==27 else .77,.10)
			if group==28:
				b.cylinder(.51,.43,0,.29,.29,.95,STONE)
				b.cylinder(.51,.43,.95,.32,0,.24,SLATE)
			elif group==29:
				b.cylinder(.50,.45,0,.34,.17,.47,RED_TILE)
			else:
				b.box(.12,.62,.71,.27,0,.18,IRON)
		31:
			_yard(s,true)
			b.floor_rect(0,.1,1.5,1.03,.018,Color("#84a269"),4)
			b.floor_rect(0,.1,.68,.15,.021,STONE)
			_house(0,-.67,.98,.42,.27,PLASTER,GREEN)
		32,34:
			b.floor_rect(0,0,1,1,0,Color("#716b55"),4)
			for i in 4:
				var x := -.29+.20*i
				b.beam(Vector3(x,0,-.22),Vector3(x+.09,.24+i*.035,.15),.05,Color("#50473c"))
				if group==34:
					b.cylinder(x,.12,.05,.13,0,.46+(i%2)*.26,Color("#e79545"),5)
		33:
			_yard(s)
			for i in 5:
				var x := -.57 + (i%3)*.58
				var z := -.43 + floori(i/3.0)*.83
				_house(x,z,.54,.63,.27,[TIMBER,HONEY,STONE][i%3],SLATE)
		41:
			_yard(s,true)
			b.cylinder(0,0,0,.45,.45,.42,STONE,12,7)
			b.cylinder(0,0,.423,.33,.33,.008,Color("#415d5b"),12)
			for x: float in [-.41,.41]:
				b.box(x,0,.07,.09,.40,.64,TIMBER)
			b.roof(0,0,1.08,.85,.99,.30,RED_TILE)
	return b.finish(material)

func make_flight(material: Material) -> ArrayMesh:
	b = Builder.new()
	_rocket(4)
	b.cylinder(0,0,-.63,.025,.20,.79,Color("#f29b42"),12)
	b.cylinder(0,0,-.32,.04,.16,.49,Color("#ffe3a2"),10)
	return b.finish(material)

func _rocket(stage: int) -> void:
	var height: float = [0.0,.64,1.48,2.26,2.48][stage]
	b.cylinder(0,0,.13,.28,.25,height,Color("#e3dfcc"),14)
	for i in 3:
		var angle := i*TAU/3.0
		b.beam(Vector3(cos(angle)*.46,.16,sin(angle)*.46),
				Vector3(cos(angle)*.21,minf(.96,height+.13),sin(angle)*.21),.07,BRICK)
	if stage == 4:
		b.cylinder(0,0,2.61,.25,0,.69,BRICK,14)

func _yard(s: float, green: bool = false) -> void:
	b.floor_rect(0,0,s,s,.002,GRASS if green else STONE,4 if green else 6)

func _house(x: float,z: float,w: float,d: float,h: float,wall: Color,tile: Color,style: int=1) -> void:
	b.box(x,z,w+.04,d+.04,.008,.08,STONE,7)
	b.box(x,z,w,d,.088,h,wall,style)
	b.box(x,z,w+.055,d+.055,h+.088,.045,STONE)
	b.roof(x,z,w+.13,d+.13,h+.13,minf(d*.42,.49),tile,minf(.13,w*.14))
	b.box(x-w*.27,z-.06,.094,.107,h+.16,minf(d*.42,.49)+.14,BRICK,7)
	b.box(x-w*.27,z-.06,.12,.13,h+.28+minf(d*.42,.49),.033,SLATE)
	b.box(x+w*.21,z+d*.5,.105,.018,.081,.22,TIMBER)

func _residences(g: int,variant: int) -> void:
	_yard(3,true)
	var tier := 0 if g==4 else (g-35)
	var storeys := [1,2,3,2,3,4]
	var palette := [PLASTER,HONEY,Color("#d3aa8c"),Color("#ede1bd"),Color("#c4b79d")]
	var roofs := [RED_TILE,Color("#875343"),SLATE,Color("#a96746")]
	var dense := tier >= 3
	for row in 2:
		for col in (3 if dense else 2):
			var w := .88 if dense else 1.02
			var x := -0.94 + col*.94 if dense else -.71 + col*1.42
			var z := -.83 + row*1.66
			var n := variant + col + 3*row + tier
			var h: float = .23 + float(storeys[tier])*.27 + .044*(n%3)
			_house(x,z,w,.84+(n%2)*.10,h,palette[n%5],roofs[n%4])
			if tier>0 and n%3!=1:
				b.box(x+.08,z+.31,.23,.22,h+.16,.17,PLASTER,1)
				b.roof(x+.08,z+.31,.28,.27,h+.33,.09,roofs[n%4])
	b.floor_rect(0,0,3,.37,.014,STONE,6)
	_tree(-1.23,.08,.52,1)
	_tree(1.23,-.15,.60,0)
	_fence(Vector3(-1.43,0,-1.42),Vector3(1.42,0,-1.42),6)
	_lantern(1.34,.21)
	for x: float in [-.9,.86]:
		b.box(x,.42,.43,.10,.06,.065,TIMBER)

func _fence(a: Vector3, end: Vector3, sections: int) -> void:
	if b.detail > 0: return
	for i in (sections+1):
		var post := a.lerp(end,i/float(sections))
		b.box(post.x,post.z,.037,.037,.01,.18,TIMBER)
	for y: float in [.066,.143]:
		b.beam(a+Vector3.UP*y,end+Vector3.UP*y,.012,TIMBER)

func _lantern(x: float,z: float) -> void:
	if b.detail > 0: return
	b.cylinder(x,z,.012,.043,.031,.10,IRON,6)
	b.cylinder(x,z,.11,.025,.016,.55,IRON,6)
	b.cylinder(x,z,.66,.027,.061,.084,PLASTER,4)
	b.cylinder(x,z,.744,.074,.016,.057,IRON,4)

func _quarry(x: float,z: float) -> void:
	# Raised spoil banks conceal the ground plane inside the stepped excavation.
	var previous := 1.12
	var level := .30
	for i in 4:
		var radius := previous-.18
		var next_level := level-.064
		for j in 12:
			var a := j*TAU/12
			var end := (j+1)*TAU/12
			var p := Vector3(cos(a),0,sin(a))
			var q := Vector3(cos(end),0,sin(end))
			var center := Vector3(x,level,z)
			var tint := Color("#92816b").darkened(i*.105)
			b.quad(center+p*previous,center+p*radius,center+q*radius,center+q*previous,tint)
			b.quad(center+p*radius,center+p*radius+Vector3.DOWN*.064,center+q*radius+Vector3.DOWN*.064,center+q*radius,tint.darkened(.16))
			if i==0:
				b.quad(center+p*previous,center+q*previous,Vector3(x,0,z)+q*(previous+.19),Vector3(x,0,z)+p*(previous+.19),tint)
		previous=radius
		level=next_level
	b.cylinder(x,z,.008,previous,previous,level-.008,Color("#645b4e"),12)

func _tree(x: float,z: float,scale: float,kind: int) -> void:
	if b.detail == Builder.PICK: return
	var bark := Color("#756446")
	b.cylinder(x,z,.003,.052*scale,.03*scale,.67*scale,bark,7)
	if kind == 2:
		for i in (5 if b.detail < 2 else 3):
			var t := i/(4.0 if b.detail < 2 else 2.0)
			var radius := lerpf(.38,.085,t)*scale
			var y := (.20+t*.67)*scale
			b.cylinder(x,z,y,radius,0,scale*(.49-.19*t),Color("#476947").lightened(t*.08),9,8)
			if b.detail == 0:
				for twig in 5:
					var a := TAU*twig/5+i*.71
					var dx := cos(a)*radius*.65
					var dz := sin(a)*radius*.65
					b.crown(x+dx,z+dz,y-.07*scale,radius*.47,.20*scale,Color("#516e4b"),a)
		return
	var leaf := Color("#69834d") if kind == 0 else Color("#82955a")
	var count := [8,5,3][b.detail] as int
	for i in count:
		var a := i*2.39996 + kind
		var t := i/float(count)
		var spread := (.24-.14*t)*scale*(.79 if kind == 1 else 1.0)
		var base := (.31+t*.29)*scale
		var center := Vector3(x+cos(a)*spread,base,z+sin(a)*spread)
		var radius := (.245+.055*sin(i*3.15+1))*scale*(1.25 if b.detail > 0 else 1.)
		var height := (.43+.10*t)*scale*(1.19 if kind == 1 else 1.)
		if b.detail < 2:
			b.beam(Vector3(x,.28*scale,z),center+Vector3.UP*height*.55,.025*scale,bark)
		var foliage := leaf.darkened(.09*(1.-t)).lightened(.027*sin(i*2.0))
		b.crown(center.x,center.z,center.y,radius,height,foliage,a)
		if b.detail < 2:
			for j in (4 if b.detail == 0 else 2):
				var angle := a+j*1.59
				var out := Vector3(cos(angle),0,sin(angle))
				var spray_base := center + out*radius*.62 + Vector3.UP*height*(.05+.042*j)
				b.leaf_spray(spray_base,out,radius*2.6,height*1.16,foliage.lightened(.08))


func _shore(variant: int) -> void:
	var stone := Color("#8e927d")
	for i in (4 if b.detail == 0 else 2):
		var x := -.38 + i*.19
		var z := .48 + sin(i*4.1+variant)*.047
		b.cylinder(x,z,-.01,.07+.023*sin(i+variant),.045,.041,stone.darkened(.06*i),6)
	if b.detail == 2: return
	for i in (4 if b.detail == 0 else 2):
		var x := -.31+i*.16
		var z := .37+.071*sin(i*4.7+variant)
		for blade in 5:
			var a := TAU*blade/5+i
			var base := Vector3(x,.001,z)
			var tip := base + Vector3(cos(a)*.053,.088+.096*absf(sin(i+blade)),sin(a)*.053)
			var across := Vector3(-sin(a)*.011,0,cos(a)*.011)
			var color := Color("#6e8154").lightened(.027*blade)
			b.triangle(base-across,tip,base+across,color)
			b.triangle(base+across,tip,base-across,color)

func _avenue(variant: int) -> void:
	b.cylinder(0,0,.012,.115,.115,.022,STONE,8)
	b.cylinder(0,0,.034,.093,.093,.006,Color("#80715b"),8)
	_tree(0,0,.72 if variant == 0 else .85,variant)
	if b.detail < 2:
		_lantern(.01,-.39)

func _market() -> void:
	_yard(2)
	_lantern(.90,.79)
	_house(0,-.37,1.72,.70,.62,PLASTER,RED_TILE)
	for col in 4:
		var x := -.69 + col*.46
		b.box(x,.16,.39,.23,.06,.22,TIMBER)
		for stripe in 4:
			var sx := x-.18+stripe*.09
			var cloth := Color("#50806a") if stripe%2==0 else Color("#e4d9ab")
			b.quad(Vector3(sx,.53,-.02),Vector3(sx,.37,.37),Vector3(sx+.091,.37,.37),Vector3(sx+.091,.53,-.02),cloth)
			b.box(sx+.045,.37,.091,.015,.33,.04,cloth)
		for k in 3:
			b.box(x-.11+k*.11,.20,.091,.13,.28,.055,[Color("#b56243"),Color("#a7a35b"),Color("#dbb65e")][(col+k)%3])
	for x: float in [-.65,.57]:
		b.box(x,.73,.43,.30,0,.23,TIMBER)
		for dx: float in [-.18,.18]:
			b.box(x+dx,.73,.025,.025,.2,.32,TIMBER)
		b.roof(x,.73,.55,.43,.51,.11,Color("#57856a"))

func _farm() -> void:
	_yard(4,true)
	_house(-.85,-1.16,1.58,.90,.51,PLASTER,RED_TILE)
	b.cylinder(1.23,-1.16,.01,.33,.33,.70,STONE)
	b.cylinder(1.23,-1.16,.71,.38,.02,.29,SLATE)
	b.floor_rect(0,.68,3.87,2.29,.008,Color("#96815b"))
	for i in 19:
		b.floor_rect(-1.8+i*.20,.68,.14,2.20,.013,[Color("#c5ab68"),Color("#9da35f"),Color("#a5a961")][i%3],4)

func _stack(x: float,z: float,h: float,r: float) -> void:
	b.cylinder(x,z,.01,r*1.36,r*1.3,.24,BRICK,12,7)
	b.cylinder(x,z,.25,r*1.12,r*.72,h,BRICK,12,7)
	for t: float in [.24,.55,.87,1.0]:
		var rad := r*lerpf(1.15,.78,t)
		b.cylinder(x,z,.24+h*t,rad,rad,.042,Color("#784c3c"),12)
	b.cylinder(x,z,.291+h,r*.59,r*.59,.009,Color("#413e36"),12)

func _industry(g: int) -> void:
	var s: float = SIZE[g]
	_yard(s)
	var roof := Color("#707b78")
	var count := 3 if g==18 else 2
	var width := s*.61
	for i in count:
		var z := -s*.26+i*(s*.64/count)
		_house(-s*.12,z,width,s*.69/count,.62 if g==12 else .88,BRICK,roof,2)
		b.box(-s*.13,z,width*.76,.21,.87 if g==12 else 1.13,.16,Color("#899992"),2)
	# Loading doors, lintels and ventilators give each shed a street elevation.
	for i in 3:
		var px := -s*.32+i*s*.19
		var pz := -s*.26+(count-1)*(s*.64/count)+s*.345/count
		b.box(px,pz,.30,.035,.06,.36,Color("#52625d"))
		b.box(px,pz+.02,.34,.046,.42,.046,STONE)
		b.box(px,pz+.055,.40,.13,.012,.062,Color("#9b9584"))
		var vent_z := -s*.24
		b.cylinder(px,vent_z,1.0,.052,.052,.52,IRON,8)
		b.cylinder(px,vent_z,1.52,.076,.048,.05,SLATE,8)
	_fence(Vector3(-s*.46,0,-s*.45),Vector3(s*.39,0,-s*.45),6)
	_stack(s*.32,s*.27,2.31 if g==12 else 3.07,.14 if g==12 else .21)
	if g==10:
		_stack(s*.31,-s*.24,2.83,.18)
		b.cylinder(-.7,1.05,0,.53,.43,.43,Color("#645f56"),9)
	elif g==18:
		for x: float in [-1.32,-.66]:
			b.cylinder(x,1.12,0,.31,.31,1.04,IRON,10)
			b.cylinder(x,1.12,1.04,.33,.14,.28,SLATE,10)
	else:
		b.box(-.67,.96,.82,.5,0,.36,STONE,7)

func _crane(x: float,z: float,h: float) -> void:
	b.box(x,z,.38,.40,0,.17,STONE)
	b.beam(Vector3(x-.13,.17,z),Vector3(x,h,z),.044,HONEY)
	b.beam(Vector3(x+.13,.17,z),Vector3(x,h,z),.044,HONEY)
	b.beam(Vector3(x-.35,h,z),Vector3(x+.77,h+.19,z),.047,HONEY)
	b.beam(Vector3(x,h+.26,z),Vector3(x+.75,h+.18,z),.025,IRON)
	b.beam(Vector3(x+.72,h+.18,z),Vector3(x+.72,h*.55,z),.012,IRON)

func _civic(g: int,s: float) -> void:
	_yard(s)
	var wall: Color = {13:PLASTER,22:Color("#e5d9ba"),26:HONEY,30:BRICK}[g]
	_house(0,-s*.10,s*.81,s*.42,.72 if g!=13 else 1.08,wall,SLATE)
	if g==13:
		_house(0,.45,.87,.83,1.17,PLASTER,SLATE)
		b.cylinder(0,.45,1.70,.23,.16,.31,STONE,8)
		b.cylinder(0,.45,2.01,.24,0,.31,Color("#64857b"),8)
	elif g==30:
		b.box(.60,.48,.45,.43,.01,1.28,BRICK,2)
		b.roof(.60,.48,.54,.52,1.29,.21,SLATE)
		for x: float in [-.44,.01]:
			b.box(x,.26,.32,.035,.01,.36,Color("#a34e3f"))
	elif g==22:
		b.box(0,.26,.40,.033,.67,.40,Color("#e4decf"))
		b.box(0,.29,.087,.024,.72,.27,Color("#ac5346"))
		b.box(0,.29,.25,.025,.80,.088,Color("#ac5346"))
	else:
		_tree(.64,.60,.77,1)
		b.floor_rect(-.13,.65,.91,.33,.025,Color("#af987a"),6)

func _windmill(g: int) -> void:
	_yard(2,true)
	var modern := g==49
	var h := 1.93 if modern else 1.11
	b.cylinder(0,0,0,.085 if modern else .40,.056 if modern else .24,h,STONE,12)
	if not modern:
		b.cylinder(0,0,h,.33,.06,.26,SLATE,10)
	b.box(0,.07,.13,.38,h-.04,.13,IRON)
	var hub := Vector3(0,h+.025,.28)
	var n := 3 if modern else 4
	for i in n:
		var a := i*TAU/n+.42
		var dir := Vector3(sin(a),cos(a),0)
		var sideways := Vector3(cos(a),-sin(a),0)
		var end := hub+dir*(.91 if modern else .72)
		b.beam(hub,end,.024,STONE if modern else TIMBER)
		var face: Color = Color("#dfdcca") if modern else Color("#c7ad7e")
		var p := hub+dir*.22
		b.quad(p,p+sideways*.13,end+sideways*.035,end,face)
		b.quad(end,end+sideways*.035,p+sideways*.13,p,face)

func _powerline(mask: int) -> void:
	b.floor_rect(0,0,1,1,.002,GRASS,4)
	b.cylinder(0,0,0,.046,.032,.91,TIMBER,7)
	var along_x := (mask & 10) != 0
	for t: float in [-.13,.13]:
		var offset := Vector3(0,0,t) if along_x else Vector3(t,0,0)
		b.beam(Vector3(0,.83,0),Vector3(0,.83,0)+offset,.028,TIMBER)
		for i in 4:
			if mask & (1<<i):
				var step: Vector3 = [Vector3.FORWARD,Vector3.RIGHT,Vector3.BACK,Vector3.LEFT][i]
				b.beam(Vector3(0,.85,0)+offset,step*.5+Vector3(0,.80,0)+offset,.009,IRON)

func _transport(g: int,mask: int) -> void:
	var bridge := g>=46
	var rail := g in [9,48]
	var road := g in [11,47]
	var y := .11 if bridge else .018
	b.floor_rect(0,0,1,1,0,WATER if bridge else (STONE if road else GRASS),5 if bridge else (6 if road else 4))
	if mask==0: mask=5
	var track: Color = Color("#656963") if road else (Color("#918a76") if rail else Color("#b59d78"))
	var width := .72 if road else (.50 if rail else .55)
	b.floor_rect(0,0,width,width,y+.002,track,9 if road else 10)
	for i in 4:
		if not mask & (1<<i): continue
		var axis: Vector3 = [Vector3.FORWARD,Vector3.RIGHT,Vector3.BACK,Vector3.LEFT][i]
		var cross := Vector3(axis.z,0,-axis.x)
		var p := axis*.25
		if bridge:
			# Masonry piers rise out of the river under the span.
			var pier := axis*.42
			b.box(pier.x,pier.z,.12 if axis.x!=0 else width*.82,.12 if axis.z!=0 else width*.82,0,.07,STONE,7)
			b.box(p.x,p.z,.5 if axis.x!=0 else width+.17,.5 if axis.z!=0 else width+.17,.03,.072,STONE)
			for side: float in [-1,1]:
				var edge := cross*(width*.5+.055)*side
				b.beam(edge+Vector3.UP*.24,edge+axis*.5+Vector3.UP*.24,.018,IRON)
				b.beam(edge+axis*.28+Vector3.UP*.10,edge+axis*.28+Vector3.UP*.24,.018,IRON)
		b.floor_rect(p.x,p.z,.50 if axis.x!=0 else width,.50 if axis.z!=0 else width,y+.003,track,9 if road else 10)
		if rail:
			for k in 3:
				var tie := axis*(.08+.158*k)
				b.floor_rect(tie.x,tie.z,.071 if axis.x!=0 else .43,.071 if axis.z!=0 else .43,y+.008,TIMBER)
			for side: float in [-1,1]:
				var offset := cross*.145*side
				b.beam(offset+Vector3.UP*(y+.03),offset+axis*.5+Vector3.UP*(y+.03),.015,IRON)
		elif road:
			for side: float in [-1.0,1.0]:
				var curb := p+cross*(width*.5+.013)*side
				# Only the exposed part of a curb survives at a junction.
				var edge_direction := (i+1 if side<0 else i+3)%4
				var length := .5 if not mask & (1<<edge_direction) else (.5 - width*.5)
				curb += axis*(.5-length)*.5
				b.box(curb.x,curb.z,length if axis.x!=0 else .025,length if axis.z!=0 else .025,y+.003,.012,STONE)
			var line := axis*.35
			b.floor_rect(line.x,line.z,.16 if axis.x!=0 else .024,.16 if axis.z!=0 else .024,y+.009,Color("#d5ceaf"))
