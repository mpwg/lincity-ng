//! Electricity production, power line losses and their animation state.

use crate::rules::{CREDIT_LIMIT, DAYS_PER_MONTH, MAX_TECH_LEVEL, power::*, transport};
use crate::{BuildingKind, Commodity};
use Commodity::{Coal, HighVoltage, Labor, LowVoltage};

/// Deliveries and trade change building stocks directly. Local consumption and
/// production additionally contribute to the monthly commodity report.
#[derive(Clone, Debug, Default)]
pub struct ProductionCounters {
    pub current: [i32; Commodity::ALL.len()],
    pub previous: [i32; Commodity::ALL.len()],
}

impl ProductionCounters {
    pub fn change(&mut self, stock: &mut [i32; 10], commodity: Commodity, delta: i32) {
        stock[commodity as usize] += delta;
        self.current[commodity as usize] += delta;
    }

    pub fn close_month(&mut self) {
        self.previous = std::mem::take(&mut self.current);
    }
}

fn installed_output(base: i32, technology: i32) -> i32 {
    (f64::from(base) + f64::from(technology) * f64::from(base) / f64::from(MAX_TECH_LEVEL)) as i32
}

fn headroom(kind: BuildingKind, commodity: Commodity, stock: &[i32; 10]) -> i32 {
    kind.metadata().rule(commodity).max_load - stock[commodity as usize]
}

/// Time offset applied to the host's nominal 100 ms animation cadence.
const fn animation_threshold(millis: u64) -> u64 {
    (millis + 50) / 100 * 100 - 50
}

#[derive(Debug, Clone)]
pub struct CoalPlant {
    pub output: i32,
    pub busy: i32,
    working_hundreds: i32,
    next_animation: u64,
    pub resource: &'static str,
    pub smoke: [i32; 8],
}

impl CoalPlant {
    pub const SMOKE_OFFSETS: [(i32, i32); 8] = [
        (5, -378),
        (29, -390),
        (52, -397),
        (76, -409),
        (65, -348),
        (89, -360),
        (112, -371),
        (136, -383),
    ];

    pub fn new(technology: i32) -> Self {
        Self {
            output: installed_output(COAL_OUTPUT, technology),
            busy: 0,
            working_hundreds: 0,
            next_animation: 0,
            resource: "PowerCoalEmpty",
            smoke: [-1; 8],
        }
    }

    pub fn update(
        &mut self,
        day: u32,
        stock: &mut [i32; 10],
        counters: &mut ProductionCounters,
        pollution: &mut i32,
    ) {
        let made = self
            .output
            .min(headroom(BuildingKind::CoalPower, HighVoltage, stock));
        if made >= COAL_OUTPUT {
            let hundreds = made / OUTPUT_QUANTUM;
            let capacity = self.output / OUTPUT_QUANTUM;
            let labor = COAL_LABOR * hundreds / capacity;
            let coal = (COAL_OUTPUT / POWER_PER_COAL) * hundreds / capacity;
            if stock[Labor as usize] >= labor && stock[Coal as usize] >= coal {
                counters.change(stock, Labor, -labor);
                counters.change(stock, Coal, -coal);
                counters.change(stock, HighVoltage, made);
                *pollution += COAL_POLLUTION * hundreds / capacity;
                self.working_hundreds += hundreds;
            }
        }
        if day % DAYS_PER_MONTH as u32 == (DAYS_PER_MONTH - 1) as u32 {
            self.busy = std::mem::take(&mut self.working_hundreds) / (self.output / OUTPUT_QUANTUM);
            counters.close_month();
        }
    }

