//! Commodity exchange between neighboring buildings.
use super::*;
use crate::Commodity;
use crate::buildings::BuildingId;

impl City {
    /// Returns the configured directions for a local market or port that handles the commodity.
    pub fn building_trade_policy(&self, point: Point, c: Commodity) -> Option<crate::TradePolicy> {
        let (_, building) = self.buildings.get(self.buildings.at(point)?)?;
        match &building.operation {
            Operation::Market(market) => {
                let rule = market.rules[c as usize];
                (rule.max_load > 0).then_some(crate::TradePolicy {
                    buy: rule.take,
                    sell: rule.give,
                })
            }
            Operation::Port(port) => port.policy(c),
            _ => None,
        }
    }

    /// Returns the world import and export permissions applied to every port.
    pub fn foreign_trade_policy(&self, c: Commodity) -> crate::TradePolicy {
        let (buy, sell) = self.economy.trade[c as usize];
        crate::TradePolicy { buy, sell }
    }

    pub(super) fn connect_building(&mut self, point: Point) {
        let Some(center_id) = self.buildings.at(point) else {
            return;
        };
        let (_, center) = self.buildings.get(center_id).unwrap();
        let kind = center.kind;
        if matches!(kind, BuildingKind::Fire | BuildingKind::PowerLine) {
            return;
        }
        let size = i32::from(kind.metadata().size);
        let transport = kind.is_transport();
        let mut candidates = Vec::new();
        if transport {
            for p in [
                Point {
                    x: point.x - 1,
                    ..point
                },
                Point {
                    y: point.y - 1,
                    ..point
                },
                Point {
                    x: point.x + 1,
                    ..point
                },
                Point {
                    y: point.y + 1,
                    ..point
                },
            ] {
                if let Some(id) = self.buildings.at(p) {
                    if !candidates.contains(&id) {
                        candidates.push(id);
                    }
                }
            }
        } else {
            for y in (point.y - 10).max(1)..(point.y + 10).min(self.side - 1) {
                for x in (point.x - 10).max(1)..(point.x + 10).min(self.side - 1) {
                    let p = Point { x, y };
                    if p != point
                        && let Some(id) = self.buildings.at(p)
                        && self.buildings.get(id).unwrap().0 == p
                    {
                        candidates.push(id);
                    }
                }
            }
        }
        for peer in candidates {
            let (other_origin, b) = self.buildings.get(peer).unwrap();
            let other = b.kind;
            if matches!(other, BuildingKind::Fire | BuildingKind::PowerLine) {
                continue;
            }
            if !Commodity::ALL.into_iter().any(|c| {
                let a = kind.metadata().rule(c);
                let b = other.metadata().rule(c);
                (a.take && b.give) || (a.give && b.take)
            }) {
                continue;
            }
            let other_size = i32::from(other.metadata().size);
            let dx = other_origin.x - point.x;
            let dy = other_origin.y - point.y;
            let adjacent = (-other_size..=size).contains(&dx)
                && (-other_size..=size).contains(&dy)
                && !((dx == size || dx == -other_size) && (dy == size || dy == -other_size));
            if adjacent {
                self.buildings
                    .get_mut(center_id)
                    .unwrap()
                    .neighbors
                    .push(peer);
                self.buildings
                    .get_mut(peer)
                    .unwrap()
                    .neighbors
                    .push(center_id);
            } else if !transport
                && !other.is_transport()
                && ((kind == BuildingKind::Market) != (other == BuildingKind::Market))
            {
                self.buildings
                    .get_mut(center_id)
                    .unwrap()
                    .partners
                    .push(peer);
                self.buildings
                    .get_mut(peer)
                    .unwrap()
                    .partners
                    .push(center_id);
            }
        }
    }

