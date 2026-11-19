//! Ordered construction decisions shared by the preview and command paths.
use super::*;
use BuildingKind::*;

/// Site, price and availability for one target in a construction batch.
#[derive(Debug, PartialEq, Eq)]
pub struct ActionQuote {
    pub anchor: Point,
    pub group: u16,
    pub size: u16,
    pub cost: i32,
    pub status: Result<(), ActionError>,
}

#[derive(Clone, Copy)]
struct SiteBuilding<'a> {
    kind: BuildingKind,
    anchor: Point,
    existing: Option<&'a Building>,
}

#[derive(Clone, Copy)]
struct Site<'a> {
    ground: Terrain,
    building: Option<SiteBuilding<'a>>,
}

/// Same-tool batches only need the treasury and the tiles changed by prior
/// commands. Build actions cannot remove rivers or change groundwater. The
/// river connection changes from flooding or demolition do not affect another
/// action with that same tool.
pub(super) struct Planner<'a> {
    city: &'a City,
    balance: i32,
    universities: u32,
    changes: BTreeMap<usize, Site<'a>>,
}

impl<'a> Planner<'a> {
    pub(super) fn new(city: &'a City) -> Self {
        Self {
            city,
            balance: city.balance,
            universities: city.stats.group_count[University.id() as usize],
            changes: BTreeMap::new(),
        }
    }

    fn index(&self, p: Point) -> usize {
        (p.y * self.city.side + p.x) as usize
    }

