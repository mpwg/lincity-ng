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
