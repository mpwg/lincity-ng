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
