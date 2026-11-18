use super::*;
use BuildingKind::{Rail, Road, RoadBridge, Track, TrackBridge};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Strategy {
    Maximize,
    Minimize,
}

/// Host-visible isometric sprite including the tile that owns it and pixel offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VehicleFrame {
    pub tile: Point,
    pub frame: i32,
    pub offset_x: i32,
    pub offset_y: i32,
}

pub(in crate::city) struct Vehicle {
    point: Point,
    next: Point,
    prev: Point,
    old1: Point,
    old2: Point,
    strategy: Strategy,
    initial_cargo: i32,
    direction: i32,
    left: bool,
    next_move: u64,
    speed: u64,
    remaining_steps: i32,
    sprite: VehicleFrame,
}

impl City {
    fn transport_group(&self, p: Point) -> Option<BuildingKind> {
        self.building_at(p).map(|(k, _)| base_kind(k))
    }
    fn labor_ratio(&self, p: Point) -> Option<i32> {
        let (_, b) = self.buildings.get(self.buildings.at(p)?)?;
        let cap = b.kind.metadata().rule(Labor).max_load;
        Some(if cap == 0 {
            -1
        } else {
            (i64::from(b.stock[Labor as usize]) * i64::from(QUANTA) / i64::from(cap)) as i32
        })
    }

    /// Current commuter sprites in simulation order, including new hidden vehicles.
    pub fn vehicle_frames(&self) -> impl Iterator<Item = VehicleFrame> + '_ {
        self.vehicles.iter().map(|v| v.sprite)
    }

    pub(in crate::city) fn animate_vehicles(&mut self, time: u64) -> io::Result<()> {
        let mut vehicles = std::mem::take(&mut self.vehicles);
        let result = vehicles.iter_mut().try_for_each(|v| v.update(self, time));
        vehicles.retain(|v| v.remaining_steps > 0);
        self.vehicles = vehicles;
        result
    }

    pub(super) fn maybe_spawn(
        &mut self,
        origin: Point,
        per_mille: i32,
        flow: i32,
        old_amount: i32,
        capacity: i32,
    ) -> io::Result<()> {
        if self.traffic == VehicleTraffic::Disabled
            || 100 * per_mille * RATE / QUANTA <= 2
            || self.transport_group(origin) != Some(Road)
        {
            return Ok(());
        }
        let mut chance = 50 * per_mille * RATE / QUANTA;
        if self.traffic == VehicleTraffic::Reduced {
            chance = (chance + 1) / 2;
        }
        // Availability is tested after the draw, including when the tile is occupied.
        if self.decisions.c_modulo(COMMUTER_TRAFFIC_RATE)? >= (chance + 1) / 2
            || self.vehicles.iter().any(|v| v.sprite.tile == origin)
        {
            return Ok(());
        }
        self.vehicles.push(Vehicle::new(
            origin,
            flow > 0,
            old_amount * QUANTA / capacity,
        ));
        Ok(())
    }
}

impl Vehicle {
    fn new(point: Point, maximize: bool, initial_cargo: i32) -> Self {
        Self {
            point,
            next: point,
            prev: point,
            old1: point,
            old2: point,
            strategy: if maximize {
                Strategy::Maximize
            } else {
                Strategy::Minimize
            },
            initial_cargo,
            direction: 0,
            left: true,
            next_move: 0,
            speed: CAR_SPEED,
            remaining_steps: CAR_RANGE,
            sprite: VehicleFrame {
                tile: point,
                frame: -2,
                offset_x: 0,
                offset_y: 0,
            },
        }
    }

    fn acceptable(&self, city: &City, dest: Point) -> bool {
        let group = city.transport_group(dest);
        if !matches!(group, Some(Road | Track)) {
            let approach = city.transport_group(self.point);
            let exit = Point {
                x: 2 * dest.x - self.point.x,
                y: 2 * dest.y - self.point.y,
            };
            if group != Some(Rail)
                || !matches!(approach, Some(Road | Track))
                || city.transport_group(exit) != approach
            {
                return false;
            }
        }
        let Some(other) = city.labor_ratio(dest) else {
            return false;
        };
        let Some(current) = city.labor_ratio(self.point) else {
            return true;
        };
        match self.strategy {
            Strategy::Maximize => {
                current * 24 / 25 < other && self.initial_cargo * 99 / 100 < other
            }
            Strategy::Minimize => {
                current > other * 24 / 25 && self.initial_cargo > other * 99 / 100
            }
        }
    }

