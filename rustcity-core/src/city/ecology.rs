//! Daily regrowth on exposed desert and the ground groups visible to adjacent tiles.
use super::*;

use crate::rules::environment::REGROWTH_CHANCE_DENOMINATOR;

impl City {
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
