use super::*;
use crate::decisions::Decisions;
use std::io::Cursor;

#[test]
fn supplied_house_matches_cpp_civic_reference() {
    let mut city = City::empty(32).unwrap();
    let site = Point { x: 12, y: 12 };
    let mut b = Building::new(BuildingKind::ResidenceMediumLow);
    let Operation::Civic(Civic::Residence(home)) = &mut b.operation else {
        panic!("missing residence");
    };
    home.occupants = 24;
    for (c, n) in [(Food, 24), (Water, 24), (LowVoltage, 114), (Goods, 12)] {
        b.stock[c as usize] = n;
    }
    let id = city.place_building(site, 2, b).unwrap();
    city.people_pool = 0;
    city.tiles[(12 * 32 + 12) as usize].coverage = FIRE | HEALTH | CRICKET;
    city.decisions = Decisions::replay(Cursor::new(
        b"LINCITY-SIMULATION-TRACE 1\nC 1\nC 1\nC 500\nZ\n",
    ))
    .unwrap();
    assert!(city.update_civic(id, site).unwrap());
    assert_eq!(city.stock(site, Labor), Some(17));
    assert_eq!(city.stock(site, Waste), Some(4));
    assert_eq!(city.stats.population.population_m.acc, 24);
    assert_eq!(city.stats.population.housing_m.acc, 100);
    assert_eq!(city.stock(site, LowVoltage), Some(0));
    let (_, b) = city.buildings.get(id).unwrap();
    let Operation::Civic(Civic::Residence(h)) = &b.operation else {
        unreachable!()
    };
    assert_eq!(
        (h.occupants, h.desirability, h.flags),
        (24, 105, 0x0100_0060)
    );
    city.decisions.finish().unwrap();
}

#[test]
fn school_costs_and_university_debits_follow_separate_ledgers() {
    let mut city = City::empty(32).unwrap();
    for (kind, point, labor, goods) in [
        (BuildingKind::School, Point { x: 4, y: 4 }, 200, 75),
        (BuildingKind::University, Point { x: 14, y: 14 }, 250, 750),
    ] {
        let mut b = Building::new(kind);
        b.stock[Labor as usize] = labor;
        b.stock[Goods as usize] = goods;
        let id = city
            .place_building(point, i32::from(kind.metadata().size), b)
            .unwrap();
        city.update_civic(id, point).unwrap();
        assert_eq!(city.stock(point, Waste), Some(goods / 3));
    }
    assert_eq!(city.balance, -23);
    assert_eq!(city.stats.expenses.school.acc, 2);
    assert_eq!(city.stats.expenses.university.acc, 23);
    assert_eq!(city.tech_level, 6);
}

#[test]
fn shanty_steals_goods_from_taxable_trade_and_burns_waste() {
    let p = Point { x: 4, y: 4 };
    let mut city = City::empty(14).unwrap();
    let mut b = Building::new(BuildingKind::Shanty);
    b.stock[Waste as usize] = 350;
    b.stock[Goods as usize] = 50;
    b.stock[Labor as usize] = 5;
    city.economy.taxable_goods = 150;
    city.economy.taxable_labor = 1;
    let id = city.place_building(p, 2, b).unwrap();
    city.decisions = Decisions::replay(Cursor::new(
        b"LINCITY-SIMULATION-TRACE 1\nS shanty.waste-fire 0\nZ\n",
    ))
    .unwrap();
    city.update_civic(id, p).unwrap();
    assert_eq!(city.stock(p, Waste), Some(0));
    assert_eq!(city.tiles[5 * 14 + 5].pollution, 368);
    assert_eq!(
        (city.economy.taxable_goods, city.economy.taxable_labor),
        (50, 0)
    );
    city.finish_decisions().unwrap();
}

#[test]
fn fire_replays_random_animation_and_covered_burn_rate() {
    let mut city = City::empty(16).unwrap();
    let p = Point { x: 3, y: 5 };
    let id = city
        .place_building(p, 1, Building::new(BuildingKind::Fire))
        .unwrap();
    city.tiles[(5 * 16 + 3) as usize].coverage = FIRE;
    city.decisions = Decisions::replay(Cursor::new(
        b"LINCITY-SIMULATION-TRACE 1\nA 1000 1 1\nC 175\nC 7\nZ\n",
    ))
    .unwrap();
    city.animate(1000).unwrap();
    city.update_civic(id, p).unwrap();
    let (_, b) = city.buildings.get(id).unwrap();
    let Operation::Civic(Civic::Fire(f)) = &b.operation else {
        panic!()
    };
    assert_eq!(f.burning, 12);
    assert_eq!(city.tiles[(5 * 16 + 3) as usize].pollution, 1);
    city.finish_decisions().unwrap();
}
