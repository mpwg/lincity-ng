//! Housing, research and municipal services.
use super::*;
use crate::Commodity::*;
use crate::save::Element;

mod fire;
mod residence;
mod shanty;
use fire::FireState;
pub(super) use fire::WasteFire;
use residence::{Neighborhood, Residence};
#[cfg(test)]
mod tests;

use crate::rules::civic as rules;
use rules::*;
pub(super) use rules::{
    COVER_DAY, COVER_MASK, COVER_PERIOD, CRICKET, FIRE, HEALTH, MARKET_CHECK, SHANTY_ADJUST_DAY,
    SHANTY_DEMOLITION_FIRE_AGE,
};
#[derive(Default)]
pub(super) struct Service {
    active: bool,
    days: i32,
    supplied: i32,
    working: i32,
    busy: i32,
    animate: bool,
    next_frame: u64,
    frame: i32,
}
#[derive(Default)]
pub(super) struct Education {
    tech: i32,
    working: i32,
    busy: i32,
    frame: i32,
    swing: i32,
    next_frame: u64,
    stop: u64,
}
#[derive(Default)]
pub(super) struct Monument {
    working: i32,
    busy: i32,
    frame: i32,
    labor: i32,
    completion: i32,
    tech: i32,
    tail: i32,
    pub(super) completed: bool,
}

pub(super) enum Civic {
    Residence(Residence),
    Service(Service),
    Education(Education),
    Monument(Monument),
    Well(i32),
    Park,
    Shanty(WasteFire),
    Fire(FireState),
}

impl Civic {
    pub(super) fn new(kind: BuildingKind) -> Option<Self> {
        use BuildingKind::*;
        Some(match kind {
            ResidenceLowLow | ResidenceMediumLow | ResidenceHighLow | ResidenceLowHigh
            | ResidenceMediumHigh | ResidenceHighHigh => Self::Residence(Default::default()),
            Health | FireStation | Cricket => Self::Service(Default::default()),
            School | University => Self::Education(Default::default()),
            BuildingKind::Monument => Self::Monument(Default::default()),
            Waterwell => Self::Well(0),
            ParkPond | Parkland => Self::Park,
            BuildingKind::Shanty => Self::Shanty(Default::default()),
            BuildingKind::Fire => Self::Fire(Default::default()),
            _ => return None,
        })
    }

