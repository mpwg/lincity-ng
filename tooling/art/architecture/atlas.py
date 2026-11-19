"""Paint the seamless masonry, roof and facade panels used by the Blender exporter."""

from pathlib import Path
from math import cos, sin, pi
from random import Random

import numpy as np
from PIL import Image, ImageDraw

SIZE = 128
WALLS = [
    (222, 202, 162),
    (218, 166, 91),
    (202, 176, 141),
    (227, 215, 190),
    (191, 157, 127),
]
import sys

sys.dont_write_bytecode = True
from layout import NAMES


def paint(name, index):
    rng = Random(6823 + index)
    im = Image.new("RGB", (SIZE, SIZE))
    d = ImageDraw.Draw(im)
    rect = d.rectangle
    line = d.line

    def fill(rgb):
        rect((0, 0, 127, 127), fill=rgb)

    def tint(rgb, change):
        return tuple(max(0, min(255, v + change)) for v in rgb)

    def masonry():
        fill((157, 131, 108))
        for row in range(16):
            for j in range(-1, 9):
                x, y = j * 24 + (row % 2) * 12, row * 8
                c = tint((161, 89, 61), rng.randrange(-19, 20))
                rect((x, y, x + 22, y + 6), c)
                line((x, y, x + 22, y), tint(c, 12))

    def panes(bounds, cols, rows):
        x, y, u, v = bounds
        rect(bounds, (62, 65, 58))
        for i in range(cols):
            for j in range(rows):
                a, b = x + (u - x) * i // cols + 2, y + (v - y) * j // rows + 2
                c, e = (
                    x + (u - x) * (i + 1) // cols - 1,
                    y + (v - y) * (j + 1) // rows - 1,
                )
                tone = tint((64, 82, 85), rng.randrange(-16, 18))
                rect((a, b, c, e), tone)
                line((a, b, c, b), tint(tone, 25))
                if c - a > 8:
                    line((a + 1, e - 2, c, b + 2), tint(tone, 8), 2)

    if name in ("brick", "brickarch", "factoryarch", "factorydoor"):
        masonry()
    if name in ("brickarch", "factoryarch"):
        # Voussoirs remain legible in the distance mesh.
        stone = (184, 146, 107) if name == "brickarch" else (130, 91, 67)
        d.pieslice((12, 14, 116, 110), 180, 360, fill=stone)
        rect((12, 61, 116, 120), stone)
        for j in range(15):
            angle = pi + j * pi / 14
            x, y = 64 + 52 * cos(angle), 62 + 48 * sin(angle)
            line((x, y, 64 + 43 * cos(angle), 62 + 39 * sin(angle)), (110, 84, 67), 2)
        d.pieslice((21, 24, 107, 99), 180, 360, fill=(56, 72, 72))
        for j in range(1, 8):
            a = pi + j * pi / 8
            line((64, 62, 64 + 42 * cos(a), 62 + 37 * sin(a)), (39, 45, 42), 2)
        panes((21, 62, 107, 116), 5, 3)
        rect((10, 117, 118, 121), (90, 81, 72))
        line((10, 117, 118, 117), (209, 186, 148), 2)
        # A solid masonry plinth and spandrels support inset windows rather than a full-height arcade.
        window = im.copy().crop((8, 11, 120, 124))
        masonry()
        width, height = (92, 87) if name == "factoryarch" else (104, 96)
        im.paste(
            window.resize((width, height), Image.Resampling.LANCZOS),
            ((128 - width) // 2, 12),
        )
    elif name == "factorydoor":
        rect((14, 16, 114, 127), (79, 74, 65))
        rect((19, 21, 109, 125), (66, 88, 81))
        for x in range(21, 109, 6):
            line((x, 22, x, 125), (48, 69, 61))
        panes((22, 27, 104, 57), 6, 2)
        line((64, 57, 64, 125), (37, 52, 46), 2)
    elif name in ("tile", "slate"):
        clay = name == "tile"
        base = (154, 78, 52) if clay else (82, 88, 91)
        fill(tint(base, -23))
        for row in range(13):
            for col in range(-1, 12):
                x, y = col * 12 + row % 2 * 6, row * 10 - 2
                c = tint(base, rng.randrange(-15, 16))
                rect((x, y, x + 10, y + 8), c)
                line((x + 1, y, x + 1, y + 7), tint(c, 17))
                line((x + 2, y + 8, x + 10, y + 8), tint(c, -17))
    elif name in ("paving", "stone"):
        fill((126, 119, 104))
        for row in range(8):
            for col in range(-1, 6):
                x, y = col * 32 + row % 2 * 16, row * 16
                c = tint(
                    (162, 155, 136) if name == "paving" else (191, 179, 156),
                    rng.randrange(-12, 12),
                )
                rect((x, y, x + 30, y + 14), c)
                line((x + 1, y, x + 30, y), tint(c, 12))
    elif name.startswith(("wall", "bay")):
        c = WALLS[int(name[-1])]
        fill(c)
        if name.startswith("bay"):
            trim = tint(c, 18)
            rect((23, 13, 105, 113), tint(c, -25))
            rect((25, 11, 103, 109), trim)
            rect((31, 17, 97, 105), (99, 92, 75))
            panes((35, 20, 93, 103), 2, 3)
            for x in (34, 63, 93):
                line((x, 19, x, 103), (205, 195, 166), 2)
            line((34, 49, 93, 49), (195, 180, 151), 3)
            rect((20, 9, 108, 13), trim)
            rect((20, 107, 109, 113), trim)
            line((22, 114, 107, 114), tint(c, -35), 2)
    elif name == "door":
        fill((203, 182, 150))
        rect((21, 4, 107, 127), (92, 76, 59))
        rect((27, 9, 101, 127), (93, 69, 49))
        panes((30, 12, 98, 45), 4, 1)
        for x in [32, 66]:
            for y in [52, 94]:
                rect(
                    (x, y, x + 28, min(125, y + 33)),
                    (68, 49, 38),
                    outline=(135, 105, 74),
                    width=2,
                )
        d.ellipse((57, 82, 60, 86), fill=(187, 153, 86))
    elif name == "shop":
        fill((172, 147, 113))
        rect((7, 11, 121, 100), (77, 89, 78))
        panes((12, 18, 85, 91), 3, 2)
        panes((91, 18, 116, 83), 1, 2)
        rect((89, 87, 118, 125), (86, 93, 73), outline=(53, 64, 52), width=3)
        rect((4, 6, 124, 18), (70, 83, 68))
        line((6, 6, 122, 6), (174, 151, 105), 2)
        for x in range(36, 94, 7):
            rect((x, 10, x + 3, 13), (194, 173, 125))
        rect((10, 103, 80, 122), (126, 118, 95))
    elif name == "clock":
        fill((155, 99, 72))
        for r, c in [
            (59, (120, 100, 80)),
            (53, (217, 199, 151)),
            (46, (62, 74, 65)),
            (40, (223, 213, 177)),
        ]:
            d.ellipse((64 - r, 64 - r, 64 + r, 64 + r), fill=c)
        for j in range(12):
            a = 2 * pi * j / 12
            line(
                (
                    64 + 33 * sin(a),
                    64 - 33 * cos(a),
                    64 + 39 * sin(a),
                    64 - 39 * cos(a),
                ),
                (69, 75, 66),
                2,
            )
        line((46, 49, 64, 64, 78, 38), (54, 61, 54), 4)
    elif name == "crates":
        fill((93, 71, 51))
        for j in range(6):
            rect(
                (3, j * 22, 125, j * 22 + 18),
                tint((142, 109, 71), rng.randrange(-9, 9)),
            )
        line((5, 4, 121, 120), (168, 136, 91), 9)
        rect((0, 0, 7, 127), (153, 119, 81))
        rect((120, 0, 127, 127), (153, 119, 81))
    elif name == "glass":
        panes((0, 0, 128, 128), 4, 3)
    elif name == "metal":
        fill((101, 106, 106))
        for x in range(0, 128, 13):
            line((x, 0, x, 127), (71, 75, 76), 2)
            line((x + 2, 0, x + 2, 127), (124, 127, 126))
    elif name == "cloth":
        fill((68, 100, 68))
        for x in range(0, 128, 16):
            rect((x, 0, x + 3, 127), (86, 120, 77))
            line((x + 4, 0, x + 4, 127), (98, 125, 84))
    elif name == "wood":
        fill((108, 82, 60))
        for j in range(12):
            y = j * 11
            line((0, y, 127, y), (70, 61, 49))
            line((0, y + 1, 127, y + 1), (135, 112, 83))
    elif name in ("brass", "cream", "grass", "black", "red", "green", "gold"):
        fill(
            {
                "brass": (161, 132, 76),
                "cream": (210, 194, 162),
                "grass": (117, 132, 82),
                "black": (58, 59, 53),
                "red": (192, 79, 51),
                "green": (109, 136, 72),
                "gold": (217, 164, 67),
            }[name]
        )
    if name.startswith("front") or name in ("arcade", "workshop"):
        is_house = name.startswith("front")
        panel = (
            "bay" + name[-1]
            if is_house
            else {"arcade": "brickarch", "workshop": "factoryarch", "windows": "glass"}[
                name
            ]
        )
        rows, cols = (3, 4) if is_house else (1, 4)
        source = paint(panel, NAMES.index(panel)).resize(
            (128 // cols, 128 // rows), Image.Resampling.LANCZOS
        )
        for row in range(rows):
            for col in range(cols):
                im.paste(source, (128 * col // cols, 128 * row // rows))
    raw = np.asarray(im).astype(float)
    gen = np.random.default_rng(index + 2)
    grain = gen.normal(0, 3.5, (SIZE, SIZE))
    low = Image.fromarray(gen.integers(70, 180, (10, 10), dtype=np.uint8)).resize(
        (SIZE, SIZE), Image.Resampling.BICUBIC
    )
    noise = (grain + (np.asarray(low).astype(float) - 128) * 0.13)[:, :, None]
    return Image.fromarray(np.clip(raw + noise, 0, 255).astype(np.uint8))


if __name__ == "__main__":
    # A four-pixel gutter around every panel keeps mip levels free of neighbouring colours.
    atlas = Image.new("RGB", (8 * 136, 5 * 136))
    for i, name in enumerate(NAMES):
        tile = np.asarray(paint(name, i))
        atlas.paste(
            Image.fromarray(np.pad(tile, ((4, 4), (4, 4), (0, 0)), mode="edge")),
            ((i % 8) * 136, (i // 8) * 136),
        )
    atlas.save(Path(__file__).with_name("architecture.png"))
