use super::*;
use crate::rules::{DAYS_PER_MONTH, civic as coverage, economy::PERCENT, market::*, transport};
use crate::{
    Commodity,
    energy::{TradeStock, exchange},
    save::Element,
};

pub(super) struct Market {
    pub rules: [crate::CommodityRule; Commodity::ALL.len()],
    labor: i32,
    working_days: i32,
    busy: i32,
    fill: i32,
    burning: bool,
    fire_until: u64,
    next_frame: u64,
    frame: i32,
}
impl Default for Market {
    fn default() -> Self {
        Self {
            rules: Commodity::ALL.map(|c| BuildingKind::Market.metadata().rule(c)),
            labor: MIN_LABOR,
            working_days: 0,
            busy: 0,
            fill: 0,
            burning: false,
            fire_until: 0,
            next_frame: 0,
            frame: -1,
        }
    }
}
impl Market {
    pub(super) fn load(&mut self, n: &Element) -> io::Result<()> {
        for c in Commodity::ALL {
            let r = &mut self.rules[c as usize];
            if r.max_load > 0 {
                r.take = n.child_i32(&format!("take_{}", c.save_name()), i32::from(r.take))? != 0;
                r.give = n.child_i32(&format!("give_{}", c.save_name()), i32::from(r.give))? != 0;
            }
        }
        Ok(())
    }
    pub(super) fn store(&self, n: &mut Element) {
        for c in Commodity::ALL {
            let r = self.rules[c as usize];
            if r.max_load > 0 {
                n.set_child(&format!("take_{}", c.save_name()), i32::from(r.take));
                n.set_child(&format!("give_{}", c.save_name()), i32::from(r.give));
            }
        }
    }
    pub(super) fn evacuate(&mut self, active: bool) {
        for r in &mut self.rules {
            if r.max_load > 0 {
                r.give = true;
                r.take = !active;
            }
        }
    }
    pub(super) fn animate(&mut self, time: u64, choices: &mut Decisions) -> io::Result<()> {
        if std::mem::take(&mut self.burning) {
            self.fire_until = time + FIRE_VISIBLE_MS;
        }
        if time >= self.fire_until {
            self.frame = -1;
        } else if time >= self.next_frame {
            let speed = i64::from(coverage::FIRE_MIN_FRAME_MS + choices.c_modulo(coverage::FIRE_FRAME_VARIATION_MS)?);
            let quantum = i64::from(coverage::ANIMATION_QUANTUM_MS);
            self.next_frame = time.saturating_add_signed((speed + quantum / 2) / quantum * quantum - quantum / 2);
            self.frame = (self.frame + 1) % coverage::FIRE_FRAMES;
        }
        Ok(())
    }
}

impl City {
    /// Selects which directions a market may trade this commodity with distant partners.
    pub fn set_market_trade(&mut self, p: Point, c: Commodity, buy: bool, sell: bool) -> bool {
        let Some(id) = self.buildings.at(p) else {
            return false;
        };
        let Operation::Market(m) = &mut self.buildings.get_mut(id).unwrap().operation else {
            return false;
        };
        let rule = &mut m.rules[c as usize];
        if rule.max_load == 0 {
            return false;
        }
        rule.take = buy;
        rule.give = sell;
        true
    }

