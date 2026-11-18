//! Terrain, climate and starting settlement parameters from LinCity's map generator.
pub(crate) const MIN_TERRAIN_SIDE: i32 = 13;
pub(crate) const MIN_VILLAGE_SIDE: i32 = 26;
pub(crate) const VILLAGE_LOCATION_TRIES: i32 = 500;
pub(crate) const BASE_RELIEF: i32 = 100;
pub(crate) const RELIEF_VARIATION: i32 = 300;
pub(crate) const RIVER_ARIDITY_VARIATION: i32 = 450;
pub(crate) const RIVER_ARIDITY_OFFSET: i32 = 150;
pub(crate) const CLIMATE_ARIDITY_RANGE: i32 = 200;
pub(crate) const STARTER_HOME_POPULATION: i32 = 50;
pub(crate) const DIAGONAL_DISTANCE_INCREMENT: f32 = 0.414_213_57;
pub(crate) const COAL_VEIN_AREA: i32 = 400;

pub(crate) const SMOOTHING_RADIUS: usize = 7;
pub(crate) const GAUSSIAN_SIGMA: f32 = 3.5;
pub(crate) const ROUGHNESS: f32 = 0.7;
pub(crate) const HEIGHT_STEP: i32 = 30;

pub(crate) struct Zone {
    pub(crate) aridity: i32,
    pub(crate) outcomes: i32,
    pub(crate) water: i32,
    pub(crate) desert: i32,
    pub(crate) grass: i32,
    pub(crate) tree: i32,
    pub(crate) tree2: i32,
}

// Inclusive selection thresholds for a native ecology draw. Negative thresholds
// exclude a terrain from that zone.
pub(crate) const ZONES: [Zone; 9] = [
    Zone {
        aridity: 300,
        outcomes: 10,
        water: -1,
        desert: 6,
        grass: 8,
        tree: 9,
        tree2: 9,
    },
    Zone {
        aridity: 160,
        outcomes: 10,
        water: -1,
        desert: 2,
        grass: 6,
        tree: 9,
        tree2: 9,
    },
    Zone {
        aridity: 80,
        outcomes: 10,
        water: -1,
        desert: 1,
        grass: 4,
        tree: 6,
        tree2: 9,
    },
    Zone {
        aridity: 40,
        outcomes: 40,
        water: -1,
        desert: 0,
        grass: 12,
        tree: 24,
        tree2: 36,
    },
    Zone {
        aridity: 0,
        outcomes: 40,
        water: -1,
        desert: -1,
        grass: 10,
        tree: 20,
        tree2: 30,
    },
    Zone {
        aridity: -40,
        outcomes: 40,
        water: -1,
        desert: -1,
        grass: 5,
        tree: 10,
        tree2: 25,
    },
    Zone {
        aridity: -80,
        outcomes: 40,
        water: 0,
        desert: -1,
        grass: 6,
        tree: 15,
        tree2: 28,
    },
    Zone {
        aridity: -120,
        outcomes: 40,
        water: 1,
        desert: -1,
        grass: 6,
        tree: 16,
        tree2: 30,
    },
    Zone {
        aridity: i32::MIN,
        outcomes: 40,
        water: 3,
        desert: -1,
        grass: 8,
        tree: 20,
        tree2: 35,
    },
];

/// Elevation bands used to scale groundwater and slope effects.
pub(crate) const ALTITUDE_BANDS: i32 = 10;
pub(crate) const SWAMP_RELIEF_DIVISOR: i32 = 5;
pub(crate) const SEA_LEVEL_CORNER_DIVISOR: f64 = 5.3;
pub(crate) const MAX_SEA_LEVEL_HEIGHT_DIVISOR: i32 = 3;
pub(crate) const SMOOTHING_PASSES: usize = 2;

pub(crate) mod coal_veins {
    pub(crate) const START_MARGIN: i32 = 6;
    pub(crate) const X_EXCLUDED_TILES: i32 = 12;
    pub(crate) const Y_EXCLUDED_TILES: i32 = 10;
    pub(crate) const VEIN_LENGTH: i32 = 5;
}

