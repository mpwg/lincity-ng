use rustcity_core::{BuildingKind, City, Point, Terrain};

#[test]
fn clipped_view_preserves_row_order_and_multitile_building_origin() {
    let city = City::load(include_bytes!("fixtures/blacksmith-2140.xml").as_slice()).unwrap();
    let edge: Vec<_> = city
        .map_region(-3..2, -1..1)
        .map(|tile| tile.point)
        .collect();
    assert_eq!(edge, [Point { x: 0, y: 0 }, Point { x: 1, y: 0 }]);
    assert_eq!(
        city.map_region(i32::MIN..i32::MAX, 0..1).count(),
        city.side() as usize
    );
    assert_eq!(city.map_region(2..2, 0..1).count(), 0);
    let origin = Point { x: 12, y: 12 };
    let tile = city.map_region(12..13, 12..13).next().unwrap();
    assert_eq!(tile.building, Some((BuildingKind::Blacksmith, origin)));
    let footprint = city.map_region(13..14, 13..14).next().unwrap();
    assert_eq!(footprint.building, tile.building);
    assert_eq!(tile.terrain, city.terrain(origin).unwrap());
    assert_eq!(
        city.map_region(6..7, 8..9).next().unwrap().terrain,
        Terrain::Water
    );
}

#[test]
fn host_can_expand_population_history_and_read_launch_totals() {
    let xml = r#"<lc-game ldsv-version="2140"><globals><rockets_launched>7</rockets_launched><rockets_launched_success>4</rockets_launched_success></globals><map size="8"/></lc-game>"#;
    let mut city = City::load(xml.as_bytes()).unwrap();
    let report = city.report();
    assert_eq!(
        (report.rockets_launched, report.successful_launches),
        (7, 4)
    );
    city.ensure_population_history(212);
    city.ensure_population_history(12);
    assert_eq!(city.statistics().history.pop.len(), 212);
    let mut checkpoint = Vec::new();
    city.save(&mut checkpoint).unwrap();
    let resumed = City::load(checkpoint.as_slice()).unwrap();
    assert_eq!(resumed.statistics().history.ppool.len(), 212);
}

#[test]
fn selected_building_exposes_status_and_stock_on_its_entire_footprint() {
    use rustcity_core::{Action, Commodity};
    let xml = r#"<lc-game ldsv-version="2140"><globals/><map size="32">
    <Construction group="6" map-x="5" map-y="5"><goods>987</goods></Construction>
    <Construction group="23" map-x="12" map-y="12"><stage>awaiting</stage></Construction>
    </map></lc-game>"#;
    let mut city = City::load(xml.as_bytes()).unwrap();
    let market = Point { x: 6, y: 6 };
    let selection = city.building_state(market).unwrap();
    assert_eq!(selection.origin, Point { x: 5, y: 5 });
    assert_eq!(selection.stock(Commodity::Goods), 987);
    assert!(!selection.evacuating);
    city.apply(Action::Evacuate, market).unwrap();
    assert!(city.building_state(market).unwrap().evacuating);
    assert!(
        city.building_state(Point { x: 13, y: 13 })
            .unwrap()
            .ready_to_launch
    );
    assert!(city.building_state(Point { x: 2, y: 2 }).is_none());
}
