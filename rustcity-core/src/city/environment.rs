use super::*;
use crate::rules::{DAYS_PER_MONTH, DAYS_PER_YEAR, PERCENT, environment::*};

impl City {
    /// Current air pollution deposited at a map coordinate.
    pub fn air_pollution(&self, p: Point) -> Option<i32> {
        self.index(p).map(|i| self.tiles[i].pollution)
    }
}

impl City {
    pub(super) fn scan_pollution(&mut self) {
        let mut total = 0;
        for tile in &mut self.tiles {
            tile.diffuses = tile.pollution > POLLUTION_THRESHOLD;
            total += tile.pollution;
        }
        self.stats.total_pollution = total;
    }

    pub(super) fn diffuse_pollution(&mut self) -> io::Result<()> {
        let side = self.side as usize;
        for i in 0..self.tiles.len() {
            if !self.tiles[i].diffuses {
                continue;
            }
            let (x, y) = (i % side, i / side);
            if x == 0 || y == 0 || x + 1 == side || y + 1 == side {
                self.tiles[i].pollution /= BORDER_DILUTION;
                continue;
            }
            let neighbor = match self.decisions.c_modulo(DIFFUSION_DRAW_RANGE)? {
                draw if draw <= WIND_NORTH_END => i - side,
                draw if draw <= WIND_WEST_END => i - 1,
                draw if draw <= WIND_SOUTH_END => i + side,
                draw if draw <= WIND_EAST_END => i + 1,
                _ => {
                    self.tiles[i].pollution -= POLLUTION_DECAY;
                    i
                }
            };
            let flow = self.tiles[i].pollution / DIFFUSION_DIVISOR;
            self.tiles[i].pollution -= flow;
            self.tiles[neighbor].pollution += flow;
        }
        Ok(())
    }

    pub(super) fn random_fire(&mut self) -> io::Result<()> {
        let interval =
            FIRE_BASE_INTERVAL * FIRE_REFERENCE_SIDE / self.side * FIRE_REFERENCE_SIDE / self.side;
        if self.tech_level <= BuildingKind::FireStation.metadata().required_tech
            || interval <= FIRE_DAY
            || self.day % interval != FIRE_DAY
        {
            return Ok(());
        }
        // The reference GCC build evaluates MapPoint's y argument first.
        let y = self.decisions.c_modulo(self.side)?;
        let x = self.decisions.c_modulo(self.side)?;
        let p = Point { x, y };
        if let Some(id) = self.buildings.at(p) {
            let (origin, b) = self.buildings.get(id).unwrap();
            if self.decisions.c_modulo(PERCENT)? < b.kind.metadata().fire_chance
                && self.tiles[(y * self.side + x) as usize].coverage & crate::rules::civic::FIRE
                    == 0
            {
                self.ignite_building(id, origin)?;
                if let Some(CityEvent::FireStarted(location, _)) = self.events.back_mut() {
                    *location = p;
                }
            }
        }
        Ok(())
    }

    pub(super) fn check_sustainability(&mut self) {
        let s = &mut self.stats.sustainability;
        for (flag, years) in [
            (&mut s.mining_flag, &mut s.mining_years),
            (&mut s.trade_flag, &mut s.trade_years),
        ] {
            if *flag {
                *years += 1;
            } else {
                *flag = true;
                *years = 0;
            }
        }
        s.money_years = if s.old_money > self.balance {
            0
        } else {
            s.money_years + 1
        };
        s.old_money = self.balance;
        let population = self.stats.population.population_m.stat;
        if (i64::from(population) - i64::from(s.old_population)).abs()
            > i64::from(s.old_population) / POPULATION_TOLERANCE_DIVISOR
            || population < SUSTAINABLE_POPULATION * DAYS_PER_MONTH
        {
            s.old_population = population;
            s.population_years = 0;
        } else {
            s.population_years += 1;
        }
        if i64::from(s.old_tech) - i64::from(self.tech_level)
            > i64::from(s.old_tech) / TECH_TOLERANCE_DIVISOR
            || self.tech_level < SUSTAINABLE_TECH
        {
            s.tech_years = 0;
            s.old_tech = self.tech_level;
        } else {
            s.tech_years += 1;
            s.old_tech = s.old_tech.max(self.tech_level);
        }
        if self.day % (COVERAGE_CHECK_YEARS * DAYS_PER_YEAR) == 0 {
            let covered = self.buildings.iter().all(|(p, b)| {
                b.kind.is_transport()
                    || matches!(
                        b.kind,
                        BuildingKind::Monument
                            | BuildingKind::OreMine
                            | BuildingKind::RocketPad
                            | BuildingKind::Fire
                            | BuildingKind::PowerLine
                    )
                    || self.tiles[(p.y * self.side + p.x) as usize].coverage
                        & crate::rules::civic::FIRE
                        != 0
            });
            s.fire_years = if covered {
                s.fire_years + COVERAGE_CHECK_YEARS
            } else {
                0
            };
        }
        if s.mining_years >= MINING_YEARS
            && s.trade_years >= TRADE_YEARS
            && s.money_years >= MONEY_YEARS
            && s.population_years >= POPULATION_YEARS
            && s.tech_years >= TECHNOLOGY_YEARS
            && s.fire_years >= FIRE_YEARS
        {
            if !s.sustainable {
                self.events
                    .push_back(CityEvent::SustainableEconomy(self.day));
            }
            s.sustainable = true;
        }
    }
}
