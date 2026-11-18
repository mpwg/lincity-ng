//! Building types registered by the simulation and their construction rules.
use crate::Commodity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum BuildingKind {
    PowerLine = 1,
    SolarPower = 2,
    Substation = 3,
    ResidenceLowLow = 4,
    OrganicFarm = 5,
    Market = 6,
    Track = 7,
    CoalMine = 8,
    Rail = 9,
    CoalPower = 10,
    Road = 11,
    LightIndustry = 12,
    University = 13,
    Commune = 14,
    OreMine = 15,
    Tip = 16,
    Port = 17,
    HeavyIndustry = 18,
    Parkland = 19,
    Recycle = 20,
    Health = 22,
    RocketPad = 23,
    Windmill = 24,
    Monument = 25,
    School = 26,
    Blacksmith = 27,
    Mill = 28,
    Pottery = 29,
    FireStation = 30,
    Cricket = 31,
    Shanty = 33,
    Fire = 34,
    ResidenceMediumLow = 36,
    ResidenceHighLow = 37,
    ResidenceLowHigh = 38,
    ResidenceMediumHigh = 39,
    ResidenceHighHigh = 40,
    Waterwell = 41,
    TrackBridge = 46,
    RoadBridge = 47,
    RailBridge = 48,
    WindPower = 49,
    ParkPond = 50,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommodityRule {
    pub max_load: i32,
    pub take: bool,
    pub give: bool,
}

impl CommodityRule {
    const fn receives(max_load: i32) -> Self {
        Self {
            max_load,
            take: true,
            give: false,
        }
    }
    const fn supplies(max_load: i32) -> Self {
        Self {
            max_load,
            take: false,
            give: true,
        }
    }
    const fn exchanges(max_load: i32) -> Self {
        Self {
            max_load,
            take: true,
            give: true,
        }
    }
}

#[derive(Debug)]
pub struct BuildingMetadata {
    pub resource: &'static str,
    pub name: &'static str,
    pub plural: &'static str,
    pub size: u16,
    pub colour: i32,
    pub base_cost: i32,
    pub cost_multiplier: i32,
    pub demolition_cost: i32,
    pub fire_chance: i32,
    pub required_tech: i32,
    pub range: i32,
    pub no_credit: bool,
    rules: &'static [(Commodity, CommodityRule)],
}

impl BuildingMetadata {
    pub fn rule(&self, commodity: Commodity) -> CommodityRule {
        self.rules
            .iter()
            .find(|(kind, _)| *kind == commodity)
            .map(|(_, rule)| *rule)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildDenial {
    NotEnoughStudents,
    NotEnoughTech,
    OutOfMoney,
}

impl BuildingKind {
    /// All registered construction groups in numeric save-file order.
    pub const ALL: [Self; 43] = [
        Self::PowerLine,
        Self::SolarPower,
        Self::Substation,
        Self::ResidenceLowLow,
        Self::OrganicFarm,
        Self::Market,
        Self::Track,
        Self::CoalMine,
        Self::Rail,
        Self::CoalPower,
        Self::Road,
        Self::LightIndustry,
        Self::University,
        Self::Commune,
        Self::OreMine,
        Self::Tip,
        Self::Port,
        Self::HeavyIndustry,
        Self::Parkland,
        Self::Recycle,
        Self::Health,
        Self::RocketPad,
        Self::Windmill,
        Self::Monument,
        Self::School,
        Self::Blacksmith,
        Self::Mill,
        Self::Pottery,
        Self::FireStation,
        Self::Cricket,
        Self::Shanty,
        Self::Fire,
        Self::ResidenceMediumLow,
        Self::ResidenceHighLow,
        Self::ResidenceLowHigh,
        Self::ResidenceMediumHigh,
        Self::ResidenceHighHigh,
        Self::Waterwell,
        Self::TrackBridge,
        Self::RoadBridge,
        Self::RailBridge,
        Self::WindPower,
        Self::ParkPond,
    ];

    pub const fn id(self) -> u16 {
        self as u16
    }

    pub(crate) fn is_transport(self) -> bool {
        matches!(
            self,
            Self::Road
                | Self::RoadBridge
                | Self::Track
                | Self::TrackBridge
                | Self::Rail
                | Self::RailBridge
        )
    }

    pub fn from_id(id: u16) -> Option<Self> {
        Self::ALL
            .binary_search_by_key(&id, |kind| kind.id())
            .ok()
            .map(|i| Self::ALL[i])
    }

    pub fn from_resource(resource: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.metadata().resource == resource)
    }

    pub fn cost(self, tech_level: i32) -> i32 {
        let m = self.metadata();
        let scaled = m.cost_multiplier * tech_level;
        (m.base_cost as f32 * (1.0 + scaled as f32 / 1_000_000.0)) as i32
    }

    /// Check city-wide requirements before inspecting the proposed site.
    pub fn can_build(
        self,
        tech_level: i32,
        money: i32,
        schools: u32,
        universities: u32,
    ) -> Result<(), BuildDenial> {
        if self == Self::University && schools / 4 <= universities {
            return Err(BuildDenial::NotEnoughStudents);
        }
        if tech_level < self.metadata().required_tech {
            return Err(BuildDenial::NotEnoughTech);
        }
        if self.metadata().no_credit && money < self.cost(tech_level) {
            return Err(BuildDenial::OutOfMoney);
        }
        Ok(())
    }
}

#[path = "building_kind/catalog.rs"]
mod catalog;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn initialized_cpp_registry_matches_every_kind_and_rule() {
        let mut catalog = String::new();
        for kind in BuildingKind::ALL {
            let m = kind.metadata();
            assert_eq!(BuildingKind::from_id(kind.id()), Some(kind));
            assert_eq!(BuildingKind::from_resource(m.resource), Some(kind));
            write!(
                catalog,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                kind.id(),
                m.resource,
                m.name,
                m.plural,
                m.size,
                m.colour,
                m.base_cost,
                m.cost_multiplier,
                m.demolition_cost,
                m.fire_chance,
                m.required_tech,
                m.range,
                u8::from(m.no_credit),
                kind.cost(783419)
            )
            .unwrap();
            for commodity in Commodity::ALL {
                let rule = m.rule(commodity);
                write!(
                    catalog,
                    "\t{},{},{}",
                    rule.max_load,
                    u8::from(rule.take),
                    u8::from(rule.give)
                )
                .unwrap();
            }
            catalog.push('\n');
        }
        assert_eq!(catalog, include_str!("building_kind/reference.tsv"));
        assert_eq!(BuildingKind::from_id(0), None);
        assert_eq!(BuildingKind::from_resource("Green"), None);
    }

    #[test]
    fn construction_checks_preserve_denial_order_and_exact_thresholds() {
        use BuildDenial::*;
        let campus = BuildingKind::University;
        assert_eq!(campus.can_build(0, -1, 7, 1), Err(NotEnoughStudents));
        assert_eq!(campus.can_build(0, -1, 8, 1), Err(NotEnoughTech));
        let tech = campus.metadata().required_tech;
        let cost = campus.cost(tech);
        assert_eq!(campus.can_build(tech, cost - 1, 8, 1), Err(OutOfMoney));
        assert_eq!(campus.can_build(tech, cost, 8, 1), Ok(()));
        assert_eq!(BuildingKind::Track.can_build(0, -1, 0, 0), Ok(()));
    }
}
