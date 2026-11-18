use rustcity_core::{Action, ActionError, City, Point, Terrain};

#[test]
fn preview_is_read_only_and_flooding_one_city_leaves_another_untouched() {
    let mut city = City::empty(8).unwrap();
    let other = City::empty(8).unwrap();
    let site = Point { x: 3, y: 4 };
    assert_eq!(city.preview(Action::Flood, site), Ok(1_000_000));
    assert_eq!(city.terrain(site), Some(Terrain::Grass));
    assert_eq!(city.balance(), 0);
    city.apply(Action::Flood, site).unwrap();
    assert_eq!(city.balance(), -1_000_000);
    assert_eq!(city.terrain(site), Some(Terrain::Water));
    assert_eq!(
        city.apply(Action::Flood, site),
        Err(ActionError::SpaceOccupied(site))
    );
    assert_eq!(other.balance(), 0);
    assert_eq!(other.terrain(site), Some(Terrain::Grass));
    for (terrain, balance) in [
        (Terrain::Grass, -2_000_000),
        (Terrain::Desert, -2_000_001),
        (Terrain::Desert, -2_000_001),
    ] {
        city.apply(Action::Bulldoze, site).unwrap();
        assert_eq!(
            (city.terrain(site), city.balance()),
            (Some(terrain), balance)
        );
    }
}

#[test]
fn border_is_stored_but_rejects_player_actions_with_original_coordinates() {
    assert!(City::empty(2).is_err());
    assert!(City::empty(46_341).is_err());
    let mut city = City::empty(3).unwrap();
    let edge = Point { x: 0, y: 1 };
    assert_eq!(city.terrain(edge), Some(Terrain::Grass));
    assert_eq!(
        city.apply(Action::Query, edge),
        Err(ActionError::OutsideMap(edge))
    );
    let outside = Point {
        x: i32::MIN,
        y: i32::MAX,
    };
    assert_eq!(city.terrain(outside), None);
    assert_eq!(
        city.preview(Action::Flood, outside),
        Err(ActionError::OutsideMap(outside))
    );
    let center = Point { x: 1, y: 1 };
    assert_eq!(
        city.preview(Action::Evacuate, center),
        Err(ActionError::NothingToEvacuate(center))
    );
}

#[test]
fn water_and_desert_frontiers_connect_only_adjacent_matching_ground() {
    let mut city = City::empty(8).unwrap();
    let site = Point { x: 3, y: 3 };
    let east = Point { x: 4, y: 3 };
    city.apply(Action::Flood, site).unwrap();
    city.apply(Action::Flood, east).unwrap();
    assert_eq!(city.ground_frame(site), Some(2));
    assert_eq!(city.ground_frame(east), Some(4));
    assert_eq!(city.has_groundwater(site), Some(true));
    city.apply(Action::Bulldoze, east).unwrap();
    assert_eq!(city.ground_frame(site), Some(0));
    assert_eq!(city.has_groundwater(east), Some(true));
}

#[test]
fn market_price_and_demolition_apply_across_the_whole_footprint() {
    use rustcity_core::BuildingKind;
    let mut city = City::empty(8).unwrap();
    let site = Point { x: 2, y: 3 };
    let covered = Point { x: 3, y: 4 };
    let build_market = Action::Build(BuildingKind::Market);
    assert_eq!(city.preview(build_market, site), Ok(100));
    city.apply(build_market, site).unwrap();
    assert_eq!(city.balance(), -100);
    assert_eq!(
        city.building_at(covered),
        Some((BuildingKind::Market, site))
    );
    assert_eq!(
        city.preview(build_market, covered),
        Err(ActionError::SpaceOccupied(covered))
    );
    assert_eq!(city.preview(Action::Bulldoze, covered), Ok(100));
    city.apply(Action::Bulldoze, covered).unwrap();
    assert_eq!(city.balance(), -200);
    assert_eq!(city.building_at(site), None);
    assert_eq!(city.building_at(covered), None);
    assert_eq!(city.preview(Action::Bulldoze, site), Ok(0));
    city.apply(build_market, site).unwrap();
}

