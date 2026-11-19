//! Convert RustCity observations to the dictionary and packed-array shapes used by Godot.
use std::{collections::VecDeque, fmt::Write};

use godot::prelude::*;
use rustcity_core::{
    City, Commodity, DAYS_PER_MONTH, DAYS_PER_YEAR, FIRE_YEARS, MAX_TECH_LEVEL, MINING_YEARS,
    MONEY_YEARS, POPULATION_YEARS, Point, ReportMode, ReportRow, TECHNOLOGY_YEARS, TRADE_YEARS,
    Terrain,
    stats::{History, Inventory},
};

use super::localized;

const COMMODITY_NAMES: [&str; Commodity::ALL.len()] = [
    "Food", "Labor", "Coal", "Goods", "Ore", "Steel", "Waste", "Lo-Volt", "Hi-Volt", "Water",
];

pub(super) fn tech_percent(value: i32) -> f64 {
    f64::from(value) * 100.0 / f64::from(MAX_TECH_LEVEL)
}

pub(super) fn snapshot(city: &City) -> VarDictionary {
    let side = city.side();
    let length = (side * side) as usize;
    let mut groups = vec![0; length];
    let mut anchors = vec![0; length];
    let mut pollution = vec![0; length];
    let mut operating = vec![0; length];
    let mut rockets = VarDictionary::new();
    let report = city.report();
    let mut population = i64::from(report.people_pool);
    let mut inventory = [Inventory::<i64>::default(); Commodity::ALL.len()];
    for (index, tile) in city.map_region(0..side, 0..side).enumerate() {
        let anchor = tile.building.map_or(tile.point, |(_, origin)| origin);
        groups[index] = i32::from(
            tile.building
                .map_or(tile.terrain.group(), |(kind, _)| kind.id()),
        );
        anchors[index] = anchor.y * side + anchor.x;
        pollution[index] = tile.pollution;
        if tile.building.is_some()
            && anchor == tile.point
            && let Some(building) = city.building_state(anchor)
        {
            operating[index] = u8::from(building.industrial_output_today > 0);
            population += i64::from(building.occupants);
            if let Some(frame) = building.rocket_frame {
                rockets.set(index as i32, i32::from(frame));
            }
            for (commodity, stock) in Commodity::ALL.into_iter().zip(&mut inventory) {
                stock.amount += i64::from(building.stock(commodity));
                stock.capacity += i64::from(building.kind.metadata().rule(commodity).max_load);
            }
        }
    }
    let stocks: VarArray = inventory
        .into_iter()
        .map(|stock| vdict! { "amount" => stock.amount, "capacity" => stock.capacity }.to_variant())
        .collect();
    let stats = city.statistics();
    let pop = &stats.population;
    let counts: PackedInt32Array = stats.group_count.iter().map(|&n| n as i32).collect();
    let focus = city.focus();
    vdict! {
        "rocket_frames" => &rockets,
        "tiles" => &PackedInt32Array::from(groups),
        "operating" => &PackedByteArray::from(operating),
        "anchors" => &PackedInt32Array::from(anchors),
        "pollution" => &PackedInt32Array::from(pollution),
        "side" => side,
        "time" => report.day,
        "money" => report.balance,
        "population" => population,
        "homeless" => report.people_pool,
        "tech" => report.tech_level,
        "tech_percent" => tech_percent(report.tech_level),
        "highest_tech_percent" => tech_percent(stats.highest_tech_level),
        "scenario" => city.scenario_name(),
        "unemployment" => pop.unemployed_m.value() / DAYS_PER_MONTH,
        "population_record" => pop.max_pop_ever,
        "evacuated" => pop.evacuated_t,
        "births" => pop.births_t,
        "deaths" => pop.starve_deaths_t + pop.pollution_deaths_t,
        "starvation_deaths" => pop.starve_deaths_t,
        "pollution_deaths" => pop.pollution_deaths_t,
        "unemployment_years" => f64::from(pop.unemployed_days_t) / f64::from(DAYS_PER_YEAR),
        "rockets" => report.rockets_launched,
        "successful_rockets" => report.successful_launches,
        "building_counts" => &counts,
        "inventory" => &stocks,
        "sustainable" => stats.sustainability.sustainable,
        "coal_survey_done" => city.coal_survey_done(),
        "focus" => Vector2i::new(focus.x, focus.y),
    }
}

