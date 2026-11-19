//! A complete campaign through the public city API.
//!
//! Run with:
//! `cargo test --release --test sustainable_campaign -- --ignored --nocapture`
//! Set `RUSTCITY_CAMPAIGN_ARTIFACTS` to a directory to retain the original map
//! and the saves immediately before and after the victory.

use rustcity_core::{
    Action, BuildingKind, City, CityEvent, Climate, GenerationOptions, Point, decisions::Decisions,
};
use std::{fs, path::PathBuf};

const SEED: u64 = 48;
const SIDE: i32 = 100;
const VICTORY_DAY: i32 = 297_600;

#[test]
#[ignore = "runs a complete 297,600-day campaign; use a release build"]
fn generated_village_wins_a_sustainable_economy() {
    let mut city = City::generate_with_decisions(
        SIDE,
        GenerationOptions {
            climate: Climate::Swamp,
            village: true,
            trees: false,
        },
        Decisions::native(SEED),
    )
    .unwrap();
    let artifacts = std::env::var_os("RUSTCITY_CAMPAIGN_ARTIFACTS").map(PathBuf::from);
    if let Some(dir) = &artifacts {
        fs::create_dir_all(dir).unwrap();
        city.save(fs::File::create(dir.join("start.gz")).unwrap())
            .unwrap();
    }

    // Commands dated 0 are issued on the newly generated map, before its first tick.
    let mut actions = include_str!("fixtures/sustainable_campaign.tsv")
        .lines()
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            let [day, "build", x, y, group] = fields[..] else {
                panic!("invalid campaign command {line:?}");
            };
            (
                day.parse::<i32>().unwrap(),
                Action::Build(BuildingKind::from_id(group.parse().unwrap()).unwrap()),
                Point {
                    x: x.parse().unwrap(),
                    y: y.parse().unwrap(),
                },
            )
        })
        .peekable();

    for day in 0..VICTORY_DAY {
        while actions.peek().is_some_and(|entry| entry.0 == day) {
            let (_, action, site) = actions.next().unwrap();
            city.preview(action, site).unwrap_or_else(|reason| {
                panic!("day {day}: preview denied {action:?} at {site:?}: {reason}")
            });
            city.apply(action, site).unwrap_or_else(|reason| {
                panic!("day {day}: {action:?} failed at {site:?}: {reason}")
            });
        }
        if day == VICTORY_DAY - 1
            && let Some(dir) = &artifacts
        {
            city.save(fs::File::create(dir.join("before-victory.gz")).unwrap())
                .unwrap();
        }
        city.tick().unwrap();
        while let Some(event) = city.pop_event() {
            match event {
                CityEvent::SustainableEconomy(date) => {
                    assert_eq!(date, VICTORY_DAY);
                    assert!(actions.next().is_none(), "campaign has unplayed commands");
                    if let Some(dir) = &artifacts {
                        city.save(fs::File::create(dir.join("victory.gz")).unwrap())
                            .unwrap();
                    }
                    let city_report = city.report();
                    let stats = city.statistics();
                    println!("SustainableEconomy({date}): {city_report:?}");
                    println!(
                        "Population {}, housed {}, starving {}, {:?}",
                        stats.population.population_m.value() / rustcity_core::DAYS_PER_MONTH,
                        stats.population.housed_m.value() / rustcity_core::DAYS_PER_MONTH,
                        stats.population.starving_m.value() / rustcity_core::DAYS_PER_MONTH,
                        stats.sustainability,
                    );
                    return;
                }
                CityEvent::NoPeopleLeft(date) => panic!("village died out on day {date}"),
                _ => {}
            }
        }
    }
    panic!(
        "campaign did not win: {:?}",
        city.statistics().sustainability
    );
}
