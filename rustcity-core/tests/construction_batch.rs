use rustcity_core::{Action, ActionError, BuildingKind, City, Point};

fn point(x: i32, y: i32) -> Point {
    Point { x, y }
}

#[test]
fn batch_spends_credit_in_input_order_and_skips_blocked_cells() {
    let mut city = City::load(
        br#"<lc-game ldsv-version="2140"><globals><total_money>-1999999998</total_money></globals>
        <map size="16"><Construction group="6" map-x="4" map-y="4"/></map></lc-game>"#
            .as_slice(),
    )
    .unwrap();
    let action = Action::Build(BuildingKind::Track);
    let targets = [point(5, 5), point(8, 8), point(9, 8), point(10, 8)];
    let preview = city.preview_batch(action, &targets);
    assert_eq!(
        preview[0].status,
        Err(ActionError::SpaceOccupied(targets[0]))
    );
    assert_eq!(preview[1].status, Ok(()));
    assert_eq!(preview[2].status, Ok(()));
    assert_eq!(preview[3].status, Err(ActionError::OutOfMoney));
    assert_eq!(city.balance(), -1_999_999_998);
    assert_eq!(city.building_at(targets[1]), None);
    assert!(city.pop_event().is_none());
    assert_eq!(city.apply_batch(action, &targets), preview);
    assert_eq!(city.balance(), -2_000_000_000);
    assert_eq!(
        city.building_at(targets[2]),
        Some((BuildingKind::Track, targets[2]))
    );
    assert_eq!(city.building_at(targets[3]), None);
}

#[test]
fn batch_accounts_for_new_and_removed_footprints() {
    let mut city = City::empty(16).unwrap();
    let origin = point(3, 3);
    let corner = point(4, 4);
    let next = point(5, 3);
    let targets = [origin, corner, next];
    let action = Action::Build(BuildingKind::Market);
    let preview = city.preview_batch(action, &targets);
    assert_eq!(preview[0].status, Ok(()));
    assert_eq!(preview[1].status, Err(ActionError::SpaceOccupied(corner)));
    assert_eq!(preview[2].status, Ok(()));
    assert_eq!(city.apply_batch(action, &targets), preview);

    let targets = [corner, origin, next];
    let preview = city.preview_batch(Action::Bulldoze, &targets);
    assert_eq!(
        (preview[0].anchor, preview[0].size, preview[0].cost),
        (origin, 2, 100)
    );
    assert_eq!((preview[1].size, preview[1].cost), (1, 0));
    assert_eq!(preview[2].cost, 100);
    assert_eq!(
        city.building_at(corner),
        Some((BuildingKind::Market, origin))
    );
    assert_eq!(city.apply_batch(Action::Bulldoze, &targets), preview);
    assert_eq!(city.balance(), -400);
    assert_eq!(city.building_at(corner), None);
}

#[test]
fn batch_preview_leaves_constructor_draws_for_the_actual_placement() {
    use rustcity_core::decisions::Decisions;
    use std::io::Cursor;

    let tape = b"LINCITY-SIMULATION-TRACE 1\nC 13\nC 52\nP 1 4 4 5\nZ\n";
    let decisions = Decisions::replay(Cursor::new(tape)).unwrap();
    let mut city = City::empty_with_decisions(16, decisions).unwrap();
    let action = Action::Build(BuildingKind::OrganicFarm);
    let sites = [point(4, 4), point(5, 5)];
    let preview = city.preview_batch(action, &sites);
    assert_eq!(preview[0].status, Ok(()));
    assert_eq!(preview[1].status, Err(ActionError::SpaceOccupied(sites[1])));
    assert_eq!(city.preview_batch(action, &sites), preview);
    assert_eq!(city.apply_batch(action, &sites), preview);
    city.finish_decisions().unwrap();
}

#[test]
fn batch_demolition_quotes_water_and_fire_revealed_by_previous_targets() {
    let xml = br#"<lc-game ldsv-version="2140"><globals/><map size="22">
        <MapTile group="0" map-x="6" map-y="6"><ore>500</ore></MapTile>
        <Construction group="15" map-x="6" map-y="6"/>
        <Construction group="33" map-x="15" map-y="15"/>
        </map></lc-game>"#;
    let mut city = City::load(xml.as_slice()).unwrap();
    let sites = [point(6, 6), point(7, 7), point(15, 15), point(15, 15)];
    let preview = city.preview_batch(Action::Bulldoze, &sites);
    assert_eq!(preview[0].status, Ok(()));
    assert_eq!(preview[1].cost, 1_000_000);
    assert_eq!(preview[1].status, Ok(()));
    assert_eq!(preview[2].status, Ok(()));
    assert_eq!(preview[3].status, Err(ActionError::ProtectedFire(sites[3])));
    assert_eq!(city.apply_batch(Action::Bulldoze, &sites), preview);
}
