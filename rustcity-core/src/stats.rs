//! Population, inventory, and fiscal accounts with independent reporting periods.
//! Based on Lincity-NG stats.hpp and stats.cpp (GPL-2.0-or-later).

use std::collections::VecDeque;
use std::ops::{AddAssign, SubAssign};

use crate::Commodity;
use crate::rules::{statistics::*, transport::QUANTA};

const COMMODITIES: usize = Commodity::ALL.len();

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stat {
    pub(crate) stat: i32,
    pub(crate) acc: i32,
}

impl Stat {
    /// Accumulated value in the open reporting period.
    pub fn period_to_date(self) -> i32 {
        self.acc
    }

    /// Value at the close of the previous reporting period.
    pub fn value(self) -> i32 {
        self.stat
    }

    pub fn finalize(&mut self) -> i32 {
        self.stat = std::mem::take(&mut self.acc);
        self.stat
    }
}

impl AddAssign<i32> for Stat {
    fn add_assign(&mut self, rhs: i32) {
        self.acc += rhs;
    }
}

impl SubAssign<i32> for Stat {
    fn sub_assign(&mut self, rhs: i32) {
        self.acc -= rhs;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Inventory<T = i32> {
    pub amount: T,
    pub capacity: T,
}

impl<T: AddAssign<U>, U> AddAssign<Inventory<U>> for Inventory<T> {
    fn add_assign(&mut self, rhs: Inventory<U>) {
        self.amount += rhs.amount;
        self.capacity += rhs.capacity;
    }
}

impl<T: SubAssign<U>, U> SubAssign<Inventory<U>> for Inventory<T> {
    fn sub_assign(&mut self, rhs: Inventory<U>) {
        self.amount -= rhs.amount;
        self.capacity -= rhs.capacity;
    }
}

impl Inventory {
    /// Reports stock in parts per thousand, truncating toward zero.
    pub fn percent(&self) -> i32 {
        if self.capacity == 0 {
            return 0;
        }
        ((i64::from(self.amount) * i64::from(QUANTA)) / i64::from(self.capacity)) as i32
    }
}

#[derive(Clone, Debug, Default)]
pub struct Population {
    pub population_m: Stat,
    pub housed_m: Stat,
    pub starving_m: Stat,
    pub unemployed_m: Stat,
    pub housing_m: Stat,
    pub births_m: Stat,
    pub deaths_m: Stat,
    pub unnat_deaths_m: Stat,
    pub evacuated_t: i32,
    pub births_t: i32,
    pub pollution_deaths_t: i32,
    pub starve_deaths_t: i32,
    pub unemployed_days_t: i32,
    pub unemployed_history: f32,
    pub pollution_deaths_history: f32,
    pub starve_deaths_history: f32,
    pub max_pop_ever: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Income {
    pub income_tax: Stat,
    pub coal_tax: Stat,
    pub goods_tax: Stat,
    pub export_tax: Stat,
}

#[derive(Clone, Debug, Default)]
pub struct Expenses {
    pub construction: Stat,
    pub coal_survey: Stat,
    pub import: Stat,
    pub unemployment: Stat,
    pub transport: Stat,
    pub windmill: Stat,
    pub university: Stat,
    pub recycle: Stat,
    pub deaths: Stat,
    pub health: Stat,
    pub rockets: Stat,
    pub school: Stat,
    pub firestation: Stat,
    pub cricket: Stat,
    pub interest: Stat,
}

#[derive(Clone, Debug)]
pub struct Sustainability {
    pub mining_flag: bool,
    pub trade_flag: bool,
    pub mining_years: i32,
    pub trade_years: i32,
    pub old_money: i32,
    pub money_years: i32,
    pub old_population: i32,
    pub population_years: i32,
    pub old_tech: i32,
    pub tech_years: i32,
    pub fire_years: i32,
    pub sustainable: bool,
}

impl Default for Sustainability {
    fn default() -> Self {
        Self {
            mining_flag: true,
            trade_flag: true,
            mining_years: 0,
            trade_years: 0,
            old_money: 0,
            money_years: 0,
            old_population: 0,
            population_years: 0,
            old_tech: 0,
            tech_years: 0,
            fire_years: 0,
            sustainable: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct History {
    pub pop: VecDeque<i32>,
    pub starve: VecDeque<i32>,
    pub nojobs: VecDeque<i32>,
    pub ppool: VecDeque<i32>,
    pub tech: VecDeque<i32>,
    pub money: VecDeque<i32>,
    pub pollution: VecDeque<i32>,
    pub births: VecDeque<i32>,
    pub deaths: VecDeque<i32>,
    pub unnat_deaths: VecDeque<i32>,
    pub inventory: [VecDeque<Inventory>; COMMODITIES],
    pub tenants: VecDeque<Inventory>,
}

impl Default for History {
    fn default() -> Self {
        Self {
            pop: VecDeque::from([0; POPULATION_SAMPLES]),
            starve: VecDeque::from([0; POPULATION_SAMPLES]),
            nojobs: VecDeque::from([0; POPULATION_SAMPLES]),
            ppool: VecDeque::from([0; POPULATION_SAMPLES]),
            tech: VecDeque::from([0; FISCAL_SAMPLES]),
            money: VecDeque::from([0; FISCAL_SAMPLES]),
            pollution: VecDeque::from([0; POLLUTION_SAMPLES]),
            births: VecDeque::from([0; FISCAL_SAMPLES]),
            deaths: VecDeque::from([0; FISCAL_SAMPLES]),
            unnat_deaths: VecDeque::from([0; FISCAL_SAMPLES]),
            inventory: std::array::from_fn(|_| {
                VecDeque::from([Inventory::default(); INVENTORY_SAMPLES])
            }),
            tenants: VecDeque::from([Inventory::default(); FISCAL_SAMPLES]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    pub population: Population,
    pub income: Income,
    pub expenses: Expenses,
    pub sustainability: Sustainability,
    pub inventory: [Inventory<Stat>; COMMODITIES],
    pub group_count: [u32; crate::BuildingKind::GROUP_SLOTS],
    pub history: History,
    pub highest_tech_level: i32,
    pub total_pollution: i32,
    pub tech_level: i32,
    pub total_money: i32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            population: Default::default(),
            income: Default::default(),
            expenses: Default::default(),
            sustainability: Default::default(),
            inventory: Default::default(),
            history: Default::default(),
            group_count: [0; crate::BuildingKind::GROUP_SLOTS],
            highest_tech_level: 0,
            total_pollution: 0,
            tech_level: 0,
            total_money: 0,
        }
    }
}

fn record<T>(history: &mut VecDeque<T>, value: T) {
    history.push_front(value);
    history.pop_back();
}

impl Stats {
    pub fn ensure_population_history(&mut self, months: usize) {
        let h = &mut self.history;
        for series in [&mut h.pop, &mut h.ppool, &mut h.nojobs, &mut h.starve] {
            if series.len() < months {
                series.resize(months, 0);
            }
        }
    }

    /// The daily tick leaves all reporting periods open.
    pub fn daily(&mut self) {}

    pub fn monthly(&mut self) {
        let p = &mut self.population;
        let h = &mut self.history;
        let population = p.population_m.finalize();
        let housed = p.housed_m.finalize();
        record(&mut h.pop, population);
        record(&mut h.ppool, population - housed);
        record(
            &mut h.tenants,
            Inventory {
                amount: housed,
                capacity: p.housing_m.finalize(),
            },
        );
        record(&mut h.starve, p.starving_m.finalize());
        record(&mut h.nojobs, p.unemployed_m.finalize());
        record(&mut h.births, p.births_m.finalize());
        record(&mut h.deaths, p.deaths_m.finalize());
        record(&mut h.unnat_deaths, p.unnat_deaths_m.finalize());
        record(&mut h.tech, self.tech_level);
        record(&mut h.money, self.total_money);
        record(&mut h.pollution, self.total_pollution);
        for (inventory, series) in self.inventory.iter_mut().zip(&mut h.inventory) {
            record(
                series,
                Inventory {
                    amount: inventory.amount.finalize(),
                    capacity: inventory.capacity.finalize(),
                },
            );
        }
    }

    pub fn yearly(&mut self) {
        let p = &mut self.population;
        for counter in [
            &mut p.unemployed_history,
            &mut p.pollution_deaths_history,
            &mut p.starve_deaths_history,
        ] {
            // The C++ expression promotes its float operand to double before multiplication.
            *counter = (f64::from(*counter) * YEARLY_HISTORY_RETENTION) as f32;
        }
        self.income.income_tax.finalize();
        self.income.coal_tax.finalize();
        self.income.goods_tax.finalize();
        self.income.export_tax.finalize();
        self.expenses.construction.finalize();
        self.expenses.coal_survey.finalize();
        self.expenses.import.finalize();
        self.expenses.unemployment.finalize();
        self.expenses.transport.finalize();
        self.expenses.windmill.finalize();
        self.expenses.university.finalize();
        self.expenses.recycle.finalize();
        self.expenses.deaths.finalize();
        self.expenses.health.finalize();
        self.expenses.rockets.finalize();
        self.expenses.school.finalize();
        self.expenses.firestation.finalize();
        self.expenses.cricket.finalize();
        self.expenses.interest.finalize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_adjustments_preserve_signed_and_wide_per_mille() {
        for (start, add, subtract, ratios) in [
            ((2, 3), (1, 3), (3, 6), [666, 500, 0]),
            ((-2, 3), (5, 7), (4, 3), [-666, 300, -142]),
            ((9_000_000, 15_000_000), (0, 0), (1, 1), [600, 600, 599]),
        ] {
            let inventory = |(amount, capacity)| Inventory { amount, capacity };
            let mut stock = inventory(start);
            assert_eq!(stock.percent(), ratios[0]);
            stock += inventory(add);
            assert_eq!(stock.percent(), ratios[1]);
            stock -= inventory(subtract);
            assert_eq!(stock.percent(), ratios[2]);
        }
    }

    #[test]
    fn month_and_year_close_separate_ledgers() {
        let mut s = Stats::default();
        s.population.population_m += 17;
        s.population.housed_m += 12;
        s.inventory[0] += Inventory {
            amount: 2,
            capacity: 3,
        };
        s.income.goods_tax += 18;
        s.expenses.transport += 7;
        s.population.births_m += 4;
        s.population.unemployed_history = 100.;
        s.daily();
        assert_eq!(s.population.population_m.acc, 17);
        assert_eq!(s.population.population_m.stat, 0);
        s.yearly();
        assert_eq!(s.income.goods_tax.stat, 18);
        assert_eq!(s.expenses.transport.stat, 7);
        assert_eq!(s.population.unemployed_history.to_bits(), 1120272384);
        s.monthly();
        assert_eq!(s.history.ppool[0], 5);
        assert_eq!(s.history.births[0], 4);
        s.monthly();
        assert_eq!((s.history.pop[0], s.history.pop[1]), (0, 17));
        assert_eq!(s.history.inventory[0][1].percent(), 666);
        s.yearly();
        assert_eq!(s.income.goods_tax.stat, 0);
        assert_eq!(s.population.unemployed_history.to_bits(), 1120142623);
    }

    #[test]
    fn consecutive_years_keep_monthly_population_and_round_decay_like_cpp() {
        let mut s = Stats::default();
        s.population.population_m += 35;
        s.population.housed_m += 35;
        s.population.unemployed_history = 23.;
        s.income.goods_tax -= 9;
        s.daily();
        s.monthly();
        assert_eq!(s.history.ppool[0], 0);
        s.yearly();
        assert_eq!(s.income.goods_tax.stat, -9);
        assert_eq!(s.population.unemployed_history.to_bits(), 1102457078);
        s.yearly();
        assert_eq!(s.population.unemployed_history.to_bits(), 1102337698);
        assert_eq!(s.population.population_m.stat, 35);
        s.monthly();
        assert_eq!((s.history.pop[0], s.history.pop[1]), (0, 35));
    }

    #[test]
    fn population_history_grows_at_the_old_end_and_keeps_its_size() {
        let mut s = Stats::default();
        assert_eq!(s.history.pop.len(), 190);
        s.population.starving_m += 3;
        s.monthly();
        s.ensure_population_history(200);
        s.ensure_population_history(5);
        s.monthly();
        for history in [
            &s.history.pop,
            &s.history.ppool,
            &s.history.nojobs,
            &s.history.starve,
        ] {
            assert_eq!(history.len(), 200);
        }
        assert_eq!(s.history.starve[1], 3);
        assert_eq!(s.history.pollution.len(), 2);
        assert_eq!(s.history.tenants.len(), 12);
    }
}
