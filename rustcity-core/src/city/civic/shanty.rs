use super::*;

impl City {
    fn grow_shanty(&mut self) -> io::Result<()> {
        let x = self.decisions.c_modulo(self.side)?;
        let y = self.decisions.c_modulo(self.side)?;
        let cluster = self.stats.group_count[BuildingKind::Shanty.id() as usize] > 0
            && self.decisions.c_modulo(SHANTY_NEW_CLUSTER_ODDS)? != 0;
        let kind = if cluster {
            BuildingKind::Shanty
        } else {
            BuildingKind::Market
        };
        let Some(anchor) = self.find_building(Point { x, y }, kind) else {
            return Ok(());
        };
        let size = i32::from(BuildingKind::Shanty.metadata().size);
        let vacant = self.find_bare_area(anchor, size);
        if let Some(origin) = vacant {
            let building = self.create_building(BuildingKind::Shanty, self.tech_level)?;
            self.place_building(origin, size, building)?;
            self.stats.group_count[BuildingKind::Shanty.id() as usize] += 1;
            self.recent_point = origin;
            for y in origin.y..origin.y + size {
                for x in origin.x..origin.x + size {
                    let cell = &mut self.tiles[(y * self.side + x) as usize];
                    cell.ground = Terrain::Desert;
                    cell.frame = 0;
                }
            }
        }
        Ok(())
    }

    pub(in crate::city) fn adjust_shanty_population(&mut self) -> io::Result<()> {
        let count = self.stats.group_count[BuildingKind::Shanty.id() as usize] as i32;
        let previous = self.people_pool;
        self.people_pool = (self.people_pool - SHANTY_MONTHLY_DEATHS * count).max(0);
        self.stats.population.deaths_m += previous - self.people_pool;
        let commune_capacity =
            COMMUNE_CAPACITY * self.stats.group_count[BuildingKind::Commune.id() as usize] as i32;
        let needed =
            ((self.people_pool - commune_capacity - SHANTY_POOL_MIN) / SHANTY_CAPACITY).max(0);
        if needed > count {
            for _ in 0..1 + (needed - count) / SHANTY_CHANGE_DIVISOR {
                self.grow_shanty()?;
            }
        } else if count > 0 && needed < count - 1 {
            for _ in 0..1 + (count - needed) / SHANTY_CHANGE_DIVISOR {
                // The reference constructs MapPoint with right-to-left argument evaluation.
                let y = self.decisions.c_modulo(self.side)?;
                let x = self.decisions.c_modulo(self.side)?;
                let Some(origin) = self.find_building(Point { x, y }, BuildingKind::Shanty) else {
                    break;
                };
                let id = self.buildings.at(origin).unwrap();
                self.remove_and_restore_site(id, origin);
                self.place_fires(
                    origin,
                    i32::from(BuildingKind::Shanty.metadata().size),
                    SHANTY_DEMOLITION_FIRE_AGE,
                )?;
            }
        }
        Ok(())
    }
}
