"""Export three original street cars. Blender -Y is the front axle direction."""

from math import cos, pi, sin
from pathlib import Path
import bpy

HERE = Path(__file__).resolve().parent
DEST = HERE.parents[2] / "godot/art/traffic"
DEST.mkdir(exist_ok=True, parents=True)
bpy.ops.wm.read_factory_settings(use_empty=True)

# One opaque material and one indexed primitive per vehicle keep instancing inexpensive.
material = bpy.data.materials.new("Lack, Glas und Beschläge")
material.use_nodes = True
shader = material.node_tree.nodes.get("Principled BSDF")
shader.inputs["Roughness"].default_value = 0.44
shader.inputs["Metallic"].default_value = 0.16
color = material.node_tree.nodes.new("ShaderNodeVertexColor")
color.layer_name = "Lack"
material.node_tree.links.new(color.outputs["Color"], shader.inputs["Base Color"])
RUBBER = "27292b"
CHROME = "b8bdba"
GLASS = "526e78"
GLASS_LIGHT = "79949c"
HEADLAMP = "fff0ba"
RED = "aa392c"


class Body:
    def __init__(self):
        self.verts, self.polygons, self.colors = [], [], []

    def face(self, vertices, color):
        self.polygons.append(
            tuple(range(len(self.verts), len(self.verts) + len(vertices)))
        )
        self.verts.extend(vertices)
        rgb = [int(color[i : i + 2], 16) / 255 for i in (0, 2, 4)]
        self.colors.extend([*rgb, 1.0] * len(vertices))

    def box(self, x, y, z, w, d, h, color):
        a, b = x - w / 2, x + w / 2
        c, e = y - d / 2, y + d / 2
        self.face([(a, c, z), (b, c, z), (b, c, z + h), (a, c, z + h)], color)
        self.face([(b, c, z), (b, e, z), (b, e, z + h), (b, c, z + h)], color)
        self.face([(b, e, z), (a, e, z), (a, e, z + h), (b, e, z + h)], color)
        self.face([(a, e, z), (a, c, z), (a, c, z + h), (a, e, z + h)], color)
        self.face([(a, c, z + h), (b, c, z + h), (b, e, z + h), (a, e, z + h)], color)

    def wheel(self, x, y, direction, r=0.026):
        # Ten-sided tyre with inset silver hubcaps. A flat face touches the road.
        n = 10
        tyre = [
            (sin(pi / n + i * 2 * pi / n) * r, cos(pi / n + i * 2 * pi / n) * r)
            for i in range(n)
        ]
        z = r * cos(pi / n)
        inside = x - direction * 0.012
        for j, (a, b) in enumerate(tyre):
            c, d = tyre[(j + 1) % n]
            vertices = [
                (inside, y + a, z + b),
                (inside, y + c, z + d),
                (x, y + c, z + d),
                (x, y + a, z + b),
            ]
            self.face(vertices[::-1] if direction == 1 else vertices, RUBBER)
        for scale, offset, paint in [
            (1, 0, RUBBER),
            (0.66, 0.0006, CHROME),
            (0.37, 0.0012, "858d8c"),
        ]:
            points = [
                (x + direction * offset, y + a * scale, z + b * scale) for a, b in tyre
            ]
            self.face(points[::-1] if direction == 1 else points, paint)

    def bake(self, collection):
        mesh = bpy.data.meshes.new("Karosserie")
        mesh.from_pydata(self.verts, [], self.polygons)
        mesh.materials.append(material)
        paint = mesh.color_attributes.new(
            name="Lack", type="BYTE_COLOR", domain="CORNER"
        )
        paint.data.foreach_set("color_srgb", self.colors)
        body = bpy.data.objects.new("Body", mesh)
        collection.objects.link(body)
        return body


