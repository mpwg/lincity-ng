use rustcity_core::{Action, BuildingKind, City, Point, Terrain};
const SMITHY: &[u8] = include_bytes!("../../lincity-core/tests/fixtures/blacksmith-2140.xml");
const LEGACY: &[u8] = include_bytes!("../../lincity-core/tests/fixtures/migration-2130.xml");

#[test]
fn changes_and_actions_continue_after_loading_a_saved_city() {
    let mut city = City::load(SMITHY).unwrap();
    let home = Point { x: 12, y: 12 };
    let water = Point { x: 6, y: 8 };
    assert_eq!(city.balance(), 1703);
    assert_eq!(city.terrain(water), Some(Terrain::Water));
    assert_eq!(
        city.building_at(home),
        Some((BuildingKind::Blacksmith, home))
    );
    city.apply(Action::Bulldoze, home).unwrap();
    city.apply(Action::Build(BuildingKind::Track), home)
        .unwrap();
    let mut bytes = Vec::new();
    city.save(&mut bytes).unwrap();
    let mut resumed = City::load(bytes.as_slice()).unwrap();
    assert_eq!(resumed.balance(), city.balance());
    assert_eq!(resumed.building_at(home), Some((BuildingKind::Track, home)));
    for c in [&mut city, &mut resumed] {
        c.apply(Action::Bulldoze, home).unwrap();
    }
    assert_eq!(resumed.balance(), city.balance());
    assert_eq!(resumed.building_at(home), None);
}

#[test]
fn legacy_expenses_are_only_charged_once() {
    let city = City::load(LEGACY).unwrap();
    assert_eq!(city.balance(), 1501);
    let mut bytes = Vec::new();
    city.save(&mut bytes).unwrap();
    assert_eq!(City::load(bytes.as_slice()).unwrap().balance(), 1501);
}

#[test]
fn smithy_stocks_continue_across_the_month_boundary_after_a_save() {
    use rustcity_core::Commodity;
    let home = Point { x: 12, y: 12 };
    let mut city = City::load(SMITHY).unwrap();
    for _ in 0..3 {
        city.tick();
    }
    assert_eq!(city.stock(home, Commodity::Goods), Some(157));
    let mut checkpoint = Vec::new();
    city.save(&mut checkpoint).unwrap();
    let mut resumed = City::load(checkpoint.as_slice()).unwrap();
    for day in 4..=5 {
        for c in [&mut city, &mut resumed] {
            c.tick();
            assert_eq!(c.stock(home, Commodity::Labor), Some(175 - day * 35));
            assert_eq!(c.stock(home, Commodity::Goods), Some(7 + day * 50));
        }
    }
}

#[test]
fn historical_market_saves_policy_in_reference_commodity_order() {
    use std::io::Read;
    let city =
        City::load(include_bytes!("../../lincity-core/opening/extreme_arid.scn.gz").as_slice())
            .unwrap();
    let mut checkpoint = Vec::new();
    city.save(&mut checkpoint).unwrap();
    let mut xml = String::new();
    flate2::read::GzDecoder::new(checkpoint.as_slice())
        .read_to_string(&mut xml)
        .unwrap();
    let market = xml
        .split("<Construction group=\"6\"")
        .nth(1)
        .unwrap()
        .split("</Construction>")
        .next()
        .unwrap();
    let mut fields = market;
    for commodity in [
        "food", "labor", "coal", "goods", "ore", "steel", "waste", "water",
    ] {
        for direction in ["give", "take"] {
            let tag = format!("<{direction}_{commodity}>");
            fields = fields
                .split_once(&tag)
                .unwrap_or_else(|| panic!("missing or out of order: {tag}"))
                .1;
        }
    }
}