#[test]
fn transactions_that_would_exceed_credit_leave_the_map_and_treasury_untouched() {
    let mut city = City::empty(4).unwrap();
    let site = Point { x: 1, y: 1 };
    for _ in 0..1000 {
        city.apply(Action::Flood, site).unwrap();
        city.apply(Action::Bulldoze, site).unwrap();
    }
    assert_eq!(city.balance(), -2_000_000_000);
    assert_eq!(city.preview(Action::Flood, site), Ok(1_000_000));
    assert_eq!(
        city.apply(Action::Flood, site),
        Err(ActionError::OutOfMoney)
    );
    assert_eq!(city.terrain(site), Some(Terrain::Grass));
    assert_eq!(city.balance(), -2_000_000_000);
}

#[test]
fn crossing_zero_queues_a_single_event_on_the_affected_city() {
    use rustcity_core::CityEvent;
    let input = include_bytes!("../../lincity-core/tests/fixtures/blacksmith-2140.xml");
    let mut city = City::load(input.as_slice()).unwrap();
    let p = Point { x: 3, y: 4 };
    city.apply(Action::Flood, p).unwrap();
    city.apply(Action::Bulldoze, p).unwrap();
    assert_eq!(city.pop_event(), Some(CityEvent::OutOfMoney));
    assert_eq!(city.pop_event(), None);
}

#[test]
fn flooding_connects_water_to_the_river_and_draining_removes_river_status() {
    let xml = r#"<lc-game ldsv-version="2140"><globals/><map size="12">
        <MapTile group="21" map-x="2" map-y="4"><flags>0x02000000</flags></MapTile>
        <MapTile group="21" map-x="0" map-y="4"/>
        <MapTile group="21" map-x="0" map-y="3"/>
        <MapTile group="21" map-x="1" map-y="2"/>
    </map></lc-game>"#;
    let mut city = City::load(xml.as_bytes()).unwrap();
    let connection = Point { x: 1, y: 4 };
    city.apply(Action::Flood, connection).unwrap();
    let river = |city: &City, x, y| city.map_region(x..x + 1, y..y + 1).next().unwrap().river;
    assert!(river(&city, 1, 4));
    assert!(river(&city, 0, 4));
    assert!(river(&city, 0, 3));
    assert!(!river(&city, 1, 2));
    city.apply(Action::Bulldoze, connection).unwrap();
    assert!(!river(&city, 1, 4));
    assert!(river(&city, 0, 4));
}

#[test]
fn evacuation_uses_the_saved_permission_before_the_first_tick() {
    use rustcity_core::BuildingKind;
    let xml = format!(
        r#"<lc-game ldsv-version="2140"><globals/><map size="16">
      <Construction group="{}" map-x="2" map-y="2"><flags>0x300</flags><completion>100</completion></Construction>
      <Construction group="{}" map-x="9" map-y="9"><flags>0</flags></Construction>
    </map></lc-game>"#,
        BuildingKind::Monument.id(),
        BuildingKind::Track.id()
    );
    let mut city = City::load(xml.as_bytes()).unwrap();
    let corner = Point { x: 3, y: 3 };
    assert_eq!(
        city.apply(Action::Evacuate, corner),
        Err(ActionError::CannotEvacuate(corner))
    );
    assert_eq!(city.apply(Action::Evacuate, Point { x: 9, y: 9 }), Ok(()));
}

#[test]
fn actions_without_a_payment_remain_available_below_the_credit_limit() {
    use rustcity_core::BuildingKind;
    let xml = format!(
        r#"<lc-game ldsv-version="2140"><globals><total_money>-2147483648</total_money></globals><map size="16">
      <Construction group="{}" map-x="2" map-y="2"/><MapTile group="42" map-x="9" map-y="9"/></map></lc-game>"#,
        BuildingKind::Market.id()
    );
    let mut city = City::load(xml.as_bytes()).unwrap();
    let market = Point { x: 2, y: 2 };
    assert_eq!(city.apply(Action::Query, market), Ok(()));
    assert_eq!(city.apply(Action::Evacuate, market), Ok(()));
    assert_eq!(city.apply(Action::Bulldoze, Point { x: 9, y: 9 }), Ok(()));
    assert_eq!(city.balance(), i32::MIN);
}

