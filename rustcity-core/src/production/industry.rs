use super::Stock;
use crate::Commodity::{Coal, Goods, HighVoltage, Labor, LowVoltage, Ore, Steel, Waste};

/// Pollution filters are fixed by the technology at construction time.
fn emissions(tech: i32, gross: f64, capacity: i32, stock: &mut Stock<'_>) -> i32 {
    let filter = ((f64::from(tech) - 1_000_000.0) / 1_000_000.0).clamp(0., 1.);
    let secondary = ((f64::from(tech) - 2_000_000.0) / 1_000_000.0).clamp(0., 1.);
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
        if day.is_multiple_of(100) {
            stock.counts.close_month();
            self.busy = std::mem::take(&mut self.goods_this_month) / 1800;
        }
        self.goods_today = 0;
        if stock[Labor] < 40 || stock[Ore] < 125 || stock[Goods] > 36_000 - 225 {
            return 0;
        }
        stock.change(Labor, -40);
        stock.change(Ore, -125);
        let mut made = 225;
        let pollution = emissions(self.tech, 0.05 * f64::from(made), 1800, stock);
        if stock[Labor] >= 19 && stock[Steel] >= 12 && stock[Goods] + 2 * made <= 36000 {
            stock.change(Labor, -19);
            stock.change(Steel, -12);
            made *= 2;
        }
        let power_per_good = (stock[LowVoltage] + 2 * stock[HighVoltage]) / (4 * made);
        if power_per_good >= 10 && stock[Ore] >= 125 && stock[Goods] + 4 * made <= 36000 {
            made *= 4;
            stock.change(Ore, -125);
            if 2 * stock[HighVoltage] > stock[LowVoltage] {
                let high = (10 * (made / 2)).min(stock[HighVoltage]);
                stock.change(HighVoltage, -high);
                stock.change(LowVoltage, -(10 * (made / 2) - high) * 2);
            } else {
                let low = (10 * made).min(stock[LowVoltage]);
                stock.change(LowVoltage, -low);
                stock.change(HighVoltage, -((10 * made - low) / 2));
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
            } else if *frame < 0 || random()?.is_multiple_of(1600) {
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
        if stock[Labor] >= 46 && stock[Ore] >= 612 && stock[Steel] <= 684 {
            let high = stock[HighVoltage].min(3600);
            let remaining = 36 - high / 100;
            let low = stock[LowVoltage].min(remaining * 200);
            let remaining = remaining - low / 200;
            let coal = stock[Coal].min(remaining * 2);
            let made = if remaining == coal / 2 {
                stock.change(HighVoltage, -high);
                stock.change(LowVoltage, -low);
                stock.change(Coal, -coal);
                if coal != 0 {
                    stock.change(Labor, -18);
                }
                36
            } else {
                7
            };
            stock.change(Labor, -28);
            stock.change(Ore, -612);
            stock.change(Steel, made);
            self.steel_this_month += made;
            pollution = emissions(self.tech, f64::from(made) * 0.25, 180, stock);
        }
        if day % 100 == 99 {
            stock.counts.close_month();
            self.busy = std::mem::take(&mut self.steel_this_month) * 17 / 612;
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
