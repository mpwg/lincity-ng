use super::*;
use BuildingKind::*;

const ROAD_FRAMES: [i32; 16] = [0, 0, 1, 2, 0, 0, 3, 4, 1, 5, 1, 6, 7, 8, 9, 10];
const LINE_FRAMES: [i32; 16] = [0, 1, 0, 2, 0, 3, 0, 4, 1, 1, 5, 6, 7, 8, 9, 10];
// Frame priorities for ramp entrances use north, west, south, east.
const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (-1, 0), (0, 1), (1, 0)];
fn step(p: Point, (dx, dy): (i32, i32), distance: i32) -> Point {
    Point {
        x: p.x + dx * distance,
        y: p.y + dy * distance,
    }
}
fn bridge_kind(kind: BuildingKind) -> BuildingKind {
    match kind {
        Track => TrackBridge,
        Road => RoadBridge,
        Rail => RailBridge,
        _ => unreachable!(),
    }
}
impl City {
    pub(in crate::city) fn refresh_transport(&mut self, origin: Point, high: Point) {
        for y in origin.y.max(1)..=high.y.min(self.side - 2) {
            for x in origin.x.max(1)..=high.x.min(self.side - 2) {
                let p = Point { x, y };
                let Some(id) = self.buildings.at(p) else {
                    continue;
                };
                let (anchor, b) = self.buildings.get(id).unwrap();
                if anchor != p {
                    continue;
                }
                if b.kind == PowerLine {
                    self.connect_line(id, p);
                    continue;
                }
                if b.kind.is_transport() {
                    let (frame, transparent) = self.route_frame(p, b.kind);
                    let Operation::Transport(t) =
                        &mut self.buildings.get_mut(id).unwrap().operation
                    else {
                        unreachable!()
                    };
                    t.frame = frame;
                    t.transparent = transparent;
                }
            }
        }
    }
    fn route_frame(&self, p: Point, kind: BuildingKind) -> (i32, bool) {
        let base = base_kind(kind);
        let bridge = bridge_kind(base);
        let groups = |n| DIRECTIONS.map(|d| self.building_at(step(p, d, n)).map(|(g, _)| g));
        let near = groups(1);
        let far = groups(2);
        let vertical = |g| near[0] == Some(g) || near[2] == Some(g);
        let horizontal = |g| near[1] == Some(g) || near[3] == Some(g);
        if kind == bridge {
            let aligned =
                vertical(bridge) || ((base != Road || !horizontal(bridge)) && vertical(base));
            return (i32::from(!aligned), true);
        }
        for a in 0..2 {
            let b = a + 2;
            if (near[a] == Some(bridge) && (near[b] == Some(bridge) || far[b] == Some(bridge)))
                || (near[b] == Some(bridge) && far[a] == Some(bridge))
            {
                return (11 + a as i32, true);
            }
        }
        if let Some(d) = near.iter().position(|g| *g == Some(bridge)) {
            return (13 + d as i32, base != Track);
        }
        if base != Track
            && let Some(d) = (0..4).find(|&d| far[d] == Some(bridge) && near[d] == Some(base))
        {
            return (17 + d as i32, false);
        }
        let pair = |d: usize, g| near[d] == Some(g) && near[d + 2] == Some(g);
        for (lane, offset) in [(Track, 0), (Road, 2)] {
            if base == Rail || base == lane {
                if pair(1, lane) && (base == Rail || pair(0, Rail)) {
                    return (22 + offset, false);
                }
                if pair(0, lane) && (base == Rail || pair(1, Rail)) {
                    return (21 + offset, false);
                }
            }
        }
        let mut mask = 0;
        for d in 0..4 {
            let n = step(p, DIRECTIONS[d], 1);
            // A building only connects its upper-left tile visually when east/south of a route.
            let g = if d >= 2 {
                self.building_at(n)
                    .filter(|(_, anchor)| *anchor == n)
                    .map(|(k, _)| k)
            } else {
                near[d]
            };
            let road = base != Rail;
            let joins = g == Some(base)
                || (road && matches!(g, Some(Road | Track)))
                || (road && g == Some(Rail) && far[d] == Some(base))
                || (d >= 2
                    && ((base == Track && g == Some(TrackBridge))
                        || matches!(
                            g,
                            Some(
                                Commune
                                    | CoalMine
                                    | OreMine
                                    | LightIndustry
                                    | HeavyIndustry
                                    | Recycle
                                    | Tip
                                    | Port
                                    | CoalPower
                            )
                        )));
            if joins {
                mask |= [2, 1, 8, 4][d];
            }
        }
        (ROAD_FRAMES[mask], false)
    }

    pub(in crate::city) fn clear_line_cables(&mut self, p: Point) {
        for d in DIRECTIONS {
            if let Some(i) = self.index(step(p, d, 1)) {
                self.tiles[i].cables &= if d.0 == 0 { !1 } else { !2 };
            }
        }
    }
    fn connect_line(&mut self, id: BuildingId, p: Point) {
        self.disconnect_building(id);
        let mut mask = 0;
        for (d, bit) in [((0, -1), 8), ((-1, 0), 4), ((1, 0), 2), ((0, 1), 1)] {
            let near = step(p, d, 1);
            let i = self.index(near).unwrap();
            let farther = step(p, d, 2);
            let far = self.index(farther).is_some()
                && (self.tiles[i].ground == Terrain::Water
                    || self
                        .building_at(near)
                        .is_some_and(|(k, _)| k.is_transport()));
            let end = if far { farther } else { near };
            let cable = if d.0 == 0 { 1 } else { 2 };
            let Some(peer) = self.buildings.at(end).filter(|&q| {
                let (_, b) = self.buildings.get(q).unwrap();
                let cap = b.kind.metadata().rule(HighVoltage).max_load;
                cap != 0 && i64::from(b.stock[HighVoltage as usize]) * 1000 / i64::from(cap) != -1
            }) else {
                self.tiles[i].cables &= !cable;
                continue;
            };
            if far {
                let (corner, b) = self.buildings.get(peer).unwrap();
                let size = i32::from(b.kind.metadata().size);
                // Scan just outside the side of the remote footprint facing the span.
                let edge = match d {
                    (0, -1) => Point {
                        x: corner.x,
                        y: corner.y + size,
                    },
                    (-1, 0) => Point {
                        x: corner.x + size,
                        y: corner.y,
                    },
                    (1, 0) => Point {
                        x: corner.x - 1,
                        y: corner.y,
                    },
                    _ => Point {
                        x: corner.x,
                        y: corner.y - 1,
                    },
                };
                if (0..size).any(|n| {
                    let point = step(edge, if d.0 == 0 { (1, 0) } else { (0, 1) }, n);
                    self.index(point)
                        .is_some_and(|i| self.tiles[i].cables & cable != 0)
                }) {
                    continue;
                }
                self.tiles[i].cables |= cable;
            }
            self.buildings.get_mut(id).unwrap().neighbors.push(peer);
            self.buildings.get_mut(peer).unwrap().neighbors.push(id);
            mask |= bit;
        }
        let Operation::Line(line) = &mut self.buildings.get_mut(id).unwrap().operation else {
            unreachable!()
        };
        line.frame = LINE_FRAMES[mask];
    }
}
