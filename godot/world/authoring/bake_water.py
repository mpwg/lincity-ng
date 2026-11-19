"""Bake tileable river wave slopes from a seeded, directional wave spectrum."""
from pathlib import Path

import numpy as np
from PIL import Image

size = 256
rng = np.random.default_rng(20261119)
frequency = np.fft.fftfreq(size) * size
kx, kz = np.meshgrid(frequency, frequency)
k = np.hypot(kx, kz)
direction = (0.25 * kx + 0.968 * kz) / np.maximum(k, 1)
spectrum = (0.15 + 0.85 * direction**4) * np.exp(-(k / 73)**2)
spectrum *= (1 - np.exp(-(k / 7)**2)) / np.maximum(k, 1)**1.7
height = np.fft.fft2(rng.normal(size=(size, size))) * spectrum
slopes = [np.fft.ifft2(1j * axis * height).real for axis in (kx, kz)]
scale = 3.2 * np.std(slopes)
pixels = np.uint8(np.clip(127.5 + 127.5 * np.stack(slopes, axis=-1) / scale, 0, 255))
rgb = np.dstack((pixels, np.full((size, size), 128, dtype=np.uint8)))
Image.fromarray(rgb).save(Path(__file__).resolve().parents[1] / "river_slopes.png")
