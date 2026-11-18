use super::*;
use crate::production::{
    CoalMine, HeavyIndustry, Landfill, LightIndustry, MineAction, Mineral, MiningTile, OreMine,
    Recycling, Stock,
};

pub(super) enum IndustryOperation {
    Coal(CoalMine),
    Ore(OreMine),
    Light(LightIndustry),
    Heavy(HeavyIndustry),
    Tip(Landfill),
    Recycle(Recycling),
}

impl IndustryOperation {
    pub(super) fn store(&self, node: &mut crate::save::Element) {
        match self {
            Self::Coal(m) => node.set_child("initial_coal_reserve", m.initial_reserve),
            Self::Ore(m) => node.set_child("total_ore_reserve", m.reserve),
            Self::Tip(t) => {
                node.set_child("total_waste", t.buried);
                node.set_child("degration_days", t.degradation);
                node.order_children(&["total_waste", "degration_days"]);
            }
            _ => {}
        }
    }

    pub(super) fn animate(&mut self, millis: u64, decisions: &mut Decisions) -> io::Result<()> {
        match self {
            Self::Ore(m) => m.animate(millis),
            Self::Light(m) => m.animate(millis, || decisions.c_random().map(|r| r as u32))?,
            Self::Heavy(m) => m.animate(millis),
            _ => {}
        }
        Ok(())
    }
}

impl MiningTile for Tile {
    fn reserve(&self, mineral: Mineral) -> i32 {
        match mineral {
            Mineral::Ore => self.ore,
            Mineral::Coal => self.coal,
        }
    }
    fn change_reserve(&mut self, mineral: Mineral, change: i32) {
        match mineral {
            Mineral::Ore => self.ore += change,
            Mineral::Coal => self.coal += change,
        }
    }
    fn pollute(&mut self, change: i32) {
        self.pollution += change;
    }
}

impl City {
    /// As in the native loader, placement recalculates mine reserves from the map
    /// after the saved building and tile fields have been read.
    pub(super) fn install_industry(&mut self, id: BuildingId, point: Point) -> io::Result<()> {
        let building = self.buildings.get_mut(id).unwrap();
        let tech = building.installed_tech;
        let factory = match building.kind {
            BuildingKind::CoalMine => {
                IndustryOperation::Coal(CoalMine::place(point, self.side, &mut self.tiles))
            }
            BuildingKind::OreMine => {
                IndustryOperation::Ore(OreMine::place(point, self.side, &self.tiles))
            }
            BuildingKind::LightIndustry => IndustryOperation::Light(LightIndustry::new(tech)),
            BuildingKind::HeavyIndustry => IndustryOperation::Heavy(HeavyIndustry::new(tech)),
            BuildingKind::Recycle => IndustryOperation::Recycle(Recycling::new(tech)),
            BuildingKind::Tip => {
                let mut tip = Landfill::default();
                if let Some(node) = building.saved.as_ref() {
                    tip.buried = node.child_i32("total_waste", 0)?;
                    tip.degradation = node.child_i32("degration_days", 0)?;
                }
                IndustryOperation::Tip(tip)
            }
            _ => return Ok(()),
        };
        building.operation = Operation::Industry(factory);
        Ok(())
    }

    pub(super) fn update_industry(&mut self, id: BuildingId, origin: Point, day: u32) {
        let index = self.index(origin).unwrap();
        let b = self.buildings.get_mut(id).unwrap();
        let Operation::Industry(operation) = &mut b.operation else {
            unreachable!()
        };
        let mut stock = Stock::new(&mut b.stock, &mut b.production);
        let sustainable = &mut self.stats.sustainability.mining_flag;
        let action = match operation {
            IndustryOperation::Coal(m) => m.update(day, &mut stock, &mut self.tiles, sustainable),
            IndustryOperation::Ore(m) => m.update(day, &mut stock, &mut self.tiles, sustainable),
            IndustryOperation::Heavy(m) => {
                self.tiles[index].pollution += m.update(day, &mut stock);
                return;
            }
            IndustryOperation::Light(m) => {
                self.tiles[index].pollution += m.update(day, &mut stock);
                return;
            }
            IndustryOperation::Tip(m) => {
                if m.update(day, &mut stock) {
                    *sustainable = false;
                }
                return;
            }
            IndustryOperation::Recycle(m) => {
                let paid = super::finance::expense(
                    &mut self.balance,
                    &mut self.stats.expenses.recycle,
                    &mut self.events,
                    Recycling::RUNNING_COST,
                );
                m.update(day, &mut stock, paid);
                return;
            }
        };
        match action {
            MineAction::Continue => {}
            MineAction::Evacuate => b.evacuating = true,
            MineAction::Abandon => self.remove_and_restore_site(id, origin),
        }
    }

