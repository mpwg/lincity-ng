use super::*;
use std::path::Path;
#[test]
fn loading_saving_and_replacing_a_city_keeps_the_instances_independent() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../lincity-core/tests/fixtures/blacksmith-2140.xml");
    let mut first = load(fixture.to_str().unwrap()).unwrap();
    let point = ffi::Point { x: 3, y: 4 };
    set_focus(&mut first, point);
    tick(&mut first).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("city.gz");
    save(&first, path.to_str().unwrap()).unwrap();
    let second = load(path.to_str().unwrap()).unwrap();
    assert_eq!(side(&second), side(&first));
    assert_eq!(focus(&second), point);
    tick(&mut first).unwrap();
    assert_eq!(first.city.report().day, second.city.report().day + 1);
    assert!(load("this-city-does-not-exist").is_err());
    assert!(generate(1, ffi::Climate::Desert, false, false).is_err());
}

#[test]
fn typed_failures_keep_quotes_building_ids_and_points_available_to_the_ui() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../lincity-core/tests/fixtures/blacksmith-2140.xml");
    let mut engine = load(fixture.to_str().unwrap()).unwrap();
    let site = ffi::Point { x: 4, y: 4 };
    let rocket = core::BuildingKind::RocketPad;
    let forbidden = can_build(&engine, rocket.id());
    assert_eq!(forbidden.error, ffi::Failure::NotEnoughTech);
    assert_eq!(
        (forbidden.current_tech, forbidden.required_tech),
        (41, rocket.metadata().required_tech)
    );
    let river = ffi::Point { x: 6, y: 8 };
    let quote = quote(
        &engine,
        ffi::ActionKind::Build,
        core::BuildingKind::Track.id(),
        river,
    );
    assert_eq!(quote.group, core::BuildingKind::TrackBridge.id());
    assert!(quote.cost > 0);
    let invalid = apply(&mut engine, ffi::ActionKind::Build, u16::MAX, site);
    assert_eq!(invalid.error, ffi::Failure::InvalidAction);
    let illegal = preview(&engine, ffi::ActionKind::Flood, 0, river);
    assert_eq!(illegal.error, ffi::Failure::SpaceOccupied);
    assert_eq!(illegal.point, river);
    assert_eq!(illegal.cost, 1_000_000);
    assert_eq!(
        apply(&mut engine, ffi::ActionKind::Flood, 0, site).error,
        ffi::Failure::None
    );
    assert!(pop_event(&mut engine).kind == ffi::EventKind::OutOfMoney);
    assert!(pop_event(&mut engine).kind == ffi::EventKind::None);
    for mode in [
        ffi::Traffic::Disabled,
        ffi::Traffic::Reduced,
        ffi::Traffic::Normal,
    ] {
        set_traffic(&mut engine, mode).unwrap();
    }
    assert!(!port_trade(&mut engine, site, u8::MAX, true, true));
    assert!(!market_trade(&mut engine, site, u8::MAX, true, true));
    assert!(!launch(&mut engine, site));
}

#[test]
fn generated_city_supports_a_survey_history_and_animation() {
    let mut engine = generate(32, ffi::Climate::Temperate, true, true).unwrap();
    let initial = engine.city.balance();
    let purchase = survey_coal(&mut engine);
    assert_eq!(
        (purchase.error, purchase.cost),
        (ffi::Failure::None, 1_000_000)
    );
    assert_eq!(survey_coal(&mut engine).cost, 0);
    assert_eq!(engine.city.balance(), initial - 1_000_000);
    ensure_history(&mut engine, 150);
    tick(&mut engine).unwrap();
    animate(&mut engine, 20).unwrap();
    assert_eq!(side(&engine), 32);
}

#[test]
fn a_corrupt_save_is_rejected_and_an_unwritable_destination_survives() {
    let dir = tempfile::tempdir().unwrap();
    let corrupt = dir.path().join("corrupt.gz");
    std::fs::write(&corrupt, b"truncated city").unwrap();
    assert!(load(corrupt.to_str().unwrap()).is_err());
    let engine = generate(32, ffi::Climate::Desert, false, false).unwrap();
    assert!(save(&engine, dir.path().to_str().unwrap()).is_err());
    assert_eq!(std::fs::read(corrupt).unwrap(), b"truncated city");
}
