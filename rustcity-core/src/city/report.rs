use super::*;
use crate::Commodity;
use crate::rules::PERCENT;
#[cfg(test)]
mod tests;

use super::{civic::Civic, industry::IndustryOperation as Industry};
use crate::rules::{DAYS_PER_MONTH, MAX_TECH_LEVEL, industry, transport};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportMode {
    Inventory,
    Production,
}

/// Ordered rows of a building report. Labels are the original translation keys.
#[derive(Clone, Debug, PartialEq)]
pub enum ReportRow {
    Blank,
    Label(&'static str),
    Text(&'static str, &'static str),
    Integer(&'static str, i32),
    Decimal(&'static str, f64),
    Percentage(&'static str, f64),
    Quantity(&'static str, i32, i32),
    PrefixedQuantity(&'static str, &'static str, i32, i32),
}
use ReportRow::*;

fn tech(value: i32) -> f64 {
    f64::from(value) * f64::from(PERCENT) / f64::from(MAX_TECH_LEVEL)
}
fn busy(value: i32) -> ReportRow {
    Percentage("busy", f64::from(value))
}
fn yes(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}
fn name(c: Commodity) -> &'static str {
    [
        "Food", "Labor", "Coal", "Goods", "Ore", "Steel", "Waste", "Lo-Volt", "Hi-Volt", "Water",
    ][c as usize]
}

impl Building {
    fn limits(&self) -> [(i32, i32); 10] {
        use crate::Commodity::*;
        let mut limits = [(0, 0); 10];
        for &(c, prod, cons) in crate::rules::report::base_limits(self.kind) {
            limits[c as usize] = (prod, cons);
        }
        let mut output = |c: Commodity, n| limits[c as usize].0 = DAYS_PER_MONTH * n;
        match &self.operation {
            Operation::Wind(w) => output(
                if self.kind == BuildingKind::Windmill {
                    LowVoltage
                } else {
                    HighVoltage
                },
                w.output,
            ),
            Operation::Coal(s) => output(HighVoltage, s.output),
            Operation::Solar(s) => output(HighVoltage, s.output),
            Operation::Farm(_) => {
                let base = crate::rules::farm::FOOD_PER_BATCH;
                output(
                    Food,
                    base + (i64::from(self.installed_tech) * i64::from(base)
                        / i64::from(MAX_TECH_LEVEL)) as i32,
                );
            }
            Operation::Industry(Industry::Recycle(r)) => {
                output(Steel, r.make_steel);
                output(Ore, r.make_ore);
            }
            Operation::Industry(Industry::Light(_) | Industry::Heavy(_)) => {
                let t = f64::from(self.installed_tech) / f64::from(MAX_TECH_LEVEL);
                let gross = if self.kind == BuildingKind::LightIndustry {
                    industry::light::POLLUTION_PER_GOOD * f64::from(industry::light::GOODS)
                } else {
                    industry::heavy::POLLUTION_PER_STEEL * f64::from(industry::heavy::OUTPUT)
                };
                output(
                    Waste,
                    (gross * (t - 1.).clamp(0., 1.) * (1. - (t - 2.).clamp(0., 1.))) as i32,
                );
            }
            _ => {}
        }
        limits
    }

    fn commodities(&self, rows: &mut Vec<ReportRow>, mode: ReportMode) {
        let production = mode == ReportMode::Production;
        rows.push(Label(if production {
            "Production:"
        } else {
            "Inventory:"
        }));
        let limits = self.limits();
        for (give, take, prefix) in [
            (true, false, "<-- "),
            (false, true, "--> "),
            (true, true, "<-> "),
            (false, false, "--- "),
        ] {
            for c in Commodity::ALL {
                let rule = self.kind.metadata().rule(c);
                if rule.max_load == 0 {
                    continue;
                }
                let i = c as usize;
                let (prod, cons) = limits[i];
                let flags = if production {
                    (prod != 0, cons != 0)
                } else {
                    (rule.give, rule.take)
                };
                if flags != (give, take) {
                    continue;
                }
                let (amount, max) = if production {
                    let amount = self.production.previous[i];
                    (amount, if amount < 0 { -cons } else { prod })
                } else {
                    (self.stock[i], rule.max_load)
                };
                rows.push(PrefixedQuantity(
                    if self.evacuating && !production {
                        "<<< "
                    } else {
                        prefix
                    },
                    name(c),
                    amount,
                    max,
                ));
            }
        }
    }
}

impl City {
    /// Generates the selected report for the building occupying a map square.
    pub fn building_report(&self, point: Point, mode: ReportMode) -> Option<Vec<ReportRow>> {
        let (origin, b) = self.buildings.get(self.buildings.at(point)?)?;
        let pollution = self.tiles[self.index(origin)?].pollution;
        let mut rows = vec![Label(b.kind.metadata().name)];
        let mut suffix = Vec::new();
        let mut commodities = true;
        let technology = Percentage(
            "Tech",
            match b.kind {
                BuildingKind::SolarPower | BuildingKind::CoalPower => f64::from(
                    (f64::from(b.installed_tech) * f64::from(PERCENT)) as f32
                        / MAX_TECH_LEVEL as f32,
                ),
                _ => tech(b.installed_tech),
            },
        );
        let size = i32::from(b.kind.metadata().size);
        match &b.operation {
            Operation::Solar(p) => {
                rows.extend([Blank, busy(p.busy), technology, Integer("Output", p.output)])
            }
            Operation::Station(p) => rows.extend([Blank, busy(p.busy)]),
            Operation::Coal(p) => {
                rows.extend([busy(p.busy), technology, Integer("Output", p.output)])
            }
            Operation::Wind(p) => rows.extend([
                Percentage("busy", f64::from(p.utilization())),
                technology,
                Integer("Output", p.output),
            ]),
            Operation::Line(p) => rows.push(Percentage("usage", p.usage())),
            Operation::Workshop(w) => {
                if b.kind == BuildingKind::Pottery {
                    rows.push(Blank);
                }
                rows.push(busy(w.busy));
            }
            Operation::Farm(f) => rows.extend([
                Blank,
                Quantity("Fertility", f.groundwater, size * size),
                technology,
                Percentage(
                    "busy",
                    f64::from(f.last_utilization as f32) / f64::from(DAYS_PER_MONTH),
                ),
                Integer("Output", f.max_food),
            ]),
            Operation::Commune(c) => {
                rows.extend([
                    Quantity("Fertility", c.wet_tiles, size * size),
                    Percentage(
                        "busy",
                        f64::from(c.last_output as f32)
                            / crate::rules::commune::OUTPUT_PER_BUSY_PERCENT,
                    ),
                    Integer("Pollution", pollution),
                    if c.idle_months != 0 {
                        Quantity(
                            "lazy months",
                            c.idle_months,
                            crate::rules::commune::ABANDONMENT_MONTHS,
                        )
                    } else {
                        Blank
                    },
                ]);
            }
            Operation::Rocket(r) => rows.extend([
                busy(r.busy),
                technology,
                Percentage(
                    "Completion",
                    f64::from(r.steps) * f64::from(PERCENT)
                        / f64::from(crate::rules::rocket::STEPS),
                ),
            ]),
            Operation::Port(p) => rows.extend([
                busy(p.busy),
                Integer(
                    "Export",
                    p.last_exports / crate::rules::economy::PENCE_PER_UNIT,
                ),
                Integer(
                    "Import",
                    p.last_imports / crate::rules::economy::PENCE_PER_UNIT,
                ),
                Percentage("Culture exchanged", tech(p.tech_made)),
            ]),
            Operation::Transport(t) => {
                rows.push(Blank);
                if mode == ReportMode::Production {
                    commodities = false;
                    rows.push(Label("Traffic"));
                    for c in Commodity::ALL {
                        if b.kind.metadata().rule(c).max_load != 0 {
                            rows.push(Percentage(
                                name(c),
                                f64::from(t.traffic[c as usize])
                                    * transport::TRAFFIC_REPORT_FACTOR
                                    * f64::from(transport::RATE)
                                    / f64::from(transport::QUANTA),
                            ));
                        }
                    }
                }
            }
            Operation::Market(m) => {
                commodities = false;
                rows.extend([Blank, busy(m.busy), Blank]);
                for c in Commodity::ALL {
                    let r = m.rules[c as usize];
                    if r.max_load > 0 {
                        let arrow = if b.evacuating {
                            "<< "
                        } else {
                            match (r.give, r.take) {
                                (true, true) => "<->",
                                (true, false) => "<--",
                                (false, true) => "-->",
                                _ => "---",
                            }
                        };
                        rows.push(PrefixedQuantity(
                            arrow,
                            name(c),
                            b.stock[c as usize],
                            r.max_load,
                        ));
                    }
                }
            }
            Operation::Industry(s) => match s {
                Industry::Coal(m) => rows.extend([
                    busy(m.activity.busy),
                    Quantity("Deposits", m.current_reserve, m.initial_reserve),
                ]),
                Industry::Ore(m) => rows.extend([
                    busy(m.activity.busy),
                    Quantity("Deposits", m.reserve, size * size * industry::ORE_RESERVE),
                ]),
                Industry::Light(s) => rows.extend([Blank, busy(s.busy), technology]),
                Industry::Heavy(s) => rows.extend([Blank, busy(s.busy), technology]),
                Industry::Recycle(s) => rows.extend([
                    Blank,
                    Percentage(
                        "Tech",
                        f64::from(b.installed_tech as f32 * PERCENT as f32 / MAX_TECH_LEVEL as f32),
                    ),
                    Percentage(
                        "Efficiency Ore",
                        f64::from(
                            s.make_ore as f32 * PERCENT as f32 / industry::recycle::WASTE as f32,
                        ),
                    ),
                    Percentage(
                        "Efficiency Steel",
                        f64::from(
                            s.make_steel as f32 * PERCENT as f32 / industry::recycle::WASTE as f32,
                        ),
                    ),
                    busy(s.activity.busy),
                ]),
                Industry::Tip(s) => rows.extend([
                    Blank,
                    busy(s.activity.busy),
                    Integer("Waste", s.buried),
                    Percentage(
                        "Filled",
                        f64::from(
                            s.buried as f32 * PERCENT as f32 / industry::tip::CAPACITY as f32,
                        ),
                    ),
                ]),
            },
            Operation::Civic(c) => {
                use crate::rules::civic::*;
                match c {
                    Civic::Well(w) => {
                        rows.extend([
                            Blank,
                            Quantity("Fertility", w.output / WELL_WATER_PER_WET_TILE, size * size),
                            busy(w.busy),
                            Quantity("Air Pollution", pollution, WELL_MAX_DRINKABLE_POLLUTION),
                            Text("Drinkable", yes(pollution <= WELL_MAX_DRINKABLE_POLLUTION)),
                        ]);
                    }
                    Civic::Residence(h) => {
                        let year = crate::rules::DAYS_PER_YEAR as f32;
                        rows.extend([
                            Quantity("Tenants", h.occupants, housing_rule(b.kind).capacity),
                            Integer("Desireability", h.desirability),
                            Decimal("Births p.a.", f64::from(year / h.births as f32)),
                            Decimal("Death p.a.", f64::from(year / h.deaths as f32)),
                            Percentage("Unnat. mortality", f64::from(h.pollution_deaths)),
                        ]);
                    }
                    Civic::Park => {
                        commodities = false;
                        rows.extend([Blank, Integer("Air Pollution", pollution)]);
                    }
                    Civic::Shanty(_) => rows.push(Integer("Air Pollution", pollution)),
                    Civic::Fire(f) => {
                        commodities = false;
                        rows.extend([
                            Blank,
                            Integer("Air Pollution", pollution),
                            if f.burning < FIRE_BURNING_DAYS {
                                Quantity("burnt down", f.burning, FIRE_BURNING_DAYS)
                            } else {
                                Quantity("degraded", f.smoking, FIRE_SMOKING_DAYS)
                            },
                        ]);
                    }
                    Civic::Education(e) => rows.extend([
                        Blank,
                        busy(e.busy),
                        Percentage(
                            if b.kind == BuildingKind::School {
                                "Lessons learned"
                            } else {
                                "Tech researched"
                            },
                            tech(e.tech),
                        ),
                    ]),
                    Civic::Service(s) => {
                        rows.push(busy(s.busy));
                        suffix.push(Text(
                            match b.kind {
                                BuildingKind::Health => "Health Care",
                                BuildingKind::Cricket => "Public sports",
                                _ => "Fire Protection",
                            },
                            yes(s.active),
                        ));
                    }
                    Civic::Monument(m) => {
                        rows.push(Blank);
                        if m.completion >= PERCENT {
                            commodities = false;
                            rows.extend([Blank, Percentage("Wisdom bestowed", tech(m.tech))]);
                        } else {
                            rows.push(busy(m.busy));
                            suffix
                                .extend([Blank, Percentage("Completion", f64::from(m.completion))]);
                        }
                    }
                }
            }
            Operation::Other => {}
        }
        if commodities {
            b.commodities(&mut rows, mode);
        }
        rows.extend(suffix);
        Some(rows)
    }
}
