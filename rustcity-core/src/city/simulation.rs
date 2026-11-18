use super::*;
use crate::Commodity;
use crate::rules::environment::*;
use crate::rules::{DAYS_PER_MONTH, DAYS_PER_YEAR, MAX_TECH_LEVEL};

impl City {
    /// Quantity stored at the building covering a point.
    pub fn stock(&self, point: Point, commodity: Commodity) -> Option<i32> {
        self.buildings
            .get(self.buildings.at(point)?)
            .map(|(_, b)| b.stock[commodity as usize])
    }

    /// Advances the city by one simulation day.
    pub fn tick(&mut self) -> io::Result<()> {
        self.day += 1;
        let day = self.day;
        self.stats.daily();
        if day % DAYS_PER_MONTH == 0 {
            self.stats.monthly();
        }
        if day % DAYS_PER_YEAR == 0 {
            self.stats.yearly();
        }
        for id in self.visit_order(VisitPhase::Daily, day as u64)? {
            let Some((point, _)) = self.buildings.get(id) else {
                self.people_pool += self.displaced.remove(&id).unwrap_or_default();
                self.participants.retain(|_, bid| *bid != id);
                continue;
            };
            self.trade_at(id)?;
            if self.buildings.get(id).unwrap().1.kind == BuildingKind::RocketPad {
                self.update_rocket(id, point)?;
                continue;
            }
            if self.buildings.get(id).unwrap().1.kind == BuildingKind::Market {
                self.update_market(id, point)?;
                continue;
            }
            if self.update_civic(id, point)? {
                continue;
            }
            if self.buildings.get(id).unwrap().1.kind == BuildingKind::Port {
                self.update_port(id);
                continue;
            }
            let b = self.buildings.get_mut(id).expect("visited live building");
            let pollution = &mut self.tiles[(point.y * self.side + point.x) as usize].pollution;
            match &mut b.operation {
                Operation::Commune(_) => self.update_commune(id, point)?,
                Operation::Farm(farm) => {
                    farm.update(day, b.installed_tech, &mut b.stock, &mut b.production)
                }
                Operation::Transport(_) => self.update_transport(id, point)?,
                Operation::Industry(_) => self.update_industry(id, point, day as u32),
                Operation::Workshop(shop) => {
                    let before = b.stock;
                    *pollution += shop.update(b.kind, day, &mut b.stock);
                    if b.kind == BuildingKind::Pottery && day % DAYS_PER_MONTH == 0 {
                        b.production.close_month();
                    }
                    for c in Commodity::ALL {
                        let i = c as usize;
                        b.production.current[i] += b.stock[i] - before[i];
                    }
                    if b.kind != BuildingKind::Pottery && day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1
                    {
                        b.production.close_month();
                    }
                }
                Operation::Coal(plant) => {
                    plant.update(day as u32, &mut b.stock, &mut b.production, pollution)
                }
                Operation::Wind(wind) => {
                    if wind.update(
                        day as u32,
                        &mut b.stock,
                        &mut b.production,
                        &mut self.wind_expenses,
                        &mut self.balance,
                        &mut self.stats.expenses.windmill.acc,
                    ) {
                        self.events.push_back(CityEvent::OutOfMoney);
                    }
                }
                Operation::Line(line) => line.update(day as u32, &mut b.stock, &mut b.production),
                Operation::Solar(plant) => {
                    plant.labor = b.stock[Commodity::Labor as usize];
                    plant.high_voltage.amount = b.stock[Commodity::HighVoltage as usize];
                    let before_labor = b.stock[Commodity::Labor as usize];
                    let before_high_voltage = b.stock[Commodity::HighVoltage as usize];
                    plant.update(day as u32);
                    b.production
                        .change(&mut b.stock, Commodity::Labor, plant.labor - before_labor);
                    b.production.change(
                        &mut b.stock,
                        Commodity::HighVoltage,
                        plant.high_voltage.amount - before_high_voltage,
                    );
                    if day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1 {
                        b.production.close_month();
                    }
                }
                Operation::Station(station) => {
                    station.high_voltage.amount = b.stock[Commodity::HighVoltage as usize];
                    station.low_voltage = b.stock[Commodity::LowVoltage as usize];
                    let before_high_voltage = b.stock[Commodity::HighVoltage as usize];
                    let before_low_voltage = b.stock[Commodity::LowVoltage as usize];
                    station.update(day as u32);
                    b.production.change(
                        &mut b.stock,
                        Commodity::HighVoltage,
                        station.high_voltage.amount - before_high_voltage,
                    );
                    b.production.change(
                        &mut b.stock,
                        Commodity::LowVoltage,
                        station.low_voltage - before_low_voltage,
                    );
                    if day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1 {
                        b.production.close_month();
                    }
                }
                Operation::Rocket(_)
                | Operation::Market(_)
                | Operation::Port(_)
                | Operation::Civic(_)
                | Operation::Other => {}
            }
        }
        self.daily_ecology()?;
        if day % DAYS_PER_YEAR == 0 {
            self.check_sustainability();
        }
        if day % POLLUTION_INTERVAL == POLLUTION_DAY {
            self.diffuse_pollution()?;
        }
        self.random_fire()?;
        if day % civic::COVER_PERIOD == civic::COVER_DAY {
            self.finalize_coverage();
        }
        if day % DAYS_PER_MONTH == civic::SHANTY_ADJUST_DAY
            && self.tech_level > BuildingKind::Health.metadata().required_tech
        {
            self.adjust_shanty_population()?;
        }
        if day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1 {
            self.scan_pollution();
            if self.people_pool > MIGRATION_THRESHOLD
                && self.decisions.c_modulo(MIGRATION_DRAW_RANGE)? < self.people_pool
            {
                self.people_pool -= MIGRANTS_PER_DEPARTURE;
            }
            self.people_pool = self.people_pool.max(0);
            self.stats.population.population_m += self.people_pool * DAYS_PER_MONTH;
            let population = f64::from(self.stats.population.population_m.acc);
            let mut tech = self.tech_level;
            if tech > TECH_LOSS_START {
                tech -= (f64::from(tech)
                    * (1.0 / TECH_LOSS)
                    * (1.0 + population * NORMAL_POPULATION_FACTOR)) as i32;
            } else {
                tech += UNAIDED_TECH_GAIN;
            }
            if tech > MAX_TECH_LEVEL {
                tech -= (f64::from(tech - MAX_TECH_LEVEL)
                    * (1.0 / TECH_LOSS)
                    * (1.0 + population * EXCESS_POPULATION_FACTOR)) as i32;
            }
            self.tech_level = tech;
            self.stats.tech_level = tech;
            self.stats.highest_tech_level = self.stats.highest_tech_level.max(tech);
            self.pay_monthly_bills();
            self.stats.total_money = self.balance;
            for (_, building) in self.buildings.iter() {
                for c in Commodity::ALL {
                    self.stats.inventory[c as usize] += crate::stats::Inventory {
                        amount: building.stock[c as usize],
                        capacity: building.kind.metadata().rule(c).max_load,
                    };
                }
            }
            if self.stats.population.population_m.acc == 0 && !self.game_end {
                self.game_end = true;
                self.events.push_back(CityEvent::NoPeopleLeft(day));
            }
        }
        if day % DAYS_PER_YEAR == DAYS_PER_YEAR - 1 {
            self.settle_year()?;
        }
        Ok(())
    }

