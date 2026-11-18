use super::*;

impl City {
    pub(super) fn river_landscape(&mut self, relief: i32) -> io::Result<i32> {
        let n = self.side;
        let mut x = (n + self.decisions.c_modulo(n)?) / 3;
        let mut y = n - 1;
        let length = 1 + n / 18 + self.decisions.c_modulo(n / 8)?;
        for j in (1..=length).rev() {
            for dx in [-1, 0, 1] {
                self.wet_tile(x + dx, y);
            }
            if j > 2 {
                x += self.decisions.c_modulo(3)? - 1;
            }
            y -= 1;
        }
        self.tributary(x - 1, y, -1)?;
        self.tributary(x, y, 1)?;
        let mut queue = BinaryHeap::new();
        for (index, t) in self.tiles.iter().enumerate() {
            if t.river {
                queue.push(Visit {
                    index,
                    priority: 0.,
                });
            }
        }
        let mut distance = vec![(n * 2) as f32; self.tiles.len()];
        while let Some(Visit { index, priority }) = queue.pop() {
            if priority >= distance[index] {
                continue;
            }
            distance[index] = priority;
            let x = index as i32 % n;
            let y = index as i32 / n;
            for (dx, dy) in [
                (0, -1),
                (0, 1),
                (-1, 0),
                (1, 0),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ] {
                if (0..n).contains(&(x + dx)) && (0..n).contains(&(y + dy)) {
                    let mut d = priority + 1.;
                    if dx != 0 && dy != 0 {
                        d += DIAGONAL_DISTANCE_INCREMENT;
                    }
                    queue.push(Visit {
                        index: self.cell(x + dx, y + dy),
                        priority: d,
                    });
                }
            }
        }
        let mut low = i32::MAX;
        let mut high = i32::MIN;
        for (index, t) in self.tiles.iter_mut().enumerate() {
            let y = index as i32 / n;
            let d = distance[index];
            t.altitude = ((relief / 10) as f32
                * (((n - y) * (n - y) / n) as f32 + 8. * d
                    - 6.5 * d * y as f32 / n as f32
                    - 1.3 * d * d / n as f32)) as i32;
            low = low.min(t.altitude);
            high = high.max(t.altitude);
        }
        Ok((high - low) / 10)
    }

    fn tributary(&mut self, mut x: i32, mut y: i32, d: i32) -> io::Result<()> {
        let n = self.side;
        let length = self
            .decisions
            .uniform("river.segment-length", n / 6, n / 2)?;
        for j in 0..length {
            let slant = if y < n / 2 { 5. } else { 7. };
            x += (self.decisions.discrete("river.slant", &[1., 2., slant])? - 1) * d;
            if x < 4
                || x >= n - 5
                || ((j >= 5 || d == -1) && self.tiles[self.cell(x - 2, y)].river)
                || (d == -1 && x >= 10 && self.tiles[self.cell(x - 10, y)].river)
            {
                return Ok(());
            }
            self.wet_tile(x, y);
            self.wet_tile(x + 1, y);
            y -= 1;
            if y < 10 {
                break;
            }
        }
        if y > 20 {
            if x >= 6 {
                self.tributary(x, y, -1)?;
            }
            if x < n - 7 {
                self.tributary(x, y, 1)?;
            }
        }
        Ok(())
    }

    pub(super) fn lakes(&mut self, aridity: i32) -> io::Result<()> {
        let n = self.side;
        let density = 5 + self.decisions.binomial("river.lake-count", 10, 0.5)?;
        let count = i64::from(100 - aridity / 4) * i64::from(density) * i64::from(n) * i64::from(n)
            / 1_000_000;
        for _ in 0..count {
            // GCC evaluates the two constructor arguments from right to left.
            let y = self.decisions.uniform("river.start-y", 0, n - 1)?;
            let x = self.decisions.uniform("river.start-x", 0, n - 1)?;
            let start = self.cell(x, y);
            let mut queue = BinaryHeap::new();
            let mut visited = std::collections::HashSet::new();
            let mut path = Vec::new();
            let mut level = self.tiles[start].altitude - 1;
            let initial = Visit {
                index: start,
                priority: self.tiles[start].altitude as f32,
            };
            queue.push(initial);
            queue.push(initial);
            loop {
                let mut best = loop {
                    let p = queue.pop().expect("river frontier");
                    if !visited.contains(&p.index) {
                        break p;
                    }
                };
                while queue.peek().is_some_and(|p| visited.contains(&p.index)) {
                    queue.pop();
                }
                let second = queue.peek().expect("river runner-up");
                let d1 = level - self.tiles[best.index].altitude;
                let d2 = level - self.tiles[second.index].altitude;
                if d2 >= 0
                    && self.decisions.bernoulli(
                        "river.runner-up",
                        if d1 == 0 {
                            0.5
                        } else {
                            f64::from(d2) / f64::from(d1 + d2)
                        },
                    )?
                {
                    let next = queue.pop().unwrap();
                    queue.push(best);
                    best = next;
                }
                let i = best.index;
                let x = i as i32 % n;
                let y = i as i32 / n;
                if x == 0 || y == 0 || x == n - 1 || y == n - 1 || self.tiles[i].river {
                    break;
                }
                self.wet_tile(x, y);
                visited.insert(i);
                path.push(i);
                level = self.tiles[i].altitude;
                for j in [i - n as usize, i + n as usize, i + 1, i - 1] {
                    if !visited.contains(&j) {
                        queue.push(Visit {
                            index: j,
                            priority: self.tiles[j].altitude as f32,
                        });
                    }
                }
            }
            level = i32::MIN;
            for i in path.into_iter().rev() {
                let t = &mut self.tiles[i];
                if t.altitude <= level {
                    t.altitude = level;
                    t.lake = true;
                } else {
                    level = t.altitude;
                }
            }
        }
        Ok(())
    }
}
