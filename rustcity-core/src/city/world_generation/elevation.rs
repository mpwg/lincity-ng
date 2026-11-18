use super::*;

impl City {
    pub(super) fn fractal_landscape(&mut self, relief: i32, aridity: i32) -> io::Result<i32> {
        let n = self.side as usize;
        let size = (n + SMOOTHING_RADIUS + 1).next_power_of_two();
        let stride = size + 1;
        let at = |x: usize, y: usize| x + y * stride;
        let shift = (size - n) / 2;
        let mut height = vec![0f32; stride * stride];
        for (x, y) in [(0, 0), (0, size), (size, 0), (size, size)] {
            height[at(x, y)] = ((self.decisions.c_modulo(HEIGHT_STEP)?
                + self.decisions.c_modulo(HEIGHT_STEP)?)
                * relief) as f32;
        }
        let sea_sum =
            height[at(0, 0)] + height[at(0, size)] + height[at(size, 0)] + height[at(size, size)];
        let mut sea = (f64::from(sea_sum) / 5.3) as i32;
        for level in 1..=size.ilog2() {
            let squares = 1 << (level - 1);
            let edge = size / squares;
            let half = edge / 2;
            let scale = f64::from(ROUGHNESS).powf(f64::from(level));
            let mut subdivide =
                |map: &mut [f32], target: usize, corners: [usize; 4]| -> io::Result<()> {
                    if map[target] == 0. {
                        let [a, b, c, d] = corners.map(|i| map[i]);
                        let mean = (a + b + c + d) / 4.;
                        let offset =
                            (self.decisions.c_modulo(HEIGHT_STEP)? - HEIGHT_STEP / 2) * relief;
                        map[target] = (f64::from(mean) + f64::from(offset) * scale) as f32;
                    }
                    Ok(())
                };
            for l in 0..squares {
                for m in 0..squares {
                    let x = l * edge;
                    let y = m * edge;
                    subdivide(
                        &mut height,
                        at(x + half, y + half),
                        [
                            at(x, y),
                            at(x, y + edge),
                            at(x + edge, y),
                            at(x + edge, y + edge),
                        ],
                    )?;
                }
            }
            for l in 0..squares {
                for m in 0..squares {
                    let x = l * edge;
                    let y = m * edge;
                    let nw = at(x, y);
                    let sw = at(x, y + edge);
                    let ne = at(x + edge, y);
                    let se = at(x + edge, y + edge);
                    let center = at(x + half, y + half);
                    if m == 0 {
                        subdivide(
                            &mut height,
                            at(x + half, y),
                            [nw, ne, center, at(x + half, size - half)],
                        )?;
                    }
                    subdivide(
                        &mut height,
                        at(x + edge, y + half),
                        [ne, se, center, at((x + half + edge) % size, y + half)],
                    )?;
                    subdivide(
                        &mut height,
                        at(x + half, y + edge),
                        [sw, se, center, at(x + half, (y + half + edge) % size)],
                    )?;
                    if l == 0 {
                        subdivide(
                            &mut height,
                            at(x, y + half),
                            [nw, sw, center, at(size - half, y + half)],
                        )?;
                    }
                }
            }
        }
        let r = SMOOTHING_RADIUS as isize;
        let mut kernel = Vec::new();
        let mut norm = 0.;
        let inv_variance = 1. / (2. * GAUSSIAN_SIGMA * GAUSSIAN_SIGMA);
        for x in -r..=r {
            for y in -r..=r {
                let weight = f64::from(-((x * x + y * y) as f32) * inv_variance).exp() as f32;
                kernel.push((x, y, weight));
                norm += weight;
            }
        }
        let norm = (1. / f64::from(norm)) as f32;
        let mut smooth = vec![0f32; size * size];
        // The second pass keeps the first-pass accumulator, matching LinCity's elevation scale.
        for _ in 0..2 {
            for x in SMOOTHING_RADIUS..size - SMOOTHING_RADIUS {
                for y in SMOOTHING_RADIUS..size - SMOOTHING_RADIUS {
                    let sum = &mut smooth[x + y * size];
                    for &(dx, dy, weight) in &kernel {
                        *sum += height[at(x.wrapping_add_signed(dx), y.wrapping_add_signed(dy))]
                            * weight;
                    }
                }
            }
            for x in SMOOTHING_RADIUS..size - SMOOTHING_RADIUS {
                for y in SMOOTHING_RADIUS..size - SMOOTHING_RADIUS {
                    height[at(x, y)] = smooth[x + y * size] * norm;
                }
            }
        }
        let mut north = 0.;
        let mut south = 0.;
        let mut west = 0.;
        let mut east = 0.;
        for i in 0..n {
            north += height[at(shift + i, shift)];
            south += height[at(shift + i, shift + n)];
            west += height[at(shift, shift + i)];
            east += height[at(shift + n, shift + i)];
        }
        let mut low = f32::INFINITY;
        for y in 0..n {
            for x in 0..n {
                let sx = if west < east {
                    size - 1 - shift - x
                } else {
                    shift + x
                };
                let sy = if north < south {
                    size - 1 - shift - y
                } else {
                    shift + y
                };
                let value = height[at(sx, sy)];
                low = low.min(value);
                self.tiles[x + y * n].altitude = value as i32;
            }
        }
        let low = low as i32;
        let mut high = 0;
        for t in &mut self.tiles {
            t.altitude = t.altitude - low + 1;
            high = high.max(t.altitude);
        }
        self.lakes(aridity)?;
        sea = (sea - low).min(high / 3);
        for y in 1..self.side - 1 {
            for x in 1..self.side - 1 {
                let i = self.cell(x, y);
                if self.tiles[i].altitude < sea {
                    self.tiles[i].altitude = sea;
                    self.wet_tile(x, y);
                }
            }
        }
        let end = self.side - 1;
        for i in 1..=end {
            for (x, y, ix, iy) in [
                (i, 0, i, 1),
                (i, end, i, end - 1),
                (end, i, end - 1, i),
                (0, i, 1, i),
            ] {
                if self.tiles[self.cell(ix, iy)].river {
                    self.wet_tile(x, y);
                }
            }
        }
        Ok(high / 10)
    }
}
