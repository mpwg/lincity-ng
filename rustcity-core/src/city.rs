use crate::stats::Stats;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Grass,
    Desert,
    Water,
    Tree,
    Tree2,
    Tree3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Query,
    Flood,
    Bulldoze,
    Evacuate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionError {
    OutsideMap(Point),
    SpaceOccupied(Point),
    NothingToEvacuate(Point),
    OutOfMoney,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapSizeError;
#[derive(Clone, Copy)]
struct Tile {
    ground: Terrain,
    frame: u16,
    groundwater: bool,
}

/// An independent city, including its terrain and financial accounts.
pub struct City {
    side: i32,
    tiles: Vec<Tile>,
    balance: i32,
    stats: Stats,
}

impl City {
    /// Creates the empty grass map used as the starting point for loading or generation.
    pub fn empty(side: i32) -> Result<Self, MapSizeError> {
        let area = side
            .checked_mul(side)
            .filter(|_| side >= 3)
            .ok_or(MapSizeError)?;
        let mut tiles = Vec::new();
        tiles
            .try_reserve_exact(area as usize)
            .map_err(|_| MapSizeError)?;
        tiles.resize(
            area as usize,
            Tile {
                ground: Terrain::Grass,
                frame: 0,
                groundwater: false,
            },
        );
        Ok(Self {
            side,
            tiles,
            balance: 0,
            stats: Stats::default(),
        })
    }

    /// Current treasury balance, including outstanding credit.
    pub fn balance(&self) -> i32 {
        self.balance
    }

    fn index(&self, point: Point) -> Option<usize> {
        if (0..self.side).contains(&point.x) && (0..self.side).contains(&point.y) {
            Some((point.y * self.side + point.x) as usize)
        } else {
            None
        }
    }

    /// Returns the ground at a map coordinate, including the outer border.
    pub fn terrain(&self, point: Point) -> Option<Terrain> {
        self.index(point).map(|index| self.tiles[index].ground)
    }

    /// Connection mask used to choose the terrain sprite.
    pub fn ground_frame(&self, point: Point) -> Option<u16> {
        self.index(point).map(|i| self.tiles[i].frame)
    }
    /// Underground water can remain after a surface water tile is drained.
    pub fn has_groundwater(&self, point: Point) -> Option<bool> {
        self.index(point).map(|i| self.tiles[i].groundwater)
    }

    // Visit the same interfaces as Map::desert_water_frontiers, including
    // unchanged adjacent pairs: their sprite bits may still be uninitialized.
    fn update_frontiers(&mut self, lo: Point, hi: Point) {
        for y in (lo.y - 1).max(1)..hi.y {
            for x in (lo.x - 1).max(1)..hi.x {
                let i = (y * self.side + x) as usize;
                if y >= lo.y && x + 1 < self.side - 1 {
                    self.connect_edge(i, i + 1, 2, 4);
                }
                if x >= lo.x && y + 1 < self.side - 1 {
                    self.connect_edge(i, i + self.side as usize, 1, 8);
                }
            }
        }
    }

    fn connect_edge(&mut self, a: usize, b: usize, mask_a: u16, mask_b: u16) {
        if self.tiles[a].ground == self.tiles[b].ground {
            self.tiles[a].frame |= mask_a;
            self.tiles[b].frame |= mask_b;
        } else {
            self.tiles[a].frame &= !mask_a;
            self.tiles[b].frame &= !mask_b;
        }
    }

    /// Quotes an action against the current city without changing it.
    pub fn preview(&self, action: Action, point: Point) -> Result<i32, ActionError> {
        let visible = 1..self.side - 1;
        if !visible.contains(&point.x) || !visible.contains(&point.y) {
            return Err(ActionError::OutsideMap(point));
        }
        let ground = self.tiles[(point.y * self.side + point.x) as usize].ground;
        match action {
            Action::Query => Ok(0),
            Action::Evacuate => Err(ActionError::NothingToEvacuate(point)),
            Action::Flood if ground == Terrain::Water => Err(ActionError::SpaceOccupied(point)),
            Action::Flood => Ok(1_000_000),
            Action::Bulldoze => Ok(match ground {
                Terrain::Water => 1_000_000,
                Terrain::Desert => 0,
                _ => 1,
            }),
        }
    }

    /// Revalidates an action and charges its cost before altering the map.
    pub fn apply(&mut self, action: Action, point: Point) -> Result<(), ActionError> {
        let cost = self.preview(action, point)?;
        let index = (point.y * self.side + point.x) as usize;
        let ground = match action {
            Action::Query => return Ok(()),
            Action::Evacuate => return Err(ActionError::NothingToEvacuate(point)),
            Action::Flood => Terrain::Water,
            Action::Bulldoze if self.tiles[index].ground == Terrain::Water => Terrain::Grass,
            Action::Bulldoze => Terrain::Desert,
        };
        let balance = self
            .balance
            .checked_sub(cost)
            .filter(|v| *v >= -2_000_000_000)
            .ok_or(ActionError::OutOfMoney)?;
        self.balance = balance;
        self.stats.expenses.construction.acc =
            self.stats.expenses.construction.acc.saturating_add(cost);
        if action == Action::Bulldoze && self.tiles[index].ground == Terrain::Desert {
            return Ok(());
        }
        self.tiles[index].ground = ground;
        if action == Action::Flood {
            self.tiles[index].frame = 0;
            self.tiles[index].groundwater = true;
        }
        self.update_frontiers(
            point,
            Point {
                x: point.x + 1,
                y: point.y + 1,
            },
        );
        Ok(())
    }
}
