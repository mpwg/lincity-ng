use super::*;
use crate::{Commodity::*, power::ProductionCounters};

#[test]
fn light_industry_combines_voltages_and_rolls_over_before_the_first_day() {
    let mut light = LightIndustry::new(0);
    let mut stock = [0; 10];
    let mut counts = ProductionCounters::default();
    let mut shift = Stock::new(&mut stock, &mut counts);
    shift.amount[Labor as usize] = 59;
    shift.amount[Ore as usize] = 250;
    shift.amount[Steel as usize] = 12;
    shift.amount[HighVoltage as usize] = 3500;
    shift.amount[LowVoltage as usize] = 11001;
    assert_eq!(light.update(99, &mut shift), 11);
    assert_eq!(
        (shift[Goods], shift[HighVoltage], shift[LowVoltage]),
        (1800, 1, 0)
    );
    light.update(100, &mut shift);
    assert_eq!(shift.counts.previous[Goods as usize], 1800);
}

#[test]
fn partial_fuel_does_not_power_steel_but_preserves_all_available_fuel() {
    let mut factory = HeavyIndustry::new(2_500_000);
    let mut stock = [0; 10];
    let mut counts = ProductionCounters::default();
    let mut shift = Stock::new(&mut stock, &mut counts);
    shift.amount[Labor as usize] = 46;
    shift.amount[Ore as usize] = 612;
    shift.amount[HighVoltage as usize] = 2000;
    shift.amount[LowVoltage as usize] = 1000;
    shift.amount[Coal as usize] = 21;
    assert_eq!(factory.update(99, &mut shift), 0);
    assert_eq!(
        (
            shift[Steel],
            shift[Labor],
            shift[Coal],
            shift[HighVoltage],
            shift[LowVoltage]
        ),
        (7, 18, 21, 2000, 1000)
    );
}

#[test]
fn unfunded_recycling_still_burns_waste_after_month_close() {
    let mut recycle = Recycling::new(2_000_000);
    let mut stock = [0; 10];
    let mut counts = ProductionCounters::default();
    let mut shift = Stock::new(&mut stock, &mut counts);
    shift.amount[Waste as usize] = 9001;
    recycle.update(99, &mut shift, false);
    assert_eq!(
        (
            shift[Waste],
            shift.counts.current[Waste as usize],
            shift.counts.previous[Waste as usize]
        ),
        (8951, -50, 0)
    );
}

#[test]
fn tip_degrades_reserves_before_refilling_the_inbox() {
    let mut tip = Landfill {
        buried: 250,
        degradation: 239999,
        ..Landfill::default()
    };
    let mut stock = [0; 10];
    let mut counts = ProductionCounters::default();
    let mut shift = Stock::new(&mut stock, &mut counts);
    assert!(!tip.update(1, &mut shift));
    assert_eq!((tip.buried, tip.degradation, shift[Waste]), (49, 249, 200));
}

#[derive(Clone, Default)]
struct Tile {
    ore: i32,
    coal: i32,
    pollution: i32,
}

impl MiningTile for Tile {
    fn reserve(&self, mineral: Mineral) -> i32 {
        match mineral {
            Mineral::Coal => self.coal,
            Mineral::Ore => self.ore,
        }
    }
    fn change_reserve(&mut self, mineral: Mineral, delta: i32) {
        match mineral {
            Mineral::Coal => self.coal += delta,
            Mineral::Ore => self.ore += delta,
        }
    }
    fn pollute(&mut self, amount: i32) {
        self.pollution += amount;
    }
}

#[test]
fn coal_deposits_respect_scan_bounds_and_release_sustainability_on_extraction() {
    let mut tiles = vec![Tile::default(); 32 * 32];
    let origin = crate::Point { x: 10, y: 10 };
    tiles[10 * 32 + 16].coal = 100;
    let mut mine = CoalMine::place(origin, 32, &mut tiles);
    assert_eq!(mine.initial_reserve, 20);
    let mut stock = [0; 10];
    let mut counts = ProductionCounters::default();
    let mut shift = Stock::new(&mut stock, &mut counts);
    shift.amount[Labor as usize] = 918;
    shift.amount[Coal as usize] = 15000;
    let mut sustainable = true;
    assert_eq!(
        mine.update(99, &mut shift, &mut tiles, &mut sustainable),
        MineAction::Continue
    );
    assert_eq!(
        (
            mine.current_reserve,
            shift[Coal],
            tiles[10 * 32 + 10].pollution,
            tiles[10 * 32 + 16].coal,
            sustainable
        ),
        (19, 16000, 3, 100, false)
    );
}

#[test]
fn exhausted_ore_mine_waits_for_the_last_load_and_marks_shallow_tiles() {
    let mut tiles = vec![Tile::default(); 16 * 16];
    let origin = crate::Point { x: 4, y: 4 };
    tiles[4 * 16 + 4].ore = 1;
    let mut mine = OreMine::place(origin, 16, &tiles);
    let mut stock = [0; 10];
    let mut counts = ProductionCounters::default();
    let mut shift = Stock::new(&mut stock, &mut counts);
    shift.amount[Labor as usize] = 209;
    let mut sustainable = true;
    assert_eq!(
        mine.update(1, &mut shift, &mut tiles, &mut sustainable),
        MineAction::Evacuate
    );
    assert_eq!(shift[Ore], 5000);
    shift.amount[Ore as usize] = 0;
    assert_eq!(
        mine.update(2, &mut shift, &mut tiles, &mut sustainable),
        MineAction::Abandon
    );
    tiles[4 * 16 + 4].ore = 500;
    assert_eq!(mine.lake_points(&tiles).count(), 15);
}