    /// Advances visual animation at the host's millisecond timestamp.
    pub fn animate(&mut self, milliseconds: u64) -> io::Result<()> {
        for id in self.visit_order(VisitPhase::Animation, milliseconds)? {
            let Some(b) = self.buildings.get_mut(id) else {
                continue;
            };
            match &mut b.operation {
                Operation::Rocket(r) => r.animate(milliseconds),
                Operation::Transport(t) => t.animate(milliseconds, &mut self.decisions)?,
                Operation::Civic(c) => c.animate(b.kind, milliseconds, &mut self.decisions)?,
                Operation::Commune(c) => c.animate(milliseconds, &mut self.decisions)?,
                Operation::Market(m) => m.animate(milliseconds, &mut self.decisions)?,
                Operation::Farm(farm) => farm.animate(self.day, &mut self.decisions)?,
                Operation::Industry(factory) => {
                    factory.animate(milliseconds, &mut self.decisions)?
                }
                Operation::Workshop(shop) => {
                    shop.animate(b.kind, milliseconds);
                }
                Operation::Coal(plant) => {
                    plant.animate(milliseconds, b.stock[Commodity::Coal as usize], || {
                        self.decisions.c_random().map(|n| n as u32)
                    })?
                }
                Operation::Wind(wind) => wind.animate(milliseconds, &b.stock),
                Operation::Line(line) => line.animate(milliseconds),
                _ => {}
            }
            b.update_sprite();
        }
        self.animate_vehicles(milliseconds)
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
        city.place_building(home, 2, smith).unwrap();
        for day in 1..=5 {
            city.animate(day as u64 * 1000).unwrap();
            city.tick().unwrap();
            assert_eq!(city.stock(home, Commodity::Goods), Some(7 + 50 * day));
            assert_eq!(city.stock(home, Commodity::Labor), Some(175 - 35 * day));
        }
    }
}
