//! Immutable snapshots consumed by the existing desktop widgets.
use crate::Engine;
use rustcity_core::{Commodity, Point, ReportMode, ReportRow};

#[cxx::bridge(namespace = "rustcity::view")]
pub mod ffi {
    extern "C++" {
        include!("rustcity-legacy/src/lib.rs.h");
        #[namespace = "rustcity"]
        type Engine = crate::Engine;
    }
    struct Tile {
        ground: u16,
        frame: u16,
        altitude: i32,
        water_altitude: i32,
        coal: i32,
        ore: i32,
        pollution: i32,
        lake: bool,
        river: bool,
        groundwater: bool,
        fire: bool,
        health: bool,
        sports: bool,
        market: bool,
        cable_ns: bool,
        cable_ew: bool,
    }
    struct Building {
        x: i32,
        y: i32,
        id: u64,
        group: u16,
        evacuating: bool,
        never_evacuate: bool,
        burning: i32,
        smoking: i32,
        occupants: i32,
        stock: [i32; 10],
        traffic: [i32; 10],
        buy: [bool; 10],
        sell: [bool; 10],
    }
    enum RocketPhase {
        Building,
        Awaiting,
        Launching,
        Launch,
        Done,
    }
    struct Sprite {
        resource: String,
        frame: i32,
        dx: i32,
        dy: i32,
    }
    struct Animation {
        x: i32,
        y: i32,
        transparent: bool,
        rocket_stage: RocketPhase,
        layers: Vec<Sprite>,
    }
    struct Car {
        x: i32,
        y: i32,
        frame: i32,
        dx: i32,
        dy: i32,
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
        fn tiles(engine: &Engine) -> Vec<Tile>;
        fn buildings(engine: &Engine) -> Vec<Building>;
        fn sprites(engine: &Engine) -> Vec<Animation>;
        fn cars(engine: &Engine) -> Vec<Car>;
        fn report(engine: &Engine, x: i32, y: i32, production: bool) -> Vec<Row>;
        fn statistics(engine: &Engine) -> Statistics;
    }
    struct Period {
        previous: i32,
        current: i32,
    }
    struct Inventory {
        amount: i32,
        capacity: i32,
    }
    struct InventoryPeriod {
        amount: Period,
        capacity: Period,
    }
    struct InventoryHistory {
        samples: Vec<Inventory>,
    }
    struct Population {
        population_m: Period,
        housed_m: Period,
        starving_m: Period,
        unemployed_m: Period,
        housing_m: Period,
        births_m: Period,
        deaths_m: Period,
        unnat_deaths_m: Period,
        evacuated_t: i32,
        births_t: i32,
        pollution_deaths_t: i32,
        starve_deaths_t: i32,
        unemployed_days_t: i32,
        max_pop_ever: i32,
        unemployed_history: f32,
        pollution_deaths_history: f32,
        starve_deaths_history: f32,
    }
    struct Income {
        income_tax: Period,
        coal_tax: Period,
        goods_tax: Period,
        export_tax: Period,
    }
    struct Expenses {
        construction: Period,
        coal_survey: Period,
        import: Period,
        unemployment: Period,
        transport: Period,
        windmill: Period,
        university: Period,
        recycle: Period,
        deaths: Period,
        health: Period,
        rockets: Period,
        school: Period,
        firestation: Period,
        cricket: Period,
        interest: Period,
    }
    struct Sustainability {
        mining_flag: bool,
        trade_flag: bool,
        sustainable: bool,
        mining_years: i32,
        trade_years: i32,
        old_money: i32,
        money_years: i32,
        old_population: i32,
        population_years: i32,
        old_tech: i32,
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
        game_end: bool,
        survey: bool,
        name: String,
        focus_x: i32,
        focus_y: i32,
        foreign_buy: [bool; 10],
        foreign_sell: [bool; 10],
        highest_tech_level: i32,
        total_pollution: i32,
        stats_tech: i32,
        stats_money: i32,
        population: Population,
        income: Income,
        expenses: Expenses,
        sustainability: Sustainability,
        history: History,
        inventory: Vec<InventoryPeriod>,
        group_count: Vec<u32>,
    }
}

