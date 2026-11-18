//! Text adapter for the versioned cross-language fixtures.
use lincity_core_slice::{Commodity, Inventory, PowerChain, Stats, Substation};
use std::{env, error::Error, fs, str::SplitWhitespace};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn number(fields: &mut SplitWhitespace<'_>) -> Result<i32> {
    Ok(fields.next().ok_or("expected integer")?.parse()?)
}

fn inventory(fields: &mut SplitWhitespace<'_>) -> Result<Inventory> {
    Ok(Inventory {
        amount: number(fields)?,
        capacity: number(fields)?,
    })
}

fn names_case() {
    for c in Commodity::ALL {
        let name = c.standard_name();
        let parsed = Commodity::from_standard_name(name).expect("canonical commodity");
        println!("{name} {}", parsed as usize);
    }
    for invalid in ["Food", "", "unknown"] {
        println!(
            "{}",
            Commodity::from_standard_name(invalid).map_or("unknown", Commodity::standard_name)
        );
    }
}

fn inventory_case(fields: &mut SplitWhitespace<'_>) -> Result<()> {
    let mut value = inventory(fields)?;
    let print = |i: Inventory| println!("{} {} {}", i.amount, i.capacity, i.permille());
    print(value);
    value += inventory(fields)?;
    print(value);
    value -= inventory(fields)?;
    print(value);
    Ok(())
}

fn stats_case(fields: &mut SplitWhitespace<'_>) -> Result<()> {
    let mut s = Stats::default();
    s.population.current = number(fields)?;
    s.housed.current = number(fields)?;
    s.food = inventory(fields)?;
    s.goods_tax.current = number(fields)?;
    s.transport.current = number(fields)?;
    s.births.current = number(fields)?;
    s.unemployment_history = number(fields)? as f32;
    for boundary in fields.next().ok_or("expected boundary sequence")?.chars() {
        match boundary {
            'd' => s.daily(),
            'm' => s.monthly(),
            'y' => s.yearly(),
            _ => return Err("unknown statistics boundary".into()),
        }
        println!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            s.population.current,
            s.population.previous,
            s.population_history.len(),
            s.population_history[0],
            s.population_history[1],
            s.pool_history[0],
            s.food.amount,
            s.food.capacity,
            s.food_history.len(),
            s.food_history[0].amount,
            s.food_history[0].capacity,
            s.food_history[1].permille(),
            s.goods_tax.current,
            s.goods_tax.previous,
            s.transport.current,
            s.transport.previous,
            s.births.current,
            s.births.previous,
            s.unemployment_history.to_bits()
        );
    }
    Ok(())
}

fn chain_case(fields: &mut SplitWhitespace<'_>) -> Result<()> {
    let mut chain = PowerChain {
        labor: number(fields)?,
        high: number(fields)?,
        station: Some(Substation {
            high: number(fields)?,
            low: number(fields)?,
        }),
    };
    let ticks = number(fields)?;
    let remove_at = number(fields)?;
    for tick in 0..=ticks {
        if tick > 0 {
            if tick == remove_at {
                chain.remove_station();
            }
            chain.tick();
        }
        print!("{tick} {} {} ", chain.labor, chain.high);
        match &chain.station {
            Some(station) => println!("{} {}", station.high, station.low),
            None => println!("removed"),
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let input = fs::read_to_string(env::args().nth(1).ok_or("expected case file")?)?;
    let mut lines = input.lines();
    if lines.next() != Some("version 1") {
        return Err("unsupported case version".into());
    }
    println!("version 1");
    for line in lines {
        println!("{line}");
        let mut fields = line.split_whitespace();
        match fields.next() {
            Some("names") => names_case(),
            Some("inventory") => inventory_case(&mut fields)?,
            Some("stats") => stats_case(&mut fields)?,
            Some("chain") => chain_case(&mut fields)?,
            _ => return Err(format!("unknown case: {line}").into()),
        }
        if fields.next().is_some() {
            return Err(format!("unexpected argument: {line}").into());
        }
    }
    Ok(())
}