pub(super) fn finances(city: &City) -> VarDictionary {
    let stats = city.statistics();
    let income = &stats.income;
    let expenses = &stats.expenses;
    vdict! {
        "income" => &vdict! {
            "Income tax" => income.income_tax.value(),
            "Coal tax" => income.coal_tax.value(),
            "Goods tax" => income.goods_tax.value(),
            "Exports" => income.export_tax.value(),
        },
        "expenses" => &vdict! {
            "Construction" => expenses.construction.value(),
            "Coal survey" => expenses.coal_survey.value(),
            "Imports" => expenses.import.value(),
            "Unemployment" => expenses.unemployment.value(),
            "Transport" => expenses.transport.value(),
            "Windmills" => expenses.windmill.value(),
            "Universities" => expenses.university.value(),
            "Recycling" => expenses.recycle.value(),
            "Deaths" => expenses.deaths.value(),
            "Health" => expenses.health.value(),
            "Rockets" => expenses.rockets.value(),
            "Schools" => expenses.school.value(),
            "Fire stations" => expenses.firestation.value(),
            "Sports" => expenses.cricket.value(),
            "Interest" => expenses.interest.value(),
        },
    }
}

fn export_series<T>(
    history: &History,
    completed_months: usize,
    key: &str,
    source: &VecDeque<T>,
    convert: impl Fn(&T) -> f64,
) -> PackedFloat64Array {
    let values: Vec<_> = source
        .iter()
        .take(completed_months)
        .enumerate()
        .map(|(index, value)| {
            if history.has_sample(key, index) {
                convert(value)
            } else {
                f64::NAN
            }
        })
        .collect();
    if values.iter().any(|value| value.is_finite()) {
        values.into()
    } else {
        PackedFloat64Array::new()
    }
}

pub(super) fn history(city: &City) -> VarDictionary {
    let stats = city.statistics();
    let history = &stats.history;
    let completed_months = (city.report().day / DAYS_PER_MONTH).max(0);
    let people = |days: &i32| f64::from(*days) / f64::from(DAYS_PER_MONTH);
    let tech = |value: &i32| tech_percent(*value);
    let raw = |value: &i32| f64::from(*value);
    let fill = |stock: &Inventory| {
        if stock.capacity > 0 {
            f64::from(stock.amount) * 100.0 / f64::from(stock.capacity)
        } else {
            f64::NAN
        }
    };
    let series = |key, source, convert: fn(&i32) -> f64| {
        export_series(history, completed_months as usize, key, source, convert)
    };
    let inventory = |key: &str, values: &VecDeque<Inventory>| {
        export_series(history, completed_months as usize, key, values, fill)
    };
    let values = vdict! {
        "population" => &series("pop", &history.pop, people),
        "homeless" => &series("ppool", &history.ppool, people),
        "starving" => &series("starve", &history.starve, people),
        "unemployed" => &series("nojobs", &history.nojobs, people),
        "births" => &series("births", &history.births, raw),
        "deaths" => &series("deaths", &history.deaths, raw),
        "unnatural_deaths" => &series("unnat_deaths", &history.unnat_deaths, raw),
        "balance" => &series("money", &history.money, raw),
        "tech" => &series("tech", &history.tech, tech),
        "pollution" => &series("pollution", &history.pollution, raw),
        "occupancy" => &inventory("tenants", &history.tenants),
    };
    let commodities: VarArray = Commodity::ALL
        .into_iter()
        .map(|commodity| {
            let index = commodity as usize;
            vdict! {
                "name" => &localized(COMMODITY_NAMES[index]),
                "values" => &inventory(commodity.save_name(), &history.inventory[index]),
            }
            .to_variant()
        })
        .collect();
    let sustain = &stats.sustainability;
    let criteria: VarArray = [
        ("Coal and ore mining", sustain.mining_years, MINING_YEARS),
        ("Port trade", sustain.trade_years, TRADE_YEARS),
        ("City finances", sustain.money_years, MONEY_YEARS),
        ("Population", sustain.population_years, POPULATION_YEARS),
        ("Technology", sustain.tech_years, TECHNOLOGY_YEARS),
        ("Fire protection", sustain.fire_years, FIRE_YEARS),
    ]
    .into_iter()
    .map(|(name, years, goal)| {
        vdict! { "name" => &localized(name), "years" => years, "goal" => goal }.to_variant()
    })
    .collect();
    vdict! {
        "last_month" => completed_months - 1,
        "series" => &values,
        "commodities" => &commodities,
        "sustainability" => &criteria,
        "sustainable" => sustain.sustainable,
    }
}

