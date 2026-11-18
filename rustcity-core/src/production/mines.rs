use super::{Activity, Stock};
use crate::rules::{
    DAYS_PER_MONTH,
    industry::{ORE_RESERVE, coal_mine, ore_mine},
};
use crate::{
    Commodity::{Coal, Labor, Ore},
    Point,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mineral {
    Coal,
    Ore,
}

/// The mine reads and changes the city's actual map tiles, including deposits
/// shared with other mines. The tile array is row-major, including the border.
pub trait MiningTile {
    fn reserve(&self, mineral: Mineral) -> i32;
    fn change_reserve(&mut self, mineral: Mineral, delta: i32);
    fn pollute(&mut self, amount: i32);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MineAction {
    Continue,
    Evacuate,
    Abandon,
}

fn condition(reserve: i32, labor: i32, output: i32) -> MineAction {
    if reserve != 0 {
        MineAction::Continue
    } else if labor == 0 && output == 0 {
        MineAction::Abandon
    } else {
        MineAction::Evacuate
    }
}

#[derive(Debug, Copy, Clone)]
struct Area {
    lo: Point,
    hi: Point,
    side: i32,
}
impl Area {
    fn points(self) -> impl Iterator<Item = Point> {
        (self.lo.y..self.hi.y)
            .flat_map(move |y| (self.lo.x..self.hi.x).map(move |x| Point { x, y }))
    }
    fn index(self, p: Point) -> usize {
        (p.y * self.side + p.x) as usize
    }
    fn cells(self) -> impl Iterator<Item = usize> {
        self.points().map(move |p| self.index(p))
    }
}

#[derive(Debug, Clone)]
pub struct CoalMine {
    area: Area,
    cursor: usize,
    pub current_reserve: i32,
    pub initial_reserve: i32,
    pub activity: Activity,
}
impl CoalMine {
    pub fn place(point: Point, side: i32, tiles: &mut [impl MiningTile]) -> Self {
        let area = Area {
            lo: Point {
                x: (point.x - coal_mine::RANGE).max(1),
                y: (point.y - coal_mine::RANGE).max(1),
            },
            hi: Point {
                x: (point.x + coal_mine::RANGE).min(side - 1),
                y: (point.y + coal_mine::RANGE).min(side - 1),
            },
            side,
        };
        let mut reserve = area
            .cells()
            .map(|i| tiles[i].reserve(Mineral::Coal))
            .sum::<i32>();
        if reserve < coal_mine::MINIMUM_DEPOSITS {
            tiles[area.index(point)]
                .change_reserve(Mineral::Coal, coal_mine::MINIMUM_DEPOSITS - reserve);
            reserve = coal_mine::MINIMUM_DEPOSITS;
        }
        Self {
            area,
            cursor: area.index(area.lo),
            current_reserve: reserve,
            initial_reserve: reserve,
            activity: Activity::default(),
        }
    }

    pub fn update(
        &mut self,
        day: u32,
        stock: &mut Stock<'_>,
        tiles: &mut [impl MiningTile],
        sustainable: &mut bool,
    ) -> MineAction {
        if stock[Labor] >= coal_mine::LABOR
            && (stock[Coal] <= coal_mine::TARGET - coal_mine::BATCH
                || stock[Coal] >= coal_mine::TARGET + coal_mine::BATCH)
        {
            let mining = stock[Coal] <= coal_mine::TARGET - coal_mine::BATCH;
            let eligible = |amount: i32| {
                if mining {
                    amount > 0
                } else {
                    amount < coal_mine::DEPOSIT_CAPACITY
                }
            };
            let index = if eligible(tiles[self.cursor].reserve(Mineral::Coal)) {
                Some(self.cursor)
            } else {
                self.area
                    .cells()
                    .find(|&i| eligible(tiles[i].reserve(Mineral::Coal)))
            };
            if let Some(i) = index {
                self.cursor = i;
                let extracted = if mining { 1 } else { -1 };
                tiles[i].change_reserve(Mineral::Coal, -extracted);
                self.current_reserve -= extracted;
                stock.change(Coal, extracted * coal_mine::BATCH);
                stock.change(Labor, -coal_mine::LABOR);
                if mining {
                    tiles[i].pollute(coal_mine::POLLUTION);
                    if self.current_reserve < self.initial_reserve {
                        *sustainable = false;
                    }
                }
                self.activity.working_days += 1;
            } else {
                self.cursor = self.area.index(self.area.lo);
                if mining {
                    self.current_reserve = 0;
                }
            }
        }
        self.activity.close_day(day, stock);
        if day % DAYS_PER_MONTH as u32 == (DAYS_PER_MONTH - 1) as u32 {
            if stock.counts.previous[Coal as usize] > 0 {
                *sustainable = false;
            }
            self.current_reserve = self
                .area
                .cells()
                .map(|i| tiles[i].reserve(Mineral::Coal))
                .sum();
        }
        condition(self.current_reserve, stock[Labor], stock[Coal])
    }

    pub fn resource(coal: i32) -> &'static str {
        match coal {
            15001.. => "CoalMineFull",
            10001.. => "CoalMineMed",
            1.. => "CoalMineLow",
            _ => "CoalMine",
        }
    }
}

#[derive(Debug, Clone)]
pub struct OreMine {
    area: Area,
    pub reserve: i32,
    pub activity: Activity,
    active: bool,
    next_animation: u64,
    animation_step: i32,
    pub frame: i32,
}

impl OreMine {
    pub fn place(point: Point, side: i32, tiles: &[impl MiningTile]) -> Self {
        let area = Area {
            lo: point,
            hi: Point {
                x: point.x + ore_mine::SIZE,
                y: point.y + ore_mine::SIZE,
            },
            side,
        };
        let reserve = area
            .cells()
            .map(|i| tiles[i].reserve(Mineral::Ore))
            .sum::<i32>()
            .max(1);
        Self {
            area,
            reserve,
            activity: Activity::default(),
            active: false,
            next_animation: 0,
            animation_step: 0,
            frame: 0,
        }
    }

    pub fn update(
        &mut self,
        day: u32,
        stock: &mut Stock<'_>,
        tiles: &mut [impl MiningTile],
        sustainable: &mut bool,
    ) -> MineAction {
        self.active = false;
        let mining = self.reserve != 0
            && stock[Ore]
                <= ore_mine::TARGET_PERCENT * (ore_mine::CAPACITY - ore_mine::BATCH) / 100;
        let returning =
            stock[Ore] > ore_mine::TARGET_PERCENT * ore_mine::CAPACITY / 100 + ore_mine::BATCH;
        if stock[Labor] >= ore_mine::LABOR && (mining || returning) {
            let i = self.area.cells().find(|&i| {
                let ore = tiles[i].reserve(Mineral::Ore);
                if mining {
                    ore > 0
                } else {
                    ore < ORE_RESERVE * 3 / 2
                }
            });
            if let Some(i) = i {
                let extracted = if mining { 1 } else { -1 };
                tiles[i].change_reserve(Mineral::Ore, -extracted);
                self.reserve -= extracted;
                stock.change(Ore, extracted * ore_mine::BATCH);
                stock.change(Labor, -ore_mine::LABOR);
                if mining && self.reserve < ore_mine::SUSTAINABLE_DEPOSITS {
                    *sustainable = false;
                }
                self.active = true;
                self.activity.working_days += 1;
            }
        }
        self.activity.close_day(day, stock);
        condition(self.reserve, stock[Labor], stock[Ore])
    }

    /// Tiles that must become surface and underground water after removal.
    pub fn lake_points<'a, T: MiningTile>(
        &self,
        tiles: &'a [T],
    ) -> impl Iterator<Item = Point> + 'a {
        let area = self.area;
        area.points()
            .filter(move |&p| tiles[area.index(p)].reserve(Mineral::Ore) < ORE_RESERVE / 2)
    }

    pub fn animate(&mut self, millis: u64) {
        if self.active && millis >= self.next_animation {
            self.next_animation =
                millis + ((14 - self.activity.busy / 11) * ore_mine::FRAME_SPEED - 50) as u64;
            self.frame = match self.animation_step {
                0..8 => self.animation_step,
                8..12 => 14 - self.animation_step,
                _ => 16 - self.animation_step,
            };
            self.animation_step = (self.animation_step + 1) % 16;
        }
    }
}
