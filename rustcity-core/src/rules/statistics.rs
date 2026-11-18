//! Retained samples and yearly decay used in population and fiscal reports.
pub(crate) const POPULATION_SAMPLES: usize = 190;
pub(crate) const FISCAL_SAMPLES: usize = super::MONTHS_PER_YEAR as usize;
pub(crate) const INVENTORY_SAMPLES: usize = 2;
pub(crate) const POLLUTION_SAMPLES: usize = 2;
pub(crate) const YEARLY_HISTORY_RETENTION: f64 = 0.99;
