use rustcity_core::{City, CityEvent, Point, decisions::Decisions};
use std::io::Cursor;

#[test]
fn saved_diffusion_set_processes_edge_and_interior_in_row_order() {
    let xml = r#"<lc-game ldsv-version="2140"><globals><total_time>2</total_time></globals><map size="5">
    <MapTile group="0" map-x="2" map-y="2" diffusion-active="1"><air_pol>160</air_pol></MapTile>
    <MapTile group="0" map-x="0" map-y="1" diffusion-active="1"><air_pol>130</air_pol></MapTile>
    <MapTile group="0" map-x="1" map-y="3"><air_pol>400</air_pol></MapTile></map></lc-game>"#;
    let tape = b"LINCITY-SIMULATION-TRACE 1\nD 3 0\nC 5\nZ\n";
    let choices = Decisions::replay(Cursor::new(tape)).unwrap();
    let mut city = City::load_with_decisions(xml.as_bytes(), choices).unwrap();
    city.tick().unwrap();
    for (x, y, n) in [(2, 2, 150), (1, 2, 10), (0, 1, 2), (1, 3, 400)] {
        assert_eq!(city.air_pollution(Point { x, y }), Some(n));
    }
    city.finish_decisions().unwrap();
}

#[test]
fn last_resident_leaving_ends_the_game_only_once_across_saves() {
    let xml = r#"<lc-game ldsv-version="2140"><globals><total_time>98</total_time><people_pool>0</people_pool></globals><map size="5"/></lc-game>"#;
    let mut city = City::load(xml.as_bytes()).unwrap();
    city.tick().unwrap();
    assert_eq!(city.pop_event(), Some(CityEvent::NoPeopleLeft(99)));
    let mut save = vec![];
    city.save(&mut save).unwrap();
    let mut city = City::load(save.as_slice()).unwrap();
    for _ in 0..100 {
        city.tick().unwrap();
    }
    assert_eq!(city.pop_event(), None);
}
