use rustcity_core::{City, Commodity, Point, decisions::Decisions};
use std::io::Cursor;

#[test]
fn farm_constructor_draws_precede_placements_and_powered_farms_scale_to_fertile_area() {
    let xml=br#"<lc-game ldsv-version="2140"><globals/><map size="12">
        <MapTile group="0" map-x="4" map-y="4"><flags>0x20000000</flags></MapTile>
        <Construction group="19" map-x="2" map-y="2"/>
        <Construction group="5" map-x="4" map-y="4"><labor>13</labor><lovolt>50</lovolt><water>750</water><waste>18</waste><tech>1000000</tech></Construction>
        </map></lc-game>"#;
    let tape = b"LINCITY-SIMULATION-TRACE 1\nC 13\nC 52\nP 1 2 2 19\nP 2 4 4 5\nD 1 2 1 2\nZ\n";
    let mut c =
        City::load_with_decisions(&xml[..], Decisions::replay(Cursor::new(tape)).unwrap()).unwrap();
    c.tick().unwrap();
    let p = Point { x: 4, y: 4 };
    for (commodity, stock) in [
        (Commodity::Food, 1100),
        (Commodity::Water, 0),
        (Commodity::Labor, 0),
        (Commodity::LowVoltage, 0),
        (Commodity::Waste, 12),
    ] {
        assert_eq!(c.stock(p, commodity), Some(stock));
    }
    c.finish_decisions().unwrap();
}

#[test]
fn ecology_draws_once_for_visible_groundwater_desert_and_preserves_built_tiles() {
    let xml = br#"<lc-game ldsv-version="2140"><globals/><map size="8">
        <MapTile group="42" map-x="2" map-y="2"><flags>0x20000000</flags></MapTile>
        <MapTile group="42" map-x="3" map-y="2"><flags>0x20000000</flags></MapTile>
        <MapTile group="42" map-x="4" map-y="2"/>
        <Construction group="7" map-x="3" map-y="2"/>
        </map></lc-game>"#;
    let tape = b"LINCITY-SIMULATION-TRACE 1\nP 1 3 2 7\nD 1 1 1\nC 301\nZ\n";
    let mut city =
        City::load_with_decisions(&xml[..], Decisions::replay(Cursor::new(tape)).unwrap()).unwrap();
    city.tick().unwrap();
    use rustcity_core::Terrain::*;
    assert_eq!(city.terrain(Point { x: 2, y: 2 }), Some(Grass));
    assert_eq!(city.terrain(Point { x: 3, y: 2 }), Some(Desert));
    assert_eq!(city.terrain(Point { x: 4, y: 2 }), Some(Desert));
    city.finish_decisions().unwrap();
}