    pub fn animate(
        &mut self,
        millis: u64,
        coal: i32,
        mut random: impl FnMut() -> std::io::Result<u32>,
    ) -> std::io::Result<()> {
        if millis >= self.next_animation {
            self.next_animation = millis + animation_threshold(COAL_SMOKE_SPEED);
            let active = 9 * self.busy / 100;
            for (index, frame) in self.smoke.iter_mut().enumerate() {
                if index as i32 >= active {
                    *frame = -1;
                } else if *frame < 0 || !random()?.is_multiple_of(SMOKE_RANDOM_PERIOD) {
                    *frame = (random()? % BLACK_SMOKE_FRAMES as u32) as i32;
                } else {
                    *frame = (*frame + 1) % BLACK_SMOKE_FRAMES;
                }
            }
        }
        let capacity = BuildingKind::CoalPower.metadata().rule(Coal).max_load;
        self.resource = if coal > capacity * 4 / 5 {
            "PowerCoalFull"
        } else if coal > capacity / 2 {
            "PowerCoalMed"
        } else if coal > capacity / 10 {
            "PowerCoalLow"
        } else {
            "PowerCoalEmpty"
        };
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WindExpenses {
    /// All low-voltage windmills in the same city share this flag.
    pub paid: bool,
}

#[derive(Clone, Debug)]
pub struct WindGenerator {
    kind: BuildingKind,
    pub output: i32,
    pub busy_output: i32,
    working_output: i32,
    spinning: bool,
    next_animation: u64,
    pub frame: i32,
    pub resource: &'static str,
}

impl WindGenerator {
    pub fn windmill(technology: i32) -> Self {
        Self::new(BuildingKind::Windmill, technology)
    }
    pub fn wind_power(technology: i32) -> Self {
        Self::new(BuildingKind::WindPower, technology)
    }

    fn new(kind: BuildingKind, technology: i32) -> Self {
        Self {
            kind,
            output: installed_output(WIND_OUTPUT, technology),
            busy_output: 0,
            working_output: 0,
            spinning: false,
            next_animation: 0,
            frame: 0,
            resource: kind.metadata().resource,
        }
    }

    /// Returns whether debiting the treasury crossed zero and raised a money warning.
    pub fn update(
        &mut self,
        day: u32,
        stock: &mut [i32; 10],
        counters: &mut ProductionCounters,
        expenses: &mut WindExpenses,
        balance: &mut i32,
        windmill_expense: &mut i32,
    ) -> bool {
        let mut money_warning = false;
        let antique = self.kind == BuildingKind::Windmill;
        let (commodity, workers) = if antique {
            if day.is_multiple_of(WINDMILL_COST_PERIOD) {
                expenses.paid = false;
            }
            if !expenses.paid && *balance > CREDIT_LIMIT {
                money_warning = *balance == 1;
                *balance -= 1;
                *windmill_expense = windmill_expense.saturating_add(1);
                expenses.paid = true;
            }
            (LowVoltage, WINDMILL_LABOR)
        } else {
            if !day.is_multiple_of(WIND_POWER_COST_PERIOD) {
                *windmill_expense += 1;
            }
            (HighVoltage, WIND_POWER_LABOR)
        };
        let made = self.output.min(headroom(self.kind, commodity, stock));
        let minimum = WIND_OUTPUT + i32::from(!antique);
        self.spinning = false;
        if made >= minimum && (!antique || expenses.paid) {
            let labor = workers * made / self.output;
            if stock[Labor as usize] >= labor {
                counters.change(stock, Labor, -labor);
                counters.change(stock, commodity, made);
                self.working_output += made;
                self.spinning = true;
            }
        }
        if day % DAYS_PER_MONTH as u32 == (DAYS_PER_MONTH - 1) as u32 {
            self.busy_output = std::mem::take(&mut self.working_output);
            counters.close_month();
        }
        money_warning
    }

    pub fn utilization(&self) -> f32 {
        self.busy_output as f32 / self.output as f32
    }

    pub fn animate(&mut self, millis: u64, stock: &[i32; 10]) {
        if self.spinning && millis >= self.next_animation {
            self.next_animation = millis + animation_threshold(WIND_SPEED);
            self.frame = (self.frame + 1) % WIND_FRAMES;
        }
        if self.kind == BuildingKind::WindPower {
            let amount = stock[HighVoltage as usize];
            let capacity = self.kind.metadata().rule(HighVoltage).max_load;
            self.resource = if amount > capacity / 2 {
                "WindMillHTechG"
            } else if amount > capacity / 10 {
                "WindMillHTechRG"
            } else {
                "WindMillHTech"
            };
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PowerLine {
    pub counter: i32,
    pub flashing: bool,
    pub frame: i32,
    pub traffic: i32,
    next_animation: u64,
}

impl PowerLine {
    pub fn update(&mut self, day: u32, stock: &mut [i32; 10], counters: &mut ProductionCounters) {
        if stock[HighVoltage as usize] > 0 {
            counters.change(stock, HighVoltage, -1);
        }
        if day % DAYS_PER_MONTH as u32 == (DAYS_PER_MONTH - 1) as u32 {
            counters.close_month();
        }
    }

    /// Book the busiest individual transfer made during this line's visit.
    pub fn record_traffic(&mut self, max_transfer: i32) {
        let capacity = BuildingKind::PowerLine
            .metadata()
            .rule(HighVoltage)
            .max_load;
        self.traffic = ((transport::SMOOTHING - 1) * self.traffic
            + max_transfer * transport::QUANTA / capacity)
            / transport::SMOOTHING;
    }

    pub fn usage(&self) -> f64 {
        f64::from(self.traffic) * transport::TRAFFIC_REPORT_FACTOR * f64::from(transport::RATE)
            / f64::from(transport::QUANTA)
    }

    /// Called after exchange for each adjacent non-line building in visit order.
    pub fn observe_supplier(&mut self, gives_stock: bool, random: impl FnOnce() -> u32) {
        if self.counter == 0 && gives_stock {
            self.counter = LINE_MODULUS + (random() % LINE_MODULUS as u32) as i32;
        }
    }

    pub fn receive_flash(&mut self) {
        if self.counter <= LINE_MODULUS / 3 {
            self.counter = LINE_MODULUS;
        }
    }

    pub fn animate(&mut self, millis: u64) {
        match self.counter {
            n if n == LINE_MODULUS - 2 && self.frame >= LINE_FLASH_OFFSET => {
                self.flashing = false;
                self.frame -= LINE_FLASH_OFFSET;
            }
            LINE_MODULUS if self.frame < LINE_FLASH_OFFSET => {
                self.flashing = true;
                self.frame += LINE_FLASH_OFFSET;
            }
            _ => {}
        }
        if self.counter > 0 && millis >= self.next_animation {
            self.next_animation = millis + animation_threshold(LINE_FLASH_SPEED);
            self.counter -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Commodity::*;

    #[test]
    fn coal_batches_preserve_rounding_pollution_and_monthly_stock_counters() {
        let mut plant = CoalPlant::new(1_000_000);
        let mut stock = [0; 10];
        let mut stock_counters = ProductionCounters::default();
        let mut pollution = 0;
        stock[Coal as usize] = 43;
        stock[Labor as usize] = 100;
        plant.update(1, &mut stock, &mut stock_counters, &mut pollution);
        assert_eq!(stock[HighVoltage as usize], 0);
        stock[Coal as usize] = 44;
        plant.update(2, &mut stock, &mut stock_counters, &mut pollution);
        assert_eq!(pollution, 20);
        assert_eq!(stock[HighVoltage as usize], 22000);
        stock[HighVoltage as usize] = 209001;
        stock[Coal as usize] = 22;
        stock[Labor as usize] = 50;
        plant.update(98, &mut stock, &mut stock_counters, &mut pollution);
        assert_eq!(stock[Coal as usize], 22);
        stock[HighVoltage as usize] = 209000;
        plant.update(99, &mut stock, &mut stock_counters, &mut pollution);
        assert_eq!(stock[HighVoltage as usize], 220000);
        assert_eq!(stock_counters.previous[HighVoltage as usize], 33000);
        assert_eq!(stock_counters.previous[Coal as usize], -66);
        assert_eq!((pollution, plant.busy), (30, 1));
    }

    #[test]
    fn smoke_uses_work_done_last_month_and_consumes_random_draws_in_order() {
        let mut coal = CoalPlant::new(0);
        coal.busy = 23;
        let mut draws = [10, 3, 0, 42, 13].into_iter();
        coal.animate(100, 704, || Ok(draws.next().unwrap()))
            .unwrap();
        assert_eq!(coal.smoke, [10, 3, -1, -1, -1, -1, -1, -1]);
        assert_eq!(coal.resource, "PowerCoalMed");
        coal.animate(349, 705, || panic!("early draw")).unwrap();
        assert_eq!(coal.resource, "PowerCoalFull");
        coal.animate(350, 88, || Ok(draws.next().unwrap())).unwrap();
        assert_eq!(coal.smoke, [0, 2, -1, -1, -1, -1, -1, -1]);
        assert_eq!(draws.next(), None);
    }

    #[test]
    fn wind_technologies_apply_distinct_output_thresholds_and_costs() {
        let mut old = WindGenerator::windmill(1_000_000);
        let mut modern = WindGenerator::wind_power(1_000_000);
        let mut low = [0; 10];
        let mut low_counters = ProductionCounters::default();
        let mut high = [0; 10];
        let mut high_counters = ProductionCounters::default();
        low[LowVoltage as usize] = 8550;
        low[Labor as usize] = 5;
        high[HighVoltage as usize] = 8550;
        high[Labor as usize] = 7;
        let mut treasury = WindExpenses::default();
        let mut budget = 20;
        let mut expense = 0;
        old.update(
            1,
            &mut low,
            &mut low_counters,
            &mut treasury,
            &mut budget,
            &mut expense,
        );
        modern.update(
            1,
            &mut high,
            &mut high_counters,
            &mut treasury,
            &mut budget,
            &mut expense,
        );
        assert_eq!(
            (
                low[LowVoltage as usize],
                high[HighVoltage as usize],
                budget,
                expense
            ),
            (9000, 8550, 19, 2)
        );
        high[HighVoltage as usize] = 8549;
        modern.update(
            2,
            &mut high,
            &mut high_counters,
            &mut treasury,
            &mut budget,
            &mut expense,
        );
        assert_eq!(
            (high[HighVoltage as usize], high[Labor as usize], expense),
            (9000, 0, 2)
        );
        modern.animate(300, &high);
        modern.animate(349, &high);
        assert_eq!((modern.frame, modern.resource), (1, "WindMillHTechG"));
        modern.animate(350, &high);
        assert_eq!(modern.frame, 2);
    }

    #[test]
    fn unpaid_windmills_retry_when_credit_becomes_available() {
        let mut old = WindGenerator::windmill(0);
        let mut stock = [0; 10];
        let mut stock_counters = ProductionCounters::default();
        stock[Labor as usize] = 100;
        let mut paid = WindExpenses::default();
        let mut budget = -2_000_000_000;
        let mut expense = 0;
        old.update(
            4,
            &mut stock,
            &mut stock_counters,
            &mut paid,
            &mut budget,
            &mut expense,
        );
        assert_eq!(stock[LowVoltage as usize], 0);
        budget = 1;
        assert!(old.update(
            5,
            &mut stock,
            &mut stock_counters,
            &mut paid,
            &mut budget,
            &mut expense
        ));
        old.update(
            6,
            &mut stock,
            &mut stock_counters,
            &mut paid,
            &mut budget,
            &mut expense,
        );
        assert_eq!((stock[LowVoltage as usize], budget, expense), (900, 0, 1));
    }

    #[test]
    fn line_loss_and_flashing_follow_trade_and_animation_order() {
        let mut line = PowerLine::default();
        let mut next = PowerLine::default();
        let mut stock = [0; 10];
        let mut stock_counters = ProductionCounters::default();
        stock[HighVoltage as usize] = 1;
        line.update(98, &mut stock, &mut stock_counters);
        line.update(99, &mut stock, &mut stock_counters);
        assert_eq!(stock_counters.previous[HighVoltage as usize], -1);
        line.observe_supplier(true, || 1);
        line.frame = 3;
        line.animate(100);
        assert_eq!((line.frame, line.counter), (3, 18));
        line.animate(101);
        assert_eq!((line.frame, line.counter), (14, 18));
        assert!(line.flashing);
        next.receive_flash();
        assert_eq!(next.counter, 18);
        line.animate(150);
        line.animate(200);
        line.animate(201);
        assert_eq!((line.frame, line.counter, line.flashing), (3, 16, false));
    }
}
