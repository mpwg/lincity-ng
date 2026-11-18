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
/// An independent city, including its terrain and financial accounts.
pub struct City {
    side: i32,
    tiles: Vec<Terrain>,
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
        tiles.resize(area as usize, Terrain::Grass);
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
        self.index(point).map(|index| self.tiles[index])
    }

    /// Quotes an action against the current city without changing it.
    pub fn preview(&self, action: Action, point: Point) -> Result<i32, ActionError> {
        let visible = 1..self.side - 1;
        if !visible.contains(&point.x) || !visible.contains(&point.y) {
            return Err(ActionError::OutsideMap(point));
        }
        let ground = self.tiles[(point.y * self.side + point.x) as usize];
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
            Action::Bulldoze if self.tiles[index] == Terrain::Water => Terrain::Grass,
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
        self.tiles[index] = ground;
        Ok(())
    }
}
