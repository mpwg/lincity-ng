"""Blender 4.5 LTS source for the townhouses, municipal market and riverside works.

Blender's -Y elevation is the street frontage, imported as +Z by Godot.
All dimensions are in simulation cells. Object transforms stay at identity.
"""

from pathlib import Path
from math import pi, cos, sin
import sys
import json
import struct
from tempfile import TemporaryDirectory

sys.dont_write_bytecode = True
import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from layout import NAMES

DEST = HERE.parents[2] / "godot/art/architecture"
DEST.mkdir(parents=True, exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
image = bpy.data.images.load(str(HERE / "architecture.png"))
image.pack()
material = bpy.data.materials.new("Stadtbau • Fassadenatlas")
material.use_nodes = True
shader = material.node_tree.nodes.get("Principled BSDF")
shader.inputs["Roughness"].default_value = 0.86
texture = material.node_tree.nodes.new("ShaderNodeTexImage")
texture.image = image
material.node_tree.links.new(texture.outputs["Color"], shader.inputs["Base Color"])


class Geometry:
    def __init__(self, lod):
        self.lod = lod
        self.verts, self.faces, self.uvs = [], [], []

    def face(self, points, tile, uv=None):
        i = NAMES.index(tile)
        if uv is None:
            uv = [(0, 0), (1, 0), (1, 1), (0, 1)][: len(points)]
        self.faces.append(tuple(range(len(self.verts), len(self.verts) + len(points))))
        self.verts.extend(points)
        self.uvs.extend(
            [
                (
                    (i % 8 * 136 + 4.5 + u * 127) / (8 * 136),
                    ((4 - i // 8) * 136 + 4.5 + v * 127) / (5 * 136),
                )
                for u, v in uv
            ]
        )

    def box(self, x, y, z, w, d, h, mat):
        if min(w, d, h) <= 0:
            return
        a, b = x - w / 2, x + w / 2
        c, e = y - d / 2, y + d / 2
        self.face([(a, c, z), (b, c, z), (b, c, z + h), (a, c, z + h)], mat)
        self.face([(b, c, z), (b, e, z), (b, e, z + h), (b, c, z + h)], mat)
        self.face([(b, e, z), (a, e, z), (a, e, z + h), (b, e, z + h)], mat)
        self.face([(a, e, z), (a, c, z), (a, c, z + h), (a, e, z + h)], mat)
        self.face([(a, c, z + h), (b, c, z + h), (b, e, z + h), (a, e, z + h)], mat)

    def paving(self, w, d, top):
        self.box(0, 0, 0.002, w, d, top - 0.002, "stone")
        if self.lod == 3:
            return
        n = max(1, round(w / 0.58)) if self.lod < 2 else 1
        for i in range(n):
            for j in range(n):
                a, b = w * (i / n - 0.5), w * ((i + 1) / n - 0.5)
                c, e = d * (j / n - 0.5), d * ((j + 1) / n - 0.5)
                self.face(
                    [
                        (a, c, top + 0.0005),
                        (b, c, top + 0.0005),
                        (b, e, top + 0.0005),
                        (a, e, top + 0.0005),
                    ],
                    "paving",
                )

    def shaft(self, x, y, z, r, q, h, mat, segments=12):
        ring = [
            (cos(i * 2 * pi / segments), sin(i * 2 * pi / segments))
            for i in range(segments)
        ]
        for i, (a, b) in enumerate(ring):
            c, d = ring[(i + 1) % segments]
            self.face(
                [
                    (x + a * r, y + b * r, z),
                    (x + c * r, y + d * r, z),
                    (x + c * q, y + d * q, z + h),
                    (x + a * q, y + b * q, z + h),
                ],
                mat,
            )
        self.face(
            [(x + a * q, y + b * q, z + h) for a, b in ring],
            mat,
            [(0.5 + 0.48 * a, 0.5 + 0.48 * b) for a, b in ring],
        )

    def hip(self, x, y, z, w, d, rise, mat, hip=0.7):
        ex = w / 2
        ey = d / 2
        k = min(ex * 0.83, ey * hip)
        a = (x - ex, y - ey, z)
        b = (x + ex, y - ey, z)
        c = (x + ex, y + ey, z)
        d0 = (x - ex, y + ey, z)
        l = (x - ex + k, y, z + rise)
        r = (x + ex - k, y, z + rise)
        # Split each long slope into continuous strips so the pantiles keep their proportions.
        count = max(1, round(w / 0.45)) if self.lod < 2 else 1

        def mix(u, v, t):
            return tuple(p + (q - p) * t for p, q in zip(u, v))

        for p, q, t, s in [(a, b, r, l), (c, d0, l, r)]:
            for i in range(count):
                lo, hi = i / count, (i + 1) / count
                self.face(
                    [mix(p, q, lo), mix(p, q, hi), mix(s, t, hi), mix(s, t, lo)], mat
                )
        self.face([b, c, r], mat, [(0, 0), (1, 0), (0.5, 1)])
        self.face([d0, a, l], mat, [(0, 0), (1, 0), (0.5, 1)])
        if self.lod < 2:
            self.box(x, y, z + rise - 0.006, 2 * (ex - k) + 0.02, 0.026, 0.021, mat)

    def panel(self, p0, p1, z, h, mat):
        self.face([(*p0, z), (*p1, z), (*p1, z + h), (*p0, z + h)], mat)

    def finish(self, col, name):
        mesh = bpy.data.meshes.new(name)
        mesh.from_pydata(self.verts, [], self.faces)
        mesh.materials.append(material)
        atlas = mesh.uv_layers.new(name="Atlas")
        atlas.data.foreach_set("uv", [k for pair in self.uvs for k in pair])
        mesh.uv_layers.new(name="Style", do_init=False)
        attr = mesh.color_attributes.new(
            name="Color", type="BYTE_COLOR", domain="CORNER"
        )
        attr.data.foreach_set("color_srgb", [1.0] * (len(mesh.loops) * 4))
        obj = bpy.data.objects.new(name, mesh)
        col.objects.link(obj)
        return obj


def elevations(x, y, w, d):
    a, b = x - w / 2, x + w / 2
    c, e = y - d / 2, y + d / 2
    return [((a, c), (b, c)), ((b, c), (b, e)), ((b, e), (a, e)), ((a, e), (a, c))]


def facade(
    g, x, y, z, w, d, h, cols, rows, bay, door=None, far=None, trim=False, shops=False
):
    for side, (p, q) in enumerate(elevations(x, y, w, d)):
        direction = (q[0] - p[0], q[1] - p[1])
        normal = Vector((direction[1], -direction[0])).normalized()
        n = cols[side % 2]
        if g.lod >= 2:
            g.panel(p, q, z, h, far or bay)
            continue
        for i in range(n):

            def pos(t):
                return (p[0] + direction[0] * t, p[1] + direction[1] * t)

            lo, hi = pos(i / n), pos((i + 1) / n)
            for j in range(rows):
                entrance = door and i == n // 2 and j == 0 and side % 2 == 0
                panel_mat = door if entrance else bay
                if shops and side == 0 and j == 0:
                    panel_mat = "door" if entrance else "shop"
                g.panel(lo, hi, z + h * j / rows, h / rows, panel_mat)
                if g.lod == 0 and trim:
                    length = Vector(direction).length / n
                    cx, cy = pos((i + 0.5) / n)
                    cx += normal.x * 0.010
                    cy += normal.y * 0.010
                    # Raised window sills, wider entrance entablatures and lintels.
                    for level in ([0.90] if entrance else [0.13, 0.91]):
                        g.box(
                            cx,
                            cy,
                            z + (j + level) * h / rows,
                            length * 0.75 if side % 2 == 0 else 0.026,
                            0.026 if side % 2 == 0 else length * 0.75,
                            0.015,
                            "cream",
                        )


def chimney(g, x, y, z, h):
    g.box(x, y, z, 0.104, 0.09, h, "brick")
    if g.lod < 2:
        g.box(x, y, z + h - 0.01, 0.13, 0.115, 0.028, "stone")
        g.box(x, y, z + h + 0.018, 0.069, 0.057, 0.008, "black")


def lodging(g, x, y, w, d, stories, color, roof, variant, shops=False):
    base = 0.055
    h = stories * 0.255 + 0.035
    eave = base + h
    wall = "wall" + str(color)
    g.box(x, y, 0.012, w + 0.045, d + 0.045, 0.043, "stone")
    if g.lod == 3:
        g.box(x, y, base, w, d, h, wall)
    else:
        facade(
            g,
            x,
            y,
            base,
            w,
            d,
            h,
            (max(3, round(w / 0.265)), max(2, round(d / 0.25))),
            stories,
            "bay" + str(color),
            "door",
            "front" + str(color),
            True,
            shops,
        )
    roof_h = 0.23 + 0.055 * variant
    g.hip(x, y, eave + 0.018, w + 0.08, d + 0.08, roof_h, roof)
    if g.lod == 3:
        return
    if g.lod < 2:
        g.box(x, y, eave - 0.019, w + 0.063, d + 0.063, 0.036, "cream")
        g.box(x, y, eave - 0.046, w + 0.028, d + 0.028, 0.018, "stone")
        if stories >= 3:
            g.box(
                x, y, base + h / stories - 0.016, w + 0.032, d + 0.032, 0.024, "cream"
            )
        if g.lod == 0:
            for sx in [-1, 1]:
                for sy in [-1, 1]:
                    for i in range(stories * 2):
                        g.box(
                            x + sx * (w / 2 - 0.039),
                            y + sy * (d / 2 - 0.031),
                            base + i * h / (stories * 2),
                            0.098,
                            0.079,
                            0.070,
                            "stone",
                        )
        if (variant != 0 or stories >= 3) and g.lod == 0:
            # Small timber dormer and zinc hood on the street-facing slope.
            for sx in [-0.24, 0.24]:
                g.box(x + sx * w, y - d * 0.28, eave + 0.06, 0.165, 0.16, 0.13, wall)
                g.panel(
                    (x + sx * w - 0.058, y - d * 0.28 - 0.081),
                    (x + sx * w + 0.058, y - d * 0.28 - 0.081),
                    eave + 0.073,
                    0.097,
                    "glass",
                )
                g.hip(
                    x + sx * w,
                    y - d * 0.28,
                    eave + 0.19,
                    0.205,
                    0.195,
                    0.052,
                    roof,
                    0.22,
                )
    chimney(g, x - w * 0.28, y + 0.06, eave + roof_h * 0.54, 0.19)
    if g.lod < 2 and w > 1.1:
        chimney(g, x + w * 0.30, y - 0.07, eave + roof_h * 0.48, 0.17)


def home(g, group, v):
    tier = [4, 36, 37, 38, 39, 40].index(group)
    dense = tier >= 3
    stories = [1, 2, 3, 2, 3, 4][tier]
    if g.lod != 3:
        g.box(
            0,
            0,
            0.002,
            2.995,
            2.995,
            0.016,
            "paving" if dense or stories >= 2 else "grass",
        )
        g.box(0, 0, 0.018, 2.96, 0.38, 0.012, "paving")
    for row in range(2):
        for col in range(2):
            seq = tier + v * 2 + col + row * 2
            streetwall = dense or (stories >= 2 and row == 0)
            if streetwall:
                # Party walls meet, while independent roofs, colours and bays reveal each property.
                w = 1.47 if col == 0 else 1.46
                x = -0.729 if col == 0 else 0.736
                d = 1.12 + 0.06 * ((row + v) % 2)
                y = (-1.436 + d / 2) if row == 0 else (1.415 - d / 2)
            else:
                w = 1.13 - 0.065 * (seq % 2)
                d = 0.84 + 0.075 * (seq % 3)
                x = -0.715 if col == 0 else 0.715
                y = (-1.37 + d / 2) if row == 0 else (0.81 + 0.035 * (seq % 3))
            levels = stories
            if tier == 4 and row and seq % 3 == 0:
                levels -= 1
            lodging(
                g,
                x,
                y,
                w,
                d,
                levels,
                seq % 5,
                "slate" if seq % 7 == 3 else "tile",
                seq % 3,
                shops=streetwall and row == 0,
            )
    if g.lod < 2:
        for sx in [-1, 1]:
            g.box(sx * 1.44, 0.85, 0.025, 0.05, 0.99, 0.085, "stone")
        if g.lod == 0:
            for x in [-1.14, -0.73, 0.72, 1.12]:
                g.box(x, 0.25, 0.03, 0.30, 0.13, 0.08, "stone")
                g.box(x, 0.25, 0.11, 0.27, 0.10, 0.077, "green")


def canopy(g, x, y, z, w, d):
    # Timber tension brackets, sloped fabric and the stitched hanging valance.
    a, b = x - w / 2, x + w / 2
    g.face([(a, y - d, z - 0.10), (b, y - d, z - 0.10), (b, y, z), (a, y, z)], "cloth")
    g.panel((a, y - d), (b, y - d), z - 0.16, 0.06, "cloth")
    if g.lod == 0:
        for xx in [a + 0.013, b - 0.013]:
            g.box(xx, y - d + 0.015, 0.045, 0.019, 0.019, z - 0.145, "metal")


def pediment(g, x, y, z, w, d, h):
    a, b = x - w / 2, x + w / 2
    for yy, indices in [(y - d / 2, False), (y + d / 2, True)]:
        pts = [(a, yy, z), (b, yy, z), (x, yy, z + h)]
        g.face(pts[::-1] if indices else pts, "brick", [(0, 0), (1, 0), (0.5, 1)])
        if g.lod < 2:
            edge_y = yy + (0.006 if indices else -0.006)
            for left, right in [(a, x), (x, b)]:
                left_z = z + (h if left == x else 0)
                right_z = z + (h if right == x else 0)
                band = [
                    (left, edge_y, left_z - 0.012),
                    (right, edge_y, right_z - 0.012),
                    (right, edge_y, right_z + 0.006),
                    (left, edge_y, left_z + 0.006),
                ]
                g.face(band[::-1] if indices else band, "cream")
    g.face(
        [
            (a, y - d / 2, z),
            (x, y - d / 2, z + h),
            (x, y + d / 2, z + h),
            (a, y + d / 2, z),
        ],
        "tile",
    )
    g.face(
        [
            (x, y - d / 2, z + h),
            (b, y - d / 2, z),
            (b, y + d / 2, z),
            (x, y + d / 2, z + h),
        ],
        "tile",
    )


def market(g):
    # Set the hall at the back of its lot. The steps, customers and stalls occupy the public forecourt.
    y = 0.39
    w, d = 1.91, 1.095
    front = y - d / 2
    if g.lod != 3:
        g.paving(1.99, 1.99, 0.024)
    g.box(0, y, 0.025, w + 0.017, d + 0.024, 0.064, "stone")
    if g.lod == 3:
        g.box(0, y, 0.089, w, d, 0.51, "brick")
    else:
        facade(g, 0, y, 0.089, w, d, 0.51, (7, 4), 1, "brickarch", None, "arcade")
    g.hip(0, y, 0.598, 1.98, d + 0.06, 0.267, "tile", 0.78)
    # The cross-gable projects through the main roof all the way to the ridge.
    entry = front - 0.017
    gw = 0.602
    g.box(0, entry, 0.075, gw - 0.049, 0.20, 0.585, "brick")
    pediment(g, 0, entry + 0.165, 0.652, gw, 0.65, 0.315)
    if g.lod != 3:
        g.panel((-0.20, entry - 0.101), (0.20, entry - 0.101), 0.079, 0.54, "brickarch")
        g.panel((-0.092, entry - 0.162), (0.092, entry - 0.162), 0.683, 0.184, "clock")
        if g.lod < 2:
            g.box(0, entry, 0.65, gw, 0.33, 0.026, "cream")
            g.box(0, y, 0.562, w + 0.027, d + 0.037, 0.038, "cream")
            for side, (p, q) in enumerate(elevations(0, y, w, d)):
                count = 7 if side % 2 == 0 else 4
                for j in range(count + 1):
                    t = j / count
                    px = p[0] + (q[0] - p[0]) * t
                    py = p[1] + (q[1] - p[1]) * t
                    g.box(px, py, 0.082, 0.034, 0.034, 0.501, "stone")
            for xx in [-0.61, 0, 0.61]:
                # Narrow counters with open space under the canopy, separate wooden trays and fruit.
                g.box(xx, -0.491, 0.118, 0.436, 0.123, 0.028, "wood")
                for leg in [-0.19, 0.19]:
                    g.box(xx + leg, -0.49, 0.024, 0.025, 0.077, 0.094, "wood")
                for j, m in enumerate(["red", "green", "gold", "red"]):
                    cx = xx - 0.161 + j * 0.107
                    g.box(cx, -0.491, 0.146, 0.099, 0.115, 0.018, "wood")
                    if g.lod == 0:
                        for k in range(3):
                            g.shaft(
                                cx - 0.028 + k * 0.028,
                                -0.491,
                                0.164,
                                0.017,
                                0.012,
                                0.023,
                                m,
                                6,
                            )
            for j, (xx, yy, shirt) in enumerate(
                [
                    (-0.65, -0.68, "green"),
                    (0.15, -0.64, "slate"),
                    (0.67, -0.78, "red"),
                    (-0.16, -0.82, "brass"),
                    (0.74, -0.55, "metal"),
                ]
            ):
                g.shaft(xx, yy, 0.025, 0.019, 0.017, 0.063, "black", 6)
                g.shaft(xx, yy, 0.085, 0.028, 0.022, 0.060, shirt, 7)
                g.shaft(xx, yy, 0.146, 0.016, 0.016, 0.030, "wall2", 7)
            for xx in [-0.61, 0, 0.61]:
                g.box(xx, front - 0.13, 0.025, 0.46, 0.19, 0.026, "stone")
        for xx in [-0.61, 0, 0.61]:
            canopy(g, xx, front - 0.035, 0.383, 0.443, 0.385)
    else:
        for xx in [-0.61, 0, 0.61]:
            g.box(xx, front - 0.23, 0.19, 0.443, 0.39, 0.12, "wood")
    if g.lod == 0:
        for xx in [-0.57, 0.59]:
            chimney(g, xx, y + 0.075, 0.79, 0.13)


def delivery_truck(g, x, y, covered):
    # A parked cab-over lorry sits between the chimney and the loading platform.
    base = 0.031
    g.box(x, y, base + 0.044, 0.146, 0.366, 0.026, "black")
    g.box(x, y - 0.112, base + 0.063, 0.151, 0.108, 0.052, "green")
    g.box(x, y - 0.104, base + 0.115, 0.144, 0.092, 0.045, "glass")
    g.box(x, y - 0.106, base + 0.16, 0.152, 0.100, 0.012, "green")
    g.box(x, y - 0.169, base + 0.080, 0.077, 0.008, 0.019, "metal")
    for side in [-1, 1]:
        g.box(x + side * 0.056, y - 0.169, base + 0.091, 0.021, 0.009, 0.014, "cream")
    g.box(x, y + 0.055, base + 0.074, 0.16, 0.218, 0.027, "wood")
    if covered:
        g.box(x, y + 0.055, base + 0.101, 0.157, 0.214, 0.110, "cloth")
    else:
        for i in range(2):
            g.box(
                x + 0.012 * i,
                y + 0.008 + i * 0.086,
                base + 0.102,
                0.118,
                0.078,
                0.066,
                "crates",
            )
        for side in [-1, 1]:
            g.box(
                x + side * 0.078, y + 0.056, base + 0.101, 0.007, 0.214, 0.039, "wood"
            )
    # Place wheels on the paved yard and bevel the hubcaps against their dark sidewalls.
    for side in [-1, 1]:
        for axle in [-0.115, 0.112]:
            points = [
                (
                    x + side * 0.081,
                    y + axle + 0.033 * sin(a * pi / 4),
                    base + 0.034 + 0.033 * cos(a * pi / 4),
                )
                for a in range(8)
            ]
            g.face(
                points[::-1] if side == 1 else points,
                "black",
                [
                    (0.5 + 0.46 * sin(a * pi / 4), 0.5 + 0.46 * cos(a * pi / 4))
                    for a in range(8)
                ],
            )
            g.box(
                x + side * 0.081, y + axle, base + 0.027, 0.005, 0.015, 0.015, "metal"
            )


def industrial(g, group):
    heavy = group == 18
    size = 4 if heavy else 3
    if g.lod != 3:
        g.paving(size - 0.02, size - 0.02, 0.031)
    count = 3 if heavy else 2
    width = size * 0.65
    depth = size * 0.72 / count
    for i in range(count):
        x = -size * 0.14
        y = size * (-0.20 + i * 0.74 / count)
        h = (0.91 if heavy else 0.74) + (0.045 if i % 2 else 0)
        if g.lod == 3:
            g.box(x, y, 0.03, width, depth, h, "brick")
        else:
            facade(
                g,
                x,
                y,
                0.03,
                width,
                depth,
                h,
                (6 if heavy else 5, 3),
                1,
                "factoryarch",
                "factorydoor",
                "workshop",
            )
        g.hip(x, y, h + 0.03, width + 0.06, depth + 0.065, 0.162, "metal", 0.12)
        g.box(x, y, h + 0.143, width * 0.79, depth * 0.28, 0.095, "glass")
        g.hip(x, y, h + 0.237, width * 0.82, depth * 0.34, 0.069, "metal", 0.12)
        if g.lod < 2:
            g.box(x, y, h - 0.004, width + 0.035, depth + 0.04, 0.035, "stone")
            for j in range(7 if heavy else 6):
                xx = x - width / 2 + j * width / (6 if heavy else 5)
                for yy in [y - depth / 2, y + depth / 2]:
                    g.box(xx, yy, 0.034, 0.038, 0.058, h - 0.018, "brick")
    # Tapered circular brick stack, corbelled cap and a dark open flue.
    sx, sy, z = size * 0.31, -size * 0.19, 0.029
    h = 3.07 if heavy else 2.31
    r = 0.205 if heavy else 0.151
    seg = [24, 16, 10, 10][g.lod]
    g.box(sx, sy, z, r * 2.8, r * 2.8, 0.16, "stone")
    bands = 9 if g.lod <= 1 else 2
    for i in range(bands):
        f = i / bands
        g.shaft(
            sx,
            sy,
            0.18 + h * 0.91 * f,
            r * (1 - 0.42 * f),
            r * (1 - 0.42 * (i + 1) / bands),
            h * 0.91 / bands,
            "brick",
            seg,
        )
        if g.lod < 2 and i % 2 == 0:
            radius = r * (1 - 0.42 * f) + 0.007
            g.shaft(sx, sy, 0.18 + h * 0.91 * f, radius, radius, 0.022, "metal", seg)
    top = 0.18 + h * 0.91
    g.shaft(sx, sy, top, r * 0.60, r * 0.72, 0.06, "brick", seg)
    g.shaft(sx, sy, top + 0.06, r * 0.73, r * 0.73, 0.04, "stone", seg)
    g.shaft(sx, sy, top + 0.101, r * 0.51, r * 0.51, 0.002, "black", seg)
    ax, ay = size * 0.29, size * 0.29
    if g.lod == 3:
        g.box(ax, ay, 0.03, size * 0.28, size * 0.31, 0.47, "brick")
    else:
        facade(
            g,
            ax,
            ay,
            0.03,
            size * 0.28,
            size * 0.31,
            0.47,
            (3, 3),
            1,
            "factoryarch",
            "factorydoor",
            "workshop",
        )
    g.hip(ax, ay, 0.50, size * 0.30, size * 0.33, 0.14, "metal", 0.16)
    if g.lod <= 1:
        delivery_truck(g, size * 0.39, -0.04, heavy)
        for j in range(4):
            xx = ax - 0.27 + j * 0.16
            yy = ay - size * 0.21
            g.box(xx, yy, 0.032, 0.16, 0.16, 0.17, "crates")
            if j % 3 == 0:
                g.box(xx + 0.013, yy, 0.202, 0.15, 0.15, 0.15, "crates")
        if g.lod == 0:
            for j in range(9):
                px = -size * 0.44 + j * size * 0.10
                g.box(px, size * 0.47, 0.032, 0.051, 0.051, 0.19, "stone")
                if j < 8:
                    for k in [0.09, 0.16]:
                        g.box(
                            px + size * 0.05,
                            size * 0.47,
                            k,
                            size * 0.10,
                            0.016,
                            0.015,
                            "metal",
                        )
    return (sx, sy, top + 0.103)


def export(group, variant):
    key = f"g{group:02d}_v{variant:02d}"
    collection = bpy.data.collections.new(key)
    bpy.context.scene.collection.children.link(collection)
    smoke = None
    objects = []
    for level, name in enumerate(["Near", "Mid", "Far", "Pick"]):
        mesh = Geometry(level)
        if group == 6:
            market(mesh)
        elif group in (12, 18):
            smoke = industrial(mesh, group)
        else:
            home(mesh, group, variant)
        objects.append(mesh.finish(collection, name))
    if smoke:
        marker = bpy.data.objects.new("Smoke", None)
        collection.objects.link(marker)
        marker.location = smoke
        objects.append(marker)
    for obj in objects:
        obj.select_set(True)
    # Store the common texture beside the GLBs. The standard GLB container can
    # reference a relative image URI; Godot then imports the atlas exactly once.
    with TemporaryDirectory() as directory:
        stage = Path(directory)
        bpy.ops.export_scene.gltf(
            filepath=str(stage / "building.gltf"),
            export_format="GLTF_SEPARATE",
            use_selection=True,
            export_yup=True,
            export_texcoords=True,
            export_image_format="JPEG",
            export_jpeg_quality=89,
            export_vertex_color="ACTIVE",
            export_all_vertex_colors=True,
            export_animations=False,
            export_cameras=False,
            export_lights=False,
        )
        doc = json.loads((stage / "building.gltf").read_text())
        buffer = (stage / doc["buffers"][0].pop("uri")).read_bytes()
        img = doc["images"][0]
        atlas = (stage / img["uri"]).read_bytes()
        img["uri"] = "architecture.jpg"
        target = DEST / img["uri"]
        if not target.exists() or target.read_bytes() != atlas:
            target.write_bytes(atlas)
        metadata = json.dumps(doc, separators=(",", ":")).encode()
        metadata += b" " * (-len(metadata) % 4)
        buffer += b"\0" * (-len(buffer) % 4)
        chunks = struct.pack("<I4s", len(metadata), b"JSON") + metadata
        chunks += struct.pack("<I4s", len(buffer), b"BIN\0") + buffer
        (DEST / (key + ".glb")).write_bytes(
            struct.pack("<4sII", b"glTF", 2, len(chunks) + 12) + chunks
        )
    for obj in objects:
        obj.select_set(False)
        component = obj.name
        obj.name = f"{key} {component}"
        if obj.type == "MESH":
            obj.data.name = obj.name
        obj.hide_render = component != "Near"
        obj.hide_set(component != "Near")
    return collection


for group in [6, 12, 18, 4, 36, 37, 38, 39, 40]:
    for variant in range(3 if group not in (6, 12, 18) else 1):
        collection = export(group, variant)
        collection.hide_viewport = group != 6
        collection.hide_render = group != 6
# Save an editable source library with the market open. Each collection uses the same origin.
for area in bpy.context.screen.areas:
    if area.type == "VIEW_3D":
        area.spaces.active.region_3d.view_distance = 4.2
        area.spaces.active.shading.color_type = "TEXTURE"
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(
    filepath=str(HERE / "Stadtarchitektur.blend"), compress=True
)
