use super::*;
use crate::rules::generation::vegetation::*;

impl City {
    pub(super) fn vegetation(&mut self, aridity: i32, step: i32, trees: bool) -> io::Result<()> {
        let n = self.side;
        let mut distance = vec![0i32; self.tiles.len()];
        let mut water = vec![0; self.tiles.len()];
        let mut queue = VecDeque::new();
        for y in 1..n - 1 {
            for x in 1..n - 1 {
                let i = self.cell(x, y);
                let t = self.tiles[i];
                if t.ground == Terrain::Water {
                    water[i] = t.water_alt;
                    queue.push_back((x, y));
                } else {
                    distance[i] = INITIAL_WATER_DISTANCE;
                    water[i] =
                        GROUNDWATER_HEIGHT_NUMERATOR * t.altitude / GROUNDWATER_HEIGHT_DENOMINATOR;
                }
            }
        }
        while let Some((x, y)) = queue.pop_front() {
            let i = self.cell(x, y);
            let d = distance[i] + 1;
            for (sx, sy) in [(x - 1, y), (x, y - 1), (x + 1, y), (x, y + 1)] {
                if !(1..n - 1).contains(&sx) || !(1..n - 1).contains(&sy) {
                    continue;
                }
                let j = self.cell(sx, sy);
                let score = |dist: i32, level: i32| {
                    dist * dist / DISTANCE_SQUARED_DIVISOR
                        + 1
                        + aridity
                        + (self.tiles[j].altitude - level) * ALTITUDE_ARIDITY_FACTOR / step
                };
                if score(d, water[i]) < score(distance[j], water[j]) {
                    distance[j] = d;
                    water[j] = water[i];
                    queue.push_back((sx, sy));
                }
            }
        }
        for (i, t) in self.tiles.iter_mut().enumerate() {
            if t.river || t.ground == Terrain::Water {
                continue;
            }
            let d2 = distance[i] * distance[i];
            let local = if aridity > 0 && d2 < NEAR_RIVER_DISTANCE_SQUARED {
                aridity / RIVER_ARIDITY_DENOMINATOR
            } else if aridity > 0 && d2 < OUTER_RIVER_DISTANCE_SQUARED {
                aridity * OUTER_RIVER_ARIDITY_NUMERATOR / RIVER_ARIDITY_DENOMINATOR
            } else {
                aridity
            };
            let r = self.decisions.c_modulo(d2 / DISTANCE_SQUARED_DIVISOR + 1)?
                + local
                + (t.altitude - water[i]) * ALTITUDE_ARIDITY_FACTOR / step;
            let z = ZONES.iter().find(|z| r >= z.aridity).unwrap();
            let roll = self.decisions.c_modulo(z.outcomes)?;
            t.ground = if roll <= z.water {
                Terrain::Water
            } else if roll <= z.desert {
                Terrain::Desert
            } else if !trees || roll <= z.grass {
                Terrain::Grass
            } else if roll <= z.tree {
                Terrain::Tree
            } else if roll <= z.tree2 {
                Terrain::Tree2
            } else {
                Terrain::Tree3
            };
            t.frame = 0;
            t.groundwater = t.ground != Terrain::Desert;
        }
        Ok(())
    }
}