    fn update(&mut self, city: &mut City, time: u64) -> io::Result<()> {
        if self.point == self.next {
            let mut weights = [0.; 4];
            let p = self.point;
            let destinations = [
                Point { x: p.x + 1, ..p },
                Point { x: p.x - 1, ..p },
                Point { y: p.y - 1, ..p },
                Point { y: p.y + 1, ..p },
            ];
            let forward = [
                p.x >= self.prev.x,
                p.x <= self.prev.x,
                p.y <= self.prev.y,
                p.y >= self.prev.y,
            ];
            for i in 0..4 {
                if forward[i] && self.acceptable(city, destinations[i]) {
                    weights[i] = 1.;
                }
            }
            if weights.iter().all(|&w| w == 0.) {
                self.remaining_steps = 0;
            } else {
                self.next =
                    destinations[city.decisions.discrete("vehicle.heading", &weights)? as usize];
            }
        }
        if time > self.next_move {
            self.drive();
            self.next_move = time
                + if self.sprite.frame < 0 {
                    HIDDEN_CAR_INTERVAL
                } else {
                    self.speed
                };
        }
        self.walk(city, time);
        Ok(())
    }
    fn drive(&mut self) {
        self.remaining_steps -= 1;
        let from_vertical = self.prev.x == self.point.x;
        let (direction, left) = match (self.next.x - self.prev.x, self.next.y - self.prev.y) {
            (2, _) => (2, self.left),
            (-2, _) => (6, self.left),
            (_, 2) => (0, self.left),
            (_, -2) => (4, self.left),
            (1, 1) => (1, from_vertical),
            (1, -1) => (3, !from_vertical),
            (-1, 1) => (7, !from_vertical),
            (-1, -1) => (5, from_vertical),
            _ => (self.direction, self.left),
        };
        self.direction = direction;
        self.left = left;
        self.speed = if direction & 1 == 0 {
            CAR_SPEED
        } else if left {
            3 * CAR_SPEED / 2
        } else {
            CAR_SPEED / 2
        };
        self.old2 = self.old1;
        self.old1 = self.prev;
        self.prev = self.point;
        self.point = self.next;
        if self.sprite.frame < -1 {
            self.sprite.frame += 1;
        } else {
            self.sprite.frame = direction % 16;
        }
    }
    fn walk(&mut self, city: &City, time: u64) {
        let r = (self.next_move - time) as f64 / self.speed as f64;
        let e = 1. - r;
        let (x, y) = match (self.direction, self.left) {
            (0, _) => (-0.65, -r),
            (4, _) => (-0.27, -e),
            (2, _) => (-r, -0.27),
            (6, _) => (-e, -0.65),
            (1, true) => (-0.65 + e * e, -1.13 + e),
            (1, false) => (-1. + 0.35 * e, -0.27 + 0.27 * e * e),
            (5, true) => (-0.27 - 0.73 * e * e, -0.65 * e),
            (5, false) => (-0.23 * e, -0.65 - 0.35 * e * e),
            (3, true) if e < 0.5 => (-1. + e, -0.27 - 0.5 * e * e),
            (3, true) => (-0.27 - 0.5 * r * r, -1. + r),
            (3, false) => (-0.27 * r * r, -0.27 * e),
            (7, true) => (-0.65 * e, -0.65 + 0.65 * e * e),
            (7, false) => (-0.65 - 0.35 * e * e, -1. + 0.35 * e),
            _ => unreachable!(),
        };
        // The original stores the world position in binary32 before projecting it.
        let x = f64::from((f64::from(self.prev.x) + (x + 0.25)) as f32);
        let y = f64::from((f64::from(self.prev.y) + (y + 0.25)) as f32);
        let tile = Point {
            x: (x.ceil() as i32).max(self.prev.x),
            y: (y.ceil() as i32).max(self.prev.y),
        };
        let (mx, my) = (x - f64::from(tile.x), y - f64::from(tile.y));
        let mut dy = ((mx + my) * 32.) as i32;
        let at = |p| city.building_at(p).map(|(k, _)| k);
        match at(self.prev) {
            Some(TrackBridge) => dy -= TRACK_BRIDGE_HEIGHT,
            Some(RoadBridge) => dy -= ROAD_BRIDGE_HEIGHT,
            Some(Road)
                if at(self.next) == Some(RoadBridge) || at(self.point) == Some(RoadBridge) =>
            {
                dy = (f64::from(dy) - e * f64::from(ROAD_BRIDGE_HEIGHT) / 2.) as i32;
                self.sprite.frame = 8 + self.direction / 2;
                if at(self.point) == Some(RoadBridge) {
                    dy -= ROAD_BRIDGE_HEIGHT / 2;
                }
            }
            Some(Road)
                if at(self.old2) == Some(RoadBridge) || at(self.old1) == Some(RoadBridge) =>
            {
                dy = (f64::from(dy) - r * f64::from(ROAD_BRIDGE_HEIGHT) / 2.) as i32;
                self.sprite.frame = 12 + self.direction / 2;
                if at(self.old1) == Some(RoadBridge) {
                    dy -= ROAD_BRIDGE_HEIGHT / 2;
                }
            }
            Some(Track) if at(self.point) == Some(TrackBridge) => {
                dy = (f64::from(dy) - e * f64::from(TRACK_BRIDGE_HEIGHT)) as i32;
                self.sprite.frame = 8 + self.direction / 2;
            }
            Some(Track) if at(self.old1) == Some(TrackBridge) => {
                dy = (f64::from(dy) - r * f64::from(TRACK_BRIDGE_HEIGHT)) as i32;
                self.sprite.frame = 12 + self.direction / 2;
            }
            _ => {}
        }
        self.sprite.tile = tile;
        self.sprite.offset_x = ((mx - my) * 64.) as i32;
        self.sprite.offset_y = dy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BuildingKind::*;
    use std::fmt::Write;

    const SITE: Point = Point { x: 12, y: 12 };
    const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (-1, 0), (0, 1), (1, 0)];
    fn step((x, y): (i32, i32), n: i32) -> Point {
        Point {
            x: SITE.x + x * n,
            y: SITE.y + y * n,
        }
    }

