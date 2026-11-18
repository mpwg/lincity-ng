use rustcity_core::{Action, City, Commodity, Point, Terrain};

const SITE: Point = Point { x: 6, y: 6 };

fn load(nodes: &str) -> City {
    let xml = format!(
        r#"<lc-game ldsv-version="2140"><globals><total_money>9</total_money></globals><map size="22">{nodes}</map></lc-game>"#
    );
    City::load(xml.as_bytes()).unwrap()
}

#[test]
fn a_buried_load_survives_save_and_blocks_bulldozing() {
    let mut city =
        load(r#"<Construction group="16" map-x="6" map-y="6"><waste>1000</waste></Construction>"#);
    city.tick().unwrap();
    assert_eq!(city.stock(SITE, Commodity::Waste), Some(800));
    let mut saved = Vec::new();
    city.save(&mut saved).unwrap();
    let mut city = City::load(saved.as_slice()).unwrap();
    assert!(city.preview(Action::Bulldoze, SITE).is_err());
    assert!(city.apply(Action::Bulldoze, SITE).is_err());
    assert_eq!(city.balance(), 9);
}

#[test]
fn flooded_ore_pit_preserves_deep_deposits_after_bulldozing() {
    let mut city = load(
        r#"<MapTile group="0" map-x="6" map-y="6"><ore>500</ore></MapTile>
        <Construction group="15" map-x="6" map-y="6"/>"#,
    );
    city.apply(Action::Bulldoze, Point { x: 7, y: 7 }).unwrap();
    assert_eq!(city.terrain(SITE), Some(Terrain::Desert));
    assert_eq!(city.terrain(Point { x: 7, y: 7 }), Some(Terrain::Water));
    assert_eq!(city.has_groundwater(Point { x: 7, y: 7 }), Some(true));
}

#[test]
fn city_operates_steel_works_and_charges_recycling_every_day() {
    let mut city = load(
        r#"<Construction group="18" map-x="6" map-y="6"><labor>46</labor><ore>612</ore><hivolt>3600</hivolt></Construction>
        <Construction group="20" map-x="14" map-y="14"/>"#,
    );
    city.tick().unwrap();
    assert_eq!(city.stock(SITE, Commodity::Steel), Some(36));
    assert_eq!(city.balance(), 6);
}

#[test]
fn exhausting_a_mine_evacuates_and_removes_it_after_the_last_ore_is_delivered() {
    let mut city = load(
        r#"<MapTile group="0" map-x="6" map-y="6"><ore>1</ore></MapTile>
        <MapTile group="21" map-x="5" map-y="6"><flags>0x02000000</flags></MapTile>
        <Construction group="15" map-x="6" map-y="6"><labor>209</labor></Construction>
        <Construction group="7" map-x="10" map-y="6"/><Construction group="7" map-x="6" map-y="10"/>"#,
    );
    for _ in 0..16 {
        city.tick().unwrap();
        if city.building_at(SITE).is_none() {
            break;
        }
    }
    assert_eq!(city.building_at(SITE), None);
    assert_eq!(city.terrain(SITE), Some(Terrain::Water));
}