pub(super) fn trade_rules(city: &City, position: Point) -> VarArray {
    Commodity::ALL
        .into_iter()
        .filter_map(|commodity| {
            let policy = city.building_trade_policy(position, commodity)?;
            let index = commodity as usize;
            Some(
                vdict! {
                    "id" => index as i32,
                    "name" => &localized(COMMODITY_NAMES[index]),
                    "buy" => policy.buy,
                    "sell" => policy.sell,
                }
                .to_variant(),
            )
        })
        .collect()
}

fn terrain_labels(terrain: Terrain) -> (&'static str, &'static str) {
    match terrain {
        Terrain::Grass => ("Empty", "Green"),
        Terrain::Water => ("Water", "Water"),
        Terrain::Desert => ("Desert", "Desert"),
        Terrain::Tree => ("Tree", "Tree"),
        Terrain::Tree2 => ("Trees", "Tree2"),
        Terrain::Tree3 => ("Forest", "Tree3"),
    }
}

pub(super) fn tile_resource(city: &City, position: Point) -> GString {
    let Some(tile) = city.tile(position) else {
        return GString::new();
    };
    if let Some((kind, anchor)) = tile.building {
        return city
            .building_visual(anchor)
            .and_then(|visual| visual.layers.first().map(|layer| layer.resource))
            .unwrap_or(kind.metadata().resource)
            .into();
    }
    terrain_labels(tile.terrain).1.into()
}

pub(super) fn inspect_tile(city: &City, position: Point, production: bool) -> GString {
    let Some(tile) = city.tile(position) else {
        return GString::new();
    };
    let mut output = String::new();
    if let Some((_, anchor)) = tile.building {
        let mode = if production {
            ReportMode::Production
        } else {
            ReportMode::Inventory
        };
        for row in city.building_report(anchor, mode).unwrap_or_default() {
            append_report(&mut output, row);
        }
    } else {
        append_report(
            &mut output,
            ReportRow::Label(terrain_labels(tile.terrain).0),
        );
        append_report(&mut output, ReportRow::Integer("Elevation", tile.altitude));
        append_report(&mut output, ReportRow::Integer("Ore", tile.ore));
        if city.coal_survey_done() {
            append_report(&mut output, ReportRow::Integer("Coal", tile.coal));
        }
    }
    append_report(
        &mut output,
        ReportRow::Integer("Air pollution", tile.pollution),
    );
    output.trim_matches(|ch| ch <= ' ').into()
}

fn append_report(output: &mut String, row: ReportRow) {
    use ReportRow::*;
    // Formatting into a String is infallible.
    let _ = match row {
        Blank => writeln!(output),
        Label(label) => writeln!(output, "{}", localized(label)),
        Text(label, text) => writeln!(output, "{}  {}", localized(label), localized(text)),
        Integer(label, value) => writeln!(output, "{}  {value}", localized(label)),
        Decimal(label, value) => writeln!(output, "{}  {value:.1}", localized(label)),
        Percentage(label, value) => writeln!(output, "{}  {value:.1}%", localized(label)),
        Quantity(label, value, capacity) => {
            writeln!(output, "{}  {value} / {capacity}", localized(label))
        }
        PrefixedQuantity(prefix, label, value, capacity) => {
            writeln!(
                output,
                "{prefix} {}  {value} / {capacity}",
                localized(label)
            )
        }
    };
}
