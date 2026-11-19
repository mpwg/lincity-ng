//! Bind the runtime reporting periods to their distinct persisted ledger entries.
use super::{Element, integer, invalid};
use crate::Commodity;
use crate::rules::DAYS_PER_MONTH;
use crate::rules::economy;
use crate::stats::{History, Inventory, Stat, Stats};
use std::{collections::VecDeque, io};

fn periods(s: &mut Stats) -> [(&'static str, &'static str, &mut Stat); 27] {
    [
        ("population", "population", &mut s.population.population_m),
        ("population", "housed", &mut s.population.housed_m),
        ("population", "starving", &mut s.population.starving_m),
        ("population", "unemployed", &mut s.population.unemployed_m),
        ("population", "housing", &mut s.population.housing_m),
        ("population", "births", &mut s.population.births_m),
        ("population", "deaths", &mut s.population.deaths_m),
        (
            "population",
            "unnat_deaths",
            &mut s.population.unnat_deaths_m,
        ),
        ("income", "income_tax", &mut s.income.income_tax),
        ("income", "coal_tax", &mut s.income.coal_tax),
        ("income", "goods_tax", &mut s.income.goods_tax),
        ("income", "export_tax", &mut s.income.export_tax),
        ("expenses", "construction", &mut s.expenses.construction),
        ("expenses", "coalSurvey", &mut s.expenses.coal_survey),
        ("expenses", "import", &mut s.expenses.import),
        ("expenses", "unemployment", &mut s.expenses.unemployment),
        ("expenses", "transport", &mut s.expenses.transport),
        ("expenses", "windmill", &mut s.expenses.windmill),
        ("expenses", "university", &mut s.expenses.university),
        ("expenses", "recycle", &mut s.expenses.recycle),
        ("expenses", "deaths", &mut s.expenses.deaths),
        ("expenses", "health", &mut s.expenses.health),
        ("expenses", "rockets", &mut s.expenses.rockets),
        ("expenses", "school", &mut s.expenses.school),
        ("expenses", "firestation", &mut s.expenses.firestation),
        ("expenses", "cricket", &mut s.expenses.cricket),
        ("expenses", "interest", &mut s.expenses.interest),
    ]
}

fn counters(s: &mut Stats) -> [(&'static str, &'static str, &mut i32); 17] {
    [
        ("", "highest_tech_level", &mut s.highest_tech_level),
        ("", "total_pollution", &mut s.total_pollution),
        (
            "population",
            "evacuated_total",
            &mut s.population.evacuated_t,
        ),
        ("population", "births_total", &mut s.population.births_t),
        (
            "population",
            "pollution_deaths_total",
            &mut s.population.pollution_deaths_t,
        ),
        (
            "population",
            "starve_deaths_total",
            &mut s.population.starve_deaths_t,
        ),
        (
            "population",
            "unemployed_days_total",
            &mut s.population.unemployed_days_t,
        ),
        ("population", "max_pop_ever", &mut s.population.max_pop_ever),
        (
            "sustainability",
            "mining_years",
            &mut s.sustainability.mining_years,
        ),
        (
            "sustainability",
            "trade_years",
            &mut s.sustainability.trade_years,
        ),
        (
            "sustainability",
            "old_money",
            &mut s.sustainability.old_money,
        ),
        (
            "sustainability",
            "money_years",
            &mut s.sustainability.money_years,
        ),
        (
            "sustainability",
            "old_population",
            &mut s.sustainability.old_population,
        ),
        (
            "sustainability",
            "population_years",
            &mut s.sustainability.population_years,
        ),
        ("sustainability", "old_tech", &mut s.sustainability.old_tech),
        (
            "sustainability",
            "tech_years",
            &mut s.sustainability.tech_years,
        ),
        (
            "sustainability",
            "fire_years",
            &mut s.sustainability.fire_years,
        ),
    ]
}

