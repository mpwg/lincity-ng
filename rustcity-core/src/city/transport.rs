use super::*;
use crate::Commodity::{self, Goods, HighVoltage, Labor, LowVoltage, Steel, Waste};
use crate::rules::{DAYS_PER_MONTH, power::LINE_MODULUS, transport::*};

mod topology;
mod vehicles;
pub(in crate::city) use vehicles::Vehicle;
pub use vehicles::VehicleFrame;

#[derive(Default)]
pub(super) struct Transport {
    pub frame: i32,
    pub transparent: bool,
    pub traffic: [i32; 10],
    fire: super::civic::WasteFire,
}
impl Transport {
    pub fn animate(&mut self, time: u64, decisions: &mut Decisions) -> io::Result<()> {
        self.fire.animate(time, decisions)
    }
}

fn base_kind(kind: BuildingKind) -> BuildingKind {
    use BuildingKind::*;
    match kind {
        TrackBridge => Track,
        RoadBridge => Road,
        RailBridge => Rail,
        k => k,
    }
}

impl City {
    pub(super) fn update_transport(&mut self, id: BuildingId, point: Point) -> io::Result<()> {
        let day = self.day;
        let i = self.index(point).unwrap();
        let tile = &mut self.tiles[i];
        let b = self.buildings.get_mut(id).unwrap();
        let Operation::Transport(t) = &mut b.operation else {
            unreachable!()
        };
        let kind = base_kind(b.kind);
        if kind != BuildingKind::Track {
            let rail = kind == BuildingKind::Rail;
            if day % DAYS_PER_MONTH == 0 {
                super::finance::expense(
                    &mut self.balance,
                    &mut self.stats.expenses.transport,
                    &mut self.events,
                    self.economy.transport_cost * if rail { RAIL_COST_MULTIPLIER } else { 1 },
                );
            }
            if day
                % if rail {
                    RAIL_POLLUTION_DAYS
                } else {
                    ROAD_POLLUTION_DAYS
                }
                == 0
            {
                tile.pollution += 1;
            }
            if day
                & if rail {
                    RAIL_GOODS_MASK
                } else {
                    ROAD_GOODS_MASK
                }
                == 0
                && b.stock[Goods as usize] > 0
            {
                b.production.change(&mut b.stock, Goods, -1);
                b.production.change(&mut b.stock, Waste, 1);
            }
            if rail && day & RAIL_STEEL_MASK == 0 && b.stock[Steel as usize] > 0 {
                b.production.change(&mut b.stock, Steel, -1);
                b.production.change(&mut b.stock, Waste, 1);
            }
        }
        let available = b.stock[LowVoltage as usize];
        let loss = if available >= VOLTAGE_LOSS {
            VOLTAGE_LOSS
        } else {
            i32::from(available > 0)
        };
        b.production.change(&mut b.stock, LowVoltage, -loss);
        let mut ignite = false;
        if b.stock[Waste as usize] > b.kind.metadata().rule(Waste).max_load * 9 / 10 {
            b.production.change(&mut b.stock, Waste, -BURN_WASTE);
            tile.pollution += BURN_POLLUTION;
            t.fire.start = true;
            ignite = self.decisions.bernoulli(
                "transport.waste-fire",
                UNCONTROLLED_FIRE * f64::from(b.kind.metadata().fire_chance) / 100.,
            )? && tile.coverage & 0x40000 == 0;
        }
        if day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1 {
            b.production.close_month();
        }
        if ignite {
            self.ignite_building(id, point)?;
        }
        Ok(())
    }
}

impl City {
    pub(super) fn record_transport_trade(
        &mut self,
        id: BuildingId,
        commodity: Commodity,
        old: i32,
        new: i32,
        max_transfer: i32,
        neighbors: &[BuildingId],
    ) -> io::Result<()> {
        let (point, b) = self.buildings.get(id).unwrap();
        let kind = b.kind;
        let capacity = kind.metadata().rule(commodity).max_load;
        if kind.is_transport() {
            let scaled = max_transfer * QUANTA / capacity;
            let Operation::Transport(t) = &mut self.buildings.get_mut(id).unwrap().operation else {
                unreachable!()
            };
            let count = &mut t.traffic[commodity as usize];
            *count = ((SMOOTHING - 1) * (*count) + scaled) / SMOOTHING;
            if commodity == Labor {
                self.maybe_spawn(point, scaled, new - old, old, capacity)?;
            }
        } else if kind == BuildingKind::PowerLine && commodity == HighVoltage {
            let Operation::Line(line) = &mut self.buildings.get_mut(id).unwrap().operation else {
                unreachable!()
            };
            line.record_traffic(max_transfer);
            let flashing = line.flashing;
            for &peer in neighbors {
                let Some((_, other)) = self.buildings.get(peer) else {
                    continue;
                };
                if other.kind == BuildingKind::PowerLine {
                    if flashing
                        && let Operation::Line(neighbor) =
                            &mut self.buildings.get_mut(peer).unwrap().operation
                        && neighbor.counter <= LINE_MODULUS / 3
                    {
                        neighbor.counter = LINE_MODULUS;
                    }
                } else if other.kind.metadata().rule(HighVoltage).give
                    && other.stock[HighVoltage as usize] > 0
                    && let Operation::Line(line) =
                        &mut self.buildings.get_mut(id).unwrap().operation
                    && line.counter == 0
                {
                    let draw = self.decisions.c_random()? as u32;
                    line.observe_supplier(true, || draw);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn labor_pickup_spawns_one_commuter_and_uses_the_native_heading_decision() {
        use std::io::Cursor;
        let xml = r#"<lc-game ldsv-version="2140"><globals/><map size="20">
            <Construction group="11" map-x="3" map-y="3"/>
            <Construction group="11" map-x="4" map-y="3"><labor>5120</labor></Construction>
        </map></lc-game>"#;
        let mut city = City::load(xml.as_bytes()).unwrap();
        city.set_vehicle_traffic(VehicleTraffic::Normal);
        city.decisions = Decisions::replay(Cursor::new(
            b"LINCITY-SIMULATION-TRACE 1\nC 0\nC 0\nS vehicle.heading 0\nZ\n",
        ))
        .unwrap();
        let site = Point { x: 3, y: 3 };
        let id = city.buildings.at(site).unwrap();
        city.trade_at(id).unwrap();
        city.trade_at(id).unwrap();
        assert_eq!(city.vehicle_frames().count(), 1);
        city.animate_vehicles(1).unwrap();
        assert_eq!(city.vehicle_frames().next().unwrap().frame, -1);
        city.finish_decisions().unwrap();
    }
}
