//! River connectivity, daily desert regrowth and the visible ground on neighboring tiles.
use super::*;

use crate::rules::environment::REGROWTH_CHANCE_DENOMINATOR;

impl City {
    pub(super) fn connect_river(&mut self, origin: Point) {
        fn neighbors(p: Point) -> [Point; 4] {
            [
                Point { x: p.x, y: p.y - 1 },
                Point { x: p.x, y: p.y + 1 },
                Point { x: p.x + 1, y: p.y },
                Point { x: p.x - 1, y: p.y },
            ]
        }
        let i = self.index(origin).unwrap();
        if self.tiles[i].river
            || self.tiles[i].ground != Terrain::Water
            || !neighbors(origin)
                .into_iter()
                .filter_map(|p| self.index(p))
                .any(|i| self.tiles[i].river)
        {
            return;
        }
        self.tiles[i].river = true;
        let mut queue = std::collections::VecDeque::from([origin]);
        while let Some(p) = queue.pop_front() {
            for next in neighbors(p) {
                if let Some(i) = self.index(next)
                    && self.tiles[i].ground == Terrain::Water
                    && !self.tiles[i].river
                {
                    self.tiles[i].river = true;
                    queue.push_back(next);
                }
            }
        }
    }

    pub(super) fn visible_group(&self, index: usize) -> u16 {
        let point = Point {
            x: index as i32 % self.side,
            y: index as i32 / self.side,
        };
        if let Some(id) = self.buildings.at(point)
            && let Some((origin, b)) = self.buildings.get(id)
            && !b.transparent
        {
            if b.kind == BuildingKind::Port
                && point.x == origin.x + i32::from(b.kind.metadata().size) - 1
            {
                return Terrain::Water.group();
            }
            return b.kind.id();
        }
        self.tiles[index].ground.group()
    }

    pub(super) fn daily_ecology(&mut self) -> io::Result<()> {
        for index in 0..self.tiles.len() {
            if self.tiles[index].groundwater
                && self.tiles[index].ground == Terrain::Desert
                && self.visible_group(index) == Terrain::Desert.group()
                && self.decisions.c_modulo(REGROWTH_CHANCE_DENOMINATOR)? == 1
            {
                let point = Point {
                    x: index as i32 % self.side,
                    y: index as i32 / self.side,
                };
                self.tiles[index].ground = Terrain::Grass;
                self.tiles[index].frame = 0;
                self.update_frontiers(
                    point,
                    Point {
                        x: point.x + 1,
                        y: point.y + 1,
                    },
                );
            }
        }
        Ok(())
    }
}