fn decays(s: &mut Stats) -> [(&'static str, &'static str, &mut f32); 3] {
    [
        (
            "population",
            "unemployed_history",
            &mut s.population.unemployed_history,
        ),
        (
            "population",
            "pollution_deaths_history",
            &mut s.population.pollution_deaths_history,
        ),
        (
            "population",
            "starve_deaths_history",
            &mut s.population.starve_deaths_history,
        ),
    ]
}

fn flags(s: &mut Stats) -> [(&'static str, &'static str, &mut bool); 3] {
    [
        (
            "sustainability",
            "trade_flag",
            &mut s.sustainability.trade_flag,
        ),
        (
            "sustainability",
            "mining_flag",
            &mut s.sustainability.mining_flag,
        ),
        (
            "sustainability",
            "sustainable",
            &mut s.sustainability.sustainable,
        ),
    ]
}

fn histories(stats: &mut Stats) -> [(&'static str, &mut VecDeque<i32>); 10] {
    let h = &mut stats.history;
    [
        ("pop", &mut h.pop),
        ("nojobs", &mut h.nojobs),
        ("starve", &mut h.starve),
        ("ppool", &mut h.ppool),
        ("tech", &mut h.tech),
        ("money", &mut h.money),
        ("pollution", &mut h.pollution),
        ("births", &mut h.births),
        ("deaths", &mut h.deaths),
        ("unnat_deaths", &mut h.unnat_deaths),
    ]
}

fn subsection<'a>(node: &'a Element, section: &str) -> Option<&'a Element> {
    if section.is_empty() {
        Some(node)
    } else {
        node.child(section)
    }
}

fn read_series<T: Default>(
    node: &Element,
    values: &mut VecDeque<T>,
    grow: bool,
    mut read: impl FnMut(&Element, &mut T) -> io::Result<()>,
) -> io::Result<VecDeque<bool>> {
    let availability = node.attr("samples");
    let mut samples = VecDeque::from(vec![false; values.len()]);
    for entry in node.elements() {
        let Some(i) = entry
            .name
            .strip_prefix("index_")
            .and_then(|n| n.parse::<usize>().ok())
        else {
            continue;
        };
        if i == values.len() && grow {
            values.push_back(T::default());
            samples.push_back(false);
        }
        if let Some(value) = values.get_mut(i) {
            read(entry, value)?;
            samples[i] = availability
                .as_ref()
                .is_none_or(|bits| bits.as_bytes().get(i) == Some(&b'1'));
        }
    }
    Ok(samples)
}

fn read_stock(entry: &Element, stock: &mut Inventory) -> io::Result<()> {
    stock.amount = entry.child_i32("amount", stock.amount)?;
    stock.capacity = entry.child_i32("capacity", stock.capacity)?;
    Ok(())
}

