use super::{Activity, Stock};
use crate::Commodity::{Labor, LowVoltage, Ore, Steel, Waste};
use crate::rules::{
    MAX_TECH_LEVEL,
    industry::{recycle, tip},
};

#[derive(Clone, Debug)]
pub struct Recycling {
    pub make_ore: i32,
    pub make_steel: i32,
    pub activity: Activity,
}

impl Recycling {
    pub const RUNNING_COST: i32 = recycle::RUNNING_COST;
    pub fn new(tech: i32) -> Self {
        let efficiency = (i64::from(recycle::WASTE)
            * (10 + (50 * i64::from(tech)) / i64::from(MAX_TECH_LEVEL))
            / 100)
            .min(i64::from(recycle::WASTE * 8 / 10)) as i32;
        Self {
            make_ore: efficiency,
            make_steel: efficiency / 50,
            activity: Activity::default(),
        }
    }

    /// The city attempts the daily operating expense even when supplies are insufficient.
    pub fn update(&mut self, day: u32, stock: &mut Stock<'_>, paid: bool) {
        if paid
            && stock[Waste] >= recycle::WASTE
            && stock[LowVoltage] >= recycle::POWER
            && stock[Labor] >= recycle::LABOR
        {
            stock.change(Labor, -recycle::LABOR);
            stock.change(LowVoltage, -recycle::POWER);
            stock.change(Waste, -recycle::WASTE);
            self.activity.working_days += 1;
            stock.change(Ore, self.make_ore);
            stock.change(Steel, self.make_steel);
            stock.cap(Ore, recycle::MAX_ORE);
            stock.cap(Steel, recycle::MAX_STEEL);
        }
        self.activity.close_day(day, stock);
        if stock[Waste] > recycle::STOCKED_WASTE * 9 / 10 {
            stock.change(Waste, -recycle::BURN);
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
        self.buried -= self.degradation / tip::DEGRADE_DAYS;
        self.degradation %= tip::DEGRADE_DAYS;
        let percent = stock[Waste] * 100 / tip::INBOX;
        let bury = stock[Waste] >= tip::BATCH
            && percent > tip::TARGET_PERCENT
            && self.buried + tip::BATCH <= tip::CAPACITY;
        if bury {
            stock.change(Waste, -tip::BATCH);
            self.buried += tip::BATCH;
            self.activity.working_days += 1;
        } else if stock[Waste] + tip::BATCH <= tip::INBOX
            && percent < tip::TARGET_PERCENT
            && self.buried > 0
        {
            let recovered = tip::BATCH.min(self.buried);
            stock.change(Waste, recovered);
            self.buried -= recovered;
            self.activity.working_days += 1;
        }
        self.activity.close_day(day, stock);
        bury
    }
    pub fn frame(&self) -> i32 {
        let level = (self.buried / 3 * 22) / tip::CAPACITY;
        level + i32::from(self.buried > 0 && level < 8)
    }
    pub fn can_bulldoze(&self) -> bool {
        self.buried <= 0
    }
}
