use super::*;

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
                let near_water = (y + 2..y + 23)
                    .any(|sy| (x + 2..x + 23).any(|sx| self.tiles[self.cell(sx, sy)].river));
                if near_water {
                    break (x, y, true);
                }
                tries -= 1;
                if tries <= 1 {
                    break (x, y, false);
                }
            };
            let inside = (y + 4..y + 22)
                .any(|sy| (x + 4..x + 22).any(|sx| self.tiles[self.cell(sx, sy)].river));
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
        for y in 5..21 {
            for x in 6..21 {
                let i = self.cell(left + x, top + y);
                if self.tiles[i].ground == Terrain::Water {
                    self.tiles[i].ground = Terrain::Grass;
                    self.tiles[i].frame = 0;
                }
            }
        }
        use BuildingKind::*;
        for left_x in [6, 17] {
            for y in 5..9 {
                for x in left_x..left_x + 4 {
                    if self.decisions.c_random()? > i32::MAX / 2 {
                        let i = self.cell(left + x, top + y);
                        self.tiles[i].groundwater = true;
                    }
                }
            }
            self.village_build(OrganicFarm, p(left_x, 5))?;
        }
        self.village_build(ResidenceMediumLow, p(10, 6))?;
        self.village_build(Pottery, p(9, 9))?;
        let well = self.cell(left + 16, top + 9);
        self.tiles[well].groundwater = true;
        self.village_build(Waterwell, p(16, 9))?;
        self.village_build(ResidenceMediumLow, p(14, 6))?;
        self.village_build(Market, p(14, 9))?;
        for q in (2..23)
            .map(|x| p(x, 11))
            .chain((2..11).map(|y| p(13, y)))
            .chain((12..23).map(|y| p(15, y)))
        {
            self.village_build(self.actual_kind(Track, q), q)?;
        }
        for x in [6, 11, 16] {
            for y in [12, 17] {
                self.village_build(Commune, p(x, y))?;
            }
        }
        Ok(())
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
