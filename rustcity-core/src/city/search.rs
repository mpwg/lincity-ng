//! Searches the playable map in the order used for selecting construction sites.
use super::{BuildingKind, City, Point, Terrain};

/// Local relief relative to a playable tile's eight neighbors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Landform {
    pub minimum: bool,
    pub maximum: bool,
    pub saddle: bool,
    pub edge_minimum: bool,
}

impl City {
    /// Classifies a tile's elevation. The outermost border has no complete neighborhood.
    pub fn landform(&self, p: Point) -> Option<Landform> {
        let last = self.side - 2;
        if !(1..=last).contains(&p.x) || !(1..=last).contains(&p.y) {
            return None;
        }
        let altitude = |dx, dy| self.tiles[((p.y + dy) * self.side + p.x + dx) as usize].altitude;
        let center = altitude(0, 0);
        let ring = [
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
        ]
        .map(|(dx, dy)| altitude(dx, dy));
        let lower_runs = ring
            .iter()
            .zip(ring.iter().cycle().skip(1))
            .filter(|(before, after)| **before >= center && **after < center)
            .count();
        let (west, east, north, south) = (ring[0], ring[4], ring[2], ring[6]);
        let edge_pair = match (p.x, p.y) {
            (1, 1) => Some((south, east)),
            (1, y) if y == last => Some((west, south)),
            (x, 1) if x == last => Some((north, east)),
            (x, y) if x == last && y == last => Some((west, north)),
            (x, _) if x == 1 || x == last => Some((west, east)),
            (_, y) if y == 1 || y == last => Some((north, south)),
            _ => None,
        };
        Some(Landform {
            minimum: ring.iter().all(|&h| center <= h),
            maximum: ring.iter().all(|&h| center >= h),
            saddle: lower_runs > 1,
            edge_minimum: edge_pair.is_some_and(|(a, b)| center < a && center < b),
        })
    }

    fn spiral_search(&self, mut p: Point, mut matches: impl FnMut(Point) -> bool) -> Option<Point> {
        // The search radius is less than twice the side length. Exclude distant
        // coordinates before stepping to avoid overflowing either coordinate.
        let reachable = -2 * self.side..3 * self.side;
        if !reachable.contains(&p.x) || !reachable.contains(&p.y) {
            return None;
        }
        for i in 2..self.side * 4 {
            let delta = if i & 2 != 0 { -1 } else { 1 };
            for _ in 0..i / 2 {
                if (1..self.side - 1).contains(&p.x)
                    && (1..self.side - 1).contains(&p.y)
                    && matches(p)
                {
                    return Some(p);
                }
                if i & 1 != 0 {
                    p.y += delta;
                } else {
                    p.x += delta;
                }
            }
        }
        None
    }

    /// Finds the first building origin of this kind in an outward spiral.
    pub fn find_building(&self, start: Point, kind: BuildingKind) -> Option<Point> {
        self.spiral_search(start, |p| self.building_at(p) == Some((kind, p)))
    }

    /// Finds a terrain group in spiral order. A building origin hides the terrain there.
    pub fn find_terrain(&self, start: Point, ground: Terrain) -> Option<Point> {
        self.spiral_search(start, |p| {
            self.terrain(p) == Some(ground)
                && !self.building_at(p).is_some_and(|(_, origin)| origin == p)
        })
    }

    /// Finds the northwestern corner of a vacant square within the playable map.
    pub fn find_bare_area(&self, start: Point, size: i32) -> Option<Point> {
        if !(1..=self.side - 2).contains(&size) {
            return None;
        }
        self.spiral_search(start, |p| {
            p.x + size < self.side
                && p.y + size < self.side
                && (p.y..p.y + size).all(|y| {
                    (p.x..p.x + size).all(|x| {
                        let q = Point { x, y };
                        self.buildings.at(q).is_none() && self.terrain(q) != Some(Terrain::Water)
                    })
                })
        })
    }
}
