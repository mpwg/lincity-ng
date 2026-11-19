"""Bake a grayscale leaf spray for the alpha-tested outer canopy of broadleaf trees."""
import math
import random
from pathlib import Path

from PIL import Image, ImageDraw

rng = random.Random(23531)
scale = 2
image = Image.new("RGBA", (256 * scale, 256 * scale))
draw = ImageDraw.Draw(image)


def project(points):
    return [(round(x * scale), round(y * scale)) for x, y in points]


def leaf(x, y, angle, length, shade):
    forward = (math.cos(angle) * length, math.sin(angle) * length)
    across = (-forward[1] * 0.28, forward[0] * 0.28)
    tip = (x + forward[0], y + forward[1])
    for sign, light in ((-1, shade), (1, min(255, shade + 22))):
        edge = (x + forward[0] * 0.48 + sign * across[0],
                y + forward[1] * 0.48 + sign * across[1])
        draw.polygon(project([(x, y), edge, tip]), fill=(light, light, light, 255))


def shoot(x, y, length, angle, depth):
    end = (x + math.cos(angle) * length, y + math.sin(angle) * length)
    draw.line(project([(x, y), end]), fill=(85, 85, 85, 255), width=scale * max(1, depth))
    for i in range(1, 6):
        t = i / 6
        px = x + (end[0] - x) * t
        py = y + (end[1] - y) * t
        direction = angle + (1 if i % 2 else -1) * rng.uniform(0.47, 1.12)
        if depth:
            shoot(px, py, length * rng.uniform(0.42, 0.66), direction, depth - 1)
        else:
            leaf(px, py, direction, rng.uniform(5.2, 13.5), rng.randrange(122, 230))
    leaf(*end, angle, 9.2, 208)


for start in [-2.33, -1.92, -1.59, -1.24, -.87]:
    shoot(128 + rng.uniform(-5, 5), 228, rng.uniform(100, 123), start, 2)

path = Path(__file__).resolve().parents[2] / "godot/art/city/leaf_spray.png"
image.resize((256, 256), Image.Resampling.LANCZOS).save(path)
