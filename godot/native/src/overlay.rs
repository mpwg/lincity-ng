// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) 2005 David Kamphausen <david.kamphausen@web.de>
// Copyright (C) 2024-2026 David Bears <dbear4q@gmail.com>

use rustcity_core::{
    BuildingKind, BuildingState, COAL_DEPOSIT_CAPACITY, City, Commodity, TRANSPORT_QUANTA,
    TRANSPORT_RATE, Terrain, TileView,
};

const MAX_POLLUTION: f64 = 600.0;
const LOW_STOCK: i32 = TRANSPORT_QUANTA * 5 / 100;
const LIMITED_STOCK: i32 = TRANSPORT_QUANTA * 10 / 100;
const HIGH_LABOR: i32 = TRANSPORT_QUANTA - LIMITED_STOCK;
const VERY_HIGH_LABOR: i32 = TRANSPORT_QUANTA - LOW_STOCK;
const PEAK_TRAFFIC: f64 = 0.928;
const STOCK_CONTRAST: f64 = 2.25;

const BLACK: u32 = 0x000000;
const RED: u32 = 0xff0000;
const DARK_RED: u32 = 0x7f0000;
const GREEN: u32 = 0x00ff00;
const DARK_GREEN: u32 = 0x007f00;
const YELLOW: u32 = 0xffff00;
const ORANGE: u32 = 0xff9900;

#[derive(Clone, Copy)]
enum Mode {
    Normal,
    Pollution,
    Unemployment,
    Starvation,
    Power,
    Fire,
    Sports,
    Health,
    Coal,
    Traffic,
    Commodities,
}

impl Mode {
    // Display mode numbers are shared with the Godot overlay selector.
    const ALL: [Self; 11] = [
        Self::Normal,
        Self::Pollution,
        Self::Unemployment,
        Self::Starvation,
        Self::Power,
        Self::Fire,
        Self::Sports,
        Self::Health,
        Self::Coal,
        Self::Traffic,
        Self::Commodities,
    ];
}

pub(crate) fn overlay(city: &City, mode: i32, commodity: i32) -> Vec<u32> {
    let Some(&mode) = Mode::ALL.get(mode as usize) else {
        return Vec::new();
    };
    let Some(&commodity) = Commodity::ALL.get(commodity as usize) else {
        return Vec::new();
    };

    let side = city.side();
    let mut colours = Vec::with_capacity(side as usize * side as usize);
    colours.extend(
        city.map_region(0..side, 0..side)
            .map(|tile| colour(city, tile, mode, commodity)),
    );
    colours
}

