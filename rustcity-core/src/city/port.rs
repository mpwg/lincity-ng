use super::{BuildingId, City, Operation, finance::expense};
use crate::rules::DAYS_PER_MONTH;
use crate::rules::{PERCENT, economy::*};
use crate::{BuildingKind, Commodity, save::Element};
use std::io;

#[derive(Default)]
pub(super) struct Port {
    rules: [(bool, bool); Commodity::ALL.len()],
    pub(super) tech_made: i32,
    pence: i32,
    working_days: i32,
    monthly_imports: i32,
    monthly_exports: i32,
    pub busy: i32,
    pub last_imports: i32,
    pub last_exports: i32,
}

impl Port {
    pub(super) fn policy(&self, c: Commodity) -> Option<crate::TradePolicy> {
        let (buy, sell) = self.rules[c as usize];
        PORT_PRICES
            .iter()
            .any(|&(kind, _)| kind == c)
            .then_some(crate::TradePolicy { buy, sell })
    }
    pub(super) fn load(&mut self, n: &Element) -> io::Result<()> {
        self.tech_made = n.child_i32("tech_made", 0)?;
        for (c, _) in PORT_PRICES {
            let name = c.save_name();
            self.rules[c as usize] = (
                n.child_i32(&format!("take_{name}"), 0)? != 0,
                n.child_i32(&format!("give_{name}"), 0)? != 0,
            );
        }
        Ok(())
    }
    pub(super) fn store(&self, n: &mut Element) {
        n.set_child("tech_made", self.tech_made);
        super::persistence::store_trade_rules(
            n,
            PORT_PRICES.map(|(c, _)| {
                let (take, give) = self.rules[c as usize];
                (c, take, give)
            }),
        );
    }
}

impl City {
    /// Sets the foreign-trade policy for the port covering this coordinate.
    /// Selecting both directions suspends foreign trade in this commodity.
    pub fn set_port_trade(
        &mut self,
        point: super::Point,
        c: Commodity,
        buy: bool,
        sell: bool,
    ) -> bool {
        if !PORT_PRICES.iter().any(|(kind, _)| *kind == c) {
            return false;
        }
        let Some(id) = self.buildings.at(point) else {
            return false;
        };
        let Some(b) = self.buildings.get_mut(id) else {
            return false;
        };
        let Operation::Port(port) = &mut b.operation else {
            return false;
        };
        port.rules[c as usize] = (buy && !sell, sell && !buy);
        true
    }

    pub(super) fn update_port(&mut self, id: BuildingId) {
        let (point, _) = self.buildings.get(id).unwrap();
        let b = self.buildings.get_mut(id).unwrap();
        let Operation::Port(port) = &mut b.operation else {
            unreachable!()
        };
        let (mut imports, mut exports) = (0, 0);
        if b.stock[Commodity::Labor as usize] >= PORT_LABOR {
            for (c, price) in PORT_PRICES {
                let i = c as usize;
                let cap = BuildingKind::Port.metadata().rule(c).max_load;
                let (take, give) = self.economy.trade[i];
                match port.rules[i] {
                    (true, false) if take => {
                        let amount = (cap - b.stock[i]) / SHIPMENT_DIVISOR;
                        let cost = amount * price;
                        if amount >= cap / MIN_SHIPMENT_DIVISOR
                            && expense(
                                &mut self.balance,
                                &mut self.stats.expenses.import,
                                &mut self.events,
                                cost * self.economy.import_rate / PERCENT,
                            )
                        {
                            imports += cost;
                            b.production.change(&mut b.stock, c, amount);
                        }
                    }
                    (false, true) if give => {
                        let amount = b.stock[i] / SHIPMENT_DIVISOR;
                        if amount >= cap / MIN_SHIPMENT_DIVISOR {
                            exports += amount * price;
                            b.production.change(&mut b.stock, c, -amount);
                        }
                    }
                    _ => {}
                }
            }
            if imports != 0 || exports != 0 {
                b.production
                    .change(&mut b.stock, Commodity::Labor, -PORT_LABOR);
                self.tiles[(point.y * self.side + point.x) as usize].pollution += 1;
                self.stats.sustainability.trade_flag = false;
                self.tech_level += 1 + i32::from(imports != 0 && exports != 0);
                port.tech_made += 1;
                port.working_days += 1;
            }
        }
        port.monthly_imports += imports;
        port.monthly_exports += exports;
        if self.day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1 {
            b.production.close_month();
            port.busy = std::mem::take(&mut port.working_days);
            port.last_imports = std::mem::take(&mut port.monthly_imports);
            port.last_exports = std::mem::take(&mut port.monthly_exports);
        }
        let gross = exports + port.pence;
        self.economy.taxable_exports += gross / PENCE_PER_UNIT;
        port.pence = gross % PENCE_PER_UNIT;
    }
}
