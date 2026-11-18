//! Building production and ordered exchange of a single commodity.

use crate::rules::{DAYS_PER_MONTH, MAX_TECH_LEVEL};
const SOLAR_BASE_OUTPUT: i32 = 900;
const SOLAR_LABOR: i32 = 50;
const STATION_INTAKE: i32 = 1500;
const STATION_LOW_CAPACITY: i32 = 20 * 2 * STATION_INTAKE;
const TRANSPORT_QUANTA: i64 = crate::rules::transport::QUANTA as i64;
const TRANSPORT_RATE: i64 = crate::rules::transport::RATE as i64;

/// A building's stock and trading rules for one commodity.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TradeStock {
    pub amount: i32,
    capacity: i32,
    take: bool,
    give: bool,
    pub evacuating: bool,
}

impl TradeStock {
    pub fn new(capacity: i32, take: bool, give: bool) -> Self {
        assert!(capacity >= 0);
        Self {
            amount: 0,
            capacity,
            take,
            give,
            evacuating: false,
        }
    }
}

/// Redistribute stock in the supplied neighbor order, using one common target
/// ratio. Calling this from the other end performs a distinct trade visit.
pub fn trade(center: &mut TradeStock, neighbors: &mut [&mut TradeStock]) {
    if center.capacity == 0 || (center.evacuating && center.amount == 0) {
        return;
    }
    let center_capacity = if center.evacuating {
        0
    } else {
        i64::from(center.capacity)
    };
    let mut total_capacity = center_capacity;
    let mut total_amount = i64::from(center.amount);
    for peer in neighbors.iter().filter(|peer| peer.capacity > 0) {
        let peer_capacity = i64::from(peer.capacity);
        let peer_amount = i64::from(peer.amount);
        if peer.evacuating {
            total_amount += peer_amount;
            continue;
        }
        let peer_is_fuller =
            peer_amount * center_capacity > i64::from(center.amount) * peer_capacity;
        let participates = if peer_is_fuller {
            center.take && peer.give
        } else {
            center.give && peer.take
        };
        if participates {
            total_amount += peer_amount;
            total_capacity += peer_capacity;
        }
    }
    if total_capacity == 0 {
        return;
    }
    let ratio = total_amount * TRANSPORT_QUANTA / total_capacity;
    for peer in neighbors.iter_mut().filter(|peer| peer.capacity > 0) {
        exchange(center, peer, ratio);
    }
}

pub(crate) fn exchange(center: &mut TradeStock, peer: &mut TradeStock, ratio: i64) {
    let transfer_limit = i64::from(center.capacity) / TRANSPORT_RATE;
    let center_room = (i64::from(center.capacity) - i64::from(center.amount)).max(0);
    let peer_amount = i64::from(peer.amount);
    let flow = if peer.evacuating {
        -center_room.min(peer_amount)
    } else {
        let target = ratio * i64::from(peer.capacity) / TRANSPORT_QUANTA;
        let delta = target - peer_amount;
        if delta > 0 && peer.take && center.give {
            let peer_room = (i64::from(peer.capacity) - peer_amount).max(0);
            delta
                .min(transfer_limit)
                .min(i64::from(center.amount))
                .min(peer_room)
        } else if delta < 0 && peer.give && center.take {
            -(-delta).min(transfer_limit).min(center_room)
        } else {
            0
        }
    };
    peer.amount += flow as i32;
    center.amount -= flow as i32;
}

#[derive(Debug, Clone)]
pub struct SolarPlant {
    pub labor: i32,
    pub high_voltage: TradeStock,
    output: i32,
    working_output: i32,
    pub busy: i32,
}

impl SolarPlant {
    /// Technology is captured when the plant is placed.
    pub fn new(tech_level: i32) -> Self {
        let output = (f64::from(SOLAR_BASE_OUTPUT)
            + f64::from(tech_level) * f64::from(SOLAR_BASE_OUTPUT) / f64::from(MAX_TECH_LEVEL))
            as i32;
        assert!(output > 0);
        Self {
            labor: 0,
            high_voltage: TradeStock::new(20 * SOLAR_BASE_OUTPUT, false, true),
            output,
            working_output: 0,
            busy: 0,
        }
    }