    fn site(&self, p: Point) -> Site<'a> {
        let index = self.index(p);
        self.changes.get(&index).copied().unwrap_or_else(|| Site {
            ground: self.city.tiles[index].ground,
            building: self
                .city
                .buildings
                .at(p)
                .and_then(|id| self.city.buildings.get(id))
                .map(|(anchor, building)| SiteBuilding {
                    kind: building.kind,
                    anchor,
                    existing: Some(building),
                }),
        })
    }

    fn check_build(&self, kind: BuildingKind, point: Point) -> Result<(), ActionError> {
        kind.can_build(
            self.city.tech_level,
            self.balance,
            self.city.stats.group_count[School.id() as usize],
            self.universities,
        )
        .map_err(ActionError::BuildDenied)?;
        let size = i32::from(kind.metadata().size);
        if point.x >= self.city.side - size || point.y >= self.city.side - size {
            return Err(ActionError::OutsideMap(point));
        }
        let bridge = matches!(kind, TrackBridge | RoadBridge | RailBridge);
        let transport = kind.is_transport();
        let mut groundwater = false;
        let mut ore = 0_i64;
        for p in footprint(point, kind.metadata().size) {
            let site = self.site(p);
            let existing = site.building.map(|b| b.kind);
            if (existing.is_some() || site.ground == Terrain::Water)
                && !(transport && existing == Some(PowerLine))
                && !(bridge && site.ground == Terrain::Water && existing != Some(kind))
            {
                return Err(ActionError::SpaceOccupied(point));
            }
            let tile = self.city.tiles[self.index(p)];
            groundwater |= tile.groundwater;
            ore += i64::from(tile.ore);
        }
        if matches!(kind, ParkPond | Parkland | Waterwell) && !groundwater {
            return Err(ActionError::DesertHere(point));
        }
        if kind == OreMine && ore < i64::from(crate::rules::industry::ORE_RESERVE) {
            return Err(ActionError::NoOre);
        }
        if kind == Port
            && !(point.y..point.y + size).all(|y| {
                self.city.tiles[self.index(Point {
                    x: point.x + size,
                    y,
                })]
                .river
            })
        {
            return Err(ActionError::RequiresRiver);
        }
        Ok(())
    }

    fn quote(&self, action: Action, point: Point) -> ActionQuote {
        let mut quote = ActionQuote {
            anchor: point,
            group: 0,
            size: 1,
            cost: 0,
            status: Ok(()),
        };
        let visible = 1..self.city.side - 1;
        if !visible.contains(&point.x) || !visible.contains(&point.y) {
            if let Action::Build(kind) = action {
                quote.group = kind.id();
                quote.size = kind.metadata().size;
            }
            quote.status = Err(ActionError::OutsideMap(point));
            return quote;
        }
        let site = self.site(point);
        if let Action::Build(kind) = action {
            let actual = actual_kind(kind, site.ground == Terrain::Water, self.city.tech_level);
            quote.group = actual.id();
            quote.size = actual.metadata().size;
            quote.cost = actual.cost(self.city.tech_level);
        } else {
            if let Some(building) = site.building {
                quote.anchor = building.anchor;
                quote.group = building.kind.id();
                quote.size = building.kind.metadata().size;
            } else {
                quote.group = site.ground.group();
            }
            quote.cost = match action {
                Action::Bulldoze => match site.building {
                    Some(building) => building.kind.metadata().demolition_cost,
                    None => match site.ground {
                        Terrain::Water => crate::rules::DRAIN_COST,
                        Terrain::Desert => 0,
                        _ => crate::rules::CLEAR_LAND_COST,
                    },
                },
                Action::Flood => crate::rules::FLOOD_COST,
                _ => 0,
            };
        }
        quote
    }

    pub(super) fn resolve(&self, action: Action, point: Point) -> ActionQuote {
        let mut quote = self.quote(action, point);
        if quote.status.is_err() {
            return quote;
        }
        let site = self.site(point);
        quote.status = match action {
            Action::Build(_) => {
                self.check_build(BuildingKind::from_id(quote.group).unwrap(), point)
            }
            Action::Query => Ok(()),
            Action::Evacuate => match site.building.and_then(|b| b.existing) {
                None => Err(ActionError::NothingToEvacuate(point)),
                Some(b) if b.never_evacuate => Err(ActionError::CannotEvacuate(point)),
                Some(_) => Ok(()),
            },
            Action::Flood if site.ground == Terrain::Water || site.building.is_some() => {
                Err(ActionError::SpaceOccupied(point))
            }
            Action::Flood => Ok(()),
            Action::Bulldoze => match site.building {
                Some(b) if b.kind == Fire => Err(ActionError::ProtectedFire(b.anchor)),
                Some(b) => match b.existing.map(|b| &b.operation) {
                    Some(Operation::Industry(industry::IndustryOperation::Tip(tip)))
                        if !tip.can_bulldoze() =>
                    {
                        Err(ActionError::NonemptyTip(b.anchor))
                    }
                    Some(Operation::Civic(civic)) if civic.incomplete_monument() => {
                        Err(ActionError::IncompleteMonument(b.anchor))
                    }
                    _ => Ok(()),
                },
                None => Ok(()),
            },
        };
        quote
    }

    pub(super) fn payable_quote(&self, action: Action, point: Point) -> ActionQuote {
        let mut quote = self.resolve(action, point);
        if quote.status.is_ok()
            && pays(action, self.site(point))
            && finance::balance_after_expense(self.balance, quote.cost).is_none()
        {
            quote.status = Err(ActionError::OutOfMoney);
        }
        quote
    }

    fn advance(&mut self, action: Action, p: Point, quote: &ActionQuote) {
        let mut site = self.site(p);
        if !pays(action, site) {
            return;
        }
        self.balance -= quote.cost;
        match action {
            Action::Build(_) => {
                let kind = BuildingKind::from_id(quote.group).unwrap();
                if kind == University {
                    self.universities += 1;
                }
                for cell in footprint(p, quote.size) {
                    let ground = built_ground(kind, self.site(cell).ground);
                    self.changes.insert(
                        self.index(cell),
                        Site {
                            ground,
                            building: Some(SiteBuilding {
                                kind,
                                anchor: p,
                                existing: None,
                            }),
                        },
                    );
                }
            }
            Action::Flood => {
                site.ground = Terrain::Water;
                self.changes.insert(self.index(p), site);
            }
            Action::Bulldoze => {
                if let Some(building) = site.building {
                    for cell in footprint(quote.anchor, quote.size) {
                        let mut tile = self.site(cell);
                        tile.building = (building.kind == Shanty).then_some(SiteBuilding {
                            kind: Fire,
                            anchor: cell,
                            existing: None,
                        });
                        if tile.building.is_some() {
                            tile.ground = built_ground(Fire, tile.ground);
                        }
                        self.changes.insert(self.index(cell), tile);
                    }
                    if let Some(Operation::Industry(industry::IndustryOperation::Ore(mine))) =
                        building.existing.map(|b| &b.operation)
                    {
                        for cell in mine.lake_points(&self.city.tiles) {
                            let index = self.index(cell);
                            self.changes.get_mut(&index).unwrap().ground = Terrain::Water;
                        }
                    }
                } else {
                    site.ground = cleared_ground(site.ground);
                    self.changes.insert(self.index(p), site);
                }
            }
            Action::Evacuate | Action::Query => unreachable!("these actions cost nothing"),
        }
    }
}