#[test]
fn invention_thresholds_and_citywide_eligibility_use_the_catalog_and_school_count() {
    use rustcity_core::{BuildDenial, BuildingKind};
    let rocket = BuildingKind::RocketPad.metadata().required_tech;
    assert_eq!(
        BuildingKind::inventions_after(rocket - 1).collect::<Vec<_>>(),
        [BuildingKind::RocketPad]
    );
    assert!(BuildingKind::inventions_after(rocket).next().is_none());
    let future: Vec<_> = BuildingKind::inventions_after(0).collect();
    assert_eq!(future.first(), Some(&BuildingKind::School));
    assert!(
        future
            .windows(2)
            .all(|kinds| kinds[0].metadata().required_tech <= kinds[1].metadata().required_tech)
    );
    assert!(!future.contains(&BuildingKind::TrackBridge));
    let city = City::empty(16).unwrap();
    assert_eq!(
        city.can_build(BuildingKind::University),
        Err(BuildDenial::NotEnoughStudents)
    );
    assert_eq!(
        city.can_build(BuildingKind::RocketPad),
        Err(BuildDenial::NotEnoughTech)
    );
    assert_eq!(city.can_build(BuildingKind::Track), Ok(()));
}

#[test]
fn map_search_uses_the_original_spiral_and_visible_footprints() {
    use rustcity_core::BuildingKind;
    let mut city = City::empty(32).unwrap();
    let p = Point { x: 8, y: 8 };
    assert_eq!(city.find_terrain(p, Terrain::Water), None);
    assert_eq!(city.find_bare_area(p, 2), Some(p));
    city.apply(Action::Flood, p).unwrap();
    assert_eq!(
        city.find_terrain(Point { x: 10, y: 8 }, Terrain::Water),
        Some(p)
    );
    assert_eq!(city.find_bare_area(p, 2), Some(Point { x: 9, y: 7 }));
    assert_eq!(city.find_bare_area(p, 31), None);
    let market = Point { x: 14, y: 10 };
    city.apply(Action::Build(BuildingKind::Market), market)
        .unwrap();
    assert_eq!(city.find_building(p, BuildingKind::Market), Some(market));
    assert_eq!(city.find_bare_area(market, i32::MAX), None);
}

#[test]
fn landforms_use_eight_neighbors_and_the_reference_edge_pairs() {
    fn sample(x: i32, y: i32, altitude: i32) -> String {
        format!(
            r#"<MapTile group="0" map-x="{x}" map-y="{y}"><altitude>{altitude}</altitude></MapTile>"#
        )
    }
    let center = Point { x: 8, y: 8 };
    let lake = Point { x: 12, y: 12 };
    let edge = Point { x: 1, y: 1 };
    let xml = format!(
        r#"<lc-game ldsv-version="2140"><globals/><map size="16">{}{}{}{}{}</map></lc-game>"#,
        sample(8, 8, 5),
        sample(8, 7, 10),
        sample(8, 9, 10),
        sample(12, 12, -1),
        sample(1, 1, -1)
    );
    let city = City::load(xml.as_bytes()).unwrap();
    let saddle = city.landform(center).unwrap();
    assert!(saddle.saddle && !saddle.maximum && !saddle.minimum);
    assert!(city.landform(Point { x: 8, y: 7 }).unwrap().maximum);
    assert!(city.landform(lake).unwrap().minimum);
    assert!(!city.landform(lake).unwrap().edge_minimum);
    assert!(city.landform(edge).unwrap().edge_minimum);
    assert!(city.landform(Point { x: 0, y: 1 }).is_none());
}