    fn setup(tiles: impl IntoIterator<Item = (Point, BuildingKind, bool)>) -> City {
        let mut city = City::empty(32).unwrap();
        for (p, k, water) in tiles {
            let i = city.index(p).unwrap();
            if water {
                city.tiles[i].ground = Terrain::Water;
            }
            let mut b = Building::new(k);
            b.stock[Labor as usize] = if k == Rail { 2560 } else { 512 };
            city.place_building(p, 1, b).unwrap();
        }
        city.refresh_transport(Point { x: 1, y: 1 }, Point { x: 30, y: 30 });
        city
    }

    fn drive(path: [Point; 6], bridge: bool, rail: bool, transcript: &mut String) {
        let mut city = setup(path.into_iter().enumerate().map(|(i, p)| {
            let k = if i == 2 && rail {
                Rail
            } else if i == 2 && bridge {
                RoadBridge
            } else {
                Road
            };
            (p, k, i == 2 && bridge)
        }));
        let mut v = Vehicle::new(path[0], true, 100);
        for time in [1, 52, 239, 615, 1200, 2304, 3150, 4600] {
            v.update(&mut city, time).unwrap();
            let f = v.sprite;
            writeln!(
                transcript,
                "{time} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                v.point.x,
                v.point.y,
                v.prev.x,
                v.prev.y,
                v.next.x,
                v.next.y,
                f.tile.x,
                f.tile.y,
                f.frame,
                f.offset_x,
                f.offset_y,
                v.direction,
                v.speed,
                v.next_move,
                v.remaining_steps
            )
            .unwrap();
        }
    }

    #[test]
    fn native_transport_and_vehicle_frames_agree_at_every_sample() {
        let mut out = String::new();
        for (kind, bridge) in [(Track, TrackBridge), (Road, RoadBridge), (Rail, RailBridge)] {
            for d in DIRECTIONS {
                let city = setup([
                    (step(d, 1), bridge, true),
                    (SITE, kind, false),
                    (step(d, -1), kind, false),
                ]);
                write!(out, "bridge {} {} {}", kind.id(), d.0, d.1).unwrap();
                for n in [1, 0, -1] {
                    let (_, b) = city
                        .buildings
                        .get(city.buildings.at(step(d, n)).unwrap())
                        .unwrap();
                    let Operation::Transport(t) = &b.operation else {
                        panic!()
                    };
                    write!(out, " {} {}", t.frame, i32::from(t.transparent)).unwrap();
                }
                out.push('\n');
            }
        }
        for a in DIRECTIONS {
            for b in DIRECTIONS {
                if a.0 * b.0 + a.1 * b.1 != 0 {
                    continue;
                }
                writeln!(out, "turn {} {} {} {}", a.0, a.1, b.0, b.1).unwrap();
                drive(
                    [
                        step(a, -1),
                        SITE,
                        step(b, 1),
                        step(b, 2),
                        step(b, 3),
                        step(b, 4),
                    ],
                    false,
                    false,
                    &mut out,
                );
            }
        }
        for bridge in [false, true] {
            writeln!(out, "vehicle {}", if bridge { "bridge" } else { "rail" }).unwrap();
            drive(
                [0, 1, 2, 3, 4, 5].map(|n| step((1, 0), n)),
                bridge,
                !bridge,
                &mut out,
            );
        }
        let expected = include_str!("reference.txt");
        assert_eq!(out.lines().count(), expected.lines().count());
        for (row, (a, b)) in out.lines().zip(expected.lines()).enumerate() {
            assert_eq!(a, b, "line {}", row + 1);
        }
    }
}
