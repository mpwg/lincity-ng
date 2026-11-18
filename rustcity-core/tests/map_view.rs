use rustcity_core::{BuildingKind, City, Point, Terrain};

#[test]
fn clipped_view_preserves_row_order_and_multitile_building_origin() {
    let city = City::load(
        include_bytes!("../../lincity-core/tests/fixtures/blacksmith-2140.xml").as_slice(),
    )
    .unwrap();
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