fn footprint(p: Point, size: u16) -> impl Iterator<Item = Point> {
    let size = i32::from(size);
    (p.y..p.y + size).flat_map(move |y| (p.x..p.x + size).map(move |x| Point { x, y }))
}

fn pays(action: Action, site: Site<'_>) -> bool {
    match action {
        Action::Query | Action::Evacuate => false,
        Action::Bulldoze => site.ground != Terrain::Desert || site.building.is_some(),
        _ => true,
    }
}

pub(super) fn built_ground(kind: BuildingKind, ground: Terrain) -> Terrain {
    match ground {
        Terrain::Water => Terrain::Water,
        Terrain::Desert => Terrain::Desert,
        _ if kind == PowerLine => Terrain::Grass,
        _ => Terrain::Desert,
    }
}

pub(super) fn cleared_ground(ground: Terrain) -> Terrain {
    match ground {
        Terrain::Water => Terrain::Grass,
        _ => Terrain::Desert,
    }
}

fn actual_kind(kind: BuildingKind, water: bool, tech: i32) -> BuildingKind {
    match (kind, water) {
        (Track | TrackBridge, true) => TrackBridge,
        (Track | TrackBridge, false) => Track,
        (Road | RoadBridge, true) => RoadBridge,
        (Road | RoadBridge, false) => Road,
        (Rail | RailBridge, true) => RailBridge,
        (Rail | RailBridge, false) => Rail,
        (kind, _) => kind.technology_variant(tech),
    }
}

impl City {
    pub(super) fn actual_kind(&self, kind: BuildingKind, point: Point) -> BuildingKind {
        actual_kind(
            kind,
            self.terrain(point) == Some(Terrain::Water),
            self.tech_level,
        )
    }

    /// Checks technology, treasury and school requirements without selecting a site.
    pub fn can_build(&self, kind: BuildingKind) -> Result<(), BuildDenial> {
        kind.technology_variant(self.tech_level).can_build(
            self.tech_level,
            self.balance,
            self.stats.group_count[School.id() as usize],
            self.stats.group_count[University.id() as usize],
        )
    }

    /// Price at a visible tile, including for an action the city rejects.
    pub fn quote(&self, action: Action, point: Point) -> Option<i32> {
        let quote = Planner::new(self).quote(action, point);
        quote.status.ok().map(|_| quote.cost)
    }

    /// Quotes an action against the current city without changing it.
    pub fn preview(&self, action: Action, point: Point) -> Result<i32, ActionError> {
        let quote = Planner::new(self).resolve(action, point);
        quote.status.map(|_| quote.cost)
    }

    /// Quotes one tool in input order against the effects of earlier accepted targets.
    pub fn preview_batch(&self, action: Action, targets: &[Point]) -> Vec<ActionQuote> {
        let mut plan = Planner::new(self);
        targets
            .iter()
            .map(|&point| {
                let quote = plan.payable_quote(action, point);
                if quote.status.is_ok() {
                    plan.advance(action, point, &quote);
                }
                quote
            })
            .collect()
    }

    /// Applies one tool in input order. Rejected targets leave later targets eligible.
    pub fn apply_batch(&mut self, action: Action, targets: &[Point]) -> Vec<ActionQuote> {
        targets
            .iter()
            .map(|&point| {
                let mut quote = Planner::new(self).payable_quote(action, point);
                if quote.status.is_ok() {
                    quote.status = self.execute_action(action, point, quote.cost);
                }
                quote
            })
            .collect()
    }
}