pub(crate) fn load(s: &mut Stats, root: &Element) -> io::Result<()> {
    if let Some(cash) = root.attr("month_end_money") {
        s.total_money = i32::try_from(integer(cash)?).map_err(invalid)?;
    }
    for (group, key, period) in periods(s) {
        if let Some(node) = root.child(group) {
            let (last, acc) = if group == "population" {
                ("lm", "mtd")
            } else {
                ("ly", "ytd")
            };
            period.stat = node.child_i32(&format!("{key}_{last}"), period.stat)?;
            period.acc = node.child_i32(&format!("{key}_{acc}"), period.acc)?;
        }
    }
    for (group, key, n) in counters(s) {
        if let Some(node) = subsection(root, group) {
            *n = node.child_i32(key, *n)?;
        }
    }
    for (group, key, n) in decays(s) {
        if let Some(node) = root.child(group).and_then(|g| g.child(key)) {
            *n = node.text().parse().map_err(invalid)?;
        }
    }
    for (group, key, b) in flags(s) {
        if let Some(node) = root.child(group).and_then(|g| g.child(key)) {
            *b = node.boolean()?;
        }
    }
    if let Some(history) = root.child("history") {
        let mut loaded_samples = std::collections::HashMap::new();
        for (name, values) in histories(s) {
            if let Some(node) = history.child(name) {
                let samples = read_series(
                    node,
                    values,
                    matches!(name, "pop" | "ppool" | "starve" | "nojobs"),
                    |e, n| {
                        *n = e.i32()?;
                        Ok(())
                    },
                )?;
                loaded_samples.insert(name, samples);
            }
        }
        for (i, c) in Commodity::ALL.into_iter().enumerate() {
            if let Some(node) = history.child(c.save_name()) {
                if node.attr("amount_mtd").is_some() {
                    s.inventory[i].amount.acc = node.attr_i32("amount_mtd")?;
                }
                if node.attr("capacity_mtd").is_some() {
                    s.inventory[i].capacity.acc = node.attr_i32("capacity_mtd")?;
                }
                let samples = read_series(node, &mut s.history.inventory[i], false, read_stock)?;
                loaded_samples.insert(c.save_name(), samples);
            }
        }
        if let Some(node) = history.child("tenants") {
            let samples = read_series(node, &mut s.history.tenants, false, read_stock)?;
            loaded_samples.insert("tenants", samples);
        }
        s.history.sample_presence = loaded_samples;
    }
    Ok(())
}

fn write_series<T>(
    node: &mut Element,
    values: &VecDeque<T>,
    history: &History,
    name: &str,
    mut set: impl FnMut(&mut Element, &T),
) {
    node.set_attr("size", values.len().to_string());
    // Preserve gaps while keeping numeric slots readable by older clients.
    let samples: String = (0..values.len())
        .map(|i| {
            if history.has_sample(name, i) {
                '1'
            } else {
                '0'
            }
        })
        .collect();
    if samples.contains('0') {
        node.set_attr("samples", samples);
    } else {
        node.remove_attr("samples");
    }
    for (i, v) in values.iter().enumerate() {
        set(node.section(&format!("index_{i}")), v);
    }
}

fn write_stock(e: &mut Element, stock: &Inventory) {
    e.set_child("amount", stock.amount);
    e.set_child("capacity", stock.capacity);
}

pub(crate) fn store(s: &Stats, root: &mut Element, day: i32, cash: i32) {
    let end = day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1;
    if end && s.total_money != cash {
        root.set_attr("month_end_money", s.total_money.to_string());
    } else {
        root.remove_attr("month_end_money");
    }
    // The same field bindings drive reading and writing; the city remains immutably borrowed.
    let mut report = s.clone();
    for (group, key, period) in periods(&mut report) {
        let node = root.section(group);
        let (last, acc) = if group == "population" {
            ("lm", "mtd")
        } else {
            ("ly", "ytd")
        };
        node.set_child(&format!("{key}_{last}"), period.stat);
        node.set_child(&format!("{key}_{acc}"), period.acc);
    }
    for (group, key, n) in counters(&mut report) {
        if group.is_empty() {
            root.set_child(key, n);
        } else {
            root.section(group).set_child(key, n);
        }
    }
    for (group, key, n) in decays(&mut report) {
        root.section(group).set_child(key, n);
    }
    for (group, key, b) in flags(&mut report) {
        root.section(group).set_child(key, b);
    }
    let history = root.section("history");
    for (name, values) in histories(&mut report) {
        write_series(history.section(name), values, &s.history, name, |e, v| {
            e.set_text(v.to_string())
        });
    }
    for (i, c) in Commodity::ALL.into_iter().enumerate() {
        let node = history.section(c.save_name());
        for (key, v) in [
            ("amount_mtd", s.inventory[i].amount.acc),
            ("capacity_mtd", s.inventory[i].capacity.acc),
        ] {
            if end && v != 0 {
                node.set_attr(key, v.to_string());
            } else {
                node.remove_attr(key);
            }
        }
        write_series(
            node,
            &s.history.inventory[i],
            &s.history,
            c.save_name(),
            write_stock,
        );
    }
    write_series(
        history.section("tenants"),
        &s.history.tenants,
        &s.history,
        "tenants",
        write_stock,
    );
}

