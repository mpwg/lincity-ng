use rustcity_core::{City, Commodity, Point};

fn load(day: i32, objects: &str) -> City {
    let xml = format!(
        r#"<lc-game ldsv-version="2140"><globals><total_time>{day}</total_time><total_money>500</total_money></globals><map size="32">{objects}</map></lc-game>"#
    );
    City::load(xml.as_bytes()).unwrap()
}

#[test]
fn rails_and_roads_use_goods_and_lose_low_voltage_at_distinct_rates() {
    let mut city = load(
        127,
        r#"<Construction group="9" map-x="3" map-y="3"><goods>2</goods><steel>2</steel><lovolt>9</lovolt></Construction>
        <Construction group="11" map-x="18" map-y="18"><goods>2</goods><lovolt>2</lovolt></Construction>"#,
    );
    city.tick().unwrap();
    let rail = Point { x: 3, y: 3 };
    let road = Point { x: 18, y: 18 };
    assert_eq!(city.stock(rail, Commodity::Waste), Some(2));
    assert_eq!(city.stock(rail, Commodity::LowVoltage), Some(6));
    assert_eq!(city.stock(road, Commodity::Waste), Some(1));
    assert_eq!(city.stock(road, Commodity::LowVoltage), Some(1));
}

#[test]
fn restored_power_line_spans_a_road_and_delivers_to_a_substation() {
    let mut city = load(
        1,
        r#"<Construction group="2" map-x="4" map-y="4"><hivolt>18000</hivolt></Construction>
        <Construction group="11" map-x="8" map-y="4"/>
        <Construction group="1" map-x="9" map-y="4"/>
        <Construction group="3" map-x="10" map-y="4"/>"#,
    );
    for _ in 0..4 {
        city.tick().unwrap();
    }
    assert!(
        city.stock(Point { x: 10, y: 4 }, Commodity::LowVoltage)
            .unwrap()
            > 0
    );
}

#[test]
fn network_view_reports_suspended_cables_and_commodity_traffic() {
    let mut city = load(
        1,
        r#"<Construction group="2" map-x="4" map-y="4"><hivolt>18000</hivolt></Construction>
        <Construction group="11" map-x="8" map-y="4"/><Construction group="1" map-x="9" map-y="4"/>
        <Construction group="3" map-x="10" map-y="4"/>"#,
    );
    let road = Point { x: 8, y: 4 };
    let wire = Point { x: 9, y: 4 };
    let tile = city.tile(road).unwrap();
    assert!(tile.east_west_cable);
    assert!(!tile.north_south_cable);
    assert_eq!(city.commodity_traffic(road, Commodity::Food), Some(0));
    assert_eq!(city.commodity_traffic(wire, Commodity::Food), None);
    assert_eq!(city.tile(Point { x: -1, y: 4 }), None);
    for _ in 0..4 {
        city.tick().unwrap();
    }
    assert!(
        city.commodity_traffic(wire, Commodity::HighVoltage)
            .unwrap()
            > 0
    );
}
