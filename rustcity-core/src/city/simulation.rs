use super::*;
use crate::Commodity;

impl City {
    /// Quantity stored at the building covering a point.
    pub fn stock(&self, point: Point, commodity: Commodity) -> Option<i32> {
        self.buildings
            .get(self.buildings.at(point)?)
            .map(|(_, b)| b.stock[commodity as usize])
    }

    /// Advances the city by one simulation day.
    pub fn tick(&mut self) {
        self.day += 1;
        let day = self.day;
        self.stats.daily();
        if day % 100 == 0 {
            self.stats.monthly();
        }
        if day % 1200 == 0 {
            self.stats.yearly();
        }
        for (point, b) in self.buildings.iter_mut() {
            let pollution = &mut self.tiles[(point.y * self.side + point.x) as usize].pollution;
            match &mut b.operation {
                Operation::Workshop(shop) => {
                    let before = b.stock;
                    *pollution += shop.update(b.kind, day, &mut b.stock);
                    if b.kind == BuildingKind::Pottery && day % 100 == 0 {
                        b.production.close_month();
                    }
                    for c in Commodity::ALL {
                        let i = c as usize;
                        b.production.current[i] += b.stock[i] - before[i];
                    }
                    if b.kind != BuildingKind::Pottery && day % 100 == 99 {
                        b.production.close_month();
                    }
                }
                Operation::Coal(plant) => {
                    plant.update(day as u32, &mut b.stock, &mut b.production, pollution)
                }
                Operation::Wind(wind) => {
                    wind.update(
                        day as u32,
                        &mut b.stock,
                        &mut b.production,
                        &mut self.wind_expenses,
                        &mut self.balance,
                        &mut self.stats.expenses.windmill.acc,
                    );
                }
                Operation::Line(line) => line.update(day as u32, &mut b.stock, &mut b.production),
                Operation::Solar(plant) => {
                    plant.labor = b.stock[Commodity::Labor as usize];
                    plant.high_voltage.amount = b.stock[Commodity::HighVoltage as usize];
                    plant.update(day as u32);
                    b.stock[Commodity::Labor as usize] = plant.labor;
                    b.stock[Commodity::HighVoltage as usize] = plant.high_voltage.amount;
                }
                Operation::Station(station) => {
                    station.high_voltage.amount = b.stock[Commodity::HighVoltage as usize];
                    station.low_voltage = b.stock[Commodity::LowVoltage as usize];
                    station.update(day as u32);
                    b.stock[Commodity::HighVoltage as usize] = station.high_voltage.amount;
                    b.stock[Commodity::LowVoltage as usize] = station.low_voltage;
                }
                Operation::Other => {}
            }
        }
        if day % 100 == 99 {
            self.stats.population.population_m += self.people_pool * 100;
            let population = f64::from(self.stats.population.population_m.acc);
            let mut tech = self.tech_level;
            if tech > 11_000 {
                tech -=
                    (f64::from(tech) / 500.0 * (1.0 + population / 100.0 / 120.0 / 300.0)) as i32;
            } else {
                tech += 2;
            }
            if tech > 1_000_000 {
                tech -= (f64::from(tech - 1_000_000) / 500.0
                    * (1.0 + population / 100.0 / 120.0 / 400.0)) as i32;
            }
            self.tech_level = tech;
            self.stats.tech_level = tech;
            self.stats.highest_tech_level = self.stats.highest_tech_level.max(tech);
            self.stats.total_money = self.balance;
            for (_, building) in self.buildings.iter() {
                for c in Commodity::ALL {
                    self.stats.inventory[c as usize] += crate::stats::Inventory {
                        amount: building.stock[c as usize],
                        capacity: building.kind.metadata().rule(c).max_load,
                    };
                }
            }
        }
    }

    /// Advances visual animation at the host's millisecond timestamp.
    pub fn animate(&mut self, milliseconds: u64) {
        for (_, b) in self.buildings.iter_mut() {
            match &mut b.operation {
                Operation::Workshop(shop) => {
                    shop.animate(b.kind, milliseconds);
                }
                Operation::Wind(wind) => wind.animate(milliseconds, &b.stock),
                Operation::Line(line) => line.animate(milliseconds),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smithy_replay_converts_stock_through_month_boundary() {
        let home = Point { x: 12, y: 12 };
        let mut city = City::empty(32).unwrap();
        city.day = 96;
        let mut smith = Building::new(BuildingKind::Blacksmith);
        for (kind, n) in [
            (Commodity::Labor, 175),
            (Commodity::Coal, 5),
            (Commodity::Steel, 5),
            (Commodity::Goods, 7),
        ] {
            smith.stock[kind as usize] = n;
        }
        city.buildings.place(home, 2, smith).unwrap();
        for day in 1..=5 {
            city.animate(day as u64 * 1000);
            city.tick();
            assert_eq!(city.stock(home, Commodity::Goods), Some(7 + 50 * day));
            assert_eq!(city.stock(home, Commodity::Labor), Some(175 - 35 * day));
        }
    }
}
