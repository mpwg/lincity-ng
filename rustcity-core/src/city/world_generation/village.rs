use super::*;
use crate::rules::generation::village::*;

impl City {
    pub(super) fn starting_village(&mut self) -> io::Result<()> {
        let mut tries = VILLAGE_LOCATION_TRIES;
        let (left, top) = loop {
            let (x, y, near_water) = loop {
                let x = self
                    .decisions
                    .c_modulo(self.side - (MIN_VILLAGE_SIDE - 1))?;
                let y = self
                    .decisions
                    .c_modulo(self.side - (MIN_VILLAGE_SIDE - 1))?;
                let near_water = self.village_has_river(x, y, RIVER_SEARCH);
                if near_water {
                    break (x, y, true);
                }
                tries -= 1;
                if tries <= 1 {
                    break (x, y, false);
                }
            };
            let inside = self.village_has_river(x, y, RIVER_FREE_INTERIOR);
            if near_water && !inside {
                break (x, y);
            }
            tries -= 1;
            if tries <= 1 {
                break (x, y);
            }
        };
        let p = |x, y| Point {
            x: left + x,
            y: top + y,
        };
        for y in DRAIN_ROWS {
            for x in DRAIN_COLUMNS {
                let i = self.cell(left + x, top + y);
                if self.tiles[i].ground == Terrain::Water {
                    self.tiles[i].ground = Terrain::Grass;
                    self.tiles[i].frame = 0;
                }
            }
        }
        use BuildingKind::*;
        let farm_size = i32::from(OrganicFarm.metadata().size);
        for left_x in FARM_COLUMNS {
            for y in FARM_ROW..FARM_ROW + farm_size {
                for x in left_x..left_x + farm_size {
                    if self.decisions.c_random()? > FARM_WATER_THRESHOLD {
                        let i = self.cell(left + x, top + y);
                        self.tiles[i].groundwater = true;
                    }
                }
            }
            self.village_build(OrganicFarm, p(left_x, FARM_ROW))?;
        }
        for (kind, x, y) in BUILDINGS {
            let site = p(x, y);
            if kind == Waterwell {
                let well = self.cell(site.x, site.y);
                self.tiles[well].groundwater = true;
            }
            self.village_build(kind, site)?;
        }
        let (cross_y, cross_columns) = CROSS_TRACK;
        let (north_x, north_rows) = NORTH_TRACK;
        let (south_x, south_rows) = SOUTH_TRACK;
        for q in cross_columns
            .map(|x| p(x, cross_y))
            .chain(north_rows.map(|y| p(north_x, y)))
            .chain(south_rows.map(|y| p(south_x, y)))
        {
            self.village_build(self.actual_kind(Track, q), q)?;
        }
        for x in COMMUNE_COLUMNS {
            for y in COMMUNE_ROWS {
                self.village_build(Commune, p(x, y))?;
            }
        }
        Ok(())
    }

    fn village_has_river(&self, x: i32, y: i32, offsets: std::ops::Range<i32>) -> bool {
        offsets.clone().any(|dy| {
            offsets
                .clone()
                .any(|dx| self.tiles[self.cell(x + dx, y + dy)].river)
        })
    }

    fn village_build(&mut self, kind: BuildingKind, point: Point) -> io::Result<()> {
        let building = self.create_building(kind, self.tech_level)?;
        let transparent = building.transparent;
        let size = i32::from(kind.metadata().size);
        let id = self.place_building(point, size, building)?;
        if kind == BuildingKind::ResidenceMediumLow
            && let Operation::Civic(c) = &mut self.buildings.get_mut(id).unwrap().operation
        {
            c.set_starter_population();
        }
        self.stats.group_count[kind.id() as usize] += 1;
        self.recent_point = point;
        for y in point.y..point.y + size {
            for x in point.x..point.x + size {
                let i = self.cell(x, y);
                let tile = &mut self.tiles[i];
                if tile.ground != Terrain::Water && (!transparent || tile.ground != Terrain::Desert)
                {
                    tile.ground = if transparent {
                        Terrain::Grass
                    } else {
                        Terrain::Desert
                    };
                    tile.frame = 0;
                }
            }
        }
        Ok(())
    }
}