    pub(super) fn remove_and_restore_site(&mut self, id: BuildingId, origin: Point) {
        self.disconnect_building(id);
        let b = self.buildings.remove(id).unwrap();
        self.stats.group_count[b.kind.id() as usize] -= 1;
        if let Operation::Civic(civic) = &b.operation
            && civic.occupants() != 0
        {
            self.displaced.insert(id, civic.occupants());
        }
        if let Operation::Industry(IndustryOperation::Ore(mine)) = b.operation {
            let points: Vec<_> = mine.lake_points(&self.tiles).collect();
            for p in points {
                let i = self.index(p).unwrap();
                self.tiles[i].ground = Terrain::Water;
                self.tiles[i].groundwater = true;
                self.tiles[i].frame = 0;
                self.connect_river(p);
            }
        }
        let size = i32::from(b.kind.metadata().size);
        self.refresh_transport(
            Point {
                x: origin.x - 2,
                y: origin.y - 2,
            },
            Point {
                x: origin.x + size + 1,
                y: origin.y + size + 1,
            },
        );
        self.update_frontiers(
            origin,
            Point {
                x: origin.x + size,
                y: origin.y + size,
            },
        );
    }

    fn connect_river(&mut self, origin: Point) {
        fn neighbors(p: Point) -> [Point; 4] {
            [
                Point { x: p.x, y: p.y - 1 },
                Point { x: p.x, y: p.y + 1 },
                Point { x: p.x + 1, y: p.y },
                Point { x: p.x - 1, y: p.y },
            ]
        }
        let i = self.index(origin).unwrap();
        if self.tiles[i].river
            || self.tiles[i].ground != Terrain::Water
            || !neighbors(origin)
                .into_iter()
                .filter_map(|p| self.index(p))
                .any(|i| self.tiles[i].river)
        {
            return;
        }
        self.tiles[i].river = true;
        let mut queue = std::collections::VecDeque::from([origin]);
        while let Some(p) = queue.pop_front() {
            for next in neighbors(p) {
                if let Some(i) = self.index(next)
                    && self.tiles[i].ground == Terrain::Water
                    && !self.tiles[i].river
                {
                    self.tiles[i].river = true;
                    queue.push_back(next);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Commodity;
    use std::fmt::Write;

    #[test]
    fn industry_visits_match_the_native_world_transcript() {
        let mut expected = include_str!("../production/cases/expected.txt").lines();
        for case in include_str!("../production/cases/input.txt").lines() {
            assert_eq!(expected.next(), Some(case));
            let values: Vec<i32> = case
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            let [group, tech, day, days, cash, coal, ore, ref amounts @ ..] = values[..] else {
                panic!("invalid case")
            };
            let mut stock_xml = String::new();
            for (commodity, amount) in Commodity::ALL.into_iter().zip(amounts) {
                let tag = commodity.save_name();
                write!(stock_xml, "<{tag}>{amount}</{tag}>").unwrap();
            }
            let input = format!(
                r#"<lc-game ldsv-version="2140"><globals><total_money>{cash}</total_money><total_time>{}</total_time><tech_level>{tech}</tech_level></globals><map size="32">
                <MapTile group="0" map-x="10" map-y="10"><coal>{coal}</coal><ore>{ore}</ore></MapTile>
                <Construction group="{group}" map-x="10" map-y="10">{stock_xml}</Construction></map></lc-game>"#,
                day - 1
            );
            let mut city = City::load(input.as_bytes()).unwrap();
            let site = Point { x: 10, y: 10 };
            for today in day..day + days {
                // The native oracle visits this building without running world
                // ecology or finance. The integration tests exercise full city days.
                if let Some(id) = city.buildings.at(site) {
                    city.update_industry(id, site, today as u32);
                }
                let tile = &city.tiles[city.index(site).unwrap()];
                let mut result = format!(
                    "{today} {} {} {} {} {} {}",
                    city.balance,
                    city.stats.expenses.recycle.acc,
                    i32::from(city.stats.sustainability.mining_flag),
                    tile.pollution,
                    tile.coal,
                    tile.ore
                );
                if let Some(id) = city.buildings.at(site) {
                    let (_, b) = city.buildings.get(id).unwrap();
                    write!(result, " {}", i32::from(b.evacuating)).unwrap();
                    for amount in b
                        .stock
                        .iter()
                        .chain(&b.production.current)
                        .chain(&b.production.previous)
                    {
                        write!(result, " {amount}").unwrap();
                    }
                } else {
                    result.push_str(" removed");
                }
                assert_eq!(
                    Some(result.as_str()),
                    expected.next(),
                    "case {case}, day {today}"
                );
            }
        }
        assert_eq!(expected.next(), None);
    }
}
