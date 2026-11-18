//! Fiscal policy and the foreign trade prices used by the reference simulation.
use crate::Commodity;

pub(crate) const TRANSPORT_RATE: i32 = 14;
pub(crate) const IMPORT_RATE: i32 = 1;
pub(crate) const INCOME_RATE: i32 = 8;
pub(crate) const COAL_RATE: i32 = 15;
pub(crate) const GOODS_RATE: i32 = 1;
pub(crate) const DOLE_RATE: i32 = 15;
pub(crate) const COAL_SURVEY_COST: i32 = 1_000_000;
pub(crate) const DEATH_EXPENSE: i32 = 500;
pub(crate) const COAL_TAX_DIVISOR: i32 = 10;
pub(crate) const GOODS_TECH_SCALE: f32 = 2.0 * super::MAX_TECH_LEVEL as f32;
pub(crate) const EXPORT_BRACKETS: [i32; 6] = [25_000, 50_000, 100_000, 200_000, 400_000, 800_000];
pub(crate) const EXPORT_DISCOUNT_DIVISOR: i32 = 10;
pub(crate) const INTEREST_PRINCIPAL_UNIT: i32 = 1000;
pub(crate) const INTEREST_PER_UNIT: i32 = 15;
pub(crate) const INTEREST_CAP: i32 = 1_000_000;
pub(crate) const CLOSE_TRADE_ODDS: i32 = 150;
pub(crate) const OPEN_TRADE_ODDS: i32 = 100;
pub(crate) const PORT_LABOR: i32 = 100;
pub(crate) const SHIPMENT_DIVISOR: i32 = 2;
pub(crate) const MIN_SHIPMENT_DIVISOR: i32 = 15;
pub(crate) const PENCE_PER_UNIT: i32 = 100;
pub(crate) const PORT_PRICES: [(Commodity, i32); 5] = [
    (Commodity::Food, 1),
    (Commodity::Coal, 50),
    (Commodity::Goods, 3),
    (Commodity::Ore, 1),
    (Commodity::Steel, 100),
];