    pub(super) fn disconnect_building(&mut self, id: BuildingId) {
        if let Some((p, b)) = self.buildings.get(id) {
            if b.kind == BuildingKind::PowerLine {
                self.clear_line_cables(p);
            } else if b.kind.is_transport() {
                let i = self.index(p).unwrap();
                self.tiles[i].cables = 0;
            }
        }
        let Some(b) = self.buildings.get_mut(id) else {
            return;
        };
        let peers = std::mem::take(&mut b.neighbors);
        let partners = std::mem::take(&mut b.partners);
        for peer in peers {
            if let Some(b) = self.buildings.get_mut(peer) {
                b.neighbors.retain(|p| *p != id);
            }
        }
        for peer in partners {
            if let Some(b) = self.buildings.get_mut(peer) {
                b.partners.retain(|p| *p != id);
            }
        }
    }

    pub(super) fn trade_at(&mut self, id: BuildingId) -> io::Result<()> {
        use crate::energy::{TradeStock, trade};
        let (_, b) = self.buildings.get(id).unwrap();
        if b.neighbors.is_empty() && !b.kind.is_transport() && b.kind != BuildingKind::PowerLine {
            return Ok(());
        }
        let neighbors = b.neighbors.clone();
        let mut peers = Vec::with_capacity(neighbors.len());
        for c in Commodity::ALL {
            let index = c as usize;
            let snapshot = |b: &Building| {
                let rule = b.kind.metadata().rule(c);
                let mut item = TradeStock::new(rule.max_load, rule.take, rule.give);
                item.amount = b.stock[index];
                item.evacuating = b.evacuating;
                item
            };
            let mut center = snapshot(self.buildings.get(id).unwrap().1);
            if self
                .buildings
                .get(id)
                .unwrap()
                .1
                .kind
                .metadata()
                .rule(c)
                .max_load
                == 0
            {
                continue;
            }
            peers.clear();
            for &peer in &neighbors {
                if let Some((_, building)) = self.buildings.get(peer) {
                    peers.push((peer, snapshot(building)));
                }
            }
            let old_amount = center.amount;
            let mut max_transfer = 0;
            trade(
                &mut center,
                &mut peers.iter_mut().map(|(_, stock)| stock).collect::<Vec<_>>(),
            );
            self.buildings.get_mut(id).unwrap().stock[index] = center.amount;
            for (peer, stock) in &peers {
                let receiver = self.buildings.get_mut(*peer).unwrap();
                let delta = stock.amount - receiver.stock[index];
                max_transfer = max_transfer.max(delta.abs());
                if delta > 0
                    && receiver.kind != BuildingKind::Market
                    && !receiver.kind.is_transport()
                    && !receiver.evacuating
                {
                    self.economy.tax_receipt(c, delta);
                }
                if delta < 0 && c == Commodity::Coal {
                    let (_, center) = self.buildings.get(id).unwrap();
                    if !center.evacuating
                        && !center.kind.is_transport()
                        && center.kind != BuildingKind::Market
                    {
                        self.economy.tax_receipt(c, -delta);
                    }
                }
                self.buildings.get_mut(*peer).unwrap().stock[index] = stock.amount;
            }
            self.record_transport_trade(
                id,
                c,
                old_amount,
                center.amount,
                max_transfer,
                &neighbors,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adjacent_power_buildings_trade_before_each_update_and_detach_on_demolition() {
        let left = Point { x: 3, y: 4 };
        let right = Point { x: 7, y: 4 };
        let source = r#"<lc-game ldsv-version="2140"><globals/><map size="20">
        <Construction group="2" map-x="3" map-y="4"><hivolt>1800</hivolt><labor>350</labor></Construction>
        <Construction group="3" map-x="7" map-y="4"/></map></lc-game>"#;
        let mut city = City::load(source.as_bytes()).unwrap();
        let a = city.buildings.at(left).unwrap();
        let b = city.buildings.at(right).unwrap();
        city.trade_at(b).unwrap();
        assert_eq!(city.stock(left, Commodity::HighVoltage), Some(666));
        assert_eq!(city.stock(right, Commodity::HighVoltage), Some(1134));
        city.apply(Action::Bulldoze, right).unwrap();
        city.trade_at(a).unwrap();
        assert_eq!(city.stock(left, Commodity::HighVoltage), Some(666));
    }
}
