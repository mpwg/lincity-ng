/// A resource traded and stored by buildings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Commodity {
    Food,
    Labor,
    Coal,
    Goods,
    Ore,
    Steel,
    Waste,
    LowVoltage,
    HighVoltage,
    Water,
}
impl Commodity {
    /// All resources in the order used by LinCity save files and reports.
    pub const ALL: [Self; 10] = [
        Self::Food,
        Self::Labor,
        Self::Coal,
        Self::Goods,
        Self::Ore,
        Self::Steel,
        Self::Waste,
        Self::LowVoltage,
        Self::HighVoltage,
        Self::Water,
    ];

    /// Returns the persistent, locale-independent name.
    pub fn save_name(self) -> &'static str {
        match self {
            Self::Food => "food",
            Self::Labor => "labor",
            Self::Coal => "coal",
            Self::Goods => "goods",
            Self::Ore => "ore",
            Self::Steel => "steel",
            Self::Waste => "waste",
            Self::LowVoltage => "lovolt",
            Self::HighVoltage => "hivolt",
            Self::Water => "water",
        }
    }

    /// Looks up a persistent name from a save file.
    pub fn from_save_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|commodity| commodity.save_name() == name)
    }
}

/// Directions enabled for one commodity in a market, a port or foreign trade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TradePolicy {
    pub buy: bool,
    pub sell: bool,
}
