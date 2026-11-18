use rustcity_core::{Action, City, Point};
use std::error::Error;

#[test]
fn hosts_can_propagate_map_and_action_errors() {
    fn start(side: i32, p: Point) -> Result<(), Box<dyn Error>> {
        City::empty(side)?.apply(Action::Query, p)?;
        Ok(())
    }
    assert!(
        start(0, Point { x: 1, y: 1 })
            .unwrap_err()
            .is::<rustcity_core::MapSizeError>()
    );
    let error = start(8, Point { x: -1, y: 1 }).unwrap_err();
    assert!(error.is::<rustcity_core::ActionError>());
    assert!(error.to_string().contains("-1"));
}