fn tiles(engine: &Engine) -> Vec<ffi::Tile> {
    let side = engine.city.side();
    engine
        .city
        .map_region(0..side, 0..side)
        .map(|t| ffi::Tile {
            ground: t.terrain.group(),
            frame: t.ground_frame,
            altitude: t.altitude,
            water_altitude: t.water_altitude,
            coal: t.coal,
            ore: t.ore,
            pollution: t.pollution,
            lake: t.lake,
            river: t.river,
            groundwater: t.groundwater,
            fire: t.fire_cover,
            health: t.health_cover,
            sports: t.sports_cover,
            market: t.market_cover,
            cable_ns: t.north_south_cable,
            cable_ew: t.east_west_cable,
        })
        .collect()
}

fn buildings(engine: &Engine) -> Vec<ffi::Building> {
    let city = &engine.city;
    city.building_inventories()
        .map(|(p, k, stock)| {
            let b = city.building_state(p).unwrap();
            let (burning, smoking) = b.fire_age.unwrap_or_default();
            let mut buy = [false; 10];
            let mut sell = [false; 10];
            let mut traffic = [0; 10];
            for c in Commodity::ALL {
                if let Some(policy) = city.building_trade_policy(p, c) {
                    buy[c as usize] = policy.buy;
                    sell[c as usize] = policy.sell;
                }
                traffic[c as usize] = city.commodity_traffic(p, c).unwrap_or(0);
            }
            ffi::Building {
                x: p.x,
                y: p.y,
                id: b.instance_id,
                group: k.id(),
                occupants: b.occupants,
                evacuating: b.evacuating,
                never_evacuate: b.never_evacuate,
                stock: *stock,
                traffic,
                buy,
                sell,
                burning,
                smoking,
            }
        })
        .collect()
}

