//! Read-only simulation data for the Godot desktop.
use crate::Engine;
use rustcity_core::{Commodity, Point, ReportMode, ReportRow};

#[cxx::bridge(namespace = "rustcity::view")]
pub mod ffi {
    extern "C++" {
        include!("rustcity-godot/src/lib.rs.h");
        #[namespace = "rustcity"]
        type Engine = crate::Engine;
        #[namespace = "rustcity"]
        type Point = crate::ffi::Point;
    }
    struct Tile {
        terrain: u16,
        altitude: i32,
        coal: i32,
        ore: i32,
        pollution: i32,
        group: u16,
        anchor_x: i32,
        anchor_y: i32,
        name: String,
        resource: String,
    }
    struct Snapshot {
        groups: Vec<i32>,
        anchors: Vec<i32>,
        pollution: Vec<i32>,
        operating: Vec<u8>,
        rockets: Vec<RocketFrame>,
        population: i64,
        inventory: Vec<Stock>,
    }
    struct RocketFrame {
        index: i32,
        frame: u8,
    }
    struct Stock {
        amount: i64,
        capacity: i64,
    }
    struct Tool {
        id: i32,
        name: String,
        size: u16,
        base_cost: i32,
        required_tech: i32,
    }
    struct TradeRule {
        id: u8,
        buy: bool,
        sell: bool,
    }
    struct Calendar {
        days_per_month: i32,
        days_per_year: i32,
        max_tech: i32,
    }
    struct Row {
        kind: RowKind,
        label: String,
        text: String,
        value: i32,
        capacity: i32,
        number: f64,
    }
    enum RowKind {
        Blank,
        Label,
        Text,
        Integer,
        Decimal,
        Percentage,
        Quantity,
        Prefixed,
    }
    extern "Rust" {
        fn snapshot(engine: &Engine) -> Snapshot;
        fn tile(engine: &Engine, point: Point) -> Tile;
        fn tool(id: i32) -> Tool;
        fn tools() -> Vec<Tool>;
        fn group_name(id: u16) -> String;
        fn trade_rules(engine: &Engine, point: Point) -> Vec<TradeRule>;
        fn calendar() -> Calendar;
        fn report(engine: &Engine, x: i32, y: i32, production: bool) -> Vec<Row>;
        fn statistics(engine: &Engine) -> Statistics;
        fn history(engine: &Engine) -> History;
        fn history_presence(engine: &Engine, series: &str, count: usize) -> Vec<u8>;
    }
    struct Inventory {
        amount: i32,
        capacity: i32,
    }
    struct InventoryHistory {
        samples: Vec<Inventory>,
    }
    struct Population {
        unemployed_m: i32,
        evacuated_t: i32,
        births_t: i32,
        pollution_deaths_t: i32,
        starve_deaths_t: i32,
        unemployed_days_t: i32,
        max_pop_ever: i32,
    }
    struct Income {
        income_tax: i32,
        coal_tax: i32,
        goods_tax: i32,
        export_tax: i32,
    }
    struct Expenses {
        construction: i32,
        coal_survey: i32,
        import: i32,
        unemployment: i32,
        transport: i32,
        windmill: i32,
        university: i32,
        recycle: i32,
        deaths: i32,
        health: i32,
        rockets: i32,
        school: i32,
        firestation: i32,
        cricket: i32,
        interest: i32,
    }
    struct Sustainability {
        sustainable: bool,
        mining_years: i32,
        trade_years: i32,
        goals: [i32; 6],
        money_years: i32,
        population_years: i32,
        tech_years: i32,
        fire_years: i32,
    }
    struct History {
        pop: Vec<i32>,
        starve: Vec<i32>,
        nojobs: Vec<i32>,
        ppool: Vec<i32>,
        tech: Vec<i32>,
        money: Vec<i32>,
        pollution: Vec<i32>,
        births: Vec<i32>,
        deaths: Vec<i32>,
        unnat_deaths: Vec<i32>,
        tenants: Vec<Inventory>,
        inventory: Vec<InventoryHistory>,
    }
    struct Statistics {
        day: i32,
        balance: i32,
        tech: i32,
        people_pool: i32,
        launched: i32,
        successes: i32,
        survey: bool,
        name: String,
        focus_x: i32,
        focus_y: i32,
        highest_tech_level: i32,
        population: Population,
        income: Income,
        expenses: Expenses,
        sustainability: Sustainability,
        group_count: Vec<u32>,
    }
}

