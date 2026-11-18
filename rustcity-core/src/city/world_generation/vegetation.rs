use super::*;

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
                    distance[i] = 50;
                    water[i] = 3 * t.altitude / 4;
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
                    dist * dist / 5 + 1 + aridity + (self.tiles[j].altitude - level) * 50 / step
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
            let local = if aridity > 0 && d2 < 5 {
                aridity / 3
            } else if aridity > 0 && d2 < 17 {
                aridity * 2 / 3
            } else {
                aridity
            };
            let r =
                self.decisions.c_modulo(d2 / 5 + 1)? + local + (t.altitude - water[i]) * 50 / step;
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