fn colour(city: &City, tile: TileView, mode: Mode, commodity: Commodity) -> u32 {
    use BuildingKind::*;

    let normal = normal_colour(tile);
    let [r, g, b] = normal;
    let brightness = ((u16::from(r) + u16::from(g) + u16::from(b)) / 3) as u8;
    let grey = rgb([brightness; 3]);
    let kind = tile.building.map(|(kind, _)| kind);
    let residence = matches!(
        kind,
        Some(
            ResidenceLowLow
                | ResidenceMediumLow
                | ResidenceHighLow
                | ResidenceLowHigh
                | ResidenceMediumHigh
                | ResidenceHighHigh
        )
    );
    let building = || city.building_state(tile.point);
    let origin_tile = || {
        tile.building
            .filter(|&(_, origin)| origin != tile.point)
            .and_then(|(_, origin)| city.tile(origin))
            .unwrap_or(tile)
    };

    match mode {
        Mode::Normal => rgb(normal),
        Mode::Pollution => {
            // The original overlay reads the pollution counter through a signed short.
            let ratio = f64::from(tile.pollution as i16) / MAX_POLLUTION;
            let ratio = ratio.clamp(0.0, 1.0) as f32;
            shade([channel(ratio), channel(1.0 - ratio), 0], brightness)
        }
        Mode::Unemployment => {
            let labor = stock_level(building().as_ref(), Commodity::Labor);
            match (residence, labor) {
                (_, -1) => grey,
                (true, level) if level > VERY_HIGH_LABOR => RED,
                (true, level) if level > HIGH_LABOR => DARK_RED,
                (false, level) if level < LOW_STOCK => YELLOW,
                (false, level) if level < LIMITED_STOCK => ORANGE,
                _ => grey,
            }
        }
        Mode::Starvation if residence => {
            let building = building();
            let food = stock_level(building.as_ref(), Commodity::Food);
            let water = stock_level(building.as_ref(), Commodity::Water);
            match food.min(water) {
                level if level < LOW_STOCK => RED,
                level if level < LIMITED_STOCK => DARK_RED,
                _ => GREEN,
            }
        }
        Mode::Starvation => grey,
        Mode::Power if kind == Some(PowerLine) => YELLOW,
        Mode::Power => {
            let building = building();
            let low = stock_level(building.as_ref(), Commodity::LowVoltage);
            let high = stock_level(building.as_ref(), Commodity::HighVoltage);
            match (low, high) {
                (_, high) if high > LOW_STOCK => GREEN,
                (low, _) if low > LOW_STOCK => DARK_GREEN,
                (low, high) if low >= 0 || high >= 0 => RED,
                _ => grey,
            }
        }
        Mode::Fire if kind == Some(Fire) => {
            if building()
                .and_then(|building| building.fire_age)
                .is_some_and(|(_, smoking)| smoking != 0)
            {
                ORANGE
            } else {
                RED
            }
        }
        Mode::Fire if kind == Some(FireStation) => GREEN,
        Mode::Sports if kind == Some(Cricket) => GREEN,
        Mode::Health if kind == Some(Health) => GREEN,
        Mode::Fire if origin_tile().fire_cover => shade([0, 255, 0], brightness),
        Mode::Sports if origin_tile().sports_cover => shade([0, 255, 0], brightness),
        Mode::Health if origin_tile().health_cover => shade([0, 255, 0], brightness),
        Mode::Fire | Mode::Sports | Mode::Health => grey,
        Mode::Coal if !city.coal_survey_done() => BLACK,
        Mode::Coal if tile.coal == 0 => grey,
        Mode::Coal if tile.coal >= COAL_DEPOSIT_CAPACITY / 2 => GREEN,
        Mode::Coal => RED,
        Mode::Traffic if kind.is_some_and(|kind| kind.is_transport() || kind == PowerLine) => {
            let Some(traffic) = city
                .commodity_traffic(tile.point, commodity)
                .filter(|&n| n >= 0)
            else {
                return grey;
            };
            let ratio = traffic as f32 * TRANSPORT_RATE as f32 / TRANSPORT_QUANTA as f32;
            let ratio = (f64::from(ratio) / PEAK_TRAFFIC).min(1.0);
            let intensity = (2.0 - 2.0 / (1.0 + ratio)) as f32;
            rgb([channel(intensity), channel(1.0 - intensity), 0])
        }
        Mode::Traffic => {
            if Market.metadata().rule(commodity).max_load != 0 && origin_tile().market_cover {
                rgb([0, 0, brightness])
            } else {
                grey
            }
        }
        Mode::Commodities => {
            let building = building();
            let level = stock_level(building.as_ref(), commodity);
            match building {
                Some(building) if level >= 0 => stock_colour(&building, commodity, level),
                _ => grey,
            }
        }
    }
}

fn normal_colour(tile: TileView) -> [u8; 3] {
    let Some((kind, _)) = tile.building else {
        return match tile.terrain {
            Terrain::Grass | Terrain::Tree | Terrain::Tree2 | Terrain::Tree3 => [0, 96, 0],
            Terrain::Desert => [144, 144, 0],
            Terrain::Water => [0, 0, 248],
        };
    };
    let colour = kind.metadata().colour;
    let intensity = ((colour & 31) * 8) as u8;
    [32, 64, 128].map(|bit| if colour & bit != 0 { intensity } else { 0 })
}

fn stock_level(building: Option<&BuildingState<'_>>, commodity: Commodity) -> i32 {
    let Some(building) = building else {
        return -1;
    };
    let capacity = building.kind.metadata().rule(commodity).max_load;
    if capacity == 0 {
        return -1;
    }

    let stock = building.stock(commodity);
    // Evacuation reports raw stock and hides the building as soon as it has at most one unit.
    if building.evacuating {
        if stock > 1 { stock } else { -1 }
    } else {
        (i64::from(stock) * i64::from(TRANSPORT_QUANTA) / i64::from(capacity)) as i32
    }
}

fn stock_colour(building: &BuildingState<'_>, commodity: Commodity, level: i32) -> u32 {
    let ratio = level as f32 / TRANSPORT_QUANTA as f32;
    let ratio = if commodity == Commodity::Waste {
        1.0 - ratio
    } else {
        ratio
    };
    let rule = building.kind.metadata().rule(commodity);
    let red = if rule.take {
        (1.0 - STOCK_CONTRAST * f64::from(ratio)).max(0.0) as f32
    } else {
        0.0
    };
    let blue = if rule.give {
        (1.0 - STOCK_CONTRAST * f64::from(1.0 - ratio)).max(0.0) as f32
    } else {
        0.0
    };
    let green = 1.0 - red - blue;
    rgb([channel(red), channel(green), channel(blue)])
}

fn channel(value: f32) -> u8 {
    // The original converts to int before narrowing, including for stock above capacity.
    (255.0 * value) as i32 as u8
}