pub(crate) fn complete_globals(globals: &mut Element) {
    globals.default_child("given_scene", "");
    for key in [
        "rockets_launched",
        "rockets_launched_success",
        "coal_survey_done",
    ] {
        globals.default_child(key, 0);
    }
    globals.default_child("game_end", false);
    let rates = globals.section("money_rates");
    let defaults = [
        ("income_tax_rate", economy::INCOME_RATE),
        ("coal_tax_rate", economy::COAL_RATE),
        ("dole_rate", economy::DOLE_RATE),
        ("transport_cost_rate", economy::TRANSPORT_RATE),
        ("goods_tax_rate", economy::GOODS_RATE),
        ("export_tax_rate", 0),
        ("import_cost_rate", economy::IMPORT_RATE),
    ];
    for (key, value) in defaults {
        rates.default_child(key, value);
    }
    rates.order_children(&defaults.map(|(name, _)| name));
    let taxable = globals.section("taxable");
    let taxes = ["labor", "coal", "goods", "trade_ex"];
    for name in taxes {
        taxable.default_child(name, 0);
    }
    taxable.order_children(&taxes);
    let rules = globals.section("tradeRules");
    let names: Vec<_> = Commodity::ALL
        .iter()
        .flat_map(|c| ["import", "export"].map(|side| format!("{side}_{}_enable", c.save_name())))
        .collect();
    for key in &names {
        rules.default_child(key, 1);
    }
    rules.order_children(&names.iter().map(String::as_str).collect::<Vec<_>>());
    let stats = globals.section("stats");
    stats.order_children(&[
        "highest_tech_level",
        "total_pollution",
        "population",
        "income",
        "expenses",
        "sustainability",
        "history",
    ]);
    let mut binding = Stats::default();
    for group in ["population", "income", "expenses"] {
        let (last, acc) = if group == "population" {
            ("lm", "mtd")
        } else {
            ("ly", "ytd")
        };
        let mut names = Vec::new();
        for (g, key, _) in periods(&mut binding) {
            if g == group {
                names.push(format!("{key}_{last}"));
                names.push(format!("{key}_{acc}"));
            }
        }
        if group == "population" {
            names.extend(
                [
                    "evacuated_total",
                    "births_total",
                    "pollution_deaths_total",
                    "starve_deaths_total",
                    "unemployed_days_total",
                    "unemployed_history",
                    "pollution_deaths_history",
                    "starve_deaths_history",
                    "max_pop_ever",
                ]
                .map(str::to_owned),
            );
        }
        stats
            .section(group)
            .order_children(&names.iter().map(String::as_str).collect::<Vec<_>>());
    }
    stats.section("sustainability").order_children(&[
        "mining_flag",
        "mining_years",
        "trade_flag",
        "trade_years",
        "old_money",
        "money_years",
        "old_population",
        "population_years",
        "old_tech",
        "tech_years",
        "fire_years",
        "sustainable",
    ]);
    let mut series = vec![
        "pop",
        "starve",
        "nojobs",
        "ppool",
        "tech",
        "money",
        "pollution",
        "births",
        "deaths",
        "unnat_deaths",
        "tenants",
    ];
    series.extend(Commodity::ALL.into_iter().map(Commodity::save_name));
    stats.section("history").order_children(&series);
    globals.order_children(&[
        "given_scene",
        "total_time",
        "people_pool",
        "total_money",
        "tech_level",
        "rockets_launched",
        "rockets_launched_success",
        "coal_survey_done",
        "game_end",
        "money_rates",
        "taxable",
        "tradeRules",
        "stats",
    ]);
}
