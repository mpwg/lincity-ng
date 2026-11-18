//! Mining, industrial production and waste handling.
use crate::{Commodity, power::ProductionCounters};
use std::ops::Index;

mod industry;
mod mines;
pub use mines::{CoalMine, MineAction, Mineral, MiningTile, OreMine};
mod waste;
pub use industry::{HeavyIndustry, LightIndustry};
pub use waste::{Landfill, Recycling};

/// One building's existing inventory and its local production reports.
pub struct Stock<'a> {
    pub amount: &'a mut [i32; 10],
    pub counts: &'a mut ProductionCounters,
}

impl<'a> Stock<'a> {
    pub fn new(amount: &'a mut [i32; 10], counts: &'a mut ProductionCounters) -> Self {
        Self { amount, counts }
    }

    fn change(&mut self, commodity: Commodity, delta: i32) {
        self.counts.change(self.amount, commodity, delta);
    }
    fn cap(&mut self, commodity: Commodity, capacity: i32) -> i32 {
        let excess = (self[commodity] - capacity).max(0);
        self.change(commodity, -excess);
        excess
    }
}

impl Index<Commodity> for Stock<'_> {
    type Output = i32;
    fn index(&self, commodity: Commodity) -> &Self::Output {
        &self.amount[commodity as usize]
    }
}

#[derive(Debug, Clone, Default)]
pub struct Activity {
    working_days: i32,
    pub busy: i32,
}

impl Activity {
    fn close_day(&mut self, day: u32, stock: &mut Stock<'_>) {
        if day % 100 == 99 {
            stock.counts.close_month();
            self.busy = std::mem::take(&mut self.working_days);
        }
    }
}

#[cfg(test)]
mod tests;
