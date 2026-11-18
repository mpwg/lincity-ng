//! The old flat accounts are charged once on load. Since the converted
//! document now holds net cash and year-to-date expenses it can be reloaded.
use super::{Element, SaveGame, integer, invalid};
use std::io;

fn field(e: &Element, name: &str, default: i32) -> io::Result<i32> {
    e.child(name)
        .map(|e| integer(&e.text()).and_then(|n| i32::try_from(n).map_err(invalid)))
        .transpose()
        .map(|v| v.unwrap_or(default))
}

fn legacy_path(name: &str) -> Option<&'static str> {
    Some(match name {
        "income_tax" => "taxable/labor",
        "coal_tax" => "taxable/coal",
        "goods_tax" => "taxable/goods",
        "export_tax" => "taxable/trade_ex",
        "ly_income_tax" => "stats/income/income_tax_ly",
        "ly_coal_tax" => "stats/income/coal_tax_ly",
        "ly_goods_tax" => "stats/income/goods_tax_ly",
        "ly_export_tax" => "stats/income/export_tax_ly",
        "ly_interest" => "stats/expenses/interest_ly",
        "highest_tech_level" => "stats/highest_tech_level",
        "total_pollution" => "stats/total_pollution",
        "tpopulation" => "stats/population/housed_mtd",
        "thousing" => "stats/population/housing_mtd",
        "tstarving_population" => "stats/population/starving_mtd",
        "tunemployed_population" => "stats/population/unemployed_mtd",
        "total_pollution_deaths" => "stats/population/pollution_deaths_total",
        "total_starve_deaths" => "stats/population/starve_deaths_total",
        "pollution_deaths_history" => "stats/population/pollution_deaths_history",
        "starve_deaths_history" => "stats/population/starve_deaths_history",
        "unemployed_history" => "stats/population/unemployed_history",
        "max_pop_ever" => "stats/population/max_pop_ever",
        "total_evacuated" => "stats/population/evacuated_total",
        "total_births" => "stats/population/births_total",
        "sust_dig_ore_coal_tip_flag" => "stats/sustainability/mining_flag",
        "sust_dig_ore_coal_count" => "stats/sustainability/mining_years",
        "sust_port_count" => "stats/sustainability/trade_years",
        "sust_old_money_count" => "stats/sustainability/money_years",
        "sust_old_population_count" => "stats/sustainability/population_years",
        "sust_old_tech_count" => "stats/sustainability/tech_years",
        "sust_fire_count" => "stats/sustainability/fire_years",
        "sust_old_money" => "stats/sustainability/old_money",
        "sust_port_flag" => "stats/sustainability/trade_flag",
        "sust_old_population" => "stats/sustainability/old_population",
        "sust_old_tech" => "stats/sustainability/old_tech",
        "sustain_flag" => "stats/sustainability/sustainable",
        _ => return None,
    })
}

fn account_expenses(old: &Element, new: &mut Element) -> io::Result<()> {
    let tech = field(old, "tech_level", 0)?;
    let starting_cash = field(old, "total_money", 0)?;
    let mut cash = starting_cash;
    let expenses = new.section("stats").section("expenses");
    for (source, account, multiplier, divisor) in [
        ("import", "import", field(old, "import_cost_rate", 1)?, 100),
        (
            "unemployment",
            "unemployment",
            field(old, "dole_rate", 15)?,
            100,
        ),
        (
            "transport",
            "transport",
            field(old, "transport_cost_rate", 14)?,
            100,
        ),
        ("windmill", "windmill", 1, 1),
        ("university", "university", 1, 1),
        ("recycle", "recycle", 1, 1),
        ("deaths", "deaths", 1, 1),
        ("health", "health", tech, 10_000),
        ("rocket_pad", "rockets", 1, 1),
        ("school", "school", 1, 1),
        ("fire", "firestation", tech, 10_000),
        ("cricket", "cricket", 1, 1),
    ] {
        let cost = field(old, &format!("{source}_cost"), 0)?;
        let amount =
            i32::try_from(i64::from(cost) * i64::from(multiplier) / divisor).map_err(invalid)?;
        cash = cash
            .checked_sub(amount)
            .filter(|n| *n >= -2_000_000_000)
            .ok_or_else(|| invalid("legacy expense exceeds credit limit"))?;
        expenses.set_child(&format!("{account}_ytd"), amount);
        if let Some(ly) = old.child(&format!("ly_{source}_cost")) {
            expenses.set_child(&format!("{account}_ly"), ly.text());
        }
    }
    new.set_child("total_money", cash);
    if cash != starting_cash {
        new.section("stats")
            .set_attr("month_end_money", starting_cash.to_string());
    }
    Ok(())
}

fn numbers(text: &str) -> io::Result<Vec<i32>> {
    text.split_ascii_whitespace()
        .map(|n| n.parse().map_err(invalid))
        .collect()
}

