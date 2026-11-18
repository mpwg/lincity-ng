use super::Stock;
use crate::Commodity::{Coal, Goods, HighVoltage, Labor, LowVoltage, Ore, Steel, Waste};
use crate::rules::{
    DAYS_PER_MONTH, MAX_TECH_LEVEL,
    industry::{LOAD_COAL, heavy, light},
};

/// Pollution filters are fixed by the technology at construction time.
fn emissions(tech: i32, gross: f64, capacity: i32, stock: &mut Stock<'_>) -> i32 {
    let filter =
        ((f64::from(tech) - f64::from(MAX_TECH_LEVEL)) / f64::from(MAX_TECH_LEVEL)).clamp(0., 1.);
    let secondary = ((f64::from(tech) - 2.0 * f64::from(MAX_TECH_LEVEL))
        / f64::from(MAX_TECH_LEVEL))
    .clamp(0., 1.);
    stock.change(Waste, (gross * filter * (1. - secondary)) as i32);
    (gross * (1. - filter)) as i32 + stock.cap(Waste, capacity)
}

#[derive(Clone, Debug)]
pub struct LightIndustry {
    pub tech: i32,
    pub busy: i32,
    goods_this_month: i32,
    goods_today: i32,
    next_animation: u64,
    pub resource: &'static str,
    pub smoke: [i32; 2],
}

impl LightIndustry {
    pub const SMOKE_OFFSETS: [(i32, i32); 2] = [(-113, -210), (-84, -198)];
    pub fn new(tech: i32) -> Self {
        Self {
            tech,
            busy: 0,
            goods_this_month: 0,
            goods_today: 0,
            next_animation: 0,
            resource: "IndustryLight",
            smoke: [-1; 2],
        }
    }
    pub fn update(&mut self, day: u32, stock: &mut Stock<'_>) -> i32 {
        if day.is_multiple_of(DAYS_PER_MONTH as u32) {
            stock.counts.close_month();
            self.busy = std::mem::take(&mut self.goods_this_month) / light::MAX_OUTPUT;
        }
        self.goods_today = 0;
        if stock[Labor] < light::BASE_LABOR
            || stock[Ore] < light::ORE
            || stock[Goods] > light::GOODS_CAPACITY - light::GOODS
        {
            return 0;
        }
        stock.change(Labor, -light::BASE_LABOR);
        stock.change(Ore, -light::ORE);
        let mut made = light::GOODS;
        let pollution = emissions(
            self.tech,
            light::POLLUTION_PER_GOOD * f64::from(made),
            light::WASTE_CAPACITY,
            stock,
        );
        if stock[Labor] >= light::STEEL_LABOR
            && stock[Steel] >= light::STEEL
            && stock[Goods] + light::STEEL_MULTIPLIER * made <= light::GOODS_CAPACITY
        {
            stock.change(Labor, -light::STEEL_LABOR);
            stock.change(Steel, -light::STEEL);
            made *= light::STEEL_MULTIPLIER;
        }
        let power_per_good =
            (stock[LowVoltage] + 2 * stock[HighVoltage]) / (light::POWER_MULTIPLIER * made);
        if power_per_good >= light::POWER_PER_GOOD
            && stock[Ore] >= light::ORE
            && stock[Goods] + light::POWER_MULTIPLIER * made <= light::GOODS_CAPACITY
        {
            made *= light::POWER_MULTIPLIER;
            stock.change(Ore, -light::ORE);
            if 2 * stock[HighVoltage] > stock[LowVoltage] {
                let high = (light::POWER_PER_GOOD * (made / 2)).min(stock[HighVoltage]);
                stock.change(HighVoltage, -high);
                stock.change(LowVoltage, -(light::POWER_PER_GOOD * (made / 2) - high) * 2);
            } else {
                let low = (light::POWER_PER_GOOD * made).min(stock[LowVoltage]);
                stock.change(LowVoltage, -low);
                stock.change(HighVoltage, -((light::POWER_PER_GOOD * made - low) / 2));
            }
        }
        stock.change(Goods, made);
        self.goods_this_month += made;
        self.goods_today = made;
        pollution
    }

