use rustcity_core::{City, CityEvent, Commodity, Point, decisions::Decisions};
use std::io::Cursor;

#[test]
fn launch_requires_completed_pad_and_animation_before_simulation_decides() {
    let xml = r#"<lc-game ldsv-version="2140"><globals><tech_level>1000000</tech_level></globals><map size="16">
    <Construction group="23" map-x="3" map-y="3"><steps>4999</steps><labor>200</labor><steel>240</steel><goods>10000</goods></Construction></map></lc-game>"#;
    let tape=b"LINCITY-SIMULATION-TRACE 1\nP 1 3 3 23\nD 1 1 1\nA 1000 1 1\nA 2000 1 1\nA 3000 1 1\nD 2 1 1\nS rocket.success 1\nD 3 1 1\nZ\n";
    let mut city = City::load_with_decisions(
        xml.as_bytes(),
        Decisions::replay(Cursor::new(tape)).unwrap(),
    )
    .unwrap();
    let origin = Point { x: 3, y: 3 };
    assert_eq!(city.building_state(origin).unwrap().rocket_frame, Some(3));
    assert!(!city.launch_rocket(origin));
    city.tick().unwrap();
    assert_eq!(city.stock(origin, Commodity::Waste), Some(3333));
    assert_eq!(city.pop_event(), Some(CityEvent::RocketReady(origin)));
    assert_eq!(city.building_state(origin).unwrap().rocket_frame, Some(4));
    city.animate(1000).unwrap();
    assert!(!city.launch_rocket(Point { x: 4, y: 3 }));
    assert!(city.launch_rocket(origin));
    assert!(!city.launch_rocket(origin));
    city.animate(2000).unwrap();
    assert_eq!(city.building_state(origin).unwrap().rocket_frame, Some(5));
    city.animate(3000).unwrap();
    assert_eq!(city.building_state(origin).unwrap().rocket_frame, Some(6));
    assert_eq!(city.pop_event(), None);
    city.tick().unwrap();
    assert_eq!(city.pop_event(), Some(CityEvent::RocketSuccess(origin)));
    city.tick().unwrap();
    assert_eq!(city.balance(), -400);
    assert_eq!(city.building_state(origin).unwrap().rocket_frame, Some(7));
    city.finish_decisions().unwrap();
}