fn shade(colour: [u8; 3], brightness: u8) -> u32 {
    rgb(colour.map(|channel| ((u16::from(channel) * u16::from(brightness)) >> 8) as u8))
}

fn rgb([red, green, blue]: [u8; 3]) -> u32 {
    u32::from_be_bytes([0, red, green, blue])
}

#[cfg(test)]
mod tests {
    use super::overlay;
    use rustcity_core::{Action, BuildingKind, City, Point};

    fn empty_residence() -> City {
        let mut city = City::empty(12).unwrap();
        city.apply(
            Action::Build(BuildingKind::ResidenceLowLow),
            Point { x: 2, y: 5 },
        )
        .unwrap();
        city
    }

    #[test]
    fn map_colours_use_the_covering_building_in_row_order() {
        let city = empty_residence();
        let colours = overlay(&city, 0, 0);
        assert_eq!(colours.len(), 144);
        assert_eq!(colours[5 * 12 + 2], 0x00c0c0);
        assert_eq!(colours[7 * 12 + 4], 0x00c0c0);
        assert_eq!(colours[2 * 12 + 5], 0x006000);
    }

    #[test]
    fn empty_residence_matches_legacy_stock_colours() {
        let city = empty_residence();
        for (mode, commodity, colour) in [
            (1, 0, 0x007f00),  // Pollution
            (2, 0, 0x808080),  // Unemployment
            (3, 0, 0xff0000),  // Starvation
            (4, 0, 0xff0000),  // Power
            (5, 0, 0x808080),  // Fire
            (6, 0, 0x808080),  // Sports
            (7, 0, 0x808080),  // Health
            (8, 0, 0x000000),  // Coal before survey
            (9, 0, 0x808080),  // Traffic
            (10, 0, 0xff0000), // Food
            (10, 1, 0x00ff00), // Labour
            (10, 6, 0x0000ff), // Waste
        ] {
            assert_eq!(
                overlay(&city, mode, commodity)[5 * 12 + 2],
                colour,
                "mode {mode}, commodity {commodity}"
            );
        }
    }

    #[test]
    fn legacy_overlays_use_coverage_at_origin_and_deposits_at_each_tile() {
        let save = br#"<?xml version="1.0"?>
<lc-game ldsv-version="2140">
  <globals><coal_survey_done>1</coal_survey_done></globals>
  <map size="32">
    <MapTile group="0" map-x="2" map-y="5"><flags>0x00550000</flags></MapTile>
    <MapTile group="21" map-x="3" map-y="5"><coal>5000</coal><air_pol>300</air_pol></MapTile>
    <MapTile group="42" map-x="4" map-y="5"><coal>4999</coal><air_pol>600</air_pol></MapTile>
    <Construction group="4" map-x="2" map-y="5">
      <flags>0x200</flags><labor>951</labor><food>1</food><lovolt>51</lovolt>
    </Construction>
    <Construction group="7" map-x="6" map-y="5"/>
    <Construction group="1" map-x="7" map-y="5"/>
    <Construction group="34" map-x="8" map-y="5"><smoking_days>1</smoking_days></Construction>
  </map>
</lc-game>"#;
        let city = City::load(save.as_slice()).unwrap();
        for (mode, commodity, x, colour) in [
            (1, 0, 3, 0x3f3f00),  // Pollution uses local counters.
            (2, 0, 3, 0xff0000),  // Evacuation reports raw stock.
            (4, 0, 3, 0x007f00),  // Raw low-voltage stock just exceeds the threshold.
            (4, 0, 7, 0xffff00),  // Power line.
            (5, 0, 3, 0x007f00),  // Fire coverage comes from the residence origin.
            (5, 0, 8, 0xff9900),  // Smoking fire.
            (8, 0, 3, 0x00ff00),  // Large coal deposit under the residence.
            (8, 0, 4, 0xff0000),  // Small deposit under the same residence.
            (9, 0, 3, 0x000080),  // Market coverage comes from the origin.
            (9, 0, 6, 0x00ff00),  // Unused route accepts food.
            (9, 0, 7, 0x8a8a8a),  // Power line does not accept food.
            (10, 0, 3, 0x808080), // Evacuation hides stock of one unit.
        ] {
            assert_eq!(
                overlay(&city, mode, commodity)[5 * 32 + x],
                colour,
                "mode {mode}, commodity {commodity}, column {x}"
            );
        }
    }

    #[test]
    fn rejects_invalid_bridge_arguments() {
        let city = City::empty(12).unwrap();
        for (mode, commodity) in [(-1, 0), (11, 0), (0, -1), (0, 10)] {
            assert!(overlay(&city, mode, commodity).is_empty());
        }
    }
}