    pub fn animate<E>(
        &mut self,
        millis: u64,
        mut random: impl FnMut() -> Result<u32, E>,
    ) -> Result<(), E> {
        if millis < self.next_animation {
            return Ok(());
        }
        self.next_animation = millis + 250;
        self.resource = match self.busy {
            81.. => "IndustryLightH",
            56.. => "IndustryLightM",
            26.. => "IndustryLightL",
            _ => "IndustryLight",
        };
        let active = usize::from(self.busy > 10) + usize::from(self.busy > 70);
        for (i, frame) in self.smoke.iter_mut().enumerate() {
            if i >= active {
                *frame = -1;
            } else if *frame < 0
                || random()?.is_multiple_of(crate::rules::power::SMOKE_RANDOM_PERIOD)
            {
                *frame = (random()? % 10) as i32;
            } else if self.goods_today != 0 {
                *frame = (*frame + 1) % 10;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct HeavyIndustry {
    pub tech: i32,
    pub busy: i32,
    steel_this_month: i32,
    next_animation: u64,
    pub frame: i32,
    pub resource: &'static str,
}

impl HeavyIndustry {
    pub fn new(tech: i32) -> Self {
        Self {
            tech,
            busy: 0,
            steel_this_month: 0,
            next_animation: 0,
            frame: 0,
            resource: "IndustryHigh",
        }
    }
    pub fn update(&mut self, day: u32, stock: &mut Stock<'_>) -> i32 {
        let mut pollution = 0;
        if stock[Labor] >= heavy::LABOR + LOAD_COAL
            && stock[Ore] >= heavy::BATCH_ORE
            && stock[Steel] <= heavy::STEEL_CAPACITY - heavy::OUTPUT
        {
            let high = stock[HighVoltage].min(heavy::OUTPUT * heavy::POWER_PER_STEEL / 2);
            let remaining = heavy::OUTPUT - high / (heavy::POWER_PER_STEEL / 2);
            let low = stock[LowVoltage].min(remaining * heavy::POWER_PER_STEEL);
            let remaining = remaining - low / heavy::POWER_PER_STEEL;
            let coal = stock[Coal].min(remaining * heavy::COAL_PER_STEEL);
            let made = if remaining == coal / heavy::COAL_PER_STEEL {
                stock.change(HighVoltage, -high);
                stock.change(LowVoltage, -low);
                stock.change(Coal, -coal);
                if coal != 0 {
                    stock.change(Labor, -LOAD_COAL);
                }
                heavy::OUTPUT
            } else {
                heavy::OUTPUT / heavy::LOW_POWER_DIVISOR
            };
            stock.change(Labor, -heavy::LABOR);
            stock.change(Ore, -heavy::BATCH_ORE);
            stock.change(Steel, made);
            self.steel_this_month += made;
            pollution = emissions(
                self.tech,
                f64::from(made) * heavy::POLLUTION_PER_STEEL,
                heavy::WASTE_CAPACITY,
                stock,
            );
        }
        if day % DAYS_PER_MONTH as u32 == (DAYS_PER_MONTH - 1) as u32 {
            stock.counts.close_month();
            self.busy = std::mem::take(&mut self.steel_this_month) * heavy::ORE_PER_STEEL
                / heavy::BATCH_ORE;
        }
        pollution
    }

    pub fn animate(&mut self, millis: u64) {
        if millis < self.next_animation {
            return;
        }
        self.next_animation = millis + 250;
        self.resource = match self.busy {
            81.. => "IndustryHighH",
            31.. => "IndustryHighM",
            1.. => "IndustryHighL",
            _ => "IndustryHigh",
        };
        self.frame = if self.busy > 0 {
            (self.frame + 1) % 8
        } else {
            0
        };
    }
}