pub(crate) mod river {
    /// The mouth starts in the middle third of the south edge.
    pub(crate) const MOUTH_POSITION_DIVISOR: i32 = 3;
    pub(crate) const MOUTH_BASE_LENGTH_DIVISOR: i32 = 18;
    pub(crate) const MOUTH_LENGTH_VARIATION_DIVISOR: i32 = 8;
    pub(crate) const MOUTH_STRAIGHT_END: i32 = 2;
    pub(crate) const SEGMENT_MIN_LENGTH_DIVISOR: i32 = 6;
    pub(crate) const SEGMENT_MAX_LENGTH_DIVISOR: i32 = 2;
    /// Discrete weights for a step inward, due north, or outward from the fork.
    pub(crate) const NORTHERN_STEP_WEIGHTS: [f64; 3] = [1., 2., 5.];
    pub(crate) const SOUTHERN_STEP_WEIGHTS: [f64; 3] = [1., 2., 7.];
    pub(crate) const WEST_MARGIN: i32 = 4;
    pub(crate) const EAST_MARGIN: i32 = 5;
    pub(crate) const JOIN_DELAY: i32 = 5;
    pub(crate) const JOIN_DISTANCE: i32 = 2;
    pub(crate) const WEST_BRANCH_CLEARANCE: i32 = 10;
    pub(crate) const NORTH_MARGIN: i32 = 10;
    pub(crate) const FORK_NORTH_MARGIN: i32 = 20;
    pub(crate) const FORK_WEST_MARGIN: i32 = 6;
    pub(crate) const FORK_EAST_MARGIN: i32 = 7;

    pub(crate) const SLOPE_DIVISOR: i32 = 10;
    pub(crate) const BANK_STEEPNESS: f32 = 8.;
    pub(crate) const SOUTH_SLOPE_REDUCTION: f32 = 6.5;
    pub(crate) const PLAIN_CURVATURE: f32 = 1.3;

    pub(crate) const BASE_LAKE_DENSITY: i32 = 5;
    pub(crate) const LAKE_DENSITY_TRIALS: i32 = 10;
    pub(crate) const LAKE_DENSITY_PROBABILITY: f64 = 0.5;
    pub(crate) const LAKE_MOISTURE_BASE: i32 = 100;
    pub(crate) const LAKE_ARIDITY_DIVISOR: i32 = 4;
    pub(crate) const LAKE_DENSITY_AREA: i64 = 1_000_000;
}

pub(crate) mod vegetation {
    pub(crate) const INITIAL_WATER_DISTANCE: i32 = 50;
    pub(crate) const GROUNDWATER_HEIGHT_NUMERATOR: i32 = 3;
    pub(crate) const GROUNDWATER_HEIGHT_DENOMINATOR: i32 = 4;
    pub(crate) const DISTANCE_SQUARED_DIVISOR: i32 = 5;
    pub(crate) const ALTITUDE_ARIDITY_FACTOR: i32 = 50;
    pub(crate) const NEAR_RIVER_DISTANCE_SQUARED: i32 = 5;
    pub(crate) const OUTER_RIVER_DISTANCE_SQUARED: i32 = 17;
    pub(crate) const OUTER_RIVER_ARIDITY_NUMERATOR: i32 = 2;
    pub(crate) const RIVER_ARIDITY_DENOMINATOR: i32 = 3;
}

/// Starter layout, in tile offsets from the northwest corner of the village.
pub(crate) mod village {
    use crate::BuildingKind::{self, Market, Pottery, ResidenceMediumLow, Waterwell};
    use std::ops::Range;
    pub(crate) const RIVER_SEARCH: Range<i32> = 2..23;
    pub(crate) const RIVER_FREE_INTERIOR: Range<i32> = 4..22;
    pub(crate) const DRAIN_COLUMNS: Range<i32> = 6..21;
    pub(crate) const DRAIN_ROWS: Range<i32> = 5..21;
    pub(crate) const FARM_COLUMNS: [i32; 2] = [6, 17];
    pub(crate) const FARM_ROW: i32 = 5;
    pub(crate) const FARM_WATER_THRESHOLD: i32 = i32::MAX / 2;
    pub(crate) const BUILDINGS: [(BuildingKind, i32, i32); 5] = [
        (ResidenceMediumLow, 10, 6),
        (Pottery, 9, 9),
        (Waterwell, 16, 9),
        (ResidenceMediumLow, 14, 6),
        (Market, 14, 9),
    ];
    pub(crate) const CROSS_TRACK: (i32, Range<i32>) = (11, 2..23);
    pub(crate) const NORTH_TRACK: (i32, Range<i32>) = (13, 2..11);
    pub(crate) const SOUTH_TRACK: (i32, Range<i32>) = (15, 12..23);
    pub(crate) const COMMUNE_COLUMNS: [i32; 3] = [6, 11, 16];
    pub(crate) const COMMUNE_ROWS: [i32; 2] = [12, 17];
}