fn history(old: &Element, new: &mut Element) -> io::Result<()> {
    let history = new.section("stats").section("history");
    for name in ["pop", "starve", "nojobs", "ppool"] {
        if let Some(s) = old.child(&format!("monthgraph_{name}")) {
            let out = history.section(name);
            for (idx, num) in numbers(&s.text())?.into_iter().enumerate() {
                out.set_child(&format!("index_{idx}"), num);
            }
        }
    }
    let bars = [
        "pop",
        "tech",
        "money",
        "food",
        "labor",
        "goods",
        "coal",
        "ore",
        "steel",
        "pollution",
        "lovolt",
        "hivolt",
        "water",
        "waste",
        "tenants",
    ];
    for bar in old.elements().filter(|e| e.name == "pbar") {
        let id = bar
            .attr("id")
            .ok_or_else(|| invalid("missing legacy pbar id"))?
            .parse::<i32>()
            .map_err(invalid)?;
        if id == 0 {
            continue;
        }
        let Some(name) = bars.get(id as usize) else {
            continue;
        };
        let inventory = !matches!(*name, "tech" | "money" | "pollution");
        let length = if matches!(*name, "tech" | "money" | "tenants") {
            12
        } else {
            2
        };
        let values = if let Some(data) = bar.child("data") {
            let mut v = numbers(&data.text())?;
            v.reverse();
            let latest = *v.first().ok_or_else(|| invalid("empty legacy pbar data"))?;
            v.push(
                latest
                    .checked_sub(field(bar, "diff", 0)?)
                    .ok_or_else(|| invalid("pbar delta overflows"))?,
            );
            v
        } else {
            continue;
        };
        let series = history.section(name);
        for (i, v) in values.into_iter().take(length).enumerate() {
            let entry = series.section(&format!("index_{i}"));
            if inventory {
                entry.set_child("amount", v);
                entry.set_child("capacity", 1000);
            } else {
                entry.set_text(v.to_string());
            }
        }
    }
    Ok(())
}

fn globals(old: &Element) -> io::Result<Element> {
    let mut new = Element::new("globals");
    for e in old.elements() {
        let name = e.name.as_str();
        if let Some(path) = legacy_path(name) {
            let target = path
                .split('/')
                .fold(&mut new, |node, part| node.section(part));
            target.set_text(e.text());
        } else if matches!(
            name,
            "income_tax_rate"
                | "coal_tax_rate"
                | "dole_rate"
                | "transport_cost_rate"
                | "goods_tax_rate"
                | "export_tax_rate"
                | "import_cost_rate"
        ) {
            new.section("money_rates").push(e.clone());
        } else if matches!(name.split('_').next(), Some("import" | "export"))
            && name.ends_with("_enable")
        {
            let current = name
                .to_ascii_lowercase()
                .replace("lo-volt", "lovolt")
                .replace("hi-volt", "hivolt");
            new.section("tradeRules").set_child(&current, e.text());
        } else if !name.ends_with("_cost")
            && !name.starts_with("monthgraph_")
            && !matches!(
                name,
                "world_id"
                    | "climate"
                    | "old_setup_ground"
                    | "global_aridity"
                    | "global_mountainity"
                    | "world_side_len"
                    | "main_screen_originx"
                    | "main_screen_originy"
                    | "cheat_flag"
                    | "pbar"
                    | "total_unemployed_days"
                    | "total_unemployed_years"
            )
        {
            new.push(e.clone());
        }
    }
    let days = i64::from(field(old, "total_unemployed_days", 0)?)
        + 1200 * i64::from(field(old, "total_unemployed_years", 0)?);
    let population = new.section("stats").section("population");
    population.set_child(
        "unemployed_days_total",
        i32::try_from(days).map_err(invalid)?,
    );
    population.set_child("population_mtd", 1);
    account_expenses(old, &mut new)?;
    history(old, &mut new)?;
    Ok(new)
}

fn tile_flags(old: u32) -> u32 {
    let mut flags = old & (0x0100_0000 | 0x2000_0000);
    for (from, to) in [
        (0x100, 0x0003_0000),
        (0x0010_0000, 0x000c_0000),
        (0x0020_0000, 0x0030_0000),
        (0x0040_0000, 0x00c0_0000),
        (0x0080_0000, 0x0200_0000),
    ] {
        if old & from != 0 {
            flags |= to;
        }
    }
    flags
}

pub(super) fn upgrade(game: &mut SaveGame) -> io::Result<()> {
    let old = game.globals();
    let current = globals(old)?;
    let size = field(old, "world_side_len", super::LEGACY_DEFAULT_SIDE)?;
    let mut x = field(old, "main_screen_originx", size / 2)?;
    let mut y = field(old, "main_screen_originy", size / 2)?;
    // Stage the conversion so a malformed legacy field leaves the source intact.
    let mut map = game.map().clone();
    map.set_attr("size", size.to_string());
    for e in map.elements_mut() {
        if e.name == "Construction" {
            // Legacy placement overwrote the view position on each building.
            x = e.attr_i32("map-x")?;
            y = e.attr_i32("map-y")?;
        }
        let mask_flags = match e.name.as_str() {
            "MapTile" => tile_flags,
            "Construction" => |n| n & 0x0900_03e0,
            _ => continue,
        };
        if let Some(flags) = e.child_mut("flags") {
            let n = integer(&flags.text())?;
            if n < i64::from(i32::MIN) || n > i64::from(u32::MAX) {
                return Err(invalid("invalid legacy flags"));
            }
            flags.set_text(mask_flags(n as u32).to_string());
        }
    }
    let point = map.section("recentPoint");
    point.set_child("x", x);
    point.set_child("y", y);
    *game.map_mut() = map;
    *game.globals_mut() = current;
    Ok(())
}