fn snapshot(engine: &Engine) -> ffi::Snapshot {
    let city = &engine.city;
    let side = city.side();
    let length = (side * side) as usize;
    let mut view = ffi::Snapshot {
        groups: Vec::with_capacity(length),
        anchors: Vec::with_capacity(length),
        pollution: Vec::with_capacity(length),
        operating: Vec::with_capacity(length),
        rockets: Vec::new(),
        population: i64::from(city.report().people_pool),
        inventory: (0..Commodity::ALL.len())
            .map(|_| ffi::Stock {
                amount: 0,
                capacity: 0,
            })
            .collect(),
    };
    for t in city.map_region(0..side, 0..side) {
        let anchor = t.building.map_or(t.point, |(_, origin)| origin);
        view.groups.push(i32::from(
            t.building.map_or(t.terrain.group(), |(kind, _)| kind.id()),
        ));
        view.anchors.push(anchor.y * side + anchor.x);
        view.pollution.push(t.pollution);
        let building = t
            .building
            .filter(|(_, origin)| *origin == t.point)
            .and_then(|_| city.building_state(anchor));
        view.operating.push(u8::from(
            building
                .as_ref()
                .is_some_and(|b| b.industrial_output_today > 0),
        ));
        if let Some(building) = building {
            view.population += i64::from(building.occupants);
            if let Some(frame) = building.rocket_frame {
                view.rockets.push(ffi::RocketFrame {
                    index: anchor.y * side + anchor.x,
                    frame,
                });
            }
            for (commodity, stock) in Commodity::ALL.into_iter().zip(&mut view.inventory) {
                stock.amount += i64::from(building.stock(commodity));
                stock.capacity += i64::from(building.kind.metadata().rule(commodity).max_load);
            }
        }
    }
    view
}

fn terrain_labels(terrain: rustcity_core::Terrain) -> (&'static str, &'static str) {
    use rustcity_core::Terrain::*;
    match terrain {
        Grass => ("Empty", "Green"),
        Water => ("Water", "Water"),
        Desert => ("Desert", "Desert"),
        Tree => ("Tree", "Tree"),
        Tree2 => ("Trees", "Tree2"),
        Tree3 => ("Forest", "Tree3"),
    }
}

fn tile(engine: &Engine, point: crate::ffi::Point) -> ffi::Tile {
    let city = &engine.city;
    let Some(t) = city.tile(point.into()) else {
        return ffi::Tile {
            terrain: 0,
            altitude: 0,
            coal: 0,
            ore: 0,
            pollution: 0,
            group: 0,
            anchor_x: point.x,
            anchor_y: point.y,
            name: String::new(),
            resource: String::new(),
        };
    };
    let (name, ground_resource) = terrain_labels(t.terrain);
    let (group, anchor, resource) = if let Some((kind, anchor)) = t.building {
        let visual = city.building_visual(anchor);
        let resource = visual
            .as_ref()
            .and_then(|v| v.layers.first())
            .map_or(kind.metadata().resource, |layer| layer.resource);
        (kind.id(), anchor, resource)
    } else {
        (t.terrain.group(), t.point, ground_resource)
    };
    ffi::Tile {
        terrain: t.terrain.group(),
        altitude: t.altitude,
        coal: t.coal,
        ore: t.ore,
        pollution: t.pollution,
        group,
        anchor_x: anchor.x,
        anchor_y: anchor.y,
        name: name.into(),
        resource: resource.into(),
    }
}

fn group_name(id: u16) -> String {
    rustcity_core::BuildingKind::from_id(id)
        .map_or("", |kind| kind.metadata().name)
        .into()
}

fn tool(id: i32) -> ffi::Tool {
    let (name, size, base_cost, required_tech) = match id {
        -1 => ("Inspect", 1, 0, 0),
        -2 => ("Bulldoze", 1, 0, 0),
        -3 => ("Evacuate", 1, 0, 0),
        -4 => ("Water", 1, rustcity_core::FLOOD_COST, 0),
        1..=31 | 36..=41 | 50 => rustcity_core::BuildingKind::from_id(id as u16)
            .map(|kind| kind.metadata())
            .map_or(("", 0, 0, 0), |m| {
                (m.name, m.size, m.base_cost, m.required_tech)
            }),
        _ => ("", 0, 0, 0),
    };
    ffi::Tool {
        id,
        name: name.into(),
        size,
        base_cost,
        required_tech,
    }
}

fn tools() -> Vec<ffi::Tool> {
    [-1, -2, -3, -4]
        .into_iter()
        .chain(1..=50)
        .map(tool)
        .filter(|tool| tool.size > 0)
        .collect()
}

fn trade_rules(engine: &Engine, point: crate::ffi::Point) -> Vec<ffi::TradeRule> {
    Commodity::ALL
        .into_iter()
        .filter_map(|c| {
            let policy = engine.city.building_trade_policy(point.into(), c)?;
            Some(ffi::TradeRule {
                id: c as u8,
                buy: policy.buy,
                sell: policy.sell,
            })
        })
        .collect()
}

fn calendar() -> ffi::Calendar {
    ffi::Calendar {
        days_per_month: rustcity_core::DAYS_PER_MONTH,
        days_per_year: rustcity_core::DAYS_PER_YEAR,
        max_tech: rustcity_core::MAX_TECH_LEVEL,
    }
}

