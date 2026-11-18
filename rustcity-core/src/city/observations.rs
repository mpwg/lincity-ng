//! Read-only views used by simulation hosts and parity tools.
use super::*;

/// Current world values reported by the decision replay protocol.
#[derive(Clone, Copy, Debug)]
pub struct CityReport {
    pub day: i32,
    pub balance: i32,
    pub people_pool: i32,
    pub housed_acc: i32,
    pub population_acc: i32,
    pub tech_level: i32,
}

impl City {
    pub fn report(&self) -> CityReport {
        CityReport {
            day: self.day,
            balance: self.balance,
            people_pool: self.people_pool,
            housed_acc: self.stats.population.housed_m.acc,
            population_acc: self.stats.population.population_m.acc,
            tech_level: self.tech_level,
        }
    }

    /// Current stock and origin of each live building.
    pub fn building_inventories(&self) -> impl Iterator<Item = (Point, BuildingKind, &[i32; 10])> {
        self.buildings.iter().map(|(p, b)| (p, b.kind, &b.stock))
    }

    /// Price at a visible tile, including for an action the city rejects.
    pub fn quote(&self, action: Action, p: Point) -> Option<i32> {
        let visible = 1..self.side - 1;
        if !visible.contains(&p.x) || !visible.contains(&p.y) {
            return None;
        }
        Some(match action {
            Action::Build(kind) => self.actual_kind(kind, p).cost(self.tech_level),
            Action::Bulldoze => self
                .building_at(p)
                .map(|(k, _)| k.metadata().demolition_cost)
                .unwrap_or_else(|| match self.terrain(p) {
                    Some(Terrain::Desert) => 0,
                    Some(Terrain::Water) => crate::rules::DRAIN_COST,
                    _ => crate::rules::CLEAR_LAND_COST,
                }),
            Action::Flood => crate::rules::FLOOD_COST,
            Action::Evacuate | Action::Query => 0,
        })
    }

    /// Resolves alternate groups such as bridges and high-technology wind power.
    pub fn building_kind_at_site(&self, kind: BuildingKind, p: Point) -> BuildingKind {
        self.actual_kind(kind, p)
    }
}