def build_car(index):
    # Wheelbase, bonnet, windscreen rake and rear overhang distinguish the silhouettes.
    paint, roof, length, front, screen, rear_screen, rear, top = [
        ("dac9a5", "e4d8bd", 0.346, -0.094, -0.050, 0.043, 0.092, 0.103),
        ("718a93", "839ba0", 0.315, -0.089, -0.041, 0.056, 0.123, 0.101),
        ("8b4547", "914a49", 0.365, -0.109, -0.060, 0.117, 0.151, 0.107),
    ][index]
    width = 0.070
    car = Body()
    # The five cross sections taper both ends and turn the shoulders into light-catching bevels.
    half = length / 2 - 0.007
    stations = [
        (-half, width - 0.012, 0.057),
        (-half + 0.022, width - 0.003, 0.065),
        (front, width, 0.067),
        (rear, width, 0.067),
        (half - 0.015, width - 0.004, 0.064),
        (half, width - 0.009, 0.057),
    ]
    rings = []
    for y, w, z in stations:
        rings.append(
            [
                (-w + 0.004, y, 0.025),
                (w - 0.004, y, 0.025),
                (w, y, 0.037),
                (w, y, z - 0.006),
                (w - 0.006, y, z),
                (-w + 0.006, y, z),
                (-w, y, z - 0.006),
                (-w, y, 0.037),
            ]
        )
    car.face(rings[0], paint)
    car.face(rings[-1][::-1], paint)
    for lo, hi in zip(rings, rings[1:]):
        for i in range(8):
            j = (i + 1) % 8
            car.face([lo[i], hi[i], hi[j], lo[j]], paint if i else RUBBER)
    # Four pillars and two or three side lights. Opaque glazing avoids transparent draw calls.
    sill = 0.067
    top_w = 0.054
    belt_w = 0.064
    for direction in (-1, 1):

        def sidepoint(f, upper, offset=0):
            if upper:
                return (
                    direction * (top_w + offset),
                    screen + (rear_screen - screen) * f,
                    top,
                )
            return (direction * (belt_w + offset), front + (rear - front) * f, sill)

        def sideface(points, mat):
            car.face(points if direction == 1 else points[::-1], mat)

        sideface(
            [
                sidepoint(0, False),
                sidepoint(1, False),
                sidepoint(1, True),
                sidepoint(0, True),
            ],
            paint,
        )
        splits = [0.015, 0.51, 0.985] if index != 2 else [0.015, 0.415, 0.750, 0.985]
        for j, (a, b) in enumerate(zip(splits, splits[1:])):
            p, q = sidepoint(a + 0.013, False, 0.0008), sidepoint(
                b - 0.013, False, 0.0008
            )
            r, s = sidepoint(b - 0.019, True, 0.0008), sidepoint(
                a + 0.019, True, 0.0008
            )
            sideface([p, q, r, s], GLASS_LIGHT if j == 0 else GLASS)
        car.box(direction * 0.069, 0, 0.055, 0.0017, length * 0.66, 0.0024, CHROME)
        for y in [-0.036, 0.042] if index != 1 else [-0.021]:
            car.box(direction * 0.070, y, 0.060, 0.002, 0.014, 0.003, CHROME)
        car.wheel(direction * 0.074, -length * 0.295, direction)
        car.wheel(direction * 0.074, length * 0.29, direction)
    car.face(
        [
            (-belt_w, front, sill),
            (belt_w, front, sill),
            (top_w, screen, top),
            (-top_w, screen, top),
        ],
        GLASS_LIGHT,
    )
    car.face(
        [
            (belt_w, rear, sill),
            (-belt_w, rear, sill),
            (-top_w, rear_screen, top),
            (top_w, rear_screen, top),
        ],
        GLASS,
    )
    car.face(
        [
            (-top_w, screen, top),
            (top_w, screen, top),
            (top_w, rear_screen, top),
            (-top_w, rear_screen, top),
        ],
        roof,
    )
    if index == 2:
        for x in [-0.042, 0.042]:
            car.box(x, 0.032, top, 0.002, 0.142, 0.0035, CHROME)
    for end in [-1, 1]:
        y = end * (half + 0.002)
        car.box(0, y, 0.031, 0.131, 0.008, 0.006, CHROME if index != 1 else RUBBER)
        car.box(0, end * (half + 0.0065), 0.036, 0.030, 0.001, 0.007, "e4ddca")
    car.box(0, -half - 0.0008, 0.044, 0.070, 0.003, 0.013, "343b3d")
    for z in [0.047, 0.051, 0.055]:
        car.box(0, -half - 0.0028, z, 0.067, 0.0015, 0.001, CHROME)
    for f in [-1, 1]:
        car.box(f * 0.044, -half - 0.002, 0.046, 0.019, 0.004, 0.014, HEADLAMP)
        car.box(f * 0.055, half + 0.001, 0.044, 0.016, 0.002, 0.011, RED)
        car.box(f * 0.054, -half - 0.002, 0.041, 0.009, 0.003, 0.0038, "d8934d")
    return car


for index, name in enumerate(
    ["Stufenheck in Ecru", "Blaugrauer Kompaktwagen", "Weinroter Kombi"]
):
    collection = bpy.data.collections.new(name)
    bpy.context.scene.collection.children.link(collection)
    obj = build_car(index).bake(collection)
    obj.select_set(True)
    bpy.ops.export_scene.gltf(
        filepath=str(DEST / f"car_{index}.glb"),
        export_format="GLB",
        use_selection=True,
        export_yup=True,
        export_vertex_color="ACTIVE",
        export_all_vertex_colors=True,
        export_animations=False,
    )
    obj.select_set(False)
    obj.name = name
    # Display the source models side by side; each exported origin remains at the road surface.
    obj.location.x = (index - 1) * 0.20

for area in bpy.context.screen.areas:
    if area.type == "VIEW_3D":
        area.spaces.active.region_3d.view_distance = 0.8
        area.spaces.active.shading.color_type = "VERTEX"
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(
    filepath=str(HERE / "Strassenfahrzeuge.blend"), compress=True
)
