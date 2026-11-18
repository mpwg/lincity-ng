//! Deterministic rules for the first independently executable LinCity core slice.
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Commodity {
    Food,
    Labor,
    Coal,
    Goods,
    Ore,
    Steel,
    Waste,
    Lovolt,
    Hivolt,
    Water,
}

impl Commodity {
    pub const ALL: [Self; 10] = [
        Self::Food,
        Self::Labor,
        Self::Coal,
        Self::Goods,
        Self::Ore,
        Self::Steel,
        Self::Waste,
        Self::Lovolt,
        Self::Hivolt,
        Self::Water,
    ];

    pub fn standard_name(self) -> &'static str {
        [
            "food", "labor", "coal", "goods", "ore", "steel", "waste", "lovolt", "hivolt", "water",
        ][self as usize]
    }

    pub fn from_standard_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|commodity| commodity.standard_name() == name)
    }
}

#[derive(Default, Clone, Copy)]
pub struct Inventory {
    pub amount: i32,
    pub capacity: i32,
}

impl Inventory {
    pub fn permille(self) -> i32 {
        if self.capacity == 0 {
            return 0;
        }
        (i64::from(self.amount) * 1000 / i64::from(self.capacity)) as i32
    }
}

impl std::ops::AddAssign for Inventory {
    fn add_assign(&mut self, delta: Self) {
        self.amount += delta.amount;
        self.capacity += delta.capacity;
    }
}

impl std::ops::SubAssign for Inventory {
    fn sub_assign(&mut self, delta: Self) {
        self.amount -= delta.amount;
        self.capacity -= delta.capacity;
    }
}

#[derive(Default)]
pub struct Counter {
    pub current: i32,
    pub previous: i32,
}

impl Counter {
    fn finalize(&mut self) -> i32 {
        self.previous = std::mem::take(&mut self.current);
        self.previous
    }
}

/// Fields covered by the monthly and annual parity cases.
pub struct Stats {
    pub population: Counter,
    pub housed: Counter,
    pub births: Counter,
    pub food: Inventory,
    pub population_history: VecDeque<i32>,
    pub pool_history: VecDeque<i32>,
    pub food_history: VecDeque<Inventory>,
    pub goods_tax: Counter,
    pub transport: Counter,
    pub unemployment_history: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            population: Counter::default(),
            housed: Counter::default(),
            births: Counter::default(),
            food: Inventory::default(),
            population_history: vec![0; 190].into(),
            pool_history: vec![0; 190].into(),
            food_history: vec![Inventory::default(); 2].into(),
            goods_tax: Counter::default(),
            transport: Counter::default(),
            unemployment_history: 0.0,
        }
    }
}

fn record<T>(history: &mut VecDeque<T>, value: T) {
    history.pop_back();
    history.push_front(value);
}

impl Stats {
    pub fn daily(&mut self) {}

    pub fn monthly(&mut self) {
        let population = self.population.finalize();
        record(&mut self.population_history, population);
        record(&mut self.pool_history, population - self.housed.finalize());
        record(&mut self.food_history, std::mem::take(&mut self.food));
        self.births.finalize();
    }

    pub fn yearly(&mut self) {
        self.unemployment_history = (f64::from(self.unemployment_history) * 0.99) as f32;
        self.goods_tax.finalize();
        self.transport.finalize();
    }
}

const SOLAR_CAPACITY: i32 = 18_000;
const STATION_CAPACITY: i32 = 30_000;

#[derive(Default)]
pub struct Substation {
    pub high: i32,
    pub low: i32,
}

impl Substation {
    fn update(&mut self) {
        let used = self.high.min(1500);
        if used > 0 && self.low <= 60_000 - 2 * used {
            self.high -= used;
            self.low += 2 * used;
        }
    }
}

/// Two adjacent buildings at maximum technology, visited solar first.
pub struct PowerChain {
    pub labor: i32,
    pub high: i32,
    pub station: Option<Substation>,
}

impl PowerChain {
    pub fn remove_station(&mut self) {
        self.station = None;
    }

    pub fn tick(&mut self) {
        // Each center executes trade before its own update. trade() computes
        // the neighbor's target twice using integer division, once per visit.
        if let Some(station) = &mut self.station {
            let ratio = if i64::from(station.high) * i64::from(SOLAR_CAPACITY)
                > i64::from(self.high) * i64::from(STATION_CAPACITY)
            {
                self.high * 1000 / SOLAR_CAPACITY
            } else {
                trade_ratio(self.high, station.high)
            };
            let sent = (ratio * STATION_CAPACITY / 1000 - station.high)
                .clamp(0, self.high.min(SOLAR_CAPACITY / 8));
            self.high -= sent;
            station.high += sent;
        }

        let produced = (SOLAR_CAPACITY - self.high).min(1800);
        let used = 50 * produced / 1800;
        if self.labor >= used && produced >= 900 {
            self.labor -= used;
            self.high += produced;
        }

        if let Some(station) = &mut self.station {
            let ratio = if i64::from(self.high) * i64::from(STATION_CAPACITY)
                > i64::from(station.high) * i64::from(SOLAR_CAPACITY)
            {
                trade_ratio(self.high, station.high)
            } else {
                station.high * 1000 / STATION_CAPACITY
            };
            let received = (self.high - ratio * SOLAR_CAPACITY / 1000).clamp(
                0,
                (STATION_CAPACITY - station.high).min(STATION_CAPACITY / 8),
            );
            self.high -= received;
            station.high += received;
            station.update();
        }
    }
}

fn trade_ratio(solar: i32, station: i32) -> i32 {
    (solar + station) * 1000 / (SOLAR_CAPACITY + STATION_CAPACITY)
}
