use super::{City, CityEvent};
use crate::rules::{PERCENT, economy::*};
use crate::{Commodity, save::Element, stats::Stat};
use std::io;

pub(super) struct Economy {
    pub coal_survey_done: bool,
    pub transport_cost: i32,
    pub import_rate: i32,
    income_rate: i32,
    coal_rate: i32,
    goods_rate: i32,
    dole_rate: i32,
    pub taxable_labor: i32,
    pub taxable_coal: i32,
    pub taxable_goods: i32,
    pub taxable_exports: i32,
    pub trade: [(bool, bool); Commodity::ALL.len()],
}

impl Default for Economy {
    fn default() -> Self {
        Self {
            coal_survey_done: false,
            import_rate: IMPORT_RATE,
            transport_cost: crate::rules::transport::DEFAULT_COST,
            income_rate: INCOME_RATE,
            coal_rate: COAL_RATE,
            goods_rate: GOODS_RATE,
            dole_rate: DOLE_RATE,
            taxable_labor: 0,
            taxable_coal: 0,
            taxable_goods: 0,
            taxable_exports: 0,
            trade: [(true, true); Commodity::ALL.len()],
        }
    }
}

impl Economy {
    pub(super) fn tax_receipt(&mut self, c: Commodity, amount: i32) {
        match c {
            Commodity::Labor => self.taxable_labor += amount,
            Commodity::Goods => self.taxable_goods += amount,
            Commodity::Coal => self.taxable_coal += amount,
            _ => {}
        }
    }
    pub(super) fn load(&mut self, globals: &Element) -> io::Result<()> {
        self.coal_survey_done = globals.child_i32("coal_survey_done", 0)? != 0;
        if let Some(n) = globals.child("money_rates") {
            self.transport_cost =
                n.child_i32("transport_cost_rate", crate::rules::transport::DEFAULT_COST)?;
            self.import_rate = n.child_i32("import_cost_rate", IMPORT_RATE)?;
            self.income_rate = n.child_i32("income_tax_rate", INCOME_RATE)?;
            self.coal_rate = n.child_i32("coal_tax_rate", COAL_RATE)?;
            self.goods_rate = n.child_i32("goods_tax_rate", GOODS_RATE)?;
            self.dole_rate = n.child_i32("dole_rate", DOLE_RATE)?;
        }
        if let Some(n) = globals.child("taxable") {
            self.taxable_labor = n.child_i32("labor", 0)?;
            self.taxable_coal = n.child_i32("coal", 0)?;
            self.taxable_goods = n.child_i32("goods", 0)?;
            self.taxable_exports = n.child_i32("trade_ex", 0)?;
        }
        if let Some(n) = globals.child("tradeRules") {
            for c in Commodity::ALL {
                let name = c.save_name();
                self.trade[c as usize] = (
                    n.child_i32(&format!("import_{name}_enable"), 1)? != 0,
                    n.child_i32(&format!("export_{name}_enable"), 1)? != 0,
                );
            }
        }
        Ok(())
    }
    pub(super) fn store(&self, globals: &mut Element) {
        globals.set_child("coal_survey_done", i32::from(self.coal_survey_done));
        let t = globals.section("taxable");
        for (name, n) in [
            ("labor", self.taxable_labor),
            ("coal", self.taxable_coal),
            ("goods", self.taxable_goods),
            ("trade_ex", self.taxable_exports),
        ] {
            t.set_child(name, n);
        }
        let rules = globals.section("tradeRules");
        for c in Commodity::ALL {
            let (buy, sell) = self.trade[c as usize];
            rules.set_child(&format!("import_{}_enable", c.save_name()), i32::from(buy));
            rules.set_child(&format!("export_{}_enable", c.save_name()), i32::from(sell));
        }
    }
}

pub(super) fn balance_after_expense(balance: i32, amount: i32) -> Option<i32> {
    balance
        .checked_sub(amount)
        .filter(|value| *value >= crate::rules::CREDIT_LIMIT)
}

pub(super) fn expense(
    balance: &mut i32,
    account: &mut Stat,
    events: &mut std::collections::VecDeque<CityEvent>,
    amount: i32,
) -> bool {
    let Some(new) = balance_after_expense(*balance, amount) else {
        return false;
    };
    if *balance > 0 && new <= 0 {
        events.push_back(CityEvent::OutOfMoney);
    }
    *balance = new;
    account.acc = account.acc.saturating_add(amount);
    true
}

impl City {
    /// Whether the city has paid for its coal reserve survey.
    pub fn coal_survey_done(&self) -> bool {
        self.economy.coal_survey_done
    }

    /// Surveys coal reserves once, recording the expense in the current fiscal year.
    ///
    /// Returns an out-of-money error if the survey would exceed the credit limit.
    pub fn survey_coal(&mut self) -> Result<(), super::ActionError> {
        if !self.economy.coal_survey_done {
            if !expense(
                &mut self.balance,
                &mut self.stats.expenses.coal_survey,
                &mut self.events,
                COAL_SURVEY_COST,
            ) {
                return Err(super::ActionError::OutOfMoney);
            }
            self.economy.coal_survey_done = true;
        }
        Ok(())
    }

    pub(super) fn pay_monthly_bills(&mut self) {
        let pop = &self.stats.population;
        expense(
            &mut self.balance,
            &mut self.stats.expenses.deaths,
            &mut self.events,
            pop.unnat_deaths_m.acc * DEATH_EXPENSE,
        );
        expense(
            &mut self.balance,
            &mut self.stats.expenses.unemployment,
            &mut self.events,
            pop.unemployed_m.acc * self.economy.dole_rate / PERCENT,
        );
    }

    pub(super) fn settle_year(&mut self) -> io::Result<()> {
        let e = &mut self.economy;
        let goods = e.taxable_goods * e.goods_rate / PERCENT;
        let goods = goods
            + ((goods * e.goods_rate) as f32 * self.tech_level as f32 / GOODS_TECH_SCALE) as i32;
        let discount: i32 = EXPORT_BRACKETS
            .into_iter()
            .map(|threshold| (e.taxable_exports - threshold).max(0) / EXPORT_DISCOUNT_DIVISOR)
            .sum();
        let income = &mut self.stats.income;
        for (amount, account) in [
            (
                e.taxable_labor * e.income_rate / PERCENT,
                &mut income.income_tax,
            ),
            (
                e.taxable_coal * e.coal_rate / COAL_TAX_DIVISOR,
                &mut income.coal_tax,
            ),
            (goods, &mut income.goods_tax),
            (e.taxable_exports - discount, &mut income.export_tax),
        ] {
            self.balance = self.balance.saturating_add(amount);
            account.acc = account.acc.saturating_sub(amount);
        }
        e.taxable_labor = 0;
        e.taxable_coal = 0;
        e.taxable_goods = 0;
        e.taxable_exports = 0;
        if self.balance < 0 {
            let interest =
                (-(self.balance / INTEREST_PRINCIPAL_UNIT) * INTEREST_PER_UNIT).min(INTEREST_CAP);
            expense(
                &mut self.balance,
                &mut self.stats.expenses.interest,
                &mut self.events,
                interest,
            );
        }
        self.balance = self.balance.clamp(
            crate::rules::CREDIT_LIMIT,
            crate::rules::YEAR_END_CASH_CEILING,
        );
        for (take, give) in &mut e.trade {
            for enabled in [take, give] {
                *enabled ^= self.decisions.c_modulo(if *enabled {
                    CLOSE_TRADE_ODDS
                } else {
                    OPEN_TRADE_ODDS
                })? == 0;
            }
        }
        Ok(())
    }
}
