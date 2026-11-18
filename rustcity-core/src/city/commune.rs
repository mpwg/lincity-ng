//! Forest communes grow fuel, reclaim ore, and turn unused wet plots into parks.
use super::*;
use crate::Commodity::*;
use crate::rules::{DAYS_PER_MONTH, commune::*};

#[derive(Default)]
pub(super) struct Commune {
    pub(super) wet_tiles: i32,
    monthly_output: i32,
    last_output: i32,
    idle_months: i32,
    animate_enabled: bool,
    steel_made: bool,
    next_frame: u64,
    frame: u8,
}

impl Commune {
    pub(super) fn animate(&mut self, time: u64, decisions: &mut Decisions) -> io::Result<()> {
        if self.animate_enabled && time >= self.next_frame {
            let ms =
                FRAME_INTERVAL_MS - FRAME_JITTER_MS / 2 + decisions.c_modulo(FRAME_JITTER_MS)?;
            let quantum = crate::rules::civic::ANIMATION_QUANTUM_MS;
            self.next_frame = time + ((ms + quantum / 2) / quantum * quantum - quantum / 2) as u64;
            self.animate_enabled = false;
            let rotation = self.frame % FRAMES_PER_ACTIVITY + 1;
            self.frame = rotation
                + if self.steel_made {
                    FRAMES_PER_ACTIVITY
                } else {
                    0
                };
            self.steel_made = false;
        } else if self.monthly_output == 0 && self.last_output == 0 {
            self.frame = 0;
        }
        Ok(())
    }
}

impl City {
    pub(super) fn update_commune(&mut self, id: BuildingId, point: Point) -> io::Result<()> {
        let b = self.buildings.get_mut(id).expect("visited forest commune");
        let Operation::Commune(state) = &mut b.operation else {
            unreachable!()
        };
        let size = i32::from(b.kind.metadata().size);
        let area = size * size;
        let stock = &mut b.stock;
        let before = *stock;
        let irrigation = (area - state.wet_tiles) * WATER_PER_DRY_TILE;
        let coal = if stock[Water as usize] >= irrigation {
            stock[Water as usize] -= irrigation;
            COAL_BATCH
        } else if state.wet_tiles < area / FERTILITY_TIERS {
            COAL_BATCH / LOW_FERTILITY_DIVISOR
        } else if state.wet_tiles < 2 * area / FERTILITY_TIERS {
            COAL_BATCH / MEDIUM_FERTILITY_DIVISOR
        } else {
            COAL_BATCH
        };
        let cap = |commodity| BuildingKind::Commune.metadata().rule(commodity).max_load;
        let mut activity = 0;
        if coal > 0 && stock[Coal as usize] + coal <= cap(Coal) {
            stock[Coal as usize] += coal;
            activity += 1;
        }
        if stock[Ore as usize] + ORE_BATCH <= cap(Ore) {
            stock[Ore as usize] += ORE_BATCH;
            activity += 1;
        }
        if stock[Waste as usize] >= WASTE_RESERVE_BATCHES * WASTE_BATCH {
            stock[Waste as usize] -= WASTE_BATCH;
            activity += 1;
            if stock[Ore as usize] + ORE_FROM_WASTE <= cap(Ore) {
                stock[Ore as usize] += ORE_FROM_WASTE;
            }
        }
        if self.day % POLLUTION_CLEANUP_INTERVAL == 0 {
            let steel_day = self.day % STEEL_INTERVAL >= POLLUTION_CLEANUP_INTERVAL;
            for y in point.y..point.y + size {
                // Only the first column alternates. All following columns in the row are cleaned.
                for x in point.x + (y + i32::from(steel_day)) % 2..point.x + size {
                    let pollution = &mut self.tiles[(y * self.side + x) as usize].pollution;
                    if *pollution != 0 {
                        *pollution -= 1;
                    }
                }
            }
            if steel_day && stock[Steel as usize] + STEEL_BATCH <= cap(Steel) {
                stock[Steel as usize] += STEEL_BATCH;
                activity += 1;
                state.steel_made = true;
            }
        }
        for (count, (after, prior)) in b
            .production
            .current
            .iter_mut()
            .zip(stock.iter().zip(before))
        {
            *count += after - prior;
        }
        state.monthly_output += activity;
        state.animate_enabled |= activity > 0;
        if self.day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1 {
            b.production.close_month();
            state.last_output = std::mem::take(&mut state.monthly_output);
            if state.last_output > 0 {
                state.idle_months = (state.idle_months - 1).max(0);
            } else {
                state.idle_months += 1;
                if state.idle_months > ABANDONMENT_MONTHS {
                    self.remove_and_restore_site(id, point);
                    for y in point.y..point.y + size {
                        for x in point.x..point.x + size {
                            let park = Point { x, y };
                            if self.has_groundwater(park) == Some(true) {
                                let tree =
                                    self.create_building(BuildingKind::Parkland, self.tech_level)?;
                                self.place_building(park, 1, tree)?;
                                self.stats.group_count[BuildingKind::Parkland.id() as usize] += 1;
                                self.recent_point = park;
                                let tile = &mut self.tiles[(y * self.side + x) as usize];
                                if tile.ground != Terrain::Water {
                                    tile.ground = Terrain::Desert;
                                    tile.frame = 0;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn idle_forest_retires_only_after_121_unproductive_months() {
        let mut city = City::empty(32).unwrap();
        let origin = Point { x: 12, y: 12 };
        let wet = Point { x: 13, y: 12 };
        city.tiles[(wet.y * city.side + wet.x) as usize].groundwater = true;
        let mut b = Building::new(BuildingKind::Commune);
        b.stock[Coal as usize] = 60;
        b.stock[Ore as usize] = 140;
        b.stock[Steel as usize] = 40;
        let id = city.place_building(origin, 4, b).unwrap();
        city.stats.group_count[BuildingKind::Commune.id() as usize] = 1;
        for month in 0..120 {
            city.day = month * 100 + 99;
            city.update_commune(id, origin).unwrap();
        }
        assert_eq!(
            city.building_at(origin),
            Some((BuildingKind::Commune, origin))
        );
        // One productive month cancels one idle month.
        city.buildings.get_mut(id).unwrap().stock[Ore as usize] = 134;
        for month in 120..122 {
            city.day = month * 100 + 99;
            city.update_commune(id, origin).unwrap();
        }
        assert!(city.building_at(origin).is_some());
        city.day = 12299;
        city.update_commune(id, origin).unwrap();
        assert_eq!(city.building_at(origin), None);
        assert_eq!(city.building_at(wet), Some((BuildingKind::Parkland, wet)));
        assert_eq!(
            city.stats.group_count[BuildingKind::Parkland.id() as usize],
            1
        );
    }
}