    pub(super) fn update_market(&mut self, id: BuildingId, origin: Point) -> io::Result<()> {
        let b = self.buildings.get(id).unwrap().1;
        let Operation::Market(m) = &b.operation else {
            unreachable!()
        };
        let rules = m.rules;
        let labor = m.labor;
        let partners = b.partners.clone();
        let mut fill = 0;
        let mut count = 0;
        let mut eligible = Vec::with_capacity(partners.len());
        for c in Commodity::ALL {
            let i = c as usize;
            let rule = rules[i];
            let b = self.buildings.get(id).unwrap().1;
            if (!rule.take && !rule.give)
                || (b.stock[Commodity::Labor as usize] < labor && c != Commodity::Labor)
            {
                continue;
            }
            let mut center = TradeStock::new(rule.max_load, rule.take, rule.give);
            center.amount = b.stock[i];
            let ratio = i64::from(center.amount) * i64::from(transport::QUANTA) / i64::from(rule.max_load);
            fill += ratio as i32;
            count += 1;
            let mut sum = i64::from(center.amount);
            let mut cap = i64::from(rule.max_load);
            eligible.clear();
            for &p in &partners {
                let peer = self.buildings.get(p).unwrap().1;
                let r = peer.kind.metadata().rule(c);
                if r.max_load == 0 {
                    continue;
                }
                if !peer.evacuating {
                    let density = i64::from(peer.stock[i]) * i64::from(transport::QUANTA) / i64::from(r.max_load);
                    if (density > ratio && !(rule.take && r.give))
                        || (density < ratio && !(rule.give && r.take))
                    {
                        continue;
                    }
                    cap += i64::from(r.max_load);
                }
                eligible.push(p);
                sum += i64::from(peer.stock[i]);
            }
            for &p in &eligible {
                let peer = self.buildings.get_mut(p).unwrap();
                let r = peer.kind.metadata().rule(c);
                let mut stock = TradeStock::new(r.max_load, r.take, r.give);
                stock.amount = peer.stock[i];
                stock.evacuating = peer.evacuating;
                exchange(&mut center, &mut stock, sum * i64::from(transport::QUANTA) / cap);
                if stock.amount > peer.stock[i] && !peer.evacuating {
                    self.economy.tax_receipt(c, stock.amount - peer.stock[i]);
                }
                peer.stock[i] = stock.amount;
            }
            self.buildings.get_mut(id).unwrap().stock[i] = center.amount;
        }
        let b = self.buildings.get_mut(id).unwrap();
        let Operation::Market(m) = &mut b.operation else {
            unreachable!()
        };
        m.fill = fill;
        if b.stock[Commodity::Labor as usize] >= labor {
            b.production.change(&mut b.stock, Commodity::Labor, -labor);
            self.economy.taxable_labor += labor;
            m.working_days += 1;
        }
        let waste_cap = rules[Commodity::Waste as usize].max_load;
        let mut ignite = false;
        if self.day % WASTE_REST_PERIOD !=  0 && b.stock[Commodity::Waste as usize] >= WASTE_THRESHOLD_PERCENT * waste_cap / PERCENT {
            m.burning = true;
            self.tiles[((origin.y + 1) * self.side + origin.x + 1) as usize].pollution +=
                waste_cap / POLLUTION_DIVISOR;
            b.production
                .change(&mut b.stock, Commodity::Waste, -(WASTE_BURN_NUMERATOR * waste_cap / WASTE_BURN_DENOMINATOR));
            ignite = self.decisions.bernoulli(
                "market.waste-fire",
                transport::UNCONTROLLED_FIRE * f64::from(b.kind.metadata().fire_chance) / f64::from(PERCENT),
            )? && self.tiles[(origin.y * self.side + origin.x) as usize].coverage
                & coverage::FIRE
                == 0;
        }
        if self.day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1 {
            b.production.close_month();
            m.busy = std::mem::take(&mut m.working_days);
        }
        if self.day % STAFF_REVIEW_PERIOD == STAFF_REVIEW_DAY {
            m.fill = if count > 0 {
                PERCENT * fill / (count * transport::QUANTA)
            } else {
                0
            };
            m.labor = STAFF_LEVELS.iter().find(|(minimum, _)| m.fill >= *minimum)
                .map_or(MIN_LABOR, |&(_, labor)| labor);
        }
        if self.day % coverage::COVER_PERIOD == coverage::COVER_DAY {
            self.mark_coverage(origin, BuildingKind::Market.metadata().range, coverage::MARKET_CHECK);
        }
        if ignite {
            self.ignite_building(id, origin)?;
        }
        Ok(())
    }

    pub(super) fn ignite_building(&mut self, id: BuildingId, origin: Point) -> io::Result<()> {
        let kind = self.buildings.get(id).unwrap().1.kind;
        self.torch_building(id, origin)?;
        self.events.push_back(CityEvent::FireStarted(origin, kind));
        Ok(())
    }
}