fn sprites(engine: &Engine) -> Vec<ffi::Animation> {
    engine
        .city
        .building_inventories()
        .map(|(p, kind, _)| {
            let phase = if kind == rustcity_core::BuildingKind::RocketPad {
                engine.city.building_state(p).and_then(|b| b.rocket_stage)
            } else {
                None
            };
            let visual = engine.city.building_visual(p).unwrap();
            ffi::Animation {
                x: p.x,
                y: p.y,
                transparent: visual.transparent,
                rocket_stage: match phase.unwrap_or_default() {
                    rustcity_core::RocketStage::Building => ffi::RocketPhase::Building,
                    rustcity_core::RocketStage::Awaiting => ffi::RocketPhase::Awaiting,
                    rustcity_core::RocketStage::Launching => ffi::RocketPhase::Launching,
                    rustcity_core::RocketStage::Launch => ffi::RocketPhase::Launch,
                    rustcity_core::RocketStage::Done => ffi::RocketPhase::Done,
                },

                layers: visual
                    .layers
                    .into_iter()
                    .map(|s| ffi::Sprite {
                        resource: s.resource.to_owned(),
                        frame: s.frame,
                        dx: s.offset.0,
                        dy: s.offset.1,
                    })
                    .collect(),
            }
        })
        .collect()
}
fn cars(engine: &Engine) -> Vec<ffi::Car> {
    engine
        .city
        .vehicle_frames()
        .filter(|v| v.frame >= 0)
        .map(|v| ffi::Car {
            x: v.tile.x,
            y: v.tile.y,
            frame: v.frame,
            dx: v.offset_x,
            dy: v.offset_y,
        })
        .collect()
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

fn period(s: rustcity_core::stats::Stat) -> ffi::Period {
    ffi::Period {
        previous: s.value(),
        current: s.period_to_date(),
    }
}
fn inventory(i: &rustcity_core::stats::Inventory) -> ffi::Inventory {
    ffi::Inventory {
        amount: i.amount,
        capacity: i.capacity,
    }
}
fn statistics(engine: &Engine) -> ffi::Statistics {
    let c = &engine.city;
    let r = c.report();
    let s = c.statistics();
    let h = &s.history;
    let p = c.focus();
    ffi::Statistics {
        day: r.day,
        balance: r.balance,
        tech: r.tech_level,
        people_pool: r.people_pool,
        launched: r.rockets_launched,
        successes: r.successful_launches,
        game_end: c.game_ended(),
        survey: c.coal_survey_done(),
        name: c.scenario_name().into(),
        focus_x: p.x,
        focus_y: p.y,
        foreign_buy: Commodity::ALL.map(|c| engine.city.foreign_trade_policy(c).buy),
        foreign_sell: Commodity::ALL.map(|c| engine.city.foreign_trade_policy(c).sell),
        highest_tech_level: s.highest_tech_level,
        total_pollution: s.total_pollution,
        stats_tech: s.tech_level,
        stats_money: s.total_money,
        inventory: s
            .inventory
            .iter()
            .map(|i| ffi::InventoryPeriod {
                amount: period(i.amount),
                capacity: period(i.capacity),
            })
            .collect(),
        group_count: s.group_count.to_vec(),
        population: ffi::Population {
            population_m: period(s.population.population_m),
            housed_m: period(s.population.housed_m),
            starving_m: period(s.population.starving_m),
            unemployed_m: period(s.population.unemployed_m),
            housing_m: period(s.population.housing_m),
            births_m: period(s.population.births_m),
            deaths_m: period(s.population.deaths_m),
            unnat_deaths_m: period(s.population.unnat_deaths_m),
            evacuated_t: s.population.evacuated_t,
            births_t: s.population.births_t,
            pollution_deaths_t: s.population.pollution_deaths_t,
            starve_deaths_t: s.population.starve_deaths_t,
            unemployed_days_t: s.population.unemployed_days_t,
            max_pop_ever: s.population.max_pop_ever,
            unemployed_history: s.population.unemployed_history,
            pollution_deaths_history: s.population.pollution_deaths_history,
            starve_deaths_history: s.population.starve_deaths_history,
        },
        income: ffi::Income {
            income_tax: period(s.income.income_tax),
            coal_tax: period(s.income.coal_tax),
            goods_tax: period(s.income.goods_tax),
            export_tax: period(s.income.export_tax),
        },
        expenses: ffi::Expenses {
            construction: period(s.expenses.construction),
            coal_survey: period(s.expenses.coal_survey),
            import: period(s.expenses.import),
            unemployment: period(s.expenses.unemployment),
            transport: period(s.expenses.transport),
            windmill: period(s.expenses.windmill),
            university: period(s.expenses.university),
            recycle: period(s.expenses.recycle),
            deaths: period(s.expenses.deaths),
            health: period(s.expenses.health),
            rockets: period(s.expenses.rockets),
            school: period(s.expenses.school),
            firestation: period(s.expenses.firestation),
            cricket: period(s.expenses.cricket),
            interest: period(s.expenses.interest),
        },
        sustainability: ffi::Sustainability {
            mining_flag: s.sustainability.mining_flag,
            trade_flag: s.sustainability.trade_flag,
            sustainable: s.sustainability.sustainable,
            mining_years: s.sustainability.mining_years,
            trade_years: s.sustainability.trade_years,
            old_money: s.sustainability.old_money,
            money_years: s.sustainability.money_years,
            old_population: s.sustainability.old_population,
            population_years: s.sustainability.population_years,
            old_tech: s.sustainability.old_tech,
            tech_years: s.sustainability.tech_years,
            fire_years: s.sustainability.fire_years,
        },
        history: ffi::History {
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
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_snapshots_outlive_the_engine_and_each_other() {
        let engine = Box::new(Engine {
            city: rustcity_core::City::empty(5).unwrap(),
        });
        let first = tiles(&engine);
        let mut second = tiles(&engine);
        drop(engine);

        let original = first[0].altitude;
        second[0].altitude = original + 1;
        drop(second);
        assert_eq!(first[0].altitude, original);
    }
}