fn report(engine: &Engine, x: i32, y: i32, production: bool) -> Vec<ffi::Row> {
    let mode = if production {
        ReportMode::Production
    } else {
        ReportMode::Inventory
    };
    engine
        .city
        .building_report(Point { x, y }, mode)
        .unwrap_or_default()
        .into_iter()
        .map(|row| {
            use ffi::RowKind as K;
            let (kind, label, text, value, capacity, number) = match row {
                ReportRow::Blank => (K::Blank, "", "", 0, 0, 0.),
                ReportRow::Label(k) => (K::Label, k, "", 0, 0, 0.),
                ReportRow::Text(k, v) => (K::Text, k, v, 0, 0, 0.),
                ReportRow::Integer(k, v) => (K::Integer, k, "", v, 0, 0.),
                ReportRow::Decimal(k, v) => (K::Decimal, k, "", 0, 0, v),
                ReportRow::Percentage(k, v) => (K::Percentage, k, "", 0, 0, v),
                ReportRow::Quantity(k, v, m) => (K::Quantity, k, "", v, m, 0.),
                ReportRow::PrefixedQuantity(p, k, v, m) => (K::Prefixed, k, p, v, m, 0.),
            };
            ffi::Row {
                kind,
                label: label.into(),
                text: text.into(),
                value,
                capacity,
                number,
            }
        })
        .collect()
}

fn inventory(i: &rustcity_core::stats::Inventory) -> ffi::Inventory {
    ffi::Inventory {
        amount: i.amount,
        capacity: i.capacity,
    }
}
fn history_presence(engine: &Engine, series: &str, count: usize) -> Vec<u8> {
    let history = &engine.city.statistics().history;
    (0..count)
        .map(|index| u8::from(history.has_sample(series, index)))
        .collect()
}

fn statistics(engine: &Engine) -> ffi::Statistics {
    let c = &engine.city;
    let r = c.report();
    let s = c.statistics();
    let p = c.focus();
    ffi::Statistics {
        day: r.day,
        balance: r.balance,
        tech: r.tech_level,
        people_pool: r.people_pool,
        launched: r.rockets_launched,
        successes: r.successful_launches,
        survey: c.coal_survey_done(),
        name: c.scenario_name().into(),
        focus_x: p.x,
        focus_y: p.y,
        highest_tech_level: s.highest_tech_level,
        group_count: s.group_count.to_vec(),
        population: ffi::Population {
            unemployed_m: s.population.unemployed_m.value(),
            evacuated_t: s.population.evacuated_t,
            births_t: s.population.births_t,
            pollution_deaths_t: s.population.pollution_deaths_t,
            starve_deaths_t: s.population.starve_deaths_t,
            unemployed_days_t: s.population.unemployed_days_t,
            max_pop_ever: s.population.max_pop_ever,
        },
        income: ffi::Income {
            income_tax: s.income.income_tax.value(),
            coal_tax: s.income.coal_tax.value(),
            goods_tax: s.income.goods_tax.value(),
            export_tax: s.income.export_tax.value(),
        },
        expenses: ffi::Expenses {
            construction: s.expenses.construction.value(),
            coal_survey: s.expenses.coal_survey.value(),
            import: s.expenses.import.value(),
            unemployment: s.expenses.unemployment.value(),
            transport: s.expenses.transport.value(),
            windmill: s.expenses.windmill.value(),
            university: s.expenses.university.value(),
            recycle: s.expenses.recycle.value(),
            deaths: s.expenses.deaths.value(),
            health: s.expenses.health.value(),
            rockets: s.expenses.rockets.value(),
            school: s.expenses.school.value(),
            firestation: s.expenses.firestation.value(),
            cricket: s.expenses.cricket.value(),
            interest: s.expenses.interest.value(),
        },
        sustainability: ffi::Sustainability {
            sustainable: s.sustainability.sustainable,
            mining_years: s.sustainability.mining_years,
            trade_years: s.sustainability.trade_years,
            goals: [
                rustcity_core::MINING_YEARS,
                rustcity_core::TRADE_YEARS,
                rustcity_core::MONEY_YEARS,
                rustcity_core::POPULATION_YEARS,
                rustcity_core::TECHNOLOGY_YEARS,
                rustcity_core::FIRE_YEARS,
            ],
            money_years: s.sustainability.money_years,
            population_years: s.sustainability.population_years,
            tech_years: s.sustainability.tech_years,
            fire_years: s.sustainability.fire_years,
        },
    }
}

fn history(engine: &Engine) -> ffi::History {
    let h = &engine.city.statistics().history;
    ffi::History {
        pop: h.pop.iter().copied().collect(),
        starve: h.starve.iter().copied().collect(),
        nojobs: h.nojobs.iter().copied().collect(),
        ppool: h.ppool.iter().copied().collect(),
        tech: h.tech.iter().copied().collect(),
        money: h.money.iter().copied().collect(),
        pollution: h.pollution.iter().copied().collect(),
        births: h.births.iter().copied().collect(),
        deaths: h.deaths.iter().copied().collect(),
        unnat_deaths: h.unnat_deaths.iter().copied().collect(),
        tenants: h.tenants.iter().map(inventory).collect(),
        inventory: h
            .inventory
            .iter()
            .map(|h| ffi::InventoryHistory {
                samples: h.iter().map(inventory).collect(),
            })
            .collect(),
    }
}
