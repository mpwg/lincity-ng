use super::{Activity, Stock};
use crate::Commodity::{Labor, LowVoltage, Ore, Steel, Waste};

#[derive(Clone, Debug)]
pub struct Recycling {
    pub tech: i32,
    pub make_ore: i32,
    pub make_steel: i32,
    pub activity: Activity,
}

impl Recycling {
    pub const RUNNING_COST: i32 = 3;
    pub fn new(tech: i32) -> Self {
        let efficiency = (500 * (10 + (50 * i64::from(tech)) / 1_000_000) / 100).min(400) as i32;
        Self {
            tech,
            make_ore: efficiency,
            make_steel: efficiency / 50,
            activity: Activity::default(),
        }
    }

    /// The city attempts the daily operating expense even when supplies are insufficient.
    pub fn update(&mut self, day: u32, stock: &mut Stock<'_>, paid: bool) {
        if paid && stock[Waste] >= 500 && stock[LowVoltage] >= 250 && stock[Labor] >= 34 {
            stock.change(Labor, -34);
            stock.change(LowVoltage, -250);
            stock.change(Waste, -500);
            self.activity.working_days += 1;
            stock.change(Ore, self.make_ore);
            stock.change(Steel, self.make_steel);
            stock.cap(Ore, 8000);
            stock.cap(Steel, 160);
        }
        self.activity.close_day(day, stock);
        if stock[Waste] > 9000 {
            stock.change(Waste, -50);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Landfill {
    pub buried: i32,
    pub degradation: i32,
    pub activity: Activity,
}

impl Landfill {
    /// Returns true when burial invalidates the city's mining sustainability flag.
    pub fn update(&mut self, day: u32, stock: &mut Stock<'_>) -> bool {
        self.degradation += self.buried;
        self.buried -= self.degradation / 240_000;
        self.degradation %= 240_000;
        let percent = stock[Waste] * 100 / 4000;
        let bury = stock[Waste] >= 200 && percent > 20 && self.buried + 200 <= 10_000_000;
        if bury {
            stock.change(Waste, -200);
            self.buried += 200;
            self.activity.working_days += 1;
        } else if stock[Waste] + 200 <= 4000 && percent < 20 && self.buried > 0 {
            let recovered = 200.min(self.buried);
            stock.change(Waste, recovered);
            self.buried -= recovered;
            self.activity.working_days += 1;
        }
        self.activity.close_day(day, stock);
        bury
    }
    pub fn frame(&self) -> i32 {
        let level = (self.buried / 3 * 22) / 10_000_000;
        level + i32::from(self.buried > 0 && level < 8)
    }
    pub fn can_bulldoze(&self) -> bool {
        self.buried <= 0
    }
}