    pub(super) fn load(&mut self, node: &Element, flags: u32) -> io::Result<()> {
        match self {
            Self::Fire(f) => f.load(node)?,
            Self::Residence(h) => {
                h.occupants = node.child_i32("local_population", 0)?;
                h.flags = flags & (FED | EMPLOYED | HAD_POWER);
            }
            Self::Service(s) => {
                s.active = node.child_i32("active", 0)? != 0;
                s.days = node.child_i32("daycount", 0)?;
                s.supplied = node.child_i32("covercount", 0)?;
            }
            Self::Education(e) => e.tech = node.child_i32("total_tech_made", 0)?,
            Self::Monument(m) => {
                m.tech = node.child_i32("tech_made", 0)?;
                m.completion = node.child_i32("completion", 0)?;
                m.tail = node.child_i32("tail_off", 0)?;
                m.labor = node.child_i32("labor_consumed", node.child_i32("jobs_consumed", 0)?)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn store(&self, node: &mut Element, flags: &mut u32) {
        match self {
            Self::Fire(f) => f.store(node),
            Self::Residence(h) => {
                node.set_child("local_population", h.occupants);
                *flags = (*flags & !(FED | EMPLOYED | HAD_POWER)) | h.flags;
            }
            Self::Service(s) => {
                node.set_child("active", i32::from(s.active));
                node.set_child("daycount", s.days);
                node.set_child("covercount", s.supplied);
            }
            Self::Education(e) => node.set_child("total_tech_made", e.tech),
            Self::Monument(m) => {
                node.set_child("tech_made", m.tech);
                node.set_child("tail_off", m.tail);
                node.set_child("completion", m.completion);
                node.retain_elements(|e| e.name != "jobs_consumed");
                node.set_child("labor_consumed", m.labor);
                if m.completed {
                    *flags |= COMPLETED_MONUMENT_FLAGS;
                }
            }
            _ => {}
        }
    }

    pub(super) fn animate(
        &mut self,
        kind: BuildingKind,
        time: u64,
        choices: &mut Decisions,
    ) -> io::Result<()> {
        match self {
            Self::Fire(f) => f.animate(time, choices)?,
            Self::Shanty(f) => f.animate(time, WASTE_FIRE_VISIBLE_MS, choices)?,
            Self::Service(s) if s.animate && time >= s.next_frame => {
                let (delay, count) = if kind == BuildingKind::Cricket {
                    (SPORTS_FRAME_MS, SPORTS_FRAMES)
                } else {
                    (STATION_FRAME_MS, STATION_FRAMES)
                };
                s.next_frame = time + delay;
                s.frame += 1;
                if kind == BuildingKind::FireStation && s.frame == STATION_HOLD_FRAME {
                    s.next_frame = time + STATION_HOLD_MS;
                }
                if s.frame >= count {
                    s.animate = false;
                    s.frame = 0;
                }
            }
            Self::Monument(m) => {
                m.frame = if m.completed {
                    0
                } else {
                    m.completion / MONUMENT_PERCENT_PER_FRAME
                }
            }
            Self::Education(e) if kind == BuildingKind::School && time >= e.next_frame => {
                e.next_frame = time + SWING_FRAME_MS;
                if e.frame != 0 {
                    e.swing = (e.swing + 1) % SWING_FRAMES;
                    if (e.swing == 0 || e.swing == SWING_FRAMES / 2)
                        && (time >= e.stop || e.busy == 0)
                    {
                        e.next_frame = time.wrapping_add_signed(
                            SWING_PAUSE_MS - i64::from(SWING_MS_PER_WORKDAY) * i64::from(e.busy),
                        );
                        e.frame = 0;
                        e.swing = -1;
                    }
                } else if e.busy >= SWING_MIN_WORKDAYS {
                    e.frame = 1;
                    e.swing = 0;
                    e.stop = time + SWING_MS_PER_WORKDAY as u64 * e.busy as u64;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn evacuate_one(&mut self) -> Option<i32> {
        if let Self::Residence(h) = self
            && h.occupants > 0
        {
            h.occupants -= 1;
            Some(h.occupants)
        } else {
            None
        }
    }
    pub(super) fn ordered_fields(&self) -> &'static [&'static str] {
        match self {
            Self::Residence(_) => &["local_population"],
            Self::Fire(_) => &["burning_days", "smoking_days", "days_before_spread"],
            Self::Service(_) => &["active", "daycount", "covercount"],
            Self::Education(_) => &["total_tech_made"],
            Self::Monument(_) => &["tech_made", "tail_off", "completion", "labor_consumed"],
            _ => &[],
        }
    }
    pub(super) fn occupants(&self) -> i32 {
        if let Self::Residence(home) = self {
            home.occupants
        } else {
            0
        }
    }
    pub(super) fn incomplete_monument(&self) -> bool {
        matches!(self,Self::Monument(m) if !m.completed)
    }
}

impl City {
    pub(super) fn mark_coverage(&mut self, point: Point, range: i32, flag: u32) {
        for y in (point.y - range).max(1)..(point.y + range).min(self.side - 1) {
            for x in (point.x - range).max(1)..(point.x + range).min(self.side - 1) {
                self.tiles[(y * self.side + x) as usize].coverage |= flag;
            }
        }
    }
    pub(super) fn finalize_coverage(&mut self) {
        for tile in &mut self.tiles {
            tile.coverage = (tile.coverage & CHECK_MASK) >> 1;
        }
    }

    pub(super) fn update_civic(&mut self, id: BuildingId, point: Point) -> io::Result<bool> {
        let idx = (point.y * self.side + point.x) as usize;
        let b = self.buildings.get_mut(id).expect("visited building");
        let Operation::Civic(civic) = &mut b.operation else {
            return Ok(false);
        };
        let stock = &mut b.stock;
        let before = *stock;
        let mut ignite = false;
        let mut coverage = 0;
        let mut disconnect = false;
        match civic {
            Civic::Residence(h) => h.update(
                b.kind,
                stock,
                Neighborhood {
                    tile: self.tiles[idx],
                    stats: &mut self.stats.population,
                    people_pool: &mut self.people_pool,
                    decisions: &mut self.decisions,
                },
            )?,
            Civic::Park => {
                if self.day % PARK_CLEANUP_PERIOD == 0
                    && self.tiles[idx].pollution > PARK_POLLUTION_FLOOR
                {
                    self.tiles[idx].pollution -= 1;
                }
            }
            Civic::Well(output) => {
                if stock[Water as usize] + *output <= b.kind.metadata().rule(Water).max_load {
                    stock[Water as usize] += *output;
                }
            }
            Civic::Monument(m) => {
                if stock[Labor as usize] > MONUMENT_LABOR_PER_BATCH && m.completion < 100 {
                    stock[Labor as usize] -= MONUMENT_LABOR_PER_BATCH;
                    m.working += 1;
                    m.labor += MONUMENT_LABOR_PER_BATCH;
                    m.completion = m.labor * 100 / MONUMENT_TOTAL_LABOR;
                } else if m.completion >= 100 {
                    m.completed = true;
                    b.evacuating = true;
                    if self.tech_level < MONUMENT_RESEARCH_CEILING
                        && self.day % MONUMENT_RESEARCH_INTERVAL == MONUMENT_RESEARCH_PHASE
                    {
                        m.tail += 1;
                        if m.tail
                            > self.tech_level / MONUMENT_TECH_PER_DELAY - MONUMENT_DELAY_DISCOUNT
                        {
                            self.tech_level += 1;
                            m.tech += 1;
                            m.tail = 0;
                        }
                    }
                }
                disconnect = self.day % crate::rules::DAYS_PER_MONTH
                    == crate::rules::DAYS_PER_MONTH - 1
                    && stock[Labor as usize] == 0
                    && m.completed;
            }
            Civic::Service(_) | Civic::Education(_) => {
                let (rule, account) = match b.kind {
                    BuildingKind::School => (SCHOOL, &mut self.stats.expenses.school),
                    BuildingKind::University => (UNIVERSITY, &mut self.stats.expenses.university),
                    BuildingKind::FireStation => {
                        (FIRE_STATION, &mut self.stats.expenses.firestation)
                    }
                    BuildingKind::Health => (HEALTH_CENTRE, &mut self.stats.expenses.health),
                    BuildingKind::Cricket => (SPORTS, &mut self.stats.expenses.cricket),
                    _ => unreachable!(),
                };
                let ServiceRule {
                    labor,
                    goods,
                    cost,
                    tech_cost_factor,
                    research,
                    cover: flag,
                } = rule;
                let price =
                    cost * (1 + tech_cost_factor * self.tech_level / crate::rules::MAX_TECH_LEVEL);
                let funded = if b.kind == BuildingKind::School {
                    true
                } else if let Some(next) = self
                    .balance
                    .checked_sub(price)
                    .filter(|n| *n >= crate::rules::CREDIT_LIMIT)
                {
                    if self.balance > 0 && next <= 0 {
                        self.events.push_back(CityEvent::OutOfMoney);
                    }
                    self.balance = next;
                    true
                } else {
                    false
                };
                let active = funded
                    && stock[Labor as usize] >= labor
                    && stock[Goods as usize] >= goods
                    && stock[Waste as usize] + goods / GOODS_PER_WASTE
                        <= b.kind.metadata().rule(Waste).max_load;
                if funded {
                    account.acc = account.acc.saturating_add(price);
                }
                if active {
                    stock[Labor as usize] -= labor;
                    stock[Goods as usize] -= goods;
                    stock[Waste as usize] += goods / GOODS_PER_WASTE;
                }
                match civic {
                    Civic::Education(e) if active => {
                        e.working += 1;
                        e.tech += research;
                        self.tech_level += research;
                    }
                    Civic::Service(s) => {
                        s.days += 1;
                        s.supplied += i32::from(active);
                        s.working += i32::from(active);
                        if self.day % COVER_PERIOD == COVER_DAY {
                            s.active = s.supplied + COVER_TOLERANCE >= s.days;
                            if s.active {
                                s.supplied -= s.days;
                                coverage = flag;
                                s.animate = b.kind != BuildingKind::Health;
                            }
                            s.days = 0;
                        }
                    }
                    _ => {}
                }
            }
            Civic::Fire(_) => {
                self.update_fire(id, point)?;
                return Ok(true);
            }
            Civic::Shanty(fire) => {
                stock[Waste as usize] += SHANTY_DAILY_WASTE;
                for (commodity, amount) in SHANTY_BATCHES {
                    if stock[commodity as usize] < amount {
                        continue;
                    }
                    stock[commodity as usize] -= amount;
                    if commodity == Goods {
                        stock[Waste as usize] += amount / GOODS_PER_WASTE;
                    }
                    match commodity {
                        Labor => {
                            self.economy.taxable_labor =
                                (self.economy.taxable_labor - SHANTY_TAX_MULTIPLIER * amount).max(0)
                        }
                        Goods => {
                            self.economy.taxable_goods =
                                (self.economy.taxable_goods - SHANTY_TAX_MULTIPLIER * amount).max(0)
                        }
                        Coal => {
                            self.economy.taxable_coal =
                                (self.economy.taxable_coal - SHANTY_TAX_MULTIPLIER * amount).max(0)
                        }
                        _ => {}
                    }
                }
                if stock[Waste as usize] >= b.kind.metadata().rule(Waste).max_load {
                    self.tiles[idx + self.side as usize + 1].pollution +=
                        std::mem::take(&mut stock[Waste as usize]);
                    fire.start = true;
                    ignite = self.decisions.bernoulli(
                        "shanty.waste-fire",
                        UNCONTROLLED_WASTE_FIRE * f64::from(b.kind.metadata().fire_chance) / 100.0,
                    )? && self.tiles[idx].coverage & FIRE == 0;
                }
            }
        }
        for (counter, (now, then)) in b
            .production
            .current
            .iter_mut()
            .zip(stock.iter().zip(before))
        {
            *counter += now - then;
        }
        if self.day % crate::rules::DAYS_PER_MONTH
            == if b.kind == BuildingKind::School {
                0
            } else {
                crate::rules::DAYS_PER_MONTH - 1
            }
        {
            match civic {
                Civic::Education(e) => e.busy = std::mem::take(&mut e.working),
                Civic::Service(s) => s.busy = std::mem::take(&mut s.working),
                Civic::Monument(m) => m.busy = std::mem::take(&mut m.working),
                _ => {}
            }
            b.production.close_month();
        }
        let range = b.kind.metadata().range;
        if coverage != 0 {
            self.mark_coverage(point, range, coverage);
        }
        if ignite {
            self.ignite_building(id, point)?;
        }
        if disconnect {
            self.disconnect_building(id);
        }
        Ok(true)
    }
}

impl Civic {
    pub(super) fn set_starter_population(&mut self) {
        if let Self::Residence(home) = self { home.occupants = crate::rules::generation::STARTER_HOME_POPULATION; }
    }
}