    pub fn update(&mut self, day: u32) {
        let made = self
            .output
            .min(self.high_voltage.capacity - self.high_voltage.amount);
        let labor_used = SOLAR_LABOR * made / self.output;
        if made >= SOLAR_BASE_OUTPUT && self.labor >= labor_used {
            self.labor -= labor_used;
            self.high_voltage.amount += made;
            self.working_output += made;
        }
        if day % DAYS_PER_MONTH as u32 == DAYS_PER_MONTH as u32 - 1 {
            self.busy = self.working_output / self.output;
            self.working_output = 0;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Substation {
    pub high_voltage: TradeStock,
    pub low_voltage: i32,
    working_intake: i32,
    pub busy: i32,
}

impl Default for Substation {
    fn default() -> Self {
        Self {
            high_voltage: TradeStock::new(20 * STATION_INTAKE, true, false),
            low_voltage: 0,
            working_intake: 0,
            busy: 0,
        }
    }
}

impl Substation {
    pub fn update(&mut self, day: u32) {
        let intake = STATION_INTAKE.min(self.high_voltage.amount);
        if intake > 0 && self.low_voltage <= STATION_LOW_CAPACITY - 2 * intake {
            self.high_voltage.amount -= intake;
            self.low_voltage += 2 * intake;
            self.working_intake += intake;
        }
        if day % DAYS_PER_MONTH as u32 == DAYS_PER_MONTH as u32 - 1 {
            self.busy = self.working_intake / STATION_INTAKE;
            self.working_intake = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_chains_match_every_tick() {
        use std::fmt::Write;
        let inputs = include_str!("../../lincity-core/parity/cases/v1/input.txt");
        let expected = include_str!("../../lincity-core/parity/cases/v1/expected.txt");
        for command in inputs.lines().filter(|line| line.starts_with("chain ")) {
            let values = command
                .split_whitespace()
                .skip(1)
                .map(|v| v.parse::<i32>().unwrap())
                .collect::<Vec<_>>();
            let [labor, solar_high, station_high, station_low, ticks, remove] = values[..] else {
                panic!("invalid chain")
            };
            let mut solar = SolarPlant::new(MAX_TECH_LEVEL);
            solar.labor = labor;
            solar.high_voltage.amount = solar_high;
            let mut station = Some(Substation::default());
            station.as_mut().unwrap().high_voltage.amount = station_high;
            station.as_mut().unwrap().low_voltage = station_low;
            let mut actual = String::new();
            for tick in 0..=ticks {
                if tick > 0 {
                    if tick == remove {
                        station = None;
                    }
                    if let Some(peer) = &mut station {
                        trade(&mut solar.high_voltage, &mut [&mut peer.high_voltage]);
                    }
                    solar.update(tick as u32);
                    if let Some(peer) = &mut station {
                        trade(&mut peer.high_voltage, &mut [&mut solar.high_voltage]);
                        peer.update(tick as u32);
                    }
                }
                write!(
                    actual,
                    "{tick} {} {} ",
                    solar.labor, solar.high_voltage.amount
                )
                .unwrap();
                if let Some(peer) = &station {
                    writeln!(actual, "{} {}", peer.high_voltage.amount, peer.low_voltage).unwrap();
                } else {
                    writeln!(actual, "removed").unwrap();
                }
            }
            let mut lines = expected.lines().skip_while(|line| *line != command).skip(1);
            for line in actual.lines() {
                assert_eq!(Some(line), lines.next(), "{command}");
            }
        }
    }

    #[test]
    fn production_uses_placement_technology_and_rolls_over_utilization() {
        let mut solar = SolarPlant::new(MAX_TECH_LEVEL / 2);
        solar.labor = 49;
        solar.update(98);
        assert_eq!(solar.high_voltage.amount, 0);
        solar.labor = 50;
        solar.update(99);
        assert_eq!(
            (solar.labor, solar.high_voltage.amount, solar.busy),
            (0, 1350, 1)
        );
        solar.update(199);
        assert_eq!(solar.busy, 0);
        let mut station = Substation::default();
        station.high_voltage.amount = 1500;
        station.low_voltage = 57001;
        station.update(98);
        assert_eq!(station.high_voltage.amount, 1500);
        station.low_voltage = 57000;
        station.update(99);
        assert_eq!((station.low_voltage, station.busy), (60000, 1));
    }

    #[test]
    fn evacuating_neighbor_fills_center_before_next_neighbor() {
        let mut center = TradeStock::new(80, true, false);
        let mut first = TradeStock::new(100, false, true);
        let mut second = first;
        first.amount = 65;
        second.amount = 70;
        first.evacuating = true;
        second.evacuating = true;
        trade(&mut center, &mut [&mut first, &mut second]);
        assert_eq!((center.amount, first.amount, second.amount), (80, 0, 55));
    }
}
