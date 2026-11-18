use super::rules::housing::*;
use super::*;
use crate::decisions::Decisions;
use crate::stats::Population;

#[derive(Default)]
pub(in crate::city) struct Residence {
    pub(super) occupants: i32,
    pub(super) flags: u32,
    pub(super) desirability: i32,
    births: i32,
    deaths: i32,
    pollution_deaths: i32,
}

pub(super) struct Neighborhood<'a> {
    pub tile: Tile,
    pub people_pool: &'a mut i32,
    pub stats: &'a mut Population,
    pub decisions: &'a mut Decisions,
}

impl Residence {
    pub(super) fn update(
        &mut self,
        kind: BuildingKind,
        stock: &mut [i32; 10],
        world: Neighborhood<'_>,
    ) -> io::Result<()> {
        let neighborhood = world.tile.coverage;
        let health = neighborhood & HEALTH != 0;
        let fire = neighborhood & FIRE != 0;
        let cricket = neighborhood & CRICKET != 0;
        let mut good = BASE_APPEAL
            + HEALTH_APPEAL * i32::from(health)
            + FIRE_APPEAL * i32::from(fire)
            + SPORTS_APPEAL * i32::from(cricket);
        let mut bad = BASE_DISCONTENT + NO_FIRE_PENALTY * i32::from(!fire);
        let mut birth_modifier = HEALTH_BIRTH_MODIFIER * i32::from(health);
        let mut death_modifier = 0;
        let mut n = self.occupants;

        if n != 0 && stock[Food as usize] >= n && stock[Water as usize] >= n {
            stock[Food as usize] -= n;
            stock[Water as usize] -= n;
            self.flags |= FED;
            good += FED_APPEAL;
        } else {
            self.flags &= !FED;
            if n != 0 {
                if world.decisions.c_modulo(STARVATION_ODDS)? == 1 {
                    n -= 1;
                    world.stats.deaths_m += 1;
                    world.stats.unnat_deaths_m += 1;
                    world.stats.starve_deaths_t += 1;
                    world.stats.starve_deaths_history += 1.0;
                }
                world.stats.starving_m += n;
                bad += STARVATION_PENALTY;
                death_modifier = STARVATION_DEATH_MODIFIER;
            }
        }
        let power = POWER_OVERHEAD + POWER_PER_RESIDENT * n;
        if stock[LowVoltage as usize] >= power {
            stock[LowVoltage as usize] -= power;
            self.flags |= HAD_POWER;
            good += POWER_APPEAL;
        } else {
            bad += POWER_SHORTAGE_PENALTY
                + LOST_POWER_PENALTY * i32::from(self.flags & HAD_POWER != 0);
        }

        let capacity = kind.metadata().rule(Labor).max_load;
        let room = capacity - stock[Labor as usize];
        let swing =
            JOB_SWING + HEALTH_SWING * i32::from(health) + SPORTS_SWING * i32::from(cricket);
        let working = WORKING_PERCENT
            + HEALTH_WORKERS * i32::from(health)
            + SPORTS_WORKERS * i32::from(cricket);
        let jobs = n * (working + swing * room / capacity) / 100;
        if room >= jobs {
            stock[Labor as usize] += jobs;
            self.flags |= EMPLOYED;
            good += EMPLOYMENT_APPEAL;
            let can_buy = |stock: &[i32; 10]| {
                stock[Goods as usize] >= n / PEOPLE_PER_GOOD
                    && kind.metadata().rule(Waste).max_load - stock[Waste as usize]
                        >= n / PEOPLE_PER_WASTE
            };
            if can_buy(stock) {
                stock[Goods as usize] -= n / PEOPLE_PER_GOOD;
                stock[Waste as usize] += n / PEOPLE_PER_WASTE;
                good += GOODS_APPEAL;
                if stock[LowVoltage as usize] >= n / PEOPLE_PER_APPLIANCE_POWER {
                    stock[LowVoltage as usize] -= n / PEOPLE_PER_APPLIANCE_POWER;
                    good += APPLIANCE_POWER_APPEAL;
                    birth_modifier += APPLIANCE_BIRTH_MODIFIER;
                    if can_buy(stock) {
                        stock[Goods as usize] -= n / PEOPLE_PER_GOOD;
                        stock[Waste as usize] += n / PEOPLE_PER_WASTE;
                        good += EXTRA_GOODS_APPEAL;
                    }
                } else {
                    bad += APPLIANCE_SHORTAGE_PENALTY;
                }
            }
        } else {
            self.flags &= !EMPLOYED;
            world.stats.unemployed_m += n;
            world.stats.unemployed_days_t += n;
            world.stats.unemployed_history = (f64::from(world.stats.unemployed_history)
                + f64::from(n) / f64::from(crate::rules::DAYS_PER_YEAR))
                as f32;
            bad += UNEMPLOYMENT_PENALTY;
        }

        let rule = rules::housing_rule(kind);
        let density = n / rule.occupancy_divisor * rule.density_factor / rule.density_divisor;
        let housing = rule.capacity;
        good += rule.appeal;
        death_modifier =
            (death_modifier + density + n / PEOPLE_PER_RATE_MODIFIER).min(BASE_DEATH_INTERVAL - 1);
        birth_modifier += rule.birth_delay + n / PEOPLE_PER_RATE_MODIFIER;
        let pollution = world.tile.pollution / POLLUTION_UNIT + 1;
        self.pollution_deaths = if pollution > 100 {
            FATAL_POLLUTION_MAX_PERCENT
        } else {
            (pollution - FATAL_POLLUTION_OFFSET).max(1)
        };
        self.deaths = (BASE_DEATH_INTERVAL - death_modifier - POLLUTION_DEATH_FACTOR * pollution)
            .max(1)
            * if health { HEALTH_SURVIVAL_FACTOR } else { 1 };
        let death = world.decisions.c_modulo(self.deaths)? == 0;
        if n > 0 {
            if death {
                n -= 1;
                world.stats.deaths_m += 1;
                if world.decisions.c_modulo(100)? < self.pollution_deaths {
                    world.stats.unnat_deaths_m += 1;
                    world.stats.pollution_deaths_t += 1;
                    world.stats.pollution_deaths_history += 1.0;
                    bad += POLLUTION_DEATH_PENALTY;
                }
            }
        } else {
            self.deaths = INACTIVE_INTERVAL;
        }
        self.births = BASE_BIRTH_INTERVAL + birth_modifier;
        if self.flags & (FED | EMPLOYED) == FED | EMPLOYED && n > 0 {
            if world.decisions.c_modulo(self.births)? == 0 {
                n += 1;
                world.stats.births_t += 1;
                world.stats.births_m += 1;
                good += BIRTH_APPEAL;
            }
        } else {
            self.births = INACTIVE_INTERVAL;
        }
        bad += world.tile.pollution / POLLUTION_PER_DISCONTENT;
        good += *world.people_pool / UNHOUSED_PER_APPEAL;
        self.desirability = good - bad;
        let draw = world.decisions.c_modulo((good + bad) * MIGRATION_ODDS)?;
        if draw < bad || n > housing {
            if n > MIN_OCCUPANTS {
                n -= 1;
                *world.people_pool += 1;
            }
        } else if *world.people_pool > 0
            && n < housing
            && draw > (good + bad) * (MIGRATION_ODDS - 1) + bad
        {
            n += 1;
            *world.people_pool -= 1;
        }
        self.occupants = n;
        world.stats.population_m += n;
        world.stats.housed_m += n;
        world.stats.housing_m += housing;
        Ok(())
    }
}
